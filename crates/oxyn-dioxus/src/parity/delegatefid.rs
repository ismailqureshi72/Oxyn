//! affectUI fidelity pass, family B part 8 (plan #32): the exact **Delegate Board**
//! ([`DelegateBoard`]). Behaviour and look only; the model is `oxyn_core::parity::delegatefid`.
//! Styles: `delegatefid.css` (classes `oxdb-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::delegatefid::{
        DB_START, DbAgent, DbTask, db_agents, db_auto_plan, db_clock, db_cost, db_load_min,
        db_tasks, db_usd,
    },
};

use crate::brand::BrandIcon;

/// Auto mode hands one task over per this many ms.
const AUTO_STEP_MS: u64 = 1000;
/// Chips shown in a queue before the `+N` chip.
const SHOWN: usize = 3;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// Delegate Board (affectUI): a pile of unassigned tasks beside your agents – drag a card onto an
/// agent (or pick it up with Enter and choose with the arrows or a digit) and it joins that
/// agent's queue, pushing its "busy until" time and the money waiting in queues. Auto hands each
/// task, one a second, to whoever is free first.
#[component]
pub fn DelegateBoard(
    /// The agents (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<DbAgent>>,
    /// The tasks (the demo's when `None`); those not in `assigned` start in the pile.
    #[props(default)]
    tasks: Option<Vec<DbTask>>,
    /// Hand-overs already made: (task id, agent id).
    #[props(default)]
    assigned: Vec<(String, String)>,
    /// Start in Auto mode.
    #[props(default)]
    auto: bool,
    /// A suggested hand-over (task id, agent id): the agent lights up with its new time.
    #[props(default)]
    suggestion: Option<(String, String)>,
    /// The clock the "busy until" times count from, minutes after midnight.
    #[props(default = DB_START)]
    start: u32,
    /// Heading.
    #[props(default = "Delegate".to_string())]
    title: String,
    /// The clock (drives Auto).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A task was handed over (task id, agent id) or put back in the pile (`None`).
    #[props(default)]
    on_assign: EventHandler<(String, Option<String>)>,
    /// Auto switched on or off.
    #[props(default)]
    on_mode_change: EventHandler<bool>,
) -> Element {
    let ags = agents.clone().unwrap_or_else(db_agents);
    let all = tasks.clone().unwrap_or_else(db_tasks);
    let mut given = use_signal(move || assigned.clone());
    let mut auto_at = use_signal(move || auto.then_some(now));
    let mut held = use_signal(|| Option::<String>::None);
    let mut target = use_signal(|| 0usize);
    let mut drag = use_signal(|| Option::<String>::None);
    let mut over = use_signal(|| Option::<usize>::None);
    let mut msg = use_signal(String::new);

    // Auto: the plan from the moment it was switched on, revealed one task per step.
    let base = given();
    let g: Vec<(String, String)> = match auto_at() {
        Some(at) => {
            let plan = db_auto_plan(&ags, &all, &base);
            let k = ((now.since(at).as_millis() as u64 + 400) / AUTO_STEP_MS) as usize;
            let mut v = base.clone();
            v.extend(plan.into_iter().take(k));
            v
        }
        None => base.clone(),
    };
    let pile: Vec<DbTask> = all
        .iter()
        .filter(|t| !g.iter().any(|(ti, _)| *ti == t.id))
        .cloned()
        .collect();
    let cost = db_cost(&ags, &all, &g);
    let moving = drag().or(held());
    let moving_task = moving
        .as_ref()
        .and_then(|id| all.iter().find(|t| t.id == *id).cloned());
    let sug = suggestion
        .clone()
        .filter(|(t, _)| pile.iter().any(|x| x.id == *t));
    let auto_on = auto_at().is_some();

    let give = {
        let ags = ags.clone();
        let all = all.clone();
        move |tid: String, ai: usize| {
            let Some(a) = ags.get(ai) else { return };
            let mut v = given();
            v.retain(|(t, _)| *t != tid);
            v.push((tid.clone(), a.id.clone()));
            given.set(v);
            held.set(None);
            drag.set(None);
            over.set(None);
            let tt = all
                .iter()
                .find(|t| t.id == tid)
                .map(|t| t.title.clone())
                .unwrap_or_default();
            msg.set(format!("{tt} → {}", a.name));
            on_assign.call((tid, Some(a.id.clone())));
        }
    };
    let n_ag = ags.len();

    rsx! {
        section { class: "oxdb", "data-style": style_attr(&style), "data-auto": auto_on, "aria-label": "{title}",
            onkeydown: {
                let pile = pile.clone();
                let mut give = give.clone();
                move |e: KeyboardEvent| {
                    let Some(tid) = held() else { return };
                    let k = e.key();
                    match k {
                        Key::Enter => { e.prevent_default(); give(tid, target()); }
                        Key::Escape => { held.set(None); msg.set("Put back in the pile".into()); }
                        Key::ArrowDown => { e.prevent_default(); target.set((target() + 1) % n_ag.max(1)); }
                        Key::ArrowUp => { e.prevent_default(); target.set((target() + n_ag.max(1) - 1) % n_ag.max(1)); }
                        Key::Character(c) => {
                            if let Ok(d) = c.parse::<usize>()
                                && (1..=n_ag).contains(&d)
                                && pile.iter().any(|t| t.id == tid)
                            {
                                e.prevent_default();
                                give(tid, d - 1);
                            }
                        }
                        _ => {}
                    }
                }
            },
            header { class: "oxdb-bar",
                span { class: "oxdb-logo", {icon("M8 2.5l1.3 3.2 3.2 1.3-3.2 1.3L8 11.5 6.7 8.3 3.5 7l3.2-1.3zM12.5 11l.6 1.4 1.4.6-1.4.6-.6 1.4-.6-1.4-1.4-.6 1.4-.6z", 15)} }
                h2 { class: "oxdb-ttl", "{title}" }
                span { class: "oxdb-sub",
                    if auto_on { "Each task goes to the agent that frees up first" } else { "Drop a task on an agent to hand it over" }
                }
                span { class: "oxdb-sp" }
                label { class: "oxdb-auto",
                    span { "Auto" }
                    button { class: "oxdb-sw", r#type: "button", role: "switch", "aria-checked": auto_on, "aria-label": "Auto",
                        onclick: move |_| {
                            if auto_at().is_some() {
                                // Keep what Auto already handed out.
                                given.set(g.clone());
                                auto_at.set(None);
                                on_mode_change.call(false);
                            } else {
                                auto_at.set(Some(now));
                                held.set(None);
                                on_mode_change.call(true);
                            }
                        },
                        i {}
                    }
                }
            }
            div { class: "oxdb-body",
                div { class: "oxdb-pile", role: "group", "aria-label": "Unassigned tasks",
                    div { class: "oxdb-cap", "Unassigned" span { class: "oxdb-badge", "{pile.len()}" } }
                    if pile.is_empty() { p { class: "oxdb-empty", "Nothing left to hand out" } }
                    for t in pile.iter() {
                        {
                            let id = t.id.clone();
                            let id2 = t.id.clone();
                            let is_held = held().as_deref() == Some(t.id.as_str());
                            let is_drag = drag().as_deref() == Some(t.id.as_str());
                            let marked = sug.as_ref().is_some_and(|(x, _)| *x == t.id);
                            rsx! {
                                button { class: "oxdb-card", r#type: "button", key: "{t.id}", "data-k": "{t.id}", draggable: "true",
                                    "aria-pressed": is_held, "data-drag": is_drag, "data-sug": marked, disabled: auto_on,
                                    "aria-label": "{t.title}, about {t.est} minutes",
                                    onclick: move |_| {
                                        if held().as_deref() == Some(id.as_str()) { held.set(None); } else { held.set(Some(id.clone())); target.set(0); }
                                    },
                                    ondragstart: move |_| { drag.set(Some(id2.clone())); held.set(None); },
                                    ondragend: move |_| { drag.set(None); over.set(None); },
                                    span { class: "oxdb-ct", "{t.title}" }
                                    span { class: "oxdb-cm", BrandIcon { name: t.tool.clone(), size: 11.0, coloured: true, fallback: true } "~{t.est} min" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxdb-agents",
                    div { class: "oxdb-cap", "Agents" span { class: "oxdb-sum", "{db_usd(cost)} in queues" } }
                    for (ai, a) in ags.iter().enumerate() {
                        {
                            let load = db_load_min(a, &all, &g);
                            let queued: Vec<DbTask> = g.iter().filter(|(_, x)| *x == a.id)
                                .filter_map(|(ti, _)| all.iter().find(|t| t.id == *ti).cloned()).collect();
                            let mut chips: Vec<(Option<String>, String)> = vec![];
                            if let Some((c, _)) = &a.current { chips.push((None, c.clone())); }
                            for q in &queued { chips.push((Some(q.id.clone()), q.title.clone())); }
                            let extra = chips.len().saturating_sub(SHOWN);
                            let rest: Vec<String> = chips.iter().skip(SHOWN).map(|c| c.1.clone()).collect();
                            let is_target = held().is_some() && target() == ai;
                            let is_over = over() == Some(ai);
                            let is_sug = sug.as_ref().is_some_and(|(_, x)| *x == a.id);
                            let preview = (is_target || is_over).then(|| moving_task.clone()).flatten()
                                .or_else(|| if is_sug { sug.as_ref().and_then(|(t, _)| all.iter().find(|x| x.id == *t).cloned()) } else { None });
                            let st = if is_over || is_target { "over" } else if is_sug { "suggest" } else { "idle" };
                            let n_q = queued.len();
                            let rate = format!("${:.2}/min", a.rate);
                            let mut give_drop = give.clone();
                            let mut give_btn = give.clone();
                            rsx! {
                                div { class: "oxdb-ag", key: "{a.id}", "data-k": "{a.id}", "data-st": st, role: "group", "aria-label": "{a.name}",
                                    ondragover: move |e| { e.prevent_default(); if over() != Some(ai) { over.set(Some(ai)); } },
                                    ondragleave: move |_| if over() == Some(ai) { over.set(None); },
                                    ondrop: move |e| { e.prevent_default(); if let Some(t) = drag() { give_drop(t, ai); } },
                                    div { class: "oxdb-ah",
                                        span { class: "oxdb-av", BrandIcon { name: a.brand.clone(), size: 13.0, coloured: true, fallback: true } }
                                        span { class: "oxdb-an",
                                            span { class: "oxdb-nm", b { "{a.name}" } if auto_on && n_q > 0 { span { class: "oxdb-qn", "{n_q} queued" } } }
                                            small { "{a.model} · {rate}" }
                                        }
                                        if let Some(t) = preview {
                                            span { class: "oxdb-pill", "data-st": "preview", "Until {db_clock(start + load + t.est)}" }
                                        } else if load == 0 {
                                            span { class: "oxdb-pill", "data-st": "free", "Free" }
                                        } else {
                                            span { class: "oxdb-pill", "Busy until {db_clock(start + load)}" }
                                        }
                                        if held().is_some() {
                                            button { class: "oxdb-to", r#type: "button", "aria-keyshortcuts": "{ai + 1}", "aria-label": "Hand to {a.name}",
                                                onclick: move |_| if let Some(t) = held() { give_btn(t, ai); },
                                                "{ai + 1}" }
                                        }
                                    }
                                    div { class: "oxdb-q",
                                        if chips.is_empty() { span { class: "oxdb-none", "Nothing queued" } }
                                        for (ci, (tid, label)) in chips.iter().take(SHOWN).enumerate() {
                                            {
                                                let tid = tid.clone();
                                                let label2 = label.clone();
                                                let k = tid.clone().unwrap_or_else(|| format!("cur-{}", a.id));
                                                let run = ci == 0;
                                                rsx! {
                                                    if let Some(t) = tid {
                                                        button { class: "oxdb-chip", r#type: "button", key: "{k}", "data-k": "{k}", "data-run": run,
                                                            title: "Put back in the pile", "aria-label": "{label2} – put back in the pile", disabled: auto_on,
                                                            onclick: move |_| {
                                                                let mut v = given(); v.retain(|(x, _)| *x != t); given.set(v);
                                                                msg.set(format!("{label2} back in the pile"));
                                                                on_assign.call((t.clone(), None));
                                                            },
                                                            if run { i { class: "oxdb-run" } }
                                                            "{label}"
                                                        }
                                                    } else {
                                                        span { class: "oxdb-chip", key: "{k}", "data-k": "{k}", "data-run": "true", i { class: "oxdb-run" } "{label}" }
                                                    }
                                                }
                                            }
                                        }
                                        if extra > 0 {
                                            span { class: "oxdb-chip oxdb-more", tabindex: "0", "aria-label": "{extra} more: {rest.join(\", \")}", "+{extra}"
                                                span { class: "oxdb-tip", role: "tooltip", for r in rest.iter() { span { key: "{r}", "data-k": "{r}", "{r}" } } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "oxdb-foot", "aria-hidden": "true",
                if held().is_some() {
                    kbd { "↑" } kbd { "↓" } "choose · " kbd { "Enter" } " hand over · " kbd { "1" } "–" kbd { "{n_ag}" } " at once · " kbd { "Esc" } " put back"
                } else {
                    "Drop a card on an agent, or press " kbd { "Enter" } " – pick with " kbd { "↑" } kbd { "↓" } " and press " kbd { "Enter" } " again. Digits hand it straight over."
                }
            }
            div { class: "oxdb-sr", role: "status", "aria-live": "polite", "{msg}" }
        }
    }
}
