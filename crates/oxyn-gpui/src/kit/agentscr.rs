//! Screens S3 in GPUI: Thread, Inbox, Inbox Board, Tasks, Activity and Memory on the shared frame
//! with the Fieldwork persona's sidebar (`oxyn_core::kit::workscr::persona("fieldwork")`). The
//! elements draw one state at a time: `.state()` for Data / Loading / Empty and `.extra()` for the
//! screen's own state (`decision`, `scrolled`, `offline`, `uploading`, `dictating`, `away`,
//! `error`, `answered`, `failed`, `finished`, `undone`, `review`) or an Oxyn option (`tools`,
//! `peek`, `compact`, `cost`, `clock`, `scope`). Presses fire `nav-<key>`, `pick-<n>`, `approve`,
//! `retry`, `undo`, `forget` and similar ids through `on_act`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::kit::agentscr::{
    ACT_TABS, AWAY, LANES, Line, Mode, ScreenState, Task, TaskState, act_counts, act_view,
    activity, age, board, board_counts, eur, memo_counts, memo_groups, memory, mmss, project,
    questions, task_counts, task_groups, tasks, thread, thread_counts, tools_words, undo_words,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_core::kit::workscr::persona;
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, skel};
use super::workscr::{btn, find, none, page, quiet, side, tabs, tone};
use crate::brand::brand_mark;

macro_rules! agent_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state or an Oxyn option (see the module docs).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`nav-<key>`, `pick-<n>`, `approve`, `undo`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}

fn st_tone(p: &ThPal, s: &str) -> Hsla {
    tone(
        p,
        match s {
            "warn" => IconTone::Approval,
            "ok" => IconTone::Done,
            "bad" => IconTone::Failed,
            _ => IconTone::Running,
        },
    )
}

fn hue(p: &ThPal, h: &str) -> Hsla {
    tone(
        p,
        match h {
            "purple" => IconTone::Skills,
            "green" => IconTone::Sheet,
            "orange" => IconTone::Slides,
            "teal" => IconTone::Audio,
            "pink" => IconTone::Image,
            _ => IconTone::Doc,
        },
    )
}

fn dot(c: Hsla) -> Div {
    div().w(px(7.)).h(px(7.)).flex_none().rounded(px(4.)).bg(c)
}

fn ring(p: &ThPal, t: &Task) -> Div {
    let c = st_tone(p, t.state.key());
    div()
        .w(px(15.))
        .h(px(15.))
        .flex_none()
        .rounded(px(8.))
        .border_2()
        .border_color(c)
        .when(t.done * 2 >= t.of, |d| d.bg(c.opacity(0.35)))
}

fn pill(p: &ThPal, l: &'static str, c: Hsla) -> Div {
    let _ = p;
    div()
        .flex_none()
        .h(px(18.))
        .px(px(6.))
        .rounded(px(5.))
        .bg(c.opacity(0.22))
        .text_size(px(11.5))
        .whitespace_nowrap()
        .child(l)
}

fn h3(p: &ThPal, t: &'static str) -> Div {
    div()
        .mt(px(22.))
        .mb(px(8.))
        .text_size(px(12.5))
        .text_color(p.mute)
        .child(t)
}

fn kv(p: &ThPal, k: &'static str, v: Div) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .h(px(26.))
        .child(div().w(px(76.)).flex_none().text_color(p.mute).child(k))
        .child(v.flex_1().min_w_0().flex().items_center().gap(px(6.)))
}

fn wtab(p: &ThPal, label: impl Into<SharedString>) -> Div {
    div()
        .h(px(48.))
        .flex_none()
        .px(px(6.))
        .flex()
        .items_center()
        .border_b_1()
        .border_color(p.line)
        .child(
            div()
                .max_w(px(200.))
                .h(px(30.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .child(clip(label))
                .child(quiet(p, "×")),
        )
}

fn work(p: &ThPal, label: impl Into<SharedString>, body: Div) -> Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(wtab(p, label))
        .child(div().flex_1().min_h_0().px(px(18.)).py(px(18.)).child(body))
}

fn files(p: &ThPal, fs: &[&'static str]) -> Div {
    let mut r = div().flex().flex_wrap().gap(px(6.));
    for f in fs {
        let c = tone(p, IconTone::for_file(f));
        r = r.child(
            div()
                .h(px(28.))
                .px(px(9.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(7.))
                .bg(p.sel)
                .child(div().w(px(9.)).h(px(11.)).rounded(px(2.)).bg(c))
                .child(*f),
        );
    }
    r
}

fn steps(p: &ThPal, t: &Task) -> Div {
    let mut s = div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .p(px(6.))
        .rounded(px(10.))
        .bg(p.card2);
    for (i, (l, _)) in t.steps.iter().enumerate() {
        let i = i as u8;
        let done = i < t.done;
        let now = i == t.done;
        s = s.child(
            div()
                .h(px(28.))
                .px(px(6.))
                .flex()
                .items_center()
                .gap(px(10.))
                .text_color(if now { p.fg } else { p.mute })
                .child(
                    div()
                        .w(px(14.))
                        .h(px(14.))
                        .rounded(px(7.))
                        .border_1()
                        .border_color(if done {
                            tone(p, IconTone::Running)
                        } else if now {
                            tone(p, IconTone::Slides)
                        } else {
                            p.line
                        })
                        .when(done, |d| d.bg(tone(p, IconTone::Running))),
                )
                .child(clip(*l))
                .when(now && t.state == TaskState::NeedsYou, |d| {
                    d.child(pill(p, "Needs you", tone(p, IconTone::Approval)))
                }),
        );
    }
    s
}

fn task_ctx(p: &ThPal, t: &Task) -> Div {
    div()
        .flex()
        .flex_col()
        .child(div().text_color(p.fg2).child(t.brief))
        .child(
            div()
                .mt(px(14.))
                .child(kv(
                    p,
                    "Status",
                    div()
                        .child(ring(p, t))
                        .child(clip(t.now))
                        .child(quiet(p, format!("{}/{}", t.done, t.of))),
                ))
                .child(kv(p, "Project", div().child(project(t.project).1)))
                .child(kv(p, "Started", div().child("Mon, 3 Nov"))),
        )
        .child(h3(p, "Steps"))
        .child(steps(p, t))
        .child(h3(p, "Artifacts"))
        .child(files(p, t.files))
}

fn sk(p: &ThPal, st: ScreenState, n: usize) -> Option<Div> {
    (st == ScreenState::Loading).then(|| skel(p, n))
}

// ───────────────────────────────────────────────────────────── Thread ──

agent_el!(
    /// Agent / Thread: the conversation with the orchestrator, tools folded into one line, task
    /// cards, file chips and the composer; `.extra()` draws one of the nine states.
    AgentThreadEl
);

impl RenderOnce for AgentThreadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let sample = st == ScreenState::Sample;
        let ts = tasks(sample);
        let x = self.extra;
        let mut col = div().w(px(680.)).flex().flex_col().gap(px(14.)).py(px(8.));
        if let Some(s) = sk(&p, st, 8) {
            col = col.child(s);
        } else if !sample {
            col = col.child(none(
                &p,
                "Ask for an outcome",
                "The orchestrator plans the tasks and checks in here when it needs you.",
            ));
        } else {
            if x == "away" {
                let mut a = div()
                    .p(px(12.))
                    .rounded(px(12.))
                    .bg(p.card2)
                    .border_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("While you were away · 3h"),
                    );
                for l in AWAY {
                    a = a.child(div().text_color(p.fg2).child(format!("• {l}")));
                }
                col = col.child(a);
            }
            for l in thread(true) {
                col = col.child(match l {
                    Line::Me(t) => div().flex().justify_end().child(
                        div()
                            .max_w(px(560.))
                            .px(px(14.))
                            .py(px(9.))
                            .rounded(px(16.))
                            .bg(p.sel)
                            .child(t),
                    ),
                    Line::Agent(t, at) if t.is_empty() => div()
                        .flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .w(px(10.))
                                .h(px(10.))
                                .rounded(px(2.))
                                .bg(hue(&p, "purple")),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Orchestrator"),
                        )
                        .child(quiet(&p, at)),
                    Line::Agent(t, _) => div().child(t),
                    Line::Tools(n, ds) => div()
                        .text_color(p.mute)
                        .child(format!("✓ {}", tools_words(n, ds))),
                    Line::Task(id) => {
                        let t = ts
                            .iter()
                            .find(|t| t.id == id)
                            .cloned()
                            .unwrap_or_else(|| ts[0].clone());
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .px(px(12.))
                            .py(px(10.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(p.line)
                            .child(ring(&p, &t))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(clip(t.name))
                                    .child(quiet(&p, t.now)),
                            )
                            .child(quiet(&p, format!("{}/{}", t.done, t.of)))
                            .child(div().font_weight(FontWeight::MEDIUM).child("Open"))
                    }
                    Line::Files(fs) => files(&p, fs),
                });
            }
            if x == "error" {
                col = col.child(
                    div()
                        .p(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(tone(&p, IconTone::Failed))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(div().flex_1().child("The reply didn't come through"))
                        .child(btn(&p, "Retry", false).id("retry").on_click({
                            let f = fire(&self.act, "retry");
                            move |_, w, cx| f(w, cx)
                        })),
                );
            }
            if x == "decision" {
                let mut d = div()
                    .p(px(14.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(pill(&p, "Needs you", tone(&p, IconTone::Approval)))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Which queue should the event service use?"),
                    );
                for (i, l) in ["Kafka", "SQS", "Postgres queue"].into_iter().enumerate() {
                    let id: &'static str = ["pick-1", "pick-2", "pick-3"][i];
                    let f = fire(&self.act, id);
                    d = d.child(
                        div()
                            .id(id)
                            .h(px(36.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .rounded(px(9.))
                            .border_1()
                            .border_color(p.line)
                            .child(quiet(&p, (i + 1).to_string()))
                            .child(l)
                            .on_click(move |_, w, cx| f(w, cx)),
                    );
                }
                col = col.child(d);
            }
        }
        let mut cmp = div()
            .w(px(680.))
            .mb(px(14.))
            .p(px(12.))
            .rounded(px(14.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .flex()
            .flex_col()
            .gap(px(8.));
        if x == "offline" {
            cmp = cmp.child(
                div()
                    .text_color(st_tone(&p, "warn"))
                    .child("You're offline. Messages send when you're back."),
            );
        }
        if x == "uploading" {
            cmp = cmp
                .child(files(&p, &["venue-floorplan.pdf"]))
                .child(quiet(&p, "62% · 4.2 MB"));
        }
        cmp = cmp
            .child(if x == "dictating" {
                div()
                    .h(px(44.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(dot(tone(&p, IconTone::Failed)))
                    .child(quiet(&p, "Listening… 0:07"))
            } else {
                div()
                    .h(px(44.))
                    .text_color(p.mute)
                    .child("Ask for anything")
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(quiet(&p, "+"))
                    .child(quiet(&p, "Request approval"))
                    .child(div().flex_1())
                    .child(div().w(px(28.)).h(px(28.)).rounded(px(8.)).bg(p.fg)),
            );
        let (r, n) = thread_counts(&ts);
        let mut main = div().flex_1().min_w_0().flex().flex_col().items_center();
        let mut top = div()
            .w_full()
            .h(px(48.))
            .px(px(14.))
            .flex()
            .items_center()
            .justify_end()
            .gap(px(8.));
        if sample {
            top = top
                .child(dot(st_tone(&p, "run")))
                .child(format!("{r} running"))
                .child(dot(st_tone(&p, "warn")))
                .child(format!("{n} need you"));
        }
        if x == "scrolled" {
            main = main.child(top).child(col).child(
                div()
                    .h(px(30.))
                    .px(px(12.))
                    .rounded(px(15.))
                    .border_1()
                    .border_color(p.line)
                    .flex()
                    .items_center()
                    .child("2 new · Jump to latest"),
            );
        } else {
            main = main.child(top).child(div().flex_1().min_h_0().child(col));
        }
        main = main.child(cmp);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "thread", st, &self.act),
            main,
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── Inbox ──

agent_el!(
    /// Agent / Inbox: the question list (320), one question with its answers (466) and the task
    /// behind it in the work area; `answered` shows the next question after an answer.
    InboxScreenEl
);

impl RenderOnce for InboxScreenEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let sample = st == ScreenState::Sample;
        let mut qs = questions(sample);
        if self.extra == "answered" && !qs.is_empty() {
            qs.remove(0);
        }
        let ts = tasks(sample);
        let mut list = div()
            .w(px(320.))
            .flex_none()
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .border_r_1()
            .border_color(p.line);
        if let Some(s) = sk(&p, st, 5) {
            list = list.child(s);
        }
        for (i, q) in qs.iter().enumerate() {
            list = list.child(
                div()
                    .px(px(10.))
                    .py(px(9.))
                    .rounded(px(10.))
                    .flex()
                    .gap(px(10.))
                    .when(i == 0, |d| d.bg(p.sel))
                    .child(
                        div()
                            .w(px(14.))
                            .h(px(14.))
                            .mt(px(2.))
                            .rounded(px(4.))
                            .border_2()
                            .border_color(hue(&p, project(q.project).2)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(clip(q.title))
                            .child(quiet(&p, format!("{} · {}", q.from, age(q.mins)))),
                    ),
            );
        }
        let mut det = div().flex_1().min_w_0().flex().justify_center();
        let mut wk = None;
        if let Some(q) = qs.first() {
            let mut opts = div()
                .p(px(4.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col();
            for (i, c) in q.choices.iter().enumerate() {
                let id: &'static str = ["pick-1", "pick-2", "pick-3"][i.min(2)];
                let f = fire(&self.act, id);
                opts = opts.child(
                    div()
                        .id(id)
                        .p(px(10.))
                        .rounded(px(9.))
                        .flex()
                        .gap(px(12.))
                        .when(i == 0, |d| d.bg(p.sel))
                        .child(quiet(&p, (i + 1).to_string()))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(div().font_weight(FontWeight::MEDIUM).child(c.label))
                                .child(quiet(&p, c.sub)),
                        )
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            det = det.child(
                div()
                    .w(px(388.))
                    .pt(px(56.))
                    .flex()
                    .flex_col()
                    .child(quiet(&p, format!("{} · {} ago", q.from, age(q.mins))))
                    .child(
                        div()
                            .mt(px(10.))
                            .text_size(px(23.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(q.title),
                    )
                    .child(
                        div()
                            .mt(px(10.))
                            .mb(px(22.))
                            .text_color(p.fg2)
                            .child(q.lead),
                    )
                    .child(opts)
                    .child(
                        div()
                            .mt(px(16.))
                            .flex()
                            .gap(px(6.))
                            .child(find(&p, "Or tell it something else", 240.))
                            .child(btn(&p, "Later", false))
                            .child(btn(&p, "Send", true)),
                    )
                    .when(self.extra == "peek", |d| {
                        d.child(quiet(&p, "Next: the following question"))
                    }),
            );
            if let Some(t) = ts.iter().find(|t| t.id == q.task) {
                wk = Some(work(&p, t.name, task_ctx(&p, t)));
            }
        } else if st != ScreenState::Loading {
            det = det.child(none(
                &p,
                "Nothing needs you yet",
                "When an agent needs a decision, it waits here.",
            ));
        }
        let ti = [("open", "Open".to_string()), ("done", "Done".to_string())];
        let body = div().flex_1().min_h_0().flex().child(list).child(det);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "inbox", st, &self.act),
            page(
                &p,
                Some(("Inbox", qs.len().to_string())),
                vec![tabs(&p, &ti, "open")],
                body,
            ),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Board ──

agent_el!(
    /// Agent / Inbox Board: Needs you · Running · Done lanes of task cards; `failed` marks a run
    /// failed with Retry in the work area.
    InboxBoardEl
);

impl RenderOnce for InboxBoardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let failed = self.extra == "failed";
        let all = board(st == ScreenState::Sample, failed);
        let c = board_counts(&all);
        let mut lanes = div().flex_1().min_h_0().p(px(10.)).flex().gap(px(10.));
        if let Some(s) = sk(&p, st, 6) {
            lanes = lanes.child(s);
        } else if all.is_empty() {
            lanes = lanes.child(none(
                &p,
                "No tasks on the board",
                "Start a task and it moves across the lanes here.",
            ));
        } else {
            for (k, l) in LANES {
                let items: Vec<_> = all.iter().filter(|x| x.lane == k).collect();
                let mut lane = div()
                    .flex_1()
                    .min_w_0()
                    .px(px(4.))
                    .pb(px(6.))
                    .rounded(px(12.))
                    .bg(p.card2)
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .h(px(36.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .child(dot(st_tone(
                                &p,
                                match k {
                                    "needs" => "warn",
                                    "running" => "run",
                                    _ => "ok",
                                },
                            )))
                            .child(
                                div()
                                    .flex_1()
                                    .text_center()
                                    .text_size(px(11.5))
                                    .text_color(p.mute)
                                    .child(l.to_uppercase()),
                            )
                            .child(quiet(&p, items.len().to_string())),
                    );
                for x in items {
                    let tn = if x.failed {
                        "bad"
                    } else {
                        match x.lane {
                            "needs" => "warn",
                            "running" => "run",
                            _ => "ok",
                        }
                    };
                    lane = lane.child(
                        div()
                            .p(px(12.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(p.line)
                            .bg(p.card)
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .w(px(9.))
                                            .h(px(9.))
                                            .rounded(px(2.))
                                            .bg(hue(&p, x.agent.1)),
                                    )
                                    .child(clip(x.title)),
                            )
                            .when(self.extra != "compact", |d| {
                                d.child(div().text_size(px(12.5)).text_color(p.mute).child(x.blurb))
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(7.))
                                    .child(dot(st_tone(&p, tn)))
                                    .child(if x.failed {
                                        "Failed".to_string()
                                    } else {
                                        mmss(x.secs)
                                    })
                                    .child(div().flex_1())
                                    .child(quiet(
                                        &p,
                                        format!("{} read · {} calls", x.reads, x.calls),
                                    )),
                            ),
                    );
                }
                lanes = lanes.child(lane);
            }
        }
        let wk = all
            .iter()
            .find(|x| x.id == if failed { 2 } else { 1 })
            .map(|x| {
                let (a, b) = if x.failed {
                    ("Retry", "Open log")
                } else {
                    ("Approve", "Reject")
                };
                let fa = fire(&self.act, if x.failed { "retry" } else { "approve" });
                work(
                    &p,
                    x.title,
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(16.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(x.title),
                        )
                        .child(div().mt(px(6.)).text_color(p.fg2).child(x.blurb))
                        .child(
                            div()
                                .mt(px(14.))
                                .flex()
                                .gap(px(6.))
                                .child(
                                    btn(&p, a, true)
                                        .id("primary")
                                        .on_click(move |_, w, cx| fa(w, cx)),
                                )
                                .child(btn(&p, b, false)),
                        )
                        .child(
                            div()
                                .mt(px(14.))
                                .child(kv(&p, "Agent", div().child(x.agent.0)))
                                .child(kv(&p, "Sources", div().child(format!("{} read", x.reads))))
                                .child(kv(&p, "Tools", div().child(format!("{} calls", x.calls)))),
                        ),
                )
            });
        let ti = [
            ("all", format!("All tasks {}", c[0])),
            ("needs", format!("Needs you {}", c[1])),
            ("running", format!("Running {}", c[2])),
            ("done", format!("Done {}", c[3])),
        ];
        let body = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(48.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(tabs(&p, &ti, "all"))
                    .child(div().flex_1())
                    .child(btn(&p, "Agents", false))
                    .child(btn(&p, "New task", true)),
            )
            .child(lanes);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "inbox", st, &self.act),
            page(&p, None, vec![], body),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Tasks ──

agent_el!(
    /// Agent / Tasks: rows grouped by project with rings and Needs you tags; `finished` shows the
    /// Done tab.
    TasksEl
);

impl RenderOnce for TasksEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = tasks(st == ScreenState::Sample);
        let done_tab = self.extra == "finished";
        let (na, nd) = task_counts(&all);
        let mut body = div().flex_1().min_h_0().p(px(8.)).flex().flex_col();
        if let Some(s) = sk(&p, st, 9) {
            body = body.child(s);
        } else if all.is_empty() {
            body = body.child(none(
                &p,
                "No tasks yet",
                "Ask for an outcome in the thread, or start one with New task.",
            ));
        }
        for ((_, pn, h), items) in task_groups(&all, done_tab) {
            body = body.child(
                div()
                    .h(px(34.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().w(px(12.)).h(px(10.)).rounded(px(2.)).bg(hue(&p, h)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(pn))
                    .child(quiet(&p, items.len().to_string())),
            );
            for t in items {
                let id: &'static str = if t.id == 1 { "task-1" } else { "task" };
                let f = fire(&self.act, id);
                body = body.child(
                    div()
                        .id(SharedString::from(format!("task-{}", t.id)))
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .rounded(px(9.))
                        .when(t.id == 1 || t.id == 10, |d| d.bg(p.sel))
                        .child(ring(&p, &t))
                        .child(
                            div()
                                .flex_none()
                                .font_weight(FontWeight::MEDIUM)
                                .child(t.name),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_color(p.mute)
                                .child(clip(t.now)),
                        )
                        .when(t.state == TaskState::NeedsYou, |d| {
                            d.child(pill(&p, "Needs you", tone(&p, IconTone::Approval)))
                        })
                        .when(self.extra == "cost", |d| {
                            d.child(quiet(&p, eur(18 + t.id * 7)))
                        })
                        .child(quiet(&p, format!("{}/{}", t.done, t.of)))
                        .child(quiet(&p, age(t.mins)))
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
        }
        let sel = if done_tab { 10 } else { 1 };
        let wk = all.iter().find(|t| t.id == sel).map(|t| {
            let mut b = div().flex().flex_col();
            if let Some(a) = t.ask {
                b = b.child(
                    div()
                        .mb(px(12.))
                        .p(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(pill(&p, "Needs you", tone(&p, IconTone::Approval)))
                                .child(a),
                        )
                        .child(btn(&p, "Decide", true)),
                );
            }
            work(&p, t.name, b.child(task_ctx(&p, t)))
        });
        let ti = [
            ("active", "Active".to_string()),
            ("done", format!("Done {nd}")),
        ];
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "tasks", st, &self.act),
            page(
                &p,
                Some(("Tasks", (na + nd).to_string())),
                vec![
                    tabs(&p, &ti, if done_tab { "done" } else { "active" }),
                    btn(&p, "New task", false),
                ],
                body,
            ),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Activity ──

agent_el!(
    /// Agent / Activity: what agents did, by day, with Undo or Final; the work area shows When /
    /// Task / Cost, what happened, why it could and Taking it back. `undone` shows the first row
    /// undone; `clock` shows the countdown as a clock.
    ActivityEl
);

impl RenderOnce for ActivityEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let mut all = activity(st == ScreenState::Sample);
        if self.extra == "undone" && !all.is_empty() {
            all[0].undone = true;
        }
        let n = act_counts(&all);
        let mut body = div().flex_1().min_h_0().p(px(8.)).flex().flex_col();
        if let Some(s) = sk(&p, st, 9) {
            body = body.child(s);
        } else if all.is_empty() {
            body = body.child(none(
                &p,
                "Nothing done yet",
                "Every action an agent takes shows here, with a way to take it back.",
            ));
        }
        for (day, items) in act_view(&all, "all", "") {
            body = body.child(
                div()
                    .h(px(32.))
                    .px(px(12.))
                    .rounded(px(9.))
                    .bg(p.card2)
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::MEDIUM).child(day))
                    .child(quiet(&p, items.len().to_string())),
            );
            for a in items {
                let f = fire(&self.act, "undo");
                let right = if a.undone {
                    quiet(&p, "Undone")
                } else if a.undo_min == 0 {
                    quiet(&p, "Final")
                } else {
                    div().child(
                        btn(&p, "Undo", false)
                            .id(SharedString::from(format!("undo-{}", a.id)))
                            .on_click(move |_, w, cx| f(w, cx)),
                    )
                };
                body = body.child(
                    div()
                        .h(px(54.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .rounded(px(10.))
                        .when(a.id == 1, |d| d.bg(p.sel))
                        .child(quiet(&p, a.at))
                        .child(
                            div()
                                .w(px(28.))
                                .h(px(28.))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(p.line)
                                .bg(gpui::white())
                                .children(brand_mark(a.app, 16., gpui::black())),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    clip(a.text)
                                        .when(a.undone, |d| d.line_through().text_color(p.mute)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(dot(st_tone(
                                            &p,
                                            if a.mode == Mode::Approved {
                                                "warn"
                                            } else {
                                                "run"
                                            },
                                        )))
                                        .child(quiet(
                                            &p,
                                            format!("{} · {}", a.mode.label(), a.task),
                                        )),
                                ),
                        )
                        .child(right),
                );
            }
        }
        let wk = all.first().map(|a| {
            let left = if a.undone {
                0
            } else {
                u64::from(a.undo_min) * 60
            };
            let words = if a.undone {
                "Already undone".to_string()
            } else if self.extra == "clock" {
                oxyn_core::kit::agentscr::undo_clock(left)
            } else {
                undo_words(left)
            };
            let mut what = div()
                .p(px(12.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(6.));
            for (k, v) in a.what {
                what = what.child(kv(&p, k, div().child(*v)));
            }
            what = what.child(div().text_color(p.fg2).child(a.body));
            let f = fire(&self.act, "undo");
            work(
                &p,
                a.text,
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(15.))
                            .child(a.text),
                    )
                    .child(
                        div()
                            .mt(px(10.))
                            .child(kv(&p, "When", div().child(format!("Today {}", a.at))))
                            .child(kv(&p, "Task", div().child(a.task)))
                            .child(kv(&p, "Cost", div().child(eur(a.cents)))),
                    )
                    .child(h3(&p, "What happened"))
                    .child(what)
                    .child(h3(&p, "Why it could"))
                    .child(
                        div()
                            .p(px(12.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(p.line)
                            .child(a.mode.label())
                            .child(quiet(&p, a.rule)),
                    )
                    .child(h3(&p, "Taking it back"))
                    .child(
                        div()
                            .p(px(12.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(p.line)
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(a.undo_does)
                                    .child(quiet(&p, words)),
                            )
                            .child(
                                btn(&p, "Undo", true)
                                    .id("undo")
                                    .on_click(move |_, w, cx| f(w, cx)),
                            ),
                    ),
            )
        });
        let ti: Vec<(&'static str, String)> = ACT_TABS
            .iter()
            .enumerate()
            .map(|(i, (k, l))| (*k, format!("{l} {}", n[i])))
            .collect();
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "activity", st, &self.act),
            page(
                &p,
                Some(("Activity", String::new())),
                vec![tabs(&p, &ti, "all"), find(&p, "Search activity", 200.)],
                body,
            ),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Memory ──

agent_el!(
    /// Agent / Memory: memories grouped by scope with source, uses and age; `review` shows the To
    /// review tab; the work area shows where one came from and where it was used, with Forget.
    MemoryEl
);

impl RenderOnce for MemoryEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = memory(st == ScreenState::Sample);
        let review = self.extra == "review";
        let (ns, nr) = memo_counts(&all);
        let mut body = div()
            .flex_1()
            .min_h_0()
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(2.));
        if let Some(s) = sk(&p, st, 10) {
            body = body.child(s);
        } else if all.is_empty() {
            body = body.child(none(
                &p,
                "Nothing remembered yet",
                "Agents keep what you tell them and decide, and show where each fact was used.",
            ));
        }
        let sel = if review { 12 } else { 3 };
        for ((_, pn, h), items) in memo_groups(&all, review, "") {
            body = body.child(
                div()
                    .mt(px(6.))
                    .h(px(34.))
                    .px(px(12.))
                    .rounded(px(9.))
                    .bg(p.card2)
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().w(px(12.)).h(px(10.)).rounded(px(2.)).bg(hue(&p, h)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(pn))
                    .child(quiet(&p, items.len().to_string())),
            );
            for m in items {
                body = body.child(
                    div()
                        .h(px(40.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .rounded(px(9.))
                        .when(m.id == sel, |d| d.bg(p.sel))
                        .child(clip(m.text))
                        .when(self.extra != "compact", |d| {
                            d.child(quiet(&p, m.source))
                                .child(quiet(&p, format!("{}×", m.uses)))
                        })
                        .child(quiet(&p, age(m.mins))),
                );
            }
        }
        let wk = all.iter().find(|m| m.id == sel).map(|m| {
            let f = fire(&self.act, "forget");
            work(
                &p,
                m.text,
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(m.text),
                    )
                    .child(
                        div()
                            .mt(px(10.))
                            .child(kv(
                                &p,
                                "Applies to",
                                div().child(if m.scope == "you" {
                                    "Everywhere"
                                } else {
                                    project(m.scope).1
                                }),
                            ))
                            .child(kv(&p, "Learned", div().child(format!("From {}", m.source))))
                            .child(kv(&p, "Used", div().child(format!("{} times", m.uses)))),
                    )
                    .child(h3(&p, "Where it was used"))
                    .child(
                        div()
                            .text_color(p.fg2)
                            .child("Weekly note · Client dashboard"),
                    )
                    .child(
                        div()
                            .mt(px(14.))
                            .flex()
                            .gap(px(6.))
                            .when(m.review, |d| d.child(btn(&p, "Keep", true)))
                            .child(
                                btn(&p, "Forget", false)
                                    .id("forget")
                                    .on_click(move |_, w, cx| f(w, cx)),
                            ),
                    ),
            )
        });
        let ti = [
            ("saved", "Saved".to_string()),
            ("review", format!("To review {nr}")),
        ];
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "memory", st, &self.act),
            page(
                &p,
                Some(("Memory", ns.to_string())),
                vec![
                    tabs(&p, &ti, if review { "review" } else { "saved" }),
                    find(&p, "Search memory", 200.),
                    btn(&p, "Remember", false),
                ],
                body,
            ),
            wk,
        )
    }
}
