mod routes;
mod openai;

use axum::{routing::any,
        Router,
        };
use crate::routes::handle_ws;

use std::sync::Arc;
use rustls::crypto::CryptoProvider;

#[tokio::main]
async fn main() {
    // 1) Grab the Arc<CryptoProvider>
    let provider_arc = CryptoProvider::get_default()
        .expect("No default CryptoProvider available");

    // 2) Clone the inner `CryptoProvider` (leaves `provider_arc` intact)
    let provider_clone = <CryptoProvider as Clone>::clone(&*provider_arc);
    //                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // this calls the `Clone` impl on `CryptoProvider`,
    // not on the Arc itself

    // 3) Move that clone into install_default()
    provider_clone
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
