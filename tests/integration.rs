use flux_backend::{
    config::AppConfig,
    database::setup_database,
    models::{CreateMutationTestRequest, MutationTestStatus},
    services::mutation_service,
};
use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = setup_database(&config.database_url)
        .await
        .expect("Failed to setup database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_create_mutation_test() {
    let pool = setup_test_db().await;

    let request = CreateMutationTestRequest {
        name: "Test Mutation".to_string(),
        description: Some("A test mutation".to_string()),
        source_code: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        language: Some("rust".to_string()),
    };

    let result = mutation_service::create_mutation_test(&pool, request).await;
    assert!(result.is_ok());

    let mutation_test = result.unwrap();
    assert_eq!(mutation_test.name, "Test Mutation");
    assert_eq!(mutation_test.status, MutationTestStatus::Pending);
    assert_eq!(mutation_test.language, "rust");
}

#[tokio::test]
async fn test_get_mutation_test() {
    let pool = setup_test_db().await;

    let request = CreateMutationTestRequest {
        name: "Get Test".to_string(),
        description: Some("Test for get operation".to_string()),
        source_code: "fn test() -> i32 { 42 }".to_string(),
        language: Some("rust".to_string()),
    };

    let created = mutation_service::create_mutation_test(&pool, request)
        .await
        .unwrap();
    let retrieved = mutation_service::get_mutation_test(&pool, created.id).await;
    assert!(retrieved.is_ok());

    let test = retrieved.unwrap().unwrap();
    assert_eq!(test.id, created.id);
    assert_eq!(test.name, "Get Test");
}

#[tokio::test]
async fn test_list_mutation_tests() {
    let pool = setup_test_db().await;

    for i in 1..=3 {
        let request = CreateMutationTestRequest {
            name: format!("Test {}", i),
            description: Some(format!("Test description {}", i)),
            source_code: format!("fn test{}() -> i32 {{ {} }}", i, i),
            language: Some("rust".to_string()),
        };
        mutation_service::create_mutation_test(&pool, request)
            .await
            .unwrap();
    }

    let tests = mutation_service::list_mutation_tests(&pool, 1, 10, None, None).await;
    assert!(tests.is_ok());

    let test_list = tests.unwrap();
    assert!(test_list.len() >= 3);
}

#[tokio::test]
async fn test_dry_run_mutation_testing() {
    let pool = setup_test_db().await;

    let request = CreateMutationTestRequest {
        name: "Dry Run Test".to_string(),
        description: Some("Test dry run".to_string()),
        source_code: r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_add() {
                    assert_eq!(add(2, 3), 5);
                }
            }
            
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#
        .to_string(),
        language: Some("rust".to_string()),
    };

    let mutation_test = mutation_service::create_mutation_test(&pool, request)
        .await
        .unwrap();

    let candidates = mutation_service::dry_run_mutation_testing(&pool, mutation_test.id).await;
    assert!(candidates.is_ok());

    let mutation_candidates = candidates.unwrap();
    assert!(!mutation_candidates.is_empty());
}

#[tokio::test]
async fn test_mutation_test_lifecycle() {
    let pool = setup_test_db().await;

    let request = CreateMutationTestRequest {
        name: "Lifecycle Test".to_string(),
        description: Some("Test full lifecycle".to_string()),
        source_code: r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_add() {
                    assert_eq!(add(2, 3), 5);
                }
            }
            
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
                }
        "#
        .to_string(),
        language: Some("rust".to_string()),
    };

    let mutation_test = mutation_service::create_mutation_test(&pool, request)
        .await
        .unwrap();
    assert_eq!(mutation_test.status, MutationTestStatus::Pending);

    let result = mutation_service::run_mutation_testing(&pool, mutation_test.id).await;
    assert!(result.is_ok());

    let final_test = mutation_service::get_mutation_test(&pool, mutation_test.id)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        final_test.status,
        MutationTestStatus::Completed | MutationTestStatus::Failed
    ));

    let results = mutation_service::get_mutation_test_with_results(&pool, mutation_test.id).await;
    assert!(results.is_ok());

    let test_with_results = results.unwrap().unwrap();
    assert!(!test_with_results.results.is_empty());
}

#[tokio::test]
async fn test_create_mutation_test_empty_name() {
    let pool = setup_test_db().await;
    let request = CreateMutationTestRequest {
        name: "".to_string(),
        description: Some("desc".to_string()),
        source_code: "fn x() -> i32 { 1 }".to_string(),
        language: Some("rust".to_string()),
    };
    let result = mutation_service::create_mutation_test(&pool, request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_mutation_test_invalid_code() {
    let pool = setup_test_db().await;
    let request = CreateMutationTestRequest {
        name: "Invalid Code".to_string(),
        description: Some("desc".to_string()),
        source_code: "fn {".to_string(),
        language: Some("rust".to_string()),
    };
    let result = mutation_service::create_mutation_test(&pool, request).await;
    assert!(result.is_ok()); // Should still create, but mutation engine may fail later
}

#[tokio::test]
async fn test_list_mutation_tests_pagination() {
    let pool = setup_test_db().await;
    for i in 0..15 {
        let request = CreateMutationTestRequest {
            name: format!("Paginate {}", i),
            description: Some("desc".to_string()),
            source_code: format!("fn x{}() -> i32 {{ {} }}", i, i),
            language: Some("rust".to_string()),
        };
        mutation_service::create_mutation_test(&pool, request)
            .await
            .unwrap();
    }
    let page1 = mutation_service::list_mutation_tests(&pool, 1, 10, None, None)
        .await
        .unwrap();
    let page2 = mutation_service::list_mutation_tests(&pool, 2, 10, None, None)
        .await
        .unwrap();
    assert_eq!(page1.len(), 10);
    assert!(page2.len() >= 5);
}
