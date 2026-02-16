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
    use std::sync::Arc;

    fn available_tcp_port() -> i64 {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("ephemeral tcp listener should bind");
        listener.local_addr().expect("ephemeral tcp listener should have local addr").port() as i64
    }

    fn available_udp_port() -> i64 {
        let socket =
            std::net::UdpSocket::bind("127.0.0.1:0").expect("ephemeral udp socket should bind");
        socket.local_addr().expect("ephemeral udp socket should have local addr").port() as i64
    }

    fn tmp_test_path(file_name: &str) -> String {
        let mut path = std::env::current_dir().expect("current_dir should resolve");
        path.push("tmp");
        path.push("native_zip_dispatch_tests");
        std::fs::create_dir_all(&path).expect("zip test tmp dir should be created");
        path.push(file_name);
        path.to_string_lossy().to_string()
    }

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
            "len",
            "type",
            "is_int",
            "is_float",
            "is_string",
            "is_bool",
            "is_array",
            "is_dict",
            "is_null",
            "is_function",
            "parse_int",
            "parse_float",
            "to_int",
            "to_float",
            "to_string",
            "to_bool",
            "bytes",
            "Set",
            "ssg_render_pages",
            "upper",
            "lower",
            "replace",
            "append",
            "starts_with",
            "ends_with",
            "repeat",
            "char_at",
            "is_empty",
            "count_chars",
            "substring",
            "capitalize",
            "trim",
            "trim_start",
            "trim_end",
            "split",
            "join",
            "Promise.all",
            "parallel_map",
            "par_map",
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
            "parallel_http",
            "jwt_encode",
            "jwt_decode",
            "oauth2_auth_url",
            "oauth2_get_token",
            "http_get_stream",
            "http_server",
            "http_response",
            "json_response",
            "html_response",
            "redirect_response",
            "set_header",
            "set_headers",
            "db_connect",
            "db_execute",
            "db_query",
            "db_close",
            "db_pool",
            "db_pool_acquire",
            "db_pool_release",
            "db_pool_stats",
            "db_pool_close",
            "db_begin",
            "db_commit",
            "db_rollback",
            "db_last_insert_id",
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
            "spawn_process",
            "pipe_commands",
            "load_image",
            "zip_create",
            "zip_add_file",
            "zip_add_dir",
            "zip_close",
            "unzip",
            "tcp_listen",
            "tcp_accept",
            "tcp_connect",
            "tcp_send",
            "tcp_receive",
            "tcp_close",
            "tcp_set_nonblocking",
            "udp_bind",
            "udp_send_to",
            "udp_receive_from",
            "udp_close",
            "sha256",
            "md5",
            "md5_file",
            "hash_password",
            "verify_password",
            "aes_encrypt",
            "aes_decrypt",
            "aes_encrypt_bytes",
            "aes_decrypt_bytes",
            "rsa_generate_keypair",
            "rsa_encrypt",
            "rsa_decrypt",
            "rsa_sign",
            "rsa_verify",
            "random",
            "random_int",
            "random_choice",
            "set_random_seed",
            "clear_random_seed",
            "now",
            "current_timestamp",
            "performance_now",
            "time_us",
            "time_ns",
            "format_duration",
            "elapsed",
            "format_date",
            "parse_date",
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
        let expected_known_legacy_dispatch_gaps: Vec<String> = vec![];

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
    fn test_release_hardening_len_polymorphic_contracts() {
        let mut interpreter = Interpreter::new();

        let string_len = call_native_function(
            &mut interpreter,
            "len",
            &[Value::Str(Arc::new("ruff".to_string()))],
        );
        assert!(matches!(string_len, Value::Int(4)));

        let array_len = call_native_function(
            &mut interpreter,
            "len",
            &[Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2)]))],
        );
        assert!(matches!(array_len, Value::Int(2)));

        let mut dict = crate::interpreter::DictMap::default();
        dict.insert("a".into(), Value::Int(1));
        dict.insert("b".into(), Value::Int(2));
        let dict_len =
            call_native_function(&mut interpreter, "len", &[Value::Dict(Arc::new(dict))]);
        assert!(matches!(dict_len, Value::Int(2)));

        let bytes_len =
            call_native_function(&mut interpreter, "len", &[Value::Bytes(vec![1, 2, 3])]);
        assert!(matches!(bytes_len, Value::Int(3)));

        let set_len = call_native_function(
            &mut interpreter,
            "len",
            &[Value::Set(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
        );
        assert!(matches!(set_len, Value::Int(3)));

        let queue_len = call_native_function(
            &mut interpreter,
            "len",
            &[Value::Queue(std::collections::VecDeque::from(vec![Value::Int(1), Value::Int(2)]))],
        );
        assert!(matches!(queue_len, Value::Int(2)));

        let stack_len = call_native_function(
            &mut interpreter,
            "len",
            &[Value::Stack(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])],
        );
        assert!(matches!(stack_len, Value::Int(4)));

        let fallback_len = call_native_function(&mut interpreter, "len", &[Value::Null]);
        assert!(matches!(fallback_len, Value::Int(0)));

        let missing_args_len = call_native_function(&mut interpreter, "len", &[]);
        assert!(matches!(missing_args_len, Value::Int(0)));
    }

    #[test]
    fn test_release_hardening_type_and_is_introspection_contracts() {
        let mut interpreter = Interpreter::new();

        let type_string = call_native_function(
            &mut interpreter,
            "type",
            &[Value::Str(Arc::new("ruff".to_string()))],
        );
        assert!(matches!(type_string, Value::Str(name) if name.as_ref() == "string"));

        let type_array = call_native_function(
            &mut interpreter,
            "type",
            &[Value::Array(Arc::new(vec![Value::Int(1)]))],
        );
        assert!(matches!(type_array, Value::Str(name) if name.as_ref() == "array"));

        let type_null = call_native_function(&mut interpreter, "type", &[Value::Null]);
        assert!(matches!(type_null, Value::Str(name) if name.as_ref() == "null"));

        let type_missing = call_native_function(&mut interpreter, "type", &[]);
        assert!(
            matches!(type_missing, Value::Error(message) if message.contains("type() requires one argument"))
        );

        let is_int_true = call_native_function(&mut interpreter, "is_int", &[Value::Int(7)]);
        assert!(matches!(is_int_true, Value::Bool(true)));
        let is_int_false = call_native_function(&mut interpreter, "is_int", &[Value::Float(7.0)]);
        assert!(matches!(is_int_false, Value::Bool(false)));
        let is_int_missing = call_native_function(&mut interpreter, "is_int", &[]);
        assert!(matches!(is_int_missing, Value::Bool(false)));

        let is_float_true =
            call_native_function(&mut interpreter, "is_float", &[Value::Float(7.5)]);
        assert!(matches!(is_float_true, Value::Bool(true)));
        let is_float_false = call_native_function(&mut interpreter, "is_float", &[Value::Int(7)]);
        assert!(matches!(is_float_false, Value::Bool(false)));

        let is_string_true = call_native_function(
            &mut interpreter,
            "is_string",
            &[Value::Str(Arc::new("x".to_string()))],
        );
        assert!(matches!(is_string_true, Value::Bool(true)));
        let is_string_false =
            call_native_function(&mut interpreter, "is_string", &[Value::Bool(true)]);
        assert!(matches!(is_string_false, Value::Bool(false)));

        let is_bool_true = call_native_function(&mut interpreter, "is_bool", &[Value::Bool(true)]);
        assert!(matches!(is_bool_true, Value::Bool(true)));
        let is_bool_false = call_native_function(
            &mut interpreter,
            "is_bool",
            &[Value::Str(Arc::new("true".to_string()))],
        );
        assert!(matches!(is_bool_false, Value::Bool(false)));

        let is_array_true =
            call_native_function(&mut interpreter, "is_array", &[Value::Array(Arc::new(vec![]))]);
        assert!(matches!(is_array_true, Value::Bool(true)));
        let is_array_false = call_native_function(&mut interpreter, "is_array", &[Value::Null]);
        assert!(matches!(is_array_false, Value::Bool(false)));

        let mut dict = crate::interpreter::DictMap::default();
        dict.insert("k".into(), Value::Int(1));
        let is_dict_true =
            call_native_function(&mut interpreter, "is_dict", &[Value::Dict(Arc::new(dict))]);
        assert!(matches!(is_dict_true, Value::Bool(true)));
        let is_dict_false =
            call_native_function(&mut interpreter, "is_dict", &[Value::Array(Arc::new(vec![]))]);
        assert!(matches!(is_dict_false, Value::Bool(false)));

        let is_null_true = call_native_function(&mut interpreter, "is_null", &[Value::Null]);
        assert!(matches!(is_null_true, Value::Bool(true)));
        let is_null_false = call_native_function(&mut interpreter, "is_null", &[Value::Int(0)]);
        assert!(matches!(is_null_false, Value::Bool(false)));

        let is_function_true = call_native_function(
            &mut interpreter,
            "is_function",
            &[Value::NativeFunction("len".to_string())],
        );
        assert!(matches!(is_function_true, Value::Bool(true)));
        let is_function_false =
            call_native_function(&mut interpreter, "is_function", &[Value::Int(1)]);
        assert!(matches!(is_function_false, Value::Bool(false)));
        let is_function_missing = call_native_function(&mut interpreter, "is_function", &[]);
        assert!(matches!(is_function_missing, Value::Bool(false)));
    }

    #[test]
    fn test_release_hardening_conversion_and_bytes_contracts() {
        let mut interpreter = Interpreter::new();

        let parse_int_ok = call_native_function(
            &mut interpreter,
            "parse_int",
            &[Value::Str(Arc::new("42".to_string()))],
        );
        assert!(matches!(parse_int_ok, Value::Int(42)));

        let parse_int_bad = call_native_function(
            &mut interpreter,
            "parse_int",
            &[Value::Str(Arc::new("abc".to_string()))],
        );
        assert!(
            matches!(parse_int_bad, Value::Error(message) if message.contains("Cannot parse 'abc' as integer"))
        );

        let parse_float_ok = call_native_function(
            &mut interpreter,
            "parse_float",
            &[Value::Str(Arc::new("3.25".to_string()))],
        );
        assert!(matches!(parse_float_ok, Value::Float(value) if (value - 3.25).abs() < 1e-12));

        let parse_float_bad = call_native_function(
            &mut interpreter,
            "parse_float",
            &[Value::Str(Arc::new("not-float".to_string()))],
        );
        assert!(
            matches!(parse_float_bad, Value::Error(message) if message.contains("Cannot parse 'not-float' as float"))
        );

        let to_int_ok = call_native_function(
            &mut interpreter,
            "to_int",
            &[Value::Str(Arc::new("7".to_string()))],
        );
        assert!(matches!(to_int_ok, Value::Int(7)));

        let to_int_bad = call_native_function(
            &mut interpreter,
            "to_int",
            &[Value::Str(Arc::new("bad".to_string()))],
        );
        assert!(
            matches!(to_int_bad, Value::Error(message) if message.contains("Cannot convert 'bad' to int"))
        );

        let to_float_ok = call_native_function(
            &mut interpreter,
            "to_float",
            &[Value::Str(Arc::new("2.5".to_string()))],
        );
        assert!(matches!(to_float_ok, Value::Float(value) if (value - 2.5).abs() < 1e-12));

        let to_float_bad = call_native_function(
            &mut interpreter,
            "to_float",
            &[Value::Str(Arc::new("bad".to_string()))],
        );
        assert!(
            matches!(to_float_bad, Value::Error(message) if message.contains("Cannot convert 'bad' to float"))
        );

        let to_string_ok = call_native_function(&mut interpreter, "to_string", &[Value::Int(99)]);
        assert!(matches!(to_string_ok, Value::Str(value) if value.as_ref() == "99"));

        let to_bool_false = call_native_function(
            &mut interpreter,
            "to_bool",
            &[Value::Str(Arc::new("false".to_string()))],
        );
        assert!(matches!(to_bool_false, Value::Bool(false)));

        let to_bool_true = call_native_function(
            &mut interpreter,
            "to_bool",
            &[Value::Str(Arc::new("yes".to_string()))],
        );
        assert!(matches!(to_bool_true, Value::Bool(true)));

        let bytes_ok = call_native_function(
            &mut interpreter,
            "bytes",
            &[Value::Array(Arc::new(vec![Value::Int(65), Value::Int(66), Value::Int(67)]))],
        );
        assert!(matches!(bytes_ok, Value::Bytes(values) if values == vec![65, 66, 67]));

        let bytes_bad_range = call_native_function(
            &mut interpreter,
            "bytes",
            &[Value::Array(Arc::new(vec![Value::Int(256)]))],
        );
        assert!(
            matches!(bytes_bad_range, Value::Error(message) if message.contains("requires integers in range 0-255"))
        );

        let bytes_bad_shape = call_native_function(
            &mut interpreter,
            "bytes",
            &[Value::Array(Arc::new(vec![Value::Str(Arc::new("x".to_string()))]))],
        );
        assert!(
            matches!(bytes_bad_shape, Value::Error(message) if message.contains("requires an array of integers"))
        );
    }

    #[test]
    fn test_release_hardening_core_alias_behavior_parity_contracts() {
        let mut interpreter = Interpreter::new();

        let to_upper = call_native_function(
            &mut interpreter,
            "to_upper",
            &[Value::Str(Arc::new("Ruff Lang".to_string()))],
        );
        let upper = call_native_function(
            &mut interpreter,
            "upper",
            &[Value::Str(Arc::new("Ruff Lang".to_string()))],
        );
        assert!(matches!(to_upper, Value::Str(s) if s.as_ref() == "RUFF LANG"));
        assert!(matches!(upper, Value::Str(s) if s.as_ref() == "RUFF LANG"));

        let to_lower = call_native_function(
            &mut interpreter,
            "to_lower",
            &[Value::Str(Arc::new("Ruff Lang".to_string()))],
        );
        let lower = call_native_function(
            &mut interpreter,
            "lower",
            &[Value::Str(Arc::new("Ruff Lang".to_string()))],
        );
        assert!(matches!(to_lower, Value::Str(s) if s.as_ref() == "ruff lang"));
        assert!(matches!(lower, Value::Str(s) if s.as_ref() == "ruff lang"));

        let replace_str = call_native_function(
            &mut interpreter,
            "replace_str",
            &[
                Value::Str(Arc::new("ruff-lang-2026".to_string())),
                Value::Str(Arc::new("-".to_string())),
                Value::Str(Arc::new("_".to_string())),
            ],
        );
        let replace = call_native_function(
            &mut interpreter,
            "replace",
            &[
                Value::Str(Arc::new("ruff-lang-2026".to_string())),
                Value::Str(Arc::new("-".to_string())),
                Value::Str(Arc::new("_".to_string())),
            ],
        );
        assert!(matches!(replace_str, Value::Str(s) if s.as_ref() == "ruff_lang_2026"));
        assert!(matches!(replace, Value::Str(s) if s.as_ref() == "ruff_lang_2026"));

        let push = call_native_function(
            &mut interpreter,
            "push",
            &[Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2)])), Value::Int(3)],
        );
        let append = call_native_function(
            &mut interpreter,
            "append",
            &[Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2)])), Value::Int(3)],
        );
        assert!(
            matches!(push, Value::Array(values) if values.len() == 3 && matches!(&values[0], Value::Int(1)) && matches!(&values[1], Value::Int(2)) && matches!(&values[2], Value::Int(3)))
        );
        assert!(
            matches!(append, Value::Array(values) if values.len() == 3 && matches!(&values[0], Value::Int(1)) && matches!(&values[1], Value::Int(2)) && matches!(&values[2], Value::Int(3)))
        );
    }

    #[test]
    fn test_release_hardening_string_utility_behavior_and_fallback_contracts() {
        let mut interpreter = Interpreter::new();

        let starts_with_true = call_native_function(
            &mut interpreter,
            "starts_with",
            &[
                Value::Str(Arc::new("ruff-lang".to_string())),
                Value::Str(Arc::new("ruff".to_string())),
            ],
        );
        assert!(matches!(starts_with_true, Value::Bool(true)));

        let starts_with_false = call_native_function(
            &mut interpreter,
            "starts_with",
            &[
                Value::Str(Arc::new("ruff-lang".to_string())),
                Value::Str(Arc::new("lang".to_string())),
            ],
        );
        assert!(matches!(starts_with_false, Value::Bool(false)));

        let starts_with_invalid = call_native_function(
            &mut interpreter,
            "starts_with",
            &[Value::Str(Arc::new("ruff-lang".to_string())), Value::Int(1)],
        );
        assert!(matches!(starts_with_invalid, Value::Bool(false)));

        let ends_with_true = call_native_function(
            &mut interpreter,
            "ends_with",
            &[
                Value::Str(Arc::new("ruff-lang".to_string())),
                Value::Str(Arc::new("lang".to_string())),
            ],
        );
        assert!(matches!(ends_with_true, Value::Bool(true)));

        let ends_with_false = call_native_function(
            &mut interpreter,
            "ends_with",
            &[
                Value::Str(Arc::new("ruff-lang".to_string())),
                Value::Str(Arc::new("ruff".to_string())),
            ],
        );
        assert!(matches!(ends_with_false, Value::Bool(false)));

        let ends_with_invalid = call_native_function(
            &mut interpreter,
            "ends_with",
            &[Value::Str(Arc::new("ruff-lang".to_string())), Value::Int(1)],
        );
        assert!(matches!(ends_with_invalid, Value::Bool(false)));

        let repeat_ok = call_native_function(
            &mut interpreter,
            "repeat",
            &[Value::Str(Arc::new("ru".to_string())), Value::Int(3)],
        );
        assert!(matches!(repeat_ok, Value::Str(value) if value.as_ref() == "rururu"));

        let repeat_missing = call_native_function(
            &mut interpreter,
            "repeat",
            &[Value::Str(Arc::new("ru".to_string()))],
        );
        assert!(matches!(repeat_missing, Value::Str(value) if value.as_ref().is_empty()));

        let char_at_ok = call_native_function(
            &mut interpreter,
            "char_at",
            &[Value::Str(Arc::new("ruff".to_string())), Value::Int(2)],
        );
        assert!(matches!(char_at_ok, Value::Str(value) if value.as_ref() == "f"));

        let char_at_oob = call_native_function(
            &mut interpreter,
            "char_at",
            &[Value::Str(Arc::new("ruff".to_string())), Value::Int(99)],
        );
        assert!(matches!(char_at_oob, Value::Str(value) if value.as_ref().is_empty()));

        let char_at_missing = call_native_function(
            &mut interpreter,
            "char_at",
            &[Value::Str(Arc::new("ruff".to_string()))],
        );
        assert!(matches!(char_at_missing, Value::Str(value) if value.as_ref().is_empty()));

        let is_empty_true = call_native_function(
            &mut interpreter,
            "is_empty",
            &[Value::Str(Arc::new(String::new()))],
        );
        assert!(matches!(is_empty_true, Value::Bool(true)));

        let is_empty_false = call_native_function(
            &mut interpreter,
            "is_empty",
            &[Value::Str(Arc::new("ruff".to_string()))],
        );
        assert!(matches!(is_empty_false, Value::Bool(false)));

        let is_empty_missing = call_native_function(&mut interpreter, "is_empty", &[]);
        assert!(matches!(is_empty_missing, Value::Bool(true)));

        let count_chars_ok = call_native_function(
            &mut interpreter,
            "count_chars",
            &[Value::Str(Arc::new("ruff".to_string()))],
        );
        assert!(matches!(count_chars_ok, Value::Int(4)));

        let count_chars_unicode = call_native_function(
            &mut interpreter,
            "count_chars",
            &[Value::Str(Arc::new("🔥a".to_string()))],
        );
        assert!(matches!(count_chars_unicode, Value::Int(2)));

        let count_chars_missing = call_native_function(&mut interpreter, "count_chars", &[]);
        assert!(matches!(count_chars_missing, Value::Int(0)));
    }

    #[test]
    fn test_release_hardening_string_transform_and_tokenization_contracts() {
        let mut interpreter = Interpreter::new();

        let substring_ok = call_native_function(
            &mut interpreter,
            "substring",
            &[
                Value::Str(Arc::new("ruff-language".to_string())),
                Value::Int(0),
                Value::Int(4),
            ],
        );
        assert!(matches!(substring_ok, Value::Str(value) if value.as_ref() == "ruff"));

        let substring_missing = call_native_function(
            &mut interpreter,
            "substring",
            &[Value::Str(Arc::new("ruff-language".to_string()))],
        );
        assert!(matches!(substring_missing, Value::Str(value) if value.as_ref().is_empty()));

        let capitalize_ok = call_native_function(
            &mut interpreter,
            "capitalize",
            &[Value::Str(Arc::new("ruff language".to_string()))],
        );
        assert!(matches!(capitalize_ok, Value::Str(value) if value.as_ref() == "Ruff language"));

        let capitalize_missing = call_native_function(&mut interpreter, "capitalize", &[]);
        assert!(matches!(capitalize_missing, Value::Str(value) if value.as_ref().is_empty()));

        let trim_ok = call_native_function(
            &mut interpreter,
            "trim",
            &[Value::Str(Arc::new("  ruff  ".to_string()))],
        );
        assert!(matches!(trim_ok, Value::Str(value) if value.as_ref() == "ruff"));

        let trim_start_ok = call_native_function(
            &mut interpreter,
            "trim_start",
            &[Value::Str(Arc::new("  ruff".to_string()))],
        );
        assert!(matches!(trim_start_ok, Value::Str(value) if value.as_ref() == "ruff"));

        let trim_end_ok = call_native_function(
            &mut interpreter,
            "trim_end",
            &[Value::Str(Arc::new("ruff  ".to_string()))],
        );
        assert!(matches!(trim_end_ok, Value::Str(value) if value.as_ref() == "ruff"));

        let trim_missing = call_native_function(&mut interpreter, "trim", &[]);
        assert!(matches!(trim_missing, Value::Str(value) if value.as_ref().is_empty()));

        let split_ok = call_native_function(
            &mut interpreter,
            "split",
            &[
                Value::Str(Arc::new("a,b,c".to_string())),
                Value::Str(Arc::new(",".to_string())),
            ],
        );
        assert!(matches!(split_ok, Value::Array(parts) if parts.len() == 3
            && matches!(&parts[0], Value::Str(s) if s.as_ref() == "a")
            && matches!(&parts[1], Value::Str(s) if s.as_ref() == "b")
            && matches!(&parts[2], Value::Str(s) if s.as_ref() == "c")));

        let split_missing = call_native_function(
            &mut interpreter,
            "split",
            &[Value::Str(Arc::new("a,b,c".to_string()))],
        );
        assert!(matches!(split_missing, Value::Array(parts) if parts.is_empty()));

        let join_ok = call_native_function(
            &mut interpreter,
            "join",
            &[
                Value::Array(Arc::new(vec![
                    Value::Str(Arc::new("ruff".to_string())),
                    Value::Int(2026),
                    Value::Bool(true),
                ])),
                Value::Str(Arc::new("-".to_string())),
            ],
        );
        assert!(matches!(join_ok, Value::Str(value) if value.as_ref() == "ruff-2026-true"));

        let join_missing = call_native_function(
            &mut interpreter,
            "join",
            &[Value::Array(Arc::new(vec![Value::Str(Arc::new("ruff".to_string()))]))],
        );
        assert!(matches!(join_missing, Value::Str(value) if value.as_ref().is_empty()));
    }

    #[test]
    fn test_release_hardening_system_random_and_time_contracts() {
        let mut interpreter = Interpreter::new();

        let random_value = call_native_function(&mut interpreter, "random", &[]);
        assert!(matches!(random_value, Value::Float(value) if (0.0..=1.0).contains(&value)));

        let random_int_missing = call_native_function(&mut interpreter, "random_int", &[]);
        assert!(matches!(random_int_missing, Value::Error(message) if message.contains("random_int requires two number arguments")));

        let random_int_value = call_native_function(
            &mut interpreter,
            "random_int",
            &[Value::Int(3), Value::Int(7)],
        );
        assert!(matches!(random_int_value, Value::Int(value) if (3..=7).contains(&value)));

        let random_choice_missing = call_native_function(&mut interpreter, "random_choice", &[]);
        assert!(matches!(random_choice_missing, Value::Error(message) if message.contains("random_choice requires an array argument")));

        let random_choice_value = call_native_function(
            &mut interpreter,
            "random_choice",
            &[Value::Array(Arc::new(vec![Value::Int(10), Value::Int(20), Value::Int(30)]))],
        );
        assert!(matches!(random_choice_value, Value::Int(10) | Value::Int(20) | Value::Int(30)));

        let set_seed_missing = call_native_function(&mut interpreter, "set_random_seed", &[]);
        assert!(matches!(set_seed_missing, Value::Error(message) if message.contains("set_random_seed requires a number argument")));

        let clear_seed_result = call_native_function(&mut interpreter, "clear_random_seed", &[]);
        assert!(matches!(clear_seed_result, Value::Null));

        let seed_result = call_native_function(&mut interpreter, "set_random_seed", &[Value::Int(12345)]);
        assert!(matches!(seed_result, Value::Null));

        let seeded_random_a = call_native_function(&mut interpreter, "random", &[]);
        let seeded_random_int_a = call_native_function(
            &mut interpreter,
            "random_int",
            &[Value::Int(1), Value::Int(100)],
        );
        let seeded_choice_a = call_native_function(
            &mut interpreter,
            "random_choice",
            &[Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))],
        );

        let reseed_result =
            call_native_function(&mut interpreter, "set_random_seed", &[Value::Int(12345)]);
        assert!(matches!(reseed_result, Value::Null));

        let seeded_random_b = call_native_function(&mut interpreter, "random", &[]);
        let seeded_random_int_b = call_native_function(
            &mut interpreter,
            "random_int",
            &[Value::Int(1), Value::Int(100)],
        );
        let seeded_choice_b = call_native_function(
            &mut interpreter,
            "random_choice",
            &[Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))],
        );

        match (seeded_random_a, seeded_random_b) {
            (Value::Float(a), Value::Float(b)) => assert!((a - b).abs() < f64::EPSILON),
            (left, right) => panic!("Expected seeded random floats, got {:?} and {:?}", left, right),
        }
        match (seeded_random_int_a, seeded_random_int_b) {
            (Value::Int(a), Value::Int(b)) => assert_eq!(a, b),
            (left, right) => {
                panic!("Expected seeded random_int ints, got {:?} and {:?}", left, right)
            }
        }
        match (seeded_choice_a, seeded_choice_b) {
            (Value::Int(a), Value::Int(b)) => assert_eq!(a, b),
            (left, right) => {
                panic!("Expected seeded random_choice ints, got {:?} and {:?}", left, right)
            }
        }

        let now_value = call_native_function(&mut interpreter, "now", &[]);
        assert!(matches!(now_value, Value::Float(value) if value > 0.0));

        let current_timestamp_value = call_native_function(&mut interpreter, "current_timestamp", &[]);
        assert!(matches!(current_timestamp_value, Value::Int(value) if value > 0));

        let performance_now_start = call_native_function(&mut interpreter, "performance_now", &[]);
        let performance_now_end = call_native_function(&mut interpreter, "performance_now", &[]);
        match (performance_now_start, performance_now_end) {
            (Value::Float(start), Value::Float(end)) => assert!(end >= start),
            (left, right) => panic!("Expected performance_now floats, got {:?} and {:?}", left, right),
        }

        let time_us_start = call_native_function(&mut interpreter, "time_us", &[]);
        let time_us_end = call_native_function(&mut interpreter, "time_us", &[]);
        match (time_us_start, time_us_end) {
            (Value::Float(start), Value::Float(end)) => assert!(end >= start),
            (left, right) => panic!("Expected time_us floats, got {:?} and {:?}", left, right),
        }

        let time_ns_start = call_native_function(&mut interpreter, "time_ns", &[]);
        let time_ns_end = call_native_function(&mut interpreter, "time_ns", &[]);
        match (time_ns_start, time_ns_end) {
            (Value::Float(start), Value::Float(end)) => assert!(end >= start),
            (left, right) => panic!("Expected time_ns floats, got {:?} and {:?}", left, right),
        }

        let format_duration_seconds =
            call_native_function(&mut interpreter, "format_duration", &[Value::Float(1500.0)]);
        assert!(matches!(format_duration_seconds, Value::Str(value) if value.as_ref() == "1.50s"));

        let format_duration_ms =
            call_native_function(&mut interpreter, "format_duration", &[Value::Float(2.5)]);
        assert!(matches!(format_duration_ms, Value::Str(value) if value.as_ref() == "2.50ms"));

        let format_duration_missing = call_native_function(&mut interpreter, "format_duration", &[]);
        assert!(matches!(format_duration_missing, Value::Error(message) if message.contains("format_duration requires a number argument")));

        let elapsed_ok = call_native_function(
            &mut interpreter,
            "elapsed",
            &[Value::Float(10.0), Value::Float(15.5)],
        );
        assert!(matches!(elapsed_ok, Value::Float(value) if (value - 5.5).abs() < 1e-12));

        let elapsed_missing = call_native_function(&mut interpreter, "elapsed", &[]);
        assert!(matches!(elapsed_missing, Value::Error(message) if message.contains("elapsed requires two number arguments")));

        let format_date_epoch = call_native_function(
            &mut interpreter,
            "format_date",
            &[
                Value::Float(0.0),
                Value::Str(Arc::new("YYYY-MM-DD".to_string())),
            ],
        );
        assert!(matches!(format_date_epoch, Value::Str(value) if value.as_ref() == "1970-01-01"));

        let format_date_missing = call_native_function(&mut interpreter, "format_date", &[]);
        assert!(matches!(format_date_missing, Value::Error(message) if message.contains("format_date requires timestamp")));

        let parse_date_epoch = call_native_function(
            &mut interpreter,
            "parse_date",
            &[
                Value::Str(Arc::new("1970-01-01".to_string())),
                Value::Str(Arc::new("YYYY-MM-DD".to_string())),
            ],
        );
        assert!(matches!(parse_date_epoch, Value::Float(value) if value == 0.0));

        let parse_date_invalid = call_native_function(
            &mut interpreter,
            "parse_date",
            &[
                Value::Str(Arc::new("not-a-date".to_string())),
                Value::Str(Arc::new("YYYY-MM-DD".to_string())),
            ],
        );
        assert!(matches!(parse_date_invalid, Value::Float(value) if value == 0.0));

        let parse_date_missing = call_native_function(&mut interpreter, "parse_date", &[]);
        assert!(matches!(parse_date_missing, Value::Error(message) if message.contains("parse_date requires date string and format string")));
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

        let parallel_http_missing = call_native_function(&mut interpreter, "parallel_http", &[]);
        assert!(
            matches!(parallel_http_missing, Value::Error(message) if message.contains("parallel_http requires an array of URL strings"))
        );

        let jwt_encode_missing = call_native_function(&mut interpreter, "jwt_encode", &[]);
        assert!(
            matches!(jwt_encode_missing, Value::Error(message) if message.contains("jwt_encode requires a dictionary payload and secret key string"))
        );

        let jwt_decode_missing = call_native_function(&mut interpreter, "jwt_decode", &[]);
        assert!(
            matches!(jwt_decode_missing, Value::Error(message) if message.contains("jwt_decode requires a token string and secret key string"))
        );

        let oauth2_auth_url_missing =
            call_native_function(&mut interpreter, "oauth2_auth_url", &[]);
        assert!(
            matches!(oauth2_auth_url_missing, Value::Error(message) if message.contains("oauth2_auth_url requires client_id, redirect_uri, auth_url, and scope strings"))
        );

        let oauth2_get_token_missing =
            call_native_function(&mut interpreter, "oauth2_get_token", &[]);
        assert!(
            matches!(oauth2_get_token_missing, Value::Error(message) if message.contains("oauth2_get_token requires code, client_id, client_secret, token_url, and redirect_uri strings"))
        );
    }

    #[test]
    fn test_release_hardening_http_advanced_api_behavior_contracts() {
        let mut interpreter = Interpreter::new();

        let parallel_http_empty = call_native_function(
            &mut interpreter,
            "parallel_http",
            &[Value::Array(Arc::new(vec![]))],
        );
        assert!(matches!(parallel_http_empty, Value::Array(results) if results.is_empty()));

        let mut payload = crate::interpreter::DictMap::default();
        payload.insert("sub".into(), Value::Str(Arc::new("user-123".to_string())));
        payload.insert("role".into(), Value::Str(Arc::new("admin".to_string())));

        let secret = Value::Str(Arc::new("release-hardening-secret".to_string()));
        let encoded = call_native_function(
            &mut interpreter,
            "jwt_encode",
            &[Value::Dict(Arc::new(payload.clone())), secret.clone()],
        );

        let token = match encoded {
            Value::Str(token) => {
                assert!(!token.is_empty());
                token
            }
            other => panic!("Expected JWT token string from jwt_encode, got {:?}", other),
        };

        let decoded = call_native_function(
            &mut interpreter,
            "jwt_decode",
            &[Value::Str(token.clone()), secret.clone()],
        );
        match decoded {
            Value::Dict(decoded_payload) => {
                assert!(
                    matches!(decoded_payload.get("sub"), Some(Value::Str(sub)) if sub.as_ref() == "user-123")
                );
                assert!(
                    matches!(decoded_payload.get("role"), Some(Value::Str(role)) if role.as_ref() == "admin")
                );
            }
            other => panic!("Expected Dict payload from jwt_decode, got {:?}", other),
        }

        let auth_url = call_native_function(
            &mut interpreter,
            "oauth2_auth_url",
            &[
                Value::Str(Arc::new("client-abc".to_string())),
                Value::Str(Arc::new("https://example.com/callback".to_string())),
                Value::Str(Arc::new("https://auth.example.com/authorize".to_string())),
                Value::Str(Arc::new("read write".to_string())),
            ],
        );

        match auth_url {
            Value::Str(url) => {
                assert!(url.contains("client_id=client-abc"));
                assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
                assert!(url.contains("scope=read+write") || url.contains("scope=read%20write"));
            }
            other => panic!("Expected URL string from oauth2_auth_url, got {:?}", other),
        }
    }

    #[test]
    fn test_release_hardening_database_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let connect_missing = call_native_function(&mut interpreter, "db_connect", &[]);
        assert!(
            matches!(connect_missing, Value::Error(message) if message.contains("db_connect requires database type"))
        );

        let execute_missing = call_native_function(&mut interpreter, "db_execute", &[]);
        assert!(
            matches!(execute_missing, Value::Error(message) if message.contains("db_execute requires a database connection"))
        );

        let pool_missing = call_native_function(&mut interpreter, "db_pool", &[]);
        assert!(
            matches!(pool_missing, Value::Error(message) if message.contains("db_pool requires database type and connection string"))
        );
    }

    #[test]
    fn test_release_hardening_process_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let spawn_missing = call_native_function(&mut interpreter, "spawn_process", &[]);
        assert!(
            matches!(spawn_missing, Value::Error(message) if message.contains("spawn_process requires an array of command arguments"))
        );

        let pipe_missing = call_native_function(&mut interpreter, "pipe_commands", &[]);
        assert!(
            matches!(pipe_missing, Value::Error(message) if message.contains("pipe_commands requires an array of command arrays"))
        );
    }

    #[test]
    fn test_release_hardening_async_alias_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        for alias in ["Promise.all", "promise_all", "await_all"] {
            let missing_args = call_native_function(&mut interpreter, alias, &[]);
            assert!(
                matches!(missing_args, Value::Error(ref message) if message.contains("Promise.all() expects 1 or 2 arguments")),
                "Unexpected missing-args contract for {}: {:?}",
                alias,
                missing_args
            );

            let non_array = call_native_function(&mut interpreter, alias, &[Value::Int(1)]);
            assert!(
                matches!(non_array, Value::Error(ref message) if message.contains("Promise.all() requires an array of promises")),
                "Unexpected non-array contract for {}: {:?}",
                alias,
                non_array
            );

            let invalid_limit = call_native_function(
                &mut interpreter,
                alias,
                &[Value::Array(Arc::new(vec![])), Value::Int(0)],
            );
            assert!(
                matches!(invalid_limit, Value::Error(ref message) if message.contains("Promise.all() concurrency_limit must be > 0")),
                "Unexpected concurrency-limit contract for {}: {:?}",
                alias,
                invalid_limit
            );
        }

        for alias in ["parallel_map", "par_map", "par_each"] {
            let non_array = call_native_function(
                &mut interpreter,
                alias,
                &[Value::Int(1), Value::NativeFunction("len".to_string())],
            );
            assert!(
                matches!(non_array, Value::Error(ref message) if message.contains("parallel_map() first argument must be an array")),
                "Unexpected non-array contract for {}: {:?}",
                alias,
                non_array
            );

            let non_callable = call_native_function(
                &mut interpreter,
                alias,
                &[Value::Array(Arc::new(vec![Value::Int(1)])), Value::Int(1)],
            );
            assert!(
                matches!(non_callable, Value::Error(ref message) if message.contains("parallel_map() second argument must be a callable function")),
                "Unexpected callable-shape contract for {}: {:?}",
                alias,
                non_callable
            );

            let invalid_limit = call_native_function(
                &mut interpreter,
                alias,
                &[
                    Value::Array(Arc::new(vec![Value::Int(1)])),
                    Value::NativeFunction("len".to_string()),
                    Value::Int(0),
                ],
            );
            assert!(
                matches!(invalid_limit, Value::Error(ref message) if message.contains("parallel_map() concurrency_limit must be > 0")),
                "Unexpected concurrency-limit contract for {}: {:?}",
                alias,
                invalid_limit
            );
        }
    }

    #[test]
    fn test_release_hardening_ssg_render_pages_dispatch_contracts() {
        let mut interpreter = Interpreter::new();

        let missing_args = call_native_function(&mut interpreter, "ssg_render_pages", &[]);
        assert!(
            matches!(missing_args, Value::Error(message) if message.contains("ssg_render_pages() expects 1 argument"))
        );

        let non_array = call_native_function(
            &mut interpreter,
            "ssg_render_pages",
            &[Value::Str(Arc::new("not-an-array".to_string()))],
        );
        assert!(
            matches!(non_array, Value::Error(message) if message.contains("requires an array of source page strings"))
        );

        let non_string_element = call_native_function(
            &mut interpreter,
            "ssg_render_pages",
            &[Value::Array(Arc::new(vec![Value::Int(1)]))],
        );
        assert!(
            matches!(non_string_element, Value::Error(message) if message.contains("source page at index 0 must be a string"))
        );

        let success = call_native_function(
            &mut interpreter,
            "ssg_render_pages",
            &[Value::Array(Arc::new(vec![
                Value::Str(Arc::new("# Post A".to_string())),
                Value::Str(Arc::new("# Post B".to_string())),
            ]))],
        );

        match success {
            Value::Dict(result) => {
                assert!(
                    matches!(result.get("pages"), Some(Value::Array(pages)) if pages.len() == 2)
                );
                assert!(
                    matches!(result.get("checksum"), Some(Value::Int(checksum)) if *checksum > 0)
                );
            }
            other => panic!("Expected Dict from ssg_render_pages, got {:?}", other),
        }
    }

    #[test]
    fn test_release_hardening_load_image_dispatch_contracts() {
        let mut interpreter = Interpreter::new();

        let load_image_missing_args = call_native_function(&mut interpreter, "load_image", &[]);
        assert!(
            matches!(load_image_missing_args, Value::Error(message) if message.contains("load_image requires a string path argument"))
        );

        let missing_path = tmp_test_path("dispatch_missing_image.png");
        let load_image_missing_path = call_native_function(
            &mut interpreter,
            "load_image",
            &[Value::Str(Arc::new(missing_path.clone()))],
        );
        assert!(
            matches!(load_image_missing_path, Value::Error(message) if message.contains("Cannot load image") && message.contains(missing_path.as_str()))
        );
    }

    #[test]
    fn test_release_hardening_load_image_round_trip_behavior() {
        let mut interpreter = Interpreter::new();
        let image_path = tmp_test_path("dispatch_load_image.png");

        let mut image_buffer = image::RgbaImage::new(2, 1);
        image_buffer.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        image_buffer.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
        image_buffer.save(&image_path).expect("seed image should be written");

        let load_result = call_native_function(
            &mut interpreter,
            "load_image",
            &[Value::Str(Arc::new(image_path.clone()))],
        );

        match load_result {
            Value::Image { data, format } => {
                assert_eq!(format, "png");
                let image_data = data.lock().expect("image mutex should lock");
                assert_eq!(image_data.width(), 2);
                assert_eq!(image_data.height(), 1);
            }
            other => panic!("Expected Value::Image, got {:?}", other),
        }

        let _ = std::fs::remove_file(image_path);
    }

    #[test]
    fn test_release_hardening_network_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let tcp_listen_missing = call_native_function(&mut interpreter, "tcp_listen", &[]);
        assert!(
            matches!(tcp_listen_missing, Value::Error(message) if message.contains("tcp_listen requires (string_host, int_port) arguments"))
        );

        let tcp_accept_missing = call_native_function(&mut interpreter, "tcp_accept", &[]);
        assert!(
            matches!(tcp_accept_missing, Value::Error(message) if message.contains("tcp_accept requires a TcpListener argument"))
        );

        let tcp_connect_missing = call_native_function(&mut interpreter, "tcp_connect", &[]);
        assert!(
            matches!(tcp_connect_missing, Value::Error(message) if message.contains("tcp_connect requires (string_host, int_port) arguments"))
        );

        let tcp_send_missing = call_native_function(&mut interpreter, "tcp_send", &[]);
        assert!(
            matches!(tcp_send_missing, Value::Error(message) if message.contains("tcp_send requires (TcpStream, string_or_bytes_data) arguments"))
        );

        let tcp_receive_missing = call_native_function(&mut interpreter, "tcp_receive", &[]);
        assert!(
            matches!(tcp_receive_missing, Value::Error(message) if message.contains("tcp_receive requires (TcpStream, int_size) arguments"))
        );

        let tcp_close_missing = call_native_function(&mut interpreter, "tcp_close", &[]);
        assert!(
            matches!(tcp_close_missing, Value::Error(message) if message.contains("tcp_close requires a TcpStream or TcpListener argument"))
        );

        let tcp_nonblocking_missing =
            call_native_function(&mut interpreter, "tcp_set_nonblocking", &[]);
        assert!(
            matches!(tcp_nonblocking_missing, Value::Error(message) if message.contains("tcp_set_nonblocking requires (TcpStream/TcpListener, bool) arguments"))
        );

        let udp_bind_missing = call_native_function(&mut interpreter, "udp_bind", &[]);
        assert!(
            matches!(udp_bind_missing, Value::Error(message) if message.contains("udp_bind requires (string_host, int_port) arguments"))
        );

        let udp_send_missing = call_native_function(&mut interpreter, "udp_send_to", &[]);
        assert!(
            matches!(udp_send_missing, Value::Error(message) if message.contains("udp_send_to requires (UdpSocket, string_or_bytes_data, string_host, int_port) arguments"))
        );

        let udp_receive_missing = call_native_function(&mut interpreter, "udp_receive_from", &[]);
        assert!(
            matches!(udp_receive_missing, Value::Error(message) if message.contains("udp_receive_from requires (UdpSocket, int_size) arguments"))
        );

        let udp_close_missing = call_native_function(&mut interpreter, "udp_close", &[]);
        assert!(
            matches!(udp_close_missing, Value::Error(message) if message.contains("udp_close requires a UdpSocket argument"))
        );
    }

    #[test]
    fn test_release_hardening_network_module_round_trip_behaviors() {
        let tcp_port = available_tcp_port();
        let mut server_interpreter = Interpreter::new();

        let listener = call_native_function(
            &mut server_interpreter,
            "tcp_listen",
            &[Value::Str(Arc::new("127.0.0.1".to_string())), Value::Int(tcp_port)],
        );
        let listener_value = match listener {
            Value::TcpListener { .. } => listener,
            other => panic!("Expected TcpListener from tcp_listen, got {:?}", other),
        };

        let listener_for_client = listener_value.clone();
        let client_thread = std::thread::spawn(move || {
            let mut client_interpreter = Interpreter::new();
            let stream = call_native_function(
                &mut client_interpreter,
                "tcp_connect",
                &[Value::Str(Arc::new("127.0.0.1".to_string())), Value::Int(tcp_port)],
            );
            let stream_value = match stream {
                Value::TcpStream { .. } => stream,
                other => panic!("Expected TcpStream from tcp_connect, got {:?}", other),
            };

            let sent = call_native_function(
                &mut client_interpreter,
                "tcp_send",
                &[stream_value.clone(), Value::Str(Arc::new("network-payload".to_string()))],
            );
            assert!(matches!(sent, Value::Int(15)));

            let close_result =
                call_native_function(&mut client_interpreter, "tcp_close", &[stream_value]);
            assert!(matches!(close_result, Value::Bool(true)));

            let close_listener_result =
                call_native_function(&mut client_interpreter, "tcp_close", &[listener_for_client]);
            assert!(matches!(close_listener_result, Value::Bool(true)));
        });

        let accepted =
            call_native_function(&mut server_interpreter, "tcp_accept", &[listener_value.clone()]);
        let accepted_stream = match accepted {
            Value::TcpStream { .. } => accepted,
            other => panic!("Expected TcpStream from tcp_accept, got {:?}", other),
        };

        let tcp_receive = call_native_function(
            &mut server_interpreter,
            "tcp_receive",
            &[accepted_stream.clone(), Value::Int(128)],
        );
        assert!(matches!(tcp_receive, Value::Str(data) if data.as_ref() == "network-payload"));

        let nonblocking_stream_result = call_native_function(
            &mut server_interpreter,
            "tcp_set_nonblocking",
            &[accepted_stream.clone(), Value::Bool(false)],
        );
        assert!(matches!(nonblocking_stream_result, Value::Bool(true)));

        let nonblocking_listener_result = call_native_function(
            &mut server_interpreter,
            "tcp_set_nonblocking",
            &[listener_value.clone(), Value::Bool(false)],
        );
        assert!(matches!(nonblocking_listener_result, Value::Bool(true)));

        let close_stream_result =
            call_native_function(&mut server_interpreter, "tcp_close", &[accepted_stream]);
        assert!(matches!(close_stream_result, Value::Bool(true)));

        let close_listener_result =
            call_native_function(&mut server_interpreter, "tcp_close", &[listener_value]);
        assert!(matches!(close_listener_result, Value::Bool(true)));

        client_thread.join().expect("tcp client thread should complete");

        let udp_port = available_udp_port();
        let mut udp_interpreter = Interpreter::new();

        let receiver_socket = call_native_function(
            &mut udp_interpreter,
            "udp_bind",
            &[Value::Str(Arc::new("127.0.0.1".to_string())), Value::Int(udp_port)],
        );
        let receiver_value = match receiver_socket {
            Value::UdpSocket { .. } => receiver_socket,
            other => panic!("Expected UdpSocket for receiver, got {:?}", other),
        };

        let sender_socket = call_native_function(
            &mut udp_interpreter,
            "udp_bind",
            &[Value::Str(Arc::new("127.0.0.1".to_string())), Value::Int(0)],
        );
        let sender_value = match sender_socket {
            Value::UdpSocket { .. } => sender_socket,
            other => panic!("Expected UdpSocket for sender, got {:?}", other),
        };

        let udp_sent = call_native_function(
            &mut udp_interpreter,
            "udp_send_to",
            &[
                sender_value.clone(),
                Value::Str(Arc::new("udp-payload".to_string())),
                Value::Str(Arc::new("127.0.0.1".to_string())),
                Value::Int(udp_port),
            ],
        );
        assert!(matches!(udp_sent, Value::Int(11)));

        let udp_received = call_native_function(
            &mut udp_interpreter,
            "udp_receive_from",
            &[receiver_value.clone(), Value::Int(128)],
        );

        match udp_received {
            Value::Dict(result) => {
                assert!(
                    matches!(result.get("data"), Some(Value::Str(data)) if data.as_ref() == "udp-payload")
                );
                assert!(matches!(result.get("size"), Some(Value::Int(11))));
                assert!(
                    matches!(result.get("from"), Some(Value::Str(from)) if from.as_ref().contains("127.0.0.1:"))
                );
            }
            other => panic!("Expected udp_receive_from dictionary result, got {:?}", other),
        }

        let udp_close_sender =
            call_native_function(&mut udp_interpreter, "udp_close", &[sender_value]);
        assert!(matches!(udp_close_sender, Value::Bool(true)));

        let udp_close_receiver =
            call_native_function(&mut udp_interpreter, "udp_close", &[receiver_value]);
        assert!(matches!(udp_close_receiver, Value::Bool(true)));
    }

    #[test]
    fn test_release_hardening_zip_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let zip_create_missing = call_native_function(&mut interpreter, "zip_create", &[]);
        assert!(
            matches!(zip_create_missing, Value::Error(message) if message.contains("zip_create requires a string path argument"))
        );

        let zip_add_file_missing = call_native_function(&mut interpreter, "zip_add_file", &[]);
        assert!(
            matches!(zip_add_file_missing, Value::Error(message) if message.contains("zip_add_file requires (ZipArchive, string_path) arguments"))
        );

        let zip_add_dir_missing = call_native_function(&mut interpreter, "zip_add_dir", &[]);
        assert!(
            matches!(zip_add_dir_missing, Value::Error(message) if message.contains("zip_add_dir requires (ZipArchive, string_path) arguments"))
        );

        let zip_close_missing = call_native_function(&mut interpreter, "zip_close", &[]);
        assert!(
            matches!(zip_close_missing, Value::Error(message) if message.contains("zip_close requires a ZipArchive argument"))
        );

        let unzip_missing = call_native_function(&mut interpreter, "unzip", &[]);
        assert!(
            matches!(unzip_missing, Value::Error(message) if message.contains("unzip requires (string_zip_path, string_output_dir) arguments"))
        );
    }

    #[test]
    fn test_release_hardening_zip_module_round_trip_behaviors() {
        let mut interpreter = Interpreter::new();
        let archive_path = tmp_test_path("dispatch_zip_roundtrip.zip");
        let source_file_path = tmp_test_path("dispatch_zip_source.txt");
        let source_dir_path = tmp_test_path("dispatch_zip_source_dir");
        let nested_file_path = format!("{}/nested.txt", source_dir_path);
        let unzip_output_path = tmp_test_path("dispatch_zip_unzip_output");

        std::fs::write(&source_file_path, "zip dispatch content")
            .expect("seed zip file should be written");
        std::fs::create_dir_all(&source_dir_path).expect("seed zip directory should be created");
        std::fs::write(&nested_file_path, "zip nested content")
            .expect("nested file should be written");

        let zip_archive = call_native_function(
            &mut interpreter,
            "zip_create",
            &[Value::Str(Arc::new(archive_path.clone()))],
        );
        let archive_value = match zip_archive {
            Value::ZipArchive { .. } => zip_archive,
            other => panic!("Expected zip archive from zip_create, got {:?}", other),
        };

        let add_file_result = call_native_function(
            &mut interpreter,
            "zip_add_file",
            &[archive_value.clone(), Value::Str(Arc::new(source_file_path.clone()))],
        );
        assert!(matches!(add_file_result, Value::Bool(true)));

        let add_dir_result = call_native_function(
            &mut interpreter,
            "zip_add_dir",
            &[archive_value.clone(), Value::Str(Arc::new(source_dir_path.clone()))],
        );
        assert!(matches!(add_dir_result, Value::Bool(true)));

        let close_result =
            call_native_function(&mut interpreter, "zip_close", &[archive_value.clone()]);
        assert!(matches!(close_result, Value::Bool(true)));

        let close_again_result =
            call_native_function(&mut interpreter, "zip_close", &[archive_value.clone()]);
        assert!(
            matches!(close_again_result, Value::Error(message) if message.contains("already been closed"))
        );

        let unzip_result = call_native_function(
            &mut interpreter,
            "unzip",
            &[
                Value::Str(Arc::new(archive_path.clone())),
                Value::Str(Arc::new(unzip_output_path.clone())),
            ],
        );

        match unzip_result {
            Value::Array(extracted_paths) => {
                assert!(!extracted_paths.is_empty());
            }
            other => panic!("Expected extracted file list from unzip, got {:?}", other),
        }

        let extracted_file = format!("{}/dispatch_zip_source.txt", unzip_output_path);
        let extracted_nested_file = format!("{}/nested.txt", unzip_output_path);
        let extracted_file_content =
            std::fs::read_to_string(&extracted_file).expect("extracted source file should exist");
        let extracted_nested_content = std::fs::read_to_string(&extracted_nested_file)
            .expect("extracted nested file should exist");
        assert_eq!(extracted_file_content, "zip dispatch content");
        assert_eq!(extracted_nested_content, "zip nested content");

        let _ = std::fs::remove_file(archive_path);
        let _ = std::fs::remove_file(source_file_path);
        let _ = std::fs::remove_dir_all(source_dir_path);
        let _ = std::fs::remove_dir_all(unzip_output_path);
    }

    #[test]
    fn test_release_hardening_crypto_module_dispatch_argument_contracts() {
        let mut interpreter = Interpreter::new();

        let sha_missing = call_native_function(&mut interpreter, "sha256", &[]);
        assert!(
            matches!(sha_missing, Value::Error(message) if message.contains("sha256 requires a string argument"))
        );

        let verify_missing = call_native_function(
            &mut interpreter,
            "verify_password",
            &[Value::Str(std::sync::Arc::new("only_one".to_string()))],
        );
        assert!(
            matches!(verify_missing, Value::Error(message) if message.contains("verify_password requires"))
        );

        let aes_missing = call_native_function(
            &mut interpreter,
            "aes_encrypt",
            &[Value::Str(std::sync::Arc::new("plaintext".to_string()))],
        );
        assert!(
            matches!(aes_missing, Value::Error(message) if message.contains("aes_encrypt requires"))
        );

        let rsa_bad_size =
            call_native_function(&mut interpreter, "rsa_generate_keypair", &[Value::Int(1024)]);
        assert!(
            matches!(rsa_bad_size, Value::Error(message) if message.contains("RSA key size must be 2048 or 4096 bits"))
        );
    }

    #[test]
    fn test_release_hardening_set_constructor_dispatch_contracts() {
        let mut interpreter = Interpreter::new();

        let set_empty = call_native_function(&mut interpreter, "Set", &[]);
        assert!(matches!(set_empty, Value::Set(items) if items.is_empty()));

        let set_from_array = call_native_function(
            &mut interpreter,
            "Set",
            &[Value::Array(std::sync::Arc::new(vec![Value::Int(1), Value::Int(1), Value::Int(2)]))],
        );
        assert!(matches!(set_from_array, Value::Set(items) if items.len() == 2));

        let set_invalid_type = call_native_function(&mut interpreter, "Set", &[Value::Int(42)]);
        assert!(
            matches!(set_invalid_type, Value::Error(message) if message.contains("Set constructor requires an array argument"))
        );

        let set_too_many = call_native_function(
            &mut interpreter,
            "Set",
            &[Value::Array(std::sync::Arc::new(vec![])), Value::Int(1)],
        );
        assert!(
            matches!(set_too_many, Value::Error(message) if message.contains("Set constructor takes at most 1 argument"))
        );
    }
}
