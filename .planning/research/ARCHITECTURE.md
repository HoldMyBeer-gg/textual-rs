# Architecture Research

**Domain:** Rust TUI application framework — v1.3 widget parity milestone
**Researched:** 2026-03-26
**Confidence:** HIGH (based on direct code inspection of existing implementation)

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Code                          │
│   impl Widget for MyScreen { fn compose() -> Vec<Box<dyn Wid.>  │
├─────────────────────────────────────────────────────────────────┤
│                       Screen Stack                               │
│   ctx.screen_stack: Vec<WidgetId>  (top = active screen)        │
│   push_screen / pop_screen  (deferred via RefCell)              │
├──────────────┬──────────────┬──────────────┬────────────────────┤
│  Widget Tree │ Style Tree   │ Layout Tree  │ Reactive Graph     │
│  SlotMap     │ SecondaryMap │ Taffy nodes  │ Reactive<T>        │
│  arena +     │ ComputedStyle│ -> Rect per  │ signals on widget  │
│  SecondaryMap│ per widget   │ widget       │ fields             │
│  children/   │              │              │                    │
│  parent      │              │              │                    │
├──────────────┴──────────────┴──────────────┴────────────────────┤
│                     AppContext (single-threaded)                  │
│  arena / children / parent / computed_styles / inline_styles    │
│  dirty / pseudo_classes / focused_widget / hovered_widget       │
│  screen_stack / active_overlay / message_queue                  │
│  pending_screen_pushes / pending_screen_pops                    │
│  pending_recompose / event_tx / worker_tx                       │
├─────────────────────────────────────────────────────────────────┤
│                     Event Loop (Tokio LocalSet)                  │
│  crossterm EventStream → flume channel → dispatch               │
│  Drain: message_queue / pending_screen_pushes / pops /          │
│          pending_recompose / pending_overlay_dismiss            │
├─────────────────────────────────────────────────────────────────┤
│                     ratatui + crossterm                          │
│  Terminal::draw → Buffer → crossterm backend                    │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `AppContext` | Shared mutable world state passed everywhere | Struct with `DenseSlotMap` arena + multiple `SecondaryMap` |
| `Widget` trait | Contract every UI node implements | `render`, `compose`, `on_mount`, `on_unmount`, `on_event`, `on_action`, `key_bindings` |
| `tree.rs` (widget/tree.rs) | All structural mutations | `mount_widget`, `unmount_widget`, `push_screen`, `pop_screen`, `advance_focus`, `recompose_widget` |
| `active_overlay` | Single floating widget above all content | `RefCell<Option<Box<dyn Widget>>>` on AppContext; rendered last, receives focus first |
| `screen_stack` | Navigation and modal layering | `Vec<WidgetId>` on AppContext; top element is active screen for focus and rendering |
| `message_queue` | Widget-to-widget typed messaging | `RefCell<Vec<(WidgetId, Box<dyn Any>)>>`; drained by event loop after each cycle |
| CSS cascade | Style resolution | Parallel `SecondaryMap<WidgetId, ComputedStyle>`; invalidated by dirty flags |
| Taffy bridge | Layout geometry | Maps `ComputedStyle` to `taffy::Style`; produces `Rect` per widget per frame |

---

## Recommended Project Structure

```
crates/textual-rs/src/
├── widget/
│   ├── mod.rs            # Widget trait, WidgetId, EventPropagation
│   ├── tree.rs           # mount/unmount/push_screen/pop_screen/focus helpers
│   ├── context.rs        # AppContext struct — the shared world
│   ├── [widget].rs       # One file per widget type (existing pattern)
│   │
│   │   NEW WIDGETS (v1.3)
│   ├── content_switcher.rs   # Visibility-switching container
│   ├── digits.rs             # Large numeric display
│   ├── directory_tree.rs     # DirectoryTree wrapping Tree
│   ├── link.rs               # Clickable hyperlink label
│   ├── loading_indicator.rs  # Animated spinner
│   ├── masked_input.rs       # Input with mask pattern
│   ├── option_list.rs        # Selectable list (OptionList)
│   ├── pretty.rs             # Pretty-printed data display
│   ├── rich_log.rs           # Styled scrolling log
│   ├── rule.rs               # Horizontal/vertical separator
│   ├── selection_list.rs     # Multi-select list
│   ├── static_widget.rs      # Static text/markup display
│   └── toast.rs              # Transient notification overlay
├── css/                  # CSS cascade, types, theme
├── event/                # AppEvent, keybinding, message trait
├── reactive/             # Reactive<T> signals
├── canvas/               # Drawing primitives (mcgugan_box, scrollbar)
├── terminal/             # TerminalCaps, MouseCaptureStack
├── worker.rs             # WorkerResult, WorkerProgress
└── lib.rs
```

### Structure Rationale

- **One file per widget:** Keeps widget files under ~400 LOC, makes review diffs clean. Existing pattern; do not deviate.
- **`tree.rs` owns all structural mutations:** All `mount_widget`, `unmount_widget`, `push_screen`, `pop_screen`, `recompose_widget` live here. Widgets never call arena.remove() directly — they go through tree helpers that handle SecondaryMap cleanup, focus clearing, and worker cancellation atomically.
- **`context.rs` as shared world:** `AppContext` is passed as `&mut AppContext` (for structural changes) or `&AppContext` (for read-only widget callbacks). All deferred operations (screen pushes/pops, overlay dismiss, recompose) use `RefCell`/`Cell` so they are schedulable from `&self` widget callbacks without borrow conflicts.

---

## Architectural Patterns

### Pattern 1: Deferred Mutations via RefCell

**What:** Widgets receive `&AppContext` (not `&mut`) in `on_event` and `on_action`. State changes that need arena mutations are enqueued into `RefCell`-wrapped fields on `AppContext` and drained by the event loop after the callback returns.

**When to use:** Always — for screen pushes/pops, overlay dismiss, recompose requests, message posting. Never try to get `&mut AppContext` inside a widget callback.

**Trade-offs:** Slightly unintuitive but completely avoids borrow checker conflicts that arise from "I need to mutate the arena while holding a reference to a widget inside it."

**Example:**
```rust
// Widget schedules a screen push from on_action(&self, ...)
fn on_action(&self, action: &str, ctx: &AppContext) {
    if action == "open_settings" {
        ctx.push_screen_deferred(Box::new(SettingsScreen::new()));
    }
}

// Event loop drains after callback returns
for screen in ctx.pending_screen_pushes.borrow_mut().drain(..) {
    push_screen(screen, ctx);
}
```

### Pattern 2: active_overlay for Floating Widgets

**What:** A single `Option<Box<dyn Widget>>` slot on `AppContext` holds one floating widget that renders above all other content at absolute coordinates. The overlay widget controls its own position in `render()`. Focus routing goes to the overlay first when it is present.

**When to use:** Select dropdown, CommandPalette, Toast notification, any widget that must paint over the current screen without affecting layout. Do not put overlays in the widget tree — they would corrupt layout.

**Trade-offs:** Only one overlay can be active at a time. This is intentional — nested overlays are complex and not needed for the current widget set.

**Example (how Toast uses it):**
```rust
// Toast is set as the active overlay on AppContext
*ctx.active_overlay.borrow_mut() = Some(Box::new(Toast {
    message: "File saved".into(),
    severity: Severity::Information,
    expiry: Instant::now() + Duration::from_secs(3),
}));

// Toast render() positions itself at a corner using absolute coords
fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
    let w = 30u16.min(area.width);
    let h = 3u16;
    let x = area.x + area.width - w;
    let y = area.y + area.height - h;
    // draw at (x, y) absolute
}
```

### Pattern 3: Screen Stack for Navigation / Modals

**What:** `ctx.screen_stack` is a `Vec<WidgetId>`. The top element is the active screen. `push_screen` mounts a new screen widget (with `parent: None`) and appends its id. `pop_screen` unmounts the top screen (recursively removing all its children) and restores the previous screen.

**When to use:** Full-screen navigation (Settings screen, Help screen), modal dialogs that need their own focus scope.

**Trade-offs:** Pushing a screen is cheap (mount + compose_subtree). Popping is also cheap (unmount_widget handles recursive cleanup). Focus is scoped to the current top-of-stack screen because `advance_focus` and `advance_focus_backward` both root their DFS at `screen_stack.last()`.

**Focus change on push/pop:** When `push_screen` is called, focus should be explicitly set to the first focusable widget in the new screen. When `pop_screen` is called, the new top screen's previous focused widget id is gone (the popped screen's arena entries were removed). The event loop should call `advance_focus` once after a pop to seat focus on the restored screen. This requires a small addition: `push_screen` should optionally accept an initial focus hint, or the event loop should auto-advance after push.

**Example:**
```rust
// From inside a widget action
ctx.push_screen_deferred(Box::new(ConfirmDialog::new("Delete file?")));

// In event loop after draining pending_screen_pushes:
// Optionally auto-focus first focusable in new screen:
if !ctx.screen_stack.is_empty() {
    advance_focus(ctx);  // seats focus in the freshly pushed screen
}
```

### Pattern 4: ContentSwitcher via recompose_widget

**What:** `ContentSwitcher` holds an `active: Reactive<usize>` field that selects which child content to show. Its `compose()` returns only the currently active child. When `active` changes, the widget calls `ctx.request_recompose(own_id)`, which schedules `recompose_widget(id, ctx)` in the event loop. `recompose_widget` unmounts all existing children and calls `compose_subtree` again.

**When to use:** Tab content switching, wizard steps, any "show one of N panels" layout.

**Trade-offs:** `recompose_widget` triggers a full CSS cascade invalidation and Taffy layout rebuild for the subtree. This is acceptable — it happens at most once per user interaction, not per frame. The alternative (keeping all children mounted and toggling visibility via CSS `display: none`) would require a `display` property in the CSS engine that Taffy can act on; `recompose_widget` achieves the same result without that complexity.

**Example structure:**
```rust
pub struct ContentSwitcher {
    children_factories: Vec<Box<dyn Fn() -> Box<dyn Widget>>>,
    active: Reactive<usize>,
    own_id: Cell<Option<WidgetId>>,
}

impl Widget for ContentSwitcher {
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let idx = self.active.get_untracked();
        if let Some(factory) = self.children_factories.get(idx) {
            vec![factory()]
        } else {
            vec![]
        }
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if let Some(idx) = action.strip_prefix("show_") {
            if let Ok(n) = idx.parse::<usize>() {
                self.active.set(n);
                if let Some(id) = self.own_id.get() {
                    ctx.request_recompose(id);
                }
            }
        }
    }
}
```

### Pattern 5: DirectoryTree as Tree Wrapper

**What:** `DirectoryTree` owns a `Tree` widget and wraps it. It does NOT extend `Tree` via Rust inheritance (there is none) — it creates a `Tree` in `compose()` and injects `TreeNode` data loaded from the filesystem. When a node is expanded (`NodeExpanded` message received), it lazily loads child entries from disk and mutates the `Tree`'s root via `tree.root.borrow_mut()`, then sets `tree.dirty`.

**When to use:** This is the only widget that needs filesystem access. The lazy loading fits the existing `run_worker` pattern: expansion triggers an async worker that reads the directory, then posts a message back with the child entries.

**Trade-offs:** DirectoryTree cannot directly call methods on its child `Tree` widget through the arena (it does not have the Tree's `WidgetId` at compose time). Solution: store the `Tree` in a `Rc<Tree>` field on `DirectoryTree`, pass a clone into `compose()`. The `Rc` is shared between `DirectoryTree` (for data mutation) and the arena entry (for rendering). This matches the existing pattern used by `Tabs`/`TabbedContent`.

**Example structure:**
```rust
pub struct DirectoryTree {
    tree: Rc<Tree>,   // shared with arena
    root_path: PathBuf,
    own_id: Cell<Option<WidgetId>>,
}

impl Widget for DirectoryTree {
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        // Clone the Rc — both DirectoryTree and the arena entry
        // point to the same Tree.
        vec![Box::new(DirectoryTreeInner {
            tree: self.tree.clone(),
        })]
    }

    fn on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation {
        if let Some(msg) = event.downcast_ref::<tree_view::messages::NodeExpanded>() {
            // Spawn worker to load directory children at msg.path
            if let Some(id) = self.own_id.get() {
                let path = self.resolve_path(&msg.path);
                ctx.run_worker(id, async move {
                    std::fs::read_dir(&path)
                        .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
                        .unwrap_or_default()
                });
            }
        }
        EventPropagation::Continue
    }
}
```

---

## Data Flow

### Event Dispatch Flow

```
crossterm KeyEvent
    ↓
EventLoop (flume channel)
    ↓
Check active_overlay first (if Some: route to overlay)
    ↓ (if None)
focused_widget → on_event(&dyn Any, &AppContext) → EventPropagation
    ↓ Continue
parent widget → on_event → EventPropagation
    ↓ Continue
screen root → on_event
    ↓
Drain message_queue (post_message targets)
    ↓
Drain pending_screen_pushes / pending_screen_pops
    ↓
Drain pending_recompose (recompose_widget)
    ↓
Drain pending_overlay_dismiss
    ↓
CSS cascade (if dirty)
    ↓
Taffy layout pass (if dirty)
    ↓
ratatui render pass → Terminal::draw
```

### Screen Stack State Changes

```
push_screen(screen_widget):
  mount_widget(screen, parent=None, ctx)
  ctx.screen_stack.push(id)
  compose_subtree(id, ctx)
  advance_focus(ctx)   ← needed; not yet in push_screen, add it

pop_screen():
  id = ctx.screen_stack.pop()
  unmount_widget(id, ctx)   ← recursively removes all children, clears focus
  advance_focus(ctx)        ← re-seats focus on restored screen
```

### Widget Visibility (ContentSwitcher)

```
active.set(new_idx)
    ↓
ctx.request_recompose(own_id)
    ↓ (event loop drains pending_recompose)
recompose_widget(id, ctx):
  unmount all children
  compose_subtree(id, ctx)   ← only new active child mounted
  mark_subtree_dirty(id, ctx)
  advance focus to first focusable child
    ↓
CSS cascade re-run for new subtree
    ↓
Taffy layout rebuild for new subtree
```

### Toast Lifecycle

```
Widget posts Toast intent:
  ctx.active_overlay = Some(Box::new(Toast { expiry, message }))

Every render frame:
  Toast.render() checks Instant::now() >= expiry
  If expired: ctx.dismiss_overlay()  ← sets pending_overlay_dismiss
    ↓
Event loop drains pending_overlay_dismiss:
  ctx.active_overlay = None
```

---

## Scaling Considerations

This is a single-process, single-threaded (LocalSet) framework. "Scale" here means widget tree size and event throughput, not distributed systems.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| < 100 widgets | Current architecture handles this trivially. No changes needed. |
| 100-500 widgets | CSS cascade O(N * rules) may become measurable. Add dirty-flag cascade invalidation (only re-cascade widgets whose pseudo-classes or ancestors changed). Already partially implemented via `ctx.dirty`. |
| > 500 widgets | Taffy layout tree rebuild is O(N). Consider incremental Taffy updates rather than full rebuild each frame. Not needed for v1.3. |

### Scaling Priorities

1. **First bottleneck:** CSS cascade re-running for the entire tree on every event. Mitigation: the existing `dirty` flag already scopes invalidation. Ensure CSS cascade only visits dirty subtrees.
2. **Second bottleneck:** DirectoryTree with thousands of filesystem entries. Mitigation: lazy loading via workers is already the correct pattern. Ensure Tree's flat entry rebuilding is not called more than once per expand/collapse action.

---

## Anti-Patterns

### Anti-Pattern 1: Holding Arena Reference During Mutation

**What people do:** Try to get `&mut Widget` out of `ctx.arena` and call a method that needs `&mut ctx`.

**Why it's wrong:** Rust borrow checker forbids holding `&mut ctx.arena[id]` and passing `&mut ctx` simultaneously. Results in compile error or forces unsafe code.

**Do this instead:** Widget callbacks take `&self` (not `&mut self`). Internal state uses `Cell<T>` and `Reactive<T>`. Mutations to the arena happen only through `tree.rs` helpers which take `&mut AppContext` as a whole.

### Anti-Pattern 2: Putting Toast/Popups in the Widget Tree

**What people do:** Mount Toast as a child widget of the screen with `position: absolute` CSS.

**Why it's wrong:** The Taffy layout engine lays out all mounted children. An absolutely positioned Toast would still need special CSS handling, and more importantly, it participates in z-order via widget tree traversal order, which is fragile. The SelectOverlay already establishes that active_overlay is the right pattern for floating content.

**Do this instead:** Use `ctx.active_overlay` exactly as Select and CommandPalette already do. Toast renders itself at absolute terminal coordinates in its `render()` method.

### Anti-Pattern 3: Extending Tree for DirectoryTree via Composition Bypass

**What people do:** Try to make DirectoryTree embed a Tree as a Rust field and call Tree's internal methods directly without going through the arena.

**Why it's wrong:** The widget in the arena is a `Box<dyn Widget>`. After compose/mount, Tree is owned by the arena as a trait object. You cannot downcast from the arena back to a concrete `Tree` without unsafe.

**Do this instead:** Share the Tree via `Rc<Tree>` between the DirectoryTree parent (which controls data) and the arena entry (which renders). The `Rc` clone is placed in `compose()`. This is the same pattern as TabbedContent sharing its tab list with the TabBar child.

### Anti-Pattern 4: Blocking in on_event / on_action

**What people do:** Call `std::fs::read_dir` or any blocking I/O directly in `on_event` or `on_action`.

**Why it's wrong:** These callbacks run on the Tokio LocalSet single thread. Blocking freezes the entire event loop including rendering.

**Do this instead:** Use `ctx.run_worker(id, async { ... })` for all I/O. The async block runs on Tokio's thread pool. Results arrive as `WorkerResult<T>` messages dispatched to the widget via `on_event`.

### Anti-Pattern 5: Storing WidgetId Before push_screen Returns

**What people do:** Try to get the WidgetId of a screen that has not yet been pushed.

**Why it's wrong:** `push_screen_deferred` only schedules the push; the WidgetId is not known until after the event loop processes `pending_screen_pushes`.

**Do this instead:** Store the WidgetId in the screen widget's `on_mount` callback (which always fires synchronously during `mount_widget`).

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Widget callbacks → tree mutations | `pending_*` RefCell fields on AppContext; event loop drains | Avoids &mut AppContext inside &self callbacks |
| Widget → Widget messaging | `ctx.post_message(target_id, msg)` | Typed Any; target widget receives via `on_event` |
| Widget → background I/O | `ctx.run_worker(id, async_block)` | Returns `WorkerResult<T>` via message; auto-cancelled on unmount |
| Screen stack transitions | `ctx.push_screen_deferred` / `ctx.pop_screen_deferred` | Deferred; event loop applies after current event fully handled |
| Overlay management | `ctx.active_overlay` write / `ctx.dismiss_overlay()` | One slot; overlay receives events before normal focus chain |
| CSS cascade → layout | `ComputedStyle` written to `ctx.computed_styles` then read by Taffy bridge | Layout re-runs when any widget in subtree is dirty |

### New v1.3 Integration Points

| New Feature | Integration Point | Pattern |
|-------------|-------------------|---------|
| Screen stack (fully wired) | `push_screen` / `pop_screen` + auto-focus on push/pop | Existing tree.rs functions; add `advance_focus` call after push |
| ContentSwitcher | `request_recompose` + `compose()` returning active child only | Existing `recompose_widget` machinery |
| Toast | `active_overlay` slot, auto-dismiss via expiry check in `render()` | Same as SelectOverlay / CommandPalette |
| DirectoryTree | `Tree` wrapped via `Rc<Tree>`, lazy load via `run_worker` | Rc sharing pattern (same as TabbedContent) |
| LoadingIndicator | Animated sprite; `skip_animations` flag for tests | Same animation gating as Switch/Tabs |
| MaskedInput | Wraps Input rendering with mask overlay | Can subclass Input logic via composition or duplicate render |
| OptionList | Standalone scrollable selectable list | Similar to ListView but with cursor highlight + message |
| SelectionList | Multi-cursor OptionList with checkbox states | Extends OptionList pattern |
| RichLog | Scrolling log with colored lines | Extends existing Log widget pattern |
| Static | Thin wrapper around ratatui `Paragraph` | Simplest widget — text + CSS styling only |
| Rule | Single-line horizontal/vertical separator | Thin render-only widget |
| Link | Label with underline + click/Enter action | Label + `click_action()` override |
| Digits | Large block-character numeric display | Custom render using sub-cell canvas primitives |
| Pretty | Syntax-colored data dump | Render-only; format input as styled spans |

---

## Suggested Build Order for v1.3

Based on dependency analysis of the 17 items (13 widgets + screen stack + cross-platform + crates.io):

### Tier 1 — Infrastructure (do first, unblocks everything)

1. **Screen stack wiring** — `push_screen` / `pop_screen` already exist in `tree.rs` and `AppContext`. What is missing: `advance_focus` call after push; user-facing `App::push_screen` / `App::pop_screen` public API; demo. No new widget code. Unblocks: modal workflows needed in several widget demos.

2. **Cross-platform verification** — Set up CI matrix for macOS and Linux. Identify any platform-specific rendering issues early. Not a code change — CI configuration. Do in parallel with Tier 1.

### Tier 2 — Trivial/render-only widgets (low complexity, high confidence)

These have no complex state or interactions. Build them quickly to accumulate widget count.

3. **Static** — `Paragraph` wrapper with CSS styling. ~30 LOC render. Zero new patterns.
4. **Rule** — Horizontal/vertical line. ~40 LOC. Uses canvas drawing primitives.
5. **Link** — Label + `click_action`. ~50 LOC. Existing click_action pattern.
6. **Pretty** — Formatted data with syntax coloring. ~80 LOC. Render-only.
7. **Digits** — Large block-character numbers. ~120 LOC. Custom canvas rendering.

### Tier 3 — Moderate complexity widgets

8. **LoadingIndicator** — Animated spinner using the existing animation/skip_animations gating. ~100 LOC. New: time-based animation tick (check how Switch/Tabs animate — may need `tick_animations` hook).
9. **RichLog** — Scrolling styled log lines. ~150 LOC. Similar to existing Log widget but with per-line style. Extend Log or duplicate-and-modify.
10. **MaskedInput** — Input with mask pattern (e.g. `##/##/####` for dates). ~180 LOC. Can use Input as a base by duplicating its render/edit logic with mask overlay.

### Tier 4 — New patterns required

11. **OptionList** — Selectable scrollable list. ~200 LOC. New widget but uses existing cursor/scroll pattern from ListView. Emits `OptionSelected` message.
12. **SelectionList** — Multi-select list with checkbox per item. ~250 LOC. Extends OptionList pattern with `Vec<bool>` selected state. Emits `SelectionChanged`.
13. **ContentSwitcher** — Container that shows one child at a time. ~150 LOC. Uses `request_recompose` pattern. Write after verifying `recompose_widget` handles CSS cascade correctly for subtree swap.

### Tier 5 — Complex / requires worker integration

14. **DirectoryTree** — `Tree` wrapper with filesystem lazy loading. ~300 LOC. Requires `Rc<Tree>` sharing pattern and `run_worker` for directory reads. Write after OptionList (to understand cursor/selection patterns well).
15. **Toast** — Overlay notification with auto-dismiss. ~150 LOC. Uses `active_overlay` pattern. Also needs a public `App::notify` / `Screen::notify` API for spawning toasts from application code. Write after screen stack is wired (Toast is often triggered on screen transitions).

### Tier 6 — Polish and publish

16. **crates.io publish** — `cargo publish` prerequisites: license headers, README with quickstart, `[package]` metadata (`description`, `repository`, `keywords`, `categories`), `CHANGELOG.md`, no `path =` dependencies in published crate. Do last, after all widgets pass CI.

---

## Focus Management with Screen Stack

### Current State (confirmed by code inspection)

`advance_focus` and `advance_focus_backward` in `tree.rs` both root their DFS at `ctx.screen_stack.last()`. This means focus is already scoped to the top-of-stack screen. This is correct.

### Gap: Focus on push_screen

`push_screen` (in tree.rs) does NOT call `advance_focus`. After pushing a new screen, `ctx.focused_widget` still points to the previous screen's focused widget — a widget that is still mounted and still in the arena, but belongs to a screen that is no longer rendered.

**Fix:** In the event loop, after draining `pending_screen_pushes`, call `advance_focus(ctx)` once. Alternatively, add it inside `push_screen` directly. The event loop approach is safer because `push_screen` is sometimes called directly (not deferred) in tests.

### Gap: Focus on pop_screen

`pop_screen` calls `unmount_widget(id, ctx)` which correctly clears `ctx.focused_widget` if the focused widget was in the popped screen's subtree. The restored screen below is still mounted and its widgets are still in the arena. `ctx.focused_widget` will be `None` after the pop (assuming the focused widget was in the popped screen).

**Fix:** After draining `pending_screen_pops` in the event loop, if focus is None, call `advance_focus(ctx)` to re-seat focus on the restored screen's first focusable widget.

### Focus with active_overlay

When `active_overlay` is Some, the event loop should route key events to the overlay first (before the focus chain). This is how SelectOverlay and CommandPalette already work — the event loop checks `active_overlay` before dispatching to `focused_widget`. Toast does not need key input routing (it is display-only), but a modal dialog implemented as an overlay would.

---

## Sources

- Direct code inspection: `crates/textual-rs/src/widget/mod.rs` — Widget trait, WidgetId definition
- Direct code inspection: `crates/textual-rs/src/widget/tree.rs` — push_screen, pop_screen, advance_focus, mount_widget, unmount_widget, recompose_widget
- Direct code inspection: `crates/textual-rs/src/widget/context.rs` — AppContext fields, push_screen_deferred, pop_screen_deferred, dismiss_overlay, request_recompose, run_worker
- Direct code inspection: `crates/textual-rs/src/widget/select.rs` — active_overlay usage pattern for SelectOverlay
- Direct code inspection: `crates/textual-rs/src/widget/tree_view.rs` — Tree widget: Rc pattern, flat entry model, lazy dirty rebuild
- `.planning/PROJECT.md` — confirmed v1.3 scope: 13 widgets + screen stack + cross-platform + crates.io
- `.planning/research/SUMMARY.md` — foundational architecture decisions from v1.0 research

---

*Architecture research for: textual-rs v1.3 widget parity milestone*
*Researched: 2026-03-26*
