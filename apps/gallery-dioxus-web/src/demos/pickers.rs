//! Pickers (plan #25): calendar, date picker, date range picker, time picker, time dial, date wheel.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{ClockMs, primitives::dial::City};
use oxyn_dioxus::{Calendar, DateWheel};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{DatePicker, DateRangePicker, TimeDial, TimePicker};

use super::{Code, q, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const TODAY: &str = "2026-11-13";

fn activity() -> Vec<(String, u8)> {
    // Runs per day across October and November (invented): busier mid-week.
    (1..=30)
        .map(|d| (format!("2026-11-{d:02}"), ((d * 7 + 3) % 5) as u8))
        .filter(|(day, _)| day.as_str() <= TODAY)
        .collect()
}

fn calendar(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        Calendar {
            value: Some("2026-11-18".to_string()),
            today: TODAY,
            min: v.b("min").then(|| "2026-11-05".to_string()),
            max: v.b("max").then(|| "2026-12-20".to_string()),
            blocked: v.string("blocked"),
            week_starts_on: v.u("week_starts_on") as u8,
            show_today: v.b("show_today"),
            week_numbers: v.b("week_numbers"),
            activity: if v.b("activity") { activity() } else { vec![] },
        }
    }
}

fn calendar_code(v: &Vals) -> String {
    Code::new("Calendar")
        .raw("value", "Some(day())")
        .raw("today", "today_iso()")
        .s("blocked", v.s("blocked"), "none")
        .raw("week_starts_on", v.s("week_starts_on").to_string())
        .b("show_today", v.b("show_today"))
        .b("week_numbers", v.b("week_numbers"))
        .raw("on_change", "move |iso: String| day.set(iso)")
        .done()
}

fn date_reel(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        DateWheel {
            mode: v.string("mode"),
            date: "2026-09-24",
            time: "09:30",
            today: "2026-09-24",
            minute_step: v.u("minute_step") as u32,
            hour_cycle: v.string("hour_cycle"),
            title: Some("Send later".to_string()),
            presets: v.b("presets"),
            routine: v.b("routine"),
            natural: v.b("natural"),
            confirm: v.b("confirm").then(|| "Schedule".to_string()),
            speed: v.f("speed") as f32,
        }
    }
}

fn date_reel_code(v: &Vals) -> String {
    Code::new("DateWheel")
        .s("mode", v.s("mode"), "datetime")
        .raw("minute_step", v.s("minute_step").to_string())
        .s("hour_cycle", v.s("hour_cycle"), "12")
        .raw("title", "Some(\"Send later\".into())")
        .b("routine", v.b("routine"))
        .b("natural", v.b("natural"))
        .raw(
            "confirm",
            if v.b("confirm") {
                "Some(\"Schedule\".into())"
            } else {
                "None"
            },
        )
        .raw("on_change", "move |(day, time)| when.set((day, time))")
        .done()
}

/// Demos.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            slug: "calendar",
            name: "Calendar",
            category: "pickers",
            summary: "A compact month view for one date: months slide the way you travel, the selection moves between days and arrows skip blocked days.",
            dioxus: "Calendar",
            gpui: "CalendarEl",
            spec: "calendar",
            source: "primitives/pickers.rs",
            controls: vec![
                sel("blocked", "Blocked", &["none", "weekends"], "none"),
                sel("week_starts_on", "Week starts", &["0", "1"], "0"),
                tog("show_today", "Today button", true),
                tog("min", "Min (Nov 5)", false),
                tog("max", "Max (Dec 20)", false),
                tog("week_numbers", "Week numbers", false),
                tog("activity", "Run history", false),
            ],
            variants: vec![
                var("default", "One date with a Today button.", &[]),
                var(
                    "weekdays",
                    "Weekends blocked, Monday first.",
                    &[("blocked", "weekends"), ("week_starts_on", "1")],
                ),
                var(
                    "bounded",
                    "Min and max.",
                    &[("min", "true"), ("max", "true")],
                ),
                var(
                    "heat",
                    "Run history per day (Oxyn plus).",
                    &[("activity", "true"), ("week_numbers", "true")],
                ),
            ],
            render: calendar,
            code: calendar_code,
            keyboard: vec![
                (
                    "Arrow keys",
                    "One day or one week, skipping blocked days and crossing months",
                ),
                ("Home / End", "Start / end of the week"),
                ("Page Up / Page Down", "A month (Shift: a year)"),
                ("Enter / Space", "Pick the focused day"),
            ],
            a11y: vec![
                "Days sit in a role=grid; each day is a gridcell button with its full date as its name, aria-selected and aria-current=date for today.",
                "One tab stop (aria-activedescendant); the month title is announced when it changes.",
                "Pagers are labelled and aria-disabled at min or max.",
            ],
            responsive: vec![
                "min(100%, 328 px) wide; days are square cells that scale.",
                "Below 260 px the title takes its own row.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "date-wheel",
            name: "Date wheel",
            category: "pickers",
            summary: "iOS-style wheels for day and time: drag, flick or scroll and it glides onto an entry; quick picks spin every reel.",
            dioxus: "DateWheel",
            gpui: "DateWheelEl",
            spec: "date-reel",
            source: "primitives/pickers.rs",
            controls: vec![
                sel("mode", "Mode", &["datetime", "date", "time"], "datetime"),
                sel("minute_step", "Minute step", &["1", "5", "15", "30"], "5"),
                sel("hour_cycle", "Clock", &["12", "24"], "12"),
                tog("presets", "Quick picks", true),
                tog("routine", "Weekday routine", false),
                tog("natural", "Type in words", false),
                tog("confirm", "Confirm button", true),
                rng("speed", "Speed", 0.5, 2.0, 0.25, "1"),
            ],
            variants: vec![
                var("datetime", "Day, hour, minute and AM / PM.", &[]),
                var(
                    "date",
                    "Month, day and year.",
                    &[("mode", "date"), ("confirm", "false")],
                ),
                var(
                    "time",
                    "Time only, 24-hour.",
                    &[("mode", "time"), ("hour_cycle", "24"), ("presets", "false")],
                ),
                var(
                    "routine",
                    "Routine pick and words (Oxyn plus).",
                    &[("routine", "true"), ("natural", "true")],
                ),
            ],
            render: date_reel,
            code: date_reel_code,
            keyboard: vec![
                ("Tab", "Between reels, the quick picks and the button"),
                ("↑ / ↓", "Turn the focused reel one entry"),
                (
                    "Page Up / Page Down",
                    "A week of days, 3 or 6 hours, 15 minutes, 10 years",
                ),
                ("Home / End", "First / last entry"),
                (
                    "Type",
                    "Jump: 4 for 4 o'clock, 45 for :45, p for PM, ma for March",
                ),
            ],
            a11y: vec![
                "A named group; each reel is a spinbutton (Day, Hour, Minute, AM or PM) with an aria-valuetext such as “Fri, Sep 25”.",
                "The full moment is announced once the reels rest; quick picks use aria-pressed.",
                "Dates that don't exist (February 30) are dimmed and the day reel turns back by itself.",
            ],
            responsive: vec![
                "Fits a 320 px phone; reels size to their widest entry under one lens.",
                "Below 480 px the confirm button goes full width and 44 px tall.",
            ],
            height: 380,
            align: Align::Center,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 6] = [
    "calendar",
    "date-picker",
    "date-range-picker",
    "time-picker",
    "time-dial",
    "date-wheel",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
