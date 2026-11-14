//! Surfaces, command and search for GPUI (plan #26), drawn from the same
//! `oxyn_core::primitives::{modal, toast, scroll, feedback, skeleton, command, search,
//! control_center}` models as the Dioxus components. Elements are `RenderOnce`: the host owns the
//! model, routes pointer and key events to its methods, feeds the shared clock and re-renders.
//!
//! Gaps against Dioxus (listed in the PR): overlays draw in a layer the host places (no deferred
//! window-level portal, focus trap or restore yet: the host moves focus); drags and the dial take
//! positions from the host's mouse handlers; text entry is display-only until the shared input
//! handler lands; no backdrop blur, CSS masks or conic beam (the working edge pulses instead);
//! the settings command is Dioxus-only for now.

use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    primitives::{
        command::{ActionState, AiCommand, AiMode, AiSearch, AnswerPhase, Palette, Seg, segments},
        control_center::{ControlCenter, Detail, Meter},
        display::Platform,
        feedback::{Alert, AlertTone, Card, CardEdge},
        modal::{
            BottomSheet, Dialog, DialogSize, EdgeDrag, Phase, SheetSide, SheetStack, StackMode,
        },
        scroll::{Metrics, PageSlot, Pagination, StickToBottom, fades, thumb},
        search::{ExpandingSearch, RecState, Recorder, caps, clear_visible},
        skeleton::{Morph, Shape, Sweep, sheen_phase, timing as sk},
        toast::{NotificationStack, ToastKind, Toaster},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;
type OnId = Rc<dyn Fn(u64, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn alpha(mut h: Hsla, a: f32) -> Hsla {
    h.a *= a;
    h
}

macro_rules! themed {
    () => {
        /// Theme.
        pub fn theme(mut self, t: &'static Theme) -> Self {
            self.theme = t;
            self
        }
    };
}

fn panel(t: &'static Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .bg(c(t, ColorRole::SurfaceOverlay))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .text_color(c(t, ColorRole::FgDefault))
        .text_size(px(13.))
        .shadow_lg()
}

fn close_x(
    t: &'static Theme,
    id: &'static str,
    label: &'static str,
    f: Option<OnClick>,
) -> impl IntoElement {
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(SharedString::from(label))
        .flex()
        .items_center()
        .justify_center()
        .size(px(26.))
        .rounded(px(7.))
        .text_color(c(t, ColorRole::FgMuted))
        .child("×")
        .when_some(f, |d, f| {
            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
        })
}

fn heading(t: &'static Theme, title: SharedString, desc: Option<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(px(4.))
        .child(
            div()
                .text_size(px(15.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .when_some(desc, |d, s| {
            d.child(div().text_color(c(t, ColorRole::FgMuted)).child(s))
        })
}

fn scrim(t: &'static Theme, a: f32, f: Option<OnClick>) -> impl IntoElement {
    div()
        .id("oxyn-scrim")
        .absolute()
        .inset_0()
        .bg(alpha(c(t, ColorRole::Scrim), a))
        .when_some(f, |d, f| d.on_click(move |_, w, cx| f(w, cx)))
}

// ---------------------------------------------------------------------------------------------
// Dialog

/// `dialog`: modal window over a scrim; enter rises 8 px from 0.96, exit is quicker.
#[derive(IntoElement)]
pub struct DialogEl {
    model: Dialog,
    title: SharedString,
    description: Option<SharedString>,
    confirm: Option<SharedString>,
    danger: bool,
    now: ClockMs,
    reduced: bool,
    content: Vec<AnyElement>,
    on_close: Option<OnClick>,
    on_confirm: Option<OnClick>,
    theme: &'static Theme,
}

impl DialogEl {
    /// From the model.
    pub fn new(model: Dialog, title: impl Into<SharedString>, now: ClockMs) -> Self {
        Self {
            model,
            title: title.into(),
            description: None,
            confirm: None,
            danger: false,
            now,
            reduced: false,
            content: vec![],
            on_close: None,
            on_confirm: None,
            theme: &Theme::DARK,
        }
    }
    /// Quiet line.
    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Footer confirm (adds Cancel).
    pub fn confirm(mut self, label: impl Into<SharedString>, danger: bool) -> Self {
        self.confirm = Some(label.into());
        self.danger = danger;
        self
    }
    /// Reduced motion.
    pub fn reduced(mut self, r: bool) -> Self {
        self.reduced = r;
        self
    }
    /// Close requested (× / scrim / Cancel): the host calls `request_close`.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
    /// Confirm pressed.
    pub fn on_confirm(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_confirm = Some(Rc::new(f));
        self
    }
    themed!();
}

impl ParentElement for DialogEl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements)
    }
}

impl RenderOnce for DialogEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ph = self.model.phase(self.now, self.reduced);
        if ph == Phase::Closed {
            return div().into_any_element();
        }
        let k = match ph {
            Phase::Opening => 0.6,
            Phase::Closing => 0.3,
            _ => 1.0,
        };
        let w = self.model.size.width().unwrap_or(720.0);
        let close = self.on_close.clone();
        let busy = self.model.busy;
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .child(scrim(
                t,
                k,
                close.clone().filter(|_| self.model.close_on_backdrop),
            ))
            .child(
                panel(t)
                    .id("oxyn-dialog")
                    .role(Role::Dialog)
                    .aria_label(self.title.clone())
                    .relative()
                    .w(px(w))
                    .max_w(px(w))
                    .rounded(px(14.))
                    .opacity(k)
                    .mt(px(if self.reduced { 0.0 } else { (1.0 - k) * 8.0 }))
                    .child(
                        div()
                            .flex()
                            .gap(px(12.))
                            .pt(px(16.))
                            .px(px(20.))
                            .pb(px(10.))
                            .child(heading(t, self.title.clone(), self.description.clone()))
                            .child(close_x(t, "oxyn-dlg-x", "Close", close.clone())),
                    )
                    .child(div().px(px(20.)).pb(px(16.)).children(self.content))
                    .when_some(self.confirm.clone(), |d, label| {
                        let ok = self.on_confirm.clone();
                        d.child(
                            div()
                                .flex()
                                .justify_end()
                                .gap(px(8.))
                                .p(px(12.))
                                .border_t_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .child(
                                    div()
                                        .id("dlg-cancel")
                                        .role(Role::Button)
                                        .px(px(10.))
                                        .py(px(5.))
                                        .rounded(px(7.))
                                        .child("Cancel")
                                        .when_some(close.clone(), |d, f| {
                                            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                                        }),
                                )
                                .child(
                                    div()
                                        .id("dlg-ok")
                                        .role(Role::Button)
                                        .px(px(10.))
                                        .py(px(5.))
                                        .rounded(px(7.))
                                        .bg(if self.danger {
                                            c(t, ColorRole::StatusDangerFg)
                                        } else {
                                            c(t, ColorRole::AccentSolid)
                                        })
                                        .text_color(c(t, ColorRole::FgOnAccent))
                                        .child(if busy {
                                            SharedString::from("Working…")
                                        } else {
                                            label
                                        })
                                        .when_some(ok.filter(|_| !busy), |d, f| {
                                            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                                        }),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }
}

// ---------------------------------------------------------------------------------------------
// Sheet

/// `sheet`: the dialog docked to an edge; follows a drag toward the edge.
#[derive(IntoElement)]
pub struct SheetEl {
    title: SharedString,
    side: SheetSide,
    size: f32,
    drag: EdgeDrag,
    content: Vec<AnyElement>,
    on_close: Option<OnClick>,
    theme: &'static Theme,
}

impl SheetEl {
    /// A sheet.
    pub fn new(title: impl Into<SharedString>, side: SheetSide, size: f32) -> Self {
        Self {
            title: title.into(),
            side,
            size,
            drag: EdgeDrag::default(),
            content: vec![],
            on_close: None,
            theme: &Theme::DARK,
        }
    }
    /// Current drag.
    pub fn drag(mut self, d: EdgeDrag) -> Self {
        self.drag = d;
        self
    }
    /// Close requested.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
    themed!();
}

impl ParentElement for SheetEl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements)
    }
}

impl RenderOnce for SheetEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let off = self.drag.offset.max(0.0);
        let dim = 1.0 - (off / self.size).clamp(0.0, 1.0);
        let s = panel(t)
            .id("oxyn-sheet")
            .role(Role::Dialog)
            .aria_label(self.title.clone())
            .absolute();
        let s = match self.side {
            SheetSide::Right => s.top_0().bottom_0().right(px(-off)).w(px(self.size)),
            SheetSide::Left => s.top_0().bottom_0().left(px(-off)).w(px(self.size)),
            SheetSide::Top => s.left_0().right_0().top(px(-off)).h(px(self.size)),
            SheetSide::Bottom => s.left_0().right_0().bottom(px(-off)).h(px(self.size)),
        };
        div()
            .absolute()
            .inset_0()
            .child(scrim(t, dim, self.on_close.clone()))
            .child(
                s.child(
                    div()
                        .flex()
                        .gap(px(12.))
                        .pt(px(18.))
                        .px(px(20.))
                        .pb(px(10.))
                        .child(heading(t, self.title, None))
                        .child(close_x(t, "oxyn-sheet-x", "Close", self.on_close)),
                )
                .child(
                    div()
                        .px(px(20.))
                        .pb(px(16.))
                        .flex_1()
                        .children(self.content),
                ),
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Sheet stack

/// `sheet-stack`: open sheets with depth (scale back 5 %, dim, peek); back names the one below.
#[derive(IntoElement)]
pub struct SheetStackEl {
    model: SheetStack,
    width: f32,
    bodies: Vec<(String, Vec<(SharedString, SharedString)>)>,
    on_back: Option<OnClick>,
    on_close: Option<OnClick>,
    on_row: Option<OnIndex>,
    theme: &'static Theme,
}

impl SheetStackEl {
    /// From the model and the layer width.
    pub fn new(model: SheetStack, width: f32) -> Self {
        Self {
            model,
            width,
            bodies: vec![],
            on_back: None,
            on_close: None,
            on_row: None,
            theme: &Theme::DARK,
        }
    }
    /// Rows (label, detail) for a sheet id.
    pub fn rows(mut self, id: &str, rows: Vec<(SharedString, SharedString)>) -> Self {
        self.bodies.push((id.into(), rows));
        self
    }
    /// Back.
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }
    /// Close all.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
    /// A row of the top sheet pressed.
    pub fn on_row(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_row = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for SheetStackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let mode = m.mode.resolve(self.width, m.breakpoint);
        let open = m.open();
        let n = open.len();
        let follow = if m.drag.active {
            (m.drag.offset / 300.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let back = m.back_label();
        let close_label = m.close_label();
        let mut layer = div().absolute().inset_0();
        if n > 0 {
            layer = layer.child(scrim(t, 1.0, self.on_close.clone()));
        }
        for (i, sh) in open.iter().enumerate() {
            let depth = n - 1 - i;
            let lv = m.level(depth, mode, follow);
            let top = depth == 0;
            let w = (self.width * lv.scale).min(if mode == StackMode::Dialog {
                440.0
            } else {
                self.width
            });
            let rows = self
                .bodies
                .iter()
                .find(|b| b.0 == sh.id)
                .map(|b| b.1.clone())
                .unwrap_or_default();
            let row_cb = self.on_row.clone();
            let card = panel(t)
                .id(SharedString::from(format!("stack-{}", sh.id)))
                .role(Role::Dialog)
                .aria_label(SharedString::from(sh.title.clone()))
                .absolute()
                .w(px(w))
                .left(px((self.width - w) / 2.0))
                .bottom(px(lv.lift + if top { -m.drag.offset.max(0.0) } else { 0.0 }))
                .min_h(px(240.))
                .rounded(px(16.))
                .p(px(16.))
                .gap(px(6.))
                .opacity(1.0 - lv.dim)
                .when(top, |d| {
                    d.child(
                        div()
                            .flex()
                            .justify_between()
                            .when_some(back.clone(), |d, b| {
                                let f = self.on_back.clone();
                                d.child(
                                    div()
                                        .id("stack-back")
                                        .role(Role::Button)
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child(format!("‹ {b}"))
                                        .when_some(f, |d, f| {
                                            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                                        }),
                                )
                            })
                            .child(close_x(t, "stack-x", close_label, self.on_close.clone())),
                    )
                })
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(sh.title.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceSunken))
                        .children(rows.into_iter().enumerate().map(move |(ri, (l, dsc))| {
                            let f = row_cb.clone();
                            div()
                                .id(("stack-row", ri))
                                .role(Role::Button)
                                .flex()
                                .justify_between()
                                .px(px(12.))
                                .py(px(10.))
                                .child(l)
                                .child(div().text_color(c(t, ColorRole::FgMuted)).child(dsc))
                                .when_some(f.filter(|_| top), |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(ri, w, cx))
                                })
                        })),
                );
            layer = layer.child(card);
        }
        layer
    }
}

// ---------------------------------------------------------------------------------------------
// Bottom sheet

/// `bottom-sheet`: rests at its detent height; the scrim follows the height.
#[derive(IntoElement)]
pub struct BottomSheetEl {
    model: BottomSheet,
    title: SharedString,
    content: Vec<AnyElement>,
    on_grabber: Option<OnClick>,
    on_close: Option<OnClick>,
    theme: &'static Theme,
}

impl BottomSheetEl {
    /// From the model.
    pub fn new(model: BottomSheet, title: impl Into<SharedString>) -> Self {
        Self {
            model,
            title: title.into(),
            content: vec![],
            on_grabber: None,
            on_close: None,
            theme: &Theme::DARK,
        }
    }
    /// Grabber pressed (the host calls `key(DetentKey::Toggle)`).
    pub fn on_grabber(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_grabber = Some(Rc::new(f));
        self
    }
    /// Close requested.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
    themed!();
}

impl ParentElement for BottomSheetEl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements)
    }
}

impl RenderOnce for BottomSheetEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let g = self.on_grabber.clone();
        div()
            .absolute()
            .inset_0()
            .child(scrim(t, m.dim() / 0.45, self.on_close.clone()))
            .child(
                panel(t)
                    .id("oxyn-bsheet")
                    .role(Role::Dialog)
                    .aria_label(self.title.clone())
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(m.height()))
                    .rounded_t(px(18.))
                    .child(
                        div()
                            .id("bs-grabber")
                            .role(Role::Button)
                            .aria_label(SharedString::from(m.grabber_label()))
                            .flex()
                            .justify_center()
                            .py(px(8.))
                            .child(
                                div()
                                    .w(px(40.))
                                    .h(px(5.))
                                    .rounded_full()
                                    .bg(c(t, ColorRole::BorderStrong)),
                            )
                            .when_some(g, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .px(px(20.))
                            .pb(px(8.))
                            .child(heading(t, self.title, None))
                            .child(close_x(t, "bs-x", "Close", self.on_close)),
                    )
                    .child(div().px(px(20.)).flex_1().children(self.content)),
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Toaster

/// `toaster`: the live toasts, collapsed (peek) or fanned out, newest at the edge.
#[derive(IntoElement)]
pub struct ToasterEl {
    model: Toaster,
    now: ClockMs,
    height: f32,
    on_dismiss: Option<OnId>,
    theme: &'static Theme,
}

impl ToasterEl {
    /// From the model (the host calls `tick(now)` each frame).
    pub fn new(model: Toaster, now: ClockMs) -> Self {
        Self {
            model,
            now,
            height: 62.0,
            on_dismiss: None,
            theme: &Theme::DARK,
        }
    }
    /// A toast's close pressed.
    pub fn on_dismiss(mut self, f: impl Fn(u64, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ToasterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let live: Vec<_> = self.model.live().into_iter().cloned().collect();
        let hs = vec![self.height; live.len()];
        let slots = self.model.layout(&hs);
        let top = self.model.position.top();
        let mut col = div()
            .id("oxyn-toaster")
            .role(Role::Group)
            .aria_label(SharedString::from(self.model.region_label()))
            .relative()
            .w(px(356.))
            .h(px(self.model.extent(&hs)));
        for (i, ts) in live.iter().enumerate() {
            let s = slots[i];
            if !s.shown {
                continue;
            }
            let id = ts.id;
            let f = self.on_dismiss.clone();
            let mark = match ts.kind {
                ToastKind::Success => ("✓", ColorRole::StatusSuccessFg),
                ToastKind::Error => ("✕", ColorRole::StatusDangerFg),
                ToastKind::Warning => ("!", ColorRole::StatusWarningFg),
                ToastKind::Info => ("i", ColorRole::FgMuted),
                ToastKind::Loading => ("◌", ColorRole::FgMuted),
                ToastKind::Default => ("", ColorRole::FgMuted),
            };
            let frac = ts.fraction_left(self.now);
            let card = panel(t)
                .id(("toast", id as usize))
                .role(Role::Status)
                .aria_label(SharedString::from(ts.title.clone()))
                .absolute()
                .left(px(356.0 * (1.0 - s.scale) / 2.0 + ts.swipe))
                .w(px(356.0 * s.scale))
                .when(top, |d| d.top(px(s.y)))
                .when(!top, |d| d.bottom(px(s.y)))
                .rounded(px(12.))
                .opacity(s.opacity)
                .overflow_hidden()
                .child(
                    div()
                        .flex()
                        .gap(px(10.))
                        .p(px(12.))
                        .when(!mark.0.is_empty(), |d| {
                            d.child(div().text_color(c(t, mark.1)).child(mark.0))
                        })
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .child(
                                    div()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(ts.title.clone()),
                                )
                                .when_some(ts.description.clone(), |d, s| {
                                    d.child(
                                        div()
                                            .text_size(px(12.5))
                                            .text_color(c(t, ColorRole::FgMuted))
                                            .child(s),
                                    )
                                }),
                        )
                        .when_some(ts.action.clone(), |d, a| {
                            d.child(
                                div()
                                    .px(px(8.))
                                    .py(px(3.))
                                    .rounded(px(6.))
                                    .border_1()
                                    .border_color(c(t, ColorRole::BorderDefault))
                                    .child(a),
                            )
                        })
                        .child(
                            div()
                                .id(("toast-x", id as usize))
                                .role(Role::Button)
                                .aria_label("Close notification")
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child("×")
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(id, w, cx))
                                }),
                        ),
                )
                .when(ts.duration.is_some(), |d| {
                    d.child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .h(px(1.))
                            .w(px(356.0 * frac))
                            .bg(c(t, ColorRole::FgSubtle)),
                    )
                });
            col = col.child(card);
        }
        col
    }
}

/// `notification-stack`: the front card and the edges of the ones behind.
#[derive(IntoElement)]
pub struct NotificationStackEl {
    model: NotificationStack,
    on_dismiss: Option<OnClick>,
    theme: &'static Theme,
}

impl NotificationStackEl {
    /// From the model.
    pub fn new(model: NotificationStack) -> Self {
        Self {
            model,
            on_dismiss: None,
            theme: &Theme::DARK,
        }
    }
    /// The front card's close (the host calls `dismiss_front`).
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for NotificationStackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let shown: Vec<_> = self.model.shown().into_iter().cloned().collect();
        let n = shown.len();
        let mut root = div()
            .id("oxyn-notices")
            .role(Role::Group)
            .aria_label("Notices")
            .relative()
            .w(px(340.))
            .h(px(130.0 + 10.0 * n.saturating_sub(1) as f32));
        for (k, nt) in shown.iter().enumerate().rev() {
            let front = k == 0;
            let w = 340.0 * (1.0 - 0.05 * k as f32);
            let f = self.on_dismiss.clone();
            root = root.child(
                panel(t)
                    .absolute()
                    .left(px((340.0 - w) / 2.0))
                    .w(px(w))
                    .top(px(10.0 * (n - 1 - k) as f32))
                    .h(px(120.))
                    .rounded(px(14.))
                    .p(px(14.))
                    .gap(px(4.))
                    .opacity(1.0 - 0.25 * k as f32)
                    .when(front, |d| {
                        d.child(
                            div()
                                .flex()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child(nt.badge.clone().unwrap_or_default()),
                                )
                                .child(
                                    div()
                                        .id("notice-x")
                                        .role(Role::Button)
                                        .aria_label(SharedString::from(format!(
                                            "Dismiss {}",
                                            nt.title
                                        )))
                                        .child("×")
                                        .when_some(f, |d, f| {
                                            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(14.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(nt.title.clone()),
                        )
                        .child(
                            div()
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(nt.text.clone()),
                        )
                    }),
            );
        }
        root
    }
}

// ---------------------------------------------------------------------------------------------
// Alert and card

/// `alert`: tone icon, title, details, actions.
#[derive(IntoElement)]
pub struct AlertEl {
    model: Alert,
    on_dismiss: Option<OnClick>,
    theme: &'static Theme,
}

impl AlertEl {
    /// From the model.
    pub fn new(model: Alert) -> Self {
        Self {
            model,
            on_dismiss: None,
            theme: &Theme::DARK,
        }
    }
    /// Dismiss pressed.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for AlertEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let a = self.model;
        if !a.open {
            return div().into_any_element();
        }
        let (fg, bg, bd, mark) = match a.tone {
            AlertTone::Info => (
                ColorRole::StatusInfoFg,
                ColorRole::StatusInfoBg,
                ColorRole::StatusInfoBorder,
                "i",
            ),
            AlertTone::Success => (
                ColorRole::StatusSuccessFg,
                ColorRole::StatusSuccessBg,
                ColorRole::StatusSuccessBorder,
                "✓",
            ),
            AlertTone::Warning => (
                ColorRole::StatusWarningFg,
                ColorRole::StatusWarningBg,
                ColorRole::StatusWarningBorder,
                "!",
            ),
            AlertTone::Danger => (
                ColorRole::StatusDangerFg,
                ColorRole::StatusDangerBg,
                ColorRole::StatusDangerBorder,
                "✕",
            ),
            AlertTone::Agent => (
                ColorRole::AgentViolet,
                ColorRole::SurfaceRaised,
                ColorRole::BorderDefault,
                "✦",
            ),
        };
        let f = self.on_dismiss.clone();
        div()
            .id("oxyn-alert")
            .role(Role::Status)
            .aria_label(SharedString::from(a.title.clone()))
            .flex()
            .gap(px(10.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, bd))
            .bg(c(t, bg))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(div().text_color(c(t, fg)).child(mark))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(2.))
                    .child(div().font_weight(FontWeight::MEDIUM).child(a.title.clone()))
                    .when_some(a.body.clone(), |d, b| {
                        d.child(div().text_color(c(t, ColorRole::FgMuted)).child(b))
                    })
                    .when(!a.actions.is_empty(), |d| {
                        d.child(
                            div()
                                .flex()
                                .gap(px(6.))
                                .mt(px(8.))
                                .children(a.actions.iter().map(|x| {
                                    div()
                                        .px(px(8.))
                                        .py(px(3.))
                                        .rounded(px(6.))
                                        .border_1()
                                        .border_color(c(t, ColorRole::BorderDefault))
                                        .child(x.clone())
                                })),
                        )
                    })
                    .when(a.expanded, |d| {
                        d.child(
                            div()
                                .mt(px(8.))
                                .p(px(8.))
                                .rounded(px(8.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(a.details.clone().unwrap_or_default()),
                        )
                    }),
            )
            .when(a.dismissible, |d| {
                d.child(close_x(t, "alert-x", "Dismiss", f))
            })
            .into_any_element()
    }
}

/// `card`: media band, title, description, byline, status; status edge.
#[derive(IntoElement)]
pub struct CardEl {
    model: Card,
    title: SharedString,
    description: Option<SharedString>,
    meta: Option<SharedString>,
    edge: CardEdge,
    now: ClockMs,
    on_open: Option<OnClick>,
    theme: &'static Theme,
}

impl CardEl {
    /// A card.
    pub fn new(model: Card, title: impl Into<SharedString>, now: ClockMs) -> Self {
        Self {
            model,
            title: title.into(),
            description: None,
            meta: None,
            edge: CardEdge::None,
            now,
            on_open: None,
            theme: &Theme::DARK,
        }
    }
    /// Description.
    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Owner.
    pub fn meta(mut self, m: impl Into<SharedString>) -> Self {
        self.meta = Some(m.into());
        self
    }
    /// Status edge.
    pub fn edge(mut self, e: CardEdge) -> Self {
        self.edge = e;
        self
    }
    /// Title pressed (quick look).
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for CardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let pulse = 0.5 + 0.5 * (sheen_phase(self.now, sk::SHEEN) * std::f32::consts::TAU).sin();
        let border = match self.edge {
            CardEdge::Working => alpha(c(t, ColorRole::FgDefault), 0.35 + 0.5 * pulse),
            CardEdge::Attention => c(t, ColorRole::StatusWarningBorder),
            CardEdge::Done => c(t, ColorRole::StatusSuccessBorder),
            CardEdge::Error => c(t, ColorRole::StatusDangerBorder),
            CardEdge::None if self.model.selected => c(t, ColorRole::FgMuted),
            CardEdge::None => c(t, ColorRole::BorderDefault),
        };
        let f = self.on_open.clone();
        div()
            .id("oxyn-card")
            .flex()
            .flex_col()
            .w(px(300.))
            .rounded(px(14.))
            .overflow_hidden()
            .border_1()
            .border_color(border)
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .child(div().h(px(120.)).bg(c(t, ColorRole::SurfaceSunken)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .p(px(14.))
                    .child(
                        div()
                            .id("card-title")
                            .role(Role::Button)
                            .text_size(px(14.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title)
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                            }),
                    )
                    .when_some(self.description, |d, s| {
                        d.child(div().text_color(c(t, ColorRole::FgMuted)).child(s))
                    }),
            )
            .when_some(self.meta, |d, m| {
                d.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .items_center()
                        .px(px(14.))
                        .pb(px(14.))
                        .child(
                            div()
                                .size(px(24.))
                                .rounded_full()
                                .bg(c(t, ColorRole::SurfaceSelected)),
                        )
                        .child(
                            div().flex().flex_col().child(m).child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(self.model.status.clone().unwrap_or_default()),
                            ),
                        ),
                )
            })
    }
}

// ---------------------------------------------------------------------------------------------
// Scroll area and pagination

/// `scroll-area`: native scroll with an overlay thumb, edge fades and the "Jump to latest" pill.
#[derive(IntoElement)]
pub struct ScrollAreaEl {
    metrics: Metrics,
    stick: StickToBottom,
    fade: f32,
    show_bar: bool,
    height: f32,
    content: Vec<AnyElement>,
    on_jump: Option<OnClick>,
    theme: &'static Theme,
}

impl ScrollAreaEl {
    /// With the last measured metrics.
    pub fn new(metrics: Metrics, height: f32) -> Self {
        Self {
            metrics,
            stick: StickToBottom::default(),
            fade: 28.0,
            show_bar: true,
            height,
            content: vec![],
            on_jump: None,
            theme: &Theme::DARK,
        }
    }
    /// Stick-to-bottom state.
    pub fn stick(mut self, s: StickToBottom) -> Self {
        self.stick = s;
        self
    }
    /// Scrollbar shown (the host applies `Scrollbars::shown`).
    pub fn bar(mut self, on: bool) -> Self {
        self.show_bar = on;
        self
    }
    /// The pill pressed.
    pub fn on_jump(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_jump = Some(Rc::new(f));
        self
    }
    themed!();
}

impl ParentElement for ScrollAreaEl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements)
    }
}

impl RenderOnce for ScrollAreaEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.metrics;
        let (ft, fb, ..) = fades(&m, self.fade);
        let (tl, to) = thumb(m.top, m.height, m.view_h, self.height - 8.0);
        let bg = c(t, ColorRole::SurfaceRaised);
        let f = self.on_jump.clone();
        div()
            .relative()
            .h(px(self.height))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(bg)
            .overflow_hidden()
            .child(
                div()
                    .id("oxyn-scroll-vp")
                    .size_full()
                    .overflow_y_scroll()
                    .children(self.content),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(ft))
                    .bg(alpha(bg, 0.7)),
            )
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(fb))
                    .bg(alpha(bg, 0.7)),
            )
            .when(self.show_bar && m.overflows_y(), |d| {
                d.child(
                    div()
                        .absolute()
                        .right(px(3.))
                        .top(px(4.0 + to))
                        .w(px(5.))
                        .h(px(tl))
                        .rounded_full()
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.32)),
                )
            })
            .when(self.stick.pill(), |d| {
                d.child(
                    div()
                        .id("scroll-pill")
                        .role(Role::Button)
                        .absolute()
                        .bottom(px(12.))
                        .left(px(0.))
                        .right(px(0.))
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded_full()
                                .bg(c(t, ColorRole::SurfaceOverlay))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderDefault))
                                .child(format!("↓ {}", self.stick.pill_label())),
                        )
                        .when_some(f, |d, f| {
                            d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                        }),
                )
            })
    }
}

/// `pagination`: previous, the page window with the current mark, next.
#[derive(IntoElement)]
pub struct PaginationEl {
    model: Pagination,
    on_page: Option<OnIndex>,
    theme: &'static Theme,
}

impl PaginationEl {
    /// From the model.
    pub fn new(model: Pagination) -> Self {
        Self {
            model,
            on_page: None,
            theme: &Theme::DARK,
        }
    }
    /// A page pressed (0 = previous, usize::MAX = next).
    pub fn on_page(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_page = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for PaginationEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.model;
        let btn = |id: (&'static str, usize),
                   label: SharedString,
                   on: bool,
                   disabled: bool,
                   target: usize,
                   f: Option<OnIndex>| {
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(label.clone())
                .flex()
                .items_center()
                .justify_center()
                .size(px(32.))
                .rounded(px(8.))
                .when(on, |d| {
                    d.bg(c(t, ColorRole::SurfaceSelected))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderStrong))
                        .font_weight(FontWeight::SEMIBOLD)
                })
                .text_color(if disabled {
                    c(t, ColorRole::FgDisabled)
                } else if on {
                    c(t, ColorRole::FgDefault)
                } else {
                    c(t, ColorRole::FgMuted)
                })
                .child(label)
                .when_some(f.filter(|_| !disabled), |d, f| {
                    d.cursor_pointer()
                        .on_click(move |_, w, cx| f(target, w, cx))
                })
        };
        div()
            .id("oxyn-pager")
            .role(Role::Group)
            .aria_label("Pagination")
            .flex()
            .items_center()
            .gap(px(4.))
            .text_size(px(13.))
            .child(btn(
                ("pg-prev", 0),
                "‹".into(),
                false,
                p.prev_disabled(),
                p.page.saturating_sub(1),
                self.on_page.clone(),
            ))
            .children(p.slots().into_iter().enumerate().map(|(i, s)| {
                match s {
                    PageSlot::Gap => div()
                        .w(px(32.))
                        .text_center()
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child("…")
                        .into_any_element(),
                    PageSlot::Page(n) => btn(
                        ("pg", i),
                        SharedString::from(n.to_string()),
                        n == p.page,
                        false,
                        n,
                        self.on_page.clone(),
                    )
                    .into_any_element(),
                }
            }))
            .child(btn(
                ("pg-next", 0),
                "›".into(),
                false,
                p.next_disabled(),
                p.page + 1,
                self.on_page.clone(),
            ))
    }
}

// ---------------------------------------------------------------------------------------------
// Skeletons

/// `skeleton`: shape rows; the shared sheen modulates every bone in step.
#[derive(IntoElement)]
pub struct SkeletonEl {
    shape: Shape,
    lines: usize,
    avatar: bool,
    sweep: Sweep,
    width: f32,
    now: ClockMs,
    theme: &'static Theme,
}

impl SkeletonEl {
    /// A skeleton.
    pub fn new(shape: Shape, lines: usize, now: ClockMs) -> Self {
        Self {
            shape,
            lines,
            avatar: false,
            sweep: Sweep::Shimmer,
            width: 320.0,
            now,
            theme: &Theme::DARK,
        }
    }
    /// Avatar row.
    pub fn avatar(mut self, a: bool) -> Self {
        self.avatar = a;
        self
    }
    /// Sweep style.
    pub fn sweep(mut self, s: Sweep) -> Self {
        self.sweep = s;
        self
    }
    themed!();
}

impl RenderOnce for SkeletonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let phase = sheen_phase(self.now, sk::SHEEN);
        let base = c(t, ColorRole::SurfaceHover);
        let w = self.width;
        div()
            .id("oxyn-skeleton")
            .role(Role::Status)
            .aria_label("Loading content")
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(w))
            .children(
                self.shape
                    .rows(self.lines, self.avatar)
                    .into_iter()
                    .enumerate()
                    .map(move |(ri, row)| {
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .children(row.into_iter().map(move |b| {
                                let bw = if b.px { b.w } else { b.w * w * 0.8 };
                                let x = bw / 2.0 / w;
                                let a = match self.sweep {
                                    Sweep::Shimmer => {
                                        1.0 + 0.6
                                            * (1.0 - ((phase * 2.0 - 0.5) - x).abs() * 3.0).max(0.0)
                                    }
                                    Sweep::Pulse => {
                                        0.7 + 0.3 * (phase * std::f32::consts::TAU).cos()
                                    }
                                    Sweep::Wave => {
                                        0.7 + 0.3
                                            * (phase * std::f32::consts::TAU - ri as f32 * 0.5)
                                                .cos()
                                    }
                                    Sweep::None => 1.0,
                                };
                                div()
                                    .w(px(bw))
                                    .h(px(b.h))
                                    .rounded(px(if b.r.is_infinite() { b.h / 2.0 } else { b.r }))
                                    .bg(alpha(base, a.min(1.6) / 1.6 + 0.35))
                            }))
                    }),
            )
    }
}

/// `skeleton-morph`: blocks fade from placeholder to content in reading order.
#[derive(IntoElement)]
pub struct SkeletonMorphEl {
    morph: Morph,
    blocks: Vec<(f32, f32, SharedString)>,
    now: ClockMs,
    reduced: bool,
    theme: &'static Theme,
}

impl SkeletonMorphEl {
    /// Blocks: (skeleton width, height, loaded text).
    pub fn new(morph: Morph, blocks: Vec<(f32, f32, SharedString)>, now: ClockMs) -> Self {
        Self {
            morph,
            blocks,
            now,
            reduced: false,
            theme: &Theme::DARK,
        }
    }
    /// Reduced motion.
    pub fn reduced(mut self, r: bool) -> Self {
        self.reduced = r;
        self
    }
    themed!();
}

impl RenderOnce for SkeletonMorphEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let n = self.blocks.len();
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .w(px(340.))
            .rounded(px(14.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .children(self.blocks.into_iter().enumerate().map(|(i, (w, h, txt))| {
                let p = self.morph.progress(i, n, self.now, self.reduced);
                let (sw, sh) = Morph::stretch((w, h), (300.0, h.max(16.0)), p);
                div()
                    .relative()
                    .min_h(px(h))
                    .child(div().opacity(p).child(txt))
                    .when(p < 1.0, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .w(px(sw))
                                .h(px(sh))
                                .rounded(px(6.))
                                .bg(c(t, ColorRole::SurfaceHover))
                                .opacity(1.0 - p),
                        )
                    })
            }))
    }
}

// ---------------------------------------------------------------------------------------------
// Command palette, AI command, AI search bar

fn input_row(
    t: &'static Theme,
    lead: &'static str,
    text: String,
    placeholder: &'static str,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .h(px(50.))
        .px(px(14.))
        .border_b_1()
        .border_color(c(t, ColorRole::BorderSubtle))
        .child(div().text_color(c(t, ColorRole::FgMuted)).child(lead))
        .child(if text.is_empty() {
            div()
                .text_size(px(15.))
                .text_color(c(t, ColorRole::FgSubtle))
                .child(placeholder)
        } else {
            div().text_size(px(15.)).child(text)
        })
}

fn result_row(
    t: &'static Theme,
    id: (&'static str, usize),
    label: String,
    on: bool,
    hint: Option<String>,
    f: Option<OnIndex>,
    idx: usize,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::ListBoxOption)
        .aria_selected(on)
        .flex()
        .items_center()
        .gap(px(10.))
        .min_h(px(38.))
        .px(px(10.))
        .rounded(px(8.))
        .when(on, |d| d.bg(c(t, ColorRole::SurfaceHover)))
        .child(div().flex_1().child(label))
        .when_some(hint, |d, h| {
            d.child(
                div()
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child(h),
            )
        })
        .when_some(f, |d, f| {
            d.cursor_pointer().on_click(move |_, w, cx| f(idx, w, cx))
        })
}

/// `command-palette`: field, grouped results with the active row, footer hints.
#[derive(IntoElement)]
pub struct CommandPaletteEl {
    model: Palette,
    on_run: Option<OnIndex>,
    theme: &'static Theme,
}

impl CommandPaletteEl {
    /// From the model.
    pub fn new(model: Palette) -> Self {
        Self {
            model,
            on_run: None,
            theme: &Theme::DARK,
        }
    }
    /// A row pressed (item index; the host calls `Palette::run`).
    pub fn on_run(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_run = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for CommandPaletteEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.model;
        let mut flat = 0usize;
        let mut list = div()
            .flex()
            .flex_col()
            .p(px(6.))
            .max_h(px(342.))
            .id("cp-list")
            .role(Role::ListBox)
            .overflow_y_scroll();
        if p.asking() {
            list = list.child(result_row(
                t,
                ("cp-ask", 0),
                format!("✦ Ask the assistant: {}", p.query),
                true,
                Some("↵".into()),
                None,
                0,
            ));
        }
        for sec in p.sections() {
            list = list.child(
                div()
                    .px(px(10.))
                    .pt(px(8.))
                    .pb(px(4.))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child(sec.label.clone()),
            );
            for (item, _) in sec.rows {
                let i = flat;
                flat += 1;
                let cmd = &p.items[item];
                let hint = p
                    .digit_for(item)
                    .map(|d| format!("⌘{d}"))
                    .or_else(|| cmd.shortcut.clone())
                    .or_else(|| cmd.hint.clone());
                list = list.child(result_row(
                    t,
                    ("cp-row", i),
                    cmd.label.clone(),
                    p.active == i,
                    hint,
                    self.on_run.clone(),
                    item,
                ));
            }
        }
        panel(t)
            .w(px(560.))
            .rounded(px(14.))
            .overflow_hidden()
            .child(input_row(
                t,
                "⌕",
                p.query.clone(),
                "Type a command or search…",
            ))
            .child(list)
            .when_some(p.ran.clone(), |d, (what, _)| {
                d.child(
                    div()
                        .px(px(14.))
                        .py(px(8.))
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(format!("✓ {what}")),
                )
            })
            .child(
                div()
                    .flex()
                    .gap(px(14.))
                    .px(px(14.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child("↑↓ Move")
                    .child("↵ Run")
                    .child("⇥ Section")
                    .child("esc Close"),
            )
    }
}

fn answer_text(t: &'static Theme, text: &str) -> gpui::Div {
    div()
        .flex()
        .flex_wrap()
        .text_size(px(13.5))
        .children(segments(text).into_iter().map(|s| {
            match s {
                Seg::Text(x) => div().child(x).into_any_element(),
                Seg::Code(x) => div()
                    .px(px(4.))
                    .rounded(px(4.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .child(x)
                    .into_any_element(),
                Seg::Cite(n) => div()
                    .mx(px(2.))
                    .px(px(4.))
                    .rounded(px(4.))
                    .bg(c(t, ColorRole::SurfaceSelected))
                    .text_size(px(10.))
                    .child(n.to_string())
                    .into_any_element(),
            }
        }))
}

/// `ai-command`: palette with Ask mode; streamed answer, sources and next steps.
#[derive(IntoElement)]
pub struct AiCommandEl {
    model: AiCommand,
    now: ClockMs,
    on_action: Option<OnIndex>,
    theme: &'static Theme,
}

impl AiCommandEl {
    /// From the model.
    pub fn new(model: AiCommand, now: ClockMs) -> Self {
        Self {
            model,
            now,
            on_action: None,
            theme: &Theme::DARK,
        }
    }
    /// A next step pressed.
    pub fn on_action(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for AiCommandEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let now = self.now;
        let ask = m.mode == AiMode::Ask;
        let phase = m.answer.phase(now);
        let mut body = div()
            .id("ai-body")
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(10.))
            .h(px(392.))
            .overflow_y_scroll();
        if !ask {
            for (i, item) in m.palette.flat().into_iter().enumerate() {
                body = body.child(result_row(
                    t,
                    ("ai-row", i),
                    m.palette.items[item].label.clone(),
                    m.palette.active == i,
                    m.palette.items[item].hint.clone(),
                    None,
                    item,
                ));
            }
        } else if phase == AnswerPhase::Thinking {
            body = body.child(
                div()
                    .text_color(c(t, ColorRole::FgMuted))
                    .child("Searching your workspace…"),
            );
        } else if phase != AnswerPhase::Idle {
            body = body
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(m.question.clone()),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .children(m.answer.sources.iter().enumerate().map(|(k, s)| {
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(6.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .text_size(px(12.))
                                .child(format!("{} {}", k + 1, s.title))
                        })),
                )
                .child(answer_text(t, &m.answer.visible(now)));
            if phase == AnswerPhase::Done {
                for (i, a) in m.actions.iter().enumerate() {
                    let label = match a.state(now) {
                        ActionState::Done => format!("✓ {}", a.done_label),
                        ActionState::Running => format!("◌ {}…", a.label),
                        _ => a.label.clone(),
                    };
                    body = body.child(result_row(
                        t,
                        ("ai-act", i),
                        label,
                        m.ask_active == i,
                        None,
                        self.on_action.clone(),
                        i,
                    ));
                }
            }
        } else {
            for (i, q) in m.suggested_questions.iter().enumerate() {
                body = body.child(result_row(
                    t,
                    ("ai-q", i),
                    q.clone(),
                    m.ask_active == i,
                    None,
                    None,
                    i,
                ));
            }
        }
        panel(t)
            .w(px(600.))
            .rounded(px(14.))
            .overflow_hidden()
            .child(input_row(
                t,
                if ask { "✦" } else { "⌕" },
                if ask {
                    m.question.clone()
                } else {
                    m.palette.query.clone()
                },
                "Search pages and actions, or press Tab to ask",
            ))
            .child(body)
            .child(
                div()
                    .px(px(14.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child(m.status(now, "Searching your workspace")),
            )
    }
}

/// `ai-search-bar`: Ask AI row (or the answer), then grouped results with citation badges.
#[derive(IntoElement)]
pub struct AiSearchBarEl {
    model: AiSearch,
    now: ClockMs,
    theme: &'static Theme,
}

impl AiSearchBarEl {
    /// From the model.
    pub fn new(model: AiSearch, now: ClockMs) -> Self {
        Self {
            model,
            now,
            theme: &Theme::DARK,
        }
    }
    themed!();
}

impl RenderOnce for AiSearchBarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let now = self.now;
        let mut body = div()
            .id("ais-body")
            .flex()
            .flex_col()
            .gap(px(4.))
            .p(px(6.))
            .h(px(320.))
            .overflow_y_scroll();
        if m.answer.phase(now) == AnswerPhase::Idle {
            if !m.query.is_empty() {
                body = body.child(result_row(
                    t,
                    ("ais-ask", 0),
                    format!("✦ Ask AI about “{}”", m.query),
                    m.active == 0,
                    Some("⇥".into()),
                    None,
                    0,
                ));
            }
        } else {
            body = body.child(
                div()
                    .p(px(10.))
                    .rounded(px(10.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .child(answer_text(t, &m.answer.visible(now))),
            );
        }
        let mut i = 1;
        for sec in m.sections() {
            body = body.child(
                div()
                    .px(px(10.))
                    .pt(px(8.))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child(sec.label.clone()),
            );
            for (item, _) in sec.rows {
                let r = &m.results[item];
                let badge = m.badge(&r.id, now).map(|n| format!("[{n}]"));
                body = body.child(result_row(
                    t,
                    ("ais-row", i),
                    r.label.clone(),
                    m.active == i,
                    badge.or(r.hint.clone()),
                    None,
                    item,
                ));
                i += 1;
            }
        }
        panel(t)
            .w(px(600.))
            .rounded(px(14.))
            .overflow_hidden()
            .child(input_row(
                t,
                "⌕",
                m.query.clone(),
                "Search docs, customers and issues",
            ))
            .child(body)
    }
}

// ---------------------------------------------------------------------------------------------
// Search field, expanding search, recorder

/// `search-field`: icon, value, the clear button in its reserved slot.
#[derive(IntoElement)]
pub struct SearchFieldEl {
    label: SharedString,
    value: String,
    on_clear: Option<OnClick>,
    theme: &'static Theme,
}

impl SearchFieldEl {
    /// A field.
    pub fn new(label: impl Into<SharedString>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            on_clear: None,
            theme: &Theme::DARK,
        }
    }
    /// Clear pressed.
    pub fn on_clear(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for SearchFieldEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let show = clear_visible(&self.value, false);
        let f = self.on_clear.clone();
        div()
            .id("oxyn-search")
            .role(Role::TextInput)
            .aria_label(self.label.clone())
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(36.))
            .w(px(320.))
            .pl(px(10.))
            .pr(px(4.))
            .rounded(px(9.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_size(px(13.5))
            .text_color(c(t, ColorRole::FgDefault))
            .child(div().text_color(c(t, ColorRole::FgSubtle)).child("⌕"))
            .child(div().flex_1().child(if self.value.is_empty() {
                SharedString::from("Search")
            } else {
                SharedString::from(self.value.clone())
            }))
            .child(
                div()
                    .id("search-clear")
                    .size(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(show, |d| {
                        d.role(Role::Button)
                            .aria_label("Clear search")
                            .child(
                                div()
                                    .size(px(20.))
                                    .rounded_full()
                                    .bg(c(t, ColorRole::SurfaceSelected))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child("×"),
                            )
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                            })
                    }),
            )
    }
}

/// `expanding-search`: the icon button, or the grown field with its results panel.
#[derive(IntoElement)]
pub struct ExpandingSearchEl {
    model: ExpandingSearch,
    lane: f32,
    on_toggle: Option<OnClick>,
    theme: &'static Theme,
}

impl ExpandingSearchEl {
    /// From the model and the lane width.
    pub fn new(model: ExpandingSearch, lane: f32) -> Self {
        Self {
            model,
            lane,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    /// Icon / close pressed.
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ExpandingSearchEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let w = m.width(self.lane, 36.0);
        let f = self.on_toggle.clone();
        let field = div()
            .id("xs-field")
            .role(if m.expanded {
                Role::ComboBox
            } else {
                Role::Button
            })
            .aria_label("Search")
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(38.))
            .w(px(w))
            .px(px(if m.expanded { 12.0 } else { 0.0 }))
            .justify_center()
            .rounded(px(if m.expanded { 10.0 } else { 19.0 }))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .child("⌕")
            .when(m.expanded, |d| {
                d.child(div().flex_1().child(m.query.clone()))
            })
            .when_some(f, |d, f| {
                d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
            });
        let mut col = div()
            .flex()
            .flex_col()
            .items_end()
            .w(px(self.lane))
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(field);
        if m.expanded {
            let rows = m.flat();
            if !rows.is_empty() {
                col = col.child(panel(t).w(px(w)).p(px(4.)).rounded(px(12.)).children(
                    rows.into_iter().enumerate().map(|(i, item)| {
                        result_row(
                            t,
                            ("xs-row", i),
                            m.item(item).map(|c| c.label.clone()).unwrap_or_default(),
                            m.active == Some(i),
                            None,
                            None,
                            item,
                        )
                    }),
                ));
            }
        }
        col
    }
}

/// `shortcut-recorder`: the recorder button with caps, held modifiers or the conflict line.
#[derive(IntoElement)]
pub struct ShortcutRecorderEl {
    model: Recorder,
    label: SharedString,
    on_press: Option<OnClick>,
    theme: &'static Theme,
}

impl ShortcutRecorderEl {
    /// From the model.
    pub fn new(model: Recorder, label: impl Into<SharedString>) -> Self {
        Self {
            model,
            label: label.into(),
            on_press: None,
            theme: &Theme::DARK,
        }
    }
    /// Pressed (the host calls `start`).
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ShortcutRecorderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.model;
        let p = r.platform;
        let cap = |s: String| {
            div()
                .px(px(6.))
                .h(px(22.))
                .flex()
                .items_center()
                .rounded(px(6.))
                .border_1()
                .border_color(c(t, ColorRole::BorderStrong))
                .bg(c(t, ColorRole::SurfaceSunken))
                .child(s)
        };
        let content: Vec<AnyElement> = match (&r.state, &r.value) {
            (RecState::Recording { held, .. }, _) => {
                let mut v: Vec<AnyElement> = vec![
                    div()
                        .size(px(7.))
                        .rounded_full()
                        .bg(c(t, ColorRole::StatusDangerFg))
                        .into_any_element(),
                ];
                let mut hs = vec![];
                if held.ctrl {
                    hs.push(if p == Platform::Mac { "⌃" } else { "Ctrl" })
                }
                if held.alt {
                    hs.push(if p == Platform::Mac { "⌥" } else { "Alt" })
                }
                if held.shift {
                    hs.push(if p == Platform::Mac { "⇧" } else { "Shift" })
                }
                if held.meta {
                    hs.push(if p == Platform::Mac { "⌘" } else { "Win" })
                }
                if hs.is_empty() {
                    v.push(
                        div()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child("Press keys…")
                            .into_any_element(),
                    );
                }
                v.extend(
                    hs.into_iter()
                        .map(|h| cap(h.to_string()).into_any_element()),
                );
                v
            }
            (_, Some(val)) => caps(val, p)
                .into_iter()
                .map(|k| cap(k).into_any_element())
                .collect(),
            _ => vec![
                div()
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child("Record shortcut")
                    .into_any_element(),
            ],
        };
        let msg = match &r.state {
            RecState::Conflict { with, .. } => Some((
                format!("Already used: {with}. Use anyway or cancel"),
                ColorRole::StatusWarningFg,
            )),
            RecState::Invalid(m) => Some((m.clone(), ColorRole::StatusDangerFg)),
            _ => None,
        };
        let f = self.on_press.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w(px(320.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .text_size(px(12.5))
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label.clone()),
            )
            .child(
                div()
                    .id("recorder")
                    .role(Role::Button)
                    .aria_label(SharedString::from(r.aria(&self.label)))
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .h(px(38.))
                    .px(px(10.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(if r.recording() {
                        c(t, ColorRole::FgMuted)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .children(content)
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
                    }),
            )
            .when_some(msg, |d, (m, role)| {
                d.child(div().text_size(px(12.)).text_color(c(t, role)).child(m))
            })
    }
}

// ---------------------------------------------------------------------------------------------
// Control center

/// `control-center`: tiles, meters, and the open detail.
#[derive(IntoElement)]
pub struct ControlCenterEl {
    model: ControlCenter,
    on_tile: Option<OnIndex>,
    theme: &'static Theme,
}

impl ControlCenterEl {
    /// From the model.
    pub fn new(model: ControlCenter) -> Self {
        Self {
            model,
            on_tile: None,
            theme: &Theme::DARK,
        }
    }
    /// Tile pressed (0 focus, 1 notifications, 2 presenting).
    pub fn on_tile(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_tile = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ControlCenterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let tile = |i: usize, name: &'static str, on: bool, line: String, f: Option<OnIndex>| {
            div()
                .id(("cc-tile", i))
                .role(Role::Button)
                .aria_label(SharedString::from(name))
                .flex()
                .flex_col()
                .w(px(160.))
                .h(px(64.))
                .p(px(12.))
                .rounded(px(14.))
                .bg(if on {
                    c(t, ColorRole::FgDefault)
                } else {
                    c(t, ColorRole::SurfaceSunken)
                })
                .text_color(if on {
                    c(t, ColorRole::SurfaceOverlay)
                } else {
                    c(t, ColorRole::FgDefault)
                })
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.5))
                        .child(name),
                )
                .child(div().text_size(px(11.)).opacity(0.7).child(line))
                .when_some(f, |d, f| {
                    d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                })
        };
        let meter = |which: Meter, name: &'static str| {
            let (lo, hi, _) = which.range();
            let v = m.meter(which);
            div()
                .relative()
                .h(px(44.))
                .w(px(328.))
                .rounded(px(14.))
                .overflow_hidden()
                .bg(c(t, ColorRole::SurfaceSunken))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(px(328.0 * (v - lo) / (hi - lo)))
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.22)),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(14.))
                        .child(name)
                        .child(m.meter_text(which)),
                )
        };
        let f = self.on_tile.clone();
        let mut root = panel(t)
            .relative()
            .w(px(360.))
            .h(px(330.))
            .p(px(12.))
            .gap(px(8.))
            .rounded(px(20.))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(tile(
                        0,
                        "Focus",
                        m.focus.is_some(),
                        m.focus_label(),
                        f.clone(),
                    ))
                    .child(tile(
                        1,
                        "Notifications",
                        m.notifications(),
                        m.notifications_label(),
                        f.clone(),
                    ))
                    .child(tile(
                        2,
                        "Presenting",
                        m.presenting,
                        if m.presenting {
                            "On".into()
                        } else {
                            "Off".into()
                        },
                        f.clone(),
                    )),
            )
            .child(meter(Meter::Volume, "Alert volume"))
            .child(meter(Meter::TextSize, "Text size"));
        if let Some(d) = m.detail {
            let body = match d {
                Detail::Focus => div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .children(m.modes.iter().enumerate().map(|(i, md)| {
                        div()
                            .p(px(8.))
                            .rounded(px(10.))
                            .when(i == m.draft_mode, |d| {
                                d.border_1().border_color(c(t, ColorRole::FgMuted))
                            })
                            .child(md.label.clone())
                    }))
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_center()
                            .child(format!("{} min", m.draft_minutes.round())),
                    )
                    .child(
                        div()
                            .text_center()
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(format!("until {}", m.draft_until())),
                    )
                    .child(div().text_center().child(m.session_button())),
                Detail::Notifications | Detail::Agent => {
                    div()
                        .flex()
                        .flex_col()
                        .children(m.channels.iter().map(|ch| {
                            div()
                                .flex()
                                .justify_between()
                                .py(px(8.))
                                .child(ch.label.clone())
                                .child(if ch.on { "On" } else { "Off" })
                        }))
                }
            };
            root = root.child(
                panel(t)
                    .id("cc-detail")
                    .role(Role::Dialog)
                    .absolute()
                    .inset_0()
                    .m(px(8.))
                    .p(px(12.))
                    .rounded(px(16.))
                    .child(body),
            );
        }
        root
    }
}

/// Size id helper so hosts can map Dioxus props onto GPUI.
pub fn dialog_size(id: &str) -> DialogSize {
    DialogSize::from_id(id)
}
