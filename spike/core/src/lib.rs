//! Spike of `oxyn-core`: AgentState, the event reducer and derived `Stalled`.
//! No UI dependencies and no clock of its own: time is passed in as milliseconds.

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind { Network, RateLimited, Tool, Model, Other }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallLevel { Soft, Hard }

#[derive(Clone, Debug, PartialEq)]
pub enum AgentState {
    Idle,
    Composing,
    Submitted,
    Thinking { verb: &'static str },
    Streaming,
    ToolRunning { tool: String },
    AwaitingApproval { request: u64 },
    Queued { count: u32 },
    Steering,
    Done { elapsed_ms: u64 },
    Error { kind: ErrorKind },
    Cancelled,
    Stalled { since_ms: u64, level: StallLevel, was: Box<AgentState> },
}

impl AgentState {
    /// Short label used by status indicators and screen readers.
    pub fn label(&self) -> String {
        match self {
            Self::Idle => "Ready".into(),
            Self::Composing => "Typing".into(),
            Self::Submitted => "Sending".into(),
            Self::Thinking { verb } => format!("{verb}…"),
            Self::Streaming => "Writing".into(),
            Self::ToolRunning { tool } => format!("Running {tool}"),
            Self::AwaitingApproval { .. } => "Needs your approval".into(),
            Self::Queued { count } => format!("{count} queued"),
            Self::Steering => "Steering".into(),
            Self::Done { elapsed_ms } => format!("Done in {:.1} s", *elapsed_ms as f64 / 1000.0),
            Self::Error { .. } => "Something went wrong".into(),
            Self::Cancelled => "Stopped".into(),
            Self::Stalled { level: StallLevel::Soft, .. } => "Still working…".into(),
            Self::Stalled { level: StallLevel::Hard, since_ms, .. } => {
                format!("No response for {} s", since_ms / 1000)
            }
        }
    }

    /// Semantic tone; maps to `color.state.*` tokens.
    pub fn tone(&self) -> Tone {
        match self {
            Self::Idle | Self::Composing | Self::Cancelled => Tone::Neutral,
            Self::Submitted | Self::Thinking { .. } | Self::Streaming | Self::ToolRunning { .. }
            | Self::Queued { .. } | Self::Steering => Tone::Active,
            Self::AwaitingApproval { .. } => Tone::Attention,
            Self::Done { .. } => Tone::Success,
            Self::Error { .. } => Tone::Danger,
            Self::Stalled { .. } => Tone::Warning,
        }
    }

    pub fn is_busy(&self) -> bool { matches!(self.tone(), Tone::Active) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone { Neutral, Active, Attention, Success, Warning, Danger }

/// Events an adapter (AG-UI, Claude SDK, OpenAI, custom) feeds in.
#[derive(Clone, Debug)]
pub enum Event {
    Typing,
    Submit,
    ReasoningStart { verb: &'static str },
    TextDelta,
    ToolStart { tool: String },
    ToolEnd,
    ApprovalRequested { id: u64 },
    ApprovalResolved,
    Queue { count: u32 },
    Steer,
    Finish,
    Fail(ErrorKind),
    Cancel,
}

/// Stall thresholds (proposed in architecture.md: 5 s soft, 45 s hard, 120 s during a tool).
#[derive(Clone, Copy, Debug)]
pub struct StallPolicy { pub soft_ms: u64, pub hard_ms: u64, pub tool_ms: u64 }

impl Default for StallPolicy {
    fn default() -> Self { Self { soft_ms: 5_000, hard_ms: 45_000, tool_ms: 120_000 } }
}

/// Holds the reported state plus timestamps; `state_at(now)` derives `Stalled`.
#[derive(Clone, Debug)]
pub struct Run {
    reported: AgentState,
    started_ms: u64,
    last_event_ms: u64,
    pub policy: StallPolicy,
}

impl Default for Run {
    fn default() -> Self {
        Self { reported: AgentState::Idle, started_ms: 0, last_event_ms: 0, policy: StallPolicy::default() }
    }
}

impl Run {
    pub fn apply(&mut self, ev: Event, now_ms: u64) {
        use AgentState as S;
        self.last_event_ms = now_ms;
        self.reported = match (ev, &self.reported) {
            (Event::Typing, S::Idle | S::Done { .. } | S::Cancelled | S::Error { .. }) => S::Composing,
            (Event::Typing, s) => s.clone(),
            (Event::Submit, _) => { self.started_ms = now_ms; S::Submitted }
            (Event::ReasoningStart { verb }, _) => S::Thinking { verb },
            (Event::TextDelta, _) => S::Streaming,
            (Event::ToolStart { tool }, _) => S::ToolRunning { tool },
            (Event::ToolEnd, _) => S::Thinking { verb: "Thinking" },
            (Event::ApprovalRequested { id }, _) => S::AwaitingApproval { request: id },
            (Event::ApprovalResolved, _) => S::Thinking { verb: "Thinking" },
            (Event::Queue { count }, _) => S::Queued { count },
            (Event::Steer, _) => S::Steering,
            (Event::Finish, _) => S::Done { elapsed_ms: now_ms.saturating_sub(self.started_ms) },
            (Event::Fail(kind), _) => S::Error { kind },
            (Event::Cancel, _) => S::Cancelled,
        };
    }

    /// The state a component should draw at `now_ms`. Only busy states can stall;
    /// waiting on the user (approval) never counts as a stall.
    pub fn state_at(&self, now_ms: u64) -> AgentState {
        if !self.reported.is_busy() { return self.reported.clone(); }
        let quiet = now_ms.saturating_sub(self.last_event_ms);
        let hard = match self.reported {
            AgentState::ToolRunning { .. } => self.policy.tool_ms,
            _ => self.policy.hard_ms,
        };
        let level = if quiet >= hard { StallLevel::Hard }
            else if quiet >= self.policy.soft_ms { StallLevel::Soft }
            else { return self.reported.clone() };
        AgentState::Stalled { since_ms: quiet, level, was: Box::new(self.reported.clone()) }
    }
}

/// Scripted demo run used by both renderers and the poster: (time ms, event).
pub fn demo_script() -> Vec<(u64, Event)> {
    vec![
        (0, Event::Typing),
        (900, Event::Submit),
        (1_300, Event::ReasoningStart { verb: "Thinking" }),
        (3_000, Event::ToolStart { tool: "cargo test".into() }),
        (5_200, Event::ToolEnd),
        (5_600, Event::ApprovalRequested { id: 1 }),
        (7_600, Event::ApprovalResolved),
        (8_000, Event::TextDelta),
        (10_500, Event::Finish),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_ends_done_with_elapsed() {
        let mut r = Run::default();
        for (t, e) in demo_script() { r.apply(e, t); }
        assert_eq!(r.state_at(10_500), AgentState::Done { elapsed_ms: 9_600 });
    }

    #[test]
    fn stall_soft_then_hard() {
        let mut r = Run::default();
        r.apply(Event::Submit, 0);
        r.apply(Event::ReasoningStart { verb: "Thinking" }, 100);
        assert!(matches!(r.state_at(5_000), AgentState::Thinking { .. }));
        assert!(matches!(r.state_at(5_100), AgentState::Stalled { level: StallLevel::Soft, .. }));
        assert!(matches!(r.state_at(45_100), AgentState::Stalled { level: StallLevel::Hard, .. }));
        r.apply(Event::TextDelta, 46_000);
        assert_eq!(r.state_at(46_001), AgentState::Streaming);
    }

    #[test]
    fn tools_get_longer_threshold() {
        let mut r = Run::default();
        r.apply(Event::ToolStart { tool: "build".into() }, 0);
        assert!(matches!(r.state_at(60_000), AgentState::Stalled { level: StallLevel::Soft, .. }));
        assert!(matches!(r.state_at(120_000), AgentState::Stalled { level: StallLevel::Hard, .. }));
    }

    #[test]
    fn approval_and_done_never_stall() {
        let mut r = Run::default();
        r.apply(Event::ApprovalRequested { id: 7 }, 0);
        assert_eq!(r.state_at(600_000), AgentState::AwaitingApproval { request: 7 });
        r.apply(Event::Finish, 1);
        assert!(matches!(r.state_at(600_000), AgentState::Done { .. }));
    }

    #[test]
    fn every_state_has_a_label() {
        let states = [
            AgentState::Idle, AgentState::Composing, AgentState::Submitted,
            AgentState::Thinking { verb: "Pondering" }, AgentState::Streaming,
            AgentState::ToolRunning { tool: "ls".into() }, AgentState::AwaitingApproval { request: 1 },
            AgentState::Queued { count: 2 }, AgentState::Steering, AgentState::Done { elapsed_ms: 1200 },
            AgentState::Error { kind: ErrorKind::Network }, AgentState::Cancelled,
            AgentState::Stalled { since_ms: 45_000, level: StallLevel::Hard, was: Box::new(AgentState::Streaming) },
        ];
        assert_eq!(states.len(), 13);
        for s in states { assert!(!s.label().is_empty()); }
    }
}
