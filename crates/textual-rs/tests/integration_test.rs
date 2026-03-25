use ratatui::backend::TestBackend;
use ratatui::Terminal;
use textual_rs::App;

#[test]
fn test_render_hello() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    App::render_frame(&mut terminal).unwrap();

    let buffer = terminal.backend().buffer().clone();
    let content: String = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(
        content.contains("Hello from textual-rs!"),
        "Buffer should contain 'Hello from textual-rs!' but got: {}",
        &content[..200.min(content.len())]
    );
}

#[test]
fn test_render_has_title() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    App::render_frame(&mut terminal).unwrap();

    let buffer = terminal.backend().buffer().clone();
    let content: String = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(
        content.contains("textual-rs"),
        "Buffer should contain border title 'textual-rs'"
    );
}
