//! affectUI fidelity pass (plan #32, overnight batch, family B part 2): the model behind the exact
//! **Agent Crew** – a lead and its sub-agents as one live tree of cards, handoffs that run along
//! the connectors, a details card with steering, and a project switcher. Behaviour only; time is
//! passed in as elapsed milliseconds.

/// What an agent in the crew is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrewState {
    /// Working: its pixel mark twinkles and its line shimmers.
    Active,
    /// Resting.
    Idle,
    /// Paused by you.
    Paused,
    /// Needs you (amber).
    Waiting,
    /// Failed (red).
    Failed,
    /// Finished (green).
    Done,
}

impl CrewState {
    /// "Working" · "Idle" · "Paused" · "Needs you" · "Failed" · "Done".
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Working",
            Self::Idle => "Idle",
            Self::Paused => "Paused",
            Self::Waiting => "Needs you",
            Self::Failed => "Failed",
            Self::Done => "Done",
        }
    }
    /// `active` · `idle` · `paused` · `waiting` · `failed` · `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Paused => "paused",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
            Self::Done => "done",
        }
    }
    /// The steering actions its details offer, in order ("message" always first).
    pub fn steering(self) -> &'static [&'static str] {
        match self {
            Self::Active => &["message", "pause", "stop"],
            Self::Paused => &["message", "resume", "stop"],
            Self::Waiting => &["message", "decline", "approve"],
            Self::Failed => &["message", "retry"],
            Self::Idle | Self::Done => &["message"],
        }
    }
}

/// The label of a steering action ("Message", "Pause", …).
pub fn steer_label(action: &str) -> &'static str {
    match action {
        "message" => "Message",
        "pause" => "Pause",
        "resume" => "Resume",
        "stop" => "Stop",
        "approve" => "Approve",
        "decline" => "Decline",
        "retry" => "Retry",
        _ => "",
    }
}

/// What a steering action turns an agent into (state, activity line); `None` for "message".
pub fn steer_result(action: &str, before: &str) -> Option<(CrewState, String)> {
    Some(match action {
        "pause" => (CrewState::Paused, "Paused".into()),
        "resume" => (CrewState::Active, before.to_string()),
        "stop" => (CrewState::Idle, "Stopped".into()),
        "approve" => (CrewState::Active, "Approved – carrying on".into()),
        "decline" => (CrewState::Idle, "Declined".into()),
        "retry" => (CrewState::Active, "Retrying".into()),
        _ => return None,
    })
}

/// The past tense the live region reads for a steering action ("Paused Coder").
pub fn steer_said(action: &str, name: &str) -> String {
    let verb = match action {
        "pause" => "Paused",
        "resume" => "Resumed",
        "stop" => "Stopped",
        "approve" => "Approved",
        "decline" => "Declined",
        "retry" => "Retrying",
        _ => "Messaged",
    };
    format!("{verb} {name}")
}

/// One agent: a card in the tree.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewCard {
    /// Stable id.
    pub id: String,
    /// Name ("Coder").
    pub name: String,
    /// Model under the name ("Claude Sonnet"); empty for helpers that show none.
    pub model: String,
    /// Brand of the model's maker, for the details tag ("claude").
    pub model_brand: String,
    /// State.
    pub state: CrewState,
    /// One line: what it does now, or what it did last.
    pub activity: String,
    /// How full its context is, 0–1 (anything else is clamped).
    pub context: f32,
    /// Context size in tokens.
    pub limit: u32,
    /// Tools as (brand, name).
    pub tools: Vec<(String, String)>,
    /// The last thing it was handed: (from, text).
    pub last: Option<(String, String)>,
    /// What it was asked.
    pub task: String,
    /// When it started, ms into the run.
    pub started_ms: u64,
    /// When it ended, ms into the run.
    pub ended_ms: Option<u64>,
    /// What it has cost so far.
    pub cost: f64,
    /// Its sub-agents.
    pub children: Vec<CrewCard>,
}

impl CrewCard {
    /// A resting agent with no tools.
    pub fn new(id: &str, name: &str, model: &str, limit: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            model: model.into(),
            model_brand: brand_of_model(model).into(),
            state: CrewState::Idle,
            activity: "Idle".into(),
            context: 0.0,
            limit,
            tools: vec![],
            last: None,
            task: String::new(),
            started_ms: 0,
            ended_ms: None,
            cost: 0.0,
            children: vec![],
        }
    }
    /// Every agent in the tree, the lead first, depth-first: (depth, index of the parent, agent).
    pub fn flatten(&self) -> Vec<(usize, Option<usize>, &CrewCard)> {
        fn walk<'a>(
            c: &'a CrewCard,
            depth: usize,
            parent: Option<usize>,
            out: &mut Vec<(usize, Option<usize>, &'a CrewCard)>,
        ) {
            let me = out.len();
            out.push((depth, parent, c));
            for k in &c.children {
                walk(k, depth + 1, Some(me), out);
            }
        }
        let mut out = vec![];
        walk(self, 0, None, &mut out);
        out
    }
    /// The agent with this id.
    pub fn find_mut(&mut self, id: &str) -> Option<&mut CrewCard> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }
    /// (working, needs you, all) across the tree.
    pub fn counts(&self) -> (usize, usize, usize) {
        let f = self.flatten();
        (
            f.iter().filter(|x| x.2.state == CrewState::Active).count(),
            f.iter().filter(|x| x.2.state == CrewState::Waiting).count(),
            f.len(),
        )
    }
    /// Tokens in use.
    pub fn tokens(&self) -> u32 {
        (clamp_context(self.context) * self.limit as f32).round() as u32
    }
}

/// The maker's brand for a model name.
pub fn brand_of_model(model: &str) -> &'static str {
    let m = model.to_ascii_lowercase();
    if m.contains("claude") || m.contains("opus") || m.contains("sonnet") || m.contains("haiku") {
        "claude"
    } else if m.contains("gpt") || m.starts_with('o') {
        "openai"
    } else if m.contains("gemini") {
        "gemini"
    } else {
        ""
    }
}

/// A context value kept in 0–1 (NaN reads 0).
pub fn clamp_context(c: f32) -> f32 {
    if c.is_nan() { 0.0 } else { c.clamp(0.0, 1.0) }
}

/// `ok` below 80 %, `warn` from 80 %, `hot` from 95 %.
pub fn context_tone(c: f32) -> &'static str {
    let c = clamp_context(c);
    if c >= 0.95 {
        "hot"
    } else if c >= 0.8 {
        "warn"
    } else {
        "ok"
    }
}

/// "640", "15k", "136k", "1.2M".
pub fn short_tokens(n: u32) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{}k", (n as f64 / 1000.0).round() as u32)
    } else {
        let m = n as f64 / 1_000_000.0;
        if (m - m.round()).abs() < 0.05 {
            format!("{}M", m.round() as u32)
        } else {
            format!("{m:.1}M")
        }
    }
}

/// "136k / 200k".
pub fn tokens_label(c: &CrewCard) -> String {
    format!("{} / {}", short_tokens(c.tokens()), short_tokens(c.limit))
}

/// "25s", "3m 4s", "1h 2m", "3d 0h".
pub fn crew_duration(ms: u64) -> String {
    let s = ms / 1000;
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {}s", s / 60, s % 60)
    } else if s < 86_400 {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}d {}h", s / 86_400, (s % 86_400) / 3600)
    }
}

/// "$0.52", "<$0.01"; empty for NaN.
pub fn crew_cost(v: f64) -> String {
    if v.is_nan() {
        String::new()
    } else if v > 0.0 && v < 0.01 {
        "<$0.01".into()
    } else {
        format!("${v:.2}")
    }
}

/// How long an agent has run at `elapsed` (counting while it works).
pub fn running_ms(c: &CrewCard, elapsed: u64) -> u64 {
    let end = c.ended_ms.unwrap_or(elapsed).min(elapsed);
    end.saturating_sub(c.started_ms)
}

/// A handoff: a light from one agent to another along the connectors.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewHandoff {
    /// Stable id – it runs once per id.
    pub id: String,
    /// From (agent id).
    pub from: String,
    /// To (agent id).
    pub to: String,
    /// What was handed.
    pub label: String,
    /// When, ms into the run.
    pub at_ms: u64,
}

/// The hops of a handoff through the tree: (index of the agent whose connector to its parent the
/// light runs along, true when it runs up towards the parent). Indices are into
/// [`CrewCard::flatten`].
pub fn handoff_hops(root: &CrewCard, from: &str, to: &str) -> Vec<(usize, bool)> {
    let flat = root.flatten();
    let idx = |id: &str| flat.iter().position(|x| x.2.id == id);
    let (Some(a), Some(b)) = (idx(from), idx(to)) else {
        return vec![];
    };
    let chain = |mut i: usize| {
        let mut v = vec![i];
        while let Some(p) = flat[i].1 {
            v.push(p);
            i = p;
        }
        v
    };
    let (ca, cb) = (chain(a), chain(b));
    let lca = ca.iter().copied().find(|x| cb.contains(x)).unwrap_or(0);
    let mut hops: Vec<(usize, bool)> = ca
        .iter()
        .copied()
        .take_while(|&x| x != lca)
        .map(|x| (x, true))
        .collect();
    let mut down: Vec<(usize, bool)> = cb
        .iter()
        .copied()
        .take_while(|&x| x != lca)
        .map(|x| (x, false))
        .collect();
    down.reverse();
    hops.extend(down);
    hops
}

/// "Lead handed to Coder: Handle empty baskets".
pub fn handoff_said(root: &CrewCard, h: &CrewHandoff) -> String {
    let flat = root.flatten();
    let name = |id: &str| {
        flat.iter()
            .find(|x| x.2.id == id)
            .map(|x| x.2.name.clone())
            .unwrap_or_else(|| id.to_string())
    };
    format!("{} handed to {}: {}", name(&h.from), name(&h.to), h.label)
}

/// A project in the switcher.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewProject {
    /// Id.
    pub id: String,
    /// Name, the header's title.
    pub name: String,
}

/// The demo projects.
pub fn crew_projects() -> Vec<CrewProject> {
    [
        ("checkout", "Stop the payment page crash"),
        ("launch", "Draft the launch article"),
        ("postgres", "Move to Postgres 17"),
    ]
    .iter()
    .map(|(i, n)| CrewProject {
        id: i.to_string(),
        name: n.to_string(),
    })
    .collect()
}

/// One change in a scripted beat: (agent, state, activity, context, last handed).
type Change = (
    &'static str,
    CrewState,
    &'static str,
    f32,
    Option<(&'static str, &'static str)>,
);
/// A scripted beat: (ms, changes, handoffs as (from, to, label)).
type Beat = (
    u64,
    Vec<Change>,
    Vec<(&'static str, &'static str, &'static str)>,
);

fn tools(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter()
        .map(|(b, n)| (b.to_string(), n.to_string()))
        .collect()
}

fn base(project: &str) -> CrewCard {
    let mk = |id: &str, name: &str, model: &str, limit: u32, task: &str, t: &[(&str, &str)]| {
        let mut c = CrewCard::new(id, name, model, limit);
        c.task = task.into();
        c.tools = tools(t);
        c
    };
    match project {
        "launch" => {
            let mut lead = mk(
                "editor",
                "Editor",
                "Claude Opus",
                200_000,
                "Draft the launch article for Atlas 4 and get it fact-checked",
                &[("notion", "Notion"), ("slack", "Slack")],
            );
            let res = mk(
                "l-research",
                "Researcher",
                "Claude Sonnet",
                200_000,
                "List what ships in 2.0 from the merged pull requests",
                &[("github", "GitHub"), ("linear", "Linear")],
            );
            let mut writer = mk(
                "writer",
                "Writer",
                "Claude Sonnet",
                200_000,
                "Draft the post from the brief and the feature list",
                &[("notion", "Notion")],
            );
            writer.children.push(mk(
                "facts",
                "Fact checker",
                "",
                32_000,
                "Check every number in the draft",
                &[],
            ));
            let seo = mk(
                "seo",
                "SEO",
                "GPT-5 mini",
                64_000,
                "Title, slug and description for the post",
                &[],
            );
            lead.children = vec![res, writer, seo];
            lead
        }
        "postgres" => {
            let mut lead = mk(
                "pg-lead",
                "Lead",
                "Claude Opus",
                200_000,
                "Move the app database to Postgres 17 without downtime",
                &[("postgresql", "Postgres"), ("github", "GitHub")],
            );
            let planner = mk(
                "planner",
                "Planner",
                "Claude Sonnet",
                200_000,
                "Order the 42 migrations and find the risky ones",
                &[("postgresql", "Postgres")],
            );
            let mut migrator = mk(
                "migrator",
                "Migrator",
                "GPT-5",
                200_000,
                "Run the migrations on the staging copy",
                &[("postgresql", "Postgres"), ("docker", "Docker")],
            );
            migrator.children.push(mk(
                "verifier",
                "Verifier",
                "",
                64_000,
                "Compare row counts and checksums after each step",
                &[],
            ));
            lead.children = vec![planner, migrator];
            lead
        }
        _ => {
            let mut lead = mk(
                "lead",
                "Lead",
                "Claude Opus",
                200_000,
                "Stop the payment page crashing on empty baskets, get it reviewed and shipped",
                &[("linear", "Linear"), ("github", "GitHub")],
            );
            let mut res = mk(
                "research",
                "Researcher",
                "Claude Sonnet",
                200_000,
                "Find where empty baskets break the payment page",
                &[("linear", "Linear"), ("notion", "Notion")],
            );
            res.children.push(mk(
                "web",
                "Web search",
                "",
                64_000,
                "Look for matching crash reports",
                &[],
            ));
            let mut coder = mk(
                "coder",
                "Coder",
                "Claude Sonnet",
                200_000,
                "Handle empty baskets in basket.ts and cover them with tests",
                &[("github", "GitHub"), ("vercel", "Vercel")],
            );
            coder.children.push(mk(
                "lint",
                "Linter",
                "",
                32_000,
                "Lint the changed files",
                &[],
            ));
            coder.children.push(mk(
                "tests",
                "Test runner",
                "",
                64_000,
                "Run the payment suite",
                &[],
            ));
            let mut rev = mk(
                "review",
                "Reviewer",
                "Claude Opus",
                200_000,
                "Review PR #317 and the test run",
                &[("github", "GitHub")],
            );
            rev.children.push(mk(
                "docs",
                "Docs writer",
                "",
                128_000,
                "Write the release note",
                &[],
            ));
            lead.children = vec![res, coder, rev];
            lead
        }
    }
}

fn script(project: &str) -> Vec<Beat> {
    use CrewState::{Active as A, Done as D, Idle as I, Waiting as W};
    match project {
        "launch" => vec![
            (
                0,
                vec![
                    ("editor", A, "Briefing the writer", 0.18, None),
                    ("l-research", A, "Reading 23 pull requests", 0.31, None),
                    ("writer", I, "Waiting for work", 0.02, None),
                    ("facts", I, "Idle", 0.02, None),
                    ("seo", I, "Idle", 0.016, None),
                ],
                vec![],
            ),
            (
                4000,
                vec![
                    ("l-research", D, "Listed 6 features", 0.44, None),
                    (
                        "editor",
                        A,
                        "Briefing the writer",
                        0.24,
                        Some(("Researcher", "6 features, 14 fixes")),
                    ),
                ],
                vec![("l-research", "editor", "6 features, 14 fixes")],
            ),
            (
                7000,
                vec![
                    (
                        "writer",
                        A,
                        "Drafting the intro",
                        0.21,
                        Some(("Editor", "Short, calm, one screen")),
                    ),
                    ("editor", A, "Watching the draft", 0.26, None),
                ],
                vec![("editor", "writer", "Short, calm, one screen")],
            ),
            (
                11000,
                vec![
                    (
                        "facts",
                        A,
                        "Checking 4 numbers",
                        0.38,
                        Some(("Writer", "Draft v1")),
                    ),
                    ("writer", A, "Waiting on facts", 0.47, None),
                ],
                vec![("writer", "facts", "Draft v1")],
            ),
            (
                14000,
                vec![
                    ("facts", D, "1 number fixed", 0.52, None),
                    (
                        "writer",
                        D,
                        "Draft ready",
                        0.55,
                        Some(("Fact checker", "27%, not 30%")),
                    ),
                    (
                        "seo",
                        A,
                        "Writing the title",
                        0.2,
                        Some(("Editor", "Title and slug")),
                    ),
                ],
                vec![
                    ("facts", "writer", "27%, not 30%"),
                    ("editor", "seo", "Title and slug"),
                ],
            ),
            (
                17000,
                vec![
                    ("seo", D, "Title and slug ready", 0.31, None),
                    (
                        "editor",
                        W,
                        "Publish the post?",
                        0.33,
                        Some(("Writer", "Draft ready")),
                    ),
                ],
                vec![("writer", "editor", "Draft ready")],
            ),
        ],
        "postgres" => vec![
            (
                0,
                vec![
                    ("pg-lead", A, "Watching the migration", 0.21, None),
                    ("planner", D, "42 migrations in order", 0.48, None),
                    (
                        "migrator",
                        A,
                        "Running migration 17 of 42",
                        0.39,
                        Some(("Lead", "Start on staging")),
                    ),
                    ("verifier", I, "Idle", 0.05, None),
                ],
                vec![],
            ),
            (
                5000,
                vec![
                    (
                        "verifier",
                        A,
                        "Comparing checksums",
                        0.27,
                        Some(("Migrator", "Migrations 1–17")),
                    ),
                    ("migrator", A, "Running migration 24 of 42", 0.52, None),
                ],
                vec![("migrator", "verifier", "Migrations 1–17")],
            ),
            (
                10000,
                vec![
                    ("verifier", D, "All rows match", 0.41, None),
                    (
                        "migrator",
                        A,
                        "Running migration 38 of 42",
                        0.83,
                        Some(("Verifier", "All rows match")),
                    ),
                ],
                vec![("verifier", "migrator", "All rows match")],
            ),
            (
                15000,
                vec![
                    ("migrator", D, "42 of 42 on staging", 0.88, None),
                    (
                        "pg-lead",
                        W,
                        "Switch production over?",
                        0.29,
                        Some(("Migrator", "Staging is on 17")),
                    ),
                ],
                vec![("migrator", "pg-lead", "Staging is on 17")],
            ),
        ],
        _ => vec![
            (
                0,
                vec![
                    ("lead", A, "Delegating", 0.22, None),
                    (
                        "research",
                        A,
                        "Tracing basket.ts",
                        0.31,
                        Some(("Lead", "Find where empty baskets break")),
                    ),
                    (
                        "web",
                        A,
                        "Searching docs",
                        0.22,
                        Some(("Researcher", "Empty basket crash reports")),
                    ),
                    ("coder", I, "Waiting for work", 0.06, None),
                    ("lint", I, "Idle", 0.03, None),
                    ("tests", I, "Idle", 0.047, None),
                    ("review", I, "Waiting for work", 0.03, None),
                    ("docs", I, "Idle", 0.047, None),
                ],
                vec![],
            ),
            (
                3000,
                vec![("web", D, "Found 3 matches", 0.41, None)],
                vec![("web", "research", "3 matching reports")],
            ),
            (
                4000,
                vec![(
                    "research",
                    A,
                    "Reading 3 reports",
                    0.42,
                    Some(("Web search", "3 matching reports")),
                )],
                vec![],
            ),
            (
                6500,
                vec![
                    ("research", D, "Found the cause", 0.52, None),
                    (
                        "lead",
                        A,
                        "Reading the root cause",
                        0.26,
                        Some(("Researcher", "Root cause: basket.ts:87")),
                    ),
                ],
                vec![("research", "lead", "Root cause: basket.ts:87")],
            ),
            (
                9500,
                vec![
                    ("lead", A, "Following the fix", 0.29, None),
                    (
                        "coder",
                        A,
                        "Editing basket.ts",
                        0.38,
                        Some(("Lead", "Handle empty baskets, cover with tests")),
                    ),
                ],
                vec![("lead", "coder", "Handle empty baskets, cover with tests")],
            ),
            (
                12500,
                vec![
                    ("coder", A, "Prepare a branch", 0.52, None),
                    (
                        "lint",
                        A,
                        "Linting basket.ts",
                        0.3,
                        Some(("Coder", "Lint basket.ts")),
                    ),
                ],
                vec![("coder", "lint", "Lint basket.ts")],
            ),
            (14500, vec![("lint", D, "All clean", 0.47, None)], vec![]),
            (
                16500,
                vec![
                    (
                        "coder",
                        A,
                        "Waiting on tests",
                        0.68,
                        Some(("Linter", "All clean")),
                    ),
                    (
                        "tests",
                        A,
                        "182/214 passing",
                        0.55,
                        Some(("Coder", "Run the payment suite")),
                    ),
                ],
                vec![("coder", "tests", "Run the payment suite")],
            ),
            (
                19500,
                vec![
                    ("tests", D, "214/214 pass", 0.73, None),
                    (
                        "coder",
                        A,
                        "Opening PR #317",
                        0.86,
                        Some(("Test runner", "214/214 passing")),
                    ),
                ],
                vec![("tests", "coder", "214/214 passing")],
            ),
            (
                22500,
                vec![
                    ("coder", D, "Opened PR #317", 0.86, None),
                    (
                        "review",
                        A,
                        "Checking the tests",
                        0.2,
                        Some(("Coder", "Review PR #317")),
                    ),
                ],
                vec![("coder", "review", "Review PR #317")],
            ),
            (
                25500,
                vec![
                    ("review", D, "Approved PR #317", 0.47, None),
                    (
                        "docs",
                        A,
                        "Writing the release note",
                        0.05,
                        Some(("Reviewer", "Write the release note")),
                    ),
                    (
                        "lead",
                        W,
                        "Ship PR #317?",
                        0.33,
                        Some(("Reviewer", "Looks good – ready to ship")),
                    ),
                ],
                vec![
                    ("review", "docs", "Write the release note"),
                    ("review", "lead", "Looks good – ready to ship"),
                ],
            ),
            (28500, vec![("docs", D, "Note written", 0.35, None)], vec![]),
        ],
    }
}

/// How long one pass of a demo project lasts before it starts over (ms).
pub fn crew_loop_ms(project: &str) -> u64 {
    script(project).last().map(|b| b.0).unwrap_or(0) + 8000
}

/// A demo project's crew `elapsed` ms into a pass, and every handoff so far.
pub fn crew_at(project: &str, elapsed: u64) -> (CrewCard, Vec<CrewHandoff>) {
    let mut root = base(project);
    let mut handoffs = vec![];
    let mut n = 0;
    for (at, changes, hs) in script(project) {
        if at > elapsed {
            break;
        }
        for (id, st, act, ctx, last) in changes {
            if let Some(c) = root.find_mut(id) {
                if c.state != CrewState::Active && st == CrewState::Active {
                    c.started_ms = at;
                    c.ended_ms = None;
                }
                if matches!(st, CrewState::Done | CrewState::Idle) && c.state == CrewState::Active {
                    c.ended_ms = Some(at);
                }
                c.state = st;
                c.activity = act.into();
                c.context = ctx;
                if let Some((f, t)) = last {
                    c.last = Some((f.into(), t.into()));
                }
            }
        }
        for (f, t, l) in hs {
            n += 1;
            handoffs.push(CrewHandoff {
                id: format!("{project}-h{n}"),
                from: f.into(),
                to: t.into(),
                label: l.into(),
                at_ms: at,
            });
        }
    }
    price(&mut root, elapsed);
    (root, handoffs)
}

/// Cost so far from the context used and the time worked.
fn price(c: &mut CrewCard, elapsed: u64) {
    let ran = running_ms(c, elapsed) as f64 / 1000.0;
    let rate = if c.model.contains("Opus") {
        0.038
    } else if c.model.is_empty() {
        0.004
    } else {
        0.016
    };
    c.cost = (clamp_context(c.context) as f64 * c.limit as f64 / 1000.0) * 0.012 + ran * rate;
    for k in &mut c.children {
        price(k, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_and_tone() {
        assert_eq!(short_tokens(640), "640");
        assert_eq!(short_tokens(136_000), "136k");
        assert_eq!(context_tone(0.86), "warn");
        assert_eq!(context_tone(0.97), "hot");
        assert_eq!(context_tone(f32::NAN), "ok");
        assert_eq!(crew_cost(0.004), "<$0.01");
        assert_eq!(crew_duration(3 * 86_400_000), "3d 0h");
    }

    #[test]
    fn hops_go_up_then_down() {
        let (root, _) = crew_at("checkout", 0);
        let flat = root.flatten();
        let i = |id: &str| flat.iter().position(|x| x.2.id == id).unwrap();
        assert_eq!(
            handoff_hops(&root, "lead", "tests"),
            vec![(i("coder"), false), (i("tests"), false)]
        );
        assert_eq!(
            handoff_hops(&root, "research", "lead"),
            vec![(i("research"), true)]
        );
        assert_eq!(
            handoff_hops(&root, "web", "coder"),
            vec![(i("web"), true), (i("research"), true), (i("coder"), false)]
        );
    }

    #[test]
    fn the_script_ends_waiting_for_you() {
        let (root, hs) = crew_at("checkout", 30_000);
        assert_eq!(root.state, CrewState::Waiting);
        assert_eq!(root.counts(), (0, 1, 8));
        assert!(hs.len() >= 9);
        assert_eq!(
            handoff_said(&root, &hs[0]),
            "Web search handed to Researcher: 3 matching reports"
        );
    }
}
