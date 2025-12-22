# jsh Performance Optimization TODO

Based on benchmarking results and flamegraph analysis, the following optimizations are prioritized by impact.

## Benchmark Summary (Baseline)

| Component | Throughput | Time per Operation |
|-----------|------------|-------------------|
| Lexer only | ~140 MiB/s | ~970 ns |
| Lexer + Parser | ~25 MiB/s | ~5.5 µs |
| Full Pipeline | ~1.3 MiB/s | ~100 µs |

### Key Findings:
- **Lexer**: High performance (~140 MiB/s)
- **Parser**: Adds ~5x overhead over lexer
- **Interpreter**: Adds ~20x overhead over parser (execution is the bottleneck)
- **Interpreter reuse**: Reusing interpreter instances shows significant speedup

---

## High Priority (Critical Path Optimizations)

### 1. String Allocation Reduction
**Impact: HIGH** | **Effort: MEDIUM**

- [ ] Use `Cow<'a, str>` for token values instead of `String`
- [ ] Implement string interning for frequently used strings (variable names, keywords)
- [ ] Use `SmallVec` for small argument lists (most commands have <8 args)
- [ ] Pool allocators for AST nodes

**Files**: `src/lexer.rs`, `src/token.rs`, `src/ast.rs`

```rust
// Example: String interning
use lasso::{Spur, Rodeo};
struct Interner {
    rodeo: Rodeo,
}
```

### 2. Variable Lookup Optimization
**Impact: HIGH** | **Effort: MEDIUM**

- [ ] Replace `HashMap<String, String>` with more efficient data structure
- [ ] Consider using `FxHashMap` from `rustc-hash` (faster hashing)
- [ ] Implement variable scope stack with arena allocation
- [ ] Cache frequently accessed variables (`$PATH`, `$HOME`, etc.)

**Files**: `src/interpreter/mod.rs`, `src/interpreter/expansion.rs`

### 3. Arithmetic Expression Caching
**Impact: MEDIUM** | **Effort: LOW**

- [ ] Cache parsed arithmetic expressions
- [ ] Pre-compute constant expressions during parsing
- [ ] Use native integer operations where possible

**Files**: `src/interpreter/arithmetic.rs`

### 4. AST Node Optimization
**Impact: HIGH** | **Effort: HIGH**

- [ ] Use arena allocation for AST nodes
- [ ] Implement a flat representation for simple commands
- [ ] Consider bytecode compilation for hot loops

**Files**: `src/ast.rs`, `src/parser/mod.rs`

---

## Medium Priority (Throughput Improvements)

### 5. Lexer Optimizations
**Impact: MEDIUM** | **Effort: MEDIUM**

- [ ] Use `memchr` for fast character scanning
- [ ] Implement SIMD-accelerated keyword detection
- [ ] Pre-compute keyword lookup table (perfect hashing)
- [ ] Batch token allocation

**Files**: `src/lexer.rs`

```rust
// Example: memchr for fast scanning
use memchr::memchr2;
fn find_word_end(input: &[u8]) -> usize {
    memchr2(b' ', b'\n', input).unwrap_or(input.len())
}
```

### 6. Parser Optimization
**Impact: MEDIUM** | **Effort: MEDIUM**

- [ ] Reduce token cloning in parser
- [ ] Use iterators instead of collecting into vectors
- [ ] Implement look-ahead buffer to avoid re-parsing
- [ ] Cache compound statement detection results

**Files**: `src/parser/mod.rs`, `src/parser/compound.rs`

### 7. Built-in Command Dispatch
**Impact: MEDIUM** | **Effort: LOW**

- [ ] Use perfect hashing for builtin lookup
- [ ] Implement static dispatch for common builtins
- [ ] Pre-allocate output buffers for `echo`, `printf`

**Files**: `src/builtins/mod.rs`, `src/builtins/output.rs`

### 8. Word Expansion Caching
**Impact: MEDIUM** | **Effort: MEDIUM**

- [ ] Cache expanded words that don't contain variables
- [ ] Implement lazy expansion (only expand when needed)
- [ ] Pre-expand constant strings during parsing

**Files**: `src/interpreter/expansion.rs`

---

## Low Priority (Long-term Improvements)

### 9. JIT Compilation for Hot Loops
**Impact: HIGH** | **Effort: VERY HIGH**

- [ ] Detect hot loops (executed >100 times)
- [ ] Compile to native code using Cranelift or LLVM
- [ ] Implement basic block caching

### 10. Parallel Execution
**Impact: MEDIUM** | **Effort: HIGH**

- [ ] Parallelize pipeline stages where independent
- [ ] Async I/O for external command execution
- [ ] Parallel glob expansion

### 11. Memory Layout Optimization
**Impact: MEDIUM** | **Effort: HIGH**

- [ ] Optimize struct layouts for cache efficiency
- [ ] Use `#[repr(C)]` where beneficial
- [ ] Implement custom allocators for hot paths

### 12. Bytecode Interpreter
**Impact: HIGH** | **Effort: VERY HIGH**

- [ ] Design bytecode instruction set
- [ ] Implement bytecode compiler
- [ ] Stack-based VM for faster execution

---

## Bug Fixes Required for Performance Testing

### Critical Bugs (Blocking Full Benchmarks)

- [ ] **Variable expansion in `local` builtin**: `local item="$1"` doesn't expand `$1`
- [ ] **`>&2` redirection syntax**: Currently causes syntax error
- [ ] **Fish-style builtins**: `string`, `math`, `contains` may hang or error

### Test Coverage Gaps

- [ ] Add unit tests for arithmetic evaluation edge cases
- [ ] Add benchmarks for glob expansion
- [ ] Add stress tests for deeply nested scopes

---

## Quick Wins (Low Hanging Fruit)

These can be implemented quickly with minimal risk:

1. [ ] Replace `HashMap` with `FxHashMap` in interpreter (~5 lines change)
2. [ ] Add `#[inline]` to hot path functions
3. [ ] Use `String::with_capacity()` for known-size strings
4. [ ] Pre-size vectors in parser (`Vec::with_capacity`)
5. [ ] Move error strings to `const` to avoid allocation

---

## Profiling Commands

```bash
# Generate flamegraph
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin jsh -o flamegraph.svg -- -c 'your_script_here'

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench full_pipeline_bench

# Profile with perf
perf record -g ./target/release/jsh -c 'your_script_here'
perf report

# Memory profiling
valgrind --tool=massif ./target/release/jsh -c 'your_script_here'
```

---

## Target Performance Goals

| Component | Current | Target | Improvement |
|-----------|---------|--------|-------------|
| Lexer | 140 MiB/s | 200 MiB/s | 1.4x |
| Parser | 25 MiB/s | 50 MiB/s | 2x |
| Full Pipeline | 1.3 MiB/s | 5 MiB/s | 3.8x |
| Simple echo | 37 µs | 10 µs | 3.7x |
| Loop iteration | ~25 µs | ~5 µs | 5x |

---

## Dependencies to Add

```toml
[dependencies]
# Fast hashing
rustc-hash = "1.1"  # FxHashMap

# String interning
lasso = "0.7"

# Fast character search
memchr = "2.6"

# Small vector optimization
smallvec = "1.11"

# Arena allocation
bumpalo = "3.14"
```

---

## Implementation Priority Order

1. **Week 1**: Quick wins + FxHashMap migration
2. **Week 2**: String allocation reduction
3. **Week 3**: Variable lookup optimization
4. **Week 4**: Parser optimizations
5. **Ongoing**: Bytecode/JIT research

---

## Notes

- Always benchmark before and after changes
- Use `#[cfg(test)]` for benchmark-only code
- Consider backward compatibility when changing data structures
- Profile memory as well as CPU time
- Test with realistic scripts from real-world use cases

Last updated: 2024-12-22
Benchmarks run on: Linux WSL2

