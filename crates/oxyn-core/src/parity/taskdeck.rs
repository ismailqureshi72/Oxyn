//! Screens S9 (Agent apps I): Taskdeck, a desktop agent app in the task-runner pattern: a sidebar
//! with New task, Tasks, Plugins & skills, a Local / Cloud switch, projects and recent tasks, and the
//! account row; a start screen with the task composer (attach, run mode, model, voice, folder);
//! a running task with its step timeline, tool calls, an approval and an artifacts side panel; and
//! a task board. Our own names and design, inspired by how such apps behave. The rules live here so
//! Dioxus and GPUI agree. Every name, file and company is fictional.

pub use super::screensfid::{
    EnvMode, PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, account_menu, usage,
};

/// The app's name.
pub const APP: &str = "Taskdeck";

/// A state list: key and label for the state bar.
pub type StateList = &'static [(&'static str, &'static str)];

const START_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("cloud", "Cloud"),
];
const TASK_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("approval", "Needs you"),
];
const BOARD_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("menu", "Account menu"),
];

/// The screens' own states.
pub const OWN: [&str; 3] = ["cloud", "approval", "menu"];

/// The views, in sidebar order: key and title.
pub const VIEWS: [(&str, &str); 3] = [("start", "New task"), ("task", "Task"), ("board", "Tasks")];

/// The states of an S9 screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "taskdeck-task" => TASK_STATES,
        "taskdeck-board" => BOARD_STATES,
        _ => START_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

/// The view a slug opens on (`start`, `task` or `board`).
pub fn view_of(slug: &str) -> &'static str {
    VIEWS
        .iter()
        .map(|v| v.0)
        .find(|v| slug.ends_with(v))
        .unwrap_or("start")
}

// ───────────────────────────────────────────────────────────── Tasks ──

/// Where a task stands. The board has one column per status except Failed, which sits in Done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Waiting for a free agent.
    Queued,
    /// Working.
    Running,
    /// Stopped on a question or an approval.
    NeedsYou,
    /// Finished.
    Done,
    /// Stopped on an error.
    Failed,
}

impl Status {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::NeedsYou => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
    /// Tone key: `run`, `wait`, `ok`, `bad` or `` (neutral).
    pub fn tone(self) -> &'static str {
        match self {
            Self::Queued => "",
            Self::Running => "run",
            Self::NeedsYou => "wait",
            Self::Done => "ok",
            Self::Failed => "bad",
        }
    }
    /// The board column (0–3).
    pub fn column(self) -> usize {
        match self {
            Self::Queued => 0,
            Self::Running => 1,
            Self::NeedsYou => 2,
            Self::Done | Self::Failed => 3,
        }
    }
}

/// The board's columns, left to right.
pub const COLUMNS: [Status; 4] = [
    Status::Queued,
    Status::Running,
    Status::NeedsYou,
    Status::Done,
];

/// Projects: key, name, workspace folder.
pub const PROJECTS: [(&str, &str, &str); 3] = [
    ("office", "Home office", "home-office"),
    ("site", "Site relaunch", "site-relaunch"),
    ("report", "Quarterly report", "q3-report"),
];

/// A task in the sidebar and on the board.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Stable id.
    pub id: u32,
    /// Short title.
    pub title: String,
    /// Project index into [`PROJECTS`], or `None` for a loose task.
    pub project: Option<usize>,
    /// Where it stands.
    pub status: Status,
    /// Where it runs.
    pub env: EnvMode,
    /// Steps done and total.
    pub steps: (u8, u8),
    /// Last change, e.g. "2m".
    pub ago: String,
}

fn t(
    id: u32,
    title: &str,
    project: Option<usize>,
    status: Status,
    env: EnvMode,
    steps: (u8, u8),
    ago: &str,
) -> Task {
    Task {
        id,
        title: title.to_string(),
        project,
        status,
        env,
        steps,
        ago: ago.to_string(),
    }
}

/// The id of the task the Task screen opens.
pub const OPEN_TASK: u32 = 1;

/// The sample tasks, newest first.
pub fn sample_tasks() -> Vec<Task> {
    use EnvMode::{Cloud, Local};
    use Status::*;
    vec![
        t(
            1,
            "Compare standing desks under $600",
            Some(0),
            Running,
            Local,
            (3, 6),
            "now",
        ),
        t(
            2,
            "Rename holiday photos by date",
            Some(0),
            NeedsYou,
            Local,
            (2, 4),
            "4m",
        ),
        t(
            3,
            "Rebuild the pricing page",
            Some(1),
            Running,
            Cloud,
            (5, 9),
            "1m",
        ),
        t(
            4,
            "Check every link on the old site",
            Some(1),
            Done,
            Cloud,
            (6, 6),
            "2h",
        ),
        t(
            5,
            "Draft the launch announcement",
            Some(1),
            Queued,
            Cloud,
            (0, 5),
            "12m",
        ),
        t(
            6,
            "Chart revenue by region",
            Some(2),
            Done,
            Local,
            (7, 7),
            "1d",
        ),
        t(
            7,
            "Summarise churn interviews",
            Some(2),
            Failed,
            Cloud,
            (3, 6),
            "3h",
        ),
        t(
            8,
            "Book a table for Friday",
            None,
            Queued,
            Local,
            (0, 3),
            "20m",
        ),
        t(
            9,
            "Tidy the downloads folder",
            None,
            Done,
            Local,
            (4, 4),
            "2d",
        ),
    ]
}

/// The tasks for a data state (none when empty or loading).
pub fn tasks(st: ScreenState) -> Vec<Task> {
    if st == ScreenState::Sample {
        sample_tasks()
    } else {
        Vec::new()
    }
}

/// Running and needs-you counts over the given tasks.
pub fn counts(list: &[Task]) -> (usize, usize) {
    (
        list.iter().filter(|t| t.status == Status::Running).count(),
        list.iter().filter(|t| t.status == Status::NeedsYou).count(),
    )
}

/// A task title from a prompt: its first sentence, at most 44 characters (cut at a word, with …).
pub fn title_from(prompt: &str) -> String {
    let p = prompt.trim();
    let first = p
        .split(['.', '\n', '?', '!'])
        .find(|s| !s.trim().is_empty())
        .unwrap_or(p)
        .trim();
    if first.chars().count() <= 44 {
        return first.to_string();
    }
    let cut: String = first.chars().take(44).collect();
    let cut = match cut.rfind(' ') {
        Some(i) if i > 20 => cut[..i].to_string(),
        _ => cut,
    };
    format!("{}…", cut.trim_end_matches([',', ';', ':']))
}

/// Longest prompt the composer takes.
pub const PROMPT_CAP: usize = 600;

/// Why a prompt cannot be sent, if it cannot.
pub fn prompt_error(p: &str) -> Option<&'static str> {
    let n = p.trim().chars().count();
    if n == 0 {
        Some("Describe the task first")
    } else if n < 6 {
        Some("Say a little more so the agent can plan")
    } else if n > PROMPT_CAP {
        Some("Keep it under 600 characters; attach the rest as a file")
    } else {
        None
    }
}

/// Add a task from a prompt at the top of the list; it starts Running. Returns its id.
pub fn add_task(
    list: &mut Vec<Task>,
    prompt: &str,
    env: EnvMode,
    project: Option<usize>,
) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    list.insert(
        0,
        t(
            id,
            &title_from(prompt),
            project,
            Status::Running,
            env,
            (0, 6),
            "now",
        ),
    );
    Ok(id)
}

/// Board actions on a card: key, label, the status it moves to.
pub const CARD_ACTS: [(&str, &str, Status); 3] = [
    ("run", "Run now", Status::Running),
    ("queue", "Back to queue", Status::Queued),
    ("done", "Mark done", Status::Done),
];

/// The actions a card offers (never its own status).
pub fn card_acts(s: Status) -> Vec<(&'static str, &'static str, Status)> {
    CARD_ACTS
        .into_iter()
        .filter(|a| a.2 != s && !(s == Status::Done && a.0 == "done"))
        .collect()
}

/// Move a task to a status (Running restarts a finished task's steps). Returns false if absent.
pub fn move_task(list: &mut [Task], id: u32, to: Status) -> bool {
    let Some(x) = list.iter_mut().find(|t| t.id == id) else {
        return false;
    };
    if to == Status::Running && matches!(x.status, Status::Done | Status::Failed) {
        x.steps.0 = 0;
    }
    if to == Status::Done {
        x.steps.0 = x.steps.1;
    }
    x.status = to;
    x.ago = "now".to_string();
    true
}

/// Remove a task. Returns false if absent.
pub fn delete_task(list: &mut Vec<Task>, id: u32) -> bool {
    let n = list.len();
    list.retain(|t| t.id != id);
    list.len() != n
}

/// Tasks whose title or project matches `q` (case-insensitive); all when `q` is blank.
pub fn matches(t: &Task, q: &str) -> bool {
    let q = q.trim().to_lowercase();
    q.is_empty()
        || t.title.to_lowercase().contains(&q)
        || t.project
            .is_some_and(|p| PROJECTS[p].1.to_lowercase().contains(&q))
}

// ───────────────────────────────────────────────────────────── Run ──

/// How far a step has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    /// Not reached.
    Pending,
    /// Working.
    Running,
    /// Waiting on the user's approval.
    Waiting,
    /// Finished.
    Done,
    /// Skipped (approval denied).
    Skipped,
}

/// One step of a run: kind (`plan`, `tool`, `ask`, `write`), title, tool, argument, output,
/// seconds and state.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    /// `plan`, `tool`, `ask` (needs approval) or `write`.
    pub kind: &'static str,
    /// What the step does.
    pub title: &'static str,
    /// Tool name (`browser.search`), empty for none.
    pub tool: &'static str,
    /// The tool's argument.
    pub arg: &'static str,
    /// What came back.
    pub out: &'static str,
    /// Seconds it took (or has taken).
    pub secs: u32,
    /// State.
    pub state: StepState,
}

const fn st(
    kind: &'static str,
    title: &'static str,
    tool: &'static str,
    arg: &'static str,
    out: &'static str,
    secs: u32,
) -> Step {
    Step {
        kind,
        title,
        tool,
        arg,
        out,
        secs,
        state: StepState::Pending,
    }
}

/// The open task's prompt.
pub const PROMPT: &str = "Compare standing desks under $600 with a memory preset and a 10-year warranty, then write me a short buying note.";

/// The run's steps, all pending.
pub fn run_plan() -> Vec<Step> {
    vec![
        st(
            "plan",
            "Plan the work",
            "",
            "",
            "6 steps: read the brief, search, build a sheet, chart it, write the note",
            4,
        ),
        st(
            "tool",
            "Read the brief",
            "files.read",
            "brief.md",
            "Budget $600 · memory preset · 10-year warranty",
            2,
        ),
        st(
            "tool",
            "Search for desks",
            "browser.search",
            "standing desk memory preset warranty",
            "14 results · 6 stores opened",
            38,
        ),
        st(
            "tool",
            "Build the comparison sheet",
            "sheet.write",
            "desks.csv",
            "6 desks × 7 columns",
            21,
        ),
        st(
            "ask",
            "Chart prices with a script",
            "terminal.run",
            "python chart.py desks.csv",
            "price-chart.png written",
            9,
        ),
        st(
            "write",
            "Write the buying note",
            "files.write",
            "buying-note.md",
            "1 page · top pick and two runners-up",
            17,
        ),
    ]
}

/// The plan any other task runs (the open task has its own, see [`run_plan`]).
pub fn generic_plan() -> Vec<Step> {
    vec![
        st("plan", "Plan the work", "", "", "5 steps", 3),
        st(
            "tool",
            "Gather what it needs",
            "files.read",
            "notes/",
            "4 files read",
            6,
        ),
        st(
            "tool",
            "Look things up",
            "browser.search",
            "sources for the task",
            "9 pages opened",
            31,
        ),
        st(
            "ask",
            "Run a script",
            "terminal.run",
            "python build.py",
            "Script finished",
            12,
        ),
        st(
            "write",
            "Write it up",
            "files.write",
            "result.md",
            "1 page",
            14,
        ),
    ]
}

/// The run of a task as it stands: its plan with the finished share of steps done and the next one
/// running, waiting (Needs you), not started (Queued) or all done.
pub fn run_for(t: &Task) -> Vec<Step> {
    let mut v = if t.id == OPEN_TASK {
        run_plan()
    } else {
        generic_plan()
    };
    let n = v.len();
    let done = match t.status {
        Status::Done => n,
        Status::Queued => 0,
        _ => (t.steps.0 as usize * n)
            .checked_div(t.steps.1 as usize)
            .unwrap_or(0)
            .min(n - 1),
    };
    for (i, s) in v.iter_mut().enumerate() {
        s.state = if i < done {
            StepState::Done
        } else if i == done && t.status == Status::Running {
            StepState::Running
        } else if i == done && t.status == Status::NeedsYou {
            StepState::Waiting
        } else if i == done && t.status == Status::Failed {
            StepState::Skipped
        } else {
            StepState::Pending
        };
    }
    if t.status == Status::NeedsYou && v[done].kind != "ask" {
        v[done].kind = "ask";
    }
    v
}

/// The run as the screens open it: `done` steps finished, the next one running (or waiting when it
/// needs approval and `ask` is set).
pub fn run_at(done: usize, ask: bool) -> Vec<Step> {
    let mut v = run_plan();
    for (i, s) in v.iter_mut().enumerate() {
        s.state = if i < done {
            StepState::Done
        } else if i == done {
            if s.kind == "ask" && ask {
                StepState::Waiting
            } else {
                StepState::Running
            }
        } else {
            StepState::Pending
        };
    }
    v
}

/// Move the run on one step: the running step finishes and the next starts (an `ask` step waits for
/// approval). Returns false when nothing moved (paused on approval, or finished).
pub fn advance(steps: &mut [Step]) -> bool {
    let Some(i) = steps.iter().position(|s| s.state == StepState::Running) else {
        return false;
    };
    steps[i].state = StepState::Done;
    if let Some(n) = steps.get_mut(i + 1) {
        n.state = if n.kind == "ask" {
            StepState::Waiting
        } else {
            StepState::Running
        };
    }
    true
}

/// Answer the waiting step: allow runs it, deny skips it and moves on. Returns false if none waits.
pub fn approve(steps: &mut [Step], allow: bool) -> bool {
    let Some(i) = steps.iter().position(|s| s.state == StepState::Waiting) else {
        return false;
    };
    if allow {
        steps[i].state = StepState::Running;
    } else {
        steps[i].state = StepState::Skipped;
        if let Some(n) = steps.get_mut(i + 1) {
            n.state = StepState::Running;
        }
    }
    true
}

/// Finished (or skipped) steps and the total.
pub fn progress(steps: &[Step]) -> (usize, usize) {
    (
        steps
            .iter()
            .filter(|s| matches!(s.state, StepState::Done | StepState::Skipped))
            .count(),
        steps.len(),
    )
}

/// The run's status from its steps.
pub fn run_status(steps: &[Step]) -> Status {
    if steps.iter().any(|s| s.state == StepState::Waiting) {
        Status::NeedsYou
    } else if steps.iter().any(|s| s.state == StepState::Running) {
        Status::Running
    } else if steps.iter().all(|s| s.state == StepState::Pending) {
        Status::Queued
    } else {
        Status::Done
    }
}

/// "42s", "1m 12s".
pub fn fmt_secs(s: u32) -> String {
    if s < 60 {
        format!("{s}s")
    } else {
        format!("{}m {:02}s", s / 60, s % 60)
    }
}

/// Seconds spent on finished steps.
pub fn elapsed(steps: &[Step]) -> u32 {
    steps
        .iter()
        .filter(|s| s.state == StepState::Done)
        .map(|s| s.secs)
        .sum()
}

/// Artifacts: file name, size, the step that writes it.
pub const ARTIFACTS: [(&str, &str, usize); 4] = [
    ("brief.md", "2 KB", 1),
    ("desks.csv", "4 KB", 3),
    ("price-chart.png", "86 KB", 4),
    ("buying-note.md", "3 KB", 5),
];

/// The artifacts written so far.
pub fn artifacts(steps: &[Step]) -> Vec<(&'static str, &'static str)> {
    ARTIFACTS
        .iter()
        .filter(|a| steps.get(a.2).is_some_and(|s| s.state == StepState::Done))
        .map(|a| (a.0, a.1))
        .collect()
}

/// The comparison sheet's rows: desk, price, preset, warranty.
pub const SHEET: [(&str, &str, &str, &str); 6] = [
    ("Arbor Lift 2", "$549", "4 presets", "10 yr"),
    ("Fenwick Duo", "$589", "3 presets", "10 yr"),
    ("Corvo Basic", "$399", "None", "5 yr"),
    ("Halden Pro", "$599", "4 presets", "12 yr"),
    ("Loma Rise", "$479", "2 presets", "7 yr"),
    ("Tessel One", "$529", "3 presets", "10 yr"),
];

/// Lines the terminal shows for the run so far.
pub fn terminal_lines(steps: &[Step]) -> Vec<String> {
    let mut v = vec!["~/home-office $ taskdeck run".to_string()];
    for s in steps.iter() {
        match s.state {
            StepState::Done if !s.tool.is_empty() => {
                v.push(format!("› {} {}", s.tool, s.arg));
                v.push(format!("  ✓ {} ({})", s.out, fmt_secs(s.secs)));
            }
            StepState::Running if !s.tool.is_empty() => v.push(format!("› {} {}", s.tool, s.arg)),
            StepState::Waiting => v.push(format!("? waiting for approval: {}", s.arg)),
            StepState::Skipped => v.push(format!("  – skipped: {}", s.arg)),
            _ => {}
        }
    }
    v
}

/// The side panel's tabs: key and label.
pub const PANEL_TABS: [(&str, &str); 4] = [
    ("artifacts", "Artifacts"),
    ("files", "Files"),
    ("terminal", "Terminal"),
    ("browser", "Browser"),
];

/// The workspace folder tree: depth, name, is a folder.
pub const TREE: [(u8, &str, bool); 7] = [
    (0, "home-office", true),
    (1, "notes", true),
    (2, "brief.md", false),
    (1, "out", true),
    (2, "desks.csv", false),
    (2, "price-chart.png", false),
    (1, "chart.py", false),
];

/// Pages the browser visited: title and address.
pub const BROWSED: [(&str, &str); 3] = [
    ("Arbor Lift 2 · Arbor Furniture", "arbor.example/lift-2"),
    ("Halden Pro desk", "halden.example/pro"),
    ("Best standing desks 2026", "deskreview.example/best"),
];

// ─────────────────────────────────────────────────────────── Composer ──

/// Run modes for the composer chip: key, label, line.
pub const MODES: [(&str, &str, &str); 3] = [
    (
        "ask",
        "Ask first",
        "Asks before running commands or writing outside the folder",
    ),
    (
        "safe",
        "Auto for safe steps",
        "Reads, searches and edits in the folder run on their own",
    ),
    (
        "full",
        "Full auto",
        "Runs every step; you can stop it at any time",
    ),
];

/// Models for the composer chip: name, brand mark id, line.
pub const MODELS: [(&str, &str, &str); 4] = [
    ("Sonnet 5.5", "claude", "Balanced, careful with tools"),
    ("GPT-6", "openai", "Broad tool use"),
    ("Gemini 3 Pro", "gemini", "Long files and video"),
    ("DeepSeek V4", "deepseek", "Low cost"),
];

/// The + menu: key, label, glyph.
pub const ADD_MENU: [(&str, &str, &str); 4] = [
    ("files", "Attach files", "clip"),
    ("folder", "Add a folder", "folder"),
    ("plugins", "Plugins", "plug"),
    ("skills", "Skills", "spark"),
];

/// Starter chips: glyph, label, the prompt they fill in.
pub const SUGGEST: [(&str, &str, &str); 4] = [
    (
        "web",
        "Build a site",
        "Build a one-page site for a weekend bakery pop-up with the menu and opening hours.",
    ),
    (
        "chart",
        "Analyse data",
        "Find the three slowest pages in last week's traffic export and say why.",
    ),
    (
        "doc",
        "Write a report",
        "Turn these meeting notes into a one-page status report with risks and next steps.",
    ),
    (
        "cycle",
        "Automate a chore",
        "Every Monday at 9, collect open invoices into one sheet and email me the total.",
    ),
];

/// Plugins in the Plugins & skills popover: name, line, on by default.
pub const PLUGINS: [(&str, &str, bool); 4] = [
    ("Browser", "Opens pages and fills forms", true),
    ("Sheets", "Reads and writes spreadsheets", true),
    ("Calendar", "Reads your free time", false),
    ("Image maker", "Draws images from a prompt", false),
];

/// The start screen's headline.
pub fn headline(env: EnvMode) -> &'static str {
    match env {
        EnvMode::Local => "What should we get done today?",
        EnvMode::Cloud => "Hand it off. It keeps going in the cloud.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s9_taskdeck() {
        assert_eq!(view_of("taskdeck-board"), "board");
        assert_eq!(view_of("taskdeck-start"), "start");
        assert_eq!(split("approval").1, "approval");
        assert_eq!(split("loading").0, ScreenState::Loading);
        assert_eq!(states_for("taskdeck-board")[3].0, "menu");
        let mut l = sample_tasks();
        assert_eq!(counts(&l), (2, 1));
        assert_eq!(prompt_error("  "), Some("Describe the task first"));
        assert!(prompt_error("Plan a trip to the coast").is_none());
        let id = add_task(
            &mut l,
            "Plan a weekend trip to the coast. Keep it cheap.",
            EnvMode::Cloud,
            None,
        )
        .unwrap();
        assert_eq!(id, 10);
        assert_eq!(l[0].title, "Plan a weekend trip to the coast");
        assert_eq!(l[0].status, Status::Running);
        assert!(title_from(PROMPT).ends_with('…'));
        assert!(title_from(PROMPT).chars().count() <= 45);
        assert!(move_task(&mut l, 4, Status::Running));
        assert_eq!(l.iter().find(|t| t.id == 4).unwrap().steps.0, 0);
        assert!(card_acts(Status::Done).iter().all(|a| a.0 != "done"));
        assert!(delete_task(&mut l, 9));
        assert!(!delete_task(&mut l, 9));
        assert!(matches(&l[1], "standing"));
        assert!(matches(&l[1], "home office"));
        let mut r = run_at(3, true);
        assert_eq!(run_status(&r), Status::Running);
        assert!(advance(&mut r));
        assert_eq!(r[4].state, StepState::Waiting);
        assert_eq!(run_status(&r), Status::NeedsYou);
        assert!(!advance(&mut r));
        assert!(approve(&mut r, false));
        assert_eq!(r[4].state, StepState::Skipped);
        assert_eq!(r[5].state, StepState::Running);
        assert_eq!(progress(&r), (5, 6));
        assert!(advance(&mut r));
        assert_eq!(run_status(&r), Status::Done);
        assert_eq!(artifacts(&r).len(), 3);
        assert_eq!(fmt_secs(72), "1m 12s");
        assert!(terminal_lines(&run_at(3, false)).len() > 3);
        assert_eq!(Status::Failed.column(), 3);
        let all = sample_tasks();
        assert_eq!(run_status(&run_for(&all[1])), Status::NeedsYou);
        assert_eq!(run_status(&run_for(&all[3])), Status::Done);
        assert_eq!(run_status(&run_for(&all[4])), Status::Queued);
        assert_eq!(run_for(&all[0])[3].state, StepState::Running);
    }
}
