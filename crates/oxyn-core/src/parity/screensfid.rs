//! Screens 1 (whole app frames): the shared app frame (Inset or Flat, plus a three-pane frame with
//! an icon rail), the AI Chat screens built on it – Start, Overview, Agents, Double sidebar,
//! Progress and Run – then Folders (Overview, Detail), Integrations, Kanban, CRM, the Workflow builder and the Settings shell. Each screen shows
//! Sample, Loading or Empty data. Behaviour only: every person, workspace, project and line of copy here
//! is fictional and our own.

/// Which data a screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenState {
    /// Filled with fictional sample data.
    Sample,
    /// Skeleton placeholders where the data will land.
    Loading,
    /// Nothing yet: each region says what will appear and how to start.
    Empty,
}

impl ScreenState {
    /// The three states, in toggle order.
    pub const ALL: [ScreenState; 3] = [
        ScreenState::Sample,
        ScreenState::Loading,
        ScreenState::Empty,
    ];

    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            ScreenState::Sample => "sample",
            ScreenState::Loading => "loading",
            ScreenState::Empty => "empty",
        }
    }

    /// From a key (sample for anything unknown).
    pub fn from_key(k: &str) -> ScreenState {
        match k.to_ascii_lowercase().as_str() {
            "loading" => ScreenState::Loading,
            "empty" => ScreenState::Empty,
            _ => ScreenState::Sample,
        }
    }
}

/// How the frame sits on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellLayout {
    /// The page floats as a card with rounded corners over a grey backdrop.
    Inset,
    /// Edge to edge, regions split by single hairlines.
    Flat,
}

impl ShellLayout {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            ShellLayout::Inset => "inset",
            ShellLayout::Flat => "flat",
        }
    }

    /// From a key (inset for anything unknown).
    pub fn from_key(k: &str) -> ShellLayout {
        if k.eq_ignore_ascii_case("flat") {
            ShellLayout::Flat
        } else {
            ShellLayout::Inset
        }
    }
}

/// Width of the native screen canvas (rule R1): every screen is authored at 1440 × 900 logical px
/// (16:10) and scaled to fit, never laid out at its column's width.
pub const STAGE_W: u32 = 1440;
/// Height of the native screen canvas.
pub const STAGE_H: u32 = 900;
/// Height of the state bar under a screen in its full view (rule R2).
pub const STATE_BAR_H: u32 = 52;
/// The data states the state bar offers: key and label.
pub const STAGE_STATES: [(&str, &str); 3] = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
];

/// The scale that fits the 1440 × 900 canvas in `avail_w` × `avail_h` (aspect kept), never above 1.
/// With no height bound the width alone decides.
pub fn stage_scale(avail_w: f32, avail_h: Option<f32>) -> f32 {
    let sw = avail_w / STAGE_W as f32;
    let s = match avail_h {
        Some(h) => sw.min(h / STAGE_H as f32),
        None => sw,
    };
    s.clamp(0.0, 1.0)
}

/// The fictional workspace every screen opens on.
pub const WORKSPACE: &str = "Kestrel Studio";
/// The fictional signed-in person.
pub const PERSON: &str = "Rowan Vale";
/// Their initials.
pub const PERSON_INI: &str = "RV";

/// Their (fictional) sign-in address, shown under the name in the account menu.
pub const PERSON_EMAIL: &str = "rowan@kestrel.example";
/// Their plan, shown under the name in the account row.
pub const PERSON_PLAN: &str = "Pro plan";

/// Sidebar width at 1440 px: default, min and max (drag the edge; double-click resets).
pub const SIDE_W: (u32, u32, u32) = (272, 220, 380);
/// Work area width at 1440 px: default, min and max.
pub const WORK_W: (u32, u32, u32) = (360, 280, 560);
/// Page header height.
pub const TOP_H: u32 = 48;
/// The gap around the inset page.
pub const INSET: u32 = 8;

/// Where an agent works: on this machine or in a cloud sandbox (the sidebar's Local / Cloud control).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvMode {
    /// On this machine.
    Local,
    /// In a cloud sandbox.
    Cloud,
}

impl EnvMode {
    /// Both, in control order.
    pub const ALL: [EnvMode; 2] = [EnvMode::Local, EnvMode::Cloud];
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Cloud => "Cloud",
        }
    }
    /// What the mode means, for the control's tooltip.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Local => "Agents work in folders on this machine",
            Self::Cloud => "Agents work in a cloud sandbox; close the lid and they keep going",
        }
    }
    /// From a key; anything else is Local.
    pub fn from_key(k: &str) -> Self {
        if k == "cloud" {
            Self::Cloud
        } else {
            Self::Local
        }
    }
}

/// One entry of the account menu: key, label, glyph, trailing hint (a shortcut, a current value, or
/// "›" for a submenu). An empty key is a divider.
pub type AcctItem = (&'static str, &'static str, &'static str, &'static str);

/// The account menu, top to bottom (opens upward from the account row).
pub const ACCOUNT_MENU: [AcctItem; 15] = [
    ("settings", "Settings", "gear", "⌘,"),
    ("usage", "Usage", "coin", "›"),
    ("upgrade", "Upgrade plan", "bolt", ""),
    ("", "", "", ""),
    ("language", "Language", "globe", "English"),
    ("appearance", "Appearance", "sun", "System"),
    ("shortcuts", "Keyboard shortcuts", "kbd", "?"),
    ("", "", "", ""),
    ("integrations", "Integrations", "apps", ""),
    ("download", "Download apps", "download", ""),
    ("changelog", "Changelog", "spark", ""),
    ("feedback", "Feedback & help", "help", "›"),
    ("", "", "", ""),
    ("logout", "Log out", "logout", ""),
    ("", "", "", ""),
];

/// The account menu entries without the trailing divider.
pub fn account_menu() -> &'static [AcctItem] {
    &ACCOUNT_MENU[..ACCOUNT_MENU.len() - 1]
}

/// The Usage flyout: plan, renewal, credits used of the month's total, and per-feature quotas.
#[derive(Clone, Debug, PartialEq)]
pub struct Usage {
    /// Plan name.
    pub plan: &'static str,
    /// When it renews.
    pub renews: &'static str,
    /// Credits used this period.
    pub used: u32,
    /// Credits in the period.
    pub total: u32,
    /// Per-feature quotas: label, used, limit.
    pub quotas: Vec<(&'static str, u32, u32)>,
}

impl Usage {
    /// Share of credits used, 0–100 (rounded down).
    pub fn pct(&self) -> u32 {
        (self.used.min(self.total) * 100)
            .checked_div(self.total)
            .unwrap_or(0)
    }
    /// Credits left.
    pub fn left(&self) -> u32 {
        self.total.saturating_sub(self.used)
    }
    /// Near the limit (80 % or more): the meter turns amber and the context card shows it.
    pub fn near(&self) -> bool {
        self.pct() >= 80
    }
}

/// The sample usage.
pub fn usage() -> Usage {
    Usage {
        plan: "Pro",
        renews: "Renews 1 Dec",
        used: 7_800,
        total: 10_000,
        quotas: vec![
            ("Agent runs", 142, 300),
            ("Images", 4, 10),
            ("Cloud hours", 11, 40),
        ],
    }
}

/// The cards the sidebar's context slot can show, by priority (one at a time; dismissing one shows
/// the next): key, title, line.
pub const CONTEXT_CARDS: [(&str, &str, &str); 3] = [
    (
        "running",
        "3 running · 1 needs you",
        "Agents keep going while you are away.",
    ),
    (
        "whatsnew",
        "New: shared folders",
        "Give a folder to the team; agents read it too.",
    ),
    ("setup", "Set up Kestrel", ""),
];

/// The first card not dismissed, if any.
pub fn context_card(dismissed: &[&str]) -> Option<(&'static str, &'static str, &'static str)> {
    CONTEXT_CARDS
        .into_iter()
        .find(|(k, _, _)| !dismissed.contains(k))
}

/// A project in the sidebar with its chats.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    /// Stable key.
    pub key: &'static str,
    /// Name.
    pub label: &'static str,
    /// Chats inside it.
    pub chats: Vec<&'static str>,
}

/// The sample projects.
pub fn projects() -> Vec<Project> {
    let p = |key, label, chats| Project { key, label, chats };
    vec![
        p(
            "autumn",
            "Autumn launch",
            vec!["Teaser hooks for week one", "Split the launch budget"],
        ),
        p("brand", "Brand kit", vec!["Logo lockups review"]),
        p("hiring", "Hiring loop", vec![]),
        p("research", "Field research", vec!["Interview notes digest"]),
    ]
}

/// A chat thread in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Pinned to the top group.
    pub pinned: bool,
    /// Has a reply you have not read.
    pub unread: bool,
    /// Minutes since the last message.
    pub mins: u32,
}

/// The sample chats (not in a project).
pub fn threads() -> Vec<Thread> {
    let t = |id, title: &str, pinned, unread, mins| Thread {
        id,
        title: title.to_string(),
        pinned,
        unread,
        mins,
    };
    vec![
        t(1, "Quarterly letter to members", true, false, 40),
        t(2, "Welcome email rewrite", false, true, 12),
        t(3, "November content plan", false, false, 190),
        t(4, "Venue shortlist for the offsite", false, false, 2900),
    ]
}

/// Pin or unpin a thread; returns its new pinned state (false when the id is unknown).
pub fn toggle_pin(list: &mut [Thread], id: u32) -> bool {
    match list.iter_mut().find(|t| t.id == id) {
        Some(t) => {
            t.pinned = !t.pinned;
            t.pinned
        }
        None => false,
    }
}

/// Mark a thread read when it is opened.
pub fn open_thread(list: &mut [Thread], id: u32) {
    if let Some(t) = list.iter_mut().find(|t| t.id == id) {
        t.unread = false;
    }
}

/// The pinned threads, then the rest, each in list order.
pub fn split_pinned(list: &[Thread]) -> (Vec<Thread>, Vec<Thread>) {
    list.iter().cloned().partition(|t| t.pinned)
}

/// A new chat's title from its first message: up to six words, first letter capitalised, at most
/// 40 characters (cut on a word with "…").
pub fn chat_title(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().take(6).collect();
    if words.is_empty() {
        return "New chat".to_string();
    }
    let mut out = String::new();
    for w in words {
        let next = if out.is_empty() {
            w.to_string()
        } else {
            format!("{out} {w}")
        };
        if next.chars().count() > 40 {
            if out.is_empty() {
                out = w.chars().take(39).collect();
            }
            out.push('…');
            break;
        }
        out = next;
    }
    let mut c = out.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => out,
    }
}

/// True when a message can be sent (anything but whitespace).
pub fn can_send(text: &str) -> bool {
    !text.trim().is_empty()
}

/// The greeting for an hour of the day (0–23).
pub fn greeting(hour: u32) -> &'static str {
    match hour % 24 {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "Working late",
    }
}

/// Minutes as a short age: "now", "12m", "3h", "2d".
pub fn ago(mins: u32) -> String {
    match mins {
        0 => "now".to_string(),
        1..=59 => format!("{mins}m"),
        60..=1439 => format!("{}h", mins / 60),
        _ => format!("{}d", mins / 1440),
    }
}

/// The setup checklist under the sidebar ("Set up Kestrel").
pub fn setup_steps() -> Vec<(&'static str, bool)> {
    vec![
        ("Start your first chat", true),
        ("Connect an app", false),
        ("Invite a teammate", false),
        ("Add a project", false),
        ("Try an agent run", false),
        ("Pick a default model", false),
        ("Turn on daily notes", false),
        ("Set a monthly budget", false),
    ]
}

/// Whole percent done, rounded down (0 when there are no steps).
pub fn pct_done(done: usize, total: usize) -> u32 {
    (done * 100).checked_div(total).unwrap_or(0) as u32
}

/// Starter suggestions on the Start screen: key, label, the prompt it fills in.
pub fn starters() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "docs",
            "Ask our docs",
            "What does our travel policy say about ",
        ),
        ("write", "Write a doc", "Draft a one-page brief about "),
        (
            "slides",
            "Make slides",
            "Turn these notes into six slides: ",
        ),
        ("table", "Build a table", "Make a table comparing "),
        ("image", "Draw an image", "Draw a flat illustration of "),
        ("deep", "Dig deeper", "Research in depth and cite sources: "),
        ("chart", "Chart it", "Chart this data by month: "),
    ]
}

/// Contribution-style activity levels (0–4) for `weeks` × 7 days, deterministic.
pub fn heat(weeks: usize) -> Vec<u8> {
    (0..weeks * 7)
        .map(|i| {
            let x = (i as u32).wrapping_mul(2_654_435_761).rotate_left(7) ^ 0x9E37;
            let ramp = (i * 4 / (weeks * 7).max(1)) as u32; // busier toward today
            let v = (x % 7 + ramp) / 2;
            if i % 7 >= 5 {
                (v / 2).min(4) as u8
            } else {
                v.min(4) as u8
            }
        })
        .collect()
}

/// Chats a heat map stands for (level × 3 per day).
pub fn heat_total(levels: &[u8]) -> u32 {
    levels.iter().map(|&l| l as u32 * 3).sum()
}

/// 1284 → "1,284".
pub fn thousands(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Signed change: "+12", "−3", "0".
pub fn delta(d: i32) -> String {
    match d {
        0 => "0".to_string(),
        d if d > 0 => format!("+{d}"),
        d => format!("−{}", d.unsigned_abs()),
    }
}

/// Percent of a cap used, clamped to 0–100.
pub fn used_pct(used: u32, cap: u32) -> u32 {
    (used as u64 * 100)
        .checked_div(cap as u64)
        .unwrap_or(0)
        .min(100) as u32
}

/// One agent in the crew working on a chat.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewMember {
    /// Stable key.
    pub key: &'static str,
    /// Role name.
    pub name: &'static str,
    /// What it is doing now.
    pub doing: &'static str,
    /// Finished.
    pub done: bool,
}

/// The crew on the Agents screen (the orchestrator is listed separately).
pub fn crew() -> Vec<CrewMember> {
    let m = |key, name, doing, done| CrewMember {
        key,
        name,
        doing,
        done,
    };
    vec![
        m(
            "lead",
            "Launch lead",
            "Keeps the brief and the budget in step",
            true,
        ),
        m(
            "brand",
            "Brand scout",
            "Reads the style guide and past posts",
            true,
        ),
        m("studio", "Studio", "Lays out six ad frames", false),
        m("people", "Audience desk", "Sizes the two audiences", true),
        m("art", "Art direction", "Picks crops and type", false),
    ]
}

/// A selling angle with its signal strength (1–3 bars).
#[derive(Clone, Debug, PartialEq)]
pub struct Angle {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// One line on why.
    pub why: &'static str,
    /// Signal strength, 1–3.
    pub signal: u8,
}

/// The angles to pick from on the brief.
pub fn angles() -> Vec<Angle> {
    let a = |key, label, why, signal| Angle {
        key,
        label,
        why,
        signal,
    };
    vec![
        a("feel", "How it feels", "Soft fabric, slow mornings", 3),
        a("fix", "What it fixes", "One layer for three seasons", 2),
        a("proof", "Who wears it", "Reviews and repeat buyers", 1),
    ]
}

/// The channels on the brief, with their default on/off.
pub fn channels() -> Vec<(&'static str, bool)> {
    vec![
        ("Photo feed", true),
        ("Short video", true),
        ("Link ads", false),
    ]
}

/// Ads a brief produces and roughly how long it takes: two per channel that is on, plus one per
/// extra format beyond the first; about 20 seconds each.
pub fn ad_estimate(channels_on: usize, formats: usize) -> (usize, usize) {
    let n = channels_on * 2 + formats.saturating_sub(1);
    (n, (n * 20).div_ceil(60))
}

/// Column letter for an index (0 → A … 25 → Z).
pub fn col_name(i: usize) -> char {
    (b'A' + (i % 26) as u8) as char
}

/// "B3" for column 1, row 2 (zero-based).
pub fn cell_ref(col: usize, row: usize) -> String {
    format!("{}{}", col_name(col), row + 1)
}

/// The comparison sheet the Double sidebar screen opens: header row, then data rows.
pub fn sheet() -> Vec<Vec<String>> {
    let rows: [[&str; 4]; 6] = [
        ["Fabric", "Weight g/m²", "Price / m", "Lead time d"],
        ["Washed linen", "185", "14.20", "21"],
        ["Linen–cotton", "160", "11.80", "14"],
        ["Hemp twill", "240", "16.50", "35"],
        ["Organic cotton", "150", "9.40", "10"],
        ["Tencel blend", "130", "12.90", "18"],
    ];
    rows.iter()
        .map(|r| r.iter().map(|c| c.to_string()).collect())
        .collect()
}

/// Sort the data rows (not the header) by a column; numbers compare as numbers.
pub fn sort_sheet(sheet: &mut [Vec<String>], col: usize, asc: bool) {
    if sheet.len() < 2 {
        return;
    }
    sheet[1..].sort_by(|a, b| {
        let (x, y) = (
            a.get(col).cloned().unwrap_or_default(),
            b.get(col).cloned().unwrap_or_default(),
        );
        let o = match (x.parse::<f64>(), y.parse::<f64>()) {
            (Ok(p), Ok(q)) => p.partial_cmp(&q).unwrap_or(std::cmp::Ordering::Equal),
            _ => x.to_lowercase().cmp(&y.to_lowercase()),
        };
        if asc { o } else { o.reverse() }
    });
}

/// A step of a run or of the progress pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    /// Finished.
    Done,
    /// Working on it now.
    Active,
    /// Not started.
    Pending,
}

impl StepState {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            StepState::Done => "done",
            StepState::Active => "active",
            StepState::Pending => "pending",
        }
    }

    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            StepState::Done => "Done",
            StepState::Active => "Working",
            StepState::Pending => "Up next",
        }
    }
}

/// The state of step `i` when `current` steps are finished (`current == len` means all done).
pub fn step_state(i: usize, current: usize) -> StepState {
    use std::cmp::Ordering::*;
    match i.cmp(&current) {
        Less => StepState::Done,
        Equal => StepState::Active,
        Greater => StepState::Pending,
    }
}

/// One step of an agent run's plan.
#[derive(Clone, Debug, PartialEq)]
pub struct RunStep {
    /// Title (editable).
    pub title: String,
    /// Why the step is there.
    pub why: &'static str,
    /// The tool it calls.
    pub tool: &'static str,
    /// Seconds it takes in the sample.
    pub secs: u32,
}

/// The sample run's plan.
pub fn run_plan() -> Vec<RunStep> {
    let s = |title: &str, why, tool, secs| RunStep {
        title: title.to_string(),
        why,
        tool,
        secs,
    };
    vec![
        s(
            "Pull October sales",
            "The update leads with revenue",
            "ledger.query",
            6,
        ),
        s(
            "Find last quarter's memo",
            "Keep the same headings",
            "files.search",
            4,
        ),
        s(
            "Draft the member update",
            "One page, plain words",
            "docs.write",
            9,
        ),
        s("Ask before sending", "Members see this", "mail.send", 5),
        s(
            "Book the review call",
            "Thirty minutes, this week",
            "calendar.add",
            3,
        ),
    ]
}

/// Move a step up (`up`) or down; returns the new index.
pub fn move_step(plan: &mut [RunStep], i: usize, up: bool) -> usize {
    if up && i > 0 {
        plan.swap(i, i - 1);
        i - 1
    } else if !up && i + 1 < plan.len() {
        plan.swap(i, i + 1);
        i + 1
    } else {
        i
    }
}

/// Seconds as a clock: 65 → "1:05", 3725 → "1:02:05".
pub fn clock(secs: u32) -> String {
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// The run's cost after `secs` seconds, in cents (a cent every two seconds, from 2 cents).
pub fn run_cents(secs: u32) -> u32 {
    2 + secs / 2
}

/// Cents as money: 14 → "$0.14".
pub fn money(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// One suggested edit in the run's draft.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// Text taken out.
    pub del: &'static str,
    /// Text put in.
    pub ins: &'static str,
}

/// The suggestions on the draft.
pub fn suggestions() -> Vec<Suggestion> {
    let s = |del, ins| Suggestion { del, ins };
    vec![
        s("grew a lot", "grew 18%"),
        s("we think", "we expect"),
        s("next month", "by 12 December"),
        s("some members", "212 members"),
        s("very soon", "on Friday"),
    ]
}

/// A review decision on a suggestion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Review {
    /// Not decided.
    Open,
    /// Kept.
    Accepted,
    /// Thrown out.
    Rejected,
}

/// How many suggestions are still open.
pub fn open_count(r: &[Review]) -> usize {
    r.iter().filter(|x| **x == Review::Open).count()
}

/// The next open suggestion after `from` (wrapping), or None when all are decided.
pub fn next_open(r: &[Review], from: usize) -> Option<usize> {
    let n = r.len();
    (1..=n)
        .map(|k| (from + k) % n)
        .find(|&i| r[i] == Review::Open)
}

/// The progress pane's checklist on the Progress screen.
pub fn progress_steps() -> Vec<&'static str> {
    vec![
        "Read the intake form",
        "Search the team wiki",
        "Match people to roles",
        "Draft the welcome plan",
        "Make the checklist sheet",
    ]
}

/// Tool calls in the Progress chat: tool, what it did, seconds.
pub fn progress_tools() -> Vec<(&'static str, &'static str, f32)> {
    vec![
        ("wiki.search", "Looked up the first-week pages", 1.8),
        ("drive.read", "Read the intake form", 0.9),
        ("people.match", "Matched three buddies by team", 2.4),
    ]
}

// ───────────────────────────────────────────────────────────── Folders ──

/// A shared folder on the Folders screens.
#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    /// Stable id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// One line on what it holds.
    pub note: String,
    /// Initials of the people it is shared with.
    pub members: Vec<String>,
    /// Minutes since something in it changed.
    pub mins: u32,
    /// Starred to the top.
    pub starred: bool,
    /// Agents may read it.
    pub agents: bool,
}

/// The sample folders.
pub fn folders() -> Vec<Folder> {
    let f = |id, name: &str, note: &str, members: Vec<&str>, mins, starred, agents| Folder {
        id,
        name: name.to_string(),
        note: note.to_string(),
        members: members.into_iter().map(String::from).collect(),
        mins,
        starred,
        agents,
    };
    vec![
        f(
            1,
            "Autumn launch",
            "Copy, timelines and the ad brief",
            vec!["RV", "JO", "TA"],
            35,
            true,
            true,
        ),
        f(
            2,
            "Brand kit",
            "Marks, type and the palette notes",
            vec!["RV", "MI"],
            1500,
            false,
            true,
        ),
        f(
            3,
            "Field research",
            "Interview notes and fabric tests",
            vec!["RV", "TA", "SE", "JO"],
            260,
            false,
            false,
        ),
        f(
            4,
            "Hiring loop",
            "Role notes and the interview plan",
            vec!["RV"],
            4300,
            false,
            false,
        ),
        f(
            5,
            "Member letters",
            "Every quarterly letter so far",
            vec!["RV", "MI", "JO"],
            9000,
            false,
            true,
        ),
        f(
            6,
            "Offsite",
            "Venues, travel and the agenda",
            vec!["RV", "SE"],
            20000,
            false,
            false,
        ),
    ]
}

/// Folders whose name or note contains the query (case-insensitive), starred first, then by most
/// recently changed.
pub fn folder_view(list: &[Folder], q: &str, by_name: bool) -> Vec<Folder> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Folder> = list
        .iter()
        .filter(|f| {
            q.is_empty() || f.name.to_lowercase().contains(&q) || f.note.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    v.sort_by(|a, b| {
        b.starred.cmp(&a.starred).then_with(|| {
            if by_name {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else {
                a.mins.cmp(&b.mins)
            }
        })
    });
    v
}

/// A file in a folder.
#[derive(Clone, Debug, PartialEq)]
pub struct FileRow {
    /// Stable id.
    pub id: u32,
    /// File name.
    pub name: String,
    /// Kind label (Doc, Sheet, Image, Notes).
    pub kind: &'static str,
    /// Owner initials.
    pub owner: &'static str,
    /// Minutes since edited.
    pub mins: u32,
    /// Size in KB.
    pub kb: u32,
}

/// The sample files of the first folder.
pub fn folder_files() -> Vec<FileRow> {
    let r = |id, name: &str, kind, owner, mins, kb| FileRow {
        id,
        name: name.to_string(),
        kind,
        owner,
        mins,
        kb,
    };
    vec![
        r(1, "launch-timeline.md", "Doc", "RV", 35, 14),
        r(2, "ad-brief.md", "Doc", "TA", 120, 9),
        r(3, "channel-budget.csv", "Sheet", "JO", 300, 42),
        r(4, "overshirt-hero.png", "Image", "TA", 1400, 2380),
        r(5, "teaser-hooks.md", "Notes", "RV", 2900, 6),
        r(6, "press-list.csv", "Sheet", "JO", 7200, 18),
    ]
}

/// Columns a file list sorts by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSort {
    /// File name, A to Z.
    Name,
    /// Kind label.
    Kind,
    /// Most recently edited first.
    Edited,
    /// Largest first.
    Size,
}

/// Sorts files by a column; `rev` flips the order.
pub fn sort_files(list: &mut [FileRow], by: FileSort, rev: bool) {
    list.sort_by(|a, b| {
        let o = match by {
            FileSort::Name => a.name.cmp(&b.name),
            FileSort::Kind => a.kind.cmp(b.kind).then_with(|| a.name.cmp(&b.name)),
            FileSort::Edited => a.mins.cmp(&b.mins),
            FileSort::Size => b.kb.cmp(&a.kb),
        };
        if rev { o.reverse() } else { o }
    });
}

/// A size in KB as words: `6 KB`, `2.3 MB`.
pub fn size_label(kb: u32) -> String {
    if kb < 1000 {
        format!("{kb} KB")
    } else {
        format!("{:.1} MB", kb as f32 / 1000.0)
    }
}

/// Total size of a list of files, as words.
pub fn total_size(list: &[FileRow]) -> String {
    size_label(list.iter().map(|f| f.kb).sum())
}

/// The next free "Untitled n" name among existing names.
pub fn untitled(names: &[String], stem: &str, ext: &str) -> String {
    let mut n = 1;
    loop {
        let c = if n == 1 {
            format!("{stem}{ext}")
        } else {
            format!("{stem}-{n}{ext}")
        };
        if !names.iter().any(|x| x == &c) {
            return c;
        }
        n += 1;
    }
}

/// A member's display name from their initials (fictional people).
pub fn member_name(ini: &str) -> &'static str {
    match ini {
        "RV" => PERSON,
        "JO" => "Jun Oduya",
        "TA" => "Tamsin Arvo",
        "MI" => "Mika Ilves",
        "SE" => "Sefa Ennis",
        _ => "Invited guest",
    }
}

/// Initials for an invite typed as an email address (`ada.lin@x.example` → `AL`), or `None` when
/// the text is not an address.
pub fn invite_initials(email: &str) -> Option<String> {
    let e = email.trim();
    let (user, host) = e.split_once('@')?;
    if user.is_empty() || !host.contains('.') || host.starts_with('.') || host.ends_with('.') {
        return None;
    }
    let mut parts = user.split(['.', '_', '-', '+']).filter(|p| !p.is_empty());
    let a = parts.next()?.chars().next()?.to_ascii_uppercase();
    let b = parts
        .next()
        .and_then(|p| p.chars().next())
        .map(|c| c.to_ascii_uppercase());
    let s = match b {
        Some(b) => format!("{a}{b}"),
        None => a.to_string(),
    };
    Some(s)
}

/// Recent activity in the first folder: who, what, when.
pub fn folder_activity() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("TA", "Edited ad-brief.md", "2h"),
        ("JO", "Added channel-budget.csv", "5h"),
        ("RV", "Let agents read the folder", "1d"),
    ]
}

// ───────────────────────────────────────────────────────────── Integrations ──

/// An app in the Integrations directory.
#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    /// Brand id (also the stable key).
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Category key: `files`, `chat`, `work`, `calendar` or `dev`.
    pub kind: &'static str,
    /// One line on what connecting it lets the assistant do (our own words).
    pub blurb: &'static str,
    /// Connected now.
    pub connected: bool,
    /// What it may do, with each permission on or off.
    pub scopes: Vec<(&'static str, bool)>,
    /// Minutes since the last sync (connected apps).
    pub synced: u32,
}

/// The directory's categories: key and label. `all` and `on` are views, the rest are kinds.
pub const APP_TABS: [(&str, &str); 7] = [
    ("all", "All"),
    ("on", "Connected"),
    ("files", "Files"),
    ("chat", "Messaging"),
    ("work", "Work"),
    ("calendar", "Calendar"),
    ("dev", "Developer"),
];

/// The sample directory; `connected` false everywhere gives the Empty state's catalogue.
pub fn apps(connected: bool) -> Vec<AppInfo> {
    let a = |id, name, kind, blurb, on: bool, scopes: &[&'static str], synced| AppInfo {
        id,
        name,
        kind,
        blurb,
        connected: on && connected,
        scopes: scopes.iter().map(|s| (*s, true)).collect(),
        synced,
    };
    vec![
        a(
            "googledrive",
            "Google Drive",
            "files",
            "Find and read the docs you point it at.",
            true,
            &["Read files you pick", "Make new files", "See folder names"],
            12,
        ),
        a(
            "dropbox",
            "Dropbox",
            "files",
            "Pull in shared folders and attachments.",
            false,
            &["Read files you pick", "Save copies of outputs"],
            0,
        ),
        a(
            "notion",
            "Notion",
            "work",
            "Read pages and write drafts into them.",
            true,
            &["Read pages you share", "Create pages", "Edit pages it made"],
            45,
        ),
        a(
            "slack",
            "Slack",
            "chat",
            "Catch up on threads and draft replies for you to send.",
            true,
            &[
                "Read channels you add",
                "Draft messages",
                "Post when you approve",
            ],
            3,
        ),
        a(
            "discord",
            "Discord",
            "chat",
            "Sum up busy servers and flag mentions.",
            false,
            &["Read channels you add", "See mentions"],
            0,
        ),
        a(
            "gmail",
            "Gmail",
            "chat",
            "Sort the inbox and draft answers.",
            false,
            &["Read mail", "Draft replies", "Send when you approve"],
            0,
        ),
        a(
            "googlecalendar",
            "Google Calendar",
            "calendar",
            "Find free time and hold slots.",
            true,
            &["See busy times", "Create events", "Invite guests"],
            30,
        ),
        a(
            "zoom",
            "Zoom",
            "calendar",
            "Add call links and pull recordings' notes.",
            false,
            &["Create meetings", "Read meeting notes"],
            0,
        ),
        a(
            "linear",
            "Linear",
            "work",
            "Turn notes into issues and track them.",
            false,
            &["Read issues", "Create issues", "Change status"],
            0,
        ),
        a(
            "figma",
            "Figma",
            "work",
            "Read frames and comments for briefs.",
            false,
            &["Read files you share", "Read comments"],
            0,
        ),
        a(
            "github",
            "GitHub",
            "dev",
            "Read code, open branches and pull requests.",
            false,
            &[
                "Read repositories you pick",
                "Open pull requests",
                "Comment",
            ],
            0,
        ),
        a(
            "sentry",
            "Sentry",
            "dev",
            "Group new errors and suggest a first look.",
            false,
            &["Read issues", "Resolve issues"],
            0,
        ),
    ]
}

/// Apps in a tab (`all`, `on` or a kind) whose name or blurb contains the query
/// (case-insensitive), connected first, then by name.
pub fn app_view(list: &[AppInfo], tab: &str, q: &str) -> Vec<AppInfo> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<AppInfo> = list
        .iter()
        .filter(|a| match tab {
            "all" => true,
            "on" => a.connected,
            k => a.kind == k,
        })
        .filter(|a| {
            q.is_empty()
                || a.name.to_lowercase().contains(&q)
                || a.blurb.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    v.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.cmp(b.name)));
    v
}

/// How many apps are connected.
pub fn connected_count(list: &[AppInfo]) -> usize {
    list.iter().filter(|a| a.connected).count()
}

/// "Synced just now" / "Synced 12m ago".
pub fn synced_words(mins: u32) -> String {
    if mins == 0 {
        "Synced just now".to_string()
    } else {
        format!("Synced {} ago", ago(mins))
    }
}

// ─────────────────────────────────────────────────────────────────── Kanban ──

/// A card on the Kanban board.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Stable id; the issue key is `KS-<id>`.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Column key (see [`KB_COLS`]).
    pub col: &'static str,
    /// Position inside its column (lower first).
    pub ord: u32,
    /// Label: `copy`, `design`, `ads` or `research`.
    pub tag: &'static str,
    /// Assignee initials, or empty when nobody has it.
    pub who: &'static str,
    /// Priority 0 (none) to 3 (urgent).
    pub pri: u8,
    /// Days until due (negative: overdue), when it has a date.
    pub due: Option<i32>,
    /// Sub-tasks done and total.
    pub sub: (u8, u8),
    /// A line of notes.
    pub note: &'static str,
}

/// The board's columns: key, label and work-in-progress limit (0: none).
pub const KB_COLS: [(&str, &str, usize); 4] = [
    ("todo", "To do", 0),
    ("doing", "In progress", 3),
    ("review", "In review", 2),
    ("done", "Done", 0),
];

/// Priority labels, 0 to 3.
pub const KB_PRI: [&str; 4] = ["No priority", "Low", "High", "Urgent"];

/// The sample board (`sample` false: an empty board).
pub fn board(sample: bool) -> Vec<Task> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    let t = |id, title: &str, col, ord, tag, who, pri, due, sub, note| Task {
        id,
        title: title.to_string(),
        col,
        ord,
        tag,
        who,
        pri,
        due,
        sub,
        note,
    };
    vec![
        t(
            14,
            "Shortlist three taglines",
            "todo",
            0,
            "copy",
            "",
            2,
            Some(3),
            (0, 3),
            "Keep each under six words.",
        ),
        t(
            15,
            "Book the studio day",
            "todo",
            1,
            "research",
            "JO",
            1,
            Some(9),
            (0, 0),
            "Two half days work too.",
        ),
        t(
            16,
            "Size the story frames",
            "todo",
            2,
            "design",
            "",
            0,
            None,
            (0, 4),
            "Four sizes from the master frame.",
        ),
        t(
            11,
            "Draft the launch email",
            "doing",
            0,
            "copy",
            "RV",
            3,
            Some(1),
            (2, 4),
            "Lead with the fabric, not the price.",
        ),
        t(
            12,
            "Colour pass on hero shots",
            "doing",
            1,
            "design",
            "TA",
            2,
            Some(2),
            (1, 3),
            "Match the swatch card under daylight.",
        ),
        t(
            9,
            "Cut the 15 s teaser",
            "review",
            0,
            "ads",
            "RV",
            2,
            Some(-1),
            (3, 3),
            "Needs a captions check.",
        ),
        t(
            8,
            "Audience notes from interviews",
            "done",
            0,
            "research",
            "SE",
            1,
            None,
            (5, 5),
            "Summed up in the folder.",
        ),
        t(
            7,
            "Budget split by channel",
            "done",
            1,
            "ads",
            "RV",
            0,
            None,
            (2, 2),
            "Signed off on Monday.",
        ),
    ]
}

/// Cards in one column whose title, note or key contains the query (case-insensitive),
/// optionally only those assigned to `who`, in board order.
pub fn column(list: &[Task], col: &str, q: &str, who: Option<&str>) -> Vec<Task> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Task> = list
        .iter()
        .filter(|t| t.col == col)
        .filter(|t| who.is_none_or(|w| t.who == w))
        .filter(|t| {
            q.is_empty()
                || t.title.to_lowercase().contains(&q)
                || t.note.to_lowercase().contains(&q)
                || task_key(t.id).to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    v.sort_by_key(|t| t.ord);
    v
}

/// The issue key shown on a card: `KS-14`.
pub fn task_key(id: u32) -> String {
    format!("KS-{id}")
}

/// Moves a card into `col` at position `at` (clamped), renumbering both columns.
/// Returns false when the id is unknown or nothing changed.
pub fn move_task(list: &mut [Task], id: u32, col: &'static str, at: usize) -> bool {
    let Some(from) = list.iter().find(|t| t.id == id).map(|t| (t.col, t.ord)) else {
        return false;
    };
    let mut ids: Vec<u32> = column(list, col, "", None)
        .iter()
        .map(|t| t.id)
        .filter(|x| *x != id)
        .collect();
    let at = at.min(ids.len());
    ids.insert(at, id);
    let before: Vec<u32> = column(list, col, "", None).iter().map(|t| t.id).collect();
    if from.0 == col && before == ids {
        return false;
    }
    for (i, x) in ids.iter().enumerate() {
        if let Some(t) = list.iter_mut().find(|t| t.id == *x) {
            t.col = col;
            t.ord = i as u32;
        }
    }
    if from.0 != col {
        let rest: Vec<u32> = column(list, from.0, "", None)
            .iter()
            .map(|t| t.id)
            .collect();
        for (i, x) in rest.iter().enumerate() {
            if let Some(t) = list.iter_mut().find(|t| t.id == *x) {
                t.ord = i as u32;
            }
        }
    }
    true
}

/// The column next to `col` (`step` -1 left, +1 right), if any.
pub fn next_col(col: &str, step: i32) -> Option<&'static str> {
    let i = KB_COLS.iter().position(|(k, _, _)| *k == col)? as i32 + step;
    KB_COLS.get(usize::try_from(i).ok()?).map(|(k, _, _)| *k)
}

/// The column's label.
pub fn col_label(col: &str) -> &'static str {
    KB_COLS
        .iter()
        .find(|(k, _, _)| *k == col)
        .map(|(_, l, _)| *l)
        .unwrap_or("Other")
}

/// True when a column with a limit holds more cards than it allows.
pub fn over_limit(list: &[Task], col: &str) -> bool {
    KB_COLS
        .iter()
        .find(|(k, _, _)| *k == col)
        .is_some_and(|(_, _, n)| *n > 0 && list.iter().filter(|t| t.col == col).count() > *n)
}

/// Adds a card at the bottom of `col` and returns its id.
pub fn add_task(list: &mut Vec<Task>, col: &'static str, title: &str) -> Option<u32> {
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let ord = list.iter().filter(|t| t.col == col).count() as u32;
    list.push(Task {
        id,
        title: title.to_string(),
        col,
        ord,
        tag: "copy",
        who: "",
        pri: 0,
        due: None,
        sub: (0, 0),
        note: "",
    });
    Some(id)
}

/// "Due today", "Due in 3d", "1d late".
pub fn due_words(d: i32) -> String {
    match d {
        0 => "Due today".to_string(),
        1.. => format!("Due in {d}d"),
        _ => format!("{}d late", -d),
    }
}

// ────────────────────────────────────────────────────────────────────── CRM ──

/// A deal row in the CRM table.
#[derive(Clone, Debug, PartialEq)]
pub struct Deal {
    /// Stable id.
    pub id: u32,
    /// Company name (editable).
    pub company: String,
    /// The person you talk to there.
    pub contact: &'static str,
    /// Stage key (see [`CRM_STAGES`]).
    pub stage: &'static str,
    /// Value in whole dollars (editable).
    pub value: u32,
    /// Owner initials, or empty when nobody owns it.
    pub owner: &'static str,
    /// Days until the expected close (negative: past it).
    pub close: i32,
    /// A line of notes.
    pub note: &'static str,
}

/// Deal stages: key and label, in pipeline order.
pub const CRM_STAGES: [(&str, &str); 5] = [
    ("lead", "Lead"),
    ("qualified", "Qualified"),
    ("proposal", "Proposal"),
    ("won", "Won"),
    ("lost", "Lost"),
];

/// Saved views shown as tabs: key and label.
pub const CRM_VIEWS: [(&str, &str); 4] = [
    ("all", "All deals"),
    ("mine", "My deals"),
    ("soon", "Closing soon"),
    ("won", "Won"),
];

/// A sortable CRM column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DealSort {
    /// Company, A to Z.
    Company,
    /// Pipeline order of the stage.
    Stage,
    /// Largest value first.
    Value,
    /// Soonest close first.
    Close,
}

/// The sample deals (`sample` false: none).
pub fn deals(sample: bool) -> Vec<Deal> {
    if !sample {
        return Vec::new();
    }
    #[allow(clippy::too_many_arguments)]
    let d = |id, company: &str, contact, stage, value, owner, close, note| Deal {
        id,
        company: company.to_string(),
        contact,
        stage,
        value,
        owner,
        close,
        note,
    };
    vec![
        d(
            1,
            "Harrow & Finch",
            "Pell Okafor",
            "proposal",
            24_000,
            "RV",
            6,
            "Wants the yearly plan with two seats added.",
        ),
        d(
            2,
            "Lumen Pantry",
            "Dara Quist",
            "qualified",
            8_400,
            "JO",
            18,
            "Trial ends next month; prefers email.",
        ),
        d(
            3,
            "Corvid Freight",
            "Wren Haldane",
            "lead",
            3_200,
            "",
            40,
            "Came in from the spring webinar.",
        ),
        d(
            4,
            "Basalt Studio",
            "Odile Marsh",
            "won",
            12_600,
            "RV",
            -3,
            "Signed; kickoff booked.",
        ),
        d(
            5,
            "Quillfield School",
            "Kit Ferreira",
            "proposal",
            15_000,
            "TA",
            11,
            "Needs a quote split by term.",
        ),
        d(
            6,
            "Tidewell Bikes",
            "Noor Sallis",
            "lost",
            5_800,
            "SE",
            -12,
            "Went with a cheaper plan for now.",
        ),
        d(
            7,
            "Marrowind Labs",
            "Bram Tolliver",
            "qualified",
            31_500,
            "RV",
            25,
            "Security review in progress.",
        ),
    ]
}

/// The stage's label.
pub fn stage_label(stage: &str) -> &'static str {
    CRM_STAGES
        .iter()
        .find(|(k, _)| *k == stage)
        .map(|(_, l)| *l)
        .unwrap_or("Other")
}

/// The stage's key as a static string, if it is one.
pub fn stage_key(stage: &str) -> Option<&'static str> {
    CRM_STAGES.iter().map(|(k, _)| *k).find(|k| *k == stage)
}

fn open_deal(d: &Deal) -> bool {
    d.stage != "won" && d.stage != "lost"
}

/// True when a deal belongs in a saved view.
pub fn in_view(d: &Deal, view: &str) -> bool {
    match view {
        "mine" => d.owner == PERSON_INI,
        "soon" => open_deal(d) && d.close <= 14,
        "won" => d.stage == "won",
        _ => true,
    }
}

/// How many deals a view holds.
pub fn view_count(list: &[Deal], view: &str) -> usize {
    list.iter().filter(|d| in_view(d, view)).count()
}

/// The deals in a view whose company, contact or note contains the query, sorted.
pub fn deal_view(list: &[Deal], view: &str, q: &str, by: DealSort, rev: bool) -> Vec<Deal> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Deal> = list
        .iter()
        .filter(|d| in_view(d, view))
        .filter(|d| {
            q.is_empty()
                || d.company.to_lowercase().contains(&q)
                || d.contact.to_lowercase().contains(&q)
                || d.note.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    let pos = |s: &str| CRM_STAGES.iter().position(|(k, _)| *k == s).unwrap_or(9);
    match by {
        DealSort::Company => v.sort_by_key(|d| d.company.to_lowercase()),
        DealSort::Stage => v.sort_by_key(|d| (pos(d.stage), d.company.to_lowercase())),
        DealSort::Value => v.sort_by_key(|d| std::cmp::Reverse(d.value)),
        DealSort::Close => v.sort_by_key(|d| d.close),
    }
    if rev {
        v.reverse();
    }
    v
}

/// Reads an amount typed into a value cell: `12400`, `12,400`, `$12,400`, `12.4k`, `3k`.
/// None when it is not an amount.
pub fn parse_money(s: &str) -> Option<u32> {
    let t: String = s
        .trim()
        .trim_start_matches('$')
        .chars()
        .filter(|c| *c != ',' && *c != ' ')
        .collect::<String>()
        .to_lowercase();
    let (num, mul) = match t.strip_suffix('k') {
        Some(n) => (n, 1000.0),
        None => (t.as_str(), 1.0),
    };
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let v: f64 = num.parse().ok()?;
    let v = (v * mul).round();
    if v > f64::from(u32::MAX) {
        return None;
    }
    Some(v as u32)
}

/// `$12,400`.
pub fn usd(v: u32) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    format!("${out}")
}

/// Total value of the deals still open (not won or lost).
pub fn pipeline(list: &[Deal]) -> u32 {
    list.iter().filter(|d| open_deal(d)).map(|d| d.value).sum()
}

/// Total value per stage, in pipeline order.
pub fn stage_totals(list: &[Deal]) -> Vec<(&'static str, u32, usize)> {
    CRM_STAGES
        .iter()
        .map(|(k, _)| {
            let v: Vec<&Deal> = list.iter().filter(|d| d.stage == *k).collect();
            (*k, v.iter().map(|d| d.value).sum(), v.len())
        })
        .collect()
}

/// Adds a lead owned by `owner` and returns its id.
pub fn add_deal(list: &mut Vec<Deal>, company: &str, owner: &'static str) -> Option<u32> {
    let company = company.trim();
    if company.is_empty() {
        return None;
    }
    let id = list.iter().map(|d| d.id).max().unwrap_or(0) + 1;
    list.push(Deal {
        id,
        company: company.to_string(),
        contact: "",
        stage: "lead",
        value: 0,
        owner,
        close: 30,
        note: "",
    });
    Some(id)
}

/// "Closes today", "Closes in 6d", "6d overdue"; won and lost deals say when they closed.
pub fn close_words(d: i32, stage: &str) -> String {
    if stage == "won" || stage == "lost" {
        return if d >= 0 {
            "Closed today".to_string()
        } else {
            format!("Closed {}d ago", -d)
        };
    }
    match d {
        0 => "Closes today".to_string(),
        1.. => format!("Closes in {d}d"),
        _ => format!("{}d overdue", -d),
    }
}

// ───────────────────────────────────────────────────────────────── Workflow ──

/// Canvas size of the workflow builder, in canvas units (px at 100 %).
pub const WF_W: i32 = 1240;
/// Canvas height.
pub const WF_H: i32 = 560;
/// A node's width on the canvas.
pub const WF_NODE_W: i32 = 184;
/// A node's height on the canvas.
pub const WF_NODE_H: i32 = 76;
/// Grid step nodes snap to (and arrow keys move by).
pub const WF_GRID: i32 = 20;
/// Zoom steps, percent.
pub const WF_ZOOMS: [u32; 6] = [50, 67, 80, 100, 125, 150];

/// Node kinds: key, label.
pub const WF_KINDS: [(&str, &str); 5] = [
    ("trigger", "Trigger"),
    ("agent", "Agent"),
    ("tool", "Tool"),
    ("logic", "Logic"),
    ("action", "Action"),
];

/// A palette group: label, kind and the nodes it offers (title, what the setting asks for).
pub type WfGroup = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

/// The palette, in order.
pub const WF_PALETTE: [WfGroup; 4] = [
    (
        "Triggers",
        "trigger",
        &[
            ("New member", "Which list"),
            ("On a schedule", "When"),
            ("Webhook", "Path"),
        ],
    ),
    (
        "Tools",
        "tool",
        &[
            ("AI agent", "Instructions"),
            ("HTTP request", "URL"),
            ("Read a file", "File types"),
        ],
    ),
    (
        "Logic",
        "logic",
        &[("Branch", "Condition"), ("Wait", "How long")],
    ),
    (
        "Actions",
        "action",
        &[
            ("Send email", "To"),
            ("Post a message", "Channel"),
            ("Save a row", "Table"),
        ],
    ),
];

/// A node on the workflow canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct WfNode {
    /// Stable id.
    pub id: u32,
    /// `trigger`, `agent`, `tool`, `logic` or `action`.
    pub kind: &'static str,
    /// Title shown on the card.
    pub title: String,
    /// What its one setting asks for ("URL", "To"…).
    pub ask: &'static str,
    /// The setting's value; empty means not configured.
    pub value: String,
    /// Left edge on the canvas.
    pub x: i32,
    /// Top edge on the canvas.
    pub y: i32,
}

impl WfNode {
    /// Ready to run: its setting is filled in.
    pub fn ready(&self) -> bool {
        !self.value.trim().is_empty()
    }
}

/// A connection: from node id to node id.
pub type WfEdge = (u32, u32);

/// The kind's label ("Trigger"…).
pub fn wf_kind_label(kind: &str) -> &'static str {
    WF_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, l)| *l)
        .unwrap_or("Step")
}

fn wn(
    id: u32,
    kind: &'static str,
    title: &str,
    ask: &'static str,
    value: &str,
    x: i32,
    y: i32,
) -> WfNode {
    WfNode {
        id,
        kind,
        title: title.to_string(),
        ask,
        value: value.to_string(),
        x,
        y,
    }
}

/// The sample workflow ("Member welcome"): nodes and connections; empty for a new one.
pub fn workflow(sample: bool) -> (Vec<WfNode>, Vec<WfEdge>) {
    if !sample {
        return (vec![], vec![]);
    }
    let n = vec![
        wn(
            1,
            "trigger",
            "New member",
            "Which list",
            "Studio members",
            40,
            220,
        ),
        wn(
            2,
            "agent",
            "Draft a welcome",
            "Instructions",
            "Warm, three short lines, mention the first session",
            280,
            220,
        ),
        wn(3, "logic", "Branch", "Condition", "Plan is Team", 520, 220),
        wn(4, "action", "Send email", "To", "The new member", 760, 120),
        wn(5, "action", "Post a message", "Channel", "", 760, 320),
        wn(6, "action", "Save a row", "Table", "Members log", 1000, 220),
    ];
    let e = vec![(1, 2), (2, 3), (3, 4), (3, 5), (4, 6), (5, 6)];
    (n, e)
}

/// Snaps a coordinate to the grid.
pub fn wf_snap(v: i32) -> i32 {
    ((v as f32 / WF_GRID as f32).round() as i32) * WF_GRID
}

/// The widest the canvas grows to the right.
pub const WF_MAX_W: i32 = 2400;

/// Keeps a node inside the canvas (it may grow to the right up to [`WF_MAX_W`]).
pub fn wf_clamp(x: i32, y: i32) -> (i32, i32) {
    (
        x.clamp(0, WF_MAX_W - WF_NODE_W),
        y.clamp(0, WF_H - WF_NODE_H),
    )
}

/// The canvas size for these nodes: at least [`WF_W`] wide, growing to fit the rightmost node.
pub fn wf_canvas(nodes: &[WfNode]) -> (i32, i32) {
    let maxx = nodes
        .iter()
        .map(|n| n.x + WF_NODE_W + 80)
        .max()
        .unwrap_or(0);
    (maxx.max(WF_W), WF_H)
}

/// Moves a node to (x, y), snapped and kept on the canvas. False when the id is unknown.
pub fn wf_place(list: &mut [WfNode], id: u32, x: i32, y: i32) -> bool {
    let Some(n) = list.iter_mut().find(|n| n.id == id) else {
        return false;
    };
    let (x, y) = wf_clamp(wf_snap(x), wf_snap(y));
    n.x = x;
    n.y = y;
    true
}

/// True when adding `a → b` would close a loop (or is a self-link).
pub fn wf_would_loop(edges: &[WfEdge], a: u32, b: u32) -> bool {
    if a == b {
        return true;
    }
    let mut stack = vec![b];
    let mut seen = vec![];
    while let Some(c) = stack.pop() {
        if c == a {
            return true;
        }
        if seen.contains(&c) {
            continue;
        }
        seen.push(c);
        stack.extend(edges.iter().filter(|(f, _)| *f == c).map(|(_, t)| *t));
    }
    false
}

/// Connects `a → b` unless it exists, loops or a trigger would get an input. True when added.
pub fn wf_connect(nodes: &[WfNode], edges: &mut Vec<WfEdge>, a: u32, b: u32) -> bool {
    let into_trigger = nodes.iter().any(|n| n.id == b && n.kind == "trigger");
    if into_trigger || edges.contains(&(a, b)) || wf_would_loop(edges, a, b) {
        return false;
    }
    edges.push((a, b));
    true
}

/// Adds a palette node. After `after`: one column right of it (lower if that spot is taken) and
/// connected from it; otherwise at the first free grid spot from the left. Returns the new id.
pub fn wf_add(
    nodes: &mut Vec<WfNode>,
    edges: &mut Vec<WfEdge>,
    kind: &'static str,
    title: &str,
    ask: &'static str,
    after: Option<u32>,
) -> u32 {
    let id = nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
    let taken = |nodes: &[WfNode], x: i32, y: i32| {
        nodes
            .iter()
            .any(|n| (n.x - x).abs() < WF_NODE_W && (n.y - y).abs() < WF_NODE_H)
    };
    let from = after.and_then(|a| nodes.iter().find(|n| n.id == a).cloned());
    let (mut x, mut y) = match &from {
        Some(f) => (f.x + WF_NODE_W + 56, f.y),
        None => (40, 40),
    };
    let mut tries = 0;
    while taken(nodes, x, y) && tries < 40 {
        y += WF_NODE_H + 24;
        if y > WF_H - WF_NODE_H {
            y = 40;
            x += WF_NODE_W + 56;
        }
        tries += 1;
    }
    let (x, y) = wf_clamp(wf_snap(x), wf_snap(y));
    nodes.push(WfNode {
        id,
        kind,
        title: title.to_string(),
        ask,
        value: String::new(),
        x,
        y,
    });
    if let Some(f) = from {
        let _ = wf_connect(nodes, edges, f.id, id);
    }
    id
}

/// Removes a node and its connections.
pub fn wf_remove(nodes: &mut Vec<WfNode>, edges: &mut Vec<WfEdge>, id: u32) {
    nodes.retain(|n| n.id != id);
    edges.retain(|(a, b)| *a != id && *b != id);
}

/// Run order: breadth first from the triggers (nodes no trigger reaches are left out).
pub fn wf_order(nodes: &[WfNode], edges: &[WfEdge]) -> Vec<u32> {
    let mut out: Vec<u32> = vec![];
    let mut q: Vec<u32> = nodes
        .iter()
        .filter(|n| n.kind == "trigger")
        .map(|n| n.id)
        .collect();
    while !q.is_empty() {
        let c = q.remove(0);
        if out.contains(&c) {
            continue;
        }
        // wait until every reachable input has run
        let ins: Vec<u32> = edges
            .iter()
            .filter(|(_, b)| *b == c)
            .map(|(a, _)| *a)
            .collect();
        let pending = ins.iter().any(|a| !out.contains(a) && q.contains(a));
        if pending {
            q.push(c);
            continue;
        }
        out.push(c);
        for (_, b) in edges.iter().filter(|(a, _)| *a == c) {
            if !out.contains(b) && !q.contains(b) {
                q.push(*b);
            }
        }
    }
    out
}

/// One node's state in a test run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WfRun {
    /// Not run (or no run yet).
    Idle,
    /// Waiting for its turn.
    Queued,
    /// Running now.
    Running,
    /// Finished fine.
    Done,
    /// Stopped here: not configured.
    Failed,
    /// Not run because a step before it failed.
    Skipped,
}

impl WfRun {
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            WfRun::Idle => "idle",
            WfRun::Queued => "queued",
            WfRun::Running => "running",
            WfRun::Done => "done",
            WfRun::Failed => "failed",
            WfRun::Skipped => "skipped",
        }
    }
    /// Words for the node card.
    pub fn label(self) -> &'static str {
        match self {
            WfRun::Idle => "",
            WfRun::Queued => "Queued",
            WfRun::Running => "Running",
            WfRun::Done => "Done",
            WfRun::Failed => "Failed",
            WfRun::Skipped => "Skipped",
        }
    }
}

/// Each node's state `tick` steps into a test run (one node per tick, in run order). A node that
/// is not configured fails when its turn comes and everything after it is skipped. `None` for
/// `tick` means no run.
pub fn wf_states(nodes: &[WfNode], edges: &[WfEdge], tick: Option<usize>) -> Vec<(u32, WfRun)> {
    let order = wf_order(nodes, edges);
    let Some(t) = tick else {
        return nodes.iter().map(|n| (n.id, WfRun::Idle)).collect();
    };
    let fail_at = order
        .iter()
        .position(|id| nodes.iter().any(|n| n.id == *id && !n.ready()));
    nodes
        .iter()
        .map(|n| {
            let st = match order.iter().position(|id| *id == n.id) {
                None => WfRun::Idle,
                Some(i) => match fail_at {
                    Some(f) if i > f => {
                        if t > f {
                            WfRun::Skipped
                        } else {
                            WfRun::Queued
                        }
                    }
                    Some(f) if i == f && t > f => WfRun::Failed,
                    _ if i < t => WfRun::Done,
                    _ if i == t => WfRun::Running,
                    _ => WfRun::Queued,
                },
            };
            (n.id, st)
        })
        .collect()
}

/// True once a run at `tick` has nothing left to do.
pub fn wf_finished(nodes: &[WfNode], edges: &[WfEdge], tick: usize) -> bool {
    wf_states(nodes, edges, Some(tick))
        .iter()
        .all(|(_, s)| !matches!(s, WfRun::Queued | WfRun::Running))
}

/// The run's outcome in words.
pub fn wf_outcome(nodes: &[WfNode], edges: &[WfEdge], tick: usize) -> String {
    let st = wf_states(nodes, edges, Some(tick));
    let done = st.iter().filter(|(_, s)| *s == WfRun::Done).count();
    let total = wf_order(nodes, edges).len();
    if let Some((id, _)) = st.iter().find(|(_, s)| *s == WfRun::Failed) {
        let t = nodes
            .iter()
            .find(|n| n.id == *id)
            .map(|n| n.title.clone())
            .unwrap_or_default();
        format!("Stopped at “{t}”: not configured")
    } else if wf_finished(nodes, edges, tick) {
        format!("All {total} steps ran")
    } else {
        format!("Running · {done} of {total}")
    }
}

/// The SVG path of a connection: a curve from the right middle of `a` to the left middle of `b`.
pub fn wf_path(a: &WfNode, b: &WfNode) -> String {
    let (x1, y1) = (a.x + WF_NODE_W, a.y + WF_NODE_H / 2);
    let (x2, y2) = (b.x, b.y + WF_NODE_H / 2);
    let d = ((x2 - x1).abs() / 2).max(40);
    format!("M{x1} {y1} C{} {y1} {} {y2} {x2} {y2}", x1 + d, x2 - d)
}

/// The next zoom step up (`dir` > 0) or down.
pub fn wf_zoom(cur: u32, dir: i32) -> u32 {
    if dir > 0 {
        *WF_ZOOMS
            .iter()
            .find(|z| **z > cur)
            .unwrap_or(&WF_ZOOMS[WF_ZOOMS.len() - 1])
    } else {
        *WF_ZOOMS
            .iter()
            .rev()
            .find(|z| **z < cur)
            .unwrap_or(&WF_ZOOMS[0])
    }
}

/// The zoom that fits every node into a viewport `vw` × `vh` px (from the zoom steps).
pub fn wf_fit(nodes: &[WfNode], vw: i32, vh: i32) -> u32 {
    if nodes.is_empty() {
        return 100;
    }
    let maxx = nodes.iter().map(|n| n.x + WF_NODE_W).max().unwrap_or(WF_W) + 40;
    let maxy = nodes.iter().map(|n| n.y + WF_NODE_H).max().unwrap_or(WF_H) + 40;
    let fit = ((vw * 100) / maxx).min((vh * 100) / maxy).max(0) as u32;
    *WF_ZOOMS
        .iter()
        .rev()
        .find(|z| **z <= fit)
        .unwrap_or(&WF_ZOOMS[0])
}

// ─────────────────────────────────────────────────────────────── Settings shell ──

/// The control a settings row carries.
#[derive(Clone, Debug, PartialEq)]
pub enum SetCtl {
    /// On / off.
    Switch(bool),
    /// One of a fixed list (index of the chosen one).
    Choice(&'static [&'static str], usize),
    /// Free text (with a character cap; 0 for none).
    Text(String, usize),
    /// A button that does something now ("Export").
    Action(&'static str),
    /// A destructive button that asks first ("Delete workspace").
    Danger(&'static str),
}

/// One row of a settings page.
#[derive(Clone, Debug, PartialEq)]
pub struct SetRow {
    /// Stable key, unique across pages.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// One line under the label.
    pub hint: &'static str,
    /// Its control.
    pub ctl: SetCtl,
    /// Key of a switch on the same page this row depends on ("" for none).
    pub needs: &'static str,
}

/// A settings page in the shell.
#[derive(Clone, Debug, PartialEq)]
pub struct SetPage {
    /// Stable key.
    pub key: &'static str,
    /// Nav label and heading.
    pub label: &'static str,
    /// `personal` or `workspace`.
    pub group: &'static str,
    /// Icon name.
    pub icon: &'static str,
    /// One line under the heading.
    pub lead: &'static str,
    /// Sections: heading and rows.
    pub sections: Vec<(&'static str, Vec<SetRow>)>,
}

/// The two nav groups: key, label.
pub const SET_GROUPS: [(&str, &str); 2] = [("personal", "Personal"), ("workspace", "Workspace")];

fn sr(key: &'static str, label: &'static str, hint: &'static str, ctl: SetCtl) -> SetRow {
    SetRow {
        key,
        label,
        hint,
        ctl,
        needs: "",
    }
}

fn sw(on: bool) -> SetCtl {
    SetCtl::Switch(on)
}

fn tx(v: &str, cap: usize) -> SetCtl {
    SetCtl::Text(v.to_string(), cap)
}

const DELIVERY: &[&str] = &["Email and phone", "Phone", "Email", "Inbox only"];

/// Every settings page. Sample has the fictional workspace filled in; a new one (`sample` false)
/// starts with blank text and the defaults.
pub fn settings(sample: bool) -> Vec<SetPage> {
    let v = |s: &str| if sample { s.to_string() } else { String::new() };
    let on = |b: bool| sample && b;
    let mut digest_at = sr(
        "digest-at",
        "Send at",
        "Your local time",
        SetCtl::Choice(&["07:00", "08:00", "09:00", "18:00"], 1),
    );
    digest_at.needs = "digest";
    let mut domain = sr(
        "join-domain",
        "Domain",
        "People with this address join as members",
        tx(&v("kestrel.example"), 0),
    );
    domain.needs = "join";
    vec![
        SetPage {
            key: "profile",
            label: "Account",
            group: "personal",
            icon: "home",
            lead: "How you appear to the people and agents you work with.",
            sections: vec![
                (
                    "About you",
                    vec![
                        sr(
                            "name",
                            "Name",
                            "Shown on your messages and runs",
                            tx(&v(PERSON), 60),
                        ),
                        sr(
                            "role",
                            "Role",
                            "Helps the assistant pitch its answers",
                            SetCtl::Choice(
                                &[
                                    "Design",
                                    "Engineering",
                                    "Operations",
                                    "Sales",
                                    "Something else",
                                ],
                                if sample { 0 } else { 4 },
                            ),
                        ),
                        sr(
                            "context",
                            "Context for the assistant",
                            "A few lines it reads before every chat",
                            tx(
                                &v("I run the studio's member programme; keep replies short."),
                                500,
                            ),
                        ),
                    ],
                ),
                (
                    "Time",
                    vec![
                        sr(
                            "week",
                            "Week starts on",
                            "Used by schedules and reports",
                            SetCtl::Choice(&["Monday", "Sunday"], 0),
                        ),
                        sr("clock", "24-hour clock", "", sw(true)),
                    ],
                ),
            ],
        },
        SetPage {
            key: "notifications",
            label: "Alerts",
            group: "personal",
            icon: "bell",
            lead: "What reaches you, where, and when it waits until morning.",
            sections: vec![
                (
                    "Digest",
                    vec![
                        sr(
                            "digest",
                            "Daily digest",
                            "One summary of what agents did",
                            sw(on(true)),
                        ),
                        digest_at,
                    ],
                ),
                (
                    "Tell me about",
                    vec![
                        sr(
                            "n-approvals",
                            "Approvals waiting",
                            "An agent needs a yes before it acts",
                            SetCtl::Choice(DELIVERY, 0),
                        ),
                        sr("n-done", "Finished runs", "", SetCtl::Choice(DELIVERY, 3)),
                        sr("n-failed", "Failed runs", "", SetCtl::Choice(DELIVERY, 2)),
                        sr(
                            "n-mentions",
                            "Mentions",
                            "Someone names you in a chat",
                            SetCtl::Choice(DELIVERY, 1),
                        ),
                    ],
                ),
                (
                    "Quiet hours",
                    vec![sr(
                        "quiet",
                        "Hold pushes at night",
                        "22:00 to 07:00; email still arrives",
                        sw(on(true)),
                    )],
                ),
            ],
        },
        SetPage {
            key: "preferences",
            label: "Chat defaults",
            group: "personal",
            icon: "gear",
            lead: "Defaults for new chats and how the app looks.",
            sections: vec![
                (
                    "Chats",
                    vec![
                        sr(
                            "model",
                            "Default model",
                            "Auto picks per message",
                            SetCtl::Choice(&["Auto", "Fast", "Deep"], 0),
                        ),
                        sr("web", "Web search", "Let answers look things up", sw(true)),
                        sr(
                            "memory",
                            "Memory",
                            "Keep what you tell it between chats",
                            sw(on(true)),
                        ),
                    ],
                ),
                (
                    "Look",
                    vec![
                        sr(
                            "theme",
                            "Appearance",
                            "",
                            SetCtl::Choice(&["System", "Light", "Dark"], 0),
                        ),
                        sr(
                            "dense",
                            "Compact lists",
                            "Fit more rows on screen",
                            sw(false),
                        ),
                    ],
                ),
            ],
        },
        SetPage {
            key: "privacy",
            label: "Privacy",
            group: "personal",
            icon: "shield",
            lead: "What is measured, what is kept, and getting your data out.",
            sections: vec![
                (
                    "Data",
                    vec![
                        sr(
                            "analytics",
                            "Product analytics",
                            "Anonymous usage that shapes what we build",
                            sw(on(true)),
                        ),
                        sr(
                            "train",
                            "Improve models with my chats",
                            "Off means your chats are never used to train",
                            sw(false),
                        ),
                    ],
                ),
                (
                    "Your data",
                    vec![
                        sr(
                            "export",
                            "Export my data",
                            "A download link arrives by email",
                            SetCtl::Action("Export"),
                        ),
                        sr(
                            "wipe-chats",
                            "Delete all chats",
                            "Removes every chat you started; cannot be undone",
                            SetCtl::Danger("Delete all chats"),
                        ),
                    ],
                ),
            ],
        },
        SetPage {
            key: "general",
            label: "Workspace",
            group: "workspace",
            icon: "home",
            lead: "The workspace's name, address and what every chat should know.",
            sections: vec![
                (
                    "Workspace",
                    vec![
                        sr("ws-name", "Name", "", tx(&v(WORKSPACE), 40)),
                        sr(
                            "ws-url",
                            "Address",
                            "Letters, numbers and dashes; old links stop working if you change it",
                            tx(&v("kestrel-studio"), 32),
                        ),
                        sr(
                            "ws-context",
                            "Company context",
                            "Added to every chat in the workspace",
                            tx(&v(""), 2000),
                        ),
                    ],
                ),
                (
                    "Danger zone",
                    vec![sr(
                        "delete-ws",
                        "Delete workspace",
                        "Removes the workspace, its chats, files and agents",
                        SetCtl::Danger("Delete workspace"),
                    )],
                ),
            ],
        },
        SetPage {
            key: "members",
            label: "People",
            group: "workspace",
            icon: "apps",
            lead: "Who can join and what they get by default.",
            sections: vec![
                (
                    "Joining",
                    vec![
                        sr("join", "Join by email domain", "", sw(on(true))),
                        domain,
                        sr(
                            "default-role",
                            "New members are",
                            "",
                            SetCtl::Choice(&["Members", "Admins", "Guests"], 0),
                        ),
                    ],
                ),
                (
                    "Invite",
                    vec![sr(
                        "invite-link",
                        "Invite link",
                        "Anyone with it can ask to join",
                        SetCtl::Action("Copy link"),
                    )],
                ),
            ],
        },
        SetPage {
            key: "models",
            label: "Models",
            group: "workspace",
            icon: "spark",
            lead: "Which models people and agents may use here.",
            sections: vec![
                (
                    "Defaults",
                    vec![
                        sr(
                            "ws-model",
                            "Chat model",
                            "",
                            SetCtl::Choice(&["Auto", "Fast", "Deep"], 0),
                        ),
                        sr(
                            "img-model",
                            "Image model",
                            "",
                            SetCtl::Choice(&["Standard", "Detailed"], 0),
                        ),
                    ],
                ),
                (
                    "Access",
                    vec![
                        sr(
                            "eu",
                            "Hosted in the EU only",
                            "Hides models served from elsewhere",
                            sw(false),
                        ),
                        sr(
                            "own-keys",
                            "Members bring their own keys",
                            "Usage is billed to their key",
                            sw(on(true)),
                        ),
                    ],
                ),
            ],
        },
        SetPage {
            key: "security",
            label: "Sign-in",
            group: "workspace",
            icon: "shield",
            lead: "How people sign in and how long they stay signed in.",
            sections: vec![
                (
                    "Sign-in",
                    vec![
                        sr(
                            "sso",
                            "Require single sign-on",
                            "Everyone signs in through your identity provider",
                            sw(on(true)),
                        ),
                        sr(
                            "2fa",
                            "Two-step verification",
                            "Ask for a code on new devices",
                            sw(false),
                        ),
                        sr(
                            "session",
                            "Sessions last",
                            "",
                            SetCtl::Choice(&["1 day", "7 days", "30 days"], 1),
                        ),
                    ],
                ),
                (
                    "Danger zone",
                    vec![sr(
                        "sign-out-all",
                        "Sign out everyone",
                        "Ends every session, including yours",
                        SetCtl::Danger("Sign out everyone"),
                    )],
                ),
            ],
        },
        SetPage {
            key: "billing",
            label: "Plan",
            group: "workspace",
            icon: "chart",
            lead: "The plan, who gets the invoices and a cap on agent spend.",
            sections: vec![
                (
                    "Plan",
                    vec![
                        sr(
                            "period",
                            "Billing period",
                            "Yearly saves two months",
                            SetCtl::Choice(&["Monthly", "Yearly"], 0),
                        ),
                        sr(
                            "bill-email",
                            "Invoices go to",
                            "",
                            tx(&v("accounts@kestrel.example"), 0),
                        ),
                        sr(
                            "cap",
                            "Monthly agent cap (USD)",
                            "Runs pause when it is reached",
                            tx(&v("500"), 0),
                        ),
                    ],
                ),
                (
                    "Invoices",
                    vec![sr(
                        "invoices",
                        "Past invoices",
                        "",
                        SetCtl::Action("Download all"),
                    )],
                ),
            ],
        },
    ]
}

/// The page a row lives on and the row itself.
pub fn set_row<'a>(pages: &'a [SetPage], key: &str) -> Option<(&'a SetPage, &'a SetRow)> {
    pages.iter().find_map(|p| {
        p.sections
            .iter()
            .flat_map(|(_, rs)| rs.iter())
            .find(|r| r.key == key)
            .map(|r| (p, r))
    })
}

/// Mutable access to a row by key.
pub fn set_row_mut<'a>(pages: &'a mut [SetPage], key: &str) -> Option<&'a mut SetRow> {
    pages
        .iter_mut()
        .flat_map(|p| p.sections.iter_mut())
        .flat_map(|(_, rs)| rs.iter_mut())
        .find(|r| r.key == key)
}

/// False when the row depends on a switch that is off.
pub fn set_enabled(pages: &[SetPage], row: &SetRow) -> bool {
    if row.needs.is_empty() {
        return true;
    }
    !matches!(
        set_row(pages, row.needs).map(|(_, r)| &r.ctl),
        Some(SetCtl::Switch(false))
    )
}

/// Row keys whose page, section, label or hint contains `q` (case-insensitive), with their page.
pub fn set_find(pages: &[SetPage], q: &str) -> Vec<(&'static str, &'static str)> {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return vec![];
    }
    let has = |s: &str| s.to_lowercase().contains(&q);
    let mut out = vec![];
    for p in pages {
        for (sec, rows) in &p.sections {
            for r in rows {
                if has(p.label) || has(sec) || has(r.label) || has(r.hint) {
                    out.push((p.key, r.key));
                }
            }
        }
    }
    out
}

/// How many rows differ between the saved pages and the edited ones.
pub fn set_changes(saved: &[SetPage], cur: &[SetPage]) -> usize {
    let rows = |ps: &[SetPage]| -> Vec<SetRow> {
        ps.iter()
            .flat_map(|p| p.sections.iter())
            .flat_map(|(_, rs)| rs.iter().cloned())
            .collect()
    };
    let (a, b) = (rows(saved), rows(cur));
    a.iter()
        .zip(b.iter())
        .filter(|(x, y)| x.ctl != y.ctl)
        .count()
}

/// What is wrong with a text value for this row, if anything.
pub fn set_text_error(key: &str, value: &str) -> Option<&'static str> {
    let v = value.trim();
    match key {
        "name" | "ws-name" if v.is_empty() => Some("Needs a name"),
        "ws-url" if v.is_empty() => Some("Needs an address"),
        "ws-url"
            if !v
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') =>
        {
            Some("Use lowercase letters, numbers and dashes")
        }
        "bill-email" if !v.is_empty() && invite_initials(v).is_none() => {
            Some("Not an email address")
        }
        "join-domain"
            if !v.is_empty() && (!v.contains('.') || v.contains('@') || v.contains(' ')) =>
        {
            Some("A domain looks like example.com")
        }
        "cap" if !v.is_empty() && v.parse::<u32>().is_err() => Some("A whole number of dollars"),
        _ => None,
    }
}

/// True when every text row on every page is valid (Save is allowed).
pub fn set_valid(pages: &[SetPage]) -> bool {
    pages
        .iter()
        .flat_map(|p| p.sections.iter())
        .flat_map(|(_, rs)| rs.iter())
        .all(|r| match &r.ctl {
            SetCtl::Text(v, cap) => {
                set_text_error(r.key, v).is_none() && (*cap == 0 || v.chars().count() <= *cap)
            }
            _ => true,
        })
}

/// A destructive row that needs the workspace name typed before it runs.
pub fn set_needs_name(key: &str) -> bool {
    key == "delete-ws"
}

#[cfg(test)]
mod tests {
    #[test]
    fn sidebar_geometry_and_account() {
        assert_eq!(SIDE_W, (272, 220, 380));
        assert!(SIDE_W.1 < SIDE_W.0 && SIDE_W.0 < SIDE_W.2);
        assert!(WORK_W.1 < WORK_W.0 && WORK_W.0 < WORK_W.2);
        assert_eq!((TOP_H, INSET), (48, 8));
        assert_eq!(
            EnvMode::ALL.map(|m| EnvMode::from_key(m.key())),
            EnvMode::ALL
        );
        let keys: Vec<_> = account_menu()
            .iter()
            .map(|i| i.0)
            .filter(|k| !k.is_empty())
            .collect();
        assert_eq!(keys.first(), Some(&"settings"));
        assert_eq!(keys.last(), Some(&"logout"));
        for k in [
            "usage",
            "upgrade",
            "language",
            "appearance",
            "shortcuts",
            "integrations",
            "download",
            "changelog",
            "feedback",
        ] {
            assert!(keys.contains(&k), "{k}");
        }
        let u = usage();
        assert_eq!((u.pct(), u.left(), u.near()), (78, 2_200, false));
        assert_eq!(thousands(7_800), "7,800");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(thousands(12), "12");
        assert_eq!(context_card(&[]).map(|c| c.0), Some("running"));
        assert_eq!(context_card(&["running"]).map(|c| c.0), Some("whatsnew"));
        assert_eq!(context_card(&["running", "whatsnew", "setup"]), None);
    }

    #[test]
    fn stage_is_16_by_10_and_fits() {
        assert_eq!(STAGE_W * 10, STAGE_H * 16);
        assert!((stage_scale(720.0, None) - 0.5).abs() < 1e-6);
        assert!((stage_scale(1440.0, Some(450.0)) - 0.5).abs() < 1e-6);
        assert_eq!(stage_scale(3000.0, None), 1.0);
        assert_eq!(
            STAGE_STATES.map(|(k, _)| ScreenState::from_key(k).key()),
            ["sample", "loading", "empty"]
        );
    }

    use super::*;

    #[test]
    fn states_and_layouts() {
        assert_eq!(ScreenState::from_key("Loading"), ScreenState::Loading);
        assert_eq!(ScreenState::from_key("x"), ScreenState::Sample);
        assert_eq!(ScreenState::Empty.key(), "empty");
        assert_eq!(ShellLayout::from_key("Flat"), ShellLayout::Flat);
        assert_eq!(ShellLayout::from_key("?"), ShellLayout::Inset);
    }

    #[test]
    fn threads_pin_and_read() {
        let mut t = threads();
        assert!(toggle_pin(&mut t, 2));
        let (p, rest) = split_pinned(&t);
        assert_eq!(p.len(), 2);
        assert_eq!(rest.len(), 2);
        assert!(!toggle_pin(&mut t, 2));
        assert!(!toggle_pin(&mut t, 99));
        open_thread(&mut t, 2);
        assert!(!t[1].unread);
    }

    #[test]
    fn titles_and_send() {
        assert_eq!(
            chat_title("  plan a two day offsite for nine people please"),
            "Plan a two day offsite for"
        );
        assert_eq!(chat_title(""), "New chat");
        let long = chat_title(&"x".repeat(60));
        assert_eq!(long.chars().count(), 40);
        assert!(long.ends_with('…'));
        assert_eq!(
            chat_title("ask about the twelve thousand nine hundred kilometre route"),
            "Ask about the twelve thousand nine"
        );
        assert!(can_send(" hi "));
        assert!(!can_send("  \n "));
    }

    #[test]
    fn small_formats() {
        assert_eq!(greeting(8), "Good morning");
        assert_eq!(greeting(19), "Good evening");
        assert_eq!(greeting(2), "Working late");
        assert_eq!(ago(0), "now");
        assert_eq!(ago(12), "12m");
        assert_eq!(ago(190), "3h");
        assert_eq!(ago(2900), "2d");
        assert_eq!(thousands(1284), "1,284");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(thousands(12), "12");
        assert_eq!(delta(12), "+12");
        assert_eq!(delta(-3), "−3");
        assert_eq!(used_pct(1284, 2000), 64);
        assert_eq!(used_pct(5, 0), 0);
        assert_eq!(pct_done(1, 8), 12);
        assert_eq!(clock(65), "1:05");
        assert_eq!(clock(3725), "1:02:05");
        assert_eq!(money(run_cents(24)), "$0.14");
    }

    #[test]
    fn heat_is_stable() {
        let h = heat(20);
        assert_eq!(h.len(), 140);
        assert!(h.iter().all(|&l| l <= 4));
        assert_eq!(h, heat(20));
        assert!(heat_total(&h) > 0);
    }

    #[test]
    fn sheet_sorts_numbers() {
        let mut s = sheet();
        sort_sheet(&mut s, 1, true);
        assert_eq!(s[0][0], "Fabric");
        assert_eq!(s[1][0], "Tencel blend");
        sort_sheet(&mut s, 2, false);
        assert_eq!(s[1][0], "Hemp twill");
        sort_sheet(&mut s, 0, true);
        assert_eq!(s[1][0], "Hemp twill");
        assert_eq!(cell_ref(1, 2), "B3");
        assert_eq!(col_name(3), 'D');
    }

    #[test]
    fn run_steps_and_review() {
        assert_eq!(step_state(0, 2), StepState::Done);
        assert_eq!(step_state(2, 2), StepState::Active);
        assert_eq!(step_state(4, 2), StepState::Pending);
        let mut p = run_plan();
        assert_eq!(move_step(&mut p, 0, true), 0);
        assert_eq!(move_step(&mut p, 0, false), 1);
        assert_eq!(p[1].tool, "ledger.query");
        assert_eq!(move_step(&mut p, 4, false), 4);
        let mut r = vec![Review::Open; 5];
        r[1] = Review::Accepted;
        assert_eq!(open_count(&r), 4);
        assert_eq!(next_open(&r, 0), Some(2));
        let all = vec![Review::Rejected; 3];
        assert_eq!(next_open(&all, 0), None);
        assert_eq!(ad_estimate(2, 3), (6, 2));
        assert_eq!(ad_estimate(0, 1), (0, 0));
    }

    #[test]
    fn folders_filter_sort_and_files() {
        let f = folders();
        let v = folder_view(&f, "", false);
        assert!(v[0].starred);
        assert!(v[1].mins <= v[2].mins);
        let n = folder_view(&f, "", true);
        assert_eq!(n[0].name, "Autumn launch");
        assert_eq!(n[1].name, "Brand kit");
        assert_eq!(folder_view(&f, "FABRIC", false).len(), 1);
        assert!(folder_view(&f, "zzz", false).is_empty());
        let mut files = folder_files();
        sort_files(&mut files, FileSort::Size, false);
        assert_eq!(files[0].name, "overshirt-hero.png");
        sort_files(&mut files, FileSort::Name, true);
        assert_eq!(files[0].name, "teaser-hooks.md");
        sort_files(&mut files, FileSort::Edited, false);
        assert_eq!(files[0].mins, 35);
        assert_eq!(size_label(6), "6 KB");
        assert_eq!(size_label(2380), "2.4 MB");
        assert_eq!(total_size(&folder_files()), "2.5 MB");
        let names = vec!["untitled.md".to_string(), "untitled-2.md".to_string()];
        assert_eq!(untitled(&names, "untitled", ".md"), "untitled-3.md");
        assert_eq!(untitled(&[], "untitled", ".md"), "untitled.md");
    }

    #[test]
    fn invites_need_an_address() {
        assert_eq!(
            invite_initials("ada.lin@studio.example").as_deref(),
            Some("AL")
        );
        assert_eq!(invite_initials(" kit@x.example ").as_deref(), Some("K"));
        assert_eq!(invite_initials("not an email"), None);
        assert_eq!(invite_initials("a@b"), None);
        assert_eq!(invite_initials("@b.example"), None);
        assert_eq!(member_name("TA"), "Tamsin Arvo");
        assert_eq!(member_name("ZZ"), "Invited guest");
    }

    #[test]
    fn apps_filter_and_count() {
        let a = apps(true);
        assert_eq!(connected_count(&a), 4);
        assert_eq!(connected_count(&apps(false)), 0);
        let all = app_view(&a, "all", "");
        assert_eq!(all.len(), 12);
        assert!(all[..4].iter().all(|x| x.connected));
        assert_eq!(all[0].name, "Google Calendar");
        assert_eq!(app_view(&a, "on", "").len(), 4);
        assert_eq!(app_view(&a, "chat", "").len(), 3);
        assert_eq!(app_view(&a, "all", "ISSUES").len(), 1);
        assert!(app_view(&apps(false), "on", "").is_empty());
        assert_eq!(synced_words(0), "Synced just now");
        assert_eq!(synced_words(12), "Synced 12m ago");
    }

    #[test]
    fn board_moves_and_limits() {
        let mut b = board(true);
        assert_eq!(column(&b, "todo", "", None).len(), 3);
        assert_eq!(column(&b, "doing", "", None)[0].id, 11);
        assert_eq!(column(&b, "todo", "tagline", None).len(), 1);
        assert_eq!(column(&b, "review", "ks-9", None).len(), 1);
        assert_eq!(column(&b, "todo", "", Some("JO")).len(), 1);
        assert!(!over_limit(&b, "doing"));
        assert!(move_task(&mut b, 14, "doing", 0));
        assert!(!over_limit(&b, "doing"));
        let d: Vec<u32> = column(&b, "doing", "", None).iter().map(|t| t.id).collect();
        assert_eq!(d, vec![14, 11, 12]);
        let t: Vec<u32> = column(&b, "todo", "", None).iter().map(|t| t.ord).collect();
        assert_eq!(t, vec![0, 1]);
        assert!(!move_task(&mut b, 14, "doing", 0));
        assert!(move_task(&mut b, 15, "doing", 9));
        assert!(over_limit(&b, "doing"));
        assert!(move_task(&mut b, 14, "doing", 9));
        assert_eq!(column(&b, "doing", "", None)[3].id, 14);
        assert!(!move_task(&mut b, 99, "done", 0));
        assert_eq!(next_col("todo", -1), None);
        assert_eq!(next_col("todo", 1), Some("doing"));
        assert_eq!(next_col("done", 1), None);
        assert_eq!(col_label("review"), "In review");
        assert_eq!(add_task(&mut b, "todo", "  "), None);
        assert_eq!(add_task(&mut b, "todo", "Write alt text"), Some(17));
        assert_eq!(column(&b, "todo", "", None).last().map(|t| t.id), Some(17));
        assert!(board(false).is_empty());
        assert_eq!(task_key(9), "KS-9");
        assert_eq!(due_words(0), "Due today");
        assert_eq!(due_words(3), "Due in 3d");
        assert_eq!(due_words(-1), "1d late");
    }

    #[test]
    fn crm_views_sort_and_edit() {
        let mut l = deals(true);
        assert_eq!(view_count(&l, "all"), 7);
        assert_eq!(view_count(&l, "mine"), 3);
        assert_eq!(view_count(&l, "soon"), 2);
        assert_eq!(view_count(&l, "won"), 1);
        let v = deal_view(&l, "all", "", DealSort::Value, false);
        assert_eq!(v[0].id, 7);
        let v = deal_view(&l, "all", "", DealSort::Value, true);
        assert_eq!(v[0].id, 3);
        let v = deal_view(&l, "all", "", DealSort::Company, false);
        assert_eq!(v[0].company, "Basalt Studio");
        let v = deal_view(&l, "all", "", DealSort::Stage, false);
        assert_eq!(v[0].stage, "lead");
        assert_eq!(v[6].stage, "lost");
        assert_eq!(
            deal_view(&l, "all", "security", DealSort::Close, false).len(),
            1
        );
        assert_eq!(
            deal_view(&l, "all", "dara", DealSort::Close, false)[0].id,
            2
        );
        assert_eq!(parse_money("12,400"), Some(12_400));
        assert_eq!(parse_money("$12400"), Some(12_400));
        assert_eq!(parse_money("12.4k"), Some(12_400));
        assert_eq!(parse_money("3K"), Some(3_000));
        assert_eq!(parse_money("lots"), None);
        assert_eq!(parse_money(""), None);
        assert_eq!(usd(1_234_567), "$1,234,567");
        assert_eq!(usd(800), "$800");
        assert_eq!(pipeline(&l), 24_000 + 8_400 + 3_200 + 15_000 + 31_500);
        assert_eq!(stage_totals(&l)[3], ("won", 12_600, 1));
        assert_eq!(add_deal(&mut l, " ", "RV"), None);
        assert_eq!(add_deal(&mut l, "Fernhall Co", "RV"), Some(8));
        assert_eq!(view_count(&l, "mine"), 4);
        assert_eq!(stage_label("proposal"), "Proposal");
        assert_eq!(stage_key("won"), Some("won"));
        assert_eq!(stage_key("x"), None);
        assert_eq!(close_words(6, "lead"), "Closes in 6d");
        assert_eq!(close_words(-2, "proposal"), "2d overdue");
        assert_eq!(close_words(-3, "won"), "Closed 3d ago");
        assert!(deals(false).is_empty());
    }

    #[test]
    fn workflow_edit_and_run() {
        let (mut n, mut e) = workflow(true);
        assert_eq!(wf_order(&n, &e), vec![1, 2, 3, 4, 5, 6]);
        assert!(!n[4].ready());
        // no run yet
        assert!(
            wf_states(&n, &e, None)
                .iter()
                .all(|(_, s)| *s == WfRun::Idle)
        );
        let s = wf_states(&n, &e, Some(1));
        assert_eq!(s[0].1, WfRun::Done);
        assert_eq!(s[1].1, WfRun::Running);
        assert_eq!(s[2].1, WfRun::Queued);
        // node 5 (index 4) is not configured: fails at its turn, 6 skipped
        let s = wf_states(&n, &e, Some(5));
        assert_eq!(s[3].1, WfRun::Done);
        assert_eq!(s[4].1, WfRun::Failed);
        assert_eq!(s[5].1, WfRun::Skipped);
        assert!(wf_finished(&n, &e, 5));
        assert_eq!(
            wf_outcome(&n, &e, 5),
            "Stopped at “Post a message”: not configured"
        );
        assert_eq!(wf_outcome(&n, &e, 2), "Running · 2 of 6");
        n[4].value = "#welcome".into();
        assert!(!wf_finished(&n, &e, 5));
        assert!(wf_finished(&n, &e, 6));
        assert_eq!(wf_outcome(&n, &e, 6), "All 6 steps ran");
        // connections
        assert!(wf_would_loop(&e, 6, 1));
        assert!(wf_would_loop(&e, 2, 2));
        assert!(!wf_connect(&n, &mut e, 6, 2));
        assert!(!wf_connect(&n, &mut e, 2, 1));
        assert!(!wf_connect(&n, &mut e, 1, 2));
        assert!(wf_connect(&n, &mut e, 2, 6));
        // add after a node: right of it, connected, configured later
        let id = wf_add(&mut n, &mut e, "tool", "HTTP request", "URL", Some(6));
        assert_eq!(id, 7);
        let m = n.iter().find(|x| x.id == 7).cloned().unwrap();
        assert_eq!(m.x, 1240);
        assert_eq!(wf_canvas(&n), (1240 + WF_NODE_W + 80, WF_H));
        assert!(e.contains(&(6, 7)));
        assert!(!m.ready());
        // a spot that is taken moves down
        let id2 = wf_add(&mut n, &mut e, "tool", "Wait", "How long", Some(6));
        let m2 = n.iter().find(|x| x.id == id2).unwrap();
        assert!(m2.y > m.y);
        wf_remove(&mut n, &mut e, 7);
        assert!(!e.iter().any(|(a, b)| *a == 7 || *b == 7));
        // place, snap and clamp
        assert!(wf_place(&mut n, 1, 51, 9999));
        assert_eq!((n[0].x, n[0].y), (60, WF_H - WF_NODE_H));
        assert!(wf_place(&mut n, 1, -30, 33));
        assert_eq!((n[0].x, n[0].y), (0, 40));
        assert!(!wf_place(&mut n, 99, 0, 0));
        assert_eq!(wf_snap(29), 20);
        assert_eq!(wf_snap(31), 40);
        // zoom
        assert_eq!(wf_zoom(100, 1), 125);
        assert_eq!(wf_zoom(100, -1), 80);
        assert_eq!(wf_zoom(150, 1), 150);
        assert_eq!(wf_zoom(50, -1), 50);
        assert_eq!(wf_zoom(90, -1), 80);
        assert_eq!(wf_fit(&workflow(true).0, 700, 500), 50);
        assert_eq!(wf_fit(&workflow(true).0, 1400, 700), 100);
        assert_eq!(wf_fit(&[], 10, 10), 100);
        assert!(wf_path(&n[1], &n[2]).starts_with("M"));
        assert_eq!(wf_kind_label("logic"), "Logic");
        assert!(workflow(false).0.is_empty());
        // empty canvas: first node goes top left
        let (mut n0, mut e0) = (vec![], vec![]);
        assert_eq!(
            wf_add(&mut n0, &mut e0, "trigger", "Webhook", "Path", None),
            1
        );
        assert_eq!((n0[0].x, n0[0].y), (40, 40));
    }

    #[test]
    fn settings_find_change_and_check() {
        let saved = settings(true);
        let mut cur = saved.clone();
        assert_eq!(saved.len(), 9);
        assert_eq!(saved.iter().filter(|p| p.group == "personal").count(), 4);
        let mut keys: Vec<&str> = saved
            .iter()
            .flat_map(|p| p.sections.iter())
            .flat_map(|(_, r)| r.iter().map(|r| r.key))
            .collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n, "row keys are unique");
        assert_eq!(set_changes(&saved, &cur), 0);
        // a switch the "Send at" row depends on
        let (_, at) = set_row(&cur, "digest-at").unwrap();
        assert!(set_enabled(&cur, &at.clone()));
        if let Some(r) = set_row_mut(&mut cur, "digest") {
            r.ctl = SetCtl::Switch(false);
        }
        let at = set_row(&cur, "digest-at").unwrap().1.clone();
        assert!(!set_enabled(&cur, &at));
        assert_eq!(set_changes(&saved, &cur), 1);
        if let Some(r) = set_row_mut(&mut cur, "theme") {
            r.ctl = SetCtl::Choice(&["System", "Light", "Dark"], 2);
        }
        assert_eq!(set_changes(&saved, &cur), 2);
        // search: label, hint and page names
        let f = set_find(&saved, "digest");
        assert_eq!(f[0], ("notifications", "digest"));
        assert!(
            set_find(&saved, "SINGLE sign")
                .iter()
                .any(|(p, r)| *p == "security" && *r == "sso")
        );
        assert!(set_find(&saved, "plan").len() >= 4);
        assert!(set_find(&saved, "  ").is_empty());
        assert!(set_find(&saved, "zzz").is_empty());
        // text checks
        assert_eq!(
            set_text_error("ws-url", "Kestrel Studio"),
            Some("Use lowercase letters, numbers and dashes")
        );
        assert_eq!(set_text_error("ws-url", "kestrel-2"), None);
        assert_eq!(set_text_error("name", " "), Some("Needs a name"));
        assert_eq!(
            set_text_error("bill-email", "accounts"),
            Some("Not an email address")
        );
        assert_eq!(set_text_error("bill-email", "a@b.example"), None);
        assert_eq!(
            set_text_error("join-domain", "kestrel"),
            Some("A domain looks like example.com")
        );
        assert_eq!(
            set_text_error("cap", "12.5"),
            Some("A whole number of dollars")
        );
        assert_eq!(set_text_error("cap", ""), None);
        assert!(set_valid(&saved));
        if let Some(r) = set_row_mut(&mut cur, "ws-url") {
            r.ctl = SetCtl::Text("No Spaces".into(), 32);
        }
        assert!(!set_valid(&cur));
        assert!(set_needs_name("delete-ws"));
        assert!(!set_needs_name("wipe-chats"));
        // a new workspace starts blank
        let blank = settings(false);
        assert!(
            matches!(&set_row(&blank, "ws-name").unwrap().1.ctl, SetCtl::Text(v, _) if v.is_empty())
        );
        assert!(matches!(
            set_row(&blank, "memory").unwrap().1.ctl,
            SetCtl::Switch(false)
        ));
    }
}
