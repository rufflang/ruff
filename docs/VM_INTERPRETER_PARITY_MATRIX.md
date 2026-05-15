# VM/Interpreter/Compiler Parity Matrix (v1.0.0)

Last updated: 2026-05-08

This matrix tracks parity for roadmap item `V1-COMP-001`.

## Status Labels

- `supported`: Implemented and parity-covered by tests.
- `unsupported (explicit)`: Rejected intentionally with deterministic errors.
- `not yet parity-covered`: Implemented surface exists, but dedicated parity evidence is still pending.
- `intentionally divergent`: Behavior differs by design and is documented.

## Matrix

| Surface | Compiler | Interpreter | VM | Status | Evidence |
| --- | --- | --- | --- | --- | --- |
| Variable/identifier resolution (`let`/`mut`/`const`, undefined identifiers) | lowers locals/globals with mutability metadata | lexical scopes + undefined-variable runtime errors | matching load/store + undefined-variable runtime errors | supported | `vm_and_interpreter_resolve_defined_identifiers`, `vm_and_interpreter_error_on_undefined_top_level_identifier`, `vm_and_interpreter_error_on_undefined_identifier_inside_function`, `vm_and_interpreter_error_on_undefined_identifier_inside_closure` |
| Function/closure/method/async/generator arity | emits callable metadata used by runtime arity checks | shared arity validation | matching callable arity checks | supported | `vm_and_interpreter_error_on_function_arity_too_few`, `vm_and_interpreter_error_on_function_arity_too_many`, `vm_and_interpreter_error_on_closure_arity_mismatch`, `vm_and_interpreter_error_on_method_arity_mismatch`, `vm_and_interpreter_error_on_async_function_arity_mismatch`, `vm_and_interpreter_error_on_generator_arity_mismatch`, `vm_and_interpreter_match_callable_arity_success_paths` |
| Struct methods (`obj.method(...)`) | lowers `MethodCall` to field-get + call | explicit `self` method dispatch | bytecode method dispatch | supported | `vm_and_interpreter_match_struct_method_behavior_contract` |
| Struct generator methods (`func*` inside `struct`) | compile-time rejection with shared message helper | runtime rejection with same shared message helper | compile path returns same message | unsupported (explicit) | `vm_and_interpreter_error_on_unsupported_struct_generator_method` |
| Collections/indexing/mutation | lowers array/dict/index ops and in-place updates | runtime checked index/map semantics | matching checked index/map semantics | supported | `vm_and_interpreter_match_valid_index_assignment_success_path`, `vm_and_interpreter_error_on_invalid_index_assignment_target`, `vm_and_interpreter_error_on_out_of_bounds_array_index`, `vm_and_interpreter_error_on_missing_string_map_key`, `vm_and_interpreter_match_successful_local_map_update` |
| Spread literals + destructuring bindings | emits marker-based spread/dict construction | spread + destructuring execution | matching marker-based spread/dict execution | supported | `vm_and_interpreter_match_spread_destructuring_surface` |
| Match/tag bindings (`Result::`/`Option::`) | lowers tag pattern checks | tag-style binding support | matching tag pattern checks | supported | `vm_and_interpreter_match_enum_match_binding_surface` |
| Imports (`import`, `from ... import ...`) | emits VM import native opcodes (`__vm_import_all`, `__vm_import_symbol`) | module-loader-backed import resolution | VM import handlers use module loader and bind into active scope | supported | `vm_and_interpreter_match_import_export_surface` |
| Control flow (`if`/`while`/`loop`/`break`/`continue`/top-level `return`) | control-flow opcodes with validation | matching runtime semantics | matching runtime semantics | supported | `vm_and_interpreter_allow_break_and_continue_inside_loop`, `vm_and_interpreter_error_on_break_outside_loop`, `vm_and_interpreter_allow_top_level_return_for_script_exit` |
| Truthiness + short-circuit boolean logic | short-circuit lowering | shared truthiness/short-circuit semantics | matching truthiness/jump semantics | supported | `vm_and_interpreter_match_truthiness_semantics_across_conditionals`, `vm_and_interpreter_short_circuit_logical_operators_skip_rhs_when_possible`, `vm_and_interpreter_short_circuit_logical_operators_evaluate_rhs_when_required` |
| Equality/comparison + numeric safety | equality/comparison opcodes and checked arithmetic | centralized equality/comparison helpers + overflow/zero checks | same helper-backed comparison + checked arithmetic | supported | `vm_and_interpreter_define_cross_type_numeric_and_string_ordering_contract`, `vm_and_interpreter_define_collection_and_callable_equality_contract`, `vm_and_interpreter_reject_integer_add_overflow`, `vm_and_interpreter_reject_float_division_by_zero` |
| Native function parity (VM-allowed natives) | native call opcodes | interpreter native dispatch | VM native dispatch + shared native impl | supported | `vm_and_interpreter_error_on_native_function_arity_mismatch`, `vm_and_interpreter_preserve_variadic_native_contracts` |
| Spawn surface (`spawn { ... }`) | lowered closure-based spawn path | background-thread spawn support | matching tested spawn scenario | supported | `vm_and_interpreter_match_spawn_surface` |

## CI Gate

- CI now includes a dedicated parity job that runs:
  - `cargo test --test vm_interpreter_parity_surfaces`
- The full release gate still requires `cargo test`, but the dedicated parity job fails fast on runtime-path drift.

## Notes

- This matrix tracks interpreter/VM/compiler parity only. JIT remains an experimental, opt-in surface (`ruff run --jit`) with explicit unsupported-opcode detection and deterministic fallback messaging; see `V1-JIT-001` notes in `ROADMAP.md`.
- Any newly added language/runtime surface must update this matrix and add parity evidence in the same change.
