use std::str::FromStr;
use alloy::signers::Signer;
use alloy::{sol};
use alloy::primitives::{Address, aliases::U256, hex, keccak256};
use alloy::sol_types::{SolStruct, SolValue};
use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use crate::signer;
use crate::tlsn;

fn serialize_u256_as_string<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}


// TLSN Verifier Message
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

#[derive(Deserialize)]
pub struct VerifyRequest {
    tlsn_presentation: String,
    registry: String,
    credential_group_id: String,
    semaphore_identity_commitment: String
}

#[derive(Serialize)]
pub struct VerifyResponse {
    verifier_message: TLSNVerifierMessage,
    verifier_hash: String,
    signature: String,
}

pub async fn handle(
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {

    // TODO Verify first to get id_hash
    let presentation = hex::decode(payload.tlsn_presentation.as_str())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let presentation = bincode::deserialize(&presentation)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let id_hash = tlsn::verify_proof(presentation, &payload.credential_group_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let credential_group_id = U256::from_str(payload.credential_group_id.as_str())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let registry = Address::from_str(payload.registry.as_str())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let verifier_message = TLSNVerifierMessage {
        registry,
        credentialGroupId: credential_group_id,
        semaphoreIdentityCommitment: semaphore_identity_commitment,
        idHash: id_hash,
    };

    let message = keccak256(verifier_message.abi_encode());

    let signature = signer::get().sign_message(message.as_slice())
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(
        Json(VerifyResponse{
            verifier_message,
            verifier_hash: hex::encode_prefixed(message),
            signature: signature.to_string(),
        })
    )
}