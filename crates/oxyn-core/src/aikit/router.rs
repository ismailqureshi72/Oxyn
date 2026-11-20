//! Ticket router: support messages from Gmail, Intercom, Slack and GitHub arrive one at a time;
//! an AI classifier labels each with intent, sentiment and confidence, and the ticket moves into
//! a team lane. Low confidence goes to a review lane for a person.

use super::nth;

/// One ticket takes this long (arrive, classify, route), ms.
pub const TICKET_MS: u64 = 2_000;

/// Lanes: name, hue.
pub const LANES: [(&str, &str); 4] = [
    ("Billing", "amber"),
    ("Bugs", "red"),
    ("Sales", "green"),
    ("Needs review", "neutral"),
];

/// Tickets: source mark, sender (fictional), subject, intent, sentiment, confidence %, lane.
pub const TICKETS: [(&str, &str, &str, &str, &str, u32, usize); 8] = [
    (
        "gmail",
        "Ines Varga",
        "Charged twice for October",
        "Refund",
        "Negative",
        96,
        0,
    ),
    (
        "intercom",
        "Teo Brandt",
        "Export to CSV hangs at 90%",
        "Bug report",
        "Frustrated",
        91,
        1,
    ),
    (
        "slack",
        "Priya Sethi",
        "Pricing for 40 seats?",
        "Quote",
        "Positive",
        88,
        2,
    ),
    (
        "github",
        "okapi-bot",
        "Webhook retries fire twice",
        "Bug report",
        "Neutral",
        84,
        1,
    ),
    (
        "gmail",
        "Jonah Weiss",
        "Can we pause our plan?",
        "Plan change",
        "Neutral",
        58,
        3,
    ),
    (
        "intercom",
        "Rina Okafor",
        "Invoice needs our VAT number",
        "Invoice",
        "Neutral",
        93,
        0,
    ),
    (
        "slack",
        "Tomas Lind",
        "Demo for our ops team",
        "Demo request",
        "Positive",
        90,
        2,
    ),
    (
        "gmail",
        "Sam Okoro",
        "Is my data stored in the EU?",
        "Question",
        "Neutral",
        61,
        3,
    ),
];

/// A ticket's phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TicketSt {
    /// Not arrived.
    Waiting,
    /// Arriving.
    Arriving,
    /// Being classified.
    Classifying,
    /// Labelled, moving to its lane.
    Routing,
    /// In its lane.
    Routed,
}

/// The live state of the router.
#[derive(Clone, Debug, PartialEq)]
pub struct Router {
    /// Run start.
    pub start: u64,
    /// Paused at (time frozen).
    pub paused: Option<u64>,
    /// Paused time accumulated, ms.
    pub held: u64,
    /// Below this confidence a ticket goes to review, %.
    pub threshold: u32,
    /// Ticket reviewed by a person into a lane (index, lane).
    pub reviewed: Vec<(usize, usize)>,
    /// Speed (1 or 2).
    pub speed: u64,
}

impl Router {
    /// Starts at `now`; `paused` starts held.
    pub fn new(paused: bool, threshold: u32, now: u64) -> Self {
        Self {
            start: now,
            paused: paused.then_some(now),
            held: 0,
            threshold,
            reviewed: vec![],
            speed: 1,
        }
    }

    /// Run time at `now`.
    pub fn rt(&self, now: u64) -> u64 {
        let t = self.paused.unwrap_or(now);
        t.saturating_sub(self.start).saturating_sub(self.held) * self.speed
    }

    /// Lane of ticket `i` after the threshold and reviews.
    pub fn lane(&self, i: usize) -> usize {
        if let Some(r) = self.reviewed.iter().find(|r| r.0 == i) {
            return r.1;
        }
        let t = TICKETS[i];
        if t.5 < self.threshold {
            3
        } else if t.6 == 3 {
            [0, 0, 0, 0, 0, 0, 0, 2][i]
        } else {
            t.6
        }
    }

    /// Phase of ticket `i`.
    pub fn st(&self, i: usize, now: u64) -> TicketSt {
        let t = self.rt(now);
        let s = i as u64 * TICKET_MS;
        if t < s {
            TicketSt::Waiting
        } else if t < s + 220 {
            TicketSt::Arriving
        } else if t < s + 1_100 {
            TicketSt::Classifying
        } else if t < s + 1_600 {
            TicketSt::Routing
        } else {
            TicketSt::Routed
        }
    }

    /// The ticket being worked (or the last one).
    pub fn current(&self, now: u64) -> Option<usize> {
        (0..TICKETS.len())
            .rev()
            .find(|&i| self.st(i, now) != TicketSt::Waiting && self.st(i, now) != TicketSt::Routed)
    }

    /// Routed tickets per lane.
    pub fn counts(&self, now: u64) -> [usize; 4] {
        let mut c = [0; 4];
        for i in 0..TICKETS.len() {
            if self.st(i, now) == TicketSt::Routed {
                c[self.lane(i)] += 1;
            }
        }
        c
    }

    /// Tickets in lane `l` at `now`, newest first.
    pub fn in_lane(&self, l: usize, now: u64) -> Vec<usize> {
        (0..TICKETS.len())
            .rev()
            .filter(|&i| self.st(i, now) == TicketSt::Routed && self.lane(i) == l)
            .collect()
    }

    /// All tickets routed.
    pub fn finished(&self, now: u64) -> bool {
        self.st(TICKETS.len() - 1, now) == TicketSt::Routed
    }

    /// Presses: `rt-pause` (pause / resume), `rt-restart`, `rt-speed`, `rt-review-<i>-<lane>`,
    /// `rt-th-<n>` (threshold %).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(rest) = id.strip_prefix("rt-review-") {
            let (i, l) = rest.split_once('-')?;
            let (i, l): (usize, usize) = (i.parse().ok()?, l.parse().ok()?);
            if i >= TICKETS.len() || l > 2 || self.lane(i) != 3 {
                return None;
            }
            self.reviewed.push((i, l));
            return Some(format!("{} moved to {}", TICKETS[i].2, LANES[l].0));
        }
        if let Some(n) = nth(id, "rt-th-") {
            self.threshold = (n as u32).min(99);
            return Some(format!("Review below {}% confidence", self.threshold));
        }
        match id {
            "rt-pause" => {
                if let Some(p) = self.paused.take() {
                    self.held += now.saturating_sub(p);
                    Some("Routing resumed".into())
                } else {
                    self.paused = Some(now);
                    Some("Routing paused".into())
                }
            }
            "rt-restart" => {
                self.start = now;
                self.held = 0;
                self.paused = None;
                self.reviewed.clear();
                Some("Routing from the start".into())
            }
            "rt-speed" => {
                let t = self.rt(now);
                self.speed = if self.speed == 1 { 2 } else { 1 };
                // Keep the run time where it was.
                let base = self.paused.unwrap_or(now);
                self.held = 0;
                self.start = base.saturating_sub(t / self.speed);
                Some(format!("Speed {}x", self.speed))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_classifies_routes_pauses_and_reviews() {
        let mut r = Router::new(false, 70, 0);
        assert_eq!(r.st(0, 100), TicketSt::Arriving);
        assert_eq!(r.st(0, 500), TicketSt::Classifying);
        assert_eq!(r.st(0, 1_300), TicketSt::Routing);
        assert_eq!(r.st(0, 2_000), TicketSt::Routed);
        assert_eq!(r.current(2_500), Some(1));
        r.press("rt-pause", 3_000);
        assert_eq!(r.rt(9_000), 3_000);
        r.press("rt-pause", 9_000);
        assert_eq!(r.rt(10_000), 4_000);
        let end = 100_000;
        assert!(r.finished(end));
        assert_eq!(r.counts(end), [2, 2, 2, 2]);
        r.press("rt-review-4-0", end);
        assert_eq!(r.counts(end)[0], 3);
        r.press("rt-th-50", end);
        assert_eq!(r.lane(7), 2);
        r.press("rt-speed", end);
        assert_eq!(r.speed, 2);
        r.press("rt-restart", end);
        assert_eq!(r.st(0, end), TicketSt::Arriving);
        assert_eq!(r.st(1, end), TicketSt::Waiting);
    }
}
