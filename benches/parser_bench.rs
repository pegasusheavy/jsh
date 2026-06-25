//! Parser benchmarks for Franken Shell
//!
//! Benchmarks parsing performance for various shell constructs.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use franken_shell::lexer::Lexer;
use franken_shell::parser::Parser;
use std::hint::black_box;

/// Helper to parse input
fn parse_input(input: &str) -> franken_shell::ast::Program {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse_program().unwrap()
}

/// Simple command parsing
fn bench_simple_commands(c: &mut Criterion) {
    let inputs = [
        ("single", "echo hello"),
        ("with_args", "ls -la /home/user --color=auto"),
        ("with_vars", "echo $HOME $USER $PATH"),
        ("assignment", "x=hello; y=world; echo $x $y"),
    ];

    let mut group = c.benchmark_group("parser_simple");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// Pipeline parsing
fn bench_pipelines(c: &mut Criterion) {
    let inputs = [
        ("two_stage", "ls | grep foo"),
        ("three_stage", "cat file | grep pattern | sort"),
        (
            "five_stage",
            "cat file | grep -v '^#' | sort | uniq | wc -l",
        ),
        (
            "with_redirects",
            "cat < input.txt | grep pattern > output.txt 2>&1",
        ),
    ];

    let mut group = c.benchmark_group("parser_pipelines");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// Control flow parsing
fn bench_control_flow(c: &mut Criterion) {
    let inputs = [
        ("if_simple", "if [ $x -eq 1 ]; then echo one; fi"),
        (
            "if_elif_else",
            "if [ $x -eq 1 ]; then echo one; elif [ $x -eq 2 ]; then echo two; else echo other; fi",
        ),
        ("for_simple", "for i in 1 2 3; do echo $i; done"),
        (
            "for_nested",
            "for i in 1 2 3; do for j in a b c; do echo $i$j; done; done",
        ),
        ("while_simple", "while [ $x -lt 10 ]; do x=$((x+1)); done"),
        (
            "case_simple",
            "case $x in 1) echo one;; 2) echo two;; *) echo other;; esac",
        ),
        (
            "case_complex",
            "case $x in 1|one) echo one;; 2|two|II) echo two;; [3-9]) echo digit;; *) echo other;; esac",
        ),
    ];

    let mut group = c.benchmark_group("parser_control_flow");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// Function definition parsing
fn bench_functions(c: &mut Criterion) {
    let inputs = [
        ("simple", "foo() { echo hello; }"),
        ("with_params", "greet() { echo \"Hello, $1!\"; }"),
        (
            "with_local",
            "foo() { local x=1; local y=2; echo $((x+y)); }",
        ),
        (
            "complex",
            r#"process_file() {
    local file="$1"
    local count=0
    while read line; do
        count=$((count+1))
        echo "$count: $line"
    done < "$file"
    return $count
}"#,
        ),
    ];

    let mut group = c.benchmark_group("parser_functions");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// franken-specific syntax parsing
fn bench_jsh_syntax(c: &mut Criterion) {
    let inputs = [
        (
            "match_simple",
            "match $x { 1 => echo one; * => echo other }",
        ),
        (
            "match_complex",
            "match $cmd { start => start_service; stop => stop_service; restart => { stop_service; start_service }; * => echo \"Unknown: $cmd\" }",
        ),
        (
            "loop_simple",
            "loop { read line; if [ \"$line\" = quit ]; then break; fi }",
        ),
        ("let_binding", "let x = 42; let name = \"world\""),
        (
            "const_binding",
            "const PI = 3.14159; const VERSION = \"1.0.0\"",
        ),
        (
            "try_catch",
            "try { risky_operation } catch e { echo \"Error: $e\" } finally { cleanup }",
        ),
        ("fn_simple", "fn greet { echo \"Hello, $1!\" }"),
        (
            "fn_complex",
            "fn process { let result = $(command \"$1\"); if [ -n \"$result\" ]; then echo \"$result\"; fi }",
        ),
    ];

    let mut group = c.benchmark_group("parser_jsh_syntax");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// Realistic script parsing
fn bench_realistic_scripts(c: &mut Criterion) {
    let small_script = r#"
name="World"
echo "Hello, $name!"
for i in 1 2 3; do
    echo $i
done
"#;

    let medium_script = r#"
#!/usr/bin/env franken
set -e

name="${1:-World}"

greet() {
    local person="$1"
    echo "Hello, $person!"
}

main() {
    if [ -z "$name" ]; then
        echo "Usage: $0 <name>"
        return 1
    fi

    greet "$name"

    for i in $(seq 1 5); do
        echo "Count: $i"
    done
}

main "$@"
"#;

    let large_script = r#"
#!/usr/bin/env franken
set -e

readonly VERSION="1.0.0"

log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                echo "Usage: $0 [options]"
                exit 0
                ;;
            -v|--version)
                echo "Version: $VERSION"
                exit 0
                ;;
            -d|--debug)
                DEBUG=1
                ;;
            *)
                ARGS="$ARGS $1"
                ;;
        esac
        shift
    done
}

process() {
    local item="$1"
    log "Processing: $item"

    if [ -f "$item" ]; then
        cat "$item" | while read line; do
            echo "  $line"
        done
    elif [ -d "$item" ]; then
        for f in "$item"/*; do
            process "$f"
        done
    else
        log "Unknown: $item"
    fi
}

main() {
    log "Starting..."

    for arg in $ARGS; do
        process "$arg"
    done

    log "Done."
}

parse_args "$@"
main
"#;

    let inputs = [
        ("small", small_script),
        ("medium", medium_script),
        ("large", large_script),
    ];

    let mut group = c.benchmark_group("parser_scripts");
    group.sample_size(50);

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

/// Deep nesting parsing
fn bench_nesting(c: &mut Criterion) {
    // Generate nested if statements
    fn nested_ifs(depth: usize) -> String {
        let mut s = String::new();
        for i in 0..depth {
            s.push_str(&format!("if [ $x -eq {} ]; then ", i));
        }
        s.push_str("echo deep");
        for _ in 0..depth {
            s.push_str("; fi");
        }
        s
    }

    // Generate nested loops
    fn nested_loops(depth: usize) -> String {
        let mut s = String::new();
        for i in 0..depth {
            s.push_str(&format!("for v{} in 1 2 3; do ", i));
        }
        s.push_str("echo nested");
        for _ in 0..depth {
            s.push_str("; done");
        }
        s
    }

    let inputs = [
        ("if_depth_3", nested_ifs(3)),
        ("if_depth_5", nested_ifs(5)),
        ("if_depth_10", nested_ifs(10)),
        ("loop_depth_3", nested_loops(3)),
        ("loop_depth_5", nested_loops(5)),
    ];

    let mut group = c.benchmark_group("parser_nesting");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), input, |b, input| {
            b.iter(|| parse_input(black_box(input)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_commands,
    bench_pipelines,
    bench_control_flow,
    bench_functions,
    bench_jsh_syntax,
    bench_realistic_scripts,
    bench_nesting,
);

criterion_main!(benches);
