// File: src/interpreter/native_functions/filesystem.rs
//
// Filesystem operation native functions

use crate::interpreter::{AsyncRuntime, Interpreter, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn handle(_interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Async file operations - return Promises for true concurrency
        "read_file_async" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                let path_clone = path.clone();

                // Create oneshot channel for result
                let (tx, rx) = tokio::sync::oneshot::channel();

                // Spawn async task to read file
                AsyncRuntime::spawn_task(async move {
                    match tokio::fs::read_to_string(path_clone.as_ref()).await {
                        Ok(content) => {
                            let _ = tx.send(Ok(Value::Str(Arc::new(content))));
                        }
                        Err(e) => {
                            let path_str = path_clone.as_ref().clone();
                            let _ = tx.send(Err(format!("Cannot read file '{}': {}", path_str, e)));
                        }
                    }
                    Value::Null
                });

                Value::Promise {
                    receiver: Arc::new(Mutex::new(rx)),
                    is_polled: Arc::new(Mutex::new(false)),
                    cached_result: Arc::new(Mutex::new(None)),
                    task_handle: None,
                }
            } else {
                Value::Error("read_file requires a string path argument".to_string())
            }
        }

        "write_file_async" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "write_file requires two arguments: path and content".to_string(),
                ));
            }
            if let (Some(Value::Str(path)), Some(Value::Str(content))) =
                (arg_values.first(), arg_values.get(1))
            {
                let path_clone = path.clone();
                let content_clone = content.clone();

                // Create oneshot channel for result
                let (tx, rx) = tokio::sync::oneshot::channel();

                // Spawn async task to write file
                AsyncRuntime::spawn_task(async move {
                    match tokio::fs::write(path_clone.as_ref(), content_clone.as_ref()).await {
                        Ok(_) => {
                            let _ = tx.send(Ok(Value::Bool(true)));
                        }
                        Err(e) => {
                            let path_str = path_clone.as_ref().clone();
                            let _ =
                                tx.send(Err(format!("Cannot write file '{}': {}", path_str, e)));
                        }
                    }
                    Value::Null
                });

                Value::Promise {
                    receiver: Arc::new(Mutex::new(rx)),
                    is_polled: Arc::new(Mutex::new(false)),
                    cached_result: Arc::new(Mutex::new(None)),
                    task_handle: None,
                }
            } else {
                Value::Error("write_file requires string arguments".to_string())
            }
        }

        "list_dir_async" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                let path_clone = path.clone();

                // Create oneshot channel for result
                let (tx, rx) = tokio::sync::oneshot::channel();

                // Spawn async task to list directory
                AsyncRuntime::spawn_task(async move {
                    match tokio::fs::read_dir(path_clone.as_ref()).await {
                        Ok(mut entries) => {
                            let mut files = Vec::new();
                            while let Ok(Some(entry)) = entries.next_entry().await {
                                if let Some(name) = entry.file_name().to_str() {
                                    files.push(Value::Str(Arc::new(name.to_string())));
                                }
                            }
                            let _ = tx.send(Ok(Value::Array(Arc::new(files))));
                        }
                        Err(e) => {
                            let path_str = path_clone.as_ref().clone();
                            let _ = tx
                                .send(Err(format!("Cannot list directory '{}': {}", path_str, e)));
                        }
                    }
                    Value::Null
                });

                Value::Promise {
                    receiver: Arc::new(Mutex::new(rx)),
                    is_polled: Arc::new(Mutex::new(false)),
                    cached_result: Arc::new(Mutex::new(None)),
                    task_handle: None,
                }
            } else {
                Value::Error("list_dir requires a string path argument".to_string())
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
                match std::fs::write(path.as_ref(), content.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot write file '{}': {}", path.as_ref(), e)),
                }
            } else {
                Value::Error("write_file requires string arguments".to_string())
            }
        }

        // Synchronous fallback versions for compatibility
        "read_file_sync" | "read_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_to_string(path.as_ref()) {
                    Ok(content) => Value::Str(Arc::new(content)),
                    Err(e) => Value::Error(format!("Cannot read file '{}': {}", path.as_ref(), e)),
                }
            } else {
                Value::Error("read_file_sync requires a string path argument".to_string())
            }
        }

        "write_file_sync" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "write_file_sync requires two arguments: path and content".to_string(),
                ));
            }
            if let (Some(Value::Str(path)), Some(Value::Str(content))) =
                (arg_values.first(), arg_values.get(1))
            {
                match std::fs::write(path.as_ref(), content.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!("Cannot write file '{}': {}", path.as_ref(), e)),
                }
            } else {
                Value::Error("write_file_sync requires string arguments".to_string())
            }
        }

        "list_dir_sync" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_dir(path.as_ref()) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                files.push(Value::Str(Arc::new(name.to_string())));
                            }
                        }
                        Value::Array(Arc::new(files))
                    }
                    Err(e) => {
                        Value::Error(format!("Cannot list directory '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("list_dir_sync requires a string path argument".to_string())
            }
        }

        "read_binary_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read(path.as_ref()) {
                    Ok(bytes) => Value::Bytes(bytes),
                    Err(e) => {
                        Value::Error(format!("Cannot read binary file '{}': {}", path.as_ref(), e))
                    }
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
                match std::fs::write(path.as_ref(), bytes) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => {
                        Value::Error(format!("Cannot write binary file '{}': {}", path.as_ref(), e))
                    }
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
                match OpenOptions::new().create(true).append(true).open(path.as_ref()) {
                    Ok(mut file) => match file.write_all(content.as_ref().as_bytes()) {
                        Ok(_) => Value::Bool(true),
                        Err(e) => Value::Error(format!(
                            "Cannot append to file '{}': {}",
                            path.as_ref(),
                            e
                        )),
                    },
                    Err(e) => Value::Error(format!("Cannot open file '{}': {}", path.as_ref(), e)),
                }
            } else {
                Value::Error("append_file requires string arguments".to_string())
            }
        }

        "file_exists" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                if Path::new(path.as_ref()).exists() {
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
                match std::fs::read_to_string(path.as_ref()) {
                    Ok(content) => {
                        let lines: Vec<Value> = content
                            .lines()
                            .map(|line| Value::Str(Arc::new(line.to_string())))
                            .collect();
                        Value::Array(Arc::new(lines))
                    }
                    Err(e) => Value::Error(format!("Cannot read file '{}': {}", path.as_ref(), e)),
                }
            } else {
                Value::Error("read_lines requires a string path argument".to_string())
            }
        }

        "list_dir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::read_dir(path.as_ref()) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                files.push(Value::Str(Arc::new(name.to_string())));
                            }
                        }
                        Value::Array(Arc::new(files))
                    }
                    Err(e) => {
                        Value::Error(format!("Cannot list directory '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("list_dir requires a string path argument".to_string())
            }
        }

        "create_dir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::create_dir_all(path.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => {
                        Value::Error(format!("Cannot create directory '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("create_dir requires a string path argument".to_string())
            }
        }

        "file_size" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::metadata(path.as_ref()) {
                    Ok(metadata) => Value::Int(metadata.len() as i64),
                    Err(e) => {
                        Value::Error(format!("Cannot get file size for '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("file_size requires a string path argument".to_string())
            }
        }

        "delete_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::remove_file(path.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => {
                        Value::Error(format!("Cannot delete file '{}': {}", path.as_ref(), e))
                    }
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
                match std::fs::rename(old_path.as_ref(), new_path.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!(
                        "Cannot rename file '{}' to '{}': {}",
                        old_path.as_ref(),
                        new_path.as_ref(),
                        e
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
                match std::fs::copy(source.as_ref(), dest.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!(
                        "Cannot copy file '{}' to '{}': {}",
                        source.as_ref(),
                        dest.as_ref(),
                        e
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
