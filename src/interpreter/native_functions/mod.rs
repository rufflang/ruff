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
    Value::Int(0)
}
