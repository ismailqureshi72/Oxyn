//! Smart form fill (`smart-form-fill.md`): paste an email signature or messy notes, the agent
//! reads it and fills a contact form field by field. Each value fades in with a confidence dot,
//! its source lights up in the pasted text, low-confidence fields ask for review and block Save
//! until checked or edited, and Undo restores the values from before the fill.
//!
//! Behaviour follows the Arc UI Smart form fill docs (rebuilt, no source used). The built-in
//! extractor is a deterministic heuristic standing in for the model (the host passes real results
//! through [`SmartFill::fill`]). Oxyn plus: undo per field, a confidence gate the user can move,
//! and a provenance list that names where each value came from.

use std::collections::BTreeMap;

use crate::ClockMs;

/// Field kinds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FillKind {
    /// Text.
    #[default]
    Text,
    /// Email.
    Email,
    /// Phone.
    Phone,
    /// Company (logo slot).
    Company,
    /// URL.
    Url,
    /// Address (spans the row).
    Address,
}

/// A form field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillField {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Kind.
    pub kind: FillKind,
}

impl FillField {
    /// A field.
    pub fn new(id: &str, label: &str, kind: FillKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
        }
    }
}

/// The default contact fields: name, title, company, email, phone, address.
pub fn contact_fields() -> Vec<FillField> {
    vec![
        FillField::new("name", "Name", FillKind::Text),
        FillField::new("title", "Title", FillKind::Text),
        FillField::new("company", "Company", FillKind::Company),
        FillField::new("email", "Email", FillKind::Email),
        FillField::new("phone", "Phone", FillKind::Phone),
        FillField::new("address", "Address", FillKind::Address),
    ]
}

/// One extracted value.
#[derive(Clone, Debug, PartialEq)]
pub struct Extracted {
    /// Value.
    pub value: String,
    /// 0..1.
    pub confidence: f32,
    /// Byte range of the source in the pasted text.
    pub source: Option<(usize, usize)>,
    /// Why it is uncertain.
    pub note: Option<String>,
}

const TITLES: [&str; 16] = [
    "engineer",
    "manager",
    "director",
    "founder",
    "head of",
    "lead",
    "designer",
    "vp",
    "ceo",
    "cto",
    "coo",
    "product",
    "partner",
    "analyst",
    "consultant",
    "president",
];

const STREET: [&str; 10] = [
    " st", " street", " ave", " avenue", " road", " rd", " blvd", " suite", " lane", " way",
];

fn line_spans(text: &str) -> Vec<(usize, usize, &str)> {
    let mut out = vec![];
    let mut start = 0;
    for l in text.split('\n') {
        let t = l.trim();
        if !t.is_empty() {
            let off = start + l.find(t).unwrap_or(0);
            out.push((off, off + t.len(), t));
        }
        start += l.len() + 1;
    }
    out
}

/// The built-in extractor: email, phone, website, name, title, company and address from pasted
/// text, with confidences and source spans.
pub fn extract(text: &str) -> BTreeMap<String, Extracted> {
    let mut out = BTreeMap::new();
    let lines = line_spans(text);
    // Email: a token with @ and a dot.
    let mut email_domain = None;
    for (s, _, l) in &lines {
        for tok in
            l.split(|c: char| c.is_whitespace() || c == '|' || c == '<' || c == '>' || c == ',')
        {
            let t = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.');
            if super::tags::is_email(t) {
                let off = s + l.find(t).unwrap_or(0);
                email_domain = t.split('@').nth(1).map(str::to_string);
                out.insert(
                    "email".into(),
                    Extracted {
                        value: t.to_lowercase(),
                        confidence: 0.98,
                        source: Some((off, off + t.len())),
                        note: None,
                    },
                );
                break;
            }
        }
        if out.contains_key("email") {
            break;
        }
    }
    // Phone: the longest run of phone characters with 7+ digits.
    for (s, _, l) in &lines {
        let mut best: Option<(usize, usize)> = None;
        let b = l.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'+' || b[i].is_ascii_digit() || b[i] == b'(' {
                let mut j = i;
                while j < b.len() && (b[j].is_ascii_digit() || b" +-().".contains(&b[j])) {
                    j += 1;
                }
                let digits = l[i..j].chars().filter(char::is_ascii_digit).count();
                if digits >= 7 && best.is_none_or(|(a, z)| j - i > z - a) {
                    best = Some((i, j));
                }
                i = j.max(i + 1);
            } else {
                i += 1;
            }
        }
        if let Some((a, z)) = best {
            let v = l[a..z].trim();
            let off = s + a + l[a..z].find(v).unwrap_or(0);
            out.insert(
                "phone".into(),
                Extracted {
                    value: v.into(),
                    confidence: if v.starts_with('+') { 0.95 } else { 0.86 },
                    source: Some((off, off + v.len())),
                    note: (!v.starts_with('+')).then(|| "No country code".into()),
                },
            );
            break;
        }
    }
    // Website.
    for (s, _, l) in &lines {
        if let Some(t) = l
            .split_whitespace()
            .find(|t| (t.starts_with("http") || t.starts_with("www.")) && !t.contains('@'))
        {
            let off = s + l.find(t).unwrap_or(0);
            out.insert(
                "website".into(),
                Extracted {
                    value: t.trim_end_matches('/').into(),
                    confidence: 0.92,
                    source: Some((off, off + t.len())),
                    note: None,
                },
            );
            break;
        }
    }
    let used = |r: (usize, usize), out: &BTreeMap<String, Extracted>| {
        out.values()
            .filter_map(|e| e.source)
            .any(|(a, b)| a < r.1 && r.0 < b)
    };
    // Title (and a company after "at" / "|" / "," on the same line).
    for (s, e, l) in &lines {
        let low = l.to_lowercase();
        if TITLES.iter().any(|k| low.contains(k)) && !l.contains('@') {
            let (title, company) = split_title_company(l);
            out.insert(
                "title".into(),
                Extracted {
                    value: title.into(),
                    confidence: 0.88,
                    source: Some((*s, s + title.len())),
                    note: None,
                },
            );
            if let Some(c) = company {
                let off = s + l.find(c).unwrap_or(0);
                out.insert(
                    "company".into(),
                    Extracted {
                        value: c.into(),
                        confidence: 0.9,
                        source: Some((off, off + c.len())),
                        note: None,
                    },
                );
            }
            let _ = e;
            break;
        }
    }
    // Address: a line with a number and a street word.
    for (s, e, l) in &lines {
        let low = format!(" {}", l.to_lowercase());
        if l.chars().any(|c| c.is_ascii_digit())
            && STREET.iter().any(|w| low.contains(w))
            && !used((*s, *e), &out)
        {
            out.insert(
                "address".into(),
                Extracted {
                    value: (*l).into(),
                    confidence: 0.74,
                    source: Some((*s, *e)),
                    note: Some("Check the street and unit".into()),
                },
            );
            break;
        }
    }
    // Name: the first line of 2–3 capitalised words with no digits or @.
    for (s, e, l) in &lines {
        let words: Vec<&str> = l.split_whitespace().collect();
        let looks = (2..=3).contains(&words.len())
            && words
                .iter()
                .all(|w| w.chars().next().is_some_and(char::is_uppercase))
            && !l
                .chars()
                .any(|c| c.is_ascii_digit() || c == '@' || c == '|');
        let low = l.to_lowercase();
        if looks && !TITLES.iter().any(|k| low.contains(k)) && !used((*s, *e), &out) {
            out.insert(
                "name".into(),
                Extracted {
                    value: (*l).into(),
                    confidence: 0.91,
                    source: Some((*s, *e)),
                    note: None,
                },
            );
            break;
        }
    }
    if !out.contains_key("name")
        && let Some(user) = out
            .get("email")
            .and_then(|e| e.value.split('@').next().map(str::to_string))
    {
        let guess: Vec<String> = user
            .split(['.', '_'])
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().chain(c).collect())
                    .unwrap_or_default()
            })
            .collect();
        if guess.len() >= 2 {
            out.insert(
                "name".into(),
                Extracted {
                    value: guess.join(" "),
                    confidence: 0.6,
                    source: out.get("email").and_then(|e| e.source),
                    note: Some("Guessed from the email address".into()),
                },
            );
        }
    }
    // Company from the email domain when nothing better.
    if !out.contains_key("company")
        && let Some(d) = email_domain
    {
        let generic = [
            "gmail.com",
            "outlook.com",
            "yahoo.com",
            "icloud.com",
            "hotmail.com",
        ];
        if !generic.contains(&d.as_str()) {
            let stem = d.split('.').next().unwrap_or(&d);
            let mut c = stem.chars();
            let name: String = c
                .next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default();
            out.insert(
                "company".into(),
                Extracted {
                    value: name,
                    confidence: 0.66,
                    source: out.get("email").and_then(|e| e.source),
                    note: Some("From the email domain".into()),
                },
            );
        }
    }
    out
}

fn split_title_company(l: &str) -> (&str, Option<&str>) {
    for sep in [" at ", " @ ", " | ", ", ", " · ", " - "] {
        if let Some((a, b)) = l.split_once(sep) {
            return (a.trim(), Some(b.trim()).filter(|b| !b.is_empty()));
        }
    }
    (l.trim(), None)
}

/// Phase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FillPhase {
    /// Waiting for a paste.
    #[default]
    Idle,
    /// The agent is reading (calm shimmer). Esc stops.
    Reading(ClockMs),
    /// Fields are filling 140 ms apart from this moment.
    Filling(ClockMs),
    /// Filled; review may be needed.
    Done,
    /// Saving.
    Saving,
    /// Saved.
    Saved,
    /// Save failed.
    Error(String),
}

/// A field's mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldMark {
    /// Nothing filled it.
    Plain,
    /// Filled with high confidence.
    Filled,
    /// Low confidence: needs review (aria-invalid until checked or edited).
    Review,
    /// Checked by the user.
    Checked,
    /// Edited by the user.
    Edited,
}

impl FieldMark {
    /// Word shown next to the dot (status is never colour alone).
    pub fn word(self) -> &'static str {
        match self {
            Self::Plain => "",
            Self::Filled => "",
            Self::Review => "Review",
            Self::Checked => "Checked",
            Self::Edited => "Edited",
        }
    }
}

/// Timings.
pub mod timing {
    /// Fields fill this far apart.
    pub const STAGGER_MS: u64 = 140;
    /// Each value fades in over this long.
    pub const FADE_MS: u64 = 200;
    /// The input breathes a tint for this long.
    pub const TINT_MS: u64 = 900;
    /// Simulated read time.
    pub const READ_MS: u64 = 900;
}

/// The smart form.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartFill {
    /// Fields.
    pub fields: Vec<FillField>,
    /// Values by id.
    pub values: BTreeMap<String, String>,
    /// Values before the last fill (Undo).
    pub before: Option<BTreeMap<String, String>>,
    /// The pasted text.
    pub text: String,
    /// The last results.
    pub results: BTreeMap<String, Extracted>,
    /// Marks by id.
    pub marks: BTreeMap<String, FieldMark>,
    /// Below this confidence a field asks for review.
    pub threshold: f32,
    /// Phase.
    pub phase: FillPhase,
    /// Field hovered or focused (its source lights up).
    pub hot: Option<String>,
    /// Live-region line.
    pub announce: String,
}

impl SmartFill {
    /// A form.
    pub fn new(fields: Vec<FillField>) -> Self {
        Self {
            fields,
            values: BTreeMap::new(),
            before: None,
            text: String::new(),
            results: BTreeMap::new(),
            marks: BTreeMap::new(),
            threshold: 0.8,
            phase: FillPhase::Idle,
            hot: None,
            announce: String::new(),
        }
    }
    /// Paste (or Cmd+Enter on typed text): starts reading.
    pub fn paste(&mut self, text: &str, now: ClockMs) {
        self.text = text.into();
        self.phase = FillPhase::Reading(now);
        self.announce = "Reading the pasted text…".into();
    }
    /// Esc during a read: stop, nothing changes.
    pub fn stop(&mut self) -> bool {
        if matches!(self.phase, FillPhase::Reading(_)) {
            self.phase = FillPhase::Idle;
            self.announce = "Stopped. Nothing changed.".into();
            return true;
        }
        false
    }
    /// The read resolved: fields fill in form order from `now`.
    pub fn fill(&mut self, results: BTreeMap<String, Extracted>, now: ClockMs) {
        self.before = Some(self.values.clone());
        for f in &self.fields {
            if let Some(r) = results.get(&f.id) {
                self.values.insert(f.id.clone(), r.value.clone());
                let m = if r.confidence < self.threshold {
                    FieldMark::Review
                } else {
                    FieldMark::Filled
                };
                self.marks.insert(f.id.clone(), m);
            }
        }
        self.results = results;
        self.phase = FillPhase::Filling(now);
        let filled = self.order().len();
        let review = self.needs_review().len();
        self.announce = if review > 0 {
            format!("Filled {filled} fields. {review} need review.")
        } else {
            format!("Filled {filled} fields.")
        };
    }
    /// Advances time: a simulated read resolves with the built-in extractor; filling ends.
    pub fn tick(&mut self, now: ClockMs) {
        match self.phase {
            FillPhase::Reading(t) if now.since(t).as_millis() >= u128::from(timing::READ_MS) => {
                let r = extract(&self.text);
                self.fill(r, now);
            }
            FillPhase::Filling(t) => {
                let n = self.order().len() as u64;
                if now.since(t).as_millis() >= u128::from(n * timing::STAGGER_MS + timing::TINT_MS)
                {
                    self.phase = FillPhase::Done;
                }
            }
            _ => {}
        }
    }
    /// Filled field ids in form order.
    pub fn order(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|f| self.results.contains_key(&f.id))
            .map(|f| f.id.clone())
            .collect()
    }
    /// How far field `id` has faded in (0..1) and its tint (0..1) at `now`.
    pub fn reveal(&self, id: &str, now: ClockMs, reduced: bool) -> (f32, f32) {
        let FillPhase::Filling(t) = self.phase else {
            return (1.0, 0.0);
        };
        if reduced {
            return (1.0, 0.0);
        }
        let Some(k) = self.order().iter().position(|x| x == id) else {
            return (1.0, 0.0);
        };
        let start = t.0 + k as u64 * timing::STAGGER_MS;
        if now.0 < start {
            return (0.0, 0.0);
        }
        let e = (now.0 - start) as f32;
        let fade = (e / timing::FADE_MS as f32).min(1.0);
        let tint = 1.0 - (e / timing::TINT_MS as f32).min(1.0);
        (fade, tint)
    }
    /// The field being filled at `now` (its source span lights up).
    pub fn filling_now(&self, now: ClockMs) -> Option<String> {
        let FillPhase::Filling(t) = self.phase else {
            return None;
        };
        let k = (now.since(t).as_millis() / u128::from(timing::STAGGER_MS)) as usize;
        self.order().get(k).cloned()
    }
    /// The source span to highlight: the hovered / focused field, else the one filling.
    pub fn highlight(&self, now: ClockMs) -> Option<(usize, usize)> {
        let id = self.hot.clone().or_else(|| self.filling_now(now))?;
        self.results.get(&id)?.source
    }
    /// Mark for a field.
    pub fn mark(&self, id: &str) -> FieldMark {
        self.marks.get(id).copied().unwrap_or(FieldMark::Plain)
    }
    /// Confidence as a percentage.
    pub fn confidence(&self, id: &str) -> Option<u8> {
        self.results
            .get(id)
            .map(|r| (r.confidence * 100.0).round() as u8)
    }
    /// Fields still needing review.
    pub fn needs_review(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter(|f| self.mark(&f.id) == FieldMark::Review)
            .map(|f| f.id.clone())
            .collect()
    }
    /// The user checks a field.
    pub fn check(&mut self, id: &str) {
        if self.mark(id) == FieldMark::Review {
            self.marks.insert(id.into(), FieldMark::Checked);
            self.announce = format!("{} checked", self.label(id));
        }
    }
    /// The user types in a field.
    pub fn edit(&mut self, id: &str, v: &str) {
        self.values.insert(id.into(), v.into());
        if self.results.contains_key(id) {
            self.marks.insert(id.into(), FieldMark::Edited);
        }
    }
    fn label(&self, id: &str) -> String {
        self.fields
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.label.clone())
            .unwrap_or_default()
    }
    /// Moves the confidence gate (Oxyn plus) and re-marks filled fields.
    pub fn set_threshold(&mut self, t: f32) {
        self.threshold = t.clamp(0.0, 1.0);
        for (id, r) in &self.results {
            match self.marks.get(id) {
                Some(FieldMark::Filled) | Some(FieldMark::Review) => {
                    let m = if r.confidence < self.threshold {
                        FieldMark::Review
                    } else {
                        FieldMark::Filled
                    };
                    self.marks.insert(id.clone(), m);
                }
                _ => {}
            }
        }
    }
    /// Undo: restores the values from before the last fill.
    pub fn undo(&mut self) -> bool {
        let Some(b) = self.before.take() else {
            return false;
        };
        self.values = b;
        self.marks.clear();
        self.results.clear();
        self.phase = FillPhase::Idle;
        self.announce = "Restored the values from before the fill".into();
        true
    }
    /// Undo one field (Oxyn plus).
    pub fn undo_field(&mut self, id: &str) {
        let old = self.before.as_ref().and_then(|b| b.get(id).cloned());
        match old {
            Some(v) => {
                self.values.insert(id.into(), v);
            }
            None => {
                self.values.remove(id);
            }
        }
        self.marks.remove(id);
        self.results.remove(id);
    }
    /// Save: blocked while a field needs review (returns the first one to focus).
    pub fn save(&mut self) -> Result<(), String> {
        if let Some(first) = self.needs_review().into_iter().next() {
            self.announce = format!("Check {} before saving", self.label(&first));
            return Err(first);
        }
        self.phase = FillPhase::Saving;
        self.announce = "Saving…".into();
        Ok(())
    }
    /// The save finished.
    pub fn saved(&mut self, ok: Result<(), String>) {
        self.phase = match ok {
            Ok(()) => {
                self.announce = "Saved".into();
                FillPhase::Saved
            }
            Err(m) => {
                self.announce = m.clone();
                FillPhase::Error(m)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIG: &str = "Thanks,\nMaya Patel\nHead of Partnerships at Lumen Labs\nmaya.patel@lumenlabs.io | +1 (415) 555-0132\n500 Howard St, Suite 300, San Francisco";

    #[test]
    fn extracts_a_signature() {
        let r = extract(SIG);
        assert_eq!(r["name"].value, "Maya Patel");
        assert_eq!(r["title"].value, "Head of Partnerships");
        assert_eq!(r["company"].value, "Lumen Labs");
        assert_eq!(r["email"].value, "maya.patel@lumenlabs.io");
        assert_eq!(r["phone"].value, "+1 (415) 555-0132");
        assert!(r["address"].value.starts_with("500 Howard St"));
        let (a, b) = r["email"].source.unwrap();
        assert_eq!(&SIG[a..b], "maya.patel@lumenlabs.io");
        let messy = extract("call jo tmrw 0300 1234567\njo.ellis@harbor.dev");
        assert_eq!(messy["name"].value, "Jo Ellis");
        assert!(messy["name"].confidence < 0.8);
        assert_eq!(messy["company"].value, "Harbor");
    }

    #[test]
    fn fills_staggered_reviews_and_undoes() {
        let mut f = SmartFill::new(contact_fields());
        f.values.insert("name".into(), "Old".into());
        f.paste(SIG, ClockMs(0));
        assert!(f.stop());
        f.paste(SIG, ClockMs(10));
        f.tick(ClockMs(500));
        assert!(matches!(f.phase, FillPhase::Reading(_)));
        f.tick(ClockMs(910));
        assert!(matches!(f.phase, FillPhase::Filling(_)));
        assert_eq!(f.reveal("title", ClockMs(910), false), (0.0, 0.0));
        let (fade, tint) = f.reveal("title", ClockMs(1_150), false);
        assert!(fade > 0.0 && tint > 0.0);
        assert_eq!(f.filling_now(ClockMs(910)).as_deref(), Some("name"));
        assert_eq!(f.needs_review(), ["address"]);
        assert_eq!(f.save(), Err("address".into()));
        f.check("address");
        assert_eq!(f.mark("address").word(), "Checked");
        f.set_threshold(0.95);
        assert!(f.needs_review().contains(&"name".to_string()));
        f.set_threshold(0.5);
        f.edit("phone", "+1 415 555 0000");
        assert_eq!(f.mark("phone"), FieldMark::Edited);
        f.undo_field("title");
        assert!(!f.values.contains_key("title"));
        assert!(f.save().is_ok());
        f.saved(Ok(()));
        assert_eq!(f.phase, FillPhase::Saved);
        assert!(f.undo());
        assert_eq!(f.values["name"], "Old");
    }
}
