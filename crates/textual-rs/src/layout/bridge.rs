use std::collections::HashMap;
use ratatui::layout::Rect;
use crate::widget::WidgetId;
use crate::widget::context::AppContext;

pub struct TaffyBridge;

impl TaffyBridge {
    pub fn new() -> Self { unimplemented!("RED phase stub") }
    pub fn sync_subtree(&mut self, _root: WidgetId, _ctx: &AppContext) { unimplemented!() }
    pub fn sync_dirty_subtree(&mut self, _root: WidgetId, _ctx: &AppContext) { unimplemented!() }
    pub fn compute_layout(&mut self, _screen_id: WidgetId, _cols: u16, _rows: u16) { unimplemented!() }
    pub fn rect_for(&self, _id: WidgetId) -> Option<Rect> { unimplemented!() }
    pub fn remove_subtree(&mut self, _root: WidgetId, _ctx: &AppContext) { unimplemented!() }
    pub fn layout_cache(&self) -> &HashMap<WidgetId, Rect> { unimplemented!() }
}
