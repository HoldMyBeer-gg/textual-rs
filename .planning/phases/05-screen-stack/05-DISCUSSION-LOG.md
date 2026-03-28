# Phase 5: Screen Stack - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 05-screen-stack
**Areas discussed:** Modal rendering, push_screen_wait, Demo shape, Integration tests

---

## Modal Rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Background visible | Render background screens bottom-up, modal on top; user sees underlying content | ✓ |
| Opaque replacement | Keep current behavior — modal fills full terminal, no background visible | |

**User's choice:** Background visible
**Notes:** Matches Python Textual's visual experience; background shown dimmed behind modal dialog.

---

## Background Dimming

| Option | Description | Selected |
|--------|-------------|----------|
| Dimmed background | Background cells darkened — draws attention to modal | ✓ |
| Full brightness | Background at normal colors | |

**User's choice:** Dimmed background

---

## push_screen_wait Inclusion

| Option | Description | Selected |
|--------|-------------|----------|
| Include in Phase 5 | Add typed async `push_screen_wait` / `pop_screen_with` API | ✓ |
| Keep as future (WIDGET-F02) | Scope stays tight to NAV-01/02/03 | |

**User's choice:** Include in Phase 5
**Notes:** Despite REQUIREMENTS.md listing it as future, user wants it shipped in this phase.

---

## push_screen_wait API Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Typed result via pop_with_value | `ctx.push_screen_wait(screen).await -> T`; modal calls `ctx.pop_screen_with(value)` | ✓ |
| String message / event | Modal emits message on dismiss; caller receives via on_event | |

**User's choice:** Typed result via pop_with_value

---

## Demo / Tutorial Shape

| Option | Description | Selected |
|--------|-------------|----------|
| tutorial_06_screens | Extends existing 01–05 sequence; push + modal + push_screen_wait | ✓ |
| Standalone screen_stack demo | Richer standalone demo like irc_demo | |

**User's choice:** tutorial_06_screens

---

## Integration Test Scope

| Option | Description | Selected |
|--------|-------------|----------|
| App-level tests via TestPilot | Full event loop, verify all 4 success criteria with rendered buffer assertions | ✓ |
| Unit tests only | Extend tree.rs unit tests | |

**User's choice:** App-level tests via TestPilot

---

## Claude's Discretion

- Exact dimming implementation (color blending vs. overlay style)
- How many background screens to render (all vs. N-1)
- Type parameter mechanism for push_screen_wait

## Deferred Ideas

- Screen transition animations
- push_screen_wait REQUIREMENTS.md traceability update (WIDGET-F02 → active)
