//! Slider (`slider.md`): one value or a range on a track that follows the finger. min / max /
//! step / large step, marks, minimum steps between thumbs, formatted readout, keyboard. Oxyn plus:
//! named detents that snap (effort levels), a live readout tag (e.g. cost estimate) and a
//! vertical orientation.

use super::nav::NavKey;

/// Slider keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderKey {
    /// → / ↑.
    Inc,
    /// ← / ↓.
    Dec,
    /// Shift+Arrow / Page Up.
    IncLarge,
    /// Shift+Arrow / Page Down.
    DecLarge,
    /// Home.
    Min,
    /// End.
    Max,
}

impl From<NavKey> for SliderKey {
    fn from(k: NavKey) -> Self {
        match k {
            NavKey::Next => Self::Inc,
            NavKey::Prev => Self::Dec,
            NavKey::PageForward => Self::IncLarge,
            NavKey::PageBack => Self::DecLarge,
            NavKey::First => Self::Min,
            NavKey::Last => Self::Max,
        }
    }
}

/// A mark under the track.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    /// Value.
    pub value: f64,
    /// Label (empty = tick only).
    pub label: String,
    /// Oxyn plus: a detent the thumb snaps to within `snap` of.
    pub detent: bool,
}

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Thumb glide on click / key (spring.snap).
    pub const GLIDE: Duration = oxyn_tokens::motion::spring::SNAP;
    /// Bubble springs up over an active thumb.
    pub const BUBBLE: Duration = oxyn_tokens::motion::dur::FAST;
    /// Released drags carry momentum this long before snapping (Arc).
    pub const MOMENTUM: Duration = Duration::from_millis(180);
    /// At a limit the thumb strains this many px past the end, then springs back.
    pub const STRAIN: f32 = 6.0;
}

/// The slider.
#[derive(Clone, Debug, PartialEq)]
pub struct Slider {
    /// Values: one (single) or two (range).
    pub values: Vec<f64>,
    /// Minimum.
    pub min: f64,
    /// Maximum.
    pub max: f64,
    /// Step.
    pub step: f64,
    /// Large step (Shift / Page keys); default 10 steps.
    pub large_step: Option<f64>,
    /// Marks.
    pub marks: Vec<Mark>,
    /// Range: thumbs keep at least this many steps apart.
    pub min_steps_between: u32,
    /// Detent snap distance in value units (Oxyn plus).
    pub snap: f64,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: vertical.
    pub vertical: bool,
    /// Thumb being dragged.
    pub active: Option<usize>,
}

impl Slider {
    /// A single-value slider.
    pub fn new(value: f64, min: f64, max: f64, step: f64) -> Self {
        Self {
            values: vec![value],
            min,
            max,
            step,
            large_step: None,
            marks: vec![],
            min_steps_between: 0,
            snap: 0.0,
            disabled: false,
            vertical: false,
            active: None,
        }
        .normalised()
    }
    /// A range slider.
    pub fn range(start: f64, end: f64, min: f64, max: f64, step: f64) -> Self {
        Self {
            values: vec![start, end],
            ..Self::new(start, min, max, step)
        }
        .normalised()
    }
    fn normalised(mut self) -> Self {
        for i in 0..self.values.len() {
            let v = self.values[i];
            self.values[i] = self.clamp_thumb(i, v);
        }
        self
    }
    /// Snap to the step grid (and detents), clamp to `[min, max]`.
    pub fn quantise(&self, v: f64) -> f64 {
        let v = v.clamp(self.min, self.max);
        let v = if self.step > 0.0 {
            let n = (((v - self.min) / self.step * 1e6).round() / 1e6).round();
            (self.min + n * self.step).clamp(self.min, self.max)
        } else {
            v
        };
        if self.snap > 0.0
            && let Some(d) = self
                .marks
                .iter()
                .filter(|m| m.detent && (m.value - v).abs() <= self.snap)
                .min_by(|a, b| {
                    (a.value - v)
                        .abs()
                        .partial_cmp(&(b.value - v).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        {
            return d.value;
        }
        // Avoid 0.30000000000000004.
        (v * 1e9).round() / 1e9
    }
    fn gap(&self) -> f64 {
        self.step * f64::from(self.min_steps_between)
    }
    fn clamp_thumb(&self, i: usize, v: f64) -> f64 {
        let v = self.quantise(v);
        if self.values.len() < 2 {
            return v;
        }
        let g = self.gap();
        if i == 0 {
            v.min(self.values[1] - g)
        } else {
            v.max(self.values[0] + g)
        }
    }
    /// Sets thumb `i`; returns true when it changed (`onValueChange`).
    pub fn set(&mut self, i: usize, v: f64) -> bool {
        if self.disabled || i >= self.values.len() {
            return false;
        }
        let v = self.clamp_thumb(i, v);
        let changed = (self.values[i] - v).abs() > f64::EPSILON;
        self.values[i] = v;
        changed
    }
    /// Keyboard on thumb `i`.
    pub fn key(&mut self, i: usize, k: SliderKey) -> bool {
        let large = self.large_step.unwrap_or(self.step * 10.0);
        let v = *self.values.get(i).unwrap_or(&self.min);
        let to = match k {
            SliderKey::Inc => v + self.step,
            SliderKey::Dec => v - self.step,
            SliderKey::IncLarge => v + large,
            SliderKey::DecLarge => v - large,
            SliderKey::Min => self.min,
            SliderKey::Max => self.max,
        };
        // Detents: arrow keys jump detent to detent when every mark is a detent.
        if self.snap > 0.0 && !self.marks.is_empty() && self.marks.iter().all(|m| m.detent) {
            let up = matches!(k, SliderKey::Inc | SliderKey::IncLarge);
            let next = if up {
                self.marks
                    .iter()
                    .map(|m| m.value)
                    .filter(|&m| m > v + 1e-9)
                    .reduce(f64::min)
            } else {
                self.marks
                    .iter()
                    .map(|m| m.value)
                    .filter(|&m| m < v - 1e-9)
                    .reduce(f64::max)
            };
            if matches!(
                k,
                SliderKey::Inc | SliderKey::Dec | SliderKey::IncLarge | SliderKey::DecLarge
            ) {
                return next.is_some_and(|n| self.set(i, n));
            }
        }
        self.set(i, to)
    }
    /// Fraction 0..1 of a value along the track.
    pub fn fraction(&self, v: f64) -> f64 {
        if self.max <= self.min {
            0.0
        } else {
            ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }
    /// Value at a pointer `x` on a track `len` px long (vertical tracks count from the bottom).
    pub fn value_at(&self, x: f32, len: f32) -> f64 {
        let f = f64::from((x / len.max(1.0)).clamp(0.0, 1.0));
        let f = if self.vertical { 1.0 - f } else { f };
        self.quantise(self.min + f * (self.max - self.min))
    }
    /// Pointer down at `x`: the nearer thumb takes it (range) and jumps there.
    pub fn press(&mut self, x: f32, len: f32) -> usize {
        let v = self.value_at(x, len);
        let i = if self.values.len() == 2 {
            if (v - self.values[0]).abs() <= (v - self.values[1]).abs() {
                0
            } else {
                1
            }
        } else {
            0
        };
        self.active = Some(i);
        self.set(i, v);
        i
    }
    /// Drag to `x`.
    pub fn drag(&mut self, x: f32, len: f32) -> bool {
        let Some(i) = self.active else { return false };
        let v = self.value_at(x, len);
        self.set(i, v)
    }
    /// Release (`onValueCommit`); returns the committed values.
    pub fn release(&mut self) -> Vec<f64> {
        self.active = None;
        self.values.clone()
    }
    /// The filled part `(from, to)` as fractions.
    pub fn fill(&self) -> (f64, f64) {
        match self.values.as_slice() {
            [v] => (0.0, self.fraction(*v)),
            [a, b] => (self.fraction(*a), self.fraction(*b)),
            _ => (0.0, 0.0),
        }
    }
    /// How far a drag past the end strains, in px (rubber band, max [`timing::STRAIN`]).
    pub fn strain(x: f32, len: f32) -> f32 {
        let over = if x < 0.0 {
            x
        } else if x > len {
            x - len
        } else {
            return 0.0;
        };
        let s = timing::STRAIN * (1.0 - (-over.abs() / 40.0).exp());
        s.copysign(over)
    }
    /// `aria-valuetext` for thumb `i` using `fmt`; the nearest detent label wins.
    pub fn value_text(&self, i: usize, fmt: impl Fn(f64) -> String) -> String {
        let v = self.values.get(i).copied().unwrap_or(self.min);
        self.marks
            .iter()
            .find(|m| m.detent && !m.label.is_empty() && (m.value - v).abs() < 1e-9)
            .map(|m| m.label.clone())
            .unwrap_or_else(|| fmt(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_keys_and_clamps() {
        let mut s = Slider::new(0.35, 0.0, 1.0, 0.1);
        assert_eq!(s.values[0], 0.4);
        assert!(s.key(0, SliderKey::Inc));
        assert_eq!(s.values[0], 0.5);
        s.key(0, SliderKey::IncLarge);
        assert_eq!(s.values[0], 1.0);
        assert!(!s.key(0, SliderKey::Inc), "at the limit");
        s.key(0, SliderKey::Min);
        assert_eq!(s.values[0], 0.0);
        s.disabled = true;
        assert!(!s.key(0, SliderKey::Max));
    }

    #[test]
    fn range_keeps_gap_and_nearest_thumb_takes_press() {
        let mut r = Slider::range(20.0, 60.0, 0.0, 100.0, 5.0);
        r.min_steps_between = 2;
        r.set(0, 59.0);
        assert_eq!(r.values[0], 50.0);
        assert_eq!(r.press(90.0, 100.0), 1);
        assert_eq!(r.values[1], 90.0);
        r.drag(10.0, 100.0);
        assert_eq!(r.values[1], 60.0, "end cannot cross the start");
        assert_eq!(r.release(), vec![50.0, 60.0]);
        assert_eq!(r.fill(), (0.5, 0.6));
    }

    #[test]
    fn detents_snap_and_label() {
        let mut s = Slider::new(0.0, 0.0, 3.0, 0.01);
        s.snap = 0.2;
        s.marks = ["Low", "Medium", "High", "Max"]
            .iter()
            .enumerate()
            .map(|(i, l)| Mark {
                value: i as f64,
                label: (*l).into(),
                detent: true,
            })
            .collect();
        s.set(0, 1.15);
        assert_eq!(s.values[0], 1.0);
        assert_eq!(s.value_text(0, |v| format!("{v}")), "Medium");
        s.key(0, SliderKey::Inc);
        assert_eq!(s.values[0], 2.0, "arrow keys jump detent to detent");
        s.set(0, 1.5);
        assert_eq!(s.value_text(0, |v| format!("{v:.1}")), "1.5");
    }

    #[test]
    fn pointer_mapping_vertical_and_strain() {
        let mut s = Slider::new(0.0, 0.0, 10.0, 1.0);
        assert_eq!(s.value_at(50.0, 100.0), 5.0);
        s.vertical = true;
        assert_eq!(s.value_at(20.0, 100.0), 8.0);
        assert_eq!(Slider::strain(50.0, 100.0), 0.0);
        let st = Slider::strain(400.0, 100.0);
        assert!(st > 5.0 && st <= 6.0);
        assert!(Slider::strain(-30.0, 100.0) < 0.0);
    }
}
