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
        use ratatui::style::{Color, Modifier};

        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = buf.cell((area.x, area.y)).map(|c| c.style()).unwrap_or_default();
        // Key badge: accent bg with dark text
        let key_style = base_style
            .fg(Color::Rgb(15, 15, 25))
            .bg(Color::Rgb(0, 212, 255))
            .add_modifier(Modifier::BOLD);
        // Description: muted text
        let desc_style = base_style.fg(Color::Rgb(120, 120, 140));

        // Collect key bindings as (key, description) pairs
        let mut bindings: Vec<(String, String)> = Vec::new();
        if let Some(focused_id) = ctx.focused_widget {
            if let Some(widget) = ctx.arena.get(focused_id) {
                for kb in widget.key_bindings().iter().filter(|kb| kb.show) {
                    bindings.push((format_key_code(&kb.key), kb.description.to_string()));
                }
            }
        }
        bindings.push(("Tab".to_string(), "Focus".to_string()));
        bindings.push(("Ctrl+P".to_string(), "Palette".to_string()));
        bindings.push(("q".to_string(), "Quit".to_string()));

        // Render each binding as: [key badge] description  [key badge] description ...
        let mut x = area.x + 1;
        for (key, desc) in &bindings {
            if x >= area.x + area.width {
                break;
            }
            // Key badge
            let key_text = format!(" {} ", key);
            let key_len = key_text.chars().count() as u16;
            if x + key_len >= area.x + area.width {
                break;
            }
            buf.set_string(x, area.y, &key_text, key_style);
            x += key_len;

            // Description
            let desc_text = format!(" {} ", desc);
            let _desc_len = desc_text.chars().count() as u16;
            let remaining = (area.x + area.width).saturating_sub(x);
            let display: String = desc_text.chars().take(remaining as usize).collect();
            buf.set_string(x, area.y, &display, desc_style);
            x += display.chars().count() as u16;

            // Separator space
            x += 1;
        }
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
