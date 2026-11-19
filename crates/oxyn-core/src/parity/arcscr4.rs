//! Screens S15 (Arc screens IV, the admin demo app): **Overview**, **Audit log**, **Status**,
//! **API keys and webhooks**, **Members** and **Billing** on the Arc screen frame (see
//! [`super::arcscr`]), and **Agent / Tool calls**: one agent and the tools it may call as an org
//! chart, with every call below. Behaviour and layout only: every company, person, key, endpoint
//! and line of copy is fictional and our own, and every date is an absolute ISO date.
//!
//! The rules are plain functions over plain data, so the Dioxus screens, the GPUI elements and
//! [`Arc4Ui`] (the live state a GPUI view keeps) agree:
//! - Overview: KPIs over 24h / 7d / 30d with a chart per KPI, an incident banner to dismiss
//!   ([`Overview`]);
//! - Audit log: events by type with a search, an event's before and after, load more and export
//!   ([`Audit`]);
//! - Status: six components with 90 days of uptime, today's state, incidents with their updates,
//!   subscribe ([`Status`]);
//! - Keys: create a key (the secret shows once), roll and revoke with a second press; webhooks
//!   with deliveries, a test event, retry and disable ([`Keys`]);
//! - Members: roles, invites with checks, seats, remove with a second press ([`Members`]);
//! - Billing: plans monthly or yearly with the prorated change, usage against limits, invoices
//!   and a past-due retry ([`Billing`]);
//! - Tool calls: tools with a mode each (auto / ask / off), a live call, approve or reject a
//!   waiting call, solve a failed one, pause the agent, connect and disconnect tools, ranges
//!   today / 7d / 30d ([`ToolCalls`]).

pub use super::arcscr::{
    HUES, ME, ME_INI, PEOPLE, ScreenState, States, TODAY, initials, iso, thousands,
};
pub use super::arcscr3::{money, money0, pct};

/// Overview states.
pub const OVERVIEW_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("week", "Last 7 days"),
    ("incident", "Incident"),
    ("errors", "Error rate"),
];
/// Audit log states.
pub const AUDIT_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("filtered", "Filtered"),
    ("detail", "Event open"),
    ("nomatch", "No matches"),
];
/// Status states.
pub const STATUS_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("degraded", "Degraded"),
    ("outage", "Outage"),
    ("maint", "Maintenance"),
];
/// API keys and webhooks states.
pub const KEYS_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("created", "Key created"),
    ("hooks", "Webhooks"),
    ("failing", "Failing endpoint"),
];
/// Members states.
pub const MEMBERS_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("invite", "Invite open"),
    ("full", "Seats full"),
    ("admins", "Admins only"),
];
/// Billing states.
pub const BILLING_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("yearly", "Yearly"),
    ("change", "Change plan"),
    ("pastdue", "Past due"),
];
/// Agent / Tool calls states.
pub const TOOLS_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("waiting", "Needs approval"),
    ("failed", "Failed call"),
    ("paused", "Agent paused"),
];

/// The seven S15 slugs, in set order.
pub const S15_SLUGS: [&str; 7] = [
    "admin-overview",
    "audit-log",
    "status-board",
    "api-keys-webhooks",
    "members-roles",
    "billing-usage",
    "agent-tool-calls",
];

/// The state bar of an S15 screen by slug.
pub fn s15_states(slug: &str) -> States {
    match slug {
        "audit-log" => AUDIT_STATES,
        "status-board" => STATUS_STATES,
        "api-keys-webhooks" => KEYS_STATES,
        "members-roles" => MEMBERS_STATES,
        "billing-usage" => BILLING_STATES,
        "agent-tool-calls" => TOOLS_STATES,
        _ => OVERVIEW_STATES,
    }
}

/// The data state and the screen's own state for a state key.
pub fn arc4_split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 21] = [
        "week", "incident", "errors", "filtered", "detail", "nomatch", "degraded", "outage",
        "maint", "created", "hooks", "failing", "invite", "full", "admins", "yearly", "change",
        "pastdue", "waiting", "failed", "paused",
    ];
    match OWN.iter().find(|x| **x == k) {
        Some(x) => (ScreenState::Sample, x),
        None => (ScreenState::from_key(k), ""),
    }
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *seed >> 33
}

/// A time of day from seconds: `09:41`.
pub fn hm(s: u32) -> String {
    format!("{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60)
}

/// A time of day from seconds: `09:41:07`.
pub fn hms(s: u32) -> String {
    format!("{:02}:{:02}:{:02}", (s / 3600) % 24, (s / 60) % 60, s % 60)
}

/// Milliseconds as `840 ms`, `1.2 s`, `1m 4s`.
pub fn dur(ms: u32) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{}.{} s", ms / 1000, (ms % 1000) / 100)
    } else {
        format!("{}m {}s", ms / 60_000, (ms / 1000) % 60)
    }
}

/// A count with k / M: `248_100` → `248.1k`.
pub fn short_n(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{}.{}M", n / 1_000_000, (n % 1_000_000) / 100_000)
    } else if n >= 10_000 {
        format!("{}.{}k", n / 1000, (n % 1000) / 100)
    } else {
        thousands(n)
    }
}

// ───────────────────────────────────────────────────────────── overview ──

/// Overview ranges: (label, points).
pub const OV_RANGES: [(&str, usize); 3] = [("24h", 24), ("7d", 7), ("30d", 30)];
/// Overview KPIs: (key, label).
pub const OV_KPIS: [(&str, &str); 4] = [
    ("requests", "Requests"),
    ("errors", "Error rate"),
    ("latency", "p95 latency"),
    ("members", "Active members"),
];

/// The series of KPI `k` over range `r` (deterministic).
pub fn ov_series(k: usize, r: usize) -> Vec<i64> {
    let n = OV_RANGES[r.min(2)].1;
    let mut seed = 77 + (k as u64) * 131 + (r as u64) * 17;
    let base: i64 = match k {
        0 => {
            if r == 0 {
                9_800
            } else {
                236_000
            }
        }
        1 => 38,
        2 => 182,
        _ => 41,
    };
    (0..n)
        .map(|i| {
            let wave = ((i as f64 / n as f64) * 6.28).sin();
            let noise = (lcg(&mut seed) % 1000) as f64 / 1000.0 - 0.5;
            let f = 1.0 + 0.12 * wave + 0.18 * noise + i as f64 / n as f64 * 0.08;
            ((base as f64) * f).round().max(1.0) as i64
        })
        .collect()
}

/// The headline of KPI `k` over range `r`.
pub fn ov_value(k: usize, r: usize) -> String {
    let v = ov_series(k, r);
    match k {
        0 => short_n(v.iter().sum::<i64>() as u64),
        1 => {
            let a = v.iter().sum::<i64>() / v.len() as i64;
            format!("{}.{:02}%", a / 100, a % 100)
        }
        2 => format!("{} ms", v.iter().copied().max().unwrap_or(0)),
        _ => v.last().copied().unwrap_or(0).to_string(),
    }
}

/// A point of KPI `k` as text.
pub fn ov_point(k: usize, x: i64) -> String {
    match k {
        0 => thousands(x as u64),
        1 => format!("{}.{:02}%", x / 100, x % 100),
        2 => format!("{x} ms"),
        _ => x.to_string(),
    }
}

/// The change of KPI `k` over range `r` in hundredths of a percent; for errors and latency
/// lower is better.
pub fn ov_delta(k: usize, r: usize) -> (i64, bool) {
    let v = ov_series(k, r);
    let h = v.len() / 2;
    let a: i64 = v[..h].iter().sum::<i64>().max(1);
    let b: i64 = v[h..].iter().sum();
    let bp = (b - a) * 10_000 / a;
    let good = if k == 1 || k == 2 { bp <= 0 } else { bp >= 0 };
    (bp, good)
}

/// The label of point `i` of range `r`: `14:00`, or an ISO date.
pub fn ov_label(r: usize, i: usize) -> String {
    let n = OV_RANGES[r.min(2)].1;
    if r == 0 {
        format!("{:02}:00", i)
    } else {
        iso(TODAY - (n - 1 - i) as i32)
    }
}

/// The Overview's live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Overview {
    /// Range index into [`OV_RANGES`].
    pub range: usize,
    /// KPI shown in the chart.
    pub kpi: usize,
    /// The incident banner is up.
    pub incident: bool,
    /// The point read out under the pointer.
    pub point: Option<usize>,
    /// With data.
    pub sample: bool,
}

impl Overview {
    /// A new overview.
    pub fn new(sample: bool) -> Self {
        Self {
            range: 1,
            kpi: 0,
            incident: false,
            point: None,
            sample,
        }
    }
    /// Picks a range.
    pub fn set_range(&mut self, r: usize) -> String {
        self.range = r.min(2);
        self.point = None;
        format!("Last {}", OV_RANGES[self.range].0)
    }
    /// Picks the chart's KPI.
    pub fn set_kpi(&mut self, k: usize) -> String {
        self.kpi = k.min(3);
        self.point = None;
        format!("{} chart", OV_KPIS[self.kpi].1)
    }
    /// Reads point `i` out.
    pub fn read(&mut self, i: usize) -> String {
        let v = ov_series(self.kpi, self.range);
        let i = i.min(v.len() - 1);
        self.point = Some(i);
        format!(
            "{}: {} {}",
            ov_label(self.range, i),
            ov_point(self.kpi, v[i]),
            OV_KPIS[self.kpi].1.to_lowercase()
        )
    }
}

// ───────────────────────────────────────────────────────────── audit ──

/// Event types: (key, label).
pub const AU_TYPES: [(&str, &str); 5] = [
    ("all", "All events"),
    ("auth", "Sign-in"),
    ("keys", "API keys"),
    ("members", "Members"),
    ("billing", "Billing"),
];

/// One audit event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Id.
    pub id: u32,
    /// Demo day.
    pub day: i32,
    /// Seconds since midnight (UTC).
    pub at: u32,
    /// Who did it (index into [`PEOPLE`]); `None` for the system.
    pub actor: Option<usize>,
    /// Type index into [`AU_TYPES`].
    pub kind: usize,
    /// What happened.
    pub action: &'static str,
    /// What it happened to.
    pub target: &'static str,
    /// Where from.
    pub ip: &'static str,
    /// It went through.
    pub ok: bool,
    /// The field before.
    pub before: &'static str,
    /// The field after.
    pub after: &'static str,
}

impl Event {
    /// `2026-10-13 09:41:07 UTC`.
    pub fn when(&self) -> String {
        format!("{} {} UTC", iso(self.day), hms(self.at))
    }
    /// The actor's name.
    pub fn who(&self) -> &'static str {
        match self.actor {
            Some(a) => PEOPLE[a % PEOPLE.len()].0,
            None => "System",
        }
    }
}

/// The sample events, newest first.
pub fn events(sample: bool) -> Vec<Event> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::type_complexity)]
    let raw: [(
        i32,
        u32,
        Option<usize>,
        usize,
        &str,
        &str,
        &str,
        bool,
        &str,
        &str,
    ); 16] = [
        (
            0,
            34_867,
            Some(0),
            2,
            "Created API key",
            "ci-deploy",
            "203.0.113.24",
            true,
            "—",
            "scope: write",
        ),
        (
            0,
            33_910,
            Some(1),
            3,
            "Changed role",
            "Theo Lindqvist",
            "198.51.100.7",
            true,
            "role: Member",
            "role: Admin",
        ),
        (
            0,
            31_502,
            Some(3),
            1,
            "Signed in",
            "Web · Firefox",
            "192.0.2.61",
            true,
            "—",
            "session: 7f3a",
        ),
        (
            0,
            30_044,
            None,
            1,
            "Blocked sign-in",
            "Unknown device",
            "203.0.113.200",
            false,
            "attempts: 2",
            "attempts: 3, locked 15 min",
        ),
        (
            0,
            26_210,
            Some(0),
            4,
            "Updated payment method",
            "Visa ending 4242",
            "203.0.113.24",
            true,
            "Mastercard ending 0005",
            "Visa ending 4242",
        ),
        (
            1,
            61_220,
            Some(2),
            2,
            "Revoked API key",
            "legacy-sync",
            "198.51.100.19",
            true,
            "status: active",
            "status: revoked",
        ),
        (
            1,
            58_090,
            Some(1),
            3,
            "Invited member",
            "kai@corvel.example",
            "198.51.100.7",
            true,
            "—",
            "role: Viewer",
        ),
        (
            1,
            51_400,
            Some(4),
            1,
            "Turned on two-step sign-in",
            "Priya Raman",
            "192.0.2.14",
            true,
            "2FA: off",
            "2FA: on",
        ),
        (
            1,
            40_315,
            None,
            4,
            "Charged invoice",
            "INV-2026-0310",
            "—",
            true,
            "status: open",
            "status: paid, $240.00",
        ),
        (
            2,
            70_011,
            Some(5),
            2,
            "Rolled API key",
            "analytics-export",
            "192.0.2.88",
            true,
            "ends: …9f2c",
            "ends: …41ad",
        ),
        (
            2,
            55_620,
            Some(0),
            3,
            "Removed member",
            "Jonas Ferreira",
            "203.0.113.24",
            true,
            "role: Member",
            "—",
        ),
        (
            3,
            47_800,
            Some(6),
            1,
            "Signed in",
            "macOS app",
            "198.51.100.43",
            true,
            "—",
            "session: 2c19",
        ),
        (
            3,
            39_000,
            Some(1),
            4,
            "Changed plan",
            "Team, monthly",
            "198.51.100.7",
            true,
            "plan: Starter",
            "plan: Team",
        ),
        (
            4,
            66_330,
            Some(2),
            2,
            "Created API key",
            "analytics-export",
            "198.51.100.19",
            true,
            "—",
            "scope: read",
        ),
        (
            5,
            28_745,
            None,
            1,
            "Expired session",
            "Rowan Vale",
            "—",
            true,
            "session: 91be",
            "—",
        ),
        (
            6,
            51_090,
            Some(3),
            3,
            "Accepted invite",
            "Mira Okafor",
            "192.0.2.61",
            true,
            "invite: pending",
            "role: Member",
        ),
    ];
    raw.iter()
        .enumerate()
        .map(|(i, r)| Event {
            id: i as u32,
            day: TODAY - r.0,
            at: r.1,
            actor: r.2,
            kind: r.3,
            action: r.4,
            target: r.5,
            ip: r.6,
            ok: r.7,
            before: r.8,
            after: r.9,
        })
        .collect()
}

/// The Audit log's live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Audit {
    /// All events.
    pub all: Vec<Event>,
    /// Type filter.
    pub kind: usize,
    /// Search text.
    pub q: String,
    /// Only failures.
    pub failed: bool,
    /// The open event.
    pub open: Option<u32>,
    /// Rows shown (load more adds 6).
    pub shown: usize,
}

impl Audit {
    /// A new log.
    pub fn new(sample: bool) -> Self {
        Self {
            all: events(sample),
            kind: 0,
            q: String::new(),
            failed: false,
            open: None,
            shown: 10,
        }
    }
    /// The events that match, before paging.
    pub fn matching(&self) -> Vec<Event> {
        let q = self.q.trim().to_lowercase();
        self.all
            .iter()
            .filter(|e| self.kind == 0 || e.kind == self.kind)
            .filter(|e| !self.failed || !e.ok)
            .filter(|e| {
                q.is_empty()
                    || e.action.to_lowercase().contains(&q)
                    || e.target.to_lowercase().contains(&q)
                    || e.who().to_lowercase().contains(&q)
                    || e.ip.contains(&q)
            })
            .cloned()
            .collect()
    }
    /// The rows on screen.
    pub fn list(&self) -> Vec<Event> {
        self.matching().into_iter().take(self.shown).collect()
    }
    /// Any filter set.
    pub fn filtered(&self) -> bool {
        self.kind != 0 || self.failed || !self.q.trim().is_empty()
    }
    /// Clears the filters.
    pub fn clear(&mut self) -> String {
        self.kind = 0;
        self.failed = false;
        self.q.clear();
        "Filters cleared".into()
    }
    /// Picks a type.
    pub fn set_kind(&mut self, k: usize) -> String {
        self.kind = k.min(AU_TYPES.len() - 1);
        format!(
            "{}: {} events",
            AU_TYPES[self.kind].1,
            self.matching().len()
        )
    }
    /// Opens event `id` (again to close).
    pub fn toggle(&mut self, id: u32) -> String {
        if self.open == Some(id) {
            self.open = None;
            return "Event closed".into();
        }
        self.open = Some(id);
        match self.all.iter().find(|e| e.id == id) {
            Some(e) => format!("{} – {}", e.action, e.target),
            None => String::new(),
        }
    }
    /// The open event.
    pub fn current(&self) -> Option<Event> {
        self.all.iter().find(|e| Some(e.id) == self.open).cloned()
    }
    /// Shows more rows; `None` when all show.
    pub fn more(&mut self) -> Option<String> {
        let n = self.matching().len();
        if self.shown >= n {
            return None;
        }
        self.shown = (self.shown + 6).min(n);
        Some(format!("Showing {} of {n}", self.shown))
    }
    /// The export, as said back.
    pub fn export(&self) -> String {
        format!(
            "Exported {} events as audit-{}.csv",
            self.matching().len(),
            iso(TODAY)
        )
    }
}

// ───────────────────────────────────────────────────────────── status ──

/// Components: (key, name, what it covers).
pub const COMPONENTS: [(&str, &str, &str); 6] = [
    ("api", "API", "REST and streaming endpoints"),
    ("dash", "Dashboard", "The web app"),
    ("hooks", "Webhooks", "Event delivery"),
    ("auth", "Sign-in", "Passwords, SSO and sessions"),
    ("jobs", "Background jobs", "Exports and imports"),
    ("files", "File storage", "Uploads and downloads"),
];

/// The days of history on the status page.
pub const STATUS_DAYS: usize = 90;

/// A component's health on a day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Svc {
    /// Operational.
    Up,
    /// Degraded performance.
    Slow,
    /// Outage.
    Down,
    /// Planned maintenance.
    Maint,
    /// No data.
    NoData,
}

impl Svc {
    /// The words.
    pub fn word(self) -> &'static str {
        match self {
            Self::Up => "Operational",
            Self::Slow => "Degraded",
            Self::Down => "Outage",
            Self::Maint => "Maintenance",
            Self::NoData => "No data",
        }
    }
    /// The hue.
    pub fn hue(self) -> &'static str {
        match self {
            Self::Up => "green",
            Self::Slow => "amber",
            Self::Down => "rose",
            Self::Maint => "blue",
            Self::NoData => "",
        }
    }
    /// The kebab key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Slow => "slow",
            Self::Down => "down",
            Self::Maint => "maint",
            Self::NoData => "none",
        }
    }
}

/// Past incidents: (component, days ago, health, title, minutes).
pub const PAST: [(usize, usize, Svc, &str, u32); 5] = [
    (2, 3, Svc::Slow, "Webhook deliveries delayed", 42),
    (0, 11, Svc::Down, "API errors in one region", 18),
    (4, 26, Svc::Slow, "Exports queued longer than usual", 95),
    (3, 48, Svc::Maint, "Planned sign-in maintenance", 30),
    (5, 71, Svc::Slow, "Slow uploads", 25),
];

/// Component `c` on day `d` (0 = 89 days ago … 89 = today) for the screen's state `mode`.
pub fn day_health(c: usize, d: usize, mode: &str) -> Svc {
    if d + 1 == STATUS_DAYS {
        return today_health(c, mode);
    }
    let ago = STATUS_DAYS - 1 - d;
    match PAST.iter().find(|p| p.0 == c && p.1 == ago) {
        Some(p) => p.2,
        None => Svc::Up,
    }
}

/// Component `c` today for state `mode`.
pub fn today_health(c: usize, mode: &str) -> Svc {
    match (mode, c) {
        ("degraded", 2) => Svc::Slow,
        ("outage", 0) | ("outage", 1) => Svc::Down,
        ("maint", 4) => Svc::Maint,
        _ => Svc::Up,
    }
}

/// Uptime of component `c` over the 90 days in hundredths of a percent.
pub fn uptime(c: usize, mode: &str) -> i64 {
    let mut lost = 0i64;
    for d in 0..STATUS_DAYS {
        lost += match day_health(c, d, mode) {
            Svc::Down => 40,
            Svc::Slow => 6,
            _ => 0,
        };
    }
    10_000 - lost * 100 / STATUS_DAYS as i64
}

/// `9_998` → `99.98%`.
pub fn pct2(bp: i64) -> String {
    format!("{}.{:02}%", bp / 100, bp % 100)
}

/// One update on an incident: (time UTC, stage, text).
pub type Update = (&'static str, &'static str, &'static str);

/// The open incident for state `mode`: (title, component, health, updates).
pub fn open_incident(mode: &str) -> Option<(&'static str, usize, Svc, &'static [Update])> {
    match mode {
        "degraded" => Some((
            "Webhook deliveries delayed",
            2,
            Svc::Slow,
            &[
                (
                    "09:52",
                    "Monitoring",
                    "Deliveries are catching up; the queue is at 12 minutes.",
                ),
                (
                    "09:31",
                    "Identified",
                    "One delivery worker pool ran out of memory. We added capacity.",
                ),
                (
                    "09:14",
                    "Investigating",
                    "Webhook deliveries are arriving late.",
                ),
            ],
        )),
        "outage" => Some((
            "API and dashboard unavailable",
            0,
            Svc::Down,
            &[
                (
                    "10:06",
                    "Identified",
                    "A database failover did not finish. We are moving traffic back.",
                ),
                (
                    "09:58",
                    "Investigating",
                    "Requests to the API and the dashboard return errors.",
                ),
            ],
        )),
        "maint" => Some((
            "Planned maintenance on background jobs",
            4,
            Svc::Maint,
            &[(
                "08:00",
                "In progress",
                "Exports and imports pause from 08:00 to 10:00 UTC. Nothing is lost.",
            )],
        )),
        _ => None,
    }
}

/// The status page's live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    /// The screen state (`""`, `degraded`, `outage`, `maint`).
    pub mode: &'static str,
    /// With data.
    pub sample: bool,
    /// The component whose day is read out.
    pub comp: usize,
    /// The day read out (0 … 89).
    pub day: Option<usize>,
    /// Subscribed to updates.
    pub subscribed: bool,
    /// The past incident open.
    pub past: Option<usize>,
}

impl Status {
    /// A new status page.
    pub fn new(sample: bool, mode: &'static str) -> Self {
        Self {
            mode,
            sample,
            comp: 0,
            day: None,
            subscribed: false,
            past: None,
        }
    }
    /// The page's banner: (words, hue).
    pub fn overall(&self) -> (&'static str, &'static str) {
        match self.mode {
            "degraded" => ("Some systems are degraded", "amber"),
            "outage" => ("Major outage", "rose"),
            "maint" => ("Planned maintenance in progress", "blue"),
            _ => ("All systems operational", "green"),
        }
    }
    /// Reads day `d` of component `c`.
    pub fn read(&mut self, c: usize, d: usize) -> String {
        self.comp = c.min(COMPONENTS.len() - 1);
        let d = d.min(STATUS_DAYS - 1);
        self.day = Some(d);
        let h = day_health(self.comp, d, self.mode);
        let date = iso(TODAY - (STATUS_DAYS - 1 - d) as i32);
        let what = PAST
            .iter()
            .find(|p| p.0 == self.comp && STATUS_DAYS - 1 - p.1 == d)
            .map(|p| format!(", {} ({} min)", p.3, p.4))
            .unwrap_or_default();
        format!("{} on {date}: {}{what}", COMPONENTS[self.comp].1, h.word())
    }
    /// Subscribe or unsubscribe.
    pub fn subscribe(&mut self) -> String {
        self.subscribed = !self.subscribed;
        if self.subscribed {
            "Subscribed: updates go to rowan@corvel.example".into()
        } else {
            "Unsubscribed".into()
        }
    }
}

// ───────────────────────────────────────────────────────────── keys ──

/// Key scopes: (key, label).
pub const SCOPES: [(&str, &str); 3] = [
    ("read", "Read only"),
    ("write", "Read and write"),
    ("admin", "Admin"),
];

/// One API key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKey {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// The last four characters shown.
    pub tail: String,
    /// Scope index.
    pub scope: usize,
    /// Day created.
    pub created: i32,
    /// Day last used.
    pub used: Option<i32>,
    /// Who made it.
    pub by: usize,
}

/// Webhook events.
pub const HOOK_EVENTS: [&str; 6] = [
    "invoice.paid",
    "member.added",
    "member.removed",
    "key.created",
    "export.ready",
    "incident.updated",
];

/// One delivery: (id, event index, status code, time, ms).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    /// Id.
    pub id: u32,
    /// Event index into [`HOOK_EVENTS`].
    pub event: usize,
    /// HTTP status (0 for a timeout).
    pub code: u16,
    /// When (seconds today).
    pub at: u32,
    /// How long.
    pub ms: u32,
}

impl Delivery {
    /// 2xx.
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.code)
    }
    /// `200`, `500`, `Timeout`.
    pub fn code_word(&self) -> String {
        if self.code == 0 {
            "Timeout".into()
        } else {
            self.code.to_string()
        }
    }
}

/// One webhook endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hook {
    /// Id.
    pub id: u32,
    /// URL.
    pub url: &'static str,
    /// Event indexes.
    pub events: Vec<usize>,
    /// Enabled.
    pub on: bool,
    /// Recent deliveries, newest first.
    pub deliveries: Vec<Delivery>,
}

impl Hook {
    /// The share of deliveries that went through, in percent.
    pub fn success(&self) -> u32 {
        if self.deliveries.is_empty() {
            return 100;
        }
        let ok = self.deliveries.iter().filter(|d| d.ok()).count();
        (ok * 100 / self.deliveries.len()) as u32
    }
    /// Failing: the last two deliveries failed.
    pub fn failing(&self) -> bool {
        self.deliveries.len() >= 2 && self.deliveries.iter().take(2).all(|d| !d.ok())
    }
}

/// API keys and webhooks live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keys {
    /// 0 keys, 1 webhooks.
    pub tab: usize,
    /// Keys.
    pub keys: Vec<ApiKey>,
    /// Revoked key names.
    pub revoked: Vec<String>,
    /// Endpoints.
    pub hooks: Vec<Hook>,
    /// The new-key form: (name, scope).
    pub draft: Option<(String, usize)>,
    /// The secret of the key just made: shown once.
    pub secret: Option<String>,
    /// The secret was copied.
    pub copied: bool,
    /// A key waiting for the second press of Revoke.
    pub confirm: Option<u32>,
    /// The open endpoint.
    pub hook: usize,
    next: u32,
}

fn mk_tail(seed: u32) -> String {
    let mut s = 91 + seed as u64 * 7_919;
    const A: &[u8] = b"0123456789abcdef";
    (0..4)
        .map(|_| A[(lcg(&mut s) % 16) as usize] as char)
        .collect()
}

impl Keys {
    /// A new page.
    pub fn new(sample: bool) -> Self {
        let keys = if sample {
            vec![
                ApiKey {
                    id: 1,
                    name: "ci-deploy".into(),
                    tail: "c41e".into(),
                    scope: 1,
                    created: TODAY,
                    used: Some(TODAY),
                    by: 0,
                },
                ApiKey {
                    id: 2,
                    name: "analytics-export".into(),
                    tail: "41ad".into(),
                    scope: 0,
                    created: TODAY - 4,
                    used: Some(TODAY - 1),
                    by: 2,
                },
                ApiKey {
                    id: 3,
                    name: "billing-sync".into(),
                    tail: "7b90".into(),
                    scope: 1,
                    created: TODAY - 19,
                    used: Some(TODAY - 2),
                    by: 1,
                },
                ApiKey {
                    id: 4,
                    name: "admin-cli".into(),
                    tail: "e2f3".into(),
                    scope: 2,
                    created: TODAY - 33,
                    used: None,
                    by: 0,
                },
            ]
        } else {
            Vec::new()
        };
        let d = |id, event, code, at, ms| Delivery {
            id,
            event,
            code,
            at,
            ms,
        };
        let hooks = if sample {
            vec![
                Hook {
                    id: 1,
                    url: "https://hooks.corvel.example/billing",
                    events: vec![0, 4],
                    on: true,
                    deliveries: vec![
                        d(9, 0, 200, 34_200, 182),
                        d(8, 4, 200, 31_010, 240),
                        d(7, 0, 200, 22_400, 205),
                        d(6, 4, 503, 18_880, 1_840),
                        d(5, 0, 200, 9_005, 199),
                    ],
                },
                Hook {
                    id: 2,
                    url: "https://ops.lumenpost.example/oxyn/events",
                    events: vec![1, 2, 3, 5],
                    on: true,
                    deliveries: vec![
                        d(4, 1, 200, 33_000, 96),
                        d(3, 3, 200, 30_120, 110),
                        d(2, 5, 200, 12_300, 88),
                    ],
                },
                Hook {
                    id: 3,
                    url: "https://legacy.corvel.example/hooks",
                    events: vec![2],
                    on: false,
                    deliveries: vec![],
                },
            ]
        } else {
            Vec::new()
        };
        Self {
            tab: 0,
            keys,
            revoked: Vec::new(),
            hooks,
            draft: None,
            secret: None,
            copied: false,
            confirm: None,
            hook: 0,
            next: 10,
        }
    }
    /// Opens the new-key form.
    pub fn open_new(&mut self) -> String {
        self.draft = Some((String::new(), 0));
        self.secret = None;
        "New API key".into()
    }
    /// What stops the draft from being created.
    pub fn check(&self) -> Option<&'static str> {
        let (n, _) = self.draft.as_ref()?;
        let n = n.trim();
        if n.is_empty() {
            Some("Name the key")
        } else if n.len() > 32 {
            Some("Keep the name under 32 characters")
        } else if self.keys.iter().any(|k| k.name == n) {
            Some("A key with this name exists")
        } else {
            None
        }
    }
    /// Creates the drafted key; the secret shows once.
    pub fn create(&mut self) -> Result<String, String> {
        if let Some(e) = self.check() {
            return Err(e.into());
        }
        let Some((n, scope)) = self.draft.take() else {
            return Err("Open New key first".into());
        };
        let id = self.next;
        self.next += 1;
        let tail = mk_tail(id);
        self.secret = Some(format!(
            "ox_live_{}{}{tail}",
            mk_tail(id + 50),
            mk_tail(id + 99)
        ));
        self.copied = false;
        self.keys.insert(
            0,
            ApiKey {
                id,
                name: n.trim().to_string(),
                tail,
                scope,
                created: TODAY,
                used: None,
                by: 0,
            },
        );
        Ok(format!(
            "Created {}. Copy the secret now; it will not show again.",
            n.trim()
        ))
    }
    /// Revoke: the first press asks, the second revokes.
    pub fn revoke(&mut self, id: u32) -> Option<String> {
        let i = self.keys.iter().position(|k| k.id == id)?;
        if self.confirm != Some(id) {
            self.confirm = Some(id);
            return Some(format!(
                "Press Revoke again to revoke {}",
                self.keys[i].name
            ));
        }
        self.confirm = None;
        let k = self.keys.remove(i);
        self.revoked.push(k.name.clone());
        Some(format!("Revoked {}. Requests with it now fail.", k.name))
    }
    /// Rolls key `id`: a new secret, the old one stops working.
    pub fn roll(&mut self, id: u32) -> Option<String> {
        let n = self.next;
        self.next += 1;
        let k = self.keys.iter_mut().find(|k| k.id == id)?;
        k.tail = mk_tail(n);
        k.created = TODAY;
        let name = k.name.clone();
        self.secret = Some(format!(
            "ox_live_{}{}{}",
            mk_tail(n + 50),
            mk_tail(n + 99),
            mk_tail(n)
        ));
        self.copied = false;
        Some(format!("Rolled {name}. Copy the new secret now."))
    }
    /// Sends a test event to the open endpoint.
    pub fn test(&mut self) -> Option<String> {
        let id = self.next;
        self.next += 1;
        let h = self.hooks.get_mut(self.hook)?;
        if !h.on {
            return Some("Turn the endpoint on to send a test".into());
        }
        let ev = h.events.first().copied().unwrap_or(0);
        h.deliveries.insert(
            0,
            Delivery {
                id,
                event: ev,
                code: 200,
                at: 35_100,
                ms: 143,
            },
        );
        Some(format!("Test {} delivered: 200 in 143 ms", HOOK_EVENTS[ev]))
    }
    /// Turns the open endpoint on or off.
    pub fn toggle(&mut self) -> Option<String> {
        let h = self.hooks.get_mut(self.hook)?;
        h.on = !h.on;
        Some(if h.on {
            "Endpoint on".into()
        } else {
            "Endpoint off: events are not sent".into()
        })
    }
    /// Retries delivery `id` of the open endpoint.
    pub fn retry(&mut self, id: u32) -> Option<String> {
        let h = self.hooks.get_mut(self.hook)?;
        let d = h.deliveries.iter_mut().find(|d| d.id == id)?;
        if d.ok() {
            return None;
        }
        d.code = 200;
        d.ms = 211;
        Some(format!("Redelivered {}: 200", HOOK_EVENTS[d.event]))
    }
}

// ───────────────────────────────────────────────────────────── members ──

/// Roles, highest first.
pub const ROLES: [&str; 4] = ["Owner", "Admin", "Member", "Viewer"];

/// One member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mem {
    /// Person index.
    pub who: usize,
    /// Role index.
    pub role: usize,
    /// Day last active.
    pub active: i32,
    /// Two-step sign-in on.
    pub mfa: bool,
}

/// One pending invite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    /// Id.
    pub id: u32,
    /// Email.
    pub email: String,
    /// Role index.
    pub role: usize,
    /// Day sent.
    pub sent: i32,
}

/// Members live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Members {
    /// Members.
    pub list: Vec<Mem>,
    /// Pending invites.
    pub invites: Vec<Invite>,
    /// Seats on the plan.
    pub seats: usize,
    /// Role filter.
    pub filter: Option<usize>,
    /// The invite form is open.
    pub form: bool,
    /// The invite email.
    pub email: String,
    /// The invite role.
    pub irole: usize,
    /// A member waiting for the second press of Remove.
    pub confirm: Option<usize>,
    next: u32,
}

/// What is wrong with an invite email, if anything.
pub fn invite_error(e: &str, m: &Members) -> Option<&'static str> {
    let e = e.trim();
    if e.is_empty() {
        return Some("Enter an email");
    }
    let Some((a, d)) = e.split_once('@') else {
        return Some("Enter an email like name@company.com");
    };
    if a.is_empty() || !d.contains('.') || d.starts_with('.') || d.ends_with('.') {
        return Some("Enter an email like name@company.com");
    }
    if m.invites.iter().any(|i| i.email.eq_ignore_ascii_case(e))
        || m.list
            .iter()
            .any(|x| member_email(x.who).eq_ignore_ascii_case(e))
    {
        return Some("Already a member or invited");
    }
    if m.used() >= m.seats {
        return Some("No seats left: add seats in Billing");
    }
    None
}

/// A member's email: `mira@corvel.example`.
pub fn member_email(who: usize) -> String {
    let first = PEOPLE[who % PEOPLE.len()]
        .0
        .split_whitespace()
        .next()
        .unwrap_or("x");
    format!("{}@corvel.example", first.to_lowercase())
}

impl Members {
    /// A new page; `full` fills every seat.
    pub fn new(sample: bool, full: bool) -> Self {
        let list = if sample {
            vec![
                Mem {
                    who: 0,
                    role: 0,
                    active: TODAY,
                    mfa: true,
                },
                Mem {
                    who: 1,
                    role: 1,
                    active: TODAY,
                    mfa: true,
                },
                Mem {
                    who: 2,
                    role: 1,
                    active: TODAY - 1,
                    mfa: true,
                },
                Mem {
                    who: 3,
                    role: 2,
                    active: TODAY,
                    mfa: false,
                },
                Mem {
                    who: 4,
                    role: 2,
                    active: TODAY - 3,
                    mfa: true,
                },
                Mem {
                    who: 5,
                    role: 2,
                    active: TODAY - 8,
                    mfa: false,
                },
                Mem {
                    who: 6,
                    role: 3,
                    active: TODAY - 15,
                    mfa: false,
                },
            ]
        } else {
            vec![Mem {
                who: 0,
                role: 0,
                active: TODAY,
                mfa: true,
            }]
        };
        let invites = if sample {
            vec![
                Invite {
                    id: 1,
                    email: "kai@corvel.example".into(),
                    role: 3,
                    sent: TODAY - 1,
                },
                Invite {
                    id: 2,
                    email: "noor@fieldline.example".into(),
                    role: 2,
                    sent: TODAY - 6,
                },
            ]
        } else {
            Vec::new()
        };
        let seats = if full { list.len() + invites.len() } else { 10 };
        Self {
            list,
            invites,
            seats,
            filter: None,
            form: false,
            email: String::new(),
            irole: 2,
            confirm: None,
            next: 10,
        }
    }
    /// Seats taken (members and invites).
    pub fn used(&self) -> usize {
        self.list.len() + self.invites.len()
    }
    /// Members in view.
    pub fn shown(&self) -> Vec<(usize, Mem)> {
        self.list
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, m)| match self.filter {
                Some(1) => m.role <= 1,
                Some(f) => m.role == f,
                None => true,
            })
            .collect()
    }
    /// Owners.
    pub fn owners(&self) -> usize {
        self.list.iter().filter(|m| m.role == 0).count()
    }
    /// Steps member `i` down a role (Viewer wraps to Admin); the last owner keeps the role.
    pub fn cycle_role(&mut self, i: usize) -> Option<String> {
        let owners = self.owners();
        let m = self.list.get_mut(i)?;
        if m.role == 0 && owners <= 1 {
            return Some("The workspace needs an owner: make someone else Owner first".into());
        }
        m.role = if m.role >= 3 { 1 } else { m.role + 1 };
        Some(format!(
            "{} is now {}",
            PEOPLE[m.who % PEOPLE.len()].0,
            ROLES[m.role]
        ))
    }
    /// Remove: the first press asks, the second removes.
    pub fn remove(&mut self, i: usize) -> Option<String> {
        let m = self.list.get(i)?;
        let name = PEOPLE[m.who % PEOPLE.len()].0;
        if m.role == 0 && self.owners() <= 1 {
            return Some("You cannot remove the only owner".into());
        }
        if self.confirm != Some(i) {
            self.confirm = Some(i);
            return Some(format!("Press Remove again to remove {name}"));
        }
        self.confirm = None;
        self.list.remove(i);
        Some(format!("Removed {name}; their seat is free"))
    }
    /// Sends the invite in the form.
    pub fn send(&mut self) -> Result<String, String> {
        if let Some(e) = invite_error(&self.email, self) {
            return Err(e.into());
        }
        let id = self.next;
        self.next += 1;
        let email = self.email.trim().to_string();
        self.invites.insert(
            0,
            Invite {
                id,
                email: email.clone(),
                role: self.irole,
                sent: TODAY,
            },
        );
        self.email.clear();
        self.form = false;
        Ok(format!("Invited {email} as {}", ROLES[self.irole]))
    }
    /// Resends invite `id`.
    pub fn resend(&mut self, id: u32) -> Option<String> {
        let v = self.invites.iter_mut().find(|v| v.id == id)?;
        v.sent = TODAY;
        Some(format!("Sent again to {}", v.email))
    }
    /// Revokes invite `id`.
    pub fn revoke(&mut self, id: u32) -> Option<String> {
        let i = self.invites.iter().position(|v| v.id == id)?;
        let v = self.invites.remove(i);
        Some(format!("Invite to {} revoked", v.email))
    }
}

// ───────────────────────────────────────────────────────────── billing ──

/// Plans: (name, per month in cents monthly, per month in cents yearly, seats, API calls).
pub const PLANS: [(&str, i64, i64, usize, u64); 3] = [
    ("Starter", 0, 0, 3, 10_000),
    ("Team", 24_000, 20_000, 10, 250_000),
    ("Scale", 79_000, 66_000, 40, 2_000_000),
];

/// One invoice: (number, day, cents, status 0 paid 1 open 2 failed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillInv {
    /// Number.
    pub no: String,
    /// Day.
    pub day: i32,
    /// Amount in cents.
    pub cents: i64,
    /// 0 paid, 1 open, 2 failed.
    pub status: u8,
}

/// Billing live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Billing {
    /// Plan index.
    pub plan: usize,
    /// Yearly billing.
    pub yearly: bool,
    /// The plan being changed to.
    pub pick: Option<usize>,
    /// Seats used.
    pub seats: usize,
    /// API calls this period.
    pub calls: u64,
    /// Storage in MB.
    pub storage: u64,
    /// Invoices, newest first.
    pub invoices: Vec<BillInv>,
    /// Days left in the period.
    pub left: i64,
}

impl Billing {
    /// A new page; `empty` is the free plan with nothing billed.
    pub fn new(sample: bool) -> Self {
        let inv = |no: &str, ago: i32, cents, status| BillInv {
            no: no.into(),
            day: TODAY - ago,
            cents,
            status,
        };
        if !sample {
            return Self {
                plan: 0,
                yearly: false,
                pick: None,
                seats: 1,
                calls: 0,
                storage: 0,
                invoices: Vec::new(),
                left: 18,
            };
        }
        Self {
            plan: 1,
            yearly: false,
            pick: None,
            seats: 9,
            calls: 186_400,
            storage: 3_420,
            invoices: vec![
                inv("INV-2026-0310", 1, 24_000, 0),
                inv("INV-2026-0274", 31, 24_000, 0),
                inv("INV-2026-0239", 62, 2_400, 0),
            ],
            left: 18,
        }
    }
    /// Price per month of plan `p`.
    pub fn price(&self, p: usize) -> i64 {
        let x = PLANS[p.min(2)];
        if self.yearly { x.2 } else { x.1 }
    }
    /// Past due: the newest invoice failed.
    pub fn past_due(&self) -> bool {
        self.invoices.first().is_some_and(|i| i.status == 2)
    }
    /// The prorated charge today for moving to `p` (negative is a credit).
    pub fn prorate(&self, p: usize) -> i64 {
        (self.price(p) - self.price(self.plan)) * self.left / 30
    }
    /// Monthly or yearly.
    pub fn toggle_cycle(&mut self) -> String {
        self.yearly = !self.yearly;
        if self.yearly {
            "Yearly billing: two months free".into()
        } else {
            "Monthly billing".into()
        }
    }
    /// Picks plan `p` to change to.
    pub fn pick(&mut self, p: usize) -> Option<String> {
        let p = p.min(2);
        if p == self.plan {
            self.pick = None;
            return Some(format!("{} is your plan", PLANS[p].0));
        }
        if self.seats > PLANS[p].3 {
            return Some(format!(
                "{} has {} seats; you use {}",
                PLANS[p].0, PLANS[p].3, self.seats
            ));
        }
        self.pick = Some(p);
        let c = self.prorate(p);
        Some(if c >= 0 {
            format!("Change to {}: {} today", PLANS[p].0, money(c))
        } else {
            format!("Change to {}: {} credit", PLANS[p].0, money(-c))
        })
    }
    /// Confirms the change.
    pub fn confirm(&mut self) -> Option<String> {
        let p = self.pick.take()?;
        self.plan = p;
        Some(format!("You are on {} from {}", PLANS[p].0, iso(TODAY)))
    }
    /// Retries the failed payment.
    pub fn retry(&mut self) -> Option<String> {
        let i = self.invoices.first_mut()?;
        if i.status != 2 {
            return None;
        }
        i.status = 0;
        Some(format!("Paid {} with Visa ending 4242", money(i.cents)))
    }
}

// ───────────────────────────────────────────────────────────── tool calls ──

/// How a tool may run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// On its own.
    Auto,
    /// After someone approves each call.
    Ask,
    /// Not at all.
    Off,
}

impl Mode {
    /// The panel's word.
    pub fn word(self) -> &'static str {
        match self {
            Self::Auto => "Allowed",
            Self::Ask => "Ask first",
            Self::Off => "Paused",
        }
    }
    /// The key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ask => "ask",
            Self::Off => "off",
        }
    }
}

/// The agent: (name, task, model, maker).
pub const TC_AGENT: (&str, &str, &str, &str) = (
    "Harbor",
    "Answer refund requests from the support inbox",
    "Claude Sonnet 4.5",
    "claude",
);

/// Tools: (name, function, brand id, source, description, default mode, params).
#[allow(clippy::type_complexity)]
pub const TC_TOOLS: [(&str, &str, &str, &str, &str, Mode, &[(&str, &str, bool)]); 7] = [
    (
        "Gmail",
        "gmail.read_thread",
        "gmail",
        "MCP · Gmail",
        "Reads the customer's thread",
        Mode::Auto,
        &[("thread_id", "string", true)],
    ),
    (
        "Postgres",
        "sql.query",
        "postgresql",
        "Built-in",
        "Looks up orders, read-only",
        Mode::Auto,
        &[("sql", "string", true), ("limit", "number", false)],
    ),
    (
        "Stripe",
        "stripe.refund",
        "stripe",
        "MCP · Stripe",
        "Refunds a charge",
        Mode::Ask,
        &[("charge", "string", true), ("amount", "number", true)],
    ),
    (
        "Notion",
        "notion.search",
        "notion",
        "MCP · Notion",
        "Searches the refund policy",
        Mode::Auto,
        &[("query", "string", true)],
    ),
    (
        "Linear",
        "linear.create_issue",
        "linear",
        "MCP · Linear",
        "Files a bug for engineering",
        Mode::Auto,
        &[("title", "string", true), ("team", "string", false)],
    ),
    (
        "Slack",
        "slack.post_message",
        "slack",
        "MCP · Slack",
        "Posts to #support-escalations",
        Mode::Ask,
        &[("channel", "string", true), ("text", "string", true)],
    ),
    (
        "GitHub",
        "github.search_code",
        "github",
        "MCP · GitHub",
        "Finds the code behind a bug",
        Mode::Off,
        &[("q", "string", true)],
    ),
];

/// Ranges: (label, days).
pub const TC_RANGES: [(&str, u32); 3] = [("Today", 1), ("7 days", 7), ("30 days", 30)];

/// A call's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallSt {
    /// Running now.
    Running,
    /// Waiting for approval.
    Waiting,
    /// Done.
    Done,
    /// Failed.
    Failed,
    /// Rejected by a person.
    Rejected,
}

impl CallSt {
    /// Word.
    pub fn word(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Rejected => "Rejected",
        }
    }
    /// Hue.
    pub fn hue(self) -> &'static str {
        match self {
            Self::Running => "blue",
            Self::Waiting => "amber",
            Self::Done => "green",
            Self::Failed => "rose",
            Self::Rejected => "",
        }
    }
    /// Key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }
}

/// One tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    /// Id.
    pub id: u32,
    /// Tool index into [`TC_TOOLS`].
    pub tool: usize,
    /// Days ago (0 today).
    pub ago: u32,
    /// Seconds since midnight.
    pub at: u32,
    /// How long (ms); a running call counts up.
    pub ms: u32,
    /// State.
    pub st: CallSt,
    /// Input, as JSON.
    pub input: String,
    /// Output, as JSON.
    pub output: Option<String>,
    /// Error.
    pub error: Option<String>,
    /// Tokens the result added.
    pub tokens: u32,
    /// The run.
    pub run: &'static str,
}

impl Call {
    /// `2026-10-13 09:41:07`.
    pub fn when(&self) -> String {
        format!("{} {}", iso(TODAY - self.ago as i32), hms(self.at))
    }
}

/// What a panel shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sel {
    /// The agent.
    Agent,
    /// A tool.
    Tool(usize),
    /// A call.
    Call(u32),
}

/// The calls the agent makes as the clock runs: (tool, input, output, ms, tokens).
const SCRIPT: [(usize, &str, &str, u32, u32); 5] = [
    (
        0,
        r#"{"thread_id":"t_8c21"}"#,
        r#"{"from":"ines@harbourline.example","subject":"Refund for order 4471"}"#,
        640,
        380,
    ),
    (
        1,
        r#"{"sql":"select * from orders where id = 4471"}"#,
        r#"{"rows":1,"total":"$84.00","status":"delivered"}"#,
        210,
        120,
    ),
    (
        3,
        r#"{"query":"refund window"}"#,
        r#"{"page":"Refunds","answer":"30 days from delivery"}"#,
        520,
        260,
    ),
    (
        4,
        r#"{"title":"Checkout shows the old price","team":"Payments"}"#,
        r#"{"id":"PAY-212","url":"linear.example/PAY-212"}"#,
        880,
        90,
    ),
    (
        2,
        r#"{"charge":"ch_3Q9x","amount":8400}"#,
        r#"{"refund":"re_51Kp","status":"succeeded"}"#,
        1_120,
        140,
    ),
];

/// Agent / Tool calls live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCalls {
    /// Tools: (index into [`TC_TOOLS`], mode).
    pub tools: Vec<(usize, Mode)>,
    /// Calls, newest first.
    pub calls: Vec<Call>,
    /// Range index.
    pub range: usize,
    /// The open panel.
    pub sel: Option<Sel>,
    /// The agent is paused.
    pub paused: bool,
    /// A tool waiting for the second press of Disconnect.
    pub disc: Option<usize>,
    /// The connect picker is open.
    pub picker: bool,
    /// Clock ticks seen.
    pub tick: u64,
    next: u32,
}

impl ToolCalls {
    /// A new screen for state `own` (`waiting`, `failed`, `paused`, or `""`).
    pub fn new(sample: bool, own: &str) -> Self {
        let tools = if sample {
            TC_TOOLS.iter().enumerate().map(|(i, t)| (i, t.5)).collect()
        } else {
            Vec::new()
        };
        let mut calls = Vec::new();
        if sample {
            let c = |id: u32,
                     tool,
                     ago,
                     at,
                     ms,
                     st,
                     input: &str,
                     output: Option<&str>,
                     error: Option<&str>,
                     tokens,
                     run| Call {
                id,
                tool,
                ago,
                at,
                ms,
                st,
                input: input.into(),
                output: output.map(Into::into),
                error: error.map(Into::into),
                tokens,
                run,
            };
            calls = vec![
                c(
                    20,
                    1,
                    0,
                    34_847,
                    1_400,
                    CallSt::Running,
                    r#"{"sql":"select * from orders where email = 'ines@harbourline.example'"}"#,
                    None,
                    None,
                    0,
                    "run_7f3a",
                ),
                c(
                    19,
                    0,
                    0,
                    34_840,
                    610,
                    CallSt::Done,
                    r#"{"thread_id":"t_8c21"}"#,
                    Some(
                        r#"{"from":"ines@harbourline.example","subject":"Refund for order 4471"}"#,
                    ),
                    None,
                    412,
                    "run_7f3a",
                ),
                c(
                    18,
                    2,
                    0,
                    34_310,
                    0,
                    CallSt::Waiting,
                    r#"{"charge":"ch_3Q8m","amount":12900}"#,
                    None,
                    None,
                    0,
                    "run_7e91",
                ),
                c(
                    17,
                    5,
                    0,
                    33_905,
                    2_240,
                    CallSt::Failed,
                    r##"{"channel":"#support-escalations","text":"Order 4402 double charged"}"##,
                    None,
                    Some("channel_not_found: the bot is not in #support-escalations"),
                    0,
                    "run_7e91",
                ),
                c(
                    16,
                    3,
                    0,
                    33_880,
                    480,
                    CallSt::Done,
                    r#"{"query":"double charge"}"#,
                    Some(r#"{"page":"Refunds","answer":"Refund the second charge in full"}"#),
                    None,
                    236,
                    "run_7e91",
                ),
                c(
                    15,
                    1,
                    0,
                    33_860,
                    190,
                    CallSt::Done,
                    r#"{"sql":"select * from charges where order_id = 4402"}"#,
                    Some(r#"{"rows":2,"amount":"$129.00"}"#),
                    None,
                    158,
                    "run_7e91",
                ),
                c(
                    14,
                    0,
                    0,
                    33_852,
                    702,
                    CallSt::Done,
                    r#"{"thread_id":"t_8b77"}"#,
                    Some(r#"{"from":"dev@quillmoor.example","subject":"Charged twice"}"#),
                    None,
                    388,
                    "run_7e91",
                ),
                c(
                    13,
                    2,
                    0,
                    30_120,
                    1_050,
                    CallSt::Done,
                    r#"{"charge":"ch_3Q2a","amount":4500}"#,
                    Some(r#"{"refund":"re_51Ab","status":"succeeded"}"#),
                    None,
                    120,
                    "run_7d02",
                ),
                c(
                    12,
                    4,
                    1,
                    52_310,
                    910,
                    CallSt::Done,
                    r#"{"title":"Refund email shows the wrong currency","team":"Payments"}"#,
                    Some(r#"{"id":"PAY-208"}"#),
                    None,
                    84,
                    "run_7c88",
                ),
                c(
                    11,
                    2,
                    1,
                    49_002,
                    0,
                    CallSt::Rejected,
                    r#"{"charge":"ch_3P9z","amount":56000}"#,
                    None,
                    Some("Rejected by Rowan Vale: over the $250 limit"),
                    0,
                    "run_7c40",
                ),
                c(
                    10,
                    1,
                    3,
                    40_020,
                    4_800,
                    CallSt::Failed,
                    r#"{"sql":"select * from refunds"}"#,
                    None,
                    Some("statement timeout after 4.8 s"),
                    0,
                    "run_7a13",
                ),
                c(
                    9,
                    3,
                    12,
                    37_400,
                    450,
                    CallSt::Done,
                    r#"{"query":"store credit"}"#,
                    Some(r#"{"page":"Store credit"}"#),
                    None,
                    210,
                    "run_6f02",
                ),
            ];
        }
        let mut s = Self {
            tools,
            calls,
            range: 0,
            sel: None,
            paused: false,
            disc: None,
            picker: false,
            tick: 0,
            next: 30,
        };
        match own {
            "waiting" => s.sel = Some(Sel::Call(18)),
            "failed" => s.sel = Some(Sel::Call(17)),
            "paused" => {
                s.paused = true;
                if let Some(c) = s.calls.iter_mut().find(|c| c.st == CallSt::Running) {
                    c.st = CallSt::Done;
                    c.output = Some(r#"{"rows":3}"#.into());
                    c.tokens = 164;
                }
            }
            _ => {}
        }
        s
    }
    /// The calls in the range.
    pub fn shown(&self) -> Vec<Call> {
        let d = TC_RANGES[self.range.min(2)].1;
        self.calls.iter().filter(|c| c.ago < d).cloned().collect()
    }
    /// The mode of tool `t`, `None` when not connected.
    pub fn mode(&self, t: usize) -> Option<Mode> {
        self.tools.iter().find(|x| x.0 == t).map(|x| x.1)
    }
    /// The tool called right now.
    pub fn live(&self) -> Option<usize> {
        self.calls
            .iter()
            .find(|c| c.st == CallSt::Running)
            .map(|c| c.tool)
    }
    /// Per tool in the range: (calls, failed, average ms).
    pub fn stats(&self, t: usize) -> (usize, usize, u32) {
        let v: Vec<Call> = self.shown().into_iter().filter(|c| c.tool == t).collect();
        let f = v.iter().filter(|c| c.st == CallSt::Failed).count();
        let done: Vec<u32> = v.iter().filter(|c| c.ms > 0).map(|c| c.ms).collect();
        let avg = if done.is_empty() {
            0
        } else {
            done.iter().sum::<u32>() / done.len() as u32
        };
        (v.len(), f, avg)
    }
    /// Totals in the range: (calls, failed, waiting, tokens).
    pub fn totals(&self) -> (usize, usize, usize, u32) {
        let v = self.shown();
        (
            v.len(),
            v.iter().filter(|c| c.st == CallSt::Failed).count(),
            v.iter().filter(|c| c.st == CallSt::Waiting).count(),
            v.iter().map(|c| c.tokens).sum(),
        )
    }
    /// Opens a panel (again to close).
    pub fn select(&mut self, s: Sel) -> String {
        self.disc = None;
        if self.sel == Some(s) {
            self.sel = None;
            return "Panel closed".into();
        }
        self.sel = Some(s);
        match s {
            Sel::Agent => format!("{} opened", TC_AGENT.0),
            Sel::Tool(t) => format!("{} opened", TC_TOOLS[t.min(6)].0),
            Sel::Call(id) => match self.calls.iter().find(|c| c.id == id) {
                Some(c) => format!("{} at {} opened", TC_TOOLS[c.tool].1, hms(c.at)),
                None => String::new(),
            },
        }
    }
    /// Sets the mode of the open tool.
    pub fn set_mode(&mut self, m: Mode) -> Option<String> {
        let Some(Sel::Tool(t)) = self.sel else {
            return None;
        };
        let x = self.tools.iter_mut().find(|x| x.0 == t)?;
        x.1 = m;
        Some(format!("{}: {}", TC_TOOLS[t].0, m.word()))
    }
    /// Disconnect: the first press asks, the second disconnects; its calls stay.
    pub fn disconnect(&mut self) -> Option<String> {
        let Some(Sel::Tool(t)) = self.sel else {
            return None;
        };
        if self.disc != Some(t) {
            self.disc = Some(t);
            return Some(format!(
                "Press Disconnect again to remove {}",
                TC_TOOLS[t].0
            ));
        }
        self.disc = None;
        self.tools.retain(|x| x.0 != t);
        self.sel = None;
        Some(format!(
            "{} disconnected; its calls stay in the table",
            TC_TOOLS[t].0
        ))
    }
    /// Tools not connected.
    pub fn unconnected(&self) -> Vec<usize> {
        (0..TC_TOOLS.len())
            .filter(|t| self.mode(*t).is_none())
            .collect()
    }
    /// Connects tool `t` (asks first by default).
    pub fn connect(&mut self, t: usize) -> Option<String> {
        if t >= TC_TOOLS.len() || self.mode(t).is_some() {
            return None;
        }
        self.tools.push((t, Mode::Ask));
        self.tools.sort_by_key(|x| x.0);
        self.picker = false;
        Some(format!("{} connected: Ask first", TC_TOOLS[t].0))
    }
    /// Pause or resume.
    pub fn pause(&mut self) -> String {
        self.paused = !self.paused;
        if self.paused {
            format!("{} paused", TC_AGENT.0)
        } else {
            format!("{} resumed", TC_AGENT.0)
        }
    }
    fn open_call(&mut self) -> Option<&mut Call> {
        let Some(Sel::Call(id)) = self.sel else {
            return None;
        };
        self.calls.iter_mut().find(|c| c.id == id)
    }
    /// Approves the open waiting call: it runs.
    pub fn approve(&mut self) -> Option<String> {
        let c = self.open_call()?;
        if c.st != CallSt::Waiting {
            return None;
        }
        c.st = CallSt::Running;
        c.ms = 0;
        Some(format!("Approved {}", TC_TOOLS[c.tool].1))
    }
    /// Rejects the open waiting call.
    pub fn reject(&mut self) -> Option<String> {
        let c = self.open_call()?;
        if c.st != CallSt::Waiting {
            return None;
        }
        c.st = CallSt::Rejected;
        c.error = Some("Rejected by Rowan Vale".into());
        Some(format!("Rejected {}", TC_TOOLS[c.tool].1))
    }
    /// Solves the open failed call: runs it again as a new call.
    pub fn solve(&mut self) -> Option<String> {
        let c = self.open_call()?.clone();
        if c.st != CallSt::Failed {
            return None;
        }
        if self.paused {
            return Some("Resume the agent to run it again".into());
        }
        let id = self.next;
        self.next += 1;
        let mut input = c.input.clone();
        if c.tool == 5 {
            input = r##"{"channel":"#support","text":"Order 4402 double charged"}"##.into();
        }
        self.calls.insert(
            0,
            Call {
                id,
                at: 35_000 + self.tick as u32,
                ago: 0,
                ms: 0,
                st: CallSt::Running,
                output: None,
                error: None,
                tokens: 0,
                input,
                ..c
            },
        );
        self.sel = Some(Sel::Call(id));
        Some(format!("Running {} again", TC_TOOLS[c.tool].1))
    }
    /// One clock tick (about a second): the running call counts up and finishes after its
    /// time; then, unless paused, the agent calls the next tool it may.
    pub fn step(&mut self) -> Option<String> {
        self.tick += 1;
        let mut said = None;
        if let Some(c) = self.calls.iter_mut().find(|c| c.st == CallSt::Running) {
            c.ms += 1000;
            if c.ms >= 2_000 {
                let k = SCRIPT.iter().find(|s| s.0 == c.tool);
                c.st = CallSt::Done;
                c.ms = k.map(|k| k.3).unwrap_or(c.ms);
                c.output = Some(k.map(|k| k.2).unwrap_or(r#"{"ok":true}"#).into());
                c.tokens = k.map(|k| k.4).unwrap_or(100);
                said = Some(format!("{} done", TC_TOOLS[c.tool].1));
            }
            return said;
        }
        if self.paused || self.tick % 3 != 0 {
            return None;
        }
        let n = (self.tick / 3) as usize;
        for k in 0..SCRIPT.len() {
            let s = SCRIPT[(n + k) % SCRIPT.len()];
            let Some(m) = self.mode(s.0) else { continue };
            if m == Mode::Off {
                continue;
            }
            let id = self.next;
            self.next += 1;
            let st = if m == Mode::Ask {
                CallSt::Waiting
            } else {
                CallSt::Running
            };
            self.calls.insert(
                0,
                Call {
                    id,
                    tool: s.0,
                    ago: 0,
                    at: 35_000 + self.tick as u32,
                    ms: 0,
                    st,
                    input: s.1.into(),
                    output: None,
                    error: None,
                    tokens: 0,
                    run: "run_7f3a",
                },
            );
            if self.calls.len() > 40 {
                self.calls.pop();
            }
            return Some(if m == Mode::Ask {
                format!("{} needs you", TC_TOOLS[s.0].1)
            } else {
                format!("Calling {}", TC_TOOLS[s.0].1)
            });
        }
        None
    }
}

// ───────────────────────────────────────────────────────────── live state ──

/// The live state of the seven S15 screens a GPUI view keeps (a host routes press ids to
/// [`Arc4Ui::press`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arc4Ui {
    /// Overview.
    pub ov: Overview,
    /// Audit log.
    pub au: Audit,
    /// Status.
    pub st: Status,
    /// Keys and webhooks.
    pub ak: Keys,
    /// Members.
    pub mb: Members,
    /// Billing.
    pub bl: Billing,
    /// Tool calls.
    pub tc: ToolCalls,
    /// The last thing done, said back.
    pub said: Option<String>,
}

impl Default for Arc4Ui {
    fn default() -> Self {
        Self {
            ov: Overview::new(true),
            au: Audit::new(true),
            st: Status::new(true, ""),
            ak: Keys::new(true),
            mb: Members::new(true, false),
            bl: Billing::new(true),
            tc: ToolCalls::new(true, ""),
            said: None,
        }
    }
}

fn nth(s: &str, prefix: &str) -> Option<usize> {
    s.strip_prefix(prefix)?.parse().ok()
}

impl Arc4Ui {
    /// Handles a press id of the S15 screens; false when the id is not one of them. Ids:
    /// - overview: `ov-range-<n>`, `ov-kpi-<n>`, `ov-pt-<n>`, `ov-incident`;
    /// - audit: `au-kind-<n>`, `au-open-<id>`, `au-failed`, `au-clear`, `au-more`, `au-export`;
    /// - status: `st-day-<c>-<d>`, `st-sub`, `st-past-<n>`;
    /// - keys: `ak-tab-<n>`, `ak-new`, `ak-scope-<n>`, `ak-create`, `ak-cancel`, `ak-copy`,
    ///   `ak-done`, `ak-revoke-<id>`, `ak-roll-<id>`, `wh-open-<n>`, `wh-test`, `wh-toggle`,
    ///   `wh-retry-<id>`;
    /// - members: `mb-invite`, `mb-irole-<n>`, `mb-send`, `mb-role-<i>`, `mb-rm-<i>`,
    ///   `mb-resend-<id>`, `mb-revoke-<id>`, `mb-filter-<n>`, `mb-all`;
    /// - billing: `bl-cycle`, `bl-pick-<n>`, `bl-confirm`, `bl-cancel`, `bl-retry`, `bl-inv-<n>`;
    /// - tool calls: `tc-agent`, `tc-tool-<n>`, `tc-call-<id>`, `tc-close`, `tc-mode-<auto|ask|off>`,
    ///   `tc-disc`, `tc-add`, `tc-connect-<n>`, `tc-pause`, `tc-approve`, `tc-reject`,
    ///   `tc-solve`, `tc-range-<n>`, `tc-tick`.
    pub fn press(&mut self, id: &str) -> bool {
        let out: Option<String> = if id.starts_with("ov-") {
            self.ov_press(id)
        } else if id.starts_with("au-") {
            self.au_press(id)
        } else if id.starts_with("st-") {
            self.st_press(id)
        } else if id.starts_with("ak-") || id.starts_with("wh-") {
            self.ak_press(id)
        } else if id.starts_with("mb-") {
            self.mb_press(id)
        } else if id.starts_with("bl-") {
            self.bl_press(id)
        } else if id.starts_with("tc-") {
            self.tc_press(id)
        } else {
            return false;
        };
        if let Some(s) = out {
            self.said = Some(s);
        }
        true
    }

    fn ov_press(&mut self, id: &str) -> Option<String> {
        if let Some(n) = nth(id, "ov-range-") {
            return Some(self.ov.set_range(n));
        }
        if let Some(n) = nth(id, "ov-kpi-") {
            return Some(self.ov.set_kpi(n));
        }
        if let Some(n) = nth(id, "ov-pt-") {
            return Some(self.ov.read(n));
        }
        if id == "ov-incident" {
            self.ov.incident = false;
            return Some("Incident banner dismissed".into());
        }
        None
    }

    fn au_press(&mut self, id: &str) -> Option<String> {
        let a = &mut self.au;
        if let Some(n) = nth(id, "au-kind-") {
            return Some(a.set_kind(n));
        }
        if let Some(n) = nth(id, "au-open-") {
            return Some(a.toggle(n as u32));
        }
        match id {
            "au-failed" => {
                a.failed = !a.failed;
                Some(if a.failed {
                    "Only failures".into()
                } else {
                    "All results".into()
                })
            }
            "au-clear" => Some(a.clear()),
            "au-more" => a.more(),
            "au-export" => Some(a.export()),
            "au-close" => {
                a.open = None;
                Some("Event closed".into())
            }
            _ => None,
        }
    }

    fn st_press(&mut self, id: &str) -> Option<String> {
        if let Some(rest) = id.strip_prefix("st-day-") {
            let (c, d) = rest.split_once('-')?;
            return Some(self.st.read(c.parse().ok()?, d.parse().ok()?));
        }
        if let Some(n) = nth(id, "st-past-") {
            self.st.past = if self.st.past == Some(n) {
                None
            } else {
                Some(n)
            };
            return Some(PAST.get(n).map(|p| p.3.to_string()).unwrap_or_default());
        }
        if id == "st-sub" {
            return Some(self.st.subscribe());
        }
        None
    }

    fn ak_press(&mut self, id: &str) -> Option<String> {
        let k = &mut self.ak;
        if let Some(n) = nth(id, "ak-tab-") {
            k.tab = n.min(1);
            return Some(if k.tab == 0 {
                "API keys".into()
            } else {
                "Webhooks".into()
            });
        }
        if let Some(n) = nth(id, "ak-scope-") {
            if let Some(d) = k.draft.as_mut() {
                d.1 = n.min(2);
                return Some(SCOPES[d.1].1.into());
            }
            return None;
        }
        if let Some(n) = nth(id, "ak-revoke-") {
            return k.revoke(n as u32);
        }
        if let Some(n) = nth(id, "ak-roll-") {
            return k.roll(n as u32);
        }
        if let Some(n) = nth(id, "wh-open-") {
            k.hook = n.min(k.hooks.len().saturating_sub(1));
            return k.hooks.get(k.hook).map(|h| h.url.to_string());
        }
        if let Some(n) = nth(id, "wh-retry-") {
            return k.retry(n as u32);
        }
        match id {
            "ak-new" => Some(k.open_new()),
            "ak-name" => {
                // A host without a text field types a sample name.
                if let Some(d) = k.draft.as_mut() {
                    d.0 = "support-bot".into();
                }
                Some("support-bot".into())
            }
            "ak-create" => Some(match k.create() {
                Ok(s) | Err(s) => s,
            }),
            "ak-cancel" => {
                k.draft = None;
                Some("Cancelled".into())
            }
            "ak-copy" => {
                k.copied = true;
                Some("Secret copied".into())
            }
            "ak-done" => {
                k.secret = None;
                Some("Secret hidden for good".into())
            }
            "wh-test" => k.test(),
            "wh-toggle" => k.toggle(),
            _ => None,
        }
    }

    fn mb_press(&mut self, id: &str) -> Option<String> {
        let m = &mut self.mb;
        if let Some(n) = nth(id, "mb-irole-") {
            m.irole = n.clamp(1, 3);
            return Some(format!("Invite as {}", ROLES[m.irole]));
        }
        if let Some(n) = nth(id, "mb-role-") {
            return m.cycle_role(n);
        }
        if let Some(n) = nth(id, "mb-rm-") {
            return m.remove(n);
        }
        if let Some(n) = nth(id, "mb-resend-") {
            return m.resend(n as u32);
        }
        if let Some(n) = nth(id, "mb-revoke-") {
            return m.revoke(n as u32);
        }
        if let Some(n) = nth(id, "mb-filter-") {
            m.filter = Some(n.min(3));
            return Some(format!("{}s", ROLES[n.min(3)]));
        }
        match id {
            "mb-all" => {
                m.filter = None;
                Some("Everyone".into())
            }
            "mb-invite" => {
                m.form = !m.form;
                Some(if m.form {
                    "Invite a member".into()
                } else {
                    "Invite closed".into()
                })
            }
            "mb-email" => {
                m.email = "sana@corvel.example".into();
                Some(m.email.clone())
            }
            "mb-send" => Some(match m.send() {
                Ok(s) | Err(s) => s,
            }),
            _ => None,
        }
    }

    fn bl_press(&mut self, id: &str) -> Option<String> {
        let b = &mut self.bl;
        if let Some(n) = nth(id, "bl-pick-") {
            return b.pick(n);
        }
        if let Some(n) = nth(id, "bl-inv-") {
            return b
                .invoices
                .get(n)
                .map(|i| format!("Downloaded {}.pdf", i.no));
        }
        match id {
            "bl-cycle" => Some(b.toggle_cycle()),
            "bl-confirm" => b.confirm(),
            "bl-cancel" => {
                b.pick = None;
                Some("Plan unchanged".into())
            }
            "bl-retry" => b.retry(),
            _ => None,
        }
    }

    fn tc_press(&mut self, id: &str) -> Option<String> {
        let t = &mut self.tc;
        if let Some(n) = nth(id, "tc-tool-") {
            return Some(t.select(Sel::Tool(n.min(6))));
        }
        if let Some(n) = nth(id, "tc-call-") {
            return Some(t.select(Sel::Call(n as u32)));
        }
        if let Some(n) = nth(id, "tc-connect-") {
            return t.connect(n);
        }
        if let Some(n) = nth(id, "tc-range-") {
            t.range = n.min(2);
            return Some(TC_RANGES[t.range].0.into());
        }
        match id {
            "tc-agent" => Some(t.select(Sel::Agent)),
            "tc-close" => {
                t.sel = None;
                t.disc = None;
                Some("Panel closed".into())
            }
            "tc-mode-auto" => t.set_mode(Mode::Auto),
            "tc-mode-ask" => t.set_mode(Mode::Ask),
            "tc-mode-off" => t.set_mode(Mode::Off),
            "tc-disc" => t.disconnect(),
            "tc-add" => {
                t.picker = !t.picker;
                Some(if t.picker {
                    "Connect a tool".into()
                } else {
                    "Closed".into()
                })
            }
            "tc-pause" => Some(t.pause()),
            "tc-approve" => t.approve(),
            "tc-reject" => t.reject(),
            "tc-solve" => t.solve(),
            "tc-tick" => t.step(),
            _ => None,
        }
    }
}

/// Static press ids for numbered presses (GPUI ids are `&'static str`).
pub mod ids {
    pub use super::super::arcscr::ids::at;
    macro_rules! table {
        ($(#[$m:meta])* $name:ident, $p:literal, $($n:literal),*) => {
            $(#[$m])*
            pub const $name: &[&str] = &[$(concat!($p, $n)),*];
        };
    }
    table!(
        /// `ov-range-0` … `ov-range-2`.
        OV_RANGE, "ov-range-", 0, 1, 2
    );
    table!(
        /// `ov-kpi-0` … `ov-kpi-3`.
        OV_KPI, "ov-kpi-", 0, 1, 2, 3
    );
    table!(
        /// `au-kind-0` … `au-kind-4`.
        AU_KIND, "au-kind-", 0, 1, 2, 3, 4
    );
    table!(
        /// `au-open-0` … `au-open-15`.
        AU_OPEN, "au-open-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `st-past-0` … `st-past-4`.
        ST_PAST, "st-past-", 0, 1, 2, 3, 4
    );
    table!(
        /// `st-day-<c>-89` (today of each component).
        ST_TODAY, "st-day-", "0-89", "1-89", "2-89", "3-89", "4-89", "5-89"
    );
    table!(
        /// `ak-tab-0` … `ak-tab-1`.
        AK_TAB, "ak-tab-", 0, 1
    );
    table!(
        /// `ak-scope-0` … `ak-scope-2`.
        AK_SCOPE, "ak-scope-", 0, 1, 2
    );
    table!(
        /// `ak-revoke-0` … `ak-revoke-15` (by key id).
        AK_REVOKE, "ak-revoke-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `ak-roll-0` … `ak-roll-15` (by key id).
        AK_ROLL, "ak-roll-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `wh-open-0` … `wh-open-2`.
        WH_OPEN, "wh-open-", 0, 1, 2
    );
    table!(
        /// `wh-retry-0` … `wh-retry-15` (by delivery id).
        WH_RETRY, "wh-retry-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `mb-irole-1` … `mb-irole-3` (index 0 is role 1).
        MB_IROLE, "mb-irole-", 1, 2, 3
    );
    table!(
        /// `mb-role-0` … `mb-role-11`.
        MB_ROLE, "mb-role-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    );
    table!(
        /// `mb-rm-0` … `mb-rm-11`.
        MB_RM, "mb-rm-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    );
    table!(
        /// `mb-resend-0` … `mb-resend-15` (by invite id).
        MB_RESEND, "mb-resend-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `mb-revoke-0` … `mb-revoke-15` (by invite id).
        MB_REVOKE, "mb-revoke-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `mb-filter-0` … `mb-filter-3`.
        MB_FILTER, "mb-filter-", 0, 1, 2, 3
    );
    table!(
        /// `bl-pick-0` … `bl-pick-2`.
        BL_PICK, "bl-pick-", 0, 1, 2
    );
    table!(
        /// `bl-inv-0` … `bl-inv-5`.
        BL_INV, "bl-inv-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `tc-tool-0` … `tc-tool-6`.
        TC_TOOL, "tc-tool-", 0, 1, 2, 3, 4, 5, 6
    );
    table!(
        /// `tc-connect-0` … `tc-connect-6`.
        TC_CONNECT, "tc-connect-", 0, 1, 2, 3, 4, 5, 6
    );
    table!(
        /// `tc-range-0` … `tc-range-2`.
        TC_RANGE, "tc-range-", 0, 1, 2
    );
    table!(
        /// `tc-call-0` … `tc-call-63` (by call id).
        TC_CALL, "tc-call-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43,
        44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s15_states_and_formats() {
        assert_eq!(S15_SLUGS.len(), 7);
        for s in S15_SLUGS {
            assert_eq!(s15_states(s).len(), 6, "{s}");
            for (k, _) in s15_states(s) {
                let _ = arc4_split(k);
            }
        }
        assert_eq!(arc4_split("pastdue"), (ScreenState::Sample, "pastdue"));
        assert_eq!(arc4_split("empty").0, ScreenState::Empty);
        assert_eq!(hms(34_867), "09:41:07");
        assert_eq!(dur(840), "840 ms");
        assert_eq!(dur(1_250), "1.2 s");
        assert_eq!(short_n(248_100), "248.1k");
        assert_eq!(pct2(9_998), "99.98%");
    }

    #[test]
    fn s15_overview_audit_status() {
        let mut o = Overview::new(true);
        assert_eq!(ov_series(0, 0).len(), 24);
        assert_eq!(ov_series(1, 2).len(), 30);
        assert_eq!(ov_series(2, 1), ov_series(2, 1));
        assert_eq!(o.set_range(0), "Last 24h");
        assert!(o.read(3).starts_with("03:00: "));
        assert!(ov_label(1, 6).starts_with("2026-"));
        let mut a = Audit::new(true);
        assert_eq!(a.list().len(), 10);
        assert_eq!(a.more().as_deref(), Some("Showing 16 of 16"));
        assert!(a.more().is_none());
        a.set_kind(2);
        assert!(a.list().iter().all(|e| e.kind == 2));
        a.clear();
        a.q = "firefox".into();
        assert_eq!(a.matching().len(), 1);
        a.q = "zzz".into();
        assert!(a.matching().is_empty());
        a.clear();
        a.failed = true;
        assert_eq!(a.matching().len(), 1);
        assert!(a.toggle(3).starts_with("Blocked sign-in"));
        assert_eq!(a.toggle(3), "Event closed");
        assert!(a.all[0].when().ends_with(" 09:41:07 UTC"));
        let mut s = Status::new(true, "");
        assert_eq!(s.overall().0, "All systems operational");
        assert!(uptime(0, "") < 10_000 && uptime(0, "") > 9_900);
        assert_eq!(uptime(1, ""), 10_000);
        assert!(s.read(2, 86).contains("Webhook deliveries delayed"));
        assert_eq!(today_health(2, "degraded"), Svc::Slow);
        assert_eq!(Status::new(true, "outage").overall().1, "rose");
        assert!(open_incident("maint").is_some() && open_incident("").is_none());
        assert!(s.subscribe().starts_with("Subscribed"));
    }

    #[test]
    fn s15_keys_members_billing() {
        let mut k = Keys::new(true);
        k.open_new();
        assert_eq!(k.create(), Err("Name the key".into()));
        k.draft = Some(("ci-deploy".into(), 0));
        assert_eq!(k.check(), Some("A key with this name exists"));
        k.draft = Some(("support-bot".into(), 1));
        assert!(k.create().is_ok());
        assert!(k.secret.as_ref().unwrap().starts_with("ox_live_"));
        assert_eq!(k.keys[0].name, "support-bot");
        let id = k.keys[0].id;
        assert!(k.revoke(id).unwrap().starts_with("Press Revoke again"));
        assert!(k.revoke(id).unwrap().starts_with("Revoked support-bot"));
        assert_eq!(k.keys.len(), 4);
        assert!(k.test().unwrap().contains("200"));
        let failed = k.hooks[0].deliveries.iter().find(|d| !d.ok()).unwrap().id;
        assert!(k.retry(failed).is_some());
        assert_eq!(k.hooks[0].success(), 100);
        k.hook = 2;
        k.toggle();
        assert!(k.hooks[2].on);
        let mut m = Members::new(true, false);
        assert_eq!(m.used(), 9);
        m.email = "bad".into();
        assert!(m.send().is_err());
        m.email = "kai@corvel.example".into();
        assert_eq!(m.send(), Err("Already a member or invited".into()));
        m.email = "sana@corvel.example".into();
        assert!(m.send().unwrap().starts_with("Invited sana"));
        assert!(m.cycle_role(0).unwrap().contains("needs an owner"));
        assert_eq!(
            m.cycle_role(3).as_deref(),
            Some("Priya Raman is now Viewer")
        );
        assert!(m.remove(6).unwrap().starts_with("Press Remove again"));
        assert!(m.remove(6).unwrap().starts_with("Removed"));
        let mut f = Members::new(true, true);
        f.email = "new@corvel.example".into();
        assert_eq!(f.send(), Err("No seats left: add seats in Billing".into()));
        let mut b = Billing::new(true);
        assert!(b.pick(0).unwrap().contains("3 seats"));
        assert!(b.pick(2).unwrap().starts_with("Change to Scale: $330.00"));
        assert!(b.confirm().unwrap().starts_with("You are on Scale"));
        b.toggle_cycle();
        assert_eq!(b.price(2), 66_000);
        b.invoices[0].status = 2;
        assert!(b.past_due());
        assert!(b.retry().is_some());
        assert!(!b.past_due());
    }

    #[test]
    fn s15_tool_calls() {
        let mut t = ToolCalls::new(true, "");
        assert_eq!(t.live(), Some(1));
        assert_eq!(t.totals().0, 8);
        t.range = 2;
        assert_eq!(t.totals().0, 12);
        t.range = 0;
        assert_eq!(t.step().as_deref(), Some("sql.query done"));
        assert!(t.live().is_none());
        assert!(t.step().is_none());
        assert_eq!(t.step().as_deref(), Some("Calling sql.query"));
        assert_eq!(t.live(), Some(1));
        t.select(Sel::Tool(2));
        assert_eq!(t.set_mode(Mode::Off).as_deref(), Some("Stripe: Paused"));
        assert!(
            t.disconnect()
                .unwrap()
                .starts_with("Press Disconnect again")
        );
        assert!(t.disconnect().unwrap().contains("disconnected"));
        assert!(t.mode(2).is_none());
        assert_eq!(t.unconnected(), vec![2]);
        assert!(t.connect(2).is_some());
        assert_eq!(t.mode(2), Some(Mode::Ask));
        let mut w = ToolCalls::new(true, "waiting");
        assert_eq!(w.approve().as_deref(), Some("Approved stripe.refund"));
        let mut w = ToolCalls::new(true, "waiting");
        assert_eq!(w.reject().as_deref(), Some("Rejected stripe.refund"));
        let mut f = ToolCalls::new(true, "failed");
        let n = f.calls.len();
        assert_eq!(
            f.solve().as_deref(),
            Some("Running slack.post_message again")
        );
        assert_eq!(f.calls.len(), n + 1);
        let mut p = ToolCalls::new(true, "paused");
        assert!(p.paused && p.live().is_none());
        for _ in 0..6 {
            assert!(p.step().is_none());
        }
        assert!(ToolCalls::new(false, "").tools.is_empty());
        let mut u = Arc4Ui::default();
        assert!(u.press("tc-tool-3"));
        assert_eq!(u.said.as_deref(), Some("Notion opened"));
        assert!(u.press("st-day-2-86"));
        assert!(u.press("mb-email"));
        assert!(u.press("mb-send"));
        assert!(u.said.as_deref().unwrap().starts_with("Invited sana"));
        assert!(!u.press("nav-x"));
    }
}
