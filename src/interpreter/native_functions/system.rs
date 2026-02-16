// File: src/interpreter/native_functions/system.rs
//
// System-related native functions (env vars, time, etc.)

use crate::builtins;
use crate::interpreter::{DictMap, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::Stdio;
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
            _ => Value::Error(
                "env_set requires two string arguments (variable name, value)".to_string(),
            ),
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
            let values: Vec<Value> =
                args.into_iter().map(|value| Value::Str(Arc::new(value))).collect();
            Value::Array(Arc::new(values))
        }

        "arg_parser" => {
            let mut fields: HashMap<String, Value> = HashMap::new();
            fields.insert("_args".to_string(), Value::Array(Arc::new(Vec::new())));
            fields.insert("_app_name".to_string(), Value::Str(Arc::new(String::new())));
            fields.insert("_description".to_string(), Value::Str(Arc::new(String::new())));
            Value::Struct { name: "ArgParser".to_string(), fields }
        }

        "input" => {
            if arg_values.len() > 1 {
                return Some(Value::Error("input() expects 0-1 arguments".to_string()));
            }

            let prompt = match arg_values.first() {
                Some(Value::Str(value)) => value.to_string(),
                Some(_) => {
                    return Some(Value::Error(
                        "input() requires a string prompt when provided".to_string(),
                    ))
                }
                None => String::new(),
            };

            print!("{}", prompt);
            let _ = std::io::stdout().flush();

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => Value::Str(Arc::new(input.trim_end().to_string())),
                Err(_) => Value::Str(Arc::new(String::new())),
            }
        }

        "exit" => {
            if arg_values.len() > 1 {
                return Some(Value::Error("exit() expects 0-1 arguments".to_string()));
            }

            let exit_code = match arg_values.first() {
                Some(Value::Int(code)) => *code as i32,
                Some(Value::Float(code)) => *code as i32,
                Some(_) => {
                    return Some(Value::Error(
                        "exit() requires a numeric exit code when provided".to_string(),
                    ))
                }
                None => 0,
            };

            std::process::exit(exit_code);
        }

        "sleep" => {
            if arg_values.len() != 1 {
                return Some(Value::Error("sleep() expects 1 argument".to_string()));
            }

            let millis = match arg_values.first() {
                Some(Value::Int(value)) => *value as f64,
                Some(Value::Float(value)) => *value,
                _ => return Some(Value::Error("sleep() requires a number argument".to_string())),
            };

            if millis < 0.0 {
                return Some(Value::Error(
                    "sleep() requires non-negative milliseconds".to_string(),
                ));
            }

            builtins::sleep_ms(millis);
            Value::Int(0)
        }

        "execute" => {
            if arg_values.len() != 1 {
                return Some(Value::Error("execute() expects 1 argument".to_string()));
            }

            match arg_values.first() {
                Some(Value::Str(command)) => {
                    Value::Str(Arc::new(builtins::execute_command(command.as_ref())))
                }
                _ => Value::Error("execute() requires a string command".to_string()),
            }
        }

        "spawn_process" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "spawn_process requires an array of command arguments".to_string(),
                ));
            }

            if let Some(Value::Array(args)) = arg_values.first() {
                if args.is_empty() {
                    return Some(Value::Error(
                        "spawn_process requires a non-empty array of command arguments".to_string(),
                    ));
                }

                let mut cmd_parts: Vec<String> = Vec::new();
                for arg in args.iter() {
                    if let Value::Str(s) = arg {
                        cmd_parts.push(s.as_ref().clone());
                    } else {
                        return Some(Value::Error(
                            "spawn_process requires an array of strings".to_string(),
                        ));
                    }
                }

                let program = &cmd_parts[0];
                let args_slice = &cmd_parts[1..];

                match std::process::Command::new(program)
                    .args(args_slice)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(mut child) => match child.wait() {
                        Ok(status) => {
                            let stdout = if let Some(mut out) = child.stdout.take() {
                                let mut buffer = String::new();
                                let _ = out.read_to_string(&mut buffer);
                                buffer
                            } else {
                                String::new()
                            };

                            let stderr = if let Some(mut err) = child.stderr.take() {
                                let mut buffer = String::new();
                                let _ = err.read_to_string(&mut buffer);
                                buffer
                            } else {
                                String::new()
                            };

                            let mut fields = HashMap::new();
                            fields.insert(
                                "exitcode".to_string(),
                                Value::Int(status.code().unwrap_or(-1) as i64),
                            );
                            fields.insert("stdout".to_string(), Value::Str(Arc::new(stdout)));
                            fields.insert("stderr".to_string(), Value::Str(Arc::new(stderr)));
                            fields.insert("success".to_string(), Value::Bool(status.success()));

                            Value::Struct { name: "ProcessResult".to_string(), fields }
                        }
                        Err(e) => Value::ErrorObject {
                            message: format!("Failed to wait for process: {}", e),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    },
                    Err(e) => Value::ErrorObject {
                        message: format!("Failed to spawn process '{}': {}", program, e),
                        stack: Vec::new(),
                        line: None,
                        cause: None,
                    },
                }
            } else {
                Value::Error("spawn_process requires an array of command arguments".to_string())
            }
        }

        "pipe_commands" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "pipe_commands requires an array of command arrays".to_string(),
                ));
            }

            if let Some(Value::Array(commands)) = arg_values.first() {
                if commands.is_empty() {
                    return Some(Value::Error(
                        "pipe_commands requires a non-empty array of commands".to_string(),
                    ));
                }

                let mut parsed_commands: Vec<Vec<String>> = Vec::new();
                for cmd in commands.iter() {
                    if let Value::Array(args) = cmd {
                        let mut cmd_parts: Vec<String> = Vec::new();
                        for arg in args.iter() {
                            if let Value::Str(s) = arg {
                                cmd_parts.push(s.as_ref().clone());
                            } else {
                                return Some(Value::Error(
                                    "Each command must be an array of strings".to_string(),
                                ));
                            }
                        }

                        if cmd_parts.is_empty() {
                            return Some(Value::Error(
                                "Each command array must not be empty".to_string(),
                            ));
                        }

                        parsed_commands.push(cmd_parts);
                    } else {
                        return Some(Value::Error(
                            "pipe_commands requires an array of command arrays".to_string(),
                        ));
                    }
                }

                let mut previous_output: Option<Vec<u8>> = None;

                for (index, cmd_parts) in parsed_commands.iter().enumerate() {
                    let program = &cmd_parts[0];
                    let args = &cmd_parts[1..];

                    let mut command = std::process::Command::new(program);
                    command.args(args);

                    if previous_output.is_some() {
                        command.stdin(Stdio::piped());
                    }

                    if index == parsed_commands.len() - 1 {
                        command.stdout(Stdio::piped());
                    } else {
                        command.stdout(Stdio::piped());
                    }

                    command.stderr(Stdio::piped());

                    match command.spawn() {
                        Ok(mut child) => {
                            if let Some(input) = previous_output.take() {
                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(&input);
                                }
                            }

                            match child.wait() {
                                Ok(status) => {
                                    if !status.success() {
                                        let stderr = if let Some(mut err) = child.stderr.take() {
                                            let mut buffer = String::new();
                                            let _ = err.read_to_string(&mut buffer);
                                            buffer
                                        } else {
                                            String::new()
                                        };

                                        return Some(Value::ErrorObject {
                                            message: format!(
                                                "Command '{}' failed with exit code {}: {}",
                                                cmd_parts.join(" "),
                                                status.code().unwrap_or(-1),
                                                stderr
                                            ),
                                            stack: Vec::new(),
                                            line: None,
                                            cause: None,
                                        });
                                    }

                                    if let Some(mut stdout) = child.stdout.take() {
                                        let mut buffer = Vec::new();
                                        if let Err(e) = stdout.read_to_end(&mut buffer) {
                                            return Some(Value::ErrorObject {
                                                message: format!(
                                                    "Failed to read output from '{}': {}",
                                                    program, e
                                                ),
                                                stack: Vec::new(),
                                                line: None,
                                                cause: None,
                                            });
                                        }
                                        previous_output = Some(buffer);
                                    }
                                }
                                Err(e) => {
                                    return Some(Value::ErrorObject {
                                        message: format!(
                                            "Failed to wait for process '{}': {}",
                                            program, e
                                        ),
                                        stack: Vec::new(),
                                        line: None,
                                        cause: None,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            return Some(Value::ErrorObject {
                                message: format!("Failed to spawn process '{}': {}", program, e),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            });
                        }
                    }
                }

                if let Some(output) = previous_output {
                    match String::from_utf8(output) {
                        Ok(result) => Value::Str(Arc::new(result)),
                        Err(e) => Value::ErrorObject {
                            message: format!("Failed to decode output as UTF-8: {}", e),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                } else {
                    Value::Str(Arc::new(String::new()))
                }
            } else {
                Value::Error("pipe_commands requires an array of command arrays".to_string())
            }
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

    #[test]
    fn test_spawn_process_rejects_invalid_argument_shape() {
        let non_array_result = handle("spawn_process", &[Value::Int(1)]).unwrap();
        assert!(matches!(
            non_array_result,
            Value::Error(message) if message == "spawn_process requires an array of command arguments"
        ));

        let empty_args_result = handle("spawn_process", &[Value::Array(Arc::new(vec![]))]).unwrap();
        assert!(matches!(
            empty_args_result,
            Value::Error(message) if message == "spawn_process requires a non-empty array of command arguments"
        ));

        let invalid_elem_result =
            handle("spawn_process", &[Value::Array(Arc::new(vec![Value::Int(1)]))]).unwrap();
        assert!(matches!(
            invalid_elem_result,
            Value::Error(message) if message == "spawn_process requires an array of strings"
        ));
    }

    #[test]
    fn test_spawn_process_returns_process_result_struct() {
        let exe_path = std::env::current_exe().expect("current exe path should be available");
        let args = vec![
            Value::Str(Arc::new(exe_path.to_string_lossy().to_string())),
            Value::Str(Arc::new("--help".to_string())),
        ];

        let result = handle("spawn_process", &[Value::Array(Arc::new(args))]).unwrap();
        match result {
            Value::Struct { name, fields } => {
                assert_eq!(name, "ProcessResult");
                assert!(matches!(fields.get("exitcode"), Some(Value::Int(_))));
                assert!(matches!(fields.get("stdout"), Some(Value::Str(_))));
                assert!(matches!(fields.get("stderr"), Some(Value::Str(_))));
                assert!(matches!(fields.get("success"), Some(Value::Bool(true))));
            }
            other => panic!("Expected ProcessResult struct from spawn_process, got {:?}", other),
        }
    }

    #[test]
    fn test_pipe_commands_rejects_invalid_argument_shape() {
        let non_array_result = handle("pipe_commands", &[Value::Int(1)]).unwrap();
        assert!(matches!(
            non_array_result,
            Value::Error(message) if message == "pipe_commands requires an array of command arrays"
        ));

        let empty_commands_result =
            handle("pipe_commands", &[Value::Array(Arc::new(vec![]))]).unwrap();
        assert!(matches!(
            empty_commands_result,
            Value::Error(message) if message == "pipe_commands requires a non-empty array of commands"
        ));

        let invalid_command_shape = handle(
            "pipe_commands",
            &[Value::Array(Arc::new(vec![Value::Str(Arc::new("echo".to_string()))]))],
        )
        .unwrap();
        assert!(matches!(
            invalid_command_shape,
            Value::Error(message) if message == "pipe_commands requires an array of command arrays"
        ));

        let invalid_command_arg = handle(
            "pipe_commands",
            &[Value::Array(Arc::new(vec![Value::Array(Arc::new(vec![Value::Int(1)]))]))],
        )
        .unwrap();
        assert!(matches!(
            invalid_command_arg,
            Value::Error(message) if message == "Each command must be an array of strings"
        ));
    }

    #[test]
    fn test_pipe_commands_returns_string_output_for_single_command_pipeline() {
        let exe_path = std::env::current_exe().expect("current exe path should be available");
        let command = Value::Array(Arc::new(vec![
            Value::Str(Arc::new(exe_path.to_string_lossy().to_string())),
            Value::Str(Arc::new("--help".to_string())),
        ]));

        let result = handle("pipe_commands", &[Value::Array(Arc::new(vec![command]))]).unwrap();
        assert!(matches!(result, Value::Str(_)));
    }

    #[test]
    fn test_process_api_strict_arity_rejects_extra_arguments() {
        let spawn_extra = handle(
            "spawn_process",
            &[
                Value::Array(Arc::new(vec![Value::Str(Arc::new("echo".to_string()))])),
                Value::Int(1),
            ],
        )
        .unwrap();
        assert!(matches!(
            spawn_extra,
            Value::Error(message) if message == "spawn_process requires an array of command arguments"
        ));

        let pipe_extra = handle(
            "pipe_commands",
            &[
                Value::Array(Arc::new(vec![Value::Array(Arc::new(vec![Value::Str(Arc::new(
                    "echo".to_string(),
                ))]))])),
                Value::Int(1),
            ],
        )
        .unwrap();
        assert!(matches!(
            pipe_extra,
            Value::Error(message) if message == "pipe_commands requires an array of command arrays"
        ));
    }
}
