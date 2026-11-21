//! Triage metro: security findings ride a metro map. Each finding enters on the Scan line, stops
//! at Triage, then branches to one of three terminals: Patched (green), False positive (grey) or
//! Needs you (amber). Counters at each terminal roll as dots arrive. Picking a dot opens its card;
//! a finding parked at Needs you can be patched or dismissed from there, and its dot rides on to
//! the new terminal. Geometry lives here so the Dioxus SVG and the GPUI canvas draw the same map.

use crate::aikit::{nth, rove};

/// Terminals: (name, hue).
pub const OUTCOMES: [(&str, &str); 3] = [
    ("Patched", "green"),
    ("False positive", "neutral"),
    ("Needs you", "amber"),
];

/// Sample findings: (title, file, severity, outcome index).
pub const FINDINGS: [(&str, &str, &str, usize); 8] = [
    ("SQL built from user input", "api/orders.ts:88", "High", 0),
    (
        "Token logged in debug output",
        "lib/auth.ts:41",
        "Medium",
        0,
    ),
    (
        "Open redirect on sign-out",
        "routes/session.ts:17",
        "Medium",
        2,
    ),
    ("Regex flagged as slow", "utils/slug.ts:9", "Low", 1),
    (
        "Missing rate limit on reset",
        "api/password.ts:23",
        "High",
        2,
    ),
    ("Outdated image library", "package.json:34", "Medium", 0),
    (
        "Test key in fixtures",
        "tests/fixtures/pay.json:2",
        "Low",
        1,
    ),
    (
        "Cookie without SameSite",
        "server/cookies.ts:12",
        "Medium",
        0,
    ),
];

/// Map size (view box).
pub const W: f64 = 600.0;
/// Map height.
pub const H: f64 = 200.0;
/// Stations on the trunk: (x, label).
pub const STATIONS: [(f64, &str); 2] = [(40.0, "Scan"), (230.0, "Triage")];
/// Terminal y for each outcome.
pub const TERM_Y: [f64; 3] = [44.0, 100.0, 156.0];
/// Terminal x.
pub const TERM_X: f64 = 560.0;

/// The polyline a dot rides to outcome `o`.
pub fn path(o: usize) -> [(f64, f64); 4] {
    let y = TERM_Y[o.min(2)];
    [
        (STATIONS[0].0, 100.0),
        (STATIONS[1].0, 100.0),
        (STATIONS[1].0 + 60.0, y),
        (TERM_X, y),
    ]
}

/// A point `t` (0..1) along a polyline, by length.
pub fn along(pts: &[(f64, f64)], t: f64) -> (f64, f64) {
    let segs: Vec<f64> = pts
        .windows(2)
        .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
        .collect();
    let total: f64 = segs.iter().sum();
    let mut d = t.clamp(0.0, 1.0) * total;
    for (i, s) in segs.iter().enumerate() {
        if d <= *s || i == segs.len() - 1 {
            let k = if *s > 0.0 { (d / s).min(1.0) } else { 1.0 };
            let (a, b) = (pts[i], pts[i + 1]);
            return (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
        }
        d -= s;
    }
    pts[pts.len() - 1]
}

/// Ms between findings entering.
pub const GAP: u64 = 900;
/// Ms a ride takes (pause at Triage included).
pub const RIDE: u64 = 3_200;
/// Ms parked at Triage.
pub const DWELL: u64 = 500;

/// The live map.
#[derive(Clone, Debug, PartialEq)]
pub struct Metro {
    /// Start.
    pub t0: u64,
    /// Paused at.
    pub paused: Option<u64>,
    /// Ms spent paused.
    pub paused_ms: u64,
    /// The open finding.
    pub open: Option<usize>,
    /// Rerouted findings: (index, new outcome, at).
    pub moved: Vec<(usize, usize, u64)>,
    /// Restart when every finding has landed.
    pub looping: bool,
}

impl Metro {
    /// A map from `now`.
    pub fn new(looping: bool, now: u64) -> Self {
        Self {
            t0: now,
            paused: None,
            paused_ms: 0,
            open: None,
            moved: Vec::new(),
            looping,
        }
    }

    /// Map time (ms), paused time removed, wrapped when looping.
    pub fn clock(&self, now: u64) -> u64 {
        let end = self.paused.unwrap_or(now);
        let e = end.saturating_sub(self.t0 + self.paused_ms);
        let cycle = GAP * FINDINGS.len() as u64 + RIDE + 2_500;
        if self.looping && self.moved.is_empty() && self.open.is_none() {
            e % cycle
        } else {
            e
        }
    }

    /// Where finding `i` is going.
    pub fn outcome(&self, i: usize) -> usize {
        self.moved
            .iter()
            .rev()
            .find(|m| m.0 == i)
            .map(|m| m.1)
            .unwrap_or(FINDINGS[i].3)
    }

    /// Ride progress 0..1 of finding `i`, or None before it enters.
    pub fn progress(&self, i: usize, now: u64) -> Option<f64> {
        let e = self.clock(now);
        let start = i as u64 * GAP;
        if e < start {
            return None;
        }
        let r = e - start;
        // trunk share ≈ 0.36 of the path; dwell at Triage.
        let trunk = 0.36;
        let half = (RIDE - DWELL) as f64 * trunk;
        let r = r as f64;
        let p = if r < half {
            r / half * trunk
        } else if r < half + DWELL as f64 {
            trunk
        } else {
            trunk
                + (r - half - DWELL as f64) / ((RIDE - DWELL) as f64 * (1.0 - trunk))
                    * (1.0 - trunk)
        };
        Some(p.min(1.0))
    }

    /// The point of finding `i`, if it is on the map. A rerouted finding jumps back to Triage
    /// and rides out to its new terminal.
    pub fn point(&self, i: usize, now: u64) -> Option<(f64, f64)> {
        let p = self.progress(i, now)?;
        if let Some(m) = self.moved.iter().rev().find(|m| m.0 == i) {
            let k = (now.saturating_sub(m.2) as f64 / 1_200.0).min(1.0);
            let from = path(FINDINGS[i].3)[3];
            let pts = [
                from,
                (TERM_X - 30.0, from.1),
                (TERM_X - 30.0, TERM_Y[m.1]),
                path(m.1)[3],
            ];
            return Some(along(&pts, k));
        }
        Some(along(&path(self.outcome(i)), p))
    }

    /// Landed counts per terminal.
    pub fn counts(&self, now: u64) -> [usize; 3] {
        let mut c = [0; 3];
        for i in 0..FINDINGS.len() {
            if self.landed(i, now) {
                c[self.outcome(i)] += 1;
            }
        }
        c
    }

    /// Finding `i` reached its terminal.
    pub fn landed(&self, i: usize, now: u64) -> bool {
        if let Some(m) = self.moved.iter().rev().find(|m| m.0 == i) {
            return now.saturating_sub(m.2) >= 1_200;
        }
        self.progress(i, now).is_some_and(|p| p >= 1.0)
    }

    /// Findings on the map so far.
    pub fn seen(&self, now: u64) -> usize {
        (0..FINDINGS.len())
            .filter(|i| self.progress(*i, now).is_some())
            .count()
    }

    /// Header line.
    pub fn status(&self, now: u64) -> String {
        let c = self.counts(now);
        let landed: usize = c.iter().sum();
        if landed == FINDINGS.len() {
            format!("{} findings triaged · {} need you", FINDINGS.len(), c[2])
        } else {
            format!("Triaging {} of {} findings", self.seen(now), FINDINGS.len())
        }
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "tm-pause" => {
                if let Some(p) = self.paused.take() {
                    self.paused_ms += now.saturating_sub(p);
                    Some("Resumed".into())
                } else {
                    self.paused = Some(now);
                    Some("Paused".into())
                }
            }
            "tm-close" => self.open.take().map(|_| "Closed".into()),
            "tm-patch" | "tm-dismiss" => {
                let i = self.open?;
                if self.outcome(i) != 2 || !self.landed(i, now) {
                    return None;
                }
                let to = if id == "tm-patch" { 0 } else { 1 };
                self.moved.push((i, to, now));
                Some(format!("{} moved to {}", FINDINGS[i].0, OUTCOMES[to].0))
            }
            "tm-restart" => {
                *self = Self::new(self.looping, now);
                Some("Started over".into())
            }
            _ => {
                let i = nth(id, "tm-dot-")?;
                if i >= FINDINGS.len() {
                    return None;
                }
                self.open = if self.open == Some(i) { None } else { Some(i) };
                Some(self.describe(i, now))
            }
        }
    }

    /// A finding spoken.
    pub fn describe(&self, i: usize, now: u64) -> String {
        let f = FINDINGS[i];
        let at = if self.landed(i, now) {
            OUTCOMES[self.outcome(i)].0
        } else {
            "on its way"
        };
        format!("{} ({}, {}): {at}", f.0, f.2, f.1)
    }

    /// Arrows move between findings, Escape closes the card, Space pauses.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Escape" => self.press("tm-close", now),
            "Space" => self.press("tm-pause", now),
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" => {
                let n = self.seen(now).max(1);
                let i = match self.open {
                    Some(i) => rove(i, n, k),
                    None => 0,
                };
                self.open = Some(i);
                Some(self.describe(i, now))
            }
            _ => None,
        }
    }
}
