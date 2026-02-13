use std::collections::HashMap;
use std::error::Error;
use std::sync::OnceLock;
use alloy::primitives::{B256, U256};
use tokio::sync::RwLock;
use crate::core::{PresentationCheck};

type HandlerFn = fn(&PresentationCheck, &str, &U256) -> Result<(bool, Option<B256>), Box<dyn Error>>;

static HANDLERS: OnceLock<RwLock<HashMap<String, HandlerFn>>> = OnceLock::new();

pub struct HandlersManager;

impl HandlersManager {
    pub fn get_handlers() -> &'static RwLock<HashMap<String, HandlerFn>> {
        HANDLERS.get_or_init(|| RwLock::new(HashMap::new()))
    }

    pub async fn get_handler(key: &str) -> Option<HandlerFn> {
        let handlers = Self::get_handlers();
        let read_lock = handlers.read().await;
        read_lock.get(key).copied()
    }

    pub async fn execute(
        check: &PresentationCheck,
        transcript: &str,
        app_id: &U256,
    ) -> Result<(bool, Option<B256>), Box<dyn Error>> {
        if let Some(key) = &check.custom_handler {
            if let Some(handler) = Self::get_handler(key).await {
                handler(check, transcript, app_id)
            } else {
                Err("Handler is not set for the Check".into())
            }
        } else {
            Err("This is a bug. Custom check was initiated by mistake".into())
        }
    }

    pub async fn register(name: String, handler: HandlerFn) -> Result<(), &'static str> {
        let handlers = Self::get_handlers();
        let mut write_lock = handlers.write().await;
        write_lock.insert(name, handler);
        Ok(())
    }
}
