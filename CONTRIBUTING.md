# Contributing to jsh

Thank you for your interest in contributing to jsh! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/jsh.git
   cd jsh
   ```
3. Set up git hooks:
   ```bash
   ./scripts/setup-hooks.sh
   ```
4. Create a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```

## 📝 Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/). All commit messages are validated using `commitlint-rs`.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Code style (formatting, semicolons, etc.) |
| `refactor` | Code refactoring (no functional change) |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `build` | Build system or external dependencies |
| `ci` | CI/CD configuration |
| `chore` | Maintenance tasks |
| `revert` | Revert a previous commit |

### Scopes

| Scope | Description |
|-------|-------------|
| `lexer` | Lexer/tokenizer |
| `parser` | Parser |
| `ast` | Abstract syntax tree |
| `interpreter` | Interpreter/executor |
| `builtins` | Built-in commands |
| `shell` | REPL/shell interface |
| `theme` | Theme system |
| `error` | Error handling |
| `deps` | Dependencies |
| `docs` | Documentation site |
| `bench` | Benchmarks |
| `ci` | CI/CD |

### Examples

```bash
# Feature
feat(parser): add match expression support

# Bug fix
fix(lexer): handle escaped quotes in strings

# Documentation
docs: update README with Fish compatibility

# Refactoring
refactor(interpreter): simplify variable expansion logic

# Dependencies
chore(deps): update rustyline to 14.0

# Breaking change (add ! after type)
feat(parser)!: change AST node structure

BREAKING CHANGE: The `Command` node now uses a different field layout.
```

## 🔧 Development Setup

### Prerequisites

- Rust 2024 edition (rustc 1.85+)
- Cargo
- Git

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Project Structure

```
jsh/
├── src/
│   ├── main.rs        # Entry point
│   ├── lib.rs         # Library exports
│   ├── lexer.rs       # Tokenizer
│   ├── token.rs       # Token definitions
│   ├── parser.rs      # Parser
│   ├── ast.rs         # AST nodes
│   ├── interpreter.rs # Executor
│   ├── builtins.rs    # Built-in commands
│   ├── shell.rs       # REPL interface
│   ├── theme.rs       # Theme system
│   └── error.rs       # Error types
├── benches/           # Benchmarks
├── docs/              # Documentation site
├── examples/          # Example scripts
└── scripts/           # Development scripts
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## 📊 Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench lexer

# Generate HTML report
cargo bench -- --save-baseline main
```

## 🔍 Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add documentation for public APIs
- Include tests for new features

## 📋 Pull Request Process

1. Ensure your code builds without errors
2. Run `cargo fmt` and `cargo clippy`
3. Add/update tests as needed
4. Update documentation if applicable
5. Create a pull request with a clear description
6. Wait for CI checks to pass
7. Address review feedback

## 🐛 Reporting Issues

When reporting bugs, please include:

- jsh version (`jsh --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant error messages or logs

## 📄 License

By contributing to jsh, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license as the project.

---

Thank you for contributing! 🦀

