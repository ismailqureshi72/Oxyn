//! Trial banner: how much of a trial is left, what the workspace loses without paying, and the
//! way to keep it, as a slim banner, a sidebar card or a header pill. The last days switch to a
//! warning tone that names the end day; expired and upgraded have their own copy; dismissing or
//! snoozing folds it into the pill, which opens it again.

use super::{norm_key, nth, rove};

/// Days in the trial.
pub const TRIAL_DAYS: i64 = 14;
/// The plan on trial.
pub const PLAN: &str = "Pro";
/// The workspace.
pub const WORKSPACE: &str = "Halcyon";
/// What the workspace loses, short (for the banner).
pub const LOSS: &str = "SSO and 12 seats";
/// What the workspace loses: label, detail, mark or icon.
pub const FEATURES: [(&str, &str, &str); 5] = [
    ("Single sign-on", "Okta and Google Workspace", "lock"),
    ("Slack alerts", "3 channels", "slack"),
    ("Linear sync", "2-way issues", "linear"),
    ("GitHub checks", "On every pull request", "github"),
    ("12 seats", "Free keeps 3", "users"),
];
/// Snooze choices.
pub const SNOOZES: [&str; 3] = ["Tomorrow", "In 3 days", "When the warning starts"];
/// Adding a card takes this long, ms.
pub const PAY_MS: u64 = 1_100;
/// Today (the sample's anchor): 2026-11-21.
pub const TODAY: (i64, u32, u32) = (2026, 11, 21);

/// The trial's tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Plenty left.
    Active,
    /// The last days.
    Warning,
    /// Ended.
    Expired,
    /// Paid.
    Upgraded,
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

fn days_in(y: i64, m: u32) -> u32 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The date `d` days after today: (month name, day, weekday).
pub fn date_after(d: i64) -> (&'static str, u32, &'static str) {
    let (mut y, mut m, mut day) = TODAY;
    let mut left = d.max(0);
    while left > 0 {
        day += 1;
        if day > days_in(y, m) {
            day = 1;
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        left -= 1;
    }
    // 2026-11-21 is a Saturday (index 5).
    let wd = ((5 + d.max(0)) % 7) as usize;
    (MONTHS[m as usize - 1], day, WEEKDAYS[wd])
}

/// The live state.
#[derive(Clone, Debug, PartialEq)]
pub struct Trial {
    /// Whole days left.
    pub days: i64,
    /// Warning starts at this many days.
    pub warn_at: i64,
    /// Folded into the pill.
    pub folded: bool,
    /// The pill's popover is open.
    pub popover: bool,
    /// The snooze menu is open, with the active option.
    pub menu: Option<usize>,
    /// Add payment pressed at.
    pub paying: Option<u64>,
    /// The last simulated note and when.
    pub note: Option<(String, u64)>,
    /// Adding a card fails once (the error scenario).
    pub fail: bool,
}

impl Trial {
    /// A trial with `days` left.
    pub fn new(days: i64, warn_at: i64, folded: bool, fail: bool) -> Self {
        Self {
            days,
            warn_at,
            folded,
            popover: false,
            menu: None,
            paying: None,
            note: None,
            fail,
        }
    }

    /// The tone at `now`.
    pub fn tone(&self, now: u64) -> Tone {
        if self.paying.is_some_and(|t| now >= t + PAY_MS && !self.fail) {
            Tone::Upgraded
        } else if self.days <= 0 {
            Tone::Expired
        } else if self.days <= self.warn_at {
            Tone::Warning
        } else {
            Tone::Active
        }
    }

    /// Adding a card is in flight.
    pub fn busy(&self, now: u64) -> bool {
        self.paying.is_some_and(|t| now < t + PAY_MS)
    }

    /// Payment failed (after the attempt).
    pub fn failed(&self, now: u64) -> bool {
        self.fail && self.paying.is_some_and(|t| now >= t + PAY_MS)
    }

    /// The ring's fill (days left of the trial).
    pub fn ring(&self, now: u64) -> f64 {
        match self.tone(now) {
            Tone::Upgraded => 1.0,
            _ => (self.days.max(0) as f64 / TRIAL_DAYS as f64).min(1.0),
        }
    }

    /// Title and the line under it.
    pub fn copy(&self, now: u64) -> (String, String) {
        let (mon, day, wd) = date_after(self.days);
        match self.tone(now) {
            Tone::Active => (
                format!("{} days left in your {PLAN} trial", self.days),
                format!("After {mon} {day}, {WORKSPACE} loses {LOSS}."),
            ),
            Tone::Warning => (
                if self.days == 1 {
                    format!("Your {PLAN} trial ends tomorrow")
                } else {
                    format!("Your {PLAN} trial ends {wd}")
                },
                format!("Add a payment method by {mon} {day} to keep {LOSS}."),
            ),
            Tone::Expired => (
                format!("Your {PLAN} trial has ended"),
                format!("{WORKSPACE} is on Free now. Add a payment method to get {LOSS} back."),
            ),
            Tone::Upgraded => (
                format!("You're on {PLAN}"),
                "Your card is on file and nothing changes for your team.".into(),
            ),
        }
    }

    /// The pill's label (`9 days left`).
    pub fn pill(&self, now: u64) -> String {
        match self.tone(now) {
            Tone::Upgraded => PLAN.to_string(),
            Tone::Expired => "Trial ended".into(),
            _ if self.days == 1 => "1 day left".into(),
            _ => format!("{} days left", self.days),
        }
    }

    /// The note, while it is fresh (4 s).
    pub fn note_at(&self, now: u64) -> Option<String> {
        self.note
            .as_ref()
            .filter(|n| now < n.1 + 4_000)
            .map(|n| n.0.clone())
    }

    fn say(&mut self, s: String, now: u64) -> Option<String> {
        self.note = Some((s.clone(), now));
        Some(s)
    }

    /// Presses: `tb-compare`, `tb-pay`, `tb-manage`, `tb-snooze`, `tb-snooze-<i>`,
    /// `tb-dismiss`, `tb-pill`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "tb-snooze-") {
            self.menu = None;
            self.folded = true;
            self.popover = false;
            return self.say(
                format!(
                    "Snoozed until {}. Simulated.",
                    SNOOZES[i.min(2)].to_lowercase()
                ),
                now,
            );
        }
        match id {
            "tb-compare" => self.say("The plans page would open here. Simulated.".into(), now),
            "tb-manage" => self.say("Billing settings would open here. Simulated.".into(), now),
            "tb-pay" if !self.busy(now) && self.tone(now) != Tone::Upgraded => {
                if self.failed(now) {
                    self.fail = false;
                }
                self.paying = Some(now);
                Some("Adding a payment method".into())
            }
            "tb-snooze" => {
                self.menu = if self.menu.is_some() { None } else { Some(0) };
                None
            }
            "tb-dismiss" => {
                self.folded = true;
                self.popover = false;
                self.menu = None;
                self.say("Dismissed. The trial pill stays in the header.".into(), now)
            }
            "tb-pill" => {
                self.popover = !self.popover;
                None
            }
            _ => None,
        }
    }

    /// Keys in the snooze menu (arrows, Home, End, Enter, Escape) or the popover (Escape).
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        if let Some(i) = self.menu {
            return match key {
                "Escape" => {
                    self.menu = None;
                    None
                }
                "Enter" | "Space" => self.press(&format!("tb-snooze-{i}"), now),
                k => {
                    self.menu = Some(rove(i, 3, k));
                    Some(SNOOZES[self.menu.unwrap_or(0)].to_string())
                }
            };
        }
        if key == "Escape" && self.popover {
            self.popover = false;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_copy_and_folding() {
        let mut t = Trial::new(9, 3, false, false);
        assert_eq!(t.tone(0), Tone::Active);
        assert!(t.copy(0).1.contains("Nov 30"));
        let w = Trial::new(3, 3, false, false);
        assert_eq!(w.tone(0), Tone::Warning);
        assert!(w.copy(0).0.ends_with("Tuesday"));
        assert_eq!(Trial::new(0, 3, false, false).tone(0), Tone::Expired);
        t.press("tb-snooze", 0);
        t.key("ArrowDown", 0);
        t.key("Enter", 0);
        assert!(t.folded && t.menu.is_none());
        t.press("tb-pill", 0);
        assert!(t.popover);
        t.press("tb-pay", 100);
        assert!(t.busy(200));
        assert_eq!(t.tone(2_000), Tone::Upgraded);
        let mut f = Trial::new(2, 3, false, true);
        f.press("tb-pay", 0);
        assert!(f.failed(2_000));
        f.press("tb-pay", 2_000);
        assert_eq!(f.tone(4_000), Tone::Upgraded);
        assert_eq!(date_after(10), ("Dec", 1, "Tuesday"));
    }
}
