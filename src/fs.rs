use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    api_key: String,
}

fn resolve_config_path() -> Result<PathBuf, String> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(format!(".config/{}/config.json", crate_name!())))
    } else {
        Err("failed to resolve config path".to_string())
    }
}

fn load_config() -> Result<Config, String> {
    let path = resolve_config_path()?;
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("failed to read config: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to deserialize config: {}", e))
}

pub fn load_api_key() -> Result<Option<String>, String> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    load_config().map(|c| Some(c.api_key.clone()))
}

fn save_config(c: &Config) -> Result<(), String> {
    let text = serde_json::to_string_pretty(c)
        .map_err(|e| format!("failed to serialize config: {}", e))?;

    let path_file = resolve_config_path()?;
    let path_dir = if let Some(path) = path_file.parent() {
        path
    } else {
        return Err("failed to resolve config directory".to_string());
    };
    if !path_dir.exists() {
        std::fs::create_dir_all(path_dir)
            .map_err(|e| format!("failed to create config directory: {}", e))?;
    }
    std::fs::write(path_file, text).map_err(|e| format!("failed to write config: {}", e))
}

pub fn save_api_key(api_key: &String) -> Result<(), String> {
    let c = Config {
        api_key: api_key.clone(),
    };
    save_config(&c)
}
