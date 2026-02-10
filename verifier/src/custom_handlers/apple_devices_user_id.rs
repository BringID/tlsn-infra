use std::error::Error;
use alloy::primitives::{keccak256, B256, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;
use crate::config;
use crate::core::{PresentationCheck};

#[derive(Debug, Serialize, Deserialize)]
struct DeviceData {
    id: String,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, Value>,
}

pub fn handler(_: &PresentationCheck, transcript: &str, app_id: &U256) -> Result<(bool, Option<B256>), Box<dyn Error>> {
    let (key, value) = transcript
        .split_once(":")
        .ok_or("Wrong transcript provided")?;

    if key.trim() != "\"devices\"" {
        return Err("Wrong transcript provided - \"devices\" key was not found".into());
    }
    let items: Vec<DeviceData> = serde_json::from_str(value)?;

    let user_id = items.first()
        .ok_or("Data array is empty")?
        .id
        .clone();
    debug!("Apple UserID: {}", user_id);
    let user_id = user_id.as_bytes();

    let private_key_bytes = config::get().private_key.to_bytes();
    let mut buf = Vec::new();
    buf.extend_from_slice(user_id);
    buf.extend_from_slice(&app_id.to_be_bytes::<32>());
    buf.extend_from_slice(&private_key_bytes);

    let credential_id = keccak256(&buf);

    Ok((true, Some(credential_id)))
}
