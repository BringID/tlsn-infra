use std::error::Error;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use tracing::{instrument, error, warn};
use crate::services::HandlersManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthVerification {
    pub(crate) id: String,
    pub(crate) domain: String,
    pub(crate) score: i32,
}

impl OAuthVerification {
    #[instrument(
        level="info",
        name="verification_check",
    )]
    pub async fn check(
        &self,
        domain: String,
        score: i32
    ) -> Result<(), Box<dyn Error>> {
        if score < self.score {
            error!("not enough score");
            return Err("low score".into());
        }

        if domain != self.domain {
            error!("wrong domain");
            return Err("wrong domain".into());
        }

        Ok(())
    }
}