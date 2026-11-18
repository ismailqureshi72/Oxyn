//! Screens S6 in GPUI: Settings / Profile, Preferences, Notifications, Privacy and General in the
//! settings shell (settings sidebar: Back to app, Personal and Workspace pages, the account) on the
//! shared frame, with a centred 696 px column. The elements draw one state at a time: `.state()`
//! for Data / Loading / Empty and `.extra()` for the screen's own state (`unsaved`, `memoff`,
//! `quiet`, `exporting`, `deleting`; S7: `inviting`, `newkey`, `twostep`, `cancelling`, `complete`)
//! or the Oxyn option `compact`. The S7 pages (Members, Models, Security, Billing, Get started) are
//! drawn by `setscr2.rs`. Presses fire `set-<page>`,
//! `back`, `save`, `discard`, `add-password`, `clear-memory`, `test-send`, `sign-out-others`,
//! `export`, `delete-chats` and `delete-workspace` through `on_act`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::screensfid::{PERSON_PLAN, ShellLayout};
use oxyn_core::parity::setscr::{
    CHANNELS, COOKIES, DAYS, DEFAULT_MODELS, DIGEST_AT, GROUPS, HOURS, KINDS, LANGS, NAV, PERSON,
    PERSON_EMAIL, PERSON_INI, Prefs, ROLES, SHARING, STYLES, ScreenState, THEMES, TOOLS,
    WEEK_START, ZONES, changes_words, digest_words, export_step, initials, lead, links_in, prefs,
    quiet_words, reach_words, slug_error,
};
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::clip;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, gh, skel};
use super::workscr::{btn, page, quiet, tone};
use crate::brand::brand_mark;

macro_rules! s6_el {
    ($(#[$m:meta])* $name:ident, $page:literal) => {
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
            /// The screen's own state or `compact` (see the module docs).
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
            /// Press ids (`set-<page>`, `save`, `export`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                shell(self.theme, self.layout, self.state, self.extra, &self.act, $page)
            }
        }
    };
}

s6_el!(
    /// Settings / Profile: picture, name, role, email, time, personal context and sign-in.
    ProfileSettingsEl,
    "profile"
);
s6_el!(
    /// Settings / Preferences: default model, language, appearance, chat tools, instructions, memory.
    PreferencesSettingsEl,
    "preferences"
);
s6_el!(
    /// Settings / Notifications: digest, kinds × channels, quiet hours.
    NotificationsSettingsEl,
    "notifications"
);
s6_el!(
    /// Settings / Privacy: data use, cookies, sessions, shared links, export and delete.
    PrivacySettingsEl,
    "privacy"
);
s6_el!(
    /// Settings / General: icon, name, address, company context, sharing, danger zone.
    GeneralSettingsEl,
    "general"
);

const NAV_IDS: [&str; 10] = [
    "set-profile",
    "set-preferences",
    "set-notifications",
    "set-privacy",
    "set-general",
    "set-members",
    "set-models",
    "set-security",
    "set-billing",
    "set-onboarding",
];

fn side(p: &ThPal, on: &str, st: ScreenState, act: &Option<Act>) -> Div {
    let back = fire(act, "back");
    let mut s = div()
        .w(px(272.))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(2.))
        .p(px(10.))
        .bg(p.inset)
        .child(
            div()
                .id("back")
                .text_color(p.fg2)
                .child("← Back to app")
                .on_click(move |_, w, cx| back(w, cx)),
        )
        .child(
            div()
                .mt(px(8.))
                .px(px(8.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Settings"),
        )
        .child(
            div()
                .mt(px(6.))
                .h(px(30.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .text_color(p.mute)
                .child("Search settings"),
        );
    if st == ScreenState::Loading {
        s = s.child(skel(p, 8));
    } else {
        for (g, gl) in GROUPS {
            s = s.child(gh(p, gl));
            for (i, n) in NAV.iter().enumerate().filter(|(_, n)| n.2 == g) {
                let id = NAV_IDS[i];
                let f = fire(act, id);
                let sel = n.0 == on;
                s = s.child(
                    div()
                        .id(SharedString::from(id))
                        .px(px(8.))
                        .py(px(6.))
                        .rounded(px(8.))
                        .when(sel, |d| d.bg(p.sel))
                        .text_color(if sel { p.fg } else { p.fg2 })
                        .child(n.1)
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
        }
    }
    s.child(div().flex_1()).child(
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(36.))
            .px(px(6.))
            .child(av(p, PERSON_INI, 26.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(clip(PERSON))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(p.mute)
                            .child(PERSON_PLAN),
                    ),
            )
            .child(quiet(p, "⌃⌄")),
    )
}

pub(super) fn av(p: &ThPal, ini: impl Into<SharedString>, s: f32) -> Div {
    div()
        .w(px(s))
        .h(px(s))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(s / 2.))
        .bg(p.sel)
        .text_size(px(s * 0.36))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(p.fg2)
        .child(ini.into())
}

/// A switch drawn on or off.
pub(super) fn sw(p: &ThPal, on: bool) -> Div {
    let knob = div()
        .w(px(16.))
        .h(px(16.))
        .rounded(px(8.))
        .bg(if on { p.card } else { p.knob });
    div()
        .flex_none()
        .w(px(34.))
        .h(px(20.))
        .p(px(2.))
        .flex()
        .when(on, |d| d.justify_end())
        .rounded(px(10.))
        .bg(if on { p.fg } else { p.off })
        .child(knob)
}

/// A menu button showing its value with a caret.
pub(super) fn menu(p: &ThPal, v: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .min_w(px(96.))
        .h(px(30.))
        .px(px(10.))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.))
        .rounded(px(8.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .text_color(p.fg2)
        .child(v.into())
        .child(quiet(p, "⌄"))
}

pub(super) fn input(p: &ThPal, v: impl Into<SharedString>, w: f32, bad: bool) -> Div {
    div()
        .w(px(w))
        .h(px(32.))
        .px(px(10.))
        .flex()
        .items_center()
        .rounded(px(9.))
        .border_1()
        .border_color(if bad { p.red } else { p.line })
        .bg(p.card)
        .overflow_hidden()
        .child(clip(v.into()))
}

fn area(p: &ThPal, v: &str, cap: usize) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .w_full()
                .min_h(px(64.))
                .p(px(10.))
                .rounded(px(9.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .text_color(if v.is_empty() { p.mute } else { p.fg })
                .child(if v.is_empty() {
                    "Write a few lines".to_string()
                } else {
                    v.to_string()
                }),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .text_size(px(11.5))
                .text_color(p.mute)
                .child(format!("{}/{cap}", v.chars().count())),
        )
}

pub(super) struct Rows<'a> {
    pub(super) p: &'a ThPal,
    pub(super) compact: bool,
}

impl Rows<'_> {
    /// Label and hint on the left, a control on the right.
    pub(super) fn row(&self, label: impl Into<SharedString>, hint: &str, ctl: Div) -> Div {
        let p = self.p;
        let mut l = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .child(div().font_weight(FontWeight::MEDIUM).child(label.into()));
        if !hint.is_empty() && !self.compact {
            l = l.child(
                div()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .child(hint.to_string()),
            );
        }
        div()
            .flex()
            .items_center()
            .gap(px(16.))
            .px(px(14.))
            .py(px(if self.compact { 8. } else { 12. }))
            .border_t_1()
            .border_color(p.line)
            .child(l)
            .child(ctl.flex_none())
    }
    /// A full-width block row (a text area, a grid).
    pub(super) fn block(&self, label: &'static str, hint: &str, body: Div) -> Div {
        let p = self.p;
        let head = self.row(label, hint, div());
        div()
            .flex()
            .flex_col()
            .child(head)
            .child(div().px(px(14.)).pb(px(12.)).child(body))
            .border_color(p.line)
    }
}

pub(super) fn sec(p: &ThPal, title: &'static str, right: Option<String>, rows: Vec<Div>) -> Div {
    let danger = title == "Danger zone";
    let mut card = div()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(if danger { p.red.opacity(0.5) } else { p.line })
        .bg(p.card)
        .overflow_hidden();
    for r in rows {
        card = card.child(r);
    }
    let mut h = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .mt(px(14.))
        .mb(px(6.))
        .px(px(2.))
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(if danger { p.red } else { p.fg2 })
        .child(title)
        .child(div().flex_1());
    if let Some(r) = right {
        h = h.child(quiet(p, r));
    }
    div().flex().flex_col().child(h).child(card)
}

pub(super) fn pill(p: &ThPal, l: &'static str, t: IconTone) -> Div {
    let c = tone(p, t);
    div()
        .flex_none()
        .h(px(18.))
        .px(px(6.))
        .rounded(px(5.))
        .bg(c.opacity(0.22))
        .text_size(px(11.5))
        .child(l)
}

pub(super) fn ic(p: &ThPal, g: &'static str, c: Hsla) -> Div {
    div()
        .w(px(30.))
        .h(px(30.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .text_color(c)
        .child(g)
}

fn kind_tone(p: &ThPal, t: &str) -> Hsla {
    match t {
        "warn" => tone(p, IconTone::Approval),
        "ok" => tone(p, IconTone::Done),
        "bad" => tone(p, IconTone::Failed),
        "run" => tone(p, IconTone::Running),
        _ => p.mute,
    }
}

pub(super) fn hue(p: &ThPal, i: usize) -> Hsla {
    match i {
        1 => tone(p, IconTone::Skills),
        2 => tone(p, IconTone::Audio),
        3 => tone(p, IconTone::Slides),
        4 => tone(p, IconTone::Image),
        5 => tone(p, IconTone::Sheet),
        _ => tone(p, IconTone::Doc),
    }
}

fn shell(
    theme: &'static Theme,
    lay: ShellLayout,
    st: ScreenState,
    extra: &'static str,
    act: &Option<Act>,
    pg: &'static str,
) -> Div {
    let p = th_pal(theme);
    let mut d = prefs(st != ScreenState::Empty);
    let saved = d.clone();
    match extra {
        "unsaved" => {
            d.name = "Rowan A. Vale".to_string();
            d.role = 0;
            d.zone = 2;
        }
        "memoff" => d.memory = false,
        "complete" => d.ws.steps = [true; 6],
        "twostep" => d.ws.two_step_me = false,
        _ => {}
    }
    let r = Rows {
        p: &p,
        compact: extra == "compact",
    };
    let label = NAV.iter().find(|n| n.0 == pg).map(|n| n.1).unwrap_or("");
    let mut col = div()
        .w_full()
        .max_w(px(if matches!(pg, "members" | "billing") {
            816.
        } else {
            696.
        }))
        .mx_auto()
        .px(px(24.))
        .py(px(26.))
        .flex()
        .flex_col()
        .gap(px(2.));
    if st == ScreenState::Loading {
        col = col.child(skel(&p, 3)).child(skel(&p, 6)).child(skel(&p, 4));
    } else {
        col = col
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(label),
            )
            .child(div().text_color(p.mute).child(lead(pg)));
        if st == ScreenState::Empty {
            col = col.child(
                div()
                    .mt(px(10.))
                    .p(px(10.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.line)
                    .text_size(px(12.5))
                    .text_color(p.fg2)
                    .child("A new account: nothing is filled in yet. Add what you need; the defaults cover the rest."),
            );
        }
        for s in match pg {
            "profile" => profile(&p, &r, &d),
            "preferences" => preferences(&p, &r, &d),
            "notifications" => notifications(&p, &r, &d, extra == "quiet"),
            "privacy" => privacy(&p, &r, &d, extra == "exporting", act),
            "general" => general(&p, &r, &d, extra == "deleting", act),
            "members" => super::setscr2::members(&p, &r, &d, extra == "inviting", act),
            "models" => super::setscr2::models(&p, &r, &d, extra == "newkey", act),
            "security" => super::setscr2::security(&p, &r, &d, extra == "twostep", act),
            "billing" => super::setscr2::billing(&p, &r, &d, extra == "cancelling", act),
            _ => super::setscr2::start(&p, &r, &d, act),
        } {
            col = col.child(s);
        }
    }
    let n = oxyn_core::parity::setscr::changes(&saved, &d);
    let mut main = div().flex_1().min_w_0().flex().flex_col().child(col);
    if n > 0 && st != ScreenState::Loading {
        let sv = fire(act, "save");
        let dc = fire(act, "discard");
        main = main.child(
            div()
                .mx_auto()
                .mb(px(16.))
                .w(px(640.))
                .flex()
                .items_center()
                .gap(px(8.))
                .py(px(8.))
                .pl(px(14.))
                .pr(px(8.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(changes_words(n)),
                )
                .child(div().flex_1())
                .child(
                    btn(&p, "Discard", false)
                        .id("discard")
                        .on_click(move |_, w, cx| dc(w, cx)),
                )
                .child(
                    btn(&p, "Save changes", true)
                        .id("save")
                        .on_click(move |_, w, cx| sv(w, cx)),
                ),
        );
    }
    frame(
        theme,
        lay,
        side(&p, pg, st, act),
        page(&p, Some(("Settings", label.to_string())), vec![], main),
        None,
    )
}

fn profile(p: &ThPal, r: &Rows, d: &Prefs) -> Vec<Div> {
    let ini = initials(&d.name);
    vec![
        sec(
            p,
            "About you",
            None,
            vec![
                r.row(
                    "Picture",
                    "Square, at least 256 px",
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(av(p, ini, 36.))
                        .child(btn(p, if d.picture { "Change" } else { "Upload" }, false)),
                ),
                r.row(
                    "Full name",
                    "Shown on your messages and runs",
                    input(p, d.name.clone(), 260., d.name.trim().is_empty()),
                ),
                r.row(
                    "Role",
                    "Helps agents pitch their answers",
                    menu(p, ROLES[d.role]),
                ),
                r.row(
                    "Email",
                    "Sign-in and receipts go here",
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(quiet(p, d.email.clone()))
                        .child(btn(p, "Change", false)),
                ),
            ],
        ),
        sec(
            p,
            "Time",
            None,
            vec![
                r.row(
                    "Time zone",
                    "Digests and quiet hours follow it",
                    menu(p, ZONES[d.zone]),
                ),
                r.row(
                    "Time format",
                    "",
                    menu(p, if d.clock24 { "24-hour" } else { "12-hour" }),
                ),
                r.row("First day of the week", "", menu(p, WEEK_START[d.week])),
            ],
        ),
        sec(
            p,
            "Personal context",
            None,
            vec![r.block(
                "What should agents know about you?",
                "Used in every chat",
                area(p, &d.context, oxyn_core::parity::setscr::CONTEXT_CAP),
            )],
        ),
        sec(
            p,
            "Sign-in",
            None,
            vec![
                r.row(
                    "Google",
                    PERSON_EMAIL,
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .children(brand_mark("google", 16., p.fg))
                        .child(pill(p, "Connected", IconTone::Done)),
                ),
                r.row(
                    "Password",
                    if d.password {
                        "Set. Change it any time."
                    } else {
                        "None yet: you sign in with Google only."
                    },
                    btn(
                        p,
                        if d.password {
                            "Change password"
                        } else {
                            "Add password"
                        },
                        false,
                    ),
                ),
            ],
        ),
    ]
}

fn preferences(p: &ThPal, r: &Rows, d: &Prefs) -> Vec<Div> {
    let (ml, mb, mh) = DEFAULT_MODELS[d.model];
    let mut tools = div().flex().flex_wrap().gap(px(8.)).p(px(10.));
    for (i, (_, l, w, _)) in TOOLS.iter().enumerate() {
        tools = tools.child(
            div()
                .w(px(310.))
                .flex()
                .items_center()
                .gap(px(10.))
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(*l))
                        .child(div().text_size(px(12.)).text_color(p.mute).child(*w)),
                )
                .child(sw(p, d.tools[i])),
        );
    }
    let mut mem: Vec<Div> = vec![r.row(
        "Remember things across chats",
        "It saves short facts it learns and uses them later",
        sw(p, d.memory),
    )];
    if !d.memory {
        mem.push(r.row(
            "Memory is off",
            "Nothing new is saved, and kept memories are not used.",
            div(),
        ));
    } else {
        for m in &d.memories {
            mem.push(r.row(*m, "", quiet(p, "×")));
        }
        if !d.memories.is_empty() {
            mem.push(r.row(
                format!("{} kept", d.memories.len()),
                "",
                btn(p, "Clear all", false).text_color(p.red),
            ));
        }
    }
    vec![
        sec(
            p,
            "Models",
            None,
            vec![
                r.row(
                    "Default model",
                    mh,
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .children(brand_mark(mb, 14., p.fg))
                        .child(menu(p, ml)),
                ),
            ],
        ),
        sec(
            p,
            "Language and look",
            None,
            vec![
                r.row("Language", "Menus and answers", menu(p, LANGS[d.lang])),
                r.row(
                    "Appearance",
                    "System follows your device",
                    menu(p, THEMES[d.theme]),
                ),
            ],
        ),
        sec(
            p,
            "Chat tools",
            Some(format!(
                "{} of 4 on",
                d.tools.iter().filter(|t| **t).count()
            )),
            vec![tools],
        ),
        sec(
            p,
            "Custom instructions",
            None,
            vec![
                r.block(
                    "How should it answer?",
                    "Added to every chat you start",
                    area(p, &d.instructions, oxyn_core::parity::setscr::INSTR_CAP),
                ),
                r.row("Answer length", "", menu(p, STYLES[d.style])),
            ],
        ),
        sec(p, "Memory", None, mem),
    ]
}

fn notifications(p: &ThPal, r: &Rows, d: &Prefs, quiet_now: bool) -> Vec<Div> {
    let mut out = Vec::new();
    if quiet_now {
        out.push(
            div()
                .mt(px(10.))
                .flex()
                .items_center()
                .gap(px(10.))
                .p(px(10.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Quiet hours are on"),
                        )
                        .child(quiet(p, "Until 07:00. Approvals still come through.")),
                )
                .child(btn(p, "End now", false)),
        );
    }
    let mut days = div().flex().gap(px(4.));
    for (i, l) in DAYS.iter().enumerate() {
        let on = d.digest_days[i];
        days = days.child(
            div()
                .w(px(30.))
                .h(px(30.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(15.))
                .border_1()
                .border_color(if on { p.fg } else { p.line })
                .bg(if on { p.fg } else { p.card })
                .text_color(if on { p.card } else { p.mute })
                .text_size(px(12.))
                .child(l[..1].to_string()),
        );
    }
    let mut dig = vec![r.row(
        "Send a digest",
        "What ran, what waits for you, what it cost",
        sw(p, d.digest),
    )];
    if d.digest {
        dig.push(r.row(
            "Send at",
            "Your local time",
            menu(p, DIGEST_AT[d.digest_at]),
        ));
        dig.push(r.row("On", "", days));
    }
    out.push(sec(p, "Daily digest", Some(digest_words(d)), dig));
    let mut grid = div().flex().flex_col();
    let mut head = div()
        .flex()
        .items_center()
        .px(px(14.))
        .py(px(6.))
        .text_size(px(12.))
        .text_color(p.mute)
        .child(div().flex_1().child("Kind"));
    for c in CHANNELS {
        head = head.child(div().w(px(72.)).flex().justify_center().child(c));
    }
    grid = grid.child(head);
    for (ri, (l, w, _, t)) in KINDS.iter().enumerate() {
        let mut rr = div()
            .flex()
            .items_center()
            .px(px(14.))
            .py(px(8.))
            .border_t_1()
            .border_color(p.line)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .w(px(8.))
                            .h(px(8.))
                            .rounded(px(4.))
                            .bg(kind_tone(p, t)),
                    )
                    .child(
                        div().min_w_0().flex().flex_col().child(clip(*l)).child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.mute)
                                .child(format!("{w} · {}", reach_words(&d.grid[ri]))),
                        ),
                    ),
            );
        for c in 0..3 {
            let on = d.grid[ri][c];
            rr = rr.child(
                div().w(px(72.)).flex().justify_center().child(
                    div()
                        .w(px(16.))
                        .h(px(16.))
                        .rounded(px(4.))
                        .border_1()
                        .border_color(if on { p.fg } else { p.tick })
                        .bg(if on { p.fg } else { p.card })
                        .text_color(p.card)
                        .text_size(px(11.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(if on { "✓" } else { "" }),
                ),
            );
        }
        grid = grid.child(rr);
    }
    out.push(sec(p, "What to tell you", None, vec![grid]));
    let mut q = vec![r.row("Quiet hours", &quiet_words(d), sw(p, d.quiet))];
    if d.quiet {
        q.push(
            r.row(
                "From – to",
                "",
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(menu(p, HOURS[d.quiet_from]))
                    .child(quiet(p, "–"))
                    .child(menu(p, HOURS[d.quiet_to])),
            ),
        );
        q.push(r.row(
            "Let approvals through",
            "An agent waiting on you can still reach your phone",
            sw(p, d.quiet_urgent),
        ));
    }
    out.push(sec(p, "Quiet hours", None, q));
    out.push(
        div()
            .mt(px(14.))
            .flex()
            .child(btn(p, "Send a test notification", false)),
    );
    out
}

fn privacy(p: &ThPal, r: &Rows, d: &Prefs, exporting: bool, act: &Option<Act>) -> Vec<Div> {
    let so = fire(act, "sign-out-others");
    let ex = fire(act, "export");
    let mut sess: Vec<Div> = Vec::new();
    for s in &d.sessions {
        sess.push(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(14.))
                .py(px(10.))
                .border_t_1()
                .border_color(p.line)
                .child(ic(p, if s.glyph == "phone" { "▯" } else { "▭" }, p.fg2))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(s.device))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.mute)
                                .child(format!("{} · {} · {}", s.app, s.place, s.seen)),
                        ),
                )
                .child(if s.current {
                    pill(p, "This device", IconTone::Done)
                } else {
                    btn(p, "Sign out", false)
                }),
        );
    }
    let mut links: Vec<Div> = vec![
        div()
            .flex()
            .gap(px(14.))
            .px(px(14.))
            .pt(px(10.))
            .pb(px(6.))
            .text_size(px(12.5))
            .child(
                div()
                    .text_color(p.fg)
                    .font_weight(FontWeight::MEDIUM)
                    .child(format!(
                        "In the workspace  {}",
                        links_in(&d.links, "workspace").len()
                    )),
            )
            .child(div().text_color(p.mute).child(format!(
                "Public links  {}",
                links_in(&d.links, "public").len()
            ))),
    ];
    for l in links_in(&d.links, "workspace") {
        links.push(r.row(
            l.title,
            &format!("{} views · shared {}", l.views, l.when),
            btn(p, "Copy link", false),
        ));
    }
    let mut data = vec![
        r.row(
            "Export your data",
            "Chats, files and memories as one archive, by email",
            div().child(
                btn(p, "Export", false)
                    .id("export")
                    .on_click(move |_, w, cx| ex(w, cx)),
            ),
        ),
    ];
    if exporting {
        let (pc, step, _) = export_step(40);
        data.push(
            div()
                .px(px(14.))
                .pb(px(12.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(quiet(p, format!("{step} {pc}%")))
                .child(
                    div().h(px(4.)).w_full().rounded(px(2.)).bg(p.line).child(
                        div()
                            .h_full()
                            .w(gpui::relative(pc as f32 / 100.))
                            .rounded(px(2.))
                            .bg(tone(p, IconTone::Running)),
                    ),
                ),
        );
    }
    data.push(r.row(
        "Delete all chats",
        "Removes every chat and its shared links. Memories stay.",
        btn(p, "Delete all chats", false).text_color(p.red),
    ));
    let others = d.sessions.iter().filter(|s| !s.current).count();
    let mut sess_sec = sec(p, "Active sessions", None, sess);
    if others > 0 {
        sess_sec = sess_sec.child(
            div().mt(px(6.)).flex().justify_end().child(
                div()
                    .id("sign-out-others")
                    .text_size(px(12.5))
                    .text_color(p.fg2)
                    .child("Sign out of all others")
                    .on_click(move |_, w, cx| so(w, cx)),
            ),
        );
    }
    vec![
        sec(
            p,
            "How your data is used",
            None,
            vec![
                r.row(
                    "Improve the models",
                    "Let chats from this account train future models",
                    sw(p, d.train),
                ),
                r.row(
                    "Product analytics",
                    "Anonymous usage that helps us fix what is slow or confusing",
                    sw(p, d.analytics),
                ),
                r.row(
                    "Ad measurement",
                    "Whether an ad brought you here",
                    sw(p, d.ads),
                ),
                r.row(
                    "Cookies",
                    "Essential cookies keep you signed in",
                    menu(p, COOKIES[d.cookies]),
                ),
                r.row(
                    "Product emails",
                    "Tips and new features, at most twice a month",
                    sw(p, d.product_mail),
                ),
            ],
        ),
        sess_sec,
        sec(p, "Shared links", None, links),
        sec(p, "Your data", None, data),
    ]
}

fn general(p: &ThPal, r: &Rows, d: &Prefs, deleting: bool, act: &Option<Act>) -> Vec<Div> {
    let del = fire(act, "delete-workspace");
    let ini = initials(&d.ws_name);
    let mut hues = div().flex().items_center().gap(px(6.));
    for i in 0..6 {
        hues = hues.child(
            div()
                .w(px(18.))
                .h(px(18.))
                .rounded(px(9.))
                .bg(hue(p, i))
                .when(i == d.ws_hue, |x| x.border_2().border_color(p.fg)),
        );
    }
    let bad = slug_error(&d.ws_slug);
    let mut danger = vec![
        r.row(
            "Delete workspace",
            "Chats, files, agents and members go with it, after 14 days.",
            div().child(
                btn(p, "Delete workspace", false)
                    .text_color(p.red)
                    .id("delete-workspace")
                    .on_click(move |_, w, cx| del(w, cx)),
            ),
        ),
    ];
    if deleting {
        danger.push(
            div()
                .mx(px(14.))
                .mb(px(12.))
                .p(px(10.))
                .rounded(px(10.))
                .bg(p.inset)
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(quiet(p, "Type Kestrel Studio to confirm"))
                .child(input(p, "Kestrel", 280., false))
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .child(btn(p, "Cancel", false))
                        .child(btn(p, "Delete workspace", false).text_color(p.mute)),
                ),
        );
    }
    vec![
        sec(
            p,
            "Workspace",
            None,
            vec![
                r.row(
                    "Icon",
                    "Its initials on a colour of your choice",
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            div()
                                .w(px(36.))
                                .h(px(36.))
                                .rounded(px(9.))
                                .bg(hue(p, d.ws_hue))
                                .text_color(gpui::white())
                                .font_weight(FontWeight::SEMIBOLD)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(ini),
                        )
                        .child(hues),
                ),
                r.row(
                    "Workspace name",
                    "Shown to everyone in it and on invites",
                    input(p, d.ws_name.clone(), 260., d.ws_name.trim().is_empty()),
                ),
                r.row(
                    "Workspace address",
                    bad.unwrap_or("Available"),
                    input(p, format!("{}.oxyn.app", d.ws_slug), 260., bad.is_some()),
                ),
            ],
        ),
        sec(
            p,
            "Company context",
            None,
            vec![
                r.block(
                    "What every agent here should know",
                    "Members can read it",
                    area(p, &d.company, oxyn_core::parity::setscr::COMPANY_CAP),
                ),
                r.row(
                    "New chats are visible to",
                    "People can still share one chat at a time",
                    menu(p, SHARING[d.sharing]),
                ),
            ],
        ),
        sec(p, "Danger zone", None, danger),
    ]
}

s6_el!(
    /// Settings / Members: seats, search and role filter, invite, the member table and teams.
    MembersSettingsEl,
    "members"
);
s6_el!(
    /// Settings / Models: defaults, providers with official marks, switches, access and API keys.
    ModelsSettingsEl,
    "models"
);
s6_el!(
    /// Settings / Security: two-step sign-in, passkeys, sign-in rules, domains, devices, audit log.
    SecuritySettingsEl,
    "security"
);
s6_el!(
    /// Settings / Billing: plan, usage meters, plans, extra usage, payment method, invoices, cancel.
    BillingSettingsEl,
    "billing"
);
s6_el!(
    /// Settings / Get started: the onboarding checklist with progress and steps.
    OnboardingSettingsEl,
    "onboarding"
);
