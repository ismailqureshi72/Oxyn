//! Thinking Selector family (variants 1) for GPUI: the shared palette of every variant and the
//! **Ruler card** ([`EffortRulerEl`]) over `oxyn_core::kit::thinkfid`. `RenderOnce`: the host
//! keeps the level and routes [`Act`] ids back (`level-0` … `level-7`, `reset`). A static
//! snapshot: no knob glide.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::thinkfid::{EffortLevel, effort_scale, level_frac, tick_tint};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

/// Ids a level press reports, by index.
pub(crate) const LEVEL_ID: [&str; 8] = [
    "level-0", "level-1", "level-2", "level-3", "level-4", "level-5", "level-6", "level-7",
];

/// The family's palette: neutral greys and one red for the effort.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct ThPal {
    pub card: Hsla,
    pub card2: Hsla,
    pub inset: Hsla,
    pub line: Hsla,
    pub fg: Hsla,
    pub fg2: Hsla,
    pub mute: Hsla,
    pub tick: Hsla,
    pub sel: Hsla,
    pub red: Hsla,
    pub knob: Hsla,
    pub off: Hsla,
}

pub(crate) fn th_pal(t: &Theme) -> ThPal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        ThPal {
            card: h(0x1C1C1E),
            card2: h(0x17171A),
            inset: h(0x141316),
            line: h(0x2A2A2E),
            fg: h(0xF2F2F3),
            fg2: h(0xD4D4D8),
            mute: h(0x8A8A93),
            tick: h(0x3E3E44),
            sel: h(0x2A2A2E),
            red: h(0xF0533F),
            knob: h(0xF2F2F3),
            off: h(0x3A3A3F),
        }
    } else {
        ThPal {
            card: h(0xFFFFFF),
            card2: h(0xFAFAFA),
            inset: h(0xF4F4F5),
            line: h(0xE6E6E8),
            fg: h(0x111113),
            fg2: h(0x2A2A2E),
            mute: h(0x6E6E76),
            tick: h(0xCFCFD4),
            sel: h(0xF0F0F1),
            red: h(0xE0442F),
            knob: h(0xFFFFFF),
            off: h(0xDADADD),
        }
    }
}

/// `a` mixed toward `b` by `t` (0.0 = a, 1.0 = b), in HSLA.
pub(crate) fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    let rgb_a = a.to_rgb();
    let rgb_b = b.to_rgb();
    gpui::Rgba {
        r: rgb_a.r + (rgb_b.r - rgb_a.r) * t,
        g: rgb_a.g + (rgb_b.g - rgb_a.g) * t,
        b: rgb_a.b + (rgb_b.b - rgb_a.b) * t,
        a: rgb_a.a + (rgb_b.a - rgb_a.a) * t,
    }
    .into()
}

/// A click handler that reports `id` to the host.
pub(crate) fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

/// `effort-ruler` (Thinking Selector – Ruler card): the host passes the level.
#[derive(IntoElement)]
pub struct EffortRulerEl {
    levels: Vec<EffortLevel>,
    value: usize,
    reset: usize,
    ticks: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for EffortRulerEl {
    fn default() -> Self {
        Self {
            levels: effort_scale(),
            value: 4,
            reset: 2,
            ticks: 44,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl EffortRulerEl {
    /// Your own scale.
    pub fn levels(mut self, l: Vec<EffortLevel>) -> Self {
        self.levels = l;
        self
    }
    /// The level index.
    pub fn value(mut self, v: usize) -> Self {
        self.value = v;
        self
    }
    /// Where reset returns.
    pub fn reset(mut self, v: usize) -> Self {
        self.reset = v;
        self
    }
    /// Tick count.
    pub fn ticks(mut self, n: usize) -> Self {
        self.ticks = n.max(2);
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Level presses (`level-<i>`) and `reset`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for EffortRulerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let n = self.levels.len().max(1);
        let cur = self.value.min(n - 1);
        let knob = level_frac(cur, n);
        let lv = self
            .levels
            .get(cur)
            .cloned()
            .unwrap_or_else(|| effort_scale()[0].clone());
        let top = cur == n - 1;
        let w = 440.0_f32;
        let inner = w - 36.0 - 44.0;
        let mut ruler = div().relative().h(px(60.)).w(px(inner));
        for t in 0..self.ticks {
            let tint = tick_tint(t, self.ticks, knob, 0.55);
            let x = t as f32 / (self.ticks - 1) as f32 * inner;
            ruler = ruler.child(
                div()
                    .absolute()
                    .left(px(x - 1.))
                    .top(px(21.))
                    .w(px(2.))
                    .h(px(18.))
                    .rounded(px(1.))
                    .bg(mix(p.tick, p.red, tint)),
            );
        }
        ruler = ruler.child(
            div()
                .absolute()
                .left(px(knob * inner - 4.))
                .top(px(15.))
                .w(px(8.))
                .h(px(30.))
                .rounded(px(999.))
                .bg(p.red),
        );
        // Each level is a press target over its stretch of the ruler.
        let mut hits = div().absolute().inset_0().flex();
        for (i, &id) in LEVEL_ID.iter().enumerate().take(n) {
            let f = fire(&self.act, id);
            hits = hits.child(
                div()
                    .id(SharedString::from(format!("oxer-{i}")))
                    .flex_1()
                    .h_full()
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let reset = fire(&self.act, "reset");
        div()
            .relative()
            .w(px(w))
            .p(px(18.))
            .rounded(px(18.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .child(
                div()
                    .id("oxer-reset")
                    .absolute()
                    .top(px(14.))
                    .right(px(14.))
                    .w(px(28.))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .text_color(if cur == self.reset { p.mute } else { p.fg2 })
                    .text_size(px(15.))
                    .on_click(move |_, w, cx| reset(w, cx))
                    .child("↺"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .pb(px(14.))
                    .child(
                        div()
                            .text_size(px(19.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if top { p.red } else { p.fg })
                            .child(lv.label.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .text_color(p.mute)
                            .child(lv.detail.clone()),
                    ),
            )
            .child(
                div()
                    .px(px(22.))
                    .rounded(px(14.))
                    .bg(p.inset)
                    .border_1()
                    .border_color(p.line)
                    .child(ruler.child(hits)),
            )
    }
}
