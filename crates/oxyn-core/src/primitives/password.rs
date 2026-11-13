//! Password field and password strength (`password-field.md`, `password-strength.md`).
//!
//! - The **field** (Arc UI Password field) reuses the text field's reveal; this module adds the
//!   slash that draws across the eye, Caps Lock detection and the reveal timer.
//! - The **strength meter** (Arc UI Password strength) scores a new password against rules: a
//!   four-segment meter that fills in a staggered wave, a checklist with remaining counts, and a
//!   word that morphs up or down (`Too short`, `Weak`, `Fair`, `Good`, `Strong`).
//!
//! Rebuilt from the public docs, no source used. Oxyn plus: policy presets (Basic, NIST 800-63B,
//! Enterprise), a common / breached password check, and an entropy estimate in bits.

/// What a rule checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleKind {
    /// At least this many characters (code points).
    MinLen(usize),
    /// An uppercase letter.
    Upper,
    /// A lowercase letter.
    Lower,
    /// A digit.
    Digit,
    /// A symbol.
    Symbol,
    /// No spaces at the start or end.
    NoEdgeSpaces,
    /// Not on the common-password list.
    NotCommon,
}

/// One rule in the checklist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Id.
    pub id: String,
    /// Label (`At least 12 characters`).
    pub label: String,
    /// Check.
    pub kind: RuleKind,
}

impl Rule {
    /// A rule.
    pub fn new(kind: RuleKind) -> Self {
        let (id, label) = match kind {
            RuleKind::MinLen(n) => ("length".to_string(), format!("At least {n} characters")),
            RuleKind::Upper => ("upper".into(), "An uppercase letter".into()),
            RuleKind::Lower => ("lower".into(), "A lowercase letter".into()),
            RuleKind::Digit => ("digit".into(), "A number".into()),
            RuleKind::Symbol => ("symbol".into(), "A symbol".into()),
            RuleKind::NoEdgeSpaces => ("spaces".into(), "No spaces at either end".into()),
            RuleKind::NotCommon => ("common".into(), "Not a common password".into()),
        };
        Self { id, label, kind }
    }
    /// Met by `pw`.
    pub fn test(&self, pw: &str) -> bool {
        match self.kind {
            RuleKind::MinLen(n) => pw.chars().count() >= n,
            RuleKind::Upper => pw.chars().any(char::is_uppercase),
            RuleKind::Lower => pw.chars().any(char::is_lowercase),
            RuleKind::Digit => pw.chars().any(|c| c.is_ascii_digit()),
            RuleKind::Symbol => pw
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace()),
            RuleKind::NoEdgeSpaces => !pw.is_empty() && pw.trim() == pw,
            RuleKind::NotCommon => !pw.is_empty() && !is_common(pw),
        }
    }
    /// What is still missing (`3 more characters`), for rules that count.
    pub fn remaining(&self, pw: &str) -> Option<usize> {
        match self.kind {
            RuleKind::MinLen(n) => {
                let c = pw.chars().count();
                (c < n).then(|| n - c)
            }
            _ => None,
        }
    }
}

/// Oxyn plus: rule presets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Policy {
    /// Arc's default: 8+ characters, upper, lower, number, symbol.
    #[default]
    Basic,
    /// NIST SP 800-63B: length and not common, no composition rules.
    Nist,
    /// 12+ characters, every class, not common.
    Enterprise,
}

impl Policy {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "nist" => Self::Nist,
            "enterprise" => Self::Enterprise,
            _ => Self::Basic,
        }
    }
    /// The rules.
    pub fn rules(self) -> Vec<Rule> {
        use RuleKind::*;
        let kinds: &[RuleKind] = match self {
            Self::Basic => &[MinLen(8), Upper, Lower, Digit, Symbol],
            Self::Nist => &[MinLen(8), NotCommon, NoEdgeSpaces],
            Self::Enterprise => &[MinLen(12), Upper, Lower, Digit, Symbol, NotCommon],
        };
        kinds.iter().map(|&k| Rule::new(k)).collect()
    }
}

/// A small list of the most common passwords (the full check belongs to a breach API).
const COMMON: [&str; 24] = [
    "password",
    "password1",
    "123456",
    "12345678",
    "123456789",
    "qwerty",
    "qwerty123",
    "111111",
    "abc123",
    "letmein",
    "welcome",
    "monkey",
    "dragon",
    "iloveyou",
    "admin",
    "football",
    "baseball",
    "sunshine",
    "princess",
    "passw0rd",
    "p@ssw0rd",
    "changeme",
    "trustno1",
    "1q2w3e4r",
];

/// On the common list (case-insensitive, ignoring a trailing `!` or digits).
pub fn is_common(pw: &str) -> bool {
    let l = pw.to_lowercase();
    let stem = l.trim_end_matches(|c: char| c.is_ascii_digit() || c == '!');
    COMMON.contains(&l.as_str()) || (!stem.is_empty() && COMMON.contains(&stem))
}

/// Strength level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Under `short_length`.
    #[default]
    TooShort,
    /// One segment.
    Weak,
    /// Two.
    Fair,
    /// Three.
    Good,
    /// Four.
    Strong,
}

impl Level {
    /// Segments lit (0–4).
    pub fn segments(self) -> u8 {
        match self {
            Self::TooShort => 0,
            Self::Weak => 1,
            Self::Fair => 2,
            Self::Good => 3,
            Self::Strong => 4,
        }
    }
    /// Word.
    pub fn word(self) -> &'static str {
        match self {
            Self::TooShort => "Too short",
            Self::Weak => "Weak",
            Self::Fair => "Fair",
            Self::Good => "Good",
            Self::Strong => "Strong",
        }
    }
    /// Id (tone hook: danger, warning, info, success).
    pub fn id(self) -> &'static str {
        match self {
            Self::TooShort => "too-short",
            Self::Weak => "weak",
            Self::Fair => "fair",
            Self::Good => "good",
            Self::Strong => "strong",
        }
    }
}

/// The score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrengthResult {
    /// Level.
    pub level: Level,
    /// Per rule: met.
    pub met: Vec<bool>,
    /// Per rule: what is still missing.
    pub remaining: Vec<Option<usize>>,
    /// The password is on the common list (Oxyn plus).
    pub common: bool,
    /// Entropy estimate in bits (Oxyn plus).
    pub bits: u32,
}

/// Estimated entropy: length × log2(pool of the classes used).
pub fn entropy_bits(pw: &str) -> u32 {
    let mut pool = 0u32;
    if pw.chars().any(|c| c.is_ascii_lowercase()) {
        pool += 26;
    }
    if pw.chars().any(|c| c.is_ascii_uppercase()) {
        pool += 26;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        pool += 10;
    }
    if pw.chars().any(|c| c.is_ascii_punctuation() || c == ' ') {
        pool += 33;
    }
    if !pw.is_ascii() {
        pool += 100;
    }
    if pool == 0 {
        return 0;
    }
    (pw.chars().count() as f32 * (pool as f32).log2()).round() as u32
}

/// Scores `pw`: below `short_length` code points the meter holds at its first step and reads
/// `Too short`; otherwise the share of rules met maps onto four steps. A common password is
/// capped at Weak.
pub fn evaluate(pw: &str, rules: &[Rule], short_length: usize) -> StrengthResult {
    let met: Vec<bool> = rules.iter().map(|r| r.test(pw)).collect();
    let remaining = rules.iter().map(|r| r.remaining(pw)).collect();
    let common = !pw.is_empty() && is_common(pw);
    let n = met.iter().filter(|&&m| m).count();
    let level = if pw.chars().count() < short_length {
        Level::TooShort
    } else if common {
        Level::Weak
    } else {
        let share = if rules.is_empty() {
            1.0
        } else {
            n as f32 / rules.len() as f32
        };
        match share {
            s if s >= 1.0 => Level::Strong,
            s if s >= 0.6 => Level::Good,
            s if s >= 0.4 => Level::Fair,
            _ => Level::Weak,
        }
    };
    StrengthResult {
        level,
        met,
        remaining,
        common,
        bits: entropy_bits(pw),
    }
}

/// Delay of segment `i` in the fill wave (ms).
pub fn segment_delay(i: usize) -> u64 {
    i as u64 * 60
}

/// Caps Lock is on, read from a key event's character and Shift state (a letter typed in upper
/// case without Shift, or lower case with it).
pub fn caps_lock_from(ch: char, shift: bool) -> Option<bool> {
    if !ch.is_alphabetic() {
        return None;
    }
    Some(ch.is_uppercase() != shift)
}

/// The slash over the eye: how far it has drawn (0..1) `ms` after a toggle; `revealed` hides it.
pub fn slash_draw(revealed: bool, ms: u64, reduced: bool) -> f32 {
    let target = if revealed { 0.0 } else { 1.0 };
    if reduced {
        return target;
    }
    let p = (ms as f32 / 180.0).clamp(0.0, 1.0);
    let k = 1.0 - (1.0 - p).powi(3);
    if revealed { 1.0 - k } else { k }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels() {
        let r = Policy::Basic.rules();
        assert_eq!(evaluate("", &r, 8).level, Level::TooShort);
        assert_eq!(evaluate("abc", &r, 8).level, Level::TooShort);
        assert_eq!(evaluate("abcdefgh", &r, 8).level, Level::Fair);
        assert_eq!(evaluate("abcdefgH", &r, 8).level, Level::Good);
        assert_eq!(evaluate("abcdefgH1!", &r, 8).level, Level::Strong);
        assert_eq!(evaluate("Password1!", &r, 8).level, Level::Weak, "common");
        let e = evaluate("abc", &r, 8);
        assert_eq!(e.remaining[0], Some(5));
        assert_eq!(e.met, [false, false, true, false, false]);
        assert_eq!("🙂🙂🙂🙂🙂🙂🙂🙂".chars().count(), 8, "code points");
        assert_ne!(evaluate("🙂🙂🙂🙂🙂🙂🙂🙂", &r, 8).level, Level::TooShort);
    }

    #[test]
    fn policies_entropy_caps() {
        let n = Policy::Nist.rules();
        assert_eq!(
            evaluate("correct horse battery", &n, 8).level,
            Level::Strong
        );
        assert_eq!(evaluate(" trailing", &n, 8).level, Level::Good);
        assert_eq!(
            Policy::Enterprise.rules()[0].label,
            "At least 12 characters"
        );
        assert!(entropy_bits("abcdefgH1!") > entropy_bits("abcdefgh"));
        assert_eq!(caps_lock_from('A', false), Some(true));
        assert_eq!(caps_lock_from('a', true), Some(true));
        assert_eq!(caps_lock_from('1', false), None);
        assert_eq!(slash_draw(false, 1_000, false), 1.0);
        assert_eq!(slash_draw(true, 1_000, false), 0.0);
        assert!(slash_draw(false, 60, false) < 1.0);
        assert_eq!(segment_delay(3), 180);
        assert!(is_common("Letmein123"));
    }
}
