mod routes;
mod openai;

use axum::{routing::any,
        Router,
        };
use crate::routes::handle_ws;
use rustls::crypto::CryptoProvider;

#[tokio::main]
async fn main() {
    // Get Arc<CryptoProvider>, clone the inner value, then install
    let provider = CryptoProvider::get_default()
        .expect("No default CryptoProvider available");
    provider.clone().install_default().expect("Failed to install CryptoProvider");

    let app = Router::new().route("/ws", any(handle_ws));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");
        axum::serve(listener,app).await.unwrap();
}
