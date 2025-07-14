use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket, Message},
    response::{IntoResponse, Response},
};

use futures_util::{SinkExt, StreamExt};
use std::env;

use crate::openai::OASocket; 

const FEYNMAN_PROMPT: &str = include_str!("../feynman_prompt.txt"); 

pub async fn handle_ws(ws: WebSocketUpgrade) -> Response {
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
        tokio::select!{
            Some(Ok(Message::Binary(buf))) = browser_ws.recv() => {
                if let Err(e) = oa.send_audio(buf).await {
                    eprintln!("Failed to send audio: {}", e);
                    break;
                }
            }
            Ok(Message::Binary(audio)) = oa.next() => {
                if browser_ws.send(Message::Binary(audio)).await.is_err() {
                    eprintln!("Failed to send audio to browser");
                    break;
                }
            }
            else => break;
        }
    }

}