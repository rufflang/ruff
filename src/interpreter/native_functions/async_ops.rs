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

        _ => None, // Not handled by this module
    }
}
