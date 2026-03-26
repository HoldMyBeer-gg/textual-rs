# Phase 2: Widget Tree, Layout, and Styling - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 02-widget-tree-layout-and-styling
**Areas discussed:** Widget API & borrow pattern, Layout-to-render bridge, CSS parser scope & property set, Screen stack & focus model

---

## Widget API & Borrow Pattern

### Arena access pattern

| Option | Description | Selected |
|--------|-------------|----------|
| AppContext pattern | Pass &mut AppContext into Widget methods. Widget methods receive (self_id, ctx) instead of &mut self. Avoids borrow conflicts entirely. | ✓ |
| HopSlotMap with temporary remove | Use HopSlotMap, temporarily remove widget from arena during method calls. Simpler trait but error-prone. | |
| Split storage | Keep widget state in SecondaryMap, behavior in primary SlotMap. Clean separation but more indirection. | |

**User's choice:** AppContext pattern
**Notes:** Resolves the SlotMap borrow problem flagged in the roadmap as a required spike.

### Render target

| Option | Description | Selected |
|--------|-------------|----------|
| Direct ratatui Buffer | Widget::render() writes to &mut Buffer directly. Leverages full ratatui ecosystem. | ✓ |
| Strip-based intermediate | Widgets produce Vec<Strip>. Enables caching but adds indirection layer. | |

**User's choice:** Direct ratatui Buffer
**Notes:** Zero-copy, enables use of any ratatui widget inside textual-rs widgets.

### Compose mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Return Vec<Box<dyn Widget>> | Simple, idiomatic. AppContext inserts and wires parent/child. | ✓ |
| Builder pattern with closures | ComposeBuilder with add_child(). More ergonomic nesting but complex API. | |
| You decide | Let Claude pick. | |

**User's choice:** Return Vec<Box<dyn Widget>>

### Dispatch mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| dyn Widget everywhere | All widgets are Box<dyn Widget>. User and built-in widgets identical. | ✓ |
| Enum for built-ins + dyn for custom | Enum wraps 22 built-ins for faster dispatch. More complex. | |

**User's choice:** dyn Widget everywhere

---

## Layout-to-Render Bridge

### Taffy integration

| Option | Description | Selected |
|--------|-------------|----------|
| TaffyBridge sync layer | Dedicated struct mirrors widget tree into TaffyTree. Converts f32 to Rect. | ✓ |
| Inline Taffy in AppContext | TaffyTree lives in AppContext directly. Simpler but tighter coupling. | |
| You decide | Let Claude pick. | |

**User's choice:** TaffyBridge sync layer

### Dock layout

| Option | Description | Selected |
|--------|-------------|----------|
| Emulate with nested flex | dock declarations compile to nested Flexbox containers in TaffyBridge. | ✓ |
| Custom layout pass before Taffy | Two-phase: dock pass carves edges, Taffy handles remaining. | |
| You decide | Let Claude pick. | |

**User's choice:** Emulate with nested flex

### Dirty flags

| Option | Description | Selected |
|--------|-------------|----------|
| Dirty-flag bubbling | Mark ancestors dirty up to Screen. Partial relayout on next render. | ✓ |
| Full relayout every frame | Always recompute. Optimize later if needed. | |
| You decide | Let Claude pick. | |

**User's choice:** Dirty-flag bubbling

---

## CSS Parser Scope & Property Set

### Parser approach

| Option | Description | Selected |
|--------|-------------|----------|
| cssparser tokenizer + hand-rolled | Mozilla's cssparser for tokenization. Hand-built selector/property parsers. | ✓ |
| Full hand-rolled parser | Write tokenizer + parser from scratch. More control, more effort. | |
| You decide | Let Claude pick. | |

**User's choice:** cssparser tokenizer + hand-rolled

### Selector scope

| Option | Description | Selected |
|--------|-------------|----------|
| Match Python Textual's set | Type, class, ID, pseudo-class, descendant, child combinators. No sibling. | ✓ |
| Minimal: type + class + ID only | Start simple. Add combinators later. | |
| You decide | Let Claude pick. | |

**User's choice:** Match Python Textual's set

### Default CSS mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Static CSS string on Widget trait | fn default_css() -> &'static str. Collected at lowest specificity. | ✓ |
| Separate .tcss files per widget | include_str! embedded files. File-based, easier to read. | |
| You decide | Let Claude pick. | |

**User's choice:** Static CSS string on Widget trait

---

## Screen Stack & Focus Model

### Screen/arena relationship

| Option | Description | Selected |
|--------|-------------|----------|
| Screens as special WidgetIds | Screens in shared arena. Vec<WidgetId> stack. Push/pop inserts/removes subtrees. | ✓ |
| Separate arena per screen | Each Screen has own SlotMap. Cleaner isolation, harder state sharing. | |
| You decide | Let Claude pick. | |

**User's choice:** Screens as special WidgetIds

### Focus traversal

| Option | Description | Selected |
|--------|-------------|----------|
| DOM order with can_focus flag | Tab follows depth-first order. can_focus() opt-in. Focus state on AppContext. | ✓ |
| Explicit tab_index | Numeric tab_index per widget. More control, more boilerplate. | |
| You decide | Let Claude pick. | |

**User's choice:** DOM order with can_focus flag

---

## Claude's Discretion

- Widget trait lifecycle method signatures (on_mount, on_unmount)
- AppContext internal data structure choices
- CSS property parsing internals
- TaffyBridge sync algorithm details
- Error types for CSS parse errors
- EventPropagation enum design
- Mouse hit map implementation

## Deferred Ideas

None — discussion stayed within phase scope.
