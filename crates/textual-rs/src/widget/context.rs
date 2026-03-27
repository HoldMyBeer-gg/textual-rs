use std::any::Any;
use std::cell::{Cell, RefCell};
use ratatui::style::Style;
use slotmap::{DenseSlotMap, SecondaryMap};
use super::WidgetId;
use super::Widget;
use crate::css::cascade::Stylesheet;
use crate::css::theme::{self, Theme};
use crate::css::types::{ComputedStyle, Declaration, PseudoClassSet};
use crate::css::render_style;
use crate::event::AppEvent;
use crate::terminal::TerminalCaps;

pub struct AppContext {
    pub arena: DenseSlotMap<WidgetId, Box<dyn Widget>>,
    pub children: SecondaryMap<WidgetId, Vec<WidgetId>>,
    pub parent: SecondaryMap<WidgetId, Option<WidgetId>>,
    pub computed_styles: SecondaryMap<WidgetId, ComputedStyle>,
    pub inline_styles: SecondaryMap<WidgetId, Vec<Declaration>>,
    pub dirty: SecondaryMap<WidgetId, bool>,
    pub pseudo_classes: SecondaryMap<WidgetId, PseudoClassSet>,
    pub focused_widget: Option<WidgetId>,
    /// Currently hovered widget (under mouse cursor). Updated by MouseMove events.
    pub hovered_widget: Option<WidgetId>,
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
    /// Active theme for CSS variable resolution (e.g., `$primary`, `$accent-lighten-2`).
    /// Defaults to `default_dark_theme()`. Set a custom theme to change all variable colors.
    pub theme: Theme,
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
    /// Active floating overlay (context menu, etc.). Rendered last, on top of everything.
    /// Not part of the widget tree — painted directly to the frame buffer at absolute coords.
    pub active_overlay: RefCell<Option<Box<dyn Widget>>>,
    /// Deferred overlay dismissal flag. Set by dismiss_overlay(), drained after event handling.
    pub pending_overlay_dismiss: Cell<bool>,
    /// Detected terminal capabilities (color depth, unicode, mouse, title).
    /// Widgets can inspect this to degrade gracefully on limited terminals.
    pub terminal_caps: TerminalCaps,
    /// When true, animations snap to their target value instead of interpolating.
    /// Set by TestApp to ensure deterministic rendering in tests.
    pub skip_animations: bool,
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
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
            hovered_widget: None,
            screen_stack: Vec::new(),
            pending_mounts: Vec::new(),
            input_buffer: String::new(),
            event_tx: None,
            message_queue: RefCell::new(Vec::new()),
            pending_screen_pushes: RefCell::new(Vec::new()),
            pending_screen_pops: Cell::new(0),
            theme: theme::default_dark_theme(),
            stylesheets: Vec::new(),
            worker_tx: None,
            worker_handles: RefCell::new(SecondaryMap::new()),
            pending_recompose: RefCell::new(Vec::new()),
            active_overlay: RefCell::new(None),
            pending_overlay_dismiss: Cell::new(false),
            terminal_caps: crate::terminal::detect_capabilities(),
            skip_animations: false,
        }
    }

    /// Set the active theme, replacing all CSS variable colors.
    /// After calling this, a full re-cascade should be triggered to apply new theme colors.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Schedule a widget for recomposition on the next event loop tick.
    /// Used by widgets like TabbedContent when their compose() output changes.
    pub fn request_recompose(&self, id: WidgetId) {
        self.pending_recompose.borrow_mut().push(id);
    }

    /// Schedule the active overlay for dismissal. Actual removal happens after the
    /// current event handler returns (avoids RefCell borrow conflict).
    pub fn dismiss_overlay(&self) {
        self.pending_overlay_dismiss.set(true);
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
            .or_default()
            .push(abort.clone());
        abort
    }

    /// Spawn an async worker with a progress channel. The worker receives a
    /// `flume::Sender<P>` for sending progress updates, and its final result is
    /// delivered as a `WorkerResult<T>` message. Progress updates are delivered
    /// as `WorkerProgress<P>` messages to the source widget.
    ///
    /// # Example
    /// ```ignore
    /// ctx.run_worker_with_progress(my_id, |progress_tx| {
    ///     Box::pin(async move {
    ///         for i in 0..100 {
    ///             let _ = progress_tx.send(i as f32 / 100.0);
    ///             tokio::time::sleep(Duration::from_millis(50)).await;
    ///         }
    ///         "done"
    ///     })
    /// });
    /// ```
    pub fn run_worker_with_progress<T, P>(
        &self,
        source_id: WidgetId,
        progress_fn: impl FnOnce(flume::Sender<P>) -> std::pin::Pin<Box<dyn std::future::Future<Output = T>>> + 'static,
    ) -> tokio::task::AbortHandle
    where
        T: Send + 'static,
        P: Send + 'static,
    {
        let worker_tx = self.worker_tx.clone()
            .expect("worker_tx not initialized — run_worker_with_progress called outside App::run()");

        let (progress_sender, progress_receiver) = flume::unbounded::<P>();

        // Spawn progress forwarding task — receives P from the worker and wraps
        // it as a WorkerProgress<P> message to the owning widget.
        let ptx = worker_tx.clone();
        let sid = source_id;
        tokio::task::spawn_local(async move {
            while let Ok(p) = progress_receiver.recv_async().await {
                let msg = crate::worker::WorkerProgress {
                    source_id: sid,
                    progress: p,
                };
                let _ = ptx.send((sid, Box::new(msg)));
            }
        });

        // Create the main future using the progress sender
        let fut = progress_fn(progress_sender);
        self.run_worker(source_id, fut)
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
            .map(render_style::text_style)
            .unwrap_or_default()
    }
}
