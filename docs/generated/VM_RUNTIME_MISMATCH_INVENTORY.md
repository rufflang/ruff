# VM Runtime Mismatch Inventory

Generated: 2026-05-21
Runner: `/Users/robertdevore/2026/ruff/target/debug/ruff`
Fixture root: `tests`

| Fixture | VM Exit | Interpreter Exit | VM Matches Snapshot | Interpreter Matches Snapshot | Delta Type | Mismatch Bucket | Owner | Priority | Rationale |
| --- | ---: | ---: | --- | --- | --- | --- | --- | --- | --- |
| `tests/arg_parser.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/array_methods_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/bytecode_vm.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/destructuring.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/dict_methods_test.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/enhanced_errors.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/env_and_args.ruff` | 4 | 4 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/error_call_stack_test.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/error_no_stack_test.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/lexer_invalid_escape.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/parser_missing_paren.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/runtime_break_outside_loop.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/runtime_invalid_unary.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/runtime_undefined_identifier.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/diagnostics/semantic_invalid_assignment.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/docgen/conformance_edges.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/docgen/ruff_async_strict_public.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/docgen/ruff_async_visibility.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/docgen/ruff_parser_assisted_fallback.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/docgen/ruff_parser_assisted_success.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/fuzz/artifacts/parser/crash-synthetic.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/fixtures/fuzz/synthetic_crash_input.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/generators_test.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/image_processing_test.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/integer_types.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/iterators_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/jit_direct_recursion.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/jit_inline_cache.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/jit_loop_tests.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/jit_register_locals.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/match_empty_body.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/match_no_param.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/minimal_match_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/net_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/range_format_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/result_option.ruff` | 0 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/simple_error_test.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/simple_image_test.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/simple_match_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/simple_ok.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/simple_result_test.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/spread_operator.ruff` | 0 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/stdlib_crypto_test.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/stdlib_io_test.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/stdlib_os_path_test.ruff` | 4 | 4 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/stdlib_test.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/string_methods_test.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_arithmetic.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_array_contains.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_array_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_assert_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_assertions.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_basic_print.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_binary_files.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_binary_simple.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_call_method.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_chain.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_chain_debug.ruff` | 4 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_collections.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_comment_edge_cases.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_connection_pooling.ruff` | 4 | 4 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_database_transactions.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_debug_add.ruff` | 4 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_display.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_doc_comments.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_dunder.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_enhanced_collections.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_enum_err.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_enum_err_only.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_enum_nested.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_enum_none.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_enum_ok.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_exceptions_comprehensive.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_for_range.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_for_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_func_loop_correct.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_function_drop_fix.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_functions.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_generators.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_higher_order.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_http.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_http_headers.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_http_type_checking.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_json_edge_cases.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_json_parse.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_json_serialize.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_loop_correct.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_method_array.ruff` | 4 | 0 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_method_chaining.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_method_features.ruff` | 4 | 4 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_method_field_ref.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_method_name.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_method_param_minimal.ruff` | 4 | 0 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_method_print.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_method_with_print.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_minimal.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_minimal_hang.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_mixed_comments.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_module_syntax.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_multiline_comments.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_negative.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_no_semi.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_op_add.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_op_add_debug.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_operator_add_working.ruff` | 4 | 0 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_operator_overloading.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_operator_simple.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_range_args.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_range_debug.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_range_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_reassign.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_regex.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_regex_simple.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_self_backward_compat.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_self_field.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_self_minimal.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_self_param.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_self_return.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_self_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_simple_random.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_standalone_dunder.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_stdlib_datetime.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_stdlib_paths.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_stdlib_random.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_stdlib_system.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_def_only.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_empty_method.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_instantiate.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_method_debug.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_struct_method_print.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_nomethod.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_only.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_parse.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_return.ruff` | 4 | 0 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_struct_simple.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_struct_simple_debug.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_tiny.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_trans_debug.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_trans_minimal.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_trans_newvar.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_trans_nostr.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_trans_vars.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_transaction_simple.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_transactions_working.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_try_except.ruff` | 0 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/test_unary_current.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_unary_lit.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_unary_mixed.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_unary_ops.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_unary_overload.ruff` | 4 | 0 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_undefined_var.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_vec_add.ruff` | 4 | 0 | no | yes | `vm_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/test_verifier.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_vm_optimizations.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/test_void_method.ruff` | 4 | 0 | no | no | `both_mismatch_different_output` | `harness-debt` | harness-owner | `P2` | both runtimes diverge from snapshot with different output, indicating fixture-harness normalization debt |
| `tests/testing_framework.ruff` | 3 | 3 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/vm_closure_adder.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/vm_closure_debug.ruff` | 0 | 0 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |
| `tests/vm_closure_detailed.ruff` | 0 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/vm_closure_multiple.ruff` | 0 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/vm_closure_order.ruff` | 0 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/vm_closure_simple.ruff` | 0 | 4 | yes | no | `interpreter_only_mismatch` | `runtime-parity-bug` | runtime-owner | `P0` | runtime-path mismatch against snapshot indicates parity defect or runtime-specific contract drift |
| `tests/vm_native_functions_test.ruff` | 4 | 4 | yes | yes | `both_match_snapshot` | `none` | n/a | `P4` | snapshot matches in both runtimes |

Summary: `163` fixtures scanned
- both match snapshot: `122`
- VM-only mismatch: `14`
- interpreter-only mismatch: `11`
- both mismatch: `16`

Mismatch classification totals (priority order):
- P0 runtime-parity-bug (`runtime-owner`): `25`
- P1 stale-snapshot-expectation (`docs-owner`): `0`
- P1 parser-invalid-fixture (`language-owner`): `0`
- P2 harness-debt (`harness-owner`): `16`
- P2 intentional-divergence (`runtime-owner`): `0`
