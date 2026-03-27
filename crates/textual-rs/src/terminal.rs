use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use std::panic;

/// Install a panic hook that restores the terminal before printing the panic message.
/// MUST be called before TerminalGuard::new() (before entering raw mode).
pub fn init_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );
        original_hook(panic_info);
    }));
}

/// RAII guard that enters raw mode + alt screen + mouse capture on creation and restores on drop.
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );
    }
}

// ---------------------------------------------------------------------------
// Mouse capture stack
// ---------------------------------------------------------------------------

/// Stack-based mouse capture state. The effective state is the top of the stack,
/// defaulting to `true` (captured) when empty. Screens/widgets push to temporarily
/// override; pop to restore. This prevents competing enable/disable calls from
/// clobbering each other.
#[derive(Debug, Clone)]
pub struct MouseCaptureStack {
    stack: Vec<bool>,
}

impl MouseCaptureStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Current effective mouse-capture state. True = terminal captures mouse events;
    /// false = pass-through to terminal emulator for native selection.
    pub fn is_enabled(&self) -> bool {
        self.stack.last().copied().unwrap_or(true)
    }

    /// Push a new capture state. Returns the previous is_enabled() value
    /// so the caller can detect transitions.
    pub fn push(&mut self, enabled: bool) -> bool {
        let prev = self.is_enabled();
        self.stack.push(enabled);
        prev
    }

    /// Pop the top capture state. Returns the new is_enabled() value.
    /// No-op if stack is empty (default state cannot be popped).
    pub fn pop(&mut self) -> bool {
        self.stack.pop();
        self.is_enabled()
    }

    /// Reset to default state (empty stack = captured). Used by resize guard.
    pub fn reset(&mut self) {
        self.stack.clear();
    }
}

impl Default for MouseCaptureStack {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Terminal capability detection
// ---------------------------------------------------------------------------

/// Color depth level supported by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// No color (dumb terminal)
    NoColor,
    /// 16 standard ANSI colors
    Standard,
    /// 256 color palette (xterm-256color)
    EightBit,
    /// 24-bit true color (16M colors)
    TrueColor,
}

/// Detected terminal capabilities.
///
/// Use [`TerminalCaps::detect()`] to probe the current environment. Widgets and
/// the rendering layer can inspect these fields to degrade gracefully on limited
/// terminals (e.g., fall back to 256 colors or ASCII-only borders).
#[derive(Debug, Clone)]
pub struct TerminalCaps {
    /// Color depth the terminal advertises.
    pub color_depth: ColorDepth,
    /// Whether the terminal supports Unicode (UTF-8 locale or Windows Terminal).
    pub unicode: bool,
    /// Whether mouse events are available (crossterm always enables this).
    pub mouse: bool,
    /// Whether the terminal supports setting the window title.
    pub title: bool,
}

impl TerminalCaps {
    /// Detect terminal capabilities from environment variables and platform heuristics.
    ///
    /// **Color depth detection (in priority order):**
    /// 1. `COLORTERM` env var contains "truecolor" or "24bit" -> TrueColor
    /// 2. `TERM` env var contains "256color" -> EightBit
    /// 3. Windows: `WT_SESSION` present (Windows Terminal) -> TrueColor, else EightBit
    /// 4. `TERM` is "dumb" -> NoColor
    /// 5. Fallback -> Standard (16 colors)
    ///
    /// **Unicode detection:**
    /// 1. `LC_ALL` or `LANG` contains "UTF-8" or "utf8" (case-insensitive) -> true
    /// 2. Windows: assume true (modern conhost + Windows Terminal handle Unicode)
    /// 3. `TERM` is in xterm family -> true
    /// 4. Fallback -> false
    ///
    /// **Mouse:** Always true (crossterm enables mouse capture).
    ///
    /// **Title:** true unless `TERM` is "dumb" or "linux" (Linux virtual console).
    pub fn detect() -> Self {
        let color_depth = detect_color_depth();
        let unicode = detect_unicode();
        let title = detect_title_support();

        Self {
            color_depth,
            unicode,
            mouse: true, // crossterm always enables mouse capture
            title,
        }
    }
}

/// Module-level convenience function equivalent to [`TerminalCaps::detect()`].
pub fn detect_capabilities() -> TerminalCaps {
    TerminalCaps::detect()
}

fn detect_color_depth() -> ColorDepth {
    // 1. COLORTERM is the strongest signal
    if let Ok(ct) = std::env::var("COLORTERM") {
        let ct_lower = ct.to_lowercase();
        if ct_lower.contains("truecolor") || ct_lower.contains("24bit") {
            return ColorDepth::TrueColor;
        }
    }

    // 2. TERM containing 256color
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") {
            return ColorDepth::EightBit;
        }
        if term == "dumb" {
            return ColorDepth::NoColor;
        }
    }

    // 3. Windows-specific heuristics
    #[cfg(target_os = "windows")]
    {
        // Windows Terminal sets WT_SESSION
        if std::env::var("WT_SESSION").is_ok() {
            return ColorDepth::TrueColor;
        }
        // Modern Windows 10+ conhost supports 256 colors
        ColorDepth::EightBit
    }

    // 4. Fallback: 16 standard colors
    #[cfg(not(target_os = "windows"))]
    ColorDepth::Standard
}

fn detect_unicode() -> bool {
    // 1. Check locale env vars
    for var_name in &["LC_ALL", "LANG", "LC_CTYPE"] {
        if let Ok(val) = std::env::var(var_name) {
            let val_upper = val.to_uppercase();
            if val_upper.contains("UTF-8") || val_upper.contains("UTF8") {
                return true;
            }
        }
    }

    // 2. Windows: modern terminals handle Unicode
    #[cfg(target_os = "windows")]
    {
        true
    }

    // 3. xterm family usually supports Unicode
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(term) = std::env::var("TERM") {
            if term.starts_with("xterm") || term.starts_with("rxvt") || term.contains("256color") {
                return true;
            }
        }
        false
    }
}

fn detect_title_support() -> bool {
    if let Ok(term) = std::env::var("TERM") {
        // Linux virtual console and dumb terminals don't support titles
        if term == "dumb" || term == "linux" {
            return false;
        }
    }
    // On Windows and most other terminals, title is supported
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_caps_detect_returns_valid_struct() {
        let caps = TerminalCaps::detect();
        // Mouse is always true
        assert!(caps.mouse, "mouse should always be true");
        // color_depth should be one of the valid variants
        match caps.color_depth {
            ColorDepth::NoColor
            | ColorDepth::Standard
            | ColorDepth::EightBit
            | ColorDepth::TrueColor => {}
        }
    }

    #[test]
    fn terminal_caps_color_depth_equality() {
        assert_eq!(ColorDepth::TrueColor, ColorDepth::TrueColor);
        assert_ne!(ColorDepth::TrueColor, ColorDepth::EightBit);
        assert_ne!(ColorDepth::Standard, ColorDepth::NoColor);
    }

    #[test]
    fn terminal_detect_capabilities_convenience() {
        let caps = detect_capabilities();
        assert!(caps.mouse);
        // Just ensure it doesn't panic and returns a valid struct
    }

    #[test]
    fn terminal_caps_clone_and_debug() {
        let caps = TerminalCaps::detect();
        let cloned = caps.clone();
        assert_eq!(caps.color_depth, cloned.color_depth);
        assert_eq!(caps.unicode, cloned.unicode);
        assert_eq!(caps.mouse, cloned.mouse);
        assert_eq!(caps.title, cloned.title);
        // Debug formatting should not panic
        let _debug = format!("{:?}", caps);
    }

    #[test]
    fn terminal_color_depth_detection_windows() {
        // On Windows (our CI/dev platform), detect should return at least EightBit
        #[cfg(target_os = "windows")]
        {
            let caps = TerminalCaps::detect();
            assert!(
                caps.color_depth == ColorDepth::EightBit
                    || caps.color_depth == ColorDepth::TrueColor,
                "Windows should detect at least 256 colors, got {:?}",
                caps.color_depth
            );
        }
    }

    #[test]
    fn terminal_unicode_detection_windows() {
        #[cfg(target_os = "windows")]
        {
            let caps = TerminalCaps::detect();
            assert!(caps.unicode, "Windows should detect Unicode support");
        }
    }
}
