# Changelog

All notable changes to textual-rs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- GitHub Actions CI (test on Linux, Windows, macOS; clippy + fmt checks)
- CSS unknown-property warnings in debug builds
- CHANGELOG.md and crates.io metadata

## [0.2.0] - 2026-03-26 (v1.1 milestone)

### Added
- **Theme variables** -- `$primary`, `$accent`, `$surface`, `$background`, `$foreground`, `$panel`, `$text`
  with automatic lighten/darken modifiers (e.g. `$primary-lighten-2`)
- **McGugan Box borders** -- one-eighth-block ultra-thin border style (`border: mcgugan-box`)
- **Tall borders** -- half-block border rendering with interior background fill
- **Hatch patterns** -- cross, horizontal, vertical, diagonal background fills
- **Grid layout** -- `display: grid`, `grid-template-columns`, `grid-template-rows`, fractional units
- **Keyline separators** -- `keyline: $primary` for grid child dividers
- **TabbedContent / Tabs** -- tab bar with click-to-switch and dynamic pane composition
- **Collapsible** -- expandable/collapsible content sections
- **Markdown** -- inline Markdown rendering widget (headers, lists, code blocks, emphasis)
- **DataTable** -- sortable, scrollable data table with column definitions
- **Tree view** -- hierarchical tree widget with expand/collapse
- **ListView** -- scrollable list of selectable items
- **Log** -- append-only scrolling log widget
- **ScrollView** -- generic scrollable container with scrollbar gutter
- **CommandPalette** -- fuzzy-search command palette overlay
- **ProgressBar** -- determinate progress indicator
- **Sparkline** -- inline sparkline chart widget
- **Select** -- dropdown selection widget with popup overlay
- **Context menus** -- right-click context menu support
- **Animation / Tweens** -- property animation with easing functions at 30fps render tick
- **TestApp** -- headless testing harness with `Pilot` for simulating input
- **`#[derive(Widget)]`** -- proc macro for automatic `Widget` trait scaffolding
- **Multi-value padding/margin** -- `padding: 1 2` and `padding: 1 2 3 4` shorthand
- **Dock layout** -- `dock: top | bottom | left | right` edge docking
- **CSS cascade** -- specificity-based rule resolution with pseudo-class support (`:focus`, `:hover`, `:disabled`)
- **Worker progress** -- `WorkerProgress<T>` for streaming updates from background tasks

### Fixed
- Horizontal layout fractional units now promote to flex_grow correctly
- Tall border corners preserve parent background instead of black gap
- Multi-value padding/margin was silently ignored
- Select dropdown anchors near widget instead of screen center
- RadioSet shows all options; Checkbox starts unchecked

## [0.1.0] - 2026-03-01 (v1.0 milestone)

### Added
- **Core framework** -- `App`, `Widget` trait, widget arena, event loop
- **CSS engine** -- TCSS parser, selector matching, computed styles
- **Flex layout** -- Taffy-powered flexbox with `layout-direction`, `flex-grow`
- **Reactive signals** -- `Reactive<T>` with automatic re-render on change
- **Key bindings** -- declarative `KeyBinding` slices, `on_action()` dispatch
- **Event bubbling** -- `on_event()` with `EventPropagation::Stop/Continue`
- **Workers** -- `ctx.run_worker()` for async background tasks
- **Built-in widgets** -- Label, Button, Input, Checkbox, Switch, RadioButton/RadioSet, TextArea, Header, Footer, Placeholder, Vertical/Horizontal containers
- **Border styles** -- solid, rounded, heavy, double, ascii
- **Focus management** -- Tab cycling, `can_focus()`, `:focus` pseudo-class
- **Inline CSS** -- `App::with_css()` and `App::with_css_file()`
