//! Provider status: health of the AI providers a product depends on. Each row has the
//! provider's mark, its health as an icon and a word, median latency and a 30-day uptime bar
//! with a readout per day; a row with an incident opens its updates, and the card checks again
//! on a visible countdown. The sample plays a story over simulated checks.

use super::{hash, norm_key, nth};

/// Health of a provider or a day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    /// Fine.
    Operational,
    /// Slow or partly failing.
    Degraded,
    /// Down.
    Outage,
    /// No data for the day.
    NoData,
}

impl Health {
    /// The word.
    pub fn word(self) -> &'static str {
        match self {
            Health::Operational => "Operational",
            Health::Degraded => "Degraded",
            Health::Outage => "Outage",
            Health::NoData => "No data",
        }
    }
    /// The hue.
    pub fn hue(self) -> &'static str {
        match self {
            Health::Operational => "green",
            Health::Degraded => "amber",
            Health::Outage => "red",
            Health::NoData => "neutral",
        }
    }
    /// The icon.
    pub fn icon(self) -> &'static str {
        match self {
            Health::Operational => "check-circle",
            Health::Degraded => "alert",
            Health::Outage => "x-circle",
            Health::NoData => "minus",
        }
    }
}

/// Providers: name, mark id, detail line, typical p50 latency ms.
pub const PROVIDERS: [(&str, &str, &str, u32); 5] = [
    ("OpenAI", "openai", "GPT-5 · GPT-5 mini", 420),
    (
        "Anthropic",
        "anthropic",
        "Claude Sonnet 4.5 · Haiku 4.5",
        380,
    ),
    ("Google Gemini", "gemini", "Gemini 2.5 Pro · Flash", 340),
    ("Mistral", "mistral", "Mistral Large · Codestral", 290),
    ("DeepSeek", "deepseek", "DeepSeek V3.1", 510),
];

/// Incident stages, oldest first.
pub const STAGES: [&str; 4] = ["Investigating", "Identified", "Monitoring", "Resolved"];

/// The story repeats every this many checks.
pub const STORY: usize = 11;
/// The check that fails, in the story.
pub const FAILING_CHECK: usize = 9;
/// A check takes this long, ms.
pub const CHECK_MS: u64 = 900;
/// Days in the uptime bar.
pub const DAYS: usize = 30;

/// An incident: title, and its updates newest first as (stage, time, text).
#[derive(Clone, Debug, PartialEq)]
pub struct Incident {
    /// Title.
    pub title: &'static str,
    /// Updates, newest first.
    pub updates: Vec<(&'static str, &'static str, &'static str)>,
}

/// Status of provider `p` at story check `s` (health, plus its incident while open or freshly closed).
pub fn story(p: usize, s: usize) -> (Health, Option<Incident>) {
    let up = |n: usize, list: &[(&'static str, &'static str, &'static str)]| {
        let mut v: Vec<_> = list[..n.min(list.len())].to_vec();
        v.reverse();
        v
    };
    match p {
        0 => {
            let ups = [
                (
                    "Investigating",
                    "09:12",
                    "Requests to the chat endpoint are failing for most callers.",
                ),
                (
                    "Identified",
                    "09:26",
                    "A bad config push in one region; rolling it back.",
                ),
                (
                    "Monitoring",
                    "09:41",
                    "Error rates are back near normal; watching closely.",
                ),
                (
                    "Resolved",
                    "10:05",
                    "All endpoints have been healthy for 20 minutes.",
                ),
            ];
            let (h, n) = match s {
                0 => (Health::Outage, 1),
                1 => (Health::Outage, 2),
                2 => (Health::Degraded, 3),
                3..=5 => (Health::Operational, 4),
                _ => (Health::Operational, 0),
            };
            let inc = (n > 0).then(|| Incident {
                title: "Elevated error rates on chat completions",
                updates: up(n, &ups),
            });
            (h, inc)
        }
        2 => {
            let ups = [
                (
                    "Investigating",
                    "08:50",
                    "Some Flash requests take several seconds longer than usual.",
                ),
                (
                    "Monitoring",
                    "09:30",
                    "Capacity added; latency is coming down.",
                ),
                ("Resolved", "09:58", "Latency is back to normal."),
            ];
            let (h, n) = match s {
                0 | 1 => (Health::Degraded, 1),
                2 => (Health::Degraded, 2),
                3 | 4 => (Health::Operational, 3),
                _ => (Health::Operational, 0),
            };
            let inc = (n > 0).then(|| Incident {
                title: "Higher latency on Flash",
                updates: up(n, &ups),
            });
            (h, inc)
        }
        3 => {
            let ups = [
                (
                    "Investigating",
                    "11:02",
                    "Fine-tuning jobs fail to start for some accounts.",
                ),
                (
                    "Identified",
                    "11:15",
                    "A scheduler update caused the failures; a fix is deploying.",
                ),
                ("Resolved", "11:31", "Jobs start normally again."),
            ];
            let (h, n) = match s {
                5 => (Health::Degraded, 1),
                6 => (Health::Degraded, 2),
                7 | 8 => (Health::Operational, 3),
                _ => (Health::Operational, 0),
            };
            let inc = (n > 0).then(|| Incident {
                title: "Fine-tuning jobs failing to start",
                updates: up(n, &ups),
            });
            (h, inc)
        }
        _ => (Health::Operational, None),
    }
}

/// Health of day `d` (0 oldest, 29 today) for provider `p` at story check `s`.
pub fn day(p: usize, d: usize, s: usize) -> Health {
    if d == DAYS - 1 {
        return story(p, s).0;
    }
    if p == 4 && d < 3 {
        return Health::NoData;
    }
    let h = hash(&format!("{p}-{d}")) % 100;
    match h {
        0..=2 => Health::Outage,
        3..=8 => Health::Degraded,
        _ => Health::Operational,
    }
}

/// A day's date label (`Oct 26`), counting back from Nov 24.
pub fn day_label(d: usize) -> String {
    let back = (DAYS - 1 - d) as i64;
    let (mut m, mut day) = (11usize, 24i64 - back);
    if day <= 0 {
        m = 10;
        day += 31;
    }
    let mon = if m == 11 { "Nov" } else { "Oct" };
    format!("{mon} {day}")
}

/// 30-day uptime percent (`99.82%`).
pub fn uptime(p: usize, s: usize) -> f64 {
    let mut lost = 0.0f64;
    let mut n = 0.0f64;
    for d in 0..DAYS {
        match day(p, d, s) {
            Health::Outage => lost += 0.9,
            Health::Degraded => lost += 0.2,
            Health::NoData => continue,
            Health::Operational => {}
        }
        n += 1.0;
    }
    100.0 - lost / n.max(1.0)
}

/// p50 latency at check `s`.
pub fn latency(p: usize, s: usize) -> u32 {
    let base = PROVIDERS[p].3;
    let h = story(p, s).0;
    let j = (hash(&format!("l{p}{s}")) % 60) as u32;
    match h {
        Health::Outage => base * 3 + j,
        Health::Degraded => base * 2 + j,
        _ => base + j - 30,
    }
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct ProvStatus {
    /// Ms between checks.
    pub interval: u64,
    /// Checks completed when the countdown last restarted.
    pub base: usize,
    /// The countdown's restart.
    pub anchor: u64,
    /// Open incident row.
    pub open: Option<usize>,
    /// Uptime readout: provider and day.
    pub cursor: Option<(usize, usize)>,
    /// Story checks are skipped to this offset (variants start mid-story).
    pub offset: usize,
}

impl ProvStatus {
    /// A card at story check `offset`, counting down from `now`.
    pub fn new(interval: u64, offset: usize, open: Option<usize>, now: u64) -> Self {
        Self {
            interval: interval.max(CHECK_MS * 3),
            base: 0,
            anchor: now,
            open,
            cursor: None,
            offset,
        }
    }

    fn started(&self, now: u64) -> (u64, bool) {
        let e = now.saturating_sub(self.anchor);
        let s = e / self.interval;
        (s, s >= 1 && e % self.interval < CHECK_MS)
    }

    /// A check is in flight.
    pub fn checking(&self, now: u64) -> bool {
        self.started(now).1
    }

    /// Checks completed.
    pub fn completed(&self, now: u64) -> usize {
        let (s, busy) = self.started(now);
        self.base + s as usize - usize::from(busy)
    }

    /// The story check of the last completed check, and whether that check failed (the data
    /// then stays at the check before).
    pub fn check(&self, now: u64) -> (usize, bool) {
        let s = (self.completed(now) + self.offset) % STORY;
        if s == FAILING_CHECK {
            (s - 1, true)
        } else {
            (s, false)
        }
    }

    /// Seconds to the next check.
    pub fn countdown(&self, now: u64) -> u64 {
        let e = now.saturating_sub(self.anchor) % self.interval;
        (self.interval - e).div_ceil(1000)
    }

    /// The countdown ring's fill, 0..1 (drains as the check nears).
    pub fn ring(&self, now: u64) -> f64 {
        let e = now.saturating_sub(self.anchor) % self.interval;
        1.0 - e as f64 / self.interval as f64
    }

    /// The header summary (`All systems operational`, `1 outage · 1 degraded`).
    pub fn summary(&self, now: u64) -> String {
        let (s, _) = self.check(now);
        let hs: Vec<Health> = (0..PROVIDERS.len()).map(|p| story(p, s).0).collect();
        let o = hs.iter().filter(|h| **h == Health::Outage).count();
        let d = hs.iter().filter(|h| **h == Health::Degraded).count();
        match (o, d) {
            (0, 0) => "All systems operational".into(),
            (0, d) => format!("{d} degraded · {} operational", PROVIDERS.len() - d),
            (o, 0) => format!("{o} outage · {} operational", PROVIDERS.len() - o),
            (o, d) => format!("{o} outage · {d} degraded"),
        }
    }

    /// The worst health right now.
    pub fn worst(&self, now: u64) -> Health {
        let (s, _) = self.check(now);
        let hs: Vec<Health> = (0..PROVIDERS.len()).map(|p| story(p, s).0).collect();
        if hs.contains(&Health::Outage) {
            Health::Outage
        } else if hs.contains(&Health::Degraded) {
            Health::Degraded
        } else {
            Health::Operational
        }
    }

    /// Presses: `ps-refresh`, `ps-row-<i>` (open or close an incident).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ps-row-") {
            let (s, _) = self.check(now);
            story(i, s).1?;
            if self.open == Some(i) {
                self.open = None;
                return Some(format!("{} incident closed", PROVIDERS[i].0));
            }
            self.open = Some(i);
            return Some(format!("{} incident opened", PROVIDERS[i].0));
        }
        if id == "ps-refresh" && !self.checking(now) {
            self.base = self.completed(now);
            self.anchor = now.saturating_sub(self.interval);
            return Some("Checking providers. Simulated, no requests were sent.".into());
        }
        None
    }

    /// The readout of one day (`OpenAI, Oct 26: Degraded`).
    pub fn day_readout(&self, p: usize, d: usize, now: u64) -> String {
        let (s, _) = self.check(now);
        format!(
            "{}, {}: {}",
            PROVIDERS[p].0,
            day_label(d),
            day(p, d, s).word()
        )
    }

    /// Keys on provider `p`'s uptime bar: Left, Right, Home, End move the day; Escape hides it.
    pub fn key(&mut self, p: usize, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        if key == "Escape" {
            self.cursor.take()?;
            return Some("Day readout hidden".into());
        }
        let cur = self.cursor.filter(|c| c.0 == p).map(|c| c.1);
        let d = match (key, cur) {
            ("ArrowLeft", None) | ("End", _) | ("ArrowRight", None) => DAYS - 1,
            ("ArrowLeft", Some(d)) => d.saturating_sub(1),
            ("ArrowRight", Some(d)) => (d + 1).min(DAYS - 1),
            ("Home", _) => 0,
            _ => return None,
        };
        self.cursor = Some((p, d));
        Some(self.day_readout(p, d, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_story_and_readouts() {
        let mut s = ProvStatus::new(10_000, 0, None, 0);
        assert_eq!(s.check(0).0, 0);
        assert_eq!(s.worst(0), Health::Outage);
        assert!(s.summary(0).contains("outage"));
        assert!(s.checking(10_100));
        assert_eq!(s.completed(10_100), 0);
        assert_eq!(s.completed(11_000), 1);
        s.press("ps-refresh", 12_000);
        assert!(s.checking(12_100));
        assert_eq!(s.completed(13_000), 2);
        s.press("ps-row-0", 13_000);
        assert_eq!(s.open, Some(0));
        assert!(s.press("ps-row-1", 13_000).is_none());
        s.key(0, "ArrowLeft", 13_000);
        assert_eq!(s.cursor, Some((0, DAYS - 1)));
        s.key(0, "Home", 13_000);
        assert_eq!(s.cursor, Some((0, 0)));
        s.key(0, "Escape", 13_000);
        assert_eq!(s.cursor, None);
        let f = ProvStatus::new(10_000, FAILING_CHECK, None, 0);
        assert!(f.check(0).1);
        let ok = ProvStatus::new(10_000, 10, None, 0);
        assert_eq!(ok.summary(0), "All systems operational");
        assert!(uptime(0, 0) > 98.0);
        assert_eq!(day_label(DAYS - 1), "Nov 24");
        assert_eq!(day_label(0), "Oct 26");
        assert_eq!(story(0, 1).1.unwrap().updates[0].0, "Identified");
    }
}
