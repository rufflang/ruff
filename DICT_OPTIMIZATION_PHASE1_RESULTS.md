# Dict Optimization - Phase 1 Results

**Date:** January 30, 2026  
**Implementation:** IndexGetInPlace and IndexSetInPlace opcodes

## Performance Improvements

### 1000 Dict Operations

| Operation | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Dict writes | 181ms | 5ms | **36x faster** |
| Dict reads | 233ms | 194ms | **1.2x faster** |
| Combined | 405ms | 346ms | **1.17x faster** |

### Key Wins
- ✅ Dict writes are now **36x faster** (181ms → 5ms)
- ✅ Eliminated O(n²) behavior for dict population
- ✅ Total improvement: **1.17x** (not the 40x we targeted, but progress)

### Remaining Issue
- Dict reads still clone values (194ms for 1000 reads)
- 100k operations still timeout (>60s)

## What Changed

### New Opcodes
- `IndexGetInPlace(var_name)`: Read from local dict/array without full HashMap clone
- `IndexSetInPlace(var_name)`: Write to local dict/array in-place

### Compiler Optimization
- Detects `local_var[index]` patterns in expressions and assignments
- Emits optimized opcodes instead of LoadVar + IndexGet/IndexSet + StoreVar sequence

### VM Implementation
- Modifies local variables in-place (in call frame's locals HashMap)
- Avoids cloning entire HashMap on every access
- Pushes null to maintain stack balance for Pop instructions

## Why Not 40x?

The 36x improvement on writes is close to our target, but reads are still slow because:
1. Getting a value from HashMap still calls `.cloned()`
2. This clones the Value enum, which contains the data
3. For primitive values this is cheap, but we still do the HashMap lookup overhead

## Next Steps

To get the full 40x improvement:
1. Consider reference-counted values (Arc<Value>) to avoid cloning
2. Or implement a more sophisticated value representation
3. Or optimize the HashMap implementation itself

For now, this is a solid first step that makes dict writes competitive.
