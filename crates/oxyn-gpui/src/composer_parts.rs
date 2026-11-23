//! Composer parts for GPUI (plan #27, chunk 3), drawn from `oxyn_core::composer::parts`:
//! `VoiceRecorderEl`, `DictationEl`, `MentionListEl`, `ModeSwitchEl`, `PermissionEl`,
//! `ToolsMenuEl`, `QueueStackEl` and `SteeringEl`.
//!
//! Elements are `RenderOnce`: the host owns the model and the clock, routes presses back through
//! the `on_*` callbacks and re-renders. Gaps against Dioxus (listed in the PR): no audio capture
//! (the level meter reads `levels`), text entry belongs to the host's input (the dictation field,
//! mention query, queue edit and steering box are display-only here), popovers draw in place, and
//! there is no blink / wave / pop motion.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::{rc::Rc, time::Duration};

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString, Styled,
    Toggled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    composer::parts::{
        ModeSwitch, Permissions, Queue, RecPhase, Recorder, Risk, Steer, SteerPhase, Suggestion,
        Tool, levels, mmss, tool_groups,
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnText = Rc<dyn Fn(String, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
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

macro_rules! on_index {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

macro_rules! on_text {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

fn btn(
    t: &'static Theme,
    id: impl Into<gpui::ElementId>,
    label: impl Into<SharedString>,
    f: Option<OnIndex>,
    i: usize,
) -> gpui::Stateful<gpui::Div> {
    let label = label.into();
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(label.clone())
        .flex()
        .items_center()
        .justify_center()
        .h(px(28.))
        .px(px(9.))
        .rounded(px(8.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .text_size(px(12.))
        .child(label)
        .when_some(f, move |d, f| {
            d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
        })
}

fn shell(t: &'static Theme) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .min_h(px(44.))
        .px(px(10.))
        .rounded(px(12.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .bg(c(t, ColorRole::SurfaceRaised))
        .text_size(px(13.))
        .text_color(c(t, ColorRole::FgDefault))
}

// ─────────────────────────────────────── Voice recorder ──

/// Voice recorder row. Actions (index): 0 record / permission, 1 pause-resume, 2 stop, 3 discard,
/// 4 send.
#[derive(IntoElement)]
pub struct VoiceRecorderEl {
    model: Recorder,
    now: ClockMs,
    on_action: Option<OnIndex>,
    theme: &'static Theme,
}

impl VoiceRecorderEl {
    /// From the model at `now`.
    pub fn new(model: Recorder, now: ClockMs) -> Self {
        Self {
            model,
            now,
            on_action: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(
        on_action,
        "An action was pressed (see the type docs for indices)."
    );
    themed!();
}

impl RenderOnce for VoiceRecorderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = self.model;
        let el = r.elapsed(self.now);
        let f = self.on_action;
        let rec = r.phase == RecPhase::Recording;
        let bars = levels(
            if rec { self.now.0 } else { 0 },
            24,
            if rec { 1.0 } else { 0.35 },
        );
        let mut row = shell(t).rounded(px(99.));
        row = match r.phase {
            RecPhase::Idle | RecPhase::Denied => row
                .child(btn(t, "rec-mic", "● Record", f.clone(), 0))
                .when(r.phase == RecPhase::Denied, |d| {
                    d.child(
                        div()
                            .text_color(c(t, ColorRole::StatusDangerFg))
                            .child("Microphone blocked"),
                    )
                }),
            RecPhase::Requesting => row
                .child(
                    div()
                        .flex_1()
                        .text_color(c(t, ColorRole::FgMuted))
                        .child("Allow microphone access…"),
                )
                .child(btn(t, "rec-allow", "Allow", f.clone(), 0)),
            RecPhase::Recording | RecPhase::Paused | RecPhase::Review => row
                .child(btn(t, "rec-discard", "Discard", f.clone(), 3))
                .child(div().size(px(8.)).rounded(px(4.)).bg(c(
                    t,
                    if rec {
                        ColorRole::StatusDangerFg
                    } else {
                        ColorRole::FgSubtle
                    },
                )))
                .child(
                    div()
                        .font_family("monospace")
                        .text_color(c(
                            t,
                            if r.near_limit(self.now) {
                                ColorRole::StatusWarningFg
                            } else {
                                ColorRole::FgDefault
                            },
                        ))
                        .child(mmss(el)),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .gap(px(2.))
                        .children(bars.into_iter().map(|h| {
                            div().w(px(3.)).h(px(24. * h)).rounded(px(2.)).bg(c(
                                t,
                                if rec {
                                    ColorRole::FgDefault
                                } else {
                                    ColorRole::FgSubtle
                                },
                            ))
                        })),
                )
                .when(r.phase != RecPhase::Review, |d| {
                    d.child(btn(
                        t,
                        "rec-pause",
                        if rec { "Pause" } else { "Resume" },
                        f.clone(),
                        1,
                    ))
                    .child(btn(t, "rec-stop", "Stop", f.clone(), 2))
                })
                .when(r.phase == RecPhase::Review, |d| {
                    d.child(btn(t, "rec-send", "Send", f.clone(), 4))
                }),
        };
        row.id("oxyn-rec")
            .role(Role::Group)
            .aria_label("Voice message")
    }
}

/// Queue stack: queued follow-ups with remove (index) and a paused marker.
#[derive(IntoElement)]
pub struct QueueStackEl {
    model: Queue,
    on_remove: Option<OnText>,
    on_send_now: Option<OnText>,
    theme: &'static Theme,
}

impl QueueStackEl {
    /// From the model.
    pub fn new(model: Queue) -> Self {
        Self {
            model,
            on_remove: None,
            on_send_now: None,
            theme: &Theme::DARK,
        }
    }
    on_text!(on_remove, "Remove was pressed (item id).");
    on_text!(on_send_now, "Send now was pressed (item id).");
    themed!();
}

impl RenderOnce for QueueStackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let q = self.model;
        let fr = self.on_remove;
        let fs = self.on_send_now;
        div()
            .flex()
            .flex_col()
            .gap(px(3.))
            .w(px(420.))
            .p(px(6.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceSunken))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .px(px(6.))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(format!(
                        "Queued · {}{}",
                        q.items.len(),
                        if q.paused { " · paused" } else { "" }
                    )),
            )
            .children(q.items.iter().enumerate().map(|(i, it)| {
                let (fr, fs) = (fr.clone(), fs.clone());
                let (a, b) = (it.id.clone(), it.id.clone());
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .child(
                        div()
                            .w(px(14.))
                            .text_size(px(11.))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child((i + 1).to_string()),
                    )
                    .child(div().flex_1().child(it.text.clone()))
                    .child(
                        div()
                            .id(("q-now", i))
                            .role(Role::Button)
                            .aria_label("Send now")
                            .child("⚡")
                            .when_some(fs, move |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| f(b.clone(), w, cx))
                            }),
                    )
                    .child(
                        div()
                            .id(("q-x", i))
                            .role(Role::Button)
                            .aria_label("Remove")
                            .child("×")
                            .when_some(fr, move |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| f(a.clone(), w, cx))
                            }),
                    )
            }))
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
