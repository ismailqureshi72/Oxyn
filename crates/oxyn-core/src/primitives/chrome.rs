//! App-chrome models (plan #27): reactions with a picker, notification center, team switcher,
//! user menu and the code-block highlighter.
//!
//! Behaviour follows the Arc UI docs for Reactions, Notification center, Team switcher, User menu
//! and Code block (rebuilt from the public descriptions; no source used).

use std::time::Duration;

use super::nav::{NavKey, Typeahead, step};
use crate::ClockMs;

// ──────────────────────────────────────── reactions ──

/// A reaction the picker offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactionOption {
    /// Id.
    pub id: String,
    /// Label ("Love").
    pub label: String,
    /// Glyph (emoji or icon name).
    pub glyph: String,
}

impl ReactionOption {
    /// An option.
    pub fn new(id: &str, label: &str, glyph: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            glyph: glyph.into(),
        }
    }
}

/// One pill in the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactionCount {
    /// Option id.
    pub id: String,
    /// Count.
    pub count: u32,
    /// Includes yours.
    pub reacted: bool,
}

/// Reactions: pills with counts plus a picker tray.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reactions {
    /// What the picker offers.
    pub options: Vec<ReactionOption>,
    /// Pills, in order. Zero counts leave the bar.
    pub value: Vec<ReactionCount>,
    /// Picker open.
    pub picker: bool,
    /// Picker focus.
    pub focus: Option<usize>,
    /// Last change, for the live region.
    pub last: Option<(String, bool)>,
}

impl Reactions {
    /// Reactions.
    pub fn new(options: Vec<ReactionOption>, value: Vec<ReactionCount>) -> Self {
        Self {
            options,
            value,
            picker: false,
            focus: None,
            last: None,
        }
    }
    /// The option for an id.
    pub fn option(&self, id: &str) -> Option<&ReactionOption> {
        self.options.iter().find(|o| o.id == id)
    }
    /// Toggles your reaction `id` (adds a pill when new). Returns (reacted, removed pill index).
    pub fn toggle(&mut self, id: &str) -> (bool, Option<usize>) {
        self.picker = false;
        if let Some(i) = self.value.iter().position(|r| r.id == id) {
            let r = &mut self.value[i];
            if r.reacted {
                r.reacted = false;
                r.count = r.count.saturating_sub(1);
            } else {
                r.reacted = true;
                r.count += 1;
            }
            let now = r.reacted;
            self.last = Some((id.into(), now));
            if r.count == 0 {
                self.value.remove(i);
                return (now, Some(i));
            }
            (now, None)
        } else {
            self.value.push(ReactionCount {
                id: id.into(),
                count: 1,
                reacted: true,
            });
            self.last = Some((id.into(), true));
            (true, None)
        }
    }
    /// Opens or closes the picker.
    pub fn set_picker(&mut self, open: bool) {
        self.picker = open;
        self.focus = if open { Some(0) } else { None };
    }
    /// Arrow keys in the picker.
    pub fn key(&mut self, k: NavKey) {
        self.focus = step(self.focus, k, self.options.len(), true, |_| true);
    }
    /// Pill label: "Love, 4 reactions, including yours".
    pub fn pill_label(&self, r: &ReactionCount) -> String {
        let l = self.option(&r.id).map(|o| o.label.as_str()).unwrap_or("");
        let n = if r.count == 1 {
            "reaction"
        } else {
            "reactions"
        };
        let yours = if r.reacted { ", including yours" } else { "" };
        format!("{l}, {} {n}{yours}", r.count)
    }
    /// Live-region text for the last change.
    pub fn announce(&self) -> String {
        match &self.last {
            Some((id, on)) => {
                let l = self.option(id).map(|o| o.label.as_str()).unwrap_or(id);
                if *on {
                    format!("Added {l}")
                } else {
                    format!("Removed {l}")
                }
            }
            None => String::new(),
        }
    }
}

// ─────────────────────────────── notification center ──

/// A notification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Body (shown when expanded).
    pub body: String,
    /// Group ("Today", "Earlier").
    pub group: String,
    /// Relative time.
    pub time: String,
    /// Unread.
    pub unread: bool,
    /// Actor (monogram / agent).
    pub actor: String,
    /// Oxyn plus: kind (`approval` · `run` · `mention` · `system`).
    pub kind: String,
}

impl Note {
    /// A notification.
    pub fn new(id: &str, actor: &str, title: &str, body: &str, group: &str, time: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            group: group.into(),
            time: time.into(),
            unread: true,
            actor: actor.into(),
            kind: "system".into(),
        }
    }
    /// Read.
    pub fn read(mut self) -> Self {
        self.unread = false;
        self
    }
    /// Kind.
    pub fn kind(mut self, k: &str) -> Self {
        self.kind = k.into();
        self
    }
}

/// Notification-center view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoteView {
    /// Everything.
    #[default]
    All,
    /// Unread only.
    Unread,
}

/// Notification center.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationCenter {
    /// Items, newest first.
    pub items: Vec<Note>,
    /// View.
    pub view: NoteView,
    /// Expanded row.
    pub expanded: Option<String>,
    /// Panel open.
    pub open: bool,
}

impl NotificationCenter {
    /// A center.
    pub fn new(items: Vec<Note>) -> Self {
        Self {
            items,
            view: NoteView::All,
            expanded: None,
            open: false,
        }
    }
    /// Unread count.
    pub fn unread(&self) -> usize {
        self.items.iter().filter(|n| n.unread).count()
    }
    /// Rows for the view, grouped in order of first appearance.
    pub fn groups(&self) -> Vec<(String, Vec<Note>)> {
        let mut out: Vec<(String, Vec<Note>)> = Vec::new();
        for n in self
            .items
            .iter()
            .filter(|n| self.view == NoteView::All || n.unread)
        {
            if let Some(g) = out.iter_mut().find(|g| g.0 == n.group) {
                g.1.push(n.clone());
            } else {
                out.push((n.group.clone(), vec![n.clone()]));
            }
        }
        out
    }
    /// Expands a row (marks it read). Returns the id that should take focus next when the
    /// unread view hides it.
    pub fn toggle(&mut self, id: &str) {
        if self.expanded.as_deref() == Some(id) {
            self.expanded = None;
        } else {
            self.expanded = Some(id.into());
            if let Some(n) = self.items.iter_mut().find(|n| n.id == id) {
                n.unread = false;
            }
        }
    }
    /// Marks one read.
    pub fn mark_read(&mut self, id: &str) {
        if let Some(n) = self.items.iter_mut().find(|n| n.id == id) {
            n.unread = false;
        }
    }
    /// Marks all read.
    pub fn mark_all(&mut self) -> usize {
        let n = self.unread();
        for x in &mut self.items {
            x.unread = false;
        }
        n
    }
    /// Dismisses one; returns the id that should take focus.
    pub fn dismiss(&mut self, id: &str) -> Option<String> {
        let i = self.items.iter().position(|n| n.id == id)?;
        self.items.remove(i);
        self.items
            .get(i)
            .or_else(|| i.checked_sub(1).and_then(|j| self.items.get(j)))
            .map(|n| n.id.clone())
    }
    /// Clears everything.
    pub fn clear(&mut self) {
        self.items.clear();
        self.expanded = None;
    }
    /// Trigger label: "Notifications, 3 unread".
    pub fn trigger_label(&self) -> String {
        match self.unread() {
            0 => "Notifications".into(),
            n => format!("Notifications, {n} unread"),
        }
    }
    /// Summary line.
    pub fn summary(&self) -> String {
        match self.unread() {
            0 => "You're all caught up".into(),
            1 => "1 unread".into(),
            n => format!("{n} unread"),
        }
    }
}

// ──────────────────────────────────── team switcher ──

/// A workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Plan ("Pro").
    pub plan: Option<String>,
    /// Members.
    pub members: Option<u32>,
    /// Your role.
    pub role: Option<String>,
    /// Tile hue name for the monogram (agent identity palette).
    pub tile: Option<String>,
}

impl Workspace {
    /// A workspace.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            plan: None,
            members: None,
            role: None,
            tile: None,
        }
    }
    /// Plan.
    pub fn plan(mut self, p: &str) -> Self {
        self.plan = Some(p.into());
        self
    }
    /// Role.
    pub fn role(mut self, r: &str) -> Self {
        self.role = Some(r.into());
        self
    }
    /// Members.
    pub fn members(mut self, n: u32) -> Self {
        self.members = Some(n);
        self
    }
    /// Tile hue.
    pub fn tile(mut self, t: &str) -> Self {
        self.tile = Some(t.into());
        self
    }
    /// First letter.
    pub fn letter(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default()
    }
}

/// Team-switcher panel view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitcherView {
    /// The list.
    #[default]
    List,
    /// Create workspace.
    Create,
    /// Join with invite.
    Join,
}

/// A row in the switcher list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitcherRow {
    /// A workspace (index into `workspaces`).
    Workspace(usize),
    /// Create workspace.
    Create,
    /// Join with invite.
    Join,
}

/// How long a switch takes in demos.
pub const SWITCH_DELAY: Duration = Duration::from_millis(650);

/// Team switcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamSwitcher {
    /// Workspaces.
    pub workspaces: Vec<Workspace>,
    /// Active id.
    pub active: String,
    /// Open.
    pub open: bool,
    /// Search.
    pub query: String,
    /// Highlighted row.
    pub highlight: usize,
    /// View.
    pub view: SwitcherView,
    /// Switching to (id, since).
    pub switching: Option<(String, ClockMs)>,
    /// Draft in the create / join field.
    pub draft: String,
}

impl TeamSwitcher {
    /// A switcher.
    pub fn new(workspaces: Vec<Workspace>, active: Option<&str>) -> Self {
        let active = active
            .map(str::to_string)
            .or_else(|| workspaces.first().map(|w| w.id.clone()))
            .unwrap_or_default();
        Self {
            workspaces,
            active,
            open: false,
            query: String::new(),
            highlight: 0,
            view: SwitcherView::List,
            switching: None,
            draft: String::new(),
        }
    }
    /// The active workspace.
    pub fn current(&self) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == self.active)
    }
    /// Rows matching the query (name, plan or role), then the two actions.
    pub fn rows(&self) -> Vec<SwitcherRow> {
        let q = self.query.to_lowercase();
        let mut v: Vec<SwitcherRow> = self
            .workspaces
            .iter()
            .enumerate()
            .filter(|(_, w)| {
                q.is_empty()
                    || w.name.to_lowercase().contains(&q)
                    || w.plan.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || w.role.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .map(|(i, _)| SwitcherRow::Workspace(i))
            .collect();
        v.push(SwitcherRow::Create);
        if q.is_empty() {
            v.push(SwitcherRow::Join);
        }
        v
    }
    /// Opens (highlight on the active workspace).
    pub fn open_menu(&mut self) {
        self.open = true;
        self.view = SwitcherView::List;
        self.query.clear();
        self.highlight = self
            .workspaces
            .iter()
            .position(|w| w.id == self.active)
            .unwrap_or(0);
    }
    /// Types in the search.
    pub fn set_query(&mut self, q: &str) {
        self.query = q.into();
        self.highlight = 0;
    }
    /// ↑ ↓ Home End.
    pub fn key(&mut self, k: NavKey) {
        let n = self.rows().len();
        if matches!(k, NavKey::First | NavKey::Last) && !self.query.is_empty() {
            return;
        }
        self.highlight = step(Some(self.highlight), k, n, true, |_| true).unwrap_or(0);
    }
    /// Enter on the highlight. With no workspace match, opens Create with the name filled in.
    pub fn enter(&mut self, now: ClockMs) -> Option<String> {
        let rows = self.rows();
        match rows.get(self.highlight)? {
            SwitcherRow::Workspace(i) => {
                let id = self.workspaces[*i].id.clone();
                self.switch_to(&id, now)
            }
            SwitcherRow::Create => {
                self.draft = self.query.clone();
                self.view = SwitcherView::Create;
                None
            }
            SwitcherRow::Join => {
                self.draft.clear();
                self.view = SwitcherView::Join;
                None
            }
        }
    }
    /// Starts a switch (closes the menu); the trigger shows switching until `settle`.
    pub fn switch_to(&mut self, id: &str, now: ClockMs) -> Option<String> {
        if !self.workspaces.iter().any(|w| w.id == id) {
            return None;
        }
        self.open = false;
        if id == self.active {
            return None;
        }
        self.switching = Some((id.into(), now));
        Some(id.into())
    }
    /// Cmd/Ctrl + `n` (1–9).
    pub fn shortcut(&mut self, n: usize, now: ClockMs) -> Option<String> {
        let id = self.workspaces.get(n.checked_sub(1)?)?.id.clone();
        self.switch_to(&id, now)
    }
    /// Finishes a switch after `delay`. Returns the new active id.
    pub fn settle(&mut self, now: ClockMs, delay: Duration) -> Option<String> {
        let (id, t) = self.switching.clone()?;
        if now.since(t) >= delay {
            self.active = id.clone();
            self.switching = None;
            return Some(id);
        }
        None
    }
    /// Creates a workspace from the draft and switches to it.
    pub fn create(&mut self, now: ClockMs) -> Option<String> {
        let name = self.draft.trim().to_string();
        if name.is_empty() {
            return None;
        }
        let id = name.to_lowercase().replace(' ', "-");
        self.workspaces
            .push(Workspace::new(&id, &name).plan("Free").role("Owner"));
        self.view = SwitcherView::List;
        self.draft.clear();
        self.switch_to(&id, now)
    }
    /// Escape: back from create / join, then clear the search, then close.
    pub fn escape(&mut self) -> bool {
        if self.view != SwitcherView::List {
            self.view = SwitcherView::List;
            true
        } else if !self.query.is_empty() {
            self.query.clear();
            self.highlight = 0;
            true
        } else if self.open {
            self.open = false;
            true
        } else {
            false
        }
    }
    /// Trigger label.
    pub fn trigger_label(&self) -> String {
        match self.current() {
            Some(w) => match &w.plan {
                Some(p) => format!("{}, {p} plan. Switch workspace", w.name),
                None => format!("{}. Switch workspace", w.name),
            },
            None => "Switch workspace".into(),
        }
    }
}

// ─────────────────────────────────────── user menu ──

/// Theme preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePref {
    /// Light.
    Light,
    /// Dark.
    Dark,
    /// System.
    #[default]
    System,
}

impl ThemePref {
    /// All in order.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
    /// Icon.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Light => "sun",
            Self::Dark => "moon",
            Self::System => "monitor",
        }
    }
}

/// Presence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Presence {
    /// Available (filled dot).
    #[default]
    Available,
    /// Busy (bar).
    Busy,
    /// Away (half moon).
    Away,
    /// Offline (ring).
    Offline,
}

impl Presence {
    /// Choosable in the menu.
    pub const CHOICES: [Self; 3] = [Self::Available, Self::Busy, Self::Away];
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Busy => "busy",
            Self::Away => "away",
            Self::Offline => "offline",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "busy" => Self::Busy,
            "away" => Self::Away,
            "offline" => Self::Offline,
            _ => Self::Available,
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::Offline => "Offline",
        }
    }
}

/// A row of the user menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserRow {
    /// A link (index into items).
    Item(usize),
    /// Theme switch.
    Theme,
    /// Status switch.
    Status,
    /// Sign out.
    SignOut,
}

/// User menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserMenu {
    /// Link labels.
    pub items: Vec<String>,
    /// Theme switch shown.
    pub show_theme: bool,
    /// Status switch shown.
    pub show_status: bool,
    /// Open.
    pub open: bool,
    /// Highlighted row.
    pub highlight: Option<usize>,
    /// Theme.
    pub theme: ThemePref,
    /// Status.
    pub status: Presence,
    /// Signing out since.
    pub signing_out: Option<ClockMs>,
    typeahead: Typeahead,
}

impl UserMenu {
    /// A menu.
    pub fn new(items: &[&str]) -> Self {
        Self {
            items: items.iter().map(|s| s.to_string()).collect(),
            show_theme: true,
            show_status: false,
            open: false,
            highlight: None,
            theme: ThemePref::System,
            status: Presence::Available,
            signing_out: None,
            typeahead: Typeahead::default(),
        }
    }
    /// Rows in order.
    pub fn rows(&self) -> Vec<UserRow> {
        let mut v: Vec<UserRow> = (0..self.items.len()).map(UserRow::Item).collect();
        if self.show_status {
            v.push(UserRow::Status);
        }
        if self.show_theme {
            v.push(UserRow::Theme);
        }
        v.push(UserRow::SignOut);
        v
    }
    /// Opens from the trigger (`last` for ArrowUp).
    pub fn open_menu(&mut self, keyboard: bool, last: bool) {
        self.open = true;
        let n = self.rows().len();
        self.highlight = if keyboard {
            Some(if last { n - 1 } else { 0 })
        } else {
            None
        };
    }
    /// Closes.
    pub fn close(&mut self) {
        self.open = false;
        self.highlight = None;
    }
    /// ↑ ↓ Home End (loops).
    pub fn key(&mut self, k: NavKey) {
        let n = self.rows().len();
        self.highlight = step(self.highlight, k, n, true, |_| true);
    }
    /// ← → on a switch row: previous / next option.
    pub fn cycle(&mut self, forward: bool) -> bool {
        let rows = self.rows();
        match self.highlight.and_then(|h| rows.get(h)) {
            Some(UserRow::Theme) => {
                let i = ThemePref::ALL
                    .iter()
                    .position(|t| *t == self.theme)
                    .unwrap_or(0);
                let j = if forward { (i + 1) % 3 } else { (i + 2) % 3 };
                self.theme = ThemePref::ALL[j];
                true
            }
            Some(UserRow::Status) => {
                let i = Presence::CHOICES
                    .iter()
                    .position(|t| *t == self.status)
                    .unwrap_or(0);
                let j = if forward { (i + 1) % 3 } else { (i + 2) % 3 };
                self.status = Presence::CHOICES[j];
                true
            }
            _ => false,
        }
    }
    /// A–Z typeahead over link labels and "Sign out".
    pub fn type_char(&mut self, c: char, now: ClockMs) {
        let rows = self.rows();
        let labels: Vec<String> = rows
            .iter()
            .map(|r| match r {
                UserRow::Item(i) => self.items[*i].clone(),
                UserRow::Theme => "Theme".into(),
                UserRow::Status => "Status".into(),
                UserRow::SignOut => "Sign out".into(),
            })
            .collect();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        if let Some(i) =
            self.typeahead
                .feed(c, now, self.highlight, refs.iter().map(|l| (*l, true)))
        {
            self.highlight = Some(i);
        }
    }
    /// Sign out pressed (async: stays open with a spinner).
    pub fn sign_out(&mut self, now: ClockMs) {
        self.signing_out = Some(now);
    }
}

/// Trigger label: "Account menu, Ada Lovelace, Busy".
pub fn account_label(name: &str, status: Option<Presence>) -> String {
    match status {
        Some(s) => format!("Account menu, {name}, {}", s.label()),
        None => format!("Account menu, {name}"),
    }
}

// ─────────────────────────────────────── code block ──

/// A highlighted token kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    /// Plain text.
    Plain,
    /// Keyword.
    Keyword,
    /// String literal.
    Str,
    /// Number.
    Num,
    /// Comment.
    Comment,
    /// Function name (an identifier before `(`).
    Func,
    /// Type (capitalised identifier).
    Type,
    /// Punctuation.
    Punct,
    /// Diff added line.
    Add,
    /// Diff removed line.
    Del,
}

impl Tok {
    /// CSS class suffix.
    pub fn id(self) -> &'static str {
        match self {
            Self::Plain => "p",
            Self::Keyword => "k",
            Self::Str => "s",
            Self::Num => "n",
            Self::Comment => "c",
            Self::Func => "f",
            Self::Type => "t",
            Self::Punct => "u",
            Self::Add => "a",
            Self::Del => "d",
        }
    }
}

fn keywords(lang: &str) -> &'static [&'static str] {
    match lang {
        "rust" | "rs" => &[
            "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "match",
            "if", "else", "for", "while", "loop", "return", "self", "Self", "async", "await",
            "move", "where", "const", "static", "crate", "super", "as", "in", "true", "false",
            "dyn", "ref", "type",
        ],
        "py" | "python" => &[
            "def", "class", "import", "from", "return", "if", "elif", "else", "for", "while", "in",
            "not", "and", "or", "with", "as", "async", "await", "None", "True", "False", "lambda",
            "yield", "try", "except", "raise", "pass",
        ],
        "sh" | "bash" | "shell" => &[
            "if", "then", "fi", "for", "do", "done", "case", "esac", "export", "echo", "cd", "npm",
            "cargo", "git", "pnpm", "npx",
        ],
        "json" => &["true", "false", "null"],
        _ => &[
            "const",
            "let",
            "var",
            "function",
            "return",
            "if",
            "else",
            "for",
            "while",
            "import",
            "from",
            "export",
            "default",
            "async",
            "await",
            "new",
            "class",
            "extends",
            "type",
            "interface",
            "true",
            "false",
            "null",
            "undefined",
            "this",
            "typeof",
        ],
    }
}

/// Highlights `code` into lines of `(kind, text)` runs. A light, dependency-free tokenizer:
/// strings, numbers, comments (`//`, `#`), keywords, calls, capitalised types and punctuation;
/// `diff` marks whole lines added or removed.
pub fn highlight(code: &str, lang: &str) -> Vec<Vec<(Tok, String)>> {
    let kw = keywords(lang);
    let hash_comment = matches!(lang, "py" | "python" | "sh" | "bash" | "shell" | "toml");
    code.lines()
        .map(|line| {
            if lang == "diff" {
                let k = if line.starts_with('+') {
                    Tok::Add
                } else if line.starts_with('-') {
                    Tok::Del
                } else {
                    Tok::Plain
                };
                return vec![(k, line.to_string())];
            }
            let cs: Vec<char> = line.chars().collect();
            let mut out: Vec<(Tok, String)> = Vec::new();
            let mut push = |k: Tok, s: String| {
                if let Some(last) = out.last_mut()
                    && last.0 == k
                {
                    last.1.push_str(&s);
                    return;
                }
                out.push((k, s));
            };
            let mut i = 0;
            while i < cs.len() {
                let c = cs[i];
                if (c == '/' && cs.get(i + 1) == Some(&'/')) || (hash_comment && c == '#') {
                    push(Tok::Comment, cs[i..].iter().collect());
                    break;
                }
                if c == '"' || c == '\'' || c == '`' {
                    let mut j = i + 1;
                    while j < cs.len() && cs[j] != c {
                        if cs[j] == '\\' {
                            j += 1;
                        }
                        j += 1;
                    }
                    let j = (j + 1).min(cs.len());
                    push(Tok::Str, cs[i..j].iter().collect());
                    i = j;
                    continue;
                }
                if c.is_ascii_digit() {
                    let mut j = i;
                    while j < cs.len()
                        && (cs[j].is_ascii_alphanumeric() || cs[j] == '.' || cs[j] == '_')
                    {
                        j += 1;
                    }
                    push(Tok::Num, cs[i..j].iter().collect());
                    i = j;
                    continue;
                }
                if c.is_alphabetic() || c == '_' {
                    let mut j = i;
                    while j < cs.len() && (cs[j].is_alphanumeric() || cs[j] == '_') {
                        j += 1;
                    }
                    let w: String = cs[i..j].iter().collect();
                    let k = if kw.contains(&w.as_str()) {
                        Tok::Keyword
                    } else if cs.get(j) == Some(&'(') || cs.get(j) == Some(&'!') {
                        Tok::Func
                    } else if w.chars().next().is_some_and(char::is_uppercase) {
                        Tok::Type
                    } else {
                        Tok::Plain
                    };
                    push(k, w);
                    i = j;
                    continue;
                }
                if "{}()[];,.:<>=+-*/&|!?".contains(c) {
                    push(Tok::Punct, c.to_string());
                } else {
                    push(Tok::Plain, c.to_string());
                }
                i += 1;
            }
            out
        })
        .collect()
}

/// Whether a source needs the Show all toggle at `max_lines` (0 = never).
pub fn code_collapses(code: &str, max_lines: usize) -> bool {
    max_lines > 0 && code.lines().count() > max_lines
}

/// Header label for a language id.
pub fn language_label(lang: &str) -> &'static str {
    match lang {
        "rust" | "rs" => "Rust",
        "ts" | "typescript" => "TypeScript",
        "tsx" => "TSX",
        "js" | "javascript" => "JavaScript",
        "py" | "python" => "Python",
        "sh" | "bash" | "shell" => "Shell",
        "json" => "JSON",
        "diff" => "Diff",
        "toml" => "TOML",
        _ => "Text",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactions_toggle_and_leave() {
        let mut r = Reactions::new(
            vec![
                ReactionOption::new("love", "Love", "♥"),
                ReactionOption::new("fire", "Fire", "🔥"),
            ],
            vec![ReactionCount {
                id: "love".into(),
                count: 1,
                reacted: true,
            }],
        );
        assert_eq!(r.toggle("love"), (false, Some(0)));
        assert!(r.value.is_empty());
        r.toggle("fire");
        assert_eq!(
            r.pill_label(&r.value[0]),
            "Fire, 1 reaction, including yours"
        );
        assert_eq!(r.announce(), "Added Fire");
        r.set_picker(true);
        r.key(NavKey::Prev);
        assert_eq!(r.focus, Some(1));
    }

    #[test]
    fn notification_center_flow() {
        let mut c = NotificationCenter::new(vec![
            Note::new("a", "Atlas", "Run finished", "…", "Today", "2m"),
            Note::new("b", "Sam", "Mentioned you", "…", "Today", "1h").read(),
            Note::new("c", "Atlas", "Approval", "…", "Earlier", "1d"),
        ]);
        assert_eq!(c.unread(), 2);
        assert_eq!(c.groups().len(), 2);
        c.view = NoteView::Unread;
        assert_eq!(c.groups()[0].1.len(), 1);
        c.toggle("a");
        assert_eq!(c.unread(), 1);
        assert_eq!(c.dismiss("b"), Some("c".into()));
        assert_eq!(c.mark_all(), 1);
        assert_eq!(c.trigger_label(), "Notifications");
    }

    fn ts() -> TeamSwitcher {
        TeamSwitcher::new(
            vec![
                Workspace::new("acme", "Acme").plan("Pro").role("Owner"),
                Workspace::new("lab", "Lab").plan("Free").role("Member"),
            ],
            None,
        )
    }

    #[test]
    fn switcher_search_create_escape() {
        let mut s = ts();
        s.open_menu();
        s.set_query("member");
        assert_eq!(
            s.rows(),
            vec![SwitcherRow::Workspace(1), SwitcherRow::Create]
        );
        s.set_query("zeta");
        assert_eq!(s.enter(ClockMs(0)), None);
        assert_eq!(s.view, SwitcherView::Create);
        assert_eq!(s.draft, "zeta");
        assert!(s.escape());
        assert!(s.escape());
        assert!(s.query.is_empty());
        assert!(s.escape());
        assert!(!s.open);
    }

    #[test]
    fn switcher_switch_and_settle() {
        let mut s = ts();
        assert_eq!(s.shortcut(2, ClockMs(0)), Some("lab".into()));
        assert_eq!(s.settle(ClockMs(100), SWITCH_DELAY), None);
        assert_eq!(s.settle(ClockMs(700), SWITCH_DELAY), Some("lab".into()));
        assert_eq!(s.active, "lab");
        assert!(s.trigger_label().starts_with("Lab, Free plan"));
        s.draft = "New Team".into();
        assert_eq!(s.create(ClockMs(0)), Some("new-team".into()));
    }

    #[test]
    fn user_menu_rows_and_switches() {
        let mut m = UserMenu::new(&["Profile", "Settings", "Billing"]);
        m.show_status = true;
        assert_eq!(m.rows().len(), 6);
        m.open_menu(true, true);
        assert_eq!(m.highlight, Some(5));
        m.key(NavKey::Prev);
        assert!(m.cycle(true));
        assert_eq!(m.theme, ThemePref::Light);
        m.key(NavKey::Prev);
        m.cycle(false);
        assert_eq!(m.status, Presence::Away);
        m.type_char('s', ClockMs(0));
        assert_eq!(m.highlight, Some(5));
        assert_eq!(
            account_label("Ada", Some(Presence::Busy)),
            "Account menu, Ada, Busy"
        );
    }

    #[test]
    fn highlighter_tokens() {
        let l = highlight("let x = foo(\"hi\", 42); // done", "rust");
        let kinds: Vec<Tok> = l[0].iter().map(|t| t.0).collect();
        assert!(kinds.contains(&Tok::Keyword));
        assert!(kinds.contains(&Tok::Func));
        assert!(kinds.contains(&Tok::Str));
        assert!(kinds.contains(&Tok::Num));
        assert_eq!(l[0].last().unwrap().0, Tok::Comment);
        let d = highlight("+ a\n- b\n c", "diff");
        assert_eq!(d[0][0].0, Tok::Add);
        assert_eq!(d[1][0].0, Tok::Del);
        assert!(code_collapses("a\nb\nc", 2));
        assert_eq!(language_label("rs"), "Rust");
        let s: String = l[0].iter().map(|t| t.1.as_str()).collect();
        assert_eq!(s, "let x = foo(\"hi\", 42); // done");
    }
}
