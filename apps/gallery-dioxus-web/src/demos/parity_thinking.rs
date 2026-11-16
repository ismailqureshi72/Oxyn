//! Thinking Selector family (variants 1) demos: one control – how hard the model thinks – done
//! many ways, each its own page in the Thinking Selector group.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        effmenufid::effort_models,
        thinkfid::{effort_scale, effort_scale_api, effort_scale_short, level_index},
    },
};
use oxyn_dioxus::{
    ContextEffortMenu, EffortList, EffortPill, EffortRuler, EffortStairs, ModelEffortMenu,
    NozzleSlider, ReasoningPanel,
};

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

// ---------- Effort pill ----------

fn effort_pill(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("level"), v.s("open"));
    let start = (v.s("level") != "none").then(|| level_index(&effort_scale(), v.s("level"), 2));
    let open = v.b("open");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:400px;max-width:100%;min-height:230px;display:flex;flex-direction:column;align-items:flex-end;gap:10px",
            EffortPill { default_value: start, default_open: open,
                on_change: move |i: usize| msg.set(format!("onChange({})", effort_scale()[i].label)),
                on_model: move |_| msg.set("onModel()".into()) }
            {status(msg)}
        }
    }
}

fn effort_pill_code(_: &Vals) -> String {
    Code::new("EffortPill")
        .raw("value", "Some(effort())")
        .s("model", "Atlas 6", "")
        .raw("on_change", "move |i| effort.set(Some(i))")
        .raw("on_model", "move |_| models.open()")
        .done()
}

fn effort_pill_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "level",
                "Level",
                &["none", "low", "medium", "high", "xhigh", "max"],
                "high",
            ),
            tog("open", "Open", true),
        ],
        variants: vec![
            var(
                "open",
                "The popover under the pill: the level, the model it applies to, and a fat slider with a dot per level.",
                &[],
            ),
            var(
                "unset",
                "Nothing chosen yet: the pill reads “Select effort” in grey and the slider rests on the default.",
                &[("level", "none"), ("open", "false")],
            ),
            var("max", "The thumb at the far end.", &[("level", "max")]),
        ],
        render: effort_pill,
        code: effort_pill_code,
        keyboard: vec![
            ("Enter / Space on the pill", "Opens or closes the popover"),
            (
                "← / → · Home / End",
                "On the slider: one level, or the ends",
            ),
            ("Escape", "Closes the popover"),
        ],
        a11y: vec![
            "The pill reports aria-expanded; the popover is a dialog named Thinking effort.",
            "The slider's value text is the level's name; reset is named after the level it returns to.",
        ],
        responsive: vec![
            "380 px popover, capped at the viewport minus 32 px; the slider fills it.",
        ],
        height: 300,
        live: false,
        align: Align::Center,
        ..mk(
            "effort-pill",
            "Effort pill",
            CAT,
            "A “Select effort” pill that opens a popover with the level, the model it applies to, a dotted step slider and reset.",
            "EffortPill",
            "EffortPillEl",
            "effort-pill",
            "parity/effpillfid.rs",
        )
    }
}

// ---------- Context + Effort menu ----------

fn context_effort_menu(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("window"), v.s("effort"), v.s("open"));
    let w = if v.s("window") == "1m" { 1 } else { 0 };
    let e = level_index(&effort_scale_api(), v.s("effort"), 0);
    let open = v.b("open");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "min-height:390px;display:flex;flex-direction:column;gap:10px",
            ContextEffortMenu { default_window: w, default_effort: e, default_open: open,
                on_window_change: move |i: usize| msg.set(format!("onWindowChange({i})")),
                on_effort_change: move |i: usize| msg.set(format!("onEffortChange({})", effort_scale_api()[i].label)) }
            {status(msg)}
        }
    }
}

fn context_effort_menu_code(_: &Vals) -> String {
    Code::new("ContextEffortMenu")
        .s("model", "Atlas 6 Swift", "")
        .raw("on_window_change", "move |i| window.set(i)")
        .raw("on_effort_change", "move |i| effort.set(i)")
        .done()
}

fn context_effort_menu_demo() -> Demo {
    Demo {
        controls: vec![
            sel("window", "Context window", &["512k", "1m"], "512k"),
            sel(
                "effort",
                "Effort",
                &["default", "low", "medium", "high", "xhigh", "max"],
                "default",
            ),
            tog("open", "Open", true),
        ],
        variants: vec![
            var(
                "open",
                "Two groups in one dropdown: the context window, where 1M carries a “Higher usage” tag, and the effort from default to max.",
                &[],
            ),
            var(
                "big",
                "1M window and max effort chosen; max is the one red word.",
                &[("window", "1m"), ("effort", "max")],
            ),
            var(
                "closed",
                "Closed: the chip sums up both choices.",
                &[("open", "false"), ("effort", "high")],
            ),
        ],
        render: context_effort_menu,
        code: context_effort_menu_code,
        keyboard: vec![
            ("↑ / ↓ on the chip", "Opens the menu"),
            ("↑ / ↓ · Home / End", "One list across both groups (wraps)"),
            (
                "Enter / Space",
                "Picks; the menu stays open so both can be set",
            ),
            ("Escape", "Closes back to the chip"),
        ],
        a11y: vec![
            "Each group is role=group named by its heading; rows are menuitemradio with aria-checked.",
            "The chip's name says the model and both choices, e.g. “Atlas 6 Swift, 512K · high”.",
        ],
        responsive: vec!["220 px menu; the chip ellipsises its summary when narrow."],
        height: 440,
        live: false,
        align: Align::Center,
        ..mk(
            "context-effort-menu",
            "Context + Effort menu",
            CAT,
            "One model chip whose dropdown sets both the context window (1M tagged “Higher usage”) and the effort, in two radio groups.",
            "ContextEffortMenu",
            "ContextEffortMenuEl",
            "context-effort-menu",
            "parity/ctxefffid.rs",
        )
    }
}

// ---------- Model effort menu ----------

fn model_effort_menu(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("open"), v.s("sub"));
    let open = v.s("open") != "closed";
    let sub = (v.s("open") == "submenu").then_some(5);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "min-height:380px;display:flex;flex-direction:column;justify-content:flex-end;gap:10px",
            ModelEffortMenu { default_open: open, default_sub: sub,
                on_change: move |(m, l): (usize, usize)| msg.set(format!("onChange({}, {})", effort_models()[m].name, effort_scale()[l].label)) }
            {status(msg)}
        }
    }
}

fn model_effort_menu_code(_: &Vals) -> String {
    Code::new("ModelEffortMenu")
        .raw("models", "Some(my_models())")
        .raw("on_change", "move |(model, level)| pick(model, level)")
        .done()
}

fn model_effort_menu_demo() -> Demo {
    Demo {
        controls: vec![sel(
            "open",
            "State",
            &["submenu", "list", "closed"],
            "submenu",
        )],
        variants: vec![
            var(
                "submenu",
                "Each model shows its effort as a grey suffix; the chosen one's submenu is open with High checked.",
                &[],
            ),
            var(
                "list",
                "Just the list: Nimbus 3 runs at Max, the suffix in red.",
                &[("open", "list")],
            ),
            var(
                "closed",
                "Closed: the chip names the chosen model.",
                &[("open", "closed")],
            ),
        ],
        render: model_effort_menu,
        code: model_effort_menu_code,
        keyboard: vec![
            (
                "↑ / ↓",
                "Moves between models, or between levels inside a submenu",
            ),
            ("→ / Enter", "Opens the model's effort submenu"),
            ("← / Escape", "Back to the models; Escape again closes"),
        ],
        a11y: vec![
            "Model rows are menuitems with aria-haspopup and aria-expanded; their name includes the effort (“Atlas 6, High”).",
            "Levels are menuitemradio rows; picking one picks the model too and returns focus to the chip.",
        ],
        responsive: vec![
            "330 px list; the submenu cascades to the right, and under a 640 px viewport it opens in place under its row.",
        ],
        height: 420,
        live: false,
        align: Align::Center,
        ..mk(
            "model-effort-menu",
            "Model effort menu",
            CAT,
            "Models listed with their effort as a suffix; hovering or → opens a cascading submenu that sets that model’s effort.",
            "ModelEffortMenu",
            "ModelEffortMenuEl",
            "model-effort-menu",
            "parity/modeleffid.rs",
        )
    }
}

// ---------- Nozzle line ----------

fn nozzle_scale(v: &Vals) -> Vec<oxyn_core::parity::thinkfid::EffortLevel> {
    match v.s("levels") {
        "four" => effort_scale_short(),
        "api" => effort_scale_api(),
        _ => effort_scale(),
    }
}

fn nozzle_slider(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}{}",
        v.s("level"),
        v.s("levels"),
        v.s("strokes"),
        v.s("caption"),
        v.s("tone"),
        v.s("size")
    );
    let scale = nozzle_scale(v);
    let names = scale.clone();
    let start = level_index(&scale, v.s("level"), scale.len() / 2);
    let strokes = v.f("strokes").max(12.0) as usize;
    let caption = v.b("caption");
    let tone = v.s("tone").to_string();
    let size = v.s("size").to_string();
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:440px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            NozzleSlider { levels: Some(scale), default_value: start, strokes, caption, tone, size,
                on_change: move |i: usize| msg.set(format!("onChange({})", names[i].label)) }
            {status(msg)}
        }
    }
}

fn nozzle_slider_code(v: &Vals) -> String {
    let mut c = Code::new("NozzleSlider");
    if v.s("tone") == "red" {
        c = c.raw("tone", "\"red\"");
    }
    if v.s("size") != "md" {
        c = c.raw("size", &format!("\"{}\"", v.s("size")));
    }
    c.raw("value", "Some(effort())")
        .raw("on_change", "move |i| effort.set(i)")
        .done()
}

fn nozzle_slider_demo() -> Demo {
    Demo {
        controls: vec![
            sel("tone", "Tone", &["gradient", "red"], "gradient"),
            sel("levels", "Levels", &["five", "four", "api"], "five"),
            sel("level", "Level", &FIVE, "high"),
            sel("size", "Size", &["sm", "md", "lg"], "md"),
            rng("strokes", "Lines", 25.0, 49.0, 4.0, "37"),
            tog("caption", "Caption", true),
        ],
        variants: vec![
            var(
                "high",
                "Even gradient lines; the solid nozzle sits on the centre stop in coral.",
                &[],
            ),
            var(
                "low",
                "On the first line, filled steel blue.",
                &[("level", "low")],
            ),
            var(
                "max",
                "At the far end, filled periwinkle.",
                &[("level", "max")],
            ),
            var(
                "red",
                "Tone red: every line and the nozzle in the one red.",
                &[("tone", "red")],
            ),
            var(
                "small",
                "Size sm, four levels, no caption.",
                &[("size", "sm"), ("levels", "four"), ("caption", "false")],
            ),
        ],
        render: nozzle_slider,
        code: nozzle_slider_code,
        keyboard: vec![
            (
                "← / → · Home / End",
                "One level, or the ends; the nozzle springs to the stop",
            ),
            (
                "Drag",
                "The nozzle follows the pointer and snaps to the nearest stop",
            ),
            ("Click", "The nozzle jumps to the nearest level"),
        ],
        a11y: vec![
            "role=slider with the level's name as value text; the lines and the nozzle are decoration.",
            "The level is named in the caption and the value text, so it never rests on colour; light-mode line colours keep 4:1 on white.",
        ],
        responsive: vec![
            "440 px up to its container; lines are placed by percentage, so they close up on narrow screens.",
        ],
        height: 200,
        live: false,
        align: Align::Center,
        ..mk(
            "nozzle-slider",
            "Nozzle line",
            CAT,
            "An Oxyn original after the user's reference: even thin lines in a steel-blue to coral to periwinkle gradient; a solid nozzle in the colour under it slides along and snaps to the levels. Tone red keeps the all-red look.",
            "NozzleSlider",
            "NozzleSliderEl",
            "nozzle-slider",
            "parity/nozzlefid.rs",
        )
    }
}

// ---------- Effort stairs ----------

fn effort_stairs(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("level"), v.s("budget"));
    let start = level_index(&effort_scale(), v.s("level"), 2);
    let budget = v.b("budget");
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:300px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            EffortStairs { default_value: start, budget,
                on_change: move |i: usize| msg.set(format!("onChange({})", effort_scale()[i].label)) }
            {status(msg)}
        }
    }
}

fn effort_stairs_code(_: &Vals) -> String {
    Code::new("EffortStairs")
        .raw("value", "Some(effort())")
        .raw("on_change", "move |i| effort.set(i)")
        .done()
}

fn effort_stairs_demo() -> Demo {
    Demo {
        controls: vec![
            sel("level", "Level", &FIVE, "high"),
            tog("budget", "Budget line", true),
        ],
        variants: vec![
            var(
                "high",
                "Three bars lit in ink; the line under the name gives a rough thinking budget and wait.",
                &[],
            ),
            var(
                "max",
                "All five lit, the top one red: no cap.",
                &[("level", "max")],
            ),
            var(
                "plain",
                "Without the budget line.",
                &[("budget", "false"), ("level", "medium")],
            ),
        ],
        render: effort_stairs,
        code: effort_stairs_code,
        keyboard: vec![
            ("← / → · Home / End", "One level, or the ends"),
            ("Pointer over a bar", "Previews that level's budget"),
        ],
        a11y: vec![
            "The bars are one slider; each bar is a click target, the slider's value text is the level's name.",
            "The budget preview on hover is extra; the chosen level is always said in the status line.",
        ],
        responsive: vec!["300 px card; bars share the width."],
        height: 260,
        live: false,
        align: Align::Center,
        ..mk(
            "effort-stairs",
            "Effort stairs",
            CAT,
            "An Oxyn original: one rising bar per level, lit up to the chosen one (the top in red); pointing at a bar previews its thinking budget and wait.",
            "EffortStairs",
            "EffortStairsEl",
            "effort-stairs",
            "parity/stairsfid.rs",
        )
    }
}

/// Every Thinking Selector demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        effort_ruler_demo(),
        reasoning_panel_demo(),
        effort_list_demo(),
        effort_pill_demo(),
        context_effort_menu_demo(),
        model_effort_menu_demo(),
        nozzle_slider_demo(),
        effort_stairs_demo(),
    ]
}
