//! Screens S13 (Arc screens II) in GPUI: [`WeekCalendarEl`], [`MonthCalendarEl`],
//! [`InboxTriageEl`], [`SupportConversationEl`] and [`HelpCenterEl`] on the Arc frame of S12,
//! over `oxyn_core::kit::arcscr2`.
//!
//! Each element draws one state (`.state()`, `.extra()`); with `.arc2(Arc2Ui)` it draws that
//! live state instead of the sample, and its presses (`cal-*`, `ib-*`, `sp-*`, `hc-*`) go back
//! through `on_act` for the host to apply with [`oxyn_core::kit::arcscr2::Arc2Ui::press`].
//! Calendar events can be dragged onto a day column; `.field()` puts a host's text field in the
//! support composer or the help center search.

use gpui::{
    AnyView, App, Div, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, relative,
};
use oxyn_core::kit::arcscr2::{
    ART_TABS, Arc2Ui, CALS, COLLECTIONS, CUSTOMERS, Event, FOLDERS, HC_NAV, HOURS, IB_TEAM, Kind,
    NOW, PEOPLE, REASONS, SAVED, SEARCHES, SNOOZES, SP_VIEWS, ScreenState, TABS, TAGS, TODAY,
    TStatus, VIEWS, WEEKDAYS, WORK, block_focus, busy_minutes, cal_counts, cal_nav, cust_hue,
    cust_ini, cust_time, day_events, events, free_gaps, helpful, hhmm, ids, iso, iso_week, lanes,
    month_cells, month_day, month_of, period_title, previous, quarter, sla_hue, sla_words,
    tab_counts, thousands, thread, waiting, week_start, weekday, when,
};
use oxyn_core::kit::screensfid::ShellLayout;
use oxyn_tokens::Theme;

use super::Act;
use super::apps::hit;
use super::arcscr::{
    ArcDrag, Frame, av, btn, card, chip, frame, glyph, group, hue, icon_for, none, nrow, q, seg,
    skel, tint,
};
use super::chatscr::{clip, mark};
use super::rulerfid::{ThPal, th_pal};

macro_rules! arc2_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
            arc: Option<Arc2Ui>,
            field: Option<AnyView>,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK, arc: None, field: None }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state (see `oxyn_core::kit::arcscr2::s13_states`).
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
            /// Press ids (`nav-<key>`, `acct`, and the screen's own `cal-*`, `ib-*`, `sp-*`,
            /// `hc-*`).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
            /// Draw this live state instead of the sample.
            pub fn arc2(mut self, a: Arc2Ui) -> Self {
                self.arc = Some(a);
                self
            }
            /// A text field: the support composer or the help center search.
            pub fn field(mut self, v: AnyView) -> Self {
                self.field = Some(v);
                self
            }
        }
    };
}

arc2_el!(
    /// Week calendar: the week's time grid; press an event, then a day or the nudge buttons
    /// (or drag it onto a day); Day / Week / Month.
    WeekCalendarEl
);
arc2_el!(
    /// Month calendar: the month grid with chips, the day panel (free time, Block focus time).
    MonthCalendarEl
);
arc2_el!(
    /// Inbox triage: folders, tabs, the list with selection, the thread, the customer card.
    InboxTriageEl
);
arc2_el!(
    /// Support conversation: tickets, the thread with notes, the composer, the customer card.
    SupportConversationEl
);
arc2_el!(
    /// Help center: articles by status and collection, search, the reader and its feedback.
    HelpCenterEl
);

fn live(a: &Option<Arc2Ui>) -> Arc2Ui {
    a.clone().unwrap_or_default()
}

fn cav(p: &ThPal, c: usize, size: f32) -> Div {
    let h = cust_hue(c);
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
        .child(cust_ini(c))
}

fn pill(p: &ThPal, h: &str, t: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .h(px(20.))
        .px(px(7.))
        .flex()
        .items_center()
        .rounded(px(6.))
        .bg(tint(p, h, 0.14))
        .text_color(if h == "mute" { p.fg2 } else { hue(p, h) })
        .text_size(px(11.5))
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .child(t.into())
}

fn sla(p: &ThPal, m: i32, pre: &str) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .text_size(px(11.5))
        .font_weight(FontWeight::MEDIUM)
        .text_color(hue(p, sla_hue(m)))
        .child(format!("◷ {pre}{}", sla_words(m)))
}

fn tagd(p: &ThPal, h: &str) -> Div {
    div().flex_none().size(px(8.)).rounded(px(2.)).bg(hue(p, h))
}

fn chan(p: &ThPal, ch: &str, size: f32) -> Div {
    let d = div().flex_none().flex().items_center();
    match mark(p, if ch == "chat" { "" } else { ch }, size) {
        Some(m) => d.child(m),
        None => d.text_size(px(size)).text_color(p.fg2).child("◍"),
    }
}

fn chan_name(ch: &str) -> &'static str {
    match ch {
        "gmail" => "Gmail",
        "slack" => "Slack",
        "x" => "X",
        _ => "Site chat",
    }
}

fn h3(p: &ThPal, t: &'static str) -> Div {
    div()
        .mt(px(6.))
        .mb(px(8.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_size(px(13.5))
        .text_color(p.fg)
        .child(t)
}

fn field_box(p: &ThPal, v: Option<AnyView>, ph: String, w: f32) -> Div {
    div()
        .w(px(w))
        .flex_none()
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
        .child(match v {
            Some(v) => div().flex_1().min_w_0().child(v),
            None => clip(ph),
        })
}

// ───────────────────────────────────────────────────────────── calendar ──

const HH: f32 = 40.;

#[allow(clippy::too_many_arguments)]
fn cal_body(
    p: &ThPal,
    act: &Option<Act>,
    slug: &'static str,
    st: ScreenState,
    extra: &'static str,
    layout: ShellLayout,
    arc: &Option<Arc2Ui>,
    start_view: usize,
) -> Div {
    let mut u = live(arc);
    if arc.is_none() {
        u.cview = start_view;
        if st != ScreenState::Sample {
            u.cal = events(false);
        }
        match extra {
            "dayview" => u.cview = 0,
            "hidden" => {
                u.hidden[4] = true;
                u.hidden[5] = true;
            }
            "focus" => {
                block_focus(&mut u.cal, TODAY, &[false; 6]);
            }
            "dragging" | "creating" => u.cpick = Some(11),
            _ => {}
        }
    }
    let ghost = (extra == "dragging").then_some((11u32, 1i32, 30i32));
    let d = u.cday;
    let h = u.hidden;
    let vw = u.cview;
    let counts = cal_counts(&u.cal, d);
    let nav = cal_nav(slug);
    let mut side = div().flex().flex_col().gap(px(1.));
    for (i, (k, ic, l, n)) in nav.iter().copied().enumerate() {
        let id = match (slug, i) {
            ("month-calendar", 0) => "nav-monthcal",
            (_, 0) => "nav-weekcal",
            _ => k,
        };
        side = side.child(nrow(p, act, id, glyph(p, icon_for(ic)), l, n, i == 0));
    }
    side = side.child(group(
        p,
        if slug == "week-calendar" {
            "Calendars"
        } else {
            "My calendars"
        },
    ));
    for (i, (_, name, hh, _)) in CALS.iter().copied().enumerate() {
        let on = !h[i];
        side = side.child(
            hit(div(), act, ids::at(ids::CAL_CAL, i))
                .h(px(30.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(9.))
                .rounded(px(8.))
                .child(
                    div()
                        .size(px(16.))
                        .rounded(px(4.))
                        .border_1()
                        .border_color(hue(p, hh))
                        .when(on, |d| d.bg(hue(p, hh)))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.))
                        .text_color(p.card)
                        .child(if on { "✓" } else { "" }),
                )
                .child(clip(name).text_color(p.fg2))
                .child(q(p, counts[i].to_string())),
        );
    }
    side = side.child(group(p, "People"));
    for (who, s, hh) in [
        (1usize, "Free until 15:00", "green"),
        (2, "In a meeting until 13:30", "amber"),
        (5, "Out until 2026-11-16", "mute"),
    ] {
        side = side.child(
            div()
                .h(px(36.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(9.))
                .child(av(p, who, 22.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(PEOPLE[who].0).text_color(p.fg2))
                        .child(clip(s).text_size(px(11.5)).text_color(hue(p, hh))),
                ),
        );
    }
    let next = u
        .cal
        .iter()
        .filter(|e| e.day == TODAY && !e.all_day && e.start > NOW)
        .min_by_key(|e| e.start)
        .cloned();
    let ctx = div()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(q(p, "Next up"))
        .child(match &next {
            Some(e) => div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(clip(e.title.clone()).font_weight(FontWeight::SEMIBOLD))
                .child(q(p, format!("{} · in {} min", e.span(), e.start - NOW))),
            None => q(p, "Nothing else today"),
        });
    let actions = div().child(btn(p, act, "cal-new", "+ New event", true));
    let mut vs = Vec::new();
    for (i, (_, l)) in VIEWS.iter().copied().enumerate() {
        vs.push(chip(p, act, ids::at(ids::CAL_VIEW, i), l, vw == i));
    }
    let bar = div()
        .child(btn(p, act, "cal-today", "Today", false))
        .child(btn(p, act, "cal-prev", "‹", false))
        .child(btn(p, act, "cal-next", "›", false))
        .child(
            div()
                .ml(px(4.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(15.))
                .whitespace_nowrap()
                .child(period_title(vw, d)),
        )
        .when(vw != 2, |b| b.child(q(p, format!("Week {}", iso_week(d)))))
        .child(div().flex_1())
        .child(seg(p, vs));
    let picked = u
        .cpick
        .and_then(|id| u.cal.iter().find(|e| e.id == id).cloned());
    let foot = match &picked {
        Some(e) => div()
            .child(div().size(px(8.)).rounded_full().bg(hue(p, CALS[e.cal].2)))
            .child(
                div()
                    .flex_none()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(e.title.clone()),
            )
            .child(clip(format!("{} · {}", iso(e.day), e.span())).text_color(p.mute))
            .child(btn(p, act, "cal-earlier", "← Day", false))
            .child(btn(p, act, "cal-later", "Day →", false))
            .child(btn(p, act, "cal-up", "↑ 15 min", false))
            .child(btn(p, act, "cal-down", "↓ 15 min", false))
            .child(btn(p, act, "cal-longer", "Longer", false)),
        None => div().child(clip(
            "Press an event to pick it, then a day header or the buttons; drag it onto a day. Alt + arrows nudge it.",
        ).text_color(p.mute)),
    };
    let loading = st == ScreenState::Loading;
    let body = if loading {
        div().p(px(20.)).child(skel(p, 12))
    } else if vw == 2 {
        month_grid(p, act, &u, d)
    } else {
        week_grid(p, act, &u, ghost)
    };
    let body = if u.cal.is_empty() && !loading {
        div().size_full().child(none(
            p,
            "No events yet",
            "Connect a calendar or add the first event.",
        ))
    } else {
        body
    };
    frame(
        p,
        layout,
        act,
        Frame {
            slug,
            side,
            ctx: Some(ctx),
            crumbs: vec![
                oxyn_core::kit::arcscr::ws(slug).name.into(),
                "Calendar".into(),
            ],
            meta: if vw == 2 {
                format!("{} events", counts.iter().sum::<usize>())
            } else {
                format!("Week {}", iso_week(d))
            },
            actions,
            bar: Some(bar),
            body,
            foot: Some(foot),
            right: (vw == 2 && !loading).then(|| day_panel(p, act, &u, d)),
        },
    )
}

fn week_grid(p: &ThPal, act: &Option<Act>, u: &Arc2Ui, ghost: Option<(u32, i32, i32)>) -> Div {
    let vw = u.cview;
    let n: i32 = if vw == 0 { 1 } else { 7 };
    let d0 = if vw == 0 { u.cday } else { week_start(u.cday) };
    let h = u.hidden;
    let live: Vec<Event> = u
        .cal
        .iter()
        .map(|e| {
            let mut e = e.clone();
            if let Some((id, dd, dm)) = ghost
                && id == e.id
            {
                e.day += dd;
                e.start += dm;
            }
            e
        })
        .collect();
    let mut head = div()
        .flex_none()
        .flex()
        .h(px(44.))
        .border_b_1()
        .border_color(p.line)
        .child(
            div()
                .w(px(56.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(q(p, format!("W{}", iso_week(d0)))),
        );
    let mut allday = div()
        .flex_none()
        .flex()
        .min_h(px(28.))
        .border_b_1()
        .border_color(p.line)
        .child(
            div()
                .w(px(56.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(q(p, "All day")),
        );
    let mut cols = div().flex_1().flex().relative();
    for (i, x) in (d0..d0 + n).enumerate() {
        let (_, dm) = month_day(x);
        let today = x == TODAY;
        head = head.child(
            hit(div(), act, ids::at(ids::CAL_DAY, i))
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .border_l_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child(WEEKDAYS[weekday(x) as usize])
                .child(
                    div()
                        .size(px(28.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(today, |d| d.bg(hue(p, "rose")).text_color(gpui::white()))
                        .when(!today, |d| d.text_color(p.fg))
                        .text_size(px(15.))
                        .child(dm.to_string()),
                ),
        );
        let mut ad = div()
            .flex_1()
            .min_w_0()
            .p(px(3.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .border_l_1()
            .border_color(p.line);
        for e in live.iter().filter(|e| e.day == x && e.all_day && !h[e.cal]) {
            ad = ad.child(
                hit(div(), act, ids::at(ids::CAL_PICK, e.id as usize))
                    .h(px(20.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .rounded(px(5.))
                    .bg(tint(p, CALS[e.cal].2, 0.18))
                    .text_size(px(11.5))
                    .child(clip(e.title.clone())),
            );
        }
        allday = allday.child(ad);
        let de = day_events(&live, x, &h);
        let ln = lanes(&de);
        let at = ids::at(ids::CAL_DAY, i);
        let a2 = act.clone();
        let mut col = div()
            .id(SharedString::from(format!("calcol-{x}")))
            .flex_1()
            .min_w_0()
            .relative()
            .h(px((HOURS.1 - HOURS.0) as f32 * HH))
            .border_l_1()
            .border_color(p.line)
            .when(weekday(x) >= 5, |d| d.bg(p.fg.opacity(0.025)))
            .drag_over::<ArcDrag>(|s, _, _, _| s.bg(gpui::black().opacity(0.06)))
            .on_drop(move |g: &ArcDrag, w, cx| {
                if let Some(f) = &a2 {
                    f(g.grab, w, cx);
                    f(at, w, cx);
                }
            });
        if today {
            col = col.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px((NOW - HOURS.0 * 60) as f32 / 60. * HH))
                    .h(px(2.))
                    .bg(hue(p, "rose")),
            );
        }
        for e in de.iter().filter(|e| !e.all_day) {
            let (lane, nl) = ln
                .iter()
                .find(|l| l.0 == e.id)
                .map(|l| (l.1, l.2))
                .unwrap_or((0, 1));
            let top = (e.start - HOURS.0 * 60) as f32 / 60. * HH;
            let hgt = (e.len as f32 / 60. * HH).max(18.);
            let hh = CALS[e.cal].2;
            let on = u.cpick == Some(e.id);
            let moving = ghost.is_some_and(|g| g.0 == e.id);
            let label: SharedString = e.title.clone().into();
            let el = hit(div(), act, ids::at(ids::CAL_PICK, e.id as usize))
                .absolute()
                .top(px(top))
                .h(px(hgt))
                .left(relative(lane as f32 / nl as f32))
                .w(relative(1. / nl as f32))
                .px(px(1.))
                .child(
                    div()
                        .size_full()
                        .overflow_hidden()
                        .rounded(px(6.))
                        .border_l_2()
                        .border_color(hue(p, hh))
                        .bg(tint(p, hh, 0.16))
                        .when(on || moving, |d| d.border_1().shadow_lg())
                        .px(px(6.))
                        .py(px(2.))
                        .flex()
                        .flex_col()
                        .text_size(px(11.5))
                        .child(clip(e.title.clone()).font_weight(FontWeight::SEMIBOLD))
                        .when(hgt >= 34., |d| {
                            d.child(
                                clip(if moving {
                                    format!("{} · {}", iso(e.day), e.span())
                                } else {
                                    e.span()
                                })
                                .text_color(p.fg2),
                            )
                        }),
                );
            col = col.child(el.on_drag(
                ArcDrag {
                    grab: ids::at(ids::CAL_GRAB, e.id as usize),
                    label,
                },
                |d: &ArcDrag, _, _, cx| {
                    let d = d.clone();
                    cx.new(|_| d)
                },
            ));
        }
        cols = cols.child(col);
    }
    let mut gut = div().w(px(56.)).flex_none().relative();
    for hr in HOURS.0 + 1..HOURS.1 {
        gut = gut.child(
            div()
                .absolute()
                .right(px(8.))
                .top(px((hr - HOURS.0) as f32 * HH - 7.))
                .text_size(px(11.))
                .text_color(p.mute)
                .child(format!("{hr:02}:00")),
        );
    }
    let mut lines = div().absolute().left(px(56.)).right_0().top_0().bottom_0();
    for hr in HOURS.0 + 1..HOURS.1 {
        lines = lines.child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(px((hr - HOURS.0) as f32 * HH))
                .h(px(1.))
                .bg(p.line.opacity(0.7)),
        );
    }
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(head)
        .child(allday)
        .child(
            div()
                .id("cal-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(div().relative().flex().child(lines).child(gut).child(cols)),
        )
}

fn month_grid(p: &ThPal, act: &Option<Act>, u: &Arc2Ui, d: i32) -> Div {
    let cells = month_cells(d);
    let first = month_of(d).0;
    let mut head = div()
        .flex_none()
        .flex()
        .h(px(30.))
        .border_b_1()
        .border_color(p.line);
    for w in WEEKDAYS {
        head = head.child(
            div()
                .flex_1()
                .px(px(10.))
                .flex()
                .items_center()
                .child(q(p, w)),
        );
    }
    let mut grid = div().flex_1().min_h_0().flex().flex_col();
    for r in 0..6 {
        let mut row = div().flex_1().min_h_0().flex();
        for c in 0..7 {
            let i = r * 7 + c;
            let x = cells[i];
            let de = day_events(&u.cal, x, &u.hidden);
            let (_, dm) = month_day(x);
            let out = month_of(x).0 != first;
            let mut cell = hit(div(), act, ids::at(ids::CAL_DAY, i))
                .flex_1()
                .min_w_0()
                .p(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .border_b_1()
                .border_color(p.line)
                .when(c < 6, |d| d.border_r_1())
                .when(out, |d| d.bg(p.fg.opacity(0.025)))
                .when(x == d, |d| d.border_2().border_color(p.fg2))
                .child(
                    div()
                        .flex_none()
                        .h(px(22.))
                        .min_w(px(24.))
                        .px(px(6.))
                        .rounded(px(11.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .when(x == TODAY, |d| {
                            d.bg(hue(p, "rose")).text_color(gpui::white())
                        })
                        .when(x != TODAY, |d| {
                            d.text_color(if out { p.mute } else { p.fg })
                        })
                        .child(dm.to_string()),
                );
            let n = de.len();
            for e in de.iter().take(if n > 3 { 2 } else { 3 }) {
                cell = cell.child(
                    div()
                        .h(px(18.))
                        .px(px(5.))
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .rounded(px(4.))
                        .when(e.all_day, |d| d.bg(tint(p, CALS[e.cal].2, 0.18)))
                        .when(!e.all_day, |d| {
                            d.border_l_2().border_color(hue(p, CALS[e.cal].2))
                        })
                        .text_size(px(11.))
                        .when(!e.all_day, |d| d.child(q(p, hhmm(e.start))))
                        .child(clip(e.title.clone())),
                );
            }
            if n > 3 {
                cell = cell.child(q(p, format!("+{} more", n - 2)));
            }
            row = row.child(cell);
        }
        grid = grid.child(row);
    }
    div().size_full().flex().flex_col().child(head).child(grid)
}

fn day_panel(p: &ThPal, act: &Option<Act>, u: &Arc2Ui, d: i32) -> Div {
    let de = day_events(&u.cal, d, &u.hidden);
    let gaps = free_gaps(&u.cal, d, &u.hidden);
    let busy = busy_minutes(&u.cal, d, &u.hidden);
    let span = (WORK.1 - WORK.0) as f32;
    let mut strip = div()
        .relative()
        .h(px(10.))
        .rounded(px(5.))
        .bg(p.sel)
        .overflow_hidden();
    for e in de
        .iter()
        .filter(|e| !e.all_day && e.end() > WORK.0 && e.start < WORK.1)
    {
        let a = (e.start.max(WORK.0) - WORK.0) as f32 / span;
        let w = (e.end().min(WORK.1) - e.start.max(WORK.0)) as f32 / span;
        strip = strip.child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(a))
                .w(relative(w))
                .bg(hue(p, CALS[e.cal].2)),
        );
    }
    let mut sched = div().flex().flex_col().gap(px(8.)).mb(px(14.));
    for e in &de {
        sched = sched.child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(52.))
                        .flex_none()
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .child(if e.all_day {
                            "All day".to_string()
                        } else {
                            hhmm(e.start)
                        }),
                )
                .child(div().size(px(7.)).rounded_full().bg(hue(p, CALS[e.cal].2)))
                .child(clip(e.title.clone())),
        );
    }
    let mut free = div().flex().flex_col().gap(px(8.)).mb(px(14.));
    for (a, b) in &gaps {
        free = free.child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(52.))
                        .flex_none()
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .child(hhmm(*a)),
                )
                .child(clip(format!("until {} · {} min", hhmm(*b), b - a)).text_color(p.mute)),
        );
    }
    div()
        .size_full()
        .p(px(16.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .flex()
                .items_center()
                .mb(px(8.))
                .child(
                    div()
                        .flex_1()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(period_title(0, d)),
                )
                .child(q(p, format!("Week {}", iso_week(d)))),
        )
        .child(strip)
        .child(q(p, format!("{busy} min busy between 09:00 and 18:00")).mb(px(10.)))
        .child(h3(p, "Schedule"))
        .child(sched)
        .child(h3(p, "Free time"))
        .child(free)
        .child(btn(p, act, "cal-focus", "Block focus time", false))
        .child(btn(
            p,
            act,
            "cal-new",
            format!("New event on {}", iso(d)),
            false,
        ))
}

impl RenderOnce for WeekCalendarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        cal_body(
            &p,
            &self.act,
            "week-calendar",
            self.state,
            self.extra,
            self.layout,
            &self.arc,
            1,
        )
    }
}

impl RenderOnce for MonthCalendarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        cal_body(
            &p,
            &self.act,
            "month-calendar",
            self.state,
            self.extra,
            self.layout,
            &self.arc,
            2,
        )
    }
}

// ───────────────────────────────────────────────────────────── inbox ──

fn cust_card(p: &ThPal, act: &Option<Act>, c: usize) -> Div {
    let (name, org, email, plan, city, _, since) = CUSTOMERS[c % CUSTOMERS.len()];
    let row = |k: &'static str, v: String| {
        div()
            .flex()
            .gap(px(12.))
            .text_size(px(12.5))
            .child(div().w(px(96.)).flex_none().text_color(p.mute).child(k))
            .child(clip(v))
    };
    let mut prev = div().flex().flex_col().gap(px(6.));
    for (s, d, stt) in previous(c) {
        prev = prev.child(
            div()
                .p(px(8.))
                .rounded(px(8.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .child(clip(s))
                .child(q(p, format!("{d} · {stt}"))),
        );
    }
    let _ = act;
    div()
        .size_full()
        .p(px(16.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .mb(px(6.))
                .child(cav(p, c, 30.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(clip(name).font_weight(FontWeight::SEMIBOLD))
                        .child(clip(org).text_color(p.mute)),
                ),
        )
        .child(row("Email", email.to_string()))
        .child(row("Local time", format!("{} in {city}", cust_time(c))))
        .child(row("Plan", plan.to_string()))
        .child(row("Customer since", since.to_string()))
        .child(h3(p, "Previous conversations"))
        .child(prev)
}

impl RenderOnce for InboxTriageEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.tri = oxyn_core::kit::arcscr2::Triage::new(st == ScreenState::Sample);
            match self.extra {
                "selected" => u.tri.sel = vec![102, 104, 108],
                "snoozing" => u.tri.snoozing = true,
                "archived" => {
                    let _ = u.tri.resolve();
                }
                _ => {}
            }
        }
        let t = &u.tri;
        let list = t.list();
        let cur = t.current();
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, l, ic)) in FOLDERS.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ids::at(ids::IB_FOLDER, i),
                glyph(&p, icon_for(ic)),
                l,
                if i == 4 {
                    String::new()
                } else {
                    t.count(i).to_string()
                },
                t.folder == i,
            ));
        }
        side = side.child(group(&p, "Team"));
        for (who, n) in IB_TEAM {
            side = side.child(
                div()
                    .h(px(34.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .child(av(&p, who, 22.))
                    .child(clip(PEOPLE[who].0).text_color(p.fg2))
                    .child(q(&p, n)),
            );
        }
        let ctx = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(div().w(px(32.)).h(px(18.)).rounded(px(9.)).bg(p.fg))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Available"))
                    .child(
                        clip("New conversations reach you")
                            .text_size(px(11.5))
                            .text_color(p.mute),
                    ),
            );
        let actions = div().child(btn(
            &p,
            &act,
            "ib-sort",
            if t.newest { "Newest ⌄" } else { "Oldest ⌄" },
            false,
        ));
        let mut tabs = Vec::new();
        for (i, l) in TABS.iter().copied().enumerate() {
            tabs.push(chip(&p, &act, ids::at(ids::IB_TAB, i), l, t.tab == i));
        }
        let mut bar = div().child(btn(
            &p,
            &act,
            "ib-all",
            if t.sel.is_empty() {
                "Select all"
            } else {
                "Clear"
            },
            false,
        ));
        if t.sel.is_empty() {
            bar = bar.child(seg(&p, tabs));
        } else {
            bar = bar
                .child(
                    div()
                        .flex_none()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("{} selected", t.sel.len())),
                )
                .child(btn(&p, &act, "ib-e", "Resolve  E", false))
                .child(btn(&p, &act, "ib-h", "Snooze  H", false))
                .child(btn(&p, &act, "ib-a", "Assign to me", false));
        }
        bar = bar.child(div().flex_1());
        bar = match t.undo.last() {
            Some((_, s)) => bar
                .child(clip(s.clone()).text_color(p.fg2))
                .child(btn(&p, &act, "ib-u", "Undo  U", false)),
            None => bar.child(q(&p, "J K move · E resolve · H snooze · X select · U undo")),
        };
        let body = if st == ScreenState::Loading {
            div()
                .size_full()
                .flex()
                .child(div().w(px(340.)).child(skel(&p, 14)))
                .child(div().flex_1().child(skel(&p, 9)))
        } else if t.convs.is_empty() {
            none(
                &p,
                "Inbox zero",
                "New conversations from email, chat, Slack and X land here.",
            )
        } else {
            let mut rows = div()
                .id("ib-list")
                .w(px(340.))
                .flex_none()
                .p(px(6.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .border_r_1()
                .border_color(p.line)
                .overflow_y_scroll();
            for (i, c) in list.iter().enumerate() {
                let on = cur.as_ref().is_some_and(|x| x.id == c.id);
                let chk = t.sel.contains(&c.id);
                rows = rows.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .p(px(8.))
                        .rounded(px(10.))
                        .when(on, |d| d.bg(p.sel))
                        .when(chk, |d| d.bg(tint(&p, "blue", 0.1)))
                        .child(
                            hit(div(), &act, ids::at(ids::IB_SEL, i))
                                .mt(px(6.))
                                .size(px(16.))
                                .flex_none()
                                .rounded(px(4.))
                                .border_1()
                                .border_color(if chk { p.fg } else { p.mute })
                                .when(chk, |d| d.bg(p.fg))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.))
                                .text_color(p.card)
                                .child(if chk { "✓" } else { "" }),
                        )
                        .child(
                            hit(div(), &act, ids::at(ids::IB_OPEN, i))
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(3.))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .child(cav(&p, c.cust, 22.))
                                        .child(chan(&p, c.channel, 12.))
                                        .child(clip(CUSTOMERS[c.cust].0).font_weight(if c.unread {
                                            FontWeight::BOLD
                                        } else {
                                            FontWeight::MEDIUM
                                        }))
                                        .child(q(
                                            &p,
                                            if c.day == TODAY {
                                                hhmm(c.at)
                                            } else {
                                                iso(c.day)
                                            },
                                        )),
                                )
                                .child(
                                    clip(format!(
                                        "{}{}",
                                        if c.unread { "● " } else { "" },
                                        c.subject
                                    ))
                                    .font_weight(FontWeight::MEDIUM),
                                )
                                .child(clip(c.preview).text_size(px(12.5)).text_color(p.mute))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(tagd(&p, TAGS[c.tag].1))
                                        .child(q(&p, TAGS[c.tag].0))
                                        .when(c.urgent, |d| d.child(pill(&p, "rose", "Urgent")))
                                        .child(div().flex_1())
                                        .children(c.sla.map(|s| sla(&p, s, ""))),
                                ),
                        ),
                );
            }
            let thread_el = match &cur {
                Some(c) => {
                    let name = CUSTOMERS[c.cust].0;
                    let mut msgs = div()
                        .id("ib-thread")
                        .flex_1()
                        .min_h_0()
                        .p(px(16.))
                        .flex()
                        .flex_col()
                        .gap(px(14.))
                        .overflow_y_scroll();
                    for (cust, dd, m, text) in thread(c) {
                        msgs = msgs.child(
                            div()
                                .flex()
                                .gap(px(10.))
                                .child(if cust {
                                    cav(&p, c.cust, 30.)
                                } else {
                                    av(&p, 0, 30.)
                                })
                                .child(
                                    card(&p)
                                        .when(!cust, |d| d.bg(p.inset))
                                        .flex_1()
                                        .min_w_0()
                                        .p(px(10.))
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.))
                                        .child(
                                            div()
                                                .flex()
                                                .gap(px(8.))
                                                .child(
                                                    div().font_weight(FontWeight::SEMIBOLD).child(
                                                        if cust { name } else { PEOPLE[0].0 },
                                                    ),
                                                )
                                                .child(q(&p, when(dd, m))),
                                        )
                                        .child(div().child(text)),
                                ),
                        );
                    }
                    let mut snz =
                        div()
                            .relative()
                            .child(btn(&p, &act, "ib-snooze", "Snooze", false));
                    if t.snoozing {
                        let mut m = card(&p)
                            .absolute()
                            .top(px(36.))
                            .right_0()
                            .w(px(250.))
                            .p(px(6.))
                            .flex()
                            .flex_col()
                            .shadow_lg();
                        for (n, (l, until)) in SNOOZES.iter().copied().enumerate() {
                            m = m.child(
                                hit(div(), &act, ids::at(ids::IB_SNOOZE, n))
                                    .h(px(32.))
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .rounded(px(8.))
                                    .child(clip(l))
                                    .child(q(&p, until)),
                            );
                        }
                        snz = snz.child(m);
                    }
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex_none()
                                .px(px(20.))
                                .pt(px(16.))
                                .pb(px(12.))
                                .border_b_1()
                                .border_color(p.line)
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(
                                    clip(c.subject)
                                        .text_size(px(16.))
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .child(chan(&p, c.channel, 13.))
                                        .child(q(&p, format!("via {}", chan_name(c.channel))))
                                        .when(c.urgent, |d| d.child(pill(&p, "rose", "Urgent")))
                                        .children(c.sla.map(|s| {
                                            sla(&p, s, if s < 0 { "Reply " } else { "Reply in " })
                                        }))
                                        .child(div().flex_1())
                                        .child(btn(
                                            &p,
                                            &act,
                                            "ib-a",
                                            if c.who == Some(0) {
                                                "Yours"
                                            } else {
                                                "Assign to me"
                                            },
                                            false,
                                        ))
                                        .child(snz)
                                        .child(btn(&p, &act, "ib-e", "Resolve", true)),
                                ),
                        )
                        .child(msgs)
                }
                None => div().flex_1().child(none(
                    &p,
                    "All caught up",
                    "Pick a conversation, or press J.",
                )),
            };
            div().size_full().flex().child(rows).child(thread_el)
        };
        let right = cur.as_ref().map(|c| cust_card(&p, &act, c.cust));
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "inbox-triage",
                side,
                ctx: Some(ctx),
                crumbs: vec!["Thistledown".into(), FOLDERS[t.folder].1.into()],
                meta: format!("{} open · {} unread", t.count(0), t.unread()),
                actions,
                bar: Some(bar),
                body,
                foot: None,
                right: if st == ScreenState::Loading {
                    None
                } else {
                    right
                },
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── support ──

impl RenderOnce for SupportConversationEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.desk = oxyn_core::kit::arcscr2::Desk::new(st == ScreenState::Sample);
            match self.extra {
                "note" => {
                    u.desk.note = true;
                    u.desk.draft = "Told Jonah the fix date; check their Monday export.".into();
                }
                "sent" => {
                    u.desk.saved(0);
                    let _ = u.desk.send();
                }
                "closed" => {
                    let _ = u.desk.close();
                }
                _ => {}
            }
        }
        let dk = &u.desk;
        let list = dk.list();
        let cur = dk.current().cloned();
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (_, l, ic)) in SP_VIEWS.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ids::at(ids::SP_VIEW, i),
                glyph(&p, icon_for(ic)),
                l,
                if i == 5 {
                    String::new()
                } else {
                    dk.count(i).to_string()
                },
                dk.view == i,
            ));
        }
        side = side.child(group(&p, "Waiting on you"));
        for (c, s) in waiting(dk) {
            side = side.child(
                div()
                    .h(px(34.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .child(cav(&p, c, 22.))
                    .child(clip(CUSTOMERS[c].0).text_color(p.fg2))
                    .child(sla(&p, s, "")),
            );
        }
        let ctx = div()
            .flex()
            .flex_col()
            .child(div().font_weight(FontWeight::SEMIBOLD).child("Available"))
            .child(q(&p, format!("{} tickets in your inbox", dk.count(0))));
        let closed = cur.as_ref().is_some_and(|t| t.status == TStatus::Closed);
        let actions = div()
            .child(btn(&p, &act, "sp-take", "Assign to me", false))
            .child(btn(&p, &act, "sp-snooze-0", "Snooze", false))
            .child(btn(
                &p,
                &act,
                "sp-close",
                if closed { "Reopen" } else { "Close" },
                !closed,
            ));
        let body = if st == ScreenState::Loading {
            div()
                .size_full()
                .flex()
                .child(div().w(px(300.)).child(skel(&p, 12)))
                .child(div().flex_1().child(skel(&p, 10)))
        } else if dk.tickets.is_empty() {
            none(
                &p,
                "No tickets yet",
                "Tickets from email, chat and Slack show here with their SLA.",
            )
        } else {
            let mut rows = div()
                .id("sp-list")
                .w(px(300.))
                .flex_none()
                .p(px(6.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .border_r_1()
                .border_color(p.line)
                .overflow_y_scroll();
            for (i, t) in list.iter().enumerate() {
                let on = dk.open == Some(t.id);
                rows = rows.child(
                    hit(div(), &act, ids::at(ids::SP_OPEN, i))
                        .p(px(10.))
                        .rounded(px(10.))
                        .when(on, |d| d.bg(p.sel))
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(cav(&p, t.cust, 22.))
                                .child(clip(CUSTOMERS[t.cust].0).font_weight(FontWeight::MEDIUM))
                                .child(q(&p, format!("#{}", t.id))),
                        )
                        .child(clip(t.subject).font_weight(FontWeight::MEDIUM))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .size(px(7.))
                                        .rounded_full()
                                        .bg(hue(&p, t.status.hue())),
                                )
                                .child(clip(t.status.label()).text_size(px(11.5)).text_color(p.fg2))
                                .children(t.sla.map(|s| sla(&p, s, ""))),
                        ),
                );
            }
            let thread_el = match &cur {
                Some(t) => {
                    let name = CUSTOMERS[t.cust].0;
                    let mut msgs = div()
                        .id("sp-thread")
                        .flex_1()
                        .min_h_0()
                        .p(px(16.))
                        .flex()
                        .flex_col()
                        .gap(px(14.))
                        .overflow_y_scroll();
                    for m in &t.msgs {
                        if m.kind == Kind::Event {
                            msgs = msgs.child(
                                div()
                                    .flex()
                                    .justify_center()
                                    .child(q(&p, format!("{} · {}", m.text, m.at))),
                            );
                            continue;
                        }
                        let (who, me, note) = match m.kind {
                            Kind::Reply(w) => (PEOPLE[w].0, Some(w), false),
                            Kind::Note(w) => (PEOPLE[w].0, Some(w), true),
                            _ => (name, None, false),
                        };
                        let mut bubble = card(&p)
                            .flex_1()
                            .min_w_0()
                            .p(px(10.))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .when(me.is_some(), |d| d.bg(p.inset))
                            .when(note, |d| {
                                d.bg(tint(&p, "amber", 0.12))
                                    .border_color(tint(&p, "amber", 0.45))
                            })
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(who))
                                    .when(note, |d| {
                                        d.child(
                                            div()
                                                .text_size(px(11.5))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(hue(&p, "amber"))
                                                .child("Internal note"),
                                        )
                                    })
                                    .child(q(&p, m.at.clone())),
                            )
                            .child(div().child(m.text.clone()));
                        if m.article.is_some() {
                            bubble = bubble.child(
                                card(&p)
                                    .p(px(8.))
                                    .flex()
                                    .gap(px(8.))
                                    .child(div().text_color(hue(&p, "rose")).child("▤"))
                                    .child(clip("Exports time out · Troubleshooting")),
                            );
                        }
                        for f in &m.files {
                            bubble = bubble.child(q(&p, format!("▤ {f}")));
                        }
                        msgs = msgs.child(
                            div()
                                .flex()
                                .gap(px(10.))
                                .child(match me {
                                    Some(w) => av(&p, w, 30.),
                                    None => cav(&p, t.cust, 30.),
                                })
                                .child(bubble),
                        );
                    }
                    let mut saved = div().flex().gap(px(6.)).flex_wrap();
                    for (i, (l, _)) in SAVED.iter().copied().enumerate() {
                        saved = saved.child(chip(&p, &act, ids::at(ids::SP_SAVED, i), l, false));
                    }
                    let mut files = div().flex().gap(px(6.));
                    for f in &dk.files {
                        files = files.child(q(&p, format!("▤ {f}")));
                    }
                    if dk.article.is_some() {
                        files = files.child(q(&p, "▤ Exports time out"));
                    }
                    let composer = card(&p)
                        .flex_none()
                        .mx(px(20.))
                        .mb(px(16.))
                        .p(px(10.))
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .when(dk.note, |d| {
                            d.bg(tint(&p, "amber", 0.09))
                                .border_color(tint(&p, "amber", 0.45))
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(seg(
                                    &p,
                                    vec![
                                        chip(&p, &act, "sp-reply", "Reply", !dk.note),
                                        chip(&p, &act, "sp-note", "Note", dk.note),
                                    ],
                                ))
                                .child(
                                    clip(if dk.note {
                                        "Only your team sees notes".to_string()
                                    } else {
                                        format!("To {name}")
                                    })
                                    .text_color(p.mute),
                                ),
                        )
                        .child(match self.field.clone() {
                            Some(v) => div().min_h(px(44.)).child(v),
                            None => div()
                                .min_h(px(44.))
                                .text_color(if dk.draft.is_empty() { p.mute } else { p.fg })
                                .child(if dk.draft.is_empty() {
                                    "Write a reply…".to_string()
                                } else {
                                    dk.draft.clone()
                                }),
                        })
                        .child(files)
                        .child(saved)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(btn(&p, &act, "sp-article", "Article", false))
                                .child(btn(&p, &act, "sp-attach", "Attach", false))
                                .child(div().flex_1())
                                .child(btn(
                                    &p,
                                    &act,
                                    "sp-send",
                                    if dk.note { "Add note" } else { "Send" },
                                    true,
                                )),
                        );
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex_none()
                                .px(px(20.))
                                .pt(px(16.))
                                .pb(px(12.))
                                .border_b_1()
                                .border_color(p.line)
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(
                                    clip(t.subject)
                                        .text_size(px(16.))
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .child(chan(&p, t.channel, 13.))
                                        .child(
                                            div()
                                                .size(px(7.))
                                                .rounded_full()
                                                .bg(hue(&p, t.status.hue())),
                                        )
                                        .child(q(&p, t.status.label()))
                                        .when(t.urgent, |d| d.child(pill(&p, "rose", "Urgent")))
                                        .children(t.sla.map(|s| {
                                            sla(
                                                &p,
                                                s,
                                                if s < 0 { "First reply " } else { "Reply in " },
                                            )
                                        })),
                                ),
                        )
                        .child(msgs)
                        .child(composer)
                }
                None => {
                    div()
                        .flex_1()
                        .child(none(&p, "No ticket open", "Pick a ticket on the left."))
                }
            };
            div().size_full().flex().child(rows).child(thread_el)
        };
        let right = cur.as_ref().map(|t| {
            let mut c = cust_card(&p, &act, t.cust);
            c = c
                .child(h3(&p, "Apps"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .children(mark(&p, "googlecalendar", 16.))
                        .child(clip("Google Calendar · call 2026-11-13 10:00")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .children(mark(&p, "stripe", 16.))
                        .child(clip("Stripe · invoice paid 2026-11-01")),
                );
            if !t.issue.is_empty() {
                c = c.child(h3(&p, "Linked issue")).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .children(mark(&p, "linear", 15.))
                        .child(clip(t.issue)),
                );
            }
            c
        });
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "support-conversation",
                side,
                ctx: Some(ctx),
                crumbs: vec![
                    "Pinecrest".into(),
                    SP_VIEWS[dk.view].1.into(),
                    cur.as_ref()
                        .map(|t| format!("#{}", t.id))
                        .unwrap_or_default(),
                ],
                meta: cur
                    .as_ref()
                    .map(|t| CUSTOMERS[t.cust].0.to_string())
                    .unwrap_or_default(),
                actions,
                bar: None,
                body,
                foot: None,
                right: if st == ScreenState::Loading {
                    None
                } else {
                    right
                },
            },
        )
    }
}

// ───────────────────────────────────────────────────────────── help center ──

impl RenderOnce for HelpCenterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let act = self.act.clone();
        let st = self.state;
        let mut u = live(&self.arc);
        if self.arc.is_none() {
            u.help = oxyn_core::kit::arcscr2::Help::new(st == ScreenState::Sample);
            match self.extra {
                "search" => {
                    u.help.query = "cancel".into();
                    u.help.open_id(5);
                }
                "noresults" => u.help.query = "dark mode".into(),
                "feedback" => {
                    u.help.open_id(12);
                    u.help.vote(false);
                    u.help.reason = Some(2);
                }
                _ => {}
            }
        }
        let h = &u.help;
        let shown = h.shown();
        let n = tab_counts(&h.arts, h.coll);
        let cur = h.current().cloned();
        let (check, pct) = quarter(&h.arts);
        let mut side = div().flex().flex_col().gap(px(1.));
        for (i, (k, ic, l, c)) in HC_NAV.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                if i == 0 { "nav-helpcenter" } else { k },
                glyph(&p, icon_for(ic)),
                l,
                if i == 0 {
                    h.arts.len().to_string()
                } else {
                    c.to_string()
                },
                i == 0,
            ));
        }
        side = side.child(group(&p, "Collections"));
        for (i, (name, hh, _)) in COLLECTIONS.iter().copied().enumerate() {
            side = side.child(nrow(
                &p,
                &act,
                ids::at(ids::HC_COLL, i),
                div().w(px(16.)).flex().justify_center().child(tagd(&p, hh)),
                name,
                h.arts.iter().filter(|a| a.coll == i).count().to_string(),
                h.coll == Some(i),
            ));
        }
        let ctx = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .child(div().flex_1().child("Quarterly review"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{pct}%")),
                    ),
            )
            .child(
                div().h(px(4.)).rounded(px(2.)).bg(p.sel).child(
                    div()
                        .h_full()
                        .w(relative(pct as f32 / 100.))
                        .rounded(px(2.))
                        .bg(p.fg),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(clip(format!("{check} articles to check")).text_color(p.mute))
                    .child(btn(&p, &act, "hc-tab-3", "Review", false)),
            );
        let actions = div().child(btn(&p, &act, "new-article", "+ New article", true));
        let mut tabs = Vec::new();
        for (i, (_, l)) in ART_TABS.iter().copied().enumerate() {
            tabs.push(chip(
                &p,
                &act,
                ids::at(ids::HC_TAB, i),
                format!("{l} {}", n[i]),
                h.tab == i,
            ));
        }
        let bar = div()
            .child(seg(&p, tabs))
            .child(div().flex_1())
            .when(!h.query.is_empty(), |d| {
                d.child(btn(&p, &act, "hc-clear", "Clear", false))
            })
            .child(field_box(
                &p,
                self.field.clone(),
                if h.query.is_empty() {
                    "Search articles".into()
                } else {
                    h.query.clone()
                },
                260.,
            ));
        let body = if st == ScreenState::Loading {
            div()
                .size_full()
                .flex()
                .child(div().w(px(340.)).child(skel(&p, 12)))
                .child(div().flex_1().child(skel(&p, 14)))
        } else if h.arts.is_empty() {
            none(
                &p,
                "No articles yet",
                "Write the first article or import them from your old help site.",
            )
        } else {
            let mut list = div()
                .id("hc-list")
                .w(px(340.))
                .flex_none()
                .p(px(8.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .border_r_1()
                .border_color(p.line)
                .overflow_y_scroll();
            if !h.query.is_empty() {
                list = list.child(q(
                    &p,
                    if shown.is_empty() {
                        format!("No articles match “{}”", h.query)
                    } else {
                        format!("{} results for “{}”", shown.len(), h.query)
                    },
                ));
                if shown.is_empty() {
                    let mut sug = div().flex().flex_wrap().gap(px(6.)).my(px(8.));
                    for (i, (s, _)) in SEARCHES.iter().copied().enumerate() {
                        if !oxyn_core::kit::arcscr2::search(&h.arts, s).is_empty() {
                            sug = sug.child(chip(&p, &act, ids::at(ids::HC_Q, i), s, false));
                        }
                    }
                    list = list.child(sug);
                }
            }
            for (i, (a, hit_)) in shown.iter().enumerate() {
                let on = h.open == Some(a.id);
                let (cn, hh, _) = COLLECTIONS[a.coll];
                let (sl, sh) = match a.status {
                    2 => ("Draft", "mute"),
                    3 => ("In review", "amber"),
                    _ => ("Published", "green"),
                };
                list = list.child(
                    hit(div(), &act, ids::at(ids::HC_OPEN, i))
                        .p(px(10.))
                        .rounded(px(10.))
                        .when(on, |d| d.bg(p.sel))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(tagd(&p, hh))
                                .child(clip(a.title).font_weight(FontWeight::MEDIUM)),
                        )
                        .children(hit_.as_ref().map(|x| {
                            clip(x.snippet.clone())
                                .text_size(px(12.5))
                                .text_color(p.fg2)
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(div().size(px(7.)).rounded_full().bg(hue(&p, sh)))
                                .child(q(&p, sl))
                                .child(clip(cn).text_size(px(12.)).text_color(p.mute))
                                .child(q(&p, iso(a.updated))),
                        ),
                );
            }
            let reader = match &cur {
                Some(a) => {
                    let up = helpful(a);
                    let mut art = div()
                        .id("hc-read")
                        .flex_1()
                        .min_w_0()
                        .px(px(32.))
                        .py(px(22.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .overflow_y_scroll()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(tagd(&p, COLLECTIONS[a.coll].1))
                                .child(q(&p, COLLECTIONS[a.coll].0))
                                .child(div().flex_1())
                                .when(a.status != 1, |d| {
                                    d.child(btn(&p, &act, "hc-publish", "Publish", true))
                                }),
                        )
                        .child(
                            div()
                                .mt(px(8.))
                                .text_size(px(24.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(a.title),
                        )
                        .child(q(
                            &p,
                            format!(
                                "{} · updated {} · {} views",
                                PEOPLE[a.owner].0,
                                iso(a.updated),
                                thousands(a.views as u64)
                            ),
                        ));
                    for (hd, tx) in a.body {
                        art = art
                            .child(
                                div()
                                    .mt(px(12.))
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(*hd),
                            )
                            .child(div().text_color(p.fg2).child(*tx));
                    }
                    if !a.related.is_empty() {
                        art = art.child(h3(&p, "Related"));
                        for r in a.related {
                            let t = h
                                .arts
                                .iter()
                                .find(|x| x.id == *r)
                                .map(|x| x.title)
                                .unwrap_or("");
                            art = art.child(
                                hit(div(), &act, ids::at(ids::HC_ART, *r as usize))
                                    .h(px(30.))
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .rounded(px(8.))
                                    .text_color(p.fg2)
                                    .child("▤")
                                    .child(clip(t)),
                            );
                        }
                    }
                    let mut fb = card(&p)
                        .mt(px(20.))
                        .p(px(14.))
                        .bg(p.inset)
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.));
                    if h.sent {
                        fb = fb.child(format!(
                            "✓ Thanks. Your note went to {}.",
                            PEOPLE[a.owner].0
                        ));
                    } else if h.vote == Some(true) {
                        fb = fb.child(format!(
                            "✓ Thanks for the feedback. {up}% found this helpful."
                        ));
                    } else if h.vote == Some(false) {
                        fb = fb.child(
                            div()
                                .w_full()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("What went wrong?"),
                        );
                        for (i, r) in REASONS.iter().copied().enumerate() {
                            fb = fb.child(chip(
                                &p,
                                &act,
                                ids::at(ids::HC_REASON, i),
                                r,
                                h.reason == Some(i),
                            ));
                        }
                        fb = fb.child(div().flex_1()).child(btn(
                            &p,
                            &act,
                            "hc-send",
                            "Send feedback",
                            h.reason.is_some(),
                        ));
                    } else {
                        fb = fb
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Was this helpful?"),
                            )
                            .child(btn(&p, &act, "hc-yes", "Yes", false))
                            .child(btn(&p, &act, "hc-no", "No", false))
                            .child(div().flex_1())
                            .child(q(&p, format!("{up}% helpful · {} votes", a.up + a.down)));
                    }
                    art = art.child(fb);
                    let mut tops = div().flex().flex_col().gap(px(6.));
                    for (i, (s, c)) in SEARCHES.iter().copied().enumerate() {
                        let empty = oxyn_core::kit::arcscr2::search(&h.arts, s).is_empty();
                        tops = tops.child(
                            hit(div(), &act, ids::at(ids::HC_Q, i))
                                .p(px(8.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(p.line)
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(clip(s))
                                .when(empty, |d| d.child(pill(&p, "rose", "No results")))
                                .child(q(&p, c.to_string())),
                        );
                    }
                    let ins = div()
                        .w(px(250.))
                        .flex_none()
                        .p(px(18.))
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .border_l_1()
                        .border_color(p.line)
                        .child(q(&p, "Views, 30 days"))
                        .child(div().text_size(px(18.)).child(thousands(a.views as u64)))
                        .child(q(&p, "Helpful"))
                        .child(div().text_size(px(18.)).child(format!("{up}%")))
                        .child(h3(&p, "Top searches"))
                        .child(tops);
                    div().flex_1().min_w_0().flex().child(art).child(ins)
                }
                None => div().flex_1().child(none(
                    &p,
                    "Pick an article",
                    "Choose an article on the left.",
                )),
            };
            div().size_full().flex().child(list).child(reader)
        };
        frame(
            &p,
            self.layout,
            &act,
            Frame {
                slug: "help-center",
                side,
                ctx: Some(ctx),
                crumbs: vec!["Quillmoor Help".into(), "Articles".into()],
                meta: format!("{} articles", h.arts.len()),
                actions,
                bar: Some(bar),
                body,
                foot: None,
                right: None,
            },
        )
    }
}
