use std::fs;
use std::path::Path;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use serde_yaml;
use toml;

use crate::mutation::types::{MutationTestConfig, MutationType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationConfigFile {
    pub timeout_seconds: Option<u64>,
    pub max_mutations_per_line: Option<usize>,
    pub excluded_patterns: Option<Vec<String>>,
    pub test_command: Option<String>,
    pub mutation_types: Option<Vec<String>>,
    pub excluded_mutations: Option<Vec<String>>,
    pub excluded_files: Option<Vec<String>>,
    pub excluded_functions: Option<Vec<String>>,
    pub min_coverage_percent: Option<f64>,
    pub parallel_jobs: Option<usize>,
    pub report_format: Option<String>,
    pub report_output_path: Option<String>,
    pub ast_mutations_enabled: Option<bool>,
}

#[allow(dead_code)] 
pub struct ConfigLoader;

#[allow(dead_code)]
impl ConfigLoader {
    
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
    
    #[allow(dead_code)]
    pub fn load_config(&self, config_path: Option<&str>) -> MutationTestConfig {
        let mut config = MutationTestConfig::default();
        
        if let Some(path) = config_path {
            if Path::new(path).exists() {
                match self.parse_config_file(path) {
                    Ok(file_config) => {
                        info!("Loading mutation configuration from {}", path);
                        self.apply_config(&mut config, file_config);
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {}: {}", path, e);
                    }
                }
            } else {
                warn!("Config file not found: {}", path);
            }
        } else {
            // Look for config in default locations
            let default_paths = [
                "flux.config.yaml",
                "flux.config.yml",
                "flux.config.toml",
                ".flux/config.yaml",
                ".flux/config.toml",
            ];
            
            for path in &default_paths {
                if Path::new(path).exists() {
                    match self.parse_config_file(path) {
                        Ok(file_config) => {
                            info!("Loading mutation configuration from {}", path);
                            self.apply_config(&mut config, file_config);
                            break;
                        }
                        Err(e) => {
                            warn!("Failed to parse config file {}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        config
    }
    
    #[allow(dead_code)]
    fn parse_config_file(&self, path: &str) -> Result<MutationConfigFile, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
            
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse YAML config: {}", e))
        } else if path.ends_with(".toml") {
            toml::from_str(&content)
                .map_err(|e| format!("Failed to parse TOML config: {}", e))
        } else {
            Err(format!("Unsupported config file format: {}", path))
        }
    }
    
    #[allow(dead_code)]
    fn apply_config(&self, config: &mut MutationTestConfig, file_config: MutationConfigFile) {
        if let Some(timeout) = file_config.timeout_seconds {
            config.timeout_seconds = timeout;
        }
        
        if let Some(max_mutations) = file_config.max_mutations_per_line {
            config.max_mutations_per_line = max_mutations;
        }
        
        if let Some(excluded_patterns) = file_config.excluded_patterns {
            config.excluded_patterns = excluded_patterns;
        }
        
        if let Some(test_command) = file_config.test_command {
            config.test_command = test_command;
        }
        
        if let Some(mutation_types) = file_config.mutation_types {
            let mut types = Vec::new();
            for type_str in mutation_types {
                match type_str.parse::<MutationType>() {
                    Ok(mutation_type) => types.push(mutation_type),
                    Err(e) => warn!("Invalid mutation type '{}': {}", type_str, e),
                }
            }
            if !types.is_empty() {
                config.mutation_types = types;
            }
        }
        
        if let Some(excluded_mutations) = file_config.excluded_mutations {
            let mut types = Vec::new();
            for type_str in excluded_mutations {
                match type_str.parse::<MutationType>() {
                    Ok(mutation_type) => types.push(mutation_type),
                    Err(e) => warn!("Invalid excluded mutation type '{}': {}", type_str, e),
                }
            }
            config.excluded_mutations = types;
        }
        
        if let Some(excluded_files) = file_config.excluded_files {
            config.excluded_files = excluded_files;
        }
        
        if let Some(excluded_functions) = file_config.excluded_functions {
            config.excluded_functions = excluded_functions;
        }
        
        if let Some(min_coverage) = file_config.min_coverage_percent {
            config.min_coverage_percent = Some(min_coverage);
        }
        
        if let Some(jobs) = file_config.parallel_jobs {
            config.parallel_jobs = Some(jobs);
        }
        
        if let Some(format_str) = file_config.report_format {
            let format = match format_str.to_lowercase().as_str() {
                "json" => crate::mutation::types::ReportFormat::JSON,
                "csv" => crate::mutation::types::ReportFormat::CSV,
                "html" => crate::mutation::types::ReportFormat::HTML,
                "markdown" | "md" => crate::mutation::types::ReportFormat::Markdown,
                _ => crate::mutation::types::ReportFormat::Console,
            };
            config.report_format = Some(format);
        }
        
        if let Some(output_path) = file_config.report_output_path {
            config.report_output_path = Some(output_path);
        }
        
        if let Some(ast_enabled) = file_config.ast_mutations_enabled {
            config.ast_mutations_enabled = ast_enabled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_load_yaml_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("flux.config.yaml");
        
        let config_content = r#"
timeout_seconds: 60
max_mutations_per_line: 10
test_command: "cargo test -- --test-threads=1"
mutation_types:
  - arithmetic
  - logical
  - boolean
excluded_mutations:
  - string
ast_mutations_enabled: true
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::new();
        let config = loader.load_config(Some(config_path.to_str().unwrap()));
        
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_mutations_per_line, 10);
        assert_eq!(config.test_command, "cargo test -- --test-threads=1");
        assert!(config.mutation_types.contains(&MutationType::ArithmeticOperator));
        assert!(config.mutation_types.contains(&MutationType::LogicalOperator));
        assert!(config.mutation_types.contains(&MutationType::BooleanLiteral));
        assert!(config.excluded_mutations.contains(&MutationType::StringLiteral));
        assert!(config.ast_mutations_enabled);
    }
    
    #[test]
    fn test_load_toml_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("flux.config.toml");
        
        let config_content = r#"
timeout_seconds = 45
max_mutations_per_line = 8
test_command = "cargo test"
mutation_types = ["relational", "numeric"]
parallel_jobs = 2
report_format = "html"
report_output_path = "./mutation-report"
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::new();
        let config = loader.load_config(Some(config_path.to_str().unwrap()));
        
        assert_eq!(config.timeout_seconds, 45);
        assert_eq!(config.max_mutations_per_line, 8);
        assert_eq!(config.test_command, "cargo test");
        assert!(config.mutation_types.contains(&MutationType::RelationalOperator));
        assert!(config.mutation_types.contains(&MutationType::NumericLiteral));
        assert_eq!(config.parallel_jobs, Some(2));
        assert_eq!(config.report_format, Some(crate::mutation::types::ReportFormat::HTML));
        assert_eq!(config.report_output_path, Some("./mutation-report".to_string()));
    }
    
    #[test]
    fn test_invalid_config_values() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("flux.config.yaml");
        
        let config_content = r#"
mutation_types:
  - arithmetic
  - invalid_type  # This should be ignored
  - logical
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let loader = ConfigLoader::new();
        let config = loader.load_config(Some(config_path.to_str().unwrap()));
        
        assert!(config.mutation_types.contains(&MutationType::ArithmeticOperator));
        assert!(config.mutation_types.contains(&MutationType::LogicalOperator));
        assert_eq!(config.mutation_types.len(), 2); // Only the valid types
    }
}
