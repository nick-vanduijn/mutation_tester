use crate::mutation::types::{MutationCandidate, MutationType};
use tracing::debug;

pub struct CodeMutator;

#[allow(dead_code)]
impl CodeMutator {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_mutation(
        &self,
        source_code: &str,
        candidate: &MutationCandidate,
        mutation: &str,
    ) -> Result<String, String> {
        if !candidate
            .suggested_mutations
            .contains(&mutation.to_string())
        {
            return Err(format!(
                "Mutation '{}' is not in the suggested mutations list: {:?}",
                mutation, candidate.suggested_mutations
            ));
        }

        let lines: Vec<&str> = source_code.lines().collect();

        if candidate.line == 0 || candidate.line > lines.len() {
            return Err(format!("Invalid line number: {}", candidate.line));
        }

        let mut mutated_lines = lines.clone();
        let target_line = lines[candidate.line - 1];

        let mutated_line = self.apply_line_mutation(target_line, candidate, mutation)?;

        mutated_lines[candidate.line - 1] = &mutated_line;

        let mutated_code = mutated_lines.join("\n");
        debug!(
            "Applied mutation: {} -> {}",
            candidate.original_code, mutation
        );

        Ok(mutated_code)
    }

    fn apply_line_mutation(
        &self,
        line: &str,
        candidate: &MutationCandidate,
        mutation: &str,
    ) -> Result<String, String> {
        let target_pos = candidate.column.saturating_sub(1);

        match candidate.mutation_type {
            MutationType::ArithmeticOperator
            | MutationType::RelationalOperator
            | MutationType::LogicalOperator => self.replace_operator_at_position(
                line,
                target_pos,
                &candidate.original_code,
                mutation,
            ),
            MutationType::BooleanLiteral => self.replace_literal_at_position(
                line,
                target_pos,
                &candidate.original_code,
                mutation,
            ),
            MutationType::NumericLiteral => self.replace_literal_at_position(
                line,
                target_pos,
                &candidate.original_code,
                mutation,
            ),
            MutationType::ConditionalBoundary => {
                self.replace_condition_at_position(line, target_pos, mutation)
            }
            _ => Err(format!(
                "Unsupported mutation type: {:?}",
                candidate.mutation_type
            )),
        }
    }

    fn replace_operator_at_position(
        &self,
        line: &str,
        pos: usize,
        original: &str,
        replacement: &str,
    ) -> Result<String, String> {
        if pos >= line.len() {
            return Err("Position out of bounds".to_string());
        }

        let chars: Vec<char> = line.chars().collect();
        let original_chars: Vec<char> = original.chars().collect();

        if pos + original_chars.len() > chars.len() {
            return Err("Original text extends beyond line".to_string());
        }

        let slice_at_pos: String = chars[pos..pos + original_chars.len()].iter().collect();
        if slice_at_pos != original {
            if let Some(found_pos) = self.find_nearest_occurrence(line, pos, original) {
                return self.replace_operator_at_position(line, found_pos, original, replacement);
            }
            return Err(format!(
                "Original text '{}' not found at position {}",
                original, pos
            ));
        }

        let mut result_chars = chars.clone();
        let replacement_chars: Vec<char> = replacement.chars().collect();

        for _ in 0..original_chars.len() {
            if pos < result_chars.len() {
                result_chars.remove(pos);
            }
        }

        for (i, &ch) in replacement_chars.iter().enumerate() {
            result_chars.insert(pos + i, ch);
        }

        Ok(result_chars.iter().collect())
    }

    fn replace_literal_at_position(
        &self,
        line: &str,
        pos: usize,
        original: &str,
        replacement: &str,
    ) -> Result<String, String> {
        if let Some(found_pos) = self.find_complete_word_at_position(line, pos, original) {
            self.replace_operator_at_position(line, found_pos, original, replacement)
        } else {
            Err(format!(
                "Literal '{}' not found as complete word near position {}",
                original, pos
            ))
        }
    }

    fn replace_condition_at_position(
        &self,
        line: &str,
        pos: usize,
        replacement: &str,
    ) -> Result<String, String> {
        if let Some(condition_range) = self.find_condition_range(line, pos) {
            let before = &line[..condition_range.0];
            let after = &line[condition_range.1..];
            Ok(format!("{}{}{}", before, replacement, after))
        } else {
            Err("Could not find condition boundaries".to_string())
        }
    }

    fn find_nearest_occurrence(
        &self,
        line: &str,
        around_pos: usize,
        target: &str,
    ) -> Option<usize> {
        let search_radius = 10;
        let start = around_pos.saturating_sub(search_radius);
        let end = (around_pos + search_radius).min(line.len());

        if let Some(relative_pos) = line[start..end].find(target) {
            Some(start + relative_pos)
        } else {
            None
        }
    }

    fn find_complete_word_at_position(
        &self,
        line: &str,
        around_pos: usize,
        word: &str,
    ) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();
        let word_chars: Vec<char> = word.chars().collect();

        let search_start = around_pos.saturating_sub(word.len());
        let search_end = (around_pos + word.len()).min(chars.len());

        for i in search_start..=search_end {
            if i + word_chars.len() <= chars.len() {
                let slice: String = chars[i..i + word_chars.len()].iter().collect();
                if slice == word && self.is_word_boundary(&chars, i, word_chars.len()) {
                    return Some(i);
                }
            }
        }

        None
    }

    fn is_word_boundary(&self, chars: &[char], start: usize, length: usize) -> bool {
        if start > 0 {
            let before = chars[start - 1];
            if before.is_alphanumeric() || before == '_' {
                return false;
            }
        }

        let end = start + length;
        if end < chars.len() {
            let after = chars[end];
            if after.is_alphanumeric() || after == '_' {
                return false;
            }
        }

        true
    }

    fn find_condition_range(&self, line: &str, around_pos: usize) -> Option<(usize, usize)> {
        let chars: Vec<char> = line.chars().collect();

        if let Some(if_pos) = line.find("if ") {
            let condition_start = if_pos + 3;
            if let Some(brace_pos) = line[condition_start..].find(" {") {
                let condition_end = condition_start + brace_pos;
                return Some((condition_start, condition_end));
            }
        }

        let mut paren_start = None;
        let mut paren_end = None;

        for i in (0..around_pos).rev() {
            if chars[i] == '(' {
                paren_start = Some(i + 1);
                break;
            }
        }

        for i in around_pos..chars.len() {
            if chars[i] == ')' {
                paren_end = Some(i);
                break;
            }
        }

        if let (Some(start), Some(end)) = (paren_start, paren_end) {
            Some((start, end))
        } else {
            None
        }
    }

    pub fn create_mutations_for_candidate(
        &self,
        source_code: &str,
        candidate: &MutationCandidate,
    ) -> Vec<Result<String, String>> {
        candidate
            .suggested_mutations
            .iter()
            .map(|mutation| self.apply_mutation(source_code, candidate, mutation))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operator_mutations() {
        let mutator = CodeMutator::new();
        let source_code = "fn add(a: i32, b: i32) -> i32 { a + b }";

        let mutations = mutator.create_mutations_for_candidate(
            source_code,
            &MutationCandidate {
                line: 1,
                column: 29,
                original_code: "+".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                suggested_mutations: vec!["-".to_string(), "*".to_string()],
            },
        );

        assert!(!mutations.is_empty());
        assert!(
            mutations
                .iter()
                .any(|m| m.as_ref().unwrap().contains("a - b"))
        );
        assert!(
            mutations
                .iter()
                .any(|m| m.as_ref().unwrap().contains("a * b"))
        );
    }

    #[test]
    fn test_numeric_literal_mutations() {
        let mutator = CodeMutator::new();
        let source_code = "fn test() -> i32 { 42 }";

        let mutations = mutator.create_mutations_for_candidate(
            source_code,
            &MutationCandidate {
                line: 1,
                column: 20,
                original_code: "42".to_string(),
                mutation_type: MutationType::NumericLiteral,
                suggested_mutations: vec!["0".to_string(), "1".to_string(), "-42".to_string()],
            },
        );

        assert!(!mutations.is_empty());
        assert!(mutations.iter().any(|m| m.as_ref().unwrap().contains("0")));
        assert!(mutations.iter().any(|m| m.as_ref().unwrap().contains("1")));
        assert!(
            mutations
                .iter()
                .any(|m| m.as_ref().unwrap().contains("-42"))
        );
    }

    #[test]
    fn test_logical_operator_mutations() {
        let mutator = CodeMutator::new();
        let source_code = "fn test(a: bool) -> bool { !a }";

        let mutations = mutator.create_mutations_for_candidate(
            source_code,
            &MutationCandidate {
                line: 1,
                column: 25,
                original_code: "!".to_string(),
                mutation_type: MutationType::LogicalOperator,
                suggested_mutations: vec!["".to_string()],
            },
        );

        assert!(!mutations.is_empty());
        assert!(
            mutations
                .iter()
                .any(|m| m.as_ref().unwrap().contains("a") && !m.as_ref().unwrap().contains("!"))
        );
    }

    #[test]
    fn test_mutation_application() {
        let mutator = CodeMutator::new();
        let source_code = "fn add(a: i32, b: i32) -> i32 { a + b }";

        let result = mutator.apply_mutation(
            source_code,
            &MutationCandidate {
                line: 1,
                column: 29,
                original_code: "+".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                suggested_mutations: vec!["-".to_string()],
            },
            "-",
        );

        assert!(result.is_ok());
        let mutated_code = result.unwrap();
        assert!(mutated_code.contains("a - b"));
        assert!(!mutated_code.contains("a + b"));
    }

    #[test]
    fn test_invalid_mutation_application() {
        let mutator = CodeMutator::new();
        let source_code = "fn add(a: i32, b: i32) -> i32 { a + b }";

        let result = mutator.apply_mutation(
            source_code,
            &MutationCandidate {
                line: 1,
                column: 29,
                original_code: "+".to_string(),
                mutation_type: MutationType::ArithmeticOperator,
                suggested_mutations: vec!["-".to_string()],
            },
            "/",
        );

        assert!(result.is_err());
    }
}
