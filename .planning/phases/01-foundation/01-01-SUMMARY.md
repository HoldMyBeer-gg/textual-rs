---
phase: 01-foundation
plan: 01
subsystem: core-runtime
tags: [workspace, ratatui, crossterm, tokio, flume, event-loop, terminal-guard]
dependency_graph:
  requires: []
  provides: [App, AppEvent, TerminalGuard, init_panic_hook, demo-example]
  affects: [phase-02-widget-tree, phase-03-reactive]
tech_stack:
  added:
    - ratatui 0.30.0 (crossterm_0_29 feature)
    - crossterm 0.29.0 (event-stream feature)
    - tokio 1.x (rt, macros, time features)
    - flume 0.12
    - anyhow 1.0
    - futures 0.3
  patterns:
    - LocalSet::block_on on new_current_thread runtime (opaque App::run)
    - EventStream reader via spawn_local on LocalSet
    - flume::unbounded channel carrying AppEvent enum
    - RAII TerminalGuard (enable_raw_mode + EnterAlternateScreen on new, restore on Drop)
    - init_panic_hook capturing prior hook, restoring terminal before re-invoking
    - Layout::vertical + Layout::horizontal with Constraint::Fill for centered box
key_files:
  created:
    - Cargo.toml (workspace root)
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/src/event.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/terminal.rs
    - crates/textual-rs/examples/demo.rs
  modified: []
decisions:
  - key: futures-as-full-dep
    summary: "futures 0.3 is a full [dependencies] entry (not dev-only) because StreamExt is used in library code (app.rs EventStream consumer), not test code"
  - key: initial-render-before-loop
    summary: "terminal.draw() called once before the flume recv loop so the box is visible immediately without waiting for the first event"
  - key: resize-triggers-redraw
    summary: "AppEvent::Resize causes an immediate terminal.draw() call to handle terminal resize without requiring a Tick timer (deferred to Phase 3)"
metrics:
  duration_minutes: 2
  completed_date: "2026-03-25"
  tasks_completed: 2
  tasks_total: 2
  files_created: 7
  files_modified: 2
---

# Phase 01 Plan 01: Cargo Workspace Foundation Summary

**One-liner:** Cargo workspace with ratatui 0.30 + crossterm 0.29 + Tokio LocalSet + flume event loop, RAII terminal guard with panic hook, and a runnable demo rendering a centered rounded-border box.

## What Was Built

A greenfield Rust workspace (`textual-rs`) scaffolded from nothing. The workspace contains a single lib crate at `crates/textual-rs/` with five source files and one demo example. The full ratatui + crossterm + Tokio LocalSet + flume stack is wired end-to-end and proven to compile on stable Rust 1.94.0.

## Tasks Completed

| Task | Name | Commit | Files Created/Modified |
|------|------|--------|------------------------|
| 1 | Create Cargo workspace and lib crate | 4e656e7 | Cargo.toml, crates/textual-rs/Cargo.toml, src/lib.rs, src/event.rs, src/app.rs (placeholder), src/terminal.rs (placeholder) |
| 2 | Implement App::run(), TerminalGuard, event loop, demo | 9f1689a | src/app.rs (full), src/terminal.rs (full), examples/demo.rs |

## Architecture Decisions

**futures as full dependency (not dev-only)**

The plan noted that `futures` is needed as a full `[dependencies]` entry because `StreamExt` is consumed in `app.rs` library code (the `EventStream` reader task inside `run_async()`), not in test code. The RESEARCH.md suggested dev-dependency but that only works if EventStream consumption is confined to test code.

**Initial render before the event loop**

`terminal.draw()` is called once before entering the flume `recv_async()` loop. This ensures the TUI box is visible immediately without requiring a key press or resize event to trigger the first frame.

**Resize triggers immediate redraw**

`AppEvent::Resize` causes a synchronous `terminal.draw()` call within the event loop. This handles terminal resizing cleanly before the Tick timer is added in Phase 3.

## Verification

All must-haves confirmed:

- `cargo build --workspace` exits 0 on stable Rust 1.94.0 (no nightly features)
- `cargo build --example demo -p textual-rs` exits 0
- `Cargo.toml` contains `[workspace]`, `members = ["crates/textual-rs"]`, `resolver = "2"`
- `workspace.package` sets `edition = "2021"`, `rust-version = "1.86"`
- `crates/textual-rs/Cargo.toml` contains `ratatui` with `crossterm_0_29` feature
- `crates/textual-rs/Cargo.toml` contains `crossterm` with `event-stream` feature
- `event.rs` exports `pub enum AppEvent` with `Key(KeyEvent)`, `Resize(u16, u16)`, `Tick`
- `lib.rs` exports `pub use app::App` and `pub use event::AppEvent`
- `terminal.rs` exports `pub fn init_panic_hook()`, `pub struct TerminalGuard`, `impl Drop for TerminalGuard`
- `app.rs` exports `pub struct App` with `pub fn run(&self) -> Result<()>`
- `app.rs` contains `Builder::new_current_thread()`, `LocalSet::new()`, `local.block_on`, `flume::unbounded`, `EventStream::new()`, `spawn_local`, `KeyCode::Char('q')`, `BorderType::Rounded`, `"Hello from textual-rs!"`
- `demo.rs` contains `App::new()` and `.run()`

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all functionality is fully wired. The `Tick` variant in `AppEvent` is intentionally reserved for Phase 3 and documented as such in comments.

## Integration Points for Future Phases

- **Phase 2** attaches the widget tree to `App` (currently a unit struct skeleton)
- **Phase 3** extends the flume channel with typed message passing and adds the Tick timer using `tokio::time::interval` inside `run_async()`
- **Phase 3** builds `TestApp` / `Pilot` test harness on top of the `TestBackend` integration pattern identified in D-11 (integration test deferred to Plan 01-02 per RESEARCH.md Wave 0 gap tracking)

## Self-Check: PASSED

Files verified present:
- Cargo.toml: FOUND
- crates/textual-rs/Cargo.toml: FOUND
- crates/textual-rs/src/lib.rs: FOUND
- crates/textual-rs/src/event.rs: FOUND
- crates/textual-rs/src/app.rs: FOUND
- crates/textual-rs/src/terminal.rs: FOUND
- crates/textual-rs/examples/demo.rs: FOUND

Commits verified:
- 4e656e7: FOUND (feat(01-01): create Cargo workspace and lib crate)
- 9f1689a: FOUND (feat(01-01): implement App::run(), TerminalGuard, event loop, and demo example)
