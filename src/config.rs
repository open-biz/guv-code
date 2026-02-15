use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub keys: Keys,
    pub budget: Budget,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Keys {
    pub gemini: Option<String>,
    pub anthropic: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Budget {
    pub limit: f64,
    pub consumed: f64,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            limit: 10.0,
            consumed: 0.0,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config file")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(path, content).context("Failed to write config file")
    }

    fn config_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "guv", "guv-code") {
             // We can also use a simple home-dir based path if preferred, 
             // but ProjectDirs is more idiomatic.
             // For "guv", let's use ~/.guv.toml as requested in spec.
             let home = directories::UserDirs::new()
                .context("Could not find home directory")?
                .home_dir()
                .to_path_buf();
             Ok(home.join(".guv.toml"))
        } else {
            anyhow::bail!("Could not determine configuration directory")
        }
    }
}
