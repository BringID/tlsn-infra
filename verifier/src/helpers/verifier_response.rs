use std::str::FromStr;
use alloy::{hex, sol};
use alloy::primitives::{keccak256, B256, U256};
use alloy::sol_types::SolValue;
use alloy::transports::http::reqwest::StatusCode;
use alloy::signers::{Signer, Signature};
use axum::Json;
use serde::{Serialize};
use tracing::{error, info};
use crate::helpers::registry_from_string;
use crate::signer;

fn serialize_u256_as_string<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

sol! {
    #[derive(Serialize)]
    struct TLSNVerifierMessage {
        address registry;
        #[serde(rename = "credential_group_id")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 credentialGroupId;
        #[serde(rename = "id_hash")]
        bytes32 idHash;
        #[serde(rename = "semaphore_identity_commitment")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 semaphoreIdentityCommitment;
    }
}

#[derive(Serialize)]
pub struct VerifyResponse {
    verifier_message: TLSNVerifierMessage,
    verifier_hash: String,
    signature: String,
}

pub async fn verifier_response(
    registry_address: String,
    credential_group_id: String,
    semaphore_identity_commitment: U256,
    id_hash: B256
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    let registry = registry_from_string(registry_address)
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let verifier_message = TLSNVerifierMessage {
        registry,
        credentialGroupId: U256::from_str(credential_group_id.as_str()).map_err(|e| {(StatusCode::BAD_REQUEST, e.to_string())})?,
        semaphoreIdentityCommitment: semaphore_identity_commitment,
        idHash: id_hash,
    };
    let message = keccak256(verifier_message.abi_encode());
    let signature = signer::get().sign_message(message.as_slice())
        .await
        .map_err(|e| {
            error!("unexpected error during message signing: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    info!("Oauth verification completed");
    Ok(
        Json(VerifyResponse {
            verifier_message,
            verifier_hash: hex::encode_prefixed(message),
            signature: signature.to_string(),
        })
    )
}