//! Composer parts for GPUI (plan #27, chunk 3), drawn from `oxyn_core::composer::parts`:
//! `VoiceRecorderEl`, `DictationEl`, `MentionListEl`, `ModeSwitchEl`, `PermissionEl`,
//! `ToolsMenuEl`, `QueueStackEl` and `SteeringEl`.
//!
//! Elements are `RenderOnce`: the host owns the model and the clock, routes presses back through
//! the `on_*` callbacks and re-renders. Gaps against Dioxus (listed in the PR): no audio capture
//! (the level meter reads `levels`), text entry belongs to the host's input (the dictation field,
//! mention query, queue edit and steering box are display-only here), popovers draw in place, and
//! there is no blink / wave / pop motion.

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

/// Dictation field: text plus muted interim words and a mic toggle (index 0).
#[derive(IntoElement)]
pub struct DictationEl {
    text: SharedString,
    interim: SharedString,
    listening: bool,
    on_mic: Option<OnIndex>,
    theme: &'static Theme,
}

impl DictationEl {
    /// Text, interim words (from `interim`) and whether it is listening.
    pub fn new(
        text: impl Into<SharedString>,
        interim: impl Into<SharedString>,
        listening: bool,
    ) -> Self {
        Self {
            text: text.into(),
            interim: interim.into(),
            listening,
            on_mic: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_mic, "The mic was pressed.");
    themed!();
}

impl RenderOnce for DictationEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let f = self.on_mic;
        shell(t)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.))
                    .child(self.text)
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(self.interim),
                    ),
            )
            .child(
                div()
                    .id("dict-mic")
                    .role(Role::Button)
                    .aria_label(if self.listening {
                        "Stop dictation"
                    } else {
                        "Dictate"
                    })
                    .size(px(30.))
                    .rounded(px(15.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(self.listening, |d| {
                        d.bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                    })
                    .child(if self.listening { "■" } else { "🎙" })
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                    }),
            )
    }
}

/// Mention / command suggestion list (from `rank`), grouped, with a highlighted row.
#[derive(IntoElement)]
pub struct MentionListEl {
    items: Vec<Suggestion>,
    highlight: usize,
    on_pick: Option<OnIndex>,
    theme: &'static Theme,
}

impl MentionListEl {
    /// Ranked suggestions and the highlighted index.
    pub fn new(items: Vec<Suggestion>, highlight: usize) -> Self {
        Self {
            items,
            highlight,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_pick, "A row was picked (index into the list).");
    themed!();
}

impl RenderOnce for MentionListEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let f = self.on_pick;
        let mut last = String::new();
        let mut rows: Vec<gpui::AnyElement> = vec![];
        for (i, s) in self.items.into_iter().enumerate() {
            if s.group != last {
                last = s.group.clone();
                rows.push(
                    div()
                        .px(px(8.))
                        .pt(px(6.))
                        .text_size(px(11.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(s.group.to_uppercase())
                        .into_any_element(),
                );
            }
            let f = f.clone();
            rows.push(
                div()
                    .id(("men", i))
                    .role(Role::ListBoxOption)
                    .aria_selected(i == self.highlight)
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .when(i == self.highlight, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(s.label))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(s.hint),
                    )
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
                    .into_any_element(),
            );
        }
        div()
            .id("oxyn-mentions")
            .role(Role::ListBox)
            .flex()
            .flex_col()
            .p(px(6.))
            .w(px(320.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceOverlay))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .children(rows)
    }
}

// ───────────────────────────────────────── Mode switch ──

/// Segmented mode switch with a pending (“next turn”) marker.
#[derive(IntoElement)]
pub struct ModeSwitchEl {
    model: ModeSwitch,
    on_select: Option<OnIndex>,
    theme: &'static Theme,
}

impl ModeSwitchEl {
    /// From the model.
    pub fn new(model: ModeSwitch) -> Self {
        Self {
            model,
            on_select: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(
        on_select,
        "A mode was pressed (feed to `ModeSwitch::select`)."
    );
    themed!();
}

impl RenderOnce for ModeSwitchEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let f = self.on_select;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .id("oxyn-mode")
                    .role(Role::RadioGroup)
                    .flex()
                    .gap(px(2.))
                    .p(px(3.))
                    .rounded(px(10.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .children(m.modes.iter().enumerate().map(|(i, md)| {
                        let on = i == m.current;
                        let pend = m.pending == Some(i);
                        let f = f.clone();
                        div()
                            .id(("mode", i))
                            .role(Role::RadioButton)
                            .aria_toggled(if on { Toggled::True } else { Toggled::False })
                            .aria_label(SharedString::from(md.label.clone()))
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .h(px(28.))
                            .px(px(12.))
                            .rounded(px(7.))
                            .when(on, |d| d.bg(c(t, ColorRole::SurfaceRaised)))
                            .when(pend, |d| {
                                d.border_1()
                                    .border_dashed()
                                    .border_color(c(t, ColorRole::BorderStrong))
                            })
                            .text_color(c(
                                t,
                                if on {
                                    ColorRole::FgDefault
                                } else {
                                    ColorRole::FgMuted
                                },
                            ))
                            .child(md.label.clone())
                            .when(pend, |d| {
                                d.child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(c(t, ColorRole::StatusInfoFg))
                                        .child("next"),
                                )
                            })
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            })
                    })),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::FgSubtle))
                    .child(m.mode().placeholder.clone()),
            )
    }
}

/// Permission presets with risk marks, locked rows above the ceiling and a confirm row.
#[derive(IntoElement)]
pub struct PermissionEl {
    model: Permissions,
    on_choose: Option<OnText>,
    on_confirm: Option<OnIndex>,
    theme: &'static Theme,
}

impl PermissionEl {
    /// From the model.
    pub fn new(model: Permissions) -> Self {
        Self {
            model,
            on_choose: None,
            on_confirm: None,
            theme: &Theme::DARK,
        }
    }
    on_text!(
        on_choose,
        "A preset was pressed (feed to `Permissions::choose`)."
    );
    on_index!(on_confirm, "Confirm row: 1 = allow, 0 = cancel.");
    themed!();
}

impl RenderOnce for PermissionEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.model;
        let f = self.on_choose;
        let fc = self.on_confirm;
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .w(px(360.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .children(p.presets.iter().enumerate().map(|(i, pr)| {
                let locked = pr.risk > p.ceiling;
                let on = pr.id == p.current;
                let f = f.clone();
                let id = pr.id.clone();
                div()
                    .id(("perm", i))
                    .role(Role::RadioButton)
                    .aria_toggled(if on { Toggled::True } else { Toggled::False })
                    .aria_label(SharedString::from(format!(
                        "{}, {} risk",
                        pr.label,
                        pr.risk.id()
                    )))
                    .flex()
                    .gap(px(10.))
                    .p(px(8.))
                    .rounded(px(9.))
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .when(locked, |d| d.opacity(0.5))
                    .child(
                        div()
                            .w(px(16.))
                            .text_color(c(
                                t,
                                match pr.risk {
                                    Risk::High => ColorRole::StatusDangerFg,
                                    Risk::Medium => ColorRole::StatusWarningFg,
                                    Risk::Low => ColorRole::FgMuted,
                                },
                            ))
                            .child(if locked {
                                "🔒"
                            } else if pr.risk == Risk::High {
                                "!"
                            } else {
                                "•"
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(pr.label.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(if locked {
                                        "Turned off by your workspace admin.".to_string()
                                    } else {
                                        pr.desc.clone()
                                    }),
                            ),
                    )
                    .when_some(f.filter(|_| !locked), |d, f| {
                        d.cursor_pointer()
                            .on_click(move |_, w, cx| f(id.clone(), w, cx))
                    })
            }))
            .when(p.confirming.is_some(), |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .p(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::StatusDangerBorder))
                        .bg(c(t, ColorRole::StatusDangerBg))
                        .child(div().flex_1().child("Give the agent full access?"))
                        .child(btn(t, "perm-no", "Cancel", fc.clone(), 0))
                        .child(btn(t, "perm-yes", "Allow", fc.clone(), 1)),
                )
            })
    }
}

/// Tools list grouped by source with switches; tools that need sign-in show Connect.
#[derive(IntoElement)]
pub struct ToolsMenuEl {
    tools: Vec<Tool>,
    query: String,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl ToolsMenuEl {
    /// Tools and the search query.
    pub fn new(tools: Vec<Tool>, query: impl Into<String>) -> Self {
        Self {
            tools,
            query: query.into(),
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(
        on_toggle,
        "A switch or Connect was pressed (index into the tools)."
    );
    themed!();
}

impl RenderOnce for ToolsMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let groups = tool_groups(&self.tools, &self.query);
        let f = self.on_toggle;
        let tools = self.tools;
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .w(px(300.))
            .p(px(6.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceOverlay))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .children(groups.into_iter().map(|(g, ix)| {
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(8.))
                            .pt(px(6.))
                            .text_size(px(11.))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(g.to_uppercase()),
                    )
                    .children(ix.into_iter().map(|i| {
                        let tl = &tools[i];
                        let f = f.clone();
                        div()
                            .id(("tool", i))
                            .role(Role::Switch)
                            .aria_toggled(if tl.on { Toggled::True } else { Toggled::False })
                            .aria_label(SharedString::from(tl.label.clone()))
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(px(32.))
                            .px(px(8.))
                            .child(tl.label.clone())
                            .child(if tl.needs_auth {
                                div()
                                    .text_size(px(12.))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child("Connect")
                            } else {
                                div()
                                    .w(px(30.))
                                    .h(px(18.))
                                    .rounded(px(9.))
                                    .p(px(2.))
                                    .flex()
                                    .when(tl.on, |d| d.justify_end())
                                    .bg(c(
                                        t,
                                        if tl.on {
                                            ColorRole::FgDefault
                                        } else {
                                            ColorRole::BorderStrong
                                        },
                                    ))
                                    .child(
                                        div()
                                            .size(px(14.))
                                            .rounded(px(7.))
                                            .bg(c(t, ColorRole::BgBase)),
                                    )
                            })
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            })
                    }))
            }))
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

/// Agent steering: the running step and steering notes with their phase.
#[derive(IntoElement)]
pub struct SteeringEl {
    step: SharedString,
    notes: Vec<Steer>,
    now: ClockMs,
    step_len: Duration,
    on_stop: Option<OnIndex>,
    theme: &'static Theme,
}

impl SteeringEl {
    /// Current step label, notes, clock and step length.
    pub fn new(
        step: impl Into<SharedString>,
        notes: Vec<Steer>,
        now: ClockMs,
        step_len: Duration,
    ) -> Self {
        Self {
            step: step.into(),
            notes,
            now,
            step_len,
            on_stop: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_stop, "Stop was pressed.");
    themed!();
}

impl RenderOnce for SteeringEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w(px(440.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                shell(t)
                    .child(
                        div()
                            .size(px(8.))
                            .rounded(px(4.))
                            .bg(c(t, ColorRole::StatusSuccessFg)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.step),
                    )
                    .child(btn(t, "steer-stop", "Stop", self.on_stop, 0)),
            )
            .children(self.notes.iter().map(|n| {
                let ph = n.phase(self.now, self.step_len);
                let (lab, role) = match ph {
                    SteerPhase::Pending => ("Waiting for next step", ColorRole::FgMuted),
                    SteerPhase::Delivered => ("Read by agent", ColorRole::FgDefault),
                    SteerPhase::Applied => ("Applied", ColorRole::StatusSuccessFg),
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(6.))
                    .rounded(px(10.))
                    .border_1()
                    .when(ph == SteerPhase::Pending, |d| d.border_dashed())
                    .border_color(c(
                        t,
                        if ph == SteerPhase::Applied {
                            ColorRole::StatusSuccessBorder
                        } else {
                            ColorRole::BorderStrong
                        },
                    ))
                    .child(div().flex_1().child(n.text.clone()))
                    .child(div().text_size(px(11.5)).text_color(c(t, role)).child(lab))
            }))
    }
}
