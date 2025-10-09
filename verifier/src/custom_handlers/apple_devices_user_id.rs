use std::error::Error;
use alloy::hex;
use alloy::primitives::{keccak256, B256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::{PresentationCheck};

#[derive(Debug, Serialize, Deserialize)]
struct DeviceData {
    id: String,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, Value>,
}

pub fn handler(_: &PresentationCheck, transcript: &String) -> Result<(bool, Option<B256>), Box<dyn Error>> {
    let (key, value) = transcript
        .split_once(":")
        .ok_or("Wrong transcript provided")?;

    if key.trim() != "\"devices\"" {
        return Err("Wrong transcript provided - \"devices\" key was not found".into());
    }
    let items: Vec<DeviceData> = serde_json::from_str(value)?;

    let user_id = items.get(0)
        .ok_or("Data array is empty")?
        .id
        .as_bytes();
    let salt_vec = hex::decode(std::env::var("SALT_HEX")?)?;
    let mut buf = Vec::with_capacity(user_id.len() + salt_vec.len());
    buf.extend_from_slice(user_id);
    buf.extend_from_slice(salt_vec.as_slice());

    let user_id_hash = keccak256(&buf);

    Ok((true, Some(user_id_hash)))
}