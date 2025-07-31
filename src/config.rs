use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server_address: String,
    pub database_url: String,
    pub log_level: String,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: String,
    pub loki_endpoint: Option<String>,
    pub environment: String,
    pub service_name: String,
    pub service_version: String,
}

#[allow(dead_code)]
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:3000".to_string(),
            database_url: "postgresql://postgres:password@localhost:5432/mutation_tester"
                .to_string(),
            log_level: "info".to_string(),
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            prometheus_endpoint: "0.0.0.0:9090".to_string(),
            loki_endpoint: Some("http://localhost:3100".to_string()),
            environment: "development".to_string(),
            service_name: "mutation-tester-backend".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let mut builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP").separator("_"));

        if let Ok(config_file) = env::var("CONFIG_FILE") {
            builder = builder.add_source(File::with_name(&config_file).required(true));
        }

        let config = builder.build()?;
        config.try_deserialize()
    }

    #[allow(dead_code)]
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
}
