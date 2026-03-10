use alloy::primitives::Address;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

static OAUTH_SIGNERS: Lazy<HashMap<String, Address>> = Lazy::new(|| {
    let filename = if matches!(std::env::var("ENV"), Ok(ref v) if v == "dev") {
        "oauth_signers_staging.json"
    } else {
        "oauth_signers.json"
    };
    let data = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("failed to read {filename}"));
    let raw: HashMap<String, String> =
        serde_json::from_str(&data).unwrap_or_else(|_| panic!("failed to parse {filename}"));
    let signers: HashMap<String, Address> = raw
        .into_iter()
        .map(|(k, v)| {
            let addr = Address::from_str(&v)
                .unwrap_or_else(|_| panic!("invalid address for credential_group_id {k}: {v}"));
            (k, addr)
        })
        .collect();
    info!("loaded {} OAuth signer(s) from {filename}", signers.len());
    signers
});

pub fn get_oauth_signer(credential_group_id: &str) -> Option<&Address> {
    OAUTH_SIGNERS.get(credential_group_id)
}
