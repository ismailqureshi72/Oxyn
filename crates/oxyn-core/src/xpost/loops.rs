//! Loop pipeline: an agent that works in rounds until a check passes. Three stations (Work, Check,
//! Improve) sit on a loop; a marker travels round them, the round counter ticks, and each round
//! leaves a chip with its score. When a round scores at or above the target the loop exits to Done;
//! when it runs out of rounds it stops with the best score and offers one more round.

use crate::aikit::rove;

/// Stations.
pub const STATIONS: [&str; 3] = ["Work", "Check", "Improve"];
/// Scores per round (percent) in the sample.
pub const SCORES: [u32; 6] = [62, 74, 81, 88, 93, 96];
/// The task.
pub const TASK: (&str, &str) = ("Make the checkout page load under 1 s", "Lighthouse score");
/// Ms per station.
pub const STATION_MS: u64 = 700;

/// The loop.
#[derive(Clone, Debug, PartialEq)]
pub struct Loops {
    /// Start.
    pub t0: u64,
    /// Target score.
    pub target: u32,
    /// Most rounds.
    pub max: usize,
    /// Paused at.
    pub paused: Option<u64>,
    /// Ms paused.
    pub paused_ms: u64,
    /// Stopped by hand at.
    pub stopped: Option<u64>,
    /// Extra rounds granted.
    pub extra: usize,
    /// The focused round chip.
    pub sel: Option<usize>,
}

impl Loops {
    /// A loop from `now`.
    pub fn new(target: u32, max: usize, now: u64) -> Self {
        Self {
            t0: now,
            target: target.clamp(50, 99),
            max: max.clamp(1, SCORES.len()),
            paused: None,
            paused_ms: 0,
            stopped: None,
            extra: 0,
            sel: None,
        }
    }

    fn limit(&self) -> usize {
        (self.max + self.extra).min(SCORES.len())
    }

    fn clock(&self, now: u64) -> u64 {
        let end = self.stopped.or(self.paused).unwrap_or(now);
        end.saturating_sub(self.t0 + self.paused_ms)
    }

    /// The round that ends the loop (1-based), if any within the limit.
    pub fn exit_round(&self) -> Option<usize> {
        SCORES
            .iter()
            .take(self.limit())
            .position(|s| *s >= self.target)
            .map(|i| i + 1)
    }

    /// Rounds to run at most (the exit round, or the limit).
    pub fn rounds_planned(&self) -> usize {
        self.exit_round().unwrap_or(self.limit())
    }

    fn round_ms() -> u64 {
        STATION_MS * STATIONS.len() as u64
    }

    /// (round 0-based, station, fraction through it), or None when finished.
    pub fn at(&self, now: u64) -> Option<(usize, usize, f64)> {
        let e = self.clock(now);
        // The last round ends at Check (no Improve after the final score).
        let total = self.rounds_planned() as u64 * Self::round_ms() - STATION_MS;
        if e >= total {
            return None;
        }
        let r = (e / Self::round_ms()) as usize;
        let w = e % Self::round_ms();
        let s = (w / STATION_MS) as usize;
        Some((r, s, (w % STATION_MS) as f64 / STATION_MS as f64))
    }

    /// Scores of finished rounds.
    pub fn scored(&self, now: u64) -> Vec<u32> {
        let e = self.clock(now);
        (0..self.rounds_planned())
            .filter(|r| e >= *r as u64 * Self::round_ms() + 2 * STATION_MS)
            .map(|r| SCORES[r])
            .collect()
    }

    /// Passed the target.
    pub fn passed(&self, now: u64) -> bool {
        self.at(now).is_none() && self.exit_round().is_some() && self.stopped.is_none()
    }

    /// Ran out of rounds (or stopped).
    pub fn out(&self, now: u64) -> bool {
        self.stopped.is_some() || (self.at(now).is_none() && self.exit_round().is_none())
    }

    /// Best score so far.
    pub fn best(&self, now: u64) -> u32 {
        self.scored(now).into_iter().max().unwrap_or(0)
    }

    /// Status line.
    pub fn status(&self, now: u64) -> String {
        if self.passed(now) {
            let n = self.scored(now).len();
            return format!(
                "Done after {n} round{}: {}%",
                if n == 1 { "" } else { "s" },
                self.best(now)
            );
        }
        if self.out(now) {
            return format!(
                "Stopped after {} rounds · best {}% (target {}%)",
                self.scored(now).len(),
                self.best(now),
                self.target
            );
        }
        match self.at(now) {
            Some((r, s, _)) => format!(
                "Round {} of up to {} · {}{}",
                r + 1,
                self.limit(),
                STATIONS[s],
                if self.paused.is_some() {
                    " · paused"
                } else {
                    ""
                }
            ),
            None => String::new(),
        }
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "lp-pause" if !self.out(now) && !self.passed(now) => {
                if let Some(p) = self.paused.take() {
                    self.paused_ms += now.saturating_sub(p);
                    Some("Resumed".into())
                } else {
                    self.paused = Some(now);
                    Some("Paused".into())
                }
            }
            "lp-stop" if self.at(now).is_some() && self.stopped.is_none() => {
                self.stopped = Some(now);
                Some(self.status(now))
            }
            "lp-more" if self.out(now) && self.limit() < SCORES.len() && self.stopped.is_none() => {
                // Continue from the end of the last scored round.
                let resume =
                    (self.scored(now).len() as u64 * Self::round_ms()).saturating_sub(STATION_MS);
                self.extra += 1;
                self.paused = None;
                self.paused_ms = 0;
                self.t0 = now.saturating_sub(resume);
                Some("One more round".into())
            }
            "lp-restart" => {
                *self = Self::new(self.target, self.max, now);
                Some("Started over".into())
            }
            _ => {
                let i = crate::aikit::nth(id, "lp-round-")?;
                let s = self.scored(now);
                if i >= s.len() {
                    return None;
                }
                self.sel = Some(i);
                Some(format!("Round {}: {}%", i + 1, s[i]))
            }
        }
    }

    /// Space pauses, Escape stops, arrows move between round chips.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Space" => self.press("lp-pause", now),
            "Escape" => self.press("lp-stop", now),
            "ArrowLeft" | "ArrowRight" | "Home" | "End" => {
                let n = self.scored(now).len();
                if n == 0 {
                    return None;
                }
                let i = rove(self.sel.unwrap_or(n - 1), n, k);
                self.press(&format!("lp-round-{i}"), now)
            }
            _ => None,
        }
    }
}
