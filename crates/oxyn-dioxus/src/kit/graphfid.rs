//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32): the exact **Agent Graph**
//! ([`ToolNetwork`]). Behaviour and look only; the model is `oxyn_core::kit::graphfid`. Styles:
//! `graphfid.css` (classes `oxgr-*`), in `PRIMITIVES_CSS`. Follows the nearest `data-cstyle`
//! (Clean / Classic) unless `style` is set; time comes in as `now`.
//!
//! The layout is computed twice – wide (756 × 454 design box) and narrow (360 × 432) – with x as
//! a share of the width, so it fits any container; below 520 px the narrow one shows.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::graphfid::{
        CallStatus, GRAPH_LOOP_MS, GraphCall, GraphNode, GraphScene, NodeKind, NodeStatus,
        call_duration, calls_per_min, graph_at, graph_cost, graph_demo_nodes, graph_layout,
        graph_links, link_path, link_turn,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

const WW: f32 = 756.0;
const NW: f32 = 360.0;
const NH: f32 = 432.0;
/// How long a pulse runs along its link (ms).
const PULSE: u64 = 900;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn tile_size(n: &GraphNode, narrow: bool) -> f32 {
    let s: f32 = match (n.kind, n.lead) {
        (NodeKind::Agent, true) => 56.0,
        (NodeKind::Agent, false) => 48.0,
        _ => 32.0,
    };
    if narrow { s * 0.8 } else { s }
}

/// The mark in a tile: the brand, a person for you, a magnifier for search, else initials.
fn mark(n: &GraphNode, size: f32) -> Element {
    if n.kind == NodeKind::User {
        return rsx! {
            svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", "aria-hidden": "true",
                circle { cx: "8", cy: "5.6", r: "2.6", fill: "currentColor" }
                path { d: "M3.2 13.4c.5-2.6 2.4-4 4.8-4s4.3 1.4 4.8 4", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round" }
            }
        };
    }
    if n.brand.is_empty() && n.id == "web" {
        return rsx! { span { class: "oxgr-srch", Icon { name: "search", size: size, stroke: 2.0 } } };
    }
    rsx! { BrandIcon { name: if n.brand.is_empty() { n.name.clone() } else { n.brand.clone() }, size, coloured: true, fallback: true } }
}

type Pos = Vec<(String, f32, f32)>;

fn pos_of(p: &Pos, id: &str) -> Option<(f32, f32)> {
    p.iter().find(|x| x.0 == id).map(|x| (x.1, x.2))
}

/// The path between two nodes in one layout, starting at `from`.
fn path_between(p: &Pos, from: &str, to: &str, lane: f32) -> Option<String> {
    let a = pos_of(p, from)?;
    let b = pos_of(p, to)?;
    let others: Vec<(f32, f32)> = p.iter().map(|x| (x.1, x.2)).collect();
    // Decide the turn on the link's canonical direction so both ways trace the same shape.
    let (ca, cb) = if from < to { (a, b) } else { (b, a) };
    let v = link_turn(ca, cb, &others);
    let v_here = if from < to { v } else { !v };
    Some(link_path(a, b, 12.0, lane, v_here))
}

/// Agent Graph (affectUI): agents and the tools they call as a living network – a pulse along
/// the link for every call and its answer (blue call, green answer, amber for you, red error),
/// agents that breathe while they work and turn amber when they need you, a node card with the
/// latest calls on a click, a hover that lights a node's neighbourhood, and a bar with the
/// "needs you" / "failed" chips and live totals.
#[component]
pub fn ToolNetwork(
    /// Your own graph (the demo's when `None`).
    #[props(default)]
    scene: Option<GraphScene>,
    /// Heading of the bar.
    #[props(default = "Agent graph".to_string())]
    title: String,
    /// Freeze the demo at this many ms into a pass (a settled state).
    #[props(default)]
    at: Option<u64>,
    /// The node whose card is open at the start (id).
    #[props(default)]
    open: String,
    /// How far back (ms) calls count towards calls per minute and a link's heat (Classic).
    #[props(default = 20_000)]
    heat_window: u64,
    /// Fixed height of the wide graph in px (0 = 0.6 × its 756 px design width).
    #[props(default)]
    height: f32,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A node's card opened (its id) – empty when it closes.
    #[props(default)]
    on_select: EventHandler<String>,
) -> Element {
    let demo = scene.is_none();
    let start = use_signal(move || now);
    let mut opened = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let mut hover = use_signal(|| None::<String>);
    let mut handles = use_signal(std::collections::HashMap::<String, Rc<MountedData>>::new);
    // Only a card you open moves focus into it (a card open from the start does not).
    let mut want_focus = use_signal(|| false);

    let raw = now.since(start()).as_millis() as u64;
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / GRAPH_LOOP_MS, raw % GRAPH_LOOP_MS),
        _ => (0, raw),
    };
    let sc = scene.clone().unwrap_or_else(|| graph_at(el));
    let links = graph_links(&sc.edges, &sc.calls);
    let wh = if height > 0.0 {
        height
    } else {
        (WW * 0.6).round()
    };
    // The demo keeps a place for the tool that comes and goes.
    let lay_nodes = if demo {
        graph_demo_nodes()
    } else {
        sc.nodes.clone()
    };
    let lay_links = if demo {
        graph_links(&sc.edges, &graph_at(8_000).calls)
    } else {
        links.clone()
    };
    let wpos = graph_layout(&lay_nodes, &lay_links, WW, wh);
    let npos = graph_layout(&lay_nodes, &lay_links, NW, NH);

    let agents = sc
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Agent)
        .count();
    let tools = sc.nodes.iter().filter(|n| n.kind == NodeKind::Tool).count();
    let cpm = calls_per_min(&sc.calls, el, heat_window.clamp(5_000, 60_000));
    let total: f64 = sc.nodes.iter().map(|n| n.cost).sum();
    let cost_s = graph_cost(total);
    let chips: Vec<(String, String, &'static str)> = sc
        .nodes
        .iter()
        .filter_map(|n| match n.status {
            NodeStatus::Waiting => Some((n.id.clone(), format!("{} needs you", n.name), "waiting")),
            NodeStatus::Error => Some((n.id.clone(), format!("{} failed", n.name), "error")),
            _ => None,
        })
        .collect();
    let hov = hover();
    let near = |id: &str| -> bool {
        match hov.as_deref() {
            None => true,
            Some(h) => {
                h == id
                    || links
                        .iter()
                        .any(|(a, b)| (a == h && b == id) || (b == h && a == id))
            }
        }
    };
    // Heat per link (Classic draws busier links thicker).
    let heat = |a: &str, b: &str| -> usize {
        sc.calls
            .iter()
            .filter(|c| el.saturating_sub(c.at) < heat_window && c.at <= el)
            .filter(|c| (c.from == a && c.to == b) || (c.from == b && c.to == a))
            .count()
    };
    // Pulses running now: (key, from, to, kind, delay ms).
    let mut pulses: Vec<(String, String, String, &'static str, i64)> = vec![];
    for c in sc.calls.iter() {
        if el >= c.at && el < c.at + PULSE {
            let kind = if c.to == "you" { "you" } else { "call" };
            pulses.push((
                format!("{}-{lap}-c", c.id),
                c.from.clone(),
                c.to.clone(),
                kind,
                -((el - c.at) as i64),
            ));
        }
        let back = c.at + c.ms;
        if c.status != CallStatus::Pending && el >= back && el < back + PULSE {
            let kind = if c.status == CallStatus::Error {
                "error"
            } else {
                "ok"
            };
            pulses.push((
                format!("{}-{lap}-a", c.id),
                c.to.clone(),
                c.from.clone(),
                kind,
                -((el - back) as i64),
            ));
        }
    }
    let present = |id: &str| sc.nodes.iter().any(|n| n.id == id);
    let open_id = opened().filter(|o| present(o));
    let latest = |id: &str| -> Vec<GraphCall> {
        let mut v: Vec<GraphCall> = sc
            .calls
            .iter()
            .filter(|c| c.from == id && c.at <= el)
            .cloned()
            .collect();
        v.reverse();
        v.truncate(4);
        v
    };
    let names: std::collections::HashMap<String, GraphNode> =
        sc.nodes.iter().map(|n| (n.id.clone(), n.clone())).collect();
    let legend: [(&str, &str); 4] = [
        ("call", "Call"),
        ("ok", "Answer"),
        ("you", "Needs you"),
        ("error", "Failed"),
    ];
    let legend_classic: [(&str, &str); 4] = [
        ("call", "Call"),
        ("ok", "Answer"),
        ("busy", "Busier link"),
        ("you", "Needs you"),
    ];
    let lap_k = format!("lap-{lap}");

    rsx! {
        div { class: "oxgr", "data-style": style_attr(&style), "data-hover": "{hov.is_some()}", style: "--wh:{wh}px;--nh:{NH}px",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && opened().is_some() {
                let back = opened().and_then(|o| handles.peek().get(&o).cloned());
                opened.set(None); on_select.call(String::new()); focus(back)
            },
            div { class: "oxgr-bar",
                span { class: "oxgr-glyph",
                    svg { width: "14", height: "14", view_box: "0 0 14 14", "aria-hidden": "true",
                        circle { cx: "3", cy: "7", r: "2", fill: "currentColor" }
                        circle { cx: "11", cy: "3", r: "2", fill: "currentColor" }
                        circle { cx: "11", cy: "11", r: "2", fill: "currentColor" }
                    }
                }
                b { class: "oxgr-ttl", "{title}" }
                span { class: "oxgr-chips", role: "status", "aria-live": "polite",
                    for (id, label, kind) in chips.iter().cloned() {
                        button { key: "{id}", "data-k": "{id}", class: "oxgr-chip", "data-kind": "{kind}", r#type: "button",
                            onclick: move |_| { want_focus.set(true); opened.set(Some(id.clone())); on_select.call(id.clone()) },
                            if kind == "waiting" { Icon { name: "clock", size: 12.0, stroke: 2.0 } } else { Icon { name: "alert", size: 12.0, stroke: 2.0 } }
                            "{label}"
                        }
                    }
                }
                span { class: "oxgr-sp" }
                span { class: "oxgr-stats",
                    span { b { "{agents}" } " agents" }
                    span { b { "{tools}" } " tools" }
                    span { class: "oxgr-cpm", b { "{cpm}" } " calls/min" }
                    span { b { "{cost_s}" } }
                }
            }
            div { class: "oxgr-panel",
                for (lay, pos, vw, vh) in [("wide", &wpos, WW, wh), ("narrow", &npos, NW, NH)] {
                    svg { key: "{lay}", class: "oxgr-links", "data-l": "{lay}", view_box: "0 0 {vw} {vh}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        for (a, b) in links.iter().cloned() {
                            if present(&a) && present(&b) {
                                {
                                    let d = path_between(pos, &a, &b, 0.0).unwrap_or_default();
                                    let lk = format!("{a}-{b}");
                                    let dim = !(near(&a) && near(&b));
                                    let h = heat(&a, &b).min(6);
                                    rsx! { path { key: "{lk}", "data-k": "{lk}", class: "oxgr-ln", "data-dim": "{dim}", "data-heat": "{h}", d: "{d}", path_length: "1" } }
                                }
                            }
                        }
                        for (k, f, t, kind, delay) in pulses.iter().cloned() {
                            if let Some(d) = path_between(pos, &f, &t, 0.0) {
                                g { key: "{k}", "data-k": "{k}", class: "oxgr-pl", "data-kind": "{kind}", style: "animation-delay:{delay}ms",
                                    path { class: "oxgr-glow", d: "{d}", path_length: "1", style: "animation-delay:{delay}ms" }
                                    path { class: "oxgr-core", d: "{d}", path_length: "1", style: "animation-delay:{delay}ms" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxgr-nodes", key: "{lap_k}", "data-lap": "{lap_k}",
                    for (i, n) in sc.nodes.iter().cloned().enumerate() {
                        {
                            let (wx, wy) = pos_of(&wpos, &n.id).unwrap_or((WW / 2.0, wh / 2.0));
                            let (nx, ny) = pos_of(&npos, &n.id).unwrap_or((NW / 2.0, NH / 2.0));
                            let ts = tile_size(&n, false);
                            let tn = tile_size(&n, true);
                            let is_open = open_id.as_deref() == Some(n.id.as_str());
                            let dim = !near(&n.id);
                            let calls_min = sc.calls.iter().filter(|c| c.from == n.id && el.saturating_sub(c.at) < 60_000 && c.at <= el).count();
                            let kind_s = match n.kind { NodeKind::Agent => "agent", NodeKind::Tool => "tool", NodeKind::User => "user" };
                            let label = format!("{}, {kind_s}, {}{}, {calls_min} calls in the last minute",
                                n.name, n.status.label().to_lowercase(),
                                if n.doing.is_empty() { String::new() } else { format!(": {}", n.doing) });
                            let sub = match n.status {
                                NodeStatus::Waiting => "Needs you".to_string(),
                                NodeStatus::Error => "Error".to_string(),
                                _ => n.detail.clone(),
                            };
                            let id = n.id.clone();
                            let (id1, id2, id3, id4) = (id.clone(), id.clone(), id.clone(), id.clone());
                            let side = if wx > WW * 0.55 { "left" } else { "right" };
                            let nside = if nx > NW * 0.55 { "left" } else { "right" };
                            let vdir = if wy > wh * 0.55 { "up" } else { "down" };
                            let delay = match (n.kind, n.lead) { (NodeKind::Agent, true) => 0, (NodeKind::Agent, false) => 120 + i * 40, _ => 380 + i * 30 };
                            let mk = mark(&n, ts * 0.46);
                            rsx! {
                                div { key: "{id}", "data-k": "{id}", class: "oxgr-slot", "data-open": "{is_open}",
                                    style: "--wx:{wx / WW * 100.0:.3}%;--wy:{wy}px;--nx:{nx / NW * 100.0:.3}%;--ny:{ny}px;--ts:{ts}px;--tn:{tn}px;--d:{delay}ms",
                                    button { class: "oxgr-n", r#type: "button", "data-kind": "{kind_s}", "data-lead": "{n.lead}", "data-st": "{n.status.id()}", "data-dim": "{dim}",
                                        "aria-label": "{label}", "aria-expanded": "{is_open}", "aria-haspopup": "dialog",
                                        onmounted: move |e| { handles.write().insert(id4.clone(), e.data()); },
                                        onmouseenter: move |_| hover.set(Some(id1.clone())),
                                        onmouseleave: move |_| hover.set(None),
                                        onfocus: move |_| hover.set(Some(id2.clone())),
                                        onblur: move |_| hover.set(None),
                                        onclick: move |_| {
                                            if opened().as_deref() == Some(id3.as_str()) { opened.set(None); on_select.call(String::new()) }
                                            else { want_focus.set(true); opened.set(Some(id3.clone())); on_select.call(id3.clone()) }
                                        },
                                        if n.lead { span { class: "oxgr-halo" } }
                                        span { class: "oxgr-tile", {mk}
                                            if n.status == NodeStatus::Waiting || n.status == NodeStatus::Error { span { class: "oxgr-shine" } }
                                        }
                                        match n.status {
                                            NodeStatus::Done => rsx! { span { class: "oxgr-bdg", "data-st": "done", Icon { name: "check", size: 9.0, stroke: 2.6 } } },
                                            NodeStatus::Waiting => rsx! { span { class: "oxgr-bdg", "data-st": "waiting", Icon { name: "clock", size: 9.0, stroke: 2.4 } } },
                                            NodeStatus::Error => rsx! { span { class: "oxgr-bdg", "data-st": "error", "!" } },
                                            _ => rsx! {},
                                        }
                                        span { class: "oxgr-lbl",
                                            b { span { class: "oxgr-full", "{n.name}" } span { class: "oxgr-short", "{n.short_name()}" } }
                                            if n.kind == NodeKind::Agent && !sub.is_empty() { span { class: "oxgr-sub", "{sub}" } }
                                        }
                                    }
                                    if is_open {
                                        div { class: "oxgr-card", role: "dialog", "aria-label": "{n.name}", "data-side": "{side}", "data-nside": "{nside}", "data-v": "{vdir}",
                                            div { class: "oxgr-ch",
                                                span { class: "oxgr-ct", {mark(&n, 13.0)} }
                                                span { class: "oxgr-cn", b { "{n.name}" } if !n.detail.is_empty() { span { "{n.detail}" } } }
                                                span { class: "oxgr-st", "data-st": "{n.status.id()}", "{n.status.label()}" }
                                                button { class: "oxgr-x", r#type: "button", "aria-label": "Close",
                                                    onmounted: move |e| if want_focus() { want_focus.set(false); focus(Some(e.data())) },
                                                    onclick: move |_| { opened.set(None); on_select.call(String::new()) },
                                                    Icon { name: "x", size: 13.0, stroke: 2.0 } }
                                            }
                                            if !n.doing.is_empty() { div { class: "oxgr-doing", "{n.doing}" } }
                                            if n.kind == NodeKind::Agent {
                                                div { class: "oxgr-lh", "Latest calls" }
                                                ul { class: "oxgr-calls",
                                                    for c in latest(&n.id) {
                                                        {
                                                            let to = names.get(&c.to).cloned();
                                                            let tname = to.as_ref().map(|t| t.name.clone()).unwrap_or(c.to.clone());
                                                            let pend = c.status == CallStatus::Pending || el < c.at + c.ms;
                                                            rsx! {
                                                                li { key: "{c.id}", "data-k": "{c.id}", class: "oxgr-call", "data-err": "{c.status == CallStatus::Error}",
                                                                    span { class: "oxgr-arr", "→" }
                                                                    if let Some(t) = to.as_ref() { span { class: "oxgr-cm", {mark(t, 12.0)} } }
                                                                    b { "{tname}" }
                                                                    span { class: "oxgr-cl", " · {c.label}" }
                                                                    span { class: "oxgr-sp" }
                                                                    if pend { i { class: "oxgr-spin" } } else { span { class: "oxgr-ms", "{call_duration(c.ms)}" } }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "oxgr-cf",
                                                if n.kind == NodeKind::Agent { b { "{graph_cost(n.cost)}" } }
                                                span { "{calls_min} calls in the last minute" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxgr-legend", "aria-hidden": "true",
                for (k, l) in legend.iter() { span { key: "c{k}", class: "oxgr-lg", "data-set": "clean", "data-kind": "{k}", i {} "{l}" } }
                for (k, l) in legend_classic.iter() { span { key: "k{k}", class: "oxgr-lg", "data-set": "classic", "data-kind": "{k}", i {} "{l}" } }
            }
            ul { class: "oxgr-sr",
                for n in sc.nodes.iter() {
                    {
                        let mut by: Vec<(String, usize)> = vec![];
                        for c in sc.calls.iter().filter(|c| c.from == n.id && el.saturating_sub(c.at) < 60_000 && c.at <= el) {
                            let nm = names.get(&c.to).map(|t| t.name.clone()).unwrap_or(c.to.clone());
                            match by.iter_mut().find(|x| x.0 == nm) { Some(x) => x.1 += 1, None => by.push((nm, 1)) }
                        }
                        let calls_s = by.iter().map(|(n, k)| format!("{n} {k}")).collect::<Vec<_>>().join(", ");
                        let kind_s = match n.kind { NodeKind::Agent => "agent", NodeKind::Tool => "tool", NodeKind::User => "user" };
                        let line = format!("{} ({kind_s}, {}){}{}{}", n.name, n.status.label().to_lowercase(),
                            if n.doing.is_empty() { String::new() } else { format!(": {}.", n.doing) },
                            if calls_s.is_empty() { String::new() } else { format!(" Calls in the last minute: {calls_s}.") },
                            if n.cost > 0.0 { format!(" Cost {}.", graph_cost(n.cost)) } else { String::new() });
                        rsx! { li { key: "{n.id}", "data-k": "{n.id}", "{line}" } }
                    }
                }
            }
        }
    }
}
