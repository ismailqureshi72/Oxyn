//! Status-text effects (family C, `docs/specs/status-text.md`): shimmer, verb cycler, elapsed
//! timer, streaming reveal and carets. Every effect is a pure function of the shared clock time;
//! nothing here owns a timer, and nothing changes advance widths except append-only reveals.

mod cycler;
mod reveal;
mod shimmer;
mod timer;

pub use cycler::{CyclerVariant, CyclerView, VerbCycler};
pub use reveal::{Caret, CaretVariant, Gear, Reveal, RevealSpan, RevealVariant};
pub use shimmer::{AsciiTier, Shimmer, ShimmerPaint, ShimmerVariant, dim_factor};
pub use timer::{
    RunMeter, TimerFormat, TimerVariant, TimerView, format_clock, format_cost, format_rate,
    format_tokens, timer_view,
};

use unicode_segmentation::UnicodeSegmentation;

/// Family-C motion tokens (`status-text.md` §4).
pub mod timing {
    use std::time::Duration;

    /// One shimmer band crossing a line of up to 200 px.
    pub const SHIMMER_SWEEP: Duration = Duration::from_millis(1400);
    /// Gap between sweeps.
    pub const SHIMMER_REST: Duration = Duration::from_millis(800);
    /// Wait after new text before the first sweep.
    pub const SHIMMER_DELAY: Duration = Duration::from_millis(600);
    /// Minimum dwell of a cycled verb.
    pub const VERB_DWELL: Duration = Duration::from_millis(2800);
    /// Per-token reveal fade.
    pub const REVEAL_TOKEN: Duration = Duration::from_millis(160);
    /// Present → past roll at completion.
    pub const TENSE_ROLL: Duration = Duration::from_millis(240);
    /// Reduced-motion crossfade.
    pub const REDUCED_FADE: Duration = Duration::from_millis(80);
}

/// Grapheme clusters (UAX #29) of `s`.
pub fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    s.graphemes(true)
}

/// Grapheme count, with CJK weighted ×1.6 for reading time (`verb-cycler.md` §3).
pub fn reading_weight(s: &str) -> f32 {
    s.graphemes(true)
        .map(|g| {
            let c = g.chars().next().unwrap_or(' ');
            if matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x9FFF | 0xAC00..=0xD7AF) {
                1.6
            } else {
                1.0
            }
        })
        .sum()
}

/// `ease.out` (cubic).
pub(crate) fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

/// `ease.in` (cubic).
pub(crate) fn ease_in(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * x
}
