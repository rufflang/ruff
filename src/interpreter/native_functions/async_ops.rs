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
