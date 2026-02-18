//! Glob expansion benchmarks for Franken Shell
//!
//! Benchmarks glob pattern matching and expansion performance.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use franken_shell::interpreter::Interpreter;
use franken_shell::lexer::Lexer;
use franken_shell::parser::Parser;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::Write;
use tempfile::TempDir;

/// Create a temporary directory with test files
fn setup_test_files(count: usize) -> TempDir {
    let dir = TempDir::new().unwrap();

    // Create various file types
    for i in 0..count {
        let filename = format!("file_{:04}.txt", i);
        File::create(dir.path().join(&filename)).unwrap();

        if i % 3 == 0 {
            let logfile = format!("log_{:04}.log", i);
            File::create(dir.path().join(&logfile)).unwrap();
        }

        if i % 5 == 0 {
            let datafile = format!("data_{:04}.dat", i);
            File::create(dir.path().join(&datafile)).unwrap();
        }
    }

    // Create some subdirectories
    for i in 0..count / 10 {
        let subdir = dir.path().join(format!("subdir_{}", i));
        fs::create_dir(&subdir).unwrap();
        for j in 0..5 {
            File::create(subdir.join(format!("nested_{}.txt", j))).unwrap();
        }
    }

    dir
}

/// Helper to execute input
fn execute_input(input: &str) -> franken_shell::interpreter::ExitStatus {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.execute(&program).unwrap()
}

/// Benchmark simple glob patterns
fn bench_simple_globs(c: &mut Criterion) {
    let dir = setup_test_files(100);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_simple");

    // Single wildcard patterns
    let patterns = [
        ("star_txt", format!("echo {}/*.txt", path)),
        ("star_log", format!("echo {}/*.log", path)),
        ("star_any", format!("echo {}/*", path)),
        ("question_mark", format!("echo {}/file_000?.txt", path)),
    ];

    for (name, cmd) in patterns.iter() {
        group.bench_with_input(BenchmarkId::new("expand", name), cmd, |b, cmd| {
            b.iter(|| execute_input(black_box(cmd)))
        });
    }

    group.finish();
}

/// Benchmark complex glob patterns
fn bench_complex_globs(c: &mut Criterion) {
    let dir = setup_test_files(100);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_complex");

    // Complex patterns with character classes
    let patterns = [
        ("bracket_range", format!("echo {}/file_00[0-5]*.txt", path)),
        ("multiple_star", format!("echo {}/*_*_*.txt", path)),
        ("recursive_star", format!("echo {}/**/*.txt", path)),
        ("negation", format!("echo {}/[!l]*.txt", path)),
    ];

    for (name, cmd) in patterns.iter() {
        group.bench_with_input(BenchmarkId::new("expand", name), cmd, |b, cmd| {
            b.iter(|| execute_input(black_box(cmd)))
        });
    }

    group.finish();
}

/// Benchmark glob with varying file counts
fn bench_glob_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("glob_scaling");

    for count in [10, 50, 100, 500] {
        let dir = setup_test_files(count);
        let path = dir.path().to_string_lossy();
        let cmd = format!("echo {}/*.txt", path);

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("files", count), &cmd, |b, cmd| {
            b.iter(|| execute_input(black_box(cmd)))
        });
    }

    group.finish();
}

/// Benchmark multiple glob patterns
fn bench_multiple_globs(c: &mut Criterion) {
    let dir = setup_test_files(100);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_multiple");

    let patterns = [
        ("single", format!("echo {}/*.txt", path)),
        ("double", format!("echo {}/*.txt {}/*.log", path, path)),
        (
            "triple",
            format!("echo {}/*.txt {}/*.log {}/*.dat", path, path, path),
        ),
        (
            "quad",
            format!(
                "echo {}/*.txt {}/*.log {}/*.dat {}/*",
                path, path, path, path
            ),
        ),
    ];

    for (name, cmd) in patterns.iter() {
        group.bench_with_input(BenchmarkId::new("expand", name), cmd, |b, cmd| {
            b.iter(|| execute_input(black_box(cmd)))
        });
    }

    group.finish();
}

/// Benchmark no-match glob patterns
fn bench_nomatch_globs(c: &mut Criterion) {
    let dir = setup_test_files(100);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_nomatch");

    // Patterns that won't match anything
    let patterns = [
        ("no_extension", format!("echo {}/*.xyz", path)),
        ("no_prefix", format!("echo {}/zzz*", path)),
        ("impossible", format!("echo {}/*impossible*pattern*", path)),
    ];

    for (name, cmd) in patterns.iter() {
        group.bench_with_input(BenchmarkId::new("expand", name), cmd, |b, cmd| {
            b.iter(|| execute_input(black_box(cmd)))
        });
    }

    group.finish();
}

/// Benchmark glob in loops
fn bench_glob_in_loop(c: &mut Criterion) {
    let dir = setup_test_files(50);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_loop");

    let cmd = format!("for f in {}/*.txt; do echo $f; done", path);

    group.bench_function("for_loop", |b| b.iter(|| execute_input(black_box(&cmd))));

    group.finish();
}

/// Direct glob expansion benchmark using internal API
fn bench_glob_direct(c: &mut Criterion) {
    let dir = setup_test_files(100);
    let path = dir.path().to_string_lossy();

    let mut group = c.benchmark_group("glob_direct");

    // Use glob crate directly for comparison
    let pattern = format!("{}/*.txt", path);

    group.bench_function("glob_crate", |b| {
        b.iter(|| {
            let matches: Vec<_> = glob::glob(black_box(&pattern))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            matches
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_globs,
    bench_complex_globs,
    bench_glob_scaling,
    bench_multiple_globs,
    bench_nomatch_globs,
    bench_glob_in_loop,
    bench_glob_direct,
);

criterion_main!(benches);
