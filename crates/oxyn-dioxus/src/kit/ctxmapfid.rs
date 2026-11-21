//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32): the exact **Context Map**
//! ([`ContextTreemap`]). Behaviour and look only; the model is `oxyn_core::kit::ctxmapfid`.
//! Styles: `ctxmapfid.css` (classes `oxcm-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::ctxmapfid::{
        CtxAgent, CtxBlock, CtxCompaction, ctx_compaction_sum, ctx_demo, ctx_demo_agents,
        ctx_demo_compacted, ctx_fmt, ctx_layout, ctx_pct, ctx_used,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

const FREE: &str = "__free";
/// The map's width per 1 of height (680 × 282).
const ASPECT: f32 = 680.0 / 282.0;

/// Context Map (affectUI): what fills an agent's context window as a treemap. Tiles keep their
/// order and glide as the context grows; the free space is dashed; pointing at a tile shows its
/// card and a click pins it; the bar switches agents and, after a compaction, Before | After.
#[component]
pub fn ContextTreemap(
    /// Every part of the context, in tile order (the demo's when `None`).
    #[props(default)]
    blocks: Option<Vec<CtxBlock>>,
    /// The window's size in tokens (the current agent's when 0).
    #[props(default)]
    limit: u32,
    /// Agents to switch between (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<CtxAgent>>,
    /// The agent shown first (id; the first when empty).
    #[props(default)]
    agent: String,
    /// The last compaction (adds Before | After and the line under the map).
    #[props(default)]
    compaction: Option<CtxCompaction>,
    /// Demo only: show the demo's compaction.
    #[props(default)]
    compacted: bool,
    /// Which side of the compaction shows first (`before` / `after`).
    #[props(default = "after".to_string())]
    view: String,
    /// Demo only: grow the session (0–1) when `live` is off.
    #[props(default = 0.35)]
    grow: f32,
    /// Demo only: the session grows, compacts and settles on its own.
    #[props(default)]
    live: bool,
    /// A short status in the bar ("Compacting").
    #[props(default)]
    status: String,
    /// A tile pointed at from the start (its card shows).
    #[props(default)]
    point: String,
    /// The agent menu open from the start.
    #[props(default)]
    menu_open: bool,
    /// Heading.
    #[props(default = "Context".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A tile was clicked (block id; empty for the free space).
    #[props(default)]
    on_select: EventHandler<String>,
    /// Another agent was picked.
    #[props(default)]
    on_agent_change: EventHandler<String>,
    /// Before | After was switched.
    #[props(default)]
    on_view_change: EventHandler<String>,
) -> Element {
    let start = use_signal(move || now);
    let ags = agents.clone().unwrap_or_else(ctx_demo_agents);
    let first = if agent.is_empty() {
        ags.first().map(|a| a.id.clone()).unwrap_or_default()
    } else {
        agent.clone()
    };
    let mut cur = use_signal(move || first);
    let mut side = use_signal(move || view.clone());
    let mut hover = use_signal(move || (!point.is_empty()).then(|| point.clone()));
    let mut pin = use_signal(|| None::<String>);
    let mut menu = use_signal(move || menu_open);

    let el = now.since(start()).as_millis() as u64;
    let a = ags.iter().find(|x| x.id == cur()).cloned();
    let lim = if limit > 0 {
        limit
    } else {
        a.as_ref().map_or(200_000, |x| x.limit)
    };
    // Demo timeline when live: grow for 9 s, compact for 1.6 s, then settle compacted.
    let (live_grow, live_comp, live_status) = if live {
        let g = (el as f32 / 9_000.0).min(1.0);
        (
            g,
            el >= 10_600,
            if (9_000..10_600).contains(&el) {
                "Compacting"
            } else {
                ""
            },
        )
    } else {
        (grow, compacted, "")
    };
    let (list, comp): (Vec<CtxBlock>, Option<CtxCompaction>) = match (&blocks, &compaction) {
        (Some(b), c) => (b.clone(), c.clone()),
        (None, _) if live_comp => {
            let (after, c) = ctx_demo_compacted();
            (after, Some(c))
        }
        (None, _) => (ctx_demo(live_grow), None),
    };
    let st = if !status.is_empty() {
        status.clone()
    } else {
        live_status.to_string()
    };
    let before = comp.is_some() && side() == "before";
    let shown: Vec<CtxBlock> = match (&comp, before) {
        (Some(c), true) => c.before.clone(),
        _ => list.clone(),
    };
    let used = ctx_used(&shown);
    let pct = ctx_pct(used, lim);
    let free = lim.saturating_sub(used);
    let mut weights: Vec<f32> = shown.iter().map(|b| b.tokens as f32).collect();
    weights.push(free as f32);
    let rects = ctx_layout(&weights, ASPECT);
    let tag = |b: &CtxBlock| -> &'static str {
        match (&comp, before) {
            (Some(c), true) if c.summarized.contains(&b.id) => "summarized",
            (Some(c), true) if c.dropped.contains(&b.id) => "dropped",
            (Some(_), true) => "kept",
            _ => "",
        }
    };
    let lit = pin().or(hover());
    let card = lit.as_ref().and_then(|id| {
        let i = shown.iter().position(|b| &b.id == id)?;
        let r = rects.iter().find(|r| r.i == i)?;
        Some((shown[i].clone(), *r))
    });
    let sum = comp.as_ref().map(|c| ctx_compaction_sum(c, &list));
    let ags_menu = ags.clone();
    let hot = pct >= 90;

    rsx! {
        section { class: "oxcm", "data-style": style_attr(&style), "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { pin.set(None); hover.set(None); menu.set(false); },
            div { class: "oxcm-bar",
                span { class: "oxcm-ic",
                    svg { width: "14", height: "14", view_box: "0 0 16 16", "aria-hidden": "true",
                        rect { x: "1.5", y: "1.5", width: "6", height: "13", rx: "1.5", fill: "currentColor" }
                        rect { x: "9", y: "1.5", width: "5.5", height: "6", rx: "1.5", fill: "currentColor", opacity: ".55" }
                        rect { x: "9", y: "9", width: "5.5", height: "5.5", rx: "1.5", fill: "currentColor", opacity: ".3" } }
                }
                h3 { class: "oxcm-ttl", "{title}" }
                if let Some(ag) = a.clone() {
                    div { class: "oxcm-agw",
                        button { class: "oxcm-ag", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu()}",
                            onclick: move |_| menu.set(!menu()),
                            BrandIcon { name: ag.brand.clone(), size: 13.0, coloured: true, fallback: true }
                            span { class: "oxcm-agn", "{ag.name}" }
                            span { class: "oxcm-lim", "{ctx_fmt(lim)}" }
                            svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", "aria-hidden": "true", path { d: "M4.5 6.5 8 10l3.5-3.5" } }
                        }
                        if menu() {
                            ul { class: "oxcm-menu", role: "listbox", "aria-label": "Agent",
                                for x in ags_menu.iter().cloned() {
                                    {
                                        let xid = x.id.clone();
                                        let on = x.id == cur();
                                        rsx! {
                                            li { key: "{x.id}", "data-k": "{x.id}", class: "oxcm-opt", role: "option", "aria-selected": "{on}", tabindex: "0",
                                                onclick: move |_| { cur.set(xid.clone()); menu.set(false); on_agent_change.call(xid.clone()) },
                                                BrandIcon { name: x.brand.clone(), size: 14.0, coloured: true, fallback: true }
                                                span { class: "oxcm-ot", b { "{x.name}" } span { "{x.model}" } }
                                                span { class: "oxcm-lim", "{ctx_fmt(x.limit)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !st.is_empty() { span { class: "oxcm-status", i {} "{st}" } }
                span { class: "oxcm-sp" }
                if comp.is_some() {
                    div { class: "oxcm-seg", role: "radiogroup", "aria-label": "Compaction",
                        for v in ["before", "after"] {
                            button { key: "{v}", "data-k": "{v}", r#type: "button", role: "radio", "aria-checked": "{side() == v}",
                                onclick: move |_| { side.set(v.to_string()); on_view_change.call(v.to_string()) },
                                if v == "before" { "Before" } else { "After" } }
                        }
                    }
                }
                span { class: "oxcm-nums",
                    b { key: "{used}", "data-k": "{used}", class: "oxcm-used", "{ctx_fmt(used)}" }
                    span { class: "oxcm-of", " / {ctx_fmt(lim)}" }
                }
                span { class: "oxcm-pct", "data-hot": "{hot}", "{pct}%" }
            }
            div { class: "oxcm-map", role: "list", "aria-label": "{title}",
                onmouseleave: move |_| hover.set(None),
                for r in rects.iter().copied() {
                    {
                        let is_free = r.i >= shown.len();
                        let style = format!("left:{:.3}%;top:{:.3}%;width:{:.3}%;height:{:.3}%", r.x * 100.0, r.y * 100.0, r.w * 100.0, r.h * 100.0);
                        if is_free {
                            rsx! {
                                div { key: "{FREE}", "data-k": "{FREE}", class: "oxcm-tile", "data-kind": "free", role: "listitem", style: "{style}",
                                    "aria-label": "Free, {ctx_fmt(free)} tokens",
                                    onclick: move |_| { pin.set(None); on_select.call(String::new()) },
                                    span { class: "oxcm-in", span { class: "oxcm-l", "Free" } span { class: "oxcm-t", "{ctx_fmt(free)}" } }
                                }
                            }
                        } else {
                            let b = shown[r.i].clone();
                            let t = tag(&b);
                            let (idh, idc, idf) = (b.id.clone(), b.id.clone(), b.id.clone());
                            let on = lit.as_deref() == Some(b.id.as_str());
                            let pinned = pin().as_deref() == Some(b.id.as_str());
                            let line = if t.is_empty() { format!("{} · {}", ctx_fmt(b.tokens), b.kind.word()) } else { format!("{} · {}", ctx_fmt(b.tokens), t) };
                            rsx! {
                                div { key: "{b.id}", "data-k": "{b.id}", class: "oxcm-tile", "data-kind": "{b.kind.id()}", "data-tag": "{t}", "data-on": "{on}", "data-pin": "{pinned}",
                                    role: "listitem", tabindex: "0", style: "{style}",
                                    "aria-label": "{b.label}, {b.kind.word()}, {ctx_fmt(b.tokens)} tokens",
                                    onmouseenter: move |_| hover.set(Some(idh.clone())),
                                    onfocus: move |_| hover.set(Some(idf.clone())),
                                    onclick: move |_| {
                                        if pin().as_deref() == Some(idc.as_str()) { pin.set(None) } else { pin.set(Some(idc.clone())) }
                                        on_select.call(idc.clone());
                                    },
                                    onkeydown: {
                                        let idk = b.id.clone();
                                        move |e: KeyboardEvent| if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                                            e.prevent_default();
                                            if pin().as_deref() == Some(idk.as_str()) { pin.set(None) } else { pin.set(Some(idk.clone())) }
                                        }
                                    },
                                    span { class: "oxcm-in",
                                        span { class: "oxcm-l",
                                            if !b.brand.is_empty() { BrandIcon { name: b.brand.clone(), size: 12.0, coloured: true } }
                                            "{b.label}"
                                        }
                                        span { class: "oxcm-t", "{line}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((b, r)) = card {
                    {
                        let (below, above) = (1.0 - (r.y + r.h), r.y);
                        let cx = (r.x + r.w / 2.0) * 100.0;
                        let horiz = format!("left:clamp(4px,calc({cx:.2}% - 125px),calc(100% - 254px))");
                        let pos = if below >= 0.56 {
                            format!("{horiz};top:calc({:.2}% + 6px)", (r.y + r.h) * 100.0)
                        } else if above >= 0.56 {
                            format!("{horiz};bottom:calc({:.2}% + 6px)", (1.0 - r.y) * 100.0)
                        } else if r.x + r.w / 2.0 < 0.5 {
                            format!("top:8px;left:min(calc({:.2}% + 6px),calc(100% - 254px))", (r.x + r.w) * 100.0)
                        } else {
                            format!("top:8px;right:min(calc({:.2}% + 6px),calc(100% - 254px))", (1.0 - r.x) * 100.0)
                        };
                        rsx! {
                            div { class: "oxcm-card", "data-kind": "{b.kind.id()}", role: "tooltip",
                                style: "{pos}",
                                div { class: "oxcm-ch",
                                    if !b.brand.is_empty() { BrandIcon { name: b.brand.clone(), size: 14.0, coloured: true } } else { i { class: "oxcm-sw", "data-kind": "{b.kind.id()}" } }
                                    b { "{b.label}" }
                                }
                                span { class: "oxcm-cs", "{b.kind.word()} · {ctx_fmt(b.tokens)} tokens · {(b.tokens as f32 / lim.max(1) as f32 * 100.0):.1}%" }
                                if !b.items.is_empty() {
                                    ul { class: "oxcm-items",
                                        for (k, (l, br)) in b.items.iter().cloned().enumerate() {
                                            li { key: "{k}", "data-k": "{k}",
                                                if br.is_empty() { i { class: "oxcm-dot" } } else { BrandIcon { name: br.clone(), size: 13.0, coloured: true } }
                                                "{l}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxcm-foot",
                if let Some(cs) = sum {
                    {
                        let (b, af, kept, summ, s, drop) = (cs.before, cs.after, cs.kept, cs.summarized, cs.summary, cs.dropped);
                        rsx! {
                    b { key: "{side()}", "data-k": "{side()}", class: "oxcm-fb", "{ctx_fmt(b)} → {ctx_fmt(af)}" }
                    span { class: "oxcm-fl", i { class: "oxcm-sw", "data-kind": "system" } "{kept.0} kept · {ctx_fmt(kept.1)}" }
                    span { class: "oxcm-fl", i { class: "oxcm-sw", "data-kind": "summary" } "{summ.0} summarized · {ctx_fmt(summ.1)} into {ctx_fmt(s)}" }
                    span { class: "oxcm-fl", i { class: "oxcm-sw", "data-kind": "dropped" } "{drop.0} dropped · {ctx_fmt(drop.1)}" }
                        }
                    }
                } else {
                    "No compaction so far – the before and after will show here when it runs."
                }
            }
        }
    }
}
