---
phase: 05-developer-experience-and-polish
plan: 04
subsystem: ui
tags: [demo, tutorial, documentation, rustdoc, widgets]

requires:
  - phase: 05-01
    provides: Widget proc macros (#[derive(Widget)], widget_impl)
  - phase: 05-02
    provides: Background worker system (WorkerResult, run_worker)
  - phase: 05-03
    provides: Command palette (CommandPalette, CommandRegistry)

provides:
  - demo.rs: 4-tab widget showcase (Inputs, Display, Layout, Interactive) with lazeport dark theme
  - demo.tcss: external TCSS stylesheet with #0a0a0f/#00ffa3/#00d4ff palette
  - irc_demo.rs: weechat-style IRC client using Log, ListView, Input, Header, Footer
  - irc_demo.tcss: IRC-specific dark theme CSS
  - tutorial_01_hello.rs: hello world app with App::new, Widget trait, run()
  - tutorial_02_layout.rs: compose(), with_css(), Header/Footer/Label layout
  - tutorial_03_events.rs: key_bindings, on_action, on_event, Cell<T>
  - tutorial_04_reactive.rs: Reactive<T>, get_untracked(), Input::Changed
  - tutorial_05_workers.rs: ctx.run_worker(), WorkerResult<T>, async tasks
  - rustdoc on App, Widget trait, Reactive<T>, WorkerResult, AppContext methods

affects:
  - future-contributors (tutorial examples serve as living documentation)
  - phase-06-if-any (docs baseline established)

tech-stack:
  added: []
  patterns:
    - "include_str!(\"file.tcss\") for external TCSS files in examples"
    - "Reactive<T> + get_untracked() for widget state in render()"
    - "Cell<Option<WidgetId>> for own_id in widgets"
    - "cargo doc --no-deps -p textual-rs for library documentation"

key-files:
  created:
    - crates/textual-rs/examples/demo.tcss
    - crates/textual-rs/examples/irc_demo.tcss
    - crates/textual-rs/examples/tutorial_01_hello.rs
    - crates/textual-rs/examples/tutorial_02_layout.rs
    - crates/textual-rs/examples/tutorial_03_events.rs
    - crates/textual-rs/examples/tutorial_04_reactive.rs
    - crates/textual-rs/examples/tutorial_05_workers.rs
  modified:
    - crates/textual-rs/examples/demo.rs
    - crates/textual-rs/examples/irc_demo.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/reactive/mod.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/worker.rs
    - crates/textual-rs/src/widget/markdown.rs
    - crates/textual-rs/Cargo.toml

key-decisions:
  - "Tutorial examples registered in [[example]] sections of Cargo.toml (not auto-discovered)"
  - "External .tcss files via include_str!() preferred over inline const CSS in demo files"
  - "TreeNode::with_children() used for tree hierarchy (no add_child() method exists)"
  - "Horizontal::with_children() / Vertical::with_children() (not Horizontal::new(vec))"
  - "Widget::render() should use get_untracked() to avoid reactive tracking loops"

patterns-established:
  - "Tutorial structure: minimal app -> layout -> events -> reactive -> workers"
  - "Doc fixes: wrap generic types in backticks (WorkerResult<T> not WorkerResult<T>)"

requirements-completed: [DX-05]

duration: ~15min
completed: 2026-03-26
---

# Phase 5 Plan 04: Demo Applications and Tutorial Examples Summary

**4-tab widget showcase (demo.rs), weechat IRC client (irc_demo.rs), 5 annotated tutorial examples, and zero-warning rustdoc on all key public types**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-26T06:20:00Z
- **Completed:** 2026-03-26T06:36:00Z
- **Tasks:** 3 (Task 4 is a human-verify checkpoint, pending user approval)
- **Files modified:** 17

## Accomplishments

- demo.rs rewritten with 4 tabs (Inputs, Display, Layout, Interactive) and external demo.tcss
- irc_demo.rs updated to use external irc_demo.tcss with lazeport palette
- 5 tutorial examples created (01-05) covering Hello World through Background Workers
- cargo doc --no-deps -p textual-rs now produces zero warnings (fixed 5 pre-existing warnings)
- Added crate-level doc comment, App struct doc, and comprehensive Widget trait docs

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite demo.rs and irc_demo.rs** - `9a82476` (feat)
2. **Task 2: Create tutorial examples 01-05** - `a84270f` (feat)
3. **Task 3: Add rustdoc to key public types** - `b6b2784` (feat)

Task 4 (Visual verification) is a `checkpoint:human-verify` — pending user approval.

## Files Created/Modified

- `crates/textual-rs/examples/demo.rs` — 4-tab widget showcase (Inputs/Display/Layout/Interactive)
- `crates/textual-rs/examples/demo.tcss` — Dark theme: #0a0a0f bg, #00ffa3 green, #00d4ff cyan
- `crates/textual-rs/examples/irc_demo.rs` — Updated header subtitle, external CSS
- `crates/textual-rs/examples/irc_demo.tcss` — IRC-specific dark theme
- `crates/textual-rs/examples/tutorial_01_hello.rs` — Hello World with App::new
- `crates/textual-rs/examples/tutorial_02_layout.rs` — compose() + with_css()
- `crates/textual-rs/examples/tutorial_03_events.rs` — key_bindings + on_action + on_event
- `crates/textual-rs/examples/tutorial_04_reactive.rs` — Reactive<T> + Input::Changed
- `crates/textual-rs/examples/tutorial_05_workers.rs` — ctx.run_worker() + WorkerResult<T>
- `crates/textual-rs/Cargo.toml` — Added [[example]] entries for all 5 tutorials
- `crates/textual-rs/src/lib.rs` — Crate-level doc comment
- `crates/textual-rs/src/app.rs` — App struct doc with example
- `crates/textual-rs/src/widget/mod.rs` — Widget trait + all method docs
- `crates/textual-rs/src/reactive/mod.rs` — Reactive<T> expanded docs
- `crates/textual-rs/src/widget/markdown.rs` — Fixed [text](url) doc link warning
- `crates/textual-rs/src/widget/context.rs` — Fixed WorkerResult<T> HTML tag warning
- `crates/textual-rs/src/worker.rs` — Fixed WorkerResult<T>/Box<dyn Any> HTML tag warnings

## Decisions Made

- Tutorial examples must be registered in Cargo.toml `[[example]]` sections — not auto-discovered
- External `.tcss` files via `include_str!()` preferred for cleanliness vs inline `const CSS: &str`
- `Horizontal::with_children(vec)` / `Vertical::with_children(vec)` — not `new(vec)` (API difference)
- `TreeNode::with_children()` for hierarchy — no `add_child()` method exists in Tree widget
- All doc examples marked `no_run` where they require a running terminal

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed Horizontal/Vertical API mismatch**
- **Found during:** Task 1 (demo.rs rewrite)
- **Issue:** Plan said `Horizontal::new(vec)` but actual API is `Horizontal::with_children(vec)`
- **Fix:** Read source files before writing, used correct API
- **Files modified:** crates/textual-rs/examples/demo.rs
- **Committed in:** 9a82476 (Task 1 commit)

**2. [Rule 3 - Blocking] Added [[example]] entries to Cargo.toml**
- **Found during:** Task 2 (tutorial examples)
- **Issue:** `cargo build --example tutorial_01_hello` failed with "no example target"
- **Fix:** Added 5 `[[example]]` entries to crates/textual-rs/Cargo.toml
- **Files modified:** crates/textual-rs/Cargo.toml
- **Committed in:** a84270f (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Known Stubs

None — all widgets in demos use real data (no hardcoded empty values or placeholders that block plan goals).

## Issues Encountered

- `cargo build --example` failed without `-p textual-rs` initially, then succeeded (workspace multi-member issue resolved by adding explicit `[[example]]` entries)

## Next Phase Readiness

- Task 4 (Visual verification) pending: run `cargo run --example demo` and `cargo run --example irc_demo` to visually verify
- All code compiles; human approval of visual quality needed
- Phase 5 plan 04 is the last plan — after human verification, Phase 5 is complete

---
*Phase: 05-developer-experience-and-polish*
*Completed: 2026-03-26*

## Self-Check: PASSED

Files verified:
- crates/textual-rs/examples/demo.rs — EXISTS
- crates/textual-rs/examples/demo.tcss — EXISTS
- crates/textual-rs/examples/irc_demo.rs — EXISTS
- crates/textual-rs/examples/irc_demo.tcss — EXISTS
- crates/textual-rs/examples/tutorial_01_hello.rs — EXISTS
- crates/textual-rs/examples/tutorial_02_layout.rs — EXISTS
- crates/textual-rs/examples/tutorial_03_events.rs — EXISTS
- crates/textual-rs/examples/tutorial_04_reactive.rs — EXISTS
- crates/textual-rs/examples/tutorial_05_workers.rs — EXISTS

Commits verified:
- 9a82476 — feat(05-04): rewrite demo.rs and irc_demo.rs widget showcase demos
- a84270f — feat(05-04): create 5 tutorial examples with heavy inline comments
- b6b2784 — feat(05-04): add rustdoc to key public types, fix cargo doc warnings
