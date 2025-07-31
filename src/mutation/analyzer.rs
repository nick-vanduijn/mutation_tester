use crate::mutation::types::{MutationCandidate, MutationTestConfig, MutationType};
use tracing::{debug, instrument};

pub struct CodeAnalyzer {
    config: MutationTestConfig,
}

impl CodeAnalyzer {
    pub fn new(config: MutationTestConfig) -> Self {
        Self { config }
    }

    #[instrument(skip(self, source_code))]
    pub fn find_mutation_candidates(&self, source_code: &str) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let lines: Vec<&str> = source_code.lines().collect();

        for (line_number, line) in lines.iter().enumerate() {
            if self.should_skip_line(line) {
                continue;
            }

            candidates.extend(self.analyze_line(line, line_number + 1));
        }

        debug!("Found {} mutation candidates", candidates.len());
        candidates
    }

    fn should_skip_line(&self, line: &str) -> bool {
        for pattern in &self.config.excluded_patterns {
            if line.contains(pattern) {
                return true;
            }
        }
        if line.contains("// mutation-ignore") || line.contains("#[mutation_ignore]") {
            return true;
        }
        let trimmed = line.trim();
        trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("#")
            || trimmed.starts_with("/*")
            || trimmed.ends_with("*/")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("const ")
    }

    fn analyze_line(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();

        if self
            .config
            .mutation_types
            .contains(&MutationType::ArithmeticOperator)
        {
            candidates.extend(self.find_arithmetic_operators(line, line_number));
        }

        if self
            .config
            .mutation_types
            .contains(&MutationType::RelationalOperator)
        {
            candidates.extend(self.find_relational_operators(line, line_number));
        }

        if self
            .config
            .mutation_types
            .contains(&MutationType::LogicalOperator)
        {
            candidates.extend(self.find_logical_operators(line, line_number));
        }

        if self
            .config
            .mutation_types
            .contains(&MutationType::BooleanLiteral)
        {
            candidates.extend(self.find_boolean_literals(line, line_number));
        }

        if self
            .config
            .mutation_types
            .contains(&MutationType::NumericLiteral)
        {
            candidates.extend(self.find_numeric_literals(line, line_number));
        }

        if self
            .config
            .mutation_types
            .contains(&MutationType::ConditionalBoundary)
        {
            candidates.extend(self.find_conditional_boundaries(line, line_number));
        }

        candidates
    }

    fn find_arithmetic_operators(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let operators = ["+", "-", "*", "/", "%"];

        for op in &operators {
            let mut start = 0;
            while let Some(pos) = line[start..].find(op) {
                let actual_pos = start + pos;

                if self.is_standalone_operator(line, actual_pos, op) {
                    let mutations = self.get_arithmetic_mutations(op);
                    candidates.push(MutationCandidate {
                        line: line_number,
                        column: actual_pos + 1,
                        original_code: op.to_string(),
                        mutation_type: MutationType::ArithmeticOperator,
                        suggested_mutations: mutations,
                    });
                }
                start = actual_pos + 1;
            }
        }

        candidates
    }

    fn find_relational_operators(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let operators = ["==", "!=", "<", ">", "<=", ">="];

        for op in &operators {
            let mut start = 0;
            while let Some(pos) = line[start..].find(op) {
                let actual_pos = start + pos;
                let mutations = self.get_relational_mutations(op);
                candidates.push(MutationCandidate {
                    line: line_number,
                    column: actual_pos + 1,
                    original_code: op.to_string(),
                    mutation_type: MutationType::RelationalOperator,
                    suggested_mutations: mutations,
                });
                start = actual_pos + op.len();
            }
        }
        candidates
    }

    fn find_logical_operators(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let operators = ["&&", "||", "!"];

        for op in &operators {
            let mut start = 0;
            while let Some(pos) = line[start..].find(op) {
                let actual_pos = start + pos;
                let mutations = self.get_logical_mutations(op);
                candidates.push(MutationCandidate {
                    line: line_number,
                    column: actual_pos + 1,
                    original_code: op.to_string(),
                    mutation_type: MutationType::LogicalOperator,
                    suggested_mutations: mutations,
                });
                start = actual_pos + op.len();
            }
        }
        candidates
    }
    fn find_boolean_literals(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let literals = ["true", "false"];

        for literal in &literals {
            let mut start = 0;
            let mutation = if *literal == "true" { "false" } else { "true" };
            while let Some(pos) = line[start..].find(literal) {
                let actual_pos = start + pos;
                if self.is_complete_word(line, actual_pos, literal) {
                    candidates.push(MutationCandidate {
                        line: line_number,
                        column: actual_pos + 1,
                        original_code: literal.to_string(),
                        mutation_type: MutationType::BooleanLiteral,
                        suggested_mutations: vec![mutation.to_string()],
                    });
                }
                start = actual_pos + literal.len();
            }
        }
        candidates
    }
    fn find_numeric_literals(&self, line: &str, line_number: usize) -> Vec<MutationCandidate> {
        let mut candidates = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_ascii_digit() {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let literal: String = chars[start..i].iter().collect();
                candidates.push(MutationCandidate {
                    line: line_number,
                    column: start + 1,
                    original_code: literal.clone(),
                    mutation_type: MutationType::NumericLiteral,
                    suggested_mutations: self.get_numeric_mutations(&literal),
                });
            } else {
                i += 1;
            }
        }
        candidates
    }

    fn find_conditional_boundaries(
        &self,
        _line: &str,
        _line_number: usize,
    ) -> Vec<MutationCandidate> {
        Vec::new()
    }

    fn is_standalone_operator(&self, line: &str, pos: usize, op: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();

        if pos > 0 {
            let prev_char = chars[pos - 1];
            if "=!<>+-*/".contains(prev_char) {
                return false;
            }
        }

        let op_end = pos + op.len();
        if op_end < chars.len() {
            let next_char = chars[op_end];
            if "=!<>+-*/".contains(next_char) {
                return false;
            }
        }

        true
    }

    fn is_complete_word(&self, line: &str, pos: usize, word: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();

        if pos > 0 && (chars[pos - 1].is_alphanumeric() || chars[pos - 1] == '_') {
            return false;
        }

        let word_end = pos + word.len();
        if word_end < chars.len() && (chars[word_end].is_alphanumeric() || chars[word_end] == '_') {
            return false;
        }

        true
    }

    fn get_arithmetic_mutations(&self, operator: &str) -> Vec<String> {
        match operator {
            "+" => vec!["-".to_string(), "*".to_string()],
            "-" => vec!["+".to_string(), "*".to_string()],
            "*" => vec!["/".to_string(), "+".to_string()],
            "/" => vec!["*".to_string(), "%".to_string()],
            "%" => vec!["/".to_string(), "*".to_string()],
            _ => vec![],
        }
    }

    fn get_relational_mutations(&self, operator: &str) -> Vec<String> {
        match operator {
            "==" => vec!["!=".to_string(), "<".to_string(), ">".to_string()],
            "!=" => vec!["==".to_string()],
            "<" => vec!["<=".to_string(), ">".to_string(), "==".to_string()],
            ">" => vec![">=".to_string(), "<".to_string(), "==".to_string()],
            "<=" => vec!["<".to_string(), ">=".to_string()],
            ">=" => vec![">".to_string(), "<=".to_string()],
            _ => vec![],
        }
    }

    fn get_logical_mutations(&self, operator: &str) -> Vec<String> {
        match operator {
            "&&" => vec!["||".to_string()],
            "||" => vec!["&&".to_string()],
            "!" => vec!["".to_string()],
            _ => vec![],
        }
    }

    fn get_numeric_mutations(&self, number: &str) -> Vec<String> {
        if let Ok(num) = number.parse::<i32>() {
            vec![
                (num + 1).to_string(),
                (num - 1).to_string(),
                (num * -1).to_string(),
                "0".to_string(),
                "1".to_string(),
            ]
        } else if let Ok(num) = number.parse::<f64>() {
            vec![
                (num + 1.0).to_string(),
                (num - 1.0).to_string(),
                (num * -1.0).to_string(),
                "0.0".to_string(),
                "1.0".to_string(),
            ]
        } else {
            vec!["0".to_string(), "1".to_string()]
        }
    }
}
