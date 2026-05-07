use ruff::compiler::Compiler;
use ruff::interpreter::{Environment, Interpreter, Value};
use ruff::lexer::tokenize;
use ruff::parser::Parser;
use ruff::vm::{VmExecutionResult, VM};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn run_interpreter(code: &str) -> Interpreter {
    let tokens = tokenize(code).expect("test source should tokenize");
    let mut parser = Parser::new(tokens);
    let program = parser.parse();
    let mut interp = Interpreter::new();
    interp.eval_stmts(&program);
    interp
}

fn interpreter_error(code: &str) -> Option<String> {
    let interp = run_interpreter(code);
    match interp.return_value {
        Some(Value::Error(error)) => Some(error),
        Some(Value::Return(value)) => match *value {
            Value::Error(error) => Some(error),
            _ => None,
        },
        _ => None,
    }
}

fn run_vm(code: &str, env: Arc<Mutex<Environment>>) -> Result<(), String> {
    let tokens = tokenize(code).map_err(|diagnostics| {
        diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.clone())
            .unwrap_or_else(|| "unknown lexer error".to_string())
    })?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    let mut compiler = Compiler::new();
    let chunk = compiler.compile(&program)?;

    let mut vm = VM::new();
    vm.set_globals(env);

    match vm.execute_until_suspend(chunk)? {
        VmExecutionResult::Completed => Ok(()),
        VmExecutionResult::Suspended { .. } => {
            vm.run_scheduler_until_complete_with_timeout(Duration::from_millis(5_000))
        }
    }
}

fn vm_env_with_builtins() -> Arc<Mutex<Environment>> {
    let interp = Interpreter::new();
    Arc::new(Mutex::new(interp.env))
}

fn assert_interpreter_and_vm_error_contains(script: &str, expected: &str) {
    let interp_error =
        interpreter_error(script).expect("expected interpreter execution to report an error");
    assert!(
        interp_error.contains(expected),
        "expected interpreter error containing {:?}, got {:?}",
        expected,
        interp_error
    );

    let vm_env = vm_env_with_builtins();
    let vm_error = run_vm(script, vm_env).expect_err("expected VM execution to report an error");
    assert!(
        vm_error.contains(expected),
        "expected VM error containing {:?}, got {:?}",
        expected,
        vm_error
    );
}

fn unique_spawn_key() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    format!("vm_interp_parity_spawn_{}_{}", std::process::id(), nanos)
}

#[test]
fn vm_and_interpreter_match_struct_method_behavior_contract() {
    let script = r#"
        struct Vec2 {
            x: float,
            y: float,

            func doubled(self) {
                return Vec2 { x: self.x * 2.0, y: self.y * 2.0 }
            }
        }

        vector := Vec2 { x: 2.0, y: 3.0 }
        doubled := vector.doubled()
        struct_x := doubled.x
        struct_y := doubled.y
        struct_ok := struct_x == 4.0 && struct_y == 6.0
    "#;

    let interp = run_interpreter(script);
    assert!(
        interp.return_value.is_none(),
        "unexpected interpreter struct-method behavior: {:?}",
        interp.return_value
    );
    assert!(
        matches!(interp.env.get("struct_ok"), Some(Value::Bool(true))),
        "interpreter struct values: struct_x={:?}, struct_y={:?}, struct_ok={:?}",
        interp.env.get("struct_x"),
        interp.env.get("struct_y"),
        interp.env.get("struct_ok")
    );

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(script, vm_env.clone());
    assert!(vm_result.is_ok(), "unexpected vm struct-method behavior: {:?}", vm_result);
    let vm_globals = vm_env.lock().expect("failed to lock vm globals");
    assert!(matches!(vm_globals.get("struct_ok"), Some(Value::Bool(true))));
}

#[test]
fn vm_and_interpreter_match_spread_destructuring_surface() {
    let script = r#"
        values := [1, 2, 3, 4]
        let [first, second, third, fourth] := values
        profile := {"a": 1, "b": 2, "c": 3}
        let {a, b, c} := profile
        destructuring_ok := first == 1 && second == 2 && third == 3 && fourth == 4 && a == 1 && b == 2 && c == 3
        spread_array := [0, ...values, 9]
        spread_dict := {"x": 0, ...profile, "z": 9}
        spread_ok := spread_array[2] == 2 && spread_dict["b"] == 2
    "#;

    let interp = run_interpreter(&script);
    assert!(
        interp.return_value.is_none(),
        "interpreter returned runtime error: {:?}",
        interp.return_value
    );
    assert!(matches!(interp.env.get("destructuring_ok"), Some(Value::Bool(true))));
    assert!(matches!(interp.env.get("spread_ok"), Some(Value::Bool(true))));

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(script, vm_env.clone());
    assert!(vm_result.is_ok(), "vm execution failed: {:?}", vm_result.err());

    let vm_globals = vm_env.lock().expect("failed to lock vm globals");
    assert!(
        matches!(vm_globals.get("destructuring_ok"), Some(Value::Bool(true))),
        "vm destructuring values: first={:?}, second={:?}, third={:?}, fourth={:?}, a={:?}, b={:?}, c={:?}, destructuring_ok={:?}",
        vm_globals.get("first"),
        vm_globals.get("second"),
        vm_globals.get("third"),
        vm_globals.get("fourth"),
        vm_globals.get("a"),
        vm_globals.get("b"),
        vm_globals.get("c"),
        vm_globals.get("destructuring_ok")
    );
    assert!(matches!(vm_globals.get("spread_ok"), Some(Value::Bool(true))));
}

#[test]
fn vm_and_interpreter_match_enum_match_binding_surface() {
    let script = r#"
        result_value := Result::Ok(42)
        label_value := Option::Some("ready")

        match result_value {
            case Result::Ok(v): { matched_ok := v }
            case Result::Err(e): { matched_ok := -1 }
            default: { matched_ok := -999 }
        }

        match label_value {
            case Option::Some(text): { matched_label := text }
            case Option::None: { matched_label := "none" }
            default: { matched_label := "none" }
        }

        match_ok := matched_ok == 42 && matched_label == "ready"
    "#;

    let interp = run_interpreter(script);
    assert!(
        interp.return_value.is_none(),
        "interpreter returned runtime error: {:?}",
        interp.return_value
    );
    assert!(
        matches!(interp.env.get("matched_ok"), Some(Value::Int(42)))
            && matches!(interp.env.get("matched_label"), Some(Value::Str(v)) if v.as_ref() == "ready")
            && matches!(interp.env.get("match_ok"), Some(Value::Bool(true))),
        "expected interpreter tag-style match binding support, got matched_ok={:?}, matched_label={:?}, match_ok={:?}",
        interp.env.get("matched_ok"),
        interp.env.get("matched_label"),
        interp.env.get("match_ok")
    );

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(script, vm_env.clone());
    assert!(vm_result.is_ok(), "vm execution failed: {:?}", vm_result.err());

    let vm_globals = vm_env.lock().expect("failed to lock vm globals");
    assert!(
        matches!(vm_globals.get("matched_ok"), Some(Value::Int(42)))
            && matches!(vm_globals.get("matched_label"), Some(Value::Str(v)) if v.as_ref() == "ready")
            && matches!(vm_globals.get("match_ok"), Some(Value::Bool(true))),
        "expected VM tag-style match binding support, got matched_ok={:?}, matched_label={:?}, match_ok={:?}",
        vm_globals.get("matched_ok"),
        vm_globals.get("matched_label"),
        vm_globals.get("match_ok")
    );
}

#[test]
fn vm_and_interpreter_error_on_missing_string_map_key() {
    let script = r#"
        profile := {"name": "ruff"}
        return profile["missing"]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Missing map key");
}

#[test]
fn vm_and_interpreter_error_on_missing_integer_map_key() {
    let script = r#"
        scores := {}
        scores[0] := 10
        return scores[1]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Missing map key");
}

#[test]
fn vm_and_interpreter_error_on_nested_missing_map_key() {
    let script = r#"
        outer := {"inner": {"present": 7}}
        return outer["inner"]["missing"]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Missing map key");
}

#[test]
fn vm_and_interpreter_error_on_invalid_map_key_type() {
    let script = r#"
        profile := {"name": "ruff"}
        return profile[true]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Invalid index operation");
}

#[test]
fn vm_and_interpreter_error_on_out_of_bounds_array_index() {
    let script = r#"
        values := [10, 20]
        return values[5]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Index out of bounds");
}

#[test]
fn vm_and_interpreter_error_on_out_of_bounds_string_index() {
    let script = r#"
        label := "ruff"
        return label[10]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Index out of bounds");
}

#[test]
fn vm_and_interpreter_error_on_indexing_non_indexable_value() {
    let script = r#"
        value := 42
        return value[0]
    "#;

    assert_interpreter_and_vm_error_contains(script, "Invalid index operation");
}

#[test]
fn vm_and_interpreter_error_on_invalid_index_assignment_target() {
    let script = r#"
        value := 42
        value[0] := 7
        return value
    "#;

    assert_interpreter_and_vm_error_contains(script, "Invalid index assignment");
}

#[test]
fn vm_and_interpreter_error_on_unsupported_binary_operation() {
    let script = r#"
        return "left" - "right"
    "#;

    assert_interpreter_and_vm_error_contains(script, "Invalid binary operation");
}

#[test]
fn vm_and_interpreter_error_on_unsupported_unary_operation() {
    let script = r#"
        return -true
    "#;

    assert_interpreter_and_vm_error_contains(script, "Invalid unary operation");
}

#[test]
fn vm_and_interpreter_match_valid_index_assignment_success_path() {
    let script = r#"
        values := [2, 4, 6]
        values[1] := values[1] + 3
        index_assignment_ok := values[1] == 7
    "#;

    assert_interpreter_and_vm_bool(script, "index_assignment_ok");
}

#[test]
fn vm_and_interpreter_error_on_undefined_top_level_identifier() {
    let script = r#"
        return missing_top_level_name
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_top_level_name");
}

#[test]
fn vm_and_interpreter_error_on_undefined_identifier_in_binary_expression() {
    let script = r#"
        return missing_operand + 1
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_operand");
}

#[test]
fn vm_and_interpreter_error_on_undefined_identifier_in_condition() {
    let script = r#"
        if missing_condition {
            condition_reached := true
        }
        return false
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_condition");
}

#[test]
fn vm_and_interpreter_error_on_undefined_identifier_inside_function() {
    let script = r#"
        func read_missing() {
            return missing_inside_function
        }

        return read_missing()
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_inside_function");
}

#[test]
fn vm_and_interpreter_error_on_undefined_identifier_inside_closure() {
    let script = r#"
        func make_reader() {
            captured := 1
            reader := func() {
                return missing_inside_closure
            }
            return reader
        }

        read := make_reader()
        return read()
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_inside_closure");
}

#[test]
fn vm_and_interpreter_error_on_undefined_method_receiver() {
    let script = r#"
        return missing_receiver.collect()
    "#;

    assert_interpreter_and_vm_error_contains(script, "Undefined variable: missing_receiver");
}

#[test]
fn vm_and_interpreter_error_on_unknown_method_member() {
    let script = r#"
        values := [1, 2, 3]
        return values.definitely_missing_method()
    "#;

    assert_interpreter_and_vm_error_contains(script, "method");
}

#[test]
fn vm_and_interpreter_error_on_function_arity_too_few() {
    let script = r#"
        func add(a, b) {
            return a + b
        }

        return add(1)
    "#;

    assert_interpreter_and_vm_error_contains(script, "add expects 2 arguments, got 1");
}

#[test]
fn vm_and_interpreter_error_on_function_arity_too_many() {
    let script = r#"
        func add(a, b) {
            return a + b
        }

        return add(1, 2, 3)
    "#;

    assert_interpreter_and_vm_error_contains(script, "add expects 2 arguments, got 3");
}

#[test]
fn vm_and_interpreter_error_on_closure_arity_mismatch() {
    let script = r#"
        multiplier := func(value) {
            return value * 2
        }

        return multiplier()
    "#;

    assert_interpreter_and_vm_error_contains(script, "expects 1 arguments, got 0");
}

#[test]
fn vm_and_interpreter_error_on_closure_arity_too_many() {
    let script = r#"
        multiplier := func(value) {
            return value * 2
        }

        return multiplier(1, 2)
    "#;

    assert_interpreter_and_vm_error_contains(script, "expects 1 arguments, got 2");
}

#[test]
fn vm_and_interpreter_error_on_method_arity_mismatch() {
    let script = r#"
        struct Counter {
            count: int,

            func bump(self, delta) {
                return self.count + delta
            }
        }

        counter := Counter { count: 4 }
        return counter.bump()
    "#;

    assert_interpreter_and_vm_error_contains(script, "Counter.bump expects 1 arguments, got 0");
}

#[test]
fn vm_and_interpreter_error_on_method_arity_too_many() {
    let script = r#"
        struct Counter {
            count: int,

            func bump(self, delta) {
                return self.count + delta
            }
        }

        counter := Counter { count: 4 }
        return counter.bump(1, 2)
    "#;

    assert_interpreter_and_vm_error_contains(script, "Counter.bump expects 1 arguments, got 2");
}

#[test]
fn vm_and_interpreter_error_on_async_function_arity_mismatch() {
    let script = r#"
        async func combine(a, b) {
            return a + b
        }

        promise := combine(1)
        return await promise
    "#;

    assert_interpreter_and_vm_error_contains(script, "combine expects 2 arguments, got 1");
}

#[test]
fn vm_and_interpreter_error_on_async_function_arity_too_many() {
    let script = r#"
        async func combine(a, b) {
            return a + b
        }

        promise := combine(1, 2, 3)
        return await promise
    "#;

    assert_interpreter_and_vm_error_contains(script, "combine expects 2 arguments, got 3");
}

#[test]
fn vm_and_interpreter_error_on_generator_arity_mismatch() {
    let script = r#"
        func* emit_twice(value) {
            yield value
            yield value
        }

        total := 0
        for item in emit_twice() {
            total := total + item
        }

        return total
    "#;

    assert_interpreter_and_vm_error_contains(script, "emit_twice expects 1 arguments, got 0");
}

#[test]
fn vm_and_interpreter_error_on_generator_arity_too_many() {
    let script = r#"
        func* emit_twice(value) {
            yield value
            yield value
        }

        total := 0
        for item in emit_twice(1, 2) {
            total := total + item
        }

        return total
    "#;

    assert_interpreter_and_vm_error_contains(script, "emit_twice expects 1 arguments, got 2");
}

#[test]
fn vm_and_interpreter_error_on_native_function_arity_mismatch() {
    let script = r#"
        return len("ruff", "extra")
    "#;

    assert_interpreter_and_vm_error_contains(script, "len expects 1 arguments, got 2");
}

#[test]
fn vm_and_interpreter_error_on_native_function_arity_too_few() {
    let script = r#"
        return len()
    "#;

    assert_interpreter_and_vm_error_contains(script, "len expects 1 arguments, got 0");
}

#[test]
fn vm_and_interpreter_error_on_range_native_arity() {
    let script = r#"
        return input("prompt", "extra")
    "#;

    assert_interpreter_and_vm_error_contains(script, "input expects 0 to 1 arguments, got 2");
}

#[test]
fn vm_and_interpreter_match_callable_arity_success_paths() {
    let script = r#"
        func add(a, b) {
            return a + b
        }

        closure := func(value) {
            return value * 2
        }

        struct Counter {
            count: int,

            func bump(self, delta) {
                return self.count + delta
            }
        }

        async func async_add(a, b) {
            return a + b
        }

        counter := Counter { count: 10 }
        async_result := await async_add(4, 5)
        arity_success_ok := add(1, 2) == 3
            && closure(5) == 10
            && counter.bump(7) == 17
            && async_result == 9
    "#;

    assert_interpreter_and_vm_bool(script, "arity_success_ok");
}

#[test]
fn vm_and_interpreter_preserve_variadic_native_contracts() {
    let script = r#"
        debug("single")
        debug("a", "b", "c")
        print("variadic-path-ok")
    "#;

    let interp = run_interpreter(script);
    assert!(
        interp.return_value.is_none(),
        "interpreter returned runtime error for variadic native call: {:?}",
        interp.return_value
    );

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(script, vm_env);
    assert!(
        vm_result.is_ok(),
        "vm returned runtime error for variadic native call: {:?}",
        vm_result.err()
    );
}

fn assert_interpreter_and_vm_bool(script: &str, flag_name: &str) {
    let interp = run_interpreter(script);
    assert!(
        interp.return_value.is_none(),
        "interpreter returned runtime error: {:?}",
        interp.return_value
    );
    assert!(matches!(interp.env.get(flag_name), Some(Value::Bool(true))));

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(script, vm_env.clone());
    assert!(vm_result.is_ok(), "vm execution failed: {:?}", vm_result.err());

    let vm_globals = vm_env.lock().expect("failed to lock vm globals");
    assert!(matches!(vm_globals.get(flag_name), Some(Value::Bool(true))));
}

#[test]
fn vm_and_interpreter_keep_string_literals_explicit() {
    let script = r#"
        literal := "missing_top_level_name"
        literal_ok := literal == "missing_top_level_name"
    "#;

    assert_interpreter_and_vm_bool(script, "literal_ok");
}

#[test]
fn vm_and_interpreter_resolve_defined_identifiers() {
    let script = r#"
        top := 10

        func add_one(value) {
            local := value + 1
            return local
        }

        func make_adder(seed) {
            return func(delta) {
                return seed + delta
            }
        }

        add_seed := make_adder(top)
        identifiers_ok := add_one(top) == 11 && add_seed(5) == 15
    "#;

    assert_interpreter_and_vm_bool(script, "identifiers_ok");
}

#[test]
fn vm_and_interpreter_match_successful_local_map_update() {
    let script = r#"
        counts := {"hits": 1}
        counts["hits"] := counts["hits"] + 1

        numeric := {}
        numeric[0] := 10
        numeric[0] := numeric[0] + 5

        map_update_ok := counts["hits"] == 2 && numeric[0] == 15
    "#;

    assert_interpreter_and_vm_bool(script, "map_update_ok");
}

#[test]
fn vm_and_interpreter_match_successful_nested_map_update() {
    let script = r#"
        nested := {"inner": {"value": 2}}
        inner := nested["inner"]
        inner["value"] := inner["value"] + 3
        nested["inner"] := inner

        map_update_ok := nested["inner"]["value"] == 5
    "#;

    assert_interpreter_and_vm_bool(script, "map_update_ok");
}

#[test]
fn vm_and_interpreter_match_successful_captured_map_update() {
    let script = r#"
        func make_bump() {
            captured := {"count": 4}
            bump := func() {
                captured["count"] := captured["count"] + 6
                return captured["count"]
            }
            return bump
        }

        bump := make_bump()
        bumped := bump()

        map_update_ok := bumped == 10
    "#;

    assert_interpreter_and_vm_bool(script, "map_update_ok");
}

#[test]
fn vm_and_interpreter_match_spawn_surface() {
    let spawn_key = unique_spawn_key();
    let script = format!(
        r#"
        shared_set("{}", 0)
        spawn {{
            shared_add_int("{}", 1)
        }}
        spawn_final := shared_get("{}")
        spawn_ok := spawn_final >= 0
        shared_delete("{}")
    "#,
        spawn_key, spawn_key, spawn_key, spawn_key
    );

    let interp = run_interpreter(&script);
    assert!(
        interp.return_value.is_none(),
        "interpreter returned runtime error: {:?}",
        interp.return_value
    );
    assert!(matches!(interp.env.get("spawn_ok"), Some(Value::Bool(true))));

    let vm_env = vm_env_with_builtins();
    let vm_result = run_vm(&script, vm_env.clone());
    assert!(vm_result.is_ok(), "vm execution failed: {:?}", vm_result.err());

    let vm_globals = vm_env.lock().expect("failed to lock vm globals");
    assert!(matches!(vm_globals.get("spawn_ok"), Some(Value::Bool(true))));
}
