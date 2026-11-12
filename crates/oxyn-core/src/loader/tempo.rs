//! A beat clock whose speed can change without the drawing jumping (`loaders.md` §2.1).

use std::time::Duration;

use crate::ClockMs;

/// Integrates `speed × dt` so a tempo change eases in and never skips frames. With speed 1 the
/// beat equals clock milliseconds, so loaders that never change tempo stay in phase with every
/// other instance of their variant (phase origin is clock zero, not mount time).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoClock {
    anchor: ClockMs,
    anchor_beats: f64,
    from: f32,
    to: f32,
    ease_ms: u64,
}

impl TempoClock {
    /// Running at `speed` from clock zero.
    pub fn new(speed: f32) -> Self {
        Self {
            anchor: ClockMs(0),
            anchor_beats: 0.0,
            from: speed,
            to: speed,
            ease_ms: 0,
        }
    }

    /// Ease to `speed` over `over`, starting at `now`. Smoothstep on speed, integrated exactly.
    pub fn set_speed(&mut self, speed: f32, now: ClockMs, over: Duration) {
        if (speed - self.to).abs() < f32::EPSILON && self.settled(now) {
            return;
        }
        let beats = self.beats(now);
        self.from = self.speed(now);
        self.to = speed;
        self.anchor = now;
        self.anchor_beats = beats;
        self.ease_ms = over.as_millis() as u64;
    }

    /// The speed right now.
    pub fn speed(&self, now: ClockMs) -> f32 {
        let x = self.x(now);
        let s = x * x * (3.0 - 2.0 * x);
        self.from + (self.to - self.from) * s as f32
    }

    /// Beats (milliseconds at speed 1) elapsed since clock zero.
    pub fn beats(&self, now: ClockMs) -> f64 {
        let dt = now.0.saturating_sub(self.anchor.0) as f64;
        let (from, to) = (f64::from(self.from), f64::from(self.to));
        if self.ease_ms == 0 || dt >= self.ease_ms as f64 {
            let ramp = self.ease_ms.min(now.0.saturating_sub(self.anchor.0)) as f64;
            // ∫₀¹ smoothstep = 1/2, so the ramp contributes its average speed.
            return self.anchor_beats + ramp * (from + to) / 2.0 + (dt - ramp) * to;
        }
        let e = self.ease_ms as f64;
        let x = dt / e;
        // ∫₀ˣ (3s² − 2s³) ds = x³ − x⁴/2
        let integral = x.powi(3) - x.powi(4) / 2.0;
        self.anchor_beats + e * (from * x + (to - from) * integral)
    }

    /// True once the last change has finished easing.
    pub fn settled(&self, now: ClockMs) -> bool {
        now.0.saturating_sub(self.anchor.0) >= self.ease_ms
    }

    fn x(&self, now: ClockMs) -> f64 {
        if self.ease_ms == 0 {
            return 1.0;
        }
        (now.0.saturating_sub(self.anchor.0) as f64 / self.ease_ms as f64).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_speed_tracks_the_clock() {
        let t = TempoClock::new(1.0);
        assert_eq!(t.beats(ClockMs(12_345)), 12_345.0);
    }

    #[test]
    fn speed_change_is_continuous() {
        let mut t = TempoClock::new(1.0);
        t.set_speed(2.0, ClockMs(1000), Duration::from_millis(400));
        // No jump at the change.
        assert!((t.beats(ClockMs(1000)) - 1000.0).abs() < 1e-9);
        // Average speed over the ramp is 1.5.
        assert!((t.beats(ClockMs(1400)) - 1600.0).abs() < 1e-9);
        assert!((t.beats(ClockMs(1500)) - 1800.0).abs() < 1e-9);
        // Monotonic and smooth around the ramp.
        let mut last = 0.0;
        for ms in (1000..1600).step_by(10) {
            let b = t.beats(ClockMs(ms));
            assert!(b > last);
            last = b;
        }
        assert!((t.speed(ClockMs(1200)) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn changing_mid_ramp_keeps_continuity() {
        let mut t = TempoClock::new(1.0);
        t.set_speed(0.5, ClockMs(0), Duration::from_millis(2000));
        let before = t.beats(ClockMs(1000));
        t.set_speed(1.25, ClockMs(1000), Duration::from_millis(200));
        assert!((t.beats(ClockMs(1000)) - before).abs() < 1e-9);
    }
}
