use textual_rs::terminal::MouseCaptureStack;

#[test]
fn new_stack_is_enabled_by_default() {
    let stack = MouseCaptureStack::new();
    assert!(stack.is_enabled(), "new stack should default to enabled (captured)");
}

#[test]
fn push_false_disables() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    assert!(!stack.is_enabled(), "push(false) should disable capture");
}

#[test]
fn push_false_then_pop_restores_true() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    stack.pop();
    assert!(stack.is_enabled(), "pop after push(false) should restore default true");
}

#[test]
fn push_false_push_true_pop_returns_to_false() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    stack.push(true);
    assert!(stack.is_enabled(), "inner push(true) should enable");
    stack.pop();
    assert!(!stack.is_enabled(), "after popping inner, should be back to false");
}

#[test]
fn nested_push_false_pop_pop_restores_true() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    stack.push(false);
    assert!(!stack.is_enabled());
    stack.pop();
    assert!(!stack.is_enabled(), "still false after first pop");
    stack.pop();
    assert!(stack.is_enabled(), "restored to default after both pops");
}

#[test]
fn pop_on_empty_stack_is_noop() {
    let mut stack = MouseCaptureStack::new();
    stack.pop(); // should not panic
    assert!(stack.is_enabled(), "empty stack pop should keep default true");
}

#[test]
fn push_true_on_default_true_stays_enabled() {
    let mut stack = MouseCaptureStack::new();
    stack.push(true);
    assert!(stack.is_enabled(), "push(true) on default-true should stay enabled");
}

#[test]
fn push_returns_previous_state() {
    let mut stack = MouseCaptureStack::new();
    let prev = stack.push(false);
    assert!(prev, "push on empty stack should return previous state true");
    let prev2 = stack.push(true);
    assert!(!prev2, "push on false stack should return false");
}

#[test]
fn pop_returns_new_state() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    let new_state = stack.pop();
    assert!(new_state, "pop should return new is_enabled (default true)");
}

#[test]
fn reset_clears_stack() {
    let mut stack = MouseCaptureStack::new();
    stack.push(false);
    stack.push(false);
    stack.reset();
    assert!(stack.is_enabled(), "reset should restore to default enabled");
}
