//! Process metrics.
//!
//! Counters start at zero because no job has run. A zero is an observation,
//! not a substitute for a completed request.

use std::sync::Arc;

use prometheus::{
    Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry, TextEncoder,
};

pub const REQUIRED_METRICS: &[&str] = &[
    "oracle_requests_total",
    "oracle_requests_failed",
    "oracle_request_latency",
    "node_jobs_total",
    "node_job_failures",
    "node_uptime",
    "node_rewards",
    "verification_failures",
    "callback_failures",
    "rpc_errors",
    "database_latency",
    "queue_depth",
];

const COUNTERS: &[&str] = &[
    "oracle_requests_total",
    "oracle_requests_failed",
    "node_jobs_total",
    "node_job_failures",
    "node_rewards",
    "verification_failures",
    "callback_failures",
    "rpc_errors",
];

const HISTOGRAMS: &[&str] = &["oracle_request_latency", "database_latency"];

const GAUGES: &[&str] = &["node_uptime", "queue_depth"];

#[derive(Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
}

impl Metrics {
    pub fn install() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        for name in COUNTERS {
            let counter = IntCounter::with_opts(Opts::new(*name, "Nuvex counter"))?;
            registry.register(Box::new(counter))?;
        }
        for name in HISTOGRAMS {
            let histogram = Histogram::with_opts(HistogramOpts::new(*name, "Nuvex histogram"))?;
            registry.register(Box::new(histogram))?;
        }
        for name in GAUGES {
            let gauge = IntGauge::with_opts(Opts::new(*name, "Nuvex gauge"))?;
            registry.register(Box::new(gauge))?;
        }
        Ok(Self {
            registry: Arc::new(registry),
        })
    }

    pub fn encode(&self) -> Result<String, prometheus::Error> {
        let families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&families, &mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|error| prometheus::Error::Msg(format!("metrics were not utf-8: {error}")))
    }
}
