//! Training run: a fine-tuning job card. A train and validation loss curve draws across a fixed
//! step axis that doubles as the epoch progress bar; the card shows the learning rate, elapsed
//! time, ETA and the newest events, has Pause and a two-press Cancel while running, a Retry from
//! checkpoint after a failure, and ends in results with Deploy.

use super::{hash, norm_key};

/// Steps in the run.
pub const TOTAL: u64 = 1_200;
/// Epochs.
pub const EPOCHS: u64 = 3;
/// Queue time, ms.
pub const QUEUE_MS: u64 = 1_400;
/// Starting up (no samples yet), ms.
pub const START_MS: u64 = 900;
/// Ms per step at speed 1.
pub const STEP_MS: u64 = 18;
/// Simulated seconds per step (for elapsed and ETA).
pub const SIM_SECS: f64 = 2.3;
/// The OOM scenario fails here.
pub const OOM_STEP: u64 = 940;
/// A checkpoint is saved every this many steps.
pub const CKPT_EVERY: u64 = 400;
/// Cancel waits this long for its confirm press, ms.
pub const CONFIRM_MS: u64 = 3_000;
/// Deploy takes this long, ms.
pub const DEPLOY_MS: u64 = 1_400;
/// Samples every this many steps.
pub const SAMPLE_EVERY: u64 = 10;

/// The base model, its mark and short facts.
pub const MODEL: (&str, &str, &str) = ("Qwen3 14B", "qwen", "LoRA r16 · 8× H100");
/// The job name.
pub const JOB: &str = "support-replies-v4";
/// The dataset and its mark.
pub const DATASET: (&str, &str, u32) = ("halcyon/support-replies", "huggingface", 48_210);
/// The deployed endpoint.
pub const ENDPOINT: &str = "ft:qwen3-14b:halcyon:support-v4";

/// Train loss at a step (raw, with noise).
pub fn train_loss(s: u64) -> f64 {
    let x = s as f64;
    let n = (hash(&format!("t{s}")) % 1000) as f64 / 1000.0 - 0.5;
    0.36 + 2.05 * (-x / 250.0).exp() + 0.4 * (-x / 30.0).exp() + n * 0.09 * (1.0 - x / 1600.0)
}

/// Train loss smoothed over the last 8 samples.
pub fn train_smooth(s: u64) -> f64 {
    let k = 8u64;
    let first = s.saturating_sub((k - 1) * SAMPLE_EVERY);
    let pts: Vec<f64> = (0..k)
        .map(|i| first + i * SAMPLE_EVERY)
        .filter(|x| *x <= s)
        .map(train_loss)
        .collect();
    pts.iter().sum::<f64>() / pts.len().max(1) as f64
}

/// Validation loss (every 50 steps).
pub fn val_loss(s: u64) -> f64 {
    let x = s as f64;
    0.47 + 1.9 * (-x / 270.0).exp() + 0.00004 * (x - 900.0).max(0.0)
}

/// Learning rate: 60 warm-up steps to 2e-4, then cosine to 2e-5.
pub fn lr(s: u64) -> f64 {
    let peak = 2e-4;
    if s < 60 {
        return peak * (s as f64 + 1.0) / 60.0;
    }
    let t = (s - 60) as f64 / (TOTAL - 60) as f64;
    2e-5 + (peak - 2e-5) * 0.5 * (1.0 + (t * std::f64::consts::PI).cos())
}

/// `1.84e-4`.
pub fn sci(v: f64) -> String {
    let e = v.abs().log10().floor() as i32;
    format!("{:.2}e{e}", v / 10f64.powi(e))
}

/// `34m 12s`, `1h 02m`.
pub fn dur(secs: f64) -> String {
    let s = secs.round() as u64;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}m {:02}s", s / 60, s % 60)
    }
}

/// Where the job is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Waiting for GPUs.
    Queued,
    /// Running, no samples yet.
    Starting,
    /// Training.
    Running,
    /// Paused.
    Paused,
    /// Finished.
    Completed,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
}

impl Status {
    /// The word.
    pub fn word(self) -> &'static str {
        match self {
            Status::Queued => "Queued",
            Status::Starting => "Starting",
            Status::Running => "Training",
            Status::Paused => "Paused",
            Status::Completed => "Completed",
            Status::Failed => "Failed",
            Status::Cancelled => "Cancelled",
        }
    }
    /// The hue.
    pub fn hue(self) -> &'static str {
        match self {
            Status::Running | Status::Starting => "blue",
            Status::Paused | Status::Queued => "neutral",
            Status::Completed => "green",
            Status::Failed => "red",
            Status::Cancelled => "neutral",
        }
    }
}

/// The live state of the job.
#[derive(Clone, Debug, PartialEq)]
pub struct Training {
    /// This attempt's start.
    pub t0: u64,
    /// Steps already done when this attempt started (a retry starts at a checkpoint).
    pub from_step: u64,
    /// Paused at, and ms spent paused before.
    pub paused: Option<u64>,
    /// Total paused ms.
    pub paused_ms: u64,
    /// Cancelled at.
    pub cancelled: Option<(u64, u64)>,
    /// Cancel pressed once, at.
    pub confirm: Option<u64>,
    /// Fails at [`OOM_STEP`] (the oom scenario).
    pub oom: bool,
    /// Deploy pressed at.
    pub deploy: Option<u64>,
    /// Inspected sample index.
    pub inspect: Option<usize>,
    /// Playback speed, steps per tick multiplier (×100).
    pub speed: u64,
    /// Endpoint copied at.
    pub copied: Option<u64>,
}

impl Training {
    /// A job queued at `now`.
    pub fn new(oom: bool, speed: u64, now: u64) -> Self {
        Self {
            t0: now,
            from_step: 0,
            paused: None,
            paused_ms: 0,
            cancelled: None,
            confirm: None,
            oom,
            deploy: None,
            inspect: None,
            speed: speed.clamp(25, 800),
            copied: None,
        }
    }

    fn active(&self, now: u64) -> u64 {
        let end = self.paused.unwrap_or(now);
        end.saturating_sub(self.t0).saturating_sub(self.paused_ms)
    }

    /// Raw step (unclamped by failure), 0 while queued.
    fn raw_step(&self, now: u64) -> u64 {
        let a = self.active(now);
        let q = if self.from_step > 0 {
            300
        } else {
            QUEUE_MS + START_MS
        };
        if a < q {
            return self.from_step;
        }
        (self.from_step + (a - q) * self.speed / 100 / STEP_MS).min(TOTAL)
    }

    /// The step reached.
    pub fn step(&self, now: u64) -> u64 {
        let s = self.raw_step(now);
        let s = if self.oom && self.from_step < OOM_STEP {
            s.min(OOM_STEP)
        } else {
            s
        };
        match self.cancelled {
            Some((_, at)) => s.min(at),
            None => s,
        }
    }

    /// The status at `now`.
    pub fn status(&self, now: u64) -> Status {
        if self.cancelled.is_some() {
            return Status::Cancelled;
        }
        let s = self.step(now);
        if self.oom && self.from_step < OOM_STEP && self.raw_step(now) >= OOM_STEP {
            return Status::Failed;
        }
        if s >= TOTAL {
            return Status::Completed;
        }
        if self.paused.is_some() {
            return Status::Paused;
        }
        let a = self.active(now);
        if self.from_step == 0 && a < QUEUE_MS {
            Status::Queued
        } else if self.from_step == 0 && a < QUEUE_MS + START_MS {
            Status::Starting
        } else {
            Status::Running
        }
    }

    /// Epoch (1-based) at a step.
    pub fn epoch(&self, now: u64) -> u64 {
        (self.step(now) * EPOCHS / TOTAL + 1).min(EPOCHS)
    }

    /// Sample steps drawn so far.
    pub fn samples(&self, now: u64) -> Vec<u64> {
        let s = self.step(now);
        if self.status(now) == Status::Queued || self.status(now) == Status::Starting {
            return Vec::new();
        }
        (0..=s / SAMPLE_EVERY).map(|i| i * SAMPLE_EVERY).collect()
    }

    /// Validation sample steps.
    pub fn vals(&self, now: u64) -> Vec<u64> {
        self.samples(now)
            .into_iter()
            .filter(|s| s % 50 == 0 && *s > 0)
            .collect()
    }

    /// Elapsed simulated seconds.
    pub fn elapsed(&self, now: u64) -> f64 {
        self.step(now) as f64 * SIM_SECS
    }

    /// ETA in simulated seconds.
    pub fn eta(&self, now: u64) -> f64 {
        (TOTAL - self.step(now)) as f64 * SIM_SECS
    }

    /// The event log, oldest first: (kind, step, text).
    pub fn events(&self, now: u64) -> Vec<(&'static str, u64, String)> {
        let s = self.step(now);
        let st = self.status(now);
        let mut v: Vec<(&'static str, u64, String)> = Vec::new();
        if st == Status::Queued {
            return v;
        }
        v.push(("start", 0, "Job started on 8× H100".into()));
        for k in 1..=s / 50 {
            let at = k * 50;
            if at % 200 == 0 {
                v.push(("eval", at, format!("Eval: val loss {:.3}", val_loss(at))));
            }
            if at % CKPT_EVERY == 0 {
                v.push(("checkpoint", at, format!("Checkpoint saved at step {at}")));
                v.push((
                    "epoch",
                    at,
                    format!("Epoch {} of {EPOCHS} done", at / CKPT_EVERY),
                ));
            }
        }
        if self.from_step > 0 {
            v.push((
                "info",
                self.from_step,
                format!("Resumed from checkpoint {}", self.from_step),
            ));
        }
        match st {
            Status::Failed => v.push((
                "error",
                OOM_STEP,
                format!("CUDA out of memory at step {OOM_STEP}"),
            )),
            Status::Cancelled => v.push(("warning", s, "Cancelled by you".into())),
            Status::Completed => v.push(("info", TOTAL, "Training finished".into())),
            _ => {}
        }
        v.sort_by_key(|e| e.1);
        v
    }

    /// The four results on completion: (label, value).
    pub fn metrics(&self) -> [(&'static str, String); 4] {
        [
            ("Final train loss", format!("{:.3}", train_smooth(TOTAL))),
            ("Val loss", format!("{:.3}", val_loss(TOTAL))),
            ("Steps", super::thousands(TOTAL)),
            ("Time", dur(TOTAL as f64 * SIM_SECS)),
        ]
    }

    /// Cancel is waiting for its confirm press.
    pub fn confirming(&self, now: u64) -> bool {
        self.confirm.is_some_and(|c| now < c + CONFIRM_MS)
    }

    /// Deploy: `None`, `loading`, `done`.
    pub fn deploy_state(&self, now: u64) -> &'static str {
        match self.deploy {
            None => "none",
            Some(t) if now < t + DEPLOY_MS => "loading",
            _ => "done",
        }
    }

    /// The chart's accessible name / legend readout for the inspected or latest sample.
    pub fn readout(&self, now: u64) -> String {
        let ss = self.samples(now);
        let Some(&s) = self.inspect.and_then(|i| ss.get(i)).or(ss.last()) else {
            return "No samples yet".into();
        };
        let v = self.vals(now).into_iter().rfind(|v| *v <= s);
        match v {
            Some(v) => format!(
                "Step {s}: train {:.3}, validation {:.3}",
                train_smooth(s),
                val_loss(v)
            ),
            None => format!("Step {s}: train {:.3}", train_smooth(s)),
        }
    }

    /// Presses: `tr-pause`, `tr-resume`, `tr-cancel` (twice), `tr-retry`, `tr-restart`,
    /// `tr-deploy`, `tr-copy`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        let st = self.status(now);
        match id {
            "tr-pause" if st == Status::Running => {
                self.paused = Some(now);
                Some("Paused".into())
            }
            "tr-resume" if st == Status::Paused => {
                let p = self.paused.take()?;
                self.paused_ms += now - p;
                Some("Training".into())
            }
            "tr-cancel"
                if matches!(
                    st,
                    Status::Queued | Status::Starting | Status::Running | Status::Paused
                ) =>
            {
                if self.confirming(now) {
                    self.cancelled = Some((now, self.step(now)));
                    self.confirm = None;
                    Some("Cancelled".into())
                } else {
                    self.confirm = Some(now);
                    Some("Press Confirm cancel to stop the run".into())
                }
            }
            "tr-retry" if st == Status::Failed => {
                let ck = OOM_STEP / CKPT_EVERY * CKPT_EVERY;
                *self = Self {
                    from_step: ck,
                    oom: false,
                    ..Self::new(false, self.speed, now)
                };
                Some(format!("Retrying from checkpoint {ck}"))
            }
            "tr-restart"
                if matches!(st, Status::Completed | Status::Failed | Status::Cancelled) =>
            {
                *self = Self::new(self.oom, self.speed, now);
                Some("Queued".into())
            }
            "tr-deploy" if st == Status::Completed && self.deploy.is_none() => {
                self.deploy = Some(now);
                Some("Deploying".into())
            }
            "tr-copy" if self.deploy_state(now) == "done" => {
                self.copied = Some(now);
                Some("Endpoint copied".into())
            }
            _ => None,
        }
    }

    /// Keys on the chart: Left, Right (Shift jumps ten), Home, End, Escape.
    pub fn key(&mut self, key: &str, shift: bool, now: u64) -> Option<String> {
        let key = norm_key(key);
        let n = self.samples(now).len();
        if n == 0 {
            return None;
        }
        let jump = if shift { 10 } else { 1 };
        let cur = self.inspect.unwrap_or(n - 1);
        self.inspect = match key {
            "ArrowLeft" => Some(cur.saturating_sub(jump)),
            "ArrowRight" => Some((cur + jump).min(n - 1)),
            "Home" => Some(0),
            "End" => Some(n - 1),
            "Escape" => {
                self.inspect.take()?;
                return Some("Back to the latest step".into());
            }
            _ => return None,
        };
        Some(self.readout(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trains_pauses_cancels_fails_and_deploys() {
        let mut t = Training::new(false, 100, 0);
        assert_eq!(t.status(100), Status::Queued);
        assert_eq!(t.status(QUEUE_MS + 10), Status::Starting);
        let run = QUEUE_MS + START_MS + 1_000;
        assert_eq!(t.status(run), Status::Running);
        assert!(t.step(run) > 40);
        t.press("tr-pause", run);
        assert_eq!(t.step(run + 5_000), t.step(run));
        t.press("tr-resume", run + 5_000);
        assert_eq!(t.status(run + 5_001), Status::Running);
        let end = run + 5_000 + TOTAL * STEP_MS;
        assert_eq!(t.status(end), Status::Completed);
        assert!(t.events(end).len() > 6);
        t.press("tr-deploy", end);
        assert_eq!(t.deploy_state(end + 100), "loading");
        assert_eq!(t.deploy_state(end + DEPLOY_MS), "done");
        t.key("ArrowLeft", true, end);
        assert!(t.readout(end).contains("validation"));
        let mut c = Training::new(false, 100, 0);
        c.press("tr-cancel", run);
        assert_eq!(c.status(run + 10), Status::Running);
        c.press("tr-cancel", run + 500);
        assert_eq!(c.status(run + 600), Status::Cancelled);
        let mut f = Training::new(true, 400, 0);
        assert_eq!(f.status(100_000), Status::Failed);
        assert_eq!(f.step(100_000), OOM_STEP);
        f.press("tr-retry", 100_000);
        assert_eq!(f.step(100_100), 800);
        assert_eq!(f.status(200_000), Status::Completed);
        assert_eq!(sci(1.84e-4), "1.84e-4");
        assert!(lr(0) < lr(60));
    }
}
