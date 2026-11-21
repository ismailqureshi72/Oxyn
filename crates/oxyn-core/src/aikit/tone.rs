//! Tone tuner: a support reply retuned by two three-step sliders (formality, detail) and voice
//! chips. Each change waits a short debounce, then a rewrite lands: changed sentences swap in
//! place, words new against the original can be marked, and Copy takes the current version.

use super::{norm_key, nth};

/// Who the reply is for: name and subject.
pub const RECIPIENT: (&str, &str) = ("Jonah Reyes", "Re: Charged twice in October");
/// The model named while rewriting, and its mark.
pub const MODEL: (&str, &str) = ("Claude Sonnet 4.5", "claude");
/// Formality levels.
pub const FORMALITY: [&str; 3] = ["Casual", "Neutral", "Formal"];
/// Detail levels.
pub const DETAIL: [&str; 3] = ["Brief", "Standard", "Detailed"];
/// Voices.
pub const VOICES: [&str; 4] = ["Neutral", "Friendly", "Confident", "Empathetic"];
/// Voice chips: voice index, and the formality and detail each chip starts from.
pub const PRESETS: [(usize, usize, usize); 3] = [(1, 0, 1), (2, 1, 0), (3, 1, 2)];
/// Quiet time before a rewrite starts, ms.
pub const DEBOUNCE_MS: u64 = 220;
/// A rewrite takes this long, ms.
pub const REWRITE_MS: u64 = 650;
/// The self-playing tour moves every this many ms.
pub const TOUR_MS: u64 = 3_400;

/// One tone: formality, detail, voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tone {
    /// 0 casual, 1 neutral, 2 formal.
    pub formality: usize,
    /// 0 brief, 1 standard, 2 detailed.
    pub detail: usize,
    /// Index in [`VOICES`].
    pub voice: usize,
}

/// The original draft's tone.
pub const ORIGINAL: Tone = Tone {
    formality: 1,
    detail: 1,
    voice: 0,
};

/// The reply in `t`, as paragraphs of sentences.
pub fn compose(t: Tone) -> Vec<Vec<&'static str>> {
    let f = t.formality.min(2);
    let greet = match (t.voice, f) {
        (1, 0) => "Hey Jonah!",
        (1, 1) => "Hi Jonah!",
        (_, 2) => "Dear Jonah,",
        _ => "Hi Jonah,",
    };
    let open = match (t.voice, f) {
        (0, 0) => "Thanks for flagging the double charge.",
        (0, 1) => "Thanks for letting us know about the double charge.",
        (0, _) => "Thank you for letting us know about the duplicate charge.",
        (1, 0) => "Thanks so much for flagging this, and sorry for the hassle!",
        (1, 1) => "Thanks for flagging this, and sorry for the trouble!",
        (1, _) => "Thank you for raising this, and please accept our apologies for the trouble.",
        (2, 0) => "We found the double charge and it's fixed.",
        (2, 1) => "We found the duplicate charge and have already fixed it.",
        (2, _) => "We have found the duplicate charge and corrected it.",
        (_, 0) => "Getting charged twice is stressful, and I'm sorry it happened.",
        (_, 1) => "I know how frustrating a double charge is, and I'm sorry it happened.",
        _ => "I understand how worrying a duplicate charge can be, and I sincerely apologise.",
    };
    let refund = [
        "We've sent the extra $49 back to your card.",
        "We've refunded the extra $49 to your Visa ending in 4242.",
        "A refund of $49 has been issued to your Visa ending in 4242.",
    ][f];
    let when = [
        "It should land in 3 to 5 business days.",
        "It should appear within 3 to 5 business days.",
        "Please allow 3 to 5 business days for it to appear.",
    ][f];
    let why = [
        "The double charge came from a retry during a short outage at our payment provider, and we've added a check so it can't repeat.",
        "The duplicate came from an automatic retry during a brief payment provider outage, and we've added a safeguard so it cannot happen again.",
        "The duplicate resulted from an automatic retry during a brief outage at our payment provider; we have since added a safeguard to prevent a recurrence.",
    ][f];
    let next = [
        "Your next invoice on Dec 1 stays the same.",
        "Your next invoice on December 1 is unchanged.",
        "Your next invoice, due on December 1, remains unchanged.",
    ][f];
    let close = match (t.voice, f) {
        (1, _) => "Have a lovely week!",
        (2, 0) => "You're all set.",
        (2, _) => "You're all set, and there's nothing else you need to do.",
        (3, _) => "If anything else looks off, reply here and I'll look into it myself.",
        (_, 0) => "Shout if anything looks off.",
        (_, 1) => "Let me know if you have any other questions.",
        _ => "Please do not hesitate to contact us with any further questions.",
    };
    let sign = [
        "Cheers, Priya",
        "Best, Priya",
        "Kind regards, Priya Nair, Billing",
    ][f];
    let mut body = vec![open, refund];
    if t.detail >= 1 {
        body.push(when);
    }
    if t.detail >= 2 {
        body.push(why);
        body.push(next);
    }
    vec![vec![greet], body, vec![close], vec![sign]]
}

/// Lower-cased words of a sentence, without punctuation.
fn words(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '$' && c != '\'')
                .to_lowercase()
        })
        .collect()
}

/// Each word of `s` with whether it is new against the original draft.
pub fn diff(s: &str) -> Vec<(String, bool)> {
    let orig: std::collections::HashSet<String> = compose(ORIGINAL)
        .concat()
        .iter()
        .flat_map(|x| words(x))
        .collect();
    s.split_whitespace()
        .zip(words(s))
        .map(|(w, k)| (w.to_string(), !k.is_empty() && !orig.contains(&k)))
        .collect()
}

/// Words that differ from the original in the whole reply.
pub fn changed_words(t: Tone) -> usize {
    compose(t)
        .concat()
        .iter()
        .map(|s| diff(s).iter().filter(|w| w.1).count())
        .sum()
}

/// The reply as plain text.
pub fn plain(t: Tone) -> String {
    compose(t)
        .iter()
        .map(|p| p.join(" "))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct ToneTuner {
    /// The tone the controls show.
    pub want: Tone,
    /// The tone of the text on screen before the last change.
    pub shown: Tone,
    /// When the controls last changed.
    pub changed: u64,
    /// Mark new words.
    pub changes: bool,
    /// Tour the presets until someone touches the card.
    pub tour: Option<u64>,
    /// Copied at.
    pub copied: Option<u64>,
    /// Rewrites fail (the error scenario).
    pub fail: bool,
}

impl ToneTuner {
    /// A card at `t` (touring from `now` when `tour`).
    pub fn new(t: Tone, tour: bool, changes: bool, fail: bool, now: u64) -> Self {
        Self {
            want: t,
            shown: t,
            changed: 0,
            changes,
            tour: tour.then_some(now),
            copied: None,
            fail,
        }
    }

    /// The tour's tone and when it changed, at `now`.
    fn toured(&self, now: u64) -> Option<(Tone, Tone, u64)> {
        let t0 = self.tour?;
        let k = now.saturating_sub(t0) / TOUR_MS;
        let at = |i: u64| match i % 4 {
            0 => ORIGINAL,
            j => {
                let p = PRESETS[j as usize - 1];
                Tone {
                    voice: p.0,
                    formality: p.1,
                    detail: p.2,
                }
            }
        };
        let prev = if k == 0 { ORIGINAL } else { at(k - 1) };
        Some((at(k), prev, t0 + k * TOUR_MS))
    }

    /// The tone on the controls.
    pub fn controls(&self, now: u64) -> Tone {
        self.toured(now).map(|t| t.0).unwrap_or(self.want)
    }

    /// `idle`, `waiting` (debounce), `rewriting`, `error`.
    pub fn phase(&self, now: u64) -> &'static str {
        let (want, shown, at) = match self.toured(now) {
            Some((w, p, at)) => (w, p, if at == self.tour.unwrap_or(0) { 0 } else { at }),
            None => (self.want, self.shown, self.changed),
        };
        if want == shown || at == 0 {
            return "idle";
        }
        if now < at + DEBOUNCE_MS {
            "waiting"
        } else if now < at + DEBOUNCE_MS + REWRITE_MS {
            "rewriting"
        } else if self.fail && self.tour.is_none() {
            "error"
        } else {
            "idle"
        }
    }

    /// The tone of the text on screen.
    pub fn text_tone(&self, now: u64) -> Tone {
        match self.toured(now) {
            Some((w, p, at)) => {
                if at > self.tour.unwrap_or(0) && now < at + DEBOUNCE_MS + REWRITE_MS {
                    p
                } else {
                    w
                }
            }
            None => {
                if self.want != self.shown
                    && (now < self.changed + DEBOUNCE_MS + REWRITE_MS || self.fail)
                {
                    self.shown
                } else {
                    self.want
                }
            }
        }
    }

    /// The status line.
    pub fn status(&self, now: u64) -> String {
        let t = self.text_tone(now);
        match self.phase(now) {
            "waiting" | "rewriting" => format!("Rewriting with {}", MODEL.0),
            "error" => "Couldn't rewrite. The last version is kept.".into(),
            _ if t == ORIGINAL => "Original draft".into(),
            _ => format!(
                "Rewritten · {} · {} · {}",
                VOICES[t.voice], FORMALITY[t.formality], DETAIL[t.detail]
            ),
        }
    }

    fn touch(&mut self, now: u64) {
        if self.tour.is_some() {
            let t = self.controls(now);
            self.want = t;
            self.shown = t;
            self.tour = None;
        }
    }

    fn change(&mut self, t: Tone, now: u64) -> Option<String> {
        self.touch(now);
        if t == self.want {
            return None;
        }
        self.shown = self.text_tone(now);
        self.want = t;
        self.changed = now;
        Some(format!(
            "{} · {} · {}",
            VOICES[t.voice], FORMALITY[t.formality], DETAIL[t.detail]
        ))
    }

    /// Presses: `tt-voice-<i>` (preset chip; again returns to neutral), `tt-f-<i>`, `tt-d-<i>`,
    /// `tt-changes`, `tt-copy`, `tt-retry`, `tt-touch` (stops the tour).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        let cur = self.controls(now);
        if let Some(i) = nth(id, "tt-voice-") {
            let p = PRESETS[i.min(2)];
            let t = if cur.voice == p.0 {
                Tone { voice: 0, ..cur }
            } else {
                Tone {
                    voice: p.0,
                    formality: p.1,
                    detail: p.2,
                }
            };
            return self.change(t, now);
        }
        if let Some(i) = nth(id, "tt-f-") {
            return self.change(
                Tone {
                    formality: i.min(2),
                    ..cur
                },
                now,
            );
        }
        if let Some(i) = nth(id, "tt-d-") {
            return self.change(
                Tone {
                    detail: i.min(2),
                    ..cur
                },
                now,
            );
        }
        match id {
            "tt-changes" => {
                self.touch(now);
                self.changes = !self.changes;
                Some(
                    if self.changes {
                        "Changes marked"
                    } else {
                        "Changes hidden"
                    }
                    .into(),
                )
            }
            "tt-copy" => {
                self.touch(now);
                self.copied = Some(now);
                Some("Copied the current version".into())
            }
            "tt-retry" if self.phase(now) == "error" => {
                self.fail = false;
                self.changed = now;
                Some(format!("Rewriting with {}", MODEL.0))
            }
            "tt-touch" => {
                self.touch(now);
                None
            }
            _ => None,
        }
    }

    /// Keys on a slider (`f` or `d`): Left, Right, Home, End.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        let cur = self.controls(now);
        let v = if part == "f" {
            cur.formality
        } else {
            cur.detail
        };
        let n = match key {
            "ArrowLeft" | "ArrowDown" => v.saturating_sub(1),
            "ArrowRight" | "ArrowUp" => (v + 1).min(2),
            "Home" => 0,
            "End" => 2,
            _ => return None,
        };
        self.press(&format!("tt-{part}-{n}"), now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunes_debounces_and_marks() {
        let mut t = ToneTuner::new(ORIGINAL, false, true, false, 0);
        assert_eq!(changed_words(ORIGINAL), 0);
        assert_eq!(t.status(0), "Original draft");
        t.press("tt-voice-0", 1_000);
        assert_eq!(t.phase(1_100), "waiting");
        assert_eq!(t.phase(1_400), "rewriting");
        assert_eq!(t.text_tone(1_400), ORIGINAL);
        assert_eq!(t.text_tone(2_000).voice, 1);
        assert!(changed_words(t.text_tone(2_000)) > 3);
        t.press("tt-voice-0", 3_000);
        assert_eq!(t.controls(3_000).voice, 0);
        t.key("d", "End", 4_000);
        assert_eq!(t.controls(4_000).detail, 2);
        assert!(plain(t.controls(4_000)).contains("Dec"));
        let mut tour = ToneTuner::new(ORIGINAL, true, true, false, 0);
        assert_eq!(tour.controls(TOUR_MS + 1).voice, 1);
        assert_eq!(tour.phase(TOUR_MS + 300), "rewriting");
        tour.press("tt-touch", TOUR_MS * 2 + 2_000);
        assert!(tour.tour.is_none());
        assert_eq!(tour.controls(99_999).voice, 2);
        let mut f = ToneTuner::new(ORIGINAL, false, false, true, 0);
        f.press("tt-f-2", 100);
        assert_eq!(f.phase(5_000), "error");
        assert_eq!(f.text_tone(5_000), ORIGINAL);
        f.press("tt-retry", 5_000);
        assert_eq!(f.text_tone(9_000).formality, 2);
    }
}
