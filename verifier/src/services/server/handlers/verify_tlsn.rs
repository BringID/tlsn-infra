use std::str::FromStr;
use alloy::primitives::{aliases::U256, hex};
use axum::extract::rejection::JsonRejection;
use axum::Json;
use serde::Deserialize;
use crate::tlsn;
use crate::helpers::{verifier_response, VerifyResponse, ApiError, ErrorCode};
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

pub async fn handle(
    payload: Result<Json<VerifyRequest>, JsonRejection>,
) -> Result<Json<VerifyResponse>, ApiError> {
    let Json(payload) = payload.map_err(ApiError::from)?;
    handle_inner(payload).await
}

#[instrument(
    name="handler",
    skip(payload),
    fields(
        group = %payload.credential_group_id,
        commitment = %payload.semaphore_identity_commitment
    )
)]
async fn handle_inner(
    payload: VerifyRequest,
) -> Result<Json<VerifyResponse>, ApiError> {
    info!("verification started");
    trace!("{:?}", &payload);
    let presentation = hex::decode(payload.tlsn_presentation.as_str())
        .map_err(|e| {
            error!("Presentation decoding failed: {e}");
            ApiError::bad_request(ErrorCode::PresentationDecodeFailed, e)
        })?;
    let presentation = bincode::deserialize(&presentation)
        .map_err(|e| {
            error!("Presentation deserialization failed: {e}");
            ApiError::bad_request(ErrorCode::PresentationDeserializeFailed, e)
        })?;

    let credential_id = tlsn::verify_proof(presentation, &payload.credential_group_id, &payload.app_id).await
        .map_err(|e| {
            warn!("verification failed");
            ApiError::bad_request(ErrorCode::ProofVerificationFailed, e)
        })?;

    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|e| {
        error!("invalid Semaphore Identity commitment: {e}");
        ApiError::bad_request(ErrorCode::InvalidSemaphoreCommitment, e)
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
