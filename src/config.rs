use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model_size: String,
    pub hotkey: String,
    pub foreground: bool,
}
