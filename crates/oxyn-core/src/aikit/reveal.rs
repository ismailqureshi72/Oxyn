//! Image reveal: an image forming the way a diffusion model samples. A coarse latent mosaic of
//! the image's own colours with noise falls away one denoising step at a time while a blurred
//! prediction sharpens; a decode band then lands the crisp image top to bottom, and a progress
//! pill shows the step and, once done, how long it took. Queued and error states stay calm.
//!
//! The sample images are Oxyn's own vector scenes ([`scene`]), so nothing is fetched.

use super::{hash, mascot::ellipse};

/// Queue time, ms.
pub const QUEUE_MS: u64 = 900;
/// Decode time, ms.
pub const DECODE_MS: u64 = 560;
/// A finished image holds this long before a loop restarts, ms.
pub const HOLD_MS: u64 = 2_600;
/// Latent grid cells per side.
pub const GRID: usize = 16;

/// Scene names (used as alt text).
pub const SCENES: [&str; 3] = [
    "A lake at dusk below layered mountains",
    "Desert dunes under a low sun",
    "A lighthouse on a green cliff above the sea",
];

/// A coloured polygon of a scene: `0xRRGGBB` and its corners in the 100 × 100 box.
pub type Patch = (u32, Vec<(f32, f32)>);

fn rect(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}

/// The polygons of scene `i`, back to front.
pub fn scene(i: usize) -> Vec<Patch> {
    match i % 3 {
        0 => vec![
            (0x2b2f5c, rect(0.0, 0.0, 100.0, 20.0)),
            (0x5b4a7a, rect(0.0, 20.0, 100.0, 14.0)),
            (0xc06a6f, rect(0.0, 34.0, 100.0, 12.0)),
            (0xf2a65e, rect(0.0, 46.0, 100.0, 10.0)),
            (0xffd38a, ellipse(64.0, 52.0, 9.0, 9.0, 0.0, 28)),
            (
                0x55456f,
                vec![
                    (0.0, 60.0),
                    (18.0, 38.0),
                    (30.0, 50.0),
                    (46.0, 32.0),
                    (62.0, 52.0),
                    (78.0, 40.0),
                    (100.0, 56.0),
                    (100.0, 64.0),
                    (0.0, 64.0),
                ],
            ),
            (
                0x2f2a48,
                vec![
                    (0.0, 64.0),
                    (14.0, 52.0),
                    (28.0, 62.0),
                    (40.0, 54.0),
                    (56.0, 64.0),
                    (100.0, 64.0),
                ],
            ),
            (0x3b4f7a, rect(0.0, 64.0, 100.0, 36.0)),
            (0xf2a65e, rect(56.0, 66.0, 16.0, 2.0)),
            (0xe8955c, rect(58.0, 71.0, 12.0, 1.6)),
            (0xd98a62, rect(60.0, 76.0, 8.0, 1.4)),
            (
                0x22304f,
                vec![
                    (0.0, 86.0),
                    (30.0, 84.0),
                    (52.0, 90.0),
                    (100.0, 86.0),
                    (100.0, 100.0),
                    (0.0, 100.0),
                ],
            ),
        ],
        1 => vec![
            (0x8fc3e0, rect(0.0, 0.0, 100.0, 30.0)),
            (0xf3d9b1, rect(0.0, 30.0, 100.0, 26.0)),
            (0xfff1c9, ellipse(26.0, 44.0, 8.0, 8.0, 0.0, 28)),
            (
                0xe0a96d,
                vec![
                    (0.0, 62.0),
                    (24.0, 50.0),
                    (48.0, 58.0),
                    (72.0, 46.0),
                    (100.0, 54.0),
                    (100.0, 100.0),
                    (0.0, 100.0),
                ],
            ),
            (
                0xc9844b,
                vec![
                    (0.0, 76.0),
                    (30.0, 62.0),
                    (58.0, 74.0),
                    (84.0, 64.0),
                    (100.0, 70.0),
                    (100.0, 100.0),
                    (0.0, 100.0),
                ],
            ),
            (
                0xa9663a,
                vec![
                    (0.0, 90.0),
                    (22.0, 80.0),
                    (50.0, 88.0),
                    (78.0, 78.0),
                    (100.0, 86.0),
                    (100.0, 100.0),
                    (0.0, 100.0),
                ],
            ),
            (
                0x7d4a2b,
                vec![(60.0, 100.0), (78.0, 92.0), (100.0, 96.0), (100.0, 100.0)],
            ),
        ],
        _ => vec![
            (0xa7d8e8, rect(0.0, 0.0, 100.0, 46.0)),
            (0xffffff, ellipse(30.0, 18.0, 12.0, 4.0, 0.0, 24)),
            (0xffffff, ellipse(70.0, 12.0, 9.0, 3.0, 0.0, 24)),
            (0x2f7fa6, rect(0.0, 46.0, 100.0, 54.0)),
            (0x5aa6c8, rect(0.0, 50.0, 100.0, 3.0)),
            (
                0x4f8f4a,
                vec![
                    (36.0, 100.0),
                    (44.0, 54.0),
                    (62.0, 48.0),
                    (100.0, 46.0),
                    (100.0, 100.0),
                ],
            ),
            (
                0x8a7a62,
                vec![(36.0, 100.0), (44.0, 54.0), (50.0, 60.0), (46.0, 100.0)],
            ),
            (0xf4f1ea, rect(76.0, 26.0, 6.0, 22.0)),
            (0xd2453c, rect(76.0, 32.0, 6.0, 4.0)),
            (0xd2453c, rect(76.0, 40.0, 6.0, 4.0)),
            (0x2f2f36, vec![(75.0, 26.0), (79.0, 21.0), (83.0, 26.0)]),
            (0xffe28a, rect(77.5, 23.0, 3.0, 2.6)),
        ],
    }
}

fn inside(p: (f32, f32), poly: &[(f32, f32)]) -> bool {
    let mut c = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
        j = i;
    }
    c
}

/// The scene's colour at a point.
pub fn color_at(s: &[Patch], p: (f32, f32)) -> u32 {
    s.iter()
        .rev()
        .find(|(_, poly)| inside(p, poly))
        .map(|x| x.0)
        .unwrap_or(0x808080)
}

/// The latent mosaic at a denoising step: `GRID × GRID` colours, row by row. `noise` 0..1 mixes
/// gaussian-ish colour noise over the image's own colours.
pub fn latent(scene_i: usize, seed: u64, step: u64, noise: f32) -> Vec<u32> {
    let s = scene(scene_i);
    let c = 100.0 / GRID as f32;
    (0..GRID * GRID)
        .map(|k| {
            let (x, y) = (
                (k % GRID) as f32 * c + c / 2.0,
                (k / GRID) as f32 * c + c / 2.0,
            );
            let base = color_at(&s, (x, y));
            let h = hash(&format!("{seed}:{step}:{k}"));
            let ch = |shift: u32, i: u32| -> u32 {
                let v = ((base >> shift) & 0xff) as f32;
                let r = (((h >> (i * 10)) & 0x3ff) as f32 / 1023.0
                    + ((h >> (i * 10 + 30)) & 0x3ff) as f32 / 1023.0)
                    - 1.0;
                (v + r * 170.0 * noise).clamp(0.0, 255.0) as u32
            };
            (ch(16, 0) << 16) | (ch(8, 1) << 8) | ch(0, 2)
        })
        .collect()
}

/// Where the image is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting.
    Queued,
    /// Denoising.
    Generating,
    /// Tiled decode.
    Decoding,
    /// Crisp.
    Done,
    /// Failed.
    Error,
}

impl Phase {
    /// The word.
    pub fn word(self) -> &'static str {
        match self {
            Phase::Queued => "Queued",
            Phase::Generating => "Generating image",
            Phase::Decoding => "Decoding",
            Phase::Done => "Image ready",
            Phase::Error => "Generation failed",
        }
    }
}

/// The live state of one canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct Reveal {
    /// Run start.
    pub t0: u64,
    /// Sampling time, ms.
    pub duration: u64,
    /// Steps.
    pub steps: u64,
    /// Extra queue time, ms.
    pub delay: u64,
    /// Loop after the hold.
    pub looping: bool,
    /// Fails at 60 % (the error scenario).
    pub fail: bool,
    /// Scene and noise seed.
    pub seed: u64,
}

impl Reveal {
    /// A canvas that starts at `now`.
    pub fn new(
        duration: u64,
        steps: u64,
        delay: u64,
        looping: bool,
        fail: bool,
        seed: u64,
        now: u64,
    ) -> Self {
        Self {
            t0: now,
            duration: duration.max(500),
            steps: steps.clamp(4, 80),
            delay,
            looping,
            fail,
            seed,
        }
    }

    fn cycle(&self) -> u64 {
        QUEUE_MS + self.delay + self.duration + DECODE_MS + HOLD_MS
    }

    /// Ms into the current run and the run number.
    fn el(&self, now: u64) -> (u64, u64) {
        let e = now.saturating_sub(self.t0);
        if self.looping && !self.fail {
            (e % self.cycle(), e / self.cycle())
        } else {
            (e, 0)
        }
    }

    /// The scene of the current run.
    pub fn scene(&self, now: u64) -> usize {
        ((self.seed + self.el(now).1) % 3) as usize
    }

    /// Phase and its progress 0..1.
    pub fn phase(&self, now: u64) -> (Phase, f32) {
        let (e, _) = self.el(now);
        let q = QUEUE_MS + self.delay;
        if e < q {
            return (Phase::Queued, e as f32 / q as f32);
        }
        let g = e - q;
        if g < self.duration {
            let p = g as f32 / self.duration as f32;
            if self.fail && p >= 0.6 {
                return (Phase::Error, 0.6);
            }
            return (Phase::Generating, p);
        }
        if self.fail {
            return (Phase::Error, 0.6);
        }
        let d = g - self.duration;
        if d < DECODE_MS {
            return (Phase::Decoding, d as f32 / DECODE_MS as f32);
        }
        (Phase::Done, ((d - DECODE_MS) as f32 / 600.0).min(1.0))
    }

    /// The step shown (1-based) while generating.
    pub fn step(&self, now: u64) -> u64 {
        match self.phase(now) {
            (Phase::Generating, p) | (Phase::Error, p) => {
                ((p * self.steps as f32) as u64 + 1).min(self.steps)
            }
            (Phase::Queued, _) => 0,
            _ => self.steps,
        }
    }

    /// Denoising progress quantised to whole steps, 0..1.
    pub fn denoised(&self, now: u64) -> f32 {
        match self.phase(now) {
            (Phase::Queued, _) => 0.0,
            (Phase::Generating, _) | (Phase::Error, _) => self.step(now) as f32 / self.steps as f32,
            _ => 1.0,
        }
    }

    /// Noise strength of the mosaic, 1..0.
    pub fn noise(&self, now: u64) -> f32 {
        (1.0 - self.denoised(now)).powf(1.3)
    }

    /// Blur of the prediction in px at 100 % size.
    pub fn blur(&self, now: u64) -> f32 {
        match self.phase(now) {
            (Phase::Decoding, _) => 2.0,
            (Phase::Done, _) => 0.0,
            _ => 2.0 + 16.0 * (1.0 - self.denoised(now)).powf(1.6),
        }
    }

    /// The mosaic's opacity over the prediction.
    pub fn mosaic(&self, now: u64) -> f32 {
        match self.phase(now) {
            (Phase::Queued, _) => 0.0,
            (Phase::Generating, _) | (Phase::Error, _) => 0.15 + 0.85 * self.noise(now),
            _ => 0.0,
        }
    }

    /// How far down the crisp decode has reached, 0..1.
    pub fn decoded(&self, now: u64) -> f32 {
        match self.phase(now) {
            (Phase::Decoding, p) => p,
            (Phase::Done, _) => 1.0,
            _ => 0.0,
        }
    }

    /// Seconds the run took (`4.6s`).
    pub fn took(&self) -> String {
        super::secs(self.duration + DECODE_MS)
    }

    /// Presses: `ir-retry`, `ir-again`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ir-retry" if self.phase(now).0 == Phase::Error => {
                self.fail = false;
                self.t0 = now;
                self.delay = 0;
                Some("Generating image".into())
            }
            "ir-again" => {
                self.seed += 1;
                self.t0 = now;
                self.delay = 0;
                Some("Generating image".into())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_runs_and_loops() {
        let r = Reveal::new(4_600, 30, 0, true, false, 1, 0);
        assert_eq!(r.phase(100).0, Phase::Queued);
        assert_eq!(r.phase(QUEUE_MS + 10).0, Phase::Generating);
        assert_eq!(r.step(QUEUE_MS + 10), 1);
        assert!(r.blur(QUEUE_MS + 10) > r.blur(QUEUE_MS + 4_000));
        assert_eq!(r.phase(QUEUE_MS + 4_700).0, Phase::Decoding);
        assert_eq!(r.phase(QUEUE_MS + 5_500).0, Phase::Done);
        assert_eq!(r.phase(r.cycle() + 10).0, Phase::Queued);
        assert_ne!(r.scene(0), r.scene(r.cycle() + 10));
        let mut f = Reveal::new(4_600, 30, 0, true, true, 0, 0);
        assert_eq!(f.phase(QUEUE_MS + 3_000).0, Phase::Error);
        assert_eq!(f.step(QUEUE_MS + 3_000), 19);
        f.press("ir-retry", 9_000);
        assert_eq!(f.phase(9_000 + QUEUE_MS + 10).0, Phase::Generating);
        assert_eq!(latent(0, 1, 3, 0.5).len(), GRID * GRID);
        assert_eq!(color_at(&scene(0), (64.0, 45.0)), 0xffd38a);
        assert_eq!(r.took(), "5.2s");
    }
}
