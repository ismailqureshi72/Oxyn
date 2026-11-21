//! Referral card: a two-sided referral program in one card. Your personal link with Copy and
//! posting buttons, an address field where each email becomes a chip, three running totals
//! (invited, joined, credit earned), a segmented bonus bar and the newest invites with where each
//! one stands.

use super::nth;

/// The invite link: domain part (muted) and code.
pub const LINK: (&str, &str) = ("https://ledgerly.app/r/", "maya20");
/// Credit each side gets, USD.
pub const REWARD: u64 = 20;
/// Paid referrals for the milestone, and its reward.
pub const MILESTONE: (usize, &str) = (4, "3 months of Pro free");
/// Most addresses one send can hold.
pub const MAX_INVITES: usize = 10;
/// Sending takes this long, ms.
pub const SEND_MS: u64 = 900;

/// A referral's stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// Invited.
    Invited,
    /// Signed up.
    SignedUp,
    /// Subscribed; both sides earned.
    Earned,
}

impl Stage {
    /// The word.
    pub fn word(self) -> &'static str {
        match self {
            Stage::Invited => "Invited",
            Stage::SignedUp => "Signed up",
            Stage::Earned => "Earned $20",
        }
    }
    /// The icon.
    pub fn icon(self) -> &'static str {
        match self {
            Stage::Invited => "mail",
            Stage::SignedUp => "users",
            Stage::Earned => "check-circle",
        }
    }
}

/// The sample program: name (or empty), email, stage, date; newest first.
pub const SAMPLE: [(&str, &str, Stage, &str); 5] = [
    ("", "dev@quillworks.io", Stage::Invited, "Nov 18"),
    ("Lena Ortiz", "lena@ortiz.studio", Stage::Invited, "Nov 15"),
    (
        "Tomas Varga",
        "tomas@vargadesign.co",
        Stage::SignedUp,
        "Nov 9",
    ),
    ("Sam Okoro", "sam@okoro.dev", Stage::Earned, "Nov 2"),
    ("Hana Sato", "hana@satolabs.jp", Stage::Earned, "Oct 27"),
];

/// Simulated steps: (ms after start, sample index, new stage).
pub const STORY: [(u64, usize, Stage); 4] = [
    (2_600, 2, Stage::Earned),
    (4_800, 1, Stage::SignedUp),
    (7_000, 1, Stage::Earned),
    (9_400, 0, Stage::SignedUp),
];

/// One row in the list.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Name, or empty.
    pub name: String,
    /// Email.
    pub email: String,
    /// Stage.
    pub stage: Stage,
    /// Date or `Just now`.
    pub when: String,
}

/// A plausible email address.
pub fn valid(e: &str) -> bool {
    let Some((a, d)) = e.split_once('@') else {
        return false;
    };
    !a.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.') && !e.contains(' ')
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Referral {
    /// Simulation start (None when not simulating).
    pub t0: Option<u64>,
    /// Chips in the field.
    pub chips: Vec<String>,
    /// The hint under the field and whether it is an error.
    pub hint: (String, bool),
    /// Sent invites (newest first) and when the send started.
    pub sent: Vec<(String, u64)>,
    /// Sending since.
    pub sending: Option<u64>,
    /// Copied at.
    pub copied: Option<u64>,
    /// Sends fail (the error scenario).
    pub fail: bool,
}

impl Referral {
    /// A card simulating from `now` when `simulate`.
    pub fn new(simulate: bool, fail: bool, now: u64) -> Self {
        Self {
            t0: simulate.then_some(now),
            chips: Vec::new(),
            hint: ("Press Enter or comma after each address".into(), false),
            sent: Vec::new(),
            sending: None,
            copied: None,
            fail,
        }
    }

    /// Everyone invited, newest first.
    pub fn rows(&self, now: u64) -> Vec<Row> {
        let el = self.t0.map(|t| now.saturating_sub(t));
        let mut out: Vec<Row> = self
            .sent
            .iter()
            .filter(|s| now >= s.1 + SEND_MS)
            .map(|(e, _)| Row {
                name: String::new(),
                email: e.clone(),
                stage: Stage::Invited,
                when: "Just now".into(),
            })
            .collect();
        for (i, (n, e, st, d)) in SAMPLE.iter().enumerate() {
            let mut stage = *st;
            if let Some(el) = el {
                for (at, j, s) in STORY {
                    if j == i && el >= at {
                        stage = s;
                    }
                }
            }
            out.push(Row {
                name: n.to_string(),
                email: e.to_string(),
                stage,
                when: d.to_string(),
            });
        }
        out
    }

    /// Funnel: invited, signed up (or further), earned dollars.
    pub fn funnel(&self, now: u64) -> (usize, usize, u64) {
        let r = self.rows(now);
        let su = r.iter().filter(|x| x.stage >= Stage::SignedUp).count();
        let ea = r.iter().filter(|x| x.stage == Stage::Earned).count();
        (r.len(), su, ea as u64 * REWARD)
    }

    /// Paid referrals toward the milestone.
    pub fn paid(&self, now: u64) -> usize {
        self.rows(now)
            .iter()
            .filter(|x| x.stage == Stage::Earned)
            .count()
    }

    /// The milestone line.
    pub fn milestone(&self, now: u64) -> String {
        let p = self.paid(now);
        if p >= MILESTONE.0 {
            format!("Unlocked: {}", MILESTONE.1)
        } else {
            let left = MILESTONE.0 - p;
            format!(
                "{left} more paid referral{} for {}",
                if left == 1 { "" } else { "s" },
                MILESTONE.1
            )
        }
    }

    /// The send is in flight.
    pub fn busy(&self, now: u64) -> bool {
        self.sending.is_some_and(|t| now < t + SEND_MS)
    }

    /// Turns `text` into chips (split on commas and spaces); returns what was left over.
    pub fn add(&mut self, text: &str) -> Option<String> {
        let mut said = None;
        for part in text
            .split([',', ' ', ';'])
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            if self.chips.len() >= MAX_INVITES {
                self.hint = (
                    format!("One send holds up to {MAX_INVITES} addresses"),
                    true,
                );
                return Some(self.hint.0.clone());
            }
            if !valid(part) {
                self.hint = (format!("{part} doesn't look like an email address"), true);
                return Some(self.hint.0.clone());
            }
            if self.chips.iter().any(|c| c == part) {
                continue;
            }
            self.chips.push(part.to_string());
            self.hint = (format!("{} ready to invite", self.chips.len()), false);
            said = Some(format!("Added {part}"));
        }
        said
    }

    /// Presses: `rf-copy`, `rf-share-<x|slack|email>`, `rf-send`, `rf-chip-<i>` (remove),
    /// `rf-sample` (adds two sample addresses).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "rf-chip-") {
            if i < self.chips.len() {
                let c = self.chips.remove(i);
                return Some(format!("Removed {c}"));
            }
            return None;
        }
        if let Some(ch) = id.strip_prefix("rf-share-") {
            let name = match ch {
                "x" => "X",
                "slack" => "Slack",
                _ => "email",
            };
            return Some(format!(
                "Share on {name} would open here. Simulated, nothing was posted."
            ));
        }
        match id {
            "rf-copy" => {
                self.copied = Some(now);
                Some("Invite link copied".into())
            }
            "rf-sample" => self.add("ana@fernpoint.co, li.wei@harborline.io"),
            "rf-send" if !self.busy(now) => {
                if self.chips.is_empty() {
                    self.hint = ("Add at least one address".into(), true);
                    return Some(self.hint.0.clone());
                }
                if self.fail {
                    self.fail = false;
                    self.hint = (
                        "Couldn't send. Your addresses are kept, try again.".into(),
                        true,
                    );
                    return Some(self.hint.0.clone());
                }
                let n = self.chips.len();
                for c in self.chips.drain(..).rev() {
                    self.sent.insert(0, (c, now));
                }
                self.sending = Some(now);
                self.hint = (
                    format!("Invited {n} {}", if n == 1 { "person" } else { "people" }),
                    false,
                );
                Some(self.hint.0.clone())
            }
            _ => None,
        }
    }

    /// Keys in the email field with its current text: Enter, comma, space and Tab make a chip;
    /// Enter on an empty field sends; Backspace on an empty field removes the last chip.
    /// Returns what to announce and whether the field should clear.
    pub fn key(&mut self, key: &str, draft: &str, now: u64) -> (Option<String>, bool) {
        let empty = draft.trim().is_empty();
        match key {
            "Enter" | "enter" if empty => (self.press("rf-send", now), false),
            "Enter" | "enter" | "," | " " | "Tab" | "tab" | "space" if !empty => {
                let said = self.add(draft);
                let ok = !self.hint.1;
                (said, ok)
            }
            "Backspace" | "backspace" if empty => {
                let c = self.chips.pop();
                (c.map(|c| format!("Removed {c}")), false)
            }
            _ => (None, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chips_send_and_funnel() {
        let mut r = Referral::new(true, false, 0);
        assert_eq!(r.funnel(0), (5, 3, 40));
        assert_eq!(r.paid(0), 2);
        assert!(r.milestone(0).starts_with("2 more"));
        assert_eq!(r.paid(8_000), 4);
        assert!(r.milestone(8_000).starts_with("Unlocked"));
        let (s, clear) = r.key("Enter", "ana@fernpoint.co", 0);
        assert!(s.is_some() && clear);
        let (_, clear) = r.key(",", "not-an-email", 0);
        assert!(!clear && r.hint.1);
        r.key("Backspace", "", 0);
        assert!(r.chips.is_empty());
        r.press("rf-sample", 0);
        assert_eq!(r.chips.len(), 2);
        r.key("Enter", "", 100);
        assert!(r.busy(200));
        assert_eq!(r.rows(2_000).len(), 7);
        assert_eq!(r.rows(2_000)[0].when, "Just now");
        let mut f = Referral::new(false, true, 0);
        f.press("rf-sample", 0);
        f.press("rf-send", 0);
        assert_eq!(f.chips.len(), 2);
        assert!(valid("a@b.co") && !valid("a@b") && !valid("ab.co"));
    }
}
