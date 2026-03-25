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
        }
    }
}
