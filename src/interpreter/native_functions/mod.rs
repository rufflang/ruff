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
            "contains",
            "index_of",
            "io_read_bytes",
            "io_write_bytes",
            "io_append_bytes",
            "io_read_at",
            "io_write_at",
            "io_seek_read",
            "io_file_metadata",
            "io_truncate",
            "io_copy_range",
            "http_get",
            "http_post",
            "http_put",
            "http_delete",
            "http_get_binary",
            "http_get_stream",
            "http_server",
            "http_response",
            "json_response",
            "html_response",
            "redirect_response",
            "set_header",
            "set_headers",
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

    #[test]
    fn test_release_hardening_builtin_dispatch_coverage_for_declared_builtins() {
        let mut interpreter = Interpreter::new();
        let skip_probe_names = ["input", "exit", "sleep", "execute"];
        let mut unknown_builtin_names = Vec::new();
        let expected_known_legacy_dispatch_gaps = vec![
            "db_connect".to_string(),
            "db_execute".to_string(),
            "db_query".to_string(),
            "db_close".to_string(),
            "db_pool".to_string(),
            "db_pool_acquire".to_string(),
            "db_pool_release".to_string(),
            "db_pool_stats".to_string(),
            "db_pool_close".to_string(),
            "db_begin".to_string(),
            "db_commit".to_string(),
            "db_rollback".to_string(),
            "db_last_insert_id".to_string(),
            "Set".to_string(),
            "load_image".to_string(),
            "zip_create".to_string(),
            "zip_add_file".to_string(),
            "zip_add_dir".to_string(),
            "zip_close".to_string(),
            "unzip".to_string(),
            "sha256".to_string(),
            "md5".to_string(),
            "md5_file".to_string(),
            "hash_password".to_string(),
            "verify_password".to_string(),
            "aes_encrypt".to_string(),
            "aes_decrypt".to_string(),
            "aes_encrypt_bytes".to_string(),
            "aes_decrypt_bytes".to_string(),
            "rsa_generate_keypair".to_string(),
            "rsa_encrypt".to_string(),
            "rsa_decrypt".to_string(),
            "rsa_sign".to_string(),
            "rsa_verify".to_string(),
            "spawn_process".to_string(),
            "pipe_commands".to_string(),
            "tcp_listen".to_string(),
            "tcp_accept".to_string(),
            "tcp_connect".to_string(),
            "tcp_send".to_string(),
            "tcp_receive".to_string(),
            "tcp_close".to_string(),
            "tcp_set_nonblocking".to_string(),
            "udp_bind".to_string(),
            "udp_send_to".to_string(),
            "udp_receive_from".to_string(),
            "udp_close".to_string(),
        ];

        for builtin_name in Interpreter::get_builtin_names() {
            if skip_probe_names.contains(&builtin_name) {
                continue;
            }

            let result = call_native_function(&mut interpreter, builtin_name, &[]);
            if is_unknown_native_error(&result) {
                unknown_builtin_names.push(builtin_name.to_string());
            }
        }

        assert_eq!(
            unknown_builtin_names,
            expected_known_legacy_dispatch_gaps,
            "Declared builtin dispatch drift changed. If a gap was fixed, remove it from expected list; if a new gap appeared, investigate and either fix dispatch or explicitly acknowledge it here."
        );
    }

    #[test]
    fn test_release_hardening_contains_index_of_argument_and_polymorphic_contracts() {
        let mut interpreter = Interpreter::new();

        let contains_missing_args = call_native_function(&mut interpreter, "contains", &[]);
        assert!(
            matches!(contains_missing_args, Value::Error(message) if message.contains("contains() requires two arguments"))
        );

        let index_of_missing_args = call_native_function(&mut interpreter, "index_of", &[]);
        assert!(
            matches!(index_of_missing_args, Value::Error(message) if message.contains("index_of() requires two arguments"))
        );

        let contains_array = call_native_function(
            &mut interpreter,
            "contains",
            &[Value::Array(std::sync::Arc::new(vec![Value::Int(1), Value::Int(2)])), Value::Int(2)],
        );
        assert!(matches!(contains_array, Value::Bool(true)));

        let index_of_array = call_native_function(
            &mut interpreter,
            "index_of",
            &[
                Value::Array(std::sync::Arc::new(vec![Value::Int(10), Value::Int(20)])),
                Value::Int(20),
            ],
        );
        assert!(matches!(index_of_array, Value::Int(1)));
    }

    #[test]
    fn test_release_hardening_io_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let read_bytes_missing = call_native_function(&mut interpreter, "io_read_bytes", &[]);
        assert!(
            matches!(read_bytes_missing, Value::Error(message) if message.contains("requires two arguments: path and count"))
        );

        let read_at_missing = call_native_function(&mut interpreter, "io_read_at", &[]);
        assert!(
            matches!(read_at_missing, Value::Error(message) if message.contains("requires three arguments: path, offset, and count"))
        );

        let copy_range_missing = call_native_function(&mut interpreter, "io_copy_range", &[]);
        assert!(
            matches!(copy_range_missing, Value::Error(message) if message.contains("requires four arguments: source, dest, offset, and count"))
        );
    }

    #[test]
    fn test_release_hardening_http_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let get_missing = call_native_function(&mut interpreter, "http_get", &[]);
        assert!(
            matches!(get_missing, Value::Error(message) if message.contains("http_get requires a URL string"))
        );

        let post_missing = call_native_function(&mut interpreter, "http_post", &[]);
        assert!(
            matches!(post_missing, Value::Error(message) if message.contains("http_post requires URL and JSON body strings"))
        );

        let response_shape = call_native_function(
            &mut interpreter,
            "http_response",
            &[Value::Int(200), Value::Str(std::sync::Arc::new("ok".to_string()))],
        );
        assert!(matches!(response_shape, Value::HttpResponse { status, .. } if status == 200));
    }
}
