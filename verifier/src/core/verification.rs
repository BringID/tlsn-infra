mod check;
mod window;
mod presentation_check;

use std::error::Error;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
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
    pub async fn check(
        &self,
        server_name: String,
        transcript: &Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        if server_name != self.host {
            return Err("Wrong server name".into());
        }

        let Some(user_id_data) = transcript.get(self.user_id.window.id) else {
            return Err("Missing user_id".into());
        };
        if !self.user_id.check(user_id_data) {
            return Err("Wrong user_id".into());
        }

        for check in &self.checks {
            let Some(data) = transcript.get(check.window.id) else {
                return Err("Missing data".into());
            };
            if check.custom_handler.is_some() {
                let (success, _) = HandlersManager::execute(check, data).await?;
                if !success {
                    return Err("Check failed".into());
                }
            } else if !check.check(data) {
                return Err("Check failed".into());
            }
        }

        Ok(())
    }
}