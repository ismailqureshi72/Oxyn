//! Home and run surfaces from the X-post references (plan #28, chunk 6): a status-grouped thread
//! list (T3 Code, Threadbear), the activity heat map and overview home (affectUI-style overview
//! from @flohoeller, Threadbear activity), a token usage chart (@pitus_), an attachment strip
//! whose uploads land one after another (@sitenley for componest), run modes with a time budget
//! and an experiment grid (Cloudy), a document grid (quiet.md) and background polling cost
//! (@TekonBurns). Behaviour-only rebuilds: nothing is copied.

use crate::ClockMs;

// ---------------------------------------------------------------------------------------------
// Numbers
// ---------------------------------------------------------------------------------------------

/// Compact count with a thin unit: `471.83 M`, `518.7 K`, `950`.
pub fn compact(n: f64) -> String {
    let a = n.abs();
    if a >= 1e9 {
        format!("{:.2} B", n / 1e9)
    } else if a >= 1e6 {
        format!("{:.2} M", n / 1e6)
    } else if a >= 1e3 {
        format!("{:.1} K", n / 1e3)
    } else {
        format!("{}", n.round() as i64)
    }
}

/// Whole number with thousands commas: `1,284`.
pub fn grouped(n: u64) -> String {
    let s = n.to_string();
    let mut g = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            g.push(',');
        }
        g.push(c);
    }
    g
}

/// Money from cents with grouping: `$1,658.61`.
pub fn dollars(cents: u64) -> String {
    format!("${}.{:02}", grouped(cents / 100), cents % 100)
}

/// "36 +12" style delta text: `+12`, `−3`, `0`.
pub fn delta(d: i64) -> String {
    match d.cmp(&0) {
        std::cmp::Ordering::Greater => format!("+{d}"),
        std::cmp::Ordering::Less => format!("\u{2212}{}", -d),
        std::cmp::Ordering::Equal => "0".into(),
    }
}

/// Ago text for a number of seconds: `now`, `8m`, `2h`, `3d`.
pub fn ago(secs: u64) -> String {
    match secs {
        0..=44 => "now".into(),
        45..=3599 => format!("{}m", (secs + 30) / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

// ---------------------------------------------------------------------------------------------
// Thread list (T3 Code, Threadbear)
// ---------------------------------------------------------------------------------------------

/// Where a thread stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState {
    /// The agent waits on an approval.
    Approval,
    /// The agent asked a question.
    Input,
    /// Running for `secs`.
    Working(u64),
    /// Finished and not yet seen.
    Done,
    /// Idle; last activity `secs` ago.
    Idle(u64),
    /// Seen and finished long ago (collapsed group).
    Settled,
}

impl ThreadState {
    /// Chip label, or `None` when the row shows its age instead.
    pub fn chip(self) -> Option<String> {
        match self {
            Self::Approval => Some("Approval".into()),
            Self::Input => Some("Input".into()),
            Self::Working(s) => Some(format!("Working {}", ago(s).replace("now", "0m"))),
            Self::Done => Some("Done".into()),
            Self::Idle(_) | Self::Settled => None,
        }
    }
    /// Stable id for styling.
    pub fn id(self) -> &'static str {
        match self {
            Self::Approval => "approval",
            Self::Input => "input",
            Self::Working(_) => "working",
            Self::Done => "done",
            Self::Idle(_) => "idle",
            Self::Settled => "settled",
        }
    }
    /// Icon name for the chip.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Approval => "hand",
            Self::Input => "thread",
            Self::Working(_) => "retry",
            Self::Done => "check",
            Self::Idle(_) | Self::Settled => "clock",
        }
    }
    /// Sort weight inside the top group: needs-you first.
    fn weight(self) -> u8 {
        match self {
            Self::Approval => 0,
            Self::Input => 1,
            Self::Done => 2,
            _ => 3,
        }
    }
    /// Seconds since the last activity, for ordering.
    fn age(self) -> u64 {
        match self {
            Self::Idle(s) => s,
            _ => 0,
        }
    }
}

/// One thread row.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    /// Id.
    pub id: String,
    /// Project ("orbit-web").
    pub project: String,
    /// Title.
    pub title: String,
    /// State.
    pub state: ThreadState,
    /// Agent / model mark (icon name).
    pub agent: String,
}

impl Thread {
    /// A thread.
    pub fn new(id: &str, project: &str, title: &str, state: ThreadState) -> Self {
        Self {
            id: id.into(),
            project: project.into(),
            title: title.into(),
            state,
            agent: "spark".into(),
        }
    }
    /// Builder: agent mark.
    pub fn agent(mut self, a: &str) -> Self {
        self.agent = a.into();
        self
    }
    /// Right-hand meta: the chip text or the age.
    pub fn meta(&self) -> String {
        match self.state {
            ThreadState::Idle(s) => ago(s),
            s => s.chip().unwrap_or_default(),
        }
    }
}

/// Project initials for the square badge: `orbit-web` → `OW`, `atlas` → `AT`.
pub fn initials(project: &str) -> String {
    let parts: Vec<&str> = project
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty())
        .collect();
    let s: String = if parts.len() >= 2 {
        parts
            .iter()
            .take(2)
            .filter_map(|p| p.chars().next())
            .collect()
    } else {
        parts
            .first()
            .map(|p| p.chars().take(2).collect())
            .unwrap_or_default()
    };
    s.to_uppercase()
}

/// A group of the list.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadGroup {
    /// `top`, `working` or `settled`.
    pub id: &'static str,
    /// Heading (empty for the top group).
    pub label: &'static str,
    /// Row indices into the input, in display order.
    pub rows: Vec<usize>,
}

/// Groups threads the way T3 Code does: the top list puts what needs you first (Approval, then
/// Input, then unseen Done), then the rest newest first; Working and Settled sit in their own
/// collapsible groups underneath. Empty groups are dropped. A `filter` (lowercase substring of
/// the title or project) narrows every group.
pub fn group_threads(threads: &[Thread], filter: &str) -> Vec<ThreadGroup> {
    let f = filter.trim().to_lowercase();
    let keep = |t: &Thread| {
        f.is_empty() || t.title.to_lowercase().contains(&f) || t.project.to_lowercase().contains(&f)
    };
    let mut top: Vec<usize> = Vec::new();
    let mut working: Vec<usize> = Vec::new();
    let mut settled: Vec<usize> = Vec::new();
    for (i, t) in threads.iter().enumerate().filter(|(_, t)| keep(t)) {
        match t.state {
            ThreadState::Working(_) => working.push(i),
            ThreadState::Settled => settled.push(i),
            _ => top.push(i),
        }
    }
    top.sort_by_key(|&i| (threads[i].state.weight(), threads[i].state.age(), i));
    working.sort_by_key(|&i| match threads[i].state {
        ThreadState::Working(s) => (s, i),
        _ => (0, i),
    });
    let mut out = Vec::new();
    for (id, label, rows) in [
        ("top", "", top),
        ("working", "Working", working),
        ("settled", "Settled", settled),
    ] {
        if !rows.is_empty() {
            out.push(ThreadGroup { id, label, rows });
        }
    }
    out
}

/// "2 need you" summary for the list header, or `None`.
pub fn needs_you(threads: &[Thread]) -> Option<String> {
    let n = threads
        .iter()
        .filter(|t| matches!(t.state, ThreadState::Approval | ThreadState::Input))
        .count();
    (n > 0).then(|| format!("{n} need{} you", if n == 1 { "s" } else { "" }))
}

// ---------------------------------------------------------------------------------------------
// Calendar helpers and the activity heat map
// ---------------------------------------------------------------------------------------------

/// Days in `year`.
pub fn year_days(year: i32) -> u32 {
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        366
    } else {
        365
    }
}

/// Weekday of Jan 1 (0 = Sunday).
pub fn jan1_weekday(year: i32) -> u32 {
    // Zeller-style count from 1 Jan 1 (a Monday in the proleptic Gregorian calendar).
    let y = (year - 1) as i64;
    let days = y * 365 + y / 4 - y / 100 + y / 400;
    ((days + 1) % 7) as u32
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// Month short name (0-based).
pub fn month_name(m: usize) -> &'static str {
    MONTHS[m % 12]
}

/// `(month 0-based, day 1-based)` for a 0-based day of `year`.
pub fn month_day(year: i32, doy: u32) -> (usize, u32) {
    let feb = if year_days(year) == 366 { 29 } else { 28 };
    let lens = [31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut d = doy;
    for (m, l) in lens.iter().enumerate() {
        if d < *l {
            return (m, d + 1);
        }
        d -= l;
    }
    (11, 31)
}

/// "Mon, Oct 5" for a 0-based day of `year`.
pub fn day_label(year: i32, doy: u32) -> String {
    let (m, d) = month_day(year, doy);
    let wd = (jan1_weekday(year) + doy) % 7;
    format!("{}, {} {d}", WEEKDAYS[wd as usize], MONTHS[m])
}

/// One cell of the heat map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeatCell {
    /// Day of the year, 0-based.
    pub doy: u32,
    /// Week column.
    pub col: u32,
    /// Weekday row (0 = Sunday).
    pub row: u32,
    /// Count that day.
    pub count: u32,
    /// Level 0–4.
    pub level: u8,
}

/// Levels 0–4: zero stays 0; the rest split at the quartiles of the non-zero days, so one busy
/// day never washes the year out.
pub fn heat_levels(counts: &[u32]) -> Vec<u8> {
    let mut nz: Vec<u32> = counts.iter().copied().filter(|&c| c > 0).collect();
    nz.sort_unstable();
    if nz.is_empty() {
        return vec![0; counts.len()];
    }
    let q = |f: f32| nz[((nz.len() - 1) as f32 * f).round() as usize];
    let (a, b, c) = (q(0.25), q(0.5), q(0.75));
    counts
        .iter()
        .map(|&n| match n {
            0 => 0,
            n if n <= a => 1,
            n if n <= b => 2,
            n if n <= c => 3,
            _ => 4,
        })
        .collect()
}

/// A year as week columns (Sunday rows first), GitHub style. `counts` is per day of the year
/// (missing days count 0).
pub fn heat_year(year: i32, counts: &[u32]) -> Vec<HeatCell> {
    let n = year_days(year);
    let all: Vec<u32> = (0..n as usize)
        .map(|i| counts.get(i).copied().unwrap_or(0))
        .collect();
    let lv = heat_levels(&all);
    let off = jan1_weekday(year);
    (0..n)
        .map(|doy| HeatCell {
            doy,
            col: (doy + off) / 7,
            row: (doy + off) % 7,
            count: all[doy as usize],
            level: lv[doy as usize],
        })
        .collect()
}

/// Week columns where each month label starts.
pub fn month_cols(year: i32) -> Vec<(u32, &'static str)> {
    let off = jan1_weekday(year);
    let mut out = Vec::new();
    let mut doy = 0;
    for (m, name) in MONTHS.iter().enumerate() {
        out.push(((doy + off) / 7, *name));
        doy += match m {
            1 => year_days(year) - 337,
            3 | 5 | 8 | 10 => 30,
            _ => 31,
        };
    }
    out
}

/// Tooltip text: "5 chats · Mon, Oct 5", "No chats · …", "1 chat · …".
pub fn heat_tip(count: u32, noun: &str, year: i32, doy: u32) -> String {
    let what = match count {
        0 => format!("No {noun}s"),
        1 => format!("1 {noun}"),
        n => format!("{n} {noun}s"),
    };
    format!("{what} · {}", day_label(year, doy))
}

/// A compact 12-week strip (affectUI overview card): the last `weeks` columns ending at `today`.
pub fn heat_recent(counts: &[u32], weeks: u32) -> Vec<(u32, u32, u8)> {
    let n = (weeks * 7) as usize;
    let start = counts.len().saturating_sub(n);
    let slice = &counts[start..];
    let lv = heat_levels(slice);
    (0..slice.len())
        .map(|i| ((i / 7) as u32, (i % 7) as u32, lv[i]))
        .collect()
}

/// Deterministic demo counts for a year (a few quiet months, then busy weeks).
pub fn demo_counts(seed: u32, days: u32, busy_from: u32) -> Vec<u32> {
    let mut x = seed.wrapping_mul(2_654_435_761).max(1);
    (0..days)
        .map(|d| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let r = x % 100;
            if d < busy_from {
                u32::from(r > 92)
            } else if d.is_multiple_of(7) || d % 7 == 6 {
                u32::from(r > 60) * (r % 3)
            } else {
                (r % 9) * u32::from(r > 18)
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Token usage chart (@pitus_)
// ---------------------------------------------------------------------------------------------

/// One day of token use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UsageDay {
    /// Total tokens.
    pub tokens: f64,
    /// Of those, served from cache.
    pub cached: f64,
}

/// Totals under the chart.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageTotals {
    /// "471.83 M".
    pub total: String,
    /// "518.7 K" average a day.
    pub avg: String,
    /// "$1,658.61" estimated.
    pub cost: String,
}

/// Totals for a window: uncached tokens cost `per_m_cents` a million, cached a tenth of that.
pub fn usage_totals(days: &[UsageDay], per_m_cents: f64) -> UsageTotals {
    let total: f64 = days.iter().map(|d| d.tokens).sum();
    let cached: f64 = days.iter().map(|d| d.cached.min(d.tokens)).sum();
    let avg = if days.is_empty() {
        0.0
    } else {
        total / days.len() as f64
    };
    let cents = ((total - cached) / 1e6 * per_m_cents + cached / 1e6 * per_m_cents * 0.1).round();
    UsageTotals {
        total: compact(total),
        avg: compact(avg),
        cost: dollars(cents.max(0.0) as u64),
    }
}

/// SVG path (`M … L …`) for a series scaled into `w × h`, with `max` at the top. `area` closes
/// it to the baseline.
pub fn spark_path(vals: &[f64], max: f64, w: f32, h: f32, area: bool) -> String {
    if vals.is_empty() || max <= 0.0 {
        return String::new();
    }
    let step = if vals.len() > 1 {
        w / (vals.len() - 1) as f32
    } else {
        0.0
    };
    let mut s = String::new();
    for (i, v) in vals.iter().enumerate() {
        let x = i as f32 * step;
        let y = h - (v / max).clamp(0.0, 1.0) as f32 * h;
        s.push_str(&format!("{}{x:.1} {y:.1} ", if i == 0 { "M" } else { "L" }));
    }
    if area {
        s.push_str(&format!("L{w:.1} {h:.1} L0 {h:.1} Z"));
    }
    s.trim_end().to_string()
}

/// Index under a pointer at fraction `x` (0..1) of the plot width.
pub fn hover_index(len: usize, x: f32) -> Option<usize> {
    (len > 0).then(|| ((x.clamp(0.0, 1.0) * (len - 1) as f32).round() as usize).min(len - 1))
}

/// Deterministic demo series: a slow rise with weekly bumps and two spikes near the end.
pub fn demo_usage(days: usize) -> Vec<UsageDay> {
    (0..days)
        .map(|i| {
            let t = i as f64 / days.max(1) as f64;
            let wave = ((i as f64) * 0.9).sin() * 0.25 + ((i as f64) * 0.23).cos() * 0.2;
            let spike = if i + 9 == days || i + 4 == days {
                2.6
            } else {
                0.0
            };
            let tokens = (2.0 + t * 4.5 + wave * 2.0 + spike * 3.0).max(0.4) * 1e6;
            UsageDay {
                tokens,
                cached: tokens * (0.25 + 0.2 * ((i as f64) * 0.5).sin().abs()),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Attachment strip uploads (@sitenley for componest)
// ---------------------------------------------------------------------------------------------

/// One tile's upload as seen at `now`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileUpload {
    /// Not started yet (queued behind earlier tiles).
    pub queued: bool,
    /// Progress 0..1 (the ring sweeps clockwise to it).
    pub progress: f32,
    /// Whole percent shown in the corner.
    pub pct: u32,
    /// The percent is blurring out (0..1 of the fade), after reaching 100.
    pub fade: f32,
    /// Finished and the percent has gone.
    pub done: bool,
}

/// Uploads run one after another: tile `i` starts when tile `i − 1` finishes, each takes
/// `dur_ms` scaled by its `weight` (bigger files take longer). After 100 the percent blurs out over
/// `fade_ms`. `added` is when the batch was dropped.
pub fn tile_upload(
    weights: &[f32],
    i: usize,
    added: ClockMs,
    now: ClockMs,
    dur_ms: u64,
    fade_ms: u64,
) -> TileUpload {
    let dur = |w: f32| (dur_ms as f32 * w.max(0.1)) as u64;
    let start: u64 = weights.iter().take(i).map(|w| dur(*w)).sum();
    let own = dur(weights.get(i).copied().unwrap_or(1.0)).max(1);
    let el = now.0.saturating_sub(added.0);
    if el < start {
        return TileUpload {
            queued: true,
            progress: 0.0,
            pct: 0,
            fade: 0.0,
            done: false,
        };
    }
    let t = (el - start) as f32 / own as f32;
    if t < 1.0 {
        // ease-out: fast start, slow finish, like a real upload
        let p = 1.0 - (1.0 - t).powf(1.8);
        return TileUpload {
            queued: false,
            progress: p,
            pct: (p * 100.0).floor() as u32,
            fade: 0.0,
            done: false,
        };
    }
    let f = ((el - start - own) as f32 / fade_ms.max(1) as f32).min(1.0);
    TileUpload {
        queued: false,
        progress: 1.0,
        pct: 100,
        fade: f,
        done: f >= 1.0,
    }
}

/// Icon name for a file name (image files show a thumbnail instead).
pub fn file_icon(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" => "image",
        "csv" | "xlsx" | "xls" | "numbers" => "table",
        "mp4" | "mov" | "webm" => "video",
        "mp3" | "wav" | "m4a" => "audio",
        "ts" | "tsx" | "rs" | "js" | "py" | "json" => "code",
        "zip" | "tar" | "gz" => "archive",
        "pdf" | "key" | "ppt" | "pptx" => "page",
        _ => "file",
    }
}

/// Truncates a name for a 56 px tile: `payrollz…` keeps 8 characters.
pub fn tile_name(name: &str, keep: usize) -> String {
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    if stem.chars().count() <= keep {
        stem.to_string()
    } else {
        format!("{}…", stem.chars().take(keep).collect::<String>())
    }
}

// ---------------------------------------------------------------------------------------------
// Run modes, time budget and the experiment grid (Cloudy)
// ---------------------------------------------------------------------------------------------

/// How much the agent may do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    /// Answer questions, change nothing.
    Ask,
    /// Propose first, run nothing.
    Plan,
    /// Talk and run code.
    Work,
    /// Works on its own toward a goal, inside a time budget.
    Auto,
}

impl RunMode {
    /// All, menu order.
    pub const ALL: [RunMode; 4] = [Self::Ask, Self::Plan, Self::Work, Self::Auto];
    /// Name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Ask",
            Self::Plan => "Plan",
            Self::Work => "Work",
            Self::Auto => "Autoresearch",
        }
    }
    /// One line under the name.
    pub fn blurb(self) -> &'static str {
        match self {
            Self::Ask => "Answer questions, change nothing",
            Self::Plan => "Propose first, run nothing",
            Self::Work => "Talk and run code",
            Self::Auto => "Works on its own toward a goal",
        }
    }
    /// Shape mark (each mode its own shape, so it reads without colour).
    pub fn shape(self) -> &'static str {
        match self {
            Self::Ask => "circle",
            Self::Plan => "triangle",
            Self::Work => "diamond",
            Self::Auto => "square",
        }
    }
    /// Stable id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Plan => "plan",
            Self::Work => "work",
            Self::Auto => "auto",
        }
    }
    /// From an id (default Work).
    pub fn from_id(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.id() == s)
            .unwrap_or(Self::Work)
    }
    /// Only Autoresearch takes a time budget.
    pub fn budgeted(self) -> bool {
        self == Self::Auto
    }
}

/// Budget presets in minutes.
pub const BUDGETS: [u32; 4] = [60, 300, 600, 1440];

/// `1h`, `24h`, `90m`, `1h 30m`.
pub fn hm(mins: u32) -> String {
    match (mins / 60, mins % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

/// Parses the custom budget field (minutes, clamped 5 … 10 080).
pub fn parse_budget(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok().map(|m| m.clamp(5, 10_080))
}

/// "2h 12m of 24h", with the fraction used.
pub fn budget_line(elapsed_secs: u64, budget_mins: u32) -> (String, f32) {
    let used = (elapsed_secs / 60) as u32;
    let f = used as f32 / budget_mins.max(1) as f32;
    (
        format!("{} of {}", hm(used.min(budget_mins)), hm(budget_mins)),
        f.min(1.0),
    )
}

/// What one experiment did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Inside the noise; reverted.
    NoGain,
    /// Kept.
    Kept,
    /// Crashed (then fixed).
    Crash,
    /// A summary point.
    Summary,
    /// You steered here.
    You,
}

impl Outcome {
    /// All, legend order.
    pub const ALL: [Outcome; 5] = [
        Self::NoGain,
        Self::Kept,
        Self::Crash,
        Self::Summary,
        Self::You,
    ];
    /// Legend label.
    pub fn label(self) -> &'static str {
        match self {
            Self::NoGain => "No gain",
            Self::Kept => "Kept",
            Self::Crash => "Crash, fixed",
            Self::Summary => "Summary",
            Self::You => "You",
        }
    }
    /// Shape mark.
    pub fn shape(self) -> &'static str {
        match self {
            Self::NoGain => "square-o",
            Self::Kept => "square",
            Self::Crash => "triangle",
            Self::Summary => "diamond",
            Self::You => "circle",
        }
    }
    /// Stable id.
    pub fn id(self) -> &'static str {
        match self {
            Self::NoGain => "none",
            Self::Kept => "kept",
            Self::Crash => "crash",
            Self::Summary => "summary",
            Self::You => "you",
        }
    }
}

/// One experiment.
#[derive(Clone, Debug, PartialEq)]
pub struct Experiment {
    /// Id ("P0002-E007").
    pub id: String,
    /// Start, seconds from the run start.
    pub at: u64,
    /// Outcome.
    pub outcome: Outcome,
    /// One line ("packed-register MXFP4 unpack: +0.7% at 120b").
    pub note: String,
}

/// Grid position of each experiment: rows are `band_h`-hour bands of the budget, columns the
/// order inside the band.
pub fn experiment_grid(exps: &[Experiment], band_h: u32) -> Vec<(usize, u32, u32)> {
    let band = u64::from(band_h.max(1)) * 3600;
    let mut per: Vec<u32> = Vec::new();
    exps.iter()
        .enumerate()
        .map(|(i, e)| {
            let row = (e.at / band) as usize;
            if per.len() <= row {
                per.resize(row + 1, 0);
            }
            let col = per[row];
            per[row] += 1;
            (i, row as u32, col)
        })
        .collect()
}

/// "27 runs · 5 kept · 2 crashes fixed".
pub fn experiment_counts(exps: &[Experiment]) -> String {
    let runs = exps
        .iter()
        .filter(|e| !matches!(e.outcome, Outcome::Summary | Outcome::You))
        .count();
    let kept = exps.iter().filter(|e| e.outcome == Outcome::Kept).count();
    let crash = exps.iter().filter(|e| e.outcome == Outcome::Crash).count();
    let mut s = format!("{runs} run{}", if runs == 1 { "" } else { "s" });
    s.push_str(&format!(" · {kept} kept"));
    if crash > 0 {
        s.push_str(&format!(
            " · {crash} crash{} fixed",
            if crash == 1 { "" } else { "es" }
        ));
    }
    s
}

/// Deterministic demo experiments across `hours`.
pub fn demo_experiments(n: usize, hours: u32) -> Vec<Experiment> {
    let notes = [
        "packed-register unpack: wrong tokens, reverted",
        "fused argmax over the logits: +1.0% at batch 64",
        "residual add in the epilogue: inside the noise",
        "ring buffer for sliding-window layers: KV halved",
        "persistent MoE kernel over all experts: +0.7%",
        "QKV projection and RoPE in one launch: +1.3%",
    ];
    let span = u64::from(hours) * 3600;
    (0..n)
        .map(|i| {
            let r = (i * 7 + 3) % 11;
            let outcome = match (i, r) {
                (0, _) => Outcome::Summary,
                (_, 0) | (_, 5) => Outcome::Kept,
                (_, 9) => Outcome::Crash,
                (i, _) if i == n / 2 => Outcome::You,
                _ => Outcome::NoGain,
            };
            Experiment {
                id: format!("P{:04}-E{:03}", i / 9 + 1, i + 1),
                at: span * i as u64 / n.max(1) as u64,
                outcome,
                note: notes[i % notes.len()].into(),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Document grid (quiet.md)
// ---------------------------------------------------------------------------------------------

/// A document card.
#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// First lines.
    pub excerpt: String,
    /// Seconds since last visited.
    pub visited: u64,
    /// Starred.
    pub favorite: bool,
}

/// Sort order of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocSort {
    /// Most recently visited first.
    LastVisited,
    /// A to Z.
    Title,
}

impl DocSort {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::LastVisited => "Last visited",
            Self::Title => "Title",
        }
    }
}

/// Indices to show: favourites only when `favorites`, filtered by `query`, then sorted.
pub fn doc_view(docs: &[Doc], favorites: bool, query: &str, sort: DocSort) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    let mut v: Vec<usize> = docs
        .iter()
        .enumerate()
        .filter(|(_, d)| !favorites || d.favorite)
        .filter(|(_, d)| {
            q.is_empty()
                || d.title.to_lowercase().contains(&q)
                || d.excerpt.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();
    match sort {
        DocSort::LastVisited => v.sort_by_key(|&i| (docs[i].visited, i)),
        DocSort::Title => v.sort_by_key(|&i| docs[i].title.to_lowercase()),
    }
    v
}

// ---------------------------------------------------------------------------------------------
// Background polling cost (@TekonBurns)
// ---------------------------------------------------------------------------------------------

/// What background status polling costs over a window.
#[derive(Clone, Debug, PartialEq)]
pub struct PollCost {
    /// Tokens if every poll re-reads the whole transcript.
    pub full: f64,
    /// Tokens with a status-only ping.
    pub ping: f64,
    /// Share saved, 0..1.
    pub saved: f32,
    /// Polls in the window.
    pub polls: u64,
}

/// `agents` background subagents polled every `every_s` seconds for `mins`: a full re-read costs
/// the transcript (`transcript` tokens, growing by `growth` tokens a minute); a ping costs
/// `ping` tokens.
pub fn poll_cost(
    agents: u32,
    every_s: u32,
    mins: u32,
    transcript: f64,
    growth: f64,
    ping: f64,
) -> PollCost {
    let every = every_s.max(1) as u64;
    let polls_each = u64::from(mins) * 60 / every;
    let mut full = 0.0;
    for k in 0..polls_each {
        let minute = (k * every) as f64 / 60.0;
        full += transcript + growth * minute;
    }
    full *= f64::from(agents);
    let pings = ping * (polls_each * u64::from(agents)) as f64;
    let saved = if full > 0.0 {
        (1.0 - pings / full) as f32
    } else {
        0.0
    };
    PollCost {
        full,
        ping: pings,
        saved: saved.max(0.0),
        polls: polls_each * u64::from(agents),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(compact(471_830_000.0), "471.83 M");
        assert_eq!(compact(518_700.0), "518.7 K");
        assert_eq!(compact(950.0), "950");
        assert_eq!(dollars(165_861), "$1,658.61");
        assert_eq!(dollars(5), "$0.05");
        assert_eq!(grouped(1284), "1,284");
        assert_eq!(grouped(12), "12");
        assert_eq!(delta(12), "+12");
        assert_eq!(delta(-3), "\u{2212}3");
        assert_eq!(ago(30), "now");
        assert_eq!(ago(21 * 60), "21m");
        assert_eq!(ago(3 * 86_400), "3d");
    }

    fn threads() -> Vec<Thread> {
        vec![
            Thread::new(
                "a",
                "orbit-web",
                "Clamp slider values",
                ThreadState::Idle(21 * 60),
            ),
            Thread::new(
                "b",
                "orbit-web",
                "Refactor the auth middleware",
                ThreadState::Approval,
            ),
            Thread::new(
                "c",
                "ledger-api",
                "Validate payout amounts",
                ThreadState::Working(120),
            ),
            Thread::new("d", "atlas-ui", "Fix focus ring on tabs", ThreadState::Done),
            Thread::new(
                "e",
                "ledger-api",
                "Pick a retry strategy",
                ThreadState::Input,
            ),
            Thread::new("f", "atlas-ui", "Old cleanup", ThreadState::Settled),
            Thread::new(
                "g",
                "orbit-web",
                "Plan the router migration",
                ThreadState::Idle(29 * 60),
            ),
        ]
    }

    #[test]
    fn grouping() {
        let t = threads();
        let g = group_threads(&t, "");
        assert_eq!(g.len(), 3);
        assert_eq!(g[0].rows, vec![1, 4, 3, 0, 6]);
        assert_eq!(g[1].label, "Working");
        assert_eq!(g[2].rows, vec![5]);
        let f = group_threads(&t, "LEDGER");
        assert_eq!(f[0].rows, vec![4]);
        assert_eq!(f[1].rows, vec![2]);
        assert_eq!(f.len(), 2);
        assert_eq!(needs_you(&t).as_deref(), Some("2 need you"));
        assert_eq!(t[2].meta(), "Working 2m");
        assert_eq!(t[0].meta(), "21m");
        assert_eq!(initials("orbit-web"), "OW");
        assert_eq!(initials("atlas"), "AT");
    }

    #[test]
    fn calendar() {
        assert_eq!(jan1_weekday(2026), 4); // Thursday
        assert_eq!(jan1_weekday(2024), 1); // Monday
        assert_eq!(year_days(2024), 366);
        assert_eq!(month_day(2026, 0), (0, 1));
        assert_eq!(month_day(2026, 59), (2, 1));
        assert_eq!(month_day(2024, 59), (1, 29));
        // 5 Oct 2026 is a Monday
        assert_eq!(day_label(2026, 277), "Mon, Oct 5");
        assert_eq!(heat_tip(5, "chat", 2026, 277), "5 chats · Mon, Oct 5");
        assert_eq!(heat_tip(0, "thread", 2026, 0), "No threads · Thu, Jan 1");
        let cols = month_cols(2026);
        assert_eq!(cols[0], (0, "Jan"));
        assert_eq!(cols[1].1, "Feb");
        let y = heat_year(2026, &[1, 2, 3]);
        assert_eq!(y.len(), 365);
        assert_eq!((y[0].col, y[0].row), (0, 4));
        assert_eq!(y[3].col, 1);
    }

    #[test]
    fn levels() {
        assert_eq!(heat_levels(&[0, 0]), vec![0, 0]);
        let l = heat_levels(&[0, 1, 2, 3, 4, 100]);
        assert_eq!(l[0], 0);
        assert_eq!(l[5], 4);
        assert!(l[1] <= l[2] && l[2] <= l[3]);
        assert_eq!(heat_recent(&demo_counts(3, 300, 100), 12).len(), 84);
    }

    #[test]
    fn usage() {
        let d = vec![
            UsageDay {
                tokens: 2e6,
                cached: 1e6,
            },
            UsageDay {
                tokens: 4e6,
                cached: 0.0,
            },
        ];
        let t = usage_totals(&d, 300.0);
        assert_eq!(t.total, "6.00 M");
        assert_eq!(t.avg, "3.00 M");
        // 5 M uncached × $3 + 1 M cached × $0.30 = $15.30
        assert_eq!(t.cost, "$15.30");
        assert_eq!(
            spark_path(&[0.0, 1.0], 1.0, 10.0, 10.0, false),
            "M0.0 10.0 L10.0 0.0"
        );
        assert_eq!(hover_index(10, 0.5), Some(5));
        assert_eq!(hover_index(0, 0.5), None);
        assert_eq!(demo_usage(30).len(), 30);
    }

    #[test]
    fn uploads() {
        let w = [1.0, 0.5];
        let a = tile_upload(&w, 1, ClockMs(0), ClockMs(500), 1000, 300);
        assert!(a.queued);
        let b = tile_upload(&w, 0, ClockMs(0), ClockMs(500), 1000, 300);
        assert!(!b.queued && b.pct > 50 && b.pct < 100);
        let c = tile_upload(&w, 0, ClockMs(0), ClockMs(1150), 1000, 300);
        assert_eq!(c.pct, 100);
        assert!((c.fade - 0.5).abs() < 0.01);
        assert!(tile_upload(&w, 0, ClockMs(0), ClockMs(1400), 1000, 300).done);
        assert!(!tile_upload(&w, 1, ClockMs(0), ClockMs(1100), 1000, 300).queued);
        assert_eq!(file_icon("cat.JPG"), "image");
        assert_eq!(file_icon("budget.csv"), "table");
        assert_eq!(tile_name("payrollzone.pdf", 7), "payroll…");
        assert_eq!(tile_name("a.pdf", 7), "a");
    }

    #[test]
    fn runs() {
        assert_eq!(RunMode::from_id("auto").label(), "Autoresearch");
        assert!(RunMode::Auto.budgeted() && !RunMode::Ask.budgeted());
        assert_eq!(hm(1440), "24h");
        assert_eq!(hm(90), "1h 30m");
        assert_eq!(parse_budget("1"), Some(5));
        assert_eq!(parse_budget("x"), None);
        let (s, f) = budget_line(2 * 3600 + 12 * 60, 1440);
        assert_eq!(s, "2h 12m of 24h");
        assert!(f > 0.09 && f < 0.1);
        let e = demo_experiments(27, 24);
        let g = experiment_grid(&e, 4);
        assert_eq!(g.len(), 27);
        assert_eq!(g[0].1, 0);
        assert!(g.iter().any(|x| x.1 == 5));
        assert!(experiment_counts(&e).starts_with("25 runs"));
    }

    #[test]
    fn docs_and_polling() {
        let d = vec![
            Doc {
                id: "a".into(),
                title: "Tables".into(),
                excerpt: "Insert a table".into(),
                visited: 300,
                favorite: true,
            },
            Doc {
                id: "b".into(),
                title: "Blocks".into(),
                excerpt: "Quotes and callouts".into(),
                visited: 60,
                favorite: false,
            },
        ];
        assert_eq!(doc_view(&d, false, "", DocSort::LastVisited), vec![1, 0]);
        assert_eq!(doc_view(&d, false, "", DocSort::Title), vec![1, 0]);
        assert_eq!(doc_view(&d, true, "", DocSort::Title), vec![0]);
        assert_eq!(doc_view(&d, false, "callout", DocSort::Title), vec![1]);
        let p = poll_cost(3, 30, 60, 40_000.0, 500.0, 300.0);
        assert_eq!(p.polls, 360);
        assert!(p.saved > 0.99);
    }
}
