//! Loaders and moving text for GPUI (plan #28, chunk 5), drawn from `oxyn_core::kit::text`.
//!
//! Elements render one frame at the clock they are given; the host re-renders on its own
//! animation tick. With `reduced(true)` nothing moves: the shimmer band is dropped, reels show
//! their landed value and loaders show a still frame.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, px,
};
use oxyn_core::{
    ClockMs,
    kit::text::{Phases, SlotText, elapsed_label, pixel_frame},
};
use oxyn_tokens::{ColorRole, Theme};

use super::{alpha, c};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_rests_when_inactive() {
        let base = Hsla::default();
        let hi = Hsla {
            l: 1.0,
            a: 1.0,
            ..base
        };
        assert!(
            sweep_chars("Thinking", ClockMs(400), base, hi, false)
                .iter()
                .all(|(_, c)| *c == base)
        );
        assert!(
            sweep_chars("Thinking", ClockMs(800), base, hi, true)
                .iter()
                .any(|(_, c)| c.l > 0.5)
        );
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
