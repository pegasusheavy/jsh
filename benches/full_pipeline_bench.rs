//! Full pipeline benchmarks for jsh
//!
//! Benchmarks the complete lex -> parse -> execute pipeline.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jsh::interpreter::Interpreter;
use jsh::lexer::Lexer;
use jsh::parser::Parser;

/// Full pipeline: lex -> parse -> execute
fn full_pipeline(input: &str) -> jsh::interpreter::ExitStatus {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.execute(&program).unwrap()
}

/// Benchmark comparing lex, parse, and full pipeline times
fn bench_pipeline_breakdown(c: &mut Criterion) {
    let script = r#"
name="World"
greet() { echo "Hello, $1!"; }

for i in 1 2 3 4 5; do
    greet "$name"
done

if [ -n "$name" ]; then
    echo "Name is set"
fi
"#;

    let mut group = c.benchmark_group("pipeline_breakdown");
    group.throughput(Throughput::Bytes(script.len() as u64));

    // Lex only
    group.bench_function("1_lex_only", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(script));
            lexer.tokenize().unwrap()
        })
    });

    // Lex + Parse
    group.bench_function("2_lex_parse", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(script));
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse_program().unwrap()
        })
    });

    // Full pipeline (lex + parse + execute)
    group.bench_function("3_full_pipeline", |b| {
        b.iter(|| full_pipeline(black_box(script)))
    });

    group.finish();
}

/// Benchmark different script sizes
fn bench_script_sizes(c: &mut Criterion) {
    let tiny = "echo hello";

    let small = r#"
name="World"
echo "Hello, $name!"
"#;

    let medium = r#"
#!/usr/bin/env jsh
name="${1:-World}"
count=5

greet() {
    echo "Hello, $1!"
}

for i in 1 2 3 4 5; do
    greet "$name"
done

case "$name" in
    World) echo "Default name" ;;
    *) echo "Custom name: $name" ;;
esac
"#;

    let large = r#"
#!/usr/bin/env jsh
set -e

VERSION="1.0.0"
DEBUG=0

log() {
    if [ "$DEBUG" = "1" ]; then
        echo "[DEBUG] $*"
    fi
}

info() {
    echo "[INFO] $*"
}

error() {
    echo "[ERROR] $*" >&2
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -d|--debug)
                DEBUG=1
                ;;
            -v|--version)
                echo "$VERSION"
                return 0
                ;;
            -h|--help)
                echo "Usage: $0 [options]"
                return 0
                ;;
            *)
                ARGS="$ARGS $1"
                ;;
        esac
        shift
    done
}

process_item() {
    local item="$1"
    log "Processing: $item"

    if [ -z "$item" ]; then
        error "Empty item"
        return 1
    fi

    info "Item: $item"
    return 0
}

main() {
    info "Starting with version $VERSION"

    local count=0
    for arg in $ARGS; do
        process_item "$arg"
        count=$((count + 1))
    done

    info "Processed $count items"
}

parse_args foo bar baz
main
"#;

    let inputs = [
        ("tiny", tiny),
        ("small", small),
        ("medium", medium),
        ("large", large),
    ];

    let mut group = c.benchmark_group("pipeline_sizes");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("full", name), input, |b, input| {
            b.iter(|| full_pipeline(black_box(input)))
        });
    }

    group.finish();
}

/// Benchmark different complexity types
fn bench_complexity(c: &mut Criterion) {
    // Linear - simple sequence
    let linear = r#"
x=1; y=2; z=3
a=4; b=5; c=6
echo $x $y $z
echo $a $b $c
"#;

    // Branching - if/elif/else
    let branching = r#"
x=5
if [ $x -eq 1 ]; then
    echo one
elif [ $x -eq 2 ]; then
    echo two
elif [ $x -eq 3 ]; then
    echo three
elif [ $x -eq 4 ]; then
    echo four
elif [ $x -eq 5 ]; then
    echo five
else
    echo other
fi
"#;

    // Looping
    let looping = r#"
for i in 1 2 3 4 5 6 7 8 9 10; do
    x=$i
done
"#;

    // Nested
    let nested = r#"
for i in 1 2 3; do
    for j in a b c; do
        if [ "$i" = "2" ]; then
            if [ "$j" = "b" ]; then
                echo "match: $i$j"
            fi
        fi
    done
done
"#;

    // Function-heavy
    let functions = r#"
f1() { echo f1; }
f2() { echo f2; }
f3() { echo f3; }
f4() { echo f4; }
f5() { echo f5; }
f1; f2; f3; f4; f5
f1; f2; f3; f4; f5
"#;

    let inputs = [
        ("linear", linear),
        ("branching", branching),
        ("looping", looping),
        ("nested", nested),
        ("functions", functions),
    ];

    let mut group = c.benchmark_group("pipeline_complexity");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("full", name), input, |b, input| {
            b.iter(|| full_pipeline(black_box(input)))
        });
    }

    group.finish();
}

/// Benchmark shell compatibility modes
fn bench_shell_styles(c: &mut Criterion) {
    // POSIX/Ash style
    let posix_style = r#"
set -e
x=hello
if [ -n "$x" ]; then
    echo "$x"
fi
for i in 1 2 3; do
    echo $i
done
"#;

    // Bash style
    let bash_style = r#"
declare -r CONST=42
export PATH="$PATH:/custom"
function greet {
    local name="$1"
    echo "Hello, $name!"
}
greet World
[[ -n "$CONST" ]] && echo "Set"
"#;

    // Fish style
    let fish_style = r#"
set name World
set -x EXPORTED value
string upper hello
string join '-' a b c
math '2 + 3 * 4'
contains apple apple banana cherry
"#;

    // jsh enhanced style
    let jsh_style = r#"
let name = "World"
let count = 5

fn greet {
    echo "Hello, $1!"
}

match $count {
    1 => echo "one"
    2 | 3 => echo "two or three"
    4..10 => echo "between 4 and 10"
    * => echo "other"
}

loop {
    count=$((count - 1))
    if [ $count -le 0 ]; then
        break
    fi
}
"#;

    let inputs = [
        ("posix_ash", posix_style),
        ("bash", bash_style),
        ("fish", fish_style),
        ("jsh_enhanced", jsh_style),
    ];

    let mut group = c.benchmark_group("pipeline_styles");

    for (name, input) in inputs.iter() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("full", name), input, |b, input| {
            b.iter(|| full_pipeline(black_box(input)))
        });
    }

    group.finish();
}

/// Benchmark repeated execution with same interpreter
fn bench_interpreter_reuse(c: &mut Criterion) {
    let script = r#"
x=$((x + 1))
echo $x
"#;

    let mut group = c.benchmark_group("interpreter_reuse");

    // New interpreter each time
    group.bench_function("new_interpreter", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(script));
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let program = parser.parse_program().unwrap();
            let mut interp = Interpreter::new();
            interp.execute(&program).unwrap()
        })
    });

    // Reuse interpreter (pre-parse)
    group.bench_function("reuse_interpreter", |b| {
        let mut lexer = Lexer::new(script);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let mut interp = Interpreter::new();

        b.iter(|| interp.execute(black_box(&program)).unwrap())
    });

    group.finish();
}

/// Benchmark with varying loop iterations
fn bench_loop_scaling(c: &mut Criterion) {
    fn make_loop(n: usize) -> String {
        format!(
            "x=0; while [ $x -lt {} ]; do x=$((x+1)); done",
            n
        )
    }

    let inputs = [
        ("loop_10", make_loop(10)),
        ("loop_50", make_loop(50)),
        ("loop_100", make_loop(100)),
        ("loop_500", make_loop(500)),
    ];

    let mut group = c.benchmark_group("loop_scaling");
    group.sample_size(20);

    for (name, input) in inputs.iter() {
        group.bench_with_input(BenchmarkId::new("execute", name), input, |b, input| {
            b.iter(|| full_pipeline(black_box(input)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pipeline_breakdown,
    bench_script_sizes,
    bench_complexity,
    bench_shell_styles,
    bench_interpreter_reuse,
    bench_loop_scaling,
);

criterion_main!(benches);

