//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Plan vs Actual**
//! ([`PlanVsActual`]). Behaviour and look only; the model is `oxyn_core::parity::planfid`.
//! Styles: `planfid.css` (classes `oxpv-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::planfid::{
        PvState, PvStep, pv_actual_total, pv_axis, pv_cost, pv_delta, pv_demo, pv_eta, pv_fmt,
        pv_plan_total, pv_rows,
    },
};

/// Row pitch, px (the dependency links use it).
const ROW: f32 = 28.0;
/// Demo: one run-second per this many ms.
const TICK_MS: u64 = 120;
/// Rest at the end before the demo loops, run-seconds.
const REST_S: u32 = 40;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn pct(s: u32, axis: u32) -> f32 {
    s as f32 / axis.max(1) as f32 * 100.0
}

/// Plan vs Actual (affectUI parity): a run laid over its plan. Real bars sit on dashed plan bars,
/// time past a step's plan is drawn amber, later steps shift to their new expected start and the
/// ETA drifts from the planned finish. Select a step to see its links and cost.
#[component]
pub fn PlanVsActual(
    /// The plan with each step's real duration (the demo's when `None`).
    #[props(default)]
    steps: Option<Vec<PvStep>>,
    /// Seconds into the run; `None` plays the demo (looping).
    #[props(default)]
    at: Option<u32>,
    /// The task.
    #[props(default = "Launch the pricing page".to_string())]
    task: String,
    /// Heading.
    #[props(default = "Plan vs actual".to_string())]
    title: String,
    /// A step selected from the start (id).
    #[props(default)]
    selected: Option<String>,
    /// The clock (drives the demo).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A step was selected (id) or cleared.
    #[props(default)]
    on_select: EventHandler<Option<String>>,
) -> Element {
    let t0 = use_signal(move || now);
    let mut sel = use_signal(move || selected.clone());
    let s = steps.clone().unwrap_or_else(pv_demo);
    let total = pv_actual_total(&s);
    let t = at.unwrap_or_else(|| {
        let el = (now.since(t0()).as_millis() as u64 / TICK_MS) as u32;
        (el % (total + REST_S)).min(total)
    });
    let rows = pv_rows(&s, t);
    let plan_total = pv_plan_total(&s);
    let eta = pv_eta(&rows);
    let axis = pv_axis(plan_total, pv_eta(&pv_rows(&s, total)).max(eta));
    let (spent, plan_cost) = pv_cost(&s, &rows, t);
    let behind = eta as i64 - plan_total as i64;
    let done = rows.iter().all(|r| r.state == PvState::Done);
    let shown_now = t.min(eta);
    let ticks: Vec<u32> = (1..=axis / 30).map(|k| k * 30).collect();
    let n = s.len();
    let h = n as f32 * ROW;
    let sel_i = sel().and_then(|id| s.iter().position(|x| x.id == id));
    // Links for the selected step: from each dependency's end to its start, and from its end to
    // each step waiting on it.
    let mut links: Vec<(f32, f32, f32, f32)> = vec![];
    if let Some(i) = sel_i {
        for d in &s[i].deps {
            if let Some(j) = s.iter().position(|x| x.id == *d) {
                links.push((
                    pct(rows[j].end, axis),
                    j as f32 * ROW + ROW / 2.0,
                    pct(rows[i].start, axis),
                    i as f32 * ROW + ROW / 2.0,
                ));
            }
        }
        for (k, x) in s.iter().enumerate() {
            if x.deps.contains(&s[i].id) {
                links.push((
                    pct(rows[i].end, axis),
                    i as f32 * ROW + ROW / 2.0,
                    pct(rows[k].start, axis),
                    k as f32 * ROW + ROW / 2.0,
                ));
            }
        }
    }
    let status = if done && behind <= 0 {
        format!(
            "{} ahead",
            pv_delta(-behind)
                .trim_start_matches('+')
                .trim_start_matches('−')
        )
    } else if behind > 0 {
        format!("{} behind", pv_delta(behind))
    } else {
        "On plan".to_string()
    };
    let tone = if behind > 0 { "late" } else { "ok" };
    let spent_s = format!("${spent:.2}");
    let plan_cost_s = format!("/ ${plan_cost:.2}");

    rsx! {
        section { class: "oxpv", "data-style": style_attr(&style), "data-done": done, "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { sel.set(None); on_select.call(None); },
            header { class: "oxpv-bar",
                span { class: "oxpv-logo", {icon("M10.5 2.5a3 3 0 0 0-2.8 4.1L2.5 11.8l1.7 1.7 5.2-5.2a3 3 0 0 0 4.1-2.8l-1.8 1.8-1.7-.4-.4-1.7z", 15)} }
                h2 { class: "oxpv-ttl", "{title}" }
                span { class: "oxpv-task", "{task}" }
                span { class: "oxpv-sp" }
                span { class: "oxpv-kv", span { "Time" } b { "{pv_fmt(shown_now)}" } i { "/ {pv_fmt(plan_total)}" } }
                span { class: "oxpv-kv", span { "Cost" } b { "{spent_s}" } i { "{plan_cost_s}" } }
                span { class: "oxpv-pill", "data-tone": tone, role: "status", "{status}" }
            }
            div { class: "oxpv-grid", style: "--axis-n:{ticks.len()}",
                div { class: "oxpv-head",
                    span { class: "oxpv-stepcap", "Step" }
                    div { class: "oxpv-axis", "aria-hidden": "true",
                        for k in ticks.iter() { span { key: "{k}", "data-k": "{k}", style: "left:{pct(*k, axis)}%", "{pv_fmt(*k)}" } }
                    }
                }
                div { class: "oxpv-body", style: "--h:{h}px",
                    ol { class: "oxpv-rows",
                        for (i, (x, r)) in s.iter().zip(rows.iter()).enumerate() {
                            {
                                let id = x.id.clone();
                                let on = sel_i == Some(i);
                                let took = match r.state {
                                    PvState::Done => format!("took {}, planned {}", pv_fmt(r.end - r.start), pv_fmt(x.plan)),
                                    PvState::Running => format!("running {}, planned {}", pv_fmt(t - r.start), pv_fmt(x.plan)),
                                    PvState::Waiting => format!("expected to start at {}", pv_fmt(r.start)),
                                };
                                let cost_s = format!("· ${:.2}", x.cost);
                                let over = r.state != PvState::Waiting && r.delta > 0;
                                let in_plan = (r.end - r.start).min(x.plan);
                                let shown_end = if r.state == PvState::Running { t } else { r.end };
                                let el = shown_end.saturating_sub(r.start);
                                let lab_at = shown_end.max(r.plan_start + x.plan).max(if r.state == PvState::Running { r.start + x.plan } else { 0 });
                                let in_w = pct(el.min(in_plan), axis);
                                let over_w = pct(el.saturating_sub(x.plan), axis);
                                rsx! {
                                    li { class: "oxpv-row", key: "{x.id}", "data-k": "{x.id}", "data-st": r.state.id(), "data-sel": on,
                                        button { class: "oxpv-hit", r#type: "button", "aria-pressed": on, "aria-label": "{x.title}: {took}",
                                            onclick: move |_| { let v = if sel().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }; sel.set(v.clone()); on_select.call(v); },
                                            span { class: "oxpv-gl", "data-st": r.state.id(),
                                                match r.state {
                                                    PvState::Done => icon("M3.5 8.5l3 3 6-7", 12),
                                                    PvState::Running => rsx! { i { class: "oxpv-spin" } },
                                                    PvState::Waiting => rsx! { i { class: "oxpv-ring" } },
                                                }
                                            }
                                            span { class: "oxpv-name", "{x.title}" }
                                        }
                                        div { class: "oxpv-lane",
                                            span { class: "oxpv-ghost", style: "left:{pct(r.plan_start, axis)}%;width:{pct(x.plan, axis)}%" }
                                            if r.state != PvState::Waiting {
                                                span { class: "oxpv-act", "data-st": r.state.id(), style: "left:{pct(r.start, axis)}%;width:{in_w + over_w}%",
                                                    i { class: "oxpv-in", style: "width:{in_w / (in_w + over_w).max(0.001) * 100.0}%" }
                                                    if over { i { class: "oxpv-over" } }
                                                }
                                                span { class: "oxpv-val", style: "left:calc({pct(lab_at, axis)}% + 6px)",
                                                    "{pv_fmt(el)}"
                                                    if r.delta != 0 && (r.state == PvState::Done || over) {
                                                        em { "data-tone": if r.delta > 0 { "late" } else { "ok" }, "{pv_delta(r.delta)}" }
                                                    }
                                                    if on { span { class: "oxpv-cost", "{cost_s}" } }
                                                }
                                            } else {
                                                span { class: "oxpv-exp", style: "left:{pct(r.start, axis)}%;width:{pct(x.plan, axis)}%" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxpv-marks", "aria-hidden": "true",
                        for k in ticks.iter() { i { class: "oxpv-gridl", key: "{k}", "data-k": "{k}", style: "left:{pct(*k, axis)}%" } }
                        if eta > plan_total { span { class: "oxpv-gap", style: "left:{pct(plan_total, axis)}%;width:{pct(eta - plan_total, axis)}%" } }
                        i { class: "oxpv-plan", style: "left:{pct(plan_total, axis)}%" }
                        if !done { i { class: "oxpv-now", style: "left:{pct(t, axis)}%" } }
                        if !links.is_empty() {
                            svg { class: "oxpv-links", view_box: "0 0 100 {h}", preserve_aspect_ratio: "none",
                                for (k, (x1, y1, x2, y2)) in links.iter().enumerate() {
                                    path { key: "{k}", "data-k": "{k}", d: "M{x1} {y1} C{x1 + 3.0} {y1} {x2 - 3.0} {y2} {x2} {y2}", "vector-effect": "non-scaling-stroke" }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "oxpv-foot",
                span { class: "oxpv-leg", i { "data-k": "plan" } "Plan" }
                span { class: "oxpv-leg", i { "data-k": "act" } "Actual" }
                span { class: "oxpv-leg", i { "data-k": "over" } "Over plan" }
                span { class: "oxpv-sp" }
                span { class: "oxpv-chip oxpv-planc", "Plan {pv_fmt(plan_total)}" }
                span { class: "oxpv-chip oxpv-eta", "data-tone": tone, "ETA {pv_fmt(eta)}" }
            }
        }
    }
}
