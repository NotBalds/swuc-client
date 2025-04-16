use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    pub url: String,
    pub uuid: String,
    pub secret: String,
    pub public: String,
}

fn get_config_dir() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("com", "swuc", "SWUC").context("Failed to get project directories")?;

    let config_dir = proj_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
    }

    Ok(config_dir.to_path_buf())
}

fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("user_config.json"))
}

pub fn load_user_config() -> Result<UserConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        anyhow::bail!("User config not found at {:?}", config_path);
    }

    let data = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config from {:?}", config_path))?;

    serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse config from {:?}", config_path))
}
