# Project Research Summary

**Project:** textual-rs
**Domain:** Rust TUI application framework library (Textual Python port)
**Researched:** 2026-03-24
**Confidence:** HIGH

## Executive Summary

textual-rs is a Rust library that ports the Textual Python TUI framework, providing CSS-like
styling, a reactive property system, a persistent widget tree, an async event loop, and a rich
set of built-in widgets. The fundamental architecture challenge is that the Rust ecosystem has
a mature rendering layer (ratatui, 19k stars, 3M downloads/month) but nothing above it that
approaches Textual's ergonomics. No existing Rust library provides the DOM-like widget tree,
CSS cascade engine, reactive signals, and event routing that Textual offers. This gap is the
core justification for textual-rs, and the research confirms it is the right project to build.

The recommended strategy is to build on top of ratatui rather than from scratch. Ratatui
solves hard problems that textual-rs should not reimplement: Unicode-correct buffer diffing,
constraint-based 1D layout, a complete rendering pipeline, and a broad third-party widget
ecosystem. textual-rs provides everything above ratatui — the retained widget tree (slotmap
arena), CSS-like styling and layout (cssparser + Taffy), reactive properties (reactive_graph
signals), event routing, focus management, and async lifecycle hooks (Tokio with LocalSet).
crossterm is the only viable terminal backend for a cross-platform library; termion lacks
Windows support and termwiz has an unstable API.

The principal risks are Rust-specific: the borrow checker makes retained widget trees with
cross-references difficult, async fn in trait objects requires workarounds, and reactive
signals need a proof-of-concept before committing to reactive_graph. The CSS/layout work
(cssparser + Taffy) is well-scoped and lower risk than it appears because the parsing surface
for TCSS is small. Testing infrastructure is excellent: ratatui's TestBackend plus insta
snapshots plus a TestApp harness model covers widget rendering, event dispatch, and visual
regression with no real terminal required.

---

## Key Findings

### Recommended Stack

The Rust ecosystem provides strong primitives for every layer of textual-rs. The rendering
foundation is ratatui (v0.30.0, modular workspace since December 2025), which should be
consumed via `ratatui-core` for API stability. The async runtime is Tokio — async-std was
discontinued March 2025, and crossterm's EventStream is Tokio-native. The widget tree should
use slotmap arenas (generational indices prevent use-after-free, O(1) lookup). Taffy (v0.9.2)
is the layout engine; ratatui's Cassowary solver cannot express CSS Grid, absolute positioning,
`align-items`, or `gap`, all of which Textual uses. cssparser (v0.35.0) handles CSS Syntax
Level 3 tokenization, with hand-rolled property and selector matching on top.

**Core technologies:**
- `ratatui` (0.30) / `ratatui-core` (0.1): Terminal rendering — battle-tested, 19k stars, modular; target `ratatui-core` for API stability
- `crossterm` (0.29): Terminal backend — only cross-platform option (Windows + macOS + Linux); 73.7M downloads
- `tokio` (1.x) with `LocalSet`: Async runtime — ecosystem standard; crossterm EventStream requires it; LocalSet avoids `Send + 'static` pressure on widget state
- `slotmap` (1.x): Widget tree storage — arena with generational keys prevents reference cycles and use-after-free bugs
- `taffy` (0.9): Layout engine — CSS Flexbox + Grid, used by Servo/Bevy/Zed/Slint; necessary because ratatui cannot express 2D layouts
- `cssparser` (0.35): CSS tokenization — handles Syntax Level 3 block structure; hand-roll property parsing on top
- `reactive_graph` (from Leptos org): Reactive signals — `RwSignal<T>`, `Memo<T>`, `Effect`; Tokio-compatible via `any_spawner`; effects schedule async on next tick
- `flume` (0.11): Event bus channel — unified sync/async API bridges keyboard thread to async event loop
- `insta` (1.x): Snapshot testing — de-facto standard; inline snapshots make visual regressions obvious in code review

### Expected Features

textual-rs is a framework library, not an application. Its "features" are the capabilities it
exposes to application developers, organized by what Textual provides and what Rust's ecosystem
needs to bridge to get there.

**Must have (table stakes — without these textual-rs is not a Textual port):**
- CSS-like stylesheet loading and cascade (`.tcss` files with type, class, id, pseudo-class selectors)
- Retained widget tree with mount/unmount lifecycle (`on_mount`, `on_unmount`, `on_ready`)
- Reactive properties — `RwSignal<T>` on widget fields; mutations trigger watch callbacks and re-render
- Flexbox and Grid layout via Taffy — including `dock`, `fr` units, `align-items`, `gap`
- Event routing: keyboard/mouse events bubble up the widget tree; widgets declare handlers
- Focus management: tab order, programmatic focus, focus ring, `:focus` pseudo-class
- Async workers: background tasks that post results to the UI without blocking the event loop
- Built-in widgets: Button, Label, Input, TextArea, Checkbox, RadioSet, Select, ProgressBar, DataTable, Tree
- Dark/light theme support and `:dark`/`:light` TCSS pseudo-classes
- TestApp harness for headless testing with event injection and snapshot assertions

**Should have (competitive differentiators in the Rust ecosystem):**
- CSS selector query API: `app.query_one::<Button>()`, `app.query("#submit")`
- Hot-reload of `.tcss` stylesheets in development builds
- Screen stack: push/pop named screens with animated transitions
- Scrollable containers with automatic scrollbar management
- Rich text / Markdown display widget
- Reactive lists (`MutableVec`-backed list views that update incrementally)
- Terminal capability detection: graceful color degradation for terminals without true color
- Mouse hit-testing: click events delivered to the widget at the clicked cell

**Defer to v2+:**
- Image rendering (Sixel/kitty protocols)
- CSS animations and `tachyonfx`-style visual effects
- Custom fonts / big-text block rendering
- WebAssembly target
- Plugin/extension system

### Architecture Approach

textual-rs sits between ratatui (immediate-mode rendering engine) and application code,
providing the retained-mode abstraction layer that ratatui deliberately omits. The
architecture separates concerns into four parallel trees: the widget tree (slotmap arena of
`Box<dyn Widget>`), the computed style tree (parallel `SecondaryMap` with resolved CSS values
after cascade), the Taffy layout tree (mapped from computed styles, produces `Rect` per node),
and the reactive graph (signals on widget fields, effects that post render requests). All
widget tree manipulation runs on a single-threaded Tokio `LocalSet`, so widget state can use
`Rc<RefCell<T>>` rather than `Arc<Mutex<T>>`. Background I/O tasks use `tokio::spawn` on the
thread pool and communicate results back via `flume` or `tokio::sync::mpsc`.

**Major components:**
1. **EventLoop** — crossterm `EventStream` polled via `tokio::select!`; events normalized and dispatched via `flume` channel to the App; Windows key-release events filtered here
2. **WidgetArena** — `SlotMap<WidgetId, Box<dyn Widget>>` with `SecondaryMap` for parent/children, dirty flags, layout rects, and z-order; event bubbling walks the parent chain until `EventPropagation::Consumed`
3. **StyleEngine** — `cssparser`-based tokenizer feeds a hand-rolled `SelectorParser` and `PropertyParser`; `CascadeResolver` computes per-widget `ComputedStyle` sorted by specificity then source order; invalidated by structural changes or class/pseudo-class changes
4. **LayoutEngine** — `TaffyBridge` maps `ComputedStyle` to `taffy::Style`; `TaffyTree` computes geometry; results stored as `Rect` per widget in `SecondaryMap`; converts Taffy pixel coords to ratatui `Rect` for the render pass
5. **ReactiveGraph** — `RwSignal<T>` per reactive widget property; `Effect` nodes post `RenderRequest` to the event loop on change; dirty-flag gate prevents redundant draws within a tick
6. **RenderPass** — traverses `WidgetArena` in layout order; calls each widget's `render(&self, Rect, &mut Buffer)` which delegates to ratatui primitives; hands the completed `Buffer` to `Terminal::draw`
7. **TestApp** — `TestBackend`-backed harness exposing `press()`, `type_text()`, `click()`, `settle()`, `assert_text()`, `assert_snapshot()`

### Critical Pitfalls

1. **Retained widget tree vs. borrow checker** — You cannot hold `&mut Widget` from the arena and simultaneously pass `&mut Arena` to the widget's method. Solution: design `handle_event` and `render` to take an `AppContext` struct that does not contain a borrow to the current widget; access peers only via `WidgetId` lookups through `AppContext`. Never use `Rc<RefCell<dyn Widget>>` for deep trees — runtime borrow panics when a child handler tries to access its parent.

2. **`async fn` in trait objects is not object-safe** — Widget lifecycle methods (`on_mount`, `on_unmount`) that are naturally async cannot appear as `async fn` in the `Widget` trait without `async_trait` (which boxes every future). Solution: make widget handler methods synchronous; background work posts to the `flume` app channel and the result arrives as a subsequent message. Only the top-level application runner uses `async fn`.

3. **`reactive_graph` requires runtime executor initialization** — Without calling `Executor::init_tokio()` at startup, effects silently fail to schedule. This is a one-time setup call but easy to forget. Add it as the first line of `App::run()` and document it clearly in the framework's initialization path.

4. **Mouse hit-testing requires explicit cell-to-widget mapping** — Ratatui gives `Rect` per widget render call, but does not maintain a mouse map. textual-rs must maintain a `HashMap<(col, row), WidgetId>` updated after every layout pass. This is straightforward but must be designed upfront; retrofitting it after the fact is disruptive.

5. **Terminal color capability detection is not provided by crossterm** — If true color is rendered into a terminal that only supports 256 colors (some CI, older macOS Terminal.app), output is unpredictable. Solution: implement `$COLORTERM=truecolor`/`$TERM_PROGRAM` detection at startup and configure the style engine to emit 256-color fallbacks when true color is unavailable.

6. **Windows crossterm emits both KeyPress and KeyRelease** — On Windows, crossterm sends `KeyEventKind::Press` AND `KeyEventKind::Release` for every key. The EventLoop normalization step must filter to `Press` only (or expose both to handlers that explicitly opt in), or all key handlers will fire twice on Windows.

---

## Implications for Roadmap

The dependency graph between components is clear. The rendering pipeline (ratatui + crossterm +
Tokio event loop) is the foundation everything else builds on. Layout (Taffy) and styling
(cssparser) are parallel but layout must precede per-widget rendering work since widgets must
receive their computed `Rect` before they can render. Reactivity integrates into the widget
trait after the widget tree itself is stable. Built-in widgets cannot be tested meaningfully
until the TestApp harness exists. This ordering translates naturally into phases.

### Phase 1: Foundation — Rendering Pipeline and Event Loop

**Rationale:** Everything else depends on this. The ratatui + crossterm + Tokio plumbing must
work before any widget or CSS work can be tested end-to-end.
**Delivers:** A running event loop that renders a static ratatui widget to the terminal, handles
keyboard input, and quits cleanly. The project compiles and `cargo run` produces visible output.
**Key work:** Tokio `LocalSet` setup, crossterm `EventStream` integration via `flume`, ratatui
`Terminal::draw` loop, alternate screen + raw mode lifecycle, panic hook + terminal restore.
**Avoids:** Windows key-release doubling (filter in this phase, once, rather than in every widget).
**Research flag:** Standard patterns — well-documented in ratatui async tutorial. Skip research-phase.

### Phase 2: Widget Tree and Trait System

**Rationale:** The slotmap arena and `Widget` trait shape every subsequent decision. Lock this
down before building CSS or reactive systems on top of an interface that might change.
**Delivers:** A `WidgetArena` (SlotMap), a `Widget` trait with `render`, `handle_event`,
`on_mount`, and `on_unmount`, basic parent/child relationships via `SecondaryMap`, event
bubbling up the parent chain, and a skeletal focus system.
**Key work:** `WidgetId` type, `Box<dyn Widget>` storage, `AppContext` pattern for arena access
during event handling, `EventPropagation` enum, tab-order focus traversal.
**Avoids:** `async fn` in trait objects (keep handlers sync; document this constraint explicitly).
**Research flag:** The `SlotMap` remove-reinsert-same-key limitation needs a concrete solution
(HopSlotMap or `std::mem::replace` pattern) before this phase starts. Verify in a spike.

### Phase 3: Layout Engine (Taffy Integration)

**Rationale:** Widgets cannot render at the right position or size without a layout pass.
Taffy integration must come before CSS (CSS drives Taffy style inputs) to establish the
data flow: `ComputedStyle` -> `taffy::Style` -> `Rect` -> ratatui render call.
**Delivers:** A `TaffyBridge` that maps widget tree structure and hard-coded styles to a Taffy
tree; `compute_layout` produces a `Rect` per widget; the render pass uses these rects.
Basic Flexbox column/row layouts work. Fixed, percentage, and fill/fr sizing work.
**Key work:** `TaffyTree` lifecycle (add/remove nodes in sync with WidgetArena), `Rect`
conversion from Taffy's pixel space to terminal cells, absolute positioning for `dock`.
**Avoids:** Attempting to use ratatui's Cassowary solver for anything beyond 1D splits inside
individual widget render methods.
**Research flag:** Standard patterns — Taffy docs are HIGH confidence and iocraft demonstrates
the Taffy+terminal integration. Skip research-phase.

### Phase 4: CSS/TCSS Styling Engine

**Rationale:** With Taffy integration established, CSS properties have a clear destination.
The style engine produces `ComputedStyle` structs that feed directly into the Taffy bridge
and the ratatui `Style` passed to widget render calls.
**Delivers:** `.tcss` file parsing (cssparser tokenizer + hand-rolled property/selector
matching), `ComputedStyle` struct, cascade resolver with specificity-ordered rule application,
`:focus`/`:hover`/`:dark`/`:light` pseudo-classes, CSS custom properties (variables),
invalidation on structural or state changes.
**Key work:** `SelectorParser` (~200 lines), `PropertyParser` (~500 lines for full TCSS property
set), `CascadeResolver`, `ComputedStyleTree` (parallel `SecondaryMap`), dirty-flag invalidation.
**Avoids:** `lightningcss` (overkill for TCSS); `selectors` crate from Servo (excessive boilerplate
for a small selector surface).
**Research flag:** Selector matching for descendant combinators (`Screen Button`) requires
walking the parent chain during matching — confirm the performance characteristics are
acceptable at widget tree sizes of 100-500 nodes before optimizing.

### Phase 5: Reactive Property System

**Rationale:** Reactivity is the "magic" of Textual. It should come after the static rendering
pipeline is solid so the reactive layer can be verified against a known-good baseline.
**Delivers:** `RwSignal<T>` fields on widgets, `Effect` nodes that post `RenderRequest` to the
event loop, `Memo<T>` for derived state, dirty-flag batching to prevent multiple redraws per
tick, `watch_` callback pattern for custom side effects on property change.
**Key work:** `Executor::init_tokio()` at startup, `RenderRequest` message in the `flume`
event bus, debounce/batch multiple signal changes into a single render tick.
**Avoids:** Per-property `tokio::sync::watch` channels (scales poorly); direct terminal I/O
from effect closures (effects must only post messages, not render).
**Research flag:** This phase needs a proof-of-concept spike to validate `reactive_graph`
v0.1+ + Tokio `LocalSet` integration before committing. The `any_spawner` API needs
verification against the current published crate version. Mark this phase as needing a
research-phase during planning.

### Phase 6: TestApp Harness and Test Infrastructure

**Rationale:** By Phase 6 the core architecture is stable enough to lock in a test API that
other phases will rely on. Every widget built in Phase 7 should be driven by TestApp.
**Delivers:** `TestApp<A>` struct with `TestBackend`, event injection (`press`, `type_text`,
`click`), `settle()` for async event draining, `assert_text()`, `assert_snapshot()` via insta,
`assert_cell_fg/bg()` for color assertions. `proptest` integration for CSS parser and layout
engine property tests.
**Key work:** `TestApp` struct, `settle()` draining loop, `BufferAssertExt` trait, `rstest`
parameterized tests for CSS color parsing, `proptest` layout overflow invariants.
**Avoids:** PTY-based testing (`ratatui-testlib`) for unit/integration tests — TestBackend is
sufficient and faster. Reserve `ratatui-testlib` for end-to-end smoke tests only.
**Research flag:** Standard patterns — HIGH confidence across all testing decisions. Skip
research-phase.

### Phase 7: Built-in Widget Library

**Rationale:** With stable architecture, reactive system, CSS, layout, and test harness in
place, widget development is straightforward and highly parallelizable.
**Delivers:** Core widgets: `Label`, `Button`, `Input`, `TextArea`, `Checkbox`, `RadioSet`,
`Select`, `ProgressBar`, `DataTable`, `Tree`, `Log`, `Markdown`. Each widget has unit rendering
tests (TestBackend), behavior tests (TestApp event injection), and insta snapshots.
**Key work:** Each widget implements the `Widget` trait, declares reactive signals for its
mutable state, maps TCSS properties to ratatui `Style` and Taffy layout constraints.
**Avoids:** Wrapping third-party crates like `ratatui-textarea` or `tui-tree-widget` directly —
wrap them behind the textual-rs `Widget` trait so they participate in the CSS and reactive
systems rather than being opaque ratatui widgets.
**Research flag:** DataTable and Tree widgets have significant state complexity (virtual
scrolling, lazy loading). Flag these two specific widgets for research-phase during planning.

### Phase 8: Application Developer Experience (DX Polish)

**Rationale:** A framework is only as good as its ergonomics. This phase addresses the
developer-facing API surface that is hard to change after initial adoption.
**Delivers:** `#[derive(Widget)]` macro for boilerplate reduction, screen stack API
(`app.push_screen`/`pop_screen`), worker API (`self.run_worker(async_fn)`), CSS hot-reload
in debug builds, terminal color capability detection with graceful degradation, comprehensive
documentation and examples.
**Key work:** Proc-macro crate for `#[derive(Widget)]`, `ScreenStack` type, `WorkerHandle`
abstraction, `$COLORTERM` + `$TERM_PROGRAM` detection at startup.
**Avoids:** Shipping the hot-reload file watcher in production builds (feature-gate it).
**Research flag:** Proc-macro development for `#[derive(Widget)]` has known complexity around
span hygiene and IDE tooling. Flag for research-phase during planning.

### Phase Ordering Rationale

- **Phases 1-2 before everything else:** The rendering pipeline and widget trait are load-bearing
  for all other work. No CSS, reactive, or test work is possible without them.
- **Phase 3 (Layout) before Phase 4 (CSS):** CSS properties are meaningless without a layout
  engine to receive them. Establishing the `ComputedStyle` -> Taffy data flow first makes CSS
  a clean additive step.
- **Phase 5 (Reactive) after Phase 4 (CSS):** Reactive signals must trigger style invalidation
  and re-layout. The invalidation pipeline must exist before signals can be tested end-to-end.
- **Phase 6 (Testing) after Phase 5:** TestApp needs the reactive system to correctly drain
  async events in `settle()`. Tests written before Phase 5 would be incomplete.
- **Phase 7 (Widgets) last in core sequence:** Widgets are the consumer of every prior system.
  Building them last validates the entire stack and is highly parallelizable across contributors.
- **Phase 8 (DX) separate:** Developer experience work (macros, hot-reload) improves the API
  but does not block widget or application development. Keeps earlier phases focused.

### Research Flags

Phases likely needing `/gsd:research-phase` during planning:

- **Phase 2 (Widget Tree):** The `SlotMap` borrow problem (cannot hold `&mut Widget` and
  `&mut Arena` simultaneously) needs a concrete verified pattern before API design is locked.
  A 1-day spike is warranted.
- **Phase 5 (Reactive):** `reactive_graph` + Tokio `LocalSet` integration has MEDIUM confidence.
  The `any_spawner` initialization API must be verified against the current published version.
  Effect batching for render debounce needs a proof-of-concept. Flag for research-phase.
- **Phase 7 (Widgets — DataTable and Tree specifically):** Virtual scrolling and lazy loading
  in a retained widget tree with reactive signals needs design research. The ratatui ecosystem
  has `tui-tree-widget` and `ratatui-textarea` to learn from but not directly reuse.
- **Phase 8 (DX — proc-macro):** `#[derive(Widget)]` macro design needs research into span
  hygiene, field attribute parsing, and IDE (rust-analyzer) compatibility.

Phases with well-established patterns (skip research-phase unless a blocker surfaces):

- **Phase 1 (Event Loop):** Ratatui async tutorial is definitive. Tokio + crossterm EventStream
  + flume channel is a well-trodden pattern.
- **Phase 3 (Taffy Integration):** Taffy docs are HIGH confidence. iocraft demonstrates
  terminal-specific Taffy integration. The data flow is clear.
- **Phase 6 (Testing):** All testing decisions are HIGH confidence. TestBackend + insta +
  rstest + proptest is a well-understood combination.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Rendering stack (ratatui + crossterm) | HIGH | 19k stars, 73.7M downloads, official docs verified |
| Async runtime (Tokio + LocalSet) | HIGH | async-std discontinued; ecosystem consensus; crossterm EventStream Tokio-native |
| Widget tree ownership (slotmap) | HIGH | Multiple frameworks use this pattern; Raph Levien's Xilem research validates arena approach |
| Layout engine (Taffy) | HIGH | Used by Servo, Bevy, Zed, Slint; v0.9 docs verified; iocraft demonstrates TUI use |
| CSS parsing (cssparser + hand-rolled) | HIGH (tokenizer) / MEDIUM (selector/cascade) | cssparser API is stable and verified; selector matching pattern is well-documented but ~300 lines of new code |
| Reactive signals (reactive_graph) | MEDIUM | Leptos org crate with sparse docs; `any_spawner` + Tokio integration needs POC; pattern is sound |
| Message channels (flume) | HIGH | Official docs; battle-tested; sync+async bridge is the right choice |
| Testing strategy | HIGH | All major decisions verified against official ratatui docs, insta docs, Tokio testing docs |

**Overall confidence:** HIGH — all major technology choices are backed by official documentation or high-star production projects. The one MEDIUM area (reactive_graph integration) is de-risked by building it as Phase 5 with an explicit proof-of-concept requirement before committing.

### Gaps to Address

- **`reactive_graph` + `LocalSet` + batching:** No existing Rust TUI framework has published
  a reference implementation combining these three. A 1-2 day spike to verify
  `Executor::init_tokio()` works with `LocalSet` and that effects can be debounced into a
  single render tick should happen at the start of Phase 5 planning.

- **SlotMap borrow ergonomics:** The standard `SlotMap` does not support reinserting a value
  with the same key after removal. Determine whether `HopSlotMap` solves this or whether the
  `AppContext` pattern (arena access through a context object, never holding `&mut Widget`
  and `&mut Arena` simultaneously) is sufficient. Resolve this in a spike before Phase 2 API
  design.

- **Windows crossterm EventStream latency:** crossterm's `event-stream` feature uses
  `tokio::io::unix` on Unix but a blocking thread on Windows. Verify that this produces
  acceptable input latency on Windows (the dev platform) before shipping. Check if upgrading
  to crossterm 0.29 resolves any known Windows async input issues.

- **Color capability detection:** No well-maintained Rust crate exists for this. The
  `$COLORTERM`/`$TERM_PROGRAM`/terminfo detection must be hand-rolled. Budget this work
  explicitly in Phase 8 rather than treating it as free.

- **`assert_snapshot!` captures characters only, not colors:** insta snapshots via
  `terminal.backend()` do not capture ANSI style attributes. The `assert_cell_fg/bg`
  API in TestApp partially addresses this, but there is no color-aware snapshot format
  for TUI tests in the ecosystem yet. Document this limitation explicitly in the test harness.

---

## Sources

### Primary (HIGH confidence)

- [Ratatui GitHub](https://github.com/ratatui/ratatui) — star count, version, modular workspace, what ratatui does and does not provide
- [Ratatui FAQ](https://ratatui.rs/faq/) — explicit missing features list
- [Ratatui 0.30.0 Highlights](https://ratatui.rs/highlights/v030/) — ratatui-core separation, no_std
- [crossterm GitHub](https://github.com/crossterm-rs/crossterm) — 73.7M downloads, 0.29.0, Windows support
- [DioxusLabs/taffy GitHub](https://github.com/DioxusLabs/taffy) — v0.9.2, Style struct, Grid support
- [taffy docs.rs](https://docs.rs/taffy/latest/taffy/) — Style fields, compute_layout API
- [servo/rust-cssparser GitHub](https://github.com/servo/rust-cssparser) — CSS Syntax Level 3 tokenizer
- [reactive_graph docs.rs](https://docs.rs/reactive_graph/latest/reactive_graph/) — RwSignal, Memo, Effect, any_spawner
- [tokio::sync::watch docs](https://docs.rs/tokio/latest/tokio/sync/watch/index.html) — channel API
- [flume docs.rs](https://docs.rs/flume) — sync+async unified channel API
- [slotmap docs.rs](https://docs.rs/slotmap/latest/slotmap/) — SlotMap, SecondaryMap API
- [ratatui TestBackend docs](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html) — assert_buffer_lines, assert_cursor_position
- [ratatui Buffer docs](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Buffer.html) — cell access, diff API
- [insta.rs](https://insta.rs/) — snapshot macros, inline snapshot syntax, cargo-insta workflow
- [Tokio testing docs](https://tokio.rs/tokio/topics/testing) — #[tokio::test], time::pause, yield_now
- [ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — crossterm EventStream + tokio::select!
- [ryhl.io actors with Tokio](https://ryhl.io/blog/actors-with-tokio/) — actor/mailbox pattern
- [Raphlinus UI architecture](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html) — arena vs Rc<RefCell> analysis

### Secondary (MEDIUM confidence)

- [corrode.dev async guide](https://corrode.dev/blog/async/) — Tokio vs smol comparison, 2025
- [book.leptos.dev reactive_graph appendix](https://book.leptos.dev/appendix_reactive_graph.html) — signal/effect model
- [Ratatui component architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) — message dispatch patterns
- [Vizia styling docs](https://book.vizia.dev/quickstart/styling.html) — CSS-stylesheet-to-widget architecture lessons
- [telex-tui blog](https://telex-tui.github.io/blog/designing-a-tui-framework-in-rust.html) — Rc<RefCell> post-mortem, borrow pitfalls
- [ratatui snapshot testing guide](https://ratatui.rs/recipes/testing/snapshots/) — insta integration
- [Robinson browser engine: Style chapter](https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html) — cascade/specificity implementation pattern
- [Textual CSS guide](https://textual.textualize.io/guide/CSS/) — TCSS property reference
- [Textual testing guide](https://textual.textualize.io/guide/testing/) — Pilot API reference model
- [RUSTSEC-2023-0049](https://rustsec.org/advisories/RUSTSEC-2023-0049.html) — tui-rs abandoned

### Tertiary (LOW confidence — informational, needs validation)

- [iocraft GitHub](https://github.com/ccbrown/iocraft) — Taffy + terminal rendering POC (no published design doc)
- [weeklyrust.substack.com — goodbye async-std](https://weeklyrust.substack.com/p/goodbye-async-std-welcome-smol) — async-std discontinuation announcement
- [kanal GitHub](https://github.com/fereidani/kanal) — faster channel alternative to flume (benchmarks not independently verified)
- [ratatui-testlib GitHub](https://github.com/raibid-labs/ratatui-testlib) — PTY integration test harness (MEDIUM adoption, use for smoke tests only)

---

*Research completed: 2026-03-24*
*Ready for roadmap: yes*
