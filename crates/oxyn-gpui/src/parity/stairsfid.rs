//! Thinking Selector family, **Effort stairs** for GPUI ([`EffortStairsEl`]), an Oxyn original over
//! `oxyn_core::parity::{thinkfid, stairsfid}`. `RenderOnce`: the host keeps the level and routes
//! [`Act`] ids back (`level-<i>`). A static snapshot.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::parity::{
    stairsfid::{stair_height, stairs_line},
    thinkfid::{EffortLevel, effort_scale},
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{LEVEL_ID, fire, th_pal};

/// `effort-stairs` (Thinking Selector – Effort stairs).
#[derive(IntoElement)]
pub struct EffortStairsEl {
    levels: Vec<EffortLevel>,
    value: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for EffortStairsEl {
    fn default() -> Self {
        Self {
            levels: effort_scale(),
            value: 2,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl EffortStairsEl {
    /// Level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
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

impl RenderOnce for EffortStairsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let mut bars = div().flex().items_end().gap(px(6.)).h(px(64.));
        for (i, &id) in LEVEL_ID.iter().enumerate().take(n) {
            let f = fire(&self.act, id);
            let on = i <= cur;
            bars = bars.child(
                div()
                    .id(SharedString::from(format!("oxes-{i}")))
                    .flex_1()
                    .h(px(64. * stair_height(i, n)))
                    .rounded(px(4.))
                    .bg(if on && i == n - 1 {
                        p.red
                    } else if on {
                        p.fg
                    } else {
                        p.sel
                    })
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        div()
            .w(px(300.))
            .p(px(16.))
            .rounded(px(16.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if cur == n - 1 { p.red } else { p.fg })
                    .child(self.levels[cur].label.clone()),
            )
            .child(
                div()
                    .mb(px(12.))
                    .text_size(px(12.5))
                    .text_color(p.mute)
                    .child(stairs_line(cur, n)),
            )
            .child(bars)
    }
}
