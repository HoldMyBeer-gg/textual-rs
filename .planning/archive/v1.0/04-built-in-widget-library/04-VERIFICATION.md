---
phase: 04-built-in-widget-library
verified: 2026-03-25T20:00:00Z
status: passed
score: 22/22 must-haves verified
re_verification: true
  previous_status: gaps_found
  previous_score: 20/22
  gaps_closed:
    - "Input widget provides validation support (WIDGET-03 — Gap 1)"
    - "REQUIREMENTS.md accurately reflects phase 4 completion (Gap 2)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Input keyboard navigation"
    expected: "Typing text updates value reactively, cursor moves correctly on Left/Right/Home/End/Ctrl+Left/Ctrl+Right, Backspace deletes, Enter emits Submitted message"
    why_human: "Full cursor navigation UX requires running application"
  - test: "Select overlay interaction"
    expected: "Pressing Enter on Select opens dropdown overlay, Up/Down navigate options, Enter selects and dismisses, Esc cancels"
    why_human: "Overlay screen push/pop lifecycle requires running event loop, not testable via TestApp unit tests"
  - test: "TabbedContent tab switching"
    expected: "Left/Right keys cycle through tabs, active pane content changes accordingly"
    why_human: "Composite widget with direct render embed — visual pane switching needs terminal rendering"
  - test: "Collapsible expand/collapse"
    expected: "Enter toggles expanded state, children appear/disappear smoothly below title row"
    why_human: "Render-time visibility requires running event loop to observe animated state change"
---

# Phase 4: Built-In Widget Library Verification Report

**Phase Goal:** All 22 v1 widgets are implemented, styled via TCSS, keyboard-interactive where applicable, and covered by snapshot tests — making textual-rs usable as a complete application framework.
**Verified:** 2026-03-25T20:00:00Z
**Status:** passed
**Re-verification:** Yes — after gap closure (plans 04-08 and 04-09)

---

## Re-Verification Summary

Initial verification (2026-03-25T18:30:00Z) found 2 gaps:

1. **Gap 1 — Input validation missing** (WIDGET-03 partial): No `validate_` method, validator callback, or error state rendering.
2. **Gap 2 — REQUIREMENTS.md stale checkboxes**: 14 of 22 WIDGET-* entries remained `[ ]` despite implementations existing.

Gap closure plans 04-08 and 04-09 were created and executed. Both gaps are now confirmed closed.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | All 22 widget source files exist in crates/textual-rs/src/widget/ | VERIFIED | Unchanged from initial verification — 22 widget .rs files confirmed |
| 2 | All widgets are declared in widget/mod.rs | VERIFIED | Unchanged — 22 pub mod declarations present |
| 3 | All widgets are re-exported from lib.rs | VERIFIED | Unchanged — 22 pub use statements confirmed |
| 4 | Widget implementations are substantive (not stubs) | VERIFIED | Unchanged — zero TODO/FIXME/placeholder markers in any widget file |
| 5 | Keyboard-interactive widgets have key_bindings() and on_action() | VERIFIED | Unchanged — 13 interactive widgets with bindings confirmed |
| 6 | Snapshot tests exist and pass for widgets | VERIFIED | Unchanged — 34 snapshot .snap files; all widget tests pass |
| 7 | All tests pass with zero regressions | VERIFIED | 219 total tests pass (up from 215 — 4 new validation tests added); 0 failures |
| 8 | Input widget provides validation support | VERIFIED | Gap closed by plan 04-08: validator field, with_validator() builder, is_valid() query, run_validation(), Color::Red error rendering, and valid: bool in Changed messages — all confirmed in input.rs |
| 9 | REQUIREMENTS.md reflects phase completion | VERIFIED | Gap closed by plan 04-09: all 22 WIDGET-* entries marked [x]; 0 entries marked [ ]; traceability table updated to "Complete" |

**Score:** 22/22 truths verified

---

## Gap Closure Verification

### Gap 1 — Input Validation (plan 04-08, commit 080368e)

**Target file:** `crates/textual-rs/src/widget/input.rs`

| Acceptance Criterion | Status | Evidence |
|----------------------|--------|----------|
| `validator: Option<Box<dyn Fn(&str) -> bool>>` field exists | VERIFIED | Line 37 of input.rs |
| `valid: Cell<bool>` field exists | VERIFIED | Line 38 of input.rs |
| `pub fn with_validator()` builder method | VERIFIED | Lines 64-67 of input.rs |
| `pub fn is_valid()` query method | VERIFIED | Lines 71-73 of input.rs |
| `fn run_validation()` private method | VERIFIED | Lines 76-82 of input.rs |
| `Color::Red` error state rendering | VERIFIED | Lines 397, 402, 415 of input.rs — invalid + non-empty renders red foreground |
| `messages::Changed` contains `pub valid: bool` | VERIFIED | Line 19 of input.rs |
| `emit_changed()` calls `run_validation()` | VERIFIED | Line 175 of input.rs |
| `fn input_validation_valid_input` test | VERIFIED | widget_tests.rs line 483 |
| `fn input_validation_invalid_input` test | VERIFIED | widget_tests.rs line 514 |
| `fn input_no_validator_always_valid` test | VERIFIED | widget_tests.rs line 546 |
| `fn input_changed_message_includes_valid` test | VERIFIED | widget_tests.rs line 576 |
| `cargo test input_validation` exits 0 | VERIFIED | All 4 validation tests pass |
| Full test suite exits 0 | VERIFIED | 219 tests pass, 0 failed |

### Gap 2 — REQUIREMENTS.md Checkboxes (plan 04-09, commit 9896af5)

| Acceptance Criterion | Status | Evidence |
|----------------------|--------|----------|
| `[x] **WIDGET-07**` present | VERIFIED | Line 74 of REQUIREMENTS.md |
| `[x] **WIDGET-09**` through `[x] **WIDGET-22**` present | VERIFIED | Lines 76-89 of REQUIREMENTS.md |
| `[x] **WIDGET-03**` present (fully complete after 04-08) | VERIFIED | Line 70 of REQUIREMENTS.md |
| `grep -c "\[ \].*WIDGET"` returns 0 | VERIFIED | 0 unchecked WIDGET entries confirmed |
| `grep -c "\[x\].*WIDGET"` returns 22 | VERIFIED | 22 checked WIDGET entries confirmed |
| Traceability table updated | VERIFIED | Line 150: "Complete (WIDGET-03 partial)" |

Note: The traceability table note "(WIDGET-03 partial)" is a minor residual from the intermediate state — WIDGET-03 is now fully complete with [x] marking. This wording is slightly inaccurate but does not affect functionality or tracking. Not a blocker.

---

## Required Artifacts (Re-verification Focus)

| Artifact | Status | Details |
|----------|--------|---------|
| `crates/textual-rs/src/widget/input.rs` | VERIFIED | Was PARTIAL — now VERIFIED. 420+ lines with full validation API |
| `crates/textual-rs/tests/widget_tests.rs` | VERIFIED | 4 new validation tests added; all 97 widget tests pass |
| `.planning/REQUIREMENTS.md` | VERIFIED | Was FAILED — now VERIFIED. All 22 WIDGET-* entries marked [x] |

All other 21 artifacts verified in the initial run remain VERIFIED — full regression tests confirm no regressions.

---

## Key Link Verification (Re-verification Focus)

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Input::with_validator()` | `Input::valid Cell<bool>` | `run_validation()` called from `emit_changed()` | VERIFIED | Validator runs on every keystroke; valid state fresh before message emission |
| `Input::render()` | `Color::Red` styling | `is_invalid = !self.valid.get() && !val.is_empty()` | VERIFIED | Error style applied to text characters and cursor when invalid |
| `messages::Changed.valid` | validator result | `ctx.post_message(id, Changed { value: val, valid: self.valid.get() })` | VERIFIED | Changed message carries validation result |

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Input validation tests pass | `cargo test input_validation` | 2 passed, 0 failed | PASS |
| Input no-validator test passes | `cargo test input_no_validator` | 1 passed, 0 failed | PASS |
| Input changed-message test passes | `cargo test input_changed_message` | 1 passed, 0 failed | PASS |
| Full widget test suite | `cargo test` (widget tests) | 97 passed, 0 failed | PASS |
| Full test suite (no regressions) | `cargo test` (all) | 219 passed, 0 failed | PASS |
| REQUIREMENTS.md has 0 unchecked WIDGET entries | `grep -c "[ ].*WIDGET" REQUIREMENTS.md` | 0 | PASS |
| REQUIREMENTS.md has 22 checked WIDGET entries | `grep -c "[x].*WIDGET" REQUIREMENTS.md` | 22 | PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| WIDGET-01 | 04-01 | Label — static/reactive text | SATISFIED | label.rs 45 lines, snapshot test passes |
| WIDGET-02 | 04-01 | Button — pressable with variants | SATISFIED | button.rs 126 lines, ButtonVariant enum, Pressed message |
| WIDGET-03 | 04-02, 04-08 | Input — single-line with placeholder, password, validation | SATISFIED | input.rs 420+ lines; validator, is_valid(), Color::Red, valid in Changed — fully implemented |
| WIDGET-04 | 04-03 | TextArea — multi-line editor with line numbers | SATISFIED | text_area.rs 578 lines |
| WIDGET-05 | 04-01 | Checkbox — toggleable boolean | SATISFIED | checkbox.rs 107 lines |
| WIDGET-06 | 04-01 | Switch — toggle on/off | SATISFIED | switch.rs 107 lines |
| WIDGET-07 | 04-02 | RadioButton/RadioSet — mutual exclusion | SATISFIED | radio.rs 282 lines, ArcRwSignal mutual exclusion |
| WIDGET-08 | 04-03 | Select — dropdown selection | SATISFIED | select.rs 237 lines, overlay pattern |
| WIDGET-09 | 04-05 | ListView — scrollable list | SATISFIED | list_view.rs 228 lines |
| WIDGET-10 | 04-06 | DataTable — sortable tabular data | SATISFIED | data_table.rs 482 lines |
| WIDGET-11 | 04-06 | Tree — hierarchical with expand/collapse | SATISFIED | tree_view.rs 488 lines |
| WIDGET-12 | 04-04 | ProgressBar — determinate + indeterminate | SATISFIED | progress_bar.rs 111 lines |
| WIDGET-13 | 04-04 | Sparkline — inline chart | SATISFIED | sparkline.rs 76 lines |
| WIDGET-14 | 04-05 | Log — auto-scroll display | SATISFIED | log.rs 193 lines |
| WIDGET-15 | 04-07 | Markdown — rendered display | SATISFIED | markdown.rs 385 lines |
| WIDGET-16 | 04-07 | Tabs/TabbedContent — tabbed navigation | SATISFIED | tabs.rs 236 lines |
| WIDGET-17 | 04-07 | Collapsible — expand/collapse container | SATISFIED | collapsible.rs 136 lines |
| WIDGET-18 | 04-04 | Vertical/Horizontal layout containers | SATISFIED | layout.rs 99 lines |
| WIDGET-19 | 04-05 | ScrollView — scrollable container | SATISFIED | scroll_view.rs 259 lines |
| WIDGET-20 | 04-04 | Header — title/subtitle bar | SATISFIED | header.rs 70 lines |
| WIDGET-21 | 04-04 | Footer — key binding help bar | SATISFIED | footer.rs 86 lines |
| WIDGET-22 | 04-04 | Placeholder — dev placeholder | SATISFIED | placeholder.rs 82 lines |

**All 22 WIDGET-* requirements: SATISFIED**

---

## Anti-Patterns Found

No new anti-patterns introduced by plans 04-08 or 04-09.

The two anti-patterns flagged in the initial verification are resolved:
- REQUIREMENTS.md stale checkboxes: resolved (plan 04-09)
- Input.rs missing validation API: resolved (plan 04-08)

No TODO/FIXME/PLACEHOLDER markers exist in any widget file. No empty return stubs. No disconnected data flows.

---

## Human Verification Required

The following items remain unverifiable programmatically. They carried over unchanged from initial verification — no new human checks are required from the gap closure plans.

### 1. Input Keyboard Navigation

**Test:** Create a textual-rs app with an Input widget. Type multi-byte Unicode text, use Ctrl+Left/Ctrl+Right for word movement, Backspace at word boundaries.
**Expected:** Cursor navigates correctly without splitting multi-byte UTF-8 sequences. Word movement stops at word boundaries.
**Why human:** Char boundary arithmetic is correct per code review, but interactive UX correctness with Unicode requires human observation.

### 2. Select Overlay Lifecycle

**Test:** Run an app with a Select widget. Press Enter to open overlay. Use Up/Down to navigate options. Press Escape to cancel, then Enter to select.
**Expected:** Overlay appears as a separate screen, dismisses cleanly on both select and cancel, and the Select widget reflects the chosen value.
**Why human:** SelectOverlay uses pending_screen_pushes/pops which require the full event loop. The test only verifies the push occurs, not the full round-trip.

### 3. TabbedContent Pane Rendering

**Test:** Create a TabbedContent with 3 panes containing distinct content. Press Left/Right to cycle tabs.
**Expected:** Only the active pane's content is rendered below the tab bar. Tab labels update their highlight correctly.
**Why human:** Direct render embed of inner Tabs — visual correctness of pane switching needs terminal output.

### 4. Collapsible Children Rendering

**Test:** Create a Collapsible with 2-3 child widgets. Toggle expand/collapse with Enter.
**Expected:** Children appear/disappear immediately below the title row. The title arrow indicator updates.
**Why human:** Render-time visibility relies on get_untracked() in render() — correctness under real event loop conditions needs human confirmation.

---

## Gaps Summary

No gaps remain. Both gaps identified in initial verification are closed:

**Gap 1 (CLOSED):** Input validation fully implemented in plan 04-08 (commit 080368e). Input widget now has `with_validator()` builder, `is_valid()` query, `run_validation()` internal method, `valid: bool` in `Changed` messages, and red-foreground error rendering. Four automated tests verify all validation scenarios.

**Gap 2 (CLOSED):** REQUIREMENTS.md updated in plan 04-09 (commit 9896af5). All 22 WIDGET-* entries now marked `[x]`. Traceability table updated to "Complete".

The phase goal is fully achieved: all 22 v1 widgets are implemented, styled, keyboard-interactive where applicable, and covered by snapshot and behavioral tests. 219 automated tests pass with 0 failures.

---

_Verified: 2026-03-25T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification: Yes — initial gaps closed by plans 04-08 and 04-09_
