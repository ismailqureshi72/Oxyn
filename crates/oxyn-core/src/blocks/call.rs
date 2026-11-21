//! Call insights: a sales call in one card. Talk time split by speaker, key moments on a rail
//! (pricing, competitor, objection) that play their quote with a seekable waveform, an AI
//! summary that streams once, and next steps whose checking lowers the derived deal risk.

use super::{first_words, nth, rove, words_at};

/// The call: prospect, title, platform brand, date, length in seconds.
pub const CALL: (&str, &str, &str, &str, u32) = (
    "Pinecrest Labs",
    "Evaluation call",
    "zoom",
    "Nov 19",
    34 * 60,
);

/// Speakers: name, title, side (0 us, 1 them), talk share in percent.
pub const SPEAKERS: [(&str, &str, u8, u32); 4] = [
    ("Ana Okafor", "Account executive", 0, 31),
    ("Ben Lund", "Solutions engineer", 0, 17),
    ("Dev Raman", "Head of finance", 1, 34),
    ("Lea Moritz", "Controller", 1, 18),
];

/// Moment kinds: label, icon, hue, risk points.
pub const KINDS: [(&str, &str, &str, u32); 3] = [
    ("Pricing", "bolt", "amber", 10),
    ("Competitor", "eye", "violet", 15),
    ("Objection", "alert", "red", 20),
];

/// Moments: second in the call, kind, speaker, quote.
pub const MOMENTS: [(u32, usize, usize, &str); 5] = [
    (
        312,
        0,
        2,
        "If it's per seat we'd be at forty people by spring, so I need to see the volume tiers.",
    ),
    (
        655,
        1,
        3,
        "We're also looking at Ledgerly; they offered a three month pilot.",
    ),
    (
        1_040,
        2,
        2,
        "Our auditors need every approval exported. If that's manual, it's a blocker.",
    ),
    (
        1_510,
        0,
        0,
        "Annual billing takes eighteen percent off, and the pilot credits carry over.",
    ),
    (
        1_880,
        2,
        3,
        "I'm not sure our ERP sync can run nightly without someone watching it.",
    ),
];

/// The AI summary.
pub const SUMMARY: &str = "Pinecrest wants to replace spreadsheets for approvals before their spring audit. Finance likes the workflow but needs audit exports and volume pricing. Ledgerly is in the running with a free pilot. The ERP sync is the main technical doubt; a sandbox test was offered.";

/// Next steps: text, owner, due, risk points it removes.
pub const STEPS: [(&str, &str, &str, u32); 3] = [
    (
        "Send volume pricing for 40 seats",
        "Ana Okafor",
        "Nov 21",
        10,
    ),
    ("Share an audit export sample", "Ben Lund", "Nov 22", 20),
    ("Book the ERP sandbox test", "Ben Lund", "Nov 26", 15),
];

/// Clip length around a moment, seconds.
pub const CLIP_S: u32 = 12;
/// Autoplay tour: ms per moment.
pub const TOUR_MS: u64 = 4_000;

/// Deal risk from moments and done steps: 0 Low, 1 Medium, 2 High, with its points.
pub fn derive_risk(done: &[bool]) -> (usize, u32) {
    let mut pts: u32 = MOMENTS.iter().map(|m| KINDS[m.1].3).sum::<u32>() / 2;
    for (i, d) in done.iter().enumerate() {
        if *d {
            pts = pts.saturating_sub(STEPS[i].3);
        }
    }
    (
        if pts >= 30 {
            2
        } else if pts >= 15 {
            1
        } else {
            0
        },
        pts,
    )
}

/// Risk words.
pub const RISK: [(&str, &str); 3] = [
    ("Low risk", "green"),
    ("Medium risk", "amber"),
    ("High risk", "red"),
];

/// The card.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    /// Selected moment.
    pub moment: usize,
    /// Touring (autoplay) since, until you interact.
    pub tour: Option<u64>,
    /// Playing: started at (clock) from second (within the clip).
    pub playing: Option<(u64, u32)>,
    /// Position within the clip when paused, seconds.
    pub pos: u32,
    /// Summary started streaming at.
    pub summary_at: u64,
    /// Steps done.
    pub done: [bool; 3],
    /// Copied.
    pub copied: bool,
}

impl Call {
    /// A card at `now`, touring when `tour`.
    pub fn new(tour: bool, done: usize, now: u64) -> Self {
        let mut d = [false; 3];
        for x in d.iter_mut().take(done) {
            *x = true;
        }
        Self {
            moment: 0,
            tour: tour.then_some(now),
            playing: None,
            pos: 0,
            summary_at: now,
            done: d,
            copied: false,
        }
    }

    /// Moment shown at `now`.
    pub fn current(&self, now: u64) -> usize {
        match self.tour {
            Some(t) => ((now.saturating_sub(t) / TOUR_MS) as usize) % MOMENTS.len(),
            None => self.moment,
        }
    }

    /// Clip position at `now`, seconds, and whether it is still playing.
    pub fn position(&self, now: u64) -> (u32, bool) {
        match self.playing {
            Some((t, from)) => {
                let p = from + (now.saturating_sub(t) / 1000) as u32;
                if p >= CLIP_S {
                    (CLIP_S, false)
                } else {
                    (p, true)
                }
            }
            None => (self.pos, false),
        }
    }

    /// Summary words visible at `now`.
    pub fn summary(&self, now: u64) -> String {
        first_words(
            SUMMARY,
            words_at(SUMMARY, now.saturating_sub(self.summary_at), 2),
        )
    }

    /// Risk now.
    pub fn risk(&self) -> (usize, u32) {
        derive_risk(&self.done)
    }

    /// Call time of the selected moment as `m:ss` at clip position.
    pub fn stamp(&self, now: u64) -> String {
        let s = MOMENTS[self.current(now)].0 + self.position(now).0;
        format!("{}:{:02}", s / 60, s % 60)
    }

    fn select(&mut self, i: usize) -> Option<String> {
        let m = MOMENTS.get(i)?;
        self.tour = None;
        self.moment = i;
        self.playing = None;
        self.pos = 0;
        Some(format!(
            "{} at {}:{:02}, {}: {}",
            KINDS[m.1].0,
            m.0 / 60,
            m.0 % 60,
            SPEAKERS[m.2].0,
            m.3
        ))
    }

    /// Presses: `ci-m-<i>`, `ci-play` (play or pause), `ci-copy`, `ci-step-<i>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ci-play" => {
                if self.tour.is_some() {
                    let c = self.current(now);
                    self.select(c);
                }
                let (p, on) = self.position(now);
                if on {
                    self.pos = p;
                    self.playing = None;
                    return Some("Paused".into());
                }
                let from = if p >= CLIP_S { 0 } else { p };
                self.playing = Some((now, from));
                return Some("Playing the clip (simulated audio)".into());
            }
            "ci-copy" => {
                self.copied = true;
                return Some("Summary copied".into());
            }
            _ => {}
        }
        if let Some(i) = nth(id, "ci-m-") {
            return self.select(i);
        }
        if let Some(i) = nth(id, "ci-step-") {
            let d = !*self.done.get(i)?;
            self.done[i] = d;
            let r = self.risk();
            return Some(format!(
                "{} {}. {}",
                STEPS[i].0,
                if d { "done" } else { "not done" },
                RISK[r.0].0
            ));
        }
        None
    }

    /// Keys: `markers` Left/Right/Home/End select moments; `wave` seeks a second or to an end.
    pub fn key(&mut self, target: &str, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        if !matches!(
            k,
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End"
        ) {
            return None;
        }
        if target == "wave" {
            let (p, on) = self.position(now);
            let np = match k {
                "ArrowLeft" | "ArrowDown" => p.saturating_sub(1),
                "Home" => 0,
                "End" => CLIP_S,
                _ => (p + 1).min(CLIP_S),
            };
            self.pos = np;
            self.playing = on.then_some((now, np));
            return Some(self.stamp(now));
        }
        let i = rove(self.current(now), MOMENTS.len(), k);
        self.select(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moments_play_risk() {
        let mut c = Call::new(true, 0, 0);
        assert_eq!(c.current(4_500), 1);
        assert_eq!(c.risk().0, 2);
        c.key("markers", "end", 0);
        assert_eq!(c.current(9_999), 4);
        assert!(c.tour.is_none());
        c.press("ci-play", 1_000);
        assert_eq!(c.position(4_000), (3, true));
        c.press("ci-play", 4_000);
        assert_eq!(c.position(9_000), (3, false));
        c.key("wave", "right", 9_000);
        assert_eq!(c.pos, 4);
        assert_eq!(c.stamp(9_000), "31:24");
        c.press("ci-step-1", 0);
        c.press("ci-step-2", 0);
        assert_eq!(c.risk().0, 0);
        assert!(c.summary(100).split_whitespace().count() < 10);
    }
}
