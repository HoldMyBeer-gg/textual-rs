# Phase 5: Developer Experience and Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-26
**Phase:** 05-developer-experience-and-polish
**Areas discussed:** Derive macro scope, Worker API design, Demo & documentation, Command palette scope

---

## Derive Macro Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal | Just generates `widget_type_name()` from struct name | |
| Standard | + Cell<WidgetId> wiring, `#[focusable]` attribute | |
| Full | + `#[on(Msg)]` dispatch, `#[keybinding]` routing — closest to Textual's Python decorator style | x |

**User's choice:** Full
**Notes:** None — clear preference for maximum boilerplate reduction.

---

## Worker API Design

| Option | Description | Selected |
|--------|-------------|----------|
| Message-based | Results delivered as typed messages through existing queue | x |
| Callback-based | Closure runs on completion | |
| Channel-based | Returns Receiver<T> widget polls | |

**User's choice:** Message-based
**Notes:** Consistent with existing event system.

### Follow-up: Cancellation

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-cancel on unmount | Workers tied to widget lifetime, dropped on unmount | x |
| Manual cancel only | Run to completion unless explicitly cancelled | |
| Both | Auto-cancel default + `run_worker_detached()` for fire-and-forget | |

**User's choice:** Auto-cancel on unmount
**Notes:** None.

---

## Demo & Documentation

| Option | Description | Selected |
|--------|-------------|----------|
| Widget showcase | Tabbed catalogue of all widgets | |
| IRC client demo | Updated irc_demo.rs with built-in widgets | |
| Both | Showcase as demo.rs + updated IRC client as irc_demo.rs | x |

**User's choice:** Both — "please make them beautiful. maybe check lazeport.pwn.zone's css"
**Notes:** User wants demos styled after lazeport.pwn.zone aesthetic — deep dark backgrounds, green (#00ffa3) and cyan (#00d4ff) accents. Terminal-adapted version of the site's color palette.

### Follow-up: Documentation format

| Option | Description | Selected |
|--------|-------------|----------|
| Rustdoc only | Doc comments, cargo doc generates reference | |
| Rustdoc + mdbook | Separate guide site | |
| Rustdoc + examples-as-docs | Tutorial example files serve as getting-started guide | x |

**User's choice:** Rustdoc + examples-as-docs
**Notes:** None.

---

## Command Palette Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full palette widget | Auto-discovers from key_bindings, fuzzy search overlay | |
| Action registry only | No visual widget, just programmatic dispatch | |
| Full palette + app commands | Palette + app.register_command() for custom commands | x |

**User's choice:** Full palette with app commands
**Notes:** None.

---

## Claude's Discretion

- Proc-macro implementation details (syn/quote)
- Worker cancellation mechanism
- Fuzzy matching algorithm
- Tutorial count and progression
- Widget showcase organization

## Deferred Ideas

None — discussion stayed within phase scope
