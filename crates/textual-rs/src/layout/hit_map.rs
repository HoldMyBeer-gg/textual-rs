use std::collections::HashMap;
use ratatui::layout::Rect;
use crate::widget::WidgetId;

pub struct MouseHitMap;

impl MouseHitMap {
    pub fn build(_widgets_dfs: &[WidgetId], _layout_cache: &HashMap<WidgetId, Rect>) -> Self {
        unimplemented!("RED phase stub")
    }
    pub fn hit_test(&self, _col: u16, _row: u16) -> Option<WidgetId> {
        unimplemented!()
    }
}
