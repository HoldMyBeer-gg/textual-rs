use textual_rs::{App, Widget, Header, Footer, ListView, Log, Input};
use textual_rs::widget::context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct ChannelPane;
impl Widget for ChannelPane {
    fn widget_type_name(&self) -> &'static str { "ChannelPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec!["#general".into(), "#rust".into(), "#help".into()]))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct ChatLog;
impl Widget for ChatLog {
    fn widget_type_name(&self) -> &'static str { "ChatLog" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[12:01] <alice> hello".into());
        log.push_line("[12:02] <bob>   world".into());
        vec![Box::new(log)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}
struct UserPane;
impl Widget for UserPane {
    fn widget_type_name(&self) -> &'static str { "UserPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec!["@alice".into(), "@bob".into()]))]
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
struct IrcScreen;
impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str { "IrcScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Header::new("IRC")), Box::new(MainRegion)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

#[test]
fn debug_layout_rects() {
    let css = "IrcScreen { layout-direction: vertical; } Header { height: 1; } MainRegion { flex-grow: 1; layout-direction: horizontal; } ChannelPane { width: 20; border: solid; } ChatLog { flex-grow: 1; border: solid; } UserPane { width: 18; border: solid; } ListView { flex-grow: 1; } Log { flex-grow: 1; }";
    let mut app = App::new(|| Box::new(IrcScreen)).with_css(css);
    let buf = app.render_to_test_backend(80, 20);
    let ctx = app.ctx();
    let bridge = app.bridge();
    for (id, widget) in ctx.arena.iter() {
        let rect = bridge.rect_for(id);
        println!("{:?}: {} rect={:?}", id, widget.widget_type_name(), rect);
    }
    for y in 0..20u16 {
        let mut line = String::new();
        for x in 0..80u16 { line.push_str(buf.cell((x, y)).unwrap().symbol()); }
        let t = line.trim_end();
        if !t.is_empty() { println!("{:2}|{}", y, t); }
    }
}
