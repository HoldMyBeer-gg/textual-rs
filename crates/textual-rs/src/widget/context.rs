use std::any::Any;
use std::cell::{Cell, RefCell};
use ratatui::style::Style;
use slotmap::{DenseSlotMap, SecondaryMap};
use super::WidgetId;
use super::Widget;
use crate::css::cascade::Stylesheet;
use crate::css::types::{ComputedStyle, Declaration, PseudoClassSet};
use crate::css::render_style;
use crate::event::AppEvent;

pub struct AppContext {
    pub arena: DenseSlotMap<WidgetId, Box<dyn Widget>>,
    pub children: SecondaryMap<WidgetId, Vec<WidgetId>>,
    pub parent: SecondaryMap<WidgetId, Option<WidgetId>>,
    pub computed_styles: SecondaryMap<WidgetId, ComputedStyle>,
    pub inline_styles: SecondaryMap<WidgetId, Vec<Declaration>>,
    pub dirty: SecondaryMap<WidgetId, bool>,
    pub pseudo_classes: SecondaryMap<WidgetId, PseudoClassSet>,
    pub focused_widget: Option<WidgetId>,
    pub screen_stack: Vec<WidgetId>,
    pub pending_mounts: Vec<WidgetId>,
    /// Temporary input buffer for demo purposes (Phase 3 replaces with proper reactive state).
    pub input_buffer: String,
    /// Event bus sender — widgets and reactive effects post events here.
    pub event_tx: Option<flume::Sender<AppEvent>>,
    /// Message queue for widget-to-widget communication.
    /// Uses RefCell so widgets can post messages from &self (on_event/on_action) without &mut.
    /// Drained by the event loop after each event is processed.
    pub message_queue: RefCell<Vec<(WidgetId, Box<dyn Any>)>>,
    /// Deferred screen pushes from widgets.
    /// Widgets in on_action(&self) can use push_screen_deferred() to schedule a new screen push
    /// without needing &mut AppContext. The event loop drains this after each action.
    pub pending_screen_pushes: RefCell<Vec<Box<dyn Widget>>>,
    /// Number of screens to pop, deferred from widgets.
    /// Widgets in on_action(&self) use pop_screen_deferred() to schedule a screen pop.
    /// The event loop drains this counter after each action cycle.
    pub pending_screen_pops: Cell<usize>,
    /// User stylesheets — stored here so ad-hoc pane rendering can resolve styles.
    pub stylesheets: Vec<Stylesheet>,
    /// Dedicated channel for worker results. Set by App::run_async before the event loop starts.
    /// Workers send (WidgetId, Box<dyn Any + Send>) through this channel to the event loop.
    pub worker_tx: Option<flume::Sender<(WidgetId, Box<dyn Any + Send>)>>,
    /// Per-widget abort handles for active workers. Used for auto-cancellation on unmount.
    pub worker_handles: RefCell<SecondaryMap<WidgetId, Vec<tokio::task::AbortHandle>>>,
    /// Widgets that need recomposition (e.g. TabbedContent after tab switch).
    /// Drained by the event loop after each event cycle.
    pub pending_recompose: RefCell<Vec<WidgetId>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            arena: DenseSlotMap::with_key(),
            children: SecondaryMap::new(),
            parent: SecondaryMap::new(),
            computed_styles: SecondaryMap::new(),
            inline_styles: SecondaryMap::new(),
            dirty: SecondaryMap::new(),
            pseudo_classes: SecondaryMap::new(),
            focused_widget: None,
            screen_stack: Vec::new(),
            pending_mounts: Vec::new(),
            input_buffer: String::new(),
            event_tx: None,
            message_queue: RefCell::new(Vec::new()),
            pending_screen_pushes: RefCell::new(Vec::new()),
            pending_screen_pops: Cell::new(0),
            stylesheets: Vec::new(),
            worker_tx: None,
            worker_handles: RefCell::new(SecondaryMap::new()),
            pending_recompose: RefCell::new(Vec::new()),
        }
    }

    /// Schedule a widget for recomposition on the next event loop tick.
    /// Used by widgets like TabbedContent when their compose() output changes.
    pub fn request_recompose(&self, id: WidgetId) {
        self.pending_recompose.borrow_mut().push(id);
    }

    /// Schedule a new screen push deferred to the next event loop tick.
    /// Use this from `on_action(&self, ...)` where only &self is available.
    /// The event loop drains `pending_screen_pushes` after each event cycle.
    pub fn push_screen_deferred(&self, screen: Box<dyn Widget>) {
        self.pending_screen_pushes.borrow_mut().push(screen);
    }

    /// Schedule a screen pop deferred to the next event loop tick.
    /// Use this from `on_action(&self, ...)` where only &self is available.
    /// The event loop drains `pending_screen_pops` after each event cycle.
    pub fn pop_screen_deferred(&self) {
        self.pending_screen_pops.set(self.pending_screen_pops.get() + 1);
    }

    /// Post a typed message from a widget.
    /// It will be dispatched via bubbling in the next event loop iteration.
    /// Takes &self so this can be called from on_event or on_action without borrow conflict.
    pub fn post_message(&self, source: WidgetId, message: impl Any + 'static) {
        self.message_queue.borrow_mut().push((source, Box::new(message)));
    }

    /// Convenience alias: post a message that bubbles up from the source widget.
    /// Equivalent to post_message — provided for API symmetry with Python Textual's notify().
    pub fn notify(&self, source: WidgetId, message: impl Any + 'static) {
        self.post_message(source, message);
    }

    /// Spawn an async worker tied to a widget. The worker runs on the Tokio LocalSet.
    /// On completion, the result is delivered as a `WorkerResult<T>` message to the
    /// source widget via the message queue. T must be Send + 'static.
    ///
    /// Returns an AbortHandle for manual cancellation. Workers are also automatically
    /// cancelled when the owning widget is unmounted.
    ///
    /// # Panics
    /// Panics if called outside of App::run() (worker_tx not initialized).
    pub fn run_worker<T: Send + 'static>(
        &self,
        source_id: WidgetId,
        fut: impl std::future::Future<Output = T> + 'static,
    ) -> tokio::task::AbortHandle {
        let tx = self
            .worker_tx
            .clone()
            .expect("worker_tx not initialized — run_worker called outside App::run()");
        let handle = tokio::task::spawn_local(async move {
            let result = fut.await;
            let _ = tx.send((
                source_id,
                Box::new(crate::worker::WorkerResult { source_id, value: result }),
            ));
        });
        let abort = handle.abort_handle();
        // Track handle for auto-cancel on unmount
        self.worker_handles
            .borrow_mut()
            .entry(source_id)
            .unwrap()
            .or_insert_with(Vec::new)
            .push(abort.clone());
        abort
    }

    /// Cancel all workers associated with a widget. Called automatically during unmount.
    pub fn cancel_workers(&self, widget_id: WidgetId) {
        if let Some(handles) = self.worker_handles.borrow_mut().remove(widget_id) {
            for handle in handles {
                handle.abort();
            }
        }
    }

    /// Get the ratatui text style (fg + bg) for a widget from its computed CSS.
    /// Returns Style::default() if the widget has no computed style.
    pub fn text_style(&self, id: WidgetId) -> Style {
        self.computed_styles
            .get(id)
            .map(|cs| render_style::text_style(cs))
            .unwrap_or_default()
    }
}
