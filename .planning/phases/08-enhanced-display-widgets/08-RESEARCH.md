# Phase 8: Enhanced Display Widgets - Research

**Researched:** 2026-03-27
**Domain:** Rust TUI widget implementation — scrolling styled log, per-widget loading overlay, standalone spinner
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIDGET-09 | User can view scrolling rich-text log output with RichLog (styled Lines, not plain strings) | Existing `Log` widget is direct template; `ratatui::text::Line` is the styled line type; max_lines eviction is a vec-deque or drain-head operation |
| WIDGET-10 | User can display a loading spinner on any widget via `widget.loading = true` | Research resolves the scope question: per-widget `loading` field on struct + render-time overlay draw is the correct approach; `active_overlay` cannot be used |
</phase_requirements>

---

## Summary

Phase 8 adds two display widgets: `RichLog` (styled scrolling log) and a loading spinner system (`LoadingIndicator` standalone + `widget.loading` overlay). Both build entirely on existing infrastructure — no new dependencies are needed.

**RichLog** is a direct evolution of the existing `Log` widget. The only structural difference is that `lines` holds `ratatui::text::Line<'static>` (styled spans) instead of `String`. All scroll mechanics (auto-scroll, key bindings, sub-cell scrollbar) are copy-adapted from `Log`. The `max_lines` parameter adds a capacity cap: when a new line is pushed and `len > max_lines`, drain the head of the vec by one (or use `VecDeque` for O(1) eviction). The existing `Log` renders lines via `buf.set_string()`; `RichLog` renders via `buf.set_line()` (ratatui's native styled-line painter).

**LoadingIndicator** and `widget.loading` share the same spinner render logic. The critical scope decision (flagged in STATE.md research flags) is resolved below: `widget.loading = true` is NOT a viral base-class property (that is WIDGET-F03, marked Future). For this phase, `loading` is a field on each widget struct that chooses to support it — but rather than requiring every widget to add a field, the planner should use a **render-time overlay approach**: the `render_widget_tree` function already has a post-render step for `active_overlay`. A per-widget loading state map on `AppContext` (a `SecondaryMap<WidgetId, bool>`) lets any widget's loading state be set without touching the Widget trait. During `render_widget_tree`, after calling `widget.render()` but before moving to the next widget, check the map and draw the spinner overlay on top of that widget's rect. This avoids `active_overlay` (single-instance) and avoids adding `loading: bool` to every widget struct.

**Primary recommendation:** Add `loading_widgets: SecondaryMap<WidgetId, bool>` to `AppContext`; expose `ctx.set_loading(id, bool)` as the API for `widget.loading = true`; draw the spinner frame in `render_widget_tree` after each widget's render if its ID is in the map. `LoadingIndicator` standalone widget simply calls the shared spinner draw function directly.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 (already in use) | `ratatui::text::Line`, `Span`, `buf.set_line()` | Already the rendering foundation |
| ratatui::text::Line | (part of ratatui) | Styled-span container for RichLog lines | Native ratatui type, no conversion needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::collections::VecDeque | std | O(1) front eviction for max_lines | Use if max_lines is set and eviction frequency is high |
| std::cell::Cell | std | tick counter for spinner frame index | Same pattern as ProgressBar's `tick: Cell<u8>` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `SecondaryMap<WidgetId, bool>` on AppContext for loading state | `loading: Reactive<bool>` field on each widget | Per-field approach requires every widget to opt in; SecondaryMap approach is zero-cost for widgets that never use loading |
| `VecDeque` for RichLog lines | `Vec` with `drain(0..1)` | Vec drain is O(n) but acceptable for typical log sizes (<10k lines); VecDeque is O(1) but adds conversion cost when rendering contiguous slices |

**Installation:** No new dependencies required.

---

## Architecture Patterns

### Recommended File Structure
```
src/widget/
├── rich_log.rs          # RichLog widget (new)
├── loading_indicator.rs # LoadingIndicator standalone widget (new)
├── log.rs               # Existing — unchanged
└── context.rs           # Add loading_widgets field + set_loading() method
src/app.rs               # render_widget_tree: add loading overlay draw step
src/lib.rs               # pub use new widgets + re-export
```

### Pattern 1: RichLog — styled line storage and rendering

**What:** `Vec<Line<'static>>` (or `VecDeque<Line<'static>>`) stores styled spans. `buf.set_line(x, y, &line, width)` renders them. Scroll mechanics are identical to `Log`.

**When to use:** Any time callers need to display pre-styled `ratatui::text::Line` objects.

**Key ratatui API:**
```rust
// Source: ratatui 0.30 docs — Buffer::set_line
// Renders a styled Line into the buffer at (x, y), truncated to `width` columns.
buf.set_line(area.x, area.y + row as u16, &self.lines[line_idx], area.width - 1);
```

**max_lines eviction:**
```rust
// Vec approach: evict from head when over capacity
pub fn write_line(&self, line: ratatui::text::Line<'static>) {
    if let Some(max) = self.max_lines {
        let mut lines = self.lines.borrow_mut();
        if lines.len() >= max {
            lines.drain(0..1);  // O(n) but simple; acceptable for log sizes
            // If scroll_offset > 0, decrement by 1 to avoid jump
            let off = self.scroll_offset.get_untracked();
            if off > 0 {
                self.scroll_offset.set(off - 1);
            }
        }
        lines.push(line);
    } else {
        self.lines.borrow_mut().push(line);
    }
    // auto-scroll logic (same as Log)
}
```

**Why not `Reactive<Vec<Line>>>`:** `Line<'static>` implements `Clone`, so `Reactive<Vec<Line<'static>>>` works. But `RefCell<Vec<Line<'static>>>` is simpler here because `write_line` takes `&self` and needs interior mutability — the `Reactive` pattern posts change notifications which we may not need for a pure-push append-only log.

**Decision:** Use `lines: RefCell<Vec<Line<'static>>>` (same internal mutability approach as the existing `Log` uses `Reactive<Vec<String>>`). Either works; `Reactive` is fine if the planner wants render-triggered re-draws on new content.

### Pattern 2: Spinner animation — frame cycling with `skip_animations` gate

**What:** The Python Textual `LoadingIndicator` animates five `●` dots with color gradient over elapsed time. For textual-rs, use the same frame-based approach as `ProgressBar` (tick counter + frame array).

**Spinner frames (Python Textual reference):**
The Python version uses five `●` characters with gradient opacity based on elapsed time. For textual-rs, the idiomatic approach is a tick counter cycling through pre-computed dot states:

```rust
// Five dots, varying brightness by cycling offset
const SPINNER_FRAMES: [&str; 5] = ["●    ", " ●   ", "  ●  ", "   ● ", "    ●"];
// Or the "breathing" variant: single dot with Unicode braille/block progression
// Simpler option: Braille clock-face characters
const SPINNER_DOTS: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
```

The `ProgressBar` indeterminate mode uses `tick: Cell<u8>` incremented on each `render()` call. The app renders at 30fps (33ms tick in `run_async`). With 8 braille frames cycling at 30fps, one full rotation takes ~267ms — visually appropriate.

**skip_animations gate (same pattern as Switch):**
```rust
fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
    if ctx.skip_animations {
        // Deterministic: always render frame 0 (static dot)
        buf.set_string(area.x, area.y, "Loading...", style);
        return;
    }
    let frame = self.tick.get() % SPINNER_FRAMES.len() as u8;
    self.tick.set(self.tick.get().wrapping_add(1));
    buf.set_string(area.x, area.y, SPINNER_FRAMES[frame as usize], style);
}
```

**Confidence:** HIGH — directly mirrors Switch/ProgressBar patterns already in the codebase.

### Pattern 3: Per-widget loading overlay via AppContext

**What:** `AppContext` holds `loading_widgets: RefCell<SecondaryMap<WidgetId, bool>>`. Setting `ctx.set_loading(id, true)` inserts the ID. In `render_widget_tree`, after `widget.render(ctx, content_area, buf)`, if `id` is in `loading_widgets`, call `draw_loading_overlay(rect, buf, ctx, tick)`.

**Why not `active_overlay`:** `active_overlay` is single-instance (holds one `Box<dyn Widget>`) and is already used by CommandPalette and ContextMenu. Multiple loading overlays cannot share it. Confirmed in STATE.md: "active_overlay on AppContext is single-instance."

**Why not a new Widget child:** Mounting a LoadingIndicator as a child widget requires the compose/mount cycle, which needs `&mut AppContext`. The deferred pattern (`pending_screen_pushes`) is the only safe way to do this from `&self`, but it adds a one-frame lag and complexity. The render-time draw approach is immediate and zero-cost.

**Implementation surface:**
```rust
// In context.rs — AppContext additions
pub loading_widgets: RefCell<SecondaryMap<WidgetId, bool>>,

pub fn set_loading(&self, id: WidgetId, loading: bool) {
    let mut map = self.loading_widgets.borrow_mut();
    if loading {
        map.insert(id, true);
    } else {
        map.remove(id);
    }
}
```

**In `render_widget_tree` (app.rs), after the existing `widget.render(ctx, content_area, buf)` call:**
```rust
// Draw loading overlay if this widget is in loading state
if ctx.loading_widgets.borrow().contains_key(id) {
    draw_loading_spinner_overlay(rect, frame.buffer_mut(), ctx);
}
```

**`draw_loading_spinner_overlay` shared function:**
```rust
fn draw_loading_spinner_overlay(rect: Rect, buf: &mut Buffer, ctx: &AppContext) {
    // Fill rect with semi-opaque background (use $boost color: Rgb(20,20,28) at ~50% blend)
    // Center a tick-based spinner string in the rect
    // Uses a module-level AtomicU8 tick OR passes tick through ctx
}
```

**Tick source for overlay spinner:** The overlay draw cannot store per-widget tick state in `render_widget_tree`. Options:
1. Store a `spinner_tick: Cell<u8>` on `AppContext` — incremented once per `full_render_pass`. All loading overlays use the same tick (they all animate in sync). This is the simplest approach.
2. Store `loading_widgets: RefCell<SecondaryMap<WidgetId, u8>>` — per-widget tick. More state, enables phase-offset animations.

**Recommendation:** Option 1 (shared tick on AppContext). Synchronized spinners look intentional, not like a bug.

### Pattern 4: LoadingIndicator standalone widget

**What:** A standalone `LoadingIndicator` struct that renders the spinner and fills its entire allocated area. Uses the same shared spinner frame logic. Sets `default_css` to `"LoadingIndicator { width: 100%; height: 100%; min-height: 1; }"`.

**When to use:** When the developer wants to compose a loading widget directly (e.g., inside a Container) rather than overlaying it on an existing widget.

**Structure:**
```rust
pub struct LoadingIndicator {
    tick: Cell<u8>,
}

impl Widget for LoadingIndicator {
    fn widget_type_name(&self) -> &'static str { "LoadingIndicator" }
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // fills background, centers spinner in area
        // gates on ctx.skip_animations
    }
}
```

### Anti-Patterns to Avoid

- **Using `active_overlay` for loading state:** It is single-instance; cannot have two widgets loading simultaneously.
- **Storing `Line<'a>` with a lifetime parameter:** Use `Line<'static>` (all spans own their strings). Avoids lifetime propagation through the widget trait (which requires `'static`).
- **Calling `buf.set_line()` with width = `area.width`:** The last column is reserved for the scrollbar; use `area.width - 1` for text width (same as `Log`).
- **Evicting lines without adjusting `scroll_offset`:** When a line is drained from the head, decrement `scroll_offset` by 1 (if > 0) to prevent view jumping.
- **Animating at `render()` call frequency without `skip_animations` gate:** Snapshots will be non-deterministic. Always check `ctx.skip_animations` and return a static frame.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Styled text rendering | Custom span painter | `buf.set_line(x, y, &line, width)` | ratatui built-in, handles modifiers, colors, unicode width |
| Spinner frame timing | Custom Instant-based timer | tick counter incremented per render + 30fps app loop | App already renders at 30fps; tick increment in render() is the established pattern (ProgressBar, Switch) |
| Loading overlay background | Custom alpha blending | Direct cell fill with a dark Rgb color | Terminal cells have no alpha; just write dark background cells over the widget area |
| Per-widget state without Widget trait changes | New trait method | `SecondaryMap<WidgetId, bool>` on AppContext | SecondaryMap is already used for computed_styles, dirty, pseudo_classes — same pattern |

**Key insight:** ratatui's `Buffer::set_line` is the correct primitive for styled text. Hand-rolling span-by-span rendering duplicates ratatui internals and misses unicode width handling.

---

## Common Pitfalls

### Pitfall 1: `Line<'a>` lifetime escaping the widget
**What goes wrong:** `RichLog` stores `Vec<Line<'a>>` with a lifetime parameter, forcing the widget struct to be generic — incompatible with `Box<dyn Widget>` (which requires `'static`).
**Why it happens:** `ratatui::text::Line<'a>` defaults to borrowing its span strings. Developers write `Line::from("text")` which creates `Line<'_>`.
**How to avoid:** Use `Line<'static>` throughout. `Line::from("literal")` and `Span::styled("text".to_string(), style)` both produce `Line<'static>`.
**Warning signs:** Compiler error "the parameter type `'a` may not live long enough" when boxing the widget.

### Pitfall 2: Evicting lines without correcting scroll_offset
**What goes wrong:** With `max_lines = 100`, when line 101 is pushed, line 0 is evicted. If `scroll_offset = 50` (user scrolled up), the view silently shifts by one line — text appears to jump.
**Why it happens:** Eviction shrinks the vec by 1 at the front; all logical line indices shift down by 1; but `scroll_offset` is still pointing at the old index.
**How to avoid:** After eviction, `if scroll_offset > 0 { scroll_offset -= 1 }`.
**Warning signs:** Log view jumps one line on every new push when user has scrolled up.

### Pitfall 3: Loading overlay drawn before widget chrome, not after
**What goes wrong:** Loading overlay appears under the widget's border because `paint_chrome` runs before `widget.render()` in `render_widget_tree`, and both write to the same buffer cells.
**Why it happens:** The draw order is: paint_chrome → widget.render → (loading overlay must go here).
**How to avoid:** The loading overlay draw must happen AFTER `widget.render()` — it overwrites cells in the widget's rect, including cells the widget just painted. The check in `render_widget_tree` must be after the `widget.render()` call, before moving to the next widget in DFS order.
**Warning signs:** Border is visible through the spinner, or spinner is invisible.

### Pitfall 4: Spinner tick advancing too fast
**What goes wrong:** tick is `Cell<u8>` incremented every render call, and the app renders at 30fps. With 8 frames and no throttle, the spinner cycles at 30/8 = ~4 rotations/second — very fast.
**Why it happens:** render() is called every 33ms; 8 frames at that rate = 266ms per cycle.
**How to avoid:** Use `tick / 2` or `tick / 4` as the frame index to slow down. OR use fewer frames. Python Textual targets ~16fps for the indicator (auto_refresh = 1/16). At 30fps app tick, incrementing every other render (tick % 2 == 0 then advance frame) gives 15fps animation — appropriate.
**Warning signs:** Spinner looks like flicker rather than a smooth rotation.

### Pitfall 5: SecondaryMap key validity after widget unmount
**What goes wrong:** A widget is unmounted (removed from arena) but its ID remains in `loading_widgets`. The dangling ID causes `SecondaryMap::get` to panic or return stale data.
**Why it happens:** SecondaryMap keys become invalid when the primary DenseSlotMap removes the entry. Accessing a stale key via `[]` panics; `get()` returns `None`.
**How to avoid:** In `on_unmount` for a widget that uses `ctx.set_loading`, call `ctx.set_loading(id, false)`. For the render-time overlay approach, since it uses `get()` not `[]`, stale keys silently return `None` — safe. Still clean up in unmount for correctness.
**Warning signs:** Memory growth in `loading_widgets` map over time if widgets are frequently mounted/unmounted while loading.

---

## Code Examples

Verified patterns from codebase inspection:

### RichLog: rendering a styled line
```rust
// Source: ratatui Buffer API — set_line renders a Line<'static> into buffer cells
// width param truncates to avoid writing into the scrollbar column
buf.set_line(area.x, area.y + row as u16, &self.lines[line_idx], area.width - 1);
```

### Loading overlay: checking the map in render_widget_tree
```rust
// After widget.render() call in render_widget_tree (app.rs)
if ctx.loading_widgets.borrow().contains_key(id) {
    let tick = ctx.spinner_tick.get();
    draw_loading_spinner_overlay(rect, frame.buffer_mut(), tick, ctx.skip_animations);
    // Do NOT increment tick here — it's incremented once per full_render_pass
}
```

### skip_animations gate (from Switch widget — established pattern)
```rust
// Source: src/widget/switch.rs lines 135-160
if ctx.skip_animations {
    // Return deterministic static render
    buf.set_string(area.x, area.y, "●    ", style);
    return;
}
let frame = (self.tick.get() / 2) % SPINNER_FRAMES.len() as u8;
self.tick.set(self.tick.get().wrapping_add(1));
```

### RichLog: write_line with max_lines eviction
```rust
pub fn write_line(&self, line: ratatui::text::Line<'static>) {
    let mut lines = self.lines.borrow_mut();
    if let Some(max) = self.max_lines {
        while lines.len() >= max {
            lines.drain(0..1);
            let off = self.scroll_offset.get_untracked();
            if off > 0 {
                self.scroll_offset.set(off - 1);
            }
        }
    }
    lines.push(line);
    // auto-scroll: same logic as Log::push_line
    if self.auto_scroll.get() {
        let count = lines.len();
        let vh = self.viewport_height.get() as usize;
        if vh > 0 && count > vh {
            self.scroll_offset.set(count - vh);
        }
    }
}
```

---

## Resolved Research Flag

**STATE.md flag:** "Phase 8: `widget.loading = true` base-class overlay integration scope vs. standalone widget only"

**Resolution:** Do NOT add `loading` to the `Widget` trait (that is WIDGET-F03, explicitly marked Future). Instead:

1. Add `loading_widgets: RefCell<SecondaryMap<WidgetId, bool>>` and `spinner_tick: Cell<u8>` to `AppContext`.
2. Expose `ctx.set_loading(id: WidgetId, loading: bool)` as the public API.
3. In `render_widget_tree`, after each widget's `render()`, check `loading_widgets` and draw the spinner overlay if present.
4. Increment `spinner_tick` once per `full_render_pass` (not per-widget-render).
5. `LoadingIndicator` standalone widget renders the same spinner using its own `tick: Cell<u8>`.

This satisfies WIDGET-10 ("any widget via `widget.loading = true`") by making the API `ctx.set_loading(my_id, true)` — callable from `on_action(&self, ctx)` without `&mut Widget`. The wording "widget.loading = true" is the Python Textual idiom; in textual-rs the equivalent idiomatic API is `ctx.set_loading(self.own_id, true)`.

**Confidence:** HIGH — the `active_overlay` single-instance constraint is verified in context.rs line 61 and STATE.md. The SecondaryMap pattern is verified as the standard way to attach per-widget metadata in this codebase.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Log` with `Vec<String>` | `RichLog` with `Vec<Line<'static>>` | Phase 8 (new) | Styled spans, colors, bold/italic in log output |
| No loading state | `ctx.set_loading(id, bool)` overlay | Phase 8 (new) | Any widget can show spinner without trait changes |

**Deprecated/outdated:**
- None for this phase. The existing `Log` widget stays as-is; `RichLog` is additive.

---

## Open Questions

1. **Spinner visual: braille chars vs. five-dot gradient**
   - What we know: Python Textual uses five `●` with time-based gradient. The textual-rs codebase uses braille in canvas.rs (`⣾⣽⣻⢿⡿⣟⣯⣷`) for sparklines.
   - What's unclear: Which is more readable at different terminal sizes / color depths.
   - Recommendation: Use braille clock-face sequence (`⣾⣽⣻⢿⡿⣟⣯⣷`) — 8 frames, same Unicode range already supported, centered in the widget area. Fallback to ASCII `|/-\` if terminal_caps.unicode is false.

2. **RichLog: `Reactive<Vec<Line<'static>>>` vs. `RefCell<Vec<Line<'static>>>`**
   - What we know: `Log` uses `Reactive<Vec<String>>`. `Reactive` signals trigger re-render subscriptions via `reactive_graph`.
   - What's unclear: Whether `RichLog` needs to participate in reactive subscriptions (e.g., if another widget reads from it) or if push-invalidation is sufficient.
   - Recommendation: Use `Reactive<Vec<Line<'static>>>` to match the `Log` widget pattern. The reactive graph re-render trigger on `write_line` is desirable — it ensures the view updates even if `write_line` is called from a worker callback.

3. **Loading overlay: does it block input events?**
   - What we know: Python Textual's `LoadingIndicator` intercepts all `InputEvent` to prevent interaction with underlying widgets.
   - What's unclear: Whether textual-rs needs the same. The loading state is per-widget, not per-screen.
   - Recommendation: For Phase 8, do not block input to the underlying widget when loading. Add input blocking as a follow-up only if users request it. The visual overlay is sufficient to communicate state.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is purely code/config changes. No external tools, databases, CLIs, or services beyond the project's own Rust toolchain are required.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | insta 1.46.3 + cargo test |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test -p textual-rs rich_log loading -- --nocapture` |
| Full suite command | `cargo test -p textual-rs` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIDGET-09 | RichLog renders styled `Line<'static>` objects | unit | `cargo test -p textual-rs rich_log` | No — Wave 0 |
| WIDGET-09 | RichLog auto-scrolls to bottom on new line | unit | `cargo test -p textual-rs rich_log_auto_scroll` | No — Wave 0 |
| WIDGET-09 | max_lines eviction removes oldest line and adjusts scroll_offset | unit | `cargo test -p textual-rs rich_log_max_lines` | No — Wave 0 |
| WIDGET-09 | Snapshot: RichLog renders styled spans correctly | snapshot | `cargo test -p textual-rs snapshot_rich_log` | No — Wave 0 |
| WIDGET-10 | ctx.set_loading(id, true) causes spinner to appear over widget area | unit | `cargo test -p textual-rs loading_overlay` | No — Wave 0 |
| WIDGET-10 | ctx.set_loading(id, false) removes the spinner | unit | `cargo test -p textual-rs loading_overlay_off` | No — Wave 0 |
| WIDGET-10 | LoadingIndicator standalone renders static frame when skip_animations=true | snapshot | `cargo test -p textual-rs snapshot_loading_indicator` | No — Wave 0 |
| WIDGET-10 | LoadingIndicator renders "Loading..." text (not empty) when skip_animations=true | unit | `cargo test -p textual-rs loading_indicator_skip_anim` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p textual-rs rich_log loading`
- **Per wave merge:** `cargo test -p textual-rs`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Tests for `RichLog` — to be added to `tests/widget_tests.rs` (same file pattern as `Log` tests at line 1555+)
- [ ] Tests for loading overlay + `LoadingIndicator` — to be added to `tests/widget_tests.rs`
- [ ] Snapshot baselines will be auto-generated by `cargo test` on first run (insta behavior: new snapshots are written to `tests/snapshots/` pending review)

*(No new test infrastructure required — insta + cargo test is already in place)*

---

## Sources

### Primary (HIGH confidence)
- `src/widget/log.rs` — direct template for RichLog struct layout, scroll mechanics, key bindings, render pattern
- `src/widget/progress_bar.rs` — tick animation pattern (`tick: Cell<u8>`, indeterminate mode)
- `src/widget/switch.rs` — `skip_animations` gate pattern, Tween usage
- `src/widget/context.rs` — AppContext field inventory; confirmed `active_overlay` is single-instance (line 61); `SecondaryMap` pattern confirmed
- `src/app.rs` fn `render_widget_tree` (lines 1138-1224) — confirmed draw order: paint_chrome → widget.render → active_overlay last; loading overlay insert point identified
- `.planning/STATE.md` — confirmed research flag; confirmed `active_overlay` single-instance constraint
- ratatui 0.30 (in Cargo.toml) — `Buffer::set_line` API is the correct primitive for `Line<'static>` rendering

### Secondary (MEDIUM confidence)
- Python Textual LoadingIndicator source (GitHub) — spinner visual reference: five `●` dots, gradient-based opacity, auto_refresh=1/16, `skip_animations` → "Loading..." text
- Python Textual Widget.loading docs — confirmed it's a reactive bool that mounts/unmounts a child `LoadingIndicator`; textual-rs equivalent is render-time overlay (different mechanism, same UX)
- Python Textual RichLog docs — confirmed `max_lines: int | None`, `auto_scroll: bool = True`, `write()` method appends `Line`-compatible renderables

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — ratatui 0.30 already in use; no new deps needed
- Architecture (RichLog): HIGH — directly derived from existing `Log` widget
- Architecture (loading overlay): HIGH — verified AppContext field patterns and render_widget_tree draw order
- Pitfalls: HIGH — derived from direct codebase reading, not speculation
- Python Textual visual reference: MEDIUM — fetched from GitHub source but not verified against a tagged release

**Research date:** 2026-03-27
**Valid until:** 2026-05-01 (stable domain; ratatui 0.30 API is stable)
