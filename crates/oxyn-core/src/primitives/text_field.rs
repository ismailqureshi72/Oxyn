//! Text field (`text-field.md`): label, hint and error in one slot, prefix and suffix, clear,
//! show-password and copy buttons, a character count. Oxyn plus: validation timing (on blur / as
//! you type / on submit), a soft limit on the counter, and a secret field whose reveal hides
//! itself again after a timer.

use std::time::Duration;

use crate::ClockMs;

/// Input type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Text.
    #[default]
    Text,
    /// Password (gets the show / hide button).
    Password,
    /// Email.
    Email,
    /// Search (Esc clears).
    Search,
    /// Number.
    Number,
    /// URL.
    Url,
}

impl FieldType {
    /// `type` attribute.
    pub fn attr(self, revealed: bool) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password if revealed => "text",
            Self::Password => "password",
            Self::Email => "email",
            Self::Search => "search",
            Self::Number => "number",
            Self::Url => "url",
        }
    }
}

/// When validation runs (Oxyn plus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Validate {
    /// When the field loses focus, then live once it has shown an error (default).
    #[default]
    OnBlur,
    /// On every change (after the first character).
    OnType,
    /// Only when the form submits.
    OnSubmit,
}

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// The error nudge (one shake).
    pub const NUDGE: Duration = oxyn_tokens::motion::dur::error::NUDGE;
    /// The message slides in out of a short blur, in place of the hint.
    pub const MESSAGE: Duration = oxyn_tokens::motion::dur::BASE;
    /// Revealed password characters come out of a short blur.
    pub const REVEAL: Duration = oxyn_tokens::motion::dur::BASE;
    /// The copy icon holds its check this long.
    pub const COPIED: Duration = oxyn_tokens::motion::dur::copy::CONFIRM;
    /// Border colour change on hover / focus.
    pub const BORDER: Duration = oxyn_tokens::motion::dur::FAST;
    /// Oxyn plus: default auto-hide for a revealed secret.
    pub const AUTO_HIDE: Duration = Duration::from_secs(10);
}

/// Counter state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountLevel {
    /// Under the soft limit.
    Ok,
    /// Past the soft limit (warning tone), still allowed.
    Near,
    /// Past the hard limit (error tone).
    Over,
}

/// The field's state (value lives here so hosts can drive it from either renderer).
#[derive(Clone, Debug, PartialEq)]
pub struct TextField {
    /// Value.
    pub value: String,
    /// Type.
    pub kind: FieldType,
    /// Hard limit (`maxlength`) for the counter.
    pub max: Option<usize>,
    /// Oxyn plus: soft limit (warning before the hard limit).
    pub soft: Option<usize>,
    /// Required.
    pub required: bool,
    /// Validation timing.
    pub validate: Validate,
    /// Error from the app (overrides the built-in checks).
    pub error: Option<String>,
    /// Disabled.
    pub disabled: bool,
    /// Read-only.
    pub readonly: bool,
    /// Password shown.
    pub revealed: bool,
    /// Oxyn plus: hide a revealed secret after this long.
    pub auto_hide: Option<Duration>,
    /// Focused.
    pub focused: bool,
    touched: bool,
    shown_error: Option<String>,
    error_at: Option<ClockMs>,
    revealed_at: Option<ClockMs>,
    copied_at: Option<ClockMs>,
}

impl TextField {
    /// A field.
    pub fn new(kind: FieldType) -> Self {
        Self {
            value: String::new(),
            kind,
            max: None,
            soft: None,
            required: false,
            validate: Validate::OnBlur,
            error: None,
            disabled: false,
            readonly: false,
            revealed: false,
            auto_hide: None,
            focused: false,
            touched: false,
            shown_error: None,
            error_at: None,
            revealed_at: None,
            copied_at: None,
        }
    }
    fn check(&self) -> Option<String> {
        if let Some(e) = &self.error {
            return Some(e.clone());
        }
        let v = self.value.trim();
        if self.required && v.is_empty() {
            return Some("Required".into());
        }
        if let Some(m) = self.max
            && self.value.chars().count() > m
        {
            return Some(format!("Keep it under {m} characters"));
        }
        match self.kind {
            FieldType::Email if !v.is_empty() => {
                let ok = v.split_once('@').is_some_and(|(a, b)| {
                    !a.is_empty() && b.contains('.') && !b.starts_with('.') && !b.ends_with('.')
                });
                (!ok).then(|| "Enter an email like name@example.com".into())
            }
            FieldType::Url if !v.is_empty() => (!(v.starts_with("http://")
                || v.starts_with("https://")))
            .then(|| "Enter a URL starting with https://".into()),
            FieldType::Number if !v.is_empty() => {
                v.parse::<f64>().is_err().then(|| "Enter a number".into())
            }
            _ => None,
        }
    }
    fn refresh(&mut self, now: ClockMs) {
        let e = self.check();
        if e.is_some() && e != self.shown_error {
            self.error_at = Some(now);
        }
        self.shown_error = e;
    }
    /// Typing.
    pub fn input(&mut self, v: &str, now: ClockMs) {
        if self.disabled || self.readonly {
            return;
        }
        self.value = v.to_string();
        let live = match self.validate {
            Validate::OnType => !self.value.is_empty() || self.touched,
            // After a blur has shown an error, keep it live so it clears as soon as it is fixed.
            Validate::OnBlur => self.shown_error.is_some(),
            Validate::OnSubmit => false,
        };
        if live {
            self.refresh(now);
        }
    }
    /// Focus in.
    pub fn focus(&mut self) {
        self.focused = true;
    }
    /// Focus out.
    pub fn blur(&mut self, now: ClockMs) {
        self.focused = false;
        self.touched = true;
        if self.validate != Validate::OnSubmit {
            self.refresh(now);
        }
    }
    /// The form submits: always validates. Returns the error, if any.
    pub fn submit(&mut self, now: ClockMs) -> Option<String> {
        self.touched = true;
        self.refresh(now);
        self.shown_error.clone()
    }
    /// The message under the field: the error, in place of the hint.
    pub fn error(&self) -> Option<&str> {
        self.shown_error.as_deref()
    }
    /// The nudge offset in px at `now` (one damped shake after a new error).
    pub fn nudge(&self, now: ClockMs, reduced: bool) -> f32 {
        let Some(t) = self.error_at else { return 0.0 };
        if reduced {
            return 0.0;
        }
        let p = now.since(t).as_secs_f32() / timing::NUDGE.as_secs_f32();
        if p >= 1.0 {
            0.0
        } else {
            4.0 * (p * std::f32::consts::TAU * 2.0).sin() * (1.0 - p)
        }
    }
    /// The clear button shows when there is a value and the field is editable.
    pub fn can_clear(&self) -> bool {
        !self.value.is_empty() && !self.disabled && !self.readonly
    }
    /// Clear (button, or Esc in the field). Focus stays in the field.
    pub fn clear(&mut self, now: ClockMs) -> bool {
        if !self.can_clear() {
            return false;
        }
        self.value.clear();
        if self.shown_error.is_some() {
            self.refresh(now);
        }
        true
    }
    /// Show / hide the password.
    pub fn toggle_reveal(&mut self, now: ClockMs) {
        self.revealed = !self.revealed;
        self.revealed_at = self.revealed.then_some(now);
    }
    /// Re-hides a revealed secret after `auto_hide`. Returns true when it hid.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if let (true, Some(t), Some(d)) = (self.revealed, self.revealed_at, self.auto_hide)
            && now.since(t) >= d
        {
            self.revealed = false;
            self.revealed_at = None;
            return true;
        }
        false
    }
    /// Seconds left before a revealed secret hides (for the ring around the eye).
    pub fn reveal_left(&self, now: ClockMs) -> Option<f32> {
        let (t, d) = (self.revealed_at?, self.auto_hide?);
        Some((1.0 - now.since(t).as_secs_f32() / d.as_secs_f32()).max(0.0))
    }
    /// Copy pressed; the icon morphs to a check that draws itself.
    pub fn copied(&mut self, now: ClockMs) -> String {
        self.copied_at = Some(now);
        self.value.clone()
    }
    /// The copy icon is a check right now.
    pub fn showing_copied(&self, now: ClockMs) -> bool {
        self.copied_at
            .is_some_and(|t| now.since(t) < timing::COPIED)
    }
    /// Counter text and level ("12 / 280").
    pub fn counter(&self) -> Option<(String, CountLevel)> {
        let n = self.value.chars().count();
        let max = self.max?;
        let level = if n > max {
            CountLevel::Over
        } else if self.soft.is_some_and(|s| n > s) {
            CountLevel::Near
        } else {
            CountLevel::Ok
        };
        Some((format!("{n} / {max}"), level))
    }
    /// `aria-invalid`.
    pub fn invalid(&self) -> bool {
        self.shown_error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_validation_then_live() {
        let mut f = TextField::new(FieldType::Email);
        f.input("ada@", ClockMs(0));
        assert_eq!(f.error(), None, "quiet while typing the first time");
        f.blur(ClockMs(100));
        assert!(f.error().unwrap().starts_with("Enter an email"));
        assert!(f.nudge(ClockMs(150), false).abs() > 0.0);
        assert_eq!(f.nudge(ClockMs(1_000), false), 0.0);
        f.input("ada@example.com", ClockMs(200));
        assert_eq!(
            f.error(),
            None,
            "live once shown: clears as soon as it is fixed"
        );
    }

    #[test]
    fn on_type_and_submit() {
        let mut f = TextField::new(FieldType::Number);
        f.validate = Validate::OnType;
        f.input("1x", ClockMs(0));
        assert_eq!(f.error(), Some("Enter a number"));
        let mut s = TextField::new(FieldType::Text);
        s.validate = Validate::OnSubmit;
        s.required = true;
        s.blur(ClockMs(0));
        assert_eq!(s.error(), None);
        assert_eq!(s.submit(ClockMs(1)).as_deref(), Some("Required"));
        assert!(s.invalid());
    }

    #[test]
    fn counter_clear_copy_reveal() {
        let mut f = TextField::new(FieldType::Password);
        f.max = Some(10);
        f.soft = Some(8);
        f.input("hunter2hunt", ClockMs(0));
        assert_eq!(f.counter(), Some(("11 / 10".into(), CountLevel::Over)));
        f.input("hunter2hu", ClockMs(0));
        assert_eq!(f.counter().unwrap().1, CountLevel::Near);
        assert_eq!(f.kind.attr(f.revealed), "password");
        f.auto_hide = Some(Duration::from_secs(5));
        f.toggle_reveal(ClockMs(0));
        assert_eq!(f.kind.attr(f.revealed), "text");
        assert!(!f.tick(ClockMs(4_000)));
        assert!(f.reveal_left(ClockMs(2_500)).unwrap() - 0.5 < 1e-3);
        assert!(f.tick(ClockMs(5_000)) && !f.revealed);
        assert_eq!(f.copied(ClockMs(6_000)), "hunter2hu");
        assert!(f.showing_copied(ClockMs(7_000)) && !f.showing_copied(ClockMs(8_000)));
        assert!(f.clear(ClockMs(9_000)) && f.value.is_empty());
        assert!(!f.clear(ClockMs(9_001)));
        f.disabled = true;
        f.input("x", ClockMs(9_002));
        assert!(f.value.is_empty());
    }
}
