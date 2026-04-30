# VM/Interpreter Parity Matrix (v1.0.0 P0 Tracking)

Last updated: 2026-05-01

This matrix tracks parity status for the runtime surfaces called out in the v1.0.0 P0 roadmap item.

## Scope

Required roadmap surfaces:

- struct method behavior
- spread/destructuring
- match bindings
- spawn semantics

## Status Matrix

| Surface | Interpreter | VM | Parity Status | Evidence |
| --- | --- | --- | --- | --- |
| Struct method behavior (`obj.method(...)`) | supported (explicit `self` methods) | supported (explicit `self` methods) | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_struct_method_behavior_contract` |
| Spread literals + destructuring bindings | spread + destructuring supported | spread + destructuring supported in parity-covered scenarios | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_spread_destructuring_surface` |
| Tag-style match bindings (`case Tag(var):`) | current script shape does not produce bound match variables in this test | current script shape does not produce bound match variables in this test | behavior aligned but capability incomplete | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_enum_match_binding_surface` |
| Spawn semantics (`spawn { ... }`) | supported in tested shared-state scenario | supported in tested shared-state scenario | aligned | `tests/vm_interpreter_parity_surfaces.rs::vm_and_interpreter_match_spawn_surface` |

## Notes

- Parity and capability are tracked separately. A surface can be parity-aligned while still not meeting desired language capability.
- VM spread/destructuring behavior is now aligned with interpreter behavior for parity-covered scenarios.
- Until the tag-style match-binding capability gap is closed, this surface should remain in the roadmap as open v1.0.0 work.
- When behavior changes, update this matrix and the associated test coverage in the same commit.
