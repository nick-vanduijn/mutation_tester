use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::{CreateMutationTestRequest, MutationTest, MutationTestWithResults},
    services::mutation_service,
};

#[derive(Debug, Deserialize)]
pub struct ListMutationsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub language: Option<String>,
}

#[instrument(skip(state))]
pub async fn create_mutation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMutationTestRequest>,
) -> AppResult<Json<MutationTest>> {
    info!("Creating new mutation test: {}", request.name);

    let mutation_test = mutation_service::create_mutation_test(&state.db, request).await?;

    info!("Created mutation test with ID: {}", mutation_test.id);
    Ok(Json(mutation_test))
}

#[instrument(skip(state))]
pub async fn list_mutations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListMutationsQuery>,
) -> AppResult<Json<Vec<MutationTest>>> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);

    if limit > 100 {
        return Err(AppError::Validation("Limit cannot exceed 100".to_string()));
    }

    info!(
        "Listing mutation tests with page: {}, limit: {}",
        page, limit
    );

    let mutation_tests = mutation_service::list_mutation_tests(
        &state.db,
        page,
        limit,
        params.status,
        params.language,
    )
    .await?;

    Ok(Json(mutation_tests))
}

#[instrument(skip(state))]
pub async fn get_mutation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MutationTest>> {
    info!("Getting mutation test: {}", id);

    let mutation_test = mutation_service::get_mutation_test(&state.db, id).await?;

    match mutation_test {
        Some(test) => Ok(Json(test)),
        None => {
            warn!("Mutation test not found: {}", id);
            Err(AppError::NotFound(format!(
                "Mutation test with ID {} not found",
                id
            )))
        }
    }
}

#[instrument(skip(state))]
pub async fn get_mutation_results(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MutationTestWithResults>> {
    info!("Getting mutation test results: {}", id);

    let mutation_test_with_results =
        mutation_service::get_mutation_test_with_results(&state.db, id).await?;

    match mutation_test_with_results {
        Some(results) => Ok(Json(results)),
        None => {
            warn!("Mutation test not found: {}", id);
            Err(AppError::NotFound(format!(
                "Mutation test with ID {} not found",
                id
            )))
        }
    }
}

#[instrument(skip(state))]
pub async fn start_mutation_testing(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MutationTest>> {
    info!("Starting mutation testing: {}", id);

    let state_clone = state.clone();
    let mutation_test_id = id;

    tokio::spawn(async move {
        if let Err(e) =
            mutation_service::run_mutation_testing(&state_clone.db, mutation_test_id).await
        {
            tracing::error!("Mutation testing failed for {}: {}", mutation_test_id, e);
        }
    });

    let mutation_test = mutation_service::get_mutation_test(&state.db, id).await?;

    match mutation_test {
        Some(test) => Ok(Json(test)),
        None => {
            warn!("Mutation test not found: {}", id);
            Err(AppError::NotFound(format!(
                "Mutation test with ID {} not found",
                id
            )))
        }
    }
}

#[instrument(skip(state))]
pub async fn dry_run_mutation_testing(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::mutation::types::MutationCandidate>>> {
    info!("Running dry run for mutation test: {}", id);

    let candidates = mutation_service::dry_run_mutation_testing(&state.db, id).await?;

    Ok(Json(candidates))
}
