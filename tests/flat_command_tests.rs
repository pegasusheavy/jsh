//! Tests for FlatCommand optimized representation

use franken_shell::ast::{FlatCommand, SimpleCommand, Word, Assignment};
use franken_shell::token::Span;

// =============================================================================
// FlatCommand Basic Tests
// =============================================================================

#[test]
fn test_flat_command_new() {
    let cmd = FlatCommand::new("echo");
    assert_eq!(cmd.name(), "echo");
    assert!(cmd.args().is_empty());
    assert_eq!(cmd.argc(), 0);
    assert_eq!(cmd.len(), 1);
}

#[test]
fn test_flat_command_with_args() {
    let mut cmd = FlatCommand::new("ls");
    cmd.push_arg("-la");
    cmd.push_arg("/tmp");

    assert_eq!(cmd.name(), "ls");
    assert_eq!(cmd.args(), &["-la".to_string(), "/tmp".to_string()]);
    assert_eq!(cmd.argc(), 2);
    assert_eq!(cmd.len(), 3);
}

#[test]
fn test_flat_command_from_vec() {
    let cmd: FlatCommand = vec!["grep".to_string(), "-r".to_string(), "pattern".to_string()].into();

    assert_eq!(cmd.name(), "grep");
    assert_eq!(cmd.argc(), 2);
}

#[test]
fn test_flat_command_from_slice() {
    let cmd: FlatCommand = ["cat", "file.txt"].as_slice().into();

    assert_eq!(cmd.name(), "cat");
    assert_eq!(cmd.args(), &["file.txt".to_string()]);
}

#[test]
fn test_flat_command_empty() {
    let cmd = FlatCommand::with_capacity(0);
    assert!(cmd.is_empty());
    assert_eq!(cmd.name(), "");
}

#[test]
fn test_flat_command_argv() {
    let cmd: FlatCommand = ["echo", "hello", "world"].as_slice().into();

    let argv = cmd.argv();
    assert_eq!(argv.len(), 3);
    assert_eq!(argv[0], "echo");
    assert_eq!(argv[1], "hello");
    assert_eq!(argv[2], "world");
}

// =============================================================================
// SimpleCommand Conversion Tests
// =============================================================================

#[test]
fn test_simple_command_is_flat_literal() {
    let span = Span::default();
    let cmd = SimpleCommand {
        name: Word::literal("echo", span),
        args: vec![Word::literal("hello", span), Word::literal("world", span)],
        assignments: vec![],
    };

    assert!(cmd.is_flat());
}

#[test]
fn test_simple_command_not_flat_with_variable() {
    let span = Span::default();

    use franken_shell::ast::WordPart;
    let name = Word::literal("echo", span);
    let arg_with_var = Word {
        parts: vec![WordPart::Variable("HOME".to_string())],
        span,
    };

    let cmd = SimpleCommand {
        name,
        args: vec![arg_with_var],
        assignments: vec![],
    };

    assert!(!cmd.is_flat());
}

#[test]
fn test_simple_command_not_flat_with_assignment() {
    let span = Span::default();
    let cmd = SimpleCommand {
        name: Word::literal("cmd", span),
        args: vec![],
        assignments: vec![Assignment {
            name: "FOO".to_string(),
            value: Some(Word::literal("bar", span)),
            op: franken_shell::ast::AssignmentOp::Assign,
            export: false,
            local: false,
            readonly: false,
            span,
        }],
    };

    assert!(!cmd.is_flat());
}

#[test]
fn test_simple_command_to_flat() {
    let span = Span::default();
    let cmd = SimpleCommand {
        name: Word::literal("ls", span),
        args: vec![Word::literal("-la", span), Word::literal("/tmp", span)],
        assignments: vec![],
    };

    let flat = cmd.to_flat().expect("Should convert to flat");
    assert_eq!(flat.name(), "ls");
    assert_eq!(flat.args(), &["-la".to_string(), "/tmp".to_string()]);
}

#[test]
fn test_simple_command_to_flat_fails_with_expansion() {
    let span = Span::default();

    use franken_shell::ast::WordPart;
    let cmd = SimpleCommand {
        name: Word::literal("echo", span),
        args: vec![Word {
            parts: vec![WordPart::Variable("USER".to_string())],
            span,
        }],
        assignments: vec![],
    };

    assert!(cmd.to_flat().is_none());
}

// =============================================================================
// FlatCommand -> SimpleCommand Round-trip Tests
// =============================================================================

#[test]
fn test_flat_to_simple_command() {
    let flat: FlatCommand = ["echo", "hello", "world"].as_slice().into();
    let span = Span::default();

    let simple = flat.to_simple_command(span);

    assert!(simple.is_flat());
    assert_eq!(simple.name.as_literal(), Some("echo"));
    assert_eq!(simple.args.len(), 2);
    assert_eq!(simple.args[0].as_literal(), Some("hello"));
    assert_eq!(simple.args[1].as_literal(), Some("world"));
}

#[test]
fn test_round_trip_conversion() {
    let span = Span::default();

    // Original simple command
    let original = SimpleCommand {
        name: Word::literal("grep", span),
        args: vec![
            Word::literal("-r", span),
            Word::literal("pattern", span),
            Word::literal(".", span),
        ],
        assignments: vec![],
    };

    // Convert to flat
    let flat = original.to_flat().expect("Should convert");

    // Convert back to simple
    let converted = flat.to_simple_command(span);

    // Should be equivalent
    assert_eq!(converted.name.as_literal(), original.name.as_literal());
    assert_eq!(converted.args.len(), original.args.len());
    for (orig, conv) in original.args.iter().zip(converted.args.iter()) {
        assert_eq!(orig.as_literal(), conv.as_literal());
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_flat_command_single_arg() {
    let cmd: FlatCommand = ["pwd"].as_slice().into();

    assert_eq!(cmd.name(), "pwd");
    assert!(cmd.args().is_empty());
    assert_eq!(cmd.argc(), 0);
}

#[test]
fn test_flat_command_many_args() {
    let mut cmd = FlatCommand::with_capacity(20);
    cmd.push_arg("find");

    for i in 0..15 {
        cmd.push_arg(format!("arg{}", i));
    }

    assert_eq!(cmd.name(), "find");
    assert_eq!(cmd.argc(), 15);
    assert_eq!(cmd.len(), 16);
}

#[test]
fn test_flat_command_args_with_spaces() {
    let cmd: FlatCommand = ["echo", "hello world", "foo bar"].as_slice().into();

    assert_eq!(cmd.name(), "echo");
    assert_eq!(cmd.args()[0], "hello world");
    assert_eq!(cmd.args()[1], "foo bar");
}

#[test]
fn test_flat_command_args_with_special_chars() {
    let cmd: FlatCommand = ["echo", "foo\nbar", "tab\there"].as_slice().into();

    assert_eq!(cmd.args()[0], "foo\nbar");
    assert_eq!(cmd.args()[1], "tab\there");
}

