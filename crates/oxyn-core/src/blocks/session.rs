//! Live session: you and a team of agents on one screen while they work. Their conversation runs
//! in the middle (it streams in on a script derived from the session clock), the team's plan ticks
//! off beside it, and the shared context fills underneath. Picking an agent narrows the stream to
//! it and points at its steps; an agent's question waits for your answer, which unblocks the plan.
//! Pausing freezes the session clock.

use super::{dur, dur_words, nth, rove, thousands};

/// The people in the session: name, brand id of the model, model label. Index 0 is you.
pub const AGENTS: [(&str, &str, &str); 4] = [
    ("You", "", ""),
    ("Researcher", "claude", "Claude Sonnet 4.5"),
    ("Writer", "openai", "GPT-5"),
    ("Designer", "gemini", "Gemini 2.5 Pro"),
];

/// The task the team works on.
pub const TASK: &str = "Launch the new pricing page";

/// The scripted conversation: session ms it appears, from, to (`None` for everyone), text.
pub const SCRIPT: [(u64, usize, Option<usize>, &str); 7] = [
    (
        0,
        0,
        None,
        "Let's get the new pricing page live by Friday. Researcher, start with what similar tools charge per seat.",
    ),
    (
        1_400,
        1,
        None,
        "On it. Collecting per-seat prices from five comparable tools.",
    ),
    (
        4_600,
        1,
        Some(2),
        "The median is $14 per seat, and two of them charge extra for single sign-on. Notes are in the brief.",
    ),
    (
        7_000,
        2,
        None,
        "Drafting the copy for Starter, Team and Business now.",
    ),
    (
        9_800,
        3,
        None,
        "The plan table is laid out. I'll drop the copy in as soon as it lands.",
    ),
    (12_600, 2, Some(0), "Should Team be $12 or $15 per seat?"),
    (
        13_400,
        3,
        Some(2),
        "Either fits the layout. $15 keeps the column widths even.",
    ),
];
/// Index of the question in [`SCRIPT`] that waits for you.
pub const ASK: usize = 5;
/// Quick answers to the question.
pub const ANSWERS: [&str; 2] = ["$12 per seat", "$15 per seat"];

/// The plan: title, agent, start and end in session ms. A `u64::MAX` end means "after your
/// answer" (it is placed relative to the answer).
pub const STEPS: [(&str, usize, u64, u64); 5] = [
    ("Compare per-seat prices", 1, 1_400, 6_400),
    ("Lay out the plan table", 3, 7_800, 11_000),
    ("Draft the copy for three plans", 2, 7_000, u64::MAX),
    ("Pick the Team price", 0, 12_600, u64::MAX),
    ("Publish the page", 2, u64::MAX, u64::MAX),
];

/// Context parts: label and tokens at the start (the conversation grows with each message).
pub const CONTEXT: [(&str, u64); 4] = [
    ("System and tools", 9_400),
    ("Brief and notes", 18_200),
    ("Competitor pages", 31_000),
    ("Conversation", 2_600),
];
/// Context window limit, tokens.
pub const LIMIT: u64 = 200_000;
/// Model the context belongs to (brand, label).
pub const CONTEXT_MODEL: (&str, &str) = ("claude", "Claude Sonnet 4.5");

/// A step's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepSt {
    /// Not started.
    Todo,
    /// The current step.
    Active,
    /// Waiting on you.
    Yours,
    /// Done.
    Done,
}

/// One message as shown: from, to, text, when (session ms), and whether it is the open question.
#[derive(Clone, Debug, PartialEq)]
pub struct Msg {
    /// Sender index.
    pub from: usize,
    /// Recipient (`None` for everyone).
    pub to: Option<usize>,
    /// Text.
    pub text: String,
    /// Session ms it arrived.
    pub at: u64,
    /// It is the question still waiting for you.
    pub asks: bool,
}

/// The live state of the session.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    /// Clock start, ms.
    pub t0: u64,
    /// Paused at (host ms), if paused.
    pub paused: Option<u64>,
    /// Total paused time so far, ms.
    pub paused_ms: u64,
    /// The spotlighted agent, if any.
    pub focus: Option<usize>,
    /// The message whose details are open.
    pub picked: Option<usize>,
    /// Your answer to the question: which, at session ms.
    pub answer: Option<(usize, u64)>,
    /// Messages you sent: session ms, to, text.
    pub sent: Vec<(u64, Option<usize>, String)>,
    /// Starting point in session ms (to open mid-run).
    pub skip: u64,
}

impl Session {
    /// A session at `now`, starting `skip` ms into the script.
    pub fn new(skip: u64, now: u64) -> Self {
        Self {
            t0: now,
            paused: None,
            paused_ms: 0,
            focus: None,
            picked: None,
            answer: None,
            sent: Vec::new(),
            skip,
        }
    }

    /// Session time at `now`, ms (frozen while paused).
    pub fn t(&self, now: u64) -> u64 {
        let end = self.paused.unwrap_or(now);
        end.saturating_sub(self.t0).saturating_sub(self.paused_ms) + self.skip
    }

    /// When the plan resumes after your answer (session ms), if answered.
    fn resumed(&self) -> Option<u64> {
        self.answer.map(|(_, t)| t)
    }

    /// Start and end of step `i` in session ms (`None` while it depends on your answer).
    pub fn span(&self, i: usize) -> (Option<u64>, Option<u64>) {
        let (_, _, s, e) = STEPS[i];
        let r = self.resumed();
        match i {
            2 => (Some(s), r.map(|r| r + 2_400)),
            3 => (Some(s), r),
            4 => (r.map(|r| r + 2_400), r.map(|r| r + 7_400)),
            _ => (Some(s), Some(e)),
        }
    }

    /// State of step `i` at `now`.
    pub fn step(&self, i: usize, now: u64) -> StepSt {
        let t = self.t(now);
        match self.span(i) {
            (Some(s), _) if t < s => StepSt::Todo,
            (None, _) => StepSt::Todo,
            (Some(_), Some(e)) if t >= e => StepSt::Done,
            _ if STEPS[i].1 == 0 => StepSt::Yours,
            _ => StepSt::Active,
        }
    }

    /// Time a step took or has taken so far.
    pub fn step_time(&self, i: usize, now: u64) -> Option<String> {
        let t = self.t(now);
        match self.span(i) {
            (Some(s), Some(e)) if t >= s => Some(dur(t.min(e) - s)),
            (Some(s), None) if t >= s => Some(dur(t - s)),
            _ => None,
        }
    }

    /// Steps done.
    pub fn done(&self, now: u64) -> usize {
        (0..STEPS.len())
            .filter(|&i| self.step(i, now) == StepSt::Done)
            .count()
    }

    /// The plan's state: running, paused, waiting on you or done.
    pub fn status(&self, now: u64) -> &'static str {
        if self.done(now) == STEPS.len() {
            "Done"
        } else if self.paused.is_some() {
            "Paused"
        } else if self.step(3, now) == StepSt::Yours {
            "Waiting on you"
        } else {
            "Running"
        }
    }

    /// The conversation at `now`: script, your messages and replies, in order.
    pub fn messages(&self, now: u64) -> Vec<Msg> {
        let t = self.t(now);
        let mut v: Vec<Msg> = SCRIPT
            .iter()
            .enumerate()
            .filter(|(_, m)| m.0 <= t)
            .map(|(i, &(at, from, to, text))| Msg {
                from,
                to,
                text: text.into(),
                at,
                asks: i == ASK && self.answer.is_none(),
            })
            .collect();
        if let Some((a, at)) = self.answer {
            v.push(Msg {
                from: 0,
                to: Some(2),
                text: ANSWERS[a].into(),
                at,
                asks: false,
            });
            if t >= at + 1_200 {
                v.push(Msg {
                    from: 2,
                    to: None,
                    text: format!(
                        "Going with {}. Finishing the copy and publishing next.",
                        ANSWERS[a]
                    ),
                    at: at + 1_200,
                    asks: false,
                });
            }
            if t >= at + 7_400 {
                v.push(Msg {
                    from: 2,
                    to: None,
                    text: "The pricing page is live.".into(),
                    at: at + 7_400,
                    asks: false,
                });
            }
        }
        for (at, to, text) in &self.sent {
            v.push(Msg {
                from: 0,
                to: *to,
                text: text.clone(),
                at: *at,
                asks: false,
            });
            if t >= at + 1_500 {
                let who = to.unwrap_or(1);
                v.push(Msg {
                    from: who,
                    to: Some(0),
                    text: "Noted. I'll fold that into my next step.".into(),
                    at: at + 1_500,
                    asks: false,
                });
            }
        }
        v.sort_by_key(|m| m.at);
        v
    }

    /// Whether an agent is typing at `now` (the next script line is under 1.2 s away).
    pub fn typing(&self, now: u64) -> Option<usize> {
        let t = self.t(now);
        SCRIPT
            .iter()
            .find(|m| m.0 > t && m.0 - t < 1_200)
            .map(|m| m.1)
            .filter(|&f| f != 0)
    }

    /// Context parts at `now`: label and tokens (the conversation grows by 900 a message).
    pub fn context(&self, now: u64) -> Vec<(&'static str, u64)> {
        let n = self.messages(now).len() as u64;
        CONTEXT
            .iter()
            .map(|&(l, tk)| {
                (
                    l,
                    if l == "Conversation" {
                        tk + n * 900
                    } else {
                        tk
                    },
                )
            })
            .collect()
    }

    /// Context used at `now`, tokens.
    pub fn used(&self, now: u64) -> u64 {
        self.context(now).iter().map(|p| p.1).sum()
    }

    /// The context line (`64,000 of 200,000 tokens · 32%`).
    pub fn context_line(&self, now: u64) -> String {
        let u = self.used(now);
        format!(
            "{} of {} tokens · {}%",
            thousands(u),
            thousands(LIMIT),
            u * 100 / LIMIT
        )
    }

    /// Agents named with `@Name` in `draft`.
    pub fn mentions(draft: &str) -> Vec<usize> {
        let d = draft.to_lowercase();
        (1..AGENTS.len())
            .filter(|&i| d.contains(&format!("@{}", AGENTS[i].0.to_lowercase())))
            .collect()
    }

    /// Agents matching an unfinished `@` at the end of `draft` (the mention menu).
    pub fn mention_menu(draft: &str) -> Option<Vec<usize>> {
        let at = draft.rfind('@')?;
        let q = draft[at + 1..].to_lowercase();
        if q.contains(' ') {
            return None;
        }
        Some(
            (1..AGENTS.len())
                .filter(|&i| AGENTS[i].0.to_lowercase().starts_with(&q))
                .collect(),
        )
    }

    /// Sends `text` from you at `now` (to the agents it names, else everyone).
    pub fn send(&mut self, text: &str, now: u64) -> Option<String> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let to = Self::mentions(text).first().copied();
        self.sent.push((self.t(now), to, text.to_string()));
        Some(match to {
            Some(i) => format!("Sent to {}", AGENTS[i].0),
            None => "Sent to everyone".into(),
        })
    }

    /// Presses: `ls-agent-<i>` (0 lets go), `ls-msg-<i>`, `ls-answer-<i>`, `ls-pause`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ls-agent-") {
            self.focus = if i == 0 || self.focus == Some(i) {
                None
            } else {
                Some(i.min(AGENTS.len() - 1))
            };
            return Some(match self.focus {
                Some(i) => {
                    let n = (0..STEPS.len()).filter(|&s| STEPS[s].1 == i).count();
                    format!("Showing {}: {n} steps", AGENTS[i].0)
                }
                None => "Showing everyone".into(),
            });
        }
        if let Some(i) = nth(id, "ls-msg-") {
            self.picked = if self.picked == Some(i) {
                None
            } else {
                Some(i)
            };
            return None;
        }
        if let Some(i) = nth(id, "ls-answer-") {
            if self.answer.is_some() || self.t(now) < SCRIPT[ASK].0 || i >= ANSWERS.len() {
                return None;
            }
            self.answer = Some((i, self.t(now)));
            return Some(format!("Answered {}. The plan moves on", ANSWERS[i]));
        }
        if id == "ls-pause" {
            if let Some(p) = self.paused.take() {
                self.paused_ms += now.saturating_sub(p);
                return Some("Resumed".into());
            }
            if self.status(now) == "Done" {
                return None;
            }
            self.paused = Some(now);
            return Some(format!("Paused at {}", dur(self.t(now))));
        }
        None
    }

    /// Keys: Left/Right/Home/End on the agent tabs (`tabs`), Up/Down/Enter on messages
    /// (`msgs`), Escape anywhere.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        if k == "Escape" {
            let had = self.picked.take().is_some() | self.focus.take().is_some();
            return had.then(|| "Showing every step".to_string());
        }
        match part {
            "tabs" => {
                let cur = self.focus.unwrap_or(0);
                let i = rove(cur, AGENTS.len(), k);
                (i != cur)
                    .then(|| self.press(&format!("ls-agent-{i}"), now))
                    .flatten()
            }
            "msgs" => {
                let n = self.messages(now).len();
                let cur = self.picked.unwrap_or(n);
                match k {
                    "ArrowUp" | "ArrowDown" => {
                        self.picked = Some(rove(cur.min(n.saturating_sub(1)), n, k));
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Accessible label of message `m` (who, to whom, when).
    pub fn label(m: &Msg) -> String {
        let to =
            m.to.map_or("everyone".to_string(), |i| AGENTS[i].0.to_string());
        format!("{} to {}, at {}", AGENTS[m.from].0, to, dur_words(m.at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_plan_and_answer() {
        let mut s = Session::new(0, 0);
        assert_eq!(s.messages(0).len(), 1);
        assert_eq!(s.step(0, 2_000), StepSt::Active);
        assert_eq!(s.step(0, 7_000), StepSt::Done);
        assert_eq!(s.status(13_000), "Waiting on you");
        assert!(s.messages(13_000).iter().any(|m| m.asks));
        assert!(s.press("ls-answer-1", 1_000).is_none());
        s.press("ls-answer-1", 13_000);
        assert_eq!(s.step(3, 13_000), StepSt::Done);
        assert_eq!(s.step(4, 16_000), StepSt::Active);
        assert_eq!(s.status(21_000), "Done");
        assert_eq!(s.done(21_000), 5);
        assert!(s.used(21_000) > s.used(0));
    }

    #[test]
    fn pause_focus_mentions_and_send() {
        let mut s = Session::new(0, 0);
        s.press("ls-pause", 2_000);
        assert_eq!(s.t(5_000), 2_000);
        s.press("ls-pause", 5_000);
        assert_eq!(s.t(6_000), 3_000);
        s.key("tabs", "ArrowRight", 0);
        assert_eq!(s.focus, Some(1));
        s.key("tabs", "Escape", 0);
        assert_eq!(s.focus, None);
        assert_eq!(Session::mentions("hey @writer and @Designer"), vec![2, 3]);
        assert_eq!(Session::mention_menu("ping @wr"), Some(vec![2]));
        assert_eq!(
            s.send("@Designer bigger type", 6_000).unwrap(),
            "Sent to Designer"
        );
        assert!(
            s.messages(9_000)
                .iter()
                .any(|m| m.from == 3 && m.to == Some(0))
        );
    }
}
