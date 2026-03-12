//! Error types for the Typefully CLI.

/// Top-level error type for the application.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// An error returned by the Typefully API.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// A configuration error.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// An I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid usage (bad flags, missing arguments).
    #[error("{0}")]
    Usage(String),
}

impl AppError {
    /// Returns the exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            _ => 1,
        }
    }
}

/// An error returned by the Typefully API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ApiError {
    /// HTTP-level error from reqwest.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    Response {
        /// HTTP status code.
        status: u16,
        /// Error message from the API.
        message: String,
    },

    /// Rate limited by the API.
    #[error("Rate limited. Retry after {retry_after} seconds.")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after: String,
    },

    /// Failed to deserialize the API response.
    #[error("Failed to parse API response: {0}")]
    Deserialization(#[from] serde_json::Error),
}

/// A configuration error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Failed to read or write the config file.
    #[error("Config I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse the config file.
    #[error("Config parse error: {0}")]
    Parse(#[from] toml::de::Error),

    /// Failed to serialize the config.
    #[error("Config serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),

    /// Missing required configuration value.
    #[error("{0}")]
    Missing(String),
}

/// Convenience result type.
pub type Result<T> = std::result::Result<T, AppError>;

/// Exit code for invalid usage.
pub const EXIT_USAGE: i32 = 2;

/// Exit code for runtime/API errors.
pub const EXIT_ERROR: i32 = 1;

/// Helper to display an optional field or a fallback.
#[must_use]
pub fn display_or(val: &Option<String>, fallback: &str) -> String {
    val.as_deref().unwrap_or(fallback).to_string()
}
