//! Thinking Selector family (variants 1): the shared model behind every effort control – how hard
//! a model thinks before it answers. A scale of named levels, stepping and snapping a position on a
//! track, the tick ruler that tints toward the knob, and the dotted fill a maxed-out bar shows.
//! Behaviour only; each variant's own module adds what only it needs.

/// One step of a thinking scale.
#[derive(Clone, Debug, PartialEq)]
pub struct EffortLevel {
    /// Stable id (`low`, `max` …).
    pub id: String,
    /// What the control shows (`Low`, `Max` …).
    pub label: String,
    /// One short line on what the level does.
    pub detail: String,
}

impl EffortLevel {
    /// A level from its id, label and line.
    pub fn new(id: &str, label: &str, detail: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
        }
    }
}

/// The five-step scale most variants use: Low, Medium, High, Extra high, Max.
pub fn effort_scale() -> Vec<EffortLevel> {
    vec![
        EffortLevel::new("low", "Low", "Answers right away"),
        EffortLevel::new("medium", "Medium", "Thinks for a moment"),
        EffortLevel::new("high", "High", "Works through the steps"),
        EffortLevel::new("xhigh", "Extra high", "Checks its own work"),
        EffortLevel::new("max", "Max", "Takes as long as the task needs"),
    ]
}

/// A four-step scale: Minimal, Low, Medium, High.
pub fn effort_scale_short() -> Vec<EffortLevel> {
    vec![
        EffortLevel::new("minimal", "Minimal", "Skips thinking where it can"),
        EffortLevel::new("low", "Low", "A quick pass"),
        EffortLevel::new("medium", "Medium", "Balanced"),
        EffortLevel::new("high", "High", "Thorough"),
    ]
}

/// The lowercase scale an API exposes: default, low, medium, high, xhigh, max.
pub fn effort_scale_api() -> Vec<EffortLevel> {
    let mut v = vec![EffortLevel::new("default", "default", "The model decides")];
    for l in effort_scale() {
        v.push(EffortLevel::new(&l.id, &l.id, &l.detail));
    }
    v
}

/// The level whose id is `id` (its index), or `fallback` when none matches.
pub fn level_index(levels: &[EffortLevel], id: &str, fallback: usize) -> usize {
    levels
        .iter()
        .position(|l| l.id == id)
        .unwrap_or(fallback.min(levels.len().saturating_sub(1)))
}

/// One keyboard step from `i` on a scale of `n`: `delta` -1/+1, clamped; `i64::MIN`/`MAX` jump
/// to the ends (Home / End).
pub fn effort_step(i: usize, delta: i64, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let last = (n - 1) as i64;
    match delta {
        i64::MIN => 0,
        i64::MAX => last as usize,
        d => (i as i64 + d).clamp(0, last) as usize,
    }
}

/// The next level, wrapping to the first after the last (a single key like R cycles).
pub fn effort_cycle(i: usize, n: usize) -> usize {
    if n == 0 { 0 } else { (i + 1) % n }
}

/// Where level `i` sits on the track, 0.0 (left) to 1.0 (right).
pub fn level_frac(i: usize, n: usize) -> f32 {
    if n <= 1 {
        1.0
    } else {
        i.min(n - 1) as f32 / (n - 1) as f32
    }
}

/// The nearest level to a point on the track (`frac` 0.0–1.0, clamped): drag and click snap here.
pub fn snap_level(frac: f32, n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    let f = if frac.is_finite() {
        frac.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (f * (n - 1) as f32).round() as usize
}

/// The fraction of a pointer at `x` across a track that starts at `left` and is `width` wide.
pub fn track_frac(x: f64, left: f64, width: f64) -> f32 {
    if width <= 0.0 {
        return 0.0;
    }
    ((x - left) / width).clamp(0.0, 1.0) as f32
}

/// How strongly tick `t` of `ticks` is tinted when the knob sits at `knob` (0.0–1.0): ticks
/// behind the knob carry the accent, fading out over `reach` of the track; ticks past it stay
/// plain. 0.0 plain … 1.0 full accent.
pub fn tick_tint(t: usize, ticks: usize, knob: f32, reach: f32) -> f32 {
    if ticks <= 1 {
        return 1.0;
    }
    let pos = t as f32 / (ticks - 1) as f32;
    if pos > knob + 1e-4 {
        return 0.0;
    }
    let reach = reach.max(0.01);
    let d = knob - pos;
    (1.0 - d / reach).clamp(0.12, 1.0)
}

/// The dotted, pixel-like fill a bar shows at the top level: `cols` × `rows` cells, each lit or
/// dim, the same for the same `seed` (no animation needed to read as "maxed out"). Row-major.
pub fn pixel_fill(cols: usize, rows: usize, seed: u32) -> Vec<bool> {
    let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(0x9E37_79B9) | 1;
    (0..cols * rows)
        .map(|_| {
            // xorshift32
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            s % 100 < 72
        })
        .collect()
}

/// The status line a screen reader hears after a change: "Effort: High. Works through the steps".
pub fn effort_said(l: &EffortLevel) -> String {
    if l.detail.is_empty() {
        format!("Effort: {}", l.label)
    } else {
        format!("Effort: {}. {}", l.label, l.detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales() {
        assert_eq!(effort_scale().len(), 5);
        assert_eq!(effort_scale_short()[0].label, "Minimal");
        let api = effort_scale_api();
        assert_eq!(api[0].id, "default");
        assert_eq!(api.last().unwrap().label, "max");
        assert_eq!(level_index(&effort_scale(), "high", 0), 2);
        assert_eq!(level_index(&effort_scale(), "nope", 9), 4);
    }

    #[test]
    fn stepping() {
        assert_eq!(effort_step(0, -1, 5), 0);
        assert_eq!(effort_step(3, 1, 5), 4);
        assert_eq!(effort_step(4, 1, 5), 4);
        assert_eq!(effort_step(2, i64::MIN, 5), 0);
        assert_eq!(effort_step(2, i64::MAX, 5), 4);
        assert_eq!(effort_step(0, 1, 0), 0);
        assert_eq!(effort_cycle(4, 5), 0);
        assert_eq!(effort_cycle(1, 5), 2);
    }

    #[test]
    fn track() {
        assert_eq!(level_frac(0, 5), 0.0);
        assert_eq!(level_frac(4, 5), 1.0);
        assert_eq!(level_frac(2, 5), 0.5);
        assert_eq!(snap_level(0.6, 5), 2);
        assert_eq!(snap_level(0.9, 5), 4);
        assert_eq!(snap_level(-3.0, 5), 0);
        assert_eq!(snap_level(f32::NAN, 5), 0);
        assert_eq!(track_frac(150.0, 100.0, 200.0), 0.25);
        assert_eq!(track_frac(50.0, 100.0, 200.0), 0.0);
        assert_eq!(track_frac(1.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn ticks_and_pixels() {
        assert_eq!(tick_tint(40, 41, 1.0, 0.5), 1.0);
        assert_eq!(tick_tint(30, 41, 0.5, 0.5), 0.0);
        let near = tick_tint(19, 41, 0.5, 0.5);
        let far = tick_tint(2, 41, 0.5, 0.5);
        assert!(near > far && far >= 0.12);
        let a = pixel_fill(40, 4, 7);
        assert_eq!(a.len(), 160);
        assert_eq!(a, pixel_fill(40, 4, 7));
        assert!(a.iter().any(|x| *x) && a.iter().any(|x| !*x));
        assert_eq!(
            effort_said(&effort_scale()[4]),
            "Effort: Max. Takes as long as the task needs"
        );
    }
}
