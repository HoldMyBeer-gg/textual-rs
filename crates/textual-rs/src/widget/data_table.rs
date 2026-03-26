use std::cell::{Cell, RefCell};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
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
        "DataTable { border: tall; min-height: 5; }"
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
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.columns.is_empty() {
            return;
        }

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
            buf.set_string(x, y, &display, Style::default());
            x += display.chars().count() as u16;

            // Column separator " | " (skip after last visible column)
            if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                buf.set_string(x, y, " | ", Style::default());
                x += 3;
            }
        }

        // --- Render separator row ---
        let sep_y = area.y + 1;
        {
            let mut x = area.x;
            for (vi, &ci) in visible_cols.iter().enumerate() {
                if x >= area.x + area.width {
                    break;
                }
                let col_w = widths[ci] as usize;
                let avail = ((area.x + area.width - x) as usize).min(col_w);
                let sep: String = "─".repeat(avail);
                buf.set_string(x, sep_y, &sep, Style::default());
                x += avail as u16;

                if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                    buf.set_string(x, sep_y, "─┼─", Style::default());
                    x += 3;
                }
            }
        }

        // --- Render data rows ---
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

                // Highlight cursor row with reverse video
                let style = if is_cursor_row {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                buf.set_string(x, row_y, &display, style);
                x += display.chars().count() as u16;

                // Column separator
                if vi + 1 < visible_cols.len() && x + 3 <= area.x + area.width {
                    buf.set_string(x, row_y, " | ", style);
                    x += 3;
                }
            }
        }

        // --- Vertical scrollbar on right edge ---
        if !rows.is_empty() && visible_row_count < rows.len() {
            let sb_x = area.x + area.width - 1;
            let sb_height = data_area_height as usize;
            let total = rows.len();
            let thumb_pos = if total > 0 {
                (scroll_row * sb_height) / total
            } else {
                0
            };
            let thumb_size = ((visible_row_count * sb_height) / total).max(1);

            for i in 0..sb_height {
                let sb_y = area.y + header_rows + i as u16;
                if sb_y >= area.y + area.height {
                    break;
                }
                let ch = if i >= thumb_pos && i < thumb_pos + thumb_size { "█" } else { "░" };
                buf.set_string(sb_x, sb_y, ch, Style::default());
            }
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
