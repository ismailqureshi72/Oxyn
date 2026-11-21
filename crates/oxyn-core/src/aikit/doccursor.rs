//! Agent doc cursor (Arc "Agent cursor"): an agent working on a shared document like a coworker.
//! Its labelled pointer marks a paragraph and retypes it, adds a new section and corrects a
//! misspelling while a colleague reads along; edits stay highlighted until someone keeps or reverts
//! them from a review strip.

use super::words_at;

/// Kinds of block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// A section heading.
    Heading,
    /// A paragraph.
    Para,
}

/// The document title and the muted line under it.
pub const TITLE: &str = "Launch brief: Tally 2.0";
/// Meta line.
pub const META: &str = "Owner Maya Lund · ships Mar 12";
/// The agent's name and its mark.
pub const AGENT: (&str, &str) = ("Claude", "claude");
/// The teammate reading along.
pub const MATE: &str = "Maya";

/// The starting document.
pub const BLOCKS: [(BlockKind, &str); 6] = [
    (BlockKind::Heading, "Goal"),
    (
        BlockKind::Para,
        "Ship Tally 2.0 to every workspace by March 12, with offline sync on by default.",
    ),
    (BlockKind::Heading, "Audience"),
    (
        BlockKind::Para,
        "Small finance teams who track spending in spreadsheets today and want something faster that still feels familiar.",
    ),
    (BlockKind::Heading, "Rollout"),
    (
        BlockKind::Para,
        "Offline sync goes to 10% of workspaces first and we watch sync errors for a week. Customers will recieve an email the day it reaches them.",
    ),
];

/// The rewrite of block 3.
pub const REWRITE: &str = "Finance teams of two to twenty people who live in spreadsheets and want a quicker tool that keeps the grid they already know.";
/// The section added after block 5.
pub const INSERTED: [(BlockKind, &str); 2] = [
    (BlockKind::Heading, "Risks"),
    (
        BlockKind::Para,
        "Very large workbooks may sync slowly the first time they open; support has a short guide ready.",
    ),
];
/// The typo and its fix in block 5.
pub const TYPO: (&str, &str) = ("recieve", "receive");

/// A beat of reading before the first edit, ms.
pub const THINK_MS: u64 = 1_000;
/// Cursor travel, ms.
pub const TRAVEL_MS: u64 = 480;
/// Drag-selecting a paragraph, ms.
pub const SELECT_MS: u64 = 640;
/// Words stream every this many ms.
pub const WORD_MS: u64 = 34;
/// A looping demo accepts after this long in review, ms.
pub const LOOP_ACCEPT_MS: u64 = 3_600;
/// Accepted holds this long before the loop replays, ms.
pub const LOOP_REST_MS: u64 = 1_800;

/// Where the run is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for Ask.
    Idle,
    /// Reading the doc.
    Thinking,
    /// Making edit `n` (0 rewrite, 1 insert, 2 typo).
    Editing(usize),
    /// Edits wait for review.
    Review,
    /// Accepted.
    Accepted,
    /// Undone.
    Reverted,
    /// Stopped by the person.
    Stopped,
    /// The agent could not be reached.
    Error,
}

/// What a block shows at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockView {
    /// Stable id (`b0`… or `n0`, `n1` for inserted).
    pub id: String,
    /// Heading or paragraph.
    pub kind: BlockKind,
    /// The text before the highlighted part.
    pub text: String,
    /// The part tinted as the agent's change (or selection).
    pub tint: String,
    /// The text after it.
    pub tail: String,
    /// `select` (being selected), `stream`, `change` (pending), or empty.
    pub mark: &'static str,
    /// The agent's cursor sits at the end of the tinted part.
    pub agent: bool,
    /// The teammate's cursor sits at the start of the block.
    pub mate: bool,
}

/// The live state of the doc.
#[derive(Clone, Debug, PartialEq)]
pub struct DocCursor {
    /// Run start (None until Ask).
    pub start: Option<u64>,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// Accepted at / reverted at.
    pub decided: Option<(bool, u64)>,
    /// Loop the demo.
    pub looping: bool,
    /// Playback speed (×100).
    pub speed: u64,
    /// Fails after reading (the error scenario).
    pub fail: bool,
    /// Pointer or focus inside: a repeating demo waits in review.
    pub hold: bool,
    /// Show the teammate.
    pub mate: bool,
    /// The edits of the last accepted run are part of the doc.
    pub applied: bool,
}

fn rewrite_ms() -> u64 {
    REWRITE.split_whitespace().count() as u64 * WORD_MS + 110
}
fn insert_ms() -> u64 {
    (INSERTED[0].1.split_whitespace().count() + INSERTED[1].1.split_whitespace().count()) as u64
        * WORD_MS
        + 300
}

impl DocCursor {
    /// A doc that starts its run at `now` when `auto`.
    pub fn new(auto: bool, looping: bool, speed: u64, fail: bool, mate: bool, now: u64) -> Self {
        Self {
            start: auto.then_some(now),
            stopped: None,
            decided: None,
            looping,
            speed: speed.clamp(25, 400),
            fail,
            hold: false,
            mate,
            applied: false,
        }
    }

    fn scale(&self, ms: u64) -> u64 {
        ms * 100 / self.speed
    }

    /// When each edit starts and ends, relative to the run start.
    fn plan(&self) -> [(u64, u64); 3] {
        let t1 = self.scale(THINK_MS);
        let e1 = t1 + self.scale(TRAVEL_MS + SELECT_MS + rewrite_ms());
        let e2 = e1 + self.scale(TRAVEL_MS + insert_ms());
        let e3 = e2 + self.scale(TRAVEL_MS + 300 + 320);
        [(t1, e1), (e1, e2), (e2, e3)]
    }

    /// The run's own clock (frozen when stopped), relative to its start.
    fn el(&self, now: u64) -> Option<u64> {
        let s = self.start?;
        Some(self.stopped.unwrap_or(now).saturating_sub(s))
    }

    /// When a looping run accepts on its own.
    fn auto_accept_at(&self) -> Option<u64> {
        let s = self.start?;
        (self.looping && !self.hold && !self.fail)
            .then(|| s + self.plan()[2].1 + self.scale(LOOP_ACCEPT_MS))
    }

    /// The phase at `now`.
    pub fn phase(&self, now: u64) -> Phase {
        if let Some((ok, _)) = self.decided {
            return if ok { Phase::Accepted } else { Phase::Reverted };
        }
        let Some(el) = self.el(now) else {
            return Phase::Idle;
        };
        let p = self.plan();
        let ph = if el < p[0].0 {
            Phase::Thinking
        } else if self.fail {
            Phase::Error
        } else if let Some(i) = p.iter().position(|e| el < e.1) {
            Phase::Editing(i)
        } else {
            Phase::Review
        };
        if self.stopped.is_some() && matches!(ph, Phase::Thinking | Phase::Editing(_)) {
            return Phase::Stopped;
        }
        if ph == Phase::Review && self.auto_accept_at().is_some_and(|a| now >= a) {
            return Phase::Accepted;
        }
        ph
    }

    /// Edits finished (and so pending review), 0..3.
    pub fn edits_done(&self, now: u64) -> usize {
        match self.el(now) {
            Some(el) if !self.fail => self.plan().iter().filter(|e| el >= e.1).count(),
            _ => 0,
        }
    }

    /// Ms until a looping demo restarts after its auto-accept (for hosts that restart it).
    pub fn restart_due(&self, now: u64) -> bool {
        self.auto_accept_at()
            .is_some_and(|a| self.decided.is_none() && now >= a + self.scale(LOOP_REST_MS))
    }

    /// The state with a looping demo's finished runs rolled forward to the current run.
    pub fn rolled(&self, now: u64) -> Self {
        let mut v = self.clone();
        if let (Some(s), Some(a)) = (v.start, v.auto_accept_at()) {
            let cyc = a + v.scale(LOOP_REST_MS) - s;
            if v.decided.is_none() && now >= s + cyc {
                v.start = Some(s + (now - s) / cyc * cyc);
                v.applied = false;
            }
        }
        v
    }

    /// The status line.
    pub fn status(&self, now: u64) -> String {
        match self.phase(now) {
            Phase::Idle => "Ask Claude to tighten the brief".into(),
            Phase::Thinking => "Reading the brief".into(),
            Phase::Editing(0) => "Rewriting Audience".into(),
            Phase::Editing(1) => "Adding a Risks section".into(),
            Phase::Editing(_) => "Fixing a typo in Rollout".into(),
            Phase::Review => "3 changes ready for review".into(),
            Phase::Accepted => "Changes accepted".into(),
            Phase::Reverted => "Changes undone".into(),
            Phase::Stopped => {
                let n = self.edits_done(now);
                if n == 0 {
                    "Stopped before any change".into()
                } else {
                    format!(
                        "Stopped · {n} change{} kept for review",
                        if n == 1 { "" } else { "s" }
                    )
                }
            }
            Phase::Error => "Couldn't reach Claude".into(),
        }
    }

    /// The document as it should look at `now`.
    pub fn blocks(&self, now: u64) -> Vec<BlockView> {
        let ph = self.phase(now);
        let el = self.el(now).unwrap_or(0);
        let p = self.plan();
        let accepted = ph == Phase::Accepted || (self.applied && self.start.is_none());
        let reverted = ph == Phase::Reverted;
        let done = if reverted { 0 } else { self.edits_done(now) };
        let mut out: Vec<BlockView> = BLOCKS
            .iter()
            .enumerate()
            .map(|(i, (k, t))| BlockView {
                id: format!("b{i}"),
                kind: *k,
                text: t.to_string(),
                tint: String::new(),
                tail: String::new(),
                mark: "",
                agent: false,
                mate: false,
            })
            .collect();
        let tint_pending = !accepted;
        // Edit 0: rewrite block 3.
        let b3 = &mut out[3];
        if done >= 1 {
            b3.text = String::new();
            b3.tint = REWRITE.into();
            b3.mark = if tint_pending { "change" } else { "" };
            if !tint_pending {
                b3.text = REWRITE.into();
                b3.tint.clear();
            }
        } else if let Phase::Editing(0) = ph {
            let e = el - p[0].0;
            let sel_at = self.scale(TRAVEL_MS);
            let stream_at = sel_at + self.scale(SELECT_MS);
            b3.agent = true;
            if e >= stream_at + self.scale(110) {
                let n = words_at(
                    REWRITE,
                    (e - stream_at - self.scale(110)) * self.speed / 100,
                    1,
                );
                b3.text = String::new();
                b3.tint = super::first_words(REWRITE, n);
                b3.mark = "stream";
            } else if e >= sel_at {
                let f = ((e - sel_at) as f64 / self.scale(SELECT_MS) as f64).min(1.0);
                let orig = BLOCKS[3].1;
                let cut = ((orig.len() as f64 * f) as usize).min(orig.len());
                let cut = (0..=cut)
                    .rev()
                    .find(|&c| orig.is_char_boundary(c))
                    .unwrap_or(0);
                b3.text = String::new();
                b3.tint = orig[..cut].into();
                b3.tail = orig[cut..].into();
                b3.mark = "select";
            }
        }
        // Edit 2: fix the typo in block 5.
        let b5 = &mut out[5];
        let (pre, post) = BLOCKS[5].1.split_once(TYPO.0).unwrap_or((BLOCKS[5].1, ""));
        if done >= 3 {
            if tint_pending {
                b5.text = pre.into();
                b5.tint = TYPO.1.into();
                b5.tail = post.into();
                b5.mark = "change";
            } else {
                b5.text = format!("{pre}{}{post}", TYPO.1);
            }
        } else if let Phase::Editing(2) = ph {
            let e = el - p[2].0;
            b5.agent = true;
            b5.text = pre.into();
            b5.tail = post.into();
            b5.tint = if e < self.scale(TRAVEL_MS + 300) {
                TYPO.0
            } else {
                TYPO.1
            }
            .into();
            b5.mark = if e < self.scale(TRAVEL_MS) {
                ""
            } else {
                "select"
            };
        }
        // Edit 1: insert a section after block 5.
        if done >= 2 || matches!(ph, Phase::Editing(1)) {
            let words = if done >= 2 {
                usize::MAX
            } else {
                let e = el - p[1].0;
                if e < self.scale(TRAVEL_MS) {
                    0
                } else {
                    ((e - self.scale(TRAVEL_MS)) * self.speed / 100 / WORD_MS) as usize + 1
                }
            };
            let mut left = words;
            for (j, (k, t)) in INSERTED.iter().enumerate() {
                let n = t.split_whitespace().count();
                let shown = left.min(n);
                left = left.saturating_sub(n);
                if shown == 0 && words != usize::MAX {
                    continue;
                }
                let s = super::first_words(t, shown);
                out.push(BlockView {
                    id: format!("n{j}"),
                    kind: *k,
                    text: if tint_pending {
                        String::new()
                    } else {
                        s.clone()
                    },
                    tint: if tint_pending { s } else { String::new() },
                    tail: String::new(),
                    mark: if !tint_pending {
                        ""
                    } else if done >= 2 {
                        "change"
                    } else {
                        "stream"
                    },
                    agent: done < 2 && (shown < n || j == 1),
                    mate: false,
                });
            }
        }
        // The teammate reads along: one paragraph every 2.6 s.
        if self.mate {
            let paras: Vec<usize> = out
                .iter()
                .enumerate()
                .filter(|(_, b)| b.kind == BlockKind::Para && !b.agent)
                .map(|(i, _)| i)
                .collect();
            if !paras.is_empty() {
                let i = paras[(now / 2_600) as usize % paras.len()];
                out[i].mate = true;
            }
        }
        out
    }

    /// Presses: `dc-ask` (start or run again), `dc-stop`, `dc-accept`, `dc-undo`, `dc-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        let ph = self.phase(now);
        match id {
            "dc-ask" | "dc-retry" => {
                if id == "dc-retry" {
                    self.fail = false;
                }
                if ph == Phase::Accepted {
                    self.applied = true;
                }
                self.start = Some(now);
                self.stopped = None;
                self.decided = None;
                Some("Claude is reading the brief".into())
            }
            "dc-stop" if matches!(ph, Phase::Thinking | Phase::Editing(_)) => {
                self.stopped = Some(now);
                Some(self.status(now))
            }
            "dc-accept"
                if self.edits_done(now) > 0 && matches!(ph, Phase::Review | Phase::Stopped) =>
            {
                self.decided = Some((true, now));
                Some("Changes accepted".into())
            }
            "dc-undo"
                if self.edits_done(now) > 0 && matches!(ph, Phase::Review | Phase::Stopped) =>
            {
                self.decided = Some((false, now));
                Some("Changes undone, the brief is back to how it was".into())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_review_accept_and_undo() {
        let mut d = DocCursor::new(true, false, 100, false, true, 0);
        assert_eq!(d.phase(10), Phase::Thinking);
        let p = d.plan();
        assert_eq!(d.phase(p[0].0 + 1), Phase::Editing(0));
        let mid = d.blocks(p[0].0 + TRAVEL_MS + 100);
        assert_eq!(mid[3].mark, "select");
        assert_eq!(d.phase(p[2].1 + 1), Phase::Review);
        let b = d.blocks(p[2].1 + 1);
        assert_eq!(b.len(), 8);
        assert_eq!(b[5].tint, "receive");
        assert!(b.iter().any(|x| x.mate));
        d.press("dc-accept", p[2].1 + 2);
        let b = d.blocks(p[2].1 + 3);
        assert!(b.iter().all(|x| x.tint.is_empty()));
        assert!(b[5].text.contains("receive"));
        let mut u = DocCursor::new(true, false, 100, false, false, 0);
        u.press("dc-stop", p[1].0 + 10);
        assert_eq!(u.phase(p[2].1 + 50), Phase::Stopped);
        assert_eq!(u.edits_done(p[2].1 + 50), 1);
        u.press("dc-undo", p[2].1 + 60);
        assert_eq!(u.blocks(p[2].1 + 70)[3].text, BLOCKS[3].1);
        let l = DocCursor::new(true, true, 200, false, false, 0);
        assert_eq!(l.phase(1_000_000), Phase::Accepted);
        assert!(l.restart_due(1_000_000));
        assert!(l.rolled(1_000_000).start.unwrap_or(0) > 0);
        let e = DocCursor::new(true, false, 100, true, false, 0);
        assert_eq!(e.phase(THINK_MS + 1), Phase::Error);
    }
}
