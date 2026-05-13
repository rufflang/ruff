use reqwest::blocking::{Client, Response};
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub const DEFAULT_NETWORK_CONNECT_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_NETWORK_READ_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_NETWORK_WRITE_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_HTTP_TIMEOUT_MS: u64 = 30_000;
pub const MAX_NETWORK_BODY_BYTES: usize = 8 * 1024 * 1024;

pub fn run_blocking_http_task<T, F>(surface: &str, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    match std::thread::spawn(task).join() {
        Ok(result) => result,
        Err(_) => Err(format!("{} failed: blocking HTTP worker thread panicked", surface)),
    }
}

pub fn connect_timeout() -> Duration {
    Duration::from_millis(DEFAULT_NETWORK_CONNECT_TIMEOUT_MS)
}

pub fn read_timeout() -> Duration {
    Duration::from_millis(DEFAULT_NETWORK_READ_TIMEOUT_MS)
}

pub fn write_timeout() -> Duration {
    Duration::from_millis(DEFAULT_NETWORK_WRITE_TIMEOUT_MS)
}

pub fn default_http_timeout() -> Duration {
    Duration::from_millis(DEFAULT_HTTP_TIMEOUT_MS)
}

pub fn build_http_client(timeout: Duration) -> Result<Client, String> {
    Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {}", error))
}

pub fn read_http_response_bytes(
    response: Response,
    surface: &str,
) -> Result<(u16, reqwest::header::HeaderMap, Vec<u8>), String> {
    let status = response.status().as_u16();
    let headers = response.headers().clone();

    if let Some(content_length) = response.content_length() {
        if content_length > MAX_NETWORK_BODY_BYTES as u64 {
            return Err(format!(
                "{} failed: response body exceeds maximum network body size ({} bytes > {} bytes)",
                surface, content_length, MAX_NETWORK_BODY_BYTES
            ));
        }
    }

    let max_plus_one = (MAX_NETWORK_BODY_BYTES as u64).saturating_add(1);
    let mut bytes = Vec::new();
    response
        .take(max_plus_one)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{} failed while reading response body: {}", surface, error))?;

    if bytes.len() > MAX_NETWORK_BODY_BYTES {
        return Err(format!(
            "{} failed: response body exceeds maximum network body size ({} bytes > {} bytes)",
            surface,
            bytes.len(),
            MAX_NETWORK_BODY_BYTES
        ));
    }

    Ok((status, headers, bytes))
}

pub fn apply_tcp_stream_timeouts(stream: &TcpStream, surface: &str) -> Result<(), String> {
    stream
        .set_read_timeout(Some(read_timeout()))
        .map_err(|error| format!("{} failed to set read timeout: {}", surface, error))?;
    stream
        .set_write_timeout(Some(write_timeout()))
        .map_err(|error| format!("{} failed to set write timeout: {}", surface, error))?;
    Ok(())
}

pub fn apply_udp_socket_timeouts(socket: &UdpSocket, surface: &str) -> Result<(), String> {
    socket
        .set_read_timeout(Some(read_timeout()))
        .map_err(|error| format!("{} failed to set read timeout: {}", surface, error))?;
    socket
        .set_write_timeout(Some(write_timeout()))
        .map_err(|error| format!("{} failed to set write timeout: {}", surface, error))?;
    Ok(())
}

pub fn connect_tcp_stream(address: &str, surface: &str) -> Result<TcpStream, String> {
    let timeout = connect_timeout();
    let addresses = address
        .to_socket_addrs()
        .map_err(|error| format!("{} failed to resolve '{}': {}", surface, address, error))?;

    let mut last_error = None;
    for candidate in addresses {
        match TcpStream::connect_timeout(&candidate, timeout) {
            Ok(stream) => {
                apply_tcp_stream_timeouts(&stream, surface)?;
                return Ok(stream);
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    match last_error {
        Some(error) => Err(format!("{} failed to connect to '{}': {}", surface, address, error)),
        None => Err(format!("{} failed: no socket addresses resolved for '{}'", surface, address)),
    }
}

pub fn validate_receive_size(size: i64, surface: &str) -> Result<usize, String> {
    if size <= 0 {
        return Err(format!("{} size must be positive", surface));
    }

    let size = size as usize;
    if size > MAX_NETWORK_BODY_BYTES {
        return Err(format!(
            "{} size exceeds maximum network body size ({} bytes > {} bytes)",
            surface, size, MAX_NETWORK_BODY_BYTES
        ));
    }

    Ok(size)
}
