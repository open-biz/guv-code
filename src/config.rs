use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub keys: Keys,
    pub budget: Budget,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Keys {
    pub gemini: Option<String>,
    pub anthropic: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        let mut config = Config::default();

        // 1. Load from Global Config
        if let Ok(global_path) = Self::global_config_path() {
            if global_path.exists() {
                if let Ok(content) = fs::read_to_string(global_path) {
                    if let Ok(global_config) = toml::from_str::<Config>(&content) {
                        config = global_config;
                    }
                }
            }
        }

        // 2. Load from Local .guvcode (Overwrites global)
        let local_path = Path::new(".guvcode");
        if local_path.exists() {
            if let Ok(content) = fs::read_to_string(local_path) {
                if let Ok(local_config) = toml::from_str::<Config>(&content) {
                    // Merge logic: only overwrite if local has values
                    if local_config.keys.gemini.is_some() {
                        config.keys.gemini = local_config.keys.gemini;
                    }
                    if local_config.keys.anthropic.is_some() {
                        config.keys.anthropic = local_config.keys.anthropic;
                    }
                    // Overwrite budget if set
                    if local_config.budget.limit != 10.0 {
                        config.budget.limit = local_config.budget.limit;
                    }
                }
            }
        }

        // 3. Load from Env Vars (Highest priority)
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            config.keys.gemini = Some(key);
        }
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            config.keys.anthropic = Some(key);
        }

        Ok(config)
    }

    pub fn save_global(&self) -> Result<()> {
        let path = Self::global_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(path, content).context("Failed to write config file")
    }

    pub fn save_local(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize local config")?;
        fs::write(".guvcode", content).context("Failed to write .guvcode file")
    }

    fn global_config_path() -> Result<PathBuf> {
        if let Some(_proj_dirs) = ProjectDirs::from("com", "guv", "guvcode") {
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
