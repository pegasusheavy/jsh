//! Scope stack integration tests

use franken_shell::interpreter::{Interpreter, Scope, ScopeGuard, ScopeStack};

// =============================================================================
// ScopeStack Unit Tests
// =============================================================================

#[test]
fn test_scope_stack_new() {
    let stack = ScopeStack::new();
    assert!(stack.is_global());
    assert_eq!(stack.depth(), 0);
}

#[test]
fn test_scope_stack_push_pop() {
    let mut stack = ScopeStack::new();

    stack.push_scope();
    assert!(!stack.is_global());
    assert_eq!(stack.depth(), 1);

    stack.push_scope();
    assert_eq!(stack.depth(), 2);

    stack.pop_scope();
    assert_eq!(stack.depth(), 1);

    stack.pop_scope();
    assert_eq!(stack.depth(), 0);
    assert!(stack.is_global());
}

#[test]
fn test_scope_stack_variable_shadowing() {
    let mut stack = ScopeStack::new();

    // Set in global
    stack.set("x", "global".to_string());
    assert_eq!(stack.get("x"), Some("global"));

    // Push scope and shadow
    stack.push_scope();
    stack.set_local("x", "local".to_string());
    assert_eq!(stack.get("x"), Some("local"));

    // Pop and verify global restored
    stack.pop_scope();
    assert_eq!(stack.get("x"), Some("global"));
}

#[test]
fn test_scope_stack_local_not_visible_after_pop() {
    let mut stack = ScopeStack::new();

    stack.push_scope();
    stack.set_local("local_var", "value".to_string());
    assert!(stack.contains("local_var"));

    stack.pop_scope();
    assert!(!stack.contains("local_var"));
    assert_eq!(stack.get("local_var"), None);
}

// =============================================================================
// Scope Unit Tests
// =============================================================================

#[test]
fn test_scope_basic_operations() {
    let mut scope = Scope::new();

    scope.set("foo", "bar".to_string());
    assert_eq!(scope.get("foo"), Some("bar"));
    assert!(scope.contains("foo"));
    assert_eq!(scope.len(), 1);

    scope.remove("foo");
    assert_eq!(scope.get("foo"), None);
    assert!(!scope.contains("foo"));
    assert!(scope.is_empty());
}

#[test]
fn test_scope_local_marking() {
    let mut scope = Scope::new();

    scope.mark_local("x");
    assert!(scope.is_local("x"));
    assert!(!scope.is_local("y"));

    scope.set("x", "value".to_string());
    assert!(scope.is_local("x"));
}

#[test]
fn test_scope_with_capacity() {
    let scope = Scope::with_capacity(100);
    assert!(scope.is_empty());
}

// =============================================================================
// Interpreter Integration Tests
// =============================================================================

#[test]
fn test_interpreter_scope_depth() {
    let mut interp = Interpreter::new();

    assert!(interp.is_global_scope());
    assert_eq!(interp.scope_depth(), 0);

    interp.push_scope();
    assert!(!interp.is_global_scope());
    assert_eq!(interp.scope_depth(), 1);

    interp.pop_scope();
    assert!(interp.is_global_scope());
}

#[test]
fn test_interpreter_local_var() {
    let mut interp = Interpreter::new();

    // Set in global
    interp.set_var("global_var", "global_value");
    assert_eq!(interp.get_var("global_var"), Some("global_value"));

    // Enter function scope
    interp.push_scope();

    // Set local
    interp.set_local_var("local_var", "local_value");
    assert_eq!(interp.get_var("local_var"), Some("local_value"));

    // Global still visible
    assert_eq!(interp.get_var("global_var"), Some("global_value"));

    // Exit function scope
    interp.pop_scope();

    // Local no longer visible
    assert_eq!(interp.get_var("local_var"), None);

    // Global still there
    assert_eq!(interp.get_var("global_var"), Some("global_value"));
}

#[test]
fn test_interpreter_shadowing() {
    let mut interp = Interpreter::new();

    interp.set_var("x", "outer");

    interp.push_scope();
    interp.set_local_var("x", "inner");
    assert_eq!(interp.get_var("x"), Some("inner"));

    interp.pop_scope();
    assert_eq!(interp.get_var("x"), Some("outer"));
}

#[test]
fn test_interpreter_nested_scopes() {
    let mut interp = Interpreter::new();

    interp.set_var("level", "0");

    interp.push_scope();
    interp.set_local_var("level", "1");
    interp.set_local_var("level1_var", "value1");

    interp.push_scope();
    interp.set_local_var("level", "2");
    interp.set_local_var("level2_var", "value2");

    // At level 2
    assert_eq!(interp.get_var("level"), Some("2"));
    assert_eq!(interp.get_var("level1_var"), Some("value1"));
    assert_eq!(interp.get_var("level2_var"), Some("value2"));

    interp.pop_scope();

    // At level 1
    assert_eq!(interp.get_var("level"), Some("1"));
    assert_eq!(interp.get_var("level1_var"), Some("value1"));
    assert_eq!(interp.get_var("level2_var"), None);

    interp.pop_scope();

    // At level 0
    assert_eq!(interp.get_var("level"), Some("0"));
    assert_eq!(interp.get_var("level1_var"), None);
}

// =============================================================================
// ScopeGuard Tests
// =============================================================================

#[test]
fn test_scope_guard_auto_pop() {
    let mut stack = ScopeStack::new();
    stack.set("outer", "value".to_string());

    {
        let mut guard = ScopeGuard::new(&mut stack);
        guard
            .stack_mut()
            .set_local("inner", "inner_value".to_string());
        assert_eq!(guard.stack().depth(), 1);
    }
    // Guard dropped, scope automatically popped

    assert_eq!(stack.depth(), 0);
    assert_eq!(stack.get("inner"), None);
    assert_eq!(stack.get("outer"), Some("value"));
}

#[test]
fn test_scope_guard_with_capacity() {
    let mut stack = ScopeStack::new();

    {
        let guard = ScopeGuard::with_capacity(&mut stack, 100);
        assert_eq!(guard.stack().depth(), 1);
    }

    assert_eq!(stack.depth(), 0);
}

// =============================================================================
// Stress Tests for Deeply Nested Scopes
// =============================================================================

#[test]
fn test_stress_deeply_nested_scopes_100() {
    let mut stack = ScopeStack::new();
    const DEPTH: usize = 100;

    // Push 100 nested scopes
    for i in 0..DEPTH {
        stack.push_scope();
        stack.set_local(&format!("var_{}", i), format!("value_{}", i));
        assert_eq!(stack.depth(), i + 1);
    }

    // Verify all variables are visible
    for i in 0..DEPTH {
        assert_eq!(
            stack.get(&format!("var_{}", i)),
            Some(format!("value_{}", i).as_str())
        );
    }

    // Pop all scopes and verify variables are removed
    for i in (0..DEPTH).rev() {
        assert_eq!(stack.depth(), i + 1);
        stack.pop_scope();
        assert_eq!(stack.get(&format!("var_{}", i)), None);
    }

    assert_eq!(stack.depth(), 0);
    assert!(stack.is_global());
}

#[test]
fn test_stress_deeply_nested_scopes_500() {
    let mut stack = ScopeStack::new();
    const DEPTH: usize = 500;

    // Push 500 nested scopes - tests SmallVec behavior
    for i in 0..DEPTH {
        stack.push_scope();
        stack.set_local("x", format!("level_{}", i));
    }

    // Verify deepest value
    assert_eq!(stack.get("x"), Some(&format!("level_{}", DEPTH - 1)[..]));
    assert_eq!(stack.depth(), DEPTH);

    // Pop all and verify
    for i in (0..DEPTH).rev() {
        stack.pop_scope();
        if i > 0 {
            assert_eq!(stack.get("x"), Some(&format!("level_{}", i - 1)[..]));
        }
    }

    assert!(stack.is_global());
}

#[test]
fn test_stress_shadowing_deep_hierarchy() {
    let mut stack = ScopeStack::new();
    const DEPTH: usize = 50;

    // Set global variable
    stack.set("shared", "global".to_string());

    // Create deep hierarchy where each level shadows 'shared'
    for i in 0..DEPTH {
        stack.push_scope();
        stack.set_local("shared", format!("level_{}", i));
    }

    // Verify deepest shadow
    assert_eq!(
        stack.get("shared"),
        Some(&format!("level_{}", DEPTH - 1)[..])
    );

    // Pop and verify each level sees correct shadow
    for i in (0..DEPTH).rev() {
        assert_eq!(stack.get("shared"), Some(&format!("level_{}", i)[..]));
        stack.pop_scope();
    }

    // Back to global
    assert_eq!(stack.get("shared"), Some("global"));
}

#[test]
fn test_stress_many_variables_per_scope() {
    let mut stack = ScopeStack::new();
    const VARS_PER_SCOPE: usize = 1000;
    const SCOPES: usize = 10;

    for scope in 0..SCOPES {
        stack.push_scope();

        // Create many variables in each scope
        for var in 0..VARS_PER_SCOPE {
            stack.set_local(
                &format!("scope{}_var{}", scope, var),
                format!("value_{}_{}", scope, var),
            );
        }
    }

    // Verify all variables exist
    for scope in 0..SCOPES {
        for var in 0..VARS_PER_SCOPE {
            assert!(
                stack.contains(&format!("scope{}_var{}", scope, var)),
                "Variable scope{}_var{} should exist",
                scope,
                var
            );
        }
    }

    // Pop and verify scope variables disappear
    for scope in (0..SCOPES).rev() {
        stack.pop_scope();

        // Variables from popped scope should be gone
        for var in 0..VARS_PER_SCOPE {
            assert!(
                !stack.contains(&format!("scope{}_var{}", scope, var)),
                "Variable scope{}_var{} should not exist after pop",
                scope,
                var
            );
        }
    }
}

#[test]
fn test_stress_rapid_push_pop() {
    let mut stack = ScopeStack::new();
    const ITERATIONS: usize = 10000;

    // Rapidly push and pop scopes
    for _ in 0..ITERATIONS {
        stack.push_scope();
        stack.set_local("temp", "value".to_string());
        assert!(stack.contains("temp"));
        stack.pop_scope();
        assert!(!stack.contains("temp"));
    }

    assert!(stack.is_global());
}

#[test]
fn test_stress_mixed_global_local() {
    let mut stack = ScopeStack::new();
    const GLOBALS: usize = 100;
    const DEPTH: usize = 20;

    // Set many globals
    for i in 0..GLOBALS {
        stack.set(&format!("global_{}", i), format!("g_value_{}", i));
    }

    // Create nested scopes that also reference globals
    for level in 0..DEPTH {
        stack.push_scope();

        // Set some locals
        stack.set_local(&format!("local_{}", level), format!("l_value_{}", level));

        // Shadow some globals
        if level % 2 == 0 {
            stack.set_local(&format!("global_{}", level), format!("shadowed_{}", level));
        }
    }

    // Verify shadowing
    for level in 0..DEPTH {
        if level % 2 == 0 {
            assert_eq!(
                stack.get(&format!("global_{}", level)),
                Some(&format!("shadowed_{}", level)[..])
            );
        }
    }

    // Pop all and verify globals restored
    for _ in 0..DEPTH {
        stack.pop_scope();
    }

    for i in 0..GLOBALS {
        assert_eq!(
            stack.get(&format!("global_{}", i)),
            Some(&format!("g_value_{}", i)[..])
        );
    }
}

#[test]
fn test_stress_interpreter_nested_function_calls() {
    use franken_shell::Shell;

    let mut shell = Shell::with_options(false, false).unwrap();

    // Define nested functions that call each other
    let result = shell.run_command(
        r#"
        level1() {
            local x=1
            echo "level1 x=$x"
        }
        level2() {
            local x=2
            level1
            echo "level2 x=$x"
        }
        level3() {
            local x=3
            level2
            echo "level3 x=$x"
        }
        level3
    "#,
    );

    // Should complete without stack issues
    assert!(result.is_ok());
}

#[test]
fn test_stress_interpreter_deep_nested_loops() {
    use franken_shell::Shell;

    let mut shell = Shell::with_options(false, false).unwrap();

    // Create deeply nested loops (each creates a scope)
    let result = shell.run_command(
        r#"
        COUNT=0
        for i in 1 2 3; do
            for j in 1 2 3; do
                for k in 1 2 3; do
                    for l in 1 2 3; do
                        COUNT=$((COUNT + 1))
                    done
                done
            done
        done
    "#,
    );

    assert!(result.is_ok());
    // 3^4 = 81 iterations
    assert_eq!(shell.interpreter.get_var("COUNT"), Some("81"));
}

#[test]
fn test_stress_interpreter_many_local_variables() {
    use franken_shell::Shell;

    let mut shell = Shell::with_options(false, false).unwrap();

    // Function with many local variables - test they don't interfere
    let result = shell.run_command(
        r#"
        many_locals() {
            local a=1
            local b=2
            local c=3
            local d=4
            local e=5
            INNER_SUM=$((a+b+c+d+e))
        }
        OUTER=100
        many_locals
        # OUTER should not be affected by local vars
    "#,
    );

    assert!(result.is_ok());
    assert_eq!(shell.interpreter.get_var("OUTER"), Some("100"));
    assert_eq!(shell.interpreter.get_var("INNER_SUM"), Some("15"));
}

#[test]
fn test_stress_scope_guard_deeply_nested() {
    let mut stack = ScopeStack::new();
    const DEPTH: usize = 50;

    // Push all scopes manually to simulate deep nesting
    for i in 0..DEPTH {
        stack.push_scope();
        stack.set_local("depth", format!("{}", i));

        assert_eq!(stack.get("depth"), Some(&format!("{}", i)[..]));
    }

    // Verify deepest value
    assert_eq!(stack.get("depth"), Some(&format!("{}", DEPTH - 1)[..]));

    // Pop all scopes and verify each level
    for i in (0..DEPTH).rev() {
        assert_eq!(stack.get("depth"), Some(&format!("{}", i)[..]));
        stack.pop_scope();
    }

    // After all scopes popped, back to global
    assert!(stack.is_global());
    assert_eq!(stack.get("depth"), None);
}
