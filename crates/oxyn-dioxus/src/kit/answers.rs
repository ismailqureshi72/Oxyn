//! affectUI fidelity pass, overnight batch, family A (plan #32): exact rebuilds of affectUI's
//! **AI Answer** ([`AnswerCard`]), **Cited Answer** ([`CitedAnswerCard`]), **Message Actions**
//! ([`MessageActionBar`]), **Thinking pill** ([`ThinkingPill`]), **Thinking panel**
//! ([`ThinkingPanel`]) and **Writing Assistant** ([`WritingAssistant`]). Behaviour and look only;
//! models in `oxyn_core::kit::answers`. Styles: `answers.css` (classes `oxaa-*`, `oxca-*`,
//! `oxma-*`, `oxtl-*`, `oxts-*`, `oxwa-*`), in `PRIMITIVES_CSS`. Each follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set. Time comes in as `now`; nothing reads a
//! clock, so a host drives the streams.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::answers::{
        ANSWER_REASONS, AnswerDoc, AnswerPhase, AnswerSource, Block, CitedSource, DICTATION,
        DICTATION_CPS, LOADER_INTERVAL, LOADER_PHASES, MESSAGE_REASONS, REGENERATE_MS, Seg,
        THINK_STEP_MS, ThinkStep, answer_phase, answer_words_at, backed, cited_in, demo_answer,
        demo_cited, demo_message, demo_think_steps, dictated_at, loader_phase, parse_cited,
        quote_runs, reply_at, sample_reply, sentences_citing, think_progress, thought_for,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn ms(now: ClockMs, then: ClockMs) -> u64 {
    now.since(then).as_millis() as u64
}

/// Stroke paths this file needs beyond the shared icon set.
fn path(name: &str) -> &'static str {
    match name {
        "up" => {
            "M7 10v10M7 10l4-7c1.2 0 2 1 2 2.2V9h5.2a2 2 0 0 1 2 2.3l-1.2 7A2 2 0 0 1 17 20H7M7 10H4.5A1.5 1.5 0 0 0 3 11.5v7A1.5 1.5 0 0 0 4.5 20H7"
        }
        "down" => {
            "M17 14V4M17 14l-4 7c-1.2 0-2-1-2-2.2V15H5.8a2 2 0 0 1-2-2.3l1.2-7A2 2 0 0 1 7 4h10M17 14h2.5a1.5 1.5 0 0 0 1.5-1.5v-7A1.5 1.5 0 0 0 19.5 4H17"
        }
        "speaker" => {
            "M4 9.5h3.5L12 5.5v13l-4.5-4H4zM16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11"
        }
        "spark" => {
            "M12 3.5l1.6 4.9 4.9 1.6-4.9 1.6L12 16.5l-1.6-4.9L5.5 10l4.9-1.6zM18.5 15l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7z"
        }
        "arrow" => "M5 12h14M13 6l6 6-6 6",
        "globe" => {
            "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3.5 12h17M12 3c2.5 2.6 3.6 5.6 3.6 9s-1.1 6.4-3.6 9c-2.5-2.6-3.6-5.6-3.6-9S9.5 5.6 12 3z"
        }
        "sparkle" => {
            "M12 2.5c.6 4.6 2.9 6.9 7.5 7.5-4.6.6-6.9 2.9-7.5 7.5-.6-4.6-2.9-6.9-7.5-7.5 4.6-.6 6.9-2.9 7.5-7.5z"
        }
        _ => "",
    }
}

#[component]
fn Stroke(name: String, #[props(default = 16.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxaa-g", width: "{size}", height: "{size}", view_box: "0 0 24 24", fill: "none", stroke: "currentColor",
            stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "{path(&name)}" }
        }
    }
}

/// A coloured brand tile (letter when the brand has no mark).
#[component]
fn Tile(brand: String, #[props(default = 16.0)] size: f32) -> Element {
    let inner = size * 0.72;
    rsx! {
        span { class: "oxaa-tile", style: "width:{size}px;height:{size}px", "aria-hidden": "true",
            BrandIcon { name: brand, size: inner, coloured: true, fallback: true }
        }
    }
}

// ───────────────────────────────────────────────────────────────────── AI Answer ──

/// AI Answer (affectUI): a streamed answer card. The question on top, three source cards with
/// coloured brand marks and a "+N more" tile, a status line with a blue spark ("Reading 6 sources"
/// → "Writing" → "Answer"), the answer streaming word by word with citation chips that preview
/// every source they cite, a RELATED block of follow-ups, and a footer with Copy (a green check
/// and "Copied"), Regenerate, thumbs up (thanks) and thumbs down (reason chips), plus "Answered in
/// 3.4s". While it writes the footer holds one Stop button. Clean: one card with a hairline footer;
/// Classic: source cards grouped in one grey band, a blue agent badge, arrows and a grey footer.
#[component]
pub fn AnswerCard(
    /// Question heading (the demo when empty).
    #[props(default)]
    question: String,
    /// Sources; the first three are cards, the rest add to "+N".
    #[props(default)]
    sources: Vec<AnswerSource>,
    /// Sources beyond the list, counted in the "+N more" tile.
    #[props(default)]
    more: usize,
    /// Answer markup (see `AnswerDoc::parse`).
    #[props(default)]
    text: String,
    /// Follow-up prompts under RELATED.
    #[props(default)]
    related: Vec<String>,
    /// Footer time, e.g. "3.4s".
    #[props(default = "2.9s".to_string())]
    took: String,
    /// Stream from the start (else show the finished answer).
    #[props(default = true)]
    autoplay: bool,
    /// Words a second.
    #[props(default = 38)]
    wps: u64,
    /// Reasons offered after a thumbs-down.
    #[props(default)]
    reasons: Vec<String>,
    /// Start with a thumbs-down and its reasons (`up`, `down` or empty).
    #[props(default)]
    vote: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// (`copy` | `regenerate` | `stop` | `vote` | `reason` | `follow-up`, detail).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let demo = use_hook(demo_answer);
    let question = if question.is_empty() {
        demo.0.clone()
    } else {
        question
    };
    let sources = if sources.is_empty() {
        demo.1.clone()
    } else {
        sources
    };
    let more = if more == 0 && text.is_empty() {
        demo.2
    } else {
        more
    };
    let text = if text.is_empty() {
        demo.3.clone()
    } else {
        text
    };
    let related = if related.is_empty() {
        demo.4.clone()
    } else {
        related
    };
    let reasons: Vec<String> = if reasons.is_empty() {
        ANSWER_REASONS.iter().map(|s| s.to_string()).collect()
    } else {
        reasons
    };
    let doc = use_memo(use_reactive!(|text| AnswerDoc::parse(&text)));
    let total = doc.read().words();
    let mut start = use_signal(move || if autoplay { Some(now) } else { None });
    let mut stopped = use_signal(|| None::<usize>);
    let init_vote = match vote.as_str() {
        "up" => 1u8,
        "down" => 2,
        _ => 0,
    };
    let mut voted = use_signal(move || init_vote);
    let mut reason = use_signal(|| None::<usize>);
    let mut copied = use_signal(|| None::<ClockMs>);
    let mut tip = use_signal(|| None::<usize>);
    let el = start().map(|s| ms(now, s)).unwrap_or(u64::MAX / 4);
    let phase = if stopped().is_some() {
        AnswerPhase::Stopped
    } else {
        answer_phase(el, total, wps)
    };
    let words = stopped().unwrap_or_else(|| answer_words_at(el, total, wps));
    let shown = doc.read().upto(words);
    let writing = matches!(phase, AnswerPhase::Reading | AnswerPhase::Writing);
    let cards: Vec<(usize, AnswerSource)> = sources.iter().cloned().enumerate().take(3).collect();
    let extra = more + sources.len().saturating_sub(3);
    let cards_in = if phase == AnswerPhase::Reading {
        (el / 300 + 1) as usize
    } else {
        9
    };
    let status = match phase {
        AnswerPhase::Reading => format!("Reading {} sources", sources.len() + more),
        AnswerPhase::Writing => "Writing".to_string(),
        AnswerPhase::Done => "Answer".to_string(),
        AnswerPhase::Stopped => "Stopped".to_string(),
    };
    let is_copied = copied().is_some_and(|c| ms(now, c) < 1600);
    let foot = if is_copied {
        "Copied".to_string()
    } else if voted() == 2 && reason().is_none() {
        "What went wrong?".to_string()
    } else if voted() != 0 {
        "Thanks for the feedback".to_string()
    } else if phase == AnswerPhase::Stopped {
        format!("Stopped at {words} words")
    } else {
        format!("Answered in {took}")
    };
    let srcs = sources.clone();
    let chip = move |key: usize, ids: Vec<usize>| -> Element {
        let first = ids
            .first()
            .and_then(|i| srcs.get(*i))
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let label = if ids.len() > 1 {
            format!("{first} +{}", ids.len() - 1)
        } else {
            first
        };
        let rows: Vec<(usize, AnswerSource)> = ids
            .iter()
            .filter_map(|i| srcs.get(*i).cloned().map(|s| (*i, s)))
            .collect();
        let name = rows
            .iter()
            .map(|(_, s)| s.title.clone())
            .collect::<Vec<_>>()
            .join(", ");
        rsx! {
            span { class: "oxaa-cw",
                onmouseenter: move |_| tip.set(Some(key)), onmouseleave: move |_| tip.set(None),
                button { class: "oxaa-cite", r#type: "button", "aria-label": "Source: {name}", "aria-describedby": if tip() == Some(key) { Some(format!("oxaa-t{key}")) } else { None },
                    onfocus: move |_| tip.set(Some(key)), onblur: move |_| tip.set(None),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { tip.set(None) },
                    "{label}" }
                if tip() == Some(key) {
                    span { class: "oxaa-tip", role: "tooltip", id: "oxaa-t{key}",
                        for (i, s) in rows {
                            a { key: "{i}", class: "oxaa-tr", href: "{s.url}", target: "_blank", rel: "noreferrer",
                                span { class: "oxaa-trh", Tile { brand: s.brand.clone(), size: 16.0 } span { class: "oxaa-dom", "{s.domain()}" } span { class: "oxaa-sp" } span { class: "oxaa-ix", "{i + 1}" } }
                                span { class: "oxaa-trt", "{s.title}" }
                            }
                        }
                    }
                }
            }
        }
    };
    let segs = move |line: &[Seg], base: usize| -> Element {
        rsx! {
            for (j, s) in line.iter().cloned().enumerate() {
                match s {
                    Seg::Word(w) => rsx! { span { key: "{j}", "{w} " } },
                    Seg::Bold(w) => rsx! { b { key: "{j}", "{w} " } },
                    Seg::Cite(ids) => rsx! { span { key: "{j}", {chip(base * 100 + j, ids)} " " } },
                }
            }
        }
    };
    let blocks = shown.blocks.clone();
    let nb = blocks.len();
    rsx! {
        div { class: "oxaa", "data-style": style_attr(&style), "data-phase": if writing { "writing" } else { "done" },
            div { class: "oxaa-card",
                h3 { class: "oxaa-q", "{question}" }
                ul { class: "oxaa-src", "aria-label": "Sources",
                    for (i, s) in cards.into_iter().filter(|(i, _)| *i < cards_in) {
                            li { key: "{i}", class: "oxaa-sc",
                                a { href: "{s.url}", target: "_blank", rel: "noreferrer", "aria-label": "{s.name}: {s.title}",
                                    span { class: "oxaa-sh", Tile { brand: s.brand.clone(), size: 16.0 } span { class: "oxaa-sn", "{s.name}" } span { class: "oxaa-sp" } span { class: "oxaa-ix", "{i + 1}" } }
                                    span { class: "oxaa-stt", "{s.title}" }
                                }
                            }
                    }
                    if extra > 0 && cards_in > 3 {
                        li { class: "oxaa-more", "aria-label": "{extra} more sources", b { "+{extra}" } span { "more" } }
                    }
                }
                div { class: "oxaa-status", "data-live": "{writing}",
                    span { class: "oxaa-spark", Stroke { name: "spark", size: 15.0 } }
                    span { class: "oxaa-badge", Icon { name: "agents", size: 12.0 } }
                    span { class: "oxaa-slw", span { key: "{status}", class: "oxaa-sl", "{status}" } }
                }
                div { class: "oxaa-body", "aria-live": "polite", "aria-busy": "{writing}",
                    for (k, b) in blocks.into_iter().enumerate() {
                        match b {
                            Block::Para(line) => rsx! { p { key: "{k}", {segs(&line, k * 10)} if writing && k == nb - 1 { span { class: "oxaa-caret", "aria-hidden": "true" } } } },
                            Block::Bullets(items) => {
                                let ni = items.len();
                                rsx! { ul { key: "{k}", for (m, it) in items.into_iter().enumerate() { li { key: "{m}", {segs(&it, k * 10 + m + 1)} if writing && k == nb - 1 && m == ni - 1 { span { class: "oxaa-caret", "aria-hidden": "true" } } } } } }
                            }
                        }
                    }
                }
                if !writing && phase != AnswerPhase::Stopped && !related.is_empty() {
                    div { class: "oxaa-rel",
                        div { class: "oxaa-rl", "Related" }
                        for r in related.iter().cloned() {
                            button { key: "{r}", class: "oxaa-fu", r#type: "button", onclick: { let r = r.clone(); move |_| on_action.call(("follow-up".into(), r.clone())) },
                                span { "{r}" }
                                span { class: "oxaa-chev", Icon { name: "chevron-right", size: 14.0 } }
                                span { class: "oxaa-arr", Stroke { name: "arrow", size: 14.0 } }
                            }
                        }
                    }
                }
                div { class: "oxaa-ft",
                    if writing {
                        button { class: "oxaa-stop", r#type: "button", onclick: move |_| { stopped.set(Some(words)); on_action.call(("stop".into(), words.to_string())) },
                            span { class: "oxaa-sq" } "Stop" }
                    } else {
                        div { class: "oxaa-acts", role: "group", "aria-label": "Answer actions",
                            button { class: "oxaa-ib", r#type: "button", "aria-label": "Copy answer", "data-ok": "{is_copied}",
                                onclick: { let d = doc; move |_| { copied.set(Some(now)); on_action.call(("copy".into(), d.read().plain())) } },
                                if is_copied { span { class: "oxaa-okc", Icon { name: "check", size: 11.0, stroke: 2.6 } } } else { Icon { name: "copy", size: 16.0 } } }
                            button { class: "oxaa-ib", r#type: "button", "aria-label": "Regenerate",
                                onclick: move |_| { stopped.set(None); voted.set(0); reason.set(None); start.set(Some(now)); on_action.call(("regenerate".into(), String::new())) },
                                Icon { name: "retry", size: 16.0 } }
                            button { class: "oxaa-ib", r#type: "button", "aria-label": "Good answer", "aria-pressed": "{voted() == 1}",
                                onclick: move |_| { let v = if voted() == 1 { 0 } else { 1 }; voted.set(v); reason.set(None); on_action.call(("vote".into(), if v == 1 { "up".into() } else { String::new() })) },
                                Stroke { name: "up", size: 16.0 } }
                            button { class: "oxaa-ib", r#type: "button", "aria-label": "Bad answer", "aria-pressed": "{voted() == 2}", "aria-expanded": "{voted() == 2 && reason().is_none()}",
                                onclick: move |_| { let v = if voted() == 2 { 0 } else { 2 }; voted.set(v); reason.set(None); on_action.call(("vote".into(), if v == 2 { "down".into() } else { String::new() })) },
                                Stroke { name: "down", size: 16.0 } }
                        }
                        span { class: "oxaa-sp" }
                        span { class: "oxaa-fw", role: "status", span { key: "{foot}", class: "oxaa-fs", "data-ok": "{is_copied}", "{foot}" } }
                    }
                }
                if !writing && voted() == 2 && reason().is_none() {
                    div { class: "oxaa-why", role: "group", "aria-label": "What went wrong?",
                        for (i, r) in reasons.iter().cloned().enumerate() {
                            button { key: "{r}", class: "oxaa-rc", r#type: "button", onclick: move |_| { reason.set(Some(i)); on_action.call(("reason".into(), r.clone())) }, "{r}" }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────── Cited Answer ──

/// Cited Answer (affectUI): an answer wired to its sources. A header with three mini tiles and
/// "3 sources · 14 pages read"; sentences end in small numbered markers; pointing at a marker or a
/// source chip lights every sentence citing it (tinted, full strength) and fades the rest; clicking
/// unfolds the quote panel – site and date, the title, the passage with the matching words marked,
/// Open source and Copy quote (a green "Copied" and "Quote copied"). The last row reads which
/// source is lit ("Cache Notes · cited in 1 sentence") with a Follow up button. Clean: one card;
/// Classic: the text on a white card over a grey tray holding the chips and the footer, the quote
/// above the chips.
#[component]
pub fn CitedAnswerCard(
    /// Header line (the demo when empty).
    #[props(default)]
    header: String,
    /// Markup: `[n]` after a sentence cites source n (see `parse_cited`).
    #[props(default)]
    text: String,
    /// Sources (the demo when empty).
    #[props(default)]
    sources: Vec<CitedSource>,
    /// Source lit at the start (0-based), as affectUI's demo lights the second.
    #[props(default)]
    lit: Option<usize>,
    /// Open that source's quote at the start.
    #[props(default)]
    open: bool,
    /// Footer button label.
    #[props(default = "Follow up".to_string())]
    action: String,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// (`open` | `close` | `copy` | `follow-up`, source name).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let demo = use_hook(demo_cited);
    let header = if header.is_empty() {
        demo.0.clone()
    } else {
        header
    };
    let text = if text.is_empty() {
        demo.1.clone()
    } else {
        text
    };
    let sources = if sources.is_empty() {
        demo.2.clone()
    } else {
        sources
    };
    let paras = use_memo(use_reactive!(|text| parse_cited(&text)));
    let mut sel = use_signal(move || lit);
    let mut opened = use_signal(move || if open { lit } else { None });
    let mut hover = use_signal(|| None::<usize>);
    let mut copied = use_signal(|| false);
    let on = hover().or(opened()).or(sel());
    let (b, all) = backed(&paras.read());
    let foot = if copied() {
        "Quote copied".to_string()
    } else if let Some(h) = hover().filter(|h| Some(*h) != opened()) {
        sources
            .get(h)
            .map(|s| format!("{} · {}", s.name, s.title))
            .unwrap_or_default()
    } else if let Some(i) = on {
        sources
            .get(i)
            .map(|s| {
                format!(
                    "{} · {}",
                    s.name,
                    cited_in(sentences_citing(&paras.read(), i))
                )
            })
            .unwrap_or_default()
    } else {
        format!("{b} of {all} sentences backed by a source")
    };
    let (fname, frest) = match foot.split_once(" · ") {
        Some((a, r)) if !copied() => (a.to_string(), format!(" · {r}")),
        _ => (String::new(), foot.clone()),
    };
    let mut toggle = move |i: usize, name: String| {
        if opened() == Some(i) {
            opened.set(None);
            on_action.call(("close".into(), name));
        } else {
            opened.set(Some(i));
            sel.set(Some(i));
            copied.set(false);
            on_action.call(("open".into(), name));
        }
    };
    let srcs = sources.clone();
    let quote = opened().and_then(|i| srcs.get(i).cloned().map(|s| (i, s)));
    let quote_el = quote.map(|(i, s)| {
        let runs = quote_runs(&s.quote, &s.marked);
        let nm = s.name.clone();
        let q = s.quote.clone();
        rsx! {
            div { class: "oxca-q", id: "oxca-q", role: "region", "aria-label": "Quote from {s.name}",
                div { class: "oxca-qh",
                    span { class: "oxca-tl", style: "--t:#{s.tint}", "{s.mark}" }
                    span { class: "oxca-qd", "{s.domain} · {s.date}" }
                    span { class: "oxca-sp" }
                    span { class: "oxca-qn", "{i + 1}" }
                    button { class: "oxca-qx", r#type: "button", "aria-label": "Close source", onclick: move |_| { opened.set(None); copied.set(false); on_action.call(("close".into(), nm.clone())) }, Icon { name: "x", size: 14.0 } }
                }
                div { class: "oxca-qt", "{s.title}" }
                blockquote { class: "oxca-bq",
                    for (k, (t, m)) in runs.into_iter().enumerate() {
                        if m { mark { key: "{k}", "{t}" } } else { span { key: "{k}", "{t}" } }
                    }
                }
                div { class: "oxca-qa",
                    a { class: "oxca-open", href: "{s.url}", target: "_blank", rel: "noreferrer", Icon { name: "external", size: 13.0 } span { "Open source" } }
                    button { class: "oxca-cq", r#type: "button", "data-ok": "{copied()}",
                        onclick: move |_| { copied.set(true); on_action.call(("copy".into(), q.clone())) },
                        if copied() { Icon { name: "check", size: 13.0, stroke: 2.4 } span { "Copied" } } else { Icon { name: "copy", size: 13.0 } span { "Copy quote" } }
                    }
                }
            }
        }
    });
    let ps = paras.read().clone();
    rsx! {
        div { class: "oxca", "data-style": style_attr(&style), "data-lit": "{on.is_some()}",
            div { class: "oxca-tray",
                div { class: "oxca-card",
                    div { class: "oxca-hd",
                        span { class: "oxca-tiles", for (i, s) in sources.iter().enumerate().take(3) { span { key: "{i}", class: "oxca-tl", style: "--t:#{s.tint}", "{s.mark}" } } }
                        span { "{header}" }
                    }
                    div { class: "oxca-txt",
                        for (k, para) in ps.into_iter().enumerate() {
                            p { key: "{k}",
                                for (j, s) in para.into_iter().enumerate() {
                                    span { key: "{j}", class: "oxca-sw",
                                        span { class: "oxca-s", "data-on": "{on.is_some_and(|o| s.cites.contains(&o))}", "data-dim": "{on.is_some_and(|o| !s.cites.contains(&o))}", "{s.text}" }
                                        for (m, c) in s.cites.iter().copied().enumerate() {
                                            button { key: "{m}", class: "oxca-m", r#type: "button", "data-on": "{on == Some(c)}", "aria-expanded": "{opened() == Some(c)}", "aria-controls": "oxca-q",
                                                "aria-label": "Source {c + 1}: {sources.get(c).map(|x| x.name.clone()).unwrap_or_default()}",
                                                onmouseenter: move |_| hover.set(Some(c)), onmouseleave: move |_| hover.set(None),
                                                onfocus: move |_| hover.set(Some(c)), onblur: move |_| hover.set(None),
                                                onclick: { let n = sources.get(c).map(|x| x.name.clone()).unwrap_or_default(); move |_| toggle(c, n.clone()) },
                                                "{c + 1}" }
                                        }
                                        " "
                                    }
                                }
                            }
                        }
                    }
                }
                ul { class: "oxca-chips", "aria-label": "Sources",
                    for (i, s) in sources.iter().cloned().enumerate() {
                        li { key: "{i}",
                            button { class: "oxca-chip", r#type: "button", "data-on": "{on == Some(i)}", "data-open": "{opened() == Some(i)}", "aria-expanded": "{opened() == Some(i)}", "aria-controls": "oxca-q",
                                "aria-label": "Source {i + 1}: {s.name} – {s.title}",
                                onmouseenter: move |_| hover.set(Some(i)), onmouseleave: move |_| hover.set(None),
                                onclick: { let n = s.name.clone(); move |_| toggle(i, n.clone()) },
                                span { class: "oxca-tl", style: "--t:#{s.tint}", "{s.mark}" }
                                span { class: "oxca-cn", "{s.name}" }
                                span { class: "oxca-ci", "{i + 1}" }
                            }
                        }
                    }
                }
                {quote_el}
                div { class: "oxca-ft",
                    span { class: "oxca-fs", "data-ok": "{copied()}", "aria-live": "polite",
                        if !fname.is_empty() { b { "{fname}" } }
                        "{frest}"
                    }
                    button { class: "oxca-fu", r#type: "button", onclick: move |_| on_action.call(("follow-up".into(), String::new())),
                        "{action}" Stroke { name: "arrow", size: 13.0 } }
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────── Message Actions ──

/// Message Actions (affectUI): the quiet bar under an AI message – a user bubble, the model line
/// and the answer, then Copy (a filled green check and a "Copied" tooltip), Regenerate (spins,
/// the text blurs over and a new version lands; a ‹ 2 / 3 › switcher unfolds from two versions
/// on), thumbs up ("Thanks!"), thumbs down (a "What went wrong?" row with reason chips and Send,
/// which turns into a green "Thanks!") and Read aloud (a blue four-bar equalizer, "Stop"). The bar
/// is dim until the message is pointed at. Clean: the bar is a tinted band under a hairline in the
/// same card; Classic: the bar and the reasons sit in their own card below, the chosen reason blue.
#[component]
pub fn MessageActionBar(
    /// The user's message (the demo when empty).
    #[props(default)]
    prompt: String,
    /// Model name on the model line.
    #[props(default)]
    model: String,
    /// Model brand for its tile.
    #[props(default)]
    brand: String,
    /// Versions of the answer so far (the demo when empty).
    #[props(default)]
    versions: Vec<String>,
    /// Answers Regenerate draws from, in turn.
    #[props(default)]
    pool: Vec<String>,
    /// Start with a thumbs-down and the reasons open.
    #[props(default)]
    feedback_open: bool,
    /// Start reading aloud.
    #[props(default)]
    reading: bool,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// (`copy` | `regenerate` | `version` | `vote` | `reason` | `send` | `read`, detail).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let demo = use_hook(demo_message);
    let prompt = if prompt.is_empty() {
        demo.0.clone()
    } else {
        prompt
    };
    let model = if model.is_empty() {
        demo.1.clone()
    } else {
        model
    };
    let brand = if brand.is_empty() {
        demo.2.clone()
    } else {
        brand
    };
    let base = if versions.is_empty() {
        demo.3.clone()
    } else {
        versions
    };
    let pool = if pool.is_empty() {
        demo.4.clone()
    } else {
        pool
    };
    let mut regens = use_signal(Vec::<ClockMs>::new);
    let mut picked = use_signal(|| None::<(usize, ClockMs)>);
    let mut copied = use_signal(|| None::<ClockMs>);
    let mut vote = use_signal(move || if feedback_open { 2u8 } else { 0 });
    let mut fb = use_signal(move || feedback_open);
    let mut why = use_signal(|| None::<usize>);
    let mut sent = use_signal(|| false);
    let mut read_at = use_signal(move || if reading { Some(now) } else { None });
    // Versions that have landed; a spin still running shows the old text blurred.
    let landed: Vec<ClockMs> = regens()
        .into_iter()
        .filter(|t| ms(now, *t) >= REGENERATE_MS)
        .collect();
    let spinning = regens().iter().any(|t| ms(now, *t) < REGENERATE_MS);
    let mut all = base.clone();
    for (k, _) in landed.iter().enumerate() {
        all.push(pool[k % pool.len().max(1)].clone());
    }
    let n = all.len();
    let last_land = landed.last().map(|t| ClockMs(t.0 + REGENERATE_MS));
    let cur = match picked() {
        Some((i, at)) if last_land.is_none_or(|l| at.0 >= l.0) => i.min(n - 1),
        _ => n - 1,
    };
    let text = all[cur].clone();
    let is_copied = copied().is_some_and(|c| ms(now, c) < 1600);
    let words = text.split(' ').count() as u64;
    let reading_now = read_at().is_some_and(|t| ms(now, t) < words * 320 + 600);
    let live = if is_copied {
        "Copied".to_string()
    } else if spinning {
        "Regenerating".to_string()
    } else if sent() {
        "Thanks for the feedback".to_string()
    } else if vote() == 1 {
        "Marked as good response".to_string()
    } else if vote() == 2 {
        "Marked as bad response. Tell us what went wrong below".to_string()
    } else if reading_now {
        "Reading aloud".to_string()
    } else {
        String::new()
    };
    let txt2 = text.clone();
    rsx! {
        div { class: "oxma", "data-style": style_attr(&style),
            div { class: "oxma-card",
                div { class: "oxma-msg",
                    div { class: "oxma-you", "{prompt}" }
                    div { class: "oxma-model", span { class: "oxma-mt", BrandIcon { name: brand.clone(), size: 11.0, coloured: true, fallback: true } } "{model}" }
                    div { class: "oxma-tw", p { key: "{cur}", class: "oxma-text", "data-spin": "{spinning}", "{text}" } }
                }
                div { class: "oxma-dock",
                    div { class: "oxma-bar", role: "toolbar", "aria-label": "Message actions", "data-on": "{is_copied || spinning || vote() != 0 || reading_now}",
                        button { class: "oxma-ib", r#type: "button", "aria-label": "Copy message", "data-tip": if is_copied { "Copied" } else { "Copy" }, "data-force": "{is_copied}", "data-ok": "{is_copied}",
                            onclick: move |_| { copied.set(Some(now)); on_action.call(("copy".into(), txt2.clone())) },
                            if is_copied { span { class: "oxma-okc", Icon { name: "check", size: 11.0, stroke: 2.6 } } } else { Icon { name: "copy", size: 16.0 } } }
                        button { class: "oxma-ib", r#type: "button", "aria-label": "Regenerate response", "data-tip": if spinning { "Writing…" } else { "Regenerate" }, "data-spin": "{spinning}", disabled: spinning,
                            onclick: move |_| { regens.write().push(now); sent.set(false); on_action.call(("regenerate".into(), String::new())) },
                            Icon { name: "retry", size: 16.0 } }
                        if n > 1 {
                            div { class: "oxma-ver", role: "group", "aria-label": "Versions",
                                button { class: "oxma-vb", r#type: "button", "aria-label": "Previous version", disabled: cur == 0,
                                    onclick: move |_| { picked.set(Some((cur.saturating_sub(1), now))); on_action.call(("version".into(), cur.to_string())) },
                                    Icon { name: "chevron-left", size: 13.0, stroke: 2.0 } }
                                span { class: "oxma-vn", "aria-label": "Version {cur + 1} of {n}", b { key: "{cur}", "{cur + 1}" } span { " / {n}" } }
                                button { class: "oxma-vb", r#type: "button", "aria-label": "Next version", disabled: cur + 1 >= n,
                                    onclick: move |_| { picked.set(Some(((cur + 1).min(n - 1), now))); on_action.call(("version".into(), (cur + 2).to_string())) },
                                    Icon { name: "chevron-right", size: 13.0, stroke: 2.0 } }
                            }
                        }
                        button { class: "oxma-ib", r#type: "button", "aria-label": "Good response", "aria-pressed": "{vote() == 1}", "data-tip": if vote() == 1 { "Thanks!" } else { "Good response" },
                            onclick: move |_| { let v = if vote() == 1 { 0 } else { 1 }; vote.set(v); fb.set(false); sent.set(false); on_action.call(("vote".into(), if v == 1 { "up".into() } else { String::new() })) },
                            Stroke { name: "up", size: 16.0 } }
                        button { class: "oxma-ib", r#type: "button", "aria-label": "Bad response", "aria-pressed": "{vote() == 2}", "aria-expanded": "{fb()}", "data-tip": "Bad response",
                            onclick: move |_| { let v = if vote() == 2 { 0 } else { 2 }; vote.set(v); fb.set(v == 2); why.set(None); sent.set(false); on_action.call(("vote".into(), if v == 2 { "down".into() } else { String::new() })) },
                            Stroke { name: "down", size: 16.0 } }
                        button { class: "oxma-ib", r#type: "button", "aria-label": if reading_now { "Stop reading" } else { "Read aloud" }, "aria-pressed": "{reading_now}", "data-tip": if reading_now { "Stop" } else { "Read aloud" }, "data-read": "{reading_now}",
                            onclick: move |_| { read_at.set(if reading_now { None } else { Some(now) }); on_action.call(("read".into(), (!reading_now).to_string())) },
                            if reading_now { span { class: "oxma-eq", "aria-hidden": "true", i {} i {} i {} i {} } } else { Stroke { name: "speaker", size: 16.0 } } }
                    }
                    if fb() {
                        div { class: "oxma-fb", role: "group", "aria-label": "What went wrong?",
                            div { class: "oxma-fh", b { "What went wrong?" } span { class: "oxma-sp" }
                                button { class: "oxma-fx", r#type: "button", "aria-label": "Close feedback", onclick: move |_| fb.set(false), Icon { name: "x", size: 14.0 } } }
                            div { class: "oxma-chips", "data-picked": "{why().is_some()}",
                                for (i, r) in MESSAGE_REASONS.into_iter().enumerate() {
                                    button { key: "{r}", class: "oxma-rc", r#type: "button", "aria-pressed": "{why() == Some(i)}", disabled: sent(),
                                        onclick: move |_| { why.set(Some(i)); on_action.call(("reason".into(), r.to_string())) }, "{r}" }
                                }
                                span { class: "oxma-sp" }
                                if sent() {
                                    span { class: "oxma-thx", Icon { name: "check", size: 13.0, stroke: 2.4 } "Thanks!" }
                                } else {
                                    button { class: "oxma-send", r#type: "button", disabled: why().is_none(),
                                        onclick: move |_| { sent.set(true); on_action.call(("send".into(), why().map(|i| MESSAGE_REASONS[i].to_string()).unwrap_or_default())) }, "Send" }
                                }
                            }
                        }
                    }
                }
                span { class: "oxma-live", role: "status", "{live}" }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────── Thinking pill ──

/// Thinking Loader (affectUI): a small working pill – eight thin accent-blue rays that fade one
/// after another (it turns without rotating) and a grey label a darker band sweeps through. On a
/// new phase the old label rolls up and blurs out, the new one rolls in and the pill glides to its
/// width; done, the spinner shrinks away, a check draws itself and the label settles on "Thought
/// for Ns". Clean: a white chip with hairline shadows and a continuous rounded-rect radius, blue
/// rays; Classic: a capsule with grey rays. `size` md (32 px) or sm (26 px); `variant` pill or plain.
#[component]
pub fn ThinkingPill(
    /// Labels walked through, one after another (affectUI's four when empty).
    #[props(default)]
    phases: Vec<String>,
    /// Time per phase, ms.
    #[props(default = LOADER_INTERVAL)]
    interval: u64,
    /// Start over after the last phase instead of settling.
    #[props(default)]
    looped: bool,
    /// Settle on done after the last phase (else hold it).
    #[props(default = true)]
    finish: bool,
    /// Done from the start (or controlled).
    #[props(default)]
    done: bool,
    /// Label once done (empty: "Thought for Ns").
    #[props(default)]
    done_label: String,
    /// `md` or `sm`.
    #[props(default = "md".to_string())]
    size: String,
    /// `pill` or `plain`.
    #[props(default = "pill".to_string())]
    variant: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
) -> Element {
    let phases: Vec<String> = if phases.is_empty() {
        LOADER_PHASES.iter().map(|s| s.to_string()).collect()
    } else {
        phases
    };
    let start = use_signal(move || now);
    let el = ms(now, start());
    let (i, past) = loader_phase(el, interval, phases.len(), looped);
    let is_done = done || (finish && past);
    let secs = if done {
        4
    } else {
        ((phases.len() as u64 * interval) / 1000).max(1)
    };
    let label = if is_done {
        if done_label.is_empty() {
            thought_for(secs)
        } else {
            done_label
        }
    } else {
        phases.get(i).cloned().unwrap_or_default()
    };
    rsx! {
        div { class: "oxtl", "data-style": style_attr(&style), "data-size": "{size}", "data-variant": "{variant}", "data-done": "{is_done}",
            role: "status", "aria-live": "polite",
            span { class: "oxtl-ic", "aria-hidden": "true",
                if is_done {
                    svg { class: "oxtl-ck", width: "14", height: "14", view_box: "0 0 14 14", fill: "none",
                        path { d: "M3 7.4l2.6 2.5L11 4.4", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "pathLength": "1" } }
                } else {
                    svg { class: "oxtl-rays", width: "14", height: "14", view_box: "0 0 14 14",
                        for r in 0..8u8 {
                            line { key: "{r}", x1: "7", y1: "1.4", x2: "7", y2: "4.2", transform: "rotate({r as u32 * 45} 7 7)", style: "animation-delay:{r as f32 * 0.1 - 0.8:.1}s" }
                        }
                    }
                }
            }
            span { class: "oxtl-lw",
                span { key: "{label}", class: "oxtl-lb", "data-shimmer": "{!is_done}", "{label}" }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Thinking panel ──

/// The 3×3 dot mark of the thinking header; the dots fade in turn while it works.
#[component]
fn DotGrid(live: bool) -> Element {
    rsx! {
        svg { class: "oxts-dots", "data-live": "{live}", width: "13", height: "13", view_box: "0 0 13 13", "aria-hidden": "true",
            for k in 0..9u8 {
                circle { key: "{k}", cx: "{2.0 + (k % 3) as f32 * 4.5}", cy: "{2.0 + (k / 3) as f32 * 4.5}", r: "1.35", style: "animation-delay:{(k % 3 + k / 3) as f32 * 0.12:.2}s" }
            }
        }
    }
}

/// Thinking State (affectUI): an AI working panel. The header shimmers "Thinking 3s" with a live
/// timer, then reads "Worked for 7s"; it folds the steps away and back. Steps fade up one after
/// another along a dashed timeline: a "Searched" step pops in source chips with coloured brand
/// marks and a "+3 more" chip that reveals the rest ("Show less"), thinking lines get a small
/// square that pulses blue while active, and a blue check draws in on the last row ("Finished
/// thinking"). Clean: a blue dot-grid mark in the header; Classic: a chevron on the left and flat
/// grey chips.
#[component]
pub fn ThinkingPanel(
    /// Steps in order (the demo when empty).
    #[props(default)]
    steps: Vec<ThinkStep>,
    /// Label of the final row.
    #[props(default = "Finished thinking".to_string())]
    done_label: String,
    /// Pause between two steps, ms.
    #[props(default = THINK_STEP_MS)]
    step_ms: u64,
    /// Start over a few seconds after finishing.
    #[props(default)]
    looped: bool,
    /// Show it finished from the start.
    #[props(default)]
    finished: bool,
    /// Fold the steps away once finished.
    #[props(default)]
    collapse_on_done: bool,
    /// Start folded.
    #[props(default)]
    collapsed: bool,
    /// Start with every source shown.
    #[props(default)]
    show_all: bool,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
) -> Element {
    let steps = if steps.is_empty() {
        demo_think_steps()
    } else {
        steps
    };
    let n = steps.len();
    let start = use_signal(move || now);
    let mut user_open = use_signal(|| None::<bool>);
    let mut all_src = use_signal(move || show_all);
    let mut el = if finished {
        step_ms * n as u64
    } else {
        ms(now, start())
    };
    if looped && !finished {
        el %= step_ms * n as u64 + 3500;
    }
    let (shown, done, secs) = think_progress(el, step_ms, n);
    let open = user_open().unwrap_or(!collapsed && !(collapse_on_done && done));
    let shown_steps: Vec<(ThinkStep, Vec<oxyn_core::kit::answers::ThinkSource>)> = steps
        .into_iter()
        .take(shown)
        .map(|s| {
            let mut c = s.sources.clone();
            if all_src() {
                c.extend(s.more.iter().cloned());
            }
            (s, c)
        })
        .collect();
    rsx! {
        div { class: "oxts", "data-style": style_attr(&style), "data-done": "{done}",
            button { class: "oxts-hd", r#type: "button", "aria-expanded": "{open}", onclick: move |_| user_open.set(Some(!open)),
                span { class: "oxts-chv", Icon { name: "chevron", size: 13.0, stroke: 2.0 } }
                DotGrid { live: !done }
                if done {
                    span { class: "oxts-hl done", "Worked for {secs}s" }
                } else {
                    span { class: "oxts-hl", span { class: "oxts-shim", "Thinking" } " " span { class: "oxts-tm", "{secs}s" } }
                }
                span { class: "oxts-chr", Icon { name: "chevron-right", size: 13.0, stroke: 2.0 } }
            }
            if open {
                ol { class: "oxts-tl",
                    for (k, (s, chips)) in shown_steps.into_iter().enumerate() {
                        li { key: "{k}", class: "oxts-st", "data-active": "{!done && k + 1 == shown}", "data-search": "{!s.sources.is_empty()}",
                            span { class: "oxts-nd", if s.sources.is_empty() { i {} } else { Stroke { name: "globe", size: 14.0 } } }
                            div { class: "oxts-sb",
                                span { class: "oxts-sl", "{s.label}" }
                                if !s.sources.is_empty() {
                                    div { class: "oxts-chips",
                                        for (j, c) in chips.into_iter().enumerate() {
                                            a { key: "{j}", class: "oxts-chip", href: "{c.url}", target: "_blank", rel: "noreferrer", title: "{c.url}", style: "animation-delay:{j as f32 * 0.08:.2}s",
                                                BrandIcon { name: c.brand.clone(), size: 13.0, coloured: true, fallback: true } span { "{c.label}" } }
                                        }
                                        if !s.more.is_empty() {
                                            button { class: "oxts-more", r#type: "button", "aria-expanded": "{all_src()}", onclick: move |_| all_src.set(!all_src()),
                                                if all_src() { "Show less" } else { "+{s.more.len()} more" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if done {
                        li { class: "oxts-st oxts-fin",
                            span { class: "oxts-nd", svg { class: "oxts-ok", width: "15", height: "15", view_box: "0 0 16 16", "aria-hidden": "true",
                                circle { cx: "8", cy: "8", r: "8" }
                                path { d: "M4.8 8.2l2.2 2.1 4.2-4.4", fill: "none", stroke: "#fff", stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", "pathLength": "1" } } }
                            div { class: "oxts-sb", span { class: "oxts-sl", "{done_label}" } }
                        }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────────────────────── Writing Assistant ──

/// Writing Assistant (affectUI): an AI chat card with a soft gradient glow. A violet sparkle, the
/// name and a tagline on top (a "New chat" + once there are messages), the welcome line and
/// suggestion chips that send themselves, a prompt field with a mic. A message shows as a grey
/// bubble, a shimmering "Writing…" holds its place and the reply streams in word by word; the mic
/// turns into a pale-blue tile with level bars while it dictates and sends what it heard. Clean:
/// the glow drifts in the lower half and quiets once you chat, bare field and mic; Classic: a
/// stronger glow on an inset panel, capsule chips, a white pill field and a round mic.
#[component]
pub fn WritingAssistant(
    /// Assistant name on top.
    #[props(default = "Wren".to_string())]
    name: String,
    /// Line under the name.
    #[props(default = "Your writing companion.".to_string())]
    tagline: String,
    /// Big line on the welcome screen.
    #[props(default = "What should we write?".to_string())]
    greeting: String,
    /// Chips that send themselves.
    #[props(default)]
    suggestions: Vec<String>,
    /// Placeholder of the prompt field.
    #[props(default = "Ask anything…".to_string())]
    placeholder: String,
    /// Start with this message already answered.
    #[props(default)]
    asked: String,
    /// Start dictating.
    #[props(default)]
    dictating: bool,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// (`send` | `dictate` | `new-chat`, text).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let suggestions: Vec<String> = if suggestions.is_empty() {
        [
            "Tighten my email",
            "Make it shorter",
            "Soften the tone",
            "Draft a reply",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    } else {
        suggestions
    };
    let mut msgs = use_signal(move || {
        if asked.is_empty() {
            vec![]
        } else {
            vec![(asked.clone(), ClockMs(now.0.saturating_sub(60_000)))]
        }
    });
    let mut draft = use_signal(String::new);
    let mut dict = use_signal(move || if dictating { Some(now) } else { None });
    let dict_len = (DICTATION.len() as u64 * 1000) / DICTATION_CPS;
    // A finished dictation sends itself: the message is shown from the moment it ends.
    let dict_msg = dict().and_then(|t| {
        (ms(now, t) >= dict_len + 500)
            .then(|| (DICTATION.to_string(), ClockMs(t.0 + dict_len + 500)))
    });
    let listening = dict().is_some() && dict_msg.is_none();
    let mut view = msgs();
    if let Some(m) = dict_msg.clone() {
        view.push(m);
    }
    let field = if listening {
        dict()
            .map(|t| dictated_at(ms(now, t)).to_string())
            .unwrap_or_default()
    } else {
        draft()
    };
    let mut commit = move || {
        if let Some(t) = dict() {
            if ms(now, t) >= dict_len + 500 {
                msgs.write()
                    .push((DICTATION.to_string(), ClockMs(t.0 + dict_len + 500)));
            }
            dict.set(None);
        }
    };
    let mut send = move |t: String| {
        let t = t.trim().to_string();
        if t.is_empty() {
            return;
        }
        commit();
        msgs.write().push((t.clone(), now));
        draft.set(String::new());
        on_action.call(("send".into(), t));
    };
    let chatting = !view.is_empty();
    rsx! {
        div { class: "oxwa", "data-style": style_attr(&style), "data-chat": "{chatting}",
            div { class: "oxwa-card",
                div { class: "oxwa-hd",
                    span { class: "oxwa-spk", Stroke { name: "sparkle", size: 13.0 } }
                    div { class: "oxwa-id", h3 { "{name}" } span { "{tagline}" } }
                    span { class: "oxwa-sp" }
                    if chatting {
                        button { class: "oxwa-new", r#type: "button", "aria-label": "New chat",
                            onclick: move |_| { msgs.set(vec![]); dict.set(None); draft.set(String::new()); on_action.call(("new-chat".into(), String::new())) },
                            Icon { name: "plus", size: 15.0 } }
                    }
                }
                div { class: "oxwa-body",
                    div { class: "oxwa-glow", "aria-hidden": "true", i {} i {} i {} }
                    if chatting {
                        div { class: "oxwa-log", "aria-live": "polite",
                            for (k, (m, at)) in view.into_iter().enumerate() {
                                div { key: "{k}", class: "oxwa-turn",
                                    div { class: "oxwa-you", "{m}" }
                                    match reply_at(&sample_reply(&m), ms(now, at)) {
                                        None => rsx! { div { class: "oxwa-wr", "aria-label": "Writing", "Writing…" } },
                                        Some(r) => rsx! { p { class: "oxwa-ai", "{r}" } },
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "oxwa-hi",
                            p { class: "oxwa-greet", "{greeting}" }
                            div { class: "oxwa-sug",
                                for s in suggestions.iter().cloned() {
                                    button { key: "{s}", class: "oxwa-chip", r#type: "button", onclick: { let s = s.clone(); move |_| send(s.clone()) }, "{s}" }
                                }
                            }
                        }
                    }
                    div { class: "oxwa-in",
                        input { class: "oxwa-field", value: "{field}", placeholder: "{placeholder}", "aria-label": "Message", readonly: listening,
                            oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); send(draft()) } }
                        if !listening && !draft().trim().is_empty() {
                            button { class: "oxwa-send", r#type: "button", "aria-label": "Send", onclick: move |_| send(draft()), Icon { name: "up", size: 14.0, stroke: 2.0 } }
                        }
                        button { class: "oxwa-mic", r#type: "button", "aria-label": if listening { "Stop dictation" } else { "Dictate" }, "aria-pressed": "{listening}",
                            onclick: move |_| { if listening { dict.set(None) } else { commit(); dict.set(Some(now)); on_action.call(("dictate".into(), String::new())) } },
                            if listening { span { class: "oxwa-lv", "aria-hidden": "true", i {} i {} i {} } } else { Icon { name: "mic", size: 16.0 } } }
                    }
                }
            }
        }
    }
}
