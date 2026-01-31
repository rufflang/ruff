# Next Steps for Dict Optimization Phase 2

## Priority 1: Optimize Dict Read Performance ✅ COMPLETED

### Final Results
- **Before**: 159ms for 1000 operations (636x slower than Python)
- **After**: ~8.8ms average for 1000 operations (35x slower than Python)
- **Speedup**: 18x faster than before
- **Implementation**: Changed Value enum to use Arc<String>, Arc<Vec<Value>>, Arc<HashMap<String, Value>>
- **.cloned()** now just increments ref count instead of deep copying

### What Was Done
1. Modified `src/interpreter/value.rs` to use Arc for heap-allocated types:
   - `Str(String)` → `Str(Arc<String>)`
   - `Array(Vec<Value>)` → `Array(Arc<Vec<Value>>)`
   - `Dict(HashMap)` → `Dict(Arc<HashMap<String, Value>>)`
2. Updated all 315+ compilation errors across 11+ files
3. Fixed IndexGet/IndexSet/IndexSetInPlace to use Arc::make_mut() for mutations
4. Added helper methods for Arc wrapping in Value impl
5. All tests passing

### Performance Notes
- Dict reads are now 18x faster
- Still 35x slower than Python (vs 636x before)
- Further optimization possible:
  - JIT compilation for dict opcodes
  - HashMap replacement with faster alternatives
  - Specialized dict implementations for common patterns

---

## Priority 2: Add JIT Support for Dict Opcodes (HIGH)

### Current State
- IndexGetInPlace and IndexSetInPlace fall back to interpreter
- JIT only compiles older opcodes
- Missing opportunity for native code performance
- With Arc optimization, dict operations are now fast enough to benefit from JIT

### What's Needed
File: `src/jit.rs` (or wherever Cranelift codegen lives)

Add codegen for:
1. **IndexGetInPlace(var_name)**
   - Load pointer to local variable from frame
   - Call runtime helper to access dict/array
   - Push result to stack

2. **IndexSetInPlace(var_name)**
   - Pop index and value from stack
   - Load pointer to local variable
   - Call runtime helper to set value
   - Push null to stack

### Implementation Steps
1. Find existing IndexGet/IndexSet JIT code (if any)
2. Add similar code for IndexGetInPlace/IndexSetInPlace
3. Create runtime helpers if needed (for dict/array access)
4. Test with DEBUG_JIT=1 to verify compilation
5. Benchmark to measure improvement

### Expected Impact
- Should bring Ruff closer to Go's performance (currently 2-4x slower)
- Particularly impactful for loops with dict access

---

## Priority 3: Test 100k Operations (LOW - after Priority 1)

### Current State
- 100k dict operations timeout after 60 seconds
- Needs read optimization to be viable

### Action Items
1. After Priority 1 is complete, test with bench_dict.ruff n=100000
2. Should complete in <5 seconds with optimized reads
3. Update BENCHMARK_RESULTS.md with 100k results
4. Run full benchmark suite (run_benchmarks.sh) without timeouts

---

## Priority 4: Update Documentation

### Files to Update
1. **BENCHMARK_RESULTS.md**
   - Add Phase 2 optimization results
   - Update dict performance numbers
   - Add 100k operation results

2. **CHANGELOG.md**
   - Document IndexGetInPlace/IndexSetInPlace opcodes
   - Note performance improvements
   - Mention Arc<Value> changes (if implemented)

3. **ROADMAP.md**
   - Mark dict optimization as complete
   - Update performance goals
   - Plan next optimization targets

---

## Quick Win: Low-Hanging Fruit

If Arc<Value> is too complex, try this first:
- Profile dict reads to find exact bottleneck
- Check if string key hashing is the issue
- Consider caching hash values
- Look for unnecessary clones in the hot path

Run this to profile:
```bash
cargo build --release
./target/release/ruff run benchmarks/cross-language/profile_dict.ruff
```

Then use `perf` or similar tools to see where time is spent:
```bash
perf record ./target/release/ruff run benchmarks/cross-language/bench_dict.ruff
perf report
```

---

## Success Criteria

### Phase 2 Complete When:
- ✅ Dict reads are <10ms for 1000 operations (currently 159ms)
- ✅ 100k dict operations complete in <5 seconds
- ✅ Total dict performance within 10x of Python (currently 656x slower)
- ✅ JIT support for new opcodes working
- ✅ Documentation updated

### Stretch Goals:
- Dict performance within 5x of Python
- Go-level performance with JIT (currently 2-4x slower)
- No timeouts on any benchmark
- All benchmarks pass in CI

---

## Code Locations Reference

- **Value enum:** `src/interpreter/value.rs`
- **Opcodes:** `src/bytecode.rs`
- **VM execution:** `src/vm.rs`
- **Compiler:** `src/compiler.rs`
- **JIT codegen:** `src/jit.rs`
- **Benchmarks:** `benchmarks/cross-language/`

---

## Notes for Next Session

Current commit: `30b9749 - Add comprehensive benchmark results`

Key achievements so far:
- ✅ 36x faster dict writes
- ✅ 30-50x faster than Python on compute
- ✅ Comprehensive benchmarks and documentation

Main blocker:
- ⚠️ Dict reads still slow (cloning values)

This is the critical path to production-ready dict performance!
