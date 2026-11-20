//! Model card: what a model can do and what it costs. Switching models rolls every value in
//! place, a volume slider estimates the monthly cost, benchmark rows show score, rank and gap on
//! hover or focus, and Compare swaps to a side-by-side table against a rival with the better
//! value marked. The card never changes size (every panel shares one grid cell).

use super::{norm_key, nth, rove, tokens_short, usd};

/// The seven modality chips, in order.
pub const MODALITIES: [&str; 7] = [
    "Text",
    "Image",
    "PDF",
    "Audio",
    "Video",
    "Tools",
    "Reasoning",
];

/// Monthly volume stops for the cost slider.
pub const VOLUMES: [u64; 9] = [
    1_000_000,
    5_000_000,
    10_000_000,
    25_000_000,
    50_000_000,
    100_000_000,
    250_000_000,
    500_000_000,
    1_000_000_000,
];

/// Benchmarks: id, short name, full name, what it measures.
pub const BENCHMARKS: [(&str, &str, &str, &str); 3] = [
    (
        "swe",
        "SWE-bench",
        "SWE-bench Verified",
        "real GitHub issues resolved",
    ),
    (
        "gpqa",
        "GPQA",
        "GPQA Diamond",
        "graduate level science questions",
    ),
    (
        "term",
        "Terminal-Bench",
        "Terminal-Bench",
        "tasks finished in a real shell",
    ),
];

/// One model.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// API id.
    pub id: &'static str,
    /// Display name (its mark is resolved from it).
    pub name: &'static str,
    /// Provider.
    pub provider: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Supported modalities (bit per [`MODALITIES`] entry).
    pub modalities: [bool; 7],
    /// Context window, tokens.
    pub context: u64,
    /// Max output, tokens.
    pub max_out: u64,
    /// Knowledge cutoff, `Mon YYYY`.
    pub cutoff: &'static str,
    /// Cutoff as months since year 0 (for compare).
    pub cutoff_m: u32,
    /// USD per 1M input tokens.
    pub input: f64,
    /// USD per 1M output tokens.
    pub output: f64,
    /// USD per 1M cached input tokens.
    pub cached: f64,
    /// Scores per [`BENCHMARKS`] (None: not listed).
    pub scores: [Option<f64>; 3],
}

/// Sample models (sample prices and scores, not official).
pub fn models() -> Vec<Model> {
    vec![
        Model {
            id: "claude-opus-4-1",
            name: "Claude Opus 4.1",
            provider: "Anthropic",
            summary: "Anthropic's most capable model for long agent runs, hard coding and research.",
            modalities: [true, true, true, false, false, true, true],
            context: 200_000,
            max_out: 32_000,
            cutoff: "Mar 2025",
            cutoff_m: 2025 * 12 + 3,
            input: 15.0,
            output: 75.0,
            cached: 1.5,
            scores: [Some(74.5), Some(80.9), Some(43.2)],
        },
        Model {
            id: "claude-sonnet-4-5",
            name: "Claude Sonnet 4.5",
            provider: "Anthropic",
            summary: "The balanced Claude for everyday coding, agents and analysis at scale.",
            modalities: [true, true, true, false, false, true, true],
            context: 1_000_000,
            max_out: 64_000,
            cutoff: "Jul 2025",
            cutoff_m: 2025 * 12 + 7,
            input: 3.0,
            output: 15.0,
            cached: 0.3,
            scores: [Some(77.2), Some(83.4), Some(50.0)],
        },
        Model {
            id: "gpt-5",
            name: "GPT-5",
            provider: "OpenAI",
            summary: "OpenAI's flagship for reasoning, coding and multimodal work.",
            modalities: [true, true, true, true, false, true, true],
            context: 400_000,
            max_out: 128_000,
            cutoff: "Sep 2024",
            cutoff_m: 2024 * 12 + 9,
            input: 1.25,
            output: 10.0,
            cached: 0.125,
            scores: [Some(74.9), Some(85.7), Some(43.8)],
        },
        Model {
            id: "gemini-2-5-pro",
            name: "Gemini 2.5 Pro",
            provider: "Google",
            summary: "Google's thinking model with a long context and native audio and video.",
            modalities: [true, true, true, true, true, true, true],
            context: 1_000_000,
            max_out: 64_000,
            cutoff: "Jan 2025",
            cutoff_m: 2025 * 12 + 1,
            input: 1.25,
            output: 10.0,
            cached: 0.31,
            scores: [Some(67.2), Some(86.4), Some(25.3)],
        },
        Model {
            id: "grok-4",
            name: "Grok 4",
            provider: "xAI",
            summary: "xAI's frontier model with live search and strong maths.",
            modalities: [true, true, false, false, false, true, true],
            context: 256_000,
            max_out: 64_000,
            cutoff: "Nov 2024",
            cutoff_m: 2024 * 12 + 11,
            input: 3.0,
            output: 15.0,
            cached: 0.75,
            scores: [Some(70.8), Some(87.5), None],
        },
    ]
}

/// The card's views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Specs, prices, cost and benchmarks.
    Details,
    /// Side by side with a rival.
    Compare,
}

/// One compare row: label, the model's value, the rival's value, which is better (0 model,
/// 1 rival, 2 tie or n/a) and the margin text on the better value.
#[derive(Clone, Debug, PartialEq)]
pub struct CompareRow {
    /// Row label.
    pub label: String,
    /// The model's value.
    pub a: String,
    /// The rival's value.
    pub b: String,
    /// 0 model, 1 rival, 2 neither.
    pub better: u8,
    /// Margin (`+2.3`, `−20%`).
    pub margin: String,
}

/// The live state of a model card.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelCard {
    /// Models; the card shows `model`, the rest are rivals.
    pub models: Vec<Model>,
    /// Shown model.
    pub model: usize,
    /// View.
    pub view: View,
    /// Rival in compare (index into `models`).
    pub rival: usize,
    /// Rival menu open.
    pub menu: bool,
    /// Benchmark shown.
    pub bench: usize,
    /// Volume stop.
    pub vol: usize,
    /// Share billed at the input price.
    pub input_share: f64,
    /// Hovered or focused benchmark row (index into [`ModelCard::bench_rows`]).
    pub hover: Option<usize>,
    /// Loading skeleton.
    pub loading: bool,
    /// Error message.
    pub error: Option<String>,
    /// The select button reads Selected.
    pub selected: bool,
    /// When the shown model last changed (values roll from it).
    pub changed_at: u64,
}

impl ModelCard {
    /// The sample card on Claude Sonnet 4.5 at 100M tokens a month.
    pub fn new() -> Self {
        Self {
            models: models(),
            model: 1,
            view: View::Details,
            rival: 2,
            menu: false,
            bench: 0,
            vol: 5,
            input_share: 0.75,
            hover: None,
            loading: false,
            error: None,
            selected: false,
            changed_at: 0,
        }
    }

    /// The shown model.
    pub fn m(&self) -> &Model {
        &self.models[self.model]
    }

    /// The compare rival.
    pub fn r(&self) -> &Model {
        &self.models[self.rival]
    }

    /// Monthly cost of model `i` at the chosen volume.
    pub fn cost_of(&self, i: usize) -> f64 {
        let m = &self.models[i];
        let v = VOLUMES[self.vol] as f64 / 1e6;
        v * (self.input_share * m.input + (1.0 - self.input_share) * m.output)
    }

    /// `Fits about 1,500 pages` for the context window (about 667 tokens a page).
    pub fn pages(&self) -> String {
        format!(
            "Fits about {} pages",
            super::thousands(self.m().context / 667 / 50 * 50)
        )
    }

    /// The two rivals shown under the model in the benchmark rows.
    pub fn bench_rivals(&self) -> Vec<usize> {
        (0..self.models.len())
            .filter(|&i| i != self.model)
            .take(2)
            .collect()
    }

    /// Benchmark rows: model index, score.
    pub fn bench_rows(&self) -> Vec<(usize, Option<f64>)> {
        let mut v = vec![(self.model, self.m().scores[self.bench])];
        for i in self.bench_rivals() {
            v.push((i, self.models[i].scores[self.bench]));
        }
        v
    }

    /// Rank of model `i` on the shown benchmark among every model with a score (1-based) and the
    /// gap to the leader.
    pub fn rank(&self, i: usize) -> Option<(usize, f64)> {
        let s = self.models[i].scores[self.bench]?;
        let mut all: Vec<f64> = self
            .models
            .iter()
            .filter_map(|m| m.scores[self.bench])
            .collect();
        all.sort_by(|a, b| b.total_cmp(a));
        let rank = all.iter().position(|&x| x <= s).unwrap_or(0) + 1;
        Some((rank, all[0] - s))
    }

    /// The readout for a benchmark row.
    pub fn readout(&self, row: usize) -> String {
        let rows = self.bench_rows();
        let Some(&(i, score)) = rows.get(row) else {
            return String::new();
        };
        let name = self.models[i].name;
        match (score, self.rank(i)) {
            (Some(s), Some((r, gap))) if gap <= 0.0 => {
                format!("{name}: {s:.1}%, rank {r} of {}, the leader", self.scored())
            }
            (Some(s), Some((r, gap))) => format!(
                "{name}: {s:.1}%, rank {r} of {}, {gap:.1} behind the leader",
                self.scored()
            ),
            _ => format!("{name}: not listed"),
        }
    }

    fn scored(&self) -> usize {
        self.models
            .iter()
            .filter(|m| m.scores[self.bench].is_some())
            .count()
    }

    /// The compare table.
    pub fn compare(&self) -> Vec<CompareRow> {
        let (a, b) = (self.m(), self.r());
        let mods = |m: &Model| {
            MODALITIES
                .iter()
                .zip(m.modalities)
                .filter(|(_, on)| *on)
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let hi = |x: f64, y: f64| -> (u8, String) {
            if (x - y).abs() < 1e-9 {
                (2, String::new())
            } else if x > y {
                (0, format!("+{:.1}", x - y))
            } else {
                (1, format!("+{:.1}", y - x))
            }
        };
        let lo = |x: f64, y: f64| -> (u8, String) {
            if (x - y).abs() < 1e-9 {
                (2, String::new())
            } else if x < y {
                (0, format!("−{:.0}%", (1.0 - x / y) * 100.0))
            } else {
                (1, format!("−{:.0}%", (1.0 - y / x) * 100.0))
            }
        };
        let tok = |x: u64, y: u64| -> (u8, String) {
            if x == y {
                (2, String::new())
            } else if x > y {
                (0, format!("{:.1}x", x as f64 / y as f64))
            } else {
                (1, format!("{:.1}x", y as f64 / x as f64))
            }
        };
        let row = |label: &str, a: String, b: String, (better, margin): (u8, String)| CompareRow {
            label: label.into(),
            a,
            b,
            better,
            margin,
        };
        let mut v = vec![
            row("Modalities", mods(a), mods(b), (2, String::new())),
            row(
                "Context window",
                format!("{} tokens", tokens_short(a.context)),
                format!("{} tokens", tokens_short(b.context)),
                tok(a.context, b.context),
            ),
            row(
                "Max output",
                format!("{} tokens", tokens_short(a.max_out)),
                format!("{} tokens", tokens_short(b.max_out)),
                tok(a.max_out, b.max_out),
            ),
            row("Knowledge cutoff", a.cutoff.into(), b.cutoff.into(), {
                let (x, y) = (a.cutoff_m, b.cutoff_m);
                if x == y {
                    (2, String::new())
                } else if x > y {
                    (0, format!("+{} mo", x - y))
                } else {
                    (1, format!("+{} mo", y - x))
                }
            }),
            row(
                "Input price",
                usd(a.input),
                usd(b.input),
                lo(a.input, b.input),
            ),
            row(
                "Output price",
                usd(a.output),
                usd(b.output),
                lo(a.output, b.output),
            ),
            row(
                &format!("Cost at {}/mo", tokens_short(VOLUMES[self.vol])),
                usd(self.cost_of(self.model)),
                usd(self.cost_of(self.rival)),
                lo(self.cost_of(self.model), self.cost_of(self.rival)),
            ),
        ];
        for (k, bm) in BENCHMARKS.iter().enumerate() {
            let fmt = |s: Option<f64>| s.map(|x| format!("{x:.1}%")).unwrap_or("Not listed".into());
            let better = match (a.scores[k], b.scores[k]) {
                (Some(x), Some(y)) => hi(x, y),
                _ => (2, String::new()),
            };
            v.push(row(bm.1, fmt(a.scores[k]), fmt(b.scores[k]), better));
        }
        v
    }

    /// Who leads the compare table (`GPT-5 leads on 5 of 10, Claude Sonnet 4.5 on 4.`).
    pub fn tally(&self) -> String {
        let rows = self.compare();
        let a = rows.iter().filter(|r| r.better == 0).count();
        let b = rows.iter().filter(|r| r.better == 1).count();
        let (n1, c1, n2, c2) = if a >= b {
            (self.m().name, a, self.r().name, b)
        } else {
            (self.r().name, b, self.m().name, a)
        };
        format!("{n1} leads on {c1} of {}, {n2} on {c2}.", rows.len())
    }

    /// Presses: `mc-model-<i>`, `mc-compare`, `mc-vol-<i>`, `mc-bench-<i>`, `mc-row-<i>` (hover
    /// or focus a benchmark row), `mc-unrow`, `mc-menu`, `mc-rival-<i>`, `mc-copy`, `mc-select`,
    /// `mc-retry`, `mc-fail`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "mc-model-") {
            if i < self.models.len() && i != self.model {
                self.model = i;
                self.changed_at = now;
                if self.rival == i {
                    self.rival = (0..self.models.len()).find(|&j| j != i).unwrap_or(0);
                }
                self.selected = false;
                return Some(format!("Showing {}", self.m().name));
            }
            return None;
        }
        if let Some(i) = nth(id, "mc-vol-") {
            self.vol = i.min(VOLUMES.len() - 1);
            return Some(format!(
                "{} tokens a month, about {} a month",
                tokens_short(VOLUMES[self.vol]),
                usd(self.cost_of(self.model))
            ));
        }
        if let Some(i) = nth(id, "mc-bench-") {
            self.bench = i.min(BENCHMARKS.len() - 1);
            self.hover = None;
            return Some(format!("Benchmark {}", BENCHMARKS[self.bench].2));
        }
        if let Some(i) = nth(id, "mc-row-") {
            self.hover = Some(i);
            return None;
        }
        if let Some(i) = nth(id, "mc-rival-") {
            if i < self.models.len() && i != self.model {
                self.rival = i;
            }
            self.menu = false;
            return Some(format!(
                "Comparing {} with {}",
                self.m().name,
                self.r().name
            ));
        }
        match id {
            "mc-unrow" => {
                self.hover = None;
                None
            }
            "mc-compare" if !self.loading && self.error.is_none() => {
                self.view = if self.view == View::Details {
                    View::Compare
                } else {
                    View::Details
                };
                self.menu = false;
                Some(if self.view == View::Compare {
                    format!("Comparing {} with {}", self.m().name, self.r().name)
                } else {
                    format!("Details of {}", self.m().name)
                })
            }
            "mc-menu" => {
                self.menu = !self.menu;
                None
            }
            "mc-copy" => Some(format!("Copied {}", self.m().id)),
            "mc-select" => {
                self.selected = true;
                Some(format!("{} selected", self.m().name))
            }
            "mc-retry" => {
                self.error = None;
                self.loading = false;
                Some(format!("Loaded {}", self.m().name))
            }
            "mc-fail" => {
                self.error = Some("The model catalogue did not answer in time.".into());
                Some("Couldn't load model details".into())
            }
            _ => None,
        }
    }

    /// Keys on the volume slider (arrows, Page up / down, Home / End) or the benchmark switcher
    /// (`bench`: ←/→, Home / End) or the rival menu (`menu`: ↑/↓, Enter, Escape).
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        match part {
            "vol" => {
                let i = match key {
                    "PageUp" => (self.vol + 2).min(VOLUMES.len() - 1),
                    "PageDown" => self.vol.saturating_sub(2),
                    k => rove(self.vol, VOLUMES.len(), k),
                };
                self.press(&format!("mc-vol-{i}"), now)
            }
            "bench" => {
                let i = rove(self.bench, BENCHMARKS.len(), key);
                self.press(&format!("mc-bench-{i}"), now)
            }
            "menu" => match key {
                "Escape" => {
                    self.menu = false;
                    None
                }
                "ArrowDown" | "ArrowUp" => {
                    let pool: Vec<usize> = (0..self.models.len())
                        .filter(|&i| i != self.model)
                        .collect();
                    let p = pool.iter().position(|&i| i == self.rival).unwrap_or(0);
                    self.rival = pool[rove(p, pool.len(), key)];
                    self.menu = true;
                    None
                }
                "Enter" | "Space" => {
                    if self.menu {
                        self.press(&format!("mc-rival-{}", self.rival), now)
                    } else {
                        self.menu = true;
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl Default for ModelCard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_card_rolls_compares_and_keys() {
        let mut c = ModelCard::new();
        assert_eq!(usd(c.cost_of(1)), "$600");
        c.key("vol", "End", 0);
        assert_eq!(c.vol, 8);
        c.key("vol", "PageDown", 0);
        assert_eq!(c.vol, 6);
        c.press("mc-model-2", 10);
        assert_eq!(c.m().name, "GPT-5");
        assert_eq!(c.changed_at, 10);
        assert_ne!(c.rival, 2);
        assert!(c.readout(0).contains("rank"));
        c.press("mc-compare", 0);
        assert_eq!(c.view, View::Compare);
        let rows = c.compare();
        assert_eq!(rows.len(), 10);
        assert!(rows.iter().any(|r| r.better == 0));
        assert!(c.tally().contains("leads on"));
        c.key("menu", "ArrowDown", 0);
        assert!(c.menu);
        c.key("menu", "Enter", 0);
        assert!(!c.menu);
        c.press("mc-fail", 0);
        assert!(c.press("mc-compare", 0).is_none());
        c.press("mc-retry", 0);
        assert!(c.error.is_none());
        assert!(c.pages().starts_with("Fits about"));
    }
}
