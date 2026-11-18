//! Screens S4: agent ops II (Agents, Skills, Schedules, Usage, Permissions, Run Detail) on the
//! Screens 1 frame with the Fieldwork persona's sidebar (`workscr::persona("fieldwork")`). The
//! data is fictional sample data written for Oxyn; every view rule (tabs and counts, the pixel
//! mark drawn from a name, the hours strip, the spend forecast, permission counts per level, the
//! trace axis) is a pure function so Dioxus and GPUI draw the same thing.

pub use super::screens2fid::{eur, toks};
pub use super::screensfid::ScreenState;

/// A state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

/// Agents states.
pub const AGENTS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("creating", "New agent"),
];
/// Skills states.
pub const SKILLS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("installing", "Installing"),
];
/// Schedules states.
pub const SCHED_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("running", "Running now"),
];
/// Usage states.
pub const USAGE_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("over", "Over budget"),
];
/// Permissions states.
pub const PERM_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("handsoff", "Hands-off"),
];
/// Run Detail states.
pub const RUN_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("stopped", "Stopped"),
];

/// The states of a screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "agent-agents" => AGENTS_STATES,
        "agent-skills" => SKILLS_STATES,
        "agent-schedules" => SCHED_STATES,
        "agent-usage" => USAGE_STATES,
        "agent-permissions" => PERM_STATES,
        _ => RUN_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 6] = [
        "creating",
        "installing",
        "running",
        "over",
        "handsoff",
        "stopped",
    ];
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

/// A model: label and the brand id of its official mark.
pub type Model = (&'static str, &'static str);

/// The models agents can run on (fictional versions; official provider marks).
pub const MODELS: [Model; 5] = [
    ("Opus 5.5", "claude"),
    ("Sonnet 5.5", "claude"),
    ("GPT-6 mini", "openai"),
    ("Gemini 3.8 Pro", "gemini"),
    ("Grok 5", "xai"),
];

// ───────────────────────────────────────────────────────────── Agents ──

/// Identity hues a mark can take.
pub const HUES: [&str; 6] = ["blue", "purple", "pink", "teal", "orange", "green"];

fn fnv(s: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in s.trim().to_lowercase().bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// The pixel mark of a name: a 3 × 3 grid mirrored left to right, so a typed name draws the
/// same mark every time. Never empty: the centre is lit when nothing else is.
pub fn mark_bits(name: &str) -> [bool; 9] {
    if name.trim().is_empty() {
        return [false; 9];
    }
    let h = fnv(name);
    let mut b = [false; 9];
    for r in 0..3 {
        let l = h >> (r * 2) & 1 == 1;
        let m = h >> (r * 2 + 1) & 1 == 1;
        b[r * 3] = l;
        b[r * 3 + 1] = m;
        b[r * 3 + 2] = l;
    }
    if b.iter().filter(|x| **x).count() < 3 {
        b[4] = true;
        b[1] = true;
        b[7] = true;
    }
    b
}

/// The hue of a typed name.
pub fn mark_hue(name: &str) -> &'static str {
    HUES[(fnv(name) >> 8) as usize % HUES.len()]
}

/// Where an agent stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    /// On a task.
    Working,
    /// Waiting for work.
    Idle,
    /// Switched off.
    Paused,
}

impl AgentState {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Idle => "idle",
            Self::Paused => "paused",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::Idle => "Idle",
            Self::Paused => "Paused",
        }
    }
    /// Tone key (`run`, ``, `warn`).
    pub fn tone(self) -> &'static str {
        match self {
            Self::Working => "run",
            Self::Idle => "",
            Self::Paused => "warn",
        }
    }
}

/// An agent on the team.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Role in a few words.
    pub role: String,
    /// State.
    pub state: AgentState,
    /// Apps it may use (brand ids).
    pub apps: Vec<&'static str>,
    /// Model index into [`MODELS`].
    pub model: usize,
    /// Runs this month.
    pub runs: u32,
    /// Spend this month, cents.
    pub spend: u32,
    /// What it is doing now.
    pub now: &'static str,
    /// Its instructions.
    pub brief: &'static str,
    /// Skills it uses.
    pub skills: &'static [&'static str],
    /// Recent tasks: (task, age).
    pub recent: &'static [(&'static str, &'static str)],
    /// Runs per day over two weeks.
    pub days: [u8; 14],
}

/// The sample team (empty unless `sample`).
pub fn agents(sample: bool) -> Vec<Agent> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    let a = |id,
             name: &str,
             role: &str,
             state,
             apps: &[&'static str],
             model,
             runs,
             spend,
             now,
             days| Agent {
        id,
        name: name.to_string(),
        role: role.to_string(),
        state,
        apps: apps.to_vec(),
        model,
        runs,
        spend,
        now,
        brief: "You work in the Kestrel monorepo. Keep pull requests small, write tests first and never deploy to production without a person approving it. When a migration touches user data, run it on staging first and post the plan in the thread.",
        skills: &["Code review", "Release notes", "Spreadsheet analysis"],
        recent: &[
            ("Migrate users table", "now"),
            ("Fix flaky upload test", "2h"),
            ("Bump the queue client", "1d"),
        ],
        days,
    };
    vec![
        a(
            1,
            "Builder",
            "Writes and ships code",
            AgentState::Working,
            &["github", "linear", "slack"],
            0,
            38,
            4620,
            "Running the migration on staging",
            [2, 4, 2, 3, 1, 1, 4, 9, 2, 8, 8, 2, 3, 2],
        ),
        a(
            2,
            "Reviewer",
            "Reviews pull requests",
            AgentState::Working,
            &["github", "slack"],
            1,
            64,
            2210,
            "Reading pull request #491",
            [5, 4, 6, 5, 3, 2, 5, 6, 4, 5, 7, 6, 3, 3],
        ),
        a(
            3,
            "Writer",
            "Drafts posts, emails and notes",
            AgentState::Working,
            &["googledrive", "slack", "gmail"],
            1,
            27,
            1240,
            "Drafting the release email",
            [1, 2, 3, 2, 1, 0, 2, 3, 2, 3, 2, 1, 2, 3],
        ),
        a(
            4,
            "Ops",
            "Keeps data and tools tidy",
            AgentState::Idle,
            &["hubspot", "stripe", "github"],
            2,
            112,
            910,
            "Waiting for the nightly sync",
            [8, 9, 7, 8, 9, 6, 8, 9, 8, 7, 9, 8, 8, 6],
        ),
        a(
            5,
            "Researcher",
            "Reads widely, writes cited briefs",
            AgentState::Idle,
            &["notion", "googledrive"],
            3,
            9,
            3210,
            "Last brief sent this morning",
            [0, 1, 0, 1, 1, 0, 0, 1, 0, 2, 1, 0, 1, 1],
        ),
        a(
            6,
            "Support",
            "Answers customer questions",
            AgentState::Paused,
            &["gmail", "slack"],
            4,
            0,
            0,
            "Paused by you on Nov 12",
            [0; 14],
        ),
    ]
}

/// The tabs: (key, label).
pub const AGENT_TABS: [(&str, &str); 4] = [
    ("all", "All agents"),
    ("working", "Working"),
    ("idle", "Idle"),
    ("paused", "Paused"),
];

/// Counts per tab: all, working, idle, paused.
pub fn agent_counts(all: &[Agent]) -> [usize; 4] {
    let n = |s| all.iter().filter(|a| a.state == s).count();
    [
        all.len(),
        n(AgentState::Working),
        n(AgentState::Idle),
        n(AgentState::Paused),
    ]
}

/// The agents a tab shows; `by_spend` (Oxyn) sorts by spend, most first.
pub fn agent_view(all: &[Agent], tab: &str, by_spend: bool) -> Vec<Agent> {
    let mut v: Vec<Agent> = all
        .iter()
        .filter(|a| tab == "all" || a.state.key() == tab)
        .cloned()
        .collect();
    if by_spend {
        v.sort_by_key(|a| std::cmp::Reverse(a.spend));
    }
    v
}

/// "Access to 3 apps".
pub fn apps_words(n: usize) -> String {
    match n {
        0 => "No apps yet".to_string(),
        1 => "Access to 1 app".to_string(),
        _ => format!("Access to {n} apps"),
    }
}

/// Add a new agent from the New agent form; `None` for a blank name.
pub fn new_agent(all: &mut Vec<Agent>, name: &str, role: &str, model: usize) -> Option<u32> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let id = all.iter().map(|a| a.id).max().unwrap_or(0) + 1;
    let role = if role.trim().is_empty() {
        "New agent"
    } else {
        role.trim()
    };
    all.push(Agent {
        id,
        name: name.to_string(),
        role: role.to_string(),
        state: AgentState::Idle,
        apps: Vec::new(),
        model: model.min(MODELS.len() - 1),
        runs: 0,
        spend: 0,
        now: "Ready for its first task",
        brief: "Tell it what it is for, what it may touch and when to ask you first.",
        skills: &[],
        recent: &[],
        days: [0; 14],
    });
    Some(id)
}

/// The bar heights of a two-week chart as percentages of its tallest day (10 at least).
pub fn bars(days: &[u8]) -> Vec<u32> {
    let top = days.iter().copied().max().unwrap_or(0).max(10) as u32;
    days.iter().map(|d| u32::from(*d) * 100 / top).collect()
}

// ───────────────────────────────────────────────────────────── Skills ──

/// A skill an agent can use.
#[derive(Clone, Debug, PartialEq)]
pub struct Skill {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// One line.
    pub blurb: &'static str,
    /// Glyph.
    pub glyph: &'static str,
    /// Hue.
    pub hue: &'static str,
    /// Category key.
    pub cat: &'static str,
    /// Who made it: `Yours`, `Built-in` or a handle.
    pub by: &'static str,
    /// Installed.
    pub installed: bool,
    /// Switched on (installed only).
    pub on: bool,
    /// How many use it (Discover).
    pub uses: &'static str,
    /// Runs per day over two weeks.
    pub days: [u8; 14],
}

/// Skill categories: (key, label).
pub const SKILL_CATS: [(&str, &str); 6] = [
    ("all", "All"),
    ("writing", "Writing"),
    ("research", "Research"),
    ("eng", "Engineering"),
    ("data", "Data"),
    ("docs", "Documents"),
];

/// The sample skills (empty unless `sample`).
pub fn skills(sample: bool) -> Vec<Skill> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    let s = |id, name, blurb, glyph, hue, cat, by, installed, on, uses| Skill {
        id,
        name,
        blurb,
        glyph,
        hue,
        cat,
        by,
        installed,
        on,
        uses,
        days: [6, 2, 5, 6, 1, 1, 6, 2, 3, 5, 9, 9, 8, 5],
    };
    vec![
        s(
            1,
            "Brand voice",
            "Writes in Kestrel's tone – short, warm, no jargon",
            "pen",
            "pink",
            "writing",
            "Yours",
            true,
            true,
            "",
        ),
        s(
            2,
            "Weekly report",
            "Turns a week of work into a one-page update",
            "doc",
            "orange",
            "writing",
            "Built-in",
            true,
            true,
            "",
        ),
        s(
            3,
            "Meeting notes",
            "Notes and next steps from a call recording",
            "mic",
            "purple",
            "writing",
            "Built-in",
            true,
            false,
            "",
        ),
        s(
            4,
            "Deep research",
            "Reads widely, checks sources, writes a cited brief",
            "globe",
            "blue",
            "research",
            "Built-in",
            true,
            true,
            "",
        ),
        s(
            5,
            "Competitor watch",
            "Spots price, product and campaign changes on rivals' sites",
            "search",
            "purple",
            "research",
            "@ines",
            true,
            true,
            "",
        ),
        s(
            6,
            "Code review",
            "Reviews a pull request for bugs, tests and naming",
            "route",
            "green",
            "eng",
            "Built-in",
            true,
            true,
            "",
        ),
        s(
            7,
            "Release notes",
            "Writes release notes people outside engineering can read",
            "text",
            "orange",
            "eng",
            "Yours",
            true,
            true,
            "",
        ),
        s(
            8,
            "Spreadsheet analysis",
            "Cleans a sheet, finds the trend and says it in a sentence",
            "table",
            "green",
            "data",
            "Built-in",
            true,
            true,
            "",
        ),
        s(
            9,
            "Chart maker",
            "Picks the right chart for the numbers and draws it",
            "chart",
            "blue",
            "data",
            "@mika",
            true,
            true,
            "",
        ),
        s(
            10,
            "PDF forms",
            "Reads and fills in PDF forms from your documents",
            "pdf",
            "orange",
            "docs",
            "@sam",
            false,
            false,
            "8.4K",
        ),
        s(
            11,
            "Translate & localize",
            "Translates and adapts copy for each market",
            "globe",
            "teal",
            "writing",
            "Built-in",
            false,
            false,
            "12.9K",
        ),
        s(
            12,
            "Slide deck",
            "Turns a brief into a short, clean deck",
            "slides",
            "purple",
            "docs",
            "Built-in",
            false,
            false,
            "10.3K",
        ),
        s(
            13,
            "Invoice extraction",
            "Pulls amounts, dates and vendors out of invoices",
            "card",
            "orange",
            "data",
            "@jo",
            false,
            false,
            "3.4K",
        ),
        s(
            14,
            "Interview synthesis",
            "Finds the patterns across customer interviews",
            "people",
            "pink",
            "research",
            "@nora",
            false,
            false,
            "2K",
        ),
        s(
            15,
            "Notes to tasks",
            "Turns any notes into tasks with owners and dates",
            "tasks",
            "blue",
            "docs",
            "Built-in",
            false,
            false,
            "7.6K",
        ),
    ]
}

/// What a category shows: (installed, discover), each filtered by the search.
pub fn skill_view(all: &[Skill], cat: &str, q: &str) -> (Vec<Skill>, Vec<Skill>) {
    let q = q.trim().to_lowercase();
    let keep = |s: &&Skill| {
        (cat == "all" || s.cat == cat)
            && (q.is_empty()
                || s.name.to_lowercase().contains(&q)
                || s.blurb.to_lowercase().contains(&q))
    };
    (
        all.iter()
            .filter(|s| s.installed)
            .filter(keep)
            .cloned()
            .collect(),
        all.iter()
            .filter(|s| !s.installed)
            .filter(keep)
            .cloned()
            .collect(),
    )
}

/// "8.4K" → 8400, for sorting Discover by use (Oxyn).
pub fn uses_n(s: &str) -> u32 {
    let t = s.trim_end_matches(['K', 'k']);
    let k = s.len() != t.len();
    let v: f32 = t.parse().unwrap_or(0.0);
    (if k { v * 1000.0 } else { v }) as u32
}

/// Install: the ring fills in steps of 25; `Some(true)` once installed.
pub fn install_step(pct: u32) -> (u32, bool) {
    let p = (pct + 25).min(100);
    (p, p == 100)
}

/// When a skill steps in (what you say).
pub fn skill_says(id: u32) -> [&'static str; 3] {
    match id {
        1 => [
            "Write this in our voice",
            "Draft an email to a client",
            "Make this sound like us",
        ],
        6 => [
            "Review this pull request",
            "Is this safe to merge?",
            "Check the tests",
        ],
        _ => [
            "Use this when I ask",
            "Run it every Friday",
            "Check this with it",
        ],
    }
}

/// What a skill does, step by step.
pub fn skill_steps(id: u32) -> [&'static str; 4] {
    match id {
        1 => [
            "Reads the brand guide in Drive",
            "Rewrites in short sentences, active voice",
            "Swaps jargon for plain words",
            "Checks British spelling",
        ],
        _ => [
            "Reads what you shared",
            "Plans the work in steps",
            "Does each step, checking as it goes",
            "Hands back the result with sources",
        ],
    }
}

// ───────────────────────────────────────────────────────────── Schedules ──

/// A schedule.
#[derive(Clone, Debug, PartialEq)]
pub struct Sched {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// The rhythm in words.
    pub rhythm: &'static str,
    /// Hue.
    pub hue: &'static str,
    /// Project name.
    pub project: &'static str,
    /// Next run: when, and how far off ("" when paused).
    pub next: (&'static str, &'static str),
    /// Minutes after 04:00 today of the next run (none when not today).
    pub at: Option<u32>,
    /// Last result tone (`ok`, `warn`, `bad`).
    pub last: &'static str,
    /// Cost per run, cents.
    pub cost: u32,
    /// Switched on.
    pub on: bool,
    /// What it does.
    pub what: &'static str,
}

/// The sample schedules (empty unless `sample`).
pub fn schedules(sample: bool) -> Vec<Sched> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    let s = |id, name, rhythm, hue, project, next, at, last, cost, on| Sched {
        id,
        name,
        rhythm,
        hue,
        project,
        next,
        at,
        last,
        cost,
        on,
        what: "Check the nine makers' sites and shops for new prices, models and campaigns – and tell me what changed since last week.",
    };
    vec![
        s(
            1,
            "Competitor scan",
            "Every Tuesday at 07:23",
            "orange",
            "Tidewater",
            ("Today 07:23", "in a moment"),
            Some(203),
            "ok",
            310,
            true,
        ),
        s(
            2,
            "Tidewater numbers",
            "Every weekday at 7:45",
            "orange",
            "Tidewater",
            ("Today 07:45", "in 22 min"),
            Some(225),
            "ok",
            42,
            true,
        ),
        s(
            3,
            "Inbox triage",
            "Two-hourly from 8:00 to 20:00",
            "purple",
            "About you",
            ("Today 08:00", "in 37 min"),
            Some(240),
            "ok",
            18,
            true,
        ),
        s(
            4,
            "Standup notes",
            "Every weekday at 16:30",
            "purple",
            "Platform",
            ("Today 16:30", "in 9 h"),
            None,
            "ok",
            26,
            true,
        ),
        s(
            5,
            "Weekly update",
            "Every Friday at 16:00",
            "orange",
            "Tidewater",
            ("Fri 16:00", "in 3 days"),
            None,
            "warn",
            55,
            true,
        ),
        s(
            6,
            "Invoice reminders",
            "Monthly, the 1st",
            "grey",
            "Internal",
            ("Dec 1 09:00", "in 14 days"),
            None,
            "bad",
            30,
            true,
        ),
        s(
            7,
            "Nightly evals",
            "Every night at 2:00",
            "teal",
            "Benchkit",
            ("Paused", ""),
            None,
            "ok",
            480,
            false,
        ),
    ]
}

/// The hours the strip shows (04:00 to 14:00) and where now sits, in minutes after 04:00.
pub const STRIP_MINS: u32 = 600;
/// Now on the strip: 07:22.
pub const NOW_MINS: u32 = 202;

/// (on, total).
pub fn sched_counts(all: &[Sched]) -> (usize, usize) {
    (all.iter().filter(|s| s.on).count(), all.len())
}

/// The marks on the strip: (id, percent across, hue), for schedules that are on and run today,
/// plus the later every-2-hours runs of Inbox triage.
pub fn strip_marks(all: &[Sched]) -> Vec<(u32, u32, &'static str)> {
    let mut v = Vec::new();
    for s in all.iter().filter(|s| s.on) {
        if let Some(at) = s.at {
            v.push((s.id, at * 100 / STRIP_MINS, s.hue));
            if s.rhythm.starts_with("Two-hourly") {
                let mut t = at + 120;
                while t <= STRIP_MINS {
                    v.push((s.id, t * 100 / STRIP_MINS, s.hue));
                    t += 120;
                }
            }
        }
    }
    v
}

/// The hour labels of the strip: (label, percent across).
pub fn strip_hours() -> Vec<(String, u32)> {
    (0..=4)
        .map(|i| (format!("{:02}:00", 4 + i * 2), i * 120 * 100 / STRIP_MINS))
        .collect()
}

/// The next schedule that is on and runs today.
pub fn next_sched(all: &[Sched]) -> Option<&Sched> {
    all.iter()
        .filter(|s| s.on && s.at.is_some())
        .min_by_key(|s| s.at)
}

/// What a month of a schedule costs, roughly (Oxyn): runs a month × cost.
pub fn month_cost(all: &[Sched]) -> u32 {
    all.iter()
        .filter(|s| s.on)
        .map(|s| {
            let per = if s.rhythm.starts_with("Every weekday") {
                22
            } else if s.rhythm.starts_with("Two-hourly") {
                7 * 30
            } else if s.rhythm.starts_with("Every night") {
                30
            } else if s.rhythm.starts_with("Monthly") {
                1
            } else {
                4
            };
            per * s.cost
        })
        .sum()
}

/// Last runs of a schedule: (title, when, tone).
pub const LAST_RUNS: [(&str, &str, &str); 3] = [
    (
        "2 makers cut prices, 1 new model",
        "Nov 10 07:23 · €3.42",
        "ok",
    ),
    ("Nothing changed", "Nov 3 07:23 · €2.80", "ok"),
    (
        "Three shops blocked the browser",
        "Oct 27 07:23 · €1.20",
        "bad",
    ),
];

// ───────────────────────────────────────────────────────────── Usage ──

/// The monthly budget, cents.
pub const BUDGET: u32 = 50_000;
/// Today's day of the month.
pub const TODAY: u32 = 19;
/// Days in the month.
pub const MONTH_DAYS: u32 = 30;

/// Spend per day this month so far, cents (empty unless `sample`; `over` doubles the pace).
pub fn daily_spend(sample: bool, over: bool) -> Vec<u32> {
    if !sample {
        return Vec::new();
    }
    let d: [u32; 19] = [
        1120, 1310, 1050, 1290, 1420, 1180, 1460, 1390, 1210, 1530, 1280, 1640, 1720, 1610, 1800,
        1340, 1490, 1840, 1960,
    ];
    let s: u32 = d.iter().sum();
    // Scale so the month to date is €270.40 (or €392.00 when over).
    let target: u32 = if over { 39_200 } else { 27_040 };
    let mut v: Vec<u32> = d.iter().map(|x| x * target / s).collect();
    let got: u32 = v.iter().sum();
    if let Some(last) = v.last_mut() {
        *last += target - got;
    }
    v
}

/// The running total by day.
pub fn cumulative(days: &[u32]) -> Vec<u32> {
    let mut t = 0;
    days.iter()
        .map(|d| {
            t += d;
            t
        })
        .collect()
}

/// The month-end forecast at today's pace, cents.
pub fn forecast(spent: u32, day: u32) -> u32 {
    if day == 0 {
        return 0;
    }
    (u64::from(spent) * u64::from(MONTH_DAYS) / u64::from(day)) as u32
}

/// The forecast in words: "On track for €427 by Nov 30" or "Over budget by Nov 24".
pub fn forecast_words(spent: u32, day: u32) -> String {
    let f = forecast(spent, day);
    if f <= BUDGET {
        format!("On track for €{} by Nov 30", (f + 50) / 100)
    } else {
        let per = spent / day.max(1);
        let hit = BUDGET.div_ceil(per.max(1));
        format!("At this pace the budget runs out on Nov {hit}")
    }
}

/// Share as a whole percent.
pub fn pct(part: u32, of: u32) -> u32 {
    if of == 0 {
        0
    } else {
        ((u64::from(part) * 100 + u64::from(of) / 2) / u64::from(of)) as u32
    }
}

/// A row of Where it went: (key, name, hue, cents, change vs last month in percent).
pub type Spend = (&'static str, &'static str, &'static str, u32, i32);

/// Where it went, by projects or by models (official marks: the hue is the brand id).
pub fn breakdown(by_models: bool) -> Vec<Spend> {
    if by_models {
        vec![
            ("opus", "Opus 5.5", "claude", 13_210, 31),
            ("sonnet", "Sonnet 5.5", "claude", 7_480, -4),
            ("gemini", "Gemini 3.8 Pro", "gemini", 3_890, 12),
            ("gpt", "GPT-6 mini", "openai", 2_460, -18),
        ]
    } else {
        vec![
            ("tide", "Tidewater", "orange", 11_840, 23),
            ("core", "Platform", "purple", 8_210, -7),
            ("bench", "Benchkit", "teal", 4_630, 102),
            ("int", "Internal", "grey", 2_360, -22),
        ]
    }
}

/// What cost the most in a project: (task, runs, cents).
pub const COST_MOST: [(&str, u32, u32); 4] = [
    ("Competitor scan", 6, 3120),
    ("Ingest pipeline", 41, 2780),
    ("Client dashboard", 18, 2260),
    ("Weekly update", 3, 410),
];

/// The month's totals: tasks, runs, per task cents, tokens.
pub fn usage_totals(spent: u32) -> (u32, u32, u32, u32) {
    let tasks = 186;
    (tasks, 1284, spent / tasks, 48_200_000)
}

// ───────────────────────────────────────────────────────────── Permissions ──

/// How much the agent does on its own.
pub const LEVELS: [(&str, &str, &str, &str); 3] = [
    (
        "careful",
        "Careful",
        "Reads on its own, asks before it changes anything",
        "shield",
    ),
    (
        "balanced",
        "Balanced",
        "Works on its own, asks before anything that needs care",
        "scope",
    ),
    (
        "handsoff",
        "Hands-off",
        "Does everything on its own – except the red lines",
        "flash",
    ),
];

/// What the agent does with an action (three settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Never does it.
    Never,
    /// Asks you first.
    Ask,
    /// Does it on its own.
    Own,
}

impl Rule {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Ask => "ask",
            Self::Own => "own",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::Ask => "Asks first",
            Self::Own => "On its own",
        }
    }
    /// From a key.
    pub fn from_key(k: &str) -> Self {
        match k {
            "never" => Self::Never,
            "own" => Self::Own,
            _ => Self::Ask,
        }
    }
}

/// An action kind: `read`, `change`, `care`.
pub type ActKind = &'static str;

/// An app the agent works in: (brand id, name, account, actions (label, kind, glyph)).
pub type PermApp = (
    &'static str,
    &'static str,
    &'static str,
    &'static [(&'static str, ActKind, &'static str)],
);

/// The apps (empty unless `sample`).
pub fn perm_apps(sample: bool) -> Vec<PermApp> {
    if !sample {
        return Vec::new();
    }
    vec![
        (
            "gmail",
            "Gmail",
            "rowan@kestrel.example",
            &[
                ("Read email", "read", "mail"),
                ("Draft replies", "change", "pen"),
                ("Send email", "care", "send"),
                ("Delete email", "care", "trash"),
            ],
        ),
        (
            "slack",
            "Slack",
            "Kestrel workspace",
            &[
                ("Read channels", "read", "chat"),
                ("Post in channels", "change", "send"),
                ("Message people", "care", "people"),
            ],
        ),
        (
            "github",
            "GitHub",
            "kestrel-labs",
            &[
                ("Read code", "read", "doc"),
                ("Open pull requests", "change", "route"),
                ("Merge to main", "care", "tree"),
            ],
        ),
        (
            "googledrive",
            "Google Drive",
            "Shared drive · Clients",
            &[
                ("Read files", "read", "doc"),
                ("Edit files", "change", "pen"),
                ("Share outside", "care", "share"),
            ],
        ),
        (
            "stripe",
            "Stripe",
            "Kestrel GmbH",
            &[
                ("Read payments", "read", "scope"),
                ("Issue refunds", "care", "coin"),
                ("Change prices", "care", "tag"),
            ],
        ),
        (
            "googlecalendar",
            "Google Calendar",
            "rowan@kestrel.example",
            &[
                ("Read events", "read", "cal"),
                ("Book meetings", "change", "clock"),
            ],
        ),
        (
            "web",
            "Web",
            "Browser",
            &[
                ("Browse sites", "read", "globe"),
                ("Fill in forms", "change", "pen"),
                ("Buy things", "care", "bag"),
            ],
        ),
    ]
}

/// The red lines: actions that always ask or never happen, whatever the level.
pub const RED_LINES: [(&str, &str, Rule); 3] = [
    ("Delete email", "gmail", Rule::Never),
    ("Merge to main", "github", Rule::Ask),
    ("Buy things", "web", Rule::Never),
];

/// The rule a level gives an action kind; red lines win.
pub fn rule_for(level: &str, kind: &str, label: &str) -> Rule {
    if let Some(r) = RED_LINES.iter().find(|r| r.0 == label) {
        return r.2;
    }
    match (level, kind) {
        (_, "read") => Rule::Own,
        ("careful", _) => Rule::Ask,
        ("balanced", "change") => Rule::Own,
        ("balanced", _) => Rule::Ask,
        _ => Rule::Own,
    }
}

/// Counts per setting (own, ask, never) over apps at a level, with overrides
/// `(label, rule)` the user picked.
pub fn perm_counts(apps: &[PermApp], level: &str, over: &[(&str, Rule)]) -> (u32, u32, u32) {
    let mut c = (0, 0, 0);
    for (_, _, _, acts) in apps {
        for (l, k, _) in acts.iter() {
            let r = over
                .iter()
                .find(|o| o.0 == *l)
                .map(|o| o.1)
                .unwrap_or_else(|| rule_for(level, k, l));
            match r {
                Rule::Own => c.0 += 1,
                Rule::Ask => c.1 += 1,
                Rule::Never => c.2 += 1,
            }
        }
    }
    c
}

/// "3 on its own · asks before 1 · never 1", leaving out zero parts.
pub fn counts_words(c: (u32, u32, u32)) -> String {
    let mut v = Vec::new();
    if c.0 > 0 {
        v.push(format!("{} on its own", c.0));
    }
    if c.1 > 0 {
        v.push(format!("asks before {}", c.1));
    }
    if c.2 > 0 {
        v.push(format!("never {}", c.2));
    }
    v.join(" · ")
}

/// Lately, in an app: (what, action and when, tone).
pub const LATELY: [(&str, &str, &str); 3] = [
    (
        "Refund €38 to Harbor Bakery – charged twice",
        "Issue refunds · 2 hours ago",
        "ok",
    ),
    (
        "Refund €240 to Lumen Studio",
        "Issue refunds · yesterday",
        "run",
    ),
    (
        "Refund €12 to Pine & Co",
        "Issue refunds · 3 days ago",
        "ok",
    ),
];

// ───────────────────────────────────────────────────────────── Run Detail ──

/// A span kind: thinking, a tool call, a sub-agent, or a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    /// Thinking.
    Think,
    /// A tool call.
    Tool,
    /// A sub-agent.
    Sub,
    /// Failed.
    Fail,
}

impl SpanKind {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Think => "think",
            Self::Tool => "tool",
            Self::Sub => "sub",
            Self::Fail => "fail",
        }
    }
    /// Legend label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Think => "Thinking",
            Self::Tool => "Tool call",
            Self::Sub => "Sub-agent",
            Self::Fail => "Failed",
        }
    }
}

/// A step of a run on the time axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// Id.
    pub id: u32,
    /// Label.
    pub label: &'static str,
    /// Kind.
    pub kind: SpanKind,
    /// The app mark for a tool call ("" for none).
    pub app: &'static str,
    /// Nesting depth (sub-agent steps are 1).
    pub depth: u8,
    /// Start, tenths of a second.
    pub at: u32,
    /// Length, tenths of a second.
    pub dur: u32,
    /// Tokens in and out.
    pub toks: (u32, u32),
    /// Cost, cents.
    pub cost: u32,
}

/// The sample run (empty unless `sample`).
pub fn spans(sample: bool) -> Vec<Span> {
    if !sample {
        return Vec::new();
    }
    use SpanKind::*;
    let s = |id, label, kind, app, depth, at, dur, ti, to, cost| Span {
        id,
        label,
        kind,
        app,
        depth,
        at,
        dur,
        toks: (ti, to),
        cost,
    };
    vec![
        s(1, "Plan the migration", Think, "", 0, 0, 32, 4200, 610, 8),
        s(2, "Read the schema", Tool, "github", 0, 32, 14, 1800, 0, 1),
        s(
            3,
            "Read 14 migrations",
            Tool,
            "github",
            0,
            46,
            32,
            6400,
            0,
            3,
        ),
        s(
            4,
            "Write migration 0042",
            Think,
            "",
            0,
            78,
            63,
            18640,
            2104,
            41,
        ),
        s(
            5,
            "Open pull request #491",
            Tool,
            "github",
            0,
            141,
            9,
            900,
            120,
            1,
        ),
        s(
            6,
            "Reviewer checks the diff",
            Sub,
            "",
            0,
            150,
            74,
            3100,
            420,
            6,
        ),
        s(
            7,
            "Run the test suite",
            Tool,
            "github",
            1,
            155,
            52,
            800,
            0,
            1,
        ),
        s(8, "Write the review", Think, "", 1, 207, 16, 2300, 380, 4),
        s(
            9,
            "Dry run on staging",
            Fail,
            "postgresql",
            0,
            224,
            14,
            300,
            0,
            0,
        ),
        s(
            10,
            "Dry run on staging · retry",
            Tool,
            "postgresql",
            0,
            238,
            96,
            600,
            0,
            1,
        ),
        s(
            11,
            "Add the missing index",
            Think,
            "",
            0,
            334,
            25,
            3900,
            520,
            6,
        ),
        s(
            12,
            "Update pull request #491",
            Tool,
            "github",
            0,
            359,
            9,
            700,
            90,
            1,
        ),
        s(
            13,
            "Run the test suite again",
            Tool,
            "github",
            0,
            368,
            50,
            800,
            0,
            1,
        ),
        s(
            14,
            "Guard checks the data rules",
            Sub,
            "",
            0,
            380,
            46,
            1900,
            260,
            3,
        ),
        s(
            15,
            "Scan the logs for personal data",
            Tool,
            "postgresql",
            1,
            382,
            34,
            600,
            0,
            1,
        ),
        s(
            16,
            "Check the access rules",
            Think,
            "",
            1,
            416,
            8,
            1100,
            140,
            2,
        ),
    ]
}

/// The axis length in tenths of a second (70 s, so 1m 10s is the last tick).
pub const AXIS: u32 = 700;
/// Now on the axis while it runs: 42 s.
pub const RUN_NOW: u32 = 420;

/// A tenth-of-a-second count as words: `3.2s`, `42s`, `1m 10s`.
pub fn secs(t: u32) -> String {
    if t >= 600 {
        let s = t / 10;
        if s.is_multiple_of(60) {
            format!("{}m", s / 60)
        } else {
            format!("{}m {}s", s / 60, s % 60)
        }
    } else if t.is_multiple_of(10) && t >= 100 {
        format!("{}s", t / 10)
    } else {
        format!("{}.{}s", t / 10, t % 10)
    }
}

/// The ticks of the axis: (label, percent across).
pub fn ticks() -> Vec<(String, u32)> {
    (0..=7)
        .map(|i| {
            let t = i * 100;
            let l = if t == 0 { "0s".to_string() } else { secs(t) };
            (l, t * 100 / AXIS)
        })
        .collect()
}

/// Where a span's bar starts and how wide it is, in percent of the axis.
pub fn bar(s: &Span) -> (f32, f32) {
    (
        s.at as f32 * 100.0 / AXIS as f32,
        (s.dur.max(4)) as f32 * 100.0 / AXIS as f32,
    )
}

/// The run's totals: (time, cents, tokens, steps, failed).
pub fn run_totals(all: &[Span], stopped: bool) -> (u32, u32, u32, usize, usize) {
    let end = if stopped { 238 } else { RUN_NOW };
    let cost = all
        .iter()
        .filter(|s| s.at < end)
        .map(|s| s.cost)
        .sum::<u32>()
        + 9;
    let tk = all
        .iter()
        .filter(|s| s.at < end)
        .map(|s| s.toks.0 + s.toks.1)
        .sum();
    let n = all.iter().filter(|s| s.at < end).count();
    let f = all.iter().filter(|s| s.kind == SpanKind::Fail).count();
    (end, cost, tk, n, f)
}

/// The spans shown: all, or (Oxyn) failures and the steps around them.
pub fn span_view(all: &[Span], errors_only: bool, stopped: bool) -> Vec<Span> {
    let end = if stopped { 238 } else { u32::MAX };
    all.iter()
        .filter(|s| s.at < end)
        .filter(|s| {
            !errors_only
                || s.kind == SpanKind::Fail
                || all
                    .iter()
                    .any(|f| f.kind == SpanKind::Fail && f.id.abs_diff(s.id) == 1)
        })
        .cloned()
        .collect()
}

/// A span's input and output.
pub fn span_io(id: u32) -> (&'static str, &'static str) {
    match id {
        9 => (
            "psql staging < 0042_split_users.sql",
            "ERROR: relation \"accounts_email_idx\" does not exist",
        ),
        4 => (
            "Write 0042_split_users with an up and a down step. Copy in batches of 5,000 and keep a view named users for old reads.",
            "CREATE TABLE accounts (…);\nCREATE TABLE profiles (…);\nINSERT INTO accounts SELECT … LIMIT 5000;\nCREATE VIEW users AS SELECT … ;",
        ),
        _ => (
            "Plan the next step from the run so far.",
            "Done. Handed the result to the next step.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s4_agent_ops() {
        assert_eq!(split("creating"), (ScreenState::Sample, "creating"));
        assert_eq!(split("empty"), (ScreenState::Empty, ""));
        assert_eq!(states_for("agent-usage")[3].0, "over");
        // Agents.
        let mut a = agents(true);
        assert!(agents(false).is_empty());
        assert_eq!(agent_counts(&a), [6, 3, 2, 1]);
        assert_eq!(agent_view(&a, "idle", false).len(), 2);
        assert_eq!(agent_view(&a, "all", true)[0].name, "Builder");
        assert_eq!(apps_words(3), "Access to 3 apps");
        assert_eq!(mark_bits(""), [false; 9]);
        assert_eq!(mark_bits("Scout"), mark_bits(" scout "));
        assert!(mark_bits("x").iter().filter(|b| **b).count() >= 3);
        let m = mark_bits("Archivist");
        assert_eq!((m[0], m[3], m[6]), (m[2], m[5], m[8]));
        assert!(HUES.contains(&mark_hue("Scout")));
        assert_eq!(new_agent(&mut a, "  ", "", 0), None);
        assert_eq!(new_agent(&mut a, "Scout", "", 9), Some(7));
        assert_eq!((a[6].role.as_str(), a[6].model), ("New agent", 4));
        assert_eq!(bars(&[5, 10, 20]), vec![25, 50, 100]);
        assert_eq!(bars(&[2]), vec![20]);
        // Skills.
        let s = skills(true);
        let (i, d) = skill_view(&s, "all", "");
        assert_eq!((i.len(), d.len()), (9, 6));
        assert_eq!(skill_view(&s, "data", "").0.len(), 2);
        assert_eq!(skill_view(&s, "all", "invoice").1.len(), 1);
        assert_eq!(uses_n("8.4K"), 8400);
        assert_eq!(uses_n("2K"), 2000);
        assert_eq!(install_step(75), (100, true));
        assert_eq!(install_step(0), (25, false));
        // Schedules.
        let sc = schedules(true);
        assert_eq!(sched_counts(&sc), (6, 7));
        assert_eq!(next_sched(&sc).map(|s| s.id), Some(1));
        let mk = strip_marks(&sc);
        assert_eq!(mk.len(), 6);
        assert_eq!(strip_hours()[1], ("06:00".to_string(), 20));
        assert!(month_cost(&sc) > 0);
        // Usage.
        let d = daily_spend(true, false);
        let c = cumulative(&d);
        assert_eq!(*c.last().unwrap(), 27_040);
        assert_eq!(forecast(27_040, 19), 42_694);
        assert_eq!(forecast_words(27_040, 19), "On track for €427 by Nov 30");
        assert!(forecast_words(39_200, 19).contains("Nov 25"));
        assert_eq!(pct(11_840, 27_040), 44);
        assert_eq!(breakdown(false).len(), 4);
        assert_eq!(breakdown(true)[0].2, "claude");
        assert!(daily_spend(false, false).is_empty());
        // Permissions.
        let ap = perm_apps(true);
        assert_eq!(rule_for("handsoff", "care", "Delete email"), Rule::Never);
        assert_eq!(rule_for("careful", "change", "Edit files"), Rule::Ask);
        let b = perm_counts(&ap, "balanced", &[]);
        assert_eq!(b, (13, 6, 2));
        let h = perm_counts(&ap, "handsoff", &[]);
        assert_eq!(h, (18, 1, 2));
        assert_eq!(
            perm_counts(&ap, "balanced", &[("Issue refunds", Rule::Own)]).0,
            14
        );
        assert_eq!(counts_words((2, 0, 1)), "2 on its own · never 1");
        assert_eq!(Rule::from_key("own"), Rule::Own);
        // Run detail.
        let sp = spans(true);
        assert_eq!(sp.len(), 16);
        assert_eq!(secs(32), "3.2s");
        assert_eq!(secs(420), "42s");
        assert_eq!(secs(700), "1m 10s");
        assert_eq!(secs(600), "1m");
        assert_eq!(ticks().len(), 8);
        assert_eq!(bar(&sp[0]).0, 0.0);
        let t = run_totals(&sp, false);
        assert_eq!((t.0, t.3, t.4), (420, 16, 1));
        assert_eq!(run_totals(&sp, true).3, 9);
        assert_eq!(span_view(&sp, true, false).len(), 3);
        assert_eq!(span_view(&sp, false, true).len(), 9);
        assert!(span_io(9).1.starts_with("ERROR"));
    }
}
