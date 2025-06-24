use std::error::Error;
use tlsn_core::attestation::{Attestation};
use tlsn_core::presentation::{Presentation, PresentationOutput};
use serde::{ Serialize, Deserialize };
use tlsn_core::{CryptoProvider};
use crate::{config, signer};

#[derive(Deserialize)]
pub struct VerifyRequest {
    proof: Vec<u8>,
    attestation: Attestation,
}

#[derive(Serialize)]
pub struct VerificationResponse {
    hash: String,
    signature: String
}

pub fn verify_proof(
    presentation: Presentation,
) -> Result<VerificationResponse, Box<dyn Error>> {
    let PresentationOutput {
        attestation,
        server_name,
        transcript,
        ..
    } = presentation.verify(&CryptoProvider::default())?;

    if attestation.body.verifying_key() != &config::get().notary_key {
        return Err("Invalid Notary key".into());
    }
    let server_name = server_name.ok_or("Server name is not set")?;
    let transcript = transcript.ok_or("Transcript is not provided")?;

    transcript.received_authed()
        .iter_ranges()
        .for_each(|x| {
            println!("\nIdx:\t{:?}", x);
            let data = &transcript.received_unsafe()[x.clone()];
            println!("Data:\t{:?}", data);
            // std::str::from_utf8(data);
            if let Ok(text) = std::str::from_utf8(data) {
                println!("Text:\t\"{}\"", text);
            } else {
                println!("Text:\t<invalid>");
            }
        });

    Ok(VerificationResponse {
        hash: "1234567890".to_string(),
        signature: "1234567890".to_string(),
    })
}