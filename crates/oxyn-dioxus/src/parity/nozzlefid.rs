//! Thinking Selector family, variant "Nozzle line" ([`NozzleSlider`]), an Oxyn original drawn by
//! the user: short, closely spaced red strokes with a hand-drawn wobble; a transparent ring – the
//! nozzle – slides along them and snaps to the levels. Models: `oxyn_core::parity::{thinkfid,
//! nozzlefid}`. Styles: `nozzlefid.css` (classes `oxen-*`).

use dioxus::prelude::*;
use oxyn_core::parity::{
    nozzlefid::{is_stop, level_tick, nozzle_ticks},
    thinkfid::{EffortLevel, effort_said, effort_scale, effort_step, level_frac, snap_level},
};

use super::rulerfid::{Track, slider_key};

/// Thinking Selector – Nozzle line: a slider drawn as a row of short red pen strokes; the nozzle
/// (a see-through ring) glides along and snaps to the levels. Drag, click, arrows, Home / End.
#[component]
pub fn NozzleSlider(
    /// The scale (Low … Max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level.
    #[props(default)]
    value: Option<usize>,
    /// Starting level when uncontrolled.
    #[props(default = 2)]
    default_value: usize,
    /// How many strokes.
    #[props(default = 41)]
    strokes: usize,
    /// Show the level's name under the line.
    #[props(default = true)]
    caption: bool,
    /// Accessible name.
    #[props(default = "Thinking effort".to_string())]
    label: String,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let levels = levels.unwrap_or_else(effort_scale);
    let n = levels.len().max(1);
    let strokes = strokes.max(n).max(2);
    let mut own = use_signal(move || default_value.min(n - 1));
    let mut said = use_signal(String::new);
    // While dragging, the nozzle follows the pointer freely; it snaps on release.
    let mut free = use_signal(|| None::<f32>);
    let track = use_hook(Track::new);
    let cur = value.unwrap_or(own()).min(n - 1);
    let all = levels.clone();
    let pick = use_callback(move |i: usize| {
        let i = i.min(n - 1);
        own.set(i);
        said.set(effort_said(&all[i]));
        on_change.call(i);
    });
    let at = use_callback(move |f: f32| {
        free.set(Some(f));
        pick.call(snap_level(f, n));
    });
    let ticks = nozzle_ticks(strokes, 7);
    let pos = free().unwrap_or(level_frac(cur, n));
    let stop = level_tick(cur, n, strokes);
    let lv = levels[cur].clone();
    let dragging = (track.drag)();

    rsx! {
        div { class: "oxen", "data-drag": "{dragging}",
            div { class: "oxen-line", role: "slider", tabindex: "0", "aria-label": "{label}",
                "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuenow": "{cur}", "aria-valuetext": "{lv.label}",
                onmounted: move |e| track.mount(e.data()),
                onpointerdown: move |e: PointerEvent| track.press(e.client_coordinates().x, at),
                onpointermove: move |e: PointerEvent| if let Some(f) = track.moved(e.client_coordinates().x) { at.call(f) },
                onpointerup: move |_| { track.release(); free.set(None); },
                onpointerleave: move |_| { track.release(); free.set(None); },
                onkeydown: move |e: KeyboardEvent| if let Some(d) = slider_key(&e.key()) { e.prevent_default(); free.set(None); pick.call(effort_step(cur, d, n)); },
                for (t, k) in ticks.iter().enumerate() {
                    {
                        let x = t as f32 / (strokes - 1) as f32;
                        let h = k.height;
                        let r = k.tilt;
                        let s = is_stop(t, n, strokes);
                        rsx! { i { key: "{t}", "data-k": "{t}", class: "oxen-s", "data-stop": "{s}", "data-at": "{t == stop}",
                            style: "left:calc(14px + (100% - 28px) * {x});--h:{h};--r:{r}deg" } }
                    }
                }
                span { class: "oxen-noz", style: "left:calc(14px + (100% - 28px) * {pos})", "aria-hidden": "true" }
            }
            if caption { p { class: "oxen-cap", "aria-hidden": "true", "{lv.label}" } }
            div { class: "oxen-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
