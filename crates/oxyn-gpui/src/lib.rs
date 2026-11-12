//! GPUI components for Oxyn (primary renderer).
//!
//! Components take `&AgentState` plus props and read time from the window's shared
//! [`MotionClock`]; the host keeps the clock advanced and requests frames while
//! [`MotionClock::next_frame_in`] says so.

mod status_indicator;

pub use status_indicator::{IndicatorSize, IndicatorVariant, StatusIndicator};

use gpui::{Hsla, Rgba};
use oxyn_tokens::Color;

#[doc(no_inline)]
pub use oxyn_core::MotionClock;

/// Converts a token colour to GPUI's colour type.
pub fn hsla(c: Color) -> Hsla {
    Rgba {
        r: f32::from(c.r) / 255.0,
        g: f32::from(c.g) / 255.0,
        b: f32::from(c.b) / 255.0,
        a: c.a,
    }
    .into()
}

/// Linear mix of two token colours in sRGB (`t` in `[0, 1]`).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color::rgba(
        ch(a.r, b.r),
        ch(a.g, b.g),
        ch(a.b, b.b),
        a.a + (b.a - a.a) * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_conversion() {
        let h = hsla(Color::rgb(255, 90, 31));
        let back: Rgba = h.into();
        assert!((back.r - 1.0).abs() < 1e-3 && (back.g - 90.0 / 255.0).abs() < 1e-3);
        assert_eq!(
            mix(Color::rgb(0, 0, 0), Color::rgb(200, 100, 50), 0.5),
            Color::rgb(100, 50, 25)
        );
    }
}
