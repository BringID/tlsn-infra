use tlsn_core::signing::{KeyAlgId, VerifyingKey};
use std::sync::OnceLock;
use std::env;
use alloy::hex;
use alloy::signers::k256::SecretKey;
use dotenv::from_filename;

#[derive(Clone)]
pub struct Config {
    pub notary_key: VerifyingKey,
    pub private_key: SecretKey,
    pub port: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init() {
    get();
}

pub fn get() -> &'static Config {
    from_filename(".env").ok();
    from_filename(".env.dev").ok();
    
    let alg: KeyAlgId =
        match env::var("NOTARY_KEY_ALG")
            .expect("NOTARY_KEY_ALG is not set")
            .as_str()
        {
            "K256" => KeyAlgId::K256,
            "P256" => KeyAlgId::P256,
            _ => panic!("Unsupported NOTARY_KEY_ALG"),
        };

    let data: Vec<u8> = hex::decode(
        &env::var("NOTARY_KEY_HEX").expect("NOTARY_KEY_HEX is not set")
    ).expect("Invalid hex format in NOTARY_KEY_HEX");
    
    let private_key = env::var("PRIVATE_KEY_HEX").expect("PRIVATE_KEY_HEX is not set");
    let private_key = SecretKey::from_slice(
        hex::decode(private_key)
            .expect("Invalid hex format in PRIVATE_KEY_HEX")
            .as_slice()
    ).expect("Invalid private key");
    
    let port = env::var("PORT").expect("PORT is not set");
    
    CONFIG.get_or_init(|| Config {
        port,
        notary_key: VerifyingKey {
            alg,
            data,
        },
        private_key,
    })
}
