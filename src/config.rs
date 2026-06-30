use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user: String,
    pub password: String,
    pub server_pwd: String,
    pub server_user: String,
    pub server_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user: String::new(),
            password: String::new(),
            server_pwd: String::new(),
            server_user: String::new(),
            server_ip: "10.237.35.254".into(),
        }
    }
}

fn config_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    let d = PathBuf::from(base).join("bastion_rdp");
    fs::create_dir_all(&d).ok();
    d
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load() -> Config {
    let p = config_path();
    if p.exists() {
        if let Ok(data) = fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }
    Config::default()
}

pub fn save(cfg: &Config) {
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        fs::write(config_path(), data).ok();
    }
}
