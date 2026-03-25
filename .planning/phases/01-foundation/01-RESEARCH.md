# Phase 1: Foundation - Research

**Researched:** 2026-03-24
**Domain:** Rust TUI — ratatui + crossterm + Tokio LocalSet + flume
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Multi-crate Cargo workspace from day one. Library lives at `crates/textual-rs/` (lib crate). Binary examples live in `crates/textual-rs/examples/`. No separate proc-macro crate in Phase 1.
- **D-02:** Workspace root `Cargo.toml` defines `[workspace]` with `members = ["crates/textual-rs"]`. A root-level `Cargo.lock` tracks all dependencies. Keep workspace flat — no nested workspaces.
- **D-03:** `cargo run` (via `examples/demo.rs`) renders a minimal styled box: bordered rectangle with title "textual-rs" and body text "Hello from textual-rs!". Uses ratatui's `Block` + `Paragraph` widgets. Centered in the terminal.
- **D-04:** Demo exits cleanly on `q` or `Ctrl+C`. No other interaction in Phase 1.
- **D-05:** On panic: first restore terminal (disable raw mode, show cursor, leave alt-screen via crossterm), then invoke the previous (default) panic hook. Use `std::panic::set_hook` with a closure. No panic info is suppressed.
- **D-06:** Terminal restoration on normal exit and `Ctrl+C` via a RAII guard (`TerminalGuard` struct with Drop impl). No `ctrlc` crate dependency — crossterm's EventStream handles `Ctrl+C` as a key event.
- **D-07:** `App::run()` is opaque — creates its own single-threaded Tokio runtime (`Builder::new_current_thread().enable_all().build()`) and LocalSet internally. No Tokio types in public API.
- **D-08:** `App` struct in Phase 1 is a skeleton: holds the terminal backend and nothing else. Its only method is `run()`.
- **D-09:** Inside `run()`, a `LocalSet` drives two concurrent tasks: (1) crossterm `EventStream` reader converting terminal events into flume messages; (2) main app loop receiving from flume and dispatching. Single flume channel (`flume::unbounded()`) carrying `AppEvent` enum (`Key(KeyEvent)`, `Resize(u16, u16)`, `Tick`).
- **D-10:** No tick timer in Phase 1. `Tick` variant reserved. Event loop blocks on flume receive with no timeout.
- **D-11:** Phase 1 includes a minimal `TestBackend` integration — single integration test that creates `App` with `TestBackend`, runs one render pass, asserts buffer contains "Hello from textual-rs!".

### Claude's Discretion

- Exact crate versions — use latest stable at time of implementation; pin in `Cargo.lock`
- Whether to split `run()` into `run_with_backend()` overload for testing — Claude decides based on ergonomics
- Error type for `run()` return — `anyhow::Error`, custom `AppError`, or `Box<dyn Error>` — Claude decides

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FOUND-01 | Library compiles on stable Rust (no nightly features) | Rust 1.94.0 stable confirmed on-machine; all recommended crates are stable-compatible |
| FOUND-02 | Cross-platform: Windows 10+, macOS, Linux (via crossterm backend) | crossterm 0.29 handles platform I/O abstraction; no platform branches needed in app code |
| FOUND-03 | ratatui-based rendering pipeline with async Tokio event loop | ratatui 0.30.0 + crossterm EventStream + tokio LocalSet pattern documented and verified |
| FOUND-04 | `App` struct as root entry point with `run()` method | Opaque `run()` using `LocalSet::block_on` is the correct pattern; verified via tokio docs |
| FOUND-05 | Alt-screen terminal management (enter/exit cleanly on panic) | crossterm `EnterAlternateScreen`/`LeaveAlternateScreen` + `set_hook` pattern fully verified |
| FOUND-06 | Terminal resize events trigger layout recomputation | crossterm `Event::Resize(cols, rows)` emitted by EventStream; handled in AppEvent::Resize |

</phase_requirements>

---

## Summary

Phase 1 is a greenfield infrastructure scaffolding task: stand up a Cargo workspace, wire ratatui + crossterm + Tokio + flume together into a working event loop, and prove the stack on all three platforms. The codebase is entirely empty — only the Python Textual source in `textual/` exists as reference.

The recommended stack is exactly what the decisions locked: ratatui 0.30.0 (crossterm backend), crossterm 0.29.0 (with `event-stream` feature), tokio 1.50.0 (`current_thread` runtime + LocalSet), and flume 0.12.0 for the event bus. All are on latest stable versions verified against crates.io as of 2026-03-24. Every crate is stable-Rust-compatible with no nightly features required.

ratatui 0.30.0 introduced a workspace split (ratatui-core, ratatui-crossterm, ratatui-widgets, ratatui-macros) and bumped MSRV to 1.86.0. The installed toolchain is Rust 1.94.0 (stable), so MSRV is satisfied. The project must use Cargo edition 2021 per the specifics in CONTEXT.md (not 2024, which ratatui itself adopts). The main `ratatui` umbrella crate re-exports everything, so you do not need to depend on sub-crates directly.

**Primary recommendation:** Use the main `ratatui` crate (not ratatui-crossterm/ratatui-core directly), pair it with `crossterm` 0.29.0 using the `crossterm_0_29` feature on ratatui, and drive the event loop with `LocalSet::block_on` on a `new_current_thread` runtime. Use `anyhow` for the error type — it eliminates boilerplate and aligns with common ratatui examples.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | Terminal UI rendering (Buffer, Frame, Widget, Layout, Terminal) | De-facto standard Rust TUI library; handles buffer diffing, Unicode, constraints |
| crossterm | 0.29.0 | Cross-platform terminal I/O, raw mode, alt-screen, event stream | ratatui's default backend; works on Windows/macOS/Linux without platform branches |
| tokio | 1.50.0 | Async runtime (single-threaded, LocalSet for !Send futures) | Project decision; LocalSet avoids `Send + 'static` pressure on future widget state |
| flume | 0.12.0 | MPMC channel bridging EventStream task and app loop | Sync + async dual API; simpler than tokio::sync::mpsc for the mixed sync/async case |
| anyhow | 1.0.102 | Error handling in `run()` and examples | Eliminates boilerplate; standard in Rust application code; not exposed in lib API |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures | 0.3.x | `StreamExt` for consuming crossterm `EventStream` | Required to call `.next().fuse()` on the event stream in a `tokio::select!` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| flume | tokio::sync::mpsc | tokio::sync::mpsc is mpsc-only; flume is MPMC, has a sync send API useful when bridging non-async code, and Phase 3 extends the channel — flume's flexibility is worth the extra dep |
| anyhow | Box\<dyn Error\> / custom AppError | `Box<dyn Error>` has no context-chaining; custom error type is premature in Phase 1. anyhow is idiomatic for app (not lib) error paths |
| LocalSet::block_on | tokio::main macro | `#[tokio::main]` leaks the multi-threaded scheduler into the binary; LocalSet::block_on on a current_thread runtime matches D-07 exactly and keeps Tokio types out of the public API |

**Installation:**
```toml
# workspace root Cargo.toml
[workspace]
members = ["crates/textual-rs"]
resolver = "2"

# crates/textual-rs/Cargo.toml
[package]
name = "textual-rs"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = { version = "0.30.0", features = ["crossterm_0_29"] }
crossterm = { version = "0.29.0", features = ["event-stream"] }
tokio = { version = "1.50.0", features = ["rt", "macros", "time"] }
flume = "0.12.0"
anyhow = "1.0"

[dev-dependencies]
futures = "0.3"

[[example]]
name = "demo"
path = "examples/demo.rs"
```

Note: `futures` is only needed in the EventStream consumer task. If you use `tokio-stream` instead, `futures` is not required. The `event-stream` feature on crossterm is required for `EventStream`; without it the type does not exist.

**Version verification (run before writing lock file):**
```bash
cargo search ratatui --limit 1    # 0.30.0 confirmed
cargo search crossterm --limit 1  # 0.29.0 confirmed
cargo search tokio --limit 1      # 1.50.0 confirmed
cargo search flume --limit 1      # 0.12.0 confirmed
cargo search anyhow --limit 1     # 1.0.102 confirmed
```

All versions confirmed against crates.io on 2026-03-24.

---

## Architecture Patterns

### Recommended Project Structure

```
textual-rs/               # workspace root
├── Cargo.toml            # [workspace] members = ["crates/textual-rs"]
├── Cargo.lock            # workspace-level lockfile
└── crates/
    └── textual-rs/
        ├── Cargo.toml    # lib crate
        ├── src/
        │   ├── lib.rs    # pub use app::App; pub use event::AppEvent;
        │   ├── app.rs    # App struct, run(), run_with_backend()
        │   ├── event.rs  # AppEvent enum, event task
        │   └── terminal.rs  # TerminalGuard, init_panic_hook()
        └── examples/
            └── demo.rs   # cargo run entry point
```

### Pattern 1: Opaque Single-Thread Runtime with LocalSet

**What:** `App::run()` creates a `new_current_thread` Tokio runtime internally, wraps it in a `LocalSet`, and blocks the calling thread until the event loop exits. No Tokio types appear in the public API.

**When to use:** Any time the library entry point must be `fn run(&mut self) -> Result<()>` with no async signature leaking to callers. Required by D-07.

```rust
// Source: https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html
use tokio::runtime::Builder;
use tokio::task::LocalSet;

pub fn run(&mut self) -> anyhow::Result<()> {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = LocalSet::new();
    local.block_on(&rt, self.run_async())
}

async fn run_async(&mut self) -> anyhow::Result<()> {
    // event loop lives here
    Ok(())
}
```

### Pattern 2: EventStream Reader Task + flume Channel

**What:** Spawn a `spawn_local` task that reads from `crossterm::event::EventStream`, converts events to `AppEvent`, and sends them over flume. Main loop receives with `recv_async().await`.

**When to use:** Decouples I/O from business logic; enables tokio::select! for future Tick timer integration (Phase 3).

```rust
// Source: https://ratatui.rs/tutorials/counter-async-app/async-event-stream/
use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::task;

let (tx, rx) = flume::unbounded::<AppEvent>();

// Spawn reader task on LocalSet (does not need to be Send)
task::spawn_local(async move {
    let mut stream = EventStream::new();
    loop {
        match stream.next().await {
            Some(Ok(crossterm::event::Event::Key(key))) => {
                let _ = tx.send(AppEvent::Key(key));
            }
            Some(Ok(crossterm::event::Event::Resize(cols, rows))) => {
                let _ = tx.send(AppEvent::Resize(cols, rows));
            }
            Some(Err(_)) | None => break,
            _ => {}
        }
    }
});

// Main loop
loop {
    match rx.recv_async().await? {
        AppEvent::Key(k) if k.code == KeyCode::Char('q') => break,
        AppEvent::Resize(cols, rows) => { /* trigger re-render */ }
        _ => {}
    }
    terminal.draw(|f| render(f))?;
}
```

### Pattern 3: RAII TerminalGuard

**What:** A struct that calls terminal cleanup in `Drop`. Ensures the terminal is always restored even if code returns early via `?`.

**When to use:** Normal exit and `Ctrl+C` handling (D-06).

```rust
// Source: crossterm docs + ratatui recipes
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use crossterm::cursor::{Hide, Show};
use std::io;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    }
}
```

### Pattern 4: Panic Hook with Terminal Restore

**What:** Capture previous panic hook, set a new one that restores the terminal first, then calls the original hook. Preserves the panic message and backtrace.

**When to use:** Must be called before entering raw mode / alt-screen (D-05).

```rust
// Source: https://ratatui.rs/recipes/apps/panic-hooks/
use std::panic;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::cursor::Show;
use crossterm::execute;
use std::io;

pub fn init_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Intentionally ignore errors — preserve the original panic message
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        original_hook(panic_info);
    }));
}
```

Critical: call `init_panic_hook()` before `TerminalGuard::new()`. The panic hook fires first, then Drop runs — both restore the same state, which is idempotent (calling `disable_raw_mode` when not in raw mode is a no-op on all platforms).

### Pattern 5: Centered Box with Block + Paragraph

**What:** Center a fixed-size box in the terminal using `Layout::vertical` + `Layout::horizontal` with `Constraint::Fill` padding.

**When to use:** The demo app (D-03).

```rust
// Source: https://docs.rs/ratatui/latest/ratatui/
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::text::Text;
use ratatui::Frame;

fn render(f: &mut Frame) {
    let area = f.area();

    // Center a 40x10 box
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Fill(1),
    ]).split(area);
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(40),
        Constraint::Fill(1),
    ]).split(vertical[1]);

    let block = Block::default()
        .title("textual-rs")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let para = Paragraph::new(Text::raw("Hello from textual-rs!"))
        .block(block)
        .centered();

    f.render_widget(para, horizontal[1]);
}
```

### Pattern 6: TestBackend Integration Test

**What:** Create a `Terminal<TestBackend>`, run one draw pass, inspect the buffer for expected text.

**When to use:** The D-11 integration test; avoids needing a real terminal in CI.

```rust
// Source: https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn test_render_hello() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    // Search the buffer for the expected string
    let content: String = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(content.contains("Hello from textual-rs!"));
}
```

Note: `Terminal::new` does not install a panic hook (unlike `ratatui::init()`). Call `init_panic_hook()` separately in production paths.

### Anti-Patterns to Avoid

- **Using `ratatui::init()` directly:** The 0.30 convenience function `ratatui::init()` sets up its own panic hook and owns the terminal. This conflicts with the RAII `TerminalGuard` pattern decided in D-06. Use the manual setup instead.
- **Spawning a `tokio::spawn` task for the EventStream:** `tokio::spawn` requires `Send`; `EventStream` is `Send + Sync`, but the `!Send` constraint emerges once widget state is attached in later phases. Use `task::spawn_local` inside the `LocalSet` from the start.
- **Calling `recv()` (sync) from inside async context:** Use `recv_async().await` in async code, never the sync `recv()`. Mixing them blocks the executor thread.
- **Double-raw-mode:** `enable_raw_mode()` called twice on Windows does not error but can cause subtle state corruption. The `TerminalGuard` drop + panic hook both call `disable_raw_mode()` — that's fine (idempotent), but do not call `enable_raw_mode()` from both the guard and elsewhere.
- **Writing to stdout vs stderr:** ratatui examples often use `stderr` to allow stdout piping. The decisions don't specify. Pick one and be consistent. `stdout` is simpler for a pure TUI app.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Buffer diffing for terminal output | Custom diff algorithm | ratatui `Terminal::draw()` | ratatui already does double-buffering and minimal-diff writes |
| Unicode width calculation | Manual `char::len_utf8` math | ratatui uses `unicode-width` internally | CJK characters are 2 cells wide; this is non-trivial |
| Constraint-based layout | Custom box-sizing math | ratatui `Layout` with `Constraint::*` | ratatui handles fractional units, min/max, fill |
| Alt-screen + raw mode state machine | Manual escape code sequences | crossterm `EnterAlternateScreen`, `enable_raw_mode()` | Platform-specific escape codes differ; crossterm handles Windows VT processing |
| Async event queue | Manual `BufReader` + thread + channel | crossterm `EventStream` | EventStream is pre-built, tested, handles all platform input quirks |
| Multi-producer channel | Roll your own with Mutex<VecDeque> | flume | flume handles all the lock-free queue complexity |

**Key insight:** The three hardest parts of TUI programming (Unicode, buffer diffing, platform input) are all solved by ratatui + crossterm. Phase 1's job is wiring them together, not reimplementing any of them.

---

## Common Pitfalls

### Pitfall 1: ratatui 0.30 MSRV is 1.86.0

**What goes wrong:** Cargo build fails with "package requires Rust 1.86.0" if the toolchain is older.
**Why it happens:** ratatui 0.30 adopted Rust 2024 edition internally and bumped MSRV to 1.86.0.
**How to avoid:** The on-machine toolchain is Rust 1.94.0 stable — this is satisfied. Document in `Cargo.toml` with `rust-version = "1.86"` at the workspace level to catch future CI regressions.
**Warning signs:** `error[E0554]` or MSRV conflict in `cargo build` output.

### Pitfall 2: crossterm feature flag version mismatch

**What goes wrong:** Two different versions of crossterm end up in the dependency graph; event queues desync, types are not interchangeable, raw mode may not be restored correctly on exit.
**Why it happens:** ratatui 0.30 uses `crossterm_0_29` as a feature gate internally. If app code also depends on crossterm directly, they must use the same major version.
**How to avoid:** Declare `crossterm = "0.29.0"` in your own `Cargo.toml` AND `ratatui = { version = "0.30.0", features = ["crossterm_0_29"] }`. This unifies the version.
**Warning signs:** `cargo tree` shows two versions of crossterm; compiler errors about type mismatches on `KeyEvent` or `Event`.

### Pitfall 3: EventStream requires `event-stream` feature flag

**What goes wrong:** `crossterm::event::EventStream` does not exist at compile time; you get `error[E0433]: failed to resolve: use of undeclared type`.
**Why it happens:** `EventStream` is behind a feature flag in crossterm. The flag is not enabled by default.
**How to avoid:** Add `features = ["event-stream"]` to the crossterm dependency in `Cargo.toml`.
**Warning signs:** Compiler error mentioning `EventStream` not found in `crossterm::event`.

### Pitfall 4: Panic hook must be set BEFORE entering raw mode

**What goes wrong:** If the process panics before the hook is set, the terminal is left in raw mode. The shell becomes unusable.
**Why it happens:** The panic hook captures the cleanup closure; if it was never set, cleanup never runs.
**How to avoid:** Call `init_panic_hook()` as the very first line of `main()` or `App::run()`, before `TerminalGuard::new()`.
**Warning signs:** Shell displays no echo after a panic during development.

### Pitfall 5: `Ctrl+C` is not automatically a signal on Windows

**What goes wrong:** On Unix, `Ctrl+C` sends SIGINT; on Windows it can behave differently depending on console mode. Using a signal handler (like the `ctrlc` crate) adds platform complexity.
**Why it happens:** Windows has no POSIX signals. `Ctrl+C` in raw mode is delivered as a key event `KeyCode::Char('c')` with `KeyModifiers::CONTROL` by crossterm.
**How to avoid:** D-06 and D-04 are aligned: handle `Ctrl+C` as a key event in the flume receive loop, same as `q`. The `TerminalGuard` Drop runs when the loop exits normally. No `ctrlc` crate needed.
**Warning signs:** On Windows, `Ctrl+C` terminates the process without running Drop — this means the `TerminalGuard` Drop may not fire. Mitigation: rely on the panic hook (which fires on abnormal exit paths) and ensure normal `Ctrl+C` is caught as a key event first.

### Pitfall 6: `task::spawn_local` requires an active LocalSet

**What goes wrong:** `task::spawn_local` panics at runtime with "no LocalSet available" if called outside a `LocalSet` context.
**Why it happens:** `spawn_local` requires the current thread to be running inside a `LocalSet::block_on` or `LocalSet::run_until`.
**How to avoid:** All `spawn_local` calls must happen inside the async block passed to `local.block_on(&rt, async { ... })`.
**Warning signs:** Runtime panic: "called `spawn_local` outside of a `task::LocalSet`".

### Pitfall 7: ratatui 0.30 removed `Block::title()` accepting `Title`

**What goes wrong:** Old examples using `Block::new().title(Title::from("text"))` fail to compile.
**Why it happens:** ratatui 0.30 breaking change: `Block::title()` now accepts `Into<Line>` directly.
**How to avoid:** Use `Block::default().title("textual-rs")` — string literals implement `Into<Line>` directly.
**Warning signs:** Compiler error: `the trait bound 'Title: Into<Line>' is not satisfied`.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield phase with no existing runtime state, rename, or refactor involved.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc (stable) | All compilation | YES | 1.94.0 | — |
| cargo | All builds | YES | 1.94.0 | — |
| Windows 10+ console (VT processing) | crossterm alt-screen | YES (Windows 11) | — | — |
| Internet / crates.io | Dependency download | YES (assumed) | — | Vendor crates if offline |

**Missing dependencies with no fallback:** None.

**Notes:** The machine is Windows 11 (`x86_64-pc-windows-msvc`). crossterm 0.29 handles Windows console Virtual Terminal Processing automatically via `enable_raw_mode()` — no manual `SetConsoleMode` calls needed in application code. macOS and Linux CI will need their own runners; this research verifies the Windows dev environment only.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | None — cargo test discovers `#[test]` and `#[cfg(test)]` automatically |
| Quick run command | `cargo test -p textual-rs` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FOUND-01 | Compiles on stable Rust | build check | `cargo build --workspace` | Wave 0 — no src yet |
| FOUND-02 | Cross-platform (Windows confirmed) | smoke | `cargo build` (manual on macOS/Linux) | Wave 0 |
| FOUND-03 | ratatui render pipeline + async loop | integration | `cargo test -p textual-rs test_render` | Wave 0 — ❌ |
| FOUND-04 | App::run() entry point | integration | `cargo test -p textual-rs test_app_run` | Wave 0 — ❌ |
| FOUND-05 | Alt-screen + panic restore | integration | `cargo test -p textual-rs test_terminal_guard` | Wave 0 — ❌ |
| FOUND-06 | Resize event triggers re-render | integration | `cargo test -p textual-rs test_resize` | Wave 0 — ❌ |

### Sampling Rate

- **Per task commit:** `cargo build --workspace` (compilation proof)
- **Per wave merge:** `cargo test -p textual-rs`
- **Phase gate:** Full test suite green (`cargo test --workspace`) before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `crates/textual-rs/src/` — entire lib crate (no Rust source exists yet)
- [ ] `crates/textual-rs/examples/demo.rs` — demo binary
- [ ] `crates/textual-rs/tests/integration_test.rs` — TestBackend render test (D-11)
- [ ] `Cargo.toml` (workspace root)
- [ ] `crates/textual-rs/Cargo.toml`

All code is created from scratch in Plans 01-01 and 01-02.

---

## Code Examples

### Minimal working Cargo workspace layout

```toml
# textual-rs/Cargo.toml (workspace root)
[workspace]
members = ["crates/textual-rs"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.86"
```

```toml
# textual-rs/crates/textual-rs/Cargo.toml
[package]
name = "textual-rs"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
ratatui = { version = "0.30.0", features = ["crossterm_0_29"] }
crossterm = { version = "0.29.0", features = ["event-stream"] }
tokio = { version = "1.50.0", features = ["rt", "macros", "time"] }
flume = "0.12.0"
anyhow = "1.0"

[dev-dependencies]
futures = "0.3"

[[example]]
name = "demo"
path = "examples/demo.rs"
```

### AppEvent enum (event.rs)

```rust
use crossterm::event::{KeyEvent};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,  // Reserved for Phase 3
}
```

### Full event loop skeleton (app.rs)

```rust
// Source: tokio LocalSet docs + ratatui counter-async-app tutorial
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use flume;
use futures::StreamExt;
use ratatui::{DefaultTerminal, Terminal};
use tokio::{runtime::Builder, task, task::LocalSet};

use crate::event::AppEvent;
use crate::terminal::TerminalGuard;

pub struct App;

impl App {
    pub fn run(&self) -> Result<()> {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()?;
        let local = LocalSet::new();
        local.block_on(&rt, self.run_async())
    }

    async fn run_async(&self) -> Result<()> {
        let _guard = TerminalGuard::new()?;
        let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;

        let (tx, rx) = flume::unbounded::<AppEvent>();

        task::spawn_local(async move {
            let mut stream = EventStream::new();
            while let Some(Ok(event)) = stream.next().await {
                match event {
                    Event::Key(k) => { let _ = tx.send(AppEvent::Key(k)); }
                    Event::Resize(c, r) => { let _ = tx.send(AppEvent::Resize(c, r)); }
                    _ => {}
                }
            }
        });

        loop {
            terminal.draw(|f| crate::ui::render(f))?;
            match rx.recv_async().await {
                Ok(AppEvent::Key(k)) if k.code == KeyCode::Char('q') => break,
                Ok(AppEvent::Key(k))
                    if k.code == KeyCode::Char('c')
                    && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break;
                }
                Ok(AppEvent::Resize(_, _)) => {} // re-render on next loop iteration
                _ => {}
            }
        }
        Ok(())
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ratatui::init()` + `restore()` manual setup | `ratatui::run(closure)` wraps both | 0.30.0 (Dec 2025) | Convenient for simple apps; conflicts with RAII guard — use manual setup |
| `Block::title(Title::from(...))` | `Block::default().title("text")` | 0.30.0 | Simpler string literal syntax |
| `Constraint::Percentage` + `Constraint::Length` only | + `Constraint::Fill(1)` for proportional space | 0.28.0 | Use `Fill` for centering instead of two `Percentage(50)` hacks |
| Single ratatui crate | ratatui workspace (ratatui-core, ratatui-crossterm, ratatui-widgets, ratatui-macros) | 0.30.0 | Main `ratatui` crate re-exports all; app code unaffected |

**Deprecated/outdated:**
- `tui` crate (predecessor to ratatui): Archived. Use `ratatui` exclusively.
- crossterm 0.27 and older: No longer supported by ratatui 0.30.
- `color_eyre` in examples: Many ratatui tutorials show `color_eyre`. Not required; `anyhow` is lighter and sufficient.

---

## Open Questions

1. **`run_with_backend()` overload for testing**
   - What we know: D-11 requires a `TestBackend` integration test; D-07 says `run()` creates its own runtime internally
   - What's unclear: Whether to expose `run_with_backend<B: Backend>(backend: B) -> Result<()>` as a separate public method or use `#[cfg(test)]` constructor
   - Recommendation: Expose `pub fn run_with_backend<B: ratatui::backend::Backend>(backend: B) -> Result<()>` as the testable overload; `run()` calls it with `CrosstermBackend`. This is the cleanest ergonomic split and avoids `#[cfg(test)]` hacks.

2. **`futures` dev-dependency vs `tokio-stream`**
   - What we know: `StreamExt` is needed to call `.next().await` on `EventStream`; both `futures` and `tokio-stream` provide it
   - What's unclear: `tokio-stream` is a heavier dep but already pulled in by tokio; `futures` is lighter
   - Recommendation: Use `futures` as a dev-dependency only (for the EventStream task), or import from `tokio_stream::StreamExt` if `tokio-stream` is already in tree. Either works; pick one and be consistent.

---

## Sources

### Primary (HIGH confidence)
- crates.io search — ratatui 0.30.0, crossterm 0.29.0, tokio 1.50.0, flume 0.12.0, anyhow 1.0.102 (verified 2026-03-24)
- [ratatui 0.30 highlights](https://ratatui.rs/highlights/v030/) — workspace split, breaking changes, new APIs
- [tokio LocalSet docs](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html) — `block_on`, `spawn_local`, `run_until` patterns
- [crossterm EventStream docs](https://docs.rs/crossterm/latest/crossterm/event/struct.EventStream.html) — feature flag, async usage
- [ratatui panic hooks recipe](https://ratatui.rs/recipes/apps/panic-hooks/) — `take_hook`/`set_hook` pattern

### Secondary (MEDIUM confidence)
- [ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — EventStream + tokio::select! pattern (verified against crossterm docs)
- [ratatui terminal and event handler recipe](https://ratatui.rs/recipes/apps/terminal-and-event-handler/) — TerminalGuard pattern, crossterm API list
- [ratatui-crossterm crate](https://docs.rs/crate/ratatui-crossterm/latest) — crossterm_0_29 feature flag documentation

### Tertiary (LOW confidence — validated by primary sources)
- [ratatui Terminal::new docs](https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html) — TestBackend usage example (partially confirmed; test example from docs)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified against crates.io live; all crates are stable-compatible
- Architecture patterns: HIGH — LocalSet pattern from official tokio docs; EventStream from official crossterm docs; panic hook from official ratatui recipes
- Pitfalls: HIGH — ratatui 0.30 breaking changes confirmed from official changelog; feature flag issues confirmed from crate docs
- TestBackend: MEDIUM — API confirmed from ratatui Terminal docs but full integration test pattern is inferred from partial docs

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable ecosystem; ratatui/crossterm are fast-moving — recheck if >30 days elapse)
