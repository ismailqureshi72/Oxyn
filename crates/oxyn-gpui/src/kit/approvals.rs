//! Confirmations and run approvals for GPUI.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use gpui::{
    App, FontWeight, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    approval::{ApprovalCard, ApprovalQueue},
    kit::{
        code_spans,
        confirm::{
            ConfirmMode, ConfirmPopover, ConfirmTone, HoldConfirm, InlineConfirm, MorphFace,
            PopoverPhase,
        },
        gates::{AgentApprovals, GateStatus, Gates, Outcome, PermRisk, PermissionRequest},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, muted, tag};
#[cfg(oxyn_pro)]
use super::dot;
use crate::ApprovalCardEl;

fn tone_fill(t: &'static Theme, tone: ConfirmTone) -> gpui::Hsla {
    match tone {
        ConfirmTone::Danger => c(t, ColorRole::StatusDangerFg),
        ConfirmTone::Neutral => c(t, ColorRole::FgDefault),
    }
}

/// `confirm-popover`: the trigger, and the card below it while open. Reports `"show"`,
/// `"dismiss"`, `"submit"`. The phrase field shows the typed text; the host edits it.
#[derive(IntoElement)]
pub struct ConfirmPopoverEl {
    model: ConfirmPopover,
    now: ClockMs,
    label: SharedString,
    title: SharedString,
    description: Option<SharedString>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ConfirmPopoverEl {
    /// A popover with a trigger label and a card title.
    pub fn new(
        model: ConfirmPopover,
        now: ClockMs,
        label: impl Into<SharedString>,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            model,
            now,
            label: label.into(),
            title: title.into(),
            description: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// The consequence.
    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ConfirmPopoverEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let phase = m.phase(self.now);
        let tone = tone_fill(t, m.tone);
        let trig: SharedString = if phase == PopoverPhase::Success {
            "✓ Deleted".into()
        } else {
            self.label.clone()
        };
        let act = self.act;
        let mut root = div().flex().flex_col().gap(px(8.)).child(
            div()
                .id("oxp-cp-trig")
                .flex()
                .items_center()
                .h(px(34.))
                .px(px(14.))
                .rounded(px(9.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .bg(c(t, ColorRole::SurfaceRaised))
                .text_color(if phase == PopoverPhase::Success {
                    c(t, ColorRole::StatusSuccessFg)
                } else {
                    tone
                })
                .text_size(px(13.))
                .cursor_pointer()
                .child(trig)
                .when_some(act.clone(), |d, f| {
                    d.on_click(move |_, w, cx| f("show", w, cx))
                }),
        );
        if m.open(self.now) {
            let fill = m.match_fill();
            let (bar, lb): (f32, SharedString) = match phase {
                PopoverPhase::Working(Some(p)) => (p, format!("Deleting {:.0}%", p * 100.0).into()),
                PopoverPhase::Working(None) => (m.sweep(self.now), "Deleting".into()),
                PopoverPhase::Error => (0.0, "Try again".into()),
                _ => (fill, "Delete".into()),
            };
            let err = phase == PopoverPhase::Error;
            root = root.child(
                card(t)
                    .w(px(300.))
                    .bg(c(t, ColorRole::SurfaceOverlay))
                    .ml(px(m.shake(self.now)))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if err {
                                c(t, ColorRole::StatusDangerFg)
                            } else {
                                c(t, ColorRole::FgDefault)
                            })
                            .child(if err {
                                SharedString::from("That didn't work")
                            } else {
                                self.title
                            }),
                    )
                    .when_some(
                        if err {
                            m.error.clone().map(SharedString::from)
                        } else {
                            self.description
                        },
                        |d, s| d.child(muted(t, s)),
                    )
                    .when_some(
                        m.confirm_text
                            .clone()
                            .filter(|_| phase == PopoverPhase::Confirming),
                        |d, p| {
                            d.child(muted(t, format!("Type {p} to confirm"))).child(
                                div()
                                    .h(px(30.))
                                    .px(px(9.))
                                    .rounded(px(7.))
                                    .border_1()
                                    .border_color(c(t, ColorRole::BorderDefault))
                                    .bg(c(t, ColorRole::SurfaceBase))
                                    .flex()
                                    .items_center()
                                    .child(SharedString::from(m.typed.clone())),
                            )
                        },
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(button(
                                t,
                                "oxp-cp-no",
                                "Cancel",
                                Weight::Plain,
                                "dismiss",
                                act.clone(),
                            ))
                            .child(
                                div()
                                    .id("oxp-cp-go")
                                    .relative()
                                    .overflow_hidden()
                                    .h(px(30.))
                                    .min_w(px(96.))
                                    .px(px(12.))
                                    .rounded(px(8.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(alpha(tone, 0.35))
                                    .text_color(c(t, ColorRole::FgOnAccent))
                                    .text_size(px(12.5))
                                    .cursor_pointer()
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(0.))
                                            .top(px(0.))
                                            .bottom(px(0.))
                                            .w(relative(bar))
                                            .bg(tone),
                                    )
                                    .child(div().relative().child(lb))
                                    .when_some(act, |d, f| {
                                        d.on_click(move |_, w, cx| f("submit", w, cx))
                                    }),
                            ),
                    ),
            );
        }
        root
    }
}

/// `approval-pager` (GPUI gap closed): the open request of a queue as an inline card with a
/// "1 of 3" pager and the batch row.
#[derive(IntoElement)]
pub struct ApprovalPagerEl {
    queue: ApprovalQueue,
    now: ClockMs,
    page: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ApprovalPagerEl {
    /// The queue on `page`.
    pub fn new(queue: ApprovalQueue, now: ClockMs, page: usize) -> Self {
        Self {
            queue,
            now,
            page,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Reports `"prev"`, `"next"`, `"allow-all"`, `"deny-all"`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ApprovalPagerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let open = self.queue.open();
        let page = self.page.min(open.len().saturating_sub(1));
        let act = self.act;
        let mut root = div().flex().flex_col().gap(px(8.)).w_full().max_w(px(520.));
        if let Some(req) = open.get(page) {
            root = root
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(button(
                            t,
                            "pg-prev",
                            "‹",
                            Weight::Quiet,
                            "prev",
                            act.clone(),
                        ))
                        .child(muted(t, self.queue.pager(page).unwrap_or_default()))
                        .child(button(
                            t,
                            "pg-next",
                            "›",
                            Weight::Quiet,
                            "next",
                            act.clone(),
                        )),
                )
                .child(ApprovalCardEl::new(
                    ApprovalCard::new((*req).clone()),
                    self.now,
                ));
        }
        if let Some(b) = self.queue.batch() {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(muted(t, b))
                    .child(div().flex_1())
                    .child(button(
                        t,
                        "pg-deny",
                        "Deny all",
                        Weight::Quiet,
                        "deny-all",
                        act.clone(),
                    ))
                    .child(button(
                        t,
                        "pg-all",
                        "Allow all",
                        Weight::Plain,
                        "allow-all",
                        act,
                    )),
            );
        }
        root
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
