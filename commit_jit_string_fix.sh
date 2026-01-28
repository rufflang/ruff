#!/bin/bash
# Script to commit JIT improvements

cd /Users/robertdevore/2026/ruff

echo "=== Git Status ==="
git status --short

echo ""
echo "=== Git Diff ==="
git diff src/jit.rs | head -100

echo ""
read -p "Commit these changes? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    git add src/jit.rs
    git commit -m ":ok_hand: IMPROVE: allow JIT compilation of loops with string constants outside loop body

- Modified is_supported_opcode() to accept all constant types, not just Int/Bool
- Modified translate_instruction LoadConst case to push placeholder 0 for non-Int/Bool constants  
- This allows loops to be JIT-compiled even when the function contains print statements after the loop
- Loops with prints INSIDE still won't JIT (Call opcode unsupported), which is expected
- Partial progress on Phase 7: String Constant Handling"
    
    echo ""
    echo "=== Committed! ==="
    git log -1 --oneline
fi
