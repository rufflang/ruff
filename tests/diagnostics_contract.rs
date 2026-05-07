use ruff::errors::{
    Diagnostic, DiagnosticSeverity, DiagnosticSubsystem, RuffError, SourceLocation,
    DIAGNOSTIC_CODE_CLI, DIAGNOSTIC_CODE_LEXER, DIAGNOSTIC_CODE_PARSER,
};
use ruff::lexer::tokenize_with_file;
use ruff::parser::Parser;

#[test]
fn diagnostic_human_render_includes_code_subsystem_and_location() {
    let diagnostic = Diagnostic::new(
        DIAGNOSTIC_CODE_CLI,
        DiagnosticSeverity::Error,
        DiagnosticSubsystem::Cli,
        "Invalid CLI invocation",
    )
    .with_location(Some("script.ruff".to_string()), 2, 8)
    .with_help("Use `ruff run <file>`");

    let rendered = diagnostic.render_human();
    assert!(rendered.contains("[RUFCLI001]"));
    assert!(rendered.contains("[cli]"));
    assert!(rendered.contains("script.ruff:2:8"));
    assert!(rendered.contains("help: Use `ruff run <file>`"));
}

#[test]
fn diagnostic_json_shape_includes_required_fields() {
    let diagnostic = Diagnostic::new(
        DIAGNOSTIC_CODE_CLI,
        DiagnosticSeverity::Error,
        DiagnosticSubsystem::Cli,
        "Invalid CLI invocation",
    )
    .with_location(Some("script.ruff".to_string()), 2, 8)
    .with_help("Use `ruff run <file>`");

    let json = diagnostic.to_json_value();
    assert_eq!(json["code"], "RUFCLI001");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["subsystem"], "cli");
    assert_eq!(json["message"], "Invalid CLI invocation");
    assert_eq!(json["help"], "Use `ruff run <file>`");
    assert_eq!(json["file"], "script.ruff");
    assert_eq!(json["line"], 2);
    assert_eq!(json["column"], 8);
}

#[test]
fn runtime_error_display_keeps_location_when_available() {
    let runtime_error = RuffError::runtime_error(
        "boom".to_string(),
        SourceLocation::with_file(3, 7, "main.ruff".to_string()),
    );

    let rendered = runtime_error.to_string();
    assert!(rendered.contains("[RUFRUN001]"));
    assert!(rendered.contains("main.ruff:3:7"));
}

#[test]
fn lexer_diagnostic_converts_to_stable_code() {
    let diagnostics = tokenize_with_file("let value := @", Some("fixture.ruff"))
        .expect_err("source should produce lexical diagnostics");
    let first = diagnostics.first().expect("diagnostics should not be empty");
    let converted = first.to_diagnostic();

    assert!(converted.code.starts_with(DIAGNOSTIC_CODE_LEXER));
    assert_eq!(converted.subsystem, DiagnosticSubsystem::Lexer);
    assert_eq!(converted.file.as_deref(), Some("fixture.ruff"));
}

#[test]
fn parser_diagnostic_converts_to_stable_code() {
    let tokens = tokenize_with_file("print((1 + 2", Some("broken.ruff"))
        .expect("source should tokenize");
    let mut parser = Parser::new(tokens);
    let parse_output = parser.parse_with_diagnostics();
    let first = parse_output
        .diagnostics
        .first()
        .expect("parse diagnostics should not be empty");
    let converted = first.to_diagnostic(Some("broken.ruff"));

    assert_eq!(converted.code, DIAGNOSTIC_CODE_PARSER);
    assert_eq!(converted.subsystem, DiagnosticSubsystem::Parser);
    assert_eq!(converted.file.as_deref(), Some("broken.ruff"));
}
