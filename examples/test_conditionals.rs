// Example: Conditional logic for mutation testing
pub fn is_even(n: i32) -> bool {
    if n % 2 == 0 { true } else { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_even() {
        assert!(is_even(2));
        assert!(!is_even(3));
        assert!(is_even(0));
    }
}

fn main() {
    println!("Run `cargo test` to execute the tests.");
}
