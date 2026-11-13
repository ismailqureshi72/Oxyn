//! Dropdown menu (`dropdown-menu.md`): actions and choices behind a trigger, with icons,
//! shortcuts, checkbox / radio items, danger items, dividers, submenus, type-ahead, the full menu
//! keyboard and one highlight that glides between rows. Oxyn plus: a filter field for long menus,
//! compact / comfortable density, and a context-menu mode that opens at the pointer.

use super::button::Glide;
use super::nav::{NavKey, Typeahead, step};
use super::placement::{Align, Side};
use crate::ClockMs;

/// What a row is.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    /// Runs an action.
    Action,
    /// Checkbox item.
    Checkbox(bool),
    /// Radio item in `group`.
    Radio {
        /// Group name.
        group: String,
        /// Picked.
        checked: bool,
    },
    /// Opens a submenu.
    Submenu(Vec<MenuItem>),
    /// A hairline between groups.
    Divider,
    /// A small group heading.
    Label,
}

/// One row.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    /// Id passed back on activation.
    pub id: String,
    /// Label.
    pub label: String,
    /// Kind.
    pub kind: ItemKind,
    /// Icon name.
    pub icon: Option<String>,
    /// Shortcut ("Mod+D"), shown right-aligned via [`super::display::parse_shortcut`].
    pub shortcut: Option<String>,
    /// Destructive (red text and a red-tinted highlight).
    pub danger: bool,
    /// Disabled.
    pub disabled: bool,
    /// Activating keeps the menu open (checkbox items default to this).
    pub keep_open: bool,
    /// Second line (Oxyn plus in comfortable density).
    pub hint: Option<String>,
}

impl MenuItem {
    /// An action.
    pub fn action(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: ItemKind::Action,
            icon: None,
            shortcut: None,
            danger: false,
            disabled: false,
            keep_open: false,
            hint: None,
        }
    }
    /// A checkbox item (keeps the menu open).
    pub fn checkbox(id: &str, label: &str, on: bool) -> Self {
        Self {
            kind: ItemKind::Checkbox(on),
            keep_open: true,
            ..Self::action(id, label)
        }
    }
    /// A radio item.
    pub fn radio(id: &str, label: &str, group: &str, on: bool) -> Self {
        Self {
            kind: ItemKind::Radio {
                group: group.into(),
                checked: on,
            },
            ..Self::action(id, label)
        }
    }
    /// A submenu.
    pub fn submenu(id: &str, label: &str, items: Vec<MenuItem>) -> Self {
        Self {
            kind: ItemKind::Submenu(items),
            ..Self::action(id, label)
        }
    }
    /// A divider.
    pub fn divider() -> Self {
        Self {
            kind: ItemKind::Divider,
            ..Self::action("", "")
        }
    }
    /// A group label.
    pub fn label(text: &str) -> Self {
        Self {
            kind: ItemKind::Label,
            ..Self::action("", text)
        }
    }
    /// With an icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// With a shortcut.
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Marked dangerous.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    /// Disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
    /// Can be highlighted.
    pub fn focusable(&self) -> bool {
        !self.disabled && !matches!(self.kind, ItemKind::Divider | ItemKind::Label)
    }
    /// `menuitem`, `menuitemcheckbox`, `menuitemradio`, `separator`, `presentation`.
    pub fn role(&self) -> &'static str {
        match self.kind {
            ItemKind::Action | ItemKind::Submenu(_) => "menuitem",
            ItemKind::Checkbox(_) => "menuitemcheckbox",
            ItemKind::Radio { .. } => "menuitemradio",
            ItemKind::Divider => "separator",
            ItemKind::Label => "presentation",
        }
    }
    /// `aria-checked`, when it has one.
    pub fn checked(&self) -> Option<bool> {
        match &self.kind {
            ItemKind::Checkbox(c) => Some(*c),
            ItemKind::Radio { checked, .. } => Some(*checked),
            _ => None,
        }
    }
}

/// Keys the menu handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKey {
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
    /// → (open submenu).
    Right,
    /// ← (close submenu).
    Left,
    /// Esc.
    Escape,
    /// Tab (closes, focus moves on).
    Tab,
    /// A printable character (type-ahead).
    Char(char),
}

/// What happened.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuOutcome {
    /// Nothing for the host.
    None,
    /// An action ran.
    Activated(String),
    /// A checkbox / radio changed.
    Toggled(String, bool),
    /// The menu closed; `restore` = focus goes back to the trigger.
    Closed {
        /// Return focus to the trigger.
        restore: bool,
    },
}

/// Density (Oxyn plus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    /// 28 px rows.
    Compact,
    /// 32 px rows (default).
    #[default]
    Comfortable,
}

impl Density {
    /// Row height.
    pub fn row(self) -> f32 {
        match self {
            Self::Compact => 28.0,
            Self::Comfortable => 32.0,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
        }
    }
}

/// How the menu opened (Oxyn plus: context mode).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MenuAnchor {
    /// Below / above the trigger.
    #[default]
    Trigger,
    /// At the pointer (right-click), or under the target for Shift+F10 / the ContextMenu key.
    Point(f32, f32),
}

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Menu fades + scales in from its anchored corner.
    pub const OPEN: Duration = oxyn_tokens::motion::dur::popover::OPEN;
    /// Closing.
    pub const CLOSE: Duration = oxyn_tokens::motion::dur::popover::CLOSE;
    /// Starting scale.
    pub const FROM_SCALE: f32 = 0.96;
    /// Row stagger on open (Arc).
    pub const STAGGER: Duration = oxyn_tokens::motion::stagger::ROW;
    /// Pointer must rest this long on a submenu row before it opens.
    pub const SUBMENU_DELAY: Duration = oxyn_tokens::motion::delay::INTENT;
}

/// The menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    /// Rows.
    pub items: Vec<MenuItem>,
    /// Open.
    pub open: bool,
    /// Side (`bottom`).
    pub side: Side,
    /// Align (`start`).
    pub align: Align,
    /// Shown when there are no items (or the filter matches none).
    pub empty: String,
    /// Oxyn plus: filter text (only when `searchable`).
    pub query: String,
    /// Oxyn plus: show the filter field.
    pub searchable: bool,
    /// Density.
    pub density: Density,
    /// Anchor.
    pub anchor: MenuAnchor,
    /// Highlight per open level (level 0 = root).
    pub levels: Vec<Glide>,
    /// Path of open submenus (indices into each level's visible rows).
    pub path: Vec<usize>,
    /// When it opened.
    pub opened_at: ClockMs,
    typeahead: Typeahead,
}

impl Menu {
    /// A closed menu.
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            open: false,
            side: Side::Bottom,
            align: Align::Start,
            empty: "Nothing here yet".into(),
            query: String::new(),
            searchable: false,
            density: Density::Comfortable,
            anchor: MenuAnchor::Trigger,
            levels: vec![Glide::default()],
            path: vec![],
            opened_at: ClockMs(0),
            typeahead: Typeahead::default(),
        }
    }

    /// The rows at `level` (0 = root, filtered by the query).
    pub fn rows(&self, level: usize) -> Vec<MenuItem> {
        let mut cur = self.filtered();
        for &i in self.path.iter().take(level) {
            match cur.get(i).map(|x| x.kind.clone()) {
                Some(ItemKind::Submenu(sub)) => cur = sub,
                _ => return vec![],
            }
        }
        cur
    }

    fn filtered(&self) -> Vec<MenuItem> {
        if !self.searchable || self.query.trim().is_empty() {
            return self.items.clone();
        }
        let q = self.query.to_lowercase();
        let mut out = vec![];
        fn walk(items: &[MenuItem], q: &str, out: &mut Vec<MenuItem>) {
            for it in items {
                match &it.kind {
                    ItemKind::Submenu(sub) => walk(sub, q, out),
                    ItemKind::Divider | ItemKind::Label => {}
                    _ if it.label.to_lowercase().contains(q) => out.push(it.clone()),
                    _ => {}
                }
            }
        }
        walk(&self.items, &q, &mut out);
        out
    }

    /// Deepest open level.
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Opens from the trigger. `last` = opened with ↑ (highlight the last row); otherwise the
    /// first row is highlighted when opened from the keyboard, none for the pointer.
    pub fn open_menu(&mut self, now: ClockMs, keyboard: bool, last: bool) {
        self.open = true;
        self.opened_at = now;
        self.path.clear();
        self.query.clear();
        self.levels = vec![Glide::default()];
        if keyboard {
            let rows = self.rows(0);
            let k = if last { NavKey::Last } else { NavKey::First };
            let to = step(None, k, rows.len(), true, |i| rows[i].focusable());
            self.levels[0].go(to, now, true);
        }
    }

    /// Opens as a context menu at a point.
    pub fn open_at(&mut self, x: f32, y: f32, now: ClockMs, keyboard: bool) {
        self.anchor = MenuAnchor::Point(x, y);
        self.open_menu(now, keyboard, false);
    }

    /// Closes everything.
    pub fn close(&mut self, restore: bool) -> MenuOutcome {
        self.open = false;
        self.path.clear();
        self.levels.truncate(1);
        self.typeahead.reset();
        MenuOutcome::Closed { restore }
    }

    /// Pointer moved over row `i` of `level` (glides, and closes deeper submenus).
    pub fn hover(&mut self, level: usize, i: Option<usize>, now: ClockMs) {
        if level >= self.levels.len() {
            return;
        }
        let rows = self.rows(level);
        let i = i.filter(|&i| rows.get(i).is_some_and(MenuItem::focusable));
        self.path.truncate(level);
        self.levels.truncate(level + 1);
        self.levels[level].go(i, now, false);
    }

    /// Highlighted row at the deepest level.
    pub fn highlighted(&self) -> Option<(usize, usize)> {
        let l = self.depth();
        self.levels.get(l)?.target.map(|i| (l, i))
    }

    fn open_sub(&mut self, now: ClockMs, keyboard: bool) -> bool {
        let Some((l, i)) = self.highlighted() else {
            return false;
        };
        let rows = self.rows(l);
        let Some(ItemKind::Submenu(sub)) = rows.get(i).map(|r| &r.kind) else {
            return false;
        };
        let first = step(None, NavKey::First, sub.len(), true, |k| sub[k].focusable());
        self.path.push(i);
        let mut g = Glide::default();
        if keyboard {
            g.go(first, now, true);
        }
        self.levels.push(g);
        true
    }

    /// Activates row `i` at `level` (click, or Enter / Space on the highlight).
    pub fn activate(&mut self, level: usize, i: usize, now: ClockMs) -> MenuOutcome {
        let rows = self.rows(level);
        let Some(item) = rows.get(i).cloned() else {
            return MenuOutcome::None;
        };
        if !item.focusable() {
            return MenuOutcome::None;
        }
        self.levels.truncate(level + 1);
        self.path.truncate(level);
        self.levels[level].go(Some(i), now, true);
        match &item.kind {
            ItemKind::Submenu(_) => {
                self.open_sub(now, true);
                MenuOutcome::None
            }
            ItemKind::Checkbox(on) => {
                let v = !on;
                set_checked(&mut self.items, &item.id, |it| {
                    it.kind = ItemKind::Checkbox(v);
                });
                if !item.keep_open {
                    self.close(true);
                }
                MenuOutcome::Toggled(item.id, v)
            }
            ItemKind::Radio { group, .. } => {
                let g = group.clone();
                set_radio(&mut self.items, &g, &item.id);
                if !item.keep_open {
                    self.close(true);
                }
                MenuOutcome::Toggled(item.id, true)
            }
            _ => {
                if !item.keep_open {
                    self.close(true);
                }
                MenuOutcome::Activated(item.id)
            }
        }
    }

    /// Keyboard.
    pub fn key(&mut self, k: MenuKey, now: ClockMs) -> MenuOutcome {
        if !self.open {
            return match k {
                MenuKey::Enter | MenuKey::Space | MenuKey::Down => {
                    self.open_menu(now, true, false);
                    MenuOutcome::None
                }
                MenuKey::Up => {
                    self.open_menu(now, true, true);
                    MenuOutcome::None
                }
                _ => MenuOutcome::None,
            };
        }
        let l = self.depth();
        let rows = self.rows(l);
        let cur = self.levels[l].target;
        let mv = |key: NavKey| step(cur, key, rows.len(), true, |i| rows[i].focusable());
        match k {
            MenuKey::Down => {
                let to = mv(NavKey::Next);
                self.levels[l].go(to, now, true);
            }
            MenuKey::Up => {
                let to = mv(NavKey::Prev);
                self.levels[l].go(to, now, true);
            }
            MenuKey::Home => {
                let to = mv(NavKey::First);
                self.levels[l].go(to, now, true);
            }
            MenuKey::End => {
                let to = mv(NavKey::Last);
                self.levels[l].go(to, now, true);
            }
            MenuKey::Right => {
                self.open_sub(now, true);
            }
            MenuKey::Left => {
                if l > 0 {
                    self.path.pop();
                    self.levels.pop();
                }
            }
            MenuKey::Escape => {
                if l > 0 {
                    self.path.pop();
                    self.levels.pop();
                } else {
                    return self.close(true);
                }
            }
            MenuKey::Tab => return self.close(false),
            MenuKey::Enter | MenuKey::Space => {
                if let Some(i) = cur {
                    return self.activate(l, i, now);
                }
            }
            MenuKey::Char(c) => {
                if self.searchable {
                    self.query.push(c);
                    let rows = self.rows(0);
                    let first = step(None, NavKey::First, rows.len(), true, |i| {
                        rows[i].focusable()
                    });
                    self.path.clear();
                    self.levels = vec![Glide::default()];
                    self.levels[0].go(first, now, true);
                } else {
                    let to = self.typeahead.feed(
                        c,
                        now,
                        cur,
                        rows.iter().map(|r| (r.label.as_str(), r.focusable())),
                    );
                    if to.is_some() {
                        self.levels[l].go(to, now, true);
                    }
                }
            }
        }
        MenuOutcome::None
    }
}

fn set_checked(items: &mut [MenuItem], id: &str, f: impl Fn(&mut MenuItem) + Copy) {
    for it in items.iter_mut() {
        if it.id == id {
            f(it);
        }
        if let ItemKind::Submenu(sub) = &mut it.kind {
            set_checked(sub, id, f);
        }
    }
}

fn set_radio(items: &mut [MenuItem], group: &str, id: &str) {
    for it in items.iter_mut() {
        let me = it.id == id;
        match &mut it.kind {
            ItemKind::Radio { group: g, checked } if g == group => *checked = me,
            ItemKind::Submenu(sub) => set_radio(sub, group, id),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> Menu {
        Menu::new(vec![
            MenuItem::label("Thread"),
            MenuItem::action("rename", "Rename")
                .icon("pencil")
                .shortcut("Mod+R"),
            MenuItem::action("dup", "Duplicate").shortcut("Mod+D"),
            MenuItem::action("pin", "Pin").disabled(),
            MenuItem::submenu(
                "move",
                "Move to",
                vec![
                    MenuItem::action("inbox", "Inbox"),
                    MenuItem::action("archive", "Archive"),
                ],
            ),
            MenuItem::checkbox("wrap", "Wrap lines", false),
            MenuItem::radio("m1", "Fast", "model", true),
            MenuItem::radio("m2", "Careful", "model", false),
            MenuItem::divider(),
            MenuItem::action("del", "Delete").danger(),
        ])
    }

    #[test]
    fn opens_from_keyboard_and_navigates() {
        let mut m = menu();
        m.key(MenuKey::Down, ClockMs(0));
        assert!(m.open);
        assert_eq!(m.highlighted(), Some((0, 1)), "label skipped");
        m.key(MenuKey::Down, ClockMs(1));
        m.key(MenuKey::Down, ClockMs(2));
        assert_eq!(m.highlighted(), Some((0, 4)), "disabled skipped");
        m.key(MenuKey::End, ClockMs(3));
        assert_eq!(m.highlighted(), Some((0, 9)));
        m.key(MenuKey::Down, ClockMs(4));
        assert_eq!(m.highlighted(), Some((0, 1)), "wraps");
        let mut m = menu();
        m.key(MenuKey::Up, ClockMs(0));
        assert_eq!(m.highlighted(), Some((0, 9)), "↑ opens on the last row");
    }

    #[test]
    fn submenus_and_escape() {
        let mut m = menu();
        m.open_menu(ClockMs(0), true, false);
        m.hover(0, Some(4), ClockMs(1));
        m.key(MenuKey::Right, ClockMs(2));
        assert_eq!(m.depth(), 1);
        assert_eq!(m.highlighted(), Some((1, 0)));
        assert_eq!(m.rows(1)[1].label, "Archive");
        assert_eq!(m.key(MenuKey::Escape, ClockMs(3)), MenuOutcome::None);
        assert_eq!(m.depth(), 0);
        assert!(m.open);
        assert_eq!(
            m.key(MenuKey::Escape, ClockMs(4)),
            MenuOutcome::Closed { restore: true }
        );
    }

    #[test]
    fn activation_checkbox_radio_and_typeahead() {
        let mut m = menu();
        m.open_menu(ClockMs(0), true, false);
        assert_eq!(
            m.activate(0, 5, ClockMs(1)),
            MenuOutcome::Toggled("wrap".into(), true)
        );
        assert!(m.open, "checkbox keeps the menu open");
        assert_eq!(m.rows(0)[5].checked(), Some(true));
        m.activate(0, 7, ClockMs(2));
        assert!(!m.open);
        assert_eq!(m.rows(0)[6].checked(), Some(false));
        assert_eq!(m.rows(0)[7].checked(), Some(true));
        m.open_menu(ClockMs(3), true, false);
        m.key(MenuKey::Char('d'), ClockMs(4));
        assert_eq!(m.highlighted(), Some((0, 2)));
        m.key(MenuKey::Char('e'), ClockMs(5));
        assert_eq!(m.highlighted(), Some((0, 9)), "\"de\" → Delete");
        assert_eq!(
            m.key(MenuKey::Enter, ClockMs(6)),
            MenuOutcome::Activated("del".into())
        );
        assert!(!m.open);
        assert_eq!(
            m.key(MenuKey::Tab, ClockMs(7)),
            MenuOutcome::None,
            "closed already"
        );
    }

    #[test]
    fn searchable_and_context_mode() {
        let mut m = menu();
        m.searchable = true;
        m.open_at(120.0, 80.0, ClockMs(0), false);
        assert_eq!(m.anchor, MenuAnchor::Point(120.0, 80.0));
        m.key(MenuKey::Char('a'), ClockMs(1));
        m.key(MenuKey::Char('r'), ClockMs(2));
        m.key(MenuKey::Char('c'), ClockMs(3));
        let rows = m.rows(0);
        assert_eq!(
            rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            ["archive"]
        );
        assert_eq!(m.highlighted(), Some((0, 0)));
        m.query = "zzz".into();
        assert!(m.rows(0).is_empty());
        assert_eq!(
            MenuItem::checkbox("a", "A", true).role(),
            "menuitemcheckbox"
        );
    }
}
