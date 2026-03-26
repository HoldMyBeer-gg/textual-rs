use textual_rs::{App, Widget, Header, Footer, Button, ButtonVariant, Label, Input, Checkbox, TabbedContent};
use textual_rs::widget::context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct DemoScreen;
impl Widget for DemoScreen {
    fn widget_type_name(&self) -> &'static str { "DemoScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("textual-rs Demo").with_subtitle("test")),
            Box::new(TabbedContent::new(
                vec!["Controls".into()],
                vec![Box::new(ControlsPane)],
            )),
        ]
    }
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

struct ControlsPane;
impl Widget for ControlsPane {
    fn widget_type_name(&self) -> &'static str { "ControlsPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Form Controls")),
            Box::new(Input::new("Type here...")),
            Box::new(Checkbox::new("Enable", true)),
            Box::new(Button::new("Submit").with_variant(ButtonVariant::Primary)),
        ]
    }
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

#[test]
fn visual_render_check() {
    let css = r#"
DemoScreen { layout-direction: vertical; background: rgb(10,10,15); color: rgb(224,224,224); }
Header { dock: top; height: 1; background: rgb(18,18,26); color: rgb(0,212,255); }
TabbedContent { flex-grow: 1; }
ControlsPane { layout-direction: vertical; }
Button { border: heavy; height: 3; color: rgb(0,255,163); }
Input { border: rounded; height: 3; }
Label { height: 1; color: rgb(0,212,255); }
Checkbox { height: 1; }
"#;
    let mut app = App::new(|| Box::new(DemoScreen)).with_css(css);
    let buf = app.render_to_test_backend(60, 16);
    
    for y in 0..16u16 {
        let mut line = String::new();
        for x in 0..60u16 {
            let cell = buf.cell((x, y)).unwrap();
            line.push_str(cell.symbol());
        }
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            println!("{:2}|{}", y, trimmed);
        }
    }
    
    // Check that borders actually render
    let cell_0_0 = buf.cell((0, 0)).unwrap();
    println!("\nCell (0,0): symbol='{}' fg={:?} bg={:?}", cell_0_0.symbol(), cell_0_0.style().fg, cell_0_0.style().bg);
    
    // Check a button border cell (should be ┏ or similar)
    // Find any cell with border chars
    let mut found_border = false;
    for y in 0..16u16 {
        for x in 0..60u16 {
            let s = buf.cell((x, y)).unwrap().symbol();
            if s == "┏" || s == "╭" || s == "│" || s == "┃" {
                println!("Border char '{}' at ({}, {})", s, x, y);
                found_border = true;
                break;
            }
        }
        if found_border { break; }
    }
    
    assert!(found_border, "No border characters found in rendered output!");
}
