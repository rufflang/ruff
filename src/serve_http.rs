use crate::path_security;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(Debug, Clone)]
pub struct ServeServerOptions {
    pub index: String,
    pub hardened: bool,
    pub cache_max_age: Option<u64>,
    pub access_log: bool,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
}

#[derive(Debug)]
struct PlannedResponse {
    status_code: u16,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    content_length: Option<usize>,
}

pub fn run_static_server(
    dir: PathBuf,
    host: String,
    port: u16,
    options: ServeServerOptions,
) -> Result<(), String> {
    let root_dir = fs::canonicalize(&dir)
        .map_err(|err| format!("Failed to resolve serve directory '{}': {}", dir.display(), err))?;

    if !root_dir.is_dir() {
        return Err(format!("Serve target is not a directory: {}", root_dir.display()));
    }

    let tls_enabled = options.tls_cert.is_some() || options.tls_key.is_some();
    if tls_enabled && (options.tls_cert.is_none() || options.tls_key.is_none()) {
        return Err(
            "HTTPS requires both --tls-cert and --tls-key to be provided together".to_string()
        );
    }

    let bind_addr = format!("{}:{}", host, port);
    let server = if tls_enabled {
        let certificate = fs::read(options.tls_cert.as_ref().expect("tls cert exists"))
            .map_err(|err| format!("Failed to read TLS certificate file: {}", err))?;
        let private_key = fs::read(options.tls_key.as_ref().expect("tls key exists"))
            .map_err(|err| format!("Failed to read TLS private key file: {}", err))?;

        Server::https(&bind_addr, tiny_http::SslConfig { certificate, private_key })
            .map_err(|err| format!("Failed to start HTTPS server on {}: {}", bind_addr, err))?
    } else {
        Server::http(&bind_addr)
            .map_err(|err| format!("Failed to start server on {}: {}", bind_addr, err))?
    };

    let scheme = if tls_enabled { "https" } else { "http" };
    println!("Serving {} on {}://{}", root_dir.display(), scheme, bind_addr);
    println!("Press Ctrl+C to stop");

    for request in server.incoming_requests() {
        let started = Instant::now();
        let method = request.method().clone();
        let url = request.url().to_string();
        let secure = request.secure();

        let planned = build_response(&root_dir, &options, &request);
        let status_code = planned.status_code;
        let response_bytes = planned.content_length.unwrap_or_else(|| planned.body.len());
        let response = planned_to_http_response(planned);
        let _ = request.respond(response);

        log_access(
            options.access_log,
            &method,
            &url,
            status_code,
            response_bytes,
            started.elapsed().as_millis(),
            secure,
        );
    }

    Ok(())
}

fn planned_to_http_response(plan: PlannedResponse) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response =
        Response::from_data(plan.body).with_status_code(StatusCode(plan.status_code));
    if let Some(content_length) = plan.content_length {
        if let Ok(content_length_header) =
            Header::from_bytes("Content-Length".as_bytes(), content_length.to_string().as_bytes())
        {
            response.add_header(content_length_header);
        }
    }

    for (name, value) in plan.headers.iter() {
        if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response.add_header(header);
        }
    }

    response
}

fn build_response(
    root_dir: &Path,
    options: &ServeServerOptions,
    request: &Request,
) -> PlannedResponse {
    let method = request.method();
    let is_head = method == &Method::Head;
    if method != &Method::Get && !is_head {
        return headify(
            text_response(405, "Method Not Allowed", options, request.secure()),
            is_head,
        );
    }

    let raw_path = request.url().split('?').next().unwrap_or("/");
    if path_security::reject_url_encoded_parent_traversal(raw_path, "request path").is_err() {
        return headify(text_response(403, "Forbidden", options, request.secure()), is_head);
    }

    let mut relative_path = raw_path.trim_start_matches('/').to_string();
    if relative_path.is_empty() {
        relative_path = options.index.clone();
    }

    let relative_path = match path_security::sanitize_relative_path(&relative_path, "request path")
    {
        Ok(path) => path,
        Err(_) => {
            return headify(text_response(403, "Forbidden", options, request.secure()), is_head);
        }
    };

    let mut target_path = match path_security::join_within_root(root_dir, &relative_path, "request path") {
        Ok(path) => path,
        Err(_) => {
            return headify(text_response(403, "Forbidden", options, request.secure()), is_head);
        }
    };
    if target_path.is_dir() {
        target_path = target_path.join(&options.index);
    }

    let canonical_target = match fs::canonicalize(&target_path) {
        Ok(path) => path,
        Err(error) => {
            return headify(status_mapped_error(&error, options, request.secure()), is_head);
        }
    };

    if path_security::ensure_path_within_root(&canonical_target, root_dir, "request path")
        .is_err()
    {
        return headify(text_response(403, "Forbidden", options, request.secure()), is_head);
    }

    let accept_encoding = header_value(request.headers(), "Accept-Encoding");
    let selected_target = choose_precompressed_path(&canonical_target, accept_encoding.as_deref());

    let file_bytes = match read_file_nofollow(&selected_target) {
        Ok(bytes) => bytes,
        Err(error) => {
            return headify(status_mapped_error(&error, options, request.secure()), is_head);
        }
    };

    let file_len = file_bytes.len();
    let etag = weak_etag(&selected_target, file_len);
    if let Some(if_none_match) = header_value(request.headers(), "If-None-Match") {
        if if_none_match_matches(&if_none_match, &etag) {
            let mut response = PlannedResponse {
                status_code: 304,
                body: Vec::new(),
                headers: Vec::new(),
                content_length: Some(0),
            };
            add_common_success_headers(
                &mut response.headers,
                &selected_target,
                &file_bytes,
                Some(&etag),
                options,
                request.secure(),
            );
            return headify(response, true);
        }
    }

    let mut status_code = 200;
    let mut body = file_bytes;
    if let Some(range_header) = header_value(request.headers(), "Range") {
        match parse_single_range(&range_header, body.len()) {
            Ok(Some((start, end))) => {
                status_code = 206;
                body = body[start..=end].to_vec();
                let mut response = PlannedResponse {
                    status_code,
                    content_length: Some(end - start + 1),
                    body,
                    headers: Vec::new(),
                };
                add_common_success_headers(
                    &mut response.headers,
                    &selected_target,
                    &response.body,
                    Some(&etag),
                    options,
                    request.secure(),
                );
                add_header(
                    &mut response.headers,
                    "Content-Range",
                    &format!("bytes {}-{}/{}", start, end, file_len),
                );
                return headify(response, is_head);
            }
            Ok(None) => {}
            Err(_) => {
                let mut response = PlannedResponse {
                    status_code: 416,
                    body: b"Range Not Satisfiable".to_vec(),
                    headers: Vec::new(),
                    content_length: Some("Range Not Satisfiable".len()),
                };
                add_header(
                    &mut response.headers,
                    "Content-Range",
                    &format!("bytes */{}", file_len),
                );
                add_header(&mut response.headers, "Content-Type", "text/plain; charset=utf-8");
                add_header(&mut response.headers, "Cache-Control", "no-store");
                add_default_security_headers(&mut response.headers, options, request.secure());
                return headify(response, is_head);
            }
        }
    }

    let mut response = PlannedResponse {
        status_code,
        content_length: Some(body.len()),
        body,
        headers: Vec::new(),
    };
    add_common_success_headers(
        &mut response.headers,
        &selected_target,
        &response.body,
        Some(&etag),
        options,
        request.secure(),
    );
    headify(response, is_head)
}

fn add_common_success_headers(
    headers: &mut Vec<(String, String)>,
    path: &Path,
    file_bytes: &[u8],
    etag: Option<&str>,
    options: &ServeServerOptions,
    secure: bool,
) {
    let (content_type, content_encoding) = guess_response_headers(path, file_bytes);
    add_header(headers, "Content-Type", &content_type);
    if let Some(encoding) = content_encoding {
        add_header(headers, "Content-Encoding", encoding);
    }
    add_header(headers, "Accept-Ranges", "bytes");
    if let Some(etag_value) = etag {
        add_header(headers, "ETag", etag_value);
    }
    if let Some(cache_max_age) = options.cache_max_age {
        add_header(headers, "Cache-Control", &format!("public, max-age={}", cache_max_age));
    } else if options.hardened {
        add_header(headers, "Cache-Control", "public, max-age=0, must-revalidate");
    }
    add_default_security_headers(headers, options, secure);
}

fn add_default_security_headers(
    headers: &mut Vec<(String, String)>,
    options: &ServeServerOptions,
    secure: bool,
) {
    add_header(headers, "X-Content-Type-Options", "nosniff");
    if options.hardened {
        add_header(headers, "X-Frame-Options", "DENY");
        add_header(headers, "Referrer-Policy", "no-referrer");
        add_header(headers, "Cross-Origin-Opener-Policy", "same-origin");
        add_header(headers, "Cross-Origin-Resource-Policy", "same-origin");
        add_header(headers, "Permissions-Policy", "geolocation=(), microphone=(), camera=()");
        add_header(
            headers,
            "Content-Security-Policy",
            "default-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
        );
        if secure {
            add_header(headers, "Strict-Transport-Security", "max-age=31536000; includeSubDomains");
        }
    }
}

fn status_mapped_error(
    error: &std::io::Error,
    options: &ServeServerOptions,
    secure: bool,
) -> PlannedResponse {
    let status_code = match error.kind() {
        std::io::ErrorKind::NotFound => 404,
        std::io::ErrorKind::PermissionDenied => 403,
        std::io::ErrorKind::InvalidInput => 404,
        _ => 500,
    };

    if 500 == status_code {
        text_response(500, "Internal Server Error", options, secure)
    } else if 403 == status_code {
        text_response(403, "Forbidden", options, secure)
    } else {
        text_response(404, "Not Found", options, secure)
    }
}

fn text_response(
    status_code: u16,
    body: &str,
    options: &ServeServerOptions,
    secure: bool,
) -> PlannedResponse {
    let mut headers = Vec::new();
    add_header(&mut headers, "Content-Type", "text/plain; charset=utf-8");
    add_header(&mut headers, "Cache-Control", "no-store");
    add_default_security_headers(&mut headers, options, secure);
    PlannedResponse {
        status_code,
        body: body.as_bytes().to_vec(),
        headers,
        content_length: Some(body.len()),
    }
}

fn headify(mut response: PlannedResponse, is_head: bool) -> PlannedResponse {
    if is_head {
        if response.content_length.is_none() {
            response.content_length = Some(response.body.len());
        }
        response.body.clear();
    }
    response
}

fn read_file_nofollow(path: &Path) -> std::io::Result<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::io::Read;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file =
            fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Requested path is not a regular file",
            ));
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    #[cfg(not(unix))]
    {
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Requested path is not a regular file",
            ));
        }
        fs::read(path)
    }
}

fn header_value(headers: &[Header], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|header| header.field.as_str().to_string().eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str().to_string())
}

fn add_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    if value.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
        return;
    }
    headers.push((name.to_string(), value.to_string()));
}

fn choose_precompressed_path(path: &Path, accept_encoding: Option<&str>) -> PathBuf {
    let extension =
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase());
    if extension.as_deref() == Some("br") || extension.as_deref() == Some("gz") {
        return path.to_path_buf();
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.to_path_buf();
    };
    let Some(header_value) = accept_encoding else {
        return path.to_path_buf();
    };

    if encoding_allowed(header_value, "br") {
        let br_path = path.with_file_name(format!("{}.br", file_name));
        if br_path.is_file() {
            return br_path;
        }
    }

    if encoding_allowed(header_value, "gzip") {
        let gz_path = path.with_file_name(format!("{}.gz", file_name));
        if gz_path.is_file() {
            return gz_path;
        }
    }

    path.to_path_buf()
}

fn encoding_allowed(header_value: &str, encoding: &str) -> bool {
    header_value.split(',').map(|item| item.trim().to_ascii_lowercase()).any(|item| {
        if item.starts_with('*') {
            return !item.contains("q=0");
        }
        if item.starts_with(encoding) {
            return !item.contains("q=0");
        }
        false
    })
}

fn parse_single_range(range_header: &str, total_len: usize) -> Result<Option<(usize, usize)>, ()> {
    let value = range_header.trim();
    if !value.starts_with("bytes=") {
        return Err(());
    }

    let range_value = &value[6..];
    if range_value.contains(',') {
        return Err(());
    }

    let parts: Vec<&str> = range_value.split('-').collect();
    if 2 != parts.len() || 0 == total_len {
        return Err(());
    }

    if parts[0].is_empty() {
        let suffix_len = parts[1].parse::<usize>().map_err(|_| ())?;
        if 0 == suffix_len {
            return Err(());
        }

        let start = total_len.saturating_sub(suffix_len);
        let end = total_len - 1;
        if start > end {
            return Err(());
        }
        return Ok(Some((start, end)));
    }

    let start = parts[0].parse::<usize>().map_err(|_| ())?;
    let end = if parts[1].is_empty() {
        total_len - 1
    } else {
        parts[1].parse::<usize>().map_err(|_| ())?
    };

    if start >= total_len || start > end {
        return Err(());
    }

    Ok(Some((start, end.min(total_len - 1))))
}

fn weak_etag(path: &Path, file_len: usize) -> String {
    let modified_secs = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("W/\"{:x}-{:x}\"", file_len, modified_secs)
}

fn if_none_match_matches(if_none_match: &str, etag: &str) -> bool {
    if "*" == if_none_match.trim() {
        return true;
    }
    if_none_match.split(',').map(|value| value.trim()).any(|candidate| candidate == etag)
}

fn guess_response_headers(path: &Path, file_bytes: &[u8]) -> (String, Option<&'static str>) {
    let (content_path, content_encoding) = split_content_path_and_encoding(path);
    let content_type = guess_content_type(&content_path, file_bytes);
    (content_type, content_encoding)
}

fn split_content_path_and_encoding(path: &Path) -> (PathBuf, Option<&'static str>) {
    let extension =
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("gz") => {
            if let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                (path.with_file_name(file_stem), Some("gzip"))
            } else {
                (path.to_path_buf(), Some("gzip"))
            }
        }
        Some("br") => {
            if let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                (path.with_file_name(file_stem), Some("br"))
            } else {
                (path.to_path_buf(), Some("br"))
            }
        }
        _ => (path.to_path_buf(), None),
    }
}

fn guess_content_type(path: &Path, file_bytes: &[u8]) -> String {
    let extension =
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase());

    if let Some(ext) = extension.as_deref() {
        if let Some(known_mime) = known_mime_for_extension(ext) {
            return known_mime.to_string();
        }
        // Unknown extensions are served as download-only bytes to avoid active-content sniffing.
        return "application/octet-stream".to_string();
    }

    let _ = file_bytes;
    // Extensionless files intentionally use octet-stream fallback.
    "application/octet-stream".to_string()
}

fn known_mime_for_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "html" | "htm" => Some("text/html; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
        "js" | "mjs" | "cjs" => Some("application/javascript; charset=utf-8"),
        "json" | "map" => Some("application/json; charset=utf-8"),
        "xml" => Some("application/xml; charset=utf-8"),
        "txt" | "text" | "md" | "csv" => Some("text/plain; charset=utf-8"),
        "wasm" => Some("application/wasm"),
        "pdf" => Some("application/pdf"),
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        "ico" => Some("image/x-icon"),
        "bmp" => Some("image/bmp"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        "ttf" => Some("font/ttf"),
        "otf" => Some("font/otf"),
        "mp3" => Some("audio/mpeg"),
        "wav" => Some("audio/wav"),
        "ogg" => Some("audio/ogg"),
        "mp4" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        _ => None,
    }
}

fn log_access(
    enabled: bool,
    method: &Method,
    url: &str,
    status_code: u16,
    response_bytes: usize,
    elapsed_ms: u128,
    secure: bool,
) {
    if !enabled {
        return;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    println!(
        "serve_access ts={} secure={} method={} path=\"{}\" status={} bytes={} elapsed_ms={}",
        timestamp, secure, method, url, status_code, response_bytes, elapsed_ms
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_range_supports_standard_and_suffix_ranges() {
        assert_eq!(parse_single_range("bytes=0-9", 100), Ok(Some((0, 9))));
        assert_eq!(parse_single_range("bytes=10-", 100), Ok(Some((10, 99))));
        assert_eq!(parse_single_range("bytes=-10", 100), Ok(Some((90, 99))));
    }

    #[test]
    fn parse_single_range_rejects_invalid_ranges() {
        assert!(parse_single_range("bytes=50-10", 100).is_err());
        assert!(parse_single_range("items=0-10", 100).is_err());
        assert!(parse_single_range("bytes=100-200", 100).is_err());
    }

    #[test]
    fn if_none_match_detects_exact_match() {
        assert!(if_none_match_matches("W/\"a-b\"", "W/\"a-b\""));
        assert!(if_none_match_matches("W/\"x-y\", W/\"a-b\"", "W/\"a-b\""));
        assert!(!if_none_match_matches("W/\"x-y\"", "W/\"a-b\""));
    }

    #[test]
    fn guess_content_type_is_case_insensitive_for_known_extensions() {
        assert_eq!(guess_content_type(Path::new("INDEX.HTML"), b""), "text/html; charset=utf-8");
        assert_eq!(guess_content_type(Path::new("asset.SVG"), b""), "image/svg+xml");
        assert_eq!(guess_content_type(Path::new("font.WOFF2"), b""), "font/woff2");
    }

    #[test]
    fn guess_content_type_covers_required_web_asset_mappings() {
        assert_eq!(guess_content_type(Path::new("index.html"), b""), "text/html; charset=utf-8");
        assert_eq!(guess_content_type(Path::new("styles.css"), b""), "text/css; charset=utf-8");
        assert_eq!(
            guess_content_type(Path::new("app.js"), b""),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            guess_content_type(Path::new("data.json"), b""),
            "application/json; charset=utf-8"
        );
        assert_eq!(guess_content_type(Path::new("image.png"), b""), "image/png");
        assert_eq!(guess_content_type(Path::new("image.jpg"), b""), "image/jpeg");
        assert_eq!(guess_content_type(Path::new("vector.svg"), b""), "image/svg+xml");
        assert_eq!(guess_content_type(Path::new("app.wasm"), b""), "application/wasm");
        assert_eq!(guess_content_type(Path::new("font.woff2"), b""), "font/woff2");
        assert_eq!(guess_content_type(Path::new("manual.pdf"), b""), "application/pdf");
        assert_eq!(guess_content_type(Path::new("notes.txt"), b""), "text/plain; charset=utf-8");
    }

    #[test]
    fn guess_content_type_uses_octet_stream_for_unknown_extension() {
        assert_eq!(
            guess_content_type(Path::new("payload.unknown"), b"not-a-known-format"),
            "application/octet-stream"
        );
    }

    #[test]
    fn guess_content_type_uses_octet_stream_for_extensionless_files() {
        assert_eq!(
            guess_content_type(Path::new("LICENSE"), b"plain text"),
            "application/octet-stream"
        );
    }

    #[test]
    fn guess_content_type_blocks_inferred_active_content_for_unknown_extension() {
        let html_bytes = b"<!DOCTYPE html><html><body>hello</body></html>";
        assert_eq!(
            guess_content_type(Path::new("payload.unknown"), html_bytes),
            "application/octet-stream"
        );
    }

    #[test]
    fn add_common_success_headers_sets_nosniff_for_file_responses() {
        let options = ServeServerOptions {
            index: "index.html".to_string(),
            hardened: false,
            cache_max_age: Some(300),
            access_log: false,
            tls_cert: None,
            tls_key: None,
        };
        let mut headers = Vec::new();
        add_common_success_headers(
            &mut headers,
            Path::new("index.html"),
            b"<html></html>",
            Some("W/\"b-1\""),
            &options,
            false,
        );

        let nosniff = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("x-content-type-options"))
            .map(|(_, value)| value.as_str());
        assert_eq!(nosniff, Some("nosniff"));
    }
}
