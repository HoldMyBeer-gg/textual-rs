use crate::widget::context::AppContext;
use crate::widget::WidgetId;

/// A discoverable command for the command palette.
#[derive(Clone)]
pub struct Command {
    /// Display name shown in the palette.
    pub name: String,
    /// Source widget type name or "app" for app-level commands.
    pub source: String,
    /// Keybinding display string (e.g., "Ctrl+S") or None.
    pub keybinding: Option<String>,
    /// The action string to dispatch when this command is selected.
    pub action: String,
    /// The widget ID to dispatch the action to, or None for app-level.
    pub target_id: Option<WidgetId>,
}

/// Registry for app-level commands and command discovery.
///
/// App-level commands can be registered via `register()`.
/// `discover_all()` returns both app-level commands and all visible
/// key bindings from widgets currently mounted in the widget tree.
pub struct CommandRegistry {
    /// App-level commands registered via register_command().
    app_commands: Vec<Command>,
}

impl CommandRegistry {
    /// Create a new empty CommandRegistry.
    pub fn new() -> Self {
        Self { app_commands: Vec::new() }
    }

    /// Register an app-level command (beyond widget key bindings).
    pub fn register(&mut self, name: &str, action: &str) {
        self.app_commands.push(Command {
            name: name.to_string(),
            source: "app".to_string(),
            keybinding: None,
            action: action.to_string(),
            target_id: None,
        });
    }

    /// Discover all commands: app-level + widget key bindings from mounted tree.
    /// Walks the arena collecting key_bindings() from every widget where show == true.
    pub fn discover_all(&self, ctx: &AppContext) -> Vec<Command> {
        let mut commands = self.app_commands.clone();

        // Walk all widgets in the arena
        for (id, widget) in ctx.arena.iter() {
            let bindings = widget.key_bindings();
            let widget_type = widget.widget_type_name();
            for binding in bindings {
                if !binding.show {
                    continue; // skip internal bindings
                }
                let key_str = format_keybinding(binding.key, binding.modifiers);
                commands.push(Command {
                    name: binding.description.to_string(),
                    source: widget_type.to_string(),
                    keybinding: Some(key_str),
                    action: binding.action.to_string(),
                    target_id: Some(id),
                });
            }
        }

        commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a KeyCode + KeyModifiers into a human-readable string like "Ctrl+S".
pub fn format_keybinding(
    key: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> String {
    use crossterm::event::{KeyCode, KeyModifiers};
    let key_str = match key {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => format!("{:?}", key),
    };
    let mut result = String::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str("Ctrl+");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str("Shift+");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        result.push_str("Alt+");
    }
    result.push_str(&key_str);
    result
}

/// Fuzzy match: returns a score (0.0 to 1.0) using Jaro-Winkler similarity.
/// Returns 1.0 for exact substring match.
pub fn fuzzy_score(query: &str, target: &str) -> f64 {
    if query.is_empty() {
        return 1.0;
    }
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();
    // Exact substring match gets top score
    if target_lower.contains(&query_lower) {
        return 1.0;
    }
    strsim::jaro_winkler(&query_lower, &target_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_score_empty_query_returns_one() {
        assert_eq!(fuzzy_score("", "anything"), 1.0);
    }

    #[test]
    fn fuzzy_score_substring_returns_one() {
        assert_eq!(fuzzy_score("save", "Save File"), 1.0);
    }

    #[test]
    fn fuzzy_score_case_insensitive() {
        assert_eq!(fuzzy_score("SAVE", "Save File"), 1.0);
    }

    #[test]
    fn fuzzy_score_no_match_returns_low() {
        let score = fuzzy_score("zzz", "Save File");
        assert!(score < 0.6, "Expected low score, got {}", score);
    }

    #[test]
    fn registry_register_adds_app_command() {
        let mut reg = CommandRegistry::new();
        reg.register("Save File", "save");
        let ctx = AppContext::new();
        let cmds = reg.discover_all(&ctx);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "Save File");
        assert_eq!(cmds[0].action, "save");
        assert_eq!(cmds[0].source, "app");
        assert!(cmds[0].target_id.is_none());
    }
}
