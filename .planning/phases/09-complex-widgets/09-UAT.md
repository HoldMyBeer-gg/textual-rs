---
status: testing
phase: 09-complex-widgets
source: [09-01-SUMMARY.md, 09-02-SUMMARY.md, 09-03-SUMMARY.md]
started: 2026-03-27T21:30:00Z
updated: 2026-03-27T21:30:00Z
---

## Current Test

<!-- OVERWRITE each test - shows where we are -->

number: 1
name: All phase 09 unit tests pass
expected: |
  `cargo test` passes all 34 new unit tests:
  15 masked_input tests, 10 directory_tree tests, 9 toast tests.
  No existing tests regressed.
awaiting: user response

## Tests

### 1. All phase 09 unit tests pass
expected: `cargo test` passes all 34 new unit tests: 15 masked_input tests, 10 directory_tree tests, 9 toast tests. No existing tests regressed.
result: [pending]

### 2. MaskedInput: digit slot rejects non-digits
expected: Creating `MaskedInput::new("##/##/####")` and calling `on_char('a')` leaves raw_value empty (letter rejected). Calling `on_char('1')` accepts the char and raw_value becomes "1".
result: [pending]

### 3. MaskedInput: separators auto-insert in display string
expected: After typing "01" into `##/##/####` mask, `build_display()` returns "01/  /    " — the slash appears automatically without the user typing it.
result: [pending]

### 4. MaskedInput: cursor skips separator on Right arrow
expected: After typing "01" (cursor at raw pos 2, which is display col 2), pressing Right moves cursor to raw pos 2 display col 3 — it visually lands after the "/" separator, not on it.
result: [pending]

### 5. MaskedInput: case modifiers transform input
expected: `MaskedInput::new(">AA")` (uppercase modifier) — typing 'a' stores 'A' in raw_value. `MaskedInput::new("<AA")` — typing 'A' stores 'a'.
result: [pending]

### 6. DirectoryTree: widget constructable with path
expected: `DirectoryTree::new("/some/path")` compiles and creates a widget. The root TreeNode label matches the directory name. A "Loading..." placeholder child is present so the root appears expandable.
result: [pending]

### 7. DirectoryTree: hidden files filtered by default
expected: `read_directory_for_tree("/tmp", false, &HashSet::new())` excludes entries starting with '.' (e.g. `.git`, `.bashrc`). With `show_hidden=true`, dot-entries are included.
result: [pending]

### 8. DirectoryTree: cycle detection prevents re-expansion
expected: `cycle_detection_root_in_visited` test passes — when root canonical path is already in visited set, `should_expand` returns false and no worker spawns (prevents infinite loops through symlinks/junctions).
result: [pending]

### 9. Toast: ctx.toast() adds entry to stack
expected: `AppContext::new()` followed by `ctx.toast("hello", ToastSeverity::Info, 3000)` results in `ctx.toast_entries.borrow().len() == 1`. The entry has `message == "hello"`, `severity == Info`, `timeout_ms == 3000`, `elapsed_ticks == 0`.
result: [pending]

### 10. Toast: max 5 cap drops oldest
expected: Pushing 6 toasts via `push_toast()` results in exactly 5 entries. The first toast (oldest) is dropped; entries 2–6 remain. `push_toast_overflow_drops_oldest` unit test confirms this.
result: [pending]

### 11. Toast: persistent toasts never expire
expected: `tick_toasts()` on a toast with `timeout_ms == 0` does NOT increment `elapsed_ticks` and does NOT remove it. It survives any number of ticks. `tick_toasts_does_not_remove_persistent_toasts` confirms this.
result: [pending]

### 12. Toast: severity colors correct
expected: `render_toasts()` applies `theme.primary` color to Info toasts, `theme.warning` to Warning, `theme.error` to Error. Each toast shows its symbol: 'i' for Info, '!' for Warning, 'x' for Error.
result: [pending]

### 13. Toast: z-order under CommandPalette
expected: In `render_widget_tree`, `render_toasts()` is called BEFORE `active_overlay` paint. This means if CommandPalette is open, it renders on top of toasts (toasts do not cover the palette).
result: [pending]

## Summary

total: 13
passed: 0
issues: 0
pending: 13
skipped: 0

## Gaps

[none yet]
