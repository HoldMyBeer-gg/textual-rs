# Pitfalls Research

**Domain:** Rust TUI framework — adding 13 widgets + screen stack to existing system
**Researched:** 2026-03-26
**Confidence:** HIGH (based on codebase audit + verified patterns)

---

## Critical Pitfalls

### Pitfall 1: Screen Stack Pops Without Focus Restoration

**What goes wrong:**
When `pop_screen` is called the underlying screen becomes visible again but `ctx.focused_widget` is `None`. The user presses Tab and gets no response. Every widget on the resumed screen requires a click to re-acquire focus.

**Why it happens:**
The current `push_screen` and `pop_screen` in `tree.rs` do not save/restore `focused_widget`. Unmounting the top screen correctly clears focus (`focused_widget = None` when the focused widget is in the unmounted subtree), but nothing restores the previously-focused widget on the screen below.

**How to avoid:**
Maintain a `focus_history: Vec<Option<WidgetId>>` alongside `screen_stack`. Before each `push_screen`, push the current `ctx.focused_widget` to `focus_history`. After each `pop_screen`, pop from `focus_history` and restore `ctx.focused_widget`. If the restored `WidgetId` is no longer in the arena (edge case: dynamic recomposition), fall back to auto-focusing the first focusable widget on the current screen.

**Warning signs:**
After dismissing a modal via `pop_screen`, Tab key does nothing and no widget shows the focused ring. Also: the `advance_focus` call in `tree.rs` walks the screen stack from `screen_stack.last()`, so correct restoration is verifiable with a snapshot test that presses Tab after a pop.

**Phase to address:**
Screen stack implementation phase — design this into the data structure before any consumer code is written.

---

### Pitfall 2: Events Routed to the Wrong Screen Layer

**What goes wrong:**
Key events reach widgets on a screen that is not the top of the stack. This manifests as background screen widgets responding to keyboard input while a modal is open, or action handlers firing on the wrong screen after a rapid push/pop.

**Why it happens:**
The existing event dispatch path in `app.rs` uses `ctx.focused_widget` as the sole routing target, not the screen of the focused widget. If `push_screen` is called without updating `ctx.focused_widget` to a widget on the new screen, focus still points into the previous screen's subtree. Any widget on the old screen with a matching key binding will handle the event.

The existing `active_overlay` guard routes key events exclusively to the overlay, but that guard is not reused for the screen stack. Screen pushes that happen via `pending_screen_pushes` (deferred from `on_action`) have a one-frame lag before focus is redirected.

**How to avoid:**
After draining `pending_screen_pushes` in the event loop, immediately auto-focus the first focusable widget on the new screen before processing any further events in the same loop iteration. Add an assertion in debug builds: if `focused_widget` is `Some(id)`, verify `id` is in the subtree of `screen_stack.last()`.

**Warning signs:**
A test that pushes a modal screen and then sends a key event observes the background widget changing state. The snapshot shows both screens affected.

**Phase to address:**
Screen stack implementation phase — define the invariant "focus is always in the active screen's subtree" as a debug assertion from day one.

---

### Pitfall 3: Render Artifacts When Screen Stack Changes Mid-Frame

**What goes wrong:**
After `pop_screen`, the area previously occupied by the modal shows stale cells from the previous frame — usually the modal's border or background colors persisting for one frame. On slow terminals this flicker is visible; on fast terminals it manifests as a brief flash.

**Why it happens:**
Ratatui's diff engine only writes cells that changed between the current and previous buffer. When a modal is removed, the cells it occupied in the previous buffer still exist. The layer below must fully repaint those cells in the new frame. If the underlying screen's render does not cover every cell (e.g., it only covers its own widget area, leaving corners/gutters unrendered), the diff engine sees no change and the stale cells persist.

The ratatui forum documents this exact issue: "List + Tabs widgets switching keeps character artifact upon rendering" — the fix is to use `Clear` on the affected area before rendering content on top.

**How to avoid:**
In `full_render_pass`, after processing a `pop_screen`, call `frame.render_widget(ratatui::widgets::Clear, full_area)` for the area that was previously occupied by the popped screen. Alternatively, call `terminal.clear()` once after any screen stack change — this forces a full redraw at the cost of one frame of blank terminal, which is imperceptible at 30fps.

**Warning signs:**
Snapshot tests taken immediately after a pop show unexpected characters in the area the modal occupied. If you see characters that match the modal's border style in a snapshot that should show only the background screen, this pitfall has occurred.

**Phase to address:**
Screen stack implementation phase — add a snapshot test: push modal, dismiss, verify snapshot matches background screen without artifacts.

---

### Pitfall 4: Toast Timers That Outlive Their Widget

**What goes wrong:**
A toast is displayed and a `tokio::time::sleep` task is spawned to dismiss it after 5 seconds. The user navigates to a different screen before the timer fires. When the timer fires, it posts a dismiss event. The event loop processes it and tries to mutate a widget that no longer exists (or worse, its `WidgetId` has been reallocated to a new widget in the arena).

**Why it happens:**
`DenseSlotMap` reuses slots. A `WidgetId` from a dismissed widget can be reassigned to a new widget. A stale timer firing and calling `ctx.send_message(old_id, Dismiss)` will deliver the message to the wrong widget.

**How to avoid:**
Do not use raw `WidgetId` as the timer's target identifier. Use the worker API (already in the framework) to spawn the timer as a worker tied to the toast widget's lifetime. When the toast is unmounted, `ctx.cancel_workers(id)` is already called in `unmount_widget`, which will cancel the timer task automatically. This is the safest pattern and requires zero additional bookkeeping.

Alternatively, store a generation counter alongside each toast's timer ID and verify the generation matches on timer fire.

**Warning signs:**
A toast's dismiss action fires but the wrong widget changes state. Test by mounting a toast, unmounting it manually before the timer fires, and verifying no panic or wrong-widget mutation occurs.

**Phase to address:**
Toast widget implementation — choose the worker-based timer from the start.

---

### Pitfall 5: Multiple Toasts Overlapping Instead of Stacking

**What goes wrong:**
A second toast notification is triggered before the first dismisses. Both are rendered at the same absolute coordinates, producing overlapping or corrupted text.

**Why it happens:**
Toast is an overlay (not part of the layout tree) painted at absolute coordinates. The active_overlay pattern supports exactly one overlay at a time. Naively using `active_overlay` for toasts means the second toast replaces the first, or if toasts are separate widgets, there is no coordinate system for stacking them.

**How to avoid:**
Do not use `active_overlay` for toasts — that slot is reserved for command palette and context menu. Implement toasts as a separate `Vec<ToastEntry>` on `AppContext` (or as a dedicated `ToastLayer` widget that is always mounted as the topmost child of the root screen). The `ToastLayer` renders all active toasts stacked vertically, anchored to the bottom-right corner. New toasts are appended to the bottom of the stack. Each entry carries its own dismiss timer handle.

Python Textual's implementation places toasts in a scrolling list in the bottom-right corner — the same approach works for textual-rs.

**Warning signs:**
Two `notify()` calls in quick succession produce a single visible toast. The second call's text does not appear. Verify with a test that calls notify twice and inspects the toast layer's rendered output.

**Phase to address:**
Toast widget implementation — design the `Vec<ToastEntry>` data structure before rendering.

---

### Pitfall 6: DirectoryTree Hangs on Symlink Loops

**What goes wrong:**
`DirectoryTree` expands a directory containing a symlink that points to an ancestor directory. The expansion recurses infinitely, consuming CPU and eventually stack-overflowing or hanging the UI.

**Why it happens:**
`std::fs::read_dir` follows symlinks by default on directory expansion. Without loop detection, a cycle `a/ -> b/ -> a/` recurses without bound.

**How to avoid:**
Use the `walkdir` crate (which has built-in symlink loop detection via `same_file`) or implement loop detection manually: before expanding a directory node, collect all ancestor paths currently in the expanded tree and check whether the candidate path resolves (via `std::fs::canonicalize`) to any ancestor. If a loop is detected, display the node as a leaf with a `[loop]` indicator rather than expanding it. Never call `canonicalize` on each keypress — cache the canonical path at mount time.

Alternatively, limit expand depth to a configurable maximum (default 50 levels) as a defense in depth.

**Warning signs:**
Expanding a directory silently causes the UI to freeze. CPU usage spikes to 100% on a single core. If using the worker API for directory loading (recommended), the worker task hangs and the tree never shows children.

**Phase to address:**
DirectoryTree widget implementation — add symlink loop detection before shipping. Test with a synthetic directory structure containing a symlink cycle.

---

### Pitfall 7: DirectoryTree Blocking the Event Loop with Synchronous fs::read_dir

**What goes wrong:**
`DirectoryTree` calls `std::fs::read_dir` synchronously in `on_event` or `compose`. On a network mount, NFS path, or a directory with thousands of entries, this blocks the tokio current_thread runtime for hundreds of milliseconds. The entire UI freezes.

**Why it happens:**
`tokio::runtime::Builder::new_current_thread()` runs all futures and the UI loop on one thread. Any blocking call in that thread stalls everything — including the 33ms render tick.

**How to avoid:**
Use `ctx.run_worker` (already in the framework) to perform directory reads on a background thread. Send back a `Vec<DirEntry>` message to the widget. Show a `LoadingIndicator` while the worker is in flight. This is the same pattern as other data-loading widgets in the framework.

**Warning signs:**
The UI freezes when expanding large directories. The render tick gaps are visible as latency spikes. The fix is always to move the blocking I/O to a worker.

**Phase to address:**
DirectoryTree widget implementation — use workers from the first implementation, not as a later optimization.

---

### Pitfall 8: MaskedInput Cursor Position Drift on Deletion

**What goes wrong:**
User presses Backspace in a masked input field formatted as `##/##/####`. The display cursor jumps backward past a mask separator character, placing it one position off from where the user expects. Subsequent input lands in the wrong position.

**Why it happens:**
The raw value (digits only: `12252026`) and the display value (`12/25/2026`) have different character offsets. A cursor at display position 5 corresponds to raw position 4. When the cursor is maintained in display space and the display string is rebuilt after deletion, the recalculation can drift if separator characters are not accounted for.

The equivalent problem in web masked inputs is well-documented: after reformatting, the cursor must be explicitly repositioned to the correct display offset, accounting for how many separators precede the raw cursor position.

**How to avoid:**
Maintain cursor position in raw (unmasked) value space only. On every render pass, compute the display cursor position by iterating the mask pattern: for each character in the mask, if it is a separator, it adds one to the display offset but zero to the raw offset; if it is a placeholder, both offsets increment. The display cursor position is derived from the raw cursor position, never stored independently.

Verify with property tests: for any raw cursor position in [0, raw_len], the round-trip raw → display → raw produces the same position.

**Warning signs:**
After typing in a masked field and pressing Backspace, the cursor is in the wrong column. The rendered character at the cursor position is a separator character rather than a digit.

**Phase to address:**
MaskedInput widget implementation — define the raw-space cursor invariant before writing any cursor movement code.

---

### Pitfall 9: MaskedInput Accepting Characters in Separator Positions

**What goes wrong:**
User types `/` in the date field and it is accepted as input, shifting digits or corrupting the raw value. Or the cursor stops on separator characters during arrow-key navigation, requiring an extra keypress to skip over them.

**Why it happens:**
Naive cursor movement (cursor += 1 per keypress) does not distinguish between separator and editable positions in the display string. The cursor can land on a separator, and if the insertion logic checks the display cursor rather than the raw cursor, a separator position is treated as an editable slot.

**How to avoid:**
Cursor movement functions (move_left, move_right, insert_char, delete_char) must all operate in raw value space. Arrow keys skip over separators automatically because the display cursor is always derived from raw position. Reject any character that does not match the mask pattern for the current raw position (e.g., reject non-digit input when the mask expects `#`).

**Warning signs:**
The raw value contains separator characters (e.g., `/` in a date stored as raw). The raw length exceeds the expected maximum. Snapshot tests show display values like `1//2/2026`.

**Phase to address:**
MaskedInput widget implementation — write unit tests for every cursor movement and insertion case before integrating with the render pipeline.

---

### Pitfall 10: crates.io Publish Fails Due to Workspace README Path

**What goes wrong:**
`cargo publish` succeeds locally but the crates.io page shows no README. The Cargo.toml for `textual-rs` has `readme = "../../README.md"` — a path relative to the crate root pointing to the workspace root. When crates.io packages the crate, it only includes files within the crate's directory tree. The `../../README.md` path resolves outside the package directory and is silently omitted.

**Why it happens:**
crates.io packages are created from the crate directory subtree. Files above the crate root are not included. The `readme` field must point to a file that is either inside the crate directory or specified as `readme = "README.md"` with a copy of the README inside `crates/textual-rs/`.

**How to avoid:**
Copy or symlink `README.md` into `crates/textual-rs/README.md` and update the `readme` field to `readme = "README.md"`. Verify with `cargo package --list` before publishing — the listed files must include `README.md`. The dry-run `cargo publish --dry-run` does not catch this because it does not verify the README is reachable on crates.io.

**Warning signs:**
`cargo package --list` output does not include `README.md`. After a test publish to a staging registry (or after the real publish), the crates.io page shows "No README found".

**Phase to address:**
crates.io publish phase — check with `cargo package --list` before the first real publish.

---

### Pitfall 11: Semver-Breaking Changes Without a Major Version Bump

**What goes wrong:**
A widget's `render` or `on_event` signature changes between patch versions, breaking downstream users who implemented the `Widget` trait.

**Why it happens:**
The `Widget` trait is the primary extension point for user code. Any change to its method signatures, associated types, or default implementations is a semver-breaking change. During rapid widget addition, it is tempting to add a new required method (e.g., `fn can_dismiss(&self) -> bool`) to the trait rather than providing a default implementation.

**How to avoid:**
Every new method added to the `Widget` trait must have a default implementation. Use `cargo semver-checks` before every publish to catch accidental breaking changes. Since the crate is pre-1.0 (0.x), breaking changes are allowed on minor version bumps per semver convention, but downstream users still expect stability.

**Warning signs:**
`cargo semver-checks` reports a breaking change. Downstream users open issues about compile errors after a minor version bump.

**Phase to address:**
crates.io publish phase and any phase that modifies the Widget trait.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Synchronous `fs::read_dir` in DirectoryTree | Simpler code path | UI freeze on large/slow directories | Never — use worker from the start |
| Storing toast timers as raw `spawn_local` handles outside the worker system | Faster to write | Timers survive unmount, stale WidgetId risk | Never — use worker API |
| Maintaining cursor position in display space for MaskedInput | Fewer conversions per keystroke | Drift bugs on every Backspace/Delete | Never — raw space is the invariant |
| Reusing `active_overlay` for toasts | No new data structure needed | Only one toast visible at a time, second notify silently replaces first | Never — toasts need their own layer |
| Skipping `cargo package --list` check before publish | Saves one step | Publish with missing README, no recovery except yank-and-republish | Never — takes 5 seconds |
| Adding required methods to `Widget` trait | Clean API | Breaks all downstream implementations | Never while 0.x — always provide default |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| tokio current_thread + `fs::read_dir` | Calling blocking I/O directly in widget methods | Wrap in `ctx.run_worker` — all blocking I/O belongs in workers |
| `DenseSlotMap` WidgetId lifetime | Holding a `WidgetId` across an unmount/remount cycle | Treat WidgetIds as single-use; timer targets should use the worker system |
| ratatui Clear widget | Forgetting to clear cells under a dismissed overlay | Render `Clear` over the overlay area in the same frame that removes the overlay |
| crossterm KeyEventKind on Windows | Processing KeyRelease events as new inputs | Already filtered in `app.rs` at line 338; new screens must not bypass this filter |
| walkdir with symlink following enabled | Infinite loop on cyclic symlinks | Never enable `follow_links(true)` without loop detection; prefer the default (links as leaves) |
| arboard clipboard on Linux | Requires an X11/Wayland display session | Already handled via `default-features = false`; verify on headless CI with `DISPLAY` set |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Expanding deep directory trees synchronously | UI freezes for hundreds of milliseconds | Use worker API for all directory reads | First time a user opens a directory with 1000+ entries |
| Re-cascading CSS on every render tick | CPU spikes, battery drain | Only re-cascade when `needs_full_sync` is set or a widget is dirty | With 50+ widgets all marked dirty every frame |
| Rendering all screen stack layers every frame | Wasted render time on hidden screens | Only render the top screen (current behavior) — verify this is maintained when stack code is added | With 3+ screens on the stack |
| Timer proliferation from rapid notifications | Multiple timers competing, race conditions | One timer per toast entry, cancelled on unmount via worker system | With notification-heavy applications |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Focus lost after modal dismiss | User must click to continue keyboard navigation | Restore focus to pre-push widget on every pop |
| Toast covers interactive content permanently | User cannot click widgets under the toast region | Anchor toasts to bottom-right, keep them narrow, auto-dismiss with visible countdown or fade |
| DirectoryTree shows no loading state | UI appears frozen during slow directory reads | Show `LoadingIndicator` as a child of the expanding node while the worker is in flight |
| MaskedInput cursor stopping on separator characters | User must press arrow key twice to skip past `/` | Skip separator positions automatically in all cursor movement functions |
| Screen stack depth unbounded | Deep navigation sequences consume memory | Consider a maximum stack depth warning in debug mode; enforce in tests |

---

## "Looks Done But Isn't" Checklist

- [ ] **Screen stack push/pop:** Focus is restored to the previously-focused widget after pop — verify with a snapshot test that Tabs correctly after dismiss.
- [ ] **Screen stack push/pop:** The new screen gets focus auto-assigned to its first focusable widget on push — verify with a snapshot test.
- [ ] **Screen stack render:** After pop, no artifacts from the dismissed screen appear in the snapshot — verify with a snapshot of the frame immediately after pop.
- [ ] **Toast auto-dismiss:** Dismiss timer is cancelled if the toast is manually dismissed before the timer fires — verify no panic with insta snapshot test.
- [ ] **Toast stacking:** Two rapid `notify()` calls both result in visible toasts stacked vertically — verify both texts appear in snapshot.
- [ ] **DirectoryTree symlinks:** Expanding a directory with a symlink cycle does not hang — verify with a test that creates a cyclic symlink fixture (on platforms supporting symlinks).
- [ ] **DirectoryTree performance:** Opening a directory with 500+ entries does not freeze the render tick — verify with a benchmark or timing test.
- [ ] **MaskedInput cursor:** Backspace at every position produces a correct raw value — verify with property tests over all cursor positions.
- [ ] **MaskedInput separators:** Arrow keys never stop on separator characters — verify by checking that cursor_pos in raw space always points to a placeholder slot.
- [ ] **crates.io package:** `cargo package --list` includes `README.md`, `LICENSE`, and all source files — verify before first publish.
- [ ] **crates.io semver:** `cargo semver-checks` reports no breaking changes vs. previous published version — add to release checklist.
- [ ] **Cross-platform KeyEventKind:** New screen push code does not re-introduce a path where KeyRelease events are processed — audit all event routing branches.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Focus lost after pop | LOW | Add `focus_history` field to `AppContext`, push before each `push_screen`, pop after each `pop_screen` |
| Events reaching wrong screen | MEDIUM | Add invariant assertion in debug builds; fix focus assignment in `push_screen` and deferred push drain |
| Render artifacts after pop | LOW | Add `terminal.clear()` or `Clear` widget render after any screen pop; caught by snapshot tests |
| Toast timer fires on unmounted widget | LOW | Switch to worker-based timers; cancel is automatic via existing `cancel_workers` in `unmount_widget` |
| Multiple toasts overlapping | MEDIUM | Introduce `toast_queue: Vec<ToastEntry>` on `AppContext`; refactor render pass to paint from queue |
| DirectoryTree symlink hang | HIGH | Requires detecting loop in tree traversal; add `visited_inodes: HashSet<u64>` during expansion |
| DirectoryTree blocking event loop | MEDIUM | Wrap `read_dir` call in `run_worker`; add loading indicator node |
| MaskedInput cursor drift | MEDIUM | Rewrite cursor to track in raw space; fix all movement functions; property-test all cases |
| README missing on crates.io | LOW | Yank the bad version; copy README into crate directory; republish with bumped patch version |
| Accidental semver break | HIGH | Major or minor version bump depending on crate maturity; notify users; document migration |
| macOS/Linux terminal differences | LOW-MEDIUM | Run CI on all three platforms; crossterm already handles most differences; test color output on macOS |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Focus not restored after screen pop | Screen stack implementation | Snapshot test: push modal, pop, verify Tab moves focus |
| Events routed to wrong screen | Screen stack implementation | Snapshot test: push modal, send key, verify background screen unchanged |
| Render artifacts after pop | Screen stack implementation | Snapshot test: push modal, pop, compare frame to baseline |
| Toast timer outlives widget | Toast widget implementation | Unit test: mount toast, unmount before timer, verify no side effects |
| Multiple toasts overlap | Toast widget implementation | Snapshot test: call notify twice, verify both visible and stacked |
| DirectoryTree symlink loop | DirectoryTree implementation | Fixture test with cyclic symlink (skip on platforms without symlink support) |
| DirectoryTree blocking event loop | DirectoryTree implementation | Timing test or worker integration test |
| MaskedInput cursor drift | MaskedInput implementation | Property test: all cursor positions, all deletion patterns |
| MaskedInput separator skip | MaskedInput implementation | Unit test: arrow key traversal over separator characters |
| README missing from crates.io package | crates.io publish phase | `cargo package --list` check before publish |
| Accidental Widget trait semver break | Every phase touching Widget trait | `cargo semver-checks` in publish phase |
| macOS/Linux terminal quirks | Cross-platform verification phase | CI matrix on macOS and Linux runners |

---

## Sources

- Ratatui buffer/diff behavior: https://ratatui.rs/concepts/rendering/under-the-hood/
- Ratatui artifact issue (tabs/list switching): https://forum.ratatui.rs/t/list-tabs-widgets-switching-keeps-character-artifact-upon-rendering/256
- Python Textual screen stack focus restore pattern: https://textual.textualize.io/guide/screens/
- Python Textual screen stack bug report (push/pop breaks ListView): https://github.com/Textualize/textual/issues/1632
- walkdir symlink loop detection: https://docs.rs/walkdir/ and https://github.com/BurntSushi/walkdir
- walkdir loop detection in cargo: https://github.com/rust-lang/cargo/pull/10214
- Masked input cursor position sync: https://giacomocerquone.com/blog/keep-input-cursor-still/
- crossterm Windows KeyEventKind duplicate events: https://github.com/ratatui/ratatui/issues/347
- crates.io publishing guide: https://doc.rust-lang.org/cargo/reference/publishing.html
- cargo-semver-checks: https://crates.io/crates/cargo-semver-checks
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- tokio task cancellation patterns: https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/
- Textual toast positioning discussion: https://github.com/Textualize/textual/discussions/3541
- Codebase audit: `crates/textual-rs/src/app.rs`, `widget/tree.rs`, `widget/context.rs`

---
*Pitfalls research for: textual-rs v1.3 — widget parity and screen stack*
*Researched: 2026-03-26*
