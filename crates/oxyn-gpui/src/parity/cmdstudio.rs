//! Screens S10 in GPUI (Agent apps II): Command Center and Session Studio, two desktop
//! coding-agent apps. Static snapshots of one state each, like the other screen sets: the sidebar
//! (threads by project, or sessions), the top bar, the conversation (work log, or read / edit /
//! bash blocks with a permission prompt and the todo list), the composer and the review / diff pane
//! with hunks. `.state()` sets Data / Loading / Empty and `.extra()` the own state (`worktree`,
//! `permission`) or the Oxyn options `rail` and `hidden`. Presses fire `new`, `send`, `accept-all`,
//! `revert-all`, `allow-once`, `allow-always`, `deny` and `handoff` through `on_act`. Rules come
//! from `oxyn_core::parity::cmdstudio`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::cmdstudio::{
    BState, Block, CC_APP, CC_MODELS, CC_PROJECTS, CC_PROMPT, CC_WORK, DiffFile, Mode, Place,
    REASONING, SS_APP, SS_MODELS, ScreenState, TState, branch_for, cc_diff, file_stat, hunk_stat,
    review_line, sample_queue, sample_todos, session_blocks, sessions, ss_diff, threads,
};
use oxyn_core::parity::cmdstudio::{
    PAGES, PANES, SIDE_CHAT, TRIGGERS, automations, sample_terms, todo_progress,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::{clip, mark};
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{gh, skel, tone_ic};
use super::workscr::{btn, quiet, tone};

macro_rules! s10_el {
    ($(#[$m:meta])* $name:ident, $app:literal) => {
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
            /// The own state, `rail` or `hidden` (see the module docs).
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
            /// Press ids (`new`, `send`, `accept-all`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                screen(self.theme, self.layout, self.state, self.extra, &self.act, $app)
            }
        }
    };
}

s10_el!(
    /// Command Center: threads by project, the running thread's work log, the queue, the composer
    /// with model and reasoning, and the review pane.
    CommandCenterEl,
    "cc"
);
s10_el!(
    /// Session Studio: sessions, read / edit / bash blocks with a permission prompt, the todo list,
    /// the composer with permission mode and model, and the diff pane.
    SessionStudioEl,
    "ss"
);

fn tone_of(p: &ThPal, t: &str) -> Hsla {
    match t {
        "run" => tone(p, IconTone::Running),
        "wait" => tone(p, IconTone::Approval),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        _ => p.mute,
    }
}

fn dot(c: Hsla) -> Div {
    div().flex_none().w(px(8.)).h(px(8.)).rounded(px(4.)).bg(c)
}

fn pill(p: &ThPal, l: &'static str, t: &str) -> Div {
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
        .child(div().w(px(6.)).h(px(6.)).rounded(px(3.)).bg(tone_of(p, t)))
        .child(l)
}

fn chip(p: &ThPal, lead: Option<Div>, l: impl Into<SharedString>, border: bool) -> Div {
    let mut d = div()
        .flex_none()
        .h(px(30.))
        .px(px(9.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(15.))
        .text_size(px(12.5))
        .text_color(p.fg2);
    if border {
        d = d.border_1().border_color(p.line);
    }
    if let Some(x) = lead {
        d = d.child(x);
    }
    d.child(l.into()).child("▾")
}

fn brand(p: &ThPal, id: &str) -> Div {
    let mut d = div()
        .flex_none()
        .w(px(16.))
        .h(px(16.))
        .flex()
        .items_center()
        .justify_center();
    if let Some(m) = mark(p, id, 14.) {
        d = d.child(m);
    }
    d
}

fn press(p: &ThPal, l: &'static str, primary: bool, act: &Option<Act>, id: &'static str) -> Div {
    let f = fire(act, id);
    div().child(
        btn(p, l, primary)
            .id(SharedString::from(id))
            .on_click(move |_, w, cx| f(w, cx)),
    )
}

fn mono(t: impl Into<SharedString>) -> Div {
    div()
        .font_family("monospace")
        .text_size(px(11.5))
        .child(t.into())
}

fn stat(p: &ThPal, a: u32, d: u32) -> Div {
    div()
        .flex_none()
        .flex()
        .gap(px(5.))
        .font_family("monospace")
        .text_size(px(11.5))
        .child(
            div()
                .text_color(tone(p, IconTone::Done))
                .child(format!("+{a}")),
        )
        .child(
            div()
                .text_color(tone(p, IconTone::Failed))
                .child(format!("−{d}")),
        )
}

fn sidebar(p: &ThPal, st: ScreenState, app: &str, rail: bool, act: &Option<Act>) -> Div {
    let w = if rail { 56. } else { 272. };
    let f = fire(act, "new");
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
                        .w(px(16.))
                        .h(px(16.))
                        .rounded(px(5.))
                        .border_2()
                        .border_color(p.fg)
                        .flex_none(),
                )
                .when(!rail, |d| {
                    d.child(if app == "cc" { CC_APP } else { SS_APP })
                }),
        )
        .child(
            div()
                .id("new")
                .on_click(move |_, w, cx| f(w, cx))
                .h(px(34.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .text_color(p.fg2)
                .child(if rail {
                    "+"
                } else if app == "cc" {
                    "New thread"
                } else {
                    "New session"
                }),
        );
    if rail {
        return s.child(div().flex_1()).child(
            div()
                .flex()
                .justify_center()
                .child(super::setscr::av(p, "RV", 28.)),
        );
    }
    let mut body = div().flex_1().min_h_0().overflow_hidden().flex().flex_col();
    if st == ScreenState::Loading {
        body = body.child(skel(p, 7));
    } else if app == "cc" {
        let list = threads(st);
        if list.is_empty() {
            body = body.child(
                div()
                    .py(px(14.))
                    .flex()
                    .justify_center()
                    .text_color(p.mute)
                    .child("No threads yet"),
            );
        }
        for (i, (_, n)) in CC_PROJECTS.iter().enumerate() {
            let inner: Vec<_> = list.iter().filter(|t| t.project == i).collect();
            if inner.is_empty() {
                continue;
            }
            body = body.child(
                div()
                    .h(px(30.))
                    .px(px(8.))
                    .mt(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_color(p.fg2)
                    .child(tone_ic(p, IconTone::Folder))
                    .child(clip(*n))
                    .child(quiet(p, inner.len().to_string())),
            );
            for t in inner {
                let on = t.id == 1;
                body = body.child(
                    div()
                        .h(px(32.))
                        .pl(px(24.))
                        .pr(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .rounded(px(8.))
                        .when(on, |d| d.bg(p.sel).text_color(p.fg))
                        .text_size(px(12.5))
                        .text_color(p.fg2)
                        .child(dot(tone_of(p, t.state.tone())))
                        .child(clip(t.title.clone()))
                        .child(stat(p, t.stat.0, t.stat.1)),
                );
            }
        }
    } else {
        let list = sessions(st);
        body = body.child(gh(p, "Sessions"));
        if list.is_empty() {
            body = body.child(
                div()
                    .py(px(14.))
                    .flex()
                    .justify_center()
                    .text_color(p.mute)
                    .child("No sessions yet"),
            );
        }
        for (id, title, folder, ago, t) in list {
            body = body.child(
                div()
                    .min_h(px(46.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .rounded(px(9.))
                    .when(id == 1, |d| d.bg(p.sel))
                    .child(dot(tone_of(p, t)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(clip(title).text_color(p.fg))
                            .child(
                                clip(format!("{folder} · {ago}"))
                                    .text_size(px(11.5))
                                    .text_color(p.mute),
                            ),
                    ),
            );
        }
    }
    s = s.child(body);
    s.child(
        div()
            .mt(px(6.))
            .pt(px(6.))
            .border_t_1()
            .border_color(p.line)
            .flex()
            .items_center()
            .gap(px(9.))
            .child(super::setscr::av(p, "RV", 28.))
            .child(clip(oxyn_core::parity::cmdstudio::PERSON)),
    )
}

fn diff_pane(
    p: &ThPal,
    st: ScreenState,
    files: &[DiffFile],
    title: &'static str,
    act: &Option<Act>,
) -> Div {
    let mut d = div()
        .w(px(440.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line)
        .bg(p.card);
    if st == ScreenState::Loading {
        return d.child(div().p(px(10.)).child(skel(p, 8)));
    }
    if files.is_empty() {
        return d.child(
            div()
                .flex_1()
                .p(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .text_color(p.mute)
                .child("Changes show up here as the agent edits."),
        );
    }
    d = d.child(
        div()
            .min_h(px(52.))
            .px(px(14.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(p.line)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                    .child(
                        clip(review_line(files))
                            .text_size(px(11.5))
                            .text_color(p.mute),
                    ),
            )
            .child(press(p, "Revert all", false, act, "revert-all"))
            .child(press(p, "Accept all", true, act, "accept-all")),
    );
    let mut body = div()
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .p(px(10.))
        .flex()
        .flex_col()
        .gap(px(10.));
    for f in files {
        let (a, r) = file_stat(f);
        let mut card = div()
            .rounded(px(10.))
            .border_1()
            .border_color(p.line)
            .overflow_hidden()
            .child(
                div()
                    .h(px(36.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .bg(p.inset)
                    .child(
                        mono(f.path)
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .text_color(p.fg),
                    )
                    .child(stat(p, a, r)),
            );
        for h in &f.hunks {
            let (ha, hr) = hunk_stat(h);
            card = card.child(
                div()
                    .h(px(30.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .border_t_1()
                    .border_color(p.line)
                    .text_color(p.mute)
                    .child(mono(h.head).flex_1().min_w_0().overflow_hidden())
                    .child(quiet(p, format!("+{ha} −{hr}"))),
            );
            for (k, t) in h.lines.iter().take(9) {
                let (bg, sg) = match k {
                    '+' => (
                        Some(tone(p, IconTone::Done).opacity(0.13)),
                        tone(p, IconTone::Done),
                    ),
                    '-' => (
                        Some(tone(p, IconTone::Failed).opacity(0.12)),
                        tone(p, IconTone::Failed),
                    ),
                    _ => (None, p.mute),
                };
                let mut ln = div()
                    .flex()
                    .h(px(19.))
                    .font_family("monospace")
                    .text_size(px(11.5))
                    .text_color(p.fg2)
                    .child(
                        div()
                            .w(px(22.))
                            .flex_none()
                            .flex()
                            .justify_center()
                            .text_color(sg)
                            .child(k.to_string()),
                    );
                if let Some(b) = bg {
                    ln = ln.bg(b).text_color(p.fg);
                }
                card = card.child(
                    ln.child(
                        div()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child(t.to_string()),
                    ),
                );
            }
        }
        body = body.child(card);
    }
    d.child(body)
}

fn composer(p: &ThPal, chips: Vec<Div>, hint: &'static str, plan: bool, act: &Option<Act>) -> Div {
    let f = fire(act, "send");
    let mut bar = div().flex().items_center().gap(px(6.));
    for c in chips {
        bar = bar.child(c);
    }
    div()
        .p(px(10.))
        .pl(px(14.))
        .rounded(px(16.))
        .border_1()
        .border_color(p.line)
        .when(plan, |d| d.border_2())
        .bg(p.card)
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(div().min_h(px(40.)).text_color(p.mute).child(hint))
        .child(
            bar.child(div().flex_1()).child(
                div()
                    .id("send")
                    .on_click(move |_, w, cx| f(w, cx))
                    .w(px(30.))
                    .h(px(30.))
                    .rounded(px(15.))
                    .bg(p.fg)
                    .text_color(p.card)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("↑"),
            ),
        )
}

fn cc_body(p: &ThPal, st: ScreenState, wt: bool, act: &Option<Act>) -> Div {
    let col = div().flex_1().min_w_0().h_full().flex().flex_col();
    let chips = vec![
        chip(p, Some(brand(p, CC_MODELS[0].1)), CC_MODELS[0].0, false),
        chip(p, None, REASONING[1].0, false),
    ];
    if st == ScreenState::Loading {
        return col.child(div().p(px(24.)).child(skel(p, 8)));
    }
    if st == ScreenState::Empty {
        return col
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("What should we build in {}?", CC_PROJECTS[0].1)),
                    )
                    .child(chip(
                        p,
                        Some(tone_ic(p, IconTone::Folder)),
                        CC_PROJECTS[0].1,
                        true,
                    )),
            )
            .child(div().px(px(24.)).pb(px(16.)).child(composer(
                p,
                chips,
                "Ask for a change · @ to add files",
                false,
                act,
            )));
    }
    let mut feed = div()
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .px(px(24.))
        .py(px(22.))
        .flex()
        .flex_col()
        .gap(px(12.));
    if wt {
        feed = feed.child(
            div()
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .bg(p.inset)
                .flex()
                .items_center()
                .gap(px(9.))
                .text_color(p.fg2)
                .child(clip(format!(
                    "Working in a worktree on {}. Your checkout stays untouched.",
                    branch_for(Place::Worktree, 1)
                )))
                .child(press(p, "Hand off to Local", false, act, "handoff")),
        );
    }
    feed = feed.child(
        div().flex().justify_end().child(
            div()
                .max_w(px(560.))
                .px(px(14.))
                .py(px(10.))
                .rounded(px(14.))
                .bg(p.sel)
                .child(CC_PROMPT),
        ),
    );
    let mut log = div().border_l_2().border_color(p.line).flex().flex_col();
    for (i, (_, l, d)) in CC_WORK.iter().enumerate() {
        let live = i == 4 && !wt;
        let (l, d) = if i == 4 && wt {
            ("Ran cargo test retry", "3 passed")
        } else {
            (*l, *d)
        };
        log = log.child(
            div()
                .h(px(28.))
                .px(px(10.))
                .flex()
                .items_center()
                .gap(px(9.))
                .text_size(px(12.5))
                .text_color(if live { p.fg } else { p.fg2 })
                .child(dot(if live {
                    tone(p, IconTone::Running)
                } else {
                    tone(p, IconTone::Done)
                }))
                .child(clip(l))
                .child(quiet(p, d)),
        );
    }
    feed = feed.child(log);
    if wt {
        feed = feed.child("Uploads now retry a 503 up to 4 times with exponential backoff. All 3 retry tests pass.");
    } else {
        feed = feed.child(div().text_color(p.mute).child("Working · 4 of 5 steps"));
    }
    let mut foot = div().px(px(24.)).pb(px(16.)).flex().flex_col().gap(px(8.));
    if !wt {
        let q = sample_queue();
        let mut qd = div()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .bg(p.inset)
            .p(px(6.))
            .child(
                div()
                    .h(px(26.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_color(p.fg2)
                    .child("Queued next")
                    .child(quiet(p, q.len().to_string())),
            );
        for (i, (_, t)) in q.iter().enumerate() {
            qd = qd.child(
                div()
                    .h(px(28.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_color(p.fg2)
                    .child(quiet(p, (i + 1).to_string()))
                    .child(clip(t.clone())),
            );
        }
        foot = foot.child(qd);
    }
    col.child(feed).child(foot.child(composer(
        p,
        chips,
        if wt {
            "Ask for a change · @ to add files"
        } else {
            "Queue a follow-up for when this finishes…"
        },
        false,
        act,
    )))
}

fn block(p: &ThPal, b: &Block, act: &Option<Act>) -> Div {
    match b.kind {
        "user" => div().flex().justify_end().child(
            div()
                .max_w(px(560.))
                .px(px(14.))
                .py(px(10.))
                .rounded(px(14.))
                .bg(p.sel)
                .child(b.body.clone()),
        ),
        "text" | "plan" => div().text_color(p.fg).child(b.body.clone()),
        k => {
            let verb = match k {
                "read" => "Read",
                "edit" => "Edit",
                _ => "Bash",
            };
            let t = match b.state {
                BState::Running => "run",
                BState::Asking => "wait",
                _ => "ok",
            };
            let mut d = div()
                .rounded(px(11.))
                .border_1()
                .border_color(if t == "wait" {
                    tone(p, IconTone::Approval)
                } else {
                    p.line
                })
                .overflow_hidden()
                .child(
                    div()
                        .h(px(36.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_color(p.fg2)
                        .child(dot(tone_of(p, t)))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.fg)
                                .child(verb),
                        )
                        .child(
                            mono(if k == "bash" {
                                format!("$ {}", b.body)
                            } else {
                                b.body.clone()
                            })
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden(),
                        )
                        .when(k == "read", |d| {
                            d.child(quiet(p, b.out.first().cloned().unwrap_or_default()))
                        }),
                );
            if k == "edit" {
                for l in b.out.iter().take(4) {
                    let add = l.starts_with('+');
                    let c = if add {
                        tone(p, IconTone::Done)
                    } else {
                        tone(p, IconTone::Failed)
                    };
                    d = d.child(
                        div()
                            .h(px(19.))
                            .px(px(10.))
                            .bg(c.opacity(0.12))
                            .child(mono(l.clone()).text_color(p.fg)),
                    );
                }
            }
            if b.state == BState::Asking {
                d = d.child(
                    div()
                        .p(px(12.))
                        .border_t_1()
                        .border_color(p.line)
                        .bg(p.inset)
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .child("Run this command? It runs the invoice tests in billing-svc.")
                        .child(
                            div()
                                .flex()
                                .gap(px(6.))
                                .child(press(p, "Allow once", true, act, "allow-once"))
                                .child(press(p, "Always allow", false, act, "allow-always"))
                                .child(press(p, "Deny", false, act, "deny")),
                        ),
                );
            }
            d
        }
    }
}

fn ss_body(p: &ThPal, st: ScreenState, ask: bool, act: &Option<Act>) -> Div {
    let col = div().flex_1().min_w_0().h_full().flex().flex_col();
    let chips = vec![
        chip(p, None, Mode::Ask.label(), false),
        chip(p, Some(brand(p, SS_MODELS[1].1)), SS_MODELS[1].0, false),
    ];
    if st == ScreenState::Loading {
        return col.child(div().p(px(24.)).child(skel(p, 8)));
    }
    if st == ScreenState::Empty {
        return col
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("What are we changing today?"),
                    ),
            )
            .child(div().px(px(24.)).pb(px(16.)).child(composer(
                p,
                chips,
                "Ask for a change · Shift + Tab switches mode",
                false,
                act,
            )));
    }
    let mut feed = div()
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .px(px(24.))
        .py(px(22.))
        .flex()
        .flex_col()
        .gap(px(12.));
    for b in session_blocks(ask) {
        feed = feed.child(block(p, &b, act));
    }
    let todos = sample_todos(ask);
    let (d, n) = todo_progress(&todos);
    let mut td = div()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.inset)
        .p(px(6.))
        .child(
            div()
                .h(px(26.))
                .px(px(6.))
                .flex()
                .items_center()
                .gap(px(8.))
                .text_color(p.fg2)
                .child("Todos")
                .child(quiet(p, format!("{d}/{n}"))),
        );
    for (t, s) in todos {
        let c = match s {
            BState::Done => tone(p, IconTone::Done),
            BState::Running => tone(p, IconTone::Running),
            BState::Asking => tone(p, IconTone::Approval),
            _ => p.mute,
        };
        td = td.child(
            div()
                .h(px(26.))
                .px(px(6.))
                .flex()
                .items_center()
                .gap(px(9.))
                .text_color(if s == BState::Done { p.mute } else { p.fg2 })
                .child(dot(c))
                .child(clip(t)),
        );
    }
    col.child(feed).child(
        div()
            .px(px(24.))
            .pb(px(16.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(td)
            .child(composer(
                p,
                chips,
                "Ask for a change · Shift + Tab switches mode",
                false,
                act,
            )),
    )
}

fn side_col(p: &ThPal) -> Div {
    div()
        .w(px(440.))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(p.line)
        .bg(p.card)
}

fn tabs_row(p: &ThPal, on: &str) -> Div {
    let mut r = div()
        .flex()
        .gap(px(4.))
        .px(px(8.))
        .pt(px(6.))
        .border_b_1()
        .border_color(p.line);
    for (k, l) in [
        ("review", "Review"),
        ("terminal", "Terminal"),
        ("browser", "Browser"),
    ] {
        r = r.child(
            div()
                .h(px(32.))
                .px(px(10.))
                .flex()
                .items_center()
                .text_size(px(12.5))
                .when(k == on, |d| {
                    d.border_b_2().border_color(p.fg).text_color(p.fg)
                })
                .when(k != on, |d| d.text_color(p.mute))
                .child(l),
        );
    }
    r
}

fn term_pane(p: &ThPal) -> Div {
    let terms = sample_terms();
    let mut tabs = div()
        .flex()
        .gap(px(4.))
        .p(px(6.))
        .border_b_1()
        .border_color(p.line);
    for (i, t) in terms.iter().enumerate() {
        tabs = tabs.child(
            div()
                .h(px(26.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .when(i == 0, |d| d.bg(p.sel))
                .text_size(px(12.))
                .child(t.title.clone()),
        );
    }
    tabs = tabs.child(quiet(p, "+"));
    let mut out = div().flex_1().p(px(12.)).flex().flex_col().bg(p.inset);
    for l in &terms[0].lines {
        out = out.child(mono(l.clone()).text_color(if l.starts_with("$ ") { p.fg } else { p.fg2 }));
    }
    side_col(p)
        .child(tabs_row(p, "terminal"))
        .child(tabs)
        .child(out)
}

fn browser_pane(p: &ThPal) -> Div {
    let (addr, title, lines) = PAGES[0];
    let mut page = div()
        .flex_1()
        .p(px(20.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .bg(gpui::rgb(0xFFFFFF))
        .text_color(gpui::rgb(0x111111))
        .child(
            div()
                .text_size(px(20.))
                .font_weight(FontWeight::BOLD)
                .child(title),
        );
    for l in lines {
        page = page.child(
            div()
                .p(px(10.))
                .rounded(px(8.))
                .border_1()
                .border_color(gpui::rgb(0xE6E6E6))
                .child(*l),
        );
    }
    side_col(p)
        .child(tabs_row(p, "browser"))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .p(px(6.))
                .border_b_1()
                .border_color(p.line)
                .child(quiet(p, "←  ↻"))
                .child(
                    div()
                        .flex_1()
                        .h(px(28.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .rounded(px(8.))
                        .bg(p.inset)
                        .text_size(px(12.5))
                        .child(addr),
                ),
        )
        .child(page)
}

fn auto_body(p: &ThPal, st: ScreenState) -> Div {
    let list = automations(st);
    let mut d = div()
        .flex_1()
        .min_w_0()
        .p(px(24.))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Automations"),
        )
        .child(quiet(
            p,
            "Threads that start themselves on a schedule or an event, each in its own worktree.",
        ));
    if st == ScreenState::Loading {
        return d.child(skel(p, 6));
    }
    let mut l = div()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line);
    for a in &list {
        let (tone, ago) = &a.last;
        l = l.child(
            div()
                .p(px(12.))
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
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .when(!a.on, |x| x.text_color(p.mute))
                                .child(a.name.clone()),
                        )
                        .child(quiet(
                            p,
                            format!("{} · {}", TRIGGERS[a.trigger].1, CC_PROJECTS[a.project].1),
                        )),
                )
                .child(pill(
                    p,
                    match *tone {
                        "ok" => "Ran",
                        "bad" => "Failed",
                        "run" => "Running",
                        _ => "Not run yet",
                    },
                    tone,
                ))
                .child(quiet(p, ago.clone()))
                .child(pill(
                    p,
                    if a.on { "On" } else { "Off" },
                    if a.on { "ok" } else { "" },
                )),
        );
    }
    if list.is_empty() {
        d = d.child(quiet(p, "No automations yet."));
    } else {
        d = d.child(l);
    }
    d
}

fn ss_panes(
    p: &ThPal,
    st: ScreenState,
    files: &[DiffFile],
    arrange: bool,
    act: &Option<Act>,
) -> Div {
    let head = |l: &'static str| {
        div()
            .h(px(36.))
            .px(px(12.))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(arrange, |d| d.child(quiet(p, "⋮⋮")))
            .child(div().font_weight(FontWeight::SEMIBOLD).child(l))
            .child(div().flex_1())
            .when(arrange, |d| d.child(quiet(p, "↑  ↓")))
    };
    let pane = |d: Div| {
        if arrange {
            d.m(px(8.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .bg(p.inset)
        } else {
            d.border_b_1().border_color(p.line)
        }
    };
    let mut chat = div().flex().flex_col().gap(px(6.)).px(px(12.)).pb(px(12.));
    for (me, t) in SIDE_CHAT {
        chat = chat.child(
            div().flex().when(me, |d| d.justify_end()).child(
                div()
                    .max_w(px(360.))
                    .p(px(8.))
                    .rounded(px(10.))
                    .bg(if me { p.sel } else { p.inset })
                    .text_size(px(12.5))
                    .child(t),
            ),
        );
    }
    let mut col = side_col(p);
    if arrange {
        for (_, l) in PANES {
            col = col.child(pane(div().child(head(l))));
        }
        return col;
    }
    let diff = diff_pane(p, st, files, "Diff", act);
    col.child(div().flex_1().min_h_0().flex().flex_col().child(diff))
        .child(pane(div().child(head("Side chat")).child(chat)))
}

fn screen(
    theme: &'static Theme,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    app: &'static str,
) -> Div {
    let p = th_pal(theme);
    let samp = st == ScreenState::Sample;
    let (title, pl, body, files, ptitle) = if app == "cc" {
        let wt = extra == "worktree";
        let state = if wt { TState::Review } else { TState::Running };
        (
            "Retry failed uploads with backoff",
            Some((state.label(), state.tone())),
            if extra == "automations" {
                auto_body(&p, st)
            } else {
                cc_body(&p, st, wt, act)
            },
            if samp { cc_diff() } else { Vec::new() },
            "Review",
        )
    } else {
        (
            "Cache invoice PDFs",
            None,
            ss_body(&p, st, extra == "permission", act),
            if samp { ss_diff() } else { Vec::new() },
            "Diff",
        )
    };
    let mut bar = div()
        .h(px(48.))
        .flex_none()
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(p.line)
        .child(div().font_weight(FontWeight::SEMIBOLD).child(if samp {
            title
        } else if app == "cc" {
            "New thread"
        } else {
            "New session"
        }));
    if samp && let Some((l, t)) = pl {
        bar = bar.child(pill(&p, l, t));
    }
    bar = bar.child(div().flex_1());
    if app == "cc" && st != ScreenState::Loading {
        let place = if extra == "worktree" {
            Place::Worktree
        } else {
            Place::Local
        };
        bar = bar.child(chip(&p, None, place.label(), true));
    }
    let main = div()
        .flex_1()
        .min_w_0()
        .h_full()
        .bg(p.card)
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(bar)
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .child(body)
                .child(match (app, extra) {
                    ("cc", "terminal") => term_pane(&p),
                    ("cc", "browser") => browser_pane(&p),
                    ("ss", "chat") => ss_panes(&p, st, &files, false, act),
                    ("ss", "arrange") => ss_panes(&p, st, &files, true, act),
                    _ => diff_pane(&p, st, &files, ptitle, act),
                }),
        );
    let mut main = match lay {
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
        root = root.child(sidebar(&p, st, app, extra == "rail", act));
    } else {
        main = main.ml(px(8.));
    }
    root.child(main)
}
