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
    let key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let mut oa = match OASocket::connect(&key, FEYNMAN_PROMPT).await{
        Ok(s) => s,
        Err(e) =>{eprintln!("Failed to connect to OpenAI: {}", e);
            return; }
    };
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
                     Some(Ok(Message::Close(_))) | None => {
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
                    None => break,
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
    }

