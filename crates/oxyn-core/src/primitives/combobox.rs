//! Combobox and multi-select (`combobox.md`, `multi-select.md`): pick one or many from a list.
//!
//! - The **combobox** (affectUI Combobox) is searchable: grouped options with avatars and hints,
//!   highlighted matches, chips for many, create-on-the-fly, `+N` past `max_visible_tags`, and the
//!   combobox keyboard (↑ ↓ ↵ ⌫ Esc Tab).
//! - The **multi-select** (Arc UI Multi-select) is the same model with no search field: a trigger
//!   with chips and a rolling `+N`, a listbox with drawn checks and a Clear button.
//!
//! Rebuilt from the public docs, no source used. Oxyn plus: match modes (contains / prefix /
//! fuzzy), a max selection with a reason, Select all within the filter, and an async source with a
//! debounce window.

use crate::ClockMs;

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboOption {
    /// Value.
    pub value: String,
    /// Label.
    pub label: String,
    /// Second line / hint.
    pub hint: Option<String>,
    /// Show an initials avatar on its chip and row.
    pub avatar: bool,
    /// Icon name.
    pub icon: Option<String>,
    /// Group title.
    pub group: Option<String>,
    /// Extra words that match.
    pub keywords: Vec<String>,
    /// Disabled.
    pub disabled: bool,
}

impl ComboOption {
    /// An option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            hint: None,
            avatar: false,
            icon: None,
            group: None,
            keywords: vec![],
            disabled: false,
        }
    }
    /// With a hint.
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.into());
        self
    }
    /// In a group.
    pub fn group(mut self, g: &str) -> Self {
        self.group = Some(g.into());
        self
    }
    /// With an avatar.
    pub fn avatar(mut self) -> Self {
        self.avatar = true;
        self
    }
    /// With an icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// With keywords.
    pub fn keywords(mut self, k: &[&str]) -> Self {
        self.keywords = k.iter().map(|s| (*s).into()).collect();
        self
    }
    /// Initials for the avatar (`Ada Lovelace` → `AL`).
    pub fn initials(&self) -> String {
        self.label
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect()
    }
}

/// How typed text matches labels (Oxyn plus; the source matches anywhere).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MatchMode {
    /// Anywhere in the label or keywords (source).
    #[default]
    Contains,
    /// At the start of a word.
    Prefix,
    /// Letters in order (`gpt4o` finds `GPT-4o`).
    Fuzzy,
}

impl MatchMode {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "prefix" => Self::Prefix,
            "fuzzy" => Self::Fuzzy,
            _ => Self::Contains,
        }
    }
}

/// Char ranges of `label` that match `q` (for highlighting), or `None` when it doesn't match.
pub fn match_ranges(label: &str, q: &str, mode: MatchMode) -> Option<Vec<(usize, usize)>> {
    let q: Vec<char> = q.trim().to_lowercase().chars().collect();
    if q.is_empty() {
        return Some(vec![]);
    }
    let l: Vec<char> = label.to_lowercase().chars().collect();
    let find = |from: usize| -> Option<usize> {
        (from..=l.len().saturating_sub(q.len())).find(|&i| l[i..].starts_with(&q))
    };
    match mode {
        MatchMode::Contains => find(0).map(|i| vec![(i, i + q.len())]),
        MatchMode::Prefix => {
            let mut i = 0;
            while let Some(at) = find(i) {
                if at == 0 || !l[at - 1].is_alphanumeric() {
                    return Some(vec![(at, at + q.len())]);
                }
                i = at + 1;
            }
            None
        }
        MatchMode::Fuzzy => {
            let mut out: Vec<(usize, usize)> = vec![];
            let mut qi = 0;
            for (i, c) in l.iter().enumerate() {
                if qi < q.len() && *c == q[qi] {
                    match out.last_mut() {
                        Some(r) if r.1 == i => r.1 = i + 1,
                        _ => out.push((i, i + 1)),
                    }
                    qi += 1;
                } else if qi < q.len() && !c.is_alphanumeric() && !q[qi].is_alphanumeric() {
                    continue;
                }
            }
            // Skip punctuation in the query too ("gpt4o" vs "gpt-4o").
            let needed = q.iter().filter(|c| c.is_alphanumeric()).count();
            let got = out.iter().map(|r| r.1 - r.0).sum::<usize>();
            (qi == q.len() || got >= needed).then_some(out)
        }
    }
}

/// A row of the open list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComboRow {
    /// Group title.
    Group(String),
    /// Option `i` with the matched char ranges.
    Option(usize, Vec<(usize, usize)>),
    /// Create “text”.
    Create(String),
    /// Oxyn plus: Select all within the filter (`n` options).
    SelectAll(usize),
    /// Loading (async source).
    Loading,
    /// Nothing matches.
    Empty,
}

impl ComboRow {
    /// Can be highlighted.
    pub fn active(&self) -> bool {
        matches!(
            self,
            Self::Option(..) | Self::Create(_) | Self::SelectAll(_)
        )
    }
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComboKey {
    /// ↑.
    Up,
    /// ↓.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Enter.
    Enter,
    /// Space (multi-select trigger only; types a space in the search).
    Space,
    /// Backspace in an empty field.
    Backspace,
    /// Esc.
    Escape,
    /// Tab.
    Tab,
}

/// What changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComboEvent {
    /// The selection is now this.
    Changed(Vec<String>),
    /// A new option was created with this text (and picked).
    Created(String),
    /// Nothing.
    None,
}

/// The combobox / multi-select.
#[derive(Clone, Debug, PartialEq)]
pub struct Combobox {
    /// Options (created ones are appended).
    pub options: Vec<ComboOption>,
    /// Picked values in pick order.
    pub value: Vec<String>,
    /// Many (chips) or one.
    pub multiple: bool,
    /// Typed text.
    pub query: String,
    /// Has a search field (combobox) or not (multi-select).
    pub searchable: bool,
    /// Open.
    pub open: bool,
    /// Highlighted row (index into [`Combobox::rows`]).
    pub active: Option<usize>,
    /// Offer Create “…”.
    pub creatable: bool,
    /// Create row text; `…` is replaced by the typed text.
    pub create_label: String,
    /// Chips shown before `+N`.
    pub max_visible: Option<usize>,
    /// Empty text.
    pub empty_text: String,
    /// Chip marked by the first Backspace.
    pub chip_marked: Option<usize>,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: match mode.
    pub mode: MatchMode,
    /// Oxyn plus: max picks and the reason shown when reached.
    pub max_selected: Option<(usize, String)>,
    /// Oxyn plus: offer Select all within the filter.
    pub select_all: bool,
    /// Oxyn plus: async source — the last keystroke, and the debounce window in ms.
    pub debounce: Option<(ClockMs, u64)>,
    /// Chip just removed (folds after a short flash) and when.
    pub removed: Option<(String, ClockMs)>,
    /// When the `+N` count last changed and its previous value (rolls).
    pub count_roll: Option<(usize, ClockMs)>,
    /// Live-region line.
    pub announce: String,
}

impl Combobox {
    /// A searchable combobox (many by default, as affectUI).
    pub fn new(options: Vec<ComboOption>, value: &[&str]) -> Self {
        Self {
            options,
            value: value.iter().map(|v| (*v).into()).collect(),
            multiple: true,
            query: String::new(),
            searchable: true,
            open: false,
            active: None,
            creatable: false,
            create_label: "Create “…”".into(),
            max_visible: None,
            empty_text: "No results".into(),
            chip_marked: None,
            disabled: false,
            mode: MatchMode::Contains,
            max_selected: None,
            select_all: false,
            debounce: None,
            removed: None,
            count_roll: None,
            announce: String::new(),
        }
    }
    /// A multi-select (no search, `+N` after 2 chips, as Arc).
    pub fn multi_select(options: Vec<ComboOption>, value: &[&str]) -> Self {
        Self {
            searchable: false,
            max_visible: Some(2),
            ..Self::new(options, value)
        }
    }
    /// Option index for a value.
    pub fn index_of(&self, v: &str) -> Option<usize> {
        self.options.iter().position(|o| o.value == v)
    }
    /// Picked options in pick order.
    pub fn selected(&self) -> Vec<&ComboOption> {
        self.value
            .iter()
            .filter_map(|v| self.index_of(v).map(|i| &self.options[i]))
            .collect()
    }
    /// Is option `i` picked.
    pub fn is_selected(&self, i: usize) -> bool {
        self.value.contains(&self.options[i].value)
    }
    fn matches(&self, o: &ComboOption) -> Option<Vec<(usize, usize)>> {
        let q = if self.searchable {
            self.query.as_str()
        } else {
            ""
        };
        match_ranges(&o.label, q, self.mode).or_else(|| {
            o.keywords
                .iter()
                .any(|k| match_ranges(k, q, self.mode).is_some())
                .then(Vec::new)
        })
    }
    /// Waiting on the async source.
    pub fn loading(&self, now: ClockMs) -> bool {
        self.debounce.is_some_and(|(t, ms)| {
            !self.query.is_empty() && now.since(t).as_millis() < u128::from(ms)
        })
    }
    /// The open list's rows.
    pub fn rows_at(&self, now: ClockMs) -> Vec<ComboRow> {
        if self.loading(now) {
            return vec![ComboRow::Loading];
        }
        let hits: Vec<(usize, Vec<(usize, usize)>)> = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(i, o)| self.matches(o).map(|r| (i, r)))
            .collect();
        let mut out = vec![];
        if self.select_all && self.multiple && hits.len() > 1 {
            out.push(ComboRow::SelectAll(hits.len()));
        }
        let mut group: Option<&str> = None;
        for (i, r) in &hits {
            let g = self.options[*i].group.as_deref();
            if let Some(gg) = g
                && g != group
            {
                out.push(ComboRow::Group(gg.into()));
            }
            group = g;
            out.push(ComboRow::Option(*i, r.clone()));
        }
        let q = self.query.trim();
        if self.creatable
            && self.searchable
            && !q.is_empty()
            && !self.options.iter().any(|o| o.label.eq_ignore_ascii_case(q))
        {
            out.push(ComboRow::Create(q.into()));
        }
        if !out.iter().any(ComboRow::active) {
            out.retain(|r| !matches!(r, ComboRow::Group(_)));
            out.push(ComboRow::Empty);
        }
        out
    }
    /// Rows with no async source pending.
    pub fn rows(&self) -> Vec<ComboRow> {
        self.rows_at(ClockMs(u64::MAX))
    }
    /// The create row's text.
    pub fn create_text(&self, q: &str) -> String {
        self.create_label.replace('…', q)
    }
    fn first_active(&self) -> Option<usize> {
        let rows = self.rows();
        // Prefer the first picked option for a single select, else the first active row.
        if !self.multiple
            && let Some(v) = self.value.first()
            && let Some(p) = rows
                .iter()
                .position(|r| matches!(r, ComboRow::Option(i, _) if self.options[*i].value == *v))
        {
            return Some(p);
        }
        rows.iter().position(|r| match r {
            ComboRow::Option(i, _) => !self.options[*i].disabled,
            r => r.active(),
        })
    }
    /// Opens.
    pub fn open_list(&mut self) {
        if self.disabled {
            return;
        }
        self.open = true;
        self.active = self.first_active();
    }
    /// Closes.
    pub fn close(&mut self) {
        self.open = false;
        self.active = None;
    }
    /// Typing in the search field: opens and filters, the first match is highlighted.
    pub fn input(&mut self, q: &str, now: ClockMs) {
        if self.disabled {
            return;
        }
        self.query = q.into();
        self.chip_marked = None;
        self.open = true;
        if let Some((_, ms)) = self.debounce {
            self.debounce = Some((now, ms));
        }
        let rows = self.rows();
        self.active = rows.iter().position(|r| match r {
            ComboRow::Option(i, _) => !self.options[*i].disabled,
            ComboRow::Create(_) => true,
            _ => false,
        });
    }
    fn set_count(&mut self, before: usize, now: ClockMs) {
        if let Some(m) = self.max_visible {
            let old = before.saturating_sub(m);
            let new = self.value.len().saturating_sub(m);
            if old != new {
                self.count_roll = Some((old, now));
            }
        }
    }
    fn limit_reached(&self) -> bool {
        self.max_selected
            .as_ref()
            .is_some_and(|(n, _)| self.value.len() >= *n)
    }
    /// Picks or unpicks option `i`.
    pub fn toggle(&mut self, i: usize, now: ClockMs) -> ComboEvent {
        let Some(o) = self.options.get(i).cloned() else {
            return ComboEvent::None;
        };
        if o.disabled || self.disabled {
            return ComboEvent::None;
        }
        let before = self.value.len();
        if let Some(p) = self.value.iter().position(|v| *v == o.value) {
            if !self.multiple {
                self.close();
                self.query.clear();
                return ComboEvent::None;
            }
            self.value.remove(p);
            self.removed = Some((o.value.clone(), now));
            self.announce = format!("Removed {}, {} selected", o.label, self.value.len());
        } else {
            if self.multiple && self.limit_reached() {
                self.announce = self
                    .max_selected
                    .as_ref()
                    .map(|m| m.1.clone())
                    .unwrap_or_default();
                return ComboEvent::None;
            }
            if self.multiple {
                self.value.push(o.value.clone());
            } else {
                self.value = vec![o.value.clone()];
            }
            self.announce = if self.multiple {
                format!("Added {}, {} selected", o.label, self.value.len())
            } else {
                format!("{} selected", o.label)
            };
        }
        self.set_count(before, now);
        if self.searchable {
            self.query.clear();
        }
        if !self.multiple {
            self.close();
        }
        ComboEvent::Changed(self.value.clone())
    }
    /// Creates an option from the typed text and picks it.
    pub fn create(&mut self, now: ClockMs) -> ComboEvent {
        let q = self.query.trim().to_string();
        if q.is_empty() || !self.creatable {
            return ComboEvent::None;
        }
        self.options.push(ComboOption::new(&q, &q));
        let i = self.options.len() - 1;
        self.toggle(i, now);
        self.announce = format!("Created {q}");
        ComboEvent::Created(q)
    }
    /// Select all / none within the filter (Oxyn plus).
    pub fn toggle_all(&mut self, now: ClockMs) -> ComboEvent {
        let hits: Vec<usize> = self
            .rows()
            .iter()
            .filter_map(|r| match r {
                ComboRow::Option(i, _) if !self.options[*i].disabled => Some(*i),
                _ => None,
            })
            .collect();
        let before = self.value.len();
        let all = hits.iter().all(|&i| self.is_selected(i));
        for i in hits {
            let v = self.options[i].value.clone();
            let has = self.value.contains(&v);
            if all && has {
                self.value.retain(|x| *x != v);
            } else if !all && !has && !self.limit_reached() {
                self.value.push(v);
            }
        }
        self.set_count(before, now);
        ComboEvent::Changed(self.value.clone())
    }
    /// Removes a chip (its ×).
    pub fn remove(&mut self, value: &str, now: ClockMs) -> ComboEvent {
        match self.index_of(value) {
            Some(i) if self.is_selected(i) => {
                self.chip_marked = None;
                if !self.multiple {
                    self.value.clear();
                    return ComboEvent::Changed(vec![]);
                }
                self.toggle(i, now)
            }
            _ => ComboEvent::None,
        }
    }
    /// Clear button.
    pub fn clear(&mut self, now: ClockMs) -> ComboEvent {
        let before = self.value.len();
        self.value.clear();
        self.set_count(before, now);
        self.announce = "Cleared selections".into();
        ComboEvent::Changed(vec![])
    }
    /// Activates the highlighted row.
    pub fn activate(&mut self, now: ClockMs) -> ComboEvent {
        let rows = self.rows();
        match self.active.and_then(|a| rows.get(a)).cloned() {
            Some(ComboRow::Option(i, _)) => self.toggle(i, now),
            Some(ComboRow::Create(_)) => self.create(now),
            Some(ComboRow::SelectAll(_)) => self.toggle_all(now),
            _ => ComboEvent::None,
        }
    }
    /// Pointer over row `r`.
    pub fn hover(&mut self, r: usize) {
        if self.rows().get(r).is_some_and(|x| {
            matches!(x, ComboRow::Option(i, _) if !self.options[*i].disabled)
                || matches!(x, ComboRow::Create(_) | ComboRow::SelectAll(_))
        }) {
            self.active = Some(r);
        }
    }
    fn step(&mut self, fwd: bool, wrap: bool) {
        let rows = self.rows();
        let ok = |r: &ComboRow| match r {
            ComboRow::Option(i, _) => !self.options[*i].disabled,
            r => r.active(),
        };
        let idx: Vec<usize> = (0..rows.len()).filter(|&i| ok(&rows[i])).collect();
        if idx.is_empty() {
            self.active = None;
            return;
        }
        let pos = self.active.and_then(|a| idx.iter().position(|&x| x == a));
        let next = match (pos, fwd) {
            (None, true) => 0,
            (None, false) => idx.len() - 1,
            (Some(p), true) if p + 1 < idx.len() => p + 1,
            (Some(_), true) => {
                if wrap {
                    0
                } else {
                    idx.len() - 1
                }
            }
            (Some(0), false) => {
                if wrap {
                    idx.len() - 1
                } else {
                    0
                }
            }
            (Some(p), false) => p - 1,
        };
        self.active = Some(idx[next]);
    }
    /// Keyboard.
    pub fn key(&mut self, k: ComboKey, now: ClockMs) -> ComboEvent {
        if self.disabled {
            return ComboEvent::None;
        }
        let wrap = !self.searchable; // Arc wraps; affectUI stops at the ends.
        match k {
            ComboKey::Down | ComboKey::Up => {
                if !self.open {
                    self.open_list();
                    return ComboEvent::None;
                }
                self.step(k == ComboKey::Down, wrap);
                ComboEvent::None
            }
            ComboKey::Home | ComboKey::End if self.open => {
                let rows = self.rows();
                let it = (0..rows.len()).filter(|&i| match &rows[i] {
                    ComboRow::Option(o, _) => !self.options[*o].disabled,
                    r => r.active(),
                });
                self.active = if k == ComboKey::Home {
                    it.min()
                } else {
                    it.max()
                };
                ComboEvent::None
            }
            ComboKey::Home | ComboKey::End => ComboEvent::None,
            ComboKey::Enter => {
                if !self.open {
                    self.open_list();
                    return ComboEvent::None;
                }
                self.activate(now)
            }
            ComboKey::Space if !self.searchable => {
                if self.open {
                    self.close();
                } else {
                    self.open_list();
                }
                ComboEvent::None
            }
            ComboKey::Space => ComboEvent::None,
            ComboKey::Backspace => {
                if self.searchable && !self.query.is_empty() {
                    return ComboEvent::None;
                }
                match self.chip_marked {
                    Some(c) => {
                        let v = self.value.get(c).cloned().unwrap_or_default();
                        self.chip_marked = None;
                        self.remove(&v, now)
                    }
                    None => {
                        if !self.value.is_empty() {
                            self.chip_marked = Some(self.value.len() - 1);
                        }
                        ComboEvent::None
                    }
                }
            }
            ComboKey::Escape => {
                if self.open {
                    self.close();
                } else {
                    self.query.clear();
                    self.chip_marked = None;
                }
                ComboEvent::None
            }
            ComboKey::Tab => {
                self.close();
                ComboEvent::None
            }
        }
    }
    /// Chips shown and the `+N` overflow count.
    pub fn chips(&self) -> (Vec<&ComboOption>, usize) {
        let sel = self.selected();
        match self.max_visible {
            Some(m) if sel.len() > m => {
                let extra = sel.len() - m;
                (sel.into_iter().take(m).collect(), extra)
            }
            _ => (sel, 0),
        }
    }
    /// Spoken summary of the picks (the trigger's hidden name in the multi-select).
    pub fn summary(&self, placeholder: &str) -> String {
        let s: Vec<&str> = self.selected().iter().map(|o| o.label.as_str()).collect();
        if s.is_empty() {
            placeholder.into()
        } else {
            s.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn people() -> Vec<ComboOption> {
        vec![
            ComboOption::new("ada", "Ada Lovelace")
                .group("Team")
                .avatar(),
            ComboOption::new("grace", "Grace Hopper")
                .group("Team")
                .avatar(),
            ComboOption::new("alan", "Alan Turing").group("Guests"),
            ComboOption::new("gpt", "GPT-4o")
                .group("Agents")
                .keywords(&["openai"]),
        ]
    }

    #[test]
    fn matching_modes() {
        assert_eq!(
            match_ranges("Ada Lovelace", "love", MatchMode::Contains),
            Some(vec![(4, 8)])
        );
        assert_eq!(match_ranges("Ada Lovelace", "ove", MatchMode::Prefix), None);
        assert_eq!(
            match_ranges("Ada Lovelace", "lov", MatchMode::Prefix),
            Some(vec![(4, 7)])
        );
        assert!(match_ranges("GPT-4o", "gpt4o", MatchMode::Fuzzy).is_some());
        assert!(match_ranges("GPT-4o", "xyz", MatchMode::Fuzzy).is_none());
    }

    #[test]
    fn filters_groups_and_creates() {
        let mut c = Combobox::new(people(), &[]);
        c.creatable = true;
        c.input("a", ClockMs(0));
        let rows = c.rows();
        assert_eq!(rows[0], ComboRow::Group("Team".into()));
        assert!(matches!(rows[c.active.unwrap()], ComboRow::Option(0, _)));
        c.input("openai", ClockMs(1));
        assert!(
            matches!(c.rows()[1], ComboRow::Option(3, _)),
            "keyword match"
        );
        c.input("Linus", ClockMs(2));
        assert_eq!(c.rows().last(), Some(&ComboRow::Create("Linus".into())));
        assert_eq!(c.create_text("Linus"), "Create “Linus”");
        assert_eq!(
            c.key(ComboKey::Enter, ClockMs(3)),
            ComboEvent::Created("Linus".into())
        );
        assert_eq!(c.value, ["Linus"]);
        assert_eq!(c.query, "");
        c.creatable = false;
        c.input("zzz", ClockMs(4));
        assert_eq!(c.rows(), vec![ComboRow::Empty]);
    }

    #[test]
    fn keyboard_chips_and_overflow() {
        let mut c = Combobox::new(people(), &["ada", "grace", "alan"]);
        c.max_visible = Some(2);
        assert_eq!(c.chips().1, 1);
        assert_eq!(c.key(ComboKey::Backspace, ClockMs(0)), ComboEvent::None);
        assert_eq!(c.chip_marked, Some(2));
        assert_eq!(
            c.key(ComboKey::Backspace, ClockMs(1)),
            ComboEvent::Changed(vec!["ada".into(), "grace".into()])
        );
        assert_eq!(c.count_roll.map(|r| r.0), Some(1));
        c.key(ComboKey::Down, ClockMs(2));
        assert!(c.open);
        c.key(ComboKey::End, ClockMs(3));
        assert!(matches!(
            c.rows()[c.active.unwrap()],
            ComboRow::Option(3, _)
        ));
        c.key(ComboKey::Escape, ClockMs(4));
        assert!(!c.open);
        c.single();
    }

    impl Combobox {
        fn single(&mut self) {
            self.multiple = false;
            self.value.clear();
            self.open_list();
            self.key(ComboKey::Down, ClockMs(10));
            let ev = self.key(ComboKey::Enter, ClockMs(11));
            assert_eq!(ev, ComboEvent::Changed(vec!["grace".into()]));
            assert!(!self.open, "single closes after a pick");
        }
    }

    #[test]
    fn multi_select_limits_select_all_and_debounce() {
        let mut m = Combobox::multi_select(people(), &[]);
        assert!(!m.searchable);
        m.key(ComboKey::Space, ClockMs(0));
        assert!(m.open);
        m.key(ComboKey::Up, ClockMs(1));
        assert!(
            matches!(m.rows()[m.active.unwrap()], ComboRow::Option(3, _)),
            "wraps to the last option"
        );
        m.max_selected = Some((2, "Pick up to 2".into()));
        m.toggle(0, ClockMs(2));
        m.toggle(1, ClockMs(3));
        assert_eq!(m.toggle(2, ClockMs(4)), ComboEvent::None);
        assert_eq!(m.announce, "Pick up to 2");
        m.max_selected = None;
        m.select_all = true;
        assert_eq!(m.rows()[0], ComboRow::SelectAll(4));
        m.toggle_all(ClockMs(5));
        assert_eq!(m.value.len(), 4);
        assert_eq!(m.chips().1, 2);
        m.toggle_all(ClockMs(6));
        assert!(m.value.is_empty());
        assert_eq!(m.summary("Select options"), "Select options");
        let mut a = Combobox::new(people(), &[]);
        a.debounce = Some((ClockMs(0), 250));
        a.input("gr", ClockMs(100));
        assert_eq!(a.rows_at(ClockMs(200)), vec![ComboRow::Loading]);
        assert!(matches!(a.rows_at(ClockMs(400))[1], ComboRow::Option(1, _)));
        assert_eq!(ComboOption::new("x", "Ada Lovelace").initials(), "AL");
    }
}
