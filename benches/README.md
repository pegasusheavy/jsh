# Franken Shell Benchmarks

Comprehensive benchmarking suite for Franken Shell performance.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench lexer_bench
cargo bench --bench parser_bench
cargo bench --bench interpreter_bench
cargo bench --bench full_pipeline_bench

# Run specific benchmark group
cargo bench -- lexer_simple
cargo bench -- parser_control_flow
cargo bench -- interp_builtins

# Quick run with shorter measurement time
cargo bench -- --warm-up-time 1 --measurement-time 2 lexer_simple
```

## Benchmark Suites

### `lexer_bench` - Lexer/Tokenizer Performance

Tests tokenization speed for:
- **lexer_simple**: Basic commands (echo, ls, etc.)
- **lexer_variables**: Variable expansion ($VAR, ${VAR:-default})
- **lexer_strings**: Single/double quoted strings with escapes
- **lexer_pipelines**: Pipes, redirections, heredocs
- **lexer_control_flow**: if/for/while/case statements
- **lexer_franken_syntax**: franken-specific match/loop/let/try/fn
- **lexer_scripts**: Realistic small/medium/large scripts

### `parser_bench` - Parser Performance

Tests parsing speed for:
- **parser_simple**: Basic commands and assignments
- **parser_pipelines**: Multi-stage pipelines with redirects
- **parser_control_flow**: Control flow constructs
- **parser_functions**: Function definitions
- **parser_franken_syntax**: franken-specific syntax
- **parser_scripts**: Realistic scripts
- **parser_nesting**: Deeply nested structures

### `interpreter_bench` - Execution Performance

Tests execution speed for:
- **interp_variables**: Variable assignment and expansion
- **interp_builtins**: Built-in commands (echo, test, etc.)
- **interp_control_flow**: if/for/while/case execution
- **interp_functions**: Function calls and recursion
- **interp_logical**: && and || operators
- **interp_arithmetic**: Arithmetic expressions
- **interp_franken**: franken match/let/loop features
- **interp_fish**: Fish builtins (string, math, contains)
- **interp_options**: Shell option handling

### `full_pipeline_bench` - End-to-End Performance

Tests full lex→parse→execute pipeline:
- **pipeline_breakdown**: Compare lex vs parse vs execute times
- **pipeline_sizes**: Tiny to large script scaling
- **pipeline_complexity**: Linear/branching/looping/nested
- **pipeline_styles**: POSIX vs Bash vs Fish vs franken syntax
- **interpreter_reuse**: New vs reused interpreter
- **loop_scaling**: Performance with varying loop counts

## Output

Benchmark results are saved to `target/criterion/` with:
- HTML reports (open `report/index.html`)
- JSON data for analysis
- Historical comparisons

## Performance Tips

Based on benchmarks:
1. **Lexer throughput**: ~60-110 MiB/s depending on complexity
2. **Interpreter reuse**: Parsing once and executing multiple times is significantly faster
3. **Builtin commands**: Much faster than external process execution
4. **Loop iterations**: Linear scaling with iteration count

## Adding New Benchmarks

1. Add benchmark function to appropriate `*_bench.rs` file
2. Add to `criterion_group!` at bottom of file
3. Run `cargo bench -- <new_group_name>` to test

Example:

```rust
fn bench_new_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_feature");

    group.bench_function("test_case", |b| {
        b.iter(|| {
            // Code to benchmark
        })
    });

    group.finish();
}

criterion_group!(benches, ..., bench_new_feature);
```
