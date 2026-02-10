use std::error::Error;
use alloy::primitives::{keccak256, B256, U256};
use rand::{rng, RngCore};
use tracing::{error, instrument, warn};
use crate::config;
use crate::core::PresentationCheck;
use crate::services::HandlersManager;

pub fn random_credential_id() -> B256 {
    let mut random_bytes = [0u8; 32];
    rng().fill_bytes(&mut random_bytes);
    B256::from(random_bytes)
}

pub fn credential_id_from_bytes(
    user_id_bytes: &[u8],
    app_id: &U256,
) -> Result<B256, Box<dyn Error>> {
    let private_key_bytes = config::get().private_key.to_bytes();
    let mut buf = Vec::new();
    buf.extend_from_slice(user_id_bytes);
    buf.extend_from_slice(&app_id.to_be_bytes::<32>());
    buf.extend_from_slice(&private_key_bytes);
    Ok(keccak256(&buf))
}

#[instrument(
    name="credential_id",
    level="info",
    skip(transcript_authed, check),
    err
)]
pub async fn credential_id(
    transcript_authed: &[String],
    check: &PresentationCheck,
    app_id: &U256,
) -> Result<B256, Box<dyn Error>> {
    match std::env::var("ENV") {
        Ok(env) if env == "dev"  => {
            Ok(random_credential_id())
        }
        _ => {
            if check.custom_handler.is_some() {
                let (success, credential_id) = HandlersManager::execute(
                    check,
                    transcript_authed
                        .get(check.window.id)
                        .ok_or_else(|| {
                            warn!("user ID window is not found");
                            "user ID window is not found"
                        })?,
                    app_id
                ).await.inspect_err(|_| warn!("verification handler execution failed"))?;
                if !success {
                    warn!("user ID check was unsuccessful");
                    return Err("User ID check was unsuccessful".into());
                }
                Ok(credential_id.ok_or_else(|| {
                    error!("credential ID was not computed");
                    "credential ID was not computed"
                })?)
            } else {
                let id_bytes = transcript_authed.get(check.window.id)
                    .ok_or_else(|| {
                        warn!("user ID window is not found");
                        "user ID window is not found"
                    })?
                    .split(":")
                    .nth(1)
                    .ok_or_else(|| {
                        warn!("user ID is not presented");
                        "user ID is not presented"
                    })?
                    .trim()
                    .trim_matches('"')
                    .as_bytes();
                credential_id_from_bytes(id_bytes, app_id)
            }
        }
    }
}
