//! Status-text views (family C). Hold the `oxyn_core::text` engines in your view, update them
//! with the agent state each frame, and render these views from their output.

use gpui::{
    App, FontWeight, HighlightStyle, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    StyledText, Window, div, px,
};
use oxyn_core::text::{
    AsciiTier, Caret, CaretVariant, CyclerVariant, CyclerView, RevealSpan, ShimmerPaint, TimerView,
    graphemes,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::{hsla, mix};

/// Average advance of a UI glyph as a fraction of the font size, used to place the shimmer band
/// when the host has no measured glyph positions.
const ADVANCE_EM: f32 = 0.55;

/// Shimmering status text: one text run per style span (no per-glyph elements).
#[derive(IntoElement)]
pub struct ShimmerText {
    text: SharedString,
    paint: ShimmerPaint,
    base_level: f32,
    warn_level: f32,
    ascii: bool,
    theme: &'static Theme,
    font_px: f32,
}

impl ShimmerText {
    /// From `Shimmer::paint`, `base_level` and `warn_level`.
    pub fn new(
        text: impl Into<SharedString>,
        paint: ShimmerPaint,
        base_level: f32,
        warn_level: f32,
    ) -> Self {
        Self {
            text: text.into(),
            paint,
            base_level,
            warn_level,
            ascii: false,
            theme: &Theme::DARK,
            font_px: 13.0,
        }
    }

    /// Terminal tiers instead of colour (the `ascii` variant).
    pub fn ascii(mut self, on: bool) -> Self {
        self.ascii = on;
        self
    }

    /// Theme.
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Font size in px.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_px = px;
        self
    }
}

impl RenderOnce for ShimmerText {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fg = t.color(ColorRole::TextStatusFg);
        let base = mix(t.color(ColorRole::TextStatusBase), fg, self.base_level);
        let base = mix(base, t.color(ColorRole::StatusWarningFg), self.warn_level);
        let crest = t.color(ColorRole::TextStatusCrest);
        let mut highlights = Vec::new();
        let mut offset = 0;
        for (i, g) in graphemes(&self.text).enumerate() {
            let x = (i as f32 + 0.5) * self.font_px * ADVANCE_EM;
            let k = self.paint.at(x);
            let range = offset..offset + g.len();
            offset += g.len();
            let style = if self.ascii {
                match AsciiTier::of(k) {
                    AsciiTier::Dim => HighlightStyle {
                        color: Some(hsla(mix(fg, t.color(ColorRole::BgBase), 0.45))),
                        ..Default::default()
                    },
                    AsciiTier::Normal => HighlightStyle {
                        color: Some(hsla(fg)),
                        ..Default::default()
                    },
                    AsciiTier::Bold => HighlightStyle {
                        color: Some(hsla(fg)),
                        font_weight: Some(FontWeight::BOLD),
                        ..Default::default()
                    },
                }
            } else {
                HighlightStyle {
                    color: Some(hsla(mix(base, crest, k))),
                    ..Default::default()
                }
            };
            highlights.push((range, style));
        }
        div()
            .text_size(px(self.font_px))
            .child(StyledText::new(self.text).with_highlights(highlights))
    }
}

/// A verb cycler frame: outgoing and incoming verbs stacked in one clipped slot.
#[derive(IntoElement)]
pub struct VerbCyclerView {
    view: CyclerView,
    variant: CyclerVariant,
    reduced: bool,
    theme: &'static Theme,
    font_px: f32,
}

impl VerbCyclerView {
    /// From `VerbCycler::view`.
    pub fn new(view: CyclerView, variant: CyclerVariant) -> Self {
        Self {
            view,
            variant,
            reduced: false,
            theme: &Theme::DARK,
            font_px: 13.0,
        }
    }

    /// Reduced motion: crossfade only.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }

    /// Theme.
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Font size in px.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_px = px;
        self
    }
}

impl RenderOnce for VerbCyclerView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colour = hsla(self.theme.color(ColorRole::TextStatusFg));
        let fs = self.font_px;
        let line = fs * 1.35;
        // GPUI has no text blur; blur-swap draws as a fade here.
        let offset = |t: f32, out: bool| -> f32 {
            let k = if out { t } else { 1.0 - t };
            let dir = if out { -1.0 } else { 1.0 };
            match (self.variant, self.reduced) {
                (_, true) | (CyclerVariant::Static | CyclerVariant::BlurSwap, _) => 0.0,
                (CyclerVariant::Slide, _) => dir * 0.6 * fs * k,
                _ => dir * 4.0 * k,
            }
        };
        let layer = |text: String, op: f32, dy: f32| {
            div()
                .absolute()
                .left_0()
                .top(px(dy))
                .opacity(op)
                .whitespace_nowrap()
                .child(text)
        };
        let mut incoming = self.view.text.clone();
        if self.variant == CyclerVariant::Ellipsis && !self.view.settled && !self.view.paused {
            incoming.push('…');
        }
        let width = self
            .view
            .outgoing
            .as_ref()
            .map_or(0, |o| o.chars().count())
            .max(incoming.chars().count()) as f32
            * fs
            * 0.55;
        let mut slot = div()
            .relative()
            .overflow_hidden()
            .h(px(line))
            .w(px(width))
            .text_size(px(fs))
            .text_color(colour);
        if let Some(o) = self.view.outgoing.clone() {
            slot = slot.child(layer(
                o,
                1.0 - self.view.out_t,
                offset(self.view.out_t, true),
            ));
        }
        slot.child(layer(
            incoming,
            self.view.in_t,
            offset(self.view.in_t, false),
        ))
    }
}

/// Timer text with tabular digits in a reserved slot.
#[derive(IntoElement)]
pub struct TimerText {
    view: TimerView,
    theme: &'static Theme,
    font_px: f32,
}

impl TimerText {
    /// From `timer_view`.
    pub fn new(view: TimerView) -> Self {
        Self {
            view,
            theme: &Theme::DARK,
            font_px: 13.0,
        }
    }

    /// Theme.
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Font size in px.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_px = px;
        self
    }
}

impl RenderOnce for TimerText {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let role = if self.view.paused {
            ColorRole::FgMuted
        } else {
            ColorRole::TextStatusFg
        };
        let mut text = self.view.text;
        if self.view.paused {
            text.push_str(" ⏸");
        }
        // Mono digits keep `0:09 → 0:10` from wiggling until GPUI exposes `tnum`.
        div()
            .font_family("JetBrains Mono")
            .text_size(px(self.font_px))
            .min_w(px(self.view.reserve.chars().count() as f32
                * self.font_px
                * 0.6))
            .text_color(hsla(self.theme.color(role)))
            .child(text)
    }
}

/// Streaming text spans plus the caret.
#[derive(IntoElement)]
pub struct RevealText {
    spans: Vec<RevealSpan>,
    caret: Option<(CaretVariant, Caret)>,
    theme: &'static Theme,
    font_px: f32,
}

impl RevealText {
    /// From `Reveal::spans` and, when the host wants one, `Reveal::caret`.
    pub fn new(spans: Vec<RevealSpan>, caret: Option<(CaretVariant, Caret)>) -> Self {
        Self {
            spans,
            caret,
            theme: &Theme::DARK,
            font_px: 15.0,
        }
    }

    /// Theme.
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Font size in px.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_px = px;
        self
    }
}

impl RenderOnce for RevealText {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let fg = self.theme.color(ColorRole::FgDefault);
        let fs = self.font_px;
        // One styled run per span; GPUI has no text blur, so word-blur draws as fade + lift.
        let mut text = String::new();
        let mut highlights = Vec::new();
        for s in &self.spans {
            let start = text.len();
            text.push_str(&s.text);
            let mut c = hsla(fg);
            c.a *= s.opacity;
            highlights.push((
                start..text.len(),
                HighlightStyle {
                    color: Some(c),
                    ..Default::default()
                },
            ));
        }
        let mut row = div()
            .flex()
            .flex_wrap()
            .items_end()
            .text_size(px(fs))
            .child(StyledText::new(text).with_highlights(highlights));
        if let Some((variant, caret)) = self.caret.filter(|(_, c)| c.opacity > 0.0) {
            let mut c = hsla(fg);
            c.a *= if caret.dim { 0.45 } else { caret.opacity };
            let (w, h) = match variant {
                CaretVariant::Bar => (2.0, fs * 1.1),
                CaretVariant::Block => {
                    c.a *= 0.7;
                    (fs * 0.55, fs * 1.1)
                }
                CaretVariant::Underscore => (fs * 0.55, 2.0),
            };
            row = row.child(div().w(px(w)).h(px(h)).rounded(px(1.0)).bg(c));
        }
        row
    }
}
