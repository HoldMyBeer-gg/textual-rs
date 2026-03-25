# Codebase Structure

**Analysis Date:** 2026-03-24

## Directory Layout

```
textual-rs/
├── src/textual/
│   ├── __init__.py                 # Package exports (App, Widget, Screen, etc)
│   ├── __main__.py                 # CLI entry point
│   │
│   ├── app.py                      # App class (187KB - core orchestrator)
│   ├── screen.py                   # Screen class (76KB - top-level container)
│   ├── widget.py                   # Widget base class (179KB - core component)
│   ├── message_pump.py             # MessagePump class (33KB - async dispatch)
│   ├── dom.py                      # DOMNode base (66KB - DOM tree logic)
│   │
│   ├── css/                        # CSS parsing and styling
│   │   ├── parse.py                # CSS selector/declaration parser
│   │   ├── stylesheet.py           # Stylesheet class, rule management
│   │   ├── styles.py               # Computed styles per widget
│   │   ├── tokenize.py             # CSS tokenizer
│   │   ├── tokenizer.py            # Tokenizer implementation
│   │   ├── model.py                # CSS AST data structures
│   │   ├── match.py                # Selector matching logic
│   │   ├── query.py                # DOM query system (selector search)
│   │   ├── scalar.py               # CSS scalar values (units: px, fr, %, w)
│   │   ├── scalar_animation.py     # Animation support for scalars
│   │   ├── transition.py           # CSS transitions
│   │   ├── types.py                # CSS type definitions
│   │   ├── _style_properties.py    # Property descriptor classes
│   │   ├── _styles_builder.py      # Declaration builder
│   │   └── errors.py               # CSS error types
│   │
│   ├── layouts/                    # Layout algorithm implementations
│   │   ├── vertical.py             # Stack widgets vertically
│   │   ├── horizontal.py           # Stack widgets horizontally
│   │   ├── grid.py                 # Grid layout
│   │   ├── stream.py               # Flow layout (wrap content)
│   │   ├── factory.py              # Layout factory
│   │   └── __init__.py             # Layout exports
│   │
│   ├── drivers/                    # Terminal driver implementations
│   │   ├── windows_driver.py       # Windows terminal (conhost/Windows Terminal)
│   │   ├── linux_driver.py         # Unix/Linux terminal
│   │   ├── transports.py           # I/O transport abstraction
│   │   └── __init__.py
│   │
│   ├── widgets/                    # Built-in widget library
│   │   ├── _button.py              # Button implementation
│   │   ├── _input.py               # Text input widget
│   │   ├── _label.py               # Label/text display
│   │   ├── _checkbox.py            # Checkbox widget
│   │   ├── _tree.py                # Tree view widget
│   │   ├── _select.py              # Dropdown select
│   │   ├── _data_table.py          # Data table widget
│   │   ├── text_area.py            # Code/text editor
│   │   ├── markdown.py             # Markdown renderer
│   │   ├── tabbed_content.py       # Tabbed interface
│   │   ├── _toast.py               # Toast notifications
│   │   ├── _tooltip.py             # Hover tooltips
│   │   ├── static.py               # Static renderable container
│   │   ├── rule.py                 # Horizontal/vertical line
│   │   ├── collapsible.py          # Collapsible section
│   │   └── __init__.py             # Widget exports
│   │
│   ├── containers.py               # Container widgets (Group, Horizontal, Vertical)
│   ├── scroll_view.py              # Scrollable container base
│   │
│   ├── _compositor.py              # Render pipeline (45KB)
│   ├── layout.py                   # Layout interfaces and placement
│   ├── _arrange.py                 # Widget arrangement logic
│   ├── strip.py                    # Strip class (line rendering) (28KB)
│   │
│   ├── message.py                  # Message base class
│   ├── events.py                   # Event types (Key, Mouse, Resize, etc)
│   ├── messages.py                 # App messages (Prune, etc)
│   │
│   ├── reactive.py                 # Reactive property descriptor (19KB)
│   ├── signal.py                   # Signal/pubsub mechanism
│   │
│   ├── geometry.py                 # Geometric types (Region, Size, Offset) (42KB)
│   ├── color.py                    # Color class (28KB)
│   ├── style.py                    # Visual style attributes
│   │
│   ├── binding.py                  # Key binding system
│   ├── keys.py                     # Key parsing and names
│   ├── actions.py                  # Action system (command dispatch)
│   ├── command.py                  # Command palette system
│   │
│   ├── timer.py                    # Timer scheduling
│   ├── worker.py                   # Background worker tasks
│   ├── worker_manager.py           # Worker lifecycle management
│   │
│   ├── cache.py                    # Caching utilities (LRU, FIFO)
│   ├── content.py                  # Rich text content handling (64KB)
│   ├── markup.py                   # Markup parsing (13KB)
│   │
│   ├── pilot.py                    # Testing automation (23KB)
│   ├── validation.py               # Input validation (14KB)
│   ├── notifications.py            # Notification system
│   ├── suggestions.py              # Autocomplete suggestions
│   │
│   ├── theme.py                    # Theme definitions
│   ├── design.py                   # Design system/constants
│   │
│   ├── renderables/                # Custom Rich renderables
│   │   ├── blank.py                # Blank space
│   │   ├── background_screen.py    # Background fill
│   │   └── __init__.py
│   │
│   ├── document/                   # Document/text models
│   │   ├── _document.py            # Document base class
│   │   └── __init__.py
│   │
│   ├── demo/                       # Demo application
│   │   ├── demo_app.py             # Demo app implementation
│   │   └── __init__.py
│   │
│   ├── _context.py                 # Context variables (active_app, active_message_pump)
│   ├── _animator.py                # Animation system (21KB)
│   ├── _loop.py                    # Event loop utilities
│   ├── _queue.py                   # Async queue wrapper
│   │
│   ├── filter.py                   # Line filters (colorization, etc)
│   ├── highlight.py                # Syntax highlighting
│   │
│   ├── constants.py                # Global constants
│   ├── errors.py                   # Custom exception types
│   ├── features.py                 # Feature flags
│   │
│   ├── compose.py                  # compose() function (widget composition)
│   ├── walk.py                     # Tree walking utilities
│   ├── getters.py                  # Property getter utilities
│   │
│   └── [120+ internal modules]     # Private utilities (_*, internals)
│
├── tests/                          # Test suite
│   ├── test_app.py
│   ├── test_widget.py
│   ├── test_screen.py
│   ├── test_css.py
│   └── [many more test files]
│
├── examples/                       # Example applications
│   ├── calculator.py
│   ├── hello_world.py
│   └── [30+ example apps]
│
├── docs/                           # Documentation
│   ├── api/                        # API reference
│   ├── examples/                   # Code examples
│   └── guide/                      # User guide pages
│
└── pyproject.toml                  # Poetry config, version, dependencies
```

## Directory Purposes

**`src/textual/`:**
- Purpose: Main Textual package, all framework code
- Contains: 121 Python files, ~40K lines of code
- Key files: app.py, screen.py, widget.py, message_pump.py, dom.py

**`src/textual/css/`:**
- Purpose: CSS parsing, styling, and selector matching subsystem
- Contains: Parser, tokenizer, style computation, CSS AST
- Key files: parse.py, stylesheet.py, styles.py, match.py, tokenizer.py
- Critical for: Widget styling, layout specification, selector queries

**`src/textual/layouts/`:**
- Purpose: Widget layout algorithm implementations
- Contains: Vertical, Horizontal, Grid, Stream layout engines
- Pattern: Each layout implements `Layout.arrange()` to position children
- Used by: Compositor during reflow phase

**`src/textual/drivers/`:**
- Purpose: Terminal driver abstraction and platform-specific implementations
- Contains: Windows driver, Linux/Unix driver, transport layer
- Critical for: Input/output, terminal capabilities detection

**`src/textual/widgets/`:**
- Purpose: Built-in interactive and display widgets
- Contains: Button, Input, Label, Tree, DataTable, TextArea, etc.
- Pattern: Each widget extends Widget, implements compose/render, defines messages
- Expandable: Users create custom widgets by extending Widget

**`src/textual/renderables/`:**
- Purpose: Custom Rich renderables for Textual (background fills, blanks)
- Contains: Rich library integration, custom render logic
- Used by: Widgets, compositor

**`tests/`:**
- Purpose: Test suite with unit/integration tests
- Pattern: Test files parallel main code (test_widget.py → widget.py)
- Tool: pytest + pilot (Textual's testing automation framework)

**`examples/`:**
- Purpose: Example applications demonstrating Textual features
- Contains: ~30+ runnable demo apps
- Pattern: Each file is a standalone runnable Textual app

**`docs/`:**
- Purpose: User documentation, guides, API reference
- Tool: MkDocs with custom Textual extensions

## Key File Locations

**Entry Points:**
- `src/textual/app.py` (lines 296+): `App` class, `App.run()` method
- `src/textual/__init__.py`: Package-level exports for public API
- `src/textual/__main__.py`: CLI runner for `python -m textual`

**Configuration:**
- `src/textual/constants.py`: Global constants (MAX_FPS, DEBUG, etc)
- `src/textual/design.py`: Design tokens and color schemes
- `src/textual/theme.py`: Theme definitions

**Core Logic:**
- `src/textual/widget.py` (line 1+): Widget base class, render pipeline
- `src/textual/screen.py` (line 1+): Screen class, focus management
- `src/textual/message_pump.py` (line 115+): MessagePump class, async dispatch
- `src/textual/dom.py` (line 133+): DOMNode base, CSS query, tree walking

**Layout & Rendering:**
- `src/textual/layout.py`: Layout interface, WidgetPlacement
- `src/textual/_arrange.py`: Widget arrangement (dock, split, layout phases)
- `src/textual/_compositor.py` (line 282+): Compositor class, render updates
- `src/textual/strip.py` (line 69+): Strip class, line rendering

**Styling:**
- `src/textual/css/stylesheet.py`: Stylesheet class, rule loading
- `src/textual/css/styles.py` (line 124+): Styles class, property descriptors
- `src/textual/css/parse.py`: CSS parser, selector parsing
- `src/textual/css/match.py`: Selector matching logic

**Reactivity:**
- `src/textual/reactive.py` (line 125+): Reactive descriptor, watchers
- `src/textual/message.py` (line 23+): Message base class
- `src/textual/events.py`: Event type definitions

**Testing:**
- `src/textual/pilot.py` (line 1+): Pilot class for app automation

## Naming Conventions

**Files:**
- `widget.py` - Public, core classes (Widget, Screen, App)
- `_widget.py` - Internal implementation details (not directly imported)
- `_internal_module.py` - Private modules starting with underscore
- `__init__.py` - Package initialization, re-exports
- `errors.py`, `constants.py` - Standalone utility modules

**Directories:**
- `css/` - CSS subsystem
- `layouts/` - Layout algorithms
- `widgets/` - Built-in widgets
- `drivers/` - Terminal drivers
- `renderables/` - Rich renderables

**Classes:**
- `PascalCase` (App, Widget, Screen, Button, Input)
- Exception classes end with `Error` or `Exception` (AppError, ScreenStackError)
- Message subclasses: Noun + optional action (Key, MouseMove, Button.Pressed)

**Methods:**
- Public: `snake_case` (render, mount, query_one)
- Private: `_snake_case` (\_process_messages, \_get_children)
- Properties: `snake_case` (app, screen, parent)
- Callbacks: `on_<event>` or `on_<widget>_<event>` (on_mount, on_button_pressed)

**Attributes:**
- Public: `snake_case` (id, classes, value)
- Private: `_snake_case` (_task, _parent, _message_queue)
- Magic/internal: `__name` (\_\_parent, \_\_post_init\_\_)
- Constants: `UPPER_CASE` (DEFAULT_CSS, MAX_FPS)

## Where to Add New Code

**New Widget:**
- Implementation: `src/textual/widgets/` directory (e.g., `_my_widget.py`)
- Export: Add to `src/textual/widgets/__init__.py`
- Tests: `tests/test_widgets/test_my_widget.py`
- Example: `examples/my_widget_demo.py`

**Pattern:**
```python
# src/textual/widgets/_my_widget.py
from textual.widget import Widget
from textual.reactive import Reactive

class MyWidget(Widget):
    DEFAULT_CSS = """
    MyWidget {
        height: auto;
    }
    """

    value: str = Reactive("")

    def render(self) -> str:
        return self.value

    def compose(self) -> ComposeResult:
        yield Label("Child widgets here")
```

**New Feature/Module:**
- Core functionality: `src/textual/feature_name.py`
- CSS system: `src/textual/css/feature_name.py`
- Widget-specific: `src/textual/widgets/_feature.py`
- Tests: Mirror structure in `tests/` directory

**New CSS Property:**
- Parser: Add to `src/textual/css/parse.py` (DeclarationError handling)
- Style descriptor: Add to `src/textual/css/_style_properties.py`
- Styles class: Add to `src/textual/css/styles.py` (RulesMap TypedDict, property)

**Layout Algorithm:**
- Implementation: `src/textual/layouts/my_layout.py`
- Extend: `Layout` ABC from `src/textual/layout.py`
- Register: Add to layout factory in `src/textual/layouts/factory.py`

**Utilities:**
- Shared helpers: `src/textual/` root (if widely used)
- Internal utilities: `src/textual/_module_name.py` (private import only)
- Math/geometry: `src/textual/geometry.py` (extend Offset, Size, Region)

## Special Directories

**`src/textual/demo/`:**
- Purpose: Built-in demo application (shown with `textual run`)
- Generated: No
- Committed: Yes
- Entry: `demo_app.py` with DemoApp class

**`src/textual/tree-sitter/`:**
- Purpose: Tree-sitter binding for syntax highlighting (TextArea widget)
- Generated: No
- Committed: Yes

**`src/textual/document/`:**
- Purpose: Document models for text editing (TextArea)
- Generated: No
- Committed: Yes

**`tests/`:**
- Purpose: Pytest test suite with Pilot test automation
- Generated: No (tests are source code)
- Committed: Yes
- Pattern: `test_*.py` files, one per module typically

**`docs/`:**
- Purpose: MkDocs documentation and API references
- Generated: Partially (some docs auto-generated from docstrings)
- Committed: Yes
- Tool: MkDocs with mkdocs.yml config

---

*Structure analysis: 2026-03-24*
