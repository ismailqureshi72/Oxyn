//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Plan vs Actual**
//! ([`PlanVsActualEl`]) over `oxyn_core::kit::planfid`. The element is `RenderOnce`: the host
//! passes the plan and the elapsed seconds and gets [`Act`] ids back (`step-<n>`). A static
//! snapshot: no sheen, links or gliding.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::planfid::{
    PvState, PvStep, pv_actual_total, pv_axis, pv_delta, pv_demo, pv_eta, pv_fmt, pv_plan_total,
    pv_rows,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const STEP: [&str; 10] = [
    "step-0", "step-1", "step-2", "step-3", "step-4", "step-5", "step-6", "step-7", "step-8",
    "step-9",
];

/// `plan-vs-actual` (affectUI Plan vs Actual): the host passes the plan and the time.
#[derive(IntoElement)]
pub struct PlanVsActualEl {
    steps: Vec<PvStep>,
    at: u32,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for PlanVsActualEl {
    fn default() -> Self {
        let s = pv_demo();
        let at = pv_actual_total(&s);
        Self {
            steps: s,
            at,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl PlanVsActualEl {
    /// The plan and the seconds elapsed.
    pub fn plan(mut self, steps: Vec<PvStep>, at: u32) -> Self {
        self.steps = steps;
        self.at = at;
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
    /// Step clicks.
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

impl RenderOnce for PlanVsActualEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, vi, amb, green, ghost) = if dark {
            (
                h(0x171717),
                h(0x2A2A2A),
                h(0xF2F2F2),
                h(0x8B8B90),
                h(0x9B87F5),
                h(0xF0A54A),
                h(0x4ADE80),
                h(0x26223A),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0x6E56CF),
                h(0xB45F06),
                h(0x1A7F4B),
                h(0xF1EEFB),
            )
        };
        let s = &self.steps;
        let rows = pv_rows(s, self.at);
        let plan = pv_plan_total(s);
        let eta = pv_eta(&rows);
        let axis = pv_axis(plan, eta).max(1) as f32;
        let mut list = div().flex().flex_col();
        for (i, (x, r)) in s.iter().zip(rows.iter()).enumerate().take(10) {
            let f = fire(&self.act, STEP[i]);
            let end = if r.state == PvState::Running {
                self.at
            } else {
                r.end
            };
            let el = end.saturating_sub(r.start);
            let over = r.state != PvState::Waiting && r.delta > 0;
            list = list.child(
                div()
                    .id(SharedString::from(format!("oxpv-{i}")))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .w(px(150.))
                            .text_size(px(12.5))
                            .text_color(if r.state == PvState::Waiting {
                                mute
                            } else {
                                fg
                            })
                            .child(x.title.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .relative()
                            .h(px(10.))
                            .child(
                                div()
                                    .absolute()
                                    .left(relative(r.plan_start as f32 / axis))
                                    .w(relative(x.plan as f32 / axis))
                                    .h_full()
                                    .rounded(px(5.))
                                    .border_1()
                                    .border_color(vi)
                                    .bg(ghost),
                            )
                            .when(r.state != PvState::Waiting, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .left(relative(r.start as f32 / axis))
                                        .w(relative(el as f32 / axis))
                                        .h_full()
                                        .rounded(px(5.))
                                        .bg(if over { amb } else { vi }),
                                )
                            }),
                    )
                    .child(
                        div()
                            .w(px(76.))
                            .pl(px(6.))
                            .text_size(px(11.))
                            .text_color(if r.delta > 0 { amb } else { mute })
                            .child(match r.state {
                                PvState::Waiting => String::new(),
                                _ if r.delta != 0 && (r.state == PvState::Done || over) => {
                                    format!("{} {}", pv_fmt(el), pv_delta(r.delta))
                                }
                                _ => pv_fmt(el),
                            }),
                    ),
            );
        }
        let late = eta > plan;
        div()
            .w(px(640.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .border_1()
            .border_color(bd)
            .rounded(px(12.))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Plan vs actual"),
                    )
                    .child(div().flex_1())
                    .child(div().text_size(px(11.5)).text_color(mute).child(format!(
                        "Time {} / {}",
                        pv_fmt(self.at.min(eta)),
                        pv_fmt(plan)
                    ))),
            )
            .child(list)
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .text_size(px(11.))
                    .child(
                        div()
                            .text_color(mute)
                            .child(format!("Plan {}", pv_fmt(plan))),
                    )
                    .child(
                        div()
                            .text_color(if late { amb } else { green })
                            .child(format!("ETA {}", pv_fmt(eta))),
                    )
                    .when(self.classic, |d| d.flex_row_reverse()),
            )
    }
}
