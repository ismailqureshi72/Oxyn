//! Thinking Selector family, variant "Reasoning panel" ([`ReasoningPanel`]): a model-details panel
//! with the context length, a notched reasoning slider (R steps it; the top level fills the bar with
//! red dots), a Fast mode switch (F) and a Favorite button. Models: `oxyn_core::kit::thinkfid`
//! and `reasonfid`. Styles: `reasonfid.css` (classes `oxep-*`, palette `--oxth-*`).

use dioxus::prelude::*;
use oxyn_core::kit::{
    reasonfid::{PanelKey, PanelModel, context_label, fast_said, panel_demo_model, panel_key},
    thinkfid::{
        EffortLevel, effort_cycle, effort_said, effort_scale, effort_step, level_frac, pixel_fill,
        snap_level,
    },
};

use super::rulerfid::{Track, slider_key};

fn star(filled: bool) -> Element {
    let fill = if filled { "currentColor" } else { "none" };
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "{fill}", stroke: "currentColor", stroke_width: "1.8",
            stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "m12 3.2 2.7 5.6 6.1.8-4.5 4.2 1.1 6-5.4-2.9-5.4 2.9 1.1-6-4.5-4.2 6.1-.8z" } }
    }
}

fn model_glyph() -> Element {
    rsx! {
        svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.7",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "8.5" } path { d: "M3.5 12h17M12 3.5c2.6 2.4 2.6 14.6 0 17M12 3.5c-2.6 2.4-2.6 14.6 0 17" } }
    }
}

/// Thinking Selector – Reasoning panel: the details of one model, with a notched reasoning slider
/// (R steps it; the top level turns the bar into red dots), a Fast mode switch (F) and a Favorite
/// button. Single-letter keys work anywhere inside the panel.
#[component]
pub fn ReasoningPanel(
    /// The model (the demo's when `None`).
    #[props(default)]
    model: Option<PanelModel>,
    /// The scale (Low … Max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level.
    #[props(default)]
    value: Option<usize>,
    /// Starting level when uncontrolled.
    #[props(default = 2)]
    default_value: usize,
    /// Controlled Fast mode.
    #[props(default)]
    fast: Option<bool>,
    /// Starting Fast mode when uncontrolled.
    #[props(default)]
    default_fast: bool,
    /// Controlled favourite.
    #[props(default)]
    favorite: Option<bool>,
    /// Starting favourite when uncontrolled.
    #[props(default)]
    default_favorite: bool,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
    /// Fast mode flipped.
    #[props(default)]
    on_fast_change: EventHandler<bool>,
    /// Favorite flipped.
    #[props(default)]
    on_favorite_change: EventHandler<bool>,
) -> Element {
    let model = model.unwrap_or_else(panel_demo_model);
    let levels = levels.unwrap_or_else(effort_scale);
    let n = levels.len().max(1);
    let mut own = use_signal(move || default_value.min(n - 1));
    let mut own_fast = use_signal(move || default_fast);
    let mut own_fav = use_signal(move || default_favorite);
    let mut said = use_signal(String::new);
    let track = use_hook(Track::new);
    let cur = value.unwrap_or(own()).min(n - 1);
    let fast_on = fast.unwrap_or(own_fast());
    let fav_on = favorite.unwrap_or(own_fav());
    let all = levels.clone();
    let pick = use_callback(move |i: usize| {
        let i = i.min(n - 1);
        own.set(i);
        said.set(effort_said(&all[i]));
        on_change.call(i);
    });
    let at = use_callback(move |f: f32| pick.call(snap_level(f, n)));
    let set_fast = use_callback(move |on: bool| {
        own_fast.set(on);
        said.set(fast_said(on).to_string());
        on_fast_change.call(on);
    });
    let lv = levels[cur].clone();
    let top = cur == n - 1;
    let frac = level_frac(cur, n);
    let dots = pixel_fill(56, 4, 11);
    let ctx = context_label(model.context);
    let fav_label = if fav_on { "Favorited" } else { "Favorite" };
    let has_fast = model.has_fast;

    rsx! {
        section { class: "oxep", "aria-label": "{model.name} details", "data-top": "{top}",
            onkeydown: move |e: KeyboardEvent| {
                let m = e.modifiers();
                let modified = m.ctrl() || m.meta() || m.alt();
                if let Key::Character(c) = e.key() {
                    match panel_key(&c, modified) {
                        Some(PanelKey::Reasoning) => { e.prevent_default(); pick.call(effort_cycle(cur, n)); }
                        Some(PanelKey::Fast) if has_fast => { e.prevent_default(); set_fast.call(!fast_on); }
                        _ => {}
                    }
                }
            },
            div { class: "oxep-head", span { class: "oxep-mark", {model_glyph()} } span { class: "oxep-name", "{model.name}" } }
            div { class: "oxep-row", span { class: "oxep-k", "Context length" } span { class: "oxep-v", "{ctx}" } }
            div { class: "oxep-sep" }
            div { class: "oxep-row",
                span { class: "oxep-k", "Reasoning" kbd { class: "oxep-kbd", "R" } }
                span { class: "oxep-v", "data-top": "{top}", "{lv.label}" }
            }
            div { class: "oxep-track", role: "slider", tabindex: "0", "aria-label": "Reasoning",
                "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuenow": "{cur}", "aria-valuetext": "{lv.label}",
                "aria-keyshortcuts": "R", "data-top": "{top}",
                onmounted: move |e| track.mount(e.data()),
                onpointerdown: move |e: PointerEvent| track.press(e.client_coordinates().x, at),
                onpointermove: move |e: PointerEvent| if let Some(f) = track.moved(e.client_coordinates().x) { at.call(f) },
                onpointerup: move |_| track.release(),
                onpointerleave: move |_| track.release(),
                onkeydown: move |e: KeyboardEvent| if let Some(d) = slider_key(&e.key()) { e.prevent_default(); pick.call(effort_step(cur, d, n)); },
                span { class: "oxep-fill", style: "width:calc(20px + (100% - 22px) * {frac})" }
                if top {
                    span { class: "oxep-dots", "aria-hidden": "true",
                        for (i, on) in dots.iter().enumerate() {
                            i { key: "{i}", "data-k": "{i}", "data-on": "{on}" }
                        }
                    }
                }
                for i in 1..n.saturating_sub(1) {
                    {
                        let x = level_frac(i, n);
                        rsx! { i { key: "n{i}", "data-k": "n{i}", class: "oxep-notch", "data-past": "{i <= cur}", style: "left:calc(11px + (100% - 22px) * {x})" } }
                    }
                }
                span { class: "oxep-thumb", style: "left:calc(11px + (100% - 22px) * {frac})", "aria-hidden": "true" }
            }
            if has_fast {
                div { class: "oxep-row",
                    span { class: "oxep-k", "Fast mode" kbd { class: "oxep-kbd", "F" } }
                    button { class: "oxep-switch", r#type: "button", role: "switch", "aria-checked": "{fast_on}",
                        "aria-label": "Fast mode", "aria-keyshortcuts": "F",
                        onclick: move |_| set_fast.call(!fast_on), span { class: "oxep-knob" } }
                }
            }
            button { class: "oxep-fav", r#type: "button", "aria-pressed": "{fav_on}",
                onclick: move |_| { own_fav.set(!fav_on); said.set(if fav_on { "Removed from favorites".into() } else { "Added to favorites".into() }); on_favorite_change.call(!fav_on); },
                {star(fav_on)} "{fav_label}" }
            div { class: "oxep-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
