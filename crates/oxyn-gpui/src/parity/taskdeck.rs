//! Screens S9 in GPUI (Agent apps I): Taskdeck, a desktop agent app. Static snapshots of one
//! state each, like the other screen sets: the sidebar (New task, Tasks, Plugins & skills, Local /
//! Cloud, projects, recents, account row) and one view: Start (the task composer), Task (the step
//! timeline with tool calls and the side panel) or Board (tasks by status). `.state()` sets Data /
//! Loading / Empty and `.extra()` the view's own state (`cloud`, `approval`, `menu`) or the Oxyn
//! options `rail` and `hidden`. Presses fire `new-task`, `tasks`, `send`, `allow`, `deny`,
//! `open-<id>` through `on_act`. Rules come from `oxyn_core::parity::taskdeck`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_core::parity::taskdeck::{
    APP, COLUMNS, EnvMode, MODELS, MODES, PANEL_TABS, PROJECTS, PROMPT, SUGGEST, ScreenState,
    Status, StepState, Task, artifacts, elapsed, fmt_secs, headline, progress, run_for, run_status,
    tasks,
};
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::{clip, file_tile, mark};
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{gh, side_foot, skel, tone_ic};
use super::workscr::{btn, quiet, tone};

macro_rules! s9_el {
    ($(#[$m:meta])* $name:ident, $view:literal) => {
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
            /// The view's own state, `rail` or `hidden` (see the module docs).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat: the main area as a card or edge to edge.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`new-task`, `send`, `allow`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                screen(self.theme, self.layout, self.state, self.extra, &self.act, $view)
            }
        }
    };
}

s9_el!(
    /// Taskdeck / Start: the task composer, folder and Local / Cloud, starter ideas.
    TaskdeckStartEl,
    "start"
);
s9_el!(
    /// Taskdeck / Task: the step timeline with tool calls, an approval and the side panel.
    TaskdeckTaskEl,
    "task"
);
s9_el!(
    /// Taskdeck / Board: tasks by status, Queued, Running, Needs you and Done.
    TaskdeckBoardEl,
    "board"
);

fn st_tone(p: &ThPal, s: Status) -> Hsla {
    match s.tone() {
        "run" => tone(p, IconTone::Running),
        "wait" => tone(p, IconTone::Approval),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        _ => p.mute,
    }
}

fn pill(p: &ThPal, s: Status) -> Div {
    let c = st_tone(p, s);
    div()
        .flex_none()
        .h(px(22.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(11.))
        .bg(p.sel)
        .text_size(px(11.5))
        .text_color(p.fg2)
        .child(div().w(px(6.)).h(px(6.)).rounded(px(3.)).bg(c))
        .child(s.label())
}

fn chip(p: &ThPal, lead: Option<Div>, l: impl Into<SharedString>) -> Div {
    let mut d = div()
        .flex_none()
        .h(px(30.))
        .px(px(9.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(15.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .text_size(px(12.5))
        .text_color(p.fg2);
    if let Some(x) = lead {
        d = d.child(x);
    }
    d.child(l.into()).child("▾")
}

fn press(p: &ThPal, l: &'static str, primary: bool, act: &Option<Act>, id: &'static str) -> Div {
    let f = fire(act, id);
    div().child(
        btn(p, l, primary)
            .id(SharedString::from(id))
            .on_click(move |_, w, cx| f(w, cx)),
    )
}

fn task_row(p: &ThPal, t: &Task, on: bool) -> Div {
    let c = st_tone(p, t.status);
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .h(px(32.))
        .px(px(8.))
        .rounded(px(8.))
        .when(on, |d| d.bg(p.sel))
        .text_size(px(12.5))
        .text_color(p.fg2)
        .child(div().flex_none().w(px(8.)).h(px(8.)).rounded(px(4.)).bg(c))
        .child(clip(t.title.clone()))
        .child(quiet(p, t.ago.clone()))
}

fn sidebar(
    p: &ThPal,
    st: ScreenState,
    env: EnvMode,
    view: &str,
    rail: bool,
    act: &Option<Act>,
) -> Div {
    let w = if rail { 56. } else { 264. };
    let list: Vec<Task> = tasks(st).into_iter().filter(|t| t.env == env).collect();
    let nav = |l: &'static str, on: bool, id: &'static str| {
        let f = fire(act, id);
        div()
            .id(SharedString::from(id))
            .on_click(move |_, w, cx| f(w, cx))
            .h(px(34.))
            .px(px(8.))
            .flex()
            .items_center()
            .rounded(px(9.))
            .when(on, |d| d.bg(p.sel).text_color(p.fg))
            .text_color(p.fg2)
            .child(if rail {
                l.chars().next().unwrap_or(' ').to_string()
            } else {
                l.to_string()
            })
    };
    let mut s = div()
        .w(px(w))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .h(px(34.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(
                    div()
                        .w(px(18.))
                        .h(px(14.))
                        .rounded(px(3.))
                        .bg(p.fg)
                        .flex_none(),
                )
                .when(!rail, |d| d.child(APP)),
        )
        .child(nav("New task", view == "start", "new-task"))
        .child(nav("Tasks", view == "board", "tasks"))
        .child(nav("Plugins & skills", false, "plugins"));
    if !rail {
        let seg = |m: EnvMode| {
            div()
                .flex_1()
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .when(m == env, |d| d.bg(p.card).text_color(p.fg))
                .text_color(p.mute)
                .child(m.label())
        };
        s = s.child(
            div()
                .mt(px(10.))
                .p(px(3.))
                .flex()
                .gap(px(2.))
                .rounded(px(10.))
                .bg(p.sel)
                .child(seg(EnvMode::Local))
                .child(seg(EnvMode::Cloud)),
        );
        let mut body = div().flex_1().min_h_0().overflow_hidden().flex().flex_col();
        if st == ScreenState::Loading {
            body = body.child(skel(p, 6));
        } else if list.is_empty() {
            body = body.child(
                div()
                    .py(px(14.))
                    .flex()
                    .justify_center()
                    .text_color(p.mute)
                    .child("No projects or tasks yet"),
            );
        } else {
            body = body.child(gh(p, "Projects"));
            for (i, (_, n, _)) in PROJECTS.iter().enumerate() {
                let inner: Vec<&Task> = list.iter().filter(|t| t.project == Some(i)).collect();
                body = body.child(
                    div()
                        .h(px(32.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_color(p.fg2)
                        .child(tone_ic(p, IconTone::Folder))
                        .child(clip(*n))
                        .child(quiet(p, inner.len().to_string())),
                );
                for t in inner {
                    body = body.child(div().pl(px(22.)).child(task_row(
                        p,
                        t,
                        t.id == 1 && view == "task",
                    )));
                }
            }
            body = body.child(gh(p, "Recents"));
            for t in list.iter().filter(|t| t.project.is_none()) {
                body = body.child(task_row(p, t, false));
            }
        }
        s = s.child(body);
    } else {
        s = s.child(div().flex_1());
    }
    if rail {
        s.child(
            div()
                .flex()
                .justify_center()
                .pb(px(4.))
                .child(super::setscr::av(p, "RV", 28.)),
        )
    } else {
        s.child(side_foot(p, act))
    }
}

fn start_view(p: &ThPal, env: EnvMode, loading: bool, act: &Option<Act>) -> Div {
    let mut col = div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(16.))
        .p(px(32.));
    if loading {
        return col.child(skel(p, 4));
    }
    let composer = div()
        .w(px(680.))
        .p(px(10.))
        .flex()
        .flex_col()
        .gap(px(6.))
        .rounded(px(16.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(
            div()
                .h(px(68.))
                .px(px(6.))
                .text_color(p.mute)
                .text_size(px(14.5))
                .child("Describe a task · @ for files, plugins and skills"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(chip(p, None, "+"))
                .child(chip(p, None, MODES[0].1))
                .child(div().flex_1())
                .child(chip(
                    p,
                    mark(p, MODELS[0].1, 14.).map(|m| div().child(m)),
                    MODELS[0].0,
                ))
                .child(press(p, "Start", true, act, "send")),
        );
    col = col
        .child(div().w(px(58.)).h(px(58.)).rounded(px(16.)).bg(p.sel))
        .child(
            div()
                .text_size(px(26.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(headline(env)),
        )
        .child(composer)
        .child(
            div()
                .w(px(680.))
                .flex()
                .gap(px(8.))
                .child(chip(p, Some(tone_ic(p, IconTone::Folder)), PROJECTS[0].2))
                .child(quiet(p, env.label())),
        );
    let mut sug = div().flex().gap(px(8.)).justify_center();
    for (_, l, _) in SUGGEST.iter() {
        sug = sug.child(
            div()
                .h(px(34.))
                .px(px(13.))
                .flex()
                .items_center()
                .rounded(px(17.))
                .border_1()
                .border_color(p.line)
                .text_color(p.fg2)
                .child(*l),
        );
    }
    col.child(sug)
}

fn task_view(p: &ThPal, st: ScreenState, approval: bool, act: &Option<Act>) -> Div {
    let list = tasks(st);
    let Some(mut t) = list.into_iter().find(|t| t.id == 1) else {
        return div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.))
            .child(if st == ScreenState::Loading {
                skel(p, 8)
            } else {
                super::screensfid::empty(
                    p,
                    "No task open",
                    "Start one and its steps, tool calls and files show up here.",
                )
            });
    };
    if approval {
        t.status = Status::NeedsYou;
        t.steps = (4, 6);
    }
    let steps = run_for(&t);
    let (d, n) = progress(&steps);
    let status = run_status(&steps);
    let mut tl = div()
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .p(px(20.))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .gap(px(10.))
                .child(super::setscr::av(p, "RV", 28.))
                .child(
                    div()
                        .max_w(px(600.))
                        .px(px(12.))
                        .py(px(9.))
                        .rounded(px(12.))
                        .bg(p.sel)
                        .child(PROMPT),
                ),
        );
    for s in steps.iter() {
        let (c, g) = match s.state {
            StepState::Done => (tone(p, IconTone::Done), "✓"),
            StepState::Running => (tone(p, IconTone::Running), "•"),
            StepState::Waiting => (tone(p, IconTone::Approval), "!"),
            StepState::Skipped => (p.mute, "–"),
            StepState::Pending => (p.line, ""),
        };
        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .font_weight(FontWeight::MEDIUM)
                    .when(s.state == StepState::Pending, |d| d.text_color(p.mute))
                    .child(clip(s.title))
                    .when(s.state == StepState::Done, |d| {
                        d.child(quiet(p, fmt_secs(s.secs)))
                    }),
            );
        if s.state != StepState::Pending && !s.tool.is_empty() {
            body = body.child(
                div()
                    .flex()
                    .gap(px(8.))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.inset)
                    .text_size(px(12.))
                    .child(div().flex_none().text_color(p.fg2).child(s.tool))
                    .child(div().text_color(p.mute).child(clip(s.arg))),
            );
        }
        if s.state == StepState::Waiting {
            body = body.child(
                div()
                    .p(px(12.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(c)
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(format!(
                        "Run this in your {} terminal?",
                        t.env.label().to_lowercase()
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(press(p, "Allow once", true, act, "allow"))
                            .child(press(
                                p,
                                "Always in this folder",
                                false,
                                act,
                                "allow-folder",
                            ))
                            .child(press(p, "Deny", false, act, "deny")),
                    ),
            );
        }
        tl = tl.child(
            div()
                .flex()
                .gap(px(12.))
                .max_w(px(640.))
                .child(
                    div()
                        .flex_none()
                        .w(px(22.))
                        .h(px(22.))
                        .rounded(px(11.))
                        .border_1()
                        .border_color(c)
                        .bg(c.opacity(0.16))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.))
                        .text_color(c)
                        .child(g),
                )
                .child(body),
        );
    }
    let head = div()
        .min_h(px(64.))
        .px(px(20.))
        .flex()
        .items_center()
        .gap(px(10.))
        .border_b_1()
        .border_color(p.line)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(5.))
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(clip(t.title.clone())),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .child(pill(p, status))
                        .child(quiet(p, t.env.label()))
                        .child(quiet(
                            p,
                            format!("Step {d} of {n} · {}", fmt_secs(elapsed(&steps))),
                        )),
                ),
        )
        .child(btn(p, "Pause", false));
    let mut panel = div()
        .w(px(340.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line);
    let mut tabs = div()
        .h(px(44.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(2.))
        .border_b_1()
        .border_color(p.line);
    for (i, (_, l)) in PANEL_TABS.iter().enumerate() {
        tabs = tabs.child(
            div()
                .h(px(30.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .when(i == 0, |d| d.bg(p.sel).text_color(p.fg))
                .text_color(p.mute)
                .child(*l),
        );
    }
    panel = panel.child(tabs);
    let mut arts = div().p(px(12.)).flex().flex_col().gap(px(4.));
    for (a, sz) in artifacts(&steps) {
        arts = arts.child(
            div()
                .min_h(px(46.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .child(file_tile(p, a))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(a))
                        .child(quiet(p, sz)),
                ),
        );
    }
    panel = panel.child(arts);
    div()
        .size_full()
        .flex()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(head)
                .child(tl),
        )
        .child(panel)
}

fn board_view(p: &ThPal, st: ScreenState, env: EnvMode, act: &Option<Act>) -> Div {
    let list: Vec<Task> = tasks(st).into_iter().filter(|t| t.env == env).collect();
    let head = div()
        .min_h(px(64.))
        .px(px(20.))
        .flex()
        .items_center()
        .gap(px(10.))
        .border_b_1()
        .border_color(p.line)
        .child(
            div()
                .flex_1()
                .text_size(px(16.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Tasks"),
        )
        .child(quiet(p, format!("{} · {} tasks", env.label(), list.len())))
        .child(press(p, "New task", true, act, "new-task"));
    if st == ScreenState::Loading {
        return div()
            .size_full()
            .flex()
            .flex_col()
            .child(head)
            .child(skel(p, 6));
    }
    if list.is_empty() {
        return div()
            .size_full()
            .flex()
            .flex_col()
            .child(head)
            .child(super::screensfid::empty(
                p,
                "No tasks yet",
                "Queued, running and finished tasks line up here.",
            ));
    }
    let mut cols = div().flex_1().min_h_0().p(px(16.)).flex().gap(px(12.));
    for (ci, s) in COLUMNS.iter().enumerate() {
        let mut col = div()
            .flex_1()
            .min_w_0()
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .rounded(px(12.))
            .bg(p.inset)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(pill(p, *s))
                    .child(quiet(
                        p,
                        list.iter()
                            .filter(|t| t.status.column() == ci)
                            .count()
                            .to_string(),
                    )),
            );
        for t in list.iter().filter(|t| t.status.column() == ci) {
            let id: &'static str = match t.id {
                1 => "open-1",
                2 => "open-2",
                3 => "open-3",
                4 => "open-4",
                5 => "open-5",
                6 => "open-6",
                7 => "open-7",
                8 => "open-8",
                _ => "open-9",
            };
            let f = fire(act, id);
            col = col.child(
                div()
                    .id(SharedString::from(id))
                    .on_click(move |_, w, cx| f(w, cx))
                    .p(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .child(div().font_weight(FontWeight::MEDIUM).child(t.title.clone()))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .items_center()
                            .child(tone_ic(p, IconTone::Folder))
                            .child(clip(
                                t.project.map(|i| PROJECTS[i].1).unwrap_or("No project"),
                            ))
                            .child(quiet(p, t.ago.clone())),
                    )
                    .when(t.status == Status::Failed, |d| {
                        d.child(pill(p, Status::Failed))
                    }),
            );
        }
        cols = cols.child(col);
    }
    div().size_full().flex().flex_col().child(head).child(cols)
}

fn screen(
    theme: &'static Theme,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    view: &'static str,
) -> Div {
    let p = th_pal(theme);
    let env = if extra == "cloud" {
        EnvMode::Cloud
    } else {
        EnvMode::Local
    };
    let loading = st == ScreenState::Loading;
    let main = match view {
        "task" => task_view(&p, st, extra == "approval", act),
        "board" => board_view(&p, st, env, act),
        _ => start_view(&p, env, loading, act),
    };
    let mut main = div()
        .flex_1()
        .min_w_0()
        .h_full()
        .bg(p.card)
        .overflow_hidden()
        .child(main);
    main = match lay {
        ShellLayout::Inset => main
            .m(px(8.))
            .ml(px(0.))
            .rounded(px(12.))
            .border_1()
            .border_color(p.line),
        ShellLayout::Flat => main.border_l_1().border_color(p.line),
    };
    let mut root = div()
        .size_full()
        .flex()
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.));
    if extra != "hidden" {
        root = root.child(sidebar(&p, st, env, view, extra == "rail", act));
    } else {
        main = main.ml(px(8.));
    }
    root.child(main)
}
