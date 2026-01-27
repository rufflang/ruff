// File: src/interpreter/native_functions/filesystem.rs
//
// Filesystem operation native functions

use crate::interpreter::{Interpreter, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn handle(_interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "read_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_to_string(path) {
                    Ok(content) => Value::Str(content),
                    Err(e) => Value::Error(format!("Cannot read file '{}': {}", path, e)),
                }
            } else {
                Value::Error("read_file requires a string path argument".to_string())
            }
        }

        "write_file" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "write_file requires two arguments: path and content".to_string(),
                ));
            }
            if let (Some(Value::Str(path)), Some(Value::Str(content))) =
                (arg_values.first(), arg_values.get(1))
            {
                match std::fs::write(path, content) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot write file '{}': {}", path, e)),
                }
            } else {
                Value::Error("write_file requires string arguments".to_string())
            }
        }

        "read_binary_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read(path) {
                    Ok(bytes) => Value::Bytes(bytes),
                    Err(e) => Value::Error(format!("Cannot read binary file '{}': {}", path, e)),
                }
            } else {
                Value::Error("read_binary_file requires a string path argument".to_string())
            }
        }

        "write_binary_file" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "write_binary_file requires two arguments: path and bytes".to_string(),
                ));
            }
            if let (Some(Value::Str(path)), Some(Value::Bytes(bytes))) =
                (arg_values.first(), arg_values.get(1))
            {
                match std::fs::write(path, bytes) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot write binary file '{}': {}", path, e)),
                }
            } else {
                Value::Error(
                    "write_binary_file requires path (string) and bytes arguments".to_string(),
                )
            }
        }

        "append_file" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "append_file requires two arguments: path and content".to_string(),
                ));
            }
            if let (Some(Value::Str(path)), Some(Value::Str(content))) =
                (arg_values.first(), arg_values.get(1))
            {
                match OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => match file.write_all(content.as_bytes()) {
                        Ok(_) => Value::Bool(true),
                        Err(e) => Value::Error(format!("Cannot append to file '{}': {}", path, e)),
                    },
                    Err(e) => Value::Error(format!("Cannot open file '{}': {}", path, e)),
                }
            } else {
                Value::Error("append_file requires string arguments".to_string())
            }
        }

        "file_exists" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                if Path::new(path).exists() {
                    Value::Bool(true)
                } else {
                    Value::Bool(false)
                }
            } else {
                Value::Error("file_exists requires a string path argument".to_string())
            }
        }

        "read_lines" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        let lines: Vec<Value> =
                            content.lines().map(|line| Value::Str(line.to_string())).collect();
                        Value::Array(lines)
                    }
                    Err(e) => Value::Error(format!("Cannot read file '{}': {}", path, e)),
                }
            } else {
                Value::Error("read_lines requires a string path argument".to_string())
            }
        }

        "list_dir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                files.push(Value::Str(name.to_string()));
                            }
                        }
                        Value::Array(files)
                    }
                    Err(e) => Value::Error(format!("Cannot list directory '{}': {}", path, e)),
                }
            } else {
                Value::Error("list_dir requires a string path argument".to_string())
            }
        }

        "create_dir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::create_dir_all(path) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot create directory '{}': {}", path, e)),
                }
            } else {
                Value::Error("create_dir requires a string path argument".to_string())
            }
        }

        "file_size" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::metadata(path) {
                    Ok(metadata) => Value::Int(metadata.len() as i64),
                    Err(e) => Value::Error(format!("Cannot get file size for '{}': {}", path, e)),
                }
            } else {
                Value::Error("file_size requires a string path argument".to_string())
            }
        }

        "delete_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::remove_file(path) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot delete file '{}': {}", path, e)),
                }
            } else {
                Value::Error("delete_file requires a string path argument".to_string())
            }
        }

        "rename_file" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "rename_file requires two arguments: old_path and new_path".to_string(),
                ));
            }
            if let (Some(Value::Str(old_path)), Some(Value::Str(new_path))) =
                (arg_values.first(), arg_values.get(1))
            {
                match std::fs::rename(old_path, new_path) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!(
                        "Cannot rename file '{}' to '{}': {}",
                        old_path, new_path, e
                    )),
                }
            } else {
                Value::Error("rename_file requires string arguments".to_string())
            }
        }

        "copy_file" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "copy_file requires two arguments: source and dest".to_string(),
                ));
            }
            if let (Some(Value::Str(source)), Some(Value::Str(dest))) =
                (arg_values.first(), arg_values.get(1))
            {
                match std::fs::copy(source, dest) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!(
                        "Cannot copy file '{}' to '{}': {}",
                        source, dest, e
                    )),
                }
            } else {
                Value::Error("copy_file requires string arguments".to_string())
            }
        }

        _ => return None,
    };

    Some(result)
}
