# Architecture

**Analysis Date:** 2026-03-24

## Pattern Overview

**Overall:** Hierarchical event-driven TUI framework with reactive data binding, CSS-based styling, and asynchronous message passing.

**Key Characteristics:**
- **Message-passing system** - All communication flows through async message queues (`MessagePump`)
- **Reactive properties** - Automatic UI updates when observable properties change
- **DOM-like hierarchy** - App > Screen > Widget tree with parent-child relationships
- **CSS-driven layout** - Textual CSS Stylesheet (TCSS) for styling and layout
- **Multi-screen support** - Stack-based screen management for modal dialogs and navigation
- **Async/await throughout** - asyncio-based event loop for all async operations

## Layers

**App Layer:**
- Purpose: Root application container, event loop orchestrator, screen/theme management
- Location: `src/textual/app.py`
- Contains: Main App class, mode system, driver initialization
- Depends on: MessagePump, Screen, DOMNode, CSS system, asyncio
- Used by: Everything (top-level entry point)

**Screen Layer:**
- Purpose: Top-level container for widget trees, manages keyboard/mouse focus, tooltip system
- Location: `src/textual/screen.py`
- Contains: Screen class, layout updates, widget discovery, focus management
- Depends on: Widget, MessagePump, Compositor, layout system
- Used by: App (manages screen stack), Widgets (parent container)

**Widget Layer:**
- Purpose: Individual UI components, rendering logic, layout participation
- Location: `src/textual/widget.py`
- Contains: Widget base class, render pipeline, styling, composition
- Depends on: DOMNode, Strip (rendering), layouts, CSS styles
- Used by: Screens (composition), other widgets (nesting)

**MessagePump Layer:**
- Purpose: Core async message queue and handler dispatch
- Location: `src/textual/message_pump.py`
- Contains: Message queue, async loop, handler resolution via metaclass
- Depends on: Python asyncio, Message classes, reactive system
- Used by: App, Screen, Widget (all inherit from MessagePump)

**DOMNode Layer:**
- Purpose: Base DOM structure, CSS matching, node traversal
- Location: `src/textual/dom.py`
- Contains: DOMNode base class, CSS query API, tree walking
- Depends on: Stylesheet, CSS matcher, Message classes
- Used by: App, Screen, Widget (provides DOM capabilities)

**CSS System:**
- Purpose: Parsing TCSS, matching selectors, computing styles
- Location: `src/textual/css/` directory
- Contains: Parser, tokenizer, selector matching, Styles object
- Key files: `parse.py` (selector parsing), `stylesheet.py` (CSS rules), `styles.py` (computed styles)
- Depends on: CSS tokenizer, style property builders
- Used by: DOMNode, Widget (style application), layouts (size/spacing)

**Layout System:**
- Purpose: Widget positioning and sizing based on CSS and parent constraints
- Location: `src/textual/layout.py`, `src/textual/layouts/` (concrete implementations)
- Contains: Layout ABC, WidgetPlacement, DockArrangeResult
- Key implementations: VerticalLayout, HorizontalLayout, GridLayout, StreamLayout
- Depends on: CSS styles, geometry primitives
- Used by: Compositor (during reflow)

**Reactive System:**
- Purpose: Observable property descriptors that trigger UI updates
- Location: `src/textual/reactive.py`
- Contains: Reactive descriptor, computed watchers, change propagation
- Depends on: MessagePump (posts Callback events for updates)
- Used by: Widget, Screen (reactive properties)

**Compositor/Render Pipeline:**
- Purpose: Combine widget renders into terminal output, manage dirty regions
- Location: `src/textual/_compositor.py`
- Contains: Compositor class, Strip management, dirty region tracking
- Depends on: Rich library (segments), geometry, widget strips
- Used by: Screen, App (for screen updates)

**Strip (Rendering):**
- Purpose: Immutable representation of a horizontal line of styled text
- Location: `src/textual/strip.py`
- Contains: Strip class, segment composition, caching
- Depends on: Rich Segment objects, style information
- Used by: Widgets (render returns strips), Compositor (combines strips)

**Driver Layer:**
- Purpose: Terminal input/output abstraction
- Location: `src/textual/driver.py`, `src/textual/drivers/`
- Contains: Driver ABC, platform-specific implementations (Windows, Unix)
- Depends on: None (lowest level)
- Used by: App (terminal I/O)

## Data Flow

**Event Processing Loop (Core Cycle):**

1. **Input Phase** - Driver captures keyboard/mouse events
2. **Event Creation** - Events (key press, mouse move, resize) created as Message subclasses
3. **Message Queue** - Events posted to root App's message queue
4. **Message Dispatch** - MessagePump processes queue via `_process_messages_loop`
   - Handler lookup via metaclass registration (decorated handlers via `@on`)
   - Message bubbling (child to parent) if `message.bubble=True`
   - Prevent/suppress support via context managers
5. **Handler Execution** - Sync/async handlers called, may post new messages
6. **Reactive Updates** - Property changes trigger compute/watch methods, post Callback events
7. **Layout Phase** - If layout dirty, Screen calls `_arrange_widgets` via layout engines
8. **Render Phase** - Compositor compares widget strips, generates dirty region updates
9. **Terminal Output** - Updated regions sent to driver

**Reactive Property Change:**

1. Property descriptor `__set__` is called (e.g., `widget.value = new`)
2. Reactive checks if value changed (respects `always_update` flag)
3. If changed:
   - Run compute method if `compute=True`
   - Post Callback event to invoke watchers
   - Optionally trigger recompose/layout/repaint based on flags
4. Watchers execute (sync or async), may change other properties (cycle continues)

**Widget Composition:**

1. `Widget.compose()` generator yields child widgets (or uses container context managers)
2. `compose()` function builds widget list, respects parent container nesting
3. Parent's `_refresh_bindings()` called if `recompose=True` on reactive change
4. New widgets mounted via `mount()`

**State Management:**
- Widget tree structure stored in parent-child links (DOM hierarchy)
- Layout state cached in Compositor (dirty regions, widget placements)
- CSS state computed via Stylesheet matching + Styles object per widget
- Reactive values stored as instance attributes with descriptor protocol
- Messages discarded after handling (no persistence)

## Key Abstractions

**Message Hierarchy:**

```
Message (base)
├── Event (user-facing events)
│   ├── Key (keyboard input)
│   ├── MouseMove, MouseDown, MouseUp
│   ├── Resize
│   ├── Focus, Blur
│   └── Load, Show, Hide, Mount, Unmount
├── Custom widget messages (e.g., Button.Pressed, Input.Changed)
└── Callback (internal, for watcher/compute invocation)
```

- Purpose: Uniform interface for all async events and widget communication
- Pattern: Dataclass-based (from Python 3.10+), with `handler_name` auto-generated from class name
- Bubbling: Messages propagate up DOM if `bubble=True` and not stopped
- Forwarding: Messages can be forwarded to specific widgets via `_bubble_to()`

**Reactive Property Descriptor:**

```python
class Widget(DOMNode):
    value: str = Reactive("default", layout=False, repaint=True, compute=True)

    def compute_value(self) -> str:
        # Called when value is accessed, can trigger recalculation
        return self._compute_value()

    def watch_value(self, old: str, new: str) -> None:
        # Called when value changes
        self.app.notify(f"Changed to {new}")
```

- Purpose: Declarative reactive data binding
- Pattern: Descriptor protocol with lazy evaluation, compute methods, and watchers
- Flags: layout (reflow), repaint (re-render), init (run watchers on mount), compute, recompose
- Cycle detection: `TooManyComputesError` if public + private compute methods exist

**CSS Styling System:**

```python
# Textual CSS (TCSS) - subset of CSS for TUI
Screen {
    layout: vertical;
}

Button {
    width: 1fr;
    height: auto;
    border: solid $primary;
}

Button:hover {
    background: $accent;
}

#my-button {
    color: white;
}

.disabled {
    opacity: 0.5;
}
```

- Purpose: Declarative styling and layout specification
- Pattern: Selector matching (type, class, ID), specificity calculation, inheritance
- Computed Styles: `widget.styles` object holds resolved CSS properties
- Animation: Scalar animations support smooth transitions on numeric properties

**DOM Query System:**

```python
# CSS selector queries
button = self.query_one("#submit", Button)
inputs = self.query("Input.required")
self.query("Label")[0].update("Text")
```

- Purpose: Find widgets by CSS selectors at runtime
- Pattern: Query caching, type validation, exception handling (NoMatches, TooManyMatches, WrongType)
- Usage: Post-render widget discovery, validation, testing

**Layout Abstraction:**

```python
# Layout engine interface
class Layout(ABC):
    def arrange(
        self,
        parent: Widget,
        children: Sequence[Widget],
        size: Size,
        viewport: Size,
    ) -> list[WidgetPlacement]:
        # Return positioned/sized placements for children
```

- Purpose: Algorithm-based widget positioning (vertical, horizontal, grid, stream)
- Pattern: Called during reflow, returns WidgetPlacement tuples with region/offset
- Used by: arrange() function, which handles docks and splits
- Replaceable: Widgets can set `styles.layout = VerticalLayout()` etc.

**Widget Composition Context:**

```python
def compose(self) -> ComposeResult:
    with Container(id="left"):
        yield Button("Click me")
    with Container(id="right"):
        yield Input()
```

- Purpose: Declarative widget tree building with context managers
- Pattern: Container context manager records parent, yielded widgets become children
- Used by: Widget.compose() generator, mounted widgets during lifecycle

## Entry Points

**App.run():**
- Location: `src/textual/app.py` (main public API)
- Triggers: `asyncio.run()` calls `App._main()`, which initializes driver and event loop
- Responsibilities:
  - Create asyncio event loop
  - Initialize terminal driver (Windows/Unix)
  - Load CSS stylesheet
  - Create default Screen and add to stack
  - Run `_process_messages_loop()` (main message pump)
  - Cleanup on exit

**MessagePump._process_messages_loop():**
- Location: `src/textual/message_pump.py`
- Triggers: Called from `App.run()`, runs until app closes
- Responsibilities:
  - Loop: Get next message from queue (or wait for Idle)
  - Call `_process_messages()` for single message processing
  - Handle exceptions and cleanup on close
  - Emit Idle event when queue empty

**Driver.process_event_loop():**
- Location: `src/textual/driver.py` / `src/textual/drivers/`
- Triggers: Runs concurrently with message loop via asyncio tasks
- Responsibilities:
  - Poll for terminal input (keyboard, mouse, resize)
  - Convert to Event messages
  - Post to app message queue
  - Render screen updates to terminal

**Screen.on_mount():**
- Location: `src/textual/screen.py`
- Triggers: Lifecycle event when Screen is first added to stack
- Responsibilities:
  - Initialize widget tree via `compose()`
  - Load CSS if defined
  - Set initial focus
  - Trigger reactive property watchers (init=True)

## Error Handling

**Strategy:** Exceptions caught at layer boundaries, logged, but allow app to continue running.

**Patterns:**

1. **Message Handler Exceptions** - Caught in `_process_messages()`, logged via app logger
2. **Reactive Watcher Exceptions** - Caught in `invoke_watcher()`, error message posted via `app.notify()`
3. **Layout Engine Errors** - Caught in `arrange()`, falls back to blank layout
4. **CSS Parse Errors** - Caught in `Stylesheet.parse()`, `StylesheetParseError` with helpful diagnostics
5. **Widget Lifecycle Errors** - Mount/unmount exceptions logged and propagated to caller

**Special Cases:**
- `NoMatches`, `TooManyMatches`, `WrongType` - Query API exceptions, caller must handle
- `ScreenStackError` - When screen stack operations invalid (no screens, duplicate push)
- `BadIdentifier` - Validation error for CSS class/ID names

## Cross-Cutting Concerns

**Logging:**
- Framework: Python `logging` module
- Pattern: Each DOMNode has `.log` property (lazy-loaded Logger)
- Usage: `self.log.debug("message")`, `self.log.exception(exc)`

**Validation:**
- CSS: Done during parse, errors reported via `StylesheetParseError`
- Widgets: ID/class names validated with regex, `BadIdentifier` exception
- Styles: Property type checking in `StylesBuilder`, `StyleValueError` on invalid values

**Authentication:**
- Not applicable (client-side TUI)

**Caching:**
- Query results: `DOMNode._query_cache` with TTL based on CSS generation
- Layout: `Compositor` caches widget placements, invalidated on style/structure changes
- Strips: `Strip` caches division/crop results with LRU eviction
- Styles: `StylesCache` stores computed styles per widget
- CSS Parsing: LRU cache on `parse_selectors()`, `is_id_selector()` with 128/1024 entry limits

**Threading:**
- Main thread: asyncio event loop (message pump, driver)
- Worker threads: `WorkerManager` for background tasks (file I/O, blocking calls)
- Thread safety: Context variables (`active_app`, `active_message_pump`) for thread-local state
- Weak references: `WeakSet` for timers/watchers to prevent circular references

**Performance Optimization:**
- Dirty region tracking: Compositor only renders changed areas
- Layout caching: `DockArrangeResult.spatial_map` for fast widget lookup
- Reactive batching: Multiple property changes can be batched (pending implementation)
- Lazy evaluation: Styles computed on demand, not pre-computed for all widgets
- LRU caches: Strip operations, DOM queries, CSS parsing

---

*Architecture analysis: 2026-03-24*
