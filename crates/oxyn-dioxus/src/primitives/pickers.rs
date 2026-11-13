//! Pickers (plan #25): `Calendar`, `DatePicker`, `DateRangePicker`, `TimePicker`, `TimeDial`,
//! `DateReel`.
//!
//! Logic lives in `oxyn_core::primitives::{date, dial}`. Dates cross the prop boundary as ISO
//! strings (`2026-11-13`) and times as `HH:mm`, so usage reads like the sources. The month grid is
//! one focusable grid with `aria-activedescendant` (one tab stop; arrows move the active day).

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        date::{
            Blocked, CalKey, Calendar as CalModel, Compare, Date, DateFormat,
            DatePicker as DpModel, DateRange, DayCell, HourCycle, RangePicker, TimeOfDay, Travel,
            nearest_option, parse_natural, time_options,
        },
        dial::{City, DateReel as ReelModel, ReelKey, ReelMode, TimeDial as DialModel},
    },
};

use super::cls;
use crate::composer::icons::Icon;

/// An applied range: `(start, end, compare (start, end))` as ISO dates.
pub type RangeValue = (String, String, Option<(String, String)>);

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

/// Date picker: a field button that opens the calendar in a popover and shows the date with the
/// changed parts rolling (up for later, down for earlier); the popover closes 240 ms after a pick
/// so the highlight can land; Clear. Oxyn plus: natural-language entry (`next tue`, `in 2 weeks`)
/// and relative presets.
#[component]
pub fn DatePicker(
    /// Label.
    label: String,
    /// Picked date (ISO).
    #[props(default)]
    value: Option<String>,
    /// Helper copy.
    #[props(default)]
    description: Option<String>,
    /// Placeholder.
    #[props(default = "Select a date".to_string())]
    placeholder: String,
    /// Today (ISO).
    #[props(default = "2026-11-13".to_string())]
    today: String,
    /// Earliest day.
    #[props(default)]
    min: Option<String>,
    /// Latest day.
    #[props(default)]
    max: Option<String>,
    /// `none` · `weekends`.
    #[props(default = "none".to_string())]
    blocked: String,
    /// `medium` · `long` · `weekday` · `day-first` · `month-first` · `iso`.
    #[props(default = "medium".to_string())]
    format: String,
    /// Today button.
    #[props(default)]
    show_today: bool,
    /// Oxyn plus: type a date in words.
    #[props(default)]
    natural: bool,
    /// Oxyn plus: relative presets (Today, Tomorrow, Next week).
    #[props(default)]
    presets: bool,
    /// Start open (review sheets).
    #[props(default)]
    open: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Picked (or cleared: `None`).
    #[props(default)]
    on_change: EventHandler<Option<String>>,
) -> Element {
    let t = d(&today).unwrap_or(Date {
        y: 2026,
        m: 11,
        d: 13,
    });
    let mut p = use_signal(|| {
        let mut p = DpModel::new(value.as_deref().and_then(d), t);
        p.cal.min = min.as_deref().and_then(d);
        p.cal.max = max.as_deref().and_then(d);
        p.cal.blocked = if blocked == "weekends" {
            Blocked::Weekends
        } else {
            Blocked::None
        };
        p.format = DateFormat::from_id(&format);
        if open {
            p.open_popover();
        }
        p
    });
    let mut typed = use_signal(String::new);
    let id = uid("dp", &label);
    let st = p.read().clone();
    let parts = st.parts();
    let dir = st.travel().id();
    let mut pick = move |dt: Date| {
        let r = p.write().pick(dt, ClockMs(1), false);
        if let Some(x) = r {
            on_change.call(Some(x.iso()))
        }
    };
    let rel: Vec<(&str, Date)> = vec![
        ("Today", t),
        ("Tomorrow", t.add_days(1)),
        ("Next week", t.add_days(7)),
    ];
    rsx! {
        div { class: cls(&["oxyn-field oxyn-dp", if st.open { "open" } else { "" }, if disabled { "disabled" } else { "" }]),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && p.peek().open { e.prevent_default(); p.write().close() },
            label { class: "flabel", id: "{id}-l", r#for: "{id}", "{label}" }
            div { class: "anchor",
                div { class: "control trig",
                    button { id: "{id}", class: cls(&["tb", if st.value().is_none() { "placeholder" } else { "" }]), r#type: "button", disabled,
                        "aria-haspopup": "dialog", "aria-expanded": "{st.open}", "aria-labelledby": "{id}-l {id}-v",
                        onclick: move |_| { let o = p.peek().open; if o { p.write().close() } else { p.write().open_popover() } },
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::ArrowDown && !p.peek().open { e.prevent_default(); p.write().open_popover() },
                        Icon { name: "calendar", size: 14.0 }
                        span { class: "sr", id: "{id}-v", if let Some(v) = st.value() { "{st.format.apply(v)}" } else { "{placeholder}" } }
                        span { class: "parts", "aria-hidden": "true",
                            if parts.is_empty() { "{placeholder}" }
                            for (i, (txt, changed)) in parts.into_iter().enumerate() {
                                span { key: "{i}-{txt}", class: if changed { "pt roll" } else { "pt" }, "data-dir": "{dir}", "{txt}" }
                            }
                        }
                    }
                    if st.value().is_some() && !disabled {
                        button { class: "act", r#type: "button", "aria-label": "Clear date", onclick: move |_| { p.write().clear(); on_change.call(None) }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                    }
                }
                if st.open {
                    div { class: "pop", role: "dialog", "aria-label": "{label} calendar",
                        if natural {
                            div { class: "nl",
                                input { class: "in", placeholder: "Type a date: next tue, in 2 weeks…", value: "{typed}", "aria-label": "Type a date",
                                    oninput: move |e: FormEvent| typed.set(e.value()),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); let v = typed.peek().clone(); let r = p.write().enter_text(&v, ClockMs(1)); if let Some(x) = r { typed.set(String::new()); on_change.call(Some(x.iso())); p.write().close_at = Some(ClockMs(241)); } },
                                }
                                if !typed.read().is_empty() {
                                    span { class: "hint", if let Some(x) = parse_natural(&typed.read(), t) { "↵ {x.format(DateFormat::Weekday)}" } else { "Not a date yet" } }
                                }
                            }
                        }
                        section { class: "oxyn-cal",
                            CalHead { title: st.cal.title(), can_prev: st.cal.can_prev(), can_next: st.cal.can_next(), today: show_today,
                                on_page: move |n| { p.write().cal.page(n, ClockMs(1)); },
                                on_today: move |_| { let t = p.peek().cal.today; pick(t) },
                            }
                            MonthGrid { id: id.clone(), title: st.cal.title(), weekdays: st.cal.weekdays(), rows: plain_rows(&st.cal, &[]), slide: slide_of(&st.cal), focusable: true,
                                on_pick: move |dt: Date| { pick(dt); },
                                on_key: move |k| { p.write().cal.key(k, ClockMs(1)); },
                                on_enter: move |_| { let f = p.peek().cal.focus; pick(f) },
                            }
                        }
                        if presets {
                            div { class: "chips",
                                for (l, dt) in rel { button { class: cls(&["pc", if st.value() == Some(dt) { "on" } else { "" }]), r#type: "button", "aria-pressed": "{st.value() == Some(dt)}", onclick: move |_| pick(dt), "{l}" } }
                            }
                        }
                        if st.close_at.is_some() { span { class: "oxyn-timer", style: "animation-duration:240ms", onanimationend: move |_| p.write().close() } }
                    }
                }
            }
            if let Some(dd) = description { div { class: "foot", div { class: "msg", "{dd}" } } }
        }
    }
}

/// Date range picker: presets and two months with a hover preview after the first click, a
/// comparison with the previous period or year (grey band, outlined ends), Apply / Cancel; one
/// month and preset chips on narrow screens. One model for affectUI's and Arc's pickers. Oxyn
/// plus: agent-run presets (Last 90 days, Last 12 months), a day count and the compare range spelled
/// out in the footer.
#[component]
pub fn DateRangePicker(
    /// Accessible name of the trigger and the dialog.
    #[props(default = "Date range".to_string())]
    label: String,
    /// Starting range start (ISO).
    #[props(default)]
    start: Option<String>,
    /// Starting range end (ISO).
    #[props(default)]
    end: Option<String>,
    /// Placeholder.
    #[props(default = "Select dates".to_string())]
    placeholder: String,
    /// Today (ISO); reference for presets.
    #[props(default = "2026-11-13".to_string())]
    today: String,
    /// Offer the comparison control.
    #[props(default = true)]
    compare: bool,
    /// `off` · `previous` · `year`.
    #[props(default = "previous".to_string())]
    default_compare: String,
    /// 0 = Sunday, 1 = Monday.
    #[props(default = 1)]
    week_starts_on: u8,
    /// `1` · `2` months.
    #[props(default = 2)]
    months: u8,
    /// Earliest day.
    #[props(default)]
    min: Option<String>,
    /// Latest day (default: today).
    #[props(default)]
    max: Option<String>,
    /// Open right away.
    #[props(default)]
    default_open: bool,
    /// Applied: `(start, end, compare start, compare end)` as ISO.
    #[props(default)]
    on_change: EventHandler<RangeValue>,
) -> Element {
    let t = d(&today).unwrap_or(Date {
        y: 2026,
        m: 11,
        d: 13,
    });
    let mut r = use_signal(|| {
        let v = match (start.as_deref().and_then(d), end.as_deref().and_then(d)) {
            (Some(a), Some(b)) => Some(DateRange::new(a, b)),
            _ => Some(DateRange::new(t.add_days(-29), t)),
        };
        let mut r = RangePicker::new(v, t);
        r.compare = Compare::from_id(&default_compare);
        r.compare_enabled = compare;
        r.cal.week_starts_on = week_starts_on.min(1);
        r.months = months.clamp(1, 2);
        if r.months == 1 {
            r.cal.month = v.map_or(t, |x| x.end).month_start();
        }
        r.cal.min = min.as_deref().and_then(d);
        r.cal.max = max.as_deref().and_then(d).or(Some(t));
        if default_open {
            r.open_panel(ClockMs(0));
        }
        r
    });
    let id = uid("drp", &label);
    let st = r.read().clone();
    let grids = st.grids();
    let active = st.active_preset();
    let cmp = st.draft.and_then(|x| x.compare(st.compare));
    rsx! {
        div { class: cls(&["oxyn-drp", if st.open { "open" } else { "" }, if st.months == 1 { "one" } else { "" }]),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && r.peek().open { e.prevent_default(); r.write().cancel() },
            button { class: "trig", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{st.open}",
                "aria-label": "{label}: {st.trigger_label(&placeholder)}",
                onclick: move |_| { let o = r.peek().open; if o { r.write().cancel() } else { r.write().open_panel(ClockMs(0)) } },
                Icon { name: "calendar", size: 14.0 }
                span { class: "lbl", key: "{st.trigger_label(&placeholder)}", "{st.trigger_label(&placeholder)}" }
                if let (Some(c), false) = (st.value.and_then(|v| v.compare(st.compare)), st.compare == Compare::Off || !compare) { span { class: "cmp", "vs {c.label()}" } }
                Icon { name: "chevron", size: 12.0, stroke: 2.0 }
            }
            if st.open {
                div { class: "panel", role: "dialog", "aria-label": "{label}",
                    div { class: "rail", role: "listbox", "aria-label": "Presets",
                        for p in st.presets.clone() {
                            button { class: cls(&["pr", if active == Some(p.id) { "on" } else { "" }]), r#type: "button", role: "option", "aria-selected": "{active == Some(p.id)}",
                                onclick: move |_| { r.write().preset(p.id, ClockMs(1)); },
                                "{p.label}"
                            }
                        }
                    }
                    div { class: "main",
                        div { class: "months",
                            for (gi, (title, _, _)) in grids.iter().enumerate() {
                                {
                                    let mut cal = st.cal.clone();
                                    cal.month = st.cal.month.add_months(gi as i32);
                                    let rows: Vec<Vec<(DayCell, String)>> = cal.grid().into_iter().map(|row| row.into_iter().map(|c| {
                                        if !c.in_month { return (c, String::new()); }
                                        let b = st.band(c.date);
                                        let mut x = vec![];
                                        if b.inside { x.push("in"); }
                                        if b.start { x.push("bs"); }
                                        if b.end { x.push("be"); }
                                        if b.preview && b.inside { x.push("pv"); }
                                        if b.compare { x.push("cmp"); }
                                        if b.compare_edge { x.push("ce"); }
                                        (DayCell { focused: c.focused && st.cal.focus.month_start() == cal.month, ..c }, x.join(" "))
                                    }).collect()).collect();
                                    let last = gi + 1 == grids.len();
                                    rsx! {
                                        section { class: "oxyn-cal",
                                            div { class: "ch",
                                                if gi == 0 { button { class: "pg", r#type: "button", "aria-label": "Previous month", onclick: move |_| { r.write().page(-1, ClockMs(1)); }, Icon { name: "chevron-left", size: 14.0 } } }
                                                span { class: "ct", key: "{title}", "{title}" }
                                                if last { button { class: "pg", r#type: "button", "aria-label": "Next month", "aria-disabled": "{!st.cal.can_next()}", onclick: move |_| { r.write().page(1, ClockMs(1)); }, Icon { name: "chevron-right", size: 14.0 } } }
                                            }
                                            MonthGrid { id: format!("{id}-{gi}"), title: title.clone(), weekdays: st.cal.weekdays(), rows, slide: slide_of(&cal), focusable: gi == 0,
                                                on_pick: move |dt| { r.write().pick(dt, ClockMs(1)); },
                                                on_hover: move |dt| r.write().hover(dt),
                                                on_key: move |k| { r.write().key(k, ClockMs(1)); },
                                                on_enter: move |_| { let f = r.peek().cal.focus; r.write().pick(f, ClockMs(1)); },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if compare {
                            div { class: "cmprow",
                                span { class: "cl", "Compare" }
                                div { class: "seg", role: "radiogroup", "aria-label": "Compare",
                                    for c in [Compare::Off, Compare::Previous, Compare::Year] {
                                        button { r#type: "button", role: "radio", "aria-checked": "{st.compare == c}", class: if st.compare == c { "on" } else { "" },
                                            onclick: move |_| r.write().compare = c, "{c.label()}" }
                                    }
                                }
                            }
                        }
                        div { class: "foot",
                            div { class: "sum",
                                if let Some(x) = st.draft { span { class: "rl", "{x.label()}" } span { class: "dn", " · {x.days()} days" } }
                                if let Some(c) = cmp { if compare && st.compare != Compare::Off { span { class: "cmpl", "vs {c.label()}" } } }
                            }
                            button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| r.write().cancel(), "Cancel" }
                            button { class: "oxyn-btn primary sm", r#type: "button", disabled: st.anchor.is_some(),
                                onclick: move |_| { let a = r.write().apply(); if let Some((x, c)) = a { on_change.call((x.start.iso(), x.end.iso(), c.map(|c| (c.start.iso(), c.end.iso())))) } },
                                "Apply"
                            }
                        }
                    }
                    span { class: "sr", role: "status", "aria-live": "polite", "{st.announce()}" }
                }
            }
        }
    }
}

/// Time picker: a trigger with a list of times at a fixed step; the shown time rises for a later
/// time and drops for an earlier one; the list opens centred on the selected time. Oxyn plus: a
/// window (working hours) that limits the list, a typed entry (`9:30 pm`), and a time zone tag.
#[component]
pub fn TimePicker(
    /// Label.
    label: String,
    /// `HH:mm` (24-hour).
    #[props(default)]
    value: Option<String>,
    /// Helper copy.
    #[props(default)]
    description: Option<String>,
    /// Placeholder.
    #[props(default = "Select a time".to_string())]
    placeholder: String,
    /// Minutes between options: 1 · 5 · 10 · 15 · 30.
    #[props(default = 15)]
    minute_step: u32,
    /// `12h` · `24h`.
    #[props(default = "12h".to_string())]
    format: String,
    /// Oxyn plus: limit to `HH:mm-HH:mm`.
    #[props(default)]
    window: Option<String>,
    /// Oxyn plus: a zone tag after the value (`PKT`).
    #[props(default)]
    zone: Option<String>,
    /// Start open (review sheets).
    #[props(default)]
    open: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed: `HH:mm`.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let cycle = HourCycle::from_id(&format);
    let win = window.as_deref().and_then(|w| {
        let (a, b) = w.split_once('-')?;
        Some((TimeOfDay::parse(a)?, TimeOfDay::parse(b)?))
    });
    let opts = time_options(minute_step, cycle, win);
    let mut v = use_signal(|| value.as_deref().and_then(TimeOfDay::parse));
    let mut is_open = use_signal(|| open);
    let mut active = use_signal(|| None::<usize>);
    let mut dir = use_signal(|| Travel::Forward);
    let id = uid("tp", &label);
    let cur = *v.read();
    let shown = cur.map(|t| t.label(cycle));
    let n = opts.len();
    let opt_sig = use_signal(|| opts.clone());
    let mut open_list = move || {
        let at = v
            .peek()
            .and_then(|t| nearest_option(&opt_sig.peek(), t))
            .unwrap_or(0);
        active.set(Some(at));
        is_open.set(true);
    };
    let mut choose = move |i: usize| {
        let o = opt_sig.peek().get(i).cloned();
        if let Some((val, _)) = o
            && let Some(t) = TimeOfDay::parse(&val)
        {
            if let Some(old) = *v.peek() {
                dir.set(Travel::between(old, t));
            }
            v.set(Some(t));
            on_change.call(val);
        }
        is_open.set(false);
    };
    let open_now = *is_open.read();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-select oxyn-tp", if open_now { "open" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: "flabel", id: "{id}-l", r#for: "{id}", "{label}" }
            div { class: "anchor",
                button { id: "{id}", class: cls(&["control trigger", if cur.is_none() { "placeholder" } else { "" }]), r#type: "button", disabled,
                    "aria-haspopup": "listbox", "aria-expanded": "{open_now}", "aria-controls": "{id}-list", "aria-labelledby": "{id}-l {id}-v",
                    "aria-activedescendant": if let (true, Some(a)) = (open_now, *active.read()) { "{id}-o{a}" },
                    onclick: move |_| if *is_open.peek() { is_open.set(false) } else { open_list() },
                    onblur: move |_| is_open.set(false),
                    onkeydown: move |e: KeyboardEvent| {
                        let o = *is_open.peek();
                        match e.key() {
                            Key::ArrowDown | Key::ArrowUp if !o => { e.prevent_default(); open_list() }
                            Key::ArrowDown => { e.prevent_default(); let a = active.peek().map_or(0, |a| (a + 1) % n.max(1)); active.set(Some(a)) }
                            Key::ArrowUp => { e.prevent_default(); let a = active.peek().map_or(0, |a| (a + n.max(1) - 1) % n.max(1)); active.set(Some(a)) }
                            Key::Enter => { e.prevent_default(); if o { if let Some(a) = *active.peek() { choose(a) } } else { open_list() } }
                            Key::Character(c) if c == " " => { e.prevent_default(); if o { if let Some(a) = *active.peek() { choose(a) } } else { open_list() } }
                            Key::Escape if o => { e.prevent_default(); is_open.set(false) }
                            _ => {}
                        }
                    },
                    Icon { name: "clock", size: 14.0 }
                    span { class: "sr", id: "{id}-v", "{shown.clone().unwrap_or(placeholder.clone())}" }
                    span { class: "val", key: "{shown.clone().unwrap_or_default()}", "data-move": "{dir.read().id()}", "aria-hidden": "true", "{shown.clone().unwrap_or(placeholder.clone())}" }
                    if let (Some(z), true) = (zone.clone(), cur.is_some()) { span { class: "zone", "{z}" } }
                    span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
                }
                if open_now {
                    div { class: "list", id: "{id}-list", role: "listbox", "aria-labelledby": "{id}-l",
                        onmousedown: move |e| e.prevent_default(),
                        for (i, (val, lbl)) in opts.iter().enumerate() {
                            {
                                let on = cur.map(|t| t.hhmm()) == Some(val.clone());
                                let hl = *active.read() == Some(i);
                                rsx! {
                                    div { id: "{id}-o{i}", class: cls(&["row", if hl { "hl-self" } else { "" }]), role: "option", "aria-selected": "{on}",
                                        onmounted: move |e| async move { if hl { let _ = e.data().scroll_to(ScrollBehavior::Instant).await; } },
                                        onpointerenter: move |_| active.set(Some(i)),
                                        onclick: move |_| choose(i),
                                        span { class: "txt", span { class: "lb", "{lbl}" } }
                                        if on { span { class: "tick", svg { view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(dd) = description { div { class: "msg", "{dd}" } }
        }
    }
}

/// Time dial: a 24-hour rotary dial for choosing a time across time zones. Every city rides the
/// dial at its own local time; a band marks working hours; the home city reads out in the centre
/// with odometer digits and a sun or moon. Drag with soft quarter-hour detents; a flick coasts to
/// the nearest quarter hour; bounds rubber-band. Oxyn plus: a quiet-hours band and a "best
/// overlap" jump.
#[component]
pub fn TimeDial(
    /// Two to five cities.
    cities: Vec<City>,
    /// UTC minutes from midnight on the reference day.
    #[props(default = 600)]
    value: i32,
    /// Home city id (defaults to the first).
    #[props(default)]
    home: Option<String>,
    /// Earliest UTC minute.
    #[props(default)]
    min: Option<i32>,
    /// Latest UTC minute.
    #[props(default)]
    max: Option<i32>,
    /// Weekday of the reference day (0 = Sunday).
    #[props(default = 1)]
    weekday: u8,
    /// `12` · `24`.
    #[props(default = "12".to_string())]
    hour_cycle: String,
    /// Local working hours `[start, end]` in minutes.
    #[props(default = (540, 1080))]
    working_hours: (i32, i32),
    /// Oxyn plus: local quiet hours in minutes.
    #[props(default)]
    quiet_hours: Option<(i32, i32)>,
    /// Oxyn plus: show "Best overlap".
    #[props(default)]
    overlap: bool,
    /// Accessible name.
    #[props(default = "Time".to_string())]
    label: String,
    /// The destination after a release or a key.
    #[props(default)]
    on_change: EventHandler<i32>,
    /// Home city chosen.
    #[props(default)]
    on_home_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut x = DialModel::new(cities.clone(), value);
        x.min = min;
        x.max = max;
        x.weekday = weekday;
        x.cycle = HourCycle::from_id(&hour_cycle);
        x.working = working_hours;
        x.quiet = quiet_hours;
        if let Some(h) = &home
            && let Some(i) = cities.iter().position(|c| c.id == *h)
        {
            x.home = i;
        }
        x
    });
    let mut root = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut center = use_signal(|| (0.0f64, 0.0f64));
    let mut last_angle = use_signal(|| None::<f32>);
    let mut last_delta = use_signal(|| 0.0f32);
    let mut dragging = use_signal(|| false);
    let st = m.read().clone();
    let (cx, cy, r) = (128.0f32, 128.0f32, 104.0f32);
    let home_i = st.home;
    let (hm, _) = st.local(home_i);
    // The face is the home city's day: rotate it so the home time sits at the top.
    let rot = -(DialModel::angle(hm).to_degrees());
    let arc = |a: i32, b: i32, rr: f32| -> String {
        let (x1, y1) = DialModel::point(a, cx, cy, rr);
        let (x2, y2) = DialModel::point(b, cx, cy, rr);
        let span = (b - a).rem_euclid(1440);
        let large = i32::from(span > 720);
        format!("M{x1:.1} {y1:.1} A{rr} {rr} 0 {large} 1 {x2:.1} {y2:.1}")
    };
    let angle_at = move |px: f64, py: f64| -> f32 {
        let (ox, oy) = *center.peek();
        ((px - ox) as f32).atan2(-(py - oy) as f32)
    };
    let digits = st.time_label(home_i);
    let day = st.daytime(home_i);
    rsx! {
        div { class: cls(&["oxyn-dial", if *dragging.read() { "dragging" } else { "" }]),
            div { class: "face", role: "slider", tabindex: "0", "aria-label": "{label}",
                "aria-valuemin": "0", "aria-valuemax": "1439", "aria-valuenow": "{hm}", "aria-valuetext": "{st.valuetext()}",
                onmounted: move |e| root.set(Some(e.data())),
                onkeydown: move |e: KeyboardEvent| {
                    let by = match e.key() { Key::ArrowRight | Key::ArrowUp => Some(15), Key::ArrowLeft | Key::ArrowDown => Some(-15), Key::PageUp => Some(60), Key::PageDown => Some(-60), _ => None };
                    if let Some(b) = by { e.prevent_default(); let v = m.write().step(b, ClockMs(0)); on_change.call(v) }
                    else if e.key() == Key::Home || e.key() == Key::End { e.prevent_default(); let v = m.write().edge(e.key() == Key::End, ClockMs(0)); on_change.call(v) }
                },
                onpointerdown: move |e: PointerEvent| {
                    let p = e.client_coordinates();
                    dragging.set(true);
                    m.write().grab();
                    if let Some(rt) = root.peek().clone() {
                        spawn(async move {
                            if let Ok(rc) = rt.get_client_rect().await {
                                center.set((rc.origin.x + rc.size.width / 2.0, rc.origin.y + rc.size.height / 2.0));
                                last_angle.set(Some(angle_at(p.x, p.y)));
                            }
                        });
                    }
                },
                onpointermove: move |e: PointerEvent| {
                    if !*dragging.peek() { return; }
                    let p = e.client_coordinates();
                    let a = angle_at(p.x, p.y);
                    if let Some(prev) = *last_angle.peek() {
                        let mut da = a - prev;
                        if da > std::f32::consts::PI { da -= std::f32::consts::TAU }
                        if da < -std::f32::consts::PI { da += std::f32::consts::TAU }
                        // Dragging clockwise moves the time forward (the face turns under the knob).
                        let dm = -da / std::f32::consts::TAU * 1440.0;
                        last_delta.set(dm);
                        m.write().drag(dm, ClockMs(0));
                    }
                    last_angle.set(Some(a));
                },
                onpointerup: move |_| if *dragging.peek() { dragging.set(false); last_angle.set(None); let v = *last_delta.peek() * 60.0; let t = m.write().release(v, ClockMs(0), true); on_change.call(t) },
                onpointerleave: move |_| if *dragging.peek() { dragging.set(false); last_angle.set(None); let t = m.write().release(0.0, ClockMs(0), true); on_change.call(t) },
                svg { view_box: "0 0 256 256", width: "100%", height: "100%", "aria-hidden": "true",
                    g { class: "rot", style: "transform:rotate({rot:.2}deg)",
                        circle { class: "ring", cx: "{cx}", cy: "{cy}", r: "{r}" }
                        path { class: "work", d: "{arc(st.working.0, st.working.1, r)}" }
                        if let Some((a, b)) = st.quiet { path { class: "quiet", d: "{arc(a, b, r)}" } }
                        for h in 0..24 {
                            {
                                let (x1, y1) = DialModel::point(h * 60, cx, cy, r - 14.0);
                                let (x2, y2) = DialModel::point(h * 60, cx, cy, r - if h % 6 == 0 { 22.0 } else { 18.0 });
                                let (tx, ty) = DialModel::point(h * 60, cx, cy, r - 32.0);
                                rsx! {
                                    line { class: if h % 6 == 0 { "tick major" } else { "tick" }, x1: "{x1:.1}", y1: "{y1:.1}", x2: "{x2:.1}", y2: "{y2:.1}" }
                                    if h % 6 == 0 { text { class: "hl", x: "{tx:.1}", y: "{ty:.1}", style: "transform-origin:{tx:.1}px {ty:.1}px;transform:rotate({-rot:.2}deg)", "{h}" } }
                                }
                            }
                        }
                    }
                    for (i, c) in st.cities.iter().enumerate() {
                        {
                            let (lm, _) = st.local(i);
                            // Relative to home: the home city sits at the top.
                            let rel = (lm - hm + 720).rem_euclid(1440);
                            let (x, y) = DialModel::point(rel, cx, cy, r);
                            let at = st.at_work(i);
                            let is_home = i == home_i;
                            rsx! {
                                g { class: cls(&["city", if is_home { "home" } else { "" }, if at { "work" } else { "" }]),
                                    circle { cx: "{x:.1}", cy: "{y:.1}", r: if is_home { "11" } else { "4.5" } }
                                    if is_home { text { class: "code", x: "{x:.1}", y: "{y + 3.5:.1}", "{c.code}" } }
                                }
                            }
                        }
                    }
                }
                div { class: "readout", "aria-hidden": "true",
                    span { class: "sun", Icon { name: if day { "sun" } else { "moon" }, size: 14.0 } }
                    span { class: "dig",
                        for (i, ch) in digits.chars().enumerate() { span { key: "{i}-{ch}", class: "dg", "{ch}" } }
                    }
                    span { class: "wd", "{st.home_weekday()} · {st.cities[home_i].name}" }
                }
            }
            div { class: "cities", role: "group", "aria-label": "Cities",
                for (i, c) in st.cities.iter().enumerate() {
                    button { class: cls(&["cr", if i == home_i { "home" } else { "" }, if st.at_work(i) { "work" } else { "" }, if st.quiet_now(i) { "quiet" } else { "" }]), r#type: "button",
                        "aria-pressed": "{i == home_i}",
                        onclick: move |_| { let r = m.write().set_home(i).map(str::to_string); if let Some(id) = r { on_home_change.call(id) } },
                        span { class: "dot", "aria-hidden": "true" }
                        span { class: "nm", "{c.name}" }
                        if let Some(n) = st.day_note(i) { span { class: "dn", "{n}" } }
                        span { class: "tm", "{st.time_label(i)}" }
                        span { class: "st", if st.quiet_now(i) { "Quiet" } else if st.at_work(i) { "At work" } else { "Off" } }
                    }
                }
                if overlap {
                    button { class: "oxyn-btn ghost sm best", r#type: "button",
                        onclick: move |_| { let (v, _) = m.peek().best_overlap(); m.write().value = v; on_change.call(v) },
                        Icon { name: "spark", size: 12.0 } "Best overlap"
                    }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.valuetext()}" }
        }
    }
}

/// Date reel: iOS-style wheels for day, hour, minute and AM / PM (or month, day, year). Each reel is
/// a 3D cylinder; drag, flick or scroll it and it glides onto an entry; drags past the ends
/// stretch; Tab moves between reels, arrows turn, Page keys turn by a large step, typing jumps
/// (`4`, `45`, `p`, `ma`); quick picks spin every reel; a confirm button shows pending and success
/// in place. Oxyn plus: a natural-language field and a routine preset.
#[component]
pub fn DateReel(
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
