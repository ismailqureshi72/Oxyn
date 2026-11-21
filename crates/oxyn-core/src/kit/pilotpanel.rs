//! Screens S16 (Agent apps IV): **Pilot Panel**, an editor's agent panel as a desktop app. Task
//! history in the sidebar with each task's cost; the open task as a run of steps (the agent's
//! notes, files it read, an edit with its diff, a command with its output) with a checkpoint
//! after each change that restores the files to that point; Plan and Act modes (Plan only
//! reads and proposes, Act changes files); tool calls that wait on Approve or Reject unless
//! auto-approve covers them; a context bar with tokens and cost; and an Editor pane with the
//! changed file and the checkpoints. Our own name and design, inspired by how such panels
//! behave. Every name, file, number and company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const PK_APP: &str = "Pilot Panel";

/// The state bar of Pilot Panel.
pub const PK_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("plan", "Plan mode"),
    ("approval", "Needs approval"),
];

/// Split a state key into the data state and the own state (`plan`, `approval` or "").
pub fn pk_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "plan" => (ScreenState::Sample, "plan"),
        "approval" => (ScreenState::Sample, "approval"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Models (bring your own key): name and brand mark id.
pub const PK_MODELS: [(&str, &str); 3] = [
    ("Sonnet 5.5", "claude"),
    ("Gemini 3 Pro", "gemini"),
    ("DeepSeek V4", "deepseek"),
];

/// The pane's tabs: key and label.
pub const PK_TABS: [(&str, &str); 2] = [("editor", "Editor"), ("checkpoints", "Checkpoints")];

/// Kinds of tool call auto-approve can cover: key and label.
pub const PK_AUTO: [(&str, &str); 3] = [
    ("read", "Read files"),
    ("edit", "Edit files"),
    ("cmd", "Run commands"),
];

/// Whether a tool call of `kind` waits on you, given what auto-approve covers (same order as
/// [`PK_AUTO`]). Plan mode never edits or runs, so it never asks.
pub fn needs_approval(kind: &str, auto: [bool; 3], plan: bool) -> bool {
    if plan {
        return false;
    }
    match PK_AUTO.iter().position(|a| a.0 == kind) {
        Some(i) => !auto[i],
        None => false,
    }
}

/// A task in the history.
#[derive(Clone, Debug, PartialEq)]
pub struct PkTask {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Cost in cents.
    pub cents: u32,
    /// Working right now.
    pub running: bool,
    /// Started, an absolute ISO time.
    pub at: &'static str,
}

fn tk(id: u32, title: &str, cents: u32, running: bool, at: &'static str) -> PkTask {
    PkTask {
        id,
        title: title.to_string(),
        cents,
        running,
        at,
    }
}

/// The sample tasks (empty for loading and empty).
pub fn pk_tasks(st: ScreenState) -> Vec<PkTask> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        tk(
            1,
            "Paginate the orders endpoint",
            42,
            false,
            "2026-11-19 08:50",
        ),
        tk(
            2,
            "Rename UserCard to MemberCard",
            18,
            false,
            "2026-11-18 15:31",
        ),
        tk(3, "Why is the build slow?", 9, false, "2026-11-18 10:02"),
        tk(4, "Add a dark mode toggle", 67, false, "2026-11-17 14:48"),
    ]
}

/// Cents as dollars, `$0.42`.
pub fn usd(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// Start a task from a prompt at the top of the history; it runs at once.
pub fn add_task(list: &mut Vec<PkTask>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(0, tk(id, &t, 0, true, "2026-11-19 09:02"));
    Ok(id)
}

/// The open task's prompt.
pub const PK_PROMPT: &str = "GET /orders returns every order at once. Add cursor pagination with a limit of 50 and update the test.";

/// A step in the task: kind (`say`, `read`, `edit`, `cmd`), title and detail. A checkpoint
/// follows every `edit` and `cmd`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PkStep {
    /// `say`, `read`, `edit` or `cmd`.
    pub kind: &'static str,
    /// What it is.
    pub title: &'static str,
    /// The line under it.
    pub detail: &'static str,
}

/// The sample task's steps.
pub const PK_STEPS: [PkStep; 5] = [
    PkStep {
        kind: "read",
        title: "src/routes/orders.ts",
        detail: "64 lines",
    },
    PkStep {
        kind: "say",
        title: "The handler loads all orders. I'll add a cursor on id and a limit capped at 50.",
        detail: "",
    },
    PkStep {
        kind: "edit",
        title: "src/routes/orders.ts",
        detail: "+14 −3",
    },
    PkStep {
        kind: "edit",
        title: "test/orders.test.ts",
        detail: "+21 −2",
    },
    PkStep {
        kind: "cmd",
        title: "npm test -- orders",
        detail: "9 passed · 2.8 s",
    },
];

/// The checkpoints: the step index each one follows.
pub fn checkpoints(steps: &[PkStep]) -> Vec<usize> {
    steps
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s.kind, "edit" | "cmd"))
        .map(|(i, _)| i)
        .collect()
}

/// Restore to the checkpoint after step `at`: the steps after it are dropped.
pub fn restore(steps: &mut Vec<PkStep>, at: usize) {
    steps.truncate(at + 1);
}

/// The command waiting on approval in the `approval` state.
pub const PK_PENDING: &str = "npm run migrate -- --env staging";

/// The plan Plan mode proposes.
pub const PK_PLAN: [&str; 4] = [
    "Read the orders route and its test",
    "Add a cursor (last id) and limit (max 50) to the query",
    "Return next_cursor when more orders remain",
    "Update the test for two pages and the limit cap",
];

/// The edited file in the Editor pane and its lines, (sign, text).
pub const PK_FILE: &str = "src/routes/orders.ts";

/// The diff of [`PK_FILE`].
pub const PK_DIFF: [(char, &str); 9] = [
    (' ', "router.get(\"/orders\", async (req, res) => {"),
    ('-', "  const orders = await db.orders.findMany();"),
    ('-', "  res.json(orders);"),
    (
        '+',
        "  const limit = Math.min(Number(req.query.limit) || 50, 50);",
    ),
    (
        '+',
        "  const cursor = req.query.cursor as string | undefined;",
    ),
    ('+', "  const orders = await db.orders.findMany({"),
    (
        '+',
        "    take: limit + 1, ...(cursor && { cursor: { id: cursor }, skip: 1 }),",
    ),
    ('+', "  });"),
    ('+', "  res.json(page(orders, limit));"),
];

/// Context used and the window, in tokens.
pub const PK_CTX: (u32, u32) = (38_200, 200_000);

/// Context used as a whole percent.
pub fn ctx_pct(used: u32, limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    ((used as u64 * 100 + limit as u64 / 2) / limit as u64) as u32
}

/// Tokens as `38.2k`.
pub fn kilo(n: u32) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let t = (n + 50) / 100;
    if t.is_multiple_of(10) {
        format!("{}k", t / 10)
    } else {
        format!("{}.{}k", t / 10, t % 10)
    }
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Pilot Panel shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct PilotPanelData {
    /// Tasks in the history; the first one opens.
    pub tasks: Vec<PkTask>,
    /// The open task's steps.
    pub steps: Vec<PkStep>,
}

impl PilotPanelData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            tasks: pk_tasks(st),
            steps: if st == ScreenState::Sample {
                PK_STEPS.to_vec()
            } else {
                Vec::new()
            },
        }
    }
}

impl Default for PilotPanelData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_pilotpanel() {
        assert_eq!(pk_split("plan").1, "plan");
        assert_eq!(pk_split("empty").0, ScreenState::Empty);
        assert!(needs_approval("cmd", [true, true, false], false));
        assert!(!needs_approval("read", [true, false, false], false));
        assert!(!needs_approval("cmd", [false, false, false], true));
        assert!(!needs_approval("say", [false, false, false], false));
        let mut l = pk_tasks(ScreenState::Sample);
        assert!(add_task(&mut l, "").is_err());
        let id = add_task(&mut l, "Add rate limiting to login").unwrap();
        assert_eq!((l[0].id, l[0].running), (id, true));
        assert_eq!(usd(42), "$0.42");
        assert_eq!(usd(1_067), "$10.67");
        let mut s = PK_STEPS.to_vec();
        assert_eq!(checkpoints(&s), vec![2, 3, 4]);
        restore(&mut s, 2);
        assert_eq!(s.len(), 3);
        assert_eq!(checkpoints(&s), vec![2]);
        assert_eq!(ctx_pct(PK_CTX.0, PK_CTX.1), 19);
        assert_eq!(kilo(38_200), "38.2k");
        assert_eq!(kilo(200_000), "200k");
        assert_eq!(kilo(640), "640");
        assert!(
            PilotPanelData::for_state(ScreenState::Loading)
                .steps
                .is_empty()
        );
    }
}
