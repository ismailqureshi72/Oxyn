//! Screens S16 (Agent apps IV): **Threadbar**, a code editor with an agent panel. Threads in a
//! sidebar grouped by project (agent threads and terminal threads side by side); the open thread
//! with the agent's tool calls (read, edit with a diff to keep or reject, run); a composer with a
//! profile (Write, Ask, Minimal), a model and an agent picker that also lists outside agents; and
//! a Project pane on the right with Files (a tree with git letters) and Git (stage and commit).
//! Our own name and design, inspired by how such editors behave. Every name, file, number and
//! company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const TB_APP: &str = "Threadbar";

/// The state bar of Threadbar.
pub const TB_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("terminal", "Terminal thread"),
    ("git", "Git"),
];

/// Split a state key into the data state and the own state (`terminal`, `git` or "").
pub fn tb_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "terminal" => (ScreenState::Sample, "terminal"),
        "git" => (ScreenState::Sample, "git"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Profiles: label, line and glyph. Write edits and runs; Ask only reads; Minimal has no tools.
pub const TB_PROFILES: [(&str, &str, &str); 3] = [
    ("Write", "Edits files and runs commands", "pen"),
    ("Ask", "Reads the project, changes nothing", "chat"),
    ("Minimal", "No tools, just the model", "spark"),
];

/// Models: name and brand mark id.
pub const TB_MODELS: [(&str, &str); 3] = [
    ("Sonnet 5.5", "claude"),
    ("GPT-6", "openai"),
    ("Gemini 3 Pro", "gemini"),
];

/// Agents the panel can run: name, line and brand mark id ("" is Threadbar's own agent).
pub const TB_AGENTS: [(&str, &str, &str); 4] = [
    ("Threadbar agent", "Built in, uses the model above", ""),
    (
        "Claude Code",
        "Outside agent over the agent protocol",
        "claudecode",
    ),
    ("Codex", "Outside agent over the agent protocol", "codex"),
    (
        "GitHub Copilot",
        "Outside agent over the agent protocol",
        "githubcopilot",
    ),
];

/// The project pane's tabs: key and label.
pub const TB_TABS: [(&str, &str); 2] = [("files", "Files"), ("git", "Git")];

/// A thread in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct TbThread {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Project it belongs to.
    pub project: &'static str,
    /// A terminal thread (a shell) rather than an agent thread.
    pub terminal: bool,
    /// Working right now.
    pub running: bool,
    /// Last activity, an absolute ISO time.
    pub at: &'static str,
}

fn th(
    id: u32,
    title: &str,
    project: &'static str,
    terminal: bool,
    running: bool,
    at: &'static str,
) -> TbThread {
    TbThread {
        id,
        title: title.to_string(),
        project,
        terminal,
        running,
        at,
    }
}

/// The projects the sample threads belong to.
pub const TB_PROJECTS: [&str; 2] = ["ledger-api", "field-notes"];

/// The sample threads (empty for loading and empty).
pub fn tb_threads(st: ScreenState) -> Vec<TbThread> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        th(
            1,
            "Retry failed webhooks with backoff",
            "ledger-api",
            false,
            true,
            "2026-11-19 09:42",
        ),
        th(
            2,
            "cargo test --workspace",
            "ledger-api",
            true,
            false,
            "2026-11-19 09:38",
        ),
        th(
            3,
            "Explain the settlement job",
            "ledger-api",
            false,
            false,
            "2026-11-18 17:10",
        ),
        th(
            4,
            "Dark mode for the map view",
            "field-notes",
            false,
            false,
            "2026-11-18 11:05",
        ),
        th(
            5,
            "pnpm dev",
            "field-notes",
            true,
            false,
            "2026-11-17 16:20",
        ),
    ]
}

/// Start a thread from a prompt at the top of the first project; it runs at once.
pub fn add_thread(list: &mut Vec<TbThread>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(
        0,
        th(id, &t, TB_PROJECTS[0], false, true, "2026-11-19 09:45"),
    );
    Ok(id)
}

/// The open thread's prompt.
pub const TB_PROMPT: &str = "Webhook deliveries that fail are dropped. Retry them with exponential backoff, at most five tries, and log each attempt.";

/// A tool call in the thread: glyph, verb, object, detail.
pub const TB_CALLS: [(&str, &str, &str, &str); 4] = [
    (
        "search",
        "Searched",
        "deliver_webhook",
        "3 matches in 2 files",
    ),
    ("file", "Read", "src/webhooks/deliver.rs", "lines 1–88"),
    ("pen", "Edited", "src/webhooks/deliver.rs", "+18 −4"),
    ("term", "Ran", "cargo test webhooks", "12 passed in 4.1 s"),
];

/// The edit's diff: (sign, line). `+` added, `-` removed, ' ' context.
pub const TB_DIFF: [(char, &str); 9] = [
    (
        ' ',
        "pub async fn deliver(hook: &Hook, body: &[u8]) -> Result<()> {",
    ),
    (
        '-',
        "    let res = client().post(&hook.url).body(body).send().await?;",
    ),
    ('-', "    res.error_for_status()?;"),
    ('+', "    for attempt in 0..MAX_TRIES {"),
    ('+', "        match try_once(hook, body).await {"),
    ('+', "            Ok(()) => return Ok(()),"),
    ('+', "            Err(e) => log_attempt(hook, attempt, &e),"),
    ('+', "        }"),
    ('+', "        sleep(backoff(attempt)).await;"),
];

/// The backoff before retry `attempt` (0-based), in seconds: 1, 2, 4, 8, 16, capped at 60.
pub fn backoff_secs(attempt: u32) -> u32 {
    (1u32 << attempt.min(6)).min(60)
}

/// The agent's closing line.
pub const TB_DONE: &str = "Deliveries now retry up to five times, waiting 1, 2, 4, 8 and 16 s, and every attempt is logged. The webhook tests pass.";

/// The terminal thread's transcript: (is a command, text).
pub const TB_TERM: [(bool, &str); 6] = [
    (true, "cargo test --workspace"),
    (false, "   Compiling ledger-api v0.4.2"),
    (false, "running 48 tests"),
    (false, "test webhooks::retries_five_times ... ok"),
    (false, "test settlement::rounds_half_even ... ok"),
    (
        false,
        "test result: ok. 48 passed; 0 failed; finished in 6.3 s",
    ),
];

/// A file in the project tree: path, depth, git letter (`M`, `A`, `?` or ' ').
pub const TB_TREE: [(&str, usize, char); 9] = [
    ("src", 0, ' '),
    ("webhooks", 1, ' '),
    ("deliver.rs", 2, 'M'),
    ("backoff.rs", 2, 'A'),
    ("mod.rs", 2, ' '),
    ("settlement.rs", 1, ' '),
    ("tests", 0, ' '),
    ("webhooks.rs", 1, 'M'),
    ("Cargo.toml", 0, ' '),
];

/// A changed file in Git: path, lines added and removed, staged.
#[derive(Clone, Debug, PartialEq)]
pub struct TbChange {
    /// Path.
    pub path: &'static str,
    /// Lines added.
    pub add: u32,
    /// Lines removed.
    pub del: u32,
    /// Staged for the next commit.
    pub staged: bool,
}

/// The sample changes (empty for loading and empty).
pub fn tb_changes(st: ScreenState) -> Vec<TbChange> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        TbChange {
            path: "src/webhooks/deliver.rs",
            add: 18,
            del: 4,
            staged: true,
        },
        TbChange {
            path: "src/webhooks/backoff.rs",
            add: 22,
            del: 0,
            staged: true,
        },
        TbChange {
            path: "tests/webhooks.rs",
            add: 31,
            del: 2,
            staged: false,
        },
    ]
}

/// Staged files and the totals over them.
pub fn staged_sum(ch: &[TbChange]) -> (usize, u32, u32) {
    ch.iter()
        .filter(|c| c.staged)
        .fold((0, 0, 0), |(n, a, d), c| (n + 1, a + c.add, d + c.del))
}

/// Why a commit cannot be made yet, or `None` when it can.
pub fn commit_error(ch: &[TbChange], msg: &str) -> Option<&'static str> {
    if staged_sum(ch).0 == 0 {
        Some("Stage at least one file")
    } else if msg.trim().chars().count() < 4 {
        Some("Write a commit message")
    } else {
        None
    }
}

/// The commit message the agent suggests.
pub const TB_COMMIT: &str = "webhooks: retry failed deliveries with backoff";

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Threadbar shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadbarData {
    /// Threads in the sidebar; the first one opens.
    pub threads: Vec<TbThread>,
    /// Changed files in the Git tab.
    pub changes: Vec<TbChange>,
    /// The branch the project is on.
    pub branch: String,
}

impl ThreadbarData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            threads: tb_threads(st),
            changes: tb_changes(st),
            branch: "webhook-retries".to_string(),
        }
    }
}

impl Default for ThreadbarData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_threadbar() {
        assert_eq!(tb_split("git").1, "git");
        assert_eq!(tb_split("loading").0, ScreenState::Loading);
        let mut l = tb_threads(ScreenState::Sample);
        assert!(add_thread(&mut l, "").is_err());
        let id = add_thread(&mut l, "Add a health check endpoint").unwrap();
        assert_eq!((l[0].id, l[0].running), (id, true));
        assert_eq!(backoff_secs(0), 1);
        assert_eq!(backoff_secs(4), 16);
        assert_eq!(backoff_secs(9), 60);
        let mut ch = tb_changes(ScreenState::Sample);
        assert_eq!(staged_sum(&ch), (2, 40, 4));
        assert_eq!(commit_error(&ch, ""), Some("Write a commit message"));
        assert_eq!(commit_error(&ch, TB_COMMIT), None);
        for c in &mut ch {
            c.staged = false;
        }
        assert_eq!(
            commit_error(&ch, TB_COMMIT),
            Some("Stage at least one file")
        );
        assert!(
            ThreadbarData::for_state(ScreenState::Empty)
                .threads
                .is_empty()
        );
    }
}
