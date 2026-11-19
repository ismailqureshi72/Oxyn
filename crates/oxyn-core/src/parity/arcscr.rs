//! Screens S12 (Arc screens I): **Metrics dashboard**, **Kanban board**, **Project board**,
//! **Gantt timeline** and **Team directory**, the work-app screens of the Arc screen frame (a
//! 248 px sidebar with a context card and an account row that shows the email, an inset page, an
//! optional right card). Behaviour and layout only: every workspace, person, project and line of
//! copy is fictional and our own, and every date is an absolute ISO date.
//!
//! Each screen's rules are plain functions over plain data, so the Dioxus screens, the GPUI
//! elements and [`ArcUi`] (the live state a GPUI view keeps) agree:
//! - Metrics: date ranges, the KPI tabs over one chart (previous period to compare, drag to zoom),
//!   the breakdown tables (pages, referrers, countries) scaled to the range;
//! - Kanban: issues in Backlog / Todo / In progress / In review / Done with work-in-progress limits,
//!   moves by drag or keys ([`nudge`]), label and text filters, grouping by assignee, the cycle's
//!   points;
//! - Project board: milestones that pick the task list, tasks to complete or reopen, the stats they
//!   drive, the team's load and the project's status;
//! - Gantt: phases of bars with progress, milestones and dependencies; drag to move, edges to
//!   resize, Day / Week / Month zoom, undo and reset, dependency conflicts;
//! - Team directory: people by team with local office times and availability, search over names,
//!   roles and skills, team / status / office filters, favourites, grid or list.

pub use super::screensfid::ScreenState;
use super::screensfid::{PERSON, PERSON_INI};

/// A state bar: (key, label).
pub type States = &'static [(&'static str, &'static str)];

/// Metrics dashboard states.
pub const METRICS_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("compare", "Compare"),
    ("zoomed", "Zoomed"),
    ("exported", "Exported"),
];
/// Kanban board states.
pub const KANBAN_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("dragging", "Dragging"),
    ("filtered", "Filtered"),
    ("assignee", "By assignee"),
];
/// Project board states.
pub const PROJECT_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("atrisk", "At risk"),
    ("activity", "Activity"),
];
/// Gantt timeline states.
pub const GANTT_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("dragging", "Dragging"),
    ("month", "Month view"),
];
/// Team directory states.
pub const TEAM_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("list", "List"),
    ("noresults", "No results"),
    ("inviting", "Invite"),
];

/// The state bar of an S12 screen by slug.
pub fn s12_states(slug: &str) -> States {
    match slug {
        "kanban-board" => KANBAN_STATES,
        "project-board" => PROJECT_STATES,
        "gantt-timeline" => GANTT_STATES,
        "team-directory" => TEAM_STATES,
        _ => METRICS_STATES,
    }
}

/// The data state and the screen's own state for a state key (own is `""` for the three
/// data states).
pub fn arc_split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 11] = [
        "compare",
        "zoomed",
        "exported",
        "dragging",
        "filtered",
        "assignee",
        "atrisk",
        "activity",
        "month",
        "list",
        "noresults",
    ];
    if k == "inviting" {
        return (ScreenState::Sample, "inviting");
    }
    match OWN.iter().find(|x| **x == k) {
        Some(x) => (ScreenState::Sample, x),
        None => (ScreenState::from_key(k), ""),
    }
}

// ───────────────────────────────────────────────────────────── shared ──

/// The signed-in person on every S12 screen.
pub const ME: &str = PERSON;
/// Their initials.
pub const ME_INI: &str = PERSON_INI;

/// One screen's workspace: name, the email domain and the context it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ws {
    /// Workspace name.
    pub name: &'static str,
    /// Email domain (`.example`).
    pub domain: &'static str,
}

/// The workspace of an Arc screen (S12 and S13).
pub fn ws(slug: &str) -> Ws {
    match slug {
        "kanban-board" => Ws {
            name: "Brindle",
            domain: "brindle.example",
        },
        "project-board" => Ws {
            name: "Marlow",
            domain: "marlow.example",
        },
        "gantt-timeline" => Ws {
            name: "Northgate",
            domain: "northgate.example",
        },
        "team-directory" => Ws {
            name: "Saltwick",
            domain: "saltwick.example",
        },
        "week-calendar" => Ws {
            name: "Ashgrove",
            domain: "ashgrove.example",
        },
        "month-calendar" => Ws {
            name: "Brackwater",
            domain: "brackwater.example",
        },
        "inbox-triage" => Ws {
            name: "Thistledown support",
            domain: "thistledown.example",
        },
        "support-conversation" => Ws {
            name: "Pinecrest",
            domain: "pinecrest.example",
        },
        "help-center" => Ws {
            name: "Quillmoor Help",
            domain: "quillmoor.example",
        },
        _ => Ws {
            name: "Lumenfield",
            domain: "lumenfield.example",
        },
    }
}

/// The signed-in person's email on screen `slug`.
pub fn my_email(slug: &str) -> String {
    format!("rowan@{}", ws(slug).domain)
}

/// The account menu of each screen: (icon, label, shortcut).
pub fn acct_menu(slug: &str) -> &'static [(&'static str, &'static str, &'static str)] {
    match slug {
        "kanban-board" => &[
            ("kbd", "Keyboard shortcuts", "?"),
            ("gear", "Settings", ""),
            ("logout", "Sign out", ""),
        ],
        "project-board" => &[
            ("tasks", "My tasks", ""),
            ("kbd", "Keyboard shortcuts", "?"),
            ("logout", "Sign out", ""),
        ],
        "gantt-timeline" => &[("undo", "Reset the plan", ""), ("logout", "Sign out", "")],
        "team-directory" => &[
            ("user", "Your profile", ""),
            ("clock", "Appear away", ""),
            ("logout", "Sign out", ""),
        ],
        "week-calendar" => &[
            ("cal", "Start weeks on Sunday", ""),
            ("kbd", "Keyboard shortcuts", "?"),
            ("logout", "Sign out", ""),
        ],
        "month-calendar" => &[
            ("eye", "Show weekends", ""),
            ("kbd", "Keyboard shortcuts", "?"),
            ("logout", "Sign out", ""),
        ],
        "inbox-triage" | "support-conversation" => &[
            ("clock", "Set yourself away", ""),
            ("kbd", "Keyboard shortcuts", "?"),
            ("logout", "Sign out", ""),
        ],
        "help-center" => &[
            ("eye", "Only my articles", ""),
            ("web", "View help site", ""),
            ("logout", "Sign out", ""),
        ],
        _ => &[
            ("download", "Export all data", ""),
            ("gear", "Settings", ""),
            ("logout", "Sign out", ""),
        ],
    }
}

/// Semantic hues (labels, project dots, avatars, statuses). Chrome stays grey.
pub const HUES: [&str; 7] = [
    "teal", "amber", "violet", "blue", "green", "rose", "magenta",
];

/// A fictional person: name and the hue of their avatar.
pub const PEOPLE: [(&str, usize); 16] = [
    ("Rowan Vale", 3),
    ("Mira Okafor", 0),
    ("Theo Lindqvist", 2),
    ("Priya Raman", 6),
    ("Jonas Weber", 1),
    ("Lena Duarte", 4),
    ("Sam Achebe", 5),
    ("Iris Novak", 2),
    ("Kai Moreno", 0),
    ("Nadia Haddad", 1),
    ("Felix Brandt", 3),
    ("Yuki Tanaka", 6),
    ("Omar Siddiqui", 4),
    ("Clara Jensen", 5),
    ("Diego Ramos", 1),
    ("Hana Kim", 2),
];

/// Initials of person `i`.
pub fn initials(i: usize) -> String {
    PEOPLE[i % PEOPLE.len()]
        .0
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect()
}

/// Day 0 of the demo calendar is 2026-10-01; "today" is 2026-11-12.
pub const TODAY: i32 = 42;

/// The ISO date (`2026-11-12`) of demo day `d` (2026-09-01 … 2027-03-31).
pub fn iso(d: i32) -> String {
    const MONTHS: [(i32, i32, u32); 7] = [
        (2026, 9, 30),
        (2026, 10, 31),
        (2026, 11, 30),
        (2026, 12, 31),
        (2027, 1, 31),
        (2027, 2, 28),
        (2027, 3, 31),
    ];
    let mut left = d + 30;
    for (y, m, n) in MONTHS {
        if left < n as i32 {
            return format!("{y}-{m:02}-{:02}", left + 1);
        }
        left -= n as i32;
    }
    "2027-04-01".to_string()
}

/// The month (`2026-11`) and day of month of demo day `d`.
pub fn month_day(d: i32) -> (String, u32) {
    let s = iso(d);
    (s[..7].to_string(), s[8..].parse().unwrap_or(1))
}

/// The weekday (0 Monday … 6 Sunday) of demo day `d` (2026-10-01 was a Thursday).
pub fn weekday(d: i32) -> u32 {
    (d + 3).rem_euclid(7) as u32
}

/// `1234567` → `1,234,567`.
pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

// ───────────────────────────────────────────────────────────── Metrics ──

/// Date ranges: (key, label, days).
pub const RANGES: [(&str, &str, u32); 4] = [
    ("7d", "Last 7 days", 7),
    ("30d", "Last 30 days", 30),
    ("90d", "Last 90 days", 90),
    ("12m", "Last 12 months", 365),
];
/// The KPI tabs over the chart: (key, label).
pub const KPIS: [(&str, &str); 5] = [
    ("visitors", "Visitors"),
    ("views", "Page views"),
    ("signups", "Signups"),
    ("bounce", "Bounce rate"),
    ("duration", "Visit duration"),
];
/// The last day the dashboard covers (2026-11-11).
pub const LAST_DAY: i32 = 41;

fn bump(d: f64, at: f64, h: f64) -> f64 {
    h * (-(d - at) * (d - at) / 3.0).exp()
}

/// Visitors on demo day `d`.
pub fn visitors_on(d: i32) -> f64 {
    let x = d as f64;
    (1500.0
        + 280.0 * (x * 0.45).sin()
        + 180.0 * (x * 0.13).cos()
        + 4.0 * x
        + bump(x, 17.0, 2300.0)
        + bump(x, 35.0, 1200.0)
        - if weekday(d) >= 5 { 260.0 } else { 0.0 })
    .max(120.0)
}

fn kpi_on(k: usize, d: i32) -> f64 {
    let v = visitors_on(d);
    let x = d as f64;
    match k {
        1 => v * 2.6 + 650.0 * (x * 0.21).sin() + 38.0 * x,
        2 => (v * 0.028 + 9.0 * (x * 0.37).cos() + 0.9 * x).max(4.0),
        3 => 41.5 + 3.0 * (x * 0.3).sin(),
        4 => 149.0 + 12.0 * (x * 0.2).sin(),
        _ => v,
    }
}

/// How the period before compares, per counted KPI (visitors, views, signups), so each tab has
/// its own change.
const PREV_RATE: [f64; 3] = [0.82, 0.88, 0.93];

fn averaged(k: usize) -> bool {
    k >= 3
}

/// The chart series for KPI `k` over range `r` (an index into [`RANGES`]): one point a day, or
/// one a week for the 12 months; `prev` is the period before (for Compare).
pub fn series(k: usize, r: usize, prev: bool) -> Vec<f64> {
    let days = RANGES[r.min(3)].2 as i32;
    let shift = if prev { days } else { 0 };
    if days > 100 {
        return (0..52)
            .map(|w| {
                let end = LAST_DAY - shift - 7 * (51 - w);
                let pts: Vec<f64> = (0..7).map(|i| kpi_on(k, end - i)).collect();
                let s: f64 = pts.iter().sum();
                let v = if averaged(k) { s / 7.0 } else { s };
                if prev {
                    v * (PREV_RATE[k.min(2)] + 0.04)
                } else {
                    v
                }
            })
            .collect();
    }
    (0..days)
        .map(|i| {
            let v = kpi_on(k, LAST_DAY - shift - (days - 1 - i));
            if prev && !averaged(k) {
                v * PREV_RATE[k.min(2)]
            } else {
                v
            }
        })
        .collect()
}

/// The day each point of [`series`] stands for (a week's last day for the 12 months).
pub fn point_day(r: usize, i: usize) -> i32 {
    let days = RANGES[r.min(3)].2 as i32;
    if days > 100 {
        LAST_DAY - 7 * (51 - i as i32)
    } else {
        LAST_DAY - (days - 1 - i as i32)
    }
}

/// The first and last ISO dates of range `r`.
pub fn range_span(r: usize) -> (String, String) {
    let days = RANGES[r.min(3)].2 as i32;
    (iso(LAST_DAY - days + 1), iso(LAST_DAY))
}

/// A KPI value as shown: (value, change, change is good). Change is percent, or points for the
/// bounce rate (where down is good).
pub fn kpi_value(k: usize, r: usize) -> (String, String, bool) {
    let cur = series(k, r, false);
    let prev = series(k, r, true);
    let total = |v: &[f64]| {
        let s: f64 = v.iter().sum();
        if averaged(k) { s / v.len() as f64 } else { s }
    };
    let (c, p) = (total(&cur), total(&prev));
    match k {
        3 => {
            let d = (c * 10.0).round() / 10.0 - (p * 10.0).round() / 10.0 - 1.0;
            (format!("{c:.1}%"), format!("{:+.1} pts", d), d <= 0.0)
        }
        4 => {
            let s = c.round() as u32;
            let ch = (c / (p - 0.6) - 1.0) * 100.0;
            (
                format!("{}m {:02}s", s / 60, s % 60),
                format!("{ch:+.1}%"),
                ch >= 0.0,
            )
        }
        _ => {
            let ch = (c / p - 1.0) * 100.0;
            (thousands(c.round() as u64), format!("{ch:+.1}%"), ch >= 0.0)
        }
    }
}

/// A compact axis value (`6K`, `850`, `42%`, `2:30`).
pub fn axis_value(k: usize, v: f64) -> String {
    match k {
        3 => format!("{v:.0}%"),
        4 => format!("{}:{:02}", (v as u32) / 60, (v as u32) % 60),
        _ if v >= 1000.0 => {
            let t = v / 1000.0;
            if (t - t.round()).abs() < 0.05 {
                format!("{t:.0}K")
            } else {
                format!("{t:.1}K")
            }
        }
        _ => format!("{v:.0}"),
    }
}

/// A "nice" chart top and step for a maximum: 4 grid steps.
pub fn nice_top(max: f64) -> (f64, f64) {
    let raw = (max * 1.08 / 4.0).max(1e-9);
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    (step * 4.0, step)
}

/// Normalises a dragged selection (`a`, `b` point indices) to a zoom window: at least 3 points
/// inside `0..len`; `None` for a click without a drag.
pub fn zoom_window(len: usize, a: usize, b: usize) -> Option<(usize, usize)> {
    if len < 3 {
        return None;
    }
    let (lo, hi) = (a.min(b).min(len - 1), a.max(b).min(len - 1));
    if hi - lo < 2 {
        return None;
    }
    Some((lo, hi))
}

/// The tables under the chart: (key, tab A, tab B).
pub const TABLES: [(&str, &str, &str); 3] = [
    ("pages", "Top pages", "Entry pages"),
    ("sources", "Referrers", "Campaigns"),
    ("places", "Countries", "Cities"),
];

/// One breakdown row: label, an icon (brand id, country code or `""`), visitors.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// What the row counts.
    pub label: &'static str,
    /// A brand mark id (`google`), a two-letter country code (`CH`) or `link` for direct.
    pub icon: &'static str,
    /// Visitors in the range.
    pub n: u64,
}

const TOP_PAGES: [(&str, u64); 7] = [
    ("/", 32093),
    ("/pricing", 14940),
    ("/features/reports", 8853),
    ("/signup", 6640),
    ("/blog/quiet-dashboards", 5201),
    ("/customers", 4482),
    ("/changelog", 3652),
];
const ENTRY_PAGES: [(&str, u64); 7] = [
    ("/", 21880),
    ("/blog/quiet-dashboards", 7410),
    ("/pricing", 6025),
    ("/docs/install", 4120),
    ("/features/reports", 3365),
    ("/compare", 2240),
    ("/careers", 1180),
];
const REFERRERS: [(&str, &str, u64); 7] = [
    ("Direct", "link", 19920),
    ("Google", "google", 14940),
    ("X", "x", 4703),
    ("GitHub", "github", 3431),
    ("Slack", "slack", 2656),
    ("Discord", "discord", 2269),
    ("Notion", "notion", 1826),
];
const CAMPAIGNS: [(&str, u64); 6] = [
    ("autumn-launch", 6120),
    ("newsletter-nov", 3988),
    ("webinar-reports", 2210),
    ("partner-kit", 1640),
    ("retarget-pricing", 990),
    ("docs-footer", 455),
];
const COUNTRIES: [(&str, &str, u64); 7] = [
    ("United States", "US", 18813),
    ("Germany", "DE", 6087),
    ("United Kingdom", "GB", 4980),
    ("Switzerland", "CH", 3320),
    ("France", "FR", 2767),
    ("Netherlands", "NL", 2213),
    ("Canada", "CA", 2170),
];
const CITIES: [(&str, &str, u64); 7] = [
    ("Berlin", "DE", 3210),
    ("London", "GB", 3104),
    ("New York", "US", 2980),
    ("Zürich", "CH", 2215),
    ("Paris", "FR", 1870),
    ("Toronto", "CA", 1422),
    ("Amsterdam", "NL", 1388),
];

/// The rows of table `t` (0 pages, 1 sources, 2 places), tab `b` (false A, true B), scaled to
/// range `r`.
pub fn table(t: usize, b: bool, r: usize) -> Vec<Row> {
    let f = {
        let s: f64 = series(0, r, false).iter().sum();
        s / 55_333.0
    };
    let sc = |n: u64| ((n as f64) * f).round().max(1.0) as u64;
    match (t, b) {
        (0, false) => TOP_PAGES
            .iter()
            .map(|(l, n)| Row {
                label: l,
                icon: "",
                n: sc(*n),
            })
            .collect(),
        (0, true) => ENTRY_PAGES
            .iter()
            .map(|(l, n)| Row {
                label: l,
                icon: "",
                n: sc(*n),
            })
            .collect(),
        (1, false) => REFERRERS
            .iter()
            .map(|(l, i, n)| Row {
                label: l,
                icon: i,
                n: sc(*n),
            })
            .collect(),
        (1, true) => CAMPAIGNS
            .iter()
            .map(|(l, n)| Row {
                label: l,
                icon: "tag",
                n: sc(*n),
            })
            .collect(),
        (_, false) => COUNTRIES
            .iter()
            .map(|(l, i, n)| Row {
                label: l,
                icon: i,
                n: sc(*n),
            })
            .collect(),
        _ => CITIES
            .iter()
            .map(|(l, i, n)| Row {
                label: l,
                icon: i,
                n: sc(*n),
            })
            .collect(),
    }
}

/// The metrics sidebar: (key, icon, label, count).
pub const MX_NAV: [(&str, &str, &str, &str); 6] = [
    ("overview", "grid", "Overview", ""),
    ("pages", "file", "Pages", ""),
    ("sources", "route", "Sources", ""),
    ("locations", "globe", "Locations", ""),
    ("devices", "laptop", "Devices", ""),
    ("goals", "target", "Goals", "6"),
];
/// Saved views: (icon or brand, label, the range it opens).
pub const MX_VIEWS: [(&str, &str, &str); 4] = [
    ("cal", "Launch week", "7d"),
    ("file", "Pricing page", "30d"),
    ("google", "Organic search", "90d"),
    ("globe", "DACH visitors", "30d"),
];
/// The "Online now" sparkline (the last 30 minutes) and the count.
pub const ONLINE: ([u32; 24], u32) = (
    [
        9, 11, 10, 12, 9, 8, 10, 11, 13, 12, 11, 10, 12, 14, 13, 12, 15, 14, 13, 16, 15, 14, 13, 17,
    ],
    142,
);

// ───────────────────────────────────────────────────────────── Kanban ──

/// A board column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Col {
    /// Not planned yet.
    Backlog,
    /// Planned for the cycle.
    Todo,
    /// Being built.
    Progress,
    /// Waiting on review.
    Review,
    /// Shipped.
    Done,
}

impl Col {
    /// Left to right.
    pub const ALL: [Col; 5] = [
        Col::Backlog,
        Col::Todo,
        Col::Progress,
        Col::Review,
        Col::Done,
    ];
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Col::Backlog => "backlog",
            Col::Todo => "todo",
            Col::Progress => "progress",
            Col::Review => "review",
            Col::Done => "done",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Col::Backlog => "Backlog",
            Col::Todo => "Todo",
            Col::Progress => "In progress",
            Col::Review => "In review",
            Col::Done => "Done",
        }
    }
    /// Work-in-progress limit (0 = none).
    pub fn limit(self) -> usize {
        match self {
            Col::Progress => 4,
            Col::Review => 3,
            _ => 0,
        }
    }
    /// Its semantic tone (the status glyph).
    pub fn tone(self) -> &'static str {
        match self {
            Col::Backlog | Col::Todo => "mute",
            Col::Progress => "amber",
            Col::Review => "blue",
            Col::Done => "green",
        }
    }
    /// From a key.
    pub fn from_key(k: &str) -> Option<Col> {
        Col::ALL.into_iter().find(|c| c.key() == k)
    }
    /// Its index.
    pub fn index(self) -> usize {
        Col::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }
}

/// Issue labels: (key, label, hue).
pub const LABELS: [(&str, &str, &str); 6] = [
    ("frontend", "Frontend", "blue"),
    ("backend", "Backend", "violet"),
    ("design", "Design", "magenta"),
    ("bug", "Bug", "rose"),
    ("growth", "Growth", "green"),
    ("perf", "Performance", "amber"),
];
/// Priorities 0..=4: (label, bars filled; 4 is Urgent).
pub const PRIOS: [(&str, u8); 5] = [
    ("No priority", 0),
    ("Low", 1),
    ("Medium", 2),
    ("High", 3),
    ("Urgent", 4),
];
/// Issue key prefix.
pub const KEY: &str = "STF";

/// One issue on the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// Number (shown as `STF-231`).
    pub id: u32,
    /// Title.
    pub title: String,
    /// Column.
    pub col: Col,
    /// Index into [`LABELS`].
    pub label: usize,
    /// Index into [`PRIOS`].
    pub prio: usize,
    /// Estimate in points.
    pub pts: u32,
    /// Assignee, an index into [`PEOPLE`].
    pub who: usize,
    /// Due day (demo calendar).
    pub due: Option<i32>,
    /// Comments.
    pub comments: u32,
    /// Linked pull request number.
    pub pr: Option<u32>,
    /// Sub-issues done of total.
    pub subs: Option<(u32, u32)>,
}

#[allow(clippy::too_many_arguments)]
fn iss(
    id: u32,
    title: &str,
    col: Col,
    label: usize,
    prio: usize,
    pts: u32,
    who: usize,
    due: Option<i32>,
    comments: u32,
    pr: Option<u32>,
    subs: Option<(u32, u32)>,
) -> Issue {
    Issue {
        id,
        title: title.to_string(),
        col,
        label,
        prio,
        pts,
        who,
        due,
        comments,
        pr,
        subs,
    }
}

/// The sample board (or nothing for an empty one).
pub fn issues(sample: bool) -> Vec<Issue> {
    use Col::*;
    if !sample {
        return Vec::new();
    }
    vec![
        iss(
            231,
            "Remember the last used card on this device",
            Backlog,
            1,
            1,
            3,
            4,
            None,
            0,
            None,
            None,
        ),
        iss(
            236,
            "Postcode lookup for Irish addresses",
            Backlog,
            0,
            2,
            5,
            8,
            None,
            2,
            None,
            None,
        ),
        iss(
            240,
            "One-tap reorder from the order history",
            Backlog,
            4,
            1,
            8,
            3,
            None,
            0,
            None,
            None,
        ),
        iss(
            242,
            "Translate payment errors for six markets",
            Backlog,
            0,
            0,
            3,
            5,
            None,
            0,
            None,
            None,
        ),
        iss(
            219,
            "Wallet sheet on the product page",
            Todo,
            0,
            2,
            5,
            1,
            Some(TODAY + 6),
            0,
            None,
            Some((0, 3)),
        ),
        iss(
            221,
            "Retry card checks when the bank times out",
            Todo,
            1,
            4,
            5,
            2,
            Some(TODAY + 2),
            5,
            None,
            None,
        ),
        iss(
            224,
            "Skeleton for the order summary while totals load",
            Todo,
            2,
            2,
            2,
            6,
            None,
            0,
            None,
            None,
        ),
        iss(
            226,
            "Sales tax line for Canadian provinces",
            Todo,
            1,
            2,
            3,
            4,
            Some(TODAY + 8),
            0,
            None,
            None,
        ),
        iss(
            229,
            "Promo field collapses on narrow screens",
            Todo,
            0,
            2,
            2,
            7,
            None,
            3,
            None,
            None,
        ),
        iss(
            208,
            "Split shipping and billing into two steps",
            Progress,
            0,
            3,
            8,
            3,
            Some(TODAY + 1),
            6,
            Some(1842),
            Some((3, 5)),
        ),
        iss(
            211,
            "Idempotency keys for payment intents",
            Progress,
            1,
            4,
            5,
            8,
            Some(TODAY),
            0,
            Some(1839),
            Some((1, 2)),
        ),
        iss(
            214,
            "New payment method picker",
            Progress,
            2,
            2,
            5,
            1,
            None,
            4,
            None,
            None,
        ),
        iss(
            216,
            "Animate the cart total when shipping changes",
            Progress,
            5,
            1,
            2,
            5,
            None,
            0,
            None,
            None,
        ),
        iss(
            201,
            "Express checkout with saved wallets",
            Review,
            0,
            3,
            8,
            2,
            None,
            11,
            Some(1831),
            Some((4, 4)),
        ),
        iss(
            204,
            "Fraud score webhook from the risk service",
            Review,
            1,
            3,
            5,
            4,
            None,
            2,
            Some(1835),
            None,
        ),
        iss(
            206,
            "Empty cart shows recently viewed items",
            Review,
            2,
            1,
            3,
            6,
            None,
            0,
            None,
            None,
        ),
        iss(
            198,
            "Guest checkout keeps the email on return",
            Done,
            0,
            2,
            3,
            7,
            None,
            1,
            Some(1820),
            None,
        ),
        iss(
            195,
            "Cache shipping rates per postcode",
            Done,
            5,
            2,
            3,
            8,
            None,
            0,
            Some(1814),
            None,
        ),
    ]
}

/// `STF-231`.
pub fn issue_key(id: u32) -> String {
    format!("{KEY}-{id}")
}

/// Whether an issue passes the text filter (title, key or label, case-insensitive) and the label
/// filter.
pub fn shows(i: &Issue, q: &str, label: Option<usize>) -> bool {
    let q = q.trim().to_lowercase();
    let text = q.is_empty()
        || i.title.to_lowercase().contains(&q)
        || issue_key(i.id).to_lowercase().contains(&q)
        || LABELS[i.label].1.to_lowercase().contains(&q);
    text && label.is_none_or(|l| i.label == l)
}

/// The issues of a column in board order, filtered.
pub fn column(v: &[Issue], c: Col, q: &str, label: Option<usize>) -> Vec<Issue> {
    v.iter()
        .filter(|i| i.col == c && shows(i, q, label))
        .cloned()
        .collect()
}

/// Issues per column (unfiltered), in [`Col::ALL`] order.
pub fn col_counts(v: &[Issue]) -> [usize; 5] {
    let mut n = [0; 5];
    for i in v {
        n[i.col.index()] += 1;
    }
    n
}

/// A column holds more than its work-in-progress limit.
pub fn over_limit(v: &[Issue], c: Col) -> bool {
    c.limit() > 0 && col_counts(v)[c.index()] > c.limit()
}

/// Moves issue `id` into column `to` at position `at` among that column's issues (`None` = the
/// end). False when nothing changed.
pub fn move_issue(v: &mut Vec<Issue>, id: u32, to: Col, at: Option<usize>) -> bool {
    let Some(from) = v.iter().position(|i| i.id == id) else {
        return false;
    };
    let mut it = v.remove(from);
    let same = it.col == to;
    it.col = to;
    let idx: Vec<usize> = v
        .iter()
        .enumerate()
        .filter(|(_, i)| i.col == to)
        .map(|(n, _)| n)
        .collect();
    let pos = match at {
        Some(a) if a < idx.len() => idx[a],
        _ => idx.last().map_or(v.len(), |l| l + 1),
    };
    v.insert(pos, it);
    !(same && pos == from)
}

/// Keyboard move of issue `id`: `dx` columns left / right, `dy` places up / down. Returns the
/// sentence to announce, or `None` when it cannot move.
pub fn nudge(v: &mut Vec<Issue>, id: u32, dx: i32, dy: i32) -> Option<String> {
    let it = v.iter().find(|i| i.id == id)?.clone();
    if dx != 0 {
        let to = it.col.index() as i32 + dx;
        if !(0..5).contains(&to) {
            return None;
        }
        let c = Col::ALL[to as usize];
        move_issue(v, id, c, None);
        return Some(format!("{} moved to {}", issue_key(id), c.label()));
    }
    let col = column(v, it.col, "", None);
    let pos = col.iter().position(|i| i.id == id)? as i32;
    let to = pos + dy;
    if dy == 0 || to < 0 || to >= col.len() as i32 {
        return None;
    }
    move_issue(v, id, it.col, Some(to as usize));
    Some(format!(
        "{} is {} of {} in {}",
        issue_key(id),
        to + 1,
        col.len(),
        it.col.label()
    ))
}

/// Adds an issue at the end of column `c`; returns its number, or `None` for a blank title.
pub fn add_issue(v: &mut Vec<Issue>, c: Col, title: &str) -> Option<u32> {
    let t = title.trim();
    if t.len() < 3 {
        return None;
    }
    let id = v.iter().map(|i| i.id).max().unwrap_or(243).max(243) + 1;
    v.push(iss(id, t, c, 0, 0, 1, 0, None, 0, None, None));
    Some(id)
}

/// The cycle: (name, start day, end day).
pub const CYCLE: (&str, i32, i32) = ("Cycle 12", TODAY - 3, TODAY + 10);

/// Points done and total over the board.
pub fn points(v: &[Issue]) -> (u32, u32) {
    let total = v.iter().map(|i| i.pts).sum();
    let done = v.iter().filter(|i| i.col == Col::Done).map(|i| i.pts).sum();
    (done, total)
}

/// Assignees on the board in first-seen order (for the By assignee grouping).
pub fn assignees(v: &[Issue]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for i in v {
        if !out.contains(&i.who) {
            out.push(i.who);
        }
    }
    out
}

/// The board's sidebar: (key, icon, label, count).
pub const KB_NAV: [(&str, &str, &str, &str); 3] = [
    ("mine", "user", "My issues", "4"),
    ("cycle", "cycle", "Current cycle", "18"),
    ("all", "layers", "All issues", ""),
];
/// The board's projects: (icon, label, count, hue).
pub const KB_PROJECTS: [(&str, &str, &str, &str); 4] = [
    ("bag", "Storefront v2", "18", "teal"),
    ("phone", "Mobile 5.1", "5", "violet"),
    ("plug", "Payments API", "4", "blue"),
    ("palette", "Design system", "3", "amber"),
];

// ───────────────────────────────────────────────────────────── Project board ──

/// A project's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    /// On track (green).
    OnTrack,
    /// At risk (amber).
    AtRisk,
    /// Off track (red).
    OffTrack,
}

impl Health {
    /// All three, for the menu.
    pub const ALL: [Health; 3] = [Health::OnTrack, Health::AtRisk, Health::OffTrack];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Health::OnTrack => "On track",
            Health::AtRisk => "At risk",
            Health::OffTrack => "Off track",
        }
    }
    /// Key.
    pub fn key(self) -> &'static str {
        match self {
            Health::OnTrack => "ontrack",
            Health::AtRisk => "atrisk",
            Health::OffTrack => "offtrack",
        }
    }
    /// Hue.
    pub fn hue(self) -> &'static str {
        match self {
            Health::OnTrack => "green",
            Health::AtRisk => "amber",
            Health::OffTrack => "rose",
        }
    }
}

/// The open project: (name, summary, lead, start day, launch day).
pub const PROJECT: (&str, &str, usize, i32, i32) = (
    "Returns portal",
    "Self-serve returns with printable labels, carrier pickup and instant refunds. The goal is a third fewer support tickets about returns by launch.",
    3,
    TODAY - 44,
    TODAY + 30,
);
/// Milestones: (name, due day, done).
pub const MILESTONES: [(&str, i32, bool); 5] = [
    ("Research and audit", TODAY - 34, true),
    ("Label flow", TODAY - 13, true),
    ("Carrier pickup", TODAY + 8, false),
    ("Beta at 10% traffic", TODAY + 21, false),
    ("General availability", TODAY + 30, false),
];
/// Sidebar projects: (icon, name, next milestone day, milestone, progress %).
pub const PB_PROJECTS: [(&str, &str, i32, &str, u32); 4] = [
    ("box", "Returns portal", TODAY + 8, "Carrier pickup", 40),
    ("phone", "Courier app 2.0", TODAY + 8, "Offline scans", 25),
    ("plug", "Partner API", TODAY + 8, "Rate limits", 60),
    ("palette", "Brand refresh", TODAY + 15, "Logo and type", 15),
];

/// One task: title, milestone, state, priority, due day, comments, assignee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    /// Title.
    pub title: String,
    /// Index into [`MILESTONES`].
    pub ms: usize,
    /// 0 todo, 1 in progress, 2 in review, 3 done.
    pub st: u8,
    /// Index into [`PRIOS`].
    pub prio: usize,
    /// Due day.
    pub due: i32,
    /// Comments.
    pub comments: u32,
    /// Assignee (index into [`PEOPLE`]).
    pub who: usize,
}

fn tk(title: &str, ms: usize, st: u8, prio: usize, due: i32, comments: u32, who: usize) -> Task {
    Task {
        title: title.to_string(),
        ms,
        st,
        prio,
        due,
        comments,
        who,
    }
}

/// The sample tasks (or none).
pub fn tasks(sample: bool) -> Vec<Task> {
    if !sample {
        return Vec::new();
    }
    vec![
        tk(
            "Interview six support agents about returns",
            0,
            3,
            2,
            TODAY - 40,
            3,
            3,
        ),
        tk(
            "Audit the current returns emails",
            0,
            3,
            1,
            TODAY - 36,
            0,
            11,
        ),
        tk("Printable label with a QR code", 1, 3, 3, TODAY - 20, 4, 2),
        tk(
            "Label fallback when the printer is offline",
            1,
            3,
            2,
            TODAY - 15,
            1,
            8,
        ),
        tk("Design states for a missed pickup", 2, 1, 2, TODAY, 2, 3),
        tk(
            "Pickup slot picker with keyboard support",
            2,
            1,
            3,
            TODAY + 2,
            1,
            1,
        ),
        tk(
            "Carrier availability behind a feature flag",
            2,
            2,
            2,
            TODAY + 1,
            1,
            8,
        ),
        tk(
            "Copy for pickup errors and hints",
            2,
            2,
            1,
            TODAY - 2,
            0,
            11,
        ),
        tk(
            "Validate pickup addresses for three countries",
            2,
            0,
            2,
            TODAY + 6,
            0,
            2,
        ),
        tk("Track the pickup booking rate", 2, 0, 1, TODAY + 7, 1, 12),
        tk("Pick a carrier pickup provider", 2, 3, 2, TODAY - 9, 0, 3),
        tk("Remove the old returns form", 2, 3, 1, TODAY - 6, 0, 1),
        tk(
            "Refund to the original payment method",
            3,
            0,
            3,
            TODAY + 18,
            2,
            8,
        ),
        tk("Beta cohort and kill switch", 3, 0, 2, TODAY + 19, 0, 12),
        tk("Help-centre article and macros", 4, 0, 1, TODAY + 27, 0, 11),
        tk("Launch email to all customers", 4, 0, 1, TODAY + 29, 0, 3),
    ]
}

/// The project's numbers: (complete %, open, in progress or review, due this week, overdue,
/// days until launch).
pub fn stats(t: &[Task]) -> (u32, usize, usize, usize, usize, i32) {
    let done = t.iter().filter(|x| x.st == 3).count();
    let pct = (done * 100).checked_div(t.len()).unwrap_or(0) as u32;
    let open: Vec<&Task> = t.iter().filter(|x| x.st < 3).collect();
    let busy = open.iter().filter(|x| x.st > 0).count();
    let week = open
        .iter()
        .filter(|x| x.due >= TODAY && x.due < TODAY + 7)
        .count();
    let late = open.iter().filter(|x| x.due < TODAY).count();
    (pct, open.len(), busy, week, late, PROJECT.4 - TODAY)
}

/// A milestone's progress: (done, total).
pub fn ms_progress(t: &[Task], ms: usize) -> (usize, usize) {
    let all = t.iter().filter(|x| x.ms == ms).count();
    let done = t.iter().filter(|x| x.ms == ms && x.st == 3).count();
    (done, all)
}

/// Milestones done of five.
pub fn ms_done(t: &[Task]) -> usize {
    (0..MILESTONES.len())
        .filter(|m| {
            let (d, a) = ms_progress(t, *m);
            a > 0 && d == a
        })
        .count()
}

/// Completes an open task or reopens a done one; returns the new state's word.
pub fn toggle_task(t: &mut [Task], i: usize) -> Option<&'static str> {
    let x = t.get_mut(i)?;
    if x.st == 3 {
        x.st = 0;
        Some("reopened")
    } else {
        x.st = 3;
        Some("completed")
    }
}

/// The team card: (person, role) with their open tasks counted from `t`.
pub const TEAM: [(usize, &str); 6] = [
    (3, "Design lead"),
    (1, "Product designer"),
    (2, "Frontend engineer"),
    (8, "Backend engineer"),
    (12, "Data analyst"),
    (11, "Customer success"),
];

/// Open tasks of person `who`.
pub fn load_of(t: &[Task], who: usize) -> usize {
    t.iter().filter(|x| x.who == who && x.st < 3).count()
}

/// Links card: (brand, title, line).
pub const LINKS: [(&str, &str, &str); 3] = [
    ("figma", "Returns flows v4", "Edited 2026-11-12 09:40"),
    ("github", "marlow/returns-web", "3 open pull requests"),
    ("stripe", "Refunds API", "Test mode, 1,248 refunds"),
];

/// Activity on the project: (person, what, day, time).
pub const ACTIVITY: [(usize, &str, i32, &str); 6] = [
    (2, "completed Remove the old returns form", TODAY, "11:20"),
    (
        8,
        "moved Carrier availability behind a feature flag to In review",
        TODAY,
        "10:05",
    ),
    (3, "set the status to On track", TODAY - 1, "17:42"),
    (
        11,
        "commented on Copy for pickup errors and hints",
        TODAY - 1,
        "15:10",
    ),
    (
        1,
        "added Pickup slot picker with keyboard support",
        TODAY - 2,
        "09:31",
    ),
    (3, "completed the Label flow milestone", TODAY - 13, "16:00"),
];

/// Project board sidebar: (key, icon, label, count).
pub const PB_NAV: [(&str, &str, &str, &str); 3] = [
    ("home", "home", "Home", ""),
    ("mytasks", "tasks", "My tasks", "5"),
    ("inbox", "inbox", "Inbox", "3"),
];

// ───────────────────────────────────────────────────────────── Gantt ──

/// Gantt phases: (key, label).
pub const PHASES: [(&str, &str); 4] = [
    ("discovery", "Discovery"),
    ("design", "Design"),
    ("build", "Build"),
    ("launch", "Launch"),
];

/// Timeline zoom levels: (key, label, px per day).
pub const ZOOMS: [(&str, &str, f32); 3] = [
    ("day", "Day", 36.0),
    ("week", "Week", 15.0),
    ("month", "Month", 6.0),
];

/// One row of the timeline: a bar (`len` ≥ 1) or a milestone (`len` 0).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bar {
    /// Stable id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// Index into [`PHASES`].
    pub phase: usize,
    /// First day.
    pub start: i32,
    /// Days long; 0 for a milestone.
    pub len: i32,
    /// Percent done.
    pub done: u32,
    /// Owner (index into [`PEOPLE`]).
    pub who: usize,
    /// Bars that must finish first.
    pub after: Vec<u32>,
}

#[allow(clippy::too_many_arguments)]
fn bar(
    id: u32,
    name: &'static str,
    phase: usize,
    start: i32,
    len: i32,
    done: u32,
    who: usize,
    after: &[u32],
) -> Bar {
    Bar {
        id,
        name,
        phase,
        start,
        len,
        done,
        who,
        after: after.to_vec(),
    }
}

/// The plan's name and the first day the timeline shows.
pub const PLAN: (&str, i32) = ("Harbor 2.0", 25);

/// The sample plan (or none).
pub fn plan(sample: bool) -> Vec<Bar> {
    if !sample {
        return Vec::new();
    }
    vec![
        bar(1, "Customer interviews", 0, 8, 10, 100, 3, &[]),
        bar(2, "Research readout", 0, 18, 0, 100, 3, &[1]),
        bar(3, "Competitor teardown", 0, 10, 9, 100, 12, &[]),
        bar(4, "Navigation model", 1, 22, 6, 100, 1, &[2]),
        bar(5, "Visual refresh", 1, 28, 15, 85, 7, &[4]),
        bar(6, "Prototype testing", 1, 33, 10, 60, 1, &[4]),
        bar(7, "Design handoff", 1, 46, 0, 0, 3, &[5, 6]),
        bar(8, "API contracts", 2, 25, 12, 100, 8, &[]),
        bar(9, "Offline sync", 2, 39, 14, 20, 10, &[8]),
        bar(10, "Payments with saved cards", 2, 39, 14, 35, 2, &[8]),
        bar(11, "Onboarding flow", 2, 47, 9, 5, 5, &[7]),
        bar(12, "iOS app", 2, 39, 22, 25, 14, &[8]),
        bar(13, "Android app", 2, 46, 18, 0, 15, &[7]),
        bar(14, "Beta freeze", 3, 69, 0, 0, 10, &[12, 13]),
        bar(15, "Press kit", 3, 46, 10, 0, 6, &[]),
        bar(16, "App store review", 3, 69, 7, 0, 4, &[14]),
        bar(17, "Launch campaign", 3, 57, 19, 0, 6, &[15]),
        bar(18, "Public launch", 3, 78, 0, 0, 3, &[16, 17]),
    ]
}

/// The last day a bar covers (a milestone is its own day).
pub fn end_of(b: &Bar) -> i32 {
    b.start + (b.len - 1).max(0)
}

/// Dependencies whose successor starts before the predecessor ends: (from, to) ids.
pub fn conflicts(v: &[Bar]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for b in v {
        for a in &b.after {
            if let Some(p) = v.iter().find(|x| x.id == *a)
                && b.start <= end_of(p)
                && !(p.len == 0 && b.start >= p.start)
            {
                out.push((p.id, b.id));
            }
        }
    }
    out
}

/// A phase's span and progress: (first day, last day, percent done weighted by length).
pub fn phase_span(v: &[Bar], phase: usize) -> Option<(i32, i32, u32)> {
    let bs: Vec<&Bar> = v.iter().filter(|b| b.phase == phase).collect();
    let a = bs.iter().map(|b| b.start).min()?;
    let z = bs.iter().map(|b| end_of(b)).max()?;
    let (w, d) = bs
        .iter()
        .filter(|b| b.len > 0)
        .fold((0i64, 0i64), |(w, d), b| {
            (w + b.len as i64, d + b.len as i64 * b.done as i64)
        });
    Some((a, z, d.checked_div(w).unwrap_or(0) as u32))
}

/// The plan with its undo history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The bars as they are now.
    pub bars: Vec<Bar>,
    /// Earlier versions, newest last.
    pub undo: Vec<Vec<Bar>>,
}

impl Plan {
    /// The sample plan, or an empty one.
    pub fn new(sample: bool) -> Self {
        Self {
            bars: plan(sample),
            undo: Vec::new(),
        }
    }

    /// Moves bar `id` by `dd` days (both ends).
    pub fn shift(&mut self, id: u32, dd: i32) -> bool {
        self.edit(id, |b| b.start += dd, dd != 0)
    }

    /// Moves bar `id` to start on day `day`.
    pub fn place(&mut self, id: u32, day: i32) -> bool {
        let Some(b) = self.bars.iter().find(|b| b.id == id) else {
            return false;
        };
        let dd = day - b.start;
        self.shift(id, dd)
    }

    /// Changes the length of bar `id` by `dd` days from its end (at least one day; milestones
    /// do not stretch).
    pub fn stretch(&mut self, id: u32, dd: i32) -> bool {
        let ok = self
            .bars
            .iter()
            .find(|b| b.id == id)
            .is_some_and(|b| b.len > 0 && b.len + dd >= 1 && dd != 0);
        self.edit(id, |b| b.len += dd, ok)
    }

    /// Changes the start of bar `id` by `dd` days and keeps its end (the left edge).
    pub fn stretch_start(&mut self, id: u32, dd: i32) -> bool {
        let ok = self
            .bars
            .iter()
            .find(|b| b.id == id)
            .is_some_and(|b| b.len > 0 && b.len - dd >= 1 && dd != 0);
        self.edit(
            id,
            |b| {
                b.start += dd;
                b.len -= dd;
            },
            ok,
        )
    }

    fn edit(&mut self, id: u32, f: impl FnOnce(&mut Bar), ok: bool) -> bool {
        if !ok || !self.bars.iter().any(|b| b.id == id) {
            return false;
        }
        self.undo.push(self.bars.clone());
        if self.undo.len() > 40 {
            self.undo.remove(0);
        }
        if let Some(b) = self.bars.iter_mut().find(|b| b.id == id) {
            f(b);
        }
        true
    }

    /// Undoes the last change.
    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(b) => {
                self.bars = b;
                true
            }
            None => false,
        }
    }

    /// Back to the sample plan.
    pub fn reset(&mut self) -> bool {
        let fresh = plan(true);
        if self.bars == fresh {
            return false;
        }
        self.undo.push(std::mem::replace(&mut self.bars, fresh));
        true
    }
}

/// "Harbor 2.0 · 18 tasks · 41% done": done weighted by length.
pub fn plan_done(v: &[Bar]) -> u32 {
    let (w, d) = v
        .iter()
        .filter(|b| b.len > 0)
        .fold((0i64, 0i64), |(w, d), b| {
            (w + b.len as i64, d + b.len as i64 * b.done as i64)
        });
    d.checked_div(w).unwrap_or(0) as u32
}

/// The bar's dates for a tooltip: `2026-11-03 → 2026-11-16 · 14 days`.
pub fn bar_dates(b: &Bar) -> String {
    if b.len == 0 {
        iso(b.start)
    } else {
        format!(
            "{} → {} · {} day{}",
            iso(b.start),
            iso(end_of(b)),
            b.len,
            if b.len == 1 { "" } else { "s" }
        )
    }
}

/// Where a dragged bar's date tooltip goes, so it never covers the bars around it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipAt {
    /// Over the bar, in the row above.
    Above,
    /// Under the bar, in the row below.
    Below,
    /// After the bar (and its label), in its own row.
    Right,
    /// Before the bar, in its own row.
    Left,
}

impl TipAt {
    /// `above`, `below`, `right` or `left`.
    pub fn key(self) -> &'static str {
        match self {
            TipAt::Above => "above",
            TipAt::Below => "below",
            TipAt::Right => "right",
            TipAt::Left => "left",
        }
    }
}

/// Free space for a tooltip `w` wide that starts at the bar's start `x`: the row above when
/// nothing there is under it, else the row below, else after the bar (`end`: where the bar and
/// its outside label stop) when that fits in `width`, else before the bar. `above` and `below`
/// are the spans (start, end) taken in those rows; `None` means there is no row there (the
/// header sits above the first row).
pub fn tip_at(
    x: f64,
    w: f64,
    end: f64,
    width: f64,
    above: Option<&[(f64, f64)]>,
    below: Option<&[(f64, f64)]>,
) -> TipAt {
    let free = |r: Option<&[(f64, f64)]>| {
        r.is_some_and(|v| v.iter().all(|(a, b)| *b < x - 4.0 || *a > x + w + 4.0))
    };
    if free(above) {
        TipAt::Above
    } else if free(below) {
        TipAt::Below
    } else if end + 16.0 + w <= width {
        TipAt::Right
    } else {
        TipAt::Left
    }
}

/// The width of a tooltip with text `t` (11.5 px type, 8 px padding each side).
pub fn tip_w(t: &str) -> f64 {
    t.chars().count() as f64 * 6.6 + 18.0
}

/// The owners of a plan's bars, sorted and once each.
pub fn owners(v: &[Bar]) -> Vec<usize> {
    let mut o: Vec<usize> = v.iter().map(|b| b.who).collect();
    o.sort();
    o.dedup();
    o
}

/// Gantt sidebar: (key, icon, label, count).
pub const GT_NAV: [(&str, &str, &str, &str); 3] = [
    ("timeline", "chart", "Timeline", ""),
    ("list", "list", "List", ""),
    ("updates", "clock", "Updates", "3"),
];
/// Saved views: (icon, label, count).
pub const GT_VIEWS: [(&str, &str, &str); 3] = [
    ("user", "My tasks", "2"),
    ("flash", "At risk", "2"),
    ("cal", "Next two weeks", "9"),
];

// ───────────────────────────────────────────────────────────── Team directory ──

/// Teams: (key, label, icon, hue).
pub const TEAMS: [(&str, &str, &str, &str); 4] = [
    ("eng", "Engineering", "term", "blue"),
    ("design", "Design", "palette", "magenta"),
    ("gtm", "Go-to-market", "mega", "amber"),
    ("ops", "Operations", "building", "teal"),
];
/// Offices: (key, city, UTC offset in minutes on 2026-11-12).
pub const OFFICES: [(&str, &str, i32); 5] = [
    ("zrh", "Zürich", 60),
    ("lon", "London", 0),
    ("nyc", "New York", -300),
    ("lis", "Lisbon", 0),
    ("sgp", "Singapore", 480),
];
/// The directory's clock: 2026-11-12 13:32 UTC, in minutes after midnight.
pub const UTC_NOW: i32 = 13 * 60 + 32;

/// Local time at office `o` (`14:32`).
pub fn local_time(o: usize) -> String {
    let m = (UTC_NOW + OFFICES[o.min(OFFICES.len() - 1)].2).rem_euclid(24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// Whether office `o` is inside working hours (09:00–18:00 local).
pub fn office_open(o: usize) -> bool {
    let m = (UTC_NOW + OFFICES[o.min(OFFICES.len() - 1)].2).rem_euclid(24 * 60);
    (9 * 60..18 * 60).contains(&m)
}

/// Someone's availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Avail {
    /// Available.
    Free,
    /// In a meeting until a local time.
    Busy(&'static str),
    /// Out of office until a day.
    Out(i32),
    /// Starts on a day.
    Starts(i32),
    /// Outside their hours; online at a local time.
    Later(&'static str),
}

impl Avail {
    /// Filter key.
    pub fn key(self) -> &'static str {
        match self {
            Avail::Free => "free",
            Avail::Busy(_) => "busy",
            Avail::Out(_) | Avail::Starts(_) => "out",
            Avail::Later(_) => "away",
        }
    }
    /// The line on a card.
    pub fn words(self) -> String {
        match self {
            Avail::Free => "Available".into(),
            Avail::Busy(t) => format!("Busy until {t}"),
            Avail::Out(d) => format!("Out until {}", iso(d)),
            Avail::Starts(d) => format!("Starts {}", iso(d)),
            Avail::Later(t) => format!("Online at {t}"),
        }
    }
    /// Its hue.
    pub fn hue(self) -> &'static str {
        match self {
            Avail::Free => "green",
            Avail::Busy(_) => "amber",
            Avail::Out(_) => "blue",
            Avail::Starts(_) => "violet",
            Avail::Later(_) => "mute",
        }
    }
    /// Its glyph.
    pub fn icon(self) -> &'static str {
        match self {
            Avail::Free => "check",
            Avail::Busy(_) => "cal",
            Avail::Out(_) => "globe",
            Avail::Starts(_) => "clock",
            Avail::Later(_) => "pause",
        }
    }
}

/// Status filter: (key, label).
pub const STATUSES: [(&str, &str); 4] = [
    ("free", "Available"),
    ("busy", "Busy"),
    ("away", "Away"),
    ("out", "Out of office"),
];

/// One person in the directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// Index into [`PEOPLE`].
    pub who: usize,
    /// Role.
    pub role: &'static str,
    /// Index into [`TEAMS`].
    pub team: usize,
    /// Index into [`OFFICES`].
    pub office: usize,
    /// Availability now.
    pub avail: Avail,
    /// Team lead (a star after the name).
    pub lead: bool,
    /// Skills (searched).
    pub skills: &'static str,
}

fn mb(
    who: usize,
    role: &'static str,
    team: usize,
    office: usize,
    avail: Avail,
    lead: bool,
    skills: &'static str,
) -> Member {
    Member {
        who,
        role,
        team,
        office,
        avail,
        lead,
        skills,
    }
}

/// The sample directory (or nobody).
pub fn members(sample: bool) -> Vec<Member> {
    use Avail::*;
    if !sample {
        return Vec::new();
    }
    vec![
        mb(
            10,
            "Engineering manager",
            0,
            0,
            Busy("15:00"),
            true,
            "rust planning hiring",
        ),
        mb(
            8,
            "Backend engineer",
            0,
            0,
            Free,
            false,
            "rust postgres payments",
        ),
        mb(
            2,
            "Frontend engineer",
            0,
            1,
            Free,
            false,
            "typescript accessibility css",
        ),
        mb(
            14,
            "Mobile engineer",
            0,
            3,
            Free,
            false,
            "swift kotlin offline",
        ),
        mb(
            5,
            "Platform engineer",
            0,
            1,
            Out(TODAY + 4),
            false,
            "kubernetes terraform",
        ),
        mb(3, "Design lead", 1, 1, Free, true, "figma research systems"),
        mb(
            1,
            "Product designer",
            1,
            0,
            Free,
            false,
            "figma prototyping motion",
        ),
        mb(
            15,
            "Brand designer",
            1,
            4,
            Later("09:00"),
            false,
            "illustration type",
        ),
        mb(
            6,
            "Head of sales",
            2,
            2,
            Free,
            true,
            "enterprise negotiation",
        ),
        mb(
            7,
            "Marketing manager",
            2,
            1,
            Busy("14:00"),
            false,
            "content seo events",
        ),
        mb(
            11,
            "Customer success",
            2,
            1,
            Free,
            false,
            "onboarding support",
        ),
        mb(
            4,
            "Account executive",
            2,
            2,
            Later("09:00"),
            false,
            "smb pipeline",
        ),
        mb(
            13,
            "Chief operating officer",
            3,
            0,
            Free,
            true,
            "finance hiring legal",
        ),
        mb(
            9,
            "Finance partner",
            3,
            0,
            Free,
            false,
            "forecasting payroll",
        ),
        mb(
            12,
            "Data analyst",
            3,
            0,
            Free,
            false,
            "sql dashboards python",
        ),
        mb(
            0,
            "Operations lead",
            3,
            3,
            Starts(TODAY + 5),
            false,
            "vendors procurement",
        ),
    ]
}

/// Indices of the members that match the search (name, role, team or skills; every word), the
/// team, status and office filters.
pub fn find_people(
    v: &[Member],
    q: &str,
    team: Option<usize>,
    status: Option<&str>,
    office: Option<usize>,
) -> Vec<usize> {
    let words: Vec<String> = q.split_whitespace().map(str::to_lowercase).collect();
    v.iter()
        .enumerate()
        .filter(|(_, m)| {
            let hay = format!(
                "{} {} {} {}",
                PEOPLE[m.who].0, m.role, TEAMS[m.team].1, m.skills
            )
            .to_lowercase();
            words.iter().all(|w| hay.contains(w.as_str()))
                && team.is_none_or(|t| m.team == t)
                && status.is_none_or(|s| m.avail.key() == s)
                && office.is_none_or(|o| m.office == o)
        })
        .map(|(i, _)| i)
        .collect()
}

/// An office's line: (working now, people there).
pub fn office_count(v: &[Member], o: usize) -> (usize, usize) {
    let here: Vec<&Member> = v.iter().filter(|m| m.office == o).collect();
    let open = office_open(o);
    let working = here
        .iter()
        .filter(|m| open && matches!(m.avail, Avail::Free | Avail::Busy(_)))
        .count();
    (working, here.len())
}

/// Team directory sidebar: (key, icon, label, count).
pub const TD_NAV: [(&str, &str, &str, &str); 3] = [
    ("people", "people", "People", "16"),
    ("org", "tree", "Org chart", ""),
    ("timeoff", "cal", "Time off", "2"),
];

// ───────────────────────────────────────────────────────────── Live state ──

/// The live state of the five screens a GPUI view keeps (a host routes press ids to
/// [`ArcUi::press`]): the metrics range, KPI, Compare and zoom; the board's issues, the picked
/// issue and the label filter; the project's tasks, milestone, tab and status; the plan, the
/// picked bar and the zoom; the directory's search, filters, view and favourites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArcUi {
    /// Index into [`RANGES`].
    pub range: usize,
    /// Index into [`KPIS`].
    pub kpi: usize,
    /// The saved view that set the range (index into [`MX_VIEWS`]).
    pub view: Option<usize>,
    /// Compare with the period before.
    pub compare: bool,
    /// Zoom window (point indices).
    pub zoom: Option<(usize, usize)>,
    /// Export done.
    pub exported: bool,
    /// The board.
    pub board: Vec<Issue>,
    /// The picked issue (keyboard moves act on it).
    pub pick: Option<u32>,
    /// Label filter.
    pub label: Option<usize>,
    /// Group by assignee.
    pub by_person: bool,
    /// The project's tasks.
    pub tasks: Vec<Task>,
    /// The milestone that picks the task list.
    pub ms: usize,
    /// Project tab: 0 Overview, 1 Tasks, 2 Activity.
    pub tab: usize,
    /// The project's status.
    pub health: Health,
    /// Right card shown.
    pub panel: bool,
    /// The plan.
    pub plan: Plan,
    /// The picked bar.
    pub bar: Option<u32>,
    /// Index into [`ZOOMS`].
    pub gzoom: usize,
    /// The timeline's owner filter (index into [`PEOPLE`]).
    pub gowner: Option<usize>,
    /// Directory search.
    pub query: String,
    /// Team filter.
    pub team: Option<usize>,
    /// Status filter.
    pub status: Option<&'static str>,
    /// Office filter.
    pub office: Option<usize>,
    /// List view (else grid).
    pub list: bool,
    /// Favourite people (indices into [`PEOPLE`]).
    pub favs: Vec<usize>,
    /// The last thing done, said back.
    pub said: Option<String>,
}

impl Default for ArcUi {
    fn default() -> Self {
        Self {
            range: 1,
            kpi: 0,
            view: None,
            compare: false,
            zoom: None,
            exported: false,
            board: issues(true),
            pick: None,
            label: None,
            by_person: false,
            tasks: tasks(true),
            ms: 2,
            tab: 0,
            health: Health::OnTrack,
            panel: true,
            plan: Plan::new(true),
            bar: None,
            gzoom: 1,
            gowner: None,
            query: String::new(),
            team: None,
            status: None,
            office: None,
            list: false,
            favs: vec![10, 3],
            said: None,
        }
    }
}

fn nth(s: &str, prefix: &str) -> Option<usize> {
    s.strip_prefix(prefix)?.parse().ok()
}

impl ArcUi {
    /// Handles a press id of the S12 screens; false when the id is not one of them (the host
    /// routes it on). Ids:
    /// - metrics: `mx-range-<7d|30d|90d|12m>`, `mx-view-<n>`, `mx-kpi-<n>`, `mx-compare`,
    ///   `mx-zoom`, `mx-unzoom`, `mx-export`;
    /// - board: `kb-pick-<n>` (the n-th issue of the sample board; toggles), `kb-grab-<n>` (picks,
    ///   for a drag), `kb-left|right|up|down`, `kb-drop-<column>`, `kb-label-<n>`, `kb-clear`,
    ///   `kb-group`, `kb-by-status`, `kb-by-person`;
    /// - project: `pb-task-<n>`, `pb-ms-<n>`, `pb-tab-<n>`, `pb-health-<n>`, `pb-panel`;
    /// - gantt: `gt-pick-<id>`, `gt-grab-<id>`, `gt-earlier|later|shorter|longer`, `gt-at-<day>`,
    ///   `gt-zoom-<n>`, `gt-undo`, `gt-reset`, `gt-today`, `gt-owner` (the next owner, then
    ///   everyone);
    /// - directory: `td-team-<n>`, `td-status-<n>`, `td-office-<n>`, `td-view`, `td-grid`,
    ///   `td-list`, `td-fav-<n>`, `td-clear`.
    pub fn press(&mut self, id: &str) -> bool {
        let said = |s: &mut Self, t: String| {
            s.said = Some(t);
            true
        };
        if let Some(k) = id.strip_prefix("mx-range-") {
            if let Some(r) = RANGES.iter().position(|x| x.0 == k) {
                self.range = r;
                self.view = None;
                self.zoom = None;
                return said(self, RANGES[r].1.to_string());
            }
            return false;
        }
        if let Some(n) = nth(id, "mx-view-") {
            let n = n.min(MX_VIEWS.len() - 1);
            self.range = RANGES
                .iter()
                .position(|x| x.0 == MX_VIEWS[n].2)
                .unwrap_or(1);
            self.view = Some(n);
            self.zoom = None;
            return said(self, format!("{}: {}", MX_VIEWS[n].1, RANGES[self.range].1));
        }
        if let Some(n) = nth(id, "mx-kpi-") {
            self.kpi = n.min(KPIS.len() - 1);
            return said(self, KPIS[self.kpi].1.to_string());
        }
        if let Some(n) = nth(id, "kb-pick-") {
            let Some(i) = issues(true).get(n).map(|x| x.id) else {
                return false;
            };
            self.pick = if self.pick == Some(i) { None } else { Some(i) };
            let t = match self.pick {
                Some(i) => format!("{} picked; Alt + arrows move it", issue_key(i)),
                None => "dropped".into(),
            };
            return said(self, t);
        }
        if let Some(n) = nth(id, "kb-grab-") {
            let Some(i) = issues(true).get(n).map(|x| x.id) else {
                return false;
            };
            self.pick = Some(i);
            return true;
        }
        if let Some(k) = id.strip_prefix("kb-drop-") {
            let (Some(c), Some(i)) = (Col::from_key(k), self.pick) else {
                return false;
            };
            move_issue(&mut self.board, i, c, None);
            self.pick = None;
            return said(self, format!("{} moved to {}", issue_key(i), c.label()));
        }
        if let Some(n) = nth(id, "kb-label-") {
            self.label = if self.label == Some(n) {
                None
            } else {
                Some(n.min(5))
            };
            return said(self, format!("label {}", LABELS[n.min(5)].1));
        }
        if let Some(n) = nth(id, "pb-task-") {
            return match toggle_task(&mut self.tasks, n) {
                Some(w) => {
                    let t = format!("{} {w}", self.tasks[n].title);
                    said(self, t)
                }
                None => false,
            };
        }
        if let Some(n) = nth(id, "pb-ms-") {
            self.ms = n.min(MILESTONES.len() - 1);
            self.tab = 0;
            return said(self, MILESTONES[self.ms].0.to_string());
        }
        if let Some(n) = nth(id, "pb-tab-") {
            self.tab = n.min(2);
            return true;
        }
        if let Some(n) = nth(id, "pb-health-") {
            self.health = Health::ALL[n.min(2)];
            return said(self, format!("Status: {}", self.health.label()));
        }
        if let Some(n) = nth(id, "gt-pick-") {
            let n = n as u32;
            self.bar = if self.bar == Some(n) { None } else { Some(n) };
            return said(
                self,
                match self
                    .bar
                    .and_then(|b| self.plan.bars.iter().find(|x| x.id == b))
                {
                    Some(b) => format!("{} · {}", b.name, bar_dates(b)),
                    None => "no bar picked".into(),
                },
            );
        }
        if let Some(n) = nth(id, "gt-grab-") {
            self.bar = Some(n as u32);
            return true;
        }
        if let Some(d) = id
            .strip_prefix("gt-at-")
            .and_then(|x| x.parse::<i32>().ok())
        {
            let Some(b) = self.bar else { return false };
            return self.plan.place(b, d) && self.say_bar(b);
        }
        if let Some(n) = nth(id, "gt-zoom-") {
            self.gzoom = n.min(2);
            return said(self, format!("{} view", ZOOMS[self.gzoom].1));
        }
        if let Some(n) = nth(id, "td-team-") {
            self.team = if self.team == Some(n) {
                None
            } else {
                Some(n.min(3))
            };
            return true;
        }
        if let Some(n) = nth(id, "td-status-") {
            let k = STATUSES[n.min(3)].0;
            self.status = if self.status == Some(k) {
                None
            } else {
                Some(k)
            };
            return true;
        }
        if let Some(n) = nth(id, "td-office-") {
            self.office = if self.office == Some(n) {
                None
            } else {
                Some(n.min(4))
            };
            return true;
        }
        if let Some(n) = nth(id, "td-fav-") {
            let w = n.min(15);
            if let Some(p) = self.favs.iter().position(|x| *x == w) {
                self.favs.remove(p);
                return said(self, format!("{} removed from favourites", PEOPLE[w].0));
            }
            self.favs.push(w);
            return said(self, format!("{} added to favourites", PEOPLE[w].0));
        }
        match id {
            "mx-compare" => {
                self.compare = !self.compare;
                true
            }
            "mx-zoom" => {
                let n = series(self.kpi, self.range, false).len();
                self.zoom = zoom_window(n, n / 4, n * 3 / 4);
                true
            }
            "mx-unzoom" => {
                self.zoom = None;
                true
            }
            "mx-export" => {
                self.exported = true;
                said(self, "Exported metrics.csv".into())
            }
            "kb-left" | "kb-right" | "kb-up" | "kb-down" => self.key_move(id),
            "kb-clear" => {
                self.label = None;
                true
            }
            "kb-group" => {
                self.by_person = !self.by_person;
                true
            }
            "kb-by-status" | "kb-by-person" => {
                self.by_person = id == "kb-by-person";
                true
            }
            "td-grid" | "td-list" => {
                self.list = id == "td-list";
                true
            }
            "pb-panel" => {
                self.panel = !self.panel;
                true
            }
            "gt-earlier" | "gt-later" | "gt-shorter" | "gt-longer" => {
                let Some(b) = self.bar else { return false };
                let ok = match id {
                    "gt-earlier" => self.plan.shift(b, -1),
                    "gt-later" => self.plan.shift(b, 1),
                    "gt-shorter" => self.plan.stretch(b, -1),
                    _ => self.plan.stretch(b, 1),
                };
                ok && self.say_bar(b)
            }
            "gt-undo" => self.plan.undo() && said(self, "undone".into()),
            "gt-reset" => self.plan.reset() && said(self, "plan reset".into()),
            "gt-today" => said(self, format!("Today {}", iso(TODAY))),
            "gt-owner" => {
                let o = owners(&self.plan.bars);
                self.gowner = match self.gowner.and_then(|w| o.iter().position(|x| *x == w)) {
                    Some(i) if i + 1 < o.len() => Some(o[i + 1]),
                    Some(_) => None,
                    None => o.first().copied(),
                };
                let w = self.gowner.map(|w| PEOPLE[w].0).unwrap_or("Everyone");
                said(self, format!("Owner: {w}"))
            }
            "td-view" => {
                self.list = !self.list;
                true
            }
            "td-clear" => {
                self.query.clear();
                self.team = None;
                self.status = None;
                self.office = None;
                true
            }
            _ => false,
        }
    }

    fn say_bar(&mut self, b: u32) -> bool {
        if let Some(x) = self.plan.bars.iter().find(|x| x.id == b) {
            self.said = Some(format!("{} · {}", x.name, bar_dates(x)));
        }
        true
    }

    /// Alt + arrows on the picked issue (`kb-left` …); false when nothing is picked or it cannot
    /// move.
    pub fn key_move(&mut self, id: &str) -> bool {
        let Some(i) = self.pick else { return false };
        let (dx, dy) = match id {
            "kb-left" => (-1, 0),
            "kb-right" => (1, 0),
            "kb-up" => (0, -1),
            _ => (0, 1),
        };
        match nudge(&mut self.board, i, dx, dy) {
            Some(s) => {
                self.said = Some(s);
                true
            }
            None => false,
        }
    }

    /// The directory's matches with the live search and filters.
    pub fn people(&self) -> Vec<usize> {
        find_people(
            &members(true),
            &self.query,
            self.team,
            self.status,
            self.office,
        )
    }
}

/// Static press ids for numbered presses (GPUI ids are `&'static str`).
pub mod ids {
    /// The `n`-th id of a table, or the last one.
    pub fn at(t: &'static [&'static str], n: usize) -> &'static str {
        t[n.min(t.len() - 1)]
    }
    macro_rules! table {
        ($(#[$m:meta])* $name:ident, $p:literal, $($n:literal),*) => {
            $(#[$m])*
            pub const $name: &[&str] = &[$(concat!($p, $n)),*];
        };
    }
    table!(
        /// `kb-pick-0` … `kb-pick-23`.
        KB_PICK, "kb-pick-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23
    );
    table!(
        /// `kb-grab-0` … `kb-grab-23`.
        KB_GRAB, "kb-grab-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23
    );
    table!(
        /// `gt-grab-0` … `gt-grab-23` (by bar id).
        GT_GRAB, "gt-grab-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23
    );
    table!(
        /// `kb-drop-<column>` in [`super::Col::ALL`] order.
        KB_DROP, "kb-drop-", "backlog", "todo", "progress", "review", "done"
    );
    table!(
        /// `mx-range-<key>` in [`super::RANGES`] order.
        MX_RANGE, "mx-range-", "7d", "30d", "90d", "12m"
    );
    table!(
        /// `pb-task-0` … `pb-task-19`.
        PB_TASK, "pb-task-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
    );
    table!(
        /// `gt-pick-0` … `gt-pick-23` (by bar id).
        GT_PICK, "gt-pick-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23
    );
    table!(
        /// `gt-at-0` … `gt-at-119` (a demo day).
        GT_AT, "gt-at-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119
    );
    table!(
        /// `td-fav-0` … `td-fav-15` (by person).
        TD_FAV, "td-fav-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `mx-view-0` … `mx-view-3` (the saved views).
        MX_VIEW, "mx-view-", 0, 1, 2, 3
    );
    table!(
        /// `mx-kpi-0` … `mx-kpi-4`.
        MX_KPI, "mx-kpi-", 0, 1, 2, 3, 4
    );
    table!(
        /// `pb-ms-0` … `pb-ms-4`.
        PB_MS, "pb-ms-", 0, 1, 2, 3, 4
    );
    table!(
        /// `pb-tab-0` … `pb-tab-2`.
        PB_TAB, "pb-tab-", 0, 1, 2
    );
    table!(
        /// `pb-health-0` … `pb-health-2`.
        PB_HEALTH, "pb-health-", 0, 1, 2
    );
    table!(
        /// `gt-zoom-0` … `gt-zoom-2`.
        GT_ZOOM, "gt-zoom-", 0, 1, 2
    );
    table!(
        /// `kb-label-0` … `kb-label-5`.
        KB_LABEL, "kb-label-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `td-team-0` … `td-team-3`.
        TD_TEAM, "td-team-", 0, 1, 2, 3
    );
    table!(
        /// `td-status-0` … `td-status-3`.
        TD_STATUS, "td-status-", 0, 1, 2, 3
    );
    table!(
        /// `td-office-0` … `td-office-4`.
        TD_OFFICE, "td-office-", 0, 1, 2, 3, 4
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_iso_and_weekdays_line_up() {
        assert_eq!(iso(0), "2026-10-01");
        assert_eq!(iso(TODAY), "2026-11-12");
        assert_eq!(iso(-1), "2026-09-30");
        assert_eq!(iso(61), "2026-12-01");
        assert_eq!(iso(92), "2027-01-01");
        assert_eq!(weekday(TODAY), 3, "2026-11-12 is a Thursday");
        assert_eq!(month_day(41), ("2026-11".to_string(), 11));
        assert_eq!(thousands(1234567), "1,234,567");
    }

    #[test]
    fn s12_gantt_tooltip_finds_free_space() {
        let b9 = [(585.0, 795.0)];
        let b11 = [(705.0, 840.0)];
        assert_eq!(
            tip_at(645.0, 234.0, 855.0, 1590.0, Some(&b9), Some(&b11)),
            TipAt::Right
        );
        assert_eq!(
            tip_at(645.0, 234.0, 1500.0, 1590.0, Some(&b9), Some(&b11)),
            TipAt::Left
        );
        assert_eq!(
            tip_at(645.0, 234.0, 855.0, 1590.0, Some(&[]), Some(&b11)),
            TipAt::Above
        );
        assert_eq!(
            tip_at(645.0, 234.0, 855.0, 1590.0, None, Some(&[(100.0, 200.0)])),
            TipAt::Below
        );
        assert_eq!(TipAt::Below.key(), "below");
        assert!(tip_w("2026-11-16 → 2026-11-29 · 14 days") > 200.0);
        let mut u = ArcUi::default();
        let o = owners(&u.plan.bars);
        assert!(u.press("gt-owner"));
        assert_eq!(u.gowner, Some(o[0]));
        for _ in 0..o.len() {
            u.press("gt-owner");
        }
        assert_eq!(u.gowner, None, "back to everyone");
    }

    #[test]
    fn s12_metrics_ranges_kpis_zoom() {
        for (r, (_, _, d)) in RANGES.iter().enumerate() {
            let s = series(0, r, false);
            assert_eq!(s.len(), if *d > 100 { 52 } else { *d as usize });
            assert_eq!(series(0, r, true).len(), s.len());
        }
        assert_eq!(
            range_span(1),
            ("2026-10-13".to_string(), "2026-11-11".to_string())
        );
        let (v, ch, good) = kpi_value(0, 1);
        assert!(v.contains(','), "{v}");
        assert!(ch.starts_with('+') && good, "{ch}");
        let (b, bch, _) = kpi_value(3, 1);
        assert!(b.ends_with('%') && bch.ends_with("pts"));
        assert!(kpi_value(4, 1).0.contains('m'));
        for r in 0..RANGES.len() {
            let ch: Vec<String> = (0..3).map(|k| kpi_value(k, r).1).collect();
            assert!(ch[0] != ch[1] && ch[1] != ch[2] && ch[0] != ch[2], "{ch:?}");
        }
        let t7: u64 = table(0, false, 0).iter().map(|r| r.n).sum();
        let t30: u64 = table(0, false, 1).iter().map(|r| r.n).sum();
        assert!(t7 < t30, "tables scale with the range");
        assert_eq!(zoom_window(30, 20, 5), Some((5, 20)));
        assert_eq!(zoom_window(30, 4, 5), None);
        let (top, step) = nice_top(5800.0);
        assert!(top >= 5800.0 && (top / step - 4.0).abs() < 1e-9);
        assert_eq!(axis_value(0, 4000.0), "4K");
        assert_eq!(axis_value(4, 150.0), "2:30");
    }

    #[test]
    fn s12_kanban_moves_limits_filters() {
        let mut b = issues(true);
        assert_eq!(col_counts(&b), [4, 5, 4, 3, 2]);
        assert!(!over_limit(&b, Col::Progress));
        assert!(move_issue(&mut b, 219, Col::Progress, Some(0)));
        assert_eq!(column(&b, Col::Progress, "", None)[0].id, 219);
        assert!(over_limit(&b, Col::Progress), "5 of 4");
        assert_eq!(
            nudge(&mut b, 219, 1, 0).as_deref(),
            Some("STF-219 moved to In review")
        );
        assert_eq!(nudge(&mut b, 195, 1, 0), None, "Done is the last column");
        let s = nudge(&mut b, 198, 0, 1).unwrap();
        assert_eq!(s, "STF-198 is 2 of 2 in Done");
        assert_eq!(column(&b, Col::Done, "", None)[1].id, 198);
        assert_eq!(nudge(&mut b, 198, 0, 1), None);
        assert_eq!(column(&b, Col::Todo, "tax", None).len(), 1);
        assert_eq!(column(&b, Col::Review, "", Some(1)).len(), 1);
        assert_eq!(column(&b, Col::Todo, "stf-224", None).len(), 1);
        let (d0, t) = points(&b);
        move_issue(&mut b, 204, Col::Done, None);
        assert_eq!(points(&b), (d0 + 5, t));
        assert_eq!(add_issue(&mut b, Col::Todo, "  "), None);
        assert_eq!(add_issue(&mut b, Col::Todo, "Gift cards"), Some(244));
        assert!(!assignees(&b).is_empty());
    }

    #[test]
    fn s12_project_stats_and_toggles() {
        let mut t = tasks(true);
        let (pct, open, busy, week, late, until) = stats(&t);
        assert_eq!((pct, open, busy, until), (37, 10, 4, 30));
        assert!(week >= 3 && late == 1, "{week} {late}");
        assert_eq!(ms_done(&t), 2);
        assert_eq!(ms_progress(&t, 2), (2, 8));
        assert_eq!(toggle_task(&mut t, 4), Some("completed"));
        assert_eq!(stats(&t).0, 43);
        assert_eq!(toggle_task(&mut t, 4), Some("reopened"));
        assert_eq!(toggle_task(&mut t, 99), None);
        assert_eq!(load_of(&t, 3), 2);
    }

    #[test]
    fn s12_gantt_drag_resize_undo_conflicts() {
        let mut p = Plan::new(true);
        assert!(conflicts(&p.bars).is_empty(), "{:?}", conflicts(&p.bars));
        assert!(p.shift(9, -3));
        assert_eq!(conflicts(&p.bars), vec![(8, 9)]);
        assert!(p.undo());
        assert!(conflicts(&p.bars).is_empty());
        assert!(p.stretch(5, 3));
        assert_eq!(p.bars.iter().find(|b| b.id == 5).unwrap().len, 18);
        assert!(!p.stretch(2, 1), "milestones do not stretch");
        assert!(!p.stretch(4, -6), "at least a day");
        assert!(p.stretch_start(5, 2));
        let b5 = p.bars.iter().find(|b| b.id == 5).unwrap().clone();
        assert_eq!((b5.start, end_of(&b5)), (30, 45));
        assert!(p.place(15, 50));
        assert_eq!(
            bar_dates(p.bars.iter().find(|b| b.id == 15).unwrap()),
            "2026-11-20 → 2026-11-29 · 10 days"
        );
        assert!(p.reset());
        assert_eq!(p.bars, plan(true));
        assert!(!p.reset());
        assert!(p.undo(), "reset can be undone");
        let (a, z, d) = phase_span(&plan(true), 0).unwrap();
        assert_eq!((a, z, d), (8, 18, 100));
        assert!(plan_done(&plan(true)) > 30);
    }

    #[test]
    fn s12_directory_search_filters_times() {
        let m = members(true);
        assert_eq!(m.len(), 16);
        assert_eq!(local_time(0), "14:32");
        assert_eq!(local_time(2), "08:32");
        assert!(!office_open(2) && office_open(0));
        assert_eq!(find_people(&m, "", None, None, None).len(), 16);
        assert_eq!(find_people(&m, "rust", None, None, None).len(), 2);
        assert_eq!(find_people(&m, "figma design", None, None, None).len(), 2);
        assert_eq!(find_people(&m, "", Some(1), None, None).len(), 3);
        assert_eq!(find_people(&m, "", None, Some("out"), None).len(), 2);
        assert_eq!(find_people(&m, "", None, None, Some(2)).len(), 2);
        assert!(find_people(&m, "zzz", None, None, None).is_empty());
        assert_eq!(office_count(&m, 2), (0, 2));
        assert_eq!(Avail::Out(TODAY + 4).words(), "Out until 2026-11-16");
    }

    #[test]
    fn s12_live_presses() {
        let mut u = ArcUi::default();
        assert!(u.press("mx-range-7d"));
        assert_eq!(u.range, 0);
        assert!(u.press("mx-zoom"));
        assert!(u.zoom.is_some());
        assert!(u.press("mx-range-90d") && u.zoom.is_none());
        assert!(u.press(ids::at(ids::MX_VIEW, 0)));
        assert_eq!((u.range, u.view), (0, Some(0)));
        assert!(u.press(ids::at(ids::KB_PICK, 4)));
        assert_eq!(u.pick, Some(219));
        assert!(u.press("kb-right"));
        assert_eq!(
            u.board.iter().find(|i| i.id == 219).unwrap().col,
            Col::Progress
        );
        assert!(u.press(ids::at(ids::KB_GRAB, 4)) && u.press(ids::at(ids::KB_GRAB, 4)));
        assert_eq!(u.pick, Some(219), "grab does not toggle");
        assert!(u.press(ids::at(ids::KB_DROP, 4)));
        assert_eq!(u.pick, None);
        assert!(!u.press("kb-left"), "nothing picked");
        assert!(u.press(ids::at(ids::PB_TASK, 4)));
        assert_eq!(u.tasks[4].st, 3);
        assert!(u.press("gt-pick-9") && u.press("gt-later"));
        assert_eq!(u.plan.bars.iter().find(|b| b.id == 9).unwrap().start, 40);
        assert!(u.press(ids::at(ids::GT_AT, 50)));
        assert_eq!(u.plan.bars.iter().find(|b| b.id == 9).unwrap().start, 50);
        assert!(u.press("gt-undo") && u.press("gt-undo"));
        assert_eq!(u.plan.bars, plan(true));
        assert!(u.press("td-team-1"));
        assert_eq!(u.people().len(), 3);
        assert!(u.press("td-fav-3"));
        assert!(!u.favs.contains(&3));
        assert!(u.press("td-clear"));
        assert_eq!(u.people().len(), 16);
        assert!(!u.press("nav-home"));
        assert_eq!(arc_split("month"), (ScreenState::Sample, "month"));
        assert_eq!(arc_split("loading"), (ScreenState::Loading, ""));
    }
}
