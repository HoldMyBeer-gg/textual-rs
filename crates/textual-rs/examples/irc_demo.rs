/// IRC client layout demo — exercises the full Phase 2 stack:
/// widget composition, TCSS styling, dock + flex layout, border rendering, Tab focus traversal.
///
/// Layout (80x24):
/// ┌─────────────────────── Header (row 0) ────────────────────────┐
/// ├──────────┬──────────────────────────────────────┬─────────────┤
/// │  Channel │          Chat area (flex-grow:1)     │  User List  │
/// │  List    │                                      │  (22 cols)  │
/// │ (18 cols)│                                      │             │
/// ├──────────┴──────────────────────────────────────┴─────────────┤
/// │                   Input bar (3 rows)                          │
/// └───────────────────────────────────────────────────────────────┘
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget as RatatuiWidget;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use textual_rs::widget::context::AppContext;
use textual_rs::{App, Widget};

// ---- Widget definitions ----

struct IrcScreen;

impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str {
        "IrcScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header),
            Box::new(MainRegion),
            Box::new(InputBar),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

struct Header;

impl Widget for Header {
    fn widget_type_name(&self) -> &'static str {
        "Header"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // Look up computed styles for colors
        // Find this widget's id by searching ctx.arena (we can't easily get our own id,
        // so we render with static styles for Phase 2)
        let _ = ctx;
        let style = Style::default()
            .fg(Color::Rgb(137, 180, 250))
            .bg(Color::Rgb(30, 30, 46));
        let para = Paragraph::new("#general — textual-rs IRC demo").style(style);
        RatatuiWidget::render(para, area, buf);
    }
}

struct MainRegion;

impl Widget for MainRegion {
    fn widget_type_name(&self) -> &'static str {
        "MainRegion"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(ChannelList),
            Box::new(ChatArea),
            Box::new(UserList),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

struct ChannelList;

impl Widget for ChannelList {
    fn widget_type_name(&self) -> &'static str {
        "ChannelList"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // Check if this widget is focused to choose border type
        // We find ourselves by scanning arena for ChannelList
        let focused = find_focused_widget_type(ctx, "ChannelList");
        let border_type = if focused {
            BorderType::Rounded
        } else {
            BorderType::Plain
        };
        let color = if focused {
            Color::Rgb(137, 180, 250)
        } else {
            Color::Reset
        };
        let block = Block::default()
            .title("Channels")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(color));
        let channels = vec!["#general", "#rust", "#tui", "#help"];
        let text = channels.join("\n");
        let para = Paragraph::new(text).block(block);
        RatatuiWidget::render(para, area, buf);
    }
}

struct ChatArea;

impl Widget for ChatArea {
    fn widget_type_name(&self) -> &'static str {
        "ChatArea"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("#general")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);
        let messages = vec![
            "alice: Hello from textual-rs!",
            "bob: Looking great!",
            "carol: Tab to cycle focus",
        ];
        let text = messages.join("\n");
        let para = Paragraph::new(text).block(block);
        RatatuiWidget::render(para, area, buf);
    }
}

struct UserList;

impl Widget for UserList {
    fn widget_type_name(&self) -> &'static str {
        "UserList"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Users")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);
        let users = vec!["alice", "bob", "carol", "dave"];
        let text = users.join("\n");
        let para = Paragraph::new(text).block(block);
        RatatuiWidget::render(para, area, buf);
    }
}

struct InputBar;

impl Widget for InputBar {
    fn widget_type_name(&self) -> &'static str {
        "InputBar"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let focused = find_focused_widget_type(ctx, "InputBar");
        let border_type = BorderType::Rounded;
        let color = if focused {
            Color::Rgb(137, 180, 250)
        } else {
            Color::Reset
        };
        let block = Block::default()
            .title("Message")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(color));
        let para = Paragraph::new("Type a message...").block(block);
        RatatuiWidget::render(para, area, buf);
    }
}

/// Helper: check if any focused widget has the given type name.
fn find_focused_widget_type(ctx: &AppContext, type_name: &str) -> bool {
    if let Some(focused_id) = ctx.focused_widget {
        if let Some(widget) = ctx.arena.get(focused_id) {
            return widget.widget_type_name() == type_name;
        }
    }
    false
}

// ---- TCSS Stylesheet ----

const IRC_STYLESHEET: &str = r#"
IrcScreen {
    layout-direction: vertical;
}
Header {
    dock: top;
    height: 1;
    background: rgb(30, 30, 46);
    color: rgb(137, 180, 250);
}
InputBar {
    dock: bottom;
    height: 3;
    border: rounded;
}
MainRegion {
    layout-direction: horizontal;
}
ChannelList {
    width: 18;
    border: solid;
}
ChatArea {
    flex-grow: 1;
    border: solid;
}
UserList {
    width: 22;
    border: solid;
}
ChannelList:focus {
    border: rounded;
    color: rgb(137, 180, 250);
}
InputBar:focus {
    border: rounded;
    color: rgb(137, 180, 250);
}
"#;

// ---- main ----

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(IrcScreen)).with_css(IRC_STYLESHEET);
    app.run()
}

// ---- Integration tests ----

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a fresh App with the IRC demo at 80x24 and return the rendered buffer.
    fn render_irc_80x24() -> (App, ratatui::buffer::Buffer) {
        let mut app = App::new(|| Box::new(IrcScreen)).with_css(IRC_STYLESHEET);
        let buf = app.render_to_test_backend(80, 24);
        (app, buf)
    }

    #[test]
    fn header_occupies_row_0_full_width() {
        let (app, _buf) = render_irc_80x24();
        // Header is docked top with height 1, so its rect should be y=0, height=1, width=80
        let screen_id = *app.ctx().screen_stack.last().unwrap();
        let header_id = find_widget_by_type(&app, screen_id, "Header");
        let rect = app.bridge().rect_for(header_id).expect("Header should have a rect");
        assert_eq!(rect.y, 0, "Header should be at row 0");
        assert_eq!(rect.height, 1, "Header should be 1 row tall");
        assert_eq!(rect.width, 80, "Header should be full width (80 cols)");
    }

    #[test]
    fn input_bar_occupies_bottom_3_rows() {
        let (app, _buf) = render_irc_80x24();
        let screen_id = *app.ctx().screen_stack.last().unwrap();
        let input_id = find_widget_by_type(&app, screen_id, "InputBar");
        let rect = app.bridge().rect_for(input_id).expect("InputBar should have a rect");
        assert_eq!(rect.height, 3, "InputBar should be 3 rows tall");
        assert_eq!(rect.y + rect.height, 24, "InputBar should end at row 24");
        assert_eq!(rect.width, 80, "InputBar should be full width");
    }

    #[test]
    fn channel_list_is_18_cols_wide_on_left() {
        let (app, _buf) = render_irc_80x24();
        let screen_id = *app.ctx().screen_stack.last().unwrap();
        let channel_id = find_widget_by_type(&app, screen_id, "ChannelList");
        let rect = app.bridge().rect_for(channel_id).expect("ChannelList should have a rect");
        assert_eq!(rect.width, 18, "ChannelList should be 18 cols wide");
        assert_eq!(rect.x, 0, "ChannelList should be on the left (x=0)");
    }

    #[test]
    fn user_list_is_22_cols_wide_on_right() {
        let (app, _buf) = render_irc_80x24();
        let screen_id = *app.ctx().screen_stack.last().unwrap();
        let user_id = find_widget_by_type(&app, screen_id, "UserList");
        let rect = app.bridge().rect_for(user_id).expect("UserList should have a rect");
        assert_eq!(rect.width, 22, "UserList should be 22 cols wide");
        assert_eq!(rect.x + rect.width, 80, "UserList should end at column 80");
    }

    #[test]
    fn chat_area_fills_remaining_width() {
        let (app, _buf) = render_irc_80x24();
        let screen_id = *app.ctx().screen_stack.last().unwrap();
        let channel_id = find_widget_by_type(&app, screen_id, "ChannelList");
        let user_id = find_widget_by_type(&app, screen_id, "UserList");
        let chat_id = find_widget_by_type(&app, screen_id, "ChatArea");

        let channel_rect = app.bridge().rect_for(channel_id).expect("ChannelList should have a rect");
        let user_rect = app.bridge().rect_for(user_id).expect("UserList should have a rect");
        let chat_rect = app.bridge().rect_for(chat_id).expect("ChatArea should have a rect");

        // Chat starts where ChannelList ends
        assert_eq!(chat_rect.x, channel_rect.x + channel_rect.width,
            "ChatArea should start immediately after ChannelList");
        // Chat ends where UserList starts
        assert_eq!(chat_rect.x + chat_rect.width, user_rect.x,
            "ChatArea should end immediately before UserList");
    }

    /// Helper: find a widget by type name in the widget tree (DFS from screen root).
    fn find_widget_by_type(app: &App, root: textual_rs::WidgetId, type_name: &str) -> textual_rs::WidgetId {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if let Some(widget) = app.ctx().arena.get(id) {
                if widget.widget_type_name() == type_name {
                    return id;
                }
            }
            if let Some(children) = app.ctx().children.get(id) {
                for &child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        panic!("Widget '{}' not found in tree", type_name);
    }
}
