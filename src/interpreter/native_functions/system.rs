// File: src/interpreter/native_functions/system.rs
//
// System-related native functions (env vars, time, etc.)

use crate::builtins;
use crate::interpreter::{DictMap, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Random functions
        "random" => Value::Float(builtins::random()),

        "random_int" => {
            if let (Some(min_val), Some(max_val)) = (arg_values.first(), arg_values.get(1)) {
                let min = match min_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => {
                        return Some(Value::Error(
                            "random_int requires number arguments".to_string(),
                        ))
                    }
                };
                let max = match max_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => {
                        return Some(Value::Error(
                            "random_int requires number arguments".to_string(),
                        ))
                    }
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
                Value::Str(Arc::new(builtins::format_duration(ms)))
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
                    _ => {
                        return Some(Value::Error("elapsed requires number arguments".to_string()))
                    }
                };
                let end = match end_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => {
                        return Some(Value::Error("elapsed requires number arguments".to_string()))
                    }
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
                Value::Str(Arc::new(builtins::format_date(timestamp, format.as_ref())))
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

        // System operation functions
        "env" => {
            if let Some(Value::Str(var_name)) = arg_values.first() {
                Value::Str(Arc::new(builtins::get_env(var_name.as_ref())))
            } else {
                Value::Error("env requires a string argument (variable name)".to_string())
            }
        }

        "env_or" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(var_name)), Some(Value::Str(default_value))) => {
                Value::Str(Arc::new(builtins::env_or(var_name.as_ref(), default_value.as_ref())))
            }
            _ => Value::Error(
                "env_or requires two string arguments (variable name, default value)".to_string(),
            ),
        },

        "env_int" => {
            if let Some(Value::Str(var_name)) = arg_values.first() {
                match builtins::env_int(var_name.as_ref()) {
                    Ok(value) => Value::Int(value),
                    Err(message) => {
                        Value::ErrorObject { message, stack: Vec::new(), line: None, cause: None }
                    }
                }
            } else {
                Value::Error("env_int requires a string argument (variable name)".to_string())
            }
        }

        "env_float" => {
            if let Some(Value::Str(var_name)) = arg_values.first() {
                match builtins::env_float(var_name.as_ref()) {
                    Ok(value) => Value::Float(value),
                    Err(message) => {
                        Value::ErrorObject { message, stack: Vec::new(), line: None, cause: None }
                    }
                }
            } else {
                Value::Error("env_float requires a string argument (variable name)".to_string())
            }
        }

        "env_bool" => {
            if let Some(Value::Str(var_name)) = arg_values.first() {
                match builtins::env_bool(var_name.as_ref()) {
                    Ok(value) => Value::Bool(value),
                    Err(message) => {
                        Value::ErrorObject { message, stack: Vec::new(), line: None, cause: None }
                    }
                }
            } else {
                Value::Error("env_bool requires a string argument (variable name)".to_string())
            }
        }

        "env_required" => {
            if let Some(Value::Str(var_name)) = arg_values.first() {
                match builtins::env_required(var_name.as_ref()) {
                    Ok(value) => Value::Str(Arc::new(value)),
                    Err(message) => {
                        Value::ErrorObject { message, stack: Vec::new(), line: None, cause: None }
                    }
                }
            } else {
                Value::Error("env_required requires a string argument (variable name)".to_string())
            }
        }

        "env_set" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Str(var_name)), Some(Value::Str(value))) => {
                builtins::env_set(var_name.as_ref(), value.as_ref());
                Value::Null
            }
            _ => {
                Value::Error("env_set requires two string arguments (variable name, value)".to_string())
            }
        },

        "env_list" => {
            let env_vars = builtins::env_list();
            let mut dict = DictMap::default();
            for (key, value) in env_vars {
                dict.insert(Arc::<str>::from(key), Value::Str(Arc::new(value)));
            }
            Value::Dict(Arc::new(dict))
        }

        "args" => {
            let args = builtins::get_args();
            let values: Vec<Value> = args.into_iter().map(|value| Value::Str(Arc::new(value))).collect();
            Value::Array(Arc::new(values))
        }

        "arg_parser" => {
            let mut fields: HashMap<String, Value> = HashMap::new();
            fields.insert("_args".to_string(), Value::Array(Arc::new(Vec::new())));
            fields.insert("_app_name".to_string(), Value::Str(Arc::new(String::new())));
            fields.insert("_description".to_string(), Value::Str(Arc::new(String::new())));
            Value::Struct { name: "ArgParser".to_string(), fields }
        }

        _ => return None,
    };

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::interpreter::Value;
    use std::sync::Arc;

    fn string_value(value: &str) -> Value {
        Value::Str(Arc::new(value.to_string()))
    }

    #[test]
    fn test_env_set_and_env_round_trip() {
        let key = "RUFF_NATIVE_SYSTEM_ENV_TEST";
        let value = "hardening_round_trip";

        let set_result = handle("env_set", &[string_value(key), string_value(value)]).unwrap();
        assert!(matches!(set_result, Value::Null));

        let get_result = handle("env", &[string_value(key)]).unwrap();
        match get_result {
            Value::Str(actual) => assert_eq!(actual.as_ref(), value),
            other => panic!("Expected Value::Str from env(), got {:?}", other),
        }
    }

    #[test]
    fn test_env_int_missing_variable_returns_error_object() {
        let missing_key = "RUFF_NATIVE_SYSTEM_MISSING_INT_TEST";
        std::env::remove_var(missing_key);

        let result = handle("env_int", &[string_value(missing_key)]).unwrap();
        match result {
            Value::ErrorObject { message, .. } => {
                assert!(message.contains("not found"));
            }
            other => panic!("Expected Value::ErrorObject from env_int(), got {:?}", other),
        }
    }

    #[test]
    fn test_args_returns_array() {
        let result = handle("args", &[]).unwrap();
        assert!(matches!(result, Value::Array(_)));
    }

    #[test]
    fn test_arg_parser_returns_struct_shape() {
        let result = handle("arg_parser", &[]).unwrap();

        match result {
            Value::Struct { name, fields } => {
                assert_eq!(name, "ArgParser");
                assert!(fields.contains_key("_args"));
                assert!(fields.contains_key("_app_name"));
                assert!(fields.contains_key("_description"));
            }
            other => panic!("Expected Value::Struct from arg_parser(), got {:?}", other),
        }
    }
}
