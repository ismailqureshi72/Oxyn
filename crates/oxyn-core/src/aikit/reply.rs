//! Reply draft: a received email with an AI reply under it. Quick replies steer a fresh draft,
//! tone and length rewrite it with one crossfade, the finished draft can be edited, and Insert
//! (or Cmd/Ctrl+Enter) hands it over.

use super::{norm_key, nth, rove, words_at};

/// Thinking before a draft streams, ms.
pub const THINK_MS: u64 = 500;

/// The received email: sender, address, subject, body, received.
pub const MESSAGE: (&str, &str, &str, &str, &str) = (
    "Mara Ilves",
    "mara@fernhill.studio",
    "Moving our kickoff to next week?",
    "Hi! Our designer is out sick until Monday, so Thursday's kickoff would be a bit thin on our side.\n\nCould we move it to Tuesday or Wednesday next week? Happy to keep the same time.",
    "Today 09:41",
);

/// Quick replies.
pub const SUGGESTIONS: [&str; 3] = ["Tuesday works", "Offer Wednesday", "Keep Thursday"];

/// Tones.
pub const TONES: [&str; 3] = ["Friendly", "Concise", "Formal"];

/// Lengths.
pub const LENGTHS: [&str; 3] = ["Short", "Medium", "Long"];

/// Writes the draft for a quick reply, tone and length (the local simulation).
pub fn compose(sug: usize, tone: usize, len: usize) -> String {
    let open = ["Hi Mara,", "Hi Mara,", "Dear Mara,"][tone];
    let core = match (sug, tone) {
        (0, 1) => "Tuesday works. Same time.",
        (0, 2) => "Tuesday next week suits us well, and we are glad to keep the same time.",
        (0, _) => "No problem at all, Tuesday works great for us and we can keep the same time.",
        (1, 1) => "Wednesday is better for us. Same time?",
        (1, 2) => "Would Wednesday next week at the same time be convenient for you?",
        (1, _) => {
            "Sorry to hear that! Wednesday would suit us best, could we do the same time then?"
        }
        (_, 1) => "We'd like to keep Thursday and catch your designer up after.",
        (_, 2) => {
            "If possible we would prefer to keep Thursday and brief your designer separately."
        }
        _ => {
            "Hope they feel better soon! Could we keep Thursday and catch your designer up afterwards?"
        }
    };
    let mid = [
        "",
        " I'll send an updated invite with the agenda attached.",
        " I'll send an updated invite with the agenda attached, and move the design review to the following day so nobody misses it.",
    ][len];
    let extra = if len == 2 {
        [
            " Let me know if anything else changes on your side.",
            " Tell me if anything changes.",
            " Please let me know should anything else change.",
        ][tone]
    } else {
        ""
    };
    let close = ["Best,\nSam", "Sam", "Kind regards,\nSam"][tone];
    format!("{open}\n\n{core}{mid}{extra}\n\n{close}")
}

/// The draft's phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// No draft yet (Draft reply button).
    Empty,
    /// Thinking before the first words.
    Thinking,
    /// Streaming words.
    Streaming,
    /// Rewriting (the old draft dims).
    Rewriting,
    /// Finished and editable.
    Ready,
    /// Inserted.
    Inserted,
}

/// The live state of a reply draft.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    /// Quick reply steering the draft.
    pub sug: usize,
    /// Tone.
    pub tone: usize,
    /// Length.
    pub len: usize,
    /// The draft being written.
    pub draft: String,
    /// The previous draft (shown dimmed during a rewrite).
    pub prev: String,
    /// When the draft started (or the rewrite).
    pub since: Option<u64>,
    /// The draft is a rewrite (crossfade, no streaming).
    pub rewrite: bool,
    /// The reader edited the finished draft.
    pub edited: Option<String>,
    /// Inserted.
    pub inserted: bool,
    /// Past drafts (Oxyn carousel extra).
    pub history: Vec<String>,
}

impl Reply {
    /// The card; `auto` starts the first draft at `now`.
    pub fn new(auto: bool, now: u64) -> Self {
        let mut r = Self {
            sug: 0,
            tone: 0,
            len: 1,
            draft: compose(0, 0, 1),
            prev: String::new(),
            since: None,
            rewrite: false,
            edited: None,
            inserted: false,
            history: vec![],
        };
        if auto {
            r.since = Some(now);
        }
        r
    }

    /// Phase at `now`.
    pub fn phase(&self, now: u64) -> Phase {
        if self.inserted {
            return Phase::Inserted;
        }
        let Some(s) = self.since else {
            return Phase::Empty;
        };
        let el = now.saturating_sub(s);
        if self.rewrite {
            return if el < THINK_MS {
                Phase::Rewriting
            } else {
                Phase::Ready
            };
        }
        if el < THINK_MS {
            Phase::Thinking
        } else if words_at(&self.draft, el - THINK_MS, 5) < self.draft.split_whitespace().count() {
            Phase::Streaming
        } else {
            Phase::Ready
        }
    }

    /// The visible draft text at `now` (whole words, line breaks kept).
    pub fn visible(&self, now: u64) -> String {
        if let Some(e) = &self.edited {
            return e.clone();
        }
        match self.phase(now) {
            Phase::Empty | Phase::Thinking => String::new(),
            Phase::Rewriting => self.prev.clone(),
            Phase::Streaming => {
                let n = words_at(&self.draft, now - self.since.unwrap_or(0) - THINK_MS, 5);
                let mut out = String::new();
                let mut k = 0;
                for line in self.draft.split('\n') {
                    if k >= n {
                        break;
                    }
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    let ws: Vec<&str> = line.split_whitespace().collect();
                    let take = ws.len().min(n - k);
                    out.push_str(&ws[..take].join(" "));
                    k += take;
                }
                out
            }
            _ => self.draft.clone(),
        }
    }

    fn redraft(&mut self, rewrite: bool, now: u64) {
        let ph = self.phase(now);
        let was_ready = ph == Phase::Ready;
        if was_ready {
            self.prev = self.edited.take().unwrap_or_else(|| self.draft.clone());
            self.history.push(self.prev.clone());
            if self.history.len() > 5 {
                self.history.remove(0);
            }
        }
        self.edited = None;
        self.draft = compose(self.sug, self.tone, self.len);
        self.rewrite = rewrite && was_ready;
        self.since = Some(now);
        self.inserted = false;
    }

    /// Presses: `rd-draft`, `rd-sug-<i>`, `rd-tone-<i>`, `rd-len-<i>`, `rd-regen`, `rd-insert`,
    /// `rd-edit:<text>`, `rd-hist-<i>` (restore a past draft).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(t) = id.strip_prefix("rd-edit:") {
            if self.phase(now) == Phase::Ready {
                self.edited = Some(t.to_string());
            }
            return None;
        }
        if let Some(i) = nth(id, "rd-sug-") {
            self.sug = i.min(SUGGESTIONS.len() - 1);
            self.redraft(false, now);
            return Some(format!("Drafting: {}", SUGGESTIONS[self.sug]));
        }
        if let Some(i) = nth(id, "rd-tone-") {
            let i = i.min(TONES.len() - 1);
            if i == self.tone {
                return None;
            }
            self.tone = i;
            self.redraft(true, now);
            return Some(format!("Rewriting, {}", TONES[i].to_lowercase()));
        }
        if let Some(i) = nth(id, "rd-len-") {
            let i = i.min(LENGTHS.len() - 1);
            if i == self.len {
                return None;
            }
            self.len = i;
            self.redraft(true, now);
            return Some(format!("Rewriting, {}", LENGTHS[i].to_lowercase()));
        }
        if let Some(i) = nth(id, "rd-hist-") {
            let h = self.history.get(i)?.clone();
            self.edited = Some(h);
            return Some(format!("Restored draft {}", i + 1));
        }
        match id {
            "rd-draft" | "rd-regen" => {
                self.redraft(false, now);
                Some("Drafting a reply".into())
            }
            "rd-insert" => {
                if self.phase(now) != Phase::Ready {
                    return None;
                }
                self.inserted = true;
                Some("Inserted into the reply, simulated, nothing was sent".into())
            }
            _ => None,
        }
    }

    /// Keys in the tone (`tone`) or length (`len`) groups: arrows, Home, End; Cmd/Ctrl+Enter
    /// (`insert`) inserts.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        match part {
            "tone" => {
                let i = rove(self.tone, TONES.len(), key);
                self.press(&format!("rd-tone-{i}"), now)
            }
            "len" => {
                let i = rove(self.len, LENGTHS.len(), key);
                self.press(&format!("rd-len-{i}"), now)
            }
            "insert" if key == "Enter" => self.press("rd-insert", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reply_streams_rewrites_and_inserts() {
        let mut r = Reply::new(true, 0);
        assert_eq!(r.phase(100), Phase::Thinking);
        assert_eq!(r.phase(600), Phase::Streaming);
        assert!(!r.visible(600).is_empty());
        assert_eq!(r.phase(5_000), Phase::Ready);
        assert!(r.press("rd-insert", 600).is_none());
        r.key("tone", "ArrowRight", 5_000);
        assert_eq!(r.tone, 1);
        assert_eq!(r.phase(5_100), Phase::Rewriting);
        assert_eq!(r.phase(5_600), Phase::Ready);
        assert_eq!(r.history.len(), 1);
        r.press("rd-edit:Hi Mara, Tuesday it is.", 5_600);
        assert_eq!(r.visible(5_600), "Hi Mara, Tuesday it is.");
        r.key("insert", "Enter", 5_600);
        assert_eq!(r.phase(5_600), Phase::Inserted);
        r.press("rd-sug-1", 6_000);
        assert_eq!(r.phase(6_100), Phase::Thinking);
        assert!(compose(2, 2, 2).contains("Kind regards"));
        let e = Reply::new(false, 0);
        assert_eq!(e.phase(0), Phase::Empty);
    }
}
