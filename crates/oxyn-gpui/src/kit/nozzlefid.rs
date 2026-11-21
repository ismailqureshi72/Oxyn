//! Thinking Selector family, **Nozzle line** for GPUI ([`NozzleSliderEl`]), an Oxyn original over
//! `oxyn_core::kit::{thinkfid, nozzlefid}`: even thin lines in the reference gradient (steel
//! blue → coral → periwinkle; `tone(Red)` for the all-red look) and a solid nozzle in the colour
//! under it. `RenderOnce`: the host keeps the level and routes [`Act`] ids back (`level-<i>`).
//! A static snapshot: no glide or drag.

use gpui::{
    App, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px, rgb,
};
use oxyn_core::kit::{
    nozzlefid::{NozzleTone, line_glow, nozzle_ink, nozzle_rgb},
    thinkfid::{EffortLevel, effort_scale, level_frac},
};
use oxyn_tokens::{ColorRole, Theme};

use super::rulerfid::{LEVEL_ID, fire, th_pal};
use super::{Act, c};

/// `nozzle-slider` (Thinking Selector – Nozzle line).
#[derive(IntoElement)]
pub struct NozzleSliderEl {
    levels: Vec<EffortLevel>,
    value: usize,
    strokes: usize,
    tone: NozzleTone,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for NozzleSliderEl {
    fn default() -> Self {
        Self {
            levels: effort_scale(),
            value: 2,
            strokes: 37,
            tone: NozzleTone::Gradient,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl NozzleSliderEl {
    /// The scale.
    pub fn levels(mut self, l: Vec<EffortLevel>) -> Self {
        if !l.is_empty() {
            self.levels = l;
        }
        self
    }
    /// Level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Line count.
    pub fn strokes(mut self, n: usize) -> Self {
        self.strokes = n.max(2);
        self
    }
    /// Gradient (default) or red.
    pub fn tone(mut self, t: NozzleTone) -> Self {
        self.tone = t;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `level-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for NozzleSliderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let bg: Hsla = rgb(if dark { 0x0D0D0F } else { 0xFFFFFF }).into();
        let lo = 0.85;
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let pos = level_frac(cur, n);
        let (pad, w) = (20.0_f32, 398.0_f32);
        let mut line = div().relative().w(px(w + pad * 2.)).h(px(52.));
        for t in 0..self.strokes {
            let x = t as f32 / (self.strokes - 1) as f32;
            let col: Hsla = rgb(nozzle_rgb(x, self.tone, dark)).into();
            line = line.child(
                div()
                    .absolute()
                    .left(px(pad + w * x - 1.))
                    .top(px(18.))
                    .w(px(2.))
                    .h(px(16.))
                    .rounded(px(2.))
                    .bg(col.opacity(lo + (1. - lo) * line_glow(x, pos))),
            );
        }
        let fill = nozzle_rgb(pos, self.tone, dark);
        let ink: Hsla = rgb(nozzle_ink(fill)).into();
        // The halo hides the lines next to the nozzle, as the CSS box-shadow does.
        line = line.child(
            div()
                .absolute()
                .left(px(pad + w * pos - 18.))
                .top(px(6.))
                .w(px(36.))
                .h(px(40.))
                .rounded(px(12.))
                .bg(bg)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .w(px(28.))
                        .h(px(32.))
                        .rounded_t(px(14.))
                        .rounded_b(px(8.))
                        .border_1()
                        .border_color(rgb(if dark { 0xFFF6EE } else { 0xFFFFFF }))
                        .bg(rgb(fill))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(8.))
                                .h(px(8.))
                                .rounded(px(999.))
                                .border_2()
                                .border_color(ink),
                        ),
                ),
        );
        let mut hits = div().absolute().inset_0().flex();
        for (i, &id) in LEVEL_ID.iter().enumerate().take(n) {
            let f = fire(&self.act, id);
            hits = hits.child(
                div()
                    .id(SharedString::from(format!("oxen-{i}")))
                    .flex_1()
                    .h_full()
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        div()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(6.))
            .pb(px(10.))
            .rounded(px(14.))
            .border_1()
            .border_color(p.line)
            .bg(bg)
            .text_color(p.fg2)
            .text_size(px(13.))
            .child(line.child(hits))
            .child(self.levels[cur].label.clone())
    }
}
