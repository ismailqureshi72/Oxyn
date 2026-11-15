//! Confirmations and run approvals for GPUI.

use gpui::{
    App, FontWeight, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    approval::{ApprovalCard, ApprovalQueue},
    parity::{
        code_spans,
        confirm::{
            ConfirmMode, ConfirmMorph, ConfirmPopover, ConfirmTone, HoldConfirm, MorphFace,
            PopoverPhase,
        },
        gates::{AgentApprovals, GateStatus, Gates, Outcome, PermRisk, PermissionRequest},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, dot, muted, tag};
use crate::ApprovalCardEl;

fn tone_fill(t: &'static Theme, tone: ConfirmTone) -> gpui::Hsla {
    match tone {
        ConfirmTone::Danger => c(t, ColorRole::StatusDangerFg),
        ConfirmTone::Neutral => c(t, ColorRole::FgDefault),
    }
}

/// `hold-to-confirm`: the fill tracks the hold; reports `"down"` and `"up"`.
#[derive(IntoElement)]
pub struct HoldConfirmEl {
    model: HoldConfirm,
    now: ClockMs,
    label: SharedString,
    act: Option<Act>,
    theme: &'static Theme,
}

impl HoldConfirmEl {
    /// A hold button at `now`.
    pub fn new(model: HoldConfirm, now: ClockMs, label: impl Into<SharedString>) -> Self {
        Self {
            model,
            now,
            label: label.into(),
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Pointer down / up reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for HoldConfirmEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fill = self.model.fill(self.now);
        let done = self.model.confirmed;
        let tone = tone_fill(t, self.model.tone);
        let label: SharedString = if done { "Done".into() } else { self.label };
        let (a1, a2) = (self.act.clone(), self.act);
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .child(
                div()
                    .id("oxp-hold")
                    .relative()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(34.))
                    .px(px(14.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(if done {
                        alpha(tone, 0.6)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .text_color(if done {
                        tone
                    } else {
                        c(t, ColorRole::FgDefault)
                    })
                    .text_size(px(13.))
                    .cursor_pointer()
                    .child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .top(px(0.))
                            .bottom(px(0.))
                            .w(relative(fill))
                            .bg(alpha(tone, 0.22)),
                    )
                    .child(div().relative().child(if done { "✓" } else { "⌫" }))
                    .child(div().relative().child(label))
                    .when_some(a1, |d, f| {
                        d.on_mouse_down(MouseButton::Left, move |_, w, cx| f("down", w, cx))
                    })
                    .when_some(a2, |d, f| {
                        d.on_mouse_up(MouseButton::Left, move |_, w, cx| f("up", w, cx))
                    }),
            )
            .child(muted(t, self.model.hint(self.now)))
    }
}

/// `confirm-morph`: reports `"open"`, `"confirm"`, `"cancel"`, `"undo"`, `"down"`, `"up"`.
#[derive(IntoElement)]
pub struct ConfirmMorphEl {
    model: ConfirmMorph,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ConfirmMorphEl {
    /// A morph at `now`.
    pub fn new(model: ConfirmMorph, now: ClockMs) -> Self {
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
}

impl RenderOnce for ConfirmMorphEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let face = m.face(self.now);
        let l = m.labels.clone();
        let text = m.text(self.now);
        let danger = if m.tone == ConfirmTone::Danger {
            Weight::Danger
        } else {
            Weight::Primary
        };
        let act = self.act;
        let row = div()
            .flex()
            .items_center()
            .gap(px(6.))
            .min_h(px(34.))
            .p(px(2.))
            .rounded(px(10.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault));
        if face == MorphFace::Idle {
            return row.child(button(
                t,
                "oxp-cm-rest",
                format!("⌫  {text}"),
                Weight::Plain,
                "open",
                act,
            ));
        }
        let row = row
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised));
        let lead = match face {
            MorphFace::Pending | MorphFace::Undoing => Some(("◌", ColorRole::FgMuted)),
            MorphFace::Done => Some(("✓", ColorRole::StatusSuccessFg)),
            MorphFace::Error => Some(("!", ColorRole::StatusDangerFg)),
            MorphFace::Counting(_) => Some(("◷", ColorRole::StatusDangerFg)),
            _ => None,
        };
        let row = row
            .when_some(lead, |d, (g, r)| {
                d.child(div().pl(px(8.)).text_color(c(t, r)).child(g))
            })
            .child(div().px(px(6.)).child(text));
        match face {
            MorphFace::Confirming => {
                let hold = m.mode == ConfirmMode::Hold;
                let fill = m.hold.fill(self.now);
                row.child(button(
                    t,
                    "oxp-cm-no",
                    l.cancel,
                    Weight::Quiet,
                    "cancel",
                    act.clone(),
                ))
                .when(!hold, |d| {
                    d.child(button(
                        t,
                        "oxp-cm-yes",
                        l.confirm.clone(),
                        danger,
                        "confirm",
                        act.clone(),
                    ))
                })
                .when(hold, |d| {
                    let (a1, a2) = (act.clone(), act.clone());
                    d.child(
                        div()
                            .id("oxp-cm-hold")
                            .relative()
                            .overflow_hidden()
                            .h(px(28.))
                            .px(px(11.))
                            .rounded(px(7.))
                            .flex()
                            .items_center()
                            .bg(alpha(c(t, ColorRole::StatusDangerFg), 0.3))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.))
                                    .top(px(0.))
                                    .bottom(px(0.))
                                    .w(relative(fill))
                                    .bg(c(t, ColorRole::StatusDangerFg)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .child(format!("Hold to {}", l.confirm.to_lowercase())),
                            )
                            .when_some(a1, |d, f| {
                                d.on_mouse_down(MouseButton::Left, move |_, w, cx| f("down", w, cx))
                            })
                            .when_some(a2, |d, f| {
                                d.on_mouse_up(MouseButton::Left, move |_, w, cx| f("up", w, cx))
                            }),
                    )
                })
            }
            MorphFace::Counting(_) => row.child(button(
                t,
                "oxp-cm-no",
                l.cancel,
                Weight::Quiet,
                "cancel",
                act,
            )),
            MorphFace::Done if m.undoable => {
                row.child(button(t, "oxp-cm-undo", l.undo, Weight::Quiet, "undo", act))
            }
            MorphFace::Error => row
                .child(button(
                    t,
                    "oxp-cm-no",
                    l.cancel,
                    Weight::Quiet,
                    "cancel",
                    act.clone(),
                ))
                .child(button(t, "oxp-cm-retry", l.retry, danger, "confirm", act)),
            _ => row,
        }
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

/// `approval-gates`: the open stack (top card plus peeking edges) and the Decided list.
/// Reports `"approve"`, `"reject"` and `"edit"` for the top gate.
#[derive(IntoElement)]
pub struct ApprovalGatesEl {
    gates: Gates,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ApprovalGatesEl {
    /// The gates of a run.
    pub fn new(gates: Gates) -> Self {
        Self {
            gates,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ApprovalGatesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let g = &self.gates;
        let open = g.open();
        let act = self.act;
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .w_full()
            .max_w(px(520.))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Approvals"))
                    .child(muted(t, g.task.clone()))
                    .child(div().flex_1())
                    .child(muted(t, g.status_line())),
            );
        if let Some(top) = open.first() {
            let peeks = open.len().min(3).saturating_sub(1);
            let mut stack = div().flex().flex_col().child(
                card(t)
                    .child(
                        div()
                            .flex()
                            .gap(px(7.))
                            .items_center()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(top.agent.clone()),
                            )
                            .child(tag(t, top.kind.id(), None))
                            .child(div().flex_1())
                            .when(top.holds > 0, |d| {
                                d.child(tag(
                                    t,
                                    format!("holds {}", top.holds),
                                    Some(ColorRole::StatusWarningFg),
                                ))
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(top.action.clone()),
                    )
                    .child(muted(t, top.reason.clone()))
                    .when_some(top.input.clone(), |d, i| {
                        d.child(
                            div()
                                .p(px(8.))
                                .rounded(px(8.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .text_size(px(11.5))
                                .child(i),
                        )
                    })
                    .children(top.diff.iter().map(|l| {
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
                            .child(SharedString::from(l.clone()))
                    }))
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(8.))
                            .child(button(
                                t,
                                "g-rej",
                                "Reject",
                                Weight::Quiet,
                                "reject",
                                act.clone(),
                            ))
                            .when(top.input.is_some(), |d| {
                                d.child(button(
                                    t,
                                    "g-edit",
                                    "Change",
                                    Weight::Plain,
                                    "edit",
                                    act.clone(),
                                ))
                            })
                            .child(button(
                                t,
                                "g-ok",
                                "Approve",
                                Weight::Primary,
                                "approve",
                                act.clone(),
                            )),
                    ),
            );
            for i in 1..=peeks {
                stack = stack.child(
                    div()
                        .mx(px(10. * i as f32))
                        .h(px(7.))
                        .rounded_b(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .bg(c(t, ColorRole::SurfaceRaised)),
                );
            }
            root = root.child(stack);
        } else {
            root = root.child(muted(
                t,
                if g.live {
                    "Running · new requests appear here"
                } else {
                    "Nothing waiting"
                },
            ));
        }
        let decided = g.decided();
        if !decided.is_empty() {
            root = root
                .child(muted(t, "DECIDED"))
                .children(decided.into_iter().map(|d| {
                    let col = match d.status {
                        GateStatus::Rejected => c(t, ColorRole::StatusDangerFg),
                        _ => c(t, ColorRole::StatusSuccessFg),
                    };
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(28.))
                        .child(dot(col))
                        .child(d.action.clone())
                        .child(div().flex_1())
                        .child(muted(t, d.status.label()))
                }));
        }
        root
    }
}

/// `permission-request`: the request, or the folded line once decided. Reports `"allow"`,
/// `"deny"`, `"revoke"`, `"once"`, `"timed"`, `"always"`.
#[derive(IntoElement)]
pub struct PermissionRequestEl {
    req: PermissionRequest,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl PermissionRequestEl {
    /// A request at `now`.
    pub fn new(req: PermissionRequest, now: ClockMs) -> Self {
        Self {
            req,
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
}

impl RenderOnce for PermissionRequestEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = &self.req;
        let act = self.act;
        if let Some(line) = r.folded(self.now) {
            return card(t)
                .flex_row()
                .items_center()
                .py(px(8.))
                .child(div().child(format!("{} · {} · {}", r.agent, r.action, r.target)))
                .child(div().flex_1())
                .child(muted(t, line))
                .child(button(t, "pr-rev", "Revoke", Weight::Quiet, "revoke", act));
        }
        let (rows, more) = r.visible_preview();
        let allow_w = if r.risk == PermRisk::Destructive {
            Weight::Danger
        } else {
            Weight::Primary
        };
        card(t)
            .max_w(px(440.))
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(r.agent.clone()),
                    )
                    .child(muted(t, "wants to"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(r.action.clone()),
                    )
                    .child(div().flex_1())
                    .when(r.risk != PermRisk::Safe, |d| {
                        d.child(tag(
                            t,
                            r.risk.id(),
                            Some(if r.risk == PermRisk::Destructive {
                                ColorRole::StatusDangerFg
                            } else {
                                ColorRole::StatusWarningFg
                            }),
                        ))
                    }),
            )
            .child(muted(t, r.target.clone()))
            .when(!r.reason.is_empty(), |d| {
                d.child(muted(t, format!("“{}”", r.reason)))
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .p(px(8.))
                    .rounded(px(9.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .child(muted(t, "WHAT WILL HAPPEN"))
                    .children(rows.iter().map(|x| {
                        div()
                            .flex()
                            .child(x.label.clone())
                            .child(div().flex_1())
                            .child(muted(t, x.meta.clone().unwrap_or_default()))
                    }))
                    .when(more > 0, |d| d.child(muted(t, format!("{more} more")))),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .items_center()
                    .children(r.offered().into_iter().map(|s| {
                        let w = if s == r.scope {
                            Weight::Plain
                        } else {
                            Weight::Quiet
                        };
                        button(t, s.id(), s.label(r.duration_min), w, s.id(), act.clone())
                    }))
                    .child(div().flex_1())
                    .child(button(
                        t,
                        "pr-deny",
                        "Deny",
                        Weight::Plain,
                        "deny",
                        act.clone(),
                    ))
                    .child(button(
                        t,
                        "pr-allow",
                        "Allow",
                        allow_w,
                        "allow",
                        act.clone(),
                    )),
            )
    }
}

/// `agent-approvals`: the top tool call with its arguments, the queue count, rules and log.
/// Reports `"approve"`, `"deny"`, `"always-agent"`, `"always-workspace"`.
#[derive(IntoElement)]
pub struct AgentApprovalsEl {
    model: AgentApprovals,
    act: Option<Act>,
    theme: &'static Theme,
}

impl AgentApprovalsEl {
    /// The block.
    pub fn new(model: AgentApprovals) -> Self {
        Self {
            model,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for AgentApprovalsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let act = self.act;
        let mut root = card(t).max_w(px(520.)).child(
            div()
                .flex()
                .gap(px(8.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Approvals"))
                .child(div().flex_1())
                .child(muted(
                    t,
                    format!("{} waiting · {} rules", m.queue.len(), m.rules.len()),
                )),
        );
        if let Some(a) = m.current() {
            root = root
                .child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(a.agent.clone()),
                        )
                        .child(muted(t, a.model.clone()))
                        .child(div().flex_1())
                        .child(tag(t, format!("{} risk", a.risk), None)),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .children(code_spans(&a.title).into_iter().map(|(s, code)| {
                            div()
                                .when(code, |d| {
                                    d.px(px(4.))
                                        .rounded(px(5.))
                                        .bg(c(t, ColorRole::SurfaceSunken))
                                })
                                .child(s)
                        })),
                )
                .child(muted(t, a.reason.clone()))
                .children(a.args.iter().map(|x| {
                    div()
                        .flex()
                        .gap(px(8.))
                        .h(px(28.))
                        .items_center()
                        .child(div().w(px(110.)).child(muted(t, x.key.clone())))
                        .child(div().text_size(px(11.5)).child(x.value.clone()))
                }))
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .justify_end()
                        .child(button(
                            t,
                            "aa-no",
                            "Deny",
                            Weight::Plain,
                            "deny",
                            act.clone(),
                        ))
                        .child(button(
                            t,
                            "aa-always",
                            "Always allow",
                            Weight::Plain,
                            "always-agent",
                            act.clone(),
                        ))
                        .child(button(
                            t,
                            "aa-ok",
                            "Approve",
                            Weight::Primary,
                            "approve",
                            act,
                        )),
                );
        } else {
            root = root.child(muted(t, "All caught up"));
        }
        root.children(m.log.iter().take(4).map(|e| {
            let col = match e.outcome {
                Outcome::Denied => c(t, ColorRole::StatusDangerFg),
                Outcome::Allowed => c(t, ColorRole::StatusInfoFg),
                Outcome::Approved => c(t, ColorRole::StatusSuccessFg),
            };
            div()
                .flex()
                .gap(px(8.))
                .items_center()
                .child(dot(col))
                .child(e.summary.clone())
                .child(div().flex_1())
                .child(muted(t, format!("{} · {}", e.outcome.label(), e.by)))
        }))
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
