// Example: Arithmetic operations for mutation testing
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
    #[test]
    fn test_sub() {
        assert_eq!(sub(5, 3), 2);
        assert_eq!(sub(0, 1), -1);
    }
}

fn main() {
    println!("Run `cargo test` to execute the tests.");
}
