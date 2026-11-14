//! GPUI elements for the plan #28 parity components, drawn from the same
//! `oxyn_core::parity` models as the Dioxus views. Elements are `RenderOnce`; buttons report
//! through one [`Act`] callback with a short action id (`"confirm"`, `"cancel"`, …) so a host can
//! route them to the model without one closure per button.

use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

mod approvals;

pub use approvals::{
    AgentApprovalsEl, ApprovalGatesEl, ApprovalPagerEl, ConfirmMorphEl, ConfirmPopoverEl,
    HoldConfirmEl, PermissionRequestEl,
};

/// Button callback: the action id and the window.
pub type Act = Rc<dyn Fn(&'static str, &mut Window, &mut App)>;

pub(crate) fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

pub(crate) fn alpha(mut h: Hsla, a: f32) -> Hsla {
    h.a *= a;
    h
}

/// Visual weight of a kit button.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Weight {
    Quiet,
    Plain,
    Primary,
    Danger,
}

/// A 30 px button that reports `action` through `act`.
pub(crate) fn button(
    t: &'static Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    w: Weight,
    action: &'static str,
    act: Option<Act>,
) -> impl IntoElement {
    let (bg, fg, border) = match w {
        Weight::Quiet => (
            alpha(c(t, ColorRole::BgBase), 0.0),
            c(t, ColorRole::FgMuted),
            None,
        ),
        Weight::Plain => (
            c(t, ColorRole::SurfaceRaised),
            c(t, ColorRole::FgDefault),
            Some(c(t, ColorRole::BorderDefault)),
        ),
        Weight::Primary => (c(t, ColorRole::FgDefault), c(t, ColorRole::BgBase), None),
        Weight::Danger => (
            c(t, ColorRole::StatusDangerFg),
            c(t, ColorRole::FgOnAccent),
            None,
        ),
    };
    div()
        .id(id.into())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .h(px(30.))
        .px(px(12.))
        .rounded(px(8.))
        .bg(bg)
        .text_color(fg)
        .text_size(px(12.5))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .when_some(border, |d, b| d.border_1().border_color(b))
        .hover(move |s| s.opacity(0.9))
        .child(label.into())
        .when_some(act, move |d, f| {
            d.on_click(move |_, win, cx| f(action, win, cx))
        })
}

/// A small pill tag.
pub(crate) fn tag(
    t: &'static Theme,
    text: impl Into<SharedString>,
    role: Option<ColorRole>,
) -> impl IntoElement {
    let fg = c(t, role.unwrap_or(ColorRole::FgMuted));
    div()
        .flex_none()
        .h(px(18.))
        .px(px(6.))
        .rounded(px(999.))
        .border_1()
        .border_color(alpha(fg, 0.45))
        .text_color(fg)
        .text_size(px(10.5))
        .flex()
        .items_center()
        .child(text.into())
}

/// A card surface.
pub(crate) fn card(t: &'static Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(9.))
        .p(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .bg(c(t, ColorRole::SurfaceRaised))
        .text_color(c(t, ColorRole::FgDefault))
        .text_size(px(13.))
}

/// Muted text.
pub(crate) fn muted(t: &'static Theme, s: impl Into<SharedString>) -> impl IntoElement {
    div()
        .text_color(c(t, ColorRole::FgMuted))
        .text_size(px(12.5))
        .child(s.into())
}

/// A coloured dot.
pub(crate) fn dot(col: Hsla) -> impl IntoElement {
    div().flex_none().size(px(7.)).rounded_full().bg(col)
}
