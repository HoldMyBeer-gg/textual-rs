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
