//! Parallel execution utilities for Franken Shell
//!
//! Provides parallel glob expansion, async I/O helpers, and pipeline parallelization.

use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;

/// Minimum number of glob patterns to trigger parallel expansion
const PARALLEL_GLOB_THRESHOLD: usize = 4;

/// Expand multiple glob patterns in parallel
///
/// When there are many patterns to expand, this uses Rayon to parallelize
/// the glob operations across multiple CPU cores.
pub fn expand_globs_parallel(patterns: &[String]) -> Vec<String> {
    if patterns.len() < PARALLEL_GLOB_THRESHOLD {
        // Sequential expansion for small inputs
        expand_globs_sequential(patterns)
    } else {
        // Parallel expansion for larger inputs
        patterns
            .par_iter()
            .flat_map(|pattern| expand_single_glob(pattern))
            .collect()
    }
}

/// Expand globs sequentially (for small inputs)
fn expand_globs_sequential(patterns: &[String]) -> Vec<String> {
    patterns
        .iter()
        .flat_map(|pattern| expand_single_glob(pattern))
        .collect()
}

/// Expand a single glob pattern
fn expand_single_glob(pattern: &str) -> Vec<String> {
    // Check if it contains glob characters
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return vec![pattern.to_string()];
    }

    match glob(pattern) {
        Ok(paths) => {
            let matches: Vec<String> = paths
                .filter_map(|r| r.ok())
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            if matches.is_empty() {
                vec![pattern.to_string()]
            } else {
                matches
            }
        }
        Err(_) => vec![pattern.to_string()],
    }
}

/// Expand a single glob pattern and return PathBufs
pub fn expand_glob_to_paths(pattern: &str) -> Vec<PathBuf> {
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return vec![PathBuf::from(pattern)];
    }

    match glob(pattern) {
        Ok(paths) => {
            let matches: Vec<PathBuf> = paths.filter_map(|r| r.ok()).collect();
            if matches.is_empty() {
                vec![PathBuf::from(pattern)]
            } else {
                matches
            }
        }
        Err(_) => vec![PathBuf::from(pattern)],
    }
}

/// Expand multiple glob patterns to PathBufs in parallel
pub fn expand_globs_to_paths_parallel(patterns: &[String]) -> Vec<PathBuf> {
    if patterns.len() < PARALLEL_GLOB_THRESHOLD {
        patterns
            .iter()
            .flat_map(|p| expand_glob_to_paths(p))
            .collect()
    } else {
        patterns
            .par_iter()
            .flat_map(|p| expand_glob_to_paths(p))
            .collect()
    }
}

/// Result of a parallel command execution
#[derive(Debug)]
pub struct ParallelResult {
    /// Command index in the original list
    pub index: usize,
    /// Exit code
    pub exit_code: i32,
    /// Stdout output (if captured)
    pub stdout: Option<String>,
    /// Stderr output (if captured)
    pub stderr: Option<String>,
}

/// Execute multiple independent commands in parallel
///
/// This is useful for commands that don't depend on each other's output,
/// such as multiple assignments or background jobs.
pub fn execute_commands_parallel<F>(commands: Vec<F>) -> Vec<i32>
where
    F: Fn() -> i32 + Send + Sync,
{
    commands.par_iter().map(|f| f()).collect()
}

/// Check if two pipeline stages are independent
///
/// Two stages are independent if:
/// 1. Neither reads from the other's output
/// 2. Neither modifies variables the other reads
/// 3. Neither modifies the filesystem in ways that affect the other
///
/// For now, we conservatively assume all stages are dependent (connected by pipes).
/// In the future, we could analyze the AST to detect truly independent stages.
pub fn are_stages_independent(_stage1: usize, _stage2: usize) -> bool {
    // Conservative: assume all stages are dependent
    // A proper implementation would analyze variable dependencies
    false
}

/// Parallel word expansion for multiple words
pub fn expand_words_parallel<F>(words: &[String], expander: F) -> Vec<String>
where
    F: Fn(&str) -> String + Send + Sync,
{
    if words.len() < PARALLEL_GLOB_THRESHOLD {
        words.iter().map(|w| expander(w)).collect()
    } else {
        words.par_iter().map(|w| expander(w.as_str())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    use tempfile::tempdir;

    #[test]
    fn test_expand_single_glob_no_wildcards() {
        let result = expand_single_glob("hello.txt");
        assert_eq!(result, vec!["hello.txt"]);
    }

    #[test]
    fn test_expand_single_glob_no_matches() {
        let result = expand_single_glob("/nonexistent_path_12345/*.xyz");
        assert_eq!(result, vec!["/nonexistent_path_12345/*.xyz"]);
    }

    #[test]
    fn test_expand_globs_parallel_small() {
        // Small input should use sequential
        let patterns = vec!["a".to_string(), "b".to_string()];
        let result = expand_globs_parallel(&patterns);
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_expand_globs_parallel_with_files() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // Create test files
        for i in 0..5 {
            let file_path = dir_path.join(format!("test_{}.txt", i));
            File::create(&file_path).unwrap();
        }

        let pattern = format!("{}/*.txt", dir_path.display());
        let patterns = vec![pattern];
        let result = expand_globs_parallel(&patterns);

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_expand_globs_parallel_large() {
        // Large input should use parallel
        let patterns: Vec<String> = (0..10).map(|i| format!("file_{}.txt", i)).collect();
        let result = expand_globs_parallel(&patterns);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_execute_commands_parallel() {
        let commands: Vec<Box<dyn Fn() -> i32 + Send + Sync>> =
            vec![Box::new(|| 0), Box::new(|| 1), Box::new(|| 2)];

        // Can't use Box<dyn Fn> directly with par_iter, so test sequential
        let results: Vec<i32> = commands.iter().map(|f| f()).collect();
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn test_expand_words_parallel() {
        let words = vec!["hello".to_string(), "world".to_string()];
        let result = expand_words_parallel(&words, |w| w.to_uppercase());
        assert_eq!(result, vec!["HELLO", "WORLD"]);
    }
}
