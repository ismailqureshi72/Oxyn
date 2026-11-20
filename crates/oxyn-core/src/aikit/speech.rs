//! Speech player: read aloud for an AI reply. Read aloud swaps the action row for a compact
//! player in the same slot (play, elapsed, a scrubbable bar, total, speed, voice, Stop reading);
//! the spoken word is highlighted while upcoming words stay dimmed, a click on a word jumps
//! there, and switching voice mid-sentence resumes from the same word.

use super::{mmss, norm_key, nth};

/// The sample reply (blank lines split paragraphs).
pub const SAMPLE: &str = "Lisbon in early November is mild, around 19 degrees by day, with a few short showers. Pack a light rain jacket and shoes with grip for the hills.\n\nStart in Alfama before the crowds, take tram 28 up to Graça for the view, then walk down to the river for lunch at the market.";

/// Voices: name, one line.
pub const VOICES: [(&str, &str); 4] = [
    ("Ava", "Warm and conversational"),
    ("Leo", "Calm and clear"),
    ("Maya", "Bright and quick"),
    ("Noah", "Low and steady"),
];

/// Speeds the speed button cycles through (x10).
pub const SPEEDS: [u64; 3] = [10, 15, 20];

/// Preparing a voice takes this long, ms.
pub const PREP_MS: u64 = 640;

/// The player's phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Action row shown.
    Idle,
    /// Fetching audio.
    Preparing,
    /// Playing.
    Playing,
    /// Paused.
    Paused,
    /// Finished (Play again).
    Done,
    /// Failed (Retry).
    Failed,
}

/// The live state of the speech player.
#[derive(Clone, Debug, PartialEq)]
pub struct Speech {
    /// The reply.
    pub text: String,
    /// Word start times on the 1x timeline, ms.
    pub starts: Vec<u64>,
    /// Total length on the 1x timeline, ms.
    pub total: u64,
    /// Player open (vs the action row).
    pub open: bool,
    /// Phase when last changed.
    pub phase: Phase,
    /// Position on the 1x timeline when last changed, ms.
    pub pos: u64,
    /// `now` when the phase last changed.
    pub since: u64,
    /// Speed index into [`SPEEDS`].
    pub speed: usize,
    /// Voice.
    pub voice: usize,
    /// Voice menu open.
    pub menu: bool,
    /// The next prepare fails.
    pub fail_next: bool,
    /// Feedback given (`up`, `down`).
    pub rating: Option<&'static str>,
}

impl Speech {
    /// The sample; `auto` opens and starts reading at `now`.
    pub fn new(auto: bool, fail: bool, now: u64) -> Self {
        let text = SAMPLE.to_string();
        let mut starts = Vec::new();
        let mut t = 0;
        for w in text.split_whitespace() {
            starts.push(t);
            t += 140 + 28 * w.chars().count() as u64;
            if w.ends_with('.') || w.ends_with(',') {
                t += if w.ends_with('.') { 260 } else { 120 };
            }
        }
        let mut s = Self {
            text,
            starts,
            total: t,
            open: false,
            phase: Phase::Idle,
            pos: 0,
            since: now,
            speed: 0,
            voice: 0,
            menu: false,
            fail_next: fail,
            rating: None,
        };
        if auto {
            s.press("sp-read", now);
        }
        s
    }

    /// Words with paragraph starts: (word, starts a paragraph).
    pub fn words(&self) -> Vec<(String, bool)> {
        let mut v = Vec::new();
        for (pi, para) in self.text.split("\n\n").enumerate() {
            for (wi, w) in para.split_whitespace().enumerate() {
                v.push((w.to_string(), pi > 0 && wi == 0));
            }
        }
        v
    }

    /// Phase at `now` (preparing resolves into playing or failed; playing runs out into done).
    pub fn phase_at(&self, now: u64) -> Phase {
        match self.phase {
            Phase::Preparing if now >= self.since + PREP_MS => {
                if self.fail_next {
                    Phase::Failed
                } else if self.pos_at(now) >= self.total {
                    Phase::Done
                } else {
                    Phase::Playing
                }
            }
            Phase::Playing if self.pos_at(now) >= self.total => Phase::Done,
            p => p,
        }
    }

    /// Position on the 1x timeline at `now`, ms.
    pub fn pos_at(&self, now: u64) -> u64 {
        let run_from = match self.phase {
            Phase::Playing => self.since,
            Phase::Preparing if !self.fail_next => self.since + PREP_MS,
            _ => return self.pos.min(self.total),
        };
        let el = now.saturating_sub(run_from) * SPEEDS[self.speed] / 10;
        (self.pos + el).min(self.total)
    }

    /// Index of the word being spoken.
    pub fn word_at(&self, now: u64) -> usize {
        let p = self.pos_at(now);
        self.starts.iter().rposition(|&s| s <= p).unwrap_or(0)
    }

    /// Elapsed / total on the speed's own clock (`0:04`, `0:22`).
    pub fn times(&self, now: u64) -> (String, String) {
        let k = SPEEDS[self.speed];
        (mmss(self.pos_at(now) * 10 / k), mmss(self.total * 10 / k))
    }

    /// The speed label (`1×`, `1.5×`, `2×`).
    pub fn speed_label(&self) -> &'static str {
        ["1×", "1.5×", "2×"][self.speed]
    }

    /// The play button's name.
    pub fn play_name(&self, now: u64) -> &'static str {
        match self.phase_at(now) {
            Phase::Playing => "Pause",
            Phase::Done => "Play again",
            Phase::Failed => "Retry",
            Phase::Preparing => "Preparing audio",
            _ => "Play",
        }
    }

    /// Stores the derived phase (preparing that resolved, playing that ran out) as the phase.
    fn settle(&mut self, now: u64) {
        let ph = self.phase_at(now);
        if ph != self.phase {
            self.pos = self.pos_at(now);
            self.phase = ph;
            self.since = now;
        }
    }

    fn set(&mut self, phase: Phase, now: u64) {
        self.pos = self.pos_at(now);
        self.phase = phase;
        self.since = now;
    }

    fn seek(&mut self, to: u64, now: u64) -> String {
        let ph = self.phase_at(now);
        self.pos = to.min(self.total);
        self.since = now;
        self.phase = match ph {
            Phase::Done | Phase::Idle => Phase::Paused,
            Phase::Preparing => Phase::Preparing,
            p => p,
        };
        let (e, t) = self.times(now);
        format!("{e} of {t}")
    }

    /// Presses: `sp-read`, `sp-play` (play / pause / again / retry), `sp-stop`, `sp-speed`,
    /// `sp-menu`, `sp-voice-<i>`, `sp-word-<i>`, `sp-seek-<ms>` (1x timeline), `sp-up`, `sp-down`,
    /// `sp-copy`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        self.settle(now);
        let voice = VOICES[self.voice].0;
        if let Some(i) = nth(id, "sp-voice-") {
            let i = i.min(VOICES.len() - 1);
            self.menu = false;
            if i == self.voice {
                return None;
            }
            self.voice = i;
            let ph = self.phase_at(now);
            if matches!(ph, Phase::Playing | Phase::Preparing) {
                // Resume from the start of the word being spoken.
                let w = self.word_at(now);
                self.pos = self.starts[w];
                self.phase = Phase::Preparing;
                self.since = now;
            }
            return Some(format!("Switching to {}", VOICES[i].0));
        }
        if let Some(i) = nth(id, "sp-word-") {
            let to = *self.starts.get(i)?;
            return Some(self.seek(to, now));
        }
        if let Some(ms) = id
            .strip_prefix("sp-seek-")
            .and_then(|x| x.parse::<u64>().ok())
        {
            return Some(self.seek(ms, now));
        }
        match id {
            "sp-read" => {
                self.open = true;
                self.pos = 0;
                self.phase = Phase::Preparing;
                self.since = now;
                Some(format!(
                    "Reading with {voice} at {}, simulated, no audio plays",
                    self.speed_label()
                ))
            }
            "sp-play" => match self.phase_at(now) {
                Phase::Playing => {
                    self.set(Phase::Paused, now);
                    Some(format!("Paused at {}", self.times(now).0))
                }
                Phase::Done => {
                    self.pos = 0;
                    self.phase = Phase::Playing;
                    self.since = now;
                    Some("Playing again".into())
                }
                Phase::Failed => {
                    self.fail_next = false;
                    self.phase = Phase::Preparing;
                    self.since = now;
                    Some(format!("Preparing {voice}'s voice"))
                }
                Phase::Paused | Phase::Idle => {
                    self.set(Phase::Playing, now);
                    Some("Playing".into())
                }
                Phase::Preparing => None,
            },
            "sp-stop" => {
                self.set(Phase::Idle, now);
                self.open = false;
                self.menu = false;
                Some("Stopped reading".into())
            }
            "sp-speed" => {
                let p = self.pos_at(now);
                self.pos = p;
                self.since = now;
                self.speed = (self.speed + 1) % SPEEDS.len();
                Some(format!("Speed {}", self.speed_label()))
            }
            "sp-menu" => {
                self.menu = !self.menu;
                None
            }
            "sp-up" | "sp-down" => {
                let r = if id == "sp-up" { "up" } else { "down" };
                self.rating = if self.rating == Some(r) {
                    None
                } else {
                    Some(r)
                };
                Some(
                    if r == "up" {
                        "Marked as a good response"
                    } else {
                        "Marked as a bad response"
                    }
                    .into(),
                )
            }
            "sp-copy" => Some("Copied the reply".into()),
            _ => None,
        }
    }

    /// Keys on the progress bar: ←/→ two seconds, PageUp/PageDown ten, Home/End, Space or K
    /// play / pause. In the voice menu (`menu`): ↑/↓ move, Enter picks, Escape closes.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        if part == "menu" {
            return match key {
                "ArrowDown" | "ArrowUp" => {
                    let i = super::rove(self.voice, VOICES.len(), key);
                    self.press(&format!("sp-voice-{i}"), now)
                }
                "Escape" => {
                    self.menu = false;
                    None
                }
                _ => None,
            };
        }
        let p = self.pos_at(now) as i64;
        let to = match key {
            "ArrowLeft" => p - 2_000,
            "ArrowRight" => p + 2_000,
            "PageDown" => p - 10_000,
            "PageUp" => p + 10_000,
            "Home" => 0,
            "End" => self.total as i64,
            "Space" | "k" | "K" => return self.press("sp-play", now),
            _ => return None,
        };
        self.press(&format!("sp-seek-{}", to.clamp(0, self.total as i64)), now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_reads_seeks_switches_and_fails() {
        let mut s = Speech::new(true, false, 0);
        assert!(s.open);
        assert_eq!(s.phase_at(100), Phase::Preparing);
        assert_eq!(s.phase_at(PREP_MS + 10), Phase::Playing);
        let w = s.word_at(PREP_MS + 2_000);
        assert!(w > 0);
        s.press("sp-voice-1", PREP_MS + 2_000);
        assert_eq!(s.phase, Phase::Preparing);
        assert_eq!(s.pos, s.starts[w]);
        s.press("sp-speed", 10_000);
        assert_eq!(s.speed_label(), "1.5×");
        s.press("sp-play", 10_000);
        assert_eq!(s.phase_at(10_000), Phase::Paused);
        s.key("bar", "End", 10_000);
        assert_eq!(s.pos_at(10_000), s.total);
        s.press("sp-play", 10_000);
        assert_eq!(s.phase_at(10_001), Phase::Done);
        s.press("sp-play", 10_001);
        assert_eq!(s.pos_at(10_001), 0);
        s.press("sp-word-5", 10_001);
        assert_eq!(s.word_at(10_001), 5);
        s.press("sp-stop", 10_002);
        assert!(!s.open);
        let mut f = Speech::new(true, true, 0);
        assert_eq!(f.phase_at(1_000), Phase::Failed);
        f.press("sp-play", 1_000);
        assert_eq!(f.phase_at(1_000 + PREP_MS + 1), Phase::Playing);
    }
}
