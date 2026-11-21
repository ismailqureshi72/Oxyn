//! affectUI fidelity pass (plan #32, overnight batch, family B part 4): the model behind the exact
//! **Job queue** – the work waiting for your agents and how fast it moves: jobs drop into a live
//! queue, glide into free worker slots as growing bars and land in Done or Failed, while a depth
//! chart shows the backlog build up when a rate limit pauses the workers and drain after.
//! Behaviour only: a deterministic simulation driven by a clock and a list of steering events.

/// What a job is: picks its tint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    /// An LLM call (violet).
    Llm,
    /// A tool call (teal).
    Tool,
    /// A sub-agent (slate).
    Agent,
    /// Human review (blue).
    Human,
}

impl JobKind {
    /// "LLM call" · "Tool call" · "Sub-agent" · "Human review".
    pub fn label(self) -> &'static str {
        match self {
            Self::Llm => "LLM call",
            Self::Tool => "Tool call",
            Self::Agent => "Sub-agent",
            Self::Human => "Human review",
        }
    }
    /// `llm` · `tool` · `agent` · `human`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::Tool => "tool",
            Self::Agent => "agent",
            Self::Human => "human",
        }
    }
}

/// Where a job is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    /// Waiting in line.
    Queued,
    /// On a worker.
    Running,
    /// Finished.
    Done,
    /// Gave up.
    Failed,
}

/// One job.
#[derive(Clone, Debug, PartialEq)]
pub struct QueueJob {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Kind.
    pub kind: JobKind,
    /// Brand mark (empty: a plain icon by kind).
    pub brand: String,
    /// Status.
    pub status: JobStatus,
    /// When it joined the queue (ms).
    pub enqueued_at: i64,
    /// When it started on a worker (ms).
    pub started_at: Option<i64>,
    /// When it ended (ms).
    pub ended_at: Option<i64>,
    /// The worker slot it runs on.
    pub worker: Option<usize>,
    /// 0–1 while it runs.
    pub progress: f32,
    /// High priority: goes to the front.
    pub high: bool,
    /// Attempt number (1 = first).
    pub attempt: u32,
    /// Attempts allowed.
    pub max_attempts: u32,
    /// A short line in the job's card.
    pub detail: String,
}

/// A worker slot.
#[derive(Clone, Debug, PartialEq)]
pub struct QueueWorker {
    /// Model name.
    pub name: String,
    /// Brand mark.
    pub brand: String,
}

/// Steering that changes the run, at a moment (ms on the queue's clock).
#[derive(Clone, Debug, PartialEq)]
pub enum QueueEvent {
    /// Pause (`true`) or resume.
    Pause(bool),
    /// A new number of workers.
    Workers(usize),
    /// Move a job to the front.
    Promote(String),
    /// Take a waiting job out.
    Cancel(String),
}

/// The queue at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct QueueScene {
    /// Waiting jobs, front first.
    pub queued: Vec<QueueJob>,
    /// One entry per worker: its job, if any.
    pub slots: Vec<Option<QueueJob>>,
    /// The workers.
    pub workers: Vec<QueueWorker>,
    /// Done since the start.
    pub done: u32,
    /// Failed since the start.
    pub failed: u32,
    /// Paused by someone.
    pub paused: bool,
    /// Seconds left of a rate limit, when one is on.
    pub limited: Option<u32>,
    /// Finished in the last minute.
    pub per_min: u32,
    /// 95th percentile wait of the jobs that started in the last minute (ms).
    pub wait_p95: u64,
    /// Waiting count once a second over the chart's window, oldest first.
    pub depth: Vec<u32>,
    /// Rate-limited stretches in the window, as (from, to) shares of it (0–1).
    pub bands: Vec<(f32, f32)>,
    /// Backpressure line.
    pub backpressure: u32,
    /// The clock (ms).
    pub now: i64,
    /// Recently finished jobs (newest first, the last few), for "a finished job flies to Done".
    pub landed: Vec<QueueJob>,
}

/// The chart's window (ms).
pub const QUEUE_WINDOW_MS: i64 = 96_000;
/// One pass of the demo (real ms) before it starts over.
pub const QUEUE_LOOP_MS: u64 = 40_000;
/// Simulation step (ms).
const DT: i64 = 100;

fn hash(i: u64, salt: u64) -> u64 {
    let mut x = i
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    x ^= x >> 31;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 29)
}

const NAMES: [(&str, JobKind, &str, &str); 16] = [
    (
        "Summarize call #{n}",
        JobKind::Llm,
        "",
        "Support agent · call notes",
    ),
    ("Look up order #{n}", JobKind::Tool, "", "Orders API"),
    ("Label new issues", JobKind::Agent, "github", "Triage agent"),
    (
        "Draft reply – Contoso",
        JobKind::Llm,
        "",
        "Sales agent · thread",
    ),
    ("Embed 180 pages", JobKind::Tool, "", "Docs index"),
    ("Extract receipt #{n}", JobKind::Llm, "", "Finance agent"),
    ("Sync CRM contacts", JobKind::Tool, "hubspot", "Ops agent"),
    (
        "Review refund #{n}",
        JobKind::Human,
        "",
        "Waiting for a person",
    ),
    ("Translate the changelog", JobKind::Llm, "", "Docs agent"),
    ("Search the help center", JobKind::Tool, "", "Support agent"),
    ("Run the smoke tests", JobKind::Agent, "", "QA agent"),
    ("Score lead – Fabrikam", JobKind::Llm, "", "Sales agent"),
    ("Open PR #{n}", JobKind::Tool, "github", "Code agent"),
    (
        "Update the roadmap page",
        JobKind::Tool,
        "notion",
        "PM agent",
    ),
    ("Classify 20 emails", JobKind::Llm, "", "Inbox agent"),
    ("Post the digest", JobKind::Tool, "slack", "Ops agent"),
];

/// The demo's workers (up to eight).
pub fn queue_workers(n: usize) -> Vec<QueueWorker> {
    let all = [
        ("Sonnet 4.5", "claude"),
        ("GPT-5", "openai"),
        ("Gemini 2.5 Flash", "gemini"),
        ("Haiku 4.5", "claude"),
        ("GPT-5 mini", "openai"),
        ("Gemini 2.5 Pro", "gemini"),
        ("Opus 4.1", "claude"),
        ("Mistral Large", "mistral"),
    ];
    all.iter()
        .take(n.clamp(1, 8))
        .map(|(a, b)| QueueWorker {
            name: a.to_string(),
            brand: b.to_string(),
        })
        .collect()
}

/// The demo's i-th job arriving at `at`: (the job, run time ms, fails on its first try).
fn demo_job(i: u64, at: i64) -> (QueueJob, i64, bool) {
    let h = hash(i, 7);
    let (name, kind, brand, detail) = NAMES[(h % 16) as usize];
    let n = 4100 + (hash(i, 3) % 900);
    let run = 1_700 + (hash(i, 11) % 2_300) as i64;
    let fails = hash(i, 13).is_multiple_of(17);
    let high = hash(i, 17).is_multiple_of(19);
    (
        QueueJob {
            id: format!("j{i}"),
            name: name.replace("{n}", &n.to_string()),
            kind,
            brand: brand.into(),
            status: JobStatus::Queued,
            enqueued_at: at,
            started_at: None,
            ended_at: None,
            worker: None,
            progress: 0.0,
            high,
            attempt: 1,
            max_attempts: 3,
            detail: detail.into(),
        },
        run,
        fails,
    )
}

/// The demo's rate-limit stretches (ms on the clock): one in each pass and two before it, so
/// the chart has history.
pub fn queue_limits() -> Vec<(i64, i64)> {
    vec![(-78_000, -68_000), (-38_000, -29_000), (10_000, 20_000)]
}

/// The demo queue at `t` ms on its clock (0 = a pass starts; the simulation runs from 96 s
/// before on four workers so the chart is full; `workers` from 0), with the steering `events`
/// (time, event) applied.
pub fn queue_at(t: i64, workers: usize, events: &[(i64, QueueEvent)]) -> QueueScene {
    let limits = queue_limits();
    // The history runs on four workers; `workers` takes over when the pass starts.
    let mut n_workers = 4;
    let mut paused = false;
    let mut pending: Vec<(QueueJob, i64, bool)> = vec![];
    let mut next = 0u64;
    // Arrivals: about 1.2 a second, slower for 15 s after a rate limit (callers back off).
    let mut next_at = -QUEUE_WINDOW_MS - 20_000;
    let mut slots: Vec<Option<(QueueJob, i64, bool)>> = vec![None; 8];
    let mut done = 137u32;
    let mut failed = 3u32;
    let mut finished: Vec<(i64, bool)> = vec![];
    let mut waits: Vec<(i64, i64)> = vec![];
    let mut landed: Vec<QueueJob> = vec![];
    let mut depth: Vec<u32> = vec![];
    let start = -QUEUE_WINDOW_MS - 20_000;
    let mut now = start;
    let mut ev = 0usize;
    let mut evs: Vec<(i64, QueueEvent)> = vec![(0, QueueEvent::Workers(workers.clamp(1, 8)))];
    evs.extend(events.iter().cloned());
    evs.sort_by_key(|e| e.0);
    while now <= t {
        while ev < evs.len() && evs[ev].0 <= now {
            match &evs[ev].1 {
                QueueEvent::Pause(p) => paused = *p,
                QueueEvent::Workers(n) => n_workers = (*n).clamp(1, 8),
                QueueEvent::Promote(id) => {
                    if let Some(k) = pending.iter().position(|j| &j.0.id == id) {
                        let mut j = pending.remove(k);
                        j.0.high = true;
                        pending.insert(0, j);
                    }
                }
                QueueEvent::Cancel(id) => pending.retain(|j| &j.0.id != id),
            }
            ev += 1;
        }
        // Arrivals.
        while next_at <= now {
            let j = demo_job(next, next_at);
            let backoff = limits
                .iter()
                .any(|(_, b)| next_at >= *b && next_at < b + 15_000);
            next_at += if backoff { 1_500 } else { 600 } + (hash(next, 5) % 300) as i64;
            next += 1;
            if j.0.high {
                let k = pending
                    .iter()
                    .position(|p| !p.0.high)
                    .unwrap_or(pending.len());
                pending.insert(k, j);
            } else {
                pending.push(j);
            }
        }
        let limited = limits.iter().any(|(a, b)| now >= *a && now < *b);
        let stopped = paused || limited;
        // Workers beyond the count finish what they hold but take nothing new.
        for (w, slot) in slots.iter_mut().enumerate() {
            if let Some((job, run, fails)) = slot.as_mut()
                && !stopped
            {
                job.progress += DT as f32 / *run as f32;
                if job.progress >= 1.0 {
                    let fail = *fails && job.attempt == 1;
                    let mut j = job.clone();
                    j.ended_at = Some(now);
                    j.progress = 1.0;
                    if fail && j.attempt < j.max_attempts {
                        // Back into the queue with a retry badge.
                        let mut again = (j.clone(), *run, false);
                        again.0.status = JobStatus::Queued;
                        again.0.attempt += 1;
                        again.0.progress = 0.0;
                        again.0.started_at = None;
                        again.0.ended_at = None;
                        again.0.worker = None;
                        pending.insert(0, again);
                        j.status = JobStatus::Failed;
                    } else {
                        j.status = JobStatus::Done;
                        done += 1;
                    }
                    finished.push((now, j.status == JobStatus::Done));
                    landed.insert(0, j);
                    landed.truncate(4);
                    *slot = None;
                }
            }
            if slot.is_none() && w < n_workers && !stopped && !pending.is_empty() {
                let (mut j, run, fails) = pending.remove(0);
                j.status = JobStatus::Running;
                j.started_at = Some(now);
                j.worker = Some(w);
                waits.push((now, now - j.enqueued_at));
                *slot = Some((j, run, fails));
            }
        }
        if (now - start) % 1000 == 0 && now > t - QUEUE_WINDOW_MS {
            depth.push(pending.len() as u32);
        }
        now += DT;
    }
    // Retried attempts count as failures in the totals.
    failed += landed_failures(&finished);
    let per_min = finished.iter().filter(|(at, _)| *at > t - 60_000).count() as u32;
    let mut recent: Vec<i64> = waits
        .iter()
        .filter(|(at, _)| *at > t - 60_000)
        .map(|(_, w)| *w)
        .collect();
    recent.sort_unstable();
    let wait_p95 = recent
        .get(((recent.len() as f32 * 0.95) as usize).min(recent.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0)
        .max(0) as u64;
    let from = t - QUEUE_WINDOW_MS;
    let bands = limits
        .iter()
        .filter(|(a, b)| *b > from && *a < t)
        .map(|(a, b)| {
            (
                ((*a).max(from) - from) as f32 / QUEUE_WINDOW_MS as f32,
                ((*b).min(t) - from) as f32 / QUEUE_WINDOW_MS as f32,
            )
        })
        .collect();
    let limited = limits
        .iter()
        .find(|(a, b)| t >= *a && t < *b)
        .map(|(_, b)| ((b - t + 999) / 1000) as u32);
    QueueScene {
        queued: pending.into_iter().map(|j| j.0).collect(),
        slots: slots
            .into_iter()
            .take(n_workers)
            .map(|s| s.map(|x| x.0))
            .collect(),
        workers: queue_workers(n_workers),
        done,
        failed,
        paused,
        limited,
        per_min,
        wait_p95,
        depth,
        bands,
        backpressure: 12,
        now: t,
        landed,
    }
}

fn landed_failures(finished: &[(i64, bool)]) -> u32 {
    finished.iter().filter(|(_, ok)| !ok).count() as u32
}

/// "Jobs: 4 running, 12 waiting".
pub fn queue_said(s: &QueueScene) -> String {
    let r = s.slots.iter().filter(|x| x.is_some()).count();
    format!("Jobs: {r} running, {} waiting", s.queued.len())
}

/// A job's accessible name: "Label new issues, Sub-agent, waiting 4.5s, position 2 of 9".
pub fn job_label(j: &QueueJob, now: i64, pos: Option<(usize, usize)>) -> String {
    let secs = |ms: i64| format!("{:.1}s", ms.max(0) as f64 / 1000.0);
    let mut s = match j.status {
        JobStatus::Running => format!(
            "{}, {}, running {}, {}%",
            j.name,
            j.kind.label(),
            secs(now - j.started_at.unwrap_or(now)),
            (j.progress * 100.0) as u32
        ),
        _ => format!(
            "{}, {}, waiting {}",
            j.name,
            j.kind.label(),
            secs(now - j.enqueued_at)
        ),
    };
    if let Some((p, n)) = pos {
        s.push_str(&format!(", position {p} of {n}"));
    }
    if j.high {
        s.push_str(", high priority");
    }
    if j.attempt > 1 {
        s.push_str(&format!(", attempt {} of {}", j.attempt, j.max_attempts));
    }
    s
}

/// "7.4s" for waits and run times.
pub fn queue_secs(ms: i64) -> String {
    let ms = ms.max(0);
    if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1000)
    }
}

/// The chart's y scale top: 24, or the next multiple of 8 that fits the peak.
pub fn depth_top(depth: &[u32], backpressure: u32) -> u32 {
    let peak = depth
        .iter()
        .copied()
        .max()
        .unwrap_or(0)
        .max(backpressure * 2);
    peak.div_ceil(8) * 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_queue_runs_and_a_rate_limit_builds_it_up() {
        let s = queue_at(5_000, 4, &[]);
        assert_eq!(s.slots.len(), 4);
        assert!(s.slots.iter().any(|x| x.is_some()));
        assert_eq!(s.depth.len(), 96);
        assert!(s.done > 137);
        let l = queue_at(19_500, 4, &[]);
        assert_eq!(l.limited, Some(1));
        assert!(l.queued.len() > s.queued.len());
        assert!(!l.bands.is_empty());
        let p = queue_at(6_000, 4, &[(5_000, QueueEvent::Pause(true))]);
        assert!(p.paused);
        assert!(p.queued.len() >= s.queued.len());
        let one = queue_at(10_000, 1, &[]);
        assert_eq!(one.workers.len(), 1);
        assert!(one.queued.len() > queue_at(10_000, 8, &[]).queued.len());
        assert!(queue_said(&s).starts_with("Jobs: "));
        assert_eq!(depth_top(&[3, 30], 12), 32);
        assert_eq!(queue_secs(7_400), "7.4s");
    }

    #[test]
    fn promote_and_cancel() {
        let s = queue_at(19_000, 4, &[]);
        let last = s.queued.last().unwrap().id.clone();
        let p = queue_at(19_000, 4, &[(18_900, QueueEvent::Promote(last.clone()))]);
        assert_eq!(p.queued[0].id, last);
        let c = queue_at(19_000, 4, &[(18_900, QueueEvent::Cancel(last.clone()))]);
        assert!(c.queued.iter().all(|j| j.id != last));
    }
}
