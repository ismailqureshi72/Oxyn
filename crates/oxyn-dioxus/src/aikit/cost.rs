//! LLM cost calculator (Dioxus).

use dioxus::prelude::*;
use oxyn_core::aikit::cost::{Cost, MODELS, PRESETS, RANGES};
use oxyn_core::aikit::{thousands, usd};

use super::{cls, ico, key_of, live, mark};

/// LLM cost calculator: set tokens per request, requests per day and the prompt cache hit rate
/// (or pick a preset) and eight models re-rank by monthly cost; rows glide to their new rank and
/// bars show each model's share of the most expensive. Select a model for its per-request cost
/// and how it compares with the cheapest. Keys on sliders: ←/→ a step (Shift ten), Page up/down,
/// Home/End; ↑/↓ on the list select. Batch pricing halves every price (Oxyn extra).
#[component]
pub fn LlmCostCalculator(
    /// Starting preset (0 support chat, 1 RAG search, 2 coding agent).
    #[props(default)]
    preset: usize,
    /// Show the batch pricing switch (Oxyn extra).
    #[props(default)]
    batch: bool,
    /// Show the skeleton.
    #[props(default)]
    loading: bool,
    /// Show the error state.
    #[props(default)]
    error: bool,
) -> Element {
    let mut lc = use_signal(move || {
        let mut c = Cost::new();
        c.press(&format!("lc-preset-{}", preset.min(2)));
        c.loading = loading;
        if error {
            c.press("lc-fail");
        }
        c
    });
    let mut said = use_signal(String::new);
    let c = lc.read().clone();
    let mut act = move |id: String| {
        let x = lc.write().press(&id);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ranked = c.ranked();
    let sliders = [
        ("Input tokens per request", c.input, 0usize),
        ("Output tokens per request", c.output, 1),
        ("Requests per day", c.requests, 2),
        ("Prompt cache hits", c.cache, 3),
    ];
    rsx! {
        section { class: "oxaik oxaik-lc", "aria-label": "LLM cost calculator", "aria-busy": "{c.loading}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "What will it cost a month?" } p { class: "oxaik-sub oxaik-num", "{c.volume()}" } }
                div { class: "oxaik-row",
                    for (i, p) in PRESETS.iter().enumerate() {
                        button { key: "{i}", class: "oxaik-chip", r#type: "button", onclick: move |_| act(format!("lc-preset-{i}")), "{p.0}" }
                    }
                }
            }
            div { class: "oxaik-lc-grid",
                div { class: "oxaik-lc-sl",
                    for (label, v, k) in sliders {
                        {
                            let (lo, hi, step) = RANGES[k];
                            let shown = if k == 3 { format!("{v}%") } else { thousands(v) };
                            rsx! {
                                div { key: "{k}", class: "oxaik-lc-s",
                                    div { class: "oxaik-row oxaik-between", label { r#for: "oxaik-lc-{k}", class: "oxaik-mute", "{label}" } b { class: "oxaik-num", "{shown}" } }
                                    input { id: "oxaik-lc-{k}", class: "oxaik-range", r#type: "range", min: "{lo}", max: "{hi}", step: "{step}", value: "{v}",
                                        "aria-valuetext": "{shown}",
                                        oninput: move |e| { if let Ok(x) = e.value().parse::<u64>() { act(format!("lc-set-{k}-{x}")) } },
                                        onkeydown: move |e: KeyboardEvent| {
                                            let key = key_of(&e);
                                            if matches!(key.as_str(), "ArrowLeft"|"ArrowRight"|"ArrowUp"|"ArrowDown"|"PageUp"|"PageDown"|"Home"|"End") {
                                                e.prevent_default();
                                                let x = lc.write().key(k, &key, e.modifiers().shift());
                                                if let Some(x) = x { said.set(x) }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                    if batch {
                        button { class: cls("oxaik-switch", c.batch), r#type: "button", role: "switch", "aria-checked": "{c.batch}", onclick: move |_| act("lc-batch".into()),
                            span { class: "oxaik-knob" } "Batch API (50% off, results within 24h)"
                        }
                    }
                }
                div { class: "oxaik-lc-list",
                    if c.loading {
                        div { class: "oxaik-skel", "aria-hidden": "true", for k in 0..8 { i { key: "{k}", style: "width:{100 - k * 7}%" } } }
                    } else if let Some(err) = c.error.clone() {
                        div { class: "oxaik-err", role: "alert", {ico("alert", 18.0)} b { "Couldn't load prices" } span { class: "oxaik-mute", "{err}" }
                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("lc-retry".into()), {ico("retry", 12.0)} "Retry" }
                        }
                    } else {
                        ol { class: "oxaik-lc-rows", role: "listbox", "aria-label": "Models by monthly cost", tabindex: "0", style: "height:{MODELS.len() * 38}px",
                            onkeydown: move |e: KeyboardEvent| { let key = key_of(&e); if matches!(key.as_str(), "ArrowUp"|"ArrowDown"|"Home"|"End") { e.prevent_default(); let x = lc.write().key(9, &key, false); if let Some(x) = x { said.set(x) } } },
                            for i in 0..MODELS.len() {
                                {
                                    let m = MODELS[i];
                                    let rank = c.rank_of(i);
                                    let share = c.share(i);
                                    let sel = i == c.selected;
                                    rsx! {
                                        li { key: "{i}", class: cls("oxaik-lc-row", sel), role: "option", "aria-selected": "{sel}", style: "transform:translateY({rank * 38}px)",
                                            onclick: move |_| act(format!("lc-sel-{i}")),
                                            span { class: "oxaik-num oxaik-mute oxaik-lc-rk", "{rank + 1}" }
                                            span { class: "oxaik-row oxaik-lc-nm", {mark(m.1, 14.0)} span { class: "oxaik-lc-n", "{m.0}" }
                                                if rank == 0 { span { class: "oxaik-pill oxaik-lc-tag", "data-hue": "green", "Cheapest" } }
                                            }
                                            div { class: "oxaik-bar", "aria-hidden": "true", span { style: "width:{share * 100.0}%" } }
                                            b { class: "oxaik-num oxaik-lc-v", "{usd(c.monthly(i))}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-lc-foot",
                p { class: "oxaik-lc-sum oxaik-num", "{c.summary()}" }
                p { class: "oxaik-mute", "Sample list prices per 1M tokens; cached input billed at the cache rate. Cheapest first: {MODELS[ranked[0]].0}." }
            }
            {live(&said())}
        }
    }
}
