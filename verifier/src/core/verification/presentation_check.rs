use std::fmt::format;
use serde::{Deserialize, Serialize};
use super::check::{Check, CheckableValue};
use super::window::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationCheck {
    pub window: Window,
    #[serde(flatten)]
    pub check: Check,
}

impl PresentationCheck {
    pub fn new(window: Window, check: Check) -> Self {
        Self { window, check }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn check(&self, transcript: &str) -> bool {
        if !transcript.starts_with(&format!("\"{}\"", self.window.key)) {
            return false;
        }
        let Some(value) = transcript
            .split(':')
            .nth(1)
            .map(|s| s.trim_matches('"'))
        else {
            return false;
        };

        match value.parse::<i64>() {
            Ok(num) => self.check_value(num),
            Err(_) => self.check_value(value),
        }
    }

    pub fn check_value<T: CheckableValue>(&self, value: T) -> bool {
        value.check_against(&self.check)
    }
}