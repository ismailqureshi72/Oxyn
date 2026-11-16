//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32): the exact **Agent Lanes**
//! ([`AgentLanes`]). Behaviour and look only; the model is `oxyn_core::parity::lanesfid`. Styles:
//! `lanesfid.css` (classes `oxln-*`), in `PRIMITIVES_CSS`. Follows the nearest `data-cstyle`
//! (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::lanesfid::{
        LANES_LOOP_MS, LaneScene, LaneStatus, LaneTask, block_label, block_span, clipped_left,
        lane_counts, lane_duration, lane_tag, lanes_at, lanes_said,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

/// Lane height and the curve drawing's width units.
const LANE_H: f32 = 50.0;
const VW: f32 = 1000.0;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

/// Agent Lanes (affectUI): where every agent is right now – a lane per agent over the last half
/// hour, tasks growing into "now", dashed hand-off curves between agents, a dark summary on a
/// click and Approve / Reject right on a waiting block.
#[component]
pub fn AgentLanes(
    /// Your own board (the demo's when `None`).
    #[props(default)]
    scene: Option<LaneScene>,
    /// Heading of the bar.
    #[props(default = "Agents".to_string())]
    title: String,
    /// Freeze the demo at this many ms into a pass (a settled state).
    #[props(default)]
    at: Option<u64>,
    /// A block open at the start (task id): its summary, or its decision when it waits.
    #[props(default)]
    open: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Approve on a waiting task (its id). Without a scene the lanes change it themselves.
    #[props(default)]
    on_approve: EventHandler<String>,
    /// Reject on a waiting task (its id).
    #[props(default)]
    on_reject: EventHandler<String>,
    /// The Open link in a summary (task id); shown when set.
    #[props(default)]
    on_open: Option<EventHandler<String>>,
) -> Element {
    let demo = scene.is_none();
    let start = use_signal(move || now);
    let mut opened = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let decided = use_signal(Vec::<(u64, String, bool, u64)>::new);

    let raw = now.since(start()).as_millis() as u64;
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / LANES_LOOP_MS, raw % LANES_LOOP_MS),
        _ => (0, raw),
    };
    let mut sc = scene.clone().unwrap_or_else(|| lanes_at(el));
    for (l, id, ok, when) in decided().iter() {
        if *l != lap {
            continue;
        }
        if let Some(t) = sc.tasks.iter_mut().find(|t| &t.id == id) {
            if *ok {
                t.status = LaneStatus::Running;
            } else {
                t.status = LaneStatus::Failed;
                t.end = Some((*when).min(sc.now));
            }
        }
    }
    let (running, waiting) = lane_counts(&sc.tasks);
    let said = lanes_said(&sc.tasks);
    let span = sc.span.max(1);
    let now_ms = sc.now;
    let lanes_n = sc.agents.len();
    let grid_h = lanes_n as f32 * LANE_H;
    // Ticks every 5 minutes back from now.
    let step = 5 * 60_000u64;
    let ticks: Vec<(f32, String)> = (1..)
        .map(|k| k * step)
        .take_while(|d| *d < span)
        .map(|d| (1.0 - d as f32 / span as f32, format!("−{}m", d / 60_000)))
        .collect();
    let lane_of = |agent: &str| sc.agents.iter().position(|a| a.id == agent);
    // Hand-off curves: (key, path, live).
    let mut curves: Vec<(String, String, bool)> = vec![];
    for t in sc.tasks.iter().filter(|t| !t.handoff_to.is_empty()) {
        let Some(n) = sc.tasks.iter().find(|x| x.id == t.handoff_to) else {
            continue;
        };
        let (Some(la), Some(lb)) = (lane_of(&t.agent), lane_of(&n.agent)) else {
            continue;
        };
        let (Some((l0, w0)), Some((l1, _))) =
            (block_span(t, now_ms, span), block_span(n, now_ms, span))
        else {
            continue;
        };
        let x0 = (l0 + w0) * VW;
        let x1 = l1 * VW;
        let y0 = la as f32 * LANE_H + LANE_H / 2.0;
        let y1 = lb as f32 * LANE_H + LANE_H / 2.0;
        let mx = (x0 + x1) / 2.0;
        curves.push((
            format!("{}-{}", t.id, n.id),
            format!(
                "M{x0:.1} {y0}C{:.1} {y0} {:.1} {y1} {x1:.1} {y1}",
                mx.max(x0 + 8.0),
                mx.min(x1 - 8.0)
            ),
            n.status == LaneStatus::Running,
        ));
    }
    let lap_k = format!("lap{lap}");

    rsx! {
        div { class: "oxln", "data-style": style_attr(&style), style: "--gh:{grid_h}px",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { opened.set(None) },
            div { class: "oxln-bar",
                span { class: "oxln-spark", Icon { name: "spark", size: 14.0, stroke: 1.8 } }
                b { class: "oxln-ttl", "{title}" }
                span { class: "oxln-cnt", "data-k": "running", i {} "{running} running" }
                if waiting > 0 { span { class: "oxln-cnt", "data-k": "waiting", i {} "{waiting} needs you" } }
                span { class: "oxln-sp" }
                span { class: "oxln-win", "Last {span / 60_000}m" }
            }
            div { class: "oxln-body",
                div { class: "oxln-ruler",
                    span { class: "oxln-rn" }
                    div { class: "oxln-rt",
                        for (x, l) in ticks.iter().cloned() {
                            span { key: "{l}", "data-k": "{l}", class: "oxln-tick", style: "left:{x * 100.0:.3}%", "{l}" }
                        }
                        span { class: "oxln-now", "now" }
                    }
                }
                div { class: "oxln-grid",
                    div { class: "oxln-names",
                        for a in sc.agents.iter().cloned() {
                            {
                                let tag = lane_tag(&a.id, &sc.tasks);
                                rsx! {
                                    div { key: "{a.id}", "data-k": "{a.id}", class: "oxln-who",
                                        span { class: "oxln-tile", title: "{a.name}", BrandIcon { name: if a.brand.is_empty() { a.name.clone() } else { a.brand.clone() }, size: 15.0, coloured: true, fallback: true } }
                                        span { class: "oxln-nm",
                                            span { class: "oxln-n1", b { "{a.name}" } if let Some(tg) = tag { span { class: "oxln-tag", "data-k": "{tg}", "{tg}" } } }
                                            span { class: "oxln-model", "{a.model}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxln-tl", key: "{lap_k}", "data-lap": "{lap_k}",
                        for (x, l) in ticks.iter().cloned() {
                            i { key: "g{l}", "data-k": "{l}", class: "oxln-gl", style: "left:{x * 100.0:.3}%" }
                        }
                        svg { class: "oxln-curves", view_box: "0 0 {VW} {grid_h}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                            for (k, d, live) in curves.iter().cloned() {
                                path { key: "{k}", "data-k": "{k}", class: "oxln-cv", "data-live": "{live}", d: "{d}" }
                            }
                        }
                        for (li, a) in sc.agents.iter().cloned().enumerate() {
                            ul { key: "{a.id}", "data-k": "{a.id}", class: "oxln-lane", "aria-label": "{a.name}", style: "top:{li as f32 * LANE_H}px",
                                for t in sc.tasks.iter().filter(|t| t.agent == a.id).cloned() {
                                    if let Some((l, w)) = block_span(&t, now_ms, span) {
                                        Block { key: "{t.id}", t: t.clone(), agent: a.name.clone(), l, w, clip: clipped_left(&t, now_ms, span), up: li * 2 >= lanes_n && lanes_n > 1,
                                            now_ms, lap, opened, decided, on_approve, on_reject, on_open }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxln-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

#[component]
#[allow(clippy::too_many_arguments)]
fn Block(
    t: LaneTask,
    agent: String,
    l: f32,
    w: f32,
    clip: bool,
    up: bool,
    now_ms: u64,
    lap: u64,
    opened: Signal<Option<String>>,
    decided: Signal<Vec<(u64, String, bool, u64)>>,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
    on_open: Option<EventHandler<String>>,
) -> Element {
    let mut opened = opened;
    let mut decided = decided;
    // Only a decision you open takes focus (one open from the start does not).
    let mut clicked = use_signal(|| false);
    let is_open = opened().as_deref() == Some(t.id.as_str());
    let label = block_label(&agent, &t, now_ms);
    let dur = lane_duration(t.end.unwrap_or(now_ms).min(now_ms).saturating_sub(t.start));
    let id = t.id.clone();
    let (ida, idr, ido) = (t.id.clone(), t.id.clone(), t.id.clone());
    let right = l + w > 0.6;
    let waits = t.status == LaneStatus::Waiting;
    rsx! {
        li { class: "oxln-li", style: "left:{l * 100.0:.3}%;width:{w * 100.0:.3}%", "data-open": "{is_open}",
            button { class: "oxln-b", r#type: "button", "data-st": "{t.status.id()}", "data-clip": "{clip}", "aria-label": "{label}", "aria-expanded": "{is_open}",
                title: "{t.title}", "aria-haspopup": if waits { "dialog" } else { "true" },
                onclick: move |_| { clicked.set(true); opened.set(if opened().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }) },
                for tool in t.tools.iter().take(3).cloned() {
                    span { key: "{tool}", "data-k": "{tool}", class: "oxln-tool", BrandIcon { name: tool.clone(), size: 11.0, coloured: true, fallback: true } }
                }
                span { class: "oxln-bt", "{t.title}" }
                if waits {
                    span { class: "oxln-need",
                        svg { width: "10", height: "10", view_box: "0 0 12 12", "aria-hidden": "true", path { d: "M6 1.2 11.2 10.4H.8z", fill: "currentColor" } }
                        "Needs you"
                    }
                }
            }
            if is_open && waits {
                div { class: "oxln-dec", role: "dialog", "aria-label": "Decide: {t.title}", "data-right": "{right}", "data-up": "{up}",
                    b { "{t.title}" }
                    if !t.ask.is_empty() { p { "{t.ask}" } }
                    div { class: "oxln-acts",
                        button { class: "oxln-btn", r#type: "button",
                            onclick: move |_| { decided.write().push((lap, idr.clone(), false, now_ms)); opened.set(None); on_reject.call(idr.clone()) },
                            "Reject" }
                        button { class: "oxln-btn", "data-k": "approve", r#type: "button",
                            onmounted: move |e| if clicked() { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                            onclick: move |_| { decided.write().push((lap, ida.clone(), true, now_ms)); opened.set(None); on_approve.call(ida.clone()) },
                            "Approve" }
                    }
                }
            } else if is_open {
                div { class: "oxln-tip", role: "tooltip", "data-right": "{right}", "data-up": "{up}",
                    span { class: "oxln-th", "{agent} · {t.status.label()} · {dur}" }
                    if !t.summary.is_empty() {
                        ul { for s in t.summary.iter().cloned() { li { key: "{s}", "data-k": "{s}", "{s}" } } }
                    }
                    if let Some(h) = on_open {
                        button { class: "oxln-open", r#type: "button", onclick: move |_| h.call(ido.clone()), "Open" }
                    }
                }
            }
        }
    }
}
