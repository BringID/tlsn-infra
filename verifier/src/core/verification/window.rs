use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct Window {
    pub(crate) id: usize,
    pub(crate) key: String,
}