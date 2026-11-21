//! Done shockwave: when a long task finishes, a ring leaves its status mark and rolls across the
//! panel (two rings, the second softer and later), the mark turns into a check and a short line
//! says what finished and how long it took. A failure sends one short red pulse instead. With
//! reduced motion the rings are replaced by a single static halo.

/// The task: (title, detail, ms it takes at speed 1).
pub const TASK: (&str, &str, u64) = (
    "Deploy quillworks-web",
    "Build, upload 214 files, warm cache",
    3_600,
);
/// Steps shown under the progress bar.
pub const STEPS: [&str; 3] = ["Building", "Uploading 214 files", "Warming the cache"];
/// Ms the rings travel.
pub const WAVE_MS: u64 = 900;
/// Delay of the second ring.
pub const ECHO_MS: u64 = 160;

/// Where the task is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Not started.
    Idle,
    /// Running.
    Running,
    /// Finished.
    Done,
    /// Failed.
    Failed,
}

/// The task card.
#[derive(Clone, Debug, PartialEq)]
pub struct Shock {
    /// Started at.
    pub started: Option<u64>,
    /// Ms to finish.
    pub ms: u64,
    /// Ends in failure.
    pub fail: bool,
    /// Ring strength 0..1.
    pub strength: f64,
}

impl Shock {
    /// A card; `autoplay` starts at `now`.
    pub fn new(fail: bool, strength: f64, speed: f64, autoplay: bool, now: u64) -> Self {
        Self {
            started: autoplay.then_some(now + 400),
            ms: (TASK.2 as f64 / speed.clamp(0.25, 4.0)) as u64,
            fail,
            strength: strength.clamp(0.0, 1.0),
        }
    }

    /// The phase.
    pub fn phase(&self, now: u64) -> Phase {
        match self.started {
            None => Phase::Idle,
            Some(s) if now < s + self.ms => Phase::Running,
            Some(_) if self.fail => Phase::Failed,
            Some(_) => Phase::Done,
        }
    }

    /// Progress 0..1.
    pub fn progress(&self, now: u64) -> f64 {
        match self.started {
            None => 0.0,
            Some(s) => (now.saturating_sub(s) as f64 / self.ms as f64).min(if self.fail {
                0.72
            } else {
                1.0
            }),
        }
    }

    /// The step running now.
    pub fn step(&self, now: u64) -> usize {
        ((self.progress(now) * STEPS.len() as f64) as usize).min(STEPS.len() - 1)
    }

    /// Ms since the end, if ended.
    pub fn since_end(&self, now: u64) -> Option<u64> {
        let s = self.started?;
        (now >= s + self.ms).then(|| now - s - self.ms)
    }

    /// The rings at `now`: (radius 0..1 of the panel diagonal, opacity 0..1). Empty when still.
    pub fn rings(&self, now: u64) -> Vec<(f64, f64)> {
        let Some(e) = self.since_end(now) else {
            return vec![];
        };
        let mut out = vec![];
        let delays: &[u64] = if self.fail { &[0] } else { &[0, ECHO_MS] };
        let reach = if self.fail { 0.25 } else { 1.0 };
        for (k, d) in delays.iter().enumerate() {
            if e < *d {
                continue;
            }
            let t = (e - d) as f64 / WAVE_MS as f64;
            if t >= 1.0 {
                continue;
            }
            let ease = 1.0 - (1.0 - t).powi(3);
            let fade = (1.0 - t) * self.strength * if k == 0 { 1.0 } else { 0.55 };
            out.push((ease * reach, fade));
        }
        out
    }

    /// Elapsed text (`3.6s`).
    pub fn took(&self) -> String {
        format!("{:.1}s", self.ms as f64 / 1000.0)
    }

    /// Status line.
    pub fn status(&self, now: u64) -> String {
        match self.phase(now) {
            Phase::Idle => "Ready to deploy".into(),
            Phase::Running => format!("{}…", STEPS[self.step(now)]),
            Phase::Done => format!("Deployed in {}", self.took()),
            Phase::Failed => "Upload failed at file 154 of 214".into(),
        }
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "sw-run" if self.phase(now) != Phase::Running => {
                self.started = Some(now);
                Some("Deploy started".into())
            }
            "sw-retry" if self.phase(now) == Phase::Failed => {
                self.fail = false;
                self.started = Some(now);
                Some("Retrying the deploy".into())
            }
            "sw-reset" => {
                self.started = None;
                Some("Reset".into())
            }
            _ => None,
        }
    }

    /// Enter starts or retries.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Enter" | "Space" => {
                if self.phase(now) == Phase::Failed {
                    self.press("sw-retry", now)
                } else {
                    self.press("sw-run", now)
                }
            }
            "Escape" => self.press("sw-reset", now),
            _ => None,
        }
    }
}
