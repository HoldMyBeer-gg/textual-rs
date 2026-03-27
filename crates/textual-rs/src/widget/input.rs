use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::context::AppContext;
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by an Input widget.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted on each keystroke with the current value.
    pub struct Changed {
        pub value: String,
        pub valid: bool,
    }
    impl Message for Changed {}

    /// Emitted when the user presses Enter.
    pub struct Submitted {
        pub value: String,
    }
    impl Message for Submitted {}
}

/// A single-line text input widget with cursor, placeholder, password mode, and validation.
///
/// Emits `messages::Changed` on each keystroke and `messages::Submitted` on Enter.
pub struct Input {
    pub value: Reactive<String>,
    pub placeholder: String,
    pub password: bool,
    validator: Option<Box<dyn Fn(&str) -> bool>>,
    valid: Cell<bool>,
    cursor_pos: Cell<usize>,
    /// When Some, text between anchor and cursor_pos is selected (byte offsets).
    selection_anchor: Cell<Option<usize>>,
    own_id: Cell<Option<WidgetId>>,
}

impl Input {
    /// Create a new Input with an optional placeholder string.
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: Reactive::new(String::new()),
            placeholder: placeholder.into(),
            password: false,
            validator: None,
            valid: Cell::new(true),
            cursor_pos: Cell::new(0),
            selection_anchor: Cell::new(None),
            own_id: Cell::new(None),
        }
    }

    /// Enable password mode -- characters are rendered as `*`.
    pub fn with_password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set a validator callback. Called on every change; returns `true` if valid.
    pub fn with_validator(mut self, f: impl Fn(&str) -> bool + 'static) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    /// Returns whether the current value passes validation.
    /// Always `true` if no validator has been set.
    pub fn is_valid(&self) -> bool {
        self.valid.get()
    }

    /// Returns true if there is an active text selection.
    pub fn has_selection(&self) -> bool {
        if let Some(anchor) = self.selection_anchor.get() {
            anchor != self.cursor_pos.get()
        } else {
            false
        }
    }

    /// Returns the selected range as (start, end) byte offsets, if any.
    fn selected_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor.get()?;
        let cursor = self.cursor_pos.get();
        if anchor == cursor {
            return None;
        }
        let start = anchor.min(cursor);
        let end = anchor.max(cursor);
        Some((start, end))
    }

    /// Returns the selected text substring, if any.
    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selected_range()?;
        let val = self.value.get_untracked();
        Some(val[start..end].to_string())
    }

    /// Delete the selected text, update cursor, clear anchor, emit Changed.
    fn delete_selection(&self, ctx: &AppContext) {
        if let Some((start, end)) = self.selected_range() {
            self.value.update(|v| {
                v.drain(start..end);
            });
            self.cursor_pos.set(start);
            self.selection_anchor.set(None);
            self.emit_changed(ctx);
        }
    }

    /// Clear the selection anchor (called on non-shift cursor movements).
    fn clear_selection(&self) {
        self.selection_anchor.set(None);
    }

    /// Ensure the selection anchor is set. If not yet set, set it to current cursor_pos.
    fn ensure_anchor(&self) {
        if self.selection_anchor.get().is_none() {
            self.selection_anchor.set(Some(self.cursor_pos.get()));
        }
    }

    /// Run the validator against the current value and update the validity state.
    fn run_validation(&self) {
        let is_valid = match &self.validator {
            Some(f) => f(&self.value.get_untracked()),
            None => true,
        };
        self.valid.set(is_valid);
    }

    // ---- cursor helpers ----

    fn value_len(&self) -> usize {
        self.value.get_untracked().len()
    }

    /// Move cursor left by one char boundary.
    fn move_cursor_left(&self) {
        let pos = self.cursor_pos.get();
        if pos == 0 {
            return;
        }
        let val = self.value.get_untracked();
        // Find the previous char boundary
        let mut new_pos = pos - 1;
        while new_pos > 0 && !val.is_char_boundary(new_pos) {
            new_pos -= 1;
        }
        self.cursor_pos.set(new_pos);
    }

    /// Move cursor right by one char boundary.
    fn move_cursor_right(&self) {
        let pos = self.cursor_pos.get();
        let val = self.value.get_untracked();
        if pos >= val.len() {
            return;
        }
        // Find the next char boundary
        let mut new_pos = pos + 1;
        while new_pos < val.len() && !val.is_char_boundary(new_pos) {
            new_pos += 1;
        }
        self.cursor_pos.set(new_pos);
    }

    /// Move cursor to the start of the previous word.
    fn move_word_left(&self) {
        let pos = self.cursor_pos.get();
        if pos == 0 {
            return;
        }
        let val = self.value.get_untracked();
        let bytes = val.as_bytes();
        // Skip whitespace/punctuation before the word
        let mut new_pos = pos;
        while new_pos > 0 && !bytes[new_pos - 1].is_ascii_alphanumeric() {
            new_pos -= 1;
        }
        // Move through the word itself
        while new_pos > 0 && bytes[new_pos - 1].is_ascii_alphanumeric() {
            new_pos -= 1;
        }
        self.cursor_pos.set(new_pos);
    }

    /// Move cursor to the start of the next word.
    fn move_word_right(&self) {
        let pos = self.cursor_pos.get();
        let val = self.value.get_untracked();
        let len = val.len();
        if pos >= len {
            return;
        }
        let bytes = val.as_bytes();
        let mut new_pos = pos;
        // Skip current word characters
        while new_pos < len && bytes[new_pos].is_ascii_alphanumeric() {
            new_pos += 1;
        }
        // Skip whitespace/punctuation to next word
        while new_pos < len && !bytes[new_pos].is_ascii_alphanumeric() {
            new_pos += 1;
        }
        self.cursor_pos.set(new_pos);
    }

    /// Insert a character at the current cursor position.
    fn insert_char(&self, c: char, ctx: &AppContext) {
        // If there's a selection, delete it first
        if self.has_selection() {
            self.delete_selection(ctx);
        }
        let pos = self.cursor_pos.get();
        let mut new_char_bytes = [0u8; 4];
        let encoded = c.encode_utf8(&mut new_char_bytes);
        self.value.update(|v| {
            v.insert_str(pos, encoded);
        });
        let char_len = c.len_utf8();
        self.cursor_pos.set(pos + char_len);
        self.emit_changed(ctx);
    }

    fn emit_changed(&self, ctx: &AppContext) {
        self.run_validation();
        if let Some(id) = self.own_id.get() {
            let val = self.value.get_untracked();
            ctx.post_message(id, messages::Changed { value: val, valid: self.valid.get() });
        }
    }
}

static INPUT_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "cursor_left",
        description: "Move cursor left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "cursor_right",
        description: "Move cursor right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "cursor_home",
        description: "Move to start",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "cursor_end",
        description: "Move to end",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::CONTROL,
        action: "word_left",
        description: "Jump word left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::CONTROL,
        action: "word_right",
        description: "Jump word right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Backspace,
        modifiers: KeyModifiers::NONE,
        action: "delete_back",
        description: "Delete backward",
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
        action: "submit",
        description: "Submit",
        show: false,
    },
    // Selection bindings
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
        description: "Select to start",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::SHIFT,
        action: "select_end",
        description: "Select to end",
        show: false,
    },
    // Ctrl+Shift selection uses CONTROL | SHIFT
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
    // Clipboard bindings
    KeyBinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::CONTROL,
        action: "select_all",
        description: "Select all",
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
        key: KeyCode::Char('x'),
        modifiers: KeyModifiers::CONTROL,
        action: "cut",
        description: "Cut",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('v'),
        modifiers: KeyModifiers::CONTROL,
        action: "paste",
        description: "Paste",
        show: false,
    },
];

impl Widget for Input {
    fn widget_type_name(&self) -> &'static str {
        "Input"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Input { border: inner; height: 3; }"
    }

    fn border_color_override(&self) -> Option<(u8, u8, u8)> {
        if !self.valid.get() && !self.value.get_untracked().is_empty() {
            Some((186, 60, 91)) // theme error color -- red border for invalid input
        } else {
            None
        }
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        INPUT_BINDINGS
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
            match key_event.code {
                KeyCode::Char(c)
                    if key_event.modifiers == KeyModifiers::NONE
                        || key_event.modifiers == KeyModifiers::SHIFT =>
                {
                    self.insert_char(c, ctx);
                    return EventPropagation::Stop;
                }
                _ => {}
            }
        }
        EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            // Movement actions -- clear selection
            "cursor_left" => {
                self.clear_selection();
                self.move_cursor_left();
            }
            "cursor_right" => {
                self.clear_selection();
                self.move_cursor_right();
            }
            "cursor_home" => {
                self.clear_selection();
                self.cursor_pos.set(0);
            }
            "cursor_end" => {
                self.clear_selection();
                self.cursor_pos.set(self.value_len());
            }
            "word_left" => {
                self.clear_selection();
                self.move_word_left();
            }
            "word_right" => {
                self.clear_selection();
                self.move_word_right();
            }

            // Selection actions -- set anchor, then move cursor
            "select_left" => {
                self.ensure_anchor();
                self.move_cursor_left();
            }
            "select_right" => {
                self.ensure_anchor();
                self.move_cursor_right();
            }
            "select_home" => {
                self.ensure_anchor();
                self.cursor_pos.set(0);
            }
            "select_end" => {
                self.ensure_anchor();
                self.cursor_pos.set(self.value_len());
            }
            "select_word_left" => {
                self.ensure_anchor();
                self.move_word_left();
            }
            "select_word_right" => {
                self.ensure_anchor();
                self.move_word_right();
            }
            "select_all" => {
                self.selection_anchor.set(Some(0));
                self.cursor_pos.set(self.value_len());
            }

            // Clipboard actions
            "copy" => {
                if let Some(text) = self.selected_text() {
                    let _ = (|| -> anyhow::Result<()> {
                        let mut clipboard = arboard::Clipboard::new()?;
                        clipboard.set_text(text)?;
                        Ok(())
                    })();
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
                    // Insert pasted text at cursor (single line -- strip newlines)
                    let clean_text: String = text.lines().collect::<Vec<_>>().join(" ");
                    let pos = self.cursor_pos.get();
                    self.value.update(|v| {
                        v.insert_str(pos, &clean_text);
                    });
                    self.cursor_pos.set(pos + clean_text.len());
                    self.emit_changed(ctx);
                }
            }

            // Editing actions
            "delete_back" => {
                if self.has_selection() {
                    self.delete_selection(ctx);
                } else {
                    let pos = self.cursor_pos.get();
                    if pos > 0 {
                        let val = self.value.get_untracked();
                        // Find the previous char boundary
                        let mut start = pos - 1;
                        while start > 0 && !val.is_char_boundary(start) {
                            start -= 1;
                        }
                        self.value.update(|v| {
                            v.drain(start..pos);
                        });
                        self.cursor_pos.set(start);
                        self.emit_changed(ctx);
                    }
                }
            }
            "delete_forward" => {
                if self.has_selection() {
                    self.delete_selection(ctx);
                } else {
                    let pos = self.cursor_pos.get();
                    let val = self.value.get_untracked();
                    if pos < val.len() {
                        // Find the next char boundary
                        let mut end = pos + 1;
                        while end < val.len() && !val.is_char_boundary(end) {
                            end += 1;
                        }
                        drop(val);
                        self.value.update(|v| {
                            v.drain(pos..end);
                        });
                        self.emit_changed(ctx);
                    }
                }
            }
            "submit" => {
                if let Some(id) = self.own_id.get() {
                    let val = self.value.get_untracked();
                    ctx.post_message(id, messages::Submitted { value: val });
                }
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        let val = self.value.get_untracked();
        let pos = self.cursor_pos.get();

        // Determine if focused
        let focused = ctx.focused_widget == self.own_id.get();

        if val.is_empty() && !focused {
            // Show placeholder in dim style
            let display: String = self.placeholder.chars().take(area.width as usize).collect();
            buf.set_string(
                area.x,
                area.y,
                &display,
                base_style.add_modifier(Modifier::DIM),
            );
            return;
        }

        // Build display string
        let display_str: String = if self.password {
            "*".repeat(val.chars().count())
        } else {
            val.clone()
        };

        // Render characters, highlighting the cursor position and selection
        let display_chars: Vec<char> = display_str.chars().collect();
        let val_bytes_len = val.len();

        // Convert byte offset cursor_pos to char index
        let cursor_char_idx = val[..pos.min(val_bytes_len)].chars().count();

        // Convert selection range to char indices (if any)
        let sel_char_range = self.selected_range().map(|(start_byte, end_byte)| {
            let start_char = val[..start_byte.min(val_bytes_len)].chars().count();
            let end_char = val[..end_byte.min(val_bytes_len)].chars().count();
            (start_char, end_char)
        });

        let max_cols = area.width as usize;
        // Scroll view if cursor is beyond display area
        let view_start = if cursor_char_idx >= max_cols {
            cursor_char_idx + 1 - max_cols
        } else {
            0
        };

        let is_invalid = !self.valid.get() && !val.is_empty();

        let mut col = area.x;
        for (char_idx, ch) in display_chars.iter().enumerate().skip(view_start) {
            if col >= area.x + area.width {
                break;
            }

            let in_selection = sel_char_range
                .map(|(s, e)| char_idx >= s && char_idx < e)
                .unwrap_or(false);

            let style = if in_selection {
                // Selected text gets REVERSED style
                if is_invalid {
                    base_style.fg(Color::Red).add_modifier(Modifier::REVERSED)
                } else {
                    base_style.add_modifier(Modifier::REVERSED)
                }
            } else if char_idx == cursor_char_idx && focused {
                if is_invalid {
                    base_style.fg(Color::Red).add_modifier(Modifier::REVERSED)
                } else {
                    base_style.add_modifier(Modifier::REVERSED)
                }
            } else if is_invalid {
                base_style.fg(Color::Red)
            } else {
                base_style
            };
            buf.set_string(col, area.y, ch.to_string(), style);
            col += 1;
        }

        // If cursor is at end of string (or string is empty), show cursor indicator
        if focused && cursor_char_idx >= display_chars.len() {
            let cursor_col = area.x + (cursor_char_idx - view_start).min(area.width as usize - 1) as u16;
            if cursor_col < area.x + area.width {
                let cursor_style = if is_invalid {
                    base_style.fg(Color::Red).add_modifier(Modifier::REVERSED)
                } else {
                    base_style.add_modifier(Modifier::REVERSED)
                };
                buf.set_string(cursor_col, area.y, " ", cursor_style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::context::AppContext;

    fn make_input(text: &str) -> (Input, AppContext) {
        let input = Input::new("placeholder");
        input.value.set(text.to_string());
        input.cursor_pos.set(text.len());
        let ctx = AppContext::new();
        (input, ctx)
    }

    #[test]
    fn input_selection_anchor_starts_none() {
        let (input, _ctx) = make_input("hello");
        assert!(!input.has_selection());
        assert!(input.selected_range().is_none());
        assert!(input.selected_text().is_none());
    }

    #[test]
    fn input_select_left_creates_selection() {
        let (input, ctx) = make_input("hello");
        // Cursor at end (5). Select left should set anchor=5, move cursor to 4.
        input.on_action("select_left", &ctx);
        assert!(input.has_selection());
        assert_eq!(input.selection_anchor.get(), Some(5));
        assert_eq!(input.cursor_pos.get(), 4);
        assert_eq!(input.selected_text().unwrap(), "o");
    }

    #[test]
    fn input_select_right_from_start() {
        let (input, ctx) = make_input("hello");
        input.cursor_pos.set(0);
        input.on_action("select_right", &ctx);
        assert!(input.has_selection());
        assert_eq!(input.selection_anchor.get(), Some(0));
        assert_eq!(input.cursor_pos.get(), 1);
        assert_eq!(input.selected_text().unwrap(), "h");
    }

    #[test]
    fn input_select_all() {
        let (input, ctx) = make_input("hello world");
        input.cursor_pos.set(3);
        input.on_action("select_all", &ctx);
        assert!(input.has_selection());
        assert_eq!(input.selected_range(), Some((0, 11)));
        assert_eq!(input.selected_text().unwrap(), "hello world");
    }

    #[test]
    fn input_select_home_and_end() {
        let (input, ctx) = make_input("hello");
        input.cursor_pos.set(3);
        input.on_action("select_home", &ctx);
        assert_eq!(input.selected_range(), Some((0, 3)));
        assert_eq!(input.selected_text().unwrap(), "hel");

        // Now select_end from position 0 (anchor stays at 3)
        input.selection_anchor.set(None);
        input.on_action("select_end", &ctx);
        assert_eq!(input.selected_range(), Some((0, 5)));
        assert_eq!(input.selected_text().unwrap(), "hello");
    }

    #[test]
    fn input_delete_selection() {
        let (input, ctx) = make_input("hello world");
        input.selection_anchor.set(Some(0));
        input.cursor_pos.set(5);
        // Selection is "hello"
        input.delete_selection(&ctx);
        assert_eq!(input.value.get_untracked(), " world");
        assert_eq!(input.cursor_pos.get(), 0);
        assert!(!input.has_selection());
    }

    #[test]
    fn input_char_replaces_selection() {
        let (input, ctx) = make_input("hello");
        input.selection_anchor.set(Some(0));
        input.cursor_pos.set(5);
        // Type 'X' -- should replace entire selection
        input.insert_char('X', &ctx);
        assert_eq!(input.value.get_untracked(), "X");
        assert_eq!(input.cursor_pos.get(), 1);
    }

    #[test]
    fn input_cursor_movement_clears_selection() {
        let (input, ctx) = make_input("hello");
        input.selection_anchor.set(Some(0));
        input.cursor_pos.set(3);
        assert!(input.has_selection());
        input.on_action("cursor_left", &ctx);
        assert!(!input.has_selection());
    }

    #[test]
    fn input_select_word_left() {
        let (input, ctx) = make_input("hello world");
        // cursor at end
        input.on_action("select_word_left", &ctx);
        assert!(input.has_selection());
        // Should select "world" (anchor=11, cursor at 6)
        assert_eq!(input.selection_anchor.get(), Some(11));
        assert_eq!(input.cursor_pos.get(), 6);
        assert_eq!(input.selected_text().unwrap(), "world");
    }

    #[test]
    fn input_select_word_right() {
        let (input, ctx) = make_input("hello world");
        input.cursor_pos.set(0);
        input.on_action("select_word_right", &ctx);
        assert!(input.has_selection());
        // Should select "hello " (anchor=0, cursor at 6)
        assert_eq!(input.selection_anchor.get(), Some(0));
        assert_eq!(input.cursor_pos.get(), 6);
        assert_eq!(input.selected_text().unwrap(), "hello ");
    }

    #[test]
    fn input_delete_back_with_selection() {
        let (input, ctx) = make_input("hello world");
        input.selection_anchor.set(Some(5));
        input.cursor_pos.set(11);
        // Selection is " world", backspace should delete it
        input.on_action("delete_back", &ctx);
        assert_eq!(input.value.get_untracked(), "hello");
        assert!(!input.has_selection());
    }

    #[test]
    fn input_delete_forward_with_selection() {
        let (input, ctx) = make_input("hello world");
        input.selection_anchor.set(Some(0));
        input.cursor_pos.set(6);
        // Selection is "hello ", delete should remove it
        input.on_action("delete_forward", &ctx);
        assert_eq!(input.value.get_untracked(), "world");
        assert!(!input.has_selection());
    }
}
