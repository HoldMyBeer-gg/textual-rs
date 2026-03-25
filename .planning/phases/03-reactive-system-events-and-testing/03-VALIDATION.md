---
phase: 3
slug: reactive-system-events-and-testing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + tokio::test for async |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p textual-rs --lib` |
| **Full suite command** | `cargo test -p textual-rs` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p textual-rs --lib`
- **After every plan wave:** Run `cargo test -p textual-rs`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | REACT-01 | unit | `cargo test reactive` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | REACT-02 | unit | `cargo test watch_` | ❌ W0 | ⬜ pending |
| 03-01-03 | 01 | 1 | REACT-03 | unit | `cargo test validate_` | ❌ W0 | ⬜ pending |
| 03-01-04 | 01 | 1 | REACT-04 | unit | `cargo test compute_` | ❌ W0 | ⬜ pending |
| 03-01-05 | 01 | 1 | REACT-05 | integration | `cargo test batching` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 1 | EVENT-01 | unit | `cargo test message` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 1 | EVENT-02 | unit | `cargo test on_event` | ❌ W0 | ⬜ pending |
| 03-02-03 | 02 | 1 | EVENT-03,04 | integration | `cargo test bubbling` | ❌ W0 | ⬜ pending |
| 03-02-04 | 02 | 1 | EVENT-05 | integration | `cargo test keyboard` | ❌ W0 | ⬜ pending |
| 03-02-05 | 02 | 1 | EVENT-06 | integration | `cargo test mouse` | ❌ W0 | ⬜ pending |
| 03-02-06 | 02 | 1 | EVENT-07 | unit | `cargo test key_binding` | ❌ W0 | ⬜ pending |
| 03-02-07 | 02 | 1 | EVENT-08 | unit | `cargo test timer` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 2 | TEST-01 | integration | `cargo test test_app` | ❌ W0 | ⬜ pending |
| 03-03-02 | 03 | 2 | TEST-02 | integration | `cargo test pilot` | ❌ W0 | ⬜ pending |
| 03-03-03 | 03 | 2 | TEST-03 | integration | `cargo test settle` | ❌ W0 | ⬜ pending |
| 03-03-04 | 03 | 2 | TEST-04 | snapshot | `cargo test snapshot` | ❌ W0 | ⬜ pending |
| 03-03-05 | 03 | 2 | TEST-05 | unit | `cargo test assert_buffer` | ❌ W0 | ⬜ pending |
| 03-03-06 | 03 | 2 | TEST-06 | proptest | `cargo test proptest` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/textual-rs/src/reactive/mod.rs` — reactive module skeleton
- [ ] `crates/textual-rs/src/event/message.rs` — Message trait skeleton
- [ ] `crates/textual-rs/src/testing/mod.rs` — TestApp/Pilot skeletons
- [ ] `reactive_graph` dependency added to Cargo.toml
- [ ] `insta` + `proptest` dev-dependencies added to Cargo.toml

*Existing infrastructure covers terminal management and layout tests from Phase 1-2.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Mouse click routing with real terminal | EVENT-06 | Requires real mouse input | Run demo, click on widgets, verify correct widget receives event |

*All other phase behaviors have automated verification via TestApp + Pilot.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
