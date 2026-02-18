//! Hotspot detection for JIT compilation
//!
//! Tracks loop execution counts and identifies hot loops eligible for JIT compilation.

use super::{HOT_LOOP_THRESHOLD, LoopId};
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

/// Signature of a loop for caching compiled code
///
/// Two loops with the same signature can share the same compiled code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopSignature {
    /// Type of loop (for, while, until, loop)
    pub loop_type: LoopType,
    /// Hash of the loop body AST
    pub body_hash: u64,
    /// Variables read in the loop
    pub read_vars: Vec<String>,
    /// Variables written in the loop
    pub write_vars: Vec<String>,
    /// Whether the loop calls external commands
    pub has_external_calls: bool,
    /// Whether the loop has command substitution
    pub has_command_sub: bool,
}

impl Hash for LoopSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.loop_type.hash(state);
        self.body_hash.hash(state);
        self.has_external_calls.hash(state);
        self.has_command_sub.hash(state);
    }
}

/// Type of loop construct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopType {
    For,
    While,
    Until,
    Loop,
    Select,
}

/// Hotspot tracker for loop execution
pub struct HotspotTracker {
    /// Execution counts per loop ID
    counts: FxHashMap<LoopId, u64>,
    /// Cached signatures for loops
    signatures: FxHashMap<LoopId, LoopSignature>,
    /// Compiled code cache (signature hash -> compiled function)
    #[cfg(feature = "jit")]
    compiled_cache: FxHashMap<u64, CompiledLoop>,
}

#[cfg(feature = "jit")]
/// Compiled loop function
pub struct CompiledLoop {
    /// Function pointer to the compiled code
    pub func_ptr: *const u8,
    /// Size of the compiled code in bytes
    pub code_size: usize,
}

impl Default for HotspotTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HotspotTracker {
    pub fn new() -> Self {
        Self {
            counts: FxHashMap::default(),
            signatures: FxHashMap::default(),
            #[cfg(feature = "jit")]
            compiled_cache: FxHashMap::default(),
        }
    }

    /// Record a loop iteration
    #[inline]
    pub fn record(&mut self, loop_id: LoopId) -> u64 {
        let count = self.counts.entry(loop_id).or_insert(0);
        *count += 1;
        *count
    }

    /// Check if a loop is hot
    #[inline]
    pub fn is_hot(&self, loop_id: LoopId) -> bool {
        self.counts
            .get(&loop_id)
            .map(|&c| c >= HOT_LOOP_THRESHOLD)
            .unwrap_or(false)
    }

    /// Get execution count for a loop
    #[inline]
    pub fn get_count(&self, loop_id: LoopId) -> u64 {
        self.counts.get(&loop_id).copied().unwrap_or(0)
    }

    /// Register a loop signature
    pub fn register_signature(&mut self, loop_id: LoopId, signature: LoopSignature) {
        self.signatures.insert(loop_id, signature);
    }

    /// Get a loop signature
    pub fn get_signature(&self, loop_id: LoopId) -> Option<&LoopSignature> {
        self.signatures.get(&loop_id)
    }

    /// Check if a loop can be JIT compiled
    pub fn is_jittable(&self, loop_id: LoopId) -> bool {
        if let Some(sig) = self.signatures.get(&loop_id) {
            // Simple heuristic: can JIT if no external calls or command substitution
            !sig.has_external_calls && !sig.has_command_sub
        } else {
            false
        }
    }

    /// Get all hot loops
    pub fn hot_loops(&self) -> Vec<LoopId> {
        self.counts
            .iter()
            .filter(|&(_, count)| *count >= HOT_LOOP_THRESHOLD)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        self.counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotspot_tracking() {
        let mut tracker = HotspotTracker::new();
        let loop_id = LoopId::new();

        for i in 1..=150 {
            let count = tracker.record(loop_id);
            assert_eq!(count, i);
        }

        assert!(tracker.is_hot(loop_id));
        assert_eq!(tracker.get_count(loop_id), 150);
    }

    #[test]
    fn test_loop_signature() {
        let sig = LoopSignature {
            loop_type: LoopType::For,
            body_hash: 12345,
            read_vars: vec!["i".to_string()],
            write_vars: vec!["sum".to_string()],
            has_external_calls: false,
            has_command_sub: false,
        };

        assert_eq!(sig.loop_type, LoopType::For);
        assert!(!sig.has_external_calls);
    }

    #[test]
    fn test_jittable_detection() {
        let mut tracker = HotspotTracker::new();
        let loop_id = LoopId::new();

        // Not jittable without signature
        assert!(!tracker.is_jittable(loop_id));

        // Jittable with clean signature
        tracker.register_signature(
            loop_id,
            LoopSignature {
                loop_type: LoopType::For,
                body_hash: 0,
                read_vars: vec![],
                write_vars: vec![],
                has_external_calls: false,
                has_command_sub: false,
            },
        );
        assert!(tracker.is_jittable(loop_id));

        // Not jittable with external calls
        let loop_id2 = LoopId::new();
        tracker.register_signature(
            loop_id2,
            LoopSignature {
                loop_type: LoopType::While,
                body_hash: 0,
                read_vars: vec![],
                write_vars: vec![],
                has_external_calls: true,
                has_command_sub: false,
            },
        );
        assert!(!tracker.is_jittable(loop_id2));
    }
}
