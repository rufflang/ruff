// File: src/interpreter/native_functions/mod.rs
//
// Module organization for native (built-in) function implementations.
// This module is part of Phase 3 modularization to split the massive
// call_native_function_impl into manageable category-based modules.

pub mod async_ops;
pub mod collections;
pub mod concurrency;
pub mod crypto;
pub mod database;
pub mod filesystem;
pub mod http;
pub mod io;
pub mod json;
pub mod math;
pub mod network;
pub mod strings;
pub mod system;
pub mod type_ops;

use super::{Interpreter, Value};

/// Main dispatcher that routes native function calls to appropriate category modules
pub fn call_native_function(interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Value {
    // Try async operations first (high priority for async functions)
    if let Some(result) = async_ops::handle(interp, name, arg_values) {
        return result;
    }
    // Try each category in order
    if let Some(result) = io::handle(interp, name, arg_values) {
        return result;
    }
    if let Some(result) = math::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = strings::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = collections::handle(interp, name, arg_values) {
        return result;
    }
    if let Some(result) = type_ops::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = filesystem::handle(interp, name, arg_values) {
        return result;
    }
    if let Some(result) = http::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = json::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = crypto::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = system::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = concurrency::handle(interp, name, arg_values) {
        return result;
    }
    if let Some(result) = database::handle(name, arg_values) {
        return result;
    }
    if let Some(result) = network::handle(interp, name, arg_values) {
        return result;
    }

    // Unknown function
    Value::Error(format!("Unknown native function: {}", name))
}

#[cfg(test)]
mod tests {
    use super::call_native_function;
    use crate::interpreter::{Interpreter, Value};

    fn is_unknown_native_error(value: &Value) -> bool {
        if let Value::Error(message) = value {
            message.starts_with("Unknown native function:")
        } else {
            false
        }
    }

    #[test]
    fn test_unknown_native_function_returns_explicit_error() {
        let mut interpreter = Interpreter::new();
        let result = call_native_function(&mut interpreter, "__unknown_native_test__", &[]);

        match result {
            Value::Error(message) => {
                assert_eq!(message, "Unknown native function: __unknown_native_test__");
            }
            other => panic!("Expected Value::Error, got {:?}", other),
        }
    }

    #[test]
    fn test_release_hardening_builtin_dispatch_coverage_for_recent_apis() {
        let mut interpreter = Interpreter::new();
        let critical_builtin_names = [
            "ssg_render_pages",
            "join_path",
            "path_join",
            "queue_size",
            "stack_size",
            "shared_set",
            "shared_get",
            "shared_has",
            "shared_delete",
            "shared_add_int",
            "async_read_files",
            "async_write_files",
            "promise_all",
            "await_all",
            "par_each",
            "set_task_pool_size",
            "get_task_pool_size",
        ];

        for builtin_name in critical_builtin_names {
            let result = call_native_function(&mut interpreter, builtin_name, &[]);
            assert!(
                !is_unknown_native_error(&result),
                "Builtin '{}' unexpectedly hit unknown-native fallback with result {:?}",
                builtin_name,
                result
            );
        }
    }
}
