//! Alert and card (`alert.md`, `card.md`).
//!
//! - **Alert** (Arc UI Alert): a persistent in-flow message with a tone icon, a title and details.
//!   Danger is `role=alert`; the rest are `role=status`. Hiding collapses its height so content
//!   below closes the gap; tone changes morph the icon, copy changes rise in. Oxyn plus: an
//!   agent-error preset with Retry and Show logs, collapsible details, and an `agent` tone.
//! - **Card** (Arc UI Card): a content card with media, byline, status and an action that can grow
//!   into a quick look. Status changes roll in word by word. Oxyn plus: a status edge (a beam while
//!   an agent works on it), a selectable card, and a compact list-row variant.
//!
//! Rebuilt from the public docs; no source used.

use crate::AgentState;

/// Alert tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertTone {
    /// Information.
    #[default]
    Info,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Danger (interrupts).
    Danger,
    /// Oxyn plus: an agent speaking (agent identity colour).
    Agent,
}

impl AlertTone {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "warning" => Self::Warning,
            "danger" => Self::Danger,
            "agent" => Self::Agent,
            _ => Self::Info,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Agent => "agent",
        }
    }
    /// ARIA role: danger interrupts, the rest are polite.
    pub fn role(self) -> &'static str {
        if self == Self::Danger {
            "alert"
        } else {
            "status"
        }
    }
    /// Icon name (Oxyn's set).
    pub fn icon(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "check-circle",
            Self::Warning => "alert",
            Self::Danger => "x-circle",
            Self::Agent => "spark",
        }
    }
}

/// An alert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    /// Tone.
    pub tone: AlertTone,
    /// Title.
    pub title: String,
    /// Details.
    pub body: Option<String>,
    /// Shown.
    pub open: bool,
    /// Dismiss button.
    pub dismissible: bool,
    /// Oxyn plus: action labels (Retry, Show logs).
    pub actions: Vec<String>,
    /// Oxyn plus: collapsible technical details (logs, stack).
    pub details: Option<String>,
    /// Details expanded.
    pub expanded: bool,
    /// Text version (bumps when copy changes so the renderer rolls it in).
    pub version: u32,
}

impl Alert {
    /// An alert.
    pub fn new(tone: AlertTone, title: &str) -> Self {
        Self {
            tone,
            title: title.into(),
            body: None,
            open: true,
            dismissible: false,
            actions: vec![],
            details: None,
            expanded: false,
            version: 0,
        }
    }
    /// With details.
    pub fn body(mut self, b: &str) -> Self {
        self.body = Some(b.into());
        self
    }
    /// Oxyn plus: what an agent shows when a run fails.
    pub fn agent_error(run: &str, error: &str, logs: &str) -> Self {
        Self {
            actions: vec!["Retry".into(), "Show logs".into()],
            details: Some(logs.into()),
            dismissible: true,
            ..Self::new(AlertTone::Danger, &format!("{run} failed")).body(error)
        }
    }
    /// Oxyn plus: tone and title from an agent state.
    pub fn from_agent(state: &AgentState, run: &str) -> Self {
        match state {
            AgentState::Error { .. } => Self::new(AlertTone::Danger, &format!("{run} failed")),
            AgentState::AwaitingApproval { .. } => Self::new(
                AlertTone::Warning,
                &format!("{run} is waiting for your approval"),
            ),
            AgentState::Done { .. } => Self::new(AlertTone::Success, &format!("{run} finished")),
            AgentState::Stalled { .. } => {
                Self::new(AlertTone::Warning, &format!("{run} looks stuck"))
            }
            _ if state.is_busy() => Self::new(AlertTone::Agent, &format!("{run} is working")),
            _ => Self::new(AlertTone::Info, run),
        }
    }
    /// Morphs to new copy / tone (one alert kept mounted; Arc's advice).
    pub fn set(&mut self, tone: AlertTone, title: &str, body: Option<&str>) {
        if self.tone != tone || self.title != title || self.body.as_deref() != body {
            self.version += 1;
        }
        self.tone = tone;
        self.title = title.into();
        self.body = body.map(Into::into);
    }
    /// Dismiss: collapses first; the host hears about it after.
    pub fn dismiss(&mut self) {
        self.open = false;
    }
    /// Dismiss button label.
    pub fn dismiss_label(&self) -> String {
        format!("Dismiss: {}", self.title)
    }
    /// Toggle the details.
    pub fn toggle_details(&mut self) -> bool {
        if self.details.is_some() {
            self.expanded = !self.expanded;
        }
        self.expanded
    }
}

/// A card's status edge (Oxyn plus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardEdge {
    /// None.
    #[default]
    None,
    /// An agent is working on it: a beam runs round the border.
    Working,
    /// Needs attention.
    Attention,
    /// Done.
    Done,
    /// Failed.
    Error,
}

impl CardEdge {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "working" => Self::Working,
            "attention" => Self::Attention,
            "done" => Self::Done,
            "error" => Self::Error,
            _ => Self::None,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Working => "working",
            Self::Attention => "attention",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
    /// From an agent state.
    pub fn from_agent(s: &AgentState) -> Self {
        match s {
            AgentState::Error { .. } => Self::Error,
            AgentState::AwaitingApproval { .. } | AgentState::Stalled { .. } => Self::Attention,
            AgentState::Done { .. } => Self::Done,
            _ if s.is_busy() => Self::Working,
            _ => Self::None,
        }
    }
}

/// One word of a status line as it rolls in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    /// Text.
    pub text: String,
    /// Changed from the previous line (rolls in with a stagger).
    pub changed: bool,
    /// Stagger index among the changed words.
    pub order: usize,
}

/// Word-by-word diff for status lines ("Updated 2 hours ago" → "Updated just now").
pub fn roll_words(old: &str, new: &str) -> Vec<Word> {
    let o: Vec<&str> = old.split_whitespace().collect();
    let mut k = 0;
    new.split_whitespace()
        .enumerate()
        .map(|(i, w)| {
            let changed = o.get(i) != Some(&w);
            let order = if changed {
                k += 1;
                k - 1
            } else {
                0
            };
            Word {
                text: w.into(),
                changed,
                order,
            }
        })
        .collect()
}

/// Card state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Card {
    /// Quick look open.
    pub open: bool,
    /// Oxyn plus: selected (selectable cards).
    pub selected: bool,
    /// Current status line.
    pub status: Option<String>,
    prev_status: Option<String>,
}

impl Card {
    /// A card.
    pub fn new(status: Option<&str>) -> Self {
        Self {
            status: status.map(Into::into),
            ..Default::default()
        }
    }
    /// New status (rolls in).
    pub fn set_status(&mut self, s: &str) {
        self.prev_status = self.status.replace(s.into());
    }
    /// Words of the status with what changed.
    pub fn status_words(&self) -> Vec<Word> {
        roll_words(
            self.prev_status.as_deref().unwrap_or(""),
            self.status.as_deref().unwrap_or(""),
        )
    }
    /// Title press: opens the quick look (when it has details).
    pub fn open_quick_look(&mut self, has_details: bool) -> bool {
        if has_details {
            self.open = true;
        }
        self.open
    }
    /// Escape / close.
    pub fn close(&mut self) {
        self.open = false;
    }
    /// Space / click on a selectable card.
    pub fn toggle_selected(&mut self) -> bool {
        self.selected = !self.selected;
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn alert_roles_and_morph() {
        assert_eq!(AlertTone::Danger.role(), "alert");
        assert_eq!(AlertTone::Warning.role(), "status");
        let mut a = Alert::new(AlertTone::Info, "Syncing");
        a.set(AlertTone::Success, "Synced", Some("All 12 files"));
        assert_eq!(a.version, 1);
        a.set(AlertTone::Success, "Synced", Some("All 12 files"));
        assert_eq!(a.version, 1, "no change, no roll");
        assert_eq!(a.dismiss_label(), "Dismiss: Synced");
        let mut e = Alert::agent_error("Deploy", "Build exited with code 1", "error[E0308]");
        assert_eq!(e.actions, vec!["Retry", "Show logs"]);
        assert!(e.toggle_details());
        let f = Alert::from_agent(
            &AgentState::Done {
                elapsed: Duration::from_secs(1),
                waited: Duration::ZERO,
            },
            "Run",
        );
        assert_eq!(f.tone, AlertTone::Success);
        assert_eq!(
            Alert::from_agent(&AgentState::Streaming, "Run").tone,
            AlertTone::Agent
        );
    }

    #[test]
    fn card_status_rolls_words() {
        let mut c = Card::new(Some("Updated 2 hours ago"));
        c.set_status("Updated just now");
        let w = c.status_words();
        assert!(!w[0].changed && w[1].changed && w[2].changed);
        assert_eq!((w[1].order, w[2].order), (0, 1));
        assert!(!c.open_quick_look(false));
        assert!(c.open_quick_look(true));
        c.close();
        assert!(c.toggle_selected());
        assert_eq!(
            CardEdge::from_agent(&AgentState::Streaming),
            CardEdge::Working
        );
    }
}
