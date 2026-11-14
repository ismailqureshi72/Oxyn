//! Skeletons, the shimmer loader and the skeleton morph (`skeletons.md`, `skeleton-morph.md`).
//!
//! - **Skeleton** (affectUI Skeleton, Arc UI Skeleton): grey shapes the size of what they stand in
//!   for. affectUI: one soft sheen sweeps across every skeleton in step (1.6 s), and a `Loading`
//!   wrapper only shows them after 150 ms so a fast load never flashes grey. Arc: lines (1–6) and an
//!   optional avatar that pulse in a staggered wave and crossfade into the content.
//! - **Shimmer loader** (affectUI Motion): the sheen is pinned to the viewport so all blocks share
//!   one sweep, plus shimmering text for "Thinking…".
//! - **Skeleton morph** (Arc UI Skeleton morph): blocks resolve one after another (40 ms apart);
//!   each placeholder stretches from its skeleton box to the content's box on a critically damped
//!   spring while it fades, and the content sharpens out of a 4 px blur. Going back to loading is
//!   immediate.
//!
//! Oxyn plus: a sweep style picker (shimmer, pulse, wave, none), sweep angle and speed, shape
//! presets for agent UIs (message, tool row, diff, card, table, form) and a cap on the stagger.
//! Rebuilt from the docs; no source used.

use std::time::Duration;

use crate::ClockMs;

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Shared sheen period (affectUI 1.6 s).
    pub const SHEEN: Duration = Duration::from_millis(1600);
    /// Reveal delay before skeletons show (affectUI 150 ms).
    pub const DELAY: Duration = oxyn_tokens::motion::delay::SKELETON;
    /// Arc pulse wave: one beat per row.
    pub const WAVE_STEP: Duration = Duration::from_millis(120);
    /// Pulse period.
    pub const PULSE: Duration = Duration::from_millis(1400);
    /// Morph stagger (Arc 40 ms).
    pub const MORPH_STAGGER: Duration = Duration::from_millis(40);
    /// Critically damped spring natural frequency (rad/s): settles ~95 % in ≈ 240 ms.
    pub const MORPH_OMEGA: f32 = 20.0;
    /// Content rises this far into place on load (Arc 4 px).
    pub const RISE: f32 = 4.0;
}

/// How the placeholders move.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Sweep {
    /// One shared sheen (affectUI).
    #[default]
    Shimmer,
    /// Opacity pulse (Arc).
    Pulse,
    /// Pulse in a staggered wave, a beat per row (Arc).
    Wave,
    /// Static.
    None,
}

impl Sweep {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "pulse" => Self::Pulse,
            "wave" => Self::Wave,
            "none" => Self::None,
            _ => Self::Shimmer,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Shimmer => "shimmer",
            Self::Pulse => "pulse",
            Self::Wave => "wave",
            Self::None => "none",
        }
    }
}

/// One placeholder block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bone {
    /// Width as a fraction of the row (or px when `px`).
    pub w: f32,
    /// Height px.
    pub h: f32,
    /// Corner radius px (`f32::INFINITY` = circle).
    pub r: f32,
    /// `w` is px rather than a fraction.
    pub px: bool,
}

impl Bone {
    /// A fractional-width bar.
    pub const fn bar(w: f32, h: f32) -> Self {
        Self {
            w,
            h,
            r: 6.0,
            px: false,
        }
    }
    /// A fixed block.
    pub const fn block(w: f32, h: f32, r: f32) -> Self {
        Self { w, h, r, px: true }
    }
    /// A circle.
    pub const fn circle(d: f32) -> Self {
        Self {
            w: d,
            h: d,
            r: f32::INFINITY,
            px: true,
        }
    }
}

/// Line widths for `lines` text lines (Arc clamps 1–6; the last of several is shorter).
pub fn line_widths(lines: usize) -> Vec<f32> {
    const W: [f32; 6] = [1.0, 0.94, 0.88, 0.97, 0.9, 0.92];
    let n = lines.clamp(1, 6);
    let mut v: Vec<f32> = W[..n].to_vec();
    if n > 1 {
        v[n - 1] = 0.62;
    }
    v
}

/// Oxyn plus: agent-UI shape presets, as rows of bones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Shape {
    /// Arc: text lines (+ avatar).
    #[default]
    Text,
    /// A chat message: avatar, name, three lines.
    Message,
    /// A tool-call row: icon, name, status pill.
    ToolRow,
    /// A diff hunk: header and +/- lines.
    Diff,
    /// A media card: image, title, two lines, byline.
    Card,
    /// A table: header and rows.
    Table,
    /// A form: label + field pairs and a button.
    Form,
}

impl Shape {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "message" => Self::Message,
            "tool-row" => Self::ToolRow,
            "diff" => Self::Diff,
            "card" => Self::Card,
            "table" => Self::Table,
            "form" => Self::Form,
            _ => Self::Text,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Message => "message",
            Self::ToolRow => "tool-row",
            Self::Diff => "diff",
            Self::Card => "card",
            Self::Table => "table",
            Self::Form => "form",
        }
    }
    /// Rows of bones for this shape with `lines` text lines.
    pub fn rows(self, lines: usize, avatar: bool) -> Vec<Vec<Bone>> {
        let text = |n: usize| -> Vec<Vec<Bone>> {
            line_widths(n)
                .into_iter()
                .map(|w| vec![Bone::bar(w, 12.0)])
                .collect()
        };
        match self {
            Self::Text => {
                let mut r = text(lines);
                if avatar {
                    r.insert(0, vec![Bone::circle(32.0), Bone::bar(0.4, 12.0)]);
                }
                r
            }
            Self::Message => {
                let mut r = vec![vec![Bone::circle(28.0), Bone::bar(0.28, 11.0)]];
                r.extend(text(lines.max(2)));
                r
            }
            Self::ToolRow => (0..lines.clamp(1, 6))
                .map(|i| {
                    vec![
                        Bone::block(16.0, 16.0, 4.0),
                        Bone::bar(0.3 + 0.08 * (i % 3) as f32, 11.0),
                        Bone::block(54.0, 18.0, 9.0),
                    ]
                })
                .collect(),
            Self::Diff => {
                let mut r = vec![vec![Bone::block(16.0, 12.0, 3.0), Bone::bar(0.46, 12.0)]];
                for i in 0..lines.clamp(2, 6) {
                    r.push(vec![
                        Bone::block(10.0, 10.0, 2.0),
                        Bone::bar([0.72, 0.58, 0.81, 0.4, 0.66, 0.5][i], 10.0),
                    ]);
                }
                r
            }
            Self::Card => {
                let mut r = vec![vec![Bone::bar(1.0, 120.0)], vec![Bone::bar(0.6, 14.0)]];
                r.extend(text(2));
                r.push(vec![Bone::circle(20.0), Bone::bar(0.32, 10.0)]);
                r
            }
            Self::Table => {
                let mut r = vec![vec![
                    Bone::bar(0.22, 10.0),
                    Bone::bar(0.3, 10.0),
                    Bone::bar(0.18, 10.0),
                ]];
                for _ in 0..lines.clamp(2, 6) {
                    r.push(vec![
                        Bone::bar(0.26, 12.0),
                        Bone::bar(0.34, 12.0),
                        Bone::bar(0.14, 12.0),
                    ]);
                }
                r
            }
            Self::Form => {
                let mut r = vec![];
                for _ in 0..lines.clamp(1, 4) {
                    r.push(vec![Bone::bar(0.24, 10.0)]);
                    r.push(vec![Bone::bar(1.0, 32.0)]);
                }
                r.push(vec![Bone::block(96.0, 32.0, 8.0)]);
                r
            }
        }
    }
}

/// Wave delay for row `i` (Arc: each a beat after the one above), capped (Oxyn plus).
pub fn wave_delay(i: usize, step: Duration, cap: Duration) -> Duration {
    (step * i as u32).min(cap)
}

/// Phase of the shared sheen at `now` (0 → 1), same for every block on screen.
pub fn sheen_phase(now: ClockMs, period: Duration) -> f32 {
    let p = period.as_millis().max(1) as u64;
    (now.0 % p) as f32 / p as f32
}

/// affectUI's `Loading` wrapper: skeletons only after a short delay, then content fades in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadingGate {
    /// Started loading.
    pub started: ClockMs,
    /// Finished (None while loading).
    pub done: Option<ClockMs>,
    /// Reveal delay.
    pub delay: Duration,
}

/// What the gate shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    /// Nothing yet (inside the delay).
    Blank,
    /// The skeletons.
    Skeleton,
    /// The content.
    Content,
}

impl LoadingGate {
    /// Loading from `now`.
    pub fn start(now: ClockMs) -> Self {
        Self {
            started: now,
            done: None,
            delay: timing::DELAY,
        }
    }
    /// Loaded at `now`.
    pub fn finish(&mut self, now: ClockMs) {
        self.done.get_or_insert(now);
    }
    /// Back to loading (immediate).
    pub fn restart(&mut self, now: ClockMs) {
        *self = Self {
            delay: self.delay,
            ..Self::start(now)
        };
    }
    /// What shows at `now`.
    pub fn show(&self, now: ClockMs) -> Gate {
        match self.done {
            Some(_) => Gate::Content,
            None if now.since(self.started) < self.delay => Gate::Blank,
            None => Gate::Skeleton,
        }
    }
    /// The skeletons were ever visible (a fast load skips them entirely).
    pub fn flashed(&self) -> bool {
        self.done
            .is_none_or(|d| d.since(self.started) >= self.delay)
    }
}

/// Critically damped spring progress at `t` seconds (0 → 1, no overshoot).
pub fn critical(t: f32, omega: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    let x = omega * t;
    (1.0 - (1.0 + x) * (-x).exp()).clamp(0.0, 1.0)
}

/// Skeleton morph (Arc UI): blocks resolve in reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Morph {
    /// Stagger between blocks.
    pub stagger: Duration,
    /// Oxyn plus: the total stagger never exceeds this.
    pub cap: Duration,
    /// When loading ended (`None` = loading).
    pub resolved_at: Option<ClockMs>,
}

impl Default for Morph {
    fn default() -> Self {
        Self {
            stagger: timing::MORPH_STAGGER,
            cap: Duration::from_millis(480),
            resolved_at: None,
        }
    }
}

impl Morph {
    /// Loading ended.
    pub fn resolve(&mut self, now: ClockMs) {
        self.resolved_at.get_or_insert(now);
    }
    /// Back to loading: immediate, nothing stuck halfway.
    pub fn reload(&mut self) {
        self.resolved_at = None;
    }
    /// Loading.
    pub fn loading(&self) -> bool {
        self.resolved_at.is_none()
    }
    /// Start delay of block `i` of `n` (stagger, compressed to the cap).
    pub fn delay(&self, i: usize, n: usize) -> Duration {
        let total = self.stagger * n.saturating_sub(1) as u32;
        if total > self.cap && n > 1 {
            self.cap * i as u32 / (n - 1) as u32
        } else {
            self.stagger * i as u32
        }
    }
    /// Progress of block `i` of `n` at `now` (0 = skeleton, 1 = content).
    pub fn progress(&self, i: usize, n: usize, now: ClockMs, reduced: bool) -> f32 {
        let Some(r) = self.resolved_at else {
            return 0.0;
        };
        let t = now.since(r).saturating_sub(self.delay(i, n)).as_secs_f32();
        if reduced {
            (t / 0.12).clamp(0.0, 1.0)
        } else {
            critical(t, timing::MORPH_OMEGA)
        }
    }
    /// Box of the placeholder while it stretches from `from` (w, h) to `to` (w, h).
    pub fn stretch(from: (f32, f32), to: (f32, f32), p: f32) -> (f32, f32) {
        (from.0 + (to.0 - from.0) * p, from.1 + (to.1 - from.1) * p)
    }
    /// Every block has settled.
    pub fn settled(&self, n: usize, now: ClockMs) -> bool {
        n == 0 || self.progress(n - 1, n, now, false) > 0.995
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_and_shapes() {
        assert_eq!(line_widths(0).len(), 1);
        assert_eq!(line_widths(9).len(), 6);
        assert_eq!(*line_widths(3).last().unwrap(), 0.62);
        assert_eq!(line_widths(1), vec![1.0]);
        assert_eq!(Shape::Text.rows(3, true).len(), 4);
        assert!(Shape::Message.rows(1, false)[0][0].r.is_infinite());
        assert_eq!(Shape::ToolRow.rows(4, false)[0].len(), 3);
        assert_eq!(Shape::Form.rows(2, false).len(), 5);
        assert_eq!(Shape::from_id("diff").id(), "diff");
        assert_eq!(
            wave_delay(10, Duration::from_millis(120), Duration::from_millis(600)),
            Duration::from_millis(600)
        );
        assert!((sheen_phase(ClockMs(800), timing::SHEEN) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn gate_skips_fast_loads() {
        let mut g = LoadingGate::start(ClockMs(0));
        assert_eq!(g.show(ClockMs(100)), Gate::Blank);
        g.finish(ClockMs(120));
        assert_eq!(g.show(ClockMs(130)), Gate::Content);
        assert!(!g.flashed());
        g.restart(ClockMs(200));
        assert_eq!(g.show(ClockMs(400)), Gate::Skeleton);
        g.finish(ClockMs(900));
        assert!(g.flashed());
    }

    #[test]
    fn morph_resolves_in_order() {
        let mut m = Morph::default();
        assert_eq!(m.progress(0, 4, ClockMs(1000), false), 0.0);
        m.resolve(ClockMs(1000));
        let a = m.progress(0, 4, ClockMs(1150), false);
        let b = m.progress(3, 4, ClockMs(1150), false);
        assert!(a > b && b > 0.0);
        assert!(m.settled(4, ClockMs(2000)));
        assert_eq!(m.delay(2, 4), Duration::from_millis(80));
        assert_eq!(m.delay(20, 41), Duration::from_millis(240), "capped");
        assert_eq!(
            Morph::stretch((100.0, 12.0), (200.0, 20.0), 0.5),
            (150.0, 16.0)
        );
        m.reload();
        assert!(m.loading());
        assert!(critical(10.0, 20.0) <= 1.0 && critical(0.05, 20.0) < critical(0.1, 20.0));
    }
}
