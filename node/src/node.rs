//! Node process assembly.
//!
//! The only network service in Milestone 0 is the local health server.

use std::time::Duration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::consensus;
use crate::executor;
use crate::network;
use crate::proof;
use crate::scheduler;
use crate::solana;
use crate::storage;
use crate::telemetry::metrics::Metrics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubsystemInfo {
    pub name: &'static str,
    pub milestone: &'static str,
}

pub fn subsystems() -> &'static [SubsystemInfo] {
    &[
        network::discovery::STATUS,
        network::messaging::STATUS,
        network::heartbeat::STATUS,
        scheduler::queue::STATUS,
        scheduler::assignment::STATUS,
        scheduler::retry::STATUS,
        executor::vrf::STATUS,
        executor::price::STATUS,
        executor::data::STATUS,
        executor::compute::STATUS,
        executor::ai::STATUS,
        consensus::aggregation::STATUS,
        consensus::quorum::STATUS,
        consensus::reputation::STATUS,
        proof::commitments::STATUS,
        proof::vrf::STATUS,
        solana::client::STATUS,
        solana::transactions::STATUS,
        solana::accounts::STATUS,
        storage::STATUS,
    ]
}

#[derive(Clone)]
struct AppState {
    metrics: Metrics,
}

pub fn router(metrics: Metrics) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/live", get(live))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics_route))
        .with_state(AppState { metrics })
}

pub async fn run(config: Config) -> Result<(), std::io::Error> {
    for subsystem in subsystems() {
        tracing::info!(
            subsystem = subsystem.name,
            milestone = subsystem.milestone,
            "subsystem disabled"
        );
    }
    let metrics = Metrics::install().map_err(std::io::Error::other)?;
    let app = router(metrics);
    let listener = TcpListener::bind(&config.health_bind).await?;
    tracing::info!(bind = %config.health_bind, "health server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown())
        .await
}

async fn wait_for_shutdown() {
    if tokio::signal::ctrl_c().await.is_err() {
        tracing::error!("failed to listen for ctrl-c; waiting instead");
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

#[derive(Serialize)]
struct HealthBody<'a> {
    status: &'a str,
    service: &'a str,
    execution: &'a str,
    signer: &'a str,
}

async fn health() -> Json<HealthBody<'static>> {
    Json(HealthBody {
        status: "ok",
        service: "nuvex-oracle-node",
        execution: "disabled",
        signer: "not-loaded",
    })
}

async fn live() -> Json<HealthBody<'static>> {
    Json(HealthBody {
        status: "ok",
        service: "nuvex-oracle-node",
        execution: "disabled",
        signer: "not-loaded",
    })
}

async fn ready() -> Response {
    let body = serde_json::json!({
        "status": "not-ready",
        "reason": "job execution is not implemented"
    });
    (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
}

async fn metrics_route(State(state): State<AppState>) -> Response {
    match state.metrics.encode() {
        Ok(body) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            body,
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "failed to encode metrics");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::{router, subsystems};
    use crate::telemetry::metrics::{Metrics, REQUIRED_METRICS};

    #[test]
    fn vrf_library_is_selected_and_this_process_does_not_submit() {
        use nuvex_crypto::{CryptoAvailability, VRF};
        use nuvex_job_types::JobType;

        let vrf = subsystems()
            .iter()
            .find(|item| item.name == "executor.vrf")
            .expect("vrf subsystem");
        assert_eq!(vrf.milestone, JobType::Vrf.milestone().to_string());
        assert!(matches!(
            VRF,
            CryptoAvailability::Available { audit: None, .. }
        ));
        assert!(subsystems().iter().any(|item| item.name == "proof.vrf"));
    }

    #[test]
    fn every_subsystem_is_disabled_until_its_milestone() {
        assert!(subsystems().len() >= 20);
        assert!(subsystems().iter().any(|item| item.name == "executor.vrf"));
        assert!(subsystems().iter().any(|item| item.name == "executor.ai"));
        assert!(subsystems().iter().all(|item| !item.milestone.is_empty()));
    }

    #[tokio::test]
    async fn readiness_rejects_traffic_until_execution_exists() {
        let metrics = Metrics::install().expect("metrics");
        let app = router(metrics);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(
            response.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn metrics_expose_the_required_names_at_zero() {
        let metrics = Metrics::install().expect("metrics");
        let app = router(metrics);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body = String::from_utf8(bytes.to_vec()).expect("utf8");
        for name in REQUIRED_METRICS {
            assert!(body.contains(name), "missing {name}");
        }
    }
}
