// File: src/interpreter/native_functions/type_ops.rs
//
// Type checking and conversion functions

use crate::builtins;
use crate::interpreter::{Interpreter, Value};

pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Type conversion functions
        "parse_int" => {
            if let Some(Value::Str(s)) = arg_values.first() {
                match s.trim().parse::<i64>() {
                    Ok(n) => Value::Int(n),
                    Err(_) => Value::Error(format!("Cannot parse '{}' as integer", s)),
                }
            } else {
                Value::Error("parse_int requires a string argument".to_string())
            }
        }

        "parse_float" => {
            if let Some(Value::Str(s)) = arg_values.first() {
                match s.trim().parse::<f64>() {
                    Ok(n) => Value::Float(n),
                    Err(_) => Value::Error(format!("Cannot parse '{}' as float", s)),
                }
            } else {
                Value::Error("parse_float requires a string argument".to_string())
            }
        }

        "to_int" => {
            if let Some(val) = arg_values.first() {
                match val {
                    Value::Int(n) => Value::Int(*n),
                    Value::Float(f) => Value::Int(f.trunc() as i64),
                    Value::Str(s) => match s.trim().parse::<i64>() {
                        Ok(n) => Value::Int(n),
                        Err(_) => Value::Error(format!("Cannot convert '{}' to int", s)),
                    },
                    Value::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
                    _ => Value::Error(format!(
                        "Cannot convert {} to int",
                        match val {
                            Value::Array(_) => "array",
                            Value::Dict(_) => "dict",
                            Value::Null => "null",
                            _ => "value",
                        }
                    )),
                }
            } else {
                Value::Error("to_int() requires one argument".to_string())
            }
        }

        "to_float" => {
            if let Some(val) = arg_values.first() {
                match val {
                    Value::Int(n) => Value::Float(*n as f64),
                    Value::Float(f) => Value::Float(*f),
                    Value::Str(s) => match s.trim().parse::<f64>() {
                        Ok(n) => Value::Float(n),
                        Err(_) => Value::Error(format!("Cannot convert '{}' to float", s)),
                    },
                    Value::Bool(b) => Value::Float(if *b { 1.0 } else { 0.0 }),
                    _ => Value::Error(format!(
                        "Cannot convert {} to float",
                        match val {
                            Value::Array(_) => "array",
                            Value::Dict(_) => "dict",
                            Value::Null => "null",
                            _ => "value",
                        }
                    )),
                }
            } else {
                Value::Error("to_float() requires one argument".to_string())
            }
        }

        "to_string" => {
            if let Some(val) = arg_values.first() {
                Value::Str(Interpreter::stringify_value(val))
            } else {
                Value::Error("to_string() requires one argument".to_string())
            }
        }

        "to_bool" => {
            if let Some(val) = arg_values.first() {
                match val {
                    Value::Bool(b) => Value::Bool(*b),
                    Value::Int(n) => Value::Bool(*n != 0),
                    Value::Float(f) => Value::Bool(*f != 0.0),
                    Value::Str(s) => {
                        let s_lower = s.to_lowercase();
                        Value::Bool(!s.is_empty() && s_lower != "false" && s_lower != "0")
                    }
                    Value::Null => Value::Bool(false),
                    Value::Array(arr) => Value::Bool(!arr.is_empty()),
                    Value::Dict(dict) => Value::Bool(!dict.is_empty()),
                    _ => Value::Bool(true),
                }
            } else {
                Value::Error("to_bool() requires one argument".to_string())
            }
        }

        "bytes" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut byte_vec = Vec::new();
                for val in arr {
                    match val {
                        Value::Int(n) => {
                            if *n < 0 || *n > 255 {
                                return Some(Value::Error(format!(
                                    "bytes() requires integers in range 0-255, got {}",
                                    n
                                )));
                            }
                            byte_vec.push(*n as u8);
                        }
                        _ => {
                            return Some(Value::Error(
                                "bytes() requires an array of integers".to_string(),
                            ));
                        }
                    }
                }
                Value::Bytes(byte_vec)
            } else {
                Value::Error("bytes() requires an array argument".to_string())
            }
        }

        // Type introspection functions
        "type" => {
            if let Some(val) = arg_values.first() {
                let type_name = match val {
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::Str(_) => "string",
                    Value::Bool(_) => "bool",
                    Value::Null => "null",
                    Value::Array(_) => "array",
                    Value::Dict(_) => "dict",
                    Value::Set(_) => "set",
                    Value::Queue(_) => "queue",
                    Value::Stack(_) => "stack",
                    Value::Function(_, _, _) => "function",
                    Value::AsyncFunction(_, _, _) => "asyncfunction",
                    Value::NativeFunction(_) => "function",
                    Value::BytecodeFunction { .. } => "function",
                    Value::ArrayMarker => "arraymarker",
                    Value::Struct { .. } => "struct",
                    Value::StructDef { .. } => "structdef",
                    Value::Tagged { .. } => "tagged",
                    Value::Enum(_) => "enum",
                    Value::Bytes(_) => "bytes",
                    Value::Channel(_) => "channel",
                    Value::HttpServer { .. } => "httpserver",
                    Value::HttpResponse { .. } => "httpresponse",
                    Value::Database { .. } => "database",
                    Value::DatabasePool { .. } => "databasepool",
                    Value::Image { .. } => "image",
                    Value::ZipArchive { .. } => "ziparchive",
                    Value::TcpListener { .. } => "tcplistener",
                    Value::TcpStream { .. } => "tcpstream",
                    Value::UdpSocket { .. } => "udpsocket",
                    Value::Return(_) => "return",
                    Value::Error(_) | Value::ErrorObject { .. } => "error",
                    Value::Result { .. } => "result",
                    Value::Option { .. } => "option",
                    Value::GeneratorDef(_, _) => "generatordef",
                    Value::Generator { .. } => "generator",
                    Value::Iterator { .. } => "iterator",
                    Value::Promise { .. } => "promise",
                };
                Value::Str(type_name.to_string())
            } else {
                Value::Error("type() requires one argument".to_string())
            }
        }

        "is_int" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Int(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_float" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Float(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_string" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Str(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_array" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Array(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_dict" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Dict(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_bool" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Bool(_)))
            } else {
                Value::Bool(false)
            }
        }

        "is_null" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Null))
            } else {
                Value::Bool(false)
            }
        }

        "is_function" => {
            if let Some(val) = arg_values.first() {
                Value::Bool(matches!(val, Value::Function(_, _, _) | Value::NativeFunction(_)))
            } else {
                Value::Bool(false)
            }
        }

        "format" => {
            if arg_values.is_empty() {
                return Some(Value::Error(
                    "format() requires at least 1 argument (template)".to_string(),
                ));
            }

            let template = match &arg_values[0] {
                Value::Str(s) => s,
                _ => {
                    return Some(Value::Error(
                        "format() first argument must be a string".to_string(),
                    ))
                }
            };

            let format_args = &arg_values[1..];
            match builtins::format_string(template, format_args) {
                Ok(s) => Value::Str(s),
                Err(e) => Value::Error(e),
            }
        }

        _ => return None,
    };

    Some(result)
}
