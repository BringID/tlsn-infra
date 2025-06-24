use axum::{routing::get, Router};
use std::error::Error;
use super::handlers::{root, verify};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(root::handle))
        .route("/verify", get(verify::handle));

    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("🔐 zkBring TLSNotary Verifier");

    axum::serve(listener, app).await?;
    Ok(())
}
