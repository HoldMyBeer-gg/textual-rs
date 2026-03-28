---
phase: 10
slug: platform-verification-and-publish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo doc --no-deps` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo doc --no-deps`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | CI fix | 1 | PLATFORM-01 | CI | `act -j test` / push to GH | ❌ W0 | ⬜ pending |
| 10-01-02 | rustdoc | 1 | PUBLISH-02 | lint | `RUSTDOCFLAGS="-D missing_docs" cargo doc --no-deps` | ✅ | ⬜ pending |
| 10-02-01 | manifest | 2 | PUBLISH-01 | dry-run | `cargo package --list` | ✅ | ⬜ pending |
| 10-02-02 | publish | 3 | PUBLISH-03 | manual | `cargo publish --dry-run` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `.github/workflows/ci.yml` — updated with correct action and 3-platform matrix

*Existing `cargo test` infrastructure covers unit/doc test verification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| crates.io publish succeeds | PUBLISH-03 | Requires live crates.io API token | `cargo publish` with `CARGO_REGISTRY_TOKEN` set |
| `cargo add textual-rs` resolves | PUBLISH-03 | Post-publish validation | Run in scratch project after publish |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
