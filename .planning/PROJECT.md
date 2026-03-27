# textual-rs

## What This Is

A Rust TUI framework inspired by Python Textual, delivering modern terminal interfaces with CSS styling, reactive state, and 25+ built-in widgets. Includes semantic theming, sub-cell rendering, animation, clipboard support, and a headless test harness.

## Core Value

Developers can build Textual-quality TUI applications in Rust with the same ease: declare widgets, style with CSS, react to events, and get a polished result on any terminal.

## Requirements

### Validated

- [x] Cross-platform terminal backend (ratatui + crossterm) — v1.0
- [x] Async event loop with Tokio LocalSet — v1.0
- [x] Stable Rust (no nightly features required) — v1.0
- [x] Cross-platform: Windows 10+ — v1.0
- [x] Reactive property system that triggers re-renders on state change — v1.0
- [x] Async event loop with message passing between widgets — v1.0
- [x] Keyboard and mouse input handling — v1.0
- [x] Snapshot testing infrastructure for visual regression tests — v1.0
- [x] Test pilot system for simulating user interaction in tests — v1.0
- [x] CSS-like styling system (TCSS-equivalent) for widget appearance and layout — v1.0
- [x] Widget tree with App > Screen > Widget hierarchy — v1.0
- [x] Layout engine: vertical, horizontal, dock layouts — v1.0
- [x] Built-in widget library: 25+ widgets — v1.0/v1.2
- [x] Scrollable containers (ScrollView, ListView, DataTable, Tree) — v1.0
- [x] Border styles, padding, margin (box model) — v1.0
- [x] Derive macros (#[derive(Widget)], #[widget_impl]) — v1.0
- [x] Worker API for background tasks — v1.0
- [x] Command palette (Ctrl+P) — v1.0
- [x] Demo apps and tutorial examples — v1.0
- [x] Semantic color theming ($primary, $surface, $accent + shade generation) — v1.1
- [x] Sub-cell rendering (half-block, eighth-block, quadrant, braille) — v1.1
- [x] Interactive states (focus, hover, pressed, selected, invalid) — v1.1
- [x] 7 built-in themes with runtime Ctrl+T switching — v1.2
- [x] Clipboard integration (Ctrl+C/V/X) — v1.2
- [x] Text selection in Input and TextArea — v1.2
- [x] Animation system (Switch toggle, Tab underline) — v1.2
- [x] Terminal capability detection — v1.2
- [x] Mouse capture push/pop stack + Shift bypass — v1.2

### Active

- [ ] Full Python Textual widget parity: ContentSwitcher, Digits, DirectoryTree, Link, LoadingIndicator, MaskedInput, OptionList, Pretty, RichLog, Rule, SelectionList, Static, Toast
- [ ] Screen stack for modal dialogs and navigation
- [ ] Cross-platform verification: macOS, Linux (Windows confirmed)
- [ ] crates.io publish

## Current Milestone: v1.3 Widget Parity & Ship

**Goal:** Achieve full Python Textual widget parity, add screen stack navigation, verify cross-platform, and publish to crates.io.

**Target features:**
- 13 missing widgets matching Python Textual docs/screenshots at https://textual.textualize.io/widgets/
- Screen stack for modal dialogs and navigation
- Cross-platform verification (macOS, Linux)
- crates.io publish

### Out of Scope

- Web/WASM deployment target — focus on native terminals first
- Python bindings — pure Rust library
- Direct API compatibility with Python Textual — inspired by, not identical to
- Accessibility / screen reader support — future consideration

## Context

Shipped through v1.2 with ~17,400 LOC Rust across 25+ widgets, CSS engine, reactive system, animation, theming, and test harness. Tech stack: ratatui 0.30, crossterm 0.29, tokio, reactive_graph, taffy, arboard.

## Constraints

- **Language**: Rust — stable channel, no nightly-only features
- **Testing**: TDD approach with insta snapshot tests
- **Quality**: Correctness and safety over speed of development
- **Cross-platform**: Windows/macOS/Linux
- **Dependencies**: Prefer well-maintained crates; ratatui as rendering foundation

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| ratatui + crossterm backend | Popular, well-maintained | Adopted v1.0 |
| tokio current_thread + LocalSet | Avoids Send pressure on widget state | Adopted v1.0 |
| reactive_graph signals | Battle-tested from Leptos | Adopted v1.0 |
| on_event + dyn Any dispatch | Simple, extensible | Adopted v1.0 |
| insta snapshot testing | Human-readable diffs | Adopted v1.0 |
| Pure-math HSL for shade generation | No external crate needed | Adopted v1.1 |
| Two-phase CSS variable resolution | Parse as Variable, resolve at cascade | Adopted v1.1 |
| McGugan Box with U+2595 fallback | Broad terminal compatibility | Adopted v1.2 |
| active_overlay for Select/CommandPalette | Avoids screen blanking | Adopted v1.2 |
| arboard for clipboard | Cross-platform clipboard access | Adopted v1.2 |
| MouseCaptureStack push/pop | Prevents competing enable/disable clobber | Adopted v1.2 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-26 after v1.3 milestone started*
