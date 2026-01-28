## 🚀 KICKSTART MESSAGE FOR NEXT AGENT

Copy and paste this to the next agent:

---

**URGENT MISSION: Make Ruff 5-10x FASTER than Python**

Current state:
- ✅ JIT works! Array sum matches Python (52ms)
- ❌ Fibonacci is 40x SLOWER than Python (11,782ms vs 282ms)
- ❌ This is BLOCKING v0.9.0 release

**Your mission:** Fix JIT to handle function calls and recursion

**Start here (in order):**
1. Read `notes/NEXT_SESSION.md` - complete mission brief
2. Read `ROADMAP.md` Phase 7 - technical implementation plan  
3. Run benchmarks: `cd benchmarks/cross-language && ./run_benchmarks.sh`
4. Check results: Latest file in `benchmarks/cross-language/results/`

**First task (Day 1-2):** Fix string constant handling in JIT
- File: `src/jit.rs` lines 719-742
- Problem: JIT fails when it sees `LoadConst` with strings
- Solution: Skip/stub string operations, don't fail entire compilation
- Impact: Allows functions with print() to be JIT-compiled

**Why this matters:**
Currently JIT only works on pure arithmetic loops. Functions with ANY string constants fall back to slow interpretation. Fibonacci has NO strings but can't compile because JIT fails on surrounding code.

**Performance targets (non-negotiable):**
- Fib Recursive: Must drop from 11,782ms to <50ms (5-10x faster than Python)
- Fib Iterative: Must drop from 918ms to <20ms (5-10x faster than Python)

**Key insight:** All the hard work is done! JIT compiles and executes. We just need to expand what opcodes it can handle. Start small (strings), then add function calls, then optimize recursion.

**Test after each change:**
```bash
cd benchmarks/cross-language && ./run_benchmarks.sh
```

Check latest results file to verify improvements.

**Success = Ruff is 5-10x FASTER than Python across ALL benchmarks.**

Go! 🚀
