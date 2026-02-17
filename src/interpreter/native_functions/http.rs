// File: src/interpreter/native_functions/http.rs
//
// HTTP client native functions

use crate::builtins;
use crate::interpreter::{DictMap, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "parallel_http" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "parallel_http() expects 1 argument (urls), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Array(urls)) = arg_values.first() {
                let url_strings: Vec<String> = urls
                    .iter()
                    .filter_map(
                        |v| if let Value::Str(s) = v { Some(s.as_ref().clone()) } else { None },
                    )
                    .collect();

                let mut handles = Vec::new();
                for url in url_strings {
                    let handle = std::thread::spawn(move || -> Result<(u16, String), String> {
                        match reqwest::blocking::get(&url) {
                            Ok(response) => {
                                let status = response.status().as_u16();
                                let body = response.text().unwrap_or_default();
                                Ok((status, body))
                            }
                            Err(e) => Err(format!("HTTP GET failed: {}", e)),
                        }
                    });
                    handles.push(handle);
                }

                let mut results = Vec::new();
                for handle in handles {
                    match handle.join() {
                        Ok(Ok((status, body))) => {
                            let mut result_map = DictMap::default();
                            result_map.insert("status".into(), Value::Int(status as i64));
                            result_map.insert("body".into(), Value::Str(Arc::new(body)));
                            results.push(Value::Dict(Arc::new(result_map)));
                        }
                        Ok(Err(e)) => results.push(Value::Error(e)),
                        Err(_) => results.push(Value::Error("Thread panicked".to_string())),
                    }
                }

                Value::Array(Arc::new(results))
            } else {
                Value::Error("parallel_http requires an array of URL strings".to_string())
            }
        }

        "http_get" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "http_get() expects 1 argument (url), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Str(url)) = arg_values.first() {
                match builtins::http_get(url.as_ref()) {
                    Ok(result_map) => Value::Dict(Arc::new(result_map)),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_get requires a URL string".to_string())
            }
        }

        "http_post" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "http_post() expects 2 arguments (url, body), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Str(url)), Some(Value::Str(body))) =
                (arg_values.first(), arg_values.get(1))
            {
                match builtins::http_post(url.as_ref(), body.as_ref()) {
                    Ok(result_map) => Value::Dict(Arc::new(result_map)),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_post requires URL and JSON body strings".to_string())
            }
        }

        "http_put" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "http_put() expects 2 arguments (url, body), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Str(url)), Some(Value::Str(body))) =
                (arg_values.first(), arg_values.get(1))
            {
                match builtins::http_put(url.as_ref(), body.as_ref()) {
                    Ok(result_map) => Value::Dict(Arc::new(result_map)),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_put requires URL and JSON body strings".to_string())
            }
        }

        "http_delete" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "http_delete() expects 1 argument (url), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Str(url)) = arg_values.first() {
                match builtins::http_delete(url.as_ref()) {
                    Ok(result_map) => Value::Dict(Arc::new(result_map)),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_delete requires a URL string".to_string())
            }
        }

        "http_get_binary" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "http_get_binary() expects 1 argument (url), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Str(url)) = arg_values.first() {
                match builtins::http_get_binary(url.as_ref()) {
                    Ok(bytes) => Value::Bytes(bytes),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_get_binary requires a URL string".to_string())
            }
        }

        "http_get_stream" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "http_get_stream() expects 1 argument (url), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Str(url)) = arg_values.first() {
                match builtins::http_get_stream(url.as_ref()) {
                    Ok(bytes) => Value::Bytes(bytes),
                    Err(error) => Value::Error(error),
                }
            } else {
                Value::Error("http_get_stream requires a URL string".to_string())
            }
        }

        "http_server" => {
            if arg_values.len() != 1 {
                return Some(Value::Error(format!(
                    "http_server() expects 1 argument (port), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Int(port)) = arg_values.first() {
                Value::HttpServer { port: *port as u16, routes: Vec::new() }
            } else {
                Value::Error("http_server requires a port number".to_string())
            }
        }

        "set_header" => {
            if arg_values.len() != 3 {
                return Some(Value::Error(format!(
                    "set_header() expects 3 arguments (response, name, value), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(response), Some(Value::Str(key)), Some(Value::Str(value))) =
                (arg_values.first(), arg_values.get(1), arg_values.get(2))
            {
                if let Value::HttpResponse { status, body, headers } = response {
                    let mut new_headers = headers.clone();
                    new_headers.insert(key.as_ref().to_string(), value.as_ref().to_string());
                    Value::HttpResponse {
                        status: *status,
                        body: body.clone(),
                        headers: new_headers,
                    }
                } else {
                    Value::Error(
                        "set_header requires an HTTP response as first argument".to_string(),
                    )
                }
            } else {
                Value::Error(
                    "set_header requires response, header name, and header value".to_string(),
                )
            }
        }

        "set_headers" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "set_headers() expects 2 arguments (response, headers), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(response), Some(Value::Dict(headers_dict))) =
                (arg_values.first(), arg_values.get(1))
            {
                if let Value::HttpResponse { status, body, headers } = response {
                    let mut new_headers = headers.clone();
                    for (key, value) in headers_dict.iter() {
                        if let Value::Str(header_value) = value {
                            new_headers.insert(
                                key.as_ref().to_string(),
                                header_value.as_ref().to_string(),
                            );
                        }
                    }
                    Value::HttpResponse {
                        status: *status,
                        body: body.clone(),
                        headers: new_headers,
                    }
                } else {
                    Value::Error(
                        "set_headers requires an HTTP response as first argument".to_string(),
                    )
                }
            } else {
                Value::Error("set_headers requires response and headers dictionary".to_string())
            }
        }

        "http_response" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "http_response() expects 2 arguments (status, body), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Int(status)), Some(Value::Str(body))) =
                (arg_values.first(), arg_values.get(1))
            {
                Value::HttpResponse {
                    status: *status as u16,
                    body: body.as_ref().to_string(),
                    headers: HashMap::new(),
                }
            } else {
                Value::Error("http_response requires status code and body string".to_string())
            }
        }

        "json_response" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "json_response() expects 2 arguments (status, data), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Int(status)), Some(data)) = (arg_values.first(), arg_values.get(1))
            {
                let body = builtins::to_json(data).unwrap_or_else(|_| "{}".to_string());
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                Value::HttpResponse { status: *status as u16, body, headers }
            } else {
                Value::Error("json_response requires status code and data".to_string())
            }
        }

        "html_response" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "html_response() expects 2 arguments (status, html), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Int(status)), Some(Value::Str(html))) =
                (arg_values.first(), arg_values.get(1))
            {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
                Value::HttpResponse {
                    status: *status as u16,
                    body: html.as_ref().to_string(),
                    headers,
                }
            } else {
                Value::Error("html_response requires status code and HTML string".to_string())
            }
        }

        "redirect_response" => {
            if !(1..=2).contains(&arg_values.len()) {
                return Some(Value::Error(format!(
                    "redirect_response() expects 1-2 arguments (url, headers?), got {}",
                    arg_values.len()
                )));
            }

            if let Some(Value::Str(url)) = arg_values.first() {
                let mut headers = HashMap::new();
                headers.insert("Location".to_string(), url.as_ref().to_string());

                if let Some(Value::Dict(extra_headers)) = arg_values.get(1) {
                    for (key, value) in extra_headers.iter() {
                        if let Value::Str(header_value) = value {
                            headers.insert(
                                key.as_ref().to_string(),
                                header_value.as_ref().to_string(),
                            );
                        }
                    }
                }

                Value::HttpResponse {
                    status: 302,
                    body: format!("Redirecting to {}", url.as_ref()),
                    headers,
                }
            } else {
                Value::Error("redirect_response requires a URL string".to_string())
            }
        }

        "jwt_encode" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "jwt_encode() expects 2 arguments (payload, secret), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Dict(payload)), Some(Value::Str(secret))) =
                (arg_values.first(), arg_values.get(1))
            {
                match builtins::jwt_encode(payload, secret) {
                    Ok(token) => Value::Str(Arc::new(token)),
                    Err(e) => Value::Error(e),
                }
            } else {
                Value::Error(
                    "jwt_encode requires a dictionary payload and secret key string".to_string(),
                )
            }
        }

        "jwt_decode" => {
            if arg_values.len() != 2 {
                return Some(Value::Error(format!(
                    "jwt_decode() expects 2 arguments (token, secret), got {}",
                    arg_values.len()
                )));
            }

            if let (Some(Value::Str(token)), Some(Value::Str(secret))) =
                (arg_values.first(), arg_values.get(1))
            {
                match builtins::jwt_decode(token, secret) {
                    Ok(payload) => Value::Dict(Arc::new(payload)),
                    Err(e) => Value::Error(e),
                }
            } else {
                Value::Error("jwt_decode requires a token string and secret key string".to_string())
            }
        }

        "oauth2_auth_url" => {
            if arg_values.len() != 4 {
                return Some(Value::Error(format!(
                    "oauth2_auth_url() expects 4 arguments (client_id, redirect_uri, auth_url, scope), got {}",
                    arg_values.len()
                )));
            }

            if let (
                Some(Value::Str(client_id)),
                Some(Value::Str(redirect_uri)),
                Some(Value::Str(auth_url)),
                Some(Value::Str(scope)),
            ) = (arg_values.first(), arg_values.get(1), arg_values.get(2), arg_values.get(3))
            {
                Value::Str(Arc::new(builtins::oauth2_auth_url(
                    client_id.as_ref(),
                    redirect_uri.as_ref(),
                    auth_url.as_ref(),
                    scope.as_ref(),
                )))
            } else {
                Value::Error(
                    "oauth2_auth_url requires client_id, redirect_uri, auth_url, and scope strings"
                        .to_string(),
                )
            }
        }

        "oauth2_get_token" => {
            if arg_values.len() != 5 {
                return Some(Value::Error(format!(
                    "oauth2_get_token() expects 5 arguments (code, client_id, client_secret, token_url, redirect_uri), got {}",
                    arg_values.len()
                )));
            }

            if let (
                Some(Value::Str(code)),
                Some(Value::Str(client_id)),
                Some(Value::Str(client_secret)),
                Some(Value::Str(token_url)),
                Some(Value::Str(redirect_uri)),
            ) = (
                arg_values.first(),
                arg_values.get(1),
                arg_values.get(2),
                arg_values.get(3),
                arg_values.get(4),
            ) {
                match builtins::oauth2_get_token(
                    code,
                    client_id,
                    client_secret,
                    token_url,
                    redirect_uri,
                ) {
                    Ok(token_data) => Value::Dict(Arc::new(token_data)),
                    Err(e) => Value::Error(e),
                }
            } else {
                Value::Error(
                    "oauth2_get_token requires code, client_id, client_secret, token_url, and redirect_uri strings"
                        .to_string(),
                )
            }
        }

        _ => return None,
    };

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_value(value: &str) -> Value {
        Value::Str(Arc::new(value.to_string()))
    }

    #[test]
    fn test_http_response_helpers_and_header_mutation() {
        let response = handle("http_response", &[Value::Int(200), str_value("ok")]).unwrap();
        match response {
            Value::HttpResponse { status, body, headers } => {
                assert_eq!(status, 200);
                assert_eq!(body, "ok");
                assert!(headers.is_empty());
            }
            _ => panic!("Expected HttpResponse from http_response"),
        }

        let json_response = handle("json_response", &[Value::Int(201), Value::Int(7)]).unwrap();
        match json_response {
            Value::HttpResponse { status, headers, .. } => {
                assert_eq!(status, 201);
                assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
            }
            _ => panic!("Expected HttpResponse from json_response"),
        }

        let html_response =
            handle("html_response", &[Value::Int(202), str_value("<h1>x</h1>")]).unwrap();
        match html_response {
            Value::HttpResponse { status, headers, .. } => {
                assert_eq!(status, 202);
                assert_eq!(
                    headers.get("Content-Type"),
                    Some(&"text/html; charset=utf-8".to_string())
                );
            }
            _ => panic!("Expected HttpResponse from html_response"),
        }

        let mut headers_dict = DictMap::default();
        headers_dict.insert("X-App".into(), str_value("ruff"));

        let set_header_result = handle(
            "set_header",
            &[
                Value::HttpResponse {
                    status: 200,
                    body: "ok".to_string(),
                    headers: HashMap::new(),
                },
                str_value("X-Test"),
                str_value("true"),
            ],
        )
        .unwrap();
        assert!(
            matches!(set_header_result, Value::HttpResponse { headers, .. } if headers.get("X-Test") == Some(&"true".to_string()))
        );

        let set_headers_result = handle(
            "set_headers",
            &[
                Value::HttpResponse {
                    status: 200,
                    body: "ok".to_string(),
                    headers: HashMap::new(),
                },
                Value::Dict(Arc::new(headers_dict)),
            ],
        )
        .unwrap();
        assert!(
            matches!(set_headers_result, Value::HttpResponse { headers, .. } if headers.get("X-App") == Some(&"ruff".to_string()))
        );
    }

    #[test]
    fn test_redirect_and_server_helpers() {
        let mut extra_headers = DictMap::default();
        extra_headers.insert("Cache-Control".into(), str_value("no-cache"));

        let redirect = handle(
            "redirect_response",
            &[str_value("https://example.com"), Value::Dict(Arc::new(extra_headers))],
        )
        .unwrap();
        assert!(
            matches!(redirect, Value::HttpResponse { status, headers, .. } if status == 302 && headers.get("Location") == Some(&"https://example.com".to_string()) && headers.get("Cache-Control") == Some(&"no-cache".to_string()))
        );

        let server = handle("http_server", &[Value::Int(8080)]).unwrap();
        assert!(matches!(server, Value::HttpServer { port, .. } if port == 8080));
    }

    #[test]
    fn test_http_argument_shape_contract_errors() {
        let get_error = handle("http_get", &[Value::Int(1)]).unwrap();
        assert!(
            matches!(get_error, Value::Error(message) if message.contains("http_get requires a URL string"))
        );

        let post_error = handle("http_post", &[str_value("https://example.com")]).unwrap();
        assert!(
            matches!(post_error, Value::Error(message) if message.contains("http_post() expects 2 arguments"))
        );

        let get_extra_error =
            handle("http_get", &[str_value("https://example.com"), str_value("extra")]).unwrap();
        assert!(
            matches!(get_extra_error, Value::Error(message) if message.contains("http_get() expects 1 argument"))
        );

        let set_header_error =
            handle("set_header", &[Value::Int(1), str_value("k"), str_value("v")]).unwrap();
        assert!(
            matches!(set_header_error, Value::Error(message) if message.contains("requires an HTTP response as first argument"))
        );
    }
}
