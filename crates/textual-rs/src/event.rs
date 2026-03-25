use crossterm::event::KeyEvent;

/// Events flowing through the application event bus.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A keyboard event from the terminal.
    Key(KeyEvent),
    /// Terminal was resized to (columns, rows).
    Resize(u16, u16),
    /// Reserved for future periodic tick events (Phase 3).
    Tick,
}
