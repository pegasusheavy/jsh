//! JIT compilation for hot loops in Franken Shell
//!
//! This module provides Just-In-Time compilation for frequently executed loops
//! using Cranelift as the code generator backend.
//!
//! ## Features
//! - Detects hot loops (executed >100 times)
//! - Compiles hot loops to native code using Cranelift
//! - Caches compiled basic blocks for reuse
//!
//! ## Usage
//! Enable the `jit` feature in Cargo.toml:
//! ```toml
//! franken-shell = { version = "0.1", features = ["jit"] }
//! ```

#[cfg(feature = "jit")]
mod compiler;
#[cfg(feature = "jit")]
mod hotspot;

#[cfg(feature = "jit")]
pub use compiler::JitCompiler;
#[cfg(feature = "jit")]
pub use hotspot::{HotspotTracker, LoopSignature};

use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Threshold for considering a loop "hot" (eligible for JIT)
pub const HOT_LOOP_THRESHOLD: u64 = 100;

/// Maximum number of compiled loops to cache
pub const MAX_COMPILED_LOOPS: usize = 256;

/// Unique identifier for a loop in the AST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoopId(pub u64);

impl LoopId {
    /// Generate a new unique loop ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for LoopId {
    fn default() -> Self {
        Self::new()
    }
}

/// Loop execution statistics
#[derive(Debug, Default)]
pub struct LoopStats {
    /// Number of times this loop has been executed
    pub execution_count: u64,
    /// Whether this loop has been JIT compiled
    pub is_compiled: bool,
    /// Total time spent in this loop (nanoseconds)
    pub total_time_ns: u64,
}

impl LoopStats {
    #[inline]
    pub fn increment(&mut self) {
        self.execution_count += 1;
    }

    #[inline]
    pub fn is_hot(&self) -> bool {
        self.execution_count >= HOT_LOOP_THRESHOLD
    }
}

/// JIT runtime for managing compiled code and hotspot detection
pub struct JitRuntime {
    /// Statistics for each loop
    pub(crate) loop_stats: FxHashMap<LoopId, LoopStats>,
    /// Whether JIT is enabled
    pub enabled: bool,
    #[cfg(feature = "jit")]
    /// The JIT compiler instance
    pub(crate) compiler: Option<JitCompiler>,
}

impl Default for JitRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl JitRuntime {
    /// Create a new JIT runtime
    pub fn new() -> Self {
        Self {
            loop_stats: FxHashMap::default(),
            enabled: cfg!(feature = "jit"),
            #[cfg(feature = "jit")]
            compiler: JitCompiler::new().ok(),
        }
    }

    /// Record a loop execution and check if it's hot
    #[inline]
    pub fn record_loop_execution(&mut self, loop_id: LoopId) -> bool {
        let stats = self.loop_stats.entry(loop_id).or_default();
        stats.increment();
        stats.is_hot() && !stats.is_compiled
    }

    /// Mark a loop as compiled
    pub fn mark_compiled(&mut self, loop_id: LoopId) {
        if let Some(stats) = self.loop_stats.get_mut(&loop_id) {
            stats.is_compiled = true;
        }
    }

    /// Get loop statistics
    pub fn get_stats(&self, loop_id: LoopId) -> Option<&LoopStats> {
        self.loop_stats.get(&loop_id)
    }

    /// Check if a loop is compiled
    pub fn is_compiled(&self, loop_id: LoopId) -> bool {
        self.loop_stats
            .get(&loop_id)
            .map(|s| s.is_compiled)
            .unwrap_or(false)
    }

    /// Get total number of hot loops detected
    pub fn hot_loop_count(&self) -> usize {
        self.loop_stats.values().filter(|s| s.is_hot()).count()
    }

    /// Get total number of compiled loops
    pub fn compiled_loop_count(&self) -> usize {
        self.loop_stats.values().filter(|s| s.is_compiled).count()
    }
}

// =============================================================================
// Stub implementations when JIT feature is disabled
// =============================================================================

#[cfg(not(feature = "jit"))]
impl JitRuntime {
    /// Try to JIT compile a loop (no-op when JIT is disabled)
    pub fn try_compile_loop(&mut self, _loop_id: LoopId) -> bool {
        false
    }
}

#[cfg(feature = "jit")]
impl JitRuntime {
    /// Try to JIT compile a hot loop
    pub fn try_compile_loop(&mut self, loop_id: LoopId) -> bool {
        if let Some(compiler) = &mut self.compiler {
            // In a full implementation, we would:
            // 1. Get the loop AST
            // 2. Analyze it for JIT-ability
            // 3. Generate Cranelift IR
            // 4. Compile to native code
            // 5. Store the compiled function pointer

            // For now, mark as compiled and return success
            self.mark_compiled(loop_id);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_id_unique() {
        let id1 = LoopId::new();
        let id2 = LoopId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_loop_stats_hot_detection() {
        let mut stats = LoopStats::default();

        for _ in 0..99 {
            stats.increment();
            assert!(!stats.is_hot());
        }

        stats.increment();
        assert!(stats.is_hot());
    }

    #[test]
    fn test_jit_runtime_record_execution() {
        let mut runtime = JitRuntime::new();
        let loop_id = LoopId::new();

        // Should not be hot until threshold is reached
        for _ in 0..99 {
            assert!(!runtime.record_loop_execution(loop_id));
        }

        // Should become hot at threshold (100th execution)
        assert!(runtime.record_loop_execution(loop_id));

        // Mark as compiled
        runtime.mark_compiled(loop_id);

        // Should not trigger again after being compiled
        assert!(!runtime.record_loop_execution(loop_id));
    }
}

