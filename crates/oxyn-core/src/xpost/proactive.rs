//! Proactive card: the assistant drops an offer into a feed when it notices something worth acting
//! on, such as a subscription about to renew on a plan that is too big. Accepting runs a few
//! visible steps in place and the card resolves into a result with Review and Dismiss; Not now opens a snooze menu and the
//! card folds into one line with Undo; the close button removes it.

use crate::aikit::{nth, rove};

/// The offer: (source, source mark, title, body).
pub const OFFER: (&str, &str, &str, &str) = (
    "From your calendar",
    "googlecalendar",
    "Fernpoint renews in 3 days",
    "Your Team plan renews on Friday at $49 a month. You used 3 of 10 seats this quarter. Want me to compare plans and draft a downgrade?",
);
/// Steps run after accepting: (doing, done).
pub const STEPS: [(&str, &str); 3] = [
    ("Reading the last 3 invoices", "Read 3 invoices"),
    ("Comparing 3 plans", "Compared 3 plans"),
    ("Drafting the downgrade request", "Drafted the request"),
];
/// Ms per step.
pub const STEP_MS: u64 = 1_100;
/// The result: (headline, detail).
pub const RESULT: (&str, &str) = (
    "Starter plan saves $216 a year",
    "Covers your 3 seats. Draft ready to send to billing@fernpoint.example.",
);
/// Snooze choices.
pub const SNOOZE: [&str; 3] = ["Tomorrow morning", "Next week", "Don't suggest this again"];

/// Where the card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Offering.
    Offer,
    /// Working through the steps.
    Working,
    /// Result shown.
    Result,
    /// Snoozed (folded).
    Snoozed,
    /// Removed.
    Gone,
    /// A step failed.
    Failed,
}

/// The card.
#[derive(Clone, Debug, PartialEq)]
pub struct Proactive {
    /// Accepted at.
    pub accepted: Option<u64>,
    /// Snooze choice.
    pub snoozed: Option<usize>,
    /// Snooze menu open with this focused.
    pub menu: Option<usize>,
    /// Removed.
    pub gone: bool,
    /// Steps fail at the second one (error scenario).
    pub fail: bool,
    /// The draft was opened.
    pub reviewing: bool,
}

impl Proactive {
    /// A card.
    pub fn new(fail: bool) -> Self {
        Self {
            accepted: None,
            snoozed: None,
            menu: None,
            gone: false,
            fail,
            reviewing: false,
        }
    }

    /// Steps finished so far.
    pub fn steps_done(&self, now: u64) -> usize {
        self.accepted
            .map(|a| ((now.saturating_sub(a) / STEP_MS) as usize).min(STEPS.len()))
            .unwrap_or(0)
    }

    /// The phase.
    pub fn phase(&self, now: u64) -> Phase {
        if self.gone {
            return Phase::Gone;
        }
        if self.snoozed.is_some() {
            return Phase::Snoozed;
        }
        match self.accepted {
            None => Phase::Offer,
            Some(_) => {
                let d = self.steps_done(now);
                if self.fail && d >= 1 {
                    Phase::Failed
                } else if d >= STEPS.len() {
                    Phase::Result
                } else {
                    Phase::Working
                }
            }
        }
    }

    /// Progress 0..1 while working.
    pub fn progress(&self, now: u64) -> f64 {
        self.accepted
            .map(|a| {
                (now.saturating_sub(a) as f64 / (STEP_MS * STEPS.len() as u64) as f64).min(1.0)
            })
            .unwrap_or(0.0)
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "pc-accept" | "pc-retry" => {
                self.accepted = Some(now);
                self.fail = id == "pc-accept" && self.fail;
                self.menu = None;
                Some(STEPS[0].0.to_string())
            }
            "pc-later" => {
                self.menu = if self.menu.is_some() { None } else { Some(0) };
                self.menu.map(|_| "Snooze options".to_string())
            }
            "pc-close" => {
                self.gone = true;
                self.menu = None;
                Some("Suggestion removed".into())
            }
            "pc-undo" => {
                self.snoozed = None;
                self.gone = false;
                Some("Suggestion is back".into())
            }
            "pc-review" => {
                self.reviewing = !self.reviewing;
                Some(
                    if self.reviewing {
                        "Draft opened"
                    } else {
                        "Draft closed"
                    }
                    .into(),
                )
            }
            "pc-reset" => {
                *self = Self::new(self.fail);
                Some("Suggestion reset".into())
            }
            _ => {
                let i = nth(id, "pc-snooze-")?;
                if i >= SNOOZE.len() {
                    return None;
                }
                self.snoozed = Some(i);
                self.menu = None;
                Some(format!("Snoozed: {}", SNOOZE[i]))
            }
        }
    }

    /// The folded line.
    pub fn folded(&self) -> String {
        match self.snoozed {
            Some(2) => "Hidden for good".into(),
            Some(i) => format!("Snoozed until {}", SNOOZE[i].to_lowercase()),
            None => String::new(),
        }
    }

    /// Keys in the snooze menu: arrows move, Enter picks, Escape closes.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        let m = self.menu?;
        match k {
            "Escape" => {
                self.menu = None;
                Some("Closed".into())
            }
            "Enter" | "Space" => self.press(&format!("pc-snooze-{m}"), now),
            _ => {
                let i = rove(m, SNOOZE.len(), k);
                self.menu = Some(i);
                Some(SNOOZE[i].into())
            }
        }
    }
}
