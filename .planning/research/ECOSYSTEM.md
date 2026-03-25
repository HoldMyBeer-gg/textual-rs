# Rust TUI Ecosystem Research

**Project:** textual-rs (Textual-inspired TUI library in Rust)
**Researched:** 2026-03-24
**Overall Confidence:** HIGH (verified against official docs, GitHub, crates.io, and ratatui.rs)

---

## Executive Summary

The Rust TUI ecosystem in early 2026 is mature at the rendering layer but incomplete at the application-framework layer. Ratatui (19.3k GitHub stars, 3M+ monthly downloads) is the unambiguous dominant crate for terminal rendering, and it is actively maintained with a healthy community. However, ratatui is deliberately an **immediate-mode rendering toolkit**, not a framework. It provides no event routing, no component tree, no focus management, no CSS-like styling, no reactive state, and no retained widget hierarchy — exactly the things that make Textual special.

No existing Rust library has achieved Textual-level ergonomics. Several projects have tried (revue, telex-tui, r3bl_tui, tui-realm) but none are production-ready or widely adopted. This creates a clear opportunity: build textual-rs **on top of ratatui** using ratatui-core as the rendering substrate, and provide the higher-level architecture (widget tree, event routing, CSS-like layout, reactive attributes) that is missing from the ecosystem.

Building from scratch on raw crossterm would mean reimplementing buffer diffing, Unicode handling, constraint layouts, and dozens of widgets that ratatui already provides correctly. That is a 12–18 month tax with no user-facing benefit. Build on ratatui.

---

## Question 1: What is ratatui? How mature/popular is it?

### Overview

Ratatui is a community-maintained fork of the abandoned `tui-rs` crate, created in February 2023 when the original author stopped maintaining it. RUSTSEC-2023-0049 officially declared `tui` unmaintained and pointed to ratatui as the replacement.

**Key metrics (as of March 2026):**
- GitHub stars: ~19,300
- Monthly downloads (crates.io): ~3,068,000
- Total dependent crates: 3,055 direct, 3,332 indirect
- Latest version: 0.30.0 (released December 26, 2025)
- License: MIT

### What ratatui provides

- **Rendering engine:** Buffer-based immediate-mode rendering with double-buffering diff — only changed cells are written to the terminal, which minimizes flicker
- **Layout system:** Cassowary constraint solver via `kasuari` crate — supports `Length`, `Percentage`, `Ratio`, `Min`, `Max`, `Fill` constraints with `Flex` alignment modes (Start, End, Center, SpaceBetween, SpaceAround, SpaceEvenly)
- **Built-in widgets:**
  - Content: `Paragraph`, `Block` (borders/titles/padding), `Clear`
  - Selection: `List`, `Table`, `Tabs`
  - Data viz: `Chart`, `BarChart`, `Gauge`, `LineGauge`, `Sparkline`
  - Navigation: `Scrollbar`
  - Drawing: `Canvas` (arbitrary shapes)
  - Calendar: `Monthly` (feature-gated)
  - Text primitives: `String`, `&str`, `Span`, `Line`, `Text`
- **Styling:** `Style` struct with foreground/background color (16 named, 256 ANSI, full RGB true color), text attributes (bold, italic, underline, crossed, dim, etc.)
- **Backend abstraction:** Pluggable backends (crossterm, termion, termwiz) via a trait
- **Unicode:** Correct grapheme cluster handling via `unicode-segmentation`
- **no_std support** (as of 0.30.0): Can run on Cortex-M microcontrollers, ESP32, STM32H7
- **Modular workspace** (as of 0.30.0): `ratatui-core`, `ratatui-widgets`, `ratatui-crossterm`, `ratatui-macros` — widget authors can target `ratatui-core` for API stability

### What ratatui does NOT provide

This is the critical list for textual-rs:

| Missing Feature | Impact |
|---|---|
| Event loop | Developer must integrate crossterm/termion directly |
| Input handling | No keyboard/mouse routing built in |
| Focus management | Must be built or use third-party `rat-focus` / `focusable` |
| Event routing to widgets | No component receives events — manual dispatch only |
| Retained widget tree | No persistent widget hierarchy; widgets are recreated every frame |
| Reactive state | No `reactive` attributes; state changes do not auto-trigger re-render |
| CSS-like styling | No stylesheet language; styling is imperative struct-building |
| Application lifecycle | No `on_mount`, `on_unmount`, async initialization patterns |
| Async framework integration | Not a native async library; async patterns require manual wiring |
| Component communication | No message-passing system between widgets |
| Z-ordering / layers | No layer system (popovers, modals require manual Clear widget use) |
| Font/terminal capability detection | No automatic true color detection |

**Confidence: HIGH** — sourced from official ratatui FAQ, architecture docs, and discussions.

---

## Question 2: Can ratatui support Textual-quality visuals?

**Short answer: Yes**, with caveats about terminal capability detection.

### Colors and Styling

Ratatui supports:
- 16 named ANSI colors (works everywhere)
- 256 ANSI colors (Windows 10+, all modern Unix terminals)
- Full RGB true color / 24-bit (Windows 10+, modern Unix; NOT Terminal.app on older macOS)

**Caveat:** Ratatui does not auto-detect terminal color capabilities. If you render RGB colors into a terminal that only supports 256 colors (like some CI environments), the display is unpredictable. The termwiz backend has fallback logic; crossterm and termion do not. This means textual-rs should implement or wrap a capability detection layer (e.g., `$COLORTERM=truecolor` env var or terminfo-based detection).

### Borders and Block Decorations

Full Unicode border support using box-drawing characters. Multiple border styles (plain, rounded, double, thick, etc.) via `BorderType`. Titles on blocks. Inner/outer padding. This is equivalent to Textual's border capabilities.

### Widgets and Visual Quality

Built-in widgets are functional but sparse compared to Textual's library. Textual ships inputs, text areas, data tables, tree views, select lists, progress bars, switches, radio buttons, checkboxes, markdown viewers, and more. Ratatui ships about 12 built-in widgets plus a growing ecosystem of third-party additions (see Question 6).

The third-party ecosystem bridges much of this gap:
- `ratatui-textarea` — multi-line text editor
- `tui-big-text` — large block-font text
- `tui-tree-widget` — hierarchical tree view
- `tui-logger` — log display widget
- `ratatui-image` — image rendering (sixel, iTerm2, kitty protocols)
- `tachyonfx` — shader-like visual effects
- `rat-widget` — comprehensive input widgets (text-input, date/number input, checkbox, radio, slider, calendar, file dialog, menubar)

**Verdict:** Ratatui's rendering layer can produce Textual-quality visuals. The gap is in higher-level widget richness and the declarative styling system, both of which textual-rs would provide.

---

## Question 3: Alternatives — crossterm, termion, cursive, tui-rs, console, tuikit

### tui-rs

- **Status: ABANDONED.** RUSTSEC-2023-0049 issued. Last commit August 2022.
- Ratatui is the official successor. Do not use.

### cursive

- **GitHub stars:** ~4,800
- **Latest version:** cursive-core 0.4.6 (October 2025)
- **Maintenance:** Active, last commit January 2025, 111 contributors
- **Architecture:** Retained-mode widget tree with event-driven design. Closer to traditional GUI toolkit than ratatui.
- **What it provides:** Dialogs, text views, select lists, custom views, theme/color support, crossterm backend by default
- **Limitations:** Layout system is less expressive than ratatui's constraint solver. Styling is palette-based (not true-color friendly). API feels dated compared to modern reactive patterns. No CSS-like layout. UTF-8 only; no RTL support.
- **Verdict:** More feature-complete as an application framework than ratatui, but architecturally limited. Its retained-mode model is actually closer to what textual-rs wants, but the styling and layout capabilities don't reach Textual's quality. Not a good foundation to build on.

### AppCUI-rs

- **GitHub stars:** ~361
- **Latest version:** v0.4.7 (March 8, 2026 — very recent)
- **Architecture:** Complete CUI framework with its own console engine. Has windows, menus, buttons, checkboxes, radio buttons, list views, tree views, combo boxes, date/time pickers, color pickers, tabs, accordions, dialogs.
- **Platform support:** Windows Console, Windows VT, NCurses, Termios, WebTerminal, CrossTerm backends
- **True color:** 24-bit support
- **Limitations:** 361 stars — very low adoption. Not built on ratatui, so no ecosystem interoperability. Independent roadmap, unclear long-term trajectory.
- **Verdict:** Interesting but too niche. Do not build on top of it.

### tuikit

- **Status:** Maintained (part of skim-rs, 0.6.5 published 2025)
- **Architecture:** Low-level terminal I/O with thread-safety focus, non-fullscreen and fullscreen modes
- **Warning:** "Not stable yet, API might change"
- **Verdict:** Niche library used internally by the `skim` fuzzy finder. Not suitable as a foundation.

### console (crate)

- Utility crate for terminal styling and text manipulation, not a TUI framework
- Used for simpler CLI output, not full-screen TUI apps

### crossterm

- See Question 4 — this is a terminal backend, not a TUI framework.

### termion

- See Question 4 — this is a terminal backend, Unix-only.

---

## Question 4: Terminal backend crates — crossterm vs termion vs termwiz

These three are the backends ratatui supports. They operate below the widget layer, providing raw terminal I/O.

### crossterm

- **GitHub:** crossterm-rs/crossterm
- **Downloads:** 73.7M+ total
- **Latest version:** 0.29.0
- **Platform support:** All UNIX + Windows 7+ (including Windows 10/11)
- **Features:**
  - Cursor movement, positioning, visibility, blinking
  - Styling: 16 named colors, 256 ANSI colors, RGB true color
  - Text attributes: bold, italic, underline, crossed, dim, overlined, reverse
  - Terminal control: alternate screen, raw mode, clear operations
  - Input: keyboard events (including key repeat, modifiers), mouse events (click, scroll, move, drag)
  - Window pixel size detection
  - Multi-threaded (Send + Sync), few dependencies
  - Event stream feature for async usage with tokio
- **Windows key event behavior:** Sends both `KeyEventKind::Press` and `KeyEventKind::Release` on Windows (unlike Unix which only sends Press). Must be handled explicitly.
- **Recommendation: Use crossterm.** It is the de facto standard backend. Ratatui enables it by default. 73.7M downloads is a decisive signal.

### termion

- **Latest version:** 4.0.5 (November 2025)
- **Downloads:** 8.4M total
- **Platform:** Redox, macOS, BSD, Linux — **NO Windows support**
- **Features:** Pure Rust, bindless (no libc), direct TTY access
- **Verdict:** Skip. Excluding Windows is a non-starter for Textual-quality cross-platform support.

### termwiz

- **Repository:** Part of the WezTerm project (`wezterm/termwiz`)
- **Status:** Active development, but subject to "fairly wild sweeping changes"
- **Features:**
  - True color and hyperlink support
  - Windows 10+ true color via both legacy console API and new VT/PTY APIs
  - Sixel and iTerm-style image display
  - Sophisticated escape sequence parser with semantic meaning
  - Surface/delta model for synchronizing screen state
  - Color fallback logic (unlike crossterm, will gracefully downgrade from true color)
- **Best for:** Applications targeting WezTerm users specifically, or that need the color fallback behavior
- **Verdict:** Technically superior in some ways (color fallback, image protocol awareness) but unstable API and tightly coupled to WezTerm's needs. Crossterm is safer for a library. Could be an optional backend for textual-rs.

### Recommendation

**Default to crossterm.** It is stable, cross-platform including Windows, has 73.7M downloads, and is ratatui's default. Provide the crossterm backend as the primary target. Consider termwiz as an optional backend for advanced terminal features (color capability detection, image protocols). Exclude termion — Windows support is mandatory.

---

## Question 5: Is there any existing Rust library that achieves Textual-level ergonomics?

**No.** This gap is the core justification for textual-rs.

### What Textual provides that nothing in Rust does:

1. **CSS-based styling engine** — Load `.tcss` files, cascade styles through the widget tree, selector syntax, pseudoclasses (`:focus`, `:hover`, `:disabled`)
2. **Reactive attributes** — `reactive` decorator auto-triggers re-render and watch callbacks on state changes, like Vue.js `ref()` or React `useState`
3. **DOM-like widget tree** — Widgets are mounted and unmounted, have parent/child relationships, query with CSS selectors (`app.query_one(Button)`)
4. **Async application lifecycle** — `on_mount`, `on_unmount`, workers run without blocking UI thread
5. **CSS layout engine** — Flexbox-like layout, grid layout, docking (sticky headers/footers/sidebars), z-layers
6. **Event bubbling and routing** — Events bubble up the tree; widgets declare handlers with `on_` prefix
7. **Built-in rich widget library** — Input, TextArea, DataTable, Tree, Select, OptionList, RadioSet, Switch, Checkbox, ProgressBar, Markdown viewer, and 30+ more

### Closest existing Rust approaches (all incomplete):

| Project | Stars | Status | Gap |
|---|---|---|---|
| tui-realm | 906 | Active (v3.3.0, Dec 2025) | React/Elm patterns on ratatui but no CSS, no tree query, complex API |
| r3bl_tui | 463 (monorepo) | Active | Reactive patterns, flexbox-like, but idiosyncratic API, limited adoption |
| revue | 1 | Pre-alpha (Dec 2024 last commit) | Vue-style but effectively unmaintained |
| telex-tui | 0 | Early (v0.3.1, Feb 2026) | React hooks pattern, 0 stars, unproven |
| AppCUI-rs | 361 | Active (v0.4.7, Mar 2026) | Complete framework but separate ecosystem, no CSS |

None of these have meaningful adoption, and none provide a Textual-equivalent CSS-based styling + reactive attribute + DOM widget tree combination.

---

## Question 6: ratatui's widget model vs Textual's widget tree approach

This is the most important architectural difference to understand.

### Ratatui: Immediate Mode

```
Each frame:
  app_state → render_fn(frame) → draw all widgets → terminal buffer → diff → stdout
```

- Widgets are **ephemeral**. They are constructed during the render function, render to the buffer, and are dropped. No widget persists between frames.
- Widget trait: `fn render(self, area: Rect, buf: &mut Buffer)` — consumes self
- `StatefulWidget` carries external mutable state through `fn render(self, area: Rect, buf: &mut Buffer, state: &mut S)` — but state is passed in, not owned by the widget
- New `WidgetRef`/`StatefulWidgetRef` traits (v0.26.0, still unstable) allow rendering by reference, enabling stored widget collections via trait objects — a step toward retained mode
- **No event routing.** A widget that handles keyboard input must be wired up by the application developer. The common pattern is a global event channel plus explicit state dispatch.
- **No focus system.** Third-party crates (`rat-focus`, `focusable`, `ratatui-interact`) exist but are not standardized.
- **Layout is pre-computed.** You call `Layout::default().constraints([...]).split(frame.area())` to get a `Vec<Rect>`, then pass rects to each widget's render call. Nested layouts require nested split calls.

### Textual: Retained Mode with Reactive System

```
Persistent widget tree (DOM):
  App
  ├── Header (docked top)
  ├── Container (layout: horizontal)
  │   ├── Sidebar (layout: vertical)
  │   │   ├── Button("Action 1")
  │   │   └── Button("Action 2")
  │   └── Main
  │       ├── DataTable
  │       └── Input
  └── Footer (docked bottom)

Reactive flow:
  reactive attribute changes → watch callback → CSS invalidation → layout pass → render pass → diff → terminal
```

- Widgets are **persistent objects** with identity and lifecycle
- CSS stylesheets describe appearance and layout declaratively
- Reactive attributes (like Vue/React signals) auto-trigger re-render watches
- Events bubble up the tree; `on_button_pressed` anywhere in the tree can be caught at any ancestor
- `app.query_one("#my-input")` or `app.query(Button)` for DOM traversal
- Async workers run in background threads without blocking the UI event loop

### Implication for textual-rs Architecture

textual-rs must provide the **retained layer on top of ratatui's immediate rendering**. The design should be:

```
textual-rs architecture:
  Widget tree (retained, owned objects)
    → CSS-inspired layout engine
    → Reactive attribute system
    → Event routing (bubbling + capturing)
    → Focus management
    → Async lifecycle hooks
    → Frame computation
      → Ratatui render pass (immediate mode, one frame per reactive update)
        → ratatui-core buffer
          → crossterm backend
            → terminal stdout
```

Ratatui handles the bottom half (buffer management, Unicode, diff, terminal I/O). textual-rs provides the top half (everything Textual has that Rust doesn't).

---

## Question 7: Cross-platform terminal handling — Windows 10+, macOS, Linux

### Status Summary

| Platform | crossterm | termion | termwiz | AppCUI-rs |
|---|---|---|---|---|
| Windows 7+ | Yes | No | Yes (legacy + VT) | Yes |
| Windows 10+ (VT) | Yes | No | Yes (preferred) | Yes |
| macOS | Yes | Yes | Yes | Yes |
| Linux | Yes | Yes | Yes | Yes |
| BSD | Limited | Yes | Limited | Unknown |
| Redox | No | Yes | No | No |

### Windows-Specific Gotchas

1. **Key events are doubled.** Crossterm on Windows emits both `KeyEventKind::Press` and `KeyEventKind::Release`. Applications must filter to `Press` only, or handle both explicitly.
2. **True color requires Windows 10 build 14931+.** Older Windows 10 builds and Windows 7/8 fall back to 16 colors. textual-rs must handle this gracefully.
3. **Console vs VT mode.** Windows has two terminal APIs — the old Console API (GetConsoleScreenBufferInfo etc.) and the new Virtual Terminal Processing. Crossterm handles both transparently. Termwiz distinguishes between them and provides both paths.
4. **Unicode rendering.** Windows Terminal (Win10+), Windows 11 terminal all render Unicode well. Legacy cmd.exe and older conhost have issues with wide characters (CJK, emoji). Design for modern Windows Terminal as the baseline.
5. **MSVC toolchain.** No known build issues with crossterm on MSVC. Standard `cargo build` works.

### macOS-Specific Notes

- Terminal.app does NOT support true color on older macOS versions — falls back to nearest 256 color, which can produce glitched or blinking text if not handled. Use termwiz backend or implement `$COLORTERM` detection.
- iTerm2 and modern macOS Terminal fully support true color.
- Mouse event support depends on terminal emulator, not OS.

### Linux

- All major terminal emulators support true color, Unicode, and mouse events
- SSH sessions generally support everything except image protocols
- 256 color fallback works universally

### Recommendation

Target crossterm as the primary backend. It handles the Windows complexity transparently. Add optional capability detection (via `$COLORTERM=truecolor`, `$TERM`, or terminfo query) to gracefully degrade colors. Document Windows Terminal as the minimum recommended Windows terminal.

---

## Ecosystem Map: Crates at a Glance

### Tier 1: Production-Ready (Use These)

| Crate | Version | Stars | Role |
|---|---|---|---|
| `ratatui` | 0.30.0 | 19,300 | Rendering framework — the foundation |
| `ratatui-core` | 0.1.0 | (part of ratatui) | Widget author target for API stability |
| `crossterm` | 0.29.0 | ~3,500 | Terminal backend — use as default |
| `ratatui-macros` | 0.7 | (part of ratatui) | Builder macros for less boilerplate |

### Tier 2: Mature Ecosystem Extensions (Useful)

| Crate | Version | Stars | Role |
|---|---|---|---|
| `ratatui-textarea` | active | moderate | Multi-line text editor widget |
| `tui-tree-widget` | active | moderate | Tree view widget |
| `rat-widget` | active | low | Rich input widgets (forms, dialogs) |
| `rat-focus` | active | low | Focus management for ratatui |
| `tachyonfx` | active | moderate | Shader-like visual effects |
| `ratatui-image` | active | moderate | Image rendering (sixel/kitty) |
| `tui-logger` | active | moderate | Log viewer widget |
| `tui-big-text` | active | moderate | Large block-font text |

### Tier 3: Frameworks (Study But Don't Depend On)

| Crate | Stars | Status | Notes |
|---|---|---|---|
| `tui-realm` | 906 | Active v3.3.0 | React/Elm on ratatui. Good ideas, complex API. |
| `r3bl_tui` | 463 (monorepo) | Active | Reactive+async, idiosyncratic. Independently interesting. |
| `cursive` | 4,800 | Slow | Retained mode, but dated API and limited styling |
| `AppCUI-rs` | 361 | Active v0.4.7 | Complete but separate ecosystem |
| `telex-tui` | 0 | Experimental | React hooks, v0.3.1 Feb 2026 |
| `revue` | 1 | Dead? | Vue-style, last commit Dec 2024 |

### Tier 4: Abandoned (Avoid)

| Crate | Notes |
|---|---|
| `tui` / `tui-rs` | RUSTSEC-2023-0049. Use ratatui instead. |
| `termion` | Unix-only. No Windows. Skip for cross-platform projects. |

---

## Build ON TOP of ratatui vs From Scratch

**Verdict: Build on top of ratatui.** This is not a close call.

### Arguments for building on ratatui

1. **Buffer diffing is hard.** Ratatui's double-buffer diff correctly handles Unicode wide characters (CJK, emoji), ANSI escape sequence batching, and platform quirks. Getting this wrong produces visual glitches. It's already solved.
2. **19,300 stars and 3M downloads/month.** The community, issue tracker, third-party widgets, and ecosystem of ratatui are all assets. textual-rs inherits them.
3. **Constraint layout engine.** The Cassowary solver with `Flex` modes is sophisticated and correct. Reimplementing this is months of work.
4. **Widget catalog.** Ratatui's built-in widgets + third-party ecosystem give textual-rs a starting point. Wrap them in the retained-mode API rather than rewriting from scratch.
5. **no_std support (0.30.0).** If textual-rs also targets embedded or exotic environments, this comes free.
6. **ratatui-core separation.** Target `ratatui-core` for the rendering interface. This isolates textual-rs from widget API churn while still using the battle-tested rendering engine.
7. **The gap being filled is the application layer.** The work for textual-rs is the retained widget tree, reactive attribute system, CSS-like layout, and event routing. None of that conflicts with ratatui; ratatui deliberately left it to higher-level libraries.

### Arguments against (and why they fail)

- "Ratatui's immediate mode conflicts with a retained tree." — No. The retained tree is textual-rs's internal data structure. At render time, textual-rs traverses its own tree, computes the layout, and calls ratatui widget render methods. The caller (textual-rs) is retained; the renderer (ratatui) is immediate. This is a clean interface.
- "ratatui widget trait is not designed for composition." — The new `WidgetRef`/`StatefulWidgetRef` traits (0.26+) address this. And textual-rs can own its own widget types that internally delegate to ratatui primitives.
- "We'd be constrained by ratatui's version churn." — Target `ratatui-core` (0.1.0), which was explicitly separated for API stability. Breaking changes in `ratatui-widgets` won't affect the core rendering contract.

### What to build on ratatui

textual-rs provides:
- `Widget` trait (textual-rs's own, not ratatui's) with `render`, lifecycle hooks (`on_mount`, `on_unmount`), event handlers
- Widget tree (retained DOM structure with parent/child/sibling relationships)
- CSS-inspired style system (parse `.tcss`-like files, cascade through tree, support selectors and pseudoclasses)
- Layout engine wrapping ratatui's `Layout` + extending with docking, z-layers, grid
- Reactive attribute system (signals + effects, auto re-render on mutation)
- Event system (key events from crossterm → routing up/down the widget tree)
- Focus management (tab order, programmatic focus, focus ring)
- Async runtime integration (tokio-based, with UI thread isolation)
- Application lifecycle (`App::run()`, initialization, cleanup, panic hooks)

---

## Critical Pitfalls to Anticipate

1. **Borrow checker and widget tree ownership.** A retained widget tree with parent references is notoriously hard in Rust. Use index-based arenas (like `slotmap`) rather than raw pointer trees. Textual's Python approach (objects with parent refs) translates poorly to Rust without this.

2. **Reactive attributes require careful design.** Rust's ownership model means you cannot have a `reactive` macro that automatically captures `self` like Python does. Design this as an explicit `Signal<T>` type that widgets hold, with explicit subscriptions. Look at how `leptos` and `dioxus` handle signals for inspiration.

3. **CSS parsing is a significant scope item.** Even a subset of CSS (no float, no relative units beyond vh/vw equivalents, no transitions in v1) is weeks of work. Consider using an existing CSS parser crate (`lightningcss`, `cssparser`) as the parse layer and implementing only the subset needed.

4. **Terminal capability detection is under-invested in the ecosystem.** No official, well-maintained crate exists for this. The `termprofile` crate (GitHub: aschey/termprofile) is niche. Plan to implement `$COLORTERM`, `$TERM_PROGRAM`, terminfo-based color depth detection as a first-class concern.

5. **Mouse event coordinate mapping.** In a retained widget tree, hit-testing (which widget owns a given terminal cell) must be tracked during layout. Ratatui gives you `Rect`s per widget — textual-rs must maintain a cell → widget mapping for the current frame. This is doable but must be designed deliberately.

6. **Windows key repeat behavior.** Both Press and Release events arrive on Windows. textual-rs must normalize this for users so cross-platform apps don't need platform branches.

7. **Font/glyph width assumptions.** Emoji and some Unicode symbols are 2 cells wide, but terminals disagree. Ratatui uses `unicode-width` for this. textual-rs should inherit this behavior and document that rendering fidelity depends on the terminal emulator's Unicode version compliance.

---

## Sources

- [Ratatui GitHub](https://github.com/ratatui/ratatui) — star count, version, architecture
- [Ratatui 0.30.0 Highlights](https://ratatui.rs/highlights/v030/) — modular workspace, no_std
- [Ratatui FAQ](https://ratatui.rs/faq/) — explicit list of what ratatui does NOT provide
- [Ratatui Widget Concepts](https://ratatui.rs/concepts/widgets/) — Widget/StatefulWidget trait design
- [Ratatui Layout Docs](https://ratatui.rs/concepts/layout/) — Cassowary solver, Flex modes
- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) — recommended patterns
- [Ratatui Backend Comparison](https://ratatui.rs/concepts/backends/comparison/) — crossterm vs termion vs termwiz
- [lib.rs ratatui](https://lib.rs/crates/ratatui) — download count, dependency count
- [crossterm GitHub](https://github.com/crossterm-rs/crossterm) — 73.7M downloads, 0.29.0, Windows support
- [cursive GitHub](https://github.com/gyscos/cursive) — 4.8k stars, v0.4.6, retained mode design
- [AppCUI-rs GitHub](https://github.com/gdt050579/AppCUI-rs) — 361 stars, v0.4.7 Mar 2026
- [tui-realm GitHub](https://github.com/veeso/tui-realm) — 906 stars, v3.3.0 Dec 2025
- [r3bl-open-core GitHub](https://github.com/r3bl-org/r3bl-open-core) — 463 stars, reactive TUI framework
- [telex-tui GitHub](https://github.com/telex-tui/telex-tui) — 0 stars, v0.3.1 Feb 2026
- [revue GitHub](https://github.com/hawk90/revue) — 1 star, Dec 2024 last commit
- [RUSTSEC-2023-0049](https://rustsec.org/advisories/RUSTSEC-2023-0049.html) — tui-rs abandoned notice
- [Textual Layout Guide](https://textual.textualize.io/guide/layout/) — CSS layout reference
- [7 Things Learned Building Textual](https://www.textualize.io/blog/7-things-ive-learned-building-a-modern-tui-framework/) — Textual architectural lessons
- [BubbleTea vs Ratatui 2026](https://www.glukhov.org/post/2026/02/tui-frameworks-bubbletea-go-vs-ratatui-rust/) — architecture comparison
- [Ratatui Third-Party Widgets](https://ratatui.rs/showcase/third-party-widgets/) — ecosystem extensions
- [Ratatui Async Tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — tokio integration patterns
- [termwiz in WezTerm](https://github.com/wezterm/wezterm/tree/main/termwiz) — termwiz features and status
