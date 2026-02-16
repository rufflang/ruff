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
            "Set",
            "ssg_render_pages",
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
