use crate::mutation::logger::MutationLogger;
use crate::mutation::{
    analyzer::CodeAnalyzer,
    mutators::CodeMutator,
    runner::MutationRunner,
    types::{MutationCandidate, MutationReport, MutationResult, MutationTestConfig, TestOutcome},
};
use std::time::Instant;
use tracing::{info, warn};
use rayon::prelude::*; 

pub struct MutationEngine {
    analyzer: CodeAnalyzer,
    mutator: CodeMutator,
    runner: MutationRunner,
    config: MutationTestConfig,
}

#[allow(dead_code)]
impl MutationEngine {
    pub fn new(config: MutationTestConfig) -> Self {
        let test_command = config.test_command.clone();
        let timeout = config.timeout_seconds;

        Self {
            analyzer: CodeAnalyzer::new(config.clone()),
            mutator: CodeMutator::new(),
            runner: MutationRunner::new(timeout, test_command),
            config,
        }
    }

    pub async fn run_mutation_testing(&self, source_code: &str) -> Result<MutationReport, String> {
        info!("Starting mutation testing");
        let start_time = Instant::now();

        self.runner.validate_test_setup(source_code).await?;
        info!("Test setup validation passed");

        let candidates = self.analyzer.find_mutation_candidates(source_code);
        info!("Found {} mutation candidates", candidates.len());

        if candidates.is_empty() {
            warn!("No mutation candidates found in source code");
            return Ok(MutationReport::new());
        }

        let mut report = MutationReport::new();

        let results: Vec<Vec<MutationResult>> = candidates
            .par_iter()
            .map(|candidate| {
                tokio::runtime::Handle::current().block_on(self.process_candidate(source_code, candidate))
            })
            .collect();

        for mutation_results in results {
            for result in mutation_results {
                report.add_result(result);
            }
        }

        let total_time = start_time.elapsed();
        report.execution_time_seconds = total_time.as_secs_f64();

        info!(
            "Mutation testing completed in {:.2}s. Score: {:.1}% ({}/{} killed)",
            report.execution_time_seconds,
            report.mutation_score,
            report.killed_mutations,
            report.total_mutations
        );

        Ok(report)
    }

    async fn process_candidate(
        &self,
        source_code: &str,
        candidate: &MutationCandidate,
    ) -> Vec<MutationResult> {
        let mut results = Vec::new();

        for mutation in &candidate.suggested_mutations {
            let start_time = Instant::now();
            MutationLogger::step(&format!(
                "Applying mutation at line {}, col {}: {:?} '{}' -> '{}'",
                candidate.line,
                candidate.column,
                candidate.mutation_type,
                candidate.original_code,
                mutation
            ));
            match self
                .mutator
                .apply_mutation(source_code, candidate, mutation)
            {
                Ok(mutated_code) => {
                    MutationLogger::info(&format!(
                        "Testing mutated code: {}",
                        Self::shorten_code(&mutated_code)
                    ));
                    let test_result = self.runner.run_tests_for_mutation(&mutated_code).await;
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    let test_outcome: TestOutcome = test_result.clone().into();

                    MutationLogger::info(&format!(
                        "Test outcome for mutation at line {}, col {}: {:?} (Execution time: {} ms)",
                        candidate.line, candidate.column, test_outcome, execution_time
                    ));

                    let killing_tests = if let TestOutcome::Killed { killing_tests } = &test_outcome {
                        Some(killing_tests.clone())
                    } else {
                        None
                    };

                    results.push(MutationResult {
                        candidate: candidate.clone(),
                        mutated_code,
                        test_result: test_outcome.clone(),
                        execution_time_ms: execution_time,
                        error_message: None,
                        killing_tests,
                        suggested_improvement: if matches!(test_outcome, TestOutcome::Survived) {
                            Some("Add or improve tests to catch this mutation (e.g., assert on edge cases or logic).".to_string())
                        } else {
                            None
                        },
                    });
                }
                Err(error) => {
                    MutationLogger::error(&format!(
                        "Failed to apply mutation at line {}, col {}: {}",
                        candidate.line, candidate.column, error
                    ));
                    let execution_time = start_time.elapsed().as_millis() as u64;
                    results.push(MutationResult {
                        candidate: candidate.clone(),
                        mutated_code: String::new(),
                        test_result: TestOutcome::Error,
                        execution_time_ms: execution_time,
                        error_message: Some(error),
                        killing_tests: None,
                        suggested_improvement: None,
                    });
                }
            }
        }

        results
    }

    fn shorten_code(code: &str) -> String {
        let trimmed = code.trim();
        if trimmed.len() > 60 {
            format!("{}...", &trimmed[..60])
        } else {
            trimmed.to_string()
        }
    }

    pub fn get_config(&self) -> &MutationTestConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: MutationTestConfig) {
        self.config = config.clone();
        self.analyzer = CodeAnalyzer::new(config.clone());
        self.runner = MutationRunner::new(config.timeout_seconds, config.test_command.clone());
    }

    pub async fn dry_run(&self, source_code: &str) -> Result<Vec<MutationCandidate>, String> {
        info!("Running dry run to find mutation candidates");

        let candidates = self.analyzer.find_mutation_candidates(source_code);

        info!("Dry run found {} potential mutations:", candidates.len());
        for (index, candidate) in candidates.iter().enumerate() {
            info!(
                "  {}: Line {}, Col {} - {:?} '{}' -> {:?}",
                index + 1,
                candidate.line,
                candidate.column,
                candidate.mutation_type,
                candidate.original_code,
                candidate.suggested_mutations
            );
        }

        Ok(candidates)
    }

    pub async fn test_single_mutation(
        &self,
        source_code: &str,
        candidate: &MutationCandidate,
        mutation: &str,
    ) -> Result<MutationResult, String> {
        let start_time = Instant::now();

        let mutated_code = self
            .mutator
            .apply_mutation(source_code, candidate, mutation)?;

        let test_result_runner = self.runner.run_tests_for_mutation(&mutated_code).await;
        let test_result: TestOutcome = test_result_runner.into();

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(MutationResult {
            candidate: candidate.clone(),
            mutated_code,
            test_result: test_result.clone(),
            execution_time_ms: execution_time,
            error_message: None,
            killing_tests: match &test_result {
                TestOutcome::Killed { killing_tests } => Some(killing_tests.clone()),
                _ => None,
            },
            suggested_improvement: match test_result {
                TestOutcome::Survived => Some("Add or improve tests to catch this mutation (e.g., assert on edge cases or logic).".to_string()),
                _ => None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation::types::MutationType;

    #[test]
    fn test_mutation_engine_creation() {
        let config = MutationTestConfig {
            timeout_seconds: 30,
            max_mutations_per_line: 100,
            excluded_patterns: vec![],
            test_command: "cargo test".to_string(),
            mutation_types: vec![
                MutationType::ArithmeticOperator,
                MutationType::NumericLiteral,
            ],
            excluded_mutations: vec![],
            excluded_files: vec![],
            excluded_functions: vec![],
            min_coverage_percent: Some(75.0),
            parallel_jobs: Some(4),
            report_format: Some(crate::mutation::types::ReportFormat::Console),
            report_output_path: None,
            ast_mutations_enabled: false,
        };

        let engine = MutationEngine::new(config);
        assert_eq!(engine.config.timeout_seconds, 30);
        assert_eq!(engine.config.max_mutations_per_line, 100);
    }

    #[test]
    fn test_mutation_engine_config_update() {
        let mut engine = MutationEngine::new(MutationTestConfig::default());

        let new_config = MutationTestConfig {
            timeout_seconds: 60,
            max_mutations_per_line: 200,
            excluded_patterns: vec![],
            test_command: "cargo test".to_string(),
            mutation_types: vec![MutationType::LogicalOperator],
            excluded_mutations: vec![],
            excluded_files: vec![],
            excluded_functions: vec![],
            min_coverage_percent: Some(80.0),
            parallel_jobs: Some(8),
            report_format: Some(crate::mutation::types::ReportFormat::JSON),
            report_output_path: Some("reports/".to_string()),
            ast_mutations_enabled: true,
        };

        engine.update_config(new_config);
        assert_eq!(engine.config.timeout_seconds, 60);
        assert_eq!(engine.config.max_mutations_per_line, 200);
    }

    #[test]
    fn test_mutation_engine_get_config() {
        let config = MutationTestConfig {
            timeout_seconds: 45,
            max_mutations_per_line: 150,
            excluded_patterns: vec![],
            test_command: "cargo test".to_string(),
            mutation_types: vec![MutationType::ArithmeticOperator],
            excluded_mutations: vec![],
            excluded_files: vec![],
            excluded_functions: vec![],
            min_coverage_percent: Some(70.0),
            parallel_jobs: Some(2),
            report_format: Some(crate::mutation::types::ReportFormat::Markdown),
            report_output_path: None,
            ast_mutations_enabled: false,
        };

        let engine = MutationEngine::new(config.clone());
        let retrieved_config = engine.get_config();

        assert_eq!(retrieved_config.timeout_seconds, config.timeout_seconds);
        assert_eq!(
            retrieved_config.max_mutations_per_line,
            config.max_mutations_per_line
        );
        assert_eq!(retrieved_config.mutation_types, config.mutation_types);
    }

    #[test]
    fn test_mutation_engine_default_config() {
        let config = MutationTestConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_mutations_per_line, 5);
        assert!(!config.mutation_types.is_empty());
    }
}
