# Async Runtimes & Reactive State Patterns in Rust

**Domain:** TUI framework (Textual-inspired, Rust)
**Researched:** 2026-03-24
**Overall confidence:** MEDIUM-HIGH

---

## 1. Async Runtime: Tokio vs async-std vs smol

### Decision

**Use Tokio.** Confidence: HIGH.

async-std was officially discontinued on March 1, 2025. Its suggested replacement is
smol. Between Tokio and smol, Tokio is the correct choice for a TUI framework for
one practical reason: the crossterm `EventStream` (the canonical way to read keyboard
and mouse events as an async stream) is tested against and documented primarily for
Tokio. The entire ratatui ecosystem runs on Tokio.

### Comparison

| Concern | Tokio | smol |
|---------|-------|------|
| Ecosystem fit | Excellent — crossterm, ratatui, reqwest, tracing all Tokio-native | Good, but fewer crates target smol directly |
| Binary size | Larger (~500KB overhead) | Smaller (~50KB) |
| Complexity | Higher — multi-thread, Send + 'static pressure | Lower — single-thread friendly |
| TUI suitability | Strong — `tokio::select!`, `tokio::sync::watch`, `EventStream` | Viable but requires adapters |
| Status | Actively maintained, dominant | Active (smol-rs org), smaller community |
| async-std migration | n/a | Direct replacement for async-std |

### The Send + 'static Problem

Tokio's multi-threaded scheduler requires spawned tasks to be `Send + 'static`. For a
widget tree where widgets hold `Rc<RefCell<T>>` (non-Send), this forces a design choice:

- **Option A:** Run the entire TUI event loop on a `LocalSet` (single-threaded Tokio
  executor). This lifts the `Send` requirement. All widget state can use `Rc<RefCell<T>>`.
  Background work is spawned via `tokio::task::spawn_blocking` or a separate
  multi-threaded pool and communicates back via channels.

- **Option B:** Use `Arc<Mutex<T>>` throughout for all shared state. This is safe but
  noisy. Unnecessary for a fundamentally single-threaded UI loop.

**Recommendation:** Use `tokio::task::LocalSet` for the main render/event loop, and
`tokio::spawn` (thread-pool) only for background I/O tasks that communicate results back
via `tokio::sync::mpsc`.

```rust
#[tokio::main]
async fn main() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async move {
        // All widget/screen/app logic here — Rc<RefCell<T>> is safe
        run_app().await;
    }).await;
}
```

### Sources
- https://corrode.dev/blog/async/ (HIGH confidence — current, authored 2025)
- https://weeklyrust.substack.com/p/goodbye-async-std-welcome-smol (HIGH confidence)

---

## 2. Event Loop Architecture

### The Standard Pattern

Modern Rust TUI event loops decouple I/O polling from rendering via a channel bridge.
The crossterm `EventStream` (enabled with `features = ["event-stream"]`) implements
`futures::Stream`, making it composable with `tokio::select!`.

```rust
use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

#[derive(Debug)]
enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Tick,
    Resize(u16, u16),
}

async fn event_task(tx: mpsc::UnboundedSender<AppEvent>, tick_rate: Duration) {
    let mut reader = EventStream::new();
    let mut ticker = interval(tick_rate);

    loop {
        let next = reader.next().fuse();
        tokio::select! {
            maybe_event = next => {
                match maybe_event {
                    Some(Ok(crossterm::event::Event::Key(k))) => {
                        let _ = tx.send(AppEvent::Key(k));
                    }
                    Some(Ok(crossterm::event::Event::Mouse(m))) => {
                        let _ = tx.send(AppEvent::Mouse(m));
                    }
                    Some(Ok(crossterm::event::Event::Resize(w, h))) => {
                        let _ = tx.send(AppEvent::Resize(w, h));
                    }
                    _ => {}
                }
            },
            _ = ticker.tick() => {
                let _ = tx.send(AppEvent::Tick);
            }
        }
    }
}
```

The main loop then becomes:

```rust
async fn run_app(mut rx: mpsc::UnboundedReceiver<AppEvent>) -> Result<()> {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(|f| view(f, &state))?;
        match rx.recv().await {
            Some(AppEvent::Key(k)) => {
                if update(&mut state, k) == Action::Quit { break; }
            }
            Some(AppEvent::Tick) => { tick(&mut state); }
            _ => {}
        }
    }
    ratatui::restore();
    Ok(())
}
```

### The Three Architectural Styles (and which to pick)

| Style | How it works | Best for |
|-------|-------------|----------|
| **Immediate mode** (ratatui default) | Redraw entire UI every event/tick | Simple apps, no retained state |
| **Elm / TEA** (tui-realm) | `update(state, msg) -> state` pure function | Testable apps, functional style |
| **Component + actor** (r3bl_tui) | Each component has mailbox, tokio::select loop | Complex apps, parallel background work |

For a Textual-inspired framework, **Elm-style with component dispatch** is the right
model: messages are typed enums, each widget has an `on_message` handler, and a central
message pump routes events to focused/targeted components.

### Key Gotcha: `tokio::select!` Pattern Matching

If a branch in `select!` uses a destructuring pattern that doesn't match, that branch
becomes disabled for the remainder of the `select!` block. Always use `if let` inside
the handler, not in the pattern:

```rust
// WRONG — branch gets disabled if pattern doesn't match
tokio::select! {
    Some(AppEvent::Key(k)) = rx.recv() => { ... }
}

// CORRECT
tokio::select! {
    msg = rx.recv() => {
        if let Some(AppEvent::Key(k)) = msg { ... }
    }
}
```

### Sources
- https://ratatui.rs/tutorials/counter-async-app/async-event-stream/ (HIGH)
- https://github.com/ratatui/ratatui/discussions/220 (MEDIUM — community discussion)

---

## 3. Reactive State: Signal-like Reactivity in Rust

### The Options

#### A. `reactive_graph` (from the Leptos org)

**Recommendation for reactive primitives.** Confidence: MEDIUM.

`reactive_graph` extracts Leptos's fine-grained reactive system into a standalone crate.
It is runtime-agnostic via `any_spawner`, meaning effects are scheduled through whatever
async executor you provide (Tokio works).

Key types:
- `RwSignal<T>` — combined read/write reactive cell
- `ReadSignal<T>` / `WriteSignal<T>` — split read/write handles
- `ArcRwSignal<T>` — `Arc`-backed version for sharing across threads
- `Memo<T>` — computed/derived value, auto-tracks dependencies
- `Effect` — side effect that re-runs when accessed signals change
- `Trigger` — data-less notification signal

```rust
use reactive_graph::signal::RwSignal;
use reactive_graph::effect::Effect;
use reactive_graph::traits::{Get, Set};

let count = RwSignal::new(0i32);
let doubled = Memo::new(move |_| count.get() * 2);

// Effect runs once immediately and whenever `count` changes
Effect::new(move |_| {
    println!("count is: {}, doubled: {}", count.get(), doubled.get());
});

count.set(5); // triggers the effect on next async tick
```

Effects are **asynchronously scheduled**: the signal update is immediate, but effects
run on the next tick of the async runtime. This matches how CSS/DOM reactivity works
and is the right behavior for triggering re-renders after state changes.

#### B. `futures-signals` (Pauan/rust-signals)

Zero-cost FRP signals built on the `futures` crate. The core type is `Mutable<T>`:

```rust
use futures_signals::signal::{Mutable, SignalExt};

let value = Mutable::new(0i32);

// Get a read handle that produces a Stream of changes
let signal = value.signal();

// React to changes as a future
signal.for_each(|new_val| {
    println!("Value changed to: {}", new_val);
    async {}
}).await;

// Update
value.set(42); // subscribers notified
```

`map_ref!` macro combines multiple signals:
```rust
use futures_signals::signal::map_ref;
let combined = map_ref!(count, name => format!("{}: {}", count, name));
```

Best for: **data pipelines and derived state**, especially when you want push-based
async updates. Less ergonomic than `reactive_graph` for fine-grained DOM-style reactivity
but integrates naturally with `Stream`-based code.

#### C. `tokio::sync::watch`

Built into Tokio. A single-producer, multi-consumer channel that retains only the most
recent value. Receivers are notified when the value changes.

```rust
use tokio::sync::watch;

let (tx, mut rx) = watch::channel(AppState::default());

// Background task watches for state changes
tokio::spawn(async move {
    loop {
        // Process current value, then wait for next change
        {
            let state = rx.borrow_and_update();
            render(&state);
        }
        if rx.changed().await.is_err() { break; }
    }
});

// Elsewhere, update state
tx.send(new_state).unwrap();
// Or modify in-place (notifies only if changed):
tx.send_if_modified(|s| { s.counter += 1; true });
```

`watch` is excellent for **broadcasting state to multiple consumers** (e.g., multiple
widgets watching the same config or application state). Not suitable for fine-grained
per-property reactivity — it notifies on any change to the watched value, not individual
fields.

#### D. `rxRust` (RxRust)

Reactive Extensions port for Rust. Supports both thread-safe (`Arc<Mutex>`) and
single-threaded (`Rc<RefCell>`) contexts, and is Tokio-compatible. Most powerful for
event stream composition (filter, map, merge, debounce) but has a steeper learning curve
and adds significant dependency weight.

Suitable for: complex event stream transformations, not basic state reactivity.

### Recommendation

For a Textual-inspired framework:
- **Primary reactive primitive:** `reactive_graph` signals for per-property change
  tracking. Signals live on widgets, effects trigger re-renders.
- **State broadcast:** `tokio::sync::watch` for broadcasting global state (theme,
  focused widget ID, screen stack) to the render loop.
- **Event streams:** `futures-signals` `SignalVec` if you need reactive lists (e.g.,
  a scrollable list whose items change reactively).

### Sources
- https://docs.rs/reactive_graph/latest/reactive_graph/index.html (HIGH — official docs)
- https://docs.rs/futures-signals/latest/futures_signals/ (HIGH — official docs)
- https://docs.rs/tokio/latest/tokio/sync/watch/index.html (HIGH — official docs)
- https://book.leptos.dev/appendix_reactive_graph.html (MEDIUM — explanatory docs)

---

## 4. Message Passing Channels

### Channel Comparison

| Channel | Sync? | Async? | MPMC? | Notes |
|---------|-------|--------|-------|-------|
| `std::sync::mpsc` | Yes | No | No (MPSC only) | Baseline, no async support |
| `tokio::sync::mpsc` | No | Yes | No (MPSC only) | Standard choice for Tokio apps |
| `tokio::sync::broadcast` | No | Yes | Yes (fan-out) | All receivers get all messages |
| `tokio::sync::watch` | No | Yes | No (last-value) | Reactive state broadcast |
| `flume` | Yes | Yes | Yes | Sync+async unified, fast, ergonomic |
| `kanal` | Yes | Yes | Yes | Fastest in benchmarks, unified API |

### Recommendation: `flume` for the event bus

`flume` provides a unified sync/async API and is the ergonomic middle ground. It is
faster than `std::sync::mpsc` and allows bridging sync and async code without recreating
channels. For a TUI framework that may need to receive events from both sync (background
thread keyboard handler) and async (network, filesystem) code, `flume` simplifies the
boundary.

```rust
use flume;

let (tx, rx) = flume::unbounded::<AppMessage>();

// Sync sender (from a thread)
tx.send(AppMessage::Quit).unwrap();

// Async receiver (in event loop)
while let Ok(msg) = rx.recv_async().await {
    dispatch(msg).await;
}
```

`kanal` is slightly faster in microbenchmarks but is less battle-tested. Use it if
performance profiling shows `flume` as a bottleneck (unlikely for TUI work).

For **background task results**, use `tokio::sync::mpsc` (bounded, with backpressure).
For **state broadcasting** (one writer, many readers, latest value), use
`tokio::sync::watch`.

### Sources
- https://github.com/fereidani/kanal (MEDIUM)
- https://docs.rs/flume (HIGH — official docs)
- https://tokio.rs/tokio/tutorial/channels (HIGH — official)

---

## 5. Textual MessagePump / Message Dispatch in Rust

Textual's `MessagePump` is effectively an actor: each widget has an async message queue,
receives messages in a loop, and dispatches them to typed handler methods. This maps
directly to the **Tokio actor pattern**.

### Core Pattern

```rust
// 1. Define your message enum
#[derive(Debug)]
enum WidgetMessage {
    Focus,
    Blur,
    KeyPress(crossterm::event::KeyEvent),
    Mount,
    Unmount,
    // User-defined messages
    Custom(Box<dyn std::any::Any + Send>),
}

// 2. The widget actor
struct Widget {
    id: WidgetId,
    receiver: tokio::sync::mpsc::Receiver<WidgetMessage>,
    // ... state
}

impl Widget {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle(msg).await;
        }
    }

    async fn handle(&mut self, msg: WidgetMessage) {
        match msg {
            WidgetMessage::Focus     => self.on_focus().await,
            WidgetMessage::Blur      => self.on_blur().await,
            WidgetMessage::KeyPress(k) => self.on_key(k).await,
            WidgetMessage::Mount     => self.on_mount().await,
            WidgetMessage::Unmount   => self.on_unmount().await,
            WidgetMessage::Custom(m) => self.on_message(m).await,
        }
    }
}
```

### Routing: From MessagePump to Widget

Textual routes messages up/down the widget tree. In Rust, the cleanest translation is:

- Each widget has a `Sender<WidgetMessage>` handle stored in the `ComponentRegistry`
  (a `SlotMap<WidgetId, Sender<WidgetMessage>>`).
- The App-level event loop holds a root `Receiver<AppMessage>`.
- When an event arrives, the loop looks up the focused widget's sender and delivers
  the message.
- Widgets can "post" to the app or to named components via the registry.

```rust
struct App {
    components: slotmap::SlotMap<WidgetId, ComponentHandle>,
    focused: WidgetId,
    app_tx: flume::Sender<AppMessage>,
    app_rx: flume::Receiver<AppMessage>,
}

struct ComponentHandle {
    tx: tokio::sync::mpsc::Sender<WidgetMessage>,
    task: tokio::task::JoinHandle<()>,
}

impl App {
    async fn dispatch_to_focused(&self, msg: WidgetMessage) {
        if let Some(handle) = self.components.get(self.focused) {
            let _ = handle.tx.send(msg).await;
        }
    }
}
```

### Bubble vs. Tunnel

Textual bubbles unhandled messages up the DOM. In Rust, implement this by having widgets
return an `EventPropagation` enum:

```rust
enum EventPropagation {
    Consumed,           // widget handled it, stop
    Propagate(WidgetMessage), // pass to parent
}
```

The registry walks the parent chain (stored as a `WidgetId` in a secondary map) until
the message is consumed or reaches the root App.

### Sources
- https://ryhl.io/blog/actors-with-tokio/ (HIGH — authoritative Tokio blog post)
- https://deepwiki.com/r3bl-org/r3bl-open-core/2-tui-framework-(r3bl_tui) (MEDIUM)

---

## 6. Watch-for-Property-Change → Trigger Re-render

Three viable patterns, from simplest to most powerful:

### Pattern A: Dirty Flag + Render on Tick

The simplest possible approach. Each widget tracks a `dirty: bool`. On each tick, the
event loop calls `render()` only on dirty widgets. State mutation sets `dirty = true`.

```rust
struct MyWidget {
    value: i32,
    dirty: bool,
}

impl MyWidget {
    fn set_value(&mut self, v: i32) {
        if self.value != v {
            self.value = v;
            self.dirty = true;
        }
    }
}
```

Good for: simple cases. No dependency tracking overhead. Works with any architecture.

### Pattern B: `tokio::sync::watch` per Property

Create a `watch` channel for each reactive property. The render loop selects across all
watched channels.

```rust
let (title_tx, mut title_rx) = watch::channel(String::from("Hello"));

// Render task
tokio::task::spawn_local(async move {
    loop {
        let title = title_rx.borrow_and_update().clone();
        render_title(&title);
        if title_rx.changed().await.is_err() { break; }
    }
});

// Elsewhere: update triggers re-render
title_tx.send("World".to_string()).unwrap();
```

Good for: a small number of high-level properties (screen title, loading state, etc.).
Scales poorly to dozens of per-widget reactive properties.

### Pattern C: `reactive_graph` Signals + Effects

The most expressive approach. Each reactive property is an `RwSignal`. Effects
automatically subscribe to the signals they read. When a signal changes, all dependent
effects are scheduled to re-run on the next async tick.

```rust
use reactive_graph::{signal::RwSignal, effect::Effect, traits::{Get, Set}};

// Widget state
let loading = RwSignal::new(false);
let items: RwSignal<Vec<String>> = RwSignal::new(vec![]);

// This effect re-runs whenever `loading` or `items` changes
let render_tx = render_channel.clone();
Effect::new(move |_| {
    let is_loading = loading.get();
    let data = items.get();
    // Send a re-render request to the main event loop
    let _ = render_tx.send(RenderRequest::Widget(widget_id, is_loading, data));
});

// Trigger the re-render chain
loading.set(true); // effect fires on next tick
```

The key architectural insight: effects should **post a message** (send on a channel) to
the render loop rather than drawing directly, keeping the reactive graph and the terminal
I/O on the same thread but decoupled.

### Recommendation

For a Textual-equivalent framework: **Pattern C for widget properties** + **Pattern A
dirty flag** as the render gate. Signals track what changed, the dirty flag prevents
redundant draws within the same tick.

---

## 7. App > Screen > Widget Ownership Hierarchy

This is the hardest problem in Rust UI. The core tension: widgets need to reference
each other (for focus, for layout, for event bubbling), but Rust's borrow checker
forbids cycles.

### The Three Strategies

#### Strategy 1: Arena + Index (Recommended)

Store all widgets in a `SlotMap`. Widgets hold keys (`WidgetId = slotmap::DefaultKey`),
not references. The arena owns all widgets; no reference cycles possible.

```rust
use slotmap::{SlotMap, DefaultKey};

type WidgetId = DefaultKey;

struct WidgetArena {
    widgets: SlotMap<WidgetId, Box<dyn Widget>>,
    parent: slotmap::SecondaryMap<WidgetId, WidgetId>,
    children: slotmap::SecondaryMap<WidgetId, Vec<WidgetId>>,
    focus: Option<WidgetId>,
}

impl WidgetArena {
    fn add_child(&mut self, parent: WidgetId, child: WidgetId) {
        self.parent.insert(child, parent);
        self.children.entry(parent).unwrap().or_default().push(child);
    }

    fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        self.widgets.get_mut(id).map(|w| w.as_mut())
    }
}
```

`SecondaryMap` stores additional per-widget data without touching the widget struct,
making it ideal for layout results, dirty flags, z-order, etc.

**Gotcha:** You cannot hold a `&mut dyn Widget` and also mutate the arena. Resolve this
by extracting the widget, operating on it, then returning it:

```rust
// Pattern: remove, operate, reinsert
if let Some(mut widget) = arena.widgets.remove(id) {
    widget.handle_event(&mut arena, event); // arena accessible, widget is moved out
    arena.widgets.insert(id, widget);       // reinsert with same key (use retain_key)
}
// Note: SlotMap does not support reinserting with the same key directly.
// Use slotmap::HopSlotMap or store in Box and use std::mem::replace instead.
```

A cleaner solution: pass an `&mut WidgetArena` to `handle_event` and `render`, and
look up child widgets by ID inside those methods, rather than holding borrows.

#### Strategy 2: `Rc<RefCell<dyn Widget>>`

Each widget is wrapped in `Rc<RefCell<>>`. Children store `Rc` clones; parent pointers
use `Weak<RefCell<>>` to avoid cycles.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct WidgetNode {
    inner: Box<dyn Widget>,
    parent: Option<Weak<RefCell<WidgetNode>>>,
    children: Vec<Rc<RefCell<WidgetNode>>>,
}
```

Pro: Familiar DOM-like tree. Cons: Runtime borrow panics if you borrow a node while
holding a borrow to a parent. Not `Send`. The borrow chain `app -> screen -> widget`
means three nested `.borrow_mut()` calls, which is both verbose and panic-prone.

Telex uses this pattern successfully for a single-threaded TUI, but their post-mortem
notes that `RefCell` panics at runtime if you forget to release a borrow before
recursing into the tree.

**Avoid for deep trees.** Use only for 2-level hierarchies (e.g., App has Screens, but
Screens manage their widget layout without cross-references).

#### Strategy 3: Flat Index Map (ECS-lite)

Store widgets as `HashMap<WidgetId, WidgetState>`. No tree structure in memory —
tree structure is computed from parent/child ID maps. Rendering walks the map in layout
order.

This is what GPUI (Zed's UI framework) and Bevy UI do. It is the most Rust-idiomatic
approach but requires a separate layout pass that computes positions from the tree
structure.

```rust
struct App {
    widgets: HashMap<WidgetId, WidgetState>,
    layout_tree: Vec<WidgetId>, // flat depth-first order
    parent_map: HashMap<WidgetId, WidgetId>,
    children_map: HashMap<WidgetId, Vec<WidgetId>>,
}
```

#### Recommendation

Use **Strategy 1 (Arena + SlotMap)** for widgets, **Strategy 2 (`Rc<RefCell<>>`)** only
for the top two levels (App owns `Vec<Rc<RefCell<Screen>>>`), and flat ID maps for
sibling relationships within a screen.

```
App (owns)
  └── Vec<Box<Screen>> indexed by SlotMap
        └── WidgetArena (SlotMap<WidgetId, Box<dyn Widget>>)
              └── SecondaryMap for children, parents, dirty flags, layout cache
```

The App-level `Screen` list can use `Box<Screen>` (App owns exclusively) since screen
transitions are simple push/pop. Within a screen, the SlotMap arena handles complexity.

### Trait Object Considerations

`Box<dyn Widget>` requires widget methods to take `&mut self`, not `self`. This is fine
for the `render` and `handle_event` methods. For a heterogeneous widget tree, define
the trait with object-safe methods only (no generics in method signatures, no `Self`
return types):

```rust
pub trait Widget {
    fn widget_id(&self) -> WidgetId;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_event(&mut self, event: &AppEvent) -> EventPropagation;
    fn on_mount(&mut self, ctx: &mut AppContext);
    fn on_unmount(&mut self);
    // Layout
    fn layout_constraints(&self) -> Constraints;
    fn set_layout(&mut self, rect: Rect);
}
```

### Sources
- https://docs.rs/slotmap/latest/slotmap/ (HIGH — official docs)
- https://telex-tui.github.io/blog/designing-a-tui-framework-in-rust.html (MEDIUM)
- https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html (HIGH — Raph Levien, Xilem author)
- https://linebender.org/blog/xilem-backend-roadmap/ (MEDIUM — 2024)

---

## 8. Synthesized Architecture for textual-rs

Combining all findings, the recommended architecture is:

```
┌─────────────────────────────────────────────────────────┐
│  tokio::main → LocalSet (single-threaded event loop)    │
│                                                         │
│  ┌──────────────┐    flume channel    ┌───────────────┐ │
│  │ crossterm    │ ──AppEvent──────►  │   App Loop    │ │
│  │ EventStream  │                    │               │ │
│  └──────────────┘                    │  match event  │ │
│                                      │  → dispatch   │ │
│  ┌──────────────┐    tokio::spawn    │  → render     │ │
│  │ Background   │ ──Result──────►   └───────┬───────┘ │
│  │ Tasks (pool) │  (mpsc channel)            │         │
│  └──────────────┘                            │         │
│                                      ┌───────▼───────┐ │
│                                      │  WidgetArena  │ │
│                                      │  (SlotMap)    │ │
│                                      │               │ │
│                                      │ RwSignal<T>   │ │
│                                      │ per property  │ │
│                                      │               │ │
│                                      │ Effect → post │ │
│                                      │ RenderRequest │ │
│                                      └───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Recommended Crates

| Role | Crate | Version | Rationale |
|------|-------|---------|-----------|
| Async runtime | `tokio` | 1.x | Ecosystem standard; crossterm EventStream requires it |
| Terminal events | `crossterm` | 0.28+ | `EventStream` feature; `event-stream` flag |
| TUI rendering | `ratatui` | 0.30+ | 2100+ downstream crates; immediate mode renders well |
| Event channel | `flume` | 0.11+ | Unified sync+async API, bridges thread boundaries |
| Reactive signals | `reactive_graph` | 0.1+ | Fine-grained reactivity, Tokio-compatible via any_spawner |
| Reactive lists | `futures-signals` | 0.3+ | `MutableVec` for reactive lists of widgets |
| State broadcast | `tokio::sync::watch` | built-in | Config/theme/screen-stack broadcast to many consumers |
| Widget storage | `slotmap` | 1.x | O(1) lookup, generational indices prevent use-after-free |
| Trait objects | std `Box<dyn Widget>` | — | No external dep needed for basic dynamic dispatch |

---

## 9. Critical Gotchas

### 9.1 Borrow Checker and Widget Mutation

You cannot hold `&mut Widget` from the arena and also pass `&mut Arena` to the widget's
method. The workaround is to either:
1. Use `std::mem::take` / clone the widget out, call the method, put it back, or
2. Design `handle_event` to receive a context object (`AppContext`) that does NOT
   contain a borrow to the current widget, only to peers.

### 9.2 `Rc<RefCell<T>>` Runtime Panics

Any pattern that borrows a `RefCell` and then calls into code that also borrows the
same cell will panic. In a widget tree this happens when: a parent widget iterates
its children, calling `borrow_mut()` on each, and a child handler tries to access the
parent. Mitigation: arena pattern avoids this entirely.

### 9.3 `async fn` in Trait Objects

`async fn` in traits is not object-safe in stable Rust without `async_trait` macro.
Widget trait methods that need to be `async` must either:
- Use `async_trait` crate (boxes the future, adds allocation per call)
- Return `Pin<Box<dyn Future<Output = ...> + '_>>` manually
- Or: avoid `async` in the Widget trait, dispatching to channels instead

For the Textual model, the recommended approach is: widget handlers are **sync** and
communicate asynchronously by posting to the App's channel. Only background tasks use
`async fn`. This keeps the widget trait object-safe without `async_trait`.

### 9.4 `tokio::select!` and Cancellation Safety

Not all futures are cancellation-safe. If you cancel a `recv().await` in mid-flight
(by selecting a different branch), the message may be lost. Design the event loop to
avoid cancelling a receive that has already selected — use `biased;` mode for
deterministic priority or use `tokio_util::sync::CancellationToken` for graceful
shutdown.

### 9.5 `reactive_graph` Requires a Runtime Executor Registration

Before using `Effect`, you must initialize `any_spawner` with the current runtime:

```rust
// At startup, register Tokio as the executor for reactive_graph
reactive_graph::executor::Executor::init_tokio().expect("already init");
```

Without this, effects will not schedule and no reactive updates will fire.

### Sources
- https://ryhl.io/blog/actors-with-tokio/ (HIGH)
- https://telex-tui.github.io/blog/designing-a-tui-framework-in-rust.html (MEDIUM)
- https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html (HIGH)

---

## 10. Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Async runtime choice (Tokio) | HIGH | async-std gone, ecosystem consensus |
| Event loop pattern | HIGH | Well-documented in ratatui ecosystem |
| `reactive_graph` signals | MEDIUM | Docs incomplete; best available option |
| `futures-signals` API | MEDIUM | 16% documented; pattern is sound |
| `tokio::sync::watch` | HIGH | Official Tokio docs, stable API |
| Channel comparison | MEDIUM | Kanal benchmarks recent; flume battle-tested |
| Arena/SlotMap ownership | HIGH | Multiple frameworks use this pattern |
| `Rc<RefCell>` gotchas | HIGH | Well-documented in Rust book and community |
| Actor/mailbox mapping | HIGH | ryhl.io is authoritative source |
| `async fn` in traits | HIGH | Stable Rust limitation, well-known |

---

## 11. Gaps to Investigate Further

1. **`reactive_graph` + Tokio integration depth** — The `any_spawner` init API needs
   verification against the current crate version. Check `reactive_graph` changelog.

2. **`slotmap` remove + reinsert with same key** — Standard `SlotMap` does not support
   this. Investigate `HopSlotMap` or alternative approach for the arena borrow problem.

3. **`futures-signals` vs `reactive_graph` for TUI** — Neither has been used in a
   published Rust TUI framework at scale. Both are LOW risk but need a proof-of-concept
   before committing.

4. **Rendering integration** — How to make `reactive_graph` effects efficiently batch
   multiple signal changes into a single re-render tick (debounce, similar to how
   React batches state updates) needs a concrete pattern.

5. **`crossterm` EventStream on Windows** — The `event-stream` feature uses
   `tokio::io::unix` on Unix but falls back to a thread on Windows. Verify behavior
   on Windows (this project's dev platform) and check for any latency implications.
