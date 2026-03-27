use std::cell::RefCell;

use ratatui::{buffer::Buffer, layout::Rect};

use super::{context::AppContext, Widget, WidgetId};

/// A screen that blocks all keyboard and mouse input to screens beneath it
/// while it is on top of the screen stack.
///
/// `ModalScreen` is a transparent wrapper — it owns one inner widget that
/// becomes its only child. Size the inner widget with CSS (width, height,
/// margin, align) to position it on screen.
///
/// Input blocking is guaranteed by the framework: keyboard focus is always
/// scoped to the top screen, and the mouse hit-map is built from the top
/// screen only. Screens below a modal are frozen but not unmounted.
///
/// # Usage
///
/// Push a modal from any `on_action` handler using
/// [`AppContext::push_screen_deferred`](crate::widget::context::AppContext::push_screen_deferred):
///
/// ```no_run
/// # use textual_rs::widget::screen::ModalScreen;
/// # use textual_rs::widget::context::AppContext;
/// # use textual_rs::Widget;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// struct ConfirmDialog;
/// impl Widget for ConfirmDialog {
///     fn widget_type_name(&self) -> &'static str { "ConfirmDialog" }
///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
///     fn on_action(&self, action: &str, ctx: &AppContext) {
///         if action == "ok" || action == "cancel" {
///             ctx.pop_screen_deferred(); // dismiss the modal
///         }
///     }
/// }
///
/// struct MainScreen;
/// impl Widget for MainScreen {
///     fn widget_type_name(&self) -> &'static str { "MainScreen" }
///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
///     fn on_action(&self, action: &str, ctx: &AppContext) {
///         if action == "open_confirm" {
///             ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(ConfirmDialog))));
///         }
///     }
/// }
/// ```
///
/// # Dismissing a modal
///
/// Call [`AppContext::pop_screen_deferred`](crate::widget::context::AppContext::pop_screen_deferred)
/// from within the inner widget's `on_action`. Focus automatically returns to
/// the widget that was focused before the modal was opened.
pub struct ModalScreen {
    /// Inner screen widget. Moved into compose() on first call via RefCell.
    inner: RefCell<Option<Box<dyn Widget>>>,
    own_id: std::cell::Cell<Option<WidgetId>>,
}

impl ModalScreen {
    pub fn new(inner: Box<dyn Widget>) -> Self {
        Self {
            inner: RefCell::new(Some(inner)),
            own_id: std::cell::Cell::new(None),
        }
    }
}

impl Widget for ModalScreen {
    fn widget_type_name(&self) -> &'static str {
        "ModalScreen"
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    /// Returns the inner widget as a child. Called once at mount time.
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        if let Some(inner) = self.inner.borrow_mut().take() {
            vec![inner]
        } else {
            vec![]
        }
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // ModalScreen is a transparent container — layout and rendering happen
        // in the inner widget returned from compose().
    }
}
