---
phase: quick
plan: 260326-uwc
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/textual-rs/src/widget/context.rs
  - crates/textual-rs/src/app.rs
  - crates/textual-rs/src/terminal.rs
  - crates/textual-rs/tests/mouse_capture.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "Mouse capture can be disabled/re-enabled without competing callers clobbering each other"
    - "Holding Shift while clicking/dragging bypasses mouse capture for native text selection"
    - "Terminal resize does not leave mouse capture stuck in wrong state"
  artifacts:
    - path: "crates/textual-rs/src/widget/context.rs"
      provides: "MouseCaptureStack and push/pop API on AppContext"
      contains: "MouseCaptureStack"
    - path: "crates/textual-rs/src/app.rs"
      provides: "Shift-modifier bypass in mouse event handler, resize guard integration"
    - path: "crates/textual-rs/tests/mouse_capture.rs"
      provides: "Unit tests for push/pop stack and Shift bypass logic"
  key_links:
    - from: "crates/textual-rs/src/app.rs"
      to: "AppContext.mouse_capture_stack"
      via: "check stack top before handling mouse events"
      pattern: "mouse_capture_stack"
    - from: "crates/textual-rs/src/app.rs"
      to: "crossterm EnableMouseCapture/DisableMouseCapture"
      via: "execute! calls when stack state changes"
      pattern: "EnableMouseCapture|DisableMouseCapture"
---

<objective>
Implement a push/pop mouse capture stack on AppContext with Shift-modifier bypass and resize guard.

Purpose: Enable screens/widgets to temporarily disable mouse capture (for native terminal text selection) without competing enable/disable calls clobbering each other. Shift+click always bypasses capture regardless of stack state, matching Python Textual behavior.

Output: MouseCaptureStack type, push/pop API on AppContext, Shift bypass in event loop, resize guard, unit tests.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@crates/textual-rs/src/widget/context.rs
@crates/textual-rs/src/app.rs
@crates/textual-rs/src/terminal.rs
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: MouseCaptureStack type and AppContext integration</name>
  <files>crates/textual-rs/src/terminal.rs, crates/textual-rs/src/widget/context.rs, crates/textual-rs/tests/mouse_capture.rs</files>
  <behavior>
    - Test: new stack is_enabled() returns true (mouse captured by default)
    - Test: push(false) makes is_enabled() return false
    - Test: push(false) then pop() restores is_enabled() to true
    - Test: push(false), push(true), pop() returns to false (inner push restored)
    - Test: nested push(false), push(false), pop(), pop() restores to true
    - Test: pop() on empty stack is a no-op (does not panic, stays at default true)
    - Test: push(true) on default-true stack keeps is_enabled() true
  </behavior>
  <action>
    Create `MouseCaptureStack` in `crates/textual-rs/src/terminal.rs` (near TerminalGuard):

    ```rust
    /// Stack-based mouse capture state. The effective state is the top of the stack,
    /// defaulting to `true` (captured) when empty. Screens/widgets push to temporarily
    /// override; pop to restore. This prevents competing enable/disable calls from
    /// clobbering each other.
    #[derive(Debug, Clone)]
    pub struct MouseCaptureStack {
        stack: Vec<bool>,
    }

    impl MouseCaptureStack {
        pub fn new() -> Self { Self { stack: Vec::new() } }

        /// Current effective mouse-capture state. True = terminal captures mouse events;
        /// false = pass-through to terminal emulator for native selection.
        pub fn is_enabled(&self) -> bool {
            self.stack.last().copied().unwrap_or(true)
        }

        /// Push a new capture state. Returns the previous is_enabled() value
        /// so the caller can detect transitions.
        pub fn push(&mut self, enabled: bool) -> bool {
            let prev = self.is_enabled();
            self.stack.push(enabled);
            prev
        }

        /// Pop the top capture state. Returns the new is_enabled() value.
        /// No-op if stack is empty (default state cannot be popped).
        pub fn pop(&mut self) -> bool {
            self.stack.pop();
            self.is_enabled()
        }

        /// Reset to default state (empty stack = captured). Used by resize guard.
        pub fn reset(&mut self) {
            self.stack.clear();
        }
    }
    ```

    Add to `AppContext`:
    - New field: `pub mouse_capture_stack: MouseCaptureStack` (initialized with `MouseCaptureStack::new()`)
    - New method: `pub fn push_mouse_enabled(&self, enabled: bool)` -- wraps stack push via RefCell. Actually, since AppContext fields are pub and the event loop has `&mut self`, make `mouse_capture_stack` a plain field (not RefCell). Widgets needing to toggle mouse capture should use a deferred pattern like `pending_mouse_capture_push: RefCell<Vec<bool>>` and `pending_mouse_capture_pops: Cell<usize>`, drained by the event loop (same pattern as pending_screen_pushes/pops).
    - New deferred fields on AppContext:
      - `pub pending_mouse_push: RefCell<Vec<bool>>` -- widgets push desired states here
      - `pub pending_mouse_pops: Cell<usize>` -- widgets increment this to schedule pops
    - New convenience methods on AppContext (&self):
      - `pub fn push_mouse_capture(&self, enabled: bool)` -- pushes to pending_mouse_push
      - `pub fn pop_mouse_capture(&self)` -- increments pending_mouse_pops

    Write tests in `crates/textual-rs/tests/mouse_capture.rs` covering the behaviors listed above. Tests operate directly on `MouseCaptureStack` (no terminal needed).
  </action>
  <verify>
    <automated>cargo test --test mouse_capture</automated>
  </verify>
  <done>MouseCaptureStack type exists with push/pop/is_enabled/reset. AppContext has mouse_capture_stack field and deferred push/pop fields. All unit tests pass.</done>
</task>

<task type="auto">
  <name>Task 2: Shift bypass, event loop drain, and resize guard in App</name>
  <files>crates/textual-rs/src/app.rs</files>
  <action>
    Wire MouseCaptureStack into the App event loop in `crates/textual-rs/src/app.rs`:

    **1. Drain deferred mouse capture changes (add to event loop after drain_message_queue):**
    Create a new method `fn drain_mouse_capture_changes(&mut self)` that:
    - Drains `self.ctx.pending_mouse_push` and calls `self.ctx.mouse_capture_stack.push(enabled)` for each
    - Reads `self.ctx.pending_mouse_pops` count and calls `self.ctx.mouse_capture_stack.pop()` that many times, resets counter to 0
    - After draining, checks if `mouse_capture_stack.is_enabled()` changed from what terminal currently has
    - If changed to false: `execute!(io::stdout(), DisableMouseCapture)`
    - If changed to true: `execute!(io::stdout(), EnableMouseCapture)`
    - Track the "last sent to terminal" state with a bool field `mouse_capture_active: bool` on App (initialized to `true` since TerminalGuard enables it)

    Call `self.drain_mouse_capture_changes()` everywhere `self.drain_message_queue()` is called (right after it).

    **2. Shift-modifier bypass in mouse event handler:**
    At the top of the `Ok(AppEvent::Mouse(m))` arm, before any hit-test logic:
    ```rust
    // Shift+mouse bypasses capture — let terminal handle native text selection.
    // When Shift is held, crossterm still delivers the event but we ignore it,
    // allowing the terminal emulator to handle selection natively.
    if m.modifiers.contains(KeyModifiers::SHIFT) {
        continue;
    }
    ```
    This goes right after the overlay check (keep overlay routing above it so overlays still work with Shift, or put it before overlay -- put it BEFORE overlay routing so Shift always means "native selection").

    **3. Mouse-capture-disabled guard:**
    After the Shift bypass check, add:
    ```rust
    // If mouse capture is disabled (stack top = false), skip all mouse handling.
    // Events may still arrive briefly after DisableMouseCapture is sent.
    if !self.ctx.mouse_capture_stack.is_enabled() {
        continue;
    }
    ```

    **4. Resize guard:**
    In the `Ok(AppEvent::Resize(_, _))` arm, after setting `needs_full_sync = true`:
    - Re-sync terminal mouse capture state: if `mouse_capture_stack.is_enabled()` and `!self.mouse_capture_active`, re-enable; vice versa. This prevents resize from desynchronizing the capture state.
    - Simpler approach: just re-send the current desired state:
    ```rust
    if self.ctx.mouse_capture_stack.is_enabled() {
        let _ = execute!(io::stdout(), EnableMouseCapture);
    } else {
        let _ = execute!(io::stdout(), DisableMouseCapture);
    }
    self.mouse_capture_active = self.ctx.mouse_capture_stack.is_enabled();
    ```

    **5. Add `mouse_capture_active: bool` field to App struct**, initialized to `true` in both `new()` and `new_bare()`.

    **6. Update the `handle_mouse_event` method** (used by TestApp) to also check `mouse_capture_stack.is_enabled()` and modifiers -- add the same guards at the top.
  </action>
  <verify>
    <automated>cargo test 2>&1 | tail -5</automated>
  </verify>
  <done>Shift+mouse events are skipped (native selection passthrough). Mouse events are skipped when capture stack says disabled. Resize re-syncs capture state. Deferred push/pop drained in event loop. All existing tests still pass.</done>
</task>

</tasks>

<verification>
- `cargo test --test mouse_capture` -- all stack unit tests pass
- `cargo test` -- all existing tests pass (no regressions)
- `cargo clippy` -- no new warnings
</verification>

<success_criteria>
- MouseCaptureStack provides correct push/pop/is_enabled semantics
- AppContext exposes deferred push/pop API for widgets
- Shift+mouse bypasses capture in the event loop
- Resize re-syncs mouse capture state to prevent stuck modes
- All existing tests pass without modification
</success_criteria>

<output>
After completion, create `.planning/quick/260326-uwc-mouse-capture-push-pop-stack-and-shift-m/260326-uwc-SUMMARY.md`
</output>
