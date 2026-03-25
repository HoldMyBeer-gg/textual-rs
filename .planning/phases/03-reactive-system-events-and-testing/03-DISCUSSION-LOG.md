# Phase 3: Reactive System, Events, and Testing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 03-reactive-system-events-and-testing
**Areas discussed:** Reactive strategy, Event dispatch model, Key/mouse routing, TestApp/Pilot design

---

## Reactive Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| reactive_graph crate | Battle-tested signals from Leptos. RwSignal<T>, Memo<T>, Effect for auto-tracking. Needs Executor::init_tokio() spike. | ✓ |
| Hand-rolled signals | Simple Reactive<T> wrapper with explicit notify/subscribe. No dependency tracking graph. | |
| You decide | Claude evaluates both during research and picks the best fit. | |

**User's choice:** reactive_graph crate
**Notes:** None

### Render Batching

| Option | Description | Selected |
|--------|-------------|----------|
| Effect sends RenderRequest via flume | reactive_graph Effect detects changes, posts RenderRequest to flume. App coalesces into one render per tick. | ✓ |
| Synchronous dirty-flag only | Setting Reactive<T> marks widget dirty immediately. Next render pass picks up all dirty widgets. | |
| You decide | Claude decides batching mechanism. | |

**User's choice:** Effect sends RenderRequest via flume
**Notes:** None

### Spike Requirement

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, spike first | Researcher verifies Executor::init_tokio() works with LocalSet, effects fire correctly, borrow pattern compatible. Fallback to hand-rolled if fails. | ✓ |
| No, commit to reactive_graph | Skip spike, assume it works. Fix issues during execution. | |
| No, go hand-rolled | Skip reactive_graph entirely. Build simple signal system. | |

**User's choice:** Yes, spike first
**Notes:** None

---

## Event Dispatch Model

| Option | Description | Selected |
|--------|-------------|----------|
| on_event(&self, event: &dyn Any) pattern | Widget trait gets on_event. Widgets downcast to concrete types. Returns EventPropagation. | ✓ |
| Handler registration map | Widgets register closures for specific message types. TypeId-keyed HashMap. | |
| You decide | Claude picks dispatch pattern. | |

**User's choice:** on_event with dyn Any downcasting
**Notes:** None

### Bubbling Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Walk parent chain, call on_event at each level | Collect parent chain as Vec<WidgetId>, iterate calling on_event. Stop via return value. | ✓ |
| Queue-based async dispatch | Events go into per-widget queue. Dispatch loop processes in parent order. | |
| You decide | Claude decides. | |

**User's choice:** Walk parent chain
**Notes:** None

---

## Key/Mouse Routing

### Keyboard Routing Order

| Option | Description | Selected |
|--------|-------------|----------|
| Focused widget first, then bubble up | Key events go to focused widget via on_event. If not consumed, bubble up parent chain. App handles last. | ✓ |
| App-level key bindings first | App checks global table first. Unmatched keys go to focused widget. | |
| You decide | Claude picks routing order. | |

**User's choice:** Focused widget first, then bubble up
**Notes:** None

### Key Binding Declaration

| Option | Description | Selected |
|--------|-------------|----------|
| Static method returning binding table | Widget trait gets key_bindings() returning [KeyBinding]. Action strings dispatch to on_action(). | ✓ |
| Inline in on_event | Widgets check for specific keys inside on_event. No separate table. | |
| You decide | Claude decides. | |

**User's choice:** Static binding table
**Notes:** None

---

## TestApp/Pilot Design

### Test API Style

| Option | Description | Selected |
|--------|-------------|----------|
| Async tests with #[tokio::test] | TestApp creates own LocalSet. Pilot methods async. settle() drains event loop. | ✓ |
| Sync wrapper hiding async | TestApp provides sync methods that internally block. Simpler but hides event loop. | |
| You decide | Claude picks. | |

**User's choice:** Async tests
**Notes:** None

### Snapshot Format

| Option | Description | Selected |
|--------|-------------|----------|
| insta with plain text buffer lines | Render to TestBackend, extract rows as strings, snapshot with insta. Human-readable diffs. | ✓ |
| insta with styled cell data | Snapshot includes content + style info. Catches style regressions but harder to read. | |
| You decide | Claude picks. | |

**User's choice:** insta with plain text buffer lines
**Notes:** None

---

## Claude's Discretion

- Timer/interval implementation details
- Exact Reactive<T> API surface beyond conventions
- Whether on_event takes &mut AppContext or &AppContext
- proptest strategy design for CSS parser
- Mouse event types beyond click
- How settle() detects quiescence

## Deferred Ideas

None
