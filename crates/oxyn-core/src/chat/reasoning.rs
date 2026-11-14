//! Reasoning and thinking state (`reasoning-block.md`).
//!
//! - **Reasoning block** (Arc UI Reasoning block, affectUI Thinking State): while the model
//!   thinks a shimmering label counts seconds and the newest lines stream under it; when it
//!   finishes the block folds to "Thought for 7s" and can open to the full reasoning, split into
//!   steps on `**Heading**` lines and blank lines. Stopped and error states keep what arrived.
//! - **Thinking traces** (Beautiful UI Thinking, MIT © 2026 Shane Levine): expandable traces of
//!   steps, reasoning, search and coding, each row with its own status.
//!
//! Display levels (Oxyn plus): `Normal` folds when done, `Thinking` keeps the preview, `Verbose`
//! stays open.

use std::time::Duration;

use crate::ClockMs;

/// Reasoning status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReasoningStatus {
    /// Thinking (shimmer, counting).
    Thinking,
    /// Finished.
    #[default]
    Done,
    /// Stopped by the user.
    Stopped,
    /// Interrupted by an error.
    Error,
}

impl ReasoningStatus {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "thinking" => Self::Thinking,
            "stopped" => Self::Stopped,
            "error" => Self::Error,
            _ => Self::Done,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Thinking => "thinking",
            Self::Done => "done",
            Self::Stopped => "stopped",
            Self::Error => "error",
        }
    }
}

/// Oxyn plus: how much reasoning is shown by default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayLevel {
    /// Fold to one line when done.
    #[default]
    Normal,
    /// Keep a two-line preview when done.
    Thinking,
    /// Stay open.
    Verbose,
}

impl DisplayLevel {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "thinking" => Self::Thinking,
            "verbose" => Self::Verbose,
            _ => Self::Normal,
        }
    }
}

/// One step of parsed reasoning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// `**Heading**` title.
    pub title: Option<String>,
    /// Body.
    pub text: String,
}

/// Splits streamed reasoning into steps: a line that is only `**Heading**` starts a titled step;
/// blank lines split paragraphs.
pub fn steps(src: &str) -> Vec<Step> {
    let mut out: Vec<Step> = Vec::new();
    let mut cur: Option<Step> = None;
    for para in src.split("\n\n") {
        // A blank line closes a titled step that already has text.
        if cur
            .as_ref()
            .is_some_and(|c| c.title.is_some() && !c.text.is_empty())
        {
            out.extend(cur.take());
        }
        let mut lines = para.lines().peekable();
        while let Some(l) = lines.next() {
            let t = l.trim();
            if t.len() > 4 && t.starts_with("**") && t.ends_with("**") {
                if let Some(c) = cur.take() {
                    out.push(c);
                }
                cur = Some(Step {
                    title: Some(t.trim_matches('*').to_string()),
                    text: String::new(),
                });
                continue;
            }
            if t.is_empty() {
                continue;
            }
            match &mut cur {
                Some(c) => {
                    if !c.text.is_empty() {
                        c.text.push(' ');
                    }
                    c.text.push_str(t);
                }
                None => {
                    cur = Some(Step {
                        title: None,
                        text: t.to_string(),
                    })
                }
            }
            if lines.peek().is_none() {
                // paragraph end: untitled paragraphs become their own step
                if cur.as_ref().is_some_and(|c| c.title.is_none()) {
                    out.extend(cur.take());
                }
            }
        }
    }
    out.extend(cur);
    out.retain(|s| s.title.is_some() || !s.text.is_empty());
    out
}

/// "7s", "1m 12s".
pub fn fmt_secs(s: u64) -> String {
    if s < 60 {
        format!("{s}s")
    } else {
        format!("{}m {}s", s / 60, s % 60)
    }
}

/// Spoken: "7 seconds", "1 minute 12 seconds".
pub fn spoken_secs(s: u64) -> String {
    let unit = |n: u64, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
    if s < 60 {
        unit(s, "second")
    } else {
        format!("{} {}", unit(s / 60, "minute"), unit(s % 60, "second"))
    }
}

/// The last `n` non-empty lines (the live preview while thinking).
pub fn preview(src: &str, n: usize) -> Vec<String> {
    let v: Vec<String> = src
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !(l.starts_with("**") && l.ends_with("**")))
        .map(str::to_string)
        .collect();
    v[v.len().saturating_sub(n)..].to_vec()
}

/// A reasoning block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reasoning {
    /// Status.
    pub status: ReasoningStatus,
    /// Reasoning so far.
    pub text: String,
    /// When thinking started.
    pub started: Option<ClockMs>,
    /// Measured or given seconds.
    pub seconds: Option<u64>,
    /// Full reasoning open.
    pub open: bool,
    /// Error reason.
    pub error: Option<String>,
    /// Display level.
    pub level: DisplayLevel,
}

impl Reasoning {
    /// A block.
    pub fn new(status: ReasoningStatus, text: &str) -> Self {
        Self {
            status,
            text: text.into(),
            started: None,
            seconds: None,
            open: false,
            error: None,
            level: DisplayLevel::Normal,
        }
    }
    /// Starts thinking at `now`.
    pub fn start(&mut self, now: ClockMs) {
        self.status = ReasoningStatus::Thinking;
        self.started = Some(now);
        self.seconds = None;
    }
    /// Appends streamed reasoning.
    pub fn push(&mut self, chunk: &str) {
        if self.status == ReasoningStatus::Thinking {
            self.text.push_str(chunk);
        }
    }
    fn settle(&mut self, now: ClockMs, st: ReasoningStatus) {
        if self.seconds.is_none() {
            self.seconds = self.started.map(|s| now.since(s).as_secs());
        }
        self.status = st;
        if self.level == DisplayLevel::Verbose {
            self.open = true;
        }
    }
    /// Done.
    pub fn finish(&mut self, now: ClockMs) {
        self.settle(now, ReasoningStatus::Done);
    }
    /// Stop pressed.
    pub fn stop(&mut self, now: ClockMs) {
        self.settle(now, ReasoningStatus::Stopped);
    }
    /// Failed.
    pub fn fail(&mut self, now: ClockMs, why: &str) {
        self.error = Some(why.into());
        self.settle(now, ReasoningStatus::Error);
    }
    /// Retry pressed: thinking again from scratch.
    pub fn retry(&mut self, now: ClockMs) {
        self.text.clear();
        self.error = None;
        self.start(now);
    }
    /// Seconds at `now`.
    pub fn secs(&self, now: ClockMs) -> Option<u64> {
        self.seconds
            .or_else(|| self.started.map(|s| now.since(s).as_secs()))
    }
    /// Header label.
    pub fn label(&self, now: ClockMs) -> String {
        match (self.status, self.secs(now)) {
            (ReasoningStatus::Thinking, _) => "Thinking".into(),
            (ReasoningStatus::Done, Some(s)) => format!("Thought for {}", fmt_secs(s)),
            (ReasoningStatus::Done, None) => "Thought".into(),
            (ReasoningStatus::Stopped, Some(s)) => format!("Stopped after {}", fmt_secs(s)),
            (ReasoningStatus::Stopped, None) => "Stopped".into(),
            (ReasoningStatus::Error, _) => "Thinking interrupted".into(),
        }
    }
    /// Polite announcement for the current status (never per chunk or per second).
    pub fn announce(&self, now: ClockMs) -> String {
        match (self.status, self.secs(now)) {
            (ReasoningStatus::Thinking, _) => "Thinking".into(),
            (ReasoningStatus::Done, Some(s)) => format!("Thought for {}", spoken_secs(s)),
            (ReasoningStatus::Done, None) => "Done thinking".into(),
            (ReasoningStatus::Stopped, _) => "Stopped".into(),
            (ReasoningStatus::Error, _) => "Thinking interrupted".into(),
        }
    }
    /// Whether the preview lines show (while thinking, or kept at the Thinking level).
    pub fn shows_preview(&self) -> bool {
        !self.open
            && (self.status == ReasoningStatus::Thinking
                || (self.level == DisplayLevel::Thinking && !self.text.is_empty()))
    }
    /// Header pressed.
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
}

// ─────────────────────────────────────────── traces ──

/// Trace kind (Beautiful UI Thinking).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TraceKind {
    /// Plan steps.
    #[default]
    Steps,
    /// Free reasoning.
    Reasoning,
    /// Web / doc search.
    Search,
    /// Coding (files and commands).
    Coding,
}

impl TraceKind {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "reasoning" => Self::Reasoning,
            "search" => Self::Search,
            "coding" => Self::Coding,
            _ => Self::Steps,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Steps => "steps",
            Self::Reasoning => "reasoning",
            Self::Search => "search",
            Self::Coding => "coding",
        }
    }
    /// Icon.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Steps => "checklist",
            Self::Reasoning => "brain",
            Self::Search => "globe",
            Self::Coding => "code",
        }
    }
}

/// A row status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowStatus {
    /// Not started.
    #[default]
    Pending,
    /// Running.
    Active,
    /// Finished.
    Done,
    /// Failed.
    Failed,
}

impl RowStatus {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// A trace row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceRow {
    /// Label.
    pub label: String,
    /// Detail (query, path, command).
    pub detail: Option<String>,
    /// Status.
    pub status: RowStatus,
}

impl TraceRow {
    /// A row.
    pub fn new(label: &str, detail: Option<&str>, status: RowStatus) -> Self {
        Self {
            label: label.into(),
            detail: detail.map(str::to_string),
            status,
        }
    }
}

/// An expandable thinking trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    /// Kind.
    pub kind: TraceKind,
    /// Rows.
    pub rows: Vec<TraceRow>,
    /// Expanded.
    pub open: bool,
}

impl Trace {
    /// A trace.
    pub fn new(kind: TraceKind, rows: Vec<TraceRow>) -> Self {
        Self {
            kind,
            rows,
            open: false,
        }
    }
    /// Rows revealed `elapsed` after start with one row per `step` (demo playback): earlier rows
    /// done, the current one active.
    pub fn playback(&self, elapsed: Duration, step: Duration) -> Vec<TraceRow> {
        let n = (elapsed.as_millis() / step.as_millis().max(1)) as usize;
        self.rows
            .iter()
            .enumerate()
            .take(n + 1)
            .map(|(i, r)| {
                let mut r = r.clone();
                if r.status != RowStatus::Failed {
                    r.status = if i < n {
                        RowStatus::Done
                    } else {
                        RowStatus::Active
                    };
                }
                r
            })
            .collect()
    }
    /// Header summary ("Searched 4 sources", "Edited 3 files").
    pub fn summary(&self, rows: &[TraceRow]) -> String {
        let done = rows.iter().filter(|r| r.status == RowStatus::Done).count();
        let active = rows.iter().any(|r| r.status == RowStatus::Active);
        let total = self.rows.len();
        match (self.kind, active) {
            (TraceKind::Steps, true) => format!("Working · step {} of {total}", done + 1),
            (TraceKind::Steps, false) => format!("Completed {done} steps"),
            (TraceKind::Reasoning, true) => "Reasoning…".into(),
            (TraceKind::Reasoning, false) => format!("Reasoned in {done} steps"),
            (TraceKind::Search, true) => format!("Searching · {done} sources so far"),
            (TraceKind::Search, false) => format!("Searched {done} sources"),
            (TraceKind::Coding, true) => format!("Coding · {done} of {total} done"),
            (TraceKind::Coding, false) => format!("Edited {done} files"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_split_on_headings_and_paragraphs() {
        let s = steps(
            "**Reading the test**\nIt fails at :59.\nOnly sometimes.\n\nA plain para.\n\n**Fix**\nAdd leeway.",
        );
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].title.as_deref(), Some("Reading the test"));
        assert_eq!(s[0].text, "It fails at :59. Only sometimes.");
        assert_eq!(s[1].title, None);
        assert_eq!(s[2].text, "Add leeway.");
    }

    #[test]
    fn labels_and_timing() {
        let mut r = Reasoning::new(ReasoningStatus::Done, "");
        r.start(ClockMs(0));
        assert_eq!(r.label(ClockMs(3000)), "Thinking");
        r.push("one\ntwo\nthree");
        assert_eq!(preview(&r.text, 2), vec!["two", "three"]);
        assert!(r.shows_preview());
        r.finish(ClockMs(72_000));
        assert_eq!(r.label(ClockMs(90_000)), "Thought for 1m 12s");
        assert_eq!(r.announce(ClockMs(0)), "Thought for 1 minute 12 seconds");
        assert!(!r.shows_preview());
        r.retry(ClockMs(100_000));
        r.fail(ClockMs(101_000), "Rate limited");
        assert_eq!(r.label(ClockMs(0)), "Thinking interrupted");
        let mut v = Reasoning::new(ReasoningStatus::Thinking, "x");
        v.level = DisplayLevel::Verbose;
        v.start(ClockMs(0));
        v.stop(ClockMs(7000));
        assert!(v.open);
        assert_eq!(v.label(ClockMs(0)), "Stopped after 7s");
    }

    #[test]
    fn trace_playback() {
        let t = Trace::new(
            TraceKind::Search,
            vec![
                TraceRow::new("a", None, RowStatus::Pending),
                TraceRow::new("b", None, RowStatus::Pending),
                TraceRow::new("c", None, RowStatus::Pending),
            ],
        );
        let rows = t.playback(Duration::from_millis(1500), Duration::from_millis(1000));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].status, RowStatus::Active);
        assert_eq!(t.summary(&rows), "Searching · 1 sources so far");
        let all = t.playback(Duration::from_secs(10), Duration::from_secs(1));
        assert_eq!(all.len(), 3);
    }
}
