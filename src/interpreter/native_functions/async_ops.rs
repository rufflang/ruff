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

        _ => None, // Not handled by this module
    }
}
