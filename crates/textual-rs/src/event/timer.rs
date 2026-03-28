//! Timer utilities for scheduling periodic tick events in the application event loop.

use crate::event::AppEvent;
use std::time::Duration;

/// Spawn a periodic tick timer on the Tokio LocalSet.
/// Posts AppEvent::Tick to the event bus at the given interval.
/// Returns a JoinHandle that can be aborted to stop the timer.
pub fn spawn_tick_timer(
    tx: flume::Sender<AppEvent>,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_local(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            if tx.send(AppEvent::Tick).is_err() {
                break; // channel closed, app shutting down
            }
        }
    })
}
