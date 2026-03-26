# Phase 5: Developer Experience and Polish - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Developer ergonomics layer that makes textual-rs practical for building real applications. Proc-macro derive reduces Widget trait boilerplate, Worker API enables background tasks, command palette provides discoverability, and polished demos + tutorial examples serve as documentation. This phase does NOT add new widgets, styling features, or framework capabilities — it wraps existing functionality in ergonomic APIs and proves the framework through beautiful example apps.

</domain>

<decisions>
## Implementation Decisions

### Derive Macro
- **D-01:** `#[derive(Widget)]` generates full boilerplate: `widget_type_name()` from struct name, `Cell<Option<WidgetId>>` field + `on_mount` wiring for self-ID access, `can_focus()` via `#[focusable]` attribute.
- **D-02:** `#[on(ButtonPressed)]` attribute macro generates `on_event()` dispatch — downcasts `&dyn Any` to the specified message type and calls the annotated handler method. Multiple `#[on(...)]` attributes compose into a single match chain.
- **D-03:** `#[keybinding("ctrl+s", "save")]` attribute generates `key_bindings()` returning a static slice and `on_action()` routing to the annotated handler. Multiple keybinding attributes compose into a single dispatch.
- **D-04:** Proc macro lives in a separate crate (`textual-rs-macros`) re-exported from the main crate. Standard Rust proc-macro workspace pattern.

### Worker API
- **D-05:** Message-based worker results. `ctx.run_worker(async { fetch_data().await })` spawns onto the Tokio LocalSet. On completion, result is delivered as a typed message (`WorkerResult<T>`) through the existing message queue. Widget handles via `#[on(WorkerResult<MyData>)]` or manual `on_event()`.
- **D-06:** Auto-cancel on unmount. Workers are tied to the spawning widget's lifetime. When the widget is unmounted (screen pop, widget removal), pending workers are dropped/cancelled. Prevents stale results arriving for dead widgets.

### Inter-Widget Communication
- **D-07:** `notify(message)` posts a message that bubbles up to ancestors (existing behavior via `post_message`). `app.post_message(target_id, message)` sends directly to any widget by ID. Both use the existing `message_queue` on AppContext.

### Command Palette
- **D-08:** Full `CommandPalette` widget that opens as a screen overlay (reuses Select's `push_screen_deferred` pattern). Auto-discovers commands from all mounted widgets' `key_bindings()` plus an app-level `CommandRegistry`.
- **D-09:** App-level command registration: `app.register_command("Open Settings", action)` for commands beyond key bindings. Fuzzy search filtering in the palette UI.
- **D-10:** Default trigger keybinding: `Ctrl+P` (configurable). Palette shows command name, source widget type, and keybinding if any.

### Demos
- **D-11:** Two example apps: `demo.rs` (widget showcase with tabbed categories — inputs, display, layout, scrollable) and `irc_demo.rs` (updated to use built-in widgets with real keyboard interaction).
- **D-12:** Visual style inspired by lazeport.pwn.zone — deep void dark backgrounds, green accent (#00ffa3), cyan highlights (#00d4ff), muted secondary text. Terminal-adapted: use ANSI approximations of the palette. Both demos should look beautiful and polished, not like test scaffolding.

### Documentation
- **D-13:** Rustdoc on every public type and method with `///` examples. `cargo doc` generates the full API reference.
- **D-14:** Tutorial examples as documentation: `examples/tutorial_01_hello.rs`, `examples/tutorial_02_widgets.rs`, `examples/tutorial_03_styling.rs`, etc. Heavily commented — each example IS a chapter of the getting-started guide. No separate mdbook or doc site.

### Claude's Discretion
- Exact proc-macro implementation strategy (syn/quote patterns)
- Worker cancellation mechanism (AbortHandle vs drop guard)
- Command palette fuzzy matching algorithm
- Tutorial example progression and content
- How many tutorial examples (3-6 range)
- Widget showcase tab organization

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Widget Trait (derive target)
- `crates/textual-rs/src/widget/mod.rs` — Widget trait definition with all 10 methods, required vs default
- `crates/textual-rs/src/widget/button.rs` — Reference implementation showing the full boilerplate pattern (Cell<WidgetId>, on_mount, key_bindings, on_action, on_event)
- `crates/textual-rs/src/widget/input.rs` — Complex widget with Reactive state, validator, multiple message types

### Event System (macro targets)
- `crates/textual-rs/src/event/keybinding.rs` — KeyBinding struct the macro must generate
- `crates/textual-rs/src/event/dispatch.rs` — Message dispatch and bubbling logic
- `crates/textual-rs/src/event/message.rs` — Message trait definition

### Worker API context
- `crates/textual-rs/src/widget/context.rs` — AppContext with message_queue, event_tx, pending_screen_pushes
- `crates/textual-rs/src/app.rs` — App::run_async with Tokio LocalSet, Owner initialization

### Screen overlay pattern (command palette)
- `crates/textual-rs/src/widget/select.rs` — push_screen_deferred/pop_screen_deferred pattern for overlay screens

### Visual reference
- `https://lazeport.pwn.zone` — Color palette and dark aesthetic to adapt for terminal demos

### Existing demos (to be updated)
- `crates/textual-rs/examples/demo.rs` — Currently blank stub, needs full rewrite
- `crates/textual-rs/examples/irc_demo.rs` — Hardcoded ratatui widgets, needs conversion to built-in widgets

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AppContext::post_message()` — Already handles message posting to queue; `notify()` can wrap this
- `AppContext::push_screen_deferred()` / `pop_screen_deferred()` — Overlay pattern ready for CommandPalette
- `Tabs`/`TabbedContent` widget — Can be used inside the widget showcase demo
- All 22 built-in widgets — Available for demos and tutorials
- `TestApp`/`Pilot` harness — Testing infrastructure for all new features

### Established Patterns
- `Cell<Option<WidgetId>>` + `on_mount` — Self-ID pattern every interactive widget uses (derive macro target)
- Static `&[KeyBinding]` slices — Zero-allocation key binding pattern (macro must generate these)
- `Reactive<T>` for widget state — Worker results should integrate with this
- Message structs in widget module `messages` submodule — Convention the derive macro should follow

### Integration Points
- `crates/textual-rs/Cargo.toml` — New `textual-rs-macros` crate dependency
- `crates/textual-rs/src/lib.rs` — Re-export derive macro
- `App::new()` — CommandRegistry initialization point
- `App::run_async()` — Worker spawning onto LocalSet

</code_context>

<specifics>
## Specific Ideas

- Demos should look like they belong on a hacker's terminal — dark void backgrounds, green/cyan accent colors from lazeport.pwn.zone palette
- IRC demo should feel like a real weechat-style client with working keyboard navigation
- Widget showcase should be a living catalogue you'd actually use to explore the framework
- Tutorial examples should be self-contained — `cargo run --example tutorial_01_hello` teaches you something complete

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-developer-experience-and-polish*
*Context gathered: 2026-03-26*
