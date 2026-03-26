use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::context::AppContext;
use super::Widget;

/// A footer widget that reads the focused widget's key bindings and displays them.
///
/// Footer reads `ctx.focused_widget` and `ctx.arena[id].key_bindings()` during render,
/// filtering by `show == true`. This is safe as render holds only `&AppContext` (shared borrow).
pub struct Footer;

impl Widget for Footer {
    fn widget_type_name(&self) -> &'static str {
        "Footer"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Footer { height: 1; background: $primary; color: $text; }"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Read the focused widget's key bindings
        let binding_text = if let Some(focused_id) = ctx.focused_widget {
            if let Some(widget) = ctx.arena.get(focused_id) {
                let bindings: Vec<String> = widget
                    .key_bindings()
                    .iter()
                    .filter(|kb| kb.show)
                    .map(|kb| {
                        // Format key code as display string
                        let key_str = format_key_code(&kb.key);
                        format!(" {} {} ", key_str, kb.description)
                    })
                    .collect();
                bindings.join("  ")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if binding_text.is_empty() {
            return;
        }

        let display: String = binding_text.chars().take(area.width as usize).collect();
        let style = buf.cell((area.x, area.y)).map(|c| c.style()).unwrap_or_default();
        buf.set_string(area.x, area.y, &display, style);
    }
}

fn format_key_code(key: &crossterm::event::KeyCode) -> String {
    use crossterm::event::KeyCode;
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "Shift+Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Del".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => format!("{:?}", key),
    }
}
