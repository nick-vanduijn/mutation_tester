use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MutationTest {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_code: String,
    pub language: String,
    pub status: MutationTestStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "mutation_test_status", rename_all = "lowercase")]
pub enum MutationTestStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMutationTestRequest {
    pub name: String,
    pub description: Option<String>,
    pub source_code: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MutationResult {
    pub id: Uuid,
    pub mutation_test_id: Uuid,
    pub mutation_type: String,
    pub original_code: String,
    pub mutated_code: String,
    pub line_number: i32,
    pub column_number: Option<i32>,
    pub test_result: TestResult,
    pub execution_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "test_result", rename_all = "lowercase")]
pub enum TestResult {
    Pending,
    Killed,
    Survived,
    Timeout,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationTestSummary {
    pub total_mutations: i64,
    pub killed_mutations: i64,
    pub survived_mutations: i64,
    pub error_mutations: i64,
    pub timeout_mutations: i64,
    pub skipped_mutations: i64,
    pub mutation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationTestWithResults {
    #[serde(flatten)]
    pub test: MutationTest,
    pub results: Vec<MutationResult>,
    pub summary: MutationTestSummary,
}

impl MutationTestSummary {
    pub fn calculate(results: &[MutationResult]) -> Self {
        let total = results.len() as i64;
        let killed = results
            .iter()
            .filter(|r| matches!(r.test_result, TestResult::Killed))
            .count() as i64;
        let survived = results
            .iter()
            .filter(|r| matches!(r.test_result, TestResult::Survived))
            .count() as i64;
        let error = results
            .iter()
            .filter(|r| matches!(r.test_result, TestResult::Error))
            .count() as i64;
        let timeout = results
            .iter()
            .filter(|r| matches!(r.test_result, TestResult::Timeout))
            .count() as i64;
        let skipped = results
            .iter()
            .filter(|r| matches!(r.test_result, TestResult::Skipped))
            .count() as i64;

        let mutation_score = if total > 0 {
            (killed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_mutations: total,
            killed_mutations: killed,
            survived_mutations: survived,
            error_mutations: error,
            timeout_mutations: timeout,
            skipped_mutations: skipped,
            mutation_score,
        }
    }
}
