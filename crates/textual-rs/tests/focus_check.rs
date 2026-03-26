use textual_rs::{App, Widget, Button, Input};
use textual_rs::widget::context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct TestScreen;
impl Widget for TestScreen {
    fn widget_type_name(&self) -> &'static str { "TestScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Button::new("Click Me")),
            Box::new(Input::new("Type here...")),
        ]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

#[test]
fn focus_highlight_test() {
    let css = "TestScreen { layout-direction: vertical; background: rgb(10,10,15); } Button { border: rounded; height: 3; color: rgb(224,224,224); } Input { border: rounded; height: 3; }";
    let mut app = App::new(|| Box::new(TestScreen)).with_css(css);

    // First render — Tab to first focusable widget
    let _buf = app.render_to_test_backend(40, 8);

    // Simulate Tab to focus the first widget
    app.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    ));

    let buf = app.render_to_test_backend(40, 8);
    for y in 0..8u16 {
        let mut line = String::new();
        for x in 0..40u16 { line.push_str(buf.cell((x, y)).unwrap().symbol()); }
        println!("{:2}|{}", y, line.trim_end());
    }

    // Check the focused widget has heavy border (┏ instead of ╭)
    let top_left = buf.cell((0, 0)).unwrap().symbol();
    println!("\nFocused border char: '{}'", top_left);
    assert!(top_left == "┏", "Expected ┏ (heavy) for focused widget, got '{}'", top_left);

    // Check it's green
    let fg = buf.cell((0, 0)).unwrap().style().fg;
    println!("Focused border color: {:?}", fg);
}
