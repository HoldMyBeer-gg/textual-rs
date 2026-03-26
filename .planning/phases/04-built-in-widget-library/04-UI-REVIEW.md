# Phase 04 — UI Review

**Audited:** 2026-03-25
**Baseline:** Abstract 6-pillar standards (no UI-SPEC.md for this phase)
**Screenshots:** Not captured — this is a Rust TUI (ratatui) framework; no web dev server. Audit is code-only using widget source, demo examples, and snapshot test baselines.
**Registry audit:** No `components.json` found — shadcn not initialized, registry audit skipped.

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Demo strings are purposeful; "Submit" / "Cancel" buttons are acceptable generic CTA labels in a framework showcase, but the Controls tab label section uses `"-- Form Controls --"` and `"-- Data Display --"` dashes which are a low-polish pattern |
| 2. Visuals | 3/4 | Clear semantic indicators (▼/▶ collapsible, ━━━◉/◉━━━ switch, (●)/( ) radio) throughout; no per-widget focus border change — focused widgets have no border highlight, only cursor-level REVERSED for Input/TextArea |
| 3. Color | 3/4 | Consistent two-tone dark palette across both demos (rgb(10,10,15) body, rgb(18,18,26) chrome, rgb(0,255,163) accent on header/footer); ButtonVariant enum (Primary/Warning/Error/Success) declared but never applied in render — all buttons render identically |
| 4. Typography | 4/4 | TUI has no font sizes; terminal weight/modifier usage is well-differentiated: BOLD for headings, ITALIC for emphasis, DIM for placeholders/code blocks/scrollbar indicators, REVERSED for selection cursors — clean and intentional |
| 5. Spacing | 3/4 | Border types are varied appropriately (rounded for Input/DataTable/ListView, heavy for buttons, solid for IRC panels); inner content area correctly shrunk by paint_chrome; Switch rendered at fixed 8-col width but demo renders it at `width: 20` CSS override creating leftover dead space |
| 6. Experience Design | 2/4 | Loading/indeterminate progress bar implemented; empty list/sparkline handled gracefully; auto-scroll logic in Log is solid; BUT: no disabled widget state anywhere in the system, no focus border visual cue on non-input widgets (Button has no focused appearance aside from footer hint), ButtonVariant visual differentiation is entirely non-functional |

**Overall: 18/24**

---

## Top 3 Priority Fixes

1. **ButtonVariant has no visual effect** — Users pressing Tab to focus a Primary button see an identical appearance to a Default button. In a showcase demo where `btn_primary = Button::new("Submit").with_variant(ButtonVariant::Primary)` is used, the button renders the same color as `Button::new("Cancel")`. Fix: in `button.rs` render(), read `self.variant` and derive a color (`Color::Cyan` for Primary, `Color::Yellow` for Warning, `Color::Red` for Error, `Color::Green` for Success) applied via `base_style.fg(color)` so variant information reaches the terminal cell.

2. **No focused-widget border highlight** — No widget except Input (cursor) and Select (overlay cursor) visually communicates focus state at the container border level. A user tabbing through the Controls pane has no indication which Button or Checkbox is focused. Fix: add a `:focus` rule in each widget's `default_css()` that changes border color, e.g. `Button:focus { border: heavy; color: rgb(0,255,163); }` — the CSS cascade and PseudoClass::Focus machinery already exist and are already wired in `advance_focus()` in `widget/tree.rs`.

3. **Demo copywriting: ASCII-art section dividers** — `"-- Form Controls --"` and `"-- Data Display --"` in `demo.rs` (lines 90, 156) use ASCII dash patterns that look unpolished in a framework demonstration. Fix: replace with clean plain labels (`"Form Controls"`, `"Data Widgets"`) styled via Label — or use a `Collapsible` widget to separate sections, which would also demonstrate that widget in context.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

**Strengths:**
- IRC demo log lines (`irc_demo.rs:108-125`) are realistic, domain-appropriate, and read naturally as IRC messages. Usernames, timestamps, and message content all feel believable.
- Header subtitles are context-specific: `"Tab to navigate | q to quit"` (demo) and `"#general -- 7 users"` (IRC) explain the interface rather than using generic help text.
- Key binding descriptions in widget sources are short and action-oriented: `"Move up"`, `"Jump word left"`, `"Toggle"`, `"Sort"`.
- Placeholder text on Input widgets (`"Type something..."`, `"Type a message..."`) is appropriate for context.
- Select widget uses `"▼ {option}"` (select.rs:104) and overlay labels `"↑↓ Navigate  Enter Select  Esc Cancel"` (select.rs:~160) — instructional and complete.
- Collapsible uses `"▼ {title}"` / `"▶ {title}"` — standard convention, correct.

**Issues:**
- `demo.rs:90`: `Label::new("-- Form Controls --")` — the double-dash pattern is a code comment convention leaking into UI copy. Use `"Form Controls"` or `"Controls"`.
- `demo.rs:156`: `Label::new("-- Data Display --")` — same issue.
- `demo.rs:130`: `Button::new("Submit")` and `demo.rs:133`: `Button::new("Cancel")` — acceptable in a form demo context, but these are placeholder-grade labels in a showcase context. A framework demo benefits from showing purpose-specific labels like `"Connect"` + `"Disconnect"` to inspire API consumers.
- Key binding description `"Press"` on both Enter and Space bindings in `button.rs:57-67` — this appears in internal data only (`show: false`) so it never surfaces to users, but if it did, "Press" as a description for the action of pressing a button is circular.

### Pillar 2: Visuals (3/4)

**Strengths:**
- Widget state indicators are consistently implemented with Unicode characters that match Textual's Python reference:
  - Switch: `━━━◉` (on) / `◉━━━` (off) — visually distinct, immediately readable
  - Checkbox: `[X]` / `[ ]` — universally understood ASCII
  - Radio: `(●)` / `( )` — correct convention
  - Collapsible: `▼` / `▶` — standard disclosure triangle convention
  - DataTable: `▲` / `▼` sort indicators with `─┼─` separator row
  - Tree: `├──` / `└──` / `│` guide chars (FlatEntry ancestor_is_last)
- Progress bar uses `█` (filled) and `░` (empty) — strong visual contrast at all terminal color depths
- Sparkline 8-level block characters (`▁▂▃▄▅▆▇█`) provide proportionally meaningful data visualization
- Scrollbar uses `█` (thumb) / `│` (track) for ListView, Log, DataTable — legible and consistent
- Header/Footer dock layout creates clear visual frame around application content
- TabbedContent renders active tab with `REVERSED` modifier, inactive tabs without — correct convention
- render_style.rs `BorderStyle` covers five variants (None/Solid/Rounded/Heavy/Double/Ascii) with correct box-drawing characters

**Issues:**
- No focus-ring equivalent: focused Button, Checkbox, Switch, RadioButton, Collapsible, ListView all look identical to unfocused state except for the footer key binding hint. Users who cannot see the footer (small terminal, no Footer widget) have no way to determine focus position. This is the largest usability gap in the library.
- Button render (`button.rs:106-126`) does not consult `self.variant` at all. `ButtonVariant::Primary`, `Warning`, `Error`, `Success` are declared but produce identical output to `Default`. The demo shows `btn_primary` and `btn_default` side by side; both render in the same style.
- Header.rs (line 67) inherits style from `buf.cell((area.x, area.y))` rather than from the widget's `own_id` computed style. If no background has been painted at that cell yet, it defaults to the terminal's style. The demo works because DemoScreen's background CSS fills first, but a Header used without a parent background could show incorrect colors.
- Markdown inline code style is noted as "append backticks as text marker" (`markdown.rs:293-297`) because per-segment style mixing is not supported in `RenderedLine`. So `` `code` `` appears as literal backtick-code-backtick in normal text style, not dim/monospace. The comment acknowledges this as a v1 limitation but it degrades visual quality for code-heavy documentation.

### Pillar 3: Color (3/4)

**Palette consistency:**
- Both demo applications share identical color tokens:
  - Body: `rgb(10,10,15)` — near-black with slight blue cast
  - Chrome (Header/Footer): `rgb(18,18,26)` — slightly lighter, same hue
  - Accent (Header/Footer text): `rgb(0,255,163)` — bright cyan-green (lazeport.pwn.zone reference palette)
  - Body text: `rgb(224,224,224)` — warm near-white
- The palette achieves a clear 60/30/10 split: dark body dominates (~80%), slightly lighter chrome frames (~15%), bright accent draws the eye to the header title (~5%).
- Semantic colors are appropriately scoped:
  - `Color::Red` only appears in `input.rs` for validation error state — correct accent-for-error usage
  - `Color::DarkGray` only appears in `markdown.rs` for code blocks — appropriate dim usage
  - `Modifier::DIM` used consistently for placeholder text, inactive indicators

**Issues:**
- ButtonVariant color semantics (Primary=Cyan, Warning=Yellow, Error=Red, Success=Green) are fully modeled in the enum but never rendered. Four named semantic variants produce zero visual differentiation. When the demo places a Primary "Submit" next to a Default "Cancel", they are visually identical — undermining the 60/30/10 principle which relies on deliberate accent placement.
- `ComputedStyle.color` defaults to `TcssColor::Reset` (transparent). The `draw_border` function in `render_style.rs:130-140` uses the foreground color for border character style. When no `color` CSS property is set, borders render in the terminal's default foreground color. This means unfocused border color is terminal-dependent (white on dark, black on light terminals), reducing cross-terminal consistency.
- No `:focus` color rules exist in widget `default_css()` methods or in either demo stylesheet. The CSS pseudo-class infrastructure is fully implemented (`cascade.rs` test at line 243 proves it works), but no widget uses it to change border color on focus.

### Pillar 4: Typography (4/4)

Terminal UI cannot use font sizes or families — this pillar is evaluated against ratatui modifier (weight/style) usage.

**Modifier inventory across all widgets:**
- `Modifier::BOLD` — Markdown H1-H6, DataTable header (via computed style), Markdown Strong spans
- `Modifier::ITALIC` — Markdown Emphasis spans
- `Modifier::UNDERLINED` — Markdown H1 only (adds underline to bold, creates H1-specific treatment)
- `Modifier::CROSSED_OUT` — Markdown Strikethrough
- `Modifier::REVERSED` — selected ListView row, DataTable cursor row, active Tabs tab, Input cursor position, TextArea cursor position, Select overlay cursor, Collapsible selection
- `Modifier::DIM` — Input placeholder, Markdown code blocks/inline code, progress bar empty fill (inherits), scrollbar track

This is a well-disciplined modifier palette. No modifier is used frivolously. The hierarchy BOLD > REVERSED > DIM maps cleanly to: heading > selection/active > secondary. ITALIC and UNDERLINED are reserved for Markdown semantic rendering only, not overloaded elsewhere.

The Markdown widget distinguishes H1 (BOLD + UNDERLINED) from H2-H6 (BOLD only) — the only typographic hierarchy deeper than two levels, and appropriate given the heading system's semantics.

No issues found. Score: 4/4.

### Pillar 5: Spacing (3/4)

**TUI spacing is measured in terminal columns/rows, not CSS px/rem.**

**Border variant usage is semantically appropriate:**
- `border: rounded` — Input, Select, DataTable, ListView, Placeholder (interactive/container widgets needing soft enclosure)
- `border: heavy` — Button (heavier visual weight signals primary interaction target)
- `border: solid` — IRC demo ChannelPane, ChatLog, UserPane (structural panel borders)
- `border: tall` — Button `default_css()` says `"tall"` but the demo overrides to `"heavy"` — the Button default_css uses a Textual-specific variant (`tall`) that may not parse correctly in TCSS. This is a silent failure risk.

**Known spacing issues:**
- `switch.rs:70`: `default_css()` declares `width: 8` but `demo.rs:113` renders the Switch into `width: 20` without a matching CSS override. The Switch label says `sw.render(ctx, Rect { x: area.x, y, width: 20, height: 1 }, buf)` directly, bypassing CSS. The `◉━━━` indicator is only 4 chars wide — 16 columns of dead space follow it in that row.
- `demo.rs:99`: Input rendered with `width: area.width.min(40)` as a magic constant. Using a hard-coded 40-column width in a responsive TUI application means the Input will misalign with adjacent widgets at different terminal widths.
- Collapsible child rendering in `collapsible.rs:128-135` allocates exactly 1 row per child regardless of child height — a child widget needing 3 rows would be clipped to 1 row. Acceptable for v1 with Label children, but an undocumented constraint.
- DataTable `adjust_scroll_col()` uses a magic `offset + 3` threshold (`data_table.rs:171`) to keep columns visible. The value 3 is not derived from any layout measurement — inconsistent with the rest of the layout system.

### Pillar 6: Experience Design (2/4)

**What works well:**
- Log.auto_scroll is a genuinely user-friendly experience design decision: auto-scrolls on new content, disables when user manually scrolls up, re-enables when user scrolls back to bottom. This is the correct behavior for a chat/log widget.
- Input validation renders red text immediately on invalid input — zero-delay error feedback.
- Input shows placeholder when empty and unfocused, removes it when focused — correct placeholder interaction pattern.
- Sparkline correctly handles empty data (returns early) and all-zero data (normalizes to 1.0 to avoid divide-by-zero).
- DataTable handles empty column list early (`data_table.rs:332`) and empty row set gracefully.
- ProgressBar indeterminate mode (`ProgressBar::indeterminate()`) provides meaningful visual feedback for unknown-duration operations.
- SelectOverlay provides Esc-to-cancel with `pending_screen_pops` — correct dismissal behavior.
- TreeView expand/collapse with Space key and guide chars correctly communicates hierarchy depth.

**Gaps:**
- No disabled widget state anywhere in the system. `ButtonVariant::Default` is used for both enabled and "Cancel" secondary actions. No widget has `pub enabled: bool` or similar — users of the library cannot build a form that grays out a Submit button while required fields are empty.
- No visible focus indicator on Button, Checkbox, Switch, RadioButton, Collapsible, ListView (container border), DataTable (container border). The Footer shows key bindings for the focused widget, which is an indirect focus indicator — but only if a Footer widget is mounted and visible. Many layouts may not include one.
- `button.rs` render ignores `self.variant` entirely. ButtonVariant::Primary, Warning, Error, Success produce zero visual differentiation — the enum is dead code in terms of UX impact.
- Markdown widget has no scroll capability — long markdown content silently truncates at `area.height` rows (`markdown.rs:370`, `.take(max_rows)`). There is no indication to the user that content was cut off. A "↓ {N} more lines" indicator or wrapping in ScrollView would be needed.
- Log widget has no visual indication when auto_scroll is disabled (user has scrolled up). Users may be confused why new messages stop appearing at the bottom. A status indicator like `"[paused]"` in the scrollbar column or footer area would help.
- Switch has no label. The user sees `━━━◉` but doesn't know what is being toggled unless a surrounding Label widget is manually placed by the app developer. Checkbox includes its label in the widget; Switch does not — inconsistent developer experience.

---

## Files Audited

**Planning documents:**
- `.planning/phases/04-built-in-widget-library/04-01-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-02-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-03-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-04-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-05-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-06-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-07-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-08-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-09-SUMMARY.md`
- `.planning/phases/04-built-in-widget-library/04-CONTEXT.md`

**Widget source files:**
- `crates/textual-rs/src/widget/button.rs`
- `crates/textual-rs/src/widget/checkbox.rs`
- `crates/textual-rs/src/widget/collapsible.rs`
- `crates/textual-rs/src/widget/data_table.rs`
- `crates/textual-rs/src/widget/footer.rs`
- `crates/textual-rs/src/widget/header.rs`
- `crates/textual-rs/src/widget/input.rs`
- `crates/textual-rs/src/widget/label.rs`
- `crates/textual-rs/src/widget/list_view.rs`
- `crates/textual-rs/src/widget/log.rs`
- `crates/textual-rs/src/widget/markdown.rs`
- `crates/textual-rs/src/widget/placeholder.rs`
- `crates/textual-rs/src/widget/progress_bar.rs`
- `crates/textual-rs/src/widget/sparkline.rs`
- `crates/textual-rs/src/widget/switch.rs`
- `crates/textual-rs/src/widget/tabs.rs`
- `crates/textual-rs/src/widget/tree_view.rs`
- `crates/textual-rs/src/widget/radio.rs` (via grep)
- `crates/textual-rs/src/widget/select.rs` (via grep)
- `crates/textual-rs/src/widget/text_area.rs` (via grep)
- `crates/textual-rs/src/widget/scroll_view.rs` (via grep)

**Framework files:**
- `crates/textual-rs/src/app.rs`
- `crates/textual-rs/src/css/render_style.rs`
- `crates/textual-rs/src/css/types.rs`

**Demo examples:**
- `crates/textual-rs/examples/demo.rs`
- `crates/textual-rs/examples/irc_demo.rs`
