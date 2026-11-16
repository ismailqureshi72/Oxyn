//! Thinking Selector family, variant "Effort pill" ([`EffortPill`]): a "Select effort" pill that
//! opens a popover with the level, the model it applies to, a fat step slider with a dot per level
//! and a reset button. Model: `oxyn_core::parity::thinkfid`. Styles: `effpillfid.css` (`oxeq-*`).

use dioxus::prelude::*;
use oxyn_core::parity::thinkfid::{
    EffortLevel, effort_said, effort_scale, effort_step, level_frac, snap_level,
};

use super::efflistfid::chevron;
use super::rulerfid::{Track, reset_glyph, slider_key};

/// Thinking Selector – Effort pill: a pill that opens a popover with a dotted step slider. The pill
/// reads "Select effort" until a level is chosen, then the level's name. Escape closes the popover.
#[component]
pub fn EffortPill(
    /// The scale (Low … Max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level (`None` inside = nothing chosen yet).
    #[props(default)]
    value: Option<Option<usize>>,
    /// Starting level when uncontrolled.
    #[props(default)]
    default_value: Option<usize>,
    /// Where reset returns.
    #[props(default = 2)]
    reset: usize,
    /// The model the effort applies to (a link-styled line under the level).
    #[props(default = "Atlas 6".to_string())]
    model: String,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
    /// The model line was pressed (to switch models).
    #[props(default)]
    on_model: EventHandler<()>,
) -> Element {
    let levels = levels.unwrap_or_else(effort_scale);
    let n = levels.len().max(1);
    let mut own = use_signal(move || default_value.map(|v| v.min(n - 1)));
    let cur = value.unwrap_or(own()).map(|v| v.min(n - 1));
    let mut open = use_signal(move || default_open);
    // Only a menu the user opened gets the click-outside catcher; a preview that starts open
    // never covers the page.
    let mut armed = use_signal(|| false);
    let mut said = use_signal(String::new);
    let track = use_hook(Track::new);
    let all = levels.clone();
    let pick = use_callback(move |i: usize| {
        let i = i.min(n - 1);
        own.set(Some(i));
        said.set(effort_said(&all[i]));
        on_change.call(i);
    });
    let at = use_callback(move |f: f32| pick.call(snap_level(f, n)));
    let shown = cur.unwrap_or(reset.min(n - 1));
    let frac = level_frac(shown, n);
    let pill = cur.map_or("Select effort".to_string(), |i| levels[i].label.clone());
    let is_open = open();
    let home = reset.min(n - 1);
    let reset_label = format!("Reset to {}", levels[home].label);

    let catch = is_open && armed();

    rsx! {
        div { class: "oxeq", "data-open": "{is_open}", "data-armed": "{catch}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && open() { e.prevent_default(); open.set(false); },
            if catch { div { class: "oxth-catch", "aria-hidden": "true", onpointerdown: move |_| open.set(false) } }
            button { class: "oxeq-pill", r#type: "button", "aria-expanded": "{is_open}", "aria-haspopup": "dialog",
                "data-set": "{cur.is_some()}",
                onclick: move |_| { armed.set(true); open.set(!open()) },
                span { "{pill}" } {chevron(false)} }
            if is_open {
                div { class: "oxeq-pop", role: "dialog", "aria-label": "Thinking effort",
                    button { class: "oxeq-reset", r#type: "button", "aria-label": "{reset_label}", title: "{reset_label}",
                        disabled: cur == Some(home), onclick: move |_| pick.call(home), {reset_glyph()} }
                    p { class: "oxeq-lv", "{levels[shown].label}" }
                    button { class: "oxeq-model", r#type: "button", onclick: move |_| on_model.call(()),
                        "{model}" span { "aria-hidden": "true", "›" } }
                    div { class: "oxeq-track", role: "slider", tabindex: "0", "aria-label": "Thinking effort",
                        "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuenow": "{shown}", "aria-valuetext": "{levels[shown].label}",
                        onmounted: move |e| track.mount(e.data()),
                        onpointerdown: move |e: PointerEvent| track.press(e.client_coordinates().x, at),
                        onpointermove: move |e: PointerEvent| if let Some(f) = track.moved(e.client_coordinates().x) { at.call(f) },
                        onpointerup: move |_| track.release(),
                        onpointerleave: move |_| track.release(),
                        onkeydown: move |e: KeyboardEvent| if let Some(d) = slider_key(&e.key()) { e.prevent_default(); pick.call(effort_step(shown, d, n)); },
                        span { class: "oxeq-fill", style: "width:calc(30px + (100% - 30px) * {frac})" }
                        for i in 0..n {
                            {
                                let x = level_frac(i, n);
                                rsx! { i { key: "{i}", "data-k": "{i}", class: "oxeq-dot", "data-past": "{i <= shown}", style: "left:calc(15px + (100% - 30px) * {x})" } }
                            }
                        }
                        span { class: "oxeq-thumb", style: "left:calc(15px + (100% - 30px) * {frac})" }
                    }
                }
            }
            div { class: "oxeq-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
