//! Verb packs, the activity table and the duration phrases (`docs/specs/status-text.md` §5 and
//! `docs/specs/elapsed-timer.md` §4).
//!
//! A pack is **content, not motion**: a TOML file of present/past pairs. Both tenses are stored,
//! never generated. Timing and animation of the words live in the status-text components.

use std::{collections::BTreeMap, fmt, sync::OnceLock, time::Duration};

use serde::Deserialize;

use crate::state::{AgentState, ErrorKind};

/// Longest entry in Latin scripts, in characters.
pub const MAX_LATIN: usize = 24;
/// Longest entry in CJK scripts, in characters.
pub const MAX_CJK: usize = 12;
/// Cycle verbs per pack (fixed so width reservation stays predictable).
pub const CYCLE_LEN: usize = 12;

/// A present/past verb pair. `{d}` in either is replaced with a duration.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerbPair {
    /// "Reading".
    pub present: String,
    /// "Read".
    pub past: String,
}

impl VerbPair {
    fn new(present: &str, past: &str) -> Self {
        Self {
            present: present.into(),
            past: past.into(),
        }
    }
}

/// What kind of work an agent event is. Adapters map tool names to these (overridable).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivityKind {
    /// Reasoning.
    Think,
    /// Planning.
    Plan,
    /// Reading a file or page.
    Read,
    /// Searching.
    Search,
    /// Editing a file.
    Edit,
    /// Writing a new file.
    Write,
    /// Running a command.
    Run,
    /// Running tests.
    Test,
    /// Browsing.
    Browse,
    /// Fetching a URL.
    Fetch,
    /// Producing media.
    Produce,
    /// Listening (voice).
    Listen,
    /// Speaking (voice).
    Speak,
    /// Waiting for the user.
    WaitUser,
    /// Retrying.
    Retry,
    /// Anything else.
    Generic,
}

impl ActivityKind {
    /// Every kind.
    pub const ALL: [ActivityKind; 16] = [
        Self::Think,
        Self::Plan,
        Self::Read,
        Self::Search,
        Self::Edit,
        Self::Write,
        Self::Run,
        Self::Test,
        Self::Browse,
        Self::Fetch,
        Self::Produce,
        Self::Listen,
        Self::Speak,
        Self::WaitUser,
        Self::Retry,
        Self::Generic,
    ];

    /// The neutral English verbs (status-text.md §5.3).
    pub fn neutral(self) -> VerbPair {
        use ActivityKind::*;
        let (present, past) = match self {
            Think => ("Thinking", "Thought for {d}"),
            Plan => ("Planning", "Planned"),
            Read => ("Reading", "Read"),
            Search => ("Searching", "Searched"),
            Edit => ("Editing", "Edited"),
            Write => ("Writing", "Wrote"),
            Run => ("Running", "Ran"),
            Test => ("Testing", "Tested"),
            Browse => ("Browsing", "Browsed"),
            Fetch => ("Fetching", "Fetched"),
            Produce => ("Producing", "Produced"),
            Listen => ("Listening", "Listened"),
            Speak => ("Speaking", "Spoke"),
            WaitUser => ("Waiting for you", "Waited {d} for you"),
            Retry => ("Retrying", "Retried"),
            Generic => ("Working", "Worked for {d}"),
        };
        VerbPair::new(present, past)
    }
}

/// Cycle order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    /// A seeded shuffle bag; no repeats until the bag empties.
    Shuffle,
    /// Pack order, wrapping.
    Sequential,
}

/// Free or Pro.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Ships in the free registry.
    Free,
    /// Ships in the Pro registry.
    Pro,
}

/// A verb pack.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerbPack {
    /// Stable id (`neutral`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Who it is for.
    pub description: String,
    /// BCP 47 locale. Packs are written per locale, never machine-translated.
    pub locale: String,
    /// Free or Pro.
    pub tier: Tier,
    /// Cycle order.
    pub order: Order,
    /// "Thought for {d}": a label users scan for, the same in every English pack.
    pub reasoning_done: String,
    /// "Worked for {d}".
    pub turn_done: String,
    /// The flavour verbs.
    pub cycle: Vec<VerbPair>,
    /// Pinned verbs for agent activities that differ from the neutral table.
    #[serde(default)]
    pub activity: BTreeMap<ActivityKind, VerbPair>,
}

/// Why a pack was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackError(pub String);

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PackError {}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF)
}

fn check_entry(pack: &str, what: &str, s: &str) -> Result<(), PackError> {
    let err = |m: String| Err(PackError(format!("{pack}: {what} {s:?}: {m}")));
    if s.trim().is_empty() {
        return err("empty".into());
    }
    if s.trim() != s {
        return err("leading or trailing space".into());
    }
    if s.ends_with('…') || s.ends_with("...") {
        return err("no trailing ellipsis; it is a variant decoration".into());
    }
    let shown = s.replace("{d}", "1m 05s");
    let n = shown.chars().count();
    let max = if shown.chars().any(is_cjk) {
        MAX_CJK
    } else {
        MAX_LATIN
    };
    if n > max {
        return err(format!("{n} characters, max {max}"));
    }
    Ok(())
}

impl VerbPack {
    /// Parses and validates a pack (status-text.md §5.1 rules).
    pub fn parse(toml_text: &str) -> Result<VerbPack, PackError> {
        let pack: VerbPack = toml::from_str(toml_text).map_err(|e| PackError(e.to_string()))?;
        pack.validate()?;
        Ok(pack)
    }

    /// Checks the rules: 12 cycle verbs, both tenses, lengths, no ellipsis, no duplicates,
    /// `{d}` in the completion templates.
    pub fn validate(&self) -> Result<(), PackError> {
        let id = &self.id;
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(PackError(format!("bad pack id {id:?}")));
        }
        if self.cycle.len() != CYCLE_LEN {
            return Err(PackError(format!(
                "{id}: {} cycle verbs, need {CYCLE_LEN}",
                self.cycle.len()
            )));
        }
        for (field, t) in [
            ("reasoning_done", &self.reasoning_done),
            ("turn_done", &self.turn_done),
        ] {
            if !t.contains("{d}") {
                return Err(PackError(format!("{id}: {field} needs {{d}}")));
            }
            check_entry(id, field, t)?;
        }
        let mut seen = std::collections::HashSet::new();
        for v in self.cycle.iter().chain(self.activity.values()) {
            check_entry(id, "present", &v.present)?;
            check_entry(id, "past", &v.past)?;
        }
        for v in &self.cycle {
            if !seen.insert(v.present.as_str()) {
                return Err(PackError(format!(
                    "{id}: duplicate cycle verb {:?}",
                    v.present
                )));
            }
        }
        Ok(())
    }

    /// The pinned verbs for an activity: the pack's override, else the neutral table.
    pub fn activity(&self, kind: ActivityKind) -> VerbPair {
        self.activity
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| kind.neutral())
    }

    /// The longest present-tense string this pack can show (for `ReserveLongest` width policies).
    pub fn longest_present(&self) -> &str {
        self.cycle
            .iter()
            .chain(self.activity.values())
            .map(|v| v.present.as_str())
            .max_by_key(|s| s.chars().count())
            .unwrap_or("")
    }

    /// The default pack.
    pub fn neutral() -> &'static VerbPack {
        builtin(0)
    }

    /// Looks up a bundled pack by id.
    pub fn builtin(id: &str) -> Option<&'static VerbPack> {
        BUILTIN.iter().position(|(pid, _)| *pid == id).map(builtin)
    }

    /// Ids of the bundled (free) packs.
    pub fn builtin_ids() -> impl Iterator<Item = &'static str> {
        BUILTIN.iter().map(|(id, _)| *id)
    }
}

const BUILTIN: [(&str, &str); 4] = [
    ("neutral", include_str!("../verb_packs/neutral.toml")),
    ("coder", include_str!("../verb_packs/coder.toml")),
    ("terse", include_str!("../verb_packs/terse.toml")),
    ("crm", include_str!("../verb_packs/crm.toml")),
];

fn builtin(i: usize) -> &'static VerbPack {
    static PACKS: OnceLock<Vec<VerbPack>> = OnceLock::new();
    &PACKS.get_or_init(|| {
        BUILTIN
            .iter()
            .map(|(id, text)| {
                let p = VerbPack::parse(text).unwrap_or_else(|e| panic!("bundled pack {id}: {e}"));
                assert_eq!(&p.id, id, "bundled pack id");
                p
            })
            .collect()
    })[i]
}

/// Live-timer format (`Units`): `42s`, `1m 05s`, `1h 02m 33s`, `1d 03h`. Always floors.
pub fn format_units(d: Duration) -> String {
    let s = d.as_secs();
    let (days, h, m, sec) = (s / 86_400, s / 3_600 % 24, s / 60 % 60, s % 60);
    match s {
        0..60 => format!("{sec}s"),
        60..3_600 => format!("{m}m {sec:02}s"),
        3_600..86_400 => format!("{h}h {m:02}m {sec:02}s"),
        _ => format!("{days}d {h:02}h"),
    }
}

/// Summary format for `{d}`: `42s`, `1m 05s`, `1h 02m`, `1d 3h` (seconds dropped from 1 h up).
pub fn format_summary(d: Duration) -> String {
    let s = d.as_secs();
    let (days, h, m, sec) = (s / 86_400, s / 3_600 % 24, s / 60 % 60, s % 60);
    match s {
        0..60 => format!("{sec}s"),
        60..3_600 => format!("{m}m {sec:02}s"),
        3_600..86_400 => format!("{h}h {m:02}m"),
        _ => format!("{days}d {h}h"),
    }
}

/// Fills `{d}` in a template with [`format_summary`].
pub fn fill(template: &str, d: Duration) -> String {
    template.replace("{d}", &format_summary(d))
}

/// A short word for an error kind ("Stopped · timeout").
pub fn error_label(kind: &ErrorKind) -> &str {
    match kind {
        ErrorKind::Tool => "tool error",
        ErrorKind::Model => "model error",
        ErrorKind::Timeout => "timeout",
        ErrorKind::Offline => "offline",
        ErrorKind::RateLimited => "rate limited",
        ErrorKind::Other(s) => s,
    }
}

/// The past-tense line for a finished run, or `None` (still running, or done in under 1 s).
///
/// `Done` → the pack's `turn_done`; `Cancelled` → "Interrupted after {d}"; `Error` →
/// "Stopped · {kind}".
pub fn summary(state: &AgentState, pack: &VerbPack) -> Option<String> {
    match state {
        AgentState::Done { elapsed, .. } if *elapsed >= Duration::from_secs(1) => {
            Some(fill(&pack.turn_done, *elapsed))
        }
        AgentState::Cancelled { after } => {
            Some(format!("Interrupted after {}", format_summary(*after)))
        }
        AgentState::Error { kind } => Some(format!("Stopped · {}", error_label(kind))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_packs_parse_and_validate() {
        let ids: Vec<_> = VerbPack::builtin_ids().collect();
        assert_eq!(ids, ["neutral", "coder", "terse", "crm"]);
        for id in ids {
            let p = VerbPack::builtin(id).unwrap();
            assert_eq!(p.cycle.len(), CYCLE_LEN);
            assert_eq!(p.tier, Tier::Free);
            assert_eq!(p.reasoning_done, "Thought for {d}");
        }
        assert!(VerbPack::builtin("oxyn").is_none());
    }

    #[test]
    fn activity_overrides_fall_back_to_neutral() {
        let coder = VerbPack::builtin("coder").unwrap();
        assert_eq!(coder.activity(ActivityKind::Search).present, "Grepping");
        assert_eq!(coder.activity(ActivityKind::Read).present, "Reading");
        assert_eq!(
            VerbPack::neutral().activity(ActivityKind::Write).past,
            "Wrote"
        );
        for kind in ActivityKind::ALL {
            let v = kind.neutral();
            check_entry("neutral", "activity", &v.present).unwrap();
            check_entry("neutral", "activity", &v.past).unwrap();
        }
    }

    #[test]
    fn rejects_bad_packs() {
        let base = include_str!("../verb_packs/neutral.toml");
        assert!(VerbPack::parse(&base.replace("Planning", "Planning…")).is_err());
        assert!(
            VerbPack::parse(&base.replace("Planning", "Planning very carefully indeed")).is_err()
        );
        assert!(VerbPack::parse(&base.replace("Worked for {d}", "Worked")).is_err());
        assert!(VerbPack::parse(&base.replace("Comparing", "Planning")).is_err());
        let eleven = base.rsplit_once("[[cycle]]").unwrap().0;
        assert!(VerbPack::parse(eleven).is_err());
        assert!(VerbPack::parse(&format!("{base}\nextra = 1\n")).is_err());
    }

    #[test]
    fn duration_formats() {
        let s = Duration::from_secs;
        assert_eq!(format_units(Duration::from_millis(59_600)), "59s");
        assert_eq!(format_units(s(65)), "1m 05s");
        assert_eq!(format_units(s(3_753)), "1h 02m 33s");
        assert_eq!(format_units(s(97_200)), "1d 03h");
        assert_eq!(format_summary(s(72)), "1m 12s");
        assert_eq!(format_summary(s(3_753)), "1h 02m");
        assert_eq!(format_summary(s(97_200)), "1d 3h");
    }

    #[test]
    fn summaries() {
        let p = VerbPack::neutral();
        let done = |ms| AgentState::Done {
            elapsed: Duration::from_millis(ms),
            waited: Duration::ZERO,
        };
        assert_eq!(
            summary(&done(72_000), p).as_deref(),
            Some("Worked for 1m 12s")
        );
        assert_eq!(summary(&done(900), p), None);
        assert_eq!(
            summary(
                &AgentState::Cancelled {
                    after: Duration::from_secs(42)
                },
                p
            )
            .as_deref(),
            Some("Interrupted after 42s")
        );
        assert_eq!(
            summary(
                &AgentState::Error {
                    kind: ErrorKind::Timeout
                },
                p
            )
            .as_deref(),
            Some("Stopped · timeout")
        );
        assert_eq!(summary(&AgentState::Streaming, p), None);
        let terse = VerbPack::builtin("terse").unwrap();
        assert_eq!(summary(&done(9_600), terse).as_deref(), Some("Done · 9s"));
    }
}
