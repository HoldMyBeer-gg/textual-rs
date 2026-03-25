// Placeholder — full implementation in Task 2.
use std::io;

/// RAII guard that enters raw mode + alt screen on creation and restores on drop.
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        unimplemented!("implemented in Task 2")
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {}
}
