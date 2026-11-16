//! affectUI fidelity pass, family B part 8 (plan #32): the exact **Delegate Board**
//! ([`DelegateBoard`]). Behaviour and look only; the model is `oxyn_core::parity::delegatefid`.
//! Styles: `delegatefid.css` (classes `oxdb-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::delegatefid::{
        DbAgent, DbStatus, DbTask, db_agents, db_best, db_fits, db_load, db_statuses, db_tasks,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

/// Delegate Board (affectUI): hand work to a specific agent yourself – pick a task from the tray,
/// the agents with the right skill light up (the one with the most free slots marked best fit),
/// hand it over with a click, a digit or a drop; it queues, runs and finishes in that column.
#[component]
pub fn DelegateBoard(
    /// The agents (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<DbAgent>>,
    /// The unassigned tasks (the demo's when `None`).
    #[props(default)]
    tasks: Option<Vec<DbTask>>,
    /// Hand-overs already made: (task id, agent id).
    #[props(default)]
    assigned: Vec<(String, String)>,
    /// How long ago the `assigned` hand-overs were made, ms (0: just now, so they run in front of you).
    #[props(default)]
    assigned_ago_ms: u64,
    /// A task selected from the start.
    #[props(default)]
    selected: String,
    /// Demo time scale: ms per estimated second.
    #[props(default = 450)]
    ms_per_s: u64,
    /// Heading.
    #[props(default = "Delegate".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A task was handed over: (task id, agent id).
    #[props(default)]
    on_assign: EventHandler<(String, String)>,
    /// A hand-over was taken back (task id).
    #[props(default)]
    on_unassign: EventHandler<String>,
) -> Element {
    let ags = agents.clone().unwrap_or_else(db_agents);
    let all = tasks.clone().unwrap_or_else(db_tasks);
    // (task id, agent id, handed over at).
    let mut given = use_signal(move || {
        assigned
            .iter()
            .map(|(t, a)| {
                (
                    t.clone(),
                    a.clone(),
                    ClockMs(now.0.saturating_sub(assigned_ago_ms)),
                )
            })
            .collect::<Vec<_>>()
    });
    let mut sel = use_signal(move || (!selected.is_empty()).then(|| selected.clone()));
    let mut drag = use_signal(|| Option::<String>::None);
    let mut msg = use_signal(String::new);

    let g = given();
    let task = |id: &str| all.iter().find(|t| t.id == id).cloned();
    // Per agent: its held tasks with statuses.
    let mut rows: Vec<(String, String, DbStatus)> = vec![];
    let cols: Vec<(DbAgent, Vec<(DbTask, DbStatus)>)> = ags
        .iter()
        .map(|a| {
            let held: Vec<(DbTask, u64)> = g
                .iter()
                .filter(|(_, ai, _)| *ai == a.id)
                .filter_map(|(ti, _, at)| task(ti).map(|t| (t, now.since(*at).as_millis() as u64)))
                .collect();
            let st = db_statuses(a, &held, ms_per_s);
            let v: Vec<(DbTask, DbStatus)> = held.into_iter().map(|(t, _)| t).zip(st).collect();
            for (t, s) in &v {
                rows.push((t.id.clone(), a.id.clone(), *s));
            }
            (a.clone(), v)
        })
        .collect();
    let tray: Vec<DbTask> = all
        .iter()
        .filter(|t| !g.iter().any(|(ti, _, _)| *ti == t.id))
        .cloned()
        .collect();
    let picked = sel().and_then(|id| tray.iter().find(|t| t.id == id).cloned());
    let best = picked.as_ref().and_then(|t| db_best(&ags, t, &rows));

    let mut give = move |tid: String, aid: String, name: String, title: String| {
        let mut v = given();
        if v.iter().any(|(t, _, _)| *t == tid) {
            return;
        }
        v.push((tid.clone(), aid.clone(), now));
        given.set(v);
        sel.set(None);
        msg.set(format!("{title} – handed to the {}.", name.to_lowercase()));
        on_assign.call((tid, aid));
    };
    let n_open = rows.iter().filter(|r| r.2 != DbStatus::Done).count();

    rsx! {
        section { class: "oxdb", "data-style": style_attr(&style), "aria-label": "{title}",
            onkeydown: {
                let ags = ags.clone();
                let picked = picked.clone();
                move |e: KeyboardEvent| {
                    let k = e.key().to_string();
                    if k == "Escape" { sel.set(None); return; }
                    let Ok(d) = k.parse::<usize>() else { return };
                    if let (Some(t), Some(a)) = (picked.clone(), ags.get(d.wrapping_sub(1)))
                        && db_fits(a, &t)
                    {
                        e.prevent_default();
                        give(t.id.clone(), a.id.clone(), a.name.clone(), t.title.clone());
                    }
                }
            },
            header { class: "oxdb-bar",
                span { class: "oxdb-logo", svg { width: "16", height: "16", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", "aria-hidden": "true", path { d: "M2.5 3.5h4v4h-4zM9.5 3.5h4v4h-4zM6 11h4v2.5H6zM4.5 7.5V9h7V7.5M8 9v2" } } }
                h2 { class: "oxdb-ttl", "{title}" }
                span { class: "oxdb-sub", "{tray.len()} to hand out · {n_open} with agents" }
            }
            div { class: "oxdb-body",
                div { class: "oxdb-tray", role: "group", "aria-label": "Unassigned tasks",
                    span { class: "oxdb-cap", "Unassigned" }
                    if tray.is_empty() { p { class: "oxdb-empty", "Everything is handed out." } }
                    for t in tray.iter() {
                        {
                            let id = t.id.clone();
                            let id2 = t.id.clone();
                            let on = sel().as_deref() == Some(t.id.as_str());
                            rsx! {
                                button { class: "oxdb-task", r#type: "button", key: "{t.id}", "data-k": "{t.id}", "aria-pressed": on, draggable: "true",
                                    onclick: move |_| { let v = if sel().as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }; sel.set(v); },
                                    ondragstart: move |_| { drag.set(Some(id2.clone())); sel.set(Some(id2.clone())); },
                                    ondragend: move |_| drag.set(None),
                                    span { class: "oxdb-tt", "{t.title}" }
                                    span { class: "oxdb-tm", span { class: "oxdb-skill", "{t.skill}" } "~{t.est_s} min" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxdb-cols",
                    for (ai, (a, held)) in cols.iter().enumerate() {
                        {
                            let fit = picked.as_ref().map(|t| db_fits(a, t));
                            let is_best = best == Some(ai);
                            let load = db_load(&a.id, &rows);
                            let st = match fit { Some(true) if is_best => "best", Some(true) => "fit", Some(false) => "nofit", None => "idle" };
                            let a2 = a.clone();
                            let a3 = a.clone();
                            let p2 = picked.clone();
                            let all2 = all.clone();
                            rsx! {
                                div { class: "oxdb-col", "data-st": st, key: "{a.id}", "data-k": "{a.id}", role: "group", "aria-label": "{a.name}",
                                    ondragover: move |e| e.prevent_default(),
                                    ondrop: move |e| {
                                        e.prevent_default();
                                        if let Some(tid) = drag()
                                            && let Some(t) = all2.iter().find(|t| t.id == tid)
                                            && db_fits(&a3, t)
                                        {
                                            give(t.id.clone(), a3.id.clone(), a3.name.clone(), t.title.clone());
                                        }
                                        drag.set(None);
                                    },
                                    div { class: "oxdb-ah",
                                        span { class: "oxdb-av", BrandIcon { name: a.brand.clone(), size: 15.0, coloured: true, fallback: true } }
                                        span { class: "oxdb-an", b { "{a.name}" } small { "{a.model}" } }
                                        span { class: "oxdb-load", title: "Busy slots", "{load}/{a.capacity}" }
                                    }
                                    div { class: "oxdb-skills", for s in a.skills.iter() { span { class: "oxdb-skill", "{s}" } } }
                                    if let Some(t) = p2 {
                                        button { class: "oxdb-give", r#type: "button", disabled: fit != Some(true), "aria-keyshortcuts": "{ai + 1}",
                                            onclick: move |_| give(t.id.clone(), a2.id.clone(), a2.name.clone(), t.title.clone()),
                                            span { class: "oxdb-kb", "{ai + 1}" }
                                            if fit == Some(true) { "Hand over" } else { "Can't do {t.skill}" }
                                            if is_best { span { class: "oxdb-best", "Best fit" } }
                                        }
                                    }
                                    ul { class: "oxdb-held",
                                        for (t, s) in held.iter() {
                                            {
                                                let tid = t.id.clone();
                                                rsx! {
                                                    li { class: "oxdb-job", "data-st": s.id(), key: "{t.id}", "data-k": "{t.id}",
                                                        span { class: "oxdb-dot" }
                                                        span { class: "oxdb-jt", "{t.title}" }
                                                        span { class: "oxdb-js", "{s.label()}" }
                                                        if *s == DbStatus::Queued {
                                                            button { class: "oxdb-undo", r#type: "button", "aria-label": "Take back {t.title}",
                                                                onclick: move |_| { let mut v = given(); v.retain(|(x, _, _)| *x != tid); given.set(v); on_unassign.call(tid.clone()); },
                                                                "×" }
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
            }
            footer { class: "oxdb-foot", "aria-hidden": "true",
                if picked.is_some() { "Hand to" kbd { "1" } "–" kbd { "{ags.len()}" } "· or drop it on an agent · " kbd { "Esc" } } else { "Pick a task, then an agent" }
            }
            div { class: "oxdb-sr", role: "status", "aria-live": "polite", "{msg}" }
        }
    }
}
