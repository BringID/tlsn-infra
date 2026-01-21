use std::str::FromStr;
use alloy::signers::Signer;
use alloy::{sol};
use alloy::primitives::{Address, aliases::U256, hex, keccak256};
use alloy::sol_types::{SolValue};
use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};
use crate::signer;
use crate::tlsn;
use tracing::{info, error, instrument, warn, trace};

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

#[derive(Deserialize, Debug)]
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

    let id_hash = tlsn::verify_proof(presentation, &payload.credential_group_id).await
        .map_err(|e| {
            warn!("verification failed");
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let credential_group_id = U256::from_str(payload.credential_group_id.as_str())
        .map_err(|e| {
            error!("wrong Credential Group provided {}: {e}", payload.credential_group_id);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let registry = Address::from_str(payload.registry.as_str())
        .map_err(|e| {
            error!("invalid Registry address {}: {e}", payload.registry);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;
    
    let semaphore_identity_commitment = U256::from_str(
        payload.semaphore_identity_commitment.as_str()
    ).map_err(|e| {
        error!("invalid Semaphore Identity commitment: {e}");
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;

    let verifier_message = TLSNVerifierMessage {
        registry,
        credentialGroupId: credential_group_id,
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

    info!("verification completed");
    Ok(
        Json(VerifyResponse{
            verifier_message,
            verifier_hash: hex::encode_prefixed(message),
            signature: signature.to_string(),
        })
    )
}