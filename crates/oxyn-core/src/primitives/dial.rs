//! Time dial and date reel (`time-dial.md`, `date-reel.md`): the two physical pickers.
//!
//! - **Time dial** (Arc UI Time dial): a 24-hour rotary dial for choosing a time across time zones.
//!   Every city rides the dial at its own local time, a band marks working hours, the home city
//!   reads out in the centre. Drags have soft quarter-hour detents, flicks coast into the nearest
//!   quarter hour, bounds rubber-band.
//! - **Date reel** (Arc UI Date reel): iOS-style wheels for day, hour, minute and AM / PM (or
//!   month, day, year). Each reel is projected onto a cylinder; flicks keep their momentum and
//!   land on an entry through a critically damped spring; drags past the ends stretch.
//!
//! Rebuilt from the public docs, no source used. Oxyn plus: a quiet-hours band and a "best
//! overlap" finder on the dial; natural-language entry and a routine preset on the reel.

use std::f32::consts::{PI, TAU};

use super::date::{Date, HourCycle, TimeOfDay, parse_natural};
use crate::ClockMs;

/// Critically damped spring: position and velocity after `dt` seconds toward `target`.
pub fn spring_step(x: f32, v: f32, target: f32, omega: f32, dt: f32) -> (f32, f32) {
    // Exact solution for a critically damped oscillator.
    let d = x - target;
    let e = (-omega * dt).exp();
    let c = v + omega * d;
    let nx = target + (d + c * dt) * e;
    let nv = (c - omega * (d + c * dt)) * e;
    (nx, nv)
}

/// Rubber band: how far a drag past a bound shows (`over` px of pull → shown px).
pub fn rubber(over: f32, dim: f32) -> f32 {
    let c = 0.55;
    (1.0 - 1.0 / (over.abs() * c / dim.max(1.0) + 1.0)) * dim * over.signum()
}

// ---------------------------------------------------------------------------------------------
// Time dial

/// A city on the dial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct City {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Two or three letters shown on the dial.
    pub code: String,
    /// Minutes from UTC, in steps of 15.
    pub offset: i32,
}

impl City {
    /// A city.
    pub fn new(id: &str, name: &str, code: &str, offset: i32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            code: code.into(),
            offset,
        }
    }
}

/// The time dial.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeDial {
    /// Two to five cities.
    pub cities: Vec<City>,
    /// UTC minutes from midnight on the reference day (may leave 0..1440).
    pub value: i32,
    /// Home city (the dial reads in it).
    pub home: usize,
    /// Earliest UTC minute.
    pub min: Option<i32>,
    /// Latest UTC minute.
    pub max: Option<i32>,
    /// Weekday of the reference day in UTC (0 = Sunday).
    pub weekday: u8,
    /// Readout clock.
    pub cycle: HourCycle,
    /// Local working hours, minutes from midnight.
    pub working: (i32, i32),
    /// Oxyn plus: local quiet hours (shaded darker), minutes from midnight.
    pub quiet: Option<(i32, i32)>,
    /// Raw value while dragging (before detents and rubber band).
    raw: Option<f32>,
    /// Coasting toward a quarter hour: from, target, velocity, start.
    coast: Option<(f32, i32, f32, ClockMs)>,
    /// Last change for the readout roll.
    pub changed_at: ClockMs,
}

impl TimeDial {
    /// A dial.
    pub fn new(cities: Vec<City>, value: i32) -> Self {
        Self {
            cities,
            value,
            home: 0,
            min: None,
            max: None,
            weekday: 1,
            cycle: HourCycle::H12,
            working: (540, 1080),
            quiet: None,
            raw: None,
            coast: None,
            changed_at: ClockMs(0),
        }
    }
    /// Local minutes in the day for city `i` at UTC minute `v`, and the day offset from the
    /// reference day.
    pub fn local_at(&self, i: usize, v: i32) -> (i32, i32) {
        let m = v + self.cities.get(i).map_or(0, |c| c.offset);
        (m.rem_euclid(1440), m.div_euclid(1440))
    }
    /// Local time for city `i` now.
    pub fn local(&self, i: usize) -> (i32, i32) {
        self.local_at(i, self.value)
    }
    /// City `i` is inside working hours.
    pub fn at_work(&self, i: usize) -> bool {
        let (m, _) = self.local(i);
        m >= self.working.0 && m < self.working.1
    }
    /// City `i` is in its quiet hours (Oxyn plus).
    pub fn quiet_now(&self, i: usize) -> bool {
        let Some((a, b)) = self.quiet else {
            return false;
        };
        let (m, _) = self.local(i);
        if a <= b {
            m >= a && m < b
        } else {
            m >= a || m < b
        }
    }
    /// Daytime (sun) in city `i`: 6:00–18:00.
    pub fn daytime(&self, i: usize) -> bool {
        let (m, _) = self.local(i);
        (360..1080).contains(&m)
    }
    /// Angle on the dial (radians, clockwise from the top) for local minutes; noon at the top.
    pub fn angle(minutes: i32) -> f32 {
        ((minutes as f32 - 720.0) / 1440.0 * TAU).rem_euclid(TAU)
    }
    /// Point on a circle of radius `r` around `(cx, cy)` for local minutes.
    pub fn point(minutes: i32, cx: f32, cy: f32, r: f32) -> (f32, f32) {
        let a = Self::angle(minutes);
        (cx + r * a.sin(), cy - r * a.cos())
    }
    /// Local minutes for a pointer angle (radians clockwise from the top).
    pub fn minutes_for(angle: f32) -> f32 {
        (angle.rem_euclid(TAU) / TAU * 1440.0 + 720.0).rem_euclid(1440.0)
    }
    fn clamp(&self, v: i32) -> i32 {
        let v = self.min.map_or(v, |m| v.max(m));
        self.max.map_or(v, |m| v.min(m))
    }
    /// Starts a drag.
    pub fn grab(&mut self) {
        self.raw = Some(self.value as f32);
        self.coast = None;
    }
    /// Drags by `delta` minutes: soft quarter-hour detents (snaps within 3 minutes), and a rubber
    /// band past the bounds.
    pub fn drag(&mut self, delta: f32, now: ClockMs) {
        let raw = self.raw.unwrap_or(self.value as f32) + delta;
        self.raw = Some(raw);
        let near = (raw / 15.0).round() * 15.0;
        let mut v = if (raw - near).abs() <= 3.0 { near } else { raw };
        if let Some(m) = self.min
            && v < m as f32
        {
            v = m as f32 - rubber(m as f32 - v, 60.0);
        }
        if let Some(m) = self.max
            && v > m as f32
        {
            v = m as f32 + rubber(v - m as f32, 60.0);
        }
        let nv = v.round() as i32;
        if nv != self.value {
            self.value = nv;
            self.changed_at = now;
        }
    }
    /// Releases a drag with `velocity` (minutes per second). The dial coasts to the nearest
    /// quarter hour inside the bounds; returns that destination (fires `on_value_change`).
    pub fn release(&mut self, velocity: f32, now: ClockMs, reduced: bool) -> i32 {
        let from = self.value as f32;
        let ahead = if reduced { 0.0 } else { velocity * 0.3 };
        let target = self.clamp(((from + ahead) / 15.0).round() as i32 * 15);
        self.raw = None;
        if reduced {
            self.value = target;
        } else {
            self.coast = Some((from, target, velocity, now));
        }
        self.changed_at = now;
        target
    }
    /// Advances the coast; returns `true` while moving.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        let Some((from, target, v, at)) = self.coast else {
            return false;
        };
        let dt = now.since(at).as_secs_f32();
        let (x, nv) = spring_step(from, v, target as f32, 14.0, dt);
        if (x - target as f32).abs() < 0.5 && nv.abs() < 2.0 {
            self.value = target;
            self.coast = None;
            return false;
        }
        self.value = x.round() as i32;
        true
    }
    /// Destination of a running coast (or the value).
    pub fn destination(&self) -> i32 {
        self.coast.map_or(self.value, |c| c.1)
    }
    /// Keyboard: `step` minutes (±15 arrows, ±60 Page keys). Returns the new value.
    pub fn step(&mut self, by: i32, now: ClockMs) -> i32 {
        let base = (self.destination() as f32 / 15.0).round() as i32 * 15;
        self.value = self.clamp(base + by);
        self.coast = None;
        self.changed_at = now;
        self.value
    }
    /// Home / End: the start or the last quarter hour of the home city's day.
    pub fn edge(&mut self, end: bool, now: ClockMs) -> i32 {
        let (m, _) = self.local(self.home);
        let start = self.value - m;
        self.value = self.clamp(if end { start + 1425 } else { start });
        self.coast = None;
        self.changed_at = now;
        self.value
    }
    /// Choose the home city.
    pub fn set_home(&mut self, i: usize) -> Option<&str> {
        if i < self.cities.len() && i != self.home {
            self.home = i;
            return Some(&self.cities[i].id);
        }
        None
    }
    /// Time label for city `i`.
    pub fn time_label(&self, i: usize) -> String {
        let (m, _) = self.local(i);
        TimeOfDay::new(0, m as u32).label(self.cycle)
    }
    /// `+1 day` / `−1 day` relative to the home city's day, when different.
    pub fn day_note(&self, i: usize) -> Option<&'static str> {
        let (_, d) = self.local(i);
        let (_, h) = self.local(self.home);
        match d - h {
            0 => None,
            1 => Some("+1 day"),
            -1 => Some("−1 day"),
            x if x > 0 => Some("+2 days"),
            _ => Some("−2 days"),
        }
    }
    /// Weekday name in the home city.
    pub fn home_weekday(&self) -> &'static str {
        let (_, d) = self.local(self.home);
        super::date::WEEKDAYS[(i32::from(self.weekday) + d).rem_euclid(7) as usize]
    }
    /// `aria-valuetext`: `9:30 AM Tuesday in London, 3 of 4 cities at work`.
    pub fn valuetext(&self) -> String {
        let at = (0..self.cities.len()).filter(|&i| self.at_work(i)).count();
        format!(
            "{} {} in {}, {at} of {} cities at work",
            self.time_label(self.home),
            self.home_weekday(),
            self.cities
                .get(self.home)
                .map(|c| c.name.as_str())
                .unwrap_or(""),
            self.cities.len()
        )
    }
    /// Oxyn plus: the UTC quarter hour (within the reference day) when the most cities are at
    /// work, earliest first.
    pub fn best_overlap(&self) -> (i32, usize) {
        let mut best = (0, 0);
        for q in 0..96 {
            let v = q * 15;
            let n = (0..self.cities.len())
                .filter(|&i| {
                    let (m, _) = self.local_at(i, v);
                    m >= self.working.0 && m < self.working.1
                })
                .count();
            if n > best.1 {
                best = (v, n);
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------------------------
// Date reel

/// Which reels show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReelMode {
    /// Day, hour, minute (and AM / PM).
    #[default]
    DateTime,
    /// Month, day, year.
    Date,
    /// Hour, minute (and AM / PM).
    Time,
}

impl ReelMode {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "date" => Self::Date,
            "time" => Self::Time,
            _ => Self::DateTime,
        }
    }
}

/// What a reel turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReelKind {
    /// A day between min and max (`Today`, `Tomorrow`, `Fri, Sep 25`).
    Day,
    /// Month.
    Month,
    /// Day of month 1–31 (days past the month's end are dimmed).
    DayOfMonth,
    /// Year.
    Year,
    /// Hour.
    Hour,
    /// Minute (by `minute_step`).
    Minute,
    /// AM / PM.
    Period,
}

impl ReelKind {
    /// Name (each reel is a spinbutton with this name).
    pub fn name(self) -> &'static str {
        match self {
            Self::Day => "Day",
            Self::Month => "Month",
            Self::DayOfMonth => "Date",
            Self::Year => "Year",
            Self::Hour => "Hour",
            Self::Minute => "Minute",
            Self::Period => "AM or PM",
        }
    }
}

/// One wheel.
#[derive(Clone, Debug, PartialEq)]
pub struct Reel {
    /// Kind.
    pub kind: ReelKind,
    /// Entries.
    pub entries: Vec<String>,
    /// Entries that exist (February 30 is dimmed).
    pub valid: Vec<bool>,
    /// Position in rows (fractional while moving).
    pub pos: f32,
    /// Velocity in rows per second.
    pub vel: f32,
    /// Entry it is heading to.
    pub target: usize,
    /// Grabbed by a drag.
    pub dragging: bool,
}

/// Angle between neighbouring entries on the cylinder.
pub const ROW_ANGLE: f32 = PI / 9.0; // 20°

impl Reel {
    fn new(kind: ReelKind, entries: Vec<String>, at: usize) -> Self {
        let n = entries.len();
        Self {
            kind,
            valid: vec![true; n],
            entries,
            pos: at as f32,
            vel: 0.0,
            target: at,
            dragging: false,
        }
    }
    /// The entry it rests on (or is heading to).
    pub fn index(&self) -> usize {
        self.target
    }
    /// At rest.
    pub fn resting(&self) -> bool {
        !self.dragging && (self.pos - self.target as f32).abs() < 0.001 && self.vel.abs() < 0.01
    }
    /// Projection of entry `i`: `(y offset in rows, vertical scale, opacity)`; `None` when it is
    /// behind the cylinder.
    pub fn project(&self, i: usize, radius_rows: f32) -> Option<(f32, f32, f32)> {
        let a = (i as f32 - self.pos) * ROW_ANGLE;
        if a.abs() >= PI / 2.0 {
            return None;
        }
        let c = a.cos();
        Some((radius_rows * a.sin(), c, c.powf(1.5)))
    }
    /// Spins to entry `i` on the spring (keyboard steps add to the motion).
    pub fn spin_to(&mut self, i: usize) {
        self.target = i.min(self.entries.len().saturating_sub(1));
    }
    /// Drag by `rows` (positive = later entries). Past the ends it stretches like rubber.
    pub fn drag(&mut self, rows: f32) {
        self.dragging = true;
        let max = self.entries.len().saturating_sub(1) as f32;
        let raw = self.pos + rows;
        self.pos = if raw < 0.0 {
            -rubber(-raw, 1.5)
        } else if raw > max {
            max + rubber(raw - max, 1.5)
        } else {
            raw
        };
    }
    /// Release with `velocity` rows per second: an exponential glide solved to land on an
    /// entry, then the spring takes over.
    pub fn release(&mut self, velocity: f32, speed: f32) {
        self.dragging = false;
        let tau = 0.325 / speed.max(0.1);
        let max = self.entries.len().saturating_sub(1) as f32;
        let land = (self.pos + velocity * tau).round().clamp(0.0, max);
        self.vel = velocity;
        self.target = land as usize;
    }
    /// Advances `dt` seconds. Returns `true` while moving.
    pub fn tick(&mut self, dt: f32, speed: f32, reduced: bool) -> bool {
        if self.dragging {
            return true;
        }
        if reduced {
            self.pos = self.target as f32;
            self.vel = 0.0;
            return false;
        }
        let (x, v) = spring_step(self.pos, self.vel, self.target as f32, 16.0 * speed, dt);
        self.pos = x;
        self.vel = v;
        if (x - self.target as f32).abs() < 0.002 && v.abs() < 0.02 {
            self.pos = self.target as f32;
            self.vel = 0.0;
            return false;
        }
        true
    }
}

/// Keys on a focused reel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReelKey {
    /// ↑ (an earlier entry).
    Up,
    /// ↓ (a later entry).
    Down,
    /// Page Up: a large step back.
    PageUp,
    /// Page Down: a large step on.
    PageDown,
    /// Home.
    Home,
    /// End.
    End,
}

/// Confirm button state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ConfirmState {
    /// Ready.
    #[default]
    Idle,
    /// The promise is pending.
    Pending,
    /// Done.
    Success,
    /// Failed.
    Failed,
}

/// A quick pick under the reels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReelPreset {
    /// Label.
    pub label: String,
    /// Day and time.
    pub value: (Date, TimeOfDay),
}

/// The date reel.
#[derive(Clone, Debug, PartialEq)]
pub struct DateReel {
    /// Mode.
    pub mode: ReelMode,
    /// Today.
    pub today: Date,
    /// Earliest day.
    pub min: Date,
    /// Latest day.
    pub max: Date,
    /// Minutes between minute entries.
    pub minute_step: u32,
    /// Clock.
    pub cycle: HourCycle,
    /// Reels left to right.
    pub reels: Vec<Reel>,
    /// Focused reel.
    pub focus: usize,
    /// Quick picks.
    pub presets: Vec<ReelPreset>,
    /// Confirm button state.
    pub confirm: ConfirmState,
    /// Glide speed multiplier.
    pub speed: f32,
    /// The last value reported (so `on_change` fires once per rest).
    pub committed: (Date, TimeOfDay),
    typed: (String, Option<ClockMs>),
}

impl DateReel {
    /// A reel at `value`; min / max default to 30 days before and a year after today (100 years
    /// back and 10 on in date mode).
    pub fn new(mode: ReelMode, value: (Date, TimeOfDay), today: Date) -> Self {
        let (min, max) = match mode {
            ReelMode::Date => (today.add_months(-1200), today.add_months(120)),
            _ => (today.add_days(-30), today.add_months(12)),
        };
        let mut r = Self {
            mode,
            today,
            min,
            max,
            minute_step: 5,
            cycle: HourCycle::H12,
            reels: vec![],
            focus: 0,
            presets: vec![],
            confirm: ConfirmState::Idle,
            speed: 1.0,
            committed: value,
            typed: (String::new(), None),
        };
        r.presets = r.default_presets();
        r.build(value);
        r
    }
    fn default_presets(&self) -> Vec<ReelPreset> {
        let t = self.today;
        let monday = t.add_days((8 - i64::from(t.weekday())).rem_euclid(7).max(1));
        vec![
            ReelPreset {
                label: "This evening".into(),
                value: (t, TimeOfDay::new(18, 0)),
            },
            ReelPreset {
                label: "Tomorrow".into(),
                value: (t.add_days(1), TimeOfDay::new(9, 0)),
            },
            ReelPreset {
                label: "Monday".into(),
                value: (monday, TimeOfDay::new(9, 0)),
            },
        ]
    }
    /// Day label: `Today`, `Tomorrow`, `Yesterday` or `Fri, Sep 25`.
    pub fn day_label(&self, d: Date) -> String {
        match self.today.days_until(d) {
            0 => "Today".into(),
            1 => "Tomorrow".into(),
            -1 => "Yesterday".into(),
            _ => super::date::DateFormat::Weekday.apply(d),
        }
    }
    fn hours(&self) -> Vec<String> {
        match self.cycle {
            HourCycle::H12 => (0..12)
                .map(|h| if h == 0 { 12 } else { h }.to_string())
                .collect(),
            HourCycle::H24 => (0..24).map(|h| format!("{h:02}")).collect(),
        }
    }
    /// Rebuilds the reels for `v` after changing `minute_step`, `cycle` or `mode`.
    pub fn rebuild(&mut self, v: (Date, TimeOfDay)) {
        self.build(v);
        self.committed = self.value();
    }
    fn build(&mut self, (d, t): (Date, TimeOfDay)) {
        let d = d.clamp(self.min, self.max);
        let step = self.minute_step.clamp(1, 30) as usize;
        let minutes: Vec<String> = (0..60).step_by(step).map(|m| format!("{m:02}")).collect();
        let mi = (usize::from(t.m) / step).min(minutes.len() - 1);
        let h_idx = match self.cycle {
            HourCycle::H12 => usize::from(t.h % 12),
            HourCycle::H24 => usize::from(t.h),
        };
        let mut reels = vec![];
        if self.mode == ReelMode::DateTime {
            let n = self.min.days_until(self.max) as usize + 1;
            let days = (0..n)
                .map(|k| self.day_label(self.min.add_days(k as i64)))
                .collect();
            reels.push(Reel::new(
                ReelKind::Day,
                days,
                self.min.days_until(d) as usize,
            ));
        }
        if self.mode == ReelMode::Date {
            let months = super::date::MONTHS
                .iter()
                .map(|m| m[..3].to_string())
                .collect();
            reels.push(Reel::new(ReelKind::Month, months, usize::from(d.m - 1)));
            let days = (1..=31).map(|x| x.to_string()).collect();
            reels.push(Reel::new(ReelKind::DayOfMonth, days, usize::from(d.d - 1)));
            let years = (self.min.y..=self.max.y).map(|y| y.to_string()).collect();
            reels.push(Reel::new(
                ReelKind::Year,
                years,
                (d.y - self.min.y) as usize,
            ));
        }
        if self.mode != ReelMode::Date {
            reels.push(Reel::new(ReelKind::Hour, self.hours(), h_idx));
            reels.push(Reel::new(ReelKind::Minute, minutes, mi));
            if self.cycle == HourCycle::H12 {
                reels.push(Reel::new(
                    ReelKind::Period,
                    vec!["AM".into(), "PM".into()],
                    usize::from(t.h >= 12),
                ));
            }
        }
        self.reels = reels;
        self.mark_valid();
    }
    fn reel(&self, k: ReelKind) -> Option<&Reel> {
        self.reels.iter().find(|r| r.kind == k)
    }
    fn mark_valid(&mut self) {
        let (y, m) = match (self.reel(ReelKind::Year), self.reel(ReelKind::Month)) {
            (Some(yr), Some(mo)) => (self.min.y + yr.index() as i32, mo.index() as u8 + 1),
            _ => return,
        };
        let dim = super::date::days_in_month(y, m);
        if let Some(r) = self
            .reels
            .iter_mut()
            .find(|r| r.kind == ReelKind::DayOfMonth)
        {
            r.valid = (1..=31).map(|d| d <= dim).collect();
            if r.target + 1 > usize::from(dim) {
                // The day reel turns back to the last real day by itself.
                r.target = usize::from(dim) - 1;
            }
        }
    }
    /// The value the reels point at (their targets).
    pub fn value(&self) -> (Date, TimeOfDay) {
        let ix = |k| self.reel(k).map(Reel::index);
        let date = match self.mode {
            ReelMode::DateTime => self.min.add_days(ix(ReelKind::Day).unwrap_or(0) as i64),
            ReelMode::Date => Date::clamped(
                self.min.y + ix(ReelKind::Year).unwrap_or(0) as i32,
                ix(ReelKind::Month).unwrap_or(0) as u8 + 1,
                ix(ReelKind::DayOfMonth).unwrap_or(0) as u8 + 1,
            ),
            ReelMode::Time => self.committed.0,
        };
        let time = if self.mode == ReelMode::Date {
            self.committed.1
        } else {
            let step = self.minute_step.clamp(1, 30);
            let m = ix(ReelKind::Minute).unwrap_or(0) as u32 * step;
            let mut h = ix(ReelKind::Hour).unwrap_or(0) as u32;
            if self.cycle == HourCycle::H12 && ix(ReelKind::Period) == Some(1) {
                h += 12;
            }
            TimeOfDay::new(h, m)
        };
        (date, time)
    }
    /// Spins every reel to `v` (presets, outside value changes).
    pub fn spin_to(&mut self, v: (Date, TimeOfDay)) {
        let (d, t) = v;
        let d = d.clamp(self.min, self.max);
        let step = self.minute_step.clamp(1, 30) as usize;
        let min = self.min;
        let cycle = self.cycle;
        for r in &mut self.reels {
            let i = match r.kind {
                ReelKind::Day => min.days_until(d) as usize,
                ReelKind::Month => usize::from(d.m - 1),
                ReelKind::DayOfMonth => usize::from(d.d - 1),
                ReelKind::Year => (d.y - min.y) as usize,
                ReelKind::Hour => match cycle {
                    HourCycle::H12 => usize::from(t.h % 12),
                    HourCycle::H24 => usize::from(t.h),
                },
                ReelKind::Minute => usize::from(t.m) / step,
                ReelKind::Period => usize::from(t.h >= 12),
            };
            r.spin_to(i);
        }
        self.mark_valid();
    }
    /// Applies quick pick `i`.
    pub fn preset(&mut self, i: usize) {
        if let Some(p) = self.presets.get(i).cloned() {
            self.spin_to(p.value);
        }
    }
    /// The preset matching the value (shown pressed).
    pub fn active_preset(&self) -> Option<usize> {
        let v = self.value();
        self.presets.iter().position(|p| p.value == v)
    }
    fn large(&self, k: ReelKind) -> usize {
        match k {
            ReelKind::Day => 7,
            ReelKind::Hour if self.cycle == HourCycle::H12 => 3,
            ReelKind::Hour => 6,
            ReelKind::Minute => (15 / self.minute_step.clamp(1, 15)) as usize,
            ReelKind::Year => 10,
            ReelKind::Month => 3,
            ReelKind::DayOfMonth => 7,
            ReelKind::Period => 1,
        }
    }
    /// Keyboard on reel `r`. Hours and minutes wrap; days and years stop at the ends.
    pub fn key(&mut self, r: usize, k: ReelKey) {
        let Some(kind) = self.reels.get(r).map(|x| x.kind) else {
            return;
        };
        let big = self.large(kind);
        let reel = &mut self.reels[r];
        let n = reel.entries.len();
        let wraps = matches!(kind, ReelKind::Hour | ReelKind::Minute | ReelKind::Period);
        let cur = reel.target as isize;
        let next = match k {
            ReelKey::Up => cur - 1,
            ReelKey::Down => cur + 1,
            ReelKey::PageUp => cur - big as isize,
            ReelKey::PageDown => cur + big as isize,
            ReelKey::Home => 0,
            ReelKey::End => n as isize - 1,
        };
        let i = if wraps {
            next.rem_euclid(n as isize) as usize
        } else {
            next.clamp(0, n as isize - 1) as usize
        };
        reel.spin_to(i);
        self.mark_valid();
    }
    /// Typing on reel `r`: `4` for 4 o'clock, `45` for :45, `p` for PM, `ma` for March. Letters
    /// typed within a second build a prefix.
    pub fn typeahead(&mut self, r: usize, ch: char, now: ClockMs) -> bool {
        if self.typed.1.is_none_or(|t| now.since(t).as_millis() > 1000) {
            self.typed.0.clear();
        }
        self.typed.1 = Some(now);
        self.typed.0.extend(ch.to_lowercase());
        let q = self.typed.0.clone();
        let Some(reel) = self.reels.get(r) else {
            return false;
        };
        let hit = reel
            .entries
            .iter()
            .position(|e| {
                let e = e.to_lowercase();
                e.starts_with(&q)
                    || e.trim_start_matches('0')
                        .starts_with(q.trim_start_matches('0'))
            })
            .or_else(|| {
                // A single digit past the list (e.g. "7" for minute :07 with step 5) → nearest.
                q.parse::<usize>().ok().and_then(|n| {
                    reel.entries
                        .iter()
                        .enumerate()
                        .filter_map(|(i, e)| e.parse::<usize>().ok().map(|x| (i, x.abs_diff(n))))
                        .min_by_key(|(_, d)| *d)
                        .map(|(i, _)| i)
                })
            });
        if let Some(i) = hit {
            self.reels[r].spin_to(i);
            self.mark_valid();
            return true;
        }
        false
    }
    /// Oxyn plus: natural-language entry beside the reels (`tomorrow 9:30 pm`).
    pub fn enter_text(&mut self, s: &str) -> bool {
        let words: Vec<&str> = s.split_whitespace().collect();
        for split in (0..=words.len()).rev() {
            let (dw, tw) = words.split_at(split);
            let day = if dw.is_empty() {
                Some(self.value().0)
            } else {
                parse_natural(&dw.join(" "), self.today)
            };
            let time = if tw.is_empty() {
                Some(self.value().1)
            } else {
                TimeOfDay::parse(&tw.join(" ").replace("at ", ""))
            };
            if let (Some(d), Some(t)) = (day, time) {
                self.spin_to((d, t));
                return true;
            }
        }
        false
    }
    /// Advances every reel `dt` seconds; when all come to rest on a new value, returns it once
    /// (`on_change`).
    pub fn tick(&mut self, dt: f32, reduced: bool) -> Option<(Date, TimeOfDay)> {
        let speed = self.speed;
        let mut moving = false;
        for r in &mut self.reels {
            moving |= r.tick(dt, speed, reduced);
        }
        if moving {
            return None;
        }
        let v = self.value();
        if v != self.committed {
            self.committed = v;
            return Some(v);
        }
        None
    }
    /// The summary over the reels: `Tomorrow at 9:30 AM`.
    pub fn summary(&self) -> String {
        let (d, t) = self.value();
        match self.mode {
            ReelMode::Time => t.label(self.cycle),
            ReelMode::Date => super::date::DateFormat::Long.apply(d),
            ReelMode::DateTime => format!("{} at {}", self.day_label(d), t.label(self.cycle)),
        }
    }
    /// The relative phrase: `In 12 days`, `Tomorrow`, `3 days ago`.
    pub fn relative(&self) -> String {
        match self.today.days_until(self.value().0) {
            0 => "Today".into(),
            1 => "Tomorrow".into(),
            -1 => "Yesterday".into(),
            n if n > 0 => format!("In {n} days"),
            n => format!("{} days ago", -n),
        }
    }
    /// `aria-valuetext` for reel `r`.
    pub fn valuetext(&self, r: usize) -> String {
        let Some(reel) = self.reels.get(r) else {
            return String::new();
        };
        match reel.kind {
            ReelKind::Day => {
                let d = self.value().0;
                super::date::DateFormat::Weekday.apply(d)
            }
            _ => reel.entries.get(reel.index()).cloned().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cities() -> Vec<City> {
        vec![
            City::new("lon", "London", "LON", 0),
            City::new("nyc", "New York", "NYC", -300),
            City::new("khi", "Karachi", "KHI", 300),
            City::new("syd", "Sydney", "SYD", 660),
        ]
    }

    #[test]
    fn spring_and_rubber() {
        let (x, v) = spring_step(0.0, 0.0, 10.0, 12.0, 1.0);
        assert!((x - 10.0).abs() < 0.01 && v.abs() < 0.1);
        assert!(rubber(100.0, 60.0) < 60.0 && rubber(100.0, 60.0) > 0.0);
        assert!(rubber(-10.0, 60.0) < 0.0);
    }

    #[test]
    fn dial_locals_detents_and_coast() {
        let mut d = TimeDial::new(cities(), 600); // 10:00 UTC
        assert_eq!(d.local(1), (300, 0), "5:00 in New York");
        assert_eq!(d.local(3), (1260, 0), "21:00 in Sydney");
        assert!(d.at_work(0) && !d.at_work(1) && d.at_work(2));
        assert_eq!(d.time_label(2), "3:00 PM");
        d.value = 1000;
        assert_eq!(d.day_note(3), Some("+1 day"));
        assert!((TimeDial::angle(720)).abs() < 1e-5, "noon at the top");
        assert!(
            (TimeDial::minutes_for(PI) - 0.0).abs() < 1e-3,
            "midnight at the bottom"
        );
        d.grab();
        d.drag(13.0, ClockMs(0));
        assert_eq!(d.value, 1013);
        d.drag(4.0, ClockMs(1));
        assert_eq!(d.value, 1020, "soft detent pulls to :00/:15/:30/:45");
        let target = d.release(200.0, ClockMs(10), false);
        assert_eq!(target % 15, 0);
        assert!(target > 1020, "a flick coasts forward");
        let mut t = 10;
        while d.tick(ClockMs(t)) {
            t += 16;
            assert!(t < 5_000);
        }
        assert_eq!(d.value, target);
        d.max = Some(1100);
        d.grab();
        d.drag(200.0, ClockMs(t));
        assert!(d.value > 1100 && d.value < 1160, "rubber band past max");
        assert_eq!(d.release(0.0, ClockMs(t), true), 1100);
    }

    #[test]
    fn dial_keys_home_overlap() {
        let mut d = TimeDial::new(cities(), 607);
        assert_eq!(d.step(15, ClockMs(0)), 615);
        assert_eq!(d.step(-60, ClockMs(1)), 555);
        d.set_home(2);
        assert_eq!(
            d.edge(false, ClockMs(2)) + 300,
            0,
            "start of Karachi's day in UTC"
        );
        assert_eq!(d.local(2).0, 0);
        assert_eq!(d.edge(true, ClockMs(3)) - d.value, 0);
        assert_eq!(d.local(2).0, 1425);
        let (v, n) = d.best_overlap();
        assert!(n >= 2);
        assert_eq!(v % 15, 0);
        d.value = 600;
        assert!(d.valuetext().contains("in Karachi"));
        d.quiet = Some((1320, 420));
        d.value = 1380;
        assert!(d.quiet_now(0));
    }

    #[test]
    fn reel_physics_and_projection() {
        let mut r = Reel::new(
            ReelKind::Minute,
            (0..12).map(|m| format!("{:02}", m * 5)).collect(),
            6,
        );
        let (y, s, o) = r.project(6, 3.0).unwrap();
        assert!(y.abs() < 1e-5 && (s - 1.0).abs() < 1e-5 && (o - 1.0).abs() < 1e-5);
        assert!(r.project(6 + 5, 3.0).is_none(), "behind the cylinder");
        r.drag(-8.0);
        assert!(r.pos < 0.0 && r.pos > -1.5, "rubber past the start");
        r.release(-20.0, 1.0);
        assert_eq!(r.target, 0);
        let mut steps = 0;
        while r.tick(1.0 / 60.0, 1.0, false) {
            steps += 1;
            assert!(steps < 600);
        }
        assert!(r.resting());
        r.spin_to(9);
        r.tick(0.0, 1.0, true);
        assert_eq!(r.pos, 9.0, "reduced motion lands at once");
    }

    #[test]
    fn date_reel_values_keys_typing() {
        let today = Date::new(2026, 9, 24).unwrap();
        let mut r = DateReel::new(ReelMode::DateTime, (today, TimeOfDay::new(9, 30)), today);
        assert_eq!(r.reels.len(), 4);
        assert_eq!(r.summary(), "Today at 9:30 AM");
        r.key(0, ReelKey::Down);
        r.key(2, ReelKey::PageDown);
        assert_eq!(r.value().1, TimeOfDay::new(9, 45));
        assert_eq!(r.summary(), "Tomorrow at 9:45 AM");
        assert!(r.typeahead(3, 'p', ClockMs(0)));
        assert_eq!(r.value().1.h, 21);
        r.key(1, ReelKey::Up);
        r.key(1, ReelKey::Up);
        assert_eq!(r.value().1.h, 19, "8 PM → 7 PM");
        let got = r.tick(1.0, false);
        assert!(got.is_none() || got == Some(r.value()));
        r.preset(1);
        assert_eq!(r.active_preset(), Some(1));
        assert!(r.enter_text("in 12 days 6:15 pm"));
        assert_eq!(r.relative(), "In 12 days");
        assert_eq!(r.value().1, TimeOfDay::new(18, 15));
        let mut dm = DateReel::new(
            ReelMode::Date,
            (Date::new(2026, 1, 31).unwrap(), TimeOfDay::default()),
            today,
        );
        dm.key(0, ReelKey::Down);
        assert_eq!(
            dm.value().0,
            Date::new(2026, 2, 28).unwrap(),
            "turns back to the last real day"
        );
        assert!(!dm.reels[1].valid[29]);
        dm.typeahead(0, 'm', ClockMs(0));
        dm.typeahead(0, 'a', ClockMs(100));
        assert_eq!(dm.value().0.m, 3);
        dm.typeahead(0, 'y', ClockMs(200));
        assert_eq!(dm.value().0.m, 5, "may");
        let tm = DateReel::new(ReelMode::Time, (today, TimeOfDay::new(14, 5)), today);
        assert_eq!(tm.summary(), "2:05 PM");
        assert_eq!(tm.valuetext(0), "2");
    }
}
