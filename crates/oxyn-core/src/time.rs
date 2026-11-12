use std::time::Duration;

/// A monotonic timestamp in milliseconds, supplied by the host's clock.
///
/// Only differences matter; the origin is whatever the host uses (app start, page load).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockMs(pub u64);

impl ClockMs {
    /// Time from `earlier` to `self`, saturating at zero if the host clock went backwards.
    pub const fn since(self, earlier: ClockMs) -> Duration {
        Duration::from_millis(self.0.saturating_sub(earlier.0))
    }

    /// `self` moved forward by `d`.
    pub const fn plus(self, d: Duration) -> ClockMs {
        ClockMs(self.0 + d.as_millis() as u64)
    }
}
