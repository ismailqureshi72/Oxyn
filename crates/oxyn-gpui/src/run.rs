//! Family F for GPUI: tool-call rows, the todo checklist and the sub-agent tree, drawn from the
//! same `oxyn-core::run` models as the Dioxus views. Row and tree variants only; detail panes,
//! cards and docks are Dioxus-first (see the PR notes).

use std::collections::BTreeSet;

use gpui::{
    App, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs, VerbPack,
    run::{ItemStatus, Plan, RowGlyph, RunGraph, Tone, ToolCall, TreeVariant, row_view},
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn tone(t: &Theme, tone: Tone) -> Hsla {
    c(
        t,
        match tone {
            Tone::Muted => ColorRole::FgSubtle,
            Tone::Default => ColorRole::FgDefault,
            Tone::Success => ColorRole::StatusSuccessFg,
            Tone::Danger => ColorRole::StatusDangerFg,
            Tone::Waiting => ColorRole::StatusWaitingFg,
            Tone::Warn => ColorRole::StatusWarningFg,
        },
    )
}

const SPIN: [&str; 8] = ["◜", "◝", "◞", "◟", "◜", "◝", "◞", "◟"];
const AGENT: [ColorRole; 8] = [
    ColorRole::AgentRed,
    ColorRole::AgentOrange,
    ColorRole::AgentYellow,
    ColorRole::AgentGreen,
    ColorRole::AgentBlue,
    ColorRole::AgentViolet,
    ColorRole::AgentPink,
    ColorRole::AgentCyan,
];

/// One tool-call row (`tool-call/row`), collapsed, with its error line.
#[derive(IntoElement)]
pub struct ToolCallRowEl {
    call: ToolCall,
    now: ClockMs,
    theme: &'static Theme,
}

impl ToolCallRowEl {
    /// A row for `call` at `now` (dark theme).
    pub fn new(call: ToolCall, now: ClockMs) -> Self {
        Self {
            call,
            now,
            theme: &Theme::DARK,
        }
    }

    /// Use another theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for ToolCallRowEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let v = row_view(&self.call, VerbPack::neutral(), self.now);
        let g = match v.glyph {
            RowGlyph::Loader => SPIN[(self.now.0 / 100) as usize % SPIN.len()],
            RowGlyph::Kind(_) => "·",
            x => x.text(),
        };
        let mono = "JetBrains Mono";
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .text_size(px(13.))
                    .child(
                        div()
                            .w(px(16.))
                            .ml(px(v.nudge_px))
                            .text_color(tone(t, v.tone))
                            .child(g),
                    )
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(v.verb.clone()),
                    )
                    .when_some(v.target.clone(), |d, tg| {
                        d.child(
                            div()
                                .font_family(mono)
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(tg),
                        )
                    })
                    .when_some(v.preview.clone(), |d, p| {
                        d.child(
                            div()
                                .font_family(mono)
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(p),
                        )
                    })
                    .when_some(v.note.clone(), |d, n| {
                        d.child(div().text_color(c(t, ColorRole::FgSubtle)).child(n))
                    })
                    .child(div().flex_1())
                    .when_some(v.stalled.clone(), |d, s| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::StatusWarningFg))
                                .child(s),
                        )
                    })
                    .when_some(v.duration.clone(), |d, du| {
                        d.child(
                            div()
                                .font_family(mono)
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(du),
                        )
                    }),
            )
            .when_some(v.error_line.clone(), |d, e| {
                d.child(
                    div()
                        .ml(px(32.))
                        .mr(px(8.))
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .bg(c(t, ColorRole::StatusDangerBg))
                        .text_color(c(t, ColorRole::StatusDangerFg))
                        .font_family(mono)
                        .text_size(px(11.5))
                        .child(e),
                )
            })
    }
}

/// `todo-list/checklist`.
#[derive(IntoElement)]
pub struct TodoChecklist {
    plan: Plan,
    now: ClockMs,
    theme: &'static Theme,
}

impl TodoChecklist {
    /// A checklist for `plan`.
    pub fn new(plan: Plan, now: ClockMs) -> Self {
        Self {
            plan,
            now,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for TodoChecklist {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.plan.progress();
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(13.))
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(self.plan.title.clone().unwrap_or_else(|| "Tasks".into())),
                    )
                    .child(div().text_color(c(t, ColorRole::FgMuted)).child(p.label())),
            )
            .children(self.plan.items.iter().map(|i| {
                let col = match i.status {
                    ItemStatus::Done => c(t, ColorRole::StatusSuccessFg),
                    ItemStatus::Failed { .. } => c(t, ColorRole::StatusDangerFg),
                    ItemStatus::Blocked { .. } => c(t, ColorRole::StatusWaitingFg),
                    _ => c(t, ColorRole::FgSubtle),
                };
                let g = if i.status == ItemStatus::Active {
                    SPIN[(self.now.0 / 100) as usize % SPIN.len()]
                } else {
                    i.status.glyph()
                };
                let label_c = if matches!(
                    i.status,
                    ItemStatus::Active | ItemStatus::Pending | ItemStatus::Blocked { .. }
                ) {
                    c(t, ColorRole::FgDefault)
                } else {
                    c(t, ColorRole::FgMuted)
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(26.))
                    .text_size(px(13.))
                    .child(div().w(px(16.)).text_color(col).child(g))
                    .child(div().text_color(label_c).child(i.label().to_string()))
                    .child(div().flex_1())
                    .when_some(i.duration(self.now), |d, du| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(du),
                        )
                    })
            }))
    }
}

/// `subagent-tree/indented` or `rail` (other variants are Dioxus-first).
#[derive(IntoElement)]
pub struct SubagentTreeEl {
    graph: RunGraph,
    now: ClockMs,
    variant: TreeVariant,
    collapsed: BTreeSet<String>,
    theme: &'static Theme,
}

impl SubagentTreeEl {
    /// A tree of `graph`.
    pub fn new(graph: RunGraph, now: ClockMs, variant: TreeVariant) -> Self {
        Self {
            graph,
            now,
            variant,
            collapsed: BTreeSet::new(),
            theme: &Theme::DARK,
        }
    }

    /// Collapsed ids.
    pub fn collapsed(mut self, ids: BTreeSet<String>) -> Self {
        self.collapsed = ids;
        self
    }
}

impl RenderOnce for SubagentTreeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let v = self.graph.view(
            self.variant,
            &self.collapsed,
            3,
            8,
            VerbPack::neutral(),
            self.now,
        );
        let rail = self.variant == TreeVariant::Rail;
        div()
            .flex()
            .flex_col()
            .p(px(6.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .child(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(v.header.clone()),
            )
            .children(v.rows.into_iter().map(|r| {
                let line = r.truncated.clone().unwrap_or_else(|| r.state.line());
                let g = if r.state.is_live() && r.animate {
                    SPIN[(self.now.0 / 100) as usize % 8]
                } else {
                    r.state.glyph()
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .pl(px(8. + if rail { 0. } else { r.depth as f32 * 18. }))
                    .pr(px(8.))
                    .text_size(px(13.))
                    .when(rail, |d| {
                        d.child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_color(c(t, ColorRole::BorderStrong))
                                .child(r.rail.clone()),
                        )
                    })
                    .child(
                        div()
                            .w(px(14.))
                            .text_color(tone(t, r.state.tone()))
                            .child(g),
                    )
                    .child(
                        div()
                            .size(px(8.))
                            .rounded(px(2.))
                            .bg(c(t, AGENT[r.colour as usize % 8])),
                    )
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(r.name.clone()),
                    )
                    .child(
                        div()
                            .text_color(match r.state.tone() {
                                Tone::Danger | Tone::Waiting => tone(t, r.state.tone()),
                                _ => c(t, ColorRole::FgMuted),
                            })
                            .child(line),
                    )
                    .child(div().flex_1())
                    .when_some(r.elapsed.clone(), |d, e| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(e),
                        )
                    })
            }))
    }
}
