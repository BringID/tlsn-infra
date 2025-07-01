mod check;
mod window;
mod presentation_check;

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
    pub fn check(&self, server_name: String, transcript: &Vec<String>) -> bool {
        if server_name != self.host {
            return false;
        }

        let Some(data) = transcript.get(self.user_id.window.id) else { return false };
        if !self.user_id.check(data) {
            return false;
        }

        for check in &self.checks {
            let Some(data) = transcript.get(check.window.id) else { continue };
            if !check.check(data) {
                return false;
            }
        }

        true
    }
}