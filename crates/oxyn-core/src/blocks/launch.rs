//! AI feature launch: an in-product card that introduces an AI feature. A looping mini demo plays
//! above a three-step tour with dots (autoplay until you take over), three benefits per step, a
//! workspace switch that only admins can turn on (saving can be cancelled), and Try it now.

use super::{nth, rove};

/// Steps: title, blurb, demo lines (the first is typed, the rest appear), benefits.
pub const STEPS: [(&str, &str, [&str; 3], [&str; 3]); 3] = [
    (
        "Rewrite inline",
        "Select any text and ask for a shorter, clearer or friendlier version without leaving the page.",
        [
            "Make this shorter",
            "We will ship the export fix on Friday.",
            "Kept your tone · 42% shorter",
        ],
        [
            "Works in docs, comments and replies",
            "Keeps your formatting and links",
            "Undo puts the original back",
        ],
    ),
    (
        "Summarise threads",
        "Long threads open with a three line summary and the decisions people made.",
        [
            "Summarise this thread",
            "Decided: launch on the 3rd. Open: pricing copy.",
            "Read 64 messages in 2 seconds",
        ],
        [
            "Names who decided what",
            "Links each point to its message",
            "Updates as the thread grows",
        ],
    ),
    (
        "Ask your workspace",
        "Ask a question in plain words and get an answer with links to the pages it came from.",
        [
            "What did we promise Northwind?",
            "A 99.9% uptime credit and SSO by January.",
            "From 3 pages · 2 emails",
        ],
        [
            "Only reads what you can open",
            "Cites every source",
            "Never trains on your data",
        ],
    ),
];

/// One step on the dots, ms.
pub const STEP_MS: u64 = 6_000;
/// Saving the switch, ms.
pub const SAVE_MS: u64 = 1_200;

/// The launch card.
#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    /// Step chosen by hand (stops autoplay).
    pub manual: Option<usize>,
    /// Clock origin for autoplay and the demo loop.
    pub t0: u64,
    /// Feature on for the workspace.
    pub enabled: bool,
    /// Saving since.
    pub saving: Option<u64>,
    /// You are an admin (non-admins see the switch disabled with a note).
    pub admin: bool,
    /// Try it now pressed.
    pub tried: bool,
    /// Dismissed.
    pub dismissed: bool,
}

impl Launch {
    /// A card starting at `now`.
    pub fn new(admin: bool, enabled: bool, auto: bool, now: u64) -> Self {
        Self {
            manual: if auto { None } else { Some(0) },
            t0: now,
            enabled,
            saving: None,
            admin,
            tried: false,
            dismissed: false,
        }
    }

    /// The step shown at `now`.
    pub fn step(&self, now: u64) -> usize {
        self.manual
            .unwrap_or(((now.saturating_sub(self.t0) / STEP_MS) % STEPS.len() as u64) as usize)
    }

    /// Progress through the current step, 0..=1 (autoplay only).
    pub fn progress(&self, now: u64) -> f64 {
        if self.manual.is_some() {
            return 1.0;
        }
        (now.saturating_sub(self.t0) % STEP_MS) as f64 / STEP_MS as f64
    }

    /// The demo frame: characters of the typed line and how many result lines show.
    pub fn demo(&self, now: u64) -> (usize, usize) {
        let t = now.saturating_sub(self.t0) % STEP_MS;
        let typed = STEPS[self.step(now)].2[0].chars().count();
        let c = ((t / 45) as usize).min(typed);
        let lines = if t < typed as u64 * 45 + 400 {
            0
        } else if t < typed as u64 * 45 + 1_400 {
            1
        } else {
            2
        };
        (c, lines)
    }

    /// Saving at `now`.
    pub fn is_saving(&self, now: u64) -> bool {
        self.saving.is_some_and(|s| now < s + SAVE_MS)
    }

    /// The switch's shown state (the target while saving).
    pub fn on(&self) -> bool {
        self.enabled
    }

    /// Presses: `fl-dot-<i>`, `fl-switch` (again while saving cancels), `fl-try`, `fl-learn`,
    /// `fl-dismiss`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "fl-switch" => {
                if !self.admin {
                    return Some("Only workspace admins can turn this on".into());
                }
                if self.is_saving(now) {
                    self.saving = None;
                    self.enabled = !self.enabled;
                    return Some("Cancelled. Nothing changed".into());
                }
                self.enabled = !self.enabled;
                self.saving = Some(now);
                return Some(format!(
                    "{} for the workspace. Simulated",
                    if self.enabled {
                        "Turning on"
                    } else {
                        "Turning off"
                    }
                ));
            }
            "fl-try" => {
                self.tried = true;
                return Some(format!(
                    "Opening {} (simulated)",
                    STEPS[self.step(now)].0.to_lowercase()
                ));
            }
            "fl-learn" => return Some("Opening the guide (simulated)".into()),
            "fl-dismiss" => {
                self.dismissed = !self.dismissed;
                return Some(
                    if self.dismissed {
                        "Hidden. Find it again in What's new"
                    } else {
                        "Shown"
                    }
                    .into(),
                );
            }
            _ => {}
        }
        if let Some(i) = nth(id, "fl-dot-") {
            STEPS.get(i)?;
            self.manual = Some(i);
            self.t0 = now;
            return Some(format!("Step {} of 3: {}", i + 1, STEPS[i].0));
        }
        None
    }

    /// Keys on the dots: Left/Right/Home/End move the step and stop autoplay; Space is the switch.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match k {
            "ArrowLeft" | "ArrowRight" | "Home" | "End" => {
                let i = rove(self.step(now), STEPS.len(), k);
                self.press(&format!("fl-dot-{i}"), now)
            }
            "Space" => self.press("fl-switch", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autoplay_dots_switch() {
        let mut l = Launch::new(true, false, true, 0);
        assert_eq!(l.step(7_000), 1);
        assert_eq!(l.demo(0), (0, 0));
        assert_eq!(l.demo(5_900).1, 2);
        l.key("end", 7_000);
        assert_eq!(l.step(30_000), 2);
        l.press("fl-switch", 100);
        assert!(l.enabled && l.is_saving(500));
        l.press("fl-switch", 600);
        assert!(!l.enabled);
        let mut n = Launch::new(false, false, false, 0);
        n.press("fl-switch", 0);
        assert!(!n.enabled);
    }
}
