//! Social proof toast: small "someone just joined" cards for marketing pages. A new card slides in
//! at the front on a timer while the older ones tuck behind; pointing at the pile or tabbing into
//! it freezes the timer (keeping its place) and spreads the cards; any card can be closed.

use super::{nth, thousands};

/// A sample event: who, what, context, minutes ago, mark (a real brand only when named), verified.
pub const EVENTS: [(&str, &str, &str, u64, &str, &str); 8] = [
    (
        "Sofia from Brightlane",
        "started a Pro trial",
        "Lisbon",
        0,
        "",
        "",
    ),
    (
        "Northpeak Labs",
        "upgraded to Team",
        "12 seats",
        4,
        "",
        "Verified by Stripe",
    ),
    (
        "Daniel K.",
        "bought the yearly plan",
        "Toronto",
        9,
        "",
        "Verified",
    ),
    (
        "Quillworks",
        "connected Slack",
        "8 channels",
        15,
        "slack",
        "",
    ),
    ("Amara O.", "started a Pro trial", "Lagos", 21, "", ""),
    (
        "Fernpoint",
        "moved 2,400 issues from Linear",
        "",
        33,
        "linear",
        "",
    ),
    (
        "Kenji T.",
        "upgraded to Pro",
        "Osaka",
        47,
        "",
        "Verified by Stripe",
    ),
    (
        "Harborline",
        "added 30 seats",
        "Team plan",
        64,
        "",
        "Verified",
    ),
];
/// Toast count shown in the header of the sample.
pub const TOTAL_THIS_WEEK: u64 = 1_284;

/// The live state of the stack.
#[derive(Clone, Debug, PartialEq)]
pub struct Proof {
    /// Start.
    pub t0: u64,
    /// Ms between toasts.
    pub interval: u64,
    /// Ms before the first.
    pub delay: u64,
    /// Toasts stacked at once.
    pub max: usize,
    /// Already shown on mount.
    pub prefill: usize,
    /// Start over at the end.
    pub looping: bool,
    /// Most toasts per session.
    pub limit: Option<usize>,
    /// Paused (hover or focus) at.
    pub paused: Option<u64>,
    /// Ms spent paused.
    pub paused_ms: u64,
    /// Dismissed toast sequence numbers.
    pub dismissed: Vec<usize>,
    /// Fan out while paused.
    pub fan: bool,
}

impl Proof {
    /// A stack from `now`.
    pub fn new(
        interval: u64,
        max: usize,
        prefill: usize,
        looping: bool,
        limit: Option<usize>,
        now: u64,
    ) -> Self {
        Self {
            t0: now,
            interval: interval.max(800),
            delay: 800,
            max: max.clamp(1, 5),
            prefill,
            looping,
            limit,
            paused: None,
            paused_ms: 0,
            dismissed: Vec::new(),
            fan: true,
        }
    }

    fn active(&self, now: u64) -> u64 {
        self.paused
            .unwrap_or(now)
            .saturating_sub(self.t0)
            .saturating_sub(self.paused_ms)
    }

    /// Toasts that have arrived.
    pub fn arrived(&self, now: u64) -> usize {
        let a = self.active(now);
        let n = if a < self.delay {
            0
        } else {
            ((a - self.delay) / self.interval) as usize + 1
        };
        let mut n = self.prefill + n;
        if !self.looping {
            n = n.min(EVENTS.len());
        }
        if let Some(l) = self.limit {
            n = n.min(l);
        }
        n
    }

    /// Ms the newest toast has been in (for its entrance).
    pub fn age(&self, now: u64) -> u64 {
        let a = self.active(now);
        if a < self.delay {
            return u64::MAX;
        }
        (a - self.delay) % self.interval
    }

    /// Visible toasts, front (newest) first, as sequence numbers.
    pub fn visible(&self, now: u64) -> Vec<usize> {
        (0..self.arrived(now))
            .rev()
            .filter(|s| !self.dismissed.contains(s))
            .take(self.max)
            .collect()
    }

    /// The event of a sequence number.
    pub fn event(
        seq: usize,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        u64,
        &'static str,
        &'static str,
    ) {
        EVENTS[seq % EVENTS.len()]
    }

    /// Time label (`Just now`, `4 min ago`, `1 h ago`).
    pub fn when(&self, seq: usize, now: u64) -> String {
        let front = self.arrived(now).saturating_sub(1);
        let m = Self::event(seq).3.min(5) + (front - seq.min(front)) as u64 * 3;
        if seq == front || m == 0 {
            "Just now".into()
        } else if m < 60 {
            format!("{m} min ago")
        } else {
            format!("{} h ago", m / 60)
        }
    }

    /// The accessible line of a toast.
    pub fn line(seq: usize) -> String {
        let e = Self::event(seq);
        format!("{} {}", e.0, e.1)
    }

    /// Ms until the next toast (kept across pauses).
    pub fn next_in(&self, now: u64) -> u64 {
        self.interval - self.age(now).min(self.interval)
    }

    /// Presses: `sp-pause` (hover or focus in), `sp-resume`, `sp-dismiss-<seq>`,
    /// `sp-dismiss-front`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(s) = nth(id, "sp-dismiss-") {
            if !self.dismissed.contains(&s) {
                self.dismissed.push(s);
                return Some(format!("Dismissed: {}", Self::line(s)));
            }
            return None;
        }
        match id {
            "sp-dismiss-front" => {
                let f = *self.visible(now).first()?;
                self.press(&format!("sp-dismiss-{f}"), now)
            }
            "sp-pause" if self.paused.is_none() => {
                self.paused = Some(now);
                None
            }
            "sp-resume" => {
                let p = self.paused.take()?;
                self.paused_ms += now.saturating_sub(p);
                None
            }
            _ => None,
        }
    }

    /// The header count (`1,284 this week`).
    pub fn week(&self) -> String {
        format!("{} people joined this week", thousands(TOTAL_THIS_WEEK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_cycles_pauses_and_dismisses() {
        let mut p = Proof::new(4_000, 3, 0, true, None, 0);
        assert!(p.visible(500).is_empty());
        assert_eq!(p.visible(900), vec![0]);
        assert_eq!(p.visible(9_000), vec![2, 1, 0]);
        assert_eq!(p.visible(13_000), vec![3, 2, 1]);
        p.press("sp-pause", 13_000);
        assert_eq!(p.arrived(60_000), 4);
        let left = p.next_in(13_000);
        p.press("sp-resume", 60_000);
        assert_eq!(p.next_in(60_000), left);
        p.press("sp-dismiss-front", 60_000);
        assert_eq!(p.visible(60_000), vec![2, 1, 0]);
        assert_eq!(p.when(3, 60_000), "Just now");
        let l = Proof::new(1_000, 3, 2, false, Some(3), 0);
        assert_eq!(l.arrived(0), 2);
        assert_eq!(l.arrived(99_000), 3);
    }
}
