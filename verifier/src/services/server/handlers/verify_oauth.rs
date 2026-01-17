use std::str::FromStr;
use alloy::signers::{Signer, Signature};
use alloy::{sol};
use alloy::primitives::{Address, aliases::U256, hex, keccak256};
use alloy::sol_types::{SolValue};
use alloy::sol_types::sol_data::Int;
use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use serde_json::Number;
use crate::signer;
use crate::tlsn;
use tracing::{info, error, instrument, warn, trace};
use tracing_subscriber::field::display::Messages;
use crate::helpers::{registry_from_string, user_id_hash_from_bytes, verifier_response, VerifyResponse};

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

    info!("Recovered address: {}", recovered_address);

    // Build Verifier message
    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|e| {
        error!("invalid Semaphore Identity commitment: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    let id_hash = user_id_hash_from_bytes(
        payload.message.user_id.as_bytes()
    ).map_err(|e| {
        error!("invalid Semaphore Identity commitment: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;;

    verifier_response(
        payload.registry,
        payload.credential_group_id,
        semaphore_identity_commitment,
        id_hash
    ).await
}