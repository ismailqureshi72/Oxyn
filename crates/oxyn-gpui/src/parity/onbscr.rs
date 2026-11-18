//! Screens S8 in GPUI: the onboarding wizard (Website, Brand, Tone, Ready) with no app shell: a
//! 56 px top bar (Back, the signed-in address, Log out), a 720 px form column and the assistant's
//! dashboard as a preview that shows the part each step changes. Static snapshots of one state
//! each, like the other screen sets: `.state()` for Data / Loading / Empty and `.extra()` for the
//! step's own state (`fetching`, `custom`, `testing`, `live`) or the Oxyn options `named` and
//! `form-only`. Presses fire `back`, `logout`, `add`, `continue`, `edit-<step>` and
//! `open-dashboard` through `on_act`. Rules come from `oxyn_core::parity::onbscr`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, rgb,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::onbscr::{
    Domain, LENGTHS, MARKS, OWN_DOMAIN, PERSON_EMAIL, PRODUCT, Phase, STEPS, SWATCHES, ScreenState,
    TEST_Q, TONES, Wiz, answer, company, first_gap, greeting, heading, initials, ink_on,
    pages_total, sources_words, summary, wiz,
};
use oxyn_core::parity::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::skel;
use super::setscr::{input, sw};
use super::workscr::{btn, quiet, tone};

macro_rules! s8_el {
    ($(#[$m:meta])* $name:ident, $step:literal) => {
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
            /// The step's own state, `named` or `form-only` (see the module docs).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat: the dashboard picture as a rounded panel or edge to edge.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`back`, `continue`, `open-dashboard`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                screen(self.theme, self.layout, self.state, self.extra, &self.act, $step)
            }
        }
    };
}

s8_el!(
    /// Onboarding / Website: the sites the assistant learns from, read with progress.
    OnboardingWebsiteEl,
    0
);
s8_el!(
    /// Onboarding / Brand: company name and short forms, brand colour and mark.
    OnboardingBrandEl,
    1
);
s8_el!(
    /// Onboarding / Tone: name, tone, reply length and handover, with a test chat.
    OnboardingToneEl,
    2
);
s8_el!(
    /// Onboarding / Ready: the summary with Edit links and Open dashboard.
    OnboardingReadyEl,
    3
);

fn hex(c: &str) -> Hsla {
    let v = u32::from_str_radix(c.trim_start_matches('#'), 16).unwrap_or(0x2563EB);
    rgb(v).into()
}

fn sk(p: &ThPal, w: f32) -> Div {
    div()
        .flex_none()
        .w(px(w))
        .h(px(9.))
        .rounded(px(5.))
        .bg(p.sel)
}

fn phase_tone(p: &ThPal, ph: Phase) -> Hsla {
    match ph.tone() {
        "run" => tone(p, IconTone::Running),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        _ => p.mute,
    }
}

fn pill(p: &ThPal, l: impl Into<SharedString>, c: Hsla) -> Div {
    div()
        .flex_none()
        .h(px(20.))
        .px(px(7.))
        .flex()
        .items_center()
        .gap(px(5.))
        .rounded(px(6.))
        .bg(c.opacity(0.16))
        .text_size(px(11.5))
        .text_color(p.fg2)
        .child(div().w(px(6.)).h(px(6.)).rounded(px(3.)).bg(c))
        .child(l.into())
}

/// The brand mark drawn from the wizard.
fn mark(w: &Wiz, s: f32) -> Div {
    let c = hex(&w.colour);
    let ink = hex(ink_on(&w.colour));
    let kind = MARKS.get(w.mark).map(|m| m.0).unwrap_or("initials");
    let d = div()
        .w(px(s))
        .h(px(s))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(if kind == "round" { s / 2. } else { s * 0.28 }))
        .bg(c)
        .text_color(ink)
        .text_size(px(s * 0.4))
        .font_weight(FontWeight::BOLD);
    match kind {
        "square" => d.child(
            div()
                .w(px(s * 0.36))
                .h(px(s * 0.36))
                .rounded(px(2.))
                .bg(ink),
        ),
        "upload" => d.bg(ink).child(
            div()
                .w(px(s * 0.5))
                .h(px(s * 0.5))
                .rounded(px(s * 0.1))
                .bg(c),
        ),
        _ => d.child(initials(company(w))),
    }
}

fn start(st: ScreenState, extra: &str) -> Wiz {
    let mut w = wiz(st != ScreenState::Empty);
    match extra {
        "fetching" => {
            w.domains.push(Domain {
                host: format!("docs.{OWN_DOMAIN}"),
                phase: Phase::Reading(50),
            });
            w.domains.push(Domain {
                host: format!("help.{OWN_DOMAIN}"),
                phase: Phase::Queued,
            });
        }
        "custom" => {
            w.colour = "#0E7C86".to_string();
            w.mark = 3;
        }
        "live" => w.live = true,
        _ => {}
    }
    w
}

fn press(p: &ThPal, l: &'static str, primary: bool, act: &Option<Act>, id: &'static str) -> Div {
    let f = fire(act, id);
    div().child(
        btn(p, l, primary)
            .id(SharedString::from(id))
            .on_click(move |_, w, cx| f(w, cx)),
    )
}

fn top(p: &ThPal, step: usize, act: &Option<Act>) -> Div {
    let mut t = div()
        .h(px(56.))
        .flex_none()
        .px(px(24.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .w(px(18.))
                .h(px(18.))
                .rounded(px(4.))
                .bg(p.fg)
                .flex_none(),
        );
    if step > 0 {
        t = t.child(press(p, "← Back", false, act, "back"));
    }
    t.child(div().flex_1())
        .child(quiet(p, format!("Logged in as {PERSON_EMAIL}")))
        .child(press(p, "Log out", false, act, "logout"))
}

fn steps(p: &ThPal, step: usize, named: bool) -> Div {
    let mut d = div().flex().justify_center().gap(px(4.)).mb(px(14.));
    for (i, (_, l)) in STEPS.iter().enumerate() {
        let on = i <= step;
        let mut s = div().flex().flex_col().items_center().gap(px(6.)).child(
            div()
                .w(px(if named { 56. } else { 16. }))
                .h(px(3.))
                .rounded(px(2.))
                .bg(if on { p.fg } else { p.line }),
        );
        if named {
            s = s.child(
                div()
                    .text_size(px(11.5))
                    .text_color(if i == step { p.fg } else { p.mute })
                    .child(*l),
            );
        }
        d = d.child(s);
    }
    d
}

fn field(p: &ThPal, v: &str, ph: &str, arrow: bool, act: &Option<Act>) -> Div {
    let mut f = div()
        .h(px(46.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(div().flex_1().min_w_0().px(px(6.)).child(if v.is_empty() {
            div().text_color(p.mute).child(ph.to_string())
        } else {
            clip(v.to_string())
        }))
        .child(press(p, "+", false, act, "add"));
    if arrow {
        f = f.child(press(p, "→", true, act, "continue"));
    }
    f
}

fn chip(p: &ThPal, lead: Div, name: &str, tag: &'static str, right: Option<Div>) -> Div {
    let mut c = div()
        .h(px(38.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(10.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(lead)
        .child(clip(name.to_string()))
        .child(
            div()
                .flex_none()
                .px(px(6.))
                .rounded(px(6.))
                .bg(p.sel)
                .text_size(px(11.))
                .text_color(p.mute)
                .child(tag),
        );
    if let Some(r) = right {
        c = c.child(r);
    }
    c.child(quiet(p, "✕"))
}

fn fav(p: &ThPal, h: &str) -> Div {
    div()
        .w(px(18.))
        .h(px(18.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(5.))
        .bg(p.sel)
        .text_size(px(10.5))
        .text_color(p.fg2)
        .child(
            h.chars()
                .next()
                .unwrap_or('?')
                .to_ascii_uppercase()
                .to_string(),
        )
}

fn label(p: &ThPal, l: &'static str) -> Div {
    div()
        .mt(px(16.))
        .mb(px(6.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(p.fg)
        .child(l)
}

fn seg(p: &ThPal, opts: &'static [&'static str], cur: usize) -> Div {
    let mut d = div()
        .flex()
        .gap(px(2.))
        .p(px(3.))
        .rounded(px(10.))
        .bg(p.inset)
        .border_1()
        .border_color(p.line);
    for (i, o) in opts.iter().enumerate() {
        let on = i == cur;
        d = d.child(
            div()
                .flex_1()
                .h(px(28.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .when(on, |x| x.bg(p.card))
                .text_color(if on { p.fg } else { p.mute })
                .child(*o),
        );
    }
    d
}

fn form(p: &ThPal, w: &Wiz, step: usize, act: &Option<Act>) -> Div {
    let mut d = div().flex().flex_col();
    match step {
        0 => {
            d = d.child(field(p, "", OWN_DOMAIN, true, act));
            let mut l = div().mt(px(14.)).flex().flex_col().gap(px(6.));
            for (i, x) in w.domains.iter().enumerate() {
                l = l.child(chip(
                    p,
                    fav(p, &x.host),
                    &x.host,
                    if i == 0 { "Yours" } else { "Site" },
                    Some(pill(p, x.phase.label(), phase_tone(p, x.phase))),
                ));
            }
            if w.domains.is_empty() {
                l = l.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(p.mute)
                        .child("Nothing added yet. Start with your main site."),
                );
            }
            d = d.child(l);
        }
        1 => {
            d = d.child(field(p, "", "Kestrel Studio", false, act));
            let mut l = div().mt(px(14.)).flex().flex_col().gap(px(6.));
            for (i, n) in w.names.iter().enumerate() {
                l = l.child(chip(
                    p,
                    div(),
                    n,
                    if i == 0 { "Company" } else { "Short form" },
                    None,
                ));
            }
            d = d.child(l).child(label(p, "Colour"));
            let mut sws = div().flex().items_center().gap(px(8.));
            for (_, c) in SWATCHES {
                let on = w.colour.eq_ignore_ascii_case(c);
                let s = div().w(px(28.)).h(px(28.)).rounded(px(14.)).bg(hex(c));
                sws = sws.child(if on {
                    div()
                        .p(px(2.))
                        .rounded(px(16.))
                        .border_2()
                        .border_color(hex(c))
                        .child(s.w(px(22.)).h(px(22.)))
                } else {
                    s
                });
            }
            let custom = !SWATCHES.iter().any(|s| s.1.eq_ignore_ascii_case(&w.colour));
            sws = sws.child(
                div()
                    .h(px(30.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded(px(15.))
                    .border_1()
                    .border_color(if custom { p.fg } else { p.line })
                    .child(
                        div()
                            .w(px(18.))
                            .h(px(18.))
                            .rounded(px(9.))
                            .bg(hex(&w.colour)),
                    )
                    .child(quiet(
                        p,
                        if custom {
                            w.colour.clone()
                        } else {
                            "#hex".to_string()
                        },
                    )),
            );
            d = d.child(sws).child(label(p, "Mark"));
            let mut mk = div().flex().gap(px(8.));
            for (i, (_, l)) in MARKS.iter().enumerate() {
                let mut m = w.clone();
                m.mark = i;
                mk = mk.child(
                    div()
                        .flex_1()
                        .py(px(10.))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(6.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(if w.mark == i { p.fg } else { p.line })
                        .child(mark(&m, 28.))
                        .child(div().text_size(px(12.)).child(*l)),
                );
            }
            d = d.child(mk).child(div().mt(px(20.)).child(press(
                p,
                "Continue →",
                true,
                act,
                "continue",
            )));
        }
        2 => {
            d = d
                .child(label(p, "Name"))
                .child(input(p, w.name.clone(), 440., w.name.trim().is_empty()))
                .child(label(p, "Tone"))
                .child(seg(p, &TONES, w.tone))
                .child(label(p, "Reply length"))
                .child(seg(p, &LENGTHS, w.length))
                .child(
                    div()
                        .mt(px(16.))
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Hand over to a person"),
                                )
                                .child(quiet(p, "When it can't answer for sure")),
                        )
                        .child(sw(p, w.handover)),
                );
            if w.handover {
                d = d.child(
                    div()
                        .mt(px(8.))
                        .child(input(p, w.handover_to.clone(), 440., false)),
                );
            }
            d = d.child(
                div()
                    .mt(px(20.))
                    .child(press(p, "Continue →", true, act, "continue")),
            );
        }
        _ => {
            let gap = first_gap(w);
            for (i, (l, v, s)) in summary(w).into_iter().enumerate() {
                let miss = gap == Some(s);
                let ids = [
                    "edit-website",
                    "edit-brand",
                    "edit-name",
                    "edit-voice",
                    "edit-handover",
                ];
                d = d.child(
                    div()
                        .h(px(42.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .border_b_1()
                        .border_color(p.line)
                        .child(div().w(px(120.)).text_color(p.mute).child(l))
                        .child(clip(v).text_color(if miss { p.mute } else { p.fg }))
                        .child(press(
                            p,
                            if miss { "Add" } else { "Edit" },
                            false,
                            act,
                            ids[i.min(4)],
                        )),
                );
            }
            if w.live {
                d = d.child(
                    div()
                        .mt(px(14.))
                        .p(px(10.))
                        .rounded(px(9.))
                        .bg(tone(p, IconTone::Done).opacity(0.12))
                        .child("Your assistant is live."),
                );
            }
            d = d.child(div().mt(px(20.)).child(press(
                p,
                "Open dashboard →",
                true,
                act,
                "open-dashboard",
            )));
        }
    }
    d
}

fn card(p: &ThPal) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .overflow_hidden()
}

fn h3(p: &ThPal, t: &'static str, on: bool) -> Div {
    div()
        .mt(px(18.))
        .mb(px(8.))
        .text_size(px(14.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(if on { p.fg } else { p.mute })
        .child(t)
}

fn preview(p: &ThPal, w: &Wiz, step: usize, lay: ShellLayout, loading: bool, typing: bool) -> Div {
    let nm = if w.name.trim().is_empty() {
        "Your assistant".to_string()
    } else {
        w.name.trim().to_string()
    };
    let comp = company(w).to_string();
    let live = w.live;
    let st_c = if live {
        tone(p, IconTone::Done)
    } else {
        p.mute
    };
    let mut side = div()
        .w(px(232.))
        .flex_none()
        .p(px(14.))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .mb(px(10.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(mark(w, 20.))
                .child(clip(if comp.is_empty() {
                    "Your company".to_string()
                } else {
                    comp.clone()
                })),
        );
    for i in 0..6u32 {
        side = side.child(
            div()
                .flex()
                .gap(px(8.))
                .child(sk(p, 12.))
                .child(sk(p, 60. + ((i * 23) % 50) as f32)),
        );
    }
    let mut body = div().flex().flex_col().pl(px(16.)).pr(px(40.)).pb(px(30.));
    if loading {
        body = body.child(skel(p, 4)).child(skel(p, 5));
    } else {
        let tiles = div()
            .mt(px(16.))
            .flex()
            .gap(px(10.))
            .child(tile(
                p,
                "Knowledge",
                sources_words(&w.domains),
                format!("{} pages learned", pages_total(&w.domains)),
                p.fg,
            ))
            .child(tile(
                p,
                "Conversations",
                "0".to_string(),
                "None yet".to_string(),
                p.fg,
            ))
            .child(tile(
                p,
                "Status",
                if live { "Live" } else { "Draft" }.to_string(),
                if live { "Answering" } else { "Not public yet" }.to_string(),
                st_c,
            ));
        let asst = card(p)
            .p(px(14.))
            .gap(px(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(mark(w, 22.))
                    .child(clip(nm.clone()).font_weight(FontWeight::SEMIBOLD))
                    .child(pill(p, if live { "Live" } else { "Draft" }, st_c)),
            )
            .child(bubble(p, greeting(w.tone, &nm, &comp)))
            .child(
                div()
                    .flex_none()
                    .h(px(28.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .bg(hex(&w.colour))
                    .text_color(hex(ink_on(&w.colour)))
                    .text_size(px(12.))
                    .child("Chat with us"),
            );
        let mut kt = card(p).child(
            div()
                .h(px(32.))
                .px(px(14.))
                .flex()
                .items_center()
                .bg(p.card2)
                .text_size(px(11.5))
                .text_color(p.mute)
                .child(div().flex_1().child("Source"))
                .child(div().w(px(130.)).child("Status"))
                .child(div().w(px(60.)).child("Pages")),
        );
        for d in &w.domains {
            kt = kt.child(
                div()
                    .h(px(40.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .border_t_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .gap(px(8.))
                            .child(fav(p, &d.host))
                            .child(clip(d.host.clone())),
                    )
                    .child(div().w(px(130.)).child(pill(
                        p,
                        d.phase.label(),
                        phase_tone(p, d.phase),
                    )))
                    .child(div().w(px(60.)).child(quiet(
                        p,
                        match d.phase {
                            Phase::Ready(n) => n.to_string(),
                            _ => "–".to_string(),
                        },
                    ))),
            );
        }
        let reply = if typing {
            "…".to_string()
        } else {
            answer(w.tone, w.length)
        };
        let chat = card(p)
            .p(px(14.))
            .gap(px(12.))
            .child(bubble(p, TEST_Q.to_string()))
            .child(
                div().flex().gap(px(10.)).child(mark(w, 20.)).child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(quiet(p, nm.clone()))
                        .child(bubble(p, reply)),
                ),
            )
            .child(quiet(
                p,
                if w.handover && !w.handover_to.is_empty() {
                    format!("Not sure? It hands over to {}", w.handover_to)
                } else {
                    "It answers everything itself".to_string()
                },
            ));
        let order: [(usize, &'static str, Div); 3] = [
            (1, "Your assistant", asst),
            (0, "Knowledge", kt),
            (2, "Test chat", chat),
        ];
        body = body.child(tiles);
        for (s, t, el) in order {
            let on = s == step || step == 3;
            body = body
                .child(h3(p, t, on))
                .child(el.opacity(if on { 1. } else { 0.5 }));
        }
    }
    let flat = lay == ShellLayout::Flat;
    div()
        .flex_1()
        .min_w_0()
        .pt(px(24.))
        .pl(px(8.))
        .overflow_hidden()
        .child(
            div()
                .w(px(736.))
                .h_full()
                .flex()
                .bg(if flat { p.card2 } else { p.inset })
                .border_1()
                .border_color(p.line)
                .rounded(px(if flat { 0. } else { 14. }))
                .overflow_hidden()
                .child(side)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .bg(p.card)
                        .border_l_1()
                        .border_color(p.line)
                        .child(
                            div()
                                .h(px(46.))
                                .px(px(16.))
                                .flex()
                                .items_center()
                                .border_b_1()
                                .border_color(p.line)
                                .text_color(p.fg2)
                                .child("Overview"),
                        )
                        .child(body),
                ),
        )
}

fn tile(p: &ThPal, l: &'static str, v: String, q: String, c: Hsla) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .p(px(12.))
        .flex()
        .flex_col()
        .gap(px(3.))
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(quiet(p, l))
        .child(
            clip(v)
                .text_size(px(18.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(c),
        )
        .child(quiet(p, q))
}

fn bubble(p: &ThPal, t: String) -> Div {
    div()
        .px(px(12.))
        .py(px(10.))
        .rounded(px(12.))
        .bg(p.inset)
        .text_size(px(12.5))
        .text_color(p.fg2)
        .child(t)
}

fn screen(
    theme: &'static Theme,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    step: usize,
) -> Div {
    let p = th_pal(theme);
    let w = start(st, extra);
    let loading = st == ScreenState::Loading;
    let (h1, lead) = heading(step);
    let mut col = div().w(px(440.)).flex().flex_col();
    if loading {
        col = col.child(skel(&p, 2)).child(skel(&p, 5));
    } else {
        col = col
            .child(steps(&p, step, extra == "named"))
            .child(
                div()
                    .flex()
                    .justify_center()
                    .text_size(px(24.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(h1),
            )
            .child(
                div()
                    .mt(px(8.))
                    .mb(px(22.))
                    .flex()
                    .justify_center()
                    .text_color(p.mute)
                    .child(lead),
            )
            .child(form(&p, &w, step, act));
    }
    let mut body = div().flex_1().min_h_0().flex().child(
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .justify_center()
            .p(px(24.))
            .child(col),
    );
    if extra != "form-only" {
        body = body.child(preview(&p, &w, step, lay, loading, extra == "testing"));
    }
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(p.card)
        .text_color(p.fg)
        .text_size(px(13.))
        .child(top(&p, step, act))
        .child(body)
        .child(
            div()
                .h(px(48.))
                .flex_none()
                .px(px(24.))
                .flex()
                .items_center()
                .gap(px(16.))
                .text_color(p.mute)
                .text_size(px(12.5))
                .child(format!("© 2026 {PRODUCT}"))
                .child("Terms")
                .child("Privacy"),
        )
}
