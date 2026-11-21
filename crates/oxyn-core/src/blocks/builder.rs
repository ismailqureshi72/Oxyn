//! Agent builder: set up one agent in a card. An inline editable name over its workspace and
//! version, a model picker, instructions with a character count, tools as switch rows with their
//! app marks, and knowledge files with upload progress (one fails and retries). A Configure / Test
//! switch swaps the body for a small chat that answers with the model and tools you turned on;
//! Publish ships the setup, and any change after that reads as unsaved until published again.

use super::{first_words, nth, words_at};

/// Models: brand id, label, note.
pub const MODELS: [(&str, &str, &str); 4] = [
    ("claude", "Claude Sonnet 4.5", "Balanced"),
    ("claude", "Claude Opus 4.1", "Most capable"),
    ("openai", "GPT-5", "Broad knowledge"),
    ("gemini", "Gemini 2.5 Flash", "Fastest"),
];
/// Tools: brand id, name, what it lets the agent do.
pub const TOOLS: [(&str, &str, &str); 5] = [
    ("stripe", "Stripe", "Look up invoices and issue refunds"),
    (
        "hubspot",
        "HubSpot",
        "Read the customer's account and notes",
    ),
    ("gmail", "Gmail", "Draft replies in the shared inbox"),
    ("slack", "Slack", "Ask the billing channel for help"),
    ("notion", "Notion", "Read the billing playbook"),
];
/// Knowledge files: name, size in KB.
pub const FILES: [(&str, u32); 3] = [
    ("Refund policy.pdf", 420),
    ("Plan matrix.xlsx", 96),
    ("Escalation playbook.md", 18),
];
/// Upload of one file, ms.
pub const UPLOAD_MS: u64 = 2_600;
/// Publishing, ms.
pub const PUBLISH_MS: u64 = 1_400;
/// Test prompts.
pub const PROMPTS: [&str; 3] = [
    "A customer was charged twice this month",
    "Can I get a refund on my annual plan?",
    "Move us from Team to Business",
];
/// Instructions at the start.
pub const INSTRUCTIONS: &str = "You are Ledger, the billing assistant for Northwind. Answer billing questions in two or three short sentences, check the customer's invoices before promising anything, and hand off to a person for refunds over $500.";

/// The configuration that can be published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Name.
    pub name: String,
    /// Model index.
    pub model: usize,
    /// Instructions.
    pub instructions: String,
    /// Tools on.
    pub tools: [bool; 5],
}

/// The live state of the builder.
#[derive(Clone, Debug, PartialEq)]
pub struct Builder {
    /// The configuration being edited.
    pub config: Config,
    /// The configuration that is live.
    pub published: Config,
    /// Test view on.
    pub testing: bool,
    /// File uploads: started at, failed (index aligned with [`FILES`]).
    pub uploads: Vec<(u64, bool)>,
    /// Publishing since.
    pub publishing: Option<u64>,
    /// Publishing fails (the error scenario).
    pub fail: bool,
    /// Publish result: true published, false failed.
    pub last: Option<bool>,
    /// Test chat: your message and when the reply started.
    pub chat: Vec<(String, u64)>,
    /// Model menu open.
    pub menu: bool,
    /// Version number.
    pub version: u32,
}

impl Builder {
    /// The sample agent at `now`.
    pub fn new(fail: bool, now: u64) -> Self {
        let config = Config {
            name: "Ledger".into(),
            model: 0,
            instructions: INSTRUCTIONS.into(),
            tools: [true, true, true, false, false],
        };
        Self {
            published: config.clone(),
            config,
            testing: false,
            uploads: vec![
                (now.saturating_sub(UPLOAD_MS), false),
                (now, true),
                (now.saturating_sub(UPLOAD_MS / 2), false),
            ],
            publishing: None,
            fail,
            last: None,
            chat: Vec::new(),
            menu: false,
            version: 7,
        }
    }

    /// Upload of file `i` at `now`: fraction and failed.
    pub fn upload(&self, i: usize, now: u64) -> (f64, bool) {
        let (t, f) = self.uploads[i];
        let p = (now.saturating_sub(t) as f64 / UPLOAD_MS as f64).min(1.0);
        if f && p >= 0.55 {
            (0.55, true)
        } else {
            (p, false)
        }
    }

    /// Unsaved changes against the live config.
    pub fn unsaved(&self) -> bool {
        self.config != self.published
    }

    /// Publish state word at `now`: `Published`, `Unsaved changes`, `Publishing`, `Publish failed`.
    pub fn publish_state(&self, now: u64) -> &'static str {
        match self.publishing {
            Some(t) if now < t + PUBLISH_MS => "Publishing",
            Some(_) if self.fail => "Publish failed",
            Some(_) => "Published",
            _ if self.last == Some(false) => "Publish failed",
            _ if self.unsaved() => "Unsaved changes",
            _ => "Published",
        }
    }

    /// Settles a finished publish (call before reading after time passes).
    pub fn settle(&mut self, now: u64) {
        if let Some(t) = self.publishing
            && now >= t + PUBLISH_MS
        {
            self.publishing = None;
            if self.fail {
                self.last = Some(false);
                self.fail = false;
            } else {
                self.published = self.config.clone();
                self.last = Some(true);
                self.version += 1;
            }
        }
    }

    /// The reply to your message `i`: the text, how much has streamed at `now`.
    pub fn reply(&self, i: usize, now: u64) -> (String, String) {
        let (q, t) = &self.chat[i];
        let tools: Vec<&str> = (0..TOOLS.len())
            .filter(|&k| self.config.tools[k])
            .map(|k| TOOLS[k].1)
            .collect();
        let used = if tools.is_empty() {
            "without any tools".to_string()
        } else {
            format!("using {}", tools.join(", "))
        };
        let full = if q.contains("twice") {
            format!(
                "I found two charges of $49 on the same day; the second one is a duplicate. I've drafted a refund for it and a reply to the customer ({used})."
            )
        } else if q.contains("annual") {
            format!(
                "Annual plans are refundable in full within 30 days, then for the unused months. This account is on day 12, so a full refund applies ({used})."
            )
        } else {
            format!(
                "I can do that. Business is $24 per seat; the change applies from the next invoice and I'll note it on the account ({used})."
            )
        };
        let shown = first_words(&full, words_at(&full, now.saturating_sub(*t), 2));
        (full, shown)
    }

    /// A reply is still streaming.
    pub fn streaming(&self, now: u64) -> bool {
        self.chat.last().is_some_and(|_| {
            let (f, s) = self.reply(self.chat.len() - 1, now);
            f != s
        })
    }

    /// Sets the name (Escape restores by passing the old one).
    pub fn rename(&mut self, name: &str) {
        let n = name.trim();
        if !n.is_empty() {
            self.config.name = n.chars().take(40).collect();
        }
    }

    /// Sets the instructions.
    pub fn instruct(&mut self, text: &str) {
        self.config.instructions = text.chars().take(1_200).collect();
    }

    /// Sends a test message at `now`.
    pub fn send(&mut self, text: &str, now: u64) -> Option<String> {
        let t = text.trim();
        if t.is_empty() || self.streaming(now) {
            return None;
        }
        self.chat.push((t.to_string(), now + 500));
        Some(format!(
            "{} is replying with {}",
            self.config.name, MODELS[self.config.model].1
        ))
    }

    /// Presses: `ab-view-<0|1>`, `ab-menu`, `ab-model-<i>`, `ab-tool-<i>`, `ab-retry-<i>`,
    /// `ab-remove-<i>`, `ab-publish`, `ab-prompt-<i>`, `ab-stop`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        self.settle(now);
        if let Some(v) = nth(id, "ab-view-") {
            self.testing = v == 1;
            return Some(if self.testing { "Test" } else { "Configure" }.into());
        }
        if let Some(i) = nth(id, "ab-model-") {
            self.config.model = i.min(MODELS.len() - 1);
            self.menu = false;
            return Some(format!("Model {}", MODELS[self.config.model].1));
        }
        if let Some(i) = nth(id, "ab-tool-") {
            let on = self.config.tools.get_mut(i)?;
            *on = !*on;
            return Some(format!("{} {}", TOOLS[i].1, if *on { "on" } else { "off" }));
        }
        if let Some(i) = nth(id, "ab-retry-") {
            if !self.upload(i, now).1 {
                return None;
            }
            self.uploads[i] = (now, false);
            return Some(format!("Uploading {} again", FILES[i].0));
        }
        if let Some(i) = nth(id, "ab-prompt-") {
            return self.send(PROMPTS.get(i)?, now);
        }
        match id {
            "ab-menu" => {
                self.menu = !self.menu;
                None
            }
            "ab-publish"
                if self.publishing.is_none() && (self.unsaved() || self.last == Some(false)) =>
            {
                self.publishing = Some(now);
                self.last = None;
                Some("Publishing. Simulated, nothing was deployed".into())
            }
            "ab-stop" if self.streaming(now) => {
                self.chat.pop();
                Some("Stopped".into())
            }
            _ => None,
        }
    }

    /// Keys: Escape closes the menu; Up and Down move the model while it is open.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        match super::norm_key(key) {
            "Escape" if self.menu => {
                self.menu = false;
                None
            }
            k @ ("ArrowUp" | "ArrowDown") if self.menu => {
                let i = super::rove(self.config.model, MODELS.len(), k);
                self.config.model = i;
                Some(MODELS[i].1.to_string())
            }
            "ArrowLeft" if self.testing => self.press("ab-view-0", now),
            "ArrowRight" if !self.testing => self.press("ab-view-1", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_publish_test() {
        let mut b = Builder::new(true, 10_000);
        assert_eq!(b.publish_state(10_000), "Published");
        assert_eq!(b.upload(0, 10_000), (1.0, false));
        assert!(b.upload(1, 12_000).1);
        b.press("ab-retry-1", 12_000);
        assert_eq!(b.upload(1, 15_000), (1.0, false));
        b.press("ab-tool-3", 10_000);
        assert_eq!(b.publish_state(10_000), "Unsaved changes");
        b.press("ab-tool-3", 10_000);
        assert!(!b.unsaved());
        b.rename("Ledger 2");
        b.press("ab-publish", 10_000);
        assert_eq!(b.publish_state(10_500), "Publishing");
        b.settle(12_000);
        assert_eq!(b.publish_state(12_000), "Publish failed");
        b.press("ab-publish", 12_000);
        b.settle(14_000);
        assert_eq!(b.publish_state(14_000), "Published");
        assert_eq!(b.version, 8);
        b.press("ab-view-1", 14_000);
        b.press("ab-prompt-0", 14_000);
        assert!(b.streaming(14_600));
        assert!(!b.streaming(30_000));
        assert!(b.reply(0, 30_000).0.contains("Stripe"));
        b.press("ab-menu", 30_000);
        b.key("ArrowDown", 30_000);
        assert_eq!(b.config.model, 1);
    }
}
