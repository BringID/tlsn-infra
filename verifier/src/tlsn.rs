use std::error::Error;
use std::str::FromStr;
use alloy::hex::ToHexExt;
use alloy::primitives::{B256, U256};
use tlsn_core::presentation::{Presentation, PresentationOutput};
use tlsn_core::{CryptoProvider};
use tracing::{debug, trace, instrument, error};
use crate::{config};
use crate::services::VerificationManager;
use crate::helpers::credential_id;

#[instrument(
    name="proof_verifier",
    level="info",
    skip(presentation, credential_group_id),
)]
pub async fn verify_proof(
    presentation: Presentation,
    credential_group_id: &String,
    app_id: &str,
) -> Result<B256, Box<dyn Error>> {
    debug!("verification started");
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
            trace!("\nrange\n{:?}", x);
            let data = &transcript.received_unsafe()[x];
            if let Ok(text) = std::str::from_utf8(data) {
                trace!("\ntext Len\n{}", text.len());
                trace!("\ntext\n{}", text);
            } else {
                trace!("\ntext\n<invalid>");
            }
        });

    let verification = VerificationManager::get(&credential_group_id)
        .ok_or_else(|| {
            error!("verification is not found");
            "verification is not found"
        })?
        .clone();

    let app_id_u256 = U256::from_str(app_id)
        .map_err(|e| -> Box<dyn Error> { format!("invalid app_id: {e}").into() })?;

    let cid = credential_id(&transcript_authed, &verification.user_id, &app_id_u256).await?;

    if attestation.body.verifying_key() != &config::get().notary_key {
        error!("invalid notary key");
        return Err(
            format!("invalid notary key.\nexpected: {}\ngot: {}",
                    &config::get().notary_key.data.encode_hex_with_prefix(),
                    attestation.body.verifying_key().data.encode_hex_with_prefix()
            ).into()
        );
    }

    verification.check(
        server_name.to_string(),
        &transcript_authed,
        &app_id_u256,
    ).await?;

    debug!("proof verified");
    Ok(cid)
}
