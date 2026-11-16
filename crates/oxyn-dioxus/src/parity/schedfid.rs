//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Scheduled Runs**
//! ([`ScheduledRuns`]). Behaviour and look only; the model is `oxyn_core::parity::schedfid`.
//! Styles: `schedfid.css` (classes `oxsr-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set.

use dioxus::prelude::*;
use oxyn_core::parity::schedfid::{
    SR_DAYS, SR_LIMIT, SR_SLOTS, SrAgent, SrMode, SrPart, SrTask, sr_agents, sr_cell_label,
    sr_count, sr_demo, sr_fits, sr_plan,
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn metric_glyph(kind: &str) -> &'static str {
    match kind {
        "tests" => "M5.5 2.5h5M6.5 2.5v4L3 12.5a1 1 0 0 0 .9 1.5h8.2a1 1 0 0 0 .9-1.5L9.5 6.5v-4",
        "pages" => "M4 2.5h5.5L12 5v8.5H4zM9.5 2.5V5H12",
        "endpoints" => "M5.5 4.5L2.5 8l3 3.5M10.5 4.5l3 3.5-3 3.5",
        _ => "M3.5 2.5h5l4 4v7h-9zM8.5 2.5v4h4",
    }
}

fn metric_name(kind: &str) -> &'static str {
    match kind {
        "tests" => "Tests",
        "pages" => "Pages",
        "endpoints" => "Endpoints",
        _ => "Files",
    }
}

fn metric_word(kind: &str, n: u32) -> String {
    format!("{n} {}", metric_name(kind).to_lowercase())
}

/// Scheduled Runs (affectUI parity): agent tasks booked into a week of usage-limited sessions.
/// Auto books the backlog (biggest first, small tasks into the gaps, oversized tasks split
/// across sessions); Manual keeps the hand-booked runs. Drag a card onto a session, or open it
/// and pick a session, to book it by hand; filter by agent to dim the others.
#[component]
pub fn ScheduledRuns(
    /// The tasks (the demo's when `None`).
    #[props(default)]
    tasks: Option<Vec<SrTask>>,
    /// The agents for the filter (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<SrAgent>>,
    /// Starting booking mode.
    #[props(default = SrMode::Auto)]
    mode: SrMode,
    /// An agent picked from the start.
    #[props(default)]
    agent: Option<String>,
    /// A task opened from the start (id).
    #[props(default)]
    open: Option<String>,
    /// Hand bookings from the start (task id → cell).
    #[props(default)]
    booked: Vec<(String, usize)>,
    /// Heading.
    #[props(default = "This week".to_string())]
    title: String,
    /// Muted line beside the heading.
    #[props(default = "Sessions of 5 h".to_string())]
    subtitle: String,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A task was booked by hand (id, cell).
    #[props(default)]
    on_book: EventHandler<(String, usize)>,
    /// The mode changed.
    #[props(default)]
    on_mode: EventHandler<SrMode>,
    /// The agent filter changed.
    #[props(default)]
    on_filter: EventHandler<Option<String>>,
) -> Element {
    let mut md = use_signal(move || mode);
    let mut ag = use_signal(move || agent.clone());
    let mut opened = use_signal(move || open.clone());
    let mut pins = use_signal(move || booked.clone());
    let mut unpinned = use_signal(Vec::<String>::new);
    let mut drag = use_signal(|| Option::<String>::None);
    let mut over = use_signal(|| Option::<usize>::None);
    let t = tasks.clone().unwrap_or_else(sr_demo);
    let agents = agents.clone().unwrap_or_else(sr_agents);
    let ndays = SR_DAYS.len();
    let nslots = SR_SLOTS.len();
    let cells = ndays * nslots;
    let plan = sr_plan(&t, cells, md(), &pins(), &unpinned());
    let count = sr_count(&t, ag().as_deref());
    let picked = ag().and_then(|a| agents.iter().find(|x| x.id == a).cloned());
    let over_cells: Vec<usize> = (0..cells).filter(|&c| plan.load[c] > SR_LIMIT).collect();
    let status = if let Some(&c) = over_cells.first() {
        format!("{} is over its limit ({}%)", sr_cell_label(c), plan.load[c])
    } else if plan.backlog.is_empty() {
        "All tasks booked".to_string()
    } else if md() == SrMode::Auto {
        format!("{} could not be fitted", plan.backlog.len())
    } else {
        format!("{} waiting to be booked", plan.backlog.len())
    };
    let count_s = if count == 1 {
        "1 task".to_string()
    } else {
        format!("{count} tasks")
    };
    let mdk = md().id();
    let nback = plan.backlog.len();

    // Book `id` into `cell` by hand.
    let mut book = move |id: String, cell: usize| {
        let mut p = pins();
        p.retain(|x| x.0 != id);
        p.push((id.clone(), cell));
        pins.set(p);
        unpinned.with_mut(|u| u.retain(|x| *x != id));
        opened.set(None);
        on_book.call((id, cell));
    };
    let mut unbook = move |id: String| {
        pins.with_mut(|p| p.retain(|x| x.0 != id));
        unpinned.with_mut(|u| {
            if !u.contains(&id) {
                u.push(id.clone())
            }
        });
        // Taking a run back only sticks in Manual; Auto would rebook it at once.
        if md() == SrMode::Auto {
            md.set(SrMode::Manual);
            on_mode.call(SrMode::Manual);
        }
        opened.set(None);
    };

    let t2 = t.clone();
    let agents2 = agents.clone();
    let plan2 = plan.clone();
    // One card (in the backlog when `part` is None).
    let card = move |i: usize, part: Option<SrPart>, n: usize, right: bool| -> Element {
        let (t, agents, plan) = (&t2, &agents2, &plan2);
        let x = &t[i];
        let id = x.id.clone();
        let id_open = x.id.clone();
        let id_book = x.id.clone();
        let id_unbook = x.id.clone();
        let dim = ag().is_some_and(|a| a != x.agent);
        let is_open = opened().as_deref() == Some(x.id.as_str());
        let share = part.as_ref().map(|p| p.share).unwrap_or(x.need);
        let partk = part
            .as_ref()
            .filter(|p| p.of > 1)
            .map(|p| format!("{}/{}", p.k, p.of));
        let ck = part
            .as_ref()
            .map(|p| format!("{}-{}-{mdk}", x.id, p.k))
            .unwrap_or_else(|| format!("{}-b-{mdk}", x.id));
        let where_s: Vec<String> = plan
            .parts
            .iter()
            .filter(|p| p.task == i)
            .map(|p| sr_cell_label(p.cell))
            .collect();
        let where_s = if where_s.is_empty() {
            "Not booked yet".to_string()
        } else {
            where_s.join(", ")
        };
        let fits: Vec<usize> = sr_fits(plan, x.need).into_iter().take(3).collect();
        let booked_now = part.is_some();
        let agent_name = agents
            .iter()
            .find(|a| a.id == x.agent)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| x.agent.clone());
        let need_s = format!("{}% of one session", x.need);
        let pop_id = format!("oxsr-pop-{}", x.id);
        let delay = format!("--i:{n}");
        let mut dragging = drag;
        let mut overc = over;
        rsx! {
            div { class: "oxsr-cw", key: "{ck}", "data-k": "{ck}", "data-open": is_open, "data-right": right,
                button { class: "oxsr-card", r#type: "button", draggable: "true", style: "{delay}",
                    "data-agent": "{x.agent}", "data-dim": dim, "data-drag": dragging().as_deref() == Some(x.id.as_str()),
                    "aria-expanded": is_open, "aria-controls": "{pop_id}",
                    "aria-label": "{x.title}, {agent_name}, needs {share}% of a session",
                    ondragstart: move |_| { dragging.set(Some(id.clone())); opened.set(None); },
                    ondragend: move |_| { dragging.set(None); overc.set(None); },
                    onclick: move |_| { let v = if opened().as_deref() == Some(id_open.as_str()) { None } else { Some(id_open.clone()) }; opened.set(v); },
                    span { class: "oxsr-ct",
                        span { class: "oxsr-mark", BrandIcon { name: x.agent.clone(), size: 11.0, coloured: true, fallback: true } }
                        span { class: "oxsr-title", "{x.title}" }
                        if let Some(k) = partk { sup { class: "oxsr-part", "{k}" } }
                    }
                    span { class: "oxsr-desc", "{agent_name} · {x.desc}" }
                    span { class: "oxsr-cm",
                        for m in x.metrics.iter().take(2) {
                            span { class: "oxsr-met", key: "{m.kind}", "data-k": "{m.kind}", title: "{metric_word(&m.kind, m.n)}",
                                {icon(metric_glyph(&m.kind), 10)} "{m.n}"
                            }
                        }
                        span { class: "oxsr-sp" }
                        b { class: "oxsr-share", "data-big": share > SR_LIMIT, "{share}%" }
                    }
                }
                if is_open {
                    div { class: "oxsr-pop", id: "{pop_id}", role: "dialog", "aria-label": "{x.title}",
                        b { class: "oxsr-pt", "{x.title}" }
                        span { class: "oxsr-pd", "{x.desc}" }
                        dl { class: "oxsr-dl",
                            dt { "Agent" } dd { "{agent_name}" }
                            dt { "Session" } dd { "{where_s}" }
                            for m in x.metrics.iter() {
                                dt { key: "{m.kind}", "data-k": "{m.kind}", "{metric_name(&m.kind)}" }
                                dd { "{m.n}" }
                            }
                            dt { "Needs" } dd { "{need_s}" }
                        }
                        if booked_now {
                            button { class: "oxsr-pb", r#type: "button", onclick: move |_| unbook(id_unbook.clone()), "Take back to the list" }
                        } else if fits.is_empty() {
                            span { class: "oxsr-hint", "No session has room for it whole – drag it in to overbook" }
                        } else {
                            span { class: "oxsr-hint", "Drag onto a session, or book it here:" }
                            div { class: "oxsr-fits",
                                for c in fits {
                                    {
                                        let idb = id_book.clone();
                                        rsx! { button { class: "oxsr-pb", r#type: "button", key: "{c}", "data-k": "{c}", onclick: move |_| book(idb.clone(), c), "{sr_cell_label(c)}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        section { class: "oxsr", "data-style": style_attr(&style), "data-mode": mdk, "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { opened.set(None); },
            header { class: "oxsr-bar",
                h2 { class: "oxsr-ttl", "{title}" }
                span { class: "oxsr-sub", "{subtitle}" }
                span { class: "oxsr-flex" }
                div { class: "oxsr-agents", role: "group", "aria-label": "Show one agent's runs",
                    for a in agents.iter() {
                        {
                            let aid = a.id.clone();
                            let on = ag().as_deref() == Some(a.id.as_str());
                            rsx! {
                                button { class: "oxsr-dot", r#type: "button", key: "{a.id}", "data-k": "{a.id}", "aria-pressed": on, title: "{a.name}", "aria-label": "Only {a.name}",
                                    onclick: move |_| { let v = if ag().as_deref() == Some(aid.as_str()) { None } else { Some(aid.clone()) }; ag.set(v.clone()); on_filter.call(v); },
                                    BrandIcon { name: a.id.clone(), size: 12.0, coloured: true, fallback: true }
                                }
                            }
                        }
                    }
                }
                span { class: "oxsr-pill",
                    if let Some(p) = picked.as_ref() {
                        BrandIcon { name: p.id.clone(), size: 12.0, coloured: true, fallback: true }
                        "{p.name}"
                    } else { "All agents" }
                }
                span { class: "oxsr-count", "{count_s}" }
                div { class: "oxsr-seg", role: "radiogroup", "aria-label": "Booking",
                    for m in [SrMode::Auto, SrMode::Manual] {
                        button { class: "oxsr-segb", r#type: "button", role: "radio", key: "{m.id()}", "data-k": m.id(), "aria-checked": md() == m,
                            onclick: move |_| { if md() != m { md.set(m); opened.set(None); on_mode.call(m); } },
                            if m == SrMode::Auto { "Auto" } else { "Manual" }
                        }
                    }
                }
            }
            div { class: "oxsr-body",
                div { class: "oxsr-back", "data-empty": nback == 0,
                    ondragover: move |e| e.prevent_default(),
                    ondrop: move |e| { e.prevent_default(); if let Some(id) = drag() { unbook(id); } drag.set(None); },
                    div { class: "oxsr-bh", span { "To book" } span { class: "oxsr-badge", "{nback}" } }
                    if nback == 0 {
                        span { class: "oxsr-empty", "Nothing left to book" }
                    } else {
                        div { class: "oxsr-stack",
                            for (n, i) in plan.backlog.iter().enumerate() { {card(*i, None, n, false)} }
                        }
                    }
                }
                div { class: "oxsr-grid", style: "--days:{ndays};--slots:{nslots}",
                span { class: "oxsr-corner" }
                for (d, (dn, dd)) in SR_DAYS.iter().enumerate() {
                    div { class: "oxsr-day", key: "{dn}", "data-k": "{dn}", style: "grid-column:{d + 2}", span { "{dn}" } b { "{dd}" } }
                }
                for (s, slot) in SR_SLOTS.iter().enumerate() {
                    span { class: "oxsr-time", key: "{slot}", "data-k": "{slot}", style: "grid-row:{s + 2}", "{slot}" }
                    for d in 0..ndays {
                        {
                            let c = d * nslots + s;
                            let load = plan.load[c];
                            let pieces: Vec<SrPart> = plan.parts.iter().filter(|p| p.cell == c).cloned().collect();
                            let is_over = load > SR_LIMIT;
                            let bar = load.min(SR_LIMIT);
                            let lab = sr_cell_label(c);
                            rsx! {
                                div { class: "oxsr-cell", key: "{c}", "data-k": "{c}", "data-over": is_over, "data-hot": over() == Some(c),
                                    style: "grid-column:{d + 2};grid-row:{s + 2}",
                                    "aria-label": "{lab}: {load}% booked",
                                    ondragover: move |e| { e.prevent_default(); if over() != Some(c) { over.set(Some(c)); } },
                                    ondragleave: move |_| if over() == Some(c) { over.set(None); },
                                    ondrop: move |e| { e.prevent_default(); if let Some(id) = drag() { book(id, c); } drag.set(None); over.set(None); },
                                    div { class: "oxsr-stack",
                                        for (n, p) in pieces.into_iter().enumerate() { {card(p.task, Some(p.clone()), n, d + 1 == ndays)} }
                                    }
                                    div { class: "oxsr-load", title: "{load}% of the session limit booked",
                                        i { b { style: "width:{bar}%" } }
                                        span { "{load}%" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
                }
            p { class: "oxsr-status", role: "status", "data-over": !over_cells.is_empty(), "{status}" }
        }
    }
}
