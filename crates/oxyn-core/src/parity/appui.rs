//! Interaction state for the agent apps (Taskdeck, Command Center, Session Studio, Fleet
//! Window, Workspace Coder, Command Board, Threadbar, Tabstack, Repo Desk, Preview Desk) as one
//! renderer-free model, so the GPUI views and any other host drive the same rules: the open
//! sidebar row, the view tab, the rail, the account menu, the composer draft and its queue, and
//! the last press. Keys follow the Dioxus apps: Escape closes the account menu first, then the
//! rail; Up / Down move the open row; Left / Right step through the views; Enter sends the draft.

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

/// Press ids with an index, `<prefix><n>` for n in 0..12, as static strings for hosts whose press
/// callback takes `&'static str`.
macro_rules! id_table {
    ($(#[$m:meta])* $name:ident, $p:literal) => {
        $(#[$m])*
        pub const $name: [&str; 12] = [
            concat!($p, "0"), concat!($p, "1"), concat!($p, "2"), concat!($p, "3"),
            concat!($p, "4"), concat!($p, "5"), concat!($p, "6"), concat!($p, "7"),
            concat!($p, "8"), concat!($p, "9"), concat!($p, "10"), concat!($p, "11"),
        ];
    };
}

/// Press ids for the live parts of the apps.
pub mod ids {
    id_table!(/// Taskdeck: switch a plugin on or off, or install it.
        PLUGIN, "plugin-");
    id_table!(/// Taskdeck: switch a skill on or off.
        SKILL, "skill-");
    id_table!(/// Taskdeck: pause or resume a schedule (by list index).
        PAUSE, "pause-");
    id_table!(/// Taskdeck: run a schedule now.
        RUN, "run-");
    id_table!(/// Taskdeck: open or close a schedule's editor.
        EDIT, "edit-");
    id_table!(/// Taskdeck: cadence chip in the schedule editor.
        CADENCE, "cadence-");
    id_table!(/// Taskdeck: preview a site (by list index).
        SITE, "site-");
    id_table!(/// Command Center: open a terminal tab (by index).
        TERM, "term-");
    id_table!(/// Command Center: open a browser page (by index into the pages).
        PAGE, "page-");
    id_table!(/// Session Studio: pick a side pane (by position).
        PANE, "pane-");
    id_table!(/// Fleet Window: toggle a model in the parallel run.
        MODEL, "model-");
    id_table!(/// Workspace Coder: open a workspace tab.
        WSTAB, "wstab-");
    id_table!(/// Command Board: scrub the timeline to an event.
        SCRUB, "scrub-");

    /// The index in a `<prefix><n>` id.
    pub fn index(id: &str, prefix: &str) -> Option<usize> {
        id.strip_prefix(prefix)?.parse().ok()
    }

    /// The table id for `i` (the last id past the table).
    pub fn at(table: &[&'static str; 12], i: usize) -> &'static str {
        table[i.min(11)]
    }
}

use super::{cmdboard, cmdstudio, fleetwin, taskdeck, wscoder};

/// What the agent apps let you change in place beyond the shared [`AppUi`]: Taskdeck's plugin and
/// skill switches, schedules and site preview; Command Center's terminal tabs and browser;
/// Session Studio's pane order and side chat; Fleet Window's models; Workspace Coder's tabs;
/// Command Board's timeline scrubber and Sleep / Wake; and a clock for running timers. Every rule
/// reuses the app modules' own functions, so the Dioxus apps and these views agree.
#[derive(Clone, Debug, PartialEq)]
pub struct AppData {
    /// Taskdeck catalogue: (installed, on) per plugin.
    pub plugins: Vec<(bool, bool)>,
    /// Taskdeck: the Skills tab is open (else the catalogue).
    pub skills_tab: bool,
    /// Taskdeck skills.
    pub skills: Vec<taskdeck::Skill>,
    /// Taskdeck schedules.
    pub schedules: Vec<taskdeck::Schedule>,
    /// Taskdeck: the schedule whose editor is open (list index).
    pub editing: Option<usize>,
    /// Taskdeck: the site shown in the preview (list index).
    pub preview: Option<usize>,
    /// Command Center terminal tabs.
    pub terms: Vec<cmdstudio::Term>,
    /// The open terminal tab (index).
    pub term: usize,
    /// The browser's page (index into [`cmdstudio::PAGES`]) and the pages before it.
    pub page: usize,
    /// Pages visited before the open one.
    pub history: Vec<usize>,
    /// Session Studio's side panes in order (indexes into [`cmdstudio::PANES`]).
    pub panes: Vec<usize>,
    /// The picked pane (position), moved with the keyboard.
    pub pane: usize,
    /// The side chat: (from you, text).
    pub side: Vec<(bool, String)>,
    /// Fleet Window: models in the parallel run (indexes into [`fleetwin::FW_MODELS`]).
    pub models: Vec<usize>,
    /// Workspace Coder: the open workspace tab (index into [`wscoder::WS_TABS`]).
    pub ws_tab: usize,
    /// Command Board sessions (Sleep / Wake changes the first one).
    pub sessions: Vec<cmdboard::BSession>,
    /// Command Board: the scrubbed event (`None` is live).
    pub scrub: Option<usize>,
    /// Command Board: events on the timeline.
    pub events: usize,
    /// Seconds the view has been open, for running timers.
    pub secs: u32,
}

impl AppData {
    /// The sample data for data state `st`.
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            plugins: taskdeck::plugin_state(st),
            skills_tab: false,
            skills: taskdeck::skills(st),
            schedules: taskdeck::schedules(st),
            editing: None,
            preview: None,
            terms: cmdstudio::sample_terms(),
            term: 0,
            page: 0,
            history: Vec::new(),
            panes: (0..cmdstudio::PANES.len()).collect(),
            pane: 0,
            side: cmdstudio::SIDE_CHAT
                .iter()
                .map(|(me, t)| (*me, t.to_string()))
                .collect(),
            models: vec![0],
            ws_tab: 0,
            sessions: cmdboard::cb_sessions(st),
            scrub: None,
            events: cmdboard::cb_events(st).len(),
            secs: 0,
        }
    }

    /// Switches plugin `i` on or off, installing (and switching on) one that is not installed.
    pub fn toggle_plugin(&mut self, i: usize) -> bool {
        let Some(p) = self.plugins.get_mut(i) else {
            return false;
        };
        *p = if p.0 { (true, !p.1) } else { (true, true) };
        true
    }

    /// Switches skill `i` on or off.
    pub fn toggle_skill(&mut self, i: usize) -> bool {
        self.skills.get_mut(i).map(|s| s.on = !s.on).is_some()
    }

    /// Pauses or resumes schedule `i`.
    pub fn toggle_pause(&mut self, i: usize) -> bool {
        self.schedules
            .get_mut(i)
            .map(|s| s.paused = !s.paused)
            .is_some()
    }

    /// Runs schedule `i` now: its last run reads Running, now.
    pub fn run_now(&mut self, i: usize) -> bool {
        self.schedules
            .get_mut(i)
            .map(|s| s.last = (taskdeck::LastRun::Running, "now".into()))
            .is_some()
    }

    /// Sets the open schedule's cadence (index into [`taskdeck::CADENCES`]).
    pub fn set_cadence(&mut self, c: usize) -> bool {
        match self.editing.and_then(|i| self.schedules.get_mut(i)) {
            Some(s) if c < taskdeck::CADENCES.len() && s.cadence != c => {
                s.cadence = c;
                true
            }
            _ => false,
        }
    }

    /// Opens or closes a toggle-able index (`editing`, `preview`): the same index closes it.
    fn flip(slot: &mut Option<usize>, i: usize, n: usize) -> bool {
        if i >= n {
            return false;
        }
        *slot = if *slot == Some(i) { None } else { Some(i) };
        true
    }

    /// Runs a command in the open terminal tab.
    pub fn run_term(&mut self, cmd: &str) -> bool {
        let Some(id) = self.terms.get(self.term).map(|t| t.id) else {
            return false;
        };
        cmdstudio::run_cmd(&mut self.terms, id, cmd)
    }

    /// Opens browser page `i`, remembering the one before.
    pub fn go(&mut self, i: usize) -> bool {
        if i >= cmdstudio::PAGES.len() || i == self.page {
            return false;
        }
        self.history.push(self.page);
        self.page = i;
        true
    }

    /// Opens a typed address; false when no page has it.
    pub fn go_addr(&mut self, addr: &str) -> bool {
        cmdstudio::page_for(addr).is_some_and(|i| self.go(i))
    }

    /// Moves the picked side pane up or down one place; the pick follows it.
    pub fn move_pick(&mut self, up: bool) -> bool {
        let to = if up {
            self.pane.checked_sub(1)
        } else {
            Some(self.pane + 1).filter(|t| *t < self.panes.len())
        };
        match to {
            Some(t) if cmdstudio::move_pane(&mut self.panes, self.pane, t) => {
                self.pane = t;
                true
            }
            _ => false,
        }
    }

    /// Asks the side chat a question; the answer comes from the open session.
    pub fn ask_side(&mut self, q: &str) -> bool {
        let q = q.trim();
        if q.is_empty() {
            return false;
        }
        self.side.push((true, q.to_string()));
        self.side
            .push((false, cmdstudio::side_reply(q).to_string()));
        true
    }

    /// The first Command Board session's status after Sleep / Wake.
    pub fn toggle_sleep(&mut self) -> Option<cmdboard::SStatus> {
        self.sessions.first_mut().map(cmdboard::toggle_sleep)
    }

    /// The timeline position on show (the last event when live).
    pub fn at(&self) -> usize {
        cmdboard::scrub_at(self.events, self.scrub)
    }

    /// Steps the scrubber one event back or forward; forward past the last goes live.
    pub fn scrub_step(&mut self, back: bool) -> bool {
        if self.events == 0 {
            return false;
        }
        let at = self.at();
        self.scrub = if back {
            Some(at.saturating_sub(1))
        } else if at + 1 >= self.events {
            None
        } else {
            Some(at + 1)
        };
        true
    }

    /// One second of the running timers.
    pub fn tick(&mut self) {
        self.secs = self.secs.saturating_add(1);
    }

    /// A running timer that started `base` seconds before the view opened, as m:ss.
    pub fn clock(&self, base: u32) -> String {
        fleetwin::mmss(base.saturating_add(self.secs))
    }

    /// Handles the live press ids (see [`ids`]) plus `skills-tab`, `plugins-tab`, `term-new`,
    /// `page-back`, `pane-up`, `pane-down`, `sleep` and `live`. True when anything changed.
    pub fn press(&mut self, id: &str) -> bool {
        use ids::index;
        if let Some(i) = index(id, "plugin-") {
            return self.toggle_plugin(i);
        }
        if let Some(i) = index(id, "skill-") {
            return self.toggle_skill(i);
        }
        if let Some(i) = index(id, "pause-") {
            return self.toggle_pause(i);
        }
        if let Some(i) = index(id, "run-") {
            return self.run_now(i);
        }
        if let Some(i) = index(id, "edit-") {
            return Self::flip(&mut self.editing, i, self.schedules.len());
        }
        if let Some(i) = index(id, "cadence-") {
            return self.set_cadence(i);
        }
        if let Some(i) = index(id, "site-") {
            return Self::flip(
                &mut self.preview,
                i,
                taskdeck::sites(ScreenState::Sample).len(),
            );
        }
        if let Some(i) = index(id, "term-") {
            let ok = i < self.terms.len() && i != self.term;
            if ok {
                self.term = i;
            }
            return ok;
        }
        if let Some(i) = index(id, "page-") {
            return self.go(i);
        }
        if let Some(i) = index(id, "pane-") {
            let ok = i < self.panes.len() && i != self.pane;
            if ok {
                self.pane = i;
            }
            return ok;
        }
        if let Some(i) = index(id, "model-") {
            return fleetwin::toggle_model(&mut self.models, i);
        }
        if let Some(i) = index(id, "wstab-") {
            let ok = i < wscoder::WS_TABS.len() && i != self.ws_tab;
            if ok {
                self.ws_tab = i;
            }
            return ok;
        }
        if let Some(i) = index(id, "scrub-") {
            let at = (i < self.events).then_some(i);
            let ok = self.events > 0 && self.scrub != at;
            self.scrub = if i + 1 >= self.events { None } else { at };
            return ok;
        }
        match id {
            "skills-tab" | "plugins-tab" => {
                let on = id == "skills-tab";
                std::mem::replace(&mut self.skills_tab, on) != on
            }
            "save-schedule" | "cancel-schedule" => self.editing.take().is_some(),
            "close-preview" => self.preview.take().is_some(),
            "term-new" => {
                cmdstudio::new_term(&mut self.terms);
                self.term = self.terms.len() - 1;
                true
            }
            "page-back" => match self.history.pop() {
                Some(p) => {
                    self.page = p;
                    true
                }
                None => false,
            },
            "page-reload" => true,
            "pane-up" => self.move_pick(true),
            "pane-down" => self.move_pick(false),
            "sleep" => self.toggle_sleep().is_some(),
            "live" => self.scrub.take().is_some(),
            "scrub-back" => self.scrub_step(true),
            "scrub-fwd" => self.scrub_step(false),
            _ => false,
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
    /// Threadbar: the agent thread, the terminal thread, the Git tab.
    pub const THREADBAR: &[&str] = &["", "terminal", "git"];
    /// Tabstack: the tab's blocks, the approval card, Review after the run.
    pub const TABSTACK: &[&str] = &["", "approval", "review"];
    /// Repo Desk: the session, the Inbox, the Project.
    pub const REPO_DESK: &[&str] = &["", "inbox", "project"];
    /// Preview Desk: the session with its site, Design mode, the diff.
    pub const PREVIEW_DESK: &[&str] = &["", "design", "diff"];
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
    #[test]
    fn data_taskdeck() {
        let mut d = AppData::for_state(ScreenState::Sample);
        assert_eq!(d.plugins[2], (true, false));
        assert!(d.press(ids::PLUGIN[2]));
        assert_eq!(d.plugins[2], (true, true));
        assert_eq!(d.plugins[4], (false, false));
        d.press("plugin-4");
        assert_eq!(d.plugins[4], (true, true), "installs and switches on");
        assert!(!d.press("plugin-40"));
        let on = d.skills[0].on;
        assert!(d.press("skill-0"));
        assert_eq!(d.skills[0].on, !on);
        assert!(d.press("skills-tab") && d.skills_tab);
        assert!(!d.press("skills-tab"));
        let paused = d.schedules[0].paused;
        d.press("pause-0");
        assert_eq!(d.schedules[0].paused, !paused);
        d.press("run-1");
        assert_eq!(d.schedules[1].last.0, taskdeck::LastRun::Running);
        assert!(d.press("edit-1"));
        assert_eq!(d.editing, Some(1));
        let c = (d.schedules[1].cadence + 1) % taskdeck::CADENCES.len();
        assert!(d.press(ids::CADENCE[c]));
        assert_eq!(d.schedules[1].cadence, c);
        assert!(d.press("save-schedule"));
        assert_eq!(d.editing, None);
        assert!(!d.press(ids::CADENCE[0]), "no editor open");
        assert!(d.press("site-0") && d.preview == Some(0));
        assert!(d.press("site-0") && d.preview.is_none());
    }

    #[test]
    fn data_terminal_browser_panes() {
        let mut d = AppData::for_state(ScreenState::Sample);
        let n = d.terms[0].lines.len();
        assert!(d.run_term("pwd"));
        assert_eq!(d.terms[0].lines.len(), n + 2);
        assert!(d.press("term-new"));
        assert_eq!(d.term, d.terms.len() - 1);
        assert!(d.press("term-0"));
        assert!(!d.run_term("   "));
        assert!(d.press("page-2"));
        assert!(d.go_addr("https://localhost:3000/settings/"));
        assert_eq!(d.page, 1);
        assert!(d.press("page-back"));
        assert_eq!(d.page, 2);
        assert!(!d.go_addr("nowhere.example"));
        assert_eq!(d.panes, vec![0, 1, 2]);
        assert!(!d.press("pane-up"));
        assert!(d.press("pane-down"));
        assert_eq!((d.panes.clone(), d.pane), (vec![1, 0, 2], 1));
        assert!(d.press("pane-2"));
        assert!(d.press("pane-up"));
        assert_eq!(d.panes, vec![1, 2, 0]);
        let k = d.side.len();
        assert!(d.ask_side("Does it have a test?"));
        assert_eq!(d.side.len(), k + 2);
        assert!(d.side.last().is_some_and(|m| !m.0 && m.1.contains("test")));
    }

    #[test]
    fn data_models_tabs_timeline_clock() {
        let mut d = AppData::for_state(ScreenState::Sample);
        assert!(!d.press("model-0"), "one model stays");
        assert!(d.press("model-2"));
        assert_eq!(d.models, vec![0, 2]);
        assert!(d.press("wstab-1") && d.ws_tab == 1);
        assert!(!d.press("wstab-9"));
        assert!(d.events > 2);
        assert_eq!(d.at(), d.events - 1);
        assert!(d.press("scrub-back"));
        assert_eq!(d.at(), d.events - 2);
        assert!(d.press("scrub-0"));
        assert_eq!(d.at(), 0);
        assert!(d.press("live"));
        assert_eq!(d.scrub, None);
        assert!(d.press("scrub-fwd"), "forward at live stays live");
        assert_eq!(d.scrub, None);
        let before = d.sessions[0].status;
        assert!(d.press("sleep"));
        assert_ne!(d.sessions[0].status, before);
        assert_eq!(d.sessions[0].status, cmdboard::SStatus::Sleeping);
        d.press("sleep");
        assert_eq!(
            d.sessions[0].status,
            cmdboard::SStatus::Working,
            "wakes to working"
        );
        d.tick();
        d.tick();
        assert_eq!(d.clock(58), "1:00");
        let e = AppData::for_state(ScreenState::Empty);
        assert!(!e.clone().press("scrub-back"));
        assert!(e.skills.is_empty() && e.sessions.is_empty());
    }
}
