# Phase 3: Reactive System, Events, and Testing - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Widget state changes automatically trigger re-renders via reactive signals; typed messages bubble up the widget tree and are handled via `on_event` dispatch with downcasting; keyboard/mouse events route to the correct widget through focus and hit-map; and a `TestApp`/`Pilot` harness lets tests simulate user interaction with no real terminal. This phase delivers the reactive property system, event/message dispatch, input routing, and test infrastructure. Built-in widgets are Phase 4.

</domain>

<decisions>
## Implementation Decisions

### Reactive Property System
- **D-01:** Use `reactive_graph` crate (Leptos signals). `RwSignal<T>`, `Memo<T>`, `Effect` for automatic dependency tracking. Integrate with Tokio LocalSet via `Executor::init_tokio()`. Research phase MUST include a spike verifying this integration works before planning commits to it. If the spike fails, fall back to hand-rolled signals.
- **D-02:** Render batching via Effect + flume. A `reactive_graph` Effect detects reactive changes and posts a `RenderRequest` to the existing flume event channel. The App event loop coalesces multiple `RenderRequest`s into a single render pass per tick. Multiple reactive field changes in one tick produce exactly one re-render.
- **D-03:** `Reactive<T>` wraps `RwSignal<T>` with textual-rs ergonomics. `watch_` method convention: a method called automatically when the reactive property changes. `validate_` convention: validate/coerce on set. `compute_` convention: derive from reactive sources (maps to `Memo<T>`).

### Event Dispatch Model
- **D-04:** `on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation` added to Widget trait. Widgets downcast `event` to concrete message types they handle. Returns `EventPropagation::Stop` to consume or `::Continue` to bubble. No handler registration map needed — simple, extensible, no TypeId registry.
- **D-05:** Bubbling walks the parent chain. Collect `Vec<WidgetId>` from widget to screen root, then iterate calling `on_event` at each level. Since `on_event` takes `&self` (immutable borrow), it can safely read AppContext. Propagation stops when any handler returns `Stop`.
- **D-06:** Typed messages implement a `Message` trait. Widgets define their own message structs (e.g., `Button::Pressed`, `Input::Changed`). Messages are posted to a queue and dispatched through the bubbling mechanism.

### Keyboard and Mouse Routing
- **D-07:** Key events dispatch to focused widget first, then bubble up the parent chain. If no widget consumes the key, the App handles it (Tab for focus, quit keys, etc.). Matches Textual's model and DOM event flow.
- **D-08:** Mouse events use the existing `MouseHitMap` from Phase 2 for hit testing. Click events dispatch to the topmost widget at the clicked cell, then bubble up.
- **D-09:** Key bindings via static binding table. Widget trait gets `fn key_bindings(&self) -> &[KeyBinding]` returning entries like `(Key::Char('q'), "quit")`. Action strings dispatch to `fn on_action(&self, action: &str, ctx: &AppContext)`. Matches Textual's BINDINGS pattern. Enables Footer widget to discover and display bindings.
- **D-10:** Timer/interval support for periodic updates (EVENT-08). Implementation details at Claude's discretion — likely a tokio::time::interval that posts Tick events to the flume channel.

### Test Infrastructure
- **D-11:** Async tests with `#[tokio::test]`. TestApp creates its own LocalSet internally. Pilot methods are async (`.press(Key::Tab).await`). `settle().await` drains the event loop until no pending messages remain. This matches the async runtime reality.
- **D-12:** `TestApp::new(|| Box::new(MyScreen))` wraps App with TestBackend. Returns a handle with `pilot()` for sending events and `app()` for inspecting state. No real terminal needed.
- **D-13:** Snapshot testing with `insta` using plain text buffer lines. Render to TestBackend, extract buffer rows as strings, snapshot with `insta::assert_snapshot!`. Human-readable diffs. Each row is a line of characters. Matches ratatui's own test patterns.
- **D-14:** `assert_buffer_lines()` helper for cell-level assertions without full snapshots. `proptest` for CSS parser and layout engine fuzz testing.

### Claude's Discretion
- Timer/interval implementation details
- Exact `Reactive<T>` API surface beyond the conventions (get/set/update methods)
- Whether `on_event` takes `&mut AppContext` or `&AppContext` (borrow analysis needed)
- proptest strategy design for CSS parser fuzzing
- Mouse event types beyond click (drag, scroll, hover)
- How `settle()` detects quiescence (empty queues + no pending effects)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python Textual Reference (reactive + events)
- `textual/src/textual/reactive.py` -- Reactive descriptor, watch/validate/compute conventions
- `textual/src/textual/message.py` -- Message base class, bubbling, handler discovery
- `textual/src/textual/message_pump.py` -- MessagePump: event queue, dispatch, timer management
- `textual/src/textual/on.py` -- @on decorator for message handler registration
- `textual/src/textual/binding.py` -- Key bindings: Binding class, action dispatch
- `textual/src/textual/events.py` -- Event types: Key, Click, Mount, Unmount, Resize, Timer
- `textual/src/textual/pilot.py` -- Test pilot: press, click, type_text, wait_for_animation
- `textual/src/textual/app.py` -- App.run_test() for headless testing

### Existing Rust Code (Phase 2 output)
- `crates/textual-rs/src/app.rs` -- App struct with AppContext + TaffyBridge + event loop; Phase 3 extends event dispatch
- `crates/textual-rs/src/widget/mod.rs` -- Widget trait with EventPropagation enum already defined
- `crates/textual-rs/src/widget/context.rs` -- AppContext with arena, focus, dirty flags; Phase 3 adds message queues
- `crates/textual-rs/src/widget/tree.rs` -- advance_focus, parent chain traversal; Phase 3 uses for event bubbling
- `crates/textual-rs/src/layout/hit_map.rs` -- MouseHitMap for click routing
- `crates/textual-rs/src/event.rs` -- AppEvent enum; Phase 3 adds RenderRequest, MessageDispatched variants

### Project Planning Documents
- `.planning/REQUIREMENTS.md` -- Phase 3 covers REACT-01..05, EVENT-01..08, TEST-01..06
- `.planning/ROADMAP.md` -- Phase 3 plan details, success criteria, research spike note
- `.planning/PROJECT.md` -- Key decisions: reactive_graph (MEDIUM confidence), Tokio LocalSet

### Crate Documentation
- `reactive_graph` crate docs -- RwSignal, Memo, Effect, Executor::init_tokio()

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `EventPropagation` enum in `widget/mod.rs` -- Already defined with Stop/Continue variants, ready for event dispatch
- `MouseHitMap` in `layout/hit_map.rs` -- O(1) cell-to-widget hit testing, ready for mouse routing
- `advance_focus` / `advance_focus_backward` in `widget/tree.rs` -- Focus traversal already works, Phase 3 adds event routing to focused widget
- `AppEvent` enum in `event.rs` -- Extend with `RenderRequest` for reactive batching
- `flume` channel in `app.rs` -- Event bus already established, Phase 3 adds message types
- `input_buffer` on `AppContext` -- Temporary hack from IRC demo; Phase 3 replaces with proper Input widget reactive state

### Established Patterns
- Tokio LocalSet + flume for event bus -- all widget code runs on LocalSet, no Send requirements
- AppContext borrow pattern -- Widget methods take `&self` + `&AppContext` for reads; mutations go through tree.rs functions that take `&mut AppContext`
- `anyhow::Result` for error propagation
- DFS tree traversal for rendering and focus -- reuse for event bubbling (parent chain walk)

### Integration Points
- `App::run_async()` event loop match arms -- Phase 3 adds: key→dispatch_to_focused, mouse→hit_test→dispatch, RenderRequest→full_render_pass
- `Widget` trait -- Phase 3 adds: on_event(), key_bindings(), on_action()
- `AppContext` -- Phase 3 adds: message queue, reactive signal storage
- TestBackend from Phase 1 integration tests -- Phase 3 wraps in TestApp for ergonomic test harness

</code_context>

<specifics>
## Specific Ideas

- The `reactive_graph` spike is critical -- if `Executor::init_tokio()` doesn't play well with LocalSet, we need to know before planning 3 plans around it. The spike should be a standalone test: create a LocalSet, init the executor, create signals, modify them, verify effects fire.
- Key binding discovery enables the Footer widget in Phase 4 to automatically display available shortcuts -- design the `key_bindings()` return type with this in mind.
- The `on_event` approach with `dyn Any` downcasting is the simplest Rust pattern for typed dispatch without a macro. Phase 5's `#[derive(Widget)]` can generate boilerplate if needed.
- `settle()` needs to be carefully designed -- it must drain both the flume event queue AND any pending reactive effects before returning. This is the key primitive that makes async tests deterministic.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 03-reactive-system-events-and-testing*
*Context gathered: 2026-03-25*
