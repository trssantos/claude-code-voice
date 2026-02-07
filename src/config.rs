use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model_size: String,
    pub hotkey: String,
    pub foreground: bool,
}

impl Config {
    pub fn new(model_size: String, hotkey: String, foreground: bool) -> Self {
        Self {
            model_size,
            hotkey,
            foreground,
        }
    }
}
