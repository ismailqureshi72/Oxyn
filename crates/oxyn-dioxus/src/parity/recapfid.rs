//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32): the exact **Agent Recap**
//! ([`AgentRecap`]). Behaviour and look only; the model is `oxyn_core::parity::recapfid`.
//! Styles: `recapfid.css` (classes `oxrc-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::recapfid::{
        RECAP_BUSY_MS, RECAP_REPLAY_MS, RECAP_SINCE, RECAP_SPAN, RecapCardState, RecapEvent,
        RecapKind, recap_counts, recap_curve, recap_demo, recap_peak, recap_span, recap_time,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn lane_icon(k: RecapKind) -> Element {
    let d = match k {
        RecapKind::Needs => "M12 4a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7zM5 20a7 7 0 0 1 14 0",
        RecapKind::Failed => "M12 4 21 19H3zM12 10v4M12 16.5v.5",
        RecapKind::Done => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM8.5 12l2.5 2.5 4.5-5",
    };
    rsx! {
        svg { width: "13", height: "13", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

/// Agent Recap (affectUI): what the agents did while you were away – a small chart of the night
/// with a peak per event, then three lanes (Needs you, Failed, Done) of cards acted on in place:
/// Approve / Merge / Retry work for a moment and settle with Undo; Undo on a done card, then Redo.
#[component]
pub fn AgentRecap(
    /// Your own events (the demo's when `None`).
    #[props(default)]
    events: Option<Vec<RecapEvent>>,
    /// When the person left (minutes after midnight).
    #[props(default = RECAP_SINCE)]
    since: u32,
    /// How long they were away (minutes).
    #[props(default = RECAP_SPAN)]
    span: u32,
    /// Heading.
    #[props(default = "While you were away".to_string())]
    title: String,
    /// Replay the night when it opens (ms; 0 shows everything at once).
    #[props(default = RECAP_REPLAY_MS)]
    replay: u64,
    /// Cards that start acted on (ids): needs / failed show their done label, done ones Undone.
    #[props(default)]
    acted: Vec<String>,
    /// An event pointed at from the start (its tooltip and lit card).
    #[props(default)]
    point: String,
    /// Cards per lane the lanes keep room for.
    #[props(default = 3)]
    limit: usize,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// The main action or Review: (event id, `primary` / `review`).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
    /// Undo (event id).
    #[props(default)]
    on_undo: EventHandler<String>,
    /// Redo (event id).
    #[props(default)]
    on_redo: EventHandler<String>,
) -> Element {
    let start = use_signal(move || now);
    // id → (state, since ms).
    let mut states = use_signal(move || {
        acted
            .iter()
            .map(|id| (id.clone(), RecapCardState::Acted, 0u64))
            .collect::<Vec<(String, RecapCardState, u64)>>()
    });
    let mut hover = use_signal(move || (!point.is_empty()).then(|| point.clone()));
    let mut said = use_signal(String::new);

    let el = now.since(start()).as_millis() as u64;
    let list = events.clone().unwrap_or_else(recap_demo);
    let prog = if replay == 0 {
        1.0
    } else {
        (el as f32 / replay as f32).min(1.0)
    };
    let state_of = |id: &str| -> RecapCardState {
        match states().iter().rev().find(|s| s.0 == id) {
            Some((_, RecapCardState::Busy, at)) if el >= at + RECAP_BUSY_MS => {
                RecapCardState::Acted
            }
            Some((id2, RecapCardState::Acted, _)) => {
                // A done card "acted" from the start means undone.
                let e = list.iter().find(|e| &e.id == id2);
                if e.is_some_and(|e| e.kind == RecapKind::Done) {
                    RecapCardState::Undone
                } else {
                    RecapCardState::Acted
                }
            }
            Some((_, s, _)) => *s,
            None => RecapCardState::Open,
        }
    };
    let shown: Vec<RecapEvent> = list
        .iter()
        .filter(|e| e.at as f32 / span.max(1) as f32 <= prog + 0.0001)
        .cloned()
        .collect();
    let (d, n, f) = recap_counts(&shown, &state_of);
    let curve = recap_curve(&list, span, 241);
    let mut line = String::new();
    for (i, (x, y)) in curve.iter().enumerate() {
        line.push_str(&format!(
            "{}{:.1} {:.1}",
            if i == 0 { "M" } else { "L" },
            x * 1000.0,
            100.0 - y * 86.0
        ));
    }
    let area = format!("{line}L1000 100L0 100Z");
    let clip_w = prog * 1000.0;
    let lit = hover();
    // Hit zones: each event owns the stretch between its neighbours' midpoints.
    let mut xs: Vec<(String, f32)> = list
        .iter()
        .map(|e| (e.id.clone(), e.at as f32 / span.max(1) as f32))
        .collect();
    xs.sort_by(|a, b| a.1.total_cmp(&b.1));
    let zones: Vec<(String, f32, f32)> = xs
        .iter()
        .enumerate()
        .map(|(i, (id, x))| {
            let a = if i == 0 { 0.0 } else { (xs[i - 1].1 + x) / 2.0 };
            let b = if i + 1 == xs.len() {
                1.0
            } else {
                (xs[i + 1].1 + x) / 2.0
            };
            (id.clone(), a, b)
        })
        .collect();
    let tip = lit
        .as_ref()
        .and_then(|id| list.iter().find(|e| &e.id == id))
        .filter(|e| shown.iter().any(|s| s.id == e.id))
        .map(|e| {
            let x = e.at as f32 / span.max(1) as f32;
            let y = 100.0 - recap_peak(&list, e, span) * 86.0;
            (x, y, format!("{} · {}", recap_time(since, e.at), e.title))
        });
    let lanes = [RecapKind::Needs, RecapKind::Failed, RecapKind::Done];

    rsx! {
        div { class: "oxrc", "data-style": style_attr(&style), style: "--lim:{limit}",
            div { class: "oxrc-head",
                div { class: "oxrc-hl",
                    h3 { class: "oxrc-ttl", "{title}" }
                    span { class: "oxrc-since", "Since {recap_time(since, 0)}" span { class: "oxrc-dur", "{recap_span(span)}" } }
                }
                div { class: "oxrc-sum",
                    span { class: "oxrc-s", {lane_icon(RecapKind::Done)} b { "{d}" } " done" }
                    span { class: "oxrc-s", {lane_icon(RecapKind::Needs)} b { "{n}" } if n == 1 { " needs you" } else { " need you" } }
                    span { class: "oxrc-s", {lane_icon(RecapKind::Failed)} b { "{f}" } " failed" }
                }
            }
            div { class: "oxrc-chart",
                div { class: "oxrc-plot", onmouseleave: move |_| hover.set(None),
                    svg { class: "oxrc-svg", view_box: "0 0 1000 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        defs {
                            linearGradient { id: "oxrc-g", x1: "0", y1: "0", x2: "0", y2: "1",
                                stop { offset: "0", stop_color: "var(--oxrc-line)", stop_opacity: ".14" }
                                stop { offset: "1", stop_color: "var(--oxrc-line)", stop_opacity: "0" } }
                            clipPath { id: "oxrc-clip", rect { x: "0", y: "-10", width: "{clip_w:.1}", height: "120" } }
                        }
                        for k in 1..8 { line { key: "{k}", "data-k": "{k}", class: "oxrc-grid", x1: "{k * 125}", x2: "{k * 125}", y1: "0", y2: "100" } }
                        line { class: "oxrc-grid", x1: "0", x2: "1000", y1: "50", y2: "50" }
                        g { clip_path: "url(#oxrc-clip)",
                            path { d: "{area}", fill: "url(#oxrc-g)" }
                            path { class: "oxrc-line", d: "{line}" }
                        }
                    }
                    if let Some((x, _, _)) = tip.clone() { i { class: "oxrc-cross", style: "left:{x * 100.0:.2}%" } }
                    for e in shown.iter().cloned() {
                        {
                            let x = e.at as f32 / span.max(1) as f32 * 100.0;
                            let y = 100.0 - recap_peak(&list, &e, span) * 86.0;
                            let st = state_of(&e.id);
                            let dot = match (e.kind, st) {
                                (RecapKind::Done, RecapCardState::Undone) => "undone",
                                (_, RecapCardState::Acted) | (RecapKind::Done, _) => "done",
                                (RecapKind::Needs, _) => "needs",
                                _ => "failed",
                            };
                            let on = lit.as_deref() == Some(e.id.as_str());
                            rsx! { i { key: "{e.id}", "data-k": "{e.id}", class: "oxrc-dot", "data-st": "{dot}", "data-on": "{on}", style: "left:{x:.2}%;top:{y:.1}%" } }
                        }
                    }
                    for (id, a, b) in zones.iter().cloned() {
                        {
                            let idh = id.clone();
                            rsx! { span { key: "{id}", "data-k": "{id}", class: "oxrc-zone", style: "left:{a * 100.0:.2}%;width:{(b - a) * 100.0:.2}%", onmouseenter: move |_| hover.set(Some(idh.clone())) } }
                        }
                    }
                    if let Some((x, y, txt)) = tip { span { class: "oxrc-tip", "data-right": "{x > 0.8}", "data-left": "{x < 0.2}", style: "left:{x * 100.0:.2}%;top:{y:.1}%", "{txt}" } }
                }
                div { class: "oxrc-ends", span { "{recap_time(since, 0)}" } span { "{recap_time(since, span)}" } }
            }
            div { class: "oxrc-board",
                for k in lanes {
                    {
                        let cards: Vec<RecapEvent> = shown.iter().filter(|e| e.kind == k).cloned().collect();
                        let open_n = cards.iter().filter(|e| match (k, state_of(&e.id)) {
                            (RecapKind::Done, RecapCardState::Undone) => false,
                            (RecapKind::Done, _) => true,
                            (_, s) => s != RecapCardState::Acted,
                        }).count();
                        rsx! {
                            section { key: "{k.id()}", "data-k": "{k.id()}", class: "oxrc-lane", "aria-label": "{k.lane()}",
                                div { class: "oxrc-lh", span { class: "oxrc-li", "data-k": "{k.id()}", {lane_icon(k)} } span { "{k.lane()}" } b { class: "oxrc-n", "{open_n}" } }
                                ul { class: "oxrc-cards",
                                    for e in cards.iter().cloned() {
                                        {
                                            let st = state_of(&e.id);
                                            let on = lit.as_deref() == Some(e.id.as_str());
                                            let (ia, ir, iu, ih, il, ifo) = (e.id.clone(), e.id.clone(), e.id.clone(), e.id.clone(), e.id.clone(), e.id.clone());
                                            let (ta, tu, tr) = (e.title.clone(), e.title.clone(), e.title.clone());
                                            let quiet = st == RecapCardState::Acted || st == RecapCardState::Undone;
                                            rsx! {
                                                li { key: "{e.id}", "data-k": "{e.id}", class: "oxrc-card", "data-on": "{on}", "data-quiet": "{quiet}",
                                                    onmouseenter: move |_| hover.set(Some(ih.clone())),
                                                    onmouseleave: move |_| hover.set(None),
                                                    onfocusin: move |_| hover.set(Some(ifo.clone())),
                                                    div { class: "oxrc-c1",
                                                        span { class: "oxrc-logo",
                                                            if st == RecapCardState::Acted {
                                                                svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "var(--oxrc-green)", stroke_width: "2", stroke_linecap: "round", "aria-hidden": "true", circle { cx: "12", cy: "12", r: "9" } path { d: "M8.5 12l2.5 2.5 4.5-5" } }
                                                            } else {
                                                                BrandIcon { name: e.brand.clone(), size: 13.0, coloured: true, fallback: true }
                                                            }
                                                        }
                                                        b { class: "oxrc-ct", "{e.title}" }
                                                    }
                                                    div { class: "oxrc-c2",
                                                        span { class: "oxrc-cd", "data-k": "{e.kind.id()}", "{e.detail}" }
                                                        if e.count > 0 { span { class: "oxrc-cnt", "{e.count}" } }
                                                        span { class: "oxrc-at", "{recap_time(since, e.at)}" }
                                                    }
                                                    div { class: "oxrc-c3",
                                                        {match (e.kind, st) {
                                                            (_, RecapCardState::Busy) => rsx! {
                                                                span { class: "oxrc-work", "Working…" }
                                                                span { class: "oxrc-sp" }
                                                                if e.kind == RecapKind::Needs { button { class: "oxrc-q", r#type: "button", disabled: true, "Review" } }
                                                                button { class: "oxrc-b", r#type: "button", disabled: true, i { class: "oxrc-spin" } "{e.action}" }
                                                            },
                                                            (RecapKind::Done, RecapCardState::Undone) => rsx! {
                                                                span { class: "oxrc-tag", "data-k": "undone", "Undone" }
                                                                span { class: "oxrc-sp" }
                                                                button { class: "oxrc-q", r#type: "button", onclick: move |_| { states.write().push((ir.clone(), RecapCardState::Open, el)); said.set(format!("Redone: {tr}")); on_redo.call(ir.clone()) }, "Redo" }
                                                            },
                                                            (RecapKind::Done, _) => rsx! {
                                                                span { class: "oxrc-tag", "Done" }
                                                                span { class: "oxrc-sp" }
                                                                button { class: "oxrc-q", r#type: "button", onclick: move |_| { states.write().push((iu.clone(), RecapCardState::Undone, el)); said.set(format!("Undone: {tu}")); on_undo.call(iu.clone()) }, "Undo" }
                                                            },
                                                            (_, RecapCardState::Acted) => rsx! {
                                                                span { class: "oxrc-tag", "{e.done_label}" }
                                                                span { class: "oxrc-sp" }
                                                                button { class: "oxrc-q", r#type: "button", onclick: move |_| { states.write().push((iu.clone(), RecapCardState::Open, el)); said.set(format!("Undone: {tu}")); on_undo.call(iu.clone()) }, "Undo" }
                                                            },
                                                            _ => rsx! {
                                                                span { class: "oxrc-sp" }
                                                                if e.kind == RecapKind::Needs {
                                                                    button { class: "oxrc-q", r#type: "button", onclick: move |_| { hover.set(Some(il.clone())); on_action.call((il.clone(), "review".into())) }, "Review" }
                                                                }
                                                                button { class: "oxrc-b", r#type: "button",
                                                                    onclick: move |_| { states.write().push((ia.clone(), RecapCardState::Busy, el)); said.set(format!("{}: {ta}", if e.kind == RecapKind::Failed { "Retrying" } else { "Working on" })); on_action.call((ia.clone(), "primary".into())) },
                                                                    "{e.action}" }
                                                            },
                                                        }}
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
            div { class: "oxrc-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
