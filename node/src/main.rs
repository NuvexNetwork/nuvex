#![forbid(unsafe_code)]
//! Oracle node process.
//!
//! Milestone 0 runs a health and metrics server. It does not load signing
//! keys, connect to an RPC, or fulfill jobs.

use anyhow::Context;
use nuvex_oracle_node::config::Config;
use nuvex_oracle_node::node;
use nuvex_oracle_node::telemetry::tracing as node_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().context("invalid node configuration")?;
    node_tracing::init(&config.log_format).context("failed to initialize tracing")?;
    tracing::info!(
        execution = "disabled",
        health_bind = %config.health_bind,
        "nuvex oracle node starting in health-only mode"
    );
    node::run(config).await.context("node stopped")
}
