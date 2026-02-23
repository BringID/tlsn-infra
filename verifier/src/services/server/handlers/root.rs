use std::time::{SystemTime, UNIX_EPOCH};
use axum::Json;

use serde::Serialize;
use crate::signer;

#[derive(Serialize)]
pub struct RootResponse {
    info: String,
    version: String,
    verifier_address: String,
    server_time: u64,
}

pub async fn handle() -> Json<RootResponse> {
    Json(
        RootResponse{
            info: "BringID Verifier".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            verifier_address: signer::get().address().to_string(),
            server_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_secs(),
        }
    )
}