//! `ContextMeterChip` (`docs/specs/composer-context-meter.md`). Controlled: draws a
//! [`ContextView`] and the breakdown from the usage report.

use dioxus::prelude::*;
use oxyn_core::composer::{
    ContextCategory, ContextLevel, ContextUsage, ContextVariant, LabelMode,
    context::{ContextView, k},
};

use super::icons::Icon;
use crate::color_var;

const R: f32 = 6.5;

fn circ() -> f32 {
    std::f32::consts::TAU * R
}

/// The context meter.
#[component]
pub fn ContextMeterChip(
    /// Frame from `ContextMeter::view`.
    view: ContextView,
    /// The usage report (breakdown and grid).
    usage: ContextUsage,
    /// `ring` · `ring-segmented` · `bar` · `grid` · `text` · `card`.
    #[props(default = "ring".to_string())]
    variant: String,
    /// Breakdown popover open.
    #[props(default)]
    open: bool,
    /// Model name for the popover header.
    #[props(default)]
    model: String,
    /// Chip pressed.
    #[props(default)]
    on_toggle: EventHandler<()>,
    /// "Compact now".
    #[props(default)]
    on_compact: EventHandler<()>,
) -> Element {
    let v = ContextVariant::from_id(&variant).unwrap_or_default();
    let fill_c = color_var(view.level.color());
    let tip = view.tooltip();
    let pct = view.label(LabelMode::Percent);
    let stale = if view.stale { " stale" } else { "" };
    let lvl = match view.level {
        ContextLevel::Warning => " warn",
        ContextLevel::Critical | ContextLevel::Overflow => " crit",
        _ => "",
    };
    let pop = rsx! {
        if open { Breakdown { view: view.clone(), usage: usage.clone(), model: model.clone(), on_compact } }
    };
    let float = rsx! {
        if let Some((t, p)) = view.float.clone() {
            span { class: "oxyn-ctx-float", style: "opacity:{1.0 - p};transform:translateY({-14.0 * p}px)", "{t}" }
        }
    };
    let chip_class = format!("oxyn-ctx {}{lvl}{stale}", v.id());
    match v {
        ContextVariant::Text => rsx! {
            span { class: "oxyn-ctx-wrap",
                button { class: "{chip_class}", title: "{tip}", onclick: move |_| on_toggle.call(()),
                    style: "color:{fill_c}", "{view.label(LabelMode::Remaining)}" }
                {float} {pop}
            }
        },
        ContextVariant::Bar => rsx! {
            span { class: "oxyn-ctx-wrap",
                button { class: "{chip_class}", title: "{tip}", onclick: move |_| on_toggle.call(()), "aria-label": "Context {pct}",
                    span { class: "bar",
                        if view.compacting { span { class: "sweep" } }
                        else { span { class: "fill", style: "width:{view.fill * 100.0:.1}%;background:{fill_c};opacity:{view.alpha:.2}" } }
                    }
                    span { class: "lbl", "{pct}" }
                }
                {float} {pop}
            }
        },
        ContextVariant::Grid => {
            let cells = usage.grid(5, 4);
            rsx! {
                span { class: "oxyn-ctx-wrap",
                    button { class: "{chip_class}", title: "{tip}", onclick: move |_| on_toggle.call(()), "aria-label": "Context {pct}",
                        span { class: "mini",
                            for c in cells { i { style: "background:{color_var(c.color())}", class: if c == ContextCategory::Free { "free" } else { "" } } }
                        }
                        span { class: "lbl", "{pct}" }
                    }
                    {float} {pop}
                }
            }
        }
        ContextVariant::Card => rsx! {
            div { class: "oxyn-ctx-card",
                Breakdown { view: view.clone(), usage: usage.clone(), model: model.clone(), on_compact, inline: true }
            }
        },
        ContextVariant::RingSegmented | ContextVariant::Ring => {
            let c = circ();
            let segs: Vec<(String, f32, f32)> = if v == ContextVariant::RingSegmented {
                let mut at = 0.0;
                usage
                    .rows()
                    .into_iter()
                    .filter(|r| r.0 != ContextCategory::Free)
                    .map(|(cat, _, p)| {
                        let p = p.min(1.0 - at).max(0.0);
                        let s = (color_var(cat.color()), p * c, at * c);
                        at += p;
                        s
                    })
                    .collect()
            } else {
                vec![(fill_c.clone(), view.fill * c, 0.0)]
            };
            let label = if view.window.is_some() {
                pct.clone()
            } else {
                String::new()
            };
            rsx! {
                span { class: "oxyn-ctx-wrap",
                    button { class: "{chip_class}", title: "{tip}", onclick: move |_| on_toggle.call(()), "aria-label": "Context {pct}",
                        svg { class: if view.compacting { "ring spin" } else { "ring" }, width: "16", height: "16", view_box: "0 0 16 16", style: "opacity:{view.alpha:.2}",
                            circle { class: "track", cx: "8", cy: "8", r: "{R}" }
                            if view.compacting {
                                circle { class: "arc", cx: "8", cy: "8", r: "{R}", stroke: "{fill_c}", "stroke-dasharray": "{c * 0.25} {c}" }
                            } else {
                                for (col, len, off) in segs {
                                    circle { class: "arc", cx: "8", cy: "8", r: "{R}", stroke: "{col}",
                                        "stroke-dasharray": "{len:.2} {c:.2}", "stroke-dashoffset": "{-off:.2}" }
                                }
                            }
                        }
                        if !label.is_empty() { span { class: "lbl", "{label}" } }
                    }
                    {float} {pop}
                }
            }
        }
    }
}

#[component]
fn Breakdown(
    view: ContextView,
    usage: ContextUsage,
    model: String,
    on_compact: EventHandler<()>,
    #[props(default)] inline: bool,
) -> Element {
    let rows = usage.rows();
    let total = view.label(LabelMode::Tokens);
    let pct = view.label(LabelMode::Percent);
    rsx! {
        div { class: if inline { "oxyn-ctx-pop inline" } else { "oxyn-pop oxyn-ctx-pop" }, role: "dialog", "aria-label": "Context window",
            div { class: "head",
                span { class: "tot", "{total} tokens" }
                span { class: "pct", "{pct}" }
                if !model.is_empty() { span { class: "model", "{model}" } }
            }
            div { class: "stack",
                for (cat, _, p) in rows.iter().copied() {
                    i { class: if cat == ContextCategory::Reserved { "hatch" } else { "" },
                        style: "flex-basis:{p * 100.0:.2}%;background:{color_var(cat.color())}" }
                }
            }
            div { class: "cats",
                for (cat, t, p) in rows.iter().copied() {
                    div { class: "cat",
                        i { class: if cat == ContextCategory::Reserved { "sw hatch" } else { "sw" }, style: "background:{color_var(cat.color())}" }
                        span { class: "nm", "{cat.label()}" }
                        span { class: "n", "{k(t)}" }
                        span { class: "p", "{p * 100.0:.1}%" }
                    }
                }
            }
            if let Some(prompt) = view.prompt.clone() {
                div { class: "prompt", Icon { name: "compact", size: 13.0 } "{prompt}" }
            }
            div { class: "acts",
                button { class: "primary", onclick: move |_| on_compact.call(()), Icon { name: "compact", size: 13.0 } "Compact now" }
                button { "New thread with summary" }
            }
        }
    }
}
