use std::cell::{Cell, RefCell};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::registry::{Command, fuzzy_score};
use crate::event::keybinding::KeyBinding;
use crate::widget::context::AppContext;
use crate::widget::{EventPropagation, Widget, WidgetId};

/// CommandPalette — a searchable overlay listing all discovered commands.
///
/// Opened via Ctrl+P as a screen overlay (push_screen_deferred).
/// Filters commands with fuzzy matching as the user types.
/// Up/Down navigate the list, Enter executes and dismisses, Esc dismisses.
pub struct CommandPalette {
    /// All commands available in this palette instance.
    commands: Vec<Command>,
    /// Current search query text.
    query: RefCell<String>,
    /// Currently selected index within the filtered list.
    selected_index: Cell<usize>,
    /// Own widget ID, set in on_mount.
    own_id: Cell<Option<WidgetId>>,
}

impl CommandPalette {
    /// Create a new CommandPalette with the given list of commands.
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            commands,
            query: RefCell::new(String::new()),
            selected_index: Cell::new(0),
            own_id: Cell::new(None),
        }
    }

    /// Return the filtered commands for the current query.
    fn filtered_commands(&self) -> Vec<&Command> {
        let query = self.query.borrow();
        if query.is_empty() {
            // Show all commands when query is empty
            return self.commands.iter().collect();
        }
        let threshold = 0.3_f64;
        let mut scored: Vec<(&Command, f64)> = self.commands
            .iter()
            .filter_map(|cmd| {
                let score = fuzzy_score(&query, &cmd.name);
                if score >= threshold {
                    Some((cmd, score))
                } else {
                    None
                }
            })
            .collect();
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(cmd, _)| cmd).collect()
    }
}

static PALETTE_BINDINGS: &[KeyBinding] = &[];

impl Widget for CommandPalette {
    fn widget_type_name(&self) -> &'static str {
        "CommandPalette"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "CommandPalette { background: #12121a; border: rounded #00d4ff; width: 60; padding: 1 2; }"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        PALETTE_BINDINGS
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(k) = event.downcast_ref::<crossterm::event::KeyEvent>() {
            if k.kind != crossterm::event::KeyEventKind::Press {
                return EventPropagation::Continue;
            }
            match k.code {
                KeyCode::Esc => {
                    ctx.dismiss_overlay();
                    return EventPropagation::Stop;
                }
                KeyCode::Enter => {
                    let filtered = self.filtered_commands();
                    let idx = self.selected_index.get();
                    if let Some(cmd) = filtered.get(idx) {
                        // Execute the command: dispatch action to target widget
                        if let Some(target_id) = cmd.target_id {
                            if let Some(widget) = ctx.arena.get(target_id) {
                                widget.on_action(&cmd.action, ctx);
                            }
                        }
                        // For app-level commands (no target_id), log for now
                    }
                    ctx.dismiss_overlay();
                    return EventPropagation::Stop;
                }
                KeyCode::Up => {
                    let current = self.selected_index.get();
                    if current > 0 {
                        self.selected_index.set(current - 1);
                    }
                    return EventPropagation::Stop;
                }
                KeyCode::Down => {
                    let filtered_count = self.filtered_commands().len();
                    let current = self.selected_index.get();
                    if current + 1 < filtered_count {
                        self.selected_index.set(current + 1);
                    }
                    return EventPropagation::Stop;
                }
                KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.query.borrow_mut().push(c);
                    self.selected_index.set(0);
                    return EventPropagation::Stop;
                }
                KeyCode::Backspace => {
                    self.query.borrow_mut().pop();
                    self.selected_index.set(0);
                    return EventPropagation::Stop;
                }
                _ => {}
            }
        }
        EventPropagation::Continue
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Colors
        let cyan = Color::Rgb(0, 212, 255);     // #00d4ff
        let green = Color::Rgb(0, 255, 163);    // #00ffa3
        let dark = Color::Rgb(10, 10, 15);      // #0a0a0f
        let muted = Color::Rgb(74, 74, 90);     // #4a4a5a
        let bg = Color::Rgb(18, 18, 26);        // #12121a
        let normal = Color::Rgb(200, 200, 216); // ~#c8c8d8

        let _bg_style = Style::default().bg(bg);

        // Calculate floating panel dimensions (centered, max 60 cols, max 20 rows)
        let panel_w = area.width.min(60);
        let panel_h = area.height.min(20);
        let px = area.x + (area.width.saturating_sub(panel_w)) / 2;
        let py = area.y + (area.height.saturating_sub(panel_h)) / 4; // upper-third

        // Draw McGugan box border
        let border_color = Color::Rgb(0, 212, 255);
        crate::canvas::mcgugan_box(buf, px, py, panel_w, panel_h, border_color, bg, Color::Reset);

        // Fill inside
        for y in (py + 1)..(py + panel_h - 1) {
            for x in (px + 1)..(px + panel_w - 1) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(bg);
                }
            }
        }

        let inner_x = px + 1;
        let inner_w = panel_w.saturating_sub(2);
        let mut row = py + 1;

        let panel_bottom = py + panel_h - 1;
        let iw = inner_w as usize;

        // Title bar: "Command Palette" bold cyan
        if row < panel_bottom {
            let title = "Command Palette";
            let title_style = Style::default()
                .fg(cyan)
                .bg(bg)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner_x, row, title, title_style);
            row += 1;
        }

        // Divider line
        if row < panel_bottom {
            let divider: String = "─".repeat(iw);
            buf.set_string(inner_x, row, &divider, Style::default().fg(muted).bg(bg));
            row += 1;
        }

        // Search query line
        if row < panel_bottom {
            let query = self.query.borrow();
            let prompt_style = Style::default().fg(cyan).bg(bg);
            let query_style = Style::default().fg(normal).bg(bg);

            buf.set_string(inner_x, row, "> ", prompt_style);
            if query.is_empty() {
                let placeholder = "Type to search commands...";
                let ph_style = Style::default().fg(muted).bg(bg);
                let display: String = placeholder.chars().take(iw.saturating_sub(2)).collect();
                buf.set_string(inner_x + 2, row, &display, ph_style);
            } else {
                let display: String = query.chars().take(iw.saturating_sub(2)).collect();
                buf.set_string(inner_x + 2, row, &display, query_style);
            }
            row += 1;
        }

        // Another divider
        if row < panel_bottom {
            let divider: String = "─".repeat(iw);
            buf.set_string(inner_x, row, &divider, Style::default().fg(muted).bg(bg));
            row += 1;
        }

        // Command list
        let filtered = self.filtered_commands();
        let selected = self.selected_index.get();

        if filtered.is_empty() {
            // No results
            if row < panel_bottom {
                let query = self.query.borrow();
                let msg = format!("No commands match '{}'", query);
                let display: String = msg.chars().take(iw).collect();
                buf.set_string(inner_x, row, &display, Style::default().fg(muted).bg(bg));
            }
        } else {
            for (i, cmd) in filtered.iter().enumerate() {
                if row >= panel_bottom {
                    break;
                }

                let is_selected = i == selected;
                let (fg, row_bg) = if is_selected {
                    (dark, green)
                } else {
                    (normal, bg)
                };
                let row_style = Style::default().fg(fg).bg(row_bg);

                // Clear the row
                let blank: String = " ".repeat(iw);
                buf.set_string(inner_x, row, &blank, row_style);

                // Command name (left-aligned)
                let name_display: String = cmd.name.chars().take(30).collect();
                buf.set_string(inner_x, row, &name_display, row_style);

                // Source type (muted, after name)
                let source_x = inner_x + 32;
                if (source_x as usize) < (inner_x + inner_w) as usize {
                    let source_style = if is_selected {
                        Style::default().fg(dark).bg(row_bg)
                    } else {
                        Style::default().fg(muted).bg(row_bg)
                    };
                    let source_display: String = cmd.source.chars().take(15).collect();
                    buf.set_string(source_x, row, &source_display, source_style);
                }

                // Keybinding (right-aligned, cyan when not selected)
                if let Some(ref kb) = cmd.keybinding {
                    let kb_len = kb.chars().count();
                    let kb_x = (inner_x + inner_w).saturating_sub(kb_len as u16 + 1);
                    if kb_x > source_x {
                        let kb_style = if is_selected {
                            Style::default().fg(dark).bg(row_bg)
                        } else {
                            Style::default().fg(cyan).bg(row_bg)
                        };
                        buf.set_string(kb_x, row, kb, kb_style);
                    }
                }

                row += 1;
            }
        }
    }
}

// Suppress unused import warning for Line/Span — they're used for future rich rendering
// but currently we use buf.set_string directly for simplicity.
#[allow(dead_code)]
fn _use_line_span() {
    let _: Line = Line::default();
    let _: Span = Span::default();
}
