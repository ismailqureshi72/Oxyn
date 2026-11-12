//! The run model behind family F (`docs/specs/tools.md`): tool calls, their row views, grouping
//! into stacks, plans and todo lists, and the sub-agent graph. Everything here is pure data and
//! pure functions of `now`, so every renderer shows the same thing.

pub mod args;
pub mod graph;
pub mod plan;
pub mod stack;

use std::time::Duration;

use crate::{ActivityKind, ClockMs, VerbPack, verbs};

pub use args::{ArgValue, args_preview, redact};
pub use graph::{AgentNode, NodeState, RunGraph, TreeRow, TreeVariant, TreeView};
pub use plan::{ItemStatus, Plan, PlanItem, PlanState, Progress, TodoDock};
pub use stack::{GroupClass, GroupState, StackItem, ToolDensity, ToolStackVariant};

/// What kind of work a call does (§2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolKind {
    /// Read a file or page.
    Read,
    /// List a directory.
    List,
    /// Search code.
    Search,
    /// Edit a file.
    Edit,
    /// Create a file.
    Create,
    /// Delete a file.
    Delete,
    /// Run a command.
    Shell,
    /// Search the web.
    WebSearch,
    /// Fetch a page.
    Fetch,
    /// Call an MCP server tool.
    Mcp,
    /// Write the todo list.
    Plan,
    /// Start a sub-agent.
    Subagent,
    /// Produce media.
    Media,
    /// Anything else.
    Generic,
}

impl ToolKind {
    /// Maps a raw tool name to a kind (the default `ToolKindMap`; apps may pass their own).
    pub fn from_name(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("mcp__") || lower.contains("::mcp") {
            return Self::Mcp;
        }
        // Words: split on separators and camelCase ("TodoWrite" → todo, write).
        let mut words: Vec<String> = vec![];
        let mut cur = String::new();
        let mut prev_lower = false;
        for ch in name.chars() {
            if !ch.is_alphanumeric() {
                if !cur.is_empty() {
                    words.push(std::mem::take(&mut cur));
                }
                prev_lower = false;
                continue;
            }
            if ch.is_uppercase() && prev_lower && !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            prev_lower = ch.is_lowercase();
            cur.extend(ch.to_lowercase());
        }
        if !cur.is_empty() {
            words.push(cur);
        }
        let has = |ws: &[&str]| words.iter().any(|w| ws.contains(&w.as_str()));
        if has(&["todo", "todos", "plan"]) {
            Self::Plan
        } else if has(&["subagent", "agent", "delegate", "spawn", "task"]) {
            Self::Subagent
        } else if has(&["websearch"]) || (has(&["web"]) && has(&["search"])) {
            Self::WebSearch
        } else if has(&["fetch", "browse", "http", "url", "web"]) {
            Self::Fetch
        } else if has(&["grep", "search", "find", "glob", "rg"]) {
            Self::Search
        } else if has(&["list", "ls", "tree", "dir"]) {
            Self::List
        } else if has(&["read", "view", "cat", "open"]) {
            Self::Read
        } else if has(&["create", "write", "new"]) {
            Self::Create
        } else if has(&["delete", "remove", "rm", "unlink"]) {
            Self::Delete
        } else if has(&["edit", "patch", "replace", "apply", "str"]) {
            Self::Edit
        } else if has(&["bash", "shell", "exec", "run", "terminal", "command", "sh"]) {
            Self::Shell
        } else if has(&["image", "video", "audio", "media", "generate"]) {
            Self::Media
        } else {
            Self::Generic
        }
    }

    /// Present and past verbs (neutral pack, `status-text.md` §5.3).
    pub fn verbs(self, pack: &VerbPack) -> (String, String) {
        let pair = |a: ActivityKind| {
            let p = pack.activity(a);
            (p.present, p.past)
        };
        match self {
            Self::Read => pair(ActivityKind::Read),
            Self::Search => pair(ActivityKind::Search),
            Self::Edit => pair(ActivityKind::Edit),
            Self::Create => pair(ActivityKind::Write),
            Self::Shell => pair(ActivityKind::Run),
            Self::WebSearch => ("Searching the web".into(), "Searched the web".into()),
            Self::Fetch => pair(ActivityKind::Fetch),
            Self::Plan => ("Planning".into(), "Updated plan".into()),
            Self::Media => pair(ActivityKind::Produce),
            Self::List => ("Listing".into(), "Listed".into()),
            Self::Delete => ("Deleting".into(), "Deleted".into()),
            Self::Mcp => ("Calling".into(), "Called".into()),
            Self::Subagent => ("Delegating".into(), "Delegated".into()),
            Self::Generic => ("Working".into(), "Ran".into()),
        }
    }

    /// Icon name (in-house icon set).
    pub fn icon(self) -> &'static str {
        match self {
            Self::Read => "page",
            Self::List => "folder",
            Self::Search => "search",
            Self::Edit | Self::Create | Self::Delete => "pencil",
            Self::Shell => "prompt",
            Self::WebSearch | Self::Fetch => "globe",
            Self::Mcp => "plug",
            Self::Plan => "checklist",
            Self::Subagent => "branch",
            Self::Media => "spark",
            Self::Generic => "cog",
        }
    }
}

/// What the call acts on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A file path (shown as its basename, middle-truncated).
    Path(String),
    /// A search query (quoted, ≤ 32 chars).
    Query(String),
    /// A URL (shown as its domain).
    Url(String),
    /// A command (shown as its head: first token + one arg).
    Command(String),
    /// Anything else.
    Label(String),
}

impl Target {
    /// The short form for the row.
    pub fn short(&self) -> String {
        match self {
            Self::Path(p) => middle_truncate(p.rsplit('/').next().unwrap_or(p), 28),
            Self::Query(q) => format!("\u{201c}{}\u{201d}", end_truncate(q, 32)),
            Self::Url(u) => {
                let rest = u.split("://").nth(1).unwrap_or(u);
                rest.split('/')
                    .next()
                    .unwrap_or(rest)
                    .trim_start_matches("www.")
                    .to_string()
            }
            Self::Command(c) => {
                let mut it = c.split_whitespace();
                let head: Vec<&str> = it.by_ref().take(2).collect();
                let more = it.next().is_some();
                format!("{}{}", head.join(" "), if more { " …" } else { "" })
            }
            Self::Label(l) => end_truncate(l, 32),
        }
    }

    /// The full text (tooltip, accessible name).
    pub fn full(&self) -> &str {
        match self {
            Self::Path(s) | Self::Query(s) | Self::Url(s) | Self::Command(s) | Self::Label(s) => s,
        }
    }
}

/// `src/…/auth.rs`-style truncation keeping both ends.
pub fn middle_truncate(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let head: String = s.chars().take(keep / 2).collect();
    let tail: String = s.chars().skip(n - (keep - keep / 2)).collect();
    format!("{head}…{tail}")
}

/// `long query te…`.
pub fn end_truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

/// Why a call failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolErrorKind {
    /// Non-zero exit.
    Exit,
    /// Timed out.
    Timeout,
    /// Transport or protocol error.
    Protocol,
    /// The tool reported an error.
    Tool,
    /// No network.
    Offline,
}

/// Who cancelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelledBy {
    /// The user stopped it.
    User,
    /// The run stopped.
    RunStopped,
    /// The approval timed out.
    ApprovalTimeout,
}

/// A call's lifecycle (§2.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolCallState {
    /// Created, not started.
    Pending,
    /// Arguments arriving.
    StreamingInput,
    /// Blocked on the user.
    AwaitingApproval,
    /// Just approved (≤ 400 ms).
    Approved,
    /// Executing, no output yet.
    Running,
    /// Output arriving; `progress` is `(done, total)` for multi-result tools.
    StreamingOutput {
        /// "Fetched 3 of 5".
        progress: Option<(u32, u32)>,
    },
    /// Finished OK.
    Succeeded {
        /// Short result ("12 matches").
        summary: Option<String>,
    },
    /// Failed.
    Failed {
        /// Why.
        kind: ToolErrorKind,
        /// Exit code.
        code: Option<i32>,
        /// First error line (stderr tail or message).
        message: Option<String>,
    },
    /// Stopped.
    Cancelled {
        /// By whom.
        by: CancelledBy,
    },
    /// User or policy said no.
    Denied {
        /// Why.
        reason: Option<String>,
    },
    /// The MCP server needs sign-in.
    AuthNeeded,
}

impl ToolCallState {
    /// Settled for good (until a retry).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded { .. }
                | Self::Failed { .. }
                | Self::Cancelled { .. }
                | Self::Denied { .. }
                | Self::AuthNeeded
        )
    }

    /// Working right now.
    pub fn is_live(&self) -> bool {
        matches!(
            self,
            Self::Pending
                | Self::StreamingInput
                | Self::Approved
                | Self::Running
                | Self::StreamingOutput { .. }
        )
    }
}

/// An MCP server's connection state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum McpStatus {
    /// Connected (no dot).
    #[default]
    Connected,
    /// Connecting.
    Connecting,
    /// Needs sign-in.
    NeedsAuth,
    /// Couldn't fetch tools.
    ToolsFetchFailed,
    /// Failed.
    Failed,
    /// Disabled.
    Disabled,
}

/// MCP server identity on a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpServer {
    /// Display name ("Linear").
    pub name: String,
    /// Connection state.
    pub status: McpStatus,
}

/// Attempt counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    /// This attempt (1-based).
    pub n: u8,
    /// Most attempts allowed.
    pub max: u8,
    /// An agent retry is scheduled for this time.
    pub next_at: Option<ClockMs>,
}

impl Default for Attempt {
    fn default() -> Self {
        Self {
            n: 1,
            max: 1,
            next_at: None,
        }
    }
}

/// One tool invocation, as the adapter keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// Adapter id, unique within the run.
    pub id: String,
    /// Kind.
    pub kind: ToolKind,
    /// Raw tool name.
    pub name: String,
    /// MCP server, if any.
    pub server: Option<McpServer>,
    /// What it acts on.
    pub target: Option<Target>,
    /// Arguments (possibly partial).
    pub args: Option<ArgValue>,
    /// Lifecycle state.
    pub state: ToolCallState,
    /// Created.
    pub created_at: ClockMs,
    /// Started running.
    pub started_at: Option<ClockMs>,
    /// Settled.
    pub ended_at: Option<ClockMs>,
    /// Time spent waiting for approval (excluded from the duration).
    pub paused: Duration,
    /// Last event (output, state change) for stall detection.
    pub last_event_at: ClockMs,
    /// Output text, if any.
    pub output: Option<String>,
    /// Edit counts `(+added, −removed)`.
    pub lines: Option<(u32, u32)>,
    /// Attempt.
    pub attempt: Attempt,
    /// Parent sub-agent call.
    pub parent: Option<String>,
}

impl ToolCall {
    /// A pending call created at `at`.
    pub fn new(id: impl Into<String>, name: impl Into<String>, at: ClockMs) -> Self {
        let name = name.into();
        Self {
            id: id.into(),
            kind: ToolKind::from_name(&name),
            name,
            server: None,
            target: None,
            args: None,
            state: ToolCallState::Pending,
            created_at: at,
            started_at: None,
            ended_at: None,
            paused: Duration::ZERO,
            last_event_at: at,
            output: None,
            lines: None,
            attempt: Attempt::default(),
            parent: None,
        }
    }

    /// Builder: target.
    pub fn with_target(mut self, t: Target) -> Self {
        self.target = Some(t);
        self
    }

    /// Builder: kind.
    pub fn with_kind(mut self, k: ToolKind) -> Self {
        self.kind = k;
        self
    }

    /// Builder: state.
    pub fn with_state(mut self, s: ToolCallState) -> Self {
        self.state = s;
        self
    }

    /// Builder: started and (optionally) ended.
    pub fn timed(mut self, start: ClockMs, end: Option<ClockMs>) -> Self {
        self.started_at = Some(start);
        self.ended_at = end;
        self.last_event_at = end.unwrap_or(start);
        self
    }

    /// Working time (excludes approval waits); `None` when never started.
    pub fn duration(&self, now: ClockMs) -> Option<Duration> {
        let s = self.started_at?;
        let e = self.ended_at.unwrap_or(now);
        Some(e.since(s).saturating_sub(self.paused))
    }
}

/// Call duration: `0.8s` under 10 s, then `1m 05s`; never `0s`.
pub fn format_call_duration(d: Duration) -> String {
    if d < Duration::from_secs(10) {
        let tenths = (d.as_millis() / 100).max(1);
        format!("{}.{}s", tenths / 10, tenths % 10)
    } else {
        verbs::format_units(d)
    }
}

/// Row timings (`tool-call.md` §6).
pub mod timing {
    use std::time::Duration;

    /// Calls settling inside this render final (no loader, no roll).
    pub const ROW_GUARD: Duration = Duration::from_millis(150);
    /// Timer appears after this.
    pub const TIMER_AFTER: Duration = Duration::from_secs(1);
    /// ✓ in success tone before the kind icon returns.
    pub const SUCCESS_HOLD: Duration = Duration::from_millis(600);
    /// Verb tense roll.
    pub const TENSE_ROLL: Duration = Duration::from_millis(240);
    /// Failure glyph nudge.
    pub const NUDGE: Duration = Duration::from_millis(300);
    /// Copied ✓.
    pub const COPIED: Duration = Duration::from_millis(1200);
    /// A call with no event for this long is stalled.
    pub const TOOL_STALL: Duration = Duration::from_secs(120);
    /// Approved flash.
    pub const APPROVED: Duration = Duration::from_millis(400);
    /// Auto-collapse a system-opened shell tail after success.
    pub const AUTO_COLLAPSE: Duration = Duration::from_millis(1200);
}

/// The leading glyph of a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowGlyph {
    /// ○ not started.
    Hollow,
    /// Animated loader (busy region).
    Loader,
    /// The kind icon.
    Kind(ToolKind),
    /// ✓.
    Check,
    /// ✕.
    Cross,
    /// ■.
    Stop,
    /// ⊘.
    Denied,
    /// Hand: waiting for you.
    Waiting,
    /// Key: needs sign-in.
    Key,
}

impl RowGlyph {
    /// Text fallback (terminal variant).
    pub fn text(self) -> &'static str {
        match self {
            Self::Hollow => "○",
            Self::Loader => "•",
            Self::Kind(_) => "•",
            Self::Check => "✓",
            Self::Cross => "✕",
            Self::Stop => "■",
            Self::Denied => "⊘",
            Self::Waiting => "⏸",
            Self::Key => "⚿",
        }
    }
}

/// Semantic tone (maps to `status.*` tokens; colour is never the only cue).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Secondary text.
    Muted,
    /// Default text.
    Default,
    /// Success.
    Success,
    /// Danger.
    Danger,
    /// Waiting on the user.
    Waiting,
    /// Warning (stall).
    Warn,
}

/// Actions a row offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowAction {
    /// Retry the call.
    Retry,
    /// Stop it.
    Stop,
    /// Connect the MCP server.
    Connect,
    /// Retry now (skip backoff).
    RetryNow,
}

impl RowAction {
    /// Button label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Retry => "Retry",
            Self::Stop => "Stop",
            Self::Connect => "Connect",
            Self::RetryNow => "Retry now",
        }
    }
}

/// One frame of a tool-call row.
#[derive(Clone, Debug, PartialEq)]
pub struct RowView {
    /// Leading glyph.
    pub glyph: RowGlyph,
    /// Glyph tone.
    pub tone: Tone,
    /// Verb, in the tense of the state.
    pub verb: String,
    /// Previous-tense verb while the tense roll runs, with progress `0..=1`.
    pub roll: Option<(String, f32)>,
    /// Short target.
    pub target: Option<String>,
    /// Argument preview (dim; Mcp / Generic).
    pub preview: Option<String>,
    /// Shimmer the preview / label (streaming input, active).
    pub shimmer: bool,
    /// Duration text (hidden under 1 s while live).
    pub duration: Option<String>,
    /// Timer paused (approval).
    pub paused: bool,
    /// Result summary or state line ("12 matches", "Waiting for you", "Stopped after 4s").
    pub note: Option<String>,
    /// Edit counts.
    pub lines: Option<(u32, u32)>,
    /// Always-visible error line.
    pub error_line: Option<String>,
    /// Attempt pill ("2/3").
    pub attempt: Option<String>,
    /// Actions.
    pub actions: Vec<RowAction>,
    /// Target struck through (denied).
    pub struck: bool,
    /// Stall overlay text.
    pub stalled: Option<String>,
    /// MCP badge `(name, status)`.
    pub server: Option<(String, McpStatus)>,
    /// The detail opens by itself (approval, failure).
    pub auto_expand: bool,
    /// Horizontal glyph nudge in px (failure).
    pub nudge_px: f32,
    /// Accessible name.
    pub aria: String,
    /// Animate (false for calls that settled inside the row guard and restored threads).
    pub animate: bool,
}

pub(crate) fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The row frame for `call` at `now`.
pub fn row_view(call: &ToolCall, pack: &VerbPack, now: ClockMs) -> RowView {
    use ToolCallState as S;
    let (present, past) = call.kind.verbs(pack);
    let target = call.target.as_ref().map(Target::short);
    let preview = match call.kind {
        ToolKind::Mcp | ToolKind::Generic => call.args.as_ref().map(|a| args_preview(a, 2, 24)),
        _ => None,
    }
    .filter(|p| !p.is_empty());
    let dur = call.duration(now);
    let fast = matches!((call.started_at, call.ended_at), (Some(s), Some(e)) if e.since(s) < timing::ROW_GUARD)
        || (call.started_at.is_none() && call.ended_at.is_some());
    let since_end = call.ended_at.map(|e| now.since(e));
    let live_dur = dur
        .filter(|d| *d >= timing::TIMER_AFTER)
        .map(format_call_duration);
    let settled_dur = dur.map(format_call_duration);
    let stalled = (call.state.is_live()
        && matches!(call.state, S::Running | S::StreamingOutput { .. })
        && now.since(call.last_event_at) >= timing::TOOL_STALL)
        .then(|| {
            format!(
                "No output for {}",
                verbs::format_summary(now.since(call.last_event_at))
            )
        });
    let roll = match since_end {
        Some(t) if !fast && t < timing::TENSE_ROLL && matches!(call.state, S::Succeeded { .. }) => {
            Some((
                present.clone(),
                t.as_secs_f32() / timing::TENSE_ROLL.as_secs_f32(),
            ))
        }
        _ => None,
    };
    let server = call.server.as_ref().map(|s| (s.name.clone(), s.status));
    let mut v = RowView {
        glyph: RowGlyph::Hollow,
        tone: Tone::Muted,
        verb: present.clone(),
        roll,
        target: target.clone(),
        preview,
        shimmer: false,
        duration: None,
        paused: false,
        note: None,
        lines: call.lines,
        error_line: None,
        attempt: (call.attempt.max > 1 && call.attempt.n > 1)
            .then(|| format!("{}/{}", call.attempt.n, call.attempt.max)),
        actions: vec![],
        struck: false,
        stalled: stalled.clone(),
        server,
        auto_expand: false,
        nudge_px: 0.0,
        aria: String::new(),
        animate: !fast,
    };
    let tgt = call
        .target
        .as_ref()
        .map(|t| t.full().to_string())
        .unwrap_or_else(|| call.name.clone());
    match &call.state {
        S::Pending => {
            v.aria = format!("{present} {tgt}, queued");
        }
        S::StreamingInput => {
            v.shimmer = true;
            v.aria = format!("Preparing {}", present.to_lowercase());
        }
        S::AwaitingApproval => {
            v.glyph = RowGlyph::Waiting;
            v.tone = Tone::Waiting;
            v.note = Some("Waiting for you".into());
            v.paused = true;
            v.duration = live_dur;
            v.auto_expand = true;
            v.aria = format!("{present} {tgt}, waiting for your approval");
        }
        S::Approved => {
            v.glyph = RowGlyph::Check;
            v.tone = Tone::Waiting;
            v.aria = format!("{present} {tgt}, approved");
        }
        S::Running | S::StreamingOutput { .. } => {
            v.glyph = RowGlyph::Loader;
            v.tone = if stalled.is_some() {
                Tone::Warn
            } else {
                Tone::Default
            };
            v.duration = live_dur.clone();
            if let S::StreamingOutput {
                progress: Some((d, t)),
            } = &call.state
            {
                v.note = Some(format!("{d} of {t}"));
            }
            if matches!(call.kind, ToolKind::Shell | ToolKind::Mcp) || stalled.is_some() {
                v.actions.push(RowAction::Stop);
            }
            v.aria = format!(
                "{present} {tgt}{}",
                dur.filter(|d| *d >= timing::TIMER_AFTER)
                    .map(|d| format!(", {} seconds", d.as_secs()))
                    .unwrap_or_default()
            );
        }
        S::Succeeded { summary } => {
            v.verb = past.clone();
            let hold = since_end.is_some_and(|t| t < timing::SUCCESS_HOLD) && !fast;
            v.glyph = if hold {
                RowGlyph::Check
            } else {
                RowGlyph::Kind(call.kind)
            };
            v.tone = if hold { Tone::Success } else { Tone::Muted };
            v.duration = settled_dur.clone();
            v.note = summary.clone();
            v.aria = format!(
                "{past} {tgt}, succeeded{}",
                dur.map(|d| format!(", {:.1} seconds", d.as_secs_f32()))
                    .unwrap_or_default()
            );
        }
        S::Failed {
            kind,
            code,
            message,
        } => {
            v.verb = past.clone();
            v.glyph = RowGlyph::Cross;
            v.tone = Tone::Danger;
            v.duration = settled_dur.clone();
            let reason = match (kind, code) {
                (ToolErrorKind::Exit, Some(c)) => format!("exit {c}"),
                (ToolErrorKind::Timeout, _) => format!(
                    "Timed out after {}",
                    dur.map(verbs::format_summary)
                        .unwrap_or_else(|| "a while".into())
                ),
                (ToolErrorKind::Offline, _) => "Offline".into(),
                (ToolErrorKind::Protocol, _) => "Protocol error".into(),
                _ => "Tool error".into(),
            };
            v.error_line = Some(match message {
                Some(m) => format!("{reason} · {m}"),
                None => reason.clone(),
            });
            if let Some(at) = call.attempt.next_at.filter(|at| *at > now) {
                let secs = at.since(now).as_secs_f32().ceil() as u64;
                v.note = Some(format!(
                    "Retrying in {secs}s · {}/{}",
                    call.attempt.n + 1,
                    call.attempt.max
                ));
                v.actions.push(RowAction::RetryNow);
            } else {
                v.actions.push(RowAction::Retry);
            }
            v.auto_expand = true;
            if let Some(t) = since_end.filter(|t| *t < timing::NUDGE && !fast) {
                let p = t.as_secs_f32() / timing::NUDGE.as_secs_f32();
                v.nudge_px = 3.0 * (1.0 - p) * (p * 2.0 * std::f32::consts::TAU).sin();
            }
            v.aria = format!(
                "{past} {tgt}, failed{}",
                code.map(|c| format!(", exit code {c}")).unwrap_or_default()
            );
        }
        S::Cancelled { .. } => {
            v.verb = past.clone();
            v.glyph = RowGlyph::Stop;
            v.tone = Tone::Muted;
            let after = dur.map(verbs::format_summary);
            v.note = Some(match &after {
                Some(a) => format!("Stopped after {a}"),
                None => "Stopped".into(),
            });
            v.actions.push(RowAction::Retry);
            v.aria = format!(
                "Stopped {tgt}{}",
                after.map(|a| format!(" after {a}")).unwrap_or_default()
            );
        }
        S::Denied { reason } => {
            v.glyph = RowGlyph::Denied;
            v.tone = Tone::Muted;
            v.struck = true;
            v.note = Some(match reason {
                Some(r) => format!("Denied: {r}"),
                None => "Denied".into(),
            });
            v.aria = v.note.clone().unwrap_or_default();
        }
        S::AuthNeeded => {
            v.glyph = RowGlyph::Key;
            v.tone = Tone::Waiting;
            let server = call
                .server
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "the server".into());
            v.note = Some(format!("Needs sign-in to {server}"));
            v.actions.push(RowAction::Connect);
            v.aria = format!("{server} needs sign-in");
        }
    }
    if let Some((name, _)) = &v.server {
        v.aria.push_str(&format!(", via {name}"));
    }
    v
}

/// One-call label in its final tense, for groups and tooltips ("Read auth.rs").
pub fn call_label(call: &ToolCall, pack: &VerbPack) -> String {
    let (present, past) = call.kind.verbs(pack);
    let verb = if call.state.is_terminal() {
        past
    } else {
        present
    };
    match &call.target {
        Some(t) => format!("{verb} {}", t.short()),
        None => verb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> &'static VerbPack {
        VerbPack::neutral()
    }

    #[test]
    fn kinds_from_names() {
        assert_eq!(ToolKind::from_name("read_file"), ToolKind::Read);
        assert_eq!(ToolKind::from_name("bash"), ToolKind::Shell);
        assert_eq!(ToolKind::from_name("grep"), ToolKind::Search);
        assert_eq!(
            ToolKind::from_name("mcp__linear__create_issue"),
            ToolKind::Mcp
        );
        assert_eq!(ToolKind::from_name("web_search"), ToolKind::WebSearch);
        assert_eq!(ToolKind::from_name("TodoWrite"), ToolKind::Plan);
        assert_eq!(ToolKind::from_name("frobnicate"), ToolKind::Generic);
        assert_eq!(ToolKind::from_name("str_replace_editor"), ToolKind::Edit);
        assert_eq!(ToolKind::from_name("ReadFile"), ToolKind::Read);
    }

    #[test]
    fn targets_shorten() {
        assert_eq!(
            Target::Path("src/middleware/auth.rs".into()).short(),
            "auth.rs"
        );
        assert_eq!(
            Target::Url("https://www.docs.rs/tokio/latest".into()).short(),
            "docs.rs"
        );
        assert_eq!(
            Target::Command("cargo test -p oxyn-core --all".into()).short(),
            "cargo test …"
        );
        assert_eq!(middle_truncate("abcdefghijklmnop", 9), "abcd…mnop");
    }

    #[test]
    fn durations() {
        assert_eq!(format_call_duration(Duration::from_millis(20)), "0.1s");
        assert_eq!(format_call_duration(Duration::from_millis(820)), "0.8s");
        assert_eq!(format_call_duration(Duration::from_secs(65)), "1m 05s");
    }

    #[test]
    fn row_states() {
        let c = ToolCall::new("1", "read_file", ClockMs(0))
            .with_target(Target::Path("src/auth.rs".into()))
            .with_state(ToolCallState::Running)
            .timed(ClockMs(0), None);
        let v = row_view(&c, pack(), ClockMs(500));
        assert_eq!((v.glyph, v.duration.clone()), (RowGlyph::Loader, None));
        let v = row_view(&c, pack(), ClockMs(1_500));
        assert_eq!(v.duration.as_deref(), Some("1.5s"));

        let done = c
            .clone()
            .with_state(ToolCallState::Succeeded {
                summary: Some("80 lines".into()),
            })
            .timed(ClockMs(0), Some(ClockMs(800)));
        let v = row_view(&done, pack(), ClockMs(900));
        assert_eq!((v.glyph, v.tone), (RowGlyph::Check, Tone::Success));
        assert!(v.roll.is_some());
        let v = row_view(&done, pack(), ClockMs(2_000));
        assert_eq!(v.glyph, RowGlyph::Kind(ToolKind::Read));
        assert_eq!(v.verb, "Read");
        assert_eq!(v.aria, "Read src/auth.rs, succeeded, 0.8 seconds");

        let fast = done.clone().timed(ClockMs(0), Some(ClockMs(40)));
        assert!(!row_view(&fast, pack(), ClockMs(50)).animate);

        let fail = ToolCall::new("2", "bash", ClockMs(0))
            .with_target(Target::Command("cargo test".into()))
            .with_state(ToolCallState::Failed {
                kind: ToolErrorKind::Exit,
                code: Some(101),
                message: Some("error[E0425]".into()),
            })
            .timed(ClockMs(0), Some(ClockMs(3_000)));
        let v = row_view(&fail, pack(), ClockMs(4_000));
        assert_eq!(v.error_line.as_deref(), Some("exit 101 · error[E0425]"));
        assert_eq!(v.actions, vec![RowAction::Retry]);
        assert!(v.aria.ends_with("failed, exit code 101"));

        let stalled = ToolCall::new("3", "bash", ClockMs(0))
            .with_state(ToolCallState::Running)
            .timed(ClockMs(0), None);
        let v = row_view(&stalled, pack(), ClockMs(130_000));
        assert_eq!(v.stalled.as_deref(), Some("No output for 2m 10s"));
        assert_eq!(v.tone, Tone::Warn);
    }
}
