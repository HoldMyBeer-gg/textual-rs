---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | None — cargo test discovers `#[test]` and `#[cfg(test)]` automatically |
| **Quick run command** | `cargo test -p textual-rs` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --workspace`
- **After every plan wave:** Run `cargo test -p textual-rs`
- **Before `/gsd:verify-work`:** Full suite must be green (`cargo test --workspace`)
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-T01 | 01-01 | 1 | FOUND-01 | build | `cargo build --workspace` | ❌ W0 | ⬜ pending |
| 1-01-T02 | 01-01 | 1 | FOUND-02 | build | `cargo build --workspace` | ❌ W0 | ⬜ pending |
| 1-01-T03 | 01-01 | 1 | FOUND-03 | integration | `cargo test -p textual-rs test_render` | ❌ W0 | ⬜ pending |
| 1-01-T04 | 01-01 | 1 | FOUND-04 | integration | `cargo test -p textual-rs test_app_run` | ❌ W0 | ⬜ pending |
| 1-02-T01 | 01-02 | 2 | FOUND-05 | integration | `cargo test -p textual-rs test_terminal_guard` | ❌ W0 | ⬜ pending |
| 1-02-T02 | 01-02 | 2 | FOUND-06 | integration | `cargo test -p textual-rs test_resize` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `Cargo.toml` (workspace root) — workspace definition, resolver = "2"
- [ ] `crates/textual-rs/Cargo.toml` — lib crate manifest with ratatui/crossterm/tokio/flume/anyhow deps
- [ ] `crates/textual-rs/src/lib.rs` — empty lib stubs (app, event, terminal modules)
- [ ] `crates/textual-rs/src/app.rs` — App struct stub with run() signature
- [ ] `crates/textual-rs/src/event.rs` — AppEvent enum stub
- [ ] `crates/textual-rs/src/terminal.rs` — TerminalGuard stub
- [ ] `crates/textual-rs/examples/demo.rs` — demo binary entry point stub
- [ ] `crates/textual-rs/tests/integration_test.rs` — TestBackend integration test stubs for FOUND-03 through FOUND-06

*All code is created from scratch — entire codebase is greenfield in Phase 1.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Alt-screen enters/exits correctly on real terminal | FOUND-05 | TestBackend cannot verify screen mode changes | Run `cargo run --example demo`, confirm TUI appears; press `q`, confirm shell restored |
| Cross-platform Windows output | FOUND-02 | CI cross-platform validation requires separate runners | Run `cargo build` on macOS and Linux (when available); confirm no platform branches in code |
| Panic restores terminal | FOUND-05 | Cannot force a real terminal panic in automated test | Add a temporary `panic!()` in demo, run, confirm shell is not broken after panic |
| Resize triggers re-render | FOUND-06 | TestBackend does not simulate real resize events | Run demo in a real terminal, resize window, confirm content redraws within one tick |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
