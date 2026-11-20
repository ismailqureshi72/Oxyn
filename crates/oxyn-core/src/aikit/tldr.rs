//! TL;DR toggle: an article card with a Full, Summary and Key points switch and a length
//! control. Switching streams an AI summary in place of the article, and each key point jumps
//! back to the paragraph it came from.

use super::{norm_key, nth, rove, words_at};

/// Thinking before a summary streams, ms.
pub const THINK_MS: u64 = 700;
/// A jumped-to paragraph stays highlighted this long, ms.
pub const JUMP_MS: u64 = 1_600;

/// The article: title, byline, paragraphs.
pub const TITLE: &str = "Why our agents now ask before they spend";
/// Byline.
pub const BYLINE: &str = "Corvel engineering blog · 6 min read";
/// Paragraphs.
pub const PARAS: [&str; 5] = [
    "Last quarter our agents ran about forty thousand tasks for customers, and a small share of them did something nobody asked for: they bought credits, renewed a domain or upgraded a plan because a tool made it easy.",
    "None of these were bugs in the usual sense. The agent read the goal, found a tool that moved it forward and used it. The problem was that spending money is a different kind of step from reading a file.",
    "We now sort every tool into three tiers. Reading is free to run, writing asks once per session, and anything that spends money or cannot be undone asks every time, with the amount shown in the request.",
    "Approvals used to feel slow, so we made them cheap. The request names the tool, the amount and what happens next, one key approves it, and the agent keeps working on other steps while it waits.",
    "Since the change, unwanted purchases dropped to zero and the median task got four seconds slower. Customers told us that trade was easy: they would rather wait a moment than explain a surprise charge.",
];

/// Summaries by length.
pub const SUMMARIES: [&str; 3] = [
    "Agents now ask every time before spending money, and unwanted purchases fell to zero.",
    "After agents made a few purchases nobody asked for, every tool was sorted into tiers. Spending or irreversible steps now ask every time, approvals are one key, and unwanted purchases fell to zero for about four seconds per task.",
    "Out of forty thousand agent tasks, a few bought credits or upgraded plans without being asked, not because of bugs but because spending was treated like any other step. Tools are now split into three tiers: reading runs freely, writing asks once per session, and spending or irreversible actions ask every time with the amount shown. Approvals were made fast so they do not stall work, and the result was zero unwanted purchases at a cost of about four seconds per task, a trade customers welcomed.",
];

/// Key points: text, source paragraph.
pub const POINTS: [(&str, usize); 5] = [
    ("A few agent tasks spent money nobody asked them to", 0),
    ("Spending is a different kind of step from reading", 1),
    ("Three tiers: read freely, write once, spend every time", 2),
    ("One-key approvals while the agent keeps working", 3),
    ("Zero unwanted purchases for four seconds a task", 4),
];

/// Views.
pub const MODES: [&str; 3] = ["Full", "Summary", "Key points"];
/// Lengths.
pub const LENGTHS: [&str; 3] = ["Short", "Medium", "Long"];

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Tldr {
    /// View.
    pub mode: usize,
    /// Length.
    pub len: usize,
    /// When the view last changed.
    pub since: u64,
    /// Jumped-to paragraph and when.
    pub jump: Option<(usize, u64)>,
    /// The next summary fails (the error scenario).
    pub fail: bool,
}

impl Tldr {
    /// The card in `mode`, switched at `now`.
    pub fn new(mode: usize, len: usize, fail: bool, now: u64) -> Self {
        Self {
            mode: mode.min(2),
            len: len.min(2),
            since: now,
            jump: None,
            fail,
        }
    }

    /// Points shown at the current length (3, 4 or 5).
    pub fn points(&self) -> &'static [(&'static str, usize)] {
        &POINTS[..3 + self.len]
    }

    /// The streamed text of the current view (summary text or points joined).
    pub fn text(&self) -> String {
        match self.mode {
            1 => SUMMARIES[self.len].to_string(),
            2 => self
                .points()
                .iter()
                .map(|p| p.0)
                .collect::<Vec<_>>()
                .join(" "),
            _ => String::new(),
        }
    }

    /// Phase: `full`, `thinking`, `streaming`, `done`, `error`.
    pub fn phase(&self, now: u64) -> &'static str {
        if self.mode == 0 {
            return "full";
        }
        let el = now.saturating_sub(self.since);
        if el < THINK_MS {
            "thinking"
        } else if self.fail {
            "error"
        } else if words_at(&self.text(), el - THINK_MS, 4) < self.text().split_whitespace().count()
        {
            "streaming"
        } else {
            "done"
        }
    }

    /// Words of the view visible.
    pub fn visible(&self, now: u64) -> usize {
        match self.phase(now) {
            "streaming" => words_at(&self.text(), now - self.since - THINK_MS, 4),
            "done" => usize::MAX,
            _ => 0,
        }
    }

    /// Paragraph highlighted by a jump.
    pub fn highlighted(&self, now: u64) -> Option<usize> {
        self.jump.filter(|j| now < j.1 + JUMP_MS).map(|j| j.0)
    }

    /// Reading time saved (`Saves about 5 min`).
    pub fn saved(&self) -> String {
        let full: usize = PARAS.iter().map(|p| p.split_whitespace().count()).sum();
        let s = self.text().split_whitespace().count();
        format!("{} words instead of {}", s, full)
    }

    /// Presses: `td-mode-<i>`, `td-len-<i>`, `td-jump-<i>`, `td-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "td-mode-") {
            let i = i.min(2);
            if i == self.mode {
                return None;
            }
            self.mode = i;
            self.since = now;
            return Some(match i {
                0 => "Full article".into(),
                1 => "Summarizing the article".into(),
                _ => "Listing the key points".into(),
            });
        }
        if let Some(i) = nth(id, "td-len-") {
            let i = i.min(2);
            if i == self.len {
                return None;
            }
            self.len = i;
            if self.mode != 0 {
                self.since = now;
            }
            return Some(format!("{} summary", LENGTHS[i]));
        }
        if let Some(i) = nth(id, "td-jump-") {
            let p = self.points().get(i)?.1;
            self.mode = 0;
            self.since = now;
            self.jump = Some((p, now));
            return Some(format!("Paragraph {} of {}", p + 1, PARAS.len()));
        }
        if id == "td-retry" {
            self.fail = false;
            self.since = now;
            return Some("Summarizing the article".into());
        }
        None
    }

    /// Keys in the view switch (`mode`) or the length control (`len`): arrows, Home, End.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        match part {
            "mode" => {
                let i = rove(self.mode, 3, key);
                self.press(&format!("td-mode-{i}"), now)
            }
            "len" => {
                let i = rove(self.len, 3, key);
                self.press(&format!("td-len-{i}"), now)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tldr_switches_streams_and_jumps() {
        let mut t = Tldr::new(0, 1, false, 0);
        assert_eq!(t.phase(0), "full");
        t.key("mode", "ArrowRight", 0);
        assert_eq!(t.phase(100), "thinking");
        assert_eq!(t.phase(800), "streaming");
        assert_eq!(t.phase(10_000), "done");
        t.press("td-mode-2", 10_000);
        assert_eq!(t.points().len(), 4);
        t.press("td-jump-2", 20_000);
        assert_eq!(t.mode, 0);
        assert_eq!(t.highlighted(20_500), Some(2));
        assert_eq!(t.highlighted(22_000), None);
        t.key("len", "End", 0);
        assert_eq!(t.len, 2);
        let mut f = Tldr::new(1, 0, true, 0);
        assert_eq!(f.phase(1_000), "error");
        f.press("td-retry", 1_000);
        assert_eq!(f.phase(9_000), "done");
        assert!(t.saved().contains("words"));
    }
}
