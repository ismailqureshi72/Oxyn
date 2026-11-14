//! Search field, expanding search and shortcut recorder (`search-field.md`,
//! `expanding-search.md`, `shortcut-recorder.md`).
//!
//! - **Search field** (Arc UI Search field, Beautiful UI Search — MIT, © 2026 Shane Levine): a
//!   search input whose clear button scales in inside a reserved slot so the width never shifts;
//!   Escape clears. Beautiful UI's Search adds live filtering with a gliding row highlight and an
//!   empty state ("No results found / Adjust your search to try again").
//! - **Expanding search** (Arc UI): an icon button that morphs into a field with grouped,
//!   highlighted results beneath; ↑ ↓ wrap, Enter picks and folds back, Escape folds.
//! - **Shortcut recorder** (Arc UI): records a key combination into caps, requires a modifier
//!   (function keys excepted), warns about combinations the system keeps and asks before taking
//!   one already bound ("Use anyway" / "Cancel"). A searchable cheatsheet lights the caps of held
//!   modifiers and fades rows that don't use them.
//!
//! Oxyn plus: scope chips and an "Ask AI" row on the search field, recent queries and a ⌘K
//! binding on the expanding search, chord sequences ("G then I") and a JSON keymap export on the
//! recorder. Rebuilt from the docs; Beautiful UI's behaviour ported with attribution.

use std::time::Duration;

use super::{
    command::{Command, Hit, Section, search},
    display::{Platform, parse_shortcut},
};
use crate::ClockMs;

/// Search field: the clear button shows when there is text.
pub fn clear_visible(value: &str, disabled: bool) -> bool {
    !value.is_empty() && !disabled
}

/// Beautiful UI Search: filters plain strings, case-insensitive substring, with match ranges.
pub fn filter_strings(items: &[String], query: &str) -> Vec<(usize, Vec<(usize, usize)>)> {
    let q = query.trim().to_lowercase();
    items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if q.is_empty() {
                return Some((i, vec![]));
            }
            let l = s.to_lowercase();
            l.find(&q).map(|b| {
                let a = l[..b].chars().count();
                (i, vec![(a, a + q.chars().count())])
            })
        })
        .collect()
}

/// Where the expanding search's button sits; the field grows away from it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// Left: grows right.
    Start,
    /// Centred: grows evenly.
    Center,
    /// Right: grows left.
    #[default]
    End,
}

impl Anchor {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "start" => Self::Start,
            "center" => Self::Center,
            _ => Self::End,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchOutcome {
    /// Nothing for the host.
    None,
    /// Picked an item (id); the field folds back.
    Select(String),
    /// Folded back.
    Folded,
}

/// Expanding search (Arc UI).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandingSearch {
    /// Items.
    pub items: Vec<Command>,
    /// Shown before anything is typed.
    pub suggestions: Vec<Command>,
    /// Expanded.
    pub expanded: bool,
    /// Query.
    pub query: String,
    /// Active option.
    pub active: Option<usize>,
    /// Most results shown (6).
    pub max_results: usize,
    /// Widest the field grows px (360).
    pub expanded_width: f32,
    /// Anchor.
    pub anchor: Anchor,
    /// Oxyn plus: recent queries.
    pub recent: Vec<String>,
}

impl ExpandingSearch {
    /// Over `items`.
    pub fn new(items: Vec<Command>) -> Self {
        Self {
            items,
            suggestions: vec![],
            expanded: false,
            query: String::new(),
            active: None,
            max_results: 6,
            expanded_width: 360.0,
            anchor: Anchor::End,
            recent: vec![],
        }
    }
    /// Field width for a container of `container` px.
    pub fn width(&self, container: f32, collapsed: f32) -> f32 {
        if self.expanded {
            self.expanded_width.min(container).max(collapsed)
        } else {
            collapsed
        }
    }
    /// Opens (Enter / Space / click on the icon, or ⌘K).
    pub fn expand(&mut self) {
        self.expanded = true;
        self.active = None;
    }
    /// Folds back (Escape, a pick, blur with nothing typed).
    pub fn fold(&mut self) {
        self.expanded = false;
        self.query.clear();
        self.active = None;
    }
    /// Sections shown now.
    pub fn sections(&self) -> Vec<Section> {
        if self.query.trim().is_empty() {
            if self.suggestions.is_empty() {
                return vec![];
            }
            return vec![Section {
                label: "Recent".into(),
                rows: (0..self.suggestions.len().min(self.max_results))
                    .map(|i| (i, Hit::default()))
                    .collect(),
            }];
        }
        search(&self.items, &self.query, None, self.max_results)
    }
    fn showing_suggestions(&self) -> bool {
        self.query.trim().is_empty()
    }
    /// Command for a section row.
    pub fn item(&self, i: usize) -> Option<&Command> {
        if self.showing_suggestions() {
            self.suggestions.get(i)
        } else {
            self.items.get(i)
        }
    }
    /// Flat rows.
    pub fn flat(&self) -> Vec<usize> {
        self.sections()
            .iter()
            .flat_map(|s| s.rows.iter().map(|r| r.0))
            .collect()
    }
    /// Typing.
    pub fn set_query(&mut self, q: &str) {
        self.query = q.into();
        self.active = (!self.flat().is_empty() && !q.is_empty()).then_some(0);
    }
    /// ↓ / ↑ (wrap).
    pub fn step(&mut self, down: bool) {
        let n = self.flat().len();
        if n == 0 {
            self.active = None;
            return;
        }
        self.active = Some(match (self.active, down) {
            (None, true) => 0,
            (None, false) => n - 1,
            (Some(i), true) => (i + 1) % n,
            (Some(i), false) => (i + n - 1) % n,
        });
    }
    /// Enter.
    pub fn enter(&mut self) -> SearchOutcome {
        let flat = self.flat();
        let Some(i) = self.active.and_then(|a| flat.get(a).copied()) else {
            return SearchOutcome::None;
        };
        self.pick(i)
    }
    /// Picks row item `i` (click).
    pub fn pick(&mut self, i: usize) -> SearchOutcome {
        let Some(c) = self.item(i).cloned() else {
            return SearchOutcome::None;
        };
        if !self.query.trim().is_empty() {
            let q = self.query.trim().to_string();
            self.recent.retain(|r| r != &q);
            self.recent.insert(0, q);
            self.recent.truncate(4);
        }
        self.fold();
        SearchOutcome::Select(c.id)
    }
    /// Escape.
    pub fn escape(&mut self) -> SearchOutcome {
        if self.expanded {
            self.fold();
            SearchOutcome::Folded
        } else {
            SearchOutcome::None
        }
    }
    /// Polite count.
    pub fn announce(&self) -> String {
        if !self.expanded {
            return String::new();
        }
        match self.flat().len() {
            0 if self.query.trim().is_empty() => String::new(),
            0 => format!("No results for {}", self.query.trim()),
            1 => "1 result".into(),
            n => format!("{n} results"),
        }
    }
}

/// Held modifiers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods {
    /// ⌘ / Win.
    pub meta: bool,
    /// Ctrl.
    pub ctrl: bool,
    /// ⌥ / Alt.
    pub alt: bool,
    /// ⇧.
    pub shift: bool,
}

impl Mods {
    /// Any held.
    pub fn any(self) -> bool {
        self.meta || self.ctrl || self.alt || self.shift
    }
    /// A modifier other than Shift (Arc's `requireModifier`).
    pub fn primary(self) -> bool {
        self.meta || self.ctrl || self.alt
    }
}

/// A recorded chord, canonical: `mod+ctrl+alt+shift+meta+key` order (mod = ⌘ on Mac, Ctrl
/// elsewhere), so `mod+shift+k` compares equal however it was written.
pub fn canonical(mods: Mods, key: &str, platform: Platform) -> String {
    let mac = platform == Platform::Mac;
    let mut parts: Vec<&str> = vec![];
    if (mac && mods.meta) || (!mac && mods.ctrl) {
        parts.push("mod");
    }
    if mac && mods.ctrl {
        parts.push("ctrl");
    }
    if mods.alt {
        parts.push("alt");
    }
    if mods.shift {
        parts.push("shift");
    }
    if !mac && mods.meta {
        parts.push("meta");
    }
    let k = key.to_lowercase();
    parts.push(&k);
    parts.join("+")
}

/// Normalises a written shortcut (`Shift+Mod+K`, `cmd+k`) to [`canonical`] order.
pub fn normalize(s: &str, platform: Platform) -> String {
    s.split_whitespace()
        .map(|chord| {
            let mut m = Mods::default();
            let mut key = String::new();
            for p in chord.split('+').filter(|p| !p.is_empty()) {
                match p.to_lowercase().as_str() {
                    "mod" => {
                        if platform == Platform::Mac {
                            m.meta = true
                        } else {
                            m.ctrl = true
                        }
                    }
                    "cmd" | "command" | "meta" => m.meta = true,
                    "ctrl" | "control" => m.ctrl = true,
                    "alt" | "option" | "opt" => m.alt = true,
                    "shift" => m.shift = true,
                    k => key = k.to_string(),
                }
            }
            canonical(m, &key, platform)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Caps to draw (platform order).
pub fn caps(shortcut: &str, platform: Platform) -> Vec<String> {
    parse_shortcut(shortcut, platform)
        .into_iter()
        .flatten()
        .map(|k| k.glyph)
        .collect()
}

/// Spoken form ("Command Shift K", "G then I").
pub fn spoken(shortcut: &str, platform: Platform) -> String {
    parse_shortcut(shortcut, platform)
        .into_iter()
        .map(|ch| ch.into_iter().map(|k| k.name).collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join(" then ")
}

fn is_function_key(k: &str) -> bool {
    let k = k.to_lowercase();
    k.len() >= 2 && k.starts_with('f') && k[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n))
}

fn is_modifier(k: &str) -> bool {
    matches!(
        k,
        "Meta" | "Control" | "Alt" | "Shift" | "OS" | "AltGraph" | "CapsLock" | "Fn"
    )
}

/// Combinations the browser or system keeps (Arc `warnReserved`).
pub const RESERVED: [&str; 10] = [
    "mod+w", "mod+q", "mod+t", "mod+n", "mod+c", "mod+v", "mod+x", "mod+z", "mod+a", "mod+tab",
];

/// A shortcut in use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// Shortcut (`mod+k`).
    pub shortcut: String,
    /// What it does.
    pub label: String,
}

impl Binding {
    /// A binding.
    pub fn new(shortcut: &str, label: &str) -> Self {
        Self {
            shortcut: shortcut.into(),
            label: label.into(),
        }
    }
}

/// Recorder state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecState {
    /// Showing the value (or the placeholder).
    Idle,
    /// Listening; held modifiers show as caps.
    Recording {
        /// Held now.
        held: Mods,
        /// First chord of a sequence (Oxyn plus).
        first: Option<String>,
    },
    /// The combination is taken: ask before using it.
    Conflict {
        /// The candidate.
        shortcut: String,
        /// Who has it ("Reserved by the browser" or the binding label).
        with: String,
        /// It is a system / browser reservation.
        reserved: bool,
    },
    /// Rejected with a message (needs a modifier).
    Invalid(String),
}

/// What a key during recording did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecOutcome {
    /// Keep going.
    None,
    /// New value (and the binding it was taken from).
    Changed(Option<String>, Option<Binding>),
    /// Recording cancelled.
    Cancelled,
    /// Tab: stop and move on.
    Left,
}

/// Shortcut recorder (Arc UI).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorder {
    /// Current value.
    pub value: Option<String>,
    /// What reset restores.
    pub reset_value: Option<String>,
    /// State.
    pub state: RecState,
    /// In use.
    pub bindings: Vec<Binding>,
    /// Warn about reserved combinations.
    pub warn_reserved: bool,
    /// Require ⌘, Ctrl or ⌥ (function keys always allowed).
    pub require_modifier: bool,
    /// Platform.
    pub platform: Platform,
    /// Oxyn plus: allow two-chord sequences ("g i").
    pub sequences: bool,
    /// Polite announcement.
    pub message: Option<String>,
}

impl Recorder {
    /// A recorder.
    pub fn new(value: Option<&str>, platform: Platform) -> Self {
        Self {
            value: value.map(Into::into),
            reset_value: value.map(Into::into),
            state: RecState::Idle,
            bindings: vec![],
            warn_reserved: true,
            require_modifier: true,
            platform,
            sequences: false,
            message: None,
        }
    }
    /// Recording now.
    pub fn recording(&self) -> bool {
        matches!(self.state, RecState::Recording { .. })
    }
    /// Enter / Space / click: starts recording.
    pub fn start(&mut self) {
        self.state = RecState::Recording {
            held: Mods::default(),
            first: None,
        };
        self.message = Some("Recording. Press a shortcut, Escape to cancel".into());
    }
    /// A key went down. `key` is the DOM `key` value.
    pub fn key_down(&mut self, key: &str, mods: Mods, now: ClockMs) -> RecOutcome {
        let _ = now;
        let RecState::Recording { first, .. } = self.state.clone() else {
            if matches!(key, "Backspace" | "Delete") && !mods.any() && self.value.is_some() {
                return self.clear();
            }
            return RecOutcome::None;
        };
        if is_modifier(key) {
            self.state = RecState::Recording { held: mods, first };
            return RecOutcome::None;
        }
        match key {
            "Escape" if !mods.any() => {
                self.state = RecState::Idle;
                self.message = Some("Recording cancelled".into());
                return RecOutcome::Cancelled;
            }
            "Tab" if !mods.any() => {
                self.state = RecState::Idle;
                return RecOutcome::Left;
            }
            "Backspace" | "Delete" if !mods.any() && first.is_none() => {
                self.state = RecState::Idle;
                return self.clear();
            }
            _ => {}
        }
        let k = match key {
            " " => "space".to_string(),
            "ArrowUp" => "up".into(),
            "ArrowDown" => "down".into(),
            "ArrowLeft" => "left".into(),
            "ArrowRight" => "right".into(),
            k => k.to_lowercase(),
        };
        let chord = canonical(mods, &k, self.platform);
        // Oxyn plus: a bare letter starts a sequence; the next key finishes it.
        if self.sequences && !mods.any() && k.chars().count() == 1 && first.is_none() {
            self.state = RecState::Recording {
                held: Mods::default(),
                first: Some(chord),
            };
            return RecOutcome::None;
        }
        let candidate = match first {
            Some(f) => format!("{f} {chord}"),
            None => chord,
        };
        let single = !candidate.contains(' ');
        if single && self.require_modifier && !mods.primary() && !is_function_key(&k) {
            let msg = "Use ⌘, Ctrl or ⌥ with another key".to_string();
            self.state = RecState::Invalid(msg.clone());
            self.message = Some(msg);
            return RecOutcome::None;
        }
        self.offer(candidate)
    }
    fn offer(&mut self, s: String) -> RecOutcome {
        if self.warn_reserved && RESERVED.iter().any(|r| normalize(r, self.platform) == s) {
            self.state = RecState::Conflict {
                with: "Reserved by the browser or system".into(),
                shortcut: s,
                reserved: true,
            };
            self.message = Some("That shortcut is reserved. Use anyway or cancel".into());
            return RecOutcome::None;
        }
        if let Some(b) = self.bindings.iter().find(|b| {
            normalize(&b.shortcut, self.platform) == s && self.value.as_deref() != Some(s.as_str())
        }) {
            self.message = Some(format!("Already used by {}. Use anyway or cancel", b.label));
            self.state = RecState::Conflict {
                with: b.label.clone(),
                shortcut: s,
                reserved: false,
            };
            return RecOutcome::None;
        }
        self.set(Some(s), None)
    }
    fn set(&mut self, v: Option<String>, replaced: Option<Binding>) -> RecOutcome {
        self.value = v.clone();
        self.state = RecState::Idle;
        self.message = Some(match &v {
            Some(s) => format!("Shortcut set to {}", spoken(s, self.platform)),
            None => "Shortcut cleared".into(),
        });
        RecOutcome::Changed(v, replaced)
    }
    /// "Use anyway" on a conflict.
    pub fn use_anyway(&mut self) -> RecOutcome {
        let RecState::Conflict { shortcut, .. } = self.state.clone() else {
            return RecOutcome::None;
        };
        let p = self.platform;
        let replaced = self
            .bindings
            .iter()
            .find(|b| normalize(&b.shortcut, p) == shortcut)
            .cloned();
        if replaced.is_some() {
            self.bindings
                .retain(|b| normalize(&b.shortcut, p) != shortcut);
        }
        self.set(Some(shortcut), replaced)
    }
    /// "Cancel" on a conflict.
    pub fn cancel(&mut self) {
        self.state = RecState::Idle;
        self.message = Some("Kept the previous shortcut".into());
    }
    /// Backspace / Delete / the clear button.
    pub fn clear(&mut self) -> RecOutcome {
        self.set(None, None)
    }
    /// The reset button.
    pub fn reset(&mut self) -> RecOutcome {
        let r = self.reset_value.clone();
        self.set(r, None)
    }
    /// Accessible name: label plus the spoken shortcut.
    pub fn aria(&self, label: &str) -> String {
        match &self.value {
            Some(v) => format!("{label}, {}", spoken(v, self.platform)),
            None => format!("{label}, not set"),
        }
    }
}

/// One cheatsheet group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutGroup {
    /// Heading.
    pub label: String,
    /// (label, shortcut).
    pub items: Vec<(String, String)>,
}

/// Cheatsheet filter: rows whose label or spoken shortcut matches every word.
pub fn filter_cheatsheet(
    groups: &[ShortcutGroup],
    q: &str,
    platform: Platform,
) -> Vec<ShortcutGroup> {
    let words: Vec<String> = q.split_whitespace().map(str::to_lowercase).collect();
    groups
        .iter()
        .map(|g| ShortcutGroup {
            label: g.label.clone(),
            items: g
                .items
                .iter()
                .filter(|(l, s)| {
                    let hay = format!("{} {} {}", l, spoken(s, platform), g.label).to_lowercase();
                    words.iter().all(|w| hay.contains(w.as_str()))
                })
                .cloned()
                .collect(),
        })
        .filter(|g| !g.items.is_empty())
        .collect()
}

/// A row fades while modifiers are held that it doesn't use.
pub fn dimmed(shortcut: &str, held: Mods, platform: Platform) -> bool {
    if !held.any() {
        return false;
    }
    let s = shortcut.to_lowercase();
    let mac = platform == Platform::Mac;
    let has = |m: &str| s.split([' ', '+']).any(|p| p == m);
    let meta_mod = if mac {
        has("mod") || has("cmd") || has("meta")
    } else {
        has("meta")
    };
    let ctrl_mod = if mac {
        has("ctrl")
    } else {
        has("mod") || has("ctrl")
    };
    (held.meta && !meta_mod)
        || (held.ctrl && !ctrl_mod)
        || (held.alt && !has("alt"))
        || (held.shift && !has("shift"))
}

/// Oxyn plus: the bindings as a JSON keymap (`{"label": "shortcut", …}`).
pub fn keymap_json(bindings: &[Binding]) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let body: Vec<String> = bindings
        .iter()
        .map(|b| format!("  \"{}\": \"{}\"", esc(&b.label), esc(&b.shortcut)))
        .collect();
    format!("{{\n{}\n}}", body.join(",\n"))
}

/// How long a sequence waits for its second key.
pub const SEQUENCE_WINDOW: Duration = Duration::from_millis(1000);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_filter() {
        let items = vec![
            "Billing".to_string(),
            "Team members".into(),
            "API keys".into(),
        ];
        let r = filter_strings(&items, "mem");
        assert_eq!(r, vec![(1, vec![(5, 8)])]);
        assert_eq!(filter_strings(&items, "").len(), 3);
        assert!(
            clear_visible("x", false) && !clear_visible("", false) && !clear_visible("x", true)
        );
    }

    #[test]
    fn expanding_search_flow() {
        let mut s = ExpandingSearch::new(vec![
            Command::new("p1", "Atlas roadmap").group("Projects"),
            Command::new("d1", "Agent API reference").group("Docs"),
            Command::new("d2", "Atlas onboarding").group("Docs"),
        ]);
        s.suggestions = vec![Command::new("r1", "Atlas roadmap")];
        assert_eq!(s.width(500.0, 36.0), 36.0);
        s.expand();
        assert_eq!(s.width(300.0, 36.0), 300.0);
        assert_eq!(s.flat(), vec![0]);
        s.set_query("atlas");
        assert_eq!(s.flat().len(), 2);
        assert_eq!(s.active, Some(0));
        s.step(true);
        s.step(true);
        assert_eq!(s.active, Some(0), "wraps");
        assert_eq!(s.announce(), "2 results");
        assert_eq!(
            s.enter(),
            SearchOutcome::Select(s.items[s.flat()[0]].id.clone())
        );
        assert!(!s.expanded && s.recent == vec!["atlas"]);
        s.expand();
        s.set_query("zzz");
        assert_eq!(s.announce(), "No results for zzz");
        assert_eq!(s.escape(), SearchOutcome::Folded);
    }

    #[test]
    fn recorder_records_and_conflicts() {
        let mut r = Recorder::new(Some("mod+k"), Platform::Mac);
        r.bindings = vec![Binding::new("Shift+Mod+P", "Command palette")];
        r.start();
        let m = Mods {
            meta: true,
            ..Default::default()
        };
        assert_eq!(r.key_down("Meta", m, ClockMs(0)), RecOutcome::None);
        assert!(matches!(r.state, RecState::Recording { held, .. } if held.meta));
        let ms = Mods {
            meta: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(
            r.key_down("J", ms, ClockMs(0)),
            RecOutcome::Changed(Some("mod+shift+j".into()), None)
        );
        assert_eq!(r.aria("Search"), "Search, Command Shift J");
        r.start();
        r.key_down("p", ms, ClockMs(0));
        assert!(
            matches!(r.state, RecState::Conflict { ref with, reserved: false, .. } if with == "Command palette")
        );
        assert_eq!(
            r.use_anyway(),
            RecOutcome::Changed(
                Some("mod+shift+p".into()),
                Some(Binding::new("Shift+Mod+P", "Command palette"))
            )
        );
        assert!(r.bindings.is_empty());
        r.start();
        r.key_down("w", m, ClockMs(0));
        assert!(matches!(r.state, RecState::Conflict { reserved: true, .. }));
        r.cancel();
        assert_eq!(r.value.as_deref(), Some("mod+shift+p"));
        r.start();
        r.key_down("k", Mods::default(), ClockMs(0));
        assert!(matches!(r.state, RecState::Invalid(_)));
        r.start();
        assert_eq!(
            r.key_down("F5", Mods::default(), ClockMs(0)),
            RecOutcome::Changed(Some("f5".into()), None)
        );
        r.start();
        assert_eq!(
            r.key_down("Escape", Mods::default(), ClockMs(0)),
            RecOutcome::Cancelled
        );
        assert_eq!(
            r.key_down("Backspace", Mods::default(), ClockMs(0)),
            RecOutcome::Changed(None, None)
        );
        assert_eq!(r.reset(), RecOutcome::Changed(Some("mod+k".into()), None));
        assert_eq!(normalize("cmd+shift+k", Platform::Mac), "mod+shift+k");
        assert_eq!(
            normalize("Ctrl+Alt+Delete", Platform::Other),
            "mod+alt+delete"
        );
    }

    #[test]
    fn sequences_and_cheatsheet() {
        let mut r = Recorder::new(None, Platform::Other);
        r.sequences = true;
        r.start();
        assert_eq!(
            r.key_down("g", Mods::default(), ClockMs(0)),
            RecOutcome::None
        );
        assert_eq!(
            r.key_down("i", Mods::default(), ClockMs(10)),
            RecOutcome::Changed(Some("g i".into()), None)
        );
        assert_eq!(spoken("g i", Platform::Other), "G then I");
        assert_eq!(caps("mod+shift+k", Platform::Mac), vec!["⌘", "⇧", "K"]);
        let groups = vec![ShortcutGroup {
            label: "General".into(),
            items: vec![
                ("Search".into(), "mod+k".into()),
                ("New thread".into(), "mod+shift+n".into()),
            ],
        }];
        assert_eq!(
            filter_cheatsheet(&groups, "thread", Platform::Mac)[0]
                .items
                .len(),
            1
        );
        assert_eq!(
            filter_cheatsheet(&groups, "command shift", Platform::Mac)[0]
                .items
                .len(),
            1
        );
        let held = Mods {
            shift: true,
            ..Default::default()
        };
        assert!(
            dimmed("mod+k", held, Platform::Mac) && !dimmed("mod+shift+n", held, Platform::Mac)
        );
        assert!(keymap_json(&[Binding::new("mod+k", "Search")]).contains("\"Search\": \"mod+k\""));
    }
}
