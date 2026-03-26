use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::context::AppContext;
use super::Widget;

/// A container that lays out children vertically (top to bottom).
pub struct Vertical {
    pub children: Vec<Box<dyn Widget>>,
}

impl Vertical {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }

    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Default for Vertical {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Vertical {
    fn widget_type_name(&self) -> &'static str {
        "Vertical"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Vertical { layout: vertical; width: 1fr; height: 1fr; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        // Children are owned — we return empty here since children are pre-registered.
        // Containers work by having children added directly to the widget tree.
        vec![]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Containers don't render themselves — children render at their computed positions.
    }
}

/// A container that lays out children horizontally (left to right).
pub struct Horizontal {
    pub children: Vec<Box<dyn Widget>>,
}

impl Horizontal {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }

    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self { children }
    }
}

impl Default for Horizontal {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Horizontal {
    fn widget_type_name(&self) -> &'static str {
        "Horizontal"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Horizontal { layout: horizontal; width: 1fr; height: 1fr; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Containers don't render themselves — children render at their computed positions.
    }
}
