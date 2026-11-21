//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Parallel Attempts**
//! ([`AttemptRace`]). Behaviour and look only; the model is `oxyn_core::kit::parallelfid`.
//! Styles: `parallelfid.css` (classes `oxpt-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::parallelfid::{
        PaAttempt, PaScript, PaStatus, pa_at, pa_best, pa_cost, pa_demo, pa_dur, pa_finished,
        pa_len, pa_usd,
    },
};

use crate::brand::BrandIcon;

/// Rest after the race before the demo loops, ms.
const REST_MS: u64 = 6000;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// Parallel Attempts (affectUI parity): a best-of-N race over one task. Each attempt is a row
/// (model, approach, progress, current step, cost so far); finished rows carry a score, the top
/// score is tagged Best and used unless another is picked.
#[component]
pub fn AttemptRace(
    /// Attempts as they stand (controlled). When `None` the demo race plays from `scripts`.
    #[props(default)]
    attempts: Option<Vec<PaAttempt>>,
    /// The scripted race for live demos (the built-in one when `None`).
    #[props(default)]
    scripts: Option<Vec<PaScript>>,
    /// Demo: loop the race; otherwise it shows the end.
    #[props(default)]
    live: bool,
    /// The task.
    #[props(default = "Steady the flaky payment test".to_string())]
    task: String,
    /// Heading; "Best of N" when empty.
    #[props(default)]
    title: String,
    /// The attempt in use (controlled); the best one when `None`.
    #[props(default)]
    picked: Option<String>,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// An attempt was picked (its id).
    #[props(default)]
    on_pick: EventHandler<String>,
) -> Element {
    let t0 = use_signal(move || now);
    let mut mine = use_signal(move || picked.clone());
    let sc = scripts.clone().unwrap_or_else(pa_demo);
    let len = pa_len(&sc);
    let (a, cycle) = match attempts.clone() {
        Some(a) => (a, 0),
        None if live => {
            let el = now.since(t0()).as_millis() as u64;
            let per = len + REST_MS;
            (pa_at(&sc, el % per), el / per)
        }
        None => (pa_at(&sc, len), 0),
    };
    let n = a.len();
    let title = if title.is_empty() {
        format!("Best of {n}")
    } else {
        title.clone()
    };
    let best = pa_best(&a);
    let done = pa_finished(&a);
    let all_done = done == n;
    // A pick from an earlier loop of the demo does not carry over.
    let use_id = mine()
        .filter(|id| a.iter().any(|x| x.id == *id && x.status == PaStatus::Done))
        .or_else(|| best.map(|i| a[i].id.clone()));

    rsx! {
        section { class: "oxpt", "data-style": style_attr(&style), "data-done": all_done, "data-cycle": "{cycle}", "aria-label": "{title}",
            header { class: "oxpt-bar",
                span { class: "oxpt-logo", {icon("M8 2 2.5 5 8 8l5.5-3zM2.5 8 8 11l5.5-3M2.5 11 8 14l5.5-3", 15)} }
                h2 { class: "oxpt-ttl", "{title}" }
                span { class: "oxpt-task", "{task}" }
                span { class: "oxpt-sp" }
                span { class: "oxpt-done", "{done} of {n} done" }
                b { class: "oxpt-cost", "{pa_usd(pa_cost(&a))}" }
            }
            div { class: "oxpt-list", role: "radiogroup", "aria-label": "{title}",
                for (i, x) in a.iter().enumerate() {
                    {
                        let in_use = use_id.as_deref() == Some(x.id.as_str()) && x.status == PaStatus::Done;
                        let is_best = best == Some(i);
                        let pct = (x.progress * 100.0).round() as u32;
                        let id = x.id.clone();
                        let model = x.model.clone();
                        let score = x.score.unwrap_or(0);
                        let line2 = if x.status == PaStatus::Running { x.step.clone() } else { x.note.clone() };
                        let back = all_done && !in_use;
                        rsx! {
                            div { class: "oxpt-row", key: "{x.id}", "data-k": "{x.id}", "data-st": x.status.id(), "data-use": in_use, "data-best": is_best, "data-back": back, style: "--i:{i}",
                                span { class: "oxpt-tile", "data-brand": "{x.brand}", BrandIcon { name: x.brand.clone(), size: 13.0, coloured: true, fallback: true } }
                                span { class: "oxpt-name",
                                    span { class: "oxpt-l1", b { "{x.model}" } span { class: "oxpt-ap", " · {x.approach}" } }
                                    span { class: "oxpt-l2", "data-run": x.status == PaStatus::Running, "{line2}" }
                                }
                                span { class: "oxpt-track", "aria-hidden": "true",
                                    i { class: "oxpt-fill", style: "width:{pct}%" }
                                }
                                span { class: "oxpt-right",
                                    match x.status {
                                        PaStatus::Running => rsx! {
                                            span { class: "oxpt-sc oxpt-pct", "{pct}%" }
                                            span { class: "oxpt-meta", "{pa_usd(x.cost)}" }
                                        },
                                        PaStatus::Failed => rsx! {
                                            span { class: "oxpt-sc oxpt-fail", "Failed" }
                                            span { class: "oxpt-meta", "{pa_dur(x.took_s)} · {pa_usd(x.cost)}" }
                                        },
                                        PaStatus::Done => rsx! {
                                            span { class: "oxpt-sc",
                                                if is_best { span { class: "oxpt-best", "Best" } }
                                                b { "{score}" }
                                            }
                                            span { class: "oxpt-meta", "{pa_dur(x.took_s)} · {pa_usd(x.cost)}" }
                                        },
                                    }
                                }
                                if x.status == PaStatus::Done {
                                    button { class: "oxpt-use", r#type: "button", role: "radio", "aria-checked": in_use,
                                        "aria-label": if in_use { format!("Using {model}, score {score}") } else { format!("Use {model}, score {score}") },
                                        onclick: move |_| { mine.set(Some(id.clone())); on_pick.call(id.clone()); },
                                        if in_use { {icon("M3.5 8.5l3 3 6-7", 12)} "In use" } else { "Use this one" }
                                    }
                                } else {
                                    span { class: "oxpt-use oxpt-ghost", "aria-hidden": "true" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
