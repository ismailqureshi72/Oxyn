//! Model card (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::model_card::{BENCHMARKS, MODALITIES, ModelCard as Mc, VOLUMES, View};
use oxyn_core::aikit::{tokens_short, usd};

use super::{cls, ico, key_of, live, mark, seg};

/// Model card: what a model can do and what it costs. Model tabs swap every value in place; a
/// monthly volume slider estimates the cost; benchmark rows show score, rank and the gap to the
/// leader on hover or focus; Compare swaps (in the same box, the card never resizes) to a side by
/// side table against a rival with the better value checked and a tally of who leads.
#[component]
pub fn ModelCard(
    /// Host clock.
    now: ClockMs,
    /// Starting model (0 Opus 4.1, 1 Sonnet 4.5, 2 GPT-5, 3 Gemini 2.5 Pro, 4 Grok 4).
    #[props(default = 1)]
    model: usize,
    /// `details` or `compare`.
    #[props(default = "details".to_string())]
    view: String,
    /// Starting volume stop (0–8: 1M … 1B tokens a month).
    #[props(default = 5)]
    volume: usize,
    /// Show the skeleton.
    #[props(default)]
    loading: bool,
    /// Show the error state.
    #[props(default)]
    error: bool,
    /// Show the Use model button (model picker detail pane).
    #[props(default)]
    selectable: bool,
    /// Show the model tabs above the card.
    #[props(default = true)]
    tabs: bool,
) -> Element {
    let mut mc = use_signal(move || {
        let mut c = Mc::new();
        c.model = model.min(4);
        if c.rival == c.model {
            c.rival = (c.model + 1) % 5;
        }
        c.view = if view == "compare" {
            View::Compare
        } else {
            View::Details
        };
        c.vol = volume.min(8);
        c.loading = loading;
        if error {
            c.press("mc-fail", 0);
        }
        c
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = mc.read().clone();
    let m = c.m().clone();
    let r = c.r().clone();
    let busy = c.loading;
    let err = c.error.clone();
    let panel = if busy {
        "loading"
    } else if err.is_some() {
        "error"
    } else if c.view == View::Compare {
        "compare"
    } else {
        "details"
    };
    let mut act = move |id: String| {
        let x = mc.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let names: Vec<String> = c
        .models
        .iter()
        .map(|x| x.name.replace("Claude ", ""))
        .collect();
    let rows = c.bench_rows();
    let readout = c
        .hover
        .map(|h| c.readout(h))
        .unwrap_or_else(|| format!("{}, {}.", BENCHMARKS[c.bench].2, BENCHMARKS[c.bench].3));
    let ctx_frac = (m.context as f64 / 1_000_000.0).min(1.0);
    rsx! {
        div { class: "oxaik-mcwrap",
            if tabs {
                {seg("Model", names, c.model, EventHandler::new(move |i| act(format!("mc-model-{i}"))), EventHandler::new(move |k: String| { let i = oxyn_core::aikit::rove(mc.peek().model, 5, &k); act(format!("mc-model-{i}")) }))}
            }
            article { class: "oxaik oxaik-mc", "aria-label": "{m.name}", "aria-busy": "{busy}",
                header { class: "oxaik-hd",
                    div { class: "oxaik-row oxaik-mc-id",
                        {mark(m.name, 22.0)}
                        div { class: "oxaik-hl", h3 { class: "oxaik-h", "{m.name}" } span { class: "oxaik-mute", "{m.provider}" } }
                    }
                    if !c.models.is_empty() {
                        button { class: cls("oxaik-switch", c.view == View::Compare), r#type: "button", role: "switch", "aria-checked": "{c.view == View::Compare}", disabled: busy || err.is_some(),
                            onclick: move |_| act("mc-compare".into()),
                            span { class: "oxaik-knob" } "Compare"
                        }
                    }
                }
                div { class: "oxaik-stack", "data-show": "{panel}",
                    // Details.
                    div { class: "oxaik-panel", "data-p": "details",
                        p { class: "oxaik-mc-sum", "{m.summary}" }
                        ul { class: "oxaik-mc-mods",
                            for (k, md) in MODALITIES.iter().enumerate() {
                                li { class: cls("oxaik-mc-mod", m.modalities[k]), "aria-label": if m.modalities[k] { format!("{md}, supported") } else { format!("{md}, not supported") }, "{md}" }
                            }
                        }
                        dl { class: "oxaik-mc-specs oxaik-num",
                            div { dt { "Context window" } dd { "{tokens_short(m.context)} tokens" } }
                            div { dt { "Max output" } dd { "{tokens_short(m.max_out)} tokens" } }
                            div { dt { "Knowledge cutoff" } dd { "{m.cutoff}" } }
                        }
                        div { class: "oxaik-mc-gauge",
                            div { class: "oxaik-bar", "aria-hidden": "true", span { style: "width:{ctx_frac * 100.0}%" } }
                            span { class: "oxaik-mute oxaik-num", "{c.pages()}" }
                        }
                        dl { class: "oxaik-mc-specs oxaik-num",
                            div { dt { "Input per 1M" } dd { "{usd(m.input)}" } }
                            div { dt { "Output per 1M" } dd { "{usd(m.output)}" } }
                            div { dt { "Cached per 1M" } dd { "{usd(m.cached)}" } }
                        }
                        div { class: "oxaik-mc-vol",
                            label { class: "oxaik-mute", r#for: "oxaik-mc-vol", "Monthly volume" }
                            input { id: "oxaik-mc-vol", class: "oxaik-range", r#type: "range", min: "0", max: "8", step: "1", value: "{c.vol}",
                                "aria-valuetext": "{tokens_short(VOLUMES[c.vol])} tokens a month",
                                oninput: move |e| { if let Ok(v) = e.value().parse::<usize>() { act(format!("mc-vol-{v}")) } },
                            }
                            div { class: "oxaik-mc-cost oxaik-num",
                                span { class: "oxaik-mono", "{tokens_short(VOLUMES[c.vol])} tokens" }
                                span { "Estimated " b { "{usd(c.cost_of(c.model))}/mo" } }
                            }
                        }
                        div { class: "oxaik-mc-bench",
                            div { class: "oxaik-row oxaik-between",
                                h4 { class: "oxaik-h4", "Benchmarks" }
                                {seg("Benchmark", BENCHMARKS.iter().map(|b| b.1.to_string()).collect(), c.bench, EventHandler::new(move |i| act(format!("mc-bench-{i}"))), EventHandler::new(move |k: String| { let x = mc.write().key("bench", &k, n); if let Some(x) = x { said.set(x) } }))}
                            }
                            ul { class: "oxaik-mc-rows", "data-hot": "{c.hover.is_some()}",
                                for (k, (i, sc)) in rows.iter().enumerate() {
                                    {
                                        let nm = c.models[*i].name;
                                        let w = sc.unwrap_or(0.0);
                                        let lab = c.readout(k);
                                        let txt = sc.map(|x| format!("{x:.1}%")).unwrap_or("None".into());
                                        rsx! {
                                            li { key: "{k}", class: cls("oxaik-mc-row", c.hover == Some(k)), tabindex: "0", "aria-label": "{lab}",
                                                onmouseenter: move |_| act(format!("mc-row-{k}")), onmouseleave: move |_| act("mc-unrow".into()),
                                                onfocus: move |_| act(format!("mc-row-{k}")), onblur: move |_| act("mc-unrow".into()),
                                                span { class: "oxaik-row oxaik-mc-rn", {mark(nm, 13.0)} span { "{nm}" } }
                                                div { class: "oxaik-bar", "data-hue": if k == 0 { "neutral" } else { "" }, "aria-hidden": "true", span { style: "width:{w}%" } }
                                                span { class: "oxaik-num oxaik-mc-sc", "{txt}" }
                                            }
                                        }
                                    }
                                }
                            }
                            p { class: "oxaik-mute oxaik-mc-read", "{readout} Sample prices and scores, not official." }
                        }
                    }
                    // Compare.
                    div { class: "oxaik-panel", "data-p": "compare",
                        div { class: "oxaik-row oxaik-between",
                            span { class: "oxaik-mute", "Compare with" }
                            div { class: "oxaik-menu-wrap",
                                button { class: "oxaik-btn sm", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{c.menu}",
                                    onclick: move |_| act("mc-menu".into()),
                                    onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowDown"|"ArrowUp"|"Enter"|" "|"Escape") { e.prevent_default(); let x = mc.write().key("menu", &k, n); if let Some(x) = x { said.set(x) } } },
                                    {mark(r.name, 13.0)} "{r.name}" {ico("chevron", 12.0)}
                                }
                                if c.menu {
                                    ul { class: "oxaik-menu", role: "listbox", "aria-label": "Rivals",
                                        for (i, x) in c.models.iter().enumerate().filter(|(i, _)| *i != c.model) {
                                            li { key: "{i}", role: "option", "aria-selected": "{i == c.rival}", class: cls("oxaik-opt", i == c.rival),
                                                onclick: move |_| act(format!("mc-rival-{i}")),
                                                {mark(x.name, 13.0)} "{x.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "oxaik-scroll oxaik-mc-tw",
                            table { class: "oxaik-mc-t oxaik-num",
                                thead { tr { th { scope: "col", "Spec" } th { scope: "col", "{m.name}" } th { scope: "col", "{r.name}" } } }
                                tbody {
                                    for (k, row) in c.compare().into_iter().enumerate() {
                                        tr { key: "{k}",
                                            th { scope: "row", "{row.label}" }
                                            td { class: cls("", row.better == 0), "{row.a}" if row.better == 0 { span { class: "oxaik-better", {ico("check", 11.0)} span { class: "oxaik-sr", ", better by " } "{row.margin}" } } }
                                            td { class: cls("", row.better == 1), "{row.b}" if row.better == 1 { span { class: "oxaik-better", {ico("check", 11.0)} span { class: "oxaik-sr", ", better by " } "{row.margin}" } } }
                                        }
                                    }
                                }
                            }
                        }
                        p { class: "oxaik-mute", "{c.tally()} Prices per 1M tokens; cost assumes 75% input. Sample figures, not official." }
                    }
                    // Loading.
                    div { class: "oxaik-panel oxaik-skel", "data-p": "loading", "aria-hidden": "true",
                        for k in 0..7 { i { key: "{k}", style: "width:{[90, 70, 100, 60, 85, 100, 50][k]}%" } }
                    }
                    // Error.
                    div { class: "oxaik-panel oxaik-err", "data-p": "error", role: if panel == "error" { "alert" } else { "none" },
                        {ico("alert", 18.0)}
                        b { "Couldn't load model details" }
                        span { class: "oxaik-mute", "{err.clone().unwrap_or_default()}" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mc-retry".into()), {ico("retry", 13.0)} "Retry" }
                    }
                }
                div { class: "oxaik-row oxaik-between oxaik-mc-foot",
                    code { class: "oxaik-mono", "{m.id}" }
                    div { class: "oxaik-row",
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mc-copy".into()), {ico("copy", 13.0)} "Copy model id" }
                        if selectable {
                            button { class: if c.selected { "oxaik-btn sm pri" } else { "oxaik-btn sm" }, r#type: "button", "aria-pressed": "{c.selected}",
                                onclick: move |_| act("mc-select".into()),
                                if c.selected { {ico("check", 13.0)} "Selected" } else { "Use model" }
                            }
                        }
                    }
                }
                {live(&said())}
            }
        }
    }
}
