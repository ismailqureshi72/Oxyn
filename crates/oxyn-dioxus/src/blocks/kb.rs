//! Knowledge base (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::kb::{CHUNKS, Kb, PIPE, SUGGESTIONS, View};

use super::{status, switch};
use crate::aikit::{ico, key_of, live, mark, seg};

/// Text with the matched words wrapped in `mark`.
fn marked(text: &str, words: &[String]) -> Element {
    let parts: Vec<(String, bool)> = text
        .split_inclusive(' ')
        .map(|w| {
            let bare = w
                .trim()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            let hit = words.iter().any(|q| bare.starts_with(q.as_str()));
            (w.to_string(), hit)
        })
        .collect();
    rsx! {
        for (k, (w, hit)) in parts.into_iter().enumerate() {
            if hit { mark { key: "{k}", class: "oxbk-hl", "{w}" } } else { span { key: "{k}", "{w}" } }
        }
    }
}

fn kind_icon(kind: &str) -> Element {
    match kind {
        "pdf" => rsx! { span { class: "oxbk-kb-k", "data-k": "pdf", "PDF" } },
        "web" => ico("globe", 14.0),
        k => mark(k, 15.0),
    }
}

/// Knowledge base: build and probe a search index for an assistant. Sources run a four-step pipeline
/// (parse, chunk, embed, index) as a progress bar that names its step while pages and chunks
/// climb; a failure names its reason with Retry; Add source brings in a new one that ingests
/// live. Test retrieval runs a question (or a suggestion) and lists the top chunks with
/// similarity bars, scores, highlighted words and the source and page; Rerank reorders them in
/// place and says how far each moved; opening a result shows it between its neighbours.
#[component]
pub fn KnowledgeBase(
    /// Host clock.
    now: ClockMs,
    /// Start in Test retrieval with this suggestion asked (0–2).
    #[props(default)]
    ask: Option<usize>,
    /// Start with every source indexed (no ingestion or failure).
    #[props(default)]
    settled: bool,
    /// Rerank on at the start.
    #[props(default)]
    rerank: bool,
    /// Heading.
    #[props(default = "Support knowledge base".to_string())]
    title: String,
) -> Element {
    let mut kb = use_signal(move || {
        let mut k = Kb::new(settled || ask.is_some(), now.0);
        k.rerank = rerank;
        if let Some(a) = ask {
            k.search(SUGGESTIONS[a.min(2)], now.0);
        }
        k
    });
    let mut said = use_signal(String::new);
    let mut q = use_signal(|| {
        ask.map(|a| SUGGESTIONS[a.min(2)].to_string())
            .unwrap_or_default()
    });
    let n = now.0;
    let k = kb.read().clone();
    let mut act = move |id: String| {
        let x = kb.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut run = move || {
        let t = q.read().clone();
        let x = kb.write().search(&t, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let results = k.results(n);
    rsx! {
        section { class: "oxaik oxbk oxbk-kb", "aria-label": "{title}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "{title}" } p { class: "oxaik-sub oxaik-num oxbk-kb-sum", "{k.summary(n)}" } }
                button { class: "oxaik-btn sm", r#type: "button", disabled: k.next_add >= 3, onclick: move |_| act("kb-add".into()), {ico("plus", 12.0)} "Add source" }
            }
            div { class: "oxaik-row oxaik-between",
                {seg("View", vec!["Sources".into(), "Test retrieval".into()], usize::from(k.view == View::Test),
                    EventHandler::new(move |v| act(format!("kb-view-{v}"))),
                    EventHandler::new(move |key: String| { let x = kb.write().key(&key, n); if let Some(x) = x { said.set(x) } }))}
                div { class: "oxaik-row oxbk-kb-set",
                    span { class: "oxaik-mute", "Top" }
                    {seg("Top k", vec!["3".into(), "4".into(), "6".into()], match k.top_k { 3 => 0, 4 => 1, _ => 2 },
                        EventHandler::new(move |i: usize| act(format!("kb-k-{}", [3, 4, 6][i]))), EventHandler::new(move |_: String| {}))}
                    label { class: "oxaik-row oxbk-kb-sw", {switch("Hybrid search", k.hybrid, EventHandler::new(move |_| act("kb-hybrid".into())))} span { "Hybrid" } }
                }
            }
            if k.view == View::Sources {
                ul { class: "oxbk-kb-src",
                    for (i, s) in k.sources.iter().enumerate() {
                        {
                            let (step, _, failed) = k.pipe(i, n);
                            let pct = k.percent(i, n);
                            let txt = k.pipe_text(i, n);
                            rsx! {
                                li { key: "{s.name}", class: "oxbk-kb-row", "data-failed": "{failed}",
                                    span { class: "oxbk-kb-ic", {kind_icon(s.kind)} }
                                    div { class: "oxbk-kb-nm",
                                        b { title: "{s.name}", "{s.name}" }
                                        span { class: "oxaik-mute oxaik-num", "{s.pages} pages · {k.chunks(i, n)} chunks" }
                                    }
                                    div { class: "oxbk-kb-pipe", role: "progressbar", "aria-label": "{s.name}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{pct}", "aria-valuetext": "{txt}",
                                        for (p, lab) in PIPE.iter().enumerate() {
                                            {
                                                let f = if p < step { 1.0 } else if p == step && !failed { k.pipe(i, n).1 } else if p == step { 0.6 } else { 0.0 };
                                                let hue = if failed && p == step { "red" } else if step == 4 { "green" } else { "blue" };
                                                rsx! { span { key: "{p}", class: "oxbk-kb-seg", "data-hue": "{hue}", span { class: "oxbk-kb-tr", i { style: "width:{f * 100.0}%" } } small { "{lab}" } } }
                                            }
                                        }
                                    }
                                    div { class: "oxbk-kb-act",
                                        if failed {
                                            {status("Failed", "red")}
                                            button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("kb-retry-{i}")), {ico("retry", 12.0)} "Retry" }
                                        } else if step == 4 {
                                            {status("Indexed", "green")}
                                        } else {
                                            span { class: "oxaik-shim", "{PIPE[step]} · {pct}%" }
                                        }
                                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Remove {s.name}", onclick: move |_| act(format!("kb-remove-{i}")), {ico("trash", 12.0)} }
                                    }
                                    if failed { p { class: "oxbk-kb-err", {ico("alert", 12.0)} "{s.error}" } }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxbk-kb-test",
                    div { class: "oxaik-row oxbk-kb-q",
                        input { class: "oxaik-in", r#type: "search", value: "{q}", placeholder: "Ask a question to test retrieval", "aria-label": "Question",
                            oninput: move |e| q.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| match key_of(&e).as_str() {
                                "Enter" => run(),
                                "Escape" => { q.set(String::new()); act("kb-clear".into()) }
                                _ => {}
                            },
                        }
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| run(), {ico("search", 12.0)} "Search" }
                        label { class: "oxaik-row oxbk-kb-sw", {switch("Rerank", k.rerank, EventHandler::new(move |_| act("kb-rerank".into())))} span { "Rerank" } }
                    }
                    if k.query.is_none() {
                        div { class: "oxaik-row oxbk-kb-sugs",
                            for (s, t) in SUGGESTIONS.iter().enumerate() {
                                button { key: "{s}", class: "oxaik-chip", r#type: "button", onclick: move |_| { q.set(SUGGESTIONS[s].to_string()); act(format!("kb-sug-{s}")) }, "{t}" }
                            }
                        }
                    } else if results.is_empty() {
                        p { class: "oxaik-mute oxbk-empty", "No chunk matches. Try other words or turn on hybrid search." }
                    }
                    ol { class: "oxbk-kb-res",
                        for (r, h) in results.iter().enumerate() {
                            {
                                let (src, page, text) = CHUNKS[h.chunk];
                                let open = k.open == Some(h.chunk);
                                let moved = h.was as i64 - (r as i64 + 1);
                                let c = h.chunk;
                                let src_name = k.sources.get(src).map_or("", |s| s.name);
                                let mv = if k.rerank && moved != 0 { format!(", {} {}, was {}", if moved > 0 { "up" } else { "down" }, moved.abs(), h.was) } else { String::new() };
                                let (before, after) = Kb::neighbours(c);
                                let mvt = if moved > 0 { format!("↑{moved}") } else { format!("↓{}", -moved) };
                                let mvh = if moved > 0 { "green" } else { "neutral" };
                                rsx! {
                                    li { key: "{c}", class: "oxbk-kb-hit",
                                        button { class: "oxbk-kb-hb", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-kb-c-{c}",
                                            "aria-label": "Result {r + 1}, score {h.score:.2}{mv}, {src_name} page {page}",
                                            onclick: move |_| act(format!("kb-open-{c}")),
                                            span { class: "oxbk-kb-rank oxaik-num", "{r + 1}" }
                                            span { class: "oxbk-kb-score",
                                                span { class: "oxaik-bar", span { style: "width:{h.score * 100.0}%" } }
                                                span { class: "oxaik-num", "{h.score:.2}" }
                                                if k.rerank && moved != 0 { span { class: "oxbk-kb-mv oxaik-num", "data-hue": "{mvh}", "{mvt}" } }
                                            }
                                            span { class: "oxbk-kb-txt", {marked(text, &h.words)} }
                                            span { class: "oxaik-mute oxbk-kb-loc", "{src_name} · p. {page}" }
                                        }
                                        if open {
                                            div { class: "oxbk-kb-ctx oxaik-fade", id: "oxbk-kb-c-{c}", role: "region", "aria-label": "Chunk in context",
                                                if let Some(b) = before { p { class: "oxaik-mute", "{CHUNKS[b].2}" } }
                                                p { class: "oxbk-kb-cur", "{text}" }
                                                if let Some(a) = after { p { class: "oxaik-mute", "{CHUNKS[a].2}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
