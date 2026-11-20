//! PII redaction: personal data is redacted before a prompt reaches the model. Scan marks each
//! name, email, phone number and card with its type, Redact swaps them for placeholders like
//! `[EMAIL_1]`, the safe prompt is sent, and the reply comes back with the placeholders restored
//! for the reader (or kept, with Show original off).

use super::{nth, words_at};

/// Scan highlights one entity every this many ms.
pub const SCAN_STEP_MS: u64 = 120;
/// The model thinks for this long before the reply streams, ms.
pub const THINK_MS: u64 = 700;

/// The sample message.
pub const SAMPLE: &str = "Hi, this is Ines Varga. Please change my billing email to ines.varga@fernmail.com and call me on +1 415 555 0142 if the card 4242 4242 4242 4242 is declined again. My manager Teo Brandt approved the change.";

/// Names the detector knows (fictional; a real detector would use a model).
pub const NAMES: [&str; 4] = ["Ines Varga", "Teo Brandt", "Mara Ilves", "Sam Okoro"];

/// Entity types: id, label, placeholder stem, hue.
pub const TYPES: [(&str, &str, &str, &str); 4] = [
    ("name", "Name", "NAME", "blue"),
    ("email", "Email", "EMAIL", "violet"),
    ("phone", "Phone", "PHONE", "teal"),
    ("card", "Card", "CARD", "amber"),
];

/// A found entity: byte range, type, number within its type (1-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    /// Start byte.
    pub start: usize,
    /// End byte.
    pub end: usize,
    /// Type index into [`TYPES`].
    pub kind: usize,
    /// Number within its type.
    pub n: usize,
}

/// Finds names, emails, phone numbers and card numbers in `text`, in reading order.
pub fn detect(text: &str) -> Vec<Entity> {
    let mut found: Vec<(usize, usize, usize)> = Vec::new();
    for name in NAMES {
        let mut from = 0;
        while let Some(i) = text[from..].find(name) {
            found.push((from + i, from + i + name.len(), 0));
            from += i + name.len();
        }
    }
    // Emails: a whitespace-delimited token with @ and a dot after it.
    let mut i = 0;
    for tok in text.split(' ') {
        let t = tok.trim_end_matches(['.', ',', ';', ')']);
        if let Some(at) = t.find('@')
            && t[at..].contains('.')
            && at > 0
        {
            found.push((i, i + t.len(), 1));
        }
        i += tok.len() + 1;
    }
    // Digit runs (digits, spaces, +, -): 13–19 digits is a card, 10–12 a phone.
    let b = text.as_bytes();
    let mut k = 0;
    while k < b.len() {
        if b[k].is_ascii_digit() || (b[k] == b'+' && k + 1 < b.len() && b[k + 1].is_ascii_digit()) {
            let s = k;
            let mut e = k;
            let mut digits = 0;
            let mut j = k;
            while j < b.len()
                && (b[j].is_ascii_digit() || b[j] == b' ' || b[j] == b'-' || b[j] == b'+')
            {
                if b[j].is_ascii_digit() {
                    digits += 1;
                    e = j + 1;
                }
                j += 1;
            }
            if (13..=19).contains(&digits) {
                found.push((s, e, 3));
            } else if (10..=12).contains(&digits) {
                found.push((s, e, 2));
            }
            k = j.max(k + 1);
        } else {
            k += 1;
        }
    }
    found.sort();
    let mut count = [0usize; 4];
    found
        .into_iter()
        .map(|(start, end, kind)| {
            count[kind] += 1;
            Entity {
                start,
                end,
                kind,
                n: count[kind],
            }
        })
        .collect()
}

/// The placeholder of an entity (`[EMAIL_1]`).
pub fn placeholder(e: &Entity) -> String {
    format!("[{}_{}]", TYPES[e.kind].2, e.n)
}

/// The phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Editing the message.
    Edit,
    /// Scanning (entities light up one by one).
    Scanning,
    /// Found, not yet redacted.
    Found,
    /// Placeholders in.
    Redacted,
    /// Sent; the model replies.
    Replying,
    /// Reply finished.
    Done,
}

/// The live state of the redaction card.
#[derive(Clone, Debug, PartialEq)]
pub struct Pii {
    /// The message.
    pub text: String,
    /// Phase when last changed.
    pub phase: Phase,
    /// When the phase began.
    pub since: u64,
    /// Restore placeholders in the reply for the reader.
    pub show_original: bool,
    /// Types to redact (Oxyn category toggles).
    pub on: [bool; 4],
}

impl Pii {
    /// The sample; `auto` scans at `now`.
    pub fn new(auto: bool, now: u64) -> Self {
        Self {
            text: SAMPLE.to_string(),
            phase: if auto { Phase::Scanning } else { Phase::Edit },
            since: now,
            show_original: true,
            on: [true; 4],
        }
    }

    /// Entities of enabled types.
    pub fn entities(&self) -> Vec<Entity> {
        detect(&self.text)
            .into_iter()
            .filter(|e| self.on[e.kind])
            .collect()
    }

    /// Phase at `now` (scanning settles into Found, replying into Done).
    pub fn phase_at(&self, now: u64) -> Phase {
        let n = self.entities().len() as u64;
        match self.phase {
            Phase::Scanning if now >= self.since + (n + 2) * SCAN_STEP_MS => Phase::Found,
            Phase::Replying
                if now >= self.since + THINK_MS
                    && words_at(&self.reply_safe(), now - self.since - THINK_MS, 4)
                        >= self.reply_safe().split_whitespace().count() =>
            {
                Phase::Done
            }
            p => p,
        }
    }

    /// Entities lit so far while scanning.
    pub fn lit(&self, now: u64) -> usize {
        match self.phase_at(now) {
            Phase::Edit => 0,
            Phase::Scanning => ((now.saturating_sub(self.since)) / SCAN_STEP_MS) as usize,
            _ => usize::MAX,
        }
    }

    /// Counts per type.
    pub fn counts(&self) -> [usize; 4] {
        let mut c = [0; 4];
        for e in self.entities() {
            c[e.kind] += 1;
        }
        c
    }

    /// Segments of the message: text and the entity it is, if any.
    pub fn segments(&self) -> Vec<(String, Option<Entity>)> {
        let mut v = Vec::new();
        let mut at = 0;
        for e in self.entities() {
            if e.start > at {
                v.push((self.text[at..e.start].to_string(), None));
            }
            v.push((self.text[e.start..e.end].to_string(), Some(e)));
            at = e.end;
        }
        if at < self.text.len() {
            v.push((self.text[at..].to_string(), None));
        }
        v
    }

    /// The safe prompt with placeholders.
    pub fn safe(&self) -> String {
        self.segments()
            .into_iter()
            .map(|(t, e)| e.map(|e| placeholder(&e)).unwrap_or(t))
            .collect()
    }

    /// The model's reply as it arrives (placeholders only).
    pub fn reply_safe(&self) -> String {
        let has = |k: usize| self.entities().iter().any(|e| e.kind == k);
        let who = if has(0) { "[NAME_1]" } else { "there" };
        let mut s = format!("Thanks {who}. I've drafted the change:");
        if has(1) {
            s.push_str(" the billing email becomes [EMAIL_1],");
        }
        if has(2) {
            s.push_str(" we'll call [PHONE_1] if a payment fails,");
        }
        if has(3) {
            s.push_str(" and [CARD_1] stays on file.");
        } else {
            s.push_str(" and nothing else changes.");
        }
        if self.entities().iter().any(|e| e.kind == 0 && e.n == 2) {
            s.push_str(" I noted that [NAME_2] approved it.");
        }
        s
    }

    /// The reply with placeholders restored.
    pub fn reply_restored(&self) -> String {
        let mut s = self.reply_safe();
        for e in self.entities() {
            s = s.replace(&placeholder(&e), &self.text[e.start..e.end]);
        }
        s
    }

    /// Visible words of the reply.
    pub fn reply_words(&self, now: u64) -> usize {
        match self.phase_at(now) {
            Phase::Replying if now >= self.since + THINK_MS => {
                words_at(&self.reply_safe(), now - self.since - THINK_MS, 4)
            }
            Phase::Done => usize::MAX,
            _ => 0,
        }
    }

    /// Presses: `pii-scan`, `pii-redact`, `pii-send`, `pii-edit:<text>`, `pii-orig`,
    /// `pii-type-<i>` (toggle a category), `pii-reset`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(t) = id.strip_prefix("pii-edit:") {
            if self.phase_at(now) == Phase::Edit {
                self.text = t.to_string();
            }
            return None;
        }
        if let Some(i) = nth(id, "pii-type-") {
            let i = i.min(3);
            self.on[i] = !self.on[i];
            return Some(format!(
                "{} {}",
                TYPES[i].1,
                if self.on[i] { "redacted" } else { "kept" }
            ));
        }
        let ph = self.phase_at(now);
        match id {
            "pii-scan" if ph == Phase::Edit || ph == Phase::Done => {
                self.phase = Phase::Scanning;
                self.since = now;
                Some("Scanning for personal data".into())
            }
            "pii-redact" if ph == Phase::Found => {
                self.phase = Phase::Redacted;
                self.since = now;
                let c = self.counts();
                Some(format!(
                    "Redacted {} items: {} names, {} emails, {} phones, {} cards",
                    c.iter().sum::<usize>(),
                    c[0],
                    c[1],
                    c[2],
                    c[3]
                ))
            }
            "pii-send" if ph == Phase::Redacted => {
                self.phase = Phase::Replying;
                self.since = now;
                Some("Sent the safe prompt, simulated".into())
            }
            "pii-orig" => {
                self.show_original = !self.show_original;
                Some(
                    if self.show_original {
                        "Showing original values"
                    } else {
                        "Showing placeholders"
                    }
                    .into(),
                )
            }
            "pii-reset" => {
                self.phase = Phase::Edit;
                self.since = now;
                Some("Editing the message".into())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pii_detects_redacts_and_restores() {
        let e = detect(SAMPLE);
        let kinds: Vec<usize> = e.iter().map(|x| x.kind).collect();
        assert_eq!(kinds, vec![0, 1, 2, 3, 0]);
        let mut p = Pii::new(true, 0);
        assert_eq!(p.phase_at(100), Phase::Scanning);
        assert_eq!(p.lit(250), 2);
        assert_eq!(p.phase_at(2_000), Phase::Found);
        p.press("pii-redact", 2_000);
        assert!(p.safe().contains("[EMAIL_1]") && !p.safe().contains('@'));
        assert!(p.safe().contains("[NAME_2]"));
        p.press("pii-send", 2_000);
        assert_eq!(p.reply_words(2_100), 0);
        assert!(p.reply_words(2_800) > 0);
        assert_eq!(p.phase_at(10_000), Phase::Done);
        assert!(p.reply_restored().contains("ines.varga@fernmail.com"));
        p.press("pii-type-3", 10_000);
        assert_eq!(p.counts()[3], 0);
        p.press("pii-scan", 10_000);
        assert_eq!(p.phase, Phase::Scanning);
        let mut q = Pii::new(false, 0);
        q.press("pii-edit:Mail sam@corvel.dev today", 0);
        assert_eq!(q.entities().len(), 1);
    }
}
