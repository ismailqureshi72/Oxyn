//! Screens S16 (Agent apps IV): **Repo Desk**, a chat-first desktop app for an agent that works
//! on a code host. A nav with Home, Inbox (issues and pull requests that need you), Projects and
//! the sessions list, each session on its own branch; a session where the chat sits next to
//! canvases the agent shares with you (Plan, Pull request, Terminal, Browser); an Inbox with
//! filters and Start session; and a Project page with a goal and its progress. No code editor.
//! Our own name and design, inspired by how such apps behave. Every name, repo, number and
//! company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const RD_APP: &str = "Repo Desk";

/// The state bar of Repo Desk.
pub const RD_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("inbox", "Inbox"),
    ("project", "Project"),
];

/// Split a state key into the data state and the own state (`inbox`, `project` or "").
pub fn rd_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "inbox" => (ScreenState::Sample, "inbox"),
        "project" => (ScreenState::Sample, "project"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Nav items: key, label, glyph.
pub const RD_NAV: [(&str, &str, &str); 3] = [
    ("home", "Home", "home"),
    ("inbox", "Inbox", "inbox"),
    ("project", "Projects", "target"),
];

/// Models: name and brand mark id.
pub const RD_MODELS: [(&str, &str); 3] = [
    ("GPT-6", "openai"),
    ("Sonnet 5.5", "claude"),
    ("Gemini 3 Pro", "gemini"),
];

/// Canvases beside the chat: key, label, glyph.
pub const RD_CANVASES: [(&str, &str, &str); 4] = [
    ("plan", "Plan", "tasks"),
    ("pr", "Pull request", "branch"),
    ("terminal", "Terminal", "term"),
    ("browser", "Browser", "globe"),
];

/// A session in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct RdSession {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Repository.
    pub repo: &'static str,
    /// Its own branch.
    pub branch: String,
    /// Working right now.
    pub running: bool,
}

fn se(id: u32, title: &str, repo: &'static str, branch: &str, running: bool) -> RdSession {
    RdSession {
        id,
        title: title.to_string(),
        repo,
        branch: branch.to_string(),
        running,
    }
}

/// The sample sessions (empty for loading and empty).
pub fn rd_sessions(st: ScreenState) -> Vec<RdSession> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        se(
            1,
            "Add CSV export to reports",
            "acorn/insights",
            "agent/csv-export",
            true,
        ),
        se(
            2,
            "Fix timezone in digest emails",
            "acorn/notify",
            "agent/digest-tz",
            false,
        ),
        se(
            3,
            "Triage the November bug list",
            "acorn/insights",
            "agent/triage-nov",
            false,
        ),
        se(4, "Upgrade to Node 24", "acorn/web", "agent/node-24", false),
    ]
}

/// Start a session from a prompt at the top; it works on a branch of its own.
pub fn add_session(list: &mut Vec<RdSession>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    let b = format!("agent/{}", branch_slug(&t));
    list.insert(0, se(id, &t, "acorn/insights", &b, true));
    Ok(id)
}

/// A short branch slug from a title (first three words, lowercase, hyphens).
pub fn branch_slug(title: &str) -> String {
    title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(3)
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/// The open session's prompt.
pub const RD_PROMPT: &str = "Add a CSV export to the monthly report page. Keep the column order of the table and include the totals row.";

/// The agent's reply.
pub const RD_REPLY: &str = "I opened a plan in the Plan canvas. The export reuses the table's column list so the order can't drift, and the totals row is written last. I'll open a draft pull request once the tests pass.";

/// The plan: step text and done at the start.
pub const RD_PLAN: [(&str, bool); 5] = [
    ("Find where the report table defines its columns", true),
    ("Add an export endpoint that streams CSV", true),
    ("Write the totals row last", true),
    ("Add an Export button to the report header", false),
    ("Tests for column order and totals", false),
];

/// The pull request's checks: name, tone (`ok`, `run`, `bad`).
pub const RD_CHECKS: [(&str, &str); 3] = [("build", "ok"), ("unit tests", "run"), ("lint", "ok")];

/// The pull request: number, title, files, lines added and removed.
pub const RD_PR: (u32, &str, u32, u32, u32) =
    (418, "Reports: export the monthly table as CSV", 4, 126, 9);

/// The terminal canvas: (is a command, text).
pub const RD_TERM: [(bool, &str); 4] = [
    (true, "npm test -- reports"),
    (false, "PASS  src/reports/export.test.ts (6 tests)"),
    (false, "PASS  src/reports/table.test.ts (11 tests)"),
    (false, "Tests: 17 passed, 17 total · 3.8 s"),
];

/// The browser canvas: address and the export it shows.
pub const RD_BROWSER: (&str, [&str; 4]) = (
    "localhost:5173/reports/2026-10",
    [
        "region,orders,revenue",
        "North,1204,48210.50",
        "South,986,39102.00",
        "Total,2190,87312.50",
    ],
);

/// Kinds in the Inbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// An issue assigned to you.
    Issue,
    /// A pull request of yours.
    Pull,
    /// A review someone asked of you.
    Review,
}

impl Kind {
    /// The kind's label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::Pull => "Pull request",
            Self::Review => "Review",
        }
    }
}

/// Inbox filters: key and label.
pub const RD_FILTERS: [(&str, &str); 4] = [
    ("all", "All"),
    ("issue", "Issues"),
    ("pull", "Pull requests"),
    ("review", "Reviews"),
];

/// An Inbox item.
#[derive(Clone, Debug, PartialEq)]
pub struct RdItem {
    /// Stable id.
    pub id: u32,
    /// Kind.
    pub kind: Kind,
    /// Repository and number ("acorn/web #77").
    pub refs: &'static str,
    /// Title.
    pub title: &'static str,
    /// Why it needs you.
    pub why: &'static str,
    /// When it arrived (ISO date).
    pub at: &'static str,
}

/// The sample Inbox (empty for loading and empty).
pub fn rd_inbox(st: ScreenState) -> Vec<RdItem> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    let it = |id, kind, refs, title, why, at| RdItem {
        id,
        kind,
        refs,
        title,
        why,
        at,
    };
    vec![
        it(
            1,
            Kind::Issue,
            "acorn/insights #391",
            "Report totals off by one cent",
            "Assigned to you",
            "2026-11-19",
        ),
        it(
            2,
            Kind::Review,
            "acorn/web #77",
            "Cache the font files at the edge",
            "Review requested",
            "2026-11-19",
        ),
        it(
            3,
            Kind::Pull,
            "acorn/notify #142",
            "Digest emails in the reader's time zone",
            "Checks failed",
            "2026-11-18",
        ),
        it(
            4,
            Kind::Issue,
            "acorn/web #80",
            "Dark mode loses the chart colours",
            "Mentioned you",
            "2026-11-17",
        ),
        it(
            5,
            Kind::Pull,
            "acorn/insights #415",
            "Paginate the audit table",
            "Approved, ready to merge",
            "2026-11-16",
        ),
    ]
}

/// Items that pass a filter key (`all`, `issue`, `pull`, `review`).
pub fn filter_inbox(items: &[RdItem], f: &str) -> Vec<RdItem> {
    items
        .iter()
        .filter(|i| match f {
            "issue" => i.kind == Kind::Issue,
            "pull" => i.kind == Kind::Pull,
            "review" => i.kind == Kind::Review,
            _ => true,
        })
        .cloned()
        .collect()
}

/// The project page: name, goal, due date (ISO).
pub const RD_GOAL: (&str, &str, &str) = (
    "Reports v2",
    "Every report can be exported and shared, and loads in under a second.",
    "2026-12-15",
);

/// The project's milestones: text and done.
pub const RD_MILESTONES: [(&str, bool); 4] = [
    ("CSV export", false),
    ("Share link with expiry", true),
    ("Report loads under 1 s (p95)", false),
    ("PDF export", false),
];

/// Done and total milestones, and the percent done.
pub fn goal_progress(m: &[(&str, bool)]) -> (usize, usize, u32) {
    let d = m.iter().filter(|x| x.1).count();
    let n = m.len();
    (d, n, (d * 100).checked_div(n).unwrap_or(0) as u32)
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Repo Desk shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct RepoDeskData {
    /// Sessions in the sidebar; the first one opens.
    pub sessions: Vec<RdSession>,
    /// Issues and pull requests that need you.
    pub inbox: Vec<RdItem>,
}

impl RepoDeskData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            sessions: rd_sessions(st),
            inbox: rd_inbox(st),
        }
    }
}

impl Default for RepoDeskData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_repo_desk() {
        assert_eq!(rd_split("inbox").1, "inbox");
        let mut l = rd_sessions(ScreenState::Sample);
        assert!(add_session(&mut l, " ").is_err());
        add_session(&mut l, "Add retries to the export job").unwrap();
        assert_eq!(l[0].branch, "agent/add-retries-to");
        let i = rd_inbox(ScreenState::Sample);
        assert_eq!(filter_inbox(&i, "all").len(), 5);
        assert_eq!(filter_inbox(&i, "pull").len(), 2);
        assert_eq!(filter_inbox(&i, "review").len(), 1);
        assert_eq!(goal_progress(&RD_MILESTONES), (1, 4, 25));
        assert_eq!(Kind::Review.label(), "Review");
        assert!(RepoDeskData::for_state(ScreenState::Empty).inbox.is_empty());
    }
}
