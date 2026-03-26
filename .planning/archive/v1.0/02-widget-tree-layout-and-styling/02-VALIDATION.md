---
phase: 2
slug: widget-tree-layout-and-styling
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `crates/textual-rs/Cargo.toml` |
| **Quick run command** | `cargo test --lib -p textual-rs` |
| **Full suite command** | `cargo test -p textual-rs` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -p textual-rs`
- **After every plan wave:** Run `cargo test -p textual-rs`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | TREE-01 | unit | `cargo test widget_arena` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | TREE-02 | unit | `cargo test widget_trait` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | TREE-03 | unit | `cargo test screen_stack` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | TREE-04 | unit | `cargo test compose` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | TREE-05 | unit | `cargo test focus` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | LAYOUT-01 | unit | `cargo test taffy_bridge` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | LAYOUT-02,03 | unit | `cargo test layout_containers` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 1 | LAYOUT-04 | unit | `cargo test dock_layout` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 1 | LAYOUT-05,06 | unit | `cargo test sizing` | ❌ W0 | ⬜ pending |
| 02-02-05 | 02 | 1 | LAYOUT-07 | unit | `cargo test dirty_flags` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 2 | CSS-01 | unit | `cargo test tcss_parser` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 2 | CSS-02,03 | unit | `cargo test selector_matching` | ❌ W0 | ⬜ pending |
| 02-03-03 | 03 | 2 | CSS-04 | unit | `cargo test inline_styles` | ❌ W0 | ⬜ pending |
| 02-03-04 | 03 | 2 | CSS-05 | unit | `cargo test pseudo_classes` | ❌ W0 | ⬜ pending |
| 02-03-05 | 03 | 2 | CSS-06,07,08 | unit | `cargo test css_properties` | ❌ W0 | ⬜ pending |
| 02-03-06 | 03 | 2 | CSS-09 | unit | `cargo test default_css` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/textual-rs/tests/` — integration test directory
- [ ] Test modules within `src/` for unit tests (inline `#[cfg(test)]` modules)
- [ ] `ratatui::backend::TestBackend` already available from Phase 1

*Existing infrastructure covers test framework. Test files created per-task.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual rendering quality | LAYOUT-04, CSS-08 | Border rendering quality requires visual inspection | Run `cargo run --example demo` and verify dock layout, border styles render correctly |
| Resize responsiveness | LAYOUT-01 | Terminal resize is interactive | Resize terminal window during `cargo run --example demo` and verify layout recomputes |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
