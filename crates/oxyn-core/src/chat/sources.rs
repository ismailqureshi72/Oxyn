//! Citations and sources (`citations-sources.md`).
//!
//! - **Source cards** (Arc UI Source cards): cited sources under an answer; while searching cards
//!   stream into reserved slots, the overflow collapses into a favicon stack, a citation click
//!   highlights its card for 2.6 s.
//! - **Cited / AI answer** (affectUI AI Answer + Cited Answer, Arc UI AI answer): an answer whose
//!   claims carry provenance (Oxyn plus: sourced · inferred · unverified).
//! - **Context cards** and **Recommendation card** (Beautiful UI, MIT © 2026 Shane Levine): the
//!   context an agent used, and a recommendation with confidence, impact and accept / dismiss.
//! - **Document extraction** (Arc UI): fields pulled from a document with confidence, page
//!   provenance and review.

use std::time::Duration;

use crate::ClockMs;

/// A source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// Page title.
    pub title: String,
    /// Full URL.
    pub url: String,
    /// Site name.
    pub site: Option<String>,
    /// Passage relied on.
    pub snippet: Option<String>,
    /// Date label.
    pub published: Option<String>,
}

impl Source {
    /// A source.
    pub fn new(title: &str, url: &str) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            site: None,
            snippet: None,
            published: None,
        }
    }
    /// Site name.
    pub fn site(mut self, s: &str) -> Self {
        self.site = Some(s.into());
        self
    }
    /// Snippet.
    pub fn snippet(mut self, s: &str) -> Self {
        self.snippet = Some(s.into());
        self
    }
    /// Date.
    pub fn published(mut self, s: &str) -> Self {
        self.published = Some(s.into());
        self
    }
    /// Host without `www.`.
    pub fn domain(&self) -> String {
        domain(&self.url)
    }
    /// Card name: "2. Title, domain".
    pub fn card_label(&self, n: usize) -> String {
        format!("{n}. {}, {}, opens in a new tab", self.title, self.domain())
    }
}

/// Host of a URL without scheme or `www.`.
pub fn domain(url: &str) -> String {
    let s = url.split("://").nth(1).unwrap_or(url);
    let h = s.split(['/', '?', '#']).next().unwrap_or(s);
    h.strip_prefix("www.").unwrap_or(h).to_string()
}

/// Search status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchStatus {
    /// Searching.
    Searching,
    /// Done.
    #[default]
    Done,
    /// Failed.
    Error,
}

impl SearchStatus {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "searching" => Self::Searching,
            "error" => Self::Error,
            _ => Self::Done,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Searching => "searching",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

/// How long a citation highlight lasts.
pub const HIGHLIGHT: Duration = Duration::from_millis(2600);

/// A slot in the card row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Slot {
    /// A card (0-based source index).
    Card(usize),
    /// Reserved while searching.
    Skeleton,
    /// The favicon stack for sources from this index on.
    Stack(usize),
}

/// Source cards state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCards {
    /// Sources in citation order.
    pub sources: Vec<Source>,
    /// Status.
    pub status: SearchStatus,
    /// Expected total (reserves slots).
    pub total: Option<usize>,
    /// Cards before the stack.
    pub max: usize,
    /// Stack open.
    pub expanded: bool,
    /// Highlighted (1-based) and since.
    pub highlighted: Option<(usize, ClockMs)>,
}

impl SourceCards {
    /// Cards.
    pub fn new(sources: Vec<Source>, status: SearchStatus) -> Self {
        Self {
            sources,
            status,
            total: None,
            max: 3,
            expanded: false,
            highlighted: None,
        }
    }
    /// Slots in the row.
    pub fn slots(&self) -> Vec<Slot> {
        let n = self.sources.len();
        if self.expanded {
            return (0..n).map(Slot::Card).collect();
        }
        let want = match self.status {
            SearchStatus::Searching => self.total.unwrap_or(n).max(n),
            _ => n,
        };
        let mut v = Vec::new();
        let shown = want.min(self.max);
        for i in 0..shown {
            v.push(if i < n { Slot::Card(i) } else { Slot::Skeleton });
        }
        if want > self.max {
            v.push(if n > self.max {
                Slot::Stack(self.max)
            } else {
                Slot::Skeleton
            });
        }
        v
    }
    /// Status line ("Searching 3 of 7", "7 sources · 2.1s", "No sources found", "Search failed").
    pub fn status_line(&self, duration_ms: Option<u64>) -> String {
        let n = self.sources.len();
        match self.status {
            SearchStatus::Searching => match self.total {
                Some(t) => format!("Searching {n} of {t}"),
                None => "Searching".into(),
            },
            SearchStatus::Done if n == 0 => "No sources found".into(),
            SearchStatus::Done => {
                let s = if n == 1 { "source" } else { "sources" };
                match duration_ms {
                    Some(d) => format!("{n} {s} · {:.1}s", d as f64 / 1000.0),
                    None => format!("{n} {s}"),
                }
            }
            SearchStatus::Error => "Search failed".into(),
        }
    }
    /// A citation was pressed: highlights its card (opening the stack when it lives there).
    pub fn cite(&mut self, n: usize, now: ClockMs) {
        if n == 0 || n > self.sources.len() {
            return;
        }
        if n > self.max {
            self.expanded = true;
        }
        self.highlighted = Some((n, now));
    }
    /// Highlighted card at `now` (expires after [`HIGHLIGHT`]).
    pub fn highlight_at(&self, now: ClockMs) -> Option<usize> {
        self.highlighted
            .filter(|(_, t)| now.since(*t) < HIGHLIGHT)
            .map(|(n, _)| n)
    }
    /// Citation button name: "Source 2: Title, domain".
    pub fn citation_label(&self, n: usize) -> String {
        match self.sources.get(n.wrapping_sub(1)) {
            Some(s) => format!("Source {n}: {}, {}", s.title, s.domain()),
            None => format!("Source {n}"),
        }
    }
}

/// Claim provenance (Oxyn plus).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Backed by these sources (1-based).
    Sourced(Vec<usize>),
    /// The model's inference from the sources.
    Inferred,
    /// Not checked against any source.
    Unverified,
}

impl Provenance {
    /// Id.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Sourced(_) => "sourced",
            Self::Inferred => "inferred",
            Self::Unverified => "unverified",
        }
    }
    /// Label.
    pub fn label(&self) -> String {
        match self {
            Self::Sourced(v) if v.len() == 1 => "1 source".into(),
            Self::Sourced(v) => format!("{} sources", v.len()),
            Self::Inferred => "Inferred".into(),
            Self::Unverified => "Unverified".into(),
        }
    }
}

/// A claim in a cited answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    /// Sentence.
    pub text: String,
    /// Provenance.
    pub provenance: Provenance,
}

impl Claim {
    /// A sourced claim.
    pub fn sourced(text: &str, cites: &[usize]) -> Self {
        Self {
            text: text.into(),
            provenance: Provenance::Sourced(cites.to_vec()),
        }
    }
    /// An inferred claim.
    pub fn inferred(text: &str) -> Self {
        Self {
            text: text.into(),
            provenance: Provenance::Inferred,
        }
    }
    /// An unverified claim.
    pub fn unverified(text: &str) -> Self {
        Self {
            text: text.into(),
            provenance: Provenance::Unverified,
        }
    }
}

/// Share of claims that are sourced (0–1), for the answer's confidence meter.
pub fn grounding(claims: &[Claim]) -> f32 {
    if claims.is_empty() {
        return 0.0;
    }
    let s = claims
        .iter()
        .filter(|c| matches!(c.provenance, Provenance::Sourced(_)))
        .count();
    s as f32 / claims.len() as f32
}

// ─────────────────────────────── context / recommend ──

/// A context card (what the agent used).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextCard {
    /// Kind: `file` · `doc` · `web` · `memory` · `tool`.
    pub kind: String,
    /// Title.
    pub title: String,
    /// Meta ("auth.rs · 182 lines").
    pub meta: String,
    /// Pinned by the user.
    pub pinned: bool,
    /// Token cost.
    pub tokens: u32,
}

impl ContextCard {
    /// A card.
    pub fn new(kind: &str, title: &str, meta: &str, tokens: u32) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            meta: meta.into(),
            pinned: false,
            tokens,
        }
    }
    /// Icon for the kind.
    pub fn icon(&self) -> &'static str {
        match self.kind.as_str() {
            "file" => "file",
            "doc" => "page",
            "web" => "globe",
            "memory" => "brain",
            "tool" => "tool",
            _ => "link",
        }
    }
}

/// Total tokens of the cards.
pub fn context_tokens(cards: &[ContextCard]) -> u32 {
    cards.iter().map(|c| c.tokens).sum()
}

/// Recommendation decision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decision {
    /// Open.
    #[default]
    Open,
    /// Accepted.
    Accepted,
    /// Dismissed.
    Dismissed,
}

/// A recommendation (Beautiful UI Recommendation Card).
#[derive(Clone, Debug, PartialEq)]
pub struct Recommendation {
    /// Title.
    pub title: String,
    /// Why.
    pub reason: String,
    /// 0–1.
    pub confidence: f32,
    /// Impact line ("−$1,240 / mo").
    pub impact: Option<String>,
    /// Supporting evidence lines.
    pub evidence: Vec<String>,
    /// Decision.
    pub decision: Decision,
}

impl Recommendation {
    /// A recommendation.
    pub fn new(title: &str, reason: &str, confidence: f32) -> Self {
        Self {
            title: title.into(),
            reason: reason.into(),
            confidence: confidence.clamp(0.0, 1.0),
            impact: None,
            evidence: vec![],
            decision: Decision::Open,
        }
    }
    /// Confidence band.
    pub fn band(&self) -> &'static str {
        if self.confidence >= 0.8 {
            "high"
        } else if self.confidence >= 0.5 {
            "medium"
        } else {
            "low"
        }
    }
}

// ───────────────────────────────── document extraction ──

/// An extracted field.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// Label.
    pub label: String,
    /// Value.
    pub value: String,
    /// 0–1.
    pub confidence: f32,
    /// Page it came from.
    pub page: u32,
    /// Reviewed by the user.
    pub reviewed: bool,
}

impl Field {
    /// A field.
    pub fn new(label: &str, value: &str, confidence: f32, page: u32) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            confidence,
            page,
            reviewed: false,
        }
    }
    /// Needs review (below the threshold and not reviewed).
    pub fn needs_review(&self, threshold: f32) -> bool {
        !self.reviewed && self.confidence < threshold
    }
}

/// Extraction phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExtractPhase {
    /// Reading pages.
    Scanning,
    /// Fields ready for review.
    #[default]
    Review,
    /// Everything confirmed.
    Confirmed,
}

/// Document extraction.
#[derive(Clone, Debug, PartialEq)]
pub struct Extraction {
    /// Fields.
    pub fields: Vec<Field>,
    /// Phase.
    pub phase: ExtractPhase,
    /// Review threshold.
    pub threshold: f32,
    /// Field whose source is highlighted.
    pub focus: Option<usize>,
}

impl Extraction {
    /// An extraction.
    pub fn new(fields: Vec<Field>) -> Self {
        Self {
            fields,
            phase: ExtractPhase::Review,
            threshold: 0.8,
            focus: None,
        }
    }
    /// Fields still needing review.
    pub fn pending(&self) -> usize {
        self.fields
            .iter()
            .filter(|f| f.needs_review(self.threshold))
            .count()
    }
    /// Marks one reviewed (optionally edited).
    pub fn review(&mut self, i: usize, value: Option<&str>) {
        if let Some(f) = self.fields.get_mut(i) {
            if let Some(v) = value {
                f.value = v.into();
                f.confidence = 1.0;
            }
            f.reviewed = true;
        }
    }
    /// Confirm: only when nothing needs review.
    pub fn confirm(&mut self) -> bool {
        if self.pending() == 0 {
            self.phase = ExtractPhase::Confirmed;
            true
        } else {
            false
        }
    }
    /// Fields visible `elapsed` into scanning at one per `step`.
    pub fn scanned(&self, elapsed: Duration, step: Duration) -> usize {
        ((elapsed.as_millis() / step.as_millis().max(1)) as usize).min(self.fields.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(n: usize) -> Vec<Source> {
        (0..n)
            .map(|i| Source::new(&format!("T{i}"), &format!("https://www.site{i}.com/a")))
            .collect()
    }

    #[test]
    fn domains_and_labels() {
        assert_eq!(domain("https://www.github.com/x?y"), "github.com");
        assert_eq!(domain("developer.mozilla.org/en"), "developer.mozilla.org");
        let s = Source::new("Req", "https://developer.mozilla.org/x");
        assert!(s.card_label(2).starts_with("2. Req, developer.mozilla.org"));
    }

    #[test]
    fn slots_reserve_and_stack() {
        let mut c = SourceCards::new(src(2), SearchStatus::Searching);
        c.total = Some(7);
        assert_eq!(
            c.slots(),
            vec![Slot::Card(0), Slot::Card(1), Slot::Skeleton, Slot::Skeleton]
        );
        assert_eq!(c.status_line(None), "Searching 2 of 7");
        c.sources = src(7);
        c.status = SearchStatus::Done;
        assert_eq!(c.slots().last(), Some(&Slot::Stack(3)));
        assert_eq!(c.status_line(Some(2100)), "7 sources · 2.1s");
        c.cite(5, ClockMs(0));
        assert!(c.expanded);
        assert_eq!(c.highlight_at(ClockMs(2000)), Some(5));
        assert_eq!(c.highlight_at(ClockMs(3000)), None);
        assert_eq!(c.slots().len(), 7);
        c.sources.clear();
        assert_eq!(c.status_line(None), "No sources found");
    }

    #[test]
    fn provenance_and_grounding() {
        let cl = vec![
            Claim::sourced("a", &[1]),
            Claim::inferred("b"),
            Claim::unverified("c"),
            Claim::sourced("d", &[1, 2]),
        ];
        assert_eq!(grounding(&cl), 0.5);
        assert_eq!(cl[3].provenance.label(), "2 sources");
    }

    #[test]
    fn extraction_review() {
        let mut e = Extraction::new(vec![
            Field::new("Vendor", "Acme", 0.98, 1),
            Field::new("Total", "$1,240", 0.62, 2),
        ]);
        assert_eq!(e.pending(), 1);
        assert!(!e.confirm());
        e.review(1, Some("$1,240.00"));
        assert!(e.confirm());
        assert_eq!(e.phase, ExtractPhase::Confirmed);
        assert_eq!(
            e.scanned(Duration::from_millis(900), Duration::from_millis(400)),
            2
        );
        let r = Recommendation::new("x", "y", 0.86);
        assert_eq!(r.band(), "high");
        assert_eq!(
            context_tokens(&[ContextCard::new("file", "a", "", 120)]),
            120
        );
    }
}
