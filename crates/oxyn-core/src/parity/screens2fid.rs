//! Screens 2 (the agent screens): models for the screens an agent product is run from –
//! Orchestration, Thread and the rest – built on the Screens 1 frame
//! ([`super::screensfid`]). Each screen shows Sample, Loading or Empty data. Behaviour only: every
//! agent, person, project and line of copy here is fictional and our own.

pub use super::screensfid::{ScreenState, ShellLayout, clock, money};

/// The projects the agent screens group work by.
pub const AG_PROJECTS: [&str; 3] = ["Harbor launch", "Field guide", "Ledger tidy"];

// ─────────────────────────────────────────────────────────────── status ──

/// Where an agent, task or step stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgStatus {
    /// Working now.
    Running,
    /// Finished.
    Done,
    /// Waiting for the person.
    NeedsYou,
    /// Not started yet.
    Queued,
    /// Stopped with an error.
    Failed,
}

impl AgStatus {
    /// Stable key (also the CSS tone).
    pub fn key(self) -> &'static str {
        match self {
            AgStatus::Running => "run",
            AgStatus::Done => "ok",
            AgStatus::NeedsYou => "warn",
            AgStatus::Queued => "idle",
            AgStatus::Failed => "bad",
        }
    }

    /// Words for a tag.
    pub fn label(self) -> &'static str {
        match self {
            AgStatus::Running => "Running",
            AgStatus::Done => "Done",
            AgStatus::NeedsYou => "Needs you",
            AgStatus::Queued => "Queued",
            AgStatus::Failed => "Failed",
        }
    }
}

/// Milliseconds as a short duration: `420ms`, `3.4s`, `2m 05s`.
pub fn dur(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{}.{}s", ms / 1000, ms % 1000 / 100)
    } else {
        format!("{}m {:02}s", ms / 60_000, ms / 1000 % 60)
    }
}

/// Euros from cents, two decimals: `€1.24`.
pub fn eur(cents: u32) -> String {
    format!("€{}.{:02}", cents / 100, cents % 100)
}

/// A token count, short: `840`, `12.4k`, `1.2M`.
pub fn toks(n: u32) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{}.{}k", n / 1000, n % 1000 / 100)
    } else {
        format!("{}.{}M", n / 1_000_000, n % 1_000_000 / 100_000)
    }
}

// ───────────────────────────────────────────────────────── Orchestration ──

/// One agent in an orchestrated run.
#[derive(Clone, Debug, PartialEq)]
pub struct OrchNode {
    /// Id.
    pub id: u32,
    /// The agent that handed it the work (none: the orchestrator).
    pub parent: Option<u32>,
    /// Name.
    pub name: &'static str,
    /// Role.
    pub role: &'static str,
    /// Where it stands.
    pub status: AgStatus,
    /// What it is doing, in one line.
    pub task: &'static str,
    /// Model.
    pub model: &'static str,
    /// Cost so far, cents (its own work only).
    pub cents: u32,
    /// Time so far, ms (its own work only).
    pub ms: u64,
    /// Tokens so far.
    pub tokens: u32,
}

#[allow(clippy::too_many_arguments)]
fn on(
    id: u32,
    parent: Option<u32>,
    name: &'static str,
    role: &'static str,
    status: AgStatus,
    task: &'static str,
    model: &'static str,
    cents: u32,
    ms: u64,
    tokens: u32,
) -> OrchNode {
    OrchNode {
        id,
        parent,
        name,
        role,
        status,
        task,
        model,
        cents,
        ms,
        tokens,
    }
}

/// The run's agents: an orchestrator, four leads by role and their helpers (empty when `sample` is
/// false).
pub fn orch_tree(sample: bool) -> Vec<OrchNode> {
    use AgStatus::*;
    if !sample {
        return Vec::new();
    }
    vec![
        on(
            1,
            None,
            "Conductor",
            "Orchestrator",
            Running,
            "Splitting the harbour market brief across the crew",
            "Atlas Pro",
            38,
            41_000,
            18_200,
        ),
        on(
            2,
            Some(1),
            "Scout",
            "Research",
            Running,
            "Reading ferry operator reports",
            "Atlas Swift",
            22,
            63_000,
            31_400,
        ),
        on(
            3,
            Some(2),
            "Scout · ports",
            "Research",
            Done,
            "Listed twelve harbour towns with ferry traffic",
            "Atlas Swift",
            9,
            24_000,
            12_800,
        ),
        on(
            4,
            Some(2),
            "Scout · prices",
            "Research",
            Running,
            "Comparing berth fees across operators",
            "Atlas Swift",
            7,
            19_000,
            9_900,
        ),
        on(
            5,
            Some(2),
            "Scout · press",
            "Research",
            Failed,
            "Local news archive refused the request",
            "Atlas Swift",
            2,
            6_000,
            1_300,
        ),
        on(
            6,
            Some(1),
            "Tally",
            "Analysis",
            NeedsYou,
            "Two sources disagree on 2025 passenger numbers",
            "Atlas Pro",
            15,
            22_000,
            8_700,
        ),
        on(
            7,
            Some(6),
            "Tally · sizing",
            "Analysis",
            Done,
            "Sized the market three ways",
            "Atlas Pro",
            11,
            17_000,
            6_100,
        ),
        on(
            8,
            Some(1),
            "Quill",
            "Writing",
            Queued,
            "Waits for the analysis",
            "Atlas Pro",
            0,
            0,
            0,
        ),
        on(
            9,
            Some(8),
            "Quill · outline",
            "Writing",
            Done,
            "Drafted the report outline",
            "Atlas Swift",
            3,
            5_000,
            2_400,
        ),
        on(
            10,
            Some(1),
            "Proof",
            "Review",
            Queued,
            "Checks every figure against its source",
            "Atlas Pro",
            0,
            0,
            0,
        ),
    ]
}

/// The agents a node handed work to, in order.
pub fn orch_children(nodes: &[OrchNode], id: u32) -> Vec<OrchNode> {
    nodes
        .iter()
        .filter(|n| n.parent == Some(id))
        .cloned()
        .collect()
}

/// A branch's cost (cents) and time (ms): the node and everything under it.
pub fn orch_rollup(nodes: &[OrchNode], id: u32) -> (u32, u64) {
    let Some(n) = nodes.iter().find(|n| n.id == id) else {
        return (0, 0);
    };
    let mut c = n.cents;
    let mut t = n.ms;
    for k in nodes.iter().filter(|k| k.parent == Some(id)) {
        let (a, b) = orch_rollup(nodes, k.id);
        c += a;
        t += b;
    }
    (c, t)
}

/// The next running agent after `after` (wrapping), for "Go to next running agent".
pub fn orch_next_running(nodes: &[OrchNode], after: Option<u32>) -> Option<u32> {
    let run: Vec<u32> = nodes
        .iter()
        .filter(|n| n.status == AgStatus::Running)
        .map(|n| n.id)
        .collect();
    if run.is_empty() {
        return None;
    }
    let i = after
        .and_then(|a| run.iter().position(|x| *x == a))
        .map(|i| (i + 1) % run.len())
        .unwrap_or(0);
    Some(run[i])
}

/// Agents done of all.
pub fn orch_done(nodes: &[OrchNode]) -> (usize, usize) {
    (
        nodes.iter().filter(|n| n.status == AgStatus::Done).count(),
        nodes.len(),
    )
}

/// A branch's ancestors (to open every branch down to an agent).
pub fn orch_path(nodes: &[OrchNode], id: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut cur = nodes.iter().find(|n| n.id == id).and_then(|n| n.parent);
    while let Some(p) = cur {
        out.push(p);
        cur = nodes.iter().find(|n| n.id == p).and_then(|n| n.parent);
    }
    out
}

/// One call in the run's trace.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceCall {
    /// Id.
    pub id: u32,
    /// The call it ran inside.
    pub parent: Option<u32>,
    /// What ran.
    pub label: &'static str,
    /// The agent that ran it.
    pub agent: u32,
    /// Start, ms from the run's start.
    pub at: u64,
    /// Length, ms (0 while it runs: it grows with the clock).
    pub ms: u64,
    /// It failed.
    pub err: bool,
}

fn tc(
    id: u32,
    parent: Option<u32>,
    label: &'static str,
    agent: u32,
    at: u64,
    ms: u64,
    err: bool,
) -> TraceCall {
    TraceCall {
        id,
        parent,
        label,
        agent,
        at,
        ms,
        err,
    }
}

/// The run's trace (calls nest under the agent turn that made them).
pub fn orch_trace(sample: bool) -> Vec<TraceCall> {
    if !sample {
        return Vec::new();
    }
    vec![
        tc(1, None, "Conductor · plan the run", 1, 0, 6_200, false),
        tc(2, Some(1), "model · split the brief", 1, 400, 4_800, false),
        tc(3, None, "Scout · research", 2, 6_400, 0, false),
        tc(
            4,
            Some(3),
            "web.search · harbour towns",
            3,
            7_000,
            9_800,
            false,
        ),
        tc(
            5,
            Some(3),
            "files.read · operator report",
            4,
            9_200,
            12_600,
            false,
        ),
        tc(
            6,
            Some(3),
            "web.fetch · news archive",
            5,
            11_000,
            2_100,
            true,
        ),
        tc(7, None, "Tally · size the market", 6, 17_500, 14_000, false),
        tc(
            8,
            Some(7),
            "sheet.calc · three models",
            7,
            18_200,
            8_400,
            false,
        ),
        tc(9, None, "Quill · outline", 9, 27_000, 5_000, false),
    ]
}

/// The trace rows shown with some calls folded (children of a folded call are hidden).
pub fn trace_rows(calls: &[TraceCall], folded: &[u32]) -> Vec<TraceCall> {
    calls
        .iter()
        .filter(|c| c.parent.is_none_or(|p| !folded.contains(&p)))
        .cloned()
        .collect()
}

/// A call's length at `elapsed` ms into the run (a running call grows).
pub fn call_ms(c: &TraceCall, elapsed: u64) -> u64 {
    if c.ms > 0 {
        c.ms
    } else {
        elapsed.saturating_sub(c.at)
    }
}

/// The trace's time axis: from 0 to the end of the latest call.
pub fn trace_span(calls: &[TraceCall], elapsed: u64) -> u64 {
    calls
        .iter()
        .map(|c| c.at + call_ms(c, elapsed))
        .max()
        .unwrap_or(0)
        .max(1)
}

/// One event in the run log.
#[derive(Clone, Debug, PartialEq)]
pub struct RunEvent {
    /// When, ms from the run's start.
    pub at: u64,
    /// It is an error.
    pub err: bool,
    /// What happened.
    pub text: &'static str,
}

/// The run's events.
pub fn orch_events(sample: bool) -> Vec<RunEvent> {
    if !sample {
        return Vec::new();
    }
    let e = |at, err, text| RunEvent { at, err, text };
    vec![
        e(0, false, "Run started by Rowan Vale"),
        e(
            6_200,
            false,
            "Conductor handed work to Scout, Tally, Quill and Proof",
        ),
        e(13_100, true, "Scout · press: the news archive answered 403"),
        e(17_500, false, "Tally started sizing the market"),
        e(31_500, false, "Tally asked you to pick a passenger figure"),
    ]
}

/// Errors in the event log.
pub fn error_count(events: &[RunEvent]) -> usize {
    events.iter().filter(|e| e.err).count()
}

/// The run's cost in cents after `elapsed` ms (a cent every three seconds while it runs).
pub fn orch_cents(nodes: &[OrchNode], elapsed: u64) -> u32 {
    if nodes.is_empty() {
        return 0;
    }
    nodes.iter().map(|n| n.cents).sum::<u32>() + (elapsed / 3000) as u32
}

/// Zoom steps for the canvas, percent.
pub const ZOOMS: [u32; 7] = [50, 67, 80, 100, 125, 150, 200];

/// The next zoom step up (`dir` > 0) or down from `z`.
pub fn zoom_step(z: u32, dir: i32) -> u32 {
    if dir > 0 {
        ZOOMS.iter().copied().find(|x| *x > z).unwrap_or(200)
    } else {
        ZOOMS.iter().rev().copied().find(|x| *x < z).unwrap_or(50)
    }
}

// ─────────────────────────────────────────────────────────────── Thread ──

/// One call inside a folded tool line.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRun {
    /// Tool and what it did.
    pub label: &'static str,
    /// How long it took, ms.
    pub ms: u64,
}

/// One entry in the conversation with the orchestrator.
#[derive(Clone, Debug, PartialEq)]
pub enum ThreadItem {
    /// The person.
    Me {
        /// Text.
        text: String,
        /// Time.
        at: &'static str,
    },
    /// The orchestrator.
    Agent {
        /// Text.
        text: String,
        /// Time.
        at: &'static str,
    },
    /// Calls that ran side by side, folded into one line.
    Tools(Vec<ToolRun>),
    /// A task the orchestrator started.
    Task {
        /// Task id.
        id: u32,
        /// Name.
        name: &'static str,
        /// Current step.
        step: &'static str,
        /// Steps done.
        done: u8,
        /// Steps in all.
        of: u8,
    },
    /// Files it made.
    Files(Vec<&'static str>),
    /// A marker after time away.
    Away {
        /// Hours away.
        hours: u32,
        /// What finished meanwhile.
        what: &'static str,
    },
}

/// The sample conversation (empty when `sample` is false).
pub fn thread_items(sample: bool) -> Vec<ThreadItem> {
    use ThreadItem::*;
    if !sample {
        return Vec::new();
    }
    let t = |label, ms| ToolRun { label, ms };
    vec![
        Me {
            text: "Get the harbour launch ready for Thursday: venue, guest list and the press note.".into(),
            at: "09:12",
        },
        Tools(vec![
            t("calendar.find · Thursday", 820),
            t("files.read · guest list", 1_400),
            t("mail.search · venue threads", 2_300),
        ]),
        Agent {
            text: "Thursday is clear from 17:00. I split this into three tasks and started them; the press note needs a quote from you before it goes out.".into(),
            at: "09:13",
        },
        Task {
            id: 21,
            name: "Book the venue",
            step: "Holding the pier hall until 12:00",
            done: 1,
            of: 3,
        },
        Task {
            id: 22,
            name: "Press note",
            step: "Waiting for your quote",
            done: 2,
            of: 4,
        },
        Away {
            hours: 3,
            what: "Guest list finished; two venues replied.",
        },
        Files(vec!["guest-list.csv", "press-note-draft.md"]),
        Me {
            text: "Use the pier hall. Quote: “Small boats, big evenings.”".into(),
            at: "12:40",
        },
        Agent {
            text: "Done – the pier hall is booked and the quote is in the note. I will send it to the three local papers at 15:00 unless you stop me.".into(),
            at: "12:41",
        },
    ]
}

/// The folded line for calls that ran side by side: `Ran 3 tools · 2.3s` (time is the longest).
pub fn tools_line(calls: &[ToolRun]) -> String {
    let ms = calls.iter().map(|c| c.ms).max().unwrap_or(0);
    let n = calls.len();
    let w = if n == 1 { "tool" } else { "tools" };
    format!("Ran {n} {w} · {}", dur(ms))
}

/// Tasks running and waiting on the person (the header chips).
pub fn thread_counts(sample: bool) -> (u32, u32) {
    if sample { (9, 5) } else { (0, 0) }
}

/// The orchestrator's reply to a message (fictional and deterministic).
pub fn thread_reply(text: &str) -> String {
    let t = text.to_lowercase();
    if t.contains("stop") || t.contains("cancel") {
        "Stopped. Nothing else goes out until you say so.".to_string()
    } else if t.contains("?") {
        "Short answer: yes. I checked the open tasks and nothing blocks it; I will note it on the Harbor launch board.".to_string()
    } else {
        "On it. I added this to Harbor launch and will tell you here when it needs you.".to_string()
    }
}

/// How a streamed reply stands `elapsed` ms after sending: the words shown, and whether the
/// connection is down. Words arrive every 70 ms after a 600 ms think; the stream drops for
/// `drop_ms` once `drop_at` words are out (0: never drops) and then picks up where it stopped.
pub fn stream_at(words: usize, elapsed: u64, drop_at: usize, drop_ms: u64) -> (usize, bool) {
    if elapsed < 600 {
        return (0, false);
    }
    let n = ((elapsed - 600) / 70) as usize;
    if drop_at == 0 || n < drop_at {
        return (n.min(words), false);
    }
    let after = elapsed - 600 - drop_at as u64 * 70;
    if after < drop_ms {
        return (drop_at.min(words), true);
    }
    let m = drop_at + ((after - drop_ms) / 70) as usize;
    (m.min(words), false)
}

/// The first `n` words of `text`.
pub fn first_words(text: &str, n: usize) -> String {
    text.split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

/// An upload's percent after `elapsed` ms on a slow link (slows near the end, never past 100).
pub fn upload_pct(elapsed: u64, total_ms: u64) -> u32 {
    if total_ms == 0 || elapsed >= total_ms {
        return 100;
    }
    let x = elapsed as f64 / total_ms as f64;
    ((1.0 - (1.0 - x).powi(2)) * 100.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_numbers() {
        assert_eq!(dur(420), "420ms");
        assert_eq!(dur(3_450), "3.4s");
        assert_eq!(dur(125_000), "2m 05s");
        assert_eq!(eur(124), "€1.24");
        assert_eq!(toks(840), "840");
        assert_eq!(toks(12_480), "12.4k");
        assert_eq!(toks(1_250_000), "1.2M");
        assert_eq!(AgStatus::NeedsYou.label(), "Needs you");
        assert_eq!(AgStatus::Failed.key(), "bad");
    }

    #[test]
    fn orchestration_tree() {
        let n = orch_tree(true);
        assert!(orch_tree(false).is_empty());
        assert_eq!(orch_children(&n, 1).len(), 4);
        assert_eq!(orch_children(&n, 2).len(), 3);
        // Scout's branch: itself plus its three helpers.
        assert_eq!(
            orch_rollup(&n, 2),
            (22 + 9 + 7 + 2, 63_000 + 24_000 + 19_000 + 6_000)
        );
        assert_eq!(orch_rollup(&n, 99), (0, 0));
        assert_eq!(orch_done(&n), (3, 10));
        assert_eq!(orch_path(&n, 4), vec![2, 1]);
        // Next running wraps round.
        assert_eq!(orch_next_running(&n, None), Some(1));
        assert_eq!(orch_next_running(&n, Some(1)), Some(2));
        assert_eq!(orch_next_running(&n, Some(4)), Some(1));
        assert_eq!(orch_next_running(&[], None), None);
    }

    #[test]
    fn orchestration_trace_and_zoom() {
        let c = orch_trace(true);
        assert_eq!(trace_rows(&c, &[]).len(), 9);
        assert_eq!(trace_rows(&c, &[3]).len(), 6);
        let running = &c[2];
        assert_eq!(call_ms(running, 10_000), 3_600);
        assert_eq!(call_ms(&c[0], 99_000), 6_200);
        assert_eq!(trace_span(&c, 40_000), 40_000);
        assert_eq!(trace_span(&[], 0), 1);
        let e = orch_events(true);
        assert_eq!(error_count(&e), 1);
        assert_eq!(orch_cents(&orch_tree(true), 9_000), 107 + 3);
        assert_eq!(orch_cents(&[], 9_000), 0);
        assert_eq!(zoom_step(100, 1), 125);
        assert_eq!(zoom_step(100, -1), 80);
        assert_eq!(zoom_step(200, 1), 200);
        assert_eq!(zoom_step(50, -1), 50);
    }

    #[test]
    fn thread_lines_and_stream() {
        let calls = vec![
            ToolRun {
                label: "a",
                ms: 820,
            },
            ToolRun {
                label: "b",
                ms: 2_300,
            },
        ];
        assert_eq!(tools_line(&calls), "Ran 2 tools · 2.3s");
        assert_eq!(tools_line(&calls[..1]), "Ran 1 tool · 820ms");
        assert_eq!(thread_items(true).len(), 9);
        assert!(thread_items(false).is_empty());
        assert_eq!(thread_counts(true), (9, 5));
        assert!(thread_reply("stop that").starts_with("Stopped"));
        assert!(thread_reply("can we?").starts_with("Short answer"));
        // Stream: think, words, a drop that holds, then the rest.
        assert_eq!(stream_at(20, 500, 0, 0), (0, false));
        assert_eq!(stream_at(20, 600 + 70 * 5, 0, 0), (5, false));
        assert_eq!(stream_at(20, 600 + 70 * 8, 6, 2000), (6, true));
        assert_eq!(
            stream_at(20, 600 + 70 * 6 + 2000 + 70 * 3, 6, 2000),
            (9, false)
        );
        assert_eq!(stream_at(20, 99_000, 6, 2000), (20, false));
        assert_eq!(first_words("a b c d", 2), "a b");
        assert_eq!(upload_pct(0, 8000), 0);
        assert_eq!(upload_pct(4000, 8000), 75);
        assert_eq!(upload_pct(9000, 8000), 100);
    }
}
