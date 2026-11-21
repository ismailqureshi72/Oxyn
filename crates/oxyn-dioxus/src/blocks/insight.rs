//! AI insight card (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::insight::{ANSWERS, Insight, METRIC, POINTS, PREV, SERIES};
use oxyn_core::blocks::thousands;

use crate::aikit::{dots, ico, key_of, live, mark};

const W: f64 = 300.0;
const H: f64 = 80.0;

fn xy(i: usize, v: f64, lo: f64, hi: f64) -> (f64, f64) {
    let x = 6.0 + i as f64 / (SERIES.len() - 1) as f64 * (W - 12.0);
    let y = 6.0 + (1.0 - (v - lo) / (hi - lo)) * (H - 12.0);
    (x, y)
}

fn path(s: &[f64], lo: f64, hi: f64) -> String {
    s.iter()
        .enumerate()
        .map(|(i, &v)| {
            let (x, y) = xy(i, v, lo, hi);
            format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { "L" })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// AI insight card: a metric tile that can say why it moved. The value, its change against
/// the previous period (sign and colour) and a sparkline with the period before dashed under it.
/// Explain streams a short reason (the button reads Stop mid-stream and Hide once done) and ranks three drivers
/// with signed contribution bars; as the answer reaches a driver, its numbered marker lights up
/// on the chart at the point it explains. Suggested questions swap the answer in place; the
/// chart reads point by point with the arrow keys.
#[component]
pub fn AiInsightCard(
    /// Host clock.
    now: ClockMs,
    /// Start explaining at once.
    #[props(default = true)]
    auto_play: bool,
    /// The first answer fails (error scenario).
    #[props(default)]
    fail: bool,
    /// Show the previous period's line.
    #[props(default = true)]
    previous: bool,
) -> Element {
    let mut ins = use_signal(move || Insight::new(auto_play, fail, now.0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = ins.read().clone();
    let mut act = move |id: String| {
        let x = ins.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let lo = SERIES
        .iter()
        .chain(PREV.iter())
        .fold(f64::MAX, |a, &b| a.min(b))
        - 10.0;
    let hi = SERIES
        .iter()
        .chain(PREV.iter())
        .fold(f64::MIN, |a, &b| a.max(b))
        + 10.0;
    let d = c.delta();
    let streaming = c.streaming(n);
    let open = c.answer.is_some() || c.error;
    let reached = c.reached(n);
    let ans = c.answer.map(|a| a.0);
    let drivers = ans.map(|a| ANSWERS[a].1.drivers).unwrap_or(&[]);
    let dmax = drivers
        .iter()
        .map(|x| x.1.unsigned_abs())
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    let explain = if streaming {
        "Stop"
    } else if open {
        "Hide"
    } else {
        "Explain"
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-in", "aria-label": "{METRIC.0}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    p { class: "oxaik-sub", "{METRIC.0} · {METRIC.1}" }
                    p { class: "oxbk-in-val oxaik-row",
                        span { class: "oxaik-num", "{thousands(METRIC.2 as u64)}" }
                        span { class: "oxaik-pill oxaik-num", "data-hue": if d >= 0.0 { "green" } else { "red" }, {ico(if d >= 0.0 { "up" } else { "chevron" }, 11.0)} "{d:+.1}%" }
                        span { class: "oxaik-mute oxaik-num", "vs {thousands(METRIC.3 as u64)} the 12 weeks before" }
                    }
                }
                button { class: if open && !streaming { "oxaik-btn sm" } else { "oxaik-btn sm pri" }, r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-in-ans",
                    onclick: move |_| act("ai-explain".into()),
                    {ico(if streaming { "stop" } else if open { "x" } else { "spark" }, 12.0)} "{explain}"
                }
            }
            div { class: "oxbk-in-chart", tabindex: "0", role: "group", "aria-label": "Chart of {METRIC.0}; use the arrow keys to read each week",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End" | "Escape") {
                        e.prevent_default();
                        let x = ins.write().key(&k, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                svg { view_box: "0 0 {W} {H}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    if previous { path { class: "oxbk-in-prev", d: "{path(&PREV, lo, hi)}" } }
                    path { class: "oxbk-in-line", d: "{path(&SERIES, lo, hi)}" }
                    if let Some(p) = c.point {
                        {
                            let (x, y) = xy(p, SERIES[p], lo, hi);
                            rsx! { line { class: "oxbk-in-cross", x1: "{x}", x2: "{x}", y1: "0", y2: "{H}" } circle { class: "oxbk-in-dot", cx: "{x}", cy: "{y}", r: "3" } }
                        }
                    }
                }
                for (k, dr) in drivers.iter().enumerate() {
                    if k < reached {
                        {
                            let (x, y) = xy(dr.2, SERIES[dr.2], lo, hi);
                            rsx! { span { key: "{k}", class: "oxbk-in-mk oxaik-fade", "data-neg": "{dr.1 < 0}", style: "left:{x / W * 100.0}%;top:{y / H * 100.0}%", "{k + 1}" } }
                        }
                    }
                }
                if let Some(p) = c.point {
                    p { class: "oxbk-in-read oxaik-num", "{POINTS[p]}: {SERIES[p]} · before {PREV[p]}" }
                }
            }
            if previous { ul { class: "oxbk-legend oxbk-in-leg", li { i { class: "oxbk-in-l1" } "This period" } li { i { class: "oxbk-in-l2" } "12 weeks before" } } }
            div { id: "oxbk-in-ans", class: "oxbk-in-ans", hidden: !open,
                if c.error {
                    p { class: "oxbk-mcp-err", {ico("alert", 12.0)} "Couldn't explain the change: the model timed out."
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ai-retry".into()), {ico("retry", 12.0)} "Retry" }
                    }
                }
                if ans.is_some() {
                    p { class: "oxbk-in-text",
                        {mark("claude", 13.0)}
                        for (k, (t, dr)) in c.segments(n).into_iter().enumerate() {
                            span { key: "{k}", "{t} " if let Some(x) = dr { if x < reached { sup { class: "oxbk-in-ref", "{x + 1}" } } } }
                        }
                        if streaming { {dots()} }
                    }
                    ol { class: "oxbk-in-drivers", "aria-label": "Drivers",
                        for (k, dr) in drivers.iter().enumerate() {
                            {
                                let w = if k < reached { dr.1.unsigned_abs() as f64 / dmax * 100.0 } else { 0.0 };
                                rsx! {
                                    li { key: "{k}", class: "oxbk-in-dr", "data-on": "{k < reached}", "aria-hidden": "{k >= reached}",
                                        span { class: "oxbk-in-n oxaik-num", "{k + 1}" }
                                        span { class: "oxbk-in-dl", if !dr.3.is_empty() { {mark(dr.3, 12.0)} } span { "{dr.0}" } }
                                        span { class: "oxbk-in-bar", "data-neg": "{dr.1 < 0}", i { style: "width:{w}%" } }
                                        span { class: "oxaik-num", "data-hue": if dr.1 < 0 { "red" } else { "green" }, "{dr.1:+}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxaik-row oxbk-in-fu",
                        for (k, (label, _)) in ANSWERS.iter().enumerate().skip(1) {
                            button { key: "{k}", class: "oxaik-chip", r#type: "button", "aria-pressed": "{ans == Some(k)}", disabled: streaming,
                                onclick: move |_| act(format!("ai-follow-{k}")), "{label}" }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
