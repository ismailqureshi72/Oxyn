//! Storage family (variants 2) demos: usage drawn as dot matrices – a capacity meter with a
//! category grid, a cleanup of unused skills, a waffle and a fill forecast.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{StorageCleanup, StorageForecast, StorageMeter, StorageWaffle};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "storage";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn cap(v: &Vals) -> f64 {
    match v.s("cap") {
        "256 GB" => 256.0,
        "1 TB" => 1000.0,
        _ => 512.0,
    }
}

// ---------- Storage meter ----------

fn storage_meter(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("cap"),
        v.s("cols"),
        v.s("rows"),
        v.s("pick"),
        v.s("actions")
    );
    let pick = match v.s("pick") {
        "caches" => Some(7),
        "photos" => Some(2),
        _ => None,
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:420px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            StorageMeter { cap: cap(v), cols: v.u("cols"), rows: v.u("rows"), default_pick: pick, actions: v.b("actions"),
                on_clear: move |(k, gb): (String, f64)| msg.set(format!("onClear({k}, {gb:.1} GB)")) }
            {status(msg)}
        }
    }
}

fn storage_meter_code(v: &Vals) -> String {
    Code::new("StorageMeter")
        .raw("cats", "Some(scan())")
        .raw("cap", format!("{:.1}", cap(v)))
        .s("cols", v.s("cols"), "48")
        .raw("on_clear", "move |(key, gb)| cleared(key, gb)")
        .done()
}

fn storage_meter_demo() -> Demo {
    Demo {
        controls: vec![
            sel("cap", "Disk", &["512 GB", "256 GB", "1 TB"], "512 GB"),
            rng("cols", "Dot columns", 24.0, 64.0, 4.0, "48"),
            rng("rows", "Dot rows", 2.0, 6.0, 1.0, "4"),
            sel("pick", "Picked", &["none", "caches", "photos"], "none"),
            tog("actions", "Rescan and clear", true),
        ],
        variants: vec![
            var(
                "default",
                "A bar of dots, one colour block per category, over a grid; the hint offers the biggest thing you can clear.",
                &[],
            ),
            var(
                "caches picked",
                "Caches picked: the other dots dim and the hint offers to clear them.",
                &[("pick", "caches")],
            ),
            var(
                "small disk",
                "A 256 GB disk in a finer bar: the same files fill it past half.",
                &[("cap", "256 GB"), ("cols", "64"), ("rows", "3")],
            ),
            var(
                "read only",
                "No Rescan or Clear: a meter to read.",
                &[("actions", "false"), ("pick", "photos")],
            ),
        ],
        render: storage_meter,
        code: storage_meter_code,
        keyboard: vec![
            ("Tab", "Rescan, each category, the hint's action"),
            (
                "Enter / Space on a category",
                "Keeps its detail in the hint",
            ),
        ],
        a11y: vec![
            "The dot bar is an image labelled with used and total; each category is a button whose label and size are text.",
            "The hint is a polite live region: scanning, clearing and what was freed are all said.",
        ],
        responsive: vec!["420 px card; the grid drops to two columns under 480 px."],
        height: 420,
        live: false,
        align: Align::Center,
        ..mk(
            "storage-meter",
            "Storage meter",
            CAT,
            "Disk use as a capacity bar of dots, one colour block per category, over a category grid; point or click for details, Rescan, and clear caches or the Trash from the hint.",
            "StorageMeter",
            "StorageMeterEl",
            "storage-meter",
            "parity/storagefid.rs",
        )
    }
}

// ---------- Storage cleanup ----------

fn storage_cleanup(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = v.string("days");
    let days = match v.s("days") {
        "90 days" => 90,
        "6 months" => 180,
        _ => 30,
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:440px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            StorageCleanup { default_days: days,
                on_trash: move |n: Vec<String>| msg.set(format!("onTrash({} items)", n.len())) }
            {status(msg)}
        }
    }
}

fn storage_cleanup_code(_: &Vals) -> String {
    Code::new("StorageCleanup")
        .raw("items", "Some(skills())")
        .raw("on_trash", "move |names| trash(names)")
        .done()
}

fn storage_cleanup_demo() -> Demo {
    Demo {
        controls: vec![sel(
            "days",
            "Not used in",
            &["30 days", "90 days", "6 months"],
            "30 days",
        )],
        variants: vec![
            var(
                "30 days",
                "Nine items idle for a month, all picked; the dots show the tokens they cost every session.",
                &[],
            ),
            var(
                "6 months",
                "Only what sat unused for half a year.",
                &[("days", "6 months")],
            ),
            var("90 days", "A quarter's window.", &[("days", "90 days")]),
        ],
        render: storage_cleanup,
        code: storage_cleanup_code,
        keyboard: vec![
            ("Tab", "Window, Select all, each row, Move to Trash, Undo"),
            ("Space", "Ticks a row"),
        ],
        a11y: vec![
            "The window is a radiogroup; rows and Select all are checkboxes (role checkbox, aria-checked).",
            "The dot row is an image labelled with the tokens freed; moving and undoing are said in a status line.",
        ],
        responsive: vec![
            "440 px card; under 480 px the place tag hides and the list scrolls inside the card.",
        ],
        height: 560,
        live: false,
        align: Align::Center,
        ..mk(
            "storage-cleanup",
            "Storage cleanup",
            CAT,
            "Skills and commands your agents load in every session, filtered by how long they sat unused; tick some, watch the tokens they free as dots, move them to the Trash and undo.",
            "StorageCleanup",
            "StorageCleanupEl",
            "storage-cleanup",
            "parity/storagefid.rs",
        )
    }
}

// ---------- Storage waffle ----------

fn storage_waffle(v: &Vals, _now: ClockMs) -> Element {
    let key = format!("{}{}{}", v.s("lens"), v.s("mono"), v.s("cap"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:440px;max-width:100%",
            StorageWaffle { default_lens: v.string("lens"), mono: v.b("mono"), cap: cap(v) }
        }
    }
}

fn storage_waffle_code(v: &Vals) -> String {
    Code::new("StorageWaffle")
        .raw("cats", "Some(scan())")
        .s("default_lens", v.s("lens"), "kind")
        .b("mono", v.b("mono"))
        .done()
}

fn storage_waffle_demo() -> Demo {
    Demo {
        controls: vec![
            sel("lens", "Lens", &["kind", "clear"], "kind"),
            sel("cap", "Disk", &["512 GB", "256 GB", "1 TB"], "512 GB"),
            tog("mono", "Monochrome", false),
        ],
        variants: vec![
            var(
                "by kind",
                "200 dots, half a percent each; point at a category to light only its dots.",
                &[],
            ),
            var(
                "win back",
                "The Win back lens: only caches and the Trash stay lit, with the space you would get back.",
                &[("lens", "clear")],
            ),
            var(
                "mono 1 TB",
                "A 1 TB disk in greys.",
                &[("mono", "true"), ("cap", "1 TB")],
            ),
        ],
        render: storage_waffle,
        code: storage_waffle_code,
        keyboard: vec![
            ("Tab", "Lens, then each category"),
            ("Focus on a category", "Lights its dots"),
        ],
        a11y: vec![
            "The waffle is an image labelled with the headline; each legend row says its size and its number of dots.",
            "The lens is a radiogroup.",
        ],
        responsive: vec!["440 px card; under 480 px the legend moves under the waffle."],
        height: 380,
        live: false,
        align: Align::Center,
        ..mk(
            "storage-waffle",
            "Storage waffle",
            CAT,
            "An Oxyn original: the disk as 200 dots, half a percent each; a Win back lens keeps only what can be cleared lit.",
            "StorageWaffle",
            "StorageWaffleEl",
            "storage-waffle",
            "parity/storagefid.rs",
        )
    }
}

// ---------- Storage forecast ----------

fn storage_forecast(v: &Vals, _now: ClockMs) -> Element {
    let key = format!("{}{}{}", v.s("pace"), v.s("rows"), v.s("ahead"));
    let pace = match v.s("pace") {
        "own" => None,
        "slow" => Some(1.0),
        _ => Some(6.0),
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:460px;max-width:100%",
            StorageForecast { pace, rows: v.u("rows"), ahead: v.u("ahead") }
        }
    }
}

fn storage_forecast_code(v: &Vals) -> String {
    Code::new("StorageForecast")
        .raw("history", "Some(daily_used())")
        .s("rows", v.s("rows"), "12")
        .done()
}

fn storage_forecast_demo() -> Demo {
    Demo {
        controls: vec![
            sel("pace", "Pace", &["own", "slow", "fast"], "own"),
            rng("rows", "Dots per column", 8.0, 16.0, 1.0, "12"),
            rng("ahead", "Weeks ahead", 4.0, 12.0, 1.0, "8"),
        ],
        variants: vec![
            var(
                "default",
                "Two weeks of dot columns, then ghost columns a week apart at the same pace; the top rows turn red.",
                &[],
            ),
            var(
                "fast",
                "At 6 GB a day the ghosts reach the red rows within weeks.",
                &[("pace", "fast")],
            ),
            var(
                "slow",
                "At 1 GB a day, with finer columns.",
                &[("pace", "slow"), ("rows", "16")],
            ),
        ],
        render: storage_forecast,
        code: storage_forecast_code,
        keyboard: vec![("Pointer over a column", "Reads that day's use")],
        a11y: vec![
            "The chart is an image labelled with the used space and the days until full.",
            "The reading in the corner is a live region; the summary line is plain text.",
        ],
        responsive: vec!["460 px card; columns share the width and dots stay round."],
        height: 320,
        live: false,
        align: Align::Center,
        ..mk(
            "storage-forecast",
            "Storage forecast",
            CAT,
            "An Oxyn original: a fortnight of dot columns lit to each day's use, then ghost columns at the current pace up to the day the disk is full.",
            "StorageForecast",
            "StorageForecastEl",
            "storage-forecast",
            "parity/storagefid.rs",
        )
    }
}

/// Every Storage (variants 2) demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        storage_meter_demo(),
        storage_cleanup_demo(),
        storage_waffle_demo(),
        storage_forecast_demo(),
    ]
}
