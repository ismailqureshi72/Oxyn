//! Number ticker (`number-ticker.md`): odometer columns that roll up when the value grows and
//! down when it shrinks, new columns that slide in, count-in from zero on first view. Also the
//! number formatting (locale grouping, compact) and the reaction bar's counts and pop.

use std::time::Duration;

use oxyn_tokens::CubicBezier;

use crate::{ClockMs, ease::Ease};

/// Digit grouping and decimal mark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Locale {
    /// `1,234.5`.
    #[default]
    En,
    /// `1.234,5`.
    De,
    /// `1 234,5` (narrow no-break space).
    Fr,
    /// `1234.5`.
    Plain,
}

impl Locale {
    /// From a BCP-47 tag's language.
    pub fn from_tag(tag: &str) -> Self {
        match tag.split(['-', '_']).next().unwrap_or("") {
            "de" | "es" | "it" | "nl" | "pt" | "id" | "tr" => Self::De,
            "fr" | "ru" | "pl" | "sv" | "fi" | "nb" | "cs" | "uk" => Self::Fr,
            _ => Self::En,
        }
    }
    fn marks(self) -> (Option<char>, char) {
        match self {
            Self::En => (Some(','), '.'),
            Self::De => (Some('.'), ','),
            Self::Fr => (Some('\u{202f}'), ','),
            Self::Plain => (None, '.'),
        }
    }
}

/// How a number is written.
#[derive(Clone, Debug, PartialEq)]
pub enum NumberFormat {
    /// Whole number with grouping.
    Integer,
    /// Fixed decimals.
    Decimal(u8),
    /// `value` is a fraction; shown ×100 with `%` and the given decimals.
    Percent(u8),
    /// Prefix + fixed decimals ("$", 2).
    Currency(String, u8),
    /// Oxyn plus: `1.2k`, `3.4M` (one decimal under 10 units).
    Compact,
}

/// Formats `v`.
pub fn format(v: f64, f: &NumberFormat, locale: Locale) -> String {
    let (group, dec) = locale.marks();
    let fixed = |v: f64, d: u8| -> String {
        let s = format!("{:.*}", d as usize, v.abs());
        let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
        let mut out = String::new();
        let n = int.len();
        for (i, ch) in int.chars().enumerate() {
            if i > 0
                && (n - i) % 3 == 0
                && let Some(g) = group
            {
                out.push(g);
            }
            out.push(ch);
        }
        if !frac.is_empty() {
            out.push(dec);
            out.push_str(frac);
        }
        if v < 0.0 && s.chars().any(|c| c != '0' && c != '.') {
            format!("-{out}")
        } else {
            out
        }
    };
    match f {
        NumberFormat::Integer => fixed(v.round(), 0),
        NumberFormat::Decimal(d) => fixed(v, *d),
        NumberFormat::Percent(d) => format!("{}%", fixed(v * 100.0, *d)),
        NumberFormat::Currency(sym, d) => {
            let s = fixed(v, *d);
            match s.strip_prefix('-') {
                Some(rest) => format!("-{sym}{rest}"),
                None => format!("{sym}{s}"),
            }
        }
        NumberFormat::Compact => compact(v, locale),
    }
}

/// `999`, `1.2k`, `12k`, `1.2M`, `3.4B`.
pub fn compact(v: f64, locale: Locale) -> String {
    let (_, dec) = locale.marks();
    let a = v.abs();
    let (n, unit) = if a >= 1e9 {
        (a / 1e9, "B")
    } else if a >= 1e6 {
        (a / 1e6, "M")
    } else if a >= 1e3 {
        (a / 1e3, "k")
    } else {
        return format!("{}{}", if v < 0.0 { "-" } else { "" }, a.round() as u64);
    };
    let body = if n < 10.0 {
        let t = (n * 10.0).floor() / 10.0;
        let s = format!("{t:.1}");
        let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        s.replace('.', &dec.to_string())
    } else {
        format!("{}", n.floor() as u64)
    };
    format!("{}{body}{unit}", if v < 0.0 { "-" } else { "" })
}

/// Ticker motion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TickerMode {
    /// Odometer columns (affectUI).
    #[default]
    Roll,
    /// Oxyn plus: the whole number counts through intermediate values.
    CountUp,
}

/// Ticker timings.
pub mod timing {
    use std::time::Duration;
    /// affectUI default duration.
    pub const DURATION: Duration = Duration::from_millis(900);
    /// Per-column stagger, right to left.
    pub const STAGGER: Duration = oxyn_tokens::motion::stagger::DIGIT;
}

/// One column at a moment.
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    /// A rolling digit: `pos` in `0..10` (fractional while moving; the strip shows
    /// `floor(pos)` and the next digit `pos.fract()` of the way in). `up` = rolling upward.
    Digit {
        /// Position on the 0–9 strip.
        pos: f32,
        /// Direction.
        up: bool,
        /// Final digit (for the accessible / static text).
        to: u8,
    },
    /// A column entering (`k` 0→1: width and opacity grow, it slides in from below / above).
    Enter {
        /// Glyph.
        ch: char,
        /// Progress.
        k: f32,
    },
    /// A column leaving (`k` 1→0).
    Exit {
        /// Glyph.
        ch: char,
        /// Remaining.
        k: f32,
    },
    /// Separator, sign or unit; crossfades when it changes.
    Static(char),
}

/// A number ticker.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberTicker {
    /// Format.
    pub format: NumberFormat,
    /// Locale.
    pub locale: Locale,
    /// Roll duration (900 ms).
    pub duration: Duration,
    /// Delay before the roll (0).
    pub delay: Duration,
    /// Mode.
    pub mode: TickerMode,
    /// Oxyn plus: easing.
    pub easing: CubicBezier,
    from: f64,
    to: f64,
    at: ClockMs,
    seen: bool,
}

impl NumberTicker {
    /// A ticker resting on `value` (not yet seen: the first [`NumberTicker::enter_view`] counts
    /// in from zero).
    pub fn new(value: f64, format: NumberFormat) -> Self {
        Self {
            format,
            locale: Locale::En,
            duration: timing::DURATION,
            delay: Duration::ZERO,
            mode: TickerMode::Roll,
            easing: oxyn_tokens::motion::ease::OUT,
            from: value,
            to: value,
            at: ClockMs(0),
            seen: false,
        }
    }
    /// The value it is heading to.
    pub fn value(&self) -> f64 {
        self.to
    }
    /// Scrolled into view: the first time, count in from 0.
    pub fn enter_view(&mut self, now: ClockMs) {
        if !self.seen {
            self.seen = true;
            self.from = 0.0;
            self.at = now;
        }
    }
    /// New value. A change mid-roll starts from where it is.
    pub fn set(&mut self, v: f64, now: ClockMs) {
        if (v - self.to).abs() < f64::EPSILON {
            return;
        }
        self.from = self.current(now);
        self.to = v;
        self.at = now;
        self.seen = true;
    }
    fn k(&self, now: ClockMs, reduced: bool) -> f32 {
        if reduced {
            return 1.0;
        }
        let t = now.since(self.at).saturating_sub(self.delay).as_secs_f32();
        let d = self.duration.as_secs_f32().max(1e-3);
        self.easing.ease((t / d).clamp(0.0, 1.0))
    }
    /// Numeric value now (for CountUp, and for re-targeting).
    pub fn current(&self, now: ClockMs) -> f64 {
        self.from + (self.to - self.from) * f64::from(self.k(now, false))
    }
    /// Still moving.
    pub fn moving(&self, now: ClockMs) -> bool {
        now.since(self.at) < self.delay + self.duration
            && (self.from - self.to).abs() > f64::EPSILON
    }
    /// The final text (accessible name; never the in-between).
    pub fn text(&self) -> String {
        format(self.to, &self.format, self.locale)
    }
    /// Columns at `now`.
    pub fn columns(&self, now: ClockMs, reduced: bool) -> Vec<Column> {
        let k = self.k(now, reduced);
        if self.mode == TickerMode::CountUp || k >= 1.0 {
            let v = if k >= 1.0 { self.to } else { self.current(now) };
            return format(v, &self.format, self.locale)
                .chars()
                .map(|c| match c.to_digit(10) {
                    Some(d) => Column::Digit {
                        pos: d as f32,
                        up: true,
                        to: d as u8,
                    },
                    None => Column::Static(c),
                })
                .collect();
        }
        let up = self.to >= self.from;
        let old: Vec<char> = format(self.from, &self.format, self.locale)
            .chars()
            .collect();
        let new: Vec<char> = format(self.to, &self.format, self.locale).chars().collect();
        let n = old.len().max(new.len());
        let pad = |v: &[char], i: usize| -> Option<char> {
            let off = n - v.len();
            (i >= off).then(|| v[i - off])
        };
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let (o, w) = (pad(&old, i), pad(&new, i));
            // Columns further left start later: stagger right → left.
            let lag = (n - 1 - i) as f32 * timing::STAGGER.as_secs_f32()
                / self.duration.as_secs_f32().max(1e-3);
            let kk = ((k - lag) / (1.0 - lag).max(0.2)).clamp(0.0, 1.0);
            out.push(match (o.and_then(|c| c.to_digit(10)), w) {
                (Some(a), Some(c)) if c.is_ascii_digit() => {
                    let b = c.to_digit(10).unwrap_or(0);
                    let steps = if up {
                        (b + 10 - a) % 10
                    } else {
                        (a + 10 - b) % 10
                    } as f32;
                    let pos = if up {
                        a as f32 + steps * kk
                    } else {
                        a as f32 - steps * kk
                    }
                    .rem_euclid(10.0);
                    Column::Digit {
                        pos: if kk >= 1.0 { b as f32 } else { pos },
                        up,
                        to: b as u8,
                    }
                }
                (None, Some(c)) if o.is_none() => Column::Enter { ch: c, k: kk },
                (_, Some(c)) => Column::Static(c),
                (Some(_), None) | (None, None) => match o {
                    Some(c) => Column::Exit { ch: c, k: 1.0 - kk },
                    None => continue,
                },
            });
        }
        out
    }
}

// ---------------------------------------------------------------------------------------------
// Reaction bar

/// One reaction / action in the bar.
#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// Id (`like`).
    pub id: String,
    /// Label ("Like").
    pub label: String,
    /// Icon name.
    pub icon: String,
    /// Count.
    pub count: u64,
    /// The user's own reaction is on.
    pub active: bool,
    /// Colour role it fills with when active (like → danger red, repost → success green).
    pub tint: oxyn_tokens::ColorRole,
    /// Toggles (like, repost) vs one-shot (comment, share).
    pub toggles: bool,
    last_kick: Option<ClockMs>,
    kicks: u32,
}

impl Reaction {
    /// A toggling reaction.
    pub fn new(
        id: &str,
        label: &str,
        icon: &str,
        count: u64,
        tint: oxyn_tokens::ColorRole,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            count,
            active: false,
            tint,
            toggles: true,
            last_kick: None,
            kicks: 0,
        }
    }
    /// A one-shot action (comment, share).
    pub fn action(id: &str, label: &str, icon: &str, count: u64) -> Self {
        Self {
            toggles: false,
            ..Self::new(id, label, icon, count, oxyn_tokens::ColorRole::FgDefault)
        }
    }
    /// Press. Toggles adjust the count; returns the new active flag.
    pub fn press(&mut self, now: ClockMs) -> bool {
        // Quick taps stack: each one within the pop window adds to the kick (Arc).
        self.kicks = match self.last_kick {
            Some(t) if now.since(t) < POP => (self.kicks + 1).min(3),
            _ => 1,
        };
        self.last_kick = Some(now);
        if self.toggles {
            self.active = !self.active;
            if self.active {
                self.count += 1;
            } else {
                self.count = self.count.saturating_sub(1);
            }
        }
        self.active
    }
    /// Icon scale at `now`: shrink, spring past 1, settle. Stacked taps kick harder.
    pub fn pop(&self, now: ClockMs, reduced: bool) -> f32 {
        let Some(t) = self.last_kick else { return 1.0 };
        if reduced {
            return 1.0;
        }
        let p = now.since(t).as_secs_f32() / POP.as_secs_f32();
        if p >= 1.0 {
            return 1.0;
        }
        let amp = 0.18 * (1.0 + 0.35 * (self.kicks.saturating_sub(1)) as f32);
        if p < 0.2 {
            1.0 - 0.15 * (p / 0.2)
        } else {
            let q = (p - 0.2) / 0.8;
            1.0 + amp * (q * std::f32::consts::PI).sin() * (1.0 - q)
        }
    }
    /// Accessible name ("Like, 1,204 likes, on").
    pub fn aria(&self, rule: &CountRule) -> String {
        let n = rule.show(self.count);
        if self.toggles {
            format!(
                "{}, {n}{}",
                self.label,
                if self.active { ", on" } else { "" }
            )
        } else {
            format!("{}, {n}", self.label)
        }
    }
}

/// Icon pop duration.
pub const POP: Duration = Duration::from_millis(420);

/// Oxyn plus: count abbreviation rules for the bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CountRule {
    /// Abbreviate at or above this (default 10 000; 0 = never).
    pub from: u64,
    /// Hide zero counts.
    pub hide_zero: bool,
    /// Locale.
    pub locale: Locale,
}

impl Default for CountRule {
    fn default() -> Self {
        Self {
            from: 10_000,
            hide_zero: true,
            locale: Locale::En,
        }
    }
}

impl CountRule {
    /// The count text ("" when hidden).
    pub fn show(&self, n: u64) -> String {
        if n == 0 && self.hide_zero {
            String::new()
        } else if self.from > 0 && n >= self.from {
            compact(n as f64, self.locale)
        } else {
            format(n as f64, &NumberFormat::Integer, self.locale)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting() {
        assert_eq!(
            format(1234567.0, &NumberFormat::Integer, Locale::En),
            "1,234,567"
        );
        assert_eq!(
            format(1234.5, &NumberFormat::Decimal(1), Locale::De),
            "1.234,5"
        );
        assert_eq!(
            format(0.425, &NumberFormat::Percent(1), Locale::En),
            "42.5%"
        );
        assert_eq!(
            format(-12.5, &NumberFormat::Currency("$".into(), 2), Locale::En),
            "-$12.50"
        );
        assert_eq!(
            format(1234.0, &NumberFormat::Integer, Locale::Fr),
            "1\u{202f}234"
        );
        assert_eq!(compact(999.0, Locale::En), "999");
        assert_eq!(compact(1250.0, Locale::En), "1.2k");
        assert_eq!(compact(1000.0, Locale::En), "1k");
        assert_eq!(compact(12_900.0, Locale::En), "12k");
        assert_eq!(compact(3_400_000.0, Locale::De), "3,4M");
        assert_eq!(Locale::from_tag("de-AT"), Locale::De);
    }

    #[test]
    fn rolls_up_and_down_and_settles() {
        let mut t = NumberTicker::new(19.0, NumberFormat::Integer);
        t.set(21.0, ClockMs(0));
        let cols = t.columns(ClockMs(0), false);
        assert_eq!(
            cols[1],
            Column::Digit {
                pos: 9.0,
                up: true,
                to: 1
            }
        );
        let mid = t.columns(ClockMs(450), false);
        if let Column::Digit { pos, up, .. } = mid[1] {
            assert!(
                up && !(pos > 1.0 && pos < 9.0),
                "9 → 0 → 1 rolls through the top: {pos}"
            );
        }
        assert_eq!(
            t.columns(ClockMs(2_000), false)[1],
            Column::Digit {
                pos: 1.0,
                up: true,
                to: 1
            }
        );
        t.set(20.0, ClockMs(3_000));
        if let Column::Digit { up, .. } = t.columns(ClockMs(3_100), false)[1] {
            assert!(!up, "shrinking rolls down");
        }
        assert_eq!(t.text(), "20");
        assert!(!t.moving(ClockMs(9_000)));
    }

    #[test]
    fn new_columns_slide_in_and_count_in() {
        let mut t = NumberTicker::new(9_999.0, NumberFormat::Integer);
        t.set(10_000.0, ClockMs(0));
        let c = t.columns(ClockMs(0), false);
        assert_eq!(c.len(), 6);
        assert!(matches!(c[0], Column::Enter { ch: '1', .. }));
        let mut v = NumberTicker::new(42.0, NumberFormat::Integer);
        v.enter_view(ClockMs(0));
        assert_eq!(v.current(ClockMs(0)), 0.0);
        assert_eq!(v.current(ClockMs(5_000)), 42.0);
        v.enter_view(ClockMs(6_000));
        assert_eq!(v.current(ClockMs(6_000)), 42.0, "only the first time");
        // Reduced motion: final value at once.
        let mut r = NumberTicker::new(1.0, NumberFormat::Integer);
        r.set(7.0, ClockMs(0));
        assert_eq!(
            r.columns(ClockMs(0), true)[0],
            Column::Digit {
                pos: 7.0,
                up: true,
                to: 7
            }
        );
        // Count-up mode shows intermediate whole values.
        let mut c = NumberTicker::new(0.0, NumberFormat::Integer);
        c.mode = TickerMode::CountUp;
        c.set(100.0, ClockMs(0));
        let s: String = c
            .columns(ClockMs(300), false)
            .iter()
            .map(|x| match x {
                Column::Digit { to, .. } => char::from(b'0' + to),
                Column::Static(ch) => *ch,
                _ => '?',
            })
            .collect();
        let n: u32 = s.parse().unwrap();
        assert!(n > 0 && n < 100);
    }

    #[test]
    fn reactions() {
        let mut r = Reaction::new(
            "like",
            "Like",
            "heart",
            1203,
            oxyn_tokens::ColorRole::StatusDangerFg,
        );
        assert!(r.press(ClockMs(0)));
        assert_eq!(r.count, 1204);
        assert!(r.pop(ClockMs(40), false) < 1.0);
        assert!(r.pop(ClockMs(250), false) > 1.0);
        assert_eq!(r.pop(ClockMs(1_000), false), 1.0);
        assert!(!r.press(ClockMs(100)));
        assert_eq!(r.count, 1203);
        let rule = CountRule::default();
        assert_eq!(r.aria(&rule), "Like, 1,203");
        assert_eq!(rule.show(0), "");
        assert_eq!(rule.show(12_400), "12k");
        let mut share = Reaction::action("share", "Share", "share", 4);
        share.press(ClockMs(0));
        assert_eq!(share.count, 4);
    }
}
