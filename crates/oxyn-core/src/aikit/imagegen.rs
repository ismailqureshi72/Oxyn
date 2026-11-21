//! Image generation: an image studio card. Four variations form on a dot matrix (a soft wave,
//! then the colours of the coming image, then a blur into the sharp image), a filmstrip picks
//! one, Upscale doubles it, Vary makes four more from it, and a prompt bar with model and aspect
//! chips closes the card. Escape stops a running batch and keeps what finished.

use super::{hash, norm_key, nth, rove};

/// Image models offered as chips: name, mark.
pub const MODELS: [(&str, &str); 3] = [
    ("GPT Image 1", "openai"),
    ("Imagen 4", "gemini"),
    ("Grok Imagine", "xai"),
];

/// Output shapes: label, width, height.
pub const ASPECTS: [(&str, u32, u32); 3] =
    [("1:1", 1024, 1024), ("4:5", 896, 1120), ("16:9", 1344, 768)];

/// Starter prompts for a blank canvas.
pub const SUGGESTIONS: [&str; 3] = [
    "A lighthouse on black rocks at golden hour, long exposure sea",
    "A quiet reading nook with plants and morning light",
    "Snowy pine forest under a violet dawn sky",
];

/// Sample scenes the simulation paints: name and the gradient stops (sky, mid, ground, sun).
pub const SCENES: [(&str, [&str; 4]); 6] = [
    (
        "golden hour coast",
        ["#F6B26B", "#E06666", "#274E63", "#FFE599"],
    ),
    (
        "violet dawn forest",
        ["#8E7CC3", "#C27BA0", "#1E3A2F", "#F9CB9C"],
    ),
    (
        "morning reading nook",
        ["#FCE5CD", "#B6D7A8", "#4E5D6C", "#FFF2CC"],
    ),
    (
        "teal night harbour",
        ["#0B5394", "#134F5C", "#0C343D", "#CFE2F3"],
    ),
    ("desert noon", ["#9FC5E8", "#F9CB9C", "#B45F06", "#FFFFFF"]),
    (
        "rainy city dusk",
        ["#674EA7", "#3D85C6", "#20124D", "#FFD966"],
    ),
];

/// One variation.
#[derive(Clone, Debug, PartialEq)]
pub struct Variation {
    /// Start, ms.
    pub start: u64,
    /// Length, ms.
    pub dur: u64,
    /// Scene index into [`SCENES`].
    pub scene: usize,
    /// Fails at 60% (the Fails scenario).
    pub fails: bool,
}

/// A variation's phase at `now`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VarSt {
    /// Not generated yet.
    Empty,
    /// Forming, with progress 0–1.
    Forming(f64),
    /// Finished.
    Done,
    /// Failed.
    Failed,
    /// Stopped by Escape.
    Stopped,
}

/// The live state of the image studio.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageGen {
    /// Prompt.
    pub prompt: String,
    /// Model chip.
    pub model: usize,
    /// Aspect chip.
    pub aspect: usize,
    /// Current batch.
    pub vars: Vec<Variation>,
    /// Stopped at.
    pub stopped: Option<u64>,
    /// Variation on the canvas.
    pub chosen: usize,
    /// Upscale started at (on the chosen image).
    pub upscale: Option<u64>,
    /// Batches made (for Vary).
    pub batches: u32,
    /// The third variation fails.
    pub fail_mode: bool,
}

/// Upscale time, ms.
pub const UPSCALE_MS: u64 = 1_400;

impl ImageGen {
    /// The studio; `auto` starts a batch at `now`.
    pub fn new(auto: bool, fail_mode: bool, now: u64) -> Self {
        let mut g = Self {
            prompt: SUGGESTIONS[0].to_string(),
            model: 0,
            aspect: 0,
            vars: vec![],
            stopped: None,
            chosen: 0,
            upscale: None,
            batches: 0,
            fail_mode,
        };
        if auto {
            g.generate(now);
        }
        g
    }

    fn generate(&mut self, now: u64) {
        let h = hash(&format!("{}{}{}", self.prompt, self.model, self.batches));
        self.vars = (0..4)
            .map(|i| Variation {
                start: now + i as u64 * 180,
                dur: 2_600 + (i as u64 * 377 + h % 500) % 900,
                scene: ((h >> (i * 4)) as usize + i) % SCENES.len(),
                fails: self.fail_mode && i == 2,
            })
            .collect();
        self.batches += 1;
        self.stopped = None;
        self.upscale = None;
        self.chosen = 0;
    }

    /// Phase of variation `i`.
    pub fn var_st(&self, i: usize, now: u64) -> VarSt {
        let Some(v) = self.vars.get(i) else {
            return VarSt::Empty;
        };
        let t = self.stopped.unwrap_or(now);
        let p = (t.saturating_sub(v.start)) as f64 / v.dur as f64;
        if v.fails && p >= 0.6 {
            return VarSt::Failed;
        }
        if p >= 1.0 {
            VarSt::Done
        } else if self.stopped.is_some() {
            VarSt::Stopped
        } else {
            VarSt::Forming(p.max(0.0))
        }
    }

    /// True while any variation is forming.
    pub fn busy(&self, now: u64) -> bool {
        (0..self.vars.len()).any(|i| matches!(self.var_st(i, now), VarSt::Forming(_)))
    }

    /// Upscale progress of the chosen image (`None` when not upscaling).
    pub fn upscale_p(&self, now: u64) -> Option<f64> {
        self.upscale
            .map(|u| ((now.saturating_sub(u)) as f64 / UPSCALE_MS as f64).min(1.0))
    }

    /// The size label (`1024 × 1024`, doubled once upscaled).
    pub fn size_label(&self, now: u64) -> String {
        let (_, w, h) = ASPECTS[self.aspect];
        let k = if self.upscale_p(now) == Some(1.0) {
            2
        } else {
            1
        };
        format!("{} × {}", w * k, h * k)
    }

    /// The status pill text.
    pub fn status(&self, now: u64) -> String {
        if self.vars.is_empty() {
            return "Ready".into();
        }
        if let Some(p) = self.upscale_p(now)
            && p < 1.0
        {
            return format!("Upscaling {}%", (p * 100.0) as u32);
        }
        let done = (0..4)
            .filter(|&i| self.var_st(i, now) == VarSt::Done)
            .count();
        if self.busy(now) {
            let p: f64 = (0..4)
                .map(|i| match self.var_st(i, now) {
                    VarSt::Forming(p) => p,
                    VarSt::Empty => 0.0,
                    _ => 1.0,
                })
                .sum::<f64>()
                / 4.0;
            return format!("Generating {}%", (p * 100.0) as u32);
        }
        if self.stopped.is_some() {
            return format!("Stopped, {done} of 4 kept");
        }
        if (0..4).any(|i| self.var_st(i, now) == VarSt::Failed) {
            return format!("{done} of 4 done, 1 failed");
        }
        "4 variations ready".into()
    }

    /// The canvas label for assistive technology.
    pub fn canvas_label(&self, now: u64) -> String {
        match self.var_st(self.chosen, now) {
            VarSt::Done => format!(
                "{}, {}, variation {} of 4",
                self.prompt,
                SCENES[self.vars[self.chosen].scene].0,
                self.chosen + 1
            ),
            VarSt::Forming(p) => format!(
                "Variation {} generating, {}%",
                self.chosen + 1,
                (p * 100.0) as u32
            ),
            VarSt::Failed => format!("Variation {} failed", self.chosen + 1),
            VarSt::Stopped => format!("Variation {} stopped", self.chosen + 1),
            VarSt::Empty => "No image yet".into(),
        }
    }

    /// Presses: `ig-gen`, `ig-stop`, `ig-pick-<i>`, `ig-model-<i>`, `ig-aspect-<i>`,
    /// `ig-upscale`, `ig-vary`, `ig-download`, `ig-suggest-<i>`, `ig-prompt:<text>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(t) = id.strip_prefix("ig-prompt:") {
            self.prompt = t.to_string();
            return None;
        }
        if let Some(i) = nth(id, "ig-pick-") {
            if i < self.vars.len() {
                self.chosen = i;
                self.upscale = None;
                return Some(format!("Variation {} on the canvas", i + 1));
            }
            return None;
        }
        if let Some(i) = nth(id, "ig-model-") {
            self.model = i.min(MODELS.len() - 1);
            return Some(format!("Model {}", MODELS[self.model].0));
        }
        if let Some(i) = nth(id, "ig-aspect-") {
            self.aspect = i.min(ASPECTS.len() - 1);
            return Some(format!("Aspect {}", ASPECTS[self.aspect].0));
        }
        if let Some(i) = nth(id, "ig-suggest-") {
            self.prompt = SUGGESTIONS.get(i)?.to_string();
            self.generate(now);
            return Some(format!(
                "Generating 4 variations with {}, simulated",
                MODELS[self.model].0
            ));
        }
        match id {
            "ig-gen" => {
                if self.prompt.trim().is_empty() {
                    return Some("Describe the image first".into());
                }
                self.generate(now);
                Some(format!(
                    "Generating 4 variations with {}, simulated",
                    MODELS[self.model].0
                ))
            }
            "ig-stop" => {
                if self.busy(now) {
                    self.stopped = Some(now);
                    Some("Stopped, finished images are kept".into())
                } else {
                    None
                }
            }
            "ig-upscale" => {
                if self.var_st(self.chosen, now) == VarSt::Done && self.upscale.is_none() {
                    self.upscale = Some(now);
                    Some("Upscaling to 2x".into())
                } else {
                    None
                }
            }
            "ig-vary" => {
                if self.var_st(self.chosen, now) == VarSt::Done {
                    let keep = self.vars[self.chosen].scene;
                    self.generate(now);
                    for v in &mut self.vars {
                        v.scene = keep;
                    }
                    Some(format!("Making 4 variations of image {}", self.chosen + 1))
                } else {
                    None
                }
            }
            "ig-download" => (self.var_st(self.chosen, now) == VarSt::Done).then(|| {
                format!(
                    "Downloaded {}-{}.png, simulated",
                    self.prompt
                        .split_whitespace()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("-")
                        .to_lowercase(),
                    self.chosen + 1
                )
            }),
            _ => None,
        }
    }

    /// Keys: ←/→ in the filmstrip (`strip`), model (`model`) or aspect (`aspect`) chips;
    /// Cmd/Ctrl+Enter generates (`gen`); Escape stops.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        if key == "Escape" {
            return self.press("ig-stop", now);
        }
        match part {
            "strip" => {
                let i = rove(self.chosen, self.vars.len(), key);
                self.press(&format!("ig-pick-{i}"), now)
            }
            "model" => {
                let i = rove(self.model, MODELS.len(), key);
                self.press(&format!("ig-model-{i}"), now)
            }
            "aspect" => {
                let i = rove(self.aspect, ASPECTS.len(), key);
                self.press(&format!("ig-aspect-{i}"), now)
            }
            "gen" if key == "Enter" => self.press("ig-gen", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imagegen_forms_stops_upscales_and_varies() {
        let mut g = ImageGen::new(true, true, 0);
        assert!(matches!(g.var_st(0, 1_000), VarSt::Forming(_)));
        assert!(g.status(1_000).starts_with("Generating"));
        assert_eq!(g.var_st(2, 10_000), VarSt::Failed);
        assert_eq!(g.var_st(0, 10_000), VarSt::Done);
        assert!(g.status(10_000).contains("1 failed"));
        g.press("ig-upscale", 10_000);
        assert_eq!(g.size_label(20_000), "2048 × 2048");
        g.key("strip", "ArrowRight", 20_000);
        assert_eq!(g.chosen, 1);
        g.press("ig-vary", 20_000);
        assert_eq!(g.batches, 2);
        g.key("x", "Escape", 20_500);
        assert!(g.stopped.is_some());
        assert!(g.status(30_000).starts_with("Stopped"));
        g.press("ig-aspect-2", 0);
        assert_eq!(g.size_label(0), "1344 × 768");
        assert!(g.press("ig-download", 40_000).is_none() || g.var_st(0, 40_000) == VarSt::Done);
        let e = ImageGen::new(false, false, 0);
        assert_eq!(e.status(0), "Ready");
    }
}
