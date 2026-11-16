//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Rate Limit Planner**
//! ([`RateLimitPlannerEl`]) over `oxyn_core::parity::ratefid`. The element is `RenderOnce`: the
//! host passes the runs and gets [`Act`] ids back (`spread`, `run-<n>`). A static snapshot: the
//! load is drawn as columns, no dragging, hatching or edit dialog.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::parity::ratefid::{
    RL_BASE, RL_DAY, RL_LIMIT, RlRun, RlState, rl_at, rl_clock, rl_demo, rl_peak, rl_state,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const RUN: [&str; 8] = [
    "run-0", "run-1", "run-2", "run-3", "run-4", "run-5", "run-6", "run-7",
];

/// `rate-limit-planner` (affectUI Rate Limit Planner): the host passes the runs.
#[derive(IntoElement)]
pub struct RateLimitPlannerEl {
    runs: Vec<RlRun>,
    limit: f64,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for RateLimitPlannerEl {
    fn default() -> Self {
        Self {
            runs: rl_demo(),
            limit: RL_LIMIT,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl RateLimitPlannerEl {
    /// The runs and the limit (thousand tokens per minute).
    pub fn runs(mut self, runs: Vec<RlRun>, limit: f64) -> Self {
        self.runs = runs;
        self.limit = limit;
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
    /// Spread out and row clicks.
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

impl RenderOnce for RateLimitPlannerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, vi, red, amb, green) = if dark {
            (
                h(0x171717),
                h(0x2A2A2A),
                h(0xF2F2F2),
                h(0x8B8B90),
                h(0x9B87F5),
                h(0xF2685C),
                h(0xF0A54A),
                h(0x4ADE80),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x111111),
                h(0x8A8A8E),
                h(0x6E56CF),
                h(0xC0362C),
                h(0xB45F06),
                h(0x1A7F4B),
            )
        };
        let runs = &self.runs;
        let (pt, pv) = rl_peak(runs, RL_BASE);
        let st = rl_state(pv, self.limit);
        let tone = match st {
            RlState::Under => green,
            RlState::Close => amb,
            RlState::Over => red,
        };
        let ymax = (self.limit * 1.18).max(pv * 1.06);
        let mut chart = div().h(px(110.)).flex().items_end().gap(px(1.));
        for k in 0..72u32 {
            let v = rl_at(runs, RL_BASE, k * 20 + 10);
            chart = chart.child(
                div()
                    .flex_1()
                    .h(relative((v / ymax) as f32))
                    .rounded_t(px(1.))
                    .bg(if v > self.limit { red } else { vi }),
            );
        }
        let mut rows = div().flex().flex_col();
        for (i, r) in runs.iter().enumerate().take(RUN.len()) {
            let f = fire(&self.act, RUN[i]);
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("oxrl-{i}")))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .w(px(132.))
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if r.started { mute } else { fg })
                            .child(r.name.clone()),
                    )
                    .child(div().w(px(50.)).text_size(px(11.)).text_color(mute).child(
                        if r.started {
                            "Running".to_string()
                        } else {
                            rl_clock(r.start)
                        },
                    ))
                    .child(
                        div().flex_1().relative().h(px(12.)).child(
                            div()
                                .absolute()
                                .left(relative(r.start as f32 / RL_DAY as f32))
                                .w(relative(
                                    ((r.climb + r.half * 5) as f32 / RL_DAY as f32).min(1.),
                                ))
                                .h_full()
                                .rounded(px(if self.classic { 2. } else { 5. }))
                                .border_1()
                                .border_color(vi)
                                .opacity(if r.started { 0.5 } else { 1. }),
                        ),
                    ),
            );
        }
        let spread = fire(&self.act, "spread");
        div()
            .w(px(560.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(8.))
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
                            .text_size(px(13.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Rate limit"),
                    )
                    .child(div().flex_1())
                    .child(div().text_size(px(11.)).text_color(tone).child(format!(
                        "Peak {:.0}% · {}",
                        pv / self.limit * 100.,
                        rl_clock(pt)
                    )))
                    .child(
                        div()
                            .id("spread")
                            .px(px(10.))
                            .h(px(26.))
                            .flex()
                            .items_center()
                            .border_1()
                            .border_color(bd)
                            .rounded(px(8.))
                            .text_size(px(12.))
                            .on_click(move |_, w, cx| spread(w, cx))
                            .child("Spread out"),
                    ),
            )
            .child(chart)
            .child(rows)
    }
}
