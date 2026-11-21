//! Screens S11 (Agent apps III): **Command Board**, a desktop app for an autonomous software agent
//! in the session-and-workspace pattern. Sessions in the sidebar (working, needs you, sleeping,
//! PR open, done); a session's chat where the agent reports progress and asks questions; a
//! workspace pane with the agent's Shell, Browser, Editor and Planner; a timeline you scrub to see
//! the workspace at any earlier moment (Live jumps back); and the pull request it opened with its
//! checks. Our own name and design, inspired by how such apps behave. Every name, repo, ticket and
//! company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const CB_APP: &str = "Command Board";

/// The state bar of Command Board.
pub const CB_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("pr", "Pull request"),
    ("planner", "Planner"),
];

/// Split a state key into the data state and the own state (`pr`, `planner` or "").
pub fn cb_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "pr" => (ScreenState::Sample, "pr"),
        "planner" => (ScreenState::Sample, "planner"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Repositories the agent can work in.
pub const CB_REPOS: [&str; 3] = ["lumen/reports-web", "lumen/billing-api", "lumen/mobile"];

/// Where a session stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SStatus {
    /// Working.
    Working,
    /// Asked you something.
    NeedsYou,
    /// Paused until woken.
    Sleeping,
    /// Opened a pull request.
    PrOpen,
    /// Finished.
    Done,
}

impl SStatus {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::NeedsYou => "Needs you",
            Self::Sleeping => "Sleeping",
            Self::PrOpen => "PR open",
            Self::Done => "Done",
        }
    }
    /// Tone key (`run`, `wait`, `idle`, `ok`).
    pub fn tone(self) -> &'static str {
        match self {
            Self::Working => "run",
            Self::NeedsYou => "wait",
            Self::Sleeping => "idle",
            Self::PrOpen | Self::Done => "ok",
        }
    }
}

/// A session in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct BSession {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`CB_REPOS`].
    pub repo: usize,
    /// Where it stands.
    pub status: SStatus,
    /// Last change.
    pub ago: String,
    /// Where it started: a brand mark id ("slack", "linear") or "" for the app.
    pub from: &'static str,
}

fn se(
    id: u32,
    title: &str,
    repo: usize,
    status: SStatus,
    ago: &str,
    from: &'static str,
) -> BSession {
    BSession {
        id,
        title: title.to_string(),
        repo,
        status,
        ago: ago.to_string(),
        from,
    }
}

/// The sample sessions (empty for loading and empty).
pub fn cb_sessions(st: ScreenState) -> Vec<BSession> {
    use SStatus::*;
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        se(
            1,
            "CSV export on the reports page",
            0,
            NeedsYou,
            "2m",
            "linear",
        ),
        se(2, "Upgrade the date library", 0, Working, "6m", ""),
        se(3, "Flaky checkout test", 1, Sleeping, "1h", "slack"),
        se(4, "Rate-limit the webhook endpoint", 1, PrOpen, "3h", ""),
        se(5, "Fix the dark-mode chart colours", 2, Done, "1d", "slack"),
    ]
}

/// Start a session from a prompt at the top of the list. Returns its id.
pub fn add_session(
    list: &mut Vec<BSession>,
    prompt: &str,
    repo: usize,
) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(
        0,
        se(
            id,
            &t,
            repo.min(CB_REPOS.len() - 1),
            SStatus::Working,
            "now",
            "",
        ),
    );
    Ok(id)
}

/// Sleep a working session, or wake a sleeping one. Returns the new status.
pub fn toggle_sleep(s: &mut BSession) -> SStatus {
    s.status = match s.status {
        SStatus::Sleeping => SStatus::Working,
        SStatus::Working | SStatus::NeedsYou => SStatus::Sleeping,
        x => x,
    };
    s.status
}

// ─────────────────────────────────────────────────────── Timeline ──

/// Which workspace tool an event used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// A command in the shell.
    Shell,
    /// A page in the browser.
    Browser,
    /// A file in the editor.
    Editor,
    /// The plan.
    Planner,
}

/// The workspace tabs, in order.
pub const TOOLS: [Tool; 4] = [Tool::Shell, Tool::Browser, Tool::Editor, Tool::Planner];

impl Tool {
    /// Tab label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Shell => "Shell",
            Self::Browser => "Browser",
            Self::Editor => "Editor",
            Self::Planner => "Planner",
        }
    }
    /// Glyph.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Shell => "term",
            Self::Browser => "globe",
            Self::Editor => "doc",
            Self::Planner => "list",
        }
    }
}

/// An event on the session's timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// Elapsed time "m:ss".
    pub at: &'static str,
    /// The tool.
    pub tool: Tool,
    /// What happened.
    pub label: &'static str,
    /// The command, address or file, and output lines.
    pub body: Vec<&'static str>,
}

fn ev(at: &'static str, tool: Tool, label: &'static str, body: &[&'static str]) -> Event {
    Event {
        at,
        tool,
        label,
        body: body.to_vec(),
    }
}

/// The open session's timeline.
pub fn cb_events(st: ScreenState) -> Vec<Event> {
    use Tool::*;
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        ev("0:04", Planner, "Wrote a plan", &[]),
        ev(
            "0:31",
            Shell,
            "Cloned and installed",
            &[
                "git clone lumen/reports-web",
                "pnpm install",
                "Done in 18.2s",
            ],
        ),
        ev(
            "1:12",
            Editor,
            "Read the reports table",
            &["src/reports/ReportsTable.tsx"],
        ),
        ev(
            "2:40",
            Browser,
            "Read the CSV docs",
            &["docs.papaparse.example/unparse"],
        ),
        ev(
            "4:05",
            Editor,
            "Added exportCsv()",
            &["src/reports/exportCsv.ts"],
        ),
        ev(
            "5:18",
            Shell,
            "Ran the tests",
            &[
                "pnpm test reports",
                "✓ 24 passed",
                "✓ exportCsv escapes quotes and commas",
            ],
        ),
        ev(
            "6:02",
            Browser,
            "Checked the page",
            &["localhost:5173/reports"],
        ),
    ]
}

/// Clamp a scrub position to the timeline (None = live, the last event).
pub fn scrub_at(n: usize, pos: Option<usize>) -> usize {
    match pos {
        Some(p) if n > 0 => p.min(n - 1),
        _ => n.saturating_sub(1),
    }
}

/// The latest event of a tool at or before `at`.
pub fn last_of(events: &[Event], tool: Tool, at: usize) -> Option<usize> {
    (0..events.len().min(at + 1))
        .rev()
        .find(|i| events[*i].tool == tool)
}

/// Shell lines up to `at`: (is a command, text).
pub fn shell_upto(events: &[Event], at: usize) -> Vec<(bool, &'static str)> {
    let mut out = Vec::new();
    for e in events.iter().take(at + 1).filter(|e| e.tool == Tool::Shell) {
        for (k, l) in e.body.iter().enumerate() {
            let cmd = k == 0
                || (k == 1 && e.body.len() == 3 && !l.starts_with('✓') && !l.starts_with("Done"));
            out.push((cmd, *l));
        }
    }
    out
}

/// Files the editor shows: path and lines.
pub const CB_FILES: [(&str, &[&str]); 2] = [
    (
        "src/reports/ReportsTable.tsx",
        &[
            "export function ReportsTable({ rows, filters }: Props) {",
            "  const shown = applyFilters(rows, filters);",
            "  return (",
            "    <Table rows={shown} columns={COLUMNS} />",
            "  );",
            "}",
        ],
    ),
    (
        "src/reports/exportCsv.ts",
        &[
            "const esc = (v: string) =>",
            "  /[\",\\n]/.test(v) ? `\"${v.replace(/\"/g, '\"\"')}\"` : v;",
            "",
            "export function exportCsv(rows: Row[], cols = COLUMNS) {",
            "  const head = cols.map((c) => esc(c.label)).join(',');",
            "  const body = rows.map((r) => cols.map((c) => esc(String(r[c.key] ?? ''))).join(','));",
            "  return [head, ...body].join('\\n');",
            "}",
        ],
    ),
];

/// The lines of an editor file (empty when unknown).
pub fn file_lines(path: &str) -> &'static [&'static str] {
    CB_FILES
        .iter()
        .find(|f| f.0 == path)
        .map(|f| f.1)
        .unwrap_or(&[])
}

/// The plan: steps and the event that finishes each.
pub const CB_PLAN: [(&str, usize); 5] = [
    ("Find where the reports table gets its rows", 2),
    ("Write exportCsv() with quoting for commas and quotes", 4),
    ("Add an Export CSV button next to Filters", 4),
    ("Test: quotes, commas, empty rows", 5),
    ("Open a PR and post the preview link", 7),
];

/// A plan step's state at `at`: "done", "run" or "todo".
pub fn plan_state(i: usize, at: usize) -> &'static str {
    let done_at = CB_PLAN.get(i).map(|p| p.1).unwrap_or(usize::MAX);
    if at >= done_at {
        "done"
    } else if i == 0 || CB_PLAN.get(i - 1).is_some_and(|p| at >= p.1) {
        "run"
    } else {
        "todo"
    }
}

// ─────────────────────────────────────────────────────────── Chat ──

/// A chat message: from the agent (true) or you, and text.
pub type Msg = (bool, String);

/// The open session's chat.
pub fn cb_msgs(st: ScreenState) -> Vec<Msg> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    [
        (false, "Add a CSV export to the reports page (LUM-212). Same columns as the table, respect the current filters."),
        (true, "On it. I'll add an Export CSV button next to Filters and build the file from the rows the table is showing."),
        (true, "exportCsv() is in and the tests pass (24, including quotes and commas)."),
    ]
    .iter()
    .map(|(a, t)| (*a, t.to_string()))
    .collect()
}

/// The question the agent is waiting on, with its two quick answers.
pub const CB_ASK: (&str, [&str; 2]) = (
    "Archived projects are hidden from the table by default. Should the export include them?",
    ["Yes, include archived", "No, match the table"],
);

/// Send a reply: it joins the chat with the agent's acknowledgement. Errors on a blank reply.
pub fn reply(msgs: &mut Vec<Msg>, text: &str) -> Result<(), &'static str> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Write a reply first");
    }
    msgs.push((false, t.to_string()));
    msgs.push((
        true,
        "Got it. I'll fold that in and update the PR.".to_string(),
    ));
    Ok(())
}

// ──────────────────────────────────────────────────────────── PR ──

/// A pull request the agent opened.
#[derive(Clone, Debug, PartialEq)]
pub struct Pr {
    /// Number.
    pub number: u32,
    /// Title.
    pub title: &'static str,
    /// Branch.
    pub branch: &'static str,
    /// Lines added and removed.
    pub stat: (u32, u32),
    /// Checks: name and state ("ok", "run", "bad").
    pub checks: Vec<(&'static str, &'static str)>,
}

/// The sample PR; `done` finishes every check.
pub fn sample_pr(done: bool) -> Pr {
    Pr {
        number: 482,
        title: "Export the reports table as CSV",
        branch: "agent/csv-export",
        stat: (96, 4),
        checks: vec![
            ("Build", "ok"),
            ("Unit tests", "ok"),
            ("Lint", "ok"),
            ("Preview deploy", if done { "ok" } else { "run" }),
        ],
    }
}

/// Checks passed and total.
pub fn checks_passed(pr: &Pr) -> (usize, usize) {
    (
        pr.checks.iter().filter(|c| c.1 == "ok").count(),
        pr.checks.len(),
    )
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Command Board shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandBoardData {
    /// Sessions in the sidebar; the first one opens.
    pub sessions: Vec<BSession>,
    /// The open session's timeline.
    pub events: Vec<Event>,
    /// The open session's chat.
    pub msgs: Vec<Msg>,
    /// The pull request, once opened.
    pub pr: Option<Pr>,
}

impl CommandBoardData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        let samp = st == ScreenState::Sample;
        Self {
            sessions: cb_sessions(st),
            events: cb_events(st),
            msgs: cb_msgs(st),
            pr: if samp { Some(sample_pr(false)) } else { None },
        }
    }
}

impl Default for CommandBoardData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s11_command_board() {
        assert_eq!(cb_split("pr").1, "pr");
        let mut l = cb_sessions(ScreenState::Sample);
        assert!(add_session(&mut l, " ", 0).is_err());
        let id = add_session(&mut l, "Add dark mode to the invoices page", 1).unwrap();
        assert_eq!((l[0].id, l[0].status), (id, SStatus::Working));
        assert_eq!(toggle_sleep(&mut l[0]), SStatus::Sleeping);
        assert_eq!(toggle_sleep(&mut l[0]), SStatus::Working);
        let e = cb_events(ScreenState::Sample);
        assert_eq!(scrub_at(e.len(), None), 6);
        assert_eq!(scrub_at(e.len(), Some(99)), 6);
        assert_eq!(last_of(&e, Tool::Editor, 3), Some(2));
        assert_eq!(last_of(&e, Tool::Browser, 2), None);
        let sh = shell_upto(&e, 6);
        assert_eq!(sh.iter().filter(|x| x.0).count(), 3);
        assert_eq!(shell_upto(&e, 0).len(), 0);
        assert_eq!(plan_state(0, 0), "run");
        assert_eq!(plan_state(1, 6), "done");
        assert_eq!(plan_state(4, 6), "run");
        assert_eq!(plan_state(2, 1), "todo");
        let mut m = cb_msgs(ScreenState::Sample);
        assert!(reply(&mut m, "  ").is_err());
        reply(&mut m, "Match the table").unwrap();
        assert_eq!(m.len(), 5);
        assert_eq!(checks_passed(&sample_pr(false)), (3, 4));
        assert_eq!(checks_passed(&sample_pr(true)), (4, 4));
        assert!(CommandBoardData::for_state(ScreenState::Empty).pr.is_none());
    }
}
