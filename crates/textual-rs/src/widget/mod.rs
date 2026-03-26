pub mod context;
pub mod tree;
pub mod label;
pub mod button;
pub mod checkbox;
pub mod switch;
pub mod input;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use slotmap::new_key_type;
use context::AppContext;
use crate::event::keybinding::KeyBinding;

new_key_type! { pub struct WidgetId; }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPropagation {
    Continue,
    Stop,
}

pub trait Widget: 'static {
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer);
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![]
    }
    fn on_mount(&self, _id: WidgetId) {}
    fn on_unmount(&self, _id: WidgetId) {}
    fn can_focus(&self) -> bool {
        false
    }
    fn widget_type_name(&self) -> &'static str;
    fn classes(&self) -> &[&str] {
        &[]
    }
    fn id(&self) -> Option<&str> {
        None
    }
    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    /// Handle a dispatched event/message. Downcast to concrete types to handle.
    /// Return Stop to consume the message, Continue to let it bubble.
    fn on_event(&self, _event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        EventPropagation::Continue
    }

    /// Declare key bindings for this widget.
    /// Checked when this widget has focus and a key event arrives.
    fn key_bindings(&self) -> &[KeyBinding] {
        &[]
    }

    /// Handle a key binding action. Called when a key matching a binding is pressed.
    fn on_action(&self, _action: &str, _ctx: &AppContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::context::AppContext;

    /// A minimal widget for testing object-safety and arena operations
    struct TestWidget {
        focusable: bool,
    }

    impl Widget for TestWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "TestWidget"
        }
        fn can_focus(&self) -> bool {
            self.focusable
        }
    }

    #[test]
    fn app_context_new_creates_empty_arena() {
        let ctx = AppContext::new();
        assert_eq!(ctx.arena.len(), 0);
        assert!(ctx.focused_widget.is_none());
        assert!(ctx.screen_stack.is_empty());
    }

    #[test]
    fn arena_insert_retrieve_remove() {
        let mut ctx = AppContext::new();
        let widget: Box<dyn Widget> = Box::new(TestWidget { focusable: false });

        // Insert into arena
        let id = ctx.arena.insert(widget);

        // Retrieve by WidgetId
        assert!(ctx.arena.contains_key(id));
        assert_eq!(ctx.arena[id].widget_type_name(), "TestWidget");

        // Remove
        let removed = ctx.arena.remove(id);
        assert!(removed.is_some());
        assert!(!ctx.arena.contains_key(id));
    }

    #[test]
    fn widget_is_object_safe_stored_as_box() {
        // This test verifies Box<dyn Widget> compiles (object-safety check)
        let widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(TestWidget { focusable: false }),
            Box::new(TestWidget { focusable: true }),
        ];
        assert_eq!(widgets.len(), 2);
        assert!(!widgets[0].can_focus());
        assert!(widgets[1].can_focus());
    }
}
