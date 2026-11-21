//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32) for GPUI: the exact
//! **Overnight recap** ([`OvernightRecapEl`]) over `oxyn_core::kit::recapfid`. The element is
//! `RenderOnce`: the host keeps each card's state and routes [`Act`] ids back (`act-<n>`,
//! `undo-<n>`, `review-<n>` for the n-th event). A static snapshot: the chart is drawn as thin
//! columns, no replay, no crosshair.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::recapfid::{
    RECAP_SINCE, RECAP_SPAN, RecapCardState, RecapEvent, RecapKind, recap_counts, recap_curve,
    recap_demo, recap_peak, recap_span, recap_time,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 804.0;
const ACT: [&str; 8] = [
    "act-0", "act-1", "act-2", "act-3", "act-4", "act-5", "act-6", "act-7",
];
const UNDO: [&str; 8] = [
    "undo-0", "undo-1", "undo-2", "undo-3", "undo-4", "undo-5", "undo-6", "undo-7",
];
const REVIEW: [&str; 8] = [
    "review-0", "review-1", "review-2", "review-3", "review-4", "review-5", "review-6", "review-7",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    bd: Hsla,
    fg: Hsla,
    fg2: Hsla,
    mute: Hsla,
    faint: Hsla,
    lane: Hsla,
    chip: Hsla,
    line: Hsla,
    dark: Hsla,
    amber: Hsla,
    red: Hsla,
    green: Hsla,
    green_bg: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x141416),
            bd: h(0x26262A),
            fg: h(0xF4F4F5),
            fg2: h(0xD4D4D8),
            mute: h(0xA1A1AA),
            faint: h(0x6B6B73),
            lane: h(0x1A1A1D),
            chip: h(0x232327),
            line: h(0xA1A1AA),
            dark: h(0xD4D4D8),
            amber: h(0xF5A524),
            red: h(0xF87171),
            green: h(0x4ADE80),
            green_bg: h(0x173022),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            bd: h(0xECECEF),
            fg: h(0x18181B),
            fg2: h(0x3F3F46),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            lane: h(0xF4F4F5),
            chip: h(0xF1F1F3),
            line: h(0x52525B),
            dark: h(0x3F3F46),
            amber: h(0xF59E0B),
            red: h(0xDC2626),
            green: h(0x16A34A),
            green_bg: h(0xE6F6EE),
        }
    }
}

/// `agent-recap` (affectUI Agent Recap): the host passes the events and each card's state.
#[derive(IntoElement)]
pub struct OvernightRecapEl {
    events: Vec<RecapEvent>,
    states: Vec<(String, RecapCardState)>,
    since: u32,
    span: u32,
    title: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for OvernightRecapEl {
    fn default() -> Self {
        Self {
            events: recap_demo(),
            states: vec![],
            since: RECAP_SINCE,
            span: RECAP_SPAN,
            title: "While you were away".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl OvernightRecapEl {
    /// Your own night: events, when the person left (minutes after midnight) and for how long.
    pub fn events(mut self, e: Vec<RecapEvent>, since: u32, span: u32) -> Self {
        self.events = e;
        self.since = since;
        self.span = span;
        self
    }
    /// Each card's state (ids not listed are open).
    pub fn states(mut self, s: Vec<(String, RecapCardState)>) -> Self {
        self.states = s;
        self
    }
    /// Heading.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.into();
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
    /// Acts.
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

impl RenderOnce for OvernightRecapEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let st = |id: &str| {
            self.states
                .iter()
                .find(|s| s.0 == id)
                .map_or(RecapCardState::Open, |s| s.1)
        };
        let (d, n, f) = recap_counts(&self.events, &st);
        let sum = |num: u32, word: &str| {
            div()
                .flex()
                .gap(px(4.))
                .child(
                    div()
                        .text_color(p.fg)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(num.to_string()),
                )
                .child(div().text_color(p.mute).child(word.to_string()))
        };
        let head = div()
            .flex()
            .items_start()
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .text_size(px(12.))
                            .text_color(p.mute)
                            .child(format!("Since {}", recap_time(self.since, 0)))
                            .child(
                                div()
                                    .px(px(6.))
                                    .rounded(px(5.))
                                    .bg(p.chip)
                                    .text_color(p.fg2)
                                    .child(recap_span(self.span)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .text_size(px(12.))
                    .child(sum(d, "done"))
                    .child(sum(n, if n == 1 { "needs you" } else { "need you" }))
                    .child(sum(f, "failed")),
            );
        // Chart as thin columns.
        let ch = 96.0;
        let curve = recap_curve(&self.events, self.span, 160);
        let mut plot = div().relative().h(px(ch)).flex().items_end();
        for (_, y) in curve.iter() {
            plot = plot.child(
                div()
                    .flex_1()
                    .h(px((y * ch * 0.86).max(1.0)))
                    .bg(alpha(p.line, 0.16))
                    .border_t_2()
                    .border_color(p.line),
            );
        }
        for e in self.events.iter() {
            let x = e.at as f32 / self.span.max(1) as f32;
            let y = recap_peak(&self.events, e, self.span) * 0.86;
            let col = match (e.kind, st(&e.id)) {
                (RecapKind::Done, RecapCardState::Undone) => p.faint,
                (_, RecapCardState::Acted) | (RecapKind::Done, _) => p.dark,
                (RecapKind::Needs, _) => p.amber,
                _ => p.red,
            };
            plot = plot.child(
                div()
                    .absolute()
                    .left(gpui::relative(x))
                    .top(px(ch - y * ch - 3.0))
                    .size(px(6.))
                    .rounded_full()
                    .bg(col),
            );
        }
        let chart = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .when(self.classic, |d| {
                d.p(px(12.)).rounded(px(12.)).border_1().border_color(p.bd)
            })
            .child(plot)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(11.))
                    .text_color(p.faint)
                    .child(recap_time(self.since, 0))
                    .child(recap_time(self.since, self.span)),
            );
        let mut board = div().flex().gap(px(8.));
        for k in [RecapKind::Needs, RecapKind::Failed, RecapKind::Done] {
            let cards: Vec<(usize, &RecapEvent)> = self
                .events
                .iter()
                .enumerate()
                .filter(|(_, e)| e.kind == k)
                .collect();
            let open_n = cards
                .iter()
                .filter(|(_, e)| match (k, st(&e.id)) {
                    (RecapKind::Done, RecapCardState::Undone) => false,
                    (RecapKind::Done, _) => true,
                    (_, s) => s != RecapCardState::Acted,
                })
                .count();
            let mut lane = div()
                .flex_1()
                .min_h(px(30. + 3. * 94.))
                .p(px(4.))
                .rounded(px(10.))
                .when(!self.classic, |d| d.bg(p.lane))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .h(px(30.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .text_size(px(11.5))
                        .text_color(p.mute)
                        .child(div().flex_1().child(k.lane()))
                        .child(
                            div()
                                .px(px(5.))
                                .rounded(px(5.))
                                .bg(p.chip)
                                .text_color(p.fg2)
                                .child(open_n.to_string()),
                        ),
                );
            for (i, e) in cards.into_iter().take(3) {
                let s = st(&e.id);
                let i = i.min(7);
                let quiet = matches!(s, RecapCardState::Acted | RecapCardState::Undone);
                let btn = |txt: String, id: &'static str, solid: bool| {
                    div()
                        .id(SharedString::from(format!("oxrc-{id}")))
                        .h(px(24.))
                        .px(px(8.))
                        .rounded(px(6.))
                        .when(solid, |d| {
                            d.border_1()
                                .border_color(if self.classic { p.fg } else { p.bd })
                                .bg(if self.classic { p.fg } else { p.card })
                                .text_color(if self.classic { p.card } else { p.fg })
                        })
                        .when(!solid, |d| d.text_color(p.mute))
                        .text_size(px(12.))
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .on_click({
                            let f = fire(&self.act, id);
                            move |_, w, cx| f(w, cx)
                        })
                        .child(txt)
                };
                let tag = |txt: String, col: Hsla, bg: Hsla| {
                    div()
                        .px(px(6.))
                        .rounded(px(5.))
                        .bg(bg)
                        .text_color(col)
                        .text_size(px(11.))
                        .child(txt)
                };
                let mut foot = div().flex().items_center().gap(px(4.));
                foot = match (e.kind, s) {
                    (RecapKind::Done, RecapCardState::Undone) => foot
                        .child(tag("Undone".into(), p.mute, p.chip))
                        .child(div().flex_1())
                        .child(btn("Redo".into(), UNDO[i], false)),
                    (RecapKind::Done, _) => foot
                        .child(tag("Done".into(), p.green, p.green_bg))
                        .child(div().flex_1())
                        .child(btn("Undo".into(), UNDO[i], false)),
                    (_, RecapCardState::Acted) => foot
                        .child(tag(e.done_label.clone(), p.green, p.green_bg))
                        .child(div().flex_1())
                        .child(btn("Undo".into(), UNDO[i], false)),
                    (_, RecapCardState::Busy) => foot
                        .child(div().text_color(p.mute).child("Working…"))
                        .child(div().flex_1()),
                    (k2, _) => {
                        let mut f2 = foot.child(div().flex_1());
                        if k2 == RecapKind::Needs {
                            f2 = f2.child(btn("Review".into(), REVIEW[i], false));
                        }
                        f2.child(btn(e.action.clone(), ACT[i], true))
                    }
                };
                let mut logo = div().size(px(16.)).flex().items_center().justify_center();
                if s == RecapCardState::Acted {
                    logo = logo.text_color(p.green).child("✓");
                } else if let Some(m) = brand_mark(&e.brand, 13.0, p.fg) {
                    logo = logo.child(m);
                }
                let mut l2 = div()
                    .pl(px(24.))
                    .flex()
                    .gap(px(8.))
                    .text_size(px(11.5))
                    .text_color(p.mute)
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .when(k == RecapKind::Failed, |d| d.text_color(p.red))
                            .child(e.detail.clone()),
                    );
                if e.count > 0 {
                    l2 = l2.child(
                        div()
                            .px(px(5.))
                            .rounded(px(4.))
                            .bg(p.chip)
                            .text_color(p.fg2)
                            .child(e.count.to_string()),
                    );
                }
                l2 = l2.child(
                    div()
                        .text_color(p.faint)
                        .child(recap_time(self.since, e.at)),
                );
                lane = lane.child(
                    div()
                        .h(px(88.))
                        .pl(px(12.))
                        .pr(px(10.))
                        .pt(px(10.))
                        .pb(px(8.))
                        .rounded(px(if self.classic { 12. } else { 8. }))
                        .border_1()
                        .border_color(p.bd)
                        .bg(p.card)
                        .flex()
                        .flex_col()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .when(quiet, |d| d.opacity(0.6))
                                .child(logo)
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(e.title.clone()),
                                ),
                        )
                        .child(l2.when(quiet, |d| d.opacity(0.6)))
                        .child(foot),
                );
            }
            board = board.child(lane);
        }
        div()
            .w(px(W))
            .p(px(if self.classic { 20. } else { 16. }))
            .rounded(px(if self.classic { 16. } else { 12. }))
            .border_1()
            .border_color(p.bd)
            .bg(p.card)
            .shadow_sm()
            .text_color(p.fg)
            .text_size(px(12.5))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(head)
            .child(chart)
            .child(board)
    }
}
