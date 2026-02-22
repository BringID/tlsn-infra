use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use alloy::{hex, sol};
use alloy::primitives::{keccak256, B256, U256};
use alloy::sol_types::SolValue;
use alloy::transports::http::reqwest::StatusCode;
use alloy::signers::Signer;
use axum::Json;
use serde::Serialize;
use tracing::{error, info};
use crate::helpers::registry_from_string;
use crate::signer;

fn serialize_u256_as_string<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn serialize_u256_as_number<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let n: u64 = value.to::<u64>();
    serializer.serialize_u64(n)
}

sol! {
    #[derive(Serialize)]
    struct Attestation {
        address registry;
        #[serde(rename = "chain_id")]
        #[serde(serialize_with = "serialize_u256_as_number")]
        uint256 chainId;
        #[serde(rename = "credential_group_id")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 credentialGroupId;
        #[serde(rename = "credential_id")]
        bytes32 credentialId;
        #[serde(rename = "app_id")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 appId;
        #[serde(rename = "semaphore_identity_commitment")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 semaphoreIdentityCommitment;
        #[serde(rename = "issued_at")]
        #[serde(serialize_with = "serialize_u256_as_number")]
        uint256 issuedAt;
    }
}

#[derive(Serialize)]
pub struct VerifyResponse {
    attestation: Attestation,
    verifier_hash: String,
    signature: String,
}

const VALID_CHAIN_IDS: &[u64] = &[8453, 84532];

pub async fn verifier_response(
    registry_address: String,
    chain_id: String,
    credential_group_id: String,
    app_id: String,
    semaphore_identity_commitment: U256,
    credential_id: B256,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    let registry = registry_from_string(registry_address)
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let chain_id: u64 = chain_id.parse().map_err(|e: std::num::ParseIntError| {
        error!("invalid chain_id: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    if !VALID_CHAIN_IDS.contains(&chain_id) {
        error!("unsupported chain_id: {chain_id}");
        return Err((StatusCode::BAD_REQUEST, format!("unsupported chain_id: {chain_id}")));
    }

    let app_id = U256::from_str(app_id.as_str()).map_err(|e| {
        error!("invalid app_id: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    let issued_at = U256::from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_secs()
    );

    let attestation = Attestation {
        registry,
        chainId: U256::from(chain_id),
        credentialGroupId: U256::from_str(credential_group_id.as_str()).map_err(|e| {(StatusCode::BAD_REQUEST, e.to_string())})?,
        credentialId: credential_id,
        appId: app_id,
        semaphoreIdentityCommitment: semaphore_identity_commitment,
        issuedAt: issued_at,
    };
    let message = keccak256(attestation.abi_encode());
    let signature = signer::get().sign_message(message.as_slice())
        .await
        .map_err(|e| {
            error!("unexpected error during message signing: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    info!("verification completed");
    Ok(
        Json(VerifyResponse {
            attestation,
            verifier_hash: hex::encode_prefixed(message),
            signature: signature.to_string(),
        })
    )
}
