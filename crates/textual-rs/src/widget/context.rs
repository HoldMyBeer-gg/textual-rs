//! Application context passed to every widget for state and service access.
use super::toast::{push_toast, ToastEntry, ToastSeverity};
use super::Widget;
use super::WidgetId;
use crate::css::cascade::Stylesheet;
use crate::css::render_style;
use crate::css::theme::{self, Theme};
use crate::css::types::{ComputedStyle, Declaration, PseudoClassSet};
use crate::event::AppEvent;
use crate::terminal::{MouseCaptureStack, TerminalCaps};
use ratatui::style::Style;
use slotmap::{DenseSlotMap, SecondaryMap};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

/// Shared application state passed by reference to every widget callback.
///
/// Provides access to the widget arena, CSS computed styles, focus state, screen stack,
/// event/message queues, and service methods (push_screen, post_message, run_worker, toast).
pub struct AppContext {
    /// Widget arena — all mounted widgets stored by their [`WidgetId`].
    pub arena: DenseSlotMap<WidgetId, Box<dyn Widget>>,
    /// Parent-to-children mapping for the widget tree.
    pub children: SecondaryMap<WidgetId, Vec<WidgetId>>,
    /// Child-to-parent mapping for the widget tree.
    pub parent: SecondaryMap<WidgetId, Option<WidgetId>>,
    /// CSS-cascaded styles for each mounted widget.
    pub computed_styles: SecondaryMap<WidgetId, ComputedStyle>,
    /// Per-widget inline style declarations (set via `Widget::inline_styles`).
    pub inline_styles: SecondaryMap<WidgetId, Vec<Declaration>>,
    /// Per-widget dirty flag; set when the widget needs re-render.
    pub dirty: SecondaryMap<WidgetId, bool>,
    /// CSS pseudo-class state (hover, focus, etc.) for each widget.
    pub pseudo_classes: SecondaryMap<WidgetId, PseudoClassSet>,
    /// Currently focused widget, or `None` if nothing has focus.
    pub focused_widget: Option<WidgetId>,
    /// Currently hovered widget (under mouse cursor). Updated by MouseMove events.
    pub hovered_widget: Option<WidgetId>,
    /// Stack of active screen widget IDs. Top of stack is the active screen.
    pub screen_stack: Vec<WidgetId>,
    /// Saved focus state for each screen push. Parallel to screen_stack.
    /// `push_screen` saves `focused_widget` here; `pop_screen` restores it.
    pub focus_history: Vec<Option<WidgetId>>,
    /// Widgets scheduled to receive `on_mount` on the next event loop tick.
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
    /// Stack-based mouse capture state. Screens/widgets push/pop to temporarily
    /// enable or disable mouse capture without competing callers clobbering each other.
    pub mouse_capture_stack: MouseCaptureStack,
    /// Deferred mouse capture pushes from widgets (drained by event loop).
    pub pending_mouse_push: RefCell<Vec<bool>>,
    /// Deferred mouse capture pop count from widgets (drained by event loop).
    pub pending_mouse_pops: Cell<usize>,
    /// Per-widget loading state. When a widget's ID is present and true,
    /// render_widget_tree draws a spinner overlay on top of that widget.
    /// Manipulated via set_loading(). Uses SecondaryMap (same as computed_styles, dirty, etc.).
    pub loading_widgets: RefCell<SecondaryMap<WidgetId, bool>>,
    /// Global spinner tick counter. Incremented once per full_render_pass.
    /// All loading overlays and LoadingIndicator widgets use this for synchronized animation.
    pub spinner_tick: Cell<u8>,
    /// Stacked toast notifications, rendered bottom-right. Max 5 visible.
    pub toast_entries: RefCell<Vec<ToastEntry>>,
    /// Deferred push_screen_wait requests: each entry is `(screen_box, oneshot_sender)`.
    /// Drained by `process_deferred_screens`; the sender is stored keyed by the new screen's WidgetId.
    #[allow(clippy::type_complexity)]
    pub pending_screen_wait_pushes: RefCell<
        Vec<(
            Box<dyn Widget>,
            tokio::sync::oneshot::Sender<Box<dyn Any + Send>>,
        )>,
    >,
    /// Maps screen WidgetId -> oneshot sender for typed result delivery.
    /// Populated when `push_screen_wait` processes a deferred push; consumed when `pop_screen_with` fires.
    pub screen_result_senders:
        RefCell<HashMap<WidgetId, tokio::sync::oneshot::Sender<Box<dyn Any + Send>>>>,
    /// Single-slot typed result for the next `pop_screen_with` call.
    /// Set by `pop_screen_with`, consumed by `process_deferred_screens` when the pop fires.
    pub pending_pop_result: RefCell<Option<Box<dyn Any + Send>>>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    /// Create a new empty `AppContext` with default state and the dark theme.
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
            focus_history: Vec::new(),
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
            mouse_capture_stack: MouseCaptureStack::new(),
            pending_mouse_push: RefCell::new(Vec::new()),
            pending_mouse_pops: Cell::new(0),
            loading_widgets: RefCell::new(SecondaryMap::new()),
            spinner_tick: Cell::new(0),
            toast_entries: RefCell::new(Vec::new()),
            pending_screen_wait_pushes: RefCell::new(Vec::new()),
            screen_result_senders: RefCell::new(HashMap::new()),
            pending_pop_result: RefCell::new(None),
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

    /// Push a new screen onto the screen stack.
    ///
    /// The current screen is kept in memory; the new screen receives keyboard
    /// focus immediately. When the new screen is later popped, focus returns
    /// to the widget that was focused before the push.
    ///
    /// Call this from `on_action` (where only `&self` is available). The
    /// push is applied at the end of the current event cycle.
    ///
    /// To present a modal dialog that blocks input to all screens beneath it,
    /// wrap your widget in [`crate::widget::screen::ModalScreen`]:
    ///
    /// ```no_run
    /// # use textual_rs::widget::context::AppContext;
    /// # use textual_rs::widget::screen::ModalScreen;
    /// # use textual_rs::{Widget, WidgetId};
    /// # use ratatui::{buffer::Buffer, layout::Rect};
    /// struct ConfirmDialog;
    /// impl Widget for ConfirmDialog {
    ///     fn widget_type_name(&self) -> &'static str { "ConfirmDialog" }
    ///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    ///     fn on_action(&self, action: &str, ctx: &AppContext) {
    ///         if action == "confirm" || action == "cancel" {
    ///             ctx.pop_screen_deferred();
    ///         }
    ///     }
    /// }
    ///
    /// struct MyScreen;
    /// impl Widget for MyScreen {
    ///     fn widget_type_name(&self) -> &'static str { "MyScreen" }
    ///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    ///     fn on_action(&self, action: &str, ctx: &AppContext) {
    ///         if action == "open_dialog" {
    ///             ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(ConfirmDialog))));
    ///         }
    ///     }
    /// }
    /// ```
    pub fn push_screen_deferred(&self, screen: Box<dyn Widget>) {
        self.pending_screen_pushes.borrow_mut().push(screen);
    }

    /// Pop the top screen from the stack and restore focus to the previous screen.
    ///
    /// The popped screen and its entire widget subtree are unmounted. Focus
    /// returns to whichever widget was focused when the screen was pushed — or
    /// advances to the next focusable widget if that widget no longer exists.
    ///
    /// Call this from `on_action` (where only `&self` is available). The pop
    /// is applied at the end of the current event cycle.
    ///
    /// Calling `pop_screen_deferred` on the last remaining screen is a no-op.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use textual_rs::widget::context::AppContext;
    /// # use textual_rs::{Widget, WidgetId};
    /// # use ratatui::{buffer::Buffer, layout::Rect};
    /// struct Dialog;
    /// impl Widget for Dialog {
    ///     fn widget_type_name(&self) -> &'static str { "Dialog" }
    ///     fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    ///     fn on_action(&self, action: &str, ctx: &AppContext) {
    ///         match action {
    ///             "ok" | "cancel" | "close" => ctx.pop_screen_deferred(),
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    pub fn pop_screen_deferred(&self) {
        self.pending_screen_pops
            .set(self.pending_screen_pops.get() + 1);
    }

    /// Push a modal screen and asynchronously await a typed result.
    ///
    /// Returns a [`tokio::sync::oneshot::Receiver`] that resolves when the modal screen
    /// calls [`pop_screen_with`](AppContext::pop_screen_with). The caller downcasts the
    /// `Box<dyn Any>` to the expected type.
    ///
    /// Because `on_action` is synchronous, the typical usage pattern is to capture the
    /// receiver in a worker:
    ///
    /// ```ignore
    /// let rx = ctx.push_screen_wait(Box::new(ModalScreen::new(Box::new(dialog))));
    /// ctx.run_worker(self_id, async move {
    ///     if let Ok(boxed) = rx.await {
    ///         let confirmed: bool = *boxed.downcast::<bool>().unwrap();
    ///         confirmed
    ///     } else {
    ///         false
    ///     }
    /// });
    /// ```
    pub fn push_screen_wait(
        &self,
        screen: Box<dyn Widget>,
    ) -> tokio::sync::oneshot::Receiver<Box<dyn Any + Send>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_screen_wait_pushes
            .borrow_mut()
            .push((screen, tx));
        rx
    }

    /// Pop the top screen and deliver a typed result to the awaiting `push_screen_wait` caller.
    ///
    /// The value is boxed and stored; `process_deferred_screens` delivers it through the
    /// oneshot channel when the pop is processed. If the top screen was not pushed via
    /// `push_screen_wait`, the result is silently discarded and the pop still occurs normally.
    ///
    /// Call this from `on_action` in a modal's inner widget to dismiss and return a value.
    ///
    /// ```ignore
    /// // Inside a dialog widget's on_action:
    /// fn on_action(&self, action: &str, ctx: &AppContext) {
    ///     match action {
    ///         "ok"     => ctx.pop_screen_with(true),
    ///         "cancel" => ctx.pop_screen_with(false),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn pop_screen_with<T: Any + Send + 'static>(&self, value: T) {
        *self.pending_pop_result.borrow_mut() = Some(Box::new(value));
        self.pop_screen_deferred();
    }

    /// Post a typed message from a widget.
    /// It will be dispatched via bubbling in the next event loop iteration.
    /// Takes &self so this can be called from on_event or on_action without borrow conflict.
    pub fn post_message(&self, source: WidgetId, message: impl Any + 'static) {
        self.message_queue
            .borrow_mut()
            .push((source, Box::new(message)));
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
                Box::new(crate::worker::WorkerResult {
                    source_id,
                    value: result,
                }),
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
        progress_fn: impl FnOnce(flume::Sender<P>) -> std::pin::Pin<Box<dyn std::future::Future<Output = T>>>
            + 'static,
    ) -> tokio::task::AbortHandle
    where
        T: Send + 'static,
        P: Send + 'static,
    {
        let worker_tx = self.worker_tx.clone().expect(
            "worker_tx not initialized — run_worker_with_progress called outside App::run()",
        );

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

    /// Schedule a mouse capture push deferred to the next event loop tick.
    /// Use from `on_action(&self, ...)` or `on_event(&self, ...)` where only &self is available.
    pub fn push_mouse_capture(&self, enabled: bool) {
        self.pending_mouse_push.borrow_mut().push(enabled);
    }

    /// Schedule a mouse capture pop deferred to the next event loop tick.
    /// Use from `on_action(&self, ...)` or `on_event(&self, ...)` where only &self is available.
    pub fn pop_mouse_capture(&self) {
        self.pending_mouse_pops
            .set(self.pending_mouse_pops.get() + 1);
    }

    /// Set or clear the loading overlay for a widget.
    ///
    /// When loading is true, `render_widget_tree` will draw a spinner overlay
    /// on top of the widget's area after calling its `render()` method.
    /// When loading is false, the overlay is removed.
    ///
    /// This is the textual-rs equivalent of Python Textual's `widget.loading = True`.
    ///
    /// # Example
    /// ```ignore
    /// // In on_action or on_message:
    /// ctx.set_loading(self.own_id.get().unwrap(), true);
    /// // Start async work...
    /// // In worker result handler:
    /// ctx.set_loading(self.own_id.get().unwrap(), false);
    /// ```
    pub fn set_loading(&self, id: WidgetId, loading: bool) {
        let mut map = self.loading_widgets.borrow_mut();
        if loading {
            map.insert(id, true);
        } else {
            map.remove(id);
        }
    }

    /// Display a toast notification in the bottom-right corner.
    ///
    /// `severity` controls the border color: Info=$primary, Warning=$warning, Error=$error.
    /// `timeout_ms` controls auto-dismiss: 0 = persistent (never dismissed automatically).
    ///
    /// Maximum 5 toasts are shown simultaneously; adding a 6th drops the oldest.
    pub fn toast(&self, message: impl Into<String>, severity: ToastSeverity, timeout_ms: u64) {
        let mut toasts = self.toast_entries.borrow_mut();
        push_toast(&mut toasts, message.into(), severity, timeout_ms);
    }

    /// Display an Info toast with default 3000ms timeout.
    pub fn toast_info(&self, message: impl Into<String>) {
        self.toast(message, ToastSeverity::Info, 3000);
    }

    /// Request a clean application exit. The event loop will break after the current frame.
    pub fn quit(&self) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(AppEvent::Quit);
        }
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
