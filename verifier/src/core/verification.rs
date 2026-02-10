mod check;
mod window;
mod presentation_check;

use std::error::Error;
use std::fmt::Debug;
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use tracing::{instrument, error, warn};
pub use presentation_check::PresentationCheck;
use crate::services::HandlersManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub(crate) id: String,
    pub(crate) host: String,
    pub(crate) user_id: PresentationCheck,
    pub(crate) checks: Vec<PresentationCheck>
}

impl Verification {
    #[instrument(
        level="info",
        name="verification_check",
        skip(self, server_name, transcript),
    )]
    pub async fn check(
        &self,
        server_name: String,
        transcript: &Vec<String>,
        app_id: &U256,
    ) -> Result<(), Box<dyn Error>> {
        if server_name != self.host {
            error!("wrong server name");
            return Err("wrong server name".into());
        }

        let Some(user_id_data) = transcript.get(self.user_id.window.id) else {
            error!("missing user_id");
            return Err("missing user_id".into());
        };
        if !self.user_id.check(user_id_data) {
            error!("wrong user_id");
            return Err("wrong user_id".into());
        }

        for check in &self.checks {
            let Some(data) = transcript.get(check.window.id) else {
                error!("missing check window data");
                return Err("missing check window data".into());
            };
            if check.custom_handler.is_some() {
                let (success, _) = HandlersManager::execute(check, data, app_id)
                    .await
                    .inspect_err(
                        |err| warn!("custom handler error: {err}")
                    )?;
                if !success {
                    warn!("custom handler failed check");
                    return Err("check failed".into());
                }
            } else if !check.check(data) {
                return Err("check failed".into());
            }
        }

        Ok(())
    }
}