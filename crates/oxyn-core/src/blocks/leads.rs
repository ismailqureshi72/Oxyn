//! Lead scoring: inbound leads ranked by an AI fit score with a ring and a one-line reason. Open
//! a lead for its signals and weighted score breakdown, then push it to a CRM. Rescore animates
//! the rings and can move a lead; sort by score or newest.

use super::{nth, rove};

/// A lead: company, person, role, score, reason, minutes since it arrived, signals and the
/// breakdown (factor, points out of its weight, weight).
pub type Lead = (
    &'static str,
    &'static str,
    &'static str,
    u32,
    &'static str,
    u32,
    &'static [&'static str],
    &'static [(&'static str, u32, u32)],
);

/// Sample leads.
pub const LEADS: [Lead; 6] = [
    (
        "Halden Freight",
        "Mira Solberg",
        "VP Operations",
        92,
        "300-person logistics team, asked for SSO and a demo this week",
        14,
        &["Pricing page ×3", "SSO", "Budget this quarter"],
        &[
            ("Company fit", 38, 40),
            ("Intent", 32, 35),
            ("Engagement", 22, 25),
        ],
    ),
    (
        "Orbitly",
        "Jonas Petri",
        "Head of Product",
        81,
        "Series B, trial workspace has 12 active members",
        52,
        &["Active trial", "12 members", "Integrations page"],
        &[
            ("Company fit", 30, 40),
            ("Intent", 29, 35),
            ("Engagement", 22, 25),
        ],
    ),
    (
        "Brightwell Clinics",
        "Sana Iqbal",
        "IT Director",
        74,
        "Large team, but needs a HIPAA review first",
        95,
        &["Security docs", "HIPAA", "Webinar"],
        &[
            ("Company fit", 34, 40),
            ("Intent", 22, 35),
            ("Engagement", 18, 25),
        ],
    ),
    (
        "Kestrel Studio",
        "Leo Varga",
        "Founder",
        58,
        "Small agency comparing plans, price sensitive",
        8,
        &["Pricing page", "Free plan"],
        &[
            ("Company fit", 18, 40),
            ("Intent", 24, 35),
            ("Engagement", 16, 25),
        ],
    ),
    (
        "Umberfield Bank",
        "Clara Nyberg",
        "Procurement",
        46,
        "Enterprise, but the request came from procurement only",
        180,
        &["RFP template", "No champion"],
        &[
            ("Company fit", 30, 40),
            ("Intent", 9, 35),
            ("Engagement", 7, 25),
        ],
    ),
    (
        "Fernway Schools",
        "Tom Achebe",
        "Teacher",
        23,
        "Education email, single user, wants a free licence",
        33,
        &["Free plan", "Single user"],
        &[
            ("Company fit", 6, 40),
            ("Intent", 10, 35),
            ("Engagement", 7, 25),
        ],
    ),
];

/// CRM destinations: name and brand.
pub const DESTS: [(&str, &str); 2] = [("HubSpot", "hubspot"), ("Airtable", "airtable")];

/// Sort orders.
pub const SORTS: [&str; 2] = ["Score", "Newest"];

/// Ring count-up and rescore time, ms.
pub const RING_MS: u64 = 900;
/// Push time, ms.
pub const PUSH_MS: u64 = 1_100;

/// Score tier: label and hue.
pub fn tier(score: u32) -> (&'static str, &'static str) {
    if score >= 80 {
        ("Hot", "green")
    } else if score >= 55 {
        ("Warm", "amber")
    } else {
        ("Cold", "neutral")
    }
}

/// A push state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Push {
    /// Pushing to a destination since.
    Pushing(usize, u64),
    /// Failed (Retry).
    Failed(usize),
}

/// The card.
#[derive(Clone, Debug, PartialEq)]
pub struct Leads {
    /// Sort.
    pub sort: usize,
    /// Scores (rescore changes them).
    pub scores: [u32; 6],
    /// Rings count up from.
    pub ring_at: u64,
    /// Open lead.
    pub open: Option<usize>,
    /// Focused row (keyboard).
    pub focus: usize,
    /// Destination.
    pub dest: usize,
    /// Push per lead.
    pub push: [Option<Push>; 6],
    /// The first push fails (error scenario).
    pub fail: bool,
    rescored: u32,
}

impl Leads {
    /// A card at `now`; `intro` counts the rings up.
    pub fn new(intro: bool, fail: bool, open: Option<usize>, now: u64) -> Self {
        Self {
            sort: 0,
            scores: LEADS.map(|l| l.3),
            ring_at: if intro { now } else { 0 },
            open,
            focus: 0,
            dest: 0,
            push: [None; 6],
            fail,
            rescored: 0,
        }
    }

    /// Lead order (ids).
    pub fn order(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..LEADS.len()).collect();
        if self.sort == 0 {
            v.sort_by(|a, b| self.scores[*b].cmp(&self.scores[*a]));
        } else {
            v.sort_by_key(|&i| LEADS[i].5);
        }
        v
    }

    /// Score shown at `now` (counting up after the intro or a rescore).
    pub fn shown(&self, i: usize, now: u64) -> u32 {
        let f = super::frac(now.saturating_sub(self.ring_at), RING_MS);
        (f * f64::from(self.scores[i])).round() as u32
    }

    /// Pushed to (destination) at `now`.
    pub fn pushed(&self, i: usize, now: u64) -> Option<usize> {
        match self.push[i] {
            Some(Push::Pushing(d, t)) if now >= t + PUSH_MS => Some(d),
            _ => None,
        }
    }

    /// Pushing at `now`.
    pub fn pushing(&self, i: usize, now: u64) -> bool {
        matches!(self.push[i], Some(Push::Pushing(_, t)) if now < t + PUSH_MS)
    }

    /// Presses: `ld-sort-<i>`, `ld-rescore`, `ld-open-<i>`, `ld-dest-<i>`, `ld-push`, `ld-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ld-rescore" => {
                self.rescored += 1;
                let k = (self.rescored as usize * 3) % LEADS.len();
                self.scores[k] = (self.scores[k] + 9).min(97);
                self.ring_at = now;
                let pos = self.order().iter().position(|&x| x == k).unwrap_or(0) + 1;
                return Some(format!(
                    "Rescored. {} is now {} and ranks {pos}. Simulated",
                    LEADS[k].0, self.scores[k]
                ));
            }
            "ld-push" | "ld-retry" => {
                let i = self.open?;
                if self.pushing(i, now) || self.pushed(i, now).is_some() {
                    return None;
                }
                if self.fail {
                    self.fail = false;
                    self.push[i] = Some(Push::Failed(self.dest));
                    return Some(format!("Couldn't reach {}. Retry", DESTS[self.dest].0));
                }
                self.push[i] = Some(Push::Pushing(self.dest, now));
                return Some(format!(
                    "Pushing {} to {}. Simulated",
                    LEADS[i].0, DESTS[self.dest].0
                ));
            }
            _ => {}
        }
        if let Some(i) = nth(id, "ld-sort-") {
            SORTS.get(i)?;
            self.sort = i;
            return Some(format!("Sorted by {}", SORTS[i].to_lowercase()));
        }
        if let Some(i) = nth(id, "ld-open-") {
            LEADS.get(i)?;
            self.focus = self.order().iter().position(|&x| x == i).unwrap_or(0);
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return self
                .open
                .map(|_| format!("{}, score {}: {}", LEADS[i].0, self.scores[i], LEADS[i].4));
        }
        if let Some(i) = nth(id, "ld-dest-") {
            DESTS.get(i)?;
            self.dest = i;
            return Some(format!("Push to {}", DESTS[i].0));
        }
        None
    }

    /// Keys: Up/Down/Home/End move the focused row, Enter or Space opens it, Left/Right switch
    /// the destination in an open row.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        let ord = self.order();
        match k {
            "ArrowUp" | "ArrowDown" | "Home" | "End" => {
                self.focus = rove(self.focus, ord.len(), k);
                let i = ord[self.focus];
                Some(format!("{}, score {}", LEADS[i].0, self.scores[i]))
            }
            "Enter" | "Space" => self.press(
                &format!("ld-open-{}", ord[self.focus.min(ord.len() - 1)]),
                now,
            ),
            "ArrowLeft" | "ArrowRight" if self.open.is_some() => {
                let d = rove(self.dest, DESTS.len(), k);
                self.press(&format!("ld-dest-{d}"), now)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_open_push() {
        let mut l = Leads::new(true, true, None, 0);
        assert_eq!(l.shown(0, 450), 46);
        assert_eq!(l.order()[0], 0);
        l.press("ld-sort-1", 0);
        assert_eq!(l.order()[0], 3);
        l.key("down", 0);
        l.key("enter", 0);
        assert_eq!(l.open, Some(0));
        l.key("right", 0);
        assert_eq!(l.dest, 1);
        l.press("ld-push", 0);
        assert_eq!(l.push[0], Some(Push::Failed(1)));
        l.press("ld-retry", 100);
        assert!(l.pushing(0, 500));
        assert_eq!(l.pushed(0, 2_000), Some(1));
        l.press("ld-rescore", 0);
        assert_eq!(l.scores[3], 67);
        assert_eq!(tier(92).0, "Hot");
    }
}
