//! Split confirm: a pill button that confirms by splitting. Pressed, the pill stretches with a
//! small overshoot, then a round check bubble pinches off its end and settles beside it while the
//! label turns to the past tense ("Approve" → "Approved"). The bubble is the Undo for a few
//! seconds. If the action fails, the bubble carries a cross, the pill shakes once and offers
//! Try again.

/// Label pairs: (verb, done).
pub const LABELS: [(&str, &str); 3] = [
    ("Approve", "Approved"),
    ("Send", "Sent"),
    ("Merge", "Merged"),
];
/// Ms of the stretch.
pub const STRETCH_MS: u64 = 260;
/// Ms of the pinch-off.
pub const SPLIT_MS: u64 = 420;
/// Ms the bubble offers Undo.
pub const UNDO_MS: u64 = 4_000;

/// Where the button is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Waiting.
    Idle,
    /// Stretching (0..1).
    Stretch(f64),
    /// Bubble pinching off (0..1).
    Split(f64),
    /// Confirmed, bubble beside.
    Done,
    /// Failed, cross bubble.
    Failed,
}

/// The button.
#[derive(Clone, Debug, PartialEq)]
pub struct Split {
    /// Label pair index.
    pub label: usize,
    /// Pressed at.
    pub pressed: Option<u64>,
    /// The action fails.
    pub fail: bool,
}

impl Split {
    /// A button.
    pub fn new(label: usize, fail: bool) -> Self {
        Self {
            label: label.min(LABELS.len() - 1),
            pressed: None,
            fail,
        }
    }

    /// The phase.
    pub fn phase(&self, now: u64) -> Phase {
        let Some(p) = self.pressed else {
            return Phase::Idle;
        };
        let e = now.saturating_sub(p);
        if e < STRETCH_MS {
            Phase::Stretch(e as f64 / STRETCH_MS as f64)
        } else if e < STRETCH_MS + SPLIT_MS {
            Phase::Split((e - STRETCH_MS) as f64 / SPLIT_MS as f64)
        } else if self.fail {
            Phase::Failed
        } else {
            Phase::Done
        }
    }

    /// Horizontal stretch of the pill (1 = rest), springy.
    pub fn scale(&self, now: u64) -> f64 {
        match self.phase(now) {
            Phase::Stretch(t) => 1.0 + 0.14 * (t * std::f64::consts::PI).sin() + 0.06 * t,
            Phase::Split(t) => 1.06 - 0.06 * t,
            _ => 1.0,
        }
    }

    /// Bubble offset from the pill's end (0 = attached, 1 = settled) and its size 0..1.
    pub fn bubble(&self, now: u64) -> Option<(f64, f64)> {
        match self.phase(now) {
            Phase::Split(t) => {
                let e = 1.0 - (1.0 - t).powi(3);
                Some((e, 0.5 + 0.5 * e))
            }
            Phase::Done | Phase::Failed => Some((1.0, 1.0)),
            _ => None,
        }
    }

    /// The label showing.
    pub fn text(&self, now: u64) -> &'static str {
        match self.phase(now) {
            Phase::Done => LABELS[self.label].1,
            Phase::Failed => "Didn't go through",
            _ => LABELS[self.label].0,
        }
    }

    /// Undo still offered.
    pub fn can_undo(&self, now: u64) -> bool {
        self.phase(now) == Phase::Done
            && self
                .pressed
                .is_some_and(|p| now.saturating_sub(p) < STRETCH_MS + SPLIT_MS + UNDO_MS)
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "sp-go" if self.phase(now) == Phase::Idle => {
                self.pressed = Some(now);
                Some(format!("{}…", LABELS[self.label].0))
            }
            "sp-undo" if self.can_undo(now) => {
                self.pressed = None;
                Some("Undone".into())
            }
            "sp-retry" if self.phase(now) == Phase::Failed => {
                self.fail = false;
                self.pressed = Some(now);
                Some("Trying again".into())
            }
            "sp-reset" => {
                self.pressed = None;
                Some("Reset".into())
            }
            _ => None,
        }
    }

    /// Enter/Space confirm or retry; Backspace or Cmd+Z (as `undo`) undo; Escape resets.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        match k {
            "Enter" | "Space" => match self.phase(now) {
                Phase::Failed => self.press("sp-retry", now),
                _ => self.press("sp-go", now),
            },
            "Backspace" | "undo" => self.press("sp-undo", now),
            "Escape" => self.press("sp-reset", now),
            _ => None,
        }
    }
}
