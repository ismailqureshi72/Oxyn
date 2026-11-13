//! OTP input (`otp-input.md`): the one-time-code field. One focus ring glides from box to box,
//! paste and SMS autofill fill every box one after another, verifying shimmers and locks it, an
//! error shakes, shows the message and clears the code for the next try, success morphs into a
//! `Verified` pill, and a resend countdown becomes a Resend button.
//!
//! Behaviour follows the affectUI OTP Input docs (rebuilt, no source used). Oxyn plus: lengths 4–8
//! with alphanumeric codes, a masked display, an attempt counter that locks after the last try.

use crate::ClockMs;

/// Verification status (affectUI `status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OtpStatus {
    /// Typing.
    #[default]
    Idle,
    /// Checking the code: a light passes over the boxes, input is locked.
    Verifying,
    /// Wrong code: shake, red edges, the message, then the code clears.
    Error,
    /// Right code: the boxes slide together into the pill.
    Success,
}

impl OtpStatus {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "verifying" => Self::Verifying,
            "error" => Self::Error,
            "success" => Self::Success,
            _ => Self::Idle,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Verifying => "verifying",
            Self::Error => "error",
            Self::Success => "success",
        }
    }
}

/// OTP timings.
pub mod timing {
    /// Shake length; the code clears after it (`clear_on_error`).
    pub const SHAKE_MS: u64 = 420;
    /// Pasted digits land one after another.
    pub const FILL_STAGGER_MS: u64 = 40;
    /// The boxes slide together and the frame morphs into the pill.
    pub const MORPH_MS: u64 = 360;
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtpKey {
    /// Backspace.
    Backspace,
    /// ←.
    Left,
    /// →.
    Right,
    /// Home.
    Home,
    /// End.
    End,
}

/// The OTP field.
#[derive(Clone, Debug, PartialEq)]
pub struct Otp {
    /// Number of boxes (clamped 4–8).
    pub length: usize,
    /// Digits only (else letters too, shown in capitals).
    pub numeric: bool,
    /// The characters per box.
    pub cells: Vec<Option<char>>,
    /// The active box (the ring).
    pub active: usize,
    /// Status.
    pub status: OtpStatus,
    /// When the status last changed.
    pub status_at: ClockMs,
    /// Clear the code shortly after an error.
    pub clear_on_error: bool,
    /// Seconds until resend is allowed (`None` hides resend).
    pub resend_in: Option<u32>,
    /// When the resend countdown started.
    pub resend_from: ClockMs,
    /// A dash after this many boxes.
    pub separator: Option<usize>,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: show dots instead of the characters.
    pub masked: bool,
    /// Oxyn plus: tries left (`None` = unlimited); at zero the field locks.
    pub attempts: Option<u32>,
    /// When each box was filled (pop-in / paste stagger).
    pub filled_at: Vec<ClockMs>,
    completed: bool,
}

impl Otp {
    /// A field with `length` boxes.
    pub fn new(length: usize) -> Self {
        let length = length.clamp(4, 8);
        Self {
            length,
            numeric: true,
            cells: vec![None; length],
            active: 0,
            status: OtpStatus::Idle,
            status_at: ClockMs(0),
            clear_on_error: true,
            resend_in: Some(30),
            resend_from: ClockMs(0),
            separator: None,
            disabled: false,
            masked: false,
            attempts: None,
            filled_at: vec![ClockMs(0); length],
            completed: false,
        }
    }
    /// The code typed so far.
    pub fn code(&self) -> String {
        self.cells.iter().flatten().collect()
    }
    /// Every box is filled.
    pub fn full(&self) -> bool {
        self.cells.iter().all(Option::is_some)
    }
    /// Input is ignored.
    pub fn locked(&self) -> bool {
        self.disabled
            || matches!(self.status, OtpStatus::Verifying | OtpStatus::Success)
            || self.attempts == Some(0)
    }
    fn accept(&self, ch: char) -> Option<char> {
        if self.numeric {
            ch.is_ascii_digit().then_some(ch)
        } else {
            ch.is_ascii_alphanumeric().then(|| ch.to_ascii_uppercase())
        }
    }
    fn first_empty(&self) -> usize {
        self.cells
            .iter()
            .position(Option::is_none)
            .unwrap_or(self.length - 1)
    }
    fn reset_status(&mut self, now: ClockMs) {
        if self.status == OtpStatus::Error {
            self.status = OtpStatus::Idle;
            self.status_at = now;
        }
    }
    fn complete(&mut self) -> Option<String> {
        if self.full() && !self.completed {
            self.completed = true;
            return Some(self.code());
        }
        None
    }
    /// A typed character: fills the active box and moves on; typing on a filled box replaces it.
    /// Returns the code once the last box is filled (`on_complete`).
    pub fn type_char(&mut self, ch: char, now: ClockMs) -> Option<String> {
        if self.locked() {
            return None;
        }
        let c = self.accept(ch)?;
        self.reset_status(now);
        let i = self.active.min(self.length - 1);
        self.cells[i] = Some(c);
        self.filled_at[i] = now;
        self.completed = self.completed && self.full();
        if i + 1 < self.length {
            self.active = i + 1;
        }
        self.complete()
    }
    /// Paste / autofill: spaces and dashes are ignored; boxes fill from the first one, one after
    /// another. Returns the code when it filled every box.
    pub fn paste(&mut self, text: &str, now: ClockMs) -> Option<String> {
        if self.locked() {
            return None;
        }
        let chars: Vec<char> = text.chars().filter_map(|c| self.accept(c)).collect();
        if chars.is_empty() {
            return None;
        }
        self.reset_status(now);
        let start = if chars.len() >= self.length {
            0
        } else {
            self.active
        };
        self.completed = false;
        for (k, c) in chars.into_iter().enumerate() {
            let i = start + k;
            if i >= self.length {
                break;
            }
            self.cells[i] = Some(c);
            self.filled_at[i] = ClockMs(now.0 + k as u64 * timing::FILL_STAGGER_MS);
        }
        self.active = self.first_empty();
        self.complete()
    }
    /// Keyboard.
    pub fn key(&mut self, k: OtpKey, now: ClockMs) {
        if self.locked() {
            return;
        }
        let last_reachable = self.first_empty();
        match k {
            OtpKey::Backspace => {
                self.reset_status(now);
                self.completed = false;
                let i = self.active;
                if self.cells[i].is_some() {
                    self.cells[i] = None;
                } else if i > 0 {
                    self.cells[i - 1] = None;
                    self.active = i - 1;
                }
            }
            OtpKey::Left => self.active = self.active.saturating_sub(1),
            OtpKey::Right => self.active = (self.active + 1).min(last_reachable),
            OtpKey::Home => self.active = 0,
            OtpKey::End => self.active = last_reachable,
        }
    }
    /// Clicking box `i` focuses it, but never past the next empty box.
    pub fn focus(&mut self, i: usize) {
        self.active = i.min(self.first_empty());
    }
    /// Sets the status (the host verifies). An error spends a try.
    pub fn set_status(&mut self, s: OtpStatus, now: ClockMs) {
        if s == self.status {
            return;
        }
        if s == OtpStatus::Error
            && let Some(a) = self.attempts.as_mut()
        {
            *a = a.saturating_sub(1);
        }
        self.status = s;
        self.status_at = now;
    }
    /// Advances time: after the shake an error clears the code for the next try. Returns `true`
    /// when it cleared.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if self.status == OtpStatus::Error
            && self.clear_on_error
            && self.full()
            && now.since(self.status_at).as_millis() >= u128::from(timing::SHAKE_MS)
        {
            self.cells = vec![None; self.length];
            self.active = 0;
            self.completed = false;
            return true;
        }
        false
    }
    /// Horizontal shake offset in px at `now` (error), decaying over [`timing::SHAKE_MS`].
    pub fn shake(&self, now: ClockMs, reduced: bool) -> f32 {
        if reduced || self.status != OtpStatus::Error {
            return 0.0;
        }
        let t = now.since(self.status_at).as_millis() as f32;
        let d = timing::SHAKE_MS as f32;
        if t >= d {
            return 0.0;
        }
        let k = 1.0 - t / d;
        (t / d * std::f32::consts::TAU * 3.0).sin() * 6.0 * k
    }
    /// Seconds left before Resend appears (`None` when resend is hidden).
    pub fn resend_left(&self, now: ClockMs) -> Option<u32> {
        let total = self.resend_in?;
        let gone = (now.since(self.resend_from).as_secs()) as u32;
        Some(total.saturating_sub(gone))
    }
    /// Resend pressed: restarts the countdown. Returns `false` when it isn't available yet.
    pub fn resend(&mut self, now: ClockMs) -> bool {
        if self.resend_left(now) != Some(0) {
            return false;
        }
        self.resend_from = now;
        true
    }
    /// The shown glyph for box `i`.
    pub fn glyph(&self, i: usize) -> Option<char> {
        let c = self.cells.get(i).copied().flatten()?;
        Some(if self.masked { '•' } else { c })
    }
    /// Where the separator dash goes (after this many boxes); `true` means the middle.
    pub fn separator_at(&self) -> Option<usize> {
        self.separator
            .map(|n| if n == 0 { self.length / 2 } else { n })
            .filter(|&n| n > 0 && n < self.length)
    }
    /// The accessible name: `Verification code, 6 digits`.
    pub fn label(&self, base: &str) -> String {
        format!(
            "{base}, {} {}",
            self.length,
            if self.numeric { "digits" } else { "characters" }
        )
    }
    /// The live-region line.
    pub fn announce(&self, error: &str, success: &str) -> String {
        match self.status {
            OtpStatus::Verifying => "Verifying code".into(),
            OtpStatus::Error => match self.attempts {
                Some(0) => format!("{error} No tries left."),
                _ => error.into(),
            },
            OtpStatus::Success => success.into(),
            OtpStatus::Idle => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_completes_once() {
        let mut o = Otp::new(6);
        assert_eq!(
            o.type_char('x', ClockMs(0)),
            None,
            "numeric ignores letters"
        );
        for (k, c) in "12345".chars().enumerate() {
            assert_eq!(o.type_char(c, ClockMs(k as u64)), None);
        }
        assert_eq!(o.type_char('6', ClockMs(9)).as_deref(), Some("123456"));
        assert_eq!(o.active, 5);
        assert_eq!(
            o.type_char('7', ClockMs(10)),
            None,
            "complete fires once per fill"
        );
        o.key(OtpKey::Backspace, ClockMs(11));
        assert_eq!(o.code(), "12345");
        o.key(OtpKey::Backspace, ClockMs(12));
        assert_eq!(o.code(), "1234");
        assert_eq!(o.active, 4);
        o.key(OtpKey::Home, ClockMs(13));
        assert_eq!(o.active, 0);
        o.key(OtpKey::End, ClockMs(14));
        assert_eq!(o.active, 4, "End goes to the next empty box");
        o.focus(5);
        assert_eq!(o.active, 4, "can't jump past the next empty box");
    }

    #[test]
    fn paste_staggers_and_ignores_separators() {
        let mut o = Otp::new(6);
        assert_eq!(
            o.paste("482 - 913", ClockMs(100)).as_deref(),
            Some("482913")
        );
        assert_eq!(o.filled_at[3], ClockMs(220));
        let mut a = Otp::new(5);
        a.numeric = false;
        a.type_char('k', ClockMs(0));
        assert_eq!(a.code(), "K", "letters in capitals");
        a.masked = true;
        assert_eq!(a.glyph(0), Some('•'));
    }

    #[test]
    fn error_clears_and_counts_tries() {
        let mut o = Otp::new(4);
        o.attempts = Some(2);
        o.paste("1111", ClockMs(0));
        o.set_status(OtpStatus::Verifying, ClockMs(10));
        assert!(o.locked());
        assert_eq!(o.type_char('2', ClockMs(20)), None);
        o.set_status(OtpStatus::Error, ClockMs(100));
        assert_eq!(o.attempts, Some(1));
        assert!(o.shake(ClockMs(150), false).abs() > 0.0);
        assert_eq!(o.shake(ClockMs(150), true), 0.0);
        assert!(!o.tick(ClockMs(300)));
        assert!(o.tick(ClockMs(600)));
        assert_eq!(o.code(), "");
        o.type_char('5', ClockMs(700));
        assert_eq!(o.status, OtpStatus::Idle, "typing again clears the error");
        o.paste("5555", ClockMs(800));
        o.set_status(OtpStatus::Error, ClockMs(900));
        assert!(o.locked(), "no tries left");
        assert!(
            o.announce("That code didn't work.", "Verified")
                .contains("No tries left")
        );
    }

    #[test]
    fn resend_countdown_and_labels() {
        let mut o = Otp::new(6);
        o.resend_from = ClockMs(0);
        assert_eq!(o.resend_left(ClockMs(12_000)), Some(18));
        assert!(!o.resend(ClockMs(12_000)));
        assert!(o.resend(ClockMs(30_000)));
        assert_eq!(o.resend_left(ClockMs(31_000)), Some(29));
        o.separator = Some(0);
        assert_eq!(o.separator_at(), Some(3));
        assert_eq!(o.label("Verification code"), "Verification code, 6 digits");
        assert_eq!(Otp::new(12).length, 8);
    }
}
