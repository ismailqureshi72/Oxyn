//! Thinking Selector family, **Nozzle line** for GPUI ([`NozzleSliderEl`]), an Oxyn original over
//! `oxyn_core::parity::{thinkfid, nozzlefid}`. `RenderOnce`: the host keeps the level and routes
//! [`Act`] ids back (`level-<i>`). Strokes are drawn upright (GPUI has no rotate on divs); the
//! wobble shows in their heights. A static snapshot.

use gpui::{
    App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::*, px,
};
use oxyn_core::parity::{
    nozzlefid::{is_stop, nozzle_ticks},
    thinkfid::{EffortLevel, effort_scale, level_frac},
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

/// `nozzle-slider` (Thinking Selector – Nozzle line).
#[derive(IntoElement)]
pub struct NozzleSliderEl {
    levels: Vec<EffortLevel>,
    value: usize,
    strokes: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for NozzleSliderEl {
    fn default() -> Self {
        Self {
            levels: effort_scale(),
            value: 2,
            strokes: 41,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl NozzleSliderEl {
    /// Level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Stroke count.
    pub fn strokes(mut self, n: usize) -> Self {
        self.strokes = n.max(2);
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
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let w = 392.0_f32;
        let mut line = div().relative().w(px(w + 28.)).h(px(48.));
        for (t, k) in nozzle_ticks(self.strokes, 7).iter().enumerate() {
            let s = is_stop(t, n, self.strokes);
            let h = (if s { 21. } else { 18. }) * k.height;
            line = line.child(
                div()
                    .absolute()
                    .left(px(14. + w * t as f32 / (self.strokes - 1) as f32 - 1.25))
                    .top(px(24. - h / 2.))
                    .w(px(if s { 3. } else { 2.5 }))
                    .h(px(h))
                    .rounded(px(2.))
                    .bg(p.red),
            );
        }
        line = line.child(
            div()
                .absolute()
                .left(px(14. + w * level_frac(cur, n) - 13.))
                .top(px(11.))
                .w(px(26.))
                .h(px(26.))
                .rounded(px(999.))
                .border_2()
                .border_color(p.fg),
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
            .text_color(p.mute)
            .text_size(px(13.))
            .child(line.child(hits))
            .child(self.levels[cur].label.clone())
    }
}
