//! LLM cost calculator: set tokens per request, requests per day and the prompt cache hit rate,
//! and eight models re-rank by monthly cost; bars show each model's share of the most expensive.

use super::{norm_key, nth, rove, thousands, usd};

/// A priced model: name, provider mark, input, output, cached input (USD per 1M tokens).
pub type Priced = (&'static str, &'static str, f64, f64, f64);

/// Sample prices (per 1M tokens, not official).
pub const MODELS: [Priced; 8] = [
    ("Claude Opus 4.1", "claude", 15.0, 75.0, 1.5),
    ("Claude Sonnet 4.5", "claude", 3.0, 15.0, 0.3),
    ("Claude Haiku 4.5", "claude", 1.0, 5.0, 0.1),
    ("GPT-5", "openai", 1.25, 10.0, 0.125),
    ("GPT-5 mini", "openai", 0.25, 2.0, 0.025),
    ("Gemini 2.5 Pro", "gemini", 1.25, 10.0, 0.31),
    ("Gemini 2.5 Flash", "gemini", 0.3, 2.5, 0.075),
    ("Mistral Medium 3", "mistral", 0.4, 2.0, 0.4),
];

/// Usage presets: name, input tokens, output tokens, requests a day, cache hit %.
pub const PRESETS: [(&str, u64, u64, u64, u64); 3] = [
    ("Support chat", 1_500, 400, 5_000, 40),
    ("RAG search", 6_000, 600, 2_000, 60),
    ("Coding agent", 40_000, 3_000, 800, 80),
];

/// Slider ranges: (min, max, step) for input, output, requests, cache %.
pub const RANGES: [(u64, u64, u64); 4] = [
    (100, 100_000, 100),
    (50, 20_000, 50),
    (10, 100_000, 10),
    (0, 100, 5),
];

/// The live state of the calculator.
#[derive(Clone, Debug, PartialEq)]
pub struct Cost {
    /// Input tokens per request.
    pub input: u64,
    /// Output tokens per request.
    pub output: u64,
    /// Requests per day.
    pub requests: u64,
    /// Share of input served from the prompt cache, %.
    pub cache: u64,
    /// Days per month.
    pub days: u64,
    /// Selected model.
    pub selected: usize,
    /// Batch API discount (50% off, Oxyn extra).
    pub batch: bool,
    /// Loading skeleton.
    pub loading: bool,
    /// Error message.
    pub error: Option<String>,
}

impl Cost {
    /// The Support chat preset.
    pub fn new() -> Self {
        let p = PRESETS[0];
        Self {
            input: p.1,
            output: p.2,
            requests: p.3,
            cache: p.4,
            days: 30,
            selected: 1,
            batch: false,
            loading: false,
            error: None,
        }
    }

    /// Monthly cost of model `i`.
    pub fn monthly(&self, i: usize) -> f64 {
        let (_, _, inp, out, cached) = MODELS[i];
        let req = (self.requests * self.days) as f64;
        let hit = self.cache as f64 / 100.0;
        let per = self.input as f64 * ((1.0 - hit) * inp + hit * cached) + self.output as f64 * out;
        let v = per * req / 1e6;
        if self.batch { v * 0.5 } else { v }
    }

    /// Model indices cheapest first.
    pub fn ranked(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..MODELS.len()).collect();
        v.sort_by(|&a, &b| self.monthly(a).total_cmp(&self.monthly(b)));
        v
    }

    /// Rank (0-based) of model `i`.
    pub fn rank_of(&self, i: usize) -> usize {
        self.ranked().iter().position(|&x| x == i).unwrap_or(0)
    }

    /// Bar share of model `i` against the most expensive.
    pub fn share(&self, i: usize) -> f64 {
        let max = (0..MODELS.len())
            .map(|j| self.monthly(j))
            .fold(0.0, f64::max);
        if max <= 0.0 {
            0.0
        } else {
            self.monthly(i) / max
        }
    }

    /// The summary line for the selected model.
    pub fn summary(&self) -> String {
        let i = self.selected;
        let cheapest = self.ranked()[0];
        let per_req = self.monthly(i) / (self.requests * self.days).max(1) as f64;
        let mut s = format!(
            "{} costs {} a month, {} per request",
            MODELS[i].0,
            usd(self.monthly(i)),
            usd(per_req)
        );
        if cheapest != i {
            s.push_str(&format!(
                ", {}x the cheapest ({})",
                format_args!("{:.1}", self.monthly(i) / self.monthly(cheapest).max(1e-9)),
                MODELS[cheapest].0
            ));
        }
        s
    }

    /// Monthly tokens line (`57M input · 12M output a month`).
    pub fn volume(&self) -> String {
        let m = self.requests * self.days;
        format!(
            "{} requests · {}M input · {}M output a month",
            thousands(m),
            format_args!("{:.1}", (self.input * m) as f64 / 1e6),
            format_args!("{:.1}", (self.output * m) as f64 / 1e6)
        )
    }

    fn val(&mut self, k: usize) -> &mut u64 {
        match k {
            0 => &mut self.input,
            1 => &mut self.output,
            2 => &mut self.requests,
            _ => &mut self.cache,
        }
    }

    /// Sets slider `k` (0 input, 1 output, 2 requests, 3 cache %) to `v`, clamped and stepped.
    pub fn set(&mut self, k: usize, v: u64) -> String {
        let (lo, hi, step) = RANGES[k.min(3)];
        let v = (v.clamp(lo, hi) / step) * step;
        *self.val(k) = v;
        let what = [
            "input tokens",
            "output tokens",
            "requests a day",
            "% cache hits",
        ][k.min(3)];
        format!(
            "{} {what}, cheapest is {}",
            thousands(v),
            MODELS[self.ranked()[0]].0
        )
    }

    /// Presses: `lc-preset-<i>`, `lc-sel-<i>`, `lc-set-<k>-<v>`, `lc-batch`, `lc-retry`, `lc-fail`.
    pub fn press(&mut self, id: &str) -> Option<String> {
        if let Some(i) = nth(id, "lc-preset-") {
            let p = PRESETS.get(i)?;
            (self.input, self.output, self.requests, self.cache) = (p.1, p.2, p.3, p.4);
            return Some(format!(
                "{} preset, cheapest is {}",
                p.0,
                MODELS[self.ranked()[0]].0
            ));
        }
        if let Some(i) = nth(id, "lc-sel-") {
            self.selected = i.min(MODELS.len() - 1);
            return Some(self.summary());
        }
        if let Some(rest) = id.strip_prefix("lc-set-") {
            let (k, v) = rest.split_once('-')?;
            return Some(self.set(k.parse().ok()?, v.parse().ok()?));
        }
        match id {
            "lc-batch" => {
                self.batch = !self.batch;
                Some(
                    if self.batch {
                        "Batch pricing, half price"
                    } else {
                        "Standard pricing"
                    }
                    .into(),
                )
            }
            "lc-retry" => {
                self.error = None;
                Some("Prices loaded".into())
            }
            "lc-fail" => {
                self.error = Some("Prices could not be loaded.".into());
                None
            }
            _ => None,
        }
    }

    /// Keys on slider `k`: ←/→ one step (Shift: ten), PageUp/PageDown ten steps, Home/End.
    /// On the model list (`k` = 9): ↑/↓, Home, End select.
    pub fn key(&mut self, k: usize, key: &str, shift: bool) -> Option<String> {
        let key = norm_key(key);
        if k == 9 {
            let r = self.ranked();
            let p = self.rank_of(self.selected);
            let i = r[rove(p, r.len(), key)];
            return self.press(&format!("lc-sel-{i}"));
        }
        let (lo, hi, step) = RANGES[k.min(3)];
        let cur = *self.val(k);
        let big = step * 10;
        let to = match key {
            "ArrowLeft" | "ArrowDown" => cur.saturating_sub(if shift { big } else { step }),
            "ArrowRight" | "ArrowUp" => cur + if shift { big } else { step },
            "PageDown" => cur.saturating_sub(big),
            "PageUp" => cur + big,
            "Home" => lo,
            "End" => hi,
            _ => return None,
        };
        Some(self.set(k, to))
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_ranks_and_keys() {
        let mut c = Cost::new();
        assert_eq!(c.ranked()[0], 4);
        assert_eq!(c.ranked()[7], 0);
        assert!((c.share(0) - 1.0).abs() < 1e-9);
        let before = c.monthly(1);
        c.key(3, "End", false);
        assert_eq!(c.cache, 100);
        assert!(c.monthly(1) < before);
        c.key(0, "ArrowRight", true);
        assert_eq!(c.input, 2_500);
        c.press("lc-preset-2");
        assert_eq!(c.input, 40_000);
        c.press("lc-batch");
        let half = c.monthly(1);
        c.press("lc-batch");
        assert!((c.monthly(1) - half * 2.0).abs() < 1e-6);
        c.key(9, "Home", false);
        assert_eq!(c.selected, c.ranked()[0]);
        assert!(c.summary().contains("a month"));
        assert!(c.press("lc-set-2-12347").unwrap().contains("12,340"));
    }
}
