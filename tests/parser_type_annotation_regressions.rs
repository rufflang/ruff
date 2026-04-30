use ruff::lexer::tokenize;
use ruff::parser::Parser;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn parse_without_panic(source: &str) {
    let parse_result = catch_unwind(AssertUnwindSafe(|| {
        let tokens = tokenize(source);
        let mut parser = Parser::new(tokens);
        parser.parse()
    }));

    assert!(
        parse_result.is_ok(),
        "parser panicked for malformed user input: {}",
        source
    );
}

#[test]
fn malformed_result_type_annotation_missing_comma_does_not_panic() {
    let source = "func broken(value: Result<int string>) { return value }";
    parse_without_panic(source);
}

#[test]
fn malformed_result_type_annotation_missing_closer_does_not_panic() {
    let source = "func broken(value: Result<int, string) { return value }";
    parse_without_panic(source);
}

#[test]
fn malformed_option_type_annotation_missing_closer_does_not_panic() {
    let source = "func broken(value: Option<int) { return value }";
    parse_without_panic(source);
}
