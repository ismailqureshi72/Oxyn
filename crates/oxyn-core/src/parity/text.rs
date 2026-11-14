//! Loaders and moving text (plan #28, chunk 5).
//!
//! - [`morph_letters`]: a label morphing into the next, letters matched by character and
//!   occurrence so shared letters glide and new ones enter.
//! - [`SlotText`]: values that spin into place on reels, stopping left to right.
//! - [`Phases`]: a thinking loader walking through phases and settling on "Thought for 4s".
//! - [`ActionSwap`]: an action label that swaps through states with a working phase, a
//!   failure label and timed holds.
//! - [`pixel_frame`]: the 4×4 pixel-grid frames of the loading state (drive, dots, orbit, surfer).
//! - [`Stream`]: a paced word stream with inline citations for streamed answers.
//!
//! Time is passed in as [`ClockMs`].

use crate::ClockMs;

// ---------- text morph ----------

/// What a letter does in a morph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterMove {
    /// In both labels: glides from `from` to `to`.
    Keep,
    /// Only in the new label: staggers in.
    Enter,
    /// Only in the old label: fades out.
    Leave,
}

/// One letter of a morph.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphLetter {
    /// The character.
    pub ch: char,
    /// Stable key: the character and its occurrence ("a1").
    pub key: String,
    /// Move.
    pub mv: LetterMove,
    /// Index in the old label.
    pub from: Option<usize>,
    /// Index in the new label.
    pub to: Option<usize>,
}

fn keyed(s: &str) -> Vec<(char, String)> {
    let mut seen: Vec<(char, usize)> = Vec::new();
    s.chars()
        .map(|c| {
            let n = match seen.iter_mut().find(|(x, _)| *x == c) {
                Some(e) => {
                    e.1 += 1;
                    e.1
                }
                None => {
                    seen.push((c, 0));
                    0
                }
            };
            (c, format!("{c}{n}"))
        })
        .collect()
}

/// Letters of `to` (keep / enter, in order) followed by the letters of `from` that leave.
pub fn morph_letters(from: &str, to: &str) -> Vec<MorphLetter> {
    let a = keyed(from);
    let b = keyed(to);
    let mut out: Vec<MorphLetter> = b
        .iter()
        .enumerate()
        .map(|(j, (c, k))| {
            let i = a.iter().position(|(_, x)| x == k);
            MorphLetter {
                ch: *c,
                key: k.clone(),
                mv: if i.is_some() {
                    LetterMove::Keep
                } else {
                    LetterMove::Enter
                },
                from: i,
                to: Some(j),
            }
        })
        .collect();
    for (i, (c, k)) in a.iter().enumerate() {
        if !b.iter().any(|(_, x)| x == k) {
            out.push(MorphLetter {
                ch: *c,
                key: k.clone(),
                mv: LetterMove::Leave,
                from: Some(i),
                to: None,
            });
        }
    }
    out
}

// ---------- slot text ----------

/// en-US grouping: 12480 → "12,480".
pub fn group(n: i64) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("−{out}") } else { out }
}

/// One reel of a [`SlotText`].
#[derive(Clone, Debug, PartialEq)]
pub struct Reel {
    /// The strip of characters it passes, ending on the target.
    pub strip: Vec<char>,
    /// When it starts landing (ms after the change).
    pub stop_ms: u64,
    /// New (grows in) or removed (collapses).
    pub grow: Option<bool>,
}

/// Slot-machine reels from one value to the next.
#[derive(Clone, Debug, PartialEq)]
pub struct SlotText {
    /// Previous text.
    pub from: String,
    /// Current text.
    pub to: String,
    /// When the value changed.
    pub at: ClockMs,
    /// First reel's spin, ms.
    pub duration_ms: u64,
    /// Between reels stopping, ms.
    pub stagger_ms: u64,
    /// Extra full turns.
    pub spins: usize,
    /// Match reels from the end (numbers) instead of the start (strings).
    pub align_end: bool,
}

impl SlotText {
    /// Settled on `text`.
    pub fn new(text: &str) -> Self {
        Self {
            from: text.into(),
            to: text.into(),
            at: ClockMs(0),
            duration_ms: 900,
            stagger_ms: 70,
            spins: 1,
            align_end: false,
        }
    }
    /// Changes the value at `now`; mid-spin it restarts from what is on screen.
    pub fn set(&mut self, text: &str, now: ClockMs) {
        if text == self.to {
            return;
        }
        self.from = if self.done(now) {
            self.to.clone()
        } else {
            self.frame(now)
        };
        self.to = text.into();
        self.at = now;
    }
    /// All reels landed.
    pub fn done(&self, now: ClockMs) -> bool {
        let n = self.reels().len() as u64;
        now.0.saturating_sub(self.at.0) >= self.duration_ms + self.stagger_ms * n.saturating_sub(1)
    }
    fn pairs(&self) -> Vec<(Option<char>, Option<char>)> {
        let a: Vec<char> = self.from.chars().collect();
        let b: Vec<char> = self.to.chars().collect();
        let n = a.len().max(b.len());
        (0..n)
            .map(|i| {
                if self.align_end {
                    let ai = (i + a.len()).checked_sub(n);
                    let bi = (i + b.len()).checked_sub(n);
                    (ai.map(|x| a[x]), bi.map(|x| b[x]))
                } else {
                    (a.get(i).copied(), b.get(i).copied())
                }
            })
            .collect()
    }
    /// The reels, left to right.
    pub fn reels(&self) -> Vec<Reel> {
        let pairs = self.pairs();
        let mut k = 0u64;
        pairs
            .into_iter()
            .map(|(f, t)| {
                let grow = match (f, t) {
                    (None, Some(_)) => Some(true),
                    (Some(_), None) => Some(false),
                    _ => None,
                };
                let target = t.or(f).unwrap_or(' ');
                let strip = match (f, t) {
                    (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => {
                        let (a, b) = (a as u8 - b'0', b as u8 - b'0');
                        let steps = (b + 10 - a) % 10;
                        let total = steps as usize + 10 * (if a == b { 0 } else { self.spins });
                        (0..=total)
                            .map(|s| (b'0' + ((a as usize + s) % 10) as u8) as char)
                            .collect()
                    }
                    (Some(a), Some(b)) if a != b => vec![a, b],
                    _ => vec![target],
                };
                let moving = strip.len() > 1;
                let stop = self.duration_ms + self.stagger_ms * k;
                if moving {
                    k += 1;
                }
                Reel {
                    strip,
                    stop_ms: stop,
                    grow,
                }
            })
            .collect()
    }
    /// Position of reel `r` along its strip at `now` (0 = first cell, len-1 = landed); a fast
    /// start and a long soft landing that never passes the target.
    pub fn pos(&self, r: &Reel, now: ClockMs) -> f32 {
        let last = (r.strip.len() - 1) as f32;
        if last <= 0.0 {
            return 0.0;
        }
        let t = (now.0.saturating_sub(self.at.0) as f32 / r.stop_ms.max(1) as f32).clamp(0.0, 1.0);
        let e = 1.0 - (1.0 - t).powi(3);
        e * last
    }
    /// The text on screen at `now` (each reel's nearest cell).
    pub fn frame(&self, now: ClockMs) -> String {
        self.reels()
            .iter()
            .filter(|r| r.grow != Some(false) || !self.done(now))
            .map(|r| r.strip[(self.pos(r, now).round() as usize).min(r.strip.len() - 1)])
            .collect()
    }
}

// ---------- thinking loader ----------

/// Phases of a thinking loader.
#[derive(Clone, Debug, PartialEq)]
pub struct Phases {
    /// Labels.
    pub list: Vec<String>,
    /// Per phase, ms.
    pub interval_ms: u64,
    /// Start over after the last.
    pub looping: bool,
    /// Started.
    pub started: ClockMs,
    /// Finished at.
    pub done: Option<ClockMs>,
}

impl Phases {
    /// The default phases.
    pub fn standard(now: ClockMs) -> Self {
        Self {
            list: [
                "Thinking",
                "Searching the web",
                "Reading sources",
                "Writing response",
            ]
            .map(String::from)
            .to_vec(),
            interval_ms: 2200,
            looping: false,
            started: now,
            done: None,
        }
    }
    /// The phase index at `now`.
    pub fn index(&self, now: ClockMs) -> usize {
        let n = self.list.len().max(1);
        let i = (now.0.saturating_sub(self.started.0) / self.interval_ms.max(1)) as usize;
        if self.looping { i % n } else { i.min(n - 1) }
    }
    /// Label at `now`: the phase, or "Thought for Ns" once done.
    pub fn label(&self, now: ClockMs) -> String {
        match self.done {
            Some(d) => format!(
                "Thought for {}s",
                (d.0.saturating_sub(self.started.0) as f64 / 1000.0)
                    .round()
                    .max(1.0)
            ),
            None => self.list.get(self.index(now)).cloned().unwrap_or_default(),
        }
    }
}

// ---------- action swap ----------

/// One state of an [`ActionSwap`].
#[derive(Clone, Debug, PartialEq)]
pub struct SwapState {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// `neutral` · `accent` · `success` · `warning` · `quiet`.
    pub tone: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Shown while the action runs on the way into this state (else a round spinner).
    pub pending: Option<String>,
    /// Advance on its own after this long.
    pub hold_ms: Option<u64>,
}

impl SwapState {
    /// A neutral state.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tone: "neutral".into(),
            icon: None,
            pending: None,
            hold_ms: None,
        }
    }
}

/// Phase of an [`ActionSwap`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwapPhase {
    /// Showing the current state.
    Idle,
    /// The action runs (since).
    Working(ClockMs),
    /// The action failed (since); shows the failure label for 1.8 s.
    Failed(ClockMs),
}

/// A button that swaps its label through states.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionSwap {
    /// States, in order.
    pub states: Vec<SwapState>,
    /// Current index.
    pub current: usize,
    /// Phase.
    pub phase: SwapPhase,
    /// When the current state was entered.
    pub since: ClockMs,
    /// Shown after a failure.
    pub failure_label: String,
}

/// Minimum time the working shape shows, ms.
pub const SWAP_MIN_WORK: u64 = 450;
/// How long the failure label shows, ms.
pub const SWAP_FAIL: u64 = 1800;

impl ActionSwap {
    /// At the first state.
    pub fn new(states: Vec<SwapState>) -> Self {
        Self {
            states,
            current: 0,
            phase: SwapPhase::Idle,
            since: ClockMs(0),
            failure_label: "Try again".into(),
        }
    }
    /// The next index.
    pub fn next(&self) -> usize {
        (self.current + 1) % self.states.len().max(1)
    }
    /// Pressed: starts working (`async_action`) or advances right away.
    pub fn press(&mut self, async_action: bool, now: ClockMs) {
        if matches!(self.phase, SwapPhase::Working(_)) {
            return;
        }
        if async_action {
            self.phase = SwapPhase::Working(now);
        } else {
            self.advance(now);
        }
    }
    fn advance(&mut self, now: ClockMs) {
        self.current = self.next();
        self.phase = SwapPhase::Idle;
        self.since = now;
    }
    /// The action settled: success advances (after the minimum working time), failure shows the
    /// failure label and stays.
    pub fn settle(&mut self, ok: bool, now: ClockMs) {
        if let SwapPhase::Working(s) = self.phase {
            let at = ClockMs(now.0.max(s.0 + SWAP_MIN_WORK));
            if ok {
                self.advance(at);
            } else {
                self.phase = SwapPhase::Failed(at);
            }
        }
    }
    /// Timers: holds advance, the failure label clears.
    pub fn tick(&mut self, now: ClockMs) {
        match self.phase {
            SwapPhase::Failed(s) if now.0.saturating_sub(s.0) >= SWAP_FAIL => {
                self.phase = SwapPhase::Idle
            }
            SwapPhase::Idle => {
                if let Some(h) = self.states.get(self.current).and_then(|s| s.hold_ms)
                    && now.0.saturating_sub(self.since.0) >= h
                {
                    self.advance(ClockMs(self.since.0 + h));
                }
            }
            _ => {}
        }
    }
    /// What the button shows: label (None = round spinner), tone, icon.
    pub fn view(&self) -> (Option<String>, String, Option<String>) {
        let s = &self.states[self.current.min(self.states.len() - 1)];
        match self.phase {
            SwapPhase::Working(_) => {
                let n = &self.states[self.next()];
                (n.pending.clone(), n.tone.clone(), None)
            }
            SwapPhase::Failed(_) => (
                Some(self.failure_label.clone()),
                "warning".into(),
                s.icon.clone(),
            ),
            SwapPhase::Idle => (Some(s.label.clone()), s.tone.clone(), s.icon.clone()),
        }
    }
}

// ---------- loading state ----------

/// One 4×4 frame (row-major, 16 cells, 0..=3 brightness) of a pixel-grid loader.
pub fn pixel_frame(variant: &str, t: ClockMs) -> [u8; 16] {
    let mut g = [0u8; 16];
    let step = (t.0 / 110) as usize;
    // Perimeter order, clockwise from the top-left.
    const RING: [usize; 12] = [0, 1, 2, 3, 7, 11, 15, 14, 13, 12, 8, 4];
    match variant {
        "dots" => {
            for (k, c) in [5usize, 6, 10, 9].iter().enumerate() {
                g[*c] = if (step / 2 + k).is_multiple_of(4) {
                    3
                } else {
                    1
                };
            }
        }
        "orbit" => {
            for k in 0..3 {
                g[RING[(step + 12 - k) % 12]] = 3 - k as u8;
            }
        }
        "surfer" => {
            for col in 0..4 {
                let h = ((step + col) % 6) as i32;
                let y = (if h > 3 { 6 - h } else { h }).clamp(0, 3) as usize;
                g[(3 - y) * 4 + col] = 3;
                if y > 0 {
                    g[(4 - y) * 4 + col] = 1;
                }
            }
        }
        // drive: a lit column sweeping left to right with a trail.
        _ => {
            let x = step % 6;
            for row in 0..4 {
                for col in 0..4 {
                    let d = x as i32 - col as i32;
                    g[row * 4 + col] = match d {
                        0 => 3,
                        1 => 2,
                        2 => 1,
                        _ => 0,
                    };
                }
            }
        }
    }
    g
}

/// Elapsed seconds with one decimal ("3.4s").
pub fn elapsed_label(started: ClockMs, now: ClockMs) -> String {
    format!("{:.1}s", now.0.saturating_sub(started.0) as f64 / 1000.0)
}

// ---------- streaming text ----------

/// A streamed answer: words arrive at a pace; `[n]` markers become citation chips.
#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    /// The full answer (the demo streams it; a host appends as tokens come).
    pub text: String,
    /// Started.
    pub started: ClockMs,
    /// Words per second.
    pub wps: f32,
}

/// A piece of a streamed answer.
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    /// A word with its trailing space.
    Word(String),
    /// A citation chip (1-based source index).
    Cite(usize),
}

impl Stream {
    /// Every piece of the answer.
    pub fn pieces(&self) -> Vec<Piece> {
        let mut out = Vec::new();
        for w in self.text.split_inclusive(' ') {
            let t = w.trim_end();
            if let Some(n) = t
                .strip_prefix('[')
                .and_then(|x| x.strip_suffix(']'))
                .and_then(|x| x.parse::<usize>().ok())
            {
                out.push(Piece::Cite(n));
            } else {
                out.push(Piece::Word(w.to_string()));
            }
        }
        out
    }
    /// How many pieces are visible at `now`.
    pub fn shown(&self, now: ClockMs) -> usize {
        let n = (now.0.saturating_sub(self.started.0) as f32 / 1000.0 * self.wps) as usize;
        n.min(self.pieces().len())
    }
    /// Finished streaming.
    pub fn done(&self, now: ClockMs) -> bool {
        self.shown(now) >= self.pieces().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph() {
        let m = morph_letters("Follow", "Following");
        assert_eq!(m.iter().filter(|l| l.mv == LetterMove::Keep).count(), 6);
        assert_eq!(m.iter().filter(|l| l.mv == LetterMove::Enter).count(), 3);
        let m = morph_letters("Save", "Saved");
        assert!(m.iter().all(|l| l.mv != LetterMove::Leave));
        let m = morph_letters("Copy", "Done");
        assert_eq!(m.iter().filter(|l| l.mv == LetterMove::Leave).count(), 3);
        assert_eq!(m[0].key, "D0");
    }

    #[test]
    fn slot() {
        assert_eq!(group(12480), "12,480");
        assert_eq!(group(-999), "−999");
        let mut s = SlotText::new("999");
        s.align_end = true;
        s.set("1,000", ClockMs(0));
        let r = s.reels();
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].grow, Some(true));
        assert_eq!(*r.last().unwrap().strip.last().unwrap(), '0');
        assert_eq!(r[4].strip.len(), 12); // 9 → 0 is 1 step + one spin
        assert!(s.done(ClockMs(5000)));
        assert_eq!(s.frame(ClockMs(5000)), "1,000");
        assert!(r[1].stop_ms <= r[4].stop_ms);
    }

    #[test]
    fn phases_and_swap() {
        let mut p = Phases::standard(ClockMs(0));
        assert_eq!(p.label(ClockMs(2300)), "Searching the web");
        assert_eq!(p.index(ClockMs(99_000)), 3);
        p.looping = true;
        assert_eq!(p.index(ClockMs(8_800)), 0);
        p.done = Some(ClockMs(4_200));
        assert_eq!(p.label(ClockMs(9_000)), "Thought for 4s");

        let mut a = ActionSwap::new(vec![
            SwapState::new("f", "Follow"),
            SwapState {
                hold_ms: Some(1500),
                ..SwapState::new("d", "Following")
            },
        ]);
        a.press(true, ClockMs(0));
        assert_eq!(a.view().0, None);
        a.settle(false, ClockMs(100));
        assert_eq!(a.view().0.as_deref(), Some("Try again"));
        a.tick(ClockMs(2300));
        assert_eq!(a.phase, SwapPhase::Idle);
        a.press(true, ClockMs(3000));
        a.settle(true, ClockMs(3100));
        assert_eq!(a.current, 1);
        assert_eq!(a.since, ClockMs(3450));
        a.tick(ClockMs(5000));
        assert_eq!(a.current, 0);
    }

    #[test]
    fn loader_and_stream() {
        assert_eq!(
            pixel_frame("orbit", ClockMs(0))
                .iter()
                .filter(|&&c| c > 0)
                .count(),
            3
        );
        assert_eq!(pixel_frame("drive", ClockMs(0))[0], 3);
        assert_eq!(elapsed_label(ClockMs(0), ClockMs(3_460)), "3.5s");
        let s = Stream {
            text: "Mint chip is up 12% [1] this summer [2]".into(),
            started: ClockMs(0),
            wps: 10.0,
        };
        assert_eq!(
            s.pieces()
                .iter()
                .filter(|p| matches!(p, Piece::Cite(_)))
                .count(),
            2
        );
        assert_eq!(s.shown(ClockMs(300)), 3);
        assert!(s.done(ClockMs(10_000)));
    }
}
