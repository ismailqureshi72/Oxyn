//! Loaders and moving text for GPUI (plan #28, chunk 5), drawn from `oxyn_core::parity::text`.
//!
//! Elements render one frame at the clock they are given; the host re-renders on its own
//! animation tick. With `reduced(true)` nothing moves: the shimmer band is dropped, reels show
//! their landed value and loaders show a still frame.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, px,
};
use oxyn_core::{
    ClockMs,
    parity::text::{Phases, SlotText, elapsed_label, pixel_frame},
};
use oxyn_tokens::{ColorRole, Theme};

use super::{alpha, c};

/// One pass of the shimmer band, ms, and the pause after it.
const SWEEP_MS: u64 = 1600;
const SWEEP_PAUSE_MS: u64 = 600;

/// Per-character colours for a band of light passing over `text` at `now`.
fn sweep_chars(text: &str, now: ClockMs, base: Hsla, hi: Hsla, active: bool) -> Vec<(char, Hsla)> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len().max(1) as f32;
    let cycle = SWEEP_MS + SWEEP_PAUSE_MS;
    let t = (now.0 % cycle) as f32 / SWEEP_MS as f32;
    // Band centre runs from before the first letter to past the last.
    let centre = -3.0 + t * (n + 6.0);
    chars
        .into_iter()
        .enumerate()
        .map(|(i, ch)| {
            if !active || t > 1.0 {
                return (ch, base);
            }
            let d = ((i as f32 - centre).abs() / 3.0).min(1.0);
            let k = 1.0 - d * d;
            let mix = Hsla {
                h: base.h,
                s: base.s + (hi.s - base.s) * k,
                l: base.l + (hi.l - base.l) * k,
                a: base.a + (hi.a - base.a) * k,
            };
            (ch, mix)
        })
        .collect()
}

fn sweep_row(t: &'static Theme, text: &str, now: ClockMs, active: bool) -> gpui::Div {
    let base = c(t, ColorRole::FgMuted);
    let hi = c(t, ColorRole::FgDefault);
    div().flex().children(
        sweep_chars(text, now, base, hi, active)
            .into_iter()
            .map(|(ch, col)| {
                div()
                    .text_color(col)
                    .child(SharedString::from(if ch == ' ' {
                        "\u{a0}".to_string()
                    } else {
                        ch.to_string()
                    }))
            }),
    )
}

/// `shimmer-label` (Text shimmer): a calm band of light across the words while active.
#[derive(IntoElement)]
pub struct ShimmerEl {
    label: SharedString,
    now: ClockMs,
    active: bool,
    reduced: bool,
    theme: &'static Theme,
}

impl ShimmerEl {
    /// `label` at `now`, active.
    pub fn new(label: impl Into<SharedString>, now: ClockMs) -> Self {
        Self {
            label: label.into(),
            now,
            active: true,
            reduced: false,
            theme: &Theme::DARK,
        }
    }
    /// Stops the band (finished work).
    pub fn active(mut self, on: bool) -> Self {
        self.active = on;
        self
    }
    /// Reduced motion: no band.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }
}

impl RenderOnce for ShimmerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        sweep_row(
            self.theme,
            &self.label,
            self.now,
            self.active && !self.reduced,
        )
        .text_size(px(13.))
    }
}

/// `thinking-loader`: an eight-ray spinner and a shimmering phase label that settles on a check
/// with "Thought for Ns".
#[derive(IntoElement)]
pub struct ThinkingLoaderEl {
    phases: Phases,
    now: ClockMs,
    reduced: bool,
    theme: &'static Theme,
}

impl ThinkingLoaderEl {
    /// The loader at `now`.
    pub fn new(phases: Phases, now: ClockMs) -> Self {
        Self {
            phases,
            now,
            reduced: false,
            theme: &Theme::DARK,
        }
    }
    /// Reduced motion: a still spinner, no shimmer.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }
}

impl RenderOnce for ThinkingLoaderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let done = self.phases.done.is_some();
        let fg = c(t, ColorRole::FgDefault);
        // Eight rays; the lit one steps every 90 ms, the trail fades behind it.
        let lit = if self.reduced {
            0
        } else {
            (self.now.0 / 90 % 8) as usize
        };
        let glyph = if done {
            div()
                .size(px(14.))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.))
                .text_color(fg)
                .child("✓")
        } else {
            // A 3×3 ring of dots standing in for the rays (centre empty).
            const RING: [usize; 8] = [0, 1, 2, 5, 8, 7, 6, 3];
            let mut a = [0.0f32; 9];
            for (k, cell) in RING.iter().enumerate() {
                let back = (lit + 8 - k) % 8;
                a[*cell] = (1.0 - back as f32 * 0.14).max(0.18);
            }
            div().size(px(14.)).flex().flex_wrap().gap(px(1.)).children(
                a.into_iter()
                    .map(move |x| div().size(px(4.)).rounded_full().bg(alpha(fg, x))),
            )
        };
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(30.))
            .px(px(12.))
            .rounded(px(999.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_size(px(12.5))
            .child(glyph)
            .child(sweep_row(
                t,
                &self.phases.label(self.now),
                self.now,
                !done && !self.reduced,
            ))
    }
}

/// `slot-text`: reels spinning into the new value, stopping left to right.
#[derive(IntoElement)]
pub struct SlotTextEl {
    model: SlotText,
    now: ClockMs,
    reduced: bool,
    size: f32,
    theme: &'static Theme,
}

impl SlotTextEl {
    /// The reels at `now`.
    pub fn new(model: SlotText, now: ClockMs) -> Self {
        Self {
            model,
            now,
            reduced: false,
            size: 28.,
            theme: &Theme::DARK,
        }
    }
    /// Reduced motion: the landed value, no spin.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }
    /// Font size, px.
    pub fn size(mut self, px_size: f32) -> Self {
        self.size = px_size;
        self
    }
}

impl RenderOnce for SlotTextEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fg = c(t, ColorRole::FgDefault);
        let h = self.size * 1.2;
        let m = &self.model;
        let now = self.now;
        let settled = self.reduced || m.done(now);
        let reels: Vec<_> = m
            .reels()
            .into_iter()
            .filter(|r| !(settled && r.grow == Some(false)))
            .collect();
        div()
            .flex()
            .h(px(h))
            .overflow_hidden()
            .text_size(px(self.size))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(fg)
            .children(reels.into_iter().map(move |r| {
                let last = r.strip.len() - 1;
                let pos = if settled { last as f32 } else { m.pos(&r, now) };
                // Growing reels widen in, collapsing ones narrow out over the spin.
                let k = if settled {
                    1.0
                } else {
                    (now.0.saturating_sub(m.at.0) as f32 / r.stop_ms.max(1) as f32).clamp(0.0, 1.0)
                };
                let wscale = match r.grow {
                    Some(true) => k,
                    Some(false) => 1.0 - k,
                    None => 1.0,
                };
                let w = self.size * 0.62 * wscale;
                div().w(px(w)).h(px(h)).overflow_hidden().relative().child(
                    div()
                        .absolute()
                        .left_0()
                        .top(px(-pos * h))
                        .flex()
                        .flex_col()
                        .children(r.strip.into_iter().map(move |ch| {
                            div()
                                .h(px(h))
                                .flex()
                                .items_center()
                                .child(SharedString::from(ch.to_string()))
                        })),
                )
            }))
    }
}

/// `loading-state`: a 4×4 pixel-grid loader, a shimmering verb and the elapsed time.
#[derive(IntoElement)]
pub struct LoadingStateEl {
    variant: SharedString,
    verb: SharedString,
    started: ClockMs,
    now: ClockMs,
    reduced: bool,
    theme: &'static Theme,
}

impl LoadingStateEl {
    /// `variant` is `drive`, `dots`, `orbit` or `surfer`.
    pub fn new(
        variant: impl Into<SharedString>,
        verb: impl Into<SharedString>,
        started: ClockMs,
        now: ClockMs,
    ) -> Self {
        Self {
            variant: variant.into(),
            verb: verb.into(),
            started,
            now,
            reduced: false,
            theme: &Theme::DARK,
        }
    }
    /// Reduced motion: a still frame, no shimmer.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }
}

impl RenderOnce for LoadingStateEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fg = c(t, ColorRole::FgDefault);
        let frame = pixel_frame(
            &self.variant,
            if self.reduced { ClockMs(0) } else { self.now },
        );
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .text_size(px(13.))
            .child(
                div()
                    .w(px(17.))
                    .flex()
                    .flex_wrap()
                    .gap(px(1.))
                    .children(frame.into_iter().map(move |v| {
                        div()
                            .size(px(3.5))
                            .rounded(px(0.5))
                            .bg(alpha(fg, [0.12, 0.35, 0.65, 1.0][v.min(3) as usize]))
                    })),
            )
            .child(sweep_row(t, &self.verb, self.now, !self.reduced))
            .child(
                div()
                    .text_color(c(t, ColorRole::FgMuted))
                    .text_size(px(12.))
                    .child(elapsed_label(self.started, self.now)),
            )
    }
}

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
