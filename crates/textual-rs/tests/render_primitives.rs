//! Render-primitive verification tests.
//!
//! Each test targets one RENDER requirement:
//!   RENDER-01: McGugan box with eighth-block characters
//!   RENDER-02: Braille sparkline produces braille chars
//!   RENDER-03: Quadrant characters appear in Placeholder
//!   RENDER-04: Scrollbar uses eighth-block thumb positioning
//!   RENDER-05: vertical_gradient produces half-block cells

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use textual_rs::canvas;
use textual_rs::testing::TestApp;

// ---------------------------------------------------------------------------
// RENDER-01: McGugan Box
// ---------------------------------------------------------------------------

#[test]
fn mcgugan_box_uses_eighth_block_chars() {
    let area = Rect::new(0, 0, 8, 5);
    let mut buf = Buffer::empty(area);
    let border = Color::Rgb(200, 200, 200);
    let inside = Color::Rgb(30, 30, 40);
    let outside = Color::Rgb(0, 0, 0);

    let (ix, iy, iw, ih) = canvas::mcgugan_box(&mut buf, 0, 0, 8, 5, border, inside, outside);

    // Inner area is inset by 1 on each side
    assert_eq!((ix, iy, iw, ih), (1, 1, 6, 3));

    // Top edge: LOWER_ONE_EIGHTH with fg=border, bg=outside
    for x in 0..8 {
        let cell = buf.cell((x, 0)).unwrap();
        assert_eq!(cell.symbol(), canvas::LOWER_ONE_EIGHTH, "top edge x={x}");
        assert_eq!(cell.fg, border, "top fg x={x}");
        assert_eq!(cell.bg, outside, "top bg x={x}");
    }

    // Bottom edge: UPPER_ONE_EIGHTH with fg=border, bg=outside
    for x in 0..8 {
        let cell = buf.cell((x, 4)).unwrap();
        assert_eq!(cell.symbol(), canvas::UPPER_ONE_EIGHTH, "bottom edge x={x}");
        assert_eq!(cell.fg, border, "bottom fg x={x}");
        assert_eq!(cell.bg, outside, "bottom bg x={x}");
    }

    // Left edge (inner rows): LEFT_ONE_QUARTER with fg=border, bg=inside
    for y in 1..4 {
        let cell = buf.cell((0, y)).unwrap();
        assert_eq!(cell.symbol(), canvas::LEFT_ONE_QUARTER, "left edge y={y}");
        assert_eq!(cell.fg, border, "left fg y={y}");
        assert_eq!(cell.bg, inside, "left bg y={y}");
    }

    // Right edge (inner rows): RIGHT_ONE_QUARTER with fg=border, bg=inside
    for y in 1..4 {
        let cell = buf.cell((7, y)).unwrap();
        assert_eq!(cell.symbol(), canvas::RIGHT_ONE_QUARTER, "right edge y={y}");
        assert_eq!(cell.fg, border, "right fg y={y}");
        assert_eq!(cell.bg, inside, "right bg y={y}");
    }
}

// ---------------------------------------------------------------------------
// RENDER-02: Braille Sparkline
// ---------------------------------------------------------------------------

#[test]
fn sparkline_produces_braille_chars() {
    use textual_rs::Sparkline;

    // Multi-row sparkline uses braille rendering
    let test_app = TestApp::new(10, 3, || {
        Box::new(Sparkline::new(vec![1.0, 5.0, 3.0, 8.0, 2.0, 6.0, 4.0, 7.0]))
    });

    let buf = test_app.buffer();
    let mut braille_count = 0;
    for y in 0..3u16 {
        for x in 0..10u16 {
            if let Some(cell) = buf.cell((x, y)) {
                let sym = cell.symbol();
                if let Some(ch) = sym.chars().next() {
                    let cp = ch as u32;
                    if (0x2800..=0x28FF).contains(&cp) {
                        braille_count += 1;
                    }
                }
            }
        }
    }
    assert!(
        braille_count > 0,
        "Expected braille characters in sparkline, found {braille_count}"
    );
}

// ---------------------------------------------------------------------------
// RENDER-03: Quadrant Characters in Placeholder
// ---------------------------------------------------------------------------

#[test]
fn placeholder_uses_quadrant_chars() {
    use textual_rs::Placeholder;

    let test_app = TestApp::new(10, 5, || Box::new(Placeholder::new()));

    let buf = test_app.buffer();

    // Collect all unique quadrant chars (QUADRANT_CHARS[1..15], excluding space and full block)
    let quadrant_set: std::collections::HashSet<&str> =
        canvas::QUADRANT_CHARS[1..15].iter().copied().collect();

    let mut quadrant_count = 0;
    for y in 0..5u16 {
        for x in 0..10u16 {
            if let Some(cell) = buf.cell((x, y)) {
                if quadrant_set.contains(cell.symbol()) {
                    quadrant_count += 1;
                }
            }
        }
    }
    assert!(
        quadrant_count > 0,
        "Expected quadrant characters in Placeholder, found {quadrant_count}"
    );
}

// ---------------------------------------------------------------------------
// RENDER-04: Eighth-block scrollbar
// ---------------------------------------------------------------------------

#[test]
fn scrollbar_uses_eighth_block_thumb() {
    let area = Rect::new(0, 0, 1, 10);
    let mut buf = Buffer::empty(area);

    // content_size=100, viewport=20, position=30 => thumb should be visible
    canvas::vertical_scrollbar(
        &mut buf,
        0,
        0,
        10,
        100,
        20,
        30,
        Color::Rgb(180, 180, 200),
        Color::Rgb(40, 40, 50),
    );

    // At least one cell should contain a VERTICAL_BLOCKS character (partial fill edges)
    // or the thumb body (solid bg). Check for block chars in the thumb edge cells.
    let block_set: std::collections::HashSet<&str> =
        canvas::VERTICAL_BLOCKS.iter().copied().collect();

    let mut has_block = false;
    let mut has_thumb_bg = false;
    for y in 0..10u16 {
        if let Some(cell) = buf.cell((0, y)) {
            if block_set.contains(cell.symbol()) {
                has_block = true;
            }
            // Thumb body cells have bar_color as bg
            if cell.bg == Color::Rgb(180, 180, 200) {
                has_thumb_bg = true;
            }
        }
    }
    assert!(
        has_block || has_thumb_bg,
        "Expected scrollbar thumb with block chars or colored bg"
    );
}

// ---------------------------------------------------------------------------
// RENDER-05: vertical_gradient produces half-block cells
// ---------------------------------------------------------------------------

#[test]
fn vertical_gradient_produces_half_block_cells() {
    let area = Rect::new(0, 0, 5, 3);
    let mut buf = Buffer::empty(area);

    let top = Color::Rgb(255, 0, 0);
    let bottom = Color::Rgb(0, 0, 255);

    canvas::vertical_gradient(&mut buf, 0, 0, 5, 3, top, bottom);

    // All cells should contain the UPPER_HALF character
    for y in 0..3u16 {
        for x in 0..5u16 {
            let cell = buf.cell((x, y)).unwrap();
            assert_eq!(
                cell.symbol(),
                canvas::UPPER_HALF,
                "Expected half-block at ({x},{y})"
            );
        }
    }

    // Row 0 should have different fg/bg from row 2 (gradient interpolation)
    let top_cell = buf.cell((0, 0)).unwrap();
    let bot_cell = buf.cell((0, 2)).unwrap();
    assert_ne!(
        top_cell.fg, bot_cell.fg,
        "Top and bottom rows should have different fg (gradient)"
    );
    assert_ne!(
        top_cell.bg, bot_cell.bg,
        "Top and bottom rows should have different bg (gradient)"
    );

    // Top cell fg should be close to red, bottom cell bg should be close to blue
    assert_eq!(top_cell.fg, Color::Rgb(255, 0, 0), "Top fg should be pure red");
    assert_eq!(bot_cell.bg, Color::Rgb(0, 0, 255), "Bottom bg should be pure blue");
}

// ---------------------------------------------------------------------------
// OVERLAY TEST: Verify overlays don't erase underlying content
// ---------------------------------------------------------------------------

#[test]
fn overlay_preserves_underlying_screen_content() {
    use textual_rs::{Label, Widget};
    use textual_rs::widget::context::AppContext;
    use textual_rs::widget::context_menu::ContextMenuItem;
    use textual_rs::event::AppEvent;
    use crossterm::event::{MouseEvent, MouseEventKind, MouseButton, KeyModifiers as KMods};

    // Screen with visible text and a context menu
    struct TestScreen;
    impl Widget for TestScreen {
        fn widget_type_name(&self) -> &'static str { "TestScreen" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("VISIBLE_TEXT_HERE"))]
        }
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn context_menu_items(&self) -> Vec<ContextMenuItem> {
            vec![ContextMenuItem::new("Test Action", "test")]
        }
    }

    let css = "TestScreen { layout-direction: vertical; } Label { height: 1; }";
    let mut test_app = TestApp::new_styled(40, 10, css, || Box::new(TestScreen));

    // Verify label rendered
    let buf = test_app.buffer();
    let row0: String = (0..40u16).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(row0.contains("VISIBLE_TEXT_HERE"), "Before overlay: {:?}", row0.trim());

    // Right-click at row 5 (away from the label at row 0) to spawn context menu
    let right_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 5,
        row: 5,
        modifiers: KMods::NONE,
    };
    test_app.process_event(AppEvent::Mouse(right_click));

    // Check overlay is active
    assert!(test_app.ctx().active_overlay.borrow().is_some(), "Overlay should be active after right-click");

    let buf = test_app.buffer();

    // Debug: dump all rows
    let mut all_rows = String::new();
    for y in 0..10u16 {
        let row: String = (0..40u16).map(|x| buf[(x, y)].symbol().to_string()).collect();
        all_rows.push_str(&format!("  row {}: {:?}\n", y, row.trim_end()));
    }

    // Label at row 0 should survive (menu renders at row 5, not row 0)
    let row0: String = (0..40u16).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(row0.contains("VISIBLE_TEXT_HERE"),
        "Label should survive when menu is below it.\nBuffer:\n{}", all_rows);

    // Context menu should be visible somewhere in the lower rows
    let mut found_menu = false;
    for y in 4..10u16 {
        let row: String = (0..40u16).map(|x| buf[(x, y)].symbol().to_string()).collect();
        if row.contains("Test Action") {
            found_menu = true;
            break;
        }
    }
    assert!(found_menu, "Context menu should be visible.\nBuffer:\n{}", all_rows);
}
