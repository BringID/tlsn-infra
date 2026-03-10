use std::str::FromStr;
use alloy::signers::Signature;
use alloy::{sol};
use alloy::primitives::{aliases::U256, keccak256, B256};
use alloy::sol_types::SolValue;
use axum::extract::rejection::JsonRejection;
use axum::Json;
use serde::{Serialize, Deserialize};
use tracing::{info, error, instrument, trace};
use crate::helpers::{random_credential_id, credential_id_from_bytes, verifier_response, VerifyResponse, ApiError, ErrorCode, get_oauth_signer};
use crate::services::{OAuthVerificationManager};

sol! {
    #[derive(Deserialize, Serialize, Debug)]
    struct OauthMessage {
        string domain;
        #[serde(rename = "userId")]
        string user_id;
        uint256 score;
        uint256 timestamp;
    }
}

#[derive(Deserialize, Debug)]
pub struct VerifyRequest {
    message: OauthMessage,
    signature: String,
    semaphore_identity_commitment: String,
    credential_group_id: String,
    app_id: String,
    registry: String,
    chain_id: u64,
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
        domain = %payload.message.domain,
        user = %payload.message.user_id
    )
)]
async fn handle_inner(
    payload: VerifyRequest,
) -> Result<Json<VerifyResponse>, ApiError> {
    info!("verification started");
    trace!("{:?}", &payload);

    let message = keccak256(payload.message.abi_encode_params());

    // Parse signature
    let signature = payload.signature.parse::<Signature>()
        .map_err(|e| {
            error!("failed to parse signature: {}", e);
            ApiError::bad_request(ErrorCode::SignatureParseFailed, e)
        })?;

    // Recover signer address
    let recovered_address = signature.recover_address_from_msg(message.as_slice())
        .map_err(|e| {
            error!("failed to recover address: {}", e);
            ApiError::bad_request(ErrorCode::AddressRecoveryFailed, e)
        })?;

    let app_id_u256 = U256::from_str(payload.app_id.as_str()).map_err(|e| {
        error!("invalid app_id: {e}");
        ApiError::bad_request(ErrorCode::InvalidAppId, e)
    })?;

    // Production: always validate signer + deterministic credential_id
    // Dev/staging: controlled by STAGING_VALIDATE_OAUTH_SIGNER and STAGING_USE_RANDOM_ID
    let is_dev = matches!(std::env::var("ENV"), Ok(ref v) if v == "dev");

    if !is_dev || std::env::var("STAGING_VALIDATE_OAUTH_SIGNER").is_ok_and(|v| v == "true") {
        let expected_signer = get_oauth_signer(&payload.credential_group_id)
            .ok_or_else(|| {
                error!("no OAuth signer configured for credential_group_id {}", payload.credential_group_id);
                ApiError::unauthorized(ErrorCode::WrongOauthSigner, "No OAuth signer configured for this credential group")
            })?;
        if recovered_address != *expected_signer {
            return Err(ApiError::unauthorized(ErrorCode::WrongOauthSigner, "Wrong OAuth signer"));
        }
    }

    let credential_id: B256 = if is_dev && std::env::var("STAGING_USE_RANDOM_ID").is_ok_and(|v| v == "true") {
        random_credential_id()
    } else {
        credential_id_from_bytes(
            payload.message.user_id.as_bytes(),
            &app_id_u256,
        ).map_err(|e| {
            error!("credential ID computation failed: {e}");
            ApiError::bad_request(ErrorCode::CredentialIdFailed, e)
        })?
    };

    let verification = OAuthVerificationManager::get(&payload.credential_group_id)
        .ok_or_else(|| {
            error!("verification is not found");
            ApiError::internal(ErrorCode::VerificationNotFound, "verification is not found")
        })?
        .clone();

    verification.check(
        payload.message.domain,
        payload.message.score.to::<i32>()
    ).await.map_err(
        |e| { ApiError::bad_request(ErrorCode::VerificationCheckFailed, e) }
    )?;

    // Build Verifier message
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
