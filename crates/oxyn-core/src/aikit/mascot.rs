//! The assistant characters (AI mascot, Ghost mascot, Cube mascot, Pixel mascot) and the AI
//! spark share one status model and one drawing format: every frame is a list of filled
//! [`Shape`]s in a 100 × 100 box, built from `now`, so the Dioxus SVG and the GPUI canvas draw
//! the same character from the same numbers. The characters are Oxyn's own drawings.
//!
//! A status change never cuts: the pose on screen blends into the new one over [`BLEND_MS`].
//! Left without a status, a character plays every status in turn ([`STEP_MS`] each).

use super::nth;
use std::f32::consts::{PI, TAU};

/// What the assistant is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    /// Resting.
    Idle,
    /// Hearing the person.
    Listening,
    /// Working something out.
    Thinking,
    /// Producing an answer.
    Writing,
    /// Finished.
    Done,
    /// Something went wrong.
    Error,
}

/// Every status, in loop order.
pub const MOODS: [Mood; 6] = [
    Mood::Idle,
    Mood::Listening,
    Mood::Thinking,
    Mood::Writing,
    Mood::Done,
    Mood::Error,
];

impl Mood {
    /// Lower-case name (`thinking`).
    pub fn name(self) -> &'static str {
        match self {
            Mood::Idle => "idle",
            Mood::Listening => "listening",
            Mood::Thinking => "thinking",
            Mood::Writing => "writing",
            Mood::Done => "done",
            Mood::Error => "error",
        }
    }
    /// The status from its name; unknown names are idle.
    pub fn parse(s: &str) -> Mood {
        MOODS
            .iter()
            .copied()
            .find(|m| m.name() == s)
            .unwrap_or(Mood::Idle)
    }
    /// Position in [`MOODS`].
    pub fn index(self) -> usize {
        MOODS.iter().position(|m| *m == self).unwrap_or(0)
    }
}

/// How long each status holds in the self-playing loop, ms.
pub const STEP_MS: u64 = 2_800;
/// How long a status change blends, ms.
pub const BLEND_MS: u64 = 420;

/// Ink of a shape; each renderer maps it to a colour of its theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    /// The character's body.
    Body,
    /// The lit top or the soft highlight of the body.
    Light,
    /// The form shade or the turned-away side.
    Shade,
    /// Eyes, brows and mouth.
    Face,
    /// The companion (antenna orb, swirl, pen).
    Mark,
    /// The contact shadow and written lines.
    Ground,
    /// Errors.
    Danger,
    /// Cheeks.
    Blush,
}

/// One filled polygon in the 100 × 100 box.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// Colour.
    pub ink: Ink,
    /// Opacity 0..1.
    pub alpha: f32,
    /// Corners, clockwise or not (non-zero fill).
    pub pts: Vec<(f32, f32)>,
}

impl Shape {
    /// SVG path data (`M x y L … Z`).
    pub fn d(&self) -> String {
        let mut s = String::with_capacity(self.pts.len() * 12);
        for (i, (x, y)) in self.pts.iter().enumerate() {
            s.push_str(if i == 0 { "M" } else { "L" });
            s.push_str(&format!("{x:.2} {y:.2}"));
        }
        s.push('Z');
        s
    }
}

/// An ellipse as a polygon (`n` corners), rotated by `rot` radians.
pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, rot: f32, n: usize) -> Vec<(f32, f32)> {
    let (s, c) = rot.sin_cos();
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * TAU;
            let (x, y) = (rx * a.cos(), ry * a.sin());
            (cx + x * c - y * s, cy + x * s + y * c)
        })
        .collect()
}

/// A polyline drawn `w` wide, as one polygon.
pub fn stroke(line: &[(f32, f32)], w: f32) -> Vec<(f32, f32)> {
    if line.len() < 2 {
        return Vec::new();
    }
    let h = w / 2.0;
    let mut left = Vec::with_capacity(line.len());
    let mut right = Vec::with_capacity(line.len());
    for i in 0..line.len() {
        let a = line[i.saturating_sub(1)];
        let b = line[(i + 1).min(line.len() - 1)];
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(0.0001);
        let (nx, ny) = (-dy / l * h, dx / l * h);
        left.push((line[i].0 + nx, line[i].1 + ny));
        right.push((line[i].0 - nx, line[i].1 - ny));
    }
    right.reverse();
    left.extend(right);
    left
}

/// Points along a circular arc from `a0` to `a1` radians.
pub fn arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// A four-point sparkle centred on `(cx, cy)`.
pub fn sparkle(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    let k = r * 0.28;
    vec![
        (cx, cy - r),
        (cx + k, cy - k),
        (cx + r, cy),
        (cx + k, cy + k),
        (cx, cy + r),
        (cx - k, cy + k),
        (cx - r, cy),
        (cx - k, cy - k),
    ]
}

/// Moves and turns every corner of `pts` about `(ox, oy)`.
pub fn place(pts: &mut [(f32, f32)], ox: f32, oy: f32, rot: f32, dx: f32, dy: f32) {
    let (s, c) = rot.sin_cos();
    for p in pts.iter_mut() {
        let (x, y) = (p.0 - ox, p.1 - oy);
        *p = (ox + x * c - y * s + dx, oy + x * s + y * c + dy);
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The face and body numbers every character is drawn from; two poses blend field by field.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    /// Vertical float, units (negative is up).
    pub bob: f32,
    /// Sideways shake, units.
    pub shake: f32,
    /// Head tilt, radians.
    pub tilt: f32,
    /// Body scale.
    pub grow: f32,
    /// Gaze, -1..1 each way.
    pub look_x: f32,
    /// Gaze, -1 (up) ..1 (down).
    pub look_y: f32,
    /// Eye opening, 0 shut ..1 open (above 1 is wide).
    pub open: f32,
    /// Happy squint, 0..1.
    pub happy: f32,
    /// Worried brows, 0..1.
    pub worry: f32,
    /// Companion visibility, 0..1.
    pub mark: f32,
    /// Sparkle size, 0..1.
    pub spark: f32,
    /// Written line length, 0..1.
    pub pen: f32,
    /// Body opacity.
    pub alpha: f32,
    /// Turn (the cube's yaw, the orbit angle), radians.
    pub turn: f32,
}

impl Pose {
    fn lerp(a: Pose, b: Pose, t: f32) -> Pose {
        Pose {
            bob: mix(a.bob, b.bob, t),
            shake: mix(a.shake, b.shake, t),
            tilt: mix(a.tilt, b.tilt, t),
            grow: mix(a.grow, b.grow, t),
            look_x: mix(a.look_x, b.look_x, t),
            look_y: mix(a.look_y, b.look_y, t),
            open: mix(a.open, b.open, t),
            happy: mix(a.happy, b.happy, t),
            worry: mix(a.worry, b.worry, t),
            mark: mix(a.mark, b.mark, t),
            spark: mix(a.spark, b.spark, t),
            pen: mix(a.pen, b.pen, t),
            alpha: mix(a.alpha, b.alpha, t),
            turn: mix(a.turn, b.turn, t),
        }
    }
}

/// A blink: eyes shut for 140 ms every 3.6 s (offset per character).
fn blink(ms: u64, off: u64) -> f32 {
    let p = (ms + off) % 3_600;
    if p < 140 {
        (p as f32 / 70.0 - 1.0).abs()
    } else {
        1.0
    }
}

/// The pose of a status `ms` after it began (shared by the drawn characters).
pub fn pose(m: Mood, ms: u64, off: u64) -> Pose {
    let s = ms as f32 / 1000.0;
    let base = Pose {
        grow: 1.0,
        open: blink(ms, off),
        alpha: 1.0,
        mark: 1.0,
        ..Pose::default()
    };
    match m {
        Mood::Idle => Pose {
            bob: (s * TAU / 3.2).sin() * 1.6,
            look_x: if (ms / 2_400) % 3 == 1 { 0.7 } else { 0.0 },
            grow: 1.0 + (s * TAU / 3.2).sin() * 0.012,
            ..base
        },
        Mood::Listening => Pose {
            bob: -1.0,
            grow: 1.05,
            open: 1.25 * blink(ms, off),
            tilt: 0.08,
            spark: 0.0,
            ..base
        },
        Mood::Thinking => Pose {
            bob: -2.0 + (s * TAU / 2.0).sin() * 0.8,
            tilt: -0.1,
            look_x: 0.4,
            look_y: -1.0,
            alpha: 0.86,
            turn: s * TAU / 1.6,
            ..base
        },
        Mood::Writing => Pose {
            look_x: -0.2 + ((s * 0.9).fract() * 0.9),
            look_y: 0.9,
            tilt: 0.05,
            pen: (s / 1.8).fract(),
            ..base
        },
        Mood::Done => Pose {
            bob: -1.5,
            happy: 1.0,
            spark: 1.0,
            mark: 0.0,
            ..base
        },
        Mood::Error => {
            let k = (1.0 - s / 0.7).max(0.0);
            Pose {
                shake: (s * TAU * 6.0).sin() * 2.4 * k,
                worry: 1.0,
                look_y: 0.4,
                open: 0.8,
                bob: 1.0,
                ..base
            }
        }
    }
}

/// The shared status state of a character or a spark.
#[derive(Clone, Debug, PartialEq)]
pub struct Mascot {
    /// A fixed status; `None` plays every status in turn.
    pub fixed: Option<Mood>,
    /// The status shown before the last change, and when the change happened.
    pub from: Mood,
    /// When the status last changed (or the loop started).
    pub since: u64,
    /// Frozen at this clock.
    pub paused: Option<u64>,
    /// Statuses the loop plays.
    pub cycle: Vec<Mood>,
    /// The loop's start.
    pub t0: u64,
}

impl Mascot {
    /// A character showing `fixed` (or the loop) from `now`.
    pub fn new(fixed: Option<Mood>, cycle: &[Mood], now: u64) -> Self {
        Self {
            fixed,
            from: fixed.unwrap_or(cycle[0]),
            since: now,
            paused: None,
            cycle: cycle.to_vec(),
            t0: now,
        }
    }

    fn clock(&self, now: u64) -> u64 {
        self.paused.unwrap_or(now)
    }

    /// The status on screen.
    pub fn mood(&self, now: u64) -> Mood {
        let now = self.clock(now);
        match self.fixed {
            Some(m) => m,
            None => {
                let i = (now.saturating_sub(self.t0) / STEP_MS) as usize % self.cycle.len();
                self.cycle[i]
            }
        }
    }

    /// The status before the current one and ms since the change.
    pub fn change(&self, now: u64) -> (Mood, u64) {
        let now = self.clock(now);
        match self.fixed {
            Some(_) => (self.from, now.saturating_sub(self.since)),
            None => {
                let el = now.saturating_sub(self.t0);
                let i = (el / STEP_MS) as usize;
                let prev = if i == 0 {
                    self.from
                } else {
                    self.cycle[(i - 1) % self.cycle.len()]
                };
                (prev, el % STEP_MS)
            }
        }
    }

    /// The blended pose (`off` staggers blinks between characters).
    pub fn pose(&self, now: u64, off: u64) -> Pose {
        let m = self.mood(now);
        let (prev, el) = self.change(now);
        let to = pose(m, el, off);
        if prev == m || el >= BLEND_MS {
            return to;
        }
        let from = pose(prev, el + STEP_MS, off);
        Pose::lerp(from, to, ease(el as f32 / BLEND_MS as f32))
    }

    /// Ms into the current status.
    pub fn elapsed(&self, now: u64) -> u64 {
        self.change(now).1
    }

    /// Sets a status (`None` returns to the loop), blending from the one on screen.
    pub fn set(&mut self, m: Option<Mood>, now: u64) -> Option<String> {
        let cur = self.mood(now);
        let now = self.clock(now);
        self.from = cur;
        self.since = now;
        self.t0 = now;
        self.fixed = m;
        Some(match m {
            Some(m) => format!("Now {}", m.name()),
            None => "Playing every status".into(),
        })
    }

    /// Presses: `ms-mood-<i>` (index in [`MOODS`]), `ms-auto`, `ms-pause`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ms-mood-") {
            return self.set(Some(MOODS[i.min(5)]), now);
        }
        match id {
            "ms-auto" => self.set(None, now),
            "ms-pause" => {
                if let Some(p) = self.paused.take() {
                    let gap = now.saturating_sub(p);
                    self.t0 += gap;
                    self.since += gap;
                    Some("Resumed".into())
                } else {
                    self.paused = Some(now);
                    Some("Paused".into())
                }
            }
            _ => None,
        }
    }

    /// The accessible label (`Atlas, thinking`).
    pub fn label(&self, name: &str, now: u64) -> String {
        format!("{name}, {}", self.mood(now).name())
    }
}

/// Which character to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The Oxyn assistant: a rounded head with an antenna orb.
    Orb,
    /// A floating ghost with a scalloped hem.
    Ghost,
    /// An isometric cube with a painted face.
    Cube,
}

/// Draws `kind` in pose `p` at clock `ms`; `compact` frames it tighter for sizes under 56 px
/// and drops the ground.
pub fn draw(kind: Kind, p: &Pose, ms: u64, compact: bool, ground: bool) -> Vec<Shape> {
    match kind {
        Kind::Orb => orb(p, ms, compact, ground),
        Kind::Ghost => ghost(p, ms, compact, ground),
        Kind::Cube => cube(p, ms, compact, ground),
    }
}

fn sh(ink: Ink, alpha: f32, pts: Vec<(f32, f32)>) -> Shape {
    Shape { ink, alpha, pts }
}

/// Two eyes (or happy arcs) and optional worried brows, centred on `(cx, cy)`.
fn face(out: &mut Vec<Shape>, p: &Pose, cx: f32, cy: f32, gap: f32, r: f32, ink: Ink) {
    let ex = p.look_x * r * 0.7;
    let ey = p.look_y * r * 0.6;
    for side in [-1.0f32, 1.0] {
        let x = cx + side * gap + ex;
        let y = cy + ey;
        if p.happy > 0.5 {
            let a = arc(x, y + r * 0.5, r, PI * 1.15, PI * 1.85, 10);
            out.push(sh(ink, 1.0, stroke(&a, r * 0.55)));
        } else {
            let ry = (r * 1.25 * p.open).max(0.35);
            out.push(sh(ink, 1.0, ellipse(x, y, r * 0.85, ry, 0.0, 18)));
        }
        if p.worry > 0.05 {
            let lift = r * 1.9;
            let a = (x - side * r * 1.1, y - lift - r * 0.7 * p.worry);
            let b = (x + side * r * 0.9, y - lift);
            out.push(sh(ink, p.worry, stroke(&[a, b], r * 0.5)));
        }
    }
}

/// The written line under a character, `pen` of the way along.
fn pen_line(out: &mut Vec<Shape>, pen: f32, y: f32, x0: f32, len: f32) {
    if pen <= 0.01 {
        return;
    }
    let mut pts = Vec::new();
    let n = 24;
    for i in 0..=n {
        let t = i as f32 / n as f32 * pen;
        pts.push((x0 + t * len, y + (t * 40.0).sin() * 0.9));
    }
    out.push(sh(Ink::Ground, 1.0, stroke(&pts, 1.6)));
}

fn orb(p: &Pose, ms: u64, compact: bool, ground: bool) -> Vec<Shape> {
    let mut out = Vec::new();
    let (k, cy0) = if compact { (1.25, 56.0) } else { (1.0, 54.0) };
    let cx = 50.0 + p.shake;
    let cy = cy0 + p.bob;
    let w = 30.0 * k * p.grow;
    let h = 26.0 * k * p.grow;
    if ground && !compact {
        let s = 1.0 - (-p.bob).max(0.0) * 0.06;
        out.push(sh(
            Ink::Ground,
            0.5,
            ellipse(50.0, 86.0, 20.0 * s, 3.2, 0.0, 24),
        ));
        pen_line(&mut out, p.pen, 90.0, 28.0, 44.0);
    }
    // Antenna stalk.
    let top = (cx + (p.tilt * 20.0), cy - h - 1.0);
    let tip = (cx + p.tilt * 34.0, cy - h - 11.0 * k);
    // Orb: rests on the stalk, orbits while thinking, pulses while listening.
    let ms_f = ms as f32 / 1000.0;
    let (ox, oy, front) = if p.turn.abs() > 0.01 {
        let a = p.turn;
        (
            cx + a.cos() * (w + 8.0),
            cy - h * 0.6 + a.sin() * 7.0,
            a.sin() > 0.0,
        )
    } else {
        (tip.0, tip.1, true)
    };
    let pulse = 1.0 + (ms_f * TAU * 2.2).sin().abs() * 0.35 * (p.grow - 1.0) * 20.0;
    let orb_r = 4.2 * k * pulse.clamp(1.0, 1.6) * p.mark;
    let orb_ink = if p.worry > 0.5 {
        Ink::Danger
    } else {
        Ink::Mark
    };
    if !front && p.mark > 0.05 {
        out.push(sh(orb_ink, 0.9, ellipse(ox, oy, orb_r, orb_r, 0.0, 20)));
    }
    out.push(sh(Ink::Shade, 1.0, stroke(&[top, tip], 1.8)));
    // Head: a rounded square (superellipse).
    let n = 40;
    let mut head: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * TAU;
            let (c, s) = (a.cos(), a.sin());
            let ex = 2.0 / 3.6;
            (
                cx + w * c.signum() * c.abs().powf(ex),
                cy + h * s.signum() * s.abs().powf(ex),
            )
        })
        .collect();
    place(&mut head, cx, cy, p.tilt, 0.0, 0.0);
    out.push(sh(Ink::Body, p.alpha, head));
    let mut shade = ellipse(cx + w * 0.25, cy + h * 0.45, w * 0.7, h * 0.4, 0.0, 24);
    place(&mut shade, cx, cy, p.tilt, 0.0, 0.0);
    out.push(sh(Ink::Shade, 0.35 * p.alpha, shade));
    let mut hi = ellipse(cx - w * 0.45, cy - h * 0.55, w * 0.22, h * 0.12, -0.5, 16);
    place(&mut hi, cx, cy, p.tilt, 0.0, 0.0);
    out.push(sh(Ink::Light, 0.7 * p.alpha, hi));
    let mut f = Vec::new();
    face(&mut f, p, cx, cy - 1.0, 9.0 * k, 3.4 * k, Ink::Face);
    if p.happy > 0.5 {
        f.push(sh(
            Ink::Blush,
            0.6,
            ellipse(cx - 15.0 * k, cy + 6.0, 3.6, 2.0, 0.0, 14),
        ));
        f.push(sh(
            Ink::Blush,
            0.6,
            ellipse(cx + 15.0 * k, cy + 6.0, 3.6, 2.0, 0.0, 14),
        ));
    }
    for s in f.iter_mut() {
        place(&mut s.pts, cx, cy, p.tilt, 0.0, 0.0);
    }
    out.extend(f);
    if front && p.mark > 0.05 {
        out.push(sh(orb_ink, 1.0, ellipse(ox, oy, orb_r, orb_r, 0.0, 20)));
        out.push(sh(
            Ink::Light,
            0.6,
            ellipse(
                ox - orb_r * 0.3,
                oy - orb_r * 0.3,
                orb_r * 0.35,
                orb_r * 0.35,
                0.0,
                10,
            ),
        ));
    }
    if p.spark > 0.05 {
        let r = 7.0 * p.spark * k;
        out.push(sh(Ink::Mark, p.spark, sparkle(tip.0 + 4.0, tip.1 - 2.0, r)));
        out.push(sh(
            Ink::Mark,
            p.spark * 0.7,
            sparkle(cx + w + 4.0, cy - h * 0.4, r * 0.5),
        ));
    }
    out
}

fn ghost(p: &Pose, ms: u64, compact: bool, ground: bool) -> Vec<Shape> {
    let mut out = Vec::new();
    let k = if compact { 1.22 } else { 1.0 };
    let cx = 50.0 + p.shake;
    let cy = 46.0 + p.bob * 1.6;
    let w = 25.0 * k * p.grow;
    let s = ms as f32 / 1000.0;
    if ground && !compact {
        let lift = (-p.bob).max(0.0);
        out.push(sh(
            Ink::Ground,
            0.45 - lift * 0.05,
            ellipse(50.0, 88.0, 17.0 - lift * 1.5, 2.8, 0.0, 24),
        ));
        pen_line(&mut out, p.pen, 91.0, 30.0, 40.0);
    }
    // Sheet: a dome, straight sides and a hem of three scallops that ripple.
    let mut pts = arc(cx, cy, w, PI, TAU, 24);
    let hem_y = cy + 30.0 * k;
    pts.push((cx + w, hem_y));
    let n = 36;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let x = cx + w - t * 2.0 * w;
        let wave = (t * 3.0 * TAU + s * 4.0).sin() * 3.2 + (t * 2.0 * TAU - s * 2.6).sin() * 1.2;
        let dip = (t * 3.0 * PI).sin().abs() * 4.5;
        pts.push((x, hem_y + wave * 0.6 + dip));
    }
    pts.push((cx - w, cy));
    place(&mut pts, cx, cy + 10.0, p.tilt, 0.0, 0.0);
    out.push(sh(Ink::Body, p.alpha, pts));
    let mut shade = ellipse(cx + w * 0.45, cy + 14.0, w * 0.4, 18.0 * k, 0.0, 20);
    place(&mut shade, cx, cy + 10.0, p.tilt, 0.0, 0.0);
    out.push(sh(Ink::Shade, 0.3 * p.alpha, shade));
    let mut f = Vec::new();
    face(&mut f, p, cx, cy + 2.0, 8.0 * k, 3.2 * k, Ink::Face);
    if p.grow > 1.02 {
        f.push(sh(
            Ink::Face,
            1.0,
            ellipse(cx, cy + 11.0, 2.0, 2.6, 0.0, 14),
        ));
    }
    if p.happy > 0.5 {
        let a = arc(cx, cy + 8.0, 4.0, 0.2 * PI, 0.8 * PI, 10);
        f.push(sh(Ink::Face, 1.0, stroke(&a, 1.6)));
    }
    for s in f.iter_mut() {
        place(&mut s.pts, cx, cy + 10.0, p.tilt, 0.0, 0.0);
    }
    out.extend(f);
    // Thinking: a spiral over the head that winds up and unwinds.
    if p.turn.abs() > 0.01 {
        let grow = ((s * 0.8).fract() * 2.0 - 1.0).abs();
        let mut sw = Vec::new();
        for i in 0..=30 {
            let t = i as f32 / 30.0 * (0.3 + grow * 0.7);
            let a = t * TAU * 1.6 + p.turn * 0.3;
            let r = 1.0 + t * 7.0;
            sw.push((cx + 16.0 + a.cos() * r, cy - w - 4.0 + a.sin() * r * 0.8));
        }
        out.push(sh(Ink::Mark, 0.9, stroke(&sw, 1.6)));
    }
    // Writing: a tiny pen at the end of the line.
    if p.pen > 0.01 && !compact {
        let tx = 30.0 + p.pen * 40.0;
        let mut pen = vec![
            (tx, 90.0),
            (tx + 2.0, 82.0),
            (tx + 4.0, 83.0),
            (tx + 1.0, 90.5),
        ];
        place(&mut pen, tx, 90.0, 0.25, 0.0, 0.0);
        out.push(sh(Ink::Mark, 1.0, pen));
    }
    if p.spark > 0.05 {
        out.push(sh(
            Ink::Mark,
            p.spark,
            sparkle(cx + w + 2.0, cy - w + 4.0, 7.0 * p.spark * k),
        ));
    }
    out
}

/// A unit cube's corners turned by `yaw` and seen from a fixed camera, in the 100 box.
fn cube_corners(yaw: f32, roll: f32, cx: f32, cy: f32, size: f32) -> [(f32, f32, f32); 8] {
    let mut out = [(0.0, 0.0, 0.0); 8];
    let (sy, cyw) = yaw.sin_cos();
    let (sr, cr) = roll.sin_cos();
    let pitch: f32 = 0.56; // camera above the cube (about 32°)
    let (sp, cp) = pitch.sin_cos();
    for (i, o) in out.iter_mut().enumerate() {
        let x = if i & 1 == 0 { -1.0 } else { 1.0 };
        let y = if i & 2 == 0 { -1.0 } else { 1.0 };
        let z = if i & 4 == 0 { -1.0 } else { 1.0 };
        // Yaw about the vertical axis, then roll, then the camera pitch.
        let (x1, z1) = (x * cyw - z * sy, x * sy + z * cyw);
        let (x2, y2) = (x1 * cr - y * sr, x1 * sr + y * cr);
        let (y3, z3) = (y2 * cp - z1 * sp, y2 * sp + z1 * cp);
        *o = (cx + x2 * size, cy + y3 * size, z3);
    }
    out
}

fn cube(p: &Pose, ms: u64, compact: bool, ground: bool) -> Vec<Shape> {
    let mut out = Vec::new();
    let k = if compact { 1.2 } else { 1.0 };
    let s = ms as f32 / 1000.0;
    // Thinking turns a quarter at a time: hold, then ease through 90°.
    let yaw = if p.turn.abs() > 0.01 {
        let t = s / 1.3;
        let step = t.floor();
        let f = ((t - step) - 0.38).max(0.0) / 0.62;
        (step + ease(f)) * PI / 2.0
    } else {
        0.0
    } + 0.62
        + p.look_x * 0.05;
    let cx = 50.0 + p.shake;
    let cy = 52.0 + p.bob;
    let size = 17.0 * k * p.grow;
    if ground && !compact {
        out.push(sh(
            Ink::Ground,
            0.45,
            ellipse(50.0, 84.0, 26.0, 4.0, 0.0, 24),
        ));
        if p.pen > 0.01 {
            for (i, w) in [36.0f32, 28.0, 32.0].iter().enumerate() {
                let t = (p.pen * 3.0 - i as f32).clamp(0.0, 1.0);
                if t > 0.0 {
                    let y = 88.0 + i as f32 * 3.6;
                    out.push(sh(
                        Ink::Ground,
                        1.0,
                        stroke(&[(32.0, y), (32.0 + w * t, y)], 1.4),
                    ));
                }
            }
        }
    }
    let c = cube_corners(yaw, p.tilt * 0.6, cx, cy, size);
    // Faces as corner indices; draw the three that face the camera.
    let faces: [([usize; 4], (f32, f32, f32)); 6] = [
        ([0, 1, 3, 2], (0.0, 0.0, -1.0)),
        ([4, 5, 7, 6], (0.0, 0.0, 1.0)),
        ([0, 2, 6, 4], (-1.0, 0.0, 0.0)),
        ([1, 3, 7, 5], (1.0, 0.0, 0.0)),
        ([0, 1, 5, 4], (0.0, -1.0, 0.0)),
        ([2, 3, 7, 6], (0.0, 1.0, 0.0)),
    ];
    let depth = |f: &[usize; 4]| f.iter().map(|&i| c[i].2).sum::<f32>() / 4.0;
    let mut vis: Vec<(usize, f32)> = faces
        .iter()
        .enumerate()
        .map(|(i, (f, _))| (i, depth(f)))
        .collect();
    vis.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    // The front-most side face carries the face.
    let face_i = vis
        .iter()
        .rev()
        .find(|(i, _)| *i < 4)
        .map(|v| v.0)
        .unwrap_or(0);
    for (i, d) in vis.iter().rev().take(3).rev() {
        let f = faces[*i].0;
        let pts: Vec<(f32, f32)> = f.iter().map(|&j| (c[j].0, c[j].1)).collect();
        let ink = if *i == 4 {
            Ink::Light
        } else if *i == face_i {
            Ink::Body
        } else {
            Ink::Shade
        };
        let _ = d;
        out.push(sh(ink, p.alpha, pts));
    }
    // Paint the face onto that side through its corners (bilinear).
    let f = faces[face_i].0;
    let q = [c[f[0]], c[f[1]], c[f[3]], c[f[2]]];
    let at = |u: f32, v: f32| -> (f32, f32) {
        let top = (mix(q[0].0, q[1].0, u), mix(q[0].1, q[1].1, u));
        let bot = (mix(q[2].0, q[3].0, u), mix(q[2].1, q[3].1, u));
        (mix(top.0, bot.0, v), mix(top.1, bot.1, v))
    };
    let mut fl = Vec::new();
    face(&mut fl, p, 0.5, 0.48, 0.2, 0.075, Ink::Face);
    for s in fl {
        let pts = s.pts.iter().map(|&(u, v)| at(u, v)).collect();
        out.push(sh(s.ink, s.alpha, pts));
    }
    if p.spark > 0.05 {
        out.push(sh(
            Ink::Mark,
            p.spark,
            sparkle(cx + size * 1.4, cy - size * 1.5, 7.0 * p.spark * k),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loops_blends_and_draws() {
        let mut m = Mascot::new(None, &MOODS, 0);
        assert_eq!(m.mood(0), Mood::Idle);
        assert_eq!(m.mood(STEP_MS + 10), Mood::Listening);
        assert_eq!(m.mood(STEP_MS * 6 + 10), Mood::Idle);
        m.press("ms-mood-2", 1_000);
        assert_eq!(m.mood(5_000), Mood::Thinking);
        let mid = m.pose(1_000 + BLEND_MS / 2, 0);
        let end = m.pose(1_000 + BLEND_MS * 3, 0);
        assert!(mid.look_y > end.look_y);
        m.press("ms-pause", 9_000);
        let a = m.pose(9_000, 0);
        assert_eq!(a, m.pose(20_000, 0));
        m.press("ms-pause", 20_000);
        assert_eq!(m.label("Atlas", 21_000), "Atlas, thinking");
        for k in [Kind::Orb, Kind::Ghost, Kind::Cube] {
            for mood in MOODS {
                let p = pose(mood, 700, 0);
                let s = draw(k, &p, 700, false, true);
                assert!(s.len() > 3);
                assert!(s.iter().all(|x| x.pts.iter().all(|p| p.0.is_finite())));
                assert!(!draw(k, &p, 700, true, false).is_empty());
            }
        }
        assert!(
            Shape {
                ink: Ink::Body,
                alpha: 1.0,
                pts: ellipse(1.0, 1.0, 1.0, 1.0, 0.0, 4)
            }
            .d()
            .starts_with('M')
        );
        assert_eq!(Mood::parse("done"), Mood::Done);
    }
}
