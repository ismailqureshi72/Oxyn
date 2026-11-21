//! affectUI fidelity pass (plan #32, overnight batch, family B part 3): the model behind the exact
//! **Live lanes** – where every agent is right now: a live timeline with a lane per agent, tasks
//! growing into "now", hand-offs drawn between agents and approvals right on the block. Behaviour
//! only; times are milliseconds on a clock the caller passes in.

/// An agent's lane.
#[derive(Clone, Debug, PartialEq)]
pub struct LaneAgent {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model under the name.
    pub model: String,
    /// Brand mark on its tile (initials when empty).
    pub brand: String,
}

/// A task's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneStatus {
    /// Finished.
    Done,
    /// Running: grows into "now".
    Running,
    /// Needs a decision.
    Waiting,
    /// Failed.
    Failed,
}

impl LaneStatus {
    /// "Done" · "Running" · "Needs you" · "Failed".
    pub fn label(self) -> &'static str {
        match self {
            Self::Done => "Done",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Failed => "Failed",
        }
    }
    /// `done` · `running` · `waiting` · `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
        }
    }
}

/// One block on a lane.
#[derive(Clone, Debug, PartialEq)]
pub struct LaneTask {
    /// Id.
    pub id: String,
    /// The lane's agent.
    pub agent: String,
    /// Title.
    pub title: String,
    /// State.
    pub status: LaneStatus,
    /// Start (ms on the clock).
    pub start: u64,
    /// End; `None` reaches "now".
    pub end: Option<u64>,
    /// The next agent's task this one hands over to (a dashed curve).
    pub handoff_to: String,
    /// For a waiting task: what needs deciding.
    pub ask: String,
    /// Two or three short points on what got done (the tooltip).
    pub summary: Vec<String>,
    /// Up to three tool logos before the title.
    pub tools: Vec<String>,
}

/// "11m" / "45s" / "1h 5m".
pub fn lane_duration(ms: u64) -> String {
    let s = ms / 1000;
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    }
}

/// A block's place in the window: (left, width) as shares of the timeline (0–1), clipped to
/// it; `None` when it is outside the window.
pub fn block_span(t: &LaneTask, now: u64, span: u64) -> Option<(f32, f32)> {
    let span = span.max(1);
    let from = now.saturating_sub(span);
    let end = t.end.unwrap_or(now).min(now);
    if end <= from || t.start >= now {
        return None;
    }
    let a = t.start.max(from);
    let l = (a - from) as f32 / span as f32;
    let w = (end - a) as f32 / span as f32;
    Some((l, w.max(0.004)))
}

/// Whether a block runs past the window's left edge (it fades there).
pub fn clipped_left(t: &LaneTask, now: u64, span: u64) -> bool {
    t.start < now.saturating_sub(span)
}

/// The block's accessible name: "Researcher: Market scan – Done, took 10m" / "… – Running, 9m".
pub fn block_label(agent: &str, t: &LaneTask, now: u64) -> String {
    let d = lane_duration(t.end.unwrap_or(now).min(now).saturating_sub(t.start));
    match t.status {
        LaneStatus::Done => format!("{agent}: {} – Done, took {d}", t.title),
        s => format!("{agent}: {} – {}, {d}", t.title, s.label()),
    }
}

/// (running, waiting) counts.
pub fn lane_counts(tasks: &[LaneTask]) -> (usize, usize) {
    (
        tasks
            .iter()
            .filter(|t| t.status == LaneStatus::Running)
            .count(),
        tasks
            .iter()
            .filter(|t| t.status == LaneStatus::Waiting)
            .count(),
    )
}

/// "1 task needs you" / "3 running".
pub fn lanes_said(tasks: &[LaneTask]) -> String {
    let (r, w) = lane_counts(tasks);
    if w > 0 {
        format!(
            "{w} task{} need{} you",
            if w == 1 { "" } else { "s" },
            if w == 1 { "s" } else { "" }
        )
    } else {
        format!("{r} running")
    }
}

/// The agent's tag: "Needs you" when one of its tasks waits, "Idle" when none runs.
pub fn lane_tag(agent: &str, tasks: &[LaneTask]) -> Option<&'static str> {
    let mine = tasks.iter().filter(|t| t.agent == agent);
    let mut running = false;
    for t in mine {
        match t.status {
            LaneStatus::Waiting => return Some("Needs you"),
            LaneStatus::Running => running = true,
            _ => {}
        }
    }
    (!running).then_some("Idle")
}

/// The whole board at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct LaneScene {
    /// Lanes.
    pub agents: Vec<LaneAgent>,
    /// Blocks.
    pub tasks: Vec<LaneTask>,
    /// The clock (ms).
    pub now: u64,
    /// How much time the lanes show (ms).
    pub span: u64,
}

/// How long one pass of the demo lasts before it starts over (real ms).
pub const LANES_LOOP_MS: u64 = 24_000;
/// Demo time runs a minute per real second.
pub const LANES_SPEED: u64 = 60;

const MIN: u64 = 60_000;
/// The demo clock's "now" when a pass starts (40 minutes in).
const T0: u64 = 40 * MIN;

#[allow(clippy::too_many_arguments)]
fn task(
    id: &str,
    agent: &str,
    title: &str,
    status: LaneStatus,
    start_min: i64,
    end_min: Option<i64>,
    tools: &[&str],
    summary: &[&str],
) -> LaneTask {
    let at = |m: i64| (T0 as i64 + m * MIN as i64).max(0) as u64;
    LaneTask {
        id: id.into(),
        agent: agent.into(),
        title: title.into(),
        status,
        start: at(start_min),
        end: end_min.map(at),
        handoff_to: String::new(),
        ask: String::new(),
        summary: summary.iter().map(|s| s.to_string()).collect(),
        tools: tools.iter().map(|s| s.to_string()).collect(),
    }
}

/// The demo board `t` real ms into a pass (its clock runs a minute per second).
pub fn lanes_at(t: u64) -> LaneScene {
    use LaneStatus::*;
    let m = (t / 1000) as i64;
    let agents = vec![
        LaneAgent {
            id: "research".into(),
            name: "Researcher".into(),
            model: "Opus 4.1".into(),
            brand: "claude".into(),
        },
        LaneAgent {
            id: "writer".into(),
            name: "Writer".into(),
            model: "Sonnet 4.5".into(),
            brand: "claude".into(),
        },
        LaneAgent {
            id: "review".into(),
            name: "Reviewer".into(),
            model: "GPT-5".into(),
            brand: "openai".into(),
        },
        LaneAgent {
            id: "ops".into(),
            name: "Ops".into(),
            model: "Gemini 2.5 Flash".into(),
            brand: "gemini".into(),
        },
    ];
    let mut scan = task(
        "scan",
        "research",
        "Market scan",
        Done,
        -27,
        Some(-17),
        &["notion"],
        &[
            "Compared 5 rival plans",
            "Logged the price per seat",
            "Passed the sheet to the Writer",
        ],
    );
    scan.handoff_to = "draft".into();
    let interviews_end = (m >= 12).then_some(12);
    let interviews = task(
        "interviews",
        "research",
        "User interviews",
        if m >= 12 { Done } else { Running },
        -10,
        interviews_end,
        &["notion", "slack"],
        &["Read 12 call notes", "Grouping quotes by theme"],
    );
    let mut draft = task(
        "draft",
        "writer",
        "Pricing page draft",
        if m >= 6 { Done } else { Running },
        -16,
        (m >= 6).then_some(6),
        &["figma", "notion"],
        &["Wrote the hero and three plans", "Added the FAQ"],
    );
    if m >= 6 {
        draft.handoff_to = "copy".into();
    }
    let mut tasks = vec![
        scan,
        interviews,
        draft,
        task(
            "pr",
            "review",
            "PR #317 invoices",
            Done,
            -31,
            Some(-21),
            &["github"],
            &["Checked the invoice totals", "Approved the change"],
        ),
        task(
            "crm",
            "ops",
            "CRM sync",
            Failed,
            -29,
            Some(-22),
            &["hubspot"],
            &["Token expired at step 3", "Nothing was written"],
        ),
    ];
    if m >= 6 {
        tasks.push(task(
            "copy",
            "review",
            "Pricing copy review",
            if m >= 18 { Done } else { Running },
            6,
            (m >= 18).then_some(18),
            &["figma"],
            &["Checked every price against the sheet", "Two wording fixes"],
        ));
    }
    let mut keys = task(
        "keys",
        "ops",
        "Rotate API keys",
        Waiting,
        -12,
        None,
        &["stripe", "github"],
        &[],
    );
    keys.ask =
        "Rotate the live Stripe and OpenAI keys? Running jobs reconnect within about 30 s.".into();
    tasks.push(keys);
    LaneScene {
        agents,
        tasks,
        now: T0 + t * LANES_SPEED,
        span: 30 * MIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_sit_in_the_window() {
        let s = lanes_at(0);
        let scan = s.tasks.iter().find(|t| t.id == "scan").unwrap();
        let (l, w) = block_span(scan, s.now, s.span).unwrap();
        assert!((l - 0.1).abs() < 0.001 && (w - 1.0 / 3.0).abs() < 0.001);
        let pr = s.tasks.iter().find(|t| t.id == "pr").unwrap();
        assert!(clipped_left(pr, s.now, s.span));
        assert_eq!(
            block_label("Researcher", scan, s.now),
            "Researcher: Market scan – Done, took 10m"
        );
        assert_eq!(lane_counts(&s.tasks), (2, 1));
        assert_eq!(lanes_said(&s.tasks), "1 task needs you");
        assert_eq!(lane_tag("ops", &s.tasks), Some("Needs you"));
        assert_eq!(lane_tag("review", &s.tasks), Some("Idle"));
        assert_eq!(lane_duration(65 * 60_000), "1h 5m");
    }

    #[test]
    fn the_draft_hands_over_to_review() {
        let s = lanes_at(8_000);
        let draft = s.tasks.iter().find(|t| t.id == "draft").unwrap();
        assert_eq!(draft.status, LaneStatus::Done);
        assert_eq!(draft.handoff_to, "copy");
        assert_eq!(lane_tag("review", &s.tasks), None);
        let late = lanes_at(23_000);
        let pr = late.tasks.iter().find(|t| t.id == "pr").unwrap();
        assert!(block_span(pr, late.now, late.span).is_none());
    }
}
