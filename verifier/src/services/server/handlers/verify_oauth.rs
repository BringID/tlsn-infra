use std::str::FromStr;
use alloy::signers::{Signer, Signature};
use alloy::{sol};
use alloy::primitives::{Address, aliases::U256, hex, keccak256, B256};
use alloy::sol_types::{SolValue};
use alloy::sol_types::sol_data::Int;
use axum::{Json, http::StatusCode};
use rand::rng;
use serde::{Serialize, Deserialize};
use serde_json::Number;
use crate::signer;
use crate::tlsn;
use tracing::{info, error, instrument, warn, trace};
use tracing_subscriber::field::display::Messages;
use crate::helpers::{random_user_id_hash, registry_from_string, user_id_hash_from_bytes, verifier_response, VerifyResponse, OAUTH_SIGNER};
use crate::services::{OAuthVerificationManager, VerificationManager};

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
    registry: String
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

    let id_hash: B256;

    match std::env::var("ENV") {
        Ok(ref value) if value == "dev" => {
            id_hash = random_user_id_hash();
        }
        _ => {
            if recovered_address != *OAUTH_SIGNER {
                return Err((StatusCode::UNAUTHORIZED, "Wrong OAuth signer".to_string()));
            }
            id_hash = user_id_hash_from_bytes(
                payload.message.user_id.as_bytes()
            ).map_err(|e| {
                error!("invalid Semaphore Identity commitment: {e}");
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
        }
    }

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
        payload.credential_group_id,
        semaphore_identity_commitment,
        id_hash
    ).await
}