//! Blocks (Oxyn Pro) in GPUI as one live view: Live session, Mission control, Mission
//! timeline, Run inspector, Background agents, Agent routines, MCP servers, Knowledge base, Eval
//! suite, Agent builder, Agent gallery and AI insight card. Every variant runs on the same
//! `oxyn_core::blocks` models as the Dioxus blocks, so presses and keys behave the same in both
//! renderers. The view keeps its own clock (a tick every 100 ms while it lives); a host can also
//! drive it with [`BlocksView::advance`]. Text fields show as text; the samples' quick picks
//! (suggestions, prompts, presets) stand in for typing.

use std::time::Duration;

use gpui::{
    App, ClickEvent, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, Stateful, Styled, Window, div, prelude::*,
    px, relative,
};
use oxyn_core::blocks::{
    bgagents::{self, BgAgents, Phase as BaPhase},
    builder::{self, Builder},
    dur,
    evals::{self, Evals, Out},
    gallery::{self, Flow, Gallery},
    insight::{self, Insight},
    inspector::{Inspector, Kind as SpanKind, SPANS, SpanSt},
    kb::{self, Kb, View as KbView},
    mcp::{self, Mcp, Sheet, SrvSt},
    mission::{self, Mission, St},
    routines::{self, Routines, Tick},
    session::{self, Session, StepSt},
    thousands,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::aikit::{bar, c, hue, mark, pill, row, text, title};

/// Which block the view draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockVariant {
    /// Live session.
    LiveSession,
    /// Mission control.
    MissionControl,
    /// Mission timeline.
    MissionTimeline,
    /// Run inspector.
    RunInspector,
    /// Background agents.
    BackgroundAgents,
    /// Agent routines.
    Routines,
    /// MCP servers.
    Mcp,
    /// Knowledge base.
    KnowledgeBase,
    /// Eval suite.
    Evals,
    /// Agent builder.
    Builder,
    /// Agent gallery.
    Gallery,
    /// AI insight card.
    Insight,
}

impl BlockVariant {
    /// Every variant, in gallery order.
    pub const ALL: [BlockVariant; 12] = [
        Self::LiveSession,
        Self::MissionControl,
        Self::MissionTimeline,
        Self::RunInspector,
        Self::BackgroundAgents,
        Self::Routines,
        Self::Mcp,
        Self::KnowledgeBase,
        Self::Evals,
        Self::Builder,
        Self::Gallery,
        Self::Insight,
    ];

    /// The page slug.
    pub fn slug(self) -> &'static str {
        oxyn_core::blocks::BLOCK_SLUGS[Self::ALL.iter().position(|&v| v == self).unwrap_or(0)]
    }
}

/// The state of every block on one clock; pure, so presses and keys are testable without a
/// window.
#[derive(Clone, Debug)]
pub struct BlocksState {
    /// Clock, ms.
    pub now: u64,
    /// Live session.
    pub session: Session,
    /// Mission control.
    pub control: Mission,
    /// Mission timeline.
    pub timeline: Mission,
    /// Run inspector.
    pub inspector: Inspector,
    /// Background agents.
    pub bg: BgAgents,
    /// Agent routines.
    pub routines: Routines,
    /// MCP servers.
    pub mcp: Mcp,
    /// Knowledge base.
    pub kb: Kb,
    /// Eval suite.
    pub evals: Evals,
    /// Agent builder.
    pub builder: Builder,
    /// Agent gallery.
    pub gallery: Gallery,
    /// AI insight card.
    pub insight: Insight,
    /// The last thing said (the live region of the web blocks).
    pub said: String,
}

impl Default for BlocksState {
    fn default() -> Self {
        let now = 100_000;
        Self {
            now,
            session: Session::new(0, now),
            control: Mission::new(0, false, now),
            timeline: Mission::new(0, false, now),
            inspector: Inspector::new(0, 100, true, now),
            bg: BgAgents::new(false, now),
            routines: Routines::new(false, now),
            mcp: Mcp::new(),
            kb: Kb::new(false, now),
            evals: Evals::new(false, false, now),
            builder: Builder::new(false, now),
            gallery: Gallery::new(2, false),
            insight: Insight::new(true, false, now),
            said: String::new(),
        }
    }
}

impl BlocksState {
    /// Routes a press id to the model of `v`. Ids that start a flow with text (`ls-send`,
    /// `ba-start`, `kb-search`) use the samples' preset text.
    pub fn press(&mut self, v: BlockVariant, id: &str) {
        let n = self.now;
        let out = match v {
            BlockVariant::LiveSession if id == "ls-send" => {
                self.session.send("@Writer keep the copy under 40 words", n)
            }
            BlockVariant::LiveSession => self.session.press(id, n),
            BlockVariant::MissionControl => self.control.press(id, n),
            BlockVariant::MissionTimeline => self.timeline.press(id, n),
            BlockVariant::RunInspector => self.inspector.press(id, n),
            BlockVariant::BackgroundAgents if id == "ba-start" => {
                self.bg.start("Add request ids to every log line", n)
            }
            BlockVariant::BackgroundAgents => self.bg.press(id, n),
            BlockVariant::Routines => self.routines.press(id, n),
            BlockVariant::Mcp => self.mcp.press(id, n),
            BlockVariant::KnowledgeBase => self.kb.press(id, n),
            BlockVariant::Evals => self.evals.press(id, n),
            BlockVariant::Builder => self.builder.press(id, n),
            BlockVariant::Gallery => self.gallery.press(id, n),
            BlockVariant::Insight => self.insight.press(id, n),
        };
        if let Some(s) = out {
            self.said = s;
        }
    }

    /// Routes a key (`left`, `enter`, `escape`…) to the main keyboard part of `v`.
    pub fn key(&mut self, v: BlockVariant, key: &str, cmd: bool) {
        let n = self.now;
        let k = oxyn_core::blocks::norm_key(key);
        let out = match v {
            BlockVariant::LiveSession => {
                if matches!(k, "ArrowUp" | "ArrowDown") {
                    self.session.key("msgs", k, n)
                } else {
                    self.session.key("tabs", k, n)
                }
            }
            BlockVariant::MissionControl => self.control.key(k, n),
            BlockVariant::MissionTimeline => self.timeline.key(k, n),
            BlockVariant::RunInspector => self.inspector.key(k, n),
            BlockVariant::BackgroundAgents => self.bg.key(k, n),
            BlockVariant::Routines => self.routines.key(k, n),
            BlockVariant::Mcp => self.mcp.key(k, cmd, n),
            BlockVariant::KnowledgeBase => self.kb.key(k, n),
            BlockVariant::Evals => self.evals.key(k, n),
            BlockVariant::Builder => self.builder.key(k, n),
            BlockVariant::Gallery => self.gallery.key(k, n),
            BlockVariant::Insight => self.insight.key(k, n),
        };
        if let Some(s) = out {
            self.said = s;
        }
    }
}

/// The blocks as one live view.
pub struct BlocksView {
    variant: BlockVariant,
    state: BlocksState,
    dark: bool,
    focus: FocusHandle,
}

impl Focusable for BlocksView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl BlocksView {
    /// A view of `variant`, focused so the keys work at once, ticking its own clock.
    pub fn new(variant: BlockVariant, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if this
                    .update(cx, |v, cx| {
                        v.state.now += 100;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            variant,
            state: BlocksState::default(),
            dark: true,
            focus,
        }
    }

    /// Switch variant.
    pub fn set_variant(&mut self, v: BlockVariant, cx: &mut Context<Self>) {
        self.variant = v;
        cx.notify();
    }

    /// Light or dark.
    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        cx.notify();
    }

    /// Moves the clock on by `ms`.
    pub fn advance(&mut self, ms: u64, cx: &mut Context<Self>) {
        self.state.now += ms;
        cx.notify();
    }

    /// The state (for hosts and tests).
    pub fn state(&self) -> &BlocksState {
        &self.state
    }

    fn theme(&self) -> &'static Theme {
        if self.dark {
            &Theme::DARK
        } else {
            &Theme::LIGHT
        }
    }

    fn btn(
        &self,
        cx: &mut Context<Self>,
        id: &str,
        label: impl Into<SharedString>,
        pri: bool,
    ) -> Stateful<Div> {
        let t = self.theme();
        let v = self.variant;
        let pid = id.to_string();
        let (bg, fg, bd) = if pri {
            (
                c(t, ColorRole::FgDefault),
                c(t, ColorRole::BgBase),
                c(t, ColorRole::FgDefault),
            )
        } else {
            (
                c(t, ColorRole::SurfaceBase),
                c(t, ColorRole::FgDefault),
                c(t, ColorRole::BorderDefault),
            )
        };
        div()
            .id(SharedString::from(format!("bk-{id}")))
            .h(px(28.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(5.))
            .flex_none()
            .rounded(px(8.))
            .border_1()
            .border_color(bd)
            .bg(bg)
            .text_color(fg)
            .text_size(px(12.5))
            .cursor_pointer()
            .hover(|d| d.opacity(0.85))
            .child(label.into())
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.state.press(v, &pid);
                cx.notify();
            }))
    }

    fn chip(
        &self,
        cx: &mut Context<Self>,
        id: &str,
        label: impl Into<SharedString>,
        on: bool,
    ) -> Stateful<Div> {
        let t = self.theme();
        self.btn(cx, id, label, false)
            .h(px(24.))
            .rounded(px(12.))
            .text_size(px(12.))
            .when(on, |d| {
                d.border_color(c(t, ColorRole::FgDefault))
                    .bg(c(t, ColorRole::SurfaceSelected))
            })
            .when(!on, |d| {
                d.text_color(c(t, ColorRole::FgMuted))
                    .border_color(c(t, ColorRole::BorderSubtle))
            })
    }

    /// A clickable row (no border) that presses `id`.
    fn hit(&self, cx: &mut Context<Self>, id: &str, on: bool) -> Stateful<Div> {
        let t = self.theme();
        let v = self.variant;
        let pid = id.to_string();
        div()
            .id(SharedString::from(format!("bk-{id}")))
            .flex()
            .items_center()
            .gap(px(8.))
            .min_w(px(0.))
            .px(px(8.))
            .py(px(6.))
            .rounded(px(8.))
            .cursor_pointer()
            .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
            .hover(|d| d.bg(c(t, ColorRole::SurfaceHover)))
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.state.press(v, &pid);
                cx.notify();
            }))
    }

    fn card(&self) -> Div {
        let t = self.theme();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .p(px(10.))
            .min_w(px(0.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceRaised))
    }

    fn status(&self, word: impl Into<SharedString>, h: &str) -> Div {
        let t = self.theme();
        let col = hue(t, h);
        row()
            .flex_none()
            .gap(px(4.))
            .child(div().size(px(7.)).rounded(px(4.)).bg(col))
            .child(div().text_size(px(12.)).text_color(col).child(word.into()))
    }

    fn sw(&self, cx: &mut Context<Self>, id: &str, on: bool) -> Stateful<Div> {
        let t = self.theme();
        let v = self.variant;
        let pid = id.to_string();
        div()
            .id(SharedString::from(format!("bk-sw-{id}")))
            .w(px(30.))
            .h(px(18.))
            .flex_none()
            .rounded(px(9.))
            .bg(if on {
                c(t, ColorRole::FgDefault)
            } else {
                c(t, ColorRole::BorderDefault)
            })
            .cursor_pointer()
            .child(
                div()
                    .size(px(14.))
                    .mt(px(2.))
                    .ml(px(if on { 14. } else { 2. }))
                    .rounded(px(7.))
                    .bg(c(t, ColorRole::BgBase)),
            )
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                this.state.press(v, &pid);
                cx.notify();
            }))
    }

    /// A brand mark on a small tile; on dark themes the tile is white so marks whose official
    /// colour is dark (Sentry, PostgreSQL) stay visible.
    fn logo_tile(&self, name: &str, size: f32) -> Div {
        let t = self.theme();
        let (bg, ink) = if self.dark {
            (gpui::white(), gpui::black())
        } else {
            (c(t, ColorRole::SurfaceBase), c(t, ColorRole::FgDefault))
        };
        div()
            .size(px(size + 12.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(7.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(bg)
            .child(mark(name, size, ink))
    }

    fn key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let m = &ev.keystroke.modifiers;
        let k = ev.keystroke.key.clone();
        self.state.key(self.variant, &k, m.platform || m.control);
        cx.notify();
    }

    fn session(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.session;
        let mut tabs = row().gap(px(4.)).flex_wrap();
        for (i, (name, brand, _)) in session::AGENTS.iter().enumerate() {
            let on = s.focus.unwrap_or(0) == i;
            let label = if i == 0 { "Everyone" } else { name };
            let mut ch = self.chip(cx, &format!("ls-agent-{i}"), label, on);
            if !brand.is_empty() {
                ch = ch.child(mark(brand, 12., c(t, ColorRole::FgDefault)));
            }
            tabs = tabs.child(ch);
        }
        let mut msgs = div().flex().flex_col().gap(px(4.)).min_w(px(0.)).flex_1();
        for (i, m) in s
            .messages(n)
            .iter()
            .enumerate()
            .rev()
            .take(7)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let fade = s.focus.is_some_and(|f| m.from != f && m.to != Some(f));
            let to =
                m.to.map_or("everyone".to_string(), |x| session::AGENTS[x].0.to_string());
            let (name, brand, _) = session::AGENTS[m.from];
            let mut body = div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .min_w(px(0.))
                .flex_1()
                .child(
                    row()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                        .child(text(
                            t,
                            format!("to {to} · {}", dur(m.at)),
                            ColorRole::FgSubtle,
                            11.5,
                        )),
                )
                .child(div().child(m.text.clone()));
            if m.asks {
                let mut ans = row().gap(px(6.));
                for (a, label) in session::ANSWERS.iter().enumerate() {
                    ans = ans.child(self.chip(cx, &format!("ls-answer-{a}"), *label, false));
                }
                body = body.child(ans);
            }
            let av = if brand.is_empty() {
                div()
                    .size(px(20.))
                    .rounded(px(10.))
                    .bg(c(t, ColorRole::FgDefault))
            } else {
                mark(brand, 18., c(t, ColorRole::FgDefault))
            };
            msgs = msgs.child(
                self.hit(cx, &format!("ls-msg-{i}"), s.picked == Some(i))
                    .items_start()
                    .when(fade, |d| d.opacity(0.32))
                    .child(av)
                    .child(body),
            );
        }
        if let Some(ty) = s.typing(n) {
            msgs = msgs.child(text(
                t,
                format!("{} is writing…", session::AGENTS[ty].0),
                ColorRole::FgSubtle,
                12.,
            ));
        }
        msgs = msgs.child(
            row()
                .child(
                    div()
                        .flex_1()
                        .h(px(30.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .rounded(px(8.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(text(
                            t,
                            "@Writer keep the copy under 40 words",
                            ColorRole::FgSubtle,
                            12.5,
                        )),
                )
                .child(self.btn(cx, "ls-send", "Send", true)),
        );
        let mut plan = div().flex().flex_col().gap(px(2.));
        for (i, (title_, ag, _, _)) in session::STEPS.iter().enumerate() {
            let st = s.step(i, n);
            let (h, w) = match st {
                StepSt::Done => ("green", "Done"),
                StepSt::Active => ("blue", "Running"),
                StepSt::Yours => ("amber", "Your turn"),
                StepSt::Todo => ("neutral", "To do"),
            };
            let dim = s.focus.is_some_and(|f| *ag != f);
            plan = plan.child(
                row()
                    .px(px(4.))
                    .py(px(4.))
                    .rounded(px(6.))
                    .when(matches!(st, StepSt::Active | StepSt::Yours), |d| {
                        d.bg(c(t, ColorRole::SurfaceSelected))
                    })
                    .when(dim, |d| d.opacity(0.3))
                    .child(self.status(w, h))
                    .child(div().flex_1().min_w(px(0.)).child(*title_))
                    .child(text(
                        t,
                        s.step_time(i, n).unwrap_or_default(),
                        ColorRole::FgSubtle,
                        11.5,
                    )),
            );
        }
        let mut ctx = div()
            .flex()
            .h(px(8.))
            .w_full()
            .rounded(px(4.))
            .overflow_hidden()
            .bg(c(t, ColorRole::SurfaceSelected));
        for (k, (_, tk)) in s.context(n).iter().enumerate() {
            ctx = ctx.child(
                div()
                    .h_full()
                    .w(relative((*tk as f64 / session::LIMIT as f64) as f32))
                    .bg(hue(t, ["neutral", "violet", "teal", "blue"][k])),
            );
        }
        let stat = s.status(n);
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        session::TASK,
                        format!(
                            "{stat} · {} of {} steps · {}",
                            s.done(n),
                            session::STEPS.len(),
                            dur(s.t(n))
                        ),
                    ))
                    .child(self.btn(
                        cx,
                        "ls-pause",
                        if s.paused.is_some() {
                            "Resume"
                        } else {
                            "Pause"
                        },
                        false,
                    )),
            )
            .child(tabs)
            .child(
                row().items_start().gap(px(12.)).child(msgs).child(
                    div()
                        .w(px(280.))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .child(
                            self.card()
                                .child(text(t, "PLAN", ColorRole::FgSubtle, 11.))
                                .child(plan),
                        )
                        .child(
                            self.card()
                                .child(
                                    row()
                                        .child(text(t, "CONTEXT", ColorRole::FgSubtle, 11.))
                                        .child(mark(
                                            session::CONTEXT_MODEL.0,
                                            12.,
                                            c(t, ColorRole::FgDefault),
                                        )),
                                )
                                .child(ctx)
                                .child(text(t, s.context_line(n), ColorRole::FgMuted, 12.)),
                        ),
                ),
            )
    }

    fn mission(&self, cx: &mut Context<Self>, lanes: bool) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let m = if lanes {
            &self.state.timeline
        } else {
            &self.state.control
        };
        let counts = m.counts(n);
        let mut filt = row().gap(px(4.));
        for (i, f) in mission::FILTERS.iter().enumerate() {
            filt = filt.child(self.chip(
                cx,
                &format!("mi-f-{i}"),
                format!("{f} {}", counts[i]),
                m.filter == i,
            ));
        }
        let curve = m.curve(n);
        let max = curve
            .iter()
            .map(|b| b.iter().sum::<u32>())
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let mut bars = row().items_end().gap(px(3.)).h(px(56.));
        for b in &curve {
            let mut col = div().flex_1().flex().flex_col().justify_end().h_full();
            for (k, h) in ["green", "amber", "red"].iter().enumerate().rev() {
                if b[k] > 0 {
                    col = col.child(
                        div()
                            .w_full()
                            .h(relative(b[k] as f32 / max))
                            .rounded(px(2.))
                            .bg(hue(t, h)),
                    );
                }
            }
            bars =
                bars.child(col.child(div().h(px(1.)).w_full().bg(c(t, ColorRole::BorderSubtle))));
        }
        let mut middle = div().flex().flex_col().gap(px(6.)).flex_1().min_w(px(0.));
        if lanes {
            for l in m.leads() {
                if !m.shows(l, n) {
                    continue;
                }
                let mut rows = vec![l];
                rows.extend(
                    m.children(l)
                        .into_iter()
                        .filter(|&c_| m.passes(c_, n) || m.filter == 0),
                );
                for i in rows {
                    let a = &mission::AGENTS[i];
                    let st = m.st(i, n);
                    let mut track = div()
                        .relative()
                        .h(px(22.))
                        .flex_1()
                        .rounded(px(6.))
                        .bg(c(t, ColorRole::SurfaceRaised));
                    if let Some((s0, e0)) = m.lane(i, n) {
                        track = track.child(
                            self.btn(cx, &format!("mi-open-{i}"), "", false)
                                .absolute()
                                .top(px(3.))
                                .h(px(16.))
                                .left(relative(s0 as f32))
                                .w(relative((e0 - s0).max(0.01) as f32))
                                .px(px(0.))
                                .rounded(px(4.))
                                .border_0()
                                .bg(hue(t, st.hue()).opacity(0.35)),
                        );
                    }
                    middle = middle.child(
                        row()
                            .child(
                                row()
                                    .w(px(140.))
                                    .pl(px(if a.parent.is_some() { 16. } else { 0. }))
                                    .child(mark(a.brand, 14., c(t, ColorRole::FgDefault)))
                                    .child(a.name),
                            )
                            .child(track)
                            .child(div().w(px(80.)).child(self.status(st.word(), st.hue()))),
                    );
                }
            }
        } else {
            for l in m.leads() {
                if !m.shows(l, n) {
                    continue;
                }
                let a = &mission::AGENTS[l];
                let st = m.st(l, n);
                let mut card = self
                    .card()
                    .when(m.lead == l, |d| d.border_color(c(t, ColorRole::FgMuted)))
                    .child(
                        self.hit(cx, &format!("mi-lead-{l}"), false)
                            .px(px(0.))
                            .py(px(0.))
                            .child(mark(a.brand, 14., c(t, ColorRole::FgDefault)))
                            .child(
                                div()
                                    .flex_1()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(a.name),
                            )
                            .child(self.status(st.word(), st.hue())),
                    )
                    .child(text(t, a.task, ColorRole::FgMuted, 12.5))
                    .child(text(
                        t,
                        format!(
                            "{} · {} · {} sub-agents",
                            a.model,
                            m.time(l, n),
                            m.children(l).len()
                        ),
                        ColorRole::FgSubtle,
                        11.5,
                    ));
                if m.lead == l {
                    for c_ in m.children(l) {
                        let ca = &mission::AGENTS[c_];
                        let cs = m.st(c_, n);
                        card = card.child(
                            row()
                                .pl(px(12.))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .child(format!("{} · {}", ca.name, ca.task)),
                                )
                                .child(self.status(cs.word(), cs.hue())),
                        );
                    }
                }
                middle = middle.child(card);
            }
        }
        let mut inbox = self.card().w(px(270.)).flex_none().child(text(
            t,
            format!("NEEDS YOU · {}", m.inbox(n).len()),
            ColorRole::FgSubtle,
            11.,
        ));
        for i in m.inbox(n) {
            let a = &mission::AGENTS[i];
            let st = m.st(i, n);
            inbox = inbox.child(
                self.hit(cx, &format!("mi-open-{i}"), m.open == Some(i))
                    .child(div().flex_1().min_w(px(0.)).child(a.name))
                    .child(self.status(st.word(), st.hue())),
            );
            if m.open == Some(i) {
                let (ok, no) = if st == St::Waiting {
                    (("mi-ok-", "Approve"), ("mi-no-", "Decline"))
                } else {
                    (("mi-retry-", "Retry"), ("mi-dismiss-", "Dismiss"))
                };
                inbox = inbox
                    .child(text(t, m.note(i, n), ColorRole::FgMuted, 12.))
                    .child(
                        row()
                            .child(self.btn(cx, &format!("{}{i}", ok.0), ok.1, true))
                            .child(self.btn(cx, &format!("{}{i}", no.0), no.1, false)),
                    );
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        if lanes {
                            "Mission timeline"
                        } else {
                            "Mission control"
                        },
                        m.summary(n),
                    ))
                    .child(filt),
            )
            .child(self.card().child(bars))
            .child(row().items_start().gap(px(12.)).child(middle).child(inbox))
    }

    fn inspector(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let r = &self.state.inspector;
        let nums = r.numbers(n);
        let mut numbers = row().gap(px(16.));
        for (k, v) in [
            ("Scope", nums.of.clone()),
            ("Time", nums.time.clone()),
            ("Calls", nums.calls.to_string()),
            (
                "Tokens",
                format!(
                    "{} / {}",
                    thousands(nums.tokens.0),
                    thousands(nums.tokens.1)
                ),
            ),
            ("Cost", nums.cost.clone()),
            ("Errors", nums.errors.to_string()),
        ] {
            numbers = numbers.child(
                div()
                    .flex()
                    .flex_col()
                    .child(text(t, k, ColorRole::FgSubtle, 11.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(v)),
            );
        }
        let mut tree = div().flex().flex_col().gap(px(8.));
        for a in r.agents(n) {
            let sp = &SPANS[a];
            let (w, h) = r.agent_st(a, n);
            let fade = r.selected.is_some_and(|s| s != a);
            let mut card = self
                .card()
                .ml(px(Inspector::depth(a) as f32 * 28.))
                .when(fade, |d| d.opacity(0.38))
                .child(
                    self.hit(cx, &format!("ri-agent-{a}"), r.selected == Some(a))
                        .px(px(0.))
                        .py(px(0.))
                        .child(mark(sp.brand, 16., c(t, ColorRole::FgDefault)))
                        .child(
                            div()
                                .flex_1()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("{} · {}", sp.name, sp.model)),
                        )
                        .child(self.status(w, h))
                        .child(text(t, r.time(a, n), ColorRole::FgSubtle, 11.5)),
                )
                .child(text(t, sp.args, ColorRole::FgMuted, 12.5));
            for ci in r.calls(a, n) {
                let cs = &SPANS[ci];
                let st = r.st(ci, n);
                let (l, bw) = r.bar(ci, n);
                let col = match st {
                    SpanSt::Failed => hue(t, "red"),
                    SpanSt::Running => hue(t, "blue"),
                    _ if cs.kind == SpanKind::Llm => hue(t, "violet"),
                    _ => c(t, ColorRole::FgMuted),
                };
                let track = div()
                    .relative()
                    .h(px(8.))
                    .flex_1()
                    .rounded(px(4.))
                    .bg(c(t, ColorRole::SurfaceSelected))
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(relative(l as f32))
                            .w(relative(bw.max(0.01) as f32))
                            .rounded(px(4.))
                            .bg(col),
                    );
                card = card.child(
                    self.hit(cx, &format!("ri-call-{ci}"), r.call == Some(ci))
                        .child(div().w(px(150.)).child(cs.name))
                        .child(track)
                        .child(div().w(px(50.)).child(text(
                            t,
                            r.time(ci, n),
                            ColorRole::FgSubtle,
                            11.5,
                        ))),
                );
                if r.call == Some(ci) {
                    let out = if st == SpanSt::Failed {
                        cs.error
                    } else {
                        cs.result
                    };
                    card = card.child(
                        div()
                            .pl(px(26.))
                            .child(text(
                                t,
                                format!("Input {}", cs.args),
                                ColorRole::FgMuted,
                                12.,
                            ))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(if st == SpanSt::Failed {
                                        hue(t, "red")
                                    } else {
                                        c(t, ColorRole::FgDefault)
                                    })
                                    .child(out),
                            ),
                    );
                }
            }
            tree = tree.child(card);
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, SPANS[0].args, r.title(n)))
                    .child(self.btn(cx, "ri-replay", "Replay", false)),
            )
            .child(self.card().child(numbers))
            .child(tree)
    }

    fn bg_agents(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let b = &self.state.bg;
        let (r, br, md) = b.pick;
        let mut chips = row().gap(px(6.));
        for (i, (name, _)) in bgagents::REPOS.iter().enumerate() {
            chips = chips.child(self.chip(cx, &format!("ba-repo-{i}"), *name, r == i));
        }
        for (i, (brand, name)) in bgagents::MODELS.iter().enumerate() {
            chips = chips.child(
                self.chip(cx, &format!("ba-model-{i}"), *name, md == i)
                    .child(mark(brand, 12., c(t, ColorRole::FgDefault))),
            );
        }
        let counts = b.counts(n);
        let mut filt = row().gap(px(4.));
        for (i, f) in bgagents::FILTERS.iter().enumerate() {
            filt = filt.child(self.chip(
                cx,
                &format!("ba-f-{i}"),
                format!("{f} {}", counts[i]),
                b.filter == i,
            ));
        }
        let mut list = div().flex().flex_col().gap(px(8.));
        for tk in b.visible(n) {
            let ph = BgAgents::phase(tk, n);
            let (lab, h) = ph.label();
            let id = tk.id;
            let mut acts = row().gap(px(6.));
            if ph.busy() {
                acts = acts.child(self.btn(cx, &format!("ba-cancel-{id}"), "Cancel", false));
            }
            if matches!(ph, BaPhase::Failed(_) | BaPhase::Cancelled) {
                acts = acts.child(self.btn(cx, &format!("ba-retry-{id}"), "Retry", false));
            }
            if ph == BaPhase::Ready {
                acts = match BgAgents::pr(tk, n) {
                    None => acts.child(self.btn(
                        cx,
                        &format!("ba-pr-{id}"),
                        "Create pull request",
                        true,
                    )),
                    Some(Err(())) => {
                        acts.child(text(t, "Creating pull request…", ColorRole::FgMuted, 12.))
                    }
                    Some(Ok(p)) => acts
                        .child(pill(t, format!("#{p}"), "neutral"))
                        .child(self.btn(cx, &format!("ba-merge-{id}"), "Merge", true)),
                };
            }
            if !ph.busy() {
                acts = acts.child(self.btn(cx, &format!("ba-archive-{id}"), "Archive", false));
            }
            let mut card = self
                .card()
                .child(
                    self.hit(cx, &format!("ba-open-{id}"), false)
                        .px(px(0.))
                        .py(px(0.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(tk.title.clone()),
                        ),
                )
                .child(
                    row()
                        .child(mark("github", 12., c(t, ColorRole::FgDefault)))
                        .child(text(
                            t,
                            format!(
                                "{} · {} · {}",
                                bgagents::REPOS[tk.repo].0,
                                bgagents::REPOS[tk.repo].1
                                    [tk.branch.min(bgagents::REPOS[tk.repo].1.len() - 1)],
                                bgagents::MODELS[tk.model].1
                            ),
                            ColorRole::FgMuted,
                            12.,
                        )),
                )
                .child(
                    row()
                        .child(self.status(lab, h))
                        .child(text(t, BgAgents::step_line(tk, n), ColorRole::FgMuted, 12.))
                        .child(text(t, BgAgents::time(tk, n).0, ColorRole::FgSubtle, 12.)),
                )
                .child(acts);
            if tk.archived.is_some() {
                card = self.card().child(
                    row()
                        .justify_between()
                        .child(text(
                            t,
                            format!("Archived “{}”", tk.title),
                            ColorRole::FgMuted,
                            12.5,
                        ))
                        .child(self.btn(cx, &format!("ba-undo-{id}"), "Undo", false)),
                );
            } else if b.open == Some(id) {
                let mut steps = div().flex().flex_col().gap(px(2.));
                for (k, st) in bgagents::STEPS.iter().enumerate() {
                    steps = steps.child(text(
                        t,
                        format!(
                            "{} {st}",
                            if matches!(ph, BaPhase::Ready | BaPhase::Merged)
                                || matches!(ph, BaPhase::Running(x) if x > k)
                            {
                                "✓"
                            } else {
                                "·"
                            }
                        ),
                        ColorRole::FgMuted,
                        12.,
                    ));
                }
                card = card.child(steps);
            }
            list = list.child(card);
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(title(
                t,
                "Background agents",
                "Tasks run on their own machines; you review the result.",
            ))
            .child(
                self.card()
                    .child(text(
                        t,
                        "Add request ids to every log line",
                        ColorRole::FgSubtle,
                        13.,
                    ))
                    .child(chips)
                    .child(
                        row()
                            .child(text(
                                t,
                                format!("Branch {}", bgagents::REPOS[r].1[br]),
                                ColorRole::FgMuted,
                                12.,
                            ))
                            .child(div().flex_1())
                            .child(self.btn(cx, "ba-start", "Start", true)),
                    ),
            )
            .child(filt)
            .child(list)
    }

    fn routines(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.routines;
        let mut list = div().flex().flex_col().gap(px(8.));
        for (i, r) in s.list.iter().enumerate() {
            let running = s.active(i, n).is_some();
            let mut ticks = row().gap(px(2.));
            for tk in s.history(i, n) {
                ticks = ticks.child(div().w(px(5.)).h(px(14.)).rounded(px(2.)).bg(
                    if tk == Tick::Skipped {
                        c(t, ColorRole::BorderDefault)
                    } else {
                        hue(t, tk.hue())
                    },
                ));
            }
            let mut marks = row().gap(px(4.));
            for src in &r.sources {
                marks = marks.child(mark(src, 13., c(t, ColorRole::FgDefault)));
            }
            marks = marks
                .child(text(t, "→", ColorRole::FgSubtle, 12.))
                .child(mark("slack", 13., c(t, ColorRole::FgDefault)))
                .child(text(t, r.dest.clone(), ColorRole::FgSubtle, 11.5))
                .child(mark(r.model.0, 13., c(t, ColorRole::FgDefault)));
            let (last, lt) = s.last_line(i, n);
            let mut card = self
                .card()
                .when(!r.enabled, |d| d.opacity(0.6))
                .child(
                    row()
                        .child(
                            self.hit(cx, &format!("ro-open-{i}"), s.open == Some(i))
                                .flex_1()
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(r.name.clone()),
                                ),
                        )
                        .child(self.btn(
                            cx,
                            &format!("ro-run-{i}"),
                            if running { "Stop" } else { "Run now" },
                            false,
                        ))
                        .child(self.sw(cx, &format!("ro-toggle-{i}"), r.enabled)),
                )
                .child(
                    row()
                        .child(text(
                            t,
                            format!("{} · {}", routines::schedule(r), s.next_line(i, n)),
                            ColorRole::FgMuted,
                            12.,
                        ))
                        .child(div().flex_1())
                        .child(self.status(last, lt.hue())),
                )
                .child(row().justify_between().child(marks).child(ticks));
            if let Some((_, label, f, failed)) = s.progress(i, n) {
                card = card
                    .child(bar(t, f, hue(t, if failed { "red" } else { r.hue })))
                    .child(text(t, label, ColorRole::FgMuted, 12.));
            }
            if s.open == Some(i) {
                let mut fr = row().gap(px(4.));
                for (k, f) in routines::FREQS.iter().enumerate() {
                    fr = fr.child(self.chip(cx, &format!("ro-freq-{i}-{k}"), *f, r.freq == k));
                }
                let mut rep = div().flex().flex_col();
                for l in s.report(i) {
                    rep = rep.child(text(t, format!("• {l}"), ColorRole::FgDefault, 12.5));
                }
                card = card
                    .child(rep)
                    .child(text(t, r.instruction.clone(), ColorRole::FgMuted, 12.5))
                    .child(fr);
            }
            list = list.child(card);
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        "Routines",
                        "Timed agents and where their results land.",
                    ))
                    .child(self.btn(cx, "ro-new", "New routine", false)),
            )
            .child(list)
    }

    fn mcp(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let m = &self.state.mcp;
        let (_, total) = m.totals(n);
        let scale = total.max(mcp::BUDGET) as f32;
        let mut meter = div()
            .relative()
            .flex()
            .h(px(8.))
            .w_full()
            .rounded(px(4.))
            .bg(c(t, ColorRole::SurfaceSelected));
        for (i, _) in m.servers.iter().enumerate() {
            let cost = m.cost(i, n);
            if cost > 0 {
                meter = meter.child(div().h_full().w(relative(cost as f32 / scale)).bg(hue(
                    t,
                    ["blue", "violet", "teal", "amber", "green", "neutral"][i % 6],
                )));
            }
        }
        meter = meter.child(
            div()
                .absolute()
                .top(px(-2.))
                .bottom(px(-2.))
                .w(px(2.))
                .left(relative(mcp::BUDGET as f32 / scale))
                .bg(c(t, ColorRole::FgDefault)),
        );
        let mut list = div().flex().flex_col().gap(px(6.));
        for (i, s) in m.servers.iter().enumerate() {
            let st = m.st(i, n);
            let (w, h) = st.label();
            let mut head = self
                .hit(cx, &format!("mcp-open-{i}"), m.open == Some(i))
                .flex_1()
                .child(self.logo_tile(s.brand, 16.))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(s.name),
                )
                .child(self.status(w, h))
                .child(pill(t, if s.local { "Local" } else { "Remote" }, "neutral"))
                .child(text(
                    t,
                    format!(
                        "{} of {} · {} tk",
                        m.on(i),
                        s.tools.len(),
                        thousands(m.cost(i, n))
                    ),
                    ColorRole::FgSubtle,
                    11.5,
                ));
            head = head.min_w(px(0.));
            let mut r_ = row().child(head);
            if st == SrvSt::Error {
                r_ = r_.child(self.btn(cx, &format!("mcp-retry-{i}"), "Retry", false));
            }
            if st == SrvSt::NeedsAuth {
                r_ = r_.child(self.btn(cx, &format!("mcp-signin-{i}"), "Sign in", true));
            }
            let mut card = self.card().child(r_);
            if st == SrvSt::Error {
                card = card.child(text(t, s.error, ColorRole::StatusDangerFg, 12.));
            }
            if m.open == Some(i) {
                for (k, tool) in s.tools.iter().enumerate() {
                    card = card.child(
                        row()
                            .pl(px(26.))
                            .child(self.sw(cx, &format!("mcp-tool-{i}-{k}"), tool.on))
                            .child(div().flex_1().child(tool.name))
                            .child(text(
                                t,
                                if tool.write { "Write" } else { "Read" },
                                ColorRole::FgMuted,
                                11.5,
                            ))
                            .child(text(t, tool.tokens.to_string(), ColorRole::FgSubtle, 11.5)),
                    );
                }
                card = card.child(
                    row()
                        .pl(px(26.))
                        .child(self.chip(
                            cx,
                            &format!("mcp-perm-{i}-0"),
                            "Ask every time",
                            !s.allow_reads,
                        ))
                        .child(self.chip(
                            cx,
                            &format!("mcp-perm-{i}-1"),
                            "Always allow reads",
                            s.allow_reads,
                        ))
                        .child(div().flex_1())
                        .child(self.btn(cx, &format!("mcp-remove-{i}"), "Remove", false)),
                );
            }
            list = list.child(card);
        }
        let mut out = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "MCP servers", m.header(n)))
                    .child(self.btn(cx, "mcp-add", "Add server", false)),
            )
            .child(meter)
            .child(text(
                t,
                m.meter_text(n),
                if total > mcp::BUDGET {
                    ColorRole::StatusWarningFg
                } else {
                    ColorRole::FgMuted
                },
                12.,
            ));
        if m.removed.as_ref().is_some_and(|r| n < r.2 + mcp::UNDO_MS) {
            out = out.child(
                row()
                    .justify_between()
                    .child(text(t, "Server removed", ColorRole::FgMuted, 12.5))
                    .child(self.btn(cx, "mcp-undo", "Undo", false)),
            );
        }
        if let Some(sheet) = m.sheet.clone() {
            let mut sh = self.card().child(
                row()
                    .justify_between()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Add a server"),
                    )
                    .child(self.btn(cx, "mcp-close", "Close", false)),
            );
            match sheet {
                Sheet::Browse(_) => {
                    for d in m.directory() {
                        let e = &mcp::DIRECTORY[d];
                        sh = sh.child(
                            self.hit(cx, &format!("mcp-pick-{d}"), false)
                                .child(mark(e.1, 16., c(t, ColorRole::FgDefault)))
                                .child(div().flex_1().child(e.0))
                                .child(text(t, e.2, ColorRole::FgMuted, 12.)),
                        );
                    }
                }
                Sheet::Authorize(_) => {
                    sh = sh.child(text(t, "Support agent wants to use this account. Simulated: no account is contacted.", ColorRole::FgMuted, 12.5)).child(self.btn(cx, "mcp-auth", "Authorize", true));
                }
                Sheet::Discover(d, _) => {
                    let (_, got, done) = m.discovered(n).unwrap_or((d, 0, false));
                    for tl in mcp::DIRECTORY[d].4.iter().take(got) {
                        sh = sh.child(text(
                            t,
                            format!("✓ {}", tl.0),
                            ColorRole::StatusSuccessFg,
                            12.5,
                        ));
                    }
                    if done {
                        sh = sh.child(self.btn(cx, "mcp-finish", "Add to agent", true));
                    }
                }
            }
            return out.child(sh);
        }
        out.child(list)
    }

    fn kb(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let k = &self.state.kb;
        let mut body = div().flex().flex_col().gap(px(6.));
        if k.view == KbView::Sources {
            for (i, s) in k.sources.iter().enumerate() {
                let (step, _, failed) = k.pipe(i, n);
                let mut r_ = row()
                    .child(mark(s.kind, 15., c(t, ColorRole::FgDefault)))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(s.name))
                            .child(text(
                                t,
                                format!("{} pages · {} chunks", s.pages, k.chunks(i, n)),
                                ColorRole::FgMuted,
                                12.,
                            )),
                    )
                    .child(
                        div()
                            .w(px(160.))
                            .child(bar(
                                t,
                                k.percent(i, n) as f64 / 100.0,
                                hue(
                                    t,
                                    if failed {
                                        "red"
                                    } else if step == 4 {
                                        "green"
                                    } else {
                                        "blue"
                                    },
                                ),
                            ))
                            .child(text(t, k.pipe_text(i, n), ColorRole::FgSubtle, 11.5)),
                    );
                if failed {
                    r_ = r_.child(self.btn(cx, &format!("kb-retry-{i}"), "Retry", false));
                }
                body = body.child(self.card().child(r_));
            }
        } else {
            let mut sugs = row().gap(px(6.));
            for (i, q) in kb::SUGGESTIONS.iter().enumerate() {
                sugs = sugs.child(self.chip(
                    cx,
                    &format!("kb-sug-{i}"),
                    *q,
                    k.query.as_deref() == Some(q),
                ));
            }
            body = body.child(sugs).child(
                row().child(self.sw(cx, "kb-rerank", k.rerank)).child(text(
                    t,
                    "Rerank",
                    ColorRole::FgMuted,
                    12.,
                )),
            );
            for (r, h) in k.results(n).iter().enumerate() {
                let (src, page, txt) = kb::CHUNKS[h.chunk];
                let moved = h.was as i64 - (r as i64 + 1);
                let mv = if k.rerank && moved != 0 {
                    format!(" · {}{}", if moved > 0 { "↑" } else { "↓" }, moved.abs())
                } else {
                    String::new()
                };
                body = body.child(
                    self.hit(cx, &format!("kb-open-{}", h.chunk), k.open == Some(h.chunk))
                        .items_start()
                        .child(div().w(px(18.)).child(format!("{}", r + 1)))
                        .child(
                            div()
                                .w(px(110.))
                                .flex_none()
                                .child(bar(t, h.score, c(t, ColorRole::FgDefault)))
                                .child(text(
                                    t,
                                    format!("{:.2}{mv}", h.score),
                                    ColorRole::FgSubtle,
                                    11.5,
                                )),
                        )
                        .child(div().flex_1().min_w(px(0.)).child(txt).child(text(
                            t,
                            format!("{} · p. {page}", k.sources.get(src).map_or("", |s| s.name)),
                            ColorRole::FgSubtle,
                            11.5,
                        ))),
                );
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "Support knowledge base", k.summary(n)))
                    .child(self.btn(cx, "kb-add", "Add source", false)),
            )
            .child(
                row()
                    .gap(px(4.))
                    .child(self.chip(cx, "kb-view-0", "Sources", k.view == KbView::Sources))
                    .child(self.chip(cx, "kb-view-1", "Test retrieval", k.view == KbView::Test)),
            )
            .child(body)
    }

    fn evals(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let e = &self.state.evals;
        let state = e.state(n);
        let (score, base) = e.score(n);
        let mut cats = div().flex().flex_col().gap(px(6.));
        for (ci, (name, _)) in evals::CATEGORIES.iter().enumerate() {
            let mut cells = row().gap(px(2.));
            for i in (0..e.cases.len()).filter(|&i| e.cases[i].cat == ci) {
                let o = e.out(i, n);
                let col = match o {
                    Some(Out::Pass) => hue(t, "green"),
                    Some(Out::Fail) => hue(t, "red"),
                    Some(Out::Error) => hue(t, "amber"),
                    None => c(t, ColorRole::BorderSubtle),
                };
                let reg = e.change(i, n) == Some(false);
                cells = cells.child(
                    div()
                        .size(px(12.))
                        .rounded(px(3.))
                        .border_1()
                        .border_color(col)
                        .when(reg, |d| d.bg(col)),
                );
            }
            let (_, _, _, _, pct) = e.cat(ci, n);
            cats = cats.child(
                row()
                    .child(div().w(px(140.)).child(*name))
                    .child(cells)
                    .child(div().flex_1())
                    .child(text(t, format!("{pct}%"), ColorRole::FgDefault, 12.5)),
            );
        }
        let counts = e.counts(n);
        let mut filt = row().gap(px(4.));
        for (i, f) in evals::FILTERS.iter().enumerate() {
            filt = filt.child(self.chip(
                cx,
                &format!("ev-f-{i}"),
                format!("{f} {}", counts[i]),
                e.filter == i,
            ));
        }
        let mut list = div().flex().flex_col();
        for i in e.list(n).into_iter().take(6) {
            let cs = &e.cases[i];
            let tag = match e.change(i, n) {
                Some(false) => "Regression",
                Some(true) => "Improved",
                None => "",
            };
            list = list.child(
                self.hit(cx, &format!("ev-open-{i}"), e.open == Some(i))
                    .child(div().flex_1().min_w(px(0.)).child(cs.title.clone()))
                    .child(text(
                        t,
                        format!(
                            "{} → {}",
                            cs.base.word(),
                            e.out(i, n).map_or("…", |o| o.word())
                        ),
                        ColorRole::FgMuted,
                        12.,
                    ))
                    .when(!tag.is_empty(), |d| {
                        d.child(pill(
                            t,
                            tag,
                            if tag == "Regression" { "red" } else { "green" },
                        ))
                    }),
            );
            if e.open == Some(i) {
                let (_, expected, bo, no, _, why) = e.detail(i);
                list = list.child(
                    self.card()
                        .child(text(
                            t,
                            format!("Expected: {expected}"),
                            ColorRole::FgMuted,
                            12.,
                        ))
                        .child(text(
                            t,
                            format!("{}: {bo}", evals::BASELINE.0),
                            ColorRole::FgDefault,
                            12.,
                        ))
                        .child(text(
                            t,
                            format!("{}: {no}", evals::CANDIDATE.0),
                            ColorRole::FgDefault,
                            12.,
                        ))
                        .child(text(t, why, ColorRole::FgSubtle, 12.))
                        .child(if e.triage[i] == 0 {
                            row()
                                .child(self.btn(
                                    cx,
                                    &format!("ev-expected-{i}"),
                                    "Mark as expected",
                                    false,
                                ))
                                .child(self.btn(cx, &format!("ev-issue-{i}"), "Open issue", false))
                        } else {
                            row().child(self.btn(cx, &format!("ev-undo-{i}"), "Undo", false))
                        }),
                );
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        evals::FEATURE,
                        format!(
                            "{} {} vs {} {} · judge {}",
                            evals::CANDIDATE.0,
                            evals::CANDIDATE.2,
                            evals::BASELINE.0,
                            evals::BASELINE.2,
                            evals::JUDGE.1
                        ),
                    ))
                    .child(self.btn(
                        cx,
                        "ev-run",
                        if state == "Running" { "Stop" } else { "Run" },
                        state != "Running",
                    )),
            )
            .child(
                self.card().child(
                    row()
                        .child(
                            div()
                                .text_size(px(26.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("{score}%")),
                        )
                        .child(text(
                            t,
                            format!("baseline {base}% · {state}"),
                            ColorRole::FgMuted,
                            12.5,
                        ))
                        .child(div().flex_1())
                        .child(mark(evals::CANDIDATE.1, 14., c(t, ColorRole::FgDefault)))
                        .child(mark(evals::BASELINE.1, 14., c(t, ColorRole::FgDefault))),
                ),
            )
            .child(cats)
            .child(filt)
            .child(list)
    }

    fn builder(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let b = &self.state.builder;
        let (mb, ml, _) = builder::MODELS[b.config.model];
        let mut body = div().flex().flex_col().gap(px(8.));
        if !b.testing {
            let mut models = row().gap(px(4.));
            for (i, (br, l, _)) in builder::MODELS.iter().enumerate() {
                models = models.child(
                    self.chip(cx, &format!("ab-model-{i}"), *l, b.config.model == i)
                        .child(mark(br, 12., c(t, ColorRole::FgDefault))),
                );
            }
            body = body.child(models).child(self.card().child(text(
                t,
                b.config.instructions.clone(),
                ColorRole::FgDefault,
                12.5,
            )));
            for (i, (br, name, what)) in builder::TOOLS.iter().enumerate() {
                body = body.child(
                    row()
                        .child(mark(br, 18., c(t, ColorRole::FgDefault)))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.))
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(*name))
                                .child(text(t, *what, ColorRole::FgMuted, 12.)),
                        )
                        .child(self.sw(cx, &format!("ab-tool-{i}"), b.config.tools[i])),
                );
            }
            for (i, (f, kbs)) in builder::FILES.iter().enumerate() {
                let (p, failed) = b.upload(i, n);
                let mut r_ = row().child(div().flex_1().child(format!("{f} · {kbs} KB")));
                r_ = if failed {
                    r_.child(text(t, "Upload failed", ColorRole::StatusDangerFg, 12.))
                        .child(self.btn(cx, &format!("ab-retry-{i}"), "Retry", false))
                } else if p < 1.0 {
                    r_.child(
                        div()
                            .w(px(120.))
                            .child(bar(t, p, c(t, ColorRole::FgDefault))),
                    )
                } else {
                    r_.child(self.status("Ready", "green"))
                };
                body = body.child(r_);
            }
        } else {
            for (i, (q, _)) in b.chat.iter().enumerate() {
                body = body
                    .child(
                        row().justify_end().child(
                            div()
                                .px(px(10.))
                                .py(px(6.))
                                .rounded(px(10.))
                                .bg(c(t, ColorRole::SurfaceSelected))
                                .child(q.clone()),
                        ),
                    )
                    .child(div().child(b.reply(i, n).1));
            }
            let mut ps = row().gap(px(6.));
            for (i, p) in builder::PROMPTS.iter().enumerate() {
                ps = ps.child(self.chip(cx, &format!("ab-prompt-{i}"), *p, false));
            }
            body = body.child(ps);
        }
        let pst = b.publish_state(n);
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(
                        row()
                            .child(mark(mb, 18., c(t, ColorRole::FgDefault)))
                            .child(title(
                                t,
                                &b.config.name,
                                format!("Northwind · v{} · {ml}", b.version),
                            )),
                    )
                    .child(
                        row()
                            .gap(px(4.))
                            .child(self.chip(cx, "ab-view-0", "Configure", !b.testing))
                            .child(self.chip(cx, "ab-view-1", "Test", b.testing)),
                    ),
            )
            .child(body)
            .child(
                row()
                    .justify_between()
                    .child(self.status(
                        pst,
                        match pst {
                            "Published" => "green",
                            "Publishing" => "blue",
                            "Publish failed" => "red",
                            _ => "amber",
                        },
                    ))
                    .child(self.btn(cx, "ab-publish", "Publish", true)),
            )
    }

    fn gallery(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let g = &self.state.gallery;
        let mut teams = row().gap(px(4.));
        for (i, tm) in gallery::TEAMS.iter().enumerate() {
            teams = teams.child(self.chip(cx, &format!("ag-team-{i}"), *tm, g.team == i));
        }
        let mut grid = div().flex().flex_wrap().gap(px(10.));
        for i in g.shown() {
            let tp = gallery::TEMPLATES[i];
            let mut apps = row().gap(px(4.));
            for a in tp.apps {
                apps = apps.child(mark(a, 15., c(t, ColorRole::FgDefault)));
            }
            let mut card = self
                .card()
                .w(px(330.))
                .child(
                    row()
                        .justify_between()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(tp.name))
                        .child(text(
                            t,
                            format!("★ {}.{}", tp.rating / 10, tp.rating % 10),
                            ColorRole::FgMuted,
                            12.,
                        )),
                )
                .child(text(t, tp.job, ColorRole::FgMuted, 12.5))
                .child(
                    row().justify_between().child(apps).child(
                        row()
                            .child(mark(tp.model.0, 12., c(t, ColorRole::FgDefault)))
                            .child(text(t, tp.model.1, ColorRole::FgSubtle, 11.5)),
                    ),
                );
            let flow = g.flow.filter(|f| f.0 == i).map(|f| f.1);
            card = match (flow, g.installed_in(i)) {
                (_, Some(ws)) => card.child(self.status(format!("Installed in {ws}"), "green")),
                (None, None) => card.child(
                    row()
                        .child(self.btn(cx, &format!("ag-preview-{i}"), "Preview", false))
                        .child(self.btn(cx, &format!("ag-install-{i}"), "Install", true)),
                ),
                (Some(Flow::Preview), None) => {
                    let mut c2 = card;
                    for (k, s) in tp.preview.iter().enumerate() {
                        c2 = c2.child(text(
                            t,
                            format!("{}. {s}", k + 1),
                            ColorRole::FgDefault,
                            12.5,
                        ));
                    }
                    c2.child(
                        row()
                            .child(self.btn(cx, "ag-back", "Back", false))
                            .child(self.btn(cx, &format!("ag-install-{i}"), "Install", true)),
                    )
                }
                (Some(Flow::Workspace(w)), None) => {
                    let mut c2 = card;
                    for (k, (ws, det)) in gallery::WORKSPACES.iter().enumerate() {
                        c2 = c2.child(
                            self.hit(cx, &format!("ag-ws-{k}"), w == k)
                                .child(div().flex_1().child(*ws))
                                .child(text(t, *det, ColorRole::FgMuted, 12.)),
                        );
                    }
                    c2.child(
                        row()
                            .child(self.btn(cx, "ag-back", "Back", false))
                            .child(self.btn(cx, "ag-grant", "Grant access", true)),
                    )
                }
                (Some(Flow::Connect(..)), None) => {
                    let (got, failed, done) = g.connect(n).unwrap_or((0, None, false));
                    let mut c2 = card;
                    for (k, a) in tp.apps.iter().enumerate() {
                        let (w, h) = if failed == Some(k) {
                            ("Could not connect", "red")
                        } else if k < got || done {
                            ("Connected", "green")
                        } else if k == got {
                            ("Connecting", "blue")
                        } else {
                            ("Waiting", "neutral")
                        };
                        c2 = c2.child(
                            row()
                                .child(mark(a, 14., c(t, ColorRole::FgDefault)))
                                .child(div().flex_1().child(gallery::app_name(a)))
                                .child(self.status(w, h)),
                        );
                    }
                    if failed.is_some() {
                        c2 = c2.child(self.btn(cx, "ag-retry", "Retry", true));
                    } else if done {
                        c2 = c2.child(self.btn(cx, "ag-back", "Done", false));
                    }
                    c2
                }
                (Some(Flow::Card), None) => card,
            };
            grid = grid.child(card);
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(title(t, "Agent gallery", g.result_line()))
            .child(teams)
            .child(grid)
    }

    fn insight(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.insight;
        let lo = insight::SERIES
            .iter()
            .chain(insight::PREV.iter())
            .fold(f64::MAX, |a, &b| a.min(b));
        let hi = insight::SERIES
            .iter()
            .chain(insight::PREV.iter())
            .fold(f64::MIN, |a, &b| a.max(b));
        let reached = s.reached(n);
        let ans = s.answer.map(|a| a.0);
        let drivers = ans.map(|a| insight::ANSWERS[a].1.drivers).unwrap_or(&[]);
        let mut chart = row().items_end().gap(px(4.)).h(px(90.));
        for (i, v) in insight::SERIES.iter().enumerate() {
            let marked = drivers.iter().take(reached).position(|d| d.2 == i);
            let h = ((v - lo) / (hi - lo)).clamp(0.05, 1.0) as f32;
            let mut col = div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .justify_end()
                .items_center();
            if let Some(k) = marked {
                col = col.child(
                    div()
                        .size(px(16.))
                        .rounded(px(8.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(hue(t, if drivers[k].1 < 0 { "red" } else { "green" }))
                        .text_size(px(10.))
                        .text_color(c(t, ColorRole::BgBase))
                        .child(format!("{}", k + 1)),
                );
            }
            col = col.child(div().w_full().h(relative(h * 0.8)).rounded(px(3.)).bg(
                if s.point == Some(i) {
                    c(t, ColorRole::FgDefault)
                } else {
                    c(t, ColorRole::FgMuted).opacity(0.5)
                },
            ));
            chart = chart.child(col);
        }
        let mut out = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        insight::METRIC.0,
                        format!(
                            "{} · {:+.1}% vs the 12 weeks before",
                            thousands(insight::METRIC.2 as u64),
                            s.delta()
                        ),
                    ))
                    .child(self.btn(
                        cx,
                        "ai-explain",
                        if s.streaming(n) {
                            "Stop"
                        } else if ans.is_some() || s.error {
                            "Hide"
                        } else {
                            "Explain"
                        },
                        ans.is_none(),
                    )),
            )
            .child(chart);
        if let Some(p) = s.point {
            out = out.child(text(
                t,
                format!(
                    "{}: {} · before {}",
                    insight::POINTS[p],
                    insight::SERIES[p],
                    insight::PREV[p]
                ),
                ColorRole::FgMuted,
                12.,
            ));
        }
        if s.error {
            out = out.child(
                row()
                    .child(text(
                        t,
                        "Couldn't explain the change.",
                        ColorRole::StatusDangerFg,
                        12.5,
                    ))
                    .child(self.btn(cx, "ai-retry", "Retry", false)),
            );
        }
        if ans.is_some() {
            let txt: Vec<String> = s.segments(n).into_iter().map(|x| x.0).collect();
            out = out.child(div().child(txt.join(" ")));
            let dmax = drivers
                .iter()
                .map(|d| d.1.unsigned_abs())
                .max()
                .unwrap_or(1)
                .max(1) as f64;
            for (k, d) in drivers.iter().enumerate() {
                let on = k < reached;
                out = out.child(
                    row()
                        .when(!on, |x| x.opacity(0.25))
                        .child(div().w(px(18.)).child(format!("{}", k + 1)))
                        .child(div().flex_1().child(d.0))
                        .child(div().w(px(120.)).child(bar(
                            t,
                            if on {
                                d.1.unsigned_abs() as f64 / dmax
                            } else {
                                0.0
                            },
                            hue(t, if d.1 < 0 { "red" } else { "green" }),
                        )))
                        .child(div().w(px(40.)).child(format!("{:+}", d.1))),
                );
            }
            let mut fu = row().gap(px(6.));
            for (k, (label, _)) in insight::ANSWERS.iter().enumerate().skip(1) {
                fu = fu.child(self.chip(cx, &format!("ai-follow-{k}"), *label, ans == Some(k)));
            }
            out = out.child(fu);
        }
        out
    }
}

impl Render for BlocksView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let fg = c(t, ColorRole::FgDefault);
        let body: Div = match self.variant {
            BlockVariant::LiveSession => self.session(cx),
            BlockVariant::MissionControl => self.mission(cx, false),
            BlockVariant::MissionTimeline => self.mission(cx, true),
            BlockVariant::RunInspector => self.inspector(cx),
            BlockVariant::BackgroundAgents => self.bg_agents(cx),
            BlockVariant::Routines => self.routines(cx),
            BlockVariant::Mcp => self.mcp(cx),
            BlockVariant::KnowledgeBase => self.kb(cx),
            BlockVariant::Evals => self.evals(cx),
            BlockVariant::Builder => self.builder(cx),
            BlockVariant::Gallery => self.gallery(cx),
            BlockVariant::Insight => self.insight(cx),
        };
        let said = self.state.said.clone();
        let line: Hsla = c(t, ColorRole::BorderSubtle);
        div()
            .id("oxyn-blocks")
            .key_context("OxynBlocks")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.key(ev, cx)))
            .p(px(16.))
            .size_full()
            .bg(c(t, ColorRole::BgBase))
            .text_color(fg)
            .text_size(px(13.))
            .child(
                div()
                    .p(px(16.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(line)
                    .bg(c(t, ColorRole::SurfaceBase))
                    .child(body),
            )
            .child(
                text(
                    t,
                    format!("{} · {}", dur(self.state.now), said),
                    ColorRole::FgSubtle,
                    11.5,
                )
                .mt(px(8.)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presses_and_keys_route_to_each_block() {
        let mut s = BlocksState::default();
        s.press(BlockVariant::LiveSession, "ls-agent-2");
        assert_eq!(s.session.focus, Some(2));
        s.key(BlockVariant::LiveSession, "escape", false);
        assert_eq!(s.session.focus, None);
        s.press(BlockVariant::MissionControl, "mi-ok-3");
        assert_eq!(s.control.st(3, s.now), St::Running);
        s.key(BlockVariant::MissionTimeline, "right", false);
        assert_eq!(s.timeline.filter, 1);
        s.press(BlockVariant::RunInspector, "ri-agent-6");
        assert_eq!(s.inspector.selected, Some(6));
        s.press(BlockVariant::BackgroundAgents, "ba-start");
        assert_eq!(s.bg.tasks.len(), 5);
        s.press(BlockVariant::Routines, "ro-toggle-3");
        assert!(s.routines.list[3].enabled);
        s.press(BlockVariant::Mcp, "mcp-add");
        assert!(s.mcp.sheet.is_some());
        s.key(BlockVariant::Mcp, "escape", false);
        assert!(s.mcp.sheet.is_none());
        s.press(BlockVariant::KnowledgeBase, "kb-view-1");
        assert_eq!(s.kb.view, KbView::Test);
        s.press(BlockVariant::Evals, "ev-run");
        assert!(s.evals.run.is_some());
        s.press(BlockVariant::Builder, "ab-tool-3");
        assert!(s.builder.unsaved());
        s.key(BlockVariant::Gallery, "right", false);
        assert_eq!(s.gallery.team, 3);
        s.key(BlockVariant::Insight, "home", false);
        assert_eq!(s.insight.point, Some(0));
        assert_eq!(BlockVariant::ALL.len(), 12);
        assert_eq!(BlockVariant::Insight.slug(), "ai-insight-card");
    }
}
