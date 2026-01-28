#!/bin/bash
set -e
cd /Users/robertdevore/2026/ruff

echo "=== Staging changes ==="
git add src/jit.rs CHANGELOG.md ROADMAP.md PHASE7_SESSION_SUMMARY.md NEXT_STEPS.md START_HERE_NEXT_SESSION.md

echo "=== Creating commit ==="
git commit -m ":ok_hand: IMPROVE: Phase 7 partial completion + MANAGEMENT DECISION: implement function-level JIT NOW

**Phase 7 Progress**: String constant handling + root cause analysis complete

**🚨 MANAGEMENT DECISION**: Function-level JIT MUST be implemented NOW for v0.9.0
- This is P0 priority, NOT deferred to v1.0
- v0.9.0 CANNOT ship without function-level JIT
- Next session MUST start with this implementation
- Timeline: 2-4 weeks, start immediately

Changes to src/jit.rs:
- Modified is_supported_opcode() to accept all constant types (not just Int/Bool)
- Modified translate_instruction LoadConst to push placeholder 0 for non-Int/Bool constants
- Result: Loops can now JIT-compile even when function contains string constants outside loop
- Small improvement, but main work (function-level JIT) still required

**Root Cause Analysis Complete**:
- Identified why Fibonacci is 42x slower than Python: NO function-level JIT
- Current JIT only triggers on JumpBack (loops), not on function calls/entry
- Fibonacci functions have no loops internally, only recursive calls
- Each recursive call goes through slow interpreter
- Solution: Function-level JIT compilation (detailed plan in ROADMAP)

**ROADMAP.md Updates** (Critical):
- Changed status to P0 CRITICAL - BLOCKING v0.9.0
- Added detailed implementation plan (Week 1-2: core arch, Week 3-4: optimization)
- Removed all deferral language - this MUST be done NOW
- Added clear directive: \"START HERE\" for next session
- Success criteria: v0.9.0 requires fibonacci faster than Python
- Performance targets: <50ms fib(30), <20ms fib_iter(100k)

**CHANGELOG.md Updates**:
- Added Phase 7 section with management decision
- Documented string constant handling improvement
- Clear directive: Function-level JIT is immediate next task

**Documentation Files**:
- PHASE7_SESSION_SUMMARY.md: Updated with management decision
- NEXT_STEPS.md: Clear instructions - implement function JIT NOW
- Both files emphasize: NO DEFERRAL, P0 priority, blocking release

**Performance Targets** (MUST ACHIEVE for v0.9.0):
Current:
- Fib Recursive (n=30):  11,782ms (42x slower than Python) - UNACCEPTABLE
- Fib Iterative (100k):  918ms (7.8x slower than Python) - UNACCEPTABLE

Required:
- Fib Recursive (n=30):  <50ms (5-10x FASTER than Python) - NON-NEGOTIABLE
- Fib Iterative (100k):  <20ms (5-10x FASTER than Python) - NON-NEGOTIABLE
- ALL benchmarks: >= Python performance (minimum)

**Next Session MUST**:
1. Read ROADMAP.md Phase 7 for complete implementation plan
2. Start with function call tracking in VM (OpCode::Call handler)
3. Implement function body compilation in JIT
4. Add Call opcode JIT support (jump to native code)
5. Optimize recursive functions
6. Achieve performance targets

**Why This Is Critical**:
- 42x slower than Python is unacceptable for production language
- Function calls are everywhere in real code (not edge case)
- This determines if Ruff is serious alternative or toy project
- Loop-level JIT alone is insufficient

**Files Changed**:
- src/jit.rs: String constant handling fix
- CHANGELOG.md: Phase 7 documentation + management decision
- ROADMAP.md: P0 priority + detailed implementation plan
- PHASE7_SESSION_SUMMARY.md: Updated with decision
- NEXT_STEPS.md: Clear directive for next session

Related: #phase7 #jit #performance #management-decision #blocking #p0"

**Phase 7 Progress**: Expanded JIT opcode coverage for string constants

Changes to src/jit.rs:
- Modified is_supported_opcode() to accept all constant types (not just Int/Bool)
- Modified translate_instruction LoadConst to push placeholder 0 for non-Int/Bool constants
- Result: Loops can now JIT-compile even when function contains string constants outside loop
- Loops with print statements INSIDE still won't JIT (Call opcode unsupported) - expected behavior

**Root Cause Analysis Complete**:
- Identified why Fibonacci is 42x slower than Python: NO function-level JIT
- Current JIT only triggers on JumpBack (loops), not on function calls/entry
- Fibonacci functions have no loops internally, only recursive calls
- Each recursive call goes through slow interpreter
- Solution requires function-level JIT compilation (major architectural change)

**Realistic Assessment** (Updated ROADMAP.md):
- Current JIT: EXCELLENT for loops (matches Python performance)
- Current JIT: POOR for recursive functions (needs function-level JIT)
- Function-level JIT: Estimated 2-4 weeks of focused development
- Recommendation: Ship v0.9.0 with current JIT, defer function JIT to v1.0

**CHANGELOG.md Updates**:
- Added Phase 7 section documenting progress
- Documented string constant handling improvement
- Explained root causes and next steps
- Set realistic expectations for v0.9.0 vs v1.0 performance

**ROADMAP.md Updates**:
- Updated Phase 7 status from \"IN PROGRESS\" to \"PARTIALLY COMPLETE\"
- Added detailed root cause analysis findings
- Added \"Path Forward\" section with 3 options (Ambitious, Pragmatic, Compromise)
- Revised success criteria to be realistic for v0.9.0
- Documented v0.9.0 deliverables vs v1.0 goals
- Removed \"BLOCKING\" status - v0.9.0 can ship with current JIT

**Performance Summary**:
✅ Array Sum: 52ms (matches Python) - JIT works great!
✅ Hash Map: 34ms (matches Python) - JIT works great!
❌ Fibonacci: 42x slower - needs function-level JIT (v1.0 feature)

**Files Changed**:
- src/jit.rs: String constant handling fix
- CHANGELOG.md: Phase 7 documentation
- ROADMAP.md: Realistic assessment and path forward

Related: #phase7 #jit #performance #realistic-planning"

echo ""
echo "=== Commit created! ==="
git log -1 --stat

echo ""
echo "=== Pushing to remote ==="
git push origin main

echo ""
echo "✅ Done!"

