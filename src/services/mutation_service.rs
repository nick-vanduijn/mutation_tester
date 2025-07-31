use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{
        CreateMutationTestRequest, MutationResult, MutationTest, MutationTestStatus,
        MutationTestSummary, MutationTestWithResults, TestResult,
    },
    mutation::logger::MutationLogger,
    mutation::{
        engine::MutationEngine,
        types::{MutationTestConfig, TestOutcome},
    },
};

#[allow(dead_code)]
pub async fn create_mutation_test(
    pool: &PgPool,
    request: CreateMutationTestRequest,
) -> AppResult<MutationTest> {
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Mutation test name cannot be empty".to_string(),
        ));
    }

    let language = request.language.unwrap_or_else(|| "rust".to_string());

    let mutation_test = sqlx::query_as!(
        MutationTest,
        r#"
        INSERT INTO mutation_tests (name, description, source_code, language, status)
        VALUES ($1, $2, $3, $4, $5::mutation_test_status)
        RETURNING 
            id,
            name,
            description,
            source_code,
            language,
            status as "status: MutationTestStatus",
            created_at,
            updated_at,
            started_at,
            completed_at
        "#,
        request.name,
        request.description,
        request.source_code,
        language,
        MutationTestStatus::Pending as MutationTestStatus
    )
    .fetch_one(pool)
    .await?;

    info!("Created mutation test: {}", mutation_test.id);

    Ok(mutation_test)
}

#[allow(dead_code)]
pub async fn run_mutation_testing(
    pool: &PgPool,
    mutation_test_id: Uuid,
) -> AppResult<MutationTest> {
    let mut mutation_test =
        update_mutation_test_status(pool, mutation_test_id, MutationTestStatus::Running).await?;

    let config = MutationTestConfig::default();
    let engine = MutationEngine::new(config);

    match engine
        .run_mutation_testing(&mutation_test.source_code)
        .await
    {
        Ok(report) => {
            for result in report.results {
                MutationLogger::step(&format!(
                    "[API] Mutation at line {}, col {}: {:?} '{}' -> '{}' | Test result: {:?}",
                    result.candidate.line,
                    result.candidate.column,
                    result.candidate.mutation_type,
                    result.candidate.original_code,
                    result.mutated_code.chars().take(30).collect::<String>(),
                    result.test_result
                ));
                let test_result = match result.test_result {
                    TestOutcome::Killed { killing_tests } => {
                        println!("Mutation killed by tests: {:?}", killing_tests);
                        TestResult::Killed
                    }
                    TestOutcome::Survived => TestResult::Survived,
                    TestOutcome::Timeout => TestResult::Timeout,
                    TestOutcome::Error => TestResult::Error,
                    TestOutcome::Skipped => TestResult::Skipped,
                };

                let mutation_type = format!("{:?}", result.candidate.mutation_type);

                sqlx::query!(
                    r#"
                    INSERT INTO mutation_results 
                    (mutation_test_id, mutation_type, original_code, mutated_code, 
                     line_number, column_number, test_result, execution_time_ms, error_message)
                    VALUES ($1, $2, $3, $4, $5, $6, $7::test_result, $8, $9)
                    "#,
                    mutation_test_id,
                    mutation_type,
                    result.candidate.original_code,
                    result.mutated_code,
                    result.candidate.line as i32,
                    result.candidate.column as i32,
                    test_result as TestResult,
                    result.execution_time_ms as i64,
                    result.error_message
                )
                .execute(pool)
                .await?;
            }

            mutation_test =
                update_mutation_test_status(pool, mutation_test_id, MutationTestStatus::Completed)
                    .await?;
        }
        Err(error) => {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Mutation testing failed: {}",
                error
            )));
        }
    }

    Ok(mutation_test)
}

pub async fn dry_run_mutation_testing(
    pool: &PgPool,
    mutation_test_id: Uuid,
) -> AppResult<Vec<crate::mutation::types::MutationCandidate>> {
    let mutation_test = get_mutation_test(pool, mutation_test_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Mutation test {} not found", mutation_test_id))
        })?;

    let config = MutationTestConfig::default();
    let engine = MutationEngine::new(config);

    let candidates = engine
        .dry_run(&mutation_test.source_code)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Dry run failed: {}", e)))?;

    Ok(candidates)
}

pub async fn list_mutation_tests(
    pool: &PgPool,
    page: u32,
    limit: u32,
    status_filter: Option<String>,
    language_filter: Option<String>,
) -> AppResult<Vec<MutationTest>> {
    let offset = ((page - 1) * limit) as i64;
    let limit = limit as i64;

    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            id,
            name,
            description,
            source_code,
            language,
            status,
            created_at,
            updated_at,
            started_at,
            completed_at
        FROM mutation_tests
        WHERE 1=1
        "#,
    );

    if let Some(status) = status_filter {
        query.push(" AND status = ");
        query.push_bind(status);
        query.push("::mutation_test_status");
    }

    if let Some(language) = language_filter {
        query.push(" AND language = ");
        query.push_bind(language);
    }

    query.push(" ORDER BY created_at DESC LIMIT ");
    query.push_bind(limit);
    query.push(" OFFSET ");
    query.push_bind(offset);

    let mutation_tests = query
        .build_query_as::<MutationTest>()
        .fetch_all(pool)
        .await?;

    Ok(mutation_tests)
}

pub async fn get_mutation_test(pool: &PgPool, id: Uuid) -> AppResult<Option<MutationTest>> {
    let mutation_test = sqlx::query_as!(
        MutationTest,
        r#"
        SELECT 
            id,
            name,
            description,
            source_code,
            language,
            status as "status: MutationTestStatus",
            created_at,
            updated_at,
            started_at,
            completed_at
        FROM mutation_tests
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(mutation_test)
}

pub async fn get_mutation_test_with_results(
    pool: &PgPool,
    id: Uuid,
) -> AppResult<Option<MutationTestWithResults>> {
    let mutation_test = get_mutation_test(pool, id).await?;

    if let Some(test) = mutation_test {
        let results = get_mutation_results(pool, id).await?;
        let summary = MutationTestSummary::calculate(&results);

        Ok(Some(MutationTestWithResults {
            test,
            results,
            summary,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_mutation_results(
    pool: &PgPool,
    mutation_test_id: Uuid,
) -> AppResult<Vec<MutationResult>> {
    let results = sqlx::query_as!(
        MutationResult,
        r#"
        SELECT 
            id,
            mutation_test_id,
            mutation_type,
            original_code,
            mutated_code,
            line_number,
            column_number,
            test_result as "test_result: crate::models::TestResult",
            execution_time_ms,
            error_message,
            created_at,
            updated_at
        FROM mutation_results
        WHERE mutation_test_id = $1
        ORDER BY line_number, column_number
        "#,
        mutation_test_id
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn update_mutation_test_status(
    pool: &PgPool,
    id: Uuid,
    status: MutationTestStatus,
) -> AppResult<MutationTest> {
    let now = chrono::Utc::now();
    let (started_at, completed_at) = match status {
        MutationTestStatus::Running => (Some(now), None),
        MutationTestStatus::Completed
        | MutationTestStatus::Failed
        | MutationTestStatus::Cancelled => (None, Some(now)),
        _ => (None, None),
    };

    let mutation_test = sqlx::query_as!(
        MutationTest,
        r#"
        UPDATE mutation_tests 
        SET 
            status = $2::mutation_test_status,
            started_at = COALESCE($3, started_at),
            completed_at = COALESCE($4, completed_at),
            updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id,
            name,
            description,
            source_code,
            language,
            status as "status: MutationTestStatus",
            created_at,
            updated_at,
            started_at,
            completed_at
        "#,
        id,
        status as MutationTestStatus,
        started_at,
        completed_at
    )
    .fetch_one(pool)
    .await?;

    Ok(mutation_test)
}