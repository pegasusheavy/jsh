//! Interpreter benchmarks for Franken Shell
//!
//! Benchmarks execution performance for various shell constructs.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use franken_shell::interpreter::Interpreter;
use franken_shell::lexer::Lexer;
use franken_shell::parser::Parser;

/// Helper to execute input
fn execute_input(input: &str) -> franken_shell::interpreter::ExitStatus {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.execute(&program).unwrap()
}

/// Variable operations benchmark
fn bench_variables(c: &mut Criterion) {
    let inputs = [
        ("assign_single", "x=hello"),
        ("assign_multiple", "x=1; y=2; z=3; a=4; b=5"),
        ("assign_and_read", "x=hello; y=$x; z=$y"),
        (
            "complex_expansion",
            "x=hello; y=${x:-default}; z=${y:+set}",
        ),
    ];

    let mut group = c.benchmark_group("interp_variables");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Builtin commands benchmark
fn bench_builtins(c: &mut Criterion) {
    let inputs = [
        ("echo_simple", "echo hello"),
        ("echo_vars", "x=world; echo hello $x"),
        ("printf_simple", "printf '%s' hello"),
        ("test_true", "test 1 -eq 1"),
        ("test_false", "test 1 -eq 2"),
        ("true_cmd", "true"),
        ("false_cmd", "false"),
        ("pwd", "pwd"),
    ];

    let mut group = c.benchmark_group("interp_builtins");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Control flow benchmark
fn bench_control_flow(c: &mut Criterion) {
    let inputs = [
        ("if_true", "if true; then echo yes; fi"),
        ("if_false", "if false; then echo yes; else echo no; fi"),
        (
            "if_elif",
            "x=2; if [ $x -eq 1 ]; then echo one; elif [ $x -eq 2 ]; then echo two; fi",
        ),
        ("for_5", "for i in 1 2 3 4 5; do x=$i; done"),
        ("for_10", "for i in 1 2 3 4 5 6 7 8 9 10; do x=$i; done"),
        ("while_5", "x=0; while [ $x -lt 5 ]; do x=$((x+1)); done"),
        ("while_10", "x=0; while [ $x -lt 10 ]; do x=$((x+1)); done"),
        (
            "case_match",
            "x=two; case $x in one) echo 1;; two) echo 2;; *) echo 0;; esac",
        ),
    ];

    let mut group = c.benchmark_group("interp_control_flow");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Function calls benchmark
fn bench_functions(c: &mut Criterion) {
    let inputs = [
        ("define_call", "foo() { echo hello; }; foo"),
        (
            "with_args",
            "greet() { echo hello $1; }; greet world",
        ),
        (
            "recursive_3",
            "countdown() { if [ $1 -gt 0 ]; then countdown $(($1-1)); fi }; countdown 3",
        ),
        (
            "call_10",
            "noop() { true; }; noop; noop; noop; noop; noop; noop; noop; noop; noop; noop",
        ),
    ];

    let mut group = c.benchmark_group("interp_functions");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Logical operators benchmark
fn bench_logical(c: &mut Criterion) {
    let inputs = [
        ("and_true", "true && echo yes"),
        ("and_false", "false && echo no"),
        ("or_true", "true || echo no"),
        ("or_false", "false || echo yes"),
        ("chain_and", "true && true && true && true && echo all"),
        ("chain_or", "false || false || false || true"),
        ("mixed", "true && false || true && echo complex"),
    ];

    let mut group = c.benchmark_group("interp_logical");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Arithmetic benchmark
fn bench_arithmetic(c: &mut Criterion) {
    let inputs = [
        ("add", "x=$((1+2))"),
        ("sub", "x=$((10-5))"),
        ("mul", "x=$((3*4))"),
        ("div", "x=$((20/4))"),
        ("mod", "x=$((17%5))"),
        ("complex", "x=$((1+2*3-4/2))"),
        ("nested", "x=$((((1+2)*3)-4))"),
        ("vars", "a=5; b=3; x=$((a*b+a-b))"),
    ];

    let mut group = c.benchmark_group("interp_arithmetic");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// franken-specific features benchmark
fn bench_jsh_features(c: &mut Criterion) {
    let inputs = [
        (
            "match_simple",
            "x=2; match $x { 1 => echo one; 2 => echo two; * => echo other }",
        ),
        ("let_binding", "let x = 42; let y = $x"),
        ("loop_break", "x=0; loop { x=$((x+1)); if [ $x -ge 3 ]; then break; fi }"),
    ];

    let mut group = c.benchmark_group("interp_jsh");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Fish builtins benchmark
fn bench_fish_builtins(c: &mut Criterion) {
    let inputs = [
        ("string_length", "string length hello"),
        ("string_upper", "string upper hello"),
        ("string_lower", "string lower HELLO"),
        ("string_split", "string split ',' 'a,b,c'"),
        ("string_join", "string join '-' a b c"),
        ("math_simple", "math '2 + 3'"),
        ("math_complex", "math '2 * 3 + 4'"),
        ("contains_found", "contains apple apple banana cherry"),
        ("contains_missing", "contains grape apple banana cherry"),
    ];

    let mut group = c.benchmark_group("interp_fish");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

/// Shell options benchmark
fn bench_shell_options(c: &mut Criterion) {
    let inputs = [
        ("set_errexit", "set -e; true"),
        ("set_multiple", "set -e; set -u; set -x; set +x"),
        ("set_by_name", "set -o errexit; set +o errexit"),
        ("set_positional", "set -- a b c; x=$1$2$3"),
    ];

    let mut group = c.benchmark_group("interp_options");

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| execute_input(black_box(input)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_variables,
    bench_builtins,
    bench_control_flow,
    bench_functions,
    bench_logical,
    bench_arithmetic,
    bench_jsh_features,
    bench_fish_builtins,
    bench_shell_options,
);

criterion_main!(benches);

