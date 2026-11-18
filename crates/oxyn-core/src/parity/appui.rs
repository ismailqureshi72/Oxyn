//! Interaction state for the six agent apps (Taskdeck, Command Center, Session Studio, Fleet
//! Window, Workspace Coder, Command Board) as one renderer-free model, so the GPUI views and any
//! other host drive the same rules: the open sidebar row, the view tab, the rail, the account
//! menu, the composer draft and its queue, and the last press. Keys follow the Dioxus apps:
//! Escape closes the account menu first, then the rail; Up / Down move the open row; Left / Right
//! step through the views; Enter sends the draft.

use super::screensfid::ScreenState;

/// A key the apps answer to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKey {
    /// Close the account menu, then the rail.
    Escape,
    /// Previous sidebar row.
    Up,
    /// Next sidebar row.
    Down,
    /// Previous view.
    Left,
    /// Next view.
    Right,
    /// Send the draft.
    Enter,
}

impl AppKey {
    /// The key for a platform key name (`escape`, `up`, `down`, `left`, `right`, `enter`).
    pub fn from_name(k: &str) -> Option<Self> {
        Some(match k {
            "escape" => Self::Escape,
            "up" => Self::Up,
            "down" => Self::Down,
            "left" => Self::Left,
            "right" => Self::Right,
            "enter" => Self::Enter,
            _ => return None,
        })
    }
}

/// Press ids for sidebar rows: `row-0` … `row-15`.
pub const ROW_IDS: [&str; 16] = [
    "row-0", "row-1", "row-2", "row-3", "row-4", "row-5", "row-6", "row-7", "row-8", "row-9",
    "row-10", "row-11", "row-12", "row-13", "row-14", "row-15",
];

/// The press id for sidebar row `i` (rows past the table share the last id).
pub fn row_id(i: usize) -> &'static str {
    ROW_IDS[i.min(ROW_IDS.len() - 1)]
}

/// The row a `row-N` press opens.
pub fn parse_row(id: &str) -> Option<usize> {
    id.strip_prefix("row-")?.parse().ok()
}

/// One app's interaction state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppUi {
    /// Sample, Loading or Empty.
    pub st: ScreenState,
    /// Rows in the sidebar list.
    pub rows: usize,
    /// The open row.
    pub sel: usize,
    /// The views Left / Right step through (`""` is the app's main view).
    pub views: &'static [&'static str],
    /// The open view, an index into `views`.
    pub view: usize,
    /// Sidebar collapsed to the icon rail.
    pub rail: bool,
    /// Account menu open.
    pub menu: bool,
    /// The composer text.
    pub draft: String,
    /// Sent prompts, oldest first.
    pub sent: Vec<String>,
    /// The last press the app did not handle itself (`allow`, `apply`…), for the host.
    pub last: Option<&'static str>,
}

impl AppUi {
    /// A sample-data app with `rows` sidebar rows and `views` (at least one).
    pub fn new(rows: usize, views: &'static [&'static str]) -> Self {
        Self {
            st: ScreenState::Sample,
            rows,
            sel: 0,
            views,
            view: 0,
            rail: false,
            menu: false,
            draft: String::new(),
            sent: Vec::new(),
            last: None,
        }
    }

    /// The open view's name.
    pub fn view_name(&self) -> &'static str {
        self.views.get(self.view).copied().unwrap_or("")
    }

    /// Opens the view named `v`; false if the app has no such view or it is already open.
    pub fn open_view(&mut self, v: &str) -> bool {
        match self.views.iter().position(|x| *x == v) {
            Some(i) if i != self.view => {
                self.view = i;
                self.menu = false;
                true
            }
            _ => false,
        }
    }

    /// Opens row `i` (clamped); false when nothing changed.
    pub fn open_row(&mut self, i: usize) -> bool {
        if self.rows == 0 {
            return false;
        }
        let i = i.min(self.rows - 1);
        let changed = i != self.sel || self.menu;
        self.sel = i;
        self.menu = false;
        changed
    }

    /// Sets the data state; Empty and Loading close the menu and keep the view.
    pub fn set_state(&mut self, st: ScreenState, rows: usize) {
        self.st = st;
        self.rows = rows;
        self.sel = self.sel.min(rows.saturating_sub(1));
        self.menu = false;
    }

    /// Sends the draft: trimmed, non-empty, added to `sent`. Returns the text sent.
    pub fn submit(&mut self) -> Option<String> {
        let t = self.draft.trim().to_string();
        if t.is_empty() {
            return None;
        }
        self.draft.clear();
        self.sent.push(t.clone());
        if self.st != ScreenState::Sample {
            self.st = ScreenState::Sample;
        }
        Some(t)
    }

    /// Handles a key; true when the state changed.
    pub fn key(&mut self, k: AppKey) -> bool {
        match k {
            AppKey::Escape => {
                if self.menu {
                    self.menu = false;
                    true
                } else if self.rail {
                    self.rail = false;
                    true
                } else {
                    false
                }
            }
            AppKey::Up => self.rows > 0 && self.sel > 0 && self.open_row(self.sel - 1),
            AppKey::Down => self.rows > 0 && self.open_row(self.sel + 1),
            AppKey::Left | AppKey::Right => {
                let n = self.views.len();
                if n < 2 {
                    return false;
                }
                self.view = if k == AppKey::Right {
                    (self.view + 1) % n
                } else {
                    (self.view + n - 1) % n
                };
                self.menu = false;
                true
            }
            AppKey::Enter => self.submit().is_some(),
        }
    }

    /// Handles the shared press ids: `row-N`, `acct` (toggle the menu), `rail` (toggle the rail),
    /// `view:<name>`, `send` and `new` (back to an empty first view). Any other id is kept in
    /// `last` for the app. True when the state changed.
    pub fn press(&mut self, id: &'static str) -> bool {
        if let Some(i) = parse_row(id) {
            return self.open_row(i);
        }
        if let Some(v) = id.strip_prefix("view:") {
            return self.open_view(v);
        }
        match id {
            "acct" => {
                self.menu = !self.menu;
                true
            }
            "rail" => {
                self.rail = !self.rail;
                self.menu = false;
                true
            }
            "send" => self.submit().is_some(),
            "new" => {
                self.st = ScreenState::Empty;
                self.view = 0;
                self.menu = false;
                self.draft.clear();
                true
            }
            _ => {
                self.last = Some(id);
                true
            }
        }
    }
}

/// The views each app steps through, matching the Dioxus apps' own states.
pub mod views {
    /// Taskdeck pages: Start, Board, Task, Plugins & skills, Scheduled, Sites.
    pub const TASKDECK: &[&str] = &["start", "board", "task", "plugins", "scheduled", "sites"];
    /// Command Center: the thread, worktree review, automations, terminal tabs, browser.
    pub const COMMAND_CENTER: &[&str] = &["", "worktree", "automations", "terminal", "browser"];
    /// Session Studio: the session, the permission prompt, side chat, rearrange.
    pub const SESSION_STUDIO: &[&str] = &["", "permission", "chat", "arrange"];
    /// Fleet Window: the agent, Compare, Plan.
    pub const FLEET_WINDOW: &[&str] = &["", "compare", "plan"];
    /// Workspace Coder: the running chat, then the finished one on Files or Preview.
    pub const WORKSPACE_CODER: &[&str] = &["", "files", "preview"];
    /// Command Board: updates with the question, the pull request, the planner.
    pub const COMMAND_BOARD: &[&str] = &["", "pr", "planner"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_arrows() {
        let mut u = AppUi::new(3, views::FLEET_WINDOW);
        assert!(!u.key(AppKey::Up));
        assert!(u.key(AppKey::Down));
        assert!(u.key(AppKey::Down));
        assert!(!u.key(AppKey::Down), "clamped at the last row");
        assert_eq!(u.sel, 2);
        assert!(u.press("row-0"));
        assert_eq!(u.sel, 0);
        assert!(!u.press("row-0"));
        assert!(u.open_row(9));
        assert_eq!(u.sel, 2, "clamped");
        assert_eq!(parse_row(row_id(40)), Some(15));
        assert_eq!(AppKey::from_name("escape"), Some(AppKey::Escape));
        assert_eq!(AppKey::from_name("tab"), None);
    }

    #[test]
    fn views_cycle_and_open() {
        let mut u = AppUi::new(2, views::TASKDECK);
        assert_eq!(u.view_name(), "start");
        u.key(AppKey::Left);
        assert_eq!(u.view_name(), "sites");
        u.key(AppKey::Right);
        assert_eq!(u.view_name(), "start");
        assert!(u.press("view:board"));
        assert_eq!(u.view_name(), "board");
        assert!(!u.press("view:nope"));
        let mut one = AppUi::new(1, &["only"]);
        assert!(!one.key(AppKey::Right));
    }

    #[test]
    fn escape_order_and_menu() {
        let mut u = AppUi::new(2, views::COMMAND_BOARD);
        u.press("rail");
        u.press("acct");
        assert!(u.rail && u.menu);
        assert!(u.key(AppKey::Escape));
        assert!(!u.menu && u.rail, "menu closes first");
        assert!(u.key(AppKey::Escape));
        assert!(!u.rail);
        assert!(!u.key(AppKey::Escape));
        u.press("acct");
        u.press("row-1");
        assert!(!u.menu, "opening a row closes the menu");
    }

    #[test]
    fn composer_send_and_new() {
        let mut u = AppUi::new(2, views::SESSION_STUDIO);
        assert!(!u.press("send"), "empty draft sends nothing");
        u.draft = "  Add a retry test  ".into();
        assert!(u.key(AppKey::Enter));
        assert_eq!(u.sent, vec!["Add a retry test".to_string()]);
        assert!(u.draft.is_empty());
        u.press("view:chat");
        u.press("new");
        assert_eq!((u.st, u.view), (ScreenState::Empty, 0));
        u.draft = "Start".into();
        u.press("send");
        assert_eq!(u.st, ScreenState::Sample, "sending fills an empty app");
        assert!(u.press("allow"));
        assert_eq!(u.last, Some("allow"));
        u.set_state(ScreenState::Empty, 0);
        assert_eq!(u.sel, 0);
        assert!(!u.key(AppKey::Down));
    }
}
