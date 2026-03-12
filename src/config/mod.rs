//! Configuration management for the Typefully CLI.

use std::path::PathBuf;

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// On-disk configuration file format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// The API key (stored as plain text in the TOML file).
    pub api_key: Option<String>,
    /// Default social set ID.
    pub default_social_set_id: Option<String>,
}

/// Resolved runtime configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The API key wrapped in a secret.
    pub api_key: Option<SecretString>,
    /// Default social set ID.
    pub default_social_set_id: Option<String>,
}

impl AppConfig {
    /// Returns the path to the config file.
    #[must_use]
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("typefully")
            .join("config.toml")
    }

    /// Load configuration from disk.
    pub fn load() -> Self {
        let file: ConfigFile = std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            api_key: file.api_key.map(SecretString::from),
            default_social_set_id: file.default_social_set_id,
        }
    }

    /// Save configuration to disk.
    pub fn save(&self) -> std::result::Result<(), ConfigError> {
        let file = ConfigFile {
            api_key: self.api_key.as_ref().map(|s| s.expose_secret().to_string()),
            default_social_set_id: self.default_social_set_id.clone(),
        };
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&file)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Resolve the API key from (in order): explicit flag, env var, config file.
    pub fn resolve_api_key(
        cli_key: Option<&str>,
    ) -> std::result::Result<SecretString, ConfigError> {
        if let Some(key) = cli_key
            && !key.is_empty()
        {
            return Ok(SecretString::from(key.to_string()));
        }
        if let Ok(key) = std::env::var("TYPEFULLY_API_KEY")
            && !key.is_empty()
        {
            return Ok(SecretString::from(key));
        }
        let config = Self::load();
        config.api_key.ok_or_else(|| {
            ConfigError::Missing(
                "No API key found. Set --api-key, TYPEFULLY_API_KEY env var, or run \
                 'typefully config init'."
                    .into(),
            )
        })
    }

    /// Resolve the social set ID from flag or config default.
    pub fn resolve_set_id(explicit: Option<&str>) -> std::result::Result<String, ConfigError> {
        if let Some(id) = explicit {
            return Ok(id.to_string());
        }
        let config = Self::load();
        config.default_social_set_id.ok_or_else(|| {
            ConfigError::Missing(
                "No social set ID provided. Use --set or run 'typefully config init' to set a \
                 default."
                    .into(),
            )
        })
    }
}
