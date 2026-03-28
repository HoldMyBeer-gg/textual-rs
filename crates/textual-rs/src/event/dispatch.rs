//! Message dispatch: routes typed messages from the queue to matching widget handlers.

use crate::widget::context::AppContext;
use crate::widget::{EventPropagation, WidgetId};
use std::any::Any;

/// Collect the parent chain from `start` up to the screen root.
/// Returns [start, parent, grandparent, ...] in bottom-up order.
pub fn collect_parent_chain(start: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    let mut chain = vec![start];
    let mut current = start;
    while let Some(&Some(parent)) = ctx.parent.get(current) {
        chain.push(parent);
        current = parent;
    }
    chain
}

/// Dispatch a message through the widget tree via bubbling.
/// Calls on_event on each widget in the parent chain from `target` upward.
/// Stops when a handler returns EventPropagation::Stop.
pub fn dispatch_message(target: WidgetId, message: &dyn Any, ctx: &AppContext) -> EventPropagation {
    let chain = collect_parent_chain(target, ctx);
    for &id in &chain {
        if let Some(widget) = ctx.arena.get(id) {
            if widget.on_event(message, ctx) == EventPropagation::Stop {
                return EventPropagation::Stop;
            }
        }
    }
    EventPropagation::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::context::AppContext;
    use crate::widget::tree::mount_widget;
    use crate::widget::{EventPropagation, Widget};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use std::any::Any;
    use std::cell::Cell;

    /// Test message type
    struct PingMsg;

    /// Widget that always continues (does not handle messages)
    struct PassthroughWidget;
    impl Widget for PassthroughWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "PassthroughWidget"
        }
        fn on_event(&self, _event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
            EventPropagation::Continue
        }
    }

    /// Widget that stops all messages
    struct StopWidget;
    impl Widget for StopWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "StopWidget"
        }
        fn on_event(&self, _event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
            EventPropagation::Stop
        }
    }

    /// Widget that counts how many times on_event is called.
    /// Uses a Cell to allow mutation from &self (needed since Widget::on_event takes &self).
    struct CountingWidget {
        count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }
    impl CountingWidget {
        fn new(count: std::sync::Arc<std::sync::atomic::AtomicU32>) -> Self {
            CountingWidget { count }
        }
    }
    impl Widget for CountingWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "CountingWidget"
        }
        fn on_event(&self, _event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
            self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            EventPropagation::Continue
        }
    }

    #[test]
    fn collect_parent_chain_single_node() {
        let mut ctx = AppContext::new();
        let id = mount_widget(Box::new(PassthroughWidget), None, &mut ctx);
        let chain = collect_parent_chain(id, &ctx);
        assert_eq!(chain, vec![id]);
    }

    #[test]
    fn collect_parent_chain_three_nodes() {
        let mut ctx = AppContext::new();
        let root = mount_widget(Box::new(PassthroughWidget), None, &mut ctx);
        let child = mount_widget(Box::new(PassthroughWidget), Some(root), &mut ctx);
        let leaf = mount_widget(Box::new(PassthroughWidget), Some(child), &mut ctx);

        let chain = collect_parent_chain(leaf, &ctx);
        // Should be [leaf, child, root]
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], leaf);
        assert_eq!(chain[1], child);
        assert_eq!(chain[2], root);
    }

    #[test]
    fn dispatch_message_single_widget_stop() {
        let mut ctx = AppContext::new();
        let id = mount_widget(Box::new(StopWidget), None, &mut ctx);

        let msg = PingMsg;
        let result = dispatch_message(id, &msg, &ctx);
        assert_eq!(result, EventPropagation::Stop);
    }

    #[test]
    fn dispatch_message_bubbles_through_chain() {
        use std::sync::atomic::AtomicU32;
        use std::sync::Arc;

        let count_root = Arc::new(AtomicU32::new(0));
        let count_child = Arc::new(AtomicU32::new(0));
        let count_leaf = Arc::new(AtomicU32::new(0));

        let mut ctx = AppContext::new();
        let root = mount_widget(
            Box::new(CountingWidget::new(count_root.clone())),
            None,
            &mut ctx,
        );
        let child = mount_widget(
            Box::new(CountingWidget::new(count_child.clone())),
            Some(root),
            &mut ctx,
        );
        let leaf = mount_widget(
            Box::new(CountingWidget::new(count_leaf.clone())),
            Some(child),
            &mut ctx,
        );

        let msg = PingMsg;
        let result = dispatch_message(leaf, &msg, &ctx);

        assert_eq!(result, EventPropagation::Continue);
        // All three should have received the message
        assert_eq!(count_leaf.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(count_child.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(count_root.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn dispatch_message_stops_at_middle_widget() {
        use std::sync::atomic::AtomicU32;
        use std::sync::Arc;

        let count_root = Arc::new(AtomicU32::new(0));
        let count_leaf = Arc::new(AtomicU32::new(0));

        let mut ctx = AppContext::new();
        let root = mount_widget(
            Box::new(CountingWidget::new(count_root.clone())),
            None,
            &mut ctx,
        );
        // Middle widget stops the message
        let child = mount_widget(Box::new(StopWidget), Some(root), &mut ctx);
        let leaf = mount_widget(
            Box::new(CountingWidget::new(count_leaf.clone())),
            Some(child),
            &mut ctx,
        );

        let msg = PingMsg;
        let result = dispatch_message(leaf, &msg, &ctx);

        // Leaf receives (Continue), middle stops, root never called
        assert_eq!(result, EventPropagation::Stop);
        assert_eq!(count_leaf.load(std::sync::atomic::Ordering::SeqCst), 1);
        // Root should NOT have been called
        assert_eq!(count_root.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn dispatch_message_no_handlers_returns_continue() {
        let mut ctx = AppContext::new();
        let root = mount_widget(Box::new(PassthroughWidget), None, &mut ctx);
        let child = mount_widget(Box::new(PassthroughWidget), Some(root), &mut ctx);

        let msg = PingMsg;
        let result = dispatch_message(child, &msg, &ctx);
        assert_eq!(result, EventPropagation::Continue);
    }
}
