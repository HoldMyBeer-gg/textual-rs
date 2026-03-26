---
phase: 4
slug: built-in-widget-library
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + insta 1.46.3 snapshots + proptest 1.11.0 |
| **Config file** | None (standard Cargo test runner) |
| **Quick run command** | `cargo test --lib -q` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -q`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | WIDGET-01 | snapshot | `cargo test --test widget_tests label` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | WIDGET-02 | snapshot + interaction | `cargo test --test widget_tests button` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | WIDGET-03 | interaction | `cargo test --test widget_tests input` | ❌ W0 | ⬜ pending |
| 04-01-04 | 01 | 1 | WIDGET-04 | interaction | `cargo test --test widget_tests text_area` | ❌ W0 | ⬜ pending |
| 04-01-05 | 01 | 1 | WIDGET-05 | interaction | `cargo test --test widget_tests checkbox` | ❌ W0 | ⬜ pending |
| 04-01-06 | 01 | 1 | WIDGET-06 | interaction | `cargo test --test widget_tests switch` | ❌ W0 | ⬜ pending |
| 04-01-07 | 01 | 1 | WIDGET-07 | interaction | `cargo test --test widget_tests radio` | ❌ W0 | ⬜ pending |
| 04-01-08 | 01 | 1 | WIDGET-08 | interaction | `cargo test --test widget_tests select` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 2 | WIDGET-09 | interaction + snapshot | `cargo test --test widget_tests list_view` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 2 | WIDGET-10 | interaction + snapshot | `cargo test --test widget_tests data_table` | ❌ W0 | ⬜ pending |
| 04-02-03 | 02 | 2 | WIDGET-11 | interaction + snapshot | `cargo test --test widget_tests tree_view` | ❌ W0 | ⬜ pending |
| 04-02-04 | 02 | 2 | WIDGET-12 | snapshot | `cargo test --test widget_tests progress_bar` | ❌ W0 | ⬜ pending |
| 04-02-05 | 02 | 2 | WIDGET-13 | snapshot | `cargo test --test widget_tests sparkline` | ❌ W0 | ⬜ pending |
| 04-02-06 | 02 | 2 | WIDGET-14 | interaction | `cargo test --test widget_tests log` | ❌ W0 | ⬜ pending |
| 04-02-07 | 02 | 2 | WIDGET-15 | snapshot | `cargo test --test widget_tests markdown` | ❌ W0 | ⬜ pending |
| 04-02-08 | 02 | 2 | WIDGET-16 | interaction + snapshot | `cargo test --test widget_tests tabs` | ❌ W0 | ⬜ pending |
| 04-02-09 | 02 | 2 | WIDGET-17 | interaction | `cargo test --test widget_tests collapsible` | ❌ W0 | ⬜ pending |
| 04-02-10 | 02 | 2 | WIDGET-18 | snapshot | `cargo test --test widget_tests layout` | ❌ W0 | ⬜ pending |
| 04-02-11 | 02 | 2 | WIDGET-19 | interaction | `cargo test --test widget_tests scroll_view` | ❌ W0 | ⬜ pending |
| 04-02-12 | 02 | 2 | WIDGET-20 | snapshot | `cargo test --test widget_tests header` | ❌ W0 | ⬜ pending |
| 04-02-13 | 02 | 2 | WIDGET-21 | snapshot | `cargo test --test widget_tests footer` | ❌ W0 | ⬜ pending |
| 04-02-14 | 02 | 2 | WIDGET-22 | snapshot | `cargo test --test widget_tests placeholder` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/textual-rs/tests/widget_tests.rs` — interaction tests for 12 interactive widgets
- [ ] Cargo.toml additions: `pulldown-cmark`, `arboard` in `[dependencies]`

*Existing infrastructure covers test framework (cargo test runner already configured).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Select overlay visual rendering | WIDGET-08 | Overlay push/pop screen visuals hard to assert in headless | 1. Run demo app 2. Tab to Select widget 3. Press Enter 4. Verify overlay appears with options 5. Select option 6. Verify overlay closes |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
