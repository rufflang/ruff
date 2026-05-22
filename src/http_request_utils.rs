use std::collections::HashMap;

/// Split a request URL into a path, parsed query parameters, and raw query string.
///
/// Query parsing is intentionally lexical-only:
/// - no URL decoding
/// - empty key pairs are ignored
/// - `key` without `=` maps to empty-string value
pub fn split_http_path_and_query(url: &str) -> (String, HashMap<String, String>, String) {
    if let Some((path, raw_query)) = url.split_once('?') {
        (path.to_string(), parse_http_query_params(raw_query), raw_query.to_string())
    } else {
        (url.to_string(), HashMap::new(), String::new())
    }
}

fn parse_http_query_params(raw_query: &str) -> HashMap<String, String> {
    let mut query_params = HashMap::new();

    for pair in raw_query.split('&') {
        if pair.is_empty() {
            continue;
        }

        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };

        if key.is_empty() {
            continue;
        }

        query_params.insert(key.to_string(), value.to_string());
    }

    query_params
}

#[cfg(test)]
mod tests {
    use super::split_http_path_and_query;
    use std::collections::HashMap;

    #[test]
    fn split_http_path_and_query_without_query_returns_empty_metadata() {
        let (path, query_map, raw_query) = split_http_path_and_query("/health");
        assert_eq!(path, "/health");
        assert!(query_map.is_empty());
        assert_eq!(raw_query, "");
    }

    #[test]
    fn split_http_path_and_query_parses_pairs_without_decoding() {
        let (path, query_map, raw_query) = split_http_path_and_query("/search?q=ruff%20lang&limit=10");
        assert_eq!(path, "/search");
        assert_eq!(raw_query, "q=ruff%20lang&limit=10");

        let mut expected = HashMap::new();
        expected.insert("q".to_string(), "ruff%20lang".to_string());
        expected.insert("limit".to_string(), "10".to_string());
        assert_eq!(query_map, expected);
    }

    #[test]
    fn split_http_path_and_query_ignores_empty_keys_and_accepts_missing_values() {
        let (_, query_map, raw_query) = split_http_path_and_query("/x?=skip&flag&name=ruff&&empty=");
        assert_eq!(raw_query, "=skip&flag&name=ruff&&empty=");
        assert_eq!(query_map.get("flag").map(String::as_str), Some(""));
        assert_eq!(query_map.get("name").map(String::as_str), Some("ruff"));
        assert_eq!(query_map.get("empty").map(String::as_str), Some(""));
        assert!(!query_map.contains_key(""));
    }
}
