//! Thinking Selector family, **Reasoning panel** for GPUI ([`ReasoningPanelEl`]) over
//! `oxyn_core::kit::{thinkfid, reasonfid}`. `RenderOnce`: the host keeps the level, Fast mode and
//! the favourite and routes [`Act`] ids back (`level-<i>`, `fast`, `favorite`). A static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::kit::{
    reasonfid::{PanelModel, context_label, panel_demo_model},
    thinkfid::{EffortLevel, effort_scale, level_frac, pixel_fill},
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, mix, th_pal};

/// `reasoning-panel` (Thinking Selector – Reasoning panel).
#[derive(IntoElement)]
pub struct ReasoningPanelEl {
    model: PanelModel,
    levels: Vec<EffortLevel>,
    value: usize,
    fast: bool,
    favorite: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ReasoningPanelEl {
    fn default() -> Self {
        Self {
            model: panel_demo_model(),
            levels: effort_scale(),
            value: 2,
            fast: false,
            favorite: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ReasoningPanelEl {
    /// The model shown.
    pub fn model(mut self, m: PanelModel) -> Self {
        self.model = m;
        self
    }
    /// Your own scale.
    pub fn levels(mut self, l: Vec<EffortLevel>) -> Self {
        self.levels = l;
        self
    }
    /// Level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Fast mode.
    pub fn fast(mut self, on: bool) -> Self {
        self.fast = on;
        self
    }
    /// Favourite.
    pub fn favorite(mut self, on: bool) -> Self {
        self.favorite = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `level-<i>`, `fast`, `favorite`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn kbd(p: super::rulerfid::ThPal, k: &'static str) -> impl IntoElement {
    div()
        .px(px(4.))
        .h(px(17.))
        .flex()
        .items_center()
        .rounded(px(4.))
        .border_1()
        .border_color(p.line)
        .bg(p.sel)
        .text_size(px(10.5))
        .text_color(p.fg2)
        .child(k)
}

impl RenderOnce for ReasoningPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let top = cur == n - 1;
        let w = 314.0_f32;
        let usable = w - 22.0;
        let x = 11.0 + usable * level_frac(cur, n);
        let mut track = div()
            .relative()
            .w(px(w))
            .h(px(22.))
            .rounded(px(7.))
            .bg(p.inset)
            .border_1()
            .border_color(p.line)
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(x + 9.))
                    .bg(if top { mix(p.inset, p.red, 0.3) } else { p.sel }),
            );
        if top {
            let dots = pixel_fill(40, 4, 11);
            let cw = (x - 6.) / 40.0;
            for (i, on) in dots.iter().enumerate() {
                let (col, row) = (i % 40, i / 40);
                track = track.child(
                    div()
                        .absolute()
                        .left(px(4. + col as f32 * cw))
                        .top(px(3. + row as f32 * 4.))
                        .w(px((cw - 1.).max(1.)))
                        .h(px(3.))
                        .bg(if *on { p.red } else { mix(p.inset, p.red, 0.4) }),
                );
            }
        }
        for i in 1..n.saturating_sub(1) {
            if i > cur {
                track = track.child(
                    div()
                        .absolute()
                        .left(px(11. + usable * level_frac(i, n)))
                        .top(px(6.))
                        .w(px(1.5))
                        .h(px(8.))
                        .bg(p.line),
                );
            }
        }
        track = track.child(
            div()
                .absolute()
                .left(px(x - 9.))
                .top(px(1.))
                .w(px(18.))
                .h(px(18.))
                .rounded(px(5.))
                .bg(p.knob),
        );
        let mut hits = div().absolute().inset_0().flex();
        for (i, &id) in LEVEL_ID.iter().enumerate().take(n) {
            let f = fire(&self.act, id);
            hits = hits.child(
                div()
                    .id(SharedString::from(format!("oxep-{i}")))
                    .flex_1()
                    .h_full()
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let lv = self.levels[cur].label.clone();
        let fast = fire(&self.act, "fast");
        let fav = fire(&self.act, "favorite");
        let row = || {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(4.))
                .h(px(24.))
        };
        div()
            .w(px(340.))
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .rounded(px(14.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .text_size(px(14.))
            .child(
                div()
                    .px(px(4.))
                    .text_size(px(15.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.model.name.clone()),
            )
            .child(
                row()
                    .child(div().text_color(p.mute).child("Context length"))
                    .child(context_label(self.model.context)),
            )
            .child(div().h(px(1.)).bg(p.line))
            .child(
                row()
                    .child(
                        div()
                            .flex()
                            .gap(px(7.))
                            .text_color(p.mute)
                            .child("Reasoning")
                            .child(kbd(p, "R")),
                    )
                    .child(div().text_color(if top { p.red } else { p.fg }).child(lv)),
            )
            .child(div().px(px(4.)).child(track.child(hits)))
            .when(self.model.has_fast, |d| {
                d.child(
                    row()
                        .child(
                            div()
                                .flex()
                                .gap(px(7.))
                                .text_color(p.mute)
                                .child("Fast mode")
                                .child(kbd(p, "F")),
                        )
                        .child(
                            div()
                                .id("oxep-fast")
                                .w(px(36.))
                                .h(px(21.))
                                .rounded(px(999.))
                                .bg(if self.fast { p.fg } else { p.off })
                                .flex()
                                .items_center()
                                .px(px(2.5))
                                .when(self.fast, |d| d.justify_end())
                                .on_click(move |_, w, cx| fast(w, cx))
                                .child(
                                    div()
                                        .w(px(16.))
                                        .h(px(16.))
                                        .rounded(px(999.))
                                        .bg(if self.fast { p.card } else { p.knob }),
                                ),
                        ),
                )
            })
            .child(
                div()
                    .id("oxep-fav")
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(7.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.sel)
                    .font_weight(FontWeight::SEMIBOLD)
                    .on_click(move |_, w, cx| fav(w, cx))
                    .child(if self.favorite {
                        "★ Favorited"
                    } else {
                        "☆ Favorite"
                    }),
            )
    }
}
