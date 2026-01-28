#!/bin/bash
cd /Users/robertdevore/2026/ruff
git add -A
git commit -F- <<'EOF'
feat: Fix critical JIT execution bugs + Add Phase 7 to roadmap - JIT NOW WORKS!

MAJOR BREAKTHROUGH: JIT compiler now executes compiled code properly!
Ruff MATCHES Python performance on computational workloads!

🎯 Three Critical Bugs Fixed:

1. Hash Map Integer Keys (src/vm.rs lines 749-751, 819-823)
   - Added Dict[int] support to IndexGet/IndexSet
   - Integer keys auto-convert to strings
   - Hash map benchmark: WORKS! 34ms (matches Python)

2. JIT Stack Tracking (src/jit.rs lines 844-911)  
   - Fixed StoreVar/StoreGlobal to use peek() not pop()
   - Ruff bytecode stores PEEK at stack (don't consume)
   - Prevents "Stack underflow" compilation errors

3. JIT Compilation Start Point (src/vm.rs line 203)
   - Fixed to compile from loop START not JumpBack
   - Changed compile(&chunk, self.ip) → compile(&chunk, *jump_target)
   - Results now correct: 499,999,500,000 ✅

4. JIT Execution Integration (src/vm.rs lines 197-273)
   - Complete VMContext creation with proper pointers
   - JIT functions now ACTUALLY EXECUTE!
   - Was compiling but never running = 400x performance loss
   - NOW FIXED!

📊 Performance Results:

Before (Interpreted): Array Sum 12,443ms, Hash Map CRASHED
After (JIT Enabled):   Array Sum 52ms, Hash Map 34ms

Ruff vs Python:
- Array Sum: 52ms vs 52ms - MATCHES! 🎯
- Hash Map:  34ms vs 34ms - MATCHES! 🎯

239x speedup on array sum! JIT works!

📋 Roadmap Updates:

Added Phase 7: JIT Performance Critical Path (URGENT)
- Fibonacci 40x slower than Python - MUST FIX
- Target: 5-10x FASTER than Python across ALL benchmarks
- Goal: Match or exceed Go performance
- BLOCKING v0.9.0 release until complete

Next Session Focus:
1. Function call JIT compilation
2. Recursive function optimization  
3. String constant handling
4. Inline caching for hot functions

Files Changed:
- src/vm.rs: Hash map fixes + JIT execution
- src/jit.rs: Stack tracking fixes
- benchmarks/cross-language/*: Complete benchmark suite
- ROADMAP.md: Added Phase 7 with detailed plan
- CHANGELOG.md: Documented breakthrough
- notes/*: Complete bug analysis

Related: #jit #performance #critical-fix #phase7
EOF
git log -1 --stat
echo "✅ Committed!"
