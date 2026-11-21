//! Test ticks: a test run drawn as one small tick per test, grouped by file. Ticks fill in the
//! order the runner reports them (grey while queued, outlined while running, green when passed,
//! red when failed). The timer, the passed and failed counters and the tests-per-second rate roll
//! with the run. When a run ends with failures, the failing rows stay lit and the rest dim, and a
//! fix card opens over the first failing file with a short diff; applying it re-runs only the
//! failed tests.

use crate::aikit::{nth, rove};

/// Test files and how many tests each holds.
pub const FILES: [(&str, usize); 4] = [
    ("cart.test.ts", 14),
    ("pricing.test.ts", 12),
    ("checkout.test.ts", 16),
    ("tax.test.ts", 6),
];
/// The run's label.
pub const SUITE: (&str, &str) = ("Checkout agent", "PR #214 · pnpm test");
/// Tests that fail in the failing scenario (global indices).
pub const FAILS: [usize; 2] = [19, 23];
/// The diff the fix card proposes: (sign, line).
pub const FIX: [(char, &str); 4] = [
    (
        ' ',
        "export function applyDiscount(total: number, pct: number) {",
    ),
    ('-', "  return total - total * pct;"),
    (
        '+',
        "  return Math.round(total * (1 - pct / 100) * 100) / 100;",
    ),
    (' ', "}"),
];

/// What one tick shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// Not started.
    Queued,
    /// Running now.
    Running,
    /// Passed.
    Pass,
    /// Failed.
    Fail,
}

impl Tick {
    /// Lowercase name (CSS data attribute).
    pub fn name(self) -> &'static str {
        match self {
            Tick::Queued => "queued",
            Tick::Running => "running",
            Tick::Pass => "pass",
            Tick::Fail => "fail",
        }
    }
}

/// Total tests.
pub fn total() -> usize {
    FILES.iter().map(|f| f.1).sum()
}

/// The file a test index belongs to and its index inside that file.
pub fn file_of(i: usize) -> (usize, usize) {
    let mut a = 0;
    for (f, (_, n)) in FILES.iter().enumerate() {
        if i < a + n {
            return (f, i - a);
        }
        a += n;
    }
    (FILES.len() - 1, 0)
}

/// The test's display name.
pub fn test_name(i: usize) -> String {
    const VERBS: [&str; 8] = [
        "adds an item",
        "removes an item",
        "keeps the quantity",
        "applies a discount",
        "rounds to cents",
        "rejects an empty cart",
        "charges shipping",
        "restores a saved cart",
    ];
    let (f, k) = file_of(i);
    let stem = FILES[f].0.split('.').next().unwrap_or("");
    format!("{stem} › {}", VERBS[(i * 5 + f) % VERBS.len()])
        + if k > 7 { " (edge case)" } else { "" }
}

/// Ms a test takes at speed 1.
pub fn dur(i: usize) -> u64 {
    90 + ((i as u64 * 37) % 150)
}

/// A test run.
#[derive(Clone, Debug, PartialEq)]
pub struct TestTicks {
    /// Run start.
    pub t0: u64,
    /// Speed factor (1 = sample timing).
    pub speed: f64,
    /// The failing scenario.
    pub fail: bool,
    /// The fix was applied at (re-run of the failed tests).
    pub fixed: Option<u64>,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// The focused tick.
    pub sel: Option<usize>,
    /// Only failed rows.
    pub only_failed: bool,
    /// The fix card was closed.
    pub fix_closed: bool,
}

impl TestTicks {
    /// A run starting at `now`.
    pub fn new(fail: bool, speed: f64, now: u64) -> Self {
        Self {
            t0: now,
            speed: speed.clamp(0.25, 4.0),
            fail,
            fixed: None,
            stopped: None,
            sel: None,
            only_failed: false,
            fix_closed: false,
        }
    }

    fn scaled(&self, ms: u64) -> u64 {
        (ms as f64 / self.speed) as u64
    }

    /// When test `i` ends (ms after start).
    pub fn ends(&self, i: usize) -> u64 {
        self.scaled((0..=i).map(dur).sum())
    }

    /// Ms the first pass takes.
    pub fn span(&self) -> u64 {
        self.ends(total() - 1)
    }

    fn clock(&self, now: u64) -> u64 {
        self.stopped.unwrap_or(now).saturating_sub(self.t0)
    }

    /// The state of tick `i`.
    pub fn tick(&self, i: usize, now: u64) -> Tick {
        let e = self.clock(now);
        let end = self.ends(i);
        let start = end.saturating_sub(self.scaled(dur(i)));
        let failing = self.fail && FAILS.contains(&i);
        if e >= end {
            if failing {
                if let Some(f) = self.fixed {
                    let k = FAILS.iter().position(|x| *x == i).unwrap_or(0) as u64;
                    let fe = now.saturating_sub(f);
                    let at = 500 + k * 450;
                    return if fe >= at {
                        Tick::Pass
                    } else if fe + 450 >= at {
                        Tick::Running
                    } else {
                        Tick::Fail
                    };
                }
                return Tick::Fail;
            }
            Tick::Pass
        } else if e >= start && self.stopped.is_none() {
            Tick::Running
        } else {
            Tick::Queued
        }
    }

    /// (passed, failed, finished) counts.
    pub fn counts(&self, now: u64) -> (usize, usize, usize) {
        let (mut p, mut f) = (0, 0);
        for i in 0..total() {
            match self.tick(i, now) {
                Tick::Pass => p += 1,
                Tick::Fail => f += 1,
                _ => {}
            }
        }
        (p, f, p + f)
    }

    /// The run finished (including a re-run of the fixed tests).
    pub fn done(&self, now: u64) -> bool {
        let (_, _, n) = self.counts(now);
        n == total() && !(0..total()).any(|i| self.tick(i, now) == Tick::Running)
    }

    /// Elapsed ms shown in the timer (frozen when done or stopped).
    pub fn elapsed(&self, now: u64) -> u64 {
        let base = self.clock(now).min(self.span());
        match self.fixed {
            Some(f) => base + now.saturating_sub(f).min(500 + FAILS.len() as u64 * 450),
            None => base,
        }
    }

    /// Tests per second so far.
    pub fn rate(&self, now: u64) -> f64 {
        let (_, _, n) = self.counts(now);
        let e = self.elapsed(now).max(1) as f64 / 1000.0;
        n as f64 / e
    }

    /// The fix card is showing.
    pub fn fix_open(&self, now: u64) -> bool {
        self.fail && self.fixed.is_none() && !self.fix_closed && self.done(now)
    }

    /// Status line.
    pub fn status(&self, now: u64) -> String {
        let (p, f, n) = self.counts(now);
        if self.stopped.is_some() && n < total() {
            return format!("Stopped after {n} of {} tests", total());
        }
        if !self.done(now) {
            if self.fixed.is_some() {
                return format!("Re-running {} failed tests", FAILS.len());
            }
            return format!("Running {n} of {}", total());
        }
        if f > 0 {
            format!("{f} failed, {p} passed")
        } else {
            format!("All {p} tests passed")
        }
    }

    /// Handles a press id. Returns what to announce.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "tt-run" => {
                *self = Self::new(self.fail, self.speed, now);
                Some("Running the suite again".into())
            }
            "tt-stop" => {
                if !self.done(now) && self.stopped.is_none() {
                    self.stopped = Some(now);
                    return Some(self.status(now));
                }
                None
            }
            "tt-filter" => {
                self.only_failed = !self.only_failed;
                Some(if self.only_failed {
                    "Showing failed tests only".into()
                } else {
                    "Showing every test".into()
                })
            }
            "tt-fix" => {
                if self.fix_open(now) {
                    self.fixed = Some(now);
                    return Some("Fix applied, re-running the failed tests".into());
                }
                None
            }
            "tt-fix-close" => {
                self.fix_closed = true;
                Some("Fix card closed".into())
            }
            _ => {
                let i = nth(id, "tt-tick-")?;
                self.sel = if self.sel == Some(i) { None } else { Some(i) };
                Some(self.describe(i, now))
            }
        }
    }

    /// A tick spoken.
    pub fn describe(&self, i: usize, now: u64) -> String {
        format!(
            "{}: {} ({} ms)",
            test_name(i),
            self.tick(i, now).name(),
            self.scaled(dur(i))
        )
    }

    /// Keys on the tick grid: arrows move the focused tick, Escape clears it, Enter applies the
    /// fix while the card is open.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Escape" => {
                if self.fix_open(now) {
                    self.fix_closed = true;
                    return Some("Fix card closed".into());
                }
                self.sel.take().map(|_| "Selection cleared".into())
            }
            "Enter" if self.fix_open(now) => self.press("tt-fix", now),
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" => {
                let i = rove(self.sel.unwrap_or(0), total(), k);
                let i = if self.sel.is_none() { 0 } else { i };
                self.sel = Some(i);
                Some(self.describe(i, now))
            }
            _ => None,
        }
    }
}
