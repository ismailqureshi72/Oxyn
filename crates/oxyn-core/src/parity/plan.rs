//! Plans, steps and progress (plan #28, chunk 2).
//!
//! - [`PlanDraft`]: an agent's plan to review before it runs: rename, reorder, remove, add, run,
//!   live timers, retry from a failed step.
//! - [`Trail`]: the steps an agent walks as one quiet trail with gates and sub-steps.
//! - [`TaskRow`]: Beautiful UI style task rows (capsules or list) with ring progress.
//! - [`Stepper`]: where a person is in a multi-step flow.
//! - [`Checklist`]: onboarding checklist with a ring, hide/show and a done state.
//! - [`Breakdown`]: a goal broken into tasks with estimates, owners, dependencies and a
//!   schedule that says whether the deadline holds.

use crate::ClockMs;

/// Status of a step anywhere in this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    /// Not reached.
    Pending,
    /// Working on it.
    Running,
    /// Waiting on a person ("Needs you").
    Waiting,
    /// Finished.
    Done,
    /// Failed.
    Failed,
    /// A person declined it.
    Declined,
    /// Skipped.
    Skipped,
}

impl StepState {
    /// Kebab id (`data-state`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Declined => "declined",
            Self::Skipped => "skipped",
        }
    }
    /// From a kebab id.
    pub fn parse(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "waiting" => Self::Waiting,
            "done" => Self::Done,
            "failed" => Self::Failed,
            "declined" => Self::Declined,
            "skipped" => Self::Skipped,
            _ => Self::Pending,
        }
    }
    /// Finished one way or another.
    pub fn settled(self) -> bool {
        matches!(self, Self::Done | Self::Skipped | Self::Declined)
    }
}

/// Short duration: `12s`, `1m 4s`, `1h 2m`, `3d 4h`.
pub fn short_duration(ms: u64) -> String {
    let s = ms / 1000;
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {}s", s / 60, s % 60)
    } else if s < 86_400 {
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}d {}h", s / 86_400, (s % 86_400) / 3600)
    }
}

/// One step of an agent plan.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// One line of detail.
    pub detail: Option<String>,
    /// Tool it uses.
    pub tool: Option<String>,
    /// State.
    pub state: StepState,
    /// Started at (running) for a live timer.
    pub started: Option<ClockMs>,
    /// How long it took (done).
    pub took_ms: Option<u64>,
    /// What went wrong.
    pub error: Option<String>,
}

impl PlanStep {
    /// A pending step.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: None,
            tool: None,
            state: StepState::Pending,
            started: None,
            took_ms: None,
            error: None,
        }
    }
    /// With a tool.
    pub fn tool(mut self, t: &str) -> Self {
        self.tool = Some(t.into());
        self
    }
    /// With detail.
    pub fn detail(mut self, d: &str) -> Self {
        self.detail = Some(d.into());
        self
    }
    /// Timer text at `now`.
    pub fn timer(&self, now: ClockMs) -> Option<String> {
        match (self.state, self.started, self.took_ms) {
            (StepState::Running, Some(s), _) => Some(short_duration(now.0.saturating_sub(s.0))),
            (_, _, Some(t)) => Some(short_duration(t)),
            _ => None,
        }
    }
}

/// Phase of a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanPhase {
    /// Review and edit.
    Draft,
    /// Executing.
    Running,
    /// Finished.
    Done,
}

/// An agent's plan to review, then run.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanDraft {
    /// Goal (heading).
    pub title: String,
    /// Estimate ("~2 min").
    pub estimate: Option<String>,
    /// Steps.
    pub steps: Vec<PlanStep>,
    /// Phase.
    pub phase: PlanPhase,
    next_id: usize,
}

impl PlanDraft {
    /// A draft plan.
    pub fn new(title: &str, steps: Vec<PlanStep>) -> Self {
        let next_id = steps.len() + 1;
        Self {
            title: title.into(),
            estimate: None,
            steps,
            phase: PlanPhase::Draft,
            next_id,
        }
    }
    /// Editing is only possible in draft.
    pub fn editable(&self) -> bool {
        self.phase == PlanPhase::Draft
    }
    /// Renames step `i`.
    pub fn rename(&mut self, i: usize, title: &str) {
        if self.editable()
            && let Some(s) = self.steps.get_mut(i)
            && !title.trim().is_empty()
        {
            s.title = title.trim().into();
        }
    }
    /// Moves step `from` to `to`.
    pub fn reorder(&mut self, from: usize, to: usize) {
        if self.editable() && from < self.steps.len() && to < self.steps.len() {
            let s = self.steps.remove(from);
            self.steps.insert(to, s);
        }
    }
    /// Removes step `i` (keeps at least one).
    pub fn remove(&mut self, i: usize) {
        if self.editable() && self.steps.len() > 1 && i < self.steps.len() {
            self.steps.remove(i);
        }
    }
    /// Adds a step at the end; returns its index.
    pub fn add(&mut self, title: &str) -> usize {
        if self.editable() {
            let id = format!("s{}", self.next_id);
            self.next_id += 1;
            self.steps.push(PlanStep::new(&id, title));
        }
        self.steps.len() - 1
    }
    /// Run plan: the first step starts.
    pub fn run(&mut self, now: ClockMs) {
        self.phase = PlanPhase::Running;
        for s in self.steps.iter_mut() {
            *s = PlanStep {
                state: StepState::Pending,
                started: None,
                took_ms: None,
                error: None,
                ..s.clone()
            };
        }
        if let Some(s) = self.steps.first_mut() {
            s.state = StepState::Running;
            s.started = Some(now);
        }
    }
    /// The running step finished (`Err` fails it and stops the run).
    pub fn finish_step(&mut self, result: Result<(), String>, now: ClockMs) {
        let Some(i) = self.steps.iter().position(|s| s.state == StepState::Running) else {
            return;
        };
        let s = &mut self.steps[i];
        s.took_ms = s.started.map(|t| now.0.saturating_sub(t.0));
        match result {
            Ok(()) => {
                s.state = StepState::Done;
                if let Some(n) = self.steps.get_mut(i + 1) {
                    n.state = StepState::Running;
                    n.started = Some(now);
                } else {
                    self.phase = PlanPhase::Done;
                }
            }
            Err(e) => {
                s.state = StepState::Failed;
                s.error = Some(e);
            }
        }
    }
    /// Retry from a failed step.
    pub fn retry_from(&mut self, id: &str, now: ClockMs) {
        if let Some(i) = self.steps.iter().position(|s| s.id == id) {
            for s in self.steps[i..].iter_mut() {
                s.state = StepState::Pending;
                s.error = None;
                s.took_ms = None;
                s.started = None;
            }
            self.steps[i].state = StepState::Running;
            self.steps[i].started = Some(now);
            self.phase = PlanPhase::Running;
        }
    }
    /// `done/total`.
    pub fn progress(&self) -> (usize, usize) {
        (
            self.steps.iter().filter(|s| s.state.settled()).count(),
            self.steps.len(),
        )
    }
}

/// One step of an agent trail.
#[derive(Clone, Debug, PartialEq)]
pub struct TrailStep {
    /// Title.
    pub title: String,
    /// State.
    pub state: StepState,
    /// What a running step does right now ("Testing").
    pub status: Option<String>,
    /// Started at.
    pub started: Option<ClockMs>,
    /// Took.
    pub took_ms: Option<u64>,
    /// Needs approval before it runs.
    pub gate: bool,
    /// Tool name (mark before the title).
    pub tool: Option<String>,
    /// Sub-steps.
    pub sub: Vec<(String, StepState)>,
}

impl TrailStep {
    /// A step.
    pub fn new(title: &str, state: StepState) -> Self {
        Self {
            title: title.into(),
            state,
            status: None,
            started: None,
            took_ms: None,
            gate: false,
            tool: None,
            sub: Vec::new(),
        }
    }
    /// Badge text: "Approval", "Needs you", "Declined", "Failed".
    pub fn badge(&self) -> Option<&'static str> {
        match self.state {
            StepState::Waiting => Some("Needs you"),
            StepState::Failed => Some("Failed"),
            StepState::Declined => Some("Declined"),
            StepState::Pending if self.gate => Some("Approval"),
            _ => None,
        }
    }
}

/// An agent trail.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Trail {
    /// Task (header).
    pub title: String,
    /// Agent name.
    pub agent: Option<String>,
    /// Steps.
    pub steps: Vec<TrailStep>,
}

impl Trail {
    /// "3/5".
    pub fn count(&self) -> String {
        let d = self.steps.iter().filter(|s| s.state.settled()).count();
        format!("{d}/{}", self.steps.len())
    }
}

/// Task row status (Beautiful UI Task Rows).
#[derive(Clone, Debug, PartialEq)]
pub struct TaskRow {
    /// Label.
    pub label: String,
    /// Amount / meta on the right.
    pub amount: Option<String>,
    /// State.
    pub state: StepState,
    /// Current step / total, for the ring.
    pub step: (usize, usize),
    /// Detail lines `(label, meta)`.
    pub details: Vec<(String, String)>,
    /// Open.
    pub open: bool,
}

impl TaskRow {
    /// Ring progress 0..1.
    pub fn ring(&self) -> f32 {
        match self.state {
            StepState::Done => 1.0,
            _ if self.step.1 == 0 => 0.0,
            _ => self.step.0 as f32 / self.step.1 as f32,
        }
    }
    /// Pill text.
    pub fn pill(&self) -> Option<String> {
        match self.state {
            StepState::Done => Some("Completed".into()),
            StepState::Failed => Some("Failed".into()),
            StepState::Running => Some(format!("Step {} of {}", self.step.0 + 1, self.step.1)),
            StepState::Waiting => Some("Needs you".into()),
            _ => None,
        }
    }
}

/// One step of a [`Stepper`].
#[derive(Clone, Debug, PartialEq)]
pub struct FlowStep {
    /// Title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
    /// Error text (shown always).
    pub error: Option<String>,
    /// Optional step.
    pub optional: bool,
}

/// A stepper: steps before `current` are complete.
#[derive(Clone, Debug, PartialEq)]
pub struct Stepper {
    /// Steps.
    pub steps: Vec<FlowStep>,
    /// Current index (== len when all complete).
    pub current: usize,
}

impl Stepper {
    /// State of step `i`.
    pub fn state(&self, i: usize) -> StepState {
        if self.steps.get(i).and_then(|s| s.error.as_ref()).is_some() {
            StepState::Failed
        } else if i < self.current {
            StepState::Done
        } else if i == self.current {
            StepState::Running
        } else {
            StepState::Pending
        }
    }
    /// Whether every step is complete.
    pub fn complete(&self) -> bool {
        self.current >= self.steps.len()
    }
    /// Connector fill 0..1 between step `i` and `i + 1`.
    pub fn connector(&self, i: usize) -> f32 {
        if i < self.current { 1.0 } else { 0.0 }
    }
    /// Arrow-key focus move among reachable (completed + current) steps.
    pub fn move_focus(&self, from: usize, delta: i32) -> usize {
        let max = self.current.min(self.steps.len().saturating_sub(1)) as i32;
        (from as i32 + delta).clamp(0, max) as usize
    }
    /// Caption for compact mode: "Step 2 of 4 · Billing".
    pub fn caption(&self, complete_label: &str) -> String {
        if self.complete() {
            complete_label.into()
        } else {
            format!(
                "Step {} of {} · {}",
                self.current + 1,
                self.steps.len(),
                self.steps[self.current].title
            )
        }
    }
}

/// One onboarding item.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckItem {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// What it is for.
    pub body: String,
    /// Action label ("Invite").
    pub action: Option<String>,
    /// Completed.
    pub done: bool,
}

/// Onboarding checklist.
#[derive(Clone, Debug, PartialEq)]
pub struct Checklist {
    /// Items.
    pub items: Vec<CheckItem>,
    /// Expanded item.
    pub open: Option<String>,
    /// Body hidden.
    pub hidden: bool,
}

impl Checklist {
    /// `(done, total)`.
    pub fn count(&self) -> (usize, usize) {
        (
            self.items.iter().filter(|i| i.done).count(),
            self.items.len(),
        )
    }
    /// Ring 0..1.
    pub fn ring(&self) -> f32 {
        let (d, t) = self.count();
        if t == 0 { 1.0 } else { d as f32 / t as f32 }
    }
    /// Toggles an item; opens the next open one.
    pub fn toggle(&mut self, id: &str) {
        if let Some(i) = self.items.iter_mut().find(|i| i.id == id) {
            i.done = !i.done;
        }
        self.open = self.items.iter().find(|i| !i.done).map(|i| i.id.clone());
    }
    /// All done.
    pub fn all_done(&self) -> bool {
        self.items.iter().all(|i| i.done)
    }
}

/// A task in a breakdown.
#[derive(Clone, Debug, PartialEq)]
pub struct BTask {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Working days (steps of 0.5).
    pub days: f32,
    /// Owner initials.
    pub owner: String,
    /// Ids this depends on.
    pub deps: Vec<String>,
}

/// A goal broken into tasks.
#[derive(Clone, Debug, PartialEq)]
pub struct Breakdown {
    /// Goal sentence.
    pub goal: String,
    /// Working days to the deadline.
    pub deadline_days: f32,
    /// Tasks.
    pub tasks: Vec<BTask>,
}

impl Breakdown {
    /// Earliest start of each task (days), respecting dependencies and one owner doing one task
    /// at a time.
    pub fn schedule(&self) -> Vec<(f32, f32)> {
        let mut out: Vec<(f32, f32)> = Vec::with_capacity(self.tasks.len());
        let mut owner_free: Vec<(String, f32)> = Vec::new();
        for t in &self.tasks {
            let dep_end = t
                .deps
                .iter()
                .filter_map(|d| self.tasks.iter().position(|x| &x.id == d))
                .filter(|&i| i < out.len())
                .map(|i| out[i].1)
                .fold(0.0, f32::max);
            let free = owner_free
                .iter()
                .find(|(o, _)| *o == t.owner)
                .map(|x| x.1)
                .unwrap_or(0.0);
            let start = dep_end.max(free);
            let end = start + t.days;
            match owner_free.iter_mut().find(|(o, _)| *o == t.owner) {
                Some(x) => x.1 = end,
                None => owner_free.push((t.owner.clone(), end)),
            }
            out.push((start, end));
        }
        out
    }
    /// Total span in days.
    pub fn span(&self) -> f32 {
        self.schedule().iter().map(|x| x.1).fold(0.0, f32::max)
    }
    /// Deadline holds.
    pub fn on_track(&self) -> bool {
        self.span() <= self.deadline_days
    }
    /// Verdict line.
    pub fn verdict(&self) -> String {
        let s = self.span();
        if self.on_track() {
            format!(
                "Fits: {s} of {} working days, {} to spare",
                self.deadline_days,
                self.deadline_days - s
            )
        } else {
            format!(
                "Misses the deadline by {} working days",
                s - self.deadline_days
            )
        }
    }
    /// Moves a task up or down.
    pub fn shift(&mut self, i: usize, up: bool) {
        let j = if up { i.checked_sub(1) } else { Some(i + 1) };
        if let Some(j) = j
            && j < self.tasks.len()
        {
            self.tasks.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_edit_run_retry() {
        let mut p = PlanDraft::new(
            "Fix login",
            vec![PlanStep::new("a", "Read"), PlanStep::new("b", "Patch")],
        );
        p.rename(0, "  Read auth.rs ");
        p.add("Test");
        p.reorder(2, 1);
        assert_eq!(p.steps[1].title, "Test");
        p.remove(1);
        p.run(ClockMs(0));
        assert!(!p.editable());
        p.finish_step(Ok(()), ClockMs(3000));
        assert_eq!(p.steps[0].timer(ClockMs(9000)).unwrap(), "3s");
        p.finish_step(Err("tests failed".into()), ClockMs(5000));
        assert_eq!(p.steps[1].state, StepState::Failed);
        p.retry_from("b", ClockMs(6000));
        p.finish_step(Ok(()), ClockMs(7000));
        assert_eq!(p.phase, PlanPhase::Done);
        assert_eq!(p.progress(), (2, 2));
    }

    #[test]
    fn stepper_and_checklist() {
        let st = Stepper {
            steps: (0..4)
                .map(|i| FlowStep {
                    title: format!("S{i}"),
                    description: None,
                    error: None,
                    optional: false,
                })
                .collect(),
            current: 2,
        };
        assert_eq!(st.state(1), StepState::Done);
        assert_eq!(st.move_focus(2, 1), 2);
        assert_eq!(st.caption("All done"), "Step 3 of 4 · S2");
        let mut c = Checklist {
            items: vec![
                CheckItem {
                    id: "a".into(),
                    title: "A".into(),
                    body: String::new(),
                    action: None,
                    done: false,
                },
                CheckItem {
                    id: "b".into(),
                    title: "B".into(),
                    body: String::new(),
                    action: None,
                    done: false,
                },
            ],
            open: None,
            hidden: false,
        };
        c.toggle("a");
        assert_eq!(c.open.as_deref(), Some("b"));
        assert!((c.ring() - 0.5).abs() < 1e-6);
        assert_eq!(short_duration(64_000), "1m 4s");
    }

    #[test]
    fn breakdown_schedule() {
        let t = |id: &str, d: f32, o: &str, deps: &[&str]| BTask {
            id: id.into(),
            title: id.into(),
            days: d,
            owner: o.into(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
        };
        let b = Breakdown {
            goal: "g".into(),
            deadline_days: 5.0,
            tasks: vec![
                t("a", 2.0, "x", &[]),
                t("b", 1.0, "y", &[]),
                t("c", 2.0, "y", &["a"]),
            ],
        };
        assert_eq!(b.schedule()[2], (2.0, 4.0));
        assert!(b.on_track());
    }
}
