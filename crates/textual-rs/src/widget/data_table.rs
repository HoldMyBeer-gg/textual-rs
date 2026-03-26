use std::cell::{Cell, RefCell};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use crossterm::event::{KeyCode, KeyModifiers};
use unicode_width::UnicodeWidthStr;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Column definition for a DataTable.
pub struct ColumnDef {
    pub label: String,
    /// Fixed width for this column. None = auto-size to content.
    pub width: Option<u16>,
}

impl ColumnDef {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), width: None }
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
}

/// Messages emitted by a DataTable.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the user selects a row by pressing Enter.
    pub struct RowSelected {
        pub row: usize,
    }

    impl Message for RowSelected {}

    /// Emitted when the sort order changes.
    pub struct SortChanged {
        pub column: usize,
        pub ascending: bool,
    }

    impl Message for SortChanged {}
}

/// Tabular data display widget with column headers, sorting, and two-axis scrolling.
///
/// Renders a header row, separator, then data rows. The cursor highlights the current
/// row. Pressing `s` sorts by the cursor column. Enter emits `messages::RowSelected`.
pub struct DataTable {
    pub columns: Vec<ColumnDef>,
    /// Row-major data storage. Interior mutability so sort_by_column can mutate from &self.
    rows: RefCell<Vec<Vec<String>>>,
    pub cursor_row: Reactive<usize>,
    pub cursor_col: Reactive<usize>,
    pub scroll_offset_row: Reactive<usize>,
    pub scroll_offset_col: Reactive<usize>,
    sort_column: Cell<Option<usize>>,
    sort_ascending: Cell<bool>,
    column_widths: RefCell<Vec<u16>>,
    viewport_rows: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
    last_area_y: Cell<u16>,
    last_header_rows: Cell<u16>,
}

impl DataTable {
    /// Create a new DataTable with the given column definitions and no rows.
    pub fn new(columns: Vec<ColumnDef>) -> Self {
        // Pre-compute widths from headers
        let widths = columns
            .iter()
            .map(|c| c.width.unwrap_or_else(|| c.label.width() as u16))
            .collect();
        Self {
            columns,
            rows: RefCell::new(Vec::new()),
            cursor_row: Reactive::new(0),
            cursor_col: Reactive::new(0),
            scroll_offset_row: Reactive::new(0),
            scroll_offset_col: Reactive::new(0),
            sort_column: Cell::new(None),
            sort_ascending: Cell::new(true),
            column_widths: RefCell::new(widths),
            viewport_rows: Cell::new(0),
            own_id: Cell::new(None),
            last_area_y: Cell::new(0),
            last_header_rows: Cell::new(2),
        }
    }

    /// Append a row and recompute column widths.
    pub fn add_row(&mut self, row: Vec<String>) {
        // Extend row to match column count if needed
        let mut padded = row;
        while padded.len() < self.columns.len() {
            padded.push(String::new());
        }
        // Update column widths
        {
            let mut widths = self.column_widths.borrow_mut();
            for (i, cell) in padded.iter().enumerate().take(self.columns.len()) {
                let col_w = if let Some(fixed) = self.columns[i].width {
                    fixed
                } else {
                    let content_w = cell.as_str().width() as u16;
                    widths[i].max(content_w)
                };
                widths[i] = col_w;
            }
        }
        self.rows.borrow_mut().push(padded);
    }

    /// Number of rows in the table.
    pub fn row_count(&self) -> usize {
        self.rows.borrow().len()
    }

    /// Sort rows by the given column index. Toggles ascending/descending if same column.
    pub fn sort_by_column(&self, col: usize) {
        if col >= self.columns.len() {
            return;
        }
        let ascending = if self.sort_column.get() == Some(col) {
            let new_asc = !self.sort_ascending.get();
            self.sort_ascending.set(new_asc);
            new_asc
        } else {
            self.sort_column.set(Some(col));
            self.sort_ascending.set(true);
            true
        };

        let mut rows = self.rows.borrow_mut();
        if ascending {
            rows.sort_by(|a, b| {
                let a_val = a.get(col).map(|s| s.as_str()).unwrap_or("");
                let b_val = b.get(col).map(|s| s.as_str()).unwrap_or("");
                a_val.cmp(b_val)
            });
        } else {
            rows.sort_by(|a, b| {
                let a_val = a.get(col).map(|s| s.as_str()).unwrap_or("");
                let b_val = b.get(col).map(|s| s.as_str()).unwrap_or("");
                b_val.cmp(a_val)
            });
        }
    }

    fn adjust_scroll_row(&self) {
        let cursor = self.cursor_row.get_untracked();
        let vp = self.viewport_rows.get() as usize;
        if vp == 0 {
            return;
        }
        let offset = self.scroll_offset_row.get_untracked();
        if cursor < offset {
            self.scroll_offset_row.set(cursor);
        } else if cursor >= offset + vp {
            self.scroll_offset_row.set(cursor + 1 - vp);
        }
    }

    fn adjust_scroll_col(&self) {
        let cursor = self.cursor_col.get_untracked();
        let offset = self.scroll_offset_col.get_untracked();
        if cursor < offset {
            self.scroll_offset_col.set(cursor);
        } else if cursor > offset + 3 {
            // Keep a few columns visible
            self.scroll_offset_col.set(cursor.saturating_sub(3));
        }
    }
}

static DATA_TABLE_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: "cursor_up",
        description: "Up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: "cursor_down",
        description: "Down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Null,
        modifiers: KeyModifiers::NONE,
        action: "scroll_up",
        description: "Scroll up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Null,
        modifiers: KeyModifiers::NONE,
        action: "scroll_down",
        description: "Scroll down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "cursor_left",
        description: "Left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "cursor_right",
        description: "Right",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "select_row",
        description: "Select",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "cursor_home",
        description: "First Row",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "cursor_end",
        description: "Last Row",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        action: "sort_column",
        description: "Sort",
        show: true,
    },
];

impl Widget for DataTable {
    fn widget_type_name(&self) -> &'static str {
        "DataTable"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "DataTable { border: inner; min-height: 5; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        DATA_TABLE_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> super::EventPropagation {
        use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            if matches!(m.kind, MouseEventKind::Down(MouseButton::Left)) {
                let header_rows = self.last_header_rows.get();
                let data_start_y = self.last_area_y.get() + header_rows;
                if m.row >= data_start_y {
                    let local_row = (m.row - data_start_y) as usize;
                    let scroll = self.scroll_offset_row.get_untracked();
                    let row_idx = scroll + local_row;
                    let rows = self.rows.borrow();
                    if row_idx < rows.len() {
                        drop(rows);
                        self.cursor_row.set(row_idx);
                        return super::EventPropagation::Stop;
                    }
                }
            }
        }
        super::EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_up" => {
                let current = self.cursor_row.get_untracked();
                if current > 0 {
                    self.cursor_row.set(current - 1);
                    self.adjust_scroll_row();
                }
            }
            "cursor_down" => {
                let current = self.cursor_row.get_untracked();
                let row_count = self.rows.borrow().len();
                if row_count > 0 && current + 1 < row_count {
                    self.cursor_row.set(current + 1);
                    self.adjust_scroll_row();
                }
            }
            "cursor_left" => {
                let current = self.cursor_col.get_untracked();
                if current > 0 {
                    self.cursor_col.set(current - 1);
                    self.adjust_scroll_col();
                }
            }
            "cursor_right" => {
                let current = self.cursor_col.get_untracked();
                if !self.columns.is_empty() && current + 1 < self.columns.len() {
                    self.cursor_col.set(current + 1);
                    self.adjust_scroll_col();
                }
            }
            "select_row" => {
                if let Some(id) = self.own_id.get() {
                    let row = self.cursor_row.get_untracked();
                    ctx.post_message(id, messages::RowSelected { row });
                }
            }
            "sort_column" => {
                let col = self.cursor_col.get_untracked();
                self.sort_by_column(col);
                if let Some(id) = self.own_id.get() {
                    ctx.post_message(
                        id,
                        messages::SortChanged {
                            column: col,
                            ascending: self.sort_ascending.get(),
                        },
                    );
                }
            }
            "cursor_home" => {
                self.cursor_row.set(0);
                self.scroll_offset_row.set(0);
            }
            "cursor_end" => {
                let row_count = self.rows.borrow().len();
                if row_count > 0 {
                    let last = row_count - 1;
                    self.cursor_row.set(last);
                    self.adjust_scroll_row();
                }
            }
            "scroll_up" => {
                let offset = self.scroll_offset_row.get_untracked();
                if offset > 0 {
                    self.scroll_offset_row.set(offset - 1);
                }
            }
            "scroll_down" => {
                let offset = self.scroll_offset_row.get_untracked();
                let viewport = self.viewport_rows.get() as usize;
                let row_count = self.rows.borrow().len();
                if viewport > 0 && row_count > viewport && offset < row_count - viewport {
                    self.scroll_offset_row.set(offset + 1);
                }
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.columns.is_empty() {
            return;
        }
        self.last_area_y.set(area.y);

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Header row = 1, separator row = 1, remaining = data rows
        let header_rows: u16 = 2; // header + separator
        let data_area_height = area.height.saturating_sub(header_rows);
        self.viewport_rows.set(data_area_height);

        let cursor_row = self.cursor_row.get_untracked();
        let _cursor_col = self.cursor_col.get_untracked();
        let scroll_row = self.scroll_offset_row.get_untracked();
        let scroll_col = self.scroll_offset_col.get_untracked();

        let widths = self.column_widths.borrow();
        let sort_col = self.sort_column.get();
        let sort_asc = self.sort_ascending.get();
        let rows = self.rows.borrow();

        // Determine visible columns from scroll_offset_col
        let visible_cols: Vec<usize> = (scroll_col..self.columns.len()).collect();

        // --- Render header row ---
        let mut x = area.x;
        let y = area.y;
        for (vi, &ci) in visible_cols.iter().enumerate() {
            if x >= area.x + area.width {
                break;
            }
            let col_w = widths[ci] as usize;
            let label = &self.columns[ci].label;
            // Add sort indicator if this is the sort column
            let cell_text = if sort_col == Some(ci) {
                let indicator = if sort_asc { " ▲" } else { " ▼" };
                format!("{}{}", label, indicator)
            } else {
                label.clone()
            };
            // Pad or truncate to column width
            let padded = pad_or_truncate(&cell_text, col_w);
            let avail = (area.x + area.width - x) as usize;
            let display: String = padded.chars().take(avail).collect();
            let header_style = style
                .add_modifier(ratatui::style::Modifier::BOLD)
                .fg(ratatui::style::Color::Rgb(0, 212, 255));
            buf.set_string(x, y, &display, header_style);
            x += display.chars().count() as u16;

            // Column separator " │ " (skip after last visible column)
            if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                let sep_style = style.fg(ratatui::style::Color::Rgb(74, 74, 90));
                buf.set_string(x, y, " │ ", sep_style);
                x += 3;
            }
        }

        // --- Render separator row ---
        let sep_y = area.y + 1;
        let sep_style = style.fg(ratatui::style::Color::Rgb(74, 74, 90));
        {
            let mut x = area.x;
            for (vi, &ci) in visible_cols.iter().enumerate() {
                if x >= area.x + area.width {
                    break;
                }
                let col_w = widths[ci] as usize;
                let avail = ((area.x + area.width - x) as usize).min(col_w);
                let sep: String = "━".repeat(avail);
                buf.set_string(x, sep_y, &sep, sep_style);
                x += avail as u16;

                if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                    buf.set_string(x, sep_y, "━┿━", sep_style);
                    x += 3;
                }
            }
        }

        // --- Render data rows ---
        // Zebra striping: alternate rows get a subtly lighter background
        let table_bg = buf.cell((area.x, area.y))
            .and_then(|c| c.style().bg)
            .unwrap_or(ratatui::style::Color::Rgb(26, 26, 46));
        let alt_bg = crate::canvas::blend_color(table_bg, ratatui::style::Color::Rgb(255, 255, 255), 0.06);

        let visible_row_count = data_area_height as usize;
        for row_offset in 0..visible_row_count {
            let row_idx = scroll_row + row_offset;
            if row_idx >= rows.len() {
                break;
            }
            let row_data = &rows[row_idx];
            let row_y = area.y + header_rows + row_offset as u16;
            if row_y >= area.y + area.height {
                break;
            }

            let is_cursor_row = row_idx == cursor_row;
            let is_alt_row = row_idx % 2 == 1;

            // Fill zebra stripe background for alt rows (before text rendering)
            if is_alt_row && !is_cursor_row {
                for fill_x in area.x..area.x + area.width {
                    if let Some(cell) = buf.cell_mut((fill_x, row_y)) {
                        cell.set_bg(alt_bg);
                    }
                }
            }

            let mut x = area.x;

            for (vi, &ci) in visible_cols.iter().enumerate() {
                if x >= area.x + area.width {
                    break;
                }
                let col_w = widths[ci] as usize;
                let cell_text = row_data.get(ci).map(|s| s.as_str()).unwrap_or("");
                let padded = pad_or_truncate(cell_text, col_w);
                let avail = (area.x + area.width - x) as usize;
                let display: String = padded.chars().take(avail).collect();

                // Highlight cursor row with accent color; zebra rows get alt_bg
                let row_style = if is_cursor_row {
                    style
                        .fg(ratatui::style::Color::Rgb(0, 255, 163))
                        .add_modifier(Modifier::BOLD)
                } else if is_alt_row {
                    style.bg(alt_bg)
                } else {
                    style
                };

                buf.set_string(x, row_y, &display, row_style);
                x += display.chars().count() as u16;

                // Column separator
                let sep_col_style = if is_cursor_row { row_style } else {
                    let base_sep = style.fg(ratatui::style::Color::Rgb(74, 74, 90));
                    if is_alt_row { base_sep.bg(alt_bg) } else { base_sep }
                };
                if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                    buf.set_string(x, row_y, " │ ", sep_col_style);
                    x += 3;
                }
            }
        }

        // --- Vertical scrollbar on right edge ---
        if !rows.is_empty() && visible_row_count < rows.len() {
            let sb_x = area.x + area.width - 1;
            let bar_color = ratatui::style::Color::Rgb(0, 255, 163);
            let track_color = ratatui::style::Color::Rgb(30, 30, 40);
            crate::canvas::vertical_scrollbar(
                buf,
                sb_x,
                area.y + header_rows,
                data_area_height,
                rows.len(),
                visible_row_count,
                scroll_row,
                bar_color,
                track_color,
            );
        }
    }
}

/// Pad a string to `width` chars, or truncate if longer.
fn pad_or_truncate(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.chars().take(width).collect()
    } else {
        let padding = " ".repeat(width - char_count);
        format!("{}{}", s, padding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use crate::widget::context::AppContext;
    use crate::widget::Widget;

    /// Helper: create a buffer pre-filled with a given background color.
    fn buf_with_bg(area: Rect, bg: Color) -> Buffer {
        let mut buf = Buffer::empty(area);
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                }
            }
        }
        buf
    }

    fn make_table_with_rows(n: usize) -> DataTable {
        let cols = vec![ColumnDef::new("Name").with_width(10)];
        let mut dt = DataTable::new(cols);
        for i in 0..n {
            dt.add_row(vec![format!("Row{}", i)]);
        }
        dt
    }

    #[test]
    fn zebra_stripe_odd_rows_have_alt_bg() {
        let bg = Color::Rgb(26, 26, 46);
        // header(1) + separator(1) + 4 data rows = 6 tall
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let table = make_table_with_rows(4);

        // cursor_row defaults to 0, so row 0 is the cursor row
        table.render(&ctx, area, &mut buf);

        let alt_bg = crate::canvas::blend_color(bg, Color::Rgb(255, 255, 255), 0.06);

        // Data rows start at y=2 (after header + separator)
        // Row 0 (y=2): cursor row -- NOT zebra
        // Row 1 (y=3): odd -- zebra (alt_bg)
        // Row 2 (y=4): even -- NOT zebra
        // Row 3 (y=5): odd -- zebra (alt_bg)

        // Check row 1 (y=3) has alt_bg on a non-text cell at the end
        let row1_bg = buf.cell((19, 3)).unwrap().bg;
        assert_eq!(row1_bg, alt_bg, "odd row (row_idx=1) should have zebra stripe bg");

        // Check row 2 (y=4) keeps original bg
        let row2_bg = buf.cell((19, 4)).unwrap().bg;
        assert_eq!(row2_bg, bg, "even row (row_idx=2) should have original bg");

        // Check row 3 (y=5) has alt_bg
        let row3_bg = buf.cell((19, 5)).unwrap().bg;
        assert_eq!(row3_bg, alt_bg, "odd row (row_idx=3) should have zebra stripe bg");
    }

    #[test]
    fn cursor_row_not_affected_by_zebra() {
        let bg = Color::Rgb(26, 26, 46);
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let table = make_table_with_rows(3);

        // Move cursor to row 1 (an odd row that would get zebra)
        table.cursor_row.set(1);
        table.render(&ctx, area, &mut buf);

        // Row 1 (y=3) is cursor row -- should NOT have zebra bg
        // Cursor row uses accent fg (green) not alt_bg
        let alt_bg = crate::canvas::blend_color(bg, Color::Rgb(255, 255, 255), 0.06);
        let row1_bg = buf.cell((19, 3)).unwrap().bg;
        assert_ne!(row1_bg, alt_bg, "cursor row should not have zebra stripe bg");
    }
}
