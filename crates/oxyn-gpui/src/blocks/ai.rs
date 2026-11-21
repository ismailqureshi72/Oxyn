//! The AI blocks of [`BlocksView`]: Blind compare, Prompt bench, Assistant memory, Catch up, AI
//! feature launch, AI setup wizard, Call insights and Lead scoring. Same models and press ids as
//! the Dioxus blocks.

use gpui::{Context, Div, FontWeight, ParentElement, Styled, div, prelude::*, px, relative};
use oxyn_core::blocks::{
    ago, arena, call, catchup, launch, leads, memory, prompt, thousands, wizard,
};
use oxyn_tokens::ColorRole;

use super::BlocksView;
use crate::aikit::{bar, c, hue, mark, pill, row, text, title};

fn col() -> Div {
    div().flex().flex_col().gap(px(8.)).min_w(px(0.))
}

impl BlocksView {
    pub(super) fn arena(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let a = &self.state.arena;
        let pair = a.pair();
        let rev = a.revealed();
        let fg = c(t, ColorRole::FgDefault);
        let mut samples = row().flex_wrap();
        for (k, (label, _)) in arena::SAMPLES.iter().enumerate() {
            samples =
                samples.child(self.chip(cx, &format!("ar-sample-{k}"), *label, a.sample == k));
        }
        let mut panes = row().items_start().gap(px(10.));
        for (s, &mi) in pair.iter().enumerate() {
            let st = a.side(s, n);
            let m = arena::MODELS[mi];
            let letter = if s == 0 { "A" } else { "B" };
            let mut head = row();
            if rev {
                head = head
                    .child(mark(m.1, 14., fg))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(m.0));
            } else {
                head = head.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("Model {letter}")),
                );
            }
            head = head.child(div().flex_1());
            if a.first(n) == Some(s) && st == arena::Side::Done {
                head = head.child(pill(t, "Finished first", "green"));
            }
            let h = match st {
                arena::Side::Done => "green",
                arena::Side::Failed => "red",
                arena::Side::Thinking | arena::Side::Writing => "blue",
                _ => "neutral",
            };
            head = head.child(self.status(st.word(), h));
            let mut pane = self.card().flex_1().child(head);
            if st == arena::Side::Failed {
                pane = pane.child(
                    row()
                        .child(text(
                            t,
                            "The model did not respond.",
                            ColorRole::StatusDangerFg,
                            12.5,
                        ))
                        .child(self.btn(cx, "ar-retry", "Retry", false)),
                );
            } else {
                pane = pane.child(div().h(px(150.)).text_size(px(12.5)).child(a.text(s, n)));
            }
            if let Some((ttft, tps, cost)) = a.stats(s, n) {
                pane = pane.child(text(
                    t,
                    format!("First token {ttft} · {tps} · {cost}"),
                    ColorRole::FgSubtle,
                    11.5,
                ));
            }
            panes = panes.child(pane);
        }
        let mut out = col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(
                        t,
                        "Blind compare",
                        format!("Round {} · two models, one prompt", a.round + 1),
                    ))
                    .child(
                        row()
                            .child(text(t, "Blind", ColorRole::FgMuted, 12.))
                            .child(self.sw(cx, "ar-blind", a.blind))
                            .child(self.btn(cx, "ar-board", "Leaderboard", false)),
                    ),
            )
            .child(
                row()
                    .child(
                        div()
                            .flex_1()
                            .px(px(10.))
                            .py(px(6.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .child(arena::SAMPLES[a.sample].1),
                    )
                    .child(self.btn(
                        cx,
                        "ar-run",
                        if a.running(n) { "Stop" } else { "Run" },
                        !a.running(n),
                    )),
            )
            .child(samples);
        if a.board_open {
            let mut lb = self.card();
            for (k, (m, wr, g)) in a.leaderboard().into_iter().enumerate() {
                lb = lb.child(
                    row()
                        .child(div().w(px(16.)).child(format!("{}", k + 1)))
                        .child(mark(arena::MODELS[m].1, 13., fg))
                        .child(div().flex_1().child(arena::MODELS[m].0))
                        .child(div().w(px(80.)).child(bar(t, wr, fg)))
                        .child(text(
                            t,
                            format!("{:.0}% · {g}", wr * 100.0),
                            ColorRole::FgMuted,
                            12.,
                        )),
                );
            }
            out = out.child(lb);
        }
        out = out.child(panes);
        if let Some(v) = a.vote {
            out = out.child(
                row()
                    .child(text(
                        t,
                        format!(
                            "{}. A: {} · B: {}",
                            v.words(),
                            arena::MODELS[pair[0]].0,
                            arena::MODELS[pair[1]].0
                        ),
                        ColorRole::FgDefault,
                        12.5,
                    ))
                    .child(div().flex_1())
                    .child(self.btn(cx, "ar-next", "Next round", true)),
            );
        } else if a.can_vote(n) {
            let mut votes = row();
            for (id, label) in [
                ("a", "A is better"),
                ("tie", "Tie"),
                ("b", "B is better"),
                ("bad", "Both bad"),
            ] {
                votes = votes.child(self.btn(cx, &format!("ar-vote-{id}"), label, false));
            }
            out = out.child(votes);
        }
        out
    }

    pub(super) fn prompt(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.prompt;
        let fg = c(t, ColorRole::FgDefault);
        let mut versions = row().flex_wrap();
        for (i, v) in prompt::VERSIONS.iter().enumerate() {
            let label = if i == s.prod {
                format!("{} · Production", v.0)
            } else {
                v.0.to_string()
            };
            versions = versions.child(self.chip(cx, &format!("ps-ver-{i}"), label, i == s.version));
        }
        let mut models = row().flex_wrap();
        for (i, m) in prompt::MODELS.iter().enumerate() {
            models = models.child(
                self.chip(cx, &format!("ps-model-{i}"), m.0, i == s.model)
                    .child(mark(m.1, 12., fg)),
            );
        }
        let mut tpl = row().flex_wrap().gap(px(3.));
        if s.compare {
            for (op, w) in prompt::diff(s.template(s.prod), s.template(s.version)) {
                let (role, pre) = match op {
                    1 => (ColorRole::StatusSuccessFg, "+"),
                    -1 => (ColorRole::StatusDangerFg, "−"),
                    _ => (ColorRole::FgDefault, ""),
                };
                tpl = tpl.child(text(t, format!("{pre}{w}"), role, 12.5));
            }
        } else {
            for p in prompt::parts(s.template(s.version)) {
                tpl = tpl.child(match p {
                    prompt::Part::Text(x) => text(t, x, ColorRole::FgDefault, 12.5),
                    prompt::Part::Var(i) => div()
                        .px(px(5.))
                        .rounded(px(5.))
                        .border_1()
                        .border_color(hue(t, prompt::VARS[i].1))
                        .text_color(hue(t, prompt::VARS[i].1))
                        .text_size(px(12.5))
                        .child(format!("{{{{{}}}}}", prompt::VARS[i].0)),
                });
            }
        }
        let mut fields = row().flex_wrap().items_start().gap(px(8.));
        for i in s.used() {
            fields = fields.child(
                col()
                    .gap(px(2.))
                    .child(
                        text(t, prompt::VARS[i].0, ColorRole::FgMuted, 11.5)
                            .text_color(hue(t, prompt::VARS[i].1)),
                    )
                    .child(text(t, s.values[i].clone(), ColorRole::FgDefault, 12.5)),
            );
        }
        let mut cases = row();
        for (i, k) in prompt::CASES.iter().enumerate() {
            cases = cases.child(self.chip(cx, &format!("ps-case-{i}"), k.0, i == s.case));
        }
        let running = s.running(n);
        let mut out = col()
            .gap(px(10.))
            .child(title(
                t,
                "Support reply",
                format!(
                    "Prompt template · production serves {}",
                    prompt::VERSIONS[s.prod].0
                ),
            ))
            .child(versions)
            .child(models)
            .child(
                row()
                    .child(self.chip(cx, "ps-tab-0", "System", s.tab == 0))
                    .child(self.chip(cx, "ps-tab-1", "User", s.tab == 1))
                    .child(div().flex_1())
                    .child(self.chip(cx, "ps-compare", "Compare", s.compare))
                    .child(self.btn(cx, "ps-promote", "Promote", false)),
            )
            .child(self.card().child(tpl))
            .child(fields)
            .child(row().child(cases).child(div().flex_1()).child(self.btn(
                cx,
                "ps-run",
                if running { "Stop" } else { "Run" },
                !running,
            )));
        if s.undo.is_some() {
            out = out.child(
                row()
                    .child(text(t, "Promoted to production.", ColorRole::FgMuted, 12.))
                    .child(self.btn(cx, "ps-undo", "Undo", false)),
            );
        }
        if s.failed {
            out = out.child(
                row()
                    .child(text(
                        t,
                        "Run failed: the model returned an error.",
                        ColorRole::StatusDangerFg,
                        12.5,
                    ))
                    .child(self.btn(cx, "ps-retry", "Retry", false)),
            );
        } else if s.run.is_some() {
            let mut card = self.card().child(div().child(s.reply(n)));
            if let Some((lat, toks, cost)) = s.stats(n) {
                card = card.child(text(
                    t,
                    format!("{lat} · {toks} tokens · {cost}"),
                    ColorRole::FgSubtle,
                    11.5,
                ));
            }
            out = out.child(card);
        }
        out
    }

    pub(super) fn memory(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let m = &self.state.memory;
        let counts = m.counts();
        let mut pills = row().flex_wrap();
        for (i, cnt) in counts.iter().enumerate() {
            let label = if i == 0 {
                "All".to_string()
            } else {
                memory::GROUPS[i - 1].to_string()
            };
            pills = pills.child(self.chip(
                cx,
                &format!("mm-f-{i}"),
                format!("{label} {cnt}"),
                m.filter == i,
            ));
        }
        let mut list = col();
        for (g, items) in m.shown() {
            list = list.child(text(
                t,
                format!("{} · {}", memory::GROUPS[g], items.len()),
                ColorRole::FgSubtle,
                11.5,
            ));
            for it in items {
                let glow = m.glow(it, n);
                list = list.child(
                    row()
                        .px(px(6.))
                        .py(px(4.))
                        .rounded(px(6.))
                        .when(glow, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                        .child(div().flex_1().min_w(px(0.)).child(it.text.clone()))
                        .child(text(t, it.source.clone(), ColorRole::FgSubtle, 11.))
                        .child(self.btn(cx, &format!("mm-forget-{}", it.id), "Forget", false)),
                );
            }
        }
        let mut out = col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "Memory", "What the assistant remembers about you"))
                    .child(
                        col()
                            .gap(px(3.))
                            .w(px(110.))
                            .child(text(
                                t,
                                format!("{:.0}% full", m.usage() * 100.0),
                                ColorRole::FgMuted,
                                12.,
                            ))
                            .child(bar(
                                t,
                                m.usage(),
                                if m.warn() {
                                    hue(t, "amber")
                                } else {
                                    c(t, ColorRole::FgMuted)
                                },
                            )),
                    ),
            )
            .child(
                row()
                    .gap(px(16.))
                    .child(
                        row()
                            .child(self.sw(cx, "mm-saved", m.saved))
                            .child("Saved memories"),
                    )
                    .child(
                        row()
                            .child(self.sw(cx, "mm-history", m.history))
                            .child("Chat history"),
                    ),
            );
        if m.pending {
            out = out.child(self.btn(
                cx,
                "mm-file",
                format!("Memory updated: {}", memory::CAPTURE.0),
                false,
            ));
        }
        out = out
            .child(pills)
            .child(list.when(!m.saved, |d| d.opacity(0.6)));
        if m.can_undo(n) {
            out = out.child(
                row()
                    .child(text(t, "Forgotten.", ColorRole::FgMuted, 12.))
                    .child(self.btn(cx, "mm-undo", format!("Undo · {}s", m.undo_left(n)), false)),
            );
        }
        out.child(
            row()
                .child(self.btn(cx, "mm-add", "Add “Prefers meetings after 10 am”", false))
                .when_some(m.added.clone(), |d, (ok, w)| {
                    d.child(self.status(w, if ok { "green" } else { "red" }))
                }),
        )
    }

    pub(super) fn catchup(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.catchup;
        let fg = c(t, ColorRole::FgDefault);
        let mut ranges = row();
        for (i, r) in catchup::RANGES.iter().enumerate() {
            ranges = ranges.child(self.chip(cx, &format!("cu-range-{i}"), *r, s.range == i));
        }
        let mut srcs = row().flex_wrap();
        for (i, src) in catchup::SOURCES.iter().enumerate() {
            let st = s.source(i, n);
            let mut chip =
                row()
                    .child(mark(src.1, 13., fg))
                    .child(text(t, src.0, ColorRole::FgMuted, 12.));
            chip = match st {
                1 => chip.child(text(t, "reading", ColorRole::FgSubtle, 11.5)),
                2 => chip.child(self.status(thousands(u64::from(src.2[s.range])), "green")),
                3 => chip.child(self.status("Skipped", "red")),
                _ => chip,
            };
            srcs = srcs.child(chip.when(st == 0, |d| d.opacity(0.55)));
        }
        let mut out = col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "Catch up", "What you missed, ranked"))
                    .child(ranges),
            )
            .child(srcs);
        if s.caught_up(n) {
            out = out.child(self.status("All caught up", "green"));
        } else {
            let left = s.left(n);
            for (si, sec) in catchup::SECTIONS.iter().enumerate() {
                let rows: Vec<usize> = left
                    .iter()
                    .copied()
                    .filter(|&i| catchup::ITEMS[i].0 == si)
                    .collect();
                if rows.is_empty() {
                    continue;
                }
                out = out.child(self.status(format!("{} · {}", sec.0, rows.len()), sec.2));
                for i in rows {
                    let it = catchup::ITEMS[i];
                    out =
                        out.child(
                            row()
                                .items_start()
                                .child(mark(catchup::SOURCES[it.1].1, 14., fg))
                                .child(col().gap(px(1.)).flex_1().child(div().child(it.2)).child(
                                    text(
                                        t,
                                        format!("{} · {}", it.3, it.4),
                                        ColorRole::FgSubtle,
                                        11.5,
                                    ),
                                ))
                                .child(self.btn(cx, &format!("cu-read-{i}"), "Read", false)),
                        );
                }
            }
        }
        let busy = s.busy(n);
        out.child(
            row()
                .child(self.btn(
                    cx,
                    "cu-run",
                    if busy {
                        "Stop"
                    } else if s.run.is_some() {
                        "Refresh"
                    } else {
                        "Catch me up"
                    },
                    s.run.is_none(),
                ))
                .when(!busy && !s.left(n).is_empty(), |d| {
                    d.child(self.btn(cx, "cu-all", "Mark all as read", false))
                }),
        )
    }

    pub(super) fn launch(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let l = &self.state.launch;
        let step = l.step(n);
        let st = launch::STEPS[step];
        let (typed, lines) = l.demo(n);
        let mut demo = self.card().h(px(110.)).justify_center().child(text(
            t,
            st.2[0].chars().take(typed).collect::<String>(),
            ColorRole::FgDefault,
            12.5,
        ));
        if lines >= 1 {
            demo = demo.child(text(t, st.2[1], ColorRole::FgMuted, 12.5));
        }
        if lines >= 2 {
            demo = demo.child(text(t, st.2[2], ColorRole::StatusSuccessFg, 11.5));
        }
        let mut bens = col().gap(px(4.));
        for b in st.3 {
            bens = bens.child(
                row()
                    .child(div().size(px(6.)).rounded(px(3.)).bg(hue(t, "green")))
                    .child(b),
            );
        }
        let mut dots = row();
        for i in 0..launch::STEPS.len() {
            dots = dots.child(self.hit(cx, &format!("fl-dot-{i}"), false).p(px(4.)).child(
                div().w(px(if i == step { 36. } else { 20. })).child(bar(
                    t,
                    if i == step { l.progress(n) } else { 0.0 },
                    c(t, ColorRole::FgDefault),
                )),
            ));
        }
        let note = if !l.admin {
            "Only admins can turn this on."
        } else if l.is_saving(n) {
            "Saving… press again to cancel"
        } else if l.on() {
            "On for everyone in Northwind"
        } else {
            "Off · members can't use it yet"
        };
        col()
            .gap(px(10.))
            .child(demo)
            .child(row().child(pill(t, "New", "violet")).child(text(
                t,
                format!("Step {} of 3", step + 1),
                ColorRole::FgSubtle,
                12.,
            )))
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(st.0),
            )
            .child(text(t, st.1, ColorRole::FgMuted, 13.))
            .child(bens)
            .child(dots)
            .child(
                self.card()
                    .child(row().child(self.sw(cx, "fl-switch", l.on())).child(title(
                        t,
                        "Enable for my workspace",
                        note,
                    ))),
            )
            .child(
                row()
                    .justify_end()
                    .child(self.btn(cx, "fl-learn", "Learn more", false))
                    .child(self.btn(cx, "fl-try", "Try it now", true)),
            )
    }

    pub(super) fn wizard(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let w = &self.state.wizard;
        let fg = c(t, ColorRole::FgDefault);
        let mut steps = row();
        for (i, s) in wizard::STEPS.iter().enumerate() {
            steps = steps.child(self.chip(
                cx,
                &format!("wz-step-{i}"),
                format!("{} {s}", i + 1),
                i == w.step && !w.live,
            ));
        }
        let mut view = col();
        if w.live {
            view = view
                .child(self.status("Ask Northwind is live", "green"))
                .child(text(
                    t,
                    format!(
                        "{} sources · {} · {} tone",
                        w.ready(n).len(),
                        wizard::MODELS[w.model].0,
                        wizard::TONES[w.tone]
                    ),
                    ColorRole::FgMuted,
                    12.5,
                ));
        } else if w.step == 0 {
            for (i, s) in wizard::SOURCES.iter().enumerate() {
                let (cnt, done) = w.synced(i, n);
                let mut r = row()
                    .child(self.logo_tile(s.1, 16.))
                    .child(div().flex_1().font_weight(FontWeight::SEMIBOLD).child(s.0));
                r = match w.src[i] {
                    wizard::Src::Off => {
                        r.child(self.btn(cx, &format!("wz-connect-{i}"), "Connect", true))
                    }
                    wizard::Src::Failed => r
                        .child(self.status("Token expired", "red"))
                        .child(self.btn(cx, &format!("wz-retry-{i}"), "Retry", false)),
                    wizard::Src::Syncing(_) => r
                        .child(div().w(px(120.)).child(bar(
                            t,
                            f64::from(cnt) / f64::from(s.2),
                            hue(t, if done { "green" } else { "blue" }),
                        )))
                        .child(self.status(
                            if done {
                                format!("Synced {}", thousands(u64::from(cnt)))
                            } else {
                                format!("Syncing {}", thousands(u64::from(cnt)))
                            },
                            if done { "green" } else { "blue" },
                        )),
                };
                view = view.child(self.card().child(r));
            }
        } else if w.step == 1 {
            let mut models = row().items_start();
            for (i, m) in wizard::MODELS.iter().enumerate() {
                models = models.child(
                    self.hit(cx, &format!("wz-model-{i}"), i == w.model)
                        .flex_1()
                        .border_1()
                        .border_color(if i == w.model {
                            fg
                        } else {
                            c(t, ColorRole::BorderSubtle)
                        })
                        .child(mark(m.1, 16., fg))
                        .child(title(t, m.0, format!("{} · {}", m.2, m.3))),
                );
            }
            let mut tones = row();
            for (i, x) in wizard::TONES.iter().enumerate() {
                tones = tones.child(self.chip(cx, &format!("wz-tone-{i}"), *x, i == w.tone));
            }
            view = view.child(models).child(tones);
            for (i, g) in wizard::GUARDS.iter().enumerate() {
                view = view.child(
                    row()
                        .child(self.sw(cx, &format!("wz-guard-{i}"), w.guards[i]))
                        .child(title(t, g.0, g.1)),
                );
            }
        } else {
            let mut qs = row().flex_wrap();
            for (i, q) in wizard::QUESTIONS.iter().enumerate() {
                qs = qs.child(self.chip(
                    cx,
                    &format!("wz-ask-{i}"),
                    q.0,
                    w.ask.map(|a| a.0) == Some(i),
                ));
            }
            view = view.child(qs);
            if w.ask.is_some() {
                let txt: Vec<String> = w
                    .answer(n)
                    .into_iter()
                    .map(|(s, c)| match c {
                        Some(k) => format!("{s} [{k}]"),
                        None => s,
                    })
                    .collect();
                let mut card = self.card().child(div().child(txt.join(" ")));
                if !w.streaming(n) {
                    for (k, s) in w.cited().into_iter().enumerate() {
                        card = card.child(
                            row()
                                .child(text(t, format!("{}", k + 1), ColorRole::FgSubtle, 11.5))
                                .child(mark(wizard::SOURCES[s].1, 12., fg))
                                .child(text(t, wizard::SOURCES[s].0, ColorRole::FgMuted, 12.)),
                        );
                    }
                }
                view = view.child(card);
            }
        }
        let foot = if w.live {
            row()
                .justify_end()
                .child(self.btn(cx, "wz-edit", "Edit setup", false))
        } else {
            row()
                .justify_end()
                .child(self.btn(cx, "wz-back", "Back", false))
                .child(self.btn(
                    cx,
                    "wz-next",
                    if w.step == 2 { "Go live" } else { "Continue" },
                    w.can_next(n),
                ))
        };
        col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "Ask Northwind", "Support assistant setup"))
                    .child(pill(
                        t,
                        if w.live { "Live" } else { "Draft" },
                        if w.live { "green" } else { "neutral" },
                    )),
            )
            .child(steps)
            .child(view.min_h(px(240.)))
            .child(foot)
    }

    pub(super) fn call(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let s = &self.state.call;
        let fg = c(t, ColorRole::FgDefault);
        let sp_col = |k: usize| {
            let base = hue(
                t,
                if call::SPEAKERS[k].2 == 0 {
                    "blue"
                } else {
                    "teal"
                },
            );
            if k % 2 == 1 { base.opacity(0.55) } else { base }
        };
        let mut tbar = row().gap(px(2.)).h(px(10.)).rounded(px(5.));
        let mut legend = row().flex_wrap().gap(px(10.));
        for (k, sp) in call::SPEAKERS.iter().enumerate() {
            tbar = tbar.child(div().h_full().w(relative(sp.3 as f32 / 100.)).bg(sp_col(k)));
            legend = legend.child(
                row()
                    .child(div().size(px(8.)).rounded(px(2.)).bg(sp_col(k)))
                    .child(text(
                        t,
                        format!("{} {}%", sp.0, sp.3),
                        ColorRole::FgMuted,
                        12.,
                    )),
            );
        }
        let cur = s.current(n);
        let mo = call::MOMENTS[cur];
        let mut rail = row();
        for (k, m) in call::MOMENTS.iter().enumerate() {
            let kind = call::KINDS[m.1];
            rail = rail.child(self.chip(
                cx,
                &format!("ci-m-{k}"),
                format!("{} {}:{:02}", kind.0, m.0 / 60, m.0 % 60),
                k == cur,
            ));
        }
        let (pos, playing) = s.position(n);
        let (risk, _) = s.risk();
        let mut steps = col().gap(px(4.));
        for (k, st) in call::STEPS.iter().enumerate() {
            steps = steps.child(
                self.hit(cx, &format!("ci-step-{k}"), false)
                    .child(
                        div()
                            .size(px(14.))
                            .rounded(px(3.))
                            .border_1()
                            .border_color(fg)
                            .when(s.done[k], |d| d.bg(fg)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .when(s.done[k], |d| d.text_color(c(t, ColorRole::FgSubtle)))
                            .child(st.0),
                    )
                    .child(text(
                        t,
                        format!("{} · {}", st.1, st.2),
                        ColorRole::FgSubtle,
                        11.5,
                    )),
            );
        }
        col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(row().child(mark(call::CALL.2, 16., fg)).child(title(
                        t,
                        &format!("{} · {}", call::CALL.0, call::CALL.1),
                        format!("Zoom · {} · {} min", call::CALL.3, call::CALL.4 / 60),
                    )))
                    .child(pill(t, call::RISK[risk].0, call::RISK[risk].1)),
            )
            .child(text(t, "Talk time", ColorRole::FgSubtle, 11.5))
            .child(tbar)
            .child(legend)
            .child(text(t, "Key moments", ColorRole::FgSubtle, 11.5))
            .child(rail.flex_wrap())
            .child(
                self.card()
                    .child(
                        row()
                            .child(self.status(call::KINDS[mo.1].0, call::KINDS[mo.1].2))
                            .child(text(
                                t,
                                format!("{} · {}", call::SPEAKERS[mo.2].0, s.stamp(n)),
                                ColorRole::FgMuted,
                                12.,
                            )),
                    )
                    .child(div().child(format!("“{}”", mo.3)))
                    .child(
                        row()
                            .child(self.btn(
                                cx,
                                "ci-play",
                                if playing { "Pause" } else { "Play" },
                                false,
                            ))
                            .child(div().flex_1().child(bar(
                                t,
                                f64::from(pos) / f64::from(call::CLIP_S),
                                fg,
                            )))
                            .child(text(
                                t,
                                format!("0:{pos:02} / 0:{:02}", call::CLIP_S),
                                ColorRole::FgSubtle,
                                11.5,
                            )),
                    ),
            )
            .child(
                row()
                    .justify_between()
                    .child(text(t, "AI summary", ColorRole::FgSubtle, 11.5))
                    .child(self.btn(
                        cx,
                        "ci-copy",
                        if s.copied { "Copied" } else { "Copy" },
                        false,
                    )),
            )
            .child(div().child(s.summary(n)))
            .child(text(t, "Next steps", ColorRole::FgSubtle, 11.5))
            .child(steps)
    }

    pub(super) fn leads(&self, cx: &mut Context<Self>) -> Div {
        let t = self.theme();
        let n = self.state.now;
        let l = &self.state.leads;
        let fg = c(t, ColorRole::FgDefault);
        let mut sorts = row();
        for (i, s) in leads::SORTS.iter().enumerate() {
            sorts = sorts.child(self.chip(cx, &format!("ld-sort-{i}"), *s, l.sort == i));
        }
        let mut list = col().gap(px(2.));
        for (pos, i) in l.order().into_iter().enumerate() {
            let lead = leads::LEADS[i];
            let (tw, th) = leads::tier(l.scores[i]);
            let open = l.open == Some(i);
            let mut item = col().gap(px(6.)).child(
                self.hit(cx, &format!("ld-open-{i}"), open || pos == l.focus)
                    .child(
                        div()
                            .size(px(30.))
                            .rounded(px(7.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(c(t, ColorRole::SurfaceSelected))
                            .child(lead.0.chars().next().unwrap_or('?').to_string()),
                    )
                    .child(
                        col()
                            .gap(px(1.))
                            .flex_1()
                            .child(
                                row()
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(lead.0))
                                    .child(text(
                                        t,
                                        format!("{} · {}", lead.1, lead.2),
                                        ColorRole::FgSubtle,
                                        12.,
                                    )),
                            )
                            .child(text(t, lead.4, ColorRole::FgMuted, 12.)),
                    )
                    .child(text(
                        t,
                        ago(u64::from(lead.5) * 60_000),
                        ColorRole::FgSubtle,
                        11.5,
                    ))
                    .child(
                        text(t, tw, ColorRole::FgMuted, 11.5).text_color(if th == "neutral" {
                            c(t, ColorRole::FgSubtle)
                        } else {
                            hue(t, th)
                        }),
                    )
                    .child(
                        div()
                            .w(px(44.))
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("{}", l.shown(i, n))),
                            )
                            .child(bar(
                                t,
                                f64::from(l.shown(i, n)) / 100.0,
                                if th == "neutral" {
                                    c(t, ColorRole::FgSubtle)
                                } else {
                                    hue(t, th)
                                },
                            )),
                    ),
            );
            if open {
                let mut chips = row().flex_wrap();
                for sg in lead.6 {
                    chips = chips.child(pill(t, *sg, "neutral"));
                }
                let mut bd = col().gap(px(4.));
                for f in lead.7 {
                    bd = bd.child(
                        row()
                            .child(div().w(px(110.)).child(f.0))
                            .child(div().flex_1().child(bar(
                                t,
                                f64::from(f.1) / f64::from(f.2),
                                fg,
                            )))
                            .child(text(
                                t,
                                format!("{} / {}", f.1, f.2),
                                ColorRole::FgMuted,
                                12.,
                            )),
                    );
                }
                let mut dests = row();
                for (k, d) in leads::DESTS.iter().enumerate() {
                    dests = dests.child(
                        self.chip(cx, &format!("ld-dest-{k}"), d.0, l.dest == k)
                            .child(mark(d.1, 12., fg)),
                    );
                }
                let action = if let Some(d) = l.pushed(i, n) {
                    self.status(format!("In {}", leads::DESTS[d].0), "green")
                } else if l.pushing(i, n) {
                    text(t, "Pushing…", ColorRole::FgMuted, 12.)
                } else if let Some(leads::Push::Failed(_)) = l.push[i] {
                    row()
                        .child(self.status("Couldn't reach the CRM", "red"))
                        .child(self.btn(cx, "ld-retry", "Retry", false))
                } else {
                    row().child(self.btn(
                        cx,
                        "ld-push",
                        format!("Push to {}", leads::DESTS[l.dest].0),
                        true,
                    ))
                };
                item = item.child(
                    col()
                        .pl(px(46.))
                        .child(chips)
                        .child(bd)
                        .child(row().justify_between().child(dests).child(action)),
                );
            }
            list = list.child(item);
        }
        col()
            .gap(px(10.))
            .child(
                row()
                    .justify_between()
                    .child(title(t, "Inbound leads", "Ranked by AI fit score"))
                    .child(sorts.child(self.btn(cx, "ld-rescore", "Rescore", false))),
            )
            .child(list)
    }
}
