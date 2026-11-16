//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Trace Waterfall**
//! ([`TraceWaterfall`]). Behaviour and look only; the model is `oxyn_core::parity::tracefid`.
//! Styles: `tracefid.css` (classes `oxtw-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::tracefid::{
        TwKind, TwSpan, TwStatus, tw_axis, tw_demo, tw_dur, tw_find, tw_rows, tw_slowest, tw_stats,
        tw_tok,
    },
};

use crate::brand::BrandIcon;

static SEQ: AtomicUsize = AtomicUsize::new(0);
/// Rest at the end before the demo loops, ms of run time.
const REST_MS: u32 = 6_000;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn kind_glyph(k: TwKind) -> &'static str {
    match k {
        TwKind::Agent => "M8 2.5v11M2.5 8h11M4.1 4.1l7.8 7.8M11.9 4.1l-7.8 7.8",
        TwKind::Subagent => {
            "M8 3a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5zM3.5 13.5c.6-2.3 2.4-3.5 4.5-3.5s3.9 1.2 4.5 3.5"
        }
        TwKind::Llm => "M8 2l1.4 3.6L13 7l-3.6 1.4L8 12l-1.4-3.6L3 7l3.6-1.4z",
        TwKind::Tool => {
            "M10.5 2.5a3 3 0 0 0-2.8 4.1L2.5 11.8l1.7 1.7 5.2-5.2a3 3 0 0 0 4.1-2.8l-1.8 1.8-1.7-.4-.4-1.7z"
        }
    }
}

fn pct(ms: u32, axis: u32) -> f64 {
    ms as f64 / axis.max(1) as f64 * 100.0
}

/// Trace Waterfall (affectUI parity): an agent run as a tree of spans on a time axis – agents,
/// model calls and tool calls as tinted bars that grow live, the run's totals above, folding
/// branches into marks, and a details window per span.
#[component]
pub fn TraceWaterfall(
    /// The trace (the demo's when `None`).
    #[props(default)]
    trace: Option<TwSpan>,
    /// Ms into the run; `None` replays the demo (looping).
    #[props(default)]
    at: Option<u32>,
    /// The task line.
    #[props(default = "Put the spring sale banner live".to_string())]
    task: String,
    /// Heading.
    #[props(default = "Trace".to_string())]
    title: String,
    /// Branches folded from the start (ids).
    #[props(default)]
    folded: Vec<String>,
    /// A span whose details are open from the start (id).
    #[props(default)]
    open: Option<String>,
    /// The clock (drives the replay).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A span's details were opened (id) or closed.
    #[props(default)]
    on_open: EventHandler<Option<String>>,
) -> Element {
    // Separate demo islands each start the counter at 0, so mix in the mount time.
    let uid = use_hook(|| {
        format!(
            "{:x}{}",
            now.0 % 0xFFFFF,
            SEQ.fetch_add(1, Ordering::Relaxed)
        )
    });
    let t0 = use_signal(move || now);
    let mut fold = use_signal(move || folded.clone());
    let mut opened = use_signal(move || open.clone());
    let mut active = use_signal(|| 0usize);
    let tr = trace.clone().unwrap_or_else(tw_demo);
    let total = tr.end;
    let t = at.unwrap_or_else(|| {
        let el = now.since(t0()).as_millis() as u32;
        (el % (total + REST_MS)).min(total)
    });
    let rows = tw_rows(&tr, t, &fold());
    let st = tw_stats(&tr, t);
    let (axis, step) = tw_axis(st.duration.max(1));
    let ticks: Vec<u32> = (0..=axis / step).map(|k| k * step).collect();
    let slow = tw_slowest(&tr, t);
    let act = active().min(rows.len().saturating_sub(1));
    let act_id = rows.get(act).map(|r| format!("oxtw-{uid}-{}", r.id));
    let live = !st.done;
    let cost_s = format!("${:.2}", st.cost);
    let dur_s = tw_dur(st.duration);
    let tok_s = tw_tok(st.tokens);
    let row_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
    let row_ids2 = row_ids.clone();
    let branch_of: Vec<bool> = rows.iter().map(|r| r.branch).collect();
    let folded_of: Vec<bool> = rows.iter().map(|r| r.folded).collect();
    let done_branches: Vec<String> = tr
        .walk()
        .into_iter()
        .filter(|s| !s.children.is_empty() && s.end <= t && s.id != tr.id)
        .map(|s| s.id.clone())
        .collect();
    let any_done = !done_branches.is_empty();
    let mut toggle = move |id: String| {
        fold.with_mut(|f| {
            if let Some(p) = f.iter().position(|x| *x == id) {
                f.remove(p);
            } else {
                f.push(id);
            }
        })
    };
    let mut open_span = move |id: Option<String>| {
        opened.set(id.clone());
        on_open.call(id);
    };

    rsx! {
        section { class: "oxtw", "data-style": style_attr(&style), "data-live": live, "aria-label": "{title}",
            header { class: "oxtw-bar",
                span { class: "oxtw-logo", {icon("M8 2v3M8 11v3M2 8h3M11 8h3M3.8 3.8l2.1 2.1M10.1 10.1l2.1 2.1M3.8 12.2l2.1-2.1M10.1 5.9l2.1-2.1", 15)} }
                h2 { class: "oxtw-ttl", "{title}" }
                span { class: "oxtw-task", "{task}" }
                span { class: "oxtw-flex" }
                if any_done {
                    button { class: "oxtw-fold", r#type: "button",
                        onclick: move |_| { let mut f = fold(); for b in done_branches.iter() { if !f.contains(b) { f.push(b.clone()); } } fold.set(f); },
                        "Fold finished"
                    }
                }
                span { class: "oxtw-pill", "data-live": live, role: "status",
                    if live { i {} "Live" } else { "Done" }
                }
            }
            dl { class: "oxtw-stats", "data-live": live,
                div { dt { "Duration" } dd { "{dur_s}" } }
                div { dt { "Cost" } dd { "{cost_s}" } }
                div { dt { "Tokens" } dd { "{tok_s}" } }
                div { dt { "Model calls" } dd { "{st.llm}" } }
                div { dt { "Tool calls" } dd { "{st.tools}" } }
                div { dt { "Errors" } dd { "data-bad": st.errors > 0, "{st.errors}" } }
            }
            div { class: "oxtw-head",
                span { class: "oxtw-cap", "Span" }
                div { class: "oxtw-axis", "aria-hidden": "true",
                    for k in ticks.iter() { span { key: "{k}", "data-k": "{k}", style: "left:{pct(*k, axis)}%", "{k / 1000}s" } }
                }
            }
            div { class: "oxtw-tree", role: "tree", tabindex: "0", "aria-label": "Spans", "aria-activedescendant": act_id,
                onkeydown: move |e: KeyboardEvent| {
                    let n = row_ids.len();
                    if n == 0 { return; }
                    let a = active().min(n - 1);
                    match e.key() {
                        Key::ArrowDown => { e.prevent_default(); active.set((a + 1).min(n - 1)); }
                        Key::ArrowUp => { e.prevent_default(); active.set(a.saturating_sub(1)); }
                        Key::Home => { e.prevent_default(); active.set(0); }
                        Key::End => { e.prevent_default(); active.set(n - 1); }
                        Key::ArrowRight if branch_of[a] && folded_of[a] => { e.prevent_default(); toggle(row_ids[a].clone()); }
                        Key::ArrowLeft if branch_of[a] && !folded_of[a] => { e.prevent_default(); toggle(row_ids[a].clone()); }
                        Key::Enter => { e.prevent_default(); open_span(Some(row_ids[a].clone())); }
                        Key::Character(c) if c == " " => { e.prevent_default(); open_span(Some(row_ids[a].clone())); }
                        Key::Escape if opened().is_some() => { e.prevent_default(); open_span(None); }
                        _ => {}
                    }
                },
                div { class: "oxtw-grid", "aria-hidden": "true",
                    for k in ticks.iter() { i { key: "{k}", "data-k": "{k}", style: "left:{pct(*k, axis)}%" } }
                    if live { b { class: "oxtw-now", style: "left:{pct(t, axis)}%" } }
                }
                for (ri, r) in rows.iter().enumerate() {
                    {
                        let s = tw_find(&tr, &r.id).cloned().unwrap_or_else(|| tr.clone());
                        let stt = s.status(t);
                        let end = s.shown_end(t);
                        let id_t = s.id.clone();
                        let id_o = row_ids2[ri].clone();
                        let rid = format!("oxtw-{uid}-{}", s.id);
                        let is_open = opened().as_deref() == Some(s.id.as_str());
                        let is_slow = slow.as_deref() == Some(s.id.as_str()) && s.children.is_empty();
                        let lab = if s.children.is_empty() && s.cost > 0.0 && stt != TwStatus::Running {
                            format!("{} · ${:.2}", tw_dur(end - s.start), s.cost)
                        } else {
                            tw_dur(end - s.start)
                        };
                        let left = pct(s.start, axis);
                        let width = pct(end - s.start, axis);
                        let indent = r.depth * 14;
                        let more = r.hidden;
                        let marks = r.marks.clone();
                        let span_ms = (end - s.start).max(1);
                        let started = format!("+{}", tw_dur(s.start));
                        let tin = tw_tok(s.tokens_in as f64);
                        let tout = tw_tok(s.tokens_out as f64);
                        let cost_d = format!("${:.2}", s.cost_at(t));
                        let dur_d = tw_dur(end - s.start);
                        let kind_line = if s.detail.is_empty() { s.kind.label().to_string() } else { format!("{} · {}", s.kind.label(), s.detail) };
                        let up = ri + 1 > rows.len() / 2 && rows.len() > 6;
                        let pop_left = format!("clamp(0px, calc({left}% - 20px), calc(100% - 300px))");
                        rsx! {
                            div { class: "oxtw-row", key: "{s.id}", "data-k": "{s.id}", id: "{rid}", role: "treeitem",
                                "aria-level": "{r.depth + 1}", "aria-expanded": if r.branch { Some(!r.folded) } else { None },
                                "aria-selected": is_open, "data-active": ri == act, "data-kind": s.kind.id(), "data-st": stt.id(),
                                onclick: move |_| { active.set(ri); let v = if opened().as_deref() == Some(id_o.as_str()) { None } else { Some(id_o.clone()) }; open_span(v); },
                                div { class: "oxtw-name", style: "padding-left:{indent}px",
                                    if r.branch {
                                        button { class: "oxtw-chev", r#type: "button", tabindex: "-1", "aria-label": if r.folded { "Unfold" } else { "Fold" }, "data-folded": r.folded,
                                            onclick: move |e| { e.stop_propagation(); toggle(id_t.clone()); },
                                            {icon("M5.5 6.5L8 9l2.5-2.5", 12)}
                                        }
                                    } else { span { class: "oxtw-chev-sp" } }
                                    span { class: "oxtw-kg", "data-kind": s.kind.id(),
                                        if s.kind == TwKind::Tool && oxyn_core::brand::brand(&s.detail).is_some() {
                                            BrandIcon { name: s.detail.clone(), size: 12.0, coloured: false }
                                        } else { {icon(kind_glyph(s.kind), 12)} }
                                    }
                                    span { class: "oxtw-nm", "{s.name}" }
                                    if !s.detail.is_empty() { span { class: "oxtw-dt", "{s.detail}" } }
                                    if more > 0 { span { class: "oxtw-more", "+{more}" } }
                                    if stt == TwStatus::Failed { span { class: "oxtw-warn", title: "Failed", {icon("M8 2.5l6 11H2zM8 6.5v3M8 11.5h.01", 12)} } }
                                }
                                div { class: "oxtw-lane",
                                    span { class: "oxtw-span", "data-kind": s.kind.id(), "data-st": stt.id(), style: "left:{left}%;width:{width}%",
                                        for (mi, m) in marks.iter().enumerate() {
                                            i { class: "oxtw-mark", key: "{mi}", "data-k": "{mi}", style: "left:{(*m - s.start) as f64 / span_ms as f64 * 100.0}%" }
                                        }
                                    }
                                    span { class: "oxtw-lab", style: "left:calc({left + width}% + 6px)",
                                        "{lab}"
                                        if is_slow { em { class: "oxtw-slow", "Slowest" } }
                                    }
                                }
                                if is_open {
                                    div { class: "oxtw-pop", "data-up": up, role: "dialog", "aria-label": "{s.name} details", style: "left:{pop_left}",
                                        onclick: move |e| e.stop_propagation(),
                                        div { class: "oxtw-ph",
                                            div { class: "oxtw-pt", b { "{s.name}" } span { "{kind_line}" } }
                                            span { class: "oxtw-ps", "data-st": stt.id(), match stt { TwStatus::Running => "Running", TwStatus::Failed => "Failed", _ => "Done" } }
                                            button { class: "oxtw-x", r#type: "button", "aria-label": "Close", onclick: move |_| open_span(None), {icon("M4.5 4.5l7 7M11.5 4.5l-7 7", 12)} }
                                        }
                                        dl { class: "oxtw-pd",
                                            div { dt { "Duration" } dd { "{dur_d}" } }
                                            div { dt { "Started" } dd { "{started}" } }
                                            div { dt { "Cost" } dd { "{cost_d}" } }
                                            if s.children.is_empty() && s.kind == TwKind::Llm {
                                                div { dt { "Tokens in" } dd { "{tin}" } }
                                                div { dt { "Tokens out" } dd { "{tout}" } }
                                            }
                                        }
                                        if !s.input.is_empty() { div { class: "oxtw-io", span { "Input" } p { "{s.input}" } } }
                                        if !s.output.is_empty() && stt != TwStatus::Running {
                                            div { class: "oxtw-io", "data-err": stt == TwStatus::Failed, span { if stt == TwStatus::Failed { "Error" } else { "Output" } } p { "{s.output}" } }
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
