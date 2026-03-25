---
status: partial
phase: 01-foundation
source: [01-VERIFICATION.md]
started: 2026-03-24T00:00:00Z
updated: 2026-03-24T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Real Terminal End-to-End Smoke Test

Run `cargo run --example demo -p textual-rs` in a real terminal (Windows Terminal, iTerm2, or any POSIX terminal).

expected: The terminal clears and enters the alternate screen. A centered rounded-border box appears with title "textual-rs" and body text "Hello from textual-rs!". Resizing the window causes the box to re-center within the new dimensions without artifact. Pressing `q` exits cleanly — alternate screen leaves, original shell prompt is visible, cursor shown, raw mode off. Pressing `Ctrl+C` produces the same clean exit as `q`.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
