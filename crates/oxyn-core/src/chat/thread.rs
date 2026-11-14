//! Conversation models (`messages.md`, `message-bubble.md`): response feedback, chat-thread
//! grouping with day separators and receipts, starter prompts, the research run and the support
//! conversation.
//!
//! Behaviour follows Arc UI Response feedback, Chat thread, Starter prompts, Research run and
//! Support conversation, and affectUI Agent Thread (rebuilt from the docs; no source used).

use std::time::Duration;

use crate::ClockMs;

// ─────────────────────────────────── response feedback ──

/// Thumbs rating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thumb {
    /// Good response.
    Up,
    /// Bad response.
    Down,
}

/// Feedback panel phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeedbackPhase {
    /// No panel.
    #[default]
    Closed,
    /// Reasons (and note after a thumbs down).
    Form,
    /// Sending.
    Sending,
    /// Thank-you line (folds away after `thanks`).
    Thanks,
    /// Send failed (Submit becomes Try again).
    Error,
}

impl FeedbackPhase {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Form => "form",
            Self::Sending => "sending",
            Self::Thanks => "thanks",
            Self::Error => "error",
        }
    }
}

/// Default reasons after a thumbs down.
pub const NEGATIVE: [&str; 5] = ["Inaccurate", "Not helpful", "Too long", "Unsafe", "Other"];
/// Default reasons after a thumbs up.
pub const POSITIVE: [&str; 4] = [
    "Accurate",
    "Easy to follow",
    "Good examples",
    "Saved me time",
];

/// Response feedback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feedback {
    /// Rating.
    pub rating: Option<Thumb>,
    /// Panel phase.
    pub phase: FeedbackPhase,
    /// Chosen reasons.
    pub reasons: Vec<String>,
    /// Note.
    pub comment: String,
    /// Note limit.
    pub max_len: usize,
}

impl Default for Feedback {
    fn default() -> Self {
        Self {
            rating: None,
            phase: FeedbackPhase::Closed,
            reasons: vec![],
            comment: String::new(),
            max_len: 1000,
        }
    }
}

impl Feedback {
    /// A thumb pressed. The active thumb clears the rating; a new one opens its panel.
    pub fn press(&mut self, t: Thumb, has_positive: bool) {
        if self.rating == Some(t) {
            self.rating = None;
            self.phase = FeedbackPhase::Closed;
        } else {
            self.rating = Some(t);
            self.reasons.clear();
            self.phase = if t == Thumb::Down || has_positive {
                FeedbackPhase::Form
            } else {
                FeedbackPhase::Closed
            };
        }
    }
    /// Toggles a reason chip.
    pub fn toggle_reason(&mut self, r: &str) {
        if let Some(i) = self.reasons.iter().position(|x| x == r) {
            self.reasons.remove(i);
        } else {
            self.reasons.push(r.into());
        }
    }
    /// Types in the note (clipped to the limit).
    pub fn set_comment(&mut self, s: &str) {
        self.comment = s.chars().take(self.max_len).collect();
    }
    /// Submit (or Try again).
    pub fn submit(&mut self) -> bool {
        if matches!(self.phase, FeedbackPhase::Form | FeedbackPhase::Error) {
            self.phase = FeedbackPhase::Sending;
            true
        } else {
            false
        }
    }
    /// The send settled.
    pub fn sent(&mut self, ok: bool) {
        self.phase = if ok {
            FeedbackPhase::Thanks
        } else {
            FeedbackPhase::Error
        };
    }
    /// Skip / Escape / the thank-you timeout: closes, keeping the rating.
    pub fn close(&mut self) {
        self.phase = FeedbackPhase::Closed;
    }
    /// Live text.
    pub fn announce(&self) -> String {
        match (self.phase, self.rating) {
            (FeedbackPhase::Sending, _) => "Sending feedback".into(),
            (FeedbackPhase::Thanks, _) => "Thanks for the feedback".into(),
            (FeedbackPhase::Error, _) => "Couldn't send feedback".into(),
            (_, Some(Thumb::Up)) => "Rated good response".into(),
            (_, Some(Thumb::Down)) => "Rated bad response".into(),
            (_, None) => String::new(),
        }
    }
}

// ─────────────────────────────────────── chat thread ──

/// Delivery status of your own message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Delivery {
    /// Sending.
    Sending,
    /// Sent.
    #[default]
    Sent,
    /// Delivered.
    Delivered,
    /// Read.
    Read,
    /// Failed.
    Failed,
}

impl Delivery {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Sending => "Sending…",
            Self::Sent => "Sent",
            Self::Delivered => "Delivered",
            Self::Read => "Read",
            Self::Failed => "Not delivered",
        }
    }
}

/// A chat message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    /// Id.
    pub id: String,
    /// Author id.
    pub author: String,
    /// Text.
    pub text: String,
    /// Send time (ms since the epoch the host uses).
    pub at: u64,
    /// Delivery (own messages).
    pub status: Delivery,
    /// (emoji, count, mine).
    pub reactions: Vec<(String, u32, bool)>,
    /// Attachment names.
    pub files: Vec<String>,
}

impl ChatMessage {
    /// A message.
    pub fn new(id: &str, author: &str, text: &str, at: u64) -> Self {
        Self {
            id: id.into(),
            author: author.into(),
            text: text.into(),
            at,
            status: Delivery::Sent,
            reactions: vec![],
            files: vec![],
        }
    }
    /// With a reaction.
    pub fn react(mut self, e: &str, n: u32, mine: bool) -> Self {
        self.reactions.push((e.into(), n, mine));
        self
    }
    /// With a file.
    pub fn file(mut self, f: &str) -> Self {
        self.files.push(f.into());
        self
    }
    /// With a status.
    pub fn status(mut self, s: Delivery) -> Self {
        self.status = s;
        self
    }
}

/// One rendered item of a thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadItem {
    /// Day separator label.
    Day(String),
    /// Consecutive messages of one author within the window.
    Group {
        /// Author id.
        author: String,
        /// Message indices.
        messages: Vec<usize>,
    },
}

const DAY: u64 = 86_400_000;

/// Day label relative to `now` ("Today", "Yesterday", "3 days ago").
pub fn day_label(at: u64, now: u64) -> String {
    let d = now / DAY - at / DAY;
    match d {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        n => format!("{n} days ago"),
    }
}

/// "14:05" from ms (UTC clock face).
pub fn clock(at: u64) -> String {
    let m = (at / 60_000) % (24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// Groups messages (oldest first) by author within `window` ms, with day separators.
pub fn group(messages: &[ChatMessage], window: u64, now: u64) -> Vec<ThreadItem> {
    let mut out: Vec<ThreadItem> = Vec::new();
    let mut last_day: Option<u64> = None;
    let mut prev: Option<&ChatMessage> = None;
    for (i, m) in messages.iter().enumerate() {
        let day = m.at / DAY;
        if last_day != Some(day) {
            out.push(ThreadItem::Day(day_label(m.at, now)));
            last_day = Some(day);
            prev = None;
        }
        let joins = prev.is_some_and(|p| p.author == m.author && m.at - p.at <= window);
        if joins && let Some(ThreadItem::Group { messages, .. }) = out.last_mut() {
            messages.push(i);
        } else {
            out.push(ThreadItem::Group {
                author: m.author.clone(),
                messages: vec![i],
            });
        }
        prev = Some(m);
    }
    out
}

/// "Sam is typing", "Sam and Ana are typing", "3 people are typing".
pub fn typing_label(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => format!("{} is typing", names[0]),
        2 => format!("{} and {} are typing", names[0], names[1]),
        n => format!("{n} people are typing"),
    }
}

/// Sticks to bottom: whether new messages should scroll the log (within 80 px of the end).
pub fn pinned(scroll_top: f32, client: f32, scroll_height: f32) -> bool {
    scroll_height - (scroll_top + client) < 80.0
}

// ─────────────────────────────────── starter prompts ──

/// A starter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Starter {
    /// Id.
    pub id: String,
    /// Category id.
    pub category: String,
    /// Title.
    pub title: String,
    /// Full prompt placed in the composer.
    pub prompt: String,
    /// Connected app.
    pub app: Option<String>,
}

impl Starter {
    /// A starter.
    pub fn new(id: &str, category: &str, title: &str, prompt: &str) -> Self {
        Self {
            id: id.into(),
            category: category.into(),
            title: title.into(),
            prompt: prompt.into(),
            app: None,
        }
    }
    /// With an app.
    pub fn app(mut self, a: &str) -> Self {
        self.app = Some(a.into());
        self
    }
}

/// Starters shown for a category (`all` = every one) at a shuffle `seed`, `count` at a time.
pub fn starters_for(all: &[Starter], category: &str, seed: u32, count: usize) -> Vec<Starter> {
    let pool: Vec<&Starter> = all
        .iter()
        .filter(|s| category == "all" || s.category == category)
        .collect();
    if pool.is_empty() {
        return vec![];
    }
    let n = pool.len();
    let start = (seed as usize * count) % n;
    (0..count.min(n))
        .map(|k| pool[(start + k) % n].clone())
        .collect()
}

/// Categories that have starters, in the given order, with `all` first.
pub fn categories(all: &[Starter], order: &[&str]) -> Vec<String> {
    let mut v = vec!["all".to_string()];
    for c in order {
        if all.iter().any(|s| s.category == *c) {
            v.push((*c).to_string());
        }
    }
    v
}

// ──────────────────────────────────────── research run ──

/// A research phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResearchPhase {
    /// Planning queries.
    #[default]
    Planning,
    /// Searching.
    Searching,
    /// Reading sources.
    Reading,
    /// Writing the report.
    Writing,
    /// Done.
    Done,
}

impl ResearchPhase {
    /// All in order.
    pub const ALL: [Self; 5] = [
        Self::Planning,
        Self::Searching,
        Self::Reading,
        Self::Writing,
        Self::Done,
    ];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Planning => "Planning",
            Self::Searching => "Searching",
            Self::Reading => "Reading",
            Self::Writing => "Writing",
            Self::Done => "Done",
        }
    }
}

/// A research run on a scripted timeline: phase, queries run, sources read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchRun {
    /// Question.
    pub question: String,
    /// Queries.
    pub queries: Vec<String>,
    /// Sources (title, domain).
    pub sources: Vec<(String, String)>,
    /// Started at.
    pub started: ClockMs,
    /// Paused.
    pub paused: bool,
}

/// Time per query / source in the demo timeline.
pub const RESEARCH_STEP: Duration = Duration::from_millis(900);

impl ResearchRun {
    /// A run.
    pub fn new(
        question: &str,
        queries: &[&str],
        sources: &[(&str, &str)],
        started: ClockMs,
    ) -> Self {
        Self {
            question: question.into(),
            queries: queries.iter().map(|s| s.to_string()).collect(),
            sources: sources
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            started,
            paused: false,
        }
    }
    /// (phase, queries done, sources read, overall 0–1) at `now`.
    pub fn at(&self, now: ClockMs) -> (ResearchPhase, usize, usize, f32) {
        let step = RESEARCH_STEP.as_millis() as u64;
        let t = now.0.saturating_sub(self.started.0);
        let plan = step;
        let q = self.queries.len() as u64 * step;
        let r = self.sources.len() as u64 * step;
        let w = 3 * step;
        let total = plan + q + r + w;
        let o = (t as f32 / total as f32).min(1.0);
        if t < plan {
            (ResearchPhase::Planning, 0, 0, o)
        } else if t < plan + q {
            (ResearchPhase::Searching, ((t - plan) / step) as usize, 0, o)
        } else if t < plan + q + r {
            (
                ResearchPhase::Reading,
                self.queries.len(),
                ((t - plan - q) / step) as usize,
                o,
            )
        } else if t < total {
            (
                ResearchPhase::Writing,
                self.queries.len(),
                self.sources.len(),
                o,
            )
        } else {
            (
                ResearchPhase::Done,
                self.queries.len(),
                self.sources.len(),
                1.0,
            )
        }
    }
}

// ──────────────────────────────────── support conversation ──

/// Support phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SupportPhase {
    /// The assistant answers.
    #[default]
    Bot,
    /// Waiting for a person (queue position).
    Handoff(u32),
    /// A person joined.
    Human,
    /// Resolved (rating asked).
    Resolved,
}

impl SupportPhase {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Bot => "bot",
            Self::Handoff(_) => "handoff",
            Self::Human => "human",
            Self::Resolved => "resolved",
        }
    }
    /// Banner text.
    pub fn banner(self) -> String {
        match self {
            Self::Bot => "AI assistant · answers instantly".into(),
            Self::Handoff(0) => "Connecting you to a person…".into(),
            Self::Handoff(n) => format!("You're #{n} in line · about {} min", n * 2),
            Self::Human => "Maya from Support joined".into(),
            Self::Resolved => "Conversation resolved".into(),
        }
    }
    /// Next phase in the scripted flow.
    pub fn next(self) -> Self {
        match self {
            Self::Bot => Self::Handoff(2),
            Self::Handoff(n) if n > 0 => Self::Handoff(n - 1),
            Self::Handoff(_) => Self::Human,
            Self::Human => Self::Resolved,
            Self::Resolved => Self::Resolved,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_flow() {
        let mut f = Feedback::default();
        f.press(Thumb::Down, true);
        assert_eq!(f.phase, FeedbackPhase::Form);
        f.toggle_reason("Too long");
        f.set_comment(&"x".repeat(2000));
        assert_eq!(f.comment.len(), 1000);
        assert!(f.submit());
        f.sent(false);
        assert_eq!(f.announce(), "Couldn't send feedback");
        assert!(f.submit());
        f.sent(true);
        assert_eq!(f.phase, FeedbackPhase::Thanks);
        f.close();
        assert_eq!(f.rating, Some(Thumb::Down));
        f.press(Thumb::Down, true);
        assert_eq!(f.rating, None);
        f.press(Thumb::Up, false);
        assert_eq!(f.phase, FeedbackPhase::Closed);
    }

    #[test]
    fn thread_grouping() {
        let now = 10 * DAY + 50_000_000;
        let ms = vec![
            ChatMessage::new("1", "sam", "hi", 9 * DAY + 100),
            ChatMessage::new("2", "sam", "there", 9 * DAY + 60_000),
            ChatMessage::new("3", "me", "yo", 9 * DAY + 120_000),
            ChatMessage::new("4", "me", "later", 9 * DAY + 900_000),
            ChatMessage::new("5", "sam", "today", 10 * DAY + 1000),
        ];
        let g = group(&ms, 300_000, now);
        assert_eq!(g[0], ThreadItem::Day("Yesterday".into()));
        assert!(matches!(&g[1], ThreadItem::Group { messages, .. } if messages == &vec![0, 1]));
        assert_eq!(g.len(), 6);
        assert_eq!(
            typing_label(&["Sam".into(), "Ana".into()]),
            "Sam and Ana are typing"
        );
        assert_eq!(clock(14 * 3_600_000 + 5 * 60_000), "14:05");
        assert!(pinned(900.0, 300.0, 1250.0));
    }

    #[test]
    fn starters_and_research() {
        let all = vec![
            Starter::new("a", "write", "A", "a"),
            Starter::new("b", "code", "B", "b"),
            Starter::new("c", "code", "C", "c"),
        ];
        assert_eq!(starters_for(&all, "code", 0, 4).len(), 2);
        assert_ne!(
            starters_for(&all, "all", 0, 2),
            starters_for(&all, "all", 1, 2)
        );
        assert_eq!(
            categories(&all, &["write", "plan", "code"]),
            vec!["all", "write", "code"]
        );
        let r = ResearchRun::new("q", &["a", "b"], &[("s", "d")], ClockMs(0));
        assert_eq!(r.at(ClockMs(100)).0, ResearchPhase::Planning);
        assert_eq!(r.at(ClockMs(1000)).1, 0);
        assert_eq!(r.at(ClockMs(2000)).0, ResearchPhase::Searching);
        assert_eq!(r.at(ClockMs(2800)).0, ResearchPhase::Reading);
        assert_eq!(r.at(ClockMs(99_999)).0, ResearchPhase::Done);
        let mut p = SupportPhase::Bot;
        p = p.next();
        assert_eq!(p.banner(), "You're #2 in line · about 4 min");
        p = p.next().next().next();
        assert_eq!(p, SupportPhase::Human);
    }
}
