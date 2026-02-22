use std::str::FromStr;
use alloy::signers::Signature;
use alloy::{sol};
use alloy::primitives::{aliases::U256, keccak256, B256};
use alloy::sol_types::SolValue;
use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use tracing::{info, error, instrument, trace};
use crate::helpers::{random_credential_id, credential_id_from_bytes, verifier_response, VerifyResponse, OAUTH_SIGNER};
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
    chain_id: String,
}

#[instrument(
    name="handler",
    skip(payload),
    fields(
        domain = %payload.message.domain,
        user = %payload.message.user_id
    )
)]
pub async fn handle(
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    info!("verification started");
    trace!("{:?}", &payload);

    let message = keccak256(payload.message.abi_encode_params());

    // Parse signature
    let signature = payload.signature.parse::<Signature>()
        .map_err(|e| {
            error!("failed to parse signature: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    // Recover signer address
    let recovered_address = signature.recover_address_from_msg(message.as_slice())
        .map_err(|e| {
            error!("failed to recover address: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let app_id_u256 = U256::from_str(payload.app_id.as_str()).map_err(|e| {
        error!("invalid app_id: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    let credential_id: B256 = match std::env::var("ENV") {
        Ok(ref value) if value == "dev" => {
            random_credential_id()
        }
        _ => {
            if recovered_address != *OAUTH_SIGNER {
                return Err((StatusCode::UNAUTHORIZED, "Wrong OAuth signer".to_string()));
            }
            credential_id_from_bytes(
                payload.message.user_id.as_bytes(),
                &app_id_u256,
            ).map_err(|e| {
                error!("credential ID computation failed: {e}");
                (StatusCode::BAD_REQUEST, e.to_string())
            })?
        }
    };

    let verification = OAuthVerificationManager::get(&payload.credential_group_id)
        .ok_or_else(|| {
            error!("verification is not found");
            (StatusCode::INTERNAL_SERVER_ERROR, "verification is not found".to_string())
        })?
        .clone();

    verification.check(
        payload.message.domain,
        payload.message.score.to::<i32>()
    ).await.map_err(
        |e| { (StatusCode::BAD_REQUEST, e.to_string()) }
    )?;

    // Build Verifier message
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
