//! Approvals and checkpoints for GPUI: the inline approval card and checkpoint lanes, drawn from
//! the same `oxyn-core` models as the Dioxus views. Pager, footer, modal, markers and side list
//! are Dioxus-first for now.

use gpui::{
    App, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    approval::{ApprovalCard, CardPhase, Risk},
    checkpoint::{Checkpoint, CheckpointStatus, format_at, lanes},
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn risk_roles(r: Risk) -> (ColorRole, ColorRole) {
    match r {
        Risk::Low => (ColorRole::RiskLowFg, ColorRole::RiskLowBg),
        Risk::Medium => (ColorRole::RiskMediumFg, ColorRole::RiskMediumBg),
        Risk::High => (ColorRole::RiskHighFg, ColorRole::RiskHighBg),
        Risk::Critical => (ColorRole::RiskCriticalFg, ColorRole::RiskCriticalBg),
    }
}

/// `approval-card/inline`: header, risk badge, details, countdown and decision buttons.
#[derive(IntoElement)]
pub struct ApprovalCardEl {
    card: ApprovalCard,
    now: ClockMs,
    theme: &'static Theme,
}

impl ApprovalCardEl {
    /// A card at `now` (dark theme).
    pub fn new(card: ApprovalCard, now: ClockMs) -> Self {
        Self {
            card,
            now,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ApprovalCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let req = self.card.request.clone();
        let phase = self.card.phase(self.now);
        if let CardPhase::Resolved { decision, .. } = &phase {
            let ok = decision.allows();
            return div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(12.))
                .rounded(px(10.))
                .bg(c(t, ColorRole::SurfaceRaised))
                .text_size(px(13.))
                .child(
                    div()
                        .text_color(c(
                            t,
                            if ok {
                                ColorRole::StatusSuccessFg
                            } else {
                                ColorRole::FgSubtle
                            },
                        ))
                        .child(if ok { "✓" } else { "⊘" }),
                )
                .child(
                    div()
                        .text_color(c(t, ColorRole::FgDefault))
                        .child(req.title.clone()),
                )
                .child(
                    div()
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(self.card.resolved_row().unwrap_or_default()),
                );
        }
        let (rf, rb) = risk_roles(req.risk);
        let deny_default = !req.risk.enter_allows();
        let fill = match phase {
            CardPhase::Arming(p) | CardPhase::Holding(p) | CardPhase::Draining(p) => Some(p),
            _ => None,
        };
        let allow_label = if req.risk == Risk::Critical {
            "Hold to allow once"
        } else {
            "Allow once"
        };
        let btn = |label: &str, primary: bool| {
            div()
                .h(px(30.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_size(px(12.5))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .when(primary, |d| {
                    d.bg(c(t, ColorRole::AccentSolid))
                        .text_color(c(t, ColorRole::FgOnAccent))
                })
                .when(!primary, |d| {
                    d.bg(c(t, ColorRole::SurfaceRaised))
                        .text_color(c(t, ColorRole::FgDefault))
                })
                .child(label.to_string())
        };
        div()
            .flex()
            .flex_col()
            .p(px(4.))
            .rounded(px(14.))
            .bg(c(t, ColorRole::SurfaceSunken))
            .when_some(req.label.clone(), |d, l| {
                d.child(
                    div()
                        .px(px(8.))
                        .py(px(4.))
                        .text_size(px(11.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(l),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .p(px(12.))
                    .rounded(px(11.))
                    .border_1()
                    .border_color(c(
                        t,
                        if req.risk >= Risk::High {
                            rf
                        } else {
                            ColorRole::BorderDefault
                        },
                    ))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(c(t, ColorRole::FgDefault))
                                            .child(req.title.clone()),
                                    )
                                    .when_some(req.description.clone(), |d, x| {
                                        d.child(
                                            div()
                                                .text_size(px(12.))
                                                .text_color(c(t, ColorRole::FgMuted))
                                                .child(x),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .h(px(22.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(11.))
                                    .bg(c(t, rb))
                                    .text_color(c(t, rf))
                                    .text_size(px(11.5))
                                    .child(format!("{} {}", req.risk.glyph(), req.risk.label())),
                            ),
                    )
                    .when_some(req.command.clone(), |d, cmd| {
                        d.child(
                            div()
                                .p(px(8.))
                                .rounded(px(8.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .font_family("JetBrains Mono")
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::FgDefault))
                                .child(format!("$ {cmd}")),
                        )
                    })
                    .children(req.details.iter().map(|(k, v)| {
                        div()
                            .flex()
                            .gap(px(12.))
                            .text_size(px(12.5))
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(k.clone()),
                            )
                            .child(
                                div()
                                    .text_color(c(t, ColorRole::FgDefault))
                                    .child(v.clone()),
                            )
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(6.))
                    .pt(px(8.))
                    .pb(px(4.))
                    .when_some(self.card.countdown(self.now), |d, (txt, warn)| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(
                                    t,
                                    if warn {
                                        ColorRole::StatusWarningFg
                                    } else {
                                        ColorRole::FgSubtle
                                    },
                                ))
                                .child(txt),
                        )
                    })
                    .child(div().flex_1())
                    .child(btn("Deny", deny_default))
                    .child(
                        btn(allow_label, !deny_default)
                            .when_some(fill, |d, p| d.opacity(0.5 + 0.5 * p)),
                    ),
            )
    }
}

/// `checkpoint-timeline/lanes`.
#[derive(IntoElement)]
pub struct CheckpointLanesEl {
    checkpoints: Vec<Checkpoint>,
    head: Option<String>,
    theme: &'static Theme,
}

impl CheckpointLanesEl {
    /// Lanes for `checkpoints` with the agent at `head`.
    pub fn new(checkpoints: Vec<Checkpoint>, head: Option<String>) -> Self {
        Self {
            checkpoints,
            head,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for CheckpointLanesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rows = lanes(&self.checkpoints, self.head.as_deref());
        div()
            .flex()
            .flex_col()
            .p(px(8.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .children(rows.into_iter().map(|r| {
                let cp = &self.checkpoints[r.index];
                let dot = match cp.status {
                    CheckpointStatus::Failed => c(t, ColorRole::StatusDangerFg),
                    _ if r.on_path => c(t, ColorRole::FgDefault),
                    _ => c(t, ColorRole::FgSubtle),
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(34.))
                    .when(!r.on_path, |d| d.opacity(0.45))
                    .child(div().w(px(18. * r.lane as f32 + 4.)))
                    .child(
                        div()
                            .size(px(if r.current { 10. } else { 8. }))
                            .rounded(px(5.))
                            .bg(dot),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(cp.title.clone()),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(format_at(cp.at)),
                    )
            }))
    }
}
