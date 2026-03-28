---
plan: 10-01
phase: 10-platform-verification-and-publish
status: complete
tags: [ci, github-actions, rust-toolchain, docs-job, lint-job]
key_files:
  modified:
    - .github/workflows/ci.yml
    - crates/textual-rs/src/lib.rs
decisions:
  - "Use dtolnay/rust-toolchain@stable (not dtolnay/rust-action/setup@v1) — the old action path returns 404"
  - "Docs CI job added now with RUSTDOCFLAGS=-D warnings; will initially fail until Plan 02 adds rustdoc — expected and acceptable"
  - "Removed premature #![deny(missing_docs)] from lib.rs working tree; that attribute belongs in Plan 02 alongside the doc pass"
---

# Phase 10 Plan 01: Fix CI Action References and Add Docs/Lint Jobs — Summary

Fixed the broken GitHub Actions CI workflow and added docs and lint jobs. The CI was referencing `dtolnay/rust-action/setup@v1` which does not exist (returns 404); replaced with the correct `dtolnay/rust-toolchain@stable` across all three jobs.

## What was done

- Rewrote `.github/workflows/ci.yml` with three jobs: `test`, `docs`, `lint`
- `test` job: matrix over ubuntu-latest, windows-latest, macos-latest; uses `dtolnay/rust-toolchain@stable`; runs `cargo build --workspace` and `cargo test --workspace`; `fail-fast: false`
- `docs` job: ubuntu-latest only; `RUSTDOCFLAGS: "-D warnings"` at job level; runs `cargo doc --no-deps --workspace`; no cache (fast one-off)
- `lint` job: ubuntu-latest; `dtolnay/rust-toolchain@stable` with `components: clippy, rustfmt`; runs `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`; cargo cache included
- Removed uncommitted `#![deny(missing_docs)]` from `crates/textual-rs/src/lib.rs` (this was a premature working-tree change that broke `cargo test --workspace`; Plan 02 will re-add it alongside the full doc pass)
- Confirmed arboard::Clipboard is only used in runtime event handlers (input.rs, text_area.rs), never in test code — headless Linux CI is safe

## Verification

- `grep -c "dtolnay/rust-toolchain@stable" .github/workflows/ci.yml` = 3 (PASS)
- `grep -c "dtolnay/rust-action" .github/workflows/ci.yml` = 0 (PASS)
- `grep "RUSTDOCFLAGS" .github/workflows/ci.yml` = 1 match with `-D warnings` (PASS)
- `grep "cargo doc --no-deps --workspace" .github/workflows/ci.yml` = 1 match (PASS)
- File contains exactly 3 jobs: test, docs, lint (PASS)
- `grep -rn "arboard::Clipboard" crates/ --include="*.rs"` shows only runtime handlers, 0 test functions (PASS)
- `cargo test --workspace` passes locally (PASS)
- `cargo doc --no-deps --workspace` produces no errors (PASS)
- No tab characters in ci.yml (PASS)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed premature #![deny(missing_docs)] from lib.rs**
- **Found during:** Task 2 verification
- **Issue:** An uncommitted working-tree change had added `#![deny(missing_docs)]` to `crates/textual-rs/src/lib.rs`. This caused 319 compilation errors and prevented `cargo test --workspace` from passing — directly blocking the plan's acceptance criterion.
- **Fix:** Reverted the uncommitted change; the attribute was not in HEAD and its removal left lib.rs identical to HEAD. Plan 02 will re-add this attribute alongside the full documentation pass.
- **Files modified:** `crates/textual-rs/src/lib.rs` (reverted to HEAD state)
- **Commit:** 8a30e39 (no separate commit needed — the revert left no diff vs HEAD)
