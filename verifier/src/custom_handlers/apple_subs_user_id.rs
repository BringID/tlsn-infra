use std::error::Error;
use alloy::primitives::{keccak256, B256, U256};
use tracing::debug;
use crate::config;
use crate::core::PresentationCheck;

pub fn handler(_: &PresentationCheck, transcript: &str, app_id: &U256) -> Result<(bool, Option<B256>), Box<dyn Error>> {
    let (key, value) = transcript
        .split_once(":")
        .ok_or("Wrong transcript provided")?;

    if key.trim() != "\"subscriptionId\"" {
        return Err("Wrong transcript provided - \"subscriptionId\" key was not found".into());
    }

    let subscription_id = value.trim().trim_matches('"');
    debug!("Apple Subscription ID: {}", subscription_id);
    let subscription_id = subscription_id.as_bytes();

    let private_key_bytes = config::get().private_key.to_bytes();
    let mut buf = Vec::new();
    buf.extend_from_slice(subscription_id);
    buf.extend_from_slice(&app_id.to_be_bytes::<32>());
    buf.extend_from_slice(&private_key_bytes);

    let credential_id = keccak256(&buf);

    Ok((true, Some(credential_id)))
}
