//! Knowledge base: build and probe a search index for an assistant. Sources go through a four-step
//! pipeline (parse, chunk, embed, index) while their page and chunk counts climb; a failed source
//! names its reason and offers Retry; Add source brings in a new one that ingests live. Test
//! retrieval runs a question and lists the top chunks with similarity bars, matching words and
//! the source and page; Rerank reorders them in place and says how far each moved; opening a
//! result shows it between its neighbours.

use super::{nth, thousands};

/// Pipeline steps.
pub const PIPE: [&str; 4] = ["Parse", "Chunk", "Embed", "Index"];
/// One step, ms.
pub const STEP_MS: u64 = 1_300;

/// A source: name, kind (brand id or `pdf`/`web`), pages, chunks, starts (ms after the clock),
/// fails at step.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    /// Name.
    pub name: &'static str,
    /// Kind: `pdf`, `web`, or a brand id (`notion`, `googledrive`).
    pub kind: &'static str,
    /// Pages.
    pub pages: u32,
    /// Chunks when indexed.
    pub chunks: u32,
    /// Ingestion started at, host ms (`None` means indexed before).
    pub at: Option<u64>,
    /// Fails at this step.
    pub fail_at: Option<usize>,
    /// Failure reason.
    pub error: &'static str,
}

/// Sources you can add from the Add source menu.
pub const ADDABLE: [(&str, &str, u32, u32); 3] = [
    ("Refund policy 2025.pdf", "pdf", 6, 31),
    ("Onboarding guide", "notion", 9, 44),
    ("status.northwind.app", "web", 12, 58),
];

/// Searchable chunks: source index, page, text.
pub const CHUNKS: [(usize, u32, &str); 9] = [
    (
        0,
        4,
        "Annual plans can be refunded in full within 30 days of purchase. After 30 days we refund the unused months.",
    ),
    (
        0,
        5,
        "Monthly plans are not refunded, but you can cancel at any time and keep access until the period ends.",
    ),
    (
        1,
        2,
        "To change a plan, open Billing, choose Change plan and confirm. The new price applies from the next invoice.",
    ),
    (
        1,
        3,
        "Seats can be added at any time and are charged pro rata for the rest of the billing period.",
    ),
    (
        2,
        1,
        "Single sign-on is available on the Business plan and can be set up by any workspace admin.",
    ),
    (
        2,
        7,
        "If an invoice fails, we retry the card three times over seven days before pausing the workspace.",
    ),
    (
        3,
        1,
        "Refunds are issued to the original payment method and arrive within five to ten business days.",
    ),
    (
        3,
        2,
        "Support answers billing questions within one business day, including refund requests.",
    ),
    (
        0,
        6,
        "Education and non-profit discounts are applied after a short verification form.",
    ),
];
/// Suggested questions.
pub const SUGGESTIONS: [&str; 3] = [
    "How do refunds work for annual plans?",
    "What happens when an invoice fails?",
    "Can I add seats mid-month?",
];

/// The view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Sources and their pipelines.
    Sources,
    /// Test retrieval.
    Test,
}

/// One result: chunk index, score (0..1), rank before rerank, matched words.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    /// Chunk index into [`CHUNKS`].
    pub chunk: usize,
    /// Similarity or reranker score.
    pub score: f64,
    /// 1-based rank before reranking.
    pub was: usize,
    /// Matched words (lowercase).
    pub words: Vec<String>,
}

/// The live state of the block.
#[derive(Clone, Debug, PartialEq)]
pub struct Kb {
    /// Sources.
    pub sources: Vec<Source>,
    /// View.
    pub view: View,
    /// The question searched, if any.
    pub query: Option<String>,
    /// Rerank on.
    pub rerank: bool,
    /// Hybrid search on.
    pub hybrid: bool,
    /// Top k.
    pub top_k: usize,
    /// Open result (chunk index).
    pub open: Option<usize>,
    /// Next addable source.
    pub next_add: usize,
}

const STOP: [&str; 18] = [
    "the", "a", "an", "do", "does", "how", "what", "for", "of", "to", "i", "can", "when", "is",
    "in", "on", "and", "work",
];

impl Kb {
    /// The sample base at `now`: two indexed, one ingesting, one failing at embed (or all
    /// indexed when `settled`).
    pub fn new(settled: bool, now: u64) -> Self {
        let src = |name, kind, pages, chunks, at: Option<u64>, fail_at, error| Source {
            name,
            kind,
            pages,
            chunks,
            at,
            fail_at,
            error,
        };
        Self {
            sources: vec![
                src("Billing handbook.pdf", "pdf", 18, 96, None, None, ""),
                src("Plans and seats", "notion", 7, 38, None, None, ""),
                src(
                    "Admin FAQ",
                    "googledrive",
                    11,
                    52,
                    (!settled).then_some(now),
                    None,
                    "",
                ),
                src(
                    "help.northwind.app",
                    "web",
                    24,
                    120,
                    (!settled).then_some(now.saturating_sub(1_000)),
                    (!settled).then_some(2),
                    "The embedding request timed out after 30 s",
                ),
            ],
            view: View::Sources,
            query: None,
            rerank: false,
            hybrid: true,
            top_k: 4,
            open: None,
            next_add: 0,
        }
    }

    /// Pipeline of source `i` at `now`: step index (4 = indexed), progress within it, failed.
    pub fn pipe(&self, i: usize, now: u64) -> (usize, f64, bool) {
        let s = &self.sources[i];
        let Some(at) = s.at else {
            return (4, 1.0, false);
        };
        let el = now.saturating_sub(at);
        let k = (el / STEP_MS) as usize;
        if let Some(f) = s.fail_at
            && k >= f
        {
            return (f, 0.6, true);
        }
        if k >= 4 {
            (4, 1.0, false)
        } else {
            (k, (el % STEP_MS) as f64 / STEP_MS as f64, false)
        }
    }

    /// Overall percent of source `i`.
    pub fn percent(&self, i: usize, now: u64) -> u32 {
        let (k, p, _) = self.pipe(i, now);
        (((k as f64 + p) / 4.0) * 100.0).min(100.0) as u32
    }

    /// The pipeline's value text (`Embed, step 3 of 4`, `Failed at embed`, `Indexed`).
    pub fn pipe_text(&self, i: usize, now: u64) -> String {
        match self.pipe(i, now) {
            (_, _, true) => format!(
                "Failed at {}",
                PIPE[self.sources[i].fail_at.unwrap_or(0)].to_lowercase()
            ),
            (4, _, _) => "Indexed".into(),
            (k, _, _) => format!("{}, step {} of 4", PIPE[k], k + 1),
        }
    }

    /// Chunks counted so far for source `i` (climbing while it ingests).
    pub fn chunks(&self, i: usize, now: u64) -> u32 {
        let s = &self.sources[i];
        match self.pipe(i, now) {
            (4, _, _) => s.chunks,
            (k, p, _) if k >= 1 => ((s.chunks as f64) * if k == 1 { p } else { 1.0 }) as u32,
            _ => 0,
        }
    }

    /// The summary beside the view switch (`3 of 4 indexed · 186 chunks`).
    pub fn summary(&self, now: u64) -> String {
        let idx = (0..self.sources.len())
            .filter(|&i| self.pipe(i, now).0 == 4)
            .count();
        let ch: u32 = (0..self.sources.len()).map(|i| self.chunks(i, now)).sum();
        format!(
            "{idx} of {} indexed · {} chunks",
            self.sources.len(),
            thousands(u64::from(ch))
        )
    }

    /// Results for the current query at `now`: only indexed sources are searched.
    pub fn results(&self, now: u64) -> Vec<Hit> {
        let Some(q) = &self.query else {
            return Vec::new();
        };
        let qw: Vec<String> = q
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 1 && !STOP.contains(w))
            .map(|w| w.trim_end_matches('s').to_string())
            .collect();
        let mut hits: Vec<Hit> = CHUNKS
            .iter()
            .enumerate()
            .filter(|(_, c)| c.0 < self.sources.len() && self.pipe(c.0, now).0 == 4)
            .map(|(i, c)| {
                let t = c.2.to_lowercase();
                let words: Vec<String> = qw
                    .iter()
                    .filter(|w| t.contains(w.as_str()))
                    .cloned()
                    .collect();
                let base = words.len() as f64 / qw.len().max(1) as f64;
                let bonus = if self.hybrid { 0.06 } else { 0.0 };
                let score = (0.42 + 0.5 * base + bonus - (i as f64) * 0.004).min(0.97);
                Hit {
                    chunk: i,
                    score,
                    was: 0,
                    words,
                }
            })
            .filter(|h| !h.words.is_empty())
            .collect();
        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        hits.truncate(self.top_k);
        for (r, h) in hits.iter_mut().enumerate() {
            h.was = r + 1;
        }
        if self.rerank {
            // The reranker prefers chunks that answer directly: sentences with numbers.
            for h in hits.iter_mut() {
                let t = CHUNKS[h.chunk].2;
                let n = t.chars().filter(|c| c.is_ascii_digit()).count() as f64;
                h.score = (h.score * 0.8 + (n.min(4.0)) * 0.05).min(0.99);
            }
            hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        }
        hits
    }

    /// Neighbours of a chunk in its source (before, after).
    pub fn neighbours(chunk: usize) -> (Option<usize>, Option<usize>) {
        let (s, p, _) = CHUNKS[chunk];
        let same: Vec<usize> = (0..CHUNKS.len()).filter(|&i| CHUNKS[i].0 == s).collect();
        let before = same
            .iter()
            .copied()
            .filter(|&i| CHUNKS[i].1 < p)
            .max_by_key(|&i| CHUNKS[i].1);
        let after = same
            .iter()
            .copied()
            .filter(|&i| CHUNKS[i].1 > p)
            .min_by_key(|&i| CHUNKS[i].1);
        (before, after)
    }

    /// Runs `q` at `now`.
    pub fn search(&mut self, q: &str, now: u64) -> Option<String> {
        let q = q.trim();
        if q.is_empty() {
            self.query = None;
            return None;
        }
        self.query = Some(q.to_string());
        self.view = View::Test;
        self.open = None;
        let n = self.results(now).len();
        Some(format!(
            "{n} chunks found{}",
            if self.rerank { ", reranked" } else { "" }
        ))
    }

    /// Presses: `kb-view-<0|1>`, `kb-retry-<i>`, `kb-remove-<i>`, `kb-add`, `kb-sug-<i>`,
    /// `kb-rerank`, `kb-hybrid`, `kb-k-<n>`, `kb-open-<chunk>`, `kb-clear`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(v) = nth(id, "kb-view-") {
            self.view = if v == 1 { View::Test } else { View::Sources };
            return Some(if v == 1 { "Test retrieval" } else { "Sources" }.into());
        }
        if let Some(i) = nth(id, "kb-retry-") {
            if !self.pipe(i, now).2 {
                return None;
            }
            let s = &mut self.sources[i];
            s.at = Some(now.saturating_sub(STEP_MS * 2));
            s.fail_at = None;
            return Some(format!("Retrying {} from embed", s.name));
        }
        if let Some(i) = nth(id, "kb-remove-") {
            if i >= self.sources.len() {
                return None;
            }
            let s = self.sources.remove(i);
            return Some(format!("{} removed", s.name));
        }
        if let Some(i) = nth(id, "kb-sug-") {
            return self.search(SUGGESTIONS.get(i)?, now);
        }
        if let Some(k) = nth(id, "kb-k-") {
            self.top_k = k.clamp(2, 8);
            return Some(format!("Top {}", self.top_k));
        }
        if let Some(c) = nth(id, "kb-open-") {
            self.open = if self.open == Some(c) { None } else { Some(c) };
            return None;
        }
        match id {
            "kb-add" => {
                let (name, kind, pages, chunks) = *ADDABLE.get(self.next_add)?;
                self.next_add += 1;
                self.sources.push(Source {
                    name,
                    kind,
                    pages,
                    chunks,
                    at: Some(now),
                    fail_at: None,
                    error: "",
                });
                self.view = View::Sources;
                Some(format!(
                    "Adding {name}. Files stay in the browser in this preview"
                ))
            }
            "kb-rerank" => {
                self.rerank = !self.rerank;
                Some(format!(
                    "Rerank {}",
                    if self.rerank {
                        "on: results reordered"
                    } else {
                        "off"
                    }
                ))
            }
            "kb-hybrid" => {
                self.hybrid = !self.hybrid;
                Some(format!(
                    "Hybrid search {}",
                    if self.hybrid { "on" } else { "off" }
                ))
            }
            "kb-clear" => {
                self.query = None;
                self.open = None;
                Some("Cleared".into())
            }
            _ => None,
        }
    }

    /// Keys: Left and Right on the view switch.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        match super::norm_key(key) {
            "ArrowLeft" | "Home" if self.view == View::Test => self.press("kb-view-0", now),
            "ArrowRight" | "End" if self.view == View::Sources => self.press("kb-view-1", now),
            "Escape" => self.press("kb-clear", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_search_rerank() {
        let mut k = Kb::new(false, 0);
        assert_eq!(k.pipe_text(2, 0), "Parse, step 1 of 4");
        assert_eq!(k.pipe_text(2, 6_000), "Indexed");
        assert_eq!(k.pipe_text(3, 3_000), "Failed at embed");
        assert!(k.summary(0).starts_with("2 of 4"));
        k.press("kb-retry-3", 3_000);
        assert_eq!(k.pipe(3, 6_000).0, 4);
        k.press("kb-sug-0", 9_000);
        let r = k.results(9_000);
        assert!(!r.is_empty() && r.len() <= 4);
        assert_eq!(CHUNKS[r[0].chunk].0, 0);
        k.press("kb-rerank", 9_000);
        let r2 = k.results(9_000);
        assert!(r2.iter().any(|h| h.was > 0));
        assert_eq!(Kb::neighbours(0), (None, Some(1)));
        k.press("kb-add", 9_000);
        assert_eq!(k.sources.len(), 5);
        k.key("ArrowLeft", 9_000);
        assert_eq!(k.view, View::Sources);
    }
}
