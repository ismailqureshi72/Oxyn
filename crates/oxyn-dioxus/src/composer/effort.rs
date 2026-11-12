//! `EffortChip` (`docs/specs/composer-effort-control.md`). Controlled: draws an
//! [`EffortView`] and reports the chosen setting.

use dioxus::prelude::*;
use oxyn_core::composer::{
    EffortLevel, EffortSetting, EffortVariant, ReasoningSupport,
    effort::{EffortView, allowed, bars, prompt_glyph},
};
use oxyn_tokens::ColorRole;

use super::icons::Icon;
use crate::color_var;

/// The tier colour (`›` amber at tier A, `»` violet at tier B).
pub fn tier_color(tier: u8) -> String {
    match tier {
        0 => color_var(ColorRole::FgMuted),
        1 => color_var(ColorRole::StatusWarningFg),
        _ => color_var(ColorRole::AgentViolet),
    }
}

/// The effort control.
#[component]
pub fn EffortChip(
    /// The frame from `EffortControl::view`.
    view: EffortView,
    /// The model's reasoning support.
    support: ReasoningSupport,
    /// `segmented` · `signal-bars` · `slider` · `chevrons` · `toggle` · `menu`.
    #[props(default = "signal-bars".to_string())]
    variant: String,
    /// Menu open.
    #[props(default)]
    open: bool,
    /// Model name for the disabled tooltip.
    #[props(default)]
    model: String,
    /// Chip pressed.
    #[props(default)]
    on_toggle: EventHandler<()>,
    /// A setting chosen.
    #[props(default)]
    on_pick: EventHandler<EffortSetting>,
) -> Element {
    let v = EffortVariant::from_id(&variant).unwrap_or_default();
    let levels = allowed(&support);
    if matches!(support, ReasoningSupport::None) {
        return rsx! {
            span { class: "oxyn-ef off", tabindex: "0", title: "{model} doesn't use adjustable reasoning",
                "aria-disabled": "true", Icon { name: "brain", size: 13.0 } "Effort" }
        };
    }
    if matches!(support, ReasoningSupport::Toggle) || v == EffortVariant::Toggle {
        let on = match view.setting {
            EffortSetting::Toggle(b) => b,
            ref s => s.level().is_some_and(|l| l > EffortLevel::Minimal),
        };
        let next = if matches!(support, ReasoningSupport::Toggle) {
            EffortSetting::Toggle(!on)
        } else {
            EffortSetting::Level(if on {
                levels.first().copied().unwrap_or(EffortLevel::Low)
            } else {
                levels.last().copied().unwrap_or(EffortLevel::High)
            })
        };
        return rsx! {
            button { class: if on { "oxyn-ef toggle on" } else { "oxyn-ef toggle" }, role: "switch", "aria-checked": "{on}",
                title: "Thinking · ⌥T", onclick: move |_| on_pick.call(next.clone()),
                span { class: "sw", i {} }
                "Thinking"
            }
        };
    }
    let level = view.setting.level().unwrap_or(EffortLevel::Medium);
    let lit = bars(level, &levels);
    let tier_c = tier_color(view.tier);
    let label = view.setting.label();
    let pending = view
        .pending
        .as_ref()
        .map(|p| p.level().map(EffortLevel::short).unwrap_or("Auto"));
    let tooltip = format!("Thinking effort: {label} · ⌥. / ⌥,");
    let menu = rsx! {
        if open {
            div { class: "oxyn-pop oxyn-ef-menu", role: "menu", "aria-label": "Thinking effort",
                for (i, l) in levels.iter().rev().enumerate() {
                    {
                        let l = *l;
                        let on = l == level;
                        let n = levels.len() - i;
                        rsx! {
                            div { class: if on { "row on" } else { "row" }, role: "menuitemradio", "aria-checked": "{on}",
                                onclick: move |_| on_pick.call(EffortSetting::Level(l)),
                                span { class: "bars", Bars { lit: bars(l, &levels), color: tier_color(0) } }
                                span { class: "nm", "{l.label()}" }
                                span { class: "hint", "{l.hint()}" }
                                span { class: "kbd", "{n}" }
                            }
                        }
                    }
                }
            }
        }
    };
    let badge = rsx! {
        if let Some(p) = pending { span { class: "oxyn-pending", title: "Applies to your next message", "→ {p}" } }
    };
    let note = rsx! {
        if let Some(n) = view.note.clone() { span { class: "oxyn-ef-note", role: "status", "{n}" } }
    };
    match v {
        EffortVariant::Segmented => rsx! {
            div { class: "oxyn-ef-wrap",
                div { class: "oxyn-ef seg", role: "radiogroup", "aria-label": "Thinking effort", title: "{tooltip}",
                    for l in levels.iter().copied() {
                        button { class: if l == level { "on" } else { "" }, role: "radio", "aria-checked": "{l == level}",
                            onclick: move |_| on_pick.call(EffortSetting::Level(l)), "{l.short()}" }
                    }
                }
                {badge} {note}
            }
        },
        EffortVariant::Slider => {
            let n = levels.len().max(2) - 1;
            let i = levels.iter().position(|l| *l == level).unwrap_or(0);
            let x = i as f32 / n as f32 * 100.0;
            rsx! {
                div { class: "oxyn-ef-wrap",
                    div { class: "oxyn-ef slider", role: "slider", tabindex: "0", "aria-label": "Thinking effort",
                        "aria-valuetext": "{label}", title: "{tooltip}",
                        span { class: "end", "Faster" }
                        span { class: "track",
                            span { class: "fill", style: "width:{x}%;background:{tier_c}" }
                            for (j, l) in levels.iter().copied().enumerate() {
                                span { class: if j <= i { "det on" } else { "det" }, style: "left:{j as f32 / n as f32 * 100.0}%",
                                    title: "{l.label()}", onclick: move |_| on_pick.call(EffortSetting::Level(l)) }
                            }
                            span { class: "thumb", style: "left:{x}%" }
                        }
                        span { class: "end", "Smarter" }
                        span { class: "val", "{level.short()}" }
                    }
                    {badge} {note}
                }
            }
        }
        EffortVariant::Chevrons => rsx! {
            div { class: "oxyn-ef-wrap",
                button { class: "oxyn-ef chev", title: "{tooltip}", "aria-label": "Thinking effort: {label}", onclick: move |_| on_toggle.call(()),
                    span { class: "g", style: "color:{tier_c}", "{prompt_glyph(view.tier)}" }
                    span { class: "lv", "{level.short().to_lowercase()}" }
                }
                {badge} {note} {menu}
            }
        },
        EffortVariant::Menu => rsx! {
            div { class: "oxyn-ef-wrap",
                button { class: "oxyn-ef menu", title: "{tooltip}", "aria-haspopup": "menu", "aria-expanded": "{open}", onclick: move |_| on_toggle.call(()),
                    "Effort: " b { "{level.short()}" } Icon { name: "chevron", size: 10.0, stroke: 2.0 }
                }
                {badge} {note} {menu}
            }
        },
        _ => rsx! {
            div { class: "oxyn-ef-wrap",
                button { class: "oxyn-ef bars-chip", title: "{tooltip}", "aria-haspopup": "menu", "aria-expanded": "{open}",
                    "aria-label": "Thinking effort: {label}", onclick: move |_| on_toggle.call(()),
                    Bars { lit, color: if view.tier > 0 { tier_c.clone() } else { "currentColor".into() } }
                    "{level.short()}"
                }
                {badge} {note} {menu}
            }
        },
    }
}

#[component]
fn Bars(lit: u8, color: String) -> Element {
    rsx! {
        span { class: "oxyn-bars", "aria-hidden": "true",
            for i in 0..4u8 {
                {
                    let h = 4 + i * 2;
                    let st = if i < lit { format!("height:{h}px;background:{color}") } else { format!("height:{h}px") };
                    rsx! { i { class: if i < lit { "on" } else { "" }, style: "{st}" } }
                }
            }
        }
    }
}
