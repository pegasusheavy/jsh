//! Lexer benchmarks for Franken Shell
//!
//! Benchmarks tokenization performance across various input types.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use franken_shell::lexer::Lexer;
use std::hint::black_box;

/// Simple command tokenization
fn bench_simple_command(c: &mut Criterion) {
    let inputs = [
        ("tiny", "echo hello"),
        ("small", "echo hello world foo bar"),
        (
            "medium",
            "ls -la /home/user/documents --color=auto | grep pattern",
        ),
        (
            "large",
            r#"for i in 1 2 3 4 5 6 7 8 9 10; do echo "Number: $i"; done"#,
        ),
    ];

    let mut group = c.benchmark_group("lexer_simple");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// Variable expansion tokenization
fn bench_variable_expansion(c: &mut Criterion) {
    let inputs = [
        ("simple_var", "$HOME"),
        ("brace_var", "${HOME}"),
        ("default_var", "${HOME:-/home/default}"),
        ("special_vars", "$? $$ $! $# $@ $* $0 $1"),
        ("mixed", r#"echo "User: $USER, Home: ${HOME}, Status: $?""#),
    ];

    let mut group = c.benchmark_group("lexer_variables");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// String tokenization
fn bench_strings(c: &mut Criterion) {
    let inputs = [
        ("single_quoted", "'hello world'"),
        ("double_quoted", "\"hello world\""),
        ("with_escapes", r#""hello\nworld\ttab""#),
        ("with_variables", r#""Hello $USER, your home is $HOME""#),
        (
            "long_string",
            r#""This is a much longer string that contains various characters and might be used in real scripts for documentation or output purposes.""#,
        ),
    ];

    let mut group = c.benchmark_group("lexer_strings");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// Pipeline and redirection tokenization
fn bench_pipelines(c: &mut Criterion) {
    let inputs = [
        ("simple_pipe", "ls | grep foo"),
        (
            "multi_pipe",
            "cat file | grep pattern | sort | uniq | wc -l",
        ),
        ("redirects", "echo hello > output.txt 2>&1"),
        (
            "complex",
            "cat < input.txt | grep -v '^#' | sort > output.txt 2> errors.log",
        ),
        ("heredoc", "cat <<EOF\nhello\nworld\nEOF"),
    ];

    let mut group = c.benchmark_group("lexer_pipelines");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// Control flow tokenization
fn bench_control_flow(c: &mut Criterion) {
    let inputs = [
        ("if_simple", "if [ $x -eq 1 ]; then echo one; fi"),
        (
            "if_else",
            "if [ $x -eq 1 ]; then echo one; elif [ $x -eq 2 ]; then echo two; else echo other; fi",
        ),
        ("for_loop", "for i in 1 2 3 4 5; do echo $i; done"),
        (
            "while_loop",
            "while [ $x -lt 10 ]; do echo $x; x=$((x+1)); done",
        ),
        (
            "case_stmt",
            "case $x in 1) echo one;; 2|3) echo two;; *) echo other;; esac",
        ),
        ("function", "myfunc() { echo hello $1; return 0; }"),
    ];

    let mut group = c.benchmark_group("lexer_control_flow");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// franken-specific syntax tokenization
fn bench_jsh_syntax(c: &mut Criterion) {
    let inputs = [
        (
            "match_simple",
            "match $x { 1 => echo one; * => echo other }",
        ),
        (
            "match_complex",
            "match $x { 1 => echo one; 2 | 3 => echo two; 4..10 => echo range; /^hello/ => echo regex; * => echo default }",
        ),
        ("loop", "loop { echo iteration; break }"),
        ("let_const", "let name = \"world\"; const PI = 3.14"),
        (
            "try_catch",
            "try { risky_cmd } catch e { echo $e } finally { cleanup }",
        ),
        ("fn_def", "fn greet { echo \"Hello, $1!\" }"),
    ];

    let mut group = c.benchmark_group("lexer_jsh_syntax");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

/// Realistic script tokenization
fn bench_realistic_scripts(c: &mut Criterion) {
    let small_script = r#"
#!/usr/bin/env franken
name="World"
echo "Hello, $name!"
"#;

    let medium_script = r#"
#!/usr/bin/env franken
# A medium-sized script

name="${1:-World}"
count="${2:-3}"

greet() {
    echo "Hello, $1!"
}

for i in $(seq 1 $count); do
    greet "$name"
done

if [ -f "config.txt" ]; then
    source config.txt
else
    echo "No config found"
fi
"#;

    let large_script = r#"
#!/usr/bin/env franken
# A larger, more complex script

set -e

# Configuration
readonly VERSION="1.0.0"
readonly CONFIG_FILE="${HOME}/.myapp/config"

# Logging functions
log_info() {
    echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S') $*"
}

log_error() {
    echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') $*" >&2
}

# Parse command line arguments
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                echo "Version: $VERSION"
                exit 0
                ;;
            -c|--config)
                shift
                CONFIG_FILE="$1"
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
        shift
    done
}

# Main function
main() {
    log_info "Starting application..."

    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
    fi

    for item in "$@"; do
        process_item "$item"
    done

    log_info "Done."
}

parse_args "$@"
main "$@"
"#;

    let inputs = [
        ("small", small_script),
        ("medium", medium_script),
        ("large", large_script),
    ];

    let mut group = c.benchmark_group("lexer_scripts");
    group.sample_size(50);

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), input, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
                lexer.tokenize().unwrap()
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_command,
    bench_variable_expansion,
    bench_strings,
    bench_pipelines,
    bench_control_flow,
    bench_jsh_syntax,
    bench_realistic_scripts,
);

criterion_main!(benches);
