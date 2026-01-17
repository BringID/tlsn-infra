use std::error::Error;
use alloy::hex;
use alloy::primitives::{keccak256, B256};
use rand::{rng, RngCore};
use tracing::{error, instrument, warn};
use crate::core::PresentationCheck;
use crate::services::HandlersManager;

pub fn user_id_hash_from_bytes(
    user_id_bytes: &[u8]
) -> Result<B256, Box<dyn Error>> {
    let salt_vec = hex::decode(std::env::var("SALT_HEX")?)?;
    let mut buf = Vec::with_capacity(user_id_bytes.len() + salt_vec.len());
    buf.extend_from_slice(user_id_bytes);
    buf.extend_from_slice(salt_vec.as_slice());
    Ok(keccak256(&buf))
}

#[instrument(
    name="user_id_hash",
    level="info",
    skip(transcript_authed, check),
    err
)]
pub async fn user_id_hash(
    transcript_authed: &Vec<String>,
    check: &PresentationCheck,
) -> Result<B256, Box<dyn Error>> {
    match std::env::var("ENV") {
        Ok(env) if env == "dev"  => {
            let mut random_bytes = [0u8; 32];
            rng().fill_bytes(&mut random_bytes);
            Ok(B256::from(random_bytes))
        }
        _ => {
            if check.custom_handler.is_some() {
                let (success, user_id_hash) = HandlersManager::execute(
                    check,
                    transcript_authed
                        .get(check.window.id)
                        .ok_or_else(|| {
                            warn!("user ID window is not found");
                            "user ID window is not found"
                        })?
                ).await.inspect_err(|_| warn!("verification handler execution failed"))?;
                if !success {
                    warn!("user ID check was unsuccessful");
                    return Err("User ID check was unsuccessful".into());
                }
                Ok(user_id_hash.ok_or_else(|| {
                    error!("user ID hash was not computed");
                    "user ID hash was not computed"
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
                user_id_hash_from_bytes(id_bytes)
            }
        }
    }

}