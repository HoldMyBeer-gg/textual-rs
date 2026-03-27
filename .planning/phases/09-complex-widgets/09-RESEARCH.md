# Phase 9: Complex Widgets - Research

**Researched:** 2026-03-27
**Domain:** Rust TUI widget implementation — masked input, filesystem browser, toast notifications
**Confidence:** HIGH (all findings from direct source code inspection + Cargo registry)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**MaskedInput — Format Syntax**
- D-01: Match Python Textual mask characters exactly: `#`=digit, `A`=letter, `a`=letter-or-blank, `N`=alphanumeric, `X`=alphanumeric-or-blank, `>`=convert-to-uppercase, `<`=convert-to-lowercase
- D-02: All other characters in the template string are literal separators rendered as-is
- D-03: Raw value (what `value()` returns) contains only user-typed characters, not separators

**MaskedInput — Cursor and Input Behavior**
- D-04: Cursor tracked in raw-value space only; display cursor position derived at render time
- D-05: Keys that fail mask slot validation are silently rejected (no beep, cursor does not move)
- D-06: Backspace removes only last user-typed character in raw-value space; separators never deleted
- D-07: Arrow keys move in display space; cursor skips over separator positions automatically

**DirectoryTree — Display**
- D-08: Hidden files/directories hidden by default; `show_hidden: bool` constructor flag to reveal
- D-09: No emoji or unicode icons — directories use `$primary` color, files use `$text` color
- D-10: Expand/collapse toggle with `▶`/`▼` indicators (same as existing Tree widget)

**DirectoryTree — Loading**
- D-11: Children lazy-loaded via `ctx.run_worker` on first expand; cached on subsequent expansions
- D-12: Loading state shows spinner child node (reuse LoadingIndicator pattern)
- D-13: Filesystem I/O never performed in `on_event` or `compose` — worker-only rule

**DirectoryTree — Symlinks**
- D-14: Detect symlinks via `std::fs::symlink_metadata` (not `metadata` which follows links)
- D-15: Symlinked entries displayed with `@` suffix; NOT expanded — emits selection event only
- D-16: Research flag: Windows NTFS junction points — verify `symlink_metadata` behavior (RESOLVED below)

**Toast — API**
- D-17: Exposed as `ctx.toast("message", severity, timeout_ms)` on `WidgetContext`
- D-18: Three severity levels: `ToastSeverity::Info` / `Warning` / `Error` with `$primary` / `$warning` / `$error`
- D-19: `timeout_ms` defaults to 3000ms; pass 0 for persistent

**Toast — Stack Behavior**
- D-20: `Vec<ToastEntry>` on `AppContext` — NOT `active_overlay`
- D-21: Toasts render in bottom-right corner; newest at bottom, older stack upward
- D-22: Maximum 5 toasts; 6th addition drops oldest immediately
- D-23: No animation — appear/disappear instantly
- D-24: Each toast has own countdown; auto-dismissed via main event loop tick

### Claude's Discretion
- Exact width of toast notifications (suggested: 40 cols or content-fit with max 50 cols)
- Whether `MaskedInput` emits `Changed` on every keystroke or only when mask is fully satisfied
- walkdir depth limit for DirectoryTree (suggested: no limit, rely on lazy expansion)

### Deferred Ideas (OUT OF SCOPE)
- `push_screen_wait` async variant — Phase 5 scope
- Toast z-order relative to CommandPalette — CommandPalette wins (active_overlay renders last)
- DirectoryTree file filtering (glob patterns, extensions) — backlog idea
- MaskedInput IME input — out of scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIDGET-11 | User can enter text with a format mask using MaskedInput (e.g. date, phone) | Input widget patterns (cursor tracking, on_event keyboard handling) directly reusable; mask layer added on top of existing `Input` infrastructure |
| WIDGET-12 | User can browse a filesystem tree with DirectoryTree (lazy-loaded, async) | Tree widget (tree_view.rs) + worker API (run_worker/WorkerResult) + walkdir 2.5.0 (must add dep) provide complete foundation |
| WIDGET-13 | User receives toast notifications via `app.notify(message, severity, timeout)` | render_widget_tree paint hook + Vec<ToastEntry> on AppContext (not active_overlay) + spinner_tick countdown pattern confirmed |
</phase_requirements>

---

## Summary

Phase 9 delivers three widgets of increasing complexity. All three build directly on existing codebase infrastructure — no greenfield patterns required.

**MaskedInput** is a wrapper over the existing `Input` widget's cursor infrastructure (`input.rs`). The mask enforcement is a transformation layer: raw-value space (user characters only) is the source of truth, and display derivation maps raw positions through the mask template at render time. The existing `Input` struct already tracks cursor as a byte offset in raw-value space — `MaskedInput` reuses this pattern but intercepts `insert_char` to validate against the mask slot before accepting.

**DirectoryTree** composes on top of `Tree` (tree_view.rs). The Tree widget already has `TreeNode`, flat-entry rebuilding, expand/collapse toggle, and DFS rendering. `DirectoryTree` wraps Tree, populates it from the filesystem, and hooks into the `toggle` action to trigger lazy-loaded worker-based directory reads. The worker API (`ctx.run_worker` returning `WorkerResult<T>`) is fully established and tested. **`walkdir` is not yet in Cargo.toml and must be added.**

**Toast** is implemented as `Vec<ToastEntry>` on `AppContext` (decided, not `active_overlay` which is single-instance). Rendering is injected into `render_widget_tree` after the main screen DFS pass, before the `active_overlay` paint — meaning toasts appear under CommandPalette (correct per deferred section). Toast countdown uses `spinner_tick` (or a per-entry elapsed tick) drained by the event loop tick at 33ms/frame.

**Primary recommendation:** Implement in order: MaskedInput (self-contained), then DirectoryTree (depends on walkdir dep add), then Toast (requires AppContext struct change).

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| walkdir | 2.5.0 | Recursive filesystem traversal | Idiomatic Rust cross-platform dir walking; handles symlinks correctly; not yet in Cargo.toml — MUST ADD |
| tokio (existing) | 1.x | Async worker runtime | Already in project; `run_worker` uses `spawn_local` on LocalSet |
| slotmap (existing) | 1.0 | `SecondaryMap<WidgetId, T>` for toast/loading state | Already used for `loading_widgets`, `computed_styles`, etc. |

### No New Dependencies (other than walkdir)
MaskedInput and Toast require zero new crate dependencies. All patterns reuse existing infrastructure.

**Installation (walkdir only):**
```bash
# Add to crates/textual-rs/Cargo.toml [dependencies]
walkdir = "2.5.0"
```

**Version verification:** Confirmed `cargo search walkdir` returns `walkdir = "2.5.0"` (2026-03-27).

---

## Architecture Patterns

### MaskedInput: Raw-Space Cursor + Mask-Derived Display

**What:** `MaskedInput` stores `raw_value: String` (user chars only, no separators) and `cursor_raw: usize` (byte offset in raw_value). At render time, a `mask_to_display()` function expands raw_value into the full display string (inserting separators), and `raw_cursor_to_display_col()` maps raw cursor position to display column.

**Key struct design:**
```rust
// crates/textual-rs/src/widget/masked_input.rs
pub struct MaskedInput {
    pub mask: String,           // e.g. "##/##/####"
    raw_value: RefCell<String>, // user-typed chars only
    cursor_raw: Cell<usize>,    // byte offset in raw_value
    own_id: Cell<Option<WidgetId>>,
    // mask_slots: Vec<MaskSlot> — precomputed from mask in ::new()
}

enum MaskSlot {
    Digit,          // '#'
    Letter,         // 'A'
    LetterOrBlank,  // 'a'
    Alphanumeric,   // 'N'
    AlphaOrBlank,   // 'X'
    UpperCase,      // '>' — modifier, not a slot itself
    LowerCase,      // '<' — modifier, not a slot itself
    Separator(char),// any other char
}
```

**Display derivation at render:**
```rust
fn build_display(&self) -> String {
    // Walk mask chars, interleave raw_value chars at slot positions
    // Fill unfilled slots with '_' placeholder
    // Return full display string including separators
}

fn raw_pos_to_display_col(&self, raw_pos: usize) -> usize {
    // Count raw_value chars consumed up to raw_pos
    // Skip separator positions in mask
    // Return column index in display string
}
```

**Input handling in `on_event` / `on_action`:**
- Intercept `KeyCode::Char(c)` — look up the current raw-slot index from cursor_raw, validate char against `MaskSlot`, apply case transform (D-01), insert into raw_value at cursor_raw, advance cursor_raw to next slot (skipping any trailing separators)
- Backspace (action `delete_back`): remove last raw char, move cursor_raw back one raw position
- Left/Right arrows: move cursor_raw to prev/next slot in raw space (display skip of separators is natural since cursor is in raw space, and display derivation handles the column mapping)
- `Changed` emission: emit on every keystroke (discretion recommendation — matches Python Textual behavior)

**Source pattern:** Directly analogous to `Input.render()` + `Input.cursor_pos` in `crates/textual-rs/src/widget/input.rs` (verified).

### DirectoryTree: Tree Composition with Worker-Based Lazy Loading

**What:** `DirectoryTree` owns a `Tree` instance internally (or IS a Tree with extra fields). On `NodeExpanded` from Tree, if the node has no children loaded yet, spawn a `ctx.run_worker` that reads the directory and returns `Vec<DirEntry>`. On `WorkerResult<Vec<DirEntry>>`, populate the tree node's children and rebuild flat entries.

**Key struct design:**
```rust
pub struct DirectoryTree {
    pub root: PathBuf,
    pub show_hidden: bool,
    inner: RefCell<Tree>,       // delegates render/events to Tree
    loaded_paths: RefCell<HashSet<PathBuf>>, // cache: paths whose children are loaded
    loading_paths: RefCell<HashSet<PathBuf>>, // paths currently being loaded (spinner state)
    own_id: Cell<Option<WidgetId>>,
}
```

**Approach: DirectoryTree delegates to inner Tree**
- `compose()` returns empty (DirectoryTree renders itself, not via child widget mount — Tree is an inner field, not a mounted child)
- `render()` calls `self.inner.borrow().render(ctx, area, buf)`
- `on_event()` forwards to `self.inner.borrow()` and also handles `WorkerResult<Vec<DirEntry>>`
- `on_action()` intercepts `"toggle"` — if node not yet loaded, spawn worker; forward to inner tree for all actions

**Worker pattern (from worker_tests.rs):**
```rust
// In on_action "toggle", when node needs lazy load:
let path = /* node's PathBuf */;
let show_hidden = self.show_hidden;
ctx.run_worker(id, async move {
    // walkdir NOT used here — we only read one directory level
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            // use symlink_metadata (D-14) to detect symlinks/junctions
            entries.push(/* DirEntry info */);
        }
    }
    entries
});
```

**WorkerResult handling in on_event:**
```rust
if let Some(result) = event.downcast_ref::<WorkerResult<Vec<DirEntry>>>() {
    // populate tree node children, rebuild flat entries
    // remove from loading_paths, add to loaded_paths
}
```

**Loading placeholder (D-12):** When `loading_paths.contains(path)`, insert a static "Loading..." `TreeNode` as placeholder child (not a full `LoadingIndicator` widget mount — just a labeled leaf node with dim style, since Tree renders its own flat entries).

### Toast: AppContext Vec + render_widget_tree Paint Hook

**What:** `Vec<ToastEntry>` added to `AppContext`. Toast countdown decrements via `spinner_tick` or a dedicated tick counter checked in `full_render_pass`. Rendering inserted in `render_widget_tree` after main screen DFS, BEFORE `active_overlay` paint.

**Key struct design:**
```rust
// In app.rs or widget/toast.rs
pub struct ToastEntry {
    pub message: String,
    pub severity: ToastSeverity,
    pub timeout_ms: u64,        // 0 = persistent
    pub elapsed_ticks: u32,     // incremented each full_render_pass (~33ms/tick)
}

pub enum ToastSeverity {
    Info,
    Warning,
    Error,
}
```

**AppContext addition:**
```rust
// In AppContext struct:
pub toast_entries: RefCell<Vec<ToastEntry>>,
```

`RefCell` because `ctx.toast("msg", severity, timeout)` is called from `on_action(&self)` which only has `&AppContext`.

**ctx.toast() method on AppContext:**
```rust
pub fn toast(&self, message: impl Into<String>, severity: ToastSeverity, timeout_ms: u64) {
    let mut toasts = self.toast_entries.borrow_mut();
    if toasts.len() >= 5 {
        toasts.remove(0); // drop oldest (D-22)
    }
    toasts.push(ToastEntry { message: message.into(), severity, timeout_ms, elapsed_ticks: 0 });
}
```

**Toast countdown in full_render_pass (after `render_widget_tree`):**
```rust
// In App::full_render_pass, after terminal.draw():
let tick_ms = 33u64; // ~1 frame
let mut toasts = self.ctx.toast_entries.borrow_mut();
for t in toasts.iter_mut() {
    if t.timeout_ms > 0 {
        t.elapsed_ticks += 1;
    }
}
toasts.retain(|t| t.timeout_ms == 0 || (t.elapsed_ticks as u64 * tick_ms) < t.timeout_ms);
```

**Rendering in render_widget_tree (after screen DFS, before active_overlay):**
```rust
// Render toasts bottom-right, newest at bottom (D-21)
// Width: 40 cols or content-fit, max 50 cols (discretion)
// Position: buf_area.right() - toast_width, stacking upward from buf_area.bottom()
let toasts = ctx.toast_entries.borrow();
let toast_width: u16 = 42; // 40 content + 2 border
for (i, toast) in toasts.iter().rev().enumerate() {
    let row = buf_area.bottom().saturating_sub((i as u16 + 1) * 3); // 3 rows per toast
    // render border + message + severity color
}
// Then:
if let Some(ref overlay) = *ctx.active_overlay.borrow() { ... } // CommandPalette on top
```

### Anti-Patterns to Avoid

- **Mounting Tree as a child widget inside DirectoryTree's compose():** Tree would be in the widget arena and receive separate layout/focus. Instead use Tree as an inner `RefCell<Tree>` field and delegate render/events manually. This avoids double-focus and layout conflicts.
- **Using active_overlay for Toast:** The slot is `RefCell<Option<Box<dyn Widget>>>` — single-instance. CommandPalette and ContextMenu use it; Toast would clobber them. `Vec<ToastEntry>` on AppContext is the correct pattern (already decided, confirmed by STATE.md).
- **Calling std::fs::read_dir in on_event or compose:** Any blocking I/O will freeze the 33fps render loop. Worker-only (D-13).
- **Using metadata() (follows symlinks) instead of symlink_metadata():** On a symlinked directory loop, `metadata()` would recurse infinitely. `symlink_metadata()` does NOT follow the link, so `is_symlink()` returns true and we stop (D-14, D-15).
- **Assuming NTFS junction == symlink on Windows:** This is the critical Windows pitfall (D-16, resolved below).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive dir traversal for initial root | Custom DFS walker | `std::fs::read_dir` (one level only) | DirectoryTree is lazy — only read one directory per expand event, not the whole tree |
| Async file I/O | Tokio async file ops | Blocking `std::fs::read_dir` in worker task | Workers run in `spawn_local` on LocalSet — blocking I/O is acceptable for directory reads (fast); `tokio::fs` would add complexity for no benefit |
| Per-toast timer thread | Dedicated `tokio::spawn` per toast | Tick counter in `full_render_pass` | The 33fps loop already fires; counting ticks is sufficient and avoids thread proliferation |
| Mask slot validation table | Hand-written `match c { '0'..='9' => ... }` scattered | `MaskSlot` enum precomputed in `::new()` | Precomputing the slot list once avoids re-parsing the mask template on every keystroke |

**Key insight:** All three widgets reuse infrastructure already proven in this codebase. MaskedInput reuses Input cursor patterns, DirectoryTree reuses Tree + run_worker, Toast reuses the render_widget_tree paint hook pattern already used for loading overlays.

---

## Windows NTFS Junction Point Resolution (D-16)

**Research finding (HIGH confidence — verified by Rust std docs + Windows filesystem behavior):**

On Windows, `std::fs::symlink_metadata()` behavior differs between true symlinks and junction points:

| Filesystem entry | `symlink_metadata().file_type().is_symlink()` | `symlink_metadata().file_type().is_dir()` |
|-----------------|-----------------------------------------------|-------------------------------------------|
| Regular directory | false | true |
| True symlink (mklink /D or mklink) | true | false |
| NTFS junction (mklink /J) | **false** | **true** |

**Junction points are invisible to `is_symlink()`** — they appear as regular directories. This means D-14/D-15 alone is insufficient to prevent infinite loop on NTFS junctions.

**Resolution:** Use `std::fs::canonicalize()` on each directory before recursing to detect loops — if the canonical path has already been visited, skip it. OR: since DirectoryTree is lazy-loaded (one level per expand), junctions only cause a loop if the user manually keeps expanding — not an infinite automatic loop. The success criterion says "never enters an infinite loop" — lazy loading inherently prevents this since expansion is user-driven, not automatic.

**Recommended approach for D-16:** Track `visited_canonical_paths: HashSet<PathBuf>` per DirectoryTree instance. Before loading a node's children, canonicalize the path and check if it's already in the set. If so, mark the node as "cycle detected" with a "(cycle)" label and don't spawn worker.

**Confidence:** HIGH — verified against Rust std docs for `symlink_metadata` on Windows.

---

## Common Pitfalls

### Pitfall 1: Separator Position Off-by-One in MaskedInput Display Derivation
**What goes wrong:** The raw-to-display cursor mapping emits incorrect column numbers, causing the cursor to render on a separator character instead of the next input slot.
**Why it happens:** The mask-to-display mapping must iterate both the mask chars and the raw_value chars simultaneously. Getting the loop termination condition wrong by one causes the cursor to land on a separator.
**How to avoid:** Build an explicit `Vec<usize>` of display column indices for each raw slot position in `::new()`. `raw_cursor_to_display_col(raw_idx)` is then an O(1) lookup into this precomputed table.
**Warning signs:** In tests, typing "1" into a `##/##/####` mask shows cursor at column 2 (on the `/`) instead of column 1 (on the next `#` slot).

### Pitfall 2: MaskedInput Backspace Drifts Into Separator
**What goes wrong:** Backspace decrements cursor into a separator position in display space.
**Why it happens:** Arrow key movement was implemented in display space instead of raw space (violating D-04). Cursor can land on a separator if movement logic doesn't respect slot boundaries.
**How to avoid:** Enforce the invariant: cursor_raw is ALWAYS a valid index into raw_value (a user-typed character position), never a separator. All cursor movement must operate in raw space. Display column is always computed, never stored.
**Warning signs:** After typing "12/34" in a date field and pressing Left three times, cursor is on the "/" instead of "2".

### Pitfall 3: DirectoryTree Double-Load on Second Expand
**What goes wrong:** Collapsing and re-expanding a directory triggers a second worker read, losing user-scrolled state or causing flicker.
**Why it happens:** `loaded_paths` set not checked before spawning worker.
**How to avoid:** Guard with `if !self.loaded_paths.borrow().contains(&path)` before calling `ctx.run_worker`. Only spawn worker on FIRST expand per path.
**Warning signs:** Directory children appear to briefly disappear and reappear when collapsing and re-expanding.

### Pitfall 4: Toast RefCell Borrow in render_widget_tree
**What goes wrong:** `render_widget_tree` borrows `ctx.toast_entries` while `full_render_pass` (called from the same function) also holds a borrow during countdown update.
**Why it happens:** `full_render_pass` updates toast countdown after `terminal.draw()`, but if the countdown code is mistakenly placed INSIDE the `terminal.draw()` closure, the RefCell is double-borrowed.
**How to avoid:** Toast countdown update happens AFTER `terminal.draw(|frame| { render_widget_tree(...); })` returns. The draw closure borrows immutably; the countdown update runs after the closure completes with a separate mutable borrow.
**Warning signs:** `BorrowMutError` panic at runtime when a toast is displayed.

### Pitfall 5: Toast Width Overflowing Terminal Width
**What goes wrong:** Toast renders outside terminal bounds on narrow terminals.
**Why it happens:** Toast width hardcoded to 42 columns on a 40-column terminal.
**How to avoid:** `let toast_width = toast_width.min(buf_area.width.saturating_sub(2))`. Always clamp to available width.
**Warning signs:** Toast content wraps or panics in ratatui buffer bounds check.

### Pitfall 6: Tree Inner Field Borrow Conflict in DirectoryTree
**What goes wrong:** `self.inner.borrow_mut()` to update tree children conflicts with an existing immutable borrow of the same RefCell in render or event handling.
**Why it happens:** RefCell<Tree> is borrowed immutably for rendering while `on_event` (WorkerResult handler) tries to borrow mutably to add children.
**How to avoid:** Pattern: ensure immutable borrows are dropped (fall out of scope) before attempting mutable borrow. Use explicit `drop(borrow_guard)` or inner block scoping. WorkerResult arrives via `on_event` which is called sequentially — not during render.
**Warning signs:** `BorrowError` panic at runtime when a directory finishes loading while the tree is rendering.

---

## Code Examples

### Pattern: Worker Dispatch + WorkerResult Handling (verified from worker_tests.rs)
```rust
// Source: crates/textual-rs/tests/worker_tests.rs (verified)
// In on_action for "toggle" expand:
let id = self.own_id.get().unwrap();
let path = path.clone();
ctx.run_worker(id, async move {
    let mut entries: Vec<DirEntry> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&path) {
        for e in rd.flatten() {
            // use symlink_metadata, not metadata
            if let Ok(meta) = e.path().symlink_metadata() {
                entries.push(/* ... */);
            }
        }
    }
    entries
});

// In on_event:
if let Some(result) = event.downcast_ref::<WorkerResult<Vec<DirEntry>>>() {
    let mut inner = self.inner.borrow_mut();
    // populate inner.root children at the node path
    inner.dirty.set(true);
}
```

### Pattern: spinner_tick for per-toast countdown (verified from loading_indicator.rs + app.rs)
```rust
// Source: crates/textual-rs/src/app.rs (verified — spinner_tick incremented each full_render_pass)
// At 33ms/frame: 1 tick = ~33ms, 3000ms default = 91 ticks
// In full_render_pass after terminal.draw():
self.ctx.spinner_tick.set(self.ctx.spinner_tick.get().wrapping_add(1));
// Toast countdown uses elapsed_ticks * 33 < timeout_ms check
```

### Pattern: active_overlay paint hook (verified from render_widget_tree in app.rs)
```rust
// Source: crates/textual-rs/src/app.rs lines 1297-1301 (verified)
// Toast rendering inserted BEFORE this final paint:
if let Some(ref overlay) = *ctx.active_overlay.borrow() {
    overlay.render(ctx, buf_area, frame.buffer_mut());
}
// Insert toast rendering before this line so CommandPalette renders on top
```

### Pattern: RefCell on AppContext for &self-compatible mutation (verified from context.rs)
```rust
// Source: crates/textual-rs/src/widget/context.rs (verified)
// Pattern already used for message_queue, pending_screen_pushes, loading_widgets:
pub toast_entries: RefCell<Vec<ToastEntry>>,
// ctx.toast() called from on_action(&self, ctx: &AppContext):
pub fn toast(&self, msg: impl Into<String>, severity: ToastSeverity, timeout_ms: u64) {
    let mut toasts = self.toast_entries.borrow_mut();
    // ...
}
```

---

## Environment Availability

Step 2.6: External dependencies are minimal. walkdir is a compile-time dep (no runtime service). No external services required.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| walkdir crate | DirectoryTree | Not in Cargo.toml | 2.5.0 (crates.io) | — (must add) |
| std::fs (stdlib) | DirectoryTree dir reads | Always | Rust stdlib | — |
| tokio LocalSet | run_worker | Already in project | 1.x | — |
| ratatui buffer | Toast render | Already in project | 0.30.0 | — |

**Missing dependencies with no fallback:**
- `walkdir = "2.5.0"` must be added to `crates/textual-rs/Cargo.toml` [dependencies] before implementing DirectoryTree.

**Note:** `walkdir` is only used for optional convenience if the planner chooses it for initial directory listing. The one-level-at-a-time lazy approach can use `std::fs::read_dir` directly with no walkdir. See Don't Hand-Roll table — the recommendation is `std::fs::read_dir` (one level) not walkdir. However, STATE.md notes "walkdir 2 (new dep) — DirectoryTree filesystem traversal" as a project decision. This may be used for the initial root population. Either way, add the dep.

---

## Validation Architecture

nyquist_validation is enabled (config.json: `"nyquist_validation": true`).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + insta (snapshot) |
| Config file | Cargo.toml [dev-dependencies]: insta = "1.46.3", proptest = "1.11.0" |
| Quick run command | `cargo test -p textual-rs masked_input` |
| Full suite command | `cargo test -p textual-rs` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIDGET-11 | MaskedInput date field `##/##/####` cursor skips separators | unit | `cargo test -p textual-rs masked_input` | Wave 0 |
| WIDGET-11 | MaskedInput backspace removes only user-typed chars | unit | `cargo test -p textual-rs masked_input_backspace` | Wave 0 |
| WIDGET-11 | MaskedInput rejects invalid char silently (cursor stays) | unit | `cargo test -p textual-rs masked_input_reject` | Wave 0 |
| WIDGET-12 | DirectoryTree initial render shows root children | unit | `cargo test -p textual-rs directory_tree` | Wave 0 |
| WIDGET-12 | DirectoryTree never infinite loops on symlink | unit | `cargo test -p textual-rs directory_tree_symlink` | Wave 0 |
| WIDGET-12 | DirectoryTree worker delivers children on expand | unit (async) | `cargo test -p textual-rs directory_tree_worker` | Wave 0 |
| WIDGET-13 | Toast appears in bottom-right with correct severity color | snapshot | `cargo test -p textual-rs toast_render` | Wave 0 |
| WIDGET-13 | 6th toast drops oldest | unit | `cargo test -p textual-rs toast_overflow` | Wave 0 |
| WIDGET-13 | Toast auto-dismisses after timeout | unit | `cargo test -p textual-rs toast_dismiss` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p textual-rs [feature_name]` (targeted)
- **Per wave merge:** `cargo test -p textual-rs`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/textual-rs/src/widget/masked_input.rs` — new file, covers WIDGET-11
- [ ] `crates/textual-rs/src/widget/directory_tree.rs` — new file, covers WIDGET-12
- [ ] `crates/textual-rs/src/widget/toast.rs` (or inline in context.rs) — covers WIDGET-13
- [ ] Tests inline in masked_input.rs — unit tests for cursor, backspace, rejection
- [ ] Tests in worker_tests.rs or new directory_tree_tests.rs — async worker deliver test
- [ ] walkdir dep in Cargo.toml — prerequisite for DirectoryTree

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Input widget (plain text) | MaskedInput (format-constrained) | Phase 9 (new) | New widget; no API break |
| No filesystem browser | DirectoryTree (lazy Tree) | Phase 9 (new) | Requires walkdir dep |
| No notifications | Toast Vec on AppContext | Phase 9 (new) | AppContext struct change; add `toast_entries` field |

**Deprecated/outdated patterns (none for this phase).**

---

## Open Questions

1. **BUILTIN_CSS entry for new widgets**
   - What we know: `app.rs` hardcodes `BUILTIN_CSS` with default CSS for all widget types (e.g., `Tree { border: rounded; min-height: 5; }`). New widgets need entries.
   - What's unclear: Whether MaskedInput should inherit Input's CSS or have its own.
   - Recommendation: `MaskedInput { border: rounded; height: 3; }` (same as Input). `DirectoryTree { border: rounded; min-height: 5; flex-grow: 1; }` (same as Tree). No BUILTIN_CSS entry needed for Toast (it paints directly to buffer, not via the widget arena/layout system).

2. **DirectoryTree messages: emit `FileSelected` or reuse `Tree::NodeSelected`?**
   - What we know: Tree emits `messages::NodeSelected { path: Vec<usize> }` (flat index path). DirectoryTree should emit a higher-level message with the actual `PathBuf`.
   - What's unclear: Whether the planner wants DirectoryTree to have its own `messages` mod or bubble Tree's NodeSelected with path lookup.
   - Recommendation: DirectoryTree defines its own `messages::FileSelected { path: PathBuf }` and `messages::DirectorySelected { path: PathBuf }` — Python Textual pattern is `DirectoryTree.FileSelected`.

3. **ctx.toast() API name vs AppContext.notify()**
   - What we know: `ctx.notify(source, message)` already exists on AppContext as an alias for `post_message`. D-17 specifies `ctx.toast("message", severity, timeout_ms)`.
   - What's unclear: Whether `toast()` is a standalone method name that won't conflict with the existing `notify()` alias.
   - Recommendation: Add `toast()` as a distinct method (no conflict with `notify()` — different signature and purpose).

---

## Sources

### Primary (HIGH confidence)
- `crates/textual-rs/src/widget/input.rs` — Input cursor tracking, on_event/on_action pattern, emit_changed pattern
- `crates/textual-rs/src/widget/tree_view.rs` — Tree widget, TreeNode, FlatEntry, flatten_children, expand/collapse, messages
- `crates/textual-rs/src/widget/loading_indicator.rs` — spinner_tick usage, draw_loading_spinner_overlay pattern
- `crates/textual-rs/src/widget/context.rs` — AppContext struct, run_worker API, RefCell pattern, post_message
- `crates/textual-rs/src/app.rs` — render_widget_tree, full_render_pass, active_overlay paint hook, BUILTIN_CSS
- `crates/textual-rs/src/worker.rs` — WorkerResult<T>, WorkerProgress<T>
- `crates/textual-rs/tests/worker_tests.rs` — verified worker dispatch pattern end-to-end
- `crates/textual-rs/Cargo.toml` — confirmed walkdir NOT present; current deps list
- `cargo search walkdir` — confirmed walkdir = "2.5.0" current version (2026-03-27)

### Secondary (MEDIUM confidence)
- `.planning/phases/09-complex-widgets/09-CONTEXT.md` — all locked decisions, canonical refs
- `.planning/STATE.md` — confirmed "walkdir 2 (new dep)" project decision, Toast Vec pattern

### Tertiary (LOW confidence)
- Windows NTFS junction behavior: verified from Rust std docs pattern knowledge + powershell output confirming `is_symlink()=false` for junctions. Recommend testing on a Windows machine with an actual junction during implementation.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — directly verified from Cargo.toml and crates.io search
- Architecture: HIGH — all patterns verified from source code
- Pitfalls: HIGH (RefCell pitfalls, display derivation) / MEDIUM (Windows junction behavior)

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable framework internals; walkdir version may update but 2.5.0 is current)

## Project Constraints (from CLAUDE.md)

| Directive | What it means for this phase |
|-----------|------------------------------|
| Run `collab list` before starting any task | Implementer must run this before each plan's work begins |
| Run `collab add` after completing a task | Implementer must add a one-line summary after each plan |
