//! Blind compare (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::arena::{Arena, MODELS, SAMPLES, Side};

use super::switch;
use crate::aikit::{cls, dots, ico, key_of, live, mark};

fn side_icon(s: Side) -> (&'static str, &'static str) {
    match s {
        Side::Idle => ("info", "neutral"),
        Side::Thinking => ("brain", "blue"),
        Side::Writing => ("pencil", "blue"),
        Side::Done => ("check-circle", "green"),
        Side::Stopped => ("stop", "neutral"),
        Side::Failed => ("x-circle", "red"),
    }
}

/// Blind compare: one prompt goes to two models whose names stay hidden. Both answers stream side
/// by side at each model's own pace with first token time, speed and cost under them; the first
/// to finish is marked. Once both are in, vote (A, tie, B or both bad) and the models are
/// revealed with their marks; the leaderboard keeps win rates. On cards under 560 px one answer
/// shows at a time, chosen from tabs that carry each side's state.
#[component]
pub fn BlindCompare(
    /// Host clock.
    now: ClockMs,
    /// Run the first sample at once.
    #[props(default = true)]
    auto_play: bool,
    /// Hide model names until the vote.
    #[props(default = true)]
    blind: bool,
    /// Model B fails on its first run (error scenario).
    #[props(default)]
    fail: bool,
    /// Open with the leaderboard showing.
    #[props(default)]
    board: bool,
) -> Element {
    let mut ar = use_signal(move || {
        let mut a = Arena::new(auto_play, fail, blind, now.0);
        a.board_open = board;
        a
    });
    let mut said = use_signal(String::new);
    let mut text = use_signal(|| SAMPLES[0].1.to_string());
    let n = now.0;
    let a = ar.read().clone();
    let mut act = move |id: String| {
        let x = ar.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let running = a.running(n);
    let pair = a.pair();
    let first = a.first(n);
    let rev = a.revealed();
    let can_vote = a.can_vote(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-ar", "aria-label": "Blind compare",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { class: "oxaik-h", "Blind compare" }
                    p { class: "oxaik-sub", "Round {a.round + 1} · two models, one prompt" }
                }
                div { class: "oxaik-row oxbk-ar-tools",
                    label { class: "oxaik-row oxbk-ar-blind", span { "Blind" } {switch("Blind", a.blind, EventHandler::new(move |_| act("ar-blind".into())))} }
                    div { class: "oxbk-ar-bw",
                        button { class: "oxaik-btn sm", r#type: "button", "aria-expanded": "{a.board_open}", "aria-label": if a.vote.is_some() { "Leaderboard, updated" } else { "Leaderboard" },
                            onclick: move |_| act("ar-board".into()),
                            {ico("star", 12.0)} span { class: "oxbk-ar-bl", "Leaderboard" }
                        }
                        if a.board_open {
                            div { class: "oxbk-menu oxbk-ar-board", role: "dialog", "aria-label": "Leaderboard",
                                onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Escape" { let x = ar.write().key("Escape", n); if let Some(x) = x { said.set(x) } },
                                p { class: "oxbk-cap", "Win rate · top 5" }
                                ol { class: "oxbk-ar-lb",
                                    for (k, (m, wr, g)) in a.leaderboard().into_iter().enumerate() {
                                        li { key: "{m}", class: "oxbk-ar-lr",
                                            span { class: "oxaik-num oxbk-ar-rk", "{k + 1}" }
                                            {mark(MODELS[m].1, 14.0)}
                                            span { class: "oxbk-ar-ln", "{MODELS[m].0}" }
                                            span { class: "oxaik-bar oxbk-ar-lbar", "aria-hidden": "true", span { style: "width:{wr * 100.0}%" } }
                                            span { class: "oxaik-num", "{wr * 100.0:.0}% · {g} games" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxbk-ar-prompt",
                input { class: "oxaik-in", value: "{text}", "aria-label": "Prompt", "aria-keyshortcuts": "Control+Enter",
                    oninput: move |e| text.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| match key_of(&e).as_str() {
                        "Enter" => act("ar-run".into()),
                        "Escape" => { let x = ar.write().key("Escape", n); if let Some(x) = x { said.set(x) } }
                        _ => {}
                    },
                }
                button { class: if running { "oxaik-btn" } else { "oxaik-btn pri" }, r#type: "button", onclick: move |_| act("ar-run".into()),
                    {ico(if running { "stop" } else { "play" }, 12.0)} if running { "Stop" } else { "Run" }
                }
            }
            div { class: "oxaik-row oxbk-ar-samples",
                for (k, (label, p)) in SAMPLES.iter().enumerate() {
                    button { key: "{k}", class: "oxaik-chip", r#type: "button", disabled: running, "aria-pressed": "{a.sample == k}",
                        onclick: move |_| { text.set(p.to_string()); act(format!("ar-sample-{k}")) }, "{label}" }
                }
            }
            div { class: "oxbk-ar-tabs", role: "tablist", "aria-label": "Answers",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") { e.prevent_default(); let x = ar.write().key(&k, n); if let Some(x) = x { said.set(x) } }
                },
                for s in 0..2usize {
                    {
                        let st = a.side(s, n);
                        let (ic, hue) = side_icon(st);
                        rsx! {
                            button { key: "{s}", class: cls("oxbk-tab", a.tab == s), r#type: "button", role: "tab", "aria-selected": "{a.tab == s}", "aria-controls": "oxbk-ar-p{s}", tabindex: if a.tab == s { "0" } else { "-1" },
                                onclick: move |_| act(format!("ar-tab-{s}")),
                                if rev { {mark(MODELS[pair[s]].1, 13.0)} }
                                span { if s == 0 { "A" } else { "B" } }
                                span { class: "oxbk-st", "data-hue": "{hue}", {ico(ic, 11.0)} span { "{st.word()}" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxbk-ar-panes",
                for s in 0..2usize {
                    {
                        let st = a.side(s, n);
                        let (ic, hue) = side_icon(st);
                        let m = MODELS[pair[s]];
                        let letter = if s == 0 { "A" } else { "B" };
                        let label = if rev { format!("Model {letter}, {}", m.0) } else { format!("Model {letter}") };
                        let body = a.text(s, n);
                        rsx! {
                            section { key: "{s}", id: "oxbk-ar-p{s}", class: "oxbk-ar-pane", role: "region", "aria-label": "{label}", "data-on": "{a.tab == s}",
                                header { class: "oxbk-ar-ph",
                                    span { class: "oxbk-ar-who", "data-rev": "{rev}",
                                        if rev { {mark(m.1, 15.0)} span { class: "oxbk-ar-name", "{m.0}" } } else { span { class: "oxbk-ar-letter", "{letter}" } span { class: "oxbk-ar-name", "Model {letter}" } }
                                    }
                                    if first == Some(s) && st == Side::Done { span { class: "oxaik-pill", "data-hue": "green", {ico("bolt", 11.0)} "Finished first" } }
                                    span { class: "oxbk-st", "data-hue": "{hue}", {ico(ic, 11.0)} span { "{st.word()}" } }
                                }
                                div { class: "oxbk-ar-ans oxaik-scroll", tabindex: "0", "aria-busy": "{matches!(st, Side::Thinking | Side::Writing)}", "aria-label": "Answer from {label}",
                                    if st == Side::Failed {
                                        p { class: "oxbk-mcp-err", {ico("alert", 12.0)} "The model did not respond."
                                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ar-retry".into()), {ico("retry", 12.0)} "Retry" }
                                        }
                                    } else if st == Side::Thinking {
                                        p { class: "oxaik-mute oxaik-row", {dots()} span { class: "oxaik-shim", "Thinking" } }
                                    } else if st == Side::Idle {
                                        p { class: "oxaik-mute", "Run a prompt to compare." }
                                    } else {
                                        p { "{body}" }
                                    }
                                }
                                if let Some((ttft, tps, cost)) = a.stats(s, n) {
                                    p { class: "oxbk-ar-stats oxaik-num", span { "First token {ttft}" } span { "{tps}" } span { "{cost}" } }
                                } else {
                                    p { class: "oxbk-ar-stats oxaik-num", "aria-hidden": "true", span { "First token —" } }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(v) = a.vote {
                div { class: "oxbk-ar-result oxaik-fade",
                    {ico("check-circle", 13.0)}
                    span { "{v.words()}. A: " } {mark(MODELS[pair[0]].1, 13.0)} b { "{MODELS[pair[0]].0}" } span { " · B: " } {mark(MODELS[pair[1]].1, 13.0)} b { "{MODELS[pair[1]].0}" }
                    button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("ar-next".into()), "Next round" }
                }
            } else {
                div { class: "oxbk-ar-vote", role: "group", "aria-label": "Which answer is better?",
                    for (id, label, ic) in [("a", "A is better", "chevron-left"), ("tie", "Tie", "minus"), ("b", "B is better", "chevron-right"), ("bad", "Both bad", "x")] {
                        button { key: "{id}", class: "oxaik-btn", r#type: "button", disabled: !can_vote, onclick: move |_| act(format!("ar-vote-{id}")), {ico(ic, 12.0)} "{label}" }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
