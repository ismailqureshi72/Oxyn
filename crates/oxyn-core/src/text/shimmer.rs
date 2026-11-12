//! `text-shimmer` (`docs/specs/text-shimmer.md`).

use std::time::Duration;

use oxyn_tokens::Color;

use super::{ease_out, timing};
use crate::{
    ClockMs,
    loader::TempoClock,
    state::{AgentState, StallLevel},
};

/// The four free shimmer variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShimmerVariant {
    /// Cosine band, 1.4 s sweep + 0.8 s rest.
    #[default]
    Sweep,
    /// One 1 s sweep every 4 s.
    Calm,
    /// Whole text breathes base ↔ crest on a 2 s sine (amplitude 0.6).
    Pulse,
    /// Terminal tiers (dim / normal / bold) from the same band, 2 s linear sweep.
    Ascii,
}

impl ShimmerVariant {
    /// Parses a variant id (`sweep` or `text-shimmer/sweep`).
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.strip_prefix("text-shimmer/").unwrap_or(id) {
            "sweep" => Self::Sweep,
            "calm" => Self::Calm,
            "pulse" => Self::Pulse,
            "ascii" => Self::Ascii,
            _ => return None,
        })
    }

    fn sweep(self, line_px: f32) -> Duration {
        match self {
            // 140 px/s above 200 px so long headers don't streak.
            Self::Sweep => timing::SHIMMER_SWEEP.max(Duration::from_millis(
                (line_px.max(0.0) / 140.0 * 1000.0) as u64,
            )),
            Self::Calm => Duration::from_millis(1000),
            Self::Pulse => Duration::from_millis(2000),
            Self::Ascii => Duration::from_millis(2000),
        }
    }

    fn rest(self) -> Duration {
        match self {
            Self::Sweep => timing::SHIMMER_REST,
            Self::Calm => Duration::from_millis(3000),
            Self::Pulse => Duration::ZERO,
            Self::Ascii => timing::SHIMMER_REST,
        }
    }
}

/// Terminal colour tier for one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsciiTier {
    /// Intensity < 0.2.
    Dim,
    /// < 0.6.
    Normal,
    /// ≥ 0.6.
    Bold,
}

impl AsciiTier {
    /// Tier for a band intensity.
    pub fn of(intensity: f32) -> Self {
        if intensity < 0.2 {
            Self::Dim
        } else if intensity < 0.6 {
            Self::Normal
        } else {
            Self::Bold
        }
    }
}

/// Largest dim factor `k ≤ k_default` such that `mix(fg → bg, k)` keeps `min_ratio` contrast
/// against `bg` (`text-shimmer.md` §3 contrast floor). Returns 0 when even `fg` fails, in which
/// case the shimmer must run with amplitude 0.
pub fn dim_factor(fg: Color, bg: Color, k_default: f32, min_ratio: f64) -> f32 {
    let mix = |k: f32| {
        let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * k).round() as u8;
        Color::rgb(ch(fg.r, bg.r), ch(fg.g, bg.g), ch(fg.b, bg.b))
    };
    if mix(k_default).contrast(bg) >= min_ratio {
        return k_default;
    }
    if fg.contrast(bg) < min_ratio {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0f32, k_default);
    for _ in 0..8 {
        let mid = (lo + hi) / 2.0;
        if mix(mid).contrast(bg) >= min_ratio {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// What a shimmer paints this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShimmerPaint {
    /// Plain text.
    None,
    /// A cosine band: `intensity(x) = amplitude · ½(1 + cos(π·|x − centre| / half_width))`.
    Band {
        /// Band centre on the line, px from the line start (may be off the line).
        centre: f32,
        /// Half width, px.
        half_width: f32,
        /// Amplitude `[0, 1]`.
        amplitude: f32,
    },
    /// The whole text at one intensity (pulse).
    Uniform(f32),
}

impl ShimmerPaint {
    /// Intensity at `x_px`.
    pub fn at(self, x_px: f32) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Uniform(i) => i,
            Self::Band {
                centre,
                half_width,
                amplitude,
            } => {
                let d = ((x_px - centre) / half_width).abs();
                if d >= 1.0 {
                    0.0
                } else {
                    amplitude * 0.5 * (1.0 + (std::f32::consts::PI * d).cos())
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Inactive,
    Running,
    /// Stopping: finish the current sweep if far enough across, else fade amplitude.
    Fading {
        at: ClockMs,
        finish_until: Option<ClockMs>,
    },
    /// Hard stall: no band, base toward warning.
    Frozen {
        at: ClockMs,
    },
}

/// A shimmer instance. Feed it the state and the text; sample it per glyph.
#[derive(Clone, Debug)]
pub struct Shimmer {
    variant: ShimmerVariant,
    text: String,
    text_beats: f64,
    text_at: ClockMs,
    tempo: TempoClock,
    mode: Mode,
    line_px: f32,
}

impl Shimmer {
    /// An inactive shimmer.
    pub fn new(variant: ShimmerVariant) -> Self {
        Self {
            variant,
            text: String::new(),
            text_beats: 0.0,
            text_at: ClockMs(0),
            tempo: TempoClock::new(1.0),
            mode: Mode::Inactive,
            line_px: 120.0,
        }
    }

    /// The variant.
    pub fn variant(&self) -> ShimmerVariant {
        self.variant
    }

    /// Sets the measured line width (px) used for sweep speed and band width.
    pub fn set_line_px(&mut self, px: f32) {
        self.line_px = px.max(1.0);
    }

    /// Feeds the shown text and the agent state. Call every frame; cheap when nothing changed.
    pub fn update(&mut self, text: &str, state: &AgentState, now: ClockMs) {
        if text != self.text {
            self.text = text.to_string();
            self.text_at = now;
            self.text_beats = self.tempo.beats(now);
        }
        let wants = matches!(
            state,
            AgentState::Thinking { .. }
                | AgentState::ToolRunning { .. }
                | AgentState::Streaming
                | AgentState::Queued { .. }
                | AgentState::Steering
                | AgentState::Stalled {
                    level: StallLevel::Soft,
                    ..
                }
        );
        let hard = matches!(
            state,
            AgentState::Stalled {
                level: StallLevel::Hard,
                ..
            }
        );
        let soft = matches!(
            state,
            AgentState::Stalled {
                level: StallLevel::Soft,
                ..
            }
        );
        let target = if soft { 0.5 } else { 1.0 };
        let ramp = if soft {
            Duration::from_millis(2000)
        } else {
            Duration::from_millis(200)
        };
        self.tempo.set_speed(target, now, ramp);
        self.mode = match (self.mode, wants, hard) {
            (Mode::Frozen { at }, false, true) => Mode::Frozen { at },
            (_, _, true) => Mode::Frozen { at: now },
            (Mode::Running, true, _) => Mode::Running,
            (_, true, _) => {
                if matches!(self.mode, Mode::Inactive) {
                    // A fresh run starts at Delay.
                    self.text_at = now;
                    self.text_beats = self.tempo.beats(now);
                }
                Mode::Running
            }
            (Mode::Running, false, _) => {
                let finish = self.band(now).filter(|&(p, _)| p >= 0.7).map(|(p, sweep)| {
                    ClockMs(now.0 + ((1.0 - p) * sweep.as_millis() as f32) as u64)
                });
                Mode::Fading {
                    at: now,
                    finish_until: finish,
                }
            }
            (m @ Mode::Fading { .. }, false, _) => m,
            (_, false, _) => Mode::Inactive,
        };
    }

    /// Band progress `[0, 1]` across the line and the sweep length, if a band is out now.
    fn band(&self, now: ClockMs) -> Option<(f32, Duration)> {
        let sweep = self.variant.sweep(self.line_px);
        let rest = self.variant.rest();
        let e = self.tempo.beats(now) - self.text_beats - timing::SHIMMER_DELAY.as_millis() as f64;
        if e < 0.0 {
            return None;
        }
        let cycle = (sweep + rest).as_millis().max(1) as f64;
        let c = e % cycle;
        let s = sweep.as_millis() as f64;
        (c < s).then(|| ((c / s) as f32, sweep))
    }

    fn amplitude(&self, now: ClockMs) -> f32 {
        match self.mode {
            Mode::Inactive | Mode::Frozen { .. } => 0.0,
            Mode::Running => 1.0,
            Mode::Fading {
                finish_until: Some(until),
                ..
            } => {
                if now.0 < until.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Mode::Fading {
                at,
                finish_until: None,
            } => 1.0 - ease_out(now.since(at).as_secs_f32() / 0.2),
        }
    }

    /// How far the base colour sits toward full fg: 0 = dimmed base, 1 = fg (inactive, static).
    pub fn base_level(&self, now: ClockMs, reduced: bool) -> f32 {
        if reduced {
            return 1.0;
        }
        match self.mode {
            Mode::Inactive => 1.0,
            Mode::Running | Mode::Frozen { .. } => 0.0,
            Mode::Fading { at, finish_until } => {
                let gone = finish_until.unwrap_or(ClockMs(at.0 + 200));
                if now.0 < gone.0 {
                    0.0
                } else {
                    ease_out(now.since(gone).as_secs_f32() / 0.2)
                }
            }
        }
    }

    /// Progress `[0, 1]` of the base toward the warning tone (hard stall).
    pub fn warn_level(&self, now: ClockMs) -> f32 {
        match self.mode {
            Mode::Frozen { at } => (now.since(at).as_secs_f32() / 2.0).min(1.0),
            _ => 0.0,
        }
    }

    /// What to paint now: a band across the line, a uniform level (pulse), or nothing.
    pub fn paint(&self, now: ClockMs, em_px: f32, reduced: bool) -> ShimmerPaint {
        let amp = self.amplitude(now);
        if reduced || amp <= 0.0 {
            return ShimmerPaint::None;
        }
        if self.variant == ShimmerVariant::Pulse {
            let e =
                self.tempo.beats(now) - self.text_beats - timing::SHIMMER_DELAY.as_millis() as f64;
            if e < 0.0 {
                return ShimmerPaint::None;
            }
            let x = (e / 2000.0) as f32 * std::f32::consts::TAU;
            return ShimmerPaint::Uniform(amp * 0.6 * 0.5 * (1.0 - x.cos()));
        }
        let Some((p, _)) = self.band(now) else {
            return ShimmerPaint::None;
        };
        let line = self.line_px;
        let hw = (0.12 * line)
            .max(1.2 * em_px)
            .min(0.5 * line.max(2.4 * em_px));
        let pad = 0.5 * em_px + hw;
        let eased = if self.variant == ShimmerVariant::Ascii {
            p
        } else {
            coast(p)
        };
        ShimmerPaint::Band {
            centre: -pad + eased * (line + 2.0 * pad),
            half_width: hw,
            amplitude: amp,
        }
    }

    /// Crest intensity `[0, 1]` for a glyph whose horizontal centre is at `x_px` on the line.
    /// Colour = mix(base, crest, intensity). Under reduced motion there is no band.
    pub fn intensity(&self, now: ClockMs, x_px: f32, em_px: f32, reduced: bool) -> f32 {
        self.paint(now, em_px, reduced).at(x_px)
    }

    /// True while the shimmer asks for repaints (≈ 30 Hz); false when idle between sweeps or off.
    pub fn animating(&self, now: ClockMs, reduced: bool) -> bool {
        if reduced {
            return false;
        }
        match self.mode {
            Mode::Inactive => false,
            Mode::Frozen { at } => now.since(at) < Duration::from_secs(2),
            Mode::Fading { at, finish_until } => {
                let gone = finish_until.unwrap_or(ClockMs(at.0 + 200));
                now.0 < gone.0 + 200
            }
            Mode::Running => self.variant == ShimmerVariant::Pulse || self.band(now).is_some(),
        }
    }

    /// True while the host should mark itself `aria-busy`.
    pub fn busy(&self) -> bool {
        self.mode == Mode::Running
    }
}

/// `ease.coast`: quadratic ramps on the first and last 20 %, linear between (OpenCode, MIT).
fn coast(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    // Velocity ramps 0→v over 0.2, flat, v→0 over the last 0.2: v = 1 / 0.8.
    let v = 1.0 / 0.8;
    if p < 0.2 {
        v * p * p / 0.4
    } else if p < 0.8 {
        v * (0.1 + (p - 0.2))
    } else {
        let q = 1.0 - p;
        1.0 - v * q * q / 0.4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thinking() -> AgentState {
        AgentState::Thinking { header: None }
    }

    #[test]
    fn coast_is_continuous_and_ends_at_one() {
        assert!((coast(1.0) - 1.0).abs() < 1e-6 && coast(0.0) == 0.0);
        assert!((coast(0.2) - coast(0.2001)).abs() < 1e-3);
        assert!((coast(0.8) - coast(0.7999)).abs() < 1e-3);
    }

    #[test]
    fn no_band_during_delay_then_a_band_crosses() {
        let mut s = Shimmer::new(ShimmerVariant::Sweep);
        s.update("Thinking", &thinking(), ClockMs(0));
        assert_eq!(s.intensity(ClockMs(500), 60.0, 13.0, false), 0.0);
        let peak = (600..2000)
            .step_by(10)
            .map(|t| s.intensity(ClockMs(t), 60.0, 13.0, false))
            .fold(0.0f32, f32::max);
        assert!(peak > 0.95);
        // Rest after the sweep.
        assert_eq!(
            s.intensity(ClockMs(600 + 1400 + 400), 60.0, 13.0, false),
            0.0
        );
        assert!(s.intensity(ClockMs(1300), 60.0, 13.0, true) == 0.0);
    }

    #[test]
    fn text_change_restarts_the_delay() {
        let mut s = Shimmer::new(ShimmerVariant::Sweep);
        s.update("Thinking", &thinking(), ClockMs(0));
        s.update("Reading a.rs", &thinking(), ClockMs(1300));
        assert_eq!(s.intensity(ClockMs(1700), 60.0, 13.0, false), 0.0);
    }

    #[test]
    fn stop_fades_and_base_returns_to_fg() {
        let mut s = Shimmer::new(ShimmerVariant::Sweep);
        s.update("Thinking", &thinking(), ClockMs(0));
        // Early in the sweep (< 70 %): amplitude fades over 200 ms.
        let done = AgentState::Done {
            elapsed: Duration::from_secs(2),
            waited: Duration::ZERO,
        };
        s.update("Worked for 2s", &done, ClockMs(800));
        assert!(s.base_level(ClockMs(900), false) == 0.0);
        assert!(s.base_level(ClockMs(1300), false) > 0.99);
    }

    #[test]
    fn contrast_floor_holds() {
        let fg = Color::rgb(0xEE, 0xEE, 0xEE);
        let bg = Color::rgb(0x0B, 0x0B, 0x0C);
        let k = dim_factor(fg, bg, 0.4, 4.5);
        assert!(k > 0.0 && k <= 0.4);
        let grey = Color::rgb(0x55, 0x55, 0x55);
        assert_eq!(dim_factor(grey, bg, 0.4, 4.5), 0.0);
    }
}
