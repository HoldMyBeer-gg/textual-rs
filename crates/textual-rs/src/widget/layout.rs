//! Layout container widgets for vertical and horizontal child arrangement.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::RefCell;

use super::context::AppContext;
use super::Widget;

/// A container that lays out children vertically (top to bottom).
pub struct Vertical {
    /// Child widgets yielded once via compose(), then drained.
    pending_children: RefCell<Vec<Box<dyn Widget>>>,
    /// Optional CSS classes applied to this container instance.
    css_classes: Vec<&'static str>,
}

impl Vertical {
    /// Create a new empty Vertical container.
    pub fn new() -> Self {
        Self {
            pending_children: RefCell::new(Vec::new()),
            css_classes: Vec::new(),
        }
    }

    /// Create a Vertical container pre-populated with the given children.
    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            pending_children: RefCell::new(children),
            css_classes: Vec::new(),
        }
    }

    /// Add a CSS class to this container (for styling via CSS selectors).
    pub fn with_class(mut self, class: &'static str) -> Self {
        self.css_classes.push(class);
        self
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

    fn classes(&self) -> &[&str] {
        &self.css_classes
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Vertical { layout: vertical; width: 1fr; height: 1fr; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        self.pending_children.borrow_mut().drain(..).collect()
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Containers don't render themselves — children render at their computed positions.
    }
}

/// A container that lays out children horizontally (left to right).
pub struct Horizontal {
    /// Child widgets yielded once via compose(), then drained.
    pending_children: RefCell<Vec<Box<dyn Widget>>>,
    /// Optional CSS classes applied to this container instance.
    css_classes: Vec<&'static str>,
}

impl Horizontal {
    /// Create a new empty Horizontal container.
    pub fn new() -> Self {
        Self {
            pending_children: RefCell::new(Vec::new()),
            css_classes: Vec::new(),
        }
    }

    /// Create a Horizontal container pre-populated with the given children.
    pub fn with_children(children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            pending_children: RefCell::new(children),
            css_classes: Vec::new(),
        }
    }

    /// Add a CSS class to this container (for styling via CSS selectors).
    pub fn with_class(mut self, class: &'static str) -> Self {
        self.css_classes.push(class);
        self
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

    fn classes(&self) -> &[&str] {
        &self.css_classes
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Horizontal { layout: horizontal; width: 1fr; height: 1fr; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        self.pending_children.borrow_mut().drain(..).collect()
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Containers don't render themselves — children render at their computed positions.
    }
}
