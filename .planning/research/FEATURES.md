# Feature Research: 13 Missing Widgets + Screen Stack

**Domain:** Rust TUI widget library (textual-rs) — parity milestone with Python Textual
**Researched:** 2026-03-26
**Confidence:** HIGH (sourced directly from https://textual.textualize.io/widgets/)

---

## Widget Specifications

Each section below is derived from the Python Textual official documentation and serves as the authoritative spec for the Rust implementation.

---

### 1. Static

**What it does:** Displays fixed content — strings with markup, Rich renderables, or Content objects. Also serves as the base class for more complex widgets (Label inherits from it).

**Behavior:**
- No reactive attributes, messages, or key bindings of its own
- `update(content)` method modifies content and optionally triggers layout recalculation
- Can expand or shrink to fill available space
- Focusable and acts as a container

**Key properties:**
- `content` — the original input as provided
- `visual` — what actually renders (may be transformed from `content`)
- Constructor params: `renderable`, `expand`, `shrink`, `markup`, `name`, `id`, `classes`

**Visual appearance:** Renders inline text/markup within its bounds. No border by default. Pure content display.

**Events:** None specific to Static.

**Implementation notes:** This is foundational — Link inherits from it. Should be the simplest widget to implement since we already have Label (which is similar). May already be partially implemented under Label.

---

### 2. ContentSwitcher

**What it does:** A container that shows exactly one child widget at a time, hiding all others. Equivalent to a tab panel body — switches which child is visible without unmounting/remounting them.

**Behavior:**
- All switchable children must have unique IDs; children without IDs are hidden and ignored
- Only one child visible at any time; non-visible children remain mounted (state preserved)
- `current` reactive attribute holds the ID of the currently visible child (string or None)
- Setting `current` to a child's ID switches visibility immediately
- `add_content(widget)` dynamically adds new child widgets
- `visible_content` property provides direct reference to the active widget

**Key properties:**
- `current: reactive[str | None]` — ID of active child
- `initial: str | None` — which child to show on startup

**Visual appearance:** Renders the visible child widget in full. No chrome of its own — just a container that delegates rendering to whichever child is active.

**Events:** None specific to ContentSwitcher itself; watch `current` reactively.

**Implementation notes:** Medium complexity. Requires that widget show/hide works correctly with layout. Typically paired with Tabs or custom buttons to drive `current`.

---

### 3. Digits

**What it does:** Displays numbers and a limited character set as large multi-line block characters, creating a digital clock / scoreboard aesthetic using Unicode box-drawing characters in a 3x3 grid layout.

**Behavior:**
- Constructor: `Digits(value: str)`
- `update(value: str)` method to change displayed content after mounting
- Respects `text-align` CSS rule for positioning within its container
- Focusable and acts as a container

**Supported characters:** `0–9`, `A–F` (hex), `+`, `-`, `^`, `:`, `×`. Unsupported characters fall back to regular font size.

**Visual appearance:** Each character renders as a tall block glyph composed of stacked Unicode characters — like a seven-segment display. Visually prominent; useful for clocks, countdowns, scoreboards.

**Events:** None.

**Implementation notes:** Medium complexity. Requires a lookup table of Unicode art for each supported character. Each glyph is 3 cells wide × multiple lines tall. Must handle `text-align`.

---

### 4. DirectoryTree

**What it does:** A specialized tree widget for navigating the filesystem. Shows directories and files in a hierarchical expandable tree. Inherits from Tree.

**Behavior:**
- Directories are expandable nodes; files are leaf nodes
- Clicking or selecting a node emits events
- `filter_paths(paths)` method can be overridden to exclude items (e.g., hidden files)
- Lazy loading of directory contents on expansion

**Key properties:**
- `path: str | Path` — root directory to display
- `show_root: bool` — whether to show the root node (default: true)
- `show_guides: bool` — indentation guide lines (default: true)
- `guide_depth: int` — spacing between hierarchy levels (default: 4)

**Visual appearance:** Tree with 📁 folder icons and 📄 file icons. Guide lines connect parent-child nodes. Component classes for styling: files, folders, hidden items, file extensions.

**Events:**
- `DirectoryTree.DirectorySelected` — posted when a directory node is selected; provides `node` and `path`
- `DirectoryTree.FileSelected` — posted when a file node is selected; provides `node` and `path`

**Implementation notes:** HIGH complexity. Requires async filesystem access (reading directory contents lazily), inherits from Tree widget (already built), and needs icon rendering. The `filter_paths` hook is important for usability. Platform path handling (Windows vs Unix) is a concern in Rust.

---

### 5. Link

**What it does:** A clickable text element that opens a URL in the system default browser. Inherits from Static.

**Behavior:**
- Clicking or pressing Enter opens `url` in the default browser
- If no `url` is set, falls back to using `text` as the URL
- Focusable; supports tooltip
- Can be disabled to prevent interaction

**Key properties:**
- `text: reactive[str]` — displayed link text (default: empty)
- `url: reactive[str]` — destination URL (default: empty)

**Visual appearance:** Renders as styled text (typically underlined or colored per app theme). Looks like a hyperlink. Inherits Static rendering.

**Events:** None emitted; opens browser as side effect of click/Enter.

**Keybindings:** `Enter` — open URL.

**Implementation notes:** LOW complexity given Static is the base. The browser-open call requires `open::that(url)` or equivalent in Rust (the `open` crate). Added in Python Textual v0.84.0.

---

### 6. LoadingIndicator

**What it does:** Displays animated pulsating dots to indicate data is being loaded. A full-widget-area spinner.

**Behavior:**
- Renders animated dots (● ● ● ●) that pulse
- Prevents input events from bubbling (effectively blocks interaction while loading)
- Integrates with the `loading` reactive property on any widget — setting `widget.loading = true` automatically overlays a LoadingIndicator on that widget
- Focusable and acts as a container

**Key properties:**
- `color` — CSS color property controls dot color
- No custom reactive attributes

**Visual appearance:** Full-area animated pulsating dots, horizontally centered. The animation cycles through the dots to indicate ongoing activity.

**Events:** None.

**Implementation notes:** MEDIUM complexity. Requires animation (timer-driven state changes). The `loading` property integration on base Widget is the most complex part — it means the Widget base class needs a `loading: bool` reactive that swaps in a LoadingIndicator overlay. Added in Python Textual v0.15.0.

---

### 7. MaskedInput

**What it does:** A text input field with a template mask that restricts and guides user input into a specific format (credit cards, dates, phone numbers, etc.). Inherits from Input.

**Behavior:**
- Constructor: `MaskedInput(template: str)`
- Template mask characters define what's allowed at each position
- Separator characters (non-mask chars) are auto-inserted as the user types
- Cursor skips over separator positions automatically
- Validates against the template pattern (acts as implicit validator)
- Supports case conversion operators in the mask
- Compatible with additional `validators` beyond template validation

**Template mask format:**
| Char | Meaning |
|------|---------|
| `9` | Digit, required |
| `0` | Digit, optional |
| `A` | Letter, required |
| `a` | Letter, optional |
| `N` | Alphanumeric, required |
| `n` | Alphanumeric, optional |
| `H` | Hex digit, required |
| `h` | Hex digit, optional |
| `>` | Convert to uppercase |
| `<` | Convert to lowercase |
| `!` | Disable case conversion |
| `;c` | Suffix sets placeholder character to `c` |

**Example:** `"9999-9999-9999-9999;0"` — credit card format with auto-inserted dashes, `0` as placeholder.

**Visual appearance:** Looks like a standard Input field but shows the mask template as placeholder, with separators fixed in place as the user types.

**Events:** Inherits all Input events (`Changed`, `Submitted`).

**Implementation notes:** HIGH complexity. Requires significant extension of the existing Input widget's cursor movement and character insertion logic. Separator auto-insertion and cursor skip logic are tricky edge cases.

---

### 8. OptionList

**What it does:** A scrollable, keyboard-navigable list of selectable options with Rich rendering support. Single-selection — pressing Enter fires a selection event.

**Behavior:**
- Navigate with arrow keys, Page Up/Down, Home/End
- Enter selects the highlighted option
- Options can be disabled (shown grayed out, non-navigable)
- `None` values in the options list insert visual separator lines
- Options can be any Rich renderable (not just plain strings) — supports tables, styled text, multi-line content
- Methods: `add_option()`, `remove_option()`, `enable_option()`, `disable_option()`, `get_option_at_index()`, `get_option(id)`

**Key properties:**
- `highlighted: reactive[int | None]` — index of currently highlighted option
- `option_count: int` — total number of options (read-only)
- `options: Sequence[Option]` — read-only sequence of all options

**Events:**
- `OptionList.OptionHighlighted` — fires when highlighted option changes; provides `option` and `option_index`
- `OptionList.OptionSelected` — fires when option is selected (Enter); provides `option` and `option_index`

**Component classes:** normal, disabled, highlighted, hover states for fine-grained CSS styling.

**Visual appearance:** Vertical list. Highlighted item has a distinct background. Disabled items appear dimmed. Separator lines appear as horizontal rules between groups. Compact mode reduces padding.

**Implementation notes:** MEDIUM complexity. This is the foundation for SelectionList — build OptionList first. Rich renderable options (not just strings) are the key differentiator from ListView.

---

### 9. Pretty

**What it does:** Pretty-prints any object with syntax highlighting and hierarchical indentation. Equivalent to `pprint` but as a visual widget with color coding.

**Behavior:**
- Constructor: `Pretty(object: Any)`
- `update(object)` method to change displayed object
- Scrollable for large objects
- Focusable and acts as a container

**Key properties:**
- `object: Any` — the Python/Rust object to display

**Visual appearance:** Color-coded display: dict keys in one color, string values in another, numbers in another, with consistent indentation for nested structures. Resembles a syntax-highlighted REPL output.

**Events:** None.

**Implementation notes:** MEDIUM complexity. In Rust, the equivalent is rendering the `Debug` or `Display` representation of a type with syntax highlighting applied. Could use `syntect` for highlighting. The key challenge is determining how to accept "any object" in Rust's type system — likely `Box<dyn Debug>` or a string input with highlighting applied.

---

### 10. RichLog

**What it does:** A scrollable log widget that accepts Rich renderables (styled text, tables, syntax-highlighted code) appended in real-time. More powerful than the existing `Log` widget which is text-only.

**Behavior:**
- `write(content)` appends a string or Rich renderable to the log
- `clear()` removes all content
- Auto-scrolls to bottom on new content (configurable)
- Optional `max_lines` parameter caps total line count to prevent unbounded memory growth
- Focusable; supports keyboard navigation within the log

**Key properties:**
- `highlight: bool` — enable automatic syntax highlighting
- `markup: bool` — enable Rich markup parsing
- `wrap: bool` — enable word wrapping
- `min_width: int` — rendering width (default: 78)
- `auto_scroll: bool` — scroll to bottom on new content (default: true)
- `max_lines: int | None` — maximum stored lines (default: None = unlimited)

**Visual appearance:** Scrollable area with Rich-formatted content. Can show syntax-highlighted code blocks, formatted tables, colored text, all interleaved in a single scroll buffer.

**Events:** None.

**Implementation notes:** MEDIUM complexity. Differs from existing `Log` in that it must accept pre-formatted renderables, not just plain text strings. Requires the rendering pipeline to handle mixed content types per line. The `max_lines` eviction logic is important for long-running processes.

---

### 11. Rule

**What it does:** A visual separator line, analogous to HTML `<hr>`. Renders a horizontal or vertical line to divide content sections.

**Behavior:**
- Orientation: `horizontal` (default) or `vertical`
- Convenience constructors: `Rule.horizontal()`, `Rule.vertical()`
- Both `orientation` and `line_style` are reactive — can be changed programmatically
- Focusable and acts as a container

**Line styles:** `solid`, `heavy`, `thick`, `dashed`, `double`, `ascii`, `blank`, `hidden`, `none`

**Visual appearance:** Horizontal: a continuous line spanning the full container width using Unicode box-drawing or ASCII characters per the selected style. Vertical: a vertical line spanning full container height.

**Events:** None.

**Implementation notes:** LOW complexity. Pure rendering widget. Mostly a lookup table of Unicode characters per style, rendered across the widget's width or height.

---

### 12. SelectionList

**What it does:** A scrollable list of options where multiple items can be selected simultaneously. Each item has a checkbox-style indicator. Inherits from OptionList.

**Behavior:**
- Navigate with arrow keys, Page Up/Down, Home/End
- Space bar toggles the highlighted item's selected state
- Highlight and selection are independent — highlighting moves with keyboard, selections persist
- Methods: `add_option()`, `clear_options()`, `select_all()`, `deselect_all()`, `toggle_all()`
- Accepts selections as tuples `(prompt, value)` or `(prompt, value, initial_state)` or as `Selection` objects
- Supports generic typing for value type (e.g., `SelectionList[int]`)

**Key properties:**
- `highlighted: reactive[int | None]` — index of currently highlighted item
- `selected: list[SelectionType]` — list of values for all selected items

**Events:**
- `SelectionList.SelectionHighlighted` — fires when highlighted item changes
- `SelectionList.SelectionToggled` — fires when an item is toggled; provides the selection
- `SelectionList.SelectedChanged` — fires when the set of selected items changes

**Component classes:** default, selected, highlighted, selected+highlighted combined states.

**Visual appearance:** Vertical list with `[X]` / `[ ]` checkbox indicators on the left of each item. Highlighted item has distinct background. Compact mode available.

**Implementation notes:** MEDIUM complexity. Inherits from OptionList — build OptionList first. The main additions are the checkbox rendering and the toggle/select tracking per item.

---

### 13. Toast

**What it does:** Displays temporary notification messages that auto-dismiss after a timeout. Three severity levels: information, warning, error. Accessed via `App.notify()` rather than direct instantiation.

**Behavior:**
- Created and managed via `App.notify(message, title=..., severity=..., timeout=...)` — not instantiated directly
- Auto-dismisses after `timeout` seconds
- Severity levels: `"information"` (default), `"warning"`, `"error"`
- Accepts Rich markup in message content
- Managed by a `ToastRack` container that can be positioned anywhere on screen (typically top-right or bottom-right)

**Key properties (on `notify()` call):**
- `message: str` — notification body (supports markup)
- `title: str` — optional header text
- `severity: str` — `"information"` | `"warning"` | `"error"`
- `timeout: float` — seconds before auto-dismiss

**Component classes:** `toast--title` for targeting the title element separately. CSS selectors: `.Toast`, `.Toast.-information`, `.Toast.-warning`, `.Toast.-error`.

**Visual appearance:** Floating notification card with title + message. Color-coded by severity (blue for info, yellow for warning, red for error by default). Overlays the current screen without blocking input. Multiple toasts stack in the ToastRack.

**Events:** None emitted; purely output.

**Implementation notes:** HIGH complexity. Requires a `ToastRack` overlay layer that sits above all other content. Needs timer-based auto-dismiss. The `App.notify()` API must be threadsafe (callable from workers). Multiple simultaneous toasts must stack properly. CSS severity classes must be implemented.

---

### 14. Screen Stack

**What it does:** The navigation system for multi-screen applications. Textual maintains a stack of Screen objects; only the topmost is rendered and receives input.

**Behavior:**

**Core stack operations:**
- `App.push_screen(screen)` — pushes a new screen onto the stack; previous screen becomes hidden but stays mounted
- `App.pop_screen()` — removes topmost screen, reveals and reactivates the one beneath; raises `ScreenStackError` if only one screen remains
- `App.switch_screen(screen)` — replaces the topmost screen (removes it, pushes new one in one operation)

**Data passing between screens:**
- `Screen.dismiss(result)` — pops the screen and triggers the callback registered during `push_screen()`
- `push_screen(screen, callback=fn)` — callback receives the dismissed result
- `push_screen_wait(screen)` — async variant; can `await` the screen's result directly (used inside workers)

**Modal screens:**
- `ModalScreen` subclass — automatically blocks key bindings from the underlying app
- Applies semi-transparent background overlay, letting the screen beneath show through visually
- Creates true modal UX: underlying screen visible but non-interactive

**Modes system (complex apps):**
- Multiple independent screen stacks, each called a "mode"
- `switch_mode(mode_name)` switches the entire active mode
- Each mode maintains its own stack independently
- Used for dashboard/settings/help style contexts with separate navigation histories

**Screen lifecycle events:**
- `on_mount` — screen initialized and ready
- `on_screen_suspend` — fired when screen is pushed below another
- `on_screen_resume` — fired when screen becomes topmost again
- `on_show` / `on_hide` — widget-level visibility within screens

**Key invariants:**
- Stack always has at least one screen (the default screen)
- Installed screens (registered by name) are not destroyed on pop
- Anonymous screen instances are destroyed when popped

**Implementation notes:** HIGH complexity. This is infrastructure, not a widget. Requires:
1. A `ScreenStack` data structure in the App
2. Screen suspend/resume lifecycle events
3. ModalScreen with overlay rendering and input blocking
4. The dismiss/callback pattern for data passing
5. The `push_screen_wait` async variant
6. Optional: Modes system (independent stacks)

The Modes system can be deferred to a later phase — core push/pop/switch + ModalScreen is the MVP.

---

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Static | Base widget everyone uses directly or indirectly | LOW | May already exist under Label |
| Rule | Fundamental layout separator | LOW | Just Unicode chars |
| OptionList | Standard scrollable selection list | MEDIUM | Foundation for SelectionList |
| SelectionList | Multi-select is a core TUI pattern | MEDIUM | Needs OptionList first |
| ContentSwitcher | Tab panels need a body switcher | MEDIUM | Pairs with existing Tabs |
| RichLog | Real-time log output is table stakes for TUI apps | MEDIUM | Upgrade from existing Log |
| Toast / App.notify() | Notification system expected in any serious TUI framework | HIGH | Needs ToastRack overlay |
| Screen Stack | Multi-screen navigation is fundamental | HIGH | Infrastructure, not a widget |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Digits | Visually striking clock/counter display | MEDIUM | Unique to Textual ecosystem |
| DirectoryTree | Built-in filesystem browser in stdlib | HIGH | High utility for file-picker apps |
| MaskedInput | Structured data entry without custom validators | HIGH | Rare in TUI frameworks |
| Pretty | Debug/inspect objects in the UI | MEDIUM | Developer tooling differentiator |
| LoadingIndicator | First-class async loading UX | MEDIUM | The `widget.loading` overlay integration is the key feature |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Modes system (full) | Independent screen stacks per section | Significant complexity multiplier; most apps don't need it | Implement basic Screen Stack first; add Modes in a later phase |
| DirectoryTree with write ops | Users want rename/delete in-widget | Scope creep; read-only tree is the spec | Keep it read-only, emit events for callers to handle mutations |
| MaskedInput regex masks | More powerful than template masks | Regex masks create confusing UX and are not in the Textual spec | Stick to the template mask format as specced |

---

## Feature Dependencies

```
OptionList
    └──required by──> SelectionList

Static
    └──required by──> Link

Tree (already built)
    └──required by──> DirectoryTree

Input (already built)
    └──required by──> MaskedInput

Log (already built)
    └──enhanced by──> RichLog (separate widget, not a replacement)

Screen (already built)
    └──required by──> Screen Stack push/pop/switch
    └──required by──> ModalScreen
    └──required by──> Toast / ToastRack (overlay screen layer)

Tabs (already built)
    └──enhanced by──> ContentSwitcher (provides the panel body)

App.notify()
    └──requires──> Toast + ToastRack
```

### Dependency Notes

- **OptionList before SelectionList:** SelectionList inherits directly from OptionList in Python Textual. Build and stabilize OptionList first.
- **Static before Link:** Link inherits from Static. If Static is essentially Label, Link can be built on top immediately.
- **Screen Stack before Toast:** Toast requires an overlay layer (ToastRack) that sits above all screens. The screen/layer system must be settled first.
- **Tree before DirectoryTree:** DirectoryTree specializes Tree. The existing Tree widget is the foundation.

---

## MVP Definition

### Phase priority ordering

**Phase 1 — Foundation widgets (LOW complexity, high value):**
- [ ] Static — base class cleanup/formalization
- [ ] Rule — pure rendering, no state
- [ ] Link — trivial extension of Static

**Phase 2 — List and selection widgets:**
- [ ] OptionList — needed as foundation
- [ ] SelectionList — builds on OptionList
- [ ] ContentSwitcher — pairs with existing Tabs

**Phase 3 — Enhanced display widgets:**
- [ ] RichLog — upgrades existing Log
- [ ] Pretty — developer tooling
- [ ] Digits — visual impact widget
- [ ] LoadingIndicator — async UX (include `widget.loading` integration)

**Phase 4 — Complex widgets:**
- [ ] MaskedInput — significant Input extension
- [ ] DirectoryTree — filesystem + async + Tree extension
- [ ] Toast + App.notify() — overlay system

**Phase 5 — Screen Stack:**
- [ ] push_screen / pop_screen / switch_screen
- [ ] ModalScreen with overlay + input blocking
- [ ] dismiss() / callback / push_screen_wait()
- [ ] Screen lifecycle events (suspend/resume)
- [ ] Modes system (defer if needed)

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Static | HIGH | LOW | P1 |
| Rule | MEDIUM | LOW | P1 |
| OptionList | HIGH | MEDIUM | P1 |
| SelectionList | HIGH | MEDIUM | P1 |
| ContentSwitcher | HIGH | MEDIUM | P1 |
| Screen Stack | HIGH | HIGH | P1 |
| RichLog | HIGH | MEDIUM | P2 |
| Link | MEDIUM | LOW | P2 |
| Toast | HIGH | HIGH | P2 |
| LoadingIndicator | MEDIUM | MEDIUM | P2 |
| Digits | MEDIUM | MEDIUM | P3 |
| Pretty | MEDIUM | MEDIUM | P3 |
| DirectoryTree | HIGH | HIGH | P3 |
| MaskedInput | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have — core parity with Python Textual
- P2: Should have — important for real applications
- P3: Nice to have — completes full parity

---

## Sources

- https://textual.textualize.io/widgets/static/ — Static widget
- https://textual.textualize.io/widgets/content_switcher/ — ContentSwitcher widget
- https://textual.textualize.io/widgets/digits/ — Digits widget
- https://textual.textualize.io/widgets/directory_tree/ — DirectoryTree widget
- https://textual.textualize.io/widgets/link/ — Link widget
- https://textual.textualize.io/widgets/loading_indicator/ — LoadingIndicator widget
- https://textual.textualize.io/widgets/masked_input/ — MaskedInput widget
- https://textual.textualize.io/widgets/option_list/ — OptionList widget
- https://textual.textualize.io/widgets/pretty/ — Pretty widget
- https://textual.textualize.io/widgets/rich_log/ — RichLog widget
- https://textual.textualize.io/widgets/rule/ — Rule widget
- https://textual.textualize.io/widgets/selection_list/ — SelectionList widget
- https://textual.textualize.io/widgets/toast/ — Toast widget
- https://textual.textualize.io/guide/screens/ — Screen Stack system

---
*Feature research for: textual-rs missing widget parity milestone*
*Researched: 2026-03-26*
