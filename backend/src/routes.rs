use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    response::{IntoResponse, Response},
};


pub async fn handle_ws(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                // Handle the received message
                println!("Received message: {:?}", msg);
                // Echo the message back
                if let Err(e) = socket.send(msg).await {
                    eprintln!("Failed to send message: {}", e);
                }
            },
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break; 
            }
        }
    }
}

