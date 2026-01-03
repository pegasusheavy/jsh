//! Arena module tests

use franken_shell::arena::{
    reset_parse_arena, with_parse_arena, StringPool, VecPool,
};

// =============================================================================
// StringPool Tests
// =============================================================================

#[test]
fn test_string_pool_get() {
    let mut pool = StringPool::new(4);
    let s = pool.get();
    assert!(s.is_empty());
}

#[test]
fn test_string_pool_get_with_capacity() {
    let mut pool = StringPool::new(4);
    let s = pool.get_with_capacity(100);
    assert!(s.capacity() >= 100);
}

#[test]
fn test_string_pool_recycle() {
    let mut pool = StringPool::new(4);

    // Get a string with capacity
    let mut s = pool.get_with_capacity(100);
    s.push_str("hello world");

    // Return it to the pool
    pool.put(s);

    // Get it back - should be cleared but retain capacity
    let recycled = pool.get();
    assert!(recycled.is_empty());
    assert!(recycled.capacity() >= 100);
}

#[test]
fn test_string_pool_capacity_limit() {
    let mut pool = StringPool::new(2);

    // Add more than capacity
    pool.put(String::from("a"));
    pool.put(String::from("b"));
    pool.put(String::from("c")); // Should be dropped

    // Should only get 2 back
    let _ = pool.get();
    let _ = pool.get();
    let s = pool.get();
    assert!(s.is_empty()); // Fresh string, not from pool
}

// =============================================================================
// VecPool Tests
// =============================================================================

#[test]
fn test_vec_pool_get() {
    let mut pool: VecPool<i32> = VecPool::new(4);
    let v = pool.get();
    assert!(v.is_empty());
}

#[test]
fn test_vec_pool_get_with_capacity() {
    let mut pool: VecPool<i32> = VecPool::new(4);
    let v = pool.get_with_capacity(50);
    assert!(v.capacity() >= 50);
}

#[test]
fn test_vec_pool_recycle() {
    let mut pool: VecPool<i32> = VecPool::new(4);

    let mut v = pool.get_with_capacity(50);
    v.push(1);
    v.push(2);
    v.push(3);

    pool.put(v);

    let recycled = pool.get();
    assert!(recycled.is_empty());
    assert!(recycled.capacity() >= 50);
}

#[test]
fn test_vec_pool_capacity_limit() {
    let mut pool: VecPool<i32> = VecPool::new(2);

    pool.put(vec![1, 2, 3]);
    pool.put(vec![4, 5, 6]);
    pool.put(vec![7, 8, 9]); // Should be dropped

    let _ = pool.get();
    let _ = pool.get();
    let v = pool.get();
    assert!(v.is_empty());
}

// =============================================================================
// Parse Arena Tests
// =============================================================================

#[test]
fn test_with_parse_arena() {
    with_parse_arena(|bump| {
        // Allocate something in the arena
        let s = bumpalo::collections::String::from_str_in("test", bump);
        assert_eq!(s.as_str(), "test");
    });
}

#[test]
fn test_reset_parse_arena() {
    with_parse_arena(|bump| {
        // Allocate some data
        for i in 0..100 {
            let _ = bumpalo::collections::String::from_str_in(&format!("string_{}", i), bump);
        }
    });

    // Reset should not panic
    reset_parse_arena();
}

#[test]
fn test_arena_multiple_allocations() {
    with_parse_arena(|bump| {
        let mut strings = Vec::new();
        for i in 0..1000 {
            strings.push(bumpalo::collections::String::from_str_in(&format!("item_{}", i), bump));
        }

        // Verify all strings are valid
        for (i, s) in strings.iter().enumerate() {
            assert_eq!(s.as_str(), format!("item_{}", i));
        }
    });
}

#[test]
fn test_arena_vec_allocation() {
    with_parse_arena(|bump| {
        let mut v = bumpalo::collections::Vec::new_in(bump);
        for i in 0..100 {
            v.push(i);
        }

        assert_eq!(v.len(), 100);
        assert_eq!(v[0], 0);
        assert_eq!(v[99], 99);
    });
}

