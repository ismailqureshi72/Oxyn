//! Thinking Selector family (variants 1) demos: one control – how hard the model thinks – done
//! many ways, each its own page in the Thinking Selector group.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::thinkfid::{effort_scale, effort_scale_short, level_index},
};
use oxyn_dioxus::{EffortList, EffortRuler, ReasoningPanel};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "thinking-selector";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn width(v: &Vals) -> &'static str {
    if v.s("width") == "narrow" {
        "320px"
    } else {
        "440px"
    }
}

const FIVE: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

// ---------- Ruler card ----------

fn effort_ruler(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("level"), v.s("ticks"), v.s("width"));
    let start = level_index(&effort_scale(), v.s("level"), 4);
    let ticks = v.f("ticks").max(12.0) as usize;
    let w = width(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:{w};max-width:100%;display:flex;flex-direction:column;gap:10px",
            EffortRuler { default_value: Some(start), ticks,
                on_change: move |i: usize| msg.set(format!("onChange({})", effort_scale()[i].label)) }
            {status(msg)}
        }
    }
}

fn effort_ruler_code(v: &Vals) -> String {
    let start = level_index(&effort_scale(), v.s("level"), 4);
    let label = &effort_scale()[start].label;
    Code::new("EffortRuler")
        .pre(format!(
            "let mut effort = use_signal(|| {start}); // {label}"
        ))
        .raw("value", "Some(effort())")
        .s("ticks", v.s("ticks"), "44")
        .raw("on_change", "move |i| effort.set(i)")
        .done()
}

fn effort_ruler_demo() -> Demo {
    Demo {
        controls: vec![
            sel("level", "Level", &FIVE, "max"),
            rng("ticks", "Ticks", 16.0, 72.0, 4.0, "44"),
            sel("width", "Width", &["wide", "narrow"], "wide"),
        ],
        variants: vec![
            var(
                "max",
                "At the top level the name turns red and every tick behind the knob carries the red, strongest next to it.",
                &[],
            ),
            var(
                "high",
                "Mid-scale: the ticks past the knob stay grey; reset is off because High is where it returns.",
                &[("level", "high")],
            ),
            var(
                "low",
                "The first level: only the first tick is lit.",
                &[("level", "low")],
            ),
            var(
                "dense",
                "72 ticks for a finer ruler; the knob still snaps to the five levels.",
                &[("ticks", "72")],
            ),
            var(
                "narrow",
                "320 px: the line wraps and the ruler keeps its ticks.",
                &[("width", "narrow")],
            ),
        ],
        render: effort_ruler,
        code: effort_ruler_code,
        keyboard: vec![
            ("← / →", "One level down or up"),
            ("Home / End", "Lowest or highest level"),
            ("Tab", "Moves to the reset button"),
        ],
        a11y: vec![
            "The ruler is a slider: its value text is the level's name, so a screen reader says “High”, not “2”.",
            "The status line says the new level and what it does after every change.",
            "Reset is a real button named after the level it returns to; it is disabled when already there.",
        ],
        responsive: vec![
            "440 px card up to its container; under 340 px the heading tightens and the ruler's side padding drops.",
            "Ticks are placed by percentage, so the ruler fits any width.",
        ],
        height: 300,
        live: false,
        align: Align::Center,
        ..mk(
            "effort-ruler",
            "Ruler card",
            CAT,
            "A card with the effort's name and what it does over a dense tick ruler: the ticks behind the capsule knob tint toward it, the knob snaps to the levels, and reset goes back to the default.",
            "EffortRuler",
            "EffortRulerEl",
            "effort-ruler",
            "parity/rulerfid.rs",
        )
    }
}

// ---------- Reasoning panel ----------

fn reasoning_panel(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("level"), v.s("fast"), v.s("favorite"));
    let start = level_index(&effort_scale(), v.s("level"), 2);
    let (fast, fav) = (v.b("fast"), v.b("favorite"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ReasoningPanel { default_value: start, default_fast: fast, default_favorite: fav,
                on_change: move |i: usize| msg.set(format!("onChange({})", effort_scale()[i].label)),
                on_fast_change: move |on: bool| msg.set(format!("onFastChange({on})")),
                on_favorite_change: move |on: bool| msg.set(format!("onFavoriteChange({on})")) }
            {status(msg)}
        }
    }
}

fn reasoning_panel_code(v: &Vals) -> String {
    let start = level_index(&effort_scale(), v.s("level"), 2);
    Code::new("ReasoningPanel")
        .raw(
            "model",
            "Some(PanelModel { name: \"Atlas 6.1\".into(), context: 1_100_000, has_fast: true })",
        )
        .raw(
            "value",
            format!("Some(effort())  /* {} */", effort_scale()[start].label),
        )
        .raw("fast", "Some(fast())")
        .raw("on_change", "move |i| effort.set(i)")
        .raw("on_fast_change", "move |on| fast.set(on)")
        .done()
}

fn reasoning_panel_demo() -> Demo {
    Demo {
        controls: vec![
            sel("level", "Level", &FIVE, "high"),
            tog("fast", "Fast mode", false),
            tog("favorite", "Favorite", false),
        ],
        variants: vec![
            var(
                "high",
                "Mid-scale: a grey fill up to the thumb and notches marking the levels still ahead. Press R to step, F for Fast mode.",
                &[],
            ),
            var(
                "max",
                "At Max the bar fills with red dots, like a meter running hot, and the level name turns red.",
                &[("level", "max")],
            ),
            var(
                "fast",
                "Fast mode on and the model in favorites.",
                &[("fast", "true"), ("favorite", "true"), ("level", "medium")],
            ),
            var(
                "low",
                "The first level: no fill, every notch ahead.",
                &[("level", "low")],
            ),
        ],
        render: reasoning_panel,
        code: reasoning_panel_code,
        keyboard: vec![
            (
                "R",
                "Next reasoning level, wrapping after Max (anywhere in the panel)",
            ),
            ("F", "Fast mode on or off (anywhere in the panel)"),
            (
                "← / → · Home / End",
                "On the slider: one level, or the ends",
            ),
            ("Space / Enter", "On the switch or Favorite: flips it"),
        ],
        a11y: vec![
            "The bar is a slider named Reasoning with the level's name as its value text and R as its key shortcut.",
            "Fast mode is a switch (aria-checked); Favorite is a toggle button (aria-pressed).",
            "Single-letter keys skip when Ctrl, Cmd or Alt is held, so browser shortcuts still work.",
        ],
        responsive: vec![
            "340 px panel, shrinking with its container; rows keep label left and value right.",
        ],
        height: 340,
        live: false,
        align: Align::Center,
        ..mk(
            "reasoning-panel",
            "Reasoning panel",
            CAT,
            "A model's details with a notched reasoning slider (R steps it; Max fills the bar with red dots), a Fast mode switch (F) and a Favorite button.",
            "ReasoningPanel",
            "ReasoningPanelEl",
            "reasoning-panel",
            "parity/reasonfid.rs",
        )
    }
}

// ---------- Effort list ----------

fn effort_list(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("level"), v.s("open"));
    let start = level_index(&effort_scale_short(), v.s("level"), 3);
    let open = v.b("open");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "min-height:250px;display:flex;flex-direction:column;justify-content:flex-end;gap:10px",
            EffortList { default_value: Some(start), default_open: open,
                on_change: move |i: usize| msg.set(format!("onChange({})", effort_scale_short()[i].label)) }
            {status(msg)}
        }
    }
}

fn effort_list_code(_: &Vals) -> String {
    Code::new("EffortList")
        .raw("levels", "Some(effort_scale_short())")
        .raw("value", "Some(effort())")
        .raw("on_change", "move |i| effort.set(i)")
        .done()
}

fn effort_list_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "level",
                "Level",
                &["minimal", "low", "medium", "high"],
                "high",
            ),
            tog("open", "Open", true),
        ],
        variants: vec![
            var(
                "open",
                "The list open above its chip; High is checked and highlighted.",
                &[],
            ),
            var(
                "closed",
                "Closed: only the chip, its caret pointing up to where the list opens.",
                &[("open", "false")],
            ),
            var("minimal", "Minimal chosen.", &[("level", "minimal")]),
        ],
        render: effort_list,
        code: effort_list_code,
        keyboard: vec![
            ("↑ / ↓ on the chip", "Opens the list on the chosen level"),
            ("↑ / ↓ · Home / End", "Moves the highlight (wraps)"),
            ("Enter / Space", "Picks the highlighted level and closes"),
            ("Escape", "Closes back to the chip"),
        ],
        a11y: vec![
            "The chip is a menu button (aria-haspopup, aria-expanded) named “Thinking effort: High”.",
            "Levels are menuitemradio rows with aria-checked; the highlight follows aria-activedescendant.",
            "Focus returns to the chip after a pick or Escape.",
        ],
        responsive: vec!["300 px list, never wider than the viewport minus 32 px."],
        height: 320,
        live: false,
        align: Align::Center,
        ..mk(
            "effort-list",
            "Effort list",
            CAT,
            "A chip with the current effort that opens a plain list of levels above it, the chosen one checked.",
            "EffortList",
            "EffortListEl",
            "effort-list",
            "parity/efflistfid.rs",
        )
    }
}

/// Every Thinking Selector demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        effort_ruler_demo(),
        reasoning_panel_demo(),
        effort_list_demo(),
    ]
}
