//! Agent micro-interactions from saved posts (C2a) in GPUI as one live view: Test ticks, Triage
//! metro, Secret card, Proactive card, Write sweep, Done shockwave, Split confirm, Publish
//! popover, Agent prompt copy and Loop pipeline. Every variant runs on the same
//! `oxyn_core::xpost` models as the Dioxus components, so presses and keys behave the same in both
//! renderers. The view keeps its own clock (a tick every 50 ms while it lives); a host can also
//! drive it with [`XPostView::advance`]. The publish address is shown as text (no editing here).

use std::time::Duration;

use gpui::{
    App, BorderStyle, Bounds, ClickEvent, Context, Div, FocusHandle, Focusable, FontWeight, Hsla,
    IntoElement, KeyDownEvent, ParentElement, PathBuilder, Render, SharedString, Stateful, Styled,
    Window, canvas, div, point, prelude::*, px, quad, relative, size,
};
use oxyn_core::aikit::{norm_key, secs};
use oxyn_core::xpost::{
    XP_SLUGS,
    loops::{Loops, STATIONS as LP_STATIONS, TASK as LP_TASK},
    metro::{FINDINGS, Metro, OUTCOMES, STATIONS as M_STATIONS, TERM_X, TERM_Y, W as MW, path},
    proactive::{OFFER, Phase as PcPhase, Proactive, RESULT, SNOOZE, STEPS as PC_STEPS},
    promptcopy::{AGENTS, FORMATS, PromptCopy},
    publish::{ACCESS, DOC, HOSTS, Phase as PpPhase, Publish, STEPS as PP_STEPS},
    secret::{Ask, SCOPES, SECRET, Secret, USED_BY},
    shock::{Phase as SwPhase, STEPS as SW_STEPS, Shock, TASK as SW_TASK},
    split::{LABELS as SP_LABELS, Phase as SpPhase, Split},
    sweep::{ASK as WS_ASK, ROWS as WS_ROWS, Sweep},
    ticks::{FILES, FIX, SUITE, TestTicks, Tick, total},
};
use oxyn_tokens::{ColorRole, Theme};

use crate::aikit::{bar, c, hue, mark, pill, row, text, title};

/// Which component the view draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XPostVariant {
    /// Test ticks.
    TestTicks,
    /// Triage metro.
    TriageMetro,
    /// Secret card.
    SecretCard,
    /// Proactive card.
    ProactiveCard,
    /// Write sweep.
    WriteSweep,
    /// Done shockwave.
    DoneShockwave,
    /// Split confirm.
    SplitConfirm,
    /// Publish popover.
    PublishPopover,
    /// Agent prompt copy.
    PromptCopy,
    /// Loop pipeline.
    LoopPipeline,
}

impl XPostVariant {
    /// Every variant, in page order.
    pub const ALL: [XPostVariant; 10] = [
        XPostVariant::TestTicks,
        XPostVariant::TriageMetro,
        XPostVariant::SecretCard,
        XPostVariant::ProactiveCard,
        XPostVariant::WriteSweep,
        XPostVariant::DoneShockwave,
        XPostVariant::SplitConfirm,
        XPostVariant::PublishPopover,
        XPostVariant::PromptCopy,
        XPostVariant::LoopPipeline,
    ];

    /// The page slug.
    pub fn slug(self) -> &'static str {
        XP_SLUGS[Self::ALL.iter().position(|&v| v == self).unwrap_or(0)]
    }
}

/// The state of every component on one clock; pure, so presses and keys are testable without a
/// window.
#[derive(Clone, Debug)]
pub struct XPostState {
    /// Clock, ms.
    pub now: u64,
    /// Test ticks.
    pub ticks: TestTicks,
    /// Triage metro.
    pub metro: Metro,
    /// Secret card.
    pub secret: Secret,
    /// Proactive card.
    pub proactive: Proactive,
    /// Write sweep.
    pub sweep: Sweep,
    /// Done shockwave.
    pub shock: Shock,
    /// Split confirm.
    pub split: Split,
    /// Publish popover.
    pub publish: Publish,
    /// Agent prompt copy.
    pub copy: PromptCopy,
    /// Loop pipeline.
    pub loops: Loops,
    /// Publish popover keys go to this part (`access` or `host`).
    pub publish_part: &'static str,
    /// Last announcement.
    pub said: String,
}

impl Default for XPostState {
    fn default() -> Self {
        Self {
            now: 0,
            ticks: TestTicks::new(true, 1.0, 0),
            metro: Metro::new(true, 0),
            secret: Secret::new(false),
            proactive: Proactive::new(false),
            sweep: Sweep::new(1_800, true, 0),
            shock: Shock::new(false, 1.0, 1.0, true, 0),
            split: Split::new(0, false),
            publish: Publish::new(true, false),
            copy: PromptCopy::new(0, 0, false),
            loops: Loops::new(85, 5, 0),
            publish_part: "host",
            said: String::new(),
        }
    }
}

impl XPostState {
    /// Routes a press id to the model of `v`.
    pub fn press(&mut self, v: XPostVariant, id: &str) {
        let n = self.now;
        let out = match v {
            XPostVariant::TestTicks => self.ticks.press(id, n),
            XPostVariant::TriageMetro => self.metro.press(id, n),
            XPostVariant::SecretCard => self.secret.press(id, n),
            XPostVariant::ProactiveCard => self.proactive.press(id, n),
            XPostVariant::WriteSweep => self.sweep.press(id, n),
            XPostVariant::DoneShockwave => self.shock.press(id, n),
            XPostVariant::SplitConfirm => self.split.press(id, n),
            XPostVariant::PublishPopover => {
                if id.starts_with("pp-access") {
                    self.publish_part = "access";
                } else if id.starts_with("pp-host") {
                    self.publish_part = "host";
                }
                self.publish.press(id, n)
            }
            XPostVariant::PromptCopy => self.copy.press(id, n),
            XPostVariant::LoopPipeline => self.loops.press(id, n),
        };
        if let Some(s) = out {
            self.said = s;
        }
    }

    /// Routes a key (`left`, `enter`, `escape`, `space`; `undo` for Cmd/Ctrl+Z).
    pub fn key(&mut self, v: XPostVariant, key: &str, cmd: bool) {
        let n = self.now;
        let k = if cmd && key.eq_ignore_ascii_case("z") {
            "undo"
        } else if key == "backspace" {
            "Backspace"
        } else {
            norm_key(key)
        };
        let out = match v {
            XPostVariant::TestTicks => self.ticks.key(k, n),
            XPostVariant::TriageMetro => self.metro.key(k, n),
            XPostVariant::SecretCard => self.secret.key(k, n),
            XPostVariant::ProactiveCard => self.proactive.key(k, n),
            XPostVariant::WriteSweep => self.sweep.key(k, n),
            XPostVariant::DoneShockwave => self.shock.key(k, n),
            XPostVariant::SplitConfirm => self.split.key(k, n),
            XPostVariant::PublishPopover => self.publish.key(self.publish_part, k, n),
            XPostVariant::PromptCopy => self.copy.key(k, n),
            XPostVariant::LoopPipeline => self.loops.key(k, n),
        };
        if let Some(s) = out {
            self.said = s;
        }
    }
}

/// The C2a components as one live view.
pub struct XPostView {
    variant: XPostVariant,
    state: XPostState,
    dark: bool,
    focus: FocusHandle,
}

impl Focusable for XPostView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl XPostView {
    /// A view of `variant`, focused so the keys work at once, ticking its own clock.
    pub fn new(variant: XPostVariant, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if this
                    .update(cx, |v, cx| {
                        v.state.now += 50;
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
            state: XPostState::default(),
            dark: true,
            focus,
        }
    }

    /// Switch variant.
    pub fn set_variant(&mut self, v: XPostVariant, cx: &mut Context<Self>) {
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
    pub fn state(&self) -> &XPostState {
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
        self.state.key(self.variant, &k, m.platform || m.control);
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
            .id(SharedString::from(format!("xp-{id}")))
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

    fn stat(&self, v: impl Into<SharedString>, label: &str, h: &str) -> Div {
        let t = self.theme();
        let col = if h == "neutral" {
            c(t, ColorRole::FgDefault)
        } else {
            hue(t, h)
        };
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(col)
                    .child(v.into()),
            )
            .child(text(t, label.to_string(), ColorRole::FgSubtle, 11.))
    }

    fn ticks(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.ticks;
        let (p, f, d) = s.counts(n);
        let ended = s.done(n) && f > 0;
        let mut grid = div()
            .flex()
            .flex_col()
            .gap(px(7.))
            .p(px(10.))
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceRaised));
        let mut start = 0;
        for (name, cnt) in FILES.iter() {
            let lit = (start..start + cnt).any(|i| s.tick(i, n) == Tick::Fail);
            if s.only_failed && !lit {
                start += cnt;
                continue;
            }
            let mut ticks = div().flex().flex_wrap().gap(px(3.)).flex_1();
            for i in start..start + cnt {
                let st = s.tick(i, n);
                let (bg, bd) = match st {
                    Tick::Queued => (
                        c(t, ColorRole::SurfaceSelected),
                        c(t, ColorRole::SurfaceSelected),
                    ),
                    Tick::Running => (c(t, ColorRole::SurfaceRaised), c(t, ColorRole::FgMuted)),
                    Tick::Pass => (hue(t, "green"), hue(t, "green")),
                    Tick::Fail => (hue(t, "red"), hue(t, "red")),
                };
                let on = s.sel == Some(i);
                ticks = ticks.child(
                    self.btn(cx, &format!("tt-tick-{i}"), "", false)
                        .w(px(9.))
                        .h(px(if st == Tick::Fail { 18. } else { 16. }))
                        .px(px(0.))
                        .rounded(px(2.5))
                        .bg(bg)
                        .border_color(if on { c(t, ColorRole::FgDefault) } else { bd }),
                );
            }
            grid = grid.child(
                row()
                    .gap(px(10.))
                    .when(ended && !lit, |d| d.opacity(0.38))
                    .child(div().w(px(128.)).flex_none().child(text(
                        t,
                        *name,
                        ColorRole::FgMuted,
                        12.,
                    )))
                    .child(ticks),
            );
            start += cnt;
        }
        let mut out = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        "Running tests",
                        format!("{} · {}", SUITE.0, SUITE.1),
                    ))
                    .child(if s.done(n) || s.stopped.is_some() {
                        self.btn(cx, "tt-run", "Run again", false)
                    } else {
                        self.btn(cx, "tt-stop", "Stop", false)
                    }),
            )
            .child(
                row()
                    .gap(px(20.))
                    .child(self.stat(secs(s.elapsed(n)), "elapsed", "neutral"))
                    .child(self.stat(p.to_string(), "passed", "green"))
                    .child(self.stat(
                        f.to_string(),
                        "failed",
                        if f > 0 { "red" } else { "neutral" },
                    ))
                    .child(self.stat(format!("{:.1}", s.rate(n)), "tests/s", "neutral"))
                    .child(self.stat(format!("{d}/{}", total()), "done", "neutral")),
            )
            .child(grid);
        if s.fix_open(n) {
            let mut diff = div()
                .flex()
                .flex_col()
                .p(px(8.))
                .rounded(px(8.))
                .bg(c(t, ColorRole::SurfaceRaised));
            for (sign, line) in FIX {
                let col = match sign {
                    '-' => hue(t, "red"),
                    '+' => hue(t, "green"),
                    _ => c(t, ColorRole::FgMuted),
                };
                diff = diff.child(
                    div()
                        .text_size(px(11.5))
                        .text_color(col)
                        .child(format!("{sign} {line}")),
                );
            }
            out = out.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(10.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .child(
                        row()
                            .justify_between()
                            .child(text(
                                t,
                                format!("Fix for {}", FILES[1].0),
                                ColorRole::FgDefault,
                                13.,
                            ))
                            .child(self.btn(cx, "tt-fix-close", "Close", false)),
                    )
                    .child(diff)
                    .child(self.btn(cx, "tt-fix", "Apply fix and re-run", true)),
            );
        }
        out.child(
            row()
                .justify_between()
                .child(text(t, s.status(n), ColorRole::FgMuted, 12.5))
                .child(self.chip(cx, "tt-filter", "Failed only", s.only_failed)),
        )
    }

    fn metro(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let m = self.state.metro.clone();
        let lines: Vec<(Vec<(f64, f64)>, Hsla)> = (0..3)
            .map(|o| {
                let h = OUTCOMES[o].1;
                let col = if h == "neutral" {
                    c(t, ColorRole::BorderDefault)
                } else {
                    hue(t, h).opacity(0.6)
                };
                (path(o)[1..].to_vec(), col)
            })
            .chain(std::iter::once((
                vec![(M_STATIONS[0].0, 100.0), (M_STATIONS[1].0, 100.0)],
                c(t, ColorRole::BorderDefault),
            )))
            .collect();
        let dots: Vec<((f64, f64), Hsla, bool)> = (0..FINDINGS.len())
            .filter_map(|i| {
                let p = m.point(i, n)?;
                let col = if m.landed(i, n) {
                    let h = OUTCOMES[m.outcome(i)].1;
                    if h == "neutral" {
                        c(t, ColorRole::FgMuted)
                    } else {
                        hue(t, h)
                    }
                } else {
                    c(t, ColorRole::FgDefault)
                };
                Some((p, col, m.open == Some(i)))
            })
            .collect();
        let stn = c(t, ColorRole::FgMuted);
        let card = c(t, ColorRole::SurfaceRaised);
        let terms: Vec<Hsla> = (0..3)
            .map(|o| {
                let h = OUTCOMES[o].1;
                if h == "neutral" {
                    c(t, ColorRole::FgMuted)
                } else {
                    hue(t, h)
                }
            })
            .collect();
        let map = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let s = f32::from(bounds.size.width) / MW as f32;
                let at = |x: f64, y: f64| {
                    point(
                        bounds.origin.x + px(x as f32 * s),
                        bounds.origin.y + px(y as f32 * s),
                    )
                };
                for (pts, col) in &lines {
                    let mut b = PathBuilder::stroke(px(5. * s));
                    for (i, (x, y)) in pts.iter().enumerate() {
                        if i == 0 {
                            b.move_to(at(*x, *y))
                        } else {
                            b.line_to(at(*x, *y))
                        }
                    }
                    if let Ok(p) = b.build() {
                        window.paint_path(p, *col);
                    }
                }
                let disc =
                    |window: &mut Window, x: f64, y: f64, r: f32, bg: Hsla, bw: f32, bd: Hsla| {
                        let o = at(x, y);
                        window.paint_quad(quad(
                            Bounds::new(
                                point(o.x - px(r * s), o.y - px(r * s)),
                                size(px(2. * r * s), px(2. * r * s)),
                            ),
                            px(r * s),
                            bg,
                            px(bw),
                            bd,
                            BorderStyle::default(),
                        ));
                    };
                for (x, _) in M_STATIONS {
                    disc(window, x, 100.0, 7.0, card, 2.5, stn);
                }
                for (o, col) in terms.iter().enumerate() {
                    disc(window, TERM_X, TERM_Y[o], 9.0, card, 3.0, *col);
                }
                for ((x, y), col, open) in &dots {
                    disc(
                        window,
                        *x,
                        *y,
                        if *open { 7.0 } else { 5.0 },
                        *col,
                        2.0,
                        card,
                    );
                }
            },
        )
        .w_full()
        .h(px(200.));
        let counts = m.counts(n);
        let mut pick = row().flex_wrap();
        for i in 0..m.seen(n) {
            pick = pick.child(self.chip(
                cx,
                &format!("tm-dot-{i}"),
                format!("{}", i + 1),
                m.open == Some(i),
            ));
        }
        let mut out = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        "Security triage",
                        "quillworks-api · 974 files · 6 checkers",
                    ))
                    .child(self.btn(
                        cx,
                        "tm-pause",
                        if m.paused.is_some() {
                            "Resume"
                        } else {
                            "Pause"
                        },
                        false,
                    )),
            )
            .child(
                div()
                    .p(px(8.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .bg(card)
                    .child(map),
            )
            .child(
                row()
                    .gap(px(22.))
                    .child(self.stat(counts[0].to_string(), OUTCOMES[0].0, "green"))
                    .child(self.stat(counts[1].to_string(), OUTCOMES[1].0, "neutral"))
                    .child(self.stat(counts[2].to_string(), OUTCOMES[2].0, "amber")),
            )
            .child(pick);
        if let Some(i) = m.open {
            let f = FINDINGS[i];
            let mut card_el = div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .child(text(t, f.0, ColorRole::FgDefault, 13.5))
                .child(
                    row()
                        .child(pill(
                            t,
                            f.2,
                            match f.2 {
                                "High" => "red",
                                "Medium" => "amber",
                                _ => "neutral",
                            },
                        ))
                        .child(text(t, f.1, ColorRole::FgMuted, 12.))
                        .child(text(
                            t,
                            if m.landed(i, n) {
                                OUTCOMES[m.outcome(i)].0
                            } else {
                                "On its way"
                            },
                            ColorRole::FgMuted,
                            12.,
                        )),
                );
            if m.landed(i, n) && m.outcome(i) == 2 {
                card_el = card_el.child(
                    row()
                        .child(self.btn(cx, "tm-patch", "Approve patch", true))
                        .child(self.btn(cx, "tm-dismiss", "Not a risk", false)),
                );
            }
            out = out.child(card_el);
        }
        out.child(text(t, m.status(n), ColorRole::FgMuted, 12.5))
    }

    fn secret(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = self.state.secret.clone();
        let off = s.is_revoked();
        let mut card = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(12.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .child(
                row()
                    .justify_between()
                    .child(
                        row()
                            .gap(px(8.))
                            .child(
                                div()
                                    .size(px(32.))
                                    .rounded(px(8.))
                                    .bg(gpui::white())
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(mark(SECRET.1, 18., c(t, ColorRole::FgDefault))),
                            )
                            .child(title(t, SECRET.0, "Stripe · live mode")),
                    )
                    .child(pill(
                        t,
                        if off {
                            "Revoked"
                        } else if s.rotation(n).is_some() {
                            "Rotating"
                        } else {
                            "Active"
                        },
                        if off {
                            "red"
                        } else if s.rotation(n).is_some() {
                            "amber"
                        } else {
                            "green"
                        },
                    )),
            )
            .child(
                row()
                    .justify_between()
                    .p(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if s.shown(n) {
                        hue(t, "amber")
                    } else {
                        c(t, ColorRole::BorderSubtle)
                    })
                    .child(text(t, s.value(n), ColorRole::FgDefault, 12.5))
                    .when(!off, |d| {
                        d.child(self.btn(
                            cx,
                            "sc-reveal",
                            if s.shown(n) { "Hide" } else { "Show" },
                            false,
                        ))
                    }),
            );
        if let Some(r) = s.rotation(n) {
            card = card.child(bar(t, r, hue(t, "amber")));
        }
        if s.ask != Ask::None {
            card = card.child(
                row()
                    .justify_between()
                    .p(px(8.))
                    .rounded(px(8.))
                    .bg(hue(t, "amber").opacity(0.08))
                    .child(text(
                        t,
                        match s.ask {
                            Ask::Reveal => "Show the full key for 5 seconds?",
                            Ask::Rotate => "Rotate now?",
                            _ => "Revoke? Both routines will pause.",
                        },
                        ColorRole::FgDefault,
                        12.5,
                    ))
                    .child(
                        row()
                            .child(self.btn(cx, "sc-cancel", "Cancel", false))
                            .child(self.btn(cx, "sc-confirm", "Confirm", true)),
                    ),
            );
        }
        let mut scopes = row().flex_wrap();
        for sc in SCOPES {
            scopes = scopes.child(pill(t, sc, "neutral"));
        }
        let mut used = div().flex().flex_col().gap(px(4.));
        for (r, m) in USED_BY {
            used = used.child(
                row()
                    .justify_between()
                    .child(text(t, r, ColorRole::FgDefault, 12.5))
                    .child(text(
                        t,
                        if off {
                            "Paused".to_string()
                        } else {
                            format!("{m} min ago")
                        },
                        ColorRole::FgSubtle,
                        12.,
                    )),
            );
        }
        let acts = if off {
            if s.can_undo(n) {
                row().child(self.btn(cx, "sc-undo", "Undo", true))
            } else {
                row().child(self.btn(cx, "sc-restore", "Restore", false))
            }
        } else {
            row()
                .child(self.btn(cx, "sc-copy", "Copy handle", false))
                .child(self.btn(cx, "sc-rotate", "Rotate", false))
                .child(
                    self.btn(cx, "sc-revoke", "Revoke", false)
                        .text_color(hue(t, "red")),
                )
        };
        card = card.child(scopes).child(used).child(
            row()
                .justify_between()
                .flex_wrap()
                .child(text(t, s.status(n), ColorRole::FgMuted, 12.))
                .child(acts),
        );
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(text(
                t,
                format!(
                    "I'll issue the refund with {}. The key itself never enters this chat.",
                    Secret::handle()
                ),
                ColorRole::FgMuted,
                12.5,
            ))
            .child(card)
    }

    fn proactive(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let p = self.state.proactive.clone();
        let ph = p.phase(n);
        let body = match ph {
            PcPhase::Gone => row()
                .justify_between()
                .child(text(t, "Suggestion removed", ColorRole::FgMuted, 12.5))
                .child(self.btn(cx, "pc-undo", "Undo", false)),
            PcPhase::Snoozed => row()
                .justify_between()
                .child(text(t, p.folded(), ColorRole::FgMuted, 12.5))
                .child(self.btn(cx, "pc-undo", "Undo", false)),
            _ => {
                let mut card = div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .child(
                        row()
                            .justify_between()
                            .child(
                                row()
                                    .child(text(t, "Assistant ·", ColorRole::FgSubtle, 12.))
                                    .child(mark(OFFER.1, 12., c(t, ColorRole::FgMuted)))
                                    .child(text(t, OFFER.0, ColorRole::FgSubtle, 12.)),
                            )
                            .child(self.btn(cx, "pc-close", "×", false)),
                    )
                    .child(
                        div()
                            .text_size(px(14.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(OFFER.2),
                    );
                let done = p.steps_done(n);
                match ph {
                    PcPhase::Offer => {
                        card = card.child(text(t, OFFER.3, ColorRole::FgMuted, 12.5));
                        let mut acts = row()
                            .child(self.btn(cx, "pc-accept", "Compare and draft", true))
                            .child(self.btn(cx, "pc-later", "Not now", false));
                        if let Some(m) = p.menu {
                            for (i, s) in SNOOZE.iter().enumerate() {
                                acts = acts.child(self.chip(
                                    cx,
                                    &format!("pc-snooze-{i}"),
                                    *s,
                                    m == i,
                                ));
                            }
                        }
                        card = card.child(acts.flex_wrap());
                    }
                    PcPhase::Working | PcPhase::Failed => {
                        for (i, (doing, did)) in PC_STEPS.iter().enumerate() {
                            if i > done {
                                break;
                            }
                            let failed = ph == PcPhase::Failed && i == done.min(1);
                            card = card.child(text(
                                t,
                                if failed {
                                    "✕ Couldn't open the billing page".to_string()
                                } else if i < done {
                                    format!("✓ {did}")
                                } else {
                                    format!("… {doing}")
                                },
                                if failed {
                                    ColorRole::StatusDangerFg
                                } else {
                                    ColorRole::FgMuted
                                },
                                12.5,
                            ));
                        }
                        if ph == PcPhase::Working {
                            card = card.child(bar(t, p.progress(n), c(t, ColorRole::FgDefault)));
                        } else {
                            card = card.child(row().child(self.btn(
                                cx,
                                "pc-retry",
                                "Try again",
                                false,
                            )));
                        }
                    }
                    _ => {
                        card = card
                            .child(text(t, RESULT.0, ColorRole::StatusSuccessFg, 14.))
                            .child(text(t, RESULT.1, ColorRole::FgMuted, 12.5))
                            .child(
                                row()
                                    .child(self.btn(
                                        cx,
                                        "pc-review",
                                        if p.reviewing {
                                            "Hide draft"
                                        } else {
                                            "Review draft"
                                        },
                                        true,
                                    ))
                                    .child(self.btn(cx, "pc-close", "Dismiss", false)),
                            );
                    }
                }
                card
            }
        };
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(text(
                t,
                "Mira Holt · Shipped the new onboarding flow today.",
                ColorRole::FgSubtle,
                12.5,
            ))
            .child(body)
            .child(text(
                t,
                "Owen Park · Anyone else moving their docs to plain markdown?",
                ColorRole::FgSubtle,
                12.5,
            ))
    }

    fn sweep(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = self.state.sweep.clone();
        let band = s.band(n);
        let mut area = div()
            .relative()
            .flex()
            .flex_col()
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .bg(c(t, ColorRole::SurfaceRaised))
            .overflow_hidden();
        let glow = hue(t, "blue");
        for (i, (day, before, after)) in WS_ROWS.iter().enumerate() {
            let changed = before != after && s.crossed(i, n);
            let settle = s
                .since(i, n)
                .map(|x| (x as f32 / 450.).min(1.))
                .unwrap_or(1.);
            let near = band.map(|b| {
                let row_c = (i as f64 + 0.5) / WS_ROWS.len() as f64;
                (1.0 - ((b - row_c).abs() * 6.0)).max(0.0) as f32
            });
            area = area.child(
                row()
                    .gap(px(12.))
                    .px(px(12.))
                    .py(px(8.))
                    .when(i > 0, |d| {
                        d.border_t_1().border_color(c(t, ColorRole::BorderSubtle))
                    })
                    .when_some(near, |d, k| d.bg(glow.opacity(0.18 * k)))
                    .child(
                        div()
                            .w(px(84.))
                            .child(text(t, *day, ColorRole::FgSubtle, 12.)),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(c(t, ColorRole::FgDefault))
                            .opacity(0.25 + 0.75 * settle)
                            .child(s.value(i, n)),
                    )
                    .when(changed, |d| d.border_l_2().border_color(glow)),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        "Week 3 · Half marathon plan",
                        format!("\u{201c}{WS_ASK}\u{201d}"),
                    ))
                    .when(band.is_some(), |d| {
                        d.child(pill(t, "Agent is writing", "blue"))
                    }),
            )
            .child(area)
            .child(
                row()
                    .justify_between()
                    .child(text(t, s.status(n), ColorRole::FgMuted, 12.5))
                    .child(
                        row()
                            .when(s.done(n), |d| {
                                d.child(self.btn(cx, "ws-undo", "Undo", false))
                            })
                            .when(s.undone, |d| {
                                d.child(self.btn(cx, "ws-redo", "Redo", false))
                            })
                            .child(self.btn(
                                cx,
                                "ws-run",
                                if s.done(n) || s.undone {
                                    "Rewrite"
                                } else {
                                    "Update plan"
                                },
                                true,
                            )),
                    ),
            )
    }

    fn shock(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = self.state.shock.clone();
        let ph = s.phase(n);
        let rings = s.rings(n);
        let col = if ph == SwPhase::Failed {
            hue(t, "red")
        } else {
            hue(t, "green")
        };
        let stage = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let diag = f32::from(bounds.size.width).hypot(f32::from(bounds.size.height));
                let o = point(bounds.origin.x + px(36.), bounds.origin.y + px(36.));
                for (r, a) in &rings {
                    let rad = (*r as f32) * diag;
                    window.paint_quad(quad(
                        Bounds::new(
                            point(o.x - px(rad), o.y - px(rad)),
                            size(px(2. * rad), px(2. * rad)),
                        ),
                        px(rad),
                        gpui::transparent_black(),
                        px(2.),
                        col.opacity(*a as f32),
                        BorderStyle::default(),
                    ));
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let (mark_bg, mark_txt) = match ph {
            SwPhase::Done => (hue(t, "green"), "✓"),
            SwPhase::Failed => (hue(t, "red"), "✕"),
            SwPhase::Running => (c(t, ColorRole::SurfaceSelected), "…"),
            SwPhase::Idle => (c(t, ColorRole::SurfaceSelected), "▶"),
        };
        let act = match ph {
            SwPhase::Running => text(
                t,
                format!("{:.0}%", s.progress(n) * 100.),
                ColorRole::FgMuted,
                12.,
            ),
            SwPhase::Failed => div().child(self.btn(cx, "sw-retry", "Retry", true)),
            SwPhase::Done => div().child(self.btn(cx, "sw-run", "Deploy again", false)),
            SwPhase::Idle => div().child(self.btn(cx, "sw-run", "Deploy", true)),
        };
        let mut steps = row().gap(px(14.));
        for (i, st) in SW_STEPS.iter().enumerate() {
            let role = if ph == SwPhase::Done || (ph != SwPhase::Idle && i < s.step(n)) {
                ColorRole::StatusSuccessFg
            } else if ph == SwPhase::Failed && i == s.step(n) {
                ColorRole::StatusDangerFg
            } else {
                ColorRole::FgSubtle
            };
            steps = steps.child(text(t, *st, role, 12.));
        }
        let mut out = div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(12.))
            .overflow_hidden()
            .child(stage)
            .child(
                row()
                    .gap(px(12.))
                    .child(
                        div()
                            .size(px(40.))
                            .rounded(px(20.))
                            .bg(mark_bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(if matches!(ph, SwPhase::Done | SwPhase::Failed) {
                                gpui::white()
                            } else {
                                c(t, ColorRole::FgMuted)
                            })
                            .child(mark_txt),
                    )
                    .child(title(t, SW_TASK.0, SW_TASK.1).flex_1())
                    .child(act),
            );
        if matches!(ph, SwPhase::Running | SwPhase::Failed) {
            out = out.child(bar(
                t,
                s.progress(n),
                if ph == SwPhase::Failed {
                    hue(t, "red")
                } else {
                    c(t, ColorRole::FgDefault)
                },
            ));
        }
        out.child(steps).child(text(
            t,
            s.status(n),
            match ph {
                SwPhase::Done => ColorRole::StatusSuccessFg,
                SwPhase::Failed => ColorRole::StatusDangerFg,
                _ => ColorRole::FgMuted,
            },
            13.,
        ))
    }

    fn split(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = self.state.split.clone();
        let ph = s.phase(n);
        let failed = ph == SpPhase::Failed;
        let done = ph == SpPhase::Done;
        let w = 128.0 * s.scale(n) as f32;
        let pill_el = self
            .btn(
                cx,
                if failed { "sp-retry" } else { "sp-go" },
                s.text(n),
                !done && !failed,
            )
            .h(px(40.))
            .w(px(w.max(if failed { 200. } else { 128. })))
            .rounded(px(20.))
            .justify_center()
            .text_size(px(14.))
            .when(done, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
            .when(failed, |d| {
                d.text_color(hue(t, "red")).border_color(hue(t, "red"))
            });
        let mut line = row().gap(px(0.)).child(pill_el);
        if let Some((off, sz)) = s.bubble(n) {
            let d = 40. * (0.5 + 0.5 * sz as f32);
            line = line.child(
                self.btn(cx, "sp-undo", if failed { "✕" } else { "✓" }, false)
                    .ml(px(-20. + off as f32 * 26.))
                    .size(px(d))
                    .px(px(0.))
                    .rounded(px(d / 2.))
                    .justify_center()
                    .bg(if failed {
                        hue(t, "red")
                    } else {
                        hue(t, "green")
                    })
                    .border_0()
                    .text_color(gpui::white()),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(text(
                t,
                [
                    "Refund $84.00 to Ada Lin",
                    "Reply to the vendor thread",
                    "PR #214 into main",
                ][s.label],
                ColorRole::FgSubtle,
                12.,
            ))
            .child(line)
            .child(text(
                t,
                if s.can_undo(n) {
                    "Tap the check to undo".to_string()
                } else {
                    format!("{} → {}", SP_LABELS[s.label].0, SP_LABELS[s.label].1)
                },
                ColorRole::FgSubtle,
                12.,
            ))
    }

    fn publish(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let p = self.state.publish.clone();
        let ph = p.phase(n);
        let live = p.live(n);
        let bar_row = row()
            .justify_between()
            .pb(px(10.))
            .border_b_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .child(title(t, DOC.0, DOC.1))
            .child(self.btn(
                cx,
                "pp-toggle",
                if live { "● Live" } else { "Publish" },
                !live,
            ));
        let mut out = div().flex().flex_col().gap(px(12.)).child(bar_row);
        if p.open {
            let mut pop = div()
                .flex()
                .flex_col()
                .gap(px(9.))
                .p(px(14.))
                .w(px(340.))
                .rounded(px(12.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .bg(c(t, ColorRole::BgBase))
                .child(
                    row()
                        .justify_between()
                        .child(text(
                            t,
                            if live {
                                "Published"
                            } else {
                                "Publish to the web"
                            },
                            ColorRole::FgDefault,
                            13.5,
                        ))
                        .child(self.btn(cx, "pp-close", "×", false)),
                );
            if live {
                pop = pop
                    .child(
                        row()
                            .p(px(6.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .child(mark(HOSTS[p.host].1, 14., c(t, ColorRole::FgDefault)))
                            .child(text(t, p.url(), ColorRole::FgDefault, 12.).flex_1())
                            .child(self.btn(cx, "pp-copy", "Copy", false)),
                    )
                    .child(text(
                        t,
                        format!("{} views · {}", p.views(n), ACCESS[p.access]),
                        ColorRole::FgMuted,
                        12.,
                    ))
                    .child(
                        self.btn(cx, "pp-unpublish", "Unpublish", false)
                            .text_color(hue(t, "red")),
                    );
            } else {
                let mut access = row().flex_wrap();
                for (i, a) in ACCESS.iter().enumerate() {
                    access =
                        access.child(self.chip(cx, &format!("pp-access-{i}"), *a, p.access == i));
                }
                let mut hosts = row().flex_wrap();
                for (i, (h, _)) in HOSTS.iter().enumerate() {
                    hosts = hosts.child(self.chip(cx, &format!("pp-host-{i}"), *h, p.host == i));
                }
                pop = pop
                    .child(text(t, "Address", ColorRole::FgMuted, 11.5))
                    .child(
                        div()
                            .p(px(7.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(if ph == PpPhase::Taken {
                                hue(t, "red")
                            } else {
                                c(t, ColorRole::BorderDefault)
                            })
                            .child(text(t, p.url(), ColorRole::FgDefault, 12.)),
                    )
                    .when(ph == PpPhase::Taken, |d| {
                        d.child(text(
                            t,
                            format!("That address is taken. Publish as {}-2?", p.slug),
                            ColorRole::StatusDangerFg,
                            12.,
                        ))
                    })
                    .child(text(t, "Who can open it", ColorRole::FgMuted, 11.5))
                    .child(access)
                    .child(text(t, "Host", ColorRole::FgMuted, 11.5))
                    .child(hosts)
                    .child(
                        row()
                            .child(self.chip(cx, "pp-index", "Search engines", p.index))
                            .child(self.chip(cx, "pp-history", "Agent edits", p.history)),
                    );
                pop = match ph {
                    PpPhase::Publishing(i) => pop
                        .child(bar(t, p.progress(n), c(t, ColorRole::FgDefault)))
                        .child(text(
                            t,
                            format!("{}…", PP_STEPS[i].0),
                            ColorRole::FgMuted,
                            12.,
                        )),
                    _ => pop.child(self.btn(cx, "pp-go", "Publish", true)),
                };
            }
            out = out.child(row().justify_end().child(pop));
        }
        out.child(text(
            t,
            "Goals: ship the beta to 40 teams and keep weekly churn under 2%.",
            ColorRole::FgMuted,
            12.5,
        ))
    }

    fn copy(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = self.state.copy.clone();
        let (name, mk, _) = AGENTS[s.agent];
        let label = if s.copied(n) {
            if s.fail {
                "Copy blocked".to_string()
            } else {
                format!("✓ Copied for {name}")
            }
        } else {
            "Give to your agent".to_string()
        };
        let mut out = div().flex().flex_col().gap(px(12.)).child(
            row()
                .justify_between()
                .child(title(
                    t,
                    "Test ticks",
                    "Hand this component to your coding agent",
                ))
                .child(
                    row()
                        .gap(px(0.))
                        .child(mark(mk, 14., c(t, ColorRole::FgDefault)).mr(px(6.)))
                        .child(self.btn(cx, "ga-copy", label, !s.copied(n)))
                        .child(self.btn(cx, "ga-menu", "⌄", false)),
                ),
        );
        if let Some(m) = s.menu {
            let mut menu = div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .p(px(4.))
                .rounded(px(10.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault));
            for (i, (a, amk, _)) in AGENTS.iter().enumerate() {
                menu = menu.child(
                    row()
                        .child(mark(amk, 14., c(t, ColorRole::FgDefault)))
                        .child(self.chip(cx, &format!("ga-agent-{i}"), *a, s.agent == i || m == i)),
                );
            }
            for (i, f) in FORMATS.iter().enumerate() {
                menu = menu.child(self.chip(
                    cx,
                    &format!("ga-format-{i}"),
                    *f,
                    s.format == i || m == AGENTS.len() + i,
                ));
            }
            out = out.child(menu);
        }
        out.child(text(
            t,
            format!("{name} · {}", FORMATS[s.format]),
            ColorRole::FgSubtle,
            12.,
        ))
        .child(
            div()
                .p(px(10.))
                .rounded(px(9.))
                .border_1()
                .border_color(c(t, ColorRole::BorderSubtle))
                .bg(c(t, ColorRole::SurfaceRaised))
                .text_size(px(11.5))
                .text_color(c(t, ColorRole::FgDefault))
                .child(s.text()),
        )
    }

    fn loops(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let l = self.state.loops.clone();
        let at = l.at(n);
        let passed = l.passed(n);
        let out_ = l.out(n);
        let scores = l.scored(n);
        let round = at.map(|a| a.0 + 1).unwrap_or(scores.len());
        let mut stations = row().gap(px(8.));
        for (k, s) in LP_STATIONS.iter().enumerate() {
            let on = at.is_some_and(|a| a.1 == k);
            stations = stations.child(pill(t, *s, if on { "blue" } else { "neutral" }));
            if k < 2 {
                stations = stations.child(text(t, "→", ColorRole::FgSubtle, 12.));
            }
        }
        stations = stations.child(text(t, "↺", ColorRole::FgSubtle, 14.));
        let mut hist = row().flex_wrap();
        for (i, s) in scores.iter().enumerate() {
            hist = hist.child(self.chip(
                cx,
                &format!("lp-round-{i}"),
                format!("#{} {s}%", i + 1),
                l.sel == Some(i),
            ));
        }
        let acts = if !passed && !out_ {
            row()
                .child(self.btn(
                    cx,
                    "lp-pause",
                    if l.paused.is_some() {
                        "Resume"
                    } else {
                        "Pause"
                    },
                    false,
                ))
                .child(self.btn(cx, "lp-stop", "Stop", false))
        } else {
            row()
                .child(self.btn(cx, "lp-restart", "Start over", false))
                .when(out_ && l.stopped.is_none(), |d| {
                    d.child(self.btn(cx, "lp-more", "One more round", false))
                })
        };
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        LP_TASK.0,
                        format!(
                            "Loops until {} ≥ {}% · up to {} rounds",
                            LP_TASK.1, l.target, l.max
                        ),
                    ))
                    .child(acts),
            )
            .child(
                row()
                    .gap(px(20.))
                    .child(self.stat(
                        round.to_string(),
                        if passed { "rounds · done" } else { "round" },
                        "neutral",
                    ))
                    .child(self.stat(
                        format!("{}%", l.best(n)),
                        "best",
                        if passed {
                            "green"
                        } else if out_ {
                            "amber"
                        } else {
                            "neutral"
                        },
                    ))
                    .child(stations),
            )
            .child(bar(
                t,
                f64::from(l.best(n)) / 100.0,
                if passed {
                    hue(t, "green")
                } else {
                    c(t, ColorRole::FgDefault)
                },
            ))
            .child(hist)
            .child(text(t, l.status(n), ColorRole::FgMuted, 12.5))
    }
}

impl Render for XPostView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let body = match self.variant {
            XPostVariant::TestTicks => self.ticks(cx),
            XPostVariant::TriageMetro => self.metro(cx),
            XPostVariant::SecretCard => self.secret(cx),
            XPostVariant::ProactiveCard => self.proactive(cx),
            XPostVariant::WriteSweep => self.sweep(cx),
            XPostVariant::DoneShockwave => self.shock(cx),
            XPostVariant::SplitConfirm => self.split(cx),
            XPostVariant::PublishPopover => self.publish(cx),
            XPostVariant::PromptCopy => self.copy(cx),
            XPostVariant::LoopPipeline => self.loops(cx),
        };
        let said = self.state.said.clone();
        div()
            .id("oxyn-xpost")
            .key_context("OxynXPost")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.key(ev, cx)))
            .p(px(16.))
            .size_full()
            .bg(c(t, ColorRole::BgBase))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .child(
                div()
                    .p(px(16.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .bg(c(t, ColorRole::SurfaceBase))
                    .w(relative(1.))
                    .child(body),
            )
            .child(text(t, said, ColorRole::FgSubtle, 11.5).mt(px(8.)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c2a_presses_and_keys_route_to_each_model() {
        let mut s = XPostState {
            now: 60_000,
            ..Default::default()
        };
        s.press(XPostVariant::TestTicks, "tt-fix");
        assert!(s.ticks.fixed.is_some());
        s.press(XPostVariant::TriageMetro, "tm-dot-2");
        assert_eq!(s.metro.open, Some(2));
        s.key(XPostVariant::TriageMetro, "escape", false);
        assert_eq!(s.metro.open, None);
        s.press(XPostVariant::SecretCard, "sc-revoke");
        s.key(XPostVariant::SecretCard, "enter", false);
        assert!(s.secret.is_revoked());
        s.press(XPostVariant::ProactiveCard, "pc-later");
        s.key(XPostVariant::ProactiveCard, "down", false);
        s.key(XPostVariant::ProactiveCard, "enter", false);
        assert_eq!(s.proactive.snoozed, Some(1));
        s.key(XPostVariant::WriteSweep, "z", true);
        assert!(s.sweep.undone);
        s.key(XPostVariant::SplitConfirm, "space", false);
        assert!(s.split.pressed.is_some());
        s.press(XPostVariant::PublishPopover, "pp-access-1");
        s.key(XPostVariant::PublishPopover, "right", false);
        assert_eq!(s.publish.access, 2);
        s.key(XPostVariant::PromptCopy, "down", false);
        assert!(s.copy.menu.is_some());
        s.key(XPostVariant::LoopPipeline, "left", false);
        assert!(s.loops.sel.is_some());
        assert_eq!(XPostVariant::ALL.len(), 10);
        assert_eq!(XPostVariant::PromptCopy.slug(), "agent-prompt-copy");
    }
}
