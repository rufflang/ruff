# Unsafe Inventory

Generated: 2026-05-24
Command: rg -n --glob '*.rs' --glob '!tests/unsafe_inventory_contract.rs' '\bunsafe\b' src tests benches fuzz

## Summary

- Total matches: 67
- Executable matches: 59
- Non-executable matches: 8
- Unknown classifications: 0

## Rows

| Path | Line | Kind | Classification | Text |
| --- | ---: | --- | --- | --- |
| src/jit.rs | 142 | executable | jit_executable |             let vm = unsafe { &mut *(ctx.vm_ptr as *mut crate::vm::VM) }; |
| src/jit.rs | 178 | executable | jit_executable | pub type CompiledFn = unsafe extern ""C"" fn(*mut VMContext) -> i64; |
| src/jit.rs | 183 | executable | jit_executable | pub type CompiledFnWithArg = unsafe extern ""C"" fn(*mut VMContext, i64) -> i64; |
| src/jit.rs | 185 | non_executable | jit_comment_or_doc | /// Invoke a compiled JIT function through one audited unsafe boundary. |
| src/jit.rs | 198 | executable | jit_executable |     unsafe { compiled_fn(ctx as *mut VMContext) } |
| src/jit.rs | 201 | non_executable | jit_comment_or_doc | /// Invoke a single-argument compiled JIT function through one audited unsafe boundary. |
| src/jit.rs | 217 | executable | jit_executable |     unsafe { compiled_fn(ctx as *mut VMContext, arg) } |
| src/jit.rs | 374 | executable | jit_executable | pub unsafe extern ""C"" fn jit_stack_push(ctx: *mut VMContext, value: i64) { |
| src/jit.rs | 393 | executable | jit_executable | pub unsafe extern ""C"" fn jit_stack_pop(ctx: *mut VMContext) -> i64 { |
| src/jit.rs | 416 | executable | jit_executable | pub unsafe extern ""C"" fn jit_obj_push_string(ctx: *mut VMContext, ptr: i64, len: i64) -> i64 { |
| src/jit.rs | 444 | executable | jit_executable | pub unsafe extern ""C"" fn jit_obj_to_vm_stack(ctx: *mut VMContext, handle: i64) -> i64 { |
| src/jit.rs | 477 | executable | jit_executable | pub unsafe extern ""C"" fn jit_load_variable( |
| src/jit.rs | 580 | executable | jit_executable | pub unsafe extern ""C"" fn jit_store_variable( |
| src/jit.rs | 622 | executable | jit_executable | pub unsafe extern ""C"" fn jit_store_variable_from_stack(ctx: *mut VMContext, name_hash: i64) -> i64 { |
| src/jit.rs | 674 | executable | jit_executable | pub unsafe extern ""C"" fn jit_append_const_string_in_place( |
| src/jit.rs | 731 | executable | jit_executable | pub unsafe extern ""C"" fn jit_append_const_char_in_place( |
| src/jit.rs | 791 | executable | jit_executable | pub unsafe extern ""C"" fn jit_local_slot_dict_get( |
| src/jit.rs | 954 | executable | jit_executable | pub unsafe extern ""C"" fn jit_local_slot_dict_set( |
| src/jit.rs | 1229 | executable | jit_executable | pub unsafe extern ""C"" fn jit_local_slot_int_dict_get( |
| src/jit.rs | 1312 | executable | jit_executable | pub unsafe extern ""C"" fn jit_local_slot_int_dict_set( |
| src/jit.rs | 1460 | executable | jit_executable | pub unsafe extern ""C"" fn jit_int_dict_unique_ptr(ctx: *mut VMContext, slot_index: i64) -> i64 { |
| src/jit.rs | 1541 | executable | jit_executable | pub unsafe extern ""C"" fn jit_int_dict_get_ptr(dict_ptr: i64, key: i64) -> i64 { |
| src/jit.rs | 1600 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dense_int_dict_int_get_ptr(dict_ptr: i64, key: i64) -> i64 { |
| src/jit.rs | 1630 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dense_int_dict_int_full_get_ptr(dict_ptr: i64, key: i64) -> i64 { |
| src/jit.rs | 1657 | executable | jit_executable | pub unsafe extern ""C"" fn jit_int_dict_set_ptr(dict_ptr: i64, key: i64, value: i64) -> i64 { |
| src/jit.rs | 1728 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dense_int_dict_int_set_ptr( |
| src/jit.rs | 1764 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dense_int_dict_int_full_set_ptr( |
| src/jit.rs | 1799 | executable | jit_executable | pub unsafe extern ""C"" fn jit_load_variable_float(ctx: *mut VMContext, name_hash: i64) -> f64 { |
| src/jit.rs | 1845 | executable | jit_executable | pub unsafe extern ""C"" fn jit_store_variable_float(ctx: *mut VMContext, name_hash: i64, value: f64) { |
| src/jit.rs | 1880 | executable | jit_executable | pub unsafe extern ""C"" fn jit_check_type_int(ctx: *mut VMContext, name_hash: i64) -> i64 { |
| src/jit.rs | 1926 | executable | jit_executable | pub unsafe extern ""C"" fn jit_check_type_float(ctx: *mut VMContext, name_hash: i64) -> i64 { |
| src/jit.rs | 1972 | executable | jit_executable | pub unsafe extern ""C"" fn jit_push_int(ctx: *mut VMContext, value: i64) -> i64 { |
| src/jit.rs | 2003 | executable | jit_executable | pub unsafe extern ""C"" fn jit_set_return_int(ctx: *mut VMContext, value: i64) -> i64 { |
| src/jit.rs | 2022 | executable | jit_executable |     unsafe { jit_set_return_int(ctx as *mut VMContext, value) } |
| src/jit.rs | 2035 | executable | jit_executable | pub unsafe extern ""C"" fn jit_get_return_int(ctx: *mut VMContext) -> i64 { |
| src/jit.rs | 2059 | executable | jit_executable | pub unsafe extern ""C"" fn jit_get_arg(ctx: *mut VMContext, index: i64) -> i64 { |
| src/jit.rs | 2093 | executable | jit_executable | pub unsafe extern ""C"" fn jit_call_function( |
| src/jit.rs | 2187 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dict_get(ctx: *mut VMContext) -> i64 { |
| src/jit.rs | 2318 | executable | jit_executable | pub unsafe extern ""C"" fn jit_dict_set(ctx: *mut VMContext) -> i64 { |
| src/jit.rs | 2735 | executable | jit_executable |     let vm_ctx = unsafe { &mut *ctx }; |
| src/jit.rs | 2739 | executable | jit_executable |     let stack = unsafe { &mut *vm_ctx.stack_ptr }; |
| src/jit.rs | 2824 | executable | jit_executable |     let vm_ctx = unsafe { &mut *ctx }; |
| src/jit.rs | 2828 | executable | jit_executable |     let stack = unsafe { &mut *vm_ctx.stack_ptr }; |
| src/jit.rs | 2841 | executable | jit_executable |     let keys = unsafe { &*(keys_ptr as *const Arc<Vec<Arc<str>>>) }; |
| src/jit.rs | 6555 | executable | jit_executable |         let compiled_fn: CompiledFn = unsafe { std::mem::transmute(code_ptr) }; |
| src/jit.rs | 7195 | executable | jit_executable |         let compiled_fn: CompiledFn = unsafe { std::mem::transmute(code_ptr) }; |
| src/jit.rs | 7407 | executable | jit_executable |         let compiled_fn: CompiledFnWithArg = unsafe { std::mem::transmute(code_ptr) }; |
| src/jit.rs | 7992 | executable | jit_executable |         let compiled_fn: CompiledFn = unsafe { std::mem::transmute(code_ptr) }; |
| src/jit.rs | 8153 | executable | jit_executable |     unsafe extern ""C"" fn dummy_compiled_fn(_ctx: *mut VMContext) -> i64 { |
| src/jit.rs | 8163 | executable | jit_executable |     unsafe extern ""C"" fn dummy_compiled_fn_with_arg(_ctx: *mut VMContext, arg: i64) -> i64 { |
| src/jit.rs | 9205 | executable | jit_executable |         unsafe { |
| src/module.rs | 645 | non_executable | src_comment_or_string |             ""expected unsafe traversal error, got: {}"", |
| tests/fixtures/unsafe_safety_contracts/malformed_contract.rs | 1 | executable | test_executable | pub unsafe extern ""C"" fn jit_ffi(ptr: *mut i64) -> i64 { |
| tests/fixtures/unsafe_safety_contracts/malformed_contract.rs | 4 | executable | test_executable |     unsafe { *ptr } |
| tests/fixtures/unsafe_safety_contracts/missing_contract.rs | 1 | executable | test_executable | pub unsafe extern ""C"" fn jit_ffi(ptr: *mut i64) -> i64 { |
| tests/fixtures/unsafe_safety_contracts/missing_contract.rs | 2 | executable | test_executable |     unsafe { *ptr } |
| tests/fixtures/unsafe_safety_contracts/type_alias_only.rs | 1 | executable | test_executable | pub type CompiledFn = unsafe extern ""C"" fn(*mut i64) -> i64; |
| tests/fixtures/unsafe_safety_contracts/valid_jit_like.rs | 4 | executable | test_executable | pub unsafe extern ""C"" fn jit_ffi(ptr: *mut i64) -> i64 { |
| tests/fixtures/unsafe_safety_contracts/valid_jit_like.rs | 8 | executable | test_executable |     unsafe { *ptr } |
| tests/fixtures/unsafe_safety_contracts/valid_jit_like.rs | 15 | executable | test_executable |     unsafe { *raw } |
| tests/fixtures/unsafe_safety_contracts/wrong_headings.rs | 1 | executable | test_executable | pub unsafe extern ""C"" fn jit_ffi(ptr: *mut i64) -> i64 { |
| tests/fixtures/unsafe_safety_contracts/wrong_headings.rs | 5 | executable | test_executable |     unsafe { *ptr } |
| tests/jit_safety_contract_checker.rs | 172 | non_executable | test_comment_or_string |     assert!(stdout.contains(""Checked 0 executable unsafe boundaries"")); |
| tests/runtime_security.rs | 237 | non_executable | test_comment_or_string |         ""expected unsafe traversal error, got: {}"", |
| tests/unsafe_safety_gate_contract.rs | 14 | non_executable | test_comment_or_string |         .expect(""failed to run unsafe safety gate help""); |
| tests/unsafe_safety_gate_contract.rs | 29 | non_executable | test_comment_or_string |         .expect(""failed to run unsafe safety gate dry-run""); |
| tests/unsafe_safety_gate_contract.rs | 56 | non_executable | test_comment_or_string |         .expect(""failed to run unsafe safety gate unknown-arg check""); |
