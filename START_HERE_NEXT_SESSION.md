# 🚨 URGENT: NEXT SESSION MUST IMPLEMENT FUNCTION-LEVEL JIT

## Management Decision

**FUNCTION-LEVEL JIT MUST BE IMPLEMENTED NOW FOR v0.9.0**

This is **NOT deferred to v1.0**. This is **P0 priority**. This is **BLOCKING the v0.9.0 release**.

## Why This Is Critical

Current performance is **UNACCEPTABLE**:
- Fibonacci recursive: **42x SLOWER than Python** (11,782ms vs 282ms)
- Fibonacci iterative: **7.8x SLOWER than Python** (918ms vs 118ms)

This is not an edge case. Function calls are fundamental to ALL real code. Being 42x slower than Python on common patterns means Ruff cannot be used in production.

## What Must Be Done

The next coding session **MUST** implement function-level JIT compilation.

### Timeline
- **Start**: Immediately (next session)
- **Duration**: 2-4 weeks of focused work
- **Priority**: P0 - Highest priority in the entire project
- **Blocking**: v0.9.0 cannot ship without this

### Implementation Plan

Read `ROADMAP.md` Phase 7 section for complete details. Summary:

**Week 1-2: Core Architecture**
1. Function call tracking in VM (2-3 days)
   - Track every `OpCode::Call` execution
   - Trigger JIT after threshold (e.g., 100 calls)
   
2. Function body compilation (3-4 days)
   - Extend `JitCompiler::compile()` for function bodies
   - Compile from function start to Return
   
3. Call opcode JIT support (2-3 days)
   - Implement Call opcode in `translate_instruction`
   - Jump to native code for JIT'd functions
   - Fallback to interpreter for non-JIT'd

**Week 3-4: Optimization**
4. Recursive function optimization (3-4 days)
5. Return value optimization (2-3 days)
6. Testing and validation (3-4 days)

### Success Criteria

v0.9.0 **CANNOT** ship until these targets are met:

```
Fibonacci recursive (n=30):  <50ms  (currently 11,782ms)
Fibonacci iterative (100k):  <20ms  (currently 918ms)
ALL benchmarks:              >= Python performance
Target:                      5-10x FASTER than Python
```

## Files to Read

**Start here (in order)**:
1. **`ROADMAP.md` Phase 7** - Complete implementation plan (READ THIS FIRST!)
2. `NEXT_STEPS.md` - What to do next
3. `PHASE7_SESSION_SUMMARY.md` - What was done this session
4. `CHANGELOG.md` - Phase 7 section

## Files to Modify

- `src/vm.rs` - Add function call tracking to `OpCode::Call`
- `src/jit.rs` - Extend `compile()` for functions, add Call opcode
- `src/bytecode.rs` - May need function metadata

## Key Technical Insights

From this session's analysis:

1. **Current JIT only triggers on `JumpBack`** (backward jumps in loops)
   - Works: while loops, for loops, computational kernels
   - Doesn't work: Functions, recursive calls, call-heavy code

2. **Fibonacci has NO LOOPS**
   - Recursive version: Just function calls
   - Iterative version: Loop is JIT'd, but function calls to it are not
   - Each function call goes through slow interpreter

3. **Solution: JIT on function entry**
   - Track function call counts
   - After N calls, JIT-compile the entire function body
   - Make `Call` opcode check: if function is JIT'd, jump to native code
   - Mixed execution: some functions JIT'd, some interpreted

## What Was Done This Session

✅ String constant handling fix (small improvement)
✅ Root cause analysis (identified function-level JIT need)
✅ Updated all documentation with management decision
✅ Created detailed implementation plan in ROADMAP

## What Must Be Done Next Session

🚨 Implement function-level JIT (P0 priority)
🚨 Achieve fibonacci performance targets
🚨 Make Ruff 5-10x faster than Python on ALL benchmarks

## To Commit Current Changes

```bash
cd /Users/robertdevore/2026/ruff
chmod +x do_commit.sh
./do_commit.sh
```

## This Is Non-Negotiable

- v0.9.0 **CANNOT** ship without function-level JIT
- This **CANNOT** be deferred to v1.0
- Next session **MUST** start with this work
- This determines if Ruff is a serious language or a toy project

## Questions?

Read the documentation files listed above. Everything is documented in detail.

---

**REMEMBER**: This is P0 priority. Start immediately. Do not defer. v0.9.0 release is blocked until this is complete.
