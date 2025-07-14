mod routes;
mod openai;
use axum::{routing::any,
        Router,
        };
use crate::routes::handle_ws;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", any(handle_ws));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");
        axum::serve(listener,app).await.unwrap();
}
