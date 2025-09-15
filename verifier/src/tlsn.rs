use std::error::Error;
use alloy::primitives::{keccak256, B256};
use tlsn_core::presentation::{Presentation, PresentationOutput};
use tlsn_core::{CryptoProvider};
use crate::{config};
use crate::services::VerificationManager;
use rand::{rng, RngCore};

// Returns user_id_hash
pub fn verify_proof(
    presentation: Presentation,
    credential_group_id: &String,
) -> Result<B256, Box<dyn Error>> {
    let PresentationOutput {
        attestation,
        server_name,
        transcript,
        ..
    } = presentation.verify(&CryptoProvider::default())?;
    
    let server_name = server_name.ok_or("Server name is not set")?;
    let transcript = transcript.ok_or("Transcript is not provided")?;

    let transcript_authed: Vec<String> = transcript.received_authed()
        .iter_ranges()
        .filter_map(|range| {
            let data = &transcript.received_unsafe()[range];
            std::str::from_utf8(data)
                .ok()
                .map(|s| s.to_string())
        })
        .collect();

    transcript.received_authed().iter_ranges()
        .for_each(|x| {
            println!("\nIdx:\t{:?}", x);
            let data = &transcript.received_unsafe()[x];
            if let Ok(text) = std::str::from_utf8(data) {
                println!("Text Len:\t{}", text.len());
                println!("Text:\t{}", text);
            } else {
                println!("Text:\t<invalid>");
            }
        });

    let verification = VerificationManager::get(&credential_group_id)
        .ok_or("Verification is not found")?
        .clone();

    let user_id_hash: B256;
    match std::env::var("ENV") {
        Ok(env) if env == "dev"  => {
            user_id_hash = {
                let mut random_bytes = [0u8; 32];
                rng().fill_bytes(&mut random_bytes);
                B256::from(random_bytes)
            };
        }
        _ => {
            if attestation.body.verifying_key() != &config::get().notary_key {
                return Err("Invalid Notary key".into());
            }
            user_id_hash = keccak256(
                transcript_authed.get(verification.user_id.window.id)
                    .ok_or("User ID is not found")?
                    .split(":")
                    .nth(1)
                    .ok_or("User ID is not found")?
                    .trim()
                    .trim_matches('"')
                    .as_bytes()
            );
        }
    }

    verification.check(
        server_name.to_string(),
        &transcript_authed
    )?;
    Ok(user_id_hash)
}