//! The composer (family A, `docs/specs/composer.md`): one derived [`ComposerState`] that every part
//! reads, plus the renderer-free models of the parts in plan #18: the input shell, attachments,
//! model picker, effort control and context meter.
//!
//! Every part is a pure function of its inputs and the shared clock. Renderers own focus, text
//! editing and pointer events, then hand the results back here (`Draft`, `Picker::key`, …).

pub mod attachments;
pub mod context;
pub mod effort;
pub mod input;
pub mod model;

use std::time::Duration;

use crate::{AgentState, ClockMs};

pub use attachments::{
    AcceptRule, AttachmentItem, AttachmentKind, AttachmentState, AttachmentVariant, ContextRef,
    FileType, PendingPolicy, Reject, SendGate, UploadRing,
};
pub use context::{
    CategoryUsage, ContextCategory, ContextLevel, ContextMeter, ContextUsage, ContextVariant,
    LabelMode, Thresholds,
};
pub use effort::{
    EffortControl, EffortLevel, EffortSetting, EffortVariant, IgnitePolicy, ReasoningSupport,
};
pub use input::{
    Draft, History, InputLimit, InputVariant, LimitState, PasteOutcome, PastePolicy, Placeholder,
    RowRange,
};
pub use model::{
    Capabilities, DraftNeeds, MidRunPolicy, ModelCatalog, ModelInfo, ModelPickerVariant,
    ModelSelection, ModelStatus, Picker, PickerKey, PickerRow, SelectOutcome,
};

/// Shared composer motion tokens (`composer.md` §4).
pub mod timing {
    use std::time::Duration;

    /// Press-down, reduced-motion crossfades.
    pub const INSTANT: Duration = Duration::from_millis(80);
    /// Hover, colour, small fades.
    pub const FAST: Duration = Duration::from_millis(120);
    /// Morphs, chip enter, popovers.
    pub const BASE: Duration = Duration::from_millis(180);
    /// Text flying between composer and transcript (cap).
    pub const TRAVEL: Duration = Duration::from_millis(260);
    /// Minimum hold of a placeholder before it may change again.
    pub const PLACEHOLDER_HOLD: Duration = Duration::from_millis(600);
}

/// Composer-level layouts (`composer.md` §5, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ComposerLayout {
    /// Centred card, max-width 760 px, footer row of pills.
    #[default]
    Chat,
    /// Docked to a 280–480 px side panel; 28 px rows; icon pills.
    IdePanel,
    /// Wide floating composer with a repo / branch / env tray under it.
    Desktop,
    /// Bottom-docked; footer collapses to `+`, model and send.
    Mobile,
    /// Single-line `›` prompt with glyph send (TUI).
    Terminal,
    /// The composer as a node on a canvas, context wired in.
    Node,
}

impl ComposerLayout {
    /// Every free layout.
    pub const ALL: [ComposerLayout; 6] = [
        Self::Chat,
        Self::IdePanel,
        Self::Desktop,
        Self::Mobile,
        Self::Terminal,
        Self::Node,
    ];

    /// Registry id (`composer/chat`, …).
    pub fn id(self) -> &'static str {
        match self {
            Self::Chat => "composer/chat",
            Self::IdePanel => "composer/ide-panel",
            Self::Desktop => "composer/desktop",
            Self::Mobile => "composer/mobile",
            Self::Terminal => "composer/terminal",
            Self::Node => "composer/node",
        }
    }

    /// Parses `chat` or `composer/chat`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("composer/").unwrap_or(id);
        Self::ALL.into_iter().find(|l| &l.id()[9..] == id)
    }

    /// Default part variants for this layout.
    pub fn defaults(self) -> LayoutDefaults {
        use AttachmentVariant as A;
        use ContextVariant as C;
        use EffortVariant as E;
        use InputVariant as I;
        use ModelPickerVariant as M;
        let (input, attachments, model, effort, context) = match self {
            Self::Chat => (
                I::RoundedBox,
                A::TileRing,
                M::PillPopover,
                E::SignalBars,
                C::Ring,
            ),
            Self::IdePanel => (I::IdeInline, A::Chip, M::Compact, E::Menu, C::Text),
            Self::Desktop => (
                I::Floating,
                A::TileRing,
                M::PillPopover,
                E::Segmented,
                C::Ring,
            ),
            Self::Mobile => (
                I::DockedBar,
                A::ThumbStrip,
                M::PillPopover,
                E::Menu,
                C::Ring,
            ),
            Self::Terminal => (
                I::TerminalPrompt,
                A::Terminal,
                M::Glyph,
                E::Chevrons,
                C::Text,
            ),
            Self::Node => (
                I::RoundedBox,
                A::ContextChip,
                M::PillPopover,
                E::Slider,
                C::Bar,
            ),
        };
        LayoutDefaults {
            input,
            attachments,
            model,
            effort,
            context,
        }
    }
}

/// The part variants a layout turns on by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutDefaults {
    /// Input shell.
    pub input: InputVariant,
    /// Attachment row.
    pub attachments: AttachmentVariant,
    /// Model picker.
    pub model: ModelPickerVariant,
    /// Effort control.
    pub effort: EffortVariant,
    /// Context meter.
    pub context: ContextVariant,
}

/// What Enter does while the agent works (`composer.md` §1 rule 3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BusyAction {
    /// Queue the draft as the next turn.
    #[default]
    Queue,
    /// Steer the running turn.
    Steer,
    /// Stop the run and send.
    Interrupt,
    /// Ask each time.
    Ask,
}

/// Why the composer can't send right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisabledReason {
    /// No connection; typing still works and the draft is kept.
    Offline,
    /// Usage limit; `resets` is the host's own label ("14:00").
    Quota {
        /// When the quota resets, as the host formats it.
        resets: String,
    },
    /// Signed out.
    Auth,
    /// Workspace policy, with its reason.
    Policy(String),
}

impl DisabledReason {
    /// The reason line under the shell.
    pub fn label(&self) -> String {
        match self {
            Self::Offline => "You're offline · your draft is saved".into(),
            Self::Quota { resets } => format!("Usage limit reached · resets {resets}"),
            Self::Auth => "Sign in to send".into(),
            Self::Policy(r) => r.clone(),
        }
    }
}

/// The surface the composer lives on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Surface {
    /// Web or desktop chat.
    #[default]
    Chat,
    /// Editor side panel.
    Ide,
    /// Standalone desktop app.
    Desktop,
    /// Phone (touch keyboard: Enter inserts a newline by default).
    Mobile,
    /// Voice-first.
    Voice,
    /// Terminal UI.
    Terminal,
}

/// The whole composer's phase (`composer.md` §3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerPhase {
    /// Nothing typed, agent idle.
    Empty,
    /// A draft, agent idle.
    Composing,
    /// Just sent, no event yet.
    Submitted,
    /// Agent working, no draft.
    Busy,
    /// Agent working, user typing (Enter follows [`BusyAction`]).
    BusyWithDraft,
    /// An approval or question waits on the user.
    AwaitingYou,
    /// Can't send.
    Disabled(DisabledReason),
}

/// The draft facts the phase needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DraftSummary {
    /// No text and no attachments.
    pub is_empty: bool,
    /// Grapheme count.
    pub chars: usize,
    /// Attachments in the draft.
    pub attachments: usize,
    /// Attachments still uploading or reading.
    pub uploading: usize,
    /// An IME composition is active (Enter never sends).
    pub ime_composing: bool,
    /// The draft is over its limit.
    pub over_limit: bool,
}

/// Settings chosen for the next send.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnSettings {
    /// Model for the next turn.
    pub model: ModelSelection,
    /// Effort for the next turn.
    pub effort: EffortSetting,
}

/// What pressing Enter (or the send control) does now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmitAction {
    /// Nothing; the reason is announced.
    Blocked(String),
    /// Send a new turn.
    Send,
    /// Hold the send until uploads finish.
    WaitForUploads(usize),
    /// Queue, steer, interrupt or ask, per [`BusyAction`].
    Busy(BusyAction),
    /// The pending card owns Enter.
    AnswerCard,
}

/// The single derived value every composer part renders from (`composer.md` §3).
#[derive(Clone, Debug, PartialEq)]
pub struct ComposerState {
    /// The single source of truth.
    pub agent: AgentState,
    /// Derived phase.
    pub phase: ComposerPhase,
    /// The draft.
    pub draft: DraftSummary,
    /// Why sending is off, if it is.
    pub disabled: Option<DisabledReason>,
    /// The busy policy.
    pub busy_action: BusyAction,
    /// Settings for the next turn.
    pub next_turn: TurnSettings,
    /// Surface.
    pub surface: Surface,
}

impl ComposerState {
    /// Derives the composer state. Pure; no I/O, no timers.
    pub fn derive(
        agent: &AgentState,
        draft: DraftSummary,
        disabled: Option<DisabledReason>,
        busy_action: BusyAction,
        next_turn: TurnSettings,
        surface: Surface,
    ) -> Self {
        let phase = if let Some(d) = &disabled {
            ComposerPhase::Disabled(d.clone())
        } else {
            match agent {
                AgentState::AwaitingApproval { .. } => ComposerPhase::AwaitingYou,
                AgentState::Submitted => ComposerPhase::Submitted,
                a if a.is_busy() && draft.is_empty => ComposerPhase::Busy,
                a if a.is_busy() => ComposerPhase::BusyWithDraft,
                _ if draft.is_empty => ComposerPhase::Empty,
                _ => ComposerPhase::Composing,
            }
        };
        Self {
            agent: agent.clone(),
            phase,
            draft,
            disabled,
            busy_action,
            next_turn,
            surface,
        }
    }

    /// True while settings changes wait for the next turn (`composer.md` §1 rule 4).
    pub fn mid_run(&self) -> bool {
        mid_run(&self.agent)
    }

    /// What Enter does now. Never sends during IME composition.
    pub fn submit_action(&self) -> SubmitAction {
        if self.draft.ime_composing {
            return SubmitAction::Blocked("Finishing composition".into());
        }
        match &self.phase {
            ComposerPhase::Disabled(r) => SubmitAction::Blocked(r.label()),
            ComposerPhase::Empty | ComposerPhase::Busy => {
                SubmitAction::Blocked("Nothing to send".into())
            }
            ComposerPhase::AwaitingYou => SubmitAction::AnswerCard,
            _ if self.draft.over_limit => SubmitAction::Blocked("Message is too long".into()),
            ComposerPhase::Composing if self.draft.uploading > 0 => {
                SubmitAction::WaitForUploads(self.draft.uploading)
            }
            ComposerPhase::Composing => SubmitAction::Send,
            ComposerPhase::Submitted | ComposerPhase::BusyWithDraft => {
                SubmitAction::Busy(self.busy_action)
            }
        }
    }
}

/// A turn is in flight: model, effort and mode changes apply to the next turn.
pub fn mid_run(agent: &AgentState) -> bool {
    agent.is_busy() || matches!(agent, AgentState::AwaitingApproval { .. })
}

/// A value that changes no faster than `hold` (placeholders, labels that must not flicker).
#[derive(Clone, Debug)]
pub struct Held<T> {
    shown: T,
    pending: Option<T>,
    since: ClockMs,
    hold: Duration,
}

impl<T: Clone + PartialEq> Held<T> {
    /// Starts showing `value` at `now`.
    pub fn new(value: T, hold: Duration, now: ClockMs) -> Self {
        Self {
            shown: value,
            pending: None,
            since: now,
            hold,
        }
    }

    /// Offers the latest value; returns what to show. A new value replaces the shown one only
    /// after the shown one has been up for `hold`; the newest offer wins.
    pub fn update(&mut self, value: T, now: ClockMs) -> &T {
        if value == self.shown {
            self.pending = None;
        } else {
            self.pending = Some(value);
        }
        if self.pending.is_some() && now.since(self.since) >= self.hold {
            self.shown = self.pending.take().expect("checked");
            self.since = now;
        }
        &self.shown
    }

    /// The value on screen.
    pub fn shown(&self) -> &T {
        &self.shown
    }

    /// When the shown value appeared (drives the swap animation).
    pub fn since(&self) -> ClockMs {
        self.since
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(agent: AgentState, draft: DraftSummary) -> ComposerState {
        ComposerState::derive(
            &agent,
            draft,
            None,
            BusyAction::Queue,
            TurnSettings::default(),
            Surface::Chat,
        )
    }

    #[test]
    fn phases() {
        let empty = DraftSummary {
            is_empty: true,
            ..Default::default()
        };
        let text = DraftSummary {
            chars: 4,
            ..Default::default()
        };
        assert_eq!(st(AgentState::Idle, empty).phase, ComposerPhase::Empty);
        assert_eq!(st(AgentState::Idle, text).phase, ComposerPhase::Composing);
        let busy = AgentState::Streaming;
        assert_eq!(st(busy.clone(), empty).phase, ComposerPhase::Busy);
        assert_eq!(st(busy, text).phase, ComposerPhase::BusyWithDraft);
        assert_eq!(
            st(AgentState::Idle, text).submit_action(),
            SubmitAction::Send
        );
        assert_eq!(
            st(AgentState::Streaming, text).submit_action(),
            SubmitAction::Busy(BusyAction::Queue)
        );
        let ime = DraftSummary {
            ime_composing: true,
            ..text
        };
        assert!(matches!(
            st(AgentState::Idle, ime).submit_action(),
            SubmitAction::Blocked(_)
        ));
        let up = DraftSummary {
            uploading: 2,
            ..text
        };
        assert_eq!(
            st(AgentState::Idle, up).submit_action(),
            SubmitAction::WaitForUploads(2)
        );
    }

    #[test]
    fn disabled_wins() {
        let s = ComposerState::derive(
            &AgentState::Idle,
            DraftSummary::default(),
            Some(DisabledReason::Offline),
            BusyAction::Queue,
            TurnSettings::default(),
            Surface::Chat,
        );
        assert_eq!(
            s.submit_action(),
            SubmitAction::Blocked("You're offline · your draft is saved".into())
        );
    }

    #[test]
    fn held_values_do_not_flicker() {
        let mut h = Held::new("a", Duration::from_millis(600), ClockMs(0));
        assert_eq!(*h.update("b", ClockMs(100)), "a");
        assert_eq!(*h.update("c", ClockMs(300)), "a");
        assert_eq!(*h.update("c", ClockMs(600)), "c");
        assert_eq!(*h.update("a", ClockMs(700)), "c");
        assert_eq!(*h.update("c", ClockMs(800)), "c"); // returned before the hold: no swap
        assert_eq!(*h.update("c", ClockMs(5000)), "c");
    }

    #[test]
    fn layouts_round_trip() {
        for l in ComposerLayout::ALL {
            assert_eq!(ComposerLayout::from_id(l.id()), Some(l));
        }
        assert_eq!(
            ComposerLayout::from_id("terminal"),
            Some(ComposerLayout::Terminal)
        );
    }
}
