use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AppConfig {
    pub directory: String,
    pub file_pattern: String,
}
