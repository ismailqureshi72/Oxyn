//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32) for GPUI: the exact
//! **Agent Queue** ([`AgentQueueEl`]) over `oxyn_core::parity::queuefid`. The element is
//! `RenderOnce`: the host keeps the clock, the steering events and the open job, and routes
//! [`Act`] ids back (`pause`, `resume`, `less`, `more`, `job-<n>` for the n-th waiting job,
//! `close`, `promote`, `cancel`). A static snapshot: the chart is drawn as bars, nothing glides.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::queuefid::{
    JobKind, QueueEvent, QueueJob, QueueScene, depth_top, queue_at, queue_secs,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 756.0;
const JOB_ACTS: [&str; 3] = ["job-0", "job-1", "job-2"];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    faint: Hsla,
    sub: Hsla,
    chip: Hsla,
    blue: Hsla,
    blue_bg: Hsla,
    amber: Hsla,
    amber_bg: Hsla,
    violet: Hsla,
    teal: Hsla,
    slate: Hsla,
    green: Hsla,
    green_bg: Hsla,
    red: Hsla,
    red_bg: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x18181B),
            line: h(0x2A2A2E),
            fg: h(0xFAFAFA),
            mute: h(0x8B8B93),
            faint: h(0x5F5F66),
            sub: h(0x1F1F22),
            chip: h(0x26262A),
            blue: h(0x6AAEFF),
            blue_bg: h(0x1E2B3D),
            amber: h(0xF5B454),
            amber_bg: h(0x33281A), // warm-ok: semantic warning tint
            violet: h(0xA78BFA),
            teal: h(0x2DD4BF),
            slate: h(0x94A3B8),
            green: h(0x4ADE80),
            green_bg: h(0x1B3022),
            red: h(0xF87171),
            red_bg: h(0x3A1F20),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xECECEF),
            fg: h(0x18181B),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            sub: h(0xF7F7F8),
            chip: h(0xF1F1F3),
            blue: h(0x2563EB),
            blue_bg: h(0xEAF2FE),
            amber: h(0xB45309),
            amber_bg: h(0xFEF3DC),
            violet: h(0x8B5CF6),
            teal: h(0x0D9488),
            slate: h(0x475569),
            green: h(0x16A34A),
            green_bg: h(0xE7F6EC),
            red: h(0xDC2626),
            red_bg: h(0xFDECEC),
        }
    }
}

fn kind_col(p: Pal, k: JobKind) -> Hsla {
    match k {
        JobKind::Llm => p.violet,
        JobKind::Tool => p.teal,
        JobKind::Agent => p.slate,
        JobKind::Human => p.blue,
    }
}

/// `agent-queue` (affectUI Agent Queue): the host passes the scene (the demo's at some ms with
/// its steering, or its own) and the open job (index in the queue).
#[derive(IntoElement)]
pub struct AgentQueueEl {
    scene: QueueScene,
    title: String,
    name: String,
    open: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentQueueEl {
    fn default() -> Self {
        Self {
            scene: queue_at(5_000, 4, &[]),
            title: "Queue".into(),
            name: "agent-work".into(),
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentQueueEl {
    /// The demo `ms` into a pass with `workers` and the steering `events`.
    pub fn demo(mut self, ms: i64, workers: usize, events: &[(i64, QueueEvent)]) -> Self {
        self.scene = queue_at(ms, workers, events);
        self
    }
    /// Your own scene.
    pub fn scene(mut self, s: QueueScene) -> Self {
        self.scene = s;
        self
    }
    /// Heading and the queue's name.
    pub fn title(mut self, t: &str, name: &str) -> Self {
        self.title = t.into();
        self.name = name.into();
        self
    }
    /// The waiting job (index) whose card is open.
    pub fn open(mut self, i: Option<usize>) -> Self {
        self.open = i;
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Acts.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

fn icon(j: &QueueJob, p: Pal) -> gpui::Div {
    let d = div()
        .size(px(16.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.))
        .text_color(kind_col(p, j.kind));
    match brand_mark(&j.brand, 12.0, p.fg) {
        Some(m) => d.child(m),
        None => d.child(match j.kind {
            JobKind::Llm => "✦",
            JobKind::Tool => "⚒",
            JobKind::Agent => "◉",
            JobKind::Human => "☑",
        }),
    }
}

impl RenderOnce for AgentQueueEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let t = sc.now;
        let nw = sc.workers.len();
        let running = sc.slots.iter().filter(|s| s.is_some()).count();
        let waiting = sc.queued.len();
        let stopped = sc.paused || sc.limited.is_some();
        let (pfg, pbg, ptxt) = if sc.paused {
            (p.mute, p.chip, "Paused".to_string())
        } else if let Some(s) = sc.limited {
            (p.amber, p.amber_bg, format!("Rate limited · {s}s"))
        } else {
            (p.blue, p.blue_bg, "Live".to_string())
        };
        let label = |s: &str| {
            div()
                .text_size(px(10.))
                .text_color(p.mute)
                .child(if self.classic {
                    s.to_string()
                } else {
                    s.to_uppercase()
                })
        };
        let small_btn = |txt: &'static str, id: &'static str| {
            div()
                .id(SharedString::from(format!("oxqu-{id}")))
                .h(px(28.))
                .px(px(10.))
                .rounded(px(7.))
                .border_1()
                .border_color(p.line)
                .text_size(px(12.5))
                .flex()
                .items_center()
                .cursor_pointer()
                .on_click({
                    let f = fire(&self.act, id);
                    move |_, w, cx| f(w, cx)
                })
                .child(txt)
        };
        let head = div()
            .min_h(px(52.))
            .pl(px(16.))
            .pr(px(14.))
            .border_b_1()
            .border_color(p.line)
            .flex()
            .items_center()
            .gap(px(8.))
            .child(div().text_color(p.blue).child("≡"))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(div().text_color(p.mute).child(self.name.clone()))
            .child(
                div()
                    .h(px(20.))
                    .px(px(7.))
                    .rounded(px(6.))
                    .bg(pbg)
                    .text_color(pfg)
                    .text_size(px(11.5))
                    .flex()
                    .items_center()
                    .child(ptxt),
            )
            .child(div().flex_1())
            .child(if sc.paused {
                small_btn("▶ Resume", "resume")
            } else {
                small_btn("⏸ Pause", "pause")
            })
            .child(
                div()
                    .h(px(28.))
                    .rounded(px(7.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .id("oxqu-less")
                            .w(px(28.))
                            .flex()
                            .justify_center()
                            .text_color(p.mute)
                            .cursor_pointer()
                            .on_click({
                                let f = fire(&self.act, "less");
                                move |_, w, cx| f(w, cx)
                            })
                            .child("−"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(nw.to_string()),
                    )
                    .child(
                        div()
                            .id("oxqu-more")
                            .w(px(28.))
                            .flex()
                            .justify_center()
                            .text_color(p.mute)
                            .cursor_pointer()
                            .on_click({
                                let f = fire(&self.act, "more");
                                move |_, w, cx| f(w, cx)
                            })
                            .child("+"),
                    ),
            );
        let stat = |k: &str, v: String, unit: &str, warn: bool| {
            div()
                .flex_1()
                .px(px(16.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(label(k))
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .child(
                            div()
                                .text_size(px(15.))
                                .text_color(if warn { p.amber } else { p.fg })
                                .child(v),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.mute)
                                .child(unit.to_string()),
                        ),
                )
        };
        let stats = div()
            .py(px(12.))
            .border_b_1()
            .border_color(p.line)
            .flex()
            .child(stat(
                "Waiting",
                waiting.to_string(),
                "",
                waiting as u32 > sc.backpressure,
            ))
            .child(stat(
                "Running",
                running.to_string(),
                &format!("/{nw}"),
                false,
            ))
            .child(stat("Throughput", sc.per_min.to_string(), "/min", false))
            .child(stat(
                "Wait p95",
                sc.wait_p95.div_ceil(1000).to_string(),
                "s",
                false,
            ));

        let mut q = div()
            .w(px(196.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(label(&format!("Queue {waiting}")));
        for (i, j) in sc.queued.iter().take(3).enumerate() {
            q = q.child(
                div()
                    .id(SharedString::from(format!("oxqu-job-{i}")))
                    .h(px(30.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if self.open == Some(i) { p.blue } else { p.line })
                    .bg(p.card)
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .text_size(px(12.))
                    .cursor_pointer()
                    .on_click({
                        let f = fire(&self.act, JOB_ACTS[i]);
                        move |_, w, cx| f(w, cx)
                    })
                    .child(icon(j, p))
                    .child(div().flex_1().overflow_hidden().child(j.name.clone()))
                    .child(
                        div()
                            .text_color(p.mute)
                            .child(queue_secs(t - j.enqueued_at)),
                    ),
            );
        }
        if waiting > 3 {
            let mut dots = div()
                .p(px(7.))
                .rounded(px(8.))
                .bg(p.sub)
                .flex()
                .flex_wrap()
                .gap(px(3.));
            for j in sc.queued.iter().skip(3).take(48) {
                dots = dots.child(
                    div()
                        .size(px(9.))
                        .rounded(px(3.))
                        .bg(alpha(kind_col(p, j.kind), 0.45)),
                );
            }
            q = q.child(dots);
        }
        let mut ws = div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(label(&format!("Workers {running}/{nw}")));
        for (wk, slot) in sc.workers.iter().zip(sc.slots.iter()) {
            let mut row = div()
                .h(px(40.))
                .px(px(8.))
                .rounded(px(9.))
                .bg(p.sub)
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .size(px(24.))
                        .rounded(px(7.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .flex()
                        .items_center()
                        .justify_center()
                        .children(brand_mark(&wk.brand, 14.0, p.fg)),
                )
                .child(
                    div()
                        .w(px(108.))
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(wk.name.clone()),
                );
            row = match slot {
                Some(j) => {
                    let col = kind_col(p, j.kind);
                    row.child(
                        div()
                            .relative()
                            .flex_1()
                            .h(px(28.))
                            .rounded(px(8.))
                            .bg(p.card)
                            .overflow_hidden()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.))
                                    .top(px(0.))
                                    .h(px(28.))
                                    .w(gpui::relative(j.progress.clamp(0.02, 1.0)))
                                    .bg(alpha(col, if stopped { 0.08 } else { 0.14 })),
                            )
                            .child(
                                div()
                                    .relative()
                                    .h(px(28.))
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .gap(px(7.))
                                    .text_size(px(12.))
                                    .child(icon(j, p))
                                    .child(div().flex_1().overflow_hidden().child(j.name.clone()))
                                    .child(if stopped {
                                        div().text_color(p.mute).child("Paused")
                                    } else {
                                        div()
                                            .text_color(col)
                                            .child(queue_secs(t - j.started_at.unwrap_or(t)))
                                    }),
                            ),
                    )
                }
                None => row.child(div().pl(px(10.)).text_color(p.faint).child(if sc.paused {
                    "Paused"
                } else {
                    "Idle"
                })),
            };
            ws = ws.child(row);
        }
        let total = |k: &str, v: u32, col: Hsla, bg: Hsla, g: &'static str| {
            div()
                .p(px(10.))
                .rounded(px(10.))
                .bg(p.sub)
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .child(
                            div()
                                .size(px(20.))
                                .rounded(px(6.))
                                .bg(bg)
                                .text_color(col)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(g),
                        )
                        .child(label(k)),
                )
                .child(
                    div()
                        .text_size(px(20.))
                        .text_color(if g == "✕" { col } else { p.fg })
                        .child(v.to_string()),
                )
        };
        let tot = div()
            .w(px(92.))
            .pt(px(18.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(total("Done", sc.done, p.green, p.green_bg, "✓"))
            .child(total("Failed", sc.failed, p.red, p.red_bg, "✕"));
        let mut main = div()
            .relative()
            .px(px(16.))
            .pt(px(14.))
            .pb(px(16.))
            .border_b_1()
            .border_color(p.line)
            .flex()
            .gap(px(24.))
            .child(q)
            .child(ws)
            .child(tot);
        if let Some(i) = self.open
            && let Some(j) = sc.queued.get(i)
        {
            let cell = |k: &str, v: String| {
                div()
                    .w(gpui::relative(0.5))
                    .child(label(k))
                    .child(div().text_size(px(13.)).child(v))
            };
            main = main.child(
                div()
                    .absolute()
                    .left(px(16.))
                    .top(px(56.))
                    .w(px(340.))
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .child(icon(j, p))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(j.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(p.mute)
                                            .child(format!("{} · {}", j.kind.label(), j.detail)),
                                    ),
                            )
                            .child(
                                div()
                                    .id("oxqu-close")
                                    .text_color(p.faint)
                                    .cursor_pointer()
                                    .on_click({
                                        let f = fire(&self.act, "close");
                                        move |_, w, cx| f(w, cx)
                                    })
                                    .child("✕"),
                            ),
                    )
                    .child(
                        div()
                            .p(px(10.))
                            .rounded(px(9.))
                            .bg(p.sub)
                            .flex()
                            .flex_wrap()
                            .gap(px(8.))
                            .child(cell("Waiting", queue_secs(t - j.enqueued_at)))
                            .child(cell("Position", format!("{} of {waiting}", i + 1)))
                            .child(cell(
                                "Priority",
                                if j.high { "High" } else { "Normal" }.into(),
                            ))
                            .child(cell(
                                "Attempt",
                                format!("{} of {}", j.attempt, j.max_attempts),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(6.))
                            .child(small_btn("Cancel", "cancel"))
                            .child(small_btn("Move to front", "promote")),
                    ),
            );
        }
        // Depth chart as bars.
        let top = depth_top(&sc.depth, sc.backpressure) as f32;
        let ch = 112.0;
        let mut plot = div()
            .relative()
            .h(px(ch))
            .ml(px(28.))
            .flex()
            .items_end()
            .gap(px(1.));
        for (a, b) in sc.bands.iter() {
            plot = plot.child(
                div()
                    .absolute()
                    .top(px(0.))
                    .h(px(ch))
                    .left(gpui::relative(*a))
                    .w(gpui::relative(b - a))
                    .bg(alpha(p.amber, 0.08)),
            );
        }
        for d in sc.depth.iter() {
            plot = plot.child(
                div()
                    .flex_1()
                    .h(px((*d as f32 / top * ch).max(1.0)))
                    .bg(alpha(p.violet, 0.35)),
            );
        }
        plot = plot.child(
            div()
                .absolute()
                .left(px(0.))
                .right(px(0.))
                .top(px(ch - sc.backpressure as f32 / top * ch))
                .h(px(1.))
                .bg(p.faint),
        );
        let chart = div()
            .px(px(16.))
            .pt(px(14.))
            .pb(px(12.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(label("Queue depth"))
            .child(plot)
            .child(
                div()
                    .ml(px(28.))
                    .flex()
                    .justify_between()
                    .text_size(px(11.5))
                    .text_color(p.mute)
                    .child("96s ago")
                    .child("now"),
            );
        div()
            .w(px(W))
            .rounded(px(if self.classic { 16. } else { 12. }))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .shadow_sm()
            .text_color(p.fg)
            .text_size(px(12.5))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(head)
            .child(stats)
            .child(main)
            .child(chart)
    }
}
