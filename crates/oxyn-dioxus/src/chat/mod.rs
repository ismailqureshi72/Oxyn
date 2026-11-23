//! Chat (plan #27): `Markdown`, `StreamingMarkdown`, `ReasoningBlock`, `ThinkingTraces`,
//! `SourceCards` + `SourceCitation`, `AiAnswer` (answer · cited), `ResponseFeedback`,
//! `MessageActions`, `MessageBubble`, `StarterPrompts`, `ChatThread`, `DocumentExtraction`,
//! `ResearchRun`, `SupportConversation`, `RecommendationCard`, `ContextCards` and `AiChat`.
//!
//! Logic lives in `oxyn_core::chat`. Streams are driven by the host clock: pass `elapsed_ms`
//! (milliseconds since the stream started) and the component reveals what has "arrived" by then,
//! so the same markup renders on the server and plays live in the browser. Styles are in
//! [`CHAT_CSS`]. Behaviour follows Arc UI and affectUI (rebuilt; no source used); Beautiful UI
//! Thinking, Chat, Context Cards and Recommendation Card are MIT ports (© 2026 Shane Levine).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    chat::{
        markdown::{Block, Inline, citations, inline, parse, revealed_at},
        reasoning::{
            DisplayLevel, Reasoning, ReasoningStatus, RowStatus, Trace, TraceKind, TraceRow,
            preview, steps,
        },
        sources::{
            Claim, ContextCard, Decision, Extraction, Field, Provenance, Recommendation,
            SearchStatus, Slot, Source, SourceCards as CardsModel, context_tokens, grounding,
        },
        thread::{
            ChatMessage, Delivery, Feedback, FeedbackPhase, NEGATIVE, POSITIVE, ResearchPhase,
            ResearchRun as RunModel, Starter, SupportPhase, ThreadItem, Thumb, categories, clock,
            group, starters_for, typing_label,
        },
    },
};

#[cfg(oxyn_pro)]
use crate::Rating;
use crate::composer::icons::Icon;
use crate::{CodeBlock, CopyButton};

/// Stylesheet for the chat components.
pub const CHAT_CSS: &str = include_str!("chat.css");

fn cls(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

// ───────────────────────────────────────── Markdown ──

fn inl(v: Vec<Inline>, on_cite: EventHandler<u32>, active: Option<u32>) -> Element {
    rsx! {
        for (k, x) in v.into_iter().enumerate() {
            match x {
                Inline::Text(s) => rsx! { span { key: "{k}", "{s}" } },
                Inline::Bold(s) => rsx! { strong { key: "{k}", "{s}" } },
                Inline::Italic(s) => rsx! { em { key: "{k}", "{s}" } },
                Inline::Code(s) => rsx! { code { key: "{k}", class: "ic", "{s}" } },
                Inline::Link(t, u) => rsx! { a { key: "{k}", href: "{u}", "{t}" } },
                Inline::Cite(n) => rsx! {
                    button { key: "{k}", class: cls(&["oxc-cite", if active == Some(n) { "on" } else { "" }]), r#type: "button", "aria-label": "Source {n}", onclick: move |_| on_cite.call(n), "{n}" }
                },
            }
        }
    }
}

/// Markdown: renders parsed blocks (headings, paragraphs, lists, quotes, rules, tables, code
/// through [`CodeBlock`]) with citation chips. `caret` puts a streaming caret after the last
/// block.
#[component]
pub fn Markdown(
    /// Source.
    text: String,
    /// Streaming caret at the end.
    #[props(default)]
    caret: bool,
    /// Highlighted citation.
    #[props(default)]
    active_cite: Option<u32>,
    /// A citation chip was pressed.
    #[props(default)]
    on_cite: EventHandler<u32>,
) -> Element {
    let blocks = parse(&text);
    let n = blocks.len();
    rsx! {
        div { class: "oxc-md",
            for (i, b) in blocks.into_iter().enumerate() {
                {
                    let last = caret && i + 1 == n;
                    let cr = rsx! { if last { span { class: "oxc-caret", "aria-hidden": "true" } } };
                    match b {
                        Block::Heading(l, v) => match l { 1 => rsx! { h3 { key: "{i}", class: "h1", {inl(v, on_cite, active_cite)} {cr} } }, 2 => rsx! { h4 { key: "{i}", class: "h2", {inl(v, on_cite, active_cite)} {cr} } }, _ => rsx! { h5 { key: "{i}", class: "h3", {inl(v, on_cite, active_cite)} {cr} } } },
                        Block::Para(v) => rsx! { p { key: "{i}", {inl(v, on_cite, active_cite)} {cr} } },
                        Block::List(o, items) => {
                            let m = items.len();
                            let body = rsx! { for (j, it) in items.into_iter().enumerate() { li { key: "{j}", {inl(it, on_cite, active_cite)} if last && j + 1 == m { span { class: "oxc-caret", "aria-hidden": "true" } } } } };
                            if o { rsx! { ol { key: "{i}", {body} } } } else { rsx! { ul { key: "{i}", {body} } } }
                        }
                        Block::Quote(v) => rsx! { blockquote { key: "{i}", {inl(v, on_cite, active_cite)} {cr} } },
                        Block::Rule => rsx! { hr { key: "{i}" } },
                        Block::Table(h, rows) => rsx! {
                            div { key: "{i}", class: "oxc-table", table {
                                thead { tr { for (j, c) in h.iter().enumerate() { th { key: "{j}", "{c}" } } } }
                                tbody { for (r, row) in rows.iter().enumerate() { tr { key: "{r}", class: "rin", for (j, c) in row.iter().enumerate() { td { key: "{j}", {inl(inline(c), on_cite, active_cite)} } } } } }
                            } }
                        },
                        Block::Code { lang, text, open } => rsx! {
                            div { key: "{i}", class: "oxc-codew", CodeBlock { code: text, language: if lang.is_empty() { "text".to_string() } else { lang }, streaming: open && caret, filename: None } }
                        },
                    }
                }
            }
            if n == 0 && caret { span { class: "oxc-caret", "aria-hidden": "true" } }
        }
    }
}

/// Streaming markdown: reveals `text` at `cps` characters per second from `elapsed_ms` (the host
/// clock), with a caret while streaming, held-back partial markers, open code fences that grow,
/// a "waiting" shimmer before the first token, and Stop / error states that keep what arrived.
/// Oxyn plus: `pace` presets, a word-fade reveal, a jump-to-end and a per-block TOC hook.
#[component]
pub fn StreamingMarkdown(
    /// Full text the model will send.
    text: String,
    /// Milliseconds since the stream started.
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// Characters per second (0 = instant).
    #[props(default = 140.0)]
    cps: f32,
    /// Delay before the first token (waiting shimmer).
    #[props(default)]
    first_token_ms: u64,
    /// `streaming` · `stopped` · `error` (stops at what arrived).
    #[props(default = "streaming".to_string())]
    phase: String,
    /// Show the caret.
    #[props(default = true)]
    caret: bool,
    /// `fade` · `none`.
    #[props(default = "fade".to_string())]
    reveal: String,
    /// Highlighted citation.
    #[props(default)]
    active_cite: Option<u32>,
    /// A citation chip was pressed.
    #[props(default)]
    on_cite: EventHandler<u32>,
) -> Element {
    let total = text.chars().count();
    let t = elapsed_ms.saturating_sub(first_token_ms);
    let waiting = elapsed_ms < first_token_ms;
    let mut n = revealed_at(total, Duration::from_millis(t), cps);
    let stopped = phase == "stopped" || phase == "error";
    if stopped {
        n = n.min(total * 2 / 3);
    }
    let shown: String = text.chars().take(n).collect();
    let streaming = !stopped && n < total;
    rsx! {
        div { class: cls(&["oxc-stream", &reveal, if streaming { "live" } else { "" }]), "aria-busy": "{streaming || waiting}",
            if waiting {
                div { class: "oxc-wait", span { class: "oxc-shim", "Waiting for the first token…" } }
            } else {
                Markdown { text: shown, caret: caret && streaming, active_cite, on_cite }
            }
            if phase == "stopped" { div { class: "oxc-end stop", Icon { name: "stop", size: 12.0 } "Stopped · the reply above is partial" } }
            if phase == "error" { div { class: "oxc-end err", role: "alert", Icon { name: "alert", size: 12.0 } "The connection dropped. " button { class: "oxyn-btn ghost xs", r#type: "button", Icon { name: "retry", size: 12.0 } "Retry" } } }
        }
    }
}

// ───────────────────────────────────── Reasoning ──

/// Reasoning block: the collapsible thinking of a reply. While thinking, a shimmering label
/// counts seconds and the newest two lines stream underneath (aria-hidden; the full region is
/// `aria-busy`). Done folds to "Thought for 7s" and opens to steps parsed from `**Heading**`
/// lines; Stop and Retry live in a fixed-height header. Ports the affectUI Thinking State and
/// Arc UI Reasoning block behaviour. Oxyn plus: display levels (`normal` · `thinking` ·
/// `verbose`), `plain` variant inside a bubble, and a model mark.
#[component]
pub fn ReasoningBlock(
    /// Full reasoning (revealed over `elapsed_ms` while thinking).
    reasoning: String,
    /// `thinking` · `done` · `stopped` · `error`.
    #[props(default = "done".to_string())]
    status: String,
    /// Seconds spent (done); measured from `elapsed_ms` while thinking.
    #[props(default)]
    seconds: Option<u64>,
    /// Milliseconds since thinking started.
    #[props(default)]
    elapsed_ms: u64,
    /// Characters per second while thinking.
    #[props(default = 60.0)]
    cps: f32,
    /// Model name.
    #[props(default = "Atlas".to_string())]
    model: String,
    /// `card` · `plain`.
    #[props(default = "card".to_string())]
    variant: String,
    /// `normal` · `thinking` · `verbose`.
    #[props(default = "normal".to_string())]
    level: String,
    /// Start open.
    #[props(default)]
    open: bool,
    /// Error reason.
    #[props(default)]
    error: Option<String>,
    /// Stop pressed.
    #[props(default)]
    on_stop: EventHandler<()>,
    /// Retry pressed.
    #[props(default)]
    on_retry: EventHandler<()>,
) -> Element {
    let lvl = DisplayLevel::from_id(&level);
    let mut is_open = use_signal(|| open || lvl == DisplayLevel::Verbose);
    let st = ReasoningStatus::from_id(&status);
    let shown_text: String = if st == ReasoningStatus::Thinking {
        let n = revealed_at(
            reasoning.chars().count(),
            Duration::from_millis(elapsed_ms),
            cps,
        );
        reasoning.chars().take(n).collect()
    } else {
        reasoning.clone()
    };
    let mut r = Reasoning::new(st, &shown_text);
    r.level = lvl;
    r.open = is_open();
    r.error = error.clone();
    r.seconds = seconds.or(if st == ReasoningStatus::Thinking {
        Some(elapsed_ms / 1000)
    } else {
        None
    });
    let label = if st == ReasoningStatus::Thinking {
        "Thinking".to_string()
    } else {
        r.label(ClockMs(0))
    };
    let secs = r.seconds.unwrap_or(0);
    let id = format!(
        "oxc-rs-{}",
        model
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    );
    let st_steps = steps(&shown_text);
    let pv = preview(&shown_text, 2);
    rsx! {
        div { class: cls(&["oxc-reason", &variant, st.id(), if r.open { "open" } else { "" }]), "aria-label": "{model} reasoning",
            div { class: "rh",
                button { class: "rt", r#type: "button", "aria-expanded": "{r.open}", "aria-controls": "{id}", onclick: move |_| { let o = !is_open(); is_open.set(o) },
                    span { class: "rm", "aria-hidden": "true", Icon { name: if st == ReasoningStatus::Error { "alert" } else { "brain" }, size: 14.0 } }
                    for _k in std::iter::once(label.clone()) { span { key: "{_k}", class: cls(&["rl", if st == ReasoningStatus::Thinking { "oxc-shim" } else { "" }]), "{label}" } }
                    if st == ReasoningStatus::Thinking { span { class: "rs", key: "{secs}", "{secs}s" } }
                    span { class: "rc", "aria-hidden": "true", Icon { name: "chevron", size: 12.0 } }
                }
                if st == ReasoningStatus::Thinking { button { class: "ra", r#type: "button", onclick: move |_| on_stop.call(()), Icon { name: "stop", size: 11.0 } "Stop" } }
                if st == ReasoningStatus::Error { button { class: "ra", r#type: "button", onclick: move |_| on_retry.call(()), Icon { name: "retry", size: 11.0 } "Retry" } }
            }
            if r.shows_preview() {
                div { class: "rp", "aria-hidden": "true", for (i, l) in pv.iter().enumerate() { div { key: "{l}", class: "rpl", style: "--i:{i}", "{l}" } } }
            }
            div { class: "rb", id: "{id}", role: "region", "aria-busy": "{st == ReasoningStatus::Thinking}", hidden: !r.open,
                div { class: "rbi",
                    for (i, s) in st_steps.iter().enumerate() {
                        div { key: "{i}", class: "rstep",
                            span { class: "rdot", "aria-hidden": "true" }
                            div { if let Some(t) = &s.title { div { class: "rst", "{t}" } } p { "{s.text}" } }
                        }
                    }
                    div { class: "rf",
                        match st { ReasoningStatus::Done => rsx! { Icon { name: "check", size: 12.0 } span { "Done · {secs}s" } }, ReasoningStatus::Stopped => rsx! { Icon { name: "stop", size: 12.0 } span { "Stopped" } }, ReasoningStatus::Error => rsx! { Icon { name: "alert", size: 12.0 } span { {error.clone().unwrap_or("Interrupted".into())} } }, ReasoningStatus::Thinking => rsx! { span { class: "oxc-shim", "Still thinking…" } } }
                        span { class: "sp" }
                        CopyButton { value: reasoning.clone(), label: "Copy reasoning", icon_only: true, variant: "plain", size: "sm" }
                    }
                }
            }
            div { class: "sr", role: "status", "{r.announce(ClockMs(0))}" }
        }
    }
}

// ────────────────────────────────────── Feedback ──

/// Response feedback: thumbs up / down toggles (`aria-pressed`). Down opens an inline panel of
/// reason chips and an optional note (Cmd / Ctrl + Enter submits); up fills and asks what was
/// good. Submit shows a short thank-you that folds away; a failed send turns Submit into Try
/// again. Escape closes the panel and keeps the rating. Oxyn plus: `fail_send` demo switch and a
/// compact row inside message actions.
#[component]
pub fn ResponseFeedback(
    /// Reasons after thumbs down.
    #[props(default = NEGATIVE.iter().map(|s| s.to_string()).collect())]
    negative: Vec<String>,
    /// Reasons after thumbs up ([] skips the row).
    #[props(default = POSITIVE.iter().map(|s| s.to_string()).collect())]
    positive: Vec<String>,
    /// Note field after thumbs down.
    #[props(default = true)]
    allow_comment: bool,
    /// Initial rating (`up` · `down`).
    #[props(default)]
    rating: Option<String>,
    /// Privacy line next to Submit.
    #[props(default = "Shares this reply for review".to_string())]
    privacy_note: String,
    /// Simulate a failed send.
    #[props(default)]
    fail_send: bool,
    /// Disabled (still streaming).
    #[props(default)]
    disabled: bool,
    /// Leading actions (copy, regenerate).
    #[props(default = VNode::empty())]
    leading: Element,
    /// Rating changed.
    #[props(default)]
    on_rating: EventHandler<Option<String>>,
) -> Element {
    let pos_n = positive.len();
    let mut f = use_signal(|| {
        let mut f = Feedback::default();
        match rating.as_deref() {
            Some("up") => f.press(Thumb::Up, pos_n > 0),
            Some("down") => f.press(Thumb::Down, true),
            _ => {}
        }
        f
    });
    let mut fails = use_signal(|| fail_send);
    let fb = f.read().clone();
    let reasons = if fb.rating == Some(Thumb::Down) {
        negative.clone()
    } else {
        positive.clone()
    };
    let rid = "oxc-fb-panel";
    let mut send = move || {
        if f.write().submit() {}
    };
    rsx! {
        div { class: cls(&["oxc-fb", fb.phase.id()]),
            div { class: "fr",
                {leading}
                button { class: cls(&["th", if fb.rating == Some(Thumb::Up) { "on" } else { "" }]), r#type: "button", disabled, "aria-pressed": "{fb.rating == Some(Thumb::Up)}", "aria-label": "Good response",
                    "aria-expanded": if fb.rating == Some(Thumb::Up) { "{fb.phase == FeedbackPhase::Form}" }, "aria-controls": "{rid}",
                    onclick: move |_| { f.write().press(Thumb::Up, pos_n > 0); on_rating.call(f.peek().rating.map(|_| "up".to_string())) },
                    svg { view_box: "0 0 24 24", width: "15", height: "15", path { d: "M7 11v9H4v-9zM7 11l4-7a2 2 0 0 1 3 2l-1 4h6a2 2 0 0 1 2 2.3l-1.4 6A2 2 0 0 1 17.6 20H7" } }
                }
                button { class: cls(&["th", if fb.rating == Some(Thumb::Down) { "on" } else { "" }]), r#type: "button", disabled, "aria-pressed": "{fb.rating == Some(Thumb::Down)}", "aria-label": "Bad response",
                    "aria-expanded": if fb.rating == Some(Thumb::Down) { "{fb.phase == FeedbackPhase::Form}" }, "aria-controls": "{rid}",
                    onclick: move |_| { f.write().press(Thumb::Down, true); on_rating.call(f.peek().rating.map(|_| "down".to_string())) },
                    svg { view_box: "0 0 24 24", width: "15", height: "15", style: "transform:rotate(180deg)", path { d: "M7 11v9H4v-9zM7 11l4-7a2 2 0 0 1 3 2l-1 4h6a2 2 0 0 1 2 2.3l-1.4 6A2 2 0 0 1 17.6 20H7" } }
                }
            }
            if fb.phase != FeedbackPhase::Closed {
                div { class: "fp", id: "{rid}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); f.write().close() },
                    match fb.phase {
                        FeedbackPhase::Thanks => rsx! {
                            div { class: "fthx", role: "status",
                                svg { view_box: "0 0 24 24", width: "16", height: "16", class: "tick", path { d: "m5 12.5 4.5 4.5L19 7.5", "pathLength": "1" } }
                                div { div { class: "t1", "Thanks for the feedback" } div { class: "t2", "It helps improve future replies." } }
                                span { class: "oxyn-timer", style: "animation-duration:1600ms", onanimationend: move |_| f.write().close() }
                            }
                        },
                        _ => rsx! {
                            div { class: "fq", if fb.rating == Some(Thumb::Down) { "What went wrong?" } else { "What was good?" } }
                            div { class: "fchips", role: "group", "aria-label": "Reasons",
                                for r in reasons.iter().cloned() {
                                    button { key: "{r}", class: cls(&["fc", if fb.reasons.contains(&r) { "on" } else { "" }]), r#type: "button", "aria-pressed": "{fb.reasons.contains(&r)}",
                                        onclick: { let r = r.clone(); move |_| f.write().toggle_reason(&r) }, "{r}" }
                                }
                            }
                            if allow_comment && fb.rating == Some(Thumb::Down) {
                                textarea { class: "fn", placeholder: "Add details (optional)", "aria-label": "Add details", value: "{fb.comment}", rows: "2",
                                    oninput: move |e| f.write().set_comment(&e.value()),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) { e.prevent_default(); send() },
                                }
                            }
                            div { class: "ff",
                                if fb.phase == FeedbackPhase::Error { span { class: "ferr", role: "alert", Icon { name: "alert", size: 12.0 } "Couldn't send." } } else { span { class: "fpriv", "{privacy_note}" } }
                                span { class: "sp" }
                                button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| f.write().close(), "Skip" }
                                button { class: "oxyn-btn primary xs", r#type: "button", disabled: fb.phase == FeedbackPhase::Sending,
                                    onclick: move |_| send(),
                                    if fb.phase == FeedbackPhase::Sending { span { class: "oxyn-spn", style: "--s:11px" } "Sending" } else if fb.phase == FeedbackPhase::Error { "Try again" } else { "Submit" }
                                }
                                if fb.phase == FeedbackPhase::Sending {
                                    span { class: "oxyn-timer", style: "animation-duration:700ms", onanimationend: move |_| { let ok = !fails(); fails.set(false); f.write().sent(ok) } }
                                }
                            }
                        },
                    }
                }
            }
            div { class: "sr", role: "status", "{fb.announce()}" }
        }
    }
}

/// Message actions: copy, regenerate, branch and more, with [`ResponseFeedback`] thumbs. Shown on
/// hover / focus within the message (or always with `persistent`).
#[component]
pub fn MessageActions(
    /// Text to copy.
    text: String,
    /// Regenerate button.
    #[props(default = true)]
    regenerate: bool,
    /// Branch button.
    #[props(default)]
    branch: bool,
    /// Always visible.
    #[props(default)]
    persistent: bool,
    /// Trailing meta ("1.2k tokens · 4.1s").
    #[props(default)]
    meta: Option<String>,
    /// Regenerate pressed.
    #[props(default)]
    on_regenerate: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: cls(&["oxc-actions", if persistent { "persist" } else { "" }]),
            ResponseFeedback {
                leading: rsx! {
                    CopyButton { value: text.clone(), icon_only: true, variant: "plain", size: "sm", label: "Copy message" }
                    if regenerate { button { class: "oxc-ab", r#type: "button", "aria-label": "Regenerate", title: "Regenerate", onclick: move |_| on_regenerate.call(()), Icon { name: "retry", size: 14.0 } } }
                    if branch { button { class: "oxc-ab", r#type: "button", "aria-label": "Branch from here", title: "Branch from here", Icon { name: "branch", size: 14.0 } } }
                },
            }
            if let Some(m) = meta { span { class: "oxc-meta", "{m}" } }
        }
    }
}

/// Message bubble: one turn of a conversation. `user` turns sit right in a filled bubble;
/// `assistant` turns are flush prose with a model mark, optional reasoning, streamed markdown and
/// actions; `system` is a centred note; `tool` is a compact row. Oxyn plus: `variant` `bubble` /
/// `flat`, failed sends with Retry, edited marker and attachments.
#[component]
pub fn MessageBubble(
    /// `user` · `assistant` · `system` · `tool`.
    #[props(default = "assistant".to_string())]
    role: String,
    /// Markdown text.
    text: String,
    /// Author name.
    #[props(default)]
    author: Option<String>,
    /// Time label.
    #[props(default)]
    time: Option<String>,
    /// `bubble` · `flat`.
    #[props(default = "bubble".to_string())]
    variant: String,
    /// Streaming (assistant): ms since start.
    #[props(default)]
    elapsed_ms: Option<u64>,
    /// Actions row (assistant).
    #[props(default = true)]
    actions: bool,
    /// Failed send (user).
    #[props(default)]
    failed: bool,
    /// Edited marker.
    #[props(default)]
    edited: bool,
    /// Attachment names.
    #[props(default)]
    files: Vec<String>,
    /// Content above the text (reasoning, tool calls).
    #[props(default)]
    children: Element,
) -> Element {
    let me = role == "user";
    let name = author
        .clone()
        .unwrap_or(if me { "You".into() } else { "Atlas".into() });
    if role == "system" {
        return rsx! { div { class: "oxc-msg system", role: "note", span { "{text}" } } };
    }
    rsx! {
        article { class: cls(&["oxc-msg", &role, &variant, if failed { "failed" } else { "" }]), "aria-label": "{name}",
            if !me { div { class: "mav", "aria-hidden": "true", Icon { name: if role == "tool" { "tool" } else { "spark" }, size: 13.0 } } }
            div { class: "mbody",
                div { class: "mhd", span { class: "mn", "{name}" } if let Some(t) = &time { span { class: "mt", "{t}" } } if edited { span { class: "mt", "· edited" } } }
                {children}
                if !files.is_empty() { div { class: "mfiles", for f in files.iter() { span { key: "{f}", class: "mfile", Icon { name: "file", size: 12.0 } "{f}" } } } }
                div { class: "mtext",
                    if let Some(e) = elapsed_ms { StreamingMarkdown { text: text.clone(), elapsed_ms: e } } else { Markdown { text: text.clone() } }
                }
                if failed { div { class: "mfail", role: "alert", Icon { name: "alert", size: 12.0 } "Not delivered" button { class: "oxyn-btn ghost xs", r#type: "button", "Retry" } } }
                if actions && role == "assistant" && elapsed_ms.is_none() { MessageActions { text: text.clone(), meta: Some("1.2k tokens · 4.1s".to_string()) } }
            }
        }
    }
}

// ─────────────────────────────────────── Chat thread ──

/// A participant of a [`ChatThread`]: (id, name, hue).
pub type Participant = (String, String, String);

/// Chat thread: grouped messages with day separators, reactions, read receipts, a typing
/// indicator and a composer. The log is `role=log` (polite, additions); typing is a separate
/// status. Your messages sit right; delivery shows under your latest message; a failed send
/// offers Retry. Sending appends locally. Oxyn plus: agent participants with identity hues and a
/// `read_only` transcript mode.
#[component]
pub fn ChatThread(
    /// Participants.
    participants: Vec<Participant>,
    /// Your id.
    me: String,
    /// Messages, oldest first.
    messages: Vec<ChatMessage>,
    /// Typing ids.
    #[props(default)]
    typing: Vec<String>,
    /// Last read message id per participant: (participant, message id).
    #[props(default)]
    read_by: Vec<(String, String)>,
    /// "Now" for day labels (ms).
    #[props(default)]
    now_ms: u64,
    /// Composer.
    #[props(default = true)]
    composer: bool,
    /// Height px.
    #[props(default = 420.0)]
    height: f32,
    /// Sent.
    #[props(default)]
    on_send: EventHandler<String>,
) -> Element {
    let mut list = use_signal(|| messages.clone());
    let mut draft = use_signal(String::new);
    let mut picker = use_signal(|| None::<String>);
    let msgs = list.read().clone();
    let now = if now_ms == 0 {
        msgs.last().map(|m| m.at).unwrap_or(0)
    } else {
        now_ms
    };
    let items = group(&msgs, 300_000, now);
    let name_of = |id: &str| {
        participants
            .iter()
            .find(|p| p.0 == id)
            .map(|p| p.1.clone())
            .unwrap_or("Unknown".into())
    };
    let hue_of = |id: &str| {
        participants
            .iter()
            .find(|p| p.0 == id)
            .map(|p| p.2.clone())
            .unwrap_or_default()
    };
    let typing_names: Vec<String> = typing
        .iter()
        .filter(|t| **t != me)
        .map(|t| name_of(t))
        .collect();
    let my_last = msgs.iter().rposition(|m| m.author == me);
    rsx! {
        div { class: "oxc-thread", style: "height:{height}px",
            ol { class: "tlog", role: "log", "aria-live": "polite", "aria-relevant": "additions", "aria-label": "Conversation",
                for (k, it) in items.iter().enumerate() {
                    match it {
                        ThreadItem::Day(d) => rsx! { li { key: "d{k}", class: "tday", span { "{d}" } } },
                        ThreadItem::Group { author, messages } => {
                            let mine = *author == me;
                            let nm = name_of(author);
                            let hue = hue_of(author);
                            rsx! {
                                li { key: "g{k}", class: cls(&["tgrp", if mine { "me" } else { "" }]),
                                    if !mine { span { class: "tav", style: if !hue.is_empty() { "--hue:var(--oxyn-color-agent-{hue})" } else { "" }, "aria-hidden": "true", {nm.chars().next().unwrap_or('?').to_string()} } }
                                    div { class: "tcol",
                                        if !mine { div { class: "tname", "{nm}" span { class: "ttime", {clock(msgs[messages[0]].at)} } } }
                                        for &mi in messages.iter() {
                                            {
                                                let m = msgs[mi].clone();
                                                let readers: Vec<String> = read_by.iter().filter(|(p, id)| *id == m.id && *p != me).map(|(p, _)| name_of(p)).collect();
                                                let mid = m.id.clone();
                                                rsx! {
                                                    div { key: "{m.id}", class: "tmsg",
                                                        div { class: cls(&["tb", if m.status == Delivery::Failed { "failed" } else { "" }]), "{m.text}"
                                                            if !m.files.is_empty() { div { class: "tfiles", for f in m.files.iter() { span { key: "{f}", class: "tfile", Icon { name: "file", size: 12.0 } "{f}" } } } }
                                                        }
                                                        div { class: "trx",
                                                            for (e, n, mine_r) in m.reactions.iter().cloned() {
                                                                button { key: "{e}", class: cls(&["rx", if mine_r { "mine" } else { "" }]), r#type: "button", "aria-pressed": "{mine_r}", "aria-label": format!("{e} {n}{}", if mine_r { ", including you" } else { "" }),
                                                                    onclick: { let mid = mid.clone(); let e = e.clone(); move |_| { let mut l = list.write(); if let Some(mm) = l.iter_mut().find(|x| x.id == mid) { if let Some(r) = mm.reactions.iter_mut().find(|r| r.0 == e) { if r.2 { r.1 -= 1; r.2 = false } else { r.1 += 1; r.2 = true } } mm.reactions.retain(|r| r.1 > 0); } } },
                                                                    "{e} " span { key: "{n}", "{n}" }
                                                                }
                                                            }
                                                            span { class: "rxadd",
                                                                button { class: "rxb", r#type: "button", "aria-label": "Add reaction", "aria-expanded": "{picker() == Some(m.id.clone())}", onclick: { let mid = mid.clone(); move |_| picker.set(if picker() == Some(mid.clone()) { None } else { Some(mid.clone()) }) }, "☺" }
                                                                if picker() == Some(m.id.clone()) {
                                                                    span { class: "rxp", role: "toolbar", "aria-label": "Reactions",
                                                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { picker.set(None) },
                                                                        for e in ["👍", "❤️", "😂", "🎉", "👀", "🙏"] {
                                                                            button { key: "{e}", r#type: "button", "aria-label": "{e}", onclick: { let mid = mid.clone(); move |_| { let mut l = list.write(); if let Some(mm) = l.iter_mut().find(|x| x.id == mid) { if let Some(r) = mm.reactions.iter_mut().find(|r| r.0 == e) { if !r.2 { r.1 += 1; r.2 = true } } else { mm.reactions.push((e.to_string(), 1, true)) } } drop(l); picker.set(None) } }, "{e}" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        if !readers.is_empty() { div { class: "tread", "aria-label": "Read by {readers.join(\", \")}", for r in readers.iter() { span { key: "{r}", class: "trd", "aria-hidden": "true", {r.chars().next().unwrap_or('?').to_string()} } } } }
                                                        if mine && Some(mi) == my_last {
                                                            div { class: "tstat", if m.status == Delivery::Failed { span { class: "tfail", Icon { name: "alert", size: 11.0 } "{m.status.label()}" } button { class: "oxyn-btn ghost xs", r#type: "button", onclick: { let mid = mid.clone(); move |_| { if let Some(mm) = list.write().iter_mut().find(|x| x.id == mid) { mm.status = Delivery::Sent } } }, "Retry" } } else { "{m.status.label()} · {clock(m.at)}" } }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !typing_names.is_empty() { li { class: "ttyping", "aria-hidden": "true", span { class: "dots", i {} i {} i {} } "{typing_label(&typing_names)}" } }
            }
            div { class: "sr", role: "status", "{typing_label(&typing_names)}" }
            if composer {
                div { class: "tcomp",
                    button { class: "oxc-ab", r#type: "button", "aria-label": "Attach files", Icon { name: "clip", size: 15.0 } }
                    textarea { rows: "1", placeholder: "Message", "aria-label": "Message", value: "{draft}",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: { let me = me.clone(); move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            let t = draft();
                            if !t.trim().is_empty() { let at = list.peek().last().map(|m| m.at + 30_000).unwrap_or(0); let id = format!("n{}", list.peek().len()); list.write().push(ChatMessage::new(&id, &me, t.trim(), at)); on_send.call(t); draft.set(String::new()); }
                        }},
                    }
                    button { class: "stsend", r#type: "button", "aria-label": "Send", disabled: draft().trim().is_empty(),
                        onclick: move |_| { let t = draft(); let at = list.peek().last().map(|m| m.at + 30_000).unwrap_or(0); let id = format!("n{}", list.peek().len()); list.write().push(ChatMessage::new(&id, &me, t.trim(), at)); on_send.call(t); draft.set(String::new()); },
                        Icon { name: "up", size: 15.0, stroke: 2.0 }
                    }
                }
            }
        }
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
