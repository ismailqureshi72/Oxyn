//! Generation stats (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::genstats::{GenStats, MODELS, Status};
use oxyn_core::aikit::{thousands, usd};

use super::{cls, dots, ico, live, mark};

fn spark(v: &[f64]) -> String {
    if v.len() < 2 {
        return String::new();
    }
    let max = v.iter().cloned().fold(1.0, f64::max);
    let step = 100.0 / (v.len() - 1) as f64;
    v.iter()
        .enumerate()
        .map(|(i, x)| {
            format!(
                "{}{:.1},{:.1}",
                if i == 0 { "M" } else { "L" },
                i as f64 * step,
                22.0 - x / max * 20.0
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generation stats: numbers for a reply while it streams. Delay to the first token, tokens per
/// second with a tiny sparkline, tokens out and spend climb in fixed-width cells next to the
/// model logo; when the reply ends they freeze and expand into where the time went. Stop holds the
/// numbers, an error keeps what arrived and offers Retry. `inline` is a compact chip; session
/// totals are an Oxyn extra.
#[component]
pub fn GenerationStats(
    /// Host clock.
    now: ClockMs,
    /// `stream` (starts on mount), `done`, `idle` or `error`.
    #[props(default = "stream".to_string())]
    scenario: String,
    /// `card` or `inline`.
    #[props(default = "card".to_string())]
    variant: String,
    /// Model (0 Claude Sonnet 4.5, 1 GPT-5, 2 Gemini 2.5 Flash).
    #[props(default)]
    model: usize,
    /// Replay after finishing.
    #[props(default)]
    looping: bool,
    /// Show session totals (Oxyn extra).
    #[props(default)]
    session: bool,
) -> Element {
    let t0 = now.0;
    let sc = scenario.clone();
    let mut gs = use_signal(move || {
        let mut g = GenStats::new(&sc, model, t0);
        g.looping = looping;
        g
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let g = gs.read().clone();
    let mut act = move |id: String| {
        let x = gs.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let st = g.status(n);
    let m = MODELS[g.model];
    let busy = matches!(st, Status::Waiting | Status::Streaming);
    let sp = g.speed(n);
    let path = spark(&g.spark(n));
    let inline = variant == "inline";
    let hue = match st {
        Status::Done => "green",
        Status::Error => "red",
        Status::Stopped => "amber",
        Status::Streaming | Status::Waiting => "blue",
        Status::Idle => "neutral",
    };
    let sess_t: u64 = g.session.iter().map(|s| s.0).sum::<u64>() + g.tokens(n);
    let sess_c: f64 = g.session.iter().map(|s| s.1).sum::<f64>() + g.cost(n);
    rsx! {
        section { class: "oxaik oxaik-gs", "data-variant": "{variant}", "aria-label": "Generation stats", "aria-busy": "{busy}",
            div { class: "oxaik-gs-strip",
                span { class: "oxaik-row oxaik-gs-model", {mark(m.1, 15.0)} if !inline { b { "{m.0}" } } }
                span { class: "oxaik-pill", "data-hue": "{hue}", if busy { {dots()} } else if st == Status::Done { {ico("check", 11.0)} } else if st == Status::Error { {ico("alert", 11.0)} } "{st.label()}" }
                dl { class: "oxaik-gs-stats oxaik-num",
                    div { dt { "TTFT" } dd { "{g.ttft(n)}" } }
                    div { class: "oxaik-gs-speed", dt { "Speed" } dd { "{sp:.0} tok/s" }
                        if !inline { svg { class: "oxaik-gs-spark", view_box: "0 0 100 24", preserve_aspect_ratio: "none", "aria-hidden": "true", path { d: "{path}" } } }
                    }
                    div { dt { "Output" } dd { "{thousands(g.tokens(n))}" } }
                    div { dt { "Cost" } dd { "{usd(g.cost(n))}" } }
                }
                div { class: "oxaik-row oxaik-gs-acts",
                    if busy {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("gs-stop".into()), {ico("stop", 11.0)} "Stop" }
                    } else if st == Status::Error {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("gs-retry".into()), {ico("retry", 12.0)} "Retry" }
                    } else {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("gs-run".into()), {ico("play", 11.0)} if st == Status::Idle { "Run" } else { "Run again" } }
                    }
                    if !inline {
                        button { class: cls("oxaik-ib", g.expanded), r#type: "button", "aria-expanded": "{g.expanded}", "aria-label": "Latency breakdown", onclick: move |_| act("gs-expand".into()), {ico("chevron", 13.0)} }
                    }
                }
            }
            if st == Status::Error { p { class: "oxaik-bad oxaik-row", role: "alert", {ico("alert", 12.0)} "Connection reset after {thousands(g.tokens(n))} tokens. What arrived is kept." } }
            if g.expanded && !inline {
                div { class: "oxaik-gs-break oxaik-fade",
                    for (lab, ms, share) in g.breakdown(n) {
                        div { key: "{lab}", class: "oxaik-gs-b",
                            span { class: "oxaik-mute", "{lab}" }
                            div { class: "oxaik-bar", "aria-hidden": "true", span { style: "width:{share * 100.0}%" } }
                            span { class: "oxaik-num", "{ms} ms" }
                        }
                    }
                    p { class: "oxaik-mute oxaik-num", "{g.summary(n)} · input {thousands(oxyn_core::aikit::genstats::INPUT_TOKENS)} tokens at {usd(m.2)}/1M, output at {usd(m.3)}/1M" }
                }
            }
            if session && !inline {
                p { class: "oxaik-mute oxaik-num oxaik-gs-sess", "This session: {g.session.len() + 1} replies · {thousands(sess_t)} tokens · {usd(sess_c)}" }
            }
            {live(&said())}
        }
    }
}
