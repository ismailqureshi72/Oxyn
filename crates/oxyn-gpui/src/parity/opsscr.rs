//! Screens S4 in GPUI: Agents, Skills, Schedules, Usage, Permissions and Run Detail on the shared
//! frame with the Fieldwork persona's sidebar (`oxyn_core::parity::workscr::persona("fieldwork")`).
//! The elements draw one state at a time: `.state()` for Data / Loading / Empty and `.extra()` for
//! the screen's own state (`creating`, `installing`, `running`, `over`, `handsoff`, `stopped`) or
//! an Oxyn option (`spend`, `budget`, `compact`, `byuse`, `hidepaused`, `month`, `daily`, `alert`,
//! `lockred`, `errors`, `cost`). Presses fire `nav-<key>`, `new-agent`, `message`, `install`,
//! `run-now`, `level-<key>`, `stop` and similar ids through `on_act`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::opsscr::{
    AGENT_TABS, AgentState, BUDGET, COST_MOST, LAST_RUNS, LATELY, LEVELS, MODELS, RED_LINES,
    RUN_NOW, Rule, SKILL_CATS, STRIP_MINS, ScreenState, SpanKind, TODAY, agent_counts, agent_view,
    agents, apps_words, bar, bars, breakdown, counts_words, cumulative, daily_spend, eur, forecast,
    forecast_words, mark_bits, pct, perm_apps, perm_counts, rule_for, run_totals, sched_counts,
    schedules, secs, skill_steps, skill_view, skills, span_io, span_view, spans, strip_hours,
    strip_marks, ticks, toks, uses_n,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_core::parity::workscr::persona;
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, skel};
use super::workscr::{btn, find, none, page, quiet, side, tabs, tone};
use crate::brand::brand_mark;

macro_rules! ops_el {
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
            /// Press ids (`nav-<key>`, `new-agent`, `install`, `run-now`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}

fn st_tone(p: &ThPal, s: &str) -> Hsla {
    match s {
        "warn" => tone(p, IconTone::Approval),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        "run" => tone(p, IconTone::Running),
        _ => p.mute,
    }
}

fn hue(p: &ThPal, h: &str) -> Hsla {
    match h {
        "purple" => tone(p, IconTone::Skills),
        "green" => tone(p, IconTone::Sheet),
        "orange" => tone(p, IconTone::Slides),
        "teal" => tone(p, IconTone::Audio),
        "pink" => tone(p, IconTone::Image),
        "blue" => tone(p, IconTone::Doc),
        _ => p.mute,
    }
}

fn pill(l: impl Into<SharedString>, c: Hsla) -> Div {
    div()
        .flex_none()
        .h(px(18.))
        .px(px(6.))
        .rounded(px(5.))
        .bg(c.opacity(0.22))
        .text_size(px(11.5))
        .whitespace_nowrap()
        .child(l.into())
}

fn h3(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div()
        .mt(px(22.))
        .mb(px(8.))
        .text_size(px(12.5))
        .text_color(p.mute)
        .child(t.into())
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

fn work(p: &ThPal, label: impl Into<SharedString>, body: Div) -> Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(
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
                        .gap(px(6.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .child(clip(label))
                        .child(quiet(p, "×")),
                ),
        )
        .child(div().flex_1().min_h_0().px(px(18.)).py(px(18.)).child(body))
}

fn logo(p: &ThPal, app: &str, s: f32) -> Div {
    let t = div()
        .w(px(s + 10.))
        .h(px(s + 10.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.))
        .border_1()
        .border_color(p.line)
        .bg(gpui::white());
    match brand_mark(app, s, gpui::black()) {
        Some(m) => t.child(m),
        None => t
            .text_size(px(s * 0.7))
            .text_color(tone(p, IconTone::Image))
            .child("◎"),
    }
}

fn model(p: &ThPal, m: usize) -> Div {
    let (l, b) = MODELS[m.min(MODELS.len() - 1)];
    let mut d = div()
        .flex_none()
        .h(px(20.))
        .px(px(6.))
        .flex()
        .items_center()
        .gap(px(5.))
        .rounded(px(5.))
        .border_1()
        .border_color(p.line)
        .bg(p.card2)
        .text_size(px(11.5))
        .text_color(p.fg2);
    if let Some(mk) = brand_mark(b, 12., p.fg) {
        d = d.child(mk);
    }
    d.child(l)
}

fn pmark(p: &ThPal, name: &str, h: &str, cell: f32) -> Div {
    let c = hue(p, h);
    let bits = mark_bits(name);
    let mut g = div().flex().flex_col().gap(px(cell / 2.));
    for r in 0..3 {
        let mut row = div().flex().gap(px(cell / 2.));
        for k in 0..3 {
            row = row.child(
                div()
                    .w(px(cell))
                    .h(px(cell))
                    .rounded(px(1.))
                    .bg(if bits[r * 3 + k] { c } else { c.opacity(0.16) }),
            );
        }
        g = g.child(row);
    }
    div()
        .w(px(cell * 8.))
        .h(px(cell * 8.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(g)
}

fn chart(p: &ThPal, days: &[u8], c: Hsla) -> Div {
    let mut r = div()
        .h(px(74.))
        .flex()
        .items_end()
        .gap(px(5.))
        .px(px(4.))
        .border_b_1()
        .border_color(p.line);
    let b = bars(days);
    let n = b.len();
    for (i, h) in b.into_iter().enumerate() {
        r = r.child(
            div()
                .flex_1()
                .h(px(74. * h.max(3) as f32 / 100.))
                .rounded_t(px(2.))
                .bg(if i + 4 >= n { c } else { c.opacity(0.35) }),
        );
    }
    div().flex().flex_col().gap(px(6.)).child(r).child(
        div()
            .flex()
            .justify_between()
            .child(quiet(p, "2 weeks ago"))
            .child(quiet(p, "Today")),
    )
}

fn card(p: &ThPal) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(10.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
}

fn row(p: &ThPal) -> Div {
    div()
        .min_h(px(44.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(10.))
        .border_b_1()
        .border_color(p.line)
}

fn sq(c: Hsla) -> Div {
    div().w(px(8.)).h(px(8.)).flex_none().rounded(px(2.)).bg(c)
}

fn sk(p: &ThPal, st: ScreenState, n: usize) -> Option<Div> {
    (st == ScreenState::Loading).then(|| skel(p, n))
}

fn body(pad: f32) -> Div {
    div()
        .flex_1()
        .min_h_0()
        .px(px(pad))
        .py(px(10.))
        .flex()
        .flex_col()
}

fn agent_hue(id: u32) -> &'static str {
    ["blue", "purple", "pink", "teal", "orange", "grey"][(id as usize + 5) % 6]
}

// ───────────────────────────────────────────────────────────── Agents ──

ops_el!(
    /// Agent / Agents: the team as cards (pixel mark, role, status, app marks, model, runs and
    /// spend) with tabs; `creating` shows New agent with the mark drawn from the typed name.
    AgentsEl
);

impl RenderOnce for AgentsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = agents(st == ScreenState::Sample);
        let c = agent_counts(&all);
        let mut b = body(28.).gap(px(10.));
        if let Some(s) = sk(&p, st, 8) {
            b = b.child(s);
        } else if all.is_empty() {
            b = b.child(none(
                &p,
                "No agents yet",
                "Make one with New agent: a name, a job and the apps it may use.",
            ));
        }
        if self.extra == "creating" {
            b = b.child(
                card(&p)
                    .p(px(14.))
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(pmark(&p, "Scout", "teal", 6.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(div().font_weight(FontWeight::MEDIUM).child("Scout"))
                                    .child(quiet(&p, "The mark draws itself from the name")),
                            ),
                    )
                    .child(find(&p, "Finds leads and qualifies them", 300.))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(btn(&p, "Cancel", false))
                            .child(btn(&p, "Create agent", true)),
                    ),
            );
        }
        let mut grid = div().flex().flex_wrap().gap(px(10.));
        for a in agent_view(&all, "all", self.extra == "spend") {
            let mut logos = div().flex().gap(px(2.));
            for ap in &a.apps {
                logos = logos.child(logo(&p, ap, 11.));
            }
            let pc = match a.state {
                AgentState::Working => tone(&p, IconTone::Running),
                AgentState::Idle => p.mute,
                AgentState::Paused => tone(&p, IconTone::Approval),
            };
            let mut cd = card(&p)
                .w(px(330.))
                .p(px(14.))
                .gap(px(12.))
                .when(a.id == 1, |d| d.border_color(p.fg2))
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(10.))
                        .child(pmark(&p, &a.name, agent_hue(a.id), 3.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(div().font_weight(FontWeight::MEDIUM).child(a.name.clone()))
                                .child(quiet(&p, a.role.clone())),
                        )
                        .child(pill(a.state.label(), pc)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(logos)
                        .child(quiet(&p, apps_words(a.apps.len()))),
                );
            if self.extra == "budget" {
                cd = cd.child(
                    div().h(px(3.)).rounded(px(2.)).bg(p.sel).child(
                        div()
                            .h_full()
                            .w(px(300. * (a.spend.min(5000) as f32) / 5000.))
                            .bg(p.fg2),
                    ),
                );
            }
            grid = grid.child(
                cd.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(model(&p, a.model))
                        .child(div().flex_1())
                        .child(quiet(&p, format!("{} runs", a.runs)))
                        .child(quiet(&p, eur(a.spend))),
                ),
            );
        }
        b = b.child(grid);
        let wk = all.first().map(|a| {
            let f = fire(&self.act, "message");
            work(
                &p,
                a.name.clone(),
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(pmark(&p, &a.name, "blue", 3.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div().font_weight(FontWeight::MEDIUM).child(a.name.clone()),
                                    )
                                    .child(quiet(&p, a.role.clone())),
                            )
                            .child(
                                btn(&p, "Message", true)
                                    .id("message")
                                    .on_click(move |_, w, cx| f(w, cx)),
                            ),
                    )
                    .child(
                        div()
                            .mt(px(12.))
                            .child(kv(&p, "Status", div().child(clip(a.now))))
                            .child(kv(&p, "Model", div().child(model(&p, a.model))))
                            .child(kv(&p, "Reports to", div().child("Rowan Vale"))),
                    )
                    .child(h3(
                        &p,
                        format!("Runs · {} this month · {}", a.runs, eur(a.spend)),
                    ))
                    .child(chart(&p, &a.days, tone(&p, IconTone::Running)))
                    .child(h3(&p, "Instructions"))
                    .child(
                        div()
                            .p(px(12.))
                            .rounded(px(10.))
                            .bg(p.card2)
                            .text_color(p.fg2)
                            .child(a.brief),
                    )
                    .child(h3(&p, "Skills"))
                    .child(div().text_color(p.fg2).child(a.skills.join(" · "))),
            )
        });
        let ti: Vec<_> = AGENT_TABS
            .iter()
            .enumerate()
            .map(|(i, (k, l))| (*k, format!("{l} {}", c[i])))
            .collect();
        let nf = fire(&self.act, "new-agent");
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "agents", st, &self.act),
            page(
                &p,
                Some(("Agents", String::new())),
                vec![
                    tabs(&p, &ti, "all"),
                    div().child(
                        btn(&p, "New agent", true)
                            .id("new-agent")
                            .on_click(move |_, w, cx| nf(w, cx)),
                    ),
                ],
                b,
            ),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Skills ──

ops_el!(
    /// Agent / Skills: installed skills with switches, category chips, Discover with Install;
    /// `installing` shows the ring half full.
    SkillsEl
);

impl RenderOnce for SkillsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = skills(st == ScreenState::Sample);
        let (inst, mut disc) = skill_view(&all, "all", "");
        if self.extra == "byuse" {
            disc.sort_by_key(|s| std::cmp::Reverse(uses_n(s.uses)));
        }
        let mut b = body(28.).gap(px(8.));
        if let Some(s) = sk(&p, st, 9) {
            b = b.child(s);
        } else if all.is_empty() {
            b = b.child(none(
                &p,
                "No skills yet",
                "Skills teach agents one job well. Make one with New skill.",
            ));
        } else {
            let ci: Vec<_> = SKILL_CATS
                .iter()
                .map(|(k, l)| (*k, l.to_string()))
                .collect();
            b = b.child(tabs(&p, &ci, "all"));
            for (title, list, is_disc) in [
                (format!("Installed {}", inst.len()), inst, false),
                (format!("Discover {}", disc.len()), disc, true),
            ] {
                b = b.child(h3(&p, title));
                let mut g = div().flex().flex_wrap().gap(px(10.));
                for s in list {
                    let c = hue(&p, s.hue);
                    let right = if is_disc {
                        let half = self.extra == "installing" && s.id == 13;
                        let f = fire(&self.act, "install");
                        div()
                            .id(SharedString::from(format!("install-{}", s.id)))
                            .w(px(24.))
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(7.))
                            .border_1()
                            .border_color(if half {
                                tone(&p, IconTone::Running)
                            } else {
                                p.line
                            })
                            .child(if half { "◐" } else { "+" })
                            .on_click(move |_, w, cx| f(w, cx))
                            .into_any_element()
                    } else {
                        div()
                            .w(px(34.))
                            .h(px(20.))
                            .rounded(px(10.))
                            .bg(if s.on { p.fg } else { p.sel })
                            .into_any_element()
                    };
                    g = g.child(
                        card(&p)
                            .w(px(220.))
                            .p(px(14.))
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(9.))
                                    .child(
                                        div()
                                            .w(px(20.))
                                            .h(px(20.))
                                            .rounded(px(6.))
                                            .bg(c.opacity(0.2)),
                                    )
                                    .child(clip(s.name))
                                    .child(right),
                            )
                            .when(self.extra != "compact", |d| {
                                d.child(div().text_size(px(12.5)).text_color(p.mute).child(s.blurb))
                            })
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .child(quiet(&p, s.by))
                                    .when(is_disc, |d| {
                                        d.child(quiet(&p, format!("{} use it", s.uses)))
                                    }),
                            ),
                    );
                }
                b = b.child(g);
            }
        }
        let sel = if self.extra == "installing" { 13 } else { 1 };
        let wk = all.iter().find(|s| s.id == sel).map(|s| {
            let mut steps = div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .p(px(12.))
                .rounded(px(10.))
                .bg(p.card2);
            for (i, t) in skill_steps(s.id).iter().enumerate() {
                steps = steps.child(
                    div()
                        .flex()
                        .gap(px(10.))
                        .child(quiet(&p, (i + 1).to_string()))
                        .child(*t),
                );
            }
            work(
                &p,
                s.name,
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(s.name),
                    )
                    .child(quiet(&p, s.by))
                    .child(div().mt(px(10.)).text_color(p.fg2).child(s.blurb))
                    .child(h3(&p, "Runs"))
                    .child(chart(&p, &s.days, hue(&p, s.hue)))
                    .child(h3(&p, "What it does"))
                    .child(steps),
            )
        });
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "skills", st, &self.act),
            page(
                &p,
                Some(("Skills", String::new())),
                vec![find(&p, "Search skills", 220.), btn(&p, "New skill", false)],
                b,
            ),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Schedules ──

ops_el!(
    /// Agent / Schedules: the hours strip with a mark per run, then the schedules with rhythm,
    /// next run, last result, cost and a switch; `running` fills the first mark.
    SchedulesEl
);

impl RenderOnce for SchedulesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = schedules(st == ScreenState::Sample);
        let (on, tot) = sched_counts(&all);
        let running = self.extra == "running";
        let mut b = body(28.).gap(px(8.));
        if let Some(s) = sk(&p, st, 9) {
            b = b.child(s);
        } else if all.is_empty() {
            b = b.child(none(
                &p,
                "No schedules yet",
                "Ask an agent to do something every week or every morning.",
            ));
        } else {
            let w = 640.;
            let mut strip = div().relative().h(px(118.)).w(px(w));
            for (i, (_, x, h)) in strip_marks(&all).into_iter().enumerate() {
                let c = hue(&p, h);
                strip = strip.child(
                    div()
                        .absolute()
                        .left(px(w * x as f32 / 100. - 5.))
                        .top(px(3.))
                        .w(px(10.))
                        .h(px(10.))
                        .rounded(px(5.))
                        .border_2()
                        .border_color(c)
                        .when(running && i == 0, |d| d.bg(c)),
                );
            }
            for (l, x) in strip_hours() {
                strip = strip.child(
                    div()
                        .absolute()
                        .left(px(w * x as f32 / 100.))
                        .bottom_0()
                        .child(quiet(&p, l)),
                );
            }
            let _ = STRIP_MINS;
            b = b.child(h3(&p, "Around now")).child(
                card(&p)
                    .p(px(14.))
                    .child(quiet(&p, "Agent activity"))
                    .child(strip)
                    .child(div().mt(px(8.)).child(if running {
                        "Running now: Competitor scan"
                    } else {
                        "Next: Competitor scan · 07:23, in a moment"
                    })),
            );
            b = b.child(h3(&p, "Schedules"));
            for s in all.iter().filter(|s| self.extra != "hidepaused" || s.on) {
                let (ll, lt) = match s.last {
                    "warn" => ("Needs you", "warn"),
                    "bad" => ("Failed", "bad"),
                    _ => ("Done", ""),
                };
                b = b.child(
                    div()
                        .h(px(44.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .rounded(px(10.))
                        .when(s.id == 1, |d| d.bg(p.sel))
                        .child(sq(hue(&p, s.hue)))
                        .child(div().font_weight(FontWeight::MEDIUM).child(s.name))
                        .child(clip(s.rhythm).text_color(p.mute))
                        .child(div().child(if s.on { s.next.0 } else { "Paused" }))
                        .child(if running && s.id == 1 {
                            pill("Running", tone(&p, IconTone::Running))
                        } else {
                            pill(ll, st_tone(&p, lt))
                        })
                        .child(quiet(&p, eur(s.cost)))
                        .child(div().w(px(34.)).h(px(20.)).rounded(px(10.)).bg(if s.on {
                            p.fg
                        } else {
                            p.sel
                        })),
                );
            }
            if self.extra == "month" {
                b = b.child(quiet(
                    &p,
                    format!(
                        "These schedules cost about {} a month",
                        eur(oxyn_core::parity::opsscr::month_cost(&all))
                    ),
                ));
            }
        }
        let wk = all.first().map(|s| {
            let f = fire(&self.act, "run-now");
            let mut last = card(&p);
            for (t, w, tn) in LAST_RUNS {
                last = last.child(
                    row(&p)
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(t)
                                .child(quiet(&p, w)),
                        )
                        .child(pill(
                            if tn == "bad" { "Failed" } else { "Done" },
                            st_tone(&p, tn),
                        )),
                );
            }
            work(
                &p,
                s.name,
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(sq(hue(&p, s.hue)))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(s.name),
                            ),
                    )
                    .child(quiet(&p, format!("{} · {}", s.rhythm, s.project)))
                    .child(
                        div()
                            .mt(px(12.))
                            .flex()
                            .gap(px(6.))
                            .child(
                                btn(&p, if running { "Running" } else { "Run now" }, true)
                                    .id("run-now")
                                    .on_click(move |_, w, cx| f(w, cx)),
                            )
                            .child(btn(&p, "Pause", false))
                            .child(div().flex_1())
                            .child(quiet(&p, format!("{} per run", eur(s.cost)))),
                    )
                    .child(h3(&p, "What it does"))
                    .child(div().text_color(p.fg2).child(s.what))
                    .child(h3(&p, "Last runs"))
                    .child(last),
            )
        });
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "schedules", st, &self.act),
            page(&p, Some(("Schedules", format!("{on} / {tot}"))), vec![], b),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Usage ──

ops_el!(
    /// Agent / Usage: spend against the budget with a forecast, the month's totals, where it went
    /// and what is worth a look; `over` runs past the budget.
    UsageEl
);

impl RenderOnce for UsageEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let days = daily_spend(st == ScreenState::Sample, self.extra == "over");
        let cum = cumulative(&days);
        let spent = cum.last().copied().unwrap_or(0);
        let run = tone(&p, IconTone::Running);
        let mut b = body(28.).gap(px(8.));
        if let Some(s) = sk(&p, st, 9) {
            b = b.child(s);
        } else if days.is_empty() {
            b = b.child(none(
                &p,
                "No spend yet this month",
                "When agents run, what they cost lands here against your budget.",
            ));
        } else {
            let top = BUDGET.max(forecast(spent, TODAY)) as f32;
            let mut ch = div()
                .h(px(140.))
                .flex()
                .items_end()
                .gap(px(4.))
                .border_t_1()
                .border_color(p.line);
            for (i, c) in cum.iter().enumerate() {
                let v = if self.extra == "daily" {
                    days[i] * 12
                } else {
                    *c
                };
                ch = ch.child(
                    div()
                        .flex_1()
                        .h(px(130. * (v as f32 / top).min(1.)))
                        .rounded_t(px(2.))
                        .bg(run.opacity(0.75)),
                );
            }
            for _ in TODAY..30 {
                ch = ch.child(div().flex_1().h(px(2.)).bg(p.line));
            }
            let mut wh = card(&p);
            for (_, n, h, c, d) in breakdown(false) {
                wh = wh.child(
                    row(&p)
                        .child(sq(hue(&p, h)))
                        .child(clip(n))
                        .child(
                            div().w(px(180.)).h(px(5.)).rounded(px(3.)).bg(p.sel).child(
                                div()
                                    .h_full()
                                    .w(px(180. * (pct(c, spent) * 2).min(100) as f32 / 100.))
                                    .rounded(px(3.))
                                    .bg(hue(&p, h)),
                            ),
                        )
                        .child(div().w(px(72.)).child(eur(c)))
                        .child(quiet(&p, format!("{}%", pct(c, spent))))
                        .child(quiet(
                            &p,
                            if d > 0 {
                                format!("↑{d}%")
                            } else {
                                format!("↓{}%", -d)
                            },
                        )),
                );
            }
            b = b
                .child(
                    card(&p)
                        .p(px(16.))
                        .gap(px(6.))
                        .child(quiet(&p, "Spent in November"))
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(28.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(eur(spent)),
                                )
                                .child(quiet(
                                    &p,
                                    format!("of €{} · {}%", BUDGET / 100, pct(spent, BUDGET)),
                                )),
                        )
                        .child(div().text_color(p.fg2).child(forecast_words(spent, TODAY)))
                        .child(quiet(&p, format!("Budget €{}", BUDGET / 100)))
                        .child(ch)
                        .when(self.extra == "alert", |d| {
                            d.child(quiet(&p, "Alert at 80 % of the budget"))
                        }),
                )
                .child(h3(&p, "Where it went"))
                .child(wh);
        }
        let wk = (!days.is_empty()).then(|| {
            let mut most = card(&p);
            for (t, r, c) in COST_MOST {
                most = most.child(
                    row(&p)
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(t)
                                .child(quiet(&p, format!("{r} runs"))),
                        )
                        .child(eur(c)),
                );
            }
            work(
                &p,
                "Tidewater",
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Tidewater"),
                    )
                    .child(div().mt(px(8.)).text_size(px(22.)).child(eur(11_840)))
                    .child(quiet(
                        &p,
                        format!("{}% of November · ↑23% vs last month", pct(11_840, spent)),
                    ))
                    .child(h3(&p, "What cost the most"))
                    .child(most),
            )
        });
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "usage", st, &self.act),
            page(&p, Some(("Usage", "November".to_string())), vec![], b),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Permissions ──

ops_el!(
    /// Agent / Permissions: Careful / Balanced / Hands-off with the counts they lead to, the apps
    /// with theirs, and the red lines; the work area holds one app's actions and rules.
    PermissionsEl
);

impl RenderOnce for PermissionsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let apps = perm_apps(st == ScreenState::Sample);
        let lv = if self.extra == "handsoff" {
            "handsoff"
        } else {
            "balanced"
        };
        let c = perm_counts(&apps, lv, &[]);
        let mut b = body(28.).gap(px(8.));
        if let Some(s) = sk(&p, st, 10) {
            b = b.child(s);
        } else if apps.is_empty() {
            b = b.child(none(
                &p,
                "No apps connected",
                "Connect an app and choose what the agent may do in it.",
            ));
        } else {
            let mut levels = div()
                .flex()
                .gap(px(2.))
                .p(px(3.))
                .rounded(px(10.))
                .bg(p.card2);
            for (k, l, d, _) in LEVELS {
                let on = k == lv;
                let f = fire(
                    &self.act,
                    if k == "careful" {
                        "level-careful"
                    } else if k == "balanced" {
                        "level-balanced"
                    } else {
                        "level-handsoff"
                    },
                );
                levels = levels.child(
                    div()
                        .id(k)
                        .flex_1()
                        .p(px(12.))
                        .rounded(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .when(on, |x| x.bg(p.card))
                        .text_color(if on { p.fg } else { p.mute })
                        .child(div().font_weight(FontWeight::MEDIUM).child(l))
                        .child(div().text_size(px(12.5)).child(d))
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            let tri = div()
                .flex()
                .gap(px(3.))
                .h(px(4.))
                .child(
                    div()
                        .w(px(c.0 as f32 * 20.))
                        .bg(tone(&p, IconTone::Running)),
                )
                .child(
                    div()
                        .w(px(c.1 as f32 * 20.))
                        .bg(tone(&p, IconTone::Approval)),
                )
                .child(div().w(px(c.2 as f32 * 20.)).bg(p.line));
            let mut al = card(&p);
            for (id, name, acct, _) in apps.iter().copied() {
                let ac = perm_counts(
                    &apps
                        .iter()
                        .copied()
                        .filter(|a| a.0 == id)
                        .collect::<Vec<_>>(),
                    lv,
                    &[],
                );
                al = al.child(
                    row(&p)
                        .when(id == "stripe", |d| d.bg(p.sel))
                        .child(logo(&p, id, 18.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(name)
                                .when(self.extra != "compact", |d| d.child(quiet(&p, acct))),
                        )
                        .child(quiet(&p, counts_words(ac))),
                );
            }
            let mut red = card(&p);
            for (l, app, r) in RED_LINES {
                red = red.child(
                    row(&p)
                        .child(logo(&p, app, 12.))
                        .child(clip(l))
                        .when(self.extra == "lockred", |d| d.child(quiet(&p, "Locked")))
                        .child(quiet(&p, r.label())),
                );
            }
            b = b
                .child(div().text_color(p.fg2).child(
                    "What your agent – and every agent it starts – may do without asking you first",
                ))
                .child(h3(&p, "How much it does on its own"))
                .child(
                    card(&p)
                        .p(px(6.))
                        .gap(px(8.))
                        .child(levels)
                        .child(div().px(px(8.)).text_color(p.fg2).child(format!(
                            "Does {} things on its own · asks before {} · never does {}",
                            c.0, c.1, c.2
                        )))
                        .child(div().px(px(8.)).pb(px(8.)).child(tri)),
                )
                .child(h3(&p, format!("Apps {}", apps.len())))
                .child(al)
                .child(h3(&p, "Red lines"))
                .child(red);
        }
        let wk = apps
            .iter()
            .find(|a| a.0 == "stripe")
            .map(|(id, name, acct, acts)| {
                let mut list = card(&p);
                for (l, k, _) in acts.iter() {
                    let r = rule_for(lv, k, l);
                    list = list.child(row(&p).child(clip(*l)).child(pill(
                        r.label(),
                        match r {
                            Rule::Own => tone(&p, IconTone::Running),
                            Rule::Ask => tone(&p, IconTone::Approval),
                            Rule::Never => p.mute,
                        },
                    )));
                }
                let mut lately = card(&p);
                for (t, w, tn) in LATELY {
                    lately = lately.child(
                        row(&p)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(clip(t))
                                    .child(quiet(&p, w)),
                            )
                            .child(pill(
                                if tn == "run" { "Approved" } else { "Done" },
                                st_tone(&p, tn),
                            )),
                    );
                }
                work(
                    &p,
                    *name,
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(12.))
                                .child(logo(&p, id, 22.))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(div().font_weight(FontWeight::SEMIBOLD).child(*name))
                                        .child(quiet(&p, *acct)),
                                ),
                        )
                        .child(h3(&p, "Actions"))
                        .child(list)
                        .child(h3(&p, "Lately"))
                        .child(lately),
                )
            });
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "permissions", st, &self.act),
            page(&p, Some(("Permissions", String::new())), vec![], b),
            wk,
        )
    }
}

// ───────────────────────────────────────────────────────────── Run Detail ──

ops_el!(
    /// Agent / Run Detail: one run on one time axis (thinking, tool calls, sub-agents, failures)
    /// with time, cost, tokens and steps; `stopped` ends it at the failure.
    RunDetailEl
);

impl RenderOnce for RunDetailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = spans(st == ScreenState::Sample);
        let stopped = self.extra == "stopped";
        let (t, cost, tk, n, f) = run_totals(&all, stopped);
        let kc = |k: SpanKind| match k {
            SpanKind::Think => tone(&p, IconTone::Skills),
            SpanKind::Tool => tone(&p, IconTone::Running),
            SpanKind::Sub => p.mute,
            SpanKind::Fail => tone(&p, IconTone::Failed),
        };
        let mut b = body(20.).gap(px(4.));
        if let Some(s) = sk(&p, st, 12) {
            b = b.child(s);
        } else if all.is_empty() {
            b = b.child(none(
                &p,
                "No run to show",
                "Start a task and its run shows here step by step.",
            ));
        } else {
            let mut tiles = div().flex().gap(px(10.)).mb(px(12.));
            for (l, v, bad) in [
                ("Time", secs(t), false),
                ("Cost", eur(cost), false),
                ("Tokens", toks(tk).to_uppercase(), false),
                ("Steps", format!("{n} · {f} failed"), f > 0),
            ] {
                tiles = tiles.child(
                    card(&p)
                        .flex_1()
                        .p(px(12.))
                        .gap(px(6.))
                        .child(quiet(&p, l))
                        .child(
                            div()
                                .text_size(px(16.))
                                .when(bad, |d| d.text_color(tone(&p, IconTone::Failed)))
                                .child(v),
                        ),
                );
            }
            b = b
                .child(quiet(&p, "Builder · Opus 5.5 · started 07:22").mb(px(10.)))
                .child(tiles);
            let tw = 480.;
            let mut ax = div().flex().h(px(22.)).child(div().w(px(260.)).flex_none());
            let mut tk_row = div().relative().w(px(tw)).h_full();
            for (l, x) in ticks() {
                tk_row = tk_row.child(
                    div()
                        .absolute()
                        .left(px(tw * x as f32 / 100.))
                        .child(quiet(&p, l)),
                );
            }
            ax = ax.child(tk_row);
            b = b.child(ax);
            for s in span_view(&all, self.extra == "errors", stopped) {
                let (x, w) = bar(&s);
                let c = kc(s.kind);
                let lab = if s.kind == SpanKind::Fail {
                    "failed".to_string()
                } else if self.extra == "cost" {
                    eur(s.cost)
                } else {
                    secs(s.dur)
                };
                let mut lead = div()
                    .w(px(260.))
                    .flex_none()
                    .pl(px(10. + 18. * s.depth as f32))
                    .flex()
                    .items_center()
                    .gap(px(9.));
                lead = match brand_mark(s.app, 13., p.fg) {
                    Some(m) if !s.app.is_empty() => lead.child(m),
                    _ => lead.child(
                        div()
                            .w(px(10.))
                            .h(px(10.))
                            .rounded(px(3.))
                            .bg(c.opacity(0.5)),
                    ),
                };
                b = b.child(
                    div()
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .rounded(px(8.))
                        .when(s.id == if stopped { 9 } else { 4 }, |d| d.bg(p.sel))
                        .child(lead.child(clip(s.label)))
                        .child(
                            div()
                                .relative()
                                .w(px(tw))
                                .h_full()
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(10.))
                                        .left(px(tw * x / 100.))
                                        .w(px(tw * w / 100.))
                                        .h(px(14.))
                                        .rounded(px(3.))
                                        .border_1()
                                        .border_color(c.opacity(0.55))
                                        .bg(c.opacity(0.18)),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(9.))
                                        .left(px(tw * (x + w) / 100. + 6.))
                                        .text_size(px(12.))
                                        .text_color(if s.kind == SpanKind::Fail {
                                            c
                                        } else {
                                            p.mute
                                        })
                                        .child(lab),
                                )
                                .when(!stopped, |d| {
                                    d.child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .bottom_0()
                                            .left(px(tw * RUN_NOW as f32 / 700.))
                                            .w(px(1.5))
                                            .bg(p.fg2),
                                    )
                                }),
                        ),
                );
            }
        }
        let sel = if stopped { 9 } else { 4 };
        let wk = all.iter().find(|s| s.id == sel).map(|s| {
            let (i, o) = span_io(s.id);
            let pre = |t: &'static str, bad: bool| {
                div()
                    .p(px(12.))
                    .rounded(px(10.))
                    .bg(p.card2)
                    .font_family("monospace")
                    .text_size(px(12.5))
                    .text_color(if bad {
                        tone(&p, IconTone::Failed)
                    } else {
                        p.fg2
                    })
                    .child(t)
            };
            work(
                &p,
                s.label,
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(clip(s.label).font_weight(FontWeight::MEDIUM))
                            .child(if s.kind == SpanKind::Fail {
                                pill("Failed", tone(&p, IconTone::Failed))
                            } else {
                                pill("Done", tone(&p, IconTone::Done))
                            }),
                    )
                    .child(quiet(&p, s.kind.label()))
                    .child(
                        div()
                            .mt(px(12.))
                            .child(kv(&p, "Started", div().child(format!("+{}", secs(s.at)))))
                            .child(kv(&p, "Took", div().child(secs(s.dur))))
                            .child(kv(
                                &p,
                                "Tokens",
                                div().child(format!("{} in · {} out", s.toks.0, s.toks.1)),
                            ))
                            .child(kv(&p, "Cost", div().child(eur(s.cost)))),
                    )
                    .child(h3(&p, "Input"))
                    .child(pre(i, false))
                    .child(h3(&p, "Output"))
                    .child(pre(o, s.kind == SpanKind::Fail)),
            )
        });
        let sf = fire(&self.act, "stop");
        let mut acts = Vec::new();
        if !all.is_empty() {
            acts.push(pill(
                if stopped { "Stopped" } else { "Running" },
                tone(
                    &p,
                    if stopped {
                        IconTone::Failed
                    } else {
                        IconTone::Running
                    },
                ),
            ));
            acts.push(
                div().child(
                    btn(&p, if stopped { "Resume" } else { "Stop" }, false)
                        .id("stop")
                        .on_click(move |_, w, cx| sf(w, cx)),
                ),
            );
        }
        frame(
            self.theme,
            self.layout,
            side(&p, persona("fieldwork"), "agents", st, &self.act),
            page(
                &p,
                Some(("Runs", "Migrate users table".to_string())),
                acts,
                b,
            ),
            wk,
        )
    }
}
