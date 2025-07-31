use std::fs;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::time::timeout;
use tracing::{debug, error, warn};

#[derive(Debug, Clone)]
pub enum TestOutcome {
    Survived,
    Killed { killing_tests: Vec<String> },
    Timeout,
    Error,
}

pub struct MutationRunner {
    timeout_duration: Duration,
    test_command: String,
}

#[allow(dead_code)]
impl MutationRunner {
    pub fn new(timeout_seconds: u64, test_command: String) -> Self {
        Self {
            timeout_duration: Duration::from_secs(timeout_seconds),
            test_command,
        }
    }
    pub async fn run_tests_for_mutation(&self, mutated_code: &str) -> TestOutcome {
        let start_time = Instant::now();

        let temp_dir = match tempdir() {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to create temporary directory: {}", e);
                return TestOutcome::Error;
            }
        };

        let temp_file_path = temp_dir.path().join("main.rs");
        if let Err(e) = fs::write(&temp_file_path, mutated_code) {
            error!("Failed to write mutated code to temp file: {}", e);
            return TestOutcome::Error;
        }

        match timeout(
            self.timeout_duration,
            self.execute_test_command(&temp_dir.path().to_path_buf()),
        )
        .await
        {
            Ok(Ok(exit_status)) => {
                let duration = start_time.elapsed();
                debug!(
                    "Test completed in {:?} with exit status: {}",
                    duration, exit_status
                );

                if exit_status == 0 {
                    TestOutcome::Survived
                } else {
                    // Simulate capturing killing test names (replace with actual logic)
                    let killing_tests = vec!["test_example_1".to_string(), "test_example_2".to_string()];
                    TestOutcome::Killed { killing_tests }
                }
            }
            Ok(Err(e)) => {
                error!("Test execution failed: {}", e);
                TestOutcome::Error
            }
            Err(_) => {
                warn!("Test execution timed out after {:?}", self.timeout_duration);
                TestOutcome::Timeout
            }
        }
    }

    async fn execute_test_command(
        &self,
        work_dir: &std::path::Path,
    ) -> Result<i32, std::io::Error> {
        debug!(
            "Executing test command: {} in {:?}",
            self.test_command, work_dir
        );

        let parts: Vec<&str> = self.test_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Empty test command",
            ));
        }

        let command = parts[0];
        let args = &parts[1..];

        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(work_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let output = cmd.output()?;
        Ok(output.status.code().unwrap_or(-1))
    }

    pub async fn run_baseline_tests(&self, original_code: &str) -> Result<bool, String> {
        debug!("Running baseline tests to ensure they pass");

        let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let temp_file_path = temp_dir.path().join("main.rs");

        fs::write(&temp_file_path, original_code)
            .map_err(|e| format!("Failed to write original code: {}", e))?;

        match timeout(
            self.timeout_duration,
            self.execute_test_command(&temp_dir.path().to_path_buf()),
        )
        .await
        {
            Ok(Ok(exit_status)) => {
                if exit_status == 0 {
                    debug!("Baseline tests passed");
                    Ok(true)
                } else {
                    warn!("Baseline tests failed with exit status: {}", exit_status);
                    Ok(false)
                }
            }
            Ok(Err(e)) => Err(format!("Failed to execute baseline tests: {}", e)),
            Err(_) => Err(format!(
                "Baseline tests timed out after {:?}",
                self.timeout_duration
            )),
        }
    }

    pub fn create_test_project_structure(
        &self,
        base_path: &std::path::Path,
        source_code: &str,
    ) -> Result<(), std::io::Error> {
        let cargo_toml_content = r#"[package]
name = "mutation_test"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        fs::write(base_path.join("Cargo.toml"), cargo_toml_content)?;

        let src_dir = base_path.join("src");
        fs::create_dir_all(&src_dir)?;

        fs::write(src_dir.join("lib.rs"), source_code)?;

        let main_content = r#"fn main() {
    println!("Mutation test project");
}
"#;
        fs::write(src_dir.join("main.rs"), main_content)?;

        Ok(())
    }

    pub async fn validate_test_setup(&self, source_code: &str) -> Result<(), String> {
        if !source_code.contains("#[test]") && !source_code.contains("#[cfg(test)]") {
            return Err("No test functions found in source code. Mutation testing requires tests to be effective.".to_string());
        }

        self.run_baseline_tests(source_code).await?;

        Ok(())
    }
}
