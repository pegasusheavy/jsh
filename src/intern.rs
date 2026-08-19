//! String interning for Franken Shell
//!
//! This module provides global string interning to reduce memory allocations
//! for frequently used strings like variable names, keywords, and builtin names.

use lasso::{Rodeo, Spur};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global string interner
static INTERNER: Lazy<Mutex<Rodeo>> = Lazy::new(|| Mutex::new(Rodeo::default()));

/// An interned string key
pub type InternedStr = Spur;

/// Intern a string and return its key
#[inline]
pub fn intern(s: &str) -> InternedStr {
    INTERNER.lock().unwrap().get_or_intern(s)
}

/// Intern a static string (more efficient for known strings)
#[inline]
pub fn intern_static(s: &'static str) -> InternedStr {
    INTERNER.lock().unwrap().get_or_intern_static(s)
}

/// Get the string value for an interned key
#[inline]
pub fn resolve(key: InternedStr) -> String {
    INTERNER.lock().unwrap().resolve(&key).to_string()
}

/// Check if a string is already interned
#[inline]
pub fn is_interned(s: &str) -> bool {
    INTERNER.lock().unwrap().get(s).is_some()
}

/// Pre-intern common strings for faster lookup
pub fn prewarm_interner() {
    let common_strings = [
        // Common variable names
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "PWD",
        "OLDPWD",
        "TERM",
        "LANG",
        "LC_ALL",
        "EDITOR",
        "VISUAL",
        "PAGER",
        "HOSTNAME",
        "PS1",
        "PS2",
        "IFS",
        "SHLVL",
        "RANDOM",
        "SECONDS",
        "LINENO",
        "PPID",
        "UID",
        "EUID",
        "BASH_VERSION",
        "ZSH_VERSION",
        "FSH_VERSION",
        "JSH",
        // Common keywords/builtins
        "if",
        "then",
        "else",
        "elif",
        "fi",
        "case",
        "esac",
        "for",
        "while",
        "until",
        "do",
        "done",
        "in",
        "function",
        "select",
        "time",
        "coproc",
        "echo",
        "cd",
        "pwd",
        "export",
        "unset",
        "set",
        "local",
        "readonly",
        "source",
        "exit",
        "return",
        "break",
        "continue",
        "shift",
        "test",
        "true",
        "false",
        "alias",
        "unalias",
        "type",
        "which",
        "command",
        "builtin",
        "eval",
        "exec",
        "trap",
        "wait",
        "jobs",
        "fg",
        "bg",
        "read",
        "printf",
        "declare",
        "typeset",
        "let",
        "history",
        // Common argument names
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        // Empty and common values
        "",
        " ",
        "\n",
        "\t",
        "/",
        ".",
        "..",
        "~",
    ];

    let mut interner = INTERNER.lock().unwrap();
    for s in common_strings {
        interner.get_or_intern_static(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_roundtrip() {
        let key = intern("test_string");
        assert_eq!(resolve(key), "test_string");
    }

    #[test]
    fn test_intern_dedup() {
        let key1 = intern("duplicate");
        let key2 = intern("duplicate");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_prewarm() {
        prewarm_interner();
        assert!(is_interned("PATH"));
        assert!(is_interned("HOME"));
        assert!(is_interned("echo"));
    }
}
