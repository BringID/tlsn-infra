use alloy::signers::Signer;
use axum::Json;
use serde::Serialize;
use crate::signer;
use crate::tlsn;

#[derive(Serialize)]
pub struct VerifyRequest {
    registry: String,
    verification: u32,
    semaphore_identity_commitment: u32
}

#[derive(Serialize)]
pub struct VerifyResponse {
    signature: String,
}

pub async fn handle() -> Json<VerifyResponse> {
    match signer::get().sign_message(b"mock").await {
        Ok(signature) => {
            Json(
                VerifyResponse{
                    signature: signature.to_string(),
                }
            )
        },
        Err(e) => { // TODO: HTTP error
            Json(
                VerifyResponse{
                    signature: "".to_string(),
                }
            )
        },
    }
}