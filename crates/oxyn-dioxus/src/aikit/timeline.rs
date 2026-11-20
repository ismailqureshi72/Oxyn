//! Agent timeline (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::timeline::{SpanAt, SpanKind, Timeline};
use oxyn_core::aikit::{secs, thousands, usd};

use super::{ico, key_of, live, mark};

/// Agent timeline: one orchestrated run on a shared time axis. Rows are agents (orchestrator
/// first) with spans for thinking, tool calls and waiting; handoffs connect rows; a playhead
/// replays the run while the header counts time, tokens and cost, then settles into a summary.
/// Hover or focus a span for what it did; Enter or a click pins it and dims everything off its
/// handoff path. Keys: ←/→ within a row, ↑/↓ between rows, Home/End, Enter/Space pin, Escape
/// unpin; on the scrubber ←/→ half a second (Shift five), Page up/down, Home/End.
#[component]
pub fn AgentTimeline(
    /// Host clock.
    now: ClockMs,
    /// Heading.
    #[props(default = "Checkout conversion drop on Oct 14".to_string())]
    title: String,
    /// `lanes` (one row per agent) or `waterfall` (one row per span, Oxyn extra).
    #[props(default = "lanes".to_string())]
    view: String,
    /// Replay from zero on mount; otherwise the run renders finished.
    #[props(default = true)]
    autoplay: bool,
    /// Follow a run in progress (no playback controls).
    #[props(default)]
    live_run: bool,
    /// Show each span's cost on the bar and in the card (Oxyn extra).
    #[props(default)]
    show_cost: bool,
    /// Starting speed (1 or 2).
    #[props(default = 1)]
    speed: u64,
) -> Element {
    let t0 = now.0;
    let mut tl = use_signal(move || {
        let mut t = Timeline::new(autoplay, live_run, t0);
        t.speed = speed.clamp(1, 2);
        t
    });
    let mut said = use_signal(String::new);
    let mut hover = use_signal(|| None::<usize>);
    let n = now.0;
    let s = tl.read().clone();
    let t = s.t(n);
    let dur = s.duration;
    let done = s.finished(n);
    let playing = s.playing(n);
    let water = view == "waterfall";
    let rows: Vec<Vec<usize>> = if water {
        s.waterfall().into_iter().map(|i| vec![i]).collect()
    } else {
        (0..s.agents.len()).map(|a| s.row(a)).collect()
    };
    let nrows = rows.len();
    let card = hover().or(s.pinned);
    let head = if done { s.summary() } else { s.status(n) };
    let ph = t as f64 / dur.max(1) as f64 * 100.0;
    let ticks: Vec<u64> = (0..=dur / 5_000).map(|k| k * 5_000).collect();
    let row_h = if water { 26 } else { 36 };
    rsx! {
        section { class: "oxaik oxaik-tl", "aria-label": "{title}", "data-view": "{view}",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "{title}" }
                    p { class: "oxaik-sub oxaik-num",
                        if done { {ico("check-circle", 13.0)} }
                        span { class: if done { "oxaik-ok" } else { "" }, "{head}" }
                    }
                }
                if !live_run {
                    div { class: "oxaik-ctl",
                        button { class: "oxaik-ib", r#type: "button", "aria-label": if playing { "Pause" } else if done { "Replay" } else { "Play" },
                            onclick: move |_| { let x = tl.write().press("tl-play", n); if let Some(x) = x { said.set(x) } },
                            {ico(if playing { "pause" } else if done { "retry" } else { "play" }, 14.0)}
                        }
                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Restart",
                            onclick: move |_| { let x = tl.write().press("tl-restart", n); if let Some(x) = x { said.set(x) } },
                            {ico("undo", 14.0)}
                        }
                        button { class: "oxaik-chip oxaik-num", r#type: "button", "aria-label": "Speed {s.speed}x",
                            onclick: move |_| { let x = tl.write().press("tl-speed", n); if let Some(x) = x { said.set(x) } },
                            "{s.speed}x"
                        }
                    }
                } else {
                    span { class: "oxaik-pill", "data-hue": "green", span { class: "oxaik-dot" } "Live" }
                }
            }
            div { class: "oxaik-tl-body",
                div { class: "oxaik-tl-labels",
                    for (r, row) in rows.iter().enumerate() {
                        {
                            let a = &s.agents[s.spans[row[0]].agent];
                            let lab = if water { s.spans[row[0]].label.to_string() } else { a.name.to_string() };
                            let model = a.model;
                            rsx! {
                                div { key: "l{r}", class: "oxaik-tl-lab", style: "height:{row_h}px", "data-hue": "{a.hue}",
                                    span { class: "oxaik-dot" }
                                    span { class: "oxaik-tl-name", "{lab}" }
                                    if !water { span { class: "oxaik-tl-model", {mark(model, 12.0)} span { "{model}" } } }
                                }
                            }
                        }
                    }
                }
                div { class: "oxaik-tl-scroll",
                    div { class: "oxaik-tl-plot", role: "group", "aria-label": "Spans",
                        style: "height:{nrows as u32 * row_h}px",
                        onkeydown: move |e: KeyboardEvent| {
                            let k = key_of(&e);
                            if matches!(k.as_str(), "ArrowLeft"|"ArrowRight"|"ArrowUp"|"ArrowDown"|"Home"|"End"|"Enter"|" "|"Escape") {
                                e.prevent_default();
                                let x = tl.write().key_span(&k, n);
                                if let Some(x) = x { said.set(x) }
                            }
                        },
                        for x in ticks.iter() {
                            div { key: "g{x}", class: "oxaik-tl-grid", style: "left:{*x as f64 / dur as f64 * 100.0}%" }
                        }
                        for (r, row) in rows.iter().enumerate() {
                            for &i in row.iter() {
                                {
                                    let sp = &s.spans[i];
                                    let (l, w) = s.geom(i, n);
                                    let at = s.span_at(i, n);
                                    let st = match at { SpanAt::Future => "future", SpanAt::Active(_) => "active", SpanAt::Done => "done", SpanAt::Failed => "failed" };
                                    let hue = s.agents[sp.agent].hue;
                                    let dim = !s.span_on_path(i);
                                    let name = s.span_name(i);
                                    let cost = usd(s.span_cost(i));
                                    let tool = sp.tool;
                                    rsx! {
                                        button { key: "s{i}", r#type: "button", class: "oxaik-tl-span", "data-kind": "{sp.kind.id()}", "data-st": "{st}", "data-hue": "{hue}",
                                            "data-dim": "{dim}", "aria-pressed": "{s.pinned == Some(i)}", "aria-label": "{name}",
                                            tabindex: if s.focus == i { "0" } else { "-1" },
                                            style: "left:{l * 100.0}%;width:max({w * 100.0}%, 3px);top:{r as u32 * row_h + 6}px;height:{row_h - 12}px",
                                            onmouseenter: move |_| hover.set(Some(i)),
                                            onmouseleave: move |_| hover.set(None),
                                            onfocus: move |_| { hover.set(Some(i)); tl.write().focus = i; },
                                            onblur: move |_| hover.set(None),
                                            onclick: move |_| { let x = tl.write().press(&format!("tl-span-{i}"), n); if let Some(x) = x { said.set(x) } },
                                            if sp.kind == SpanKind::Tool && w * 900.0 > 24.0 { if crate::brand::has_brand(tool) { {mark(tool, 11.0)} } else { {ico("globe", 11.0)} } }
                                            if at == SpanAt::Failed { {ico("x", 11.0)} }
                                            if sp.retry && at != SpanAt::Future { {ico("retry", 11.0)} }
                                            if show_cost && at != SpanAt::Future && w > 0.08 { span { class: "oxaik-tl-cost oxaik-num", "{cost}" } }
                                        }
                                    }
                                }
                            }
                        }
                        if !water {
                            for (h, x) in s.handoffs.iter().enumerate() {
                                if s.handoff_shown(h, n) {
                                    {
                                        let top = x.from.min(x.to) as u32 * row_h + row_h / 2;
                                        let hgt = x.from.abs_diff(x.to) as u32 * row_h;
                                        let left = x.at as f64 / dur as f64 * 100.0;
                                        let up = x.to < x.from;
                                        let dim = !s.on_path(h);
                                        rsx! {
                                            div { key: "h{h}", class: "oxaik-tl-wire", "data-up": "{up}", "data-dim": "{dim}", "aria-hidden": "true", title: "{x.label}",
                                                style: "left:{left}%;top:{top}px;height:{hgt}px",
                                                i {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !done { div { class: "oxaik-tl-head", style: "left:{ph}%", "aria-hidden": "true" } }
                    }
                    div { class: "oxaik-tl-axis oxaik-num", "aria-hidden": "true",
                        for x in ticks.iter() { span { key: "a{x}", style: "left:{*x as f64 / dur as f64 * 100.0}%", "{x / 1000}s" } }
                    }
                }
            }
            if let Some(i) = card {
                {
                    let sp = &s.spans[i];
                    let a = &s.agents[sp.agent];
                    let hand: Vec<String> = s.handoffs.iter().filter(|x| (x.from == sp.agent || x.to == sp.agent) && x.at + 400 >= sp.start && x.at <= sp.end + 400).map(|x| format!("{} → {}: {}", s.agents[x.from].name, s.agents[x.to].name, x.label)).collect();
                    rsx! {
                        div { class: "oxaik-tl-card", "aria-hidden": "true",
                            div { class: "oxaik-row",
                                span { class: "oxaik-tl-k", "data-kind": "{sp.kind.id()}", "{sp.kind.label()}" }
                                if !sp.tool.is_empty() { span { class: "oxaik-row", {mark(sp.tool, 12.0)} "{sp.tool}" } }
                                if sp.failed { span { class: "oxaik-pill", "data-hue": "red", {ico("x-circle", 12.0)} "Failed" } }
                                if sp.retry { span { class: "oxaik-pill", {ico("retry", 12.0)} "Retry" } }
                            }
                            b { "{sp.label}" }
                            span { class: "oxaik-mute oxaik-num", "{a.name} · {secs(sp.start)}–{secs(sp.end)} · {secs(sp.end - sp.start)} · {thousands(sp.tokens)} tokens · {usd(s.span_cost(i))}" }
                            if !sp.detail.is_empty() { span { class: "oxaik-mute", "{sp.detail}" } }
                            for hl in hand { span { class: "oxaik-mute", "↳ {hl}" } }
                        }
                    }
                }
            }
            if !live_run {
                div { class: "oxaik-tl-foot",
                    input { class: "oxaik-range", r#type: "range", min: "0", max: "{dur}", step: "100", value: "{t}",
                        "aria-label": "Replay position", "aria-valuetext": "{secs(t)} of {secs(dur)}",
                        oninput: move |e| { if let Ok(v) = e.value().parse::<u64>() { let x = tl.write().press(&format!("tl-seek-{v}"), n); if let Some(x) = x { said.set(x) } } },
                        onkeydown: move |e: KeyboardEvent| {
                            let k = key_of(&e);
                            if matches!(k.as_str(), "ArrowLeft"|"ArrowRight"|"ArrowUp"|"ArrowDown"|"Home"|"End"|"PageUp"|"PageDown"|" ") {
                                e.prevent_default();
                                let x = tl.write().key_scrub(&k, e.modifiers().shift(), n);
                                if let Some(x) = x { said.set(x) }
                            }
                        },
                    }
                    ul { class: "oxaik-legend", "aria-label": "Legend",
                        li { i { "data-kind": "thinking" } "Thinking" }
                        li { i { "data-kind": "tool" } "Tool call" }
                        li { i { "data-kind": "waiting" } "Waiting" }
                        li { i { class: "w" } "Handoff" }
                        li { {ico("x", 11.0)} "Failed" }
                    }
                }
            }
            {live(&said())}
        }
    }
}
