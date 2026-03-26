use textual_rs::{App, Widget, Header, Footer, ListView, Log, Input};
use textual_rs::widget::context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct ChannelPane;
impl Widget for ChannelPane {
    fn widget_type_name(&self) -> &'static str { "ChannelPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "#general".into(), "#rust".into(), "#tui-dev".into(),
            "#help".into(), "#off-topic".into(), "#announcements".into(), "#code-review".into(),
        ]))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct ChatLog;
impl Widget for ChatLog {
    fn widget_type_name(&self) -> &'static str { "ChatLog" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[12:01] <alice>  hey everyone, just pushed the new layout engine".into());
        log.push_line("[12:02] <bob>    nice! does it handle flex and grid?".into());
        log.push_line("[12:03] <carol>  what about dock layout?".into());
        log.push_line("[12:04] <dave>   the CSS cascade is slick".into());
        log.push_line("[12:05] <alice>  Tab cycles through focusable widgets".into());
        log.push_line("[12:06] <carol>  love the dark color palette".into());
        log.push_line("[12:07] <bob>    phase 4 is looking solid".into());
        vec![Box::new(log)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct UserPane;
impl Widget for UserPane {
    fn widget_type_name(&self) -> &'static str { "UserPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "@alice [op]".into(), "@bob".into(), "@carol".into(), "@dave".into(),
            "erin".into(), "frank".into(), "grace".into(),
        ]))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct MainRegion;
impl Widget for MainRegion {
    fn widget_type_name(&self) -> &'static str { "MainRegion" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ChannelPane), Box::new(ChatLog), Box::new(UserPane)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct InputRegion;
impl Widget for InputRegion {
    fn widget_type_name(&self) -> &'static str { "InputRegion" }
    fn compose(&self) -> Vec<Box<dyn Widget>> { vec![Box::new(Input::new("Type a message..."))] }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct IrcScreen;
impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str { "IrcScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("textual-rs IRC").with_subtitle("#general -- 7 users")),
            Box::new(MainRegion),
            Box::new(InputRegion),
            Box::new(Footer),
        ]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

#[test]
fn final_irc_visual() {
    let css = r#"
IrcScreen { layout-direction: vertical; background: rgb(10,10,15); color: rgb(224,224,224); }
Header { height: 1; background: rgb(18,18,26); color: rgb(0,255,163); }
Footer { height: 1; background: rgb(18,18,26); color: rgb(224,224,224); }
MainRegion { layout-direction: horizontal; flex-grow: 1; }
ChannelPane { width: 20; border: solid; color: rgb(224,224,224); }
ChatLog { flex-grow: 1; border: solid; color: rgb(0,255,163); }
UserPane { width: 20; border: solid; color: rgb(224,224,224); }
InputRegion { height: 3; }
Input { border: rounded; flex-grow: 1; height: 3; color: rgb(224,224,224); }
ListView { flex-grow: 1; } Log { flex-grow: 1; }
"#;
    let mut app = App::new(|| Box::new(IrcScreen)).with_css(css);
    let buf = app.render_to_test_backend(100, 24);
    for y in 0..24u16 {
        let mut line = String::new();
        for x in 0..100u16 { line.push_str(buf.cell((x, y)).unwrap().symbol()); }
        println!("{:2}|{}", y, line.trim_end());
    }
}
