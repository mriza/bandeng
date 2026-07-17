use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use base64::Engine;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub ip: Option<String>,
    pub user: Option<String>,
    pub pass_b64: Option<String>,
    pub secure: Option<bool>,
    pub save_credentials: Option<bool>,
}

impl AppConfig {
    pub fn get_password(&self) -> Option<String> {
        if let Some(ref b64) = self.pass_b64 {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                if let Ok(s) = String::from_utf8(bytes) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn set_password(&mut self, pass: &str) {
        self.pass_b64 = Some(base64::engine::general_purpose::STANDARD.encode(pass));
    }
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".bandeng");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("config.json");
    path
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
            return config;
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, data);
    }
}
