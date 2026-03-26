use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::context::AppContext;
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a TextArea.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the text content changes. Contains the full text joined with "\n".
    pub struct Changed {
        pub value: String,
    }

    impl Message for Changed {}
}

/// A multi-line text editor widget with cursor navigation and optional line numbers.
pub struct TextArea {
    pub lines: Reactive<Vec<String>>,
    pub show_line_numbers: bool,
    cursor_row: Cell<usize>,
    cursor_col: Cell<usize>,
    scroll_offset: Cell<usize>,
    own_id: Cell<Option<WidgetId>>,
}

impl TextArea {
    /// Create a new TextArea with a single empty line.
    pub fn new() -> Self {
        Self {
            lines: Reactive::new(vec![String::new()]),
            show_line_numbers: false,
            cursor_row: Cell::new(0),
            cursor_col: Cell::new(0),
            scroll_offset: Cell::new(0),
            own_id: Cell::new(None),
        }
    }

    /// Create a new TextArea with line numbers shown.
    pub fn with_line_numbers() -> Self {
        Self {
            lines: Reactive::new(vec![String::new()]),
            show_line_numbers: true,
            cursor_row: Cell::new(0),
            cursor_col: Cell::new(0),
            scroll_offset: Cell::new(0),
            own_id: Cell::new(None),
        }
    }

    fn current_line_len(&self) -> usize {
        let lines = self.lines.get_untracked();
        let row = self.cursor_row.get();
        lines.get(row).map(|l| l.len()).unwrap_or(0)
    }

    fn post_changed(&self, ctx: &AppContext) {
        if let Some(id) = self.own_id.get() {
            let text = self.lines.get_untracked().join("\n");
            ctx.post_message(id, messages::Changed { value: text });
        }
    }

    fn word_left(&self) {
        let col = self.cursor_col.get();
        let row = self.cursor_row.get();
        let lines = self.lines.get_untracked();
        if col == 0 {
            if row > 0 {
                let new_row = row - 1;
                self.cursor_row.set(new_row);
                self.cursor_col.set(lines[new_row].len());
            }
            return;
        }
        let line = &lines[row];
        let chars: Vec<char> = line[..col].chars().collect();
        let mut new_col = col;
        // Skip whitespace backwards
        while new_col > 0 && chars[new_col - 1].is_whitespace() {
            new_col -= 1;
        }
        // Skip word chars backwards
        while new_col > 0 && !chars[new_col - 1].is_whitespace() {
            new_col -= 1;
        }
        self.cursor_col.set(new_col);
    }

    fn word_right(&self) {
        let lines = self.lines.get_untracked();
        let row = self.cursor_row.get();
        let col = self.cursor_col.get();
        let line = &lines[row];
        if col >= line.len() {
            if row < lines.len() - 1 {
                self.cursor_row.set(row + 1);
                self.cursor_col.set(0);
            }
            return;
        }
        let chars: Vec<char> = line[col..].chars().collect();
        let mut offset = 0;
        // Skip word chars forwards
        while offset < chars.len() && !chars[offset].is_whitespace() {
            offset += 1;
        }
        // Skip whitespace forwards
        while offset < chars.len() && chars[offset].is_whitespace() {
            offset += 1;
        }
        self.cursor_col.set(col + offset);
    }

    fn margin_width(&self) -> usize {
        if !self.show_line_numbers {
            return 0;
        }
        let lines = self.lines.get_untracked();
        let digit_count = lines.len().to_string().len();
        digit_count + 1 // number + space
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

static TEXT_AREA_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: "cursor_up",
        description: "Move up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: "cursor_down",
        description: "Move down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "cursor_left",
        description: "Move left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "cursor_right",
        description: "Move right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "cursor_home",
        description: "Move to line start",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "cursor_end",
        description: "Move to line end",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::CONTROL,
        action: "word_left",
        description: "Move word left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::CONTROL,
        action: "word_right",
        description: "Move word right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Backspace,
        modifiers: KeyModifiers::NONE,
        action: "delete_back",
        description: "Delete back",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Delete,
        modifiers: KeyModifiers::NONE,
        action: "delete_forward",
        description: "Delete forward",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "newline",
        description: "New line",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        action: "copy",
        description: "Copy",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('v'),
        modifiers: KeyModifiers::CONTROL,
        action: "paste",
        description: "Paste",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        action: "select_all",
        description: "Select all",
        show: false,
    },
];

impl Widget for TextArea {
    fn widget_type_name(&self) -> &'static str {
        "TextArea"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "TextArea { border: tall; min-height: 5; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        TEXT_AREA_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
            // Handle character insertion — only for plain chars (no Control/Alt, but allow Shift)
            let relevant_modifiers = key_event.modifiers & !KeyModifiers::SHIFT;
            if let KeyCode::Char(c) = key_event.code {
                if relevant_modifiers == KeyModifiers::NONE {
                    // Check it doesn't match a keybinding (those are handled by on_action)
                    let is_bound = TEXT_AREA_BINDINGS
                        .iter()
                        .any(|kb| kb.matches(key_event.code, key_event.modifiers));
                    if !is_bound {
                        let row = self.cursor_row.get();
                        let col = self.cursor_col.get();
                        self.lines.update(|lines| {
                            if let Some(line) = lines.get_mut(row) {
                                // Insert char at byte position
                                let byte_pos = line
                                    .char_indices()
                                    .nth(col)
                                    .map(|(i, _)| i)
                                    .unwrap_or(line.len());
                                line.insert(byte_pos, c);
                            }
                        });
                        self.cursor_col.set(col + 1);
                        self.post_changed(ctx);
                        return EventPropagation::Stop;
                    }
                }
            }
        }
        EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_up" => {
                let row = self.cursor_row.get();
                if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    let line_len = lines[new_row].len();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "cursor_down" => {
                let lines = self.lines.get_untracked();
                let row = self.cursor_row.get();
                if row < lines.len().saturating_sub(1) {
                    let new_row = row + 1;
                    self.cursor_row.set(new_row);
                    let line_len = lines[new_row].len();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "cursor_left" => {
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                if col > 0 {
                    self.cursor_col.set(col - 1);
                } else if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    self.cursor_col.set(lines[new_row].len());
                }
            }
            "cursor_right" => {
                let line_len = self.current_line_len();
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                if col < line_len {
                    self.cursor_col.set(col + 1);
                } else {
                    let lines = self.lines.get_untracked();
                    if row < lines.len().saturating_sub(1) {
                        self.cursor_row.set(row + 1);
                        self.cursor_col.set(0);
                    }
                }
            }
            "cursor_home" => {
                self.cursor_col.set(0);
            }
            "cursor_end" => {
                self.cursor_col.set(self.current_line_len());
            }
            "word_left" => {
                self.word_left();
            }
            "word_right" => {
                self.word_right();
            }
            "delete_back" => {
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                if col > 0 {
                    self.lines.update(|lines| {
                        if let Some(line) = lines.get_mut(row) {
                            let byte_pos = line
                                .char_indices()
                                .nth(col - 1)
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            let end_pos = line
                                .char_indices()
                                .nth(col)
                                .map(|(i, _)| i)
                                .unwrap_or(line.len());
                            line.drain(byte_pos..end_pos);
                        }
                    });
                    self.cursor_col.set(col - 1);
                    self.post_changed(ctx);
                } else if row > 0 {
                    // Join with previous line
                    let new_row = row - 1;
                    let mut lines = self.lines.get_untracked();
                    let current_line = lines.remove(row);
                    let prev_len = lines[new_row].len();
                    lines[new_row].push_str(&current_line);
                    self.lines.set(lines);
                    self.cursor_row.set(new_row);
                    self.cursor_col.set(prev_len);
                    self.post_changed(ctx);
                }
            }
            "delete_forward" => {
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                let lines_snap = self.lines.get_untracked();
                let line_len = lines_snap.get(row).map(|l| l.len()).unwrap_or(0);
                if col < line_len {
                    drop(lines_snap);
                    self.lines.update(|lines| {
                        if let Some(line) = lines.get_mut(row) {
                            let byte_pos = line
                                .char_indices()
                                .nth(col)
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            let end_pos = line
                                .char_indices()
                                .nth(col + 1)
                                .map(|(i, _)| i)
                                .unwrap_or(line.len());
                            line.drain(byte_pos..end_pos);
                        }
                    });
                    self.post_changed(ctx);
                } else {
                    drop(lines_snap);
                    let mut lines = self.lines.get_untracked();
                    if row < lines.len().saturating_sub(1) {
                        // Join next line into current
                        let next_line = lines.remove(row + 1);
                        lines[row].push_str(&next_line);
                        self.lines.set(lines);
                        self.post_changed(ctx);
                    }
                }
            }
            "newline" => {
                let row = self.cursor_row.get();
                let col = self.cursor_col.get();
                let mut lines = self.lines.get_untracked();
                let rest = if let Some(line) = lines.get_mut(row) {
                    let byte_pos = line
                        .char_indices()
                        .nth(col)
                        .map(|(i, _)| i)
                        .unwrap_or(line.len());
                    line.split_off(byte_pos)
                } else {
                    String::new()
                };
                lines.insert(row + 1, rest);
                self.lines.set(lines);
                self.cursor_row.set(row + 1);
                self.cursor_col.set(0);
                self.post_changed(ctx);
            }
            "copy" => {
                // Try to copy current line to clipboard. Fail silently if clipboard unavailable.
                let lines = self.lines.get_untracked();
                let row = self.cursor_row.get();
                if let Some(line) = lines.get(row) {
                    let text = line.clone();
                    let _ = (|| -> anyhow::Result<()> {
                        let mut clipboard = arboard::Clipboard::new()?;
                        clipboard.set_text(text)?;
                        Ok(())
                    })();
                }
            }
            "paste" => {
                // Try to paste from clipboard. Fail silently if clipboard unavailable.
                let result = (|| -> anyhow::Result<String> {
                    let mut clipboard = arboard::Clipboard::new()?;
                    Ok(clipboard.get_text()?)
                })();
                if let Ok(text) = result {
                    let parts: Vec<&str> = text.split('\n').collect();
                    if parts.is_empty() {
                        return;
                    }
                    let row = self.cursor_row.get();
                    let col = self.cursor_col.get();
                    if parts.len() == 1 {
                        // Single line paste — insert inline
                        self.lines.update(|lines| {
                            if let Some(line) = lines.get_mut(row) {
                                let byte_pos = line
                                    .char_indices()
                                    .nth(col)
                                    .map(|(i, _)| i)
                                    .unwrap_or(line.len());
                                line.insert_str(byte_pos, parts[0]);
                            }
                        });
                        self.cursor_col.set(col + parts[0].len());
                    } else {
                        // Multi-line paste
                        let mut lines = self.lines.get_untracked();
                        let byte_pos = lines[row]
                            .char_indices()
                            .nth(col)
                            .map(|(i, _)| i)
                            .unwrap_or(lines[row].len());
                        let suffix = lines[row].split_off(byte_pos);
                        // Append first part to current line
                        lines[row].push_str(parts[0]);
                        // Insert middle parts as new lines
                        for (i, part) in parts[1..parts.len() - 1].iter().enumerate() {
                            lines.insert(row + 1 + i, part.to_string());
                        }
                        // Last part + suffix
                        let last_idx = row + parts.len() - 1;
                        let mut last_line = parts[parts.len() - 1].to_string();
                        last_line.push_str(&suffix);
                        lines.insert(last_idx, last_line);
                        self.cursor_row.set(last_idx);
                        self.cursor_col.set(parts[parts.len() - 1].len());
                        self.lines.set(lines);
                    }
                    self.post_changed(ctx);
                }
            }
            "select_all" => {
                // No-op for v1 — selection via Shift+arrow deferred to later refinement
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let lines = self.lines.get_untracked();
        let cursor_row = self.cursor_row.get();
        let cursor_col = self.cursor_col.get();

        // Auto-scroll: adjust scroll_offset so cursor stays visible
        let mut scroll = self.scroll_offset.get();
        if cursor_row < scroll {
            scroll = cursor_row;
            self.scroll_offset.set(scroll);
        } else if cursor_row >= scroll + area.height as usize {
            scroll = cursor_row + 1 - area.height as usize;
            self.scroll_offset.set(scroll);
        }

        let margin = self.margin_width();
        let text_width = area.width.saturating_sub(margin as u16) as usize;

        for row_offset in 0..area.height {
            let line_idx = scroll + row_offset as usize;
            if line_idx >= lines.len() {
                break;
            }
            let y = area.y + row_offset;

            // Render line number if enabled
            if self.show_line_numbers && margin > 0 {
                let num_str = format!("{:>width$} ", line_idx + 1, width = margin - 1);
                let display: String = num_str.chars().take(margin).collect();
                buf.set_string(area.x, y, &display, Style::default());
            }

            let line = &lines[line_idx];
            let text_x = area.x + margin as u16;

            // Render line content
            let display: String = line.chars().take(text_width).collect();
            buf.set_string(text_x, y, &display, Style::default());

            // Render cursor on the cursor row
            if line_idx == cursor_row {
                let cx = text_x + cursor_col.min(text_width.saturating_sub(1)) as u16;
                if cx < area.x + area.width {
                    let current_char = line.chars().nth(cursor_col).unwrap_or(' ');
                    let sym = current_char.to_string();
                    buf.set_string(cx, y, &sym, Style::default().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}
