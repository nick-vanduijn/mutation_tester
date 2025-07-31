use crate::mutation::engine::MutationEngine;
use crate::mutation::logger::MutationLogger;
use crate::mutation::types::MutationTestConfig;
use crate::mutation::types::{MutationJob, MutationType};
use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use clap::{Parser, Subcommand};
use futures_lite::stream::StreamExt;
use lapin::{BasicProperties, Connection, ConnectionProperties, options::*, types::FieldTable};
use reqwest;
use reqwest::Client;
use serde_json;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use toml;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tracing::info;

mod app;
mod config;
mod database;
mod error;
mod handlers;
mod models;
mod mutation;
mod services;

use crate::app::AppState;
use crate::config::AppConfig;
use crate::handlers::{health, mutations};

use dotenvy::dotenv;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    TestFiles {
        #[arg(required = false)]
        files: Vec<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        file_list: Option<String>,
        #[arg(long)]
        json: Option<String>,
        #[arg(long)]
        html: Option<String>,
        #[arg(long)]
        filter_types: Option<Vec<MutationType>>,
        #[arg(long)]
        webhook: Option<String>,
        #[arg(long)]
        databaseless: bool,
    },
    EnqueueJobs {
        #[arg(required = true)]
        files: Vec<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        queue_url: String,
        #[arg(long, default_value = "mutation_jobs")]
        queue_name: String,
        #[arg(long)]
        filter_types: Option<Vec<MutationType>>,
    },
    QueueRunner {
        #[arg(long)]
        queue_url: String,
        #[arg(long, default_value = "mutation_jobs")]
        queue_name: String,
        #[arg(long)]
        output_dir: Option<String>,
    },
    Wizard,
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<()> {
    dotenv().ok();

    if let Ok(env) = env::var("RUN_MODE") {
        println!("Running in {env} mode");
    }
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::TestFiles {
            files,
            config,
            file_list,
            json,
            html: _,
            filter_types: _,
            webhook,
            databaseless,
        }) => {
            let test_config = if let Some(cfg_path) = config {
                let cfg_str = fs::read_to_string(cfg_path)?;
                toml::from_str::<MutationTestConfig>(&cfg_str)?
            } else {
                MutationTestConfig::default()
            };

            let mut all_files = files.clone();
            if let Some(list_path) = file_list {
                let list_content = fs::read_to_string(list_path)?;
                for line in list_content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        all_files.push(trimmed.to_string());
                    }
                }
            }
            if all_files.is_empty() {
                MutationLogger::error("No files provided for mutation testing.");
                return Ok(());
            }

            let engine = MutationEngine::new(test_config.clone());
            let mut all_reports = Vec::new();
            for file in all_files {
                MutationLogger::info_file(&file, &format!("=== Mutation Testing ==="));
                let code = fs::read_to_string(&file)?;
                MutationLogger::step("Analyzing source code for mutation candidates...");
                match engine.run_mutation_testing(&code).await {
                    Ok(report) => {
                        all_reports.push((file.clone(), report.clone()));
                        MutationLogger::info_file(
                            &file,
                            &format!("Total mutations: {}", report.total_mutations),
                        );
                        MutationLogger::info_file(
                            &file,
                            &format!(
                                "Killed: {} | Survived: {} | Timeouts: {} | Errors: {} | Skipped: {}",
                                report.killed_mutations,
                                report.survived_mutations,
                                report.timeout_mutations,
                                report.error_mutations,
                                report.skipped_mutations
                            ),
                        );
                        MutationLogger::info_file(
                            &file,
                            &format!("Mutation Score: {:.1}%", report.mutation_score),
                        );
                        MutationLogger::info_file(
                            &file,
                            &format!("Execution Time: {:.2}s", report.execution_time_seconds),
                        );
                        if report.survived_mutations > 0 {
                            MutationLogger::warn(
                                "Some mutations survived. Consider improving your tests to catch these cases.",
                            );
                            MutationLogger::fix(
                                "Review survived mutations and add assertions or edge case tests.",
                            );
                        }
                        if report.error_mutations > 0 {
                            MutationLogger::error(
                                "Some mutations caused errors. Check for panics or unhandled cases in your code.",
                            );
                        }
                    }
                    Err(e) => {
                        MutationLogger::error_file(
                            &file,
                            &format!("Error running mutation testing for {}: {}", file, e),
                        );
                        MutationLogger::fix(
                            "Ensure the file compiles and contains valid Rust code with tests.",
                        );
                    }
                }
            }

            if let Some(json_path) = json {
                if all_reports.len() == 1 {
                    MutationLogger::info_file(&json_path, "Exported JSON report to");
                } else {
                    use std::collections::BTreeMap;
                    let mut map = BTreeMap::new();
                    for (file, report) in &all_reports {
                        map.insert(file, report);
                    }
                    let json = serde_json::to_string_pretty(&map)
                        .expect("Failed to serialize multi-file report");
                    std::fs::write(json_path, json)?;
                    MutationLogger::info_file(&json_path, "Exported JSON report to");
                }
            }

            if let Some(webhook_url) = webhook {
                if all_reports.len() == 1 {
                    let json = serde_json::to_string_pretty(&all_reports[0].1)?;
                    let client = Client::new();
                    match client
                        .post(webhook_url)
                        .header("Content-Type", "application/json")
                        .body(json)
                        .send()
                        .await
                    {
                        Ok(r) if r.status().is_success() => MutationLogger::info_file(
                            &webhook_url,
                            &format!("Posted results to webhook: {}", webhook_url),
                        ),
                        Ok(r) => MutationLogger::error_file(
                            &webhook_url,
                            &format!("Webhook POST failed: {}", r.status()),
                        ),
                        Err(e) => MutationLogger::error_file(
                            &webhook_url,
                            &format!("Webhook POST error: {}", e),
                        ),
                    }
                } else {
                    let json = serde_json::to_string_pretty(&all_reports)?;
                    let client = Client::new();
                    match client
                        .post(webhook_url)
                        .header("Content-Type", "application/json")
                        .body(json)
                        .send()
                        .await
                    {
                        Ok(r) if r.status().is_success() => MutationLogger::info_file(
                            &webhook_url,
                            &format!("Posted results to webhook: {}", webhook_url),
                        ),
                        Ok(r) => MutationLogger::error_file(
                            &webhook_url,
                            &format!("Webhook POST failed: {}", r.status()),
                        ),
                        Err(e) => MutationLogger::error_file(
                            &webhook_url,
                            &format!("Webhook POST error: {}", e),
                        ),
                    }
                }
            }
            if *databaseless {
                MutationLogger::info("Databaseless mode: skipping DB writes.");
            }
            Ok(())
        }
        Some(Commands::EnqueueJobs {
            files,
            config,
            queue_url,
            queue_name,
            filter_types,
        }) => {
            enqueue_jobs(
                files.clone(),
                config.clone(),
                queue_url,
                queue_name,
                filter_types.clone(),
            )
            .await?;
            Ok(())
        }
        Some(Commands::QueueRunner {
            queue_url,
            queue_name,
            output_dir,
        }) => {
            run_queue_runner(queue_url, queue_name, output_dir.clone()).await?;
            Ok(())
        }
        Some(Commands::Wizard) => {
            use std::io::{self, Write};
            println!("\nWelcome to the Mutation Tester Setup Wizard!\n");
            print!("Project name: ");
            io::stdout().flush().unwrap();
            let mut project = String::new();
            io::stdin().read_line(&mut project).unwrap();
            print!("Default test command [cargo test]: ");
            io::stdout().flush().unwrap();
            let mut test_cmd = String::new();
            io::stdin().read_line(&mut test_cmd).unwrap();
            let test_cmd = if test_cmd.trim().is_empty() {
                "cargo test".to_string()
            } else {
                test_cmd.trim().to_string()
            };
            print!("Mutation timeout (seconds) [30]: ");
            io::stdout().flush().unwrap();
            let mut timeout = String::new();
            io::stdin().read_line(&mut timeout).unwrap();
            let timeout = timeout.trim().parse().unwrap_or(30);
            let config = crate::mutation::types::MutationTestConfig {
                timeout_seconds: timeout,
                test_command: test_cmd,
                ..Default::default()
            };
            let config_toml = toml::to_string_pretty(&config).unwrap();
            std::fs::write("mutation_tester_config.toml", config_toml).unwrap();
            println!("\nConfig saved to mutation_tester_config.toml\n");
            println!("Next steps:");
            println!("  1. Add your Rust files and tests as usual.");
            println!(
                "  2. Run: cargo run -- test-files src/your_file.rs --config mutation_tester_config.toml"
            );
            println!("  3. Review colored logs and HTML/JSON reports.");
            println!("  4. Improve your tests for any surviving mutations.\n");
            println!("For more info, see the README.md or run with --help. Happy testing!\n");
            Ok(())
        }
        None => {
            let config = AppConfig::load()?;

            info!("Starting mutation tester backend service");

            let db = database::setup_database(&config.database_url).await?;

            database::run_migrations(&db).await?;

            let state = Arc::new(AppState {
                db,
                config: config.clone(),
            });

            let app = create_router(state);

            let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
            info!("Server listening on {}", config.server_address);

            axum::serve(listener, app).await?;

            Ok(())
        }
    }
}

fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/api/v1/mutations", post(mutations::create_mutation))
        .route("/api/v1/mutations", get(mutations::list_mutations))
        .route("/api/v1/mutations/:id", get(mutations::get_mutation))
        .route(
            "/api/v1/mutations/:id/results",
            get(mutations::get_mutation_results),
        )
        .route(
            "/api/v1/mutations/:id/start",
            post(mutations::start_mutation_testing),
        )
        .route(
            "/api/v1/mutations/:id/dry-run",
            get(mutations::dry_run_mutation_testing),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
}

async fn enqueue_jobs(
    files: Vec<String>,
    config: Option<String>,
    queue_url: &str,
    queue_name: &str,
    filter_types: Option<Vec<MutationType>>,
) -> anyhow::Result<()> {
    let test_config = if let Some(cfg_path) = config {
        let cfg_str = std::fs::read_to_string(cfg_path)?;
        Some(toml::from_str::<MutationTestConfig>(&cfg_str)?)
    } else {
        None
    };
    let conn = Connection::connect(queue_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    for file in &files {
        let job = MutationJob {
            file: file.clone(),
            config: test_config.clone(),
            filter_types: filter_types.clone(),
        };
        let payload = serde_json::to_vec(&job)?;
        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?
            .await?;
    }
    MutationLogger::info_file(
        &queue_name,
        &format!("Enqueued {} jobs to queue {}", files.len(), queue_name),
    );
    Ok(())
}

async fn run_queue_runner(
    queue_url: &str,
    queue_name: &str,
    output_dir: Option<String>,
) -> anyhow::Result<()> {
    let _ = output_dir;
    let conn = Connection::connect(queue_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut consumer = channel
        .basic_consume(
            queue_name,
            "mutation_tester_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let job: MutationJob = serde_json::from_slice(&delivery.data)?;
        MutationLogger::info_file(
            &job.file,
            &format!("Runner picked up job for file: {}", job.file),
        );
        let code = std::fs::read_to_string(&job.file)?;
        let config = job
            .config
            .clone()
            .unwrap_or_else(MutationTestConfig::default);
        let mut config = config;
        if let Some(filter) = &job.filter_types {
            config.mutation_types = filter.clone();
        }
        let engine = MutationEngine::new(config);
        let start = std::time::Instant::now();
        let report = engine.run_mutation_testing(&code).await;
        let elapsed = start.elapsed().as_secs_f64();
        MutationLogger::info_file(&job.file, &format!("Job completed in {:.2}s", elapsed));
        if let Ok(report) = &report {
            if report.survived_mutations > 0 {
                MutationLogger::warn(
                    "[Notify] Some mutations survived. Consider improving your tests.",
                );
            }
        }
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await?;
    }
    Ok(())
}
