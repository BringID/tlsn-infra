use std::str::FromStr;
use alloy::signers::Signer;
use alloy::{sol};
use alloy::primitives::{Address, aliases::U256, hex, keccak256};
use alloy::sol_types::{SolStruct, SolValue};
use axum::{Json, http::StatusCode};
use rand::RngCore;
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
        #[serde(rename = "verification_id")]
        #[serde(serialize_with = "serialize_u256_as_string")]
        uint256 verificationId;
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
    verification_id: String,
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
) -> Result<Json<VerifyResponse>, StatusCode> {

    // TODO Verify first to get id_hash
    let presentation = hex::decode(payload.tlsn_presentation.as_str())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let presentation = bincode::deserialize(&presentation)
        .map_err(|e| {
            println!("Error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let id_hash = tlsn::verify_proof(presentation, &payload.verification_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let verification_id = U256::from_str(payload.verification_id.as_str())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let registry = Address::from_str(payload.registry.as_str())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|_| StatusCode::BAD_REQUEST)?;

    let verifier_message = TLSNVerifierMessage {
        registry,
        verificationId: verification_id,
        semaphoreIdentityCommitment: semaphore_identity_commitment,
        idHash: id_hash,
    };

    let message = keccak256(verifier_message.abi_encode());

    let signature = signer::get().sign_message(message.as_slice())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(
        Json(VerifyResponse{
            verifier_message,
            verifier_hash: hex::encode_prefixed(message),
            signature: signature.to_string(),
        })
    )
}