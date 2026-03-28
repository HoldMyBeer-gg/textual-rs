//! Widget trait, widget ID type, and all built-in widget implementations.
// Widget documentation is deferred to plan 10-04 — suppress the lint here until then.
#![allow(missing_docs)]
pub mod button;
pub mod checkbox;
pub mod screen;
pub mod collapsible;
pub mod context;
pub mod context_menu;
pub mod data_table;
pub mod directory_tree;
pub mod footer;
pub mod header;
pub mod input;
pub mod label;
pub mod layout;
pub mod list_view;
pub mod loading_indicator;
pub mod log;
pub mod markdown;
pub mod masked_input;
pub mod placeholder;
pub mod progress_bar;
pub mod radio;
pub mod rich_log;
pub mod scroll_region;
pub mod scroll_view;
pub mod select;
pub mod sparkline;
pub mod switch;
pub mod tabs;
pub mod text_area;
pub mod toast;
pub mod tree;
pub mod tree_view;

use crate::event::keybinding::KeyBinding;
use context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use slotmap::new_key_type;

/// Unique identifier for a widget in the arena (slotmap generational index).
/// Passed to `on_mount`, `on_action`, `post_message`, and `run_worker`.
new_key_type! { pub struct WidgetId; }

/// Controls whether an event continues bubbling up the widget tree after being handled.
///
/// Return `Stop` from `on_event` to consume the event and prevent parent widgets
/// from seeing it. Return `Continue` to let it bubble further.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPropagation {
    /// Keep bubbling — parent widgets will also receive this event.
    Continue,
    /// Stop bubbling — this widget consumed the event.
    Stop,
}

/// Core trait implemented by every UI node in textual-rs.
///
/// Widgets form a tree: App > Screen > Widget hierarchy. The framework
/// manages mounting, layout, rendering, and event dispatch.
///
/// # Minimal implementation
///
/// ```no_run
/// # use textual_rs::Widget;
/// # use textual_rs::widget::context::AppContext;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// struct MyWidget;
///
/// impl Widget for MyWidget {
///     fn widget_type_name(&self) -> &'static str { "MyWidget" }
///     fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
///         buf.set_string(area.x, area.y, "Hello!", ratatui::style::Style::default());
///     }
/// }
/// ```
pub trait Widget: 'static {
    /// Paint this widget's content into the terminal buffer.
    ///
    /// Called every render frame by the framework. Only draw inside `area` —
    /// it is pre-clipped to the widget's computed layout rectangle.
    ///
    /// Use `ctx.text_style(id)` to get the CSS-computed fg/bg style.
    /// Use `get_untracked()` on reactive values to avoid tracking loops.
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer);

    /// Declare child widgets. Called once at mount time to build the widget tree.
    ///
    /// Return a `Vec<Box<dyn Widget>>` of children. The framework inserts them
    /// into the arena and lays them out according to CSS rules.
    /// Container widgets typically implement this; leaf widgets return `vec![]`.
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![]
    }

    /// Called when this widget is inserted into the widget tree.
    ///
    /// Use this to store `own_id` for later use in `on_action` or `post_message`.
    fn on_mount(&self, _id: WidgetId) {}

    /// Called when this widget is removed from the widget tree.
    ///
    /// Use this to clear stored `own_id` and release resources.
    fn on_unmount(&self, _id: WidgetId) {}

    /// Whether this widget participates in Tab-based focus cycling.
    ///
    /// Returns `false` by default. Override to return `true` for interactive widgets.
    /// When focused, `key_bindings()` are active and a focus indicator is rendered.
    fn can_focus(&self) -> bool {
        false
    }

    /// Whether this screen blocks all keyboard and mouse input to screens beneath it.
    ///
    /// Returns `false` by default. Implement `is_modal() -> bool { true }` on any
    /// screen widget to make it behave as a modal dialog. See also [`screen::ModalScreen`].
    fn is_modal(&self) -> bool {
        false
    }

    /// The CSS type selector name for this widget (e.g., `"Button"`, `"Input"`).
    ///
    /// Used by the CSS engine to match style rules: `Button { color: red; }`.
    /// Must be unique per widget type. Convention: PascalCase matching the struct name.
    fn widget_type_name(&self) -> &'static str;

    /// CSS class names applied to this widget instance (e.g., `&["primary", "active"]`).
    ///
    /// Used for class selector rules: `.primary { background: green; }`.
    fn classes(&self) -> &[&str] {
        &[]
    }

    /// Element ID for this widget instance (used for `#id` CSS selectors).
    ///
    /// Returns `None` by default. Override to return a unique string ID.
    fn id(&self) -> Option<&str> {
        None
    }

    /// Built-in default CSS for this widget type (static version).
    ///
    /// Applied at lowest priority (before user stylesheets). Override to provide
    /// sensible defaults like `"Button { border: heavy; height: 3; }"`.
    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    /// Instance-callable version of `default_css()`. Override this alongside
    /// `default_css()` to return the same value — this version is callable on
    /// `dyn Widget` and used by the framework to collect default styles at mount time.
    fn widget_default_css(&self) -> &'static str {
        ""
    }

    /// Handle a dispatched event/message. Downcast to concrete types to handle.
    ///
    /// Called by the framework when an event is dispatched to this widget or bubbled
    /// up from a child. Use `downcast_ref::<T>()` to match specific message types.
    ///
    /// Return `EventPropagation::Stop` to consume the event (stops bubbling).
    /// Return `EventPropagation::Continue` to let it keep bubbling to parents.
    fn on_event(&self, _event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        EventPropagation::Continue
    }

    /// Declare key bindings for this widget.
    ///
    /// Bindings are checked when this widget has focus and a key event arrives.
    /// Each `KeyBinding` maps a key+modifier combo to an action string.
    /// Set `show: true` to display the binding in the Footer and command palette.
    fn key_bindings(&self) -> &[KeyBinding] {
        &[]
    }

    /// Handle a key binding action. Called when a key matching a binding is pressed.
    ///
    /// The `action` string matches the `action` field of the triggered `KeyBinding`.
    /// Widget state must be mutated via `Cell<T>` or `Reactive<T>` since this takes `&self`.
    fn on_action(&self, _action: &str, _ctx: &AppContext) {}

    /// Override the border color for this widget based on internal state.
    ///
    /// Returns `Some((r, g, b))` when the widget wants to override its CSS border color
    /// (e.g., Input with invalid content shows a red border). Returns `None` by default.
    fn border_color_override(&self) -> Option<(u8, u8, u8)> {
        None
    }

    /// Whether this widget is a transparent overlay (context menu, tooltip, etc.).
    /// Overlay widgets skip paint_chrome (no background fill, no border from CSS)
    /// and paint their own chrome in render(). This prevents overlays from
    /// erasing the underlying screen content.
    fn is_overlay(&self) -> bool {
        false
    }

    /// Return context menu items for right-click. Empty vec = no context menu.
    /// Override to provide widget-specific menu items.
    fn context_menu_items(&self) -> Vec<context_menu::ContextMenuItem> {
        Vec::new()
    }

    /// Return the action to trigger on mouse click, if any.
    ///
    /// Widgets that should activate on click (e.g. buttons, checkboxes, switches)
    /// override this to return the same action string their Space/Enter key binding uses.
    /// The framework calls `on_action(click_action, ctx)` after click-to-focus.
    fn click_action(&self) -> Option<&str> {
        None
    }

    /// Whether this widget currently has selected text.
    ///
    /// Used by the app event loop to route Ctrl+C to copy instead of quit
    /// when a text widget has an active selection. Returns `false` by default.
    fn has_text_selection(&self) -> bool {
        false
    }
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
