//! Screens S13 (Arc screens II): **Week calendar**, **Month calendar**, **Inbox triage**,
//! **Support conversation** and **Help center** on the Arc screen frame (see
//! [`super::arcscr`]). Behaviour and layout only: every workspace, person, customer, article and
//! line of copy is fictional and our own, and every date is an absolute ISO date.
//!
//! The rules are plain functions over plain data, so the Dioxus screens, the GPUI elements and
//! [`Arc2Ui`] (the live state a GPUI view keeps) agree:
//! - Calendar: events on calendars that can be hidden, Day / Week / Month views, overlapping
//!   events laid out side by side ([`lanes`]), moves and resizes snapped to 15 minutes by drag or
//!   keys ([`nudge_event`], [`resize_event`]), new events, free gaps and "Block focus time";
//! - Inbox triage: folders and tabs, J / K to walk the list, E to resolve, H to snooze, X to
//!   select, U to undo ([`Triage`]);
//! - Support: tickets by view, a thread with replies, internal notes and article cards, a
//!   composer with saved replies, attachments and Send, Snooze and Close ([`Desk`]);
//! - Help center: articles by collection and status, search over titles and bodies with
//!   snippets ([`search`]), the reader, Was this helpful? with reasons, Publish.

pub use super::arcscr::{
    HUES, ME, ME_INI, PEOPLE, ScreenState, States, TODAY, initials, iso, month_day, thousands,
    weekday,
};

/// Week calendar states.
pub const WEEK_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("dragging", "Dragging"),
    ("creating", "New event"),
    ("dayview", "Day view"),
];
/// Month calendar states.
pub const MONTH_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("more", "More events"),
    ("hidden", "Calendars off"),
    ("focus", "Focus blocked"),
];
/// Inbox triage states.
pub const INBOX_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("selected", "Selected"),
    ("snoozing", "Snooze menu"),
    ("archived", "Resolved"),
];
/// Support conversation states.
pub const SUPPORT_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("note", "Internal note"),
    ("sent", "Reply sent"),
    ("closed", "Closed"),
];
/// Help center states.
pub const HELP_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("search", "Search"),
    ("noresults", "No results"),
    ("feedback", "Feedback"),
];

/// The five S13 slugs, in set order.
pub const S13_SLUGS: [&str; 5] = [
    "week-calendar",
    "month-calendar",
    "inbox-triage",
    "support-conversation",
    "help-center",
];

/// The state bar of an S13 screen by slug.
pub fn s13_states(slug: &str) -> States {
    match slug {
        "month-calendar" => MONTH_STATES,
        "inbox-triage" => INBOX_STATES,
        "support-conversation" => SUPPORT_STATES,
        "help-center" => HELP_STATES,
        _ => WEEK_STATES,
    }
}

/// The data state and the screen's own state for a state key (own is `""` for the three data
/// states).
pub fn arc2_split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 15] = [
        "dragging",
        "creating",
        "dayview",
        "more",
        "hidden",
        "focus",
        "selected",
        "snoozing",
        "archived",
        "note",
        "sent",
        "closed",
        "search",
        "noresults",
        "feedback",
    ];
    match OWN.iter().find(|x| **x == k) {
        Some(x) => (ScreenState::Sample, x),
        None => (ScreenState::from_key(k), ""),
    }
}

/// `570` → `09:30`.
pub fn hhmm(m: i32) -> String {
    let m = m.rem_euclid(24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// Minutes since midnight on [`TODAY`] when the screens are drawn (14:32).
pub const NOW: i32 = 14 * 60 + 32;

/// A moment as the screens show it: `14:05` today, else `2026-11-11 16:40`.
pub fn when(day: i32, m: i32) -> String {
    if day == TODAY {
        hhmm(m)
    } else {
        format!("{} {}", iso(day), hhmm(m))
    }
}

// ───────────────────────────────────────────────────────────── calendar ──

/// The calendars: (key, name, hue, group: 0 work, 1 personal).
pub const CALS: [(&str, &str, &str, usize); 6] = [
    ("team", "Team meetings", "blue", 0),
    ("focus", "Focus time", "violet", 0),
    ("launch", "Launch", "magenta", 0),
    ("holidays", "Holidays", "green", 0),
    ("personal", "Personal", "teal", 1),
    ("travel", "Travel", "amber", 1),
];

/// The calendar views: (key, label).
pub const VIEWS: [(&str, &str); 3] = [("day", "Day"), ("week", "Week"), ("month", "Month")];

/// The first and last hour the time grid shows.
pub const HOURS: (i32, i32) = (7, 23);

/// The working day used for free gaps (09:00–18:00).
pub const WORK: (i32, i32) = (9 * 60, 18 * 60);

/// One event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`CALS`].
    pub cal: usize,
    /// Demo day (see [`iso`]).
    pub day: i32,
    /// Start, minutes after midnight.
    pub start: i32,
    /// Length in minutes (all-day events: 0).
    pub len: i32,
    /// All day.
    pub all_day: bool,
    /// Where: a room, an address, or `zoom` for a video call.
    pub place: &'static str,
    /// Guests (indices into [`PEOPLE`]).
    pub who: Vec<usize>,
}

impl Event {
    /// End, minutes after midnight.
    pub fn end(&self) -> i32 {
        self.start + self.len
    }
    /// `09:30–10:30`, or `All day`.
    pub fn span(&self) -> String {
        if self.all_day {
            "All day".to_string()
        } else {
            format!("{}–{}", hhmm(self.start), hhmm(self.end()))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn ev(
    id: u32,
    title: &str,
    cal: usize,
    day: i32,
    h: i32,
    m: i32,
    len: i32,
    place: &'static str,
    who: &[usize],
) -> Event {
    Event {
        id,
        title: title.to_string(),
        cal,
        day,
        start: h * 60 + m,
        len,
        all_day: len == 0,
        place,
        who: who.to_vec(),
    }
}

/// The sample events (2026-10-26 … 2026-12-06), or none.
pub fn events(sample: bool) -> Vec<Event> {
    if !sample {
        return Vec::new();
    }
    vec![
        // the week of 2026-11-09
        ev(
            1,
            "Weekly planning",
            0,
            39,
            9,
            30,
            60,
            "Room Juniper",
            &[1, 2, 4],
        ),
        ev(
            2,
            "Design review: onboarding",
            2,
            39,
            11,
            0,
            90,
            "zoom",
            &[3, 5],
        ),
        ev(3, "Focus: pricing page copy", 1, 39, 14, 0, 120, "", &[]),
        ev(4, "1:1 with Mira", 0, 40, 10, 0, 30, "Room Alder", &[1]),
        ev(5, "Launch sync", 2, 40, 10, 15, 45, "zoom", &[2, 6, 7]),
        ev(6, "Dentist", 4, 40, 16, 30, 60, "14 Harbour Street", &[]),
        ev(
            7,
            "Interview: product designer",
            0,
            41,
            13,
            0,
            60,
            "Room Juniper",
            &[3, 8],
        ),
        ev(8, "Focus: release notes", 1, 41, 9, 30, 120, "", &[]),
        ev(
            9,
            "Team lunch",
            0,
            42,
            12,
            30,
            60,
            "Copper Kettle",
            &[1, 2, 3, 4],
        ),
        ev(
            10,
            "Customer call: Okapi Freight",
            0,
            42,
            15,
            0,
            45,
            "zoom",
            &[9],
        ),
        ev(
            11,
            "Launch dry run",
            2,
            42,
            15,
            30,
            90,
            "Room Birch",
            &[2, 6],
        ),
        ev(12, "Standup", 0, 42, 9, 15, 15, "zoom", &[1, 2]),
        ev(13, "Standup", 0, 39, 9, 0, 15, "zoom", &[1, 2]),
        ev(14, "Standup", 0, 40, 9, 15, 15, "zoom", &[1, 2]),
        ev(15, "Standup", 0, 41, 9, 0, 15, "zoom", &[1, 2]),
        ev(16, "Standup", 0, 43, 9, 15, 15, "zoom", &[1, 2]),
        ev(17, "Flight to Lisbon", 5, 44, 8, 10, 175, "LHR → LIS", &[]),
        ev(18, "Yoga", 4, 43, 18, 0, 60, "Studio Nine", &[]),
        ev(
            19,
            "Sprint retro",
            0,
            43,
            16,
            0,
            60,
            "Room Juniper",
            &[1, 2, 4, 5],
        ),
        ev(20, "Founders' Day (office closed)", 3, 41, 0, 0, 0, "", &[]),
        ev(21, "Lisbon offsite", 5, 44, 0, 0, 0, "Lisbon", &[]),
        ev(22, "Lisbon offsite", 5, 45, 0, 0, 0, "Lisbon", &[]),
        ev(
            23,
            "Brunch with Hana",
            4,
            45,
            11,
            0,
            90,
            "Rua das Flores 28",
            &[15],
        ),
        // the rest of the month
        ev(
            24,
            "All hands",
            0,
            27,
            16,
            0,
            60,
            "Main hall",
            &[1, 2, 3, 4, 5],
        ),
        ev(
            25,
            "Quarterly planning",
            0,
            33,
            10,
            0,
            120,
            "Room Juniper",
            &[1, 2, 4],
        ),
        ev(26, "Focus: hiring plan", 1, 34, 9, 0, 180, "", &[]),
        ev(27, "Board prep", 0, 35, 14, 0, 60, "Room Alder", &[1]),
        ev(28, "Brand workshop", 2, 36, 10, 0, 240, "Studio B", &[3, 5]),
        ev(29, "Return flight", 5, 47, 19, 20, 120, "LIS → LHR", &[]),
        ev(30, "Launch day", 2, 48, 0, 0, 0, "", &[]),
        ev(31, "Go / no-go", 2, 48, 9, 0, 30, "zoom", &[2, 6]),
        ev(32, "Press briefing", 2, 48, 11, 0, 60, "Room Birch", &[5]),
        ev(33, "Support standby", 0, 48, 13, 0, 240, "", &[9, 12]),
        ev(
            34,
            "Launch party",
            4,
            48,
            18,
            30,
            120,
            "The Foundry",
            &[1, 2, 3],
        ),
        ev(35, "Postmortem", 0, 50, 15, 0, 60, "zoom", &[1, 2, 6]),
        ev(36, "Focus: Q1 roadmap", 1, 54, 9, 0, 180, "", &[]),
        ev(37, "Harvest holiday", 3, 56, 0, 0, 0, "", &[]),
        ev(38, "Dinner with Theo", 4, 57, 19, 30, 120, "Saltyard", &[2]),
        ev(
            39,
            "Budget review",
            0,
            60,
            11,
            0,
            60,
            "Room Alder",
            &[1, 10],
        ),
        ev(40, "Offsite planning", 0, 62, 10, 0, 60, "zoom", &[1, 4]),
        ev(
            41,
            "Weekly planning",
            0,
            46,
            9,
            30,
            60,
            "Room Juniper",
            &[1, 2, 4],
        ),
        ev(
            42,
            "Weekly planning",
            0,
            53,
            9,
            30,
            60,
            "Room Juniper",
            &[1, 2, 4],
        ),
    ]
}

/// The Monday of the week holding day `d`.
pub fn week_start(d: i32) -> i32 {
    d - weekday(d) as i32
}

/// The ISO week number of day `d` (2026-11-09 is week 46).
pub fn iso_week(d: i32) -> i32 {
    // 2027-01-04 (day 95) starts week 1 of 2027; 2025-12-29 started week 1 of 2026.
    if d >= 95 {
        (d - 95).div_euclid(7) + 1
    } else {
        (d + 276).div_euclid(7) + 1
    }
}

const MONTH_FIRST: [(i32, &str); 7] = [
    (-30, "2026-09"),
    (0, "2026-10"),
    (31, "2026-11"),
    (61, "2026-12"),
    (92, "2027-01"),
    (123, "2027-02"),
    (151, "2027-03"),
];

/// The first day and the `YYYY-MM` of the month holding day `d`.
pub fn month_of(d: i32) -> (i32, &'static str) {
    let mut out = MONTH_FIRST[0];
    for m in MONTH_FIRST {
        if d >= m.0 {
            out = m;
        }
    }
    out
}

/// The first day of the month after (or before, `dir < 0`) the one holding `d`, kept inside
/// the demo calendar.
pub fn month_step(d: i32, dir: i32) -> i32 {
    let i = MONTH_FIRST.iter().rposition(|m| d >= m.0).unwrap_or(0) as i32;
    MONTH_FIRST[(i + dir).clamp(0, MONTH_FIRST.len() as i32 - 1) as usize].0
}

/// The 42 days of the month grid holding day `d` (weeks start on Monday).
pub fn month_cells(d: i32) -> Vec<i32> {
    let first = month_of(d).0;
    let s = week_start(first);
    (s..s + 42).collect()
}

/// The weekday names from Monday.
pub const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Events of day `d` on calendars not `hidden`, all-day first, then by start.
pub fn day_events(v: &[Event], d: i32, hidden: &[bool; 6]) -> Vec<Event> {
    let mut out: Vec<Event> = v
        .iter()
        .filter(|e| e.day == d && !hidden[e.cal.min(5)])
        .cloned()
        .collect();
    out.sort_by_key(|e| (!e.all_day, e.start, e.id));
    out
}

/// Side-by-side layout of one day's timed events: (id, lane, lanes in its cluster).
pub fn lanes(day: &[Event]) -> Vec<(u32, usize, usize)> {
    let mut t: Vec<&Event> = day.iter().filter(|e| !e.all_day).collect();
    t.sort_by_key(|e| (e.start, std::cmp::Reverse(e.len), e.id));
    let mut out: Vec<(u32, usize, usize)> = Vec::new();
    let mut cluster: Vec<(u32, usize)> = Vec::new();
    let mut ends: Vec<i32> = Vec::new();
    let mut cluster_end = i32::MIN;
    let flush = |cluster: &mut Vec<(u32, usize)>, ends: &mut Vec<i32>, out: &mut Vec<_>| {
        let n = ends.len().max(1);
        for (id, l) in cluster.drain(..) {
            out.push((id, l, n));
        }
        ends.clear();
    };
    for e in t {
        if e.start >= cluster_end && !cluster.is_empty() {
            flush(&mut cluster, &mut ends, &mut out);
        }
        let lane = match ends.iter().position(|x| *x <= e.start) {
            Some(i) => {
                ends[i] = e.end();
                i
            }
            None => {
                ends.push(e.end());
                ends.len() - 1
            }
        };
        cluster.push((e.id, lane));
        cluster_end = cluster_end.max(e.end());
    }
    flush(&mut cluster, &mut ends, &mut out);
    out
}

/// Snaps minutes to the 15-minute grid.
pub fn snap(m: i32) -> i32 {
    (m as f64 / 15.0).round() as i32 * 15
}

/// Moves event `id` to day `day` at `start` (snapped, kept inside the day). False when the
/// event is all day, unknown or did not move.
pub fn move_event(v: &mut [Event], id: u32, day: i32, start: i32) -> bool {
    let Some(e) = v.iter_mut().find(|e| e.id == id) else {
        return false;
    };
    if e.all_day {
        if e.day == day {
            return false;
        }
        e.day = day;
        return true;
    }
    let s = snap(start).clamp(0, 24 * 60 - e.len.max(15));
    if e.day == day && e.start == s {
        return false;
    }
    e.day = day;
    e.start = s;
    true
}

/// Moves event `id` by `dd` days and `dm` minutes and says where it went.
pub fn nudge_event(v: &mut [Event], id: u32, dd: i32, dm: i32) -> Option<String> {
    let e = v.iter().find(|e| e.id == id)?.clone();
    if !move_event(v, id, e.day + dd, e.start + dm) {
        return None;
    }
    let e = v.iter().find(|e| e.id == id)?;
    Some(format!("{} · {} {}", e.title, iso(e.day), e.span()))
}

/// Changes the length of event `id` by `dm` minutes (15 minutes at least, ending by midnight).
pub fn resize_event(v: &mut [Event], id: u32, dm: i32) -> Option<String> {
    let e = v.iter_mut().find(|e| e.id == id && !e.all_day)?;
    let len = snap(e.len + dm).clamp(15, 24 * 60 - e.start);
    if len == e.len {
        return None;
    }
    e.len = len;
    Some(format!("{} · {}", e.title, e.span()))
}

/// Adds a timed event and returns its id.
pub fn add_event(
    v: &mut Vec<Event>,
    day: i32,
    start: i32,
    len: i32,
    title: &str,
    cal: usize,
) -> u32 {
    let id = v.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    let t = title.trim();
    v.push(Event {
        id,
        title: if t.is_empty() {
            "New event".to_string()
        } else {
            t.to_string()
        },
        cal: cal.min(5),
        day,
        start: snap(start).clamp(0, 24 * 60 - 15),
        len: snap(len).max(15),
        all_day: false,
        place: "",
        who: Vec::new(),
    });
    id
}

/// Free gaps of day `d` inside working hours, 30 minutes or longer: (start, end).
pub fn free_gaps(v: &[Event], d: i32, hidden: &[bool; 6]) -> Vec<(i32, i32)> {
    let mut busy: Vec<(i32, i32)> = day_events(v, d, hidden)
        .iter()
        .filter(|e| !e.all_day)
        .map(|e| (e.start.max(WORK.0), e.end().min(WORK.1)))
        .filter(|(a, b)| a < b)
        .collect();
    busy.sort();
    let mut out = Vec::new();
    let mut at = WORK.0;
    for (a, b) in busy {
        if a - at >= 30 {
            out.push((at, a));
        }
        at = at.max(b);
    }
    if WORK.1 - at >= 30 {
        out.push((at, WORK.1));
    }
    out
}

/// Busy minutes of day `d` inside working hours.
pub fn busy_minutes(v: &[Event], d: i32, hidden: &[bool; 6]) -> i32 {
    let free: i32 = free_gaps(v, d, hidden).iter().map(|(a, b)| b - a).sum();
    // gaps under 30 minutes count as busy
    WORK.1 - WORK.0 - free
}

/// Blocks focus time in the first free gap of an hour or more on day `d` (up to two hours).
pub fn block_focus(v: &mut Vec<Event>, d: i32, hidden: &[bool; 6]) -> Option<String> {
    let (a, b) = free_gaps(v, d, hidden)
        .into_iter()
        .find(|(a, b)| b - a >= 60)?;
    let len = (b - a).min(120);
    add_event(v, d, a, len, "Focus time", 1);
    Some(format!(
        "Focus time blocked {} {}–{}",
        iso(d),
        hhmm(a),
        hhmm(a + len)
    ))
}

/// Events per calendar inside the month holding day `d`.
pub fn cal_counts(v: &[Event], d: i32) -> [usize; 6] {
    let (first, _) = month_of(d);
    let next = month_step(d, 1);
    let mut n = [0usize; 6];
    for e in v.iter().filter(|e| e.day >= first && e.day < next) {
        n[e.cal.min(5)] += 1;
    }
    n
}

/// The header title of a view anchored on day `d`: `2026-11-09 – 2026-11-15 · Week 46`,
/// `2026-11` or `Thursday 2026-11-12`.
pub fn period_title(view: usize, d: i32) -> String {
    match view {
        0 => format!(
            "{} {}",
            [
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday"
            ][weekday(d) as usize],
            iso(d)
        ),
        2 => month_of(d).1.to_string(),
        _ => {
            let s = week_start(d);
            format!("{} – {}", iso(s), iso(s + 6))
        }
    }
}

/// The calendar sidebar nav of each calendar screen: (key, icon, label, count).
pub fn cal_nav(slug: &str) -> [(&'static str, &'static str, &'static str, &'static str); 3] {
    if slug == "month-calendar" {
        [
            ("calendar", "cal", "Calendar", ""),
            ("upcoming", "clock", "Upcoming", ""),
            ("invites", "mail", "Invitations", "3"),
        ]
    } else {
        [
            ("calendar", "cal", "Calendar", ""),
            ("invites", "mail", "Invites", "3"),
            ("scheduling", "link", "Scheduling", ""),
        ]
    }
}

// ───────────────────────────────────────────────────────────── customers ──

/// A fictional customer: name, company, email, plan, city, UTC offset (minutes), customer since.
pub type Customer = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    i32,
    &'static str,
);

/// The customers of the inbox and the support desk.
pub const CUSTOMERS: [Customer; 12] = [
    (
        "Amara Okonkwo",
        "Okapi Freight",
        "amara@okapifreight.example",
        "Scale",
        "Lagos",
        60,
        "2024-03-18",
    ),
    (
        "Jonah Pike",
        "Brightwell Labs",
        "jonah@brightwell.example",
        "Team",
        "Toronto",
        -300,
        "2025-01-07",
    ),
    (
        "Selin Aydın",
        "Lumen & Vine",
        "selin@lumenvine.example",
        "Scale",
        "Istanbul",
        180,
        "2023-11-02",
    ),
    (
        "Marcus Bell",
        "Harborline",
        "marcus@harborline.example",
        "Starter",
        "Glasgow",
        0,
        "2026-09-21",
    ),
    (
        "Ines Carvalho",
        "Northpaw",
        "ines@northpaw.example",
        "Team",
        "Porto",
        0,
        "2025-06-30",
    ),
    (
        "Tomasz Wrona",
        "Quarry Cloud",
        "tomasz@quarrycloud.example",
        "Enterprise",
        "Kraków",
        60,
        "2022-08-15",
    ),
    (
        "Grace Liu",
        "Pebble & Post",
        "grace@pebblepost.example",
        "Team",
        "Singapore",
        480,
        "2024-12-01",
    ),
    (
        "Ravi Menon",
        "Tallow Studio",
        "ravi@tallow.example",
        "Starter",
        "Pune",
        330,
        "2026-10-04",
    ),
    (
        "Elena Petrova",
        "Fjordline",
        "elena@fjordline.example",
        "Scale",
        "Oslo",
        60,
        "2023-05-12",
    ),
    (
        "Kofi Mensah",
        "Saffron Health",
        "kofi@saffronhealth.example",
        "Enterprise",
        "Accra",
        0,
        "2022-02-28",
    ),
    (
        "Laila Haddad",
        "Driftwood Books",
        "laila@driftwood.example",
        "Team",
        "Amman",
        180,
        "2025-09-09",
    ),
    (
        "Owen Fraser",
        "Kiln Coffee",
        "owen@kilncoffee.example",
        "Starter",
        "Wellington",
        780,
        "2026-07-19",
    ),
];

/// Initials of customer `c`.
pub fn cust_ini(c: usize) -> String {
    CUSTOMERS[c % CUSTOMERS.len()]
        .0
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect()
}

/// A customer's hue (for the avatar).
pub fn cust_hue(c: usize) -> &'static str {
    HUES[(c * 3 + 1) % HUES.len()]
}

/// A customer's local time now (`15:32`). The screens are drawn at 13:32 UTC.
pub fn cust_time(c: usize) -> String {
    hhmm(13 * 60 + 32 + CUSTOMERS[c % CUSTOMERS.len()].5)
}

// ───────────────────────────────────────────────────────────── inbox ──

/// Inbox folders: (key, label, icon).
pub const FOLDERS: [(&str, &str, &str); 5] = [
    ("inbox", "Inbox", "inbox"),
    ("mine", "Assigned to me", "user"),
    ("unassigned", "Unassigned", "people"),
    ("snoozed", "Snoozed", "clock"),
    ("resolved", "Resolved", "check"),
];

/// Inbox tabs.
pub const TABS: [&str; 3] = ["All", "Unread", "Due soon"];

/// Snooze choices: (label, until).
pub const SNOOZES: [(&str, &str); 3] = [
    ("Later today", "2026-11-12 17:00"),
    ("Tomorrow", "2026-11-13 09:00"),
    ("Next week", "2026-11-16 09:00"),
];

/// Conversation labels: (name, hue).
pub const TAGS: [(&str, &str); 4] = [
    ("Billing", "amber"),
    ("Bug", "rose"),
    ("Question", "blue"),
    ("Feature request", "violet"),
];

/// Where a conversation is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// In the inbox.
    Open,
    /// Snoozed until (`2026-11-13 09:00`).
    Snoozed(&'static str),
    /// Resolved.
    Resolved,
}

/// One inbox conversation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conv {
    /// Stable id.
    pub id: u32,
    /// Index into [`CUSTOMERS`].
    pub cust: usize,
    /// Channel mark: `gmail`, `slack`, `x` or `chat` (the site widget).
    pub channel: &'static str,
    /// Subject.
    pub subject: &'static str,
    /// The last message.
    pub preview: &'static str,
    /// Demo day of the last message.
    pub day: i32,
    /// Minutes after midnight of the last message.
    pub at: i32,
    /// Unread.
    pub unread: bool,
    /// Urgent.
    pub urgent: bool,
    /// Minutes left to reply (negative: late), when an SLA runs.
    pub sla: Option<i32>,
    /// Assignee (index into [`PEOPLE`]; 0 is the signed-in person).
    pub who: Option<usize>,
    /// Index into [`TAGS`].
    pub tag: usize,
    /// Where it is.
    pub place: Place,
}

#[allow(clippy::too_many_arguments)]
fn cv(
    id: u32,
    cust: usize,
    channel: &'static str,
    subject: &'static str,
    preview: &'static str,
    day: i32,
    at: i32,
    unread: bool,
    urgent: bool,
    sla: Option<i32>,
    who: Option<usize>,
    tag: usize,
    place: Place,
) -> Conv {
    Conv {
        id,
        cust,
        channel,
        subject,
        preview,
        day,
        at,
        unread,
        urgent,
        sla,
        who,
        tag,
        place,
    }
}

/// The sample conversations, or none.
pub fn convs(sample: bool) -> Vec<Conv> {
    use Place::*;
    if !sample {
        return Vec::new();
    }
    let t = TODAY;
    vec![
        cv(
            101,
            0,
            "gmail",
            "Invoices show the wrong tax ID",
            "Our VAT number changed in October and the last two invoices still carry the old one.",
            t,
            14 * 60 + 18,
            true,
            true,
            Some(14),
            Some(0),
            0,
            Open,
        ),
        cv(
            102,
            1,
            "chat",
            "Can't export more than 10k rows",
            "Export spins for a minute and then says something went wrong. Smaller ranges work.",
            t,
            13 * 60 + 52,
            true,
            false,
            Some(38),
            None,
            1,
            Open,
        ),
        cv(
            103,
            2,
            "slack",
            "Seats for the design team",
            "We'd like to add six seats before Monday. Do we get the annual price on those too?",
            t,
            13 * 60 + 5,
            false,
            false,
            Some(95),
            Some(0),
            0,
            Open,
        ),
        cv(
            104,
            3,
            "gmail",
            "Password reset email never arrives",
            "Tried three times since this morning, nothing in spam either.",
            t,
            12 * 60 + 41,
            true,
            true,
            Some(-12),
            None,
            1,
            Open,
        ),
        cv(
            105,
            4,
            "x",
            "Dark mode for the mobile app?",
            "Love the app, any plans for a dark theme? My eyes at midnight say thanks in advance.",
            t,
            11 * 60 + 30,
            false,
            false,
            None,
            Some(4),
            3,
            Open,
        ),
        cv(
            106,
            5,
            "gmail",
            "SAML login loops back to the start",
            "After the IdP redirect we land on the sign-in page again. Happens for about a third of the team.",
            t,
            10 * 60 + 2,
            true,
            false,
            Some(55),
            Some(0),
            1,
            Open,
        ),
        cv(
            107,
            6,
            "chat",
            "How do I move a project between workspaces?",
            "We split into two teams and want the Atlas project in the new workspace.",
            t,
            9 * 60 + 47,
            false,
            false,
            Some(180),
            Some(2),
            2,
            Open,
        ),
        cv(
            108,
            7,
            "gmail",
            "Refund for a double charge",
            "I was charged twice on 2026-11-10, receipt numbers in the attachment.",
            t - 1,
            17 * 60 + 20,
            true,
            false,
            Some(240),
            None,
            0,
            Open,
        ),
        cv(
            109,
            8,
            "slack",
            "Webhook retries look stuck",
            "Deliveries to our endpoint show Retrying since yesterday evening.",
            t - 1,
            16 * 60 + 40,
            false,
            false,
            Some(300),
            Some(0),
            1,
            Open,
        ),
        cv(
            110,
            9,
            "gmail",
            "Data processing agreement",
            "Our legal team needs the signed DPA before the renewal on 2026-12-01.",
            t - 1,
            11 * 60 + 15,
            false,
            false,
            None,
            None,
            2,
            Open,
        ),
        cv(
            111,
            10,
            "chat",
            "Calendar sync shows events twice",
            "Every meeting appears twice since I connected my second calendar.",
            t - 2,
            15 * 60 + 5,
            false,
            false,
            None,
            Some(4),
            1,
            Open,
        ),
        cv(
            112,
            11,
            "x",
            "Annual plan in NZD?",
            "Is there a way to pay the annual plan in NZD instead of USD?",
            t - 2,
            9 * 60 + 12,
            false,
            false,
            None,
            Some(0),
            0,
            Open,
        ),
        cv(
            113,
            0,
            "gmail",
            "Add a billing contact",
            "Please copy finance@okapifreight.example on every invoice.",
            t - 3,
            14 * 60,
            false,
            false,
            None,
            Some(0),
            0,
            Snoozed("2026-11-13 09:00"),
        ),
        cv(
            114,
            5,
            "slack",
            "Audit log export format",
            "Could the audit log export as JSON lines?",
            t - 4,
            10 * 60 + 30,
            false,
            false,
            None,
            Some(2),
            3,
            Snoozed("2026-11-16 09:00"),
        ),
        cv(
            115,
            2,
            "gmail",
            "Change the account owner",
            "Done, thank you!",
            t - 5,
            16 * 60,
            false,
            false,
            None,
            Some(0),
            2,
            Resolved,
        ),
        cv(
            116,
            6,
            "chat",
            "Bulk invite from CSV",
            "That worked, thanks.",
            t - 6,
            12 * 60,
            false,
            false,
            None,
            Some(4),
            2,
            Resolved,
        ),
    ]
}

/// The thread of conversation `id`: (from the customer, day, minute, text). The last message is
/// the conversation's preview.
pub fn thread(c: &Conv) -> Vec<(bool, i32, i32, String)> {
    let opener = match c.id {
        101 => Some("Hi, quick one on billing."),
        102 => Some("Hey team, exports have been failing for us since the weekend."),
        104 => Some("Hello, I can't get into my account."),
        106 => Some("We switched to SAML on Monday."),
        108 => Some("Hi there,"),
        109 => Some("Hi, our webhook endpoint is healthy but deliveries are not going through."),
        _ => None,
    };
    let mut out = Vec::new();
    if let Some(o) = opener {
        out.push((true, c.day, (c.at - 6).max(0), o.to_string()));
    }
    if c.id == 106 {
        out.push((
            false,
            c.day,
            c.at - 3,
            "Thanks Tomasz, which identity provider do you use, and does it happen in a private window too?".to_string(),
        ));
    }
    out.push((true, c.day, c.at, c.preview.to_string()));
    out
}

/// The inbox's live state: conversations, folder, tab, order, the open one, the selection and
/// what Undo brings back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triage {
    /// Every conversation.
    pub convs: Vec<Conv>,
    /// Index into [`FOLDERS`].
    pub folder: usize,
    /// Index into [`TABS`].
    pub tab: usize,
    /// Newest first (else oldest first).
    pub newest: bool,
    /// The open conversation.
    pub open: Option<u32>,
    /// Selected conversations (bulk actions act on them).
    pub sel: Vec<u32>,
    /// Snapshots for Undo, with what they undo.
    pub undo: Vec<(Vec<Conv>, String)>,
    /// The snooze menu is open.
    pub snoozing: bool,
}

impl Triage {
    /// The sample inbox (or an empty one) with the first conversation open.
    pub fn new(sample: bool) -> Self {
        let convs = convs(sample);
        let open = convs.first().map(|c| c.id);
        Self {
            convs,
            folder: 0,
            tab: 0,
            newest: true,
            open,
            sel: Vec::new(),
            undo: Vec::new(),
            snoozing: false,
        }
    }

    fn in_folder(c: &Conv, f: usize) -> bool {
        match f {
            1 => c.place == Place::Open && c.who == Some(0),
            2 => c.place == Place::Open && c.who.is_none(),
            3 => matches!(c.place, Place::Snoozed(_)),
            4 => c.place == Place::Resolved,
            _ => c.place == Place::Open,
        }
    }

    /// Conversations in folder `f`.
    pub fn count(&self, f: usize) -> usize {
        self.convs.iter().filter(|c| Self::in_folder(c, f)).count()
    }

    /// Unread conversations in the open folder.
    pub fn unread(&self) -> usize {
        self.convs
            .iter()
            .filter(|c| Self::in_folder(c, self.folder) && c.unread)
            .count()
    }

    /// The list as shown: the open folder and tab, in the chosen order.
    pub fn list(&self) -> Vec<Conv> {
        let mut v: Vec<Conv> = self
            .convs
            .iter()
            .filter(|c| Self::in_folder(c, self.folder))
            .filter(|c| match self.tab {
                1 => c.unread,
                2 => c.sla.is_some_and(|s| s <= 60),
                _ => true,
            })
            .cloned()
            .collect();
        v.sort_by_key(|c| (c.day, c.at));
        if self.newest {
            v.reverse();
        }
        v
    }

    /// The open conversation, when it is in the list.
    pub fn current(&self) -> Option<Conv> {
        let l = self.list();
        self.open.and_then(|id| l.into_iter().find(|c| c.id == id))
    }

    /// Opens conversation `id` and marks it read.
    pub fn open_id(&mut self, id: u32) -> bool {
        let Some(c) = self.convs.iter_mut().find(|c| c.id == id) else {
            return false;
        };
        c.unread = false;
        self.open = Some(id);
        true
    }

    /// J (`1`) or K (`-1`): opens the next or previous conversation in the list.
    pub fn step(&mut self, d: i32) -> Option<String> {
        let l = self.list();
        if l.is_empty() {
            return None;
        }
        let at = self.open.and_then(|id| l.iter().position(|c| c.id == id));
        let i = match at {
            Some(i) => (i as i32 + d).clamp(0, l.len() as i32 - 1) as usize,
            None => 0,
        };
        if at == Some(i) {
            return None;
        }
        let c = l[i].clone();
        self.open_id(c.id);
        Some(format!("{}: {}", CUSTOMERS[c.cust].0, c.subject))
    }

    /// X: selects or clears conversation `id` (the open one when `None`).
    pub fn toggle(&mut self, id: Option<u32>) -> Option<String> {
        let id = id.or(self.open)?;
        if let Some(i) = self.sel.iter().position(|x| *x == id) {
            self.sel.remove(i);
        } else {
            self.sel.push(id);
        }
        Some(format!("{} selected", self.sel.len()))
    }

    /// Selects every conversation in the list, or clears the selection when all are selected.
    pub fn select_all(&mut self) -> String {
        let l: Vec<u32> = self.list().iter().map(|c| c.id).collect();
        if !l.is_empty() && l.iter().all(|id| self.sel.contains(id)) {
            self.sel.clear();
        } else {
            self.sel = l;
        }
        format!("{} selected", self.sel.len())
    }

    fn targets(&self) -> Vec<u32> {
        if self.sel.is_empty() {
            self.open.into_iter().collect()
        } else {
            self.sel.clone()
        }
    }

    fn after_move(&mut self, gone: &[u32], before: Vec<Conv>) {
        // open the next conversation that is still in the list
        let l: Vec<u32> = before
            .iter()
            .filter(|c| Self::in_folder(c, self.folder))
            .map(|c| c.id)
            .collect();
        if let Some(o) = self.open
            && gone.contains(&o)
        {
            let old = Triage {
                convs: before,
                ..self.clone()
            };
            let ol = old.list();
            let at = ol.iter().position(|c| c.id == o).unwrap_or(0);
            let next = ol
                .iter()
                .skip(at + 1)
                .chain(ol.iter().take(at).rev())
                .find(|c| !gone.contains(&c.id) && l.contains(&c.id))
                .map(|c| c.id);
            self.open = next;
            if let Some(n) = next {
                self.open_id(n);
            }
        }
        self.sel.clear();
        self.snoozing = false;
    }

    /// E: resolves the selection (or the open conversation).
    pub fn resolve(&mut self) -> Option<String> {
        let t = self.targets();
        if t.is_empty() {
            return None;
        }
        let before = self.convs.clone();
        for c in self.convs.iter_mut().filter(|c| t.contains(&c.id)) {
            c.place = Place::Resolved;
        }
        let s = if t.len() == 1 {
            let c = before.iter().find(|c| c.id == t[0])?;
            format!("Resolved: {}", c.subject)
        } else {
            format!("{} conversations resolved", t.len())
        };
        self.undo.push((before.clone(), s.clone()));
        self.after_move(&t, before);
        Some(s)
    }

    /// H: snoozes the selection (or the open conversation) with choice `n` of [`SNOOZES`].
    pub fn snooze(&mut self, n: usize) -> Option<String> {
        let t = self.targets();
        if t.is_empty() {
            return None;
        }
        let (l, until) = SNOOZES[n.min(SNOOZES.len() - 1)];
        let before = self.convs.clone();
        for c in self.convs.iter_mut().filter(|c| t.contains(&c.id)) {
            c.place = Place::Snoozed(until);
        }
        let s = if t.len() == 1 {
            format!("Snoozed until {until} ({})", l.to_lowercase())
        } else {
            format!("{} conversations snoozed until {until}", t.len())
        };
        self.undo.push((before.clone(), s.clone()));
        self.after_move(&t, before);
        Some(s)
    }

    /// Assigns the selection (or the open conversation) to the signed-in person.
    pub fn assign_me(&mut self) -> Option<String> {
        let t = self.targets();
        if t.is_empty() {
            return None;
        }
        let before = self.convs.clone();
        for c in self.convs.iter_mut().filter(|c| t.contains(&c.id)) {
            c.who = Some(0);
        }
        let s = format!("Assigned to you ({})", t.len());
        self.undo.push((before, s.clone()));
        Some(s)
    }

    /// U: undoes the last resolve, snooze or assignment.
    pub fn undo_last(&mut self) -> Option<String> {
        let (v, s) = self.undo.pop()?;
        self.convs = v;
        Some(format!("Undone: {s}"))
    }

    /// A triage key (`j`, `k`, `e`, `h`, `x`, `u`, `a`); `None` when it did nothing.
    pub fn key(&mut self, k: &str) -> Option<String> {
        match k {
            "j" => self.step(1),
            "k" => self.step(-1),
            "e" => self.resolve(),
            "h" => self.snooze(0),
            "x" => self.toggle(None),
            "u" => self.undo_last(),
            "a" => self.assign_me(),
            _ => None,
        }
    }

    /// Opens folder `f` (clears the selection, opens its first conversation).
    pub fn folder_to(&mut self, f: usize) -> String {
        self.folder = f.min(FOLDERS.len() - 1);
        self.sel.clear();
        self.snoozing = false;
        self.open = self.list().first().map(|c| c.id);
        format!("{}: {}", FOLDERS[self.folder].1, self.list().len())
    }

    /// Opens tab `t`.
    pub fn tab_to(&mut self, t: usize) -> String {
        self.tab = t.min(TABS.len() - 1);
        self.sel.clear();
        if self.current().is_none() {
            self.open = self.list().first().map(|c| c.id);
        }
        format!("{}: {}", TABS[self.tab], self.list().len())
    }
}

/// Minutes left as a short SLA (`14m`, `1h 35m`, `12m late`).
pub fn sla_words(m: i32) -> String {
    let a = m.abs();
    let t = if a >= 60 {
        format!("{}h {}m", a / 60, a % 60)
    } else {
        format!("{a}m")
    };
    if m < 0 { format!("{t} late") } else { t }
}

/// The SLA hue: rose when late or under 15 minutes, amber under an hour, else muted.
pub fn sla_hue(m: i32) -> &'static str {
    if m < 15 {
        "rose"
    } else if m <= 60 {
        "amber"
    } else {
        "mute"
    }
}

/// The inbox team in the sidebar: (person, open conversations or a status).
pub const IB_TEAM: [(usize, &str); 3] = [(1, "2"), (4, "1"), (2, "Back at 16:00")];

// ───────────────────────────────────────────────────────────── support ──

/// Support views: (key, label, icon).
pub const SP_VIEWS: [(&str, &str, &str); 6] = [
    ("inbox", "Your inbox", "inbox"),
    ("mentions", "Mentions", "bell"),
    ("unassigned", "Unassigned", "people"),
    ("open", "All open", "list"),
    ("snoozed", "Snoozed", "clock"),
    ("closed", "Closed", "check"),
];

/// What a message is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// From the customer.
    Customer,
    /// A reply from a teammate (index into [`PEOPLE`]).
    Reply(usize),
    /// An internal note by a teammate.
    Note(usize),
    /// A system line (assigned, snoozed, closed).
    Event,
}

/// One message in a ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Msg {
    /// What it is.
    pub kind: Kind,
    /// When (`2026-11-12 13:58`).
    pub at: String,
    /// Text.
    pub text: String,
    /// An article card (index into [`articles`]).
    pub article: Option<u32>,
    /// Attached file names.
    pub files: Vec<String>,
}

fn msg(kind: Kind, day: i32, m: i32, text: &str) -> Msg {
    Msg {
        kind,
        at: format!("{} {}", iso(day), hhmm(m)),
        text: text.to_string(),
        article: None,
        files: Vec::new(),
    }
}

/// A ticket's status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TStatus {
    /// Waiting on us.
    Open,
    /// We replied; waiting on the customer.
    Waiting,
    /// Snoozed until.
    Snoozed(&'static str),
    /// Closed.
    Closed,
}

impl TStatus {
    /// Label.
    pub fn label(&self) -> String {
        match self {
            TStatus::Open => "Open".to_string(),
            TStatus::Waiting => "Waiting on customer".to_string(),
            TStatus::Snoozed(u) => format!("Snoozed until {u}"),
            TStatus::Closed => "Closed".to_string(),
        }
    }
    /// Hue.
    pub fn hue(&self) -> &'static str {
        match self {
            TStatus::Open => "blue",
            TStatus::Waiting => "amber",
            TStatus::Snoozed(_) => "violet",
            TStatus::Closed => "green",
        }
    }
}

/// One support ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    /// Stable id (`#4812`).
    pub id: u32,
    /// Index into [`CUSTOMERS`].
    pub cust: usize,
    /// Subject.
    pub subject: &'static str,
    /// Status.
    pub status: TStatus,
    /// Minutes left to reply (negative: late).
    pub sla: Option<i32>,
    /// Assignee (0 is the signed-in person).
    pub who: Option<usize>,
    /// The signed-in person is mentioned in a note.
    pub mention: bool,
    /// Urgent.
    pub urgent: bool,
    /// Unread.
    pub unread: bool,
    /// Channel mark (`gmail`, `slack`, `chat`).
    pub channel: &'static str,
    /// Linked issue (`STF-231 Export times out over 10k rows`).
    pub issue: &'static str,
    /// The messages, oldest first.
    pub msgs: Vec<Msg>,
}

/// The sample tickets, or none.
pub fn tickets(sample: bool) -> Vec<Ticket> {
    if !sample {
        return Vec::new();
    }
    let t = TODAY;
    let mut art = msg(
        Kind::Reply(0),
        t,
        13 * 60 + 41,
        "Thanks Jonah. While we look, splitting the export by month works around the limit:",
    );
    art.article = Some(12);
    let mut shot = msg(
        Kind::Customer,
        t,
        13 * 60 + 36,
        "Here's what we see after about a minute.",
    );
    shot.files = vec!["export-error.png".to_string()];
    vec![
        Ticket {
            id: 4812,
            cust: 1,
            subject: "Exports over 10k rows time out",
            status: TStatus::Open,
            sla: Some(11),
            who: Some(0),
            mention: true,
            urgent: true,
            unread: false,
            channel: "gmail",
            issue: "STF-231 · Export times out over 10k rows",
            msgs: vec![
                msg(
                    Kind::Customer,
                    t,
                    13 * 60 + 30,
                    "Hi! Since Saturday our weekly export stops with Something went wrong. It's the orders table, about 14,000 rows. Smaller date ranges still work.",
                ),
                shot,
                msg(
                    Kind::Event,
                    t,
                    13 * 60 + 38,
                    "Rowan Vale assigned this to themselves",
                ),
                art,
                msg(
                    Kind::Note(1),
                    t,
                    13 * 60 + 50,
                    "@Rowan this matches STF-231; the fix ships in 4.18 on 2026-11-16. Same root cause as Brightwell's ticket last week.",
                ),
                msg(
                    Kind::Customer,
                    t,
                    14 * 60 + 21,
                    "Splitting works, thank you. Is there a date for the real fix? Finance runs this every Monday.",
                ),
            ],
        },
        Ticket {
            id: 4809,
            cust: 3,
            subject: "Reset email never arrives",
            status: TStatus::Open,
            sla: Some(-12),
            who: Some(0),
            mention: false,
            urgent: true,
            unread: true,
            channel: "gmail",
            issue: "",
            msgs: vec![
                msg(
                    Kind::Customer,
                    t,
                    12 * 60 + 35,
                    "Hello, I can't get into my account.",
                ),
                msg(
                    Kind::Customer,
                    t,
                    12 * 60 + 41,
                    "Tried the reset three times since this morning, nothing in spam either.",
                ),
            ],
        },
        Ticket {
            id: 4807,
            cust: 5,
            subject: "SAML login loops back to the start",
            status: TStatus::Open,
            sla: Some(55),
            who: Some(0),
            mention: true,
            urgent: false,
            unread: false,
            channel: "slack",
            issue: "STF-228 · SAML relay state dropped",
            msgs: vec![
                msg(
                    Kind::Customer,
                    t,
                    9 * 60 + 58,
                    "We switched to SAML on Monday. After the identity provider redirect, about a third of the team lands on the sign-in page again.",
                ),
                msg(
                    Kind::Reply(0),
                    t,
                    10 * 60 + 4,
                    "Thanks Tomasz. Which identity provider do you use, and does it happen in a private window too?",
                ),
                msg(
                    Kind::Customer,
                    t,
                    10 * 60 + 9,
                    "Our own Keycloak. Private windows work, so it's probably a cookie.",
                ),
                msg(
                    Kind::Note(2),
                    t,
                    10 * 60 + 20,
                    "@Rowan relay state is dropped when the old session cookie is present. Workaround: sign out everywhere first.",
                ),
            ],
        },
        Ticket {
            id: 4801,
            cust: 8,
            subject: "Webhook retries look stuck",
            status: TStatus::Waiting,
            sla: None,
            who: Some(0),
            mention: false,
            urgent: false,
            unread: false,
            channel: "slack",
            issue: "",
            msgs: vec![
                msg(
                    Kind::Customer,
                    t - 1,
                    16 * 60 + 40,
                    "Deliveries to our endpoint show Retrying since yesterday evening.",
                ),
                msg(
                    Kind::Reply(0),
                    t - 1,
                    17 * 60 + 2,
                    "Your endpoint answers 301 to our requests; webhooks don't follow redirects. Could you point it at the final URL?",
                ),
            ],
        },
        Ticket {
            id: 4798,
            cust: 0,
            subject: "Invoices show the wrong tax ID",
            status: TStatus::Open,
            sla: Some(140),
            who: None,
            mention: false,
            urgent: false,
            unread: true,
            channel: "chat",
            issue: "",
            msgs: vec![msg(
                Kind::Customer,
                t,
                14 * 60 + 18,
                "Our VAT number changed in October and the last two invoices still carry the old one.",
            )],
        },
        Ticket {
            id: 4790,
            cust: 10,
            subject: "Calendar sync shows events twice",
            status: TStatus::Snoozed("2026-11-13 09:00"),
            sla: None,
            who: Some(0),
            mention: false,
            urgent: false,
            unread: false,
            channel: "chat",
            issue: "",
            msgs: vec![
                msg(
                    Kind::Customer,
                    t - 2,
                    15 * 60 + 5,
                    "Every meeting appears twice since I connected my second calendar.",
                ),
                msg(
                    Kind::Reply(0),
                    t - 2,
                    15 * 60 + 30,
                    "Both calendars hold the same events; turning off sync for one of them fixes it. I'll check back tomorrow.",
                ),
            ],
        },
        Ticket {
            id: 4775,
            cust: 6,
            subject: "Bulk invite from CSV",
            status: TStatus::Closed,
            sla: None,
            who: Some(0),
            mention: false,
            urgent: false,
            unread: false,
            channel: "chat",
            issue: "",
            msgs: vec![msg(Kind::Customer, t - 6, 12 * 60, "That worked, thanks.")],
        },
    ]
}

/// Saved replies: (name, text with `{first}` for the customer's first name).
pub const SAVED: [(&str, &str); 4] = [
    (
        "Fix date",
        "Hi {first}, the fix ships in version 4.18 on 2026-11-16. I'll write as soon as it's live.",
    ),
    (
        "Need more detail",
        "Hi {first}, could you send the time it happened (with time zone) and a screenshot? That lets us find it in the logs.",
    ),
    (
        "Refund on its way",
        "Hi {first}, I've refunded the duplicate charge. It shows on your statement within 5 to 10 business days.",
    ),
    (
        "Closing the loop",
        "Hi {first}, glad that's sorted. I'll close this ticket; reply here any time to reopen it.",
    ),
];

/// Support snooze choices: (label, until).
pub const SP_SNOOZES: [(&str, &str); 3] = SNOOZES;

/// The support desk's live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Desk {
    /// Every ticket.
    pub tickets: Vec<Ticket>,
    /// The open ticket.
    pub open: Option<u32>,
    /// Index into [`SP_VIEWS`].
    pub view: usize,
    /// The composer writes an internal note (else a reply).
    pub note: bool,
    /// The composer's text.
    pub draft: String,
    /// Files attached to the draft.
    pub files: Vec<String>,
    /// Article attached to the draft.
    pub article: Option<u32>,
    /// Minute of the next message (advances with each send).
    pub clock: i32,
    /// The customer card's tab: 0 Profile, 1 Activity.
    pub card: usize,
}

impl Desk {
    /// The sample desk (or an empty one) with the first ticket open.
    pub fn new(sample: bool) -> Self {
        let tickets = tickets(sample);
        let open = tickets.first().map(|t| t.id);
        Self {
            tickets,
            open,
            view: 0,
            note: false,
            draft: String::new(),
            files: Vec::new(),
            article: None,
            clock: NOW,
            card: 0,
        }
    }

    fn in_view(t: &Ticket, v: usize) -> bool {
        let live = matches!(t.status, TStatus::Open | TStatus::Waiting);
        match v {
            1 => t.mention && live,
            2 => t.who.is_none() && live,
            3 => live,
            4 => matches!(t.status, TStatus::Snoozed(_)),
            5 => t.status == TStatus::Closed,
            _ => t.who == Some(0) && live,
        }
    }

    /// Tickets in view `v`.
    pub fn count(&self, v: usize) -> usize {
        self.tickets.iter().filter(|t| Self::in_view(t, v)).count()
    }

    /// The open view's tickets, most urgent first (late SLAs, then the least time left).
    pub fn list(&self) -> Vec<Ticket> {
        let mut v: Vec<Ticket> = self
            .tickets
            .iter()
            .filter(|t| Self::in_view(t, self.view))
            .cloned()
            .collect();
        v.sort_by_key(|t| (t.sla.is_none(), t.sla.unwrap_or(0), std::cmp::Reverse(t.id)));
        v
    }

    /// The open ticket.
    pub fn current(&self) -> Option<&Ticket> {
        self.open
            .and_then(|id| self.tickets.iter().find(|t| t.id == id))
    }

    /// Opens ticket `id` (marks it read).
    pub fn open_id(&mut self, id: u32) -> bool {
        let Some(t) = self.tickets.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        t.unread = false;
        self.open = Some(id);
        self.files.clear();
        self.article = None;
        true
    }

    /// Opens view `v` and its first ticket.
    pub fn view_to(&mut self, v: usize) -> String {
        self.view = v.min(SP_VIEWS.len() - 1);
        if let Some(id) = self.list().first().map(|t| t.id) {
            self.open_id(id);
        } else {
            self.open = None;
        }
        format!("{}: {}", SP_VIEWS[self.view].1, self.count(self.view))
    }

    /// Puts saved reply `n` in the draft for the open ticket's customer.
    pub fn saved(&mut self, n: usize) -> Option<String> {
        let t = self.current()?;
        let first = CUSTOMERS[t.cust].0.split_whitespace().next().unwrap_or("");
        let (name, text) = SAVED[n.min(SAVED.len() - 1)];
        self.draft = text.replace("{first}", first);
        self.note = false;
        Some(format!("Saved reply: {name}"))
    }

    /// Attaches the next demo file to the draft.
    pub fn attach(&mut self) -> String {
        const FILES: [&str; 3] = [
            "steps-to-export.pdf",
            "orders-sample.csv",
            "screen-recording.mp4",
        ];
        let f = FILES[self.files.len() % FILES.len()].to_string();
        self.files.push(f.clone());
        format!("{f} attached")
    }

    /// Sends the draft as a reply (status Waiting on customer) or posts it as a note. `Err` says
    /// why nothing was sent.
    pub fn send(&mut self) -> Result<String, &'static str> {
        let text = self.draft.trim().to_string();
        if text.is_empty() && self.files.is_empty() && self.article.is_none() {
            return Err("Write a reply first");
        }
        let note = self.note;
        let clock = self.clock;
        let files = std::mem::take(&mut self.files);
        let article = self.article.take();
        let id = self.open.ok_or("Open a ticket first")?;
        let t = self
            .tickets
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or("Open a ticket first")?;
        t.msgs.push(Msg {
            kind: if note { Kind::Note(0) } else { Kind::Reply(0) },
            at: format!("{} {}", iso(TODAY), hhmm(clock)),
            text,
            article,
            files,
        });
        if !note {
            t.status = TStatus::Waiting;
            t.sla = None;
        }
        self.clock += 1;
        self.draft.clear();
        Ok(if note {
            "Note added".to_string()
        } else {
            format!("Reply sent to {}", CUSTOMERS[t.cust].0)
        })
    }

    /// Closes the open ticket (or reopens a closed one).
    pub fn close(&mut self) -> Option<String> {
        let id = self.open?;
        let clock = self.clock;
        let t = self.tickets.iter_mut().find(|t| t.id == id)?;
        let s = if t.status == TStatus::Closed {
            t.status = TStatus::Open;
            "Ticket reopened"
        } else {
            t.status = TStatus::Closed;
            t.sla = None;
            "Ticket closed"
        };
        t.msgs.push(msg(
            Kind::Event,
            TODAY,
            clock,
            &format!(
                "Rowan Vale {}",
                if s == "Ticket closed" {
                    "closed the ticket"
                } else {
                    "reopened the ticket"
                }
            ),
        ));
        Some(format!("{s} #{id}"))
    }

    /// Snoozes the open ticket with choice `n`.
    pub fn snooze(&mut self, n: usize) -> Option<String> {
        let id = self.open?;
        let clock = self.clock;
        let (_, until) = SP_SNOOZES[n.min(SP_SNOOZES.len() - 1)];
        let t = self.tickets.iter_mut().find(|t| t.id == id)?;
        t.status = TStatus::Snoozed(until);
        t.msgs.push(msg(
            Kind::Event,
            TODAY,
            clock,
            &format!("Rowan Vale snoozed this until {until}"),
        ));
        Some(format!("#{id} snoozed until {until}"))
    }

    /// Assigns the open ticket to the signed-in person.
    pub fn take(&mut self) -> Option<String> {
        let id = self.open?;
        let clock = self.clock;
        let t = self.tickets.iter_mut().find(|t| t.id == id)?;
        if t.who == Some(0) {
            return None;
        }
        t.who = Some(0);
        t.msgs.push(msg(
            Kind::Event,
            TODAY,
            clock,
            "Rowan Vale assigned this to themselves",
        ));
        Some(format!("#{id} assigned to you"))
    }
}

/// "Waiting on you" in the support sidebar: (customer, SLA minutes).
pub fn waiting(d: &Desk) -> Vec<(usize, i32)> {
    let mut v: Vec<(usize, i32)> = d
        .tickets
        .iter()
        .filter(|t| t.status == TStatus::Open && t.who == Some(0))
        .filter_map(|t| t.sla.map(|s| (t.cust, s)))
        .collect();
    v.sort_by_key(|x| x.1);
    v
}

/// Support teams in the sidebar: (name, hue, open tickets).
pub const SP_TEAMS: [(&str, &str, &str); 3] = [
    ("Billing", "amber", "3"),
    ("Platform", "blue", "5"),
    ("Onboarding", "teal", "2"),
];

/// The customer's earlier conversations: (subject, ISO date, status).
pub fn previous(c: usize) -> Vec<(&'static str, String, &'static str)> {
    match c % 4 {
        0 => vec![
            ("Add a billing contact", iso(TODAY - 3), "Snoozed"),
            ("Change the account owner", iso(TODAY - 40), "Closed"),
        ],
        1 => vec![
            ("Export of archived projects", iso(TODAY - 9), "Closed"),
            ("Seat pricing for contractors", iso(TODAY - 61), "Closed"),
        ],
        2 => vec![("Bulk invite from CSV", iso(TODAY - 6), "Closed")],
        _ => vec![("Two-factor codes on a new phone", iso(TODAY - 18), "Closed")],
    }
}

// ───────────────────────────────────────────────────────────── help center ──

/// Article collections: (name, hue, icon).
pub const COLLECTIONS: [(&str, &str, &str); 5] = [
    ("Getting started", "teal", "flash"),
    ("Account & billing", "amber", "card"),
    ("Integrations", "violet", "plug"),
    ("Troubleshooting", "rose", "bug"),
    ("Developers", "blue", "term"),
];

/// Article status tabs: (key, label).
pub const ART_TABS: [(&str, &str); 4] = [
    ("all", "All"),
    ("published", "Published"),
    ("draft", "Drafts"),
    ("review", "In review"),
];

/// One help article.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Article {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: &'static str,
    /// Index into [`COLLECTIONS`].
    pub coll: usize,
    /// 1 published, 2 draft, 3 in review (indices into [`ART_TABS`]).
    pub status: usize,
    /// Demo day it was last updated.
    pub updated: i32,
    /// Views in the last 30 days.
    pub views: u32,
    /// Helpful votes.
    pub up: u32,
    /// Not helpful votes.
    pub down: u32,
    /// Owner (index into [`PEOPLE`]).
    pub owner: usize,
    /// Sections: (heading, text).
    pub body: &'static [(&'static str, &'static str)],
    /// Related articles.
    pub related: &'static [u32],
}

#[allow(clippy::too_many_arguments)]
fn art(
    id: u32,
    title: &'static str,
    coll: usize,
    status: usize,
    updated: i32,
    views: u32,
    up: u32,
    down: u32,
    owner: usize,
    body: &'static [(&'static str, &'static str)],
    related: &'static [u32],
) -> Article {
    Article {
        id,
        title,
        coll,
        status,
        updated,
        views,
        up,
        down,
        owner,
        body,
        related,
    }
}

/// The sample articles, or none.
pub fn articles(sample: bool) -> Vec<Article> {
    if !sample {
        return Vec::new();
    }
    vec![
        art(
            1,
            "Set up your workspace",
            0,
            1,
            30,
            4820,
            312,
            14,
            0,
            &[
                (
                    "Before you start",
                    "A workspace holds your projects, people and settings. You need an email address you can confirm; everything else can change later.",
                ),
                (
                    "Create it",
                    "Choose New workspace from the workspace menu, give it a name and pick a URL. The URL can be changed once a month under Settings, General.",
                ),
                (
                    "Next steps",
                    "Invite your team, import existing work from a spreadsheet and turn on the integrations you use every day.",
                ),
            ],
            &[2, 3],
        ),
        art(
            2,
            "Invite your team",
            0,
            1,
            36,
            3110,
            201,
            9,
            4,
            &[
                (
                    "Invite by email",
                    "Open People, choose Invite and paste one address per line. Invites stay valid for 14 days and can be resent from the pending list.",
                ),
                (
                    "Roles",
                    "Members can create and edit work; guests see only the projects you share with them; admins manage billing and settings.",
                ),
                (
                    "Bulk invite",
                    "Upload a CSV with email and role columns to invite up to 500 people at once.",
                ),
            ],
            &[1, 5],
        ),
        art(
            3,
            "Import from a spreadsheet",
            0,
            1,
            24,
            1980,
            96,
            11,
            2,
            &[
                (
                    "Supported files",
                    "CSV and XLSX files up to 50 MB. The first row must hold the column names.",
                ),
                (
                    "Map your columns",
                    "After upload, match each column to a field. Unmatched columns are kept as custom fields so nothing is lost.",
                ),
                (
                    "Check the result",
                    "The import runs in the background; you get an email with the number of rows created and any rows that were skipped.",
                ),
            ],
            &[1, 12],
        ),
        art(
            4,
            "Keyboard shortcuts",
            0,
            1,
            18,
            2240,
            188,
            3,
            7,
            &[
                (
                    "Everywhere",
                    "Press ⌘K to search, ? to see every shortcut, and G then I to jump to the inbox.",
                ),
                (
                    "Lists",
                    "J and K move through a list, X selects, E resolves and H snoozes.",
                ),
            ],
            &[1],
        ),
        art(
            5,
            "Change your plan",
            1,
            1,
            38,
            2790,
            140,
            22,
            9,
            &[
                (
                    "Upgrade",
                    "Open Settings, Billing and choose a plan. Upgrades apply at once and the difference is prorated to the day.",
                ),
                (
                    "Downgrade",
                    "Downgrades take effect at the end of the billing period. Features above the new plan become read-only rather than deleted.",
                ),
                (
                    "Cancel your subscription",
                    "Choose Cancel subscription at the bottom of the Billing page. Your workspace stays readable for 30 days and can be restarted any time.",
                ),
            ],
            &[6, 8],
        ),
        art(
            6,
            "Download invoices",
            1,
            1,
            33,
            1650,
            98,
            6,
            9,
            &[
                (
                    "Where to find them",
                    "Settings, Billing, Invoices lists every invoice with its date and amount. Each one downloads as a PDF.",
                ),
                (
                    "Tax details",
                    "Your company name, address and tax ID print on every invoice issued after you save them under Billing details.",
                ),
            ],
            &[5, 7],
        ),
        art(
            7,
            "Update a payment method",
            1,
            3,
            41,
            890,
            31,
            5,
            9,
            &[
                (
                    "Cards",
                    "Add a new card under Settings, Billing, Payment methods and make it the default. The old card is removed after the next successful charge.",
                ),
                (
                    "Invoicing",
                    "Annual plans over 20 seats can pay by bank transfer against an invoice.",
                ),
            ],
            &[6],
        ),
        art(
            8,
            "Refunds and credits",
            1,
            1,
            12,
            1320,
            77,
            19,
            9,
            &[
                (
                    "Duplicate charges",
                    "Duplicate charges are refunded in full. Write to support with the receipt numbers and we'll reverse the second one.",
                ),
                (
                    "Credits",
                    "Unused time after a downgrade is kept as credit and applied to your next invoices.",
                ),
            ],
            &[5],
        ),
        art(
            9,
            "Connect Slack",
            2,
            1,
            27,
            2410,
            166,
            8,
            2,
            &[
                (
                    "Install",
                    "Open Integrations, choose Slack and approve the permissions. Pick the channels that may receive notifications.",
                ),
                (
                    "Notifications",
                    "Each project can post to its own channel. Mentions in Slack threads sync back as comments.",
                ),
            ],
            &[10],
        ),
        art(
            10,
            "Sync with Google Calendar",
            2,
            1,
            22,
            1870,
            120,
            17,
            5,
            &[
                (
                    "Connect",
                    "Under Integrations, Google Calendar, sign in with Google and pick the calendars to sync.",
                ),
                (
                    "Events appear twice",
                    "When two connected calendars hold the same events, turn off sync for one of them. Duplicates disappear within five minutes.",
                ),
            ],
            &[9, 13],
        ),
        art(
            11,
            "Two-way Stripe sync",
            2,
            2,
            40,
            0,
            0,
            0,
            2,
            &[(
                "What syncs",
                "Customers, plans and invoices from Stripe appear on the matching companies. Status changes flow back to Stripe.",
            )],
            &[6],
        ),
        art(
            12,
            "Exports time out",
            3,
            1,
            41,
            3560,
            210,
            41,
            1,
            &[
                (
                    "Why it happens",
                    "Exports run for up to 60 seconds. Very large tables (over about 10,000 rows) can take longer.",
                ),
                (
                    "Work around it",
                    "Split the export by month with the date filter, or schedule it under Exports so it runs in the background and arrives by email.",
                ),
                (
                    "What's next",
                    "Background exports for every table ship in version 4.18 on 2026-11-16.",
                ),
            ],
            &[3, 15],
        ),
        art(
            13,
            "Fix sign-in loops",
            3,
            1,
            37,
            2980,
            180,
            26,
            4,
            &[
                (
                    "Clear old sessions",
                    "A sign-in that returns to the start usually means an old session cookie. Sign out everywhere from your profile, then sign in again.",
                ),
                (
                    "Single sign-on",
                    "With SAML, check that the identity provider sends the relay state back unchanged.",
                ),
            ],
            &[18, 14],
        ),
        art(
            14,
            "Emails not arriving",
            3,
            3,
            42,
            1240,
            64,
            30,
            4,
            &[
                (
                    "Check the address",
                    "Make sure the address on your profile is spelled right and confirmed.",
                ),
                (
                    "Allow our sender",
                    "Add notifications@quillmoor.example to your allow list. Corporate filters sometimes hold mail for up to an hour.",
                ),
            ],
            &[13],
        ),
        art(
            15,
            "API keys and scopes",
            4,
            1,
            29,
            2120,
            150,
            7,
            7,
            &[
                (
                    "Create a key",
                    "Settings, Developers, API keys. Give each key only the scopes it needs; keys can be read-only.",
                ),
                (
                    "Rotate",
                    "Create the new key, deploy it, then revoke the old one. Revoked keys stop working within a minute.",
                ),
            ],
            &[16, 17],
        ),
        art(
            16,
            "Webhooks: retries and signatures",
            4,
            3,
            39,
            960,
            52,
            4,
            7,
            &[
                (
                    "Retries",
                    "Failed deliveries retry eight times over 24 hours with growing gaps. Webhooks don't follow redirects; point them at the final URL.",
                ),
                (
                    "Verify the signature",
                    "Every request carries a signature header; compute it with your signing secret and compare before trusting the body.",
                ),
            ],
            &[15],
        ),
        art(
            17,
            "Rate limits",
            4,
            2,
            35,
            0,
            0,
            0,
            7,
            &[(
                "Limits",
                "Each key may make 600 requests a minute. Responses carry the remaining budget in their headers.",
            )],
            &[15],
        ),
        art(
            18,
            "Single sign-on with SAML",
            1,
            1,
            20,
            1430,
            88,
            12,
            5,
            &[
                (
                    "Requirements",
                    "SAML is available on the Enterprise plan. You need admin access to your identity provider.",
                ),
                (
                    "Set it up",
                    "Under Settings, Security, add the metadata URL from your provider, test with one account, then require it for everyone.",
                ),
            ],
            &[13],
        ),
    ]
}

/// Articles with tab `t` (index into [`ART_TABS`]) and collection `coll`.
pub fn art_list(v: &[Article], t: usize, coll: Option<usize>) -> Vec<Article> {
    v.iter()
        .filter(|a| t == 0 || a.status == t)
        .filter(|a| coll.is_none_or(|c| a.coll == c))
        .cloned()
        .collect()
}

/// Articles per tab.
pub fn tab_counts(v: &[Article], coll: Option<usize>) -> [usize; 4] {
    let mut n = [0; 4];
    for (t, slot) in n.iter_mut().enumerate() {
        *slot = art_list(v, t, coll).len();
    }
    n
}

/// One search hit: the article id and a snippet with the match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Article id.
    pub id: u32,
    /// The sentence that matched (the title when only the title did).
    pub snippet: String,
    /// The matched word in the snippet: (start, end) byte offsets.
    pub mark: Option<(usize, usize)>,
}

/// Searches titles and bodies for every word of `q` (case-insensitive). Title matches rank
/// first, then views.
pub fn search(v: &[Article], q: &str) -> Vec<Hit> {
    let words: Vec<String> = q
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(u32, u32, Hit)> = Vec::new();
    for a in v {
        let body: String = a
            .body
            .iter()
            .map(|(h, t)| format!("{h}. {t}"))
            .collect::<Vec<_>>()
            .join(" ");
        let all = format!("{} {}", a.title, body).to_lowercase();
        if !words.iter().all(|w| all.contains(w.as_str())) {
            continue;
        }
        let in_title = words
            .iter()
            .filter(|w| a.title.to_lowercase().contains(w.as_str()))
            .count() as u32;
        // the first sentence holding the first word
        let w0 = &words[0];
        let sentence = body
            .split_inclusive(['.', '?'])
            .map(str::trim)
            .find(|s| s.to_lowercase().contains(w0.as_str()))
            .map(str::to_string);
        let snippet = sentence.unwrap_or_else(|| a.title.to_string());
        let mark = snippet
            .to_lowercase()
            .find(w0.as_str())
            .map(|i| (i, i + w0.len()))
            .filter(|(a, b)| snippet.is_char_boundary(*a) && snippet.is_char_boundary(*b));
        hits.push((
            in_title,
            a.views,
            Hit {
                id: a.id,
                snippet,
                mark,
            },
        ));
    }
    hits.sort_by_key(|(t, v, h)| (std::cmp::Reverse(*t), std::cmp::Reverse(*v), h.id));
    hits.into_iter().map(|x| x.2).collect()
}

/// Top searches on the help site in the last 30 days: (query, searches).
pub const SEARCHES: [(&str, u32); 5] = [
    ("cancel subscription", 412),
    ("export", 377),
    ("sso", 221),
    ("dark mode", 184),
    ("delete account", 126),
];

/// Reasons for a "No" vote.
pub const REASONS: [&str; 4] = [
    "Out of date",
    "Hard to follow",
    "Missing steps",
    "Didn't answer my question",
];

/// The share of helpful votes, in percent.
pub fn helpful(a: &Article) -> u32 {
    (a.up * 100).checked_div(a.up + a.down).unwrap_or(0)
}

/// The help center's sidebar nav: (key, icon, label, count).
pub const HC_NAV: [(&str, &str, &str, &str); 4] = [
    ("articles", "doc", "Articles", ""),
    ("site", "web", "Help site", ""),
    ("insights", "chart", "Search insights", "5"),
    ("feedback", "star", "Feedback", "6"),
];

/// The help center's live state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Help {
    /// Every article.
    pub arts: Vec<Article>,
    /// The open article.
    pub open: Option<u32>,
    /// Index into [`ART_TABS`].
    pub tab: usize,
    /// Collection filter.
    pub coll: Option<usize>,
    /// The search.
    pub query: String,
    /// This reader's vote on the open article.
    pub vote: Option<bool>,
    /// The picked reason after a "No".
    pub reason: Option<usize>,
    /// The feedback was sent.
    pub sent: bool,
}

impl Help {
    /// The sample help center (or an empty one) with the most read article open.
    pub fn new(sample: bool) -> Self {
        let arts = articles(sample);
        let open = arts.iter().max_by_key(|a| a.views).map(|a| a.id);
        Self {
            arts,
            open,
            tab: 0,
            coll: None,
            query: String::new(),
            vote: None,
            reason: None,
            sent: false,
        }
    }

    /// The open article.
    pub fn current(&self) -> Option<&Article> {
        self.open
            .and_then(|id| self.arts.iter().find(|a| a.id == id))
    }

    /// Opens article `id` (clears this reader's vote).
    pub fn open_id(&mut self, id: u32) -> Option<String> {
        let a = self.arts.iter().find(|a| a.id == id)?;
        let t = a.title.to_string();
        self.open = Some(id);
        self.vote = None;
        self.reason = None;
        self.sent = false;
        Some(t)
    }

    /// Votes on the open article.
    pub fn vote(&mut self, up: bool) -> Option<String> {
        let id = self.open?;
        if self.vote.is_some() {
            return None;
        }
        let a = self.arts.iter_mut().find(|a| a.id == id)?;
        if up {
            a.up += 1;
        } else {
            a.down += 1;
        }
        self.vote = Some(up);
        Some(if up {
            "Thanks for the feedback".to_string()
        } else {
            "What went wrong?".to_string()
        })
    }

    /// Sends the "No" feedback with its reason.
    pub fn send_feedback(&mut self) -> Result<String, &'static str> {
        if self.vote != Some(false) {
            return Err("Vote first");
        }
        let r = self.reason.ok_or("Pick a reason")?;
        self.sent = true;
        let owner = self
            .current()
            .map(|a| PEOPLE[a.owner % PEOPLE.len()].0)
            .unwrap_or("the owner");
        Ok(format!("Sent to {owner}: {}", REASONS[r]))
    }

    /// Publishes the open article when it is a draft or in review.
    pub fn publish(&mut self) -> Option<String> {
        let id = self.open?;
        let a = self.arts.iter_mut().find(|a| a.id == id)?;
        if a.status == 1 {
            return None;
        }
        a.status = 1;
        a.updated = TODAY;
        Some(format!("Published: {}", a.title))
    }

    /// The list as shown: search hits when there is a query, else the tab and collection.
    pub fn shown(&self) -> Vec<(Article, Option<Hit>)> {
        if self.query.trim().is_empty() {
            art_list(&self.arts, self.tab, self.coll)
                .into_iter()
                .map(|a| (a, None))
                .collect()
        } else {
            let pool = art_list(&self.arts, self.tab, self.coll);
            search(&pool, &self.query)
                .into_iter()
                .filter_map(|h| {
                    pool.iter()
                        .find(|a| a.id == h.id)
                        .cloned()
                        .map(|a| (a, Some(h)))
                })
                .collect()
        }
    }
}

/// Articles to check this quarter (in review or not updated in 30 days) and the share done.
pub fn quarter(v: &[Article]) -> (usize, u32) {
    let check = v
        .iter()
        .filter(|a| a.status == 3 || (a.status == 1 && TODAY - a.updated > 20))
        .count();
    let pct = if v.is_empty() {
        0
    } else {
        ((v.len() - check) * 100 / v.len()) as u32
    };
    (check, pct)
}

// ───────────────────────────────────────────────────────────── live state ──

/// The live state of the five S13 screens a GPUI view keeps (a host routes press ids to
/// [`Arc2Ui::press`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arc2Ui {
    /// Calendar events.
    pub cal: Vec<Event>,
    /// Calendar view (index into [`VIEWS`]).
    pub cview: usize,
    /// The day the calendar shows (and the month's selected day).
    pub cday: i32,
    /// Hidden calendars.
    pub hidden: [bool; 6],
    /// The picked event (keys and drops act on it).
    pub cpick: Option<u32>,
    /// The inbox.
    pub tri: Triage,
    /// The support desk.
    pub desk: Desk,
    /// The help center.
    pub help: Help,
    /// The last thing done, said back.
    pub said: Option<String>,
}

impl Default for Arc2Ui {
    fn default() -> Self {
        Self {
            cal: events(true),
            cview: 1,
            cday: TODAY,
            hidden: [false; 6],
            cpick: None,
            tri: Triage::new(true),
            desk: Desk::new(true),
            help: Help::new(true),
            said: None,
        }
    }
}

fn nth(s: &str, prefix: &str) -> Option<usize> {
    s.strip_prefix(prefix)?.parse().ok()
}

impl Arc2Ui {
    /// Handles a press id of the S13 screens; false when the id is not one of them. Ids:
    /// - calendar: `cal-view-<n>`, `cal-prev`, `cal-next`, `cal-today`, `cal-day-<0..41>` (a
    ///   cell of the month grid, or a weekday column of the week), `cal-pick-<id>`,
    ///   `cal-grab-<id>` (picks, for a drag), `cal-earlier|later|up|down|longer|shorter`,
    ///   `cal-cal-<n>` (show / hide), `cal-new`,
    ///   `cal-focus`;
    /// - inbox: `ib-open-<n>` (n-th of the list), `ib-sel-<n>`, `ib-all`, `ib-j|k|e|h|x|u|a`,
    ///   `ib-snooze`, `ib-snooze-<n>`, `ib-folder-<n>`, `ib-tab-<n>`, `ib-sort`;
    /// - support: `sp-open-<n>`, `sp-view-<n>`, `sp-mode` (or `sp-reply`, `sp-note`), `sp-saved-<n>`, `sp-article`,
    ///   `sp-attach`, `sp-send`, `sp-close`, `sp-snooze-<n>`, `sp-take`, `sp-card-<n>`;
    /// - help: `hc-open-<n>` (n-th of the list), `hc-art-<id>`, `hc-tab-<n>`, `hc-coll-<n>`,
    ///   `hc-coll-all`, `hc-yes`, `hc-no`, `hc-reason-<n>`, `hc-send`, `hc-publish`, `hc-q-<n>`
    ///   (a top search), `hc-clear`.
    pub fn press(&mut self, id: &str) -> bool {
        let out: Option<String> = if id.starts_with("cal-") {
            self.cal_press(id)
        } else if id.starts_with("ib-") {
            self.ib_press(id)
        } else if id.starts_with("sp-") {
            self.sp_press(id)
        } else if id.starts_with("hc-") {
            self.hc_press(id)
        } else {
            return false;
        };
        if let Some(s) = out {
            self.said = Some(s);
        }
        true
    }

    fn cal_press(&mut self, id: &str) -> Option<String> {
        if let Some(n) = nth(id, "cal-view-") {
            self.cview = n.min(2);
            return Some(format!("{} view", VIEWS[self.cview].1));
        }
        if let Some(n) = nth(id, "cal-day-") {
            let d = if self.cview == 2 {
                month_cells(self.cday)[n.min(41)]
            } else {
                week_start(self.cday) + n.min(6) as i32
            };
            if let Some(e) = self.cpick
                && self.cview != 2
            {
                let start = self.cal.iter().find(|x| x.id == e).map(|x| x.start)?;
                if move_event(&mut self.cal, e, d, start) {
                    let x = self.cal.iter().find(|x| x.id == e)?;
                    return Some(format!("{} moved to {} {}", x.title, iso(d), x.span()));
                }
                return None;
            }
            self.cday = d;
            return Some(period_title(0, d));
        }
        if let Some(n) = nth(id, "cal-grab-") {
            self.cpick = Some(n as u32);
            return None;
        }
        if let Some(n) = nth(id, "cal-pick-") {
            let n = n as u32;
            self.cpick = if self.cpick == Some(n) { None } else { Some(n) };
            return self
                .cal
                .iter()
                .find(|e| e.id == n)
                .map(|e| format!("{} · {} {}", e.title, iso(e.day), e.span()));
        }
        if let Some(n) = nth(id, "cal-cal-") {
            let n = n.min(5);
            self.hidden[n] = !self.hidden[n];
            return Some(format!(
                "{} {}",
                CALS[n].1,
                if self.hidden[n] { "hidden" } else { "shown" }
            ));
        }
        match id {
            "cal-prev" | "cal-next" => {
                let dir = if id == "cal-prev" { -1 } else { 1 };
                self.cday = match self.cview {
                    0 => self.cday + dir,
                    2 => month_step(self.cday, dir),
                    _ => self.cday + 7 * dir,
                }
                .clamp(-30, 180);
                Some(period_title(self.cview, self.cday))
            }
            "cal-today" => {
                self.cday = TODAY;
                Some(format!("Today, {}", iso(TODAY)))
            }
            "cal-earlier" | "cal-later" | "cal-up" | "cal-down" => {
                let e = self.cpick?;
                let (dd, dm) = match id {
                    "cal-earlier" => (-1, 0),
                    "cal-later" => (1, 0),
                    "cal-up" => (0, -15),
                    _ => (0, 15),
                };
                nudge_event(&mut self.cal, e, dd, dm)
            }
            "cal-longer" | "cal-shorter" => {
                let e = self.cpick?;
                resize_event(&mut self.cal, e, if id == "cal-longer" { 15 } else { -15 })
            }
            "cal-new" => {
                let d = self.cday;
                let start = free_gaps(&self.cal, d, &self.hidden)
                    .first()
                    .map(|g| g.0)
                    .unwrap_or(WORK.0);
                let e = add_event(&mut self.cal, d, start, 30, "New event", 0);
                self.cpick = Some(e);
                Some(format!("New event {} {}", iso(d), hhmm(start)))
            }
            "cal-focus" => block_focus(&mut self.cal, self.cday, &self.hidden),
            _ => None,
        }
    }

    fn ib_press(&mut self, id: &str) -> Option<String> {
        let t = &mut self.tri;
        if let Some(n) = nth(id, "ib-open-") {
            let c = t.list().get(n)?.clone();
            t.open_id(c.id);
            return Some(format!("{}: {}", CUSTOMERS[c.cust].0, c.subject));
        }
        if let Some(n) = nth(id, "ib-sel-") {
            let c = t.list().get(n)?.id;
            return t.toggle(Some(c));
        }
        if let Some(n) = nth(id, "ib-snooze-") {
            return t.snooze(n);
        }
        if let Some(n) = nth(id, "ib-folder-") {
            return Some(t.folder_to(n));
        }
        if let Some(n) = nth(id, "ib-tab-") {
            return Some(t.tab_to(n));
        }
        match id {
            "ib-all" => Some(t.select_all()),
            "ib-sort" => {
                t.newest = !t.newest;
                Some(
                    if t.newest {
                        "Newest first"
                    } else {
                        "Oldest first"
                    }
                    .to_string(),
                )
            }
            "ib-snooze" => {
                t.snoozing = !t.snoozing;
                None
            }
            _ => {
                let k = id.strip_prefix("ib-")?;
                t.key(k)
            }
        }
    }

    fn sp_press(&mut self, id: &str) -> Option<String> {
        let d = &mut self.desk;
        if let Some(n) = nth(id, "sp-open-") {
            let tk = d.list().get(n)?.clone();
            d.open_id(tk.id);
            return Some(format!("#{} {}", tk.id, tk.subject));
        }
        if let Some(n) = nth(id, "sp-view-") {
            return Some(d.view_to(n));
        }
        if let Some(n) = nth(id, "sp-saved-") {
            return d.saved(n);
        }
        if let Some(n) = nth(id, "sp-snooze-") {
            return d.snooze(n);
        }
        if let Some(n) = nth(id, "sp-card-") {
            d.card = n.min(1);
            return None;
        }
        match id {
            "sp-reply" | "sp-note" => {
                d.note = id == "sp-note";
                Some(if d.note { "Internal note" } else { "Reply" }.to_string())
            }
            "sp-mode" => {
                d.note = !d.note;
                Some(if d.note { "Internal note" } else { "Reply" }.to_string())
            }
            "sp-article" => {
                d.article = Some(12);
                Some("Article attached: Exports time out".to_string())
            }
            "sp-attach" => Some(d.attach()),
            "sp-send" => Some(match d.send() {
                Ok(s) => s,
                Err(e) => e.to_string(),
            }),
            "sp-close" => d.close(),
            "sp-take" => d.take(),
            _ => None,
        }
    }

    fn hc_press(&mut self, id: &str) -> Option<String> {
        let h = &mut self.help;
        if let Some(n) = nth(id, "hc-open-") {
            let a = h.shown().get(n)?.0.id;
            return h.open_id(a);
        }
        if let Some(n) = nth(id, "hc-art-") {
            return h.open_id(n as u32);
        }
        if let Some(n) = nth(id, "hc-tab-") {
            h.tab = n.min(3);
            return Some(format!("{}: {}", ART_TABS[h.tab].1, h.shown().len()));
        }
        if let Some(n) = nth(id, "hc-coll-") {
            let n = n.min(COLLECTIONS.len() - 1);
            h.coll = if h.coll == Some(n) { None } else { Some(n) };
            return Some(match h.coll {
                Some(c) => COLLECTIONS[c].0.to_string(),
                None => "All collections".to_string(),
            });
        }
        if let Some(n) = nth(id, "hc-reason-") {
            h.reason = Some(n.min(REASONS.len() - 1));
            return None;
        }
        if let Some(n) = nth(id, "hc-q-") {
            h.query = SEARCHES[n.min(SEARCHES.len() - 1)].0.to_string();
            return Some(format!("{} results", h.shown().len()));
        }
        match id {
            "hc-coll-all" => {
                h.coll = None;
                Some("All collections".to_string())
            }
            "hc-yes" => h.vote(true),
            "hc-no" => h.vote(false),
            "hc-send" => Some(match h.send_feedback() {
                Ok(s) => s,
                Err(e) => e.to_string(),
            }),
            "hc-publish" => h.publish(),
            "hc-clear" => {
                h.query.clear();
                Some("Search cleared".to_string())
            }
            _ => None,
        }
    }

    /// A triage or calendar key from a host's key handler (`j`, `k`, `e`, `h`, `x`, `u`, `a` on
    /// the inbox). True when handled.
    pub fn key(&mut self, slug: &str, k: &str) -> bool {
        let out = match slug {
            "inbox-triage" => self.tri.key(k),
            _ => None,
        };
        match out {
            Some(s) => {
                self.said = Some(s);
                true
            }
            None => false,
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
        /// `cal-day-0` … `cal-day-41`.
        CAL_DAY, "cal-day-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41
    );
    table!(
        /// `cal-pick-0` … `cal-pick-59` (by event id).
        CAL_PICK, "cal-pick-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
    );
    table!(
        /// `cal-grab-0` … `cal-grab-59` (by event id).
        CAL_GRAB, "cal-grab-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
    );
    table!(
        /// `cal-view-0` … `cal-view-2`.
        CAL_VIEW, "cal-view-", 0, 1, 2
    );
    table!(
        /// `cal-cal-0` … `cal-cal-5`.
        CAL_CAL, "cal-cal-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `ib-open-0` … `ib-open-15`.
        IB_OPEN, "ib-open-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `ib-sel-0` … `ib-sel-15`.
        IB_SEL, "ib-sel-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `ib-folder-0` … `ib-folder-4`.
        IB_FOLDER, "ib-folder-", 0, 1, 2, 3, 4
    );
    table!(
        /// `ib-tab-0` … `ib-tab-2`.
        IB_TAB, "ib-tab-", 0, 1, 2
    );
    table!(
        /// `ib-snooze-0` … `ib-snooze-2`.
        IB_SNOOZE, "ib-snooze-", 0, 1, 2
    );
    table!(
        /// `sp-open-0` … `sp-open-9`.
        SP_OPEN, "sp-open-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    );
    table!(
        /// `sp-view-0` … `sp-view-5`.
        SP_VIEW, "sp-view-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `sp-saved-0` … `sp-saved-3`.
        SP_SAVED, "sp-saved-", 0, 1, 2, 3
    );
    table!(
        /// `sp-snooze-0` … `sp-snooze-2`.
        SP_SNOOZE, "sp-snooze-", 0, 1, 2
    );
    table!(
        /// `sp-card-0` … `sp-card-1`.
        SP_CARD, "sp-card-", 0, 1
    );
    table!(
        /// `hc-open-0` … `hc-open-19`.
        HC_OPEN, "hc-open-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
    );
    table!(
        /// `hc-art-0` … `hc-art-19` (by article id).
        HC_ART, "hc-art-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
    );
    table!(
        /// `hc-tab-0` … `hc-tab-3`.
        HC_TAB, "hc-tab-", 0, 1, 2, 3
    );
    table!(
        /// `hc-coll-0` … `hc-coll-4`.
        HC_COLL, "hc-coll-", 0, 1, 2, 3, 4
    );
    table!(
        /// `hc-reason-0` … `hc-reason-3`.
        HC_REASON, "hc-reason-", 0, 1, 2, 3
    );
    table!(
        /// `hc-q-0` … `hc-q-4` (the top searches).
        HC_Q, "hc-q-", 0, 1, 2, 3, 4
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s13_dates_weeks_months() {
        assert_eq!(week_start(TODAY), 39);
        assert_eq!(iso(39), "2026-11-09");
        assert_eq!(iso_week(39), 46);
        assert_eq!(iso_week(88), 53, "2026-12-28 is in week 53");
        assert_eq!(iso_week(95), 1, "2027-01-04 starts week 1");
        assert_eq!(month_of(TODAY), (31, "2026-11"));
        let cells = month_cells(TODAY);
        assert_eq!(cells.len(), 42);
        assert_eq!(iso(cells[0]), "2026-10-26");
        assert_eq!(weekday(cells[0]), 0);
        assert_eq!(month_step(TODAY, 1), 61);
        assert_eq!(month_step(TODAY, -1), 0);
        assert_eq!(period_title(1, TODAY), "2026-11-09 – 2026-11-15");
        assert_eq!(period_title(0, TODAY), "Thursday 2026-11-12");
        assert_eq!(hhmm(570), "09:30");
        assert_eq!(when(TODAY - 1, 600), "2026-11-11 10:00");
    }

    #[test]
    fn s13_calendar_lanes_moves_gaps() {
        let h = [false; 6];
        let mut v = events(true);
        let tue = day_events(&v, 40, &h);
        let l = lanes(&tue);
        let one = l.iter().find(|x| x.0 == 4).unwrap();
        let sync = l.iter().find(|x| x.0 == 5).unwrap();
        assert_eq!((one.2, sync.2), (2, 2), "1:1 and Launch sync overlap");
        assert_ne!(one.1, sync.1);
        let dentist = l.iter().find(|x| x.0 == 6).unwrap();
        assert_eq!(dentist.2, 1);
        assert!(day_events(&v, 41, &h)[0].all_day, "all-day first");
        let mut hid = h;
        hid[3] = true;
        assert!(!day_events(&v, 41, &hid).iter().any(|e| e.cal == 3));
        assert_eq!(
            nudge_event(&mut v, 4, 1, 15).as_deref(),
            Some("1:1 with Mira · 2026-11-11 10:15–10:45")
        );
        assert_eq!(snap(607), 600);
        assert!(move_event(&mut v, 4, 40, 10 * 60));
        assert!(
            !move_event(&mut v, 4, 40, 10 * 60 + 5),
            "snaps back to the same slot"
        );
        assert_eq!(
            resize_event(&mut v, 4, 30).as_deref(),
            Some("1:1 with Mira · 10:00–11:00")
        );
        assert_eq!(
            resize_event(&mut v, 4, -120).as_deref(),
            Some("1:1 with Mira · 10:00–10:15")
        );
        let g = free_gaps(&v, TODAY, &h);
        assert_eq!(g[0], (9 * 60 + 30, 12 * 60 + 30));
        let s = block_focus(&mut v, TODAY, &h).unwrap();
        assert_eq!(s, "Focus time blocked 2026-11-12 09:30–11:30");
        assert_eq!(free_gaps(&v, TODAY, &h)[0], (11 * 60 + 30, 12 * 60 + 30));
        let n = cal_counts(&v, TODAY);
        assert!(n[0] > 10 && n[3] == 2, "{n:?}");
        let id = add_event(&mut v, 43, 14 * 60 + 7, 50, "  ", 0);
        let e = v.iter().find(|e| e.id == id).unwrap();
        assert_eq!(
            (e.title.as_str(), e.start, e.len),
            ("New event", 14 * 60, 45)
        );
        assert!(busy_minutes(&v, TODAY, &h) > 0);
    }

    #[test]
    fn s13_inbox_triage_keys_and_undo() {
        let mut t = Triage::new(true);
        assert_eq!(t.count(0), 12);
        assert_eq!(t.count(1), 5);
        assert_eq!(t.count(2), 4);
        assert_eq!(t.count(3), 2);
        assert_eq!(t.count(4), 2);
        assert_eq!(t.list()[0].id, 101, "newest first");
        assert_eq!(t.open, Some(101));
        assert_eq!(
            t.key("j").as_deref(),
            Some("Jonah Pike: Can't export more than 10k rows")
        );
        assert_eq!(t.open, Some(102));
        assert!(!t.convs.iter().find(|c| c.id == 102).unwrap().unread);
        assert_eq!(
            t.key("e").as_deref(),
            Some("Resolved: Can't export more than 10k rows")
        );
        assert_eq!(t.open, Some(103), "the next one opens");
        assert_eq!(t.count(0), 11);
        assert_eq!(
            t.key("h").as_deref(),
            Some("Snoozed until 2026-11-12 17:00 (later today)")
        );
        assert_eq!(t.count(3), 3);
        assert!(t.key("u").unwrap().starts_with("Undone"));
        assert_eq!(t.count(3), 2);
        t.toggle(Some(104));
        t.toggle(Some(105));
        assert_eq!(t.resolve().as_deref(), Some("2 conversations resolved"));
        assert!(t.sel.is_empty());
        assert_eq!(t.tab_to(2), "Due soon: 2");
        assert!(t.list().iter().all(|c| c.sla.unwrap() <= 60));
        t.tab_to(0);
        t.newest = false;
        assert_eq!(t.list()[0].id, 112, "oldest first");
        assert_eq!(t.select_all(), "9 selected");
        assert_eq!(t.select_all(), "0 selected");
        assert_eq!(sla_words(-12), "12m late");
        assert_eq!(sla_words(95), "1h 35m");
        assert_eq!(sla_hue(14), "rose");
        assert!(Triage::new(false).key("e").is_none());
    }

    #[test]
    fn s13_support_reply_note_close() {
        let mut d = Desk::new(true);
        assert_eq!(d.count(0), 4);
        assert_eq!(d.count(1), 2);
        assert_eq!(d.count(2), 1);
        assert_eq!(d.count(3), 5);
        assert_eq!(d.count(4), 1);
        assert_eq!(d.count(5), 1);
        assert_eq!(d.list()[0].id, 4809, "late first");
        d.open_id(4812);
        assert_eq!(d.send(), Err("Write a reply first"));
        assert_eq!(d.saved(0).as_deref(), Some("Saved reply: Fix date"));
        assert!(d.draft.starts_with("Hi Jonah, the fix ships"));
        d.attach();
        assert_eq!(d.send().as_deref(), Ok("Reply sent to Jonah Pike"));
        let t = d.current().unwrap();
        assert_eq!(t.status, TStatus::Waiting);
        assert_eq!(
            t.msgs.last().unwrap().files,
            vec!["steps-to-export.pdf".to_string()]
        );
        assert_eq!(t.msgs.last().unwrap().at, "2026-11-12 14:32");
        d.note = true;
        d.draft = "Customer told; watch 4.18.".into();
        assert_eq!(d.send().as_deref(), Ok("Note added"));
        assert!(matches!(
            d.current().unwrap().msgs.last().unwrap().kind,
            Kind::Note(0)
        ));
        assert_eq!(d.close().as_deref(), Some("Ticket closed #4812"));
        assert_eq!(d.count(5), 2);
        assert_eq!(d.close().as_deref(), Some("Ticket reopened #4812"));
        d.open_id(4798);
        assert_eq!(d.take().as_deref(), Some("#4798 assigned to you"));
        assert_eq!(
            d.snooze(1).as_deref(),
            Some("#4798 snoozed until 2026-11-13 09:00")
        );
        assert_eq!(waiting(&d)[0], (3, -12));
        assert_eq!(d.view_to(5), "Closed: 1");
    }

    #[test]
    fn s13_help_search_vote_publish() {
        let mut h = Help::new(true);
        assert_eq!(h.current().unwrap().title, "Set up your workspace");
        let n = tab_counts(&h.arts, None);
        assert_eq!(n[0], 18);
        assert_eq!(n[1] + n[2] + n[3], 18);
        let r = search(&h.arts, "cancel");
        assert_eq!(r[0].id, 5);
        assert!(r[0].snippet.to_lowercase().contains("cancel"));
        let (a, b) = r[0].mark.unwrap();
        assert_eq!(r[0].snippet[a..b].to_lowercase(), "cancel");
        assert_eq!(search(&h.arts, "Export")[0].id, 12, "title first");
        assert!(search(&h.arts, "dark mode").is_empty());
        assert!(search(&h.arts, "  ").is_empty());
        h.query = "slack".into();
        assert_eq!(h.shown().len(), 1);
        h.query.clear();
        h.coll = Some(4);
        assert_eq!(h.shown().len(), 3);
        h.open_id(14);
        assert_eq!(h.send_feedback(), Err("Vote first"));
        assert_eq!(h.vote(false).as_deref(), Some("What went wrong?"));
        assert!(h.vote(true).is_none(), "one vote");
        assert_eq!(h.send_feedback(), Err("Pick a reason"));
        h.reason = Some(0);
        assert_eq!(
            h.send_feedback().as_deref(),
            Ok("Sent to Jonas Weber: Out of date")
        );
        assert_eq!(
            h.publish().as_deref(),
            Some("Published: Emails not arriving")
        );
        assert!(h.publish().is_none());
        let (check, pct) = quarter(&h.arts);
        assert!(check > 0 && pct > 0 && pct < 100);
        assert_eq!(helpful(&h.arts[0]), 95);
    }

    #[test]
    fn s13_live_presses() {
        let mut u = Arc2Ui::default();
        assert!(u.press("cal-view-2"));
        assert_eq!(u.cview, 2);
        assert!(u.press("cal-next"));
        assert_eq!(u.cday, 61);
        assert!(u.press("cal-today"));
        assert!(u.press("cal-view-1"));
        assert!(u.press("cal-pick-9"));
        assert!(u.press("cal-down"));
        assert_eq!(
            u.said.as_deref(),
            Some("Team lunch · 2026-11-12 12:45–13:45")
        );
        assert!(u.press("cal-day-4"));
        assert_eq!(u.cal.iter().find(|e| e.id == 9).unwrap().day, 43);
        assert!(u.press("cal-grab-10"));
        assert_eq!(u.cpick, Some(10));
        assert!(u.press("cal-cal-4"));
        assert!(u.hidden[4]);
        assert!(u.press("ib-j"));
        assert_eq!(u.tri.open, Some(102));
        assert!(u.press("ib-snooze-1"));
        assert_eq!(u.tri.count(3), 3);
        assert!(u.press("sp-saved-2"));
        assert!(u.press("sp-send"));
        assert!(u.said.as_deref().unwrap().starts_with("Reply sent"));
        assert!(u.press("sp-send"));
        assert_eq!(u.said.as_deref(), Some("Write a reply first"));
        assert!(u.press("hc-q-0"));
        assert!(u.press("hc-open-0"));
        assert_eq!(u.help.open, Some(5));
        assert!(u.press("hc-no"));
        assert!(u.press("hc-reason-2"));
        assert!(u.press("hc-send"));
        assert_eq!(
            u.said.as_deref(),
            Some("Sent to Nadia Haddad: Missing steps")
        );
        assert!(!u.press("mx-range-7d"), "not an S13 press");
        assert_eq!(s13_states("help-center").len(), 6);
        assert_eq!(arc2_split("noresults").1, "noresults");
        assert_eq!(arc2_split("loading").0, ScreenState::Loading);
    }
}
