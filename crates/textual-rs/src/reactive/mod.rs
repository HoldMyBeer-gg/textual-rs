use reactive_graph::signal::ArcRwSignal;
use reactive_graph::computed::ArcMemo;
use reactive_graph::prelude::*;

/// Reactive property wrapper around `ArcRwSignal<T>`.
/// Triggers reactive graph tracking on reads, notifies dependents on writes.
pub struct Reactive<T: Clone + PartialEq + Send + Sync + 'static> {
    inner: ArcRwSignal<T>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Reactive<T> {
    pub fn new(value: T) -> Self {
        Self { inner: ArcRwSignal::new(value) }
    }

    /// Read the current value (tracked — creates dependency in Effects/Memos).
    pub fn get(&self) -> T {
        self.inner.get()
    }

    /// Read without creating a tracking dependency.
    pub fn get_untracked(&self) -> T {
        self.inner.get_untracked()
    }

    /// Set a new value, notifying all dependents.
    pub fn set(&self, value: T) {
        self.inner.set(value);
    }

    /// Update the value in-place via closure, notifying dependents.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(f);
    }

    /// Clone the inner ArcRwSignal for use in Effect/Memo closures.
    pub fn signal(&self) -> ArcRwSignal<T> {
        self.inner.clone()
    }
}

/// Computed reactive property wrapping `ArcMemo<T>` (per D-03 compute_ convention).
/// Derives its value from one or more reactive sources.
pub struct ComputedReactive<T: Clone + PartialEq + Send + Sync + 'static> {
    inner: ArcMemo<T>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> ComputedReactive<T> {
    pub fn new(f: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Self {
        Self { inner: ArcMemo::new(f) }
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }

    pub fn get_untracked(&self) -> T {
        self.inner.get_untracked()
    }
}

/// Helper to create a render-triggering Effect.
/// Call inside an async context with active Owner.
/// The Effect watches tracked reactive reads in its closure and posts RenderRequest
/// to the provided flume sender whenever any tracked reactive changes.
pub fn spawn_render_effect(tx: flume::Sender<crate::event::AppEvent>) {
    use reactive_graph::effect::Effect;
    use crate::event::AppEvent;
    Effect::new(move |_| {
        let _ = tx.try_send(AppEvent::RenderRequest);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    fn run_reactive_test<F: Future<Output = ()>>(f: impl FnOnce() -> F) {
        use any_spawner::Executor;
        use reactive_graph::owner::Owner;
        use tokio::runtime::Builder;
        use tokio::task::LocalSet;

        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let local = LocalSet::new();
        local.block_on(&rt, async {
            let _ = Executor::init_tokio();
            let _owner = Owner::new();
            f().await;
        });
    }

    #[test]
    fn reactive_new_get_returns_initial_value() {
        let r = Reactive::new(42i32);
        assert_eq!(r.get_untracked(), 42);
    }

    #[test]
    fn reactive_set_then_get_returns_new_value() {
        let r = Reactive::new(0i32);
        r.set(10);
        assert_eq!(r.get_untracked(), 10);
    }

    #[test]
    fn reactive_update_increments_value() {
        let r = Reactive::new(5i32);
        r.update(|v| *v += 1);
        assert_eq!(r.get_untracked(), 6);
    }

    #[test]
    fn reactive_get_untracked_reads_without_tracking() {
        let r = Reactive::new(99i32);
        // get_untracked should return the value without creating a dependency
        let val = r.get_untracked();
        assert_eq!(val, 99);
    }

    #[test]
    fn reactive_signal_returns_cloneable_arc_rw_signal() {
        let r = Reactive::new(7i32);
        let sig = r.signal();
        // Clone of signal should read the same value
        let sig2 = sig.clone();
        assert_eq!(sig.get_untracked(), 7);
        assert_eq!(sig2.get_untracked(), 7);
    }

    #[test]
    fn computed_reactive_derives_from_source_signal() {
        run_reactive_test(|| async {
            let r = Reactive::new(3i32);
            let sig = r.signal();
            let computed = ComputedReactive::new(move |_| sig.get() * 2);
            assert_eq!(computed.get_untracked(), 6);
            r.set(5);
            assert_eq!(computed.get_untracked(), 10);
        });
    }

    #[test]
    fn validate_convention_pattern() {
        // validate_ is a widget method convention, not a Reactive<T> method.
        // Widgets call validate_field_name() before calling field.set().
        // This test demonstrates the pattern:
        fn validate_count(value: &i32) -> i32 {
            (*value).clamp(0, 100)
        }
        let r = Reactive::new(50i32);
        let validated = validate_count(&150);
        r.set(validated);
        assert_eq!(r.get_untracked(), 100);
    }

    #[test]
    fn app_event_render_request_variant_can_be_sent_via_flume() {
        use crate::event::AppEvent;
        let (tx, rx) = flume::unbounded::<AppEvent>();
        tx.send(AppEvent::RenderRequest).unwrap();
        let received = rx.recv().unwrap();
        assert!(matches!(received, AppEvent::RenderRequest));
    }
}
