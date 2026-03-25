# Coding Conventions

**Analysis Date:** 2026-03-24

## Naming Patterns

**Files:**
- Module files use lowercase with underscores: `_button.py`, `_input.py`, `_label.py`
- Widget implementation files are prefixed with underscore: `src/textual/widgets/_button.py`
- Internal/private modules use single leading underscore: `_animator.py`, `_callback.py`, `_context.py`
- Test files follow pytest convention: `test_app.py`, `test_animation.py`

**Classes:**
- PascalCase for all classes: `Button`, `Widget`, `DOMNode`, `Reactive`
- Custom Exception classes end with "Error" or "Exception": `ReactiveError`, `TooManyComputesError`, `MountError`
- Message classes are nested inside widgets and use PascalCase: `Button.Pressed`, `Input.Changed`
- Message classes with compound names use separate words: `Pressed`, `Changed`, `Focused`, `Blurred`

**Functions:**
- snake_case for functions: `invoke_watcher()`, `await_watcher()`, `dispatch_key()`
- Private functions use leading underscore: `_check_watchers()`, `_initialize_reactive()`, `_compute()`
- Async functions follow same convention: `async def await_watcher(...)`
- Action methods use `action_` prefix: `action_press()` in `_button.py`
- Event handler methods use `_on_` prefix for private handlers: `_on_click()` in `_button.py`
- Watch methods use `watch_` prefix: `watch_variant()`, `watch_flat()` in `_button.py`
- Validate methods use `validate_` prefix: `validate_variant()`, `validate_label()` in `_button.py`
- Compute methods use `compute_` prefix: `compute_variant()` (public or private: `_compute_variant()`)

**Variables:**
- Instance variables with leading underscore for private: `_default`, `_layout`, `_owner`, `_sender`
- Internal reactive storage uses `_reactive_{name}` pattern: `_reactive_label`, `_reactive_variant`
- Type variables use UPPER_SNAKE_CASE: `ReactiveType`, `ReactableType`
- Boolean flags use positive language: `layout`, `repaint`, `init`, `always_update`

**Types:**
- Type aliases use descriptive names: `ButtonVariant`, `RenderResult`, `ContentText`, `RenderableType`
- Literal types are defined at module level: `ButtonVariant = Literal["default", "primary", "success", "warning", "error"]`
- Validation sets use UPPER_CASE: `_VALID_BUTTON_VARIANTS = {"default", "primary", ...}`

**Constants:**
- All-caps with underscores: `ALLOW_SELECTOR_MATCH`, `DEFAULT_CSS`, `BINDINGS`
- Private constants use leading underscore: `_VALID_BUTTON_VARIANTS`

## Code Style

**Formatting:**
- Black formatter (version 24.4.2) configured via pyproject.toml
- Line length: standard (black default ~88 characters)
- Two blank lines between top-level definitions
- One blank line between method definitions within a class

**Linting:**
- Ruff linter (target Python 3.9+)
- isort (version 5.13.2) for import organization
- mypy (version 1.0.0) for type checking

**Imports Organization:**
```
from __future__ import annotations

# Standard library imports
import asyncio
import os
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable

# Rich library imports
import rich.repr
from rich.console import Console
from rich.style import Style

# Textual imports
from textual import events
from textual.reactive import reactive
from textual.widget import Widget

# Conditional imports for TYPE_CHECKING
if TYPE_CHECKING:
    from textual.app import ComposeResult
```

**Path Aliases:**
- Textual consistently uses relative imports within the package
- No path aliases detected in core modules
- Import groups: standard library, third-party, local, TYPE_CHECKING block

## Descriptor and Reactive Patterns

**Reactive Descriptors:**
- `reactive[T]` descriptor with type parameter: `label: reactive[ContentText] = reactive[ContentText](...)`
- `reactive()` class with initialization parameters: `layout`, `repaint`, `init`, `always_update`, `compute`, `recompose`, `bindings`, `toggle_class`
- `var[T]` descriptor for non-auto-refresh reactives: no automatic layout/repaint
- `Initialize[T]` callable wrapper for lazy initialization: `reactive(Initialize(get_names))`

**Implementation Details:**
- Descriptors use `__set_name__()` to introspect owner class and register in `_reactives` dict
- Internal value stored in `_reactive_{name}` attribute
- Reactive attributes cached in `_default_{name}` at class level
- Default value can be literal, callable, or `Initialize` instance
- See `src/textual/reactive.py` lines 1-533 for full descriptor implementation

**Watch Methods:**
- Pattern: `watch_{attribute_name}(self, old_value, new_value) -> None` (2 or 3 parameters accepted)
- Private watchers: `_watch_{attribute_name}()` called first (if exists), then public version
- Watchers can be async and return awaitable
- Global watchers registered via `_watch()` function (line 505-532)
- Watchers invoked in order: private `_watch_*`, public `watch_*`, global watchers from `__watchers` dict

**Validate Methods:**
- Pattern: `validate_{attribute_name}(self, value: T) -> T`
- Private validators: `_validate_{attribute_name}()` checked first, then public
- Must return modified/validated value (enforced return value)
- Called before watchers in `_set()` method

**Compute Methods:**
- Pattern: `compute_{attribute_name}(self) -> T` or `_compute_{attribute_name}(self) -> T`
- Called on attribute access if present, returns fresh computed value
- Makes reactive read-only (cannot be set directly)
- Multiple computes tracked in `_computes` list on class

**Toggle Class Pattern:**
- Use `toggle_class` parameter on reactive: `compact = reactive(False, toggle_class="-textual-compact")`
- Automatically toggles CSS class based on truthiness of value
- Supports multiple classes: `toggle_class="class1 class2"`

## Message and Event Handlers

**Message Classes:**
- Nested inside widget classes: `class Button.Pressed(Message)`
- Inherit from `Message` base class in `src/textual/message.py`
- Include `__init__` with relevant attributes
- Define `control` property returning the widget that originated the message
- Set class attributes: `bubble: ClassVar[bool] = True`, `namespace: ClassVar[str] = ""`

**Handler Name Convention:**
- Automatically derived from message class name via `__init_subclass__()`
- Converts CamelCase to snake_case: `Button.Pressed` → `on_button_pressed`
- Stored in `handler_name: ClassVar[str]` on message class
- Can be called explicitly as `on_{handler_name}(self, message: MessageType) -> None`

**Event Handler Methods:**
- Sync handlers: `def on_button_pressed(self, message: Button.Pressed) -> None:`
- Async handlers: `async def on_button_pressed(self, message: Button.Pressed) -> None:`
- Private event handlers: `async def _on_click(self, event: events.Click) -> None:` (internal implementation)
- Event handlers can call `event.stop()` to prevent bubbling

**@on Decorator Pattern:**
- Located in `src/textual/_on.py`
- Syntax: `@on(MessageType, selector="css_selector")`
- Selector matches widget via CSS selectors when message has `control` property
- Stores metadata in `_textual_on` method attribute as list of tuples
- Example from test: `@on(Button.Pressed, "#quit")`
- Keyword arguments for additional message attributes in `ALLOW_SELECTOR_MATCH`

**Action Methods:**
- Pattern: `action_{name}(self) -> None:`
- Can be bound to key bindings in `BINDINGS` class attribute
- Example: `action_press(self)` in Button widget

## Error Handling

**Patterns:**
- Create custom exception classes inheriting from base exception types
- Exceptions documented in docstrings with "Raises:" sections
- Try/except blocks use specific exception types (not bare except)
- Context managers with specific exceptions: `except NoScreen:`, `except AttributeError:`, `except ValueError:`

**Error Types in Textual:**
- Base: `Exception`, `ReactiveError` (base for reactive-specific errors)
- Specific: `TooManyComputesError`, `MountError`, `WidgetError`, `NoMatches`, `WrongType`
- Custom: `InvalidButtonVariant` raises with formatted friendly error messages

**Error Messages:**
- Use f-strings with repr for objects: `f"Invalid value {value!r}"`
- Include context and suggestion when helpful
- Example: `f"Can't set {obj}.{self.name!r}; reactive attributes with a compute method are read-only"`

## Logging

**Framework:** `_rich_traceback_omit = True` variable set at function start to hide from Rich tracebacks

**Patterns:**
- Used in reactive system for user-friendly error reporting
- Consistent with Rich's traceback formatting

## Comments

**When to Comment:**
- Explain *why* not *what* (code is readable, intent should be clear)
- Complex algorithms or non-obvious logic
- Important state transitions or edge cases
- Example: `# Watcher may have changed the state, so run compute again` (line 86 in reactive.py)

**Code Comments:**
- Inline comments explain purpose of complex code blocks
- Comments precede the code they explain

**No JSDoc/TSDoc:**
- Textual uses NumPy-style docstrings (Args, Returns, Raises sections)
- Examples included with code blocks using triple-backtick markdown

## Docstring Style

**Function/Method Docstrings:**
- Triple-quoted strings
- First line: brief description (one sentence)
- Blank line, then expanded description (if needed)
- Args section with type info
- Returns section
- Raises section (if applicable)
- Example section with code blocks (markdown)

**Example:**
```python
def validate_variant(self, variant: str) -> str:
    """Validate button variant.

    Args:
        variant: The variant name to validate.

    Returns:
        The validated variant string.

    Raises:
        InvalidButtonVariant: If variant is not in _VALID_BUTTON_VARIANTS.
    """
```

## CSS Integration

**DEFAULT_CSS Class Variable:**
- Multi-line string containing TCSS (Textual CSS)
- Defines widget-scoped styles
- Uses nesting syntax for pseudo-classes and modifiers: `&:hover`, `&.-active`, `&:disabled`
- Class methods use `-{variant}` pattern: `.-primary`, `.-success`, `.-warning`, `.-error`
- Examples: `src/textual/widgets/_button.py` lines 49-229

**CSS Classes Management:**
- `add_class()`, `remove_class()`, `set_class()` for runtime class toggling
- Toggle based on reactive attributes: `watch_flat()` calls `set_class()`
- Single class name vs space-separated strings handled automatically

**Pseudo-Classes:**
- Applied automatically: `:hover`, `:focus`, `:disabled`
- Custom pseudo-classes tracked in `pseudo_classes` set on widgets

## Type Annotations

**Style:**
- PEP 484/585 compliant
- Use `from __future__ import annotations` for forward references
- Generic types with `[]` syntax: `reactive[ContentText]`, `Callable[[int], str]`
- Union types with `|` operator (Python 3.10+)
- `TypeVar` for generic constraints
- `TYPE_CHECKING` guard for circular imports

**Pattern Examples:**
```python
from __future__ import annotations
from typing import TYPE_CHECKING, Generic, TypeVar

ReactiveType = TypeVar("ReactiveType")

class Reactive(Generic[ReactiveType]):
    def __get__(self, obj: Reactable | None, obj_type: type) -> Reactive[ReactiveType] | ReactiveType:
        ...
```

## Module Organization

**Structure:**
- Module docstring at top (triple quotes)
- `from __future__ import annotations`
- Imports (standard library, third-party, local, TYPE_CHECKING block)
- Type definitions and TypeVars
- Exception classes
- Helper functions
- Main classes

**Class Organization:**
- Class docstring
- Class variables (BINDINGS, DEFAULT_CSS, ALLOW_SELECT)
- Reactive descriptors
- Nested message classes
- `__init__` method
- Property methods
- Public methods
- Private methods (underscore prefix)
- Event/message handlers

---

*Convention analysis: 2026-03-24*
