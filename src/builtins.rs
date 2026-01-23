// File: src/builtins.rs
//
// Built-in native functions for the Ruff standard library.
// These are implemented in Rust for performance and provide
// core functionality for math, strings, arrays, I/O operations, and JSON.

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

/// JSON functions

/// Parse a JSON string into a Ruff value
pub fn parse_json(json_str: &str) -> Result<Value, String> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(json_value) => Ok(json_to_ruff_value(json_value)),
        Err(e) => Err(format!("JSON parse error: {}", e)),
    }
}

/// Convert a Ruff value to a JSON string
pub fn to_json(value: &Value) -> Result<String, String> {
    let json_value = ruff_value_to_json(value)?;
    match serde_json::to_string(&json_value) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("JSON serialization error: {}", e)),
    }
}

/// Convert serde_json::Value to Ruff Value
fn json_to_ruff_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Number(0.0), // null -> 0 (Ruff convention)
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Number(0.0)
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            let ruff_arr: Vec<Value> = arr.into_iter().map(json_to_ruff_value).collect();
            Value::Array(ruff_arr)
        }
        serde_json::Value::Object(obj) => {
            let mut ruff_dict = HashMap::new();
            for (key, val) in obj {
                ruff_dict.insert(key, json_to_ruff_value(val));
            }
            Value::Dict(ruff_dict)
        }
    }
}

/// Convert Ruff Value to serde_json::Value
fn ruff_value_to_json(value: &Value) -> Result<serde_json::Value, String> {
    match value {
        Value::Number(n) => {
            // Check if it's 0 and might represent null, but we'll always use number
            // Users can explicitly use 0 if they want null in their JSON
            Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(*n)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ))
        }
        Value::Str(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Array(arr) => {
            let mut json_arr = Vec::new();
            for item in arr {
                json_arr.push(ruff_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        }
        Value::Dict(dict) => {
            let mut json_obj = serde_json::Map::new();
            for (key, val) in dict {
                json_obj.insert(key.clone(), ruff_value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        }
        _ => Err(format!("Cannot convert {:?} to JSON", value)),
    }
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
