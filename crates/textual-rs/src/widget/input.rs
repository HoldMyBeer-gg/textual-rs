use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
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
            own_id: Cell::new(None),
        }
    }

    /// Enable password mode — characters are rendered as `*`.
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
        "Input { border: tall; height: 3; }"
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
            "cursor_left" => self.move_cursor_left(),
            "cursor_right" => self.move_cursor_right(),
            "cursor_home" => self.cursor_pos.set(0),
            "cursor_end" => self.cursor_pos.set(self.value_len()),
            "word_left" => self.move_word_left(),
            "word_right" => self.move_word_right(),
            "delete_back" => {
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
            "delete_forward" => {
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
                Style::default().add_modifier(Modifier::DIM),
            );
            return;
        }

        // Build display string
        let display_str: String = if self.password {
            "*".repeat(val.chars().count())
        } else {
            val.clone()
        };

        // Render characters, highlighting the cursor position
        let display_chars: Vec<char> = display_str.chars().collect();
        let val_bytes_len = val.len();

        // Convert byte offset cursor_pos to char index
        let cursor_char_idx = val[..pos.min(val_bytes_len)].chars().count();

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
            let style = if char_idx == cursor_char_idx && focused {
                if is_invalid {
                    Style::default().fg(Color::Red).add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(Modifier::REVERSED)
                }
            } else if is_invalid {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            buf.set_string(col, area.y, &ch.to_string(), style);
            col += 1;
        }

        // If cursor is at end of string (or string is empty), show cursor indicator
        if focused && cursor_char_idx >= display_chars.len() {
            let cursor_col = area.x + (cursor_char_idx - view_start).min(area.width as usize - 1) as u16;
            if cursor_col < area.x + area.width {
                let cursor_style = if is_invalid {
                    Style::default().fg(Color::Red).add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(Modifier::REVERSED)
                };
                buf.set_string(cursor_col, area.y, " ", cursor_style);
            }
        }
    }
}
