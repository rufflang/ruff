# Ruff Performance Status - Pre-Optimization

**Date:** January 30, 2026  
**Baseline:** benchmarks/cross-language/results/benchmark_20260130_153641.txt

## Summary

Ruff is **30-70x faster** than Python on compute workloads but **1620x slower** on dictionary operations.

## Working Benchmarks ✅

| Benchmark | Ruff | Python | Speedup |
|-----------|------|--------|---------|
| Fibonacci Recursive (n=30) | 7.5ms | 374ms | **50x faster** |
| Fibonacci Iterative (n=100k) | 1.3ms | N/A | Working |
| Array Sum (1M elements) | 1.1ms | 60ms | **54x faster** |

## Blocked Benchmark ❌

| Benchmark | Ruff | Python | Status |
|-----------|------|--------|--------|
| Dict Operations (100k items) | **HANGS** | 36ms | **~1000x slower** |
| Dict Operations (1000 items) | 405ms | 0.25ms | **1620x slower** |

### Why It's Blocked

- 1000 dict operations take 405ms in Ruff vs 0.25ms in Python
- 100k operations would take ~40 seconds (causes timeout/hang)
- Root cause: `LoadVar`/`StoreVar` clone entire HashMap on every access
- Creates O(n²) behavior for growing dicts

## Optimization Plan 🎯

### Goal
Reduce dict operations to **<10ms for 1000 items** (40x improvement)

### Approach
Implement new opcodes:
- `IndexGetInPlace` - Read from local dict/array without cloning
- `IndexSetInPlace` - Write to local dict/array without cloning

### Expected Outcome
- 1000 items: 405ms → <10ms
- 100k items: ~40s → ~1s (benchmark can complete)
- Ruff becomes competitive with Python on dict operations

## Files to Review

Investigation:
- `notes/2026-01-30_dict_optimization_investigation.md`
- `notes/bug_dict_index_assignment_hangs.md`

Benchmarks:
- `benchmarks/cross-language/bench_hashmap.ruff` (1000 items)
- `benchmarks/cross-language/profile_dict.ruff` (detailed profiling)
- `benchmarks/cross-language/test_hashmap.py` (Python baseline)

Results:
- `benchmarks/cross-language/results/benchmark_20260130_*.txt`
