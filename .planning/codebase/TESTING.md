# Testing Patterns

**Analysis Date:** 2026-03-24

## Test Framework

**Runner:**
- pytest 8.3.1+
- pytest-asyncio for async test support
- pytest-cov for code coverage
- pytest-textual-snapshot for snapshot testing
- pytest-xdist for parallel test execution
- Config: `pyproject.toml` [tool.pytest.ini_options]

**Assertion Library:**
- Standard Python `assert` statements
- No external assertion library required

**Run Commands:**
```bash
pytest tests/                          # Run all tests
pytest tests/ -v                       # Verbose output
pytest tests/ -k test_name             # Run specific test
pytest tests/ --no-header -x           # Stop on first failure
pytest tests/test_app.py               # Run single file
pytest tests/ -m "not syntax"          # Skip marked tests
pytest --cov=textual tests/            # Generate coverage report
```

**Configuration:**
```toml
[tool.pytest.ini_options]
asyncio_mode = "auto"                 # Automatic async event loop management
testpaths = ["tests"]
addopts = "--strict-markers"
markers = [
    "syntax: marks tests that require syntax highlighting (deselect with '-m \"not syntax\"')",
]
asyncio_default_fixture_loop_scope = "function"
```

## Test File Organization

**Location:**
- Tests co-located with codebase structure: `tests/` directory mirrors functionality
- Subdirectories for specific widget/component tests: `tests/animations/`, `tests/css/`, `tests/widgets/`
- Snapshot tests in: `tests/snapshot_tests/`
- General tests at root: `tests/test_*.py`

**Naming:**
- Test files: `test_{feature}.py`
- Test functions: `test_{specific_behavior}()`
- Test classes: Not typically used; prefer function-based tests

**Structure:**
```
tests/
├── test_app.py              # App-level tests
├── test_animation.py        # Animation system tests
├── snapshot_tests/          # Visual snapshot tests
│   ├── __snapshots__/       # Generated snapshot comparisons
│   ├── test_snapshots.py    # Snapshot test definitions
│   └── snapshot_apps/       # Apps for snapshot testing
├── widgets/                 # Widget-specific tests
├── css/                     # CSS parsing and matching tests
├── layouts/                 # Layout system tests
└── ...
```

## Test Structure

**Basic Async Test Pattern:**
```python
async def test_hover_update_styles():
    """Docstring describing what is being tested."""
    app = MyApp(ansi_color=False)
    async with app.run_test() as pilot:
        button = app.query_one(Button)
        initial_background = button.styles.background

        await pilot.hover(Button)

        assert button.pseudo_classes != initial_background.pseudo_classes
        assert button.styles.background != initial_background
```

**Basic Sync Test Pattern:**
```python
def test_batch_update():
    """Test `batch_update` context manager"""
    app = App()
    assert app._batch_count == 0

    with app.batch_update():
        assert app._batch_count == 1

    assert app._batch_count == 0
```

**App Test Pattern:**
```python
class MyApp(App):
    def compose(self) -> ComposeResult:
        yield Input()
        yield Button("Click me!")

async def test_specific_behavior():
    app = MyApp()
    async with app.run_test() as pilot:
        # Test code here
        pass
```

## Test Patterns with Pilot

**Pilot System:**
- Simulates user interaction: key presses, mouse events, resizing
- Accessible via `async with app.run_test() as pilot:`
- Provides methods to drive the app without a real terminal

**Key Pilot Methods:**

**Keyboard Input:**
```python
async def test_input():
    app = MyApp()
    async with app.run_test() as pilot:
        await pilot.press("enter")                    # Single key
        await pilot.press(*"Darren")                  # Multiple chars
        await pilot.press("shift+tab")                # Key combinations
        await pilot.press("enter", "enter")           # Multiple presses
```

**Mouse Interaction:**
```python
await pilot.click(Button)                    # Click widget by type
await pilot.click("#my-button")              # Click by selector
await pilot.click(offset=(5, 5))             # Click with offset
await pilot.hover(Button)                    # Hover over widget
await pilot.hover("#tabs", offset=(10, 0))  # Hover at offset

await pilot.mouse_down(Button)               # Mouse down event
await pilot.mouse_up(Button)                 # Mouse up event
```

**Screen and Layout:**
```python
await pilot.resize_terminal(width=120, height=40)  # Resize terminal
await pilot.pause()                                 # Wait briefly
await pilot.wait_for_screen()                      # Wait for screen update
await pilot.wait_for_idle()                        # Wait for app to idle
await pilot.wait_for_animation()                   # Wait for animations
await pilot.wait_for_scheduled_animations()        # Wait for deferred animations
```

**Animation Waiting:**
```python
async def test_animate_height():
    app = AnimApp()
    async with app.run_test() as pilot:
        static = app.query_one(Static)
        static.styles.animate("height", 100, duration=0.5, easing="linear")
        await pilot.wait_for_animation()  # Blocks until animation completes
        assert static.styles.height.value == 100
```

## Query and Assertion Patterns

**DOM Querying:**
```python
# Query by type
button = app.query_one(Button)
inputs = app.query(Input)

# Query by selector
element = app.query_one("#my-id")
elements = app.query(".my-class")

# Expect specific type
button = app.query_one(Button, expect_type=Button)

# Handle no matches
try:
    widget = app.query_one(Static, no_default=True)
except NoMatches:
    pass
```

**State Assertions:**
```python
# Check pseudo-classes
assert "hover" in button.pseudo_classes
assert button.pseudo_classes == {"blur", "can-focus", "dark", "enabled", ...}

# Check styles
assert button.styles.background != initial_background
assert static.styles.height.value == 100

# Check attributes
assert button.disabled == False
assert button.visible == True

# Check content
assert button.label == "Click me!"
```

## Snapshot Testing

**Framework:** pytest-textual-snapshot

**Purpose:** Capture visual rendering of Textual apps for regression testing

**Test Pattern:**
```python
def test_switches(snap_compare):
    """Tests switches rendering with user interaction."""
    press = [
        "shift+tab",
        "enter",      # toggle off
        "shift+tab",
        "wait:20",    # Wait 20 screen updates
        "enter",      # toggle on
        "wait:20",
    ]
    assert snap_compare(WIDGET_EXAMPLES_DIR / "switch.py", press=press)
```

**How snap_compare Works:**
1. Takes path to a Textual app file (must have `if __name__ == "__main__": ... app.run()`)
2. Optionally takes `press` list: sequence of key presses and wait commands
3. Simulates the interaction sequence
4. Captures rendered terminal output
5. Compares against stored snapshot in `__snapshots__/` directory
6. Returns True if matches, raises if different (use `-vv` to see diff)

**Interaction Commands in press List:**
```python
press = [
    "a",                    # Press key 'a'
    "shift+tab",            # Key combination
    "enter",                # Return key
    "wait:10",              # Wait N screen updates
    "wait:20",              # Wait longer
]
```

**Snapshot Storage:**
- Snapshots stored in: `tests/snapshot_tests/__snapshots__/`
- File naming: `test_snapshots.ambr` (AMBR format - serialized output)
- Snapshots are version-controlled and part of the git repo
- To update snapshots: `pytest tests/snapshot_tests/ --snapshot-update`

**Snapshot Test Examples:**
```python
# Simple rendering test
def test_grid_layout_basic(snap_compare):
    assert snap_compare(LAYOUT_EXAMPLES_DIR / "grid_layout1.py")

# Widget example with interaction
def test_input_and_focus(snap_compare):
    press = [
        *"Darren",          # Type name
        "tab",              # Move to next field
        *"Burns",           # Type surname
    ]
    assert snap_compare(WIDGET_EXAMPLES_DIR / "input.py", press=press)

# Complex interaction sequence
def test_input_validation(snap_compare):
    """Checking that invalid styling is applied."""
    press = [
        "h",
        "e",
        "l",
        "l",
        "o",
        "tab",
        "1",
        "2",
        "3",
    ]
    assert snap_compare(SNAPSHOT_APPS_DIR / "input_validation.py", press=press)
```

**Working with Snapshots:**
- First run creates snapshot baseline
- Subsequent runs compare against baseline
- If rendering changes: review diff carefully
- Update snapshot if change is intentional: `pytest --snapshot-update`
- If unintentional: fix the code and re-run

## Message and Event Testing

**Testing Message Handlers:**
```python
class MyApp(App):
    pressed_button = None

    def on_button_pressed(self, message: Button.Pressed) -> None:
        self.pressed_button = message.button

async def test_button_pressed():
    app = MyApp()
    async with app.run_test() as pilot:
        button = app.query_one(Button)
        await pilot.click(button)
        assert app.pressed_button is button
```

**Testing with @on Decorator:**
```python
class MyApp(App):
    called_with_id = None

    @on(Button.Pressed, "#quit")
    def handle_quit(self) -> None:
        self.called_with_id = "quit"

async def test_on_decorator():
    app = MyApp()
    async with app.run_test() as pilot:
        await pilot.click("#quit")
        assert app.called_with_id == "quit"
```

## Reactive/Watch Testing

**Testing Reactive Changes:**
```python
class MyApp(App):
    count = reactive(0)

    def watch_count(self, old_value: int, new_value: int) -> None:
        # Handle change
        pass

async def test_reactive_change():
    app = MyApp()
    async with app.run_test() as pilot:
        app.count = 5
        await pilot.pause()  # Let watchers execute
        assert app.count == 5
```

**Testing Computed Reactives:**
```python
class MyApp(App):
    first = reactive("")
    last = reactive("")

    def compute_full_name(self) -> str:
        return f"{self.first} {self.last}"

    full_name = reactive(compute_full_name)

async def test_computed_reactive():
    app = MyApp()
    async with app.run_test() as pilot:
        app.first = "John"
        app.last = "Doe"
        # Computed reactives update on access
        assert app.full_name == "John Doe"
```

## Async Test Patterns

**Basic Async Test:**
```python
async def test_something():
    app = MyApp()
    async with app.run_test():
        # Async operations here
        await pilot.press("enter")
        assert something
```

**Async Context Manager:**
```python
async def test_with_async_operations():
    app = MyApp()
    async with app.run_test() as pilot:
        # pilot is available for 'async with' block scope
        await pilot.click(Button)
        await pilot.pause()
```

**Exception Handling in Async:**
```python
async def test_exception_handling():
    import contextlib

    class FailApp(App):
        def key_p(self):
            raise ValueError("Intentional error")

    app = FailApp()
    with contextlib.suppress(ValueError):
        async with app.run_test() as pilot:
            await pilot.press("p")
    assert app.return_code == 1
```

## Test Markers

**Available Markers:**
```python
@pytest.mark.syntax  # Tests requiring syntax highlighting
```

**Usage:**
```bash
pytest -m "not syntax"  # Skip syntax tests
pytest -m syntax       # Run only syntax tests
```

## Coverage

**Requirements:** Not enforced by default

**View Coverage:**
```bash
pytest --cov=textual tests/ --cov-report=html
```

**Coverage Report:**
- HTML report generated in `htmlcov/`
- View in browser: `open htmlcov/index.html`

## Test Organization Best Practices

**File Size:**
- Keep test files focused (test_app.py ~230 lines, test_animation.py ~150 lines)
- Group related tests in same file
- Use helper functions/classes for shared setup

**Test Independence:**
- Each test should be runnable independently
- Use `async with app.run_test()` to get fresh app instance
- Avoid test interdependencies

**Descriptive Names:**
- Test function names describe what is being tested
- Examples: `test_hover_update_styles()`, `test_animate_height()`, `test_batch_update()`
- Include context: `test_return_code_is_one_after_crash()`

**Docstrings:**
- Include docstring explaining purpose of test
- Especially important for non-obvious tests
- One sentence is sufficient for simple tests

**Fixture Pattern (No Fixtures):**
- Textual tests typically don't use pytest fixtures
- Instead, create test app classes and pass to test functions
- Pilot handles app lifecycle automatically

---

*Testing analysis: 2026-03-24*
