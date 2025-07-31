use flux_backend::mutation::logger::MutationLogger;

#[test]
fn test_logger_failures() {
    MutationLogger::info("=== Mutation Testing: example.rs ===");
    MutationLogger::step("Analyzing source code for mutation candidates...");
    MutationLogger::info("Killed: 3 | Survived: 2 | Timeouts: 0 | Errors: 0 | Skipped: 0");
    MutationLogger::info("Mutation Score: 60.0%");
    MutationLogger::info("Execution Time: 1.23s");
    MutationLogger::warn(
        "Some mutations survived. Consider improving your tests to catch these cases.",
    );
    MutationLogger::fix("Review survived mutations and add assertions or edge case tests.");
}

#[test]
fn test_logger_errors() {
    MutationLogger::info("=== Mutation Testing: broken.rs ===");
    MutationLogger::step("Analyzing source code for mutation candidates...");
    MutationLogger::error("Error running mutation testing for broken.rs: failed to compile");
    MutationLogger::fix("Ensure the file compiles and contains valid Rust code with tests.");
}
