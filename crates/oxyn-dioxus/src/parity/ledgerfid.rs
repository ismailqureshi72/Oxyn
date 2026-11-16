//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32): the exact **Agent Ledger**
//! ([`AgentLedger`]). Behaviour and look only; the model is `oxyn_core::parity::ledgerfid`.
//! Styles: `ledgerfid.css` (classes `oxlg-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::ledgerfid::{
        LEDGER_LOOP_MS, LedgerEntry, ledger_ago, ledger_at, ledger_sorted, ledger_undo_plan,
        undo_label,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

const UNDO: &str = "M4 8h10a6 6 0 0 1 0 12h-3M8 4 4 8l4 4";
const REDO: &str = "M20 8H10a6 6 0 0 0 0 12h3M16 4l4 4-4 4";

fn glyph(d: &'static str) -> Element {
    rsx! {
        svg { class: "oxlg-gl", width: "13", height: "13", view_box: "0 0 24 24", fill: "none", stroke: "currentColor",
            stroke_width: "1.9", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "{d}" }
        }
    }
}

fn lock() -> Element {
    rsx! {
        svg { class: "oxlg-gl", width: "11", height: "11", view_box: "0 0 24 24", fill: "none", stroke: "currentColor",
            stroke_width: "2", stroke_linecap: "round", "aria-hidden": "true",
            rect { x: "5", y: "10.5", width: "14", height: "10", rx: "2.5" }
            path { d: "M8 10.5V8a4 4 0 0 1 8 0v2.5" }
        }
    }
}

/// Agent Ledger (affectUI): every change an agent made, newest first, before → after per row and
/// an Undo on each; pointing at Undo lights what builds on it ("Also undone" in blue, "Can't
/// undo" in amber); undone rows are struck through with a Redo.
#[component]
pub fn AgentLedger(
    /// Your own changes (the demo's when `None`).
    #[props(default)]
    entries: Option<Vec<LedgerEntry>>,
    /// Ids already undone (your own entries).
    #[props(default)]
    undone: Vec<String>,
    /// Heading of the bar.
    #[props(default = "Ledger".to_string())]
    title: String,
    /// Who made the changes, next to the title.
    #[props(default = "Release agent".to_string())]
    agent: String,
    /// Freeze the demo this many ms into a pass.
    #[props(default)]
    at: Option<u64>,
    /// Light up a change and what builds on it, as if its Undo were pointed at.
    #[props(default)]
    point_at: Option<String>,
    /// The clock.
    now: ClockMs,
    /// The ledger's own clock for "2m ago" with your own entries (ms); the demo keeps its own.
    #[props(default)]
    clock: u64,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Undo: the change and its reversible dependents.
    #[props(default)]
    on_undo: EventHandler<Vec<String>>,
    /// Redo: the change.
    #[props(default)]
    on_redo: EventHandler<Vec<String>>,
) -> Element {
    let demo = entries.is_none();
    let start = use_signal(move || now);
    // The person's own undo / redo on top of the scene: (lap, ids, undone?).
    let mut acts = use_signal(Vec::<(u64, Vec<String>, bool)>::new);
    let mut hover = use_signal(|| None::<String>);

    let raw = now.since(start()).as_millis() as u64;
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / LEDGER_LOOP_MS, raw % LEDGER_LOOP_MS),
        _ => (0, raw),
    };
    let sc = ledger_at(el);
    let (list, mut gone, auto_point, clk) = match entries.clone() {
        Some(e) => (e, undone.clone(), None, clock),
        None => (
            sc.entries,
            sc.undone,
            if at.is_some() { None } else { sc.point },
            sc.now,
        ),
    };
    for (l, ids, u) in acts().iter() {
        if *l != lap {
            continue;
        }
        for id in ids {
            gone.retain(|x| x != id);
            if *u {
                gone.push(id.clone());
            }
        }
    }
    let point = hover().or(point_at.clone()).or(auto_point);
    let (lit, kept) = match point.as_deref() {
        Some(p) if !gone.iter().any(|g| g == p) => ledger_undo_plan(&list, p, &gone),
        _ => (vec![], vec![]),
    };
    let rows: Vec<LedgerEntry> = ledger_sorted(&list).into_iter().cloned().collect();
    let n = rows.len();
    let n_undone = rows.iter().filter(|e| gone.contains(&e.id)).count();
    let lap_k = format!("lap{lap}");

    rsx! {
        div { class: "oxlg", "data-style": style_attr(&style),
            div { class: "oxlg-bar",
                span { class: "oxlg-ic",
                    svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round", "aria-hidden": "true",
                        rect { x: "4", y: "3", width: "16", height: "18", rx: "3" }
                        path { d: "M8 8h8M8 12h8M8 16h5" }
                    }
                }
                b { class: "oxlg-ttl", "{title}" }
                if !agent.is_empty() { span { class: "oxlg-agent", "{agent}" } }
                span { class: "oxlg-sp" }
                span { class: "oxlg-cnt", b { "{n}" } if n == 1 { " change" } else { " changes" } }
                if n_undone > 0 { span { class: "oxlg-cnt", b { "{n_undone}" } " undone" } }
            }
            ol { class: "oxlg-list", "aria-label": "{title}", key: "{lap_k}", "data-lap": "{lap_k}",
                for e in rows.iter().cloned() {
                    {
                        let is_gone = gone.contains(&e.id);
                        let flag = if lit.first() == Some(&e.id) { "self" } else if lit.contains(&e.id) { "also" } else if kept.contains(&e.id) { "kept" } else { "" };
                        let also = ledger_undo_plan(&list, &e.id, &gone).0.len().saturating_sub(1);
                        let ago = ledger_ago(clk, e.at);
                        let (ir, ih, il) = (e.id.clone(), e.id.clone(), e.id.clone());
                        let plan = ledger_undo_plan(&list, &e.id, &gone).0;
                        let ulabel = undo_label(&e, also);
                        let rlabel = format!("Redo: {} {}", e.action, e.target);
                        rsx! {
                            li { key: "{e.id}", "data-k": "{e.id}", class: "oxlg-row", "data-flag": "{flag}", "data-gone": "{is_gone}",
                                span { class: "oxlg-tile", BrandIcon { name: e.mark(), size: 17.0, coloured: true, fallback: true } }
                                span { class: "oxlg-txt",
                                    span { class: "oxlg-l1", title: "{e.action} {e.target}", b { "{e.action}" } if !e.target.is_empty() { span { "{e.target}" } } }
                                    if e.has_change() {
                                        span { class: "oxlg-l2", s { "{e.change.0}" } span { class: "oxlg-to", "aria-label": "to", "→" } span { class: "oxlg-after", "{e.change.1}" } }
                                    } else {
                                        span { class: "oxlg-l2", "{e.detail}" }
                                    }
                                }
                                span { class: "oxlg-slot",
                                    {match flag {
                                        "also" => rsx! { span { class: "oxlg-pill", "data-k": "also", "Also undone" } },
                                        "kept" => rsx! { span { class: "oxlg-pill", "data-k": "kept", {lock()} "Can't undo" } },
                                        _ => rsx! {
                                            span { class: "oxlg-time", if is_gone { "Undone" } else { "{ago} · " span { class: "oxlg-sys", "{e.system}" } } }
                                            if is_gone {
                                                button { class: "oxlg-btn", r#type: "button", "aria-label": "{rlabel}", "data-k": "redo",
                                                    onclick: move |_| { acts.write().push((lap, vec![ir.clone()], false)); on_redo.call(vec![ir.clone()]) },
                                                    {glyph(REDO)} span { class: "oxlg-bl", "Redo" } }
                                            } else if e.reversible {
                                                button { class: "oxlg-btn", r#type: "button", "aria-label": "{ulabel}", "data-on": "{flag == \"self\"}",
                                                    onmouseenter: move |_| hover.set(Some(ih.clone())),
                                                    onmouseleave: move |_| hover.set(None),
                                                    onfocus: move |_| hover.set(Some(il.clone())),
                                                    onblur: move |_| hover.set(None),
                                                    onclick: move |_| { hover.set(None); acts.write().push((lap, plan.clone(), true)); on_undo.call(plan.clone()) },
                                                    {glyph(UNDO)} span { class: "oxlg-bl", "Undo" } }
                                            }
                                        },
                                    }}
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxlg-sr", role: "status", "aria-live": "polite",
                if n_undone > 0 { "{n_undone} undone" }
            }
        }
    }
}
