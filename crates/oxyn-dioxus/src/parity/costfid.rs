//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32): the exact **Cost Waterfall**
//! ([`CostWaterfall`]). Behaviour and look only; the model is `oxyn_core::parity::costfid`.
//! Styles: `costfid.css` (classes `oxcf-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::costfid::{
        CostKind, CostMode, CostStatus, CostStep, cost_axis, cost_demo, cost_money, cost_path,
        cost_rows, cost_sum, cost_time, cost_tok, cost_tokens, cost_top, cost_total,
    },
};

use crate::brand::BrandIcon;

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

fn sparkle() -> Element {
    rsx! { svg { width: "13", height: "13", view_box: "0 0 16 16", "aria-hidden": "true", path { fill: "currentColor", d: "M8 1.5c.4 3.4 1.9 5 5.5 5.5-3.6.5-5.1 2.1-5.5 5.5-.4-3.4-1.9-5-5.5-5.5 3.6-.5 5.1-2.1 5.5-5.5z" } } }
}

fn fmt_axis(v: f64, mode: CostMode) -> String {
    match mode {
        CostMode::Cost => format!("${v:.2}"),
        CostMode::Tokens => {
            if v <= 0.0 {
                "0".into()
            } else {
                cost_tok(v.round() as u32)
            }
        }
    }
}

/// Mark live steps: the first `shown` leaves are in; the last one shown runs.
fn reveal(steps: &[CostStep], shown: &mut usize, running: &mut bool) -> Vec<CostStep> {
    let mut out = vec![];
    for s in steps {
        if *shown == 0 {
            break;
        }
        let mut c = s.clone();
        if s.children.is_empty() {
            *shown -= 1;
            if *shown == 0 && *running {
                c.status = CostStatus::Running;
                c.tokens_out /= 2;
                c.cost = c.cost.map(|v| v * 0.5);
                *running = false;
            }
        } else {
            c.children = reveal(&s.children, shown, running);
            if c.children.iter().any(|k| k.status == CostStatus::Running) {
                c.status = CostStatus::Running;
            }
        }
        out.push(c);
    }
    out
}

fn leaves(steps: &[CostStep]) -> usize {
    steps
        .iter()
        .map(|s| {
            if s.children.is_empty() {
                1
            } else {
                leaves(&s.children)
            }
        })
        .sum()
}

/// Cost Waterfall (affectUI): every step of an agent run as a waterfall row – the bars step
/// across to the total – with model, tokens in and out, cost and time; the stats on top name
/// the most expensive step; Cost | Tokens re-bases every bar; sub-agents fold; a row opens its
/// prompt and response.
#[component]
pub fn CostWaterfall(
    /// The run's steps (the demo's when `None`).
    #[props(default)]
    steps: Option<Vec<CostStep>>,
    /// What the run is about, muted next to the title.
    #[props(default = "Release notes for v2.4".to_string())]
    task: String,
    /// Starts in the tokens view.
    #[props(default)]
    tokens: bool,
    /// Sub-agents folded from the start (ids).
    #[props(default)]
    collapsed: Vec<String>,
    /// A step whose details are open from the start.
    #[props(default)]
    open_id: String,
    /// Demo: the run plays out step by step (`step_ms` apart).
    #[props(default)]
    live: bool,
    /// Gap between steps while live, ms.
    #[props(default = 900)]
    step_ms: u64,
    /// Heading.
    #[props(default = "Run cost".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// The switch was used (`cost` / `tokens`).
    #[props(default)]
    on_mode_change: EventHandler<String>,
    /// Details opened (id) or closed (empty).
    #[props(default)]
    on_open_change: EventHandler<String>,
    /// The folded sub-agents changed.
    #[props(default)]
    on_collapsed_change: EventHandler<Vec<String>>,
) -> Element {
    let start = use_signal(move || now);
    let mut mode = use_signal(move || {
        if tokens {
            CostMode::Tokens
        } else {
            CostMode::Cost
        }
    });
    let mut fold = use_signal(move || collapsed.clone());
    let mut open = use_signal(move || (!open_id.is_empty()).then(|| open_id.clone()));
    let mut cur = use_signal(|| 0usize);
    let mut refs = use_signal(Vec::<Option<Rc<MountedData>>>::new);

    let el = now.since(start()).as_millis() as u64;
    let all = steps.clone().unwrap_or_else(cost_demo);
    let total_leaves = leaves(&all);
    let list = if live {
        let mut n = ((el / step_ms.max(1)) as usize + 1).min(total_leaves);
        let mut running = n < total_leaves || el < step_ms * total_leaves as u64;
        reveal(&all, &mut n, &mut running)
    } else {
        all.clone()
    };
    let m = mode();
    let rows = cost_rows(&list, m, &fold());
    let total: f64 = list.iter().map(|s| cost_total(s, m)).sum();
    let total_cost: f64 = list.iter().map(|s| cost_total(s, CostMode::Cost)).sum();
    let (tin, tout) = list.iter().fold((0u32, 0u32), |(a, b), s| {
        let (x, y) = cost_tokens(s);
        (a + x, b + y)
    });
    let running = rows.iter().any(|r| r.step.status == CostStatus::Running);
    let (axis_max, tick) = cost_axis(total);
    let ticks: Vec<f64> = (0..)
        .map(|i| i as f64 * tick)
        .take_while(|t| *t <= axis_max + 1e-9)
        .collect();
    let top = cost_top(&list);
    let top_id = top.as_ref().map(|t| t.id.clone()).unwrap_or_default();
    let n = rows.len();
    let all_for_top = list.clone();

    rsx! {
        section { class: "oxcf", "data-style": style_attr(&style), "data-mode": if m == CostMode::Cost { "cost" } else { "tokens" }, "aria-label": "{title}",
            div { class: "oxcf-bar",
                span { class: "oxcf-ic", {sparkle()} }
                h3 { class: "oxcf-ttl", "{title}" }
                span { class: "oxcf-task", "{task}" }
                span { class: "oxcf-sp" }
                div { class: "oxcf-sw", role: "group", "aria-label": "Measure by",
                    for (k, md) in [("cost", CostMode::Cost), ("tokens", CostMode::Tokens)] {
                        button { key: "{k}", "data-k": "{k}", r#type: "button", "aria-pressed": "{m == md}",
                            onclick: move |_| { mode.set(md); on_mode_change.call(k.to_string()) },
                            if md == CostMode::Cost { "Cost" } else { "Tokens" } }
                    }
                }
                span { class: "oxcf-st", "data-st": if running { "live" } else { "done" }, role: "status",
                    if running { i {} "Live" } else { "Done" } }
            }
            dl { class: "oxcf-stats",
                div { dt { "Total cost" } dd { b { class: "oxcf-big", "data-run": "{running}", "{cost_money(Some(total_cost))}" } } }
                div { dt { "Total tokens" } dd { b { class: "oxcf-big", "data-run": "{running}", "{cost_tok(tin + tout)}" } span { class: "oxcf-mut", "{cost_tok(tin)} in · {cost_tok(tout)} out" } } }
                div { dt { "Most expensive" }
                    dd {
                        if let Some(t) = top.clone() {
                            button { class: "oxcf-top", r#type: "button",
                                onclick: move |_| {
                                    let path = cost_path(&all_for_top, &t.id);
                                    fold.write().retain(|f| !path.contains(f));
                                    open.set(Some(t.id.clone()));
                                    on_open_change.call(t.id.clone());
                                    let rs = cost_rows(&all_for_top, mode(), &fold());
                                    if let Some(i) = rs.iter().position(|r| r.step.id == t.id) { cur.set(i); focus(refs.read().get(i).cloned().flatten()); }
                                },
                                b { "{t.name}" } span { class: "oxcf-mut", "{cost_money(t.cost)}" } }
                        } else { span { class: "oxcf-mut", "–" } }
                    }
                }
            }
            div { class: "oxcf-grid",
                div { class: "oxcf-head", "aria-hidden": "true",
                    span { class: "oxcf-c0", "Step" }
                    span { class: "oxcf-band",
                        for t in ticks.iter().copied() {
                            span { key: "{t}", "data-k": "{t}", class: "oxcf-tick", style: "left:{t / axis_max * 100.0:.2}%", "{fmt_axis(t, m)}" }
                        }
                    }
                    span { class: "oxcf-n oxcf-io", "In" } span { class: "oxcf-n oxcf-io", "Out" } span { class: "oxcf-n", "Cost" } span { class: "oxcf-n", "Time" }
                }
                ul { class: "oxcf-rows", role: "tree", "aria-label": "Steps",
                    onkeydown: move |e: KeyboardEvent| {
                        let c = cur();
                        let next = match e.key() {
                            Key::ArrowDown => Some((c + 1).min(n.saturating_sub(1))),
                            Key::ArrowUp => Some(c.saturating_sub(1)),
                            Key::Home => Some(0),
                            Key::End => Some(n.saturating_sub(1)),
                            Key::Escape => { open.set(None); on_open_change.call(String::new()); None }
                            _ => None,
                        };
                        if let Some(i) = next { e.prevent_default(); cur.set(i); focus(refs.read().get(i).cloned().flatten()); }
                    },
                    for (i, r) in rows.iter().cloned().enumerate() {
                        {
                            let s = r.step.clone();
                            let is_open = open().as_deref() == Some(s.id.as_str());
                            let has_kids = !s.children.is_empty();
                            let folded = fold().contains(&s.id);
                            let (tin, tout) = if has_kids { cost_tokens(&s) } else { (s.tokens_in, s.tokens_out) };
                            let cost = if has_kids { cost_sum(&s) } else { s.cost };
                            let x = r.start / axis_max * 100.0;
                            let w = r.len / axis_max * 100.0;
                            let (id1, id2, id3) = (s.id.clone(), s.id.clone(), s.id.clone());
                            let label = format!("{}, {}, {}, {}, {} tokens, {}", s.name, s.kind.id(), match s.status { CostStatus::Running => "running", CostStatus::Done => "done", CostStatus::Failed => "failed" }, cost_money(cost), cost_tok(tin + tout), cost_time(s.duration));
                            let (li, lo) = s.kind.io();
                            let inp = if s.input.is_empty() { "–".to_string() } else { s.input.clone() };
                            let outp = if !s.error.is_empty() { s.error.clone() } else if s.output.is_empty() { "–".to_string() } else { s.output.clone() };
                            rsx! {
                                li { key: "{s.id}", "data-k": "{s.id}", class: "oxcf-li", role: "treeitem", "aria-level": "{r.depth + 1}",
                                    "aria-expanded": if has_kids { Some(if folded { "false" } else { "true" }) } else { None },
                                    "aria-selected": "{is_open}", "aria-label": "{label}",
                                    tabindex: if cur() == i { "0" } else { "-1" },
                                    onmounted: move |e| { let mut v = refs.write(); if v.len() <= i { v.resize(i + 1, None) } v[i] = Some(e.data()) },
                                    onfocus: move |_| cur.set(i),
                                    onkeydown: move |e: KeyboardEvent| match e.key() {
                                        Key::Enter => { e.prevent_default(); e.stop_propagation(); let nx = if open().as_deref() == Some(id1.as_str()) { None } else { Some(id1.clone()) }; open.set(nx.clone()); on_open_change.call(nx.unwrap_or_default()) }
                                        Key::Character(ref c) if c == " " => { e.prevent_default(); e.stop_propagation(); let nx = if open().as_deref() == Some(id1.as_str()) { None } else { Some(id1.clone()) }; open.set(nx.clone()); on_open_change.call(nx.unwrap_or_default()) }
                                        Key::ArrowRight if has_kids && folded => { e.prevent_default(); e.stop_propagation(); fold.write().retain(|f| f != &id1); on_collapsed_change.call(fold()) }
                                        Key::ArrowLeft if has_kids && !folded => { e.prevent_default(); e.stop_propagation(); fold.write().push(id1.clone()); on_collapsed_change.call(fold()) }
                                        _ => {}
                                    },
                                    div { class: "oxcf-row", "data-open": "{is_open}", "data-st": match s.status { CostStatus::Running => "running", CostStatus::Done => "done", CostStatus::Failed => "failed" }, "data-kind": "{s.kind.id()}",
                                        onclick: move |_| { cur.set(i); let nx = if open().as_deref() == Some(id2.as_str()) { None } else { Some(id2.clone()) }; open.set(nx.clone()); on_open_change.call(nx.unwrap_or_default()) },
                                        span { class: "oxcf-c0", style: "padding-left:{6 + r.depth * 14}px",
                                            if has_kids {
                                                button { class: "oxcf-chev", r#type: "button", tabindex: "-1", "aria-label": if folded { "Open sub-agent" } else { "Fold sub-agent" }, "data-open": "{!folded}",
                                                    onclick: move |e| { e.stop_propagation(); if fold().contains(&id3) { fold.write().retain(|f| f != &id3) } else { fold.write().push(id3.clone()) } on_collapsed_change.call(fold()) },
                                                    svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "M6 4l4 4-4 4" } } }
                                            } else { span { class: "oxcf-chev-sp" } }
                                            span { class: "oxcf-tile", "data-kind": "{s.kind.id()}",
                                                if s.kind == CostKind::Agent { {sparkle()} }
                                                else if !s.brand.is_empty() { BrandIcon { name: s.brand.clone(), size: 12.0, coloured: true } }
                                                else { svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.4", "aria-hidden": "true", circle { cx: "8", cy: "8", r: "5.5" } path { d: "M2.5 8h11M8 2.5c2 2 2 9 0 11M8 2.5c-2 2-2 9 0 11" } } }
                                            }
                                            span { class: "oxcf-nm", "data-agent": "{has_kids}", title: "{s.name} · {s.detail}", "{s.name}" }
                                            span { class: "oxcf-md", "{s.detail}" }
                                            if folded && r.hidden > 0 { span { class: "oxcf-plus", "+{r.hidden}" } }
                                            if s.id == top_id { span { class: "oxcf-topb", "Top" } }
                                            if s.status == CostStatus::Failed { span { class: "oxcf-warn", title: "{s.error}", svg { width: "13", height: "13", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "M10.3 4.1 2.6 17.5A2 2 0 0 0 4.3 20.5h15.4a2 2 0 0 0 1.7-3L13.7 4.1a2 2 0 0 0-3.4 0z" } path { d: "M12 9.5v4M12 17h.01" } } } }
                                        }
                                        span { class: "oxcf-band",
                                            for t in ticks.iter().copied().skip(1) { i { key: "{t}", "data-k": "{t}", class: "oxcf-grid-l", style: "left:{t / axis_max * 100.0:.2}%" } }
                                            i { class: "oxcf-bar-b", "data-kind": "{s.kind.id()}", style: "left:{x:.3}%;width:max(2px,{w:.3}%)" }
                                        }
                                        span { class: "oxcf-n oxcf-io", "{cost_tok(tin)}" }
                                        span { class: "oxcf-n oxcf-io", "{cost_tok(tout)}" }
                                        span { class: "oxcf-n oxcf-cost", "{cost_money(cost)}" }
                                        span { class: "oxcf-n oxcf-time", "data-failed": "{s.status == CostStatus::Failed}", "{cost_time(s.duration)}" }
                                    }
                                    if is_open {
                                        div { class: "oxcf-det",
                                            div { class: "oxcf-box", span { class: "oxcf-bl", "{li}" } p { "{inp}" } }
                                            div { class: "oxcf-box", "data-err": "{!s.error.is_empty()}",
                                                span { class: "oxcf-bl", if s.error.is_empty() { "{lo}" } else { "Error" } }
                                                p { "{outp}" } }
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
