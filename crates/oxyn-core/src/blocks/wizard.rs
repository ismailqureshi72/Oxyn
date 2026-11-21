//! AI setup wizard: three steps to turn on an AI assistant. Connect sources and watch them sync,
//! pick the model, tone and guardrails, then ask a test question whose answer streams with
//! numbered citations from the connected sources before you go live.

use super::{first_words, nth, rove, thousands, words_at};

/// Steps.
pub const STEPS: [&str; 3] = ["Connect sources", "Behaviour", "Test and go live"];

/// Sources: name, brand, documents.
pub const SOURCES: [(&str, &str, u32); 4] = [
    ("Notion", "notion", 1_284),
    ("Google Drive", "googledrive", 3_910),
    ("Intercom", "intercom", 862),
    ("Slack", "slack", 5_420),
];

/// Models: name, brand, speed word, cost word.
pub const MODELS: [(&str, &str, &str, &str); 3] = [
    ("Claude Sonnet 4.5", "claude", "Balanced", "$$"),
    ("GPT-5 mini", "openai", "Fastest", "$"),
    ("Gemini 2.5 Pro", "gemini", "Deepest", "$$$"),
];

/// Tones.
pub const TONES: [&str; 3] = ["Friendly", "Neutral", "Formal"];

/// Guardrails: label and hint.
pub const GUARDS: [(&str, &str); 3] = [
    (
        "Only answer from sources",
        "Say so when the sources don't cover it",
    ),
    (
        "Hand off to a person",
        "Offer a human when the customer asks",
    ),
    (
        "Hide personal data",
        "Mask emails, phone numbers and card digits",
    ),
];

/// Test questions and answers: segments with the source they cite.
pub const QUESTIONS: [(&str, &[(&str, usize)]); 2] = [
    (
        "How do I reset two-factor login?",
        &[
            (
                "Open Settings, then Security, and choose Reset two-factor.",
                0,
            ),
            (
                "You will need a backup code or a confirmation email to finish.",
                1,
            ),
            ("If both are lost, support can verify you by video call.", 2),
        ],
    ),
    (
        "Can I get an invoice in euros?",
        &[
            (
                "Yes. Change the billing currency under Billing before the next invoice.",
                1,
            ),
            ("Past invoices stay in the currency they were issued in.", 0),
            (
                "Finance answers currency questions within one working day.",
                3,
            ),
        ],
    ),
];

/// Sync time per source, ms.
pub const SYNC_MS: u64 = 2_400;

/// A source's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Src {
    /// Not connected.
    Off,
    /// Syncing since.
    Syncing(u64),
    /// Failed (Retry).
    Failed,
}

/// The wizard.
#[derive(Clone, Debug, PartialEq)]
pub struct Wizard {
    /// Current step.
    pub step: usize,
    /// Furthest step reached.
    pub reached: usize,
    /// Source states.
    pub src: [Src; 4],
    /// Model.
    pub model: usize,
    /// Tone.
    pub tone: usize,
    /// Guardrails.
    pub guards: [bool; 3],
    /// Question asked and when.
    pub ask: Option<(usize, u64)>,
    /// Stopped at word n.
    pub stopped: Option<usize>,
    /// Live.
    pub live: bool,
    /// Intercom fails on its first sync (error scenario).
    pub fail: bool,
}

impl Wizard {
    /// A wizard at `start` step (sources before it already synced).
    pub fn new(start: usize, fail: bool, now: u64) -> Self {
        let mut w = Self {
            step: 0,
            reached: 0,
            src: [Src::Off; 4],
            model: 0,
            tone: 0,
            guards: [true, true, false],
            ask: None,
            stopped: None,
            live: false,
            fail,
        };
        if start > 0 {
            w.src = [Src::Syncing(0), Src::Syncing(0), Src::Off, Src::Syncing(0)];
            w.step = start.min(2);
            w.reached = w.step;
        }
        if start > 2 {
            w.ask = Some((0, 0));
            w.live = true;
        }
        let _ = now;
        w
    }

    /// Synced documents of a source at `now` and whether it is done.
    pub fn synced(&self, i: usize, now: u64) -> (u32, bool) {
        match self.src[i] {
            Src::Syncing(t) => {
                let f = super::frac(now.saturating_sub(t), SYNC_MS);
                ((f * f64::from(SOURCES[i].2)) as u32, f >= 1.0)
            }
            _ => (0, false),
        }
    }

    /// Connected and synced sources.
    pub fn ready(&self, now: u64) -> Vec<usize> {
        (0..SOURCES.len())
            .filter(|&i| self.synced(i, now).1)
            .collect()
    }

    /// Next is allowed.
    pub fn can_next(&self, now: u64) -> bool {
        match self.step {
            0 => !self.ready(now).is_empty(),
            1 => true,
            _ => self.ask.is_some() && !self.streaming(now),
        }
    }

    fn full(q: usize) -> String {
        QUESTIONS[q]
            .1
            .iter()
            .map(|s| s.0)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Words of the answer visible at `now`.
    pub fn words(&self, now: u64) -> usize {
        let Some((q, t)) = self.ask else { return 0 };
        let n = words_at(&Self::full(q), now.saturating_sub(t), 3);
        self.stopped.map_or(n, |s| s.min(n))
    }

    /// Streaming.
    pub fn streaming(&self, now: u64) -> bool {
        self.ask.is_some_and(|(q, _)| {
            self.stopped.is_none() && self.words(now) < Self::full(q).split_whitespace().count()
        })
    }

    /// The answer as shown: segment text and the citation number (1-based among the cited
    /// sources), the citation shown once its segment is complete.
    pub fn answer(&self, now: u64) -> Vec<(String, Option<usize>)> {
        let Some((q, _)) = self.ask else {
            return Vec::new();
        };
        let mut left = self.words(now);
        let cited = self.cited();
        let mut out = Vec::new();
        for &(t, s) in QUESTIONS[q].1 {
            if left == 0 {
                break;
            }
            let n = t.split_whitespace().count();
            let shown = left.min(n);
            out.push((
                first_words(t, shown),
                (shown == n).then(|| cited.iter().position(|&c| c == s).map_or(0, |p| p + 1)),
            ));
            left -= shown;
        }
        out
    }

    /// Sources the current answer cites, in order of first citation.
    pub fn cited(&self) -> Vec<usize> {
        let Some((q, _)) = self.ask else {
            return Vec::new();
        };
        let mut v = Vec::new();
        for &(_, s) in QUESTIONS[q].1 {
            if !v.contains(&s) {
                v.push(s);
            }
        }
        v
    }

    /// Presses: `wz-step-<i>`, `wz-connect-<i>`, `wz-retry-<i>`, `wz-model-<i>`, `wz-tone-<i>`,
    /// `wz-guard-<i>`, `wz-next`, `wz-back`, `wz-ask-<i>`, `wz-stop`, `wz-live`, `wz-edit`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "wz-next" => {
                if self.step == 2 {
                    return self.press("wz-live", now);
                }
                if !self.can_next(now) {
                    return Some("Connect and sync at least one source first".into());
                }
                self.step += 1;
                self.reached = self.reached.max(self.step);
                return Some(format!("Step {} of 3: {}", self.step + 1, STEPS[self.step]));
            }
            "wz-back" => {
                self.step = self.step.saturating_sub(1);
                return Some(format!("Step {} of 3: {}", self.step + 1, STEPS[self.step]));
            }
            "wz-stop" => {
                if self.streaming(now) {
                    self.stopped = Some(self.words(now));
                    return Some("Stopped".into());
                }
                return None;
            }
            "wz-live" => {
                if !self.can_next(now) {
                    return Some("Ask a test question first".into());
                }
                self.live = true;
                return Some("Live. Ask Northwind now answers customers. Simulated".into());
            }
            "wz-edit" => {
                self.live = false;
                self.step = 1;
                return Some("Back to draft".into());
            }
            _ => {}
        }
        if let Some(i) = nth(id, "wz-step-") {
            if i > self.reached || self.live {
                return None;
            }
            self.step = i;
            return Some(format!("Step {} of 3: {}", i + 1, STEPS[i]));
        }
        if let Some(i) = nth(id, "wz-connect-").or_else(|| nth(id, "wz-retry-")) {
            SOURCES.get(i)?;
            if matches!(self.src[i], Src::Syncing(_)) && id.starts_with("wz-connect-") {
                self.src[i] = Src::Off;
                return Some(format!("{} disconnected", SOURCES[i].0));
            }
            if self.fail && i == 2 {
                self.fail = false;
                self.src[i] = Src::Failed;
                return Some("Intercom: the token expired. Retry".into());
            }
            self.src[i] = Src::Syncing(now);
            return Some(format!(
                "Connecting {}. Syncing {} documents. Simulated",
                SOURCES[i].0,
                thousands(u64::from(SOURCES[i].2))
            ));
        }
        if let Some(i) = nth(id, "wz-model-") {
            MODELS.get(i)?;
            self.model = i;
            return Some(format!("{} selected", MODELS[i].0));
        }
        if let Some(i) = nth(id, "wz-tone-") {
            TONES.get(i)?;
            self.tone = i;
            return Some(format!("Tone {}", TONES[i]));
        }
        if let Some(i) = nth(id, "wz-guard-") {
            let g = self.guards.get_mut(i)?;
            *g = !*g;
            return Some(format!("{} {}", GUARDS[i].0, if *g { "on" } else { "off" }));
        }
        if let Some(i) = nth(id, "wz-ask-") {
            QUESTIONS.get(i)?;
            if self.ready(now).is_empty() {
                return Some("Connect a source first".into());
            }
            self.ask = Some((i, now));
            self.stopped = None;
            return Some(format!("Asking: {}", QUESTIONS[i].0));
        }
        None
    }

    /// Keys: on the model cards Left/Right/Up/Down/Home/End select a model; Enter asks or stops.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match k {
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End"
                if self.step == 1 =>
            {
                let i = rove(self.model, MODELS.len(), k);
                self.press(&format!("wz-model-{i}"), now)
            }
            "Enter" if self.step == 2 => {
                if self.streaming(now) {
                    self.press("wz-stop", now)
                } else {
                    self.press("wz-ask-0", now)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_configure_test_live() {
        let mut w = Wizard::new(0, true, 0);
        assert!(!w.can_next(0));
        w.press("wz-connect-2", 0);
        assert_eq!(w.src[2], Src::Failed);
        w.press("wz-retry-2", 0);
        w.press("wz-connect-0", 0);
        assert_eq!(w.synced(0, 1_200).0, 642);
        assert!(w.can_next(3_000));
        w.press("wz-next", 3_000);
        w.key("right", 3_000);
        assert_eq!(w.model, 1);
        w.press("wz-guard-2", 0);
        assert!(w.guards[2]);
        w.press("wz-next", 3_000);
        w.press("wz-ask-0", 3_000);
        assert!(w.streaming(3_100));
        assert!(!w.can_next(3_100));
        let a = w.answer(9_000);
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].1, Some(1));
        w.press("wz-next", 9_000);
        assert!(w.live);
        assert_eq!(Wizard::new(3, false, 0).ready(5_000).len(), 3);
    }
}
