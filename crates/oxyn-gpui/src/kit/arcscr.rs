//! Screens S12 (Arc screens I) in GPUI: [`MetricsDashboardEl`], [`KanbanBoardEl`],
//! [`ProjectBoardEl`], [`GanttTimelineEl`] and [`TeamDirectoryEl`] on the Arc frame (248 px
//! sidebar with a context card and an account row with the email, an inset page, an optional
//! right card), over `oxyn_core::kit::arcscr`.
//!
//! Each element draws one state (`.state()`, `.extra()`); with `.arc(ArcUi)` it draws that live
//! state instead of the sample, and its presses (`mx-*`, `kb-*`, `pb-*`, `gt-*`, `td-*`) go back
//! through `on_act` for the host to apply with [`oxyn_core::kit::arcscr::ArcUi::press`]. Board
//! cards and timeline bars can be dragged: a card onto a column, a bar onto a day.

use gpui::{
    AnyView, App, Context, Div, FontWeight, Hsla, IntoElement, ParentElement, Render, RenderOnce,
    SharedString, Stateful, Styled, Window, div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::arcscr::{
    ACTIVITY, ArcUi, Avail, Col, GT_NAV, GT_VIEWS, HUES, Health, KB_NAV, KB_PROJECTS, KPIS, LABELS,
    LINKS, MILESTONES, MX_NAV, MX_VIEWS, OFFICES, ONLINE, PB_NAV, PB_PROJECTS, PEOPLE, PHASES,
    PLAN, PRIOS, PROJECT, RANGES, STATUSES, ScreenState, TABLES, TD_NAV, TEAM, TEAMS, TODAY, TipAt,
    ZOOMS, bar_dates, col_counts, column, conflicts, end_of, find_people, ids, initials, iso,
    issue_key, issues, kpi_value, load_of, local_time, members, month_day, ms_done, ms_progress,
    my_email, nice_top, office_count, office_open, phase_span, plan, plan_done, point_day, points,
    range_span, series, stats, table, tasks, thousands, tip_at, tip_w, ws,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::hit;
use super::chatscr::{clip, mark};
use super::rulerfid::{ThPal, th_pal};

/// A card or bar being dragged: what it is (the press that grabs it) and its label.
#[derive(Clone)]
pub struct ArcDrag {
    /// The press that picks it up (`kb-grab-<n>`, `gt-grab-<id>`).
    pub grab: &'static str,
    /// Its label, drawn under the pointer.
    pub label: SharedString,
}

impl Render for ArcDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.))
            .py(px(5.))
            .rounded(px(8.))
            .bg(rgb(0x2A2A2E))
            .text_color(rgb(0xF4F4F5))
            .text_size(px(12.))
            .shadow_lg()
            .child(self.label.clone())
    }
}

macro_rules! arc_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
            arc: Option<ArcUi>,
            search: Option<AnyView>,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK, arc: None, search: None }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state (see `oxyn_core::kit::arcscr::s12_states`).
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
            /// Press ids (`nav-<key>`, `acct`, and the screen's own `mx-*`, `kb-*`, `pb-*`,
            /// `gt-*`, `td-*`).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
            /// Draw this live state instead of the sample.
            pub fn arc(mut self, a: ArcUi) -> Self {
                self.arc = Some(a);
                self
            }
            /// A text field for the screen's search (the Team directory's).
            pub fn search(mut self, v: AnyView) -> Self {
                self.search = Some(v);
                self
            }
        }
    };
}

arc_el!(
    /// Metrics dashboard: range, Export, KPI tabs over the chart (Compare, zoom), breakdowns.
    MetricsDashboardEl
);
arc_el!(
    /// Kanban board: the cycle's columns; press a card then a column (or drag it), Alt + arrows.
    KanbanBoardEl
);
arc_el!(
    /// Project board: status, stats, milestones, tasks to complete, the team and links card.
    ProjectBoardEl
);
arc_el!(
    /// Gantt timeline: phases and bars; press a bar then nudge it, or drag it onto a day.
    GanttTimelineEl
);
arc_el!(
    /// Team directory: people by team with local times; search, filters, favourites, views.
    TeamDirectoryEl
);

pub(super) fn hue(p: &ThPal, h: &str) -> Hsla {
    let dark = p.fg.l > 0.5;
    let x = match (h, dark) {
        ("teal", true) => 0x3CCBBF,
        ("amber", true) => 0xF5B341,
        ("violet", true) => 0xA48AFB,
        ("blue", true) => 0x5B9BFF,
        ("green", true) => 0x3FCB7E,
        ("rose", true) => 0xFF6B8A,
        ("magenta", true) => 0xF06BC0,
        ("teal", false) => 0x0F8F84,
        ("amber", false) => 0xB76E00,
        ("violet", false) => 0x6E4FE8,
        ("blue", false) => 0x1F6FEB,
        ("green", false) => 0x1A7F4B,
        ("rose", false) => 0xD92D5B,
        ("magenta", false) => 0xC11574,
        _ => return p.mute,
    };
    rgb(x).into()
}

pub(super) fn tint(p: &ThPal, h: &str, a: f32) -> Hsla {
    let mut c = hue(p, h);
    c.a = a;
    c
}

pub(super) fn q(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .text_size(px(12.))
        .text_color(p.mute)
        .child(t.into())
}

pub(super) fn av(p: &ThPal, who: usize, size: f32) -> Div {
    let h = HUES[PEOPLE[who % PEOPLE.len()].1];
    div()
        .flex_none()
        .size(px(size))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(tint(p, h, 0.22))
        .text_color(hue(p, h))
        .text_size(px(if size < 26. { 9. } else { 11. }))
        .font_weight(FontWeight::SEMIBOLD)
        .child(initials(who))
}

pub(super) fn dot(c: Hsla, s: f32) -> Div {
    div().flex_none().size(px(s)).rounded_full().bg(c)
}

pub(super) fn btn(
    p: &ThPal,
    act: &Option<Act>,
    id: &'static str,
    l: impl Into<SharedString>,
    pri: bool,
) -> Stateful<Div> {
    hit(div(), act, id)
        .flex_none()
        .h(px(30.))
        .px(px(12.))
        .flex()
        .items_center()
        .rounded(px(8.))
        .border_1()
        .border_color(if pri { p.fg } else { p.line })
        .bg(if pri { p.fg } else { p.card })
        .text_color(if pri { p.card } else { p.fg })
        .font_weight(FontWeight::MEDIUM)
        .text_size(px(12.5))
        .whitespace_nowrap()
        .child(l.into())
}

pub(super) fn chip(
    p: &ThPal,
    act: &Option<Act>,
    id: &'static str,
    l: impl Into<SharedString>,
    on: bool,
) -> Stateful<Div> {
    hit(div(), act, id)
        .flex_none()
        .h(px(26.))
        .px(px(10.))
        .flex()
        .items_center()
        .rounded(px(7.))
        .text_size(px(12.5))
        .whitespace_nowrap()
        .text_color(if on { p.fg } else { p.mute })
        .when(on, |d| d.bg(p.card).border_1().border_color(p.line))
        .child(l.into())
}

pub(super) fn seg(p: &ThPal, items: Vec<Stateful<Div>>) -> Div {
    div()
        .flex_none()
        .flex()
        .gap(px(2.))
        .p(px(2.))
        .rounded(px(9.))
        .bg(p.sel)
        .children(items)
}

pub(super) fn nrow(
    p: &ThPal,
    act: &Option<Act>,
    id: &'static str,
    icon: Div,
    l: impl Into<SharedString>,
    n: impl Into<SharedString>,
    on: bool,
) -> Stateful<Div> {
    hit(div(), act, id)
        .flex_none()
        .h(px(32.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(9.))
        .rounded(px(8.))
        .text_color(if on { p.fg } else { p.fg2 })
        .when(on, |d| d.bg(p.sel))
        .child(icon)
        .child(clip(l.into()))
        .child(q(p, n.into()))
}

pub(super) fn glyph(p: &ThPal, t: &'static str) -> Div {
    div()
        .flex_none()
        .w(px(16.))
        .flex()
        .justify_center()
        .text_size(px(12.))
        .text_color(p.mute)
        .child(t)
}

pub(super) fn icon_for(k: &str) -> &'static str {
    match k {
        "grid" => "▦",
        "file" => "▤",
        "route" => "⤳",
        "globe" => "◍",
        "laptop" => "▭",
        "target" => "◎",
        "cal" => "▦",
        "user" => "◉",
        "cycle" => "↻",
        "layers" => "≡",
        "home" => "⌂",
        "tasks" => "☰",
        "inbox" => "▣",
        "chart" => "▬",
        "list" => "☰",
        "clock" => "◷",
        "flash" => "ϟ",
        "people" => "◉",
        "tree" => "⋔",
        "mail" => "✉",
        "link" => "↗",
        "check" => "✓",
        "bell" => "◔",
        "doc" => "▤",
        "web" => "◍",
        "star" => "☆",
        "card" => "▭",
        "plug" => "⌁",
        "bug" => "✱",
        "term" => "›",
        _ => "•",
    }
}

pub(super) fn group(p: &ThPal, l: &'static str) -> Div {
    div()
        .flex_none()
        .mt(px(10.))
        .h(px(28.))
        .px(px(8.))
        .flex()
        .items_center()
        .text_size(px(12.))
        .text_color(p.mute)
        .child(l)
}

pub(super) fn skel(p: &ThPal, n: usize) -> Div {
    let mut d = div().flex().flex_col().gap(px(10.)).p(px(12.));
    for i in 0..n {
        d = d.child(
            div()
                .h(px(11.))
                .w(relative([0.9, 0.6, 0.75, 0.45, 0.7][i % 5]))
                .rounded(px(6.))
                .bg(p.sel),
        );
    }
    d
}

pub(super) fn none(p: &ThPal, t: &'static str, l: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.))
        .py(px(64.))
        .child(div().size(px(40.)).rounded(px(12.)).bg(p.sel))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(15.))
                .child(t),
        )
        .child(div().text_color(p.mute).child(l))
}

pub(super) fn card(p: &ThPal) -> Div {
    div()
        .rounded(px(12.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
}

pub(super) struct Frame {
    pub(super) slug: &'static str,
    pub(super) side: Div,
    pub(super) ctx: Option<Div>,
    pub(super) crumbs: Vec<String>,
    pub(super) meta: String,
    pub(super) actions: Div,
    pub(super) bar: Option<Div>,
    pub(super) body: Div,
    pub(super) foot: Option<Div>,
    pub(super) right: Option<Div>,
}

pub(super) fn frame(p: &ThPal, lay: ShellLayout, act: &Option<Act>, f: Frame) -> Div {
    let w = ws(f.slug);
    let flat = lay == ShellLayout::Flat;
    let mut crumbs = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .min_w_0()
        .overflow_hidden();
    let last = f.crumbs.len().saturating_sub(1);
    for (i, c) in f.crumbs.into_iter().enumerate() {
        if i > 0 {
            crumbs = crumbs.child(q(p, "›"));
        }
        crumbs = crumbs.child(
            div()
                .whitespace_nowrap()
                .text_color(if i == last { p.fg } else { p.mute })
                .when(i == last, |d| d.font_weight(FontWeight::MEDIUM))
                .child(c),
        );
    }
    let side = div()
        .w(px(248.))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(8.))
        .pt(px(10.))
        .bg(p.inset)
        .child(
            hit(div(), act, "workspace")
                .flex_none()
                .h(px(34.))
                .px(px(6.))
                .flex()
                .items_center()
                .gap(px(9.))
                .rounded(px(8.))
                .child(div().size(px(22.)).rounded(px(6.)).bg(p.fg))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(w.name))
                .child(q(p, "⌄")),
        )
        .child(
            hit(div(), act, "search")
                .flex_none()
                .h(px(32.))
                .mt(px(6.))
                .mb(px(10.))
                .px(px(9.))
                .flex()
                .items_center()
                .gap(px(8.))
                .rounded(px(8.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .text_color(p.mute)
                .child(if f.slug == "team-directory" {
                    "Find a person"
                } else {
                    "Search"
                })
                .child(div().flex_1())
                .child(q(p, "⌘K")),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .gap(px(1.))
                .overflow_hidden()
                .child(f.side),
        )
        .children(f.ctx.map(|c| {
            div()
                .flex_none()
                .border_t_1()
                .border_color(p.line)
                .px(px(8.))
                .pt(px(10.))
                .pb(px(6.))
                .child(c)
        }))
        .child(
            hit(div(), act, "acct")
                .flex_none()
                .h(px(48.))
                .px(px(6.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(10.))
                .child(
                    div()
                        .size(px(30.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(tint(p, "blue", 0.22))
                        .text_color(hue(p, "blue"))
                        .text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(oxyn_core::kit::arcscr::ME_INI),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_color(p.fg)
                                .font_weight(FontWeight::MEDIUM)
                                .child(oxyn_core::kit::arcscr::ME),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(p.mute)
                                .child(my_email(f.slug)),
                        ),
                )
                .child(q(p, "⇅")),
        );
    let head = div()
        .flex_none()
        .h(px(53.))
        .pl(px(20.))
        .pr(px(12.))
        .flex()
        .items_center()
        .gap(px(10.))
        .border_b_1()
        .border_color(p.line)
        .child(crumbs)
        .when(!f.meta.is_empty(), |d| d.child(q(p, f.meta.clone())))
        .child(div().flex_1())
        .child(f.actions.flex_none().flex().items_center().gap(px(8.)));
    let page = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .bg(p.card)
        .when(!flat, |d| {
            d.m(px(8.))
                .ml_0()
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
        })
        .when(flat, |d| d.border_l_1().border_color(p.line))
        .child(head)
        .children(f.bar.map(|b| {
            b.flex_none()
                .min_h(px(50.))
                .pl(px(20.))
                .pr(px(12.))
                .py(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .border_b_1()
                .border_color(p.line)
        }))
        .child(div().flex_1().min_h_0().overflow_hidden().child(f.body))
        .children(f.foot.map(|b| {
            b.flex_none()
                .h(px(46.))
                .pl(px(20.))
                .pr(px(12.))
                .flex()
                .items_center()
                .gap(px(8.))
                .border_t_1()
                .border_color(p.line)
        }));
    div()
        .size_full()
        .flex()
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.))
        .child(side)
        .child(page)
        .children(f.right.map(|r| {
            div()
                .w(px(328.))
                .flex_none()
                .overflow_hidden()
                .bg(p.card)
                .when(!flat, |d| {
                    d.m(px(8.))
                        .ml_0()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                })
                .when(flat, |d| d.border_l_1().border_color(p.line))
                .child(r)
        }))
}

fn live(arc: &Option<ArcUi>) -> ArcUi {
    arc.clone().unwrap_or_default()
}

// ───────────────────────────────────────────────────────────── Metrics ──

impl RenderOnce for MetricsDashboardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let u = live(&self.arc);
        let st = self.state;
        let empty = st == ScreenState::Empty;
        let r = u.range;
        let compare = u.compare || self.extra == "compare";
        let zoom = u.zoom.or(if self.extra == "zoomed" {
            Some((8, 21))
        } else {
            None
        });
        let exported = u.exported || self.extra == "exported";
        let (a, z) = range_span(r);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (k, ic, l, n)) in MX_NAV.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                [
                    "nav-overview",
                    "nav-pages",
                    "nav-sources",
                    "nav-locations",
                    "nav-devices",
                    "nav-goals",
                ][i],
                glyph(&p, icon_for(ic)),
                l,
                n,
                k == "overview",
            ));
        }
        side = side.child(group(&p, "Saved views"));
        for (i, (ic, l, rk)) in MX_VIEWS.iter().copied().enumerate() {
            let icon = if ic == "google" {
                div()
                    .flex_none()
                    .w(px(16.))
                    .children(mark(&p, "google", 14.))
            } else {
                glyph(&p, icon_for(ic))
            };
            let ri = RANGES.iter().position(|x| x.0 == rk).unwrap_or(1);
            side = side.child(nrow(
                &p,
                &act,
                ids::at(ids::MX_VIEW, i),
                icon,
                l,
                rk,
                u.view == Some(i) && u.range == ri,
            ));
        }
        let mut spark = div().flex().items_end().gap(px(2.)).h(px(26.)).my(px(7.));
        for (i, v) in ONLINE.0.iter().enumerate() {
            let last = i == ONLINE.0.len() - 1;
            spark = spark.child(
                div()
                    .flex_1()
                    .h(px(*v as f32 * 26. / 17.))
                    .rounded(px(2.))
                    .bg(if last {
                        hue(&p, "blue")
                    } else {
                        tint(&p, "blue", 0.42)
                    }),
            );
        }
        let ctx = div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().text_color(hue(&p, "green")).child("∿"))
                    .child(div().flex_1().child("Online now"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(ONLINE.1.to_string()),
                    ),
            )
            .child(spark)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(11.5))
                    .text_color(p.mute)
                    .child("30 min ago")
                    .child("Now"),
            );
        let mut ranges = Vec::new();
        for (i, (_, l, _)) in RANGES.iter().copied().enumerate() {
            ranges.push(chip(&p, &act, ids::at(ids::MX_RANGE, i), l, i == r));
        }
        let actions = div().child(seg(&p, ranges)).child(btn(
            &p,
            &act,
            "mx-export",
            if exported { "✓ Exported" } else { "Export" },
            false,
        ));
        let bar = div()
            .child(btn(&p, &act, "filter", "Filter", false))
            .child(div().flex_1())
            .child(q(
                &p,
                if exported {
                    format!("metrics-{a}-to-{z}.csv · {a} to {z}")
                } else {
                    format!("{a} to {z}")
                },
            ));
        let body = if st == ScreenState::Loading {
            div().p(px(20.)).child(card(&p).child(skel(&p, 12)))
        } else {
            let mut kpis = card(&p).flex().overflow_hidden();
            for (i, (_, l)) in KPIS.iter().copied().enumerate() {
                let (v, ch, good) = if empty {
                    ("0".to_string(), String::new(), true)
                } else {
                    kpi_value(i, r)
                };
                let on = u.kpi == i;
                let up = ch.starts_with('+');
                kpis = kpis.child(
                    hit(div(), &act, ids::at(ids::MX_KPI, i))
                        .flex_1()
                        .min_w_0()
                        .px(px(18.))
                        .py(px(14.))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .when(i < 4, |d| d.border_r_1().border_color(p.line))
                        .when(on, |d| d.bg(p.sel))
                        .child(div().text_color(p.fg2).child(l))
                        .child(
                            div()
                                .text_size(px(26.))
                                .font_weight(FontWeight::MEDIUM)
                                .child(v),
                        )
                        .when(!ch.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(hue(&p, if good { "green" } else { "rose" }))
                                    .child(format!(
                                        "{} {}",
                                        if up { "↗" } else { "↘" },
                                        ch.trim_start_matches(['+', '-'])
                                    )),
                            )
                        }),
                );
            }
            let chart = if empty {
                none(
                    &p,
                    "No visits yet",
                    "Add the tracking snippet to your site; visits show here within a minute.",
                )
            } else {
                let full = series(u.kpi, r, false);
                let fullp = series(u.kpi, r, true);
                let (lo, hi) = zoom.unwrap_or((0, full.len() - 1));
                let (lo, hi) = (lo.min(full.len() - 1), hi.min(full.len() - 1));
                let cur = &full[lo..=hi];
                let prev = &fullp[lo..=hi];
                let mut max = cur.iter().cloned().fold(0.0, f64::max);
                if compare {
                    max = prev.iter().cloned().fold(max, f64::max);
                }
                let (top, _) = nice_top(max);
                let mut cols = div()
                    .flex_1()
                    .h(px(230.))
                    .flex()
                    .items_end()
                    .gap(px(if cur.len() > 60 { 1. } else { 3. }))
                    .border_b_1()
                    .border_color(p.line);
                for (i, v) in cur.iter().enumerate() {
                    let hcur = (*v / top) as f32;
                    let hp = (prev[i] / top) as f32;
                    cols = cols.child(
                        div()
                            .flex_1()
                            .h_full()
                            .relative()
                            .when(compare, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .bottom_0()
                                        .left_0()
                                        .right_0()
                                        .h(relative(hp))
                                        .border_t_2()
                                        .border_color(p.mute),
                                )
                            })
                            .child(
                                div()
                                    .absolute()
                                    .bottom_0()
                                    .left_0()
                                    .right_0()
                                    .h(relative(hcur))
                                    .rounded_t(px(2.))
                                    .bg(tint(&p, "blue", 0.55)),
                            ),
                    );
                }
                let n = cur.len();
                let mut xl = div()
                    .flex()
                    .justify_between()
                    .pl(px(48.))
                    .text_size(px(11.5))
                    .text_color(p.mute);
                for j in 0..6.min(n) {
                    let i = if n < 2 {
                        0
                    } else {
                        j * (n - 1) / 5.min(n - 1).max(1)
                    };
                    xl = xl.child(iso(point_day(r, lo + i.min(n - 1))));
                }
                let mut yl = div()
                    .w(px(40.))
                    .h(px(230.))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .text_size(px(11.5))
                    .text_color(p.mute);
                for k in (0..=4).rev() {
                    yl = yl.child(div().flex().justify_end().child(
                        oxyn_core::kit::arcscr::axis_value(u.kpi, top * k as f64 / 4.0),
                    ));
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(14.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .child(div().w(px(14.)).h(px(2.)).bg(hue(&p, "blue")))
                                    .child(KPIS[u.kpi].1),
                            )
                            .when(compare, |d| d.child(q(&p, "– – Previous period")))
                            .child(div().flex_1())
                            .child(if zoom.is_some() {
                                btn(&p, &act, "mx-unzoom", "Reset zoom", false)
                            } else {
                                btn(&p, &act, "mx-zoom", "Zoom in", false)
                            })
                            .child(chip(
                                &p,
                                &act,
                                "mx-compare",
                                if compare {
                                    "Compare ●"
                                } else {
                                    "Compare ○"
                                },
                                compare,
                            )),
                    )
                    .child(div().flex().gap(px(8.)).child(yl).child(cols))
                    .child(xl)
            };
            let mut tbls = div().flex().gap(px(16.));
            for (t, (_, ta, _)) in TABLES.iter().copied().enumerate() {
                let rows = if empty { vec![] } else { table(t, false, r) };
                let max = rows.iter().map(|x| x.n).max().unwrap_or(1).max(1) as f32;
                let mut c = card(&p)
                    .flex_1()
                    .min_w_0()
                    .p(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .mb(px(6.))
                            .child(
                                div()
                                    .px(px(10.))
                                    .py(px(4.))
                                    .rounded(px(7.))
                                    .bg(p.sel)
                                    .child(ta),
                            )
                            .child(div().flex_1())
                            .child(q(&p, "Visitors")),
                    );
                if rows.is_empty() {
                    c = c.child(div().p(px(10.)).text_color(p.mute).child("Nothing yet"));
                }
                for x in rows {
                    let icon = match x.icon {
                        "" | "link" | "tag" => None,
                        cc if cc.len() == 2 => Some(
                            div()
                                .w(px(20.))
                                .flex_none()
                                .text_size(px(10.5))
                                .text_color(p.mute)
                                .child(cc)
                                .into_any_element(),
                        ),
                        b => mark(&p, b, 14.),
                    };
                    c = c.child(
                        div()
                            .relative()
                            .h(px(30.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .bottom_0()
                                    .w(relative(x.n as f32 / max))
                                    .rounded(px(6.))
                                    .bg(tint(&p, "blue", 0.13)),
                            )
                            .children(icon)
                            .child(clip(x.label))
                            .child(div().flex_none().child(thousands(x.n))),
                    );
                }
                tbls = tbls.child(c);
            }
            div()
                .p(px(20.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(kpis)
                .child(card(&p).px(px(18.)).py(px(14.)).child(chart))
                .child(tbls)
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "metrics-dashboard",
                side,
                ctx: Some(ctx),
                crumbs: vec!["Lumenfield".into(), "Overview".into()],
                meta: String::new(),
                actions,
                bar: Some(bar),
                body,
                foot: None,
                right: None,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── Kanban ──

fn prio(p: &ThPal, n: usize) -> Div {
    let (_, k) = PRIOS[n.min(4)];
    if k == 4 {
        return div()
            .flex_none()
            .size(px(14.))
            .rounded(px(4.))
            .bg(hue(p, "rose"))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(10.))
            .text_color(gpui::white())
            .child("!");
    }
    let mut d = div().flex_none().flex().items_end().gap(px(1.5)).h(px(12.));
    for i in 0..3u8 {
        d = d.child(
            div()
                .w(px(2.6))
                .h(px(4. + i as f32 * 3.))
                .rounded(px(1.))
                .bg(if i < k { p.fg2 } else { p.tick }),
        );
    }
    d
}

fn col_mark(p: &ThPal, c: Col) -> Div {
    let (t, h) = match c {
        Col::Backlog => ("◌", "mute"),
        Col::Todo => ("○", "mute"),
        Col::Progress => ("◐", "amber"),
        Col::Review => ("◕", "blue"),
        Col::Done => ("●", "green"),
    };
    div().flex_none().text_color(hue(p, h)).child(t)
}

fn label_chip(p: &ThPal, l: usize) -> Div {
    let (_, n, h) = LABELS[l.min(5)];
    div()
        .flex()
        .items_center()
        .gap(px(5.))
        .h(px(22.))
        .px(px(7.))
        .rounded(px(6.))
        .border_1()
        .border_color(p.line)
        .text_size(px(11.5))
        .text_color(p.fg2)
        .child(dot(hue(p, h), 7.))
        .child(n)
}

impl RenderOnce for KanbanBoardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() && st != ScreenState::Sample {
            u.board = issues(false);
        }
        if self.extra == "filtered" && u.label.is_none() {
            u.label = Some(1);
        }
        let by_person = u.by_person || self.extra == "assignee";
        let lifted = if self.extra == "dragging" {
            Some(214)
        } else {
            u.pick
        };
        let b = &u.board;
        let (done, total) = points(b);
        let counts = col_counts(b);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, ic, l, n)) in KB_NAV.iter().copied().enumerate() {
            let c = if i == 1 {
                b.len().to_string()
            } else {
                n.to_string()
            };
            side = side.child(nrow(
                &p,
                &act,
                ["nav-myissues", "nav-cycle", "nav-allissues"][i],
                glyph(&p, icon_for(ic)),
                l,
                c,
                i == 1,
            ));
        }
        side = side.child(group(&p, "Projects"));
        for (i, (_, l, n, h)) in KB_PROJECTS.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ["nav-storefront", "nav-mobile", "nav-payments", "nav-design"][i],
                div()
                    .w(px(16.))
                    .flex()
                    .justify_center()
                    .child(dot(hue(&p, h), 8.)),
                l,
                n,
                i == 0,
            ));
        }
        let pct = (done * 100).checked_div(total).unwrap_or(0) as f32 / 100.;
        let ctx = div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().text_color(hue(&p, "blue")).child("↻"))
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(oxyn_core::kit::arcscr::CYCLE.0),
                    )
                    .child(q(
                        &p,
                        format!("{} days left", oxyn_core::kit::arcscr::CYCLE.2 - TODAY),
                    )),
            )
            .child(
                div()
                    .h(px(4.))
                    .my(px(8.))
                    .rounded(px(2.))
                    .bg(p.sel)
                    .child(div().h_full().w(relative(pct)).rounded(px(2.)).bg(p.fg)),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(11.5))
                    .text_color(p.mute)
                    .child(format!("{done} of {total} points"))
                    .child("Day 4 of 14"),
            );
        let actions = div()
            .child(btn(&p, &act, "share", "Share", false))
            .child(btn(&p, &act, "new-issue", "+ New issue", true));
        let mut labs = Vec::new();
        for (i, (_, l, _)) in LABELS.iter().copied().enumerate() {
            labs.push(chip(
                &p,
                &act,
                ids::at(ids::KB_LABEL, i),
                l,
                u.label == Some(i),
            ));
        }
        let bar = div()
            .child(q(&p, "Labels"))
            .child(seg(&p, labs))
            .when(u.label.is_some(), |d| {
                d.child(btn(&p, &act, "kb-clear", "Clear", false))
            })
            .child(div().flex_1())
            .child(seg(
                &p,
                vec![
                    chip(&p, &act, "kb-by-status", "Status", !by_person),
                    chip(&p, &act, "kb-by-person", "Assignee", by_person),
                ],
            ));
        let body = if st == ScreenState::Loading {
            let mut d = div().flex().gap(px(12.)).p(px(16.));
            for _ in 0..5 {
                d = d.child(
                    div()
                        .w(px(262.))
                        .rounded(px(12.))
                        .bg(p.inset)
                        .child(skel(&p, 6)),
                );
            }
            d
        } else if b.is_empty() {
            none(
                &p,
                "No issues in this cycle",
                "Create the first one; drag it across the board as it moves.",
            )
        } else if by_person {
            let mut d = div().flex().gap(px(12.)).p(px(16.)).h_full();
            for w in oxyn_core::kit::arcscr::assignees(b) {
                let mine: Vec<_> = b
                    .iter()
                    .filter(|i| i.who == w && i.col != Col::Done)
                    .collect();
                let mut col = div()
                    .w(px(240.))
                    .flex_none()
                    .rounded(px(12.))
                    .bg(p.inset)
                    .p(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .h(px(30.))
                            .child(av(&p, w, 22.))
                            .child(clip(PEOPLE[w].0))
                            .child(q(&p, mine.len().to_string())),
                    );
                for i in mine.into_iter().take(5) {
                    col = col.child(
                        card(&p)
                            .p(px(10.))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child(col_mark(&p, i.col))
                                    .child(q(&p, issue_key(i.id))),
                            )
                            .child(clip(i.title.clone())),
                    );
                }
                d = d.child(col);
            }
            d
        } else {
            let sample = issues(true);
            let mut d = div().flex().gap(px(12.)).p(px(16.)).pl(px(20.)).h_full();
            for (ci, c) in Col::ALL.into_iter().enumerate() {
                let n = counts[c.index()];
                let lim = c.limit();
                let over = lim > 0 && n > lim;
                let drop_id = ids::at(ids::KB_DROP, ci);
                let a2 = act.clone();
                let mut col = hit(div(), &act, drop_id)
                    .w(px(262.))
                    .flex_none()
                    .rounded(px(12.))
                    .bg(p.inset)
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .pb(px(8.))
                    .when(
                        lifted.is_some() && self.extra == "dragging" && c == Col::Review,
                        |d| d.border_2().border_color(p.fg2),
                    )
                    .drag_over::<ArcDrag>(|s, _, _, _| s.bg(gpui::black().opacity(0.06)))
                    .on_drop(move |d: &ArcDrag, w, cx| {
                        if let Some(f) = &a2 {
                            f(d.grab, w, cx);
                            f(drop_id, w, cx);
                        }
                    })
                    .child(
                        div()
                            .h(px(40.))
                            .pl(px(12.))
                            .pr(px(8.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(col_mark(&p, c))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(c.label()))
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(if over { hue(&p, "rose") } else { p.mute })
                                    .child(if lim > 0 {
                                        format!("{n}/{lim}")
                                    } else {
                                        n.to_string()
                                    }),
                            )
                            .child(div().flex_1())
                            .child(q(&p, "+")),
                    );
                let cards = column(b, c, "", u.label);
                let more = cards.len().saturating_sub(5);
                for i in cards.into_iter().take(5) {
                    let n = sample.iter().position(|x| x.id == i.id).unwrap_or(23);
                    let picked = lifted == Some(i.id);
                    let due = i.due.map(|d| {
                        (
                            format!("◷ {}", &iso(d)[5..]),
                            if d < TODAY {
                                "rose"
                            } else if d <= TODAY + 2 {
                                "amber"
                            } else {
                                "mute"
                            },
                        )
                    });
                    let pick_id = ids::at(ids::KB_PICK, n);
                    let grab = ids::at(ids::KB_GRAB, n);
                    let label: SharedString = format!("{} {}", issue_key(i.id), i.title).into();
                    col = col.child(
                        hit(div(), &act, pick_id)
                            .mx(px(8.))
                            .p(px(10.))
                            .pl(px(12.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(if picked { p.fg } else { p.line })
                            .when(picked, |d| d.border_2())
                            .bg(p.card)
                            .on_drag(ArcDrag { grab, label }, |d: &ArcDrag, _, _, cx| {
                                let d = d.clone();
                                cx.new(|_| d)
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.))
                                    .text_color(p.mute)
                                    .child(issue_key(i.id))
                                    .children(i.pr.map(|pr| {
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(4.))
                                            .text_color(p.fg2)
                                            .children(mark(&p, "github", 12.))
                                            .child(format!("#{pr}"))
                                    }))
                                    .children(due.map(|(ds, h)| {
                                        div()
                                            .flex_none()
                                            .text_size(px(11.5))
                                            .text_color(hue(&p, h))
                                            .child(ds)
                                    }))
                                    .child(div().flex_1())
                                    .child(av(&p, i.who, 22.)),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .h(px(36.))
                                    .overflow_hidden()
                                    .child(i.title.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.))
                                    .text_color(p.mute)
                                    .child(prio(&p, i.prio))
                                    .child(label_chip(&p, i.label))
                                    .child(div().flex_1())
                                    .children(i.subs.map(|(a, b)| q(&p, format!("{a}/{b}"))))
                                    .when(i.comments > 0, |d| {
                                        d.child(q(&p, format!("◌ {}", i.comments)))
                                    }),
                            ),
                    );
                }
                if more > 0 {
                    col = col.child(
                        div()
                            .px(px(16.))
                            .text_color(p.mute)
                            .child(format!("{more} more")),
                    );
                }
                d = d.child(col);
            }
            d
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "kanban-board",
                side,
                ctx: Some(ctx),
                crumbs: vec!["Brindle".into(), "Projects".into(), "Storefront v2".into()],
                meta: format!("{} issues · {} points", b.len(), total),
                actions,
                bar: Some(bar),
                body,
                foot: Some(div().child(q(&p, "Press a card, then a column; or drag it. Alt + arrows move the picked card.")).child(div().flex_1()).children(lifted.map(|i| q(&p, format!("{} picked", issue_key(i)))))),
                right: None,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── Project board ──

fn task_mark(p: &ThPal, st: u8) -> Div {
    col_mark(
        p,
        match st {
            1 => Col::Progress,
            2 => Col::Review,
            3 => Col::Done,
            _ => Col::Todo,
        },
    )
}

impl RenderOnce for ProjectBoardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() && st != ScreenState::Sample {
            u.tasks = tasks(false);
        }
        if self.extra == "atrisk" {
            u.health = Health::AtRisk;
        }
        if self.extra == "activity" {
            u.tab = 2;
        }
        let t = &u.tasks;
        let (pct, open, busy, week, late, until) = stats(t);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, ic, l, n)) in PB_NAV.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ["nav-projecthome", "nav-mytasks", "nav-projectinbox"][i],
                glyph(&p, icon_for(ic)),
                l,
                n,
                false,
            ));
        }
        side = side.child(group(&p, "Projects"));
        for (i, (_, l, d, m, pc)) in PB_PROJECTS.iter().copied().enumerate() {
            let pp = if i == 0 { pct } else { pc };
            side = side.child(
                hit(
                    div(),
                    &act,
                    ["nav-project", "nav-courier", "nav-partner", "nav-brand"][i],
                )
                .h(px(44.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(9.))
                .rounded(px(8.))
                .when(i == 0, |d| d.bg(p.sel))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(l))
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(p.mute)
                                .child(format!("{} · {m}", iso(d))),
                        ),
                )
                .child(q(&p, format!("{pp}%"))),
            );
        }
        let actions = div()
            .child(btn(&p, &act, "share", "Share", false))
            .child(btn(&p, &act, "new-task", "+ New task", true))
            .child(chip(&p, &act, "pb-panel", "▥", u.panel));
        let mut tabs = Vec::new();
        for (k, l) in [
            (0usize, "Overview".to_string()),
            (1, format!("Tasks {open}")),
            (2, "Activity".to_string()),
        ] {
            tabs.push(chip(&p, &act, ids::at(ids::PB_TAB, k), l, u.tab == k));
        }
        let bar = div().child(seg(&p, tabs));
        let task_row = |i: usize, x: &oxyn_core::kit::arcscr::Task| {
            let done = x.st == 3;
            div()
                .h(px(38.))
                .flex()
                .items_center()
                .gap(px(10.))
                .border_b_1()
                .border_color(p.line)
                .child(
                    hit(div(), &act, ids::at(ids::PB_TASK, i))
                        .size(px(22.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .child(task_mark(&p, x.st)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_color(if done { p.mute } else { p.fg })
                        .when(done, |d| d.line_through())
                        .child(x.title.clone()),
                )
                .when(x.comments > 0, |d| {
                    d.child(q(&p, format!("◌ {}", x.comments)))
                })
                .child(prio(&p, x.prio))
                .child(
                    div()
                        .flex_none()
                        .text_size(px(11.5))
                        .text_color(if !done && x.due < TODAY {
                            hue(&p, "rose")
                        } else {
                            p.mute
                        })
                        .child(iso(x.due)),
                )
                .child(av(&p, x.who, 22.))
        };
        let body = if st == ScreenState::Loading {
            div().p(px(28.)).child(skel(&p, 14))
        } else if u.tab == 2 {
            let mut l = div().px(px(32.)).py(px(26.)).flex().flex_col().child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .mb(px(10.))
                    .child("Activity"),
            );
            if t.is_empty() {
                l = l.child(
                    div()
                        .text_color(p.mute)
                        .child("Nothing has happened on this project yet."),
                );
            } else {
                for (w, what, d, tm) in ACTIVITY {
                    l = l.child(
                        div()
                            .h(px(42.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .border_b_1()
                            .border_color(p.line)
                            .child(av(&p, w, 22.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .child(format!("{} {what}", PEOPLE[w].0)),
                            )
                            .child(q(&p, format!("{} {tm}", iso(d)))),
                    );
                }
            }
            l
        } else {
            let h = u.health;
            let mut hs = Vec::new();
            for (i, x) in Health::ALL.into_iter().enumerate() {
                hs.push(
                    hit(div(), &act, ids::at(ids::PB_HEALTH, i))
                        .h(px(24.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .rounded(px(6.))
                        .text_size(px(12.))
                        .text_color(if x == h { hue(&p, x.hue()) } else { p.mute })
                        .when(x == h, |d| {
                            d.bg(tint(&p, x.hue(), 0.14))
                                .font_weight(FontWeight::SEMIBOLD)
                        })
                        .child(dot(hue(&p, x.hue()), 7.))
                        .child(x.label()),
                );
            }
            let stat = |l: &'static str, v: String, sub: Div, last: bool| {
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(18.))
                    .py(px(14.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .when(!last, |d| d.border_r_1().border_color(p.line))
                    .child(q(&p, l))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(v),
                    )
                    .child(sub)
            };
            let mut mst = div().flex().gap(px(8.)).mb(px(20.));
            for (m, (mn, d, _)) in MILESTONES.iter().copied().enumerate() {
                let (dn, a) = ms_progress(t, m);
                let full = a > 0 && dn == a;
                let pc = (dn * 100).checked_div(a).unwrap_or(0);
                mst = mst.child(
                    hit(div(), &act, ids::at(ids::PB_MS, m))
                        .flex_1()
                        .min_w_0()
                        .p(px(10.))
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .rounded(px(10.))
                        .when(u.ms == m, |x| x.bg(p.sel))
                        .child(div().h(px(3.)).mb(px(8.)).rounded(px(2.)).bg(p.line).child(
                            div().h_full().w(relative(pc as f32 / 100.)).bg(if full {
                                hue(&p, "green")
                            } else {
                                p.fg
                            }),
                        ))
                        .child(clip(mn).font_weight(FontWeight::MEDIUM))
                        .child(q(&p, iso(d)))
                        .child(q(
                            &p,
                            if full {
                                "Done".to_string()
                            } else {
                                format!("{pc}%")
                            },
                        )),
                );
            }
            let mut list = div().flex().flex_col();
            for (i, x) in t.iter().enumerate().filter(|(_, x)| x.ms == u.ms) {
                list = list.child(task_row(i, x));
            }
            div()
                .px(px(32.))
                .py(px(26.))
                .flex()
                .flex_col()
                .child(div().flex().items_center().gap(px(10.)).child(div().size(px(30.)).rounded(px(8.)).bg(tint(&p, "teal", 0.16)).flex().items_center().justify_center().text_color(hue(&p, "teal")).child("▣")).child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).child(PROJECT.0)))
                .child(div().my(px(10.)).max_w(px(660.)).text_color(p.fg2).child(PROJECT.1))
                .child(div().flex().items_center().gap(px(14.)).mb(px(16.)).child(div().flex().gap(px(4.)).children(hs)).child(av(&p, PROJECT.2, 22.)).child(div().text_color(p.fg2).child(format!("{} · lead", PEOPLE[PROJECT.2].0))).child(q(&p, format!("{} → {}", iso(PROJECT.3), iso(PROJECT.4)))))
                .when(h != Health::OnTrack, |d| d.child(div().mb(px(14.)).px(px(12.)).py(px(8.)).rounded(px(8.)).bg(tint(&p, h.hue(), 0.12)).text_color(p.fg).child(format!("{late} task overdue and {week} due this week; the next milestone is {}.", iso(MILESTONES[2].1)))))
                .child(
                    card(&p)
                        .flex()
                        .mb(px(22.))
                        .child(stat("Complete", format!("{pct}%"), div().h(px(4.)).mt(px(6.)).rounded(px(2.)).bg(p.sel).child(div().h_full().w(relative(pct as f32 / 100.)).rounded(px(2.)).bg(p.fg)), false))
                        .child(stat("Open tasks", open.to_string(), q(&p, format!("{busy} in progress or review")), false))
                        .child(stat("Due this week", week.to_string(), div().text_size(px(12.)).text_color(if late > 0 { hue(&p, "rose") } else { p.mute }).child(format!("{late} overdue")), false))
                        .child(stat("Until launch", format!("{until} days"), q(&p, iso(PROJECT.4)), true)),
                )
                .child(div().flex().items_center().gap(px(6.)).mb(px(10.)).child(div().font_weight(FontWeight::SEMIBOLD).child("Milestones")).child(q(&p, format!("{} of {} done", ms_done(t), MILESTONES.len()))))
                .child(mst)
                .child(div().font_weight(FontWeight::SEMIBOLD).mb(px(4.)).child(MILESTONES[u.ms].0))
                .child(list)
        };
        let right = u.panel.then(|| {
            let mut team = div().flex().flex_col().gap(px(12.)).mb(px(22.));
            for (w, role) in TEAM {
                let n = load_of(t, w);
                team = team.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(av(&p, w, 30.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(div().text_color(p.fg).child(if w == PROJECT.2 {
                                    format!("{} · lead", PEOPLE[w].0)
                                } else {
                                    PEOPLE[w].0.to_string()
                                }))
                                .child(div().text_size(px(11.5)).text_color(p.mute).child(role)),
                        )
                        .child(
                            div().w(px(44.)).h(px(4.)).rounded(px(2.)).bg(p.sel).child(
                                div()
                                    .h_full()
                                    .w(relative((n as f32 * 0.25).min(1.)))
                                    .bg(p.fg2),
                            ),
                        )
                        .child(q(&p, n.to_string())),
                );
            }
            let mut links = div().flex().flex_col().gap(px(12.));
            for (b, l, s) in LINKS {
                links = links.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(div().w(px(18.)).children(mark(&p, b, 16.)))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(div().text_color(p.fg).child(l))
                                .child(div().text_size(px(11.5)).text_color(p.mute).child(s)),
                        )
                        .child(q(&p, "⧉")),
                );
            }
            div()
                .p(px(18.))
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .mb(px(12.))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Team"))
                        .child(q(&p, TEAM.len().to_string())),
                )
                .child(team)
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .mb(px(12.))
                        .child("Links"),
                )
                .child(links)
        });
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "project-board",
                side,
                ctx: None,
                crumbs: vec!["Projects".into(), PROJECT.0.into()],
                meta: String::new(),
                actions,
                bar: Some(bar),
                body,
                foot: None,
                right,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── Gantt ──

fn phase_hue(ph: usize) -> &'static str {
    ["teal", "magenta", "blue", "amber"][ph.min(3)]
}

const GDAYS: i32 = 106;

impl RenderOnce for GanttTimelineEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() && st != ScreenState::Sample {
            u.plan.bars = plan(false);
        }
        if self.extra == "month" {
            u.gzoom = 2;
        }
        let ghost = if self.extra == "dragging" {
            Some((10u32, 4i32))
        } else {
            None
        };
        let qq = u.query.to_lowercase();
        let shown_bars: Vec<oxyn_core::kit::arcscr::Bar> = u
            .plan
            .bars
            .iter()
            .filter(|b| {
                (qq.is_empty() || b.name.to_lowercase().contains(&qq))
                    && u.gowner.is_none_or(|w| b.who == w)
            })
            .cloned()
            .collect();
        let bars = &shown_bars;
        let ppd = ZOOMS[u.gzoom].2;
        let bad = conflicts(bars);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, ic, l, n)) in GT_NAV.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ["nav-timeline", "nav-ganttlist", "nav-updates"][i],
                glyph(&p, icon_for(ic)),
                l,
                n,
                i == 0,
            ));
        }
        side = side.child(group(&p, "Milestones"));
        for b in bars.iter().filter(|b| b.len == 0) {
            side = side.child(nrow(
                &p,
                &act,
                ids::at(ids::GT_PICK, b.id as usize),
                div()
                    .w(px(16.))
                    .flex()
                    .justify_center()
                    .text_color(hue(&p, phase_hue(b.phase)))
                    .child("◆"),
                b.name,
                iso(b.start),
                u.bar == Some(b.id),
            ));
        }
        side = side.child(group(&p, "Saved views"));
        for (i, (ic, l, n)) in GT_VIEWS.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ["nav-mygantt", "nav-atrisk", "nav-nexttwoweeks"][i],
                glyph(&p, icon_for(ic)),
                l,
                n,
                false,
            ));
        }
        let actions = div()
            .child(btn(&p, &act, "share", "Share", false))
            .child(btn(&p, &act, "new-task", "+ New task", true));
        let mut zs = Vec::new();
        for (i, (_, l, _)) in ZOOMS.iter().copied().enumerate() {
            zs.push(chip(&p, &act, ids::at(ids::GT_ZOOM, i), l, u.gzoom == i));
        }
        let picked = u.bar.and_then(|i| bars.iter().find(|b| b.id == i));
        // The same toolbar as the Dioxus screen: filter field, owner, zoom, Today; nudges sit in
        // the foot with Undo and Reset.
        let field = div()
            .w(px(250.))
            .h(px(32.))
            .px(px(9.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(8.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.mute)
            .child("⌕")
            .child(match self.search.clone() {
                Some(v) => div().flex_1().min_w_0().child(v),
                None => div().flex_1().child(if u.query.is_empty() {
                    "Filter tasks".to_string()
                } else {
                    u.query.clone()
                }),
            });
        let bar = div()
            .child(field)
            .child(btn(
                &p,
                &act,
                "gt-owner",
                format!("{} ⌄", u.gowner.map(|w| PEOPLE[w].0).unwrap_or("Owner")),
                false,
            ))
            .child(div().flex_1())
            .child(seg(&p, zs))
            .child(btn(&p, &act, "gt-today", "Today", false));
        let foot = div()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .child(match picked {
                        Some(b) => format!("{} · {}", b.name, bar_dates(b)),
                        None => "Drag a bar onto a day to move it, or press it and nudge it. Alt + ← → moves it, Alt + Shift + ← → its end.".to_string(),
                    }),
            )
            .when(picked.is_some(), |d| {
                d.child(btn(&p, &act, "gt-earlier", "← Earlier", false))
                    .child(btn(&p, &act, "gt-later", "Later →", false))
                    .child(btn(&p, &act, "gt-shorter", "Shorter", false))
                    .child(btn(&p, &act, "gt-longer", "Longer", false))
            })
            .child(btn(&p, &act, "gt-undo", "Undo", false).when(u.plan.undo.is_empty(), |d| d.opacity(0.45)))
            .child(btn(&p, &act, "gt-reset", "Reset", false));
        let body = if st == ScreenState::Loading {
            div()
                .flex()
                .child(div().w(px(262.)).child(skel(&p, 14)))
                .child(div().flex_1().child(skel(&p, 14)))
        } else if bars.is_empty() {
            none(
                &p,
                "No tasks on the timeline",
                "Add a task to start the plan; drag it to set its dates.",
            )
        } else {
            let row_h = 36.;
            let mut list = div()
                .w(px(262.))
                .flex_none()
                .flex()
                .flex_col()
                .border_r_1()
                .border_color(p.line)
                .child(
                    div()
                        .h(px(52.))
                        .px(px(20.))
                        .flex()
                        .items_center()
                        .border_b_1()
                        .border_color(p.line)
                        .child(q(&p, format!("{} tasks", bars.len()))),
                );
            let mut grid = div()
                .relative()
                .w(px(GDAYS as f32 * ppd))
                .h(px(52. + row_h * (bars.len() + 4) as f32));
            // month labels and ticks
            for d in 0..GDAYS {
                let (m, day) = month_day(d);
                let x = d as f32 * ppd;
                if day == 1 || d == 0 {
                    grid = grid.child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .left(px(x + 6.))
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(p.fg2)
                            .child(m),
                    );
                }
                let show = match u.gzoom {
                    0 => true,
                    1 => oxyn_core::kit::arcscr::weekday(d) == 0,
                    _ => day == 1 || day == 15,
                };
                if show {
                    grid = grid.child(
                        div()
                            .absolute()
                            .top(px(30.))
                            .left(px(x + 3.))
                            .text_size(px(11.))
                            .text_color(p.mute)
                            .child(format!("{day:02}")),
                    );
                    grid = grid.child(
                        div()
                            .absolute()
                            .top(px(52.))
                            .bottom_0()
                            .left(px(x))
                            .w(px(1.))
                            .bg(p.line.opacity(0.6)),
                    );
                }
                // a drop target for each day
                let at = ids::at(ids::GT_AT, d as usize);
                let a2 = act.clone();
                grid = grid.child(
                    div()
                        .id(SharedString::from(format!("day-{d}")))
                        .absolute()
                        .top(px(52.))
                        .bottom_0()
                        .left(px(x))
                        .w(px(ppd))
                        .drag_over::<ArcDrag>(|s, _, _, _| s.bg(gpui::black().opacity(0.06)))
                        .on_drop(move |g: &ArcDrag, w, cx| {
                            if let Some(f) = &a2 {
                                f(g.grab, w, cx);
                                f(at, w, cx);
                            }
                        }),
                );
            }
            grid = grid.child(
                div()
                    .absolute()
                    .top(px(52.))
                    .left_0()
                    .right_0()
                    .h(px(1.))
                    .bg(p.line),
            );
            let tx = (TODAY as f32 + 0.5) * ppd;
            grid = grid.child(
                div()
                    .absolute()
                    .top(px(52.))
                    .bottom_0()
                    .left(px(tx))
                    .w(px(1.5))
                    .bg(p.fg),
            );
            grid = grid.child(
                div()
                    .absolute()
                    .top(px(58.))
                    .left(px(tx - 58.))
                    .w(px(116.))
                    .h(px(20.))
                    .rounded(px(10.))
                    .bg(p.fg)
                    .text_color(p.card)
                    .text_size(px(11.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(format!("Today {}", iso(TODAY))),
            );
            // What each row holds on the time axis, so a dragged bar's dates go into free space
            // (the same rule as the Dioxus screen: above, below, after, before).
            let mut spans: Vec<Vec<(f64, f64)>> = Vec::new();
            for ph in 0..PHASES.len() {
                let mut v = Vec::new();
                if spans.is_empty() {
                    v.push((tx as f64 - 58., tx as f64 + 58.));
                }
                if let Some((a, z, _)) = phase_span(bars, ph) {
                    v.push((a as f64 * ppd as f64, (z + 1) as f64 * ppd as f64));
                }
                spans.push(v);
                for b in bars.iter().filter(|b| b.phase == ph) {
                    let shift = ghost.filter(|g| g.0 == b.id).map(|g| g.1).unwrap_or(0);
                    let x = ((b.start + shift) as f32 * ppd) as f64;
                    let nw = b.name.chars().count() as f64 * 7.;
                    let w = if b.len == 0 {
                        20. + nw + 80.
                    } else {
                        (b.len as f32 * ppd) as f64
                    };
                    let out = if b.len > 0 && w <= 120. { 8. + nw } else { 0. };
                    spans.push(vec![(x - 7., x + w + out)]);
                }
            }
            let width = (GDAYS as f32 * ppd) as f64;
            let mut ri = 0usize;
            for (ph, (_, pl)) in PHASES.iter().copied().enumerate() {
                let bs: Vec<_> = bars.iter().filter(|b| b.phase == ph).collect();
                let top = 52. + ri as f32 * row_h;
                list = list.child(
                    div()
                        .h(px(row_h))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(q(&p, "⌄"))
                        .child(div().font_weight(FontWeight::MEDIUM).child(pl))
                        .child(div().flex_1())
                        .child(q(&p, bs.len().to_string())),
                );
                if let Some((a, z, pc)) = phase_span(bars, ph) {
                    grid = grid.child(
                        div()
                            .absolute()
                            .top(px(top + 15.))
                            .left(px(a as f32 * ppd))
                            .w(px((z - a + 1) as f32 * ppd))
                            .h(px(6.))
                            .rounded(px(3.))
                            .bg(p.mute.opacity(0.26))
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(pc as f32 / 100.))
                                    .rounded(px(3.))
                                    .bg(p.mute),
                            ),
                    );
                }
                ri += 1;
                for b in bs {
                    let top = 52. + ri as f32 * row_h;
                    let late = bad.iter().any(|x| x.1 == b.id);
                    list = list.child(
                        div()
                            .h(px(row_h))
                            .pl(px(30.))
                            .pr(px(14.))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_color(p.fg2)
                            .child(av(&p, b.who, 22.))
                            .child(clip(b.name))
                            .when(late, |d| {
                                d.child(div().text_color(hue(&p, "rose")).child("!"))
                            })
                            .child(q(&p, iso(end_of(b)))),
                    );
                    let h = phase_hue(b.phase);
                    let on = u.bar == Some(b.id);
                    let shift = ghost.filter(|g| g.0 == b.id).map(|g| g.1).unwrap_or(0);
                    let start = b.start + shift;
                    let label: SharedString = b.name.into();
                    let el = if b.len == 0 {
                        hit(div(), &act, ids::at(ids::GT_PICK, b.id as usize))
                            .absolute()
                            .top(px(top))
                            .left(px(start as f32 * ppd + ppd / 2. - 7.))
                            .h(px(row_h))
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .text_size(px(12.))
                            .child(div().text_color(hue(&p, h)).child("◆"))
                            .child(div().font_weight(FontWeight::MEDIUM).child(b.name))
                            .child(q(&p, iso(start)))
                    } else {
                        let w = b.len as f32 * ppd;
                        let inside = w > 120.;
                        hit(div(), &act, ids::at(ids::GT_PICK, b.id as usize))
                            .absolute()
                            .top(px(top + 6.))
                            .left(px(start as f32 * ppd))
                            .w(px(w))
                            .h(px(24.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(if late {
                                hue(&p, "rose")
                            } else if on {
                                hue(&p, h)
                            } else {
                                tint(&p, h, 0.45)
                            })
                            .when(on || shift != 0, |d| d.border_2())
                            .bg(tint(&p, h, 0.16))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .bottom_0()
                                    .w(relative(b.done as f32 / 100.))
                                    .rounded(px(5.))
                                    .bg(tint(&p, h, 0.32)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .size_full()
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.))
                                    .when(inside, |d| {
                                        d.child(clip(b.name).font_weight(FontWeight::MEDIUM))
                                    })
                                    .when(!inside, |d| d.child(div().flex_1()))
                                    .when(w > 44., |d| {
                                        d.child(
                                            div()
                                                .flex_none()
                                                .text_color(p.fg2)
                                                .text_size(px(11.5))
                                                .child(format!("{}%", b.done)),
                                        )
                                    }),
                            )
                            .when(!inside, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .left(px(w + 8.))
                                        .top(px(4.))
                                        .whitespace_nowrap()
                                        .text_size(px(12.))
                                        .text_color(p.fg2)
                                        .child(b.name),
                                )
                            })
                            .when(shift != 0, |d| {
                                let mut c = b.clone();
                                c.start = start;
                                let t = bar_dates(&c);
                                let x = (start as f32 * ppd) as f64;
                                let nw = b.name.chars().count() as f64 * 7.;
                                let end = x + w as f64 + if inside { 0. } else { 8. + nw };
                                let above = if ri == 0 {
                                    None
                                } else {
                                    spans.get(ri - 1).map(|v| v.as_slice())
                                };
                                let below = spans.get(ri + 1).map(|v| v.as_slice());
                                let at = tip_at(x, tip_w(&t), end, width, above, below);
                                let tip = div().absolute();
                                let tip = match at {
                                    TipAt::Above => tip.top(px(-26.)).left_0(),
                                    TipAt::Below => tip.top(px(28.)).left_0(),
                                    TipAt::Right => {
                                        tip.top(px(0.)).left(px((end - x) as f32 + 16.))
                                    }
                                    TipAt::Left => tip.top(px(0.)).right(px(w + 12.)),
                                };
                                d.child(
                                    tip.px(px(8.))
                                        .py(px(3.))
                                        .rounded(px(6.))
                                        .bg(p.fg)
                                        .text_color(p.card)
                                        .text_size(px(11.5))
                                        .whitespace_nowrap()
                                        .child(t),
                                )
                            })
                    };
                    grid = grid.child(el.on_drag(
                        ArcDrag {
                            grab: ids::at(ids::GT_GRAB, b.id as usize),
                            label,
                        },
                        |d: &ArcDrag, _, _, cx| {
                            let d = d.clone();
                            cx.new(|_| d)
                        },
                    ));
                    ri += 1;
                }
            }
            div().size_full().flex().child(list).child(
                div()
                    .id("gantt-scroll")
                    .flex_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .child(grid),
            )
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "gantt-timeline",
                side,
                ctx: None,
                crumbs: vec!["Northgate".into(), PLAN.0.into(), "Timeline".into()],
                meta: format!("{} tasks · {}% done", bars.len(), plan_done(bars)),
                actions,
                bar: Some(bar),
                body,
                foot: Some(foot),
                right: None,
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── Team directory ──

fn avail(p: &ThPal, a: Avail) -> Div {
    let g = match a {
        Avail::Free => "✓",
        Avail::Busy(_) => "▦",
        Avail::Out(_) => "✈",
        Avail::Starts(_) => "◷",
        Avail::Later(_) => "☾",
    };
    div()
        .flex()
        .items_center()
        .gap(px(5.))
        .min_w_0()
        .text_size(px(12.5))
        .text_color(p.fg2)
        .child(div().flex_none().text_color(hue(p, a.hue())).child(g))
        .child(clip(a.words()))
}

impl RenderOnce for TeamDirectoryEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let u = live(&self.arc);
        let all = members(st != ScreenState::Empty);
        let list_view = u.list || self.extra == "list";
        let query = if self.extra == "noresults" && u.query.is_empty() {
            "astrophysics".to_string()
        } else {
            u.query.clone()
        };
        let hits = find_people(&all, &query, u.team, u.status, u.office);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, ic, l, n)) in TD_NAV.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ["nav-people", "nav-org", "nav-timeoff"][i],
                glyph(&p, icon_for(ic)),
                l,
                if i == 0 {
                    all.len().to_string()
                } else {
                    n.to_string()
                },
                i == 0,
            ));
        }
        side = side.child(group(&p, "Offices"));
        for (o, (_, city, _)) in OFFICES.iter().copied().enumerate() {
            let (wk, here) = office_count(&all, o);
            let open = office_open(o);
            side = side.child(
                hit(div(), &act, ids::at(ids::TD_OFFICE, o))
                    .h(px(44.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .rounded(px(8.))
                    .when(u.office == Some(o), |d| d.bg(p.sel))
                    .child(
                        div()
                            .w(px(16.))
                            .flex()
                            .justify_center()
                            .text_color(if open { hue(&p, "amber") } else { p.mute })
                            .child(if open { "☀" } else { "☾" }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(clip(city))
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(p.mute)
                                    .child(format!("{wk} of {here} working now")),
                            ),
                    )
                    .child(q(&p, local_time(o))),
            );
        }
        side = side.child(group(&p, "Favorites"));
        for w in &u.favs {
            side = side.child(
                div()
                    .h(px(32.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .text_color(p.fg2)
                    .child(av(&p, *w, 20.))
                    .child(clip(PEOPLE[*w].0)),
            );
        }
        let actions = div().child(btn(&p, &act, "invite", "Invite", true));
        let field = div()
            .w(px(280.))
            .h(px(32.))
            .px(px(9.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(8.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.mute)
            .child("⌕")
            .child(match self.search.clone() {
                Some(v) => div().flex_1().min_w_0().child(v),
                None => div().flex_1().child(if query.is_empty() {
                    "Search people, roles, skills".to_string()
                } else {
                    query.clone()
                }),
            });
        let mut teams = Vec::new();
        for (i, (_, l, _, _)) in TEAMS.iter().copied().enumerate() {
            teams.push(chip(
                &p,
                &act,
                ids::at(ids::TD_TEAM, i),
                l,
                u.team == Some(i),
            ));
        }
        let mut sts = Vec::new();
        for (i, (k, l)) in STATUSES.iter().copied().enumerate() {
            sts.push(chip(
                &p,
                &act,
                ids::at(ids::TD_STATUS, i),
                l,
                u.status == Some(k),
            ));
        }
        let bar = div()
            .child(field)
            .child(seg(&p, teams))
            .child(seg(&p, sts))
            .child(div().flex_1())
            .child(seg(
                &p,
                vec![
                    chip(&p, &act, "td-grid", "Grid", !list_view),
                    chip(&p, &act, "td-list", "List", list_view),
                ],
            ));
        let body = if st == ScreenState::Loading {
            div().p(px(24.)).child(skel(&p, 12))
        } else if all.is_empty() {
            none(
                &p,
                "No one here yet",
                "Invite your team; they show here with their office and hours.",
            )
        } else if hits.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(none(
                    &p,
                    "No one matches the search",
                    "Try a name, a role or a skill, or clear the filters.",
                ))
                .child(btn(&p, &act, "td-clear", "Clear search", false))
        } else {
            let mut d = div()
                .id("td-scroll")
                .size_full()
                .overflow_y_scroll()
                .px(px(24.))
                .py(px(18.))
                .flex()
                .flex_col()
                .gap(px(20.));
            if self.extra == "inviting" {
                d = d.child(
                    card(&p)
                        .px(px(12.))
                        .py(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(q(&p, "✉"))
                        .child(div().flex_1().child("noor@saltwick.example"))
                        .child(btn(&p, &act, "invite-cancel", "Cancel", false))
                        .child(btn(&p, &act, "invite-send", "Send invite", true)),
                );
            }
            for (t, (_, tl, _, h)) in TEAMS.iter().copied().enumerate() {
                let mine: Vec<usize> = hits.iter().copied().filter(|i| all[*i].team == t).collect();
                if mine.is_empty() {
                    continue;
                }
                let mut grid = div().flex().flex_wrap().gap(px(10.));
                if list_view {
                    grid = grid.flex_col().gap_0();
                }
                for i in mine.iter().copied() {
                    let m = &all[i];
                    let fav = u.favs.contains(&m.who);
                    let star = hit(div(), &act, ids::at(ids::TD_FAV, m.who))
                        .px(px(4.))
                        .text_color(if fav { hue(&p, "amber") } else { p.mute })
                        .child(if fav { "★" } else { "☆" });
                    let headr = div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .min_w_0()
                        .child(av(&p, m.who, 30.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.))
                                        .child(
                                            clip(PEOPLE[m.who].0)
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(p.fg),
                                        )
                                        .when(m.lead, |d| {
                                            d.child(
                                                div()
                                                    .px(px(5.))
                                                    .rounded(px(4.))
                                                    .bg(p.sel)
                                                    .text_size(px(10.5))
                                                    .child("Lead"),
                                            )
                                        }),
                                )
                                .child(div().text_size(px(12.)).text_color(p.mute).child(m.role)),
                        )
                        .child(star);
                    let footr = div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .min_w_0()
                        .child(q(
                            &p,
                            format!("{} {}", OFFICES[m.office].1, local_time(m.office)),
                        ))
                        .child(div().flex_1())
                        .child(avail(&p, m.avail));
                    grid = grid.child(if list_view {
                        div()
                            .h(px(56.))
                            .flex()
                            .items_center()
                            .gap(px(14.))
                            .border_b_1()
                            .border_color(p.line)
                            .child(div().flex_1().min_w_0().child(headr))
                            .child(div().w(px(140.)).child(q(&p, TEAMS[m.team].1)))
                            .child(div().w(px(320.)).child(footr))
                    } else {
                        card(&p)
                            .w(px(264.))
                            .p(px(12.))
                            .flex()
                            .flex_col()
                            .gap(px(10.))
                            .child(headr.pb(px(10.)).border_b_1().border_color(p.line))
                            .child(footr)
                    });
                }
                d = d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(dot(hue(&p, h), 8.))
                                .child(div().font_weight(FontWeight::SEMIBOLD).child(tl))
                                .child(q(&p, mine.len().to_string())),
                        )
                        .child(grid),
                );
            }
            div().size_full().child(d)
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "team-directory",
                side,
                ctx: None,
                crumbs: vec!["Saltwick".into(), "People".into()],
                meta: format!("{} people in 5 offices", all.len()),
                actions,
                bar: Some(bar),
                body,
                foot: None,
                right: None,
            },
        )
    }
}
