//! Catch up: one button reads your tools for a time range and streams back a short ranked
//! digest (Needs you, Decisions made, Shipped, FYI). Each line names its source, people and a
//! link; marking lines as read clears them until you are all caught up.

use super::{nth, rove};

/// Time ranges: label and how many items qualify.
pub const RANGES: [&str; 3] = ["Since yesterday", "Last 3 days", "Last week"];

/// Sources read: name, brand, items read per range.
pub const SOURCES: [(&str, &str, [u32; 3]); 4] = [
    ("Slack", "slack", [41, 128, 260]),
    ("Linear", "linear", [9, 26, 61]),
    ("GitHub", "github", [14, 37, 82]),
    ("Gmail", "gmail", [11, 30, 74]),
];

/// Sections: label, icon, hue.
pub const SECTIONS: [(&str, &str, &str); 4] = [
    ("Needs you", "alert", "amber"),
    ("Decisions made", "check-circle", "violet"),
    ("Shipped", "bolt", "green"),
    ("FYI", "info", "neutral"),
];

/// Digest items: section, source index, sentence, people, link label, oldest range it shows in
/// (0 shows for every range, 2 only for the last week).
pub const ITEMS: [(usize, usize, &str, &str, &str, usize); 10] = [
    (
        0,
        2,
        "Ines asked you to review the rate limiter PR before Thursday's release.",
        "Ines Duarte",
        "PR #482",
        0,
    ),
    (
        0,
        3,
        "Northwind's renewal contract is waiting on your signature.",
        "Legal · Kai",
        "Email",
        0,
    ),
    (
        0,
        0,
        "Design wants a yes or no on dropping the dark sidebar.",
        "Mara, Theo",
        "#design",
        1,
    ),
    (
        1,
        0,
        "The team chose Postgres over DynamoDB for the events store.",
        "Platform",
        "#platform",
        0,
    ),
    (
        1,
        1,
        "Mobile release moves to Dec 3 to fit the payments fix.",
        "Ola",
        "MOB-118",
        1,
    ),
    (
        2,
        2,
        "Search suggestions shipped to all workspaces.",
        "Ravi, Jun",
        "v2.14",
        0,
    ),
    (
        2,
        1,
        "Billing export bug closed after the hotfix.",
        "Ines",
        "BIL-77",
        1,
    ),
    (
        2,
        2,
        "Onboarding checklist went to 50% of new teams.",
        "Growth",
        "PR #471",
        2,
    ),
    (
        3,
        3,
        "Quarterly planning doc is open for comments until Monday.",
        "Priya",
        "Email",
        1,
    ),
    (
        3,
        0,
        "Office closed on Dec 8 for the holiday.",
        "People team",
        "#general",
        2,
    ),
];

/// Ms to read each source.
pub const READ_MS: u64 = 550;
/// Ms between digest lines.
pub const ITEM_MS: u64 = 260;

/// The card.
#[derive(Clone, Debug, PartialEq)]
pub struct CatchUp {
    /// Range.
    pub range: usize,
    /// Run start.
    pub run: Option<u64>,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// Item ids marked as read.
    pub read: Vec<usize>,
    /// Item open (its link previewed).
    pub open: Option<usize>,
    /// GitHub fails on the first read (error scenario).
    pub fail: bool,
    /// GitHub was skipped this run.
    pub skipped: bool,
}

impl CatchUp {
    /// A card, optionally reading at `now`.
    pub fn new(range: usize, auto: bool, fail: bool, now: u64) -> Self {
        let mut c = Self {
            range: range.min(2),
            run: None,
            stopped: None,
            read: Vec::new(),
            open: None,
            fail,
            skipped: false,
        };
        if auto {
            c.press("cu-run", now);
        }
        c
    }

    /// Items in this range (ids), in section order, without skipped sources.
    pub fn items(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..ITEMS.len())
            .filter(|&i| ITEMS[i].5 <= self.range && !(self.skipped && ITEMS[i].1 == 2))
            .collect();
        v.sort_by_key(|&i| ITEMS[i].0);
        v
    }

    fn t(&self, now: u64) -> Option<u64> {
        self.run
            .map(|r| self.stopped.map_or(now, |s| s.min(now)) - r)
    }

    /// Source states at `now`: 0 waiting, 1 reading, 2 read, 3 failed.
    pub fn source(&self, i: usize, now: u64) -> u8 {
        let Some(t) = self.t(now) else { return 0 };
        let at = i as u64 * READ_MS;
        if t < at {
            0
        } else if t < at + READ_MS {
            1
        } else if self.skipped && i == 2 {
            3
        } else {
            2
        }
    }

    /// Lines streamed in at `now`.
    pub fn streamed(&self, now: u64) -> usize {
        let Some(t) = self.t(now) else { return 0 };
        let start = SOURCES.len() as u64 * READ_MS;
        if t < start {
            return 0;
        }
        (((t - start) / ITEM_MS) as usize + 1).min(self.items().len())
    }

    /// Reading or streaming.
    pub fn busy(&self, now: u64) -> bool {
        self.run.is_some() && self.stopped.is_none() && self.streamed(now) < self.items().len()
    }

    /// Lines visible and unread at `now`.
    pub fn left(&self, now: u64) -> Vec<usize> {
        self.items()
            .into_iter()
            .take(self.streamed(now))
            .filter(|i| !self.read.contains(i))
            .collect()
    }

    /// Ready and everything read.
    pub fn caught_up(&self, now: u64) -> bool {
        self.run.is_some() && !self.busy(now) && self.left(now).is_empty()
    }

    /// Presses: `cu-run` (Catch me up, Stop or Refresh), `cu-range-<i>`, `cu-open-<i>`,
    /// `cu-read-<i>`, `cu-all`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if id == "cu-run" {
            if self.busy(now) {
                self.stopped = Some(now);
                return Some("Stopped".into());
            }
            self.run = Some(now);
            self.stopped = None;
            self.read.clear();
            self.open = None;
            self.skipped = false;
            if self.fail {
                self.fail = false;
                self.skipped = true;
                return Some(format!(
                    "Reading {}. GitHub could not be reached and is skipped. Simulated, nothing was read",
                    RANGES[self.range].to_lowercase()
                ));
            }
            return Some(format!(
                "Reading Slack, Linear, GitHub and Gmail, {}. Simulated, nothing was read",
                RANGES[self.range].to_lowercase()
            ));
        }
        if id == "cu-all" {
            let l = self.left(now);
            self.read.extend(l);
            return Some("All caught up".into());
        }
        if let Some(i) = nth(id, "cu-range-") {
            if i >= RANGES.len() || self.busy(now) {
                return None;
            }
            self.range = i;
            if self.run.is_some() {
                return self.press("cu-run", now);
            }
            return Some(RANGES[i].to_string());
        }
        if let Some(i) = nth(id, "cu-open-") {
            ITEMS.get(i)?;
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return self.open.map(|_| {
                format!(
                    "Opening {} in {} (simulated)",
                    ITEMS[i].4, SOURCES[ITEMS[i].1].0
                )
            });
        }
        if let Some(i) = nth(id, "cu-read-") {
            if self.read.contains(&i) || ITEMS.get(i).is_none() {
                return None;
            }
            self.read.push(i);
            let n = self.left(now).len();
            return Some(if n == 0 {
                "All caught up".into()
            } else {
                format!("Marked as read. {n} left")
            });
        }
        None
    }

    /// Keys: Left/Right/Home/End move the range.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        if matches!(k, "ArrowLeft" | "ArrowRight" | "Home" | "End") {
            let i = rove(self.range, RANGES.len(), k);
            return self.press(&format!("cu-range-{i}"), now);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_stream_clear() {
        let mut c = CatchUp::new(1, true, false, 0);
        assert_eq!(c.source(0, 100), 1);
        assert_eq!(c.streamed(1_000), 0);
        assert!(c.busy(2_500));
        assert_eq!(c.items().len(), 8);
        assert!(!c.busy(10_000));
        for i in c.left(10_000) {
            c.press(&format!("cu-read-{i}"), 10_000);
        }
        assert!(c.caught_up(10_000));
        c.key("end", 10_000);
        assert_eq!(c.range, 2);
        assert_eq!(c.items().len(), 10);
        let mut f = CatchUp::new(0, true, true, 0);
        assert_eq!(f.source(2, 3_000), 3);
        assert!(f.items().iter().all(|&i| ITEMS[i].1 != 2));
        f.press("cu-all", 10_000);
        assert!(f.caught_up(10_000));
    }
}
