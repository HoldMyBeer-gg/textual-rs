---
phase: 9
slug: complex-widgets
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (insta snapshot + unit) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test -p textual-rs 2>&1 \| tail -5` |
| **Full suite command** | `cargo test -p textual-rs -- --include-ignored 2>&1` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p textual-rs 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test -p textual-rs -- --include-ignored`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 0 | WIDGET-11 | unit | `cargo test masked_input` | ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 1 | WIDGET-11 | unit | `cargo test masked_input_cursor` | ❌ W0 | ⬜ pending |
| 09-01-03 | 01 | 1 | WIDGET-11 | unit | `cargo test masked_input_backspace` | ❌ W0 | ⬜ pending |
| 09-01-04 | 01 | 2 | WIDGET-11 | snapshot | `cargo test masked_input_render` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 0 | WIDGET-12 | unit | `cargo test directory_tree` | ❌ W0 | ⬜ pending |
| 09-02-02 | 02 | 1 | WIDGET-12 | unit | `cargo test directory_tree_worker` | ❌ W0 | ⬜ pending |
| 09-02-03 | 02 | 1 | WIDGET-12 | unit | `cargo test directory_tree_symlink` | ❌ W0 | ⬜ pending |
| 09-02-04 | 02 | 2 | WIDGET-12 | snapshot | `cargo test directory_tree_render` | ❌ W0 | ⬜ pending |
| 09-03-01 | 03 | 0 | WIDGET-13 | unit | `cargo test toast` | ❌ W0 | ⬜ pending |
| 09-03-02 | 03 | 1 | WIDGET-13 | unit | `cargo test toast_stack` | ❌ W0 | ⬜ pending |
| 09-03-03 | 03 | 1 | WIDGET-13 | unit | `cargo test toast_autodismiss` | ❌ W0 | ⬜ pending |
| 09-03-04 | 03 | 2 | WIDGET-13 | snapshot | `cargo test toast_render` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/textual-rs/tests/widget_tests.rs` — add test stubs for WIDGET-11, WIDGET-12, WIDGET-13
- [ ] `crates/textual-rs/src/widget/masked_input.rs` — file must exist (even empty struct) before cursor tests compile
- [ ] `crates/textual-rs/src/widget/directory_tree.rs` — file must exist before worker tests compile
- [ ] `crates/textual-rs/Cargo.toml` — add `walkdir = "2"` dependency

*If cargo test infrastructure already covers the phase: update existing test files only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Toast renders bottom-right in real terminal | WIDGET-13 | Snapshot can't verify terminal position | Run demo, visually confirm toast appears bottom-right |
| MaskedInput cursor visually skips separator | WIDGET-11 | Snapshot shows chars, not cursor position | Run demo with `##/##/####` mask, confirm cursor jumps over `/` |
| DirectoryTree cycles avoided on real NTFS junctions | WIDGET-12 | No NTFS junction fixture in CI | Manual test on Windows with mklink /J |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
