# Franken Shell TODO

## 🚀 Feature Roadmap

### Nushell Compatibility
**Priority: HIGH** | **Status: NOT STARTED**

Add Nushell-style structured data and pipeline support:

- [ ] **Structured Data Types**
  - [ ] Tables (rows and columns)
  - [ ] Records (key-value maps)
  - [ ] Lists with typed elements
  - [ ] Duration, filesize, date types

- [ ] **Pipeline Operators**
  - [ ] `|` for structured data pipelines
  - [ ] `where` for filtering rows
  - [ ] `select` for column selection
  - [ ] `get` for accessing nested data
  - [ ] `sort-by` for sorting tables
  - [ ] `group-by` for grouping data

- [ ] **Built-in Commands (Nu-style)**
  - [ ] `ls` returning table with name, type, size, modified
  - [ ] `ps` returning process table
  - [ ] `sys` for system information
  - [ ] `open` for reading files as structured data (JSON, YAML, TOML, CSV)
  - [ ] `to json`, `to yaml`, `to csv` for output formatting
  - [ ] `from json`, `from yaml`, `from csv` for parsing

- [ ] **Cell Paths**
  - [ ] `$data.field` for record access
  - [ ] `$data.0` for list indexing
  - [ ] `$data.column.0` for table cell access

- [ ] **Type System**
  - [ ] Runtime type checking
  - [ ] Type annotations for functions
  - [ ] Automatic type coercion where sensible

**Files to create/modify:**
- `src/nu/mod.rs` - Nushell compatibility module
- `src/nu/table.rs` - Table data structure
- `src/nu/value.rs` - Nu-style value types
- `src/nu/commands.rs` - Nu-compatible commands
- `src/builtins/nu.rs` - Nu-style builtins

---

## 📊 Benchmark Results

### Current Performance (2025-12-24)

| Component | Throughput | Time |
|-----------|------------|------|
| Lexer | ~122 MiB/s | ~1.1 µs |
| Lex + Parse | ~28.5 MiB/s | ~4.8 µs |
| Full Pipeline | ~3.0 MiB/s | ~45 µs |

### Improvement vs Original Baseline

| Component | Original | Current | Change |
|-----------|----------|---------|--------|
| Lexer | 140 MiB/s | 122 MiB/s | -13% |
| Parser | 25 MiB/s | 28.5 MiB/s | **+14%** |
| Full Pipeline | 1.3 MiB/s | 3.0 MiB/s | **+130%** |

### Target Goals

| Component | Current | Target | Status |
|-----------|---------|--------|--------|
| Lexer | 122 MiB/s | 200 MiB/s | 61% |
| Parser | 28.5 MiB/s | 50 MiB/s | 57% |
| Full Pipeline | 3.0 MiB/s | 5 MiB/s | **60%** |

---

## ✅ Completed Work

### Performance Optimizations
All high-priority optimizations implemented:
- FxHashMap, string interning (`lasso`), variable caching
- SmallVec for positional params, arithmetic expression caching
- Arena allocation (`bumpalo`), object pools
- PHF perfect hashing for keywords and builtins
- Parallel glob expansion (`rayon`), async I/O (`crossbeam-channel`)
- JIT compilation infrastructure (`cranelift`, optional)
- Bytecode interpreter (60+ opcodes, stack-based VM)

### Bug Fixes
- Variable expansion in `local` builtin
- `>&2` and `2>&1` redirection syntax
- Fish-style builtins (`string`, `math`, `contains`)

### Testing
- 58 arithmetic evaluation edge case tests
- Glob expansion benchmarks
- 23 scope stress tests (up to 500 depth)

---

## 📝 Notes

- Always benchmark before and after changes
- Profile memory as well as CPU time
- Test with realistic scripts from real-world use cases

---

Last updated: 2025-12-24
