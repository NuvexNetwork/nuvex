//! Process configuration.
//!
//! Values come from the environment. There is no default cluster RPC.

use std::fmt;

use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub health_bind: String,
    pub solana_rpc_url: Option<String>,
    pub log_format: String,
    /// Path is stored so operators can see that a key file was named.
    /// Milestone 0 does not open the file.
    pub key_path: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("NUVEX_NODE_HEALTH_BIND must be host:port, got `{0}`")]
    InvalidBind(String),
    #[error("NUVEX_LOG_FORMAT must be `json` or `text`, got `{0}`")]
    InvalidLogFormat(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut pairs = Vec::new();
        for key in [
            "NUVEX_NODE_HEALTH_BIND",
            "NUVEX_SOLANA_RPC_URL",
            "NUVEX_LOG_FORMAT",
            "NUVEX_NODE_KEY_PATH",
        ] {
            if let Ok(value) = std::env::var(key) {
                pairs.push((key.to_string(), value));
            }
        }
        let borrowed: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        Self::from_pairs(&borrowed)
    }

    pub fn from_pairs(pairs: &[(&str, &str)]) -> Result<Self, ConfigError> {
        let mut health_bind = "127.0.0.1:8081".to_string();
        let mut solana_rpc_url = None;
        let mut log_format = "json".to_string();
        let mut key_path = None;
        for (key, value) in pairs {
            if value.is_empty() {
                continue;
            }
            match *key {
                "NUVEX_NODE_HEALTH_BIND" => health_bind = (*value).to_string(),
                "NUVEX_SOLANA_RPC_URL" => solana_rpc_url = Some((*value).to_string()),
                "NUVEX_LOG_FORMAT" => log_format = (*value).to_string(),
                "NUVEX_NODE_KEY_PATH" => key_path = Some((*value).to_string()),
                _ => {}
            }
        }
        if !health_bind.contains(':') {
            return Err(ConfigError::InvalidBind(health_bind));
        }
        if log_format != "json" && log_format != "text" {
            return Err(ConfigError::InvalidLogFormat(log_format));
        }
        Ok(Self {
            health_bind,
            solana_rpc_url,
            log_format,
            key_path,
        })
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("health_bind", &self.health_bind)
            .field(
                "solana_rpc_url",
                &redact_url(self.solana_rpc_url.as_deref()),
            )
            .field("log_format", &self.log_format)
            .field("key_path", &self.key_path.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

fn redact_url(url: Option<&str>) -> &'static str {
    match url {
        Some(_) => "[redacted]",
        None => "[unset]",
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigError};

    #[test]
    fn defaults_do_not_select_a_cluster() {
        let config = Config::from_pairs(&[]).expect("defaults");
        assert!(config.solana_rpc_url.is_none());
        assert_eq!(config.health_bind, "127.0.0.1:8081");
        assert_eq!(config.log_format, "json");
    }

    #[test]
    fn debug_output_redacts_rpc_and_key_path() {
        let config = Config::from_pairs(&[
            (
                "NUVEX_SOLANA_RPC_URL",
                "https://example.invalid/?api-key=secret",
            ),
            ("NUVEX_NODE_KEY_PATH", "/var/lib/nuvex/node.json"),
        ])
        .expect("config");
        let rendered = format!("{config:?}");
        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("node.json"));
    }

    #[test]
    fn invalid_log_format_is_rejected() {
        let error = Config::from_pairs(&[("NUVEX_LOG_FORMAT", "pretty")]).expect_err("format");
        assert_eq!(error, ConfigError::InvalidLogFormat("pretty".to_string()));
    }
}
