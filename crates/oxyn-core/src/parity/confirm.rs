//! Destructive buttons that ask before they act.
//!
//! - [`HoldConfirm`]: hold to confirm. The fill runs linearly while held, rewinds on a smooth
//!   curve when released early and resumes from where it was on a quick re-press.
//! - [`ConfirmMorph`]: a button that morphs into an inline question, then a spinner, then a
//!   result with Undo (or an error with Retry). Confirm by press, hold or a cancellable countdown.
//! - [`ConfirmPopover`]: a button that grows into a small card with the consequence, an optional
//!   type-to-confirm phrase, progress on Confirm, and a check or error before folding back.

use crate::ClockMs;

use super::smooth;

/// Tone of the confirm action.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfirmTone {
    /// Destroys something: the confirm control takes the danger colour.
    #[default]
    Danger,
    /// Serious but reversible: the confirm control takes the foreground.
    Neutral,
}

impl ConfirmTone {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Danger => "danger",
            Self::Neutral => "neutral",
        }
    }
    /// From a kebab id (anything else is danger).
    pub fn parse(s: &str) -> Self {
        if s == "neutral" {
            Self::Neutral
        } else {
            Self::Danger
        }
    }
}

/// Rewind time after an early release, ms.
pub const REWIND_MS: u64 = 240;

/// Hold to confirm: a button that confirms after being held for `duration_ms`.
#[derive(Clone, Debug, PartialEq)]
pub struct HoldConfirm {
    /// Hold length, ms (1200).
    pub duration_ms: u64,
    /// Fill colour.
    pub tone: ConfirmTone,
    /// Fill (0..1) reached when the current press started.
    base: f32,
    /// Start of the current press.
    down_at: Option<ClockMs>,
    /// Release time and the fill at release (for the rewind).
    up: Option<(ClockMs, f32)>,
    /// Completed.
    pub confirmed: bool,
}

impl Default for HoldConfirm {
    fn default() -> Self {
        Self::new(1200)
    }
}

impl HoldConfirm {
    /// A hold of `duration_ms`.
    pub fn new(duration_ms: u64) -> Self {
        Self {
            duration_ms: duration_ms.max(1),
            tone: ConfirmTone::Danger,
            base: 0.0,
            down_at: None,
            up: None,
            confirmed: false,
        }
    }
    /// Pointer or Space/Enter down. Resumes from the current fill (re-press during a rewind).
    pub fn press(&mut self, now: ClockMs) {
        if self.confirmed || self.down_at.is_some() {
            return;
        }
        self.base = self.fill(now);
        self.down_at = Some(now);
        self.up = None;
    }
    /// Pointer or key up, or Tab away: an early release starts the rewind.
    pub fn release(&mut self, now: ClockMs) {
        if self.down_at.is_none() {
            return;
        }
        let f = self.fill(now);
        self.down_at = None;
        if f >= 1.0 {
            self.confirmed = true;
        } else {
            self.up = Some((now, f));
        }
    }
    /// Advances the clock: completes the hold once the fill reaches 1. Returns `true` on the
    /// frame it completes.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if !self.confirmed && self.down_at.is_some() && self.fill(now) >= 1.0 {
            self.confirmed = true;
            self.down_at = None;
            return true;
        }
        false
    }
    /// Is a hold in progress.
    pub fn holding(&self) -> bool {
        self.down_at.is_some()
    }
    /// Resets to rest (controlled `confirmed = false`).
    pub fn reset(&mut self) {
        *self = Self {
            tone: self.tone,
            ..Self::new(self.duration_ms)
        };
    }
    /// Fill 0..1 at `now`: linear while held, a smooth rewind after release.
    pub fn fill(&self, now: ClockMs) -> f32 {
        if self.confirmed {
            return 1.0;
        }
        if let Some(d) = self.down_at {
            let t = now.0.saturating_sub(d.0) as f32 / self.duration_ms as f32;
            return (self.base + t).min(1.0);
        }
        if let Some((at, from)) = self.up {
            let t = now.0.saturating_sub(at.0) as f32 / REWIND_MS as f32;
            return from * (1.0 - smooth(t));
        }
        0.0
    }
    /// The hint under the button: "Hold to confirm", "Keep holding…", "Done".
    pub fn hint(&self, now: ClockMs) -> &'static str {
        if self.confirmed {
            "Done"
        } else if self.holding() {
            "Keep holding…"
        } else if self.fill(now) > 0.0 {
            "Hold longer to confirm"
        } else {
            "Hold to confirm"
        }
    }
}

/// How a [`ConfirmMorph`] question is answered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfirmMode {
    /// A press on Confirm.
    #[default]
    Press,
    /// Hold Confirm for the hold duration while it fills.
    Hold,
    /// Confirm, then a countdown that Cancel or Escape can still stop.
    Countdown,
}

impl ConfirmMode {
    /// From a kebab id.
    pub fn parse(s: &str) -> Self {
        match s {
            "hold" => Self::Hold,
            "countdown" => Self::Countdown,
            _ => Self::Press,
        }
    }
}

/// Face of a [`ConfirmMorph`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MorphFace {
    /// The resting button.
    Idle,
    /// The inline question with Confirm and Cancel.
    Confirming,
    /// Countdown mode: seconds left before the action runs.
    Counting(u64),
    /// Spinner with the pending label.
    Pending,
    /// Result with Undo.
    Done,
    /// Error with Retry.
    Error,
    /// Spinner with the undoing label.
    Undoing,
}

impl MorphFace {
    /// Kebab id (`data-face`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Confirming => "confirming",
            Self::Counting(_) => "counting",
            Self::Pending => "pending",
            Self::Done => "done",
            Self::Error => "error",
            Self::Undoing => "undoing",
        }
    }
}

/// Labels of a [`ConfirmMorph`].
#[derive(Clone, Debug, PartialEq)]
pub struct MorphLabels {
    /// Resting label ("Delete files").
    pub label: String,
    /// Question; defaults to the label with a question mark.
    pub prompt: Option<String>,
    /// Confirm ("Delete").
    pub confirm: String,
    /// Cancel.
    pub cancel: String,
    /// Pending ("Deleting").
    pub pending: String,
    /// Done ("Deleted").
    pub done: String,
    /// Error ("Couldn’t finish").
    pub error: String,
    /// Retry.
    pub retry: String,
    /// Undo.
    pub undo: String,
    /// Undoing ("Restoring").
    pub undoing: String,
}

impl MorphLabels {
    /// Defaults around a resting label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.into(),
            prompt: None,
            confirm: "Delete".into(),
            cancel: "Cancel".into(),
            pending: "Deleting".into(),
            done: "Deleted".into(),
            error: "Couldn’t finish".into(),
            retry: "Retry".into(),
            undo: "Undo".into(),
            undoing: "Restoring".into(),
        }
    }
    /// The question.
    pub fn question(&self) -> String {
        self.prompt
            .clone()
            .unwrap_or_else(|| format!("{}?", self.label))
    }
}

/// Confirm morph: idle → confirming → (counting) → pending → done (undo) or error (retry).
#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmMorph {
    /// Labels.
    pub labels: MorphLabels,
    /// Tone.
    pub tone: ConfirmTone,
    /// Mode.
    pub mode: ConfirmMode,
    /// Countdown seconds (3).
    pub countdown: u64,
    /// Unanswered question returns to idle after this, ms (6000).
    pub confirm_timeout_ms: u64,
    /// Undo offered.
    pub undoable: bool,
    /// Hold state in hold mode.
    pub hold: HoldConfirm,
    face: MorphFace,
    since: ClockMs,
}

impl ConfirmMorph {
    /// A morph with a resting label.
    pub fn new(label: &str) -> Self {
        Self {
            labels: MorphLabels::new(label),
            tone: ConfirmTone::Danger,
            mode: ConfirmMode::Press,
            countdown: 3,
            confirm_timeout_ms: 6000,
            undoable: true,
            hold: HoldConfirm::new(1000),
            face: MorphFace::Idle,
            since: ClockMs(0),
        }
    }
    /// Starts in `face` at `at` (controlled state, demos).
    pub fn with_face(mut self, face: MorphFace, at: ClockMs) -> Self {
        self.face = face;
        self.since = at;
        self
    }
    fn go(&mut self, f: MorphFace, now: ClockMs) {
        self.face = f;
        self.since = now;
    }
    /// The face at `now` (countdown seconds and timeouts resolved).
    pub fn face(&self, now: ClockMs) -> MorphFace {
        match self.face {
            MorphFace::Counting(total) => {
                let gone = now.0.saturating_sub(self.since.0) / 1000;
                if gone >= total {
                    MorphFace::Pending
                } else {
                    MorphFace::Counting(total - gone)
                }
            }
            MorphFace::Confirming
                if now.0.saturating_sub(self.since.0) >= self.confirm_timeout_ms =>
            {
                MorphFace::Idle
            }
            f => f,
        }
    }
    /// Advances the clock; returns `true` when the action should start (countdown elapsed or a
    /// hold completed).
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if self.mode == ConfirmMode::Hold
            && self.face == MorphFace::Confirming
            && (self.hold.tick(now) || self.hold.confirmed)
        {
            self.go(MorphFace::Pending, now);
            return true;
        }
        let f = self.face(now);
        if f != self.face && !matches!(f, MorphFace::Counting(_)) {
            let start = f == MorphFace::Pending;
            self.go(f, now);
            if f == MorphFace::Idle {
                self.hold.reset();
            }
            return start;
        }
        false
    }
    /// Resting button pressed: ask.
    pub fn open(&mut self, now: ClockMs) {
        if matches!(self.face, MorphFace::Idle | MorphFace::Done) {
            self.hold.reset();
            self.go(MorphFace::Confirming, now);
        }
    }
    /// Confirm pressed (press mode) or Enter. Returns `true` when the action should start now.
    pub fn confirm(&mut self, now: ClockMs) -> bool {
        match (self.face(now), self.mode) {
            (MorphFace::Confirming, ConfirmMode::Press) | (MorphFace::Error, _) => {
                self.go(MorphFace::Pending, now);
                true
            }
            (MorphFace::Confirming, ConfirmMode::Countdown) => {
                self.go(MorphFace::Counting(self.countdown.max(1)), now);
                false
            }
            _ => false,
        }
    }
    /// Cancel, Escape or an outside press: back to idle (also stops a countdown).
    pub fn cancel(&mut self, now: ClockMs) {
        if matches!(
            self.face(now),
            MorphFace::Confirming | MorphFace::Counting(_) | MorphFace::Error
        ) {
            self.hold.reset();
            self.go(MorphFace::Idle, now);
        }
    }
    /// The action finished.
    pub fn resolve(&mut self, ok: bool, now: ClockMs) {
        if matches!(self.face, MorphFace::Pending) || matches!(self.face(now), MorphFace::Pending) {
            self.go(
                if ok {
                    MorphFace::Done
                } else {
                    MorphFace::Error
                },
                now,
            );
        }
    }
    /// Undo pressed on the result.
    pub fn undo(&mut self, now: ClockMs) {
        if self.face == MorphFace::Done && self.undoable {
            self.go(MorphFace::Undoing, now);
        }
    }
    /// Undo finished.
    pub fn undone(&mut self, now: ClockMs) {
        if self.face == MorphFace::Undoing {
            self.go(MorphFace::Idle, now);
        }
    }
    /// The main text of the face.
    pub fn text(&self, now: ClockMs) -> String {
        let l = &self.labels;
        match self.face(now) {
            MorphFace::Idle => l.label.clone(),
            MorphFace::Confirming => l.question(),
            MorphFace::Counting(s) => format!("{} in {s}", l.pending),
            MorphFace::Pending => l.pending.clone(),
            MorphFace::Done => l.done.clone(),
            MorphFace::Error => l.error.clone(),
            MorphFace::Undoing => l.undoing.clone(),
        }
    }
    /// Live-region text for the face (empty at rest).
    pub fn announcement(&self, now: ClockMs) -> String {
        match self.face(now) {
            MorphFace::Idle => String::new(),
            f => format!("{}: {}", f.id(), self.text(now)),
        }
    }
}

/// Phase of a [`ConfirmPopover`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PopoverPhase {
    /// Button at rest.
    Idle,
    /// The card is open with the question.
    Confirming,
    /// Confirm pressed; `Some(progress)` when reported, `None` for an indeterminate sweep.
    Working(Option<f32>),
    /// The check, in the button's shape.
    Success,
    /// The failure, with Retry.
    Error,
}

impl PopoverPhase {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Confirming => "confirming",
            Self::Working(_) => "working",
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

/// Edge of the trigger the card keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    /// Grows toward the end.
    #[default]
    Start,
    /// Grows both ways.
    Center,
    /// Grows toward the start (buttons at the end of a row).
    End,
}

impl PopoverAlign {
    /// From a kebab id.
    pub fn parse(s: &str) -> Self {
        match s {
            "center" => Self::Center,
            "end" => Self::End,
            _ => Self::Start,
        }
    }
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

/// Shake length after a wrong submit, ms.
pub const SHAKE_MS: u64 = 320;
/// Success face shows this long before folding back, ms.
pub const SUCCESS_MS: u64 = 1400;

/// Confirm popover: the trigger grows into a card that asks, optionally makes you type a phrase,
/// shows progress on Confirm, then a check or an error.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmPopover {
    /// Trigger and confirm tone.
    pub tone: ConfirmTone,
    /// Phrase to type before Confirm unlocks (case and outer spaces ignored).
    pub confirm_text: Option<String>,
    /// Which edge the card keeps.
    pub align: PopoverAlign,
    /// Typed text.
    pub typed: String,
    /// Error message from the last failure.
    pub error: Option<String>,
    phase: PopoverPhase,
    since: ClockMs,
    shake_at: Option<ClockMs>,
}

impl Default for ConfirmPopover {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfirmPopover {
    /// A closed popover.
    pub fn new() -> Self {
        Self {
            tone: ConfirmTone::Danger,
            confirm_text: None,
            align: PopoverAlign::Start,
            typed: String::new(),
            error: None,
            phase: PopoverPhase::Idle,
            since: ClockMs(0),
            shake_at: None,
        }
    }
    /// Sets the type-to-confirm phrase.
    pub fn require(mut self, phrase: &str) -> Self {
        self.confirm_text = Some(phrase.into());
        self
    }
    /// Starts in a phase (controlled, demos).
    pub fn with_phase(mut self, p: PopoverPhase, at: ClockMs) -> Self {
        self.phase = p;
        self.since = at;
        self
    }
    /// Phase at `now` (success folds back to idle after [`SUCCESS_MS`]).
    pub fn phase(&self, now: ClockMs) -> PopoverPhase {
        if self.phase == PopoverPhase::Success && now.0.saturating_sub(self.since.0) >= SUCCESS_MS {
            PopoverPhase::Idle
        } else {
            self.phase
        }
    }
    /// Is the card open.
    pub fn open(&self, now: ClockMs) -> bool {
        matches!(
            self.phase(now),
            PopoverPhase::Confirming | PopoverPhase::Working(_) | PopoverPhase::Error
        )
    }
    /// Trigger pressed.
    pub fn show(&mut self, now: ClockMs) {
        if matches!(self.phase(now), PopoverPhase::Idle) {
            self.typed.clear();
            self.error = None;
            self.phase = PopoverPhase::Confirming;
            self.since = now;
        }
    }
    /// Cancel, Escape or an outside press.
    pub fn dismiss(&mut self, now: ClockMs) {
        if matches!(
            self.phase(now),
            PopoverPhase::Confirming | PopoverPhase::Error
        ) {
            self.phase = PopoverPhase::Idle;
            self.since = now;
        }
    }
    /// Text typed into the phrase field.
    pub fn type_text(&mut self, s: &str) {
        self.typed = s.to_string();
    }
    fn norm(s: &str) -> String {
        s.trim().to_lowercase()
    }
    /// How much of the phrase is matched as a prefix, 0..1 (Confirm fills with it).
    pub fn match_fill(&self) -> f32 {
        let Some(p) = &self.confirm_text else {
            return 1.0;
        };
        let (p, t) = (Self::norm(p), Self::norm(&self.typed));
        if p.is_empty() {
            return 1.0;
        }
        if !p.starts_with(&t) {
            return 0.0;
        }
        t.chars().count() as f32 / p.chars().count() as f32
    }
    /// Confirm is unlocked.
    pub fn unlocked(&self) -> bool {
        self.match_fill() >= 1.0
    }
    /// Confirm pressed: starts the work when unlocked, otherwise shakes. Returns `true` when the
    /// action should run.
    pub fn submit(&mut self, now: ClockMs) -> bool {
        match self.phase(now) {
            PopoverPhase::Confirming | PopoverPhase::Error if self.unlocked() => {
                self.phase = PopoverPhase::Working(None);
                self.since = now;
                self.error = None;
                true
            }
            PopoverPhase::Confirming => {
                self.shake_at = Some(now);
                false
            }
            _ => false,
        }
    }
    /// Reported progress 0..1.
    pub fn progress(&mut self, p: f32) {
        if let PopoverPhase::Working(_) = self.phase {
            self.phase = PopoverPhase::Working(Some(p.clamp(0.0, 1.0)));
        }
    }
    /// The work finished: `Err(message)` shows the failure.
    pub fn finish(&mut self, result: Result<(), String>, now: ClockMs) {
        if let PopoverPhase::Working(_) = self.phase {
            match result {
                Ok(()) => self.phase = PopoverPhase::Success,
                Err(e) => {
                    self.error = Some(e);
                    self.phase = PopoverPhase::Error;
                }
            }
            self.since = now;
        }
    }
    /// Shake offset in px at `now` (a damped wobble after a wrong submit).
    pub fn shake(&self, now: ClockMs) -> f32 {
        let Some(at) = self.shake_at else { return 0.0 };
        let t = now.0.saturating_sub(at.0);
        if t >= SHAKE_MS {
            return 0.0;
        }
        let k = t as f32 / SHAKE_MS as f32;
        (k * std::f32::consts::PI * 6.0).sin() * 6.0 * (1.0 - k)
    }
    /// Sweep position 0..1 for an indeterminate bar (1.2 s loop).
    pub fn sweep(&self, now: ClockMs) -> f32 {
        (now.0.saturating_sub(self.since.0) % 1200) as f32 / 1200.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_fills_rewinds_and_resumes() {
        let mut h = HoldConfirm::new(1000);
        h.press(ClockMs(0));
        assert!((h.fill(ClockMs(500)) - 0.5).abs() < 1e-3);
        h.release(ClockMs(500));
        assert!(h.fill(ClockMs(620)) < 0.5);
        assert_eq!(h.fill(ClockMs(800)), 0.0);
        h.press(ClockMs(560));
        let f = h.fill(ClockMs(560));
        assert!(f > 0.0 && f < 0.5);
        assert!(!h.tick(ClockMs(900)));
        assert!(h.tick(ClockMs(1600)));
        assert!(h.confirmed);
        assert_eq!(h.hint(ClockMs(1700)), "Done");
    }

    #[test]
    fn morph_press_and_undo() {
        let mut m = ConfirmMorph::new("Delete files");
        m.open(ClockMs(0));
        assert_eq!(m.text(ClockMs(10)), "Delete files?");
        assert!(m.confirm(ClockMs(100)));
        assert_eq!(m.face(ClockMs(150)), MorphFace::Pending);
        m.resolve(true, ClockMs(900));
        m.undo(ClockMs(1000));
        assert_eq!(m.face(ClockMs(1000)), MorphFace::Undoing);
        m.undone(ClockMs(1500));
        assert_eq!(m.face(ClockMs(1500)), MorphFace::Idle);
    }

    #[test]
    fn morph_countdown_and_timeout() {
        let mut m = ConfirmMorph::new("Delete");
        m.mode = ConfirmMode::Countdown;
        m.open(ClockMs(0));
        assert!(!m.confirm(ClockMs(10)));
        assert_eq!(m.face(ClockMs(1100)), MorphFace::Counting(2));
        assert!(m.tick(ClockMs(3100)));
        assert_eq!(m.face(ClockMs(3100)), MorphFace::Pending);
        let mut t = ConfirmMorph::new("Delete");
        t.open(ClockMs(0));
        t.tick(ClockMs(7000));
        assert_eq!(t.face(ClockMs(7000)), MorphFace::Idle);
    }

    #[test]
    fn popover_phrase_and_shake() {
        let mut p = ConfirmPopover::new().require("acme-web");
        p.show(ClockMs(0));
        p.type_text("  ACME");
        assert!(p.match_fill() > 0.4 && !p.unlocked());
        assert!(!p.submit(ClockMs(100)));
        assert!(p.shake(ClockMs(150)).abs() > 0.0);
        p.type_text("acme-web ");
        assert!(p.submit(ClockMs(400)));
        p.progress(0.5);
        assert_eq!(p.phase(ClockMs(500)), PopoverPhase::Working(Some(0.5)));
        p.finish(Err("Forbidden".into()), ClockMs(600));
        assert_eq!(p.phase(ClockMs(600)), PopoverPhase::Error);
        assert!(p.submit(ClockMs(700)));
        p.finish(Ok(()), ClockMs(800));
        assert_eq!(p.phase(ClockMs(900)), PopoverPhase::Success);
        assert_eq!(p.phase(ClockMs(2300)), PopoverPhase::Idle);
    }
}
