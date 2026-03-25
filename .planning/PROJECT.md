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

### Active

- [ ] CSS-like styling system (TCSS-equivalent) for widget appearance and layout
- [ ] Widget tree with App > Screen > Widget hierarchy
- [ ] Reactive property system that triggers re-renders on state change
- [ ] Async event loop with message passing between widgets
- [ ] Layout engine: vertical, horizontal, grid, dock layouts
- [ ] Built-in widget library: Button, Input, Label, Checkbox, Select, DataTable, TextArea, ListView, Tree, Tabs, ProgressBar, Sparkline, Log, Markdown, Switch, RadioButton, Collapsible, TabbedContent, ContentSwitcher, Footer, Header
- [ ] Keyboard and mouse input handling
- [ ] Scrollable containers with scrollbar widgets
- [ ] Screen stack for modal dialogs and navigation
- [ ] Border styles, padding, margin (box model)
- [ ] Color themes and dark/light mode support
- [ ] Snapshot testing infrastructure for visual regression tests
- [ ] Test pilot system for simulating user interaction in tests
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

Phase 1 (Foundation) complete — Cargo workspace scaffolded, ratatui + crossterm + Tokio LocalSet event loop wired end-to-end, RAII terminal guard + panic hook implemented, 5 integration tests pass. Ready for Phase 2: Widget Tree, Layout, and Styling.

*Last updated: 2026-03-24 after Phase 1: Foundation*
