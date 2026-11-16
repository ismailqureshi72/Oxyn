//! affectUI fidelity pass, family B part 9 (plan #32): the **Fan-out Grid** model. One job split
//! into many small tasks that a fixed pool of workers takes in turn. Each task has a start and an
//! end on the job clock; a few fail once and are retried by the worker, a few fail for good until
//! the user retries them. At any moment the grid reads each task's status, the counts, the cost
//! so far and an estimate of the time left. Behaviour only: the views live in `oxyn-dioxus` /
//! `oxyn-gpui` (`parity/fanfid.rs`).

/// A task's status at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FoStatus {
    /// Waiting for a worker.
    Queued,
    /// A worker has it.
    Running,
    /// Its first try failed; the worker is trying again.
    Retrying,
    /// Finished.
    Done,
    /// Failed for good (until retried by hand).
    Failed,
}

impl FoStatus {
    /// `queued` / `running` / `retrying` / `done` / `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Retrying => "retrying",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
    /// "Done" etc.
    pub fn word(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Retrying => "Retrying",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
    /// The four counted states, in display order (queued is not counted).
    pub const COUNTED: [FoStatus; 4] = [Self::Done, Self::Running, Self::Retrying, Self::Failed];
}

/// One task.
#[derive(Clone, Debug, PartialEq)]
pub struct FoTask {
    /// Stable id.
    pub id: String,
    /// What it works on ("larkfield.io").
    pub label: String,
    /// Worker number (1-based).
    pub worker: u32,
    /// Start, ms on the job clock.
    pub start: u32,
    /// End, ms.
    pub end: u32,
    /// Cost, dollars.
    pub cost: f64,
    /// The first try fails at this ms and the worker retries until `end`.
    pub retry_at: Option<u32>,
    /// Ends failed (until retried by hand).
    pub fails: bool,
    /// Result line ("Seed · 14 people · founder email found") or the error.
    pub result: String,
}

impl FoTask {
    /// Status at `now`; `retried` is when the user pressed Retry (a failed task then runs again
    /// for its own duration and ends done).
    pub fn status(&self, now: u32, retried: Option<u32>) -> FoStatus {
        if let (true, Some(r)) = (self.fails, retried)
            && r >= self.end
        {
            return if now >= r + (self.end - self.start) {
                FoStatus::Done
            } else {
                FoStatus::Running
            };
        }
        if now < self.start {
            FoStatus::Queued
        } else if now < self.end {
            match self.retry_at {
                Some(r) if now >= r => FoStatus::Retrying,
                _ => FoStatus::Running,
            }
        } else if self.fails {
            FoStatus::Failed
        } else {
            FoStatus::Done
        }
    }
    /// Tries made by `now` (2 once a retry started).
    pub fn attempts(&self, now: u32, retried: Option<u32>) -> u32 {
        let base = if self.retry_at.is_some_and(|r| now >= r) {
            2
        } else {
            1
        };
        base + u32::from(self.fails && retried.is_some_and(|r| now >= r))
    }
    /// How long it ran (or has run) by `now`, ms.
    pub fn took(&self, now: u32, retried: Option<u32>) -> u32 {
        match (self.fails, retried) {
            (true, Some(r)) if now >= r => (now - r).min(self.end - self.start),
            _ => now.clamp(self.start, self.end) - self.start,
        }
    }
}

/// The grid at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct FoView {
    /// Status per task.
    pub status: Vec<FoStatus>,
    /// Count per counted status (done, running, retrying, failed).
    pub counts: [usize; 4],
    /// Dollars spent (done tasks).
    pub cost: f64,
    /// Estimated ms left (0 when finished).
    pub left: u32,
    /// Nothing queued, running or retrying.
    pub finished: bool,
    /// When the last task ended, ms.
    pub ended_at: u32,
}

/// Read the job at `now`. `retried` is when the user retried the failed tasks.
pub fn fo_view(tasks: &[FoTask], now: u32, retried: Option<u32>, workers: u32) -> FoView {
    let status: Vec<FoStatus> = tasks.iter().map(|t| t.status(now, retried)).collect();
    let mut counts = [0usize; 4];
    for s in &status {
        if let Some(i) = FoStatus::COUNTED.iter().position(|c| c == s) {
            counts[i] += 1;
        }
    }
    let cost = tasks
        .iter()
        .zip(&status)
        .filter(|(_, s)| **s == FoStatus::Done)
        .map(|(t, _)| t.cost)
        .sum();
    let finished = status
        .iter()
        .all(|s| matches!(s, FoStatus::Done | FoStatus::Failed));
    let end_of = |t: &FoTask| match (t.fails, retried) {
        (true, Some(r)) if r >= t.end => r + (t.end - t.start),
        _ => t.end,
    };
    let ended_at = tasks.iter().map(end_of).max().unwrap_or(0);
    let left = if finished {
        0
    } else {
        // Remaining work spread over the workers.
        let rem: u32 = tasks
            .iter()
            .map(|t| end_of(t).saturating_sub(now.max(t.start)))
            .sum();
        rem / workers.max(1)
    };
    FoView {
        status,
        counts,
        cost,
        left,
        finished,
        ended_at,
    }
}

/// Seconds as "16s" / "1m 05s".
pub fn fo_secs(ms: u32) -> String {
    let s = ms.div_ceil(1000);
    if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

/// A small deterministic generator for the demo.
fn lcg(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    (*seed >> 16) & 0x7fff
}

/// The demo job: enrich `n` made-up company domains with `workers` workers; three fail once and
/// are retried, two fail for good.
pub fn fo_demo(n: usize, workers: u32) -> Vec<FoTask> {
    const A: [&str; 16] = [
        "amber", "birch", "cobalt", "dune", "ember", "fennel", "gale", "harbor", "iris", "juniper",
        "kestrel", "lark", "moss", "nimbus", "orchid", "pebble",
    ];
    const B: [&str; 8] = [
        "works", "field", "loop", "labs", "craft", "stack", "yard", "wave",
    ];
    const T: [&str; 3] = [".co", ".io", ".app"];
    const STAGE: [&str; 4] = ["Seed", "Series A", "Series B", "Bootstrapped"];
    let mut seed = 7u32;
    let mut free = vec![0u32; workers.max(1) as usize];
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let w = (0..free.len()).min_by_key(|&k| (free[k], k)).unwrap_or(0);
        let dur = 800 + lcg(&mut seed) % 900;
        let start = free[w] + 40;
        let retry = matches!(i % 83, 41).then_some(start + dur / 2);
        let dur = if retry.is_some() { dur + 700 } else { dur };
        let end = start + dur;
        free[w] = end;
        let fails = matches!(i, 57 | 173) && n > 173;
        let a = A[i % 16];
        let b = B[(i / 16 + i) % 8];
        let people = 4 + lcg(&mut seed) % 40;
        let stage = STAGE[(lcg(&mut seed) % 4) as usize];
        let result = if fails {
            "The site timed out three times".to_string()
        } else if lcg(&mut seed).is_multiple_of(3) {
            format!("{stage} · {people} people · no contact found")
        } else {
            format!("{stage} · {people} people · founder email found")
        };
        out.push(FoTask {
            id: format!("t{i}"),
            label: format!("{a}{b}{}", T[i % 3]),
            worker: w as u32 + 1,
            start,
            end,
            cost: 0.008 + (lcg(&mut seed) % 14) as f64 / 1000.0,
            retry_at: retry,
            fails,
            result,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_and_retry() {
        let t = fo_demo(240, 12);
        assert_eq!(t.len(), 240);
        let v0 = fo_view(&t, 0, None, 12);
        assert_eq!(v0.counts, [0, 0, 0, 0]);
        assert!(!v0.finished);
        let mid = fo_view(&t, 8_000, None, 12);
        assert!(mid.counts[0] > 40 && mid.counts[1] > 0, "{:?}", mid.counts);
        assert!(mid.left > 0);
        let end = t.iter().map(|x| x.end).max().unwrap();
        assert!((20_000..32_000).contains(&end), "{end}");
        let fin = fo_view(&t, end, None, 12);
        assert!(fin.finished);
        assert_eq!(fin.counts[3], 2);
        assert_eq!(fin.counts[0], 238);
        // Retry the failures: they run again and end done.
        let r = fo_view(&t, end + 100, Some(end), 12);
        assert_eq!(r.counts[3], 0);
        assert_eq!(r.counts[1], 2);
        let r = fo_view(&t, end + 5_000, Some(end), 12);
        assert!(r.finished);
        assert_eq!(r.counts[0], 240);
        assert!(t.iter().any(|x| x.retry_at.is_some()));
        let k = t.iter().find(|x| x.retry_at.is_some()).unwrap();
        assert_eq!(k.status(k.retry_at.unwrap(), None), FoStatus::Retrying);
        assert_eq!(k.attempts(k.end, None), 2);
        assert_eq!(fo_secs(16_000), "16s");
        assert_eq!(fo_secs(65_000), "1m 05s");
    }
}
