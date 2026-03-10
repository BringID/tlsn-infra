use alloy::primitives::Address;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

static OAUTH_SIGNERS: Lazy<HashMap<String, Address>> = Lazy::new(|| {
    let data = std::fs::read_to_string("oauth_signers.json")
        .expect("failed to read oauth_signers.json");
    let raw: HashMap<String, String> =
        serde_json::from_str(&data).expect("failed to parse oauth_signers.json");
    let signers: HashMap<String, Address> = raw
        .into_iter()
        .map(|(k, v)| {
            let addr = Address::from_str(&v)
                .unwrap_or_else(|_| panic!("invalid address for credential_group_id {k}: {v}"));
            (k, addr)
        })
        .collect();
    info!("loaded {} OAuth signer(s) from oauth_signers.json", signers.len());
    signers
});

pub fn get_oauth_signer(credential_group_id: &str) -> Option<&Address> {
    OAUTH_SIGNERS.get(credential_group_id)
}
