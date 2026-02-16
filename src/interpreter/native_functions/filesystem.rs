// File: src/interpreter/native_functions/filesystem.rs
//
// Filesystem operation native functions

use crate::interpreter::{AsyncRuntime, Interpreter, Value};
use crate::{builtins, interpreter::DictMap};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

fn zip_add_dir_recursive(
    zip_writer: &mut ZipWriter<File>,
    directory_path: &Path,
    zip_prefix: &str,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory_path)
        .map_err(|error| format!("Failed to read directory: {}", error))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("Failed to read entry: {}", error))?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let zip_path = if zip_prefix.is_empty() {
            file_name.to_string()
        } else {
            format!("{}/{}", zip_prefix, file_name)
        };

        if entry_path.is_dir() {
            let options = FileOptions::default();
            zip_writer
                .add_directory(&zip_path, options)
                .map_err(|error| format!("Failed to add directory '{}': {}", zip_path, error))?;
            zip_add_dir_recursive(zip_writer, &entry_path, &zip_path)?;
        } else {
            let file_contents = std::fs::read(&entry_path)
                .map_err(|error| format!("Failed to read '{}': {}", entry_path.display(), error))?;
            let options =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip_writer
                .start_file(&zip_path, options)
                .map_err(|error| format!("Failed to start file '{}': {}", zip_path, error))?;
            zip_writer
                .write_all(&file_contents)
                .map_err(|error| format!("Failed to write file '{}': {}", zip_path, error))?;
        }
    }

    Ok(())
}

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
            if arg_values.len() != 2 {
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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "read_file_sync requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "read_binary_file requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 2 {
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

        "load_image" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match image::open(path.as_ref()) {
                    Ok(image_data) => {
                        let format = Path::new(path.as_ref())
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .unwrap_or("unknown")
                            .to_lowercase();

                        Value::Image { data: Arc::new(Mutex::new(image_data)), format }
                    }
                    Err(error) => {
                        Value::Error(format!("Cannot load image '{}': {}", path.as_ref(), error))
                    }
                }
            } else {
                Value::Error("load_image requires a string path argument".to_string())
            }
        }

        "zip_create" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "zip_create requires a string path argument".to_string(),
                ));
            }

            if let Some(Value::Str(path)) = arg_values.first() {
                match File::create(path.as_ref()) {
                    Ok(file) => {
                        let writer = ZipWriter::new(file);
                        Value::ZipArchive {
                            writer: Arc::new(Mutex::new(Some(writer))),
                            path: path.as_ref().clone(),
                        }
                    }
                    Err(error) => Value::ErrorObject {
                        message: format!(
                            "Failed to create zip file '{}': {}",
                            path.as_ref(),
                            error
                        ),
                        stack: Vec::new(),
                        line: None,
                        cause: None,
                    },
                }
            } else {
                Value::Error("zip_create requires a string path argument".to_string())
            }
        }

        "zip_add_file" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(
                    "zip_add_file requires (ZipArchive, string_path) arguments".to_string(),
                ));
            }

            match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::ZipArchive { writer, .. }), Some(Value::Str(source_path))) => {
                    let mut writer_guard = writer.lock().unwrap();
                    if let Some(zip_writer) = writer_guard.as_mut() {
                        match std::fs::read(source_path.as_ref()) {
                            Ok(file_contents) => {
                                let file_name = Path::new(source_path.as_ref())
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(source_path.as_ref());
                                let options = FileOptions::default()
                                    .compression_method(zip::CompressionMethod::Deflated);

                                match zip_writer.start_file(file_name, options) {
                                    Ok(_) => match zip_writer.write_all(&file_contents) {
                                        Ok(_) => Value::Bool(true),
                                        Err(error) => Value::ErrorObject {
                                            message: format!(
                                                "Failed to write file to zip: {}",
                                                error
                                            ),
                                            stack: Vec::new(),
                                            line: None,
                                            cause: None,
                                        },
                                    },
                                    Err(error) => Value::ErrorObject {
                                        message: format!(
                                            "Failed to start zip entry '{}': {}",
                                            file_name, error
                                        ),
                                        stack: Vec::new(),
                                        line: None,
                                        cause: None,
                                    },
                                }
                            }
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to read source file '{}': {}",
                                    source_path.as_ref(),
                                    error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    } else {
                        Value::Error("Zip archive has been closed".to_string())
                    }
                }
                _ => Value::Error(
                    "zip_add_file requires (ZipArchive, string_path) arguments".to_string(),
                ),
            }
        }

        "zip_add_dir" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(
                    "zip_add_dir requires (ZipArchive, string_path) arguments".to_string(),
                ));
            }

            match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::ZipArchive { writer, .. }), Some(Value::Str(directory_path))) => {
                    let mut writer_guard = writer.lock().unwrap();
                    if let Some(zip_writer) = writer_guard.as_mut() {
                        let directory_path = Path::new(directory_path.as_ref());
                        match zip_add_dir_recursive(zip_writer, directory_path, "") {
                            Ok(_) => Value::Bool(true),
                            Err(message) => Value::ErrorObject {
                                message,
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    } else {
                        Value::Error("Zip archive has been closed".to_string())
                    }
                }
                _ => Value::Error(
                    "zip_add_dir requires (ZipArchive, string_path) arguments".to_string(),
                ),
            }
        }

        "zip_close" => {
            if arg_values.len() != 1 {
                return Some(Value::Error("zip_close requires a ZipArchive argument".to_string()));
            }

            if let Some(Value::ZipArchive { writer, .. }) = arg_values.first() {
                let mut writer_guard = writer.lock().unwrap();
                if let Some(mut zip_writer) = writer_guard.take() {
                    match zip_writer.finish() {
                        Ok(_) => Value::Bool(true),
                        Err(error) => Value::ErrorObject {
                            message: format!("Failed to finalize zip archive: {}", error),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                } else {
                    Value::Error("Zip archive has already been closed".to_string())
                }
            } else {
                Value::Error("zip_close requires a ZipArchive argument".to_string())
            }
        }

        "unzip" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(
                    "unzip requires (string_zip_path, string_output_dir) arguments".to_string(),
                ));
            }

            match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Str(zip_path)), Some(Value::Str(output_dir))) => {
                    match File::open(zip_path.as_ref()) {
                        Ok(file) => match ZipArchive::new(file) {
                            Ok(mut archive) => {
                                if let Err(error) = std::fs::create_dir_all(output_dir.as_ref()) {
                                    return Some(Value::ErrorObject {
                                        message: format!(
                                            "Failed to create output directory '{}': {}",
                                            output_dir.as_ref(),
                                            error
                                        ),
                                        stack: Vec::new(),
                                        line: None,
                                        cause: None,
                                    });
                                }

                                let mut extracted_files = Vec::new();

                                for entry_index in 0..archive.len() {
                                    match archive.by_index(entry_index) {
                                        Ok(mut archive_file) => {
                                            let output_path = Path::new(output_dir.as_ref())
                                                .join(archive_file.name());

                                            if archive_file.is_dir() {
                                                if let Err(error) =
                                                    std::fs::create_dir_all(&output_path)
                                                {
                                                    return Some(Value::ErrorObject {
                                                        message: format!(
                                                            "Failed to create directory '{}': {}",
                                                            output_path.display(),
                                                            error
                                                        ),
                                                        stack: Vec::new(),
                                                        line: None,
                                                        cause: None,
                                                    });
                                                }
                                            } else {
                                                if let Some(parent) = output_path.parent() {
                                                    if let Err(error) =
                                                        std::fs::create_dir_all(parent)
                                                    {
                                                        return Some(Value::ErrorObject {
                                                            message: format!(
                                                                "Failed to create parent directory for '{}': {}",
                                                                output_path.display(),
                                                                error
                                                            ),
                                                            stack: Vec::new(),
                                                            line: None,
                                                            cause: None,
                                                        });
                                                    }
                                                }

                                                match File::create(&output_path) {
                                                    Ok(mut output_file) => {
                                                        if let Err(error) = std::io::copy(
                                                            &mut archive_file,
                                                            &mut output_file,
                                                        ) {
                                                            return Some(Value::ErrorObject {
                                                                message: format!(
                                                                "Failed to extract file '{}': {}",
                                                                archive_file.name(),
                                                                error
                                                            ),
                                                                stack: Vec::new(),
                                                                line: None,
                                                                cause: None,
                                                            });
                                                        }
                                                        extracted_files.push(Value::Str(Arc::new(
                                                            output_path
                                                                .to_string_lossy()
                                                                .to_string(),
                                                        )));
                                                    }
                                                    Err(error) => {
                                                        return Some(Value::ErrorObject {
                                                            message: format!(
                                                            "Failed to create output file '{}': {}",
                                                            output_path.display(),
                                                            error
                                                        ),
                                                            stack: Vec::new(),
                                                            line: None,
                                                            cause: None,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            return Some(Value::ErrorObject {
                                                message: format!(
                                                    "Failed to read zip entry {}: {}",
                                                    entry_index, error
                                                ),
                                                stack: Vec::new(),
                                                line: None,
                                                cause: None,
                                            });
                                        }
                                    }
                                }

                                Value::Array(Arc::new(extracted_files))
                            }
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to open zip archive '{}': {}",
                                    zip_path.as_ref(),
                                    error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        },
                        Err(error) => Value::ErrorObject {
                            message: format!(
                                "Failed to open file '{}': {}",
                                zip_path.as_ref(),
                                error
                            ),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                }
                _ => Value::Error(
                    "unzip requires (string_zip_path, string_output_dir) arguments".to_string(),
                ),
            }
        }

        "append_file" => {
            if arg_values.len() != 2 {
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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "file_exists requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "read_lines requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error("list_dir requires a string path argument".to_string()));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "create_dir requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error("file_size requires a string path argument".to_string()));
            }

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
            if arg_values.len() != 1 {
                return Some(Value::Error(
                    "delete_file requires a string path argument".to_string(),
                ));
            }

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
            if arg_values.len() != 2 {
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
            if arg_values.len() != 2 {
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

        // OS module functions
        "os_getcwd" => match std::env::current_dir() {
            Ok(path) => Value::Str(Arc::new(path.to_string_lossy().to_string())),
            Err(e) => Value::Error(format!("Cannot get current directory: {}", e)),
        },

        "os_chdir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::env::set_current_dir(path.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => Value::Error(format!(
                        "Cannot change directory to '{}': {}",
                        path.as_ref(),
                        e
                    )),
                }
            } else {
                Value::Error("os_chdir requires a string argument (path)".to_string())
            }
        }

        "os_rmdir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::remove_dir(path.as_ref()) {
                    Ok(_) => Value::Bool(true),
                    Err(e) => {
                        Value::Error(format!("Cannot remove directory '{}': {}", path.as_ref(), e))
                    }
                }
            } else {
                Value::Error("os_rmdir requires a string argument (path)".to_string())
            }
        }

        "os_environ" => {
            let mut dict = DictMap::default();
            for (key, value) in std::env::vars() {
                dict.insert(Arc::<str>::from(key), Value::Str(Arc::new(value)));
            }
            Value::Dict(Arc::new(dict))
        }

        // Path operation functions
        "join_path" | "path_join" => {
            if arg_values.is_empty() {
                Value::Error(format!("{} requires at least one string argument", name))
            } else {
                let mut parts: Vec<String> = Vec::with_capacity(arg_values.len());
                for (index, value) in arg_values.iter().enumerate() {
                    match value {
                        Value::Str(s) => parts.push(s.as_ref().clone()),
                        _ => {
                            return Some(Value::Error(format!(
                                "{} argument {} must be a string",
                                name,
                                index + 1
                            )));
                        }
                    }
                }

                Value::Str(Arc::new(builtins::join_path(&parts)))
            }
        }

        "dirname" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                Value::Str(Arc::new(builtins::dirname(path.as_ref())))
            } else {
                Value::Error("dirname requires a string argument (path)".to_string())
            }
        }

        "basename" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                Value::Str(Arc::new(builtins::basename(path.as_ref())))
            } else {
                Value::Error("basename requires a string argument (path)".to_string())
            }
        }

        "path_exists" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                Value::Bool(builtins::path_exists(path.as_ref()))
            } else {
                Value::Error("path_exists requires a string argument (path)".to_string())
            }
        }

        "path_absolute" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match std::fs::canonicalize(Path::new(path.as_ref())) {
                    Ok(abs_path) => Value::Str(Arc::new(abs_path.to_string_lossy().to_string())),
                    Err(e) => Value::Error(format!(
                        "Cannot get absolute path for '{}': {}",
                        path.as_ref(),
                        e
                    )),
                }
            } else {
                Value::Error("path_absolute requires a string argument (path)".to_string())
            }
        }

        "path_is_dir" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                Value::Bool(Path::new(path.as_ref()).is_dir())
            } else {
                Value::Error("path_is_dir requires a string argument (path)".to_string())
            }
        }

        "path_is_file" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                Value::Bool(Path::new(path.as_ref()).is_file())
            } else {
                Value::Error("path_is_file requires a string argument (path)".to_string())
            }
        }

        "path_extension" => {
            if let Some(Value::Str(path)) = arg_values.first() {
                match Path::new(path.as_ref()).extension() {
                    Some(ext) => Value::Str(Arc::new(ext.to_string_lossy().to_string())),
                    None => Value::Str(Arc::new(String::new())),
                }
            } else {
                Value::Error("path_extension requires a string argument (path)".to_string())
            }
        }

        _ => return None,
    };

    Some(result)
}
