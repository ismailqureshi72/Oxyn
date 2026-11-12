//! `text-reveal` and `caret` (`docs/specs/text-reveal.md`): a two-gear pacer plus per-unit
//! effects. Units are laid out at their final positions; effects only touch opacity, blur and
//! small y offsets, so revealing never reflows text (typewriter and line-commit append).

use std::time::Duration;

use unicode_segmentation::UnicodeSegmentation;

use super::{ease_out, timing};
use crate::ClockMs;

/// The five free reveal variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RevealVariant {
    /// Words fade in over 160 ms with a 25 ms stagger.
    #[default]
    FadeToken,
    /// Words sharpen from a 6 px blur with a 3 px lift (180 ms).
    WordBlur,
    /// Graphemes appear at stream pace, no fade.
    Typewriter,
    /// Whole lines land with an 80 ms fade.
    LineCommit,
    /// Pacing only, no effect.
    Instant,
}

impl RevealVariant {
    /// Parses a variant id (`fade-token` or `text-reveal/fade-token`).
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.strip_prefix("text-reveal/").unwrap_or(id) {
            "fade-token" => Self::FadeToken,
            "word-blur" => Self::WordBlur,
            "typewriter" => Self::Typewriter,
            "line-commit" => Self::LineCommit,
            "instant" => Self::Instant,
            _ => return None,
        })
    }

    fn effect_ms(self) -> u64 {
        match self {
            Self::FadeToken => timing::REVEAL_TOKEN.as_millis() as u64,
            Self::WordBlur => 180,
            Self::LineCommit => 80,
            Self::Typewriter | Self::Instant => 0,
        }
    }

    fn staggers(self) -> bool {
        matches!(self, Self::FadeToken | Self::WordBlur)
    }
}

/// Pacer gear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gear {
    /// Unit by unit at `max(40 g/s, buffer / 250 ms)`.
    Smooth,
    /// Whole buffer per frame with a 120 ms block fade.
    CatchUp,
}

/// Pacer numbers (`text-reveal.md` §3).
mod pace {
    pub const LAG_S: f64 = 0.25;
    pub const MIN_RATE: f64 = 40.0;
    pub const CATCHUP_BUFFER: usize = 600;
    pub const CATCHUP_AGE_MS: u64 = 600;
    pub const EXIT_BUFFER: usize = 60;
    pub const EXIT_AGE_MS: u64 = 100;
    pub const EXIT_HOLD_MS: u64 = 250;
    pub const DRAIN_MS: u64 = 300;
    pub const BURST: usize = 400;
    pub const BLOCK_FADE_MS: u64 = 120;
    pub const STAGGER_MS: u64 = 25;
    pub const STAGGER_CAP_MS: u64 = 300;
    pub const IN_FLIGHT: usize = 64;
}

#[derive(Clone, Debug)]
struct Unit {
    text: String,
    graphemes: usize,
    arrived: ClockMs,
    /// When it became visible, its stagger, and whether it came in a block (catch-up, burst).
    shown: Option<(ClockMs, u64, bool)>,
}

/// One run of text to draw with one style.
#[derive(Clone, Debug, PartialEq)]
pub struct RevealSpan {
    /// The text.
    pub text: String,
    /// Opacity `[0, 1]`.
    pub opacity: f32,
    /// Blur in px.
    pub blur_px: f32,
    /// Y offset in px (positive = below its final position).
    pub dy_px: f32,
}

/// The three free carets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaretVariant {
    /// 2 px × 1.1 em bar.
    #[default]
    Bar,
    /// 0.55 em × 1.1 em block at 70 % (terminal `█`).
    Block,
    /// 0.55 em × 2 px on the baseline.
    Underscore,
}

impl CaretVariant {
    /// Parses a variant id (`bar` or `caret/bar`).
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.strip_prefix("caret/").unwrap_or(id) {
            "bar" => Self::Bar,
            "block" => Self::Block,
            "underscore" => Self::Underscore,
            _ => return None,
        })
    }
}

/// Caret state at a moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caret {
    /// Opacity to draw it with (0 = not drawn this frame).
    pub opacity: f32,
    /// Stalled: dimmed to the 3:1 non-text floor (host picks the colour).
    pub dim: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Open,
    Ended(ClockMs),
    Interrupted,
    Failed,
}

/// A streaming reveal: push chunks as they arrive, `tick` each frame, draw `spans`.
#[derive(Clone, Debug)]
pub struct Reveal {
    variant: RevealVariant,
    units: Vec<Unit>,
    pending: String,
    pending_at: ClockMs,
    revealed_units: usize,
    budget: f64,
    gear: Gear,
    calm_since: Option<ClockMs>,
    gear_at: Option<ClockMs>,
    last_tick: ClockMs,
    last_reveal: Option<ClockMs>,
    end: End,
    drained_at: Option<ClockMs>,
    stalled: bool,
}

impl Reveal {
    /// An empty reveal (`Waiting`: the caret blinks at the insertion point).
    pub fn new(variant: RevealVariant) -> Self {
        Self {
            variant,
            units: Vec::new(),
            pending: String::new(),
            pending_at: ClockMs(0),
            revealed_units: 0,
            budget: 0.0,
            gear: Gear::Smooth,
            calm_since: None,
            gear_at: None,
            last_tick: ClockMs(0),
            last_reveal: None,
            end: End::Open,
            drained_at: None,
            stalled: false,
        }
    }

    /// The variant.
    pub fn variant(&self) -> RevealVariant {
        self.variant
    }

    /// Current gear.
    pub fn gear(&self) -> Gear {
        self.gear
    }

    /// Everything received so far.
    pub fn received(&self) -> String {
        let mut s: String = self.units.iter().map(|u| u.text.as_str()).collect();
        s.push_str(&self.pending);
        s
    }

    fn split(&mut self, now: ClockMs, flush: bool) {
        let text = std::mem::take(&mut self.pending);
        let mut rest = text.as_str();
        let push = |t: &str, units: &mut Vec<Unit>| {
            units.push(Unit {
                text: t.to_string(),
                graphemes: t.graphemes(true).count(),
                arrived: now,
                shown: None,
            });
        };
        match self.variant {
            RevealVariant::Typewriter => {
                for g in rest.graphemes(true) {
                    push(g, &mut self.units);
                }
                rest = "";
            }
            RevealVariant::LineCommit => {
                while let Some(i) = rest.find('\n') {
                    push(&rest[..=i], &mut self.units);
                    rest = &rest[i + 1..];
                }
            }
            _ => {
                // A word is its letters plus the whitespace after them (UAX #29 boundaries).
                let mut start = 0;
                let mut after_space = false;
                for (i, w) in rest.split_word_bound_indices() {
                    let space = w.chars().all(char::is_whitespace);
                    if !space && after_space && i > start {
                        push(&rest[start..i], &mut self.units);
                        start = i;
                    }
                    after_space = space;
                }
                rest = &rest[start..];
                // The last word may still be growing unless whitespace closed it.
                if after_space && !rest.trim().is_empty() {
                    push(rest, &mut self.units);
                    rest = "";
                }
            }
        }
        if flush && !rest.is_empty() {
            push(rest, &mut self.units);
            rest = "";
        }
        self.pending = rest.to_string();
    }

    /// Appends a chunk from the stream.
    pub fn push(&mut self, chunk: &str, now: ClockMs) {
        if self.end != End::Open {
            return;
        }
        if self.pending.is_empty() {
            self.pending_at = now;
        }
        let first = self.units.is_empty() && self.pending.is_empty();
        self.pending.push_str(chunk);
        self.split(now, false);
        // Burst at start: reveal the first 400 graphemes at once with a block fade.
        if first {
            let mut g = 0;
            let mut n = 0;
            for u in &self.units {
                if g + u.graphemes > pace::BURST {
                    break;
                }
                g += u.graphemes;
                n += 1;
            }
            if g >= pace::BURST {
                self.show_units(n, now, true);
            }
        }
    }

    /// The stream ended normally: drain the rest within 300 ms.
    pub fn finish(&mut self, now: ClockMs) {
        if self.end == End::Open {
            self.split(now, true);
            self.end = End::Ended(now);
        }
    }

    /// Stop: everything received appears at once, no animation, caret removed.
    pub fn interrupt(&mut self, now: ClockMs) {
        self.split(now, true);
        for u in &mut self.units[self.revealed_units..] {
            u.shown = None;
        }
        self.revealed_units = self.units.len();
        self.end = End::Interrupted;
    }

    /// Error: partial text kept, caret removed.
    pub fn fail(&mut self, now: ClockMs) {
        self.interrupt(now);
        self.end = End::Failed;
    }

    /// The stream is stalled (status-text owns the words); the caret dims.
    pub fn set_stalled(&mut self, stalled: bool) {
        self.stalled = stalled;
    }

    fn show_units(&mut self, upto: usize, now: ClockMs, block: bool) {
        let from = self.revealed_units;
        for (k, u) in self.units[from..upto].iter_mut().enumerate() {
            let stagger = if block || !self.variant.staggers() {
                0
            } else {
                (k as u64 * pace::STAGGER_MS).min(pace::STAGGER_CAP_MS)
            };
            u.shown = Some((now, stagger, block));
        }
        if upto > from {
            self.revealed_units = upto;
            self.last_reveal = Some(now);
        }
    }

    fn buffer(&self) -> usize {
        self.units[self.revealed_units..]
            .iter()
            .map(|u| u.graphemes)
            .sum()
    }

    /// Advances the pacer. Call once per frame.
    pub fn tick(&mut self, now: ClockMs) {
        let dt = now.since(self.last_tick).as_secs_f64().min(0.25);
        self.last_tick = now;
        if matches!(self.end, End::Interrupted | End::Failed) {
            return;
        }
        let buffer = self.buffer();
        let age = self
            .units
            .get(self.revealed_units)
            .map_or(0, |u| now.since(u.arrived).as_millis() as u64);
        // Gear changes with hysteresis.
        match self.gear {
            Gear::Smooth if buffer >= pace::CATCHUP_BUFFER || age >= pace::CATCHUP_AGE_MS => {
                let reentry_ok = self
                    .gear_at
                    .is_none_or(|g| now.since(g).as_millis() as u64 >= pace::EXIT_HOLD_MS)
                    || buffer >= 4000
                    || age >= 1500;
                if reentry_ok {
                    self.gear = Gear::CatchUp;
                    self.gear_at = Some(now);
                }
            }
            Gear::CatchUp => {
                if buffer <= pace::EXIT_BUFFER && age <= pace::EXIT_AGE_MS {
                    let since = *self.calm_since.get_or_insert(now);
                    if now.since(since).as_millis() as u64 >= pace::EXIT_HOLD_MS {
                        self.gear = Gear::Smooth;
                        self.gear_at = Some(now);
                        self.calm_since = None;
                    }
                } else {
                    self.calm_since = None;
                }
            }
            Gear::Smooth => {}
        }
        if buffer == 0 {
            if let End::Ended(_) = self.end {
                self.drained_at.get_or_insert(now);
            }
            self.budget = 0.0;
            return;
        }
        if self.variant == RevealVariant::LineCommit {
            // Whole lines land as soon as they are complete.
            let n = self.units.len();
            self.show_units(n, now, false);
            return;
        }
        if self.gear == Gear::CatchUp {
            let n = self.units.len();
            self.show_units(n, now, true);
            return;
        }
        let mut rate = pace::MIN_RATE.max(buffer as f64 / pace::LAG_S);
        if let End::Ended(at) = self.end {
            let left = pace::DRAIN_MS
                .saturating_sub(now.since(at).as_millis() as u64)
                .max(16);
            rate = rate.max(buffer as f64 / (left as f64 / 1000.0));
        }
        self.budget += rate * dt;
        let mut upto = self.revealed_units;
        while let Some(u) = self.units.get(upto) {
            if (u.graphemes as f64) > self.budget {
                break;
            }
            self.budget -= u.graphemes as f64;
            upto += 1;
        }
        self.show_units(upto, now, false);
    }

    /// The text to draw: settled text merged into plain spans, in-flight units styled.
    /// Unrevealed text is not included (it is not laid out yet).
    pub fn spans(&self, now: ClockMs, reduced: bool) -> Vec<RevealSpan> {
        let mut out: Vec<RevealSpan> = Vec::new();
        let shown = &self.units[..self.revealed_units];
        let in_flight_from = shown.len().saturating_sub(pace::IN_FLIGHT);
        for (i, u) in shown.iter().enumerate() {
            let (op, blur, dy) = match u.shown {
                Some((at, stagger, block)) if i >= in_flight_from => {
                    let e = now.since(at).as_millis() as u64;
                    let e = e.saturating_sub(stagger) as f32;
                    if block {
                        (ease_out(e / pace::BLOCK_FADE_MS as f32), 0.0, 0.0)
                    } else if reduced {
                        ((e / 80.0).min(1.0), 0.0, 0.0)
                    } else {
                        let d = self.variant.effect_ms() as f32;
                        if d == 0.0 {
                            (1.0, 0.0, 0.0)
                        } else {
                            let t = if self.variant == RevealVariant::LineCommit {
                                (e / d).min(1.0)
                            } else {
                                ease_out(e / d)
                            };
                            match self.variant {
                                RevealVariant::WordBlur => (t, 6.0 * (1.0 - t), 3.0 * (1.0 - t)),
                                _ => (t, 0.0, 0.0),
                            }
                        }
                    }
                }
                _ => (1.0, 0.0, 0.0),
            };
            match out.last_mut() {
                Some(last)
                    if op >= 1.0
                        && last.opacity >= 1.0
                        && last.blur_px == 0.0
                        && last.dy_px == 0.0
                        && blur == 0.0 =>
                {
                    last.text.push_str(&u.text);
                }
                _ => out.push(RevealSpan {
                    text: u.text.clone(),
                    opacity: op,
                    blur_px: blur,
                    dy_px: dy,
                }),
            }
        }
        out
    }

    /// The caret after the last revealed grapheme.
    pub fn caret(&self, now: ClockMs, reduced: bool) -> Caret {
        let hidden = Caret {
            opacity: 0.0,
            dim: false,
        };
        match self.end {
            End::Interrupted | End::Failed => return hidden,
            End::Ended(_) => {
                if let Some(d) = self.drained_at {
                    let t = now.since(d).as_secs_f32() / 0.2;
                    return Caret {
                        opacity: 1.0 - ease_out(t),
                        dim: false,
                    };
                }
            }
            End::Open => {}
        }
        if self.stalled {
            return Caret {
                opacity: 1.0,
                dim: true,
            };
        }
        let solid = Caret {
            opacity: 1.0,
            dim: false,
        };
        if reduced {
            return solid;
        }
        let since = self
            .last_reveal
            .map_or(now.since(self.pending_at), |r| now.since(r));
        if since < Duration::from_millis(500) || since >= Duration::from_secs(5) {
            // Solid while text arrives; stops blinking after 5 s of idle (WCAG 2.2.2).
            return solid;
        }
        let on = (since.as_millis() / 500) % 2 == 1;
        Caret {
            opacity: if on { 1.0 } else { 0.0 },
            dim: false,
        }
    }

    /// True while anything still moves (pacer, effects, caret blink, caret fade).
    pub fn animating(&self, now: ClockMs) -> bool {
        if self.buffer() > 0 {
            return true;
        }
        let effect = Duration::from_millis(
            self.variant.effect_ms().max(pace::BLOCK_FADE_MS) + pace::STAGGER_CAP_MS,
        );
        let recent = self.last_reveal.is_some_and(|r| now.since(r) < effect);
        let blinking = self.end == End::Open
            && self
                .last_reveal
                .is_some_and(|r| now.since(r) < Duration::from_secs(5));
        let fading = self
            .drained_at
            .is_some_and(|d| now.since(d) < Duration::from_millis(200));
        recent || blinking || fading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(r: &mut Reveal, from: u64, to: u64) {
        for t in (from..=to).step_by(16) {
            r.tick(ClockMs(t));
        }
    }

    #[test]
    fn words_split_and_reveal_without_losing_text() {
        let mut r = Reveal::new(RevealVariant::FadeToken);
        r.push("Hello wor", ClockMs(0));
        r.push("ld, this is Oxyn.", ClockMs(50));
        r.finish(ClockMs(60));
        run(&mut r, 0, 1000);
        let text: String = r
            .spans(ClockMs(1000), false)
            .iter()
            .map(|s| s.text.clone())
            .collect();
        assert_eq!(text, "Hello world, this is Oxyn.");
        assert_eq!(
            r.spans(ClockMs(1000), false).len(),
            1,
            "settled spans are unwrapped"
        );
        assert_eq!(r.caret(ClockMs(1000), false).opacity, 0.0);
    }

    #[test]
    fn pacer_never_hides_text_longer_than_max_lag() {
        let mut r = Reveal::new(RevealVariant::Typewriter);
        r.push(&"a".repeat(300), ClockMs(0));
        run(&mut r, 0, 620);
        assert_eq!(r.buffer(), 0);
    }

    #[test]
    fn big_buffers_shift_to_catch_up_and_back() {
        let mut r = Reveal::new(RevealVariant::FadeToken);
        r.push("x ", ClockMs(0));
        run(&mut r, 0, 100);
        r.push(&"word ".repeat(200), ClockMs(100));
        r.tick(ClockMs(116));
        assert_eq!(r.gear(), Gear::CatchUp);
        run(&mut r, 116, 800);
        assert_eq!(r.gear(), Gear::Smooth);
    }

    #[test]
    fn line_commit_waits_for_the_newline() {
        let mut r = Reveal::new(RevealVariant::LineCommit);
        r.push("fn main() {", ClockMs(0));
        run(&mut r, 0, 200);
        assert!(r.spans(ClockMs(200), false).is_empty());
        r.push("\n", ClockMs(200));
        run(&mut r, 200, 400);
        assert_eq!(r.spans(ClockMs(400), false)[0].text, "fn main() {\n");
    }

    #[test]
    fn interrupt_shows_everything_at_once_and_removes_the_caret() {
        let mut r = Reveal::new(RevealVariant::WordBlur);
        r.push("one two three", ClockMs(0));
        r.interrupt(ClockMs(10));
        let s = r.spans(ClockMs(10), false);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "one two three");
        assert_eq!(r.caret(ClockMs(10), false).opacity, 0.0);
    }

    #[test]
    fn caret_blinks_then_goes_solid() {
        let mut r = Reveal::new(RevealVariant::FadeToken);
        r.push("hi ", ClockMs(0));
        run(&mut r, 0, 100);
        let blink: Vec<f32> = (0..10)
            .map(|k| r.caret(ClockMs(700 + k * 250), false).opacity)
            .collect();
        assert!(blink.contains(&0.0) && blink.contains(&1.0));
        assert_eq!(r.caret(ClockMs(6000), false).opacity, 1.0);
        assert_eq!(r.caret(ClockMs(700), true).opacity, 1.0);
    }
}
