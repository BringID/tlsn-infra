use std::error::Error;
use alloy::primitives::{keccak256, B256};
use rand::{rng, RngCore};
use crate::core::PresentationCheck;
use crate::services::HandlersManager;

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
                    transcript_authed.get(check.window.id).ok_or("User ID window is not found")?
                ).await?;
                if !success {
                    return Err("User ID check was unsuccessful".into());
                }
                Ok(user_id_hash.ok_or("This is a bug. User ID hash was not computed")?)
            } else {
                Ok(keccak256(
                    transcript_authed.get(check.window.id)
                        .ok_or("User ID is not found")?
                        .split(":")
                        .nth(1)
                        .ok_or("User ID is not found")?
                        .trim()
                        .trim_matches('"')
                        .as_bytes()
                ))
            }
        }
    }

}