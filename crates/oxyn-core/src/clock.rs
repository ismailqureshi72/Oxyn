//! The shared motion clock (`docs/motion.md` §9 and §11.1). One per window; every loop, timer and
//! transition reads time from it so instances stay phase-locked.
//!
//! The host advances it with its own frame time and reports window state. The clock answers two
//! questions: *what phase is a loop at now* (computed from absolute time, so a paused loop resumes
//! where the clock says, never catching up) and *when should the next frame be*.

use std::time::Duration;

use oxyn_tokens::motion::fps;

use crate::ClockMs;

/// Window or system state, from the host's activation and visibility events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowState {
    /// Focused and visible.
    #[default]
    Focused,
    /// Visible but not focused.
    Unfocused,
    /// Visible, battery saver or low-power mode.
    LowPower,
    /// Hidden, minimised, occluded or scrolled offscreen.
    Hidden,
}

/// What kind of motion a lease drives; each has its own rate cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKind {
    /// One-shot tweens and springs (short; full rate unless hidden).
    Transition,
    /// Frame-set glyph spinners: their own frame time (e.g. `frame.glyph`, 80 ms).
    Glyph(Duration),
    /// Vector and shader loops.
    Vector,
    /// Particle loops.
    Particle,
    /// Text effects: shimmer, rolls (30 fps).
    TextEffect,
}

/// Particle loops drop further than vector loops when unfocused (`motion.fps.unfocused`:
/// "20 fps vector/shader, 15 fps particle").
const PARTICLE_UNFOCUSED_FPS: f32 = 15.0;

/// A handle to an active animation; release it when the animation settles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lease(u64);

/// The per-window clock.
#[derive(Clone, Debug, Default)]
pub struct MotionClock {
    now: ClockMs,
    window: WindowState,
    reduced: bool,
    leases: Vec<(Lease, MotionKind)>,
    next_id: u64,
}

impl MotionClock {
    /// A clock at time zero, focused, full motion.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current time.
    pub fn now(&self) -> ClockMs {
        self.now
    }

    /// Moves time forward. Earlier timestamps are ignored (the clock is monotonic).
    pub fn advance(&mut self, now: ClockMs) {
        if now > self.now {
            self.now = now;
        }
    }

    /// Reports window state.
    pub fn set_window(&mut self, window: WindowState) {
        self.window = window;
    }

    /// Current window state.
    pub fn window(&self) -> WindowState {
        self.window
    }

    /// Sets reduced motion (the OS setting or the app's override).
    pub fn set_reduced(&mut self, reduced: bool) {
        self.reduced = reduced;
    }

    /// Whether reduced motion is on. Presets return their reduced form when it is, so components
    /// never branch on it themselves.
    pub fn reduced(&self) -> bool {
        self.reduced
    }

    /// Starts an animation of `kind`; the clock keeps frames coming while any lease is held.
    pub fn lease(&mut self, kind: MotionKind) -> Lease {
        self.next_id += 1;
        let lease = Lease(self.next_id);
        self.leases.push((lease, kind));
        lease
    }

    /// Ends an animation.
    pub fn release(&mut self, lease: Lease) {
        self.leases.retain(|(l, _)| *l != lease);
    }

    /// Number of active leases.
    pub fn active(&self) -> usize {
        self.leases.len()
    }

    /// Maximum frame rate for `kind` in the current window state, or `None` when it must not run
    /// (hidden windows: loops pause, transitions jump to their end state).
    pub fn max_fps(&self, kind: MotionKind) -> Option<f32> {
        use MotionKind::*;
        use WindowState::*;
        if self.window == Hidden {
            return None;
        }
        Some(match (kind, self.window) {
            (Transition, _) => fps::MAX,
            (Glyph(frame), _) => 1000.0 / (frame.as_millis().max(1) as f32),
            (TextEffect, Focused | LowPower) => fps::TEXT_EFFECTS,
            (TextEffect, _) => fps::UNFOCUSED.min(fps::TEXT_EFFECTS),
            (Vector, Focused) => fps::VECTOR,
            (Particle, Focused) => fps::PARTICLE,
            (Vector, Unfocused) => fps::UNFOCUSED,
            (Particle, Unfocused) => PARTICLE_UNFOCUSED_FPS,
            (Vector | Particle, LowPower) => fps::LOW_POWER,
            (_, Hidden) => unreachable!(),
        })
    }

    /// How long until the next frame is needed: the fastest active lease, or the 1 Hz idle tick
    /// for timers (`motion.fps.idle`). `None` when the window is hidden.
    pub fn next_frame_in(&self) -> Option<Duration> {
        if self.window == WindowState::Hidden {
            return None;
        }
        let fastest = self
            .leases
            .iter()
            .filter_map(|(_, k)| self.max_fps(*k))
            .fold(fps::IDLE, f32::max);
        Some(Duration::from_micros(
            (1_000_000.0 / f64::from(fastest)).round() as u64,
        ))
    }

    /// Phase in `[0, 1)` of a loop with this period, from absolute clock time. Every instance of
    /// a loop reads the same phase, and a loop that was paused resumes at the current phase.
    pub fn phase(&self, period: Duration) -> f32 {
        let p = period.as_millis().max(1) as u64;
        (self.now.0 % p) as f32 / p as f32
    }

    /// Progress in `[0, 1]` of a transition that started at `start` and lasts `duration`.
    /// Hidden windows and reduced motion snap to the end.
    pub fn progress(&self, start: ClockMs, duration: Duration) -> f32 {
        if self.window == WindowState::Hidden || duration.is_zero() {
            return 1.0;
        }
        (self.now.since(start).as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxyn_tokens::motion::{dur::frame, loop_};

    #[test]
    fn idle_clock_ticks_at_one_hz() {
        let c = MotionClock::new();
        assert_eq!(c.next_frame_in(), Some(Duration::from_secs(1)));
    }

    #[test]
    fn rate_follows_the_fastest_lease_and_window_state() {
        let mut c = MotionClock::new();
        let glyph = c.lease(MotionKind::Glyph(frame::GLYPH));
        assert_eq!(c.next_frame_in(), Some(Duration::from_millis(80)));
        let v = c.lease(MotionKind::Vector);
        assert_eq!(c.max_fps(MotionKind::Vector), Some(60.0));
        c.set_window(WindowState::Unfocused);
        assert_eq!(c.max_fps(MotionKind::Vector), Some(20.0));
        assert_eq!(c.max_fps(MotionKind::Particle), Some(15.0));
        c.set_window(WindowState::LowPower);
        assert_eq!(c.max_fps(MotionKind::Particle), Some(30.0));
        c.set_window(WindowState::Hidden);
        assert_eq!(c.next_frame_in(), None);
        assert_eq!(c.max_fps(MotionKind::Transition), None);
        c.set_window(WindowState::Focused);
        c.release(v);
        c.release(glyph);
        assert_eq!(c.active(), 0);
    }

    #[test]
    fn phase_is_shared_and_never_catches_up() {
        let mut c = MotionClock::new();
        c.advance(ClockMs(1_500));
        assert!((c.phase(loop_::BASE) - 0.5).abs() < 1e-6);
        c.set_window(WindowState::Hidden);
        c.advance(ClockMs(10_250));
        c.set_window(WindowState::Focused);
        assert!((c.phase(loop_::BASE) - 0.25).abs() < 1e-6);
        c.advance(ClockMs(5)); // backwards: ignored
        assert_eq!(c.now(), ClockMs(10_250));
    }

    #[test]
    fn transitions_snap_when_hidden() {
        let mut c = MotionClock::new();
        c.advance(ClockMs(90));
        assert!((c.progress(ClockMs(0), Duration::from_millis(180)) - 0.5).abs() < 1e-6);
        c.set_window(WindowState::Hidden);
        assert_eq!(c.progress(ClockMs(0), Duration::from_millis(180)), 1.0);
    }
}
