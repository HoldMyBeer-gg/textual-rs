---
phase: 5
slug: developer-experience-and-polish
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-25
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + insta 1.46.3 for snapshots |
| **Config file** | none (workspace-level `rust-version = "1.88"` in Cargo.toml) |
| **Quick run command** | `cargo test -p textual-rs -- --test-output immediate 2>&1` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p textual-rs -- --test-output immediate 2>&1`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | DX-01 | unit (compile + runtime) | `cargo test -p textual-rs derive_widget` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | DX-01 | unit | `cargo test -p textual-rs derive_widget_focusable` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | DX-01 | unit | `cargo test -p textual-rs on_event_dispatch` | ❌ W0 | ⬜ pending |
| 05-01-04 | 01 | 1 | DX-01 | unit | `cargo test -p textual-rs keybinding_dispatch` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 1 | DX-02 | unit (tokio) | `cargo test -p textual-rs worker_result_delivered` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 1 | DX-02 | unit | `cargo test -p textual-rs worker_cancelled_on_unmount` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 1 | DX-03 | unit | `cargo test -p textual-rs notify_bubbles` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 1 | DX-03 | unit | `cargo test -p textual-rs post_message_target` | ❌ W0 | ⬜ pending |
| 05-04-01 | 04 | 2 | DX-04 | integration (TestApp) | `cargo test -p textual-rs command_palette_opens` | ❌ W0 | ⬜ pending |
| 05-04-02 | 04 | 2 | DX-04 | unit | `cargo test -p textual-rs command_palette_fuzzy_search` | ❌ W0 | ⬜ pending |
| 05-04-03 | 04 | 2 | DX-04 | integration | `cargo test -p textual-rs command_palette_dispatch` | ❌ W0 | ⬜ pending |
| 05-04-03 | 04 | 3 | DX-05 | smoke | `cargo doc --no-deps 2>&1 \| grep -E "warning\|error"` | N/A | ⬜ pending |
| 05-04-04 | 04 | 3 | DX-05 | compile smoke | `cargo build --example tutorial_01_hello` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/textual-rs-macros/` — entire new proc-macro crate directory (DX-01 prerequisite)
- [ ] `crates/textual-rs/tests/macro_tests.rs` — derive Widget, on, keybinding compile tests (DX-01)
- [ ] `crates/textual-rs/tests/worker_tests.rs` — Worker API async tests with tokio test runtime (DX-02)
- [ ] `crates/textual-rs/tests/command_palette_tests.rs` — palette open/search/dispatch integration (DX-04)

*Existing infrastructure covers DX-03 (notify/post_message) and DX-05 (docs/examples).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| rust-analyzer shows no red underlines for `#[derive(Widget)]` | DX-01 | IDE integration cannot be automated | Open example file in VS Code, verify zero diagnostics |
| Documentation guide is followable by new user | DX-05 | Subjective UX quality | Follow guide from scratch in clean environment |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-25
