use ruff::lsp_server::{LspServer, LspServerConfig};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Fixture {
    name: String,
    messages: Vec<Value>,
    expected: Vec<Value>,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lsp_fixtures")
}

fn load_fixture(path: &PathBuf) -> Fixture {
    let raw = fs::read_to_string(path).expect("failed to read fixture file");
    let parsed: Value = serde_json::from_str(&raw).expect("fixture should be valid json");

    let name = parsed
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unnamed_fixture")
        .to_string();

    let messages = parsed
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let expected = parsed
        .get("expected")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    Fixture {
        name,
        messages,
        expected,
    }
}

fn normalize_json(value: Value) -> Value {
    value
}

#[test]
fn protocol_fixtures_match_expected_responses() {
    let mut paths: Vec<PathBuf> = fs::read_dir(fixture_dir())
        .expect("failed to list fixture directory")
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|value| value == "json")
                .unwrap_or(false)
        })
        .collect();
    paths.sort();

    assert!(!paths.is_empty(), "expected at least one LSP fixture file");

    for path in paths.iter() {
        let fixture = load_fixture(path);
        let mut server = LspServer::new(LspServerConfig::default());
        let mut actual = Vec::new();

        for message in fixture.messages.iter() {
            let responses = server.process_message(message);
            actual.extend(responses.into_iter());
        }

        let normalized_actual: Vec<Value> = actual.into_iter().map(normalize_json).collect();
        let normalized_expected: Vec<Value> = fixture
            .expected
            .clone()
            .into_iter()
            .map(normalize_json)
            .collect();

        assert_eq!(
            normalized_actual,
            normalized_expected,
            "fixture '{}' produced unexpected responses",
            fixture.name
        );
    }
}
