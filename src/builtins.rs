// File: src/builtins.rs
//
// Built-in native functions for the Ruff standard library.
// These are implemented in Rust for performance and provide
// core functionality for math, strings, arrays, and I/O operations.

use crate::interpreter::Value;
use std::collections::HashMap;

/// Returns a HashMap of all built-in functions
#[allow(dead_code)]
pub fn get_builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();

    // Math constants
    builtins.insert("PI".to_string(), Value::Number(std::f64::consts::PI));
    builtins.insert("E".to_string(), Value::Number(std::f64::consts::E));

    builtins
}

/// Math functions

pub fn abs(x: f64) -> f64 {
    x.abs()
}

pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

pub fn pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

pub fn floor(x: f64) -> f64 {
    x.floor()
}

pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

pub fn round(x: f64) -> f64 {
    x.round()
}

pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn tan(x: f64) -> f64 {
    x.tan()
}

/// String functions

pub fn str_len(s: &str) -> f64 {
    s.len() as f64
}

pub fn substring(s: &str, start: f64, end: f64) -> String {
    let start_idx = start as usize;
    let end_idx = end as usize;
    let chars: Vec<char> = s.chars().collect();

    if start_idx >= chars.len() {
        return String::new();
    }

    let end_idx = end_idx.min(chars.len());
    chars[start_idx..end_idx].iter().collect()
}

pub fn to_upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

pub fn contains(s: &str, substr: &str) -> bool {
    s.contains(substr)
}

pub fn replace(s: &str, old: &str, new: &str) -> String {
    s.replace(old, new)
}

pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

pub fn index_of(s: &str, substr: &str) -> f64 {
    match s.find(substr) {
        Some(idx) => idx as f64,
        None => -1.0,
    }
}

pub fn repeat(s: &str, count: f64) -> String {
    let count = count as usize;
    s.repeat(count)
}

pub fn split(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|s| s.to_string()).collect()
}

pub fn join(arr: &[String], separator: &str) -> String {
    arr.join(separator)
}

/// Array functions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_functions() {
        assert_eq!(abs(-5.0), 5.0);
        assert_eq!(abs(3.0), 3.0);
        assert_eq!(floor(3.7), 3.0);
        assert_eq!(ceil(3.2), 4.0);
        assert_eq!(round(3.5), 4.0);
        assert_eq!(min(5.0, 3.0), 3.0);
        assert_eq!(max(5.0, 3.0), 5.0);
    }

    #[test]
    fn test_string_functions() {
        assert_eq!(str_len("hello"), 5.0);
        assert_eq!(substring("hello", 1.0, 4.0), "ell");
        assert_eq!(to_upper("hello"), "HELLO");
        assert_eq!(to_lower("HELLO"), "hello");
        assert_eq!(trim("  hello  "), "hello");
        assert!(contains("hello world", "world"));
        assert!(!contains("hello", "xyz"));
        assert_eq!(replace("hello world", "world", "rust"), "hello rust");
        
        // New string functions
        assert!(starts_with("hello world", "hello"));
        assert!(!starts_with("hello world", "world"));
        assert!(ends_with("test.ruff", ".ruff"));
        assert!(!ends_with("test.ruff", ".py"));
        assert_eq!(index_of("hello", "ll"), 2.0);
        assert_eq!(index_of("hello", "xyz"), -1.0);
        assert_eq!(repeat("ha", 3.0), "hahaha");
        assert_eq!(split("a,b,c", ","), vec!["a", "b", "c"]);
        assert_eq!(join(&vec!["a".to_string(), "b".to_string(), "c".to_string()], ","), "a,b,c");
    }
}
