---
status: testing
phase: full-project
source: [01-SUMMARY through 05-04-SUMMARY]
started: 2026-03-26T09:05:04-07:00
updated: 2026-03-26T09:45:00-07:00
---

## Current Test

number: 21
name: Accessibility — Click-to-Interact Widgets
expected: |
  Checkbox, Switch, RadioButton, and Button should respond to mouse click directly (toggle/activate on click without requiring a separate spacebar press). Click alone should be sufficient for interaction.
awaiting: user response

## Tests

### 1. Demo App Launches and Shows Tabs
expected: Terminal enters alternate screen. Dark-themed layout with 4 tabs (Inputs/Display/Layout/Interactive), Header, Footer with key hints.
result: pass

### 2. Demo Tab Navigation
expected: Tabs switch content when focused and Right/Left arrows pressed. Each tab shows different widget content.
result: pass

### 3. Demo Inputs Tab — Input Widget
expected: Text input with placeholder, cursor navigation, Backspace/Delete.
result: pass

### 4. Demo Inputs Tab — Checkbox and Switch
expected: Checkbox [X]/[ ] and Switch ━━━◉/◉━━━ toggle with Space/Enter when focused.
result: issue
reported: "After switching tabs and switching back, focus cycling goes through ALL widgets in the tree including widgets on non-visible tabs. Cannot reliably Tab to checkbox/switch on the current tab because hidden tab widgets are in the focus order. When finally on the checkbox, Space/Enter should toggle but focus doesn't land predictably."
severity: major

### 5. Demo Inputs Tab — RadioSet
expected: Radio buttons (●)/( ) show all options. Selecting one deselects others.
result: pass

### 6. Demo Display Tab — ProgressBar and Sparkline
expected: ProgressBar (filled bar) and Sparkline (block chart) visible on Display tab.
result: pass

### 7. Demo Display Tab — Markdown Rendering
expected: Markdown renders headings, bold text, and content.
result: pass

### 8. Demo Layout Tab — Nested Layouts
expected: Vertical/Horizontal containers visible with nested content.
result: issue
reported: "Layout tab shows labels 'Horizontal container (3 panels)' and 'Vertical container (2 rows)' and a Collapsible, but the nested layout panels don't render visible child content. Only the section labels and collapsible header are visible — the actual arranged widgets inside are either missing or not rendered."
severity: minor

### 9. Demo Interactive Tab — DataTable
expected: Columnar data with header, Up/Down navigation, s-to-sort.
result: pass

### 10. Demo Interactive Tab — Tree View
expected: Hierarchical nodes with guide chars, expand/collapse.
result: pass

### 11. Demo Interactive Tab — ListView
expected: Selectable items with scrollbar, Up/Down/Enter.
result: pass

### 12. Focus Cycling (Tab/Shift+Tab)
expected: Tab cycles through focusable widgets with visible focus indicator.
result: issue
reported: "Focus cycles through ALL widgets in the arena including widgets on non-visible TabbedContent panes. When on the Display tab, Tabbing reaches Input/Checkbox/RadioButton widgets from the Inputs tab that aren't visible. Focus should only cycle through widgets on the currently active pane."
severity: blocker

### 13. Command Palette (Ctrl+P)
expected: Overlay with searchable commands, fuzzy filter, Esc to dismiss.
result: pass

### 14. IRC Demo — Full Layout
expected: Weechat-style with header, channel list, chat area, user list, input bar.
result: pass

### 15. IRC Demo — Typing and Sending Messages
expected: Type in input bar, Enter sends, message appears in chat.
result: pass

### 16. IRC Demo — Channel Switching
expected: Navigate channels, switch active channel.
result: skipped
reason: Cannot verify channel switching behavior programmatically in tmux — focus order doesn't reliably land on channel list. Requires mouse click or more complex tab navigation.

### 17. Footer Key Hints
expected: Footer shows current focused widget's key bindings. Updates on focus change.
result: pass

### 18. Window Resize
expected: Layout re-flows on terminal resize. No crash.
result: pass

### 19. Clean Exit
expected: q/Ctrl+C exits cleanly. Terminal restored.
result: pass

### 20. Tutorial Examples Build and Run
expected: All 5 tutorials compile.
result: pass

### 21. Accessibility — Click-to-Interact Widgets
expected: Checkbox, Switch, RadioButton respond to mouse click directly.
result: blocked
blocked_by: other
reason: tmux cannot inject mouse click events. However, user reports that clicking requires a subsequent spacebar press — this is likely because mouse click dispatches to the widget (setting focus) but doesn't trigger the toggle action. The on_event handler for MouseEvent::Down should trigger the same action as Space/Enter.

## Summary

total: 21
passed: 13
issues: 3
pending: 0
skipped: 1
blocked: 1

## Gaps

- truth: "Focus cycles only through visible widgets on the active tab pane"
  status: failed
  reason: "Focus cycles through ALL widgets in the arena including non-visible pane children. TabbedContent mounts all panes at compose time but only renders the active one — the widget tree still contains all panes' children as focusable."
  severity: blocker
  test: 12
  artifacts:
    - path: "crates/textual-rs/src/widget/tabs.rs"
      issue: "TabbedContent composes all panes into widget tree; inactive pane children remain focusable"
    - path: "crates/textual-rs/src/widget/tree.rs"
      issue: "advance_focus walks all children regardless of visibility/active-pane state"
  missing:
    - "TabbedContent should only mount the active pane's widget subtree, or advance_focus should skip widgets whose ancestor pane is inactive"
    - "On tab switch, unmount old pane children and compose/mount new pane children"

- truth: "Checkbox and Switch toggle with Space/Enter when focused"
  status: failed
  reason: "Cannot reliably Tab to checkbox/switch because focus order includes non-visible widgets from other tabs. When focus does land on them, the toggle may work, but reaching them is broken by the focus cycling bug (Gap 1)."
  severity: major
  test: 4
  artifacts:
    - path: "crates/textual-rs/src/widget/checkbox.rs"
    - path: "crates/textual-rs/src/widget/switch.rs"
  missing:
    - "Fix focus cycling (Gap 1) first, then verify toggle works"

- truth: "Layout tab shows nested Vertical/Horizontal containers with visible child content"
  status: failed
  reason: "Layout tab shows container labels and Collapsible but nested children don't render visible content inside the layout panels."
  severity: minor
  test: 8
  artifacts:
    - path: "crates/textual-rs/examples/demo.rs"
      issue: "Layout tab pane may not have enough child widgets with visible content to demonstrate layout"
  missing:
    - "Add visible child widgets (Labels, Placeholders) inside the Horizontal/Vertical containers on the Layout tab"

- truth: "Mouse click on Checkbox/Switch/RadioButton directly toggles without needing spacebar"
  status: failed
  reason: "User reported: clicking checkbox requires subsequent spacebar press. Mouse click likely dispatches focus but doesn't fire toggle action. on_event for MouseEvent::Down should trigger the same action as Space keybinding."
  severity: major
  test: 21
  artifacts:
    - path: "crates/textual-rs/src/widget/checkbox.rs"
      issue: "on_event doesn't handle MouseEvent::Down to trigger toggle"
    - path: "crates/textual-rs/src/widget/switch.rs"
      issue: "Same — no mouse click handler"
    - path: "crates/textual-rs/src/widget/radio.rs"
      issue: "Same — no mouse click handler"
    - path: "crates/textual-rs/src/widget/button.rs"
      issue: "Same — no mouse click handler for Pressed"
  missing:
    - "Add on_event handler for MouseEvent::Down that triggers the same action as Space/Enter keybinding"
    - "Mouse click should both set focus AND trigger the widget action in one click"
