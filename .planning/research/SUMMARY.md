# Project Research Summary

**Project:** textual-rs v1.3 — Widget Parity & Screen Stack
**Domain:** Rust TUI application framework (parity milestone with Python Textual)
**Researched:** 2026-03-26
**Confidence:** HIGH

## Executive Summary

textual-rs v1.3 is a Rust TUI framework milestone to reach functional parity with Python Textual's widget library. The scope is 13 missing widgets plus screen stack behavioral wiring. Research confirms that the existing codebase infrastructure — widget arena, reactive signals, CSS cascade, layout engine, and event loop — is fully capable of supporting all 13 widgets without architectural rework. Only two new crates are needed: `walkdir` for DirectoryTree's filesystem traversal and `serde_json` for Pretty's structured output. Screen stack infrastructure already exists in `AppContext` and `tree.rs`; what is missing is focus save/restore on push/pop and the `advance_focus` call in the event loop drain path.

The recommended approach is a 6-tier build order that respects hard widget dependencies: screen stack wiring first (unblocks modal workflows in demos), then trivial render-only widgets to accumulate widget count quickly, then list/selection widgets, then enhanced display widgets, then the three complex widgets that require novel patterns (DirectoryTree with worker integration, MaskedInput with raw-space cursor tracking, Toast with a dedicated `Vec<ToastEntry>` layer). The key architectural insight confirmed by research is that Toast must NOT reuse `active_overlay` — that slot is for single-instance overlays (Select, CommandPalette); toasts need their own queue to support stacking.

Three design decisions must be made before writing code to avoid costly rewrites: (1) focus history must be a `Vec<Option<WidgetId>>` stored alongside `screen_stack` in `AppContext`, pushed before each `push_screen` and popped after each `pop_screen`; (2) Toast's `Vec<ToastEntry>` render layer must be designed as a dedicated structure distinct from `active_overlay`; and (3) MaskedInput cursor position must be tracked in raw-value space only, with display cursor derived per render, or drift bugs will appear on every Backspace. All three decisions are cheap to make upfront and expensive to retrofit.

## Key Findings

### Recommended Stack

The stack is almost entirely fixed — the existing crate set handles everything for v1.3. Only two new dependencies are required. See [STACK.md](STACK.md) for full dependency rationale and alternatives considered.

**Core technologies (existing, confirmed, no change):**
- `ratatui 0.30`: Terminal rendering backend — all widget rendering builds on this
- `crossterm 0.29`: Terminal I/O — Windows KeyEventKind filtering already handled in `app.rs`
- `tokio current_thread + LocalSet`: Single-threaded event loop — all blocking I/O must go through `ctx.run_worker`
- `reactive_graph 0.2.13`: Reactive signals — `Reactive<T>` on widget fields drives state
- `taffy 0.9.2`: CSS layout engine — ContentSwitcher uses `recompose_widget` to avoid needing display:none
- `slotmap 1.0`: Widget arena — WidgetIds are single-use; treat as invalid across unmount/remount cycles
- `insta 1.46.3`: Snapshot testing — primary verification for all new widgets

**New dependencies for v1.3 (exactly two crates):**
- `walkdir 2`: DirectoryTree filesystem traversal — BurntSushi crate with built-in symlink loop detection, depth limits, cross-platform sorted iteration
- `serde_json 1` (preserve_order feature): Pretty widget structured output — `Value` tree as "any Rust object" equivalent; hand-written 80-line JSON tokenizer produces colored ratatui Spans; no syntect needed

**What to avoid:**
- `notify` (filesystem watcher): DirectoryTree only needs manual `reload()`, not live watching
- `syntect`: 4MB embedded syntax definitions for a use case solved by a 6-color hand-written tokenizer
- `figlet-rs`: Digits only needs a 17-char static lookup table, not a full font renderer

### Expected Features

All 13 widget specs sourced directly from https://textual.textualize.io/widgets/. See [FEATURES.md](FEATURES.md) for full behavioral specifications and the dependency graph.

**Must have (table stakes — P1, core parity):**
- Static — base display widget; essentially a formalization of what Label already provides
- Rule — horizontal/vertical separator; pure render, no state
- OptionList — scrollable single-select list; hard prerequisite for SelectionList
- SelectionList — multi-select with checkboxes; requires OptionList first
- ContentSwitcher — show exactly one of N children; pairs with existing Tabs widget
- Screen Stack — push/pop/switch_screen, ModalScreen with input blocking, focus save/restore, dismiss/callback

**Should have (competitive differentiators — P2):**
- RichLog — styled real-time log; upgrade from existing text-only Log widget; `max_lines` eviction important for long-running processes
- Link — clickable URL opener; trivial extension of Static
- Toast + App.notify() — overlay notifications with auto-dismiss and stacking; `notify()` API must be callable from workers
- LoadingIndicator — animated spinner; the `widget.loading = true` base-class overlay integration is the key feature, not just the standalone widget

**Complete parity (P3):**
- Digits — large block-character numbers; visually distinctive clock/counter widget
- Pretty — structured data display with syntax coloring; developer tooling differentiator
- DirectoryTree — filesystem browser with lazy loading; highest implementation complexity of all widgets
- MaskedInput — structured input with template masks (credit cards, dates, phone numbers); highest cursor logic complexity

**Explicitly defer:**
- Modes system (independent screen stacks per section): significant complexity multiplier; most applications do not need it; implement basic push/pop first

### Architecture Approach

textual-rs uses a single-threaded arena-based widget system with deferred mutation. Widgets receive `&AppContext` (not `&mut`) in callbacks; all structural changes are enqueued into `RefCell`-wrapped fields on `AppContext` and drained by the event loop after each callback returns. This pattern prevents borrow-checker conflicts between arena access and context mutation. Five key patterns govern v1.3 widget implementations. See [ARCHITECTURE.md](ARCHITECTURE.md) for full component diagram, data flow diagrams, and code examples for each pattern.

**Major components:**
1. `AppContext` — shared world state: SlotMap widget arena, screen_stack, active_overlay, message_queue, pending_screen_pushes/pops, pending_recompose, CSS computed style cache
2. `widget/tree.rs` — all structural mutations: mount_widget, unmount_widget, push_screen, pop_screen, advance_focus, recompose_widget; widgets never mutate the arena directly
3. Event loop (tokio LocalSet) — crossterm events dispatched through flume channel; pending_* queues drained in order; CSS cascade + Taffy layout + ratatui render triggered after drains
4. `active_overlay` — single floating widget slot above all content (Select, CommandPalette); NOT for Toast
5. `screen_stack: Vec<WidgetId>` — only top element renders and receives focus; push/pop implemented; missing: focus_history save/restore and advance_focus on push/pop drain

**Key architectural patterns for v1.3 (all confirmed by codebase inspection):**
- ContentSwitcher: `request_recompose` + `compose()` returning only the active child; do NOT use CSS display toggling
- DirectoryTree: `Rc<Tree>` shared between parent DirectoryTree and arena entry; lazy loads via `run_worker` on NodeExpanded message; same Rc sharing pattern as TabbedContent
- Toast: dedicated `Vec<ToastEntry>` on AppContext, rendered as a separate bottom-right anchored layer; worker-based timers so auto-dismiss cancels automatically on unmount
- LoadingIndicator: frame-skip animation using existing AppEvent::Tick at 33ms; `skip_animations` flag for deterministic snapshot tests (same gating as Switch/Tabs)
- OptionList/SelectionList: extend ListView cursor/scroll pattern; OptionList emits OptionSelected message; SelectionList adds `Vec<bool>` per-item selection state

### Critical Pitfalls

11 pitfalls identified across all research files. See [PITFALLS.md](PITFALLS.md) for prevention strategies, warning signs, and recovery costs for each.

**Top 5 highest-impact pitfalls:**

1. **Screen stack pop loses focus** — Design `focus_history: Vec<Option<WidgetId>>` into AppContext before writing any screen stack consumer code. Push before `push_screen`, pop after `pop_screen`. If restored id is no longer in the arena, fall back to `advance_focus`. Without this, Tab navigation silently breaks after every modal dismiss.

2. **Events routed to wrong screen layer** — After draining `pending_screen_pushes`, immediately call `advance_focus` to redirect focus to the new screen before processing further events. Add debug assertion: focused widget must be in the subtree of `screen_stack.last()`.

3. **Multiple toasts overlap via active_overlay reuse** — Never use `active_overlay` for toasts. That slot supports exactly one widget at a time. Toast requires a separate `toast_queue: Vec<ToastEntry>` on AppContext with a dedicated render pass. Two rapid `notify()` calls must both be visible and stacked.

4. **MaskedInput cursor drift on Backspace** — Maintain cursor position in raw-value space only. Display cursor is always computed from raw position by counting separators during render. Storing cursor in display space causes position drift on every delete. Property-test all cursor positions before integrating with the render pipeline.

5. **DirectoryTree blocks event loop** — Never call `fs::read_dir` in `on_event` or `compose`. Always use `ctx.run_worker`. On NFS mounts or directories with thousands of entries, synchronous I/O on the LocalSet thread freezes the entire UI including the 33ms render tick.

**Additional pitfalls to address per phase:**
- Render artifacts after screen pop: render `ratatui::widgets::Clear` over the former modal area in the same frame that removes the overlay
- Toast timer firing on unmounted widget: use worker-based timers so `cancel_workers` in `unmount_widget` handles cancellation automatically
- DirectoryTree symlink loops: `walkdir` handles this by default; never enable `follow_links(true)` without loop detection
- crates.io README path: `cargo package --list` must include `README.md`; `../../README.md` paths do not package correctly
- Accidental Widget trait semver breaks: every new trait method must have a default implementation; run `cargo semver-checks` before every publish

## Implications for Roadmap

Based on combined research across all four files, the 6-tier build order from ARCHITECTURE.md is the correct phase structure. Pitfall prevention work is front-loaded into the phases that introduce the highest-risk patterns.

### Phase 1: Screen Stack Wiring
**Rationale:** Infrastructure that unblocks modal workflows needed in demos for every subsequent phase. Already partially implemented — `push_screen`/`pop_screen` exist in `tree.rs`. What is missing: `focus_history` data structure, `advance_focus` after push/pop drain, `ModalScreen` input blocking, dismiss/callback pattern, and public `App::push_screen`/`App::pop_screen` API. Design `focus_history` into AppContext before writing any consumer code — retrofitting it later breaks every demo.
**Delivers:** Working push_screen / pop_screen / switch_screen, ModalScreen with input blocking, dismiss/callback data passing, focus save/restore on push/pop, screen lifecycle events (suspend/resume), snapshot tests for focus and render-artifact invariants
**Features from FEATURES.md:** Screen Stack (all sub-items except Modes system)
**Avoids:** Pitfalls 1 (focus loss), 2 (wrong screen routing), 3 (render artifacts) — all designed in from the start
**Research flag:** `push_screen_wait` async variant (await a modal screen's result) needs explicit design for tokio LocalSet integration. Screen suspend/resume lifecycle events: decide whether to include in v1.3 scope.

### Phase 2: Render-Only Foundation Widgets
**Rationale:** No complex state or interactions; high confidence; accumulates widget count quickly. All are under ~120 LOC each. Static and Link are prerequisites if any downstream work builds on them.
**Delivers:** Static, Rule, Link, Pretty, Digits
**Features from FEATURES.md:** Static (P1), Rule (P1), Link (P2), Pretty (P3), Digits (P3)
**Stack from STACK.md:** `serde_json` Value tree + hand-written tokenizer for Pretty; static Unicode lookup table for Digits/Rule; `std::process::Command` with `#[cfg(target_os)]` for Link
**Avoids:** No significant pitfalls in this tier; all are straightforward render implementations
**Research flag:** Standard patterns — skip additional research

### Phase 3: List and Selection Widgets
**Rationale:** OptionList is a hard prerequisite for SelectionList. ContentSwitcher is a prerequisite for meaningful Tabs-panel demos. These three are the highest-value interactive widgets that application authors will reach for first.
**Delivers:** OptionList, SelectionList, ContentSwitcher
**Features from FEATURES.md:** OptionList (P1), SelectionList (P1), ContentSwitcher (P1)
**Implements:** `recompose_widget` pattern for ContentSwitcher; new cursor+scroll+selection pattern for OptionList extending ListView; multi-select `Vec<bool>` state for SelectionList
**Avoids:** ContentSwitcher must use `recompose_widget`, not CSS display toggling (Taffy does not support display:none)
**Research flag:** Standard patterns — recompose_widget and ListView cursor patterns already established in codebase

### Phase 4: Enhanced Display and Async Loading Widgets
**Rationale:** RichLog upgrades the existing Log widget used in real applications. LoadingIndicator's `widget.loading = true` base-class integration is the most impactful async UX feature. Both are independently deliverable.
**Delivers:** RichLog, LoadingIndicator
**Features from FEATURES.md:** RichLog (P2) with max_lines eviction, LoadingIndicator (P2) with base Widget `loading` integration
**Avoids:** LoadingIndicator animation must gate on `skip_animations` for deterministic snapshot tests; timer via AppEvent::Tick at 33ms (frame-skip, no new timer infrastructure)
**Research flag:** The `widget.loading = true` overlay integration requires changes to the Widget trait or AppContext — decide scope (full Widget trait integration vs. standalone widget only) before implementation

### Phase 5: Complex Widgets — DirectoryTree, MaskedInput, Toast
**Rationale:** These three share the characteristic of requiring novel patterns not yet established in the codebase. Group together so the novel patterns can be reviewed as a batch. Each has a design decision that must be made before writing code.
**Delivers:** DirectoryTree (with lazy loading and symlink protection), MaskedInput (with raw-space cursor tracking), Toast + App.notify() (with Vec<ToastEntry> stacking layer)
**Features from FEATURES.md:** DirectoryTree (P3), MaskedInput (P3), Toast (P2)
**Stack from STACK.md:** `walkdir 2` for DirectoryTree; `Rc<Tree>` sharing pattern; worker-based Toast timers
**Avoids:** Pitfall 4 (Toast timer on unmounted widget — use worker API), Pitfall 5 (multiple toasts overlapping — Vec<ToastEntry> not active_overlay), Pitfall 6 (DirectoryTree symlink loop — walkdir default behavior), Pitfall 7 (DirectoryTree blocking event loop — run_worker), Pitfalls 8+9 (MaskedInput cursor drift and separator skip — raw-space cursor invariant)
**Research flag:** DirectoryTree symlink loop detection on Windows (no `same_file` inodes on NTFS); MaskedInput cursor round-trip property tests; Toast Vec<ToastEntry> render layer z-order relative to active_overlay

### Phase 6: Cross-Platform Verification and crates.io Publish
**Rationale:** Publish prerequisites must be verified before the first public release. `cargo package --list` catches the README omission. `cargo semver-checks` catches breaking Widget trait changes. CI matrix catches platform-specific rendering differences.
**Delivers:** CI matrix on macOS + Linux; crates.io package with `README.md` inside crate directory, license headers, keywords, categories, description; CHANGELOG.md; `cargo semver-checks` passing
**Avoids:** Pitfall 10 (README missing from package — `cargo package --list` before publish), Pitfall 11 (semver break — default implementations on all new Widget trait methods)
**Research flag:** Standard checklist — no additional research needed

### Phase Ordering Rationale

- Screen stack first: focus scoping and modal overlay semantics are required for demos in every subsequent phase. Building widget demos without correct focus is a dead end.
- Render-only widgets second: zero new patterns, high confidence, immediately visible; clears the low-complexity widget backlog before tackling harder widgets.
- List widgets before complex widgets: OptionList establishes the cursor+scroll pattern that SelectionList reuses, and deeply understanding it informs MaskedInput's cursor design in Phase 5.
- Toast deferred to Phase 5 despite being P2: its Vec<ToastEntry> architecture decision is cheap to get right when focused on it, expensive to retrofit if rushed into an earlier phase.
- Publish last: premature publishing with a broken semver or missing README is recoverable (yank + republish) but creates friction for any early adopters.

### Research Flags

Phases likely needing deeper design review or research during planning:
- **Phase 1 (Screen Stack):** `push_screen_wait` async variant design; `focus_history` data structure; screen suspend/resume scope decision
- **Phase 4 (LoadingIndicator):** `widget.loading = true` base-class overlay integration scope — whether to change the Widget trait or implement a separate mechanism
- **Phase 5 (DirectoryTree):** Symlink loop detection on Windows without `same_file` inode comparison; async worker pattern for cross-platform path canonicalization
- **Phase 5 (MaskedInput):** Raw-space cursor property testing coverage; edge cases with multi-byte Unicode in mask positions
- **Phase 5 (Toast):** Vec<ToastEntry> render layer interaction with active_overlay z-order; worker-based timer cancellation verification

Phases with standard patterns (skip additional research):
- **Phase 2 (render-only widgets):** All are Unicode lookups + ratatui primitives; well-understood
- **Phase 3 (list widgets):** OptionList/SelectionList follow the ListView pattern; ContentSwitcher uses the recompose_widget path already verified
- **Phase 4 (RichLog):** Straightforward extension of existing Log widget
- **Phase 6 (publish):** Standard Cargo workspace publishing checklist

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All existing crate choices verified by direct codebase audit; only 2 new deps identified; both are battle-tested with hundreds of millions of downloads |
| Features | HIGH | All 13 widget specs sourced directly from Python Textual official documentation; behavioral requirements are unambiguous |
| Architecture | HIGH | Based on direct code inspection of widget/tree.rs, widget/context.rs, widget/mod.rs, widget/select.rs, widget/tree_view.rs; patterns are concrete, tested in existing widgets |
| Pitfalls | HIGH | 11 pitfalls identified with specific prevention strategies grounded in actual borrow rules, ratatui diff behavior, and codebase-specific DenseSlotMap WidgetId lifetime rules |

**Overall confidence:** HIGH — all four research areas are grounded in direct codebase inspection and official documentation. No MEDIUM or LOW areas. The existing codebase is the primary source of truth and was inspected directly.

### Gaps to Address

- **push_screen_wait async variant:** Python Textual supports `await app.push_screen_wait(screen)` for getting a result from a modal without a callback. Implementing this requires a oneshot channel or similar pattern within the tokio LocalSet runtime. Design this explicitly in Phase 1 planning — it is optional for v1.3 scope but worth deciding early.

- **widget.loading base-class integration:** The `loading: bool` reactive on the Widget base class that auto-overlays a LoadingIndicator on any widget (not just a standalone LoadingIndicator) requires changes to the Widget trait or AppContext. This is the feature that distinguishes LoadingIndicator from "just another widget." Confirm scope in Phase 4 planning.

- **DirectoryTree on Windows symlinks:** Windows NTFS symlinks require elevated permissions to create in most configurations. The test fixture for symlink loop detection (Pitfall 6) will need to be conditionally compiled or skipped on Windows CI. Evaluate during Phase 5 planning.

- **Screen suspend/resume lifecycle events:** Python Textual fires `on_screen_suspend` and `on_screen_resume` when screens are pushed below or become topmost. Not currently present in textual-rs. Useful for pausing background workers on suspended screens. Decide in Phase 1 whether to include in v1.3 or defer.

- **Toast z-ordering vs. active_overlay:** The Vec<ToastEntry> render layer must paint above the current screen but below active_overlay (which is for interactive overlays). The exact render pass ordering needs explicit design in Phase 5.

## Sources

### Primary (HIGH confidence)
- Official Python Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/ — all 13 widget behavioral specifications
- Official Python Textual docs: https://textual.textualize.io/guide/screens/ — screen stack behavior, ModalScreen, dismiss/callback pattern
- Codebase audit (2026-03-26): `widget/tree.rs`, `widget/context.rs`, `widget/mod.rs`, `widget/select.rs`, `widget/tree_view.rs`, `app.rs`, `event/timer.rs`, `widget/log.rs` — all architectural patterns verified against actual implementation
- walkdir crate: https://docs.rs/walkdir — symlink loop detection via same_file, depth limits, cross-platform path sorting; 291M+ downloads
- serde_json crate: https://docs.rs/serde_json — Value tree, to_string_pretty, preserve_order feature

### Secondary (MEDIUM confidence)
- Ratatui rendering internals: https://ratatui.rs/concepts/rendering/under-the-hood/ — diff engine behavior, why Clear is required after overlay removal
- Ratatui artifact issue: https://forum.ratatui.rs/t/list-tabs-widgets-switching-keeps-character-artifact-upon-rendering/256 — confirms Clear widget is the correct fix
- Python Textual screen stack bug: https://github.com/Textualize/textual/issues/1632 — focus restore pattern, informed focus_history design
- cargo-semver-checks: https://crates.io/crates/cargo-semver-checks — breaking change detection tool
- crates.io publishing guide: https://doc.rust-lang.org/cargo/reference/publishing.html — README path requirements and package verification

### Tertiary (LOW confidence — validate during implementation)
- Masked input cursor position sync: https://giacomocerquone.com/blog/keep-input-cursor-still/ — raw-space cursor invariant pattern; informed MaskedInput pitfall analysis
- Textual toast positioning: https://github.com/Textualize/textual/discussions/3541 — Python Textual's ToastRack stacking behavior; confirms Vec<ToastEntry> approach
- crossterm Windows KeyEventKind: https://github.com/ratatui/ratatui/issues/347 — confirms key-release filtering is required; already handled in app.rs line 338
- tokio task cancellation: https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/ — informed worker-based timer design for Toast

---
*Research completed: 2026-03-26*
*Ready for roadmap: yes*
