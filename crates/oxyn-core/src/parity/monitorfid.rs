//! affectUI fidelity pass (plan #32, overnight batch, family B part 4): the model behind the exact
//! **Agent Monitor** – one agent at work as a card: its current step with a clock and the bill so
//! far, a time strip and the finished steps with each tool's logo, the exact command when it needs
//! an OK, and a fold into a small bar. Behaviour only; times are milliseconds.

/// Where the run is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorState {
    /// Not started.
    Idle,
    /// Working.
    Running,
    /// Needs an OK.
    Waiting,
    /// Paused by someone.
    Paused,
    /// Done.
    Finished,
    /// Stopped by an error.
    Failed,
    /// Stopped by someone.
    Cancelled,
}

impl MonitorState {
    /// The pill's words.
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Paused => "Paused",
            Self::Finished => "Finished",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
    /// `idle` · `running` · `waiting` · `paused` · `finished` · `failed` · `cancelled`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Paused => "paused",
            Self::Finished => "finished",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    /// Whether the clock runs and the cost shimmers.
    pub fn busy(self) -> bool {
        matches!(self, Self::Running | Self::Waiting)
    }
}

/// A step's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonStepState {
    /// Not yet.
    Queued,
    /// Now.
    Running,
    /// Done.
    Done,
    /// Failed.
    Failed,
}

/// One step of the run.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorStep {
    /// Id.
    pub id: String,
    /// What it does or did ("Wrote the changelog").
    pub label: String,
    /// Tool brand mark (empty: a plain icon by `kind`).
    pub brand: String,
    /// `read` · `search` · `edit` · `run` · `think` · `web`.
    pub kind: String,
    /// State.
    pub state: MonStepState,
    /// When it began (ms since the run began).
    pub since: u64,
    /// How long it took (ms), once done or failed.
    pub took: u64,
    /// The tool's name in the info card.
    pub tool: String,
    /// One sentence on what happened.
    pub detail: String,
    /// The command it ran or the place it touched.
    pub output: String,
}

/// What it asks while waiting.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorAsk {
    /// Short title ("Ready to publish").
    pub title: String,
    /// The exact command.
    pub command: String,
    /// Tool brand.
    pub brand: String,
}

/// The card at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorScene {
    /// Agent name.
    pub agent: String,
    /// Role or model line.
    pub role: String,
    /// Agent brand mark.
    pub brand: String,
    /// State.
    pub state: MonitorState,
    /// Steps, oldest first.
    pub steps: Vec<MonitorStep>,
    /// Clock (ms since the run began, pauses left out).
    pub clock: u64,
    /// USD so far.
    pub cost: f64,
    /// The ask while waiting.
    pub ask: Option<MonitorAsk>,
    /// What went wrong, once failed.
    pub error: String,
    /// After "Finished in 18s".
    pub note: String,
}

/// "0:07" · "1:04".
pub fn monitor_clock(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// "2.3s" · "41s" · "1m 5s".
pub fn step_took(ms: u64) -> String {
    if ms < 10_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 60_000 {
        format!("{}s", ms / 1000)
    } else {
        format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1000)
    }
}

/// The cost as ("$0.04", "9"): the last digit goes muted while live.
pub fn monitor_cost(c: f64) -> (String, String) {
    let s = format!("${c:.3}");
    let (a, b) = s.split_at(s.len() - 1);
    (a.to_string(), b.to_string())
}

/// "4/6": steps started or done out of all.
pub fn step_count(steps: &[MonitorStep]) -> String {
    let n = steps
        .iter()
        .filter(|s| s.state != MonStepState::Queued)
        .count();
    format!("{n}/{}", steps.len())
}

/// The time strip: (step index, share of the run 0–1) for every step that started.
pub fn time_strip(steps: &[MonitorStep], clock: u64) -> Vec<(usize, f32)> {
    let span: u64 = steps
        .iter()
        .map(|s| match s.state {
            MonStepState::Running => clock.saturating_sub(s.since).max(1),
            MonStepState::Queued => 0,
            _ => s.took.max(1),
        })
        .sum::<u64>()
        .max(1);
    steps
        .iter()
        .enumerate()
        .filter(|(_, s)| s.state != MonStepState::Queued)
        .map(|(i, s)| {
            let d = match s.state {
                MonStepState::Running => clock.saturating_sub(s.since).max(1),
                _ => s.took.max(1),
            };
            (i, d as f32 / span as f32)
        })
        .collect()
}

/// A step's share of the run, "23%".
pub fn step_share(steps: &[MonitorStep], i: usize) -> String {
    let total: u64 = steps.iter().map(|s| s.took).sum::<u64>().max(1);
    format!(
        "{}%",
        (steps.get(i).map_or(0, |s| s.took) * 100 + total / 2) / total
    )
}

/// The card's accessible name: "Claude Opus 4.1: Running – Wrote the changelog".
pub fn monitor_label(s: &MonitorScene) -> String {
    let now = s
        .steps
        .iter()
        .rev()
        .find(|x| x.state == MonStepState::Running)
        .map(|x| x.label.clone())
        .or_else(|| s.ask.as_ref().map(|a| a.title.clone()))
        .unwrap_or_default();
    if now.is_empty() {
        format!("{}: {}", s.agent, s.state.label())
    } else {
        format!("{}: {} – {now}", s.agent, s.state.label())
    }
}

/// One pass of the demo (real ms).
pub const MONITOR_LOOP_MS: u64 = 24_000;
/// When the demo's ask appears and how long it waits (real ms).
pub const MONITOR_ASK_AT: u64 = 9_100;
/// How long the demo waits for an OK before it goes on by itself.
pub const MONITOR_ASK_MS: u64 = 3_400;

struct Plan(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    u64,
    &'static str,
    &'static str,
);

const PLAN: [Plan; 6] = [
    Plan(
        "tickets",
        "Gathered 11 tickets",
        "linear",
        "search",
        1_600,
        "Pulled the tickets closed since v2.3",
        "Team: Web · Done since 3 Nov",
    ),
    Plan(
        "commits",
        "Read 24 commits",
        "github",
        "read",
        2_100,
        "Compared main against the last tag",
        "git log v2.3.0..main",
    ),
    Plan(
        "notes",
        "Wrote the changelog",
        "notion",
        "edit",
        2_600,
        "Grouped the changes into fixes and features",
        "Releases / v2.4",
    ),
    Plan(
        "build",
        "Built the preview in 33s",
        "",
        "run",
        2_400,
        "A clean build with every test passing",
        "npm run build",
    ),
    Plan(
        "publish",
        "Published v2.4.0",
        "github",
        "run",
        1_500,
        "Tagged the release and attached the notes",
        "gh release create v2.4.0",
    ),
    Plan(
        "announce",
        "Posted in #releases",
        "slack",
        "web",
        1_800,
        "Shared the notes with a link to the release",
        "#releases",
    ),
];

fn running_label(id: &str) -> &'static str {
    match id {
        "tickets" => "Gathering the tickets",
        "commits" => "Reading the commits",
        "notes" => "Writing the changelog",
        "build" => "Building the preview",
        "publish" => "Publishing v2.4.0",
        _ => "Posting in #releases",
    }
}

/// The demo card `t` real ms into a pass: five steps run, the card asks before publishing (it
/// goes on by itself after a few seconds; `allowed_at` – real ms – lets it go on sooner, a
/// `declined_at` cancels it), the last step posts and the run finishes.
pub fn monitor_at(t: u64, allowed_at: Option<u64>, declined_at: Option<u64>) -> MonitorScene {
    let mut steps = vec![];
    let mut at = 400u64;
    // Wait (real ms) actually spent at the ask.
    let wait_end = allowed_at
        .filter(|a| *a >= MONITOR_ASK_AT)
        .map_or(MONITOR_ASK_AT + MONITOR_ASK_MS, |a| {
            a.min(MONITOR_ASK_AT + MONITOR_ASK_MS)
        });
    let mut state = if t < at {
        MonitorState::Idle
    } else {
        MonitorState::Running
    };
    let mut ask = None;
    for (i, p) in PLAN.iter().enumerate() {
        if i == 4 {
            // The ask before publishing.
            if t >= at && t < wait_end {
                state = MonitorState::Waiting;
                ask = Some(MonitorAsk {
                    title: "Ready to publish".into(),
                    command: p.6.into(),
                    brand: "github".into(),
                });
            }
            at = wait_end;
        }
        let st = if t < at {
            MonStepState::Queued
        } else if t < at + p.4 {
            MonStepState::Running
        } else {
            MonStepState::Done
        };
        let label = if st == MonStepState::Running {
            running_label(p.0).to_string()
        } else {
            p.1.to_string()
        };
        steps.push(MonitorStep {
            id: p.0.into(),
            label,
            brand: p.2.into(),
            kind: p.3.into(),
            state: st,
            since: at.saturating_sub(400),
            took: if st == MonStepState::Done { p.4 } else { 0 },
            tool: match p.2 {
                "" => "Terminal".into(),
                b => {
                    let mut c = b.chars();
                    c.next()
                        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                        .unwrap_or_default()
                }
            },
            detail: p.5.into(),
            output: p.6.into(),
        });
        at += p.4;
    }
    let end = at;
    if t >= end {
        state = MonitorState::Finished;
    }
    if let Some(d) = declined_at.filter(|d| *d >= MONITOR_ASK_AT && *d < wait_end && t >= *d) {
        for s in steps.iter_mut().skip(4) {
            s.state = MonStepState::Queued;
            s.took = 0;
            s.label = PLAN.iter().find(|p| p.0 == s.id).map_or("", |p| p.1).into();
        }
        let clock = d.saturating_sub(400);
        return MonitorScene {
            clock,
            cost: cost_at(clock),
            ..base(MonitorState::Cancelled, steps, None)
        };
    }
    let clock = t.min(end).saturating_sub(400);
    MonitorScene {
        clock,
        cost: cost_at(clock),
        note: if state == MonitorState::Finished {
            "+214 −38 in 9 files".into()
        } else {
            String::new()
        },
        ..base(state, steps, ask)
    }
}

fn cost_at(clock: u64) -> f64 {
    clock as f64 * 0.0000041
}

fn base(state: MonitorState, steps: Vec<MonitorStep>, ask: Option<MonitorAsk>) -> MonitorScene {
    MonitorScene {
        agent: "Claude Opus 4.1".into(),
        role: "Release agent · app v2.4".into(),
        brand: "claude".into(),
        state,
        steps,
        clock: 0,
        cost: 0.0,
        ask,
        error: String::new(),
        note: String::new(),
    }
}

/// A failed run for the failed variant: the third step could not open its sheet.
pub fn monitor_failed() -> MonitorScene {
    let mut s = monitor_at(5_200, None, None);
    s.agent = "GPT-5".into();
    s.role = "Support agent · ticket #3107".into();
    s.brand = "openai".into();
    for st in s.steps.iter_mut() {
        if st.state == MonStepState::Running {
            st.state = MonStepState::Failed;
            st.took = 900;
            st.label = "Couldn't open the orders sheet".into();
            st.brand = "googledrive".into();
        }
    }
    s.state = MonitorState::Failed;
    s.error = "The orders sheet is locked".into();
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_runs_asks_and_finishes() {
        assert_eq!(monitor_at(0, None, None).state, MonitorState::Idle);
        let r = monitor_at(3_000, None, None);
        assert_eq!(r.state, MonitorState::Running);
        assert_eq!(step_count(&r.steps), "2/6");
        assert_eq!(r.steps[1].label, "Reading the commits");
        let w = monitor_at(MONITOR_ASK_AT + 500, None, None);
        assert_eq!(w.state, MonitorState::Waiting);
        assert_eq!(w.ask.as_ref().unwrap().command, "gh release create v2.4.0");
        let a = monitor_at(MONITOR_ASK_AT + 900, Some(MONITOR_ASK_AT + 600), None);
        assert_eq!(a.state, MonitorState::Running);
        let d = monitor_at(MONITOR_ASK_AT + 900, None, Some(MONITOR_ASK_AT + 600));
        assert_eq!(d.state, MonitorState::Cancelled);
        let f = monitor_at(20_000, None, None);
        assert_eq!(f.state, MonitorState::Finished);
        assert_eq!(step_count(&f.steps), "6/6");
        assert_eq!(step_share(&f.steps, 2), "22%");
        assert_eq!(monitor_failed().state, MonitorState::Failed);
    }

    #[test]
    fn formats() {
        assert_eq!(monitor_clock(67_000), "1:07");
        assert_eq!(step_took(2_300), "2.3s");
        assert_eq!(step_took(41_000), "41s");
        assert_eq!(monitor_cost(0.0491), ("$0.04".to_string(), "9".to_string()));
        let s = monitor_at(3_000, None, None);
        let strip = time_strip(&s.steps, s.clock);
        assert_eq!(strip.len(), 2);
        assert!((strip.iter().map(|x| x.1).sum::<f32>() - 1.0).abs() < 0.01);
        assert_eq!(
            monitor_label(&s),
            "Claude Opus 4.1: Running – Reading the commits"
        );
    }
}
