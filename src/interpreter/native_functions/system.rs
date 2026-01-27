// File: src/interpreter/native_functions/system.rs
//
// System-related native functions (env vars, time, etc.)

use crate::builtins;
use crate::interpreter::Value;

pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Random functions
        "random" => Value::Float(builtins::random()),

        "random_int" => {
            if let (Some(min_val), Some(max_val)) = (arg_values.first(), arg_values.get(1)) {
                let min = match min_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Error("random_int requires number arguments".to_string())),
                };
                let max = match max_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Error("random_int requires number arguments".to_string())),
                };
                Value::Int(builtins::random_int(min, max) as i64)
            } else {
                Value::Error("random_int requires two number arguments: min and max".to_string())
            }
        }

        "random_choice" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                builtins::random_choice(arr)
            } else {
                Value::Error("random_choice requires an array argument".to_string())
            }
        }

        // Random seed control (for deterministic testing)
        "set_random_seed" => {
            if let Some(Value::Int(seed)) = arg_values.first() {
                builtins::set_random_seed(*seed as u64);
                Value::Null
            } else if let Some(Value::Float(seed)) = arg_values.first() {
                builtins::set_random_seed(*seed as u64);
                Value::Null
            } else {
                Value::Error("set_random_seed requires a number argument".to_string())
            }
        }

        "clear_random_seed" => {
            builtins::clear_random_seed();
            Value::Null
        }

        // Date/Time functions
        "now" => Value::Float(builtins::now()),

        "current_timestamp" => Value::Int(builtins::current_timestamp()),

        "performance_now" => Value::Float(builtins::performance_now()),

        "time_us" => Value::Float(builtins::time_us()),

        "time_ns" => Value::Float(builtins::time_ns()),

        "format_duration" => {
            if let Some(ms_val) = arg_values.first() {
                let ms = match ms_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => {
                        return Some(Value::Error(
                            "format_duration requires a number argument".to_string(),
                        ))
                    }
                };
                Value::Str(builtins::format_duration(ms))
            } else {
                Value::Error(
                    "format_duration requires a number argument (milliseconds)".to_string(),
                )
            }
        }

        "elapsed" => {
            if let (Some(start_val), Some(end_val)) = (arg_values.first(), arg_values.get(1)) {
                let start = match start_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Error("elapsed requires number arguments".to_string())),
                };
                let end = match end_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Error("elapsed requires number arguments".to_string())),
                };
                Value::Float(builtins::elapsed(start, end))
            } else {
                Value::Error("elapsed requires two number arguments: start and end".to_string())
            }
        }

        "format_date" => {
            if let (Some(ts_val), Some(Value::Str(format))) =
                (arg_values.first(), arg_values.get(1))
            {
                let timestamp = match ts_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => {
                        return Some(Value::Error(
                            "format_date requires a number timestamp".to_string(),
                        ))
                    }
                };
                Value::Str(builtins::format_date(timestamp, format))
            } else {
                Value::Error(
                    "format_date requires timestamp (number) and format (string)".to_string(),
                )
            }
        }

        "parse_date" => {
            if let (Some(Value::Str(date_str)), Some(Value::Str(format))) =
                (arg_values.first(), arg_values.get(1))
            {
                Value::Float(builtins::parse_date(date_str, format))
            } else {
                Value::Error("parse_date requires date string and format string".to_string())
            }
        }

        _ => return None,
    };

    Some(result)
}
