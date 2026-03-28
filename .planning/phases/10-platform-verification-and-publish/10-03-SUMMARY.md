---
phase: 10-platform-verification-and-publish
plan: 03
subsystem: infra
tags: [cargo, crates-io, publish, versioning, changelog]

# Dependency graph
requires:
  - phase: 10-01
    provides: CI green on Linux/macOS/Windows
  - phase: 10-02
    provides: Rustdoc coverage with deny(missing_docs)
  - phase: 10-04
    provides: Widget rustdoc coverage
provides:
  - Both crates at version 0.3.0 with correct crates.io metadata
  - CHANGELOG.md [0.3.0] release entry
  - cargo package --list clean for both crates (no broken paths, README included)
  - cargo publish --dry-run clean for textual-rs-macros
  - Full test suite passing at 0.3.0
  - Docs build clean with -D warnings
  - crates.io publish pending human action (token required)
affects: [crates-io, downstream-users, yubitui]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Publish macros crate first, then main crate (dependency ordering on crates.io)"

key-files:
  created: []
  modified:
    - crates/textual-rs-macros/Cargo.toml
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/src/lib.rs
    - CHANGELOG.md

key-decisions:
  - "cargo publish --dry-run fails for textual-rs when macros not on crates.io yet — this is expected, not a bug; resolves after publishing macros first"
  - "textual-rs-macros must be published before textual-rs due to version dependency"

patterns-established:
  - "Publish order: textual-rs-macros first, wait ~60s for index propagation, then textual-rs"

requirements-completed:
  - PUBLISH-01
  - PUBLISH-03

# Metrics
duration: 15min
completed: 2026-03-28
---

# Phase 10 Plan 03: Version Bump and Publish Prep Summary

**Both crates bumped to 0.3.0 with full crates.io metadata, CHANGELOG released, and packaging verified clean — awaiting human publish step with crates.io token**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-28T04:01:15Z
- **Completed:** 2026-03-28T04:16:00Z
- **Tasks:** 2 of 3 complete (Task 3 is human-action checkpoint)
- **Files modified:** 4

## Accomplishments
- Version bumped to 0.3.0 in both Cargo.toml files
- textual-rs-macros gained full crates.io metadata: description, license, repository, keywords, categories
- textual-rs macros dependency updated with explicit version = "0.3.0"
- CHANGELOG.md [Unreleased] promoted to [0.3.0] - 2026-03-28 with widget parity, screen stack, rustdoc, and CI entries
- lib.rs quick-start snippet updated to textual-rs = "0.3"
- cargo package --list clean for both crates (macros: 7 files; main: 100+ files including README.md)
- cargo publish --dry-run for textual-rs-macros exits 0
- Full test suite passes (all tests pass)
- RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace exits 0

## Task Commits

1. **Task 1: Fix Cargo manifests and bump versions to 0.3.0** - `68fcfa2` (feat)
2. **Task 2: Verify package and publish dry-run** - `ea1ac32` (chore)
3. **Task 3: Publish to crates.io** - PENDING (human-action checkpoint)

## Files Created/Modified
- `crates/textual-rs-macros/Cargo.toml` - version 0.1.0 -> 0.3.0, added description/license/repository/keywords/categories
- `crates/textual-rs/Cargo.toml` - version 0.1.0 -> 0.3.0, macros dep adds version = "0.3.0"
- `crates/textual-rs/src/lib.rs` - quick-start snippet updated from "0.2" to "0.3"
- `CHANGELOG.md` - [Unreleased] -> [0.3.0] - 2026-03-28, new [Unreleased] section added above

## Decisions Made
- `cargo publish --dry-run` for the main crate errors (not warns) when macros not on crates.io — this is inherent to cargo's dependency resolution and not fixable without publishing macros first. The macros dry-run passes cleanly, confirming the manifest is correct.
- Publish order is mandatory: macros first, ~60s propagation delay, then main crate.

## Deviations from Plan

None - plan executed exactly as written. The `cargo publish --dry-run -p textual-rs` failure is documented in the plan as expected behavior ("Note: This may warn that `textual-rs-macros@0.3.0` is not on crates.io yet").

## Issues Encountered
- `cargo publish --dry-run -p textual-rs` produces a hard error (not just a warning) because cargo verifies dependency existence against the crates.io index. This is inherent behavior and cannot be suppressed. The macros crate dry-run passes cleanly, confirming it is ready to publish.

## User Setup Required

**External service requires manual action.** Task 3 is a human-action checkpoint:

1. Ensure `CARGO_REGISTRY_TOKEN` is set (via env var or `cargo login`)
2. Publish macros crate first: `cargo publish -p textual-rs-macros`
3. Wait ~60 seconds for crates.io index propagation
4. Publish main crate: `cargo publish -p textual-rs`
5. Verify at https://crates.io/crates/textual-rs that version 0.3.0 appears
6. In a scratch directory, run `cargo add textual-rs` and confirm it resolves to 0.3.0

Token source: https://crates.io/settings/tokens -> New Token -> name "textual-rs-publish" -> scope: publish-new + publish-update

## Next Phase Readiness
- Both crates fully prepared and verified for publish
- After human publishes, the v1.3 milestone is complete
- REQUIREMENTS.md PUBLISH-01 and PUBLISH-03 are fulfilled once publish completes

## Self-Check: PASSED

- FOUND: .planning/phases/10-platform-verification-and-publish/10-03-SUMMARY.md
- FOUND: commit 68fcfa2 (feat: bump both crates to 0.3.0 and fix publish metadata)
- FOUND: commit ea1ac32 (chore: verify packaging and publish dry-run readiness)

---
*Phase: 10-platform-verification-and-publish*
*Completed: 2026-03-28*
