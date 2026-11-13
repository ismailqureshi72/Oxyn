//! Dates and times for the pickers (`calendar.md`, `date-picker.md`, `date-range-picker.md`,
//! `time-picker.md`): a civil [`Date`] with no time zone and no allocation, a [`TimeOfDay`], the
//! month grid with roving focus ([`Calendar`]), the range picker with presets, hover preview and
//! comparison ([`RangePicker`]), the date field ([`DatePicker`]) and the time list
//! ([`time_options`]).
//!
//! Behaviour follows the affectUI Date Range Picker and Arc UI Calendar, Date picker, Date range
//! picker and Time picker docs (rebuilt from the public docs, no source used).

use std::fmt;

use crate::ClockMs;

/// English month names.
pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// English weekday names, Sunday first.
pub const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// A calendar day (proleptic Gregorian, no time zone).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// Year.
    pub y: i32,
    /// Month 1–12.
    pub m: u8,
    /// Day 1–31.
    pub d: u8,
}

/// Whether `y` is a leap year.
pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Days in month `m` of year `y`.
pub fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// A date; `None` when it does not exist (February 30).
    pub fn new(y: i32, m: u8, d: u8) -> Option<Self> {
        ((1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)).then_some(Self { y, m, d })
    }
    /// A date, clamping the day to the month (Jan 31 + 1 month = Feb 28).
    pub fn clamped(y: i32, m: u8, d: u8) -> Self {
        let m = m.clamp(1, 12);
        Self {
            y,
            m,
            d: d.clamp(1, days_in_month(y, m)),
        }
    }
    /// Days since 1970-01-01 (negative before).
    pub fn to_days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let y = i64::from(self.y) - i64::from(self.m <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.m);
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.d) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let y = (yoe + era * 400 + i64::from(m <= 2)) as i32;
        Self { y, m, d }
    }
    /// Weekday, 0 = Sunday.
    pub fn weekday(self) -> u8 {
        (self.to_days() + 4).rem_euclid(7) as u8
    }
    /// `n` days later (earlier when negative).
    pub fn add_days(self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }
    /// `n` months later, clamping the day.
    pub fn add_months(self, n: i32) -> Self {
        let idx = self.y * 12 + i32::from(self.m) - 1 + n;
        Self::clamped(idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u8, self.d)
    }
    /// First day of this month.
    pub fn month_start(self) -> Self {
        Self { d: 1, ..self }
    }
    /// Last day of this month.
    pub fn month_end(self) -> Self {
        Self {
            d: days_in_month(self.y, self.m),
            ..self
        }
    }
    /// Start of the week holding this day.
    pub fn week_start(self, week_starts_on: u8) -> Self {
        let back = (i64::from(self.weekday()) - i64::from(week_starts_on)).rem_euclid(7);
        self.add_days(-back)
    }
    /// Whole days from `self` to `other`.
    pub fn days_until(self, other: Date) -> i64 {
        other.to_days() - self.to_days()
    }
    /// ISO week number (1–53).
    pub fn iso_week(self) -> u8 {
        let wd = (i64::from(self.weekday()) + 6) % 7; // Monday = 0
        let thursday = self.add_days(3 - wd);
        let jan1 = Date {
            y: thursday.y,
            m: 1,
            d: 1,
        };
        (jan1.days_until(thursday) / 7 + 1) as u8
    }
    /// `2026-11-13`.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
    /// Parses `2026-11-13`.
    pub fn parse_iso(s: &str) -> Option<Self> {
        let mut it = s.trim().splitn(3, '-');
        let y = it.next()?.parse().ok()?;
        let m = it.next()?.parse().ok()?;
        let d = it.next()?.parse().ok()?;
        Self::new(y, m, d)
    }
    /// Month name.
    pub fn month_name(self) -> &'static str {
        MONTHS[usize::from(self.m - 1)]
    }
    /// Weekday name.
    pub fn weekday_name(self) -> &'static str {
        WEEKDAYS[usize::from(self.weekday())]
    }
    /// `Friday, November 13, 2026` (each day button's accessible name).
    pub fn long(self) -> String {
        format!(
            "{}, {} {}, {}",
            self.weekday_name(),
            self.month_name(),
            self.d,
            self.y
        )
    }
    /// Formats with `f`.
    pub fn format(self, f: DateFormat) -> String {
        f.apply(self)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.iso())
    }
}

/// How a date is shown in a field (Arc `format`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// `Nov 13, 2026` (default).
    #[default]
    Medium,
    /// `November 13, 2026`.
    Long,
    /// `Fri, Nov 13`.
    Weekday,
    /// `13/11/2026`.
    DayFirst,
    /// `11/13/2026`.
    MonthFirst,
    /// `2026-11-13`.
    Iso,
}

impl DateFormat {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "long" => Self::Long,
            "weekday" => Self::Weekday,
            "day-first" => Self::DayFirst,
            "month-first" => Self::MonthFirst,
            "iso" => Self::Iso,
            _ => Self::Medium,
        }
    }
    /// Formats `d`.
    pub fn apply(self, d: Date) -> String {
        let short = &d.month_name()[..3];
        match self {
            Self::Medium => format!("{short} {}, {}", d.d, d.y),
            Self::Long => format!("{} {}, {}", d.month_name(), d.d, d.y),
            Self::Weekday => format!("{}, {short} {}", &d.weekday_name()[..3], d.d),
            Self::DayFirst => format!("{:02}/{:02}/{}", d.d, d.m, d.y),
            Self::MonthFirst => format!("{:02}/{:02}/{}", d.m, d.d, d.y),
            Self::Iso => d.iso(),
        }
    }
    /// The parts of the shown value, so a picker can roll only the parts that changed.
    pub fn parts(self, d: Date) -> Vec<String> {
        let s = self.apply(d);
        let mut out = vec![];
        let mut cur = String::new();
        for ch in s.chars() {
            let sep = matches!(ch, ' ' | ',' | '/' | '-');
            if sep {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(ch.to_string());
            } else {
                cur.push(ch);
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        out
    }
}

/// Oxyn plus: natural-language entry for the date field. Understands `today`, `tomorrow`,
/// `yesterday`, `in 3 days`, `in 2 weeks`, `next monday` / `mon`, `3 days ago`, ISO dates and
/// `Nov 13` / `13 Nov` (this year).
pub fn parse_natural(input: &str, today: Date) -> Option<Date> {
    let s = input.trim().to_lowercase();
    if let Some(d) = Date::parse_iso(&s) {
        return Some(d);
    }
    match s.as_str() {
        "today" | "now" => return Some(today),
        "tomorrow" | "tmr" => return Some(today.add_days(1)),
        "yesterday" => return Some(today.add_days(-1)),
        _ => {}
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    let unit = |w: &str| -> Option<i64> {
        match w.trim_end_matches('s') {
            "day" => Some(1),
            "week" => Some(7),
            _ => None,
        }
    };
    if let ["in", n, u] = words.as_slice() {
        return Some(today.add_days(n.parse::<i64>().ok()? * unit(u)?));
    }
    if let [n, u, "ago"] = words.as_slice() {
        return Some(today.add_days(-n.parse::<i64>().ok()? * unit(u)?));
    }
    let weekday = |w: &str| {
        WEEKDAYS
            .iter()
            .position(|d| w.len() >= 3 && d.to_lowercase().starts_with(w))
    };
    let target = match words.as_slice() {
        ["next", w] => weekday(w),
        [w] => weekday(w),
        _ => None,
    };
    if let Some(t) = target {
        let ahead = (t as i64 - i64::from(today.weekday())).rem_euclid(7);
        return Some(today.add_days(if ahead == 0 { 7 } else { ahead }));
    }
    let month = |w: &str| {
        MONTHS
            .iter()
            .position(|m| w.len() >= 3 && m.to_lowercase().starts_with(w))
    };
    let (mw, dw) = match words.as_slice() {
        [a, b] if a.parse::<u8>().is_ok() => (*b, *a),
        [a, b] => (*a, *b),
        _ => return None,
    };
    let m = month(mw)? as u8 + 1;
    Date::new(today.y, m, dw.trim_end_matches(',').parse().ok()?)
}

/// Which way the shown value moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Travel {
    /// Later (rises from below / slides from the right).
    Forward,
    /// Earlier (drops from above / slides from the left).
    Back,
}

impl Travel {
    /// From two comparable values.
    pub fn between<T: PartialOrd>(from: T, to: T) -> Self {
        if to < from { Self::Back } else { Self::Forward }
    }
    /// `later` / `earlier` (CSS hook).
    pub fn id(self) -> &'static str {
        match self {
            Self::Forward => "later",
            Self::Back => "earlier",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Calendar

/// Keys the day grid understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalKey {
    /// ←.
    Left,
    /// →.
    Right,
    /// ↑ (a week back).
    Up,
    /// ↓ (a week on).
    Down,
    /// Start of the week.
    Home,
    /// End of the week.
    End,
    /// Previous month (`year` with Shift).
    PageUp {
        /// Shift held: a year.
        year: bool,
    },
    /// Next month (`year` with Shift).
    PageDown {
        /// Shift held: a year.
        year: bool,
    },
}

/// One cell of the month grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayCell {
    /// The day.
    pub date: Date,
    /// In the shown month (others are the leading / trailing days).
    pub in_month: bool,
    /// Today (`aria-current="date"`).
    pub today: bool,
    /// Picked.
    pub selected: bool,
    /// Blocked (outside min / max or by the rule).
    pub disabled: bool,
    /// Holds the roving tab stop.
    pub focused: bool,
}

/// Which days are blocked besides min / max.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Blocked {
    /// None.
    #[default]
    None,
    /// Saturdays and Sundays.
    Weekends,
    /// These days.
    Dates(Vec<Date>),
}

/// The month view (Arc Calendar): one picked day, a visible month, roving focus that skips blocked
/// days and crosses months, min / max with disabled pagers, a Today button and the slide direction
/// for month changes. Oxyn plus: ISO week numbers and a per-day activity level (run-history heat).
#[derive(Clone, Debug, PartialEq)]
pub struct Calendar {
    /// Picked day.
    pub value: Option<Date>,
    /// First day of the visible month.
    pub month: Date,
    /// Focused day (roving tab stop).
    pub focus: Date,
    /// Today.
    pub today: Date,
    /// Earliest selectable day.
    pub min: Option<Date>,
    /// Latest selectable day.
    pub max: Option<Date>,
    /// Blocked days.
    pub blocked: Blocked,
    /// 0 = Sunday, 1 = Monday.
    pub week_starts_on: u8,
    /// Show a Today button.
    pub show_today: bool,
    /// Oxyn plus: show ISO week numbers.
    pub week_numbers: bool,
    /// Last month change: direction and when (slide).
    pub moved: Option<(Travel, ClockMs)>,
}

impl Calendar {
    /// A calendar showing `value`'s month (or today's).
    pub fn new(value: Option<Date>, today: Date) -> Self {
        let at = value.unwrap_or(today);
        Self {
            value,
            month: at.month_start(),
            focus: at,
            today,
            min: None,
            max: None,
            blocked: Blocked::None,
            week_starts_on: 0,
            show_today: false,
            week_numbers: false,
            moved: None,
        }
    }
    /// Whether `d` can be picked.
    pub fn enabled(&self, d: Date) -> bool {
        if self.min.is_some_and(|m| d < m) || self.max.is_some_and(|m| d > m) {
            return false;
        }
        match &self.blocked {
            Blocked::None => true,
            Blocked::Weekends => !matches!(d.weekday(), 0 | 6),
            Blocked::Dates(v) => !v.contains(&d),
        }
    }
    /// Weekday headers in order, as `(short, long)`.
    pub fn weekdays(&self) -> Vec<(&'static str, &'static str)> {
        (0..7)
            .map(|i| {
                let w = WEEKDAYS[usize::from((self.week_starts_on + i) % 7)];
                (&w[..2], w)
            })
            .collect()
    }
    /// The grid: 6 rows of 7 days (fixed height, so the card never jumps).
    pub fn grid(&self) -> Vec<[DayCell; 7]> {
        let start = self.month.week_start(self.week_starts_on);
        (0..6)
            .map(|r| {
                std::array::from_fn(|c| {
                    let date = start.add_days(r * 7 + c as i64);
                    DayCell {
                        date,
                        in_month: date.m == self.month.m,
                        today: date == self.today,
                        selected: self.value == Some(date),
                        disabled: !self.enabled(date),
                        focused: date == self.focus,
                    }
                })
            })
            .collect()
    }
    /// `November 2026`.
    pub fn title(&self) -> String {
        format!("{} {}", self.month.month_name(), self.month.y)
    }
    /// The previous-month button is enabled.
    pub fn can_prev(&self) -> bool {
        self.min.is_none_or(|m| self.month > m.month_start())
    }
    /// The next-month button is enabled.
    pub fn can_next(&self) -> bool {
        self.max.is_none_or(|m| self.month < m.month_start())
    }
    fn show(&mut self, month: Date, now: ClockMs) {
        let month = month.month_start();
        if month != self.month {
            self.moved = Some((Travel::between(self.month, month), now));
            self.month = month;
        }
    }
    /// Pager: `delta` months. Focus moves to the same day in the new month.
    pub fn page(&mut self, delta: i32, now: ClockMs) -> bool {
        if (delta < 0 && !self.can_prev()) || (delta > 0 && !self.can_next()) {
            return false;
        }
        let target = self.month.add_months(delta);
        self.show(target, now);
        self.focus = self.focus.add_months(delta);
        self.clamp_focus();
        true
    }
    fn clamp_focus(&mut self) {
        if let Some(m) = self.min {
            self.focus = self.focus.max(m);
        }
        if let Some(m) = self.max {
            self.focus = self.focus.min(m);
        }
    }
    /// Moves focus by a key, skipping blocked days (up to a year) and following into the next or
    /// previous month. Returns the new focus.
    pub fn key(&mut self, k: CalKey, now: ClockMs) -> Date {
        let ws = self.week_starts_on;
        let (target, step): (Date, i64) = match k {
            CalKey::Left => (self.focus.add_days(-1), -1),
            CalKey::Right => (self.focus.add_days(1), 1),
            CalKey::Up => (self.focus.add_days(-7), -7),
            CalKey::Down => (self.focus.add_days(7), 7),
            CalKey::Home => (self.focus.week_start(ws), 1),
            CalKey::End => (self.focus.week_start(ws).add_days(6), -1),
            CalKey::PageUp { year } => (self.focus.add_months(if year { -12 } else { -1 }), -1),
            CalKey::PageDown { year } => (self.focus.add_months(if year { 12 } else { 1 }), 1),
        };
        let mut d = target;
        let mut tries = 0;
        while !self.enabled(d) && tries < 366 {
            // Step over blocked days in the direction of travel; stop at min / max.
            if self.min.is_some_and(|m| d < m) {
                d = self.min.unwrap_or(d);
                if !self.enabled(d) {
                    d = d.add_days(1);
                }
            } else if self.max.is_some_and(|m| d > m) {
                d = self.max.unwrap_or(d);
                if !self.enabled(d) {
                    d = d.add_days(-1);
                }
            } else {
                d = d.add_days(step.signum());
            }
            tries += 1;
        }
        if self.enabled(d) {
            self.focus = d;
            self.show(d, now);
        }
        self.focus
    }
    /// Picks `d` (click / Enter / Space). Returns it when it changed.
    pub fn select(&mut self, d: Date, now: ClockMs) -> Option<Date> {
        if !self.enabled(d) {
            return None;
        }
        self.focus = d;
        self.show(d, now);
        if self.value == Some(d) {
            return None;
        }
        self.value = Some(d);
        Some(d)
    }
    /// Picks the focused day.
    pub fn select_focused(&mut self, now: ClockMs) -> Option<Date> {
        self.select(self.focus, now)
    }
    /// Today button: shows today's month and picks today when allowed.
    pub fn go_today(&mut self, now: ClockMs) -> Option<Date> {
        let t = self.today;
        self.show(t, now);
        self.focus = t;
        self.select(t, now)
    }
    /// The live-region line for a month change (`November 2026`).
    pub fn announce(&self) -> String {
        self.title()
    }
}

// ---------------------------------------------------------------------------------------------
// Range picker

/// An inclusive range of whole days.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DateRange {
    /// First day.
    pub start: Date,
    /// Last day (≥ start).
    pub end: Date,
}

impl DateRange {
    /// A range; swaps the ends when given backwards.
    pub fn new(a: Date, b: Date) -> Self {
        if b < a {
            Self { start: b, end: a }
        } else {
            Self { start: a, end: b }
        }
    }
    /// Number of days, both ends included.
    pub fn days(self) -> i64 {
        self.start.days_until(self.end) + 1
    }
    /// Contains `d`.
    pub fn contains(self, d: Date) -> bool {
        d >= self.start && d <= self.end
    }
    /// `Oct 15 – Nov 13, 2026` (year once when shared).
    pub fn label(self) -> String {
        let s = self.start;
        let e = self.end;
        if s == e {
            return DateFormat::Medium.apply(s);
        }
        if s.y == e.y {
            format!(
                "{} {} – {} {}, {}",
                &s.month_name()[..3],
                s.d,
                &e.month_name()[..3],
                e.d,
                e.y
            )
        } else {
            format!(
                "{} – {}",
                DateFormat::Medium.apply(s),
                DateFormat::Medium.apply(e)
            )
        }
    }
    /// The comparison range for `mode`.
    pub fn compare(self, mode: Compare) -> Option<DateRange> {
        match mode {
            Compare::Off => None,
            Compare::Previous => {
                let n = self.days();
                Some(DateRange {
                    start: self.start.add_days(-n),
                    end: self.start.add_days(-1),
                })
            }
            Compare::Year => Some(DateRange {
                start: self.start.add_months(-12),
                end: self.end.add_months(-12),
            }),
        }
    }
}

/// Comparison mode (affectUI `compare`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Compare {
    /// No comparison.
    Off,
    /// The same number of days just before.
    #[default]
    Previous,
    /// The same days a year earlier.
    Year,
}

impl Compare {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Previous => "previous",
            Self::Year => "year",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "off" => Self::Off,
            "year" => Self::Year,
            _ => Self::Previous,
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Previous => "Previous period",
            Self::Year => "Previous year",
        }
    }
}

/// A named shortcut.
#[derive(Clone, Copy, Debug)]
pub struct Preset {
    /// Id.
    pub id: &'static str,
    /// Label.
    pub label: &'static str,
    /// The range for `today`.
    pub range: fn(Date) -> DateRange,
}

impl PartialEq for Preset {
    fn eq(&self, o: &Self) -> bool {
        self.id == o.id && self.label == o.label
    }
}

/// The default presets: Today … Year to date. Oxyn plus: Last 90 days and Last 12 months.
pub fn default_presets() -> Vec<Preset> {
    fn r(a: Date, b: Date) -> DateRange {
        DateRange::new(a, b)
    }
    vec![
        Preset {
            id: "today",
            label: "Today",
            range: |t| r(t, t),
        },
        Preset {
            id: "yesterday",
            label: "Yesterday",
            range: |t| r(t.add_days(-1), t.add_days(-1)),
        },
        Preset {
            id: "7d",
            label: "Last 7 days",
            range: |t| r(t.add_days(-6), t),
        },
        Preset {
            id: "30d",
            label: "Last 30 days",
            range: |t| r(t.add_days(-29), t),
        },
        Preset {
            id: "90d",
            label: "Last 90 days",
            range: |t| r(t.add_days(-89), t),
        },
        Preset {
            id: "month",
            label: "This month",
            range: |t| r(t.month_start(), t),
        },
        Preset {
            id: "last-month",
            label: "Last month",
            range: |t| {
                let p = t.add_months(-1);
                r(p.month_start(), p.month_end())
            },
        },
        Preset {
            id: "ytd",
            label: "Year to date",
            range: |t| r(Date { y: t.y, m: 1, d: 1 }, t),
        },
        Preset {
            id: "12m",
            label: "Last 12 months",
            range: |t| r(t.add_months(-12).add_days(1), t),
        },
    ]
}

/// What the band over the grid shows for a day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Band {
    /// Inside the picked (or previewed) range.
    pub inside: bool,
    /// First day of the band.
    pub start: bool,
    /// Last day of the band.
    pub end: bool,
    /// The band is the lighter hover preview (after the first click).
    pub preview: bool,
    /// Inside the comparison range (grey band, outlined ends).
    pub compare: bool,
    /// Comparison start / end (outlined).
    pub compare_edge: bool,
}

/// The range picker (affectUI Date Range Picker + Arc Date range picker): two months (or one),
/// presets, first click sets the start and the band follows the pointer as a preview until the
/// second click, comparison with the previous period or year, Apply / Cancel.
#[derive(Clone, Debug, PartialEq)]
pub struct RangePicker {
    /// The applied range.
    pub value: Option<DateRange>,
    /// The draft range inside the open panel.
    pub draft: Option<DateRange>,
    /// The first click, waiting for the second.
    pub anchor: Option<Date>,
    /// Day under the pointer / keyboard focus (the preview's other end).
    pub hover: Option<Date>,
    /// The grid (left month; the right month is `+1`), focus, limits, week start.
    pub cal: Calendar,
    /// Months shown (1 or 2).
    pub months: u8,
    /// Comparison.
    pub compare: Compare,
    /// Offer the comparison control.
    pub compare_enabled: bool,
    /// Presets.
    pub presets: Vec<Preset>,
    /// Open.
    pub open: bool,
    /// When the panel opened.
    pub opened_at: ClockMs,
}

impl RangePicker {
    /// A picker. `value` defaults to the last 30 days (affectUI) when `None` is not wanted.
    pub fn new(value: Option<DateRange>, today: Date) -> Self {
        let mut cal = Calendar::new(value.map(|r| r.end), today);
        cal.week_starts_on = 1;
        cal.max = Some(today);
        // Show the month before the end on the left, so the end month is on the right.
        cal.month = cal.month.add_months(-1);
        Self {
            value,
            draft: value,
            anchor: None,
            hover: None,
            cal,
            months: 2,
            compare: Compare::Previous,
            compare_enabled: true,
            presets: default_presets(),
            open: false,
            opened_at: ClockMs(0),
        }
    }
    /// Opens the panel with the applied range as the draft.
    pub fn open_panel(&mut self, now: ClockMs) {
        self.open = true;
        self.opened_at = now;
        self.draft = self.value;
        self.anchor = None;
        self.hover = None;
    }
    /// Esc / Cancel: closes without applying.
    pub fn cancel(&mut self) {
        self.open = false;
        self.anchor = None;
        self.hover = None;
        self.draft = self.value;
    }
    /// Apply: commits the draft and closes. Returns the range and its comparison.
    pub fn apply(&mut self) -> Option<(DateRange, Option<DateRange>)> {
        let r = self.draft?;
        self.value = Some(r);
        self.open = false;
        self.anchor = None;
        Some((r, r.compare(self.compare)))
    }
    /// A day click / Enter: the first sets the start, the second finishes the range.
    pub fn pick(&mut self, d: Date, now: ClockMs) -> Option<DateRange> {
        if !self.cal.enabled(d) {
            return None;
        }
        self.cal.focus = d;
        match self.anchor.take() {
            None => {
                self.anchor = Some(d);
                self.hover = Some(d);
                self.draft = Some(DateRange::new(d, d));
                None
            }
            Some(a) => {
                let r = DateRange::new(a, d);
                self.draft = Some(r);
                self.hover = None;
                let _ = now;
                Some(r)
            }
        }
    }
    /// Pointer over / keyboard focus on `d`: moves the preview end.
    pub fn hover(&mut self, d: Option<Date>) {
        self.hover = d.filter(|&d| self.cal.enabled(d));
    }
    /// A preset: fills the draft (and the comparison), shows its end month.
    pub fn preset(&mut self, id: &str, now: ClockMs) -> Option<DateRange> {
        let p = self.presets.iter().find(|p| p.id == id)?;
        let r = (p.range)(self.cal.today);
        self.draft = Some(r);
        self.anchor = None;
        self.hover = None;
        let left = if self.months == 2 {
            r.end.add_months(-1)
        } else {
            r.end
        };
        self.cal.show(left, now);
        self.cal.focus = r.end;
        Some(r)
    }
    /// The preset the draft matches, if any.
    pub fn active_preset(&self) -> Option<&'static str> {
        let d = self.draft?;
        self.presets
            .iter()
            .find(|p| (p.range)(self.cal.today) == d)
            .map(|p| p.id)
    }
    /// The band shown now: the preview while the second click is pending, else the draft.
    pub fn shown(&self) -> Option<(DateRange, bool)> {
        match (self.anchor, self.hover) {
            (Some(a), Some(h)) => Some((DateRange::new(a, h), true)),
            (Some(a), None) => Some((DateRange::new(a, a), true)),
            _ => self.draft.map(|r| (r, false)),
        }
    }
    /// The band for one day.
    pub fn band(&self, d: Date) -> Band {
        let mut b = Band::default();
        if let Some((r, preview)) = self.shown() {
            b.inside = r.contains(d);
            b.start = d == r.start;
            b.end = d == r.end;
            b.preview = preview;
        }
        if self.compare_enabled
            && let Some(c) = self.draft.and_then(|r| r.compare(self.compare))
        {
            b.compare = c.contains(d);
            b.compare_edge = d == c.start || d == c.end;
        }
        b
    }
    /// The month grids shown (left first).
    pub fn grids(&self) -> Vec<(String, Vec<[DayCell; 7]>, u8)> {
        (0..self.months)
            .map(|i| {
                let mut c = self.cal.clone();
                c.month = self.cal.month.add_months(i32::from(i));
                (c.title(), c.grid(), c.month.m)
            })
            .collect()
    }
    /// Keyboard in the grid: moves focus (and the preview once a start is picked).
    pub fn key(&mut self, k: CalKey, now: ClockMs) -> Date {
        let before = self.cal.month;
        let d = self.cal.key(k, now);
        // Keep the focused day visible across two months without jumping the left month needlessly.
        if self.months == 2 && d.month_start() == before.add_months(1) {
            self.cal.month = before;
            self.cal.moved = None;
        }
        if self.anchor.is_some() {
            self.hover = Some(d);
        }
        d
    }
    /// Pager.
    pub fn page(&mut self, delta: i32, now: ClockMs) -> bool {
        self.cal.page(delta, now)
    }
    /// The trigger text.
    pub fn trigger_label(&self, placeholder: &str) -> String {
        self.value
            .map(DateRange::label)
            .unwrap_or_else(|| placeholder.into())
    }
    /// The live-region line: the start, then the range and its length.
    pub fn announce(&self) -> String {
        match (self.anchor, self.draft) {
            (Some(a), _) => format!("Start {}. Pick the end date.", a.long()),
            (None, Some(r)) => format!(
                "{}, {} day{}",
                r.label(),
                r.days(),
                if r.days() == 1 { "" } else { "s" }
            ),
            _ => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Date picker

/// The date field (Arc Date picker): a trigger that opens the calendar in a popover, shows the
/// value with per-part rolling, and closes about 240 ms after a pick so the highlight can land.
/// Oxyn plus: natural-language entry and relative presets.
#[derive(Clone, Debug, PartialEq)]
pub struct DatePicker {
    /// The calendar (holds the value).
    pub cal: Calendar,
    /// Open.
    pub open: bool,
    /// Value format.
    pub format: DateFormat,
    /// When the popover should close after a pick.
    pub close_at: Option<ClockMs>,
    /// The previous value (per-part roll direction).
    pub prev: Option<Date>,
    /// Typed text (natural-language entry).
    pub typed: String,
}

/// Delay between a pick and the popover closing.
pub const PICK_CLOSE_MS: u64 = 240;

impl DatePicker {
    /// A picker.
    pub fn new(value: Option<Date>, today: Date) -> Self {
        Self {
            cal: Calendar::new(value, today),
            open: false,
            format: DateFormat::Medium,
            close_at: None,
            prev: None,
            typed: String::new(),
        }
    }
    /// The value.
    pub fn value(&self) -> Option<Date> {
        self.cal.value
    }
    /// Opens, focusing the picked or today's day.
    pub fn open_popover(&mut self) {
        self.open = true;
        self.close_at = None;
        let at = self.cal.value.unwrap_or(self.cal.today);
        self.cal.focus = at;
        self.cal.month = at.month_start();
        self.cal.moved = None;
    }
    /// Esc / outside click.
    pub fn close(&mut self) {
        self.open = false;
        self.close_at = None;
    }
    /// A pick: sets the value and schedules the close (`reduced` closes at once).
    pub fn pick(&mut self, d: Date, now: ClockMs, reduced: bool) -> Option<Date> {
        let before = self.cal.value;
        let r = self.cal.select(d, now)?;
        self.prev = before;
        if reduced {
            self.close();
        } else {
            self.close_at = Some(ClockMs(now.0 + PICK_CLOSE_MS));
        }
        Some(r)
    }
    /// Advances the close timer; returns `true` when it closed now.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if self.close_at.is_some_and(|t| now >= t) {
            self.close();
            return true;
        }
        false
    }
    /// Clear button.
    pub fn clear(&mut self) {
        self.prev = self.cal.value.take();
    }
    /// Natural-language entry; picks the parsed day.
    pub fn enter_text(&mut self, text: &str, now: ClockMs) -> Option<Date> {
        self.typed = text.into();
        let d = parse_natural(text, self.cal.today)?;
        let before = self.cal.value;
        self.cal.select(d, now).inspect(|_| self.prev = before)
    }
    /// The shown parts with a flag per part that changed (those roll).
    pub fn parts(&self) -> Vec<(String, bool)> {
        let Some(v) = self.cal.value else {
            return vec![];
        };
        let now = self.format.parts(v);
        let before = self.prev.map(|p| self.format.parts(p)).unwrap_or_default();
        now.into_iter()
            .enumerate()
            .map(|(i, p)| {
                let changed = before.get(i) != Some(&p);
                (p, changed && self.prev.is_some())
            })
            .collect()
    }
    /// Roll direction of the last change.
    pub fn travel(&self) -> Travel {
        match (self.prev, self.cal.value) {
            (Some(a), Some(b)) => Travel::between(a, b),
            _ => Travel::Forward,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Time

/// A time of day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeOfDay {
    /// Hour 0–23.
    pub h: u8,
    /// Minute 0–59.
    pub m: u8,
}

impl TimeOfDay {
    /// A time (wraps into the day).
    pub fn new(h: u32, m: u32) -> Self {
        let t = (h * 60 + m) % 1440;
        Self {
            h: (t / 60) as u8,
            m: (t % 60) as u8,
        }
    }
    /// Minutes from midnight.
    pub fn minutes(self) -> u32 {
        u32::from(self.h) * 60 + u32::from(self.m)
    }
    /// `HH:mm` (the value format).
    pub fn hhmm(self) -> String {
        format!("{:02}:{:02}", self.h, self.m)
    }
    /// Parses `HH:mm`, `9:30`, `9:30 pm`, `9pm`, `21`.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase().replace(' ', "");
        let (body, pm) = if let Some(b) = s.strip_suffix("pm") {
            (b.to_string(), Some(true))
        } else if let Some(b) = s.strip_suffix("am") {
            (b.to_string(), Some(false))
        } else {
            (s.clone(), None)
        };
        let (h, m) = match body.split_once(':') {
            Some((h, m)) => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?),
            None => (body.parse::<u32>().ok()?, 0),
        };
        if m > 59 {
            return None;
        }
        let h = match pm {
            Some(pm) if (1..=12).contains(&h) => (h % 12) + if pm { 12 } else { 0 },
            Some(_) => return None,
            None if h < 24 => h,
            None => return None,
        };
        Some(Self::new(h, m))
    }
    /// Shown text for `cycle` (12 → `9:30 AM`, 24 → `09:30`).
    pub fn label(self, cycle: HourCycle) -> String {
        match cycle {
            HourCycle::H24 => self.hhmm(),
            HourCycle::H12 => {
                let h = match self.h % 12 {
                    0 => 12,
                    h => h,
                };
                format!(
                    "{h}:{:02} {}",
                    self.m,
                    if self.h < 12 { "AM" } else { "PM" }
                )
            }
        }
    }
}

/// 12- or 24-hour clock.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HourCycle {
    /// `9:30 AM`.
    #[default]
    H12,
    /// `09:30`.
    H24,
}

impl HourCycle {
    /// From `12` / `24` / `12h` / `24h`.
    pub fn from_id(s: &str) -> Self {
        if s.starts_with("24") {
            Self::H24
        } else {
            Self::H12
        }
    }
}

/// The time list (Arc Time picker): every `step` minutes, as `(value "HH:mm", label)`. Oxyn plus:
/// a window (`from..=to`, e.g. working hours) that limits the list.
pub fn time_options(
    step: u32,
    cycle: HourCycle,
    window: Option<(TimeOfDay, TimeOfDay)>,
) -> Vec<(String, String)> {
    let step = step.clamp(1, 60);
    (0..1440)
        .step_by(step as usize)
        .map(|m| TimeOfDay::new(0, m))
        .filter(|t| window.is_none_or(|(a, b)| *t >= a && *t <= b))
        .map(|t| (t.hhmm(), t.label(cycle)))
        .collect()
}

/// The option nearest to `t` (the list opens centred on it).
pub fn nearest_option(options: &[(String, String)], t: TimeOfDay) -> Option<usize> {
    options
        .iter()
        .enumerate()
        .filter_map(|(i, (v, _))| TimeOfDay::parse(v).map(|x| (i, x)))
        .min_by_key(|(_, x)| x.minutes().abs_diff(t.minutes()))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }

    #[test]
    fn civil_arithmetic() {
        assert_eq!(d(1970, 1, 1).to_days(), 0);
        assert_eq!(Date::from_days(0), d(1970, 1, 1));
        for n in [-800_000i64, -1, 59, 60, 365, 11_000, 20_405, 300_000] {
            assert_eq!(Date::from_days(n).to_days(), n);
        }
        assert_eq!(d(2026, 11, 13).weekday(), 5, "a Friday");
        assert_eq!(d(2024, 1, 31).add_months(1), d(2024, 2, 29));
        assert_eq!(d(2026, 1, 15).add_months(-2), d(2025, 11, 15));
        assert!(Date::new(2026, 2, 30).is_none());
        assert_eq!(d(2026, 11, 13).week_start(1), d(2026, 11, 9));
        assert_eq!(d(2026, 1, 1).iso_week(), 1);
        assert_eq!(d(2026, 12, 31).iso_week(), 53);
        assert_eq!(d(2026, 11, 13).long(), "Friday, November 13, 2026");
        assert_eq!(Date::parse_iso("2026-11-13"), Some(d(2026, 11, 13)));
    }

    #[test]
    fn formats_and_parts() {
        let x = d(2026, 11, 13);
        assert_eq!(x.format(DateFormat::Medium), "Nov 13, 2026");
        assert_eq!(x.format(DateFormat::Weekday), "Fri, Nov 13");
        assert_eq!(
            DateFormat::Medium.parts(x),
            ["Nov", " ", "13", ",", " ", "2026"]
        );
    }

    #[test]
    fn natural_language() {
        let t = d(2026, 11, 13); // Friday
        assert_eq!(parse_natural("tomorrow", t), Some(d(2026, 11, 14)));
        assert_eq!(parse_natural("in 2 weeks", t), Some(d(2026, 11, 27)));
        assert_eq!(parse_natural("3 days ago", t), Some(d(2026, 11, 10)));
        assert_eq!(parse_natural("next tue", t), Some(d(2026, 11, 17)));
        assert_eq!(parse_natural("fri", t), Some(d(2026, 11, 20)));
        assert_eq!(parse_natural("Dec 25", t), Some(d(2026, 12, 25)));
        assert_eq!(parse_natural("25 dec", t), Some(d(2026, 12, 25)));
        assert_eq!(parse_natural("blue", t), None);
    }

    #[test]
    fn calendar_grid_and_keys() {
        let mut c = Calendar::new(Some(d(2026, 11, 13)), d(2026, 11, 13));
        let g = c.grid();
        assert_eq!(g.len(), 6);
        assert_eq!(g[0][0].date, d(2026, 11, 1), "Nov 1 2026 is a Sunday");
        assert!(g[1][5].selected && g[1][5].today && g[1][5].focused);
        c.blocked = Blocked::Weekends;
        assert_eq!(
            c.key(CalKey::Right, ClockMs(0)),
            d(2026, 11, 16),
            "skips the weekend"
        );
        c.key(CalKey::PageDown { year: false }, ClockMs(1));
        assert_eq!(c.month, d(2026, 12, 1));
        assert_eq!(c.moved.map(|m| m.0), Some(Travel::Forward));
        c.max = Some(d(2026, 12, 20));
        assert!(!c.can_next());
        assert!(!c.page(1, ClockMs(2)));
        c.key(CalKey::Down, ClockMs(3));
        c.key(CalKey::Down, ClockMs(4));
        assert!(c.focus <= d(2026, 12, 20), "stops at max");
        assert_eq!(c.select(d(2026, 12, 19), ClockMs(5)), None, "a Saturday");
        assert_eq!(c.select(d(2026, 12, 18), ClockMs(5)), Some(d(2026, 12, 18)));
        c.week_starts_on = 1;
        assert_eq!(c.weekdays()[0], ("Mo", "Monday"));
        assert_eq!(c.go_today(ClockMs(6)), Some(d(2026, 11, 13)));
        assert_eq!(c.moved.map(|m| m.0), Some(Travel::Back));
    }

    #[test]
    fn range_pick_preview_presets_compare() {
        let today = d(2026, 11, 13);
        let mut r = RangePicker::new(None, today);
        r.open_panel(ClockMs(0));
        assert_eq!(r.pick(d(2026, 11, 10), ClockMs(1)), None);
        r.hover(Some(d(2026, 11, 3)));
        let b = r.band(d(2026, 11, 5));
        assert!(b.inside && b.preview);
        let done = r.pick(d(2026, 11, 3), ClockMs(2)).unwrap();
        assert_eq!(done, DateRange::new(d(2026, 11, 3), d(2026, 11, 10)));
        assert_eq!(done.days(), 8);
        r.hover(Some(d(2026, 11, 20)));
        assert_eq!(r.hover, None, "after max");
        r.preset("7d", ClockMs(3));
        assert_eq!(r.active_preset(), Some("7d"));
        let (applied, cmp) = r.apply().unwrap();
        assert_eq!(applied.label(), "Nov 7 – Nov 13, 2026");
        assert_eq!(
            cmp.unwrap(),
            DateRange::new(d(2026, 10, 31), d(2026, 11, 6))
        );
        assert_eq!(
            applied.compare(Compare::Year).unwrap().start,
            d(2025, 11, 7)
        );
        assert!(r.band(d(2026, 11, 2)).compare);
        r.open_panel(ClockMs(4));
        r.pick(d(2026, 11, 1), ClockMs(5));
        r.cancel();
        assert_eq!(r.value, Some(applied), "cancel keeps the applied range");
    }

    #[test]
    fn date_picker_closes_after_pick() {
        let mut p = DatePicker::new(Some(d(2026, 11, 13)), d(2026, 11, 13));
        p.open_popover();
        p.pick(d(2026, 12, 2), ClockMs(1_000), false);
        assert!(p.open);
        assert!(!p.tick(ClockMs(1_100)));
        assert!(p.tick(ClockMs(1_240)));
        assert_eq!(p.travel(), Travel::Forward);
        let parts = p.parts();
        assert!(
            parts[0].1 && parts[2].1 && !parts[5].1,
            "month and day roll, year stays"
        );
        assert_eq!(
            p.enter_text("yesterday", ClockMs(2_000)),
            Some(d(2026, 11, 12))
        );
        p.clear();
        assert_eq!(p.value(), None);
    }

    #[test]
    fn times() {
        assert_eq!(TimeOfDay::parse("9:30 pm"), Some(TimeOfDay::new(21, 30)));
        assert_eq!(TimeOfDay::parse("12am"), Some(TimeOfDay::new(0, 0)));
        assert_eq!(TimeOfDay::parse("13pm"), None);
        assert_eq!(TimeOfDay::new(0, 5).label(HourCycle::H12), "12:05 AM");
        assert_eq!(TimeOfDay::new(13, 0).label(HourCycle::H24), "13:00");
        let o = time_options(15, HourCycle::H12, None);
        assert_eq!(o.len(), 96);
        let w = time_options(
            30,
            HourCycle::H24,
            Some((TimeOfDay::new(9, 0), TimeOfDay::new(17, 0))),
        );
        assert_eq!(w.first().unwrap().0, "09:00");
        assert_eq!(w.len(), 17);
        assert_eq!(nearest_option(&o, TimeOfDay::new(9, 7)), Some(36));
    }
}
