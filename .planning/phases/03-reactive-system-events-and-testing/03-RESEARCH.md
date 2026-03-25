# Phase 3: Reactive System, Events, and Testing - Research

**Researched:** 2026-03-25
**Domain:** Reactive signals (reactive_graph), event dispatch, test infrastructure (ratatui TestBackend, insta, proptest)
**Confidence:** HIGH — spike verified, all core claims validated against official docs or live code

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Reactive Property System**
- D-01: Use `reactive_graph` crate (Leptos signals). `RwSignal<T>`, `Memo<T>`, `Effect` for automatic dependency tracking. Integrate with Tokio LocalSet via `Executor::init_tokio()`. Research phase MUST include a spike verifying this integration works before planning commits to it. If the spike fails, fall back to hand-rolled signals.
- D-02: Render batching via Effect + flume. A `reactive_graph` Effect detects reactive changes and posts a `RenderRequest` to the existing flume event channel. The App event loop coalesces multiple `RenderRequest`s into a single render pass per tick. Multiple reactive field changes in one tick produce exactly one re-render.
- D-03: `Reactive<T>` wraps `RwSignal<T>` with textual-rs ergonomics. `watch_` method convention: a method called automatically when the reactive property changes. `validate_` method convention: validate/coerce on set. `compute_` method convention: derive from reactive sources (maps to `Memo<T>`).

**Event Dispatch Model**
- D-04: `on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation` added to Widget trait. Widgets downcast `event` to concrete message types they handle. Returns `EventPropagation::Stop` to consume or `::Continue` to bubble. No handler registration map needed.
- D-05: Bubbling walks the parent chain. Collect `Vec<WidgetId>` from widget to screen root, then iterate calling `on_event` at each level. Propagation stops when any handler returns `Stop`.
- D-06: Typed messages implement a `Message` trait. Widgets define their own message structs (e.g., `Button::Pressed`, `Input::Changed`). Messages are posted to a queue and dispatched through the bubbling mechanism.

**Keyboard and Mouse Routing**
- D-07: Key events dispatch to focused widget first, then bubble up the parent chain. If no widget consumes the key, the App handles it.
- D-08: Mouse events use the existing `MouseHitMap` from Phase 2 for hit testing. Click events dispatch to the topmost widget at the clicked cell, then bubble up.
- D-09: Key bindings via static binding table. Widget trait gets `fn key_bindings(&self) -> &[KeyBinding]` returning entries like `(Key::Char('q'), "quit")`. Action strings dispatch to `fn on_action(&self, action: &str, ctx: &AppContext)`.
- D-10: Timer/interval support for periodic updates (EVENT-08). Details at Claude's discretion — likely a `tokio::time::interval` that posts Tick events to the flume channel.

**Test Infrastructure**
- D-11: Async tests with `#[tokio::test]`. TestApp creates its own LocalSet internally. Pilot methods are async. `settle().await` drains the event loop until no pending messages remain.
- D-12: `TestApp::new(|| Box::new(MyScreen))` wraps App with TestBackend. Returns a handle with `pilot()` for sending events and `app()` for inspecting state.
- D-13: Snapshot testing with `insta` using plain text buffer lines. Render to TestBackend, extract buffer rows as strings, snapshot with `insta::assert_snapshot!`.
- D-14: `assert_buffer_lines()` helper for cell-level assertions without full snapshots. `proptest` for CSS parser and layout engine fuzz testing.

### Claude's Discretion
- Timer/interval implementation details
- Exact `Reactive<T>` API surface beyond the conventions (get/set/update methods)
- Whether `on_event` takes `&mut AppContext` or `&AppContext` (borrow analysis needed)
- proptest strategy design for CSS parser fuzzing
- Mouse event types beyond click (drag, scroll, hover)
- How `settle()` detects quiescence (empty queues + no pending effects)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REACT-01 | `Reactive<T>` property type that triggers re-render on change | reactive_graph `ArcRwSignal<T>` wraps cleanly; Effect posts `RenderRequest` to flume |
| REACT-02 | `watch_` method convention: method called when reactive property changes | Effect closure calls `watch_<field_name>(&self, old, new)` on the widget |
| REACT-03 | `validate_` method convention: validate and coerce on set | Custom setter on `Reactive<T>` calls `validate_<field_name>` before writing signal |
| REACT-04 | `compute_` method convention: derive from reactive sources | `ArcMemo<T>` wrapping a closure that reads other signals |
| REACT-05 | Render batching — multiple reactive changes in one tick produce one render pass | Spike confirmed: each `set()` + `yield_now()` fires effect once; coalescing `RenderRequest`s in the event loop achieves per-tick batching |
| EVENT-01 | Typed message system — messages are Rust structs implementing `Message` trait | Marker trait approach with `dyn Any` downcasting confirmed viable |
| EVENT-02 | `on_` method convention for message handling | Implemented as `on_event()` with manual downcast; `on_` naming for per-message methods is user convention, not framework dispatch |
| EVENT-03 | Event bubbling — unhandled messages propagate up widget tree | Parent chain traversal already exists in `tree.rs`; walk `ctx.parent` chain |
| EVENT-04 | Event stopping — handlers consume message to prevent bubbling | `EventPropagation::Stop` already defined in `widget/mod.rs` |
| EVENT-05 | Keyboard event routing to focused widget | `ctx.focused_widget` already tracked; dispatch key → focused widget → bubble |
| EVENT-06 | Mouse event routing with hit testing | `MouseHitMap` already built in `layout/hit_map.rs` |
| EVENT-07 | Key bindings — declare bindings with action dispatch | `key_bindings() -> &[KeyBinding]` on Widget trait; `on_action()` dispatch |
| EVENT-08 | Timer/interval support | `tokio::time::interval` spawned in LocalSet, posts `AppEvent::Tick` to flume |
| TEST-01 | `TestApp` harness with no real terminal | `ratatui::backend::TestBackend` already used in Phase 2; wrap in ergonomic `TestApp` |
| TEST-02 | `Pilot` type for simulating key presses, mouse, focus | Async Pilot with press/type_text/click/focus methods |
| TEST-03 | `settle().await` drains pending messages | Drain flume channel + poll reactive effects via `Executor::poll_local()` |
| TEST-04 | Snapshot testing with `insta` | `assert_snapshot!(terminal.backend())` — ratatui TestBackend implements Display |
| TEST-05 | `assert_buffer_lines()` and cell-level assertions | `ratatui::backend::TestBackend::assert_buffer_lines()` built-in |
| TEST-06 | Property-based tests with `proptest` | `proptest!` macro with `prop::string::string_regex` for CSS fuzzing |
</phase_requirements>

---

## Summary

Phase 3 builds three interconnected systems: reactive signals, event dispatch, and test infrastructure.

**Reactive system:** The `reactive_graph` crate (Leptos signals, v0.2.13) provides `ArcRwSignal<T>`, `ArcMemo<T>`, and `Effect` as the signal primitives. **The spike PASSED**: `Executor::init_tokio()` + `LocalSet::block_on()` works correctly — Effects fire, Memos update, and the integration is solid. The key insight is that `any_spawner::Executor::init_tokio()` registers `tokio::task::spawn_local` as the local spawner, so all Effects and Memos run correctly within the existing `LocalSet` context already established by `App::run()`. `Reactive<T>` will wrap `ArcRwSignal<T>` (Arc-based, cloneable across closures) rather than arena-allocated `RwSignal<T>` (which requires an active Owner context to create).

**Event dispatch:** The architecture is straightforward Rust: `Message` is a marker trait, widgets downcast `&dyn Any` to concrete types in `on_event()`. Bubbling walks `ctx.parent` which already exists. The only borrow complexity is that `on_event(&self, ...)` takes `&self` (consistent with Phase 2 decisions), so any state changes go through the message queue rather than direct mutation.

**Test infrastructure:** `ratatui::TestBackend` is already used in Phase 2. Phase 3 wraps it in an ergonomic `TestApp`/`Pilot` harness. `insta 1.46.3` provides `assert_snapshot!` which natively handles `TestBackend`'s `Display` impl. `proptest 1.11.0` is mature and straightforward. The `settle().await` primitive needs to drain both the flume channel and reactive effects — this is achievable by alternating between `rx.try_recv()` loops and `Executor::poll_local()` calls.

**Primary recommendation:** Proceed directly to planning. The reactive_graph spike confirmed integration works. Add `reactive_graph = "0.2.13"` and `any_spawner = "0.3.0"` as regular (non-dev) dependencies since they're needed in library code. Add `insta` and `proptest` as dev-only. Note: workspace `rust-version = "1.86"` must be bumped to `"1.88"` to satisfy reactive_graph 0.2.13's MSRV requirement.

---

## Project Constraints (from CLAUDE.md)

No `CLAUDE.md` found in the repository root. No project-specific overrides apply.

---

## Standard Stack

### Core (Phase 3 Additions)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reactive_graph | 0.2.13 | Fine-grained reactive signals (RwSignal, Memo, Effect) | Leptos signal engine — proven in production, async runtime agnostic, push-pull avoids diamond problem |
| any_spawner | 0.3.0 | Runtime-agnostic executor init for reactive_graph effects | Required companion — reactive_graph uses it for spawn_local internally |
| insta | 1.46.3 | Snapshot testing with human-readable diffs | ratatui's own test recipes recommend it; TestBackend implements Display |
| proptest | 1.11.0 | Property-based/fuzz testing for CSS parser and layout engine | Standard Rust fuzz testing; regex-based string strategies ideal for CSS fuzzing |

### Already Present (Phase 1-2)

| Library | Version | Purpose |
|---------|---------|---------|
| ratatui | 0.30.0 | Includes TestBackend for headless rendering |
| tokio | 1 (with `rt`, `macros`, `time`) | LocalSet for !Send widget code; `time` feature for interval timers |
| flume | 0.12 | Event bus channel — extended with RenderRequest, MessageDispatched variants |
| slotmap | 1.0 | Widget arena (AppContext) — unchanged |

### Rust Version Constraint

reactive_graph 0.2.13 requires `rust-version = "1.88"`. The workspace currently declares `rust-version = "1.86"`. **The installed compiler is 1.94 so this compiles fine**, but the workspace declaration must be bumped in `Cargo.toml` (workspace level) to `"1.88"` to avoid cargo warnings and accurately document the actual MSRV.

**Installation:**
```bash
# In crates/textual-rs/Cargo.toml
[dependencies]
reactive_graph = { version = "0.2.13", features = ["effects"] }
any_spawner = { version = "0.3.0", features = ["tokio"] }

[dev-dependencies]
insta = "1.46.3"
proptest = "1.11.0"

# In Cargo.toml (workspace root)
[workspace.package]
rust-version = "1.88"   # bumped from 1.86 for reactive_graph 0.2.13
```

**Version verification:** Confirmed against crates.io on 2026-03-25:
- `reactive_graph 0.2.13` — latest; rust-version 1.88
- `any_spawner 0.3.0` — latest
- `insta 1.46.3` — latest
- `proptest 1.11.0` — latest

---

## Architecture Patterns

### Reactive Property System

#### How reactive_graph Works with Tokio LocalSet

**SPIKE RESULT (verified 2026-03-25):** Integration confirmed working.

Setup sequence that MUST be followed (order matters):
1. `LocalSet::block_on(&rt, async { ... })` — establishes LocalSet context
2. `let _ = Executor::init_tokio();` — registers tokio's `spawn_local` as the local spawner. Returns `Err` if already called; ignore the error.
3. `let _owner = Owner::new();` — creates a reactive Owner scope; must be kept alive for signals/effects to work
4. Create signals, memos, effects inside the async block

```rust
// Source: spike verified 2026-03-25 in this project
use any_spawner::Executor;
use reactive_graph::{
    computed::ArcMemo,
    effect::Effect,
    owner::Owner,
    prelude::*,
    signal::ArcRwSignal,
};

let rt = Builder::new_current_thread().enable_all().build()?;
let local = LocalSet::new();
local.block_on(&rt, async {
    let _ = Executor::init_tokio();
    let _owner = Owner::new();

    let count = ArcRwSignal::new(0i32);
    let count_clone = count.clone();

    Effect::new(move |_| {
        println!("count = {}", *count_clone.read());
    });

    tokio::task::yield_now().await;  // let initial effect run
    count.set(1);
    tokio::task::yield_now().await;  // let effect re-run
});
```

**Critical facts from spike:**
- Effects fire asynchronously — they run on the NEXT `yield_now()` after signal mutation, not synchronously
- Each `set()` followed by `yield_now()` produces exactly ONE effect fire
- `ArcRwSignal` (reference-counted) is preferred over `RwSignal` (arena-allocated) because it can be cloned into closures without an active Owner at the clone site
- Accessing a `Memo` outside a tracking context (outside an Effect) prints a diagnostic warning. Use `.get_untracked()` to suppress it when reading for inspection/test assertions

#### Reactive<T> Wrapper Design

```rust
// Source: architecture derived from reactive_graph docs + Textual Python reactive.py
use reactive_graph::signal::ArcRwSignal;
use reactive_graph::prelude::*;

pub struct Reactive<T: Clone + PartialEq + 'static> {
    inner: ArcRwSignal<T>,
}

impl<T: Clone + PartialEq + 'static> Reactive<T> {
    pub fn new(value: T) -> Self {
        Self { inner: ArcRwSignal::new(value) }
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }

    pub fn get_untracked(&self) -> T {
        self.inner.get_untracked()
    }

    pub fn set(&self, value: T) {
        self.inner.set(value);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(f);
    }

    /// Returns a clone of the inner signal for use in closures (Effects, Memos)
    pub fn signal(&self) -> ArcRwSignal<T> {
        self.inner.clone()
    }
}
```

#### RenderRequest Batching via Effect + flume

The render batching design (D-02):

```rust
// Inside App::run_async(), after Executor::init_tokio():
let tx_clone = tx.clone();
let render_signal = ArcRwSignal::new(0u64);  // or a unit signal
let render_signal_clone = render_signal.clone();

Effect::new(move |_| {
    let _ = render_signal_clone.read();  // track the signal
    let _ = tx_clone.try_send(AppEvent::RenderRequest);
});
```

The App event loop match arm coalesces:
```rust
// Drain all pending RenderRequests from the channel, render once
Ok(AppEvent::RenderRequest) => {
    // Drain any additional RenderRequests queued in same tick
    while let Ok(AppEvent::RenderRequest) = rx.try_recv() {}
    self.full_render_pass(&mut terminal)?;
}
```

### Event Dispatch Pattern

#### Message Trait and Typed Messages

```rust
// Source: design derived from textual/src/textual/message.py patterns
use std::any::Any;

/// Marker trait for all typed messages in textual-rs.
/// Messages are plain structs — no heap allocation required.
pub trait Message: Any + 'static {
    /// Whether this message bubbles up the widget tree by default.
    fn bubbles() -> bool where Self: Sized { true }
}

// Widget-scoped message (inside a Button module):
pub struct Pressed {
    pub button_id: WidgetId,
}
impl Message for Pressed {}
```

#### on_event Dispatch and Bubbling

```rust
// Widget trait extension for Phase 3:
pub trait Widget: 'static {
    // ... existing methods ...
    fn on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation {
        EventPropagation::Continue  // default: pass through
    }
    fn key_bindings(&self) -> &[KeyBinding] { &[] }
    fn on_action(&self, _action: &str, _ctx: &AppContext) {}
}

// Bubbling implementation in app.rs:
fn dispatch_message(
    target: WidgetId,
    message: &dyn Any,
    ctx: &AppContext,
) -> EventPropagation {
    // Collect parent chain from target to root
    let mut chain = vec![target];
    let mut current = target;
    while let Some(&Some(parent)) = ctx.parent.get(current) {
        chain.push(parent);
        current = parent;
    }
    // Walk chain calling on_event
    for id in chain {
        if let Some(widget) = ctx.arena.get(id) {
            if widget.on_event(message, ctx) == EventPropagation::Stop {
                return EventPropagation::Stop;
            }
        }
    }
    EventPropagation::Continue
}
```

#### Key Bindings Structure

```rust
pub struct KeyBinding {
    pub key: crossterm::event::KeyCode,
    pub modifiers: crossterm::event::KeyModifiers,
    pub action: &'static str,
    pub description: &'static str,
    pub show: bool,  // whether to show in Footer
}
```

#### Timer/Interval Pattern (EVENT-08)

```rust
// Spawn in LocalSet — posts Tick events at fixed intervals
let tx_timer = tx.clone();
task::spawn_local(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        if tx_timer.send(AppEvent::Tick).is_err() { break; }
    }
});
```

The first `interval.tick()` completes immediately. Subsequent ticks fire every 100ms. Widgets opt in by handling `AppEvent::Tick` via a dedicated dispatch path or through the message system.

### Test Infrastructure

#### TestApp / Pilot Structure

```rust
// Source: D-11, D-12 from CONTEXT.md; pattern from textual/src/textual/pilot.py
use ratatui::backend::TestBackend;
use ratatui::Terminal;

pub struct TestApp {
    app: App,
    terminal: Terminal<TestBackend>,
    tx: flume::Sender<AppEvent>,
    rx: flume::Receiver<AppEvent>,
}

impl TestApp {
    pub fn new(factory: impl FnOnce() -> Box<dyn Widget> + 'static) -> Self {
        // Build App, attach TestBackend, init LocalSet + Executor
    }

    pub fn pilot(&mut self) -> Pilot<'_> {
        Pilot { test_app: self }
    }

    pub fn ctx(&self) -> &AppContext { &self.app.ctx }
    pub fn buffer(&self) -> &ratatui::buffer::Buffer {
        self.terminal.backend().buffer()
    }
}

pub struct Pilot<'a> {
    test_app: &'a mut TestApp,
}

impl<'a> Pilot<'a> {
    pub async fn press(&mut self, key: crossterm::event::KeyCode) -> &mut Self {
        self.test_app.tx.send(AppEvent::Key(/* ... */)).ok();
        self.settle().await;
        self
    }

    pub async fn type_text(&mut self, text: &str) -> &mut Self { /* ... */ self }
    pub async fn click(&mut self, col: u16, row: u16) -> &mut Self { /* ... */ self }

    pub async fn settle(&mut self) {
        // Drain flume + poll reactive effects until quiescent
        loop {
            tokio::task::yield_now().await;
            // process one event from rx if available
            // check if rx is empty AND no pending effects
            if self.test_app.rx.is_empty() { break; }
        }
    }
}
```

#### settle() Quiescence Algorithm

`settle()` must drain BOTH the flume event queue AND pending reactive Effects. The challenge is detecting when reactive effects are done — `Executor::poll_local()` from `any_spawner` polls the local executor once. The pattern:

```rust
pub async fn settle(&mut self) {
    loop {
        // Process all queued events
        while let Ok(event) = self.test_app.rx.try_recv() {
            self.test_app.process_event(event).await;
        }
        // Yield to let any reactive effects scheduled by those events fire
        tokio::task::yield_now().await;
        // If still empty after yield, we're quiescent
        if self.test_app.rx.is_empty() {
            break;
        }
    }
}
```

This may require 2-3 iterations since reactive Effects → RenderRequest → more events could chain.

#### Snapshot Testing Pattern (TEST-04)

```rust
// Source: https://ratatui.rs/recipes/testing/snapshots/ (verified)
#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    #[test]
    fn test_widget_snapshot() {
        let mut test_app = TestApp::new(|| Box::new(MyWidget::new()));
        // TestBackend implements Display — ratatui renders as text rows
        assert_snapshot!(test_app.terminal.backend());
    }
}
```

Snapshot files are stored in `tests/snapshots/` by default.
First run: creates `.snap` file. Changed UI: creates `.snap.new`, failing test.
Review workflow: `cargo insta review` (requires `cargo install cargo-insta`).

#### assert_buffer_lines Pattern (TEST-05)

```rust
// Source: ratatui 0.30.0 TestBackend docs — method exists natively
test_app.terminal.backend().assert_buffer_lines([
    "Button [OK]   ",
    "              ",
]);
```

Use `assert_buffer_lines` for specific widget output assertions. Use `assert_snapshot!` for full-screen visual regression.

#### proptest CSS Fuzzing Pattern (TEST-06)

```rust
// Source: proptest 1.11.0 docs
use proptest::prelude::*;

proptest! {
    #[test]
    fn css_parser_never_panics(input in ".*") {
        // Parser must not panic on arbitrary input
        let _ = Stylesheet::parse(&input);
    }

    #[test]
    fn css_valid_colors_parse(
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255
    ) {
        let css = format!("Widget {{ color: rgb({}, {}, {}); }}", r, g, b);
        let (sheet, errors) = Stylesheet::parse(&css);
        prop_assert!(errors.is_empty(), "Valid RGB should parse without errors");
    }
}
```

For CSS string fuzzing, use `prop::string::string_regex("[a-z#.: {}0-9%]+")` to generate plausible CSS-like strings.

### Anti-Patterns to Avoid

- **Hand-rolled dirty-flag signals:** Do not reimplement reactive tracking manually. reactive_graph's push-pull handles diamond problems and batching correctly.
- **Synchronous effect execution:** Effects are async — never assume they run immediately after `signal.set()`. Always `yield_now().await` in tests before asserting effect-driven state.
- **Arena-allocated `RwSignal` in widget structs:** Use `ArcRwSignal` instead. `RwSignal` requires an active Owner at creation time; `ArcRwSignal` is self-contained and cloneable into Effect closures.
- **`dyn Message` instead of `dyn Any`:** The decision (D-04) uses `&dyn Any` for the event parameter, enabling standard `downcast_ref::<ConcreteType>()`. Do not use `dyn Message` as it adds unnecessary vtable indirection.
- **Calling `Executor::init_tokio()` outside a LocalSet:** Effects that spawn async tasks will panic if called outside a LocalSet context. Always ensure the App is running inside `LocalSet::block_on`.
- **Blocking the LocalSet thread in tests:** `settle()` must be async. Never use `std::thread::sleep` or synchronous channel blocking in test harness code.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Reactive signals with dependency tracking | Custom dirty-flag or observer pattern | `reactive_graph::ArcRwSignal` + `ArcMemo` + `Effect` | Diamond problem, batching, cycle detection — all handled |
| Snapshot diffing for widget output | Custom buffer comparison | `insta::assert_snapshot!` with TestBackend | Generates human-readable diffs, `.snap` files, `cargo insta review` workflow |
| Property-based test input generation | Custom random string generators | `proptest!` macro + `prop::string::string_regex` | Shrinking on failure, coverage of edge cases |
| Async executor for non-Send futures | Custom LocalSet wrapper | `tokio::task::LocalSet` (already in use) | Already established pattern in codebase |

**Key insight:** reactive_graph solves the "glitch problem" — when multiple signals change, dependent computations run exactly once after all changes settle, not once per change. Hand-rolled observers always have this subtle bug.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield phase adding new code, not a rename/refactor/migration.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/Cargo | All compilation | ✓ | rustc 1.94.0 | — |
| tokio (rt + macros + time) | ReactiveGraph executor, timers | ✓ | 1 (in Cargo.toml) | — |
| reactive_graph 0.2.13 | REACT-01..05 | ✓ (resolves correctly) | 0.2.13 | hand-rolled signals (fallback not needed) |
| any_spawner 0.3.0 | reactive_graph Effects | ✓ (auto-resolved) | 0.3.0 | — |
| ratatui TestBackend | TEST-01..05 | ✓ (already in ratatui 0.30.0) | 0.30.0 | — |
| insta 1.46.3 | TEST-04 | ✗ (not in Cargo.toml yet) | — | manual string comparison |
| proptest 1.11.0 | TEST-06 | ✗ (not in Cargo.toml yet) | — | skip property tests |
| cargo-insta | Snapshot review workflow | ✗ (not installed on machine) | — | `INSTA_UPDATE=always` env var |

**Missing dependencies with no fallback:**
- None — all missing items have fallbacks or are easily installed via `cargo add`.

**Missing dependencies with fallback:**
- `insta`: Must be added as dev-dependency. Without it, TEST-04 cannot be implemented. Install: `cargo add insta --dev`
- `proptest`: Must be added as dev-dependency. Without it, TEST-06 cannot be implemented. Install: `cargo add proptest --dev`
- `cargo-insta`: CLI tool for reviewing snapshots. Not blocking — tests run fine without it; snapshots can be accepted via `INSTA_UPDATE=always cargo test`. Install: `cargo install cargo-insta`

---

## Common Pitfalls

### Pitfall 1: Effects Fire Asynchronously
**What goes wrong:** Widget sets a signal, immediately reads a derived value, gets stale data.
**Why it happens:** `Effect::new()` schedules the closure as a `spawn_local` task — it runs on the next async yield, not synchronously.
**How to avoid:** In production code, Effects are for side effects (posting RenderRequest, calling watch_ methods) — not for computing synchronous return values. Use `Memo::get()` for synchronous derived reads.
**Warning signs:** Test asserts state change after `signal.set()` without `yield_now().await` and sees stale value.

### Pitfall 2: Owner Dropped Too Early
**What goes wrong:** Effects stop firing, signals appear to not update.
**Why it happens:** `Owner::new()` must be kept alive (`let _owner = Owner::new()`) for the entire scope. If `_owner` is immediately dropped, all effects under it are cancelled.
**How to avoid:** Store `_owner` in `App` struct or a long-lived scope, not in a temporary.
**Warning signs:** Effects fire once (initial run) then never again.

### Pitfall 3: Executor::init_tokio Called Twice
**What goes wrong:** `Executor::init_tokio()` returns `Err(ExecutorError::AlreadySet)` and panics if unwrapped.
**Why it happens:** Called once globally; second call fails. In tests, multiple test functions run in the same process.
**How to avoid:** Always `let _ = Executor::init_tokio();` (ignore the error). The executor is set correctly on the first call; subsequent calls are no-ops.
**Warning signs:** Test suite passes individually but panics when run together.

### Pitfall 4: RwSignal vs ArcRwSignal
**What goes wrong:** `RwSignal::new(value)` panics with "tried to create a signal in a runtime that has been disposed."
**Why it happens:** `RwSignal` uses arena allocation tied to the current `Owner`. If no Owner is active at creation time, it panics.
**How to avoid:** Use `ArcRwSignal::new(value)` for signals stored in widget structs. Reserve arena-allocated `RwSignal` only for ephemeral use inside Effect closures where an Owner is guaranteed.
**Warning signs:** Panic during widget construction.

### Pitfall 5: settle() Not Draining Completely
**What goes wrong:** Test asserts snapshot too early; rendered buffer is from before the last event was processed.
**Why it happens:** Each event → Effect → RenderRequest chain requires multiple async yields to fully propagate.
**How to avoid:** `settle()` must loop: process events → yield → check empty → repeat until truly quiescent (see settle() algorithm above). A single `yield_now()` is not sufficient for chains.
**Warning signs:** Flaky tests that pass when run in isolation but fail intermittently.

### Pitfall 6: dyn Any Downcast Without Type Check
**What goes wrong:** `on_event` downcasts to wrong type, returns `Continue` silently, message never handled.
**Why it happens:** `event.downcast_ref::<Button::Pressed>()` returns `None` if message is a different type — this is correct behavior, but forgetting to check produces silent non-handling.
**How to avoid:** Standard pattern: `if let Some(msg) = event.downcast_ref::<Button::Pressed>() { ... }`. Document that widgets should return `Continue` for unrecognized messages.
**Warning signs:** Messages posted but handlers never fire.

### Pitfall 7: Mouse Events Without CrosstermFeature Flag
**What goes wrong:** Mouse events from crossterm never arrive.
**Why it happens:** Mouse event capture requires explicitly enabling it via `crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)`.
**How to avoid:** Add `EnableMouseCapture` to the terminal initialization sequence in `TerminalGuard::new()`. Add `DisableMouseCapture` to teardown.
**Warning signs:** Mouse hit testing works in tests but no events appear in production.

---

## Code Examples

### Complete Reactive Widget Pattern

```rust
// Source: reactive_graph 0.2.13 docs + spike verification 2026-03-25
use reactive_graph::{computed::ArcMemo, signal::ArcRwSignal, prelude::*};

pub struct Counter {
    count: Reactive<i32>,
}

impl Counter {
    pub fn new(initial: i32) -> Self {
        Self { count: Reactive::new(initial) }
    }

    // watch_ convention: called by Effect when count changes
    fn watch_count(&self, old: i32, new: i32) {
        println!("count changed: {} -> {}", old, new);
    }
}
```

### Executor Setup in App::run_async()

```rust
// Source: any_spawner 0.3.0 API + spike verification
use any_spawner::Executor;
use reactive_graph::owner::Owner;

async fn run_async(&mut self) -> Result<()> {
    // One-time global init (safe to call multiple times — ignores AlreadySet)
    let _ = Executor::init_tokio();
    // Keep owner alive for the duration of the app
    let _owner = Owner::new();

    // ... rest of setup ...
}
```

### on_event Downcast Pattern

```rust
// Source: Rust std::any::Any API
fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
    if let Some(msg) = event.downcast_ref::<button::Pressed>() {
        // handle it
        return EventPropagation::Stop;
    }
    EventPropagation::Continue
}
```

### insta Snapshot Test (Verified Pattern)

```rust
// Source: https://ratatui.rs/recipes/testing/snapshots/
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn snapshot_my_widget() {
    let mut test_app = TestApp::new(|| Box::new(MyWidget::default()));
    assert_snapshot!(test_app.terminal.backend());
    // Snapshot stored in tests/snapshots/snapshot_my_widget.snap
}
```

### proptest CSS Never-Panic Test

```rust
// Source: proptest 1.11.0 docs
use proptest::prelude::*;

proptest! {
    #[test]
    fn css_parser_never_panics(input in ".*") {
        let _ = crate::css::cascade::Stylesheet::parse(&input);
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual observer/dirty-flag signals | `reactive_graph` push-pull graph | Leptos v0.5+ (2023) | Eliminates diamond problem; effects run exactly once per tick |
| `tokio::LocalSet` (being reconsidered) | `LocalSet` still standard for !Send tasks | 2025 (proposal only) | LocalSet deprecation proposed but NOT happened; safe to use |
| Arena-allocated `RwSignal` | `ArcRwSignal` for widget-stored signals | reactive_graph 0.1→0.2 | Arc-based avoids Owner context requirement at construction |
| `leptos::create_runtime()` in tests | `Owner::new()` + `Executor::init_*()` | reactive_graph 0.2 | Cleaner separation; runtime and ownership are independent |

**Deprecated/outdated:**
- `create_runtime()` from older Leptos: replaced by `Owner::new()` in reactive_graph 0.2
- `Executor::init_futures_executor()` for tests: works but requires `futures-executor` feature; `init_tokio()` is preferred when tokio is already a dependency

---

## Open Questions

1. **settle() iteration bound**
   - What we know: Each event can produce new events via reactive effects; need to loop until quiescent
   - What's unclear: Maximum number of iterations needed before declaring quiescence; should there be a timeout/panic-on-infinite-loop?
   - Recommendation: Start with a fixed loop limit (e.g., 100 iterations) and panic with a diagnostic message if exceeded — catches deadlocks in test code

2. **on_event borrows: &AppContext vs &mut AppContext**
   - What we know: Phase 2 established `&self` + `&AppContext` for reads; mutations go through tree.rs functions taking `&mut AppContext`. The CONTEXT.md lists this as Claude's discretion.
   - What's unclear: Can `on_event` need to mutate AppContext directly (e.g., update focus, push screen)?
   - Recommendation: Take `&AppContext` for reads in `on_event`. Mutations are enqueued as secondary messages (e.g., `AppEvent::FocusWidget(id)`) dispatched by the App event loop. This avoids the double-borrow problem of having `arena[id]` borrowed while also mutating `ctx`.

3. **Mouse event types**
   - What we know: D-08 specifies click routing via MouseHitMap. Crossterm provides `MouseEvent` with `MouseEventKind` (Down, Up, Drag, ScrollDown, ScrollUp, Moved).
   - What's unclear: Phase 3 scope — just Click or also Scroll/Drag?
   - Recommendation: Implement Click (MouseDown + MouseUp pair) and ScrollUp/ScrollDown as separate message types. Drag and Hover are Phase 4+ concern.

4. **reactive_graph Owner lifecycle in TestApp**
   - What we know: `_owner = Owner::new()` must stay alive; Effects are cancelled when Owner drops.
   - What's unclear: Where to store Owner in TestApp — the App struct or TestApp wrapper?
   - Recommendation: Store in `App` struct directly alongside `AppContext`. This ensures Owner lives exactly as long as the App.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + tokio-rt (`#[tokio::test]`) |
| Config file | none (uses cargo's built-in test harness) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |
| Snapshot update command | `INSTA_UPDATE=always cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REACT-01 | Mutating Reactive<T> triggers re-render | unit | `cargo test --lib reactive` | ❌ Wave 0 |
| REACT-02 | watch_ method called on change | unit | `cargo test --lib reactive` | ❌ Wave 0 |
| REACT-03 | validate_ called on set | unit | `cargo test --lib reactive` | ❌ Wave 0 |
| REACT-04 | compute_ derives from sources | unit | `cargo test --lib reactive` | ❌ Wave 0 |
| REACT-05 | Multiple changes → one render pass per tick | unit | `cargo test --lib reactive` | ❌ Wave 0 |
| EVENT-01 | Typed messages implement Message trait | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-02 | on_ dispatch convention | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-03 | Event bubbling up parent chain | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-04 | EventPropagation::Stop prevents further bubbling | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-05 | Key events route to focused widget | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-06 | Mouse events route via hit map | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-07 | Key bindings trigger on_action | unit | `cargo test --lib event` | ❌ Wave 0 |
| EVENT-08 | Timer posts Tick at interval | unit | `cargo test --lib event` | ❌ Wave 0 |
| TEST-01 | TestApp runs headlessly | integration | `cargo test` | ❌ Wave 0 |
| TEST-02 | Pilot press/type_text/click/focus work | integration | `cargo test` | ❌ Wave 0 |
| TEST-03 | settle() drains before assertions | integration | `cargo test` | ❌ Wave 0 |
| TEST-04 | insta snapshot captures buffer | integration | `cargo test` | ❌ Wave 0 |
| TEST-05 | assert_buffer_lines cell assertions | integration | `cargo test` | ❌ Wave 0 |
| TEST-06 | proptest CSS parser never panics | unit | `cargo test --lib css` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib` (unit tests only, fast)
- **Per wave merge:** `cargo test` (all tests including integration)
- **Phase gate:** `cargo test` fully green before `/gsd:verify-work`

### Wave 0 Gaps
All test files for Phase 3 are new. The following must be created:
- [ ] `crates/textual-rs/src/reactive/mod.rs` — unit tests for Reactive<T> within module
- [ ] `crates/textual-rs/src/event/dispatch.rs` — unit tests for event dispatch + bubbling
- [ ] `crates/textual-rs/tests/test_app_harness.rs` — TestApp + Pilot integration tests
- [ ] `crates/textual-rs/tests/snapshots/` — directory created by insta on first snapshot run

Dev dependencies to add in Wave 0 (Plan 03-03):
```toml
[dev-dependencies]
insta = "1.46.3"
proptest = "1.11.0"
```

*(No framework gaps — cargo test is already the test runner. Wave 0 is adding new test files and dev-dependencies.)*

---

## Sources

### Primary (HIGH confidence)
- Spike verification — `reactive_graph 0.2.13` + `any_spawner 0.3.0` integration with Tokio LocalSet, run in-project 2026-03-25. Both tests PASS.
- `docs.rs/any_spawner/latest` — `Executor::init_tokio()` signature and `spawn_local` → `tokio::task::spawn_local` mapping confirmed
- `docs.rs/reactive_graph/0.2.13` — RwSignal, ArcRwSignal, Memo, ArcMemo, Effect, Owner APIs
- `docs.rs/ratatui/0.30.0/ratatui/backend/struct.TestBackend.html` — `assert_buffer_lines`, `buffer()` API
- `ratatui.rs/recipes/testing/snapshots/` — confirmed `assert_snapshot!(terminal.backend())` pattern
- `docs.rs/insta/latest` — `assert_snapshot!` macro, snapshot storage, `INSTA_UPDATE` env var
- `docs.rs/proptest/1.11.0` — `proptest!` macro, `prop::string::string_regex`, `prop_assert!`

### Secondary (MEDIUM confidence)
- `book.leptos.dev/appendix_reactive_graph.html` — reactive graph batching and push-pull mechanics explanation
- `github.com/leptos-rs/leptos/issues/3158` — confirmed `Executor::init_futures_executor()` + `Owner::new()` test pattern
- `github.com/tokio-rs/tokio/issues/6741` — LocalSet deprecation proposal (NOT yet deprecated; safe to use in 2025)

### Tertiary (LOW confidence, flagged)
- WebSearch results on `spawn_local` panic patterns — consistent with official docs, not independently verified with source code

---

## Metadata

**Confidence breakdown:**
- Spike verification: HIGH — ran in-process, both tests pass
- reactive_graph API: HIGH — verified against docs.rs 0.2.13
- Standard Stack versions: HIGH — verified against crates.io 2026-03-25
- Architecture patterns: HIGH — derived from locked decisions + verified APIs
- Pitfalls: HIGH for items 1-4 (verified during spike + docs), MEDIUM for items 5-7 (standard Rust patterns)
- Test infrastructure: HIGH — ratatui TestBackend pattern verified against official docs

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable ecosystem; reactive_graph 0.2.x is in active development but API is stable)
