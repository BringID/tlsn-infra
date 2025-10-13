use serde::{Deserialize, Serialize};
use serde_json::{Value};
use super::check::{Check, CheckableValue};
use super::window::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationCheck {
    pub window: Window,
    #[serde(flatten)]
    pub check: Check,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_handler: Option<String>,

}

impl PresentationCheck {
    pub fn check(&self, transcript: &str) -> bool {
        if self.window.key != "-" {
            if !transcript.trim_matches(' ').starts_with(&format!("\"{}\"", self.window.key)) {
                return false;
            }
            let Some(value) = transcript
                .split_once(':')
                .map(|(_, val)| val.trim_matches('"'))
            else {
                return false;
            };

            // Trying to parse JSON array
            if let Ok(json_value) = serde_json::from_str::<Value>(value) {
                if let Some(array) = json_value.as_array() {
                    return self.check_value(array);
                }
            }

            // Trying to parse i64, if error - process &str
            match value.parse::<i64>() {
                Ok(num) => self.check_value(num),
                Err(_) => self.check_value(value),
            }
        } else {
            self.check_value(transcript)
        }
    }

    pub fn check_value<T: CheckableValue>(&self, value: T) -> bool {
        value.check_against(&self.check)
    }
}