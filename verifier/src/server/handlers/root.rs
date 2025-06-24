use axum::Json;

use serde::Serialize;
use crate::signer;

#[derive(Serialize)]
pub struct RootResponse {
    info: String,
    version: String,
    verifier_address: String,
}

pub async fn handle() -> Json<RootResponse> {
    Json(
        RootResponse{
            info: "zkBring TLSNotary Verifier".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            verifier_address: signer::get().address().to_string(),
        }
    )
}