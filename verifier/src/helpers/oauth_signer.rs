use alloy::primitives::Address;
use std::str::FromStr;
use once_cell::sync::Lazy;

pub static OAUTH_SIGNER: Lazy<Address> = Lazy::new(|| {
    Address::from_str(
        &std::env::var("OAUTH_SIGNER_ADDRESS")
            .expect("OAUTH_SIGNER_ADDRESS not set")
    ).expect("invalid address")
});