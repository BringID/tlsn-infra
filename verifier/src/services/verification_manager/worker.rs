use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, OnceLock, RwLock};
use crate::core::Verification;
use std::collections::HashMap;
use tracing::{debug, error, instrument};
use crate::services::handlers_manager::HandlersManager;

static VERIFICATIONS: OnceLock<RwLock<HashMap<String, Arc<Verification>>>> = OnceLock::new();

pub struct VerificationManager;

impl VerificationManager {
    pub fn get(id: &str) -> Option<Arc<Verification>> {
        VERIFICATIONS
            .get()
            .and_then(|map| map.read().ok()?.get(id).cloned())
    }

    #[instrument(
        name="verification_manager_loader",
        level="info",
        err
    )]
    pub async fn from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let raw_verifications: HashMap<String, Verification> = serde_json::from_reader(reader)?;
        let arc_verifications = raw_verifications
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect();

        VERIFICATIONS
            .set(RwLock::new(arc_verifications))
            .map_err(|_| {
                error!("already initialized");
                "Already initialized"
            })?;

        let handlers = HandlersManager::get_handlers().read().await;

        // Validating verifications
        for (_, verification) in VERIFICATIONS.get().unwrap().read().unwrap().iter() {
            if let Some(user_id_handler) = &verification.user_id.custom_handler {
                if handlers.get(user_id_handler).is_none() {
                    error!("handler {user_id_handler} is not found");
                    return Err(format!("handler {} is not found", user_id_handler).into());
                }
            }
            for check in verification.checks.iter() {
                if let Some(handler) = &check.custom_handler {
                    if handlers.get(handler).is_none() {
                        error!("handler {handler} is not found");
                        return Err(format!("handler {} is not found", handler).into());
                    }
                }
            }
            debug!("loaded {:?}", verification);
        }
        Ok(())
    }
}
