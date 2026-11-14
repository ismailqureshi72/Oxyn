//! Tag input (`tag-input.md`): several values in one field as chips. Paste a whole list, a check
//! per value, duplicates flash the chip they repeat, Backspace marks then removes the last chip,
//! arrows walk the chips, a max with a `3/10` counter.
//!
//! Behaviour follows the affectUI Tag Input and Arc UI Tag input docs (rebuilt, no source used).
//! Oxyn plus: typed chips (person / file / tool / label, from a prefix), agent suggestions that
//! fill as you type, and an overflow mode that folds chips past a count into `+N`.

use crate::ClockMs;

/// How a typed value is cleaned before it becomes a chip (affectUI `normalize`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Normalize {
    /// Trim spaces (default).
    #[default]
    Trim,
    /// Trim and lowercase (emails).
    Lower,
    /// Trim, lowercase and turn spaces into `-` (slugs, labels).
    Slug,
}

impl Normalize {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "lower" => Self::Lower,
            "slug" => Self::Slug,
            _ => Self::Trim,
        }
    }
    /// Applies it.
    pub fn apply(self, s: &str) -> String {
        let t = s.trim();
        match self {
            Self::Trim => t.to_string(),
            Self::Lower => t.to_lowercase(),
            Self::Slug => t
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-"),
        }
    }
}

/// The check run on a value before it becomes a chip (affectUI `validate`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TagRule {
    /// Anything non-empty.
    #[default]
    Any,
    /// An email address.
    Email,
    /// A domain (`acme.com`).
    Domain,
    /// No longer than this many characters.
    MaxLen(usize),
}

impl TagRule {
    /// From an id (`email`, `domain`, `max:24`).
    pub fn from_id(s: &str) -> Self {
        match s {
            "email" => Self::Email,
            "domain" => Self::Domain,
            _ => s
                .strip_prefix("max:")
                .and_then(|n| n.parse().ok())
                .map(Self::MaxLen)
                .unwrap_or(Self::Any),
        }
    }
    /// `Ok` or the message shown under the field.
    pub fn check(self, v: &str) -> Result<(), String> {
        match self {
            Self::Any => Ok(()),
            Self::Email => {
                if is_email(v) {
                    Ok(())
                } else {
                    Err(format!("“{v}” isn't an email address"))
                }
            }
            Self::Domain => {
                let ok = v.contains('.')
                    && !v.contains('@')
                    && !v.starts_with('.')
                    && !v.ends_with('.')
                    && v.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
                if ok {
                    Ok(())
                } else {
                    Err(format!("“{v}” isn't a domain"))
                }
            }
            Self::MaxLen(n) => {
                if v.chars().count() <= n {
                    Ok(())
                } else {
                    Err(format!("Keep tags under {n} characters"))
                }
            }
        }
    }
}

/// A loose email check: one `@`, a dot in the domain, no spaces.
pub fn is_email(v: &str) -> bool {
    let Some((user, host)) = v.split_once('@') else {
        return false;
    };
    !user.is_empty()
        && !host.contains('@')
        && host.contains('.')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !v.contains(char::is_whitespace)
}

/// Oxyn plus: what a chip refers to, read from its prefix (`@` person, `/` or a dot-extension
/// file, `#` label, `tool:` tool).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TagKind {
    /// Plain text.
    #[default]
    Text,
    /// A person (`@sam`, or an email).
    Person,
    /// A file (`src/auth.rs`, `/docs`).
    File,
    /// A tool (`tool:search`).
    Tool,
    /// A label (`#bug`).
    Label,
}

impl TagKind {
    /// Reads the kind from a value.
    pub fn of(v: &str) -> Self {
        if v.starts_with('@') || is_email(v) {
            Self::Person
        } else if v.starts_with("tool:") {
            Self::Tool
        } else if v.starts_with('#') {
            Self::Label
        } else if v.starts_with('/')
            || v.contains('/')
            || v.rsplit_once('.').is_some_and(|(a, b)| {
                !a.is_empty() && (1..=4).contains(&b.len()) && !v.contains(' ')
            })
        {
            Self::File
        } else {
            Self::Text
        }
    }
    /// Id (CSS hook).
    pub fn id(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Person => "person",
            Self::File => "file",
            Self::Tool => "tool",
            Self::Label => "label",
        }
    }
    /// Icon name.
    pub fn icon(self) -> Option<&'static str> {
        match self {
            Self::Text => None,
            Self::Person => Some("agents"),
            Self::File => Some("file"),
            Self::Tool => Some("tool"),
            Self::Label => Some("symbol"),
        }
    }
}

/// One chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    /// Value.
    pub text: String,
    /// Kind (Oxyn plus).
    pub kind: TagKind,
    /// When it was added (pop-in).
    pub added_at: ClockMs,
}

/// What a commit did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagEvent {
    /// Added these values.
    Added(Vec<String>),
    /// The value repeats chip `i`, which flashes.
    Duplicate(usize),
    /// The value failed its check; it stays in the field with the message.
    Invalid(String),
    /// At the max; nothing added.
    Full,
    /// Chip `i` was removed (its value).
    Removed(usize, String),
    /// Nothing happened.
    None,
}

/// Keys the field understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagKey {
    /// Enter.
    Enter,
    /// `,`.
    Comma,
    /// Space (adds only with `split_on_space`).
    Space,
    /// Tab (adds, then moves on).
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// ←.
    Left,
    /// →.
    Right,
    /// Esc.
    Escape,
}

/// How chips past a count show (Oxyn plus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Overflow {
    /// Wrap onto new lines; after about five lines the field scrolls (source).
    #[default]
    Wrap,
    /// Fold chips past `n` into `+N`.
    Fold(usize),
}

/// The tag input.
#[derive(Clone, Debug, PartialEq)]
pub struct TagInput {
    /// Chips.
    pub tags: Vec<Tag>,
    /// What is being typed.
    pub draft: String,
    /// Chip marked by Backspace or walked to with arrows.
    pub picked: Option<usize>,
    /// Max chips.
    pub max: Option<usize>,
    /// Split on spaces too.
    pub split_on_space: bool,
    /// Cleaning.
    pub normalize: Normalize,
    /// Check.
    pub rule: TagRule,
    /// Disabled.
    pub disabled: bool,
    /// Chip that flashes (duplicate) and when.
    pub flash: Option<(usize, ClockMs)>,
    /// Message under the field (validation; replaces the hint until the user types).
    pub message: Option<String>,
    /// Oxyn plus: suggestions offered under the field as you type.
    pub suggestions: Vec<String>,
    /// Oxyn plus: overflow mode.
    pub overflow: Overflow,
    /// Last announcement for the live region.
    pub announce: String,
}

impl TagInput {
    /// A field with these chips.
    pub fn new(values: &[&str]) -> Self {
        Self {
            tags: values
                .iter()
                .map(|v| Tag {
                    text: (*v).into(),
                    kind: TagKind::of(v),
                    added_at: ClockMs(0),
                })
                .collect(),
            draft: String::new(),
            picked: None,
            max: None,
            split_on_space: false,
            normalize: Normalize::Trim,
            rule: TagRule::Any,
            disabled: false,
            flash: None,
            message: None,
            suggestions: vec![],
            overflow: Overflow::Wrap,
            announce: String::new(),
        }
    }
    /// The values.
    pub fn values(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.text.clone()).collect()
    }
    /// At the max.
    pub fn full(&self) -> bool {
        self.max.is_some_and(|m| self.tags.len() >= m)
    }
    /// `3/10` under the field.
    pub fn counter(&self) -> Option<String> {
        self.max.map(|m| format!("{}/{}", self.tags.len(), m))
    }
    /// Placeholder: `Add…` while empty, `Add more…` after the first chip.
    pub fn placeholder<'a>(&self, base: &'a str) -> &'a str {
        if self.tags.is_empty() {
            base
        } else {
            "Add more…"
        }
    }
    /// Typing: clears the pick and the message.
    pub fn input(&mut self, text: &str) {
        if self.disabled {
            return;
        }
        // A separator typed at the end commits (Enter is a key; `,` may arrive as text).
        self.picked = None;
        self.message = None;
        self.draft = text.into();
    }
    fn add_one(&mut self, raw: &str, now: ClockMs) -> TagEvent {
        let v = self.normalize.apply(raw);
        if v.is_empty() {
            return TagEvent::None;
        }
        if let Some(i) = self
            .tags
            .iter()
            .position(|t| t.text.to_lowercase() == v.to_lowercase())
        {
            self.flash = Some((i, now));
            return TagEvent::Duplicate(i);
        }
        if let Err(m) = self.rule.check(&v) {
            return TagEvent::Invalid(m);
        }
        if self.full() {
            return TagEvent::Full;
        }
        self.tags.push(Tag {
            kind: TagKind::of(&v),
            text: v.clone(),
            added_at: now,
        });
        TagEvent::Added(vec![v])
    }
    /// Commits the draft (Enter, comma, Tab, blur). A draft holding separators is split.
    pub fn commit(&mut self, now: ClockMs) -> TagEvent {
        if self.disabled {
            return TagEvent::None;
        }
        let draft = std::mem::take(&mut self.draft);
        let ev = self.add_many(&draft, now);
        match &ev {
            TagEvent::Invalid(m) => {
                self.message = Some(m.clone());
                self.draft = draft;
            }
            TagEvent::Full => {
                self.message = Some(format!("Up to {} tags", self.max.unwrap_or(0)));
                self.draft = draft;
            }
            _ => {}
        }
        ev
    }
    fn split<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let space = self.split_on_space;
        text.split(|c: char| c == ',' || c == '\n' || c == ';' || (space && c.is_whitespace()))
            .filter(|s| !s.trim().is_empty())
            .collect()
    }
    fn add_many(&mut self, text: &str, now: ClockMs) -> TagEvent {
        let parts = self.split(text);
        if parts.len() <= 1 {
            let ev = self.add_one(parts.first().copied().unwrap_or(""), now);
            self.announce_ev(&ev);
            return ev;
        }
        let mut added = vec![];
        let mut bad = vec![];
        let mut last = TagEvent::None;
        for p in parts {
            match self.add_one(p, now) {
                TagEvent::Added(v) => added.extend(v),
                TagEvent::Invalid(_) => bad.push(p.trim().to_string()),
                TagEvent::Full => {
                    last = TagEvent::Full;
                    break;
                }
                e => last = e,
            }
        }
        let ev = if !added.is_empty() {
            TagEvent::Added(added)
        } else {
            last
        };
        self.announce_ev(&ev);
        if !bad.is_empty() {
            // Keep the rejected values in the field so they can be fixed.
            self.draft = bad.join(", ");
            self.message = Some(format!(
                "{} value{} didn't pass the check",
                bad.len(),
                if bad.len() == 1 { "" } else { "s" }
            ));
        }
        ev
    }
    /// Paste: the whole list at once (commas, semicolons and new lines split it).
    pub fn paste(&mut self, text: &str, now: ClockMs) -> TagEvent {
        if self.disabled {
            return TagEvent::None;
        }
        let joined = format!("{}{}", self.draft, text);
        self.draft.clear();
        self.add_many(&joined, now)
    }
    fn announce_ev(&mut self, ev: &TagEvent) {
        self.announce = match ev {
            TagEvent::Added(v) if v.len() == 1 => format!("Added {}", v[0]),
            TagEvent::Added(v) => format!("Added {} tags", v.len()),
            TagEvent::Duplicate(i) => format!("{} is already added", self.tags[*i].text),
            TagEvent::Removed(_, v) => format!("Removed {v}"),
            TagEvent::Full => "Tag limit reached".into(),
            TagEvent::Invalid(m) => m.clone(),
            TagEvent::None => return,
        };
    }
    /// Removes chip `i` (its × button).
    pub fn remove(&mut self, i: usize) -> TagEvent {
        if self.disabled || i >= self.tags.len() {
            return TagEvent::None;
        }
        let t = self.tags.remove(i);
        self.picked = None;
        self.flash = None;
        let ev = TagEvent::Removed(i, t.text);
        self.announce_ev(&ev);
        ev
    }
    /// Keyboard. `caret_at_start` says the caret sits at the start of the draft.
    pub fn key(&mut self, k: TagKey, caret_at_start: bool, now: ClockMs) -> TagEvent {
        if self.disabled {
            return TagEvent::None;
        }
        match k {
            TagKey::Enter | TagKey::Comma | TagKey::Tab => {
                if let (TagKey::Enter, Some(i)) = (k, self.picked) {
                    // Enter on a picked chip moves it back into the field for editing.
                    let t = self.tags.remove(i);
                    self.picked = None;
                    self.draft = t.text.clone();
                    return TagEvent::Removed(i, t.text);
                }
                self.commit(now)
            }
            TagKey::Space if self.split_on_space => self.commit(now),
            TagKey::Space => TagEvent::None,
            TagKey::Backspace => match self.picked {
                Some(i) => self.remove(i),
                None if self.draft.is_empty() || caret_at_start => {
                    if !self.tags.is_empty() && self.draft.is_empty() {
                        self.picked = Some(self.tags.len() - 1);
                    }
                    TagEvent::None
                }
                None => TagEvent::None,
            },
            TagKey::Delete => match self.picked {
                Some(i) => {
                    let ev = self.remove(i);
                    // Focus stays on the chip that slid into its place.
                    if i < self.tags.len() {
                        self.picked = Some(i);
                    }
                    ev
                }
                None => TagEvent::None,
            },
            TagKey::Left => {
                self.picked = match self.picked {
                    Some(i) => Some(i.saturating_sub(1)),
                    None if caret_at_start && !self.tags.is_empty() => Some(self.tags.len() - 1),
                    None => None,
                };
                TagEvent::None
            }
            TagKey::Right => {
                self.picked = match self.picked {
                    Some(i) if i + 1 < self.tags.len() => Some(i + 1),
                    _ => None,
                };
                TagEvent::None
            }
            TagKey::Escape => {
                if self.picked.is_some() {
                    self.picked = None;
                } else {
                    self.draft.clear();
                    self.message = None;
                }
                TagEvent::None
            }
        }
    }
    /// Suggestions matching the draft, not yet added (Oxyn plus).
    pub fn matches(&self) -> Vec<&str> {
        let q = self.draft.trim().to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        self.suggestions
            .iter()
            .filter(|s| s.to_lowercase().contains(&q))
            .filter(|s| !self.tags.iter().any(|t| t.text.eq_ignore_ascii_case(s)))
            .take(5)
            .map(String::as_str)
            .collect()
    }
    /// Accepts a suggestion.
    pub fn accept(&mut self, s: &str, now: ClockMs) -> TagEvent {
        self.draft = s.into();
        self.commit(now)
    }
    /// Chips shown and the `+N` count (Oxyn plus overflow; the picked chip always shows).
    pub fn shown(&self) -> (usize, usize) {
        match self.overflow {
            Overflow::Wrap => (self.tags.len(), 0),
            Overflow::Fold(n) => {
                let n = n.max(1).max(self.picked.map_or(0, |p| p + 1));
                let shown = self.tags.len().min(n);
                (shown, self.tags.len() - shown)
            }
        }
    }
    /// Whether chip `i` is flashing at `now` (600 ms).
    pub fn flashing(&self, i: usize, now: ClockMs) -> bool {
        self.flash
            .is_some_and(|(f, t)| f == i && now.since(t).as_millis() < 600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_splits_and_flashes_duplicates() {
        let mut t = TagInput::new(&["design"]);
        t.input("  Research ");
        assert_eq!(
            t.commit(ClockMs(0)),
            TagEvent::Added(vec!["Research".into()])
        );
        t.input("research");
        assert_eq!(t.commit(ClockMs(10)), TagEvent::Duplicate(1));
        assert!(t.flashing(1, ClockMs(300)) && !t.flashing(1, ClockMs(700)));
        assert_eq!(t.draft, "", "a duplicate is not kept in the field");
        let ev = t.paste("ops, infra\nml;  ", ClockMs(20));
        assert_eq!(
            ev,
            TagEvent::Added(vec!["ops".into(), "infra".into(), "ml".into()])
        );
        assert_eq!(t.announce, "Added 3 tags");
        t.split_on_space = true;
        t.input("a b");
        t.commit(ClockMs(30));
        assert_eq!(t.tags.len(), 7);
    }

    #[test]
    fn validates_normalizes_and_caps() {
        let mut t = TagInput::new(&[]);
        t.rule = TagRule::Email;
        t.normalize = Normalize::Lower;
        t.max = Some(2);
        t.input("Sam@Acme.dev");
        t.commit(ClockMs(0));
        assert_eq!(t.values(), ["sam@acme.dev"]);
        assert_eq!(t.tags[0].kind, TagKind::Person);
        t.input("not-an-email");
        assert!(matches!(t.commit(ClockMs(1)), TagEvent::Invalid(_)));
        assert_eq!(t.draft, "not-an-email", "kept for fixing");
        assert!(t.message.is_some());
        t.input("x@y.io");
        t.commit(ClockMs(2));
        t.input("z@y.io");
        assert_eq!(t.commit(ClockMs(3)), TagEvent::Full);
        assert_eq!(t.counter().as_deref(), Some("2/2"));
        t.input("");
        let ev = t.paste("ok@a.io, bad", ClockMs(4));
        assert_eq!(ev, TagEvent::Full);
    }

    #[test]
    fn keyboard_marks_walks_and_removes() {
        let mut t = TagInput::new(&["a", "b", "c"]);
        assert_eq!(t.key(TagKey::Backspace, true, ClockMs(0)), TagEvent::None);
        assert_eq!(t.picked, Some(2));
        assert_eq!(
            t.key(TagKey::Backspace, true, ClockMs(1)),
            TagEvent::Removed(2, "c".into())
        );
        t.key(TagKey::Left, true, ClockMs(2));
        t.key(TagKey::Left, true, ClockMs(3));
        assert_eq!(t.picked, Some(0));
        t.key(TagKey::Delete, true, ClockMs(4));
        assert_eq!(t.values(), ["b"]);
        assert_eq!(t.picked, Some(0), "focus stays on the chip that slid in");
        t.key(TagKey::Right, true, ClockMs(5));
        assert_eq!(t.picked, None, "back to the field");
        t.input("draft");
        t.key(TagKey::Escape, false, ClockMs(6));
        assert_eq!(t.draft, "");
        t.key(TagKey::Left, true, ClockMs(7));
        t.key(TagKey::Enter, true, ClockMs(8));
        assert_eq!(t.draft, "b", "Enter on a chip edits it");
    }

    #[test]
    fn kinds_suggestions_overflow() {
        assert_eq!(TagKind::of("@sam"), TagKind::Person);
        assert_eq!(TagKind::of("src/auth.rs"), TagKind::File);
        assert_eq!(TagKind::of("tool:search"), TagKind::Tool);
        assert_eq!(TagKind::of("#bug"), TagKind::Label);
        assert_eq!(TagKind::of("billing team"), TagKind::Text);
        let mut t = TagInput::new(&["a", "b", "c", "d"]);
        t.overflow = Overflow::Fold(2);
        assert_eq!(t.shown(), (2, 2));
        t.suggestions = vec!["Alpha".into(), "alphabet".into(), "beta".into()];
        t.input("alp");
        assert_eq!(t.matches(), ["Alpha", "alphabet"]);
        t.accept("Alpha", ClockMs(0));
        assert_eq!(t.tags.len(), 5);
        assert_eq!(
            TagRule::from_id("max:3").check("long"),
            Err("Keep tags under 3 characters".into())
        );
        assert!(TagRule::Domain.check("acme.com").is_ok());
        assert_eq!(Normalize::Slug.apply(" Needs Review "), "needs-review");
    }
}
