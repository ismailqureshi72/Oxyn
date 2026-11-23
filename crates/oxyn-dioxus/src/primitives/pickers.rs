//! Pickers (plan #25): `Calendar`, `DatePicker`, `DateRangePicker`, `TimePicker`, `TimeDial`,
//! `DateWheel`.
//!
//! Logic lives in `oxyn_core::primitives::{date, dial}`. Dates cross the prop boundary as ISO
//! strings (`2026-11-13`) and times as `HH:mm`, so usage reads like the sources. The month grid is
//! one focusable grid with `aria-activedescendant` (one tab stop; arrows move the active day).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        date::{
            Blocked, CalKey, Calendar as CalModel, Compare, Date, DateFormat,
            DatePicker as DpModel, DateRange, DayCell, HourCycle, RangePicker, TimeOfDay, Travel,
            nearest_option, parse_natural, time_options,
        },
        dial::{City, DateWheel as ReelModel, ReelKey, ReelMode, TimeDial as DialModel},
    },
};

use super::cls;
use crate::composer::icons::Icon;

fn uid(prefix: &str, label: &str) -> String {
    let slug: String = label
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_lowercase();
    format!("oxyn-{prefix}-{slug}")
}

fn d(s: &str) -> Option<Date> {
    Date::parse_iso(s)
}

fn cal_key(e: &KeyboardEvent) -> Option<CalKey> {
    let shift = e.modifiers().shift();
    Some(match e.key() {
        Key::ArrowLeft => CalKey::Left,
        Key::ArrowRight => CalKey::Right,
        Key::ArrowUp => CalKey::Up,
        Key::ArrowDown => CalKey::Down,
        Key::Home => CalKey::Home,
        Key::End => CalKey::End,
        Key::PageUp => CalKey::PageUp { year: shift },
        Key::PageDown => CalKey::PageDown { year: shift },
        _ => return None,
    })
}

/// One month as a grid. `extra` adds classes per cell (range bands, heat).
#[component]
fn MonthGrid(
    id: String,
    title: String,
    weekdays: Vec<(&'static str, &'static str)>,
    rows: Vec<Vec<(DayCell, String)>>,
    /// Slide direction for the month change (`later` / `earlier`), and a key that changes with it.
    #[props(default)]
    slide: Option<(String, String)>,
    #[props(default)] week_numbers: bool,
    #[props(default)] focusable: bool,
    on_pick: EventHandler<Date>,
    #[props(default)] on_hover: EventHandler<Option<Date>>,
    #[props(default)] on_key: EventHandler<CalKey>,
    #[props(default)] on_enter: EventHandler<()>,
) -> Element {
    let focus = rows
        .iter()
        .flatten()
        .find(|(c, _)| c.focused)
        .map(|(c, _)| c.date);
    let (dir, k) = slide.unwrap_or_default();
    let _ = &k; // `key:` is compiled out in some rsx profiles
    rsx! {
        div { class: "mgrid", role: "grid", "aria-labelledby": "{id}-t", tabindex: if focusable { "0" } else { "-1" },
            "aria-activedescendant": if let Some(f) = focus { "{id}-{f.iso()}" },
            onkeydown: move |e: KeyboardEvent| {
                if let Some(k) = cal_key(&e) { e.prevent_default(); on_key.call(k) }
                else if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) { e.prevent_default(); on_enter.call(()) }
            },
            onpointerleave: move |_| on_hover.call(None),
            div { class: "mt", id: "{id}-t", key: "{title}", "data-dir": "{dir}", "aria-live": "polite", "{title}" }
            div { class: "wk", role: "row",
                if week_numbers { span { class: "wn", role: "columnheader", "aria-label": "Week", "Wk" } }
                for (s, l) in weekdays { span { role: "columnheader", "aria-label": "{l}", title: "{l}", "{s}" } }
            }
            div { class: "days", key: "{k}", "data-dir": "{dir}",
                for row in rows {
                    div { class: "wr", role: "row",
                        if week_numbers { span { class: "wn", role: "rowheader", "{row[0].0.date.add_days(3).iso_week()}" } }
                        for (c, extra) in row {
                            {
                                let dt = c.date;
                                rsx! {
                                    button { id: "{id}-{dt.iso()}", r#type: "button", tabindex: "-1", role: "gridcell",
                                        class: cls(&["day", if !c.in_month { "out" } else { "" }, if c.today { "today" } else { "" }, if c.selected { "sel" } else { "" }, if c.focused { "focus" } else { "" }, &extra]),
                                        "aria-label": "{dt.long()}", "aria-selected": "{c.selected || extra.contains(\"in\")}",
                                        "aria-current": if c.today { "date" }, "aria-disabled": "{c.disabled}",
                                        onclick: move |_| if !c.disabled { on_pick.call(dt) },
                                        onpointerenter: move |_| on_hover.call(Some(dt)),
                                        span { class: "n", "{dt.d}" }
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

fn plain_rows(c: &CalModel, heat: &[(String, u8)]) -> Vec<Vec<(DayCell, String)>> {
    c.grid()
        .into_iter()
        .map(|r| {
            r.into_iter()
                .map(|cell| {
                    let iso = cell.date.iso();
                    let h = heat
                        .iter()
                        .find(|(k, _)| *k == iso)
                        .map(|(_, l)| format!("heat h{}", l.min(&4)))
                        .unwrap_or_default();
                    (cell, h)
                })
                .collect()
        })
        .collect()
}

fn slide_of(c: &CalModel) -> Option<(String, String)> {
    Some((
        c.moved.map(|m| m.0.id()).unwrap_or("").to_string(),
        c.month.iso(),
    ))
}

#[component]
fn CalHead(
    title: String,
    can_prev: bool,
    can_next: bool,
    on_page: EventHandler<i32>,
    #[props(default)] today: bool,
    #[props(default)] on_today: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "ch",
            span { class: "ct", "aria-hidden": "true", key: "{title}", "{title}" }
            if today { button { class: "tdy", r#type: "button", onclick: move |_| on_today.call(()), "Today" } }
            button { class: "pg", r#type: "button", "aria-label": "Previous month", "aria-disabled": "{!can_prev}", onclick: move |_| if can_prev { on_page.call(-1) }, Icon { name: "chevron-left", size: 14.0 } }
            button { class: "pg", r#type: "button", "aria-label": "Next month", "aria-disabled": "{!can_next}", onclick: move |_| if can_next { on_page.call(1) }, Icon { name: "chevron-right", size: 14.0 } }
        }
    }
}

/// Calendar: a month grid for one date. Months slide in the direction of travel, the title rolls,
/// arrows move day by day skipping blocked days and crossing months, Page keys move by month
/// (Shift: year), a Today button. Oxyn plus: ISO week numbers and an activity heat overlay
/// (run history per day).
#[component]
pub fn Calendar(
    /// Picked date (ISO).
    #[props(default)]
    value: Option<String>,
    /// Today (ISO); pass it so server and client agree.
    #[props(default = "2026-11-13".to_string())]
    today: String,
    /// Earliest day (ISO).
    #[props(default)]
    min: Option<String>,
    /// Latest day (ISO).
    #[props(default)]
    max: Option<String>,
    /// `none` · `weekends`.
    #[props(default = "none".to_string())]
    blocked: String,
    /// 0 = Sunday, 1 = Monday.
    #[props(default)]
    week_starts_on: u8,
    /// Today button.
    #[props(default)]
    show_today: bool,
    /// Oxyn plus: ISO week numbers.
    #[props(default)]
    week_numbers: bool,
    /// Oxyn plus: activity per day `(ISO, 0–4)`.
    #[props(default)]
    activity: Vec<(String, u8)>,
    /// Accessible name.
    #[props(default = "Calendar".to_string())]
    label: String,
    /// Picked.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Month changed (first of the month, ISO).
    #[props(default)]
    on_month_change: EventHandler<String>,
) -> Element {
    let t = d(&today).unwrap_or(Date {
        y: 2026,
        m: 11,
        d: 13,
    });
    let mut c = use_signal(|| {
        let mut c = CalModel::new(value.as_deref().and_then(d), t);
        c.min = min.as_deref().and_then(d);
        c.max = max.as_deref().and_then(d);
        c.blocked = if blocked == "weekends" {
            Blocked::Weekends
        } else {
            Blocked::None
        };
        c.week_starts_on = week_starts_on.min(1);
        c.show_today = show_today;
        c.week_numbers = week_numbers;
        c
    });
    let id = uid("cal", &label);
    let st = c.read().clone();
    let mut pick = move |dt: Date| {
        let before = c.peek().month;
        let r = c.write().select(dt, ClockMs(1));
        if c.peek().month != before {
            on_month_change.call(c.peek().month.iso())
        }
        if let Some(x) = r {
            on_change.call(x.iso())
        }
    };
    rsx! {
        section { class: cls(&["oxyn-cal", if week_numbers { "wn" } else { "" }]), "aria-label": "{label}",
            CalHead { title: st.title(), can_prev: st.can_prev(), can_next: st.can_next(), today: show_today,
                on_page: move |n| { if c.write().page(n, ClockMs(1)) { on_month_change.call(c.peek().month.iso()) } },
                on_today: move |_| { let r = c.write().go_today(ClockMs(1)); if let Some(x) = r { on_change.call(x.iso()) } },
            }
            MonthGrid { id: id.clone(), title: st.title(), weekdays: st.weekdays(), rows: plain_rows(&st, &activity), slide: slide_of(&st),
                week_numbers, focusable: true,
                on_pick: move |dt: Date| { pick(dt); },
                on_key: move |k| { let before = c.peek().month; c.write().key(k, ClockMs(1)); if c.peek().month != before { on_month_change.call(c.peek().month.iso()) } },
                on_enter: move |_| { let f = c.peek().focus; pick(f) },
            }
            if !activity.is_empty() {
                div { class: "legend", "aria-hidden": "true", "Less" for l in 0..5u8 { span { class: "heat h{l}" } } "More" }
            }
        }
    }
}

/// Date wheel: iOS-style wheels for day, hour, minute and AM / PM (or month, day, year). Each reel is
/// a 3D cylinder; drag, flick or scroll it and it glides onto an entry; drags past the ends
/// stretch; Tab moves between reels, arrows turn, Page keys turn by a large step, typing jumps
/// (`4`, `45`, `p`, `ma`); quick picks spin every reel; a confirm button shows pending and success
/// in place. Oxyn plus: a natural-language field and a routine preset.
#[component]
pub fn DateWheel(
    /// `datetime` · `date` · `time`.
    #[props(default = "datetime".to_string())]
    mode: String,
    /// Starting day (ISO).
    #[props(default = "2026-09-24".to_string())]
    date: String,
    /// Starting time (`HH:mm`).
    #[props(default = "09:30".to_string())]
    time: String,
    /// Today (ISO).
    #[props(default = "2026-09-24".to_string())]
    today: String,
    /// Minutes between minute entries.
    #[props(default = 5)]
    minute_step: u32,
    /// `12` · `24`.
    #[props(default = "12".to_string())]
    hour_cycle: String,
    /// Heading above the summary.
    #[props(default)]
    title: Option<String>,
    /// Show quick picks.
    #[props(default = true)]
    presets: bool,
    /// Oxyn plus: a weekday-routine pick ("Weekdays 9:00").
    #[props(default)]
    routine: bool,
    /// Confirm button label (`None` hides it).
    #[props(default)]
    confirm: Option<String>,
    /// Glide speed multiplier.
    #[props(default = 1.0)]
    speed: f32,
    /// Oxyn plus: type a moment in words.
    #[props(default)]
    natural: bool,
    /// Accessible name.
    #[props(default = "Date and time".to_string())]
    label: String,
    /// Came to rest on a new moment: `(ISO date, HH:mm)`.
    #[props(default)]
    on_change: EventHandler<(String, String)>,
    /// Confirmed.
    #[props(default)]
    on_confirm: EventHandler<(String, String)>,
) -> Element {
    let t = d(&today).unwrap_or(Date {
        y: 2026,
        m: 9,
        d: 24,
    });
    let mut m = use_signal(|| {
        let v = (
            d(&date).unwrap_or(t),
            TimeOfDay::parse(&time).unwrap_or_default(),
        );
        let mut r = ReelModel::new(ReelMode::from_id(&mode), v, t);
        // Rebuild with the requested step and cycle.
        r.minute_step = minute_step;
        r.cycle = HourCycle::from_id(&hour_cycle);
        r.speed = speed;
        let mut fresh = ReelModel::new(ReelMode::from_id(&mode), v, t);
        fresh.minute_step = r.minute_step;
        fresh.cycle = r.cycle;
        fresh.speed = r.speed;
        fresh.rebuild(v);
        fresh
    });
    let mut drag_y = use_signal(|| None::<(usize, f64, f32)>);
    let mut typed_clock = use_signal(|| 0u64);
    let mut typed_n = use_signal(|| 0u32);
    let mut phase = use_signal(|| 0u8); // 0 idle · 1 pending · 2 done
    let mut words = use_signal(String::new);
    let st = m.read().clone();
    let row_h = 34.0f64;
    let (vd, vt) = st.value();
    let mut emit = move || {
        let (dd, tt) = m.peek().value();
        m.write().committed = (dd, tt);
        on_change.call((dd.iso(), tt.hhmm()));
    };
    let mut emit2 = emit;
    rsx! {
        div { class: "oxyn-reel", role: "group", "aria-label": "{label}",
            if let Some(tt) = title.clone() { div { class: "rt", "{tt}" } }
            div { class: "sum", "aria-hidden": "true", span { class: "sv", key: "{st.summary()}", "{st.summary()}" } span { class: "rel", "{st.relative()}" } }
            div { class: "wheels",
                div { class: "lens", "aria-hidden": "true" }
                for (ri, reel) in st.reels.iter().enumerate() {
                    {
                        let dragging = drag_y.read().is_some_and(|x| x.0 == ri);
                        let pos = if dragging { reel.pos } else { reel.target as f32 };
                        let n = reel.entries.len();
                        let lo = (pos.round() as i64 - 5).max(0) as usize;
                        let hi = ((pos.round() as i64 + 6).max(0) as usize).min(n);
                        let dist = (reel.pos - reel.target as f32).abs();
                        let dur = (220.0 + dist * 45.0).min(900.0) / speed.max(0.1);
                        let name = reel.kind.name();
                        rsx! {
                            div { class: cls(&["reel", if dragging { "drag" } else { "" }]), role: "spinbutton", tabindex: "0",
                                "aria-label": "{name}", "aria-valuenow": "{reel.index()}", "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuetext": "{st.valuetext(ri)}",
                                style: "--dur:{dur:.0}ms",
                                onkeydown: move |e: KeyboardEvent| {
                                    let k = match e.key() { Key::ArrowUp => Some(ReelKey::Up), Key::ArrowDown => Some(ReelKey::Down), Key::PageUp => Some(ReelKey::PageUp), Key::PageDown => Some(ReelKey::PageDown), Key::Home => Some(ReelKey::Home), Key::End => Some(ReelKey::End), _ => None };
                                    if let Some(k) = k { e.prevent_default(); m.write().key(ri, k); let tg = m.peek().reels[ri].target as f32; m.write().reels[ri].pos = tg; emit(); }
                                    else if let Key::Character(c) = e.key() && let Some(ch) = c.chars().next() && ch.is_alphanumeric() {
                                        let now = ClockMs(*typed_clock.peek());
                                        if m.write().typeahead(ri, ch, now) { emit(); }
                                        typed_n += 1;
                                    }
                                },
                                onwheel: move |e: WheelEvent| { e.prevent_default(); let dy = e.delta().strip_units().y; if dy.abs() > 1.0 { m.write().key(ri, if dy > 0.0 { ReelKey::Down } else { ReelKey::Up }); emit2() } },
                                onpointerdown: move |e: PointerEvent| { let y = e.client_coordinates().y; let p = m.peek().reels[ri].target as f32; m.write().reels[ri].pos = p; drag_y.set(Some((ri, y, 0.0))); },
                                onpointermove: move |e: PointerEvent| {
                                    let Some((r0, y0, _)) = *drag_y.peek() else { return };
                                    if r0 != ri { return; }
                                    let y = e.client_coordinates().y;
                                    let rows = (y0 - y) / row_h;
                                    m.write().reels[ri].drag(rows as f32);
                                    drag_y.set(Some((ri, y, rows as f32)));
                                },
                                onpointerup: move |_| {
                                    let Some((r0, _, last)) = *drag_y.peek() else { return };
                                    if r0 != ri { return; }
                                    drag_y.set(None);
                                    let sp = m.peek().speed;
                                    m.write().reels[ri].release(last * 60.0, sp);
                                    let (dd, tt) = m.peek().value();
                                    m.write().spin_to((dd, tt));
                                    let mut w = m.write();
                                    let tg = w.reels[ri].target as f32;
                                    w.reels[ri].pos = tg;
                                    w.reels[ri].dragging = false;
                                    drop(w);
                                    on_change.call((dd.iso(), tt.hhmm()));
                                },
                                div { class: "drum", style: "transform:translateZ(-60px) rotateX({pos * 20.0:.2}deg)",
                                    for i in lo..hi {
                                        div { class: cls(&["ent", if i == reel.index() { "cur" } else { "" }, if !reel.valid[i] { "bad" } else { "" }]),
                                            style: "transform:rotateX({-(i as f32) * 20.0}deg) translateZ(96px)",
                                            onclick: move |_| { m.write().reels[ri].spin_to(i); m.write().reels[ri].pos = i as f32; emit() },
                                            "{reel.entries[i]}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if *typed_n.read() > 0 { span { class: "oxyn-timer", key: "{typed_n}", style: "animation-duration:1000ms", onanimationend: move |_| typed_clock += 2_000 } }
            if natural {
                input { class: "nl", placeholder: "Or type: tomorrow 6 pm", value: "{words}", "aria-label": "Type a date and time",
                    oninput: move |e: FormEvent| words.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); let w = words.peek().clone(); if m.write().enter_text(&w) { words.set(String::new()); let (dd, tt) = m.peek().value(); on_change.call((dd.iso(), tt.hhmm())) } },
                }
            }
            div { class: "foot",
                if presets {
                    div { class: "picks", role: "group", "aria-label": "Quick picks",
                        for (i, p) in st.presets.iter().enumerate() {
                            button { class: cls(&["pc", if st.active_preset() == Some(i) { "on" } else { "" }]), r#type: "button", "aria-pressed": "{st.active_preset() == Some(i)}",
                                onclick: move |_| { m.write().preset(i); let (dd, tt) = m.peek().value(); on_change.call((dd.iso(), tt.hhmm())) }, "{p.label}" }
                        }
                        if routine {
                            button { class: "pc", r#type: "button", title: "Every weekday at 9:00",
                                onclick: move |_| { let mut w = m.write(); let t0 = w.today; let next = (1..8).map(|k| t0.add_days(k)).find(|x| (1..=5).contains(&x.weekday())).unwrap_or(t0); w.spin_to((next, TimeOfDay::new(9, 0))); drop(w); let (dd, tt) = m.peek().value(); on_change.call((dd.iso(), tt.hhmm())) },
                                Icon { name: "retry", size: 11.0 } "Weekdays 9:00" }
                        }
                    }
                }
                if let Some(cl) = confirm.clone() {
                    button { class: cls(&["oxyn-btn primary sm conf", match *phase.read() { 1 => "pending", 2 => "done", _ => "" }]), r#type: "button", "aria-busy": "{*phase.read() == 1}",
                        onclick: move |_| { if *phase.peek() == 0 { phase.set(1); on_confirm.call((vd.iso(), vt.hhmm())) } },
                        span { class: "lb", match *phase.read() { 1 => "Scheduling…".to_string(), 2 => "Scheduled".to_string(), _ => cl.clone() } }
                    }
                    if *phase.read() == 1 { span { class: "oxyn-timer", style: "animation-duration:900ms", onanimationend: move |_| phase.set(2) } }
                    if *phase.read() == 2 { span { class: "oxyn-timer", style: "animation-duration:1600ms", onanimationend: move |_| phase.set(0) } }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.summary()}" }
        }
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
