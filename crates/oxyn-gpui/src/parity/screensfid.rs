//! Screens 1 for GPUI over `oxyn_core::parity::screensfid`: static snapshots of the app frames
//! ([`ScreenShellEl`], [`ScreenRailEl`]) and the AI Chat screens ([`ChatStartEl`], [`ChatOverviewEl`],
//! [`ChatAgentsEl`], [`ChatRailEl`], [`ChatProgressEl`], [`ChatRunEl`]) Folders ([`FoldersEl`],
//! [`FolderDetailEl`]), [`IntegrationsEl`], [`KanbanEl`], [`CrmEl`], [`WorkflowEl`] and [`SettingsEl`]. Each takes the data state (Sample, Loading, Empty), the frame (Inset or
//! Flat) and the theme; the host owns any live state.
//! Rows a host can route carry `Act` ids: `nav-<key>`, `thread-<id>`, `folder-<id>`,
//! `app-<brand id>`, `kb-<column>`, `crm-<view>`, `wf-<kind>`, `set-<page>`.

use gpui::{
    App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::screensfid::{
    APP_TABS, CRM_VIEWS, DealSort, KB_COLS, KB_PRI, PERSON, PERSON_INI, ScreenState, ShellLayout,
    WORKSPACE, ago, app_view, apps, board, close_words, col_label, column, connected_count,
    deal_view, deals, due_words, folder_files, folder_view, folders, member_name, over_limit,
    projects, size_label, split_pinned, stage_label, stage_totals, synced_words, task_key, threads,
    total_size, usd, view_count,
};
use oxyn_core::parity::screensfid::{
    EnvMode, PERSON_EMAIL, PERSON_PLAN, SIDE_W, STAGE_H, STAGE_STATES, STAGE_W, STATE_BAR_H, TOP_H,
    WORK_W, account_menu, context_card, thousands, usage,
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{ThPal, fire, th_pal};
use crate::brand::{brand_mark, tone_color};

pub(super) fn skel(p: &ThPal, n: usize) -> Div {
    let mut d = div().flex().flex_col().gap(px(10.)).p(px(10.));
    for i in 0..n {
        d = d.child(
            div()
                .h(px(12.))
                .w(px([220., 150., 190., 110., 170.][i % 5]))
                .rounded(px(6.))
                .bg(p.sel),
        );
    }
    d
}

pub(super) fn empty(p: &ThPal, t: &'static str, l: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .py(px(48.))
        .child(div().w(px(40.)).h(px(40.)).rounded(px(12.)).bg(p.sel))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
        .child(div().text_color(p.mute).child(l))
}

pub(super) fn gh(p: &ThPal, t: &'static str) -> Div {
    div()
        .mt(px(10.))
        .mb(px(4.))
        .px(px(8.))
        .text_size(px(11.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(p.mute)
        .child(t)
}

/// A small object glyph in its semantic tone (a folder tab over a body, or a rounded tile).
pub(super) fn tone_ic(p: &ThPal, tone: IconTone) -> Div {
    let c = tone_color(tone, p.fg.l > 0.5);
    let tile = div().w(px(14.)).h(px(11.)).rounded(px(3.)).bg(c);
    if tone == IconTone::Folder {
        div()
            .flex_none()
            .flex()
            .flex_col()
            .child(
                div()
                    .w(px(6.))
                    .h(px(2.))
                    .rounded_t(px(1.5))
                    .bg(c.opacity(0.75)),
            )
            .child(tile)
    } else {
        div().flex_none().child(tile)
    }
}

/// A row with an object glyph in front (projects, files).
pub(super) fn row_ic(p: &ThPal, tone: IconTone, t: String, a: String, on: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .h(px(30.))
        .px(px(8.))
        .rounded(px(8.))
        .when(on, |d| d.bg(p.sel))
        .text_color(p.fg2)
        .child(tone_ic(p, tone))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(t),
        )
        .child(
            div()
                .flex_none()
                .text_size(px(12.))
                .text_color(p.mute)
                .child(a),
        )
}

pub(super) fn row(p: &ThPal, t: String, a: String, on: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .h(px(30.))
        .px(px(8.))
        .rounded(px(8.))
        .when(on, |d| d.bg(p.sel))
        .text_color(p.fg2)
        .child(div().flex_1().overflow_hidden().child(t))
        .child(div().text_size(px(12.)).text_color(p.mute).child(a))
}

pub(super) fn sidebar(p: &ThPal, st: ScreenState, act: &Option<Act>, runs: bool) -> Div {
    let mut s = div()
        .w(px(SIDE_W.0 as f32))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(6.))
                .child(div().w(px(22.)).h(px(22.)).rounded(px(7.)).bg(p.fg))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(WORKSPACE)),
        );
    s = s.child(
        div()
            .h(px(32.))
            .my(px(8.))
            .px(px(8.))
            .flex()
            .items_center()
            .rounded(px(9.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.mute)
            .child("Search  ⌘K"),
    );
    let nav: Vec<(&str, &str)> = if runs {
        vec![
            ("home", "Home"),
            ("chat", "Chat"),
            ("runs", "Runs  3"),
            ("stats", "Numbers"),
            ("made", "Made for you"),
        ]
    } else {
        vec![("new", "New chat"), ("files", "Files"), ("apps", "Apps")]
    };
    for (k, l) in nav {
        let f = fire(act, k);
        s = s.child(
            div()
                .id(SharedString::from(format!("nav-{k}")))
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_color(p.fg2)
                .when(runs && k == "runs", |d| d.bg(p.sel))
                .child(l)
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    if !runs {
        s = s.child(env_seg(p, EnvMode::Local, act));
    }
    match st {
        ScreenState::Loading => s = s.child(gh(p, "Chats")).child(skel(p, 6)),
        ScreenState::Empty => {
            s = s.child(gh(p, "Chats")).child(
                div()
                    .px(px(8.))
                    .text_color(p.mute)
                    .child("Chats you start land here."),
            )
        }
        ScreenState::Sample if runs => {
            s = s.child(gh(p, "Agents"));
            for a in ["Member update", "Revenue desk", "Inbox sweeper", "Flows"] {
                s = s.child(row(p, a.to_string(), String::new(), false));
            }
        }
        ScreenState::Sample => {
            let (pin, rest) = split_pinned(&threads());
            s = s.child(gh(p, "Pinned"));
            for t in pin {
                s = s.child(row(p, t.title, ago(t.mins), false));
            }
            s = s.child(gh(p, "Projects"));
            for pr in projects() {
                s = s.child(row_ic(
                    p,
                    IconTone::Folder,
                    pr.label.to_string(),
                    pr.chats.len().to_string(),
                    false,
                ));
            }
            s = s.child(gh(p, "Chats"));
            for t in rest {
                s = s.child(row(p, t.title, ago(t.mins), t.id == 2));
            }
        }
    }
    s.child(div().flex_1()).child(side_foot(p, act))
}

/// The Local / Cloud control (`env-local`, `env-cloud`).
fn env_seg(p: &ThPal, cur: EnvMode, act: &Option<Act>) -> Div {
    let mut seg = div()
        .flex()
        .gap(px(2.))
        .mt(px(8.))
        .mb(px(2.))
        .p(px(2.))
        .rounded(px(9.))
        .bg(p.sel);
    for m in EnvMode::ALL {
        let id: &'static str = if m == EnvMode::Cloud {
            "env-cloud"
        } else {
            "env-local"
        };
        let f = fire(act, id);
        let on = m == cur;
        let tone = if m == EnvMode::Cloud {
            IconTone::Cloud
        } else {
            IconTone::Terminal
        };
        seg = seg.child(
            div()
                .id(id)
                .flex_1()
                .h(px(26.))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .rounded(px(7.))
                .text_size(px(12.5))
                .text_color(p.mute)
                .when(on, |d| d.bg(p.card).text_color(p.fg))
                .child(div().w(px(8.)).h(px(8.)).rounded(px(4.)).bg(if on {
                    tone_color(tone, p.fg.l > 0.5)
                } else {
                    p.mute
                }))
                .child(m.label())
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    seg
}

fn avatar(p: &ThPal, d: f32) -> Div {
    div()
        .w(px(d))
        .h(px(d))
        .flex_none()
        .rounded(px(d / 2.))
        .bg(tone_color(IconTone::Skills, p.fg.l > 0.5).opacity(0.22))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(PERSON_INI)
}

fn two_line(p: &ThPal, a: &'static str, b: &'static str) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(
            div()
                .text_size(px(13.))
                .font_weight(FontWeight::MEDIUM)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(a),
        )
        .child(
            div()
                .text_size(px(11.5))
                .text_color(p.mute)
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .child(b),
        )
}

/// The sidebar bottom: the context card slot (first card by priority), then the account row
/// (avatar, name over plan, up/down chevron) and the bell (`acct`, `bell`).
fn side_foot(p: &ThPal, act: &Option<Act>) -> Div {
    let mut f = div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .mx(px(-8.))
        .mb(px(-8.))
        .p(px(8.))
        .border_t_1()
        .border_color(p.line);
    if let Some((_, t, l)) = context_card(&[]) {
        f = f.child(
            div()
                .p(px(10.))
                .rounded(px(10.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(
                            div()
                                .w(px(7.))
                                .h(px(7.))
                                .rounded(px(4.))
                                .bg(tone_color(IconTone::Running, p.fg.l > 0.5)),
                        )
                        .child(t),
                )
                .child(div().text_size(px(12.)).text_color(p.mute).child(l)),
        );
    }
    let fa = fire(act, "acct");
    let fb = fire(act, "bell");
    f.child(
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .child(
                div()
                    .id("acct")
                    .flex_1()
                    .min_w_0()
                    .h(px(44.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .rounded(px(9.))
                    .child(avatar(p, 28.))
                    .child(two_line(p, PERSON, PERSON_PLAN))
                    .child(div().text_color(p.mute).child("⌄"))
                    .on_click(move |_, w, cx| fa(w, cx)),
            )
            .child(
                div()
                    .id("bell")
                    .relative()
                    .w(px(30.))
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .text_color(p.mute)
                    .child(
                        div()
                            .w(px(12.))
                            .h(px(11.))
                            .rounded_t(px(6.))
                            .border_1()
                            .border_color(p.mute),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(3.))
                            .right(px(2.))
                            .min_w(px(14.))
                            .h(px(14.))
                            .px(px(3.))
                            .rounded(px(7.))
                            .bg(tone_color(IconTone::Failed, p.fg.l > 0.5))
                            .text_color(gpui::white())
                            .text_size(px(9.5))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("3"),
                    )
                    .on_click(move |_, w, cx| fb(w, cx)),
            ),
    )
}

/// The account menu that opens upward from the sidebar's account row: a header (avatar, name,
/// email, plan), Settings, Usage ▸, Upgrade plan, Language, Appearance, Keyboard shortcuts,
/// Integrations, Download apps, Changelog, Feedback & help ▸ and Log out, with the Usage flyout
/// (plan, renewal, credits meter, quotas, View usage) beside it when `usage(true)`. Items fire
/// their key (`settings`, `usage`, `logout`…).
#[derive(IntoElement)]
pub struct AccountMenuEl {
    usage: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AccountMenuEl {
    fn default() -> Self {
        Self {
            usage: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AccountMenuEl {
    /// Show the Usage flyout (default true).
    pub fn usage(mut self, on: bool) -> Self {
        self.usage = on;
        self
    }
    /// Light or dark chrome.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// An item's key (`settings`, `usage`…) when it is pressed.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn menu_box(p: &ThPal, w: f32) -> Div {
    div()
        .w(px(w))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(5.))
        .rounded(px(12.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .text_size(px(13.))
        .text_color(p.fg2)
}

fn menu_sep(p: &ThPal) -> Div {
    div().h(px(1.)).mx(px(6.)).my(px(4.)).bg(p.line)
}

impl RenderOnce for AccountMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let dark = p.fg.l > 0.5;
        let u = usage();
        let accent = tone_color(IconTone::Skills, dark);
        let mut m = menu_box(&p, 264.).child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(8.))
                .pt(px(8.))
                .pb(px(10.))
                .mb(px(4.))
                .border_b_1()
                .border_color(p.line)
                .child(avatar(&p, 32.))
                .child(two_line(&p, PERSON, PERSON_EMAIL))
                .child(
                    div()
                        .h(px(20.))
                        .px(px(7.))
                        .rounded(px(10.))
                        .bg(accent.opacity(0.16))
                        .text_color(accent)
                        .text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .flex()
                        .items_center()
                        .child(u.plan),
                ),
        );
        for (k, l, _, h) in account_menu().iter().copied() {
            if k.is_empty() {
                m = m.child(menu_sep(&p));
                continue;
            }
            let id = SharedString::from(format!("acct-{k}"));
            let f = fire(&self.act, k);
            let on = self.usage && k == "usage";
            m = m.child(
                div()
                    .id(id)
                    .h(px(32.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(8.))
                    .when(on, |d| d.bg(p.sel).text_color(p.fg))
                    .when(k == "upgrade", |d| d.text_color(accent))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(l),
                    )
                    .child(div().text_size(px(11.5)).text_color(p.mute).child(h))
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let mut out = div().flex().items_end().gap(px(6.)).child(m);
        if self.usage {
            let pct = u.pct();
            let bar = if u.near() {
                IconTone::Schedule
            } else {
                IconTone::Running
            };
            let mut fly = menu_box(&p, 248.)
                .child(
                    div()
                        .px(px(8.))
                        .pt(px(6.))
                        .pb(px(8.))
                        .child(two_line(&p, "Pro plan", u.renews)),
                )
                .child(
                    div()
                        .px(px(8.))
                        .pb(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex()
                                .text_size(px(12.5))
                                .child(div().flex_1().child("Credits"))
                                .child(div().text_color(p.mute).child(format!(
                                    "{} of {} · {pct}%",
                                    thousands(u.used),
                                    thousands(u.total)
                                ))),
                        )
                        .child(
                            div().h(px(6.)).rounded(px(3.)).bg(p.line).child(
                                div()
                                    .h_full()
                                    .w(px(232. * pct as f32 / 100.))
                                    .rounded(px(3.))
                                    .bg(tone_color(bar, dark)),
                            ),
                        ),
                );
            for &(l, used, lim) in &u.quotas {
                fly = fly.child(row(&p, l.to_string(), format!("{used} / {lim}"), false));
            }
            fly = fly
                .child(menu_sep(&p))
                .child(row(&p, "View usage".into(), String::new(), false));
            out = out.child(fly);
        }
        out
    }
}

pub(super) fn page(p: &ThPal, title: &'static str, body: Div) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(TOP_H as f32))
                .px(px(14.))
                .flex()
                .items_center()
                .border_b_1()
                .border_color(p.line)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(body)
}

fn work_pane(p: &ThPal, w: Div) -> Div {
    div()
        .w(px(WORK_W.0 as f32))
        .flex_none()
        .border_l_1()
        .border_color(p.line)
        .bg(p.card2)
        .p(px(12.))
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(w)
}

pub(super) fn frame(
    theme: &Theme,
    lay: ShellLayout,
    left: Div,
    main: Div,
    work: Option<Div>,
) -> Div {
    let p = th_pal(theme);
    let mut pg = div().flex_1().flex().bg(p.card).child(main);
    if let Some(w) = work {
        pg = pg.child(work_pane(&p, w));
    }
    let pg = match lay {
        ShellLayout::Inset => pg
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .overflow_hidden(),
        ShellLayout::Flat => pg.border_l_1().border_color(p.line),
    };
    let root = div()
        .w(px(STAGE_W as f32))
        .h(px(STAGE_H as f32))
        .flex()
        .rounded(px(16.))
        .border_1()
        .border_color(p.line)
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.))
        .overflow_hidden()
        .child(left)
        .child(pg);
    match lay {
        ShellLayout::Inset => root.pt(px(8.)).pb(px(8.)).pr(px(8.)),
        ShellLayout::Flat => root,
    }
}

/// The screen stage (rules R1/R2): a screen on its fixed 1440 × 900 canvas, with the 52 px state
/// bar under it (Data · Loading · Empty) firing `state-sample`, `state-loading` or `state-empty`
/// so the host re-renders the screen in that state. GPUI has no element transform, so the canvas
/// draws at its native size: size the window (or a scroll view) with
/// `oxyn_core::parity::screensfid::stage_scale`.
#[derive(IntoElement)]
pub struct ScreenStageEl {
    screen: gpui::AnyElement,
    state: ScreenState,
    states: &'static [(&'static str, &'static str)],
    cur: &'static str,
    bar: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ScreenStageEl {
    /// A stage around `screen` (any screen element, already in its state).
    pub fn new(screen: impl IntoElement) -> Self {
        Self {
            screen: screen.into_any_element(),
            state: ScreenState::Sample,
            states: &STAGE_STATES,
            cur: "",
            bar: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// The state the bar shows as current.
    pub fn state(mut self, s: ScreenState) -> Self {
        self.state = s;
        self
    }
    /// The states the bar offers (default Data · Loading · Empty); a screen passes its own row,
    /// e.g. `oxyn_core::parity::screensfid::RUN_STATES`.
    pub fn states(mut self, s: &'static [(&'static str, &'static str)]) -> Self {
        self.states = s;
        self
    }
    /// The current state when it is one of the screen's own (`input`, `attached`…).
    pub fn current(mut self, k: &'static str) -> Self {
        self.cur = k;
        self
    }
    /// Show the state bar (default true).
    pub fn bar(mut self, on: bool) -> Self {
        self.bar = on;
        self
    }
    /// Light or dark chrome.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `state-<key>` from the bar.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ScreenStageEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let canvas = div()
            .w(px(STAGE_W as f32))
            .h(px(STAGE_H as f32))
            .flex_none()
            .overflow_hidden()
            .child(self.screen);
        let mut seg = div().flex().gap(px(2.)).p(px(2.)).rounded(px(9.)).bg(p.sel);
        let cur = if self.cur.is_empty() {
            self.state.key()
        } else {
            self.cur
        };
        for &(k, l) in self.states {
            let id = state_id(k);
            let on = cur == k;
            let fi = fire(&self.act, id);
            seg = seg.child(
                div()
                    .id(id)
                    .h(px(28.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .when(on, |d| d.bg(p.card).text_color(p.fg))
                    .child(l)
                    .on_click(move |_, w, cx| fi(w, cx)),
            );
        }
        let bar = div()
            .w(px(STAGE_W as f32))
            .h(px(STATE_BAR_H as f32))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(12.))
            .px(px(16.))
            .border_t_1()
            .border_color(p.line)
            .bg(p.card)
            .text_size(px(12.5))
            .text_color(p.mute)
            .child(
                div()
                    .text_color(p.fg2)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("State"),
            )
            .child(seg)
            .child(div().flex_1())
            .child(format!("{STAGE_W} × {STAGE_H}"));
        div()
            .flex()
            .flex_col()
            .child(canvas)
            .when(self.bar, |d| d.child(bar))
    }
}

/// The `Act` id a state button fires.
fn state_id(k: &str) -> &'static str {
    match k {
        "loading" => "state-loading",
        "empty" => "state-empty",
        "attached" => "state-attached",
        "dictating" => "state-dictating",
        "limit" => "state-limit",
        "making" => "state-making",
        "folded" => "state-folded",
        "working" => "state-working",
        "input" => "state-input",
        "done" => "state-done",
        _ => "state-sample",
    }
}

macro_rules! screen_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        pub struct $name {
            state: ScreenState,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
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
            /// `nav-<key>` from the sidebar.
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}
pub(super) use screen_el;

screen_el!(
    /// App shell (Inset or Flat): sidebar, a placeholder page and the workspace.
    ScreenShellEl
);
screen_el!(
    /// Three-pane shell: icon rail, chats column, page and work area.
    ScreenRailEl
);

fn workspace(p: &ThPal, st: ScreenState) -> Div {
    let mut w = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .h(px(30.))
                .w(px(110.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .child("New tab"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child("Search or type an address"),
        );
    if st == ScreenState::Loading {
        return w.child(skel(p, 6));
    }
    w = w.child(gh(p, "Tools"));
    for (t, k) in [("Terminal", "⌃`"), ("Browser", "⌘⇧B"), ("Notes", "⌘⇧N")] {
        w = w.child(row(p, t.to_string(), k.to_string(), false));
    }
    w = w.child(gh(p, "Suggested"));
    if st == ScreenState::Empty {
        return w.child(
            div()
                .px(px(8.))
                .text_color(p.mute)
                .child("Pages you open often show up here."),
        );
    }
    for (t, d) in [
        ("Two papers on dye-free linen", "papers.example.org"),
        ("Launch checklist", "wiki.kestrel.example"),
    ] {
        w = w.child(row(p, t.to_string(), d.to_string(), false));
    }
    w
}

fn placeholder(p: &ThPal, st: ScreenState) -> Div {
    match st {
        ScreenState::Loading => skel(p, 7).p(px(24.)),
        ScreenState::Empty => empty(
            p,
            "Your page goes here",
            "The sidebar and work area stay put.",
        ),
        ScreenState::Sample => {
            let mut g = div().flex().flex_wrap().gap(px(12.)).mt(px(20.));
            for _ in 0..6 {
                g = g.child(
                    div()
                        .w(px(160.))
                        .h(px(110.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.inset),
                );
            }
            div()
                .p(px(24.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Your page goes here"),
                )
                .child(g)
        }
    }
}

impl RenderOnce for ScreenShellEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Home", placeholder(&p, self.state)),
            Some(workspace(&p, self.state)),
        )
    }
}

fn rail_left(p: &ThPal, st: ScreenState) -> Div {
    let mut rail = div()
        .w(px(52.))
        .flex_none()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .py(px(10.))
        .child(
            div()
                .w(px(22.))
                .h(px(22.))
                .mb(px(8.))
                .rounded(px(7.))
                .bg(p.fg),
        );
    for i in 0..5 {
        rail = rail.child(
            div()
                .w(px(36.))
                .h(px(36.))
                .rounded(px(10.))
                .when(i == 0, |d| d.bg(p.sel)),
        );
    }
    let mut col = div()
        .w(px(230.))
        .flex_none()
        .p(px(8.))
        .flex()
        .flex_col()
        .bg(p.card2)
        .child(
            div()
                .h(px(30.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("All projects"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child("Search"),
        )
        .child(
            div()
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .child("New chat"),
        );
    col = match st {
        ScreenState::Loading => col.child(skel(p, 7)),
        ScreenState::Empty => col.child(div().px(px(8.)).text_color(p.mute).child("No chats yet.")),
        ScreenState::Sample => {
            let mut c = col.child(gh(p, "Chats"));
            for t in threads() {
                c = c.child(row(p, t.title, String::new(), t.id == 3));
            }
            c
        }
    };
    div().flex().flex_none().child(rail).child(col)
}

impl RenderOnce for ScreenRailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        frame(
            self.theme,
            self.layout,
            rail_left(&p, self.state),
            page(&p, "Home", placeholder(&p, self.state)),
            Some(workspace(&p, self.state)),
        )
    }
}

screen_el!(
    /// Folders / Overview: folder cards with members, agent access and when they last changed.
    /// Rows a host can route carry `folder-<id>`.
    FoldersEl
);
screen_el!(
    /// Folders / Detail: one folder's files as a table, with its details beside it.
    FolderDetailEl
);

fn faces_el(p: &ThPal, members: &[String]) -> Div {
    let mut d = div().flex().items_center().gap(px(2.));
    for m in members.iter().take(4) {
        d = d.child(
            div()
                .w(px(22.))
                .h(px(22.))
                .rounded(px(11.))
                .bg(p.sel)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(9.5))
                .child(SharedString::from(m.clone())),
        );
    }
    d
}

impl RenderOnce for FoldersEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut body = div().flex().flex_col().gap(px(12.)).p(px(22.)).child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Folders"),
        );
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 6)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "No folders yet",
                    "Make one for a project: files, chats and agents can all share it.",
                ))
            }
            ScreenState::Sample => {
                let mut grid = div().flex().flex_wrap().gap(px(10.));
                for f in folder_view(&folders(), "", false) {
                    const IDS: [&str; 6] = [
                        "folder-1", "folder-2", "folder-3", "folder-4", "folder-5", "folder-6",
                    ];
                    let key = IDS
                        .get((f.id as usize).wrapping_sub(1))
                        .copied()
                        .unwrap_or("folder");
                    let fi = fire(&self.act, key);
                    grid = grid.child(
                        div()
                            .id(SharedString::from(key))
                            .w(px(232.))
                            .p(px(12.))
                            .rounded(px(12.))
                            .border_1()
                            .border_color(p.line)
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(tone_ic(&p, IconTone::Folder))
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(
                                        if f.starred {
                                            format!("★ {}", f.name)
                                        } else {
                                            f.name.clone()
                                        },
                                    )),
                            )
                            .child(div().text_color(p.mute).child(f.note.clone()))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(faces_el(&p, &f.members))
                                    .when(f.agents, |d| {
                                        d.child(div().text_color(p.fg2).child("Agents"))
                                    })
                                    .child(div().flex_1())
                                    .child(div().text_color(p.mute).child(ago(f.mins))),
                            )
                            .on_click(move |_, w, cx| fi(w, cx)),
                    );
                }
                body = body.child(grid);
            }
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Folders", body),
            None,
        )
    }
}

impl RenderOnce for FolderDetailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let f = folders().remove(0);
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(22.))
            .child(div().text_color(p.mute).child("← All folders"))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(f.name.clone()),
            )
            .child(div().text_color(p.mute).child(f.note.clone()));
        let files = if self.state == ScreenState::Sample {
            folder_files()
        } else {
            vec![]
        };
        match self.state {
            ScreenState::Loading => body = body.child(skel(&p, 8)),
            ScreenState::Empty => {
                body = body.child(empty(
                    &p,
                    "Nothing in this folder",
                    "Add a file and everyone it is shared with sees it.",
                ))
            }
            ScreenState::Sample => {
                let mut tb = div()
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .child(row(&p, "Name".into(), "Edited · Size".into(), false));
                for x in &files {
                    tb = tb.child(row_ic(
                        &p,
                        IconTone::for_file(&x.name),
                        format!("{}  ·  {}", x.name, x.kind),
                        format!("{} · {}", ago(x.mins), size_label(x.kb)),
                        false,
                    ));
                }
                body = body.child(tb).child(div().text_color(p.mute).child(format!(
                    "{} files · {}",
                    files.len(),
                    total_size(&files)
                )));
            }
        }
        let mut info = div().flex().flex_col().gap(px(4.)).child(gh(&p, "About"));
        if self.state == ScreenState::Loading {
            info = info.child(skel(&p, 6));
        } else {
            info = info
                .child(div().px(px(8.)).child(f.note.clone()))
                .child(gh(&p, "Shared with"));
            let members = if self.state == ScreenState::Sample {
                f.members.clone()
            } else {
                vec![PERSON_INI.to_string()]
            };
            for m in &members {
                let r = if m == PERSON_INI { "Owner" } else { "Can edit" };
                info = info.child(row(&p, member_name(m).to_string(), r.to_string(), false));
            }
            info = info.child(row(
                &p,
                "Agents can read this folder".into(),
                if f.agents && self.state == ScreenState::Sample {
                    "On".into()
                } else {
                    "Off".into()
                },
                false,
            ));
        }
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Folders", body),
            Some(info),
        )
    }
}

screen_el!(
    /// Integrations: the app directory with connect states and a connected app's permissions.
    /// Cards a host can route carry `app-<brand id>`.
    IntegrationsEl
);

impl RenderOnce for IntegrationsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let list = apps(self.state == ScreenState::Sample);
        let n = connected_count(&list);
        let mut tabs = div().flex().gap(px(14.)).text_color(p.mute);
        for (k, l) in APP_TABS {
            tabs = tabs.child(
                div()
                    .when(k == "all", |d| {
                        d.text_color(p.fg).font_weight(FontWeight::SEMIBOLD)
                    })
                    .child(if k == "on" {
                        format!("{l} {n}")
                    } else {
                        l.to_string()
                    }),
            );
        }
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(22.))
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Integrations"),
            )
            .child(tabs);
        if self.state == ScreenState::Loading {
            body = body.child(skel(&p, 6));
        } else {
            let mut grid = div().flex().flex_wrap().gap(px(10.));
            for a in app_view(&list, "all", "") {
                let key: &'static str = match a.id {
                    "googledrive" => "app-googledrive",
                    "dropbox" => "app-dropbox",
                    "notion" => "app-notion",
                    "slack" => "app-slack",
                    "discord" => "app-discord",
                    "gmail" => "app-gmail",
                    "googlecalendar" => "app-googlecalendar",
                    "zoom" => "app-zoom",
                    "linear" => "app-linear",
                    "figma" => "app-figma",
                    "github" => "app-github",
                    _ => "app-sentry",
                };
                let fi = fire(&self.act, key);
                grid = grid.child(
                    div()
                        .id(SharedString::from(key))
                        .w(px(232.))
                        .p(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(if a.connected { p.fg2 } else { p.line })
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .children(brand_mark(a.id, 20., p.fg))
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(a.name)),
                        )
                        .child(div().text_color(p.fg2).child(a.blurb))
                        .child(div().text_color(p.mute).child(if a.connected {
                            "Connected · Manage"
                        } else {
                            "Connect"
                        }))
                        .on_click(move |_, w, cx| fi(w, cx)),
                );
            }
            body = body.child(grid);
        }
        let work = if self.state == ScreenState::Sample {
            let a = list.iter().find(|a| a.id == "slack").cloned();
            a.map(|a| {
                let mut w = div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(row(&p, a.name.to_string(), synced_words(a.synced), false))
                    .child(gh(&p, "It may"));
                for (s, on) in a.scopes {
                    w = w.child(row(
                        &p,
                        s.to_string(),
                        if on { "On" } else { "Off" }.to_string(),
                        false,
                    ));
                }
                w.child(row(&p, "Sync now".into(), "Disconnect".into(), false))
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Integrations", body),
            work,
        )
    }
}

screen_el!(
    /// Kanban: the board's columns with counts and limits, its cards, and the open card beside
    /// it. Columns a host can route carry `kb-<column key>`.
    KanbanEl
);

impl RenderOnce for KanbanEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let list = board(self.state == ScreenState::Sample);
        let mut body = div().flex().flex_col().gap(px(12.)).p(px(22.)).child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Board"),
        );
        if self.state == ScreenState::Loading {
            body = body.child(skel(&p, 6));
        } else {
            let mut cols = div().flex().gap(px(10.));
            for (k, l, lim) in KB_COLS {
                let n = list.iter().filter(|t| t.col == k).count();
                let id: &'static str = match k {
                    "todo" => "kb-todo",
                    "doing" => "kb-doing",
                    "review" => "kb-review",
                    _ => "kb-done",
                };
                let fi = fire(&self.act, id);
                let mut c = div()
                    .id(SharedString::from(id))
                    .w(px(212.))
                    .p(px(8.))
                    .rounded(px(12.))
                    .bg(p.inset)
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(l))
                            .child(
                                div()
                                    .text_color(if over_limit(&list, k) { p.red } else { p.mute })
                                    .child(if lim > 0 {
                                        format!("{n}/{lim}")
                                    } else {
                                        n.to_string()
                                    }),
                            ),
                    )
                    .on_click(move |_, w, cx| fi(w, cx));
                let cards = column(&list, k, "", None);
                if cards.is_empty() {
                    c = c.child(div().text_color(p.mute).child("Drop cards here"));
                }
                for t in cards {
                    c = c.child(
                        div()
                            .p(px(10.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(if t.id == 11 { p.fg } else { p.line })
                            .bg(p.card)
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(div().text_color(p.mute).child(task_key(t.id)))
                            .child(div().child(t.title.clone()))
                            .child(div().text_color(p.mute).child(match t.due {
                                Some(d) => format!("{} · {}", t.tag, due_words(d)),
                                None => t.tag.to_string(),
                            })),
                    );
                }
                cols = cols.child(c);
            }
            body = body.child(cols);
        }
        let work = if self.state == ScreenState::Sample {
            list.iter().find(|t| t.id == 11).cloned().map(|t| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(row(&p, task_key(t.id), col_label(t.col).to_string(), false))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(t.title.clone()),
                    )
                    .child(row(
                        &p,
                        "Priority".into(),
                        KB_PRI[usize::from(t.pri.min(3))].into(),
                        false,
                    ))
                    .child(row(&p, "Assignee".into(), member_name(t.who).into(), false))
                    .child(row(
                        &p,
                        "Sub-tasks".into(),
                        format!("{} of {} done", t.sub.0, t.sub.1),
                        false,
                    ))
                    .child(gh(&p, "Notes"))
                    .child(div().text_color(p.fg2).child(t.note))
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Autumn launch / Board", body),
            work,
        )
    }
}

screen_el!(
    /// CRM: the saved views as tabs with counts, the deals table (company and contact, stage,
    /// value, owner, close) and the open deal beside it. Views a host can route carry
    /// `crm-<view key>`.
    CrmEl
);

impl RenderOnce for CrmEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let list = deals(self.state == ScreenState::Sample);
        let mut body = div().flex().flex_col().gap(px(12.)).p(px(22.)).child(
            div()
                .text_size(px(22.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Deals"),
        );
        let mut tabs = div().flex().gap(px(4.));
        for (k, l) in CRM_VIEWS {
            let id: &'static str = match k {
                "mine" => "crm-mine",
                "soon" => "crm-soon",
                "won" => "crm-won",
                _ => "crm-all",
            };
            let fi = fire(&self.act, id);
            let on = k == "all";
            tabs = tabs.child(
                div()
                    .id(SharedString::from(id))
                    .px(px(10.))
                    .py(px(6.))
                    .border_b_2()
                    .border_color(if on { p.fg } else { p.line })
                    .text_color(if on { p.fg } else { p.mute })
                    .child(format!("{l} {}", view_count(&list, k)))
                    .on_click(move |_, w, cx| fi(w, cx)),
            );
        }
        body = body.child(tabs);
        if self.state == ScreenState::Loading {
            body = body.child(skel(&p, 6));
        } else if list.is_empty() {
            body = body.child(div().text_color(p.mute).child("No deals yet"));
        } else {
            let mut tb = div()
                .flex()
                .flex_col()
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.card);
            let cell = |t: String, w: f32, c| div().w(px(w)).text_color(c).child(t);
            tb = tb.child(
                div()
                    .flex()
                    .px(px(10.))
                    .py(px(8.))
                    .text_color(p.mute)
                    .child(div().flex_1().child("Company"))
                    .child(cell("Stage".into(), 100., p.mute))
                    .child(cell("Value".into(), 90., p.mute))
                    .child(cell("Owner".into(), 56., p.mute))
                    .child(cell("Close".into(), 110., p.mute)),
            );
            for d in deal_view(&list, "all", "", DealSort::Value, false) {
                tb = tb.child(
                    div()
                        .flex()
                        .items_center()
                        .px(px(10.))
                        .py(px(8.))
                        .border_t_1()
                        .border_color(p.line)
                        .bg(if d.id == 1 { p.sel } else { p.card })
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(div().child(d.company.clone()))
                                .child(div().text_color(p.mute).child(d.contact)),
                        )
                        .child(cell(stage_label(d.stage).into(), 100., p.fg))
                        .child(cell(usd(d.value), 90., p.fg))
                        .child(cell(d.owner.to_string(), 56., p.fg2))
                        .child(cell(close_words(d.close, d.stage), 110., p.mute)),
                );
            }
            body = body.child(tb);
        }
        let work = if self.state == ScreenState::Sample {
            list.iter().find(|d| d.id == 1).cloned().map(|d| {
                let mut w = div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(d.company.clone()),
                    )
                    .child(row(&p, "Contact".into(), d.contact.into(), false))
                    .child(row(&p, "Stage".into(), stage_label(d.stage).into(), false))
                    .child(row(&p, "Value".into(), usd(d.value), false))
                    .child(row(&p, "Owner".into(), member_name(d.owner).into(), false))
                    .child(gh(&p, "Notes"))
                    .child(div().text_color(p.fg2).child(d.note))
                    .child(gh(&p, "Pipeline"));
                for (k, v, n) in stage_totals(&list) {
                    w = w.child(row(
                        &p,
                        stage_label(k).into(),
                        format!("{} · {n}", usd(v)),
                        false,
                    ));
                }
                w
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Sales / Deals", body),
            work,
        )
    }
}

screen_el!(
    /// Workflow builder: node palette, the canvas with each node placed where the model puts it,
    /// and the inspector for the selected node. Palette rows fire `wf-<kind>`.
    WorkflowEl
);

impl RenderOnce for WorkflowEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        use oxyn_core::parity::screensfid::{
            WF_NODE_H, WF_NODE_W, WF_PALETTE, wf_kind_label, workflow,
        };
        let p = th_pal(self.theme);
        let (nodes, edges) = workflow(self.state == ScreenState::Sample);
        let mut pal = div()
            .w(px(180.))
            .flex_none()
            .flex()
            .flex_col()
            .p(px(8.))
            .border_r_1()
            .border_color(p.line)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .px(px(8.))
                    .child("Add nodes"),
            );
        if self.state == ScreenState::Loading {
            pal = pal.child(skel(&p, 8));
        } else {
            for (g, kind, xs) in WF_PALETTE {
                let id: &'static str = match kind {
                    "trigger" => "wf-trigger",
                    "tool" => "wf-tool",
                    "logic" => "wf-logic",
                    _ => "wf-action",
                };
                pal = pal.child(gh(&p, g));
                for (t, _) in xs.iter() {
                    let fi = fire(&self.act, id);
                    pal = pal.child(
                        div()
                            .id(SharedString::from(format!("{id}-{t}")))
                            .px(px(8.))
                            .py(px(5.))
                            .rounded(px(8.))
                            .text_color(p.fg2)
                            .child(*t)
                            .on_click(move |_, w, cx| fi(w, cx)),
                    );
                }
            }
        }
        let scale = 0.62;
        let mut cv = div()
            .relative()
            .flex_1()
            .h(px(560.))
            .bg(p.inset)
            .overflow_hidden();
        if self.state == ScreenState::Loading {
            cv = cv.child(skel(&p, 4));
        } else if nodes.is_empty() {
            cv = cv.child(empty(
                &p,
                "Start with a trigger",
                "Add one from the palette",
            ));
        }
        for n in &nodes {
            let on = n.id == 5;
            let outs = edges.iter().filter(|(a, _)| *a == n.id).count();
            cv = cv.child(
                div()
                    .absolute()
                    .left(px(n.x as f32 * scale + 12.))
                    .top(px(n.y as f32 * scale + 12.))
                    .w(px(WF_NODE_W as f32 * scale + 20.))
                    .h(px(WF_NODE_H as f32 * scale + 12.))
                    .p(px(6.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(if on {
                        p.fg
                    } else if n.ready() {
                        p.line
                    } else {
                        p.red
                    })
                    .bg(p.card)
                    .flex()
                    .flex_col()
                    .text_size(px(11.5))
                    .child(div().text_color(p.mute).child(format!(
                        "{}{}",
                        wf_kind_label(n.kind),
                        if outs > 1 { " · branches" } else { "" }
                    )))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(n.title.clone()),
                    )
                    .child(
                        div()
                            .text_color(if n.ready() { p.fg2 } else { p.red })
                            .child(if n.ready() {
                                n.value.clone()
                            } else {
                                format!("{} not set", n.ask)
                            }),
                    ),
            );
        }
        let body = div().flex().child(pal).child(cv);
        let work = if self.state == ScreenState::Sample {
            nodes.iter().find(|n| n.id == 5).map(|n| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(div().text_color(p.mute).child(wf_kind_label(n.kind)))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(n.title.clone()),
                    )
                    .child(row(&p, n.ask.into(), "Not set".into(), false))
                    .child(row(&p, "Comes from".into(), "Branch".into(), false))
                    .child(row(&p, "Goes to".into(), "Save a row".into(), false))
            })
        } else {
            None
        };
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "Flows / Member welcome", body),
            work,
        )
    }
}

screen_el!(
    /// Settings shell: the settings sidebar (Back to app, Personal and Workspace pages) and the
    /// Notifications page's rows. Pages fire `set-<page>`.
    SettingsEl
);

impl RenderOnce for SettingsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        use oxyn_core::parity::screensfid::{SET_GROUPS, SetCtl, settings};
        let p = th_pal(self.theme);
        let pages = settings(self.state != ScreenState::Empty);
        let mut left = div()
            .w(px(232.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(px(10.))
            .child(div().text_color(p.fg2).child("← Back to app"))
            .child(
                div()
                    .mt(px(8.))
                    .px(px(8.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Settings"),
            );
        if self.state == ScreenState::Loading {
            left = left.child(skel(&p, 8));
        } else {
            for (g, gl) in SET_GROUPS {
                left = left.child(gh(&p, gl));
                for pg in pages.iter().filter(|x| x.group == g) {
                    let id: &'static str = match pg.key {
                        "profile" => "set-profile",
                        "preferences" => "set-preferences",
                        "notifications" => "set-notifications",
                        "privacy" => "set-privacy",
                        "general" => "set-general",
                        "members" => "set-members",
                        "models" => "set-models",
                        "security" => "set-security",
                        _ => "set-billing",
                    };
                    let fi = fire(&self.act, id);
                    let on = pg.key == "notifications";
                    left = left.child(
                        div()
                            .id(SharedString::from(id))
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(8.))
                            .bg(if on { p.sel } else { p.inset })
                            .text_color(if on { p.fg } else { p.fg2 })
                            .child(pg.label)
                            .on_click(move |_, w, cx| fi(w, cx)),
                    );
                }
            }
        }
        left = left
            .child(div().flex_1())
            .child(row(&p, PERSON.into(), PERSON_INI.into(), false));
        let left = left.bg(p.inset);
        let mut body = div().flex().flex_col().gap(px(10.)).p(px(24.)).w(px(620.));
        if self.state == ScreenState::Loading {
            body = body.child(skel(&p, 8));
        } else if let Some(pg) = pages.iter().find(|x| x.key == "notifications") {
            body = body
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(pg.label),
                )
                .child(div().text_color(p.mute).child(pg.lead));
            for (sec, rows) in &pg.sections {
                body = body.child(gh(&p, sec));
                let mut card = div()
                    .flex()
                    .flex_col()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card);
                for r in rows {
                    let val = match &r.ctl {
                        SetCtl::Switch(v) => if *v { "On" } else { "Off" }.to_string(),
                        SetCtl::Choice(o, i) => o.get(*i).copied().unwrap_or("").to_string(),
                        SetCtl::Text(v, _) => v.clone(),
                        SetCtl::Action(l) | SetCtl::Danger(l) => l.to_string(),
                    };
                    card = card.child(
                        div()
                            .flex()
                            .items_center()
                            .px(px(12.))
                            .py(px(10.))
                            .border_t_1()
                            .border_color(p.line)
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .child(div().child(r.label))
                                    .child(div().text_color(p.mute).child(r.hint)),
                            )
                            .child(div().text_color(p.fg2).child(val)),
                    );
                }
                body = body.child(card);
            }
        }
        frame(
            self.theme,
            self.layout,
            left,
            page(&p, "Settings / Alerts", body),
            None,
        )
    }
}
