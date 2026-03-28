//! Async worker tasks: background computation with progress streaming and result delivery.

use crate::widget::WidgetId;

/// Result message delivered to a widget when its spawned worker completes.
///
/// Handle via `on_event` by downcasting:
/// ```ignore
/// if let Some(result) = event.downcast_ref::<WorkerResult<MyData>>() {
///     // use result.value
/// }
/// ```
///
/// `WorkerResult<T>` is dispatched via the message queue (`Box<dyn Any>`) path.
pub struct WorkerResult<T> {
    /// The WidgetId of the widget that spawned this worker.
    pub source_id: WidgetId,
    /// The computed value from the worker.
    pub value: T,
}

/// Progress message delivered to a widget from its spawned worker.
///
/// Use with `run_worker_with_progress` to receive incremental updates during
/// long-running tasks. Handle via `on_event` by downcasting:
/// ```ignore
/// if let Some(progress) = event.downcast_ref::<WorkerProgress<f32>>() {
///     // update progress bar with progress.progress
/// }
/// ```
pub struct WorkerProgress<T: Send + 'static> {
    /// The WidgetId of the widget that spawned this worker.
    pub source_id: WidgetId,
    /// The progress payload from the worker.
    pub progress: T,
}
