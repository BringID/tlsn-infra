use axum::{routing::get, routing::post, Router};
use std::error::Error;
use super::handlers::{root, verify};
use crate::config;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(root::handle))
        .route("/verify", post(verify::handle));
    let port = &config::get().port;

    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("zkBring TLSNotary Verifier");
    println!("Listening on port {port}");

    axum::serve(listener, app).await?;
    Ok(())
}
