use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, OnceLock, RwLock};
use crate::core::OAuthVerification;
use std::collections::HashMap;
use tracing::{debug, error, instrument};
use crate::services::handlers_manager::HandlersManager;

static OAUTH_VERIFICATIONS: OnceLock<RwLock<HashMap<String, Arc<OAuthVerification>>>> = OnceLock::new();

pub struct OAuthVerificationManager;

impl OAuthVerificationManager {
    pub fn get(id: &str) -> Option<Arc<OAuthVerification>> {
        OAUTH_VERIFICATIONS
            .get()
            .and_then(|map| map.read().ok()?.get(id).cloned())
    }

    #[instrument(
        name="oauth_verification_manager_loader",
        level="info",
        err
    )]
    pub async fn from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let raw_verifications: HashMap<String, OAuthVerification> = serde_json::from_reader(reader)?;
        let arc_verifications = raw_verifications
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect();

        OAUTH_VERIFICATIONS
            .set(RwLock::new(arc_verifications))
            .map_err(|_| {
                error!("already initialized");
                "Already initialized"
            })?;

        Ok(())
    }
}
