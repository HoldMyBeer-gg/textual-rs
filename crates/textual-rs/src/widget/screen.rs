use std::cell::RefCell;

use ratatui::{buffer::Buffer, layout::Rect};

use super::{context::AppContext, Widget, WidgetId};

/// A screen that blocks all keyboard and mouse input to screens beneath it
/// while it is on top of the screen stack.
///
/// Wrap any widget in `ModalScreen` to present it as a modal dialog:
///
/// ```no_run
/// # use textual_rs::widget::screen::ModalScreen;
/// # use textual_rs::Widget;
/// # use textual_rs::widget::context::AppContext;
/// struct PinDialog;
/// impl Widget for PinDialog {
///     fn widget_type_name(&self) -> &'static str { "PinDialog" }
///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut ratatui::buffer::Buffer) {}
/// }
///
/// // In a button handler:
/// // ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(PinDialog))));
/// ```
///
/// Input blocking is guaranteed by the framework: focus is always scoped to the
/// top screen, and the mouse hit-map is built from the top screen only.
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
