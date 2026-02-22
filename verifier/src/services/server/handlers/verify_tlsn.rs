use std::str::FromStr;
use alloy::primitives::{aliases::U256, hex};
use axum::{Json, http::StatusCode};
use serde::Deserialize;
use crate::tlsn;
use crate::helpers::{verifier_response, VerifyResponse};
use tracing::{info, error, instrument, warn, trace};

#[derive(Deserialize, Debug)]
pub struct VerifyRequest {
    tlsn_presentation: String,
    registry: String,
    chain_id: String,
    credential_group_id: String,
    app_id: String,
    semaphore_identity_commitment: String,
}

#[instrument(
    name="handler",
    skip(payload),
    fields(
        group = %payload.credential_group_id,
        commitment = %payload.semaphore_identity_commitment
    )
)]
pub async fn handle(
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    info!("verification started");
    trace!("{:?}", &payload);
    let presentation = hex::decode(payload.tlsn_presentation.as_str())
        .map_err(|e| {
            error!("Presentation decoding failed: {e}");
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;
    let presentation = bincode::deserialize(&presentation)
        .map_err(|e| {
            error!("Presentation deserialization failed: {e}");
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let credential_id = tlsn::verify_proof(presentation, &payload.credential_group_id, &payload.app_id).await
        .map_err(|e| {
            warn!("verification failed");
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|e| {
        error!("invalid Semaphore Identity commitment: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    verifier_response(
        payload.registry,
        payload.chain_id,
        payload.credential_group_id,
        payload.app_id,
        semaphore_identity_commitment,
        credential_id,
    ).await
}
