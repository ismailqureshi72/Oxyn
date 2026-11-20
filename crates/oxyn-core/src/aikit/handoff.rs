//! Human handoff (Arc "Agent handoff"): a support chat where an AI assistant hands over to a
//! person. Transferring shows the summary it passes to the team and the typical wait; the person
//! joins and takes over; a thin timeline marks each stage. When nobody is available the card
//! says so and offers to keep chatting or leave an email.

use super::{nth, words_at};

/// Transfer takes this long before a person joins, ms.
pub const TRANSFER_MS: u64 = 2_600;
/// The person types for this long before their first message, ms.
pub const TYPING_MS: u64 = 1_400;
/// The assistant thinks for this long before a reply streams, ms.
pub const BOT_MS: u64 = 600;

/// The assistant (name, mark) and the person who joins (name, team).
pub const ASSISTANT: (&str, &str) = ("Corvel Assistant", "claude");
/// The person who joins (fictional).
pub const PERSON: (&str, &str) = ("Mara Ilves", "Billing support");

/// The summary handed to the team.
pub const SUMMARY: &str = "Customer was charged twice for the October invoice (INV-2291, $49). Card ending 4242. The assistant confirmed both charges and could not refund, so a person is needed.";

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    /// The customer.
    User,
    /// The assistant.
    Bot,
    /// The person.
    Agent,
    /// A system line.
    System,
}

/// One message.
#[derive(Clone, Debug, PartialEq)]
pub struct Msg {
    /// Author.
    pub who: Who,
    /// Text.
    pub text: String,
    /// When it lands (streamed from then for the bot), ms.
    pub at: u64,
}

/// The stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The assistant answers.
    Assistant,
    /// Handing over.
    Transferring,
    /// The person took over.
    Connected,
    /// Marked resolved.
    Resolved,
    /// Nobody available.
    Unavailable,
}

impl Stage {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Assistant => "Assistant",
            Self::Transferring => "Transferring",
            Self::Connected => "With a person",
            Self::Resolved => "Resolved",
            Self::Unavailable => "Unavailable",
        }
    }
}

/// The live state of the handoff chat.
#[derive(Clone, Debug, PartialEq)]
pub struct Handoff {
    /// Messages.
    pub msgs: Vec<Msg>,
    /// Transfer requested at.
    pub transfer: Option<u64>,
    /// Resolved at.
    pub resolved: Option<u64>,
    /// No person is online (the unavailable scenario).
    pub nobody: bool,
    /// Draft in the composer.
    pub draft: String,
    /// The summary can be edited before transfer (Oxyn extra).
    pub edit_summary: bool,
    /// The edited summary.
    pub summary: String,
    /// Place in the queue while transferring (Oxyn extra).
    pub queue: u32,
}

impl Handoff {
    /// `scenario`: `handoff` (starts chatting), `connected`, `unavailable`, `resolved`.
    pub fn new(scenario: &str, now: u64) -> Self {
        let base = now.saturating_sub(60_000);
        let mut h = Self {
            msgs: vec![
                Msg { who: Who::User, text: "I was charged twice for my October invoice.".into(), at: base },
                Msg { who: Who::Bot, text: "I can see two charges of $49 on November 1 for invoice INV-2291. Refunds need someone from billing, would you like me to bring in a person?".into(), at: base + 1_000 },
            ],
            transfer: None,
            resolved: None,
            nobody: scenario == "unavailable",
            draft: String::new(),
            edit_summary: false,
            summary: SUMMARY.to_string(),
            queue: 2,
        };
        match scenario {
            "connected" => {
                h.transfer = Some(base + 5_000);
                h.add_agent_lines(base + 5_000);
            }
            "resolved" => {
                h.transfer = Some(base + 5_000);
                h.add_agent_lines(base + 5_000);
                h.resolved = Some(base + 30_000);
            }
            "unavailable" => {
                h.transfer = Some(base + 5_000);
                h.msgs.push(Msg {
                    who: Who::System,
                    text: "Nobody from billing is online right now".into(),
                    at: base + 5_000 + TRANSFER_MS,
                });
            }
            _ => {}
        }
        h
    }

    fn add_agent_lines(&mut self, t: u64) {
        self.msgs.push(Msg {
            who: Who::System,
            text: format!("{} from {} joined", PERSON.0, PERSON.1),
            at: t + TRANSFER_MS,
        });
        self.msgs.push(Msg { who: Who::Agent, text: "Hi, I'm Mara. I've read the summary, I can see the duplicate charge and I'm refunding it now. It should be back on your card in 3 to 5 days.".into(), at: t + TRANSFER_MS + TYPING_MS });
    }

    /// Stage at `now`.
    pub fn stage(&self, now: u64) -> Stage {
        if self.resolved.is_some_and(|r| now >= r) {
            return Stage::Resolved;
        }
        match self.transfer {
            None => Stage::Assistant,
            Some(t) if now < t + TRANSFER_MS => Stage::Transferring,
            Some(_) if self.nobody => Stage::Unavailable,
            Some(_) => Stage::Connected,
        }
    }

    /// Messages that have landed at `now`.
    pub fn shown(&self, now: u64) -> Vec<&Msg> {
        self.msgs.iter().filter(|m| m.at <= now).collect()
    }

    /// Visible words of message `i` (the bot streams).
    pub fn words(&self, i: usize, now: u64) -> usize {
        let m = &self.msgs[i];
        if m.who == Who::Bot {
            words_at(&m.text, now.saturating_sub(m.at), 4)
        } else {
            m.text.split_whitespace().count()
        }
    }

    /// Who is typing (`assistant` thinking before a reply or the person before theirs).
    pub fn typing(&self, now: u64) -> Option<Who> {
        let next = self.msgs.iter().find(|m| m.at > now)?;
        match next.who {
            Who::Bot if next.at - now <= BOT_MS => Some(Who::Bot),
            Who::Agent if next.at - now <= TYPING_MS => Some(Who::Agent),
            _ => None,
        }
    }

    /// Timeline marks: stage label and when it began (relative text).
    pub fn marks(&self, now: u64) -> Vec<(Stage, bool)> {
        let st = self.stage(now);
        let order = [
            Stage::Assistant,
            Stage::Transferring,
            Stage::Connected,
            Stage::Resolved,
        ];
        let cur = match st {
            Stage::Unavailable => 1,
            s => order.iter().position(|&x| x == s).unwrap_or(0),
        };
        order
            .iter()
            .enumerate()
            .map(|(i, &s)| (s, i <= cur))
            .collect()
    }

    /// The wait line while transferring (`Typical wait about 2 min · 2nd in line`).
    pub fn wait(&self) -> String {
        let pos = ["", "1st", "2nd", "3rd"]
            .get(self.queue as usize)
            .copied()
            .unwrap_or("next");
        format!("Typical reply in about 2 min · {pos} in line")
    }

    /// Presses: `ho-human`, `ho-cancel`, `ho-send`, `ho-draft:<text>`, `ho-resolve`,
    /// `ho-retry` (from unavailable), `ho-keep` (keep chatting with the assistant),
    /// `ho-chip-<i>` (suggested reply), `ho-sum:<text>` (edit the summary), `ho-editsum`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(t) = id.strip_prefix("ho-draft:") {
            self.draft = t.to_string();
            return None;
        }
        if let Some(t) = id.strip_prefix("ho-sum:") {
            self.summary = t.to_string();
            return None;
        }
        if let Some(i) = nth(id, "ho-chip-") {
            let t = ["Yes, please", "Can you refund it?", "Thanks!"].get(i)?;
            self.draft = (*t).to_string();
            return self.press("ho-send", now);
        }
        match id {
            "ho-human" => {
                if self.stage(now) != Stage::Assistant {
                    return None;
                }
                self.transfer = Some(now);
                self.queue = 2;
                if self.nobody {
                    self.msgs.push(Msg {
                        who: Who::System,
                        text: "Nobody from billing is online right now".into(),
                        at: now + TRANSFER_MS,
                    });
                } else {
                    self.add_agent_lines(now);
                }
                Some(format!("Transferring to {}", PERSON.1))
            }
            "ho-cancel" => {
                if self.stage(now) != Stage::Transferring {
                    return None;
                }
                self.transfer = None;
                self.msgs.retain(|m| m.at <= now);
                Some("Transfer cancelled, the assistant is back".into())
            }
            "ho-send" => {
                let text = self.draft.trim().to_string();
                if text.is_empty() {
                    return None;
                }
                self.draft.clear();
                self.msgs.push(Msg {
                    who: Who::User,
                    text: text.clone(),
                    at: now,
                });
                match self.stage(now) {
                    Stage::Assistant | Stage::Unavailable => {
                        let reply = if text.to_lowercase().contains("yes")
                            || text.to_lowercase().contains("person")
                        {
                            "Of course. Press Talk to a person and I'll pass on everything so you don't have to repeat it."
                        } else if text.to_lowercase().contains("refund") {
                            "I can't issue refunds myself, but a person from billing can do it right away."
                        } else {
                            "Happy to help with anything else while you wait."
                        };
                        self.msgs.push(Msg {
                            who: Who::Bot,
                            text: reply.into(),
                            at: now + BOT_MS,
                        });
                    }
                    Stage::Connected => {
                        self.msgs.push(Msg {
                            who: Who::Agent,
                            text: "You're welcome! Anything else I can do?".into(),
                            at: now + TYPING_MS,
                        });
                    }
                    _ => {}
                }
                Some("Message sent".into())
            }
            "ho-resolve" => {
                if self.stage(now) != Stage::Connected {
                    return None;
                }
                self.resolved = Some(now);
                self.msgs.push(Msg {
                    who: Who::System,
                    text: "Marked as resolved".into(),
                    at: now,
                });
                Some("Conversation resolved".into())
            }
            "ho-retry" => {
                if self.stage(now) != Stage::Unavailable {
                    return None;
                }
                self.nobody = false;
                self.transfer = Some(now);
                self.add_agent_lines(now);
                Some(format!("Trying {} again", PERSON.1))
            }
            "ho-keep" => {
                if self.stage(now) != Stage::Unavailable {
                    return None;
                }
                self.transfer = None;
                self.msgs.push(Msg { who: Who::Bot, text: "I'm still here. I've emailed billing your summary, and they'll reply within a day.".into(), at: now + BOT_MS });
                Some("Back with the assistant".into())
            }
            "ho-editsum" => {
                self.edit_summary = !self.edit_summary;
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_transfers_connects_and_resolves() {
        let mut h = Handoff::new("handoff", 100_000);
        assert_eq!(h.stage(100_000), Stage::Assistant);
        h.press("ho-human", 100_000);
        assert_eq!(h.stage(101_000), Stage::Transferring);
        assert_eq!(h.stage(100_000 + TRANSFER_MS), Stage::Connected);
        assert_eq!(h.typing(100_000 + TRANSFER_MS + 100), Some(Who::Agent));
        h.press("ho-resolve", 110_000);
        assert_eq!(h.stage(110_000), Stage::Resolved);
        assert!(h.marks(110_000).iter().all(|m| m.1));
        let mut c = Handoff::new("handoff", 100_000);
        c.press("ho-human", 100_000);
        c.press("ho-cancel", 101_000);
        assert_eq!(c.stage(200_000), Stage::Assistant);
        c.press("ho-chip-1", 200_000);
        assert_eq!(c.typing(200_100), Some(Who::Bot));
        let mut u = Handoff::new("unavailable", 100_000);
        assert_eq!(u.stage(100_000), Stage::Unavailable);
        u.press("ho-retry", 100_000);
        assert_eq!(u.stage(100_000 + TRANSFER_MS), Stage::Connected);
        let mut k = Handoff::new("unavailable", 100_000);
        k.press("ho-keep", 100_000);
        assert_eq!(k.stage(100_000), Stage::Assistant);
        assert!(k.wait().contains("2nd"));
    }
}
