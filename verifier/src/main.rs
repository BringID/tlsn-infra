mod tlsn;
mod config;
mod signer;
mod core;
mod services;
mod helpers;
mod custom_handlers;
mod telemetry;
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    config::init();
    telemetry::init_logging();
    services::HandlersManager::register(
        "apple_devices_user_id".to_string(),
        custom_handlers::apple_devices_user_id
    ).await?;
    services::HandlersManager::register(
        "uber_rides_amount".to_string(),
        custom_handlers::uber_rides_amount
    ).await?;

    services::VerificationManager::from_file("verifications.json").await?;

    info!("service is running");
    services::Server::run().await
}