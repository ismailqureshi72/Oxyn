//! Screens 1 (whole app frames): the shared app frame (Inset or Flat, plus a three-pane frame with
//! an icon rail), the AI Chat screens built on it – Start, Overview, Agents, Double sidebar,
//! Progress and Run – then Folders (Overview, Detail), Integrations and Kanban. Each screen shows
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

/// The fictional workspace every screen opens on.
pub const WORKSPACE: &str = "Kestrel Studio";
/// The fictional signed-in person.
pub const PERSON: &str = "Rowan Vale";
/// Their initials.
pub const PERSON_INI: &str = "RV";

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

#[cfg(test)]
mod tests {
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
}
