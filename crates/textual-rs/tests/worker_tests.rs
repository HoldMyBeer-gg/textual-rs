/// Worker API and notify() integration tests.
///
/// Worker tests require a Tokio LocalSet (spawn_local) so they use #[tokio::test]
/// with an explicit LocalSet. Notify/post_message tests are synchronous.
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use textual_rs::widget::context::AppContext;
use textual_rs::widget::tree::{mount_widget, unmount_widget};
use textual_rs::widget::{EventPropagation, Widget, WidgetId};
use textual_rs::WorkerResult;

// ---- Minimal widget helpers ----

struct NoopWidget;

impl Widget for NoopWidget {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str { "NoopWidget" }
}

/// Widget that sets a flag when it receives WorkerResult<u32>
struct WorkerResultWidget {
    got_result: Arc<AtomicBool>,
    result_value: Arc<std::sync::Mutex<Option<u32>>>,
}

impl WorkerResultWidget {
    fn new(flag: Arc<AtomicBool>, value: Arc<std::sync::Mutex<Option<u32>>>) -> Self {
        WorkerResultWidget { got_result: flag, result_value: value }
    }
}

impl Widget for WorkerResultWidget {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str { "WorkerResultWidget" }

    fn on_event(&self, event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
        if let Some(result) = event.downcast_ref::<WorkerResult<u32>>() {
            self.got_result.store(true, Ordering::SeqCst);
            *self.result_value.lock().unwrap() = Some(result.value);
        }
        EventPropagation::Continue
    }
}

// ---- Worker result delivery test ----

/// Verify that ctx.run_worker delivers WorkerResult<T> via the worker_tx channel.
/// We manually drive the worker by running a LocalSet, then check the worker_tx
/// receiver for the delivered result.
#[tokio::test]
async fn worker_result_delivered() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let mut ctx = AppContext::new();

        // Set up a dedicated worker channel (normally done by App::run_async)
        let (worker_tx, worker_rx) = flume::unbounded::<(WidgetId, Box<dyn Any + Send>)>();
        ctx.worker_tx = Some(worker_tx);

        // Mount a widget
        let id = mount_widget(Box::new(NoopWidget), None, &mut ctx);

        // Spawn worker that returns 42u32
        let _handle = ctx.run_worker(id, async { 42u32 });

        // Yield to allow the LocalSet task to run
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        // Worker result should have arrived on the channel
        let (source_id, payload) = worker_rx.try_recv().expect("worker result not delivered");
        assert_eq!(source_id, id, "result source_id should match the worker's widget id");

        // Payload should be WorkerResult<u32>
        let result = payload.downcast_ref::<WorkerResult<u32>>()
            .expect("payload should be WorkerResult<u32>");
        assert_eq!(result.value, 42u32);
    }).await;
}

// ---- Worker result goes through message queue ----

/// Verify the full worker result pipeline: worker_rx -> message_queue -> on_event dispatch.
/// We simulate what App::run_async does: read worker_rx, push to message_queue, then drain.
#[tokio::test]
async fn worker_result_dispatched_via_message_queue() {
    use textual_rs::event::dispatch::dispatch_message;

    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let got_result = Arc::new(AtomicBool::new(false));
        let result_value = Arc::new(std::sync::Mutex::new(None::<u32>));

        let mut ctx = AppContext::new();
        let (worker_tx, worker_rx) = flume::unbounded::<(WidgetId, Box<dyn Any + Send>)>();
        ctx.worker_tx = Some(worker_tx);

        let id = mount_widget(
            Box::new(WorkerResultWidget::new(got_result.clone(), result_value.clone())),
            None,
            &mut ctx,
        );

        // Spawn worker
        let _handle = ctx.run_worker(id, async { 99u32 });

        // Yield to let worker task complete
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        // Simulate what the App event loop does: move result to message_queue
        while let Ok((source_id, payload)) = worker_rx.try_recv() {
            ctx.message_queue.borrow_mut().push((source_id, payload));
        }

        // Drain message queue manually (same as App::drain_message_queue but inline for test)
        let messages: Vec<_> = ctx.message_queue.borrow_mut().drain(..).collect();
        for (source, message) in messages {
            dispatch_message(source, message.as_ref(), &ctx);
        }

        assert!(got_result.load(Ordering::SeqCst), "widget should have received WorkerResult");
        assert_eq!(*result_value.lock().unwrap(), Some(99u32));
    }).await;
}

// ---- Worker cancelled on unmount ----

/// Verify that workers associated with a widget are cancelled when the widget is unmounted.
#[tokio::test]
async fn worker_cancelled_on_unmount() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let mut ctx = AppContext::new();
        let (worker_tx, _worker_rx) = flume::unbounded::<(WidgetId, Box<dyn Any + Send>)>();
        ctx.worker_tx = Some(worker_tx);

        // Mount widget
        let id = mount_widget(Box::new(NoopWidget), None, &mut ctx);

        // Spawn a long-running worker (60 second sleep — will never complete naturally)
        let abort = ctx.run_worker(id, async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            0u32
        });

        // Verify the worker handle is tracked
        assert!(
            ctx.worker_handles.borrow().contains_key(id),
            "worker handle should be tracked for widget"
        );

        // Unmount the widget — should cancel workers
        unmount_widget(id, &mut ctx);

        // Abort handle should now be finished (aborted)
        // Yield to allow the abort to propagate
        tokio::task::yield_now().await;
        assert!(abort.is_finished(), "worker task should be aborted after widget unmount");

        // Worker handles entry should have been removed
        assert!(
            !ctx.worker_handles.borrow().contains_key(id),
            "worker handles should be removed after unmount"
        );
    }).await;
}

// ---- notify() bubbles to parent ----

/// Verify ctx.notify() posts a message that bubbles up from child to parent.
///
/// Parent widget sets a flag when it receives TestMessage.
struct FlagWidget {
    received: Arc<AtomicBool>,
}

impl FlagWidget {
    fn new(flag: Arc<AtomicBool>) -> Self {
        FlagWidget { received: flag }
    }
}

impl Widget for FlagWidget {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str { "FlagWidget" }

    fn on_event(&self, event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
        if event.downcast_ref::<TestMessage>().is_some() {
            self.received.store(true, Ordering::SeqCst);
        }
        EventPropagation::Continue
    }
}

struct TestMessage;

#[test]
fn notify_bubbles_to_parent() {
    use textual_rs::event::dispatch::dispatch_message;

    let parent_received = Arc::new(AtomicBool::new(false));

    let mut ctx = AppContext::new();
    let parent_id = mount_widget(Box::new(FlagWidget::new(parent_received.clone())), None, &mut ctx);
    let child_id = mount_widget(Box::new(NoopWidget), Some(parent_id), &mut ctx);

    // child notifies with TestMessage — should bubble up to parent
    ctx.notify(child_id, TestMessage);

    // Drain message queue (simulate what App event loop does)
    let messages: Vec<_> = ctx.message_queue.borrow_mut().drain(..).collect();
    for (source, message) in messages {
        dispatch_message(source, message.as_ref(), &ctx);
    }

    assert!(
        parent_received.load(Ordering::SeqCst),
        "parent should receive TestMessage via notify() bubble"
    );
}

// ---- post_message dispatches to target ----

/// Verify ctx.post_message(target_id, msg) delivers to target widget.
struct CounterWidget {
    count: Arc<std::sync::atomic::AtomicU32>,
}

impl CounterWidget {
    fn new(count: Arc<std::sync::atomic::AtomicU32>) -> Self {
        CounterWidget { count }
    }
}

impl Widget for CounterWidget {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str { "CounterWidget" }

    fn on_event(&self, event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
        if event.downcast_ref::<TestMessage>().is_some() {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
        EventPropagation::Continue
    }
}

#[test]
fn post_message_to_target() {
    use textual_rs::event::dispatch::dispatch_message;

    let count_a = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let count_b = Arc::new(std::sync::atomic::AtomicU32::new(0));

    let mut ctx = AppContext::new();
    let _id_a = mount_widget(Box::new(CounterWidget::new(count_a.clone())), None, &mut ctx);
    let id_b = mount_widget(Box::new(CounterWidget::new(count_b.clone())), None, &mut ctx);

    // Post message targeting widget B
    ctx.post_message(id_b, TestMessage);

    // Drain message queue
    let messages: Vec<_> = ctx.message_queue.borrow_mut().drain(..).collect();
    for (source, message) in messages {
        dispatch_message(source, message.as_ref(), &ctx);
    }

    // Only widget B should have received the message
    assert_eq!(count_b.load(Ordering::SeqCst), 1, "widget B should receive message");
    assert_eq!(count_a.load(Ordering::SeqCst), 0, "widget A should not receive message");
}

// ---- cancel_workers only cancels for the given widget ----

/// Verify cancel_workers() only removes handles for the specified widget.
#[tokio::test]
async fn cancel_workers_targets_correct_widget() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let mut ctx = AppContext::new();
        let (worker_tx, _worker_rx) = flume::unbounded::<(WidgetId, Box<dyn Any + Send>)>();
        ctx.worker_tx = Some(worker_tx);

        let id_a = mount_widget(Box::new(NoopWidget), None, &mut ctx);
        let id_b = mount_widget(Box::new(NoopWidget), None, &mut ctx);

        // Spawn a worker for each widget
        let abort_a = ctx.run_worker(id_a, async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            0u32
        });
        let abort_b = ctx.run_worker(id_b, async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            0u32
        });

        // Cancel only widget A's workers
        ctx.cancel_workers(id_a);
        tokio::task::yield_now().await;

        assert!(abort_a.is_finished(), "widget A's worker should be aborted");
        assert!(!abort_b.is_finished(), "widget B's worker should still be running");
        assert!(!ctx.worker_handles.borrow().contains_key(id_a), "widget A's handles removed");
        assert!(ctx.worker_handles.borrow().contains_key(id_b), "widget B's handles remain");

        // Clean up: cancel B too
        ctx.cancel_workers(id_b);
    }).await;
}
