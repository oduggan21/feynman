use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::{Response},
};
use std::env;

use crate::openai::OASocket; 
use tokio_tungstenite::tungstenite;

const FEYNMAN_PROMPT: &str = include_str!("../feynman_prompt.txt"); 


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
                let response = format!("Test mode echo: {}", text);
                if let Err(e) = browser_ws.send(Message::Text(response.into())).await {
                    eprintln!("Failed to send text echo: {}", e);
                    break;
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
    loop {
        tokio::select! {
            msg = browser_ws.recv() => {
                match msg {
                    Some(Ok(Message::Binary(buf))) => {
                        if let Err(e) = oa.send_audio(buf).await {
                            eprintln!("Failed to send audio: {}", e);
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
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

