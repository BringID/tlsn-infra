use std::str::FromStr;
use alloy::hex::FromHexError;
use alloy::primitives::Address;
use once_cell::sync::Lazy;

pub static REGISTRY_WHITELIST: Lazy<Vec<Address>> = Lazy::new(|| {
    std::env::var("REGISTRY_WHITELIST")
        .expect("REGISTRY_WHITELIST not set")
        .split(',')
        .map(|s| Address::from_str(s.trim()).expect("invalid address in REGISTRY_WHITELIST"))
        .collect()
});

pub fn registry_from_string(
    registry_address: String
) -> Result<Address, FromHexError> {
    Address::from_str(registry_address.as_str())
}

pub fn is_registry_whitelisted(address: &Address) -> bool {
    REGISTRY_WHITELIST.contains(address)
}