use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::{Response, IntoResponse},
};

use crate::openai::OASocket; 
use tokio_tungstenite::tungstenite;

const FEYNMAN_PROMPT: &str = include_str!("../feynman_prompt.txt"); 


pub async fn handle_ws(ws: WebSocketUpgrade) -> Response {
    dotenvy::dotenv().ok();
    
    // Validate API key exists before accepting WebSocket connections
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.is_empty() && key != "your_openai_api_key_here" => key,
        _ => {
            eprintln!("ERROR: OPENAI_API_KEY is not set or is invalid");
            return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    
    println!("WebSocket connection request received, API key validated");
    ws.on_upgrade(move |socket| socket_task(socket, api_key))
}
async fn socket_task(mut browser_ws: WebSocket, api_key: String){
    println!("WebSocket connection established with browser, connecting to OpenAI...");
    
    let mut oa = match OASocket::connect(&api_key, FEYNMAN_PROMPT).await{
        Ok(s) => {
            println!("Successfully connected to OpenAI, ready to relay messages");
            s
        },
        Err(e) => {
            eprintln!("Failed to connect to OpenAI: {}", e);
            let error_msg = format!("Failed to connect to OpenAI: {}", e);
            let _ = browser_ws.send(Message::Text(error_msg.into())).await;
            let _ = browser_ws.send(Message::Close(None)).await;
            return;
        }
    };
    
    loop {
        tokio::select! {
            msg = browser_ws.recv() => {
                match msg {
                    Some(Ok(Message::Binary(buf))) => {
                        if let Err(e) = oa.send_audio(buf).await {
                            eprintln!("Failed to send audio to OpenAI: {}", e);
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
                        }
                    }
                     Some(Ok(Message::Close(_))) | None => {
                        println!("Browser WebSocket closed gracefully");
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
                        println!("Received text from OpenAI: {}", text);
                        if browser_ws.send(axum::extract::ws::Message::Text(text.to_string().into())).await.is_err() {
                            eprintln!("Failed to send text to browser");
                            let _ = browser_ws.send(Message::Close(None)).await;
                            oa.close().await.ok();
                            break;
                        }
                    }
                     Ok(tungstenite::Message::Close(_)) | Err(_) => {
                        eprintln!("OpenAI WebSocket closed or errored");
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
    println!("WebSocket session ended");
}

