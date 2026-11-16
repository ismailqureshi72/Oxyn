//! affectUI fidelity pass, family B part 8 (plan #32) for GPUI: the exact **Decision Tree**
//! ([`DecisionTreeEl`]) over `oxyn_core::parity::dtreefid`. The element is `RenderOnce`: the host
//! keeps the queue, the current decision and its [`DtState`] and routes [`Act`] ids back
//! (`b-<n>` for a branch or, once a branch is picked, a follow-up option; `other`, `next`, `prev`).
//! A static snapshot: no connector lines, no grow-in, no notes.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::dtreefid::{DtDecision, DtOption, DtState, dt_demo, dt_waiting};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const B: [&str; 4] = ["b-0", "b-1", "b-2", "b-3"];

/// `decision-tree` (affectUI Decision Tree): the host passes the queue and the state.
#[derive(IntoElement)]
pub struct DecisionTreeEl {
    queue: Vec<DtDecision>,
    cur: usize,
    state: DtState,
    title: String,
    allow_other: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for DecisionTreeEl {
    fn default() -> Self {
        Self {
            queue: dt_demo(),
            cur: 0,
            state: DtState::default(),
            title: "Decisions".into(),
            allow_other: true,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl DecisionTreeEl {
    /// The decisions waiting and the one shown.
    pub fn queue(mut self, q: Vec<DtDecision>, cur: usize) -> Self {
        self.queue = q;
        self.cur = cur;
        self
    }
    /// Where the shown decision stands.
    pub fn state(mut self, s: DtState) -> Self {
        self.state = s;
        self
    }
    /// Offer "Other…".
    pub fn allow_other(mut self, on: bool) -> Self {
        self.allow_other = on;
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Branches, paging and Other….
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

fn effects(o: &DtOption) -> String {
    o.effects
        .iter()
        .map(|l| l.iter().map(|s| s.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join(" · ")
}

impl RenderOnce for DecisionTreeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, chip, blue, band, red_bg, red, amb_bg, amb) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x26262B),
                h(0x4C8DFF),
                h(0x111113),
                h(0x3A1C1E),
                h(0xF87171),
                h(0x3A2E12),
                h(0xFBBF24),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6E6),
                h(0x111827),
                h(0x8A8A93),
                h(0xF1F1F3),
                h(0x2563EB),
                h(0xFAFAFA),
                h(0xFEE2E2),
                h(0xDC2626),
                h(0xFEF3C7),
                h(0xB45309),
            )
        };
        let n = self.queue.len();
        let mut bar = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .when(self.classic, |d| d.px(px(16.)).py(px(12.)).bg(band))
            .child(div().text_color(blue).text_size(px(15.)).child("⑂"))
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(mute)
                    .child(dt_waiting(n)),
            )
            .child(div().flex_1());
        for (id, g) in [("prev", "‹"), ("next", "›")] {
            let f = fire(&self.act, id);
            bar = bar.child(
                div()
                    .id(SharedString::from(format!("oxdt-{id}")))
                    .px(px(4.))
                    .text_color(mute)
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(g),
            );
        }
        bar = bar.child(
            div()
                .text_size(px(13.))
                .text_color(mute)
                .child(format!("{} of {n}", (self.cur + 1).min(n))),
        );
        let root = div()
            .w(px(820.))
            .flex()
            .flex_col()
            .gap(px(22.))
            .text_color(fg)
            .when(self.classic, |d| {
                d.border_1().border_color(bd).rounded(px(16.)).bg(band)
            })
            .child(bar);
        let Some(d) = self.queue.get(self.cur).cloned() else {
            return root.child(
                div()
                    .p(px(40.))
                    .text_color(mute)
                    .child("All decided – the agents are on their way."),
            );
        };
        let mut q = div()
            .w(px(205.))
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .border_1()
            .border_color(bd)
            .rounded(px(12.))
            .bg(card);
        if let Some(tag) = d.risk.tag() {
            let (bg, tx) = if d.risk.id() == "high" {
                (red_bg, red)
            } else {
                (amb_bg, amb)
            };
            q = q.child(
                div()
                    .px(px(8.))
                    .py(px(3.))
                    .rounded(px(6.))
                    .bg(bg)
                    .text_color(tx)
                    .text_size(px(11.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(tag),
            );
        }
        q = q
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(d.agent.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(mute)
                            .child(d.model.clone()),
                    ),
            )
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(d.question.clone()),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(mute)
                    .child(d.context.clone()),
            );
        let s = &self.state;
        let mut col = div().flex_1().flex().flex_col().gap(px(18.));
        for (i, o) in d.options.iter().enumerate().take(4) {
            if s.branch.is_some_and(|b| b != i) {
                continue;
            }
            let picked = s.branch == Some(i);
            let f = fire(&self.act, B[i]);
            let mut btn = div()
                .id(SharedString::from(format!("oxdt-b{i}")))
                .w(px(183.))
                .px(px(12.))
                .py(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .border_1()
                .border_color(if picked || o.pick { blue } else { bd })
                .rounded(px(10.))
                .bg(card)
                .child(
                    div()
                        .w(px(18.))
                        .h(px(18.))
                        .rounded(px(5.))
                        .bg(chip)
                        .text_size(px(11.))
                        .text_color(mute)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(format!("{}", i + 1)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(14.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(o.label.clone()),
                        )
                        .when(o.pick, |d| {
                            d.child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(blue)
                                    .child("Agent's pick"),
                            )
                        }),
                );
            if !picked {
                btn = btn.on_click(move |_, w, cx| f(w, cx));
            }
            let mut cons = div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(div().text_size(px(12.5)).text_color(mute).child(effects(o)));
            if let Some(fu) = &o.next {
                let mut fcol = div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .opacity(if picked { 1.0 } else { 0.42 })
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(5.))
                            .border_1()
                            .border_color(bd)
                            .rounded(px(8.))
                            .text_size(px(13.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fu.question.clone()),
                    );
                for (j, fo) in fu.options.iter().enumerate().take(4) {
                    let ff = fire(&self.act, B[j]);
                    let chosen = picked && s.then == Some(j);
                    let mut fb = div()
                        .id(SharedString::from(format!("oxdt-f{i}-{j}")))
                        .w(px(283.))
                        .px(px(12.))
                        .py(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .border_1()
                        .border_color(if chosen { blue } else { bd })
                        .rounded(px(10.))
                        .bg(card)
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(13.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(fo.label.clone()),
                                )
                                .when(fo.pick, |d| {
                                    d.child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(blue)
                                            .child("Agent's pick"),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(mute)
                                .child(effects(fo)),
                        );
                    if picked && s.then.is_none() {
                        fb = fb.on_click(move |_, w, cx| ff(w, cx));
                    }
                    fcol = fcol.child(fb);
                }
                cons = cons.child(fcol);
            }
            if picked && s.done(&d) {
                cons = cons.child(div().text_size(px(13.)).child("✓ Approved · running"));
            }
            col = col.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(36.))
                    .child(btn)
                    .child(cons),
            );
        }
        if self.allow_other && s.branch.is_none() {
            let f = fire(&self.act, "other");
            col = col.child(
                div()
                    .id("oxdt-other")
                    .w(px(120.))
                    .px(px(12.))
                    .py(px(8.))
                    .border_1()
                    .border_color(bd)
                    .rounded(px(10.))
                    .bg(card)
                    .text_size(px(14.))
                    .text_color(mute)
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(format!("{}  Other…", d.options.len() + 1)),
            );
        }
        root.child(
            div()
                .flex()
                .items_start()
                .gap(px(40.))
                .when(self.classic, |d| {
                    d.mx(px(6.)).p(px(20.)).bg(card).rounded(px(12.))
                })
                .child(q)
                .child(col),
        )
    }
}
