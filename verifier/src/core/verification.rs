mod check;
mod window;
mod presentation_check;

use std::fmt::Debug;
use serde::{Deserialize, Serialize};
pub use presentation_check::PresentationCheck;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub(crate) id: String,
    pub(crate) host: String,
    pub(crate) user_id: PresentationCheck,
    pub(crate) checks: Vec<PresentationCheck>
}

impl Verification {
    pub fn check(
        &self,
        server_name: String,
        transcript: &Vec<String>
    ) -> Result<(), &'static str> {
        if server_name != self.host {
            return Err("Wrong server name");
        }

        let Some(user_id_data) = transcript.get(self.user_id.window.id) else {
            return Err("Missing user_id");
        };
        if !self.user_id.check(user_id_data) {
            return Err("Wrong user_id");
        }

        for check in &self.checks {
            let Some(data) = transcript.get(check.window.id) else {
                return Err("Missing data");
            };
            if !check.check(data) {
                return Err("Check failed");
            }
        }

        Ok(())
    }
}