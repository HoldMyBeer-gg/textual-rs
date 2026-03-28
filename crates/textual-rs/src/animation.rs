//! Lightweight tweening animation system.
//!
//! Provides a [`Tween`] struct for interpolating values over time with easing functions.
//! Used by widgets like Switch (knob slide) and Tabs (underline position) for smooth
//! visual transitions.

use std::time::{Duration, Instant};

/// An easing function maps normalized time [0.0, 1.0] to an eased value [0.0, 1.0].
pub type EasingFn = fn(f64) -> f64;

/// Cubic ease-in-out: slow start, fast middle, slow end. The default for UI transitions.
pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Linear interpolation: constant speed throughout.
pub fn linear(t: f64) -> f64 {
    t
}

/// Ease-out cubic: fast start, slow end. Good for "snap into place" animations.
pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

/// A value that transitions smoothly from one value to another over a duration.
///
/// Create with [`Tween::new`], then call [`Tween::value`] to get the current
/// interpolated value. Check [`Tween::is_complete`] to know when the animation
/// has finished.
///
/// # Example
/// ```
/// use std::time::Duration;
/// use textual_rs::animation::{Tween, linear};
///
/// let tween = Tween::new(0.0, 1.0, Duration::from_millis(300), linear);
/// // tween.value() returns a value between 0.0 and 1.0 based on elapsed time
/// ```
pub struct Tween {
    /// The starting value of the animation.
    pub from: f64,
    /// The ending value of the animation.
    pub to: f64,
    /// The total duration of the animation.
    pub duration: Duration,
    /// The easing function applied to normalize time.
    pub easing: EasingFn,
    /// The instant when the animation was created and started.
    pub start_time: Instant,
}

impl Tween {
    /// Create a new tween that interpolates from `from` to `to` over `duration`
    /// using the given `easing` function. Animation starts immediately.
    pub fn new(from: f64, to: f64, duration: Duration, easing: EasingFn) -> Self {
        Self {
            from,
            to,
            duration,
            easing,
            start_time: Instant::now(),
        }
    }

    /// Get the current interpolated value based on elapsed time since creation.
    pub fn value(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        if total <= 0.0 {
            return self.to;
        }
        let t = (elapsed / total).clamp(0.0, 1.0);
        let eased = (self.easing)(t);
        self.from + (self.to - self.from) * eased
    }

    /// Returns true if the animation has completed (elapsed >= duration).
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    /// Get the final target value of this tween.
    pub fn target(&self) -> f64 {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_easing_is_identity() {
        assert!((linear(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((linear(0.5) - 0.5).abs() < f64::EPSILON);
        assert!((linear(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_in_out_cubic_endpoints() {
        assert!((ease_in_out_cubic(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((ease_in_out_cubic(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_in_out_cubic_midpoint() {
        // At t=0.5, ease_in_out_cubic should return 0.5
        assert!((ease_in_out_cubic(0.5) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ease_out_cubic_endpoints() {
        assert!((ease_out_cubic(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tween_zero_duration_returns_target() {
        let tween = Tween::new(0.0, 100.0, Duration::ZERO, linear);
        assert!((tween.value() - 100.0).abs() < f64::EPSILON);
        assert!(tween.is_complete());
    }

    #[test]
    fn tween_immediately_after_creation() {
        let tween = Tween::new(0.0, 100.0, Duration::from_secs(10), linear);
        // Should be very close to 0.0 right after creation
        assert!(tween.value() < 1.0);
        assert!(!tween.is_complete());
    }

    #[test]
    fn tween_target_returns_to_value() {
        let tween = Tween::new(10.0, 50.0, Duration::from_millis(300), ease_in_out_cubic);
        assert!((tween.target() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tween_value_stays_in_range() {
        // Even with extreme durations, value should be clamped between from and to
        let tween = Tween::new(0.0, 1.0, Duration::from_nanos(1), linear);
        std::thread::sleep(Duration::from_millis(1));
        let v = tween.value();
        assert!(v >= 0.0 && v <= 1.0, "value {} out of range", v);
    }
}
