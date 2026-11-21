//! affectUI fidelity pass, family B part 8 (plan #32): the exact **Fallback Chain**
//! ([`FallbackChain`]). Behaviour and look only; the model is `oxyn_core::kit::fallbackfid`.
//! Styles: `fallbackfid.css` (classes `oxfb-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::fallbackfid::{
        FbAttempt, FbState, fb_demo, fb_final, fb_meter, fb_progress, fb_s, fb_states, fb_status,
        fb_total_ms, fb_usd,
    },
};

use crate::brand::BrandIcon;

/// Row pitch of the model column, px (row 46 + gap 10) – the SVG links use it.
const PITCH: f32 = 56.0;
const ROW: f32 = 46.0;
const LINK_W: f32 = 44.0;
/// Pause after a finished run before the demo loops, ms.
const REST_MS: u64 = 4200;
/// Lead-in before the first try, ms.
const LEAD_MS: u64 = 500;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn svg_path(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// The status line under a model's name and its tone.
fn line(x: &FbAttempt, s: FbState, next: bool) -> (String, &'static str) {
    match s {
        FbState::Failed => (x.outcome.reason(), "bad"),
        FbState::Answered => (format!("Answered · {} tokens", x.tokens), "ok"),
        FbState::Trying => ("Trying…".into(), "live"),
        FbState::Skipped => ("Not needed".into(), "mute"),
        FbState::Waiting if next => ("Up next".into(), "next"),
        FbState::Waiting => ("Waiting".into(), "mute"),
    }
}

/// Fallback Chain (affectUI): a request drawn as curves to every model it may use, in order –
/// a dotted light grows along the branch being tried, a failed branch breaks red with its reason,
/// the first model that answers is joined on to the response, later ones are never asked. The
/// header keeps a running clock and cost; every node opens a details card on hover, focus or click.
#[component]
pub fn FallbackChain(
    /// The models in fallback order (the demo's when `None`).
    #[props(default)]
    attempts: Option<Vec<FbAttempt>>,
    /// The request as the header title.
    #[props(default = "Summarise the support ticket".to_string())]
    request: String,
    /// The router the request went through (under "Request").
    #[props(default = "router/auto".to_string())]
    route: String,
    /// Demo: the chain plays out live and loops; otherwise it shows the settled result.
    #[props(default)]
    live: bool,
    /// Hand-over pause between two tries, ms.
    #[props(default = 300)]
    gap_ms: u64,
    /// The attempt whose details are open from the start.
    #[props(default)]
    open: Option<usize>,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Retry pressed (shown when every model failed).
    #[props(default)]
    on_retry: EventHandler<()>,
    /// An attempt's details opened (index) or closed (`None`).
    #[props(default)]
    on_open_change: EventHandler<Option<usize>>,
) -> Element {
    let mut t0 = use_signal(move || now);
    let mut sel = use_signal(move || open);
    let a = attempts.clone().unwrap_or_else(fb_demo);
    let n = a.len();
    let total = fb_total_ms(&a, gap_ms);
    let el = if live {
        let cycle = LEAD_MS + total + REST_MS;
        (now.since(t0()).as_millis() as u64 % cycle).saturating_sub(LEAD_MS)
    } else {
        u64::MAX / 4
    };
    let (states, _) = if live {
        fb_states(&a, el, gap_ms)
    } else {
        (fb_final(&a), true)
    };
    let (clock, cost) = fb_meter(&a, el, gap_ms);
    let prog = if live {
        fb_progress(&a, el, gap_ms)
    } else {
        None
    };
    let (pill, word) = fb_status(&states);
    let answered = states.iter().position(|s| *s == FbState::Answered);
    let all_failed = pill == "failed";
    let next = states
        .iter()
        .position(|s| *s == FbState::Trying)
        .map(|i| i + 1);
    let h = (n as f32 * PITCH - (PITCH - ROW)).max(ROW);
    let mid = h / 2.0;
    let yi = |i: usize| i as f32 * PITCH + ROW / 2.0;
    let mut toggle = move |i: usize| {
        let v = if sel() == Some(i) { None } else { Some(i) };
        sel.set(v);
        on_open_change.call(v);
    };
    let res_open = sel() == Some(n);

    rsx! {
        section { class: "oxfb", "data-style": style_attr(&style), "data-st": pill, "aria-label": "Fallback chain: {request}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { sel.set(None); },
            header { class: "oxfb-bar",
                h2 { class: "oxfb-ttl", "{request}" }
                span { class: "oxfb-sp" }
                span { class: "oxfb-st", "data-st": pill, role: "status", i { class: "oxfb-sdot" } "{word}" }
                span { class: "oxfb-meter", {svg_path("M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11zM8 5v3.2l2 1.3", 12)} "{fb_s(clock)}" }
                span { class: "oxfb-meter", "{fb_usd(cost)}" }
                if all_failed {
                    button { class: "oxfb-retry", r#type: "button",
                        onclick: move |_| { t0.set(now); sel.set(None); on_retry.call(()); },
                        {svg_path("M3 8a5 5 0 1 0 1.5-3.5M3 2.5V5h2.5", 12)} "Retry" }
                }
            }
            div { class: "oxfb-flow", style: "--h:{h}px",
                div { class: "oxfb-endw",
                    button { class: "oxfb-end oxfb-req", r#type: "button", "aria-expanded": sel() == Some(n + 1),
                        onclick: move |_| toggle(n + 1),
                        span { class: "oxfb-eic", {svg_path("M2.5 4.5h11v7h-11zM2.5 5l5.5 4 5.5-4", 13)} }
                        span { class: "oxfb-et", b { "Request" } small { "{route}" } }
                    }
                    div { class: "oxfb-pop", role: "tooltip", "data-open": sel() == Some(n + 1),
                        div { class: "oxfb-ph", b { "Request" } }
                        dl {
                            dt { "Route" } dd { "{route}" }
                            dt { "Models" } dd { "{n} in order" }
                            dt { "Asked" } dd { "{request}" }
                        }
                    }
                }
                svg { class: "oxfb-links", width: "{LINK_W}", height: "{h}", view_box: "0 0 {LINK_W} {h}", "aria-hidden": "true",
                    for (i, s) in states.iter().enumerate() {
                        {
                            let y = yi(i);
                            let d = format!("M0 {mid} C{} {mid} {} {y} {LINK_W} {y}", LINK_W * 0.5, LINK_W * 0.5);
                            let p = prog.filter(|(j, _)| *j == i).map(|(_, p)| (p * 100.0).clamp(4.0, 100.0));
                            rsx! {
                                path { class: "oxfb-ln", "data-st": s.id(), "data-i": "{i}", key: "{i}", d: "{d}", "pathLength": "100" }
                                if let Some(p) = p { path { class: "oxfb-light", d: "{d}", "pathLength": "100", style: "stroke-dasharray:{p:.1} 100" } }
                            }
                        }
                    }
                }
                ol { class: "oxfb-models",
                    for (i, (x, s)) in a.iter().zip(states.iter()).enumerate() {
                        {
                            let x = x.clone();
                            let s = *s;
                            let is_open = sel() == Some(i);
                            let (txt, tone) = line(&x, s, next == Some(i) || (i == 0 && s == FbState::Waiting));
                            let p = prog.filter(|(j, _)| *j == i).map(|(_, p)| p);
                            let took = match s {
                                FbState::Trying => p.map(|p| (x.ms as f64 * p) as u64),
                                FbState::Failed | FbState::Answered => Some(x.ms),
                                _ => None,
                            };
                            let spent = match s {
                                FbState::Trying => p.map(|p| x.cost * p),
                                FbState::Failed | FbState::Answered => Some(x.cost),
                                _ => None,
                            };
                            let tag = match s {
                                FbState::Failed => "Failed",
                                FbState::Answered => "Answered",
                                FbState::Trying => "Trying",
                                FbState::Skipped => "Not needed",
                                FbState::Waiting => "Waiting",
                            };
                            let lbl = format!("{}. {} on {} – {txt}", i + 1, x.model, x.provider);
                            rsx! {
                                li { class: "oxfb-m", "data-st": s.id(), "data-i": "{i}", key: "{i}", style: "--i:{i}",
                                    button { class: "oxfb-node", r#type: "button", "aria-expanded": is_open, "aria-label": "{lbl}",
                                        onclick: move |_| toggle(i),
                                        span { class: "oxfb-ord", "{i + 1}" }
                                        span { class: "oxfb-brand", BrandIcon { name: x.brand.clone(), size: 14.0, coloured: true, fallback: true } }
                                        span { class: "oxfb-mt",
                                            span { class: "oxfb-l1", b { "{x.model}" } span { class: "oxfb-prov", "{x.provider}" } }
                                            span { class: "oxfb-l2", "data-tone": tone,
                                                match s {
                                                    FbState::Failed => svg_path("M4 12 12 4M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z", 11),
                                                    FbState::Answered => svg_path("M3.5 8.5l3 3 6-7", 11),
                                                    FbState::Waiting if tone == "next" => svg_path("M3 8h10M9.5 4.5 13 8l-3.5 3.5", 11),
                                                    _ => rsx! {},
                                                }
                                                "{txt}"
                                            }
                                        }
                                        span { class: "oxfb-num",
                                            span { if let Some(t) = took { "{fb_s(t)}" } else { "–" } }
                                            span { if let Some(c) = spent { "{fb_usd(c)}" } else { "" } }
                                        }
                                    }
                                    div { class: "oxfb-pop", role: "tooltip", "data-open": is_open,
                                        div { class: "oxfb-ph", b { "{x.provider}" } span { class: "oxfb-tag", "data-st": s.id(), "{tag}" } }
                                        dl {
                                            dt { "Model" } dd { "{x.model}" }
                                            dt { "Time" } dd { if let Some(t) = took { "{fb_s(t)} of {fb_s(x.budget_ms)}" } else { "– of {fb_s(x.budget_ms)}" } }
                                            dt { "Cost" } dd { "{fb_usd(spent.unwrap_or(0.0))}" }
                                            if matches!(s, FbState::Failed | FbState::Answered) { dt { "Code" } dd { "{x.outcome.code()}" } }
                                            dt { "Price" } dd { "{x.price}" }
                                            if s == FbState::Failed { dt { "Why" } dd { class: "oxfb-why", "{x.outcome.reason()}" } }
                                            if s == FbState::Skipped { dt { "Why" } dd { "An earlier model answered" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                svg { class: "oxfb-links oxfb-out", width: "{LINK_W}", height: "{h}", view_box: "0 0 {LINK_W} {h}", "aria-hidden": "true",
                    if let Some(i) = answered {
                        path { class: "oxfb-ln oxfb-draw", "data-st": "answered", "pathLength": "100", d: "M0 {yi(i)} C{LINK_W * 0.5} {yi(i)} {LINK_W * 0.5} {mid} {LINK_W} {mid}" }
                    } else {
                        path { class: "oxfb-ln", "data-st": "waiting", d: "M0 {mid} L{LINK_W} {mid}" }
                    }
                }
                div { class: "oxfb-endw",
                    button { class: "oxfb-end oxfb-res", r#type: "button", "data-st": if answered.is_some() { "answered" } else if all_failed { "failed" } else { "waiting" }, "aria-expanded": res_open,
                        onclick: move |_| toggle(n),
                        span { class: "oxfb-eic", if answered.is_some() { {svg_path("M3.5 8.5l3 3 6-7", 13)} } else if all_failed { {svg_path("M4.5 4.5l7 7M11.5 4.5l-7 7", 13)} } else { {svg_path("M3 8h10M9.5 4.5 13 8l-3.5 3.5", 13)} } }
                        span { class: "oxfb-et",
                            b { "Response" }
                            small { if let Some(i) = answered { "{a[i].tokens} tokens" } else if all_failed { "None" } else { "Waiting" } }
                        }
                    }
                    div { class: "oxfb-pop oxfb-popr", role: "tooltip", "data-open": res_open,
                        div { class: "oxfb-ph", b { "Response" } }
                        dl {
                            if let Some(i) = answered {
                                dt { "From" } dd { "{a[i].model}" }
                                dt { "Tokens" } dd { "{a[i].tokens}" }
                                dt { "Took" } dd { "{fb_s(total)} with {i} fallback(s)" }
                                dt { "Spent" } dd { "{fb_usd(cost)}" }
                            } else if all_failed {
                                dt { "Status" } dd { class: "oxfb-why", "No model answered" }
                            } else {
                                dt { "Status" } dd { "Not back yet" }
                            }
                        }
                    }
                }
            }
        }
    }
}
