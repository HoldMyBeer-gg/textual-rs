# Stack Research

**Domain:** Rust TUI Framework — v1.3 Widget Parity & Screen Stack
**Researched:** 2026-03-26
**Confidence:** HIGH (all decisions derived from existing codebase + verified widget behavior)

---

## Context: What Already Exists

The following are confirmed, validated, and must NOT be changed:

| Technology | Version | Role |
|------------|---------|------|
| ratatui | 0.30 | Rendering backend |
| crossterm | 0.29 | Terminal I/O |
| tokio (current_thread + LocalSet) | 1.x | Async event loop |
| reactive_graph | 0.2.13 | Reactive signals |
| taffy | 0.9.2 | CSS layout engine |
| arboard | 3.6 | Clipboard |
| pulldown-cmark | 0.13 | Markdown parsing (used by Markdown widget) |
| slotmap | 1.0 | Widget arena |
| flume | 0.12 | Channel messaging |
| cssparser / cssparser-color | 0.37 / 0.5 | CSS parsing |
| strsim | 0.11 | Fuzzy match (command palette) |
| unicode-width | 0.2 | Text width |
| insta | 1.46.3 | Snapshot testing |

Screen stack infrastructure is already present in `AppContext` (`screen_stack: Vec<WidgetId>`, `pending_screen_pushes`, `pending_screen_pops`). `push_screen` and `pop_screen` in `widget/tree.rs` are fully implemented and tested. **Screen stack requires zero new dependencies.**

---

## New Dependencies Required for v1.3

### Required: walkdir

**For:** DirectoryTree widget

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| walkdir | 2.x | Recursive filesystem traversal for DirectoryTree | Standard choice by BurntSushi; 291M+ downloads; cross-platform; handles symlinks, depth limits, descriptor caps. std::fs::read_dir alone doesn't give sorted, typed, depth-aware iteration. |

DirectoryTree needs to enumerate directory children on expand (lazy: only one level at a time, not full recursive walk upfront). `walkdir` with `max_depth(1)` per node and `min_depth(1)` is the correct pattern. No filesystem watcher needed — Python Textual only offers a manual `reload()` method.

```toml
walkdir = "2"
```

Minimum supported Rust: 1.60. Compatible with workspace rust-version 1.88.

### Required: serde_json

**For:** Pretty widget

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| serde_json | 1.x | Parse and pretty-print arbitrary JSON/data for Pretty widget | Python Textual's Pretty widget displays arbitrary Python objects as structured, syntax-highlighted output. The Rust equivalent accepts `serde_json::Value` (the universal arbitrary-data type). `to_string_pretty()` gives indented JSON; a thin hand-written tokenizer produces colored ratatui Spans (string=yellow, number=cyan, bool=magenta, key=green, null=red) — no external syntax-highlight crate needed. |

`serde` is not yet a direct dependency, but `serde_json` brings it. The Pretty widget API should accept `serde_json::Value` since that is the natural Rust equivalent of "any Python object."

```toml
serde_json = { version = "1", features = ["preserve_order"] }
```

`preserve_order` keeps dict key ordering predictable in output, matching user expectations.

### No New Dependency: Everything Else

The table below records the investigation outcome for each remaining widget:

| Widget | Decision | Rationale |
|--------|----------|-----------|
| ContentSwitcher | No new dep | Pure logic: tracks `current: Option<String>` (child ID), hides/shows children by mutating CSS display state. Already done with existing CSS + widget tree APIs. |
| Digits | No new dep | Renders each digit as a 3-row × 3-col Unicode block glyph grid. Python Textual uses a hardcoded lookup table of Unicode chars per digit. Implemented as a `static` array of 5×3 character patterns using block drawing chars (U+2580–U+2588). Zero dependency; ~150 lines. |
| Link | No new dep | Opens URL via `std::process::Command::new("open"/"xdg-open"/"start")` dispatched by platform. No external crate needed — same approach as any Rust CLI tool. Detects platform at compile time with `#[cfg(target_os)]`. |
| LoadingIndicator | No new dep | Animates a sequence of braille spinner frames (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) driven by existing `AppEvent::Tick`. The tick timer already fires at 33ms. Widget holds a `Cell<usize>` frame counter, advances on each `Tick` event received via `on_event`. No new timer infrastructure needed. |
| MaskedInput | No new dep | Builds on existing `Input` widget. Mask is a `String` template; each position maps to a character class (digit, letter, hex, any, separator). A hand-written ~80-line mask engine validates keystrokes and auto-inserts separators. The mask table in Python Textual is fully specified in documentation and can be faithfully implemented with a `match` on mask chars. No external parsing crate needed. |
| OptionList | No new dep | Single-select scrollable list. Architecturally identical to existing `ListView` but accepts `Rich`-style content (styled `ratatui::text::Line` values rather than plain strings). Internally uses existing scroll pattern from `Log`/`ListView`. |
| Pretty | serde_json (above) | See serde_json row. |
| RichLog | No new dep | Extension of `Log` widget. `Log` stores `Vec<String>`; RichLog stores `Vec<ratatui::text::Line<'static>>`. The `write()` method accepts either a plain string or a pre-styled `Line`. Syntax-highlight callers can pass pre-styled Lines. Existing ratatui `Line`/`Span`/`Style` types handle all styled output. |
| Rule | No new dep | Draws a horizontal or vertical line with Unicode box-drawing characters (`─`, `━`, `═`, `╌`, etc.) via ratatui `Buffer::set_string`. Direction and line_style are enums; 9 styles map directly to Unicode chars. Pure render widget, ~60 lines. |
| SelectionList | No new dep | Extends OptionList (which extends the pattern from ListView). Adds per-item `selected: bool` state and checkbox rendering using `☐`/`☑` chars already used by existing `Checkbox` widget. Multi-select state fits in a `Vec<bool>` or `HashSet<usize>`. |
| Static | No new dep | Thin wrapper over existing `Label` widget that additionally supports `ratatui::text::Text` (multi-line, pre-styled). Python Textual's Static is the base class for simple text display. Already covered by Label; Static can either alias Label or be a trivial new struct. |
| Toast | No new dep | Overlay widget (uses existing `is_overlay()` + `active_overlay` in AppContext). A `ToastRack` is a `Vec<(String, Severity, Instant, Duration)>` stored as an overlay or docked widget. Auto-dismiss uses existing `AppEvent::Tick` to check elapsed time since push. Severity maps to three ratatui color styles. |
| DirectoryTree | walkdir (above) | See walkdir row. |
| Screen Stack | No new dep | Already fully implemented: `push_screen`, `pop_screen`, `push_screen_deferred`, `pop_screen_deferred` all exist in `widget/tree.rs` and `widget/context.rs`. The v1.3 work is behavioral (modal overlay semantics, Escape to pop, focus restore on pop) — not structural. |

---

## Recommended Stack Additions

```toml
# Add to crates/textual-rs/Cargo.toml [dependencies]
walkdir = "2"
serde_json = { version = "1", features = ["preserve_order"] }
```

That is the complete list. Two crates.

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| walkdir 2.x | std::fs::read_dir | std gives only one-level, unsorted, no built-in depth limit or symlink protection |
| walkdir 2.x | jwalk | jwalk is parallel; DirectoryTree is interactive/single-threaded — no benefit, extra dep |
| serde_json Value for Pretty | syntect + ansi-to-tui | syntect adds ~4MB of bundled syntax definitions; hand-written JSON tokenizer is 80 lines and sufficient for the Pretty widget's single use case |
| serde_json Value for Pretty | colored_json | colored_json outputs ANSI escape strings, not ratatui Span objects — requires a second conversion layer; simpler to tokenize directly |
| Hand-written mask engine for MaskedInput | nom / regex | Mask grammar is a finite set of ~10 characters; a full parser combinator is overkill |
| AppEvent::Tick for LoadingIndicator | New timer channel | Tick already fires at 33ms (30fps); LoadingIndicator needs ~100ms updates; frame-skip with a counter is trivial |
| Hand-written Digits font | figlet-rs / toilet-rs | Digits only needs 0-9, A-F, and 7 symbols; a static lookup table is 50 lines vs. adding a font-rendering crate |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| notify (filesystem watcher crate) | DirectoryTree only needs `reload()`, not live watching. Adds OS-level watchers with complex lifecycle management for zero user-visible benefit in v1.3 | walkdir per expand + manual reload |
| syntect | ~4MB embedded syntax definitions; only needed for JSON highlighting in Pretty widget which is fully solved by a 6-color hand-written tokenizer | Hand-written JSON tokenizer using serde_json Value tree |
| crossterm::open_url / webbrowser crate | Platform-specific URL opening is 5 lines with std::process::Command and cfg macros | std::process::Command with #[cfg(target_os)] |
| figlet-rs | Large embedded font data for full ASCII art; Digits only needs a 17-char lookup table (0-9, A-F, +, -, ^, :, ×) | Static array of 5×3 Unicode block patterns |

---

## Screen Stack: Architectural Notes (No New Deps)

The infrastructure exists. What v1.3 must add behaviorally:

1. **Focus save/restore on push/pop** — When `push_screen` is called, save `ctx.focused_widget` per screen layer. On `pop_screen`, restore the saved focus for the now-top screen. Current `pop_screen` clears focus entirely.
2. **Escape key binding** — Modal screens (dialogs) need Escape mapped to `ctx.pop_screen_deferred()`. This is a widget-level concern, not a framework concern — each modal screen widget declares an Escape key binding.
3. **Screen-scoped rendering** — The render loop already renders only `ctx.screen_stack.last()`. No change needed.
4. **Mouse capture on modal screens** — Modal screens should push `mouse_capture: true` on mount and pop on unmount. This uses existing `MouseCaptureStack`.

No new fields in `AppContext` required. No new crates required.

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| walkdir 2.x | ratatui 0.30, tokio 1.x | No shared deps; fully independent |
| serde_json 1.x | Rust 1.88 (workspace) | serde_json 1.x MSRV is 1.56; compatible |

---

## Sources

- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/digits/ — confirms 3×3 Unicode grid rendering, no font crate
- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/masked_input/ — full mask character table
- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/rich_log/ — confirms RichLog = Log + styled lines
- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/toast/ — confirms ToastRack overlay pattern
- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/directory_tree/ — confirms manual reload() only, no live watcher
- Official Textual docs (fetched 2026-03-26): https://textual.textualize.io/widgets/pretty/ — arbitrary object display, structured colorized output
- walkdir crate (BurntSushi): https://docs.rs/walkdir — 291M downloads, MSRV 1.60, cross-platform
- serde_json crate: https://docs.rs/serde_json — `to_string_pretty` + Value tree for arbitrary data
- Codebase audit (2026-03-26): `widget/context.rs`, `widget/tree.rs` — screen stack already fully implemented; `event/timer.rs` — Tick at 33ms already active; `widget/log.rs` — Vec<String> scroll pattern to extend for RichLog

---

*Stack research for: textual-rs v1.3 Widget Parity & Screen Stack*
*Researched: 2026-03-26*
