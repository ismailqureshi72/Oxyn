//! affectUI fidelity pass, family B part 8 (plan #32): the exact **Fallback Chain**
//! ([`FallbackChain`]). Behaviour and look only; the model is `oxyn_core::parity::fallbackfid`.
//! Styles: `fallbackfid.css` (classes `oxfb-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::fallbackfid::{
        FbAttempt, FbState, fb_demo, fb_final, fb_headline, fb_ms, fb_states, fb_total_ms,
    },
};

use crate::brand::BrandIcon;

/// Row pitch of the model column, px (row 52 + gap 10) – the SVG links use it.
const PITCH: f32 = 62.0;
const ROW: f32 = 52.0;
const LINK_W: f32 = 48.0;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn svg_path(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// Fallback Chain (affectUI): a request drawn as curves to every model it may use, in order –
/// a moving highlight marks the current try, a failed try's curve turns red and broken with its
/// reason, the first model that answers is joined on to the response, later ones stay unused.
#[component]
pub fn FallbackChain(
    /// The models in fallback order (the demo's when `None`).
    #[props(default)]
    attempts: Option<Vec<FbAttempt>>,
    /// The request, as a short line.
    #[props(default = "Summarise this week's support tickets".to_string())]
    request: String,
    /// The response's first words.
    #[props(default = "Three themes stand out: slow exports, SSO sign-in loops and…".to_string())]
    response: String,
    /// Demo: the chain plays out live; otherwise it shows the settled result.
    #[props(default)]
    live: bool,
    /// Hand-over pause between two tries, ms.
    #[props(default = 350)]
    gap_ms: u64,
    /// The attempt whose details are open from the start.
    #[props(default)]
    open: Option<usize>,
    /// Heading.
    #[props(default = "Fallback chain".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Replay pressed.
    #[props(default)]
    on_replay: EventHandler<()>,
    /// An attempt's details opened (index) or closed (`None`).
    #[props(default)]
    on_open_change: EventHandler<Option<usize>>,
) -> Element {
    let mut t0 = use_signal(move || now);
    let mut playing = use_signal(move || live);
    let mut sel = use_signal(move || open);
    let a = attempts.clone().unwrap_or_else(fb_demo);
    let n = a.len();
    let el = now.since(t0()).as_millis() as u64;
    let (states, finished) = if playing() {
        fb_states(&a, el.saturating_sub(500), gap_ms)
    } else {
        (fb_final(&a), true)
    };
    let headline = fb_headline(&a, &states);
    let answered = states.iter().position(|s| *s == FbState::Answered);
    let all_failed = finished && answered.is_none();
    let total = fb_total_ms(&a, gap_ms);
    let h = (n as f32 * PITCH - (PITCH - ROW)).max(ROW);
    let mid = h / 2.0;
    let yi = |i: usize| i as f32 * PITCH + ROW / 2.0;
    let pill = if !finished {
        "live"
    } else if all_failed {
        "failed"
    } else {
        "done"
    };

    rsx! {
        section { class: "oxfb", "data-style": style_attr(&style), "data-st": pill, "aria-label": "{title}",
            header { class: "oxfb-bar",
                span { class: "oxfb-logo", {svg_path("M2.5 8h3M5.5 8c2 0 2-4.5 4-4.5h4M5.5 8c2 0 2 4.5 4 4.5h4M9.5 8h4", 16)} }
                h2 { class: "oxfb-ttl", "{title}" }
                span { class: "oxfb-sub", "{n} models in order" }
                span { class: "oxfb-sp" }
                span { class: "oxfb-st", "data-st": pill, role: "status",
                    if pill == "live" { i {} }
                    "{headline}"
                }
                button { class: "oxfb-replay", r#type: "button", "aria-label": "Replay the chain", title: "Replay",
                    onclick: move |_| { t0.set(now); playing.set(true); sel.set(None); on_replay.call(()); },
                    {svg_path("M3 8a5 5 0 1 0 1.5-3.5M3 2.5V5h2.5", 14)}
                }
            }
            div { class: "oxfb-flow", style: "--h:{h}px",
                div { class: "oxfb-req", "data-lit": !states.is_empty(),
                    span { class: "oxfb-cap", "Request" }
                    span { class: "oxfb-reqt", "{request}" }
                }
                svg { class: "oxfb-links", width: "{LINK_W}", height: "{h}", view_box: "0 0 {LINK_W} {h}", "aria-hidden": "true",
                    for (i, s) in states.iter().enumerate() {
                        {
                            let y = yi(i);
                            let d = format!("M0 {mid} C{} {mid} {} {y} {LINK_W} {y}", LINK_W * 0.5, LINK_W * 0.5);
                            rsx! {
                                path { class: "oxfb-ln", "data-st": s.id(), d: "{d}" }
                                if *s == FbState::Trying { path { class: "oxfb-light", d: "{d}", key: "{i}-{el / 4000}" } }
                            }
                        }
                    }
                }
                ol { class: "oxfb-models",
                    for (i, (x, s)) in a.iter().zip(states.iter()).enumerate() {
                        {
                            let x = x.clone();
                            let is_open = sel() == Some(i);
                            let detail = match s {
                                FbState::Failed => x.outcome.reason(),
                                FbState::Answered => format!("Answered in {}", fb_ms(x.ms)),
                                FbState::Trying => "Trying…".to_string(),
                                FbState::Skipped => "Not needed".to_string(),
                                FbState::Waiting => "Next in line".to_string(),
                            };
                            let lbl = format!("{}. {} – {detail}", i + 1, x.model);
                            rsx! {
                                li { class: "oxfb-m", "data-st": s.id(), key: "{i}",
                                    button { class: "oxfb-node", r#type: "button", "aria-expanded": is_open, "aria-label": "{lbl}",
                                        onclick: move |_| { let v = if sel() == Some(i) { None } else { Some(i) }; sel.set(v); on_open_change.call(v); },
                                        span { class: "oxfb-ord", "{i + 1}" }
                                        span { class: "oxfb-brand", BrandIcon { name: x.brand.clone(), size: 16.0, coloured: true, fallback: true } }
                                        span { class: "oxfb-mt",
                                            b { "{x.model}" }
                                            small { "{detail}" }
                                        }
                                        match s {
                                            FbState::Failed => rsx! { span { class: "oxfb-mk oxfb-x", {svg_path("M4.5 4.5l7 7M11.5 4.5l-7 7", 12)} } },
                                            FbState::Answered => rsx! { span { class: "oxfb-mk oxfb-ok", {svg_path("M3.5 8.5l3 3 6-7", 12)} } },
                                            FbState::Trying => rsx! { span { class: "oxfb-mk oxfb-spin" } },
                                            _ => rsx! {},
                                        }
                                    }
                                    if is_open {
                                        div { class: "oxfb-det",
                                            span { "Took {fb_ms(x.ms)}" }
                                            if *s == FbState::Failed { span { "Handed on to #{i + 2}" } }
                                            if *s == FbState::Skipped { span { "An earlier model answered" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                svg { class: "oxfb-links oxfb-out", width: "{LINK_W}", height: "{h}", view_box: "0 0 {LINK_W} {h}", "aria-hidden": "true",
                    if let Some(i) = answered {
                        path { class: "oxfb-ln", "data-st": "answered", d: "M0 {yi(i)} C{LINK_W * 0.5} {yi(i)} {LINK_W * 0.5} {mid} {LINK_W} {mid}" }
                    }
                }
                div { class: "oxfb-res", "data-st": if answered.is_some() { "answered" } else if all_failed { "failed" } else { "waiting" },
                    span { class: "oxfb-cap", if all_failed { "No response" } else { "Response" } }
                    if let Some(i) = answered {
                        span { class: "oxfb-rest", "{response}" }
                        span { class: "oxfb-meta", "{a[i].model} · {fb_ms(total)} total" }
                    } else if all_failed {
                        span { class: "oxfb-rest", "Every model in the chain failed – the request was not answered." }
                    } else {
                        span { class: "oxfb-wait", i {} i {} i {} }
                    }
                }
            }
        }
    }
}
