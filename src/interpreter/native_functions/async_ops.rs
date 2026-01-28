// File: src/interpreter/native_functions/async_ops.rs
//
// Asynchronous operations native functions for Ruff.
// Provides async functions for sleep, timeouts, Promise.all, Promise.race, etc.

use crate::interpreter::{AsyncRuntime, Value};
use std::time::Duration;

/// Handle async operations native functions
pub fn handle(_interp: &mut crate::interpreter::Interpreter, name: &str, args: &[Value]) -> Option<Value> {
    match name {
        "async_sleep" => {
            // async_sleep(milliseconds: Int) -> Promise<Null>
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "async_sleep() expects 1 argument (milliseconds), got {}",
                    args.len()
                )));
            }

            let ms = match &args[0] {
                Value::Int(n) if *n >= 0 => *n as u64,
                Value::Int(n) => {
                    return Some(Value::Error(format!(
                        "async_sleep() requires non-negative milliseconds, got {}",
                        n
                    )));
                }
                _ => {
                    return Some(Value::Error(
                        "async_sleep() requires an integer milliseconds argument".to_string(),
                    ));
                }
            };

            // Create a tokio oneshot channel
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn async task that sleeps then resolves
            AsyncRuntime::spawn_task(async move {
                AsyncRuntime::sleep(Duration::from_millis(ms)).await;
                let _ = tx.send(Ok(Value::Null));
                Value::Null // Task return value (not used)
            });

            // Return promise immediately
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        "async_timeout" => {
            // async_timeout(promise: Promise, timeout_ms: Int) -> Promise<Value>
            // Race a promise against a timeout - returns Error if timeout expires
            if args.len() != 2 {
                return Some(Value::Error(format!(
                    "async_timeout() expects 2 arguments (promise, timeout_ms), got {}",
                    args.len()
                )));
            }

            // Extract timeout duration
            let timeout_ms = match &args[1] {
                Value::Int(n) if *n > 0 => *n as u64,
                Value::Int(n) => {
                    return Some(Value::Error(format!(
                        "async_timeout() requires positive timeout_ms, got {}",
                        n
                    )));
                }
                _ => {
                    return Some(Value::Error(
                        "async_timeout() requires an integer timeout_ms argument".to_string(),
                    ));
                }
            };

            // Extract the promise
            match &args[0] {
                Value::Promise { receiver, is_polled, cached_result } => {
                    // Clone the Arc pointers to move into async task
                    let receiver = receiver.clone();
                    let is_polled = is_polled.clone();
                    let cached_result = cached_result.clone();

                    // Create new channel for timeout result
                    let (tx, rx) = tokio::sync::oneshot::channel();

                    // Spawn task that races promise against timeout
                    AsyncRuntime::spawn_task(async move {
                        // Check if promise already polled (has cached result)
                        {
                            let polled = is_polled.lock().unwrap();
                            let cached = cached_result.lock().unwrap();

                            if *polled {
                                // Use cached result
                                let result = match cached.as_ref() {
                                    Some(Ok(val)) => Ok(val.clone()),
                                    Some(Err(err)) => Err(format!("Promise rejected: {}", err)),
                                    None => Err("Promise polled but no result cached".to_string()),
                                };
                                let _ = tx.send(result);
                                return Value::Null;
                            }
                        }

                        // Extract the receiver from the mutex
                        let actual_rx = {
                            let mut recv_guard = receiver.lock().unwrap();
                            let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
                            drop(dummy_tx);
                            std::mem::replace(&mut *recv_guard, dummy_rx)
                        };

                        // Race the promise against the timeout
                        let timeout_result = AsyncRuntime::timeout(
                            Duration::from_millis(timeout_ms),
                            actual_rx,
                        )
                        .await;

                        // Process result
                        let result = match timeout_result {
                            Ok(Ok(Ok(value))) => {
                                // Promise resolved successfully within timeout
                                Ok(value)
                            }
                            Ok(Ok(Err(err))) => {
                                // Promise rejected (not a timeout)
                                Err(format!("Promise rejected: {}", err))
                            }
                            Ok(Err(_)) => {
                                // Channel closed without sending
                                Err("Promise never resolved".to_string())
                            }
                            Err(_elapsed) => {
                                // Timeout elapsed
                                Err(format!("Timeout after {}ms", timeout_ms))
                            }
                        };

                        let _ = tx.send(result);
                        Value::Null
                    });

                    // Return new promise
                    Some(Value::Promise {
                        receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                        is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                        cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                    })
                }
                _ => Some(Value::Error(
                    "async_timeout() requires a Promise as first argument".to_string(),
                )),
            }
        }

        "async_http_get" => {
            // async_http_get(url: String) -> Promise<Dict>
            // Non-blocking HTTP GET request
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "async_http_get() expects 1 argument (url), got {}",
                    args.len()
                )));
            }

            let url = match &args[0] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_http_get() requires a string URL argument".to_string(),
                    ));
                }
            };

            // Create channel for result
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn async HTTP request
            AsyncRuntime::spawn_task(async move {
                let result = async {
                    let client = reqwest::Client::new();
                    let response = client.get(&url).send().await.map_err(|e| format!("HTTP GET failed: {}", e))?;
                    
                    let status = response.status().as_u16() as i64;
                    let headers_map = response.headers().clone();
                    let body = response.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;
                    
                    // Build result dictionary
                    let mut result_dict = std::collections::HashMap::new();
                    result_dict.insert("status".to_string(), Value::Int(status));
                    result_dict.insert("body".to_string(), Value::Str(body));
                    
                    // Convert headers to dict
                    let mut headers_dict = std::collections::HashMap::new();
                    for (name, value) in headers_map.iter() {
                        if let Ok(value_str) = value.to_str() {
                            headers_dict.insert(name.to_string(), Value::Str(value_str.to_string()));
                        }
                    }
                    result_dict.insert("headers".to_string(), Value::Dict(headers_dict));
                    
                    Ok::<Value, String>(Value::Dict(result_dict))
                }.await;
                
                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        "async_http_post" => {
            // async_http_post(url: String, body: String, headers?: Dict) -> Promise<Dict>
            // Non-blocking HTTP POST request
            if args.len() < 2 || args.len() > 3 {
                return Some(Value::Error(format!(
                    "async_http_post() expects 2-3 arguments (url, body, headers?), got {}",
                    args.len()
                )));
            }

            let url = match &args[0] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_http_post() requires a string URL argument".to_string(),
                    ));
                }
            };

            let body = match &args[1] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_http_post() requires a string body argument".to_string(),
                    ));
                }
            };

            // Optional headers
            let headers = if args.len() == 3 {
                match &args[2] {
                    Value::Dict(dict) => Some(dict.clone()),
                    _ => {
                        return Some(Value::Error(
                            "async_http_post() headers must be a dictionary".to_string(),
                        ));
                    }
                }
            } else {
                None
            };

            // Create channel for result
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn async HTTP request
            AsyncRuntime::spawn_task(async move {
                let result = async {
                    let client = reqwest::Client::new();
                    let mut request = client.post(&url).body(body);
                    
                    // Add custom headers if provided
                    if let Some(headers_dict) = headers {
                        for (key, value) in headers_dict.iter() {
                            if let Value::Str(value_str) = value {
                                request = request.header(key, value_str);
                            }
                        }
                    }
                    
                    let response = request.send().await.map_err(|e| format!("HTTP POST failed: {}", e))?;
                    
                    let status = response.status().as_u16() as i64;
                    let headers_map = response.headers().clone();
                    let response_body = response.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;
                    
                    // Build result dictionary
                    let mut result_dict = std::collections::HashMap::new();
                    result_dict.insert("status".to_string(), Value::Int(status));
                    result_dict.insert("body".to_string(), Value::Str(response_body));
                    
                    // Convert headers to dict
                    let mut headers_dict = std::collections::HashMap::new();
                    for (name, value) in headers_map.iter() {
                        if let Ok(value_str) = value.to_str() {
                            headers_dict.insert(name.to_string(), Value::Str(value_str.to_string()));
                        }
                    }
                    result_dict.insert("headers".to_string(), Value::Dict(headers_dict));
                    
                    Ok::<Value, String>(Value::Dict(result_dict))
                }.await;
                
                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        "async_read_file" => {
            // async_read_file(path: String) -> Promise<String>
            // Non-blocking file read
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "async_read_file() expects 1 argument (path), got {}",
                    args.len()
                )));
            }

            let path = match &args[0] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_read_file() requires a string path argument".to_string(),
                    ));
                }
            };

            // Create channel for result
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn async file read
            AsyncRuntime::spawn_task(async move {
                let result = async {
                    tokio::fs::read_to_string(&path)
                        .await
                        .map(Value::Str)
                        .map_err(|e| format!("Failed to read file '{}': {}", path, e))
                }.await;
                
                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        "async_write_file" => {
            // async_write_file(path: String, content: String) -> Promise<Bool>
            // Non-blocking file write
            if args.len() != 2 {
                return Some(Value::Error(format!(
                    "async_write_file() expects 2 arguments (path, content), got {}",
                    args.len()
                )));
            }

            let path = match &args[0] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_write_file() requires a string path argument".to_string(),
                    ));
                }
            };

            let content = match &args[1] {
                Value::Str(s) => s.clone(),
                _ => {
                    return Some(Value::Error(
                        "async_write_file() requires a string content argument".to_string(),
                    ));
                }
            };

            // Create channel for result
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn async file write
            AsyncRuntime::spawn_task(async move {
                let result = async {
                    tokio::fs::write(&path, content)
                        .await
                        .map(|_| Value::Bool(true))
                        .map_err(|e| format!("Failed to write file '{}': {}", path, e))
                }.await;
                
                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        "spawn_task" => {
            // spawn_task(async_func: AsyncFunction) -> TaskHandle
            // Spawn a background task that runs independently
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "spawn_task() expects 1 argument (async function), got {}",
                    args.len()
                )));
            }

            let func = match &args[0] {
                Value::AsyncFunction(params, body, env) => {
                    (params.clone(), body.clone(), env.clone())
                }
                Value::Function(params, body, env) => {
                    // Allow regular functions to be spawned as tasks too
                    (params.clone(), body.clone(), env.clone())
                }
                _ => {
                    return Some(Value::Error(
                        "spawn_task() requires an async function argument".to_string(),
                    ));
                }
            };

            let (_params, _body, _env) = func;

            // Clone interpreter context needed for execution
            // Note: We need to pass the interpreter or create a way to execute
            // For now, we'll create a simple task that just returns the function
            // In a real implementation, we'd need to execute the function body
            // TODO: Task #35 - Execute actual function body with interpreter context
            
            // Create the task handle
            let is_cancelled = std::sync::Arc::new(std::sync::Mutex::new(false));
            let is_cancelled_clone = is_cancelled.clone();
            
            // Spawn the task
            let handle = AsyncRuntime::spawn_task(async move {
                // Check if cancelled
                {
                    let cancelled = is_cancelled_clone.lock().unwrap();
                    if *cancelled {
                        return Value::Error("Task was cancelled".to_string());
                    }
                }
                
                // For now, just sleep to simulate work
                // TODO: Execute the actual function body with interpreter
                AsyncRuntime::sleep(std::time::Duration::from_millis(1)).await;
                
                // Return placeholder - in full implementation would execute function
                Value::Null
            });

            Some(Value::TaskHandle {
                handle: std::sync::Arc::new(std::sync::Mutex::new(Some(handle))),
                is_cancelled,
            })
        }

        "await_task" => {
            // await_task(task_handle: TaskHandle) -> Promise<Value>
            // Wait for a spawned task to complete and get its result
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "await_task() expects 1 argument (task handle), got {}",
                    args.len()
                )));
            }

            match &args[0] {
                Value::TaskHandle { handle: handle_arc, is_cancelled } => {
                    let handle_arc = handle_arc.clone();
                    let is_cancelled = is_cancelled.clone();
                    
                    // Create channel for result
                    let (tx, rx) = tokio::sync::oneshot::channel();

                    // Spawn task to await the handle
                    AsyncRuntime::spawn_task(async move {
                        // Extract the handle
                        let handle = {
                            let mut handle_guard = handle_arc.lock().unwrap();
                            handle_guard.take()
                        };

                        let result = if let Some(h) = handle {
                            // Check if cancelled (drop guard before await)
                            let is_task_cancelled = {
                                let cancelled = is_cancelled.lock().unwrap();
                                *cancelled
                            };
                            
                            if is_task_cancelled {
                                Err("Task was cancelled".to_string())
                            } else {
                                // Await the task completion
                                match h.await {
                                    Ok(value) => Ok(value),
                                    Err(e) => Err(format!("Task panicked: {}", e)),
                                }
                            }
                        } else {
                            Err("Task handle already consumed".to_string())
                        };

                        let _ = tx.send(result);
                        Value::Null
                    });

                    // Return promise
                    Some(Value::Promise {
                        receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                        is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                        cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                    })
                }
                _ => Some(Value::Error(
                    "await_task() requires a TaskHandle argument".to_string(),
                )),
            }
        }

        "cancel_task" => {
            // cancel_task(task_handle: TaskHandle) -> Bool
            // Request cancellation of a running task
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "cancel_task() expects 1 argument (task handle), got {}",
                    args.len()
                )));
            }

            match &args[0] {
                Value::TaskHandle { handle: handle_arc, is_cancelled } => {
                    // Mark as cancelled
                    {
                        let mut cancelled = is_cancelled.lock().unwrap();
                        *cancelled = true;
                    }
                    
                    // Abort the task if possible
                    let mut handle_guard = handle_arc.lock().unwrap();
                    if let Some(handle) = handle_guard.take() {
                        handle.abort();
                        Some(Value::Bool(true))
                    } else {
                        Some(Value::Bool(false)) // Already consumed
                    }
                }
                _ => Some(Value::Error(
                    "cancel_task() requires a TaskHandle argument".to_string(),
                )),
            }
        }

        "Promise.all" | "promise_all" => {
            // Promise.all(promises: Array<Promise>) -> Promise<Array<Value>>
            // Await all promises concurrently, return array of results
            // If any promise rejects, the whole operation fails with first error
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "Promise.all() expects 1 argument (array of promises), got {}",
                    args.len()
                )));
            }

            // Extract array of promises
            let promises = match &args[0] {
                Value::Array(arr) => arr.clone(),
                _ => {
                    return Some(Value::Error(
                        "Promise.all() requires an array of promises".to_string(),
                    ));
                }
            };

            if promises.is_empty() {
                // Empty array - return immediately resolved promise with empty array
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(Ok(Value::Array(vec![])));
                return Some(Value::Promise {
                    receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                    is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                });
            }

            // Extract receivers from all promises
            let mut receivers = Vec::new();
            for (idx, promise) in promises.iter().enumerate() {
                match promise {
                    Value::Promise { receiver, .. } => {
                        receivers.push((idx, receiver.clone()));
                    }
                    _ => {
                        return Some(Value::Error(format!(
                            "Promise.all() array element {} is not a Promise",
                            idx
                        )));
                    }
                }
            }

            // Create channel for final result
            let (tx, rx) = tokio::sync::oneshot::channel();
            let count = receivers.len();

            // Spawn task that awaits all promises concurrently
            AsyncRuntime::spawn_task(async move {
                // Extract all receivers
                let mut futures = Vec::new();
                for (idx, receiver_arc) in receivers {
                    let actual_rx = {
                        let mut recv_guard = receiver_arc.lock().unwrap();
                        let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
                        drop(dummy_tx);
                        std::mem::replace(&mut *recv_guard, dummy_rx)
                    };
                    futures.push((idx, actual_rx));
                }

                // Await all futures concurrently using tokio::join! or futures combinator
                // We'll use a simple loop that spawns and collects
                let mut tasks = Vec::new();
                for (idx, rx) in futures {
                    tasks.push(tokio::spawn(async move {
                        (idx, rx.await)
                    }));
                }

                // Collect all results
                let mut results = vec![Value::Null; count];
                for task in tasks {
                    match task.await {
                        Ok((idx, Ok(Ok(value)))) => {
                            results[idx] = value;
                        }
                        Ok((idx, Ok(Err(err)))) => {
                            let _ = tx.send(Err(format!("Promise {} rejected: {}", idx, err)));
                            return Value::Null;
                        }
                        Ok((_, Err(_))) | Err(_) => {
                            let _ = tx.send(Err("Promise never resolved".to_string()));
                            return Value::Null;
                        }
                    }
                }

                // All promises resolved successfully
                let _ = tx.send(Ok(Value::Array(results)));
                Value::Null
            });

            // Return promise that resolves to array of results
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            })
        }

        _ => None, // Not handled by this module
    }
}
