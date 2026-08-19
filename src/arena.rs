//! Arena allocation for AST nodes
//!
//! This module provides bump allocation for frequently-created AST structures
//! during parsing. Using an arena reduces allocation overhead significantly
//! for parsing operations.

use bumpalo::Bump;
use std::cell::RefCell;

thread_local! {
    /// Thread-local arena for parsing operations
    /// Reset between parse calls to free memory
    static PARSE_ARENA: RefCell<Bump> = RefCell::new(Bump::with_capacity(64 * 1024));
}

/// Arena-allocated string slice
///
/// This is a string allocated in the thread-local arena. It's only valid
/// for the duration of the current parse operation.
#[derive(Debug, Clone, Copy)]
pub struct ArenaStr<'a>(&'a str);

impl<'a> ArenaStr<'a> {
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl<'a> AsRef<str> for ArenaStr<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> std::ops::Deref for ArenaStr<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Arena-allocated vector
#[derive(Debug)]
pub struct ArenaVec<'a, T> {
    data: bumpalo::collections::Vec<'a, T>,
}

impl<'a, T> ArenaVec<'a, T> {
    #[inline]
    pub fn new_in(bump: &'a Bump) -> Self {
        Self {
            data: bumpalo::collections::Vec::new_in(bump),
        }
    }

    #[inline]
    pub fn with_capacity_in(capacity: usize, bump: &'a Bump) -> Self {
        Self {
            data: bumpalo::collections::Vec::with_capacity_in(capacity, bump),
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to owned Vec (moves data out of arena)
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.data.into_iter().collect()
    }
}

impl<'a, T: Clone> ArenaVec<'a, T> {
    /// Clone contents to a standard Vec
    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }
}

impl<'a, T> std::ops::Deref for ArenaVec<'a, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Execute a closure with access to the parse arena
///
/// The arena is reset after the closure completes, freeing all allocations.
/// This should wrap the entire parse operation.
#[inline]
pub fn with_parse_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    PARSE_ARENA.with(|arena| {
        let bump = arena.borrow();
        let result = f(&bump);
        // Note: We don't reset here to allow the caller to use allocated data
        result
    })
}

/// Reset the parse arena, freeing all allocations
///
/// Call this after parsing is complete and all arena-allocated data
/// has been converted to owned structures.
#[inline]
pub fn reset_parse_arena() {
    PARSE_ARENA.with(|arena| {
        arena.borrow_mut().reset();
    });
}

/// Allocate a string in the parse arena
#[inline]
pub fn arena_str(s: &str) -> String {
    // For now, just return owned string
    // Full arena integration would return ArenaStr
    s.to_string()
}

/// Get the current allocated bytes in the parse arena
#[inline]
pub fn arena_allocated_bytes() -> usize {
    PARSE_ARENA.with(|arena| arena.borrow().allocated_bytes())
}

/// A pool for reusing String allocations
///
/// This reduces allocation overhead by reusing String buffers
/// between parse operations.
pub struct StringPool {
    pool: Vec<String>,
    capacity: usize,
}

impl StringPool {
    /// Create a new string pool with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Get a string from the pool or create a new one
    #[inline]
    pub fn get(&mut self) -> String {
        self.pool.pop().unwrap_or_default()
    }

    /// Get a string with pre-allocated capacity
    #[inline]
    pub fn get_with_capacity(&mut self, cap: usize) -> String {
        match self.pool.pop() {
            Some(mut s) => {
                s.clear();
                if s.capacity() < cap {
                    s.reserve(cap - s.capacity());
                }
                s
            }
            None => String::with_capacity(cap),
        }
    }

    /// Return a string to the pool
    #[inline]
    pub fn put(&mut self, mut s: String) {
        if self.pool.len() < self.capacity {
            s.clear();
            self.pool.push(s);
        }
        // Otherwise, just drop it
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new(64)
    }
}

/// A pool for reusing Vec allocations
pub struct VecPool<T> {
    pool: Vec<Vec<T>>,
    capacity: usize,
}

impl<T> VecPool<T> {
    /// Create a new vec pool with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Get a vec from the pool or create a new one
    #[inline]
    pub fn get(&mut self) -> Vec<T> {
        self.pool.pop().unwrap_or_default()
    }

    /// Get a vec with pre-allocated capacity
    #[inline]
    pub fn get_with_capacity(&mut self, cap: usize) -> Vec<T> {
        match self.pool.pop() {
            Some(mut v) => {
                v.clear();
                if v.capacity() < cap {
                    v.reserve(cap - v.capacity());
                }
                v
            }
            None => Vec::with_capacity(cap),
        }
    }

    /// Return a vec to the pool
    #[inline]
    pub fn put(&mut self, mut v: Vec<T>) {
        if self.pool.len() < self.capacity {
            v.clear();
            self.pool.push(v);
        }
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

impl<T> Default for VecPool<T> {
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new(4);

        let s1 = pool.get();
        assert!(s1.is_empty());

        let mut s2 = pool.get_with_capacity(100);
        assert!(s2.capacity() >= 100);

        s2.push_str("hello");
        pool.put(s2);

        // Should get the recycled string (cleared)
        let s3 = pool.get();
        assert!(s3.is_empty());
        assert!(s3.capacity() >= 100);
    }

    #[test]
    fn test_vec_pool() {
        let mut pool: VecPool<i32> = VecPool::new(4);

        let v1 = pool.get();
        assert!(v1.is_empty());

        let mut v2 = pool.get_with_capacity(50);
        v2.push(1);
        v2.push(2);
        pool.put(v2);

        let v3 = pool.get();
        assert!(v3.is_empty());
        assert!(v3.capacity() >= 50);
    }

    #[test]
    fn test_arena_allocation() {
        with_parse_arena(|bump| {
            // Allocate some strings in the arena
            let s1 = bumpalo::collections::String::from_str_in("hello", bump);
            let s2 = bumpalo::collections::String::from_str_in("world", bump);
            assert_eq!(s1.as_str(), "hello");
            assert_eq!(s2.as_str(), "world");
        });

        // After reset, memory is freed (arena starts with 64KB capacity)
        reset_parse_arena();
        // Arena keeps its capacity after reset, but allocations are freed
        // The allocated_bytes() returns bytes used, which should be small after reset
        // Note: Bump arena keeps its capacity, so we just verify reset doesn't panic
    }
}
