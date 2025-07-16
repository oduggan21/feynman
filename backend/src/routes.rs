use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::{Response},
};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::openai::OASocket; 
use tokio_tungstenite::tungstenite;

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
    
    let mut conversation_step = 0;
    
    loop {
        match browser_ws.recv().await {
            Some(Ok(Message::Binary(audio_data))) => {
                eprintln!("Test mode: received {} bytes of audio data", audio_data.len());
                
                // Simulate different responses based on conversation step
                let mock_response = match conversation_step {
                    0 => {
                        conversation_step += 1;
                        r#"{"type":"response.audio_transcript.done","transcript":"Hello there! I am Feynman, your AI learning companion. What topic would you like to teach me today?"}"#
                    }
                    1 => {
                        conversation_step += 1;
                        r#"{"type":"response.audio_transcript.done","transcript":"photosynthesis"}"#
                    }
                    2 => {
                        conversation_step += 1;
                        r#"{"type":"response.audio_transcript.done","transcript":"Awesome! I'm ready to start learning about photosynthesis whenever you are. Please go ahead and teach me everything you know about this topic."}"#
                    }
                    3 => {
                        conversation_step += 1;
                        r#"{"type":"response.audio_transcript.done","transcript":"What role does chlorophyll play in capturing light energy?"}"#
                    }
                    4 => {
                        conversation_step += 1;
                        r#"{"type":"response.audio_transcript.done","transcript":"Can you explain the difference between the light-dependent and light-independent reactions?"}"#
                    }
                    _ => {
                        r#"{"type":"response.audio_transcript.done","transcript":"Excellent work! You've demonstrated a solid understanding of photosynthesis. Well done!"}"#
                    }
                };
                
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
                    eprintln!("Test mode: simulating audio commit and response creation");
                    // Send response done event
                    let response_done = r#"{"type":"response.done"}"#;
                    if let Err(e) = browser_ws.send(Message::Text(response_done.into())).await {
                        eprintln!("Failed to send response done: {}", e);
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

    // Send initial greeting by creating a response
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
                            let should_commit = {
                                let ctx = context.lock().await;
                                ctx.audio_buffer_has_data
                            };
                            
                            if should_commit {
                                if let Err(e) = oa.commit_audio_buffer().await {
                                    eprintln!("Failed to commit audio buffer: {}", e);
                                }
                                if let Err(e) = oa.create_response().await {
                                    eprintln!("Failed to create response: {}", e);
                                }
                                
                                // Reset audio buffer tracking
                                {
                                    let mut ctx = context.lock().await;
                                    ctx.audio_buffer_has_data = false;
                                }
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
                        
                        // Update conversation state based on OpenAI response
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(response_type) = parsed.get("type").and_then(|t| t.as_str()) {
                                match response_type {
                                    "response.audio_transcript.done" => {
                                        if let Some(transcript) = parsed.get("transcript").and_then(|t| t.as_str()) {
                                            eprintln!("User said: {}", transcript);
                                            update_conversation_state(&context, transcript).await;
                                        }
                                    }
                                    "response.done" => {
                                        eprintln!("OpenAI response completed");
                                    }
                                    _ => {
                                        // Handle other response types as needed
                                    }
                                }
                            }
                        }
                       
                        if browser_ws.send(axum::extract::ws::Message::Text(text.to_string().into())).await.is_err() {
                            eprintln!("Failed to send text to browser");
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
                        }
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

async fn update_conversation_state(context: &Arc<Mutex<ConversationContext>>, transcript: &str) {
    let mut ctx = context.lock().await;
    
    match ctx.state {
        ConversationState::Initial => {
            // After greeting, look for topic in user's response
            if !transcript.trim().is_empty() && transcript.len() > 10 {
                ctx.topic = Some(transcript.to_string());
                ctx.state = ConversationState::WaitingForTopic;
                eprintln!("Detected topic: {}", transcript);
            }
        }
        ConversationState::WaitingForTopic => {
            // Topic acknowledged, ready for teaching
            ctx.state = ConversationState::ReadyToTeach;
            eprintln!("Ready to receive teaching about: {:?}", ctx.topic);
        }
        ConversationState::ReadyToTeach => {
            // User starts teaching
            ctx.state = ConversationState::Teaching;
            eprintln!("User started teaching");
        }
        ConversationState::Teaching => {
            // User finished teaching, analyze and generate questions
            ctx.state = ConversationState::Analyzing;
            eprintln!("User finished teaching, moving to analysis");
        }
        ConversationState::Analyzing => {
            // Move to questioning phase
            ctx.state = ConversationState::Questioning;
            ctx.current_question_index = 0;
            eprintln!("Moving to questioning phase");
        }
        ConversationState::Questioning => {
            // Handle question responses
            ctx.current_question_index += 1;
            eprintln!("Question {} answered", ctx.current_question_index);
            
            // For now, assume 3 questions max
            if ctx.current_question_index >= 3 {
                ctx.state = ConversationState::Complete;
                eprintln!("All questions answered, conversation complete");
            }
        }
        ConversationState::Complete => {
            eprintln!("Conversation is complete");
        }
    }
}

