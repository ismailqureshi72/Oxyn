//! AI special components (Oxyn Pro) in GPUI as one live view: Agent timeline, Model card, AI
//! workflow, Image generation, Speech player, Meeting summary, Reply draft, Human handoff, PII
//! redaction, LLM cost calculator, Provider keys, Generation stats, AI translate, TL;DR toggle,
//! Connector setup, Enrich column and Ticket router. Every variant runs on the same
//! `oxyn_core::aikit` models as the Dioxus components, so presses and keys behave the same in
//! both renderers. The view keeps its own clock (a tick every 100 ms while it lives); a host can
//! also drive it with [`AiKitView::advance`]. Text fields are shown as text (no editing here).

use std::time::Duration;

use gpui::{
    App, ClickEvent, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, Stateful, Styled, Window, div, prelude::*,
    px, relative, rgb,
};
use oxyn_core::aikit::{
    connector::{APPS, ConnSt, Connectors},
    cost::{Cost, MODELS as COST_MODELS},
    enrich::{CellSt, Enrich, ROWS},
    first_words,
    genstats::{GenStats, MODELS as GS_MODELS},
    handoff::{Handoff, Stage, Who},
    imagegen::{ASPECTS, ImageGen, MODELS as IG_MODELS, SCENES, VarSt},
    keys::{KeySt, Keys, PROVIDERS},
    meeting::{LINES, MEETING, Meeting, SendSt, Status as MsStatus},
    model_card::{BENCHMARKS, MODALITIES, ModelCard, VOLUMES, View},
    pii::{Phase as PiiPhase, Pii, TYPES, placeholder},
    reply::{LENGTHS, MESSAGE, Phase as RdPhase, Reply, TONES},
    router::{LANES, Router, TICKETS},
    secs,
    speech::{Phase as SpPhase, Speech, VOICES},
    thousands,
    timeline::{SpanAt, SpanKind, Timeline},
    tldr::{MODES, PARAS, SUMMARIES, Tldr},
    tokens_short,
    translate::{LANGS, TEXT, Translate},
    usd,
    workflow::{StepSt, Workflow},
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

/// Which AI special component the view draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiVariant {
    /// Agent timeline.
    Timeline,
    /// Model card.
    ModelCard,
    /// AI workflow.
    Workflow,
    /// Image generation.
    ImageGen,
    /// Speech player.
    Speech,
    /// Meeting summary.
    Meeting,
    /// Reply draft.
    Reply,
    /// Human handoff.
    Handoff,
    /// PII redaction.
    Pii,
    /// LLM cost calculator.
    Cost,
    /// Provider keys.
    Keys,
    /// Generation stats.
    GenStats,
    /// AI translate.
    Translate,
    /// TL;DR toggle.
    Tldr,
    /// Connector setup.
    Connector,
    /// Enrich column.
    Enrich,
    /// Ticket router.
    Router,
}

impl AiVariant {
    /// Every variant, in gallery order.
    pub const ALL: [AiVariant; 17] = [
        Self::Timeline,
        Self::ModelCard,
        Self::Workflow,
        Self::ImageGen,
        Self::Speech,
        Self::Meeting,
        Self::Reply,
        Self::Handoff,
        Self::Pii,
        Self::Cost,
        Self::Keys,
        Self::GenStats,
        Self::Translate,
        Self::Tldr,
        Self::Connector,
        Self::Enrich,
        Self::Router,
    ];

    /// The page slug.
    pub fn slug(self) -> &'static str {
        oxyn_core::aikit::AI_SLUGS[Self::ALL.iter().position(|&v| v == self).unwrap_or(0)]
    }
}

/// The state of every AI special component, on one clock; pure, so presses and keys are
/// testable without a window.
#[derive(Clone, Debug)]
pub struct AiKitState {
    /// Clock, ms.
    pub now: u64,
    /// Agent timeline.
    pub timeline: Timeline,
    /// Model card.
    pub model: ModelCard,
    /// AI workflow.
    pub workflow: Workflow,
    /// Image generation.
    pub image: ImageGen,
    /// Speech player.
    pub speech: Speech,
    /// Meeting summary.
    pub meeting: Meeting,
    /// Reply draft.
    pub reply: Reply,
    /// Human handoff.
    pub handoff: Handoff,
    /// PII redaction.
    pub pii: Pii,
    /// LLM cost calculator.
    pub cost: Cost,
    /// Provider keys.
    pub keys: Keys,
    /// Generation stats.
    pub stats: GenStats,
    /// AI translate.
    pub translate: Translate,
    /// TL;DR toggle.
    pub tldr: Tldr,
    /// Connector setup.
    pub connectors: Connectors,
    /// Enrich column.
    pub enrich: Enrich,
    /// Ticket router.
    pub router: Router,
    /// The last thing said (the live region of the web components).
    pub said: String,
}

impl Default for AiKitState {
    fn default() -> Self {
        let now = 100_000;
        let mut workflow = Workflow::new("passes", false);
        workflow.press("wf-run", now);
        let mut connectors = Connectors::new(false);
        connectors.press("cs-connect-2", now);
        Self {
            now,
            timeline: Timeline::new(true, false, now),
            model: ModelCard::new(),
            workflow,
            image: ImageGen::new(true, false, now),
            speech: Speech::new(true, false, now),
            meeting: Meeting::new("play", now),
            reply: Reply::new(true, now),
            handoff: Handoff::new("handoff", now),
            pii: Pii::new(true, now),
            cost: Cost::new(),
            keys: Keys::new(false),
            stats: GenStats::new("stream", 0, now),
            translate: Translate::new(1, false, now),
            tldr: Tldr::new(1, 1, false, now),
            connectors,
            enrich: Enrich::new(0, 3, true, now),
            router: Router::new(false, 70, now),
            said: String::new(),
        }
    }
}

impl AiKitState {
    /// Routes a press id to the model of `v`.
    pub fn press(&mut self, v: AiVariant, id: &str) {
        let n = self.now;
        let out = match v {
            AiVariant::Timeline => self.timeline.press(id, n),
            AiVariant::ModelCard => self.model.press(id, n),
            AiVariant::Workflow => self.workflow.press(id, n),
            AiVariant::ImageGen => self.image.press(id, n),
            AiVariant::Speech => self.speech.press(id, n),
            AiVariant::Meeting => self.meeting.press(id, n),
            AiVariant::Reply => self.reply.press(id, n),
            AiVariant::Handoff => self.handoff.press(id, n),
            AiVariant::Pii => self.pii.press(id, n),
            AiVariant::Cost => self.cost.press(id),
            AiVariant::Keys => self.keys.press(id, n),
            AiVariant::GenStats => self.stats.press(id, n),
            AiVariant::Translate => self.translate.press(id, n),
            AiVariant::Tldr => self.tldr.press(id, n),
            AiVariant::Connector => self.connectors.press(id, n),
            AiVariant::Enrich => self.enrich.press(id, n),
            AiVariant::Router => self.router.press(id, n),
        };
        if let Some(s) = out {
            self.said = s;
        }
    }

    /// Routes a key (`left`, `enter`, `escape`…) to the model of `v`, with its main keyboard
    /// part (spans for the timeline, the volume slider for the model card, and so on).
    pub fn key(&mut self, v: AiVariant, key: &str, shift: bool, cmd: bool) {
        let n = self.now;
        let k = oxyn_core::aikit::norm_key(key);
        let out = match v {
            AiVariant::Timeline => self.timeline.key_span(k, n),
            AiVariant::ModelCard => self.model.key("vol", k, n),
            AiVariant::Workflow => {
                if self.workflow.menu {
                    self.workflow.key("menu", k, n)
                } else {
                    self.workflow.key("tabs", k, n)
                }
            }
            AiVariant::ImageGen => {
                if cmd && k == "Enter" {
                    self.image.key("gen", k, n)
                } else {
                    self.image.key("strip", k, n)
                }
            }
            AiVariant::Speech => self.speech.key("bar", k, n),
            AiVariant::Meeting => self.meeting.key(k),
            AiVariant::Reply => {
                if cmd && k == "Enter" {
                    self.reply.key("insert", k, n)
                } else {
                    self.reply.key("tone", k, n)
                }
            }
            AiVariant::Handoff => None,
            AiVariant::Pii => {
                if cmd && k == "Enter" {
                    let id = match self.pii.phase_at(n) {
                        PiiPhase::Edit | PiiPhase::Done => "pii-scan",
                        PiiPhase::Found => "pii-redact",
                        PiiPhase::Redacted => "pii-send",
                        _ => "",
                    };
                    self.pii.press(id, n)
                } else {
                    None
                }
            }
            AiVariant::Cost => self.cost.key(9, k, shift),
            AiVariant::Keys => (k == "Escape")
                .then(|| self.keys.press("pk-cancel", n))
                .flatten(),
            AiVariant::GenStats => None,
            AiVariant::Translate => self.translate.key(k, n),
            AiVariant::Tldr => self.tldr.key("mode", k, n),
            AiVariant::Connector => self.connectors.key(k, n),
            AiVariant::Enrich => self.enrich.key(k, cmd, n),
            AiVariant::Router => None,
        };
        if let Some(s) = out {
            self.said = s;
        }
    }
}

/// The AI special components as one live view.
pub struct AiKitView {
    variant: AiVariant,
    state: AiKitState,
    dark: bool,
    focus: FocusHandle,
}

impl Focusable for AiKitView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl AiKitView {
    /// A view of `variant`, focused so the keys work at once, ticking its own clock.
    pub fn new(variant: AiVariant, window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            state: AiKitState::default(),
            dark: true,
            focus,
        }
    }

    /// Switch variant.
    pub fn set_variant(&mut self, v: AiVariant, cx: &mut Context<Self>) {
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
    pub fn state(&self) -> &AiKitState {
        &self.state
    }

    fn theme(&self) -> &'static Theme {
        if self.dark {
            &Theme::DARK
        } else {
            &Theme::LIGHT
        }
    }

    fn key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let m = &ev.keystroke.modifiers;
        let k = ev.keystroke.key.clone();
        self.state
            .key(self.variant, &k, m.shift, m.platform || m.control);
        cx.notify();
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
            .id(SharedString::from(format!("ai-{id}")))
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
}

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn hue(t: &Theme, h: &str) -> Hsla {
    match h {
        "blue" => c(t, ColorRole::StatusInfoFg),
        "green" => c(t, ColorRole::StatusSuccessFg),
        "amber" => c(t, ColorRole::StatusWarningFg),
        "red" => c(t, ColorRole::StatusDangerFg),
        "violet" => rgb(0x8b7cf6).into(),
        "teal" => rgb(0x22a699).into(),
        _ => c(t, ColorRole::FgMuted),
    }
}

fn text(t: &Theme, s: impl Into<SharedString>, r: ColorRole, size: f32) -> Div {
    div()
        .text_size(px(size))
        .text_color(c(t, r))
        .child(s.into())
}

fn title(t: &Theme, h: &str, sub: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .min_w(px(0.))
        .child(
            div()
                .text_size(px(14.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(c(t, ColorRole::FgDefault))
                .child(h.to_string()),
        )
        .child(text(t, sub, ColorRole::FgMuted, 12.5))
}

fn row() -> Div {
    div().flex().items_center().gap(px(6.)).min_w(px(0.))
}

fn pill(t: &Theme, s: impl Into<SharedString>, h: &str) -> Div {
    let col = hue(t, h);
    div()
        .h(px(20.))
        .px(px(7.))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(4.))
        .rounded(px(10.))
        .border_1()
        .border_color(col.opacity(0.4))
        .bg(col.opacity(0.1))
        .text_size(px(11.5))
        .text_color(col)
        .child(s.into())
}

fn bar(t: &Theme, frac: f64, col: Hsla) -> Div {
    div()
        .h(px(6.))
        .w_full()
        .rounded(px(3.))
        .bg(c(t, ColorRole::SurfaceSelected))
        .child(
            div()
                .h_full()
                .rounded(px(3.))
                .w(relative(frac.clamp(0.0, 1.0) as f32))
                .bg(col),
        )
}

fn mark(name: &str, size: f32, fg: Hsla) -> Div {
    let d = div()
        .flex_none()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center();
    match crate::brand::brand_mark(name, size, fg) {
        Some(m) => d.child(m),
        None => d,
    }
}

fn line(t: &Theme) -> Div {
    div().h(px(1.)).w_full().bg(c(t, ColorRole::BorderSubtle))
}

impl Render for AiKitView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let n = self.state.now;
        let fg = c(t, ColorRole::FgDefault);
        let muted = ColorRole::FgMuted;
        let body: Div = match self.variant {
            AiVariant::Timeline => {
                let s = self.state.timeline.clone();
                let done = s.finished(n);
                let head = if done { s.summary() } else { s.status(n) };
                let mut plot = div().flex().flex_col();
                for (a, ag) in s.agents.iter().enumerate() {
                    let mut lane = div().relative().h(px(32.)).flex_1().min_w(px(240.));
                    for i in s.row(a) {
                        let sp = &s.spans[i];
                        let (l, w) = s.geom(i, n);
                        let at = s.span_at(i, n);
                        if at == SpanAt::Future {
                            continue;
                        }
                        let col = hue(t, ag.hue);
                        let bg = match (sp.kind, at) {
                            (_, SpanAt::Failed) => hue(t, "red").opacity(0.35),
                            (SpanKind::Tool, _) => col,
                            (SpanKind::Thinking, _) => col.opacity(0.3),
                            (SpanKind::Waiting, _) => col.opacity(0.12),
                        };
                        let dim = !s.span_on_path(i);
                        let id = format!("tl-span-{i}");
                        lane = lane.child(
                            self.btn(cx, &id, "", false)
                                .absolute()
                                .left(relative(l as f32))
                                .w(relative(w.max(0.004) as f32))
                                .top(px(7.))
                                .h(px(18.))
                                .px(px(0.))
                                .rounded(px(4.))
                                .border_0()
                                .bg(bg)
                                .when(dim, |d| d.opacity(0.25))
                                .when(s.pinned == Some(i), |d| d.border_1().border_color(fg)),
                        );
                    }
                    if !done {
                        lane = lane.child(
                            div()
                                .absolute()
                                .top_0()
                                .bottom_0()
                                .left(relative((s.t(n) as f64 / s.duration as f64) as f32))
                                .w(px(1.5))
                                .bg(fg),
                        );
                    }
                    plot = plot.child(
                        row()
                            .child(
                                row()
                                    .w(px(170.))
                                    .flex_none()
                                    .child(div().size(px(7.)).rounded(px(4.)).bg(hue(t, ag.hue)))
                                    .child(text(t, ag.name, ColorRole::FgDefault, 12.5))
                                    .child(mark(ag.model, 12., fg))
                                    .child(text(t, ag.model, ColorRole::FgSubtle, 11.)),
                            )
                            .child(lane),
                    );
                }
                let pinned = s
                    .pinned
                    .map(|i| s.span_name(i))
                    .unwrap_or_else(|| "Enter pins the focused span; ←/→ ↑/↓ move".into());
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        row()
                            .justify_between()
                            .child(title(t, "Checkout conversion drop on Oct 14", head))
                            .child(
                                row()
                                    .child(self.btn(
                                        cx,
                                        "tl-play",
                                        if s.playing(n) {
                                            "Pause"
                                        } else if done {
                                            "Replay"
                                        } else {
                                            "Play"
                                        },
                                        false,
                                    ))
                                    .child(self.btn(cx, "tl-restart", "Restart", false))
                                    .child(self.chip(
                                        cx,
                                        "tl-speed",
                                        format!("{}x", s.speed),
                                        false,
                                    )),
                            ),
                    )
                    .child(plot)
                    .child(text(t, pinned, muted, 12.))
            }
            AiVariant::ModelCard => {
                let mc = self.state.model.clone();
                let m = mc.m().clone();
                let mut tabs = row();
                for (i, x) in mc.models.iter().enumerate() {
                    tabs =
                        tabs.child(self.chip(cx, &format!("mc-model-{i}"), x.name, i == mc.model));
                }
                let mut mods = row().flex_wrap();
                for (k, md) in MODALITIES.iter().enumerate() {
                    mods = mods.child(
                        pill(t, *md, if m.modalities[k] { "neutral" } else { "" })
                            .when(!m.modalities[k], |d| d.opacity(0.4)),
                    );
                }
                let content = if mc.view == View::Compare {
                    let mut tb = div().flex().flex_col();
                    for r in mc.compare() {
                        tb = tb.child(
                            row()
                                .py(px(4.))
                                .child(text(t, r.label.clone(), muted, 12.).w(px(150.)))
                                .child(
                                    text(
                                        t,
                                        format!(
                                            "{}{}",
                                            r.a,
                                            if r.better == 0 {
                                                format!(" ✓ {}", r.margin)
                                            } else {
                                                String::new()
                                            }
                                        ),
                                        ColorRole::FgDefault,
                                        12.5,
                                    )
                                    .flex_1(),
                                )
                                .child(
                                    text(
                                        t,
                                        format!(
                                            "{}{}",
                                            r.b,
                                            if r.better == 1 {
                                                format!(" ✓ {}", r.margin)
                                            } else {
                                                String::new()
                                            }
                                        ),
                                        ColorRole::FgDefault,
                                        12.5,
                                    )
                                    .flex_1(),
                                ),
                        );
                    }
                    div().flex().flex_col().gap(px(6.)).child(tb).child(text(
                        t,
                        mc.tally(),
                        muted,
                        12.,
                    ))
                } else {
                    let mut bench = div().flex().flex_col().gap(px(4.));
                    for (k, (i, sc)) in mc.bench_rows().iter().enumerate() {
                        bench = bench.child(
                            row()
                                .child(
                                    text(t, mc.models[*i].name, ColorRole::FgDefault, 12.5)
                                        .w(px(150.)),
                                )
                                .child(bar(
                                    t,
                                    sc.unwrap_or(0.0) / 100.0,
                                    if k == 0 { fg } else { c(t, muted) },
                                ))
                                .child(
                                    text(
                                        t,
                                        sc.map(|x| format!("{x:.1}%")).unwrap_or("None".into()),
                                        ColorRole::FgDefault,
                                        12.5,
                                    )
                                    .w(px(52.)),
                                ),
                        );
                    }
                    let mut bs = row();
                    for (i, b) in BENCHMARKS.iter().enumerate() {
                        bs = bs.child(self.chip(cx, &format!("mc-bench-{i}"), b.1, i == mc.bench));
                    }
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(text(t, m.summary, muted, 12.5))
                        .child(mods)
                        .child(text(
                            t,
                            format!(
                                "Context {} · max output {} · cutoff {} · {}",
                                tokens_short(m.context),
                                tokens_short(m.max_out),
                                m.cutoff,
                                mc.pages()
                            ),
                            ColorRole::FgDefault,
                            12.5,
                        ))
                        .child(text(
                            t,
                            format!(
                                "Input {} · output {} · cached {} per 1M",
                                usd(m.input),
                                usd(m.output),
                                usd(m.cached)
                            ),
                            ColorRole::FgDefault,
                            12.5,
                        ))
                        .child(
                            row()
                                .child(self.btn(
                                    cx,
                                    &format!("mc-vol-{}", mc.vol.saturating_sub(1)),
                                    "−",
                                    false,
                                ))
                                .child(text(
                                    t,
                                    format!(
                                        "{} tokens a month · about {}/mo",
                                        tokens_short(VOLUMES[mc.vol]),
                                        usd(mc.cost_of(mc.model))
                                    ),
                                    ColorRole::FgDefault,
                                    12.5,
                                ))
                                .child(self.btn(
                                    cx,
                                    &format!("mc-vol-{}", (mc.vol + 1).min(8)),
                                    "+",
                                    false,
                                )),
                        )
                        .child(bs)
                        .child(bench)
                };
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(tabs)
                    .child(
                        row()
                            .justify_between()
                            .child(
                                row()
                                    .child(mark(m.name, 20., fg))
                                    .child(title(t, m.name, m.provider)),
                            )
                            .child(self.chip(
                                cx,
                                "mc-compare",
                                "Compare",
                                mc.view == View::Compare,
                            )),
                    )
                    .child(content)
                    .child(line(t))
                    .child(
                        row()
                            .justify_between()
                            .child(text(t, m.id, muted, 12.))
                            .child(self.btn(cx, "mc-copy", "Copy model id", false)),
                    )
            }
            AiVariant::Workflow => {
                let w = self.state.workflow.clone();
                let mut list = div().flex().flex_col().gap(px(4.));
                let mut steps: Vec<_> = w.steps.clone();
                if let Some(p) = w.paths.get(w.tab) {
                    steps.extend(p.steps.clone());
                }
                for s in steps {
                    let st = w.status(s.id, n);
                    let (lab, h) = match st {
                        StepSt::Idle => (String::new(), "neutral"),
                        StepSt::Pending => ("Waiting".into(), "neutral"),
                        StepSt::Travel | StepSt::Running => ("Running".into(), "blue"),
                        StepSt::Approval => ("Needs you".into(), "amber"),
                        StepSt::Done(ms) => (format!("{ms}ms"), "green"),
                        StepSt::Failed => ("Failed".into(), "red"),
                        StepSt::Skipped => ("Skipped".into(), "neutral"),
                    };
                    let busy = matches!(st, StepSt::Running | StepSt::Travel);
                    list = list.child(
                        row()
                            .h(px(32.))
                            .child(mark(if s.app == "branch" { "" } else { s.app }, 16., fg))
                            .child(text(t, s.title, ColorRole::FgDefault, 13.).flex_none())
                            .child(
                                text(
                                    t,
                                    if busy { s.activity } else { s.detail },
                                    ColorRole::FgSubtle,
                                    12.,
                                )
                                .flex_1()
                                .overflow_hidden()
                                .whitespace_nowrap(),
                            )
                            .when(!lab.is_empty(), |d| d.child(pill(t, lab, h)))
                            .when(st == StepSt::Failed, |d| {
                                d.child(self.btn(cx, "wf-retry", "Retry", false))
                            })
                            .when(st == StepSt::Approval, |d| {
                                d.child(self.btn(cx, "wf-approve", "Approve", true))
                            }),
                    );
                }
                let mut tabs = row();
                for (i, p) in w.paths.iter().enumerate() {
                    tabs = tabs.child(self.chip(cx, &format!("wf-path-{i}"), p.label, i == w.tab));
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
                                "Escalate urgent support email",
                                "Runs on every new email to support@corvel.dev",
                            ))
                            .child(self.btn(
                                cx,
                                "wf-run",
                                if w.running(n) { "Stop" } else { "Test run" },
                                true,
                            )),
                    )
                    .child(tabs)
                    .child(list)
                    .child(line(t))
                    .child(text(t, w.outcome(n), muted, 12.))
            }
            AiVariant::ImageGen => {
                let g = self.state.image.clone();
                let (_, aw, ah) = ASPECTS[g.aspect];
                let canvas_h = 300.0 * ah as f32 / aw as f32;
                let tile = |i: usize, h: f32| -> Div {
                    let vs = g.var_st(i, n);
                    let sc = g.vars.get(i).map(|v| v.scene).unwrap_or(0);
                    let cols = SCENES[sc].1;
                    let k = match vs {
                        VarSt::Done => 1.0,
                        VarSt::Forming(p) => (p as f32 - 0.2).max(0.0),
                        _ => 0.25,
                    };
                    let top: Hsla = rgb(u32::from_str_radix(&cols[0][1..], 16).unwrap_or(0)).into();
                    let bot: Hsla = rgb(u32::from_str_radix(&cols[2][1..], 16).unwrap_or(0)).into();
                    div()
                        .h(px(h))
                        .w(px(h * aw as f32 / ah as f32))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .bg(c(t, ColorRole::SurfaceRaised))
                        .flex()
                        .flex_col()
                        .child(div().flex_1().bg(top.opacity(k)))
                        .child(div().h(relative(0.42)).bg(bot.opacity(k)))
                };
                let mut strip = row();
                for i in 0..4 {
                    strip = strip.child(
                        self.btn(cx, &format!("ig-pick-{i}"), "", false)
                            .p(px(0.))
                            .h_auto()
                            .border_0()
                            .child(tile(i, 46.))
                            .when(i == g.chosen, |d| d.border_1().border_color(fg)),
                    );
                }
                let mut models = row();
                for (i, (name, _)) in IG_MODELS.iter().enumerate() {
                    models =
                        models.child(self.chip(cx, &format!("ig-model-{i}"), *name, i == g.model));
                }
                let mut aspects = row();
                for (i, (lab, _, _)) in ASPECTS.iter().enumerate() {
                    aspects = aspects.child(self.chip(
                        cx,
                        &format!("ig-aspect-{i}"),
                        *lab,
                        i == g.aspect,
                    ));
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        row()
                            .justify_between()
                            .child(text(t, g.status(n), ColorRole::FgDefault, 12.5))
                            .child(text(t, g.size_label(n), muted, 12.)),
                    )
                    .child(tile(g.chosen, canvas_h.min(320.)))
                    .child(
                        row().justify_between().child(strip).child(
                            row()
                                .child(self.btn(cx, "ig-upscale", "Upscale", false))
                                .child(self.btn(cx, "ig-vary", "Vary", false))
                                .child(self.btn(cx, "ig-download", "Download", false)),
                        ),
                    )
                    .child(text(t, g.prompt.clone(), ColorRole::FgDefault, 13.))
                    .child(
                        row()
                            .flex_wrap()
                            .child(models)
                            .child(aspects)
                            .child(if g.busy(n) {
                                self.btn(cx, "ig-stop", "Stop", false)
                            } else {
                                self.btn(cx, "ig-gen", "Generate", true)
                            }),
                    )
            }
            AiVariant::Speech => {
                let s = self.state.speech.clone();
                let ph = s.phase_at(n);
                let cur = s.word_at(n);
                let mut words = div().flex().flex_wrap().gap(px(4.));
                for (i, (w, _)) in s.words().iter().enumerate() {
                    let col = if s.open && i == cur {
                        ColorRole::FgDefault
                    } else if s.open && i > cur {
                        ColorRole::FgSubtle
                    } else {
                        ColorRole::FgDefault
                    };
                    words =
                        words.child(text(t, w.clone(), col, 14.).when(s.open && i == cur, |d| {
                            d.bg(c(t, ColorRole::SurfaceSelected)).rounded(px(3.))
                        }));
                }
                let (el, tot) = s.times(n);
                let controls = if s.open {
                    row()
                        .child(self.btn(cx, "sp-play", s.play_name(n), false))
                        .child(text(
                            t,
                            if ph == SpPhase::Preparing {
                                format!("Preparing {}'s voice", VOICES[s.voice].0)
                            } else {
                                el
                            },
                            muted,
                            12.,
                        ))
                        .child(bar(t, s.pos_at(n) as f64 / s.total.max(1) as f64, fg))
                        .child(text(t, tot, muted, 12.))
                        .child(self.chip(cx, "sp-speed", s.speed_label(), false))
                        .child(self.chip(
                            cx,
                            &format!("sp-voice-{}", (s.voice + 1) % VOICES.len()),
                            VOICES[s.voice].0,
                            false,
                        ))
                        .child(self.btn(cx, "sp-stop", "Stop reading", false))
                } else {
                    row()
                        .child(self.btn(cx, "sp-read", "Read aloud", false))
                        .child(self.btn(cx, "sp-copy", "Copy", false))
                };
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(row().child(mark("claude", 16., fg)).child(text(
                        t,
                        "Claude Sonnet 4.5",
                        ColorRole::FgDefault,
                        12.5,
                    )))
                    .child(words)
                    .child(controls)
            }
            AiVariant::Meeting => {
                let m = self.state.meeting.clone();
                let st = m.status_at(n);
                let mut notes = div().flex().flex_col().gap(px(6.));
                if st == MsStatus::Summarizing {
                    notes = notes.child(text(
                        t,
                        "Summarizing 34 minutes of transcript…",
                        muted,
                        12.5,
                    ));
                } else {
                    for i in m.lines() {
                        let l = LINES[i];
                        let vis = m.visible(i, n);
                        if vis == 0 {
                            continue;
                        }
                        let mut r = row()
                            .child(
                                text(t, first_words(l.1, vis), ColorRole::FgDefault, 12.5).flex_1(),
                            )
                            .child(self.chip(
                                cx,
                                &format!("ms-quote-{i}"),
                                l.2,
                                m.quote == Some(i),
                            ));
                        if l.0 == 2 {
                            r = row()
                                .child(self.chip(
                                    cx,
                                    &format!("ms-check-{i}"),
                                    if m.checked[i] { "✓" } else { " " },
                                    m.checked[i],
                                ))
                                .child(r.flex_1());
                            r = match m.send_st(i, n) {
                                SendSt::None => r
                                    .child(self.btn(cx, &format!("ms-linear-{i}"), "Linear", false))
                                    .child(self.btn(
                                        cx,
                                        &format!("ms-notion-{i}"),
                                        "Notion",
                                        false,
                                    )),
                                SendSt::Sending(_) => r.child(pill(t, "Sending", "blue")),
                                SendSt::Sent(tg, num) => {
                                    r.child(pill(t, Meeting::reference(tg, num), "green"))
                                }
                                SendSt::Failed(_) => r.child(pill(t, "Failed", "red")),
                            };
                        }
                        notes = notes.child(r);
                        if m.quote == Some(i) {
                            notes = notes.child(text(t, format!("{}: “{}”", l.3, l.4), muted, 12.));
                        }
                    }
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(row().child(mark(MEETING.1, 20., fg)).child(title(
                        t,
                        MEETING.0,
                        format!("Zoom · {} · {} min", MEETING.2, MEETING.3),
                    )))
                    .child(notes)
                    .child(line(t))
                    .child(
                        row()
                            .child(self.chip(cx, "ms-brief", "Brief", m.brief))
                            .child(self.btn(cx, "ms-regen", "Regenerate", false)),
                    )
            }
            AiVariant::Reply => {
                let r = self.state.reply.clone();
                let ph = r.phase(n);
                let mut sugs = row();
                for (i, s) in oxyn_core::aikit::reply::SUGGESTIONS.iter().enumerate() {
                    sugs = sugs.child(self.chip(cx, &format!("rd-sug-{i}"), *s, i == r.sug));
                }
                let mut tones = row();
                for (i, s) in TONES.iter().enumerate() {
                    tones = tones.child(self.chip(cx, &format!("rd-tone-{i}"), *s, i == r.tone));
                }
                for (i, s) in LENGTHS.iter().enumerate() {
                    tones = tones.child(self.chip(cx, &format!("rd-len-{i}"), *s, i == r.len));
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(title(
                        t,
                        MESSAGE.2,
                        format!("{} · {}", MESSAGE.0, MESSAGE.1),
                    ))
                    .child(text(t, MESSAGE.3.replace("\n\n", " "), muted, 12.5))
                    .child(line(t))
                    .child(sugs)
                    .child(
                        text(t, r.visible(n), ColorRole::FgDefault, 13.)
                            .when(ph == RdPhase::Rewriting, |d| d.opacity(0.45)),
                    )
                    .child(tones)
                    .child(
                        row()
                            .child(self.btn(cx, "rd-regen", "Regenerate", false))
                            .child(self.btn(
                                cx,
                                "rd-insert",
                                if ph == RdPhase::Inserted {
                                    "Inserted"
                                } else {
                                    "Insert"
                                },
                                true,
                            )),
                    )
            }
            AiVariant::Handoff => {
                let h = self.state.handoff.clone();
                let st = h.stage(n);
                let mut thread = div().flex().flex_col().gap(px(6.));
                for (i, m) in h.msgs.iter().enumerate().filter(|(_, m)| m.at <= n) {
                    let txt = if m.who == Who::Bot {
                        first_words(&m.text, h.words(i, n))
                    } else {
                        m.text.clone()
                    };
                    let mine = m.who == Who::User;
                    thread = thread.child(
                        row()
                            .when(mine, |d| d.justify_end())
                            .when(m.who == Who::System, |d| d.justify_center())
                            .child(
                                div()
                                    .max_w(px(380.))
                                    .px(px(10.))
                                    .py(px(6.))
                                    .rounded(px(10.))
                                    .text_size(px(12.5))
                                    .when(mine, |d| d.bg(fg).text_color(c(t, ColorRole::BgBase)))
                                    .when(!mine && m.who != Who::System, |d| {
                                        d.bg(c(t, ColorRole::SurfaceRaised)).text_color(fg)
                                    })
                                    .when(m.who == Who::System, |d| {
                                        d.text_color(c(t, ColorRole::FgSubtle))
                                    })
                                    .child(txt),
                            ),
                    );
                }
                let mut marks = row();
                for (s, on) in h.marks(n) {
                    marks = marks.child(pill(t, s.label(), if on { "green" } else { "" }));
                }
                let act = match st {
                    Stage::Assistant => row()
                        .child(self.btn(cx, "ho-chip-1", "Can you refund it?", false))
                        .child(self.btn(cx, "ho-human", "Talk to a person", true)),
                    Stage::Transferring => row()
                        .child(text(t, h.wait(), muted, 12.))
                        .child(self.btn(cx, "ho-cancel", "Cancel transfer", false)),
                    Stage::Connected => {
                        row().child(self.btn(cx, "ho-resolve", "Mark resolved", false))
                    }
                    Stage::Unavailable => row()
                        .child(self.btn(cx, "ho-retry", "Try again", false))
                        .child(self.btn(cx, "ho-keep", "Keep chatting", false)),
                    Stage::Resolved => {
                        row().child(text(t, "Resolved", ColorRole::StatusSuccessFg, 12.5))
                    }
                };
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(row().child(mark("intercom", 14., fg)).child(title(
                        t,
                        "Support chat",
                        st.label(),
                    )))
                    .child(marks)
                    .child(thread)
                    .child(act)
            }
            AiVariant::Pii => {
                let p = self.state.pii.clone();
                let ph = p.phase_at(n);
                let red = matches!(ph, PiiPhase::Redacted | PiiPhase::Replying | PiiPhase::Done);
                let mut msg = div().flex().flex_wrap().gap(px(2.));
                for (s, e) in p.segments() {
                    msg = msg.child(match e {
                        Some(e) => pill(t, if red { placeholder(&e) } else { s }, TYPES[e.kind].3),
                        None => text(t, s, ColorRole::FgDefault, 13.),
                    });
                }
                let counts = p.counts();
                let mut types = row();
                for (i, ty) in TYPES.iter().enumerate() {
                    types = types.child(pill(t, format!("{} {}", ty.1, counts[i]), ty.3));
                }
                let next = match ph {
                    PiiPhase::Edit => Some(("pii-scan", "Scan")),
                    PiiPhase::Found => Some(("pii-redact", "Redact")),
                    PiiPhase::Redacted => Some(("pii-send", "Send safe prompt")),
                    PiiPhase::Done => Some(("pii-reset", "Edit message")),
                    _ => None,
                };
                let reply = if p.show_original {
                    p.reply_restored()
                } else {
                    p.reply_safe()
                };
                let mut d = div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        row()
                            .justify_between()
                            .child(title(
                                t,
                                "Redact before sending",
                                "Personal data never reaches the model",
                            ))
                            .child(self.chip(cx, "pii-orig", "Show original", p.show_original)),
                    )
                    .child(msg)
                    .child(types);
                if matches!(ph, PiiPhase::Replying | PiiPhase::Done) {
                    d = d.child(text(
                        t,
                        first_words(&reply, p.reply_words(n)),
                        ColorRole::FgDefault,
                        12.5,
                    ));
                }
                if let Some((id, l)) = next {
                    d = d.child(self.btn(cx, id, l, true));
                }
                d
            }
            AiVariant::Cost => {
                let cs = self.state.cost.clone();
                let mut list = div().flex().flex_col().gap(px(3.));
                for i in cs.ranked() {
                    let m = COST_MODELS[i];
                    list = list.child(
                        self.btn(cx, &format!("lc-sel-{i}"), "", false)
                            .h(px(30.))
                            .border_0()
                            .when(i == cs.selected, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                            .child(mark(m.1, 14., fg))
                            .child(text(t, m.0, ColorRole::FgDefault, 12.5).w(px(150.)))
                            .child(bar(t, cs.share(i), c(t, muted)))
                            .child(
                                text(t, usd(cs.monthly(i)), ColorRole::FgDefault, 12.5).w(px(70.)),
                            ),
                    );
                }
                let mut presets = row();
                for (i, p) in oxyn_core::aikit::cost::PRESETS.iter().enumerate() {
                    presets = presets.child(self.chip(cx, &format!("lc-preset-{i}"), p.0, false));
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(title(t, "What will it cost a month?", cs.volume()))
                    .child(presets)
                    .child(
                        row()
                            .child(self.btn(
                                cx,
                                &format!("lc-set-3-{}", cs.cache.saturating_sub(10)),
                                "Cache −10%",
                                false,
                            ))
                            .child(text(
                                t,
                                format!("{}% cache hits", cs.cache),
                                ColorRole::FgDefault,
                                12.5,
                            ))
                            .child(self.btn(
                                cx,
                                &format!("lc-set-3-{}", (cs.cache + 10).min(100)),
                                "Cache +10%",
                                false,
                            )),
                    )
                    .child(list)
                    .child(text(t, cs.summary(), ColorRole::FgDefault, 12.5))
            }
            AiVariant::Keys => {
                let k = self.state.keys.clone();
                let mut list = div().flex().flex_col().gap(px(8.));
                for (i, key) in k.keys.iter().enumerate() {
                    let p = PROVIDERS[key.provider];
                    let (h, st) = match key.status {
                        KeySt::Active => ("green", "Active"),
                        KeySt::Invalid => ("red", "Invalid"),
                        KeySt::Limited => ("amber", "Near limit"),
                    };
                    list = list.child(
                        row()
                            .child(mark(p.0, 18., fg))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        row()
                                            .child(text(t, p.1, ColorRole::FgDefault, 13.))
                                            .child(pill(t, st, h)),
                                    )
                                    .child(text(
                                        t,
                                        if k.revealed(i, n) {
                                            key.secret.clone()
                                        } else {
                                            key.masked()
                                        },
                                        ColorRole::FgMuted,
                                        12.,
                                    ))
                                    .child(text(
                                        t,
                                        format!("Used {}", key.used),
                                        ColorRole::FgSubtle,
                                        11.5,
                                    )),
                            )
                            .child(self.btn(
                                cx,
                                &format!("pk-reveal-{i}"),
                                if k.revealed(i, n) {
                                    format!("{}", k.reveal_left(n))
                                } else {
                                    "Reveal".into()
                                },
                                false,
                            ))
                            .child(self.btn(cx, &format!("pk-rotate-{i}"), "Rotate", false))
                            .child(self.btn(cx, &format!("pk-remove-{i}"), "Remove", false)),
                    );
                }
                let mut d = div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        row()
                            .justify_between()
                            .child(title(t, "Provider keys", "Bring your own keys"))
                            .child(self.btn(cx, "pk-add", "Add key", false)),
                    )
                    .child(list)
                    .child(line(t))
                    .child(text(t, k.footer(), muted, 12.));
                if k.can_undo(n) {
                    d = d.child(self.btn(cx, "pk-undo", "Undo remove", false));
                }
                d
            }
            AiVariant::GenStats => {
                let g = self.state.stats.clone();
                let st = g.status(n);
                let m = GS_MODELS[g.model];
                let mut d = div().flex().flex_col().gap(px(10.)).child(
                    row()
                        .flex_wrap()
                        .gap(px(14.))
                        .child(row().child(mark(m.1, 15., fg)).child(text(
                            t,
                            m.0,
                            ColorRole::FgDefault,
                            12.5,
                        )))
                        .child(pill(
                            t,
                            st.label(),
                            match st {
                                oxyn_core::aikit::genstats::Status::Done => "green",
                                oxyn_core::aikit::genstats::Status::Error => "red",
                                _ => "blue",
                            },
                        ))
                        .child(text(
                            t,
                            format!("TTFT {}", g.ttft(n)),
                            ColorRole::FgDefault,
                            12.5,
                        ))
                        .child(text(
                            t,
                            format!("{:.0} tok/s", g.speed(n)),
                            ColorRole::FgDefault,
                            12.5,
                        ))
                        .child(text(
                            t,
                            format!("{} tokens", thousands(g.tokens(n))),
                            ColorRole::FgDefault,
                            12.5,
                        ))
                        .child(text(t, usd(g.cost(n)), ColorRole::FgDefault, 12.5))
                        .child(self.btn(cx, "gs-run", "Run again", false))
                        .child(self.chip(cx, "gs-expand", "Breakdown", g.expanded)),
                );
                if g.expanded {
                    for (lab, ms, share) in g.breakdown(n) {
                        d = d.child(
                            row()
                                .child(text(t, lab, muted, 12.).w(px(140.)))
                                .child(bar(t, share, c(t, ColorRole::StatusInfoFg)))
                                .child(
                                    text(t, format!("{ms} ms"), ColorRole::FgDefault, 12.)
                                        .w(px(64.)),
                                ),
                        );
                    }
                }
                d
            }
            AiVariant::Translate => {
                let tr = self.state.translate.clone();
                let mut langs = row().flex_wrap();
                for (i, (_, name)) in LANGS.iter().enumerate() {
                    langs = langs.child(self.chip(
                        cx,
                        &format!("tr-lang-{i}"),
                        if i == 0 { "Original" } else { *name },
                        i == tr.lang,
                    ));
                }
                let mut txt = String::new();
                for (i, orig) in TEXT[0].iter().enumerate() {
                    if tr.lang == 0 || tr.original {
                        txt.push_str(orig);
                    } else if tr.done_sentence(i, n) {
                        txt.push_str(TEXT[tr.lang][i]);
                    } else if tr.shown(i, n) > 0 {
                        txt.push_str(&tr.sentence(i, n));
                    } else {
                        txt.push_str(orig);
                    }
                    txt.push(' ');
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(title(t, "Rina Okafor", "#launch · Detected English"))
                    .child(langs)
                    .child(text(t, txt, ColorRole::FgDefault, 14.))
                    .child(
                        row()
                            .justify_between()
                            .child(text(t, tr.status_line(n), muted, 12.))
                            .child(self.chip(cx, "tr-orig", "Show original", tr.original)),
                    )
            }
            AiVariant::Tldr => {
                let td = self.state.tldr.clone();
                let mut modes = row();
                for (i, m) in MODES.iter().enumerate() {
                    modes = modes.child(self.chip(cx, &format!("td-mode-{i}"), *m, i == td.mode));
                }
                let mut body = div().flex().flex_col().gap(px(8.));
                match td.phase(n) {
                    "full" => {
                        for (i, p) in PARAS.iter().enumerate() {
                            body = body.child(
                                text(t, *p, ColorRole::FgMuted, 12.5)
                                    .when(td.highlighted(n) == Some(i), |d| {
                                        d.bg(c(t, ColorRole::SurfaceSelected)).text_color(fg)
                                    }),
                            );
                        }
                    }
                    "thinking" => body = body.child(text(t, "Reading the article…", muted, 12.5)),
                    "error" => body = body.child(self.btn(cx, "td-retry", "Try again", false)),
                    _ if td.mode == 1 => {
                        body = body.child(text(
                            t,
                            first_words(SUMMARIES[td.len], td.visible(n)),
                            ColorRole::FgDefault,
                            13.5,
                        ))
                    }
                    _ => {
                        for (i, (pt, _)) in td.points().iter().enumerate() {
                            body = body.child(
                                row()
                                    .child(text(
                                        t,
                                        format!("{}. {pt}", i + 1),
                                        ColorRole::FgDefault,
                                        13.,
                                    ))
                                    .child(self.btn(cx, &format!("td-jump-{i}"), "Jump", false)),
                            );
                        }
                    }
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(title(
                        t,
                        oxyn_core::aikit::tldr::TITLE,
                        oxyn_core::aikit::tldr::BYLINE,
                    ))
                    .child(modes)
                    .child(body)
            }
            AiVariant::Connector => {
                let cs = self.state.connectors.clone();
                let mut list = div().flex().flex_col().gap(px(8.));
                for i in cs.rows() {
                    let a = APPS[i];
                    let st = cs.st(i, n);
                    let action = match st {
                        ConnSt::Off => self
                            .btn(cx, &format!("cs-connect-{i}"), "Connect", false)
                            .into_any_element(),
                        ConnSt::On => self
                            .btn(cx, &format!("cs-disc-{i}"), "Disconnect", false)
                            .into_any_element(),
                        ConnSt::Denied => self
                            .btn(cx, &format!("cs-retry-{i}"), "Retry", false)
                            .into_any_element(),
                        ConnSt::Confirm => row()
                            .child(self.btn(cx, &format!("cs-yes-{i}"), "Disconnect", true))
                            .child(self.btn(cx, "cs-no", "Keep", false))
                            .into_any_element(),
                        _ => pill(t, "Connecting", "blue").into_any_element(),
                    };
                    let prog = match st {
                        ConnSt::Syncing(f) => Some(f),
                        _ => None,
                    };
                    list = list.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(
                                row()
                                    .child(mark(a.0, 18., fg))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .child(text(t, a.1, ColorRole::FgDefault, 13.))
                                            .child(text(
                                                t,
                                                cs.line(i, n),
                                                ColorRole::FgSubtle,
                                                12.,
                                            )),
                                    )
                                    .child(action),
                            )
                            .when_some(prog, |d, f| d.child(bar(t, f, fg))),
                    );
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(title(
                        t,
                        "Connect your tools",
                        format!("{} of {} connected", cs.connected(n), APPS.len()),
                    ))
                    .child(list)
            }
            AiVariant::Enrich => {
                let e = self.state.enrich.clone();
                let mut table = div().flex().flex_col();
                for (i, (name, dom)) in ROWS.iter().enumerate() {
                    let cell: Div = match e.st(i, n) {
                        CellSt::Empty => text(t, "—", muted, 12.5),
                        CellSt::Queued => text(t, "Queued", muted, 12.5),
                        CellSt::Running => text(t, "Looking up…", ColorRole::StatusInfoFg, 12.5),
                        CellSt::Done => text(
                            t,
                            format!("{} · {dom}", e.value(i)),
                            ColorRole::FgDefault,
                            12.5,
                        ),
                        CellSt::Failed => row()
                            .child(text(t, "Not found", ColorRole::StatusDangerFg, 12.5))
                            .child(self.btn(cx, &format!("ec-retry-{i}"), "Retry", false)),
                    };
                    table = table.child(
                        row()
                            .h(px(34.))
                            .child(text(t, *name, ColorRole::FgDefault, 12.5).w(px(150.)))
                            .child(text(t, *dom, ColorRole::FgSubtle, 12.).w(px(130.)))
                            .child(cell.flex_1()),
                    );
                }
                let mut cols = row();
                for (i, col) in oxyn_core::aikit::enrich::COLUMNS.iter().enumerate() {
                    cols = cols.child(self.chip(cx, &format!("ec-col-{i}"), col.0, i == e.column));
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
                                "Accounts",
                                oxyn_core::aikit::enrich::COLUMNS[e.column].1,
                            ))
                            .child(self.btn(cx, "ec-run", "Run column", true)),
                    )
                    .child(cols)
                    .child(table)
                    .child(text(t, e.footer(n), muted, 12.))
            }
            AiVariant::Router => {
                let r = self.state.router.clone();
                let counts = r.counts(n);
                let cur = r.current(n).map(|i| {
                    let tk = TICKETS[i];
                    format!(
                        "{} · {} → {} ({} · {} · {}%)",
                        tk.1,
                        tk.2,
                        LANES[r.lane(i)].0,
                        tk.3,
                        tk.4,
                        tk.5
                    )
                });
                let mut lanes = row().items_start();
                for (l, (name, h)) in LANES.iter().enumerate() {
                    let mut col = div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .flex_1()
                        .min_w(px(0.))
                        .p(px(8.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(
                            row()
                                .child(div().size(px(7.)).rounded(px(4.)).bg(hue(t, h)))
                                .child(text(
                                    t,
                                    format!("{name} {}", counts[l]),
                                    ColorRole::FgDefault,
                                    12.5,
                                )),
                        );
                    for i in r.in_lane(l, n).into_iter().take(3) {
                        col = col.child(
                            row().child(mark(TICKETS[i].0, 12., fg)).child(
                                text(t, TICKETS[i].2, ColorRole::FgMuted, 11.5)
                                    .overflow_hidden()
                                    .whitespace_nowrap(),
                            ),
                        );
                        if l == 3 {
                            col = col.child(self.btn(
                                cx,
                                &format!("rt-review-{i}-0"),
                                "To Billing",
                                false,
                            ));
                        }
                    }
                    lanes = lanes.child(col);
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
                                "Support inbox router",
                                format!("Review below {}%", r.threshold),
                            ))
                            .child(
                                row()
                                    .child(self.btn(
                                        cx,
                                        "rt-pause",
                                        if r.paused.is_some() {
                                            "Resume"
                                        } else {
                                            "Pause"
                                        },
                                        false,
                                    ))
                                    .child(self.btn(cx, "rt-restart", "Restart", false)),
                            ),
                    )
                    .child(text(
                        t,
                        cur.unwrap_or_else(|| "Waiting for tickets".into()),
                        ColorRole::FgDefault,
                        12.5,
                    ))
                    .child(lanes)
            }
        };
        let said = self.state.said.clone();
        div()
            .id("oxyn-aikit")
            .key_context("OxynAiKit")
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
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .bg(c(t, ColorRole::SurfaceBase))
                    .child(body),
            )
            .child(
                text(
                    t,
                    format!("{} · {}", secs(n), said),
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
    fn c1d_presses_and_keys_route_to_each_model() {
        let mut s = AiKitState::default();
        s.press(AiVariant::Timeline, "tl-span-4");
        assert_eq!(s.timeline.pinned, Some(4));
        s.key(AiVariant::Timeline, "escape", false, false);
        assert_eq!(s.timeline.pinned, None);
        s.key(AiVariant::ModelCard, "end", false, false);
        assert_eq!(s.model.vol, 8);
        s.press(AiVariant::Cost, "lc-preset-2");
        assert_eq!(s.cost.input, 40_000);
        s.now += 60_000;
        s.press(AiVariant::Keys, "pk-remove-0");
        assert!(s.said.contains("Undo"));
        s.key(AiVariant::Tldr, "right", false, false);
        assert_eq!(s.tldr.mode, 2);
        s.key(AiVariant::Reply, "enter", false, true);
        assert!(s.reply.inserted);
        s.press(AiVariant::Router, "rt-pause");
        assert!(s.router.paused.is_some());
        assert_eq!(AiVariant::ALL.len(), 17);
        assert_eq!(AiVariant::Router.slug(), "ticket-router");
    }
}
