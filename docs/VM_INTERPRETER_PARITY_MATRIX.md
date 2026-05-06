# VM/Interpreter Parity Matrix (v1.0.0 P0 Tracking)

Last updated: 2026-05-06

This matrix tracks parity status for the runtime surfaces called out in the v1.0.0 P0 roadmap item.

## Scope

Required roadmap surfaces:

- struct method behavior
- spread/destructuring
- match bindings
- spawn semantics
- map missing-key and update semantics

## Status Matrix

| Surface | Interpreter | VM | Parity Status | Evidence |
| --- | --- | --- | --- | --- |
| Struct method behavior (`obj.method(...)`) | supported (explicit `self` methods) | supported (explicit `self` methods) | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_struct_method_behavior_contract` |
| Spread literals + destructuring bindings | spread + destructuring supported | spread + destructuring supported in parity-covered scenarios | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_spread_destructuring_surface` |
| Tag-style match bindings (`case Tag(var):`) | supported for `Result::`/`Option::` tag patterns with bound values | supported for `Result::`/`Option::` tag patterns with bound values | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_enum_match_binding_surface` |
| Spawn semantics (`spawn { ... }`) | supported in tested shared-state scenario | supported in tested shared-state scenario | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_spawn_surface` |
| Missing map keys and invalid map key types | missing keys error; invalid key types error | missing keys error; invalid key types error | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_error_on_missing_string_map_key`, `vm_and_interpreter_error_on_missing_integer_map_key`, `vm_and_interpreter_error_on_nested_missing_map_key`, `vm_and_interpreter_error_on_invalid_map_key_type` |
| Map update paths | local, nested, and closure-captured updates covered | local, nested, and closure-captured updates covered | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_successful_local_map_update`, `vm_and_interpreter_match_successful_nested_map_update`, `vm_and_interpreter_match_successful_captured_map_update` |

## Notes

- Parity and capability are tracked separately. A surface can be parity-aligned while still not meeting desired language capability.
- VM spread/destructuring behavior is now aligned with interpreter behavior for parity-covered scenarios.
- Missing map keys are now a runtime error in both runtime paths; tests cover string keys, integer keys, nested map reads, invalid key types, and successful local/nested/captured updates.
- When behavior changes, update this matrix and the associated test coverage in the same commit.
