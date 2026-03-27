use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
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
    /// When Some, text between anchor (row, col) and cursor is selected.
    selection_anchor: Cell<Option<(usize, usize)>>,
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
            selection_anchor: Cell::new(None),
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
            selection_anchor: Cell::new(None),
            own_id: Cell::new(None),
        }
    }

    fn current_line_len(&self) -> usize {
        let lines = self.lines.get_untracked();
        let row = self.cursor_row.get();
        lines.get(row).map(|l| l.chars().count()).unwrap_or(0)
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
                self.cursor_col.set(lines[new_row].chars().count());
            }
            return;
        }
        let line = &lines[row];
        let chars: Vec<char> = line.chars().take(col).collect();
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
        let char_count = line.chars().count();
        if col >= char_count {
            if row < lines.len() - 1 {
                self.cursor_row.set(row + 1);
                self.cursor_col.set(0);
            }
            return;
        }
        let chars: Vec<char> = line.chars().skip(col).collect();
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

    // ---- Selection helpers ----

    /// Returns true if there is an active text selection.
    pub fn has_selection(&self) -> bool {
        if let Some((ar, ac)) = self.selection_anchor.get() {
            let cr = self.cursor_row.get();
            let cc = self.cursor_col.get();
            ar != cr || ac != cc
        } else {
            false
        }
    }

    /// Clear the selection anchor.
    fn clear_selection(&self) {
        self.selection_anchor.set(None);
    }

    /// Ensure the selection anchor is set to current cursor position if not already set.
    fn ensure_anchor(&self) {
        if self.selection_anchor.get().is_none() {
            self.selection_anchor.set(Some((self.cursor_row.get(), self.cursor_col.get())));
        }
    }

    /// Returns the normalized selection range as ((start_row, start_col), (end_row, end_col)).
    fn selected_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let (ar, ac) = self.selection_anchor.get()?;
        let cr = self.cursor_row.get();
        let cc = self.cursor_col.get();
        if ar == cr && ac == cc {
            return None;
        }
        let (start, end) = if (ar, ac) <= (cr, cc) {
            ((ar, ac), (cr, cc))
        } else {
            ((cr, cc), (ar, ac))
        };
        Some((start, end))
    }

    /// Returns the selected text as a String, joining multi-line selections with \n.
    fn selected_text(&self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selected_range()?;
        let lines = self.lines.get_untracked();
        if sr == er {
            // Single-line selection
            let line = &lines[sr];
            let chars: Vec<char> = line.chars().collect();
            let text: String = chars[sc..ec.min(chars.len())].iter().collect();
            return Some(text);
        }
        // Multi-line selection
        let mut result = Vec::new();
        // First line: from sc to end
        let first_chars: Vec<char> = lines[sr].chars().collect();
        result.push(first_chars[sc..].iter().collect::<String>());
        // Middle lines: full lines
        for row in (sr + 1)..er {
            result.push(lines[row].clone());
        }
        // Last line: from start to ec
        let last_chars: Vec<char> = lines[er].chars().collect();
        result.push(last_chars[..ec.min(last_chars.len())].iter().collect::<String>());
        Some(result.join("\n"))
    }

    /// Delete the selected text, merge boundary lines, update cursor, clear anchor.
    fn delete_selection(&self, ctx: &AppContext) {
        let ((sr, sc), (er, ec)) = match self.selected_range() {
            Some(r) => r,
            None => return,
        };
        let mut lines = self.lines.get_untracked();

        if sr == er {
            // Single-line deletion
            if let Some(line) = lines.get_mut(sr) {
                let chars: Vec<char> = line.chars().collect();
                let mut new_line = String::new();
                for (i, ch) in chars.iter().enumerate() {
                    if i < sc || i >= ec {
                        new_line.push(*ch);
                    }
                }
                *line = new_line;
            }
        } else {
            // Multi-line deletion
            // Keep chars before sc on start line
            let start_chars: Vec<char> = lines[sr].chars().collect();
            let prefix: String = start_chars[..sc].iter().collect();
            // Keep chars after ec on end line
            let end_chars: Vec<char> = lines[er].chars().collect();
            let suffix: String = end_chars[ec.min(end_chars.len())..].iter().collect();
            // Merge prefix + suffix
            let merged = format!("{}{}", prefix, suffix);
            // Remove lines from sr to er inclusive, replace with merged
            lines.drain(sr..=er);
            lines.insert(sr, merged);
        }

        self.lines.set(lines);
        self.cursor_row.set(sr);
        self.cursor_col.set(sc);
        self.selection_anchor.set(None);
        self.post_changed(ctx);
    }

    /// Returns true if a given (row, col) position is within the current selection.
    fn is_in_selection(&self, row: usize, col: usize) -> bool {
        if let Some(((sr, sc), (er, ec))) = self.selected_range() {
            if row < sr || row > er {
                return false;
            }
            if sr == er {
                return col >= sc && col < ec;
            }
            if row == sr {
                return col >= sc;
            }
            if row == er {
                return col < ec;
            }
            // Middle row -- fully selected
            true
        } else {
            false
        }
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
        key: KeyCode::Char('x'),
        modifiers: KeyModifiers::CONTROL,
        action: "cut",
        description: "Cut",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        action: "select_all",
        description: "Select all",
        show: false,
    },
    // Selection bindings
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::SHIFT,
        action: "select_up",
        description: "Select up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::SHIFT,
        action: "select_down",
        description: "Select down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::SHIFT,
        action: "select_left",
        description: "Select left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::SHIFT,
        action: "select_right",
        description: "Select right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::SHIFT,
        action: "select_home",
        description: "Select to line start",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::SHIFT,
        action: "select_end",
        description: "Select to line end",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        action: "select_word_left",
        description: "Select word left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        action: "select_word_right",
        description: "Select word right",
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
        "TextArea { border: inner; min-height: 5; }"
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

    fn context_menu_items(&self) -> Vec<super::context_menu::ContextMenuItem> {
        vec![
            super::context_menu::ContextMenuItem::new("Cut", "cut").with_shortcut("Ctrl+X"),
            super::context_menu::ContextMenuItem::new("Copy", "copy").with_shortcut("Ctrl+C"),
            super::context_menu::ContextMenuItem::new("Paste", "paste").with_shortcut("Ctrl+V"),
            super::context_menu::ContextMenuItem::new("Select All", "select_all").with_shortcut("Ctrl+A"),
        ]
    }

    fn has_text_selection(&self) -> bool {
        self.has_selection()
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
            // Handle character insertion -- only for plain chars (no Control/Alt, but allow Shift)
            let relevant_modifiers = key_event.modifiers & !KeyModifiers::SHIFT;
            if let KeyCode::Char(c) = key_event.code {
                if relevant_modifiers == KeyModifiers::NONE {
                    // Check it doesn't match a keybinding (those are handled by on_action)
                    let is_bound = TEXT_AREA_BINDINGS
                        .iter()
                        .any(|kb| kb.matches(key_event.code, key_event.modifiers));
                    if !is_bound {
                        // Delete selection first if any
                        if self.has_selection() {
                            self.delete_selection(ctx);
                        }
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
            // Movement actions -- clear selection
            "cursor_up" => {
                self.clear_selection();
                let row = self.cursor_row.get();
                if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    let line_len = lines[new_row].chars().count();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "cursor_down" => {
                self.clear_selection();
                let lines = self.lines.get_untracked();
                let row = self.cursor_row.get();
                if row < lines.len().saturating_sub(1) {
                    let new_row = row + 1;
                    self.cursor_row.set(new_row);
                    let line_len = lines[new_row].chars().count();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "cursor_left" => {
                self.clear_selection();
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                if col > 0 {
                    self.cursor_col.set(col - 1);
                } else if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    self.cursor_col.set(lines[new_row].chars().count());
                }
            }
            "cursor_right" => {
                self.clear_selection();
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
                self.clear_selection();
                self.cursor_col.set(0);
            }
            "cursor_end" => {
                self.clear_selection();
                self.cursor_col.set(self.current_line_len());
            }
            "word_left" => {
                self.clear_selection();
                self.word_left();
            }
            "word_right" => {
                self.clear_selection();
                self.word_right();
            }

            // Selection actions -- set anchor, then move cursor
            "select_up" => {
                self.ensure_anchor();
                let row = self.cursor_row.get();
                if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    let line_len = lines[new_row].chars().count();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "select_down" => {
                self.ensure_anchor();
                let lines = self.lines.get_untracked();
                let row = self.cursor_row.get();
                if row < lines.len().saturating_sub(1) {
                    let new_row = row + 1;
                    self.cursor_row.set(new_row);
                    let line_len = lines[new_row].chars().count();
                    self.cursor_col.set(self.cursor_col.get().min(line_len));
                }
            }
            "select_left" => {
                self.ensure_anchor();
                let col = self.cursor_col.get();
                let row = self.cursor_row.get();
                if col > 0 {
                    self.cursor_col.set(col - 1);
                } else if row > 0 {
                    let new_row = row - 1;
                    self.cursor_row.set(new_row);
                    let lines = self.lines.get_untracked();
                    self.cursor_col.set(lines[new_row].chars().count());
                }
            }
            "select_right" => {
                self.ensure_anchor();
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
            "select_home" => {
                self.ensure_anchor();
                self.cursor_col.set(0);
            }
            "select_end" => {
                self.ensure_anchor();
                self.cursor_col.set(self.current_line_len());
            }
            "select_word_left" => {
                self.ensure_anchor();
                self.word_left();
            }
            "select_word_right" => {
                self.ensure_anchor();
                self.word_right();
            }
            "select_all" => {
                self.selection_anchor.set(Some((0, 0)));
                let lines = self.lines.get_untracked();
                let last_row = lines.len().saturating_sub(1);
                let last_col = lines[last_row].chars().count();
                self.cursor_row.set(last_row);
                self.cursor_col.set(last_col);
            }

            // Editing actions
            "delete_back" => {
                if self.has_selection() {
                    self.delete_selection(ctx);
                } else {
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
                        let prev_len = lines[new_row].chars().count();
                        lines[new_row].push_str(&current_line);
                        self.lines.set(lines);
                        self.cursor_row.set(new_row);
                        self.cursor_col.set(prev_len);
                        self.post_changed(ctx);
                    }
                }
            }
            "delete_forward" => {
                if self.has_selection() {
                    self.delete_selection(ctx);
                } else {
                    let col = self.cursor_col.get();
                    let row = self.cursor_row.get();
                    let lines_snap = self.lines.get_untracked();
                    let line_len = lines_snap.get(row).map(|l| l.chars().count()).unwrap_or(0);
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
            }
            "newline" => {
                // Delete selection first if any
                if self.has_selection() {
                    self.delete_selection(ctx);
                }
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

            // Clipboard actions
            "copy" => {
                if let Some(text) = self.selected_text() {
                    let _ = (|| -> anyhow::Result<()> {
                        let mut clipboard = arboard::Clipboard::new()?;
                        clipboard.set_text(text)?;
                        Ok(())
                    })();
                } else {
                    // Fallback: copy current line if no selection
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
            }
            "cut" => {
                if let Some(text) = self.selected_text() {
                    let _ = (|| -> anyhow::Result<()> {
                        let mut clipboard = arboard::Clipboard::new()?;
                        clipboard.set_text(text)?;
                        Ok(())
                    })();
                    self.delete_selection(ctx);
                }
            }
            "paste" => {
                let result = (|| -> anyhow::Result<String> {
                    let mut clipboard = arboard::Clipboard::new()?;
                    Ok(clipboard.get_text()?)
                })();
                if let Ok(text) = result {
                    // Delete selection first if any
                    if self.has_selection() {
                        self.delete_selection(ctx);
                    }
                    let parts: Vec<&str> = text.split('\n').collect();
                    if parts.is_empty() {
                        return;
                    }
                    let row = self.cursor_row.get();
                    let col = self.cursor_col.get();
                    if parts.len() == 1 {
                        // Single line paste -- insert inline
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
                        self.cursor_col.set(col + parts[0].chars().count());
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
                        self.cursor_col.set(parts[parts.len() - 1].chars().count());
                        self.lines.set(lines);
                    }
                    self.post_changed(ctx);
                }
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

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

        let has_sel = self.has_selection();

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
                buf.set_string(area.x, y, &display, style);
            }

            let line = &lines[line_idx];
            let text_x = area.x + margin as u16;

            // Render line content with selection highlighting
            let chars: Vec<char> = line.chars().take(text_width).collect();
            for (col_idx, ch) in chars.iter().enumerate() {
                let cx = text_x + col_idx as u16;
                if cx >= area.x + area.width {
                    break;
                }

                let in_sel = has_sel && self.is_in_selection(line_idx, col_idx);
                let is_cursor = line_idx == cursor_row && col_idx == cursor_col;

                let char_style = if in_sel || is_cursor {
                    style.add_modifier(Modifier::REVERSED)
                } else {
                    style
                };
                buf.set_string(cx, y, ch.to_string(), char_style);
            }

            // Render cursor at end of line if cursor is past content
            if line_idx == cursor_row && cursor_col >= chars.len() {
                let cx = text_x + cursor_col.min(text_width.saturating_sub(1)) as u16;
                if cx < area.x + area.width {
                    let current_char = ' ';
                    let sym = current_char.to_string();
                    buf.set_string(cx, y, &sym, style.add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::context::AppContext;

    fn make_textarea(text: &str) -> (TextArea, AppContext) {
        let ta = TextArea::new();
        let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        let last_row = lines.len().saturating_sub(1);
        let last_col = lines[last_row].chars().count();
        ta.lines.set(lines);
        ta.cursor_row.set(last_row);
        ta.cursor_col.set(last_col);
        let ctx = AppContext::new();
        (ta, ctx)
    }

    #[test]
    fn text_area_selection_starts_none() {
        let (ta, _ctx) = make_textarea("hello\nworld");
        assert!(!ta.has_selection());
        assert!(ta.selected_range().is_none());
        assert!(ta.selected_text().is_none());
    }

    #[test]
    fn text_area_select_left() {
        let (ta, ctx) = make_textarea("hello");
        // cursor at (0, 5)
        ta.on_action("select_left", &ctx);
        assert!(ta.has_selection());
        assert_eq!(ta.selection_anchor.get(), Some((0, 5)));
        assert_eq!(ta.cursor_col.get(), 4);
        assert_eq!(ta.selected_text().unwrap(), "o");
    }

    #[test]
    fn text_area_select_right() {
        let (ta, ctx) = make_textarea("hello");
        ta.cursor_col.set(0);
        ta.on_action("select_right", &ctx);
        assert!(ta.has_selection());
        assert_eq!(ta.selected_text().unwrap(), "h");
    }

    #[test]
    fn text_area_select_all() {
        let (ta, ctx) = make_textarea("hello\nworld");
        ta.cursor_row.set(0);
        ta.cursor_col.set(0);
        ta.on_action("select_all", &ctx);
        assert!(ta.has_selection());
        assert_eq!(ta.selected_text().unwrap(), "hello\nworld");
    }

    #[test]
    fn text_area_multiline_selection() {
        let (ta, _ctx) = make_textarea("hello\nworld\nfoo");
        ta.selection_anchor.set(Some((0, 3)));
        ta.cursor_row.set(1);
        ta.cursor_col.set(2);
        assert!(ta.has_selection());
        assert_eq!(ta.selected_text().unwrap(), "lo\nwo");
    }

    #[test]
    fn text_area_delete_selection_single_line() {
        let (ta, ctx) = make_textarea("hello world");
        ta.cursor_row.set(0);
        ta.selection_anchor.set(Some((0, 0)));
        ta.cursor_col.set(5);
        ta.delete_selection(&ctx);
        assert_eq!(ta.lines.get_untracked(), vec![" world"]);
        assert_eq!(ta.cursor_col.get(), 0);
        assert!(!ta.has_selection());
    }

    #[test]
    fn text_area_delete_selection_multiline() {
        let (ta, ctx) = make_textarea("hello\nworld\nfoo");
        ta.selection_anchor.set(Some((0, 3)));
        ta.cursor_row.set(2);
        ta.cursor_col.set(1);
        ta.delete_selection(&ctx);
        // Should become "heloo"
        assert_eq!(ta.lines.get_untracked(), vec!["heloo"]);
        assert_eq!(ta.cursor_row.get(), 0);
        assert_eq!(ta.cursor_col.get(), 3);
    }

    #[test]
    fn text_area_cursor_movement_clears_selection() {
        let (ta, ctx) = make_textarea("hello");
        ta.selection_anchor.set(Some((0, 0)));
        ta.cursor_col.set(3);
        assert!(ta.has_selection());
        ta.on_action("cursor_left", &ctx);
        assert!(!ta.has_selection());
    }

    #[test]
    fn text_area_select_up_down() {
        let (ta, ctx) = make_textarea("hello\nworld");
        // cursor at (1, 5)
        ta.on_action("select_up", &ctx);
        assert!(ta.has_selection());
        assert_eq!(ta.selection_anchor.get(), Some((1, 5)));
        assert_eq!(ta.cursor_row.get(), 0);
        assert_eq!(ta.cursor_col.get(), 5);
        assert_eq!(ta.selected_text().unwrap(), "\nworld");

        // Select down from current position
        ta.on_action("select_down", &ctx);
        assert_eq!(ta.cursor_row.get(), 1);
        // Anchor unchanged, cursor back to row 1 -- no selection (anchor == cursor)
        assert!(!ta.has_selection());
    }

    #[test]
    fn text_area_select_home_end() {
        let (ta, ctx) = make_textarea("hello");
        ta.cursor_col.set(3);
        ta.on_action("select_home", &ctx);
        assert_eq!(ta.selected_text().unwrap(), "hel");

        ta.selection_anchor.set(None);
        ta.on_action("select_end", &ctx);
        assert_eq!(ta.selected_text().unwrap(), "hello");
    }

    #[test]
    fn text_area_delete_back_with_selection() {
        let (ta, ctx) = make_textarea("hello world");
        ta.cursor_row.set(0);
        ta.selection_anchor.set(Some((0, 5)));
        ta.cursor_col.set(11);
        ta.on_action("delete_back", &ctx);
        assert_eq!(ta.lines.get_untracked(), vec!["hello"]);
        assert!(!ta.has_selection());
    }

    #[test]
    fn text_area_is_in_selection() {
        let (ta, _ctx) = make_textarea("hello\nworld");
        ta.selection_anchor.set(Some((0, 2)));
        ta.cursor_row.set(1);
        ta.cursor_col.set(3);
        // (0, 0) should NOT be in selection
        assert!(!ta.is_in_selection(0, 0));
        // (0, 2) should be in selection
        assert!(ta.is_in_selection(0, 2));
        // (0, 4) should be in selection (middle of first line selected part)
        assert!(ta.is_in_selection(0, 4));
        // (1, 0) should be in selection (start of second line)
        assert!(ta.is_in_selection(1, 0));
        // (1, 2) should be in selection
        assert!(ta.is_in_selection(1, 2));
        // (1, 3) should NOT be in selection (end is exclusive)
        assert!(!ta.is_in_selection(1, 3));
    }
}
