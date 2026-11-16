//! Thinking Selector family, variant "Effort stairs" ([`EffortStairs`]), an Oxyn original: rising
//! bars, one per level, lit up to the chosen one; pointing at a bar previews its rough budget.
//! Models: `oxyn_core::parity::{thinkfid, stairsfid}`. Styles: `stairsfid.css` (classes `oxes-*`).

use dioxus::prelude::*;
use oxyn_core::parity::{
    stairsfid::{stair_height, stairs_line},
    thinkfid::{EffortLevel, effort_said, effort_scale, effort_step},
};

use super::rulerfid::slider_key;

/// Thinking Selector – Effort stairs: one rising bar per level; bars up to the chosen level are
/// lit (the top one red), and the level under the pointer previews its thinking budget.
#[component]
pub fn EffortStairs(
    /// The scale (Low … Max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level.
    #[props(default)]
    value: Option<usize>,
    /// Starting level when uncontrolled.
    #[props(default = 2)]
    default_value: usize,
    /// Show the budget line.
    #[props(default = true)]
    budget: bool,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let levels = levels.unwrap_or_else(effort_scale);
    let n = levels.len().max(1);
    let mut own = use_signal(move || default_value.min(n - 1));
    let mut peek = use_signal(|| None::<usize>);
    let mut said = use_signal(String::new);
    let cur = value.unwrap_or(own()).min(n - 1);
    let all = levels.clone();
    let pick = use_callback(move |i: usize| {
        let i = i.min(n - 1);
        own.set(i);
        said.set(effort_said(&all[i]));
        on_change.call(i);
    });
    let shown = peek().unwrap_or(cur);
    let line = stairs_line(shown, n);
    let lv = levels[shown].clone();

    rsx! {
        div { class: "oxes", "data-top": "{cur == n - 1}",
            div { class: "oxes-head",
                span { class: "oxes-lv", "data-top": "{shown == n - 1}", "data-peek": "{peek().is_some()}", "{lv.label}" }
                if budget { span { class: "oxes-bud", "{line}" } }
            }
            div { class: "oxes-bars", role: "slider", tabindex: "0", "aria-label": "Thinking effort",
                "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuenow": "{cur}", "aria-valuetext": "{levels[cur].label}",
                onkeydown: move |e: KeyboardEvent| if let Some(d) = slider_key(&e.key()) { e.prevent_default(); pick.call(effort_step(cur, d, n)); },
                onpointerleave: move |_| peek.set(None),
                for i in 0..n {
                    {
                        let h = stair_height(i, n) * 100.0;
                        rsx! {
                            span { key: "{i}", "data-k": "{i}", class: "oxes-step", "data-on": "{i <= cur}", "data-top": "{i == n - 1}",
                                "data-peek": "{peek() == Some(i)}",
                                onpointerenter: move |_| peek.set(Some(i)),
                                onclick: move |_| pick.call(i),
                                i { class: "oxes-bar", style: "height:{h}%" }
                            }
                        }
                    }
                }
            }
            div { class: "oxes-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
