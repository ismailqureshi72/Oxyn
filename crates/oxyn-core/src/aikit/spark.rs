//! AI spark: a small star glyph that signals machine work, with four long points, four short paler
//! ones and a round centre. Every state moves differently (a slow swell when resting, a wave around
//! the points while thinking, a pulse while writing, a pull-in when finished, stubby faded points
//! on failure), and a change eases from the pose on screen. Drawn as [`Shape`]s in the 100 × 100
//! box.

use super::mascot::{BLEND_MS, Ink, Mascot, Mood, STEP_MS, Shape};
use std::f32::consts::{PI, TAU};

/// The statuses the spark loops through when left alone.
pub const CYCLE: [Mood; 4] = [Mood::Idle, Mood::Thinking, Mood::Writing, Mood::Done];

/// The pill label for a status.
pub fn label(m: Mood) -> &'static str {
    match m {
        Mood::Idle | Mood::Listening => "Ready",
        Mood::Thinking => "Thinking",
        Mood::Writing => "Writing",
        Mood::Done => "Done",
        Mood::Error => "Couldn't finish",
    }
}

/// The spoken name of a status (`AI is thinking`).
pub fn spoken(m: Mood) -> &'static str {
    match m {
        Mood::Idle | Mood::Listening => "AI is ready",
        Mood::Thinking => "AI is thinking",
        Mood::Writing => "AI is writing",
        Mood::Done => "AI is done",
        Mood::Error => "AI couldn't finish",
    }
}

/// Ray lengths (four primaries then four diagonals), whole-mark scale and turn, diagonal opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparkPose {
    /// Lengths in units.
    pub len: [f32; 8],
    /// Scale of the whole mark.
    pub scale: f32,
    /// Turn, radians.
    pub rot: f32,
    /// Diagonal opacity.
    pub dim: f32,
    /// Core radius.
    pub core: f32,
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The pose of a status `ms` after it began, at `speed`.
pub fn pose(m: Mood, ms: u64, speed: f32) -> SparkPose {
    let s = ms as f32 / 1000.0 * speed;
    let mut len = [44.0, 44.0, 44.0, 44.0, 24.0, 24.0, 24.0, 24.0];
    let mut p = SparkPose {
        len,
        scale: 1.0,
        rot: 0.0,
        dim: 0.55,
        core: 5.5,
    };
    match m {
        Mood::Idle | Mood::Listening => {
            p.scale = 1.0 + (s * TAU / 6.4).sin() * 0.035;
            let d = 4 + ((s / 1.6) as usize % 4);
            len[d] += ((s / 1.6).fract() * PI).sin() * 6.0;
        }
        Mood::Thinking => {
            for (i, l) in len.iter_mut().enumerate() {
                let a = i as f32 * PI / 2.0 + if i >= 4 { PI / 4.0 } else { 0.0 };
                *l *= 1.0 + 0.22 * (s * TAU / 1.1 - a).sin();
            }
            p.rot = s * 0.5;
        }
        Mood::Writing => {
            let ph = (s / 0.9).fract();
            let beat =
                (-((ph - 0.1) / 0.05).powi(2)).exp() + 0.6 * (-((ph - 0.3) / 0.05).powi(2)).exp();
            p.scale = 1.0 + beat * 0.12;
            p.rot = s * 1.2;
            p.dim = 0.7;
        }
        Mood::Done => {
            let t = (s / 0.6).min(1.0);
            let g = (t * PI).sin();
            for l in len.iter_mut() {
                *l *= 1.0 - g * 0.35;
            }
            p.rot = t * PI / 4.0;
            p.core = 5.5 + g * 3.0;
        }
        Mood::Error => {
            for (i, l) in len.iter_mut().enumerate() {
                *l *= if i < 4 { 0.62 } else { 0.5 };
            }
            p.dim = 0.25;
        }
    }
    p.len = len;
    p
}

fn lerp(a: SparkPose, b: SparkPose, t: f32) -> SparkPose {
    let mut len = [0.0; 8];
    for (i, l) in len.iter_mut().enumerate() {
        *l = mix(a.len[i], b.len[i], t);
    }
    SparkPose {
        len,
        scale: mix(a.scale, b.scale, t),
        rot: mix(a.rot, b.rot, t),
        dim: mix(a.dim, b.dim, t),
        core: mix(a.core, b.core, t),
    }
}

/// The blended pose on screen.
pub fn pose_at(m: &Mascot, now: u64, speed: f32) -> SparkPose {
    let cur = m.mood(now);
    let (prev, el) = m.change(now);
    let to = pose(cur, el, speed);
    if prev == cur || el >= BLEND_MS {
        return to;
    }
    let t = el as f32 / BLEND_MS as f32;
    let t = t * t * (3.0 - 2.0 * t);
    lerp(pose(prev, el + STEP_MS, speed), to, t)
}

/// The mark as shapes; error rays use [`Ink::Danger`].
pub fn draw(p: &SparkPose, error: bool) -> Vec<Shape> {
    let ink = if error { Ink::Danger } else { Ink::Mark };
    let mut out = Vec::new();
    let (cx, cy) = (50.0, 50.0);
    for pass in [1usize, 0] {
        for i in 0..4 {
            let j = pass * 4 + i;
            let a = p.rot + i as f32 * PI / 2.0 - PI / 2.0 + if pass == 1 { PI / 4.0 } else { 0.0 };
            let l = p.len[j] * p.scale;
            let w = if pass == 0 { 6.5 } else { 3.6 } * p.scale.sqrt();
            let (c, s) = (a.cos(), a.sin());
            let (nx, ny) = (-s, c);
            let m = l * 0.24;
            let pts = vec![
                (cx + c * 2.0, cy + s * 2.0),
                (cx + c * m + nx * w, cy + s * m + ny * w),
                (cx + c * l, cy + s * l),
                (cx + c * m - nx * w, cy + s * m - ny * w),
            ];
            out.push(Shape {
                ink: if pass == 1 { Ink::Light } else { ink },
                alpha: if pass == 1 { p.dim } else { 1.0 },
                pts,
            });
        }
    }
    out.push(Shape {
        ink,
        alpha: 1.0,
        pts: super::mascot::ellipse(cx, cy, p.core * p.scale, p.core * p.scale, 0.0, 20),
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spark_poses_blend_and_draw() {
        let mut m = Mascot::new(None, &CYCLE, 0);
        assert_eq!(m.mood(STEP_MS + 1), Mood::Thinking);
        m.set(Some(Mood::Error), 500);
        let p = pose_at(&m, 500 + BLEND_MS + 10, 1.0);
        assert!(p.len[0] < 30.0);
        assert_eq!(draw(&p, true).len(), 9);
        assert_eq!(label(Mood::Writing), "Writing");
        let g = pose(Mood::Writing, 100, 1.0);
        assert!(g.scale > 1.0);
    }
}
