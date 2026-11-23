//! Number entry, meters, uploads, composers and model menus for GPUI (plan #28, chunk 4b).
//! Elements are `RenderOnce` views of the `oxyn_core::kit::{fields, meters, upload, compose}`
//! models; the host keeps the model, drives the clock and routes [`Act`] ids (and row picks
//! through [`Pick`]) back to it. Motion that needs per-frame springs (strain, rubber band, wave
//! fills, clip-path morphs) is left to the host's animation loop.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    kit::{
        compose::{BOLD, CODE, Doc, Effort, Kind, RingTone, RunSeg, RunStatus},
        fields::{NumberField, Scrub, UnfoldSelect, format_number},
        meters::{
            Band, Cat, CellField, ContextGrid, Share, Tone, UsageState, amount, gauge, pct1, split,
            tokens, usage,
        },
        upload::{Crop, Status, Uploads, ext, progress, size, time_left},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, muted, tag};

/// Row pick callback: the row index and the window.
pub type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn bar(t: &'static Theme, frac: f32, col: Hsla) -> impl IntoElement {
    div()
        .h(px(4.))
        .w_full()
        .rounded(px(999.))
        .bg(alpha(c(t, ColorRole::FgDefault), 0.1))
        .child(
            div()
                .h_full()
                .w(relative(frac.clamp(0.0, 1.0)))
                .rounded(px(999.))
                .bg(col),
        )
}

/// `file-dropzone`: the target and its rows.
#[derive(IntoElement)]
pub struct FileDropzoneEl {
    u: Uploads,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl FileDropzoneEl {
    /// A dropzone.
    pub fn new(u: Uploads, now: ClockMs) -> Self {
        Self {
            u,
            now,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Browse callback ("browse").
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for FileDropzoneEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut u = self.u.clone();
        u.tick(self.now);
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(px(420.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .p(px(22.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderStrong))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Add files"))
                    .child(muted(t, u.hint()))
                    .child(button(
                        t,
                        "dz-browse",
                        "Browse",
                        Weight::Plain,
                        "browse",
                        self.act,
                    )),
            )
            .children(u.rows.iter().map(|r| {
                let p = progress(r, self.now);
                let line = match r.status {
                    Status::Done => format!("{} · Uploaded", size(r.bytes)),
                    Status::Failed => r.error.clone().unwrap_or_default(),
                    _ => time_left(p, self.now.since(r.started).as_millis() as u64)
                        .unwrap_or_else(|| size(r.bytes)),
                };
                let failed = r.status == Status::Failed;
                card(t).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(tag(t, ext(&r.name), None))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.))
                                .child(div().text_size(px(12.5)).child(r.name.clone()))
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(c(
                                            t,
                                            if failed {
                                                ColorRole::StatusDangerFg
                                            } else {
                                                ColorRole::FgMuted
                                            },
                                        ))
                                        .child(line),
                                )
                                .when(r.status == Status::Uploading, |d| {
                                    d.child(bar(t, p, c(t, ColorRole::FgDefault)))
                                }),
                        ),
                )
            }))
    }
}

/// `run-controls`: status, steered answer, queue, mode, permission and context.
#[derive(IntoElement)]
pub struct RunControlsEl {
    status: RunStatus,
    prompt: SharedString,
    segs: Vec<RunSeg>,
    queue: Vec<String>,
    act_mode: bool,
    full_access: bool,
    context: f32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl RunControlsEl {
    /// A run.
    pub fn new(status: RunStatus, prompt: impl Into<SharedString>, segs: Vec<RunSeg>) -> Self {
        Self {
            status,
            prompt: prompt.into(),
            segs,
            queue: vec![],
            act_mode: true,
            full_access: false,
            context: 0.4,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Queue, Act mode, full access and context share.
    pub fn state(
        mut self,
        queue: Vec<String>,
        act_mode: bool,
        full_access: bool,
        context: f32,
    ) -> Self {
        self.queue = queue;
        self.act_mode = act_mode;
        self.full_access = full_access;
        self.context = context;
        self
    }
    /// The run's buttons: "stop", "resume", "retry", "dismiss", "send".
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for RunControlsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let a = self.act;
        let ctx_role = if self.context >= 0.8 {
            ColorRole::StatusWarningFg
        } else {
            ColorRole::FgMuted
        };
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(520.))
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.prompt),
                    )
                    .child(muted(t, format!("{:?}", self.status))),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(2.))
                    .children(self.segs.into_iter().map(|s| {
                        match s {
                            RunSeg::Text(x) => div().child(x),
                            RunSeg::Steer(x) => div()
                                .px(px(6.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderStrong))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(format!("↳ {x}")),
                        }
                    })),
            )
            .child(
                div().flex().gap(px(6.)).children(
                    self.status
                        .actions()
                        .iter()
                        .map(|x| button(t, *x, *x, Weight::Plain, x, a.clone())),
                ),
            )
            .children(
                self.queue
                    .into_iter()
                    .map(|q| card(t).child(muted(t, format!("Queued · {q}")))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(tag(t, if self.act_mode { "Act" } else { "Chat" }, None))
                    .when(self.act_mode, |d| {
                        d.child(tag(
                            t,
                            if self.full_access {
                                "Full access"
                            } else {
                                "Ask first"
                            },
                            self.full_access.then_some(ColorRole::StatusDangerFg),
                        ))
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(c(t, ctx_role))
                            .child(format!("{:.0}%", self.context * 100.0)),
                    )
                    .child(button(t, "rc-send", "↑", Weight::Primary, "send", a)),
            )
    }
}

/// `model-menu`: rows with strength bars, the thinking switch and effort.
#[derive(IntoElement)]
pub struct ModelMenuEl {
    rows: Vec<(String, String, Option<[u8; 3]>)>,
    hi: usize,
    thinking: Option<Effort>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl ModelMenuEl {
    /// Rows: name, description and (speed, quality, cost) or `None` for a router.
    pub fn new(rows: Vec<(String, String, Option<[u8; 3]>)>, hi: usize) -> Self {
        Self {
            rows,
            hi,
            thinking: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Thinking on at this effort.
    pub fn thinking(mut self, e: Option<Effort>) -> Self {
        self.thinking = e;
        self
    }
    /// Row picks.
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

impl RenderOnce for ModelMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let bars = move |v: u8| {
            div().flex().gap(px(2.)).children((1..=5u8).map(move |i| {
                div().w(px(5.)).h(px(8.)).rounded(px(1.5)).bg(if i <= v {
                    c(t, ColorRole::FgDefault)
                } else {
                    alpha(c(t, ColorRole::FgDefault), 0.14)
                })
            }))
        };
        card(t)
            .w(px(300.))
            .children(self.rows.into_iter().enumerate().map(|(i, (n, d, s))| {
                let p = self.pick.clone();
                div()
                    .id(("mm", i))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .p(px(8.))
                    .rounded(px(9.))
                    .cursor_pointer()
                    .when(i == self.hi, |x| x.bg(c(t, ColorRole::SurfaceHover)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(n))
                    .child(muted(t, d))
                    .when(i == self.hi, |x| {
                        x.child(match s {
                            Some([sp, q, co]) => div()
                                .flex()
                                .gap(px(10.))
                                .text_size(px(11.))
                                .child(div().flex().gap(px(4.)).child("Speed").child(bars(sp)))
                                .child(div().flex().gap(px(4.)).child("Quality").child(bars(q)))
                                .child(div().flex().gap(px(4.)).child("Cost").child(bars(co))),
                            None => div().text_size(px(11.)).child("Varies"),
                        })
                    })
                    .when_some(p, move |x, p| x.on_click(move |_, w, cx| p(i, w, cx)))
            }))
            .child(muted(
                t,
                match self.thinking {
                    Some(e) => format!("Extended thinking · {}", e.label()),
                    None => "Extended thinking off".into(),
                },
            ))
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
