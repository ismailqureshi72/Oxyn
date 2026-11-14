//! Filter menu (`filter-menu.md`): grouped filters with removable chips, collapsible checkbox
//! groups, Clear all and one roving tab stop over headers and options.
//!
//! Behaviour follows the affectUI Filter Menu docs (rebuilt, no source used). Oxyn plus: per-group
//! AND / OR logic, saved presets, and a per-option result count.

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterOption {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Oxyn plus: how many results it matches.
    pub count: Option<u32>,
}

/// One group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterGroup {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Options.
    pub options: Vec<FilterOption>,
    /// Chip prefix (`Status: Open`); defaults to the label.
    pub chip_prefix: Option<String>,
}

impl FilterGroup {
    /// A group from `(id, label)` pairs.
    pub fn new(id: &str, label: &str, options: &[(&str, &str)]) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: options
                .iter()
                .map(|(i, l)| FilterOption {
                    id: (*i).into(),
                    label: (*l).into(),
                    count: None,
                })
                .collect(),
            chip_prefix: None,
        }
    }
    /// With counts per option, in order.
    pub fn counts(mut self, counts: &[u32]) -> Self {
        for (o, c) in self.options.iter_mut().zip(counts) {
            o.count = Some(*c);
        }
        self
    }
}

/// How options inside a group combine (Oxyn plus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Logic {
    /// Any of them (source behaviour).
    #[default]
    Any,
    /// All of them.
    All,
}

/// A row of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterRow {
    /// Group header `g`.
    Header(usize),
    /// Option `o` of group `g`.
    Option(usize, usize),
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKey {
    /// ↑.
    Up,
    /// ↓.
    Down,
    /// Enter / Space.
    Activate,
    /// →.
    Right,
    /// ←.
    Left,
    /// Home.
    Home,
    /// End.
    End,
}

/// The filter menu.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterMenu {
    /// Groups.
    pub groups: Vec<FilterGroup>,
    /// Picked option ids per group (same order as `groups`).
    pub value: Vec<Vec<String>>,
    /// Open per group.
    pub open: Vec<bool>,
    /// Focused row (index into [`FilterMenu::rows`]).
    pub focus: usize,
    /// Logic per group (Oxyn plus).
    pub logic: Vec<Logic>,
    /// Saved presets (Oxyn plus): name and value.
    pub presets: Vec<(String, Vec<Vec<String>>)>,
    /// Live-region line.
    pub announce: String,
}

impl FilterMenu {
    /// A menu with the first group open.
    pub fn new(groups: Vec<FilterGroup>) -> Self {
        let n = groups.len();
        let mut open = vec![false; n];
        if let Some(o) = open.first_mut() {
            *o = true;
        }
        Self {
            groups,
            value: vec![vec![]; n],
            open,
            focus: 0,
            logic: vec![Logic::Any; n],
            presets: vec![],
            announce: String::new(),
        }
    }
    fn gi(&self, id: &str) -> Option<usize> {
        self.groups.iter().position(|g| g.id == id)
    }
    /// Picks `option` in `group` (setup).
    pub fn with(mut self, group: &str, option: &str) -> Self {
        if let Some(g) = self.gi(group) {
            self.value[g].push(option.into());
        }
        self
    }
    /// Visible rows.
    pub fn rows(&self) -> Vec<FilterRow> {
        let mut out = vec![];
        for (g, grp) in self.groups.iter().enumerate() {
            out.push(FilterRow::Header(g));
            if self.open[g] {
                out.extend((0..grp.options.len()).map(|o| FilterRow::Option(g, o)));
            }
        }
        out
    }
    /// Is option `o` of group `g` picked.
    pub fn checked(&self, g: usize, o: usize) -> bool {
        self.value[g].contains(&self.groups[g].options[o].id)
    }
    /// Picks / unpicks.
    pub fn toggle(&mut self, g: usize, o: usize) -> bool {
        let Some(opt) = self.groups.get(g).and_then(|x| x.options.get(o)) else {
            return false;
        };
        let id = opt.id.clone();
        let label = opt.label.clone();
        if let Some(p) = self.value[g].iter().position(|v| *v == id) {
            self.value[g].remove(p);
            self.announce = format!("Removed {label} filter, {} active", self.count());
            false
        } else {
            self.value[g].push(id);
            self.announce = format!("Added {label} filter, {} active", self.count());
            true
        }
    }
    /// Opens / closes a group (the focus stays on its header).
    pub fn set_open(&mut self, g: usize, open: bool) {
        if let Some(o) = self.open.get_mut(g) {
            *o = open;
        }
        if let Some(p) = self.rows().iter().position(|r| *r == FilterRow::Header(g)) {
            self.focus = p;
        }
    }
    /// Keyboard on the list.
    pub fn key(&mut self, k: FilterKey) {
        let rows = self.rows();
        let n = rows.len();
        if n == 0 {
            return;
        }
        let row = rows[self.focus.min(n - 1)];
        match k {
            FilterKey::Up => self.focus = self.focus.saturating_sub(1),
            FilterKey::Down => self.focus = (self.focus + 1).min(n - 1),
            FilterKey::Home => self.focus = 0,
            FilterKey::End => self.focus = n - 1,
            FilterKey::Activate => match row {
                FilterRow::Header(g) => self.set_open(g, !self.open[g]),
                FilterRow::Option(g, o) => {
                    self.toggle(g, o);
                }
            },
            FilterKey::Right => {
                if let FilterRow::Header(g) = row
                    && !self.open[g]
                {
                    self.set_open(g, true);
                }
            }
            FilterKey::Left => match row {
                FilterRow::Header(g) => self.set_open(g, false),
                FilterRow::Option(g, _) => self.set_open(g, false),
            },
        }
    }
    /// Active filters as chips: `(group, option, "Status: Open")`.
    pub fn chips(&self) -> Vec<(usize, usize, String)> {
        let mut out = vec![];
        for (g, vals) in self.value.iter().enumerate() {
            let grp = &self.groups[g];
            for v in vals {
                if let Some(o) = grp.options.iter().position(|x| x.id == *v) {
                    let pre = grp.chip_prefix.as_deref().unwrap_or(&grp.label);
                    out.push((g, o, format!("{pre}: {}", grp.options[o].label)));
                }
            }
        }
        out
    }
    /// Number of active filters.
    pub fn count(&self) -> usize {
        self.value.iter().map(Vec::len).sum()
    }
    /// Picked count in group `g` (shown on its header).
    pub fn group_count(&self, g: usize) -> usize {
        self.value[g].len()
    }
    /// Clear all.
    pub fn clear(&mut self) {
        self.value.iter_mut().for_each(Vec::clear);
        self.announce = "Cleared all filters".into();
    }
    /// Toggles a group's logic (Oxyn plus).
    pub fn flip_logic(&mut self, g: usize) {
        if let Some(l) = self.logic.get_mut(g) {
            *l = match l {
                Logic::Any => Logic::All,
                Logic::All => Logic::Any,
            };
        }
    }
    /// Saves the current filters as a preset (Oxyn plus).
    pub fn save_preset(&mut self, name: &str) {
        self.presets.retain(|p| p.0 != name);
        self.presets.push((name.into(), self.value.clone()));
    }
    /// Applies a preset.
    pub fn apply_preset(&mut self, name: &str) -> bool {
        match self.presets.iter().find(|p| p.0 == name) {
            Some(p) => {
                self.value = p.1.clone();
                self.announce = format!("Applied {name}");
                true
            }
            None => false,
        }
    }
    /// A plain-language summary (`Status is Open or Blocked · Owner is Sam`).
    pub fn summary(&self) -> String {
        let mut parts = vec![];
        for (g, vals) in self.value.iter().enumerate() {
            if vals.is_empty() {
                continue;
            }
            let grp = &self.groups[g];
            let labels: Vec<&str> = vals
                .iter()
                .filter_map(|v| grp.options.iter().find(|o| o.id == *v))
                .map(|o| o.label.as_str())
                .collect();
            let join = match self.logic[g] {
                Logic::Any => " or ",
                Logic::All => " and ",
            };
            parts.push(format!("{} is {}", grp.label, labels.join(join)));
        }
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> FilterMenu {
        FilterMenu::new(vec![
            FilterGroup::new(
                "status",
                "Status",
                &[("open", "Open"), ("blocked", "Blocked"), ("done", "Done")],
            ),
            FilterGroup::new("owner", "Owner", &[("sam", "Sam"), ("ari", "Ari")]),
        ])
    }

    #[test]
    fn rows_keys_and_chips() {
        let mut m = menu();
        assert_eq!(m.rows().len(), 5, "first group open");
        m.key(FilterKey::Down);
        m.key(FilterKey::Activate);
        assert!(m.checked(0, 0));
        assert_eq!(m.announce, "Added Open filter, 1 active");
        m.key(FilterKey::End);
        assert_eq!(m.rows()[m.focus], FilterRow::Header(1));
        m.key(FilterKey::Right);
        assert!(m.open[1]);
        m.key(FilterKey::Down);
        m.key(FilterKey::Activate);
        m.key(FilterKey::Left);
        assert!(!m.open[1]);
        assert_eq!(
            m.rows()[m.focus],
            FilterRow::Header(1),
            "Left jumps to the header"
        );
        assert_eq!(m.chips()[1].2, "Owner: Sam");
        assert_eq!(m.count(), 2);
        m.key(FilterKey::Home);
        m.key(FilterKey::Activate);
        assert_eq!(m.rows().len(), 2, "closed the first group");
        m.clear();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn logic_presets_summary() {
        let mut m = menu().with("status", "open").with("status", "blocked");
        assert_eq!(m.summary(), "Status is Open or Blocked");
        m.flip_logic(0);
        assert_eq!(m.summary(), "Status is Open and Blocked");
        m.save_preset("Triage");
        m.clear();
        assert!(m.apply_preset("Triage"));
        assert_eq!(m.count(), 2);
        assert!(!m.apply_preset("Nope"));
        assert_eq!(m.group_count(0), 2);
    }
}
