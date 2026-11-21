//! Blind compare: one prompt to two models, both answers stream side by side at their own pace,
//! a vote picks the better one and only then are the models revealed. A leaderboard keeps win
//! rates across rounds; a Blind switch can reveal names early; a pane that fails offers Retry.

use super::{first_words, nth, rove, usd, words_at};

/// Sample prompts: chip label and prompt text.
pub const SAMPLES: [(&str, &str); 3] = [
    (
        "Explain retries",
        "Explain exponential backoff to a new engineer in three short paragraphs.",
    ),
    (
        "Name a feature",
        "Suggest five names for a feature that summarises long support threads.",
    ),
    (
        "Write SQL",
        "Write a SQL query for the ten customers with the most orders last month.",
    ),
];

/// Answers per sample: side A and side B.
pub const ANSWERS: [[&str; 2]; 3] = [
    [
        "When a request fails, retrying at once often fails again because the service is still busy. Backoff waits a little before the first retry, then twice as long before the next, and so on. Adding a small random jitter stops many clients from retrying in lockstep. Cap the wait and the number of attempts so a broken service does not hold work forever.",
        "Exponential backoff means waiting longer after each failed attempt: one second, then two, then four. It gives an overloaded service room to recover instead of piling on. Add jitter so clients spread out, set a maximum delay, and give up after a few tries with a clear error.",
    ],
    [
        "Here are five: Thread Digest, Catch Up, Gist, Recap and Thread Notes. Recap is short and plain; Thread Digest says exactly what it does; Gist feels friendly in a chat product.",
        "Try Summary Lens, Quick Recap, Thread Brief, Skim and TL;DR. Thread Brief reads well in a menu and works as a verb in docs. Skim is the shortest but may be too casual for enterprise teams.",
    ],
    [
        "SELECT customer_id, COUNT(*) AS orders FROM orders WHERE created_at >= date_trunc('month', now()) - interval '1 month' AND created_at < date_trunc('month', now()) GROUP BY customer_id ORDER BY orders DESC LIMIT 10; This counts each order once per customer for the previous calendar month.",
        "SELECT c.name, COUNT(o.id) AS order_count FROM customers c JOIN orders o ON o.customer_id = c.id WHERE o.created_at >= now() - interval '30 days' GROUP BY c.name ORDER BY order_count DESC LIMIT 10; Swap the date filter for a calendar month if you report monthly.",
    ],
];

/// Models: name, brand mark, first token ms, words per tick, cost per answer (USD).
pub const MODELS: [(&str, &str, u64, usize, f64); 6] = [
    ("Claude Sonnet 4.5", "claude", 620, 2, 0.0041),
    ("GPT-5 mini", "openai", 480, 3, 0.0018),
    ("Gemini 2.5 Pro", "gemini", 900, 2, 0.0032),
    ("Mistral Large", "mistral", 540, 2, 0.0024),
    ("Llama 4 Maverick", "meta", 380, 3, 0.0011),
    ("DeepSeek V3", "deepseek", 1_100, 2, 0.0009),
];

/// The pairs played, round after round (indices into [`MODELS`]).
pub const PAIRS: [(usize, usize); 3] = [(0, 1), (2, 4), (3, 5)];

/// Starting leaderboard: wins (in half points, a tie is one) and games per model.
pub const BOARD: [(u32, u32); 6] = [(58, 44), (49, 41), (40, 37), (31, 33), (36, 39), (22, 30)];

/// A vote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vote {
    /// A is better.
    A,
    /// A tie.
    Tie,
    /// B is better.
    B,
    /// Both bad.
    Bad,
}

impl Vote {
    /// Words for the result line.
    pub fn words(self) -> &'static str {
        match self {
            Vote::A => "You picked A",
            Vote::B => "You picked B",
            Vote::Tie => "You called it a tie",
            Vote::Bad => "You marked both as bad",
        }
    }
}

/// A pane's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// No run yet.
    Idle,
    /// Waiting for the first token.
    Thinking,
    /// Streaming.
    Writing,
    /// Finished.
    Done,
    /// Stopped by you.
    Stopped,
    /// Failed (Retry).
    Failed,
}

impl Side {
    /// The status word.
    pub fn word(self) -> &'static str {
        match self {
            Side::Idle => "Ready",
            Side::Thinking => "Thinking",
            Side::Writing => "Writing",
            Side::Done => "Done",
            Side::Stopped => "Stopped",
            Side::Failed => "Failed",
        }
    }
}

/// The arena.
#[derive(Clone, Debug, PartialEq)]
pub struct Arena {
    /// Sample shown in the prompt field.
    pub sample: usize,
    /// Round number (picks the pair).
    pub round: usize,
    /// Run start per side (B restarts on Retry).
    pub start: Option<[u64; 2]>,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// Side B fails on its first run (error scenario).
    pub fail: bool,
    /// Side B failed this round.
    pub failed: bool,
    /// The vote, once cast.
    pub vote: Option<Vote>,
    /// Blind (names hidden until the vote).
    pub blind: bool,
    /// Leaderboard open.
    pub board_open: bool,
    /// Leaderboard (wins in half points, games).
    pub board: [(u32, u32); 6],
    /// The narrow layout's tab.
    pub tab: usize,
}

impl Arena {
    /// An arena, optionally running at `now`.
    pub fn new(auto: bool, fail: bool, blind: bool, now: u64) -> Self {
        let mut a = Self {
            sample: 0,
            round: 0,
            start: None,
            stopped: None,
            fail,
            failed: false,
            vote: None,
            blind,
            board_open: false,
            board: BOARD,
            tab: 0,
        };
        if auto {
            a.press("ar-run", now);
        }
        a
    }

    /// The pair this round (model index per side).
    pub fn pair(&self) -> [usize; 2] {
        let p = PAIRS[self.round % PAIRS.len()];
        [p.0, p.1]
    }

    /// Model names shown: revealed after the vote or with Blind off.
    pub fn revealed(&self) -> bool {
        self.vote.is_some() || !self.blind
    }

    /// The full answer of a side.
    pub fn answer(&self, side: usize) -> &'static str {
        ANSWERS[self.sample][side]
    }

    fn done_at(&self, side: usize) -> Option<u64> {
        let st = self.start?[side];
        let m = MODELS[self.pair()[side]];
        let n = self.answer(side).split_whitespace().count();
        Some(st + m.2 + (n.div_ceil(m.3) as u64) * crate::aikit::STREAM_TICK_MS)
    }

    /// Status of a side at `now`.
    pub fn side(&self, side: usize, now: u64) -> Side {
        let Some(st) = self.start else {
            return Side::Idle;
        };
        if side == 1 && self.failed {
            return Side::Failed;
        }
        let done = self.done_at(side).unwrap_or(0);
        let t = self.stopped.map_or(now, |s| s.min(now));
        if t >= done {
            Side::Done
        } else if self.stopped.is_some() {
            Side::Stopped
        } else if t < st[side] + MODELS[self.pair()[side]].2 {
            Side::Thinking
        } else {
            Side::Writing
        }
    }

    /// Text shown for a side at `now`.
    pub fn text(&self, side: usize, now: u64) -> String {
        let Some(st) = self.start else {
            return String::new();
        };
        if side == 1 && self.failed {
            return String::new();
        }
        let m = MODELS[self.pair()[side]];
        let t = self.stopped.map_or(now, |s| s.min(now));
        let at = st[side] + m.2;
        if t < at {
            return String::new();
        }
        let full = self.answer(side);
        first_words(full, words_at(full, t - at, m.3))
    }

    /// Running (either side thinking or writing).
    pub fn running(&self, now: u64) -> bool {
        (0..2).any(|s| matches!(self.side(s, now), Side::Thinking | Side::Writing))
    }

    /// The side that finished first, once both are done.
    pub fn first(&self, now: u64) -> Option<usize> {
        let a = self.side(0, now) == Side::Done;
        let b = self.side(1, now) == Side::Done;
        match (a, b) {
            (true, true) => Some(usize::from(
                self.done_at(1).unwrap_or(0) < self.done_at(0).unwrap_or(0),
            )),
            (true, false) => Some(0),
            (false, true) => Some(1),
            _ => None,
        }
    }

    /// Voting open: both answers in and no vote yet.
    pub fn can_vote(&self, now: u64) -> bool {
        self.vote.is_none() && (0..2).all(|s| self.side(s, now) == Side::Done)
    }

    /// Stats line of a side: first token, words per second, cost.
    pub fn stats(&self, side: usize, now: u64) -> Option<(String, String, String)> {
        let st = self.side(side, now);
        if matches!(st, Side::Idle | Side::Thinking | Side::Failed) {
            return None;
        }
        let m = MODELS[self.pair()[side]];
        let words = self.text(side, now).split_whitespace().count() as f64;
        let tps = m.3 as f64 * 1000.0 / crate::aikit::STREAM_TICK_MS as f64 * 1.3;
        let full = self.answer(side).split_whitespace().count() as f64;
        Some((
            format!("{:.2}s", m.2 as f64 / 1000.0),
            format!("{tps:.0} tok/s"),
            usd(m.4 * (words / full.max(1.0))),
        ))
    }

    /// Leaderboard rows: model, win rate (0..1), games; best first, top five.
    pub fn leaderboard(&self) -> Vec<(usize, f64, u32)> {
        let mut v: Vec<(usize, f64, u32)> = self
            .board
            .iter()
            .enumerate()
            .map(|(i, &(w, g))| (i, f64::from(w) / 2.0 / f64::from(g.max(1)), g))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v.truncate(5);
        v
    }

    fn run(&mut self, now: u64) -> String {
        self.start = Some([now, now]);
        self.stopped = None;
        self.vote = None;
        self.failed = false;
        if self.fail {
            self.fail = false;
            self.failed = true;
            return "Running. Model B failed to respond. Retry is available".into();
        }
        "Running on both models. The answers are simulated".into()
    }

    /// Presses: `ar-run` (Run or Stop), `ar-sample-<i>`, `ar-vote-<a|tie|b|bad>`, `ar-next`,
    /// `ar-blind`, `ar-board`, `ar-retry`, `ar-tab-<i>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ar-run" => {
                if self.running(now) {
                    self.stopped = Some(now);
                    return Some("Stopped both answers".into());
                }
                return Some(self.run(now));
            }
            "ar-next" => {
                self.round += 1;
                self.sample = (self.sample + 1) % SAMPLES.len();
                return Some(format!("Round {}. {}", self.round + 1, self.run(now)));
            }
            "ar-blind" => {
                self.blind = !self.blind;
                return Some(
                    if self.blind {
                        "Blind on: names hidden until you vote"
                    } else {
                        "Blind off: names shown"
                    }
                    .into(),
                );
            }
            "ar-board" => {
                self.board_open = !self.board_open;
                return None;
            }
            "ar-retry" if self.failed => {
                self.failed = false;
                if let Some(s) = self.start.as_mut() {
                    s[1] = now;
                }
                self.stopped = None;
                return Some("Retrying model B".into());
            }
            _ => {}
        }
        if let Some(i) = nth(id, "ar-sample-") {
            if i >= SAMPLES.len() || self.running(now) {
                return None;
            }
            self.sample = i;
            return Some(self.run(now));
        }
        if let Some(i) = nth(id, "ar-tab-") {
            self.tab = i.min(1);
            return None;
        }
        if let Some(v) = id.strip_prefix("ar-vote-") {
            if !self.can_vote(now) {
                return None;
            }
            let v = match v {
                "a" => Vote::A,
                "b" => Vote::B,
                "tie" => Vote::Tie,
                _ => Vote::Bad,
            };
            let [a, b] = self.pair();
            let (wa, wb) = match v {
                Vote::A => (2, 0),
                Vote::B => (0, 2),
                Vote::Tie => (1, 1),
                Vote::Bad => (0, 0),
            };
            self.board[a].0 += wa;
            self.board[b].0 += wb;
            self.board[a].1 += 1;
            self.board[b].1 += 1;
            self.vote = Some(v);
            return Some(format!(
                "{}. A was {}, B was {}. Leaderboard updated",
                v.words(),
                MODELS[a].0,
                MODELS[b].0
            ));
        }
        None
    }

    /// Keys: Escape stops a run or closes the leaderboard; Left/Right/Home/End move the narrow tabs.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match k {
            "Escape" if self.board_open => {
                self.board_open = false;
                None
            }
            "Escape" if self.running(now) => self.press("ar-run", now),
            "ArrowLeft" | "ArrowRight" | "Home" | "End" => {
                self.tab = rove(self.tab, 2, k);
                Some(format!("Model {}", if self.tab == 0 { "A" } else { "B" }))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_vote_reveal() {
        let mut a = Arena::new(true, false, true, 0);
        assert_eq!(a.side(0, 100), Side::Thinking);
        assert_eq!(a.side(0, 1_000), Side::Writing);
        assert!(!a.revealed());
        assert!(a.can_vote(20_000));
        assert_eq!(a.first(20_000), Some(1));
        let before = a.board[1];
        a.press("ar-vote-b", 20_000);
        assert!(a.revealed());
        assert_eq!(a.board[1], (before.0 + 2, before.1 + 1));
        assert_eq!(a.leaderboard().len(), 5);
        a.press("ar-next", 20_000);
        assert_eq!(a.pair(), [2, 4]);
        assert!(a.vote.is_none());
        a.press("ar-run", 20_100);
        assert_eq!(a.side(0, 20_200), Side::Stopped);
    }

    #[test]
    fn failure_and_retry() {
        let mut a = Arena::new(true, true, true, 0);
        assert_eq!(a.side(1, 5_000), Side::Failed);
        assert!(!a.can_vote(30_000));
        a.press("ar-retry", 30_000);
        assert!(a.can_vote(60_000));
        a.key("right", 0);
        assert_eq!(a.tab, 1);
    }
}
