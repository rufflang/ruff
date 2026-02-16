# Optional Static Typing Design (v0.10 Exploratory)

This document captures the v0.10 exploratory design package for optional static typing in Ruff.

## Goals

- Define a concrete annotation syntax surface for functions, variables, and collections.
- Define optional runtime type-check mode behavior and error contract.
- Define typed-JIT optimization boundaries and explicit deferrals.
- Preserve backward compatibility with existing untyped Ruff code.

## Design Status

- **Scope**: Design-only in v0.10 (no mandatory typing, no breaking changes)
- **Implementation commitment**: Deferred pending feedback and benchmark validation

## Stage 1: Annotation Surface Proposal

### Supported Type Syntax (proposed)

```ruff
func calculate(x: int, y: float) -> float {
    return x * y
}

let name: string := "Alice"
let enabled: bool := true
let scores: Array<int> := [95, 87, 92]
let user_scores: Dict<string, int> := {"alice": 95}
```

### Type Forms (proposed)

- Primitive: `int`, `float`, `string`, `bool`, `null`
- Collections: `Array<T>`, `Dict<K, V>`, `Set<T>`
- Function return type: `func f(...) -> T`

### Annotation Semantics (proposed)

- Annotations are optional and additive.
- Untyped code remains valid and unchanged.
- Mixed typed/untyped code is supported.
- Missing annotations imply current dynamic behavior.

### Parser and Type-Checker Impact

- Parser: add annotation nodes on variable declarations and function signatures.
- AST: preserve annotations as metadata without changing runtime semantics by default.
- Type checker: run as optional pass that can emit diagnostics without blocking execution unless explicitly configured.

## Stage 2: Optional Runtime Type-Check Mode Contract

### Proposed opt-in forms

```ruff
@type_check
func calculate(x: int, y: float) -> float {
    return x * y
}
```

Future global opt-in could be provided via CLI and/or config, for example:

- `ruff run script.ruff --type-check`
- project-level `ruff.toml` setting (deferred decision)

### Proposed Runtime Behavior

- Default mode: no runtime type enforcement (current behavior).
- Opt-in mode: enforce argument and return type contracts on annotated functions.
- Errors are deterministic and include function name, parameter/return position, expected type, and actual runtime type.

### Proposed Error Shape

Human-facing format (subject to final wording):

- `TypeError: calculate arg#2 expected float, got string`
- `TypeError: calculate return expected float, got int`

Compatibility note:

- Existing error model remains authoritative; this format is a contract target for any future implementation.

## Stage 3: Typed-JIT Optimization Boundaries

### In Scope (future candidates)

- Specialized numeric fast paths for consistently typed arithmetic.
- Reduced boxing/unboxing in tight loops over typed arrays.
- Improved call-site specialization for stable typed signatures.

### Out of Scope for v0.10/v1.0

- Mandatory typing across all code.
- Full Hindley-Milner inference.
- Breaking syntax or runtime behavior changes for untyped code.
- Guarantees of fixed speedup multiples.

## Migration and Compatibility

- Existing untyped scripts continue to run unchanged.
- Teams can adopt annotations incrementally per module/function.
- Typed and untyped modules interoperate through existing runtime value model.
- Any future runtime checks must remain opt-in and backward compatible by default.

## Open Decisions

- Final syntax for union/optional types (e.g., `A | B` vs helper forms).
- How generic parameter constraints are represented (if at all).
- CLI-only vs config-based type-check mode enabling.
- Whether warnings-only mode should be separate from strict fail mode.

## Non-Goals (for this design package)

- Shipping a full compiler-grade static type system.
- Requiring developers to annotate all public functions.
- Changing default dynamic semantics in v1.0.

## Decision Summary

Ruff should pursue **optional, incremental typing** with strict backward compatibility:

- Keep dynamic execution as default.
- Add annotations as metadata first.
- Introduce runtime checks only via explicit opt-in.
- Use typed information for targeted JIT specialization where measurable and safe.

This approach lowers adoption risk while preserving Ruff’s ergonomics and enabling future performance/tooling gains.
