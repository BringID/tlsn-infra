use std::sync::OnceLock;
use crate::{config};
use alloy::signers::{
    Signature,
    local::PrivateKeySigner,
    Signer,
    Result as SignerResult,
};

static SIGNER: OnceLock<PrivateKeySigner> = OnceLock::new();

pub fn get() -> &'static PrivateKeySigner {
    SIGNER.get_or_init(||
        PrivateKeySigner::from(config::get().private_key.clone())
    )
}

pub async fn sign(chain_id: u32) -> SignerResult<Signature> {
    get().sign_message(b"hello").await
}