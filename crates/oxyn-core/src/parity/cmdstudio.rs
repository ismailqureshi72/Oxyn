//! Screens S10 (Agent apps II): two desktop coding-agent apps. **Command Center**, in the
//! threads-and-review pattern: a sidebar of projects and their threads, a composer with model and
//! reasoning pickers, a running thread whose changes land in a review pane (files, hunks, accept
//! or revert), a Local / Worktree / Cloud switch and a task queue. **Session Studio**, in the
//! session-and-tools pattern: a session list, a chat whose turns show tool blocks (read, edit,
//! bash), permission prompts, a permission mode with plan mode, a todo list and a diff pane. Our
//! own names and design, inspired by how such apps behave. The rules live here so Dioxus and GPUI
//! agree. Every name, file and company is fictional.

pub use super::screensfid::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, account_menu, usage,
};
pub use super::taskdeck::{prompt_error, title_from};

/// The Command Center app's name.
pub const CC_APP: &str = "Command Center";
/// The Session Studio app's name.
pub const SS_APP: &str = "Session Studio";

/// A state list: key and label for the state bar.
pub type StateList = &'static [(&'static str, &'static str)];

const CC_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("worktree", "Worktree"),
    ("automations", "Automations"),
    ("terminal", "Terminals"),
    ("browser", "Browser"),
];
const SS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("permission", "Permission"),
    ("chat", "Side chat"),
    ("arrange", "Rearrange"),
];

/// The screens' own states.
pub const OWN: [&str; 7] = [
    "worktree",
    "permission",
    "automations",
    "terminal",
    "browser",
    "chat",
    "arrange",
];

/// The states of an S10 screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    if slug == "session-studio" {
        SS_STATES
    } else {
        CC_STATES
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

// ───────────────────────────────────────────────────────── Where ──

/// Where a Command Center thread runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// In the project folder on this machine.
    Local,
    /// In a separate git worktree on this machine, on its own branch.
    Worktree,
    /// In a cloud sandbox.
    Cloud,
}

impl Place {
    /// All three, in menu order.
    pub const ALL: [Place; 3] = [Place::Local, Place::Worktree, Place::Cloud];
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Worktree => "worktree",
            Self::Cloud => "cloud",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Worktree => "Worktree",
            Self::Cloud => "Cloud",
        }
    }
    /// What it means.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Local => "Edits your checkout directly",
            Self::Worktree => "Own branch and folder; your checkout stays clean",
            Self::Cloud => "Runs in a sandbox; keeps going when you close the lid",
        }
    }
    /// The glyph name.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Local => "laptop",
            Self::Worktree => "branch",
            Self::Cloud => "cloud",
        }
    }
}

/// The branch a thread works on in a place.
pub fn branch_for(place: Place, thread: u32) -> String {
    match place {
        Place::Local => "main".to_string(),
        Place::Worktree => format!("cc/thread-{thread}"),
        Place::Cloud => format!("cloud/thread-{thread}"),
    }
}

// ─────────────────────────────────────────────────────── Threads ──

/// Where a thread stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TState {
    /// Waiting in the queue.
    Queued,
    /// Working.
    Running,
    /// Finished with changes to review.
    Review,
    /// Changes kept or the thread closed.
    Done,
}

impl TState {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Review => "To review",
            Self::Done => "Done",
        }
    }
    /// Tone key (`run`, `wait`, `ok` or neutral).
    pub fn tone(self) -> &'static str {
        match self {
            Self::Queued => "",
            Self::Running => "run",
            Self::Review => "wait",
            Self::Done => "ok",
        }
    }
}

/// Command Center projects: key and folder.
pub const CC_PROJECTS: [(&str, &str); 3] = [
    ("relay", "relay-api"),
    ("atlas", "atlas-web"),
    ("notes", "field-notes"),
];

/// A thread in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`CC_PROJECTS`].
    pub project: usize,
    /// Where it stands.
    pub state: TState,
    /// Where it runs.
    pub place: Place,
    /// Lines added and removed.
    pub stat: (u32, u32),
    /// Last change.
    pub ago: String,
}

fn th(
    id: u32,
    title: &str,
    project: usize,
    state: TState,
    place: Place,
    stat: (u32, u32),
    ago: &str,
) -> Thread {
    Thread {
        id,
        title: title.to_string(),
        project,
        state,
        place,
        stat,
        ago: ago.to_string(),
    }
}

/// The thread the Command Center opens.
pub const OPEN_THREAD: u32 = 1;

/// The sample threads.
pub fn sample_threads() -> Vec<Thread> {
    use Place::*;
    use TState::*;
    vec![
        th(
            1,
            "Retry failed uploads with backoff",
            0,
            Running,
            Local,
            (18, 6),
            "now",
        ),
        th(
            2,
            "Paginate the audit log endpoint",
            0,
            Review,
            Worktree,
            (64, 12),
            "9m",
        ),
        th(3, "Fix flaky webhook test", 0, Done, Cloud, (5, 3), "2h"),
        th(
            4,
            "Dark mode for the settings page",
            1,
            Running,
            Cloud,
            (120, 31),
            "1m",
        ),
        th(
            5,
            "Lazy-load the photo grid",
            1,
            Done,
            Local,
            (22, 40),
            "1d",
        ),
        th(
            6,
            "Export notes as Markdown",
            2,
            Review,
            Worktree,
            (48, 2),
            "3h",
        ),
    ]
}

/// The threads for a data state.
pub fn threads(st: ScreenState) -> Vec<Thread> {
    if st == ScreenState::Sample {
        sample_threads()
    } else {
        Vec::new()
    }
}

/// Start a thread from a prompt at the top; it runs in `place`. Returns its id.
pub fn add_thread(
    list: &mut Vec<Thread>,
    prompt: &str,
    project: usize,
    place: Place,
) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    list.insert(
        0,
        th(
            id,
            &title_from(prompt),
            project,
            TState::Running,
            place,
            (0, 0),
            "now",
        ),
    );
    Ok(id)
}

/// Models for the composer: name, brand mark id.
pub const CC_MODELS: [(&str, &str); 3] = [
    ("GPT-6", "openai"),
    ("Sonnet 5.5", "claude"),
    ("Gemini 3 Pro", "gemini"),
];

/// Reasoning levels: label and line.
pub const REASONING: [(&str, &str); 4] = [
    ("Low", "Quick edits"),
    ("Medium", "Most tasks"),
    ("High", "Hard bugs, careful plans"),
    ("Max", "Slowest, most thorough"),
];

/// The sample prompt of the open thread.
pub const CC_PROMPT: &str = "Uploads fail for good when the storage API returns a 503. Retry them with exponential backoff, at most 4 tries, and add a test.";

/// The open thread's work log: glyph, line, detail.
pub const CC_WORK: [(&str, &str, &str); 5] = [
    (
        "search",
        "Searched for upload call sites",
        "3 results in src/",
    ),
    ("file", "Read src/upload/client.rs", "142 lines"),
    ("pen", "Edited src/upload/client.rs", "+14 −6"),
    ("pen", "Added tests/retry.rs", "+4"),
    ("term", "Running cargo test retry", "2 of 3 passed…"),
];

// ────────────────────────────────────────────────────────── Diff ──

/// What the reviewer decided for a hunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Not decided.
    Open,
    /// Kept.
    Accepted,
    /// Undone.
    Reverted,
}

/// A hunk: header and lines (`+`, `-` or ` ` and the text).
#[derive(Clone, Debug, PartialEq)]
pub struct Hunk {
    /// `@@ -40,6 +40,14 @@ fn send`.
    pub head: &'static str,
    /// Diff lines.
    pub lines: Vec<(char, &'static str)>,
    /// Decision.
    pub verdict: Verdict,
}

/// A changed file.
#[derive(Clone, Debug, PartialEq)]
pub struct DiffFile {
    /// Path.
    pub path: &'static str,
    /// True for a new file.
    pub new: bool,
    /// Hunks.
    pub hunks: Vec<Hunk>,
}

fn hk(head: &'static str, lines: &[(char, &'static str)]) -> Hunk {
    Hunk {
        head,
        lines: lines.to_vec(),
        verdict: Verdict::Open,
    }
}

/// The open Command Center thread's changes.
pub fn cc_diff() -> Vec<DiffFile> {
    vec![
        DiffFile {
            path: "src/upload/client.rs",
            new: false,
            hunks: vec![
                hk(
                    "@@ -38,9 +38,17 @@ impl Client",
                    &[
                        (' ', "pub async fn send(&self, part: Part) -> Result<()> {"),
                        (
                            '-',
                            "    let res = self.http.put(&part.url).body(part.bytes).send().await?;",
                        ),
                        ('-', "    res.error_for_status()?;"),
                        ('-', "    Ok(())"),
                        ('+', "    let mut wait = Duration::from_millis(250);"),
                        ('+', "    for attempt in 1..=MAX_TRIES {"),
                        ('+', "        match self.put(&part).await {"),
                        (
                            '+',
                            "            Err(e) if e.is_retryable() && attempt < MAX_TRIES => {",
                        ),
                        ('+', "                sleep(wait).await;"),
                        ('+', "                wait *= 2;"),
                        ('+', "            }"),
                        ('+', "            other => return other,"),
                        ('+', "        }"),
                        ('+', "    }"),
                        ('+', "    unreachable!()"),
                        (' ', "}"),
                    ],
                ),
                hk(
                    "@@ -12,6 +12,9 @@ use std::time::Duration;",
                    &[
                        (' ', "use crate::Part;"),
                        ('+', ""),
                        ('+', "/// Tries per part before giving up."),
                        ('+', "const MAX_TRIES: u32 = 4;"),
                        ('-', "use tokio::time::timeout;"),
                        ('-', "use tracing::debug;"),
                        ('-', "use tracing::warn;"),
                    ],
                ),
            ],
        },
        DiffFile {
            path: "tests/retry.rs",
            new: true,
            hunks: vec![hk(
                "@@ -0,0 +1,4 @@",
                &[
                    ('+', "#[tokio::test]"),
                    ('+', "async fn retries_a_503_then_succeeds() {"),
                    ('+', "    assert_eq!(fake(503, 2).send_ok().await, 3);"),
                    ('+', "}"),
                ],
            )],
        },
    ]
}

/// Added and removed lines in a hunk.
pub fn hunk_stat(h: &Hunk) -> (u32, u32) {
    let a = h.lines.iter().filter(|l| l.0 == '+').count() as u32;
    let d = h.lines.iter().filter(|l| l.0 == '-').count() as u32;
    (a, d)
}

/// Added and removed lines over a file's hunks that are not reverted.
pub fn file_stat(f: &DiffFile) -> (u32, u32) {
    f.hunks
        .iter()
        .filter(|h| h.verdict != Verdict::Reverted)
        .map(hunk_stat)
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

/// Added and removed lines over every file.
pub fn diff_stat(files: &[DiffFile]) -> (u32, u32) {
    files
        .iter()
        .map(file_stat)
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

/// Set one hunk's verdict. Returns false when out of range.
pub fn set_hunk(files: &mut [DiffFile], f: usize, h: usize, v: Verdict) -> bool {
    match files.get_mut(f).and_then(|x| x.hunks.get_mut(h)) {
        Some(x) => {
            x.verdict = v;
            true
        }
        None => false,
    }
}

/// Set every hunk of a file. Returns false when out of range.
pub fn set_file(files: &mut [DiffFile], f: usize, v: Verdict) -> bool {
    match files.get_mut(f) {
        Some(x) => {
            for h in &mut x.hunks {
                h.verdict = v;
            }
            true
        }
        None => false,
    }
}

/// A file's verdict when all its hunks agree.
pub fn file_verdict(f: &DiffFile) -> Verdict {
    let first = f.hunks.first().map(|h| h.verdict).unwrap_or(Verdict::Open);
    if f.hunks.iter().all(|h| h.verdict == first) {
        first
    } else {
        Verdict::Open
    }
}

/// Hunks decided and total.
pub fn reviewed(files: &[DiffFile]) -> (usize, usize) {
    let all = files.iter().flat_map(|f| f.hunks.iter());
    let n = all.clone().count();
    (all.filter(|h| h.verdict != Verdict::Open).count(), n)
}

/// The review summary line.
pub fn review_line(files: &[DiffFile]) -> String {
    let (d, n) = reviewed(files);
    let (a, r) = diff_stat(files);
    let f = files.len();
    let s = if f == 1 { "" } else { "s" };
    if n > 0 && d == n {
        format!("All {n} hunks reviewed · +{a} −{r} kept")
    } else {
        format!("{f} file{s} changed · +{a} −{r} · {d} of {n} reviewed")
    }
}

// ───────────────────────────────────────────────────────── Queue ──

/// The sample task queue: id, text.
pub fn sample_queue() -> Vec<(u32, String)> {
    vec![
        (1, "Add a metrics counter for retries".to_string()),
        (2, "Update the upload docs".to_string()),
        (3, "Bump the HTTP client".to_string()),
    ]
}

/// Queue a follow-up task. Returns its id.
pub fn enqueue(q: &mut Vec<(u32, String)>, text: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(text) {
        return Err(e);
    }
    let id = q.iter().map(|x| x.0).max().unwrap_or(0) + 1;
    q.push((id, title_from(text)));
    Ok(id)
}

/// Move a queued item one place up. Returns false when it is first or absent.
pub fn queue_up(q: &mut [(u32, String)], id: u32) -> bool {
    match q.iter().position(|x| x.0 == id) {
        Some(i) if i > 0 => {
            q.swap(i, i - 1);
            true
        }
        _ => false,
    }
}

/// Take a queued item out. Returns false when absent.
pub fn dequeue(q: &mut Vec<(u32, String)>, id: u32) -> bool {
    let n = q.len();
    q.retain(|x| x.0 != id);
    q.len() != n
}

// ───────────────────────────────────────────── Session Studio ──

/// How much the session may do on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Asks before every edit and command.
    Ask,
    /// Edits files on its own; asks before commands.
    AcceptEdits,
    /// Reads and plans only; nothing changes until you approve the plan.
    Plan,
}

impl Mode {
    /// In cycle order (Shift + Tab).
    pub const ALL: [Mode; 3] = [Mode::Ask, Mode::AcceptEdits, Mode::Plan];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Ask before edits",
            Self::AcceptEdits => "Accept edits",
            Self::Plan => "Plan mode",
        }
    }
    /// Line.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Ask => "Asks before every edit and command",
            Self::AcceptEdits => "Edits files on its own, asks before commands",
            Self::Plan => "Reads and plans; changes nothing until you approve",
        }
    }
    /// The next mode (Shift + Tab).
    pub fn next(self) -> Mode {
        match self {
            Self::Ask => Self::AcceptEdits,
            Self::AcceptEdits => Self::Plan,
            Self::Plan => Self::Ask,
        }
    }
}

/// A session in the list: id, title, folder, age, status (`run`, `wait`, ``).
pub type Session = (u32, &'static str, &'static str, &'static str, &'static str);

/// The sample sessions.
pub const SESSIONS: [Session; 5] = [
    (1, "Cache invoice PDFs", "billing-svc", "now", "run"),
    (2, "Why is the sync job slow?", "billing-svc", "25m", ""),
    (
        3,
        "Migrate forms to the new validator",
        "atlas-web",
        "2h",
        "wait",
    ),
    (4, "Write the CLI help text", "tidy-cli", "1d", ""),
    (5, "Add a health endpoint", "tidy-cli", "3d", ""),
];

/// The sessions for a data state.
pub fn sessions(st: ScreenState) -> Vec<Session> {
    if st == ScreenState::Sample {
        SESSIONS.to_vec()
    } else {
        Vec::new()
    }
}

/// Session Studio models: name, brand mark id.
pub const SS_MODELS: [(&str, &str); 3] = [
    ("Opus 5", "claude"),
    ("Sonnet 5.5", "claude"),
    ("Haiku 5", "claude"),
];

/// How a tool block or a todo stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BState {
    /// Waiting to run or not started.
    Pending,
    /// Working.
    Running,
    /// Needs your permission.
    Asking,
    /// Finished.
    Done,
    /// You said no.
    Denied,
}

/// A block of the session chat.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// `user`, `text`, `read`, `edit`, `bash` or `plan`.
    pub kind: &'static str,
    /// The text, the path or the command.
    pub body: String,
    /// Detail: lines read, the change, the output.
    pub out: Vec<String>,
    /// State (text blocks are Done).
    pub state: BState,
}

fn blk(kind: &'static str, body: &str, out: &[&str], state: BState) -> Block {
    Block {
        kind,
        body: body.to_string(),
        out: out.iter().map(|s| s.to_string()).collect(),
        state,
    }
}

/// The sample prompt of the open session.
pub const SS_PROMPT: &str = "Invoice PDFs are rendered on every download. Cache them by invoice id and version, and clear the cache when an invoice changes.";

/// The command the session asks permission for.
pub const ASK_CMD: &str = "npm test -- invoices";

/// The open session's blocks; `ask` ends on a permission prompt for the test command.
pub fn session_blocks(ask: bool) -> Vec<Block> {
    use BState::*;
    let mut v = vec![
        blk("user", SS_PROMPT, &[], Done),
        blk(
            "text",
            "I'll find where PDFs are rendered, add a cache keyed by id and version, then run the invoice tests.",
            &[],
            Done,
        ),
        blk("read", "src/invoices/pdf.ts", &["Read 88 lines"], Done),
        blk("read", "src/invoices/events.ts", &["Read 41 lines"], Done),
        blk(
            "edit",
            "src/invoices/pdf.ts",
            &[
                "+ const cache = new Map<string, Buffer>();",
                "+ const key = `${inv.id}@${inv.version}`;",
                "+ if (cache.has(key)) return cache.get(key)!;",
                "- return render(inv);",
                "+ const pdf = await render(inv);",
                "+ cache.set(key, pdf);",
                "+ return pdf;",
            ],
            Done,
        ),
        blk(
            "edit",
            "src/invoices/events.ts",
            &["+ onInvoiceChanged((id) => clearPdf(id));"],
            Done,
        ),
    ];
    v.push(blk(
        "bash",
        ASK_CMD,
        &[],
        if ask { Asking } else { Running },
    ));
    v
}

/// What the user answers on a permission prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Run it this once.
    Once,
    /// Run it and never ask again for this command in this folder.
    Always,
    /// Do not run it.
    Deny,
}

/// The output the test command prints when it runs.
pub const TEST_OUT: [&str; 3] = [
    "PASS  invoices/pdf.test.ts (6 tests)",
    "PASS  invoices/events.test.ts (3 tests)",
    "Tests: 9 passed, 9 total",
];

/// Answer the first block that is asking. Allowed commands finish with [`TEST_OUT`]; a denied one
/// gets a reply that says what the session will do instead. Returns false when nothing asks.
pub fn answer(blocks: &mut Vec<Block>, a: Answer) -> bool {
    let Some(i) = blocks.iter().position(|b| b.state == BState::Asking) else {
        return false;
    };
    match a {
        Answer::Deny => {
            blocks[i].state = BState::Denied;
            blocks.push(blk("text", "Skipped the tests. The change is ready to review in the diff pane; tell me when to run them.", &[], BState::Done));
        }
        _ => {
            blocks[i].state = BState::Done;
            blocks[i].out = TEST_OUT.iter().map(|s| s.to_string()).collect();
            blocks.push(blk("text", "All 9 invoice tests pass. PDFs are cached per id and version and cleared when an invoice changes.", &[], BState::Done));
        }
    }
    true
}

/// Finish a running command block (the timer's tick). Returns false when none runs.
pub fn finish_running(blocks: &mut Vec<Block>) -> bool {
    let Some(i) = blocks.iter().position(|b| b.state == BState::Running) else {
        return false;
    };
    blocks[i].state = BState::Asking;
    let _ = answer(blocks, Answer::Once);
    true
}

/// A prompt sent in a mode: the user block, then a plan (Plan mode) or a reply that starts work.
pub fn send(blocks: &mut Vec<Block>, prompt: &str, mode: Mode) -> Result<(), &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    blocks.push(blk("user", prompt.trim(), &[], BState::Done));
    if mode == Mode::Plan {
        let steps = plan_for(prompt);
        let s: Vec<&str> = steps.iter().map(|s| s.as_str()).collect();
        blocks.push(blk(
            "plan",
            "Here is the plan. Nothing changes until you approve it.",
            &s,
            BState::Asking,
        ));
    } else {
        blocks.push(blk(
            "text",
            "On it. I'll read the relevant files first.",
            &[],
            BState::Done,
        ));
    }
    Ok(())
}

/// A short plan for a prompt.
pub fn plan_for(prompt: &str) -> Vec<String> {
    let t = title_from(prompt);
    vec![
        "Read the files the change touches".to_string(),
        format!("Make the change: {t}"),
        "Add or update tests".to_string(),
        "Run the tests and report".to_string(),
    ]
}

/// Approve the waiting plan: it becomes the todo list and the mode switches to Accept edits.
/// Returns the new todos, or None when no plan waits.
pub fn approve_plan(blocks: &mut [Block]) -> Option<Vec<(String, BState)>> {
    let b = blocks
        .iter_mut()
        .rev()
        .find(|b| b.kind == "plan" && b.state == BState::Asking)?;
    b.state = BState::Done;
    Some(
        b.out
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    s.clone(),
                    if i == 0 {
                        BState::Running
                    } else {
                        BState::Pending
                    },
                )
            })
            .collect(),
    )
}

/// The open session's todo list.
pub fn sample_todos(ask: bool) -> Vec<(String, BState)> {
    [
        ("Find where PDFs are rendered", BState::Done),
        ("Cache by invoice id and version", BState::Done),
        ("Clear the cache on invoice changes", BState::Done),
        (
            "Run the invoice tests",
            if ask { BState::Asking } else { BState::Running },
        ),
        ("Summarise the change", BState::Pending),
    ]
    .iter()
    .map(|(s, b)| (s.to_string(), *b))
    .collect()
}

/// Todos done and total.
pub fn todo_progress(t: &[(String, BState)]) -> (usize, usize) {
    (t.iter().filter(|x| x.1 == BState::Done).count(), t.len())
}

/// Tick a todo done, or open it again. Returns false when out of range.
pub fn toggle_todo(t: &mut [(String, BState)], i: usize) -> bool {
    match t.get_mut(i) {
        Some(x) => {
            x.1 = if x.1 == BState::Done {
                BState::Pending
            } else {
                BState::Done
            };
            true
        }
        None => false,
    }
}

/// The open session's diff (for the diff pane).
pub fn ss_diff() -> Vec<DiffFile> {
    vec![
        DiffFile {
            path: "src/invoices/pdf.ts",
            new: false,
            hunks: vec![hk(
                "@@ -21,7 +21,12 @@ export async function pdfFor(inv)",
                &[
                    (' ', "export async function pdfFor(inv: Invoice) {"),
                    ('+', "  const key = `${inv.id}@${inv.version}`;"),
                    ('+', "  if (cache.has(key)) return cache.get(key)!;"),
                    ('-', "  return render(inv);"),
                    ('+', "  const pdf = await render(inv);"),
                    ('+', "  cache.set(key, pdf);"),
                    ('+', "  return pdf;"),
                    (' ', "}"),
                ],
            )],
        },
        DiffFile {
            path: "src/invoices/events.ts",
            new: false,
            hunks: vec![hk(
                "@@ -9,3 +9,4 @@",
                &[
                    (' ', "export function wire() {"),
                    ('+', "  onInvoiceChanged((id) => clearPdf(id));"),
                    (' ', "}"),
                ],
            )],
        },
    ]
}

// ───────────────────────────────────────────── Command Center extras ──

/// What starts an automation: key, label.
pub const TRIGGERS: [(&str, &str); 4] = [
    ("nightly", "Every night at 02:00"),
    ("issue", "When an issue is labelled bug"),
    ("ci", "When CI fails on main"),
    ("weekly", "Every Monday at 09:00"),
];

/// An automation: a prompt the app runs on a trigger.
#[derive(Clone, Debug, PartialEq)]
pub struct Automation {
    /// Stable id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Index into [`TRIGGERS`].
    pub trigger: usize,
    /// What it asks for.
    pub prompt: String,
    /// Index into [`CC_PROJECTS`].
    pub project: usize,
    /// Switched on.
    pub on: bool,
    /// Last run: tone (`ok`, `bad`, `run`, `` for never) and when.
    pub last: (&'static str, String),
}

/// The sample automations (none when empty).
pub fn automations(st: ScreenState) -> Vec<Automation> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    let a = |id, n: &str, t, p: &str, pj, on, tone, ago: &str| Automation {
        id,
        name: n.into(),
        trigger: t,
        prompt: p.into(),
        project: pj,
        on,
        last: (tone, ago.into()),
    };
    vec![
        a(
            1,
            "Bump dependencies",
            0,
            "Update patch versions, run the tests, open a review thread if they pass.",
            0,
            true,
            "ok",
            "6h ago",
        ),
        a(
            2,
            "Triage new bugs",
            1,
            "Reproduce the bug, find the likely file and write a first fix in a worktree.",
            1,
            true,
            "ok",
            "2d ago",
        ),
        a(
            3,
            "Fix red CI",
            2,
            "Read the failing job, fix the cause in a worktree and wait for review.",
            0,
            false,
            "bad",
            "3d ago",
        ),
        a(
            4,
            "Weekly changelog",
            3,
            "Summarise merged changes since last Monday into CHANGELOG.md.",
            2,
            true,
            "",
            "",
        ),
    ]
}

/// Why an automation cannot be saved, if it cannot.
pub fn automation_error(name: &str, prompt: &str) -> Option<&'static str> {
    if name.trim().is_empty() {
        Some("Give it a name")
    } else if prompt.trim().chars().count() < 8 {
        Some("Say what it should do")
    } else {
        None
    }
}

/// A terminal tab: id, title and its lines (`$ ` starts a command).
#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    /// Stable id.
    pub id: u32,
    /// Tab title.
    pub title: String,
    /// Scrollback.
    pub lines: Vec<String>,
}

/// The sample terminals: the thread's test run and a shell.
pub fn sample_terms() -> Vec<Term> {
    vec![
        Term {
            id: 1,
            title: "cargo test".into(),
            lines: vec![
                "$ cargo test retry".into(),
                "   Compiling upload v0.4.2".into(),
                "running 3 tests".into(),
                "test retries_a_503_then_succeeds ... ok".into(),
                "test gives_up_after_four_tries ... ok".into(),
                "test does_not_retry_a_400 ... ok".into(),
                "test result: ok. 3 passed; 0 failed".into(),
            ],
        },
        Term {
            id: 2,
            title: "zsh".into(),
            lines: vec![
                "$ git status --short".into(),
                " M src/upload/client.rs".into(),
                "?? tests/retry.rs".into(),
            ],
        },
    ]
}

/// Open a new shell tab. Returns its id.
pub fn new_term(t: &mut Vec<Term>) -> u32 {
    let id = t.iter().map(|x| x.id).max().unwrap_or(0) + 1;
    t.push(Term {
        id,
        title: format!("zsh {id}"),
        lines: Vec::new(),
    });
    id
}

/// Close a tab; the last one cannot close. Returns false when it did not.
pub fn close_term(t: &mut Vec<Term>, id: u32) -> bool {
    if t.len() <= 1 {
        return false;
    }
    let n = t.len();
    t.retain(|x| x.id != id);
    t.len() < n
}

/// Run a command in a tab: echo it and print a (simulated) answer. `clear` empties the tab.
pub fn run_cmd(t: &mut [Term], id: u32, cmd: &str) -> bool {
    let c = cmd.trim();
    let Some(x) = t.iter_mut().find(|x| x.id == id) else {
        return false;
    };
    if c.is_empty() {
        return false;
    }
    if c == "clear" {
        x.lines.clear();
        return true;
    }
    x.lines.push(format!("$ {c}"));
    let out: &[&str] = match c.split_whitespace().next().unwrap_or("") {
        "ls" => &["Cargo.toml  README.md  src  tests"],
        "pwd" => &["~/code/upload-svc"],
        "git" => &[
            "On branch main",
            "Changes not staged: src/upload/client.rs, tests/retry.rs",
        ],
        "cargo" => &["running 3 tests", "test result: ok. 3 passed; 0 failed"],
        "echo" => &[],
        _ => &["(sample terminal: commands are simulated)"],
    };
    if let Some(rest) = c.strip_prefix("echo ") {
        x.lines.push(rest.to_string());
    }
    x.lines.extend(out.iter().map(|s| s.to_string()));
    true
}

/// Pages the browser pane can show: address, title, lines.
pub const PAGES: [(&str, &str, &[&str]); 3] = [
    (
        "localhost:3000/uploads",
        "Uploads",
        &[
            "Drop files to upload",
            "report-q3.pdf · retried 2× · done",
            "photos.zip · uploading 64%",
        ],
    ),
    (
        "localhost:3000/settings",
        "Settings",
        &["Storage region: eu-west", "Retries: 4 with backoff"],
    ),
    (
        "docs.upload.example/retries",
        "Retries · Upload docs",
        &[
            "Retry 5xx and timeouts with exponential backoff.",
            "Never retry a 4xx.",
        ],
    ),
];

/// The page for an address the user typed (scheme and `www.` ignored); None when unknown.
pub fn page_for(addr: &str) -> Option<usize> {
    let a = addr
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/');
    PAGES.iter().position(|p| p.0 == a)
}

// ───────────────────────────────────────────── Session Studio extras ──

/// The side chat's opening messages: (from you, text). A side chat asks about the session without
/// interrupting it.
pub const SIDE_CHAT: [(bool, &str); 2] = [
    (true, "Why key the cache by version and not by updated_at?"),
    (
        false,
        "Versions only change when the invoice does; updated_at also moves on unrelated edits, so the cache would miss more often.",
    ),
];

/// The side chat's answer to a question (canned, about the open session).
pub fn side_reply(q: &str) -> &'static str {
    let q = q.to_lowercase();
    if q.contains("test") {
        "The session runs npm test -- invoices after the edit; it asks first in Ask mode."
    } else if q.contains("memory") || q.contains("size") {
        "The cache is an LRU of 200 PDFs, about 40 MB at the sample sizes."
    } else {
        "Noted. I answered from the session so far without interrupting it."
    }
}

/// Session Studio's side panes, in default order: key, title.
pub const PANES: [(&str, &str); 3] = [
    ("diff", "Diff"),
    ("chat", "Side chat"),
    ("context", "Context"),
];

/// Move the pane at `from` to `to` (both clamped). Returns false when nothing moved.
pub fn move_pane(order: &mut Vec<usize>, from: usize, to: usize) -> bool {
    if order.is_empty() {
        return false;
    }
    let (f, t) = (from.min(order.len() - 1), to.min(order.len() - 1));
    if f == t {
        return false;
    }
    let x = order.remove(f);
    order.insert(t, x);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s10_extras() {
        assert_eq!(split("browser").1, "browser");
        assert_eq!(states_for("session-studio")[5].0, "arrange");
        for id in 1..=6 {
            assert!(!thread_diff(id).is_empty(), "thread {id}");
            assert_eq!(
                threads(ScreenState::Sample)[(id - 1) as usize].stat,
                diff_stat(&thread_diff(id))
            );
        }
        for id in 1..=5 {
            assert!(!session_diff(id).is_empty(), "session {id}");
        }
        assert_ne!(thread_diff(2), thread_diff(3));
        assert!(thread_diff(9).is_empty());
        assert_eq!(automation_error("", "x"), Some("Give it a name"));
        assert!(automation_error("Nightly", "Update the lockfile").is_none());
        let mut t = sample_terms();
        let id = new_term(&mut t);
        assert_eq!(id, 3);
        assert!(run_cmd(&mut t, id, "echo hi"));
        assert_eq!(t[2].lines, vec!["$ echo hi", "hi"]);
        assert!(run_cmd(&mut t, id, "clear") && t[2].lines.is_empty());
        assert!(close_term(&mut t, 1) && close_term(&mut t, 2));
        assert!(!close_term(&mut t, 3));
        assert_eq!(page_for("https://localhost:3000/settings/"), Some(1));
        assert_eq!(page_for("example.org"), None);
        let mut o = vec![0, 1, 2];
        assert!(move_pane(&mut o, 2, 0));
        assert_eq!(o, vec![2, 0, 1]);
        assert!(!move_pane(&mut o, 1, 1));
        assert!(side_reply("does it run the tests?").contains("npm test"));
        let d = CommandCenterData::default();
        assert_eq!(d.diff(2), thread_diff(2));
        assert!(d.diff(99).is_empty());
        assert_eq!(SessionStudioData::default().diff(3), session_diff(3));
    }

    #[test]
    fn s10_cmdstudio() {
        assert_eq!(split("worktree"), (ScreenState::Sample, "worktree"));
        assert_eq!(split("loading").0, ScreenState::Loading);
        assert_eq!(states_for("session-studio")[3].0, "permission");
        assert_eq!(branch_for(Place::Worktree, 7), "cc/thread-7");
        let mut l = sample_threads();
        let id = add_thread(
            &mut l,
            "Add rate limits to the public API. Keep it simple.",
            0,
            Place::Cloud,
        )
        .unwrap();
        assert_eq!(id, 7);
        assert_eq!(l[0].title, "Add rate limits to the public API");
        assert!(add_thread(&mut l, " ", 0, Place::Local).is_err());
        let mut d = cc_diff();
        assert_eq!(hunk_stat(&d[0].hunks[0]), (11, 3));
        assert_eq!(diff_stat(&d), (18, 6));
        assert_eq!(
            review_line(&d),
            "2 files changed · +18 −6 · 0 of 3 reviewed"
        );
        assert!(set_hunk(&mut d, 0, 1, Verdict::Reverted));
        assert_eq!(diff_stat(&d), (15, 3));
        assert_eq!(file_verdict(&d[0]), Verdict::Open);
        assert!(set_file(&mut d, 0, Verdict::Accepted));
        assert_eq!(file_verdict(&d[0]), Verdict::Accepted);
        assert!(set_file(&mut d, 1, Verdict::Accepted));
        assert!(!set_hunk(&mut d, 3, 0, Verdict::Accepted));
        assert_eq!(review_line(&d), "All 3 hunks reviewed · +18 −6 kept");
        let mut q = sample_queue();
        assert_eq!(enqueue(&mut q, "Write a changelog entry").unwrap(), 4);
        assert!(queue_up(&mut q, 4));
        assert_eq!(q[2].0, 4);
        assert!(!queue_up(&mut q, 1));
        assert!(dequeue(&mut q, 2));
        assert!(!dequeue(&mut q, 2));
        assert_eq!(Mode::Plan.next(), Mode::Ask);
        let mut b = session_blocks(true);
        assert!(answer(&mut b, Answer::Deny));
        assert_eq!(b[6].state, BState::Denied);
        assert!(!answer(&mut b, Answer::Once));
        let mut b = session_blocks(false);
        assert!(finish_running(&mut b));
        assert_eq!(b[6].out.len(), 3);
        assert!(send(&mut b, "Add a size cap to the cache", Mode::Plan).is_ok());
        let todos = approve_plan(&mut b).unwrap();
        assert_eq!(todos.len(), 4);
        assert_eq!(todos[0].1, BState::Running);
        assert!(approve_plan(&mut b).is_none());
        let mut t = sample_todos(false);
        assert_eq!(todo_progress(&t), (3, 5));
        assert!(toggle_todo(&mut t, 4));
        assert_eq!(todo_progress(&t), (4, 5));
        assert_eq!(diff_stat(&ss_diff()), (6, 1));
        assert_eq!(sessions(ScreenState::Empty).len(), 0);
    }
}
