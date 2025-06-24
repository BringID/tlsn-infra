mod tlsn;
mod config;
mod signer;
mod server;

use bincode;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    config::init();
    server::worker::run().await
}