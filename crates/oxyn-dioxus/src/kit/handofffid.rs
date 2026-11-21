//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32): the exact **Agent
//! Handoff** ([`RelayTrail`]). Behaviour and look only; the model is
//! `oxyn_core::kit::handofffid`. Styles: `handofffid.css` (classes `oxho-*`), in
//! `PRIMITIVES_CSS`. Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set;
//! time comes in as `now`.
//!
//! The relay is a grid of half columns: agent i spans 2i+1..2i+3, the handoff column between i and
//! i+1 spans 2i+2..2i+4 – centred under the gap. Below 620 px it stacks: agent, handoff, agent.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::handofffid::{
        HANDOFF_LOOP_MS, Handoff, HandoffItem, HandoffScene, ItemKind, TaskStatus, handoff_at,
        handoff_cost, handoff_status, items_line,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

/// Half-column width in the connector drawing's units; the offset of a line from a column's middle.
const U: f32 = 100.0;
const OFF: f32 = 11.0;
/// How long a handoff's light runs (ms).
const LIGHT: u64 = 1_400;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn kind_icon(k: ItemKind) -> &'static str {
    match k {
        ItemKind::File => "file",
        ItemKind::Source => "link",
        ItemKind::Fact => "info",
        ItemKind::Decision => "check",
        ItemKind::Note => "text",
    }
}

fn spinner() -> Element {
    rsx! {
        svg { class: "oxho-spin", width: "11", height: "11", view_box: "0 0 12 12", "aria-hidden": "true",
            circle { cx: "6", cy: "6", r: "4.5", fill: "none", stroke: "currentColor", stroke_width: "1.6", opacity: ".25" }
            path { d: "M6 1.5A4.5 4.5 0 0 1 10.5 6", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round" }
        }
    }
}

/// What a dragged or picked-up card is: (handoff id, task title, the column's `to`).
type Picked = Option<(String, String, String)>;

/// Agent Handoff (affectUI): a task passed from agent to agent like a relay – agent cards in
/// order with what each is on, and between each pair a column of what was handed over, latest on
/// top: what went along, and what got lost on the way (struck through, with why and "Add back").
/// A card dragged (or double-clicked, then an agent clicked) onto an agent to the right of its
/// column gives that task to it.
#[component]
pub fn RelayTrail(
    /// Your own relay (the demo's when `None`).
    #[props(default)]
    scene: Option<HandoffScene>,
    /// Freeze the demo at this many ms into a pass (a settled state).
    #[props(default)]
    at: Option<u64>,
    /// A handoff card open at the start (its id).
    #[props(default)]
    open: String,
    /// A lost item open at the start (its id, inside the open card).
    #[props(default)]
    open_item: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A card was dropped on an agent: (task title, agent id).
    #[props(default)]
    on_assign: EventHandler<(String, String)>,
    /// "Add back" on a lost item: (handoff id, item id).
    #[props(default)]
    on_restore: EventHandler<(String, String)>,
) -> Element {
    let demo = scene.is_none();
    let start = use_signal(move || now);
    let mut opened = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let mut item_open = use_signal(move || (!open_item.is_empty()).then(|| open_item.clone()));
    let mut agent_open = use_signal(|| None::<usize>);
    let mut picked = use_signal(|| None as Picked);
    let mut assigned = use_signal(Vec::<(u64, String, String)>::new);
    let restored = use_signal(Vec::<(u64, String, String)>::new);

    let raw = now.since(start()).as_millis() as u64;
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / HANDOFF_LOOP_MS, raw % HANDOFF_LOOP_MS),
        _ => (0, raw),
    };
    let mut sc = scene.clone().unwrap_or_else(|| handoff_at(el));
    for (l, task, agent) in assigned().iter() {
        if *l == lap {
            sc.assign(task, agent, "Checking claims");
        }
    }
    for (l, h, i) in restored().iter() {
        if *l != lap {
            continue;
        }
        if let Some(it) = sc
            .handoffs
            .iter_mut()
            .find(|x| &x.id == h)
            .and_then(|x| x.items.iter_mut().find(|it| &it.id == i))
        {
            it.lost = false;
        }
    }
    let n = sc.agents.len().max(1);
    let halves = 2 * n;
    let vw = halves as f32 * U;
    let pick = picked();
    // Lines: per column j, one from agent j into its left entry and one from agent j+1 into its
    // right entry; (key, path, x of the column end, x of the agent end).
    let mut lines: Vec<(String, String, f32, f32)> = vec![];
    for j in 0..n.saturating_sub(1) {
        let cx = (2 * j + 2) as f32 * U;
        let a0 = (2 * j + 1) as f32 * U;
        let a1 = (2 * j + 3) as f32 * U;
        let l = cx - OFF;
        let r = cx + OFF;
        lines.push((
            format!("l{j}"),
            format!(
                "M{a0} 0L{a0} 22Q{a0} 30 {} 30L{} 30Q{l} 30 {l} 38L{l} 60",
                a0 + 8.0,
                l - 8.0
            ),
            l,
            a0,
        ));
        lines.push((
            format!("r{j}"),
            format!(
                "M{a1} 0L{a1} 22Q{a1} 30 {} 30L{} 30Q{r} 30 {r} 38L{r} 60",
                a1 - 8.0,
                r + 8.0
            ),
            r,
            a1,
        ));
    }
    // Lights for handoffs that just landed: in from the left line, out along the right one.
    let mut lights: Vec<(String, usize, bool, i64)> = vec![];
    for h in sc.handoffs.iter() {
        if h.at == 0 || el < h.at || el >= h.at + LIGHT || at.is_some() {
            continue;
        }
        let Some(j) = sc.agents.iter().position(|a| a.id == h.from) else {
            continue;
        };
        let d = -((el - h.at) as i64);
        lights.push((format!("{}-{lap}-in", h.id), 2 * j, false, d));
        lights.push((format!("{}-{lap}-out", h.id), 2 * j + 1, true, d + 700));
    }
    let can = |agent: &str| -> bool {
        pick.as_ref()
            .is_some_and(|(_, _, to)| sc.can_take(to, agent))
    };

    rsx! {
        div { class: "oxho", "data-style": style_attr(&style), "data-picking": "{pick.is_some()}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape {
                if picked().is_some() { picked.set(None) } else if item_open().is_some() { item_open.set(None) }
                else if agent_open().is_some() { agent_open.set(None) } else { opened.set(None) }
            },
            div { class: "oxho-grid", style: "--n:{n};grid-template-columns:repeat({halves},1fr)",
                for (i, a) in sc.agents.iter().cloned().enumerate() {
                    {
                        let task = sc.agent_task(i);
                        let working = task.as_ref().is_some_and(|(t, up, _)| !up && t.status == TaskStatus::Working);
                        let target = can(&a.id);
                        let passed = pick.is_some() && !target;
                        let aid = a.id.clone();
                        let (aid1, aid2) = (aid.clone(), aid.clone());
                        let is_open = agent_open() == Some(i);
                        let cost = handoff_cost(a.cost);
                        let todo: Vec<(String, String)> = sc.tasks.iter().filter(|t| t.status == TaskStatus::Todo).map(|t| (t.title.clone(), t.logo.clone())).collect();
                        rsx! {
                            div { key: "{aid}", "data-k": "{aid}", class: "oxho-ag", "data-target": "{target}", "data-passed": "{passed}",
                                style: "grid-column:{2 * i + 1} / span 2;grid-row:1;--o:{2 * i}",
                                ondragover: move |e: DragEvent| if target { e.prevent_default() },
                                ondrop: move |e: DragEvent| {
                                    e.prevent_default();
                                    if let Some((_, t, _)) = picked() {
                                        assigned.write().push((lap, t.clone(), aid1.clone()));
                                        on_assign.call((t, aid1.clone()));
                                    }
                                    picked.set(None);
                                },
                                onclick: move |_| if target {
                                    if let Some((_, t, _)) = picked() {
                                        assigned.write().push((lap, t.clone(), aid2.clone()));
                                        on_assign.call((t, aid2.clone()));
                                    }
                                    picked.set(None);
                                },
                                div { class: "oxho-ah",
                                    span { class: "oxho-tile", BrandIcon { name: a.brand.clone(), size: 15.0, coloured: true, fallback: true } }
                                    span { class: "oxho-an", b { "{a.name}" } span { "{a.model}" } }
                                    span { class: "oxho-sp" }
                                    span { class: "oxho-cost", "{cost}" }
                                    span { class: "oxho-pill", "data-on": "{working}", title: if working { "Working" } else { "Idle" },
                                        if working { {spinner()} }
                                        span { class: "oxho-pt", if working { "Working" } else { "Idle" } }
                                    }
                                }
                                p { class: "oxho-desc", "{a.description}" }
                                match task.clone() {
                                    Some((t, up, more)) => rsx! {
                                        button { class: "oxho-chip", r#type: "button", "aria-expanded": "{is_open}", "aria-haspopup": "dialog",
                                            onclick: move |e| { e.stop_propagation(); agent_open.set(if agent_open() == Some(i) { None } else { Some(i) }) },
                                            if up { span { class: "oxho-up", "Up next" } }
                                            span { class: "oxho-cl", BrandIcon { name: t.logo.clone(), size: 13.0, coloured: true, fallback: true } }
                                            b { class: "oxho-ct", "{t.title}" }
                                            if !up && !t.note.is_empty() { span { class: "oxho-cn", "{t.note}" } }
                                            if up && more > 0 { span { class: "oxho-more", title: "{more} more", "+{more}" } }
                                        }
                                        if is_open {
                                            div { class: "oxho-tp", role: "dialog", "aria-label": "{t.title}",
                                                div { class: "oxho-tph",
                                                    span { class: "oxho-cl", BrandIcon { name: t.logo.clone(), size: 14.0, coloured: true, fallback: true } }
                                                    span { class: "oxho-an", b { "{t.title}" } span { if up { "Up next" } else { "{t.note}" } } }
                                                    span { class: "oxho-sp" }
                                                    button { class: "oxho-x", r#type: "button", "aria-label": "Close", onclick: move |_| agent_open.set(None), Icon { name: "x", size: 12.0, stroke: 2.0 } }
                                                }
                                                if up && todo.len() > 1 {
                                                    div { class: "oxho-tpl", "Waiting" }
                                                    ul { class: "oxho-tpu",
                                                        for (tt, lg) in todo.iter().skip(1).cloned() {
                                                            li { key: "{tt}", "data-k": "{tt}", span { class: "oxho-cl", BrandIcon { name: lg, size: 13.0, coloured: true, fallback: true } } "{tt}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    None => rsx! { div { class: "oxho-chip", "data-empty": "true", "Ready for a task" } },
                                }
                            }
                        }
                    }
                }
                div { class: "oxho-wires", style: "grid-column:1 / -1;grid-row:2",
                    svg { view_box: "0 0 {vw} 60", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        for (k, d, _, _) in lines.iter().cloned() {
                            path { key: "{k}", "data-k": "{k}", class: "oxho-ln", d: "{d}" }
                        }
                        for (k, li, back, delay) in lights.iter().cloned() {
                            if let Some((_, d, _, _)) = lines.get(li).cloned() {
                                path { key: "{k}", "data-k": "{k}", class: "oxho-lt", "data-back": "{back}", d: "{d}", path_length: "1", style: "animation-delay:{delay}ms" }
                            }
                        }
                    }
                    for (k, _, cx, ax) in lines.iter().cloned() {
                        span { key: "{k}", "data-k": "{k}", class: "oxho-dots",
                            i { class: "oxho-dot", style: "left:{ax / vw * 100.0:.3}%;top:0" }
                            i { class: "oxho-dot", style: "left:{cx / vw * 100.0:.3}%;top:100%" }
                        }
                    }
                }
                for j in 0..n.saturating_sub(1) {
                    {
                        let from = sc.agents[j].clone();
                        let to = sc.agents[j + 1].clone();
                        let col = sc.column(&from.id, &to.id);
                        let count = col.len();
                        let ck = format!("col{j}");
                        rsx! {
                            div { key: "{ck}", "data-k": "{ck}", class: "oxho-col", style: "grid-column:{2 * j + 2} / span 2;grid-row:3;--o:{2 * j + 1}",
                                div { class: "oxho-chd", title: "{from.name} → {to.name}",
                                    BrandIcon { name: from.brand.clone(), size: 12.0, coloured: true, fallback: true }
                                    span { "{from.name}" }
                                    Icon { name: "chevron-right", size: 11.0 }
                                    BrandIcon { name: to.brand.clone(), size: 12.0, coloured: true, fallback: true }
                                    span { "{to.name}" }
                                    span { class: "oxho-sp" }
                                    span { class: "oxho-cnt", "{count}" }
                                }
                                ul { class: "oxho-list", "aria-label": "Handed from {from.name} to {to.name}",
                                    for h in col.iter().cloned() {
                                        HandoffCard { key: "{h.id}", h: h.clone(), sc: sc.clone(), el, lap, opened, item_open, picked, restored, on_restore }
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

/// One handoff card: title, what went along and where the task is; open, its items.
#[component]
fn HandoffCard(
    h: Handoff,
    sc: HandoffScene,
    el: u64,
    lap: u64,
    opened: Signal<Option<String>>,
    item_open: Signal<Option<String>>,
    picked: Signal<Picked>,
    restored: Signal<Vec<(u64, String, String)>>,
    on_restore: EventHandler<(String, String)>,
) -> Element {
    let mut opened = opened;
    let mut item_open = item_open;
    let mut picked = picked;
    let is_open = opened().as_deref() == Some(h.id.as_str());
    let (kind, text) = handoff_status(&h, &sc.tasks, &sc.agents, el);
    let logo = sc
        .tasks
        .iter()
        .find(|t| t.title == h.task)
        .map(|t| t.logo.clone())
        .unwrap_or_default();
    let line = items_line(&h.items);
    let fresh = h.at > 0 && el >= h.at && el < h.at + 900;
    let is_picked = picked().as_ref().is_some_and(|p| p.0 == h.id);
    let (id1, id2) = (h.id.clone(), h.id.clone());
    let (t1, t2) = (h.task.clone(), h.task.clone());
    let (to1, to2) = (h.to.clone(), h.to.clone());
    let label = format!("{} {} {}", h.task, line, text);
    rsx! {
        li { class: "oxho-li", "data-fresh": "{fresh}", "data-open": "{is_open}",
            button { class: "oxho-card", r#type: "button", "aria-expanded": "{is_open}", "aria-label": "{label}", draggable: "true",
                "data-picked": "{is_picked}",
                onclick: move |_| { opened.set(if opened().as_deref() == Some(id1.as_str()) { None } else { Some(id1.clone()) }); item_open.set(None) },
                ondoubleclick: move |_| picked.set(Some((id2.clone(), t1.clone(), to1.clone()))),
                ondragstart: move |_| picked.set(Some((String::new(), t2.clone(), to2.clone()))),
                ondragend: move |_| picked.set(None),
                span { class: "oxho-kt",
                    span { class: "oxho-cl", BrandIcon { name: logo.clone(), size: 13.0, coloured: true, fallback: true } }
                    b { "{h.task}" }
                }
                span { class: "oxho-kl", "{line}" }
                span { class: "oxho-ks", "data-kind": "{kind}",
                    match kind {
                        "spin" => spinner(),
                        "warn" => rsx! { svg { width: "11", height: "11", view_box: "0 0 12 12", "aria-hidden": "true", path { d: "M6 1.2 11.2 10.4H.8z", fill: "currentColor" } } },
                        _ => rsx! { i { class: "oxho-ok" } },
                    }
                    "{text}"
                }
            }
            if is_open {
                ul { class: "oxho-items", "aria-label": "What went along",
                    for it in h.items.iter().cloned() {
                        ItemRow { key: "{it.id}", it: it.clone(), hid: h.id.clone(), lap, item_open, restored, on_restore }
                    }
                }
            }
        }
    }
}

#[component]
fn ItemRow(
    it: HandoffItem,
    hid: String,
    lap: u64,
    item_open: Signal<Option<String>>,
    restored: Signal<Vec<(u64, String, String)>>,
    on_restore: EventHandler<(String, String)>,
) -> Element {
    let mut item_open = item_open;
    let mut restored = restored;
    if !it.lost {
        return rsx! {
            li { class: "oxho-it", Icon { name: kind_icon(it.kind), size: 12.0, stroke: 1.8 } span { "{it.label}" } }
        };
    }
    let key = format!("{hid}/{}", it.id);
    let open = item_open().as_deref() == Some(key.as_str())
        || item_open().as_deref() == Some(it.id.as_str());
    let (k1, h1, i1) = (key.clone(), hid.clone(), it.id.clone());
    rsx! {
        li { class: "oxho-it", "data-lost": "true",
            button { class: "oxho-lost", r#type: "button", "aria-expanded": "{open}", title: "{it.why}",
                onclick: move |_| item_open.set(if open { None } else { Some(k1.clone()) }),
                Icon { name: "ban", size: 12.0, stroke: 1.8 } s { "{it.label}" }
            }
            if open {
                div { class: "oxho-why",
                    span { "{it.why}" }
                    button { class: "oxho-add", r#type: "button",
                        onclick: move |_| { restored.write().push((lap, h1.clone(), i1.clone())); on_restore.call((h1.clone(), i1.clone())) },
                        "Add back" }
                }
            }
        }
    }
}
