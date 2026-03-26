# textual-rs

## What This Is

A Rust library that ports the Textual Python TUI framework to Rust, delivering the same visual quality and developer ergonomics in a compiled, memory-safe language. The library provides a widget tree, CSS-like styling engine, reactive properties, async event loop, and rich built-in widgets — targeting developers who want beautiful terminal UIs without the Python runtime.

## Core Value

Developers can build Textual-quality TUI applications in Rust with the same ease: declare widgets, style with CSS, react to events, and get a polished result on any terminal.

## Requirements

### Validated

- [x] Cross-platform terminal backend (ratatui + crossterm) — Validated in Phase 1: Foundation
- [x] Async event loop with Tokio LocalSet — Validated in Phase 1: Foundation
- [x] Stable Rust (no nightly features required) — Validated in Phase 1: Foundation
- [x] Cross-platform: Windows 10+ confirmed (FOUND-01, FOUND-02) — Validated in Phase 1: Foundation
- [x] Reactive property system that triggers re-renders on state change — Validated in Phase 3: Reactive System, Events, and Testing
- [x] Async event loop with message passing between widgets — Validated in Phase 3: Reactive System, Events, and Testing
- [x] Keyboard and mouse input handling — Validated in Phase 3: Reactive System, Events, and Testing
- [x] Snapshot testing infrastructure for visual regression tests — Validated in Phase 3: Reactive System, Events, and Testing
- [x] Test pilot system for simulating user interaction in tests — Validated in Phase 3: Reactive System, Events, and Testing

### Active

- [ ] CSS-like styling system (TCSS-equivalent) for widget appearance and layout
- [ ] Widget tree with App > Screen > Widget hierarchy
- [ ] Layout engine: vertical, horizontal, grid, dock layouts
- [x] Built-in widget library: 22 widgets implemented — Validated in Phase 4: Built-in Widget Library
- [x] Scrollable containers (ScrollView, ListView, DataTable, Tree) — Validated in Phase 4: Built-in Widget Library
- [ ] Screen stack for modal dialogs and navigation
- [ ] Border styles, padding, margin (box model)
- [ ] Color themes and dark/light mode support
- [ ] Cross-platform: Windows 10+, macOS, Linux

### Out of Scope

- Web/WASM deployment target — focus on native terminals first
- Python bindings — pure Rust library
- Direct API compatibility with Python Textual — inspired by, not identical to

## Context

The Python Textual codebase (in `textual/` subdirectory) serves as the primary reference. Key architectural insights from the codebase map:

- **Textual's core pattern**: Hierarchical event-driven TUI with reactive data binding, CSS-based styling, and async message passing
- **CSS engine**: Custom TCSS parser with selector matching, cascade, computed styles — a significant subsystem
- **Render pipeline**: Compositor combines Strip-based widget renders into terminal output with dirty region tracking
- **Async design**: asyncio-based, every node (App/Screen/Widget) inherits MessagePump — maps naturally to Rust's tokio/async-std
- **Rich dependency**: Textual uses the `rich` Python library for ANSI/segment rendering — Rust has no equivalent; this must be built or an existing crate used
- **ratatui**: Popular Rust TUI crate. Must be evaluated — if it can serve as the rendering backend, build on top of it rather than reinventing the wheel

**Cross-platform requirements**: Must work on Windows 10+, macOS, and Linux. Terminal input/output handling differs significantly across platforms.

## Constraints

- **Language**: Rust — stable channel, no nightly-only features
- **Testing**: Unit tests written before implementation code (TDD approach)
- **Quality**: No shortcuts — correctness and safety over speed of development
- **Cross-platform**: Windows/macOS/Linux from day one — no platform-specific assumptions
- **Dependencies**: Prefer crates with broad adoption and maintenance; evaluate ratatui as rendering foundation before building from scratch

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Research ratatui before building | Popular, well-maintained — may eliminate need to build low-level terminal backend | ✓ Adopted: ratatui 0.30.0 + crossterm 0.29.0 (Phase 1) |
| TDD approach | User requirement: tests before code | ✓ Applied: TestBackend integration tests written TDD-style (Phase 1) |
| Async runtime choice (tokio vs async-std vs smol) | Event loop and message passing require async | ✓ Decided: tokio current_thread + LocalSet; avoids Send pressure on future widget state (Phase 1) |
| reactive_graph for reactive signals | Battle-tested signals from Leptos; ArcRwSignal/ArcMemo for widget state | ✓ Adopted: reactive_graph 0.2.13 + any_spawner 0.3.0; Executor::init_tokio() verified with LocalSet (Phase 3) |
| Event dispatch via on_event + dyn Any | Simple, extensible, no TypeId registry needed | ✓ Applied: Widget::on_event with downcast, parent-chain bubbling (Phase 3) |
| insta for snapshot testing | Human-readable diffs, ratatui TestBackend Display integration | ✓ Adopted: insta 1.42, plain-text buffer snapshots (Phase 3) |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition:**
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone:**
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
## Current State

Phase 4 (Built-in Widget Library) complete — all 22 v1 widgets implemented with keyboard interaction, snapshot tests, message emission, and scrolling. Input validation added via gap closure. 218 tests pass. Ready for Phase 5: Developer Experience and Polish.

*Last updated: 2026-03-26 after Phase 4: Built-in Widget Library*
