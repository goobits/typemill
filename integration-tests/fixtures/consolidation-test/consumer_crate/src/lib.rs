/// Consumer crate - uses source_crate
use source_crate::say_hello;
use source_crate::get_version;

pub fn greet() -> String {
    format!("{} (version: {})", say_hello(), get_version())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let greeting = greet();
        assert!(greeting.contains("Hello"));
    }
}
