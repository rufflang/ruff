use ruff::ast::{Stmt, TypeAnnotation};
use ruff::interpreter::{Interpreter, Value};
use ruff::lexer::tokenize;
use ruff::parser::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("ruff_optional_typing_contract_{}_{}", prefix, nanos));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn parse_program(source: &str) -> Vec<Stmt> {
    let tokens = tokenize(source).expect("test source should tokenize");
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn run_program(source: &str) -> Interpreter {
    let program = parse_program(source);
    let mut interpreter = Interpreter::new();
    interpreter.eval_stmts(&program);
    interpreter
}

#[test]
fn v1_annotations_are_preserved_as_parser_metadata() {
    let source = r#"
		func labeled(value: int) -> string {
			return "ok"
		}
		let score: float := 3.14
		const active: bool := true
	"#;

    let program = parse_program(source);

    let mut saw_function = false;
    let mut saw_let = false;
    let mut saw_const = false;

    for stmt in program {
        match stmt {
            Stmt::FuncDef { param_types, return_type, .. } => {
                saw_function = true;
                assert!(matches!(param_types.first(), Some(Some(TypeAnnotation::Int))));
                assert!(matches!(return_type, Some(TypeAnnotation::String)));
            }
            Stmt::Let { type_annotation, .. } => {
                saw_let = true;
                assert!(matches!(type_annotation, Some(TypeAnnotation::Float)));
            }
            Stmt::Const { type_annotation, .. } => {
                saw_const = true;
                assert!(matches!(type_annotation, Some(TypeAnnotation::Bool)));
            }
            _ => {}
        }
    }

    assert!(saw_function, "expected to parse typed function");
    assert!(saw_let, "expected to parse typed let declaration");
    assert!(saw_const, "expected to parse typed const declaration");
}

#[test]
fn v1_annotations_do_not_enforce_runtime_types_by_default() {
    let source = r#"
		func typed_identity(value: int) -> int {
			return value
		}

		observed := typed_identity("dynamic-string")
	"#;

    let interpreter = run_program(source);
    assert!(
        interpreter.return_value.is_none(),
        "unexpected runtime error in default dynamic mode: {:?}",
        interpreter.return_value
    );
    assert!(matches!(interpreter.env.get("observed"), Some(Value::Str(_))));
}

#[test]
fn v1_optional_typing_warnings_are_interpreter_only() {
    let dir = unique_temp_dir("warning_boundary");
    let source_path = dir.join("typed_warning_boundary.ruff");
    fs::write(
        &source_path,
        r#"
            func typed_identity(value: int) -> int {
                return value
            }

            observed := typed_identity("dynamic-string")
            print(observed)
        "#,
    )
    .expect("failed to write test source");

    let interpreter_output = Command::new(env!("CARGO_BIN_EXE_ruff"))
        .arg("run")
        .arg("--interpreter")
        .arg(&source_path)
        .output()
        .expect("failed to execute interpreter mode");

    assert!(
        interpreter_output.status.success(),
        "interpreter mode should still execute dynamic code: stdout={} stderr={}",
        String::from_utf8_lossy(&interpreter_output.stdout),
        String::from_utf8_lossy(&interpreter_output.stderr)
    );
    let interpreter_stderr = String::from_utf8_lossy(&interpreter_output.stderr);
    assert!(
        interpreter_stderr.contains("Type checking warnings:"),
        "interpreter mode should emit optional typing warnings; stderr={interpreter_stderr}"
    );

    let vm_output = Command::new(env!("CARGO_BIN_EXE_ruff"))
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("failed to execute vm mode");

    assert!(
        vm_output.status.success(),
        "vm mode should execute dynamic code: stdout={} stderr={}",
        String::from_utf8_lossy(&vm_output.stdout),
        String::from_utf8_lossy(&vm_output.stderr)
    );
    let vm_stderr = String::from_utf8_lossy(&vm_output.stderr);
    assert!(
        !vm_stderr.contains("Type checking warnings:"),
        "vm mode should not emit interpreter-only type checker warnings; stderr={vm_stderr}"
    );
}
