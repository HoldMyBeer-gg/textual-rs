# Phase 5: Screen Stack - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver complete screen stack navigation: push/pop screens with focus save/restore, modal screens that block input to screens below and render as overlays on the background, and a typed async `push_screen_wait` API for modals that return a value. Includes a tutorial example and App-level integration tests against all 4 success criteria.

</domain>

<decisions>
## Implementation Decisions

### Modal Rendering
- **D-01:** Background screens are rendered first (bottom-up through the stack), then the modal renders on top — the user sees the underlying app content behind the dialog. This matches Python Textual's visual experience. The `full_render_pass` in `app.rs` must render all screens in the stack in order, not just the top one.
- **D-02:** Background screens are visually dimmed (darkened) behind the modal. Draws attention to the modal and matches Python Textual's default `Screen { background: $background 60%; }` behavior.

### push_screen_wait — Async Modal Result
- **D-03:** Include `push_screen_wait` in Phase 5 scope (despite REQUIREMENTS.md listing it as WIDGET-F02 future). API: `ctx.push_screen_wait(screen).await` returns `T`; modal calls `ctx.pop_screen_with(value: T)` to dismiss and deliver the result. Typed, no string/event indirection — matches Python Textual's `push_screen` + `dismiss()` pattern.

### Demo / Tutorial
- **D-04:** `tutorial_06_screens` — fits the existing sequence (01–05). Demonstrates: push a named navigation screen, push a `ModalScreen` confirm dialog, receive OK/Cancel result via `push_screen_wait`, pop back and display the result on the main screen.

### Integration Tests
- **D-05:** App-level integration tests via `TestPilot` (not just tree.rs unit tests). Tests drive push/pop through the full App event loop. Must verify all 4 success criteria:
  1. `push_screen_deferred()` scopes focus to the new screen
  2. `pop_screen_deferred()` restores focus to the exact prior widget
  3. A `ModalScreen` blocks keyboard/mouse input to screens below
  4. After pop, the screen below repaints cleanly (no artifacts)

### Claude's Discretion
- Exact dimming implementation for background screens (e.g., alpha-blend cell colors toward black, or apply a dark overlay style to the rendered buffer)
- Number of background screens to render (render all in stack vs. only screen N-1 when modal is on top)
- Exact type parameter mechanism for `push_screen_wait<T>` (generic on App, oneshot channel, or other)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing screen stack implementation
- `crates/textual-rs/src/widget/tree.rs` — `push_screen()`, `pop_screen()`, `advance_focus()` implementations and unit tests (lines ~160–700)
- `crates/textual-rs/src/widget/screen.rs` — `ModalScreen` struct with `is_modal() -> true`
- `crates/textual-rs/src/widget/context.rs` — `AppContext` fields: `screen_stack`, `focus_history`, `push_screen_deferred()`, `pop_screen_deferred()`
- `crates/textual-rs/src/app.rs` — `full_render_pass()` (only renders top screen currently — D-01 requires change), `process_deferred_screens()`

### Python Textual parity reference
- `.planning/codebase/ARCHITECTURE.md` — Rust architecture patterns

### Testing patterns
- `crates/textual-rs/src/testing/mod.rs` — TestApp, Pilot API

No external ADRs — requirements fully captured in decisions above and ROADMAP.md Phase 5 success criteria.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `push_screen()` / `pop_screen()` in `tree.rs` — core mechanics already implemented and unit-tested; Phase 5 builds on top without rewriting them
- `ModalScreen` in `screen.rs` — transparent wrapper, `is_modal() -> true`, already public in `lib.rs`
- `push_screen_deferred()` / `pop_screen_deferred()` on `AppContext` — deferred API already exists; `push_screen_wait` is a new async variant

### Established Patterns
- `active_overlay` pattern (`select.rs`) — single-instance overlay; NOT used for screen stack (different mechanism)
- `pending_screen_pushes` / `pending_screen_pops` `RefCell`/`Cell` on `AppContext` — existing deferred mechanism; `push_screen_wait` will need a complementary result-return channel
- `SecondaryMap<WidgetId, T>` — per-widget state storage (not directly needed here but established pattern)
- `TestPilot` / `TestApp` — integration test harness already in use for existing widget tests

### Integration Points
- `full_render_pass()` in `app.rs` — must be changed to iterate `screen_stack` bottom-up, rendering each screen, then applying dim effect to all but the top
- `render_to_test_backend()` in `app.rs` — same change needed for test renders
- `AppContext` — will need a result-return mechanism for `push_screen_wait` (e.g., `pending_screen_results: RefCell<HashMap<WidgetId, Box<dyn Any>>>`)

</code_context>

<specifics>
## Specific Ideas

- Python Textual modal visual: background screen visible but dimmed — the dialog "floats" over the content the user was looking at, preserving context
- `tutorial_06_screens` should show the complete modal round-trip: open confirm dialog from a button press, await the result, update a label on the main screen with "You chose: OK/Cancel"
- `push_screen_wait` usage pattern: `let ok = ctx.push_screen_wait(Box::new(ModalScreen::new(Box::new(ConfirmDialog)))).await;`

</specifics>

<deferred>
## Deferred Ideas

- `push_screen_wait` was originally listed as WIDGET-F02 in REQUIREMENTS.md — now promoted to Phase 5 scope per user decision. REQUIREMENTS.md should be updated to move NAV-04 (push_screen_wait) from future to active.
- Multi-level modal stacking (modal on top of modal) — should work naturally with the stack, but no specific test required in Phase 5
- Screen transition animations (crossfade, slide) — out of scope, terminal TUI focus

</deferred>

---

*Phase: 05-screen-stack*
*Context gathered: 2026-03-28*
