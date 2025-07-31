// Example: Logical operations for mutation testing
pub fn both_true(a: bool, b: bool) -> bool {
    a && b
}

pub fn either_true(a: bool, b: bool) -> bool {
    a || b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_both_true() {
        assert!(both_true(true, true));
        assert!(!both_true(true, false));
        assert!(!both_true(false, true));
        assert!(!both_true(false, false));
    }
    #[test]
    fn test_either_true() {
        assert!(either_true(true, false));
        assert!(either_true(false, true));
        assert!(either_true(true, true));
        assert!(!either_true(false, false));
    }
}

fn main() {
    println!("Run `cargo test` to execute the tests.");
}
