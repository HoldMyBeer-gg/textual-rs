//! Event system: application events, message passing, key bindings, and timers.

pub mod dispatch;
pub mod keybinding;
pub mod message;
pub mod timer;

// Re-export key types
pub use dispatch::dispatch_message;
pub use keybinding::KeyBinding;
pub use message::Message;

use crossterm::event::{KeyEvent, MouseEvent};

/// Events flowing through the application event bus.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A keyboard event from the terminal.
    Key(KeyEvent),
    /// A mouse event from the terminal.
    Mouse(MouseEvent),
    /// Terminal was resized to (columns, rows).
    Resize(u16, u16),
    /// Reserved for periodic tick events from spawn_tick_timer.
    Tick,
    /// Reactive signal change detected — triggers a render pass.
    RenderRequest,
}
