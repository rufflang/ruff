// File: src/interpreter/native_functions/async_ops.rs
//
// Asynchronous operations native functions for Ruff.
// Provides async functions for sleep, timeouts, Promise.all, Promise.race, etc.

use crate::interpreter::{AsyncRuntime, DictMap, Value};
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use std::time::Duration;

/// Handle async operations native functions
pub fn handle(
    _interp: &mut crate::interpreter::Interpreter,
    name: &str,
    args: &[Value],
) -> Option<Value> {
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
                task_handle: None,
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
                Value::Promise { receiver, is_polled, cached_result, .. } => {
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
                        let timeout_result =
                            AsyncRuntime::timeout(Duration::from_millis(timeout_ms), actual_rx)
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
                        task_handle: None,
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
                    let response = client
                        .get(url.as_ref())
                        .send()
                        .await
                        .map_err(|e| format!("HTTP GET failed: {}", e))?;

                    let status = response.status().as_u16() as i64;
                    let headers_map = response.headers().clone();
                    let body = response
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read response body: {}", e))?;

                    // Build result dictionary
                    let mut result_dict = DictMap::default();
                    result_dict.insert("status".into(), Value::Int(status));
                    result_dict.insert("body".into(), Value::Str(Arc::new(body)));

                    // Convert headers to dict
                    let mut headers_dict = DictMap::default();
                    for (name, value) in headers_map.iter() {
                        if let Ok(value_str) = value.to_str() {
                            headers_dict.insert(
                                name.to_string().into(),
                                Value::Str(Arc::new(value_str.to_string())),
                            );
                        }
                    }
                    result_dict.insert("headers".into(), Value::Dict(Arc::new(headers_dict)));

                    Ok::<Value, String>(Value::Dict(Arc::new(result_dict)))
                }
                .await;

                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                task_handle: None,
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
                    let mut request = client.post(url.as_ref()).body(body.as_ref().clone());

                    // Add custom headers if provided
                    if let Some(headers_dict) = headers {
                        for (key, value) in headers_dict.iter() {
                            if let Value::Str(value_str) = value {
                                request = request.header(key.as_ref(), value_str.as_str());
                            }
                        }
                    }

                    let response =
                        request.send().await.map_err(|e| format!("HTTP POST failed: {}", e))?;

                    let status = response.status().as_u16() as i64;
                    let headers_map = response.headers().clone();
                    let response_body = response
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read response body: {}", e))?;

                    // Build result dictionary
                    let mut result_dict = DictMap::default();
                    result_dict.insert("status".into(), Value::Int(status));
                    result_dict.insert("body".into(), Value::Str(Arc::new(response_body)));

                    // Convert headers to dict
                    let mut headers_dict = DictMap::default();
                    for (name, value) in headers_map.iter() {
                        if let Ok(value_str) = value.to_str() {
                            headers_dict.insert(
                                name.to_string().into(),
                                Value::Str(Arc::new(value_str.to_string())),
                            );
                        }
                    }
                    result_dict.insert("headers".into(), Value::Dict(Arc::new(headers_dict)));

                    Ok::<Value, String>(Value::Dict(Arc::new(result_dict)))
                }
                .await;

                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                task_handle: None,
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
                    tokio::fs::read_to_string(path.as_ref())
                        .await
                        .map(|s| Value::Str(Arc::new(s)))
                        .map_err(|e| format!("Failed to read file '{}': {}", path.as_ref(), e))
                }
                .await;

                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                task_handle: None,
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
                    tokio::fs::write(path.as_ref(), content.as_ref())
                        .await
                        .map(|_| Value::Bool(true))
                        .map_err(|e| format!("Failed to write file '{}': {}", path.as_ref(), e))
                }
                .await;

                let _ = tx.send(result);
                Value::Null
            });

            // Return promise
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                task_handle: None,
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
                        task_handle: None,
                    })
                }
                _ => Some(Value::Error("await_task() requires a TaskHandle argument".to_string())),
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
                _ => Some(Value::Error("cancel_task() requires a TaskHandle argument".to_string())),
            }
        }

        "Promise.all" | "promise_all" => {
            // Promise.all(promises: Array<Promise>, concurrency_limit?: Int) -> Promise<Array<Value>>
            // Await all promises concurrently, return array of results
            // If any promise rejects, the whole operation fails with first error
            if args.len() != 1 && args.len() != 2 {
                return Some(Value::Error(format!(
                    "Promise.all() expects 1 or 2 arguments (array of promises, optional concurrency_limit), got {}",
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

            // Optional concurrency limit (batch size for awaiting)
            let concurrency_limit = if args.len() == 2 {
                match &args[1] {
                    Value::Int(n) if *n > 0 => *n as usize,
                    Value::Int(n) => {
                        return Some(Value::Error(format!(
                            "Promise.all() concurrency_limit must be > 0, got {}",
                            n
                        )));
                    }
                    _ => {
                        return Some(Value::Error(
                            "Promise.all() optional concurrency_limit must be an integer"
                                .to_string(),
                        ));
                    }
                }
            } else {
                _interp.get_async_task_pool_size()
            };

            // Debug logging
            if std::env::var("DEBUG_ASYNC").is_ok() {
                eprintln!("Promise.all: received {} promises", promises.len());
            }

            if promises.is_empty() {
                // Empty array - return immediately resolved promise with empty array
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(Ok(Value::Array(Arc::new(vec![]))));
                return Some(Value::Promise {
                    receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                    is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                    cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                    task_handle: None,
                });
            }

            // Extract receivers from all promises
            let mut receivers = Vec::new();
            for (idx, promise) in promises.iter().enumerate() {
                match promise {
                    Value::Promise { receiver, .. } => {
                        if std::env::var("DEBUG_ASYNC").is_ok() {
                            eprintln!("Promise.all: extracting receiver from promise {}", idx);
                        }
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

            if std::env::var("DEBUG_ASYNC").is_ok() {
                eprintln!("Promise.all: extracted {} receivers, spawning task", receivers.len());
            }

            // Create channel for final result
            let (tx, rx) = tokio::sync::oneshot::channel();
            let count = receivers.len();
            let effective_batch_size = concurrency_limit.min(count.max(1));

            // Spawn task that awaits all promises concurrently
            AsyncRuntime::spawn_task(async move {
                if std::env::var("DEBUG_ASYNC").is_ok() {
                    eprintln!(
                        "Promise.all: inside spawned task, extracting {} receivers",
                        receivers.len()
                    );
                }

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

                if std::env::var("DEBUG_ASYNC").is_ok() {
                    eprintln!(
                        "Promise.all: prepared {} futures, awaiting with bounded in-task concurrency",
                        futures.len()
                    );
                }

                // Collect all results
                let mut results = vec![Value::Null; count];
                let mut completed = 0;

                // Await receivers with bounded in-task concurrency (no per-receiver tokio::spawn overhead)
                let mut pending = futures.into_iter();
                let mut in_flight = FuturesUnordered::new();
                let make_wait_future = |idx, rx| async move { (idx, rx.await) };

                for _ in 0..effective_batch_size {
                    match pending.next() {
                        Some((idx, rx)) => {
                            in_flight.push(make_wait_future(idx, rx));
                        }
                        None => break,
                    }
                }

                while let Some((idx, recv_result)) = in_flight.next().await {
                    if std::env::var("DEBUG_ASYNC").is_ok() {
                        eprintln!("Promise.all: awaiting task {}/{}", completed + 1, count);
                    }

                    match recv_result {
                        Ok(Ok(value)) => {
                            if std::env::var("DEBUG_ASYNC").is_ok() {
                                eprintln!("Promise.all: task {} resolved successfully", idx);
                            }
                            results[idx] = value;
                            completed += 1;
                        }
                        Ok(Err(err)) => {
                            if std::env::var("DEBUG_ASYNC").is_ok() {
                                eprintln!("Promise.all: task {} rejected: {}", idx, err);
                            }
                            let _ = tx.send(Err(format!("Promise {} rejected: {}", idx, err)));
                            return Value::Null;
                        }
                        Err(_) => {
                            if std::env::var("DEBUG_ASYNC").is_ok() {
                                eprintln!(
                                    "Promise.all: task {} channel closed (Err from oneshot)",
                                    idx
                                );
                            }
                            let _ = tx.send(Err(format!(
                                "Promise {} never resolved (channel closed)",
                                idx
                            )));
                            return Value::Null;
                        }
                    }

                    if let Some((next_idx, next_rx)) = pending.next() {
                        in_flight.push(make_wait_future(next_idx, next_rx));
                    }
                }

                if std::env::var("DEBUG_ASYNC").is_ok() {
                    eprintln!("Promise.all: all tasks resolved, sending {} results", results.len());
                }

                // All promises resolved successfully
                let _ = tx.send(Ok(Value::Array(Arc::new(results))));
                Value::Null
            });

            // Return promise that resolves to array of results
            Some(Value::Promise {
                receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                task_handle: None,
            })
        }

        "await_all" => {
            // await_all is an alias for Promise.all
            // Redirect to Promise.all handler
            handle(_interp, "Promise.all", args)
        }

        "parallel_map" => {
            // parallel_map(array: Array, func: Function|NativeFunction, concurrency_limit?: Int) -> Promise<Array<Value>>
            // Apply a mapper across array elements and await all results with optional bounded batching.
            if args.len() != 2 && args.len() != 3 {
                return Some(Value::Error(format!(
                    "parallel_map() expects 2 or 3 arguments (array, function, optional concurrency_limit), got {}",
                    args.len()
                )));
            }

            let array = match &args[0] {
                Value::Array(arr) => arr.clone(),
                _ => {
                    return Some(Value::Error(
                        "parallel_map() first argument must be an array".to_string(),
                    ));
                }
            };

            let mapper = match &args[1] {
                func @ Value::NativeFunction(_)
                | func @ Value::Function(_, _, _)
                | func @ Value::BytecodeFunction { .. }
                | func @ Value::GeneratorDef(_, _) => func.clone(),
                _ => {
                    return Some(Value::Error(
                        "parallel_map() second argument must be a callable function".to_string(),
                    ));
                }
            };

            let concurrency_limit = if args.len() == 3 {
                match &args[2] {
                    Value::Int(n) if *n > 0 => Some(*n),
                    Value::Int(n) => {
                        return Some(Value::Error(format!(
                            "parallel_map() concurrency_limit must be > 0, got {}",
                            n
                        )));
                    }
                    _ => {
                        return Some(Value::Error(
                            "parallel_map() optional concurrency_limit must be an integer"
                                .to_string(),
                        ));
                    }
                }
            } else {
                Some(_interp.get_async_task_pool_size() as i64)
            };

            let mut mapped_promises = Vec::with_capacity(array.len());
            for element in array.iter() {
                let mapped = match &mapper {
                    Value::NativeFunction(name) => {
                        _interp.call_native_function_impl(name, &[element.clone()])
                    }
                    _ => _interp.call_user_function(&mapper, &[element.clone()]),
                };

                match mapped {
                    Value::Error(_) | Value::ErrorObject { .. } => {
                        return Some(mapped);
                    }
                    Value::Promise { .. } => mapped_promises.push(mapped),
                    immediate => {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        let _ = tx.send(Ok(immediate));
                        mapped_promises.push(Value::Promise {
                            receiver: std::sync::Arc::new(std::sync::Mutex::new(rx)),
                            is_polled: std::sync::Arc::new(std::sync::Mutex::new(false)),
                            cached_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
                            task_handle: None,
                        });
                    }
                }
            }

            let mut promise_all_args = vec![Value::Array(Arc::new(mapped_promises))];
            if let Some(limit) = concurrency_limit {
                promise_all_args.push(Value::Int(limit));
            }

            handle(_interp, "Promise.all", &promise_all_args)
        }

        "set_task_pool_size" => {
            // set_task_pool_size(size: Int) -> Int
            // Sets default async task pool size used when no explicit concurrency_limit is provided.
            // Returns previous size.
            if args.len() != 1 {
                return Some(Value::Error(format!(
                    "set_task_pool_size() expects 1 argument (size), got {}",
                    args.len()
                )));
            }

            let size = match &args[0] {
                Value::Int(n) if *n > 0 => *n as usize,
                Value::Int(n) => {
                    return Some(Value::Error(format!(
                        "set_task_pool_size() size must be > 0, got {}",
                        n
                    )));
                }
                _ => {
                    return Some(Value::Error(
                        "set_task_pool_size() requires an integer size argument".to_string(),
                    ));
                }
            };

            let previous_size = _interp.set_async_task_pool_size(size);
            Some(Value::Int(previous_size as i64))
        }

        "get_task_pool_size" => {
            // get_task_pool_size() -> Int
            // Returns default async task pool size used by promise_all/await_all/parallel_map
            // when no explicit concurrency_limit is provided.
            if !args.is_empty() {
                return Some(Value::Error(format!(
                    "get_task_pool_size() expects 0 arguments, got {}",
                    args.len()
                )));
            }

            Some(Value::Int(_interp.get_async_task_pool_size() as i64))
        }

        _ => None, // Not handled by this module
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::Interpreter;
    use std::fs;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn string_value(s: &str) -> Value {
        Value::Str(Arc::new(s.to_string()))
    }

    fn await_promise(value: Value) -> Result<Value, String> {
        match value {
            Value::Promise { receiver, .. } => AsyncRuntime::block_on(async {
                let rx = {
                    let mut receiver_guard = receiver.lock().unwrap();
                    let (dummy_tx, dummy_rx) = tokio::sync::oneshot::channel();
                    drop(dummy_tx);
                    std::mem::replace(&mut *receiver_guard, dummy_rx)
                };

                match rx.await {
                    Ok(result) => result,
                    Err(_) => Err("Promise channel closed before resolution".to_string()),
                }
            }),
            _ => panic!("Expected Promise value"),
        }
    }

    fn unique_temp_dir(prefix: &str) -> String {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("{}_{}", prefix, nanos));
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_parallel_map_async_read_file_preserves_order_with_limit() {
        let temp_dir = unique_temp_dir("ruff_parallel_map");
        fs::create_dir_all(&temp_dir).unwrap();

        let file_a = format!("{}/a.txt", temp_dir);
        let file_b = format!("{}/b.txt", temp_dir);
        let file_c = format!("{}/c.txt", temp_dir);

        fs::write(&file_a, "alpha").unwrap();
        fs::write(&file_b, "beta").unwrap();
        fs::write(&file_c, "gamma").unwrap();

        let mut interp = Interpreter::new();
        let args = vec![
            Value::Array(Arc::new(vec![
                string_value(&file_a),
                string_value(&file_b),
                string_value(&file_c),
            ])),
            Value::NativeFunction("async_read_file".to_string()),
            Value::Int(2),
        ];

        let result = handle(&mut interp, "parallel_map", &args).unwrap();
        let resolved = await_promise(result).unwrap();

        match resolved {
            Value::Array(values) => {
                assert_eq!(values.len(), 3);
                match &values[0] {
                    Value::Str(s) => assert_eq!(s.as_str(), "alpha"),
                    _ => panic!("Expected string at index 0"),
                }
                match &values[1] {
                    Value::Str(s) => assert_eq!(s.as_str(), "beta"),
                    _ => panic!("Expected string at index 1"),
                }
                match &values[2] {
                    Value::Str(s) => assert_eq!(s.as_str(), "gamma"),
                    _ => panic!("Expected string at index 2"),
                }
            }
            _ => panic!("Expected Array result"),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parallel_map_wraps_non_promise_results() {
        let mut interp = Interpreter::new();
        let args = vec![
            Value::Array(Arc::new(vec![
                string_value("a"),
                string_value("bc"),
                string_value("def"),
            ])),
            Value::NativeFunction("len".to_string()),
        ];

        let result = handle(&mut interp, "parallel_map", &args).unwrap();
        let resolved = await_promise(result).unwrap();

        match resolved {
            Value::Array(values) => {
                assert_eq!(values.len(), 3);
                match &values[0] {
                    Value::Int(n) => assert_eq!(*n, 1),
                    _ => panic!("Expected Int at index 0"),
                }
                match &values[1] {
                    Value::Int(n) => assert_eq!(*n, 2),
                    _ => panic!("Expected Int at index 1"),
                }
                match &values[2] {
                    Value::Int(n) => assert_eq!(*n, 3),
                    _ => panic!("Expected Int at index 2"),
                }
            }
            _ => panic!("Expected Array result"),
        }
    }

    #[test]
    fn test_parallel_map_rejects_non_callable_mapper() {
        let mut interp = Interpreter::new();
        let args = vec![Value::Array(Arc::new(vec![Value::Int(1)])), Value::Int(123)];

        let result = handle(&mut interp, "parallel_map", &args).unwrap();
        match result {
            Value::Error(msg) => {
                assert!(msg.contains("second argument must be a callable function"));
            }
            _ => panic!("Expected Value::Error for non-callable mapper"),
        }
    }

    #[test]
    fn test_parallel_map_validates_concurrency_limit() {
        let mut interp = Interpreter::new();
        let args = vec![
            Value::Array(Arc::new(vec![Value::Int(1)])),
            Value::NativeFunction("len".to_string()),
            Value::Int(0),
        ];

        let result = handle(&mut interp, "parallel_map", &args).unwrap();
        match result {
            Value::Error(msg) => {
                assert!(msg.contains("concurrency_limit must be > 0"));
            }
            _ => panic!("Expected Value::Error for invalid concurrency limit"),
        }
    }

    #[test]
    fn test_parallel_map_propagates_rejected_promises() {
        let missing_file = unique_temp_dir("ruff_parallel_map_missing");
        let mut interp = Interpreter::new();
        let args = vec![
            Value::Array(Arc::new(vec![string_value(&missing_file)])),
            Value::NativeFunction("async_read_file".to_string()),
        ];

        let result = handle(&mut interp, "parallel_map", &args).unwrap();
        let resolved = await_promise(result);
        match resolved {
            Ok(_) => panic!("Expected Promise rejection for missing file"),
            Err(msg) => assert!(msg.contains("Promise 0 rejected")),
        }
    }

    #[test]
    fn test_set_task_pool_size_round_trip() {
        let mut interp = Interpreter::new();

        let initial = handle(&mut interp, "get_task_pool_size", &[]).unwrap();
        let initial_size = match initial {
            Value::Int(n) => n,
            _ => panic!("Expected Int from get_task_pool_size"),
        };

        let previous = handle(&mut interp, "set_task_pool_size", &[Value::Int(7)]).unwrap();
        match previous {
            Value::Int(n) => assert_eq!(n, initial_size),
            _ => panic!("Expected Int previous size from set_task_pool_size"),
        }

        let current = handle(&mut interp, "get_task_pool_size", &[]).unwrap();
        match current {
            Value::Int(n) => assert_eq!(n, 7),
            _ => panic!("Expected Int from get_task_pool_size"),
        }
    }

    #[test]
    fn test_set_task_pool_size_validates_input() {
        let mut interp = Interpreter::new();

        let non_positive = handle(&mut interp, "set_task_pool_size", &[Value::Int(0)]).unwrap();
        match non_positive {
            Value::Error(msg) => assert!(msg.contains("size must be > 0")),
            _ => panic!("Expected Value::Error for non-positive pool size"),
        }

        let wrong_type =
            handle(&mut interp, "set_task_pool_size", &[Value::Str(Arc::new("2".to_string()))])
                .unwrap();
        match wrong_type {
            Value::Error(msg) => assert!(msg.contains("requires an integer size argument")),
            _ => panic!("Expected Value::Error for non-integer pool size"),
        }
    }

    #[test]
    fn test_get_task_pool_size_rejects_arguments() {
        let mut interp = Interpreter::new();

        let result = handle(&mut interp, "get_task_pool_size", &[Value::Int(1)]).unwrap();
        match result {
            Value::Error(msg) => assert!(msg.contains("expects 0 arguments")),
            _ => panic!("Expected Value::Error for get_task_pool_size arguments"),
        }
    }
}
