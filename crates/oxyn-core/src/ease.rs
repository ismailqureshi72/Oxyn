//! Easing evaluation for the `motion::ease::*` tokens (`docs/motion.md` §11.1).
//!
//! `y = f(x)` for a CSS-style cubic Bézier: solve x(t) = x with Newton–Raphson (4 iterations),
//! fall back to bisection when the slope is flat, then return y(t). Matches CSS
//! `cubic-bezier()` closely enough that GPUI and the web look the same.

use oxyn_tokens::CubicBezier;

/// Something that maps linear progress in `[0, 1]` to eased progress.
pub trait Ease {
    /// Eased value at `x` (clamped to `[0, 1]`).
    fn ease(&self, x: f32) -> f32;
}

impl Ease for CubicBezier {
    fn ease(&self, x: f32) -> f32 {
        let [x1, y1, x2, y2] = self.0.map(f64::from);
        let x = f64::from(x.clamp(0.0, 1.0));
        if x1 == y1 && x2 == y2 {
            return x as f32; // linear
        }
        let bez = |t: f64, a: f64, b: f64| {
            let u = 1.0 - t;
            3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
        };
        let slope = |t: f64, a: f64, b: f64| {
            let u = 1.0 - t;
            3.0 * u * u * a + 6.0 * u * t * (b - a) + 3.0 * t * t * (1.0 - b)
        };
        let mut t = x;
        let mut solved = false;
        for _ in 0..4 {
            let err = bez(t, x1, x2) - x;
            if err.abs() < 1e-7 {
                solved = true;
                break;
            }
            let d = slope(t, x1, x2);
            if d.abs() < 1e-6 {
                break;
            }
            t -= err / d;
        }
        if !solved && (bez(t, x1, x2) - x).abs() > 1e-5 {
            let (mut lo, mut hi) = (0.0, 1.0);
            t = x;
            for _ in 0..40 {
                let v = bez(t, x1, x2);
                if (v - x).abs() < 1e-7 {
                    break;
                }
                if v < x {
                    lo = t
                } else {
                    hi = t
                }
                t = (lo + hi) / 2.0;
            }
        }
        bez(t, y1, y2) as f32
    }
}

/// As a plain function, for renderers that take `Fn(f32) -> f32` (GPUI's `with_easing`).
pub fn easing_fn(curve: CubicBezier) -> impl Fn(f32) -> f32 + Copy {
    move |x| curve.ease(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxyn_tokens::motion::ease;

    #[test]
    fn endpoints_and_linear() {
        for c in [
            ease::OUT,
            ease::IN,
            ease::IN_OUT,
            ease::FIRM,
            ease::SINE,
            ease::COAST,
        ] {
            assert!(c.ease(0.0).abs() < 1e-4);
            assert!((c.ease(1.0) - 1.0).abs() < 1e-4);
        }
        assert!((ease::LINEAR.ease(0.37) - 0.37).abs() < 1e-6);
    }

    #[test]
    fn known_values() {
        // Symmetric curves cross 0.5 at 0.5.
        assert!((ease::IN_OUT.ease(0.5) - 0.5).abs() < 1e-3);
        assert!((ease::SINE.ease(0.5) - 0.5).abs() < 1e-3);
        // ease-out is ahead of linear early on, ease-in behind.
        assert!(ease::OUT.ease(0.25) > 0.5);
        assert!(ease::IN.ease(0.25) < 0.1);
        let f = easing_fn(ease::OUT);
        assert!((f(0.25) - ease::OUT.ease(0.25)).abs() < 1e-9);
    }

    #[test]
    fn monotonic() {
        for c in [ease::OUT, ease::IN, ease::IN_OUT, ease::FIRM] {
            let mut prev = 0.0;
            for i in 0..=100 {
                let y = c.ease(i as f32 / 100.0);
                assert!(y + 1e-4 >= prev, "{c:?} at {i}");
                prev = y;
            }
        }
    }
}
