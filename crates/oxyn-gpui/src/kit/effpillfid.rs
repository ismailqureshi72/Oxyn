//! Thinking Selector family, **Effort pill** for GPUI ([`EffortPillEl`]) over
//! `oxyn_core::kit::thinkfid`. `RenderOnce`: the host keeps the level and whether the popover
//! is open and routes [`Act`] ids back (`toggle`, `level-<i>`, `reset`, `model`). A static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::kit::thinkfid::{EffortLevel, effort_scale, level_frac};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

/// `effort-pill` (Thinking Selector – Effort pill).
#[derive(IntoElement)]
pub struct EffortPillEl {
    levels: Vec<EffortLevel>,
    value: Option<usize>,
    model: String,
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for EffortPillEl {
    fn default() -> Self {
        Self {
            levels: effort_scale(),
            value: Some(2),
            model: "Atlas 6".into(),
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl EffortPillEl {
    /// Your own scale.
    pub fn levels(mut self, l: Vec<EffortLevel>) -> Self {
        self.levels = l;
        self
    }
    /// Level index (`None` = not chosen yet).
    pub fn value(mut self, v: Option<usize>) -> Self {
        self.value = v;
        self
    }
    /// The model line.
    pub fn model(mut self, m: &str) -> Self {
        self.model = m.into();
        self
    }
    /// Popover open.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `toggle`, `level-<i>`, `reset`, `model`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for EffortPillEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let n = self.levels.len().max(1);
        let shown = self.value.unwrap_or(n / 2).min(n - 1);
        let w = 340.0_f32;
        let x = 15.0 + (w - 30.0) * level_frac(shown, n);
        let mut track = div()
            .relative()
            .w(px(w))
            .h(px(36.))
            .rounded(px(999.))
            .bg(p.inset)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(x + 15.))
                    .rounded(px(999.))
                    .bg(p.fg),
            );
        for i in 0..n {
            let dx = 15.0 + (w - 30.0) * level_frac(i, n);
            track = track.child(
                div()
                    .absolute()
                    .left(px(dx - 2.5))
                    .top(px(15.5))
                    .w(px(5.))
                    .h(px(5.))
                    .rounded(px(999.))
                    .bg(if i <= shown { p.card } else { p.mute }),
            );
        }
        track = track.child(
            div()
                .absolute()
                .left(px(x - 19.))
                .top(px(-1.))
                .w(px(38.))
                .h(px(38.))
                .rounded(px(999.))
                .border_2()
                .border_color(p.fg)
                .bg(p.knob),
        );
        let mut hits = div().absolute().inset_0().flex();
        for (i, &id) in LEVEL_ID.iter().enumerate().take(n) {
            let f = fire(&self.act, id);
            hits = hits.child(
                div()
                    .id(SharedString::from(format!("oxeq-{i}")))
                    .flex_1()
                    .h_full()
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let toggle = fire(&self.act, "toggle");
        let model = fire(&self.act, "model");
        let reset = fire(&self.act, "reset");
        let pill = self.value.map_or("Select effort".to_string(), |i| {
            self.levels[i.min(n - 1)].label.clone()
        });
        let pop = div()
            .relative()
            .w(px(380.))
            .p(px(18.))
            .flex()
            .flex_col()
            .items_center()
            .rounded(px(20.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .child(
                div()
                    .id("oxeq-reset")
                    .absolute()
                    .top(px(12.))
                    .right(px(12.))
                    .text_color(p.fg2)
                    .on_click(move |_, w, cx| reset(w, cx))
                    .child("↺"),
            )
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.levels[shown].label.clone()),
            )
            .child(
                div()
                    .id("oxeq-model")
                    .mb(px(16.))
                    .text_color(p.mute)
                    .on_click(move |_, w, cx| model(w, cx))
                    .child(format!("{} ›", self.model)),
            )
            .child(track.child(hits));
        div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(10.))
            .text_color(p.fg)
            .child(
                div()
                    .id("oxeq-pill")
                    .h(px(34.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .rounded(px(999.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card2)
                    .text_color(if self.value.is_some() { p.fg } else { p.mute })
                    .on_click(move |_, w, cx| toggle(w, cx))
                    .child(format!("{pill} ⌄")),
            )
            .when(self.open, |d| d.child(pop))
    }
}
