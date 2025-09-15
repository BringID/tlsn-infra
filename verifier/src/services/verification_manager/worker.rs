use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, OnceLock, RwLock};
use crate::core::Verification;
use std::collections::HashMap;
use crate::services::handlers_manager::HandlersManager;

static VERIFICATIONS: OnceLock<RwLock<HashMap<String, Arc<Verification>>>> = OnceLock::new();

pub struct VerificationManager;

impl VerificationManager {
    // TODO implement HTTP autoupdate
    pub fn autoupdate() -> Result<(), &'static str> {
        Err("Not implemented")
    }

    pub fn get(id: &str) -> Option<Arc<Verification>> {
        VERIFICATIONS
            .get()
            .and_then(|map| map.read().ok()?.get(id).cloned())
    }

    pub async fn from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Читаем верификации из файла
        let raw_verifications: HashMap<String, Verification> = serde_json::from_reader(reader)?;

        // Преобразуем в HashMap с Arc<Verification>
        let arc_verifications = raw_verifications
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect();

        VERIFICATIONS
            .set(RwLock::new(arc_verifications))
            .map_err(|_| "Already initialized")?;

        let handlers = HandlersManager::get_handlers().read().await;

        // Validating verifications
        for (_, verification) in VERIFICATIONS.get().unwrap().read().unwrap().iter() {
            if let Some(user_id_handler) = &verification.user_id.custom_handler {
                if handlers.get(user_id_handler).is_none() {
                    return Err(format!("Handler {} is not found", user_id_handler).into());
                }
            }
            for check in verification.checks.iter() {
                if let Some(handler) = &check.custom_handler {
                    println!("Handler: {}", handler);
                    if handlers.get(handler).is_none() {
                        return Err(format!("Handler {} is not found", handler).into());
                    }
                }
            }
            dbg!(verification);
        }
        Ok(())
    }

    pub fn add(verification: Verification) -> Result<(), &'static str> {
        let id = verification.id.clone();
        let map = VERIFICATIONS.get().ok_or("Not initialized")?;

        match map.write() {
            Ok(mut write_guard) => {
                write_guard.insert(id, Arc::new(verification));
                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock"),
        }
    }
}
