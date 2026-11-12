//! Send control model (`docs/specs/composer-send-control.md`): one state machine for send, stop,
//! queue, steer, pause, answer, dictation confirm and retry, with the 12 free variants as views of
//! it. Timing guards (double submit, fast-agent no-flash, stop confirm) live here so every
//! renderer behaves the same.

use std::time::Duration;

use super::{BusyAction, ComposerPhase, ComposerState, SubmitAction};
use crate::{AgentState, ClockMs};

/// Send control variants (§5, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SendVariant {
    /// Round ↑ → ■.
    #[default]
    Classic,
    /// ■ with a thin elapsed / budget ring.
    RingProgress,
    /// ■ widens into a labelled "Queue" pill when a draft exists.
    QueuePill,
    /// ↑ ▾ split; mid-run ■ │ ↑ (stop │ queue).
    Split,
    /// A steer button slides in beside ■ when there is a draft.
    SteerSlide,
    /// Stop lives in the status tab; the button stays ↑ for queue.
    StatusTab,
    /// Separate pause and stop.
    PauseStop,
    /// `⏎` / `■` / `⇥ queue` (TUI).
    Glyph,
    /// Mic → ✓ while dictating.
    VoiceConfirm,
    /// ↑ answers the pending card.
    AnswerCard,
    /// ⟲ with shake and "Couldn't send · Retry".
    Retry,
    /// Text-only "Send / Stop / Queue".
    Minimal,
}

impl SendVariant {
    /// Every free variant.
    pub const ALL: [SendVariant; 12] = [
        Self::Classic,
        Self::RingProgress,
        Self::QueuePill,
        Self::Split,
        Self::SteerSlide,
        Self::StatusTab,
        Self::PauseStop,
        Self::Glyph,
        Self::VoiceConfirm,
        Self::AnswerCard,
        Self::Retry,
        Self::Minimal,
    ];

    /// Short id; registry id is `send-control/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::RingProgress => "ring-progress",
            Self::QueuePill => "queue-pill",
            Self::Split => "split",
            Self::SteerSlide => "steer-slide",
            Self::StatusTab => "status-tab",
            Self::PauseStop => "pause-stop",
            Self::Glyph => "glyph",
            Self::VoiceConfirm => "voice-confirm",
            Self::AnswerCard => "answer-card",
            Self::Retry => "retry",
            Self::Minimal => "minimal",
        }
    }

    /// Parses `split` or `send-control/split`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("send-control/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
}

/// The button's state (§2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendState {
    /// Nothing to send.
    Empty,
    /// A draft, agent idle.
    Ready,
    /// Sent, no event yet (■ already).
    Sending,
    /// Agent working, no draft.
    Working,
    /// Agent working, draft present: Enter follows `busy_action`.
    WorkingWithDraft,
    /// Paused by the user (pause-stop variant).
    Paused,
    /// A steer note is being absorbed.
    Steering,
    /// ↑ answers the pending approval / question card.
    Answering,
    /// Dictation open: ✓ accepts.
    Dictating,
    /// Waiting for uploads.
    WaitingUploads(usize),
    /// Can't send, with the reason.
    Disabled(String),
    /// The last send failed.
    Error(String),
}

/// What a press does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendAction {
    /// Nothing.
    None,
    /// Send a new turn.
    Send,
    /// Stop the run.
    Stop,
    /// Queue the draft.
    Queue,
    /// Steer the running turn.
    Steer,
    /// Stop and send now.
    Interrupt,
    /// Ask which (shake Steer / Queue / Stop).
    Choose,
    /// Pause the run.
    Pause,
    /// Resume a paused run.
    Resume,
    /// Submit the card answer.
    Answer,
    /// Insert the dictated text.
    Accept,
    /// Retry the failed send.
    Retry,
}

impl SendState {
    /// Derives the button state from the composer state and the control's own flags.
    pub fn derive(cs: &ComposerState, error: Option<&str>, dictating: bool, paused: bool) -> Self {
        if dictating {
            return Self::Dictating;
        }
        if let Some(e) = error {
            return Self::Error(e.to_string());
        }
        if paused && cs.agent.is_busy() {
            return Self::Paused;
        }
        match &cs.phase {
            ComposerPhase::Disabled(r) => Self::Disabled(r.label()),
            ComposerPhase::AwaitingYou => Self::Answering,
            ComposerPhase::Submitted => Self::Sending,
            ComposerPhase::Busy if matches!(cs.agent, AgentState::Steering) => Self::Steering,
            ComposerPhase::Busy => Self::Working,
            ComposerPhase::BusyWithDraft => Self::WorkingWithDraft,
            ComposerPhase::Empty => Self::Empty,
            ComposerPhase::Composing => match cs.submit_action() {
                SubmitAction::WaitForUploads(n) => Self::WaitingUploads(n),
                SubmitAction::Blocked(r) => Self::Disabled(r),
                _ => Self::Ready,
            },
        }
    }

    /// A run is in flight (the button offers stop).
    pub fn busy(&self) -> bool {
        matches!(
            self,
            Self::Sending | Self::Working | Self::WorkingWithDraft | Self::Steering | Self::Paused
        )
    }
}

/// The glyph a button draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendGlyph {
    /// ↑.
    Up,
    /// ■.
    Stop,
    /// ❚❚.
    Pause,
    /// ▶.
    Resume,
    /// ✓.
    Check,
    /// ⟲.
    Retry,
    /// ▾ (split menu).
    Menu,
    /// ↳ steer.
    Steer,
}

impl SendGlyph {
    /// Text fallback.
    pub fn text(self) -> &'static str {
        match self {
            Self::Up => "↑",
            Self::Stop => "■",
            Self::Pause => "❚❚",
            Self::Resume => "▶",
            Self::Check => "✓",
            Self::Retry => "⟲",
            Self::Menu => "▾",
            Self::Steer => "↳",
        }
    }
}

/// Visual weight of a button segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Filled with the accent (in the default themes: white on dark, black on light).
    Primary,
    /// Filled with the strong foreground (stop).
    Strong,
    /// Quiet surface.
    Muted,
    /// Outlined, labelled (queue pill, steer).
    Soft,
    /// Error.
    Danger,
}

/// One pressable part of the control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendSegment {
    /// What it draws.
    pub glyph: Option<SendGlyph>,
    /// Visible text (`Queue`, `Steer`, `⏎ send`), if any.
    pub label: Option<String>,
    /// Accessible name (§7).
    pub aria: String,
    /// What a press does.
    pub action: SendAction,
    /// Weight.
    pub tone: Tone,
    /// Pressable.
    pub enabled: bool,
}

/// Everything a renderer needs for one frame.
#[derive(Clone, Debug, PartialEq)]
pub struct SendView {
    /// The state.
    pub state: SendState,
    /// Segments, leading to trailing (send is the trailing-most).
    pub segments: Vec<SendSegment>,
    /// Inline note (`Couldn't send · Retry`). Upload waits are the composer's hint row.
    pub note: Option<String>,
    /// Ring fraction for `ring-progress` (elapsed minute or budget).
    pub ring: Option<f32>,
    /// Horizontal shake offset in px (error, busy choice).
    pub shake_px: f32,
    /// Glyph morph progress `0..=1` since the last state change.
    pub morph: f32,
    /// Scale for the Empty → Ready light-up.
    pub scale: f32,
    /// Esc again (or keep holding) to stop.
    pub confirm_hint: Option<&'static str>,
}

/// How stop is confirmed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StopConfirm {
    /// Stop at once.
    #[default]
    None,
    /// Esc twice within 1.2 s.
    DoubleEsc,
    /// Hold for the given time (touch).
    Hold(Duration),
}

/// Send-control timings (§4).
pub mod timing {
    use std::time::Duration;

    /// Empty → Ready light-up.
    pub const LIGHT_UP: Duration = Duration::from_millis(120);
    /// Glyph morph.
    pub const MORPH: Duration = Duration::from_millis(160);
    /// Queue pill widen.
    pub const WIDEN: Duration = Duration::from_millis(180);
    /// Error / choice shake (2 cycles, 6 px).
    pub const SHAKE: Duration = Duration::from_millis(300);
    /// Ignore a second Enter within this window while sending.
    pub const DOUBLE_SUBMIT: Duration = Duration::from_millis(300);
    /// Esc twice within this window to stop.
    pub const DOUBLE_ESC: Duration = Duration::from_millis(1200);
    /// A run that ends within this time of sending never shows ■.
    pub const NO_FLASH: Duration = Duration::from_millis(50);
}

/// The control's own state: timings, guards and announcements. One per composer.
#[derive(Clone, Debug)]
pub struct SendControl {
    /// Variant.
    pub variant: SendVariant,
    /// Stop confirmation.
    pub stop_confirm: StopConfirm,
    /// Reduced motion: no shake, morph or scale.
    pub reduced: bool,
    last: Option<SendState>,
    changed_at: ClockMs,
    sent_at: Option<ClockMs>,
    shake_at: Option<ClockMs>,
    esc_at: Option<ClockMs>,
    hold_from: Option<ClockMs>,
    announce: Option<String>,
}

impl SendControl {
    /// A control of `variant`.
    pub fn new(variant: SendVariant) -> Self {
        Self {
            variant,
            stop_confirm: StopConfirm::None,
            reduced: false,
            last: None,
            changed_at: ClockMs(0),
            sent_at: None,
            shake_at: None,
            esc_at: None,
            hold_from: None,
            announce: None,
        }
    }

    /// The polite live-region text since the last call (`Message queued`), taken once.
    pub fn take_announcement(&mut self) -> Option<String> {
        self.announce.take()
    }

    /// Resolves a press of `action` at `now`, applying the guards. Returns what the host should
    /// do (possibly [`SendAction::None`]).
    pub fn press(&mut self, action: SendAction, now: ClockMs) -> SendAction {
        match action {
            SendAction::Send => {
                if self
                    .sent_at
                    .is_some_and(|t| now.since(t) < timing::DOUBLE_SUBMIT)
                {
                    return SendAction::None;
                }
                self.sent_at = Some(now);
                self.announce = Some("Message sent".into());
                SendAction::Send
            }
            SendAction::Choose => {
                self.shake_at = Some(now);
                SendAction::Choose
            }
            SendAction::Queue => {
                self.announce = Some("Message queued".into());
                SendAction::Queue
            }
            SendAction::Steer => {
                self.announce = Some("Steering the agent".into());
                SendAction::Steer
            }
            SendAction::Stop => self.stop(now),
            a => a,
        }
    }

    fn stop(&mut self, now: ClockMs) -> SendAction {
        match self.stop_confirm {
            StopConfirm::DoubleEsc => {
                if self
                    .esc_at
                    .is_some_and(|t| now.since(t) < timing::DOUBLE_ESC)
                {
                    self.esc_at = None;
                    self.announce = Some("Generation stopped".into());
                    SendAction::Stop
                } else {
                    self.esc_at = Some(now);
                    SendAction::None
                }
            }
            StopConfirm::Hold(_) => SendAction::None, // use hold_start / hold_end
            StopConfirm::None => {
                self.announce = Some("Generation stopped".into());
                SendAction::Stop
            }
        }
    }

    /// Hold-to-stop: pointer down.
    pub fn hold_start(&mut self, now: ClockMs) {
        self.hold_from = Some(now);
    }

    /// Hold-to-stop: pointer up. Returns `Stop` when held long enough.
    pub fn hold_end(&mut self, now: ClockMs) -> SendAction {
        let from = self.hold_from.take();
        match (self.stop_confirm, from) {
            (StopConfirm::Hold(d), Some(f)) if now.since(f) >= d => {
                self.announce = Some("Generation stopped".into());
                SendAction::Stop
            }
            _ => SendAction::None,
        }
    }

    /// Enter in the composer: maps the state and busy policy to an action and presses it.
    pub fn enter(
        &mut self,
        state: &SendState,
        busy: BusyAction,
        mod_enter: bool,
        now: ClockMs,
    ) -> SendAction {
        let a = match state {
            SendState::Ready => SendAction::Send,
            SendState::WorkingWithDraft => match (busy, mod_enter) {
                (BusyAction::Queue, false) | (BusyAction::Steer, true) => SendAction::Queue,
                (BusyAction::Queue, true) | (BusyAction::Steer, false) => SendAction::Steer,
                (BusyAction::Interrupt, _) => SendAction::Interrupt,
                (BusyAction::Ask, _) => SendAction::Choose,
            },
            SendState::Answering => SendAction::Answer,
            SendState::Dictating => SendAction::Accept,
            SendState::Error(_) => SendAction::Retry,
            _ => SendAction::None,
        };
        self.press(a, now)
    }

    /// Esc in the composer while busy.
    pub fn escape(&mut self, state: &SendState, now: ClockMs) -> SendAction {
        if state.busy() {
            self.stop(now)
        } else {
            SendAction::None
        }
    }

    /// The frame for `state`. `elapsed` drives the ring (one turn per minute) unless `budget`
    /// (fraction used) is given.
    pub fn view(
        &mut self,
        state: SendState,
        busy: BusyAction,
        elapsed: Duration,
        budget: Option<f32>,
        now: ClockMs,
    ) -> SendView {
        // Fast agent: a run that ended right after sending never flashes ■.
        // Fast agent: ■ appears only once sending has lasted a frame, so a run that ends at
        // once never flashes stop.
        let state = if state == SendState::Sending
            && self
                .sent_at
                .is_some_and(|t| now.since(t) < timing::NO_FLASH)
        {
            SendState::Empty
        } else {
            state
        };
        let from_empty = matches!(self.last, Some(SendState::Empty));
        if self.last.as_ref() != Some(&state) {
            if matches!(state, SendState::Error(_)) {
                self.shake_at = Some(now);
            }
            self.changed_at = now;
            self.last = Some(state.clone());
        }
        let dt = now.since(self.changed_at);
        let frac = |d: Duration| (dt.as_secs_f32() / d.as_secs_f32()).clamp(0.0, 1.0);
        let morph = if self.reduced {
            1.0
        } else {
            frac(timing::MORPH)
        };
        let scale = if self.reduced || !(from_empty || dt < timing::LIGHT_UP) {
            1.0
        } else if state == SendState::Ready {
            let t = frac(timing::LIGHT_UP);
            0.92 + 0.08 * (1.0 - (1.0 - t).powi(3))
        } else {
            1.0
        };
        let shake_px = match self.shake_at {
            Some(t) if !self.reduced && now.since(t) < timing::SHAKE => {
                let p = now.since(t).as_secs_f32() / timing::SHAKE.as_secs_f32();
                6.0 * (1.0 - p) * (p * 2.0 * std::f32::consts::TAU).sin()
            }
            _ => 0.0,
        };
        let confirm_hint = match self.stop_confirm {
            StopConfirm::DoubleEsc
                if self
                    .esc_at
                    .is_some_and(|t| now.since(t) < timing::DOUBLE_ESC) =>
            {
                Some("Esc again to stop")
            }
            StopConfirm::Hold(_) if self.hold_from.is_some() => Some("Keep holding to stop"),
            _ => None,
        };
        let ring = (self.variant == SendVariant::RingProgress && state.busy())
            .then(|| budget.unwrap_or_else(|| (elapsed.as_secs_f32() % 60.0) / 60.0));
        let (segments, note) = segments(self.variant, &state, busy);
        SendView {
            state,
            segments,
            note,
            ring,
            shake_px,
            morph,
            scale,
            confirm_hint,
        }
    }

    /// Frames are needed.
    pub fn animating(&self, now: ClockMs) -> bool {
        now.since(self.changed_at) < timing::WIDEN
            || self.shake_at.is_some_and(|t| now.since(t) < timing::SHAKE)
            || (self.variant == SendVariant::RingProgress
                && self.last.as_ref().is_some_and(SendState::busy))
    }
}

fn seg(
    glyph: Option<SendGlyph>,
    label: Option<&str>,
    aria: impl Into<String>,
    action: SendAction,
    tone: Tone,
) -> SendSegment {
    SendSegment {
        glyph,
        label: label.map(str::to_string),
        aria: aria.into(),
        action,
        tone,
        enabled: action != SendAction::None,
    }
}

fn busy_draft_action(busy: BusyAction) -> (SendAction, &'static str, &'static str) {
    match busy {
        BusyAction::Queue => (SendAction::Queue, "Queue", "Queue message"),
        BusyAction::Steer => (SendAction::Steer, "Steer", "Steer now"),
        BusyAction::Interrupt => (SendAction::Interrupt, "Send now", "Stop and send"),
        BusyAction::Ask => (SendAction::Choose, "Send…", "Choose: steer or queue"),
    }
}

/// The segments each variant shows for a state (§2 table, §5).
pub fn segments(
    v: SendVariant,
    s: &SendState,
    busy: BusyAction,
) -> (Vec<SendSegment>, Option<String>) {
    use SendGlyph as G;
    use SendVariant as V;
    use Tone as T;
    let stop = || {
        seg(
            Some(G::Stop),
            None,
            "Stop generating",
            SendAction::Stop,
            T::Strong,
        )
    };
    let send = || {
        seg(
            Some(G::Up),
            None,
            "Send message",
            SendAction::Send,
            T::Primary,
        )
    };
    let (ba, bl, baria) = busy_draft_action(busy);
    // Text-only and glyph variants share the state logic with their own drawing.
    if v == V::Minimal || v == V::Glyph {
        let g = v == V::Glyph;
        let one = |label: &str, glyph_label: &str, aria: &str, a: SendAction, t: T| {
            vec![seg(
                None,
                Some(if g { glyph_label } else { label }),
                aria,
                a,
                t,
            )]
        };
        let segs = match s {
            SendState::Empty => one(
                "Send",
                "⏎ send",
                "Send message (type a message first)",
                SendAction::None,
                T::Muted,
            ),
            SendState::Ready => one(
                "Send",
                "⏎ send",
                "Send message",
                SendAction::Send,
                T::Primary,
            ),
            SendState::Sending | SendState::Working | SendState::Steering | SendState::Paused => {
                one(
                    "Stop",
                    "■ esc",
                    "Stop generating",
                    SendAction::Stop,
                    T::Strong,
                )
            }
            SendState::WorkingWithDraft => {
                let mut x = one(
                    "Stop",
                    "■ esc",
                    "Stop generating",
                    SendAction::Stop,
                    T::Muted,
                );
                x.push(seg(
                    None,
                    Some(if g { "⇥ queue" } else { bl }),
                    baria,
                    ba,
                    T::Soft,
                ));
                x
            }
            SendState::Answering => one(
                "Answer",
                "⏎ answer",
                "Submit answer",
                SendAction::Answer,
                T::Primary,
            ),
            SendState::Dictating => one(
                "Insert",
                "⏎ insert",
                "Insert dictated text",
                SendAction::Accept,
                T::Primary,
            ),
            SendState::WaitingUploads(_) | SendState::Disabled(_) => one(
                "Send",
                "⏎ send",
                "Send unavailable",
                SendAction::None,
                T::Muted,
            ),
            SendState::Error(_) => one(
                "Retry",
                "⟲ retry",
                "Retry sending",
                SendAction::Retry,
                T::Danger,
            ),
        };
        return (segs, note(s));
    }
    let segs = match (v, s) {
        (_, SendState::Dictating) => vec![seg(
            Some(G::Check),
            None,
            "Insert dictated text",
            SendAction::Accept,
            T::Primary,
        )],
        (_, SendState::Error(_)) => vec![seg(
            Some(G::Retry),
            None,
            "Retry sending",
            SendAction::Retry,
            T::Danger,
        )],
        (_, SendState::Answering) => vec![seg(
            Some(G::Up),
            (v == V::AnswerCard).then_some("Answer"),
            "Submit answer",
            SendAction::Answer,
            T::Primary,
        )],
        (_, SendState::Empty) => vec![seg(
            Some(G::Up),
            None,
            "Send message (type a message first)",
            SendAction::None,
            T::Muted,
        )],
        (_, SendState::WaitingUploads(_) | SendState::Disabled(_)) => {
            vec![seg(
                Some(G::Up),
                None,
                "Send unavailable",
                SendAction::None,
                T::Muted,
            )]
        }
        (V::Split, SendState::Ready) => vec![
            send(),
            seg(
                Some(G::Menu),
                None,
                "Send options",
                SendAction::Choose,
                T::Primary,
            ),
        ],
        (_, SendState::Ready) => vec![send()],
        (V::PauseStop, SendState::Paused) => vec![
            seg(Some(G::Resume), None, "Resume", SendAction::Resume, T::Soft),
            stop(),
        ],
        (V::PauseStop, SendState::Working | SendState::Sending | SendState::Steering) => vec![
            seg(Some(G::Pause), None, "Pause", SendAction::Pause, T::Soft),
            stop(),
        ],
        (V::PauseStop, SendState::WorkingWithDraft) => vec![
            seg(Some(G::Pause), None, "Pause", SendAction::Pause, T::Soft),
            stop(),
            seg(Some(G::Up), Some(bl), baria, ba, T::Soft),
        ],
        (V::StatusTab, s) if s.busy() => {
            // Stop lives in the status tab; the button only queues.
            let has_draft = matches!(s, SendState::WorkingWithDraft);
            vec![seg(
                Some(G::Up),
                None,
                if has_draft {
                    baria
                } else {
                    "Type to queue a message"
                },
                if has_draft { ba } else { SendAction::None },
                if has_draft { T::Primary } else { T::Muted },
            )]
        }
        (V::QueuePill, SendState::WorkingWithDraft) => {
            vec![seg(Some(G::Up), Some(bl), baria, ba, T::Soft)]
        }
        (V::Split, SendState::WorkingWithDraft) => {
            vec![stop(), seg(Some(G::Up), None, baria, ba, T::Primary)]
        }
        (V::SteerSlide, SendState::WorkingWithDraft) => vec![
            seg(
                Some(G::Steer),
                Some("Steer"),
                "Steer now",
                SendAction::Steer,
                T::Soft,
            ),
            stop(),
        ],
        (_, SendState::WorkingWithDraft) => vec![seg(
            Some(G::Stop),
            None,
            "Stop generating",
            SendAction::Stop,
            T::Strong,
        )],
        (_, SendState::Steering) => vec![seg(
            Some(G::Steer),
            None,
            "Steering the agent",
            SendAction::Stop,
            T::Strong,
        )],
        (_, _) => vec![stop()],
    };
    (segs, note(s))
}

fn note(s: &SendState) -> Option<String> {
    match s {
        SendState::Error(e) => Some(format!("{e} · Retry")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composer::{DraftSummary, Surface, TurnSettings};

    fn cs(agent: AgentState, text: bool) -> ComposerState {
        ComposerState::derive(
            &agent,
            DraftSummary {
                is_empty: !text,
                chars: usize::from(text),
                ..Default::default()
            },
            None,
            BusyAction::Queue,
            TurnSettings::default(),
            Surface::Chat,
        )
    }

    #[test]
    fn states() {
        assert_eq!(
            SendState::derive(&cs(AgentState::Idle, false), None, false, false),
            SendState::Empty
        );
        assert_eq!(
            SendState::derive(&cs(AgentState::Idle, true), None, false, false),
            SendState::Ready
        );
        assert_eq!(
            SendState::derive(&cs(AgentState::Streaming, false), None, false, false),
            SendState::Working
        );
        assert_eq!(
            SendState::derive(&cs(AgentState::Streaming, true), None, false, false),
            SendState::WorkingWithDraft
        );
        assert_eq!(
            SendState::derive(&cs(AgentState::Streaming, true), None, false, true),
            SendState::Paused
        );
        assert_eq!(
            SendState::derive(
                &cs(AgentState::Idle, true),
                Some("Couldn't send"),
                false,
                false
            ),
            SendState::Error("Couldn't send".into())
        );
        assert_eq!(
            SendState::derive(&cs(AgentState::Idle, true), None, true, false),
            SendState::Dictating
        );
    }

    #[test]
    fn variants_show_the_right_segments() {
        let wd = SendState::WorkingWithDraft;
        let (q, _) = segments(SendVariant::QueuePill, &wd, BusyAction::Queue);
        assert_eq!(q[0].label.as_deref(), Some("Queue"));
        let (s, _) = segments(SendVariant::Split, &wd, BusyAction::Queue);
        assert_eq!(
            s.iter().map(|x| x.action).collect::<Vec<_>>(),
            vec![SendAction::Stop, SendAction::Queue]
        );
        let (st, _) = segments(SendVariant::SteerSlide, &wd, BusyAction::Queue);
        assert_eq!(st[0].action, SendAction::Steer);
        let (tab, _) = segments(
            SendVariant::StatusTab,
            &SendState::Working,
            BusyAction::Queue,
        );
        assert!(!tab[0].enabled, "status-tab: stop is in the tab");
        let (g, _) = segments(SendVariant::Glyph, &wd, BusyAction::Queue);
        assert_eq!(g[1].label.as_deref(), Some("⇥ queue"));
        let (p, _) = segments(
            SendVariant::PauseStop,
            &SendState::Working,
            BusyAction::Queue,
        );
        assert_eq!(p[0].action, SendAction::Pause);
        let (e, n) = segments(
            SendVariant::Retry,
            &SendState::Error("Couldn't send".into()),
            BusyAction::Queue,
        );
        assert_eq!(
            (e[0].action, n.as_deref()),
            (SendAction::Retry, Some("Couldn't send · Retry"))
        );
        for v in SendVariant::ALL {
            assert_eq!(SendVariant::from_id(v.id()), Some(v));
        }
    }

    #[test]
    fn guards() {
        let mut c = SendControl::new(SendVariant::Classic);
        assert_eq!(
            c.enter(&SendState::Ready, BusyAction::Queue, false, ClockMs(0)),
            SendAction::Send
        );
        assert_eq!(
            c.enter(&SendState::Ready, BusyAction::Queue, false, ClockMs(200)),
            SendAction::None,
            "double submit"
        );
        assert_eq!(
            c.enter(
                &SendState::WorkingWithDraft,
                BusyAction::Queue,
                true,
                ClockMs(500)
            ),
            SendAction::Steer
        );
        assert_eq!(
            c.enter(
                &SendState::WorkingWithDraft,
                BusyAction::Ask,
                false,
                ClockMs(600)
            ),
            SendAction::Choose
        );
        c.stop_confirm = StopConfirm::DoubleEsc;
        assert_eq!(
            c.escape(&SendState::Working, ClockMs(1_000)),
            SendAction::None
        );
        assert_eq!(
            c.view(
                SendState::Working,
                BusyAction::Queue,
                Duration::ZERO,
                None,
                ClockMs(1_100)
            )
            .confirm_hint,
            Some("Esc again to stop")
        );
        assert_eq!(
            c.escape(&SendState::Working, ClockMs(1_500)),
            SendAction::Stop
        );
        assert_eq!(c.take_announcement().as_deref(), Some("Generation stopped"));
        c.stop_confirm = StopConfirm::Hold(Duration::from_millis(400));
        c.hold_start(ClockMs(2_000));
        assert_eq!(c.hold_end(ClockMs(2_200)), SendAction::None);
        c.hold_start(ClockMs(3_000));
        assert_eq!(c.hold_end(ClockMs(3_450)), SendAction::Stop);
    }

    #[test]
    fn motion() {
        let mut c = SendControl::new(SendVariant::RingProgress);
        c.view(
            SendState::Empty,
            BusyAction::Queue,
            Duration::ZERO,
            None,
            ClockMs(0),
        );
        let v = c.view(
            SendState::Ready,
            BusyAction::Queue,
            Duration::ZERO,
            None,
            ClockMs(1_000),
        );
        assert!(v.scale < 1.0, "light-up starts small");
        let v = c.view(
            SendState::Working,
            BusyAction::Queue,
            Duration::from_secs(15),
            None,
            ClockMs(2_000),
        );
        assert_eq!(v.ring, Some(0.25));
        let v = c.view(
            SendState::Error("x".into()),
            BusyAction::Queue,
            Duration::ZERO,
            None,
            ClockMs(3_000),
        );
        assert_eq!(v.shake_px, 0.0);
        let v = c.view(
            SendState::Error("x".into()),
            BusyAction::Queue,
            Duration::ZERO,
            None,
            ClockMs(3_040),
        );
        assert!(v.shake_px.abs() > 0.5, "shakes on error");
    }
}
