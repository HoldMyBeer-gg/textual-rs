//! Format-constrained input widget using a mask template (e.g. date, phone number).
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use std::cell::{Cell, RefCell};

use super::context::AppContext;
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;

/// Messages emitted by a MaskedInput widget.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted on each keystroke with the current raw value.
    pub struct Changed {
        /// Current raw value (user-typed characters only, no separator literals).
        pub value: String,
    }
    impl Message for Changed {}

    /// Emitted when the user presses Enter.
    pub struct Submitted {
        /// Raw value at the time of submission.
        pub value: String,
    }
    impl Message for Submitted {}
}

/// Case transform directive applied to an input slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseMode {
    Upper,
    Lower,
}

/// A slot in the mask template. Separator slots are literals; all others accept user input.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MaskSlot {
    /// `#` — accepts ASCII digits only.
    Digit,
    /// `A` — accepts ASCII letters only.
    Letter,
    /// `a` — accepts ASCII letters or space.
    LetterOrBlank,
    /// `N` — accepts ASCII alphanumeric.
    Alphanumeric,
    /// `X` — accepts ASCII alphanumeric or space.
    AlphaOrBlank,
    /// Any other character — literal separator, not editable.
    Separator(char),
}

impl MaskSlot {
    fn is_input_slot(&self) -> bool {
        !matches!(self, MaskSlot::Separator(_))
    }
}

/// A format-constrained single-line text input widget.
///
/// The mask template (e.g. `##/##/####`) defines the fixed format. Input-slot
/// characters (`#`, `A`, `a`, `N`, `X`) accept user input; all other characters
/// are fixed separators that auto-insert at render time.
///
/// Cursor is tracked in *raw-value space* (index into the user-typed characters),
/// and the display column is derived at render time via a precomputed lookup table.
/// This means separator skipping is implicit — raw cursor movements never land on
/// separators.
///
/// # Example
/// ```no_run
/// use textual_rs::widget::masked_input::MaskedInput;
/// let input = MaskedInput::new("##/##/####");
/// ```
pub struct MaskedInput {
    /// The mask string as provided (e.g. `"##/##/####"`).
    pub mask: String,
    /// User-typed characters only — no separators.
    raw_value: RefCell<String>,
    /// Cursor position in raw-value space (0 = before first char).
    cursor_raw: Cell<usize>,
    /// Parsed mask slots (includes separator slots).
    mask_slots: Vec<MaskSlot>,
    /// For each input-slot index (0..N where N = number of non-separator slots),
    /// the display column index where that slot is rendered.
    slot_display_cols: Vec<usize>,
    /// Case transform per input slot (None, Some(Upper), Some(Lower)).
    slot_case: Vec<Option<CaseMode>>,
    own_id: Cell<Option<WidgetId>>,
}

impl MaskedInput {
    /// Create a new MaskedInput with the given mask template.
    ///
    /// Mask syntax:
    /// - `#` — digit slot
    /// - `A` — letter slot
    /// - `a` — letter-or-blank slot
    /// - `N` — alphanumeric slot
    /// - `X` — alphanumeric-or-blank slot
    /// - `>` — enable uppercase transform for subsequent slots (not a slot itself)
    /// - `<` — enable lowercase transform for subsequent slots (not a slot itself)
    /// - Any other character — literal separator (auto-inserted in display)
    pub fn new(mask: impl Into<String>) -> Self {
        let mask = mask.into();
        let (mask_slots, slot_display_cols, slot_case) = Self::parse_mask(&mask);
        Self {
            mask,
            raw_value: RefCell::new(String::new()),
            cursor_raw: Cell::new(0),
            mask_slots,
            slot_display_cols,
            slot_case,
            own_id: Cell::new(None),
        }
    }

    /// Parse the mask string into slot metadata.
    /// Returns (mask_slots, slot_display_cols, slot_case).
    fn parse_mask(mask: &str) -> (Vec<MaskSlot>, Vec<usize>, Vec<Option<CaseMode>>) {
        let mut slots: Vec<MaskSlot> = Vec::new();
        let mut display_cols: Vec<usize> = Vec::new(); // one entry per input slot
        let mut case_per_slot: Vec<Option<CaseMode>> = Vec::new();

        let mut current_case: Option<CaseMode> = None;
        let mut display_col: usize = 0;

        for ch in mask.chars() {
            match ch {
                '>' => {
                    current_case = Some(CaseMode::Upper);
                    // not a slot
                }
                '<' => {
                    current_case = Some(CaseMode::Lower);
                    // not a slot
                }
                '#' => {
                    display_cols.push(display_col);
                    case_per_slot.push(current_case);
                    slots.push(MaskSlot::Digit);
                    display_col += 1;
                }
                'A' => {
                    display_cols.push(display_col);
                    case_per_slot.push(current_case);
                    slots.push(MaskSlot::Letter);
                    display_col += 1;
                }
                'a' => {
                    display_cols.push(display_col);
                    case_per_slot.push(current_case);
                    slots.push(MaskSlot::LetterOrBlank);
                    display_col += 1;
                }
                'N' => {
                    display_cols.push(display_col);
                    case_per_slot.push(current_case);
                    slots.push(MaskSlot::Alphanumeric);
                    display_col += 1;
                }
                'X' => {
                    display_cols.push(display_col);
                    case_per_slot.push(current_case);
                    slots.push(MaskSlot::AlphaOrBlank);
                    display_col += 1;
                }
                sep => {
                    slots.push(MaskSlot::Separator(sep));
                    display_col += 1;
                }
            }
        }

        (slots, display_cols, case_per_slot)
    }

    /// Returns the raw user-typed value (no separators).
    pub fn value(&self) -> String {
        self.raw_value.borrow().clone()
    }

    /// Maximum number of characters the user can type (number of non-separator slots).
    fn max_raw_len(&self) -> usize {
        self.slot_display_cols.len()
    }

    /// Check if char `c` is accepted at input-slot index `slot_idx`.
    /// Returns `Some(transformed_char)` if accepted, `None` if rejected.
    fn accepts_char(&self, slot_idx: usize, c: char) -> Option<char> {
        let slot = self
            .mask_slots
            .iter()
            .filter(|s| s.is_input_slot())
            .nth(slot_idx)?;
        let accepted = match slot {
            MaskSlot::Digit => c.is_ascii_digit(),
            MaskSlot::Letter => c.is_ascii_alphabetic(),
            MaskSlot::LetterOrBlank => c.is_ascii_alphabetic() || c == ' ',
            MaskSlot::Alphanumeric => c.is_alphanumeric(),
            MaskSlot::AlphaOrBlank => c.is_alphanumeric() || c == ' ',
            MaskSlot::Separator(_) => false,
        };
        if !accepted {
            return None;
        }
        let transformed = match self.slot_case.get(slot_idx).copied().flatten() {
            Some(CaseMode::Upper) => {
                let mut u = c.to_uppercase();
                u.next().unwrap_or(c)
            }
            Some(CaseMode::Lower) => {
                let mut l = c.to_lowercase();
                l.next().unwrap_or(c)
            }
            None => c,
        };
        Some(transformed)
    }

    /// Build the full display string by interleaving raw_value chars with separators.
    /// Unfilled input slots are shown as `_`.
    fn build_display(&self) -> String {
        let raw = self.raw_value.borrow();
        let raw_chars: Vec<char> = raw.chars().collect();
        let mut raw_idx = 0usize;
        let mut display = String::new();

        for slot in &self.mask_slots {
            match slot {
                MaskSlot::Separator(sep) => {
                    display.push(*sep);
                }
                _ => {
                    if raw_idx < raw_chars.len() {
                        display.push(raw_chars[raw_idx]);
                        raw_idx += 1;
                    } else {
                        display.push('_');
                    }
                }
            }
        }
        display
    }

    /// Map a raw cursor position to the display column index (O(1) lookup).
    fn raw_pos_to_display_col(&self, raw_pos: usize) -> usize {
        if raw_pos < self.slot_display_cols.len() {
            self.slot_display_cols[raw_pos]
        } else if let Some(&last) = self.slot_display_cols.last() {
            last + 1
        } else {
            0
        }
    }

    fn emit_changed(&self, ctx: &AppContext) {
        if let Some(id) = self.own_id.get() {
            let val = self.value();
            ctx.post_message(id, messages::Changed { value: val });
        }
    }
}

static MASKED_INPUT_BINDINGS: &[KeyBinding] = &[
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
        key: KeyCode::Backspace,
        modifiers: KeyModifiers::NONE,
        action: "delete_back",
        description: "Delete backward",
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

impl Widget for MaskedInput {
    fn widget_type_name(&self) -> &'static str {
        "MaskedInput"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        MASKED_INPUT_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
            match key_event.code {
                KeyCode::Char(c)
                    if key_event.modifiers == KeyModifiers::NONE
                        || key_event.modifiers == KeyModifiers::SHIFT =>
                {
                    let cursor = self.cursor_raw.get();
                    if cursor < self.max_raw_len() {
                        if let Some(transformed) = self.accepts_char(cursor, c) {
                            self.raw_value.borrow_mut().insert(cursor, transformed);
                            self.cursor_raw.set(cursor + 1);
                            self.emit_changed(ctx);
                        }
                    }
                    return EventPropagation::Stop;
                }
                _ => {}
            }
        }
        EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "delete_back" => {
                let cursor = self.cursor_raw.get();
                if cursor > 0 {
                    self.raw_value.borrow_mut().remove(cursor - 1);
                    self.cursor_raw.set(cursor - 1);
                    self.emit_changed(ctx);
                }
            }
            "cursor_left" => {
                let cursor = self.cursor_raw.get();
                if cursor > 0 {
                    self.cursor_raw.set(cursor - 1);
                }
            }
            "cursor_right" => {
                let cursor = self.cursor_raw.get();
                let raw_len = self.raw_value.borrow().len();
                if cursor < raw_len {
                    self.cursor_raw.set(cursor + 1);
                }
            }
            "cursor_home" => {
                self.cursor_raw.set(0);
            }
            "cursor_end" => {
                let raw_len = self.raw_value.borrow().len();
                self.cursor_raw.set(raw_len);
            }
            "submit" => {
                if let Some(id) = self.own_id.get() {
                    let val = self.value();
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

        let base_style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        let focused = ctx.focused_widget == self.own_id.get();
        let display = self.build_display();
        let cursor_col = self.raw_pos_to_display_col(self.cursor_raw.get());

        let display_chars: Vec<char> = display.chars().collect();
        let raw = self.raw_value.borrow();
        let raw_len = raw.chars().count();
        drop(raw);

        // Which display chars are filled (not '_' placeholders)?
        // A char at display index i is filled if it corresponds to a raw char
        // (i.e., it's a separator or a filled input slot).
        let mut filled_up_to = 0usize; // count of filled input slots

        for (col_x, (disp_idx, &ch)) in (area.x..).zip(display_chars.iter().enumerate()) {
            if col_x >= area.x + area.width {
                break;
            }

            let is_separator =
                matches!(self.mask_slots.get(disp_idx), Some(MaskSlot::Separator(_)));

            let is_cursor_here = focused && disp_idx == cursor_col;

            let style = if is_cursor_here {
                base_style.add_modifier(Modifier::REVERSED)
            } else if ch == '_' {
                // Unfilled placeholder — dim
                base_style.add_modifier(Modifier::DIM)
            } else {
                base_style
            };

            if !is_separator {
                filled_up_to += 1;
            }

            buf.set_string(col_x, area.y, ch.to_string(), style);
        }

        // If cursor is past the last char (at end position), show cursor as a space
        if focused && cursor_col >= display_chars.len() {
            let cursor_x = (area.x + cursor_col as u16).min(area.x + area.width.saturating_sub(1));
            if cursor_x < area.x + area.width {
                buf.set_string(
                    cursor_x,
                    area.y,
                    " ",
                    base_style.add_modifier(Modifier::REVERSED),
                );
            }
        }

        let _ = (filled_up_to, raw_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::context::AppContext;

    fn make_ctx() -> AppContext {
        AppContext::new()
    }

    // ---- MaskSlot parsing ----

    #[test]
    fn parse_mask_slots_date() {
        // "##/##/####" -> [Digit, Digit, Sep('/'), Digit, Digit, Sep('/'), Digit, Digit, Digit, Digit]
        let input = MaskedInput::new("##/##/####");
        assert_eq!(
            input.mask_slots,
            vec![
                MaskSlot::Digit,
                MaskSlot::Digit,
                MaskSlot::Separator('/'),
                MaskSlot::Digit,
                MaskSlot::Digit,
                MaskSlot::Separator('/'),
                MaskSlot::Digit,
                MaskSlot::Digit,
                MaskSlot::Digit,
                MaskSlot::Digit,
            ]
        );
        assert_eq!(input.max_raw_len(), 8);
    }

    // ---- Char insertion ----

    #[test]
    fn insert_first_char() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        // simulate typing '1'
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        input.on_event(&key, &ctx);
        assert_eq!(input.value(), "1");
        assert_eq!(input.cursor_raw.get(), 1);
    }

    // ---- Separator skip ----

    #[test]
    fn two_digits_cursor_skips_separator() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        let k1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let k2 = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
        input.on_event(&k1, &ctx);
        input.on_event(&k2, &ctx);
        // raw_value should be "12", cursor_raw=2 (not onto '/' separator in display)
        assert_eq!(input.value(), "12");
        assert_eq!(input.cursor_raw.get(), 2);
        // display cursor should be at col 3 (after '/')
        assert_eq!(input.raw_pos_to_display_col(input.cursor_raw.get()), 3);
    }

    // ---- Rejection ----

    #[test]
    fn letter_rejected_in_digit_slot() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        input.on_event(&key, &ctx);
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor_raw.get(), 0);
    }

    // ---- Backspace ----

    #[test]
    fn backspace_removes_last_char() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        // Set raw_value = "123", cursor=3
        *input.raw_value.borrow_mut() = "123".to_string();
        input.cursor_raw.set(3);

        input.on_action("delete_back", &ctx);
        assert_eq!(input.value(), "12");
        assert_eq!(input.cursor_raw.get(), 2);
    }

    // ---- Arrow right skips separator ----

    #[test]
    fn cursor_right_in_raw_space() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        *input.raw_value.borrow_mut() = "12".to_string();
        input.cursor_raw.set(1);
        input.on_action("cursor_right", &ctx);
        // raw cursor moves from 1 to 2 — display col 3 (skipping '/')
        assert_eq!(input.cursor_raw.get(), 2);
        assert_eq!(input.raw_pos_to_display_col(2), 3);
    }

    // ---- value() returns raw chars only ----

    #[test]
    fn value_returns_raw_chars_only() {
        let input = MaskedInput::new("##/##/####");
        *input.raw_value.borrow_mut() = "12312024".to_string();
        assert_eq!(input.value(), "12312024");
    }

    // ---- build_display ----

    #[test]
    fn build_display_partial() {
        let input = MaskedInput::new("##/##/####");
        *input.raw_value.borrow_mut() = "1231".to_string();
        let display = input.build_display();
        assert_eq!(display, "12/31/____");
    }

    // ---- raw_pos_to_display_col ----

    #[test]
    fn raw_pos_to_display_col_mappings() {
        let input = MaskedInput::new("##/##/####");
        // Raw pos 0 -> display col 0
        assert_eq!(input.raw_pos_to_display_col(0), 0);
        // Raw pos 1 -> display col 1
        assert_eq!(input.raw_pos_to_display_col(1), 1);
        // Raw pos 2 -> display col 3 (after '/' at display col 2)
        assert_eq!(input.raw_pos_to_display_col(2), 3);
        // Raw pos 4 -> display col 6 (after second '/' at display col 5)
        assert_eq!(input.raw_pos_to_display_col(4), 6);
    }

    // ---- Case conversion ----

    #[test]
    fn uppercase_modifier_converts_letter() {
        let ctx = make_ctx();
        // ">AA" -> two uppercase letter slots
        let input = MaskedInput::new(">AA");
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        input.on_event(&key, &ctx);
        assert_eq!(input.value(), "A");
    }

    #[test]
    fn lowercase_modifier_converts_letter() {
        let ctx = make_ctx();
        // "<AA" -> two lowercase letter slots
        let input = MaskedInput::new("<AA");
        let key = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);
        input.on_event(&key, &ctx);
        assert_eq!(input.value(), "a");
    }

    // ---- Full mask completion ----

    #[test]
    fn full_mask_completion() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        for ch in ['1', '2', '3', '1', '2', '0', '2', '4'] {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            input.on_event(&key, &ctx);
        }
        assert_eq!(input.value(), "12312024");
        assert_eq!(input.cursor_raw.get(), 8);
    }

    // ---- Cursor home / end ----

    #[test]
    fn cursor_home_and_end() {
        let ctx = make_ctx();
        let input = MaskedInput::new("##/##/####");
        *input.raw_value.borrow_mut() = "1234".to_string();
        input.cursor_raw.set(4);
        input.on_action("cursor_home", &ctx);
        assert_eq!(input.cursor_raw.get(), 0);
        input.on_action("cursor_end", &ctx);
        assert_eq!(input.cursor_raw.get(), 4);
    }

    // ---- build_display with empty ----

    #[test]
    fn build_display_empty() {
        let input = MaskedInput::new("##/##/####");
        let display = input.build_display();
        assert_eq!(display, "__/__/____");
    }
}
