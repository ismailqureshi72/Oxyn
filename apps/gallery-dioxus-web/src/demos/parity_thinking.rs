//! Thinking Selector family (variants 1) demos: one control – how hard the model thinks – done
//! many ways, each its own page in the Thinking Selector group.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::thinkfid::{effort_scale, level_index},
};
use oxyn_dioxus::EffortRuler;

use super::{Code, mk, rng, sel, var};
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

/// Every Thinking Selector demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![effort_ruler_demo()]
}
