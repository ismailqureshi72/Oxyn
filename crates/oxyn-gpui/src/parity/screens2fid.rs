//! Screens 2 for GPUI over `oxyn_core::parity::screens2fid`: static snapshots of the agent screens
//! ([`OrchestrationEl`], [`AgentThreadEl`]) in the Screens 1 frame with the agent sidebar. Each
//! takes the data state (Sample, Loading, Empty), the frame (Inset or Flat) and the theme; the host
//! owns any live state. Rows a host can route carry `Act` ids: `nav-<key>` in the sidebar,
//! `agent-<id>` on the orchestration tree, `task-<id>` on thread task cards.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::parity::screens2fid::{
    AgStatus, OrchNode, ScreenState, ShellLayout, ThreadItem, clock, dur, eur, orch_children,
    orch_done, orch_rollup, orch_trace, orch_tree, thread_counts, thread_items, toks, tools_line,
    trace_span,
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{empty, frame, gh, page, row, screen_el, skel};

const NAV: [(&str, &str); 12] = [
    ("thread", "Thread"),
    ("inbox", "Inbox  5"),
    ("tasks", "Tasks"),
    ("agents", "Agents"),
    ("runs", "Runs"),
    ("memory", "Memory"),
    ("skills", "Skills"),
    ("schedules", "Schedules"),
    ("activity", "Activity"),
    ("usage", "Usage"),
    ("permissions", "Permissions"),
    ("settings", "Settings"),
];

/// The agent sidebar: workspace, the primary pages, then the Workspace group; `on` is current.
pub(super) fn agent_side(p: &ThPal, act: &Option<Act>, on: &str) -> Div {
    let mut s = div()
        .w(px(232.))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(6.))
                .child(div().w(px(22.)).h(px(22.)).rounded(px(7.)).bg(p.fg))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Kestrel Studio"),
                ),
        );
    for (i, (k, l)) in NAV.iter().enumerate() {
        if i == 7 {
            s = s.child(gh(p, "Workspace"));
        }
        let id: &'static str = k;
        let f = fire(act, id);
        s = s.child(
            div()
                .id(SharedString::from(format!("nav-{k}")))
                .h(px(30.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_color(if *k == on { p.fg } else { p.fg2 })
                .when(*k == on, |d| d.bg(p.sel))
                .child(*l)
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    s
}

fn status_c(p: &ThPal, s: AgStatus) -> gpui::Hsla {
    match s {
        AgStatus::Failed => p.red,
        AgStatus::Running | AgStatus::NeedsYou => p.fg,
        _ => p.mute,
    }
}

screen_el!(
    /// Orchestration: the run's agents as a tree (orchestrator, leads, helpers) with status, task,
    /// model and branch cost, the run summary and the trace. Agents carry `agent-<id>`.
    OrchestrationEl
);

fn orch_card(p: &ThPal, n: &OrchNode, all: &[OrchNode], act: &Option<Act>) -> Div {
    let (c, t) = orch_rollup(all, n.id);
    let id: &'static str = match n.id {
        1 => "agent-1",
        2 => "agent-2",
        6 => "agent-6",
        8 => "agent-8",
        10 => "agent-10",
        _ => "agent",
    };
    let f = fire(act, id);
    div().child(
        div()
            .id(SharedString::from(format!("agent-{}", n.id)))
            .w(px(196.))
            .p(px(10.))
            .rounded(px(12.))
            .border_1()
            .border_color(if n.id == 2 { p.fg } else { p.line })
            .bg(p.card)
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(n.name))
                    .child(
                        div()
                            .text_color(status_c(p, n.status))
                            .child(n.status.label()),
                    ),
            )
            .child(div().text_color(p.fg2).child(n.task))
            .child(div().text_size(px(11.5)).text_color(p.mute).child(format!(
                "{} · {} · {}",
                n.model,
                eur(c),
                dur(t)
            )))
            .on_click(move |_, w, cx| f(w, cx)),
    )
}

impl RenderOnce for OrchestrationEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let all = orch_tree(self.state == ScreenState::Sample);
        let mut body = div().flex().flex_col().gap(px(14.)).p(px(18.));
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 6)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "No run yet",
                    "Ask for something in the Thread; its agents appear here.",
                ))
            }
            ScreenState::Sample => {
                if let Some(root) = all.iter().find(|n| n.parent.is_none()) {
                    body = body.child(
                        div()
                            .flex()
                            .justify_center()
                            .child(orch_card(&p, root, &all, &self.act)),
                    );
                    let mut lead = div().flex().gap(px(10.)).justify_center();
                    for k in orch_children(&all, root.id) {
                        let mut col = div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(orch_card(&p, &k, &all, &self.act));
                        for h in orch_children(&all, k.id) {
                            col = col.child(div().pl(px(14.)).text_color(p.fg2).child(format!(
                                "└ {} · {}",
                                h.name,
                                h.status.label()
                            )));
                        }
                        lead = lead.child(col);
                    }
                    body = body.child(lead);
                }
                let (d, t) = orch_done(&all);
                let calls = orch_trace(true);
                let span = trace_span(&calls, 31_500);
                let mut trace = div()
                    .flex()
                    .flex_col()
                    .border_t_1()
                    .border_color(p.line)
                    .pt(px(8.));
                trace = trace.child(row(
                    &p,
                    format!("Agents done {d}/{t} · Elapsed {}", clock(31)),
                    format!("Trace · {}", dur(span)),
                    false,
                ));
                for c in calls.iter().filter(|c| c.parent.is_none()) {
                    trace = trace.child(row(
                        &p,
                        c.label.to_string(),
                        if c.ms == 0 {
                            "running".to_string()
                        } else {
                            dur(c.ms)
                        },
                        false,
                    ));
                }
                body = body.child(trace);
            }
        }
        let work = if self.state == ScreenState::Sample {
            all.iter().find(|n| n.id == 2).map(|n| {
                let (c, t) = orch_rollup(&all, n.id);
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_color(p.mute)
                            .child(format!("{} · {}", n.role, n.model)),
                    )
                    .child(
                        div()
                            .text_size(px(17.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(n.name),
                    )
                    .child(div().text_color(p.fg2).child(n.task))
                    .child(row(
                        &p,
                        "With its helpers".into(),
                        format!("{} · {}", eur(c), dur(t)),
                        false,
                    ))
                    .child(row(&p, "Tokens".into(), toks(n.tokens), false))
                    .child(gh(&p, "Tasks it handed out"))
                    .children(
                        orch_children(&all, n.id)
                            .into_iter()
                            .map(|k| row(&p, k.name.into(), k.status.label().into(), false)),
                    )
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            agent_side(&p, &self.act, "runs"),
            page(&p, "Runs", body),
            work,
        )
    }
}

screen_el!(
    /// Thread: the conversation with the orchestrator – messages, the folded tool line, task cards
    /// (`task-<id>`), files and the time-away marker – under the Today header with its counts.
    AgentThreadEl
);

impl RenderOnce for AgentThreadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let (run, need) = thread_counts(self.state == ScreenState::Sample);
        let mut head = div().flex().items_center().gap(px(8.)).child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Today"),
        );
        if self.state == ScreenState::Sample {
            head = head
                .child(
                    div()
                        .px(px(10.))
                        .py(px(3.))
                        .rounded(px(999.))
                        .border_1()
                        .border_color(p.line)
                        .child(format!("{run} running")),
                )
                .child(
                    div()
                        .px(px(10.))
                        .py(px(3.))
                        .rounded(px(999.))
                        .border_1()
                        .border_color(p.line)
                        .child(format!("{need} need you")),
                );
        }
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(22.))
            .max_w(px(700.))
            .child(head);
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 7)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "Start with the orchestrator",
                    "Ask for an outcome; it plans, runs and checks in here.",
                ))
            }
            ScreenState::Sample => {
                for it in thread_items(true) {
                    body = body.child(match it {
                        ThreadItem::Me { text, .. } => div().flex().justify_end().child(
                            div()
                                .px(px(13.))
                                .py(px(9.))
                                .rounded(px(16.))
                                .bg(p.sel)
                                .max_w(px(480.))
                                .child(text),
                        ),
                        ThreadItem::Agent { text, .. } => div().text_color(p.fg).child(text),
                        ThreadItem::Tools(c) => div().text_color(p.mute).child(tools_line(&c)),
                        ThreadItem::Task {
                            id,
                            name,
                            step,
                            done,
                            of,
                        } => {
                            let aid: &'static str = if id == 21 { "task-21" } else { "task-22" };
                            let f = fire(&self.act, aid);
                            div().child(
                                div()
                                    .id(SharedString::from(format!("task-{id}")))
                                    .p(px(10.))
                                    .rounded(px(12.))
                                    .border_1()
                                    .border_color(p.line)
                                    .bg(p.card)
                                    .flex()
                                    .flex_col()
                                    .gap(px(3.))
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                                    .child(
                                        div()
                                            .text_color(p.mute)
                                            .child(format!("{step} · {done} of {of} steps")),
                                    )
                                    .on_click(move |_, w, cx| f(w, cx)),
                            )
                        }
                        ThreadItem::Files(fs) => div().text_color(p.fg2).child(fs.join("  ·  ")),
                        ThreadItem::Away { hours, what } => div()
                            .border_t_1()
                            .border_b_1()
                            .border_color(p.line)
                            .py(px(6.))
                            .text_color(p.mute)
                            .child(format!("While you were away · {hours}h — {what}")),
                    });
                }
            }
        }
        frame(
            self.theme,
            self.layout,
            agent_side(&p, &self.act, "thread"),
            page(&p, "Thread", body),
            None,
        )
    }
}
