//! Visual regression test suite.
//!
//! Renders every major widget headlessly via TestApp and verifies the output
//! buffer contains expected characters, styles, and layout positions.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::Widget;
use textual_rs::{
    Button, Checkbox, ColumnDef, DataTable, Footer, Header, Label, ListView, Placeholder,
    RadioButton, Sparkline, Switch, TabbedContent, Tabs,
};

// ============================================================================
// Helpers
// ============================================================================

/// Extract a single row from the buffer as a String.
fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

/// Extract a single row, trimmed of trailing whitespace.
fn row_trimmed(buf: &Buffer, y: u16) -> String {
    row_text(buf, y).trim_end().to_string()
}

/// Check whether a cell has a specific modifier.
fn cell_has_modifier(buf: &Buffer, x: u16, y: u16, m: Modifier) -> bool {
    buf[(x, y)].modifier.contains(m)
}

/// Collect all unique symbols from a rectangular region.
fn symbols_in_region(buf: &Buffer, x: u16, y: u16, w: u16, h: u16) -> Vec<String> {
    let mut syms = Vec::new();
    for cy in y..y + h {
        for cx in x..x + w {
            if let Some(cell) = buf.cell((cx, cy)) {
                syms.push(cell.symbol().to_string());
            }
        }
    }
    syms
}

// ============================================================================
// 1. Horizontal layout -- 3 children side by side
// ============================================================================

#[test]
fn horizontal_layout_children_in_different_columns() {
    // Use layout-direction: horizontal on the screen itself to lay children side by side
    struct HScreen;
    impl Widget for HScreen {
        fn widget_type_name(&self) -> &'static str {
            "HScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(Label::new("AAA")),
                Box::new(Label::new("BBB")),
                Box::new(Label::new("CCC")),
            ]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = r#"
        HScreen { layout-direction: horizontal; }
        Label { width: 1fr; height: 1; }
    "#;
    let app = TestApp::new_styled(30, 3, css, || Box::new(HScreen));
    let buf = app.buffer();

    // All three labels should appear on row 0
    let row = row_text(buf, 0);
    assert!(
        row.contains("AAA"),
        "Row 0 should contain AAA, got: {:?}",
        row.trim_end()
    );
    assert!(
        row.contains("BBB"),
        "Row 0 should contain BBB, got: {:?}",
        row.trim_end()
    );
    assert!(
        row.contains("CCC"),
        "Row 0 should contain CCC, got: {:?}",
        row.trim_end()
    );

    // AAA should appear before BBB, and BBB before CCC
    let pos_a = row.find("AAA").unwrap();
    let pos_b = row.find("BBB").unwrap();
    let pos_c = row.find("CCC").unwrap();
    assert!(
        pos_a < pos_b,
        "AAA (col {}) should be left of BBB (col {})",
        pos_a,
        pos_b
    );
    assert!(
        pos_b < pos_c,
        "BBB (col {}) should be left of CCC (col {})",
        pos_b,
        pos_c
    );
}

// ============================================================================
// 2. Vertical layout -- children stacked on different rows
// ============================================================================

#[test]
fn vertical_layout_children_on_different_rows() {
    // Use layout-direction: vertical on the screen to stack children top-to-bottom
    struct VScreen;
    impl Widget for VScreen {
        fn widget_type_name(&self) -> &'static str {
            "VScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(Label::new("ROW_ONE")),
                Box::new(Label::new("ROW_TWO")),
                Box::new(Label::new("ROW_THREE")),
            ]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = r#"
        VScreen { layout-direction: vertical; }
        Label { height: 1; }
    "#;
    let app = TestApp::new_styled(20, 6, css, || Box::new(VScreen));
    let buf = app.buffer();

    // Find each label on different rows
    let mut row_one = None;
    let mut row_two = None;
    let mut row_three = None;
    for y in 0..6u16 {
        let text = row_text(buf, y);
        if text.contains("ROW_ONE") {
            row_one = Some(y);
        }
        if text.contains("ROW_TWO") {
            row_two = Some(y);
        }
        if text.contains("ROW_THREE") {
            row_three = Some(y);
        }
    }

    assert!(row_one.is_some(), "ROW_ONE not found in buffer");
    assert!(row_two.is_some(), "ROW_TWO not found in buffer");
    assert!(row_three.is_some(), "ROW_THREE not found in buffer");
    assert!(
        row_one.unwrap() < row_two.unwrap(),
        "ROW_ONE should be above ROW_TWO"
    );
    assert!(
        row_two.unwrap() < row_three.unwrap(),
        "ROW_TWO should be above ROW_THREE"
    );
}

// ============================================================================
// 3. Padding -- content shifts by padding amount
// ============================================================================

#[test]
fn padding_shifts_content_correctly() {
    struct PadScreen;
    impl Widget for PadScreen {
        fn widget_type_name(&self) -> &'static str {
            "PadScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("PADDED"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    // padding: 2 3 = top 2, right 3, bottom 2, left 3
    let css = "PadScreen { layout-direction: vertical; padding: 2 3; } Label { height: 1; }";
    let app = TestApp::new_styled(30, 8, css, || Box::new(PadScreen));
    let buf = app.buffer();

    // Rows 0 and 1 should NOT contain content (top padding = 2)
    for y in 0..2u16 {
        let text = row_text(buf, y);
        assert!(
            !text.contains("PADDED"),
            "Row {} should be empty (top padding), got: {:?}",
            y,
            text.trim_end()
        );
    }

    // Row 2 should contain content, shifted right by left padding = 3
    let row2 = row_text(buf, 2);
    assert!(
        row2.contains("PADDED"),
        "Row 2 should contain PADDED, got: {:?}",
        row2.trim_end()
    );
    let col = row2.find("PADDED").unwrap();
    assert!(
        col >= 3,
        "PADDED should start at col >= 3 (left padding), found at col {}",
        col
    );
}

// ============================================================================
// 4. Border types
// ============================================================================

#[test]
fn tall_border_uses_half_block_chars() {
    struct TallScreen;
    impl Widget for TallScreen {
        fn widget_type_name(&self) -> &'static str {
            "TallScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("TB"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "TallScreen { border: tall; height: 5; width: 10; } Label { height: 1; }";
    let app = TestApp::new_styled(12, 6, css, || Box::new(TallScreen));
    let buf = app.buffer();

    // Tall border chars: top = ▀, bottom = ▄, left = ▐, right = ▌
    let top_row = row_text(buf, 0);
    assert!(
        top_row.contains('▀'),
        "Top edge should contain ▀, got: {:?}",
        top_row.trim_end()
    );

    // Find bottom row (height 5 means row 4)
    let mut found_bottom = false;
    for y in 1..6u16 {
        let text = row_text(buf, y);
        if text.contains('▄') {
            found_bottom = true;
            break;
        }
    }
    assert!(found_bottom, "Bottom edge should contain ▄");

    // Side edges
    let mut found_left = false;
    let mut found_right = false;
    for y in 1..5u16 {
        let text = row_text(buf, y);
        if text.contains('▐') {
            found_left = true;
        }
        if text.contains('▌') {
            found_right = true;
        }
    }
    assert!(found_left, "Left edge should contain ▐");
    assert!(found_right, "Right edge should contain ▌");
}

#[test]
fn mcgugan_border_uses_eighth_block_chars() {
    struct McgScreen;
    impl Widget for McgScreen {
        fn widget_type_name(&self) -> &'static str {
            "McgScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("MC"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "McgScreen { border: mcgugan; height: 5; width: 10; } Label { height: 1; }";
    let app = TestApp::new_styled(12, 6, css, || Box::new(McgScreen));
    let buf = app.buffer();

    // McGugan chars: top = ▁ (U+2581), bottom = ▔ (U+2594), left = ▎ (U+258E)
    let all_syms: Vec<String> = symbols_in_region(buf, 0, 0, 12, 6);
    let all_text: String = all_syms.join("");
    assert!(
        all_text.contains('\u{2581}'),
        "McGugan top should use ▁ (U+2581)"
    );
    assert!(
        all_text.contains('\u{2594}'),
        "McGugan bottom should use ▔ (U+2594)"
    );
    assert!(
        all_text.contains('\u{258E}'),
        "McGugan left should use ▎ (U+258E)"
    );
}

#[test]
fn rounded_border_uses_round_corners() {
    struct RndScreen;
    impl Widget for RndScreen {
        fn widget_type_name(&self) -> &'static str {
            "RndScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("RN"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "RndScreen { border: rounded; height: 5; width: 10; } Label { height: 1; }";
    let app = TestApp::new_styled(12, 6, css, || Box::new(RndScreen));
    let buf = app.buffer();

    // Rounded corners: ╭ ╮ ╰ ╯
    let all_syms: Vec<String> = symbols_in_region(buf, 0, 0, 12, 6);
    let all_text: String = all_syms.join("");
    assert!(
        all_text.contains('╭'),
        "Should have top-left rounded corner ╭"
    );
    assert!(
        all_text.contains('╮'),
        "Should have top-right rounded corner ╮"
    );
    assert!(
        all_text.contains('╰'),
        "Should have bottom-left rounded corner ╰"
    );
    assert!(
        all_text.contains('╯'),
        "Should have bottom-right rounded corner ╯"
    );
}

// ============================================================================
// 5. Theme variable resolution -- $primary produces Rgb(1,120,212)
// ============================================================================

#[test]
fn theme_variable_primary_resolves_to_correct_rgb() {
    struct ThemeScreen;
    impl Widget for ThemeScreen {
        fn widget_type_name(&self) -> &'static str {
            "ThemeScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("THEMED"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    // Apply $primary as background so we can check bg color of cells
    let css =
        "ThemeScreen { layout-direction: vertical; background: $primary; } Label { height: 1; }";
    let app = TestApp::new_styled(20, 3, css, || Box::new(ThemeScreen));
    let buf = app.buffer();

    // Default dark theme: primary = Rgb(1, 120, 212)
    // Check that background color of a cell in the themed area matches
    let cell = &buf[(0, 0)];
    let bg = cell.bg;
    assert_eq!(
        bg,
        Color::Rgb(1, 120, 212),
        "Background from $primary should be Rgb(1,120,212), got: {:?}",
        bg
    );
}

// ============================================================================
// 6. Checkbox states
// ============================================================================

#[test]
fn checkbox_checked_shows_checkmark() {
    let app = TestApp::new(20, 3, || Box::new(Checkbox::new("Enabled", true)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    assert!(
        row.contains('✓'),
        "Checked checkbox should show checkmark, got: {:?}",
        row
    );
    assert!(row.contains("Enabled"), "Should show label text");
}

#[test]
fn checkbox_unchecked_shows_empty_box() {
    let app = TestApp::new(20, 3, || Box::new(Checkbox::new("Disabled", false)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    assert!(
        row.contains('☐'),
        "Unchecked checkbox should show ☐, got: {:?}",
        row
    );
    assert!(row.contains("Disabled"), "Should show label text");
}

// ============================================================================
// 7. Switch states
// ============================================================================

#[test]
fn switch_on_has_knob_right() {
    let app = TestApp::new(10, 3, || Box::new(Switch::new(true)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    // ON: track fills with blocks, knob (▌) appears on the right side
    // The pattern is "━━██▌" (track then solid blocks ending with half block)
    assert!(
        row.contains("██"),
        "Switch ON should have solid track blocks, got: {:?}",
        row
    );
    // Knob indicator is at the right side of the switch
    let knob_pos = row.find('▌');
    let block_pos = row.find("██");
    assert!(
        knob_pos.is_some(),
        "Switch ON should contain ▌ (knob), got: {:?}",
        row
    );
    assert!(
        knob_pos.unwrap() > block_pos.unwrap(),
        "Knob (▌) should be to the right of track blocks"
    );
}

#[test]
fn switch_off_has_knob_left() {
    let app = TestApp::new(10, 3, || Box::new(Switch::new(false)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    // OFF: knob (▐██) on the left, track (━) fills the rest
    assert!(
        row.contains("▐██"),
        "Switch OFF should have knob on left ▐██, got: {:?}",
        row
    );
    assert!(
        row.contains('━'),
        "Switch OFF should have empty track ━, got: {:?}",
        row
    );
}

// ============================================================================
// 8. RadioButton -- selected green, unselected dim
// ============================================================================

#[test]
fn radio_button_selected_shows_filled_dot() {
    let app = TestApp::new(20, 3, || Box::new(RadioButton::new("Opt", true)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    assert!(
        row.contains('◉'),
        "Selected radio should show ◉, got: {:?}",
        row
    );

    // Check the color of the indicator -- should be green Rgb(0, 255, 163)
    let indicator_col = row.find('◉').unwrap() as u16;
    let cell = &buf[(indicator_col, 0)];
    assert_eq!(
        cell.fg,
        Color::Rgb(0, 255, 163),
        "Selected radio indicator should be green, got: {:?}",
        cell.fg
    );
}

#[test]
fn radio_button_unselected_shows_empty_circle() {
    let app = TestApp::new(20, 3, || Box::new(RadioButton::new("Opt", false)));
    let buf = app.buffer();
    let row = row_trimmed(buf, 0);
    assert!(
        row.contains('○'),
        "Unselected radio should show ○, got: {:?}",
        row
    );

    // Check the color -- should be dim Rgb(100, 100, 110)
    let indicator_col = row.find('○').unwrap() as u16;
    let cell = &buf[(indicator_col, 0)];
    assert_eq!(
        cell.fg,
        Color::Rgb(100, 100, 110),
        "Unselected radio indicator should be dim, got: {:?}",
        cell.fg
    );
}

// ============================================================================
// 9. DataTable -- zebra striped rows, bold headers
// ============================================================================

#[test]
fn data_table_has_bold_headers_and_zebra_rows() {
    struct DtScreen;
    impl Widget for DtScreen {
        fn widget_type_name(&self) -> &'static str {
            "DtScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            let mut dt = DataTable::new(vec![
                ColumnDef::new("Name").with_width(10),
                ColumnDef::new("Age").with_width(5),
            ]);
            dt.add_row(vec!["Alice".into(), "30".into()]);
            dt.add_row(vec!["Bob".into(), "25".into()]);
            dt.add_row(vec!["Carol".into(), "35".into()]);
            dt.add_row(vec!["Dave".into(), "28".into()]);
            vec![Box::new(dt)]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "DtScreen { layout-direction: vertical; } DataTable { height: 10; width: 30; border: none; }";
    let app = TestApp::new_styled(30, 10, css, || Box::new(DtScreen));
    let buf = app.buffer();

    // Header row should contain "Name" and "Age"
    let header = row_text(buf, 0);
    assert!(
        header.contains("Name"),
        "Header should contain 'Name', got: {:?}",
        header.trim_end()
    );
    assert!(
        header.contains("Age"),
        "Header should contain 'Age', got: {:?}",
        header.trim_end()
    );

    // Header cells should be BOLD
    let name_col = header.find("Name").unwrap() as u16;
    assert!(
        cell_has_modifier(buf, name_col, 0, Modifier::BOLD),
        "Header 'Name' should be bold"
    );

    // Separator row should contain ━
    let sep = row_text(buf, 1);
    assert!(
        sep.contains('━'),
        "Separator row should contain ━, got: {:?}",
        sep.trim_end()
    );

    // Zebra striping: odd data rows (row index 1 = Bob, row index 3 = Dave) should have
    // a different background than even data rows (row index 0 = Alice, row index 2 = Carol).
    // Data rows start at buffer row 2.
    let even_bg = buf[(0, 2)].bg; // Alice row (data index 0)
    let odd_bg = buf[(0, 3)].bg; // Bob row (data index 1)
    assert_ne!(
        even_bg, odd_bg,
        "Zebra striping: even row bg ({:?}) should differ from odd row bg ({:?})",
        even_bg, odd_bg
    );
}

// ============================================================================
// 10. ListView -- selected item has accent color + bold
// ============================================================================

#[test]
fn list_view_selected_item_is_highlighted() {
    struct LvScreen;
    impl Widget for LvScreen {
        fn widget_type_name(&self) -> &'static str {
            "LvScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(ListView::new(vec![
                "First".into(),
                "Second".into(),
                "Third".into(),
            ]))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "LvScreen { layout-direction: vertical; } ListView { height: 5; }";
    let app = TestApp::new_styled(20, 5, css, || Box::new(LvScreen));
    let buf = app.buffer();

    // The first item (index 0) is selected by default
    let first_row = row_text(buf, 0);
    assert!(
        first_row.contains("First"),
        "First item should be visible, got: {:?}",
        first_row.trim_end()
    );

    // Selected item should have accent color Rgb(0, 255, 163) and BOLD
    let first_col = first_row.find("First").unwrap() as u16;
    let cell = &buf[(first_col, 0)];
    assert_eq!(
        cell.fg,
        Color::Rgb(0, 255, 163),
        "Selected list item should have accent color, got: {:?}",
        cell.fg
    );
    assert!(
        cell.modifier.contains(Modifier::BOLD),
        "Selected list item should be bold"
    );

    // Non-selected items should NOT be bold
    let second_row = row_text(buf, 1);
    let second_col = second_row.find("Second").unwrap() as u16;
    let cell2 = &buf[(second_col, 1)];
    assert!(
        !cell2.modifier.contains(Modifier::BOLD),
        "Non-selected list item should not be bold"
    );
}

// ============================================================================
// 11. Tab bar -- active tab has BOLD + UNDERLINED
// ============================================================================

#[test]
fn tab_bar_active_tab_is_bold_underlined() {
    struct TabScreen;
    impl Widget for TabScreen {
        fn widget_type_name(&self) -> &'static str {
            "TabScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Tabs::new(vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
            ]))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "TabScreen { layout-direction: vertical; } Tabs { height: 1; }";
    let app = TestApp::new_styled(40, 3, css, || Box::new(TabScreen));
    let buf = app.buffer();

    // Find "Alpha" (active tab, index 0)
    let row = row_text(buf, 0);
    let alpha_pos = row
        .find("Alpha")
        .expect("Active tab 'Alpha' should be visible");
    let alpha_col = alpha_pos as u16;

    // Active tab chars should be BOLD and UNDERLINED
    assert!(
        cell_has_modifier(buf, alpha_col, 0, Modifier::BOLD),
        "Active tab 'Alpha' should be BOLD"
    );
    assert!(
        cell_has_modifier(buf, alpha_col, 0, Modifier::UNDERLINED),
        "Active tab 'Alpha' should be UNDERLINED"
    );

    // Inactive tab "Beta" should NOT be bold
    let beta_pos = row
        .find("Beta")
        .expect("Inactive tab 'Beta' should be visible");
    let beta_col = beta_pos as u16;
    assert!(
        !cell_has_modifier(buf, beta_col, 0, Modifier::BOLD),
        "Inactive tab 'Beta' should NOT be BOLD"
    );
}

// ============================================================================
// 12. Button -- bold centered label
// ============================================================================

#[test]
fn button_renders_bold_centered_label() {
    // Use bare TestApp so the button IS the root widget and gets full area
    let app = TestApp::new(20, 3, || Box::new(Button::new("OK")));
    let buf = app.buffer();

    // Find "OK" in the buffer -- scan all cells for a cell containing "O" followed by "K"
    let mut found_row = None;
    let mut found_col = None;
    for y in 0..3u16 {
        let text = row_text(buf, y);
        if let Some(col) = text.find("OK") {
            found_row = Some(y);
            found_col = Some(col as u16);
            break;
        }
    }

    let y = found_row.expect("Button label 'OK' should appear in buffer");
    let x = found_col.unwrap();

    // Label should be BOLD -- Button.render always adds Modifier::BOLD
    assert!(
        cell_has_modifier(buf, x, y, Modifier::BOLD),
        "Button label should be BOLD at ({}, {}), modifier: {:?}",
        x,
        y,
        buf[(x, y)].modifier
    );
}

// ============================================================================
// 13. Sparkline -- braille chars in multi-row mode
// ============================================================================

#[test]
fn sparkline_renders_braille_characters() {
    let app = TestApp::new(10, 3, || {
        Box::new(Sparkline::new(vec![1.0, 5.0, 3.0, 8.0, 2.0, 6.0, 4.0, 7.0]))
    });
    let buf = app.buffer();

    let mut braille_count = 0;
    for y in 0..3u16 {
        for x in 0..10u16 {
            if let Some(cell) = buf.cell((x, y)) {
                if let Some(ch) = cell.symbol().chars().next() {
                    if (0x2800..=0x28FF).contains(&(ch as u32)) {
                        braille_count += 1;
                    }
                }
            }
        }
    }
    assert!(
        braille_count > 0,
        "Multi-row sparkline should contain braille characters (U+2800..U+28FF), found {}",
        braille_count
    );
}

// ============================================================================
// 14. Placeholder -- quadrant cross-hatch pattern
// ============================================================================

#[test]
fn placeholder_renders_quadrant_pattern() {
    let app = TestApp::new(10, 5, || Box::new(Placeholder::new()));
    let buf = app.buffer();

    // QUADRANT_CHARS[1..15] are the cross-hatch quadrant characters
    let quadrant_set: std::collections::HashSet<&str> = textual_rs::canvas::QUADRANT_CHARS[1..15]
        .iter()
        .copied()
        .collect();

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
        "Placeholder should contain quadrant cross-hatch characters, found {}",
        quadrant_count
    );
}

// ============================================================================
// 15. Footer -- key badges with cyan bg
// ============================================================================

#[test]
fn footer_renders_key_badges_with_cyan_background() {
    struct FtScreen;
    impl Widget for FtScreen {
        fn widget_type_name(&self) -> &'static str {
            "FtScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Footer)]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = "FtScreen { layout-direction: vertical; } Footer { height: 1; }";
    let app = TestApp::new_styled(60, 3, css, || Box::new(FtScreen));
    let buf = app.buffer();

    // Footer should contain at least "Tab" and "q" key badges
    let row = row_text(buf, 0);
    assert!(
        row.contains("Tab"),
        "Footer should contain Tab binding, got: {:?}",
        row.trim_end()
    );
    assert!(
        row.contains("q"),
        "Footer should contain q binding, got: {:?}",
        row.trim_end()
    );

    // Key badges should have cyan-ish bg: Rgb(0, 212, 255)
    let cyan_bg = Color::Rgb(0, 212, 255);
    let tab_pos = row.find("Tab").unwrap() as u16;
    // The key badge is rendered as " Tab " with padding, check cells around the Tab text
    let mut found_cyan = false;
    for x in tab_pos.saturating_sub(1)..tab_pos + 5 {
        if x < buf.area.width {
            if buf[(x, 0)].bg == cyan_bg {
                found_cyan = true;
                break;
            }
        }
    }
    assert!(
        found_cyan,
        "Footer key badges should have cyan background Rgb(0,212,255)"
    );
}

// ============================================================================
// 16. Full demo screen -- Header + TabbedContent + Footer all render
// ============================================================================

#[test]
fn full_demo_screen_renders_header_tabs_footer() {
    // Pane widget that composes children (TabbedContent calls pane.compose())
    struct HomePane;
    impl Widget for HomePane {
        fn widget_type_name(&self) -> &'static str {
            "HomePane"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("Welcome Home"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    struct SettingsPane;
    impl Widget for SettingsPane {
        fn widget_type_name(&self) -> &'static str {
            "SettingsPane"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Label::new("Settings Panel"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    struct DemoScreen;
    impl Widget for DemoScreen {
        fn widget_type_name(&self) -> &'static str {
            "DemoScreen"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(Header::new("Demo App")),
                Box::new(TabbedContent::new(
                    vec!["Home".into(), "Settings".into()],
                    vec![Box::new(HomePane), Box::new(SettingsPane)],
                )),
                Box::new(Footer),
            ]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let css = r#"
        DemoScreen { layout-direction: vertical; }
        Header { height: 1; }
        TabbedContent { min-height: 3; }
        HomePane { layout-direction: vertical; }
        SettingsPane { layout-direction: vertical; }
        Footer { height: 1; }
        Label { height: 1; }
    "#;
    let app = TestApp::new_styled(60, 10, css, || Box::new(DemoScreen));
    let buf = app.buffer();

    // Header should contain title
    let mut found_header = false;
    for y in 0..2u16 {
        let text = row_text(buf, y);
        if text.contains("Demo App") {
            found_header = true;
            break;
        }
    }
    assert!(found_header, "Header should render 'Demo App'");

    // Tab bar should show "Home" (active)
    let mut found_tabs = false;
    for y in 0..5u16 {
        let text = row_text(buf, y);
        if text.contains("Home") {
            found_tabs = true;
            break;
        }
    }
    assert!(found_tabs, "Tab bar should render 'Home' tab label");

    // Active pane content: first tab => "Welcome Home"
    let mut found_content = false;
    for y in 0..10u16 {
        let text = row_text(buf, y);
        if text.contains("Welcome Home") {
            found_content = true;
            break;
        }
    }
    assert!(found_content, "Active pane should render 'Welcome Home'");

    // Footer should be somewhere in the buffer with key bindings
    // It renders key badges like " Tab ", " q " etc.
    let mut found_footer = false;
    let mut all_rows_debug = String::new();
    for y in 0..10u16 {
        let text = row_text(buf, y);
        all_rows_debug.push_str(&format!("  row {}: {:?}\n", y, text.trim_end()));
        if text.contains("Tab") || text.contains("Quit") || text.contains(" q ") {
            found_footer = true;
        }
    }
    assert!(
        found_footer,
        "Footer should render key bindings.\nBuffer:\n{}",
        all_rows_debug
    );
}
