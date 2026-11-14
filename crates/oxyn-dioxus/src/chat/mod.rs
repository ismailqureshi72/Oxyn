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

use crate::{CodeBlock, CopyButton, Rating, composer::icons::Icon};

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

fn fmt_ms(ms: u64) -> String {
    oxyn_core::chat::reasoning::fmt_secs(ms / 1000)
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

/// Thinking traces (Beautiful UI Thinking port, MIT © 2026 Shane Levine): expandable traces of
/// `steps`, `reasoning`, `search` or `coding`. Rows play in one at a time from `elapsed_ms`,
/// each with a pending · active · done · failed mark; the header summarises ("Searched 4
/// sources"). Oxyn plus: a failed row and the `step_ms` pace.
#[component]
pub fn ThinkingTraces(
    /// Kind.
    #[props(default = "steps".to_string())]
    kind: String,
    /// Rows: (label, detail).
    rows: Vec<(String, String)>,
    /// Milliseconds since the trace started (`u64::MAX` = finished).
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// Time per row.
    #[props(default = 900)]
    step_ms: u64,
    /// Row index that fails.
    #[props(default)]
    fail_at: Option<usize>,
    /// Start open.
    #[props(default = true)]
    open: bool,
) -> Element {
    let mut is_open = use_signal(|| open);
    let k = TraceKind::from_id(&kind);
    let tr = Trace::new(
        k,
        rows.iter()
            .enumerate()
            .map(|(i, (a, b))| {
                TraceRow::new(
                    a,
                    if b.is_empty() { None } else { Some(b) },
                    if fail_at == Some(i) {
                        RowStatus::Failed
                    } else {
                        RowStatus::Pending
                    },
                )
            })
            .collect(),
    );
    let shown = tr.playback(
        Duration::from_millis(elapsed_ms.min(u64::MAX / 2)),
        Duration::from_millis(step_ms),
    );
    let summary = tr.summary(&shown);
    let busy = shown.iter().any(|r| r.status == RowStatus::Active);
    rsx! {
        div { class: cls(&["oxc-trace", k.id(), if busy { "busy" } else { "" }]),
            button { class: "th", r#type: "button", "aria-expanded": "{is_open()}", onclick: move |_| { let o = !is_open(); is_open.set(o) },
                span { class: "ti", Icon { name: k.icon(), size: 13.0 } }
                span { class: cls(&["ts", if busy { "oxc-shim" } else { "" }]), key: "{summary}", "{summary}" }
                span { class: "tc", Icon { name: "chevron", size: 12.0 } }
            }
            if is_open() {
                ol { class: "tl",
                    for (i, r) in shown.iter().enumerate() {
                        li { key: "{i}", class: "tr {r.status.id()}",
                            span { class: "tm", "aria-label": "{r.status.id()}",
                                match r.status { RowStatus::Done => rsx! { Icon { name: "check", size: 11.0, stroke: 2.2 } }, RowStatus::Failed => rsx! { Icon { name: "x", size: 11.0, stroke: 2.2 } }, RowStatus::Active => rsx! { span { class: "oxyn-spn", style: "--s:10px" } }, RowStatus::Pending => rsx! { span { class: "pd" } } }
                            }
                            span { class: "tlb", "{r.label}" }
                            if let Some(d) = &r.detail { code { class: "tdt", "{d}" } }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────── Source cards ──

/// An inline `[n]` citation chip (Arc UI SourceCitation); highlights its card.
#[component]
pub fn SourceCitation(
    /// 1-based number.
    n: u32,
    /// Name ("Source 2: Title, domain").
    #[props(default)]
    label: Option<String>,
    /// Highlighted.
    #[props(default)]
    active: bool,
    /// Pressed.
    #[props(default)]
    on_click: EventHandler<u32>,
) -> Element {
    rsx! { button { class: cls(&["oxc-cite", if active { "on" } else { "" }]), r#type: "button", "aria-label": label.unwrap_or(format!("Source {n}")), onclick: move |_| on_click.call(n), "{n}" } }
}

/// Source cards: cited sources under an answer. Searching shows a spinner, "Searching 3 of 7",
/// the query and reserved skeleton slots; cards fade in as they arrive (one per `step_ms` from
/// `elapsed_ms`). Done shows "7 sources · 2.1s"; extras collapse into a favicon stack that opens
/// the rest; a citation highlights its card for 2.6 s; hover / focus shows a preview with the
/// snippet. Error shows "Search failed" with Retry.
#[component]
pub fn SourceCards(
    /// Sources in citation order.
    sources: Vec<Source>,
    /// `searching` · `done` · `error`.
    #[props(default = "done".to_string())]
    status: String,
    /// Expected total (searching).
    #[props(default)]
    total: Option<usize>,
    /// Query.
    #[props(default)]
    query: Option<String>,
    /// Search time ms (done).
    #[props(default)]
    duration_ms: Option<u64>,
    /// Cards before the stack.
    #[props(default = 3)]
    max: usize,
    /// Searching: ms since start (one source per `step_ms`).
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// Arrival pace.
    #[props(default = 500)]
    step_ms: u64,
    /// Highlighted (1-based).
    #[props(default)]
    highlighted: Option<usize>,
    /// Error detail.
    #[props(default)]
    error: Option<String>,
    /// Start with the stack open.
    #[props(default)]
    expanded: bool,
    /// The answer, between the status line and the cards.
    #[props(default)]
    children: Element,
    /// Retry pressed.
    #[props(default)]
    on_retry: EventHandler<()>,
) -> Element {
    let mut open = use_signal(|| expanded);
    let mut hover = use_signal(|| None::<usize>);
    let st = SearchStatus::from_id(&status);
    let arrived = if st == SearchStatus::Searching {
        ((elapsed_ms.min(u64::MAX / 2) / step_ms.max(1)) as usize).min(sources.len())
    } else {
        sources.len()
    };
    let mut m = CardsModel::new(sources[..arrived].to_vec(), st);
    m.total = total.or(if st == SearchStatus::Searching {
        Some(sources.len())
    } else {
        None
    });
    m.max = max;
    m.expanded = open();
    let slots = m.slots();
    let line = m.status_line(duration_ms);
    let srcs = m.sources.clone();
    rsx! {
        section { class: cls(&["oxc-sources", st.id()]), "aria-label": "Answer with sources",
            div { class: "ssl", "aria-hidden": "true",
                match st { SearchStatus::Searching => rsx! { span { class: "oxyn-spn", style: "--s:11px" } }, SearchStatus::Done => rsx! { Icon { name: "globe", size: 12.0 } }, SearchStatus::Error => rsx! { Icon { name: "alert", size: 12.0 } } }
                span { class: cls(&["sst", if st == SearchStatus::Searching { "oxc-shim" } else { "" }]), key: "{line}", "{line}" }
                if let Some(q) = query.clone().filter(|_| st == SearchStatus::Searching || srcs.is_empty()) { span { class: "sq", "“{q}”" } }
                if st == SearchStatus::Error { if let Some(e) = error.clone() { span { class: "sq", "{e}" } } button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| on_retry.call(()), "Retry" } }
            }
            {children}
            ul { class: "scards", "aria-label": "Sources", "aria-busy": "{st == SearchStatus::Searching}", style: "--cols:{max + 1}",
                for (k, s) in slots.iter().cloned().enumerate() {
                    match s {
                        Slot::Card(i) => {
                            let src = srcs[i].clone();
                            let n = i + 1;
                            let dom = src.domain();
                            rsx! {
                                li { key: "c{i}", class: cls(&["sc", if highlighted == Some(n) { "is-hl" } else { "" }]),
                                    onpointerenter: move |_| hover.set(Some(i)), onpointerleave: move |_| hover.set(None),
                                    a { href: "{src.url}", target: "_blank", rel: "noreferrer", "aria-label": "{src.card_label(n)}", onfocus: move |_| hover.set(Some(i)), onblur: move |_| hover.set(None),
                                        div { class: "sh", span { class: "fav", "{dom.chars().next().unwrap_or('•').to_uppercase()}" } span { class: "dom", "{dom}" } span { class: "num", "{n}" } }
                                        div { class: "stt", "{src.title}" }
                                    }
                                    if hover() == Some(i) && src.snippet.is_some() {
                                        div { class: "spv", role: "tooltip",
                                            div { class: "sh", span { class: "fav", "{dom.chars().next().unwrap_or('•').to_uppercase()}" } span { class: "dom", {src.site.clone().unwrap_or(dom.clone())} } if let Some(p) = &src.published { span { class: "pub", "{p}" } } }
                                            div { class: "stt", "{src.title}" }
                                            p { {src.snippet.clone().unwrap_or_default()} }
                                        }
                                    }
                                }
                            }
                        }
                        Slot::Skeleton => rsx! { li { key: "k{k}", class: "sc sk", "aria-hidden": "true", i {} i {} i {} } },
                        Slot::Stack(from) => {
                            let rest: Vec<Source> = srcs[from..].to_vec();
                            let more = rest.len();
                            rsx! {
                                li { key: "stack-{from}", class: "sc stack",
                                    button { r#type: "button", "aria-expanded": "false", "aria-label": "Show {more} more sources", onclick: move |_| open.set(true),
                                        div { class: "favs", for (j, r) in rest.iter().take(4).enumerate() { span { key: "{j}", class: "fav", style: "--j:{j}", "{r.domain().chars().next().unwrap_or('•').to_uppercase()}" } } }
                                        div { class: "stt", "+{more} more" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if open() && srcs.len() > max { button { class: "sless", r#type: "button", "aria-expanded": "true", onclick: move |_| open.set(false), "Show fewer" } }
            div { class: "sr", role: "status", "{line}" }
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

// ─────────────────────────────────────── AI answer ──

/// AI answer (affectUI AI Answer, Arc UI AI answer) and cited answer (affectUI Cited Answer): a
/// question, a streamed answer with inline citations, source cards, follow-up suggestions and
/// feedback. `variant: "cited"` lists claims with provenance (sourced · inferred · unverified)
/// and a grounding meter. Pressing a citation highlights its card.
#[component]
pub fn AiAnswer(
    /// Question.
    question: String,
    /// Answer markdown with `[n]` citations.
    answer: String,
    /// Sources.
    sources: Vec<Source>,
    /// Claims (cited variant).
    #[props(default)]
    claims: Vec<Claim>,
    /// `answer` · `cited`.
    #[props(default = "answer".to_string())]
    variant: String,
    /// ms since the answer started (search, then stream).
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// Search phase length ms.
    #[props(default = 1600)]
    search_ms: u64,
    /// Follow-up suggestions.
    #[props(default)]
    follow_ups: Vec<String>,
    /// A follow-up was picked.
    #[props(default)]
    on_follow_up: EventHandler<String>,
) -> Element {
    let mut hl = use_signal(|| None::<u32>);
    let searching = elapsed_ms < search_ms;
    let st = if searching { "searching" } else { "done" };
    let stream_t = elapsed_ms.saturating_sub(search_ms);
    let done = !searching
        && revealed_at(
            answer.chars().count(),
            Duration::from_millis(stream_t.min(u64::MAX / 2)),
            160.0,
        ) >= answer.chars().count();
    let cited_n = citations(&parse(&answer)).len();
    rsx! {
        section { class: cls(&["oxc-answer", &variant]), "aria-label": "Answer",
            h3 { class: "aq", "{question}" }
            SourceCards { sources: sources.clone(), status: st, elapsed_ms: elapsed_ms, step_ms: (search_ms / sources.len().max(1) as u64).max(1), duration_ms: Some(search_ms), highlighted: hl().map(|n| n as usize), query: Some(question.clone()),
                if !searching {
                    div { class: "abody",
                        if variant == "cited" && done && !claims.is_empty() {
                            div { class: "aclaims",
                                div { class: "agr", "aria-label": "Grounding {(grounding(&claims) * 100.0) as u32} percent",
                                    span { "Grounded" } div { class: "agb", i { style: "width:{grounding(&claims) * 100.0}%" } } span { class: "agn", "{(grounding(&claims) * 100.0) as u32}%" }
                                }
                                for (i, c) in claims.iter().enumerate() {
                                    p { key: "{i}", class: "acl {c.provenance.id()}",
                                        "{c.text} "
                                        match &c.provenance {
                                            Provenance::Sourced(v) => rsx! { for n in v.iter() { SourceCitation { key: "{n}", n: *n as u32, active: hl() == Some(*n as u32), on_click: move |n| hl.set(Some(n)) } } },
                                            p => rsx! { span { class: "aprov", title: "{p.label()}", Icon { name: if matches!(p, Provenance::Inferred) { "spark" } else { "alert" }, size: 11.0 } "{p.label()}" } },
                                        }
                                    }
                                }
                            }
                        } else {
                            StreamingMarkdown { text: answer.clone(), elapsed_ms: stream_t, cps: 160.0, active_cite: hl(), on_cite: move |n| hl.set(Some(n)) }
                        }
                    }
                }
            }
            if done {
                div { class: "afoot",
                    MessageActions { text: answer.clone(), persistent: true, meta: Some(format!("{cited_n} citations")) }
                }
                if !follow_ups.is_empty() {
                    div { class: "afu", "aria-label": "Follow-ups",
                        for q in follow_ups.iter().cloned() { button { key: "{q}", class: "afq", r#type: "button", onclick: { let q = q.clone(); move |_| on_follow_up.call(q.clone()) }, Icon { name: "chevron-right", size: 12.0 } "{q}" } }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────── Starter prompts ──

/// Starter prompts: the empty state of an AI app. A greeting, category tabs (All first, empty
/// categories hidden), a grid of starter cards with a Shuffle that crossfades them with a 35 ms
/// stagger, and a small composer the cards fill (Enter sends, Shift+Enter adds a line). `loading`
/// shows placeholder cards in the same rhythm.
#[component]
pub fn StarterPrompts(
    /// Heading.
    #[props(default = "What are we working on?".to_string())]
    greeting: String,
    /// Quiet line.
    #[props(default)]
    description: Option<String>,
    /// Starters.
    prompts: Vec<Starter>,
    /// Category order: (id, label).
    #[props(default = vec![("write".into(), "Write".into()), ("code".into(), "Code".into()), ("analyze".into(), "Analyze".into()), ("plan".into(), "Plan".into())])]
    category_labels: Vec<(String, String)>,
    /// Cards at once.
    #[props(default = 4)]
    count: usize,
    /// Loading placeholders.
    #[props(default)]
    loading: bool,
    /// Composer under the cards.
    #[props(default = true)]
    composer: bool,
    /// Submitted text.
    #[props(default)]
    on_submit: EventHandler<String>,
) -> Element {
    let mut cat = use_signal(|| "all".to_string());
    let mut seed = use_signal(|| 0u32);
    let mut draft = use_signal(String::new);
    let mut live = use_signal(String::new);
    let order: Vec<&str> = category_labels.iter().map(|(a, _)| a.as_str()).collect();
    let cats = categories(&prompts, &order);
    let shown = starters_for(&prompts, &cat(), seed(), count);
    let label_of = |id: &str| -> String {
        if id == "all" {
            "All".into()
        } else {
            category_labels
                .iter()
                .find(|(a, _)| a == id)
                .map(|(_, b)| b.clone())
                .unwrap_or(id.into())
        }
    };
    rsx! {
        section { class: "oxc-starters", "aria-labelledby": "oxc-st-h",
            h2 { id: "oxc-st-h", class: "sth", "{greeting}" }
            if let Some(d) = &description { p { class: "std", "{d}" } }
            div { class: "stbar",
                div { class: "sttabs", role: "tablist", "aria-label": "Categories",
                    for c in cats.iter().cloned() {
                        button { key: "{c}", r#type: "button", role: "tab", "aria-selected": "{cat() == c}", class: if cat() == c { "on" } else { "" }, onclick: { let c = c.clone(); move |_| { cat.set(c.clone()); seed.set(0) } }, {label_of(&c)} }
                    }
                }
                button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| { seed += 1; live.set("Shuffled starters".into()) }, Icon { name: "retry", size: 12.0 } "Shuffle" }
            }
            div { class: "stgrid", style: "--n:{count}",
                if loading {
                    for i in 0..count { div { key: "l{i}", class: "stc sk", "aria-hidden": "true", i {} i {} } }
                } else {
                    for (k, s) in shown.iter().cloned().enumerate() {
                        button { key: "{seed}-{cat}-{k}", class: "stc", r#type: "button", style: "--k:{k}",
                            "aria-label": format!("{}{}, {}", s.title, s.app.as_ref().map(|a| format!(", {a}")).unwrap_or_default(), label_of(&s.category)),
                            onclick: { let s = s.clone(); move |_| { draft.set(s.prompt.clone()); live.set(format!("Picked {}", s.title)) } },
                            span { class: "stk", {label_of(&s.category)} if let Some(a) = &s.app { " · {a}" } }
                            span { class: "stt", "{s.title}" }
                            span { class: "stp", "{s.prompt}" }
                        }
                    }
                }
            }
            if composer {
                div { class: "stcomp",
                    textarea { rows: "2", placeholder: "Ask anything, or pick a starter above", "aria-label": "Message", value: "{draft}",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); let t = draft(); if !t.trim().is_empty() { on_submit.call(t); draft.set(String::new()); live.set("Sent (simulated)".into()) } },
                    }
                    button { class: "stsend", r#type: "button", "aria-label": "Send message", disabled: draft().trim().is_empty(), onclick: move |_| { let t = draft(); on_submit.call(t); draft.set(String::new()); live.set("Sent (simulated)".into()) }, Icon { name: "up", size: 15.0, stroke: 2.0 } }
                }
            }
            div { class: "sr", role: "status", "{live}" }
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

// ───────────────────────────────── Document extraction ──

/// Document extraction (Arc UI): a document preview beside fields the agent pulled out. Fields
/// appear as the scan reaches them, each with a confidence meter and page provenance; selecting
/// one highlights where it came from. Low-confidence fields need review (edit or confirm) before
/// Confirm all. Oxyn plus: a `threshold` control and a provenance highlight on the page.
#[component]
pub fn DocumentExtraction(
    /// Document name.
    #[props(default = "invoice-2026-114.pdf".to_string())]
    document: String,
    /// Fields.
    fields: Vec<Field>,
    /// ms since the scan started.
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// ms per field.
    #[props(default = 450)]
    step_ms: u64,
    /// Review threshold 0–1.
    #[props(default = 0.8)]
    threshold: f32,
) -> Element {
    let mut m = use_signal(|| {
        let mut e = Extraction::new(fields.clone());
        e.threshold = threshold;
        e
    });
    let e = m.read().clone();
    let n = e.scanned(
        Duration::from_millis(elapsed_ms.min(u64::MAX / 2)),
        Duration::from_millis(step_ms),
    );
    let scanning = n < e.fields.len();
    rsx! {
        div { class: "oxc-extract",
            div { class: "xdoc", "aria-label": "{document} preview",
                div { class: "xdh", Icon { name: "page", size: 13.0 } span { "{document}" } }
                div { class: "xpage",
                    if scanning { i { class: "xscan", style: "top:{(n as f32 / e.fields.len().max(1) as f32) * 100.0}%" } }
                    for (i, f) in e.fields.iter().enumerate() {
                        div { key: "{i}", class: cls(&["xline", if e.focus == Some(i) { "is-hl" } else { "" }, if i < n { "got" } else { "" }]), span { class: "xk", "{f.label}" } span { class: "xv", "{f.value}" } }
                    }
                }
            }
            div { class: "xfields",
                div { class: "xfh",
                    span { if scanning { span { class: "oxc-shim", "Extracting {n} of {e.fields.len()}" } } else if e.phase == oxyn_core::chat::sources::ExtractPhase::Confirmed { "Confirmed" } else { "{e.pending()} to review" } }
                    button { class: "oxyn-btn primary xs", r#type: "button", disabled: scanning || e.pending() > 0 || e.phase == oxyn_core::chat::sources::ExtractPhase::Confirmed, onclick: move |_| { m.write().confirm(); }, "Confirm all" }
                }
                for (i, f) in e.fields.iter().take(n).enumerate() {
                    div { key: "{i}", class: cls(&["xf", if f.needs_review(e.threshold) { "review" } else { "" }, if e.focus == Some(i) { "on" } else { "" }]),
                        onpointerenter: move |_| m.write().focus = Some(i), onpointerleave: move |_| m.write().focus = None,
                        label { class: "xfl", r#for: "oxc-xf-{i}", "{f.label}" span { class: "xpg", "p. {f.page}" } }
                        div { class: "xfr",
                            input { id: "oxc-xf-{i}", class: "xin", value: "{f.value}", onfocus: move |_| m.write().focus = Some(i), onchange: move |ev| m.write().review(i, Some(&ev.value())) }
                            span { class: cls(&["xc", if f.confidence >= 0.9 { "hi" } else if f.confidence >= e.threshold { "md" } else { "lo" }]), "aria-label": "Confidence {(f.confidence * 100.0) as u32} percent", i { style: "width:{f.confidence * 100.0}%" } }
                            span { class: "xpct", "{(f.confidence * 100.0) as u32}%" }
                            if f.needs_review(e.threshold) { button { class: "oxyn-btn secondary xs", r#type: "button", onclick: move |_| m.write().review(i, None), Icon { name: "check", size: 11.0 } "Looks right" } }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────── Research run ──

/// Research run (Arc UI): a deep-research job on a timeline of Planning → Searching → Reading →
/// Writing → Done, with the queries it ran, the sources it read (favicons rolling in), an overall
/// progress bar, elapsed time and Pause / Stop. Ends on a report card with source count.
#[component]
pub fn ResearchRun(
    /// Question.
    question: String,
    /// Queries.
    queries: Vec<String>,
    /// Sources (title, domain).
    sources: Vec<(String, String)>,
    /// ms since start.
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
) -> Element {
    let qs: Vec<&str> = queries.iter().map(String::as_str).collect();
    let ss: Vec<(&str, &str)> = sources
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let run = RunModel::new(&question, &qs, &ss, ClockMs(0));
    let (ph, qd, sr, o) = run.at(ClockMs(elapsed_ms.min(u64::MAX / 2)));
    let pi = ResearchPhase::ALL
        .iter()
        .position(|p| *p == ph)
        .unwrap_or(0);
    rsx! {
        section { class: cls(&["oxc-research", if ph == ResearchPhase::Done { "done" } else { "" }]), "aria-label": "Research run",
            div { class: "rrh",
                Icon { name: "globe", size: 14.0 }
                div { class: "rrq", "{question}" }
                span { class: "rrt", {fmt_ms(elapsed_ms.min(9_999_000))} }
            }
            ol { class: "rrph", "aria-label": "Phases",
                for (i, p) in ResearchPhase::ALL.iter().enumerate().take(4) {
                    li { key: "{i}", class: cls(&[if i < pi { "done" } else if i == pi { "cur" } else { "" }]), "aria-current": if i == pi { "step" }, span { class: "pd" } "{p.label()}" }
                }
            }
            div { class: "rrbar", role: "progressbar", "aria-valuenow": "{(o * 100.0) as u32}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Research progress", i { style: "width:{o * 100.0}%" } }
            div { class: "rrcols",
                div { class: "rrcol",
                    div { class: "rrl", "Queries · {qd}/{queries.len()}" }
                    for (i, q) in queries.iter().enumerate() {
                        div { key: "{i}", class: cls(&["rrqq", if i < qd { "done" } else if i == qd && ph == ResearchPhase::Searching { "cur" } else { "" }]), Icon { name: "search", size: 11.0 } span { "{q}" } }
                    }
                }
                div { class: "rrcol",
                    div { class: "rrl", "Sources read · {sr}/{sources.len()}" }
                    for (i, (t, d)) in sources.iter().enumerate().take(sr.max(if ph == ResearchPhase::Reading { sr + 1 } else { 0 })) {
                        div { key: "{i}", class: cls(&["rrs", if i == sr { "cur" } else { "" }]), span { class: "fav", {d.chars().next().unwrap_or('•').to_uppercase().to_string()} } span { class: "rrst", "{t}" } span { class: "rrsd", "{d}" } }
                    }
                }
            }
            if ph == ResearchPhase::Writing { div { class: "rrw", span { class: "oxc-shim", "Writing the report from {sources.len()} sources…" } } }
            if ph == ResearchPhase::Done {
                div { class: "rrdone", Icon { name: "check-circle", size: 16.0 } div { div { class: "rrdt", "Report ready" } div { class: "rrds", "{sources.len()} sources · {queries.len()} queries · 3 claims flagged for review" } } button { class: "oxyn-btn primary xs", r#type: "button", "Open report" } }
            } else {
                div { class: "rract", button { class: "oxyn-btn ghost xs", r#type: "button", Icon { name: "pause", size: 11.0 } "Pause" } button { class: "oxyn-btn ghost xs", r#type: "button", Icon { name: "stop", size: 11.0 } "Stop" } }
            }
        }
    }
}

// ─────────────────────────────── Support conversation ──

/// Support conversation (Arc UI): an AI assistant answers first; "Talk to a person" hands off with
/// a queue position banner, a person joins with their name, and the end asks for a rating.
/// Advance through the scripted phases with the buttons. Oxyn plus: `phase` control and a CSAT
/// rating built on [`Rating`].
#[component]
pub fn SupportConversation(
    /// `bot` · `handoff` · `human` · `resolved`.
    #[props(default = "bot".to_string())]
    phase: String,
    /// Height px.
    #[props(default = 440.0)]
    height: f32,
) -> Element {
    let start = match phase.as_str() {
        "handoff" => SupportPhase::Handoff(2),
        "human" => SupportPhase::Human,
        "resolved" => SupportPhase::Resolved,
        _ => SupportPhase::Bot,
    };
    let mut ph = use_signal(|| start);
    let p = ph();
    let order = |x: SupportPhase| match x {
        SupportPhase::Bot => 0,
        SupportPhase::Handoff(_) => 1,
        SupportPhase::Human => 2,
        SupportPhase::Resolved => 3,
    };
    let o = order(p);
    rsx! {
        div { class: cls(&["oxc-support", p.id()]), style: "height:{height}px",
            div { class: "suh", span { class: cls(&["sud", p.id()]) } span { key: "{p.banner()}", class: "sub", "{p.banner()}" } }
            div { class: "sulog", role: "log", "aria-live": "polite",
                MessageBubble { role: "user", text: "My invoice shows two charges for March." }
                MessageBubble { role: "assistant", author: Some("Support assistant".to_string()), text: "I can see two charges on **March 3**: the Pro plan ($20) and a usage overage ($4.12). The overage came from runs that passed your monthly credits. Want me to set a hard cap so it can't happen again?", actions: false }
                if o >= 1 { MessageBubble { role: "user", text: "I'd like to talk to a person." } }
                if o >= 1 { MessageBubble { role: "system", text: "Handing off to the support team · your conversation is shared with them" } }
                if o >= 2 { MessageBubble { role: "assistant", author: Some("Maya · Support".to_string()), text: "Hi! I've refunded the $4.12 overage and turned on a hard cap at 500 credits. It will show on your next invoice.", actions: false } }
                if o >= 3 { MessageBubble { role: "system", text: "Maya marked this conversation as resolved" } }
            }
            div { class: "sufoot",
                match p {
                    SupportPhase::Bot => rsx! { button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| ph.set(SupportPhase::Bot.next()), Icon { name: "users", size: 13.0 } "Talk to a person" } },
                    SupportPhase::Handoff(_) => rsx! { span { class: "oxc-shim", "Waiting for the next available person" } button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| ph.set(ph().next()), "Skip ahead" } },
                    SupportPhase::Human => rsx! { button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| ph.set(SupportPhase::Resolved), Icon { name: "check", size: 13.0 } "Mark resolved" } },
                    SupportPhase::Resolved => rsx! { Rating { label: "How did we do?", value: 0.0, size: "sm" } },
                }
            }
        }
    }
}

// ─────────────────────────────────── BUI cards ──

/// Recommendation card (Beautiful UI port, MIT © 2026 Shane Levine): a suggestion with its
/// reason, a confidence band, an impact line and evidence, plus Accept / Dismiss that settle into
/// a quiet resolved state with Undo.
#[component]
pub fn RecommendationCard(
    /// Title.
    title: String,
    /// Why.
    reason: String,
    /// 0–1.
    confidence: f32,
    /// Impact.
    #[props(default)]
    impact: Option<String>,
    /// Evidence lines.
    #[props(default)]
    evidence: Vec<String>,
) -> Element {
    let mut d = use_signal(|| Decision::Open);
    let mut r = Recommendation::new(&title, &reason, confidence);
    r.impact = impact.clone();
    r.evidence = evidence.clone();
    let dec = d();
    rsx! {
        div { class: cls(&["oxc-rec", match dec { Decision::Open => "", Decision::Accepted => "acc", Decision::Dismissed => "dis" }]),
            div { class: "rech", span { class: "reci", Icon { name: "spark", size: 13.0 } } span { class: "recl", "Recommendation" } span { class: "recb {r.band()}", "{(r.confidence * 100.0) as u32}% confidence" } }
            div { class: "rect", "{r.title}" }
            p { class: "recr", "{r.reason}" }
            if let Some(i) = &r.impact { div { class: "reci2", Icon { name: "bolt", size: 12.0 } "{i}" } }
            if !r.evidence.is_empty() { ul { class: "rece", for (i, e) in r.evidence.iter().enumerate() { li { key: "{i}", "{e}" } } } }
            div { class: "reca",
                match dec {
                    Decision::Open => rsx! { button { class: "oxyn-btn primary sm", r#type: "button", onclick: move |_| d.set(Decision::Accepted), "Accept" } button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| d.set(Decision::Dismissed), "Dismiss" } },
                    _ => rsx! { span { class: "recdone", role: "status", Icon { name: if dec == Decision::Accepted { "check" } else { "x" }, size: 12.0 } if dec == Decision::Accepted { "Accepted · the agent will apply it" } else { "Dismissed" } } button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| d.set(Decision::Open), Icon { name: "undo", size: 11.0 } "Undo" } },
                }
            }
        }
    }
}

/// Context cards (Beautiful UI port, MIT © 2026 Shane Levine): the files, docs, pages, memories
/// and tools an agent used, as small cards with kind icons and token cost; pin keeps one in
/// context, remove drops it, and the header totals the tokens.
#[component]
pub fn ContextCards(
    /// Cards.
    cards: Vec<ContextCard>,
    /// Token budget for the meter.
    #[props(default = 200_000)]
    budget: u32,
    /// `grid` · `row`.
    #[props(default = "grid".to_string())]
    layout: String,
) -> Element {
    let mut list = use_signal(|| cards.clone());
    let v = list.read().clone();
    let total = context_tokens(&v);
    rsx! {
        div { class: cls(&["oxc-ctx", &layout]),
            div { class: "cxh", span { "Context · {v.len()} items" } span { class: "cxt", "{total} / {budget} tokens" } div { class: "cxb", i { style: "width:{(total as f32 / budget as f32 * 100.0).min(100.0)}%" } } }
            div { class: "cxl",
                for (i, c) in v.iter().cloned().enumerate() {
                    div { key: "{c.title}", class: cls(&["cx", &c.kind, if c.pinned { "pin" } else { "" }]),
                        span { class: "cxi", Icon { name: c.icon(), size: 13.0 } }
                        div { class: "cxtx", div { class: "cxn", "{c.title}" } div { class: "cxm", "{c.meta} · {c.tokens} tok" } }
                        button { class: "cxa", r#type: "button", "aria-pressed": "{c.pinned}", "aria-label": "Pin {c.title}", title: "Pin", onclick: move |_| { if let Some(x) = list.write().get_mut(i) { x.pinned = !x.pinned } }, Icon { name: "lock", size: 12.0 } }
                        button { class: "cxa", r#type: "button", "aria-label": "Remove {c.title}", title: "Remove", disabled: c.pinned, onclick: move |_| { list.write().remove(i); }, Icon { name: "x", size: 12.0 } }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────── AiChat ──

/// AI chat (Arc UI AI chat block, Beautiful UI Chat): a full conversation surface: a user turn,
/// an assistant turn with reasoning, a tool trace, a streamed answer with citations and sources,
/// actions, and a composer slot. Plays from `elapsed_ms`.
#[component]
pub fn AiChat(
    /// ms since the reply started.
    #[props(default = u64::MAX)]
    elapsed_ms: u64,
    /// Show the reasoning block.
    #[props(default = true)]
    reasoning: bool,
    /// Show sources.
    #[props(default = true)]
    sources: bool,
    /// Height px.
    #[props(default = 520.0)]
    height: f32,
    /// Composer (rendered under the log).
    #[props(default)]
    children: Element,
) -> Element {
    let think_ms = if reasoning { 3200 } else { 0 };
    let t = elapsed_ms.min(u64::MAX / 2);
    let thinking = t < think_ms;
    let after = t.saturating_sub(think_ms);
    let srcs = vec![
        Source::new("RFC 9068: JWT Profile for OAuth 2.0 Access Tokens", "https://www.rfc-editor.org/rfc/rfc9068").snippet("Resource servers MUST check the exp claim, allowing for a small leeway for clock skew."),
        Source::new("jsonwebtoken crate: Validation::leeway", "https://docs.rs/jsonwebtoken/latest/jsonwebtoken/struct.Validation.html").snippet("Add some leeway (in seconds) to the exp and nbf validation to account for clock skew."),
        Source::new("Flaky tests at the minute boundary", "https://github.com/acme/atlas/issues/482"),
    ];
    let answer = "The test fails when a token is minted in the last second of a minute: `exp` is checked a few milliseconds later and the strict comparison rejects it [1].\n\n**Fix**: add a small leeway to the expiry check, as the crate supports [2]:\n\n```rust\nlet mut v = Validation::new(Algorithm::RS256);\nv.leeway = 30; // seconds\n```\n\nThis matches the issue reported in #482 [3].";
    rsx! {
        div { class: "oxc-aichat", style: "height:{height}px",
            div { class: "aclog", role: "log", "aria-live": "polite", "aria-label": "Conversation",
                MessageBubble { role: "user", text: "Why does auth.rs fail at the minute boundary?", time: Some("14:02".to_string()) }
                MessageBubble { role: "assistant", text: answer.to_string(), elapsed_ms: if thinking { Some(0) } else if after > 9_000 { None } else { Some(after) }, time: Some("14:02".to_string()),
                    if reasoning {
                        ReasoningBlock { reasoning: "**Reading the test**\nThe failure only shows when the token is minted at :59.\n\n**Checking the validator**\nThe expiry check uses a strict less-than with no leeway.\n\n**Plan**\nAdd a 30 second leeway and a regression test that freezes the clock.", status: if thinking { "thinking" } else { "done" }, elapsed_ms: t, seconds: if thinking { None } else { Some(3) }, variant: "plain" }
                    }
                    if !thinking && sources {
                        SourceCards { sources: srcs.clone(), status: if after < 1200 { "searching" } else { "done" }, elapsed_ms: after, step_ms: 400, duration_ms: Some(1200), max: 3 }
                    }
                }
            }
            {children}
        }
    }
}
