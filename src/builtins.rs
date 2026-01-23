// File: src/builtins.rs
//
// Built-in native functions for the Ruff standard library.
// These are implemented in Rust for performance and provide
// core functionality for math, strings, arrays, I/O operations, and JSON.

use crate::interpreter::Value;
use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

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

/// Random number functions

/// Generate a random float between 0.0 and 1.0
pub fn random() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>()
}

/// Generate a random integer between min and max (inclusive)
pub fn random_int(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let min_i = min as i64;
    let max_i = max as i64;
    rng.gen_range(min_i..=max_i) as f64
}

/// Select a random element from an array
pub fn random_choice(arr: &[Value]) -> Value {
    if arr.is_empty() {
        return Value::Number(0.0);
    }
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..arr.len());
    arr[idx].clone()
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
        serde_json::Value::Null => Value::Null, // null -> Null
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
        Value::Null => Ok(serde_json::Value::Null),
        Value::Number(n) => {
            // Check if it's 0 and might represent null, but we'll always use number
            // Users can explicitly use 0 if they want null in their JSON
            Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
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

/// Date/Time functions

/// Get current Unix timestamp (seconds since epoch)
pub fn now() -> f64 {
    Utc::now().timestamp() as f64
}

/// Format a Unix timestamp to a date string
/// Supports basic format: "YYYY-MM-DD HH:mm:ss"
pub fn format_date(timestamp: f64, format_str: &str) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp as i64, 0).unwrap();

    // Simple format string replacement
    let result = format_str
        .replace("YYYY", &dt.format("%Y").to_string())
        .replace("MM", &dt.format("%m").to_string())
        .replace("DD", &dt.format("%d").to_string())
        .replace("HH", &dt.format("%H").to_string())
        .replace("mm", &dt.format("%M").to_string())
        .replace("ss", &dt.format("%S").to_string());

    result
}

/// Parse a date string to Unix timestamp
/// Supports format: "YYYY-MM-DD"
pub fn parse_date(date_str: &str, _format: &str) -> f64 {
    // Simple parser for "YYYY-MM-DD" format
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return 0.0;
    }

    let year = parts[0].parse::<i32>().unwrap_or(1970);
    let month = parts[1].parse::<u32>().unwrap_or(1);
    let day = parts[2].parse::<u32>().unwrap_or(1);

    if let Some(dt) = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).single() {
        dt.timestamp() as f64
    } else {
        0.0
    }
}

/// System operation functions

/// Get environment variable value
pub fn get_env(var_name: &str) -> String {
    env::var(var_name).unwrap_or_default()
}

/// Get command-line arguments
pub fn get_args() -> Vec<String> {
    env::args().skip(1).collect() // Skip the program name
}

/// Sleep for specified milliseconds
pub fn sleep_ms(ms: f64) {
    thread::sleep(Duration::from_millis(ms as u64));
}

/// Execute a shell command and return output
pub fn execute_command(command: &str) -> String {
    if cfg!(target_os = "windows") {
        match Command::new("cmd").args(["/C", command]).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Error: {}", e),
        }
    } else {
        match Command::new("sh").arg("-c").arg(command).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Error: {}", e),
        }
    }
}

/// Path operation functions

/// Join path components
pub fn join_path(parts: &[String]) -> String {
    let path: PathBuf = parts.iter().collect();
    path.to_string_lossy().to_string()
}

/// Get directory name from path
pub fn dirname(path_str: &str) -> String {
    let path = Path::new(path_str);
    path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| String::from("/"))
}

/// Get base filename from path
pub fn basename(path_str: &str) -> String {
    let path = Path::new(path_str);
    path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default()
}

/// Check if path exists
pub fn path_exists(path_str: &str) -> bool {
    Path::new(path_str).exists()
}

/// Regular expression functions

/// Check if string matches regex pattern
pub fn regex_match(text: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => false, // Invalid regex returns false
    }
}

/// Find all matches of regex pattern in text
pub fn regex_find_all(text: &str, pattern: &str) -> Vec<String> {
    match Regex::new(pattern) {
        Ok(re) => re.find_iter(text).map(|m| m.as_str().to_string()).collect(),
        Err(_) => vec![], // Invalid regex returns empty array
    }
}

/// Replace all matches of regex pattern with replacement string
pub fn regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    match Regex::new(pattern) {
        Ok(re) => re.replace_all(text, replacement).to_string(),
        Err(_) => text.to_string(), // Invalid regex returns original text
    }
}

/// Split string by regex pattern
pub fn regex_split(text: &str, pattern: &str) -> Vec<String> {
    match Regex::new(pattern) {
        Ok(re) => re.split(text).map(|s| s.to_string()).collect(),
        Err(_) => vec![text.to_string()], // Invalid regex returns original text as single element
    }
}

/// Array functions

/// HTTP Client Functions

/// Make an HTTP GET request
/// Returns a dictionary with status, body, and headers
pub fn http_get(url: &str) -> Result<HashMap<String, Value>, String> {
    match reqwest::blocking::get(url) {
        Ok(response) => {
            let status = response.status().as_u16() as f64;
            let body = response.text().unwrap_or_default();

            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::Number(status));
            result.insert("body".to_string(), Value::Str(body));

            Ok(result)
        }
        Err(e) => Err(format!("HTTP GET failed: {}", e)),
    }
}

/// Make an HTTP POST request with JSON body
/// body_json should be a stringified JSON
pub fn http_post(url: &str, body_json: &str) -> Result<HashMap<String, Value>, String> {
    let client = reqwest::blocking::Client::new();

    match client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body_json.to_string())
        .send()
    {
        Ok(response) => {
            let status = response.status().as_u16() as f64;
            let body = response.text().unwrap_or_default();

            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::Number(status));
            result.insert("body".to_string(), Value::Str(body));

            Ok(result)
        }
        Err(e) => Err(format!("HTTP POST failed: {}", e)),
    }
}

/// Make an HTTP PUT request with JSON body
pub fn http_put(url: &str, body_json: &str) -> Result<HashMap<String, Value>, String> {
    let client = reqwest::blocking::Client::new();

    match client
        .put(url)
        .header("Content-Type", "application/json")
        .body(body_json.to_string())
        .send()
    {
        Ok(response) => {
            let status = response.status().as_u16() as f64;
            let body = response.text().unwrap_or_default();

            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::Number(status));
            result.insert("body".to_string(), Value::Str(body));

            Ok(result)
        }
        Err(e) => Err(format!("HTTP PUT failed: {}", e)),
    }
}

/// Make an HTTP DELETE request
pub fn http_delete(url: &str) -> Result<HashMap<String, Value>, String> {
    let client = reqwest::blocking::Client::new();

    match client.delete(url).send() {
        Ok(response) => {
            let status = response.status().as_u16() as f64;
            let body = response.text().unwrap_or_default();

            let mut result = HashMap::new();
            result.insert("status".to_string(), Value::Number(status));
            result.insert("body".to_string(), Value::Str(body));

            Ok(result)
        }
        Err(e) => Err(format!("HTTP DELETE failed: {}", e)),
    }
}

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
