//! Write sweep: an "agent is writing here" pass. A soft band of light sweeps down the region the
//! agent is changing; each row the band crosses blurs out its old value and settles into the new
//! one, so the eye follows the edit instead of hunting for it. Afterwards the card lists the
//! number of changes with Undo, and each changed row keeps a small marker until it is seen.

/// Rows: (label, before, after). Equal values are left alone by the sweep.
pub const ROWS: [(&str, &str, &str); 6] = [
    ("Monday", "Rest", "Rest"),
    ("Tuesday", "Intervals 6 × 400 m", "Intervals 8 × 400 m"),
    ("Wednesday", "Easy 5 km", "Easy 6 km"),
    ("Thursday", "Tempo 20 min", "Tempo 25 min"),
    ("Saturday", "Long run 12 km", "Long run 14 km"),
    ("Sunday", "Recovery 4 km", "Recovery 4 km"),
];
/// The request the agent is answering.
pub const ASK: &str = "Build up to a half marathon by March";

/// Rows that change.
pub fn changed() -> Vec<usize> {
    (0..ROWS.len())
        .filter(|i| ROWS[*i].1 != ROWS[*i].2)
        .collect()
}

/// The sweep.
#[derive(Clone, Debug, PartialEq)]
pub struct Sweep {
    /// Started at.
    pub started: Option<u64>,
    /// Ms the band takes top to bottom.
    pub ms: u64,
    /// Undone after finishing.
    pub undone: bool,
}

impl Sweep {
    /// A sweep; `autoplay` starts it at `now`.
    pub fn new(ms: u64, autoplay: bool, now: u64) -> Self {
        Self {
            started: autoplay.then_some(now + 600),
            ms: ms.clamp(600, 6_000),
            undone: false,
        }
    }

    /// Band position 0..1 (top of the region to past the bottom), None when idle or finished.
    pub fn band(&self, now: u64) -> Option<f64> {
        let s = self.started?;
        if now < s {
            return None;
        }
        let p = (now - s) as f64 / self.ms as f64;
        (p < 1.0).then_some(p)
    }

    /// The band has crossed row `i` (of ROWS) at `now`.
    pub fn crossed(&self, i: usize, now: u64) -> bool {
        if self.undone {
            return false;
        }
        let Some(s) = self.started else { return false };
        if now < s {
            return false;
        }
        let p = (now - s) as f64 / self.ms as f64;
        p >= (i as f64 + 0.5) / ROWS.len() as f64
    }

    /// Ms since the band crossed row `i` (for the settle motion), if it has.
    pub fn since(&self, i: usize, now: u64) -> Option<u64> {
        let s = self.started?;
        let at = s + (self.ms as f64 * (i as f64 + 0.5) / ROWS.len() as f64) as u64;
        (self.crossed(i, now) && now >= at).then(|| now - at)
    }

    /// The value row `i` shows.
    pub fn value(&self, i: usize, now: u64) -> &'static str {
        if self.crossed(i, now) {
            ROWS[i].2
        } else {
            ROWS[i].1
        }
    }

    /// Finished (band gone and every row crossed).
    pub fn done(&self, now: u64) -> bool {
        !self.undone && self.started.is_some_and(|s| now >= s + self.ms)
    }

    /// Status line.
    pub fn status(&self, now: u64) -> String {
        if self.undone {
            return "Changes undone".into();
        }
        if self.band(now).is_some() {
            let n = changed().iter().filter(|i| self.crossed(**i, now)).count();
            return format!("Writing… {n} of {} changes", changed().len());
        }
        if self.done(now) {
            return format!("{} changes · tap a row to compare", changed().len());
        }
        "Ready to update the plan".into()
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ws-run" => {
                self.started = Some(now);
                self.undone = false;
                Some("Agent is updating the plan".into())
            }
            "ws-undo" if self.done(now) => {
                self.undone = true;
                Some("Changes undone".into())
            }
            "ws-redo" if self.undone => {
                self.undone = false;
                Some("Changes applied again".into())
            }
            _ => None,
        }
    }

    /// Enter runs, Cmd+Z undoes (passed as `undo`), Escape stops the band.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Enter" | "Space" if self.band(now).is_none() => self.press("ws-run", now),
            "undo" => self.press("ws-undo", now),
            "Escape" if self.band(now).is_some() => {
                self.started = Some(now.saturating_sub(self.ms));
                Some("Applied at once".into())
            }
            _ => None,
        }
    }
}
