use std::any::Any;
use std::cell::RefCell;
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
        }
    }

    /// Post a typed message from a widget.
    /// It will be dispatched via bubbling in the next event loop iteration.
    /// Takes &self so this can be called from on_event or on_action without borrow conflict.
    pub fn post_message(&self, source: WidgetId, message: impl Any + 'static) {
        self.message_queue.borrow_mut().push((source, Box::new(message)));
    }
}
