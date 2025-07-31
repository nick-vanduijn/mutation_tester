use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{error, instrument};

use crate::{app::AppState, database};

#[instrument]
pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "mutation-tester-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[instrument(skip(state))]
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    match database::health_check(&state.db).await {
        Ok(true) => Ok(Json(json!({
            "status": "ready",
            "service": "mutation-tester-backend",
            "version": env!("CARGO_PKG_VERSION"),
            "checks": {
                "database": "healthy"
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Ok(false) => {
            error!("Database health check returned false");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
