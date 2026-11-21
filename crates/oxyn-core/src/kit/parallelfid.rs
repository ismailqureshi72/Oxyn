//! affectUI fidelity pass, family B part 9 (plan #32): the **Parallel Attempts** model. One task
//! is tried several ways at once; each attempt races with a model, an approach, a live progress
//! bar, the step it is on and its running cost, then finishes with a score (or fails where it
//! stopped). The highest score is Best and is picked unless you pick another. Behaviour only: the
//! views live in `oxyn-dioxus` / `oxyn-gpui` (`kit/parallelfid.rs`).

/// Where an attempt stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaStatus {
    /// Still going.
    Running,
    /// Finished with a score.
    Done,
    /// Stopped with an error.
    Failed,
}

impl PaStatus {
    /// `running` / `done` / `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// One attempt as drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct PaAttempt {
    /// Stable id.
    pub id: String,
    /// Model name.
    pub model: String,
    /// Brand id for the model tile.
    pub brand: String,
    /// The approach it takes.
    pub approach: String,
    /// Running, done or failed.
    pub status: PaStatus,
    /// 0–1 (a failed attempt keeps where it stopped).
    pub progress: f64,
    /// What it is doing now (running).
    pub step: String,
    /// 0–100 once done.
    pub score: Option<u8>,
    /// The outcome line once finished.
    pub note: String,
    /// Seconds taken so far.
    pub took_s: u32,
    /// Dollars spent so far.
    pub cost: f64,
}

/// A scripted attempt for live demos: how long it runs, its steps, how it ends.
#[derive(Clone, Debug, PartialEq)]
pub struct PaScript {
    /// The attempt at its end (status Done or Failed, progress where it stops).
    pub end: PaAttempt,
    /// Run time to the end, ms (demo time).
    pub ms: u64,
    /// Steps, shown in turn while running.
    pub steps: Vec<String>,
}

/// Every scripted attempt `el` ms after the start.
pub fn pa_at(scripts: &[PaScript], el: u64) -> Vec<PaAttempt> {
    scripts
        .iter()
        .map(|s| {
            if el >= s.ms {
                return s.end.clone();
            }
            let f = el as f64 / s.ms.max(1) as f64;
            let p = (s.end.progress * f).clamp(0.03, 1.0);
            let n = s.steps.len().max(1);
            let step = s
                .steps
                .get(((f * n as f64) as usize).min(n - 1))
                .cloned()
                .unwrap_or_default();
            PaAttempt {
                status: PaStatus::Running,
                progress: p,
                step,
                score: None,
                note: String::new(),
                took_s: (s.end.took_s as f64 * f) as u32,
                cost: s.end.cost * f.max(0.02),
                ..s.end.clone()
            }
        })
        .collect()
}

/// The longest script, ms.
pub fn pa_len(scripts: &[PaScript]) -> u64 {
    scripts.iter().map(|s| s.ms).max().unwrap_or(0)
}

/// The best finished attempt (highest score; the earlier one on a tie).
pub fn pa_best(a: &[PaAttempt]) -> Option<usize> {
    a.iter()
        .enumerate()
        .filter(|(_, x)| x.status == PaStatus::Done && x.score.is_some())
        .max_by_key(|(i, x)| (x.score, std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
}

/// Finished attempts (done or failed).
pub fn pa_finished(a: &[PaAttempt]) -> usize {
    a.iter().filter(|x| x.status != PaStatus::Running).count()
}

/// Money spent by every attempt.
pub fn pa_cost(a: &[PaAttempt]) -> f64 {
    a.iter().map(|x| x.cost).sum()
}

/// "$0.31" / "$0.007".
pub fn pa_usd(v: f64) -> String {
    if v > 0.0 && v < 0.01 {
        format!("${v:.3}")
    } else {
        format!("${v:.2}")
    }
}

/// "2m 36s" / "48s".
pub fn pa_dur(s: u32) -> String {
    if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

#[allow(clippy::too_many_arguments)]
fn end(
    id: &str,
    model: &str,
    brand: &str,
    approach: &str,
    status: PaStatus,
    progress: f64,
    score: Option<u8>,
    note: &str,
    took_s: u32,
    cost: f64,
) -> PaAttempt {
    PaAttempt {
        id: id.into(),
        model: model.into(),
        brand: brand.into(),
        approach: approach.into(),
        status,
        progress,
        step: String::new(),
        score,
        note: note.into(),
        took_s,
        cost,
    }
}

fn steps(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// The demo race: three ways to steady a flaky test – one wins, one scores lower, one fails.
pub fn pa_demo() -> Vec<PaScript> {
    vec![
        PaScript {
            end: end(
                "a",
                "Opus 4.1",
                "claude",
                "Retry the request with backoff",
                PaStatus::Done,
                1.0,
                Some(78),
                "Passes 19 of 20 runs · slower by 1.1s",
                151,
                0.29,
            ),
            ms: 7600,
            steps: steps(&[
                "Reading payment.spec.ts",
                "Adding the retry",
                "Running the suite 20×",
            ]),
        },
        PaScript {
            end: end(
                "b",
                "GPT-5",
                "openai",
                "Stub the payment gateway",
                PaStatus::Done,
                1.0,
                Some(94),
                "20 of 20 runs pass · no extra time",
                128,
                0.33,
            ),
            ms: 6400,
            steps: steps(&[
                "Reading payment.spec.ts",
                "Writing the stub",
                "Running the suite 20×",
            ]),
        },
        PaScript {
            end: end(
                "c",
                "Gemini 2.5 Pro",
                "gemini",
                "Wait for the network to settle",
                PaStatus::Failed,
                0.62,
                None,
                "Still flaky – 4 of 20 runs fail",
                96,
                0.14,
            ),
            ms: 5200,
            steps: steps(&["Reading payment.spec.ts", "Adding the waits"]),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_runs_and_settles() {
        let d = pa_demo();
        let a = pa_at(&d, 0);
        assert!(a.iter().all(|x| x.status == PaStatus::Running));
        assert!((a[0].progress - 0.03).abs() < 1e-9);
        assert_eq!(a[0].step, "Reading payment.spec.ts");
        assert_eq!(pa_best(&a), None);
        let mid = pa_at(&d, 5500);
        assert_eq!(mid[2].status, PaStatus::Failed);
        assert_eq!(pa_finished(&mid), 1);
        assert_eq!(mid[0].step, "Running the suite 20×");
        let end = pa_at(&d, pa_len(&d));
        assert_eq!(pa_finished(&end), 3);
        assert_eq!(pa_best(&end), Some(1));
        assert!((pa_cost(&end) - 0.76).abs() < 1e-9);
        assert_eq!(end[2].progress, 0.62);
    }

    #[test]
    fn formats() {
        assert_eq!(pa_usd(0.007), "$0.007");
        assert_eq!(pa_usd(0.76), "$0.76");
        assert_eq!(pa_dur(156), "2m 36s");
        assert_eq!(pa_dur(48), "48s");
    }
}
