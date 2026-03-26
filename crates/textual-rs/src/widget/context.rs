use std::any::Any;
use std::cell::{Cell, RefCell};
use slotmap::{DenseSlotMap, SecondaryMap};
use super::WidgetId;
use super::Widget;
use crate::css::types::{ComputedStyle, Declaration, PseudoClassSet};
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
        }
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
}
