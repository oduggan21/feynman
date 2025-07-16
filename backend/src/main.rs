mod routes;
mod openai;

use axum::{routing::any,
        Router,
        };
use crate::routes::handle_ws;

use rustls::crypto::ring;

#[tokio::main]
async fn main() {
    // Install the ring-based crypto provider so rustls can operate.
    ring::default_provider()
        .install_default()
        .expect("Failed to install CryptoProvider");

    // You can still use `provider_arc` later if you like.
    // …

    let app = Router::new().route("/ws", any(handle_ws));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");
        axum::serve(listener,app).await.unwrap();
}
