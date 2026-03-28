//! Test assertion helpers for verifying rendered buffer content.

use ratatui::buffer::Buffer;

/// Assert that specific rows of the buffer match expected strings.
///
/// Each `&str` in `expected` corresponds to one row, starting from row 0.
/// Trailing spaces in the buffer are trimmed before comparison.
///
/// # Panics
///
/// Panics with a descriptive message if any row does not match the expected string,
/// or if more rows are expected than the buffer contains.
///
/// # Example
///
/// ```ignore
/// use textual_rs::testing::assertions::assert_buffer_lines;
/// assert_buffer_lines(test_app.buffer(), &["Hello", "World"]);
/// ```
pub fn assert_buffer_lines(buffer: &Buffer, expected: &[&str]) {
    let area = buffer.area;
    for (row_idx, expected_line) in expected.iter().enumerate() {
        assert!(
            (row_idx as u16) < area.height,
            "Expected line {} but buffer only has {} rows",
            row_idx,
            area.height
        );
        let mut actual = String::new();
        for col in 0..area.width {
            let cell = &buffer[(col, row_idx as u16)];
            actual.push_str(cell.symbol());
        }
        let actual_trimmed = actual.trim_end();
        assert_eq!(
            actual_trimmed, *expected_line,
            "Row {} mismatch:\n  expected: {:?}\n  actual:   {:?}",
            row_idx, expected_line, actual_trimmed
        );
    }
}

/// Assert that the cell at `(col, row)` contains the expected symbol string.
///
/// # Panics
///
/// Panics if the cell symbol does not match.
pub fn assert_cell(buffer: &Buffer, col: u16, row: u16, expected: &str) {
    let cell = &buffer[(col, row)];
    assert_eq!(
        cell.symbol(),
        expected,
        "Cell ({}, {}) mismatch: expected {:?}, got {:?}",
        col,
        row,
        expected,
        cell.symbol()
    );
}
