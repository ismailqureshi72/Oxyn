//! Upgrade prompt: the nudge shown when someone hits a limit. A usage meter, the price with a
//! monthly and yearly switch, what the next plan unlocks (with the model marks it names), and
//! Upgrade or Maybe later. Upgrading lands on a success state with the new limits; Maybe later
//! leaves a calm reminder with a way back. Every state shares one footprint.

use super::{norm_key, nth, thousands};

/// The plan on offer: name, monthly price, yearly price per month, its limit.
pub const PLAN: (&str, u32, u32, u64) = ("Pro", 20, 16, 2_000);
/// What the plan unlocks: text and the marks it names.
pub const FEATURES: [(&str, &[&str]); 3] = [
    ("2,000 AI credits every month", &[]),
    (
        "Claude Opus 4.1 and GPT-5 in every chat",
        &["claude", "openai"],
    ),
    ("Image generation with Gemini", &["gemini"]),
];
/// The plan's limits shown after upgrading.
pub const LIMITS: [(&str, &str); 3] = [
    ("AI credits", "2,000 / month"),
    ("Projects", "Unlimited"),
    ("Chat history", "1 year"),
];
/// Upgrading takes this long, ms.
pub const UPGRADE_MS: u64 = 1_200;

/// Where the card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The offer.
    Offer,
    /// Charging.
    Upgrading,
    /// Done.
    Upgraded,
    /// Snoozed with a reminder.
    Later,
    /// The charge failed.
    Error,
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Upgrade {
    /// Used.
    pub used: u64,
    /// The current plan's limit.
    pub limit: u64,
    /// What is counted.
    pub unit: &'static str,
    /// When the limit resets.
    pub resets: &'static str,
    /// Yearly billing.
    pub yearly: bool,
    /// Upgrade pressed at.
    pub started: Option<u64>,
    /// Maybe later pressed.
    pub later: bool,
    /// The charge fails (the error scenario).
    pub fail: bool,
}

impl Upgrade {
    /// A card at `used` of `limit`.
    pub fn new(used: u64, limit: u64, unit: &'static str, yearly: bool, fail: bool) -> Self {
        Self {
            used,
            limit: limit.max(1),
            unit,
            resets: "Dec 1",
            yearly,
            started: None,
            later: false,
            fail,
        }
    }

    /// The phase at `now`.
    pub fn phase(&self, now: u64) -> Phase {
        if self.later {
            return Phase::Later;
        }
        match self.started {
            None => Phase::Offer,
            Some(t) if now < t + UPGRADE_MS => Phase::Upgrading,
            Some(_) if self.fail => Phase::Error,
            Some(_) => Phase::Upgraded,
        }
    }

    /// Price per month for the interval.
    pub fn price(&self) -> u32 {
        if self.yearly { PLAN.2 } else { PLAN.1 }
    }

    /// Charged today.
    pub fn due(&self) -> u32 {
        if self.yearly { PLAN.2 * 12 } else { PLAN.1 }
    }

    /// The billing note (`$192 billed yearly`).
    pub fn billing(&self) -> String {
        if self.yearly {
            format!("${} billed yearly · save 20%", self.due())
        } else {
            "Billed monthly · cancel anytime".into()
        }
    }

    /// At or over the limit.
    pub fn at_limit(&self) -> bool {
        self.used >= self.limit
    }

    /// Title.
    pub fn title(&self, now: u64) -> String {
        match self.phase(now) {
            Phase::Upgraded => format!("You're on {}", PLAN.0),
            Phase::Later => "We'll leave it there".into(),
            _ if self.at_limit() => {
                format!("You've used all {} {}", thousands(self.limit), self.unit)
            }
            _ => format!(
                "You've used {} of {} {}",
                thousands(self.used),
                thousands(self.limit),
                self.unit
            ),
        }
    }

    /// The line under the title.
    pub fn description(&self, now: u64) -> String {
        match self.phase(now) {
            Phase::Upgraded => format!(
                "Your new limits apply now. {} {} left this month.",
                thousands(PLAN.3 - self.used.min(PLAN.3)),
                self.unit
            ),
            Phase::Later => format!(
                "Your {} reset on {}. You can upgrade any time before then.",
                self.unit, self.resets
            ),
            Phase::Error => "Your card was declined. Nothing was charged.".into(),
            _ => format!(
                "Upgrade to {} to keep going, or wait until {} for a fresh {}.",
                PLAN.0,
                self.resets,
                thousands(self.limit)
            ),
        }
    }

    /// The meter: used, max and its words (`Limit reached`, `12 AI credits left`).
    pub fn meter(&self, now: u64) -> (u64, u64, String) {
        if self.phase(now) == Phase::Upgraded {
            return (
                self.used,
                PLAN.3,
                format!("{} of {} used", thousands(self.used), thousands(PLAN.3)),
            );
        }
        let words = if self.at_limit() {
            "Limit reached".to_string()
        } else {
            format!("{} {} left", thousands(self.limit - self.used), self.unit)
        };
        (self.used.min(self.limit), self.limit, words)
    }

    /// Presses: `up-int-<0|1>`, `up-upgrade`, `up-later`, `up-again`, `up-continue`, `up-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "up-int-") {
            let y = i == 1;
            if y == self.yearly || self.phase(now) != Phase::Offer {
                return None;
            }
            self.yearly = y;
            return Some(format!(
                "{} billing, ${} a month",
                if y { "Yearly" } else { "Monthly" },
                self.price()
            ));
        }
        match id {
            "up-upgrade" | "up-retry" if matches!(self.phase(now), Phase::Offer | Phase::Error) => {
                if id == "up-retry" {
                    self.fail = false;
                }
                self.started = Some(now);
                Some(format!("Upgrading to {}", PLAN.0))
            }
            "up-later" if self.phase(now) == Phase::Offer => {
                self.later = true;
                Some(format!(
                    "Reminder set. Your {} reset on {}.",
                    self.unit, self.resets
                ))
            }
            "up-again" if self.later => {
                self.later = false;
                Some(format!("{} offer shown again", PLAN.0))
            }
            "up-continue" if self.phase(now) == Phase::Upgraded => Some("Back to work".into()),
            _ => None,
        }
    }

    /// Keys on the billing switch: Left, Right, Home, End.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        match norm_key(key) {
            "ArrowLeft" | "Home" | "ArrowUp" => self.press("up-int-0", now),
            "ArrowRight" | "End" | "ArrowDown" => self.press("up-int-1", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offer_upgrade_and_later() {
        let mut u = Upgrade::new(50, 50, "AI credits", true, false);
        assert_eq!(u.title(0), "You've used all 50 AI credits");
        assert_eq!(u.meter(0).2, "Limit reached");
        assert_eq!(u.price(), 16);
        u.key("ArrowLeft", 0);
        assert_eq!(u.price(), 20);
        u.press("up-upgrade", 100);
        assert_eq!(u.phase(200), Phase::Upgrading);
        assert_eq!(u.phase(2_000), Phase::Upgraded);
        assert_eq!(u.meter(2_000).1, 2_000);
        let mut l = Upgrade::new(38, 50, "AI credits", false, false);
        assert_eq!(l.meter(0).2, "12 AI credits left");
        l.press("up-later", 0);
        assert_eq!(l.phase(0), Phase::Later);
        l.press("up-again", 0);
        assert_eq!(l.phase(0), Phase::Offer);
        let mut f = Upgrade::new(3, 3, "seats", false, true);
        f.press("up-upgrade", 0);
        assert_eq!(f.phase(5_000), Phase::Error);
        f.press("up-retry", 5_000);
        assert_eq!(f.phase(9_000), Phase::Upgraded);
    }
}
