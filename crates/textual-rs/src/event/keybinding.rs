//! Key binding declarations and matching for widget keyboard shortcut handling.

use crossterm::event::{KeyCode, KeyModifiers};

/// A key binding declaration on a widget.
/// Widgets return these from `key_bindings()` to declare keyboard shortcuts.
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// The key that triggers this binding.
    pub key: KeyCode,
    /// Required modifier keys (e.g., CONTROL, SHIFT). Use KeyModifiers::NONE for no modifiers.
    pub modifiers: KeyModifiers,
    /// Action string dispatched to on_action when this binding fires.
    pub action: &'static str,
    /// Human-readable description (shown in Footer widget).
    pub description: &'static str,
    /// Whether to display this binding in the Footer. Set false for internal bindings.
    pub show: bool,
}

impl KeyBinding {
    /// Check if a key event matches this binding.
    pub fn matches(&self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        self.key == key && self.modifiers == modifiers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn keybinding_holds_fields() {
        let kb = KeyBinding {
            key: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            action: "save",
            description: "Save the file",
            show: true,
        };
        assert_eq!(kb.key, KeyCode::Char('s'));
        assert_eq!(kb.modifiers, KeyModifiers::CONTROL);
        assert_eq!(kb.action, "save");
        assert_eq!(kb.description, "Save the file");
        assert!(kb.show);
    }

    #[test]
    fn matches_returns_true_for_exact_match() {
        let kb = KeyBinding {
            key: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            action: "save",
            description: "Save",
            show: true,
        };
        assert!(kb.matches(KeyCode::Char('s'), KeyModifiers::CONTROL));
    }

    #[test]
    fn matches_returns_false_for_wrong_key() {
        let kb = KeyBinding {
            key: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            action: "save",
            description: "Save",
            show: true,
        };
        assert!(!kb.matches(KeyCode::Char('x'), KeyModifiers::CONTROL));
    }

    #[test]
    fn matches_returns_false_for_wrong_modifier() {
        let kb = KeyBinding {
            key: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            action: "save",
            description: "Save",
            show: true,
        };
        assert!(!kb.matches(KeyCode::Char('s'), KeyModifiers::NONE));
    }

    #[test]
    fn matches_no_modifiers() {
        let kb = KeyBinding {
            key: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            action: "submit",
            description: "Submit",
            show: false,
        };
        assert!(kb.matches(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!kb.matches(KeyCode::Enter, KeyModifiers::SHIFT));
    }
}
