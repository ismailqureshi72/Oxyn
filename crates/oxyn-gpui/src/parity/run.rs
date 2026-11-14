//! Plans, progress and agents for GPUI (plan #28, chunk 2).

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    parity::{
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

fn bar(t: &'static Theme, p: f32) -> impl IntoElement {
    div()
        .h(px(6.))
        .w_full()
        .rounded(px(999.))
        .bg(c(t, ColorRole::SurfaceSunken))
        .child(
            div()
                .h_full()
                .w(relative(p.clamp(0.0, 1.0)))
                .rounded(px(999.))
                .bg(c(t, ColorRole::FgDefault)),
        )
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
    AgentPlanEl,
    PlanDraft,
    "`agent-plan`: steps with timers; reports `run`, `cancel`, `retry`."
);
el!(
    AgentTrailEl,
    Trail,
    "`agent-trail`: header with the count, marks, live timer, sub-steps."
);
el!(
    TaskRowsEl,
    Vec<TaskRow>,
    "`task-rows`: rows with a ring or mark, pill and details."
);
el!(
    FlowStepperEl,
    Stepper,
    "`stepper`: horizontal markers with connectors and titles."
);
el!(
    ChecklistEl,
    Checklist,
    "`onboarding-checklist`: progress, items, open item body."
);
el!(
    ReadinessEl,
    Readiness,
    "`release-readiness`: bar, prerequisites, release button (`release`)."
);
el!(
    BreakdownEl,
    Breakdown,
    "`task-breakdown`: tasks with a Gantt strip and the verdict."
);
el!(
    AgentTreeEl,
    CrewAgent,
    "`agent-tree`: indented cards with cost roll-up; reports `approve`, `reject`, `retry`, `dismiss` for the first agent that needs one."
);
el!(
    FleetEl,
    Vec<CrewAgent>,
    "`agent-fleet`: one summary card per main agent."
);
el!(
    OrchestratorEl,
    Orchestration,
    "`agent-orchestrator`: lead, lanes with progress, status line."
);
el!(
    TreeTableEl,
    TreeTable,
    "`tree-table`: visible rows with indentation and tri-state marks."
);
el!(
    ActivityFeedEl,
    Feed,
    "`activity-feed` / `timeline`: day groups and event rows; reports `show-new`, `read-all`."
);
el!(
    CountdownEl,
    Countdown,
    "`countdown`: large digits, or Live now at zero."
);

impl RenderOnce for AgentPlanEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = &self.model;
        let (d, n) = p.progress();
        let act = self.act;
        card(t)
            .max_w(px(520.))
            .child(header(
                t,
                p.title.clone(),
                if p.editable() {
                    p.estimate.clone().unwrap_or_default()
                } else {
                    format!("{d}/{n}")
                },
            ))
            .children(p.steps.iter().enumerate().map(|(i, s)| {
                div()
                    .flex()
                    .gap(px(8.))
                    .items_center()
                    .py(px(4.))
                    .child(mark(t, s.state))
                    .child(muted(t, format!("{}", i + 1)))
                    .child(div().flex_1().child(s.title.clone()))
                    .when_some(s.tool.clone(), |d, x| d.child(tag(t, x, None)))
                    .when_some(s.timer(self.now), |d, x| d.child(muted(t, x)))
            }))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .justify_end()
                    .when(p.editable(), |d| {
                        d.child(button(
                            t,
                            "pl-cancel",
                            "Cancel",
                            Weight::Plain,
                            "cancel",
                            act.clone(),
                        ))
                        .child(button(
                            t,
                            "pl-run",
                            "Run plan",
                            Weight::Primary,
                            "run",
                            act.clone(),
                        ))
                    })
                    .when(p.phase == PlanPhase::Done, |d| {
                        d.child(button(
                            t,
                            "pl-again",
                            "Run again",
                            Weight::Plain,
                            "run",
                            act.clone(),
                        ))
                    })
                    .when(p.steps.iter().any(|s| s.state == StepState::Failed), |d| {
                        d.child(button(
                            t,
                            "pl-retry",
                            "Retry from here",
                            Weight::Plain,
                            "retry",
                            act.clone(),
                        ))
                    }),
            )
    }
}

impl RenderOnce for AgentTrailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let tr = &self.model;
        let now = self.now;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .when(!tr.title.is_empty(), |d| {
                d.child(header(t, tr.title.clone(), tr.count()))
            })
            .children(tr.steps.iter().map(|s| {
                let time = match (s.state, s.started, s.took_ms) {
                    (StepState::Running, Some(st), _) => {
                        Some(short_duration(now.0.saturating_sub(st.0)))
                    }
                    (_, _, Some(x)) => Some(short_duration(x)),
                    _ => None,
                };
                div().flex().gap(px(10.)).child(mark(t, s.state)).child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .gap(px(2.))
                        .child(
                            div()
                                .flex()
                                .gap(px(7.))
                                .when(s.state == StepState::Pending, |d| d.opacity(0.55))
                                .when_some(s.tool.clone(), |d, x| d.child(tag(t, x, None)))
                                .child(s.title.clone())
                                .when_some(s.badge(), |d, b| {
                                    d.child(tag(t, b, Some(ColorRole::StatusWarningFg)))
                                })
                                .child(div().flex_1())
                                .when_some(time, |d, x| d.child(muted(t, x))),
                        )
                        .when_some(
                            s.status.clone().filter(|_| s.state == StepState::Running),
                            |d, x| d.child(muted(t, format!("{x}…"))),
                        )
                        .when(s.state == StepState::Running, |d| {
                            d.children(s.sub.iter().map(|(x, st)| {
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(mark(t, *st))
                                    .child(muted(t, x.clone()))
                            }))
                        }),
                )
            }))
    }
}

impl RenderOnce for TaskRowsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w_full()
            .children(self.model.iter().map(|r| {
                div()
                    .flex()
                    .flex_col()
                    .px(px(12.))
                    .py(px(9.))
                    .rounded(px(18.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .child(
                        div()
                            .flex()
                            .gap(px(9.))
                            .items_center()
                            .child(mark(t, r.state))
                            .child(r.label.clone())
                            .child(div().flex_1())
                            .when_some(r.amount.clone(), |d, a| d.child(muted(t, a)))
                            .when_some(r.pill(), |d, p| d.child(tag(t, p, None))),
                    )
                    .when(r.state == StepState::Running, |d| {
                        d.child(div().mt(px(6.)).child(bar(t, r.ring())))
                    })
                    .when(r.open, |d| {
                        d.children(r.details.iter().map(|(l, m)| {
                            div()
                                .flex()
                                .pl(px(25.))
                                .child(muted(t, l.clone()))
                                .child(div().flex_1())
                                .child(muted(t, m.clone()))
                        }))
                    })
            }))
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

impl RenderOnce for ChecklistEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let l = &self.model;
        let (d, n) = l.count();
        card(t)
            .max_w(px(420.))
            .child(header(
                t,
                if l.all_done() {
                    "You’re all set"
                } else {
                    "Set up your workspace"
                },
                format!("{d} of {n}"),
            ))
            .child(bar(t, l.ring()))
            .when(!l.hidden, |dv| {
                dv.children(l.items.iter().map(|it| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .py(px(4.))
                        .child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .child(mark(
                                    t,
                                    if it.done {
                                        StepState::Done
                                    } else {
                                        StepState::Pending
                                    },
                                ))
                                .child(it.title.clone()),
                        )
                        .when(l.open.as_deref() == Some(it.id.as_str()) && !it.done, |x| {
                            x.child(div().pl(px(24.)).child(muted(t, it.body.clone())))
                        })
                }))
            })
    }
}

impl RenderOnce for ReadinessEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = &self.model;
        let (d, n) = r.count();
        card(t)
            .max_w(px(480.))
            .child(header(
                t,
                format!("Release {}", r.version),
                format!("{d}/{n}"),
            ))
            .child(bar(t, d as f32 / n.max(1) as f32))
            .children(r.items.iter().map(|it| {
                div()
                    .flex()
                    .gap(px(8.))
                    .items_center()
                    .child(mark(
                        t,
                        if it.done {
                            StepState::Done
                        } else {
                            StepState::Pending
                        },
                    ))
                    .child(it.title.clone())
                    .when(it.blocking && !it.done, |x| {
                        x.child(tag(t, "Blocker", Some(ColorRole::StatusDangerFg)))
                    })
                    .child(div().flex_1())
                    .child(muted(t, it.owner.clone()))
            }))
            .when_some(r.error.clone(), |x, e| {
                x.child(div().text_color(c(t, ColorRole::StatusDangerFg)).child(e))
            })
            .child(div().flex().justify_end().child(button(
                t,
                "rr-go",
                format!("Release {}", r.version),
                if r.can_release() {
                    Weight::Primary
                } else {
                    Weight::Plain
                },
                "release",
                self.act.filter(|_| r.can_release()),
            )))
    }
}

impl RenderOnce for BreakdownEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let b = &self.model;
        let sched = b.schedule();
        let span = b.span().max(b.deadline_days).max(1.0);
        card(t)
            .max_w(px(560.))
            .child(
                div()
                    .p(px(8.))
                    .rounded(px(10.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .child(b.goal.clone()),
            )
            .children(b.tasks.iter().enumerate().map(|(i, x)| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(x.title.clone())
                            .child(div().flex_1())
                            .child(muted(t, format!("{}d · {}", x.days, x.owner))),
                    )
                    .child(
                        div()
                            .relative()
                            .h(px(5.))
                            .rounded(px(3.))
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .child(
                                div()
                                    .absolute()
                                    .top(px(0.))
                                    .bottom(px(0.))
                                    .left(relative(sched[i].0 / span))
                                    .w(relative(x.days / span))
                                    .rounded(px(3.))
                                    .bg(c(t, ColorRole::FgMuted)),
                            ),
                    )
            }))
            .child(
                div()
                    .text_color(c(
                        t,
                        if b.on_track() {
                            ColorRole::StatusSuccessFg
                        } else {
                            ColorRole::StatusDangerFg
                        },
                    ))
                    .child(b.verdict()),
            )
    }
}

fn agent_card(t: &'static Theme, a: &CrewAgent, depth: usize, now: ClockMs) -> impl IntoElement {
    let cost = if a.children.is_empty() {
        a.cost
    } else {
        a.branch_cost()
    };
    div()
        .ml(px(depth as f32 * 22.))
        .flex()
        .gap(px(9.))
        .p(px(9.))
        .rounded(px(10.))
        .border_1()
        .border_color(match a.state {
            StepState::Waiting => c(t, ColorRole::StatusWarningBorder),
            StepState::Failed => c(t, ColorRole::StatusDangerBorder),
            _ => c(t, ColorRole::BorderDefault),
        })
        .bg(c(t, ColorRole::SurfaceRaised))
        .when(
            matches!(a.state, StepState::Pending | StepState::Skipped),
            |d| d.opacity(0.6),
        )
        .child(mark(t, a.state))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .child(a.name.clone())
                        .when(a.state == StepState::Waiting, |d| {
                            d.child(tag(t, "Needs you", Some(ColorRole::StatusWarningFg)))
                        }),
                )
                .child(muted(t, a.task.clone())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_end()
                .child(muted(t, dollars(cost)))
                .when_some(a.time(now), |d, x| d.child(muted(t, x))),
        )
}

impl RenderOnce for AgentTreeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut rows = Vec::new();
        self.model.rows(&[], 0, &mut rows);
        let (r, w, d, f) = self.model.tally();
        let needs = rows
            .iter()
            .find(|(_, a)| matches!(a.state, StepState::Waiting | StepState::Failed))
            .map(|(_, a)| a.state);
        let act = self.act;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .child(header(
                t,
                "Agents",
                format!(
                    "{r} running · {w} need you · {d} done · {f} failed · {}",
                    dollars(self.model.branch_cost())
                ),
            ))
            .children(rows.iter().map(|(dp, a)| agent_card(t, a, *dp, self.now)))
            .when_some(needs, |x, s| {
                let (a1, a2, l1, l2) = if s == StepState::Waiting {
                    ("reject", "approve", "Reject", "Approve")
                } else {
                    ("dismiss", "retry", "Dismiss", "Retry")
                };
                x.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .justify_end()
                        .child(button(t, "at-a", l1, Weight::Plain, a1, act.clone()))
                        .child(button(t, "at-b", l2, Weight::Primary, a2, act.clone())),
                )
            })
    }
}

impl RenderOnce for FleetEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_wrap()
            .gap(px(8.))
            .children(self.model.iter().map(|a| {
                let (r, w, d, f) = a.tally();
                card(t)
                    .w(px(220.))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(mark(t, a.state))
                            .child(a.name.clone())
                            .child(div().flex_1())
                            .child(muted(t, dollars(a.branch_cost()))),
                    )
                    .child(muted(t, a.task.clone()))
                    .child(muted(
                        t,
                        format!("{r} running · {w} need you · {d} done · {f} failed"),
                    ))
            }))
    }
}

impl RenderOnce for OrchestratorEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let o = &self.model;
        card(t)
            .child(header(
                t,
                o.goal.clone(),
                format!("{} · parallel {}", o.lead, o.concurrency),
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .children(o.lanes.iter().map(|l| {
                        div()
                            .w(px(150.))
                            .flex()
                            .flex_col()
                            .gap(px(5.))
                            .p(px(9.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .when(l.state == StepState::Pending, |d| d.opacity(0.6))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .child(mark(t, l.state))
                                    .child(l.title.clone()),
                            )
                            .child(muted(t, l.worker.clone()))
                            .child(bar(t, l.progress))
                            .when_some(l.result.clone(), |d, r| d.child(muted(t, r)))
                    })),
            )
            .child(muted(t, o.status()))
            .when_some(o.summary.clone(), |d, s| d.child(s))
    }
}

impl RenderOnce for TreeTableEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault))
            .children(m.visible().into_iter().map(|(d, r)| {
                let tri = m.tri(&r.id);
                div()
                    .flex()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .when(tri == Tri::On, |x| {
                        x.bg(alpha(c(t, ColorRole::FgDefault), 0.06))
                    })
                    .child(div().w(px(14.)).child(match tri {
                        Tri::On => "☑",
                        Tri::Mixed => "▣",
                        Tri::Off => "☐",
                    }))
                    .child(div().w(px(d as f32 * 18.)))
                    .child(if r.children.is_empty() && !r.lazy {
                        " "
                    } else if m.expanded.contains(&r.id) {
                        "▾"
                    } else {
                        "▸"
                    })
                    .children(r.cells.iter().enumerate().map(|(i, x)| {
                        let cell = div().child(x.clone());
                        if i == 0 {
                            cell.flex_1()
                        } else {
                            cell.w(px(90.)).text_color(c(t, ColorRole::FgMuted))
                        }
                    }))
            }))
    }
}

impl RenderOnce for ActivityFeedEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let f = &self.model;
        let act = self.act;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Activity"))
                    .child(div().flex_1())
                    .child(button(
                        t,
                        "af-read",
                        "Mark all read",
                        Weight::Quiet,
                        "read-all",
                        act.clone(),
                    )),
            )
            .when(!f.incoming.is_empty(), |d| {
                d.child(button(
                    t,
                    "af-new",
                    format!("{} new updates", f.incoming.len()),
                    Weight::Primary,
                    "show-new",
                    act.clone(),
                ))
            })
            .children(f.groups().into_iter().map(|(day, evs)| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(muted(t, day.to_uppercase()))
                    .children(evs.into_iter().map(|e| {
                        div()
                            .flex()
                            .gap(px(6.))
                            .when(e.unread, |d| {
                                d.child(
                                    div()
                                        .size(px(6.))
                                        .rounded_full()
                                        .bg(c(t, ColorRole::StatusInfoFg)),
                                )
                            })
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(e.actor.clone()),
                            )
                            .child(muted(t, e.action.clone()))
                            .child(e.target.clone())
                            .child(div().flex_1())
                            .child(muted(t, e.time.clone()))
                    }))
            }))
    }
}

impl RenderOnce for CountdownEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        match self.model.parts(self.now, false) {
            None => div()
                .flex()
                .gap(px(8.))
                .items_center()
                .text_size(px(18.))
                .text_color(c(t, ColorRole::FgDefault))
                .child(
                    div()
                        .size(px(9.))
                        .rounded_full()
                        .bg(c(t, ColorRole::StatusDangerFg)),
                )
                .child("Live now"),
            Some(p) => div().flex().gap(px(14.)).children(
                ["days", "hours", "minutes", "seconds"]
                    .into_iter()
                    .zip(p)
                    .map(|(u, v)| {
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(36.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(c(t, ColorRole::FgDefault))
                                    .child(format!("{v:02}")),
                            )
                            .child(muted(t, u))
                    }),
            ),
        }
    }
}

/// `animated-counter`: prefix, grouped digits, suffix (the host animates value changes).
#[derive(IntoElement)]
pub struct CounterEl {
    value: f64,
    decimals: usize,
    prefix: SharedString,
    suffix: SharedString,
    theme: &'static Theme,
}

impl CounterEl {
    /// A counter.
    pub fn new(value: f64, decimals: usize) -> Self {
        Self {
            value,
            decimals,
            prefix: "".into(),
            suffix: "".into(),
            theme: &Theme::DARK,
        }
    }
    /// Prefix and suffix.
    pub fn affix(
        mut self,
        prefix: impl Into<SharedString>,
        suffix: impl Into<SharedString>,
    ) -> Self {
        self.prefix = prefix.into();
        self.suffix = suffix.into();
        self
    }
}

impl RenderOnce for CounterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .items_end()
            .text_color(c(t, ColorRole::FgDefault))
            .child(muted(t, self.prefix))
            .child(
                div()
                    .text_size(px(32.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(group(self.value, self.decimals)),
            )
            .child(muted(t, self.suffix))
    }
}

/// `agent-status`: avatar with a state dot, name, state and activity, live timer.
#[derive(IntoElement)]
pub struct AgentStatusEl {
    name: SharedString,
    state: &'static str,
    activity: Option<SharedString>,
    started: Option<ClockMs>,
    now: ClockMs,
    theme: &'static Theme,
}

impl AgentStatusEl {
    /// A status (`idle` · `thinking` · `acting` · `waiting` · `error` · `done`).
    pub fn new(name: impl Into<SharedString>, state: &'static str, now: ClockMs) -> Self {
        Self {
            name: name.into(),
            state,
            activity: None,
            started: None,
            now,
            theme: &Theme::DARK,
        }
    }
    /// What it does and since when.
    pub fn activity(mut self, a: impl Into<SharedString>, started: Option<ClockMs>) -> Self {
        self.activity = Some(a.into());
        self.started = started;
        self
    }
}

impl RenderOnce for AgentStatusEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let dot = match self.state {
            "thinking" | "acting" => ColorRole::StatusInfoFg,
            "waiting" => ColorRole::StatusWarningFg,
            "error" => ColorRole::StatusDangerFg,
            "done" => ColorRole::StatusSuccessFg,
            _ => ColorRole::FgSubtle,
        };
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .pl(px(4.))
            .pr(px(12.))
            .py(px(4.))
            .rounded(px(999.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault))
            .child(div().size(px(8.)).rounded_full().bg(c(t, dot)))
            .child(self.name)
            .child(muted(t, self.state))
            .when_some(self.activity, |d, a| d.child(muted(t, a)))
            .when_some(self.started, |d, s| {
                d.child(muted(t, short_duration(self.now.0.saturating_sub(s.0))))
            })
    }
}

/// `agent-run`: one card following a coding agent through a task: header with the model, the
/// steps as a trail, a proposed change (file + diff) to approve or send back, then the result
/// with Open pull request. Reports `approve`, `reject` and `pr`.
#[derive(IntoElement)]
pub struct AgentRunEl {
    task: SharedString,
    model: SharedString,
    steps: Vec<TrailStep>,
    change: Option<(String, Vec<String>)>,
    result: Option<SharedString>,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl AgentRunEl {
    /// A run of `task` on `model` with the steps so far.
    pub fn new(
        task: impl Into<SharedString>,
        model: impl Into<SharedString>,
        steps: Vec<TrailStep>,
        now: ClockMs,
    ) -> Self {
        Self {
            task: task.into(),
            model: model.into(),
            steps,
            change: None,
            result: None,
            now,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// A proposed change `(file, diff lines)` waiting for review.
    pub fn change(mut self, file: impl Into<String>, diff: Vec<String>) -> Self {
        self.change = Some((file.into(), diff));
        self
    }
    /// The result line once done ("12 tests passed").
    pub fn result(mut self, r: impl Into<SharedString>) -> Self {
        self.result = Some(r.into());
        self
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for AgentRunEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let act = self.act.clone();
        let initial: SharedString = self.model.chars().next().unwrap_or('M').to_string().into();
        let trail = Trail {
            title: String::new(),
            agent: None,
            steps: self.steps,
        };
        card(t)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex_none()
                            .size(px(26.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(initial),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(self.task))
                            .child(muted(t, self.model)),
                    ),
            )
            .child(AgentTrailEl::new(trail, self.now))
            .when_some(self.change, |d, (file, diff)| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .p(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(div().text_size(px(12.)).child(SharedString::from(file)))
                                .child(div().flex_1())
                                .child(tag(t, "Review", Some(ColorRole::StatusWarningFg))),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .p(px(8.))
                                .rounded(px(8.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .children(diff.into_iter().map(|l| {
                                    let r = if l.starts_with('+') {
                                        ColorRole::StatusSuccessFg
                                    } else if l.starts_with('-') {
                                        ColorRole::StatusDangerFg
                                    } else {
                                        ColorRole::FgMuted
                                    };
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(c(t, r))
                                        .child(SharedString::from(l))
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap(px(8.))
                                .child(button(
                                    t,
                                    "run-rej",
                                    "Request changes",
                                    Weight::Plain,
                                    "reject",
                                    act.clone(),
                                ))
                                .child(button(
                                    t,
                                    "run-ok",
                                    "Approve & write",
                                    Weight::Primary,
                                    "approve",
                                    act.clone(),
                                )),
                        ),
                )
            })
            .when_some(self.result, |d, r| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_color(c(t, ColorRole::StatusSuccessFg))
                                .child("✓"),
                        )
                        .child(r)
                        .child(div().flex_1())
                        .child(button(
                            t,
                            "run-pr",
                            "Open pull request",
                            Weight::Primary,
                            "pr",
                            act,
                        )),
                )
            })
    }
}
