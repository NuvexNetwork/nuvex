//! Structured logging.
//!
//! The process never logs environment dumps, key paths, or RPC query strings.
//! Callers that log [`crate::config::Config`] go through its redacting `Debug`.

use tracing_subscriber::EnvFilter;

#[derive(Debug)]
pub struct TracingError;

impl std::fmt::Display for TracingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("tracing subscriber was already initialized")
    }
}

impl std::error::Error for TracingError {}

pub fn init(format: &str) -> Result<(), TracingError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    let result = if format == "json" {
        builder.json().try_init()
    } else {
        builder.try_init()
    };
    result.map_err(|_| TracingError)
}
