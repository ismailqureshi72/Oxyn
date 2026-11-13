//! Pickers (plan #25): calendar, date picker, date range picker, time picker, time dial, date reel.

use dioxus::prelude::*;
use oxyn_core::{ClockMs, primitives::dial::City};
use oxyn_dioxus::{Calendar, DatePicker, DateRangePicker, DateReel, TimeDial, TimePicker};

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

fn date_picker(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w320 oxd-room-menu-lg",
            DatePicker {
                label: "Due date",
                value: Some("2026-11-20".to_string()),
                today: TODAY,
                format: v.string("format"),
                blocked: v.string("blocked"),
                show_today: v.b("show_today"),
                natural: v.b("natural"),
                presets: v.b("presets"),
                open: v.b("open"),
                disabled: v.b("disabled"),
                description: Some("The agent works toward this date".to_string()),
            }
        }
    }
}

fn date_picker_code(v: &Vals) -> String {
    Code::new("DatePicker")
        .raw("label", q("Due date"))
        .raw("value", "due()")
        .s("format", v.s("format"), "medium")
        .s("blocked", v.s("blocked"), "none")
        .b("show_today", v.b("show_today"))
        .b("natural", v.b("natural"))
        .b("presets", v.b("presets"))
        .raw("on_change", "move |d: Option<String>| due.set(d)")
        .done()
}

fn range(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-room-menu-lg",
            DateRangePicker {
                label: "Runs",
                start: Some("2026-10-15".to_string()),
                end: Some(TODAY.to_string()),
                today: TODAY,
                compare: v.b("compare"),
                default_compare: v.string("default_compare"),
                week_starts_on: v.u("week_starts_on") as u8,
                months: v.u("months") as u8,
                default_open: v.b("open"),
            }
        }
    }
}

fn range_code(v: &Vals) -> String {
    Code::new("DateRangePicker")
        .raw("label", q("Runs"))
        .raw("start", "Some(from())")
        .raw("end", "Some(to())")
        .b("compare", v.b("compare"))
        .s("default_compare", v.s("default_compare"), "previous")
        .raw("week_starts_on", v.s("week_starts_on").to_string())
        .raw("months", v.s("months").to_string())
        .raw("on_change", "move |(a, b, cmp)| set_range(a, b, cmp)")
        .done()
}

fn time_picker(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w320 oxd-room-menu-lg",
            TimePicker {
                label: "Run every day at",
                value: Some("09:30".to_string()),
                minute_step: v.u("minute_step") as u32,
                format: v.string("format"),
                window: v.b("window").then(|| "08:00-18:00".to_string()),
                zone: v.b("zone").then(|| "PKT".to_string()),
                open: v.b("open"),
                disabled: v.b("disabled"),
                description: Some("Local time".to_string()),
            }
        }
    }
}

fn time_picker_code(v: &Vals) -> String {
    Code::new("TimePicker")
        .raw("label", q("Run every day at"))
        .raw("value", "Some(at())")
        .raw("minute_step", v.s("minute_step").to_string())
        .s("format", v.s("format"), "12h")
        .raw("on_change", "move |hhmm: String| at.set(hhmm)")
        .done()
}

fn cities() -> Vec<City> {
    vec![
        City::new("khi", "Karachi", "KHI", 300),
        City::new("lon", "London", "LON", 0),
        City::new("nyc", "New York", "NYC", -300),
        City::new("sfo", "San Francisco", "SF", -480),
        City::new("syd", "Sydney", "SYD", 660),
    ]
}

fn time_dial(v: &Vals, _: ClockMs) -> Element {
    let mut c = cities();
    c.truncate(v.u("cities").clamp(2, 5));
    rsx! {
        div { class: "oxd-w560",
            TimeDial {
                cities: c,
                value: 600,
                hour_cycle: v.string("hour_cycle"),
                working_hours: (540, 1080),
                quiet_hours: v.b("quiet").then_some((1320, 420)),
                overlap: v.b("overlap"),
                min: v.b("bounds").then_some(240),
                max: v.b("bounds").then_some(1200),
            }
        }
    }
}

fn time_dial_code(v: &Vals) -> String {
    Code::new("TimeDial")
        .pre("let cities = vec![City::new(\"khi\", \"Karachi\", \"KHI\", 300), City::new(\"lon\", \"London\", \"LON\", 0)];")
        .raw("cities", "cities")
        .raw("value", "utc_minutes()")
        .s("hour_cycle", v.s("hour_cycle"), "12")
        .b("overlap", v.b("overlap"))
        .raw("on_change", "move |m: i32| utc_minutes.set(m)")
        .done()
}

fn date_reel(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        DateReel {
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
    Code::new("DateReel")
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
    vec![
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
            slug: "date-picker",
            name: "Date picker",
            category: "pickers",
            summary: "A field that opens the calendar in a popover; the changed parts of the date roll, and the popover closes just after a pick.",
            dioxus: "DatePicker",
            gpui: "DatePickerEl",
            spec: "date-picker",
            source: "primitives/pickers.rs",
            controls: vec![
                tog("open", "Open", true),
                sel(
                    "format",
                    "Format",
                    &[
                        "medium",
                        "long",
                        "weekday",
                        "day-first",
                        "month-first",
                        "iso",
                    ],
                    "medium",
                ),
                sel("blocked", "Blocked", &["none", "weekends"], "none"),
                tog("show_today", "Today button", true),
                tog("natural", "Type in words", false),
                tog("presets", "Relative presets", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("open", "Calendar open under the field.", &[]),
                var("closed", "Just the field.", &[("open", "false")]),
                var(
                    "words",
                    "“next tue”, “in 2 weeks” (Oxyn plus).",
                    &[("natural", "true")],
                ),
                var(
                    "presets",
                    "Today, Tomorrow, Next week (Oxyn plus).",
                    &[("presets", "true"), ("format", "weekday")],
                ),
            ],
            render: date_picker,
            code: date_picker_code,
            keyboard: vec![
                (
                    "Enter / Space / ↓",
                    "Open and focus the picked or current day",
                ),
                ("Esc", "Close and return to the field"),
                (
                    "Calendar keys",
                    "Arrows, Home, End, Page Up / Down as in the calendar",
                ),
            ],
            a11y: vec![
                "The trigger has aria-haspopup=dialog and aria-expanded; the popover is a dialog named “Due date calendar”.",
                "The value lives in a visually hidden span; the rolling parts are aria-hidden.",
            ],
            responsive: vec![
                "The popover is min(320 px, 100vw − 32 px), left-aligned to the field.",
                "Below 360 px it stretches to both field edges.",
            ],
            height: 460,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "date-range-picker",
            name: "Date range picker",
            category: "pickers",
            summary: "Presets and two months with a hover preview after the first click, comparison with the previous period or year, Apply and Cancel.",
            dioxus: "DateRangePicker",
            gpui: "RangePickerEl",
            spec: "date-range-picker",
            source: "primitives/pickers.rs",
            controls: vec![
                tog("open", "Open", true),
                tog("compare", "Compare", true),
                sel(
                    "default_compare",
                    "Compare with",
                    &["off", "previous", "year"],
                    "previous",
                ),
                sel("months", "Months", &["1", "2"], "2"),
                sel("week_starts_on", "Week starts", &["0", "1"], "1"),
            ],
            variants: vec![
                var("two-months", "Presets, two months and the comparison.", &[]),
                var(
                    "one-month",
                    "One month, no comparison.",
                    &[("months", "1"), ("compare", "false")],
                ),
                var(
                    "year",
                    "Compared with last year.",
                    &[("default_compare", "year")],
                ),
                var(
                    "closed",
                    "The trigger with its range.",
                    &[("open", "false")],
                ),
            ],
            render: range,
            code: range_code,
            keyboard: vec![
                (
                    "Enter / Space",
                    "Open; in the grid pick a start, then an end",
                ),
                (
                    "Arrow keys",
                    "Move by day or week; once a start is picked the preview follows",
                ),
                ("Home / End", "Start / end of the week"),
                ("Page Up / Page Down", "A month (Shift: a year)"),
                ("Esc", "Close without applying"),
            ],
            a11y: vec![
                "The trigger opens a dialog and its name includes the current range.",
                "Each month is a grid; days have full dates, aria-selected inside the range and aria-current for today.",
                "A polite region announces the start, then the range and its length.",
            ],
            responsive: vec![
                "Below 600 px the presets become a scrolling row of chips and one month shows; the footer stacks.",
                "The panel is as wide as the screen allows, up to 652 px.",
            ],
            height: 520,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "time-picker",
            name: "Time picker",
            category: "pickers",
            summary: "A time field with a list at a fixed step; a later time rises in, an earlier one drops, and the list opens on the chosen time.",
            dioxus: "TimePicker",
            gpui: "TimePickerEl",
            spec: "time-picker",
            source: "primitives/pickers.rs",
            controls: vec![
                tog("open", "Open", true),
                sel("minute_step", "Step", &["5", "10", "15", "30"], "15"),
                sel("format", "Format", &["12h", "24h"], "12h"),
                tog("window", "Working hours only", false),
                tog("zone", "Zone tag", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("open", "Every 15 minutes.", &[]),
                var(
                    "24h",
                    "24-hour, every 30 minutes.",
                    &[("format", "24h"), ("minute_step", "30")],
                ),
                var(
                    "window",
                    "Working hours with a zone tag (Oxyn plus).",
                    &[("window", "true"), ("zone", "true")],
                ),
            ],
            render: time_picker,
            code: time_picker_code,
            keyboard: vec![
                ("Enter / Space", "Open, or pick the active time"),
                ("↓ / ↑", "Open, then move through times (wraps)"),
                ("Esc", "Close"),
            ],
            a11y: vec![
                "Trigger: aria-haspopup=listbox, aria-expanded, aria-controls, labelled by the label and a hidden value.",
                "Options carry aria-selected; the list scrolls itself, never the page.",
            ],
            responsive: vec!["The list spans the field and caps at 250 px."],
            height: 400,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "time-dial",
            name: "Time dial",
            category: "pickers",
            summary: "Spin a 24-hour dial to pick a time and see which team cities are at work; the readout rolls and the sun turns into the moon.",
            dioxus: "TimeDial",
            gpui: "TimeDialEl",
            spec: "time-dial",
            source: "primitives/pickers.rs",
            controls: vec![
                rng("cities", "Cities", 2.0, 5.0, 1.0, "4"),
                sel("hour_cycle", "Clock", &["12", "24"], "12"),
                tog("bounds", "Bounds 4:00–20:00 UTC", false),
                tog("quiet", "Quiet hours", false),
                tog("overlap", "Best overlap", true),
            ],
            variants: vec![
                var("team", "Four cities, home in Karachi.", &[]),
                var(
                    "24h",
                    "24-hour readout, five cities.",
                    &[("hour_cycle", "24"), ("cities", "5")],
                ),
                var(
                    "quiet",
                    "Quiet hours band (Oxyn plus).",
                    &[("quiet", "true")],
                ),
                var(
                    "bounded",
                    "Rubber-bands past the bounds.",
                    &[("bounds", "true")],
                ),
            ],
            render: time_dial,
            code: time_dial_code,
            keyboard: vec![
                ("→ / ↑", "A quarter hour forward"),
                ("← / ↓", "A quarter hour back"),
                ("Page Up / Page Down", "An hour"),
                (
                    "Home / End",
                    "Start / last quarter hour of the home city's day",
                ),
            ],
            a11y: vec![
                "The dial is a focusable role=slider with aria-valuenow in home-city minutes and a spoken aria-valuetext.",
                "Cities are buttons with aria-pressed for the home city; changes are announced politely.",
                "The SVG face and the readout are aria-hidden.",
            ],
            responsive: vec![
                "From 600 px the dial and city list sit side by side; below that they stack.",
                "The dial is min(100%, 256 px) with touch-action none.",
            ],
            height: 340,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "date-reel",
            name: "Date reel",
            category: "pickers",
            summary: "iOS-style wheels for day and time: drag, flick or scroll and it glides onto an entry; quick picks spin every reel.",
            dioxus: "DateReel",
            gpui: "DateReelEl",
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
    ]
}
