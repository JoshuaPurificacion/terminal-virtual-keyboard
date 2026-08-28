use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shell: Option<String>,
    pub gamepad_device: Option<String>,
    pub keyboard_height_ratio: f32,
    pub min_keyboard_height: u16,
    pub max_keyboard_height: u16,
    pub repeat_delay_ms: u64,
    pub repeat_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: None,
            gamepad_device: None,
            keyboard_height_ratio: 0.45,
            min_keyboard_height: 9,
            max_keyboard_height: 14,
            repeat_delay_ms: 250,
            repeat_rate_ms: 50,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let paths = ["config.toml", "/etc/cyberdeck-kb/config.toml"];
        for path_str in &paths {
            let path = Path::new(path_str);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(cfg) = toml::from_str::<Config>(&content) {
                        return cfg;
                    }
                }
            }
        }
        Config::default()
    }
}
