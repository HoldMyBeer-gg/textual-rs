/// IRC client layout demo — weechat-style TUI using the textual-rs built-in widget library.
///
/// Layout:
/// ┌─────────────────────── Header (row 0) ────────────────────────┐
/// ├───────────┬─────────────────────────────────────┬─────────────┤
/// │ ChannelPane│         ChatLog (flex-grow)         │  UserPane   │
/// │ (20 cols) │                                      │  (22 cols)  │
/// ├───────────┴─────────────────────────────────────┴─────────────┤
/// │                   InputRegion (3 rows)                        │
/// └───────────────────────────────────────────────────────────────┘
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use textual_rs::{
    App, Widget,
    Header, Footer,
    ListView, Log, Input,
};
use textual_rs::widget::context::AppContext;

// ---- Widget wrappers ----

/// Sidebar showing channel list. CSS selector "ChannelPane" controls width.
struct ChannelPane;

impl Widget for ChannelPane {
    fn widget_type_name(&self) -> &'static str {
        "ChannelPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "#general".to_string(),
            "#rust".to_string(),
            "#tui-dev".to_string(),
            "#help".to_string(),
            "#off-topic".to_string(),
            "#announcements".to_string(),
            "#code-review".to_string(),
        ]))]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

/// Main chat log area. CSS selector "ChatLog" takes flex-grow: 1.
struct ChatLog;

impl Widget for ChatLog {
    fn widget_type_name(&self) -> &'static str {
        "ChatLog"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[12:01] <alice>  hey everyone, just pushed the new layout engine".to_string());
        log.push_line("[12:02] <bob>    nice! does it handle flex and grid?".to_string());
        log.push_line("[12:02] <alice>  yep, taffy does the heavy lifting. flex-grow, fixed widths, the works".to_string());
        log.push_line("[12:03] <carol>  what about dock layout? i need top/bottom bars".to_string());
        log.push_line("[12:03] <alice>  dock:top and dock:bottom both work -- check the header and input bar".to_string());
        log.push_line("[12:04] <dave>   just pulled. the CSS cascade is slick -- specificity ordering and all".to_string());
        log.push_line("[12:05] <bob>    how's focus traversal?".to_string());
        log.push_line("[12:05] <alice>  Tab cycles through focusable widgets. try it".to_string());
        log.push_line("[12:06] <carol>  love the dark color palette".to_string());
        log.push_line("[12:07] <dave>   we should add :hover next. and mouse hit testing".to_string());
        log.push_line("[12:07] <bob>    one step at a time :) phase 4 is looking solid".to_string());
        log.push_line("[12:08] <alice>  agreed. phase 5 will polish the developer experience".to_string());
        log.push_line("[12:09] <carol>  can't wait. this is going to be a great TUI framework".to_string());
        log.push_line("[12:10] <erin>   just joined. what's textual-rs?".to_string());
        log.push_line("[12:10] <alice>  textual-rs: declare widgets, style with CSS, react to events".to_string());
        log.push_line("[12:11] <frank>  the 22-widget library is impressive. data table, sparkline, tree...".to_string());
        log.push_line("[12:12] <grace>  reactive state with signals too! this is Textual quality for Rust".to_string());
        log.push_line("[12:13] <alice>  let's do a demo recording this weekend".to_string());
        log.push_line("[12:14] <bob>    i'll prep the irc demo. it already looks great".to_string());
        log.push_line("[12:15] <carol>  the half-block borders really make a difference".to_string());
        vec![Box::new(log)]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

/// Sidebar showing user list. CSS selector "UserPane" controls width.
struct UserPane;

impl Widget for UserPane {
    fn widget_type_name(&self) -> &'static str {
        "UserPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "@alice [op]".to_string(),
            "@bob".to_string(),
            "@carol".to_string(),
            "@dave".to_string(),
            "erin".to_string(),
            "frank".to_string(),
            "grace".to_string(),
        ]))]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

/// Horizontal container for channel, chat, and user panes.
struct MainRegion;

impl Widget for MainRegion {
    fn widget_type_name(&self) -> &'static str {
        "MainRegion"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(ChannelPane),
            Box::new(ChatLog),
            Box::new(UserPane),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

/// Input bar at the bottom of the screen.
struct InputRegion;

impl Widget for InputRegion {
    fn widget_type_name(&self) -> &'static str {
        "InputRegion"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Input::new("Type a message..."))]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- Top-level screen ----

struct IrcScreen;

impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str {
        "IrcScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("textual-rs IRC").with_subtitle("weechat-style terminal client")),
            Box::new(MainRegion),
            Box::new(InputRegion),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- main ----

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(IrcScreen))
        .with_css(include_str!("irc_demo.tcss"));
    app.run()
}
