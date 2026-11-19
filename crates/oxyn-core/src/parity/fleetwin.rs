//! Screens S11 (Agent apps III): **Fleet Window**, a desktop coding-agent app in the
//! agents-window pattern. A sidebar of agents (each on its own worktree, with its model and
//! lines changed), a conversation per agent with its steps, a review pane of its changes, a mode
//! picker (Agent, Plan, Ask) and parallel runs: one prompt on up to four models side by side,
//! compared by tests and size, and one of them applied. Our own name and design, inspired by how
//! such apps behave. Every name, file and company is fictional.

pub use super::cmdstudio::{
    DiffFile, PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, TState,
    prompt_error, thread_diff,
};
use super::cmdstudio::{Hunk, Verdict};

/// The app's name.
pub const FW_APP: &str = "Fleet Window";

/// The state bar of Fleet Window.
pub const FW_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("compare", "Compare"),
    ("plan", "Plan"),
];

/// Split a state key into the data state and the own state (`compare`, `plan` or "").
pub fn fw_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "compare" => (ScreenState::Sample, "compare"),
        "plan" => (ScreenState::Sample, "plan"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Models an agent can run on: name and brand mark id.
pub const FW_MODELS: [(&str, &str); 4] = [
    ("Sonnet 5.5", "claude"),
    ("GPT-6", "openai"),
    ("Gemini 3 Pro", "gemini"),
    ("Grok 5", "grok"),
];

/// How the agent works on a prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentMode {
    /// Edits files and runs commands.
    Agent,
    /// Writes a plan to approve first.
    Plan,
    /// Answers without editing.
    Ask,
}

impl AgentMode {
    /// All modes, in picker order.
    pub const ALL: [AgentMode; 3] = [AgentMode::Agent, AgentMode::Plan, AgentMode::Ask];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Plan => "Plan",
            Self::Ask => "Ask",
        }
    }
    /// One-line hint.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Agent => "Edits files and runs commands",
            Self::Plan => "Writes a plan for you to approve",
            Self::Ask => "Answers, no edits",
        }
    }
}

/// An agent in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`FW_MODELS`].
    pub model: usize,
    /// Where it stands.
    pub state: TState,
    /// Lines added and removed.
    pub stat: (u32, u32),
    /// Last change.
    pub ago: String,
    /// Parallel group (0 for a single agent).
    pub group: u32,
}

/// The worktree branch an agent works on.
pub fn worktree_for(id: u32) -> String {
    format!("fw/agent-{id}")
}

fn ag(
    id: u32,
    title: &str,
    model: usize,
    state: TState,
    stat: (u32, u32),
    ago: &str,
    group: u32,
) -> Agent {
    Agent {
        id,
        title: title.to_string(),
        model,
        state,
        stat,
        ago: ago.to_string(),
        group,
    }
}

/// The sample agents (empty for loading and empty).
pub fn fw_agents(st: ScreenState) -> Vec<Agent> {
    use TState::*;
    if st != ScreenState::Sample {
        return Vec::new();
    }
    let mut v = vec![
        ag(1, "Retry uploads on 503", 0, Running, (0, 0), "now", 0),
        ag(2, "Paginate the invoices API", 1, Review, (48, 12), "4m", 7),
        ag(3, "Paginate the invoices API", 0, Review, (41, 9), "4m", 7),
        ag(4, "Paginate the invoices API", 2, Review, (63, 20), "5m", 7),
        ag(5, "Lazy-load the photo grid", 1, Done, (22, 6), "1h", 0),
        ag(6, "Export notes as Markdown", 2, Done, (35, 0), "3h", 0),
    ];
    for a in v.iter_mut().filter(|a| a.state != Running) {
        a.stat = super::cmdstudio::diff_stat(&agent_diff(a.id));
    }
    v
}

fn file(
    path: &'static str,
    new: bool,
    head: &'static str,
    lines: &[(char, &'static str)],
) -> DiffFile {
    DiffFile {
        path,
        new,
        hunks: vec![Hunk {
            head,
            lines: lines.to_vec(),
            verdict: Verdict::Open,
        }],
    }
}

/// The diff an agent shows (each sample agent has its own).
pub fn agent_diff(id: u32) -> Vec<DiffFile> {
    let route = |cursor: &'static str| {
        file(
            "src/api/invoices.rs",
            false,
            "@@ -18,7 +18,12 @@ async fn list_invoices",
            &[
                ('-', "    let rows = db.invoices().all().await?;"),
                ('+', "    let limit = q.limit.unwrap_or(50).min(50);"),
                ('+', cursor),
                (
                    '+',
                    "    let rows = db.invoices().after(after).take(limit + 1).await?;",
                ),
                (
                    '+',
                    "    let next = (rows.len() > limit).then(|| encode(&rows[limit - 1]));",
                ),
                (' ', "    Ok(Json(Page { items: rows, next }))"),
            ],
        )
    };
    match id {
        2 => vec![route(
            "    let after = q.cursor.map(|c| c.parse::<u64>()).transpose()?;",
        )],
        3 => vec![
            route("    let after = q.cursor.as_deref().map(decode).transpose()?;"),
            file(
                "src/api/cursor.rs",
                true,
                "@@ -0,0 +1,6 @@",
                &[
                    ('+', "/// Opaque cursor over (created_at, id)."),
                    ('+', "pub fn encode(r: &Invoice) -> String {"),
                    ('+', "    b64(format!(\"{}:{}\", r.created_at, r.id))"),
                    ('+', "}"),
                ],
            ),
        ],
        4 => vec![
            route("    let after = q.cursor.as_deref().map(Cursor::parse).transpose()?;"),
            file(
                "src/api/page.rs",
                true,
                "@@ -0,0 +1,6 @@",
                &[
                    ('+', "#[derive(Serialize)]"),
                    ('+', "pub struct Page<T> {"),
                    ('+', "    pub items: Vec<T>,"),
                    ('+', "    pub next: Option<String>,"),
                    ('+', "}"),
                ],
            ),
        ],
        5 | 6 => thread_diff(id),
        _ => Vec::new(),
    }
}

/// Start agents from a prompt: one per chosen model, sharing a group if two or more are chosen.
/// Returns the new ids (first one opens).
pub fn spawn(
    list: &mut Vec<Agent>,
    prompt: &str,
    models: &[usize],
) -> Result<Vec<u32>, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    if models.is_empty() {
        return Err("Pick at least one model");
    }
    let title = super::taskdeck::title_from(prompt);
    let base = list.iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let group = if models.len() > 1 {
        list.iter().map(|a| a.group).max().unwrap_or(0) + 1
    } else {
        0
    };
    let mut ids = Vec::new();
    for (k, m) in models.iter().enumerate() {
        let id = base + k as u32;
        list.insert(k, ag(id, &title, *m, TState::Running, (0, 0), "now", group));
        ids.push(id);
    }
    Ok(ids)
}

/// Toggle a model in a parallel selection (at least one stays, at most four).
pub fn toggle_model(sel: &mut Vec<usize>, m: usize) -> bool {
    if let Some(i) = sel.iter().position(|x| *x == m) {
        if sel.len() == 1 {
            return false;
        }
        sel.remove(i);
    } else {
        if sel.len() >= FW_MODELS.len() || m >= FW_MODELS.len() {
            return false;
        }
        sel.push(m);
        sel.sort_unstable();
    }
    true
}

// ─────────────────────────────────────────────────────── Compare ──

/// One model's attempt in a parallel run.
#[derive(Clone, Debug, PartialEq)]
pub struct Attempt {
    /// The agent behind it.
    pub agent: u32,
    /// Index into [`FW_MODELS`].
    pub model: usize,
    /// Seconds it took.
    pub secs: u32,
    /// Lines added and removed.
    pub stat: (u32, u32),
    /// Tests passed and run.
    pub tests: (u32, u32),
    /// What it did.
    pub summary: &'static str,
    /// None while open; Some(true) applied, Some(false) discarded.
    pub picked: Option<bool>,
}

/// The parallel run's prompt.
pub const COMPARE_PROMPT: &str =
    "Paginate GET /invoices with a cursor, 50 per page, and update the client and its tests.";

/// The sample parallel run (group 7).
pub fn sample_attempts() -> Vec<Attempt> {
    let mut v = vec![
        Attempt {
            agent: 3,
            model: 0,
            secs: 214,
            stat: (41, 9),
            tests: (18, 18),
            summary: "Opaque base64 cursor over (created_at, id); client follows next until null.",
            picked: None,
        },
        Attempt {
            agent: 2,
            model: 1,
            secs: 187,
            stat: (48, 12),
            tests: (17, 18),
            summary: "Cursor on id only; one ordering test fails when two invoices share a second.",
            picked: None,
        },
        Attempt {
            agent: 4,
            model: 2,
            secs: 262,
            stat: (63, 20),
            tests: (18, 18),
            summary: "Cursor plus a page-size cap and a new Page<T> type shared with the notes API.",
            picked: None,
        },
    ];
    for a in v.iter_mut() {
        a.stat = super::cmdstudio::diff_stat(&agent_diff(a.agent));
    }
    v
}

/// The suggested attempt: most tests passing, then fewest lines changed.
pub fn best_attempt(a: &[Attempt]) -> Option<usize> {
    (0..a.len()).max_by(|x, y| {
        let (ax, ay) = (&a[*x], &a[*y]);
        let rx = ax.tests.0 * 1000 / ax.tests.1.max(1);
        let ry = ay.tests.0 * 1000 / ay.tests.1.max(1);
        rx.cmp(&ry)
            .then((ay.stat.0 + ay.stat.1).cmp(&(ax.stat.0 + ax.stat.1)))
    })
}

/// Apply one attempt: it is kept and the others are discarded. False when already decided.
pub fn apply_attempt(a: &mut [Attempt], i: usize) -> bool {
    if i >= a.len() || a.iter().any(|x| x.picked == Some(true)) {
        return false;
    }
    for (k, x) in a.iter_mut().enumerate() {
        x.picked = Some(k == i);
    }
    true
}

/// Undo an apply: every attempt is open again.
pub fn reopen_attempts(a: &mut [Attempt]) {
    for x in a.iter_mut() {
        x.picked = None;
    }
}

/// "m:ss" for a duration.
pub fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

// ────────────────────────────────────────────────────────── Steps ──

/// The open agent's prompt.
pub const FW_PROMPT: &str = "Uploads fail for good when storage returns a 503. Retry with exponential backoff, at most 4 tries, and add a test.";

/// The open agent's steps: glyph, label, detail.
pub const FW_STEPS: [(&str, &str, &str); 5] = [
    ("search", "Searched for upload call sites", "6 results"),
    ("doc", "Read src/upload/client.rs", "212 lines"),
    ("pen", "Edited src/upload/client.rs", "+28 −6"),
    ("plus", "Added tests/retry.rs", "+41"),
    ("term", "Running cargo test retry", ""),
];

/// A plan step: text and done.
pub type PlanStep = (String, bool);

/// The sample plan (Plan mode).
pub fn sample_plan() -> Vec<PlanStep> {
    [
        "Add a RetryPolicy (4 tries, 250 ms base, doubling, ±20% jitter)",
        "Wrap put_object in the policy; retry 503 and timeouts only",
        "Surface the attempt count in the upload log line",
        "Test: a server that fails twice, then succeeds",
        "Test: a 400 is not retried",
    ]
    .iter()
    .map(|s| (s.to_string(), false))
    .collect()
}

/// Plan progress: done and total.
pub fn plan_progress(p: &[PlanStep]) -> (usize, usize) {
    (p.iter().filter(|s| s.1).count(), p.len())
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Fleet Window shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct FleetWindowData {
    /// Agents in the sidebar; the first one opens.
    pub agents: Vec<Agent>,
    /// The parallel run's attempts (Compare).
    pub attempts: Vec<Attempt>,
    /// Each agent's changes: (agent id, files).
    pub diffs: Vec<(u32, Vec<DiffFile>)>,
    /// The plan Plan mode shows.
    pub plan: Vec<PlanStep>,
}

impl FleetWindowData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        let agents = fw_agents(st);
        let samp = st == ScreenState::Sample;
        Self {
            diffs: agents.iter().map(|a| (a.id, agent_diff(a.id))).collect(),
            agents,
            attempts: if samp { sample_attempts() } else { Vec::new() },
            plan: if samp { sample_plan() } else { Vec::new() },
        }
    }
    /// An agent's changes (empty when it has none).
    pub fn diff(&self, id: u32) -> Vec<DiffFile> {
        self.diffs
            .iter()
            .find(|d| d.0 == id)
            .map(|d| d.1.clone())
            .unwrap_or_default()
    }
}

impl Default for FleetWindowData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s11_fleet_window() {
        assert_eq!(fw_split("compare").1, "compare");
        assert_eq!(fw_split("loading").0, ScreenState::Loading);
        let mut l = fw_agents(ScreenState::Sample);
        assert!(spawn(&mut l, "hi", &[0]).is_err());
        assert!(spawn(&mut l, "Add a health check endpoint", &[]).is_err());
        let ids = spawn(&mut l, "Add a health check endpoint", &[0, 1, 2]).unwrap();
        assert_eq!(ids.len(), 3);
        assert!(
            l[..3]
                .iter()
                .all(|a| a.group == 8 && a.state == TState::Running)
        );
        let mut sel = vec![0];
        assert!(!toggle_model(&mut sel, 0));
        assert!(toggle_model(&mut sel, 2));
        assert_eq!(sel, vec![0, 2]);
        let mut a = sample_attempts();
        assert_eq!(best_attempt(&a), Some(0));
        assert!(apply_attempt(&mut a, 2));
        assert!(!apply_attempt(&mut a, 0));
        assert_eq!(a[2].picked, Some(true));
        assert_eq!(a[0].picked, Some(false));
        reopen_attempts(&mut a);
        assert!(a.iter().all(|x| x.picked.is_none()));
        assert_eq!(mmss(214), "3:34");
        let d = FleetWindowData::default();
        assert!(d.diff(1).is_empty() && !d.diff(3).is_empty());
        assert_eq!(plan_progress(&d.plan), (0, 5));
        assert!(
            FleetWindowData::for_state(ScreenState::Empty)
                .agents
                .is_empty()
        );
    }
}
