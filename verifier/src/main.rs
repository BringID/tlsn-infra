mod tlsn;
mod config;
mod signer;
mod core;
mod services;

use bincode;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    config::init();
    services::VerificationManager::from_file("verifications.json")?;
    services::Server::run().await
}