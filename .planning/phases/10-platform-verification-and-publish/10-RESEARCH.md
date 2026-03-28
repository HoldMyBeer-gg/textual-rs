# Phase 10: Platform Verification and Publish - Research

**Researched:** 2026-03-27
**Domain:** Rust CI/CD, crates.io publishing, rustdoc documentation
**Confidence:** HIGH

## Summary

Phase 10 is an infrastructure and publishing phase — no new Rust feature code is written. The work
divides into four distinct tracks: (1) fixing the broken CI action reference so the existing
three-platform matrix actually passes, (2) documenting all 329 public API items that currently
fail `#![deny(missing_docs)]`, (3) fixing the Cargo manifest so `cargo package` succeeds for both
crates in the workspace, and (4) publishing both crates to crates.io in dependency order.

The good news is that the CI already has a three-platform test matrix (`ubuntu-latest`,
`windows-latest`, `macos-latest`) and all 471 tests pass locally on Windows. The bad news is the
CI references `dtolnay/rust-action/setup@v1` which does not exist (the real action is
`dtolnay/rust-toolchain@stable`), so the matrix has never successfully run on any platform.

Documentation is the largest task: `RUSTDOCFLAGS="-D missing_docs" cargo doc` emits 329 errors
across both crates, dominated by undocumented struct fields (146), enum variants (61), and modules
(58). The macros crate has only one error (crate-level doc). Publishing requires publishing
`textual-rs-macros` first (proc-macro dependency), then `textual-rs`, each with corrected
manifests and a version bump aligned to CHANGELOG (current Cargo.toml says `0.1.0` but the last
released version in CHANGELOG is `0.2.0`, and the lib.rs quick-start already says `"0.2"`).

**Primary recommendation:** Fix CI action first to verify platform compatibility, then add docs,
then fix manifests, then publish. Do not publish until all prior CI checks pass.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PLATFORM-01 | Library builds and all tests pass on macOS and Linux (CI verified) | CI matrix already targets all three platforms; the dtolnay action name is wrong and must be corrected. No platform-specific test failures expected once CI is fixed (all test logic is headless/in-process). |
| PUBLISH-01 | Library is published to crates.io with correct README, docs, and semver metadata | textual-rs Cargo.toml has README, keywords, categories; version is stale at 0.1.0 when CHANGELOG shows 0.2.0 released. Must bump to 0.3.0 (or 0.2.1 if only patch changes). Both crates need version alignment. |
| PUBLISH-02 | All public API items have rustdoc documentation | 329 missing_docs errors across both crates. Requires systematic doc pass before publish; `#![deny(missing_docs)]` should be added to lib.rs as enforcement. |
| PUBLISH-03 | `cargo package --list` produces a clean, complete package with no broken paths | Currently fails: `textual-rs-macros` path dep has no version field. Both crates need version requirement on the path dep. textual-rs-macros must also have publish metadata (license, description, repository). |
</phase_requirements>

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `dtolnay/rust-toolchain` | `@stable` (or `@master`) | Install Rust in GitHub Actions | Official dtolnay action; used in virtually all Rust CI |
| `actions/checkout` | `v4` | Checkout repo | Already in use |
| `actions/cache` | `v4` | Cache cargo registry + target | Already in use |
| `RUSTDOCFLAGS="-D warnings"` | env var | Fail CI on doc warnings | Standard Rust CI pattern |
| `cargo doc --no-deps` | cargo built-in | Build docs without deps | Avoids noise from upstream undoc items |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `cargo package --list` | cargo built-in | Verify package contents before publish | Run before every publish |
| `cargo publish --dry-run` | cargo built-in | Verify publish will succeed without uploading | Final check before live publish |
| `#![deny(missing_docs)]` | Rust lint | Enforce doc coverage in-codebase | Add to lib.rs; prevents regression |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `dtolnay/rust-toolchain` | `actions-rust-lang/setup-rust-toolchain` | Either works; dtolnay is simpler and already partially referenced |
| Manual sequential publish | `cargo-workspaces` tool | cargo-workspaces adds complexity; two crates can be published manually in order |

## Architecture Patterns

### Recommended CI Structure (updated ci.yml)

```yaml
jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable   # FIXED: was dtolnay/rust-action/setup@v1
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-
      - run: cargo build --workspace
      - run: cargo test --workspace

  docs:
    name: Docs
    runs-on: ubuntu-latest
    env:
      RUSTDOCFLAGS: "-D warnings"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --no-deps --workspace

  lint:
    # existing lint job — fix action name here too
```

### Publish Order (two-crate workspace)

Cargo requires proc-macro dependencies to exist on crates.io before the crate that depends on them
can be published. The mandatory order is:

1. Publish `textual-rs-macros` first
2. Publish `textual-rs` second (after macros appears in the crates.io index)

### Rustdoc Pattern: `#![deny(missing_docs)]`

Add to `crates/textual-rs/src/lib.rs` (top of file, before existing `//!` doc comment):

```rust
// Source: Rust Reference — inner attributes
#![deny(missing_docs)]
```

Add to `crates/textual-rs-macros/src/lib.rs`:

```rust
#![deny(missing_docs)]
//! Procedural macros for the textual-rs TUI framework.
//!
//! This crate is an implementation detail of `textual-rs`. Use that crate directly.
```

### Cargo Manifest: Path + Version for Workspace Publishing

```toml
# In crates/textual-rs/Cargo.toml — CURRENT (broken):
textual-rs-macros = { path = "../textual-rs-macros" }

# REQUIRED for cargo publish:
textual-rs-macros = { path = "../textual-rs-macros", version = "0.3.0" }
```

The version field must match what is on crates.io, so both crates must bump to the same version
before publish (or macros can use a different version, but both must be declared).

### textual-rs-macros Cargo.toml: Add Publish Metadata

The macros crate is missing fields required for crates.io:

```toml
[package]
name = "textual-rs-macros"
version = "0.3.0"
edition.workspace = true
description = "Procedural macros for the textual-rs TUI framework"
license = "MIT"
repository = "https://github.com/mbeha/textual-rs"
keywords = ["tui", "terminal", "macro", "derive"]
categories = ["command-line-interface"]
```

### Version Alignment

| Location | Current Value | Required Value |
|----------|--------------|----------------|
| `crates/textual-rs/Cargo.toml` | `0.1.0` | `0.3.0` (next after 0.2.0 in CHANGELOG) |
| `crates/textual-rs-macros/Cargo.toml` | `0.1.0` | `0.3.0` (keep in sync) |
| `crates/textual-rs/src/lib.rs` quick-start snippet | `"0.2"` | `"0.3"` |
| `CHANGELOG.md` | `[Unreleased]` block exists | Add `[0.3.0]` release block |

Note: CHANGELOG already has a `[Unreleased]` block with the CI/docs/metadata additions.
Rename that to `[0.3.0]` on publish day.

### Anti-Patterns to Avoid
- **Publish before CI is green:** Never publish to crates.io before the GitHub Actions matrix
  passes on all three platforms.
- **Publish main crate before macros:** crates.io index propagation takes ~30 seconds; check
  the crate appears at `https://crates.io/crates/textual-rs-macros` before publishing the main.
- **`cargo publish` without `--dry-run` first:** Always run `cargo publish --dry-run` and inspect
  the output before the live publish.
- **Missing `#![deny(missing_docs)]` after doc pass:** Without the lint, docs will regress.
  Add the attribute as part of the doc work, not after.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rust toolchain install in CI | Custom script | `dtolnay/rust-toolchain@stable` | Handles component installation, caching of toolchain metadata |
| Doc coverage checking | Custom grep/count script | `RUSTDOCFLAGS="-D missing_docs"` + `cargo doc` | Compiler-enforced, catches private-made-public items |
| Package verification | Manual file listing | `cargo package --list` | Respects .cargo-ignore, validates paths, checksums |

## Common Pitfalls

### Pitfall 1: CI Action Name Is Wrong
**What goes wrong:** `dtolnay/rust-action/setup@v1` returns a 404. GitHub Actions fails at the
checkout step, not the test step, so it looks like CI never ran.
**Why it happens:** The action was written with a nonexistent path; the correct name is
`dtolnay/rust-toolchain@stable` (no `/setup` subdirectory, no `@v1` tag).
**How to avoid:** Fix in ci.yml in all three places (test job and lint job, plus new docs job).
**Warning signs:** GitHub Actions shows "action not found" or immediate failure before any Rust
commands execute.

### Pitfall 2: Snapshot Tests Produce Different Output on macOS/Linux
**What goes wrong:** Insta snapshots are compared byte-for-byte. If terminal rendering differs by
platform (e.g., Unicode width tables differ between OS, line ending differences), snapshots fail
on non-Windows platforms.
**Why it happens:** The snapshots were created on Windows. `\r\n` vs `\n` differences in test
output, or unicode-width crate giving different widths for some codepoints, can cause mismatches.
**How to avoid:** All snapshot files in the repo are `\n` line endings. The unicode-width crate
(0.2) is cross-platform consistent. If failures appear, run `INSTA_UPDATE=new cargo test` on the
failing platform and check diffs.
**Warning signs:** CI fails only on macOS or Linux snapshot tests with "snapshot mismatch" but not
on Windows.

### Pitfall 3: arboard Clipboard on Linux Headless CI
**What goes wrong:** arboard requires an X11 server on Linux for clipboard tests. GitHub Actions
ubuntu runners do not have an X display by default. Any test that constructs `arboard::Clipboard`
will panic.
**Why it happens:** arboard calls X11 APIs at construction time on Linux. No `DISPLAY` env var =
connection error.
**How to avoid:** Check all tests for clipboard use. If any test directly creates an `arboard::Clipboard`,
either: (a) skip it with `#[cfg_attr(not(target_os = "windows"), ignore)]`, or (b) mock it in the
test harness. The existing code uses `default-features = false` on arboard which may limit which
backend is compiled, but x11rb IS in the lockfile so X11 backend is present.
**Warning signs:** Tests that pass locally (Windows) fail on Linux CI with "cannot connect to X
server" or similar.

### Pitfall 4: crates.io Index Propagation Delay
**What goes wrong:** After publishing `textual-rs-macros`, attempting to immediately publish
`textual-rs` fails because the registry index has not propagated the new macros crate yet.
**Why it happens:** crates.io index is a git repository; it takes 30-120 seconds to propagate.
**How to avoid:** After publishing macros, wait ~60 seconds or poll `cargo search textual-rs-macros`
before publishing the main crate.
**Warning signs:** `cargo publish -p textual-rs` fails with "no matching package named
`textual-rs-macros` found in registry".

### Pitfall 5: Version Mismatch Between Cargo.toml and CHANGELOG
**What goes wrong:** Publishing `0.1.0` when `0.2.0` is already in CHANGELOG (even if 0.2.0 was
never actually published to crates.io) will confuse users and create an inconsistent history.
**Why it happens:** Version was never bumped from initial scaffolding value.
**How to avoid:** Bump both crates to `0.3.0` (the next logical version after the 0.2.0 milestone
documented in CHANGELOG). Update lib.rs quick-start snippet. Add `[0.3.0]` entry to CHANGELOG
from the existing `[Unreleased]` block.

### Pitfall 6: Missing Publish Metadata in textual-rs-macros
**What goes wrong:** `cargo publish -p textual-rs-macros` fails with "missing required fields:
description, license".
**Why it happens:** The macros crate Cargo.toml has only `name`, `version`, `edition`, and
`[lib]` proc-macro entry — no `description`, `license`, `repository`, `keywords`, or `categories`.
**How to avoid:** Add all required crates.io metadata fields to the macros Cargo.toml before
attempting publish.

## Code Examples

Verified patterns from official sources:

### Correct GitHub Actions Rust Toolchain Setup
```yaml
# Source: https://github.com/dtolnay/rust-toolchain
- name: Install Rust (stable)
  uses: dtolnay/rust-toolchain@stable

# With components:
- name: Install Rust (stable with clippy+rustfmt)
  uses: dtolnay/rust-toolchain@stable
  with:
    components: clippy, rustfmt
```

### Docs CI Job with Warning-as-Error
```yaml
# Source: standard Rust CI pattern, verified by gadom.ski Rust docs CI article
docs:
  name: Docs
  runs-on: ubuntu-latest
  env:
    RUSTDOCFLAGS: "-D warnings"
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Check documentation
      run: cargo doc --no-deps --workspace
```

### Cargo Manifest for Workspace Publishing
```toml
# Source: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
# Path + version required for publishing:
[dependencies]
textual-rs-macros = { path = "../textual-rs-macros", version = "0.3.0" }
```

### Module-Level Doc Comment Pattern
```rust
// Source: Rust Reference — doc comments
//! The `app` module provides the [`App`] type, which owns the widget arena,
//! event loop, and terminal I/O.
pub mod app;
```

### Deny Missing Docs Lint
```rust
// Source: Rust Reference — lint attributes
// Place at top of lib.rs, before any `//!` doc:
#![deny(missing_docs)]
```

### Cargo Publish Workflow
```bash
# Source: Cargo Book — publishing
# Step 1: verify package contents
cargo package --list -p textual-rs-macros
cargo package --list -p textual-rs

# Step 2: dry run both crates
cargo publish --dry-run -p textual-rs-macros
cargo publish --dry-run -p textual-rs

# Step 3: publish macros first
cargo publish -p textual-rs-macros

# Step 4: wait for index propagation, then publish main
sleep 60
cargo publish -p textual-rs
```

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| GitHub Actions (ubuntu-latest runner) | PLATFORM-01 CI | ✓ | ubuntu-22.04/24.04 | — |
| GitHub Actions (macos-latest runner) | PLATFORM-01 CI | ✓ | macOS 14 (Sonoma) | — |
| GitHub Actions (windows-latest runner) | PLATFORM-01 CI | ✓ | Windows Server 2022 | — |
| crates.io API token | PUBLISH-01 | Must be set in repo secrets | — | — |
| Cargo (local) | Manifest verification | ✓ | 1.88+ (workspace rust-version) | — |

**Missing dependencies with no fallback:**
- crates.io API token — must be set as `CRATES_IO_TOKEN` repository secret before publish plan
  can execute. The planner should include a task: "Set CRATES_IO_TOKEN in GitHub repo secrets."

**Missing dependencies with fallback:**
- X11 display on Linux CI — needed if any test constructs `arboard::Clipboard` directly. Fallback:
  skip/ignore those tests with `#[cfg_attr(not(target_os = "windows"), ignore)]`. Investigate
  before adding xvfb-action overhead to CI.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + insta 1.46.3 + proptest 1.11.0 |
| Config file | none (uses default `cargo test`) |
| Quick run command | `cargo test --workspace` |
| Full suite command | `cargo test --workspace && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PLATFORM-01 | All tests pass on macOS and Linux | CI matrix | `cargo test --workspace` (in GitHub Actions on ubuntu-latest, macos-latest) | CI yaml (needs fix) |
| PUBLISH-01 | Crate publishes to crates.io | smoke (dry-run) | `cargo publish --dry-run -p textual-rs` | N/A — manual step |
| PUBLISH-02 | No missing_docs warnings | doc lint | `RUSTDOCFLAGS="-D missing_docs" cargo doc --no-deps --workspace` | ❌ Wave 0: add to CI docs job |
| PUBLISH-03 | cargo package clean | package check | `cargo package --list -p textual-rs` | N/A — manual step |

### Sampling Rate
- **Per task commit:** `cargo test --workspace` (local, Windows)
- **Per wave merge:** `cargo test --workspace && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`
- **Phase gate:** Full suite green + GitHub Actions matrix green + `cargo publish --dry-run` clean

### Wave 0 Gaps
- [ ] Add `docs` CI job to `.github/workflows/ci.yml` — covers PUBLISH-02
- [ ] No new test files needed — existing 471 tests cover PLATFORM-01 once CI is fixed

## Sources

### Primary (HIGH confidence)
- Rust Reference (lint attributes) — `#![deny(missing_docs)]` behavior
- Cargo Book — `cargo publish` path+version requirement, `cargo package --list`
- Direct code inspection — `cargo test --workspace` output (471 passing tests), `RUSTDOCFLAGS="-D missing_docs"` output (329 errors), `cargo package --list -p textual-rs` output (error on missing version)
- Direct code inspection — `.github/workflows/ci.yml` (wrong action name: `dtolnay/rust-action/setup@v1`)
- Direct code inspection — `CHANGELOG.md` (last released version `0.2.0`; `Cargo.toml` still at `0.1.0`)
- GitHub API — confirmed `dtolnay/rust-action` returns 404; `dtolnay/rust-toolchain` exists

### Secondary (MEDIUM confidence)
- [GitHub — dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — correct action name and syntax (verified via GitHub API)
- arboard search results — x11rb backend active by default on Linux; wayland-data-control optional; `default-features = false` still compiles x11rb backend

### Tertiary (LOW confidence)
- Snapshot cross-platform compatibility — assumed safe because unicode-width 0.2 is deterministic and snapshots use `\n`; not verified by running CI on macOS/Linux

## Metadata

**Confidence breakdown:**
- CI fix: HIGH — wrong action name confirmed by 404 on GitHub API
- Missing docs count: HIGH — measured directly with `RUSTDOCFLAGS="-D missing_docs"`
- Manifest issues: HIGH — confirmed by `cargo package` error output
- Version discrepancy: HIGH — Cargo.toml vs CHANGELOG vs lib.rs all inspected
- arboard Linux behavior: MEDIUM — based on arboard docs/source, not tested
- Snapshot cross-platform: LOW — theoretical analysis, not empirically verified

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable Cargo/CI tooling)
