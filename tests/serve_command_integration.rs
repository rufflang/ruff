use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEST_HOST: &str = "127.0.0.1";

struct HttpResponse {
	status_code: u16,
	headers: HashMap<String, String>,
	body: Vec<u8>,
}

struct ServeProcess {
	child: Child,
	port: u16,
}

impl Drop for ServeProcess {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("system time should be after unix epoch")
		.as_nanos();
	let path = std::env::temp_dir().join(format!("ruff_{}_{}", prefix, nanos));
	fs::create_dir_all(&path).expect("failed to create temp directory");
	path
}

fn ruff_binary() -> String {
	env!("CARGO_BIN_EXE_ruff").to_string()
}

fn find_free_port() -> u16 {
	let listener = TcpListener::bind((TEST_HOST, 0)).expect("failed to allocate test port");
	listener
		.local_addr()
		.expect("listener should have local addr")
		.port()
}

fn spawn_serve_process(root: &Path) -> ServeProcess {
	let port = find_free_port();
	let mut child = Command::new(ruff_binary())
		.current_dir(root)
		.args([
			"serve",
			root.to_str().expect("path should be utf-8"),
			"--host",
			TEST_HOST,
			"--port",
			&port.to_string(),
			"--index",
			"index.html",
			"--cache-max-age",
			"120",
		])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.expect("failed to spawn ruff serve process");

	for _ in 0..40 {
		if TcpStream::connect((TEST_HOST, port)).is_ok() {
			return ServeProcess { child, port };
		}

		if let Some(status) = child.try_wait().expect("failed to poll child process") {
			panic!("ruff serve exited before readiness check completed: {}", status);
		}

		thread::sleep(Duration::from_millis(50));
	}

	let _ = child.kill();
	let _ = child.wait();
	panic!("ruff serve did not become reachable in time on port {}", port);
}

fn send_http_request(port: u16, request: &str) -> HttpResponse {
	let mut stream = TcpStream::connect((TEST_HOST, port)).expect("failed to connect to serve process");
	stream
		.set_read_timeout(Some(Duration::from_secs(2)))
		.expect("failed to set read timeout");
	stream
		.set_write_timeout(Some(Duration::from_secs(2)))
		.expect("failed to set write timeout");

	stream
		.write_all(request.as_bytes())
		.expect("failed to write HTTP request");
	let _ = stream.shutdown(Shutdown::Write);

	let mut response_bytes = Vec::new();
	stream
		.read_to_end(&mut response_bytes)
		.expect("failed to read HTTP response");
	parse_http_response(&response_bytes)
}

fn parse_http_response(response_bytes: &[u8]) -> HttpResponse {
	let Some(headers_end) = response_bytes
		.windows(4)
		.position(|window| window == b"\r\n\r\n")
	else {
		panic!("invalid HTTP response: missing header terminator");
	};

	let headers_blob = &response_bytes[..headers_end + 4];
	let body = response_bytes[headers_end + 4..].to_vec();
	let headers_text = String::from_utf8_lossy(headers_blob);
	let mut lines = headers_text.lines();

	let status_line = lines
		.next()
		.expect("HTTP response should include status line");
	let status_code = status_line
		.split_whitespace()
		.nth(1)
		.expect("status line should include numeric status code")
		.parse::<u16>()
		.expect("status code should be valid u16");

	let mut headers = HashMap::new();
	for line in lines {
		if line.is_empty() {
			break;
		}

		if let Some((name, value)) = line.split_once(':') {
			headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
		}
	}

	HttpResponse {
		status_code,
		headers,
		body,
	}
}

#[test]
fn serve_head_returns_headers_without_body() {
	let root = unique_temp_dir("serve_head");
	fs::write(root.join("index.html"), "<h1>Hello</h1>").expect("failed to write index.html");

	let server = spawn_serve_process(&root);
	let response = send_http_request(
		server.port,
		"HEAD / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
	);

	assert_eq!(200, response.status_code);
	assert_eq!(
		"text/html; charset=utf-8",
		response
			.headers
			.get("content-type")
			.expect("expected Content-Type header")
	);
	assert_eq!(0, response.body.len(), "HEAD responses should not include body bytes");
	assert_eq!(
		"14",
		response
			.headers
			.get("content-length")
			.expect("expected Content-Length header")
	);

	drop(server);
	let _ = fs::remove_dir_all(root);
}

#[test]
fn serve_range_returns_partial_content_and_content_range_header() {
	let root = unique_temp_dir("serve_range");
	fs::write(root.join("index.html"), b"0123456789").expect("failed to write index.html");

	let server = spawn_serve_process(&root);
	let response = send_http_request(
		server.port,
		"GET /index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes=2-5\r\nConnection: close\r\n\r\n",
	);

	assert_eq!(206, response.status_code);
	assert_eq!(
		"bytes 2-5/10",
		response
			.headers
			.get("content-range")
			.expect("expected Content-Range header")
	);
	assert_eq!(b"2345", response.body.as_slice());

	drop(server);
	let _ = fs::remove_dir_all(root);
}

#[test]
fn serve_if_none_match_returns_304_for_matching_etag() {
	let root = unique_temp_dir("serve_etag");
	fs::write(root.join("index.html"), b"etag-body").expect("failed to write index.html");

	let server = spawn_serve_process(&root);
	let first = send_http_request(
		server.port,
		"GET /index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
	);
	assert_eq!(200, first.status_code);
	let etag = first
		.headers
		.get("etag")
		.expect("expected ETag header")
		.to_string();

	let second = send_http_request(
		server.port,
		&format!(
			"GET /index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nIf-None-Match: {}\r\nConnection: close\r\n\r\n",
			etag
		),
	);

	assert_eq!(304, second.status_code);
	assert_eq!(0, second.body.len());

	drop(server);
	let _ = fs::remove_dir_all(root);
}

#[test]
fn serve_accept_encoding_prefers_gzip_sibling_asset() {
	let root = unique_temp_dir("serve_encoding");
	fs::write(root.join("index.html"), b"plain-body").expect("failed to write index.html");
	let gzip_bytes = vec![0x1f, 0x8b, 0x08, 0x00, 0x13, 0x37, 0x00, 0x00];
	fs::write(root.join("index.html.gz"), &gzip_bytes).expect("failed to write gzip sibling");

	let server = spawn_serve_process(&root);
	let response = send_http_request(
		server.port,
		"GET /index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept-Encoding: gzip\r\nConnection: close\r\n\r\n",
	);

	assert_eq!(200, response.status_code);
	assert_eq!(
		"gzip",
		response
			.headers
			.get("content-encoding")
			.expect("expected Content-Encoding header")
	);
	assert_eq!(gzip_bytes, response.body);

	drop(server);
	let _ = fs::remove_dir_all(root);
}