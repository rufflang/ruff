use ruff::ast::{Stmt, TypeAnnotation};
use ruff::interpreter::{Interpreter, Value};
use ruff::lexer::tokenize;
use ruff::parser::Parser;

fn parse_program(source: &str) -> Vec<Stmt> {
	let tokens = tokenize(source);
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
