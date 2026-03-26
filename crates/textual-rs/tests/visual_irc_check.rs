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
            "#help".into(), "#off-topic".into(),
        ]))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct ChatLog;
impl Widget for ChatLog {
    fn widget_type_name(&self) -> &'static str { "ChatLog" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[12:01] <alice> hey everyone".into());
        log.push_line("[12:02] <bob>   nice work!".into());
        log.push_line("[12:03] <carol> what about dock layout?".into());
        log.push_line("[12:04] <dave>  the CSS cascade is slick".into());
        log.push_line("[12:05] <alice> Tab cycles through widgets".into());
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
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Input::new("Type a message..."))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct IrcScreen;
impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str { "IrcScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("textual-rs IRC").with_subtitle("#general -- 5 users")),
            Box::new(Footer),
            Box::new(MainRegion),
            Box::new(InputRegion),
        ]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

#[test]
fn visual_irc_check() {
    let css = r#"
IrcScreen { layout-direction: vertical; background: rgb(10,10,15); color: rgb(224,224,224); }
Header { height: 1; background: rgb(18,18,26); color: rgb(0,255,163); }
Footer { height: 1; background: rgb(18,18,26); color: rgb(224,224,224); }
MainRegion { layout-direction: horizontal; flex-grow: 1; }
ChannelPane { width: 20; border: solid; }
ChatLog { flex-grow: 1; border: solid; }
UserPane { width: 18; border: solid; }
InputRegion { height: 3; }
Input { border: rounded; flex-grow: 1; height: 3; }
"#;
    let mut app = App::new(|| Box::new(IrcScreen)).with_css(css);
    let buf = app.render_to_test_backend(80, 20);
    
    for y in 0..20u16 {
        let mut line = String::new();
        for x in 0..80u16 {
            let cell = buf.cell((x, y)).unwrap();
            line.push_str(cell.symbol());
        }
        println!("{:2}|{}", y, line.trim_end());
    }
}
