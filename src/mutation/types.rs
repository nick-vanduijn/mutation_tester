impl From<crate::mutation::runner::TestOutcome> for TestOutcome {
    fn from(runner_outcome: crate::mutation::runner::TestOutcome) -> Self {
        match runner_outcome {
            crate::mutation::runner::TestOutcome::Killed { killing_tests } => TestOutcome::Killed { killing_tests },
            crate::mutation::runner::TestOutcome::Survived => TestOutcome::Survived,
            crate::mutation::runner::TestOutcome::Timeout => TestOutcome::Timeout,
            crate::mutation::runner::TestOutcome::Error => TestOutcome::Error,
        }
    }
}
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationCandidate {
    pub line: usize,
    pub column: usize,
    pub original_code: String,
    pub mutation_type: MutationType,
    pub suggested_mutations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ValueEnum)]
pub enum MutationType {
    // Operator mutations
    ArithmeticOperator,
    RelationalOperator,
    LogicalOperator,
    AssignmentOperator,
    BitwiseOperator,
    IncrementDecrement,
    
    // Literal mutations
    BooleanLiteral,
    NumericLiteral,
    StringLiteral,
    CharLiteral,
    
    // Boundary mutations
    ConditionalBoundary,
    LoopBoundary,
    
    // Control flow mutations
    StatementDeletion,
    ReturnValue,
    BreakContinueReplacement,
    
    // Pattern-based mutations
    NullCheck,
    OptionalUnwrap,
    VariableReference,
    FunctionCall,
    
    // Advanced mutations (requires AST)
    ConstantReplacement,
    MethodChain,
    ExceptionHandling,
    SwitchCase,
}

impl FromStr for MutationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // Operator mutations
            "arithmeticoperator" | "arithmetic" => Ok(MutationType::ArithmeticOperator),
            "relationaloperator" | "relational" => Ok(MutationType::RelationalOperator),
            "logicaloperator" | "logical" => Ok(MutationType::LogicalOperator),
            "assignmentoperator" | "assignment" => Ok(MutationType::AssignmentOperator),
            "bitwiseoperator" | "bitwise" => Ok(MutationType::BitwiseOperator),
            "incrementdecrement" | "increment" => Ok(MutationType::IncrementDecrement),
            
            // Literal mutations
            "booleanliteral" | "boolean" => Ok(MutationType::BooleanLiteral),
            "numericliteral" | "numeric" => Ok(MutationType::NumericLiteral),
            "stringliteral" | "string" => Ok(MutationType::StringLiteral),
            "charliteral" | "char" => Ok(MutationType::CharLiteral),
            
            // Boundary mutations
            "conditionalboundary" | "conditional" => Ok(MutationType::ConditionalBoundary),
            "loopboundary" | "loop" => Ok(MutationType::LoopBoundary),
            
            // Control flow mutations
            "statementdeletion" | "statement" => Ok(MutationType::StatementDeletion),
            "returnvalue" | "return" => Ok(MutationType::ReturnValue),
            "breakcontinuereplacement" | "breakreplacement" => Ok(MutationType::BreakContinueReplacement),
            
            // Pattern-based mutations
            "nullcheck" | "null" => Ok(MutationType::NullCheck),
            "optionalunwrap" | "optional" => Ok(MutationType::OptionalUnwrap),
            "variablereference" | "variable" => Ok(MutationType::VariableReference),
            "functioncall" | "function" => Ok(MutationType::FunctionCall),
            
            // Advanced mutations (requires AST)
            "constantreplacement" | "constant" => Ok(MutationType::ConstantReplacement),
            "methodchain" | "chain" => Ok(MutationType::MethodChain),
            "exceptionhandling" | "exception" => Ok(MutationType::ExceptionHandling),
            "switchcase" | "switch" => Ok(MutationType::SwitchCase),
            
            _ => Err(format!("Unknown mutation type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub candidate: MutationCandidate,
    pub mutated_code: String,
    pub test_result: TestOutcome,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub killing_tests: Option<Vec<String>>,
    pub suggested_improvement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestOutcome {
    Killed { killing_tests: Vec<String> },
    Survived,
    Timeout,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportFormat {
    JSON,
    CSV,
    HTML,
    Markdown,
    Console
}

impl Default for ReportFormat {
    fn default() -> Self {
        Self::Console
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationTestConfig {
    pub timeout_seconds: u64,
    pub max_mutations_per_line: usize,
    pub excluded_patterns: Vec<String>,
    pub test_command: String,
    pub mutation_types: Vec<MutationType>,
    pub excluded_mutations: Vec<MutationType>,
    pub excluded_files: Vec<String>,
    pub excluded_functions: Vec<String>,
    pub min_coverage_percent: Option<f64>,
    pub parallel_jobs: Option<usize>,
    pub report_format: Option<ReportFormat>,
    pub report_output_path: Option<String>,
    pub ast_mutations_enabled: bool,
}

impl Default for MutationTestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_mutations_per_line: 5,
            excluded_patterns: vec![
                "// @no-mutation".to_string(),
                "#[cfg(test)]".to_string(),
                "#[test]".to_string(),
            ],
            test_command: "cargo test".to_string(),
            mutation_types: vec![
                MutationType::ArithmeticOperator,
                MutationType::RelationalOperator,
                MutationType::LogicalOperator,
                MutationType::BooleanLiteral,
                MutationType::NumericLiteral,
                MutationType::ConditionalBoundary,
            ],
            excluded_mutations: vec![],
            excluded_files: vec![],
            excluded_functions: vec![],
            min_coverage_percent: Some(75.0),
            parallel_jobs: Some(4),
            report_format: Some(ReportFormat::Console),
            report_output_path: None,
            ast_mutations_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationReport {
    pub total_mutations: usize,
    pub killed_mutations: usize,
    pub survived_mutations: usize,
    pub error_mutations: usize,
    pub timeout_mutations: usize,
    pub skipped_mutations: usize,
    pub mutation_score: f64,
    pub execution_time_seconds: f64,
    pub results: Vec<MutationResult>,
}

impl MutationReport {
    pub fn new() -> Self {
        Self {
            total_mutations: 0,
            killed_mutations: 0,
            survived_mutations: 0,
            error_mutations: 0,
            timeout_mutations: 0,
            skipped_mutations: 0,
            mutation_score: 0.0,
            execution_time_seconds: 0.0,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: MutationResult) {
        self.total_mutations += 1;
        self.execution_time_seconds += result.execution_time_ms as f64 / 1000.0;

        match result.test_result {
            TestOutcome::Killed { .. } => self.killed_mutations += 1,
            TestOutcome::Survived => self.survived_mutations += 1,
            TestOutcome::Error => self.error_mutations += 1,
            TestOutcome::Timeout => self.timeout_mutations += 1,
            TestOutcome::Skipped => self.skipped_mutations += 1,
        }

        self.results.push(result);
        self.calculate_score();
    }

    fn calculate_score(&mut self) {
        let detected = self.killed_mutations + self.timeout_mutations;
        let total_tested = self.total_mutations - self.skipped_mutations - self.error_mutations;

        if total_tested > 0 {
            self.mutation_score = (detected as f64 / total_tested as f64) * 100.0;
        } else {
            self.mutation_score = 0.0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationJob {
    pub file: String,
    pub config: Option<MutationTestConfig>,
    pub filter_types: Option<Vec<MutationType>>,
}
