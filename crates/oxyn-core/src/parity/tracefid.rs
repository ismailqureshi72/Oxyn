//! affectUI fidelity pass, family B part 9 (plan #32): the **Trace Waterfall** model. An agent run
//! as a tree of spans (agents, sub-agents, model calls, tool calls) with start and end times in
//! milliseconds from the start of the run. At any moment a span has not started, is running
//! (drawn to "now") or has ended, done or failed; the run's totals (duration, cost, tokens, call
//! counts, errors) are read at that moment, and the time axis grows to the next round range.
//! Behaviour only: the views live in `oxyn-dioxus` / `oxyn-gpui` (`parity/tracefid.rs`).

/// What a span is (picks its tint).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwKind {
    /// The top agent.
    Agent,
    /// A delegated agent.
    Subagent,
    /// A model call.
    Llm,
    /// A tool call.
    Tool,
}

impl TwKind {
    /// `agent` / `subagent` / `llm` / `tool`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Subagent => "subagent",
            Self::Llm => "llm",
            Self::Tool => "tool",
        }
    }
    /// "Model call" etc.
    pub fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Subagent => "Sub-agent",
            Self::Llm => "Model call",
            Self::Tool => "Tool call",
        }
    }
}

/// Where a span stands at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwStatus {
    /// Not started yet (hidden).
    Pending,
    /// Working.
    Running,
    /// Ended well.
    Done,
    /// Ended with an error.
    Failed,
}

impl TwStatus {
    /// `pending` / `running` / `done` / `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// One span of the trace.
#[derive(Clone, Debug, PartialEq)]
pub struct TwSpan {
    /// Stable id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Kind.
    pub kind: TwKind,
    /// Model or tool name (muted after the name).
    pub detail: String,
    /// Start, ms from the run start.
    pub start: u32,
    /// End, ms.
    pub end: u32,
    /// Ends in an error.
    pub failed: bool,
    /// Tokens in.
    pub tokens_in: u32,
    /// Tokens out.
    pub tokens_out: u32,
    /// Cost, dollars.
    pub cost: f64,
    /// What went in (shown in the details).
    pub input: String,
    /// What came out, or the error text when failed.
    pub output: String,
    /// Children.
    pub children: Vec<TwSpan>,
}

impl TwSpan {
    /// Status at `now` ms.
    pub fn status(&self, now: u32) -> TwStatus {
        if now < self.start {
            TwStatus::Pending
        } else if now < self.end {
            TwStatus::Running
        } else if self.failed {
            TwStatus::Failed
        } else {
            TwStatus::Done
        }
    }
    /// The end as drawn at `now`.
    pub fn shown_end(&self, now: u32) -> u32 {
        self.end.min(now.max(self.start))
    }
    /// Cost so far: a leaf's own pro rata, a branch's children's sum.
    pub fn cost_at(&self, now: u32) -> f64 {
        if self.children.is_empty() {
            self.cost * self.share(now)
        } else {
            self.children.iter().map(|c| c.cost_at(now)).sum()
        }
    }
    /// Tokens so far (in + out), as for cost.
    pub fn tokens_at(&self, now: u32) -> f64 {
        if self.children.is_empty() {
            (self.tokens_in + self.tokens_out) as f64 * self.share(now)
        } else {
            self.children.iter().map(|c| c.tokens_at(now)).sum()
        }
    }
    fn share(&self, now: u32) -> f64 {
        match self.status(now) {
            TwStatus::Pending => 0.0,
            TwStatus::Running => (now - self.start) as f64 / (self.end - self.start).max(1) as f64,
            _ => 1.0,
        }
    }
    /// Every span in the tree, depth first.
    pub fn walk(&self) -> Vec<&TwSpan> {
        let mut v = vec![self];
        for c in &self.children {
            v.extend(c.walk());
        }
        v
    }
}

/// One visible row.
#[derive(Clone, Debug, PartialEq)]
pub struct TwRow {
    /// The span id.
    pub id: String,
    /// Nesting depth (root 0).
    pub depth: u32,
    /// Has children.
    pub branch: bool,
    /// Folded (children hidden).
    pub folded: bool,
    /// Descendants hidden by the fold that have started (drawn as marks): their start times.
    pub marks: Vec<u32>,
    /// How many started descendants are hidden.
    pub hidden: usize,
}

/// The rows shown at `now`: started spans, depth first, skipping children of folded branches.
pub fn tw_rows(root: &TwSpan, now: u32, folded: &[String]) -> Vec<TwRow> {
    fn go(s: &TwSpan, d: u32, now: u32, folded: &[String], out: &mut Vec<TwRow>) {
        if s.start > now {
            return;
        }
        let f = !s.children.is_empty() && folded.contains(&s.id);
        let hid: Vec<u32> = if f {
            s.walk()
                .into_iter()
                .skip(1)
                .filter(|x| x.children.is_empty() && x.start <= now)
                .map(|x| x.start)
                .collect()
        } else {
            vec![]
        };
        out.push(TwRow {
            id: s.id.clone(),
            depth: d,
            branch: !s.children.is_empty(),
            folded: f,
            hidden: hid.len(),
            marks: hid,
        });
        if !f {
            for c in &s.children {
                go(c, d + 1, now, folded, out);
            }
        }
    }
    let mut out = vec![];
    go(root, 0, now, folded, &mut out);
    out
}

/// Find a span by id.
pub fn tw_find<'a>(root: &'a TwSpan, id: &str) -> Option<&'a TwSpan> {
    root.walk().into_iter().find(|s| s.id == id)
}

/// The run's totals at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct TwStats {
    /// Elapsed, ms.
    pub duration: u32,
    /// Dollars so far.
    pub cost: f64,
    /// Tokens so far.
    pub tokens: f64,
    /// Model calls started.
    pub llm: usize,
    /// Tool calls started.
    pub tools: usize,
    /// Spans ended in an error.
    pub errors: usize,
    /// Every span has ended.
    pub done: bool,
}

/// Totals at `now`.
pub fn tw_stats(root: &TwSpan, now: u32) -> TwStats {
    let all = root.walk();
    TwStats {
        duration: root.shown_end(now),
        cost: root.cost_at(now),
        tokens: root.tokens_at(now),
        llm: all
            .iter()
            .filter(|s| s.kind == TwKind::Llm && s.start <= now)
            .count(),
        tools: all
            .iter()
            .filter(|s| s.kind == TwKind::Tool && s.start <= now)
            .count(),
        errors: all
            .iter()
            .filter(|s| s.status(now) == TwStatus::Failed)
            .count(),
        done: now >= root.end,
    }
}

/// The longest ended model or tool call at `now`.
pub fn tw_slowest(root: &TwSpan, now: u32) -> Option<String> {
    root.walk()
        .into_iter()
        .filter(|s| s.children.is_empty() && s.end <= now)
        .max_by_key(|s| (s.end - s.start, std::cmp::Reverse(s.start)))
        .map(|s| s.id.clone())
}

/// The axis range at `now`: the smallest round range (ms) that holds the elapsed time with a
/// little room, and its tick step.
pub fn tw_axis(elapsed: u32) -> (u32, u32) {
    const R: [(u32, u32); 9] = [
        (5_000, 1_000),
        (10_000, 2_500),
        (15_000, 5_000),
        (30_000, 10_000),
        (45_000, 15_000),
        (60_000, 15_000),
        (90_000, 30_000),
        (120_000, 30_000),
        (180_000, 60_000),
    ];
    let need = elapsed + elapsed / 12;
    R.iter()
        .copied()
        .find(|r| r.0 >= need)
        .unwrap_or((need.div_ceil(60_000) * 60_000, 60_000))
}

/// "6.9s" / "820ms" / "1m 05s".
pub fn tw_dur(ms: u32) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}m {:02}s", ms / 60_000, (ms % 60_000) / 1000)
    }
}

/// "3.2k" / "880".
pub fn tw_tok(n: f64) -> String {
    if n >= 1000.0 {
        format!("{:.1}k", n / 1000.0)
    } else {
        format!("{n:.0}")
    }
}

#[allow(clippy::too_many_arguments)]
fn sp(
    id: &str,
    name: &str,
    kind: TwKind,
    detail: &str,
    start: u32,
    end: u32,
    tin: u32,
    tout: u32,
    cost: f64,
    io: (&str, &str),
) -> TwSpan {
    TwSpan {
        id: id.into(),
        name: name.into(),
        kind,
        detail: detail.into(),
        start,
        end,
        failed: false,
        tokens_in: tin,
        tokens_out: tout,
        cost,
        input: io.0.into(),
        output: io.1.into(),
        children: vec![],
    }
}

fn branch(id: &str, name: &str, kind: TwKind, detail: &str, kids: Vec<TwSpan>) -> TwSpan {
    let start = kids
        .iter()
        .map(|k| k.start)
        .min()
        .unwrap_or(0)
        .saturating_sub(300);
    let end = kids.iter().map(|k| k.end).max().unwrap_or(0) + 300;
    TwSpan {
        children: kids,
        ..sp(id, name, kind, detail, start, end, 0, 0, 0.0, ("", ""))
    }
}

/// The demo run: an orchestrator that plans, sends out a researcher, a writer and a coder (whose
/// first test run fails), then wraps up.
pub fn tw_demo() -> TwSpan {
    use TwKind::*;
    let mut tests1 = sp(
        "tests1",
        "Run banner tests",
        Tool,
        "GitHub",
        33_500,
        41_000,
        0,
        0,
        0.0,
        ("npm test -- banner", ""),
    );
    tests1.failed = true;
    tests1.output = "3 of 11 banner tests failed: the end date is read in UTC, so the sale closes a day early for viewers in the west.".into();
    let mut root = branch(
        "orch",
        "Orchestrator",
        Agent,
        "Sonnet 4.5",
        vec![
            sp(
                "plan",
                "Plan the work",
                Llm,
                "Sonnet 4.5",
                600,
                6_400,
                3_100,
                760,
                0.11,
                (
                    "Goal: a spring sale banner on the home page by Thursday.",
                    "Research rival offers, write the copy, build the banner, test it.",
                ),
            ),
            branch(
                "research",
                "Researcher",
                Subagent,
                "Haiku 4.5",
                vec![
                    sp(
                        "search",
                        "Search for spring offers",
                        Tool,
                        "Web search",
                        7_000,
                        10_100,
                        0,
                        0,
                        0.01,
                        ("spring sale banner retail 2026", "12 results"),
                    ),
                    sp(
                        "read",
                        "Read four rival pages",
                        Tool,
                        "Browser",
                        10_300,
                        17_900,
                        0,
                        0,
                        0.0,
                        ("4 links", "Discounts of 15–30 %, most end on a Sunday."),
                    ),
                    sp(
                        "sum",
                        "Sum up the offers",
                        Llm,
                        "Haiku 4.5",
                        18_100,
                        23_500,
                        6_400,
                        540,
                        0.02,
                        (
                            "Four pages of offers.",
                            "Go with 20 % off, ending Sunday night.",
                        ),
                    ),
                ],
            ),
            branch(
                "writer",
                "Writer",
                Subagent,
                "Sonnet 4.5",
                vec![
                    sp(
                        "draft",
                        "Draft the banner copy",
                        Llm,
                        "Sonnet 4.5",
                        24_300,
                        30_800,
                        2_200,
                        410,
                        0.07,
                        (
                            "Offer: 20 % off until Sunday.",
                            "Spring into savings – 20 % off everything until Sunday.",
                        ),
                    ),
                    sp(
                        "tone",
                        "Check the tone",
                        Llm,
                        "Haiku 4.5",
                        31_000,
                        34_600,
                        900,
                        120,
                        0.01,
                        (
                            "The draft and the style guide.",
                            "Fits the guide; shorten the second line.",
                        ),
                    ),
                ],
            ),
            branch(
                "coder",
                "Coder",
                Subagent,
                "Opus 4.1",
                vec![
                    sp(
                        "edit",
                        "Edit the banner component",
                        Llm,
                        "Opus 4.1",
                        24_600,
                        33_200,
                        8_800,
                        2_300,
                        0.29,
                        (
                            "Banner.tsx and the copy.",
                            "A dismissible banner with a countdown.",
                        ),
                    ),
                    tests1,
                    sp(
                        "fix",
                        "Fix the date check",
                        Llm,
                        "Opus 4.1",
                        41_200,
                        46_100,
                        5_100,
                        690,
                        0.13,
                        ("The failing tests.", "Compare dates in the viewer's zone."),
                    ),
                    sp(
                        "tests2",
                        "Run banner tests",
                        Tool,
                        "GitHub",
                        46_300,
                        49_600,
                        0,
                        0,
                        0.0,
                        ("npm test -- banner", "11 of 11 passed"),
                    ),
                ],
            ),
            sp(
                "wrap",
                "Write the hand-off note",
                Llm,
                "Sonnet 4.5",
                49_900,
                52_000,
                1_400,
                300,
                0.04,
                (
                    "Everything above.",
                    "Banner ready for review; one date bug found and fixed.",
                ),
            ),
        ],
    );
    root.start = 0;
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_rows_stats() {
        let r = tw_demo();
        assert_eq!((r.start, r.end), (0, 52_300));
        // Early: only the root and the plan call.
        let rows = tw_rows(&r, 3_000, &[]);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            tw_find(&r, "plan").unwrap().status(3_000),
            TwStatus::Running
        );
        // Mid-run, coder folded: its started leaves become marks.
        let rows = tw_rows(&r, 36_000, &["coder".into()]);
        let c = rows.iter().find(|x| x.id == "coder").unwrap();
        assert!(c.folded);
        assert_eq!(c.marks, vec![24_600, 33_500]);
        assert!(rows.iter().all(|x| x.id != "edit"));
        let s = tw_stats(&r, 36_000);
        assert_eq!(s.errors, 0);
        assert!(!s.done);
        let s = tw_stats(&r, 60_000);
        assert!(s.done);
        assert_eq!((s.llm, s.tools, s.errors), (7, 4, 1));
        assert!((s.cost - 0.68).abs() < 1e-9, "{}", s.cost);
        assert_eq!(s.duration, 52_300);
        assert_eq!(tw_slowest(&r, 60_000).as_deref(), Some("edit"));
        assert_eq!(tw_axis(12_000), (15_000, 5_000));
        assert_eq!(tw_axis(52_300), (60_000, 15_000));
        assert_eq!(tw_dur(6_900), "6.9s");
        assert_eq!(tw_dur(820), "820ms");
        assert_eq!(tw_tok(3_200.0), "3.2k");
        assert_eq!(tw_rows(&r, 60_000, &[]).len(), 15);
    }
}
