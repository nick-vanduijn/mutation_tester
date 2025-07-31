use anyhow::Result;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tracing::{info, instrument};

pub type DatabasePool = PgPool;

#[instrument]
pub async fn setup_database(database_url: &str) -> Result<DatabasePool> {
    info!("Connecting to database");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(database_url)
        .await?;

    info!("Database connection established");
    Ok(pool)
}

#[instrument(skip(pool))]
pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    info!("Running database migrations");

    sqlx::migrate!("./migrations").run(pool).await?;

    info!("Database migrations completed successfully");
    Ok(())
}

#[instrument(skip(pool))]
pub async fn health_check(pool: &DatabasePool) -> Result<bool> {
    let result = sqlx::query("SELECT 1 as health").fetch_one(pool).await?;

    let health: i32 = result.get("health");
    Ok(health == 1)
}
