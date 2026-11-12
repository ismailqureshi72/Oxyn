//! Plans and todo lists (`plan-todo.md`): item states, progress, the todo dock, the plan card and
//! the step timeline.

use std::time::Duration;

use super::format_call_duration;
use crate::{AgentState, ClockMs};

/// An item's state (§2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemStatus {
    /// Not started.
    Pending,
    /// In progress.
    Active,
    /// Finished.
    Done,
    /// Not needed.
    Skipped {
        /// Why.
        reason: Option<String>,
    },
    /// Failed; the run may continue.
    Failed {
        /// Why.
        reason: Option<String>,
    },
    /// Waiting on another item or the user.
    Blocked {
        /// "#3" or "you".
        on: String,
    },
    /// Removed or stopped.
    Cancelled,
}

impl ItemStatus {
    /// Glyph (terminal and fallback).
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Active => "◐",
            Self::Done => "✓",
            Self::Skipped { .. } => "⤼",
            Self::Failed { .. } => "✕",
            Self::Blocked { .. } => "⧗",
            Self::Cancelled => "–",
        }
    }

    /// Short word for accessible names.
    pub fn word(&self) -> &'static str {
        match self {
            Self::Pending => "to do",
            Self::Active => "in progress",
            Self::Done => "done",
            Self::Skipped { .. } => "skipped",
            Self::Failed { .. } => "failed",
            Self::Blocked { .. } => "blocked",
            Self::Cancelled => "cancelled",
        }
    }
}

/// How an item changed in the last plan update (highlight overlay).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemChange {
    /// New.
    Added,
    /// Text changed.
    Reworded {
        /// Old text.
        from: String,
    },
    /// Moved.
    Moved,
    /// Reopened after done or failed.
    Reopened,
}

/// One step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanItem {
    /// Stable id.
    pub id: String,
    /// Label ("Run the tests").
    pub text: String,
    /// Present form while active ("Running the tests").
    pub present: Option<String>,
    /// Past form when done ("Ran the tests").
    pub past: Option<String>,
    /// State.
    pub status: ItemStatus,
    /// Sub-items (do not count toward the parent's total).
    pub children: Vec<PlanItem>,
    /// Tool calls attributed to this step (step timeline).
    pub calls: Vec<String>,
    /// Started / ended.
    pub span: Option<(ClockMs, Option<ClockMs>)>,
    /// Sub-agent colour index (owner), if any.
    pub owner: Option<u8>,
    /// Last change, with when.
    pub change: Option<(ItemChange, ClockMs)>,
}

impl PlanItem {
    /// A pending item.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            present: None,
            past: None,
            status: ItemStatus::Pending,
            children: vec![],
            calls: vec![],
            span: None,
            owner: None,
            change: None,
        }
    }

    /// Builder: status.
    pub fn with(mut self, s: ItemStatus) -> Self {
        self.status = s;
        self
    }

    /// Builder: verb forms.
    pub fn verbs(mut self, present: &str, past: &str) -> Self {
        self.present = Some(present.into());
        self.past = Some(past.into());
        self
    }

    /// Builder: span.
    pub fn span(mut self, start: u64, end: Option<u64>) -> Self {
        self.span = Some((ClockMs(start), end.map(ClockMs)));
        self
    }

    /// The label in the tense of its status.
    pub fn label(&self) -> &str {
        match self.status {
            ItemStatus::Active => self.present.as_deref().unwrap_or(&self.text),
            ItemStatus::Done => self.past.as_deref().unwrap_or(&self.text),
            _ => &self.text,
        }
    }

    /// Duration text (`0.9s`, `1m 05s`).
    pub fn duration(&self, now: ClockMs) -> Option<String> {
        let (s, e) = self.span?;
        let d = e.unwrap_or(now).since(s);
        (e.is_some() || d >= Duration::from_secs(1)).then(|| format_call_duration(d))
    }
}

/// Plan card lifecycle (§3.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanState {
    /// Skeleton.
    Drafting,
    /// Steps arriving.
    Streaming,
    /// Ready; editable.
    Proposed,
    /// Waiting for sign-off.
    AwaitingApproval,
    /// Being rewritten.
    Revising,
    /// Just approved (≤ 600 ms).
    Approved,
    /// Running; the card folds to a pill.
    Executing,
    /// Replaced by a newer version.
    Superseded {
        /// The newer version.
        by: u16,
    },
    /// Rejected, with the user's note.
    Rejected {
        /// Note.
        note: Option<String>,
    },
    /// The run stopped.
    Interrupted,
    /// Every counted item done.
    Done,
}

/// A plan or todo list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Title.
    pub title: Option<String>,
    /// Version (1-based).
    pub version: u16,
    /// Items.
    pub items: Vec<PlanItem>,
    /// Card state.
    pub state: PlanState,
}

/// Progress (§2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// Done items.
    pub done: u32,
    /// Counted items (all − skipped − cancelled).
    pub total: u32,
    /// Skipped.
    pub skipped: u32,
    /// Failed.
    pub failed: u32,
    /// Active.
    pub active: u32,
}

impl Progress {
    /// `3/7 · 1 skipped · 1 failed`; `Done · 7/7`; empty for a single item.
    pub fn label(&self) -> String {
        if self.total <= 1 && self.skipped == 0 {
            return String::new();
        }
        let mut s = if self.done == self.total && self.total > 0 {
            format!("Done · {}/{}", self.done, self.total)
        } else {
            format!("{}/{}", self.done, self.total)
        };
        if self.skipped > 0 {
            s.push_str(&format!(" · {} skipped", self.skipped));
        }
        if self.failed > 0 {
            s.push_str(&format!(" · {} failed", self.failed));
        }
        s
    }

    /// Fraction for bars and rings.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.done as f32 / self.total as f32
        }
    }
}

impl Plan {
    /// A plan with these items.
    pub fn new(title: Option<&str>, items: Vec<PlanItem>) -> Self {
        Self {
            title: title.map(str::to_string),
            version: 1,
            items,
            state: PlanState::Proposed,
        }
    }

    /// Progress over top-level items.
    pub fn progress(&self) -> Progress {
        let mut p = Progress {
            done: 0,
            total: 0,
            skipped: 0,
            failed: 0,
            active: 0,
        };
        for i in &self.items {
            match i.status {
                ItemStatus::Skipped { .. } => p.skipped += 1,
                ItemStatus::Cancelled => {}
                _ => p.total += 1,
            }
            match i.status {
                ItemStatus::Done => p.done += 1,
                ItemStatus::Failed { .. } => p.failed += 1,
                ItemStatus::Active => p.active += 1,
                _ => {}
            }
        }
        p
    }

    /// The first active item.
    pub fn active(&self) -> Option<&PlanItem> {
        self.items.iter().find(|i| i.status == ItemStatus::Active)
    }

    /// Items with a highlight still showing (changed in the last 2 s).
    pub fn changed(&self, now: ClockMs) -> impl Iterator<Item = &PlanItem> {
        self.items.iter().filter(move |i| {
            i.change
                .as_ref()
                .is_some_and(|(_, at)| now.since(*at) < CHANGE_HIGHLIGHT)
        })
    }
}

/// How long a changed item stays highlighted.
pub const CHANGE_HIGHLIGHT: Duration = Duration::from_millis(2000);
/// The dock's "Done" state fades after this.
pub const DOCK_DONE_FADE: Duration = Duration::from_secs(4);

/// The todo dock above the composer (§3.3).
#[derive(Clone, Debug, PartialEq)]
pub enum TodoDock {
    /// No list.
    Hidden,
    /// "Tasks 3/7 · Running tests".
    Live {
        /// Header.
        label: String,
        /// Current item, shimmering.
        current: Option<String>,
    },
    /// "Done · 7/7", fading.
    Completed {
        /// Header.
        label: String,
        /// Fade-out progress `0..=1`.
        fade: f32,
    },
    /// "Stopped · 4 left".
    Stopped {
        /// Header.
        label: String,
    },
}

impl TodoDock {
    /// The dock for `plan` given the agent state; `done_at` is when the last item finished.
    pub fn derive(
        plan: Option<&Plan>,
        agent: &AgentState,
        done_at: Option<ClockMs>,
        now: ClockMs,
    ) -> Self {
        let Some(plan) = plan else {
            return Self::Hidden;
        };
        if plan.items.is_empty() {
            return Self::Hidden;
        }
        let p = plan.progress();
        if p.total > 0 && p.done == p.total {
            let t = done_at.map(|d| now.since(d)).unwrap_or_default();
            if t >= DOCK_DONE_FADE + Duration::from_millis(300) {
                return Self::Hidden;
            }
            let fade = (t.saturating_sub(DOCK_DONE_FADE).as_secs_f32() / 0.3).clamp(0.0, 1.0);
            return Self::Completed {
                label: format!("Done · {}/{}", p.done, p.total),
                fade,
            };
        }
        if matches!(
            agent,
            AgentState::Cancelled { .. } | AgentState::Error { .. }
        ) || (matches!(agent, AgentState::Done { .. }) && p.done < p.total)
        {
            return Self::Stopped {
                label: format!("Stopped · {} left", p.total - p.done),
            };
        }
        Self::Live {
            label: format!("Tasks {}/{}", p.done, p.total),
            current: plan.active().map(|i| i.label().to_string()),
        }
    }
}

/// The slice of a long list the collapsed checklist shows: 2 done above the active item and 4
/// pending below; the rest fold into "n more".
pub fn item_window(items: &[PlanItem], done_above: usize, pending_below: usize) -> (usize, usize) {
    let focus = items
        .iter()
        .position(|i| i.status == ItemStatus::Active)
        .or_else(|| items.iter().position(|i| i.status == ItemStatus::Pending))
        .unwrap_or(items.len().saturating_sub(1));
    let start = focus.saturating_sub(done_above);
    let end = (focus + 1 + pending_below).min(items.len());
    (start, end)
}

/// Plan-card footer action for a state.
pub fn plan_cta(state: &PlanState, gated: bool) -> Option<&'static str> {
    match state {
        PlanState::Proposed if gated => Some("Approve"),
        PlanState::Proposed => Some("Build"),
        PlanState::AwaitingApproval => Some("Approve"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> Plan {
        Plan::new(
            Some("Fix refresh-token race"),
            vec![
                PlanItem::new("1", "Read the auth middleware").with(ItemStatus::Done),
                PlanItem::new("2", "Write a failing test").with(ItemStatus::Done),
                PlanItem::new("3", "Run the tests")
                    .verbs("Running the tests", "Ran the tests")
                    .with(ItemStatus::Active),
                PlanItem::new("4", "Patch the lock").with(ItemStatus::Pending),
                PlanItem::new("5", "Update docs").with(ItemStatus::Skipped { reason: None }),
            ],
        )
    }

    #[test]
    fn progress_and_dock() {
        let p = plan();
        assert_eq!(p.progress().label(), "2/4 · 1 skipped");
        let d = TodoDock::derive(Some(&p), &AgentState::Streaming, None, ClockMs(0));
        assert_eq!(
            d,
            TodoDock::Live {
                label: "Tasks 2/4".into(),
                current: Some("Running the tests".into())
            }
        );
        let d = TodoDock::derive(
            Some(&p),
            &AgentState::Cancelled {
                after: Duration::from_secs(4),
            },
            None,
            ClockMs(0),
        );
        assert_eq!(
            d,
            TodoDock::Stopped {
                label: "Stopped · 2 left".into()
            }
        );
        let mut done = p.clone();
        for i in &mut done.items {
            if i.status != (ItemStatus::Skipped { reason: None }) {
                i.status = ItemStatus::Done;
            }
        }
        assert_eq!(done.progress().label(), "Done · 4/4 · 1 skipped");
        assert!(matches!(
            TodoDock::derive(
                Some(&done),
                &AgentState::Idle,
                Some(ClockMs(0)),
                ClockMs(1_000)
            ),
            TodoDock::Completed { .. }
        ));
        assert_eq!(
            TodoDock::derive(
                Some(&done),
                &AgentState::Idle,
                Some(ClockMs(0)),
                ClockMs(9_000)
            ),
            TodoDock::Hidden
        );
    }

    #[test]
    fn window_and_labels() {
        let items: Vec<PlanItem> = (0..12)
            .map(|i| {
                PlanItem::new(i.to_string(), format!("Step {i}")).with(match i {
                    0..6 => ItemStatus::Done,
                    6 => ItemStatus::Active,
                    _ => ItemStatus::Pending,
                })
            })
            .collect();
        assert_eq!(item_window(&items, 2, 4), (4, 11));
        assert_eq!(plan().items[2].label(), "Running the tests");
        assert_eq!(plan_cta(&PlanState::Proposed, false), Some("Build"));
    }
}
