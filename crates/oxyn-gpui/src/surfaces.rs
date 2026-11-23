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
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

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

/// Size id helper so hosts can map Dioxus props onto GPUI.
pub fn dialog_size(id: &str) -> DialogSize {
    DialogSize::from_id(id)
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
