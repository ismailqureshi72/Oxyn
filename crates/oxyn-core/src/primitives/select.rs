//! Select (`select.md`): pick one option in the text field's look. Icons and a second line per
//! option, one gliding highlight, type-ahead, a list that flips to stay on screen, a chevron that
//! turns half a circle. Oxyn plus: a filter field, grouped options, and async options with
//! skeleton rows.

use super::button::Glide;
use super::nav::{NavKey, Typeahead, step};
use crate::ClockMs;

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    /// Value.
    pub value: String,
    /// Label.
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Second line.
    pub description: Option<String>,
    /// Oxyn plus: group heading this option sits under.
    pub group: Option<String>,
    /// Disabled.
    pub disabled: bool,
}

impl SelectOption {
    /// An option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            description: None,
            group: None,
            disabled: false,
        }
    }
    /// With a second line.
    pub fn desc(mut self, d: &str) -> Self {
        self.description = Some(d.into());
        self
    }
    /// With an icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// In a group.
    pub fn group(mut self, g: &str) -> Self {
        self.group = Some(g.into());
        self
    }
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectKey {
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
    /// Space.
    Space,
    /// Esc.
    Escape,
    /// Tab.
    Tab,
    /// Printable.
    Char(char),
    /// Backspace (filter field).
    Backspace,
}

/// Which way the shown value moved, so the new label rises from below (later option) or drops
/// from above (earlier option) (Arc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueMove {
    /// Picked something further down.
    Later,
    /// Picked something further up.
    Earlier,
}

/// A row in the open list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListRow {
    /// Group heading.
    Group(String),
    /// Option `i` of [`Select::options`].
    Option(usize),
    /// Oxyn plus: placeholder row while options load.
    Skeleton,
    /// Nothing matches the filter.
    Empty,
}

/// The select.
#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    /// Options.
    pub options: Vec<SelectOption>,
    /// Chosen value (`None` shows the placeholder).
    pub value: Option<String>,
    /// Placeholder (`Choose…`).
    pub placeholder: String,
    /// Required.
    pub required: bool,
    /// Disabled.
    pub disabled: bool,
    /// Open.
    pub open: bool,
    /// Highlight over the visible options (indices into [`Select::visible`]).
    pub glide: Glide,
    /// Oxyn plus: filter field.
    pub searchable: bool,
    /// Filter text.
    pub query: String,
    /// Oxyn plus: options are loading.
    pub loading: bool,
    /// Last value move and when.
    pub moved: Option<(ValueMove, ClockMs)>,
    /// When the list opened (enter animation, chevron turn).
    pub opened_at: ClockMs,
    typeahead: Typeahead,
}

impl Select {
    /// A select.
    pub fn new(options: Vec<SelectOption>, value: Option<&str>) -> Self {
        Self {
            options,
            value: value.map(str::to_string),
            placeholder: "Choose…".into(),
            required: false,
            disabled: false,
            open: false,
            glide: Glide::default(),
            searchable: false,
            query: String::new(),
            loading: false,
            moved: None,
            opened_at: ClockMs(0),
            typeahead: Typeahead::default(),
        }
    }
    /// The chosen option.
    pub fn selected(&self) -> Option<&SelectOption> {
        let v = self.value.as_deref()?;
        self.options.iter().find(|o| o.value == v)
    }
    /// The trigger text.
    pub fn display(&self) -> &str {
        self.selected()
            .map(|o| o.label.as_str())
            .unwrap_or(&self.placeholder)
    }
    /// Option indices that pass the filter.
    pub fn visible(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        (0..self.options.len())
            .filter(|&i| q.is_empty() || self.options[i].label.to_lowercase().contains(&q))
            .collect()
    }
    /// Rows for the list, with group headings, skeletons or the empty row.
    pub fn rows(&self) -> Vec<ListRow> {
        if self.loading {
            return vec![ListRow::Skeleton; 3];
        }
        let vis = self.visible();
        if vis.is_empty() {
            return vec![ListRow::Empty];
        }
        let mut out = vec![];
        let mut group: Option<&str> = None;
        for i in vis {
            let g = self.options[i].group.as_deref();
            if g.is_some() && g != group {
                out.push(ListRow::Group(g.unwrap_or_default().into()));
            }
            group = g;
            out.push(ListRow::Option(i));
        }
        out
    }
    /// The highlighted option index (into `options`).
    pub fn highlighted(&self) -> Option<usize> {
        self.visible().get(self.glide.target?).copied()
    }
    fn vis_enabled(&self) -> (Vec<usize>, Vec<bool>) {
        let v = self.visible();
        let e = v.iter().map(|&i| !self.options[i].disabled).collect();
        (v, e)
    }
    /// Opens, highlighting the chosen option (or the first enabled one).
    pub fn open_list(&mut self, now: ClockMs) {
        if self.disabled {
            return;
        }
        self.open = true;
        self.opened_at = now;
        self.query.clear();
        let (v, e) = self.vis_enabled();
        let sel = self
            .value
            .as_deref()
            .and_then(|val| v.iter().position(|&i| self.options[i].value == val));
        let to = sel.or_else(|| step(None, NavKey::First, v.len(), true, |i| e[i]));
        self.glide = Glide::default();
        self.glide.go(to, now, true);
    }
    /// Closes without choosing.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.typeahead.reset();
    }
    /// Choose option `i` (into `options`). Returns the new value when it changed.
    pub fn choose(&mut self, i: usize, now: ClockMs) -> Option<String> {
        let o = self.options.get(i)?.clone();
        if o.disabled {
            return None;
        }
        let old = self
            .selected()
            .and_then(|s| self.options.iter().position(|x| x == s));
        self.close();
        if self.value.as_deref() == Some(&o.value) {
            return None;
        }
        let v = o.value;
        if let Some(old) = old {
            self.moved = Some((
                if i > old {
                    ValueMove::Later
                } else {
                    ValueMove::Earlier
                },
                now,
            ));
        }
        self.value = Some(v.clone());
        Some(v)
    }
    /// Pointer over visible row `k` (glides).
    pub fn hover(&mut self, k: Option<usize>, now: ClockMs) {
        let (_, e) = self.vis_enabled();
        let k = k.filter(|&k| e.get(k).copied().unwrap_or(false));
        self.glide.go(k, now, false);
    }
    /// Keyboard. Returns the new value when the choice changed.
    pub fn key(&mut self, k: SelectKey, now: ClockMs) -> Option<String> {
        if self.disabled {
            return None;
        }
        if !self.open {
            match k {
                SelectKey::Down | SelectKey::Up | SelectKey::Enter | SelectKey::Space => {
                    self.open_list(now)
                }
                // Closed type-ahead changes the value directly (native select).
                SelectKey::Char(c) => {
                    let cur = self
                        .selected()
                        .and_then(|s| self.options.iter().position(|x| x == s));
                    let to = self.typeahead.feed(
                        c,
                        now,
                        cur,
                        self.options.iter().map(|o| (o.label.as_str(), !o.disabled)),
                    )?;
                    let r = self.choose(to, now);
                    return r;
                }
                _ => {}
            }
            return None;
        }
        let (v, e) = self.vis_enabled();
        let cur = self.glide.target;
        let mv = |key| step(cur, key, v.len(), false, |i| e[i]);
        match k {
            SelectKey::Down => self.glide.go(mv(NavKey::Next), now, true),
            SelectKey::Up => self.glide.go(mv(NavKey::Prev), now, true),
            SelectKey::Home => self.glide.go(mv(NavKey::First), now, true),
            SelectKey::End => self.glide.go(mv(NavKey::Last), now, true),
            SelectKey::Enter | SelectKey::Space if !(self.searchable && k == SelectKey::Space) => {
                let i = self.highlighted()?;
                return self.choose(i, now);
            }
            SelectKey::Escape | SelectKey::Tab => self.close(),
            SelectKey::Space if self.searchable => {
                self.query.push(' ');
                self.refilter(now);
            }
            SelectKey::Char(c) if self.searchable => {
                self.query.push(c);
                self.refilter(now);
            }
            SelectKey::Backspace if self.searchable => {
                self.query.pop();
                self.refilter(now);
            }
            SelectKey::Char(c) => {
                let labels: Vec<(String, bool)> = v
                    .iter()
                    .zip(e.iter())
                    .map(|(&i, &en)| (self.options[i].label.clone(), en))
                    .collect();
                let to = self.typeahead.feed(
                    c,
                    now,
                    cur,
                    labels.iter().map(|(l, en)| (l.as_str(), *en)),
                );
                if to.is_some() {
                    self.glide.go(to, now, true);
                }
            }
            _ => {}
        }
        None
    }
    fn refilter(&mut self, now: ClockMs) {
        let (v, e) = self.vis_enabled();
        let to = step(None, NavKey::First, v.len(), true, |i| e[i]);
        self.glide = Glide::default();
        self.glide.go(to, now, true);
    }
    /// Validation message for a required select with no value.
    pub fn validate(&self) -> Option<&'static str> {
        (self.required && self.value.is_none()).then_some("Choose an option")
    }
    /// Chevron rotation in degrees (half a circle when open).
    pub fn chevron(&self) -> f32 {
        if self.open { 180.0 } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel() -> Select {
        let mut o = vec![
            SelectOption::new("haiku", "Haiku")
                .desc("Fast")
                .group("Anthropic"),
            SelectOption::new("sonnet", "Sonnet")
                .desc("Balanced")
                .group("Anthropic"),
            SelectOption::new("opus", "Opus").group("Anthropic"),
            SelectOption::new("mini", "Mini").group("OpenAI"),
            SelectOption::new("o3", "o3").group("OpenAI"),
        ];
        o[2].disabled = true;
        Select::new(o, None)
    }

    #[test]
    fn placeholder_open_navigate_choose() {
        let mut s = sel();
        assert_eq!(s.display(), "Choose…");
        s.key(SelectKey::Down, ClockMs(0));
        assert!(s.open && s.chevron() == 180.0);
        assert_eq!(s.highlighted(), Some(0));
        s.key(SelectKey::Down, ClockMs(1));
        s.key(SelectKey::Down, ClockMs(2));
        assert_eq!(s.highlighted(), Some(3), "disabled Opus skipped");
        assert_eq!(s.key(SelectKey::Enter, ClockMs(3)).as_deref(), Some("mini"));
        assert!(!s.open);
        s.open_list(ClockMs(4));
        assert_eq!(s.highlighted(), Some(3), "opens on the chosen option");
        s.key(SelectKey::Home, ClockMs(5));
        s.key(SelectKey::Enter, ClockMs(6));
        assert_eq!(s.moved.map(|m| m.0), Some(ValueMove::Earlier));
    }

    #[test]
    fn groups_typeahead_escape() {
        let mut s = sel();
        let rows = s.rows();
        assert_eq!(rows[0], ListRow::Group("Anthropic".into()));
        assert_eq!(rows[4], ListRow::Group("OpenAI".into()));
        assert_eq!(
            s.key(SelectKey::Char('s'), ClockMs(0)).as_deref(),
            Some("sonnet"),
            "closed type-ahead picks"
        );
        s.open_list(ClockMs(1));
        s.key(SelectKey::Char('m'), ClockMs(2));
        assert_eq!(s.highlighted(), Some(3));
        s.key(SelectKey::Escape, ClockMs(3));
        assert!(!s.open);
        assert_eq!(s.value.as_deref(), Some("sonnet"));
        s.required = true;
        s.value = None;
        assert_eq!(s.validate(), Some("Choose an option"));
    }

    #[test]
    fn searchable_and_loading() {
        let mut s = sel();
        s.searchable = true;
        s.open_list(ClockMs(0));
        s.key(SelectKey::Char('o'), ClockMs(1));
        assert_eq!(s.visible(), vec![1, 2, 4]);
        assert_eq!(s.highlighted(), Some(1), "first enabled match");
        s.key(SelectKey::Char('x'), ClockMs(2));
        assert_eq!(s.rows(), vec![ListRow::Empty]);
        s.key(SelectKey::Backspace, ClockMs(3));
        assert_eq!(s.visible().len(), 3);
        s.loading = true;
        assert_eq!(s.rows().len(), 3);
    }
}
