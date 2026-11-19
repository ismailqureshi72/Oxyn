//! Screens S13 (Arc screens II): **Week calendar**, **Month calendar**, **Inbox triage**,
//! **Support conversation** and **Help center** on the Arc frame shared with S12 (`arcscr.rs`):
//! a 248 px sidebar with a context card and an account row with the email, an inset page and an
//! optional right card. Rules in `oxyn_core::parity::arcscr2`; styles in `arcscr2.css`
//! (`oxs13-`). Chrome stays monochrome; colour is for calendars, labels, SLAs, statuses and
//! official marks.

use dioxus::prelude::*;
use oxyn_core::parity::arcscr2::{
    ART_TABS, CALS, COLLECTIONS, CUSTOMERS, Desk, Event as CEvent, FOLDERS, HC_NAV, HOURS, Help,
    IB_TEAM, Kind, NOW, PEOPLE, REASONS, SAVED, SEARCHES, SNOOZES, SP_TEAMS, SP_VIEWS, ScreenState,
    TABS, TAGS, TODAY, TStatus, Triage, VIEWS, WEEKDAYS, WORK, add_event, arc2_split, block_focus,
    busy_minutes, cal_counts, cal_nav, cust_hue, cust_ini, cust_time, day_events, events,
    free_gaps, helpful, hhmm, iso, iso_week, lanes, month_cells, month_of, month_step, nudge_event,
    period_title, previous, quarter, resize_event, s13_states as core_states, sla_hue, sla_words,
    snap, tab_counts, thousands, thread, waiting, week_start, weekday, when,
};

use super::arcscr::{ArcFrame, av, btn, find_field, group, none, nrow, seg, skl};
use super::screensfid::{MenuButton, nico, stage_scale};
use crate::brand::BrandIcon;

/// The state bar entries of an S13 screen (for a host's `ScreenStage { states }`).
pub fn s13_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    core_states(slug).to_vec()
}

// ───────────────────────────────────────────────────────────── shared parts ──

fn cav(c: usize, small: bool) -> Element {
    let n = CUSTOMERS[c % CUSTOMERS.len()].0;
    let hue = cust_hue(c);
    let ini = cust_ini(c);
    rsx! { span { class: "oxs12-av", "data-hue": "{hue}", "data-sm": "{small}", title: "{n}", "aria-hidden": "true", "{ini}" } }
}

fn channel(ch: &str, size: f32) -> Element {
    match ch {
        "chat" => rsx! { span { class: "oxs13-chm", title: "Site chat", {nico("chat")} } },
        b => {
            rsx! { span { class: "oxs13-chm", BrandIcon { name: b.to_string(), size, coloured: true } } }
        }
    }
}

fn channel_name(ch: &str) -> &'static str {
    match ch {
        "gmail" => "Gmail",
        "slack" => "Slack",
        "x" => "X",
        _ => "Site chat",
    }
}

fn pill(hue: &str, text: String) -> Element {
    rsx! { span { class: "oxs13-pill", "data-hue": "{hue}", "{text}" } }
}

fn first_name(c: usize) -> &'static str {
    CUSTOMERS[c % CUSTOMERS.len()]
        .0
        .split_whitespace()
        .next()
        .unwrap_or("")
}

// ───────────────────────────────────────────────────────────── calendar ──

#[derive(Clone, Copy, PartialEq)]
struct CDrag {
    id: u32,
    /// 0 move, 1 resize the end.
    mode: u8,
    x0: f64,
    y0: f64,
    dx: f64,
    dy: f64,
    /// A fixed (days, minutes) offset (the Dragging state).
    fixed: Option<(i32, i32)>,
}

const HOUR_H: f64 = 48.0;

fn ytop(m: i32) -> f64 {
    (m - HOURS.0 * 60) as f64 / 60.0 * HOUR_H
}

fn hours_words(m: i32) -> String {
    match (m / 60, m % 60) {
        (0, x) => format!("{x} min"),
        (h, 0) => format!("{h} h"),
        (h, x) => format!("{h} h {x} min"),
    }
}

/// Screen – Week calendar: a time grid of the week (Day and Month a press away), events tinted by
/// calendar with places, video marks and guests; drag an event to move it (its bottom edge to
/// resize), drag on an empty slot to create one; keys move and resize the focused event. States:
/// `sample`, `loading`, `empty`, `dragging`, `creating`, `dayview`.
#[component]
pub fn WeekCalendarScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    rsx! { CalendarScreen { slug: "week-calendar", layout, state, start: 1 } }
}

/// Screen – Month calendar: the month grid with event chips and "N more", a day panel (busy
/// strip, schedule, free gaps with Block focus time, New event), calendars with counts and Only;
/// Week and Day a press away. States: `sample`, `loading`, `empty`, `more`, `hidden`, `focus`.
#[component]
pub fn MonthCalendarScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    rsx! { CalendarScreen { slug: "month-calendar", layout, state, start: 2 } }
}

#[component]
fn CalendarScreen(slug: &'static str, layout: String, state: String, start: usize) -> Element {
    let (st, own) = arc2_split(&state);
    let sample = st != ScreenState::Empty;
    let mut said = use_signal(String::new);
    let mut evs = use_signal(move || {
        let mut v = events(sample);
        if own == "focus" {
            block_focus(&mut v, TODAY, &[false; 6]);
        }
        v
    });
    let mut view = use_signal(move || if own == "dayview" { 0usize } else { start });
    let mut day = use_signal(move || if own == "more" { 48 } else { TODAY });
    let mut hidden = use_signal(move || {
        let mut h = [false; 6];
        if own == "hidden" {
            h[4] = true;
            h[5] = true;
        }
        h
    });
    let mut sel = use_signal(move || if own == "dragging" { Some(11u32) } else { None });
    let mut drag = use_signal(move || {
        (own == "dragging").then_some(CDrag {
            id: 11,
            mode: 0,
            x0: 0.0,
            y0: 0.0,
            dx: 0.0,
            dy: 0.0,
            fixed: Some((1, 30)),
        })
    });
    // drag on an empty slot: (day, minute pressed, y0, dy)
    let mut mk = use_signal(|| None::<(i32, i32, f64, f64)>);
    let mut draft = use_signal(move || (own == "creating").then_some((TODAY, 10 * 60 + 30, 60)));
    let mut title = use_signal(move || {
        if own == "creating" {
            "Pricing sync with Mira".to_string()
        } else {
            String::new()
        }
    });
    let mut dcal = use_signal(|| 0usize);
    let mut more = use_signal(move || (own == "more").then_some(48));
    let mut colw = use_signal(|| 160.0f64);
    let q = use_signal(String::new);
    let loading = st == ScreenState::Loading;
    let vw = view();
    let d = day();
    let h = hidden();
    let qq = q().to_lowercase();
    let all = evs();
    let shown = |e: &CEvent| qq.is_empty() || e.title.to_lowercase().contains(&qq);
    let ws_name = oxyn_core::parity::arcscr::ws(slug).name;
    let nav = cal_nav(slug);
    let counts = cal_counts(&all, d);
    let ndays: i32 = if vw == 0 { 1 } else { 7 };
    let d0 = if vw == 0 { d } else { week_start(d) };
    // the live geometry of the dragged event
    let dg = drag();
    let cw = colw();
    let off = |g: &CDrag| -> (i32, i32) {
        g.fixed.unwrap_or((
            if ndays == 1 {
                0
            } else {
                (g.dx / cw.max(1.0)).round() as i32
            },
            snap((g.dy / HOUR_H * 60.0) as i32),
        ))
    };
    let live: Vec<CEvent> = all
        .iter()
        .map(|e| {
            let mut e = e.clone();
            if let Some(g) = dg
                && g.id == e.id
            {
                let (dd, dm) = off(&g);
                if g.mode == 1 {
                    e.len = (e.len + dm).max(15);
                } else {
                    e.day += dd;
                    e.start = (e.start + dm).clamp(0, 24 * 60 - e.len);
                }
            }
            e
        })
        .filter(shown)
        .collect();
    let mut end_drag = move || {
        let g0 = *drag.peek();
        if let Some(g) = g0 {
            if g.fixed.is_none() {
                let (dd, dm) = {
                    let c = *colw.peek();
                    let n = if *view.peek() == 0 {
                        0
                    } else {
                        (g.dx / c.max(1.0)).round() as i32
                    };
                    (n, snap((g.dy / HOUR_H * 60.0) as i32))
                };
                let msg = if g.mode == 1 {
                    resize_event(&mut evs.write(), g.id, dm)
                } else if dd != 0 || dm != 0 {
                    nudge_event(&mut evs.write(), g.id, dd, dm)
                } else {
                    None
                };
                if let Some(m) = msg {
                    said.set(m);
                }
            }
            drag.set(None);
        }
        let m0v = *mk.peek();
        if let Some((dday, m0, _, dy)) = m0v {
            let m1 = m0 + snap((dy / HOUR_H * 60.0) as i32);
            let (a, b) = (m0.min(m1), m0.max(m1));
            let len = (b - a).max(30);
            draft.set(Some((dday, a, len)));
            title.set(String::new());
            said.set(format!(
                "New event {} {}–{}",
                iso(dday),
                hhmm(a),
                hhmm(a + len)
            ));
            mk.set(None);
        }
    };
    let mut save = move || {
        let d0v = *draft.peek();
        if let Some((dd, a, len)) = d0v {
            let t = title.peek().clone();
            let id = add_event(&mut evs.write(), dd, a, len, &t, *dcal.peek());
            sel.set(Some(id));
            draft.set(None);
            said.set(format!(
                "{} added, {} {}–{}",
                if t.trim().is_empty() {
                    "New event"
                } else {
                    t.trim()
                },
                iso(dd),
                hhmm(a),
                hhmm(a + len)
            ));
        }
    };
    let mut go = move |dir: i32| {
        let v = *view.peek();
        let cur = *day.peek();
        let nd = match v {
            0 => cur + dir,
            2 => month_step(cur, dir),
            _ => cur + 7 * dir,
        }
        .clamp(-30, 180);
        day.set(nd);
        said.set(period_title(v, nd));
    };
    let mut set_view = move |v: usize| {
        view.set(v);
        more.set(None);
        said.set(format!(
            "{} view, {}",
            VIEWS[v].1,
            period_title(v, *day.peek())
        ));
    };
    let root_key = move |e: KeyboardEvent| {
        let k = e.key();
        match k {
            Key::Character(c) if !e.modifiers().ctrl() && !e.modifiers().meta() => match c.as_str()
            {
                "t" => {
                    day.set(TODAY);
                    said.set(format!("Today, {}", iso(TODAY)));
                }
                "d" => set_view(0),
                "w" => set_view(1),
                "m" => set_view(2),
                "n" => {
                    let dd = *day.peek();
                    let a = free_gaps(&evs.peek(), dd, &hidden.peek())
                        .first()
                        .map(|g| g.0)
                        .unwrap_or(WORK.0);
                    draft.set(Some((dd, a, 30)));
                    said.set(format!("New event {} {}", iso(dd), hhmm(a)));
                }
                _ => {}
            },
            Key::Escape => {
                if more.peek().is_some() {
                    more.set(None);
                } else if draft.peek().is_some() {
                    draft.set(None);
                } else {
                    sel.set(None);
                }
            }
            _ => {}
        }
    };
    let sel_ev = sel().and_then(|id| all.iter().find(|e| e.id == id).cloned());
    let (week_lbl, title_lbl) = (format!("Week {}", iso_week(d)), period_title(vw, d));
    let mini = month_cells(d);
    let (_, mlabel) = month_of(d);
    let view_items: Vec<(&'static str, String)> =
        VIEWS.iter().map(|v| (v.0, v.1.to_string())).collect();
    let next_up = all
        .iter()
        .filter(|e| e.day == TODAY && !e.all_day && e.start > NOW)
        .min_by_key(|e| e.start)
        .cloned();
    let month_n = counts.iter().sum::<usize>();
    let busy_month: i32 = mini
        .iter()
        .filter(|x| month_of(**x).0 == month_of(d).0)
        .map(|x| busy_minutes(&all, *x, &h))
        .sum();
    rsx! {
        ArcFrame { slug, layout, said,
            crumbs: vec![ws_name.to_string(), "Calendar".to_string()],
            meta: if vw == 2 { format!("{month_n} events") } else { week_lbl.clone() },
            show_right: vw == 2 && !loading,
            side: rsx! {
                for (i, (_, ic, l, n)) in nav.iter().copied().enumerate() {
                    {nrow(nico(ic), l.to_string(), n.to_string(), i == 0, EventHandler::new(move |_| said.set(format!("{l} opened"))))}
                }
                if slug == "week-calendar" {
                    {group("Jump to date", false)}
                    div { class: "oxs13-mini", role: "group", "aria-label": "Jump to date",
                        div { class: "oxs13-minih",
                            b { "{mlabel}" }
                            span { class: "oxs12-grow" }
                            button { r#type: "button", class: "oxs12-ib oxs13-ibs", "aria-label": "Previous month", onclick: move |_| { let nd = month_step(*day.peek(), -1); day.set(nd); said.set(month_of(nd).1.to_string()); }, span { class: "oxs13-flip", {nico("next")} } }
                            button { r#type: "button", class: "oxs12-ib oxs13-ibs", "aria-label": "Next month", onclick: move |_| { let nd = month_step(*day.peek(), 1); day.set(nd); said.set(month_of(nd).1.to_string()); }, {nico("next")} }
                        }
                        div { class: "oxs13-minig",
                            for (i, w) in WEEKDAYS.iter().enumerate() { span { key: "w{i}", class: "oxs13-minid", "aria-hidden": "true", "{&w[..1]}" } }
                            for x in mini.iter().copied() {
                                { let out = month_of(x).0 != month_of(d).0; let inwk = week_start(x) == week_start(d); let (_, dm) = oxyn_core::parity::arcscr2::month_day(x);
                                  rsx! { button { key: "{x}", r#type: "button", class: "oxs13-minic", "data-out": "{out}", "data-wk": "{inwk && vw != 2}", "data-today": "{x == TODAY}", "aria-pressed": "{x == d}", "aria-label": "{iso(x)}",
                                    onclick: move |_| { day.set(x); said.set(period_title(*view.peek(), x)); }, "{dm}" } } }
                            }
                        }
                    }
                }
                {group(if slug == "week-calendar" { "Calendars" } else { "My calendars" }, true)}
                for (i, (_, name, hue, g)) in CALS.iter().copied().enumerate() {
                    if i == 4 { div { key: "{i}pg", class: "oxs13-sub", "Personal" } }
                    { let on = !h[i]; let n = counts[i]; let _ = g;
                      rsx! { div { key: "c{i}", class: "oxs13-calr",
                        label { class: "oxs13-cbx",
                            input { r#type: "checkbox", checked: on, "aria-label": "Show {name}", onchange: move |e| { hidden.write()[i] = !e.checked(); said.set(format!("{name} {}", if e.checked() { "shown" } else { "hidden" })); } }
                            span { class: "oxs13-box", "data-hue": "{hue}", "aria-hidden": "true", {nico("check")} }
                            span { class: "oxs12-ell", "{name}" }
                        }
                        span { class: "oxs12-grow" }
                        if slug == "month-calendar" { span { class: "oxs12-nrc", "{n}" } }
                        button { r#type: "button", class: "oxs12-ib oxs13-only", "aria-label": "Show only {name}", title: "Only",
                            onclick: move |_| { let mut w = [true; 6]; w[i] = false; hidden.set(w); said.set(format!("Only {name}")); }, {nico("eye")} }
                      } } }
                }
                {group(if slug == "week-calendar" { "People" } else { "Team calendars" }, false)}
                for (j, (who, st2, hue)) in [(1usize, "Free until 15:00", "green"), (2, "In a meeting until 13:30", "amber"), (5, "Out until 2026-11-16", "mute")].into_iter().enumerate() {
                    div { key: "p{j}", class: "oxs13-pr", {av(who, true)} span { class: "oxs12-an", b { class: "oxs12-ell", "{PEOPLE[who].0}" } span { class: "oxs12-ell", "data-hue": "{hue}", "{st2}" } } }
                }
            },
            card: rsx! {
                if slug == "week-calendar" {
                    div { class: "oxs13-next",
                        div { class: "oxs12-cr1", span { class: "oxs12-hi", "data-hue": "blue", {nico("clock")} } span { class: "oxs12-ell", "Next up" } }
                        if let Some(e) = next_up.clone() {
                            { let t = e.title.clone(); let mins = e.start - NOW;
                              rsx! {
                                b { class: "oxs12-ell oxs13-nt", "{t}" }
                                span { class: "oxs12-q", "{e.span()} · in {hours_words(mins)}" }
                                if e.place == "zoom" { button { r#type: "button", class: "oxs12-btn oxs12-sm oxs13-join", onclick: move |_| said.set(format!("Joining {t}")), BrandIcon { name: "zoom".to_string(), size: 14.0, coloured: true } span { class: "oxs12-btl", "Join" } } }
                              } }
                        } else { span { class: "oxs12-q", "Nothing else today" } }
                    }
                } else {
                    div { class: "oxs13-next",
                        div { class: "oxs12-cr1", span { class: "oxs12-hi", "data-hue": "violet", {nico("cal")} } span { class: "oxs12-ell", "{month_of(d).1}" } b { "{month_n}" } }
                        span { class: "oxs12-q", "{hours_words(busy_month)} busy in working hours" }
                    }
                }
            },
            actions: rsx! {
                {btn("New event", "plus", true, EventHandler::new(move |_| { let dd = *day.peek(); let a = free_gaps(&evs.peek(), dd, &hidden.peek()).first().map(|g| g.0).unwrap_or(WORK.0); draft.set(Some((dd, a, 30))); title.set(String::new()); said.set(format!("New event {} {}", iso(dd), hhmm(a))); }))}
            },
            bar: rsx! {
                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { day.set(TODAY); said.set(format!("Today, {}", iso(TODAY))); }, span { class: "oxs12-btl", "Today" } }
                button { r#type: "button", class: "oxs12-ib", "aria-label": "Previous", title: "Previous", onclick: move |_| go(-1), span { class: "oxs13-flip", {nico("next")} } }
                button { r#type: "button", class: "oxs12-ib", "aria-label": "Next", title: "Next", onclick: move |_| go(1), {nico("next")} }
                h1 { class: "oxs13-ttl oxs12-ell", "{title_lbl}" }
                if vw != 2 { span { class: "oxs12-q oxs13-wk", "{week_lbl}" } }
                span { class: "oxs12-grow" }
                {find_field("Filter events", "Filter events", q, None)}
                {seg("View", view_items, VIEWS[vw].0, EventHandler::new(move |k: &'static str| { if let Some(i) = VIEWS.iter().position(|v| v.0 == k) { set_view(i); } }))}
            },
            foot: rsx! {
                if let Some((dd, a, len)) = draft() {
                    div { class: "oxs13-new", role: "group", "aria-label": "New event",
                        span { class: "oxs12-hi", "data-hue": "{CALS[dcal()].2}", {nico("plus")} }
                        input { class: "oxs12-in oxs13-newt", r#type: "text", placeholder: "Add a title", "aria-label": "Event title", value: "{title}", autofocus: true,
                            oninput: move |e| title.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); if e.key() == Key::Enter { save(); } else if e.key() == Key::Escape { draft.set(None); } } }
                        span { class: "oxs12-q oxs12-ell", "{iso(dd)} {hhmm(a)}–{hhmm(a + len)}" }
                        span { class: "oxs13-ncal", MenuButton { label: CALS[dcal()].1.to_string(), aria: "Calendar", icon: "cal", caret: true, up: true, items: CALS.iter().map(|c| (c.0, c.1, "")).collect::<Vec<_>>(), checked: CALS[dcal()].0.to_string(),
                            on_pick: move |k: String| if let Some(i) = CALS.iter().position(|c| c.0 == k) { dcal.set(i); } } }
                        span { class: "oxs12-grow" }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| draft.set(None), span { class: "oxs12-btl", "Cancel" } }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| save(), span { class: "oxs12-btl", "Save" } }
                    }
                } else if let Some(e) = sel_ev.clone() {
                    { let id = e.id; let t = e.title.clone();
                      rsx! {
                        span { class: "oxs12-dot", "data-hue": "{CALS[e.cal].2}" }
                        b { class: "oxs12-ell oxs13-ft", "{e.title}" }
                        span { class: "oxs12-q oxs12-ell", "{iso(e.day)} · {e.span()}" if !e.place.is_empty() && e.place != "zoom" { " · {e.place}" } }
                        if !e.who.is_empty() { span { class: "oxs12-stack oxs13-st", for w in e.who.iter().take(4) { {av(*w, true)} } } }
                        span { class: "oxs12-grow" }
                        if e.place == "zoom" { button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| said.set("Joining the call".to_string()), BrandIcon { name: "zoom".to_string(), size: 14.0, coloured: true } span { class: "oxs12-btl", "Join" } } }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| { evs.write().retain(|x| x.id != id); sel.set(None); said.set(format!("{t} deleted")); }, {nico("trash")} span { class: "oxs12-btl", "Delete" } }
                        button { r#type: "button", class: "oxs12-ib", "aria-label": "Close details", onclick: move |_| sel.set(None), {nico("close")} }
                      } }
                } else {
                    span { class: "oxs12-ell oxs12-q", if vw == 2 { "Arrows move the day, Enter opens it. T today · D W M views · N new event" } else { "Drag an event to move it, its edge to resize, an empty slot to create. Focus an event: arrows move it, Shift + ↑ ↓ resize." } }
                }
            },
            right: rsx! { {day_panel(&all, d, &h, evs, said, draft, title)} },
            if loading {
                div { class: "oxs13-calsk", {skl(10, "")} }
            } else {
                div { class: "oxs13-cal", tabindex: "0", "aria-label": "Calendar. T today, D W M views, N new event", onkeydown: root_key,
                    if all.is_empty() {
                        div { class: "oxs13-calnone", "data-inline": "{vw == 2}", {none("cal", "No events yet", "Connect a calendar or add the first event; drag on the grid to block time.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| { draft.set(Some((TODAY, 10 * 60, 60))); title.set(String::new()); }, {nico("plus")} span { class: "oxs12-btl", "New event" } } })} }
                    }
                    if vw == 2 {
                        {month_view(&live, d, &h, day, sel, more, said, view)}
                        div { class: "oxs13-dayin", {day_panel(&all, d, &h, evs, said, draft, title)} }
                    } else {
                        div { class: "oxs13-wkv", "data-days": "{ndays}",
                            div { class: "oxs13-wkh",
                                span { class: "oxs13-gut oxs12-q", "{week_lbl}" }
                                for x in d0..d0 + ndays {
                                    { let (_, dm) = oxyn_core::parity::arcscr2::month_day(x); let wd = WEEKDAYS[weekday(x) as usize];
                                      rsx! { button { key: "{x}", r#type: "button", class: "oxs13-dh", "aria-current": if x == TODAY { "date" } else { "false" }, "aria-label": "{wd} {iso(x)}", title: "{iso(x)}",
                                        onclick: move |_| { day.set(x); set_view(0); },
                                        span { "{wd}" } b { "{dm}" } } } }
                                }
                            }
                            div { class: "oxs13-ad",
                                span { class: "oxs13-gut oxs12-q", "All day" }
                                for x in d0..d0 + ndays {
                                    div { key: "a{x}", class: "oxs13-adc",
                                        for e in live.iter().filter(|e| e.day == x && e.all_day && !h[e.cal]).cloned() {
                                            { let id = e.id; rsx! { button { key: "{id}", r#type: "button", class: "oxs13-adv", "data-hue": "{CALS[e.cal].2}", "aria-pressed": "{sel() == Some(id)}", title: "{e.title}", onclick: move |_| sel.set(Some(id)), span { class: "oxs12-ell", "{e.title}" } } } }
                                        }
                                    }
                                }
                            }
                            div { class: "oxs13-tg", style: "height:{(HOURS.1 - HOURS.0) as f64 * HOUR_H:.0}px",
                                onpointermove: move |e: PointerEvent| {
                                    let c = e.client_coordinates(); let s = stage_scale();
                                    if let Some(mut g) = drag() && g.fixed.is_none() { g.dx = (c.x - g.x0) / s; g.dy = (c.y - g.y0) / s; drag.set(Some(g)); }
                                    if let Some((dd, m0, y0, _)) = mk() { mk.set(Some((dd, m0, y0, (c.y - y0) / s))); }
                                },
                                onpointerup: move |_| end_drag(),
                                onpointerleave: move |_| { if drag.peek().is_some_and(|g| g.fixed.is_none()) { drag.set(None); } mk.set(None); },
                                div { class: "oxs13-gutc", "aria-hidden": "true",
                                    for hr in HOURS.0 + 1..HOURS.1 { span { key: "{hr}", class: "oxs13-hr", style: "top:{(hr - HOURS.0) as f64 * HOUR_H:.0}px", "{hr:02}:00" } }
                                }
                                div { class: "oxs13-cols", onresize: move |e: Event<ResizeData>| if let Ok(s) = e.get_border_box_size() { colw.set(s.width / ndays as f64); },
                                    for x in d0..d0 + ndays {
                                        { let de = day_events(&live, x, &h); let ln = lanes(&de); let today = x == TODAY; let we = weekday(x) >= 5;
                                          let mkd = mk().filter(|m| m.0 == x).map(|(_, m0, _, dy)| { let m1 = m0 + snap((dy / HOUR_H * 60.0) as i32); (m0.min(m1), (m0.max(m1) - m0.min(m1)).max(30)) });
                                          let dft = draft().filter(|m| m.0 == x);
                                          rsx! {
                                            div { key: "{x}", class: "oxs13-col", "data-today": "{today}", "data-we": "{we}", role: "group", "aria-label": "{iso(x)}",
                                                onpointerdown: move |e: PointerEvent| { let y = e.element_coordinates().y; let m = (HOURS.0 * 60 + snap((y / HOUR_H * 60.0) as i32 - 7)).clamp(0, 24 * 60 - 30); mk.set(Some((x, m, e.client_coordinates().y, 0.0))); },
                                                if today { span { class: "oxs13-now", style: "top:{ytop(NOW):.0}px", "aria-hidden": "true" } }
                                                if let Some((a, len)) = mkd { div { class: "oxs13-draft", style: "top:{ytop(a):.0}px;height:{len as f64 / 60.0 * HOUR_H:.0}px", "aria-hidden": "true", span { "{hhmm(a)}–{hhmm(a + len)}" } } }
                                                if let Some((_, a, len)) = dft { div { class: "oxs13-draft", "data-hue": "{CALS[dcal()].2}", style: "top:{ytop(a):.0}px;height:{len as f64 / 60.0 * HOUR_H:.0}px", span { class: "oxs12-ell", b { if title().trim().is_empty() { "New event" } else { "{title}" } } " {hhmm(a)}–{hhmm(a + len)}" } } }
                                                for e in de.iter().filter(|e| !e.all_day).cloned() {
                                                    {
                                                        let (lane, n) = ln.iter().find(|l| l.0 == e.id).map(|l| (l.1, l.2)).unwrap_or((0, 1));
                                                        let top = ytop(e.start);
                                                        let hgt = (e.len as f64 / 60.0 * HOUR_H).max(20.0);
                                                        let left = lane as f64 * 100.0 / n as f64;
                                                        let w = 100.0 / n as f64;
                                                        let id = e.id;
                                                        let hue = CALS[e.cal].2;
                                                        let is_live = dg.is_some_and(|g| g.id == id);
                                                        let short = hgt < 34.0;
                                                        let t = e.title.clone();
                                                        let lbl = format!("{}, {} {}{}", e.title, iso(e.day), e.span(), if e.place.is_empty() { String::new() } else if e.place == "zoom" { ", video call".to_string() } else { format!(", {}", e.place) });
                                                        let kd = move |k: KeyboardEvent| {
                                                            let sh = k.modifiers().shift();
                                                            let r = match k.key() {
                                                                Key::ArrowUp if sh => resize_event(&mut evs.write(), id, -15),
                                                                Key::ArrowDown if sh => resize_event(&mut evs.write(), id, 15),
                                                                Key::ArrowUp => nudge_event(&mut evs.write(), id, 0, -15),
                                                                Key::ArrowDown => nudge_event(&mut evs.write(), id, 0, 15),
                                                                Key::ArrowLeft => nudge_event(&mut evs.write(), id, -1, 0),
                                                                Key::ArrowRight => nudge_event(&mut evs.write(), id, 1, 0),
                                                                Key::Delete | Key::Backspace => { evs.write().retain(|x| x.id != id); sel.set(None); Some(format!("{t} deleted")) }
                                                                Key::Escape => { sel.set(None); None }
                                                                _ => return,
                                                            };
                                                            k.prevent_default(); k.stop_propagation();
                                                            if let Some(s) = r { said.set(s); }
                                                        };
                                                        rsx! {
                                                            div { key: "{id}", class: "oxs13-ev", "data-hue": "{hue}", "data-live": "{is_live}", "data-short": "{short}", "aria-pressed": "{sel() == Some(id)}",
                                                                style: "top:{top:.0}px;height:{hgt:.0}px;left:calc({left:.3}% + 1px);width:calc({w:.3}% - 3px)",
                                                                button { r#type: "button", class: "oxs13-evb", "aria-label": "{lbl}", "aria-roledescription": "Draggable event",
                                                                    onkeydown: kd, onclick: move |_| sel.set(Some(id)),
                                                                    onpointerdown: move |p: PointerEvent| { p.stop_propagation(); let c = p.client_coordinates(); sel.set(Some(id)); drag.set(Some(CDrag { id, mode: 0, x0: c.x, y0: c.y, dx: 0.0, dy: 0.0, fixed: None })); },
                                                                    if short {
                                                                        span { class: "oxs12-ell oxs13-evl", b { "{e.title}" } " {hhmm(e.start)}" }
                                                                    } else {
                                                                        b { class: "oxs12-ell oxs13-evt", "{e.title}" }
                                                                        span { class: "oxs12-ell oxs13-evs", if is_live { "{iso(e.day)} · " } "{e.span()}" }
                                                                        if hgt >= 64.0 && !e.place.is_empty() {
                                                                            span { class: "oxs12-ell oxs13-evs",
                                                                                if e.place == "zoom" { BrandIcon { name: "zoom".to_string(), size: 11.0, coloured: true } " Zoom" } else { "{e.place}" } }
                                                                        }
                                                                    }
                                                                }
                                                                span { class: "oxs13-rz", "aria-hidden": "true", onpointerdown: move |p: PointerEvent| { p.stop_propagation(); let c = p.client_coordinates(); drag.set(Some(CDrag { id, mode: 1, x0: c.x, y0: c.y, dx: 0.0, dy: 0.0, fixed: None })); } }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                          } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn month_view(
    live: &[CEvent],
    d: i32,
    h: &[bool; 6],
    mut day: Signal<i32>,
    mut sel: Signal<Option<u32>>,
    mut more: Signal<Option<i32>>,
    mut said: Signal<String>,
    mut view: Signal<usize>,
) -> Element {
    let cells = month_cells(d);
    let first = month_of(d).0;
    let grid_key = move |e: KeyboardEvent| {
        let cur = *day.peek();
        let nd = match e.key() {
            Key::ArrowLeft => cur - 1,
            Key::ArrowRight => cur + 1,
            Key::ArrowUp => cur - 7,
            Key::ArrowDown => cur + 7,
            Key::Enter => {
                e.prevent_default();
                view.set(0);
                said.set(format!("Day view, {}", period_title(0, cur)));
                return;
            }
            _ => return,
        };
        e.prevent_default();
        e.stop_propagation();
        day.set(nd.clamp(-30, 180));
        said.set(iso(nd));
    };
    rsx! {
        div { class: "oxs13-mo",
            div { class: "oxs13-moh", "aria-hidden": "true", for w in WEEKDAYS { span { key: "{w}", "{w}" } } }
            div { class: "oxs13-mog", role: "grid", tabindex: "0", "aria-label": "{month_of(d).1}, selected {iso(d)}. Arrows move the day, Enter opens it.", "aria-activedescendant": "oxs13-c{d}", onkeydown: grid_key,
                for x in cells.iter().copied() {
                    {
                        let out = month_of(x).0 != first;
                        let de = day_events(live, x, h);
                        let n = de.len();
                        let (_, dm) = oxyn_core::parity::arcscr2::month_day(x);
                        let open = more() == Some(x);
                        let show: Vec<CEvent> = de.iter().take(if n > 3 { 2 } else { 3 }).cloned().collect();
                        rsx! {
                            div { key: "{x}", id: "oxs13-c{x}", class: "oxs13-mc", role: "gridcell", "data-out": "{out}", "data-today": "{x == TODAY}", "aria-selected": "{x == d}",
                                button { r#type: "button", class: "oxs13-mcd", tabindex: "-1", "aria-label": "{iso(x)}, {n} events", onclick: move |_| { day.set(x); said.set(format!("{}, {n} events", iso(x))); },
                                    if dm == 1 { "{iso(x)[5..7].to_string()}-01" } else { "{dm}" } }
                                ul { class: "oxs13-chips",
                                    for e in show {
                                        { let id = e.id; let ed = e.day; rsx! { li { key: "{id}",
                                            button { r#type: "button", class: "oxs13-chip", tabindex: "-1", "data-hue": "{CALS[e.cal].2}", "data-allday": "{e.all_day}", title: "{e.title} · {e.span()}",
                                                onclick: move |_| { day.set(ed); sel.set(Some(id)); },
                                                if !e.all_day { span { class: "oxs13-ct", "{hhmm(e.start)}" } }
                                                span { class: "oxs12-ell", "{e.title}" } } } } }
                                    }
                                }
                                if n > 3 { button { r#type: "button", class: "oxs13-more", tabindex: "-1", onclick: move |_| { more.set(Some(x)); day.set(x); }, "+{n - 2} more" } }
                                span { class: "oxs13-dots", "aria-hidden": "true", for (j, e) in de.iter().take(4).enumerate() { span { key: "{j}", class: "oxs12-dot", "data-hue": "{CALS[e.cal].2}" } } }
                                if open {
                                    div { class: "oxs13-pop", role: "dialog", "aria-label": "Events on {iso(x)}",
                                        div { class: "oxs13-poph", b { "{iso(x)}" } span { class: "oxs12-grow" } button { r#type: "button", class: "oxs12-ib oxs13-ibs", "aria-label": "Close", onclick: move |e| { e.stop_propagation(); more.set(None); }, {nico("close")} } }
                                        for e in de.iter().cloned() {
                                            { let id = e.id; rsx! { button { key: "{id}", r#type: "button", class: "oxs13-chip", "data-hue": "{CALS[e.cal].2}", "data-allday": "{e.all_day}", onclick: move |_| { sel.set(Some(id)); more.set(None); },
                                                span { class: "oxs13-ct", if e.all_day { "All day" } else { "{hhmm(e.start)}" } } span { class: "oxs12-ell", "{e.title}" } } } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn day_panel(
    all: &[CEvent],
    d: i32,
    h: &[bool; 6],
    mut evs: Signal<Vec<CEvent>>,
    mut said: Signal<String>,
    mut draft: Signal<Option<(i32, i32, i32)>>,
    mut title: Signal<String>,
) -> Element {
    let de = day_events(all, d, h);
    let gaps = free_gaps(all, d, h);
    let busy = busy_minutes(all, d, h);
    let span = (WORK.1 - WORK.0) as f64;
    let hh = *h;
    let wd = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ][weekday(d) as usize];
    let can_focus = gaps.iter().any(|(a, b)| b - a >= 60);
    let first_gap = gaps.first().map(|g| g.0).unwrap_or(WORK.0);
    rsx! {
        div { class: "oxs12-rcin oxs13-dp",
            div { class: "oxs12-rch", b { "{wd} {iso(d)}" } span { class: "oxs12-q", "Week {iso_week(d)}" } }
            div { class: "oxs13-busy", role: "img", "aria-label": "{hours_words(busy)} busy between 09:00 and 18:00",
                for e in de.iter().filter(|e| !e.all_day && e.end() > WORK.0 && e.start < WORK.1) {
                    { let a = (e.start.max(WORK.0) - WORK.0) as f64 / span * 100.0; let w = (e.end().min(WORK.1) - e.start.max(WORK.0)) as f64 / span * 100.0;
                      rsx! { span { key: "{e.id}", "data-hue": "{CALS[e.cal].2}", style: "left:{a:.2}%;width:{w:.2}%" } } }
                }
            }
            div { class: "oxs12-cr2 oxs13-busyx", span { "09:00" } span { "13:30" } span { "18:00" } }
            p { class: "oxs12-q oxs13-busyt", "{hours_words(busy)} busy · {hours_words(WORK.1 - WORK.0 - busy)} free" }
            h3 { class: "oxs12-h2", "Schedule" }
            if de.is_empty() { p { class: "oxs12-faint", "Nothing planned" } }
            ul { class: "oxs13-sched",
                for e in de.iter() {
                    li { key: "{e.id}",
                        span { class: "oxs13-sct", if e.all_day { "All day" } else { "{hhmm(e.start)}" } }
                        span { class: "oxs12-dot", "data-hue": "{CALS[e.cal].2}" }
                        span { class: "oxs12-an", b { class: "oxs12-ell", "{e.title}" } if !e.place.is_empty() { span { class: "oxs12-ell", if e.place == "zoom" { "Zoom call" } else { "{e.place}" } } } }
                    }
                }
            }
            h3 { class: "oxs12-h2", "Free time" }
            if gaps.is_empty() { p { class: "oxs12-faint", "No free time in working hours" } }
            ul { class: "oxs13-sched",
                for (a, b) in gaps.iter().copied() {
                    li { key: "{a}", span { class: "oxs13-sct", "{hhmm(a)}" } span { class: "oxs12-ell oxs12-q", "until {hhmm(b)} · {hours_words(b - a)}" } }
                }
            }
            div { class: "oxs13-dpa",
                button { r#type: "button", class: "oxs12-btn oxs12-sm", disabled: !can_focus, onclick: move |_| { let r = block_focus(&mut evs.write(), d, &hh); said.set(r.unwrap_or_else(|| "No gap of an hour left".to_string())); }, {nico("target")} span { class: "oxs12-btl", "Block focus time" } }
                button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| { let a = first_gap; draft.set(Some((d, a, 30))); title.set(String::new()); said.set(format!("New event {} {}", iso(d), hhmm(a))); }, {nico("plus")} span { class: "oxs12-btl", "New event on {iso(d)}" } }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── inbox ──

/// Screen – Inbox triage: support conversations by folder (Inbox, Assigned to me, Unassigned,
/// Snoozed, Resolved) and tab (All, Unread, Due soon); J / K walk the list, E resolves, H snoozes,
/// X selects, U undoes, A assigns to you; bulk actions on a selection; the thread with its SLA
/// and a quick reply; the customer card. States: `sample`, `loading`, `empty`, `selected`,
/// `snoozing`, `archived`.
#[component]
pub fn InboxTriageScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc2_split(&state);
    let mut said = use_signal(String::new);
    let mut tri = use_signal(move || {
        let mut t = Triage::new(st != ScreenState::Empty);
        match own {
            "selected" => {
                t.sel = vec![102, 104, 108];
            }
            "snoozing" => t.snoozing = true,
            "archived" => {
                let _ = t.resolve();
            }
            _ => {}
        }
        t
    });
    let mut reply = use_signal(String::new);
    let mut sent = use_signal(Vec::<(u32, String)>::new);
    let loading = st == ScreenState::Loading;
    let t = tri();
    let list = t.list();
    let cur = t.current();
    let nsel = t.sel.len();
    let all_sel = !list.is_empty() && list.iter().all(|c| t.sel.contains(&c.id));
    let mut act = move |k: &str| {
        let r = tri.write().key(k);
        if let Some(s) = r {
            said.set(s);
        }
    };
    let keys = move |e: KeyboardEvent| {
        if e.modifiers().ctrl() || e.modifiers().meta() || e.modifiers().alt() {
            return;
        }
        match e.key() {
            Key::Character(c) if ["j", "k", "e", "h", "x", "u", "a"].contains(&c.as_str()) => {
                e.prevent_default();
                act(&c);
            }
            Key::ArrowDown => {
                e.prevent_default();
                act("j");
            }
            Key::ArrowUp => {
                e.prevent_default();
                act("k");
            }
            Key::Escape => {
                let mut w = tri.write();
                w.sel.clear();
                w.snoozing = false;
            }
            _ => {}
        }
    };
    let sort_items: Vec<(&'static str, &'static str, &'static str)> = vec![
        ("newest", "Newest first", ""),
        ("oldest", "Oldest first", ""),
    ];
    let tab_items: Vec<(&'static str, String)> = TABS
        .iter()
        .enumerate()
        .map(|(i, l)| (["all", "unread", "due"][i], l.to_string()))
        .collect();
    let undo_s = t.undo.last().map(|u| u.1.clone());
    let mut snooze = move |n: usize| {
        let r = tri.write().snooze(n);
        if let Some(s) = r {
            said.set(s);
        }
    };
    rsx! {
        ArcFrame { slug: "inbox-triage", layout, said,
            crumbs: vec!["Thistledown".to_string(), FOLDERS[t.folder].1.to_string()],
            meta: format!("{} open · {} unread", t.count(0), t.unread()),
            side: rsx! {
                for (i, (_, l, ic)) in FOLDERS.iter().copied().enumerate() {
                    { let n = t.count(i); rsx! { {nrow(nico(ic), l.to_string(), if i == 4 { String::new() } else { n.to_string() }, t.folder == i, EventHandler::new(move |_| { let s = tri.write().folder_to(i); said.set(s); }))} } }
                }
                {group("Team", true)}
                for (who, n) in IB_TEAM.iter().copied() {
                    div { key: "{who}", class: "oxs13-pr", {av(who, true)} span { class: "oxs12-ell oxs13-prn", "{PEOPLE[who].0}" } span { class: "oxs12-nrc", "{n}" } }
                }
                {group("Labels", false)}
                for (i, (l, hue)) in TAGS.iter().copied().enumerate() {
                    { let n = t.convs.iter().filter(|c| c.tag == i && c.place == oxyn_core::parity::arcscr2::Place::Open).count();
                      rsx! { div { key: "t{i}", class: "oxs13-pr", span { class: "oxs13-tagd", "data-hue": "{hue}" } span { class: "oxs12-ell oxs13-prn", "{l}" } span { class: "oxs12-nrc", "{n}" } } } }
                }
            },
            card: rsx! {
                div { class: "oxs13-avail",
                    label { class: "oxs12-sw",
                        input { r#type: "checkbox", role: "switch", checked: true, "aria-label": "Available for new conversations", onchange: move |e| said.set(if e.checked() { "Available".to_string() } else { "Away: new conversations skip you".to_string() }) }
                        span { class: "oxs12-swt", "aria-hidden": "true" }
                        span { class: "oxs12-an", b { "Available" } span { class: "oxs12-ell", "New conversations reach you" } }
                    }
                }
            },
            actions: rsx! {
                MenuButton { label: if t.newest { "Newest".to_string() } else { "Oldest".to_string() }, aria: "Sort", icon: "sort", caret: true, right: true, items: sort_items, checked: if t.newest { "newest".to_string() } else { "oldest".to_string() },
                    on_pick: move |k: String| { tri.write().newest = k == "newest"; said.set(if k == "newest" { "Newest first".to_string() } else { "Oldest first".to_string() }); } }
            },
            bar: rsx! {
                label { class: "oxs13-cbx oxs13-allx",
                    input { r#type: "checkbox", checked: all_sel, "aria-label": "Select all conversations", onchange: move |_| { let s = tri.write().select_all(); said.set(s); } }
                    span { class: "oxs13-box", "aria-hidden": "true", {nico("check")} }
                }
                if nsel > 0 {
                    span { class: "oxs13-seln", "{nsel} selected" }
                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| act("e"), {nico("check")} span { class: "oxs12-btl", "Resolve" } kbd { "E" } }
                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| snooze(0), {nico("clock")} span { class: "oxs12-btl", "Snooze" } kbd { "H" } }
                    button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| act("a"), {nico("user")} span { class: "oxs12-btl", "Assign to me" } }
                } else {
                    {seg("Show", tab_items, ["all", "unread", "due"][t.tab], EventHandler::new(move |k: &'static str| { let i = ["all", "unread", "due"].iter().position(|x| *x == k).unwrap_or(0); let s = tri.write().tab_to(i); said.set(s); }))}
                }
                span { class: "oxs12-grow" }
                if let Some(u) = undo_s.clone() {
                    span { class: "oxs13-undo", role: "status",
                        span { class: "oxs12-ell", "{u}" }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| act("u"), {nico("undo")} span { class: "oxs12-btl", "Undo" } kbd { "U" } }
                    }
                } else {
                    span { class: "oxs13-keys oxs12-ell", kbd { "J" } kbd { "K" } " move · " kbd { "E" } " resolve · " kbd { "H" } " snooze · " kbd { "X" } " select" }
                }
            },
            right: rsx! {
                if let Some(c) = cur.clone() { {customer_card(c.cust, said, Some(c.tag))} }
            },
            show_right: cur.is_some() && !loading,
            if loading {
                div { class: "oxs13-ib", div { class: "oxs13-ibl", {skl(14, "")} } div { class: "oxs13-ibt", {skl(9, "")} } }
            } else if t.convs.is_empty() || (list.is_empty() && t.folder == 0 && t.tab == 0) {
                {none("inbox", "Inbox zero", "New conversations from email, chat, Slack and X land here. Connect a channel to start.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| said.set("Connect a channel".to_string()), BrandIcon { name: "gmail".to_string(), size: 15.0, coloured: true } span { class: "oxs12-btl", "Connect Gmail" } } })}
            } else {
                div { class: "oxs13-ib", tabindex: "0", "aria-label": "Inbox. J and K move, E resolves, H snoozes, X selects, U undoes, A assigns to you.", onkeydown: keys,
                    ul { class: "oxs13-ibl", role: "listbox", "aria-label": "Conversations", "aria-multiselectable": "true",
                        if list.is_empty() { li { class: "oxs12-faint", "Nothing here" } }
                        for c in list.iter().cloned() {
                            { let id = c.id; let tm = if c.day == TODAY { hhmm(c.at) } else { iso(c.day) }; let on = cur.as_ref().is_some_and(|x| x.id == id); let chk = t.sel.contains(&id); let name = CUSTOMERS[c.cust].0;
                              rsx! {
                                li { key: "{id}", class: "oxs13-cv", role: "option", "aria-selected": "{on}", "data-unread": "{c.unread}", "data-checked": "{chk}",
                                    label { class: "oxs13-cbx",
                                        input { r#type: "checkbox", checked: chk, "aria-label": "Select {name}: {c.subject}", onchange: move |_| { let s = tri.write().toggle(Some(id)); if let Some(s) = s { said.set(s); } } }
                                        span { class: "oxs13-box", "aria-hidden": "true", {nico("check")} }
                                    }
                                    button { r#type: "button", class: "oxs13-cvb", tabindex: "-1", onclick: move |_| { tri.write().open_id(id); said.set(format!("{name}: {}", c.subject)); },
                                        span { class: "oxs13-cvh",
                                            span { class: "oxs13-cva", {cav(c.cust, true)} span { class: "oxs13-cvc", {channel(c.channel, 10.0)} } }
                                            b { class: "oxs12-ell oxs13-cvn", "{name}" }
                                            span { class: "oxs12-grow" }
                                            span { class: "oxs12-q", "{tm}" }
                                        }
                                        span { class: "oxs12-ell oxs13-cvs", if c.unread { span { class: "oxs13-ud", "aria-label": "Unread" } } "{c.subject}" }
                                        span { class: "oxs12-ell oxs13-cvp", "{c.preview}" }
                                        span { class: "oxs13-cvf",
                                            span { class: "oxs13-tag", span { class: "oxs13-tagd", "data-hue": "{TAGS[c.tag].1}" } "{TAGS[c.tag].0}" }
                                            if c.urgent { {pill("rose", "Urgent".to_string())} }
                                            span { class: "oxs12-grow" }
                                            if let Some(s) = c.sla { span { class: "oxs13-sla", "data-hue": "{sla_hue(s)}", {nico("clock")} "{sla_words(s)}" } }
                                            if let Some(w) = c.who { {av(w, true)} }
                                        }
                                    }
                                }
                              } }
                        }
                    }
                    div { class: "oxs13-ibt",
                        if let Some(c) = cur.clone() {
                            { let id = c.id; let name = CUSTOMERS[c.cust].0; let mine: Vec<String> = sent().iter().filter(|x| x.0 == id).map(|x| x.1.clone()).collect();
                              rsx! {
                                div { class: "oxs13-th",
                                    div { class: "oxs13-thh",
                                        h2 { class: "oxs12-ell", "{c.subject}" }
                                        div { class: "oxs13-thm",
                                            span { class: "oxs13-via", {channel(c.channel, 13.0)} "via {channel_name(c.channel)}" }
                                            if c.urgent { {pill("rose", "Urgent".to_string())} }
                                            if let Some(s) = c.sla { span { class: "oxs13-sla", "data-hue": "{sla_hue(s)}", {nico("clock")} if s < 0 { "Reply {sla_words(s)}" } else { "Reply in {sla_words(s)}" } } }
                                            span { class: "oxs12-grow" }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", disabled: c.who == Some(0), onclick: move |_| act("a"), {nico("user")} span { class: "oxs12-btl", if c.who == Some(0) { "Yours" } else { "Assign to me" } } }
                                            div { class: "oxs13-snz",
                                                button { r#type: "button", class: "oxs12-btn oxs12-sm", "aria-haspopup": "menu", "aria-expanded": "{t.snoozing}", onclick: move |_| { let mut w = tri.write(); w.snoozing = !w.snoozing; }, {nico("clock")} span { class: "oxs12-btl", "Snooze" } }
                                                if t.snoozing {
                                                    div { class: "oxs12-menu oxs12-menud oxs13-snzm", role: "menu", "aria-label": "Snooze until",
                                                        for (n, (l, u)) in SNOOZES.iter().copied().enumerate() {
                                                            button { key: "{n}", r#type: "button", class: "oxs12-mi", role: "menuitem", onclick: move |_| snooze(n), span { class: "oxs12-ell", "{l}" } kbd { "{u}" } }
                                                        }
                                                    }
                                                }
                                            }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| act("e"), {nico("check")} span { class: "oxs12-btl", "Resolve" } }
                                        }
                                    }
                                    ol { class: "oxs13-msgs",
                                        for (j, (cust, dd, m, text)) in thread(&c).into_iter().enumerate() {
                                            li { key: "{j}", class: "oxs13-msg", "data-me": "{!cust}",
                                                if cust { {cav(c.cust, false)} } else { {av(0, false)} }
                                                div { class: "oxs13-mb",
                                                    div { class: "oxs13-mbh", b { if cust { "{name}" } else { "{PEOPLE[0].0}" } } span { class: "oxs12-q", "{when(dd, m)}" } }
                                                    p { "{text}" }
                                                }
                                            }
                                        }
                                        for (j, text) in mine.into_iter().enumerate() {
                                            li { key: "r{j}", class: "oxs13-msg", "data-me": "true",
                                                {av(0, false)}
                                                div { class: "oxs13-mb", div { class: "oxs13-mbh", b { "{PEOPLE[0].0}" } span { class: "oxs12-q", "{hhmm(NOW)}" } } p { "{text}" } }
                                            }
                                        }
                                    }
                                    div { class: "oxs13-qr",
                                        textarea { class: "oxs13-ta", rows: "2", placeholder: "Reply to {first_name(c.cust)}…", "aria-label": "Reply to {name}", value: "{reply}",
                                            oninput: move |e| reply.set(e.value()),
                                            onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); if e.key() == Key::Enter && (e.modifiers().ctrl() || e.modifiers().meta()) { let r = reply.peek().trim().to_string(); if !r.is_empty() { sent.write().push((id, r)); reply.set(String::new()); said.set(format!("Reply sent to {name}")); } } } }
                                        div { class: "oxs13-qra",
                                            span { class: "oxs12-q oxs12-ell", "Ctrl + Enter sends" }
                                            span { class: "oxs12-grow" }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: reply().trim().is_empty(),
                                                onclick: move |_| { let r = reply.peek().trim().to_string(); if !r.is_empty() { sent.write().push((id, r)); reply.set(String::new()); said.set(format!("Reply sent to {name}")); } },
                                                {nico("send")} span { class: "oxs12-btl", "Send" } }
                                        }
                                    }
                                }
                              } }
                        } else {
                            {none("check", "All caught up", "Pick a conversation, or press J to open the next one.", rsx! {})}
                        }
                    }
                }
            }
        }
    }
}

fn customer_card(c: usize, mut said: Signal<String>, tag: Option<usize>) -> Element {
    let (name, org, email, plan, city, _, since) = CUSTOMERS[c % CUSTOMERS.len()];
    let prev = previous(c);
    rsx! {
        div { class: "oxs12-rcin oxs13-cc",
            div { class: "oxs13-cch", {cav(c, false)} span { class: "oxs12-an", b { class: "oxs12-ell", "{name}" } span { class: "oxs12-ell", "{org}" } } }
            dl { class: "oxs13-dl",
                dt { "Email" } dd { class: "oxs12-ell", "{email}" }
                dt { "Local time" } dd { class: "oxs12-ell", "{cust_time(c)} in {city}" }
                dt { "Plan" } dd { {pill(if plan == "Enterprise" { "violet" } else if plan == "Scale" { "blue" } else { "mute" }, plan.to_string())} }
                dt { "Customer since" } dd { "{since}" }
                if let Some(t) = tag { dt { "Label" } dd { span { class: "oxs13-tag", span { class: "oxs13-tagd", "data-hue": "{TAGS[t].1}" } "{TAGS[t].0}" } } }
            }
            h3 { class: "oxs12-h2", "Previous conversations" }
            ul { class: "oxs13-prev",
                for (i, (s, d, stt)) in prev.into_iter().enumerate() {
                    li { key: "{i}", button { r#type: "button", class: "oxs13-prb", onclick: move |_| said.set(format!("{s} ({d})")), span { class: "oxs12-ell", "{s}" } span { class: "oxs12-q", "{d} · {stt}" } } }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── support ──

/// Screen – Support conversation: tickets by view with SLAs, the open ticket's thread (customer
/// messages, replies, internal notes in amber, article cards, attachments and system lines), a
/// composer with Reply / Note, saved replies, article, attach and Send (Ctrl + Enter), Snooze and
/// Close; the customer card with apps (Google Calendar, Stripe), the linked issue and earlier
/// conversations. States: `sample`, `loading`, `empty`, `note`, `sent`, `closed`.
#[component]
pub fn SupportConversationScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc2_split(&state);
    let mut said = use_signal(String::new);
    let mut desk = use_signal(move || {
        let mut d = Desk::new(st != ScreenState::Empty);
        match own {
            "note" => {
                d.note = true;
                d.draft = "Told Jonah the fix date; let's check their Monday export on 2026-11-16."
                    .to_string();
            }
            "sent" => {
                d.saved(0);
                let _ = d.send();
            }
            "closed" => {
                let _ = d.close();
            }
            _ => {}
        }
        d
    });
    let mut snz = use_signal(|| false);
    let loading = st == ScreenState::Loading;
    let dk = desk();
    let list = dk.list();
    let cur = dk.current().cloned();
    let mut send = move || {
        let r = desk.write().send();
        said.set(match r {
            Ok(s) => s,
            Err(e) => e.to_string(),
        });
    };
    let saved_items: Vec<(&'static str, &'static str, &'static str)> =
        SAVED.iter().map(|s| (s.0, s.0, "")).collect();
    let wait = waiting(&dk);
    rsx! {
        ArcFrame { slug: "support-conversation", layout, said,
            crumbs: vec!["Pinecrest".to_string(), SP_VIEWS[dk.view].1.to_string(), cur.as_ref().map(|t| format!("#{}", t.id)).unwrap_or_default()],
            meta: cur.as_ref().map(|t| CUSTOMERS[t.cust].0.to_string()).unwrap_or_default(),
            side: rsx! {
                for (i, (_, l, ic)) in SP_VIEWS.iter().copied().enumerate() {
                    { let n = if i == 5 { String::new() } else { dk.count(i).to_string() }; rsx! { {nrow(nico(ic), l.to_string(), n, dk.view == i, EventHandler::new(move |_| { let s = desk.write().view_to(i); said.set(s); }))} } }
                }
                {group("Waiting on you", false)}
                for (c, s) in wait.iter().copied() {
                    div { key: "{c}", class: "oxs13-pr", {cav(c, true)} span { class: "oxs12-ell oxs13-prn", "{CUSTOMERS[c].0}" } span { class: "oxs13-sla", "data-hue": "{sla_hue(s)}", if s < 0 { "{sla_words(s)}" } else { "{sla_words(s)} left" } } }
                }
                {group("Teams", true)}
                for (n, hue, c) in SP_TEAMS.iter().copied() {
                    div { key: "{n}", class: "oxs13-pr", span { class: "oxs13-tagd", "data-hue": "{hue}" } span { class: "oxs12-ell oxs13-prn", "{n}" } span { class: "oxs12-nrc", "{c}" } }
                }
            },
            card: rsx! {
                div { class: "oxs13-avail",
                    label { class: "oxs12-sw",
                        input { r#type: "checkbox", role: "switch", checked: true, "aria-label": "Available", onchange: move |e| said.set(if e.checked() { "Available".to_string() } else { "Away".to_string() }) }
                        span { class: "oxs12-swt", "aria-hidden": "true" }
                        span { class: "oxs12-an", b { "Available" } span { class: "oxs12-ell", "{dk.count(0)} tickets in your inbox" } }
                    }
                }
            },
            actions: rsx! {
                if let Some(t) = cur.clone() {
                    if t.who != Some(0) { button { r#type: "button", class: "oxs12-btn", onclick: move |_| { let r = desk.write().take(); if let Some(s) = r { said.set(s); } }, {nico("user")} span { class: "oxs12-btl", "Assign to me" } } }
                    div { class: "oxs13-snz",
                        button { r#type: "button", class: "oxs12-btn", "aria-haspopup": "menu", "aria-expanded": "{snz()}", onclick: move |_| snz.toggle(), {nico("clock")} span { class: "oxs12-btl", "Snooze" } }
                        if snz() {
                            div { class: "oxs12-menu oxs12-menud oxs13-snzm oxs13-snzr", role: "menu", "aria-label": "Snooze until",
                                for (n, (l, u)) in SNOOZES.iter().copied().enumerate() {
                                    button { key: "{n}", r#type: "button", class: "oxs12-mi", role: "menuitem", onclick: move |_| { snz.set(false); let r = desk.write().snooze(n); if let Some(s) = r { said.set(s); } }, span { class: "oxs12-ell", "{l}" } kbd { "{u}" } }
                                }
                            }
                        }
                    }
                    button { r#type: "button", class: "oxs12-btn", "data-pri": "{t.status != TStatus::Closed}", onclick: move |_| { let r = desk.write().close(); if let Some(s) = r { said.set(s); } },
                        {nico(if t.status == TStatus::Closed { "undo" } else { "check" })} span { class: "oxs12-btl", if t.status == TStatus::Closed { "Reopen" } else { "Close" } } }
                }
            },
            right: rsx! { if let Some(t) = cur.clone() { {ticket_card(&t, dk.card, desk, said)} } },
            show_right: cur.is_some() && !loading,
            if loading {
                div { class: "oxs13-sp", div { class: "oxs13-spl", {skl(12, "")} } div { class: "oxs13-spt", {skl(10, "")} } }
            } else if dk.tickets.is_empty() {
                {none("inbox", "No tickets yet", "Tickets from email, chat and Slack show here with their SLA. Share your support address to get the first one.", rsx! { button { r#type: "button", class: "oxs12-btn", onclick: move |_| said.set("support@pinecrest.example copied".to_string()), {nico("copy")} span { class: "oxs12-btl", "Copy support address" } } })}
            } else {
                div { class: "oxs13-sp",
                    ul { class: "oxs13-spl", "aria-label": "Tickets",
                        if list.is_empty() { li { class: "oxs12-faint", "Nothing in {SP_VIEWS[dk.view].1}" } }
                        for tk in list.iter().cloned() {
                            { let id = tk.id; let on = dk.open == Some(id); let last = tk.msgs.last().map(|m| m.text.clone()).unwrap_or_default();
                              rsx! { li { key: "{id}",
                                button { r#type: "button", class: "oxs13-tk", "aria-current": "{on}", "data-unread": "{tk.unread}", onclick: move |_| { desk.write().open_id(id); said.set(format!("#{id} {}", tk.subject)); },
                                    span { class: "oxs13-cvh", {cav(tk.cust, true)} b { class: "oxs12-ell oxs13-cvn", "{CUSTOMERS[tk.cust].0}" } span { class: "oxs12-grow" } span { class: "oxs12-q", "#{id}" } }
                                    span { class: "oxs12-ell oxs13-cvs", if tk.unread { span { class: "oxs13-ud", "aria-label": "Unread" } } "{tk.subject}" }
                                    span { class: "oxs12-ell oxs13-cvp", "{last}" }
                                    span { class: "oxs13-cvf",
                                        span { class: "oxs13-st2", "data-hue": "{tk.status.hue()}", span { class: "oxs12-dot" } "{tk.status.label()}" }
                                        span { class: "oxs12-grow" }
                                        if let Some(s) = tk.sla { span { class: "oxs13-sla", "data-hue": "{sla_hue(s)}", {nico("clock")} "{sla_words(s)}" } }
                                    }
                                }
                              } } }
                        }
                    }
                    div { class: "oxs13-spt",
                        if let Some(t) = cur.clone() {
                            { let name = CUSTOMERS[t.cust].0; let draft_empty = dk.draft.trim().is_empty() && dk.files.is_empty() && dk.article.is_none();
                              rsx! {
                                div { class: "oxs13-thh",
                                    h2 { class: "oxs12-ell", "{t.subject}" }
                                    div { class: "oxs13-thm",
                                        span { class: "oxs13-via", {channel(t.channel, 13.0)} "via {channel_name(t.channel)}" }
                                        span { class: "oxs13-st2", "data-hue": "{t.status.hue()}", span { class: "oxs12-dot" } "{t.status.label()}" }
                                        if t.urgent { {pill("rose", "Urgent".to_string())} }
                                        if let Some(s) = t.sla { span { class: "oxs13-sla", "data-hue": "{sla_hue(s)}", {nico("clock")} if s < 0 { "First reply {sla_words(s)}" } else { "Reply in {sla_words(s)}" } } }
                                        span { class: "oxs12-grow" }
                                        span { class: "oxs12-q oxs12-ell", "Assignee: " if t.who == Some(0) { "You" } else if let Some(w) = t.who { "{PEOPLE[w].0}" } else { "Unassigned" } }
                                    }
                                }
                                ol { class: "oxs13-msgs", "aria-label": "Conversation",
                                    for (j, m) in t.msgs.iter().cloned().enumerate() {
                                        { match m.kind {
                                            Kind::Event => rsx! { li { key: "{j}", class: "oxs13-evl2", span { class: "oxs12-q oxs12-ell", "{m.text} · {m.at}" } } },
                                            k => {
                                                let (who_l, me, note) = match k { Kind::Customer => (name.to_string(), false, false), Kind::Reply(w) => (PEOPLE[w].0.to_string(), true, false), Kind::Note(w) => (PEOPLE[w].0.to_string(), true, true), Kind::Event => (String::new(), false, false) };
                                                let who_i = match k { Kind::Reply(w) | Kind::Note(w) => w, _ => 0 };
                                                rsx! {
                                                    li { key: "{j}", class: "oxs13-msg", "data-me": "{me}", "data-note": "{note}",
                                                        if me { {av(who_i, false)} } else { {cav(t.cust, false)} }
                                                        div { class: "oxs13-mb",
                                                            div { class: "oxs13-mbh", b { "{who_l}" } if note { span { class: "oxs13-nl", "data-hue": "amber", {nico("lock")} "Internal note" } } span { class: "oxs12-q", "{m.at}" } }
                                                            p { "{m.text}" }
                                                            if let Some(a) = m.article { {article_card(a)} }
                                                            if !m.files.is_empty() { div { class: "oxs13-files", for f in m.files.iter() { span { key: "{f}", class: "oxs13-file", {nico(if f.ends_with(".png") { "image" } else { "file" })} span { class: "oxs12-ell", "{f}" } } } } }
                                                        }
                                                    }
                                                }
                                            }
                                        } }
                                    }
                                }
                                div { class: "oxs13-cmp", "data-note": "{dk.note}",
                                    div { class: "oxs13-cmh",
                                        div { class: "oxs12-seg", role: "radiogroup", "aria-label": "Composer mode",
                                            button { r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{!dk.note}", onclick: move |_| desk.write().note = false, "Reply" }
                                            button { r#type: "button", role: "radio", class: "oxs12-sego", "aria-checked": "{dk.note}", onclick: move |_| desk.write().note = true, "Note" }
                                        }
                                        span { class: "oxs12-q oxs12-ell", if dk.note { "Only your team sees notes" } else { "To {name} via {channel_name(t.channel)}" } }
                                    }
                                    textarea { class: "oxs13-ta", rows: "3", placeholder: if dk.note { "Write a note; @ mentions a teammate" } else { "Write a reply…" }, "aria-label": if dk.note { "Internal note" } else { "Reply" }, value: "{dk.draft}",
                                        oninput: move |e| desk.write().draft = e.value(),
                                        onkeydown: move |e: KeyboardEvent| { e.stop_propagation(); if e.key() == Key::Enter && (e.modifiers().ctrl() || e.modifiers().meta()) { e.prevent_default(); send(); } } }
                                    if !dk.files.is_empty() || dk.article.is_some() {
                                        div { class: "oxs13-files",
                                            for (fi, f) in dk.files.iter().cloned().enumerate() { span { key: "{fi}", class: "oxs13-file", {nico("file")} span { class: "oxs12-ell", "{f}" } button { r#type: "button", class: "oxs12-chx", "aria-label": "Remove {f}", onclick: move |_| { desk.write().files.remove(fi); }, {nico("close")} } } }
                                            if dk.article.is_some() { span { class: "oxs13-file", {nico("doc")} span { class: "oxs12-ell", "Exports time out" } button { r#type: "button", class: "oxs12-chx", "aria-label": "Remove the article", onclick: move |_| desk.write().article = None, {nico("close")} } } }
                                        }
                                    }
                                    div { class: "oxs13-cmf",
                                        MenuButton { label: "Saved replies".to_string(), aria: "Saved replies", icon: "note", up: true, items: saved_items, on_pick: move |k: String| { if let Some(i) = SAVED.iter().position(|s| s.0 == k) { let r = desk.write().saved(i); if let Some(s) = r { said.set(s); } } } }
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| { desk.write().article = Some(12); said.set("Article attached: Exports time out".to_string()); }, {nico("doc")} span { class: "oxs12-btl", "Article" } }
                                        button { r#type: "button", class: "oxs12-ib", "aria-label": "Attach a file", title: "Attach", onclick: move |_| { let s = desk.write().attach(); said.set(s); }, {nico("clip")} }
                                        span { class: "oxs12-grow" }
                                        kbd { class: "oxs13-kh", "Ctrl ↵" }
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: draft_empty, onclick: move |_| send(),
                                            {nico("send")} span { class: "oxs12-btl", if dk.note { "Add note" } else { "Send" } } }
                                    }
                                }
                              } }
                        } else {
                            {none("check", "No ticket open", "Pick a ticket on the left.", rsx! {})}
                        }
                    }
                }
            }
        }
    }
}

fn article_card(id: u32) -> Element {
    let arts = oxyn_core::parity::arcscr2::articles(true);
    let Some(a) = arts.iter().find(|a| a.id == id) else {
        return rsx! {};
    };
    let (cn, hue, _) = COLLECTIONS[a.coll];
    let lead = a.body.first().map(|b| b.1).unwrap_or("");
    rsx! {
        div { class: "oxs13-art",
            span { class: "oxs12-hi", "data-hue": "{hue}", {nico("doc")} }
            span { class: "oxs12-an", b { class: "oxs12-ell", "{a.title}" } span { class: "oxs12-ell", "{cn} · {lead}" } }
        }
    }
}

fn ticket_card(
    t: &oxyn_core::parity::arcscr2::Ticket,
    tab: usize,
    mut desk: Signal<Desk>,
    mut said: Signal<String>,
) -> Element {
    let c = t.cust;
    let (name, org, email, plan, city, _, since) = CUSTOMERS[c % CUSTOMERS.len()];
    let prev = previous(c);
    let issue = t.issue;
    let first = first_name(c);
    rsx! {
        div { class: "oxs12-rcin oxs13-cc",
            div { class: "oxs13-cch", {cav(c, false)} span { class: "oxs12-an", b { class: "oxs12-ell", "{name}" } span { class: "oxs12-ell", "{org}" } } }
            div { class: "oxs12-seg oxs13-cct", role: "tablist", "aria-label": "Customer",
                button { r#type: "button", role: "tab", class: "oxs12-sego", "aria-selected": "{tab == 0}", onclick: move |_| desk.write().card = 0, "Profile" }
                button { r#type: "button", role: "tab", class: "oxs12-sego", "aria-selected": "{tab == 1}", onclick: move |_| desk.write().card = 1, "Activity" }
            }
            if tab == 0 {
                dl { class: "oxs13-dl",
                    dt { "Email" } dd { class: "oxs12-ell", "{email}" }
                    dt { "Local time" } dd { class: "oxs12-ell", "{cust_time(c)} in {city}" }
                    dt { "Plan" } dd { {pill(if plan == "Enterprise" { "violet" } else if plan == "Scale" { "blue" } else { "mute" }, plan.to_string())} }
                    dt { "Customer since" } dd { "{since}" }
                }
                h3 { class: "oxs12-h2", "Apps" }
                ul { class: "oxs13-apps",
                    li { BrandIcon { name: "googlecalendar".to_string(), size: 16.0, coloured: true } span { class: "oxs12-an", b { class: "oxs12-ell", "Google Calendar" } span { class: "oxs12-ell", "Onboarding call 2026-11-13 10:00" } } }
                    li { BrandIcon { name: "stripe".to_string(), size: 16.0, coloured: true } span { class: "oxs12-an", b { class: "oxs12-ell", "Stripe" } span { class: "oxs12-ell", "{plan} · last invoice paid 2026-11-01" } } }
                }
                if !issue.is_empty() {
                    h3 { class: "oxs12-h2", "Linked issue" }
                    button { r#type: "button", class: "oxs13-iss", onclick: move |_| said.set(format!("Opened {issue}")), BrandIcon { name: "linear".to_string(), size: 15.0, coloured: true } span { class: "oxs12-ell", "{issue}" } }
                }
                h3 { class: "oxs12-h2", "Previous conversations" }
                ul { class: "oxs13-prev",
                    for (i, (s, d, stt)) in prev.into_iter().enumerate() {
                        li { key: "{i}", button { r#type: "button", class: "oxs13-prb", onclick: move |_| said.set(format!("{s} ({d})")), span { class: "oxs12-ell", "{s}" } span { class: "oxs12-q", "{d} · {stt}" } } }
                    }
                }
            } else {
                ul { class: "oxs13-act",
                    li { span { class: "oxs12-dot", "data-hue": "green" } span { class: "oxs12-ell", "{first} exported orders" } span { class: "oxs12-q", "{iso(TODAY)} 13:20" } }
                    li { span { class: "oxs12-dot", "data-hue": "blue" } span { class: "oxs12-ell", "Viewed Exports time out" } span { class: "oxs12-q", "{iso(TODAY)} 13:44" } }
                    li { span { class: "oxs12-dot", "data-hue": "amber" } span { class: "oxs12-ell", "Invoice paid" } span { class: "oxs12-q", "2026-11-01" } }
                    li { span { class: "oxs12-dot", "data-hue": "violet" } span { class: "oxs12-ell", "Added 3 seats" } span { class: "oxs12-q", "2026-10-22" } }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── help center ──

/// Screen – Help center: articles by status (All, Published, Drafts, In review) and collection;
/// search over titles and bodies with highlighted snippets (`/` focuses, Escape clears), the
/// reader with related articles, Was this helpful? (a No asks for a reason and sends it to the
/// article's owner), Publish for drafts, and a panel of views, votes and top searches. States:
/// `sample`, `loading`, `empty`, `search`, `noresults`, `feedback`.
#[component]
pub fn HelpCenterScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
) -> Element {
    let (st, own) = arc2_split(&state);
    let mut said = use_signal(String::new);
    let mut help = use_signal(move || {
        let mut h = Help::new(st != ScreenState::Empty);
        match own {
            "feedback" => {
                h.open_id(12);
                h.vote(false);
                h.reason = Some(2);
            }
            "search" => {
                h.open_id(5);
            }
            _ => {}
        }
        h
    });
    let mut q = use_signal(move || match own {
        "search" => "cancel".to_string(),
        "noresults" => "dark mode".to_string(),
        _ => String::new(),
    });
    let mut comment = use_signal(move || {
        if own == "feedback" {
            "Step 2 needs the Schedule button; I couldn't find it.".to_string()
        } else {
            String::new()
        }
    });
    let search_el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let loading = st == ScreenState::Loading;
    let mut hv = help();
    hv.query = q();
    let shown = hv.shown();
    let n = tab_counts(&hv.arts, hv.coll);
    let cur = hv.current().cloned();
    let (check, pct) = quarter(&hv.arts);
    let focus_search = move |_: ()| {
        if let Some(m) = search_el() {
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let keys = move |e: KeyboardEvent| {
        if let Key::Character(c) = e.key()
            && c == "/"
        {
            e.prevent_default();
            focus_search(());
        }
    };
    let tab_items: Vec<(&'static str, String)> = ART_TABS
        .iter()
        .enumerate()
        .map(|(i, t)| (t.0, format!("{} {}", t.1, n[i])))
        .collect();
    rsx! {
        ArcFrame { slug: "help-center", layout, said, on_search: EventHandler::new(focus_search),
            crumbs: vec!["Quillmoor Help".to_string(), "Articles".to_string()],
            meta: format!("{} articles", hv.arts.len()),
            side: rsx! {
                for (i, (_, ic, l, c)) in HC_NAV.iter().copied().enumerate() {
                    { let c = if i == 0 { hv.arts.len().to_string() } else { c.to_string() }; rsx! { {nrow(nico(ic), l.to_string(), c, i == 0, EventHandler::new(move |_| said.set(format!("{l} opened"))))} } }
                }
                {group("Collections", true)}
                for (i, (name, hue, ic)) in COLLECTIONS.iter().copied().enumerate() {
                    { let on = hv.coll == Some(i); let cnt = hv.arts.iter().filter(|a| a.coll == i).count();
                      rsx! { button { key: "{i}", r#type: "button", class: "oxs12-nr", "aria-pressed": "{on}", onclick: move |_| { let mut w = help.write(); w.coll = if on { None } else { Some(i) }; said.set(if on { "All collections".to_string() } else { name.to_string() }); },
                        span { class: "oxs12-nri", span { class: "oxs12-hi", "data-hue": "{hue}", {nico(ic)} } } span { class: "oxs12-ell oxs12-nrl", "{name}" } span { class: "oxs12-nrc", "{cnt}" } } } }
                }
            },
            card: rsx! {
                div { class: "oxs13-qrv",
                    div { class: "oxs12-cr1", span { class: "oxs12-ell", "Quarterly review" } b { "{pct}%" } }
                    span { class: "oxs12-prog", span { style: "width:{pct}%" } }
                    div { class: "oxs12-cr1", span { class: "oxs12-q oxs12-ell", "{check} articles to check" }
                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { help.write().tab = 3; said.set("In review".to_string()); }, span { class: "oxs12-btl", "Review" } } }
                }
            },
            actions: rsx! {
                {btn("New article", "plus", true, EventHandler::new(move |_| said.set("New draft in Getting started".to_string())))}
            },
            bar: rsx! {
                {seg("Status", tab_items, ART_TABS[hv.tab].0, EventHandler::new(move |k: &'static str| { if let Some(i) = ART_TABS.iter().position(|t| t.0 == k) { help.write().tab = i; said.set(ART_TABS[i].1.to_string()); } }))}
                span { class: "oxs12-grow" }
                {find_field("Search articles", "Search articles", q, Some(search_el))}
            },
            if loading {
                div { class: "oxs13-hc", div { class: "oxs13-hcl", {skl(12, "")} } div { class: "oxs13-hcr", {skl(14, "")} } }
            } else if hv.arts.is_empty() {
                {none("doc", "No articles yet", "Write the first article or import them from your old help site.", rsx! { button { r#type: "button", class: "oxs12-btn", "data-pri": "true", onclick: move |_| said.set("New draft".to_string()), {nico("plus")} span { class: "oxs12-btl", "Write an article" } } })}
            } else {
                div { class: "oxs13-hc", tabindex: "-1", onkeydown: keys,
                    div { class: "oxs13-hcl",
                        if !q().trim().is_empty() {
                            p { class: "oxs13-hn", role: "status", if shown.is_empty() { "No articles match “{q}”" } else { "{shown.len()} results for “{q}”" } }
                        }
                        if shown.is_empty() && !q().trim().is_empty() {
                            div { class: "oxs13-nores",
                                p { class: "oxs12-q", "Readers searched this too. Write the article, or try:" }
                                div { class: "oxs13-sugg",
                                    for (i, (s, _)) in SEARCHES.iter().copied().enumerate().filter(|(_, (s, _))| !oxyn_core::parity::arcscr2::search(&hv.arts, s).is_empty()).take(3) {
                                        button { key: "{i}", r#type: "button", class: "oxs12-chip oxs13-sg", onclick: move |_| q.set(s.to_string()), "{s}" }
                                    }
                                }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let t = q.peek().clone(); said.set(format!("New draft: {t}")); }, {nico("plus")} span { class: "oxs12-btl oxs12-ell", "Write “{q}”" } }
                                button { r#type: "button", class: "oxs12-btn oxs12-sm oxs12-ghost", onclick: move |_| q.set(String::new()), span { class: "oxs12-btl", "Clear search" } }
                            }
                        }
                        ul { class: "oxs13-arts", "aria-label": "Articles",
                            for (a, hit) in shown.iter().cloned() {
                                { let id = a.id; let on = hv.open == Some(id); let (cn, hue, _) = COLLECTIONS[a.coll];
                                  let stl = match a.status { 2 => ("Draft", "mute"), 3 => ("In review", "amber"), _ => ("Published", "green") };
                                  rsx! { li { key: "{id}",
                                    button { r#type: "button", class: "oxs13-ar", "aria-current": "{on}", onclick: move |_| { let r = help.write().open_id(id); if let Some(s) = r { said.set(s); } },
                                        span { class: "oxs13-arh", span { class: "oxs13-tagd", "data-hue": "{hue}" } b { class: "oxs12-ell", "{a.title}" } }
                                        if let Some(h) = hit.clone() {
                                            { let (pre, mid, post) = match h.mark { Some((x, y)) => (h.snippet[..x].to_string(), h.snippet[x..y].to_string(), h.snippet[y..].to_string()), None => (h.snippet.clone(), String::new(), String::new()) };
                                              rsx! { span { class: "oxs13-snip", "{pre}" mark { "{mid}" } "{post}" } } }
                                        }
                                        span { class: "oxs13-arf", span { class: "oxs13-st2", "data-hue": "{stl.1}", span { class: "oxs12-dot" } "{stl.0}" } span { class: "oxs12-ell oxs12-q", "{cn}" } span { class: "oxs12-grow" } span { class: "oxs12-q", "{iso(a.updated)}" } }
                                    }
                                  } } }
                            }
                        }
                    }
                    div { class: "oxs13-hcr",
                        if let Some(a) = cur.clone() {
                            { let (cn, hue, _) = COLLECTIONS[a.coll]; let pub_ok = a.status != 1; let up = helpful(&a);
                              rsx! {
                                article { class: "oxs13-read",
                                    div { class: "oxs13-rdh",
                                        span { class: "oxs13-tag", span { class: "oxs13-tagd", "data-hue": "{hue}" } "{cn}" }
                                        span { class: "oxs12-grow" }
                                        if pub_ok { button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", onclick: move |_| { let r = help.write().publish(); if let Some(s) = r { said.set(s); } }, {nico("upload")} span { class: "oxs12-btl", "Publish" } } }
                                        button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| said.set(format!("Link to {} copied", a.title)), {nico("link")} span { class: "oxs12-btl", "Copy link" } }
                                    }
                                    h1 { "{a.title}" }
                                    p { class: "oxs13-rdm", {av(a.owner, true)} span { class: "oxs12-ell", "{PEOPLE[a.owner].0} · updated {iso(a.updated)} · {thousands(a.views as u64)} views" } }
                                    for (j, (hd, tx)) in a.body.iter().copied().enumerate() {
                                        section { key: "{j}", h2 { "{hd}" } p { "{tx}" } }
                                    }
                                    if !a.related.is_empty() {
                                        h3 { class: "oxs12-h2", "Related" }
                                        ul { class: "oxs13-rel",
                                            for r in a.related.iter().copied() {
                                                { let t = hv.arts.iter().find(|x| x.id == r).map(|x| x.title).unwrap_or("");
                                                  rsx! { li { key: "{r}", button { r#type: "button", class: "oxs13-rlb", onclick: move |_| { let s = help.write().open_id(r); if let Some(s) = s { said.set(s); } }, {nico("doc")} span { class: "oxs12-ell", "{t}" } } } } }
                                            }
                                        }
                                    }
                                    div { class: "oxs13-fb", role: "group", "aria-label": "Was this helpful?",
                                        if hv.sent {
                                            p { class: "oxs13-fbt", role: "status", span { class: "oxs12-hi", "data-hue": "green", {nico("check")} } "Thanks. Your note went to {PEOPLE[a.owner].0}." }
                                        } else if hv.vote == Some(true) {
                                            p { class: "oxs13-fbt", role: "status", span { class: "oxs12-hi", "data-hue": "green", {nico("check")} } "Thanks for the feedback. {up}% found this helpful." }
                                        } else if hv.vote == Some(false) {
                                            p { class: "oxs13-fbq", "What went wrong?" }
                                            div { class: "oxs13-rsn", role: "radiogroup", "aria-label": "Reason",
                                                for (i, r) in REASONS.iter().copied().enumerate() {
                                                    button { key: "{i}", r#type: "button", role: "radio", class: "oxs12-chip oxs13-sg", "aria-checked": "{hv.reason == Some(i)}", onclick: move |_| help.write().reason = Some(i), "{r}" }
                                                }
                                            }
                                            textarea { class: "oxs13-ta", rows: "2", placeholder: "Anything else? (optional)", "aria-label": "Comment", value: "{comment}", oninput: move |e| comment.set(e.value()), onkeydown: move |e: KeyboardEvent| e.stop_propagation() }
                                            div { class: "oxs13-qra",
                                                if hv.reason.is_none() { span { class: "oxs12-q", "Pick a reason to send" } }
                                                span { class: "oxs12-grow" }
                                                button { r#type: "button", class: "oxs12-btn oxs12-sm", "data-pri": "true", disabled: hv.reason.is_none(), onclick: move |_| { let r = help.write().send_feedback(); said.set(match r { Ok(s) => s, Err(e) => e.to_string() }); }, span { class: "oxs12-btl", "Send feedback" } }
                                            }
                                        } else {
                                            span { class: "oxs13-fbq", "Was this helpful?" }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let r = help.write().vote(true); if let Some(s) = r { said.set(s); } }, "Yes" }
                                            button { r#type: "button", class: "oxs12-btn oxs12-sm", onclick: move |_| { let r = help.write().vote(false); if let Some(s) = r { said.set(s); } }, "No" }
                                            span { class: "oxs12-grow" }
                                            span { class: "oxs12-q", "{up}% helpful · {a.up + a.down} votes" }
                                        }
                                    }
                                }
                                aside { class: "oxs13-ins", "aria-label": "Article insights",
                                    div { class: "oxs13-kv", span { class: "oxs12-q", "Views, 30 days" } b { "{thousands(a.views as u64)}" } }
                                    div { class: "oxs13-kv", span { class: "oxs12-q", "Helpful" } b { "{up}%" } span { class: "oxs12-prog", span { style: "width:{up}%" } } }
                                    div { class: "oxs13-kv", span { class: "oxs12-q", "Votes" } b { "{a.up} yes · {a.down} no" } }
                                    h3 { class: "oxs12-h2", "Top searches" }
                                    ul { class: "oxs13-tops",
                                        for (i, (s, c)) in SEARCHES.iter().copied().enumerate() {
                                            { let empty = oxyn_core::parity::arcscr2::search(&hv.arts, s).is_empty();
                                              rsx! { li { key: "{i}", button { r#type: "button", class: "oxs13-prb", onclick: move |_| q.set(s.to_string()), span { class: "oxs12-ell", "{s}" } if empty { span { class: "oxs13-sla", "data-hue": "rose", "No results" } } span { class: "oxs12-q", "{c}" } } } } }
                                        }
                                    }
                                }
                              } }
                        } else {
                            {none("doc", "Pick an article", "Choose an article on the left to read it here.", rsx! {})}
                        }
                    }
                }
            }
        }
    }
}
