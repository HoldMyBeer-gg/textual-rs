# Phase 5: Developer Experience and Polish - Research

**Researched:** 2026-03-25
**Domain:** Rust proc-macros (syn/quote), Tokio worker spawning, inter-widget communication, command palette UI, tutorial documentation patterns
**Confidence:** HIGH (all findings verified against codebase source + cargo registry)

## Summary

Phase 5 wraps the existing textual-rs framework in ergonomic APIs. Four new capabilities (derive macro, worker API, command palette, notify API) layer on top of already-working primitives from Phases 1-4. The derive macro is the highest-stakes item: it requires a new `textual-rs-macros` workspace crate with `proc-macro = true` and must generate code that compiles correctly against the exact Widget trait shape already in the codebase. The Worker API integrates with the existing Tokio `LocalSet` and `message_queue` on `AppContext`. The command palette reuses the Select widget's `push_screen_deferred` overlay pattern. Documentation is entirely Rustdoc + examples — no external toolchain.

The codebase is in excellent shape for this phase. All 22 widgets are done, the test harness works, and every primitive the new APIs need (post_message, push_screen_deferred, Reactive, drain_message_queue) already exists and is tested. The main risk is the proc-macro crate workspace integration — getting the Cargo workspace, crate features, and re-export right from the first task.

**Primary recommendation:** Create `textual-rs-macros` as the first task (P01), get the workspace plumbing correct, generate a single method stub, and verify `cargo build` passes before implementing all four derive capabilities.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Derive Macro**
- D-01: `#[derive(Widget)]` generates full boilerplate: `widget_type_name()` from struct name, `Cell<Option<WidgetId>>` field + `on_mount` wiring for self-ID access, `can_focus()` via `#[focusable]` attribute.
- D-02: `#[on(ButtonPressed)]` attribute macro generates `on_event()` dispatch — downcasts `&dyn Any` to the specified message type and calls the annotated handler method. Multiple `#[on(...)]` attributes compose into a single match chain.
- D-03: `#[keybinding("ctrl+s", "save")]` attribute generates `key_bindings()` returning a static slice and `on_action()` routing to the annotated handler. Multiple keybinding attributes compose into a single dispatch.
- D-04: Proc macro lives in a separate crate (`textual-rs-macros`) re-exported from the main crate. Standard Rust proc-macro workspace pattern.

**Worker API**
- D-05: Message-based worker results. `ctx.run_worker(async { fetch_data().await })` spawns onto the Tokio LocalSet. On completion, result is delivered as a typed message (`WorkerResult<T>`) through the existing message queue. Widget handles via `#[on(WorkerResult<MyData>)]` or manual `on_event()`.
- D-06: Auto-cancel on unmount. Workers are tied to the spawning widget's lifetime. When the widget is unmounted (screen pop, widget removal), pending workers are dropped/cancelled. Prevents stale results arriving for dead widgets.

**Inter-Widget Communication**
- D-07: `notify(message)` posts a message that bubbles up to ancestors (existing behavior via `post_message`). `app.post_message(target_id, message)` sends directly to any widget by ID. Both use the existing `message_queue` on AppContext.

**Command Palette**
- D-08: Full `CommandPalette` widget that opens as a screen overlay (reuses Select's `push_screen_deferred` pattern). Auto-discovers commands from all mounted widgets' `key_bindings()` plus an app-level `CommandRegistry`.
- D-09: App-level command registration: `app.register_command("Open Settings", action)` for commands beyond key bindings. Fuzzy search filtering in the palette UI.
- D-10: Default trigger keybinding: `Ctrl+P` (configurable). Palette shows command name, source widget type, and keybinding if any.

**Demos**
- D-11: Two example apps: `demo.rs` (widget showcase with tabbed categories) and `irc_demo.rs` (updated to use built-in widgets with real keyboard interaction).
- D-12: Visual style inspired by lazeport.pwn.zone — deep void dark backgrounds, green accent (#00ffa3), cyan highlights (#00d4ff), muted secondary text. Terminal-adapted ANSI approximations.

**Documentation**
- D-13: Rustdoc on every public type and method with `///` examples. `cargo doc` generates the full API reference.
- D-14: Tutorial examples as documentation: `examples/tutorial_01_hello.rs`, etc. Heavily commented — each example IS a chapter of the getting-started guide. No separate mdbook or doc site.

### Claude's Discretion
- Exact proc-macro implementation strategy (syn/quote patterns)
- Worker cancellation mechanism (AbortHandle vs drop guard)
- Command palette fuzzy matching algorithm
- Tutorial example progression and content
- How many tutorial examples (3-6 range)
- Widget showcase tab organization

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DX-01 | `#[derive(Widget)]` proc-macro for common widget boilerplate | syn 2.0 + quote 1.0 crate patterns; exact Widget trait shape read from `widget/mod.rs`; workspace crate pattern documented |
| DX-02 | Worker API for running blocking tasks without blocking event loop | Tokio `spawn_local` on existing LocalSet; `AbortHandle` for cancellation; `WorkerResult<T>` message through existing `message_queue` |
| DX-03 | `notify()` / `post_message()` API for inter-widget communication | `AppContext::post_message()` already exists; `notify()` is a thin wrapper from widget context; `post_message` with target_id already supported |
| DX-04 | Application-level command palette support | `push_screen_deferred` overlay pattern verified in `select.rs`; `CommandRegistry` new field on `App`; fuzzy matching via `strsim` |
| DX-05 | Comprehensive documentation with examples matching Textual's guide structure | Rustdoc + `examples/` approach; no external toolchain; `cargo doc --no-deps` workflow |
</phase_requirements>

## Standard Stack

### Core (proc-macro crate)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| syn | 2.0.117 | Parse Rust token streams in proc-macros | Industry standard; syn 2.x is the current stable API |
| quote | 1.0.45 | Generate Rust token streams from quasi-quote templates | Companion to syn; every proc-macro uses this |
| proc-macro2 | 1.0.106 | Proc-macro token types usable outside proc-macro context | Required by syn/quote; enables testing macro logic |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| strsim | 0.11.1 | String similarity (Jaro-Winkler, Levenshtein) for fuzzy command palette search | Command palette search — simple, zero-dependency, sufficient for < 200 commands |
| tokio-util | 0.7.18 | `CancellationToken` for worker lifecycle management | Worker cancellation (alternative: `tokio::task::AbortHandle` from tokio 1.x which is already a dependency) |

**Recommendation for worker cancellation:** Use `tokio::task::AbortHandle` (already available via existing `tokio = "1"` dep, no new dep needed). `AbortHandle` is returned by `spawn_local` and can be stored per widget, dropped on unmount.

### Cargo Workspace Pattern for Proc-Macro Crate
```toml
# Cargo.toml (workspace root) — add member:
members = ["crates/textual-rs", "crates/textual-rs-macros"]

# crates/textual-rs-macros/Cargo.toml:
[package]
name = "textual-rs-macros"
version = "0.1.0"
edition.workspace = true

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"

# crates/textual-rs/Cargo.toml — add dependency:
textual-rs-macros = { path = "../textual-rs-macros" }
```

**Re-export in lib.rs:**
```rust
// In crates/textual-rs/src/lib.rs:
pub use textual_rs_macros::{Widget, on, keybinding};
```

**Installation:**
```bash
# No new installs — syn/quote/proc-macro2 added as deps to the new macros crate
```

**Version verification:** Confirmed via `cargo search` on 2026-03-25:
- syn 2.0.117 (current)
- quote 1.0.45 (current)
- proc-macro2 1.0.106 (current)
- strsim 0.11.1 (current)

## Architecture Patterns

### Recommended Project Structure
```
crates/
├── textual-rs/              # main library crate (unchanged members + new macros re-export)
│   ├── src/
│   │   ├── worker/          # NEW: Worker API (worker.rs, worker_result.rs)
│   │   ├── command/         # NEW: CommandRegistry + CommandPalette widget
│   │   └── lib.rs           # adds pub use textual_rs_macros::{Widget, on, keybinding}
│   └── examples/
│       ├── demo.rs          # REWRITE: full widget showcase
│       ├── irc_demo.rs      # REWRITE: use built-in widgets
│       ├── tutorial_01_hello.rs
│       ├── tutorial_02_widgets.rs
│       ├── tutorial_03_styling.rs
│       ├── tutorial_04_events.rs
│       └── tutorial_05_workers.rs (optional)
└── textual-rs-macros/       # NEW: proc-macro crate
    ├── Cargo.toml           # proc-macro = true
    └── src/
        ├── lib.rs           # pub use derive_widget, on_attribute, keybinding_attribute
        ├── derive_widget.rs # #[derive(Widget)] implementation
        ├── on_attr.rs       # #[on(MessageType)] implementation
        └── keybinding_attr.rs # #[keybinding("ctrl+s", "save")] implementation
```

### Pattern 1: proc-macro Derive (DeriveInput → TokenStream)

**What:** Parse struct fields/attributes with syn, generate impl block with quote.
**When to use:** For all three macros in this phase.

```rust
// Source: syn 2.x derive macro pattern
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Widget, attributes(focusable, on, keybinding))]
pub fn derive_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Check for #[focusable] attribute on struct
    let can_focus_val = input.attrs.iter().any(|a| a.path().is_ident("focusable"));

    let expanded = quote! {
        impl ::textual_rs::widget::Widget for #name {
            fn widget_type_name(&self) -> &'static str {
                #name_str
            }
            fn can_focus(&self) -> bool {
                #can_focus_val
            }
            fn on_mount(&self, id: ::textual_rs::widget::WidgetId) {
                self.own_id.set(Some(id));
            }
            fn on_unmount(&self, _id: ::textual_rs::widget::WidgetId) {
                self.own_id.set(None);
            }
        }
    };
    TokenStream::from(expanded)
}
```

**Critical constraint:** The derive macro generates an `impl Widget` block but does NOT provide the `own_id: Cell<Option<WidgetId>>` field — the struct must declare that field itself, or the macro injects it via `#[derive(Widget)]` on the struct fields. The cleanest approach is to require the struct to have `own_id: std::cell::Cell<Option<::textual_rs::WidgetId>>` declared, and the macro wires it in `on_mount`. Document this in rustdoc on the derive macro.

### Pattern 2: Attribute Macro for #[on(MessageType)]

**What:** Proc-macro attribute that wraps a method, injecting a downcast dispatch match into `on_event`.
**When to use:** Any widget method that handles a specific message type.

The tricky part is that multiple `#[on(...)]` attributes on different methods must be collected into a **single** `on_event` impl. The correct approach: collect all `#[on(T)]`-annotated methods in the derive pass (not as standalone attribute macros), then generate one combined `on_event` body. This requires `#[derive(Widget)]` to also handle `#[on]` collection.

Alternatively (simpler): `#[on(T)]` is an inert attribute recognized by `#[derive(Widget)]` (declared via `attributes(on)` in the derive). The derive proc-macro scans all methods in the impl block for `#[on(T)]` attrs. This is the correct pattern — it keeps one derive entry point.

**However:** proc-macro derives operate on struct definitions, not impl blocks. The standard solution is to make `#[on(...)]` a proc-macro attribute (`#[proc_macro_attribute]`) applied to methods that expands to an `on_event` collector. Consult real-world practice: Leptos and Actix use accumulator patterns via `inventory` crate for this. For simplicity in textual-rs: use an attribute proc-macro on the impl block (not individual methods) that scans method attrs.

**Simplest viable approach:** Make `#[on(ButtonPressed)]` a method-level attribute macro that stores metadata in a thread-local, then have the `#[derive(Widget)]` collect from that store. This is fragile. **Better approach:** Have `#[on(...)]` be an attribute applied to the entire `impl MyWidget {}` block, which scans the block for annotated methods and rewrites `on_event`. This is cleanly achievable with syn's `ItemImpl` parsing.

```rust
// #[on(...)] attribute on impl block
#[proc_macro_attribute]
pub fn widget_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(input as syn::ItemImpl);
    // Collect methods with #[on(T)] attrs
    // Generate fn on_event matching on those types
    // Remove the #[on(T)] helper attrs from methods
    ...
}
```

### Pattern 3: Worker API Integration

**What:** `ctx.run_worker(widget_id, future)` spawns onto Tokio LocalSet, delivers `WorkerResult<T>` via message queue on completion.
**When to use:** Any async or blocking operation that should not block the event loop.

```rust
// In crates/textual-rs/src/worker.rs
use tokio::task::{AbortHandle, JoinHandle};

pub struct WorkerResult<T> {
    pub value: T,
}
impl<T: 'static> crate::event::message::Message for WorkerResult<T> {}

// On AppContext (new method):
pub fn run_worker<T: 'static + Send>(
    &self,
    source_id: WidgetId,
    fut: impl Future<Output = T> + 'static,
) -> AbortHandle {
    let tx = self.event_tx.clone().expect("event_tx not set");
    let queue = // need to inject result back via message_queue
    // spawn_local returns JoinHandle; .abort_handle() gives AbortHandle
    let handle = tokio::task::spawn_local(async move {
        let result = fut.await;
        // post WorkerResult<T> to message_queue
        // ... but we can't borrow AppContext from inside spawn_local
    });
    handle.abort_handle()
}
```

**Key constraint — AppContext borrowing in spawn_local:** `spawn_local` closures cannot borrow `AppContext` (not `Send + 'static`). The worker must communicate back via the `flume` event channel (`event_tx`). The cleanest design:

1. Worker future completes, wraps result in `Box<dyn Any + Send>` and sends it via `event_tx` as a new `AppEvent::WorkerResult { source_id, payload }` variant.
2. Main event loop receives `AppEvent::WorkerResult`, unwraps payload, posts to `message_queue` as `(source_id, payload)`.
3. `drain_message_queue` dispatches as usual.

This requires adding `WorkerResult` variant to `AppEvent` enum. **Check:** `AppEvent` currently has `Key`, `Mouse`, `Resize`, `RenderRequest`. Adding `WorkerResult { source_id: WidgetId, payload: Box<dyn Any + Send> }` requires `Box<dyn Any + Send>` — `Send` is satisfiable if T: Send.

**Constraint D-05 says** result is delivered via message queue. The `event_tx`-based relay is the implementation mechanism.

**Abort on unmount (D-06):** Store `Vec<AbortHandle>` per widget in `AppContext` (secondary map `worker_handles: SecondaryMap<WidgetId, Vec<AbortHandle>>`). On unmount (called from `widget/tree.rs`), drain the list and call `abort()` on each handle.

### Pattern 4: Command Palette Overlay

**What:** `CommandPalette` is a `Box<dyn Widget>` pushed via `push_screen_deferred`. It renders a full-screen overlay with a search input and filtered command list. Selecting a command calls the registered action closure.
**When to use:** App-level command discovery, `Ctrl+P` trigger.

```rust
// In crates/textual-rs/src/command/mod.rs
pub struct Command {
    pub name: &'static str,
    pub source: &'static str,      // widget type name or "app"
    pub keybinding: Option<String>, // display only
    pub action: Box<dyn Fn(&AppContext) + 'static>,
}

pub struct CommandRegistry {
    pub commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn register(&mut self, name: &'static str, action: impl Fn(&AppContext) + 'static) {
        self.commands.push(Command { name, source: "app", keybinding: None, action: Box::new(action) });
    }
    pub fn discover_from_tree(&self, ctx: &AppContext) -> Vec<Command> { ... }
}
```

`CommandRegistry` lives on `App` (new field). The existing global quit binding `Ctrl+P` intercept goes in `run_async` event loop — when `Ctrl+P` fires, call `ctx.push_screen_deferred(Box::new(CommandPalette::new(registry.collect(ctx))))`.

**Fuzzy matching:** Use `strsim::jaro_winkler` (in `strsim 0.11.1`) — no new dependency complexity, good enough for < 200 commands.

### Pattern 5: notify() API

`notify()` is a convenience wrapper on `AppContext`. Decision D-07 says it posts a message that bubbles up to ancestors via the existing `post_message` mechanism. Since `post_message` already dispatches via bubbling in `drain_message_queue` → `dispatch_message`, `notify()` is:

```rust
// Widget trait default method (or trait extension):
fn notify(&self, message: impl Any + 'static, ctx: &AppContext) {
    if let Some(id) = self.own_id_opt() {
        ctx.post_message(id, message);
    }
}
```

This requires that widgets expose their `own_id` — which the derive macro will ensure (via `Cell<Option<WidgetId>>` field). For hand-rolled widgets without the derive, they can call `ctx.post_message(id, msg)` directly.

`app.post_message(target_id, message)` already exists as `ctx.post_message(source, message)`. The "target" semantics differ from "source" in the current implementation — `dispatch_message` dispatches from source upward. For a true "send to target by ID" semantic, the implementation should call `dispatch_message(target_id, ...)` directly, which dispatches from target up. This is already the mechanism; the new API just provides named access via `App`.

### Anti-Patterns to Avoid
- **Proc-macro crate not in workspace:** If `textual-rs-macros` is not in `[workspace.members]`, it won't share the `rust-version` workspace key and will fail with confusing errors.
- **Using `proc_macro` types in lib.rs:** `proc_macro::TokenStream` is only available in proc-macro crates. Use `proc_macro2::TokenStream` for any shared logic, then convert at the entry point.
- **Borrowing AppContext inside spawn_local:** `AppContext` is not `Send`. Workers must communicate back via the flume channel, not by capturing `&AppContext`.
- **Storing Box<dyn Fn()> in CommandRegistry with Widget derive:** Closures are not `Clone` or `Debug`. Avoid deriving those traits on `Command` or `CommandRegistry`.
- **AbortHandle on drop race:** `AbortHandle::abort()` is safe to call even if the task already completed. No guard needed.
- **Fuzzy search with full Levenshtein on every keystroke:** Use Jaro-Winkler or simple prefix + substring for real-time search on < 200 commands. Full Levenshtein is overkill and adds latency.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parsing Rust token streams | Custom tokenizer | syn 2.0 | syn handles the full Rust grammar; edge cases in attributes, generics, lifetimes are enormous |
| Token stream code generation | String formatting | quote! macro | Quote handles hygiene, escaping, and nested token trees correctly |
| String similarity for fuzzy search | Edit distance loop | strsim::jaro_winkler | Battle-tested, handles Unicode, no off-by-one bugs |
| Task cancellation | Channel drop tricks | tokio::task::AbortHandle | Correct async cancellation; drop-on-cancel has race conditions |
| Proc-macro token types outside proc-macro context | Re-implement token types | proc-macro2 | Testing macro logic requires proc-macro2 since `proc_macro` is compiler-internal |

**Key insight:** Proc-macros have no runtime — they are compile-time text transformations. Every edge case (generics, where clauses, visibility modifiers, nested types) that syn handles for free becomes a manual parsing nightmare if done by hand.

## Common Pitfalls

### Pitfall 1: Proc-Macro Crate Circular Dependency
**What goes wrong:** `textual-rs-macros` imports types from `textual-rs` to use in generated code paths. This creates a circular dependency.
**Why it happens:** Generated code refers to `::textual_rs::widget::WidgetId` by path — this is a reference in generated source, not a Rust dependency. The macro crate itself does NOT need to depend on `textual-rs`.
**How to avoid:** Use fully-qualified paths in quote! output (e.g., `::textual_rs::widget::WidgetId`), not imports. The macro crate depends only on `syn`, `quote`, `proc-macro2`.
**Warning signs:** `Cargo.toml` for `textual-rs-macros` listing `textual-rs` as a dependency.

### Pitfall 2: #[derive(Widget)] on Non-Struct Types
**What goes wrong:** Derive macro panics or gives confusing error when applied to enum or tuple struct.
**Why it happens:** Widget trait is struct-centric (it requires a `own_id` field).
**How to avoid:** Add a compile-time assertion in the derive: `if let syn::Data::Struct(_) = &input.data { ... } else { return syn::Error::new(..., "Widget derive only supports named structs").to_compile_error().into() }`.
**Warning signs:** User applies `#[derive(Widget)]` to an enum, gets a confusing rustc error about missing field.

### Pitfall 3: WorkerResult<T> Requires T: Send
**What goes wrong:** Worker future produces `T` that is not `Send` (e.g., contains `Rc`, `Cell`). Passing it through the flume channel requires `T: Send`.
**Why it happens:** The relay path goes `spawn_local` → flume channel → main loop. Flume channels require `Send` types.
**How to avoid:** Document that `run_worker` requires `T: Send + 'static`. For non-Send results, use `RefCell<Option<T>>` on the widget and a `RenderRequest` trigger instead.
**Warning signs:** Compiler error "T is not Send" on `ctx.run_worker(...)` call.

### Pitfall 4: Command Palette Overlay Stealing Focus
**What goes wrong:** CommandPalette is pushed as a new screen; after pop, focus is not restored to the original widget.
**Why it happens:** `push_screen` / `pop_screen` in `widget/tree.rs` do not save/restore focused widget.
**How to avoid:** CommandPalette should save `ctx.focused_widget` on open and call `pop_screen_deferred` + restore focus when dismissed. Verify this path with a test.
**Warning signs:** After closing command palette, Tab cycling starts from scratch (focus is `None`).

### Pitfall 5: rust-analyzer Red Underlines on Derive
**What goes wrong:** `#[derive(Widget)]` compiles correctly but rust-analyzer shows red underlines for `own_id` field usage.
**Why it happens:** Proc-macros inject fields that rust-analyzer's incremental analysis doesn't see until re-analysis.
**How to avoid:** The derive macro must NOT inject `own_id` as a struct field — it must be declared by the user. The macro only generates the impl block. This keeps the struct layout explicit and visible to rust-analyzer. Document clearly that users must declare `own_id: std::cell::Cell<Option<textual_rs::WidgetId>>`.
**Warning signs:** Success criterion 1 says "recognized by rust-analyzer without IDE red-underlines" — test by actually opening VS Code after implementing.

### Pitfall 6: Multiple on_event Impls from Multiple #[on] Calls
**What goes wrong:** Rust forbids multiple `impl Widget for Foo` blocks. If each `#[on(T)]` attribute generates a separate `on_event` impl, compilation fails.
**Why it happens:** Naive attribute macro implementation generates a new impl block per annotation.
**How to avoid:** Use the "attribute on the impl block" approach: `#[on]` as an attribute on `impl MyWidget { }`, scanning all method annotations to generate one combined `on_event`. All `#[on(T)]` annotations on methods become arms in the generated `on_event` match chain.
**Warning signs:** "error[E0119]: conflicting implementations" or "duplicate method `on_event`".

## Code Examples

Verified patterns from codebase:

### Widget Trait (target for derive macro)
```rust
// Source: crates/textual-rs/src/widget/mod.rs
pub trait Widget: 'static {
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer);
    fn compose(&self) -> Vec<Box<dyn Widget>> { vec![] }
    fn on_mount(&self, _id: WidgetId) {}
    fn on_unmount(&self, _id: WidgetId) {}
    fn can_focus(&self) -> bool { false }
    fn widget_type_name(&self) -> &'static str;
    fn classes(&self) -> &[&str] { &[] }
    fn id(&self) -> Option<&str> { None }
    fn default_css() -> &'static str where Self: Sized { "" }
    fn on_event(&self, _event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        EventPropagation::Continue
    }
    fn key_bindings(&self) -> &[KeyBinding] { &[] }
    fn on_action(&self, _action: &str, _ctx: &AppContext) {}
}
```

### Full Boilerplate Pattern (derive macro target)
```rust
// Source: crates/textual-rs/src/widget/button.rs — canonical reference implementation
pub struct Button {
    pub label: String,
    own_id: Cell<Option<WidgetId>>,  // MUST be declared by user
}
// Key bindings as static slice (zero-allocation pattern):
static BUTTON_BINDINGS: &[KeyBinding] = &[KeyBinding { ... }];

impl Widget for Button {
    fn widget_type_name(&self) -> &'static str { "Button" }
    fn can_focus(&self) -> bool { true }
    fn on_mount(&self, id: WidgetId) { self.own_id.set(Some(id)); }
    fn on_unmount(&self, _id: WidgetId) { self.own_id.set(None); }
    fn key_bindings(&self) -> &[KeyBinding] { BUTTON_BINDINGS }
    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "press" {
            if let Some(id) = self.own_id.get() { ctx.post_message(id, messages::Pressed); }
        }
    }
}
```

### AppContext post_message (existing primitive for Worker + notify)
```rust
// Source: crates/textual-rs/src/widget/context.rs
pub fn post_message(&self, source: WidgetId, message: impl Any + 'static) {
    self.message_queue.borrow_mut().push((source, Box::new(message)));
}
```

### push_screen_deferred (Command Palette overlay pattern)
```rust
// Source: crates/textual-rs/src/widget/context.rs
pub fn push_screen_deferred(&self, screen: Box<dyn Widget>) {
    self.pending_screen_pushes.borrow_mut().push(screen);
}
// Called from on_action(&self) without &mut — same pattern for CommandPalette trigger
```

### Existing AppEvent (add WorkerResult variant here)
```rust
// Source: crates/textual-rs/src/event/mod.rs (inferred from app.rs)
// Currently: Key, Mouse, Resize, RenderRequest
// Add: WorkerResult { source_id: WidgetId, payload: Box<dyn Any + Send> }
```

### lazeport.pwn.zone Color Palette (for demos)
```rust
// Source: reference_lazeport.md memory
// ANSI approximations for terminal rendering:
const BG_VOID: ratatui::style::Color = Color::Rgb(6, 6, 11);      // #06060b
const BG_SURFACE: ratatui::style::Color = Color::Rgb(15, 15, 26); // #0f0f1a
const ACCENT: ratatui::style::Color = Color::Rgb(0, 255, 163);    // #00ffa3
const CYAN: ratatui::style::Color = Color::Rgb(0, 212, 255);      // #00d4ff
const TEXT_PRIMARY: ratatui::style::Color = Color::Rgb(232, 232, 240); // #e8e8f0
const TEXT_MUTED: ratatui::style::Color = Color::Rgb(119, 119, 170);   // #7777aa
const RED: ratatui::style::Color = Color::Rgb(255, 51, 102);       // #ff3366
const AMBER: ratatui::style::Color = Color::Rgb(255, 170, 0);     // #ffaa00
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| syn 1.x (parse_quote) | syn 2.x (DeriveInput, full feature) | syn 2.0 released 2023 | Different API — use syn 2.x docs, not old tutorials |
| Manual derive via old `procedural-masquerade` | Native `proc_macro` + syn/quote | Rust stable 2018 | No workarounds needed |
| `proc_macro_hack` for function-like macros in expression position | Direct `proc_macro` support | Rust 1.45 | Not needed in this project |

**Deprecated/outdated:**
- syn 1.x: Most blog posts and tutorials before 2023 use syn 1.x API. The `parse_macro_input!`, `DeriveInput`, `quote!` macro are stable across versions, but attribute parsing changed. Use syn 2.x docs at docs.rs/syn/2.
- `proc_macro_attribute` on individual methods for event dispatch: This pattern (each method generates its own impl) does not compose. Use attribute on impl block instead.

## Open Questions

1. **AppEvent::WorkerResult variant — is `Box<dyn Any + Send>` compatible with current event loop?**
   - What we know: `AppEvent` is used with flume channel (unbounded). `flume` requires `T: Send` for `Sender<T>`. `Box<dyn Any + Send>` satisfies this.
   - What's unclear: Does adding a new variant require updating exhaustive match arms in `run_async` and `process_event` in `TestApp`? Yes — both have `match event { ... Ok(_) => {} }` catch-all which will silently ignore the new variant unless explicitly handled. Need to add a handler arm.
   - Recommendation: Add `AppEvent::WorkerResult` arm explicitly in `run_async` to post to message queue. Update `process_event` in `TestApp` similarly.

2. **`#[on]` attribute scope — impl block vs method level?**
   - What we know: Method-level generates multiple impls (compile error). Impl-block level requires rewriting the whole impl block (invasive but correct).
   - What's unclear: Whether the ergonomics feel natural to users (applying `#[on]` to the impl block is unusual).
   - Recommendation: Use `#[on]` as a method-level *inert* attribute recognized by `#[derive(Widget)]` — the derive operates on the struct but cannot see method bodies. **Best final answer:** Keep it simple: `#[on(T)]` is a method attribute that gets picked up by a wrapping `#[widget_impl]` proc-macro attribute applied to the entire `impl MyWidget { }` block. Two attributes total per widget: `#[derive(Widget)]` on struct + `#[widget_impl]` on impl.

3. **Tutorial count (3-6 examples) — what is the right progression?**
   - What we know: Textual's docs go: Install → First App → Widgets → CSS → Events → Workers
   - Recommendation (5 tutorials): `01_hello` (App + one widget), `02_layout` (compose + CSS), `03_events` (key bindings + message handling), `04_reactive` (Reactive + watch_), `05_workers` (Worker API). Each adds one new concept. Command palette shown in `demo.rs`, not tutorials.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All compilation | Yes | 1.94.0 | — |
| cargo | Build + test | Yes | bundled with rustc | — |
| syn 2.x | textual-rs-macros | Not yet in Cargo.toml | — | Must be added to macros crate |
| quote 1.x | textual-rs-macros | Not yet in Cargo.toml | — | Must be added to macros crate |
| proc-macro2 | textual-rs-macros | Not yet in Cargo.toml | — | Must be added to macros crate |
| strsim | Command palette fuzzy search | Not in Cargo.toml | — | Could use substring match instead; strsim is lightweight |
| tokio::task::AbortHandle | Worker cancellation | Yes — tokio already a dep | tokio 1.x | — |

**Missing dependencies with no fallback:**
- `syn`, `quote`, `proc-macro2` — must be added to `textual-rs-macros/Cargo.toml` (not the main crate).

**Missing dependencies with fallback:**
- `strsim` — simple substring/prefix match is sufficient MVP for command palette if strsim is not desired.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + insta 1.46.3 for snapshots |
| Config file | none (workspace-level `rust-version = "1.88"` in Cargo.toml) |
| Quick run command | `cargo test -p textual-rs -- --test-output immediate 2>&1` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DX-01 | `#[derive(Widget)]` compiles, generates correct impl | unit (compile test + runtime) | `cargo test -p textual-rs derive_widget` | ❌ Wave 0 |
| DX-01 | `#[focusable]` attribute sets `can_focus() = true` | unit | `cargo test -p textual-rs derive_widget_focusable` | ❌ Wave 0 |
| DX-01 | `#[on(T)]` generates correct on_event dispatch | unit | `cargo test -p textual-rs on_event_dispatch` | ❌ Wave 0 |
| DX-01 | `#[keybinding]` generates correct key_bindings + on_action | unit | `cargo test -p textual-rs keybinding_dispatch` | ❌ Wave 0 |
| DX-02 | `ctx.run_worker(id, fut)` delivers WorkerResult<T> via message queue | unit (tokio) | `cargo test -p textual-rs worker_result_delivered` | ❌ Wave 0 |
| DX-02 | Worker aborted on widget unmount | unit | `cargo test -p textual-rs worker_cancelled_on_unmount` | ❌ Wave 0 |
| DX-03 | `notify(msg)` posts message that bubbles to parent | unit | `cargo test -p textual-rs notify_bubbles` | ❌ Wave 0 |
| DX-03 | `ctx.post_message(target_id, msg)` dispatches to target | unit (existing mechanism) | `cargo test -p textual-rs post_message_target` | ❌ Wave 0 |
| DX-04 | CommandPalette opens as overlay on Ctrl+P | integration (TestApp) | `cargo test -p textual-rs command_palette_opens` | ❌ Wave 0 |
| DX-04 | Fuzzy search filters command list | unit | `cargo test -p textual-rs command_palette_fuzzy_search` | ❌ Wave 0 |
| DX-04 | Executing a command dispatches the correct action | integration | `cargo test -p textual-rs command_palette_dispatch` | ❌ Wave 0 |
| DX-05 | `cargo doc --no-deps` completes without warnings | smoke | `cargo doc --no-deps 2>&1 \| grep -E "warning|error"` | N/A (cargo run) |
| DX-05 | `cargo run --example tutorial_01_hello` compiles | compile smoke | `cargo build --example tutorial_01_hello` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p textual-rs 2>&1`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/textual-rs/tests/macro_tests.rs` — derive Widget, on, keybinding compile tests (DX-01)
- [ ] `crates/textual-rs/tests/worker_tests.rs` — Worker API async tests with tokio test runtime (DX-02)
- [ ] `crates/textual-rs/tests/command_palette_tests.rs` — palette open/search/dispatch integration (DX-04)
- [ ] `crates/textual-rs-macros/` — entire new crate directory (DX-01 prerequisite)

## Sources

### Primary (HIGH confidence)
- Codebase direct read: `crates/textual-rs/src/widget/mod.rs` — Widget trait exact method signatures
- Codebase direct read: `crates/textual-rs/src/widget/button.rs` — canonical boilerplate pattern
- Codebase direct read: `crates/textual-rs/src/widget/context.rs` — AppContext fields and post_message
- Codebase direct read: `crates/textual-rs/src/app.rs` — run_async event loop, drain_message_queue
- Codebase direct read: `crates/textual-rs/src/event/dispatch.rs` — dispatch_message bubbling
- `cargo search syn` 2026-03-25 — syn 2.0.117 confirmed current
- `cargo search quote` 2026-03-25 — quote 1.0.45 confirmed current
- `cargo search proc-macro2` 2026-03-25 — proc-macro2 1.0.106 confirmed current
- `cargo search strsim` 2026-03-25 — strsim 0.11.1 confirmed current

### Secondary (MEDIUM confidence)
- `cargo search tokio-util` — tokio-util 0.7.18 available but AbortHandle already in tokio 1.x
- Project memory `reference_lazeport.md` — lazeport palette values previously extracted and verified

### Tertiary (LOW confidence)
- Proc-macro `#[on]` attribute-on-impl-block pattern: based on knowledge of syn/quote ecosystem; specific API shapes are training-data-informed but patterns are well-established in syn docs

## Metadata

**Confidence breakdown:**
- Standard stack (syn/quote/proc-macro2): HIGH — verified via cargo search + known stable ecosystem
- Widget trait boilerplate pattern: HIGH — read directly from codebase
- AppContext primitives available: HIGH — read directly from context.rs and app.rs
- Worker API design: MEDIUM — design pattern is sound but exact AppEvent variant addition needs careful implementation
- `#[on]` attribute scope strategy: MEDIUM — multiple valid approaches; impl-block attribute recommended but users may find it unusual
- Command palette overlay: HIGH — push_screen_deferred pattern verified in select.rs

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable Rust ecosystem, no fast-moving dependencies)
