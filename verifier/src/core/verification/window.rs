use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct Window {
    pub(crate) id: usize,
    pub(crate) key: String,
}

impl Window {
    pub fn new(id: usize, key: String) -> Self {
        Self { id, key }
    }
}