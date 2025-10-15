use std::error::Error;
use alloy::primitives::{B256};
use serde::{Deserialize, Serialize};
use crate::core::{PresentationCheck};

#[derive(Debug, Serialize, Deserialize)]
struct RideData {
    uuid: String,
    description: String,
}

pub fn handler(_: &PresentationCheck, transcript: &String) -> Result<(bool, Option<B256>), Box<dyn Error>> {
    let (key, value) = transcript
        .split_once(":")
        .ok_or("Wrong transcript provided")?;

    if key.trim() != "\"activities\"" {
        return Err("Wrong transcript provided - \"devices\" key was not found".into());
    }
    let rides: Vec<RideData> = serde_json::from_str(value)?;

    let valid_rides_count = rides
        .iter()
        .filter(|ride| !ride.description.contains("Canceled"))
        .count();

    if valid_rides_count < 5 {
        return Ok((false, None))
    }

    Ok((true, None))
}