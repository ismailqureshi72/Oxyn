//! App shells and screens (plan #27, chunk 4), renderer-free: a resizable sidebar that snaps to an
//! icon rail, nested navigation, a workspace sidebar's favourites and projects, an inbox's folders
//! and messages, terminal tabs, the AI side panel's page edits with a word diff, and the harness.
//!
//! Behaviour follows Arc UI (Sidebar, Workspace sidebar, Sidebar rail, Inbox sidebar, AI side
//! panel), affectUI (App Shell Flat / Inset and the AI Chat screens) and Beautiful UI (Sidebar
//! Nav, MIT, © 2026 Shane Levine), rebuilt from their public docs; no source used.

/// Width of the folded icon rail (px).
pub const RAIL_W: f32 = 64.0;

/// A sidebar that can be dragged wider or narrower and snaps to an icon rail.
#[derive(Clone, Debug, PartialEq)]
pub struct Rail {
    /// Width when open (px).
    pub width: f32,
    /// Folded to the icon rail.
    pub collapsed: bool,
    /// Narrowest open width.
    pub min: f32,
    /// Widest open width.
    pub max: f32,
    /// Below this while dragging, the sidebar snaps to the rail.
    pub snap: f32,
}

impl Default for Rail {
    fn default() -> Self {
        Self {
            width: 248.0,
            collapsed: false,
            min: 200.0,
            max: 360.0,
            snap: 140.0,
        }
    }
}

impl Rail {
    /// The width the shell draws.
    pub fn shown(&self) -> f32 {
        if self.collapsed { RAIL_W } else { self.width }
    }
    /// Toggles the rail (`[` or the fold button).
    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }
    /// A drag to `x` px from the shell's left edge. Below `snap` it folds to the rail; between
    /// `snap` and `min` it holds at `min`; above `max` it holds at `max`.
    pub fn drag(&mut self, x: f32) {
        if x < self.snap {
            self.collapsed = true;
        } else {
            self.collapsed = false;
            self.width = x.clamp(self.min, self.max);
        }
    }
    /// Keyboard resize on the separator: ±16 px per press, folding past the minimum.
    pub fn nudge(&mut self, dir: i32) {
        if self.collapsed {
            if dir > 0 {
                self.collapsed = false;
                self.width = self.min;
            }
            return;
        }
        let next = self.width + 16.0 * dir as f32;
        if next < self.min {
            self.collapsed = true;
        } else {
            self.width = next.min(self.max);
        }
    }
    /// The separator's value for `aria-valuenow` (0 when folded).
    pub fn value(&self) -> u32 {
        if self.collapsed {
            0
        } else {
            self.width.round() as u32
        }
    }
}

/// A navigation row.
#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon name.
    pub icon: String,
    /// Unread or count badge (0 hides it).
    pub count: u32,
    /// Children (a group).
    pub children: Vec<NavItem>,
}

impl NavItem {
    /// A row.
    pub fn new(id: &str, label: &str, icon: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            count: 0,
            children: Vec::new(),
        }
    }
    /// With a count.
    pub fn count(mut self, n: u32) -> Self {
        self.count = n;
        self
    }
    /// With children.
    pub fn with(mut self, children: Vec<NavItem>) -> Self {
        self.children = children;
        self
    }
}

/// A labelled section of rows.
#[derive(Clone, Debug, PartialEq)]
pub struct NavSection {
    /// Heading (empty for none).
    pub title: String,
    /// Rows.
    pub items: Vec<NavItem>,
}

impl NavSection {
    /// A section.
    pub fn new(title: &str, items: Vec<NavItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// The path of labels from the section root to `id` (for breadcrumbs).
pub fn trail(sections: &[NavSection], id: &str) -> Vec<String> {
    fn walk(items: &[NavItem], id: &str, path: &mut Vec<String>) -> bool {
        for it in items {
            path.push(it.label.clone());
            if it.id == id || walk(&it.children, id, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    for s in sections {
        let mut p = Vec::new();
        if walk(&s.items, id, &mut p) {
            return p;
        }
    }
    Vec::new()
}

/// The group that contains `id`, if it is nested.
pub fn parent_of(sections: &[NavSection], id: &str) -> Option<String> {
    sections
        .iter()
        .flat_map(|s| s.items.iter())
        .find(|g| g.children.iter().any(|c| c.id == id))
        .map(|g| g.id.clone())
}

/// Rows whose label contains `q` (case-insensitive), keeping their order.
pub fn filter<'a>(items: &'a [String], q: &str) -> Vec<(usize, &'a String)> {
    let q = q.trim().to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, s)| q.is_empty() || s.to_lowercase().contains(&q))
        .collect()
}

/// Moves item `i` by `delta` (Alt+↑/↓ on favourites). Returns its new index.
pub fn shift<T>(v: &mut [T], i: usize, delta: i32) -> usize {
    if v.is_empty() || i >= v.len() {
        return i;
    }
    let j = (i as i64 + delta as i64).clamp(0, v.len() as i64 - 1) as usize;
    if i < j {
        v[i..=j].rotate_left(1);
    } else if j < i {
        v[j..=i].rotate_right(1);
    }
    j
}

/// A mail folder.
#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon.
    pub icon: String,
}

impl Folder {
    /// A folder.
    pub fn new(id: &str, label: &str, icon: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
        }
    }
}

/// A message in the inbox.
#[derive(Clone, Debug, PartialEq)]
pub struct Mail {
    /// Id.
    pub id: String,
    /// Sender.
    pub from: String,
    /// Subject.
    pub subject: String,
    /// Preview line.
    pub preview: String,
    /// Time label.
    pub when: String,
    /// Folder id.
    pub folder: String,
    /// Unread.
    pub unread: bool,
    /// Starred.
    pub starred: bool,
    /// Label (empty for none).
    pub label: String,
}

impl Mail {
    /// A message in the inbox.
    pub fn new(id: &str, from: &str, subject: &str, preview: &str, when: &str) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            subject: subject.into(),
            preview: preview.into(),
            when: when.into(),
            folder: "inbox".into(),
            unread: false,
            starred: false,
            label: String::new(),
        }
    }
    /// Unread.
    pub fn unread(mut self) -> Self {
        self.unread = true;
        self
    }
    /// Starred.
    pub fn starred(mut self) -> Self {
        self.starred = true;
        self
    }
    /// Labelled.
    pub fn label(mut self, l: &str) -> Self {
        self.label = l.into();
        self
    }
}

/// The inbox: folders, messages, the open folder and an undo slot for the last move.
#[derive(Clone, Debug, PartialEq)]
pub struct Inbox {
    /// Folders.
    pub folders: Vec<Folder>,
    /// Messages.
    pub mail: Vec<Mail>,
    /// Open folder id (`starred` is a view of starred mail).
    pub open: String,
    /// The last move: (message id, from folder).
    pub last_move: Option<(String, String)>,
}

impl Inbox {
    /// An inbox open on `inbox`.
    pub fn new(folders: Vec<Folder>, mail: Vec<Mail>) -> Self {
        Self {
            folders,
            mail,
            open: "inbox".into(),
            last_move: None,
        }
    }
    /// Messages shown in the open folder.
    pub fn visible(&self) -> Vec<&Mail> {
        self.mail
            .iter()
            .filter(|m| {
                if self.open == "starred" {
                    m.starred && m.folder != "trash"
                } else {
                    m.folder == self.open
                }
            })
            .collect()
    }
    /// Unread count for a folder (`starred` counts starred mail).
    pub fn count(&self, folder: &str) -> usize {
        if folder == "starred" {
            return self.mail.iter().filter(|m| m.starred).count();
        }
        self.mail
            .iter()
            .filter(|m| m.folder == folder && m.unread)
            .count()
    }
    /// Moves a message to a folder (drop or the Move to menu). Returns false for a no-op.
    pub fn move_to(&mut self, id: &str, folder: &str) -> bool {
        if folder == "starred" {
            return false;
        }
        let Some(m) = self.mail.iter_mut().find(|m| m.id == id) else {
            return false;
        };
        if m.folder == folder {
            return false;
        }
        self.last_move = Some((id.into(), m.folder.clone()));
        m.folder = folder.into();
        true
    }
    /// Undoes the last move.
    pub fn undo(&mut self) -> bool {
        let Some((id, from)) = self.last_move.take() else {
            return false;
        };
        if let Some(m) = self.mail.iter_mut().find(|m| m.id == id) {
            m.folder = from;
            return true;
        }
        false
    }
    /// Toggles the star (S). Returns the new state.
    pub fn star(&mut self, id: &str) -> bool {
        self.mail
            .iter_mut()
            .find(|m| m.id == id)
            .map(|m| {
                m.starred = !m.starred;
                m.starred
            })
            .unwrap_or(false)
    }
    /// Opens a message: marks it read.
    pub fn read(&mut self, id: &str) {
        if let Some(m) = self.mail.iter_mut().find(|m| m.id == id) {
            m.unread = false;
        }
    }
}

/// A terminal tab.
#[derive(Clone, Debug, PartialEq)]
pub struct TermTab {
    /// Id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// New output since it was last shown.
    pub activity: bool,
    /// The process exited with this code.
    pub exit: Option<i32>,
    /// Output lines.
    pub lines: Vec<String>,
}

/// Terminal tabs: add, close, rename, reorder, activity markers.
#[derive(Clone, Debug, PartialEq)]
pub struct TermTabs {
    /// Tabs.
    pub tabs: Vec<TermTab>,
    /// Active tab id.
    pub active: u32,
    next: u32,
}

impl TermTabs {
    /// Tabs from titles; the first is active.
    pub fn new(titles: &[&str]) -> Self {
        let tabs: Vec<TermTab> = titles
            .iter()
            .enumerate()
            .map(|(i, t)| TermTab {
                id: i as u32 + 1,
                title: t.to_string(),
                activity: false,
                exit: None,
                lines: Vec::new(),
            })
            .collect();
        Self {
            active: tabs.first().map(|t| t.id).unwrap_or(0),
            next: tabs.len() as u32 + 1,
            tabs,
        }
    }
    /// Opens a new shell tab and focuses it.
    pub fn add(&mut self) -> u32 {
        let id = self.next;
        self.next += 1;
        self.tabs.push(TermTab {
            id,
            title: format!("zsh {id}"),
            activity: false,
            exit: None,
            lines: vec!["$ ".into()],
        });
        self.active = id;
        id
    }
    /// Closes a tab; focus moves to its right neighbour, else its left.
    pub fn close(&mut self, id: u32) {
        let Some(i) = self.tabs.iter().position(|t| t.id == id) else {
            return;
        };
        self.tabs.remove(i);
        if self.active == id {
            self.active = self
                .tabs
                .get(i)
                .or_else(|| i.checked_sub(1).and_then(|j| self.tabs.get(j)))
                .map(|t| t.id)
                .unwrap_or(0);
        }
    }
    /// Activates a tab and clears its activity marker.
    pub fn select(&mut self, id: u32) {
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == id) {
            t.activity = false;
            self.active = id;
        }
    }
    /// Moves focus by `delta` tabs, wrapping (Ctrl+Tab / Ctrl+Shift+Tab).
    pub fn cycle(&mut self, delta: i32) {
        let n = self.tabs.len() as i32;
        if n == 0 {
            return;
        }
        let i = self
            .tabs
            .iter()
            .position(|t| t.id == self.active)
            .unwrap_or(0) as i32;
        let j = (i + delta).rem_euclid(n) as usize;
        let id = self.tabs[j].id;
        self.select(id);
    }
    /// Renames a tab; blank names keep the old title.
    pub fn rename(&mut self, id: u32, title: &str) {
        if title.trim().is_empty() {
            return;
        }
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == id) {
            t.title = title.trim().into();
        }
    }
    /// Output arrives on a tab; a background tab gets the activity marker.
    pub fn output(&mut self, id: u32, line: &str) {
        let active = self.active;
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == id) {
            t.lines.push(line.into());
            if id != active {
                t.activity = true;
            }
        }
    }
    /// Moves a tab by `delta` places.
    pub fn reorder(&mut self, id: u32, delta: i32) {
        if let Some(i) = self.tabs.iter().position(|t| t.id == id) {
            shift(&mut self.tabs, i, delta);
        }
    }
}

/// One word of a diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffOp {
    /// Unchanged.
    Same,
    /// Removed.
    Del,
    /// Added.
    Ins,
}

/// A word diff (LCS over whitespace-separated words), for the side panel's change card.
pub fn word_diff(before: &str, after: &str) -> Vec<(DiffOp, String)> {
    let a: Vec<&str> = before.split_whitespace().collect();
    let b: Vec<&str> = after.split_whitespace().collect();
    let (n, m) = (a.len(), b.len());
    let mut l = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            l[i][j] = if a[i] == b[j] {
                l[i + 1][j + 1] + 1
            } else {
                l[i + 1][j].max(l[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            out.push((DiffOp::Same, a[i].to_string()));
            i += 1;
            j += 1;
        } else if j < m && (i == n || l[i][j + 1] >= l[i + 1][j]) {
            out.push((DiffOp::Ins, b[j].to_string()));
            j += 1;
        } else {
            out.push((DiffOp::Del, a[i].to_string()));
            i += 1;
        }
    }
    out
}

/// Where the side panel's proposed change stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditPhase {
    /// No change proposed.
    None,
    /// A diff waits for Apply or Discard.
    Proposed,
    /// Applied; Undo is available.
    Applied,
    /// Discarded.
    Discarded,
    /// Undone.
    Undone,
}

/// A page block the side panel may edit.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// Id.
    pub id: String,
    /// Label (`Intro`, `Goals`) or a heading.
    pub label: String,
    /// Text.
    pub text: String,
}

impl Block {
    /// A block.
    pub fn new(id: &str, label: &str, text: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            text: text.into(),
        }
    }
}

/// The page beside the AI side panel and the one change in flight.
#[derive(Clone, Debug, PartialEq)]
pub struct PageEdit {
    /// Blocks.
    pub blocks: Vec<Block>,
    /// The proposed change: (block id, new text).
    pub change: Option<(String, String)>,
    /// The text a change replaced (for Undo).
    pub before: Option<String>,
    /// Phase.
    pub phase: EditPhase,
}

impl PageEdit {
    /// A page with nothing proposed.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            change: None,
            before: None,
            phase: EditPhase::None,
        }
    }
    /// Proposes new text for a block.
    pub fn propose(&mut self, id: &str, after: &str) {
        self.change = Some((id.into(), after.into()));
        self.phase = EditPhase::Proposed;
    }
    /// The diff of the proposed change.
    pub fn diff(&self) -> Vec<(DiffOp, String)> {
        match &self.change {
            Some((id, after)) => self
                .blocks
                .iter()
                .find(|b| &b.id == id)
                .map(|b| word_diff(&b.text, after))
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }
    /// Applies the proposed change. Returns the block id.
    pub fn apply(&mut self) -> Option<String> {
        if self.phase != EditPhase::Proposed {
            return None;
        }
        let (id, after) = self.change.clone()?;
        let b = self.blocks.iter_mut().find(|b| b.id == id)?;
        self.before = Some(std::mem::replace(&mut b.text, after));
        self.phase = EditPhase::Applied;
        Some(id)
    }
    /// Discards the proposed change.
    pub fn discard(&mut self) {
        if self.phase == EditPhase::Proposed {
            self.phase = EditPhase::Discarded;
        }
    }
    /// Undoes an applied change.
    pub fn undo(&mut self) -> bool {
        if self.phase != EditPhase::Applied {
            return false;
        }
        let (Some((id, _)), Some(before)) = (self.change.clone(), self.before.take()) else {
            return false;
        };
        if let Some(b) = self.blocks.iter_mut().find(|b| b.id == id) {
            b.text = before;
        }
        self.phase = EditPhase::Undone;
        true
    }
}

/// The seven AI Chat screens (affectUI), composed from chat, composer, plan and approval parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Start: greeting, starters and a composer.
    Start,
    /// Overview: recent threads and agents at a glance.
    Overview,
    /// Agents: a conversation with an agent.
    Agents,
    /// Double sidebar: an app rail plus a thread list.
    DoubleSidebar,
    /// Progress: a thread with a progress side panel.
    Progress,
    /// Run: plan, steps and an approval while the agent works.
    Run,
    /// Agent / Thread: one agent's thread with tool calls.
    Thread,
}

impl Screen {
    /// All screens in order.
    pub const ALL: [Screen; 7] = [
        Screen::Start,
        Screen::Overview,
        Screen::Agents,
        Screen::DoubleSidebar,
        Screen::Progress,
        Screen::Run,
        Screen::Thread,
    ];
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Screen::Start => "start",
            Screen::Overview => "overview",
            Screen::Agents => "agents",
            Screen::DoubleSidebar => "double",
            Screen::Progress => "progress",
            Screen::Run => "run",
            Screen::Thread => "thread",
        }
    }
    /// Parses an id (unknown ids give Start).
    pub fn from_id(s: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|x| x.id() == s)
            .unwrap_or(Screen::Start)
    }
    /// Title.
    pub fn title(self) -> &'static str {
        match self {
            Screen::Start => "Start",
            Screen::Overview => "Overview",
            Screen::Agents => "Agents",
            Screen::DoubleSidebar => "Double sidebar",
            Screen::Progress => "Progress",
            Screen::Run => "Run",
            Screen::Thread => "Agent thread",
        }
    }
}

/// A fork of a harness demo: the slug and its current control values as a query string.
pub fn fork_url(base: &str, slug: &str, vals: &[(&str, &str)]) -> String {
    let q: Vec<String> = vals
        .iter()
        .map(|(k, v)| format!("{k}={}", v.replace(' ', "+")))
        .collect();
    if q.is_empty() {
        format!("{base}/{slug}/fork")
    } else {
        format!("{base}/{slug}/fork?{}", q.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_snaps_and_clamps() {
        let mut r = Rail::default();
        r.drag(500.0);
        assert_eq!(r.width, 360.0);
        r.drag(170.0);
        assert!(!r.collapsed);
        assert_eq!(r.width, 200.0);
        r.drag(100.0);
        assert!(r.collapsed);
        assert_eq!(r.shown(), RAIL_W);
        assert_eq!(r.value(), 0);
        r.nudge(1);
        assert_eq!(r.shown(), 200.0);
        r.nudge(-1);
        assert!(r.collapsed);
    }

    #[test]
    fn trail_and_parent() {
        let s = vec![NavSection::new(
            "Workspace",
            vec![
                NavItem::new("projects", "Projects", "folder").with(vec![NavItem::new(
                    "checkout",
                    "Checkout redesign",
                    "",
                )]),
            ],
        )];
        assert_eq!(trail(&s, "checkout"), vec!["Projects", "Checkout redesign"]);
        assert_eq!(parent_of(&s, "checkout").as_deref(), Some("projects"));
        assert!(trail(&s, "nope").is_empty());
    }

    #[test]
    fn shift_moves() {
        let mut v = vec!['a', 'b', 'c', 'd'];
        assert_eq!(shift(&mut v, 0, 2), 2);
        assert_eq!(v, vec!['b', 'c', 'a', 'd']);
        assert_eq!(shift(&mut v, 2, -5), 0);
        assert_eq!(v, vec!['a', 'b', 'c', 'd']);
    }

    #[test]
    fn inbox_moves_and_undo() {
        let mut ib = Inbox::new(
            vec![
                Folder::new("inbox", "Inbox", ""),
                Folder::new("archive", "Archive", ""),
            ],
            vec![Mail::new("m1", "Emma", "Hi", "", "9:42").unread()],
        );
        assert_eq!(ib.count("inbox"), 1);
        assert!(ib.move_to("m1", "archive"));
        assert!(ib.visible().is_empty());
        assert!(!ib.move_to("m1", "archive"));
        assert!(ib.undo());
        assert_eq!(ib.visible().len(), 1);
        assert!(ib.star("m1"));
        assert_eq!(ib.count("starred"), 1);
        ib.read("m1");
        assert_eq!(ib.count("inbox"), 0);
    }

    #[test]
    fn terminal_tabs() {
        let mut t = TermTabs::new(&["server", "tests"]);
        let id = t.add();
        assert_eq!(t.active, id);
        t.output(1, "ready");
        assert!(t.tabs[0].activity);
        t.select(1);
        assert!(!t.tabs[0].activity);
        t.close(1);
        assert_eq!(t.active, 2);
        t.cycle(-1);
        assert_eq!(t.active, id);
        t.rename(2, "  ");
        assert_eq!(t.tabs[0].title, "tests");
        t.reorder(2, 1);
        assert_eq!(t.tabs[1].id, 2);
    }

    #[test]
    fn diff_and_page_edit() {
        let d = word_diff("one single place", "one place");
        assert_eq!(
            d,
            vec![
                (DiffOp::Same, "one".into()),
                (DiffOp::Del, "single".into()),
                (DiffOp::Same, "place".into())
            ]
        );
        let mut p = PageEdit::new(vec![Block::new("intro", "Intro", "old text")]);
        p.propose("intro", "new text");
        assert_eq!(p.diff().len(), 3);
        assert_eq!(p.apply().as_deref(), Some("intro"));
        assert_eq!(p.blocks[0].text, "new text");
        assert!(p.undo());
        assert_eq!(p.blocks[0].text, "old text");
        assert!(!p.undo());
    }

    #[test]
    fn screens_and_fork() {
        for s in Screen::ALL {
            assert_eq!(Screen::from_id(s.id()), s);
        }
        assert_eq!(
            fork_url("/fork", "app-shell", &[("variant", "inset")]),
            "/fork/app-shell/fork?variant=inset"
        );
    }
}
