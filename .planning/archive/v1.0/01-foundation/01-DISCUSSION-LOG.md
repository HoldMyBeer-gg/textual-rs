# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 01-foundation
**Mode:** Auto (`--auto` flag — all choices made by Claude using recommended defaults)
**Areas discussed:** Workspace Structure, Smoke Test Content, Panic Hook Behavior, App::run() API Surface

---

## Workspace Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Single crate | Everything in one `textual-rs/` crate at root | |
| Multi-crate workspace | Library at `crates/textual-rs/`, examples in `examples/` | ✓ |
| Multi-crate with separate bin | Separate `textual-demo` binary crate in workspace | |

**Auto-selected:** Multi-crate workspace with `crates/textual-rs/` lib crate and examples in `crates/textual-rs/examples/`
**Rationale:** Keeps library and examples clearly separated from Phase 1, supports future proc-macro crate (Phase 5) without restructuring. Standard practice for Rust library crates.

---

## Smoke Test Content

| Option | Description | Selected |
|--------|-------------|----------|
| Plain text | Just "Hello from textual-rs!" with no styling | |
| Styled box | Bordered rectangle with title and body text | ✓ |
| Interactive demo | Multiple widgets with navigation | |

**Auto-selected:** Minimal styled box — `Block` with title "textual-rs" + `Paragraph` with "Hello from textual-rs!", centered
**Rationale:** Proves ratatui rendering, borders, and text layout. Sets visual quality bar. Simple enough to implement quickly, meaningful enough to validate the stack.

---

## Panic Hook Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Restore only | Restore terminal, suppress panic output | |
| Restore then default hook | Restore terminal, then let default panic handler print to stderr | ✓ |
| Custom panic display | Restore terminal, format and print panic info in a custom way | |

**Auto-selected:** Restore terminal first, then invoke the previous default panic hook
**Rationale:** Standard crossterm pattern. No panic info is lost. Works correctly on all platforms (no `signal` crate needed). Use `std::panic::set_hook` with closure capturing prior hook.

---

## App::run() API Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Opaque runtime | `run()` creates its own Tokio runtime internally | ✓ |
| Caller-supplied runtime | `run()` accepts a `&mut Runtime` or `Handle` | |
| `#[tokio::main]` expectation | Caller must be inside a Tokio context | |

**Auto-selected:** Opaque — `App::run()` creates single-threaded Tokio runtime + LocalSet internally
**Rationale:** Matches Textual's pattern (users just call `.run()`). Tokio types don't leak into public API. Can always expose runtime customization as a later addition.

---

## Claude's Discretion

- Exact crate versions (pinned in Cargo.lock at implementation time)
- Whether to expose `run_with_backend()` for testing ergonomics
- Error type for `run()` return value

## Deferred Ideas

None during this session.
