//! Command line argument parsing library for jsh builtins and scripts
//!
//! This module provides a lightweight, ergonomic argument parser designed
//! specifically for shell builtins and jsh scripts.
//!
//! # Example
//!
//! ```rust
//! use jsh::argparse::{ArgParser, ArgType};
//!
//! let mut parser = ArgParser::new("mycommand")
//!     .description("A sample command")
//!     .flag("-v", "--verbose", "Enable verbose output")
//!     .flag("-q", "--quiet", "Suppress output")
//!     .option("-o", "--output", "FILE", "Output file path")
//!     .option("-n", "--count", "NUM", "Number of iterations")
//!     .positional("input", "Input file", true)
//!     .positional("extra", "Extra arguments", false);
//!
//! let args = vec!["mycommand", "-v", "-o", "out.txt", "input.txt"];
//! let result = parser.parse(&args)?;
//!
//! if result.has("verbose") {
//!     println!("Verbose mode enabled");
//! }
//! if let Some(output) = result.get("output") {
//!     println!("Output file: {}", output);
//! }
//! ```

use std::collections::HashMap;

/// Argument type
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    /// Boolean flag (e.g., -v, --verbose)
    Flag,
    /// Option with value (e.g., -o FILE, --output=FILE)
    Option,
    /// Positional argument
    Positional,
}

/// Single argument definition
#[derive(Debug, Clone)]
pub struct ArgDef {
    /// Short form (e.g., "-v")
    pub short: Option<String>,
    /// Long form (e.g., "--verbose")
    pub long: Option<String>,
    /// Name for positional args or metavar for options
    pub name: String,
    /// Help text
    pub help: String,
    /// Argument type
    pub arg_type: ArgType,
    /// Whether this argument is required
    pub required: bool,
    /// Default value
    pub default: Option<String>,
    /// Whether this option can be repeated
    pub multiple: bool,
}

/// Parsed argument results
#[derive(Debug, Clone, Default)]
pub struct ParsedArgs {
    /// Flag values (true if present)
    flags: HashMap<String, bool>,
    /// Option values
    options: HashMap<String, Vec<String>>,
    /// Positional arguments
    positionals: Vec<String>,
    /// All remaining arguments after --
    rest: Vec<String>,
}

impl ParsedArgs {
    /// Check if a flag is set
    pub fn has(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    /// Get an option value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.options.get(name).and_then(|v| v.first().map(|s| s.as_str()))
    }

    /// Get all values for a repeated option
    pub fn get_all(&self, name: &str) -> Option<&Vec<String>> {
        self.options.get(name)
    }

    /// Get a positional argument by index
    pub fn positional(&self, index: usize) -> Option<&str> {
        self.positionals.get(index).map(|s| s.as_str())
    }

    /// Get all positional arguments
    pub fn positionals(&self) -> &[String] {
        &self.positionals
    }

    /// Get remaining arguments after --
    pub fn rest(&self) -> &[String] {
        &self.rest
    }

    /// Get option as integer
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|s| s.parse().ok())
    }

    /// Get option as float
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|s| s.parse().ok())
    }

    /// Get option with default
    pub fn get_or(&self, name: &str, default: &str) -> String {
        self.get(name).map(|s| s.to_string()).unwrap_or_else(|| default.to_string())
    }

    /// Get int with default
    pub fn get_int_or(&self, name: &str, default: i64) -> i64 {
        self.get_int(name).unwrap_or(default)
    }

    /// Check if any positional arguments were provided
    pub fn has_positionals(&self) -> bool {
        !self.positionals.is_empty()
    }

    /// Count of positional arguments
    pub fn positional_count(&self) -> usize {
        self.positionals.len()
    }
}

/// Parse error types
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Unknown option
    UnknownOption(String),
    /// Missing required argument
    MissingRequired(String),
    /// Missing value for option
    MissingValue(String),
    /// Invalid value
    InvalidValue(String, String),
    /// Too many positional arguments
    TooManyPositionals(usize),
    /// Help requested
    HelpRequested,
    /// Version requested
    VersionRequested,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownOption(opt) => write!(f, "unknown option: {}", opt),
            ParseError::MissingRequired(name) => write!(f, "missing required argument: {}", name),
            ParseError::MissingValue(opt) => write!(f, "option requires a value: {}", opt),
            ParseError::InvalidValue(opt, val) => write!(f, "invalid value for {}: {}", opt, val),
            ParseError::TooManyPositionals(max) => write!(f, "too many arguments (max {})", max),
            ParseError::HelpRequested => write!(f, "help requested"),
            ParseError::VersionRequested => write!(f, "version requested"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Argument parser builder
#[derive(Debug, Clone)]
pub struct ArgParser {
    /// Command name
    name: String,
    /// Command description
    description: String,
    /// Version string
    version: Option<String>,
    /// Argument definitions
    args: Vec<ArgDef>,
    /// Whether to auto-add help flag
    auto_help: bool,
    /// Whether to auto-add version flag
    auto_version: bool,
    /// Whether to allow unknown options
    allow_unknown: bool,
    /// Whether to stop parsing at first positional
    stop_early: bool,
    /// Usage examples
    examples: Vec<String>,
    /// Additional notes
    notes: Option<String>,
}

impl ArgParser {
    /// Create a new argument parser
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            version: None,
            args: Vec::new(),
            auto_help: true,
            auto_version: false,
            allow_unknown: false,
            stop_early: false,
            examples: Vec::new(),
            notes: None,
        }
    }

    /// Set command description
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set version string
    pub fn version(mut self, ver: &str) -> Self {
        self.version = Some(ver.to_string());
        self.auto_version = true;
        self
    }

    /// Add a boolean flag
    pub fn flag(mut self, short: &str, long: &str, help: &str) -> Self {
        // Derive name from long form, or short form if no long form
        let name = if !long.is_empty() {
            long.trim_start_matches('-').to_string()
        } else {
            short.trim_start_matches('-').to_string()
        };
        self.args.push(ArgDef {
            short: if short.is_empty() { None } else { Some(short.to_string()) },
            long: if long.is_empty() { None } else { Some(long.to_string()) },
            name,
            help: help.to_string(),
            arg_type: ArgType::Flag,
            required: false,
            default: None,
            multiple: false,
        });
        self
    }

    /// Add an option with a value
    pub fn option(mut self, short: &str, long: &str, metavar: &str, help: &str) -> Self {
        let name = if !long.is_empty() {
            long.trim_start_matches('-').to_string()
        } else {
            short.trim_start_matches('-').to_string()
        };
        self.args.push(ArgDef {
            short: if short.is_empty() { None } else { Some(short.to_string()) },
            long: if long.is_empty() { None } else { Some(long.to_string()) },
            name,
            help: format!("{} ({})", help, metavar),
            arg_type: ArgType::Option,
            required: false,
            default: None,
            multiple: false,
        });
        self
    }

    /// Add a required option
    pub fn required_option(mut self, short: &str, long: &str, metavar: &str, help: &str) -> Self {
        let name = if !long.is_empty() {
            long.trim_start_matches('-').to_string()
        } else {
            short.trim_start_matches('-').to_string()
        };
        self.args.push(ArgDef {
            short: if short.is_empty() { None } else { Some(short.to_string()) },
            long: if long.is_empty() { None } else { Some(long.to_string()) },
            name,
            help: format!("{} ({})", help, metavar),
            arg_type: ArgType::Option,
            required: true,
            default: None,
            multiple: false,
        });
        self
    }

    /// Add an option with a default value
    pub fn option_default(mut self, short: &str, long: &str, metavar: &str, help: &str, default: &str) -> Self {
        let name = if !long.is_empty() {
            long.trim_start_matches('-').to_string()
        } else {
            short.trim_start_matches('-').to_string()
        };
        self.args.push(ArgDef {
            short: if short.is_empty() { None } else { Some(short.to_string()) },
            long: if long.is_empty() { None } else { Some(long.to_string()) },
            name,
            help: format!("{} ({}, default: {})", help, metavar, default),
            arg_type: ArgType::Option,
            required: false,
            default: Some(default.to_string()),
            multiple: false,
        });
        self
    }

    /// Add a repeatable option (can be specified multiple times)
    pub fn multi_option(mut self, short: &str, long: &str, metavar: &str, help: &str) -> Self {
        let name = if !long.is_empty() {
            long.trim_start_matches('-').to_string()
        } else {
            short.trim_start_matches('-').to_string()
        };
        self.args.push(ArgDef {
            short: if short.is_empty() { None } else { Some(short.to_string()) },
            long: if long.is_empty() { None } else { Some(long.to_string()) },
            name,
            help: format!("{} ({}, can be repeated)", help, metavar),
            arg_type: ArgType::Option,
            required: false,
            default: None,
            multiple: true,
        });
        self
    }

    /// Add a positional argument
    pub fn positional(mut self, name: &str, help: &str, required: bool) -> Self {
        self.args.push(ArgDef {
            short: None,
            long: None,
            name: name.to_string(),
            help: help.to_string(),
            arg_type: ArgType::Positional,
            required,
            default: None,
            multiple: false,
        });
        self
    }

    /// Add a variadic positional argument (collects all remaining)
    pub fn positional_variadic(mut self, name: &str, help: &str) -> Self {
        self.args.push(ArgDef {
            short: None,
            long: None,
            name: name.to_string(),
            help: format!("{} (can be multiple)", help),
            arg_type: ArgType::Positional,
            required: false,
            default: None,
            multiple: true,
        });
        self
    }

    /// Disable auto-help flag
    pub fn no_help(mut self) -> Self {
        self.auto_help = false;
        self
    }

    /// Allow unknown options (don't error)
    pub fn allow_unknown(mut self) -> Self {
        self.allow_unknown = true;
        self
    }

    /// Stop parsing at first positional argument
    pub fn stop_early(mut self) -> Self {
        self.stop_early = true;
        self
    }

    /// Add usage example
    pub fn example(mut self, example: &str) -> Self {
        self.examples.push(example.to_string());
        self
    }

    /// Add notes section
    pub fn notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }

    /// Generate help text
    pub fn help(&self) -> String {
        let mut help = String::new();

        // Usage line
        help.push_str(&format!("{}", self.name));
        if !self.description.is_empty() {
            help.push_str(&format!(" - {}", self.description));
        }
        help.push('\n');
        help.push('\n');

        // Usage
        help.push_str("Usage: ");
        help.push_str(&self.name);
        help.push_str(" [OPTIONS]");

        // Add positional args to usage
        for arg in &self.args {
            if arg.arg_type == ArgType::Positional {
                if arg.required {
                    help.push_str(&format!(" <{}>", arg.name));
                } else if arg.multiple {
                    help.push_str(&format!(" [{}...]", arg.name));
                } else {
                    help.push_str(&format!(" [{}]", arg.name));
                }
            }
        }
        help.push('\n');
        help.push('\n');

        // Options
        let flags: Vec<_> = self.args.iter()
            .filter(|a| a.arg_type == ArgType::Flag)
            .collect();
        let options: Vec<_> = self.args.iter()
            .filter(|a| a.arg_type == ArgType::Option)
            .collect();
        let positionals: Vec<_> = self.args.iter()
            .filter(|a| a.arg_type == ArgType::Positional)
            .collect();

        if !flags.is_empty() || self.auto_help || self.auto_version {
            help.push_str("Flags:\n");

            if self.auto_help {
                help.push_str("  -h, --help        Show this help message\n");
            }
            if self.auto_version {
                help.push_str("  -V, --version     Show version information\n");
            }

            for arg in flags {
                let short = arg.short.as_ref().map(|s| format!("{}, ", s)).unwrap_or_default();
                let long = arg.long.as_ref().map(|s| s.as_str()).unwrap_or("");
                help.push_str(&format!("  {}{:<12}  {}\n", short, long, arg.help));
            }
            help.push('\n');
        }

        if !options.is_empty() {
            help.push_str("Options:\n");
            for arg in options {
                let short = arg.short.as_ref().map(|s| format!("{}, ", s)).unwrap_or_default();
                let long = arg.long.as_ref().map(|s| s.as_str()).unwrap_or("");
                let required = if arg.required { " (required)" } else { "" };
                help.push_str(&format!("  {}{:<12}  {}{}\n", short, long, arg.help, required));
            }
            help.push('\n');
        }

        if !positionals.is_empty() {
            help.push_str("Arguments:\n");
            for arg in positionals {
                let required = if arg.required { " (required)" } else { "" };
                help.push_str(&format!("  {:<14}  {}{}\n", arg.name, arg.help, required));
            }
            help.push('\n');
        }

        // Examples
        if !self.examples.is_empty() {
            help.push_str("Examples:\n");
            for example in &self.examples {
                help.push_str(&format!("  {}\n", example));
            }
            help.push('\n');
        }

        // Notes
        if let Some(notes) = &self.notes {
            help.push_str(&format!("{}\n", notes));
        }

        help
    }

    /// Generate short usage text
    pub fn usage(&self) -> String {
        let mut usage = format!("Usage: {} [OPTIONS]", self.name);

        for arg in &self.args {
            if arg.arg_type == ArgType::Positional {
                if arg.required {
                    usage.push_str(&format!(" <{}>", arg.name));
                } else {
                    usage.push_str(&format!(" [{}]", arg.name));
                }
            }
        }

        usage
    }

    /// Find argument definition by short or long form
    fn find_arg(&self, key: &str) -> Option<&ArgDef> {
        self.args.iter().find(|a| {
            a.short.as_ref().map(|s| s == key).unwrap_or(false) ||
            a.long.as_ref().map(|s| s == key).unwrap_or(false)
        })
    }

    /// Parse arguments
    pub fn parse(&self, args: &[impl AsRef<str>]) -> Result<ParsedArgs, ParseError> {
        let mut result = ParsedArgs::default();
        let positional_defs: Vec<_> = self.args.iter()
            .filter(|a| a.arg_type == ArgType::Positional)
            .collect();

        // Apply defaults
        for arg in &self.args {
            if let Some(default) = &arg.default {
                result.options.insert(arg.name.clone(), vec![default.clone()]);
            }
        }

        let args: Vec<String> = args.iter().map(|a| a.as_ref().to_string()).collect();
        let mut i = 0;

        // Skip the command name if present
        if !args.is_empty() && !args[0].starts_with('-') && args[0] == self.name {
            i = 1;
        }

        while i < args.len() {
            let arg = &args[i];

            // Handle -- (rest goes to positionals)
            if arg == "--" {
                result.rest = args[i + 1..].to_vec();
                break;
            }

            // Check for help
            if self.auto_help && (arg == "-h" || arg == "--help") {
                return Err(ParseError::HelpRequested);
            }

            // Check for version
            if self.auto_version && (arg == "-V" || arg == "--version") {
                return Err(ParseError::VersionRequested);
            }

            // Long option with =
            if arg.starts_with("--") && arg.contains('=') {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                let key = parts[0];
                let value = parts[1];

                if let Some(def) = self.find_arg(key) {
                    if def.arg_type == ArgType::Flag {
                        // Flags don't take values
                        result.flags.insert(def.name.clone(), value == "true" || value == "1");
                    } else {
                        if def.multiple {
                            result.options.entry(def.name.clone())
                                .or_default()
                                .push(value.to_string());
                        } else {
                            result.options.insert(def.name.clone(), vec![value.to_string()]);
                        }
                    }
                } else if !self.allow_unknown {
                    return Err(ParseError::UnknownOption(key.to_string()));
                }
                i += 1;
                continue;
            }

            // Long option
            if arg.starts_with("--") {
                if let Some(def) = self.find_arg(arg) {
                    if def.arg_type == ArgType::Flag {
                        result.flags.insert(def.name.clone(), true);
                    } else {
                        // Option needs value
                        i += 1;
                        if i >= args.len() {
                            return Err(ParseError::MissingValue(arg.clone()));
                        }
                        let value = &args[i];
                        if def.multiple {
                            result.options.entry(def.name.clone())
                                .or_default()
                                .push(value.clone());
                        } else {
                            result.options.insert(def.name.clone(), vec![value.clone()]);
                        }
                    }
                } else if !self.allow_unknown {
                    return Err(ParseError::UnknownOption(arg.clone()));
                }
                i += 1;
                continue;
            }

            // Short option(s)
            if arg.starts_with('-') && arg.len() > 1 {
                let chars: Vec<char> = arg[1..].chars().collect();
                let mut j = 0;

                while j < chars.len() {
                    let short = format!("-{}", chars[j]);

                    if let Some(def) = self.find_arg(&short) {
                        if def.arg_type == ArgType::Flag {
                            result.flags.insert(def.name.clone(), true);
                            j += 1;
                        } else {
                            // Option needs value
                            // Check if value is attached (e.g., -n10)
                            if j + 1 < chars.len() {
                                let value: String = chars[j + 1..].iter().collect();
                                if def.multiple {
                                    result.options.entry(def.name.clone())
                                        .or_default()
                                        .push(value);
                                } else {
                                    result.options.insert(def.name.clone(), vec![value]);
                                }
                                break;
                            } else {
                                // Value is next argument
                                i += 1;
                                if i >= args.len() {
                                    return Err(ParseError::MissingValue(short));
                                }
                                let value = &args[i];
                                if def.multiple {
                                    result.options.entry(def.name.clone())
                                        .or_default()
                                        .push(value.clone());
                                } else {
                                    result.options.insert(def.name.clone(), vec![value.clone()]);
                                }
                                break;
                            }
                        }
                    } else if !self.allow_unknown {
                        return Err(ParseError::UnknownOption(short));
                    } else {
                        j += 1;
                    }
                }
                i += 1;
                continue;
            }

            // Positional argument
            if self.stop_early {
                // Everything from here is positional
                result.positionals.extend(args[i..].iter().cloned());
                break;
            }

            result.positionals.push(arg.clone());
            i += 1;
        }

        // Check required arguments
        for arg in &self.args {
            if arg.required {
                match arg.arg_type {
                    ArgType::Option => {
                        if !result.options.contains_key(&arg.name) {
                            return Err(ParseError::MissingRequired(arg.name.clone()));
                        }
                    }
                    ArgType::Positional => {
                        let idx = positional_defs.iter()
                            .position(|a| a.name == arg.name)
                            .unwrap_or(0);
                        if idx >= result.positionals.len() {
                            return Err(ParseError::MissingRequired(arg.name.clone()));
                        }
                    }
                    ArgType::Flag => {}
                }
            }
        }

        Ok(result)
    }

    /// Parse and handle help/version automatically
    pub fn parse_or_exit(&self, args: &[impl AsRef<str>]) -> ParsedArgs {
        match self.parse(args) {
            Ok(result) => result,
            Err(ParseError::HelpRequested) => {
                print!("{}", self.help());
                std::process::exit(0);
            }
            Err(ParseError::VersionRequested) => {
                if let Some(ver) = &self.version {
                    println!("{} {}", self.name, ver);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("{}: {}", self.name, e);
                eprintln!("{}", self.usage());
                eprintln!("Try '{} --help' for more information.", self.name);
                std::process::exit(1);
            }
        }
    }
}

/// Quick parser for simple cases - parses flags and collects positionals
pub fn quick_parse(args: &[String]) -> (HashMap<String, bool>, HashMap<String, String>, Vec<String>) {
    let mut flags = HashMap::new();
    let mut options = HashMap::new();
    let mut positionals = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            positionals.extend(args[i + 1..].iter().cloned());
            break;
        }

        if arg.starts_with("--") && arg.contains('=') {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            let key = parts[0][2..].to_string();
            let value = parts[1].to_string();
            options.insert(key, value);
        } else if arg.starts_with("--") {
            let key = arg[2..].to_string();
            // Check if next arg is a value
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                options.insert(key, args[i + 1].clone());
                i += 1;
            } else {
                flags.insert(key, true);
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Handle short flags like -abc
            for c in arg[1..].chars() {
                flags.insert(c.to_string(), true);
            }
        } else {
            positionals.push(arg.clone());
        }

        i += 1;
    }

    (flags, options, positionals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flags() {
        let parser = ArgParser::new("test")
            .flag("-v", "--verbose", "Verbose output")
            .flag("-q", "--quiet", "Quiet mode");

        let result = parser.parse(&["test", "-v", "--quiet"]).unwrap();
        assert!(result.has("verbose"));
        assert!(result.has("quiet"));
    }

    #[test]
    fn test_options() {
        let parser = ArgParser::new("test")
            .option("-o", "--output", "FILE", "Output file")
            .option("-n", "--count", "NUM", "Count");

        let result = parser.parse(&["test", "-o", "out.txt", "--count=10"]).unwrap();
        assert_eq!(result.get("output"), Some("out.txt"));
        assert_eq!(result.get_int("count"), Some(10));
    }

    #[test]
    fn test_positional() {
        let parser = ArgParser::new("test")
            .flag("-v", "--verbose", "Verbose")
            .positional("input", "Input file", true)
            .positional("output", "Output file", false);

        let result = parser.parse(&["test", "-v", "in.txt", "out.txt"]).unwrap();
        assert!(result.has("verbose"));
        assert_eq!(result.positional(0), Some("in.txt"));
        assert_eq!(result.positional(1), Some("out.txt"));
    }

    #[test]
    fn test_combined_short_flags() {
        let parser = ArgParser::new("test")
            .flag("-a", "", "A flag")
            .flag("-b", "", "B flag")
            .flag("-c", "", "C flag");

        let result = parser.parse(&["test", "-abc"]).unwrap();
        assert!(result.has("a"));
        assert!(result.has("b"));
        assert!(result.has("c"));
    }

    #[test]
    fn test_default_value() {
        let parser = ArgParser::new("test")
            .option_default("-n", "--count", "NUM", "Count", "5");

        let result = parser.parse(&["test"]).unwrap();
        assert_eq!(result.get("count"), Some("5"));

        let result = parser.parse(&["test", "-n", "10"]).unwrap();
        assert_eq!(result.get("count"), Some("10"));
    }

    #[test]
    fn test_multi_option() {
        let parser = ArgParser::new("test")
            .multi_option("-f", "--file", "FILE", "Input files");

        let result = parser.parse(&["test", "-f", "a.txt", "-f", "b.txt", "--file", "c.txt"]).unwrap();
        let files = result.get_all("file").unwrap();
        assert_eq!(files, &["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_required_missing() {
        let parser = ArgParser::new("test")
            .required_option("-o", "--output", "FILE", "Output file");

        let result = parser.parse(&["test"]);
        assert!(matches!(result, Err(ParseError::MissingRequired(_))));
    }

    #[test]
    fn test_unknown_option() {
        let parser = ArgParser::new("test")
            .flag("-v", "--verbose", "Verbose");

        let result = parser.parse(&["test", "--unknown"]);
        assert!(matches!(result, Err(ParseError::UnknownOption(_))));
    }

    #[test]
    fn test_rest_args() {
        let parser = ArgParser::new("test")
            .flag("-v", "--verbose", "Verbose")
            .positional("cmd", "Command", true);

        let result = parser.parse(&["test", "-v", "run", "--", "-a", "-b", "extra"]).unwrap();
        assert!(result.has("verbose"));
        assert_eq!(result.positional(0), Some("run"));
        assert_eq!(result.rest(), &["-a", "-b", "extra"]);
    }

    #[test]
    fn test_help_generation() {
        let parser = ArgParser::new("myapp")
            .description("A sample application")
            .version("1.0.0")
            .flag("-v", "--verbose", "Enable verbose output")
            .option("-o", "--output", "FILE", "Output file path")
            .positional("input", "Input file", true)
            .example("myapp -v input.txt")
            .example("myapp -o out.txt input.txt");

        let help = parser.help();
        assert!(help.contains("myapp"));
        assert!(help.contains("verbose"));
        assert!(help.contains("output"));
        assert!(help.contains("input"));
    }
}

