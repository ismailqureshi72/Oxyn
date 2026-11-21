//! Context Visualizer family (variants 2) for GPUI over `oxyn_core::kit::ctxvisfid`: the
//! **Context bar** ([`ContextBarEl`]), **Context breakdown** ([`ContextBreakdownEl`]), **Context
//! stack** ([`ContextStackEl`]) and **Context tide** ([`ContextTideEl`]). `RenderOnce` static
//! snapshots: the host keeps the parts and routes [`Act`] ids back (`compact`, `undo`, `turn`,
//! `kind-<i>`). Chrome is the Thinking Selector's neutral palette; hue marks the kinds only.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::ctxvisfid::{
    CtxPart, ctx_compact, ctx_demo, ctx_load_order, ctx_pct, ctx_used, fill_level, fmt_tokens,
    kind_hex, slab_heights, turns_left,
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{fire, th_pal};

/// A `RRGGBB` string as a colour.
pub(crate) fn hex(h: &str) -> Hsla {
    rgb(u32::from_str_radix(h, 16).unwrap_or(0x8A8A93)).into()
}

const KIND_ID: [&str; 6] = ["kind-0", "kind-1", "kind-2", "kind-3", "kind-4", "kind-5"];

fn amber(theme: &Theme) -> Hsla {
    if th_pal(theme).card.l < 0.5 {
        hex("F5A524")
    } else {
        hex("B45309")
    }
}

/// `context-bar` (Context Visualizer – Context bar): figure, stacked bar and kind legend.
#[derive(IntoElement)]
pub struct ContextBarEl {
    parts: Vec<CtxPart>,
    limit: u32,
    compacted: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextBarEl {
    fn default() -> Self {
        Self {
            parts: ctx_demo(),
            limit: 200_000,
            compacted: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextBarEl {
    /// The parts.
    pub fn parts(mut self, p: Vec<CtxPart>) -> Self {
        self.parts = p;
        self
    }
    /// Shows the compacted window (the button reads Undo).
    pub fn compacted(mut self, c: bool) -> Self {
        self.compacted = c;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `compact` / `undo`, `kind-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ContextBarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let shown = if self.compacted {
            ctx_compact(&self.parts, 180)
        } else {
            self.parts.clone()
        };
        let used = ctx_used(&shown);
        let w = 284.0;
        let l = self.limit.max(1) as f32;
        let mut bar = div().flex().gap(px(4.)).h(px(40.));
        for q in &shown {
            bar = bar.child(
                div()
                    .w(px((q.tokens as f32 / l * w).max(5.)))
                    .h_full()
                    .rounded(px(5.))
                    .bg(hex(kind_hex(q.kind))),
            );
        }
        bar = bar.child(div().flex_1().h_full().rounded(px(5.)).bg(p.sel));
        let mut legend = div().flex().flex_wrap().gap(px(6.)).mt(px(12.));
        for (i, q) in shown.iter().enumerate() {
            let f = fire(&self.act, KIND_ID[i.min(5)]);
            legend = legend.child(
                div()
                    .id(SharedString::from(format!("oxcv-k{i}")))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(26.))
                    .px(px(10.))
                    .rounded(px(13.))
                    .bg(p.sel)
                    .text_size(px(12.))
                    .child(div().size(px(9.)).rounded(px(3.)).bg(hex(kind_hex(q.kind))))
                    .child(q.label.clone())
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let f = fire(&self.act, if self.compacted { "undo" } else { "compact" });
        div()
            .w(px(320.))
            .p(px(18.))
            .rounded(px(18.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_size(px(14.)).text_color(p.fg2).child("Context"))
                    .child(
                        div()
                            .id("oxcv-btn")
                            .h(px(28.))
                            .px(px(14.))
                            .flex()
                            .items_center()
                            .rounded(px(14.))
                            .bg(if self.compacted { p.sel } else { p.fg })
                            .text_color(if self.compacted { p.fg } else { p.card })
                            .text_size(px(12.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if self.compacted { "Undo" } else { "Compact" })
                            .on_click(move |_, w, cx| f(w, cx)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(8.))
                    .my(px(10.))
                    .child(div().text_size(px(34.)).child(fmt_tokens(used)))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(p.mute)
                            .pb(px(6.))
                            .child(format!("of {}", fmt_tokens(self.limit))),
                    ),
            )
            .child(bar)
            .child(legend)
    }
}

/// `context-breakdown` (Context Visualizer – Context breakdown): kinds with their share.
#[derive(IntoElement)]
pub struct ContextBreakdownEl {
    parts: Vec<CtxPart>,
    limit: u32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextBreakdownEl {
    fn default() -> Self {
        Self {
            parts: ctx_demo(),
            limit: 200_000,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextBreakdownEl {
    /// The parts.
    pub fn parts(mut self, p: Vec<CtxPart>) -> Self {
        self.parts = p;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `kind-<i>` (open a kind).
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ContextBreakdownEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let used = ctx_used(&self.parts);
        let pct = ctx_pct(used, self.limit);
        let lv = fill_level(used, self.limit, 92);
        let top = self
            .parts
            .iter()
            .map(|q| q.tokens)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let mut rows = div().flex().flex_col().mt(px(10.));
        let mut order: Vec<usize> = (0..self.parts.len()).collect();
        order.sort_by(|a, b| self.parts[*b].tokens.cmp(&self.parts[*a].tokens));
        for i in order {
            let q = &self.parts[i];
            let f = fire(&self.act, KIND_ID[i.min(5)]);
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("oxcb-{i}")))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(38.))
                    .border_t_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .size(px(10.))
                            .rounded(px(3.))
                            .bg(hex(kind_hex(q.kind))),
                    )
                    .child(div().w(px(64.)).child(q.label.clone()))
                    .child(
                        div().flex_1().h(px(6.)).rounded(px(3.)).bg(p.sel).child(
                            div()
                                .h_full()
                                .w(px(120. * q.tokens as f32 / top))
                                .rounded(px(3.))
                                .bg(hex(kind_hex(q.kind))),
                        ),
                    )
                    .child(
                        div()
                            .w(px(52.))
                            .text_size(px(13.))
                            .child(fmt_tokens(q.tokens)),
                    )
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        div()
            .w(px(360.))
            .p(px(16.))
            .rounded(px(18.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .text_size(px(14.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Context"))
                            .child(div().text_size(px(12.5)).text_color(p.mute).child(format!(
                                "{} of {} tokens",
                                fmt_tokens(used),
                                fmt_tokens(self.limit)
                            ))),
                    )
                    .child(
                        div()
                            .px(px(9.))
                            .py(px(4.))
                            .rounded(px(12.))
                            .bg(p.sel)
                            .text_size(px(12.))
                            .text_color(match lv {
                                "hot" => p.red,
                                "warm" => amber(self.theme),
                                _ => p.fg2,
                            })
                            .child(format!("{pct}%")),
                    ),
            )
            .child(rows)
    }
}

/// `context-stack` (Context Visualizer – Context stack, Oxyn original): slabs and the line.
#[derive(IntoElement)]
pub struct ContextStackEl {
    parts: Vec<CtxPart>,
    limit: u32,
    auto: u32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextStackEl {
    fn default() -> Self {
        Self {
            parts: ctx_demo(),
            limit: 200_000,
            auto: 92,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextStackEl {
    /// The parts.
    pub fn parts(mut self, p: Vec<CtxPart>) -> Self {
        self.parts = p;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `turn`, `compact`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ContextStackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let ps = ctx_load_order(&self.parts);
        let hs = slab_heights(&ps, self.limit, 232., 12.);
        let used = ctx_used(&ps);
        let mut col = div()
            .relative()
            .w(px(112.))
            .h(px(232.))
            .p(px(3.))
            .flex()
            .flex_col()
            .justify_end()
            .gap(px(2.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(px(232. * self.auto as f32 / 100.))
                    .h(px(1.5))
                    .bg(p.red),
            );
        for q in ps.iter().rev() {
            col = col.child(
                div()
                    .h(px(hs[ps.iter().position(|x| x.kind == q.kind).unwrap_or(0)]))
                    .rounded(px(6.))
                    .bg(hex(kind_hex(q.kind))),
            );
        }
        let mut legend = div().flex().flex_col().gap(px(2.)).flex_1();
        for q in ps.iter().rev() {
            legend = legend.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(28.))
                    .child(
                        div()
                            .size(px(10.))
                            .rounded(px(3.))
                            .bg(hex(kind_hex(q.kind))),
                    )
                    .child(div().flex_1().child(q.label.clone()))
                    .child(div().text_color(p.mute).child(fmt_tokens(q.tokens))),
            );
        }
        let ft = fire(&self.act, "turn");
        let fc = fire(&self.act, "compact");
        let btn = |id: &'static str, label: &'static str| {
            div()
                .id(id)
                .h(px(30.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .border_1()
                .border_color(p.line)
                .bg(p.card2)
                .text_size(px(12.5))
                .child(label)
        };
        div()
            .w(px(340.))
            .p(px(16.))
            .rounded(px(18.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .mb(px(12.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Context stack"),
                    )
                    .child(div().text_color(p.mute).child(format!(
                        "{} / {} · {}%",
                        fmt_tokens(used),
                        fmt_tokens(self.limit),
                        ctx_pct(used, self.limit)
                    ))),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(16.))
                    .child(col)
                    .child(legend),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .mt(px(14.))
                    .child(btn("oxcs-turn", "Add a turn").on_click(move |_, w, cx| ft(w, cx)))
                    .child(btn("oxcs-compact", "Compact").on_click(move |_, w, cx| fc(w, cx))),
            )
    }
}

/// `context-tide` (Context Visualizer – Context tide, Oxyn original): glass, water, marks.
#[derive(IntoElement)]
pub struct ContextTideEl {
    used: u32,
    limit: u32,
    auto: u32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextTideEl {
    fn default() -> Self {
        Self {
            used: ctx_used(&ctx_demo()),
            limit: 200_000,
            auto: 92,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextTideEl {
    /// Used tokens.
    pub fn used(mut self, u: u32) -> Self {
        self.used = u;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `turn`, `compact`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ContextTideEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let pct = ctx_pct(self.used, self.limit);
        let water = match fill_level(self.used, self.limit, self.auto) {
            "hot" => p.red,
            "warm" => amber(self.theme),
            _ => p.mute,
        };
        let turns = match turns_left(self.used, self.limit, self.auto, 6_000) {
            None | Some(0) => "Compacts on the next turn".to_string(),
            Some(n) => format!("≈ {n} turns before it compacts"),
        };
        let ft = fire(&self.act, "turn");
        let fc = fire(&self.act, "compact");
        div()
            .w(px(340.))
            .p(px(18.))
            .flex()
            .gap(px(20.))
            .rounded(px(18.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .child(
                div()
                    .relative()
                    .w(px(92.))
                    .h(px(220.))
                    .rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.inset)
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .h(px(220. * pct as f32 / 100.))
                            .bg(water.opacity(0.7)),
                    )
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .bottom(px(220. * self.auto as f32 / 100.))
                            .h(px(1.5))
                            .bg(p.red),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_end()
                    .gap(px(4.))
                    .child(div().text_size(px(13.)).text_color(p.mute).child("Context"))
                    .child(div().text_size(px(40.)).child(format!("{pct}%")))
                    .child(div().text_size(px(12.5)).text_color(p.fg2).child(format!(
                        "{} of {} tokens",
                        fmt_tokens(self.used),
                        fmt_tokens(self.limit)
                    )))
                    .child(div().text_size(px(12.)).text_color(p.mute).child(turns))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .mt(px(8.))
                            .child(
                                div()
                                    .id("oxct-turn")
                                    .px(px(12.))
                                    .h(px(30.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(9.))
                                    .border_1()
                                    .border_color(p.line)
                                    .child("Send a turn")
                                    .on_click(move |_, w, cx| ft(w, cx)),
                            )
                            .child(
                                div()
                                    .id("oxct-compact")
                                    .px(px(12.))
                                    .h(px(30.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(9.))
                                    .border_1()
                                    .border_color(p.line)
                                    .child("Compact")
                                    .on_click(move |_, w, cx| fc(w, cx)),
                            ),
                    ),
            )
    }
}
