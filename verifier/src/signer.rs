use std::sync::OnceLock;
use crate::{config};
use alloy::signers::{local::PrivateKeySigner};

static SIGNER: OnceLock<PrivateKeySigner> = OnceLock::new();

pub fn get() -> &'static PrivateKeySigner {
    SIGNER.get_or_init(||
        PrivateKeySigner::from(config::get().private_key.clone())
    )
}