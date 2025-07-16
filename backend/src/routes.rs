use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::{Response},
};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::openai::OASocket; 
use tokio_tungstenite::tungstenite;
use base64;

const FEYNMAN_PROMPT: &str = include_str!("../feynman_prompt.txt"); 

#[derive(Debug, Clone, PartialEq)]
enum ConversationState {
    Initial,
    WaitingForTopic,
    ReadyToTeach,
    Teaching,
    Analyzing,
    Questioning,
    Complete,
}

struct ConversationContext {
    state: ConversationState,
    topic: Option<String>,
    questions: Vec<String>,
    current_question_index: usize,
    audio_buffer_has_data: bool,
} 


pub async fn handle_ws(ws: WebSocketUpgrade) -> Response {
    dotenvy::dotenv().ok();
    ws.on_upgrade(socket_task)
}
async fn socket_task(mut browser_ws: WebSocket){
    let key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("OPENAI_API_KEY environment variable not set");
            let _ = browser_ws.send(Message::Text("Error: OPENAI_API_KEY not configured".into())).await;
            let _ = browser_ws.send(Message::Close(None)).await;
            return;
        }
    };
    
    // Check for test mode
    let test_mode = env::var("TEST_MODE").unwrap_or_default() == "true";
    
    if test_mode {
        eprintln!("Running in TEST_MODE - simulating OpenAI connection");
        let _ = browser_ws.send(Message::Text("TEST_MODE: Simulated OpenAI connection".into())).await;
        socket_task_test_mode(browser_ws).await;
        return;
    }
    
    // Validate API key format
    if !key.starts_with("sk-") {
        eprintln!("Invalid OpenAI API key format");
        let _ = browser_ws.send(Message::Text("Error: Invalid API key format".into())).await;
        let _ = browser_ws.send(Message::Close(None)).await;
        return;
    }
    
    let oa = match OASocket::connect(&key, FEYNMAN_PROMPT).await{
        Ok(s) => {
            eprintln!("Successfully connected to OpenAI");
            if let Err(e) = browser_ws.send(Message::Text("Connected to OpenAI".into())).await {
                eprintln!("Failed to send connection status: {}", e);
                return;
            }
            s
        },
        Err(e) => {
            eprintln!("Failed to connect to OpenAI: {}", e);
            let error_msg = format!("OpenAI connection failed: {}", e);
            let _ = browser_ws.send(Message::Text(error_msg.into())).await;
            
            // Keep the WebSocket open and wait for browser commands instead of closing
            eprintln!("Keeping browser WebSocket open despite OpenAI failure");
            loop {
                match browser_ws.recv().await {
                    Some(Ok(Message::Close(_))) => {
                        eprintln!("Browser WebSocket closed after OpenAI failure");
                        break;
                    }
                    Some(Ok(Message::Text(text))) => {
                        if text == "retry_openai" {
                            eprintln!("Retrying OpenAI connection...");
                            match OASocket::connect(&key, FEYNMAN_PROMPT).await {
                                Ok(new_oa) => {
                                    eprintln!("OpenAI reconnection successful");
                                    let _ = browser_ws.send(Message::Text("OpenAI reconnected".into())).await;
                                    // Continue with the new OpenAI connection
                                    socket_task_with_openai(browser_ws, new_oa).await;
                                    return;
                                }
                                Err(e) => {
                                    eprintln!("OpenAI reconnection failed: {}", e);
                                    let _ = browser_ws.send(Message::Text(format!("Reconnection failed: {}", e).into())).await;
                                }
                            }
                        } else {
                            let _ = browser_ws.send(Message::Text("OpenAI unavailable - send 'retry_openai' to retry".into())).await;
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error while OpenAI disconnected: {}", e);
                        break;
                    }
                    None => {
                        eprintln!("Browser WebSocket connection ended");
                        break;
                    }
                }
            }
            return;
        }
    };
    
    socket_task_with_openai(browser_ws, oa).await;
}

async fn socket_task_test_mode(mut browser_ws: WebSocket) {
    eprintln!("Test mode: simulating OpenAI responses");
    
    loop {
        match browser_ws.recv().await {
            Some(Ok(Message::Binary(audio_data))) => {
                eprintln!("Test mode: received {} bytes of audio data", audio_data.len());
                
                // Simulate OpenAI text response
                let mock_response = r#"{"type":"response.audio.delta","delta":"dGVzdCBhdWRpbyBkYXRh"}"#;
                if let Err(e) = browser_ws.send(Message::Text(mock_response.into())).await {
                    eprintln!("Failed to send mock response: {}", e);
                    break;
                }
                
                // Simulate some audio response (just echo back some test data)
                let mock_audio: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
                if let Err(e) = browser_ws.send(Message::Binary(mock_audio.into())).await {
                    eprintln!("Failed to send mock audio: {}", e);
                    break;
                }
            }
            Some(Ok(Message::Text(text))) => {
                eprintln!("Test mode: received text message: {}", text);
                
                if text == "commit_audio" {
                    // Simulate OpenAI text response
                    let mock_response = r#"{"type":"response.audio.delta","delta":"dGVzdCBhdWRpbyBkYXRh"}"#;
                    if let Err(e) = browser_ws.send(Message::Text(mock_response.into())).await {
                        eprintln!("Failed to send mock response: {}", e);
                        break;
                    }
                    
                    // Simulate audio response with actual PCM16 data (1 second of 440Hz tone)
                    let sample_rate = 16000;
                    let duration = 1.0; // 1 second
                    let frequency = 440.0; // A4 note
                    let samples = (sample_rate as f32 * duration) as usize;
                    let mut audio_data = Vec::with_capacity(samples * 2); // 2 bytes per sample for PCM16
                    
                    for i in 0..samples {
                        let t = i as f32 / sample_rate as f32;
                        let amplitude = 0.3; // Reduce volume
                        let sample = (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin() * 32767.0) as i16;
                        audio_data.extend_from_slice(&sample.to_le_bytes());
                    }
                    
                    eprintln!("Test mode: sending {} bytes of mock audio", audio_data.len());
                    if let Err(e) = browser_ws.send(Message::Binary(audio_data.into())).await {
                        eprintln!("Failed to send mock audio: {}", e);
                        break;
                    }
                } else {
                    let response = format!("Test mode echo: {}", text);
                    if let Err(e) = browser_ws.send(Message::Text(response.into())).await {
                        eprintln!("Failed to send text echo: {}", e);
                        break;
                    }
                }
            }
            Some(Ok(Message::Close(_))) => {
                eprintln!("Browser WebSocket closed in test mode");
                break;
            }
            Some(Ok(_)) => {
                // Ignore other message types
            }
            Some(Err(e)) => {
                eprintln!("WebSocket error in test mode: {}", e);
                break;
            }
            None => {
                eprintln!("Browser WebSocket connection ended in test mode");
                break;
            }
        }
    }
}

async fn socket_task_with_openai(mut browser_ws: WebSocket, mut oa: OASocket) {
    let context = Arc::new(Mutex::new(ConversationContext {
        state: ConversationState::Initial,
        topic: None,
        questions: Vec::new(),
        current_question_index: 0,
        audio_buffer_has_data: false,
    }));

    // Send initial greeting
    let _ = oa.create_response().await;

    loop {
        tokio::select! {
            msg = browser_ws.recv() => {
                match msg {
                    Some(Ok(Message::Binary(buf))) => {
                        eprintln!("Received audio frame from browser: {} bytes", buf.len());
                        
                        // Track that we have audio data
                        {
                            let mut ctx = context.lock().await;
                            ctx.audio_buffer_has_data = true;
                        }
                        
                        if let Err(e) = oa.send_audio(buf).await {
                            eprintln!("Failed to send audio: {}", e);
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        eprintln!("Received text from browser: {}", text);
                        
                        // Handle special browser commands
                        if text == "commit_audio" {
                            eprintln!("Processing commit_audio command");
                            let should_commit = {
                                let ctx = context.lock().await;
                                let has_data = ctx.audio_buffer_has_data;
                                eprintln!("Audio buffer has data: {}", has_data);
                                has_data
                            };
                            
                            if should_commit {
                                eprintln!("Committing audio buffer and creating response");
                                if let Err(e) = oa.commit_audio_buffer().await {
                                    eprintln!("Failed to commit audio buffer: {}", e);
                                } else {
                                    eprintln!("Audio buffer committed successfully");
                                }
                                if let Err(e) = oa.create_response().await {
                                    eprintln!("Failed to create response: {}", e);
                                } else {
                                    eprintln!("Response creation requested successfully");
                                }
                                
                                // Reset audio buffer tracking
                                {
                                    let mut ctx = context.lock().await;
                                    ctx.audio_buffer_has_data = false;
                                    eprintln!("Reset audio buffer tracking");
                                }
                            } else {
                                eprintln!("No audio data to commit, skipping");
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        eprintln!("Browser WebSocket closed");
                        oa.close().await.ok();
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket receive error: {}", e);
                        break;
                    }
                    None => {
                        eprintln!("Browser WebSocket connection ended");
                        oa.close().await.ok();
                        break;
                    }
                }
            },
            oa_msg = oa.next() => {
                match oa_msg {
                    Ok(tungstenite::Message::Binary(audio)) => {
                        if browser_ws.send(Message::Binary(audio)).await.is_err() {
                            eprintln!("Failed to send audio to browser");
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
                        }
                    }
                    Ok(tungstenite::Message::Text(text)) => {
                        eprintln!("Received text from OpenAI: {}", text);
                        
                        // Parse the JSON response to extract audio data
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(event_type) = json_value.get("type").and_then(|t| t.as_str()) {
                                match event_type {
                                    "response.audio.delta" => {
                                        if let Some(delta) = json_value.get("delta").and_then(|d| d.as_str()) {
                                            // Decode base64 audio data
                                            if let Ok(audio_bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, delta) {
                                                eprintln!("Sending {} bytes of audio to browser", audio_bytes.len());
                                                if browser_ws.send(Message::Binary(audio_bytes.into())).await.is_err() {
                                                    eprintln!("Failed to send audio to browser");
                                                    let _ = browser_ws.send(Message::Close(None)).await;
                                                    oa.close().await.ok();
                                                    break;
                                                }
                                            } else {
                                                eprintln!("Failed to decode base64 audio data");
                                            }
                                        }
                                    }
                                    _ => {
                                        // For non-audio events, send the text to browser for debugging
                                        if browser_ws.send(axum::extract::ws::Message::Text(text.to_string().into())).await.is_err() {
                                            eprintln!("Failed to send text to browser");
                                            let _ = browser_ws.send(Message::Close(None)).await;
                                            oa.close().await.ok();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Handle conversation state based on OpenAI response
                        let _should_update_state = {
                            let _ctx = context.lock().await;
                            // Parse the response and update conversation state if needed
                            // This is where we would implement the Feynman tutor logic
                            false
                        };
                    }
                     Ok(tungstenite::Message::Close(_)) => {
                        eprintln!("OpenAI WebSocket closed");
                        let _ = browser_ws.send(Message::Close(None)).await;
                        oa.close().await.ok();
                        break;
                    }
                    Err(e) => {
                        eprintln!("OpenAI WebSocket errored: {:?}", e);
                        let _ = browser_ws.send(Message::Close(None)).await;
                        oa.close().await.ok();
                        break;
                    }
                    Ok(_) => {
                        // Ignore other message types
                    }
                }
            }
            }
        }
    }

