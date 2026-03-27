# Phase 9: Complex Widgets - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning
**Mode:** auto (all decisions Claude-selected with recommended defaults)

<domain>
## Phase Boundary

Deliver three widgets with distinct interaction complexity: `MaskedInput` (templated text input with cursor-aware mask enforcement), `DirectoryTree` (lazy-loading filesystem browser), and `Toast` (stacked transient notification system). Screen stack (Phase 5) is a hard dependency — do not begin until Phase 5 is complete.

</domain>

<decisions>
## Implementation Decisions

### MaskedInput — Format Syntax
- **D-01:** Match Python Textual mask characters exactly: `#`=digit, `A`=letter, `a`=letter-or-blank, `N`=alphanumeric, `X`=alphanumeric-or-blank, `>`=convert-to-uppercase, `<`=convert-to-lowercase
- **D-02:** All other characters in the template string are literal separators rendered as-is (e.g., `/` in `##/##/####`)
- **D-03:** The raw value (what `value()` returns) contains only the characters the user typed, not the separators — separators are display-only

### MaskedInput — Cursor and Input Behavior
- **D-04:** Cursor is tracked in raw-value space only; display cursor position is derived at render time by mapping raw position → display position through the mask
- **D-05:** Pressing a key that doesn't satisfy the mask slot (e.g., a letter when slot requires a digit) is silently rejected — no beep, no error state, cursor does not move
- **D-06:** Backspace removes only the last user-typed character in raw-value space; display cursor re-derives accordingly; separator characters are never deleted
- **D-07:** Arrow keys move in display space; the cursor skips over separator positions automatically (stops only on mask slots)

### DirectoryTree — Display
- **D-08:** Hidden files/directories (dot-prefixed on Unix, hidden attribute on Windows) are hidden by default; `show_hidden: bool` constructor flag to reveal them
- **D-09:** No emoji or unicode icons — directories and files are distinguished by color only (directory label uses `$primary`, file label uses `$text`), ensuring broad terminal compatibility
- **D-10:** Expand/collapse toggle with `▶`/`▼` indicators (same as existing `Tree` widget pattern)

### DirectoryTree — Loading
- **D-11:** Directory children are lazy-loaded via `ctx.run_worker` when a node is first expanded; subsequent expansions re-use cached children (no re-read)
- **D-12:** While a directory is loading, show a spinner child node (reuse `LoadingIndicator` pattern if feasible, otherwise a static "Loading…" placeholder)
- **D-13:** Filesystem I/O is never performed in `on_event` or `compose` — worker-only rule enforced by design

### DirectoryTree — Symlinks
- **D-14:** Detect symlinks via `std::fs::symlink_metadata` (not `metadata` which follows links)
- **D-15:** Symlinked entries are displayed with `@` suffix (e.g., `link-to-dir@`) and are NOT expanded — clicking a symlink node emits a selection event but does not recurse into it, preventing infinite loops on Windows NTFS and Unix
- **D-16:** Research flag from STATE.md: Windows NTFS junction points — researcher should verify `symlink_metadata` behavior on Windows junctions vs true symlinks

### Toast — API
- **D-17:** Exposed as `ctx.toast("message", severity, timeout_ms)` on `WidgetContext` — matches Python Textual's `app.notify()` naming loosely but lives on ctx for consistency with the rest of the API
- **D-18:** Three severity levels: `ToastSeverity::Info` / `Warning` / `Error`, styled with `$primary` / `$warning` / `$error` semantic theme colors
- **D-19:** `timeout_ms` defaults to 3000ms if not specified; pass 0 for persistent (no auto-dismiss)

### Toast — Stack Behavior
- **D-20:** `Vec<ToastEntry>` on `AppContext` — NOT `active_overlay` (that slot is single-instance only)
- **D-21:** Toasts render in the bottom-right corner; newest toast appears at the bottom of the stack (closest to corner), older toasts stack upward
- **D-22:** Maximum 5 toasts visible simultaneously; if a 6th is added, the oldest is dropped immediately
- **D-23:** No slide/fade animation — toasts appear and disappear instantly (terminal-safe, avoids animation complexity)
- **D-24:** Each toast has its own countdown; auto-dismissed independently when its timer fires (via the main event loop tick, not a separate timer thread)

### Claude's Discretion
- Exact width of toast notifications (suggested: 40 cols or content-fit with max 50 cols)
- Whether `MaskedInput` emits `Changed` on every keystroke or only when mask is fully satisfied
- walkdir depth limit for DirectoryTree (suggested: no limit, rely on lazy expansion)

</decisions>

<specifics>
## Specific Ideas

- Python Textual widget gallery screenshots at https://textual.textualize.io/widgets/ are the visual reference for all three widgets
- MaskedInput parity goal: a date field `##/##/####` must behave identically to Python Textual's — type `1`, `2`, `/` auto-inserts, `3`, `1`, `/` auto-inserts, `1`, `9`, `9`, `9`
- Toast stacking reference: Python Textual's notify() shows multiple toasts without obscuring each other

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python Textual parity reference
- `.planning/codebase/ARCHITECTURE.md` — Rust architecture patterns, widget trait, compose/render cycle
- `.planning/codebase/CONVENTIONS.md` — Note: this file analyzes the Python Textual source; Rust conventions are inferred from existing widget implementations
- `.planning/codebase/STACK.md` — Dependency list including walkdir 2 (added for DirectoryTree)

### Widget implementation patterns
- `crates/textual-rs/src/widget/input.rs` — MaskedInput should derive from Input patterns (cursor tracking, on_event keyboard handling)
- `crates/textual-rs/src/widget/tree.rs` — DirectoryTree wraps/extends Tree widget (node expansion, lazy load hooks)
- `crates/textual-rs/src/widget/loading_indicator.rs` — Loading placeholder pattern for directory children
- `crates/textual-rs/src/widget/context.rs` — Worker API (`run_worker`, `run_worker_with_progress`) and notify() signature
- `crates/textual-rs/src/app.rs` — `active_overlay` pattern (single-instance); Toast must use separate `Vec<ToastEntry>` field on AppContext

### Existing widget with relevant overlay pattern
- `crates/textual-rs/src/widget/select.rs` — active_overlay usage (shows what NOT to do for Toast — that slot is taken)

No external ADRs — all requirements captured in decisions above and ROADMAP.md Phase 9 success criteria.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Tree` widget (`tree.rs`) — DirectoryTree should compose on top of Tree, not reimplement node expand/collapse
- `Input` widget (`input.rs`) — MaskedInput reuses cursor tracking, keyboard event handling; adds mask layer on top
- `LoadingIndicator` (`loading_indicator.rs`) — Can be used as placeholder child node during directory load
- `ctx.run_worker` — Established async work pattern; workers deliver `WorkerResult<T>` messages back to widget

### Established Patterns
- `active_overlay: RefCell<Option<Box<dyn OverlayWidget>>>` — Single-instance overlay slot; Toast cannot use this
- Cursor in raw-value space → display derivation at render: same pattern already decided for MaskedInput
- `SecondaryMap<WidgetId, T>` — Per-widget state storage pattern used in LoadingIndicator overlay
- `ctx.spinner_tick` — Synchronized animation tick (used in LoadingIndicator, available for loading placeholder)

### Integration Points
- `AppContext` struct in `app.rs` — Add `toast_entries: Vec<ToastEntry>` field here
- App render loop (`app.rs`) — Toast overlay rendering appended after main screen render, before flush
- `Widget::on_event` — MaskedInput keyboard handler intercepts KeyEvent before passing to base Input
- `compose()` on DirectoryTree — Mounts root Tree node; children loaded via worker on first expand

</code_context>

<deferred>
## Deferred Ideas

- `push_screen_wait` async variant — Phase 5 scope
- Toast z-order relative to CommandPalette — if they overlap, CommandPalette wins (active_overlay renders last); noted as research flag
- DirectoryTree file filtering (glob patterns, extensions) — backlog idea, not in Phase 9 scope
- MaskedInput IME input (East Asian input methods) — out of scope, future consideration

</deferred>

---

*Phase: 09-complex-widgets*
*Context gathered: 2026-03-27*
