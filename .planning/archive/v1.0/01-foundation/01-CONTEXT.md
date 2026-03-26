# Phase 1: Foundation - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove the full ratatui + crossterm + Tokio LocalSet stack works end-to-end: a runnable Cargo workspace where `cargo run` opens an alternate-screen TUI, renders visible content, handles keyboard input, exits cleanly on `q` or panic, and responds to terminal resize. This is infrastructure scaffolding only — no widget tree, no CSS, no reactive system.

</domain>

<decisions>
## Implementation Decisions

### Workspace Structure
- **D-01:** Multi-crate Cargo workspace from day one. Library lives at `crates/textual-rs/` (lib crate). Binary examples live in `crates/textual-rs/examples/`. No separate proc-macro crate in Phase 1 — that comes in Phase 5.
- **D-02:** Workspace root `Cargo.toml` defines `[workspace]` with `members = ["crates/textual-rs"]`. A root-level `Cargo.lock` tracks all dependencies. Keep workspace flat — no nested workspaces.

### Smoke Test / Demo App
- **D-03:** `cargo run` (via `examples/demo.rs`) renders a minimal styled box: a bordered rectangle with title "textual-rs" and body text "Hello from textual-rs!". Uses ratatui's `Block` + `Paragraph` widgets. Centered in the terminal. This proves ratatui rendering, borders, text layout, and the event loop in one visible artifact.
- **D-04:** Demo exits cleanly on `q` or `Ctrl+C`. No other interaction in Phase 1.

### Panic Hook
- **D-05:** On panic: first restore terminal (disable raw mode, show cursor, leave alt-screen via crossterm), then invoke the previous (default) panic hook so the panic message + backtrace prints to stderr normally. Use `std::panic::set_hook` with a closure that calls `crossterm::terminal::disable_raw_mode()`, `crossterm::execute!(io::stderr(), LeaveAlternateScreen, Show)`, then calls the prior hook. No panic info is suppressed.
- **D-06:** Terminal restoration must also happen on normal exit and on `Ctrl+C` (SIGINT). Use a RAII guard (Drop impl) or `ctrlc` crate — prefer RAII `TerminalGuard` struct that restores on drop. No `ctrlc` crate dependency needed if crossterm's EventStream handles SIGINT.

### App::run() API Surface
- **D-07:** `App::run()` is opaque — it creates its own single-threaded Tokio runtime (`Builder::new_current_thread().enable_all().build()`) and LocalSet internally. The caller just calls `app.run()` and blocks. No Tokio types leak into the public API in Phase 1.
- **D-08:** `App` struct in Phase 1 is a skeleton: holds the terminal backend and nothing else. Its only method is `run()`. This is intentionally minimal — Phase 2 replaces it with the full widget tree root.

### Event Loop Architecture
- **D-09:** Inside `run()`, a `LocalSet` drives two concurrent tasks: (1) a crossterm `EventStream` reader that converts terminal events (key, resize) into flume messages, and (2) the main app loop that receives from flume and dispatches. Single flume channel (`flume::unbounded()`) carrying an `AppEvent` enum (`Key(KeyEvent)`, `Resize(u16, u16)`, `Tick`).
- **D-10:** No tick timer in Phase 1 — the `Tick` variant is reserved for future phases. Event loop blocks on flume receive with no timeout.

### Testing in Phase 1
- **D-11:** Phase 1 includes a minimal `TestBackend` integration — enough to verify that the render pipeline produces output without a real terminal. A single integration test that creates `App` with `TestBackend`, runs one render pass, and asserts the buffer contains "Hello from textual-rs!". Full `TestApp`/`Pilot` harness is Phase 3.

### Claude's Discretion
- Exact crate versions — use latest stable at time of implementation; pin in `Cargo.lock`
- Whether to split `run()` into `run_with_backend()` overload for testing — Claude decides based on ergonomics
- Error type for `run()` return — `anyhow::Error`, custom `AppError`, or `Box<dyn Error>` — Claude decides

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python Textual Reference (architecture inspiration)
- `textual/src/textual/app.py` — App entry point, run() method, driver initialization pattern
- `textual/src/textual/drivers/` — Driver abstraction: how Python Textual handles platform I/O; crossterm replaces this in Rust
- `textual/src/textual/_compositor.py` — Render pipeline: how strip-based rendering feeds the terminal; ratatui replaces this

### Project Planning Documents
- `.planning/REQUIREMENTS.md` — Full v1 requirements; Phase 1 covers FOUND-01 through FOUND-06
- `.planning/ROADMAP.md` — Phase 1 plan details (01-01, 01-02) and success criteria
- `.planning/PROJECT.md` — Key decisions table (ratatui, crossterm, Tokio LocalSet, flume, slotmap, Taffy)

### Codebase Analysis
- `.planning/codebase/ARCHITECTURE.md` — Python Textual architecture: event loop, message pump, driver layer
- `.planning/codebase/STACK.md` — Python Textual stack: what ratatui + crossterm + Tokio replaces

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield Rust project. The `textual/` directory contains the Python Textual source as a reference, not reusable Rust code.

### Established Patterns
- Python Textual's `App.run()` → `asyncio.run(App._main())` maps to Rust's `App::run()` → `Runtime::block_on(LocalSet::run_until(app_main()))`. Same opaque-entry-point pattern.
- Python Textual uses a driver abstraction with platform-specific impls (Windows/Unix). Crossterm eliminates this need — single cross-platform implementation.
- Python Textual's `MessagePump._process_messages_loop()` maps to the flume receive loop. Start simple, extend in Phase 3.

### Integration Points
- Phase 1 output (`App` struct, `run()`, `AppEvent`, `TerminalGuard`) becomes the base that Phase 2 attaches the widget tree to.
- The flume channel established in Phase 1 is the same channel Phase 3 extends with typed message passing.
- The `TestBackend` integration in Phase 1 is the foundation Phase 3's `TestApp` harness builds on.

</code_context>

<specifics>
## Specific Ideas

- The demo app should look like a Textual-quality output even though it's hard-coded — set the tone for what the library will produce.
- Panic hook must be rock-solid on Windows (no `signal` crate, use crossterm's portable cleanup APIs only).
- `Cargo.toml` edition should be 2021 (stable, well-supported). No edition 2024 yet.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-24*
