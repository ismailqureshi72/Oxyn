//! The agent state model (`docs/architecture.md`): one [`AgentState`] drives every component, and
//! [`AgentMachine`] folds adapter events into it.
//!
//! Rules:
//! - Components receive `&AgentState` plus their own props. They keep no state of their own about
//!   the run.
//! - `Stalled` is **derived** here from event timestamps ([`AgentMachine::tick`]); apps and adapters
//!   never set it.
//! - The run timer pauses while an approval is open, so `Done { elapsed }` is working time and
//!   `waited` is the time spent waiting for the user (`docs/specs/elapsed-timer.md`).
//! - The machine is lenient: adapters are messy, so any work event while idle starts a run rather
//!   than being rejected.

use std::time::Duration;

use crate::{ClockMs, verbs::ActivityKind};

/// A tool call the agent is running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRef {
    /// Adapter-assigned call id, unique within a run.
    pub id: String,
    /// Tool name as the protocol reports it (`bash`, `read_file`, `web_search`).
    pub name: String,
    /// What kind of work it is; picks the pinned verb (`verbs::ActivityKind`).
    pub activity: ActivityKind,
    /// Short target shown after the verb: a file basename, a domain, a command head.
    pub target: Option<String>,
}

/// An open request for the user (approval, question, plan sign-off).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestRef {
    /// Adapter-assigned request id.
    pub id: String,
    /// The tool call it gates, when there is one.
    pub tool: Option<String>,
}

/// Why a run stopped with an error. Always shown in words, never colour alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// A tool failed and the agent gave up.
    Tool,
    /// The model or provider returned an error.
    Model,
    /// The run timed out.
    Timeout,
    /// The agent could not be reached.
    Offline,
    /// Rate limit or quota.
    RateLimited,
    /// Anything else, with a short label.
    Other(String),
}

/// How long a busy run has been silent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StallLevel {
    /// Past `soft`: tone ramps toward warning, timers keep going.
    Soft,
    /// Past `hard`: "No response for 45 s", retry offered, cyclers freeze.
    Hard,
}

/// The one state every component reads.
#[derive(Clone, Debug, PartialEq)]
pub enum AgentState {
    /// Nothing happening.
    Idle,
    /// The user is typing (a draft is present).
    Composing,
    /// Sent; waiting for the first event. Shown as a pulse, not a spinner.
    Submitted,
    /// Reasoning or planning. `header` is a model-written summary line when the protocol has one;
    /// otherwise the verb comes from the verb pack at render time.
    Thinking {
        /// Model-written header ("Checking the auth flow"), if any.
        header: Option<String>,
    },
    /// Answer tokens arriving.
    Streaming,
    /// A tool is running. With several, this is the most recently started.
    ToolRunning {
        /// The call shown.
        tool: ToolRef,
        /// How many calls are running at once ("Running 3 tools").
        concurrent: u32,
    },
    /// Waiting for the user. The run timer is paused.
    AwaitingApproval {
        /// The open request.
        request: RequestRef,
    },
    /// The agent is busy and the user has queued messages.
    Queued {
        /// Number of queued messages.
        count: u32,
    },
    /// A user interrupt is being applied mid-run.
    Steering,
    /// The run finished.
    Done {
        /// Working time, excluding time spent waiting for the user.
        elapsed: Duration,
        /// Time spent waiting for the user.
        waited: Duration,
    },
    /// The run stopped with an error.
    Error {
        /// Why.
        kind: ErrorKind,
    },
    /// The user stopped the run.
    Cancelled {
        /// Working time before the stop ("Interrupted after 42s").
        after: Duration,
    },
    /// No event for a while. Derived; never set by apps.
    Stalled {
        /// When the last event arrived (silence is measured from here).
        since: ClockMs,
        /// Soft or hard.
        level: StallLevel,
        /// The busy state the run was in; restored when an event arrives.
        was: Box<AgentState>,
    },
}

impl AgentState {
    /// The agent is working on a run (the states that can stall).
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            AgentState::Submitted
                | AgentState::Thinking { .. }
                | AgentState::Streaming
                | AgentState::ToolRunning { .. }
                | AgentState::Queued { .. }
                | AgentState::Steering
                | AgentState::Stalled { .. }
        )
    }

    /// The run has ended (`Done`, `Error`, `Cancelled`).
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            AgentState::Done { .. } | AgentState::Error { .. } | AgentState::Cancelled { .. }
        )
    }
}

/// Stall thresholds (`docs/specs/composer-status-strip.md` and `tools.md`, proposed defaults).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StallPolicy {
    /// Silence before `Soft` when no tool is running (5 s).
    pub soft: Duration,
    /// Silence before `Hard` when no tool is running (45 s).
    pub hard: Duration,
    /// Silence before `Soft` while a tool runs (120 s; tools are often quiet).
    pub tool_soft: Duration,
    /// Silence before `Hard` while a tool runs (300 s).
    pub tool_hard: Duration,
}

impl Default for StallPolicy {
    fn default() -> Self {
        Self {
            soft: Duration::from_secs(5),
            hard: Duration::from_secs(45),
            tool_soft: Duration::from_secs(120),
            tool_hard: Duration::from_secs(300),
        }
    }
}

/// What adapters feed the machine. Each adapter maps its protocol (AI SDK parts, AG-UI, ACP, MCP,
/// provider streams) onto these.
#[derive(Clone, Debug, PartialEq)]
pub enum AgentEvent {
    /// The draft changed. `empty` is true when the user cleared it.
    Draft {
        /// Draft is now empty.
        empty: bool,
    },
    /// The user sent a message.
    Submit,
    /// Reasoning started or its header changed.
    Thinking {
        /// Model-written header, if any.
        header: Option<String>,
    },
    /// Answer text arrived.
    TextDelta,
    /// A tool call started.
    ToolStarted(ToolRef),
    /// A tool call produced output (keeps the run alive, changes nothing else).
    ToolOutput {
        /// Call id.
        id: String,
    },
    /// A tool call ended.
    ToolFinished {
        /// Call id.
        id: String,
    },
    /// The agent needs the user.
    ApprovalRequested(RequestRef),
    /// The user answered a request.
    ApprovalResolved {
        /// Request id.
        id: String,
    },
    /// The number of queued user messages changed.
    QueueChanged {
        /// New count.
        count: u32,
    },
    /// The user steered mid-run.
    Steer,
    /// Any other sign of life from the agent (usage update, keep-alive).
    Heartbeat,
    /// The run completed.
    Finished,
    /// The run failed.
    Failed(ErrorKind),
    /// The user stopped the run.
    Cancel,
    /// Back to idle (the host dismissed the summary, or a new chat).
    Reset,
}

/// Working time for one run, paused while waiting for the user.
#[derive(Clone, Copy, Debug)]
struct RunTimer {
    start: ClockMs,
    paused_total: Duration,
    paused_since: Option<ClockMs>,
}

impl RunTimer {
    fn new(now: ClockMs) -> Self {
        Self {
            start: now,
            paused_total: Duration::ZERO,
            paused_since: None,
        }
    }
    fn pause(&mut self, now: ClockMs) {
        self.paused_since.get_or_insert(now);
    }
    fn resume(&mut self, now: ClockMs) {
        if let Some(p) = self.paused_since.take() {
            self.paused_total += now.since(p);
        }
    }
    fn waited(&self, now: ClockMs) -> Duration {
        self.paused_total + self.paused_since.map_or(Duration::ZERO, |p| now.since(p))
    }
    fn elapsed(&self, now: ClockMs) -> Duration {
        now.since(self.start).saturating_sub(self.waited(now))
    }
}

/// Folds [`AgentEvent`]s into an [`AgentState`] and derives `Stalled` on [`tick`](Self::tick).
#[derive(Clone, Debug)]
pub struct AgentMachine {
    state: AgentState,
    policy: StallPolicy,
    run: Option<RunTimer>,
    last_event: ClockMs,
    tools: Vec<ToolRef>,
    requests: Vec<RequestRef>,
    queued: u32,
}

impl Default for AgentMachine {
    fn default() -> Self {
        Self::new(StallPolicy::default())
    }
}

impl AgentMachine {
    /// A machine in `Idle` with the given stall thresholds.
    pub fn new(policy: StallPolicy) -> Self {
        Self {
            state: AgentState::Idle,
            policy,
            run: None,
            last_event: ClockMs(0),
            tools: Vec::new(),
            requests: Vec::new(),
            queued: 0,
        }
    }

    /// The current state.
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Queued user messages (kept across work events; `Queued` itself is only shown until the
    /// next work event).
    pub fn queued(&self) -> u32 {
        self.queued
    }

    /// Tool calls running now, oldest first.
    pub fn running_tools(&self) -> &[ToolRef] {
        &self.tools
    }

    /// Working time so far in the current run (frozen once it ends), or zero.
    pub fn elapsed(&self, now: ClockMs) -> Duration {
        match &self.state {
            AgentState::Done { elapsed, .. } => *elapsed,
            AgentState::Cancelled { after } => *after,
            _ => self.run.map_or(Duration::ZERO, |r| r.elapsed(now)),
        }
    }

    fn start_run(&mut self, now: ClockMs) {
        if self.run.is_none()
            || self.state.is_finished()
            || !self.state.is_busy() && !self.waiting()
        {
            self.run = Some(RunTimer::new(now));
            self.tools.clear();
            self.requests.clear();
        }
    }

    fn waiting(&self) -> bool {
        matches!(self.state, AgentState::AwaitingApproval { .. })
    }

    /// The state the run is in when nothing more specific applies: approval beats tools, tools
    /// beat thinking.
    fn working_state(&self) -> AgentState {
        if let Some(request) = self.requests.last() {
            return AgentState::AwaitingApproval {
                request: request.clone(),
            };
        }
        if let Some(tool) = self.tools.last() {
            return AgentState::ToolRunning {
                tool: tool.clone(),
                concurrent: self.tools.len() as u32,
            };
        }
        AgentState::Thinking { header: None }
    }

    /// Applies one event at `now` and returns the new state.
    pub fn apply(&mut self, event: AgentEvent, now: ClockMs) -> &AgentState {
        self.last_event = now;
        if let AgentState::Stalled { was, .. } = &self.state {
            self.state = (**was).clone();
        }
        let next = match event {
            AgentEvent::Draft { empty } => {
                if self.state.is_busy() || self.waiting() {
                    None // typing during a run does not change the run's state
                } else if empty {
                    Some(AgentState::Idle)
                } else {
                    Some(AgentState::Composing)
                }
            }
            AgentEvent::Submit => {
                if self.state.is_busy() || self.waiting() {
                    // A send during a run queues (the composer shows the queue stack).
                    self.queued += 1;
                    Some(AgentState::Queued { count: self.queued })
                } else {
                    self.run = Some(RunTimer::new(now));
                    self.tools.clear();
                    self.requests.clear();
                    Some(AgentState::Submitted)
                }
            }
            AgentEvent::Thinking { header } => {
                self.start_run(now);
                if self.waiting() || !self.tools.is_empty() {
                    None
                } else {
                    Some(AgentState::Thinking { header })
                }
            }
            AgentEvent::TextDelta => {
                self.start_run(now);
                if self.waiting() || !self.tools.is_empty() {
                    None
                } else {
                    Some(AgentState::Streaming)
                }
            }
            AgentEvent::ToolStarted(tool) => {
                self.start_run(now);
                self.tools.retain(|t| t.id != tool.id);
                self.tools.push(tool);
                Some(self.working_state())
            }
            AgentEvent::ToolOutput { .. } | AgentEvent::Heartbeat => None,
            AgentEvent::ToolFinished { id } => {
                self.tools.retain(|t| t.id != id);
                self.state.is_busy().then(|| self.working_state())
            }
            AgentEvent::ApprovalRequested(request) => {
                self.start_run(now);
                if let Some(run) = &mut self.run {
                    run.pause(now);
                }
                self.requests.retain(|r| r.id != request.id);
                self.requests.push(request);
                Some(self.working_state())
            }
            AgentEvent::ApprovalResolved { id } => {
                self.requests.retain(|r| r.id != id);
                if self.requests.is_empty()
                    && let Some(run) = &mut self.run
                {
                    run.resume(now);
                }
                self.waiting().then(|| self.working_state())
            }
            AgentEvent::QueueChanged { count } => {
                self.queued = count;
                (self.state.is_busy() && count > 0).then_some(AgentState::Queued { count })
            }
            AgentEvent::Steer => self.state.is_busy().then_some(AgentState::Steering),
            AgentEvent::Finished => {
                self.finish(now, |elapsed, waited| AgentState::Done { elapsed, waited })
            }
            AgentEvent::Failed(kind) => self.finish(now, |_, _| AgentState::Error { kind }),
            AgentEvent::Cancel => self.finish(now, |after, _| AgentState::Cancelled { after }),
            AgentEvent::Reset => {
                self.run = None;
                self.tools.clear();
                self.requests.clear();
                self.queued = 0;
                Some(AgentState::Idle)
            }
        };
        if let Some(s) = next {
            self.state = s;
        }
        &self.state
    }

    fn finish(
        &mut self,
        now: ClockMs,
        make: impl FnOnce(Duration, Duration) -> AgentState,
    ) -> Option<AgentState> {
        let run = self.run?;
        if self.state.is_finished() {
            return None;
        }
        let (elapsed, waited) = (run.elapsed(now), run.waited(now));
        self.tools.clear();
        self.requests.clear();
        Some(make(elapsed, waited))
    }

    /// Re-derives `Stalled` from the time since the last event. Call it from the clock (1 Hz is
    /// enough). Returns the state.
    pub fn tick(&mut self, now: ClockMs) -> &AgentState {
        let base = match &self.state {
            AgentState::Stalled { was, .. } => (**was).clone(),
            s if s.is_busy() => s.clone(),
            _ => return &self.state,
        };
        let (soft, hard) = if matches!(base, AgentState::ToolRunning { .. }) {
            (self.policy.tool_soft, self.policy.tool_hard)
        } else {
            (self.policy.soft, self.policy.hard)
        };
        let silent = now.since(self.last_event);
        self.state = if silent >= hard {
            AgentState::Stalled {
                since: self.last_event,
                level: StallLevel::Hard,
                was: Box::new(base),
            }
        } else if silent >= soft {
            AgentState::Stalled {
                since: self.last_event,
                level: StallLevel::Soft,
                was: Box::new(base),
            }
        } else {
            base
        };
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(id: &str, activity: ActivityKind) -> ToolRef {
        ToolRef {
            id: id.into(),
            name: "bash".into(),
            activity,
            target: Some("cargo test".into()),
        }
    }

    fn ms(x: u64) -> ClockMs {
        ClockMs(x)
    }

    #[test]
    fn happy_path() {
        let mut m = AgentMachine::default();
        assert_eq!(
            m.apply(AgentEvent::Draft { empty: false }, ms(0)),
            &AgentState::Composing
        );
        assert_eq!(m.apply(AgentEvent::Submit, ms(100)), &AgentState::Submitted);
        m.apply(AgentEvent::Thinking { header: None }, ms(400));
        assert!(matches!(m.state(), AgentState::Thinking { .. }));
        m.apply(
            AgentEvent::ToolStarted(tool("t1", ActivityKind::Run)),
            ms(1_000),
        );
        assert!(matches!(
            m.state(),
            AgentState::ToolRunning { concurrent: 1, .. }
        ));
        m.apply(AgentEvent::ToolFinished { id: "t1".into() }, ms(3_000));
        assert!(matches!(m.state(), AgentState::Thinking { .. }));
        m.apply(AgentEvent::TextDelta, ms(3_200));
        assert_eq!(m.state(), &AgentState::Streaming);
        m.apply(AgentEvent::Finished, ms(12_100));
        assert_eq!(
            m.state(),
            &AgentState::Done {
                elapsed: Duration::from_secs(12),
                waited: Duration::ZERO
            }
        );
        assert_eq!(m.apply(AgentEvent::Reset, ms(16_000)), &AgentState::Idle);
    }

    #[test]
    fn concurrent_tools_show_latest_and_count() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(
            AgentEvent::ToolStarted(tool("a", ActivityKind::Read)),
            ms(10),
        );
        m.apply(
            AgentEvent::ToolStarted(tool("b", ActivityKind::Search)),
            ms(20),
        );
        match m.state() {
            AgentState::ToolRunning { tool, concurrent } => {
                assert_eq!(tool.id, "b");
                assert_eq!(*concurrent, 2);
            }
            s => panic!("{s:?}"),
        }
        m.apply(AgentEvent::ToolFinished { id: "b".into() }, ms(30));
        assert!(
            matches!(m.state(), AgentState::ToolRunning { tool, concurrent: 1 } if tool.id == "a")
        );
    }

    #[test]
    fn approval_pauses_the_timer_and_wins_over_tools() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(
            AgentEvent::ToolStarted(tool("t", ActivityKind::Edit)),
            ms(1_000),
        );
        let req = RequestRef {
            id: "r1".into(),
            tool: Some("t".into()),
        };
        m.apply(AgentEvent::ApprovalRequested(req), ms(2_000));
        assert!(matches!(m.state(), AgentState::AwaitingApproval { .. }));
        // Tool events while waiting do not hide the approval.
        m.apply(
            AgentEvent::ToolStarted(tool("u", ActivityKind::Read)),
            ms(3_000),
        );
        assert!(matches!(m.state(), AgentState::AwaitingApproval { .. }));
        // Waiting never stalls.
        assert!(matches!(
            m.tick(ms(600_000)),
            AgentState::AwaitingApproval { .. }
        ));
        m.apply(AgentEvent::ApprovalResolved { id: "r1".into() }, ms(62_000));
        assert!(matches!(m.state(), AgentState::ToolRunning { .. }));
        m.apply(AgentEvent::Finished, ms(65_000));
        assert_eq!(
            m.state(),
            &AgentState::Done {
                elapsed: Duration::from_secs(5),
                waited: Duration::from_secs(60)
            }
        );
    }

    #[test]
    fn stalls_soft_then_hard_and_recovers() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(AgentEvent::TextDelta, ms(1_000));
        assert_eq!(m.tick(ms(5_999)), &AgentState::Streaming);
        assert!(
            matches!(m.tick(ms(6_000)), AgentState::Stalled { level: StallLevel::Soft, since, .. } if *since == ms(1_000))
        );
        assert!(matches!(
            m.tick(ms(46_000)),
            AgentState::Stalled {
                level: StallLevel::Hard,
                ..
            }
        ));
        // An event restores the busy state it stalled from, then applies.
        m.apply(AgentEvent::Heartbeat, ms(47_000));
        assert_eq!(m.state(), &AgentState::Streaming);
    }

    #[test]
    fn tools_get_longer_stall_thresholds() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(
            AgentEvent::ToolStarted(tool("t", ActivityKind::Test)),
            ms(0),
        );
        assert!(matches!(m.tick(ms(60_000)), AgentState::ToolRunning { .. }));
        assert!(matches!(
            m.tick(ms(120_000)),
            AgentState::Stalled {
                level: StallLevel::Soft,
                ..
            }
        ));
        m.apply(AgentEvent::ToolOutput { id: "t".into() }, ms(121_000));
        assert!(matches!(m.state(), AgentState::ToolRunning { .. }));
    }

    #[test]
    fn idle_and_finished_never_stall() {
        let mut m = AgentMachine::default();
        assert_eq!(m.tick(ms(1_000_000)), &AgentState::Idle);
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(AgentEvent::Finished, ms(10));
        assert!(matches!(m.tick(ms(1_000_000)), AgentState::Done { .. }));
    }

    #[test]
    fn cancel_and_error_record_time() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(AgentEvent::Cancel, ms(42_500));
        assert_eq!(
            m.state(),
            &AgentState::Cancelled {
                after: Duration::from_millis(42_500)
            }
        );
        // A second terminal event is ignored.
        m.apply(AgentEvent::Failed(ErrorKind::Offline), ms(43_000));
        assert!(matches!(m.state(), AgentState::Cancelled { .. }));

        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(AgentEvent::Failed(ErrorKind::Timeout), ms(1_000));
        assert_eq!(
            m.state(),
            &AgentState::Error {
                kind: ErrorKind::Timeout
            }
        );
    }

    #[test]
    fn sending_during_a_run_queues_and_steer_is_shown() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ms(0));
        m.apply(AgentEvent::TextDelta, ms(10));
        m.apply(AgentEvent::Submit, ms(20));
        assert_eq!(m.state(), &AgentState::Queued { count: 1 });
        m.apply(AgentEvent::TextDelta, ms(30));
        assert_eq!(m.state(), &AgentState::Streaming);
        assert_eq!(m.queued(), 1);
        m.apply(AgentEvent::Steer, ms(40));
        assert_eq!(m.state(), &AgentState::Steering);
        // Typing during a run changes nothing.
        m.apply(AgentEvent::Draft { empty: false }, ms(50));
        assert_eq!(m.state(), &AgentState::Steering);
    }

    #[test]
    fn work_without_submit_starts_a_run() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::TextDelta, ms(1_000));
        assert_eq!(m.state(), &AgentState::Streaming);
        m.apply(AgentEvent::Finished, ms(3_000));
        assert_eq!(
            m.state(),
            &AgentState::Done {
                elapsed: Duration::from_secs(2),
                waited: Duration::ZERO
            }
        );
        // A new run after Done restarts the timer.
        m.apply(AgentEvent::TextDelta, ms(10_000));
        m.apply(AgentEvent::Finished, ms(11_000));
        assert!(
            matches!(m.state(), AgentState::Done { elapsed, .. } if *elapsed == Duration::from_secs(1))
        );
    }
}
