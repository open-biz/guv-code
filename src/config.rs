use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    #[serde(default, skip_serializing)]
    pub keys: Keys,
    pub budget: Budget,
    #[serde(default)]
    pub model: ModelChoice,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Provider {
    Google,
    Anthropic,
    OpenRouter,
}

impl Default for Provider {
    fn default() -> Self {
        Self::Google
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Google => write!(f, "Google"),
            Self::Anthropic => write!(f, "Anthropic"),
            Self::OpenRouter => write!(f, "OpenRouter"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelChoice {
    pub provider: Provider,
    pub model_id: String,
}

impl Default for ModelChoice {
    fn default() -> Self {
        Self {
            provider: Provider::Google,
            model_id: "gemini-2.5-flash".into(),
        }
    }
}

impl ModelChoice {
    pub fn display_name(&self) -> String {
        format!("{}/{}", self.provider, self.model_id)
    }
}

/// Static catalog of known models per provider.
pub fn model_catalog() -> Vec<(Provider, &'static str, &'static str)> {
    vec![
        (Provider::Google, "gemini-2.5-pro", "Most capable, best for complex tasks"),
        (Provider::Google, "gemini-2.5-flash", "Fast and efficient, good balance"),
        (Provider::Google, "gemini-2.5-flash-lite", "Lightweight, fastest response"),
        (Provider::Anthropic, "claude-sonnet-4-20250514", "Latest Sonnet, best balance"),
        (Provider::Anthropic, "claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet"),
        (Provider::Anthropic, "claude-3-5-haiku-20241022", "Fast, cost-effective"),
        (Provider::OpenRouter, "anthropic/claude-sonnet-4", "Claude Sonnet 4 via OpenRouter"),
        (Provider::OpenRouter, "google/gemini-2.5-pro", "Gemini 2.5 Pro via OpenRouter"),
        (Provider::OpenRouter, "google/gemini-2.5-flash", "Gemini 2.5 Flash via OpenRouter"),
        (Provider::OpenRouter, "deepseek/deepseek-r1", "DeepSeek R1 via OpenRouter"),
        (Provider::OpenRouter, "openai/gpt-4.1", "GPT-4.1 via OpenRouter"),
    ]
}

/// Get models for a specific provider.
pub fn models_for_provider(provider: &Provider) -> Vec<(&'static str, &'static str)> {
    model_catalog()
        .into_iter()
        .filter(|(p, _, _)| p == provider)
        .map(|(_, id, desc)| (id, desc))
        .collect()
}

impl Config {
    /// Auto-select the best model based on available credentials.
    /// Priority: explicit config > OpenRouter key > Gemini key > Anthropic key.
    /// Returns true if the model was changed.
    pub fn auto_select_model(&mut self) -> bool {
        // If user explicitly set a non-default model, respect it
        let default = ModelChoice::default();
        let is_default = self.model.provider == default.provider
            && self.model.model_id == default.model_id;

        // Check if current model's provider has a valid key
        let current_has_key = match self.model.provider {
            Provider::Google => self.keys.gemini.is_some(),
            Provider::Anthropic => self.keys.anthropic.is_some(),
            Provider::OpenRouter => self.keys.openrouter.is_some(),
        };

        // If current provider has a key, keep it
        if current_has_key && !is_default {
            return false;
        }

        // Auto-detect best available provider
        if self.keys.openrouter.is_some() {
            self.model = ModelChoice {
                provider: Provider::OpenRouter,
                model_id: "anthropic/claude-sonnet-4".into(),
            };
            return true;
        }
        if self.keys.gemini.is_some() {
            self.model = ModelChoice {
                provider: Provider::Google,
                model_id: "gemini-2.5-flash".into(),
            };
            return true;
        }
        if self.keys.anthropic.is_some() {
            self.model = ModelChoice {
                provider: Provider::Anthropic,
                model_id: "claude-sonnet-4-20250514".into(),
            };
            return true;
        }

        false
    }

    /// Get the API key for the currently selected provider.
    pub fn active_api_key(&self) -> Option<&str> {
        match self.model.provider {
            Provider::Google => self.keys.gemini.as_deref(),
            Provider::Anthropic => self.keys.anthropic.as_deref(),
            Provider::OpenRouter => self.keys.openrouter.as_deref(),
        }
    }

    /// Check if any provider has valid credentials.
    pub fn has_any_credentials(&self) -> bool {
        self.keys.gemini.is_some()
            || self.keys.anthropic.is_some()
            || self.keys.openrouter.is_some()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Keys {
    pub gemini: Option<String>,
    pub anthropic: Option<String>,
    pub openrouter: Option<String>,
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
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            config.keys.openrouter = Some(key);
        }

        // Also try loading OpenRouter key from ~/.guv/openrouter-key.env
        if config.keys.openrouter.is_none() {
            if let Some(home) = directories::UserDirs::new() {
                let key_file = home.home_dir().join(".guv").join("openrouter-key.env");
                if key_file.exists() {
                    if let Ok(content) = fs::read_to_string(&key_file) {
                        for line in content.lines() {
                            if let Some(val) = line.strip_prefix("OPENROUTER_API_KEY=") {
                                let val = val.trim_matches('"').trim();
                                if !val.is_empty() {
                                    config.keys.openrouter = Some(val.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Load credentials from auth store (~/.guv/credentials.json)
        //    NOTE: Google OAuth tokens do NOT work with generativelanguage.googleapis.com
        //    (returns ACCESS_TOKEN_TYPE_UNSUPPORTED). Only API keys work with that endpoint.
        //    We only bridge stored API keys here, not OAuth tokens.
        if let Ok(Some(creds)) = crate::auth::load_credentials() {
            match creds.auth_type {
                crate::auth::AuthType::GoogleOAuth => {
                    // Google OAuth tokens cannot be used with the Gemini API directly.
                    // User needs a Gemini API key from https://aistudio.google.com/apikey
                }
                crate::auth::AuthType::OpenRouter => {
                    if config.keys.openrouter.is_none() {
                        if let Some(ref key) = creds.api_key {
                            config.keys.openrouter = Some(key.clone());
                        }
                    }
                }
                crate::auth::AuthType::GeminiApiKey => {
                    if config.keys.gemini.is_none() {
                        if let Some(ref key) = creds.api_key {
                            config.keys.gemini = Some(key.clone());
                        }
                    }
                }
                crate::auth::AuthType::AnthropicApiKey => {
                    if config.keys.anthropic.is_none() {
                        if let Some(ref key) = creds.api_key {
                            config.keys.anthropic = Some(key.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        // 5. Auto-select model based on available credentials
        config.auto_select_model();

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
