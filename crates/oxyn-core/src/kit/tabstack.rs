//! Screens S16 (Agent apps IV): **Tabstack**, an agent-first terminal. Vertical tabs, one per
//! shell or agent session, each with its agent's mark, branch, worktree and pull request; a
//! terminal of blocks (each command with its output, exit status and time) where the agent's turns
//! sit between the commands; a command the agent wants to run waits for Run / Edit / Deny; an
//! input that switches between Terminal and Agent with a profile (Default, Auto, Prod) and model;
//! and a pane with Agents (local and cloud runs) and Review (the session's diff). Our own name and
//! design, inspired by how such terminals behave. Every name, file, number and company is
//! fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const TS_APP: &str = "Tabstack";

/// The state bar of Tabstack.
pub const TS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("approval", "Needs approval"),
    ("review", "Review"),
];

/// Split a state key into the data state and the own state (`approval`, `review` or "").
pub fn ts_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "approval" => (ScreenState::Sample, "approval"),
        "review" => (ScreenState::Sample, "review"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Profiles: label and line. Default asks before writes; Auto runs safe commands; Prod asks for
/// everything.
pub const TS_PROFILES: [(&str, &str, &str); 3] = [
    ("Default", "Asks before it writes or runs", "shield"),
    ("Auto", "Runs read-only commands on its own", "bolt"),
    ("Prod", "Asks before every command", "lock"),
];

/// Models: name and brand mark id.
pub const TS_MODELS: [(&str, &str); 3] = [
    ("Sonnet 5.5", "claude"),
    ("GPT-6", "openai"),
    ("Gemini 3 Pro", "gemini"),
];

/// The pane's tabs: key and label.
pub const TS_PANE: [(&str, &str); 2] = [("agents", "Agents"), ("review", "Review")];

/// Where a tab's session stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsState {
    /// A plain shell.
    Shell,
    /// The agent is working.
    Working,
    /// The agent waits on you.
    Waiting,
    /// The agent finished.
    Done,
}

impl TsState {
    /// The status tone (`run`, `wait`, `ok` or "").
    pub fn tone(self) -> &'static str {
        match self {
            Self::Shell => "",
            Self::Working => "run",
            Self::Waiting => "wait",
            Self::Done => "ok",
        }
    }
}

/// A vertical tab.
#[derive(Clone, Debug, PartialEq)]
pub struct TsTab {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// The agent's brand mark id ("" for a plain shell or Tabstack's own agent).
    pub agent: &'static str,
    /// Git branch.
    pub branch: &'static str,
    /// Worktree folder.
    pub worktree: &'static str,
    /// Pull request number, 0 for none.
    pub pr: u32,
    /// Where it stands.
    pub state: TsState,
}

fn tab(
    id: u32,
    title: &str,
    agent: &'static str,
    branch: &'static str,
    worktree: &'static str,
    pr: u32,
    state: TsState,
) -> TsTab {
    TsTab {
        id,
        title: title.to_string(),
        agent,
        branch,
        worktree,
        pr,
        state,
    }
}

/// The sample tabs (empty for loading and empty).
pub fn ts_tabs(st: ScreenState) -> Vec<TsTab> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        tab(
            1,
            "Fix flaky upload test",
            "",
            "fix/upload-flake",
            "~/wt/upload-flake",
            0,
            TsState::Working,
        ),
        tab(
            2,
            "Migrate config to TOML",
            "claudecode",
            "chore/toml-config",
            "~/wt/toml-config",
            214,
            TsState::Waiting,
        ),
        tab(
            3,
            "Bump the image crate",
            "codex",
            "deps/image-0.26",
            "~/wt/image-bump",
            212,
            TsState::Done,
        ),
        tab(4, "zsh", "", "main", "~/src/harbor", 0, TsState::Shell),
    ]
}

/// Open a new agent tab from a prompt at the top; it starts working.
pub fn add_tab(list: &mut Vec<TsTab>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(
        0,
        tab(
            id,
            &t,
            "",
            "agent/new-task",
            "~/wt/new-task",
            0,
            TsState::Working,
        ),
    );
    Ok(id)
}

/// Tabs waiting on you (the badge on the app's bell).
pub fn waiting(list: &[TsTab]) -> usize {
    list.iter().filter(|t| t.state == TsState::Waiting).count()
}

/// A block in the terminal.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// A command you ran: command, output lines, exit code, time in ms.
    Cmd(String, Vec<String>, i32, u32),
    /// Your prompt to the agent.
    Ask(String),
    /// The agent's reply.
    Say(String),
}

/// The open tab's sample blocks.
pub fn ts_blocks() -> Vec<Block> {
    let s = |x: &[&str]| x.iter().map(|l| l.to_string()).collect::<Vec<_>>();
    vec![
        Block::Cmd(
            "cargo test upload".to_string(),
            s(&[
                "running 9 tests",
                "test upload::resumes_after_drop ... FAILED",
                "thread 'resumes_after_drop' panicked: timed out after 2000 ms",
                "test result: FAILED. 8 passed; 1 failed",
            ]),
            101,
            6_240,
        ),
        Block::Ask("This test fails about one run in five. Find out why and fix it.".to_string()),
        Block::Say(
            "The test waits a fixed 2 s for the retry, but the retry timer starts only after the \
             socket closes, which can take longer on a busy machine. I'll wait on the retry event \
             instead of the clock."
                .to_string(),
        ),
        Block::Cmd(
            "rg -n \"sleep\\(\" tests/upload.rs".to_string(),
            s(&["42:    sleep(Duration::from_millis(2000)).await;"]),
            0,
            38,
        ),
    ]
}

/// The command the agent asks to run next.
pub const TS_PENDING: &str = "cargo test upload -- --test-threads 8 --nocapture";

/// The answer to a typed command in the sample terminal: output lines and exit code.
pub fn run_cmd(cmd: &str) -> (Vec<String>, i32) {
    let c = cmd.trim();
    let out: (&[&str], i32) = match c.split_whitespace().next().unwrap_or("") {
        "" => (&[], 0),
        "ls" => (&["Cargo.toml  src  tests  README.md"], 0),
        "pwd" => (&["~/wt/upload-flake"], 0),
        "git" => (
            &[
                "On branch fix/upload-flake",
                "1 file changed, 9 insertions(+), 3 deletions(-)",
            ],
            0,
        ),
        "cargo" => (
            &["running 9 tests", "test result: ok. 9 passed; 0 failed"],
            0,
        ),
        "echo" => return (vec![c.trim_start_matches("echo").trim().to_string()], 0),
        _ => (&["command not found in this demo"], 127),
    };
    (out.0.iter().map(|l| l.to_string()).collect(), out.1)
}

/// A run in the Agents pane: title, where (Local or Cloud), state, started (ISO).
pub const TS_RUNS: [(&str, &str, TsState, &str); 4] = [
    (
        "Fix flaky upload test",
        "Local",
        TsState::Working,
        "2026-11-19 10:02",
    ),
    (
        "Migrate config to TOML",
        "Local",
        TsState::Waiting,
        "2026-11-19 09:31",
    ),
    (
        "Bump the image crate",
        "Cloud",
        TsState::Done,
        "2026-11-19 08:12",
    ),
    (
        "Nightly dependency audit",
        "Cloud",
        TsState::Done,
        "2026-11-19 03:00",
    ),
];

/// The session's diff for Review: (sign, line).
pub const TS_DIFF: [(char, &str); 7] = [
    (' ', "#[tokio::test]"),
    (' ', "async fn resumes_after_drop() {"),
    ('-', "    sleep(Duration::from_millis(2000)).await;"),
    ('-', "    assert!(upload.retried());"),
    ('+', "    let ev = upload.events().wait_for(Event::Retry);"),
    (
        '+',
        "    timeout(Duration::from_secs(10), ev).await.expect(\"retry\");",
    ),
    ('+', "    assert_eq!(upload.attempts(), 2);"),
];

/// Time as the block footer shows it ("6.24 s", "38 ms").
pub fn took(ms: u32) -> String {
    if ms >= 1000 {
        format!("{}.{:02} s", ms / 1000, (ms % 1000) / 10)
    } else {
        format!("{ms} ms")
    }
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Tabstack shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct TabstackData {
    /// Vertical tabs; the first one opens.
    pub tabs: Vec<TsTab>,
    /// The open tab's blocks.
    pub blocks: Vec<Block>,
}

impl TabstackData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            tabs: ts_tabs(st),
            blocks: if st == ScreenState::Sample {
                ts_blocks()
            } else {
                Vec::new()
            },
        }
    }
}

impl Default for TabstackData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_tabstack() {
        assert_eq!(ts_split("review").1, "review");
        let mut l = ts_tabs(ScreenState::Sample);
        assert_eq!(waiting(&l), 1);
        assert!(add_tab(&mut l, "hi").is_err());
        let id = add_tab(&mut l, "Speed up the CI cache").unwrap();
        assert_eq!((l[0].id, l[0].state), (id, TsState::Working));
        assert_eq!(run_cmd("echo hello").0, vec!["hello".to_string()]);
        assert_eq!(run_cmd("frobnicate").1, 127);
        assert_eq!(took(6_240), "6.24 s");
        assert_eq!(took(38), "38 ms");
        assert_eq!(TsState::Waiting.tone(), "wait");
        assert!(
            TabstackData::for_state(ScreenState::Loading)
                .blocks
                .is_empty()
        );
    }
}
