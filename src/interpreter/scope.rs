//! Variable scope stack with efficient allocation
//!
//! Provides proper scoping for function-local variables with fast lookup.
//! Uses a stack of scopes where each scope is a FxHashMap for fast hashing.

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

/// A single variable scope
#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// Variables in this scope
    vars: FxHashMap<String, String>,
    /// Variables declared as local in this scope (won't propagate to parent)
    locals: FxHashMap<String, ()>,
}

impl Scope {
    /// Create a new empty scope
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new scope with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vars: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            locals: FxHashMap::default(),
        }
    }

    /// Get a variable from this scope
    #[inline]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(|s| s.as_str())
    }

    /// Set a variable in this scope
    #[inline]
    pub fn set(&mut self, name: &str, value: String) {
        self.vars.insert(name.to_string(), value);
    }

    /// Remove a variable from this scope
    #[inline]
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.locals.remove(name);
        self.vars.remove(name)
    }

    /// Mark a variable as local to this scope
    #[inline]
    pub fn mark_local(&mut self, name: &str) {
        self.locals.insert(name.to_string(), ());
    }

    /// Check if a variable is local to this scope
    #[inline]
    pub fn is_local(&self, name: &str) -> bool {
        self.locals.contains_key(name)
    }

    /// Check if scope contains a variable
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// Get all variable names in this scope
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.vars.keys()
    }

    /// Get all variables in this scope
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.vars.iter()
    }

    /// Clear the scope
    #[inline]
    pub fn clear(&mut self) {
        self.vars.clear();
        self.locals.clear();
    }

    /// Number of variables in this scope
    #[inline]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Check if scope is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }
}

/// Stack of variable scopes for proper scoping semantics
///
/// The stack grows down - index 0 is the global scope, higher indices are
/// nested scopes (function calls). Variable lookup starts from the top
/// (current scope) and walks down to the global scope.
#[derive(Debug)]
pub struct ScopeStack {
    /// Stack of scopes (SmallVec for common case of shallow nesting)
    /// Most scripts have 1-4 levels of nesting
    scopes: SmallVec<[Scope; 4]>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeStack {
    /// Create a new scope stack with a global scope
    pub fn new() -> Self {
        let mut scopes = SmallVec::new();
        // Start with a global scope with reasonable capacity
        scopes.push(Scope::with_capacity(32));
        Self { scopes }
    }

    /// Push a new scope onto the stack (entering a function)
    #[inline]
    pub fn push_scope(&mut self) {
        // Most function scopes have 4-8 local variables
        self.scopes.push(Scope::with_capacity(8));
    }

    /// Push a scope with specific capacity
    #[inline]
    pub fn push_scope_with_capacity(&mut self, capacity: usize) {
        self.scopes.push(Scope::with_capacity(capacity));
    }

    /// Pop the current scope (exiting a function)
    /// Returns the popped scope, or None if only global scope remains
    #[inline]
    pub fn pop_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None
        }
    }

    /// Get a variable, searching from current scope to global
    #[inline]
    pub fn get(&self, name: &str) -> Option<&str> {
        // Search from top (current) to bottom (global)
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Set a variable in the appropriate scope
    ///
    /// If the variable is declared local in a scope, set it there.
    /// Otherwise, set it in the current scope.
    #[inline]
    pub fn set(&mut self, name: &str, value: String) {
        // Check if variable is local in any scope (search from top)
        for scope in self.scopes.iter_mut().rev() {
            if scope.is_local(name) || scope.contains(name) {
                scope.set(name, value);
                return;
            }
        }

        // If not found anywhere, set in current scope
        if let Some(current) = self.scopes.last_mut() {
            current.set(name, value);
        }
    }

    /// Set a variable in the current scope only
    #[inline]
    pub fn set_local(&mut self, name: &str, value: String) {
        if let Some(current) = self.scopes.last_mut() {
            current.mark_local(name);
            current.set(name, value);
        }
    }

    /// Set a variable in the global scope
    #[inline]
    pub fn set_global(&mut self, name: &str, value: String) {
        if let Some(global) = self.scopes.first_mut() {
            global.set(name, value);
        }
    }

    /// Remove a variable from the current scope
    #[inline]
    pub fn remove(&mut self, name: &str) -> Option<String> {
        // Remove from the first scope that contains it
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains(name) {
                return scope.remove(name);
            }
        }
        None
    }

    /// Check if a variable exists in any scope
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.scopes.iter().any(|scope| scope.contains(name))
    }

    /// Get the current scope depth (0 = global only)
    #[inline]
    pub fn depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }

    /// Check if we're in the global scope
    #[inline]
    pub fn is_global(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Get the current (topmost) scope
    #[inline]
    pub fn current(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    /// Get the current (topmost) scope mutably
    #[inline]
    pub fn current_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    /// Get the global scope
    #[inline]
    pub fn global(&self) -> Option<&Scope> {
        self.scopes.first()
    }

    /// Get the global scope mutably
    #[inline]
    pub fn global_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.first_mut()
    }

    /// Get all variable names visible from current scope
    pub fn visible_vars(&self) -> Vec<&String> {
        let mut seen = FxHashMap::default();
        let mut result = Vec::new();

        // Collect from top to bottom, tracking what we've seen
        for scope in self.scopes.iter().rev() {
            for name in scope.keys() {
                if !seen.contains_key(name.as_str()) {
                    seen.insert(name.as_str(), ());
                    result.push(name);
                }
            }
        }

        result
    }

    /// Clear all scopes except global
    pub fn clear_to_global(&mut self) {
        while self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Merge current scope into parent (for export-like behavior)
    pub fn merge_into_parent(&mut self) {
        if self.scopes.len() > 1 {
            if let Some(current) = self.scopes.pop() {
                if let Some(parent) = self.scopes.last_mut() {
                    for (name, value) in current.iter() {
                        parent.set(name, value.clone());
                    }
                }
            }
        }
    }
}

/// RAII guard for automatically managing scope lifetime
pub struct ScopeGuard<'a> {
    stack: &'a mut ScopeStack,
}

impl<'a> ScopeGuard<'a> {
    /// Create a new scope guard, pushing a new scope
    #[inline]
    pub fn new(stack: &'a mut ScopeStack) -> Self {
        stack.push_scope();
        Self { stack }
    }

    /// Create a new scope guard with specific capacity
    #[inline]
    pub fn with_capacity(stack: &'a mut ScopeStack, capacity: usize) -> Self {
        stack.push_scope_with_capacity(capacity);
        Self { stack }
    }

    /// Get access to the scope stack
    #[inline]
    pub fn stack(&self) -> &ScopeStack {
        self.stack
    }

    /// Get mutable access to the scope stack
    #[inline]
    pub fn stack_mut(&mut self) -> &mut ScopeStack {
        self.stack
    }
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        self.stack.pop_scope();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_basic() {
        let mut scope = Scope::new();

        scope.set("foo", "bar".to_string());
        assert_eq!(scope.get("foo"), Some("bar"));
        assert!(scope.contains("foo"));

        scope.remove("foo");
        assert_eq!(scope.get("foo"), None);
    }

    #[test]
    fn test_scope_local() {
        let mut scope = Scope::new();

        scope.mark_local("local_var");
        scope.set("local_var", "value".to_string());

        assert!(scope.is_local("local_var"));
        assert!(!scope.is_local("other_var"));
    }

    #[test]
    fn test_scope_stack_basic() {
        let mut stack = ScopeStack::new();

        stack.set("global_var", "global_value".to_string());
        assert_eq!(stack.get("global_var"), Some("global_value"));
    }

    #[test]
    fn test_scope_stack_nested() {
        let mut stack = ScopeStack::new();

        // Set global variable
        stack.set("x", "global".to_string());
        assert_eq!(stack.get("x"), Some("global"));

        // Push new scope (function call)
        stack.push_scope();

        // Can still see global
        assert_eq!(stack.get("x"), Some("global"));

        // Shadow with local
        stack.set_local("x", "local".to_string());
        assert_eq!(stack.get("x"), Some("local"));

        // Pop scope
        stack.pop_scope();

        // Back to global value
        assert_eq!(stack.get("x"), Some("global"));
    }

    #[test]
    fn test_scope_stack_local_only() {
        let mut stack = ScopeStack::new();

        stack.push_scope();
        stack.set_local("local_only", "value".to_string());
        assert_eq!(stack.get("local_only"), Some("value"));

        stack.pop_scope();
        assert_eq!(stack.get("local_only"), None);
    }

    #[test]
    fn test_scope_guard() {
        let mut stack = ScopeStack::new();
        stack.set("outer", "value".to_string());

        assert_eq!(stack.depth(), 0);

        {
            let mut guard = ScopeGuard::new(&mut stack);
            assert_eq!(guard.stack().depth(), 1);
            guard
                .stack_mut()
                .set_local("inner", "inner_value".to_string());
            assert_eq!(guard.stack().get("inner"), Some("inner_value"));
        }

        // Guard dropped, scope popped
        assert_eq!(stack.depth(), 0);
        assert_eq!(stack.get("inner"), None);
        assert_eq!(stack.get("outer"), Some("value"));
    }

    #[test]
    fn test_deep_nesting() {
        let mut stack = ScopeStack::new();

        for i in 0..10 {
            stack.push_scope();
            stack.set_local(&format!("var_{}", i), format!("value_{}", i));
        }

        assert_eq!(stack.depth(), 10);

        // Can see all variables
        for i in 0..10 {
            assert_eq!(
                stack.get(&format!("var_{}", i)),
                Some(format!("value_{}", i).as_str())
            );
        }

        // Pop all scopes
        for _ in 0..10 {
            stack.pop_scope();
        }

        assert_eq!(stack.depth(), 0);

        // All local variables are gone
        for i in 0..10 {
            assert_eq!(stack.get(&format!("var_{}", i)), None);
        }
    }

    #[test]
    fn test_visible_vars() {
        let mut stack = ScopeStack::new();

        stack.set_global("global1", "g1".to_string());
        stack.set_global("global2", "g2".to_string());

        stack.push_scope();
        stack.set_local("local1", "l1".to_string());
        stack.set_local("global1", "shadowed".to_string()); // Shadow global

        let visible = stack.visible_vars();
        assert!(visible.contains(&&"global1".to_string()));
        assert!(visible.contains(&&"global2".to_string()));
        assert!(visible.contains(&&"local1".to_string()));

        // Global1 should have the shadowed value
        assert_eq!(stack.get("global1"), Some("shadowed"));
    }
}
