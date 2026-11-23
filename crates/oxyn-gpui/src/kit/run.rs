//! Plans, progress and agents for GPUI (plan #28, chunk 2).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    kit::{
        fleet::{
            Countdown, CrewAgent, Feed, Orchestration, Readiness, TreeTable, Tri, dollars, group,
        },
        plan::{
            Breakdown, Checklist, PlanDraft, PlanPhase, StepState, Stepper, TaskRow, Trail,
            TrailStep, short_duration,
        },
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, muted, tag};

fn mark(t: &'static Theme, s: StepState) -> impl IntoElement {
    let (g, fg, bg) = match s {
        StepState::Done => (
            "✓",
            ColorRole::StatusSuccessFg,
            Some(ColorRole::StatusSuccessBg),
        ),
        StepState::Running => ("◌", ColorRole::FgDefault, None),
        StepState::Waiting => ("⏸", ColorRole::BgBase, Some(ColorRole::StatusWarningFg)),
        StepState::Failed => (
            "×",
            ColorRole::StatusDangerFg,
            Some(ColorRole::StatusDangerBg),
        ),
        StepState::Declined | StepState::Skipped => {
            ("–", ColorRole::FgMuted, Some(ColorRole::SurfaceSunken))
        }
        StepState::Pending => ("", ColorRole::FgMuted, None),
    };
    div()
        .flex_none()
        .size(px(16.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.))
        .text_color(c(t, fg))
        .when_some(bg, |d, b| d.bg(c(t, b)))
        .when(bg.is_none(), |d| {
            d.border_1().border_color(c(t, ColorRole::BorderStrong))
        })
        .child(g)
}

fn header(
    t: &'static Theme,
    title: impl Into<SharedString>,
    right: impl Into<SharedString>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(14.))
                .child(title.into()),
        )
        .child(div().flex_1())
        .child(muted(t, right))
}

macro_rules! el {
    ($name:ident, $model:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(IntoElement)]
        pub struct $name {
            model: $model,
            now: ClockMs,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl $name {
            /// The element at `now`.
            pub fn new(model: $model, now: ClockMs) -> Self {
                Self {
                    model,
                    now,
                    act: None,
                    theme: &Theme::DARK,
                }
            }
            /// Button reports.
            pub fn on_act(mut self, f: Act) -> Self {
                self.act = Some(f);
                self
            }
            /// The clock this element renders at.
            pub fn clock(&self) -> ClockMs {
                self.now
            }
        }
    };
}

el!(
    AgentTrailEl,
    Trail,
    "`agent-trail`: header with the count, marks, live timer, sub-steps."
);

el!(
    FlowStepperEl,
    Stepper,
    "`stepper`: horizontal markers with connectors and titles."
);

impl RenderOnce for AgentTrailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let tr = &self.model;
        let now = self.now;
        let cur = tr.current();
        let fg = c(t, ColorRole::FgDefault);
        let warn = c(t, ColorRole::StatusWarningFg);
        let answer = self.act.is_some();
        let mut list = div().flex().flex_col();
        if tr.steps.is_empty() {
            list = list.child(muted(t, "No steps yet"));
        }
        for (i, s) in tr.steps.iter().enumerate() {
            let time = oxyn_core::kit::plan::trail_time(s, now);
            let title = if s.title.is_empty() {
                "Untitled step".to_string()
            } else {
                s.title.clone()
            };
            let gate_pending = s.gate && s.state == StepState::Pending;
            let mk = if gate_pending {
                div()
                    .flex_none()
                    .size(px(16.))
                    .rounded_full()
                    .border_1()
                    .border_dashed()
                    .border_color(warn)
                    .into_any_element()
            } else {
                mark(t, s.state).into_any_element()
            };
            let faded = s.state == StepState::Pending;
            let row = div()
                .h(px(32.))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(mk)
                .when_some(s.tool.clone(), |d, x| {
                    d.child(
                        div()
                            .flex_none()
                            .when(faded, |d| d.opacity(0.5))
                            .children(crate::brand::brand_mark(&x, 14., fg)),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .when(faded, |d| d.opacity(0.5))
                        .when(s.state == StepState::Declined, |d| {
                            d.line_through().text_color(c(t, ColorRole::FgMuted))
                        })
                        .when(s.state == StepState::Failed, |d| {
                            d.text_color(c(t, ColorRole::StatusDangerFg))
                        })
                        .child(title.clone()),
                )
                .when_some(
                    s.status.clone().filter(|_| s.state == StepState::Running),
                    |d, x| d.child(muted(t, x)),
                )
                .when_some(time, |d, x| d.child(muted(t, x)))
                .when_some(s.badge(), |d, b| {
                    let col = match s.state {
                        StepState::Failed => Some(ColorRole::StatusDangerFg),
                        StepState::Declined => None,
                        _ => Some(ColorRole::StatusWarningFg),
                    };
                    d.child(tag(t, b, col))
                });
            list = list.child(row);
            if s.state == StepState::Waiting && answer {
                list =
                    list.child(
                        div()
                            .ml(px(24.))
                            .mb(px(6.))
                            .p(px(10.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(warn)
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(s.ask.clone().unwrap_or_else(|| {
                                format!("Let the agent {}", title.to_lowercase())
                            }))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(button(
                                        t,
                                        "trail-approve",
                                        "Approve",
                                        Weight::Primary,
                                        "trail-approve",
                                        self.act.clone(),
                                    ))
                                    .child(button(
                                        t,
                                        "trail-decline",
                                        "Decline",
                                        Weight::Plain,
                                        "trail-decline",
                                        self.act.clone(),
                                    )),
                            ),
                    );
            }
            if Some(i) == cur && s.state == StepState::Running && !s.sub.is_empty() {
                let f = oxyn_core::kit::plan::fold(s.sub.len(), s.current_sub(), false);
                let k = s.current_sub();
                let mut subs = div().ml(px(24.)).flex().flex_col();
                if f.earlier > 0 {
                    subs = subs.child(
                        div()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .child(muted(t, format!("+{} earlier", f.earlier))),
                    );
                }
                for j in f.start..f.end {
                    let (x, st) = &s.sub[j];
                    let on = Some(j) == k && *st == StepState::Running;
                    subs = subs.child(
                        div()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .gap(px(7.))
                            .text_size(px(12.))
                            .child(mark(t, *st))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .text_color(if on { fg } else { c(t, ColorRole::FgMuted) })
                                    .child(x.clone()),
                            )
                            .when_some(s.detail(j, 24), |d, x| d.child(muted(t, x)))
                            .when(on, |d| {
                                d.child(
                                    div()
                                        .w(px(18.))
                                        .h(px(2.))
                                        .rounded(px(1.))
                                        .bg(alpha(fg, 0.6)),
                                )
                            }),
                    );
                }
                if f.more > 0 {
                    subs = subs.child(
                        div()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .child(muted(t, format!("+{} more", f.more))),
                    );
                }
                list = list.child(subs);
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_color(fg)
            .text_size(px(13.))
            .when(!tr.title.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .when_some(tr.agent.clone(), |d, a| {
                            d.children(crate::brand::brand_mark(&a, 14., fg))
                                .child(muted(t, a))
                        })
                        .child(
                            div()
                                .flex_1()
                                .child(header(t, tr.title.clone(), tr.count())),
                        ),
                )
            })
            .child(list)
    }
}

impl RenderOnce for FlowStepperEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = &self.model;
        div()
            .flex()
            .w_full()
            .children(s.steps.iter().enumerate().map(|(i, st)| {
                let state = s.state(i);
                let (fg, bg) = match state {
                    StepState::Done => (c(t, ColorRole::BgBase), c(t, ColorRole::FgDefault)),
                    StepState::Failed => (
                        c(t, ColorRole::StatusDangerFg),
                        c(t, ColorRole::SurfaceBase),
                    ),
                    _ => (c(t, ColorRole::FgMuted), c(t, ColorRole::SurfaceBase)),
                };
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .size(px(26.))
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .border_1()
                                    .border_color(if state == StepState::Running {
                                        c(t, ColorRole::FgDefault)
                                    } else {
                                        c(t, ColorRole::BorderStrong)
                                    })
                                    .bg(bg)
                                    .text_color(fg)
                                    .text_size(px(12.))
                                    .child(match state {
                                        StepState::Done => "✓".to_string(),
                                        StepState::Failed => "!".to_string(),
                                        _ => format!("{}", i + 1),
                                    }),
                            )
                            .when(i + 1 < s.steps.len(), |d| {
                                d.child(div().flex_1().mx(px(6.)).h(px(2.)).bg(
                                    if s.connector(i) > 0.0 {
                                        c(t, ColorRole::FgDefault)
                                    } else {
                                        c(t, ColorRole::BorderDefault)
                                    },
                                ))
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(st.title.clone()),
                    )
                    .when_some(st.error.clone(), |d, e| {
                        d.child(
                            div()
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::StatusDangerFg))
                                .child(e),
                        )
                    })
            }))
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
