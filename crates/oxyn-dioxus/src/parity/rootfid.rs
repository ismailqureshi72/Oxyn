//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Root Cause Tree**
//! ([`RootCauseTree`]). Behaviour and look only; the model is `oxyn_core::parity::rootfid`.
//! Styles: `rootfid.css` (classes `oxrt-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::rootfid::{
        RcEvidence, RcGate, RcNode, RcStatus, rc_counts, rc_demo, rc_flat, rc_fmt, rc_found,
        rc_len, rc_path, rc_roots, rc_status,
    },
};

use crate::brand::BrandIcon;

/// Demo: run-seconds per real ms.
const MS_PER_S: u64 = 40;
/// Rest at the end before the demo loops, run-seconds.
const REST_S: u32 = 160;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// What a node's status line says.
fn status_line(n: &RcNode, st: RcStatus, root: bool) -> String {
    let at = n.check.map(|c| rc_fmt(c.1)).unwrap_or_default();
    match st {
        RcStatus::Open => "Not checked".into(),
        RcStatus::Checking => "Checking".into(),
        RcStatus::RuledOut => format!("Ruled out · {at}"),
        RcStatus::Confirmed if root => format!("Root cause · {at}"),
        RcStatus::Confirmed => format!("Confirmed · {at}"),
    }
}

/// Everything the tree needs to draw one node.
#[derive(Clone, PartialEq)]
struct Ctx {
    t: u32,
    sel: Option<String>,
    path: Vec<String>,
    roots: Vec<String>,
    found: bool,
    fail_line: String,
}

fn files(f: &[String]) -> Element {
    rsx! {
        if let Some(first) = f.first() {
            span { class: "oxrt-files",
                span { class: "oxrt-file", "{first}" }
                if f.len() > 1 { span { class: "oxrt-more", "+{f.len() - 1}" } }
            }
        }
    }
}

fn card(
    n: &RcNode,
    depth: usize,
    cx: &Ctx,
    mut sel: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    let st = rc_status(n, cx.t);
    let st_id = if depth == 0 {
        if cx.found { "found" } else { "failed" }
    } else {
        st.id()
    };
    let root = cx.roots.contains(&n.id);
    let on_path = cx.path.contains(&n.id);
    let dim = cx.found && !on_path && depth > 0;
    let id = n.id.clone();
    let line = if depth == 0 {
        cx.fail_line.clone()
    } else {
        status_line(n, st, root)
    };
    let selected = cx.sel.as_deref() == Some(n.id.as_str());
    rsx! {
        button { class: "oxrt-card", r#type: "button", role: "treeitem", "aria-level": "{depth + 1}", "aria-selected": selected,
            "data-st": st_id, "data-path": on_path || (depth == 0 && cx.found), "data-dim": dim, "data-root": root, "data-jid": "{n.id}",
            "aria-label": "{n.title}: {line}",
            onclick: move |_| { sel.set(Some(id.clone())); on_select.call(id.clone()); },
            span { class: "oxrt-ch",
                span { class: "oxrt-ic", BrandIcon { name: n.brand.clone(), size: 12.0, coloured: st != RcStatus::RuledOut, fallback: true } }
                b { "{n.title}" }
            }
            {files(&n.files)}
            span { class: "oxrt-st", "data-st": st_id, "data-root": root,
                match st {
                    RcStatus::Checking => rsx! { i { class: "oxrt-spin" } },
                    RcStatus::RuledOut => icon("M4.5 4.5l7 7M11.5 4.5l-7 7", 10),
                    RcStatus::Confirmed => rsx! { i { class: "oxrt-dot" } },
                    RcStatus::Open if depth == 0 => rsx! { i { class: "oxrt-dot" } },
                    RcStatus::Open => rsx! { i { class: "oxrt-hollow" } },
                }
                "{line}"
            }
        }
    }
}

fn branch(
    n: &RcNode,
    depth: usize,
    cx: &Ctx,
    sel: Signal<Option<String>>,
    on_select: EventHandler<String>,
) -> Element {
    let gate = match n.gate {
        RcGate::Or => "or",
        RcGate::And => "and",
    };
    let lit = cx.path.contains(&n.id) || (depth == 0 && cx.found);
    rsx! {
        div { class: "oxrt-sub", "data-path": cx.path.contains(&n.id), role: "none",
            {card(n, depth, cx, sel, on_select)}
            if !n.children.is_empty() {
                span { class: "oxrt-gate", "data-path": lit, "aria-hidden": "true", "{gate}" }
                div { class: "oxrt-kids", role: "group",
                    for c in n.children.iter() {
                        div { class: "oxrt-kid", key: "{c.id}", "data-k": "{c.id}", "data-path": cx.path.contains(&c.id),
                            {branch(c, depth + 1, cx, sel, on_select)}
                        }
                    }
                }
            }
        }
    }
}

/// Root Cause Tree (affectUI parity): a failed run drawn as a fault tree. Hypotheses are checked
/// in turn and marked ruled out or confirmed; once a cause is confirmed its route to the failure
/// is highlighted and the rest fades. Select any node to read its verdict and evidence.
#[component]
pub fn RootCauseTree(
    /// The failure with its hypotheses (the demo's when `None`).
    #[props(default)]
    failure: Option<RcNode>,
    /// Run seconds; `None` plays the demo (looping).
    #[props(default)]
    at: Option<u32>,
    /// The node whose evidence shows from the start (the newest checked one when `None`).
    #[props(default)]
    selected: Option<String>,
    /// Heading.
    #[props(default = "Root cause".to_string())]
    title: String,
    /// The clock (drives the demo).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A node was picked (id).
    #[props(default)]
    on_select: EventHandler<String>,
    /// "Retry from here" on a root cause (its id).
    #[props(default)]
    on_retry: EventHandler<String>,
    /// "Fix it" on a root cause (its id).
    #[props(default)]
    on_fix: EventHandler<String>,
) -> Element {
    let t0 = use_signal(move || now);
    let sel = use_signal(move || selected.clone());
    let f = failure.clone().unwrap_or_else(rc_demo);
    let len = rc_len(&f);
    let t = at.unwrap_or_else(|| {
        let el = (now.since(t0()).as_millis() as u64 / MS_PER_S) as u32;
        (el % (len + REST_S)).min(len)
    });
    let found = rc_found(&f, t);
    let (n_h, n_out, _) = rc_counts(&f, t);
    let flat = rc_flat(&f);
    // The evidence card: the picked node, else the newest finished check, else the failure.
    let newest = flat
        .iter()
        .filter(|(n, _)| {
            matches!(
                rc_status(n, t),
                RcStatus::RuledOut | RcStatus::Confirmed | RcStatus::Checking
            )
        })
        .max_by_key(|(n, _)| n.check.map(|c| c.0))
        .map(|(n, _)| n.id.clone());
    let shown_id = sel().or(newest).unwrap_or_else(|| f.id.clone());
    let shown: RcNode = if shown_id == f.id {
        f.clone()
    } else {
        flat.iter()
            .find(|(n, _)| n.id == shown_id)
            .map(|(n, _)| (*n).clone())
            .unwrap_or_else(|| f.clone())
    };
    let fail_line = if found {
        "Cause found".to_string()
    } else {
        format!(
            "Failed · {}",
            f.verdict.split(' ').take(3).collect::<Vec<_>>().join(" ")
        )
    };
    let cx = Ctx {
        t,
        sel: Some(shown_id.clone()),
        path: rc_path(&f, t),
        roots: rc_roots(&f, t),
        found,
        fail_line: fail_line.clone(),
    };
    let shown_st = if shown.id == f.id {
        RcStatus::Confirmed
    } else {
        rc_status(&shown, t)
    };
    let shown_root = cx.roots.contains(&shown.id);
    let order: Vec<String> = std::iter::once(f.id.clone())
        .chain(flat.iter().map(|(n, _)| n.id.clone()))
        .collect();

    rsx! {
        section { class: "oxrt", "data-style": style_attr(&style), "data-found": found, "aria-label": "{title}",
            header { class: "oxrt-bar",
                span { class: "oxrt-logo", {icon("M8 2.5v4M8 6.5 4 10v3.5M8 6.5l4 3.5v3.5M2.5 13.5h3M10.5 13.5h3", 15)} }
                h2 { class: "oxrt-ttl", "{title}" }
                if !found { span { class: "oxrt-pill", "Investigating" } }
                span { class: "oxrt-sp" }
                span { class: "oxrt-count", role: "status", "{n_h} hypotheses · {n_out} ruled out" }
            }
            div { class: "oxrt-tree", role: "tree", "aria-label": "{title}",
                onkeydown: {
                    let order = order.clone();
                    let mut sel = sel;
                    move |e: KeyboardEvent| {
                        let cur = order.iter().position(|x| Some(x) == sel().as_ref()).unwrap_or(0);
                        let next = match e.key() {
                            Key::ArrowRight | Key::ArrowDown => Some((cur + 1).min(order.len() - 1)),
                            Key::ArrowLeft | Key::ArrowUp => Some(cur.saturating_sub(1)),
                            Key::Home => Some(0),
                            Key::End => Some(order.len() - 1),
                            _ => None,
                        };
                        if let Some(i) = next { e.prevent_default(); sel.set(Some(order[i].clone())); on_select.call(order[i].clone()); }
                    }
                },
                div { class: "oxrt-top",
                    {branch(&f, 0, &cx, sel, on_select)}
                }
            }
            div { class: "oxrt-ev", key: "{shown.id}", "data-k": "{shown.id}",
                div { class: "oxrt-eh",
                    span { class: "oxrt-ic", BrandIcon { name: shown.brand.clone(), size: 13.0, coloured: true, fallback: true } }
                    b { "{shown.title}" }
                    span { class: "oxrt-sp" }
                    span { class: "oxrt-st", "data-st": if shown.id == f.id { if found { "found" } else { "failed" } } else { shown_st.id() }, "data-root": shown_root,
                        if shown.id == f.id { "{fail_line}" } else { "{status_line(&shown, shown_st, shown_root)}" }
                    }
                }
                if shown.verdict.is_empty() || shown_st == RcStatus::Open || shown_st == RcStatus::Checking {
                    p { class: "oxrt-verdict oxrt-pending", if shown_st == RcStatus::Checking { "Checking this now…" } else { "Not checked – the cause was found elsewhere or it is still queued." } }
                } else {
                    p { class: "oxrt-verdict", "{shown.verdict}" }
                }
                if !shown.files.is_empty() {
                    div { class: "oxrt-fl", for x in shown.files.iter() { span { class: "oxrt-file", key: "{x}", "data-k": "{x}", "{x}" } } }
                }
                if matches!(shown_st, RcStatus::RuledOut | RcStatus::Confirmed) {
                    match shown.evidence.clone() {
                        Some(RcEvidence::Tool(name, ls)) => rsx! {
                            div { class: "oxrt-evb",
                                span { class: "oxrt-evl", "Tool call" b { "{name}" } }
                                pre { class: "oxrt-code", for (k, l) in ls.iter().enumerate() { span { key: "{k}", "data-k": "{k}", "{l}\n" } } }
                            }
                        },
                        Some(RcEvidence::Diff(file, ls)) => rsx! {
                            div { class: "oxrt-evb",
                                span { class: "oxrt-evl", "Diff" b { "{file}" } }
                                pre { class: "oxrt-code",
                                    for (k, l) in ls.iter().enumerate() {
                                        span { key: "{k}", "data-k": "{k}", "data-d": if l.starts_with('-') { "del" } else if l.starts_with('+') { "add" } else { "ctx" }, "{l}\n" }
                                    }
                                }
                            }
                        },
                        None => rsx! {},
                    }
                }
                if shown_root {
                    {
                        let (a, b) = (shown.id.clone(), shown.id.clone());
                        rsx! {
                            div { class: "oxrt-acts",
                                button { class: "oxrt-btn", r#type: "button", onclick: move |_| on_retry.call(a.clone()), "Retry from here" }
                                button { class: "oxrt-btn oxrt-pri", r#type: "button", onclick: move |_| on_fix.call(b.clone()), "Fix it" }
                            }
                        }
                    }
                }
            }
        }
    }
}
