//! Screens S3: agent ops I (Thread, Inbox, Inbox Board, Tasks, Activity, Memory) on the Screens 1
//! frame with the Fieldwork persona's sidebar (`workscr::persona("fieldwork")`). The data is
//! fictional sample data written for Oxyn; every view rule (filters, counts, groups, the next
//! question, the undo window) is a pure function so Dioxus and GPUI draw the same thing.

pub use super::screens2fid::eur;
pub use super::screensfid::ScreenState;

/// A state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

/// Thread: affectUI's nine states, then Empty.
pub const THREAD_STATES: StateList = &[
    ("sample", "Live"),
    ("decision", "Decision"),
    ("scrolled", "Scrolled up"),
    ("offline", "Offline"),
    ("uploading", "Uploading"),
    ("dictating", "Dictating"),
    ("away", "Away summary"),
    ("error", "Error"),
    ("loading", "Loading"),
    ("empty", "Empty"),
];
/// Inbox states.
pub const INBOX_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("answered", "Just answered"),
];
/// Inbox Board states.
pub const BOARD_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("failed", "Run failed"),
];
/// Tasks states.
pub const TASKS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("finished", "Done tab"),
];
/// Activity states.
pub const ACTIVITY_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("undone", "Undone"),
];
/// Memory states.
pub const MEMORY_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("review", "To review"),
];

/// The states of a screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "agent-thread" => THREAD_STATES,
        "inbox" => INBOX_STATES,
        "agent-inbox-board" => BOARD_STATES,
        "agent-tasks" => TASKS_STATES,
        "agent-activity" => ACTIVITY_STATES,
        _ => MEMORY_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 12] = [
        "decision",
        "scrolled",
        "offline",
        "uploading",
        "dictating",
        "away",
        "error",
        "answered",
        "failed",
        "finished",
        "undone",
        "review",
    ];
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

// ───────────────────────────────────────────────────────────── Projects ──

/// A project: key, name, identity hue (`purple`, `blue`, `green`, `orange`).
pub type Project = (&'static str, &'static str, &'static str);

/// The workspace's projects.
pub const PROJECTS: [Project; 4] = [
    ("you", "About you", "purple"),
    ("tide", "Tidewater", "blue"),
    ("core", "Platform", "green"),
    ("bench", "Benchkit", "orange"),
];

/// A project's name and hue.
pub fn project(k: &str) -> Project {
    PROJECTS
        .iter()
        .copied()
        .find(|p| p.0 == k)
        .unwrap_or(PROJECTS[1])
}

// ───────────────────────────────────────────────────────────── Tasks ──

/// Where a task stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// Working.
    Running,
    /// Waiting on a decision.
    NeedsYou,
    /// Finished.
    Done,
    /// Stopped on an error.
    Failed,
}

impl TaskState {
    /// Stable key (`run`, `warn`, `ok`, `bad`).
    pub fn key(self) -> &'static str {
        match self {
            Self::Running => "run",
            Self::NeedsYou => "warn",
            Self::Done => "ok",
            Self::Failed => "bad",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::NeedsYou => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

/// A step of a task: label and 0 done, 1 current, 2 to do.
pub type Step = (&'static str, u8);

/// A task the agents run.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// What it is doing now, in a few words.
    pub now: &'static str,
    /// Project key.
    pub project: &'static str,
    /// State.
    pub state: TaskState,
    /// Steps done.
    pub done: u8,
    /// Steps in all.
    pub of: u8,
    /// Minutes since it last moved.
    pub mins: u32,
    /// One-line brief.
    pub brief: &'static str,
    /// Steps.
    pub steps: &'static [Step],
    /// What happened so far, newest first.
    pub log: &'static [&'static str],
    /// Files it made.
    pub files: &'static [&'static str],
    /// The question waiting on you, if any.
    pub ask: Option<&'static str>,
}

const PIPE_STEPS: &[Step] = &[
    ("Draft the event schema", 0),
    ("Load test at 20k a minute", 0),
    ("Choose a queue", 1),
    ("Ship to staging", 2),
];

/// The sample tasks (empty unless `sample`).
pub fn tasks(sample: bool) -> Vec<Task> {
    if !sample {
        return Vec::new();
    }
    let t = |id, name, now, project, state, done, of, mins| Task {
        id,
        name,
        now,
        project,
        state,
        done,
        of,
        mins,
        brief: "Collect Tidewater's order events and keep a day of them to replay after a bad release.",
        steps: PIPE_STEPS,
        log: &[
            "Asked you which queue fits",
            "Load test held at 20k events a minute",
            "Wrote the design note and the schema",
        ],
        files: &[
            "pipeline-design.md",
            "load-test.csv",
            "event-schema.json",
            "staging",
        ],
        ask: None,
    };
    use TaskState::*;
    let mut v = vec![
        t(
            1,
            "Event pipeline",
            "Choosing a queue",
            "tide",
            NeedsYou,
            2,
            4,
            4,
        ),
        t(
            2,
            "Client dashboard",
            "Picking an empty state",
            "tide",
            NeedsYou,
            1,
            3,
            18,
        ),
        t(
            3,
            "Weekly note",
            "Ready to send",
            "tide",
            NeedsYou,
            2,
            3,
            42,
        ),
        t(
            4,
            "Welcome emails",
            "Drafting the copy",
            "tide",
            Running,
            1,
            3,
            12,
        ),
        t(
            5,
            "Billing hooks",
            "Shipping to staging",
            "core",
            Running,
            3,
            4,
            6,
        ),
        t(
            6,
            "API sunset",
            "Settling the v1 end date",
            "core",
            NeedsYou,
            2,
            4,
            180,
        ),
        t(
            7,
            "Support triage",
            "Rolling out to every queue",
            "core",
            Running,
            2,
            3,
            25,
        ),
        t(
            8,
            "Launch post",
            "Settling on a tone",
            "bench",
            NeedsYou,
            1,
            4,
            60,
        ),
        t(
            9,
            "Eval suite",
            "Running the evals",
            "bench",
            Running,
            3,
            5,
            2,
        ),
        t(
            10,
            "Release notes 2.3",
            "Sent to the team",
            "core",
            Done,
            3,
            3,
            1440,
        ),
        t(11, "Price page copy", "Merged", "bench", Done, 2, 2, 2880),
    ];
    v[0].ask = Some("Which queue should the event service use?");
    v[1].ask = Some("Which empty state goes on the dashboard?");
    v[2].ask = Some("Send the weekly note to Tidewater now?");
    v[5].ask = Some("Keep API v1 until March, or switch it off now?");
    v[7].ask = Some("Which tone suits the launch post?");
    v
}

/// The Tasks view: active (not done) or done, grouped by project in [`PROJECTS`] order.
pub fn task_groups(all: &[Task], done_tab: bool) -> Vec<(Project, Vec<Task>)> {
    PROJECTS
        .iter()
        .filter_map(|p| {
            let v: Vec<Task> = all
                .iter()
                .filter(|t| t.project == p.0 && ((t.state == TaskState::Done) == done_tab))
                .cloned()
                .collect();
            (!v.is_empty()).then_some((*p, v))
        })
        .collect()
}

/// Active and done counts.
pub fn task_counts(all: &[Task]) -> (usize, usize) {
    let d = all.iter().filter(|t| t.state == TaskState::Done).count();
    (all.len() - d, d)
}

/// A progress ring's sweep, 0..=100.
pub fn ring_pct(done: u8, of: u8) -> u32 {
    if of == 0 {
        0
    } else {
        u32::from(done.min(of)) * 100 / u32::from(of)
    }
}

/// A short age: `4m`, `3h`, `2d`.
pub fn age(mins: u32) -> String {
    super::screensfid::ago(mins)
}

// ───────────────────────────────────────────────────────────── Inbox ──

/// One answer to a question.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// Label.
    pub label: &'static str,
    /// One line under it.
    pub sub: &'static str,
    /// The agent's pick.
    pub pick: bool,
    /// For and against (`+` / `−` lines), shown on the open choice.
    pub why: &'static [(&'static str, bool)],
}

/// A question an agent asks.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    /// Id.
    pub id: u32,
    /// The task it belongs to.
    pub task: u32,
    /// The question.
    pub title: &'static str,
    /// Task name.
    pub from: &'static str,
    /// Project key (tints the row glyph).
    pub project: &'static str,
    /// Kind: `choose`, `approve` or `variants`.
    pub kind: &'static str,
    /// Context paragraph.
    pub lead: &'static str,
    /// Minutes ago.
    pub mins: u32,
    /// Not opened yet.
    pub unread: bool,
    /// The answers.
    pub choices: Vec<Choice>,
}

/// The sample questions (empty unless `sample`).
pub fn questions(sample: bool) -> Vec<Question> {
    if !sample {
        return Vec::new();
    }
    let c = |label, sub, pick, why| Choice {
        label,
        sub,
        pick,
        why,
    };
    vec![
        Question {
            id: 1,
            task: 1,
            title: "Which queue should the event service use?",
            from: "Event pipeline",
            project: "tide",
            kind: "choose",
            lead: "Orders land in bursts of up to 20k a minute. The service has to retry, and to replay a whole day of events after a bad release.",
            mins: 4,
            unread: false,
            choices: vec![
                c(
                    "Kafka",
                    "A lasting log with replay out of the box",
                    true,
                    &[
                        ("Replays any day after a bad release", true),
                        ("Absorbs the bursts with headroom", true),
                        ("One more cluster to look after", false),
                        ("More than we need under 1k a minute", false),
                    ],
                ),
                c("SQS", "Hosted, nothing to run", false, &[]),
                c(
                    "Postgres queue",
                    "Reuses the database you already run",
                    false,
                    &[],
                ),
            ],
        },
        Question {
            id: 2,
            task: 2,
            title: "Which empty state goes on the dashboard?",
            from: "Client dashboard",
            project: "tide",
            kind: "variants",
            lead: "New accounts open the dashboard before any orders exist. Two drafts are ready; pick the one to build.",
            mins: 18,
            unread: true,
            choices: vec![
                c(
                    "Sample numbers",
                    "Greyed demo data with a banner",
                    true,
                    &[],
                ),
                c(
                    "First steps",
                    "A checklist that fills as data arrives",
                    false,
                    &[],
                ),
            ],
        },
        Question {
            id: 3,
            task: 3,
            title: "Send the weekly note to Tidewater now?",
            from: "Weekly note",
            project: "tide",
            kind: "approve",
            lead: "Three things shipped this week and one needs their reply: the v1 end date. The note is drafted and checked.",
            mins: 42,
            unread: true,
            choices: vec![
                c("Send it", "Goes out from your address", true, &[]),
                c("Hold until Monday", "Sends Monday at 9:00", false, &[]),
            ],
        },
        Question {
            id: 4,
            task: 8,
            title: "Which tone suits the launch post?",
            from: "Launch post",
            project: "bench",
            kind: "choose",
            lead: "The post goes to developers first. Plain and dry reads well there; warmer copy tested better with buyers.",
            mins: 60,
            unread: false,
            choices: vec![
                c("Plain", "Short lines, facts first", true, &[]),
                c("Warm", "A story, then the facts", false, &[]),
                c("Bold", "Big claims, numbers up top", false, &[]),
            ],
        },
        Question {
            id: 5,
            task: 6,
            title: "Keep API v1 until March, or switch it off now?",
            from: "API sunset",
            project: "core",
            kind: "choose",
            lead: "Two clients still call v1 every day. Switching off now saves a server; waiting gives them time to move.",
            mins: 180,
            unread: false,
            choices: vec![
                c("Keep until March", "Write to both clients today", true, &[]),
                c("Switch off now", "Return a clear error", false, &[]),
            ],
        },
    ]
}

/// After answering `id`, the question that glides in next: the following open one, else the
/// first, else none.
pub fn next_question(open: &[Question], id: u32) -> Option<u32> {
    let i = open.iter().position(|q| q.id == id)?;
    let rest: Vec<&Question> = open.iter().filter(|q| q.id != id).collect();
    if rest.is_empty() {
        return None;
    }
    Some(rest.get(i).unwrap_or(&rest[0]).id)
}

/// The answer a key picks: `1`…`9` the nth choice, `y` the first, `n` the second.
pub fn key_choice(key: &str, n: usize) -> Option<usize> {
    let i = match key {
        "y" | "Y" => 0,
        "n" | "N" => 1,
        k => k.parse::<usize>().ok()?.checked_sub(1)?,
    };
    (i < n).then_some(i)
}

// ───────────────────────────────────────────────────────────── Board ──

/// A card on the Inbox Board.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardCard {
    /// Id.
    pub id: u32,
    /// Title.
    pub title: &'static str,
    /// Two lines of what it's doing.
    pub blurb: &'static str,
    /// Lane: `needs`, `running` or `done`.
    pub lane: &'static str,
    /// Agent name and hue.
    pub agent: (&'static str, &'static str),
    /// Seconds it has run.
    pub secs: u32,
    /// Sources read.
    pub reads: u32,
    /// Tool calls.
    pub calls: u32,
    /// Failed.
    pub failed: bool,
    /// What it did: (line, age).
    pub did: &'static [(&'static str, &'static str)],
}

/// The board lanes: key, label.
pub const LANES: [(&str, &str); 3] = [
    ("needs", "Needs you"),
    ("running", "Running"),
    ("done", "Done"),
];

/// The sample board (empty unless `sample`); `failed` marks the second card failed.
pub fn board(sample: bool, failed: bool) -> Vec<BoardCard> {
    if !sample {
        return Vec::new();
    }
    let did: &'static [(&'static str, &'static str)] = &[
        ("Read the schema and 14 migrations", "6m ago"),
        ("Wrote migration 0042_split_accounts", "4m ago"),
        ("Staging rehearsal passed cleanly", "2m ago"),
        ("Ready to apply it to production", "1m ago"),
    ];
    let c = |id, title, blurb, lane, agent, secs, reads, calls| BoardCard {
        id,
        title,
        blurb,
        lane,
        agent,
        secs,
        reads,
        calls,
        failed: false,
        did,
    };
    let mut v = vec![
        c(
            1,
            "Split the accounts table",
            "Moves sign-ins and profiles into two tables on Postgres 17. The staging dry run passed.",
            "needs",
            ("Builder", "blue"),
            373,
            9,
            14,
        ),
        c(
            2,
            "Tidy the contact list",
            "Found 1,180 likely duplicates and is merging them in batches of fifty.",
            "needs",
            ("Keeper", "teal"),
            512,
            2,
            6,
        ),
        c(
            3,
            "Review PR #482",
            "Billing hooks: going through the changes and test output before leaving notes.",
            "running",
            ("Reviewer", "purple"),
            221,
            12,
            21,
        ),
        c(
            4,
            "Release notes 2.4",
            "Sorting 23 merged changes into new features and bug fixes.",
            "running",
            ("Writer", "pink"),
            79,
            23,
            5,
        ),
        c(
            5,
            "Package audit",
            "Checking 214 packages for known security advisories.",
            "running",
            ("Guard", "teal"),
            43,
            3,
            4,
        ),
        c(
            6,
            "Weekly numbers",
            "Collected this week's figures and shared a short summary in chat.",
            "done",
            ("Analyst", "pink"),
            252,
            3,
            8,
        ),
        c(
            7,
            "Welcome emails",
            "Rewrote the three welcome emails in the house voice.",
            "done",
            ("Writer", "pink"),
            423,
            5,
            11,
        ),
    ];
    if failed {
        v[1].failed = true;
        v[1].blurb = "Merged 600 of 1,180 duplicates, then the CRM's rate limit stopped the run.";
    }
    v
}

/// Seconds as `6m 13s`, `43s`.
pub fn mmss(s: u32) -> String {
    if s < 60 {
        format!("{s}s")
    } else {
        format!("{}m {:02}s", s / 60, s % 60)
    }
}

/// The board's tab counts: all, needs, running, done.
pub fn board_counts(all: &[BoardCard]) -> [usize; 4] {
    let n = |l: &str| all.iter().filter(|c| c.lane == l).count();
    [all.len(), n("needs"), n("running"), n("done")]
}

/// Move a card to a lane (approve → running, retry → running, reject → done).
pub fn move_to(all: &mut [BoardCard], id: u32, lane: &'static str) {
    if let Some(c) = all.iter_mut().find(|c| c.id == id) {
        c.lane = lane;
        c.failed = false;
    }
}

// ───────────────────────────────────────────────────────────── Activity ──

/// How an action was allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The agent's permissions let it act alone.
    OnItsOwn,
    /// You approved it first.
    Approved,
}

impl Mode {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::OnItsOwn => "On its own",
            Self::Approved => "You approved",
        }
    }
    /// Key (`own`, `ok`).
    pub fn key(self) -> &'static str {
        match self {
            Self::OnItsOwn => "own",
            Self::Approved => "ok",
        }
    }
}

/// One thing an agent did in an app.
#[derive(Clone, Debug, PartialEq)]
pub struct Act {
    /// Id.
    pub id: u32,
    /// 0 today, 1 yesterday.
    pub day: u8,
    /// Clock time.
    pub at: &'static str,
    /// Official mark id.
    pub app: &'static str,
    /// App name.
    pub app_name: &'static str,
    /// What it did.
    pub text: &'static str,
    /// How it was allowed.
    pub mode: Mode,
    /// Task.
    pub task: &'static str,
    /// Minutes left to take it back (0: final).
    pub undo_min: u32,
    /// Already taken back.
    pub undone: bool,
    /// Cost in cents.
    pub cents: u32,
    /// Minutes ago.
    pub mins: u32,
    /// What happened: (field, value) lines then a body.
    pub what: &'static [(&'static str, &'static str)],
    /// The body.
    pub body: &'static str,
    /// The permission that allowed it.
    pub rule: &'static str,
    /// What undo does.
    pub undo_does: &'static str,
}

/// The sample activity (empty unless `sample`).
pub fn activity(sample: bool) -> Vec<Act> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    fn a(
        id: u32,
        day: u8,
        at: &'static str,
        app: &'static str,
        app_name: &'static str,
        text: &'static str,
        mode: Mode,
        task: &'static str,
        undo_min: u32,
        mins: u32,
    ) -> Act {
        Act {
            id,
            day,
            at,
            app,
            app_name,
            text,
            mode,
            task,
            undo_min,
            undone: false,
            cents: 4,
            mins,
            what: &[],
            body: "",
            rule: "Act inside the project's apps",
            undo_does: "Reverts the change it made",
        }
    }
    use Mode::*;
    let mut v = vec![
        a(
            1,
            0,
            "07:10",
            "gmail",
            "Gmail",
            "Sent the weekly note to Juno Hart",
            Approved,
            "Weekly note",
            18,
            12,
        ),
        a(
            2,
            0,
            "06:44",
            "stripe",
            "Stripe",
            "Refunded €38 to Otis Fenn",
            OnItsOwn,
            "Billing hooks",
            0,
            38,
        ),
        a(
            3,
            0,
            "06:18",
            "slack",
            "Slack",
            "Posted the staging link in #core",
            OnItsOwn,
            "Billing hooks",
            45,
            64,
        ),
        a(
            4,
            0,
            "05:47",
            "github",
            "GitHub",
            "Opened pull request #482: retry failed billing hooks",
            OnItsOwn,
            "Billing hooks",
            90,
            95,
        ),
        a(
            5,
            0,
            "04:22",
            "googlecalendar",
            "Calendar",
            "Booked a review call with Lark Abara for Thursday 10:30",
            Approved,
            "Welcome emails",
            120,
            180,
        ),
        a(
            6,
            0,
            "03:22",
            "googledrive",
            "Drive",
            "Created “Tidewater pipeline design” in Clients",
            OnItsOwn,
            "Event pipeline",
            240,
            240,
        ),
        a(
            7,
            1,
            "17:22",
            "github",
            "GitHub",
            "Merged #479: move our apps to API v2",
            Approved,
            "API sunset",
            0,
            820,
        ),
        a(
            8,
            1,
            "14:22",
            "gmail",
            "Gmail",
            "Sent a payment reminder to Brightwell Studio",
            Approved,
            "Billing hooks",
            0,
            1000,
        ),
        a(
            9,
            1,
            "12:22",
            "slack",
            "Slack",
            "Asked Lark Abara for the staging keys",
            Approved,
            "Billing hooks",
            0,
            1120,
        ),
    ];
    v[0].what = &[
        ("To", "juno@tidewater.example"),
        ("Subject", "Tidewater, week 46"),
    ];
    v[0].body = "Hi Juno, three things shipped this week, and one needs your answer: the v1 end date. Details below.";
    v[0].rule = "Send email to clients";
    v[0].undo_does = "Pulls the email back while it is still unread";
    v[1].what = &[("Customer", "Otis Fenn"), ("Amount", "€38.00")];
    v[1].body = "A double charge on the March invoice, found by the billing hooks check.";
    v[1].rule = "Refund up to €50";
    v[1].undo_does = "Refunds can't be reversed";
    v[2].what = &[("Channel", "#core")];
    v[2].body =
        "Staging is up with the billing hook retries. Link inside; it stays up until Friday.";
    v[3].what = &[("Repository", "kestrel/billing"), ("Branch", "retry-hooks")];
    v[3].body = "Retries failed billing hooks three times with backoff, then files a ticket.";
    v[4].what = &[("Guest", "Lark Abara"), ("When", "Thursday 10:30")];
    v[4].body = "Thirty minutes to review the welcome emails before they go out.";
    v[5].what = &[("Folder", "Clients")];
    v[5].body = "The design note for the event pipeline, shared with the Tidewater team.";
    v[6].what = &[("Repository", "kestrel/api")];
    v[6].body = "Our own apps now call API v2 only.";
    v[7].undone = true;
    v[7].what = &[("To", "accounts@brightwell.example")];
    v[7].body = "A friendly nudge about invoice 2207.";
    v[8].what = &[("Channel", "Direct message")];
    v[8].body = "Could you share the staging keys for the billing service?";
    v
}

/// Activity filter tabs: key, label.
pub const ACT_TABS: [(&str, &str); 4] = [
    ("all", "All"),
    ("own", "On its own"),
    ("ok", "You approved"),
    ("undone", "Undone"),
];

/// Counts for [`ACT_TABS`].
pub fn act_counts(all: &[Act]) -> [usize; 4] {
    [
        all.len(),
        all.iter().filter(|a| a.mode == Mode::OnItsOwn).count(),
        all.iter().filter(|a| a.mode == Mode::Approved).count(),
        all.iter().filter(|a| a.undone).count(),
    ]
}

/// The day-grouped view for a tab and a search: (day label, rows).
pub fn act_view(all: &[Act], tab: &str, q: &str) -> Vec<(&'static str, Vec<Act>)> {
    let q = q.trim().to_lowercase();
    let keep = |a: &&Act| {
        (match tab {
            "own" => a.mode == Mode::OnItsOwn,
            "ok" => a.mode == Mode::Approved,
            "undone" => a.undone,
            _ => true,
        }) && (q.is_empty()
            || a.text.to_lowercase().contains(&q)
            || a.task.to_lowercase().contains(&q)
            || a.app_name.to_lowercase().contains(&q))
    };
    [(0u8, "Today"), (1, "Yesterday")]
        .into_iter()
        .filter_map(|(d, l)| {
            let v: Vec<Act> = all
                .iter()
                .filter(|a| a.day == d)
                .filter(keep)
                .cloned()
                .collect();
            (!v.is_empty()).then_some((l, v))
        })
        .collect()
}

/// The undo window left `elapsed_s` seconds after opening: seconds, 0 when closed.
pub fn undo_left(a: &Act, elapsed_s: u64) -> u64 {
    if a.undone || a.undo_min == 0 {
        return 0;
    }
    (u64::from(a.undo_min) * 60).saturating_sub(elapsed_s)
}

/// The countdown in words: `Possible for 18 more minutes`, `… 45 more seconds`.
pub fn undo_words(left_s: u64) -> String {
    match left_s {
        0 => "The window has closed".to_string(),
        1..=59 => format!("Possible for {left_s} more seconds"),
        60..=119 => "Possible for 1 more minute".to_string(),
        _ => format!("Possible for {} more minutes", left_s / 60),
    }
}

/// The countdown as a clock: `17:42`.
pub fn undo_clock(left_s: u64) -> String {
    format!("{:02}:{:02}", left_s / 60, left_s % 60)
}

// ───────────────────────────────────────────────────────────── Memory ──

/// A thing the agents remember.
#[derive(Clone, Debug, PartialEq)]
pub struct Memo {
    /// Id.
    pub id: u32,
    /// The memory.
    pub text: &'static str,
    /// Scope: a project key.
    pub scope: &'static str,
    /// Where it came from: `thread`, `you`, `decision` or `file`.
    pub from: &'static str,
    /// The source label (or file name).
    pub source: &'static str,
    /// Times used.
    pub uses: u32,
    /// Minutes since last used.
    pub mins: u32,
    /// Waiting for you to review.
    pub review: bool,
}

/// The sample memory (empty unless `sample`).
pub fn memory(sample: bool) -> Vec<Memo> {
    if !sample {
        return Vec::new();
    }
    let m = |id, text, scope, from, source, uses, mins| Memo {
        id,
        text,
        scope,
        from,
        source,
        uses,
        mins,
        review: false,
    };
    let mut v = vec![
        m(
            1,
            "Give the answer first, then up to three lines of why",
            "you",
            "thread",
            "Thread",
            34,
            26,
        ),
        m(
            2,
            "Works from Lisbon on WET; nothing booked before 9:30",
            "you",
            "you",
            "You",
            12,
            300,
        ),
        m(
            3,
            "Nothing goes to clients after 15:00 on a Friday",
            "you",
            "decision",
            "Decision",
            6,
            2880,
        ),
        m(
            4,
            "Amounts in euros with the sign in front: €1,240",
            "you",
            "file",
            "house-style.md",
            9,
            1440,
        ),
        m(
            5,
            "Spells the British way: colour, organise",
            "you",
            "you",
            "You",
            21,
            60,
        ),
        m(
            6,
            "Juno Hart approves anything their customers will see",
            "tide",
            "thread",
            "Thread",
            8,
            42,
        ),
        m(
            7,
            "Order events are JSON with a version field",
            "tide",
            "decision",
            "Decision",
            5,
            240,
        ),
        m(
            8,
            "Their team checks the dashboard at 8:30, so numbers land by 8:00",
            "tide",
            "thread",
            "Thread",
            3,
            4320,
        ),
        m(
            9,
            "Staging gets a release only when every test passes, with no exceptions for small fixes",
            "core",
            "you",
            "You",
            14,
            6,
        ),
        m(
            10,
            "Two clients still call API v1; write before changing it",
            "core",
            "file",
            "v1-calls.csv",
            2,
            180,
        ),
        m(
            11,
            "Scores show one decimal: 87.4, never 87.38",
            "bench",
            "decision",
            "Decision",
            4,
            2880,
        ),
        m(
            12,
            "Prefers short release notes over long ones",
            "core",
            "thread",
            "Thread",
            0,
            30,
        ),
        m(
            13,
            "Tidewater's fiscal year starts in April",
            "tide",
            "file",
            "contract.pdf",
            0,
            90,
        ),
    ];
    v[11].review = true;
    v[12].review = true;
    v
}

/// Memory groups for a tab (`saved` or `review`) and a search, in [`PROJECTS`] order.
pub fn memo_groups(all: &[Memo], review: bool, q: &str) -> Vec<(Project, Vec<Memo>)> {
    let q = q.trim().to_lowercase();
    PROJECTS
        .iter()
        .filter_map(|p| {
            let v: Vec<Memo> = all
                .iter()
                .filter(|m| {
                    m.scope == p.0
                        && m.review == review
                        && (q.is_empty() || m.text.to_lowercase().contains(&q))
                })
                .cloned()
                .collect();
            (!v.is_empty()).then_some((*p, v))
        })
        .collect()
}

/// Saved and to-review counts.
pub fn memo_counts(all: &[Memo]) -> (usize, usize) {
    let r = all.iter().filter(|m| m.review).count();
    (all.len() - r, r)
}

/// Add a memory the person typed (About you, from You); returns its id, or None when blank.
pub fn remember(all: &mut Vec<Memo>, text: &str) -> Option<u32> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let id = all.iter().map(|m| m.id).max().unwrap_or(0) + 1;
    all.insert(
        0,
        Memo {
            id,
            text: Box::leak(t.to_string().into_boxed_str()),
            scope: "you",
            from: "you",
            source: "You",
            uses: 0,
            mins: 0,
            review: false,
        },
    );
    Some(id)
}

// ───────────────────────────────────────────────────────────── Thread ──

/// One entry of the thread with the orchestrator.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    /// The person.
    Me(String),
    /// The orchestrator: text and time.
    Agent(String, &'static str),
    /// Tools that ran side by side: count and seconds ×10.
    Tools(u32, u32),
    /// A task card: task id.
    Task(u32),
    /// File chips.
    Files(&'static [&'static str]),
}

/// The sample thread (empty unless `sample`).
pub fn thread(sample: bool) -> Vec<Line> {
    if !sample {
        return Vec::new();
    }
    vec![
        Line::Me("Morning. Tidewater first this week: the event pipeline, the client dashboard and the weekly note. Keep the API sunset moving alongside.".into()),
        Line::Agent(String::new(), "08:33"),
        Line::Tools(3, 21),
        Line::Agent("Started. Four tasks are running in parallel, and I'll only come back to you when one needs a call.".into(), ""),
        Line::Task(2),
        Line::Task(6),
        Line::Me("How did the pipeline load test go?".into()),
        Line::Agent(String::new(), "11:13"),
        Line::Tools(2, 11),
        Line::Agent("It held at 20k events a minute: p95 under 100 ms and 0.02% errors. It falls behind past 30k, far above Tidewater's peaks.".into(), ""),
        Line::Files(&["load-test.csv", "pipeline-design.md"]),
        Line::Me("Good. Get the pipeline onto staging today, and tell me what you need from me.".into()),
    ]
}

/// The header chips: running and needing you.
pub fn thread_counts(all: &[Task]) -> (usize, usize) {
    (
        all.iter().filter(|t| t.state == TaskState::Running).count() + 5,
        all.iter()
            .filter(|t| t.state == TaskState::NeedsYou)
            .count(),
    )
}

/// The folded tools line: `Ran 3 tools · 2.1s`.
pub fn tools_words(n: u32, ds: u32) -> String {
    let w = if n == 1 { "tool" } else { "tools" };
    format!("Ran {n} {w} · {}.{}s", ds / 10, ds % 10)
}

/// What came in while you were away (the Away summary).
pub const AWAY: [&str; 3] = [
    "Billing hooks reached staging; all 212 tests pass",
    "Weekly note drafted and waiting for your yes",
    "Eval suite finished: 87.4, up 1.2 on last week",
];

/// The reply the orchestrator gives to a message (fictional, deterministic).
pub fn reply(text: &str) -> String {
    let t = text.to_lowercase();
    if t.contains("stop") || t.contains("pause") {
        "Paused. Nothing else goes out until you say so.".to_string()
    } else if t.contains('?') {
        "Short answer: yes. Nothing blocks it; I noted it on the Tidewater tasks.".to_string()
    } else {
        "On it. I added this to the plan and will check in here when it needs you.".to_string()
    }
}

/// Seconds of dictation as `0:07`.
pub fn dict_clock(s: u64) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s3_agent_ops() {
        assert_eq!(split("decision"), (ScreenState::Sample, "decision"));
        assert_eq!(split("loading"), (ScreenState::Loading, ""));
        assert_eq!(THREAD_STATES.len(), 10);
        assert_eq!(states_for("inbox")[3].0, "answered");
        // Tasks.
        let t = tasks(true);
        assert!(tasks(false).is_empty());
        assert_eq!(task_counts(&t), (9, 2));
        let g = task_groups(&t, false);
        assert_eq!(g.len(), 3);
        assert_eq!(g[0].1.len(), 4);
        assert_eq!(ring_pct(2, 4), 50);
        assert_eq!(ring_pct(1, 0), 0);
        // Inbox: the next question glides in.
        let q = questions(true);
        assert_eq!(next_question(&q, 1), Some(2));
        assert_eq!(next_question(&q, 5), Some(1));
        assert_eq!(next_question(&q[..1], 1), None);
        assert_eq!(key_choice("2", 3), Some(1));
        assert_eq!(key_choice("4", 3), None);
        assert_eq!(key_choice("y", 2), Some(0));
        assert_eq!(key_choice("x", 2), None);
        // Board.
        let mut b = board(true, true);
        assert!(b[1].failed);
        assert_eq!(board_counts(&b), [7, 2, 3, 2]);
        move_to(&mut b, 1, "running");
        assert_eq!(board_counts(&b), [7, 1, 4, 2]);
        assert_eq!(mmss(373), "6m 13s");
        assert_eq!(mmss(43), "43s");
        // Activity.
        let a = activity(true);
        assert_eq!(act_counts(&a), [9, 4, 5, 1]);
        let v = act_view(&a, "all", "");
        assert_eq!((v[0].1.len(), v[1].1.len()), (6, 3));
        assert_eq!(act_view(&a, "own", "")[0].1.len(), 4);
        assert_eq!(act_view(&a, "all", "stripe").len(), 1);
        assert_eq!(undo_left(&a[0], 30), 18 * 60 - 30);
        assert_eq!(undo_left(&a[1], 0), 0);
        assert_eq!(undo_words(18 * 60), "Possible for 18 more minutes");
        assert_eq!(undo_words(45), "Possible for 45 more seconds");
        assert_eq!(undo_clock(1062), "17:42");
        // Memory.
        let mut m = memory(true);
        assert_eq!(memo_counts(&m), (11, 2));
        assert_eq!(memo_groups(&m, false, "").len(), 4);
        assert_eq!(memo_groups(&m, true, "").len(), 2);
        assert_eq!(memo_groups(&m, false, "decimal").len(), 1);
        assert_eq!(remember(&mut m, "  "), None);
        assert_eq!(remember(&mut m, "Coffee at 10"), Some(14));
        assert_eq!(memo_counts(&m), (12, 2));
        // Thread.
        assert_eq!(thread(true).len(), 12);
        assert_eq!(thread_counts(&t), (9, 5));
        assert_eq!(tools_words(3, 21), "Ran 3 tools · 2.1s");
        assert!(reply("stop").starts_with("Paused"));
        assert_eq!(dict_clock(67), "1:07");
    }
}
