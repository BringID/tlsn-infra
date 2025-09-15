mod tlsn;
mod config;
mod signer;
mod core;
mod services;
mod helpers;
mod custom_handlers;

use bincode;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    config::init();
    services::HandlersManager::register(
        "apple_devices_user_id".to_string(),
        custom_handlers::handler
    ).await?;

    services::VerificationManager::from_file("verifications.json").await?;
    services::Server::run().await
}