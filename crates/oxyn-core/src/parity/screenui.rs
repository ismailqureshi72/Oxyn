//! Interaction state for the screen sets S0–S8 and S12–S15 (the app shells, AI Chat, Workspace,
//! Agent ops I and II, Orchestration / Workflow / Evals, Settings I and II, Onboarding, Arc
//! screens I and II)
//! as one renderer-free
//! model, so a GPUI view and any other host drive every screen with the same rules: which set and
//! screen are open, the screen's state (its state bar: Data, Loading, Empty and its own states),
//! the account menu, the command bar and the last thing said.
//!
//! The Arc screens' own presses (`mx-*`, `kb-*`, `pb-*`, `gt-*`, `td-*`) go first to the live
//! [`ArcUi`]: range, board moves, tasks, bars and filters; the S13 presses (`cal-*`, `ib-*`,
//! `sp-*`, `hc-*`) to [`Arc2Ui`]: calendar views and moves, triage, replies and articles.
//!
//! Presses arrive as the screens' own ids (`nav-<key>`, `set-<page>`, `open-<page>`,
//! `state-<key>`, `continue`, `export`…). [`ScreenUi::press`] routes them in this order: the
//! screen's own action table (`export` opens Exporting), a state of the open screen, a screen of
//! the open set whose keys match, a screen of any set, then the shared ids (`acct`, `back`,
//! `continue`). Anything else is kept in `last` and said back, so a press never goes unanswered.
//!
//! Keys follow the agent apps ([`super::appui::AppKey`]): Left / Right step through the open
//! screen's states, Up / Down through the set's screens, Escape closes the menu and then returns to
//! the first state, Enter runs the command bar.

use super::appui::AppKey;
use super::arcscr::{self as a12, ArcUi};
use super::arcscr2::{self as b13, Arc2Ui};
use super::arcscr3::{self as c14, Arc3Ui};
use super::arcscr4::{self as d15, Arc4Ui};
use super::screensfid::ScreenState;

/// A state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

const STAGE: StateList = &super::screensfid::STAGE_STATES;

/// One screen: slug, name, states, the press keys that open it, and its own presses
/// (id → state key).
#[derive(Clone, Copy, Debug)]
pub struct ScreenDef {
    /// The screen's slug (as on the site).
    pub slug: &'static str,
    /// Its name.
    pub name: &'static str,
    /// Its state bar.
    pub states: StateList,
    /// Press keys that open it (`files` for `nav-files`).
    pub keys: &'static [&'static str],
    /// Its own presses: (id, state key it opens).
    pub acts: &'static [(&'static str, &'static str)],
}

/// One screen set.
#[derive(Clone, Copy, Debug)]
pub struct SetDef {
    /// Short id (`s0` … `s8`).
    pub id: &'static str,
    /// Its name.
    pub name: &'static str,
    /// Steps in order with Continue / Back (the onboarding wizard).
    pub wizard: bool,
    /// Its screens.
    pub screens: &'static [ScreenDef],
}

const fn sc(
    slug: &'static str,
    name: &'static str,
    states: StateList,
    keys: &'static [&'static str],
    acts: &'static [(&'static str, &'static str)],
) -> ScreenDef {
    ScreenDef {
        slug,
        name,
        states,
        keys,
        acts,
    }
}

use super::{agentscr as a3, buildscr as b5, opsscr as o4, screensfid as s1, setscr as s6};
use super::{setscr2 as s7, workscr as w2};

/// Every screen set, S0 to S8, then S12 and S13.
pub const SETS: &[SetDef] = &[
    SetDef {
        id: "s0",
        name: "App shells",
        wizard: false,
        screens: &[
            sc("shell-inset", "App shell", STAGE, &["home"], &[]),
            sc("shell-three-pane", "Three-pane shell", STAGE, &[], &[]),
            sc("settings", "Settings shell", STAGE, &["settings"], &[]),
        ],
    },
    SetDef {
        id: "s1",
        name: "AI Chat",
        wizard: false,
        screens: &[
            sc(
                "chat-start",
                "Start",
                &s1::START_STATES,
                &["new"],
                &[("plus-open", "attached"), ("mic", "dictating")],
            ),
            sc(
                "chat-overview",
                "Overview",
                &s1::OVERVIEW_STATES,
                &["home"],
                &[],
            ),
            sc(
                "chat-agents",
                "Agents",
                &s1::AGENTS_STATES,
                &["made", "chat"],
                &[("suggest", "making")],
            ),
            sc(
                "chat-double-sidebar",
                "Double sidebar",
                &s1::RAIL_STATES,
                &["stats"],
                &[("fold", "folded")],
            ),
            sc("chat-progress", "Progress", &s1::PROGRESS_STATES, &[], &[]),
            sc(
                "chat-run",
                "Run",
                &s1::RUN_STATES,
                &["runs"],
                &[("approve", "done")],
            ),
        ],
    },
    SetDef {
        id: "s2",
        name: "Workspace",
        wizard: false,
        screens: &[
            sc(
                "folders-overview",
                "Folders",
                &w2::FILES_STATES,
                &["files", "home"],
                &[("create-folder", "creating")],
            ),
            sc(
                "folders-detail",
                "Folder detail",
                &w2::FOLDER_STATES,
                &["autumn", "open-1", "open-2", "open-3", "open-4"],
                &[("upload", "uploading")],
            ),
            sc(
                "integrations",
                "Integrations",
                &w2::INTEG_STATES,
                &["apps"],
                &[],
            ),
            sc(
                "kanban",
                "Kanban",
                &w2::KANBAN_STATES,
                &["board", "projects", "mine"],
                &[],
            ),
            sc(
                "crm",
                "CRM",
                &w2::CRM_STATES,
                &["companies", "contacts", "deals"],
                &[],
            ),
            sc(
                "analytics",
                "Analytics",
                &w2::ANALYTICS_STATES,
                &[
                    "revenue",
                    "reports",
                    "invoices",
                    "payouts",
                    "customers",
                    "analytics",
                ],
                &[],
            ),
        ],
    },
    SetDef {
        id: "s3",
        name: "Agent ops I",
        wizard: false,
        screens: &[
            sc(
                "agent-thread",
                "Thread",
                a3::THREAD_STATES,
                &["thread", "task-1"],
                &[("retry", "sample")],
            ),
            sc(
                "inbox",
                "Inbox",
                a3::INBOX_STATES,
                &["inbox"],
                &[
                    ("pick-1", "answered"),
                    ("pick-2", "answered"),
                    ("pick-3", "answered"),
                ],
            ),
            sc(
                "agent-inbox-board",
                "Inbox board",
                a3::BOARD_STATES,
                &[],
                &[],
            ),
            sc("agent-tasks", "Tasks", a3::TASKS_STATES, &["tasks"], &[]),
            sc(
                "agent-activity",
                "Activity",
                a3::ACTIVITY_STATES,
                &["activity"],
                &[("undo", "undone")],
            ),
            sc(
                "agent-memory",
                "Memory",
                a3::MEMORY_STATES,
                &["memory"],
                &[],
            ),
        ],
    },
    SetDef {
        id: "s4",
        name: "Agent ops II",
        wizard: false,
        screens: &[
            sc(
                "agent-agents",
                "Agents",
                o4::AGENTS_STATES,
                &["agents"],
                &[("new-agent", "creating")],
            ),
            sc(
                "agent-skills",
                "Skills",
                o4::SKILLS_STATES,
                &["skills"],
                &[("install", "installing")],
            ),
            sc(
                "agent-schedules",
                "Schedules",
                o4::SCHED_STATES,
                &["schedules"],
                &[("run-now", "running")],
            ),
            sc("agent-usage", "Usage", o4::USAGE_STATES, &["usage"], &[]),
            sc(
                "agent-permissions",
                "Permissions",
                o4::PERM_STATES,
                &["permissions"],
                &[
                    ("level-handsoff", "handsoff"),
                    ("level-balanced", "sample"),
                    ("level-careful", "sample"),
                ],
            ),
            sc(
                "agent-run-detail",
                "Run detail",
                o4::RUN_STATES,
                &["runs", "run"],
                &[("stop", "stopped")],
            ),
        ],
    },
    SetDef {
        id: "s5",
        name: "Orchestration, Workflow, Evals",
        wizard: false,
        screens: &[
            sc("agent-orchestration", "Orchestration", STAGE, &[], &[]),
            sc("workflow", "Workflow builder", STAGE, &["workflow"], &[]),
            sc(
                "evals",
                "Evals",
                b5::EVALS_STATES,
                &["evals"],
                &[("run-eval", "running"), ("accept-all", "sample")],
            ),
        ],
    },
    SetDef {
        id: "s6",
        name: "Settings I",
        wizard: false,
        screens: &[
            sc(
                "settings-profile",
                "Profile",
                s6::PROFILE_STATES,
                &["profile"],
                &[("save", "sample"), ("discard", "sample")],
            ),
            sc(
                "settings-preferences",
                "Preferences",
                s6::PREFS_STATES,
                &["preferences"],
                &[("save", "sample"), ("discard", "sample")],
            ),
            sc(
                "settings-notifications",
                "Notifications",
                s6::NOTIF_STATES,
                &["notifications"],
                &[],
            ),
            sc(
                "settings-privacy",
                "Privacy",
                s6::PRIVACY_STATES,
                &["privacy"],
                &[("export", "exporting")],
            ),
            sc(
                "settings-general",
                "General",
                s6::GENERAL_STATES,
                &["general"],
                &[("delete-workspace", "deleting")],
            ),
        ],
    },
    SetDef {
        id: "s7",
        name: "Settings II",
        wizard: false,
        screens: &[
            sc(
                "settings-members",
                "Members",
                s7::MEMBERS_STATES,
                &["members", "teams"],
                &[("invite", "inviting")],
            ),
            sc(
                "settings-models",
                "Models",
                s7::MODELS_STATES,
                &["models"],
                &[("add-key", "newkey"), ("rotate-key", "newkey")],
            ),
            sc(
                "settings-security",
                "Security",
                s7::SECURITY_STATES,
                &["security"],
                &[("setup-two-step", "twostep")],
            ),
            sc(
                "settings-billing",
                "Billing",
                s7::BILLING_STATES,
                &["billing"],
                &[("cancel-plan", "cancelling")],
            ),
            sc(
                "settings-onboarding",
                "Get started",
                s7::START_STATES,
                &["onboarding"],
                &[],
            ),
        ],
    },
    SetDef {
        id: "s8",
        name: "Onboarding",
        wizard: true,
        screens: &[
            sc(
                "onboarding-website",
                "Website",
                ONB_WEBSITE,
                &["website", "edit-website"],
                &[("fetch", "fetching")],
            ),
            sc(
                "onboarding-brand",
                "Brand",
                ONB_BRAND,
                &["brand", "edit-brand", "edit-name"],
                &[("add", "custom")],
            ),
            sc(
                "onboarding-tone",
                "Tone",
                ONB_TONE,
                &["tone", "edit-voice", "edit-handover"],
                &[("test", "testing")],
            ),
            sc(
                "onboarding-ready",
                "Ready",
                ONB_READY,
                &["ready"],
                &[("open-dashboard", "live")],
            ),
        ],
    },
    SetDef {
        id: "s12",
        name: "Arc screens I",
        wizard: false,
        screens: &[
            sc(
                "metrics-dashboard",
                "Metrics",
                a12::METRICS_STATES,
                &["overview", "metrics"],
                &[("export", "exported")],
            ),
            sc(
                "kanban-board",
                "Kanban board",
                a12::KANBAN_STATES,
                &["cycle", "issues"],
                &[],
            ),
            sc(
                "project-board",
                "Project board",
                a12::PROJECT_STATES,
                &["project", "mytasks"],
                &[],
            ),
            sc(
                "gantt-timeline",
                "Gantt",
                a12::GANTT_STATES,
                &["timeline", "gantt"],
                &[],
            ),
            sc(
                "team-directory",
                "Team directory",
                a12::TEAM_STATES,
                &["people", "directory"],
                &[("invite", "inviting")],
            ),
        ],
    },
    SetDef {
        id: "s13",
        name: "Arc screens II",
        wizard: false,
        screens: &[
            sc(
                "week-calendar",
                "Week calendar",
                b13::WEEK_STATES,
                &["weekcal", "calendar-week"],
                &[],
            ),
            sc(
                "month-calendar",
                "Month calendar",
                b13::MONTH_STATES,
                &["monthcal", "calendar-month"],
                &[],
            ),
            sc(
                "inbox-triage",
                "Inbox triage",
                b13::INBOX_STATES,
                &["triage", "ibx"],
                &[],
            ),
            sc(
                "support-conversation",
                "Support",
                b13::SUPPORT_STATES,
                &["support", "tickets"],
                &[],
            ),
            sc(
                "help-center",
                "Help center",
                b13::HELP_STATES,
                &["helpcenter", "articles"],
                &[],
            ),
        ],
    },
    SetDef {
        id: "s14",
        name: "Arc screens III",
        wizard: false,
        screens: &[
            sc(
                "market-terminal",
                "Market terminal",
                c14::MARKET_STATES,
                &["market", "trade"],
                &[],
            ),
            sc(
                "product-list",
                "Product list",
                c14::PLIST_STATES,
                &["products", "catalog"],
                &[],
            ),
            sc(
                "product-detail",
                "Product detail",
                c14::PDETAIL_STATES,
                &["product"],
                &[],
            ),
            sc(
                "invoice-studio",
                "Invoice studio",
                c14::INVOICE_STATES,
                &["invoicestudio", "invoice"],
                &[],
            ),
            sc(
                "sign-in-split",
                "Sign-in split",
                c14::SIGNIN_STATES,
                &["signin", "login"],
                &[],
            ),
        ],
    },
    SetDef {
        id: "s15",
        name: "Arc screens IV",
        wizard: false,
        screens: &[
            sc(
                "admin-overview",
                "Overview",
                d15::OVERVIEW_STATES,
                &["overview", "home"],
                &[],
            ),
            sc("audit-log", "Audit log", d15::AUDIT_STATES, &["audit"], &[]),
            sc(
                "status-board",
                "Status",
                d15::STATUS_STATES,
                &["status"],
                &[],
            ),
            sc(
                "api-keys-webhooks",
                "API keys and webhooks",
                d15::KEYS_STATES,
                &["keys", "webhooks"],
                &[],
            ),
            sc(
                "members-roles",
                "Members",
                d15::MEMBERS_STATES,
                &["members"],
                &[],
            ),
            sc(
                "billing-usage",
                "Billing",
                d15::BILLING_STATES,
                &["billing"],
                &[],
            ),
            sc(
                "agent-tool-calls",
                "Agent / Tool calls",
                d15::TOOLS_STATES,
                &["tools", "agent"],
                &[],
            ),
        ],
    },
];

const ONB_WEBSITE: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("fetching", "Reading site"),
];
const ONB_BRAND: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("custom", "Own colour"),
];
const ONB_TONE: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("testing", "Test reply"),
];
const ONB_READY: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("live", "Live"),
];

/// Split a state key into the data state and the screen's own state (`""` for none).
pub fn split(k: &'static str) -> (ScreenState, &'static str) {
    match k {
        "sample" | "loading" | "empty" => (ScreenState::from_key(k), ""),
        own => (ScreenState::Sample, own),
    }
}

/// The set and screen index of a slug.
pub fn find(slug: &str) -> Option<(usize, usize)> {
    SETS.iter().enumerate().find_map(|(si, s)| {
        s.screens
            .iter()
            .position(|x| x.slug == slug)
            .map(|i| (si, i))
    })
}

fn bare(id: &str) -> &str {
    for p in ["nav-", "set-", "open-", "state-", "edit-"] {
        if let Some(r) = id.strip_prefix(p)
            && !r.is_empty()
            && !r.chars().all(|c| c.is_ascii_digit())
        {
            return r;
        }
    }
    id
}

/// Lower-case words of a press id, for saying it back (`delete-workspace` → `delete workspace`).
pub fn words(id: &str) -> String {
    id.replace(['-', '_', ':'], " ")
}

/// What a press or key did, for the host to redraw or say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// Nothing changed.
    None,
    /// The open screen's state changed.
    State,
    /// Another screen (maybe in another set) opened.
    Screen,
    /// A menu or the command bar changed.
    Chrome,
    /// The press was not routed; it is in `last`.
    Kept,
}

/// The screens' interaction state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenUi {
    /// Open set, an index into [`SETS`].
    pub set: usize,
    /// Open screen in the set.
    pub screen: usize,
    /// Open state, an index into the screen's state bar.
    pub state: usize,
    /// Account menu open.
    pub menu: bool,
    /// Command bar text.
    pub query: String,
    /// The last press no rule handled.
    pub last: Option<&'static str>,
    /// What the screen last said (a short line the host shows).
    pub said: Option<String>,
    /// Screens opened before, newest last (Back walks them).
    pub trail: Vec<(usize, usize)>,
    /// The Arc screens' live state (S12).
    pub arc: ArcUi,
    /// The Arc screens II live state (S13).
    pub arc2: Arc2Ui,
    /// The Arc screens III live state (S14).
    pub arc3: Arc3Ui,
    /// The Arc screens IV live state (S15).
    pub arc4: Arc4Ui,
}

impl Default for ScreenUi {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ScreenUi {
    /// Opens the first screen of set `set` on Data.
    pub fn new(set: usize) -> Self {
        Self {
            set: set.min(SETS.len() - 1),
            screen: 0,
            state: 0,
            menu: false,
            query: String::new(),
            last: None,
            said: None,
            trail: Vec::new(),
            arc: ArcUi::default(),
            arc2: Arc2Ui::default(),
            arc3: Arc3Ui::default(),
            arc4: Arc4Ui::default(),
        }
    }

    /// The open set.
    pub fn set_def(&self) -> &'static SetDef {
        &SETS[self.set]
    }

    /// The open screen.
    pub fn screen_def(&self) -> &'static ScreenDef {
        &self.set_def().screens[self.screen]
    }

    /// The open state's key.
    pub fn state_key(&self) -> &'static str {
        self.screen_def()
            .states
            .get(self.state)
            .map_or("sample", |s| s.0)
    }

    /// The open state as data state and own state.
    pub fn split(&self) -> (ScreenState, &'static str) {
        split(self.state_key())
    }

    /// Opens state `k` of the open screen; false when it has no such state or it is open.
    pub fn open_state(&mut self, k: &str) -> bool {
        match self.screen_def().states.iter().position(|s| s.0 == k) {
            Some(i) if i != self.state => {
                self.state = i;
                self.menu = false;
                true
            }
            _ => false,
        }
    }

    /// Opens screen `i` of set `set` on its first state; false when it is already open.
    pub fn open(&mut self, set: usize, i: usize) -> bool {
        let set = set.min(SETS.len() - 1);
        let i = i.min(SETS[set].screens.len() - 1);
        if (set, i) == (self.set, self.screen) {
            return false;
        }
        self.trail.push((self.set, self.screen));
        if self.trail.len() > 32 {
            self.trail.remove(0);
        }
        self.set = set;
        self.screen = i;
        self.state = 0;
        self.menu = false;
        self.said = None;
        true
    }

    /// Opens the screen with this slug.
    pub fn open_slug(&mut self, slug: &str) -> bool {
        find(slug).is_some_and(|(s, i)| self.open(s, i))
    }

    fn screen_for(&self, k: &str) -> Option<(usize, usize)> {
        let hit = |s: &ScreenDef| s.keys.contains(&k) || s.slug == k;
        if let Some(i) = self.set_def().screens.iter().position(hit) {
            return Some((self.set, i));
        }
        SETS.iter()
            .enumerate()
            .find_map(|(si, s)| s.screens.iter().position(hit).map(|i| (si, i)))
    }

    /// Routes a press (see the module docs).
    pub fn press(&mut self, id: &'static str) -> Moved {
        if self.arc.press(id) {
            self.said = self.arc.said.take();
            self.menu = false;
            return Moved::State;
        }
        if self.arc2.press(id) {
            self.said = self.arc2.said.take();
            self.menu = false;
            return Moved::State;
        }
        if self.arc3.press(id) {
            self.said = self.arc3.said.take();
            self.menu = false;
            return Moved::State;
        }
        if self.arc4.press(id) {
            self.said = self.arc4.said.take();
            self.menu = false;
            return Moved::State;
        }
        let sd = self.screen_def();
        if let Some(&(_, k)) = sd.acts.iter().find(|(a, _)| *a == id) {
            self.said = Some(words(id));
            return if self.open_state(k) {
                Moved::State
            } else {
                Moved::Chrome
            };
        }
        let k = bare(id);
        if id.starts_with("state-") || sd.states.iter().any(|s| s.0 == k) {
            return if self.open_state(k) {
                Moved::State
            } else {
                Moved::None
            };
        }
        if let Some((s, i)) = self.screen_for(id).or_else(|| self.screen_for(k)) {
            return if self.open(s, i) {
                Moved::Screen
            } else {
                Moved::None
            };
        }
        let wiz = self.set_def().wizard;
        match id {
            "acct" => {
                self.menu = !self.menu;
                Moved::Chrome
            }
            "continue" if wiz => self.step(true),
            "back" if wiz => self.step(false),
            "back" => match self.trail.pop() {
                Some((s, i)) => {
                    self.set = s;
                    self.screen = i;
                    self.state = 0;
                    self.menu = false;
                    Moved::Screen
                }
                None => {
                    self.said = Some("back to the app".into());
                    Moved::Chrome
                }
            },
            _ => {
                self.last = Some(id);
                self.said = Some(words(id));
                self.menu = false;
                Moved::Kept
            }
        }
    }

    fn step(&mut self, fwd: bool) -> Moved {
        let n = self.set_def().screens.len();
        let to = if fwd {
            (self.screen + 1).min(n - 1)
        } else {
            self.screen.saturating_sub(1)
        };
        if self.open(self.set, to) {
            Moved::Screen
        } else {
            Moved::None
        }
    }

    /// Handles a key (see the module docs).
    pub fn key(&mut self, k: AppKey) -> Moved {
        let ns = self.screen_def().states.len();
        let nn = self.set_def().screens.len();
        match k {
            AppKey::Left | AppKey::Right if ns > 1 => {
                self.state = if k == AppKey::Right {
                    (self.state + 1) % ns
                } else {
                    (self.state + ns - 1) % ns
                };
                self.menu = false;
                Moved::State
            }
            AppKey::Up | AppKey::Down if nn > 1 => {
                let to = if k == AppKey::Down {
                    (self.screen + 1) % nn
                } else {
                    (self.screen + nn - 1) % nn
                };
                self.open(self.set, to);
                Moved::Screen
            }
            AppKey::Escape if self.menu => {
                self.menu = false;
                Moved::Chrome
            }
            AppKey::Escape if self.state != 0 => {
                self.state = 0;
                Moved::State
            }
            AppKey::Enter => {
                let q = std::mem::take(&mut self.query);
                self.run(&q)
            }
            _ => Moved::None,
        }
    }

    /// The command bar's matches for `q`: (set, screen, state) for screens and their states whose
    /// name, slug or set name contain every word of `q`; at most `max`.
    pub fn matches(q: &str, max: usize) -> Vec<(usize, usize, Option<usize>)> {
        let words: Vec<String> = q.split_whitespace().map(str::to_lowercase).collect();
        if words.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (si, s) in SETS.iter().enumerate() {
            for (i, x) in s.screens.iter().enumerate() {
                let base = format!("{} {} {} {}", s.id, s.name, x.name, x.slug).to_lowercase();
                if words.iter().all(|w| base.contains(w.as_str())) {
                    out.push((si, i, None));
                }
                for (k, (key, l)) in x.states.iter().enumerate().skip(1) {
                    let t = format!("{base} {key} {l}").to_lowercase();
                    if !words.iter().all(|w| base.contains(w.as_str()))
                        && words.iter().all(|w| t.contains(w.as_str()))
                    {
                        out.push((si, i, Some(k)));
                    }
                }
            }
        }
        out.truncate(max);
        out
    }

    /// Runs a command: opens the best match (a screen, or a screen in a state) or says what was
    /// not found.
    pub fn run(&mut self, q: &str) -> Moved {
        let q = q.trim();
        if q.is_empty() {
            return Moved::None;
        }
        match Self::matches(q, 1).first() {
            Some(&(s, i, st)) => {
                let moved = self.open(s, i);
                if let Some(k) = st {
                    self.state = k;
                }
                self.said = None;
                if moved || st.is_some() {
                    Moved::Screen
                } else {
                    Moved::None
                }
            }
            None => {
                self.said = Some(format!("no screen matches {q}"));
                Moved::Chrome
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_set_has_screens_with_a_data_state() {
        assert_eq!(SETS.len(), 13);
        for s in SETS {
            assert!(!s.screens.is_empty(), "{}", s.id);
            for x in s.screens {
                assert_eq!(x.states[0].0, "sample", "{}", x.slug);
                for (_, k) in x.acts {
                    assert!(x.states.iter().any(|s| s.0 == *k), "{} {k}", x.slug);
                }
                assert_eq!(find(x.slug).map(|(a, _)| SETS[a].id), Some(s.id));
            }
        }
    }

    #[test]
    fn presses_route_to_actions_states_and_screens() {
        let mut u = ScreenUi::new(6);
        assert_eq!(u.screen_def().slug, "settings-profile");
        assert_eq!(u.press("set-privacy"), Moved::Screen);
        assert_eq!(u.press("export"), Moved::State);
        assert_eq!(u.split(), (ScreenState::Sample, "exporting"));
        assert_eq!(u.press("state-loading"), Moved::State);
        assert_eq!(u.split(), (ScreenState::Loading, ""));
        assert_eq!(u.press("set-members"), Moved::Screen, "crosses to S7");
        assert_eq!(u.set_def().id, "s7");
        assert_eq!(u.press("invite"), Moved::State);
        assert_eq!(u.press("back"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "settings-privacy");
        assert_eq!(u.press("bell"), Moved::Kept);
        assert_eq!(u.last, Some("bell"));
        assert_eq!(u.press("acct"), Moved::Chrome);
        assert!(u.menu);
        let mut w = ScreenUi::new(3);
        assert_eq!(w.press("nav-skills"), Moved::Screen);
        assert_eq!(w.screen_def().slug, "agent-skills");
        assert_eq!(w.press("nav-inbox"), Moved::Screen);
        assert_eq!(w.press("pick-2"), Moved::State);
        assert_eq!(w.state_key(), "answered");
    }

    #[test]
    fn wizard_and_keys() {
        let mut u = ScreenUi::new(8);
        assert_eq!(u.press("back"), Moved::None);
        assert_eq!(u.press("continue"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "onboarding-brand");
        u.press("continue");
        u.press("continue");
        assert_eq!(u.press("continue"), Moved::None, "Ready is the last step");
        assert_eq!(u.press("edit-website"), Moved::Screen);
        assert_eq!(u.key(AppKey::Right), Moved::State);
        assert_eq!(u.split(), (ScreenState::Loading, ""));
        assert_eq!(u.key(AppKey::Left), Moved::State);
        assert_eq!(u.key(AppKey::Left), Moved::State);
        assert_eq!(u.state_key(), "fetching");
        assert_eq!(u.key(AppKey::Escape), Moved::State);
        assert_eq!(u.state, 0);
        assert_eq!(u.key(AppKey::Escape), Moved::None);
        assert_eq!(u.key(AppKey::Up), Moved::Screen);
        assert_eq!(u.screen_def().slug, "onboarding-ready");
        u.press("acct");
        assert_eq!(u.key(AppKey::Escape), Moved::Chrome);
        assert!(!u.menu);
    }

    #[test]
    fn arc_screens_press_live_state() {
        let mut u = ScreenUi::new(9);
        assert_eq!(u.set_def().id, "s12");
        assert_eq!(u.press("mx-range-7d"), Moved::State);
        assert_eq!(u.arc.range, 0);
        assert_eq!(u.press("export"), Moved::State);
        assert_eq!(u.state_key(), "exported");
        assert_eq!(u.press("nav-timeline"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "gantt-timeline");
        assert_eq!(u.press("gt-pick-5"), Moved::State);
        assert_eq!(u.press("gt-longer"), Moved::State);
        assert!(u.said.as_deref().unwrap_or("").contains("16 days"));
        assert_eq!(u.press("nav-people"), Moved::Screen);
        assert_eq!(u.press("invite"), Moved::State);
        assert_eq!(u.state_key(), "inviting");
    }

    #[test]
    fn arc2_screens_press_live_state() {
        let mut u = ScreenUi::new(10);
        assert_eq!(u.set_def().id, "s13");
        assert_eq!(u.screen_def().slug, "week-calendar");
        assert_eq!(u.press("cal-view-2"), Moved::State);
        assert_eq!(u.arc2.cview, 2);
        assert_eq!(u.press("nav-triage"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "inbox-triage");
        assert_eq!(u.press("ib-e"), Moved::State);
        assert!(u.said.as_deref().unwrap_or("").starts_with("Resolved"));
        assert_eq!(u.press("nav-helpcenter"), Moved::Screen);
        assert_eq!(u.press("state-noresults"), Moved::State);
        assert_eq!(u.split(), (ScreenState::Sample, "noresults"));
    }

    #[test]
    fn arc3_screens_press_live_state() {
        let mut u = ScreenUi::new(11);
        assert_eq!(u.set_def().id, "s14");
        assert_eq!(u.screen_def().slug, "market-terminal");
        assert_eq!(u.press("mk-sym-3"), Moved::State);
        assert_eq!(u.arc3.mk.sym, 3);
        assert_eq!(u.press("nav-products"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "product-list");
        assert_eq!(u.press("pl-view"), Moved::State);
        assert_eq!(u.said.as_deref(), Some("List view"));
        assert_eq!(u.press("nav-signin"), Moved::Screen);
        assert_eq!(u.press("state-wrong"), Moved::State);
        assert_eq!(u.split(), (ScreenState::Sample, "wrong"));
    }

    #[test]
    fn arc4_screens_press_live_state() {
        let mut u = ScreenUi::new(12);
        assert_eq!(u.set_def().id, "s15");
        assert_eq!(u.screen_def().slug, "admin-overview");
        assert_eq!(u.press("ov-range-0"), Moved::State);
        assert_eq!(u.said.as_deref(), Some("Last 24h"));
        assert_eq!(u.press("nav-tools"), Moved::Screen);
        assert_eq!(u.screen_def().slug, "agent-tool-calls");
        assert_eq!(u.press("tc-tool-2"), Moved::State);
        assert_eq!(u.press("tc-mode-off"), Moved::State);
        assert_eq!(u.said.as_deref(), Some("Stripe: Paused"));
        assert_eq!(u.press("nav-billing"), Moved::Screen);
        assert_eq!(u.press("state-pastdue"), Moved::State);
        assert_eq!(u.split(), (ScreenState::Sample, "pastdue"));
    }

    #[test]
    fn command_bar() {
        let mut u = ScreenUi::new(0);
        assert!(ScreenUi::matches("", 5).is_empty());
        let m = ScreenUi::matches("kanban", 5);
        assert_eq!(
            m.first().map(|x| SETS[x.0].screens[x.1].slug),
            Some("kanban")
        );
        u.query = "billing cancel".into();
        assert_eq!(u.key(AppKey::Enter), Moved::Screen);
        assert_eq!(u.screen_def().slug, "settings-billing");
        assert_eq!(u.state_key(), "cancelling");
        assert!(u.query.is_empty());
        assert_eq!(u.run("zzz nothing"), Moved::Chrome);
        assert_eq!(u.said.as_deref(), Some("no screen matches zzz nothing"));
        assert!(u.open_slug("workflow"));
        assert_eq!(u.set_def().id, "s5");
        assert!(!u.open_slug("workflow"));
    }
}
