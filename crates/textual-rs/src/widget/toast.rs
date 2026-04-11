//! Toast notification system: severity levels, entry type, and rendering helpers.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

/// Severity level of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastSeverity {
    /// Informational notification, styled with the primary theme color.
    Info,
    /// Warning notification, styled with the warning theme color.
    Warning,
    /// Error notification, styled with the error theme color.
    Error,
}

/// A single toast notification entry.
#[derive(Debug, Clone)]
pub struct ToastEntry {
    /// Notification message text displayed inside the toast.
    pub message: String,
    /// Severity level controlling the border color.
    pub severity: ToastSeverity,
    /// Auto-dismiss after this many milliseconds. 0 = persistent (never auto-dismiss).
    pub timeout_ms: u64,
    /// Number of ticks elapsed since this toast was shown. Each tick ≈ 33ms at 30fps.
    pub elapsed_ticks: u32,
}

/// Push a new toast onto the stack. If there are already 5 toasts, the oldest (index 0) is dropped.
pub fn push_toast(
    toasts: &mut Vec<ToastEntry>,
    message: String,
    severity: ToastSeverity,
    timeout_ms: u64,
) {
    if toasts.len() >= 5 {
        toasts.remove(0);
    }
    toasts.push(ToastEntry {
        message,
        severity,
        timeout_ms,
        elapsed_ticks: 0,
    });
}

/// Advance all toast countdowns by one tick and remove expired toasts.
/// Persistent toasts (timeout_ms == 0) are never removed.
/// Each tick ≈ 33ms at ~30fps.
pub fn tick_toasts(toasts: &mut Vec<ToastEntry>) {
    for toast in toasts.iter_mut() {
        if toast.timeout_ms > 0 {
            toast.elapsed_ticks += 1;
        }
    }
    toasts.retain(|t| t.timeout_ms == 0 || (t.elapsed_ticks as u64 * 33) < t.timeout_ms);
}

/// Render all active toasts into the buffer, stacked in the bottom-right corner.
/// Newest toast appears at the bottom; older toasts stack upward.
/// Renders below active_overlay (caller must invoke this before painting active_overlay).
pub fn render_toasts(
    toasts: &[ToastEntry],
    area: Rect,
    buf: &mut Buffer,
    theme: &crate::css::theme::Theme,
) {
    if toasts.is_empty() {
        return;
    }

    // Toast dimensions
    let toast_width = 42u16.min(area.width.saturating_sub(2));
    let toast_height = 3u16; // top border + 1 content line + bottom border

    let toast_x = area.right().saturating_sub(toast_width + 1);

    // Iterate in reverse so newest (last in vec) is at the bottom
    for (i, toast) in toasts.iter().rev().enumerate() {
        let row_bottom = area.bottom().saturating_sub((i as u16 + 1) * toast_height);
        if row_bottom < area.y {
            break; // No vertical room for more toasts
        }

        // Severity color for border and indicator
        let (r, g, b) = match toast.severity {
            ToastSeverity::Info => theme.primary,
            ToastSeverity::Warning => theme.warning,
            ToastSeverity::Error => theme.error,
        };
        let severity_color = Color::Rgb(r, g, b);
        let border_style = Style::default().fg(severity_color);

        // Background: dark tint behind the toast
        let bg_color = Color::Rgb(20, 20, 28);
        let bg_style = Style::default().bg(bg_color);
        let text_style = Style::default().fg(Color::White).bg(bg_color);

        let toast_rect = Rect::new(toast_x, row_bottom, toast_width, toast_height);

        // Fill background
        for y in toast_rect.y..toast_rect.y + toast_rect.height {
            for x in toast_rect.x..toast_rect.x + toast_rect.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ');
                    cell.set_style(bg_style);
                }
            }
        }

        // Draw border manually (thin box drawing chars, colored with severity)
        // Top border
        let top_y = toast_rect.y;
        let bot_y = toast_rect.y + toast_rect.height - 1;
        let left_x = toast_rect.x;
        let right_x = toast_rect.x + toast_rect.width - 1;

        // Corners
        if let Some(cell) = buf.cell_mut((left_x, top_y)) {
            cell.set_symbol("\u{256D}"); // ╭ top-left rounded
            cell.set_style(border_style);
        }
        if let Some(cell) = buf.cell_mut((right_x, top_y)) {
            cell.set_symbol("\u{256E}"); // ╮ top-right rounded
            cell.set_style(border_style);
        }
        if let Some(cell) = buf.cell_mut((left_x, bot_y)) {
            cell.set_symbol("\u{2570}"); // ╰ bottom-left rounded
            cell.set_style(border_style);
        }
        if let Some(cell) = buf.cell_mut((right_x, bot_y)) {
            cell.set_symbol("\u{256F}"); // ╯ bottom-right rounded
            cell.set_style(border_style);
        }

        // Top and bottom horizontal borders
        for x in (left_x + 1)..right_x {
            if let Some(cell) = buf.cell_mut((x, top_y)) {
                cell.set_symbol("\u{2500}"); // ─
                cell.set_style(border_style);
            }
            if let Some(cell) = buf.cell_mut((x, bot_y)) {
                cell.set_symbol("\u{2500}"); // ─
                cell.set_style(border_style);
            }
        }

        // Left and right vertical borders
        let content_y = top_y + 1;
        if let Some(cell) = buf.cell_mut((left_x, content_y)) {
            cell.set_symbol("\u{2502}"); // │
            cell.set_style(border_style);
        }
        if let Some(cell) = buf.cell_mut((right_x, content_y)) {
            cell.set_symbol("\u{2502}"); // │
            cell.set_style(border_style);
        }

        // Content: message text, truncated to fit inside the box
        // Interior width = toast_width - 2 (for left/right borders)
        let interior_width = toast_width.saturating_sub(2) as usize;
        // Reserve 1 char for severity symbol + 1 space
        let max_msg_chars = interior_width.saturating_sub(2);
        let severity_symbol = match toast.severity {
            ToastSeverity::Info => "i",
            ToastSeverity::Warning => "!",
            ToastSeverity::Error => "x",
        };

        // Write severity symbol
        if let Some(cell) = buf.cell_mut((left_x + 1, content_y)) {
            cell.set_symbol(severity_symbol);
            cell.set_style(Style::default().fg(severity_color).bg(bg_color));
        }

        // Truncate message to fit
        let msg: &str = &toast.message;
        let truncated: String = if msg.chars().count() > max_msg_chars {
            msg.chars()
                .take(max_msg_chars.saturating_sub(1))
                .collect::<String>()
                + "…"
        } else {
            msg.to_string()
        };

        buf.set_string(left_x + 2, content_y, &truncated, text_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::theme::default_dark_theme;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn make_info_entry(msg: &str, timeout_ms: u64) -> ToastEntry {
        ToastEntry {
            message: msg.to_string(),
            severity: ToastSeverity::Info,
            timeout_ms,
            elapsed_ticks: 0,
        }
    }

    #[test]
    fn toast_entry_creation() {
        let entry = ToastEntry {
            message: "Hello".to_string(),
            severity: ToastSeverity::Warning,
            timeout_ms: 3000,
            elapsed_ticks: 0,
        };
        assert_eq!(entry.message, "Hello");
        assert_eq!(entry.severity, ToastSeverity::Warning);
        assert_eq!(entry.timeout_ms, 3000);
        assert_eq!(entry.elapsed_ticks, 0);
    }

    #[test]
    fn push_toast_overflow_drops_oldest() {
        let mut toasts: Vec<ToastEntry> = Vec::new();
        for i in 0..5 {
            push_toast(&mut toasts, format!("msg-{}", i), ToastSeverity::Info, 3000);
        }
        assert_eq!(toasts.len(), 5);
        // Adding a 6th should drop index 0 ("msg-0")
        push_toast(&mut toasts, "msg-5".to_string(), ToastSeverity::Info, 3000);
        assert_eq!(toasts.len(), 5);
        assert_eq!(toasts[0].message, "msg-1");
        assert_eq!(toasts[4].message, "msg-5");
    }

    #[test]
    fn tick_toasts_retains_active_toasts() {
        let mut toasts = vec![make_info_entry("active", 3000)]; // needs 91 ticks to expire (3000/33)
        tick_toasts(&mut toasts);
        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts[0].elapsed_ticks, 1);
    }

    #[test]
    fn tick_toasts_removes_expired_toasts() {
        let mut toasts = vec![ToastEntry {
            message: "expired".to_string(),
            severity: ToastSeverity::Info,
            timeout_ms: 33, // expires after 1 tick (1 * 33 >= 33)
            elapsed_ticks: 0,
        }];
        tick_toasts(&mut toasts); // elapsed_ticks becomes 1; 1*33 >= 33 → remove
        assert!(toasts.is_empty(), "expired toast should be removed");
    }

    #[test]
    fn tick_toasts_does_not_remove_persistent_toasts() {
        let mut toasts = vec![ToastEntry {
            message: "persistent".to_string(),
            severity: ToastSeverity::Info,
            timeout_ms: 0, // persistent
            elapsed_ticks: 0,
        }];
        // Tick many times — should never be removed
        for _ in 0..1000 {
            tick_toasts(&mut toasts);
        }
        assert_eq!(toasts.len(), 1);
        // elapsed_ticks should not advance for persistent toasts
        assert_eq!(toasts[0].elapsed_ticks, 0);
    }

    #[test]
    fn render_toasts_empty_produces_no_writes() {
        let theme = default_dark_theme();
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let empty: Vec<ToastEntry> = vec![];
        // Capture initial content (all spaces)
        let initial_content: Vec<String> = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        render_toasts(&empty, area, &mut buf, &theme);
        // Buffer should be unchanged
        let after_content: Vec<String> = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert_eq!(initial_content, after_content);
    }

    #[test]
    fn render_toasts_single_toast_writes_to_bottom_right() {
        let theme = default_dark_theme();
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let toasts = vec![ToastEntry {
            message: "Hello world".to_string(),
            severity: ToastSeverity::Info,
            timeout_ms: 3000,
            elapsed_ticks: 0,
        }];
        render_toasts(&toasts, area, &mut buf, &theme);

        // Toast should appear at the bottom-right; toast_width = min(42, 80-2) = 42
        // toast_x = 80 - 42 - 1 = 37
        // row_bottom for i=0: 24 - 1*3 = 21 (top row of toast box)
        // content row at y=22 (row_bottom + 1)
        let content_y = 22u16;
        let content_x_start = 37u16 + 2; // left_x+2 for message (skip border + severity symbol)

        // Check that something was written at the content position
        let cell = &buf[(content_x_start, content_y)];
        // "Hello world" should be written starting here
        assert_ne!(
            cell.symbol(),
            " ",
            "Expected text content at toast position"
        );
    }

    #[test]
    fn render_toasts_multiple_stack_upward() {
        let theme = default_dark_theme();
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let toasts = vec![
            ToastEntry {
                message: "toast-0".to_string(),
                severity: ToastSeverity::Info,
                timeout_ms: 3000,
                elapsed_ticks: 0,
            },
            ToastEntry {
                message: "toast-1".to_string(),
                severity: ToastSeverity::Warning,
                timeout_ms: 3000,
                elapsed_ticks: 0,
            },
            ToastEntry {
                message: "toast-2".to_string(),
                severity: ToastSeverity::Error,
                timeout_ms: 3000,
                elapsed_ticks: 0,
            },
        ];
        render_toasts(&toasts, area, &mut buf, &theme);

        // toast-2 (newest/last) is at i=0 → row_bottom = 24 - 3 = 21, content_y = 22
        // toast-1 (mid) is at i=1 → row_bottom = 24 - 6 = 18, content_y = 19
        // toast-0 (oldest) is at i=2 → row_bottom = 24 - 9 = 15, content_y = 16

        let toast_x = 37u16;
        let msg_x = toast_x + 2; // skip border + severity symbol

        // Newest at bottom
        let newest_cell = &buf[(msg_x, 22u16)];
        assert_ne!(
            newest_cell.symbol(),
            " ",
            "Newest toast (toast-2) should be at bottom"
        );

        // Middle toast
        let mid_cell = &buf[(msg_x, 19u16)];
        assert_ne!(
            mid_cell.symbol(),
            " ",
            "Middle toast (toast-1) should stack above newest"
        );

        // Oldest at top
        let oldest_cell = &buf[(msg_x, 16u16)];
        assert_ne!(
            oldest_cell.symbol(),
            " ",
            "Oldest toast (toast-0) should be highest"
        );
    }

    #[test]
    fn render_toasts_width_clamped_on_narrow_terminal() {
        let theme = default_dark_theme();
        // Narrow terminal: 20 wide
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let toasts = vec![make_info_entry("narrow terminal test", 3000)];
        // Should not panic — toast_width = min(42, 20-2) = 18
        render_toasts(&toasts, area, &mut buf, &theme);
        // toast_width = 18, toast_x = 20 - 18 - 1 = 1
        // row_bottom = 10 - 3 = 7, content_y = 8
        // Just verify it didn't panic and something is written
        let content_y = 8u16;
        let toast_x = 1u16;
        let cell = &buf[(toast_x, content_y)]; // left border
        assert_ne!(
            cell.symbol(),
            " ",
            "Expected border at left edge of narrow toast"
        );
    }
}
