//! Context Visualizer family (variants 2): four views of what fills a model's context window –
//! [`ContextBar`] (one stacked bar or a field of dots, with a kind legend and Compact / Undo),
//! [`ContextBreakdown`] (a list of kinds that open into what each holds), [`ContextStack`] (an
//! Oxyn original: the window as a pile of slabs with the auto-compact line) and [`ContextTide`]
//! (an Oxyn original: a glass whose water rises a turn at a time). Models:
//! `oxyn_core::parity::ctxvisfid`. Styles: `ctxvisfid.css` (`oxcv-`, `oxcb-`, `oxcxs-`, `oxcxt-`).
//! Chrome uses the neutral `--oxth-*` greys; hue only marks the kinds and the fill level.

use dioxus::prelude::*;
use oxyn_core::parity::ctxvisfid::{
    CtxPart, ctx_add_turn, ctx_compact, ctx_demo, ctx_drop_item, ctx_load_order, ctx_pct, ctx_said,
    ctx_shares, ctx_used, field_cells, fill_level, fmt_tokens, kind_hex, kind_mono, slab_heights,
    turns_left,
};

use super::efflistfid::chevron;

/// Inline style giving a kind its colour in light and dark (`--kl` / `--kd`; `.oxcv-k` picks).
pub(crate) fn kind_style(kind: &str, mono: bool) -> String {
    if mono {
        format!(
            "--kl:#{};--kd:#{}",
            kind_mono(kind, false),
            kind_mono(kind, true)
        )
    } else {
        let h = kind_hex(kind);
        format!("--kl:#{h};--kd:#{h}")
    }
}

/// Context Visualizer – Context bar: "Context", the used figure over the limit, the window as
/// one stacked bar (or a field of dots) and a legend of kinds. Pointing at a kind lights it and
/// puts its own count in the figure; Compact squeezes the chat into a summary, Undo brings it back.
#[component]
pub fn ContextBar(
    /// The parts (the demo window when `None`).
    #[props(default)]
    parts: Option<Vec<CtxPart>>,
    /// Window size in tokens.
    #[props(default = 200_000)]
    limit: u32,
    /// `bar` (one stacked bar) or `field` (a field of dots).
    #[props(default = "bar".to_string())]
    look: String,
    /// Greys instead of kind colours.
    #[props(default)]
    mono: bool,
    /// Show the Compact / Undo button.
    #[props(default = true)]
    compact: bool,
    /// Starts compacted.
    #[props(default)]
    default_compacted: bool,
    /// Heading.
    #[props(default = "Context".to_string())]
    title: String,
    /// Compacted (true) or undone (false).
    #[props(default)]
    on_compact: EventHandler<bool>,
) -> Element {
    let base = parts.unwrap_or_else(ctx_demo);
    let mut done = use_signal(move || default_compacted);
    let mut hov = use_signal(|| None::<usize>);
    let mut said = use_signal(String::new);
    let shown = if done() {
        ctx_compact(&base, 180)
    } else {
        base.clone()
    };
    let used = ctx_used(&shown);
    let free = limit.saturating_sub(used);
    let shares = ctx_shares(&shown, limit);
    let (num, of) = match hov().and_then(|i| shown.get(i)) {
        Some(p) => (fmt_tokens(p.tokens), p.label.clone()),
        None => (fmt_tokens(used), format!("of {}", fmt_tokens(limit))),
    };
    let field = look == "field";
    let cells = if field {
        field_cells(&shown, limit, 200)
    } else {
        Vec::new()
    };
    let level = fill_level(used, limit, 92);
    let label = ctx_said(used, limit);
    let all = base.clone();

    rsx! {
        div { class: "oxcv", "data-look": "{look}", "data-level": "{level}", "data-hov": "{hov().is_some()}",
            div { class: "oxcv-head",
                span { class: "oxcv-title", "{title}" }
                if compact {
                    button { class: "oxcv-btn", r#type: "button", "data-undo": "{done()}", "aria-pressed": "{done()}",
                        onclick: move |_| {
                            let now = !done();
                            done.set(now);
                            let after = if now { ctx_compact(&all, 180) } else { all.clone() };
                            said.set(format!("{} {}", if now { "Compacted." } else { "Undone." }, ctx_said(ctx_used(&after), limit)));
                            on_compact.call(now);
                        },
                        if done() { "Undo" } else { "Compact" }
                    }
                }
            }
            div { class: "oxcv-fig",
                for k in std::iter::once(num.clone()) { span { key: "{k}", class: "oxcv-num", "{k}" } }
                span { class: "oxcv-of", "{of}" }
            }
            if field {
                div { class: "oxcv-field", role: "img", "aria-label": "{label}",
                    for (c, k) in cells.iter().enumerate() {
                        {
                            let st = match k { Some(i) => kind_style(shown[*i].kind, mono), None => String::new() };
                            let dim = matches!((hov(), k), (Some(h), Some(i)) if h != *i) || (hov().is_some() && k.is_none());
                            let on = k.is_some();
                            rsx! { i { key: "{c}", class: "oxcv-dot oxcv-k", "data-on": "{on}", "data-dim": "{dim}", style: "{st}" } }
                        }
                    }
                }
            } else {
                div { class: "oxcv-bar", role: "img", "aria-label": "{label}",
                    for (i, p) in shown.iter().enumerate() {
                        {
                            let st = format!("{};flex-grow:{}", kind_style(p.kind, mono), shares[i].max(0.4));
                            let dim = hov().is_some_and(|h| h != i);
                            rsx! {
                                span { key: "{p.kind}", class: "oxcv-seg oxcv-k", "data-dim": "{dim}", style: "{st}",
                                    onpointerenter: move |_| hov.set(Some(i)),
                                    onpointerleave: move |_| hov.set(None),
                                }
                            }
                        }
                    }
                    {
                        let st = format!("flex-grow:{}", free as f32 * 100.0 / limit.max(1) as f32);
                        rsx! { span { class: "oxcv-free", "data-dim": "{hov().is_some()}", style: "{st}" } }
                    }
                }
            }
            div { class: "oxcv-legend",
                for (i, p) in shown.iter().enumerate() {
                    {
                        let st = kind_style(p.kind, mono);
                        let t = fmt_tokens(p.tokens);
                        rsx! {
                            button { key: "{p.kind}", class: "oxcv-chip", r#type: "button", "data-on": "{hov() == Some(i)}",
                                "aria-label": "{p.label}: {t} tokens",
                                onpointerenter: move |_| hov.set(Some(i)),
                                onpointerleave: move |_| hov.set(None),
                                onfocus: move |_| hov.set(Some(i)),
                                onblur: move |_| hov.set(None),
                                i { class: "oxcv-sw oxcv-k", style: "{st}" }
                                "{p.label}"
                            }
                        }
                    }
                }
            }
            div { class: "oxcv-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Context Visualizer – Context breakdown: the used figure and a fill pill over a list of kinds,
/// each with its share; a kind opens into what it holds, and files or tools can be unloaded.
#[component]
pub fn ContextBreakdown(
    /// The parts (the demo window when `None`).
    #[props(default)]
    parts: Option<Vec<CtxPart>>,
    /// Window size in tokens.
    #[props(default = 200_000)]
    limit: u32,
    /// Auto-compact threshold in percent.
    #[props(default = 92)]
    auto: u32,
    /// Starts with this kind open.
    #[props(default)]
    default_open: Option<usize>,
    /// `size` (largest first) or `load` (load order).
    #[props(default = "size".to_string())]
    default_sort: String,
    /// Files and tools get an Unload button.
    #[props(default = true)]
    droppable: bool,
    /// Greys instead of kind colours.
    #[props(default)]
    mono: bool,
    /// An item was unloaded: (part index, item index) in the parts as given.
    #[props(default)]
    on_drop: EventHandler<(usize, usize)>,
) -> Element {
    let start = parts.unwrap_or_else(ctx_demo);
    let mut list = use_signal(move || start);
    let mut open = use_signal(move || default_open);
    let mut sort = use_signal(move || default_sort.clone());
    let mut said = use_signal(String::new);
    let parts = list();
    let used = ctx_used(&parts);
    let pct = ctx_pct(used, limit);
    let level = fill_level(used, limit, auto);
    let mut order: Vec<usize> = (0..parts.len()).collect();
    if sort() == "size" {
        order.sort_by(|a, b| parts[*b].tokens.cmp(&parts[*a].tokens));
    } else {
        let lo = ctx_load_order(&parts);
        order.sort_by_key(|i| lo.iter().position(|p| p.kind == parts[*i].kind));
    }
    let top = parts.iter().map(|p| p.tokens).max().unwrap_or(1).max(1);
    let shares = ctx_shares(&parts, limit);
    let foot = match turns_left(used, limit, auto, 6_000) {
        Some(0) | None => format!("The next turn passes {auto} % – the chat will be compacted."),
        Some(1) => format!("About 1 turn before it compacts at {auto} %."),
        Some(n) => format!("About {n} turns before it compacts at {auto} %."),
    };
    let u = fmt_tokens(used);
    let l = fmt_tokens(limit);

    rsx! {
        div { class: "oxcb", "data-level": "{level}",
            div { class: "oxcb-head",
                div { class: "oxcb-hl",
                    span { class: "oxcb-title", "Context" }
                    span { class: "oxcb-sub", "{u} of {l} tokens" }
                }
                span { class: "oxcb-pct", "data-level": "{level}", "{pct}%" }
            }
            div { class: "oxcb-track", "aria-hidden": "true",
                for (i, p) in parts.iter().enumerate() {
                    {
                        let st = format!("{};width:{}%", kind_style(p.kind, mono), shares[i]);
                        rsx! { i { key: "{p.kind}", class: "oxcv-k", "data-on": "{open() == Some(i)}", "data-dim": "{open().is_some_and(|o| o != i)}", style: "{st}" } }
                    }
                }
            }
            div { class: "oxcb-sort", role: "radiogroup", "aria-label": "Order",
                for (k, name) in [("size", "By size"), ("load", "Load order")] {
                    button { key: "{k}", r#type: "button", role: "radio", class: "oxcb-sb", "aria-checked": "{sort() == k}",
                        onclick: move |_| sort.set(k.to_string()), "{name}" }
                }
            }
            ul { class: "oxcb-list",
                for i in order.iter().copied() {
                    {
                        let p = parts[i].clone();
                        let st = kind_style(p.kind, mono);
                        let w = p.tokens as f32 * 100.0 / top as f32;
                        let is = open() == Some(i);
                        let t = fmt_tokens(p.tokens);
                        let sh = format!("{:.0}%", shares[i]);
                        let can = droppable && (p.kind == "files" || p.kind == "tools");
                        rsx! {
                            li { key: "{p.kind}", class: "oxcb-li", "data-open": "{is}",
                                button { class: "oxcb-row", r#type: "button", "aria-expanded": "{is}",
                                    onclick: move |_| open.set(if is { None } else { Some(i) }),
                                    i { class: "oxcb-sw oxcv-k", style: "{st}" }
                                    span { class: "oxcb-name", "{p.label}" }
                                    span { class: "oxcb-mini", i { class: "oxcv-k", style: "{st};width:{w}%" } }
                                    span { class: "oxcb-tok", "{t}" }
                                    span { class: "oxcb-sh", "{sh}" }
                                    span { class: "oxcb-chev", {chevron(is)} }
                                }
                                if is {
                                    ul { class: "oxcb-items",
                                        for (j, (name, tok)) in p.items.iter().enumerate() {
                                            {
                                                let tt = fmt_tokens(*tok);
                                                let nm = name.clone();
                                                rsx! {
                                                    li { key: "{name}", class: "oxcb-item",
                                                        span { class: "oxcb-in", "{name}" }
                                                        span { class: "oxcb-it", "{tt}" }
                                                        if can {
                                                            button { class: "oxcb-drop", r#type: "button", "aria-label": "Unload {name}",
                                                                onclick: move |_| {
                                                                    ctx_drop_item(&mut list.write(), i, j);
                                                                    said.set(format!("Unloaded {nm}, {tt} tokens freed."));
                                                                    on_drop.call((i, j));
                                                                },
                                                                "Unload"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if p.items.is_empty() { li { class: "oxcb-item oxcb-empty", "Nothing loaded." } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxcb-foot", "{foot}" }
            div { class: "oxcv-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Context Visualizer – Context stack (an Oxyn original): the window as a pile of slabs, system
/// at the bottom and the chat on top, under a dashed line where it compacts by itself. Add a turn
/// to watch the chat slab grow; at the line, or on Compact, it is pressed into a thin summary.
#[component]
pub fn ContextStack(
    /// The parts (the demo window when `None`).
    #[props(default)]
    parts: Option<Vec<CtxPart>>,
    /// Window size in tokens.
    #[props(default = 200_000)]
    limit: u32,
    /// Auto-compact threshold in percent.
    #[props(default = 92)]
    auto: u32,
    /// Tokens one turn adds.
    #[props(default = 6_000)]
    per_turn: u32,
    /// Column height in px.
    #[props(default = 232.0)]
    height: f32,
    /// Greys instead of kind colours.
    #[props(default)]
    mono: bool,
    /// Compacted (automatically when `true`).
    #[props(default)]
    on_compact: EventHandler<bool>,
) -> Element {
    let start = ctx_load_order(&parts.unwrap_or_else(ctx_demo));
    let first = start.clone();
    let mut list = use_signal(move || start);
    let mut hov = use_signal(|| None::<usize>);
    let mut note = use_signal(String::new);
    let ps = list();
    let used = ctx_used(&ps);
    let pct = ctx_pct(used, limit);
    let hs = slab_heights(&ps, limit, height, 12.0);
    let squeezed = ps.iter().any(|p| p.kind == "summary");
    let level = fill_level(used, limit, auto);
    let u = fmt_tokens(used);
    let l = fmt_tokens(limit);
    let label = ctx_said(used, limit);
    let line = format!("bottom:{auto}%");

    rsx! {
        div { class: "oxcxs", "data-level": "{level}",
            div { class: "oxcxs-head",
                span { class: "oxcxs-title", "Context stack" }
                span { class: "oxcxs-fig", b { "{u}" } " / {l} · {pct}%" }
            }
            div { class: "oxcxs-body",
                div { class: "oxcxs-col", role: "img", "aria-label": "{label}", style: "height:{height}px",
                    div { class: "oxcxs-line", style: "{line}", span { "Compacts at {auto}%" } }
                    for (i, p) in ps.iter().enumerate() {
                        {
                            let st = format!("{};height:{}px", kind_style(p.kind, mono), hs[i]);
                            rsx! {
                                div { key: "{p.kind}", class: "oxcxs-slab oxcv-k", "data-kind": "{p.kind}", "data-hov": "{hov() == Some(i)}", style: "{st}",
                                    onpointerenter: move |_| hov.set(Some(i)),
                                    onpointerleave: move |_| hov.set(None),
                                }
                            }
                        }
                    }
                }
                ul { class: "oxcxs-legend",
                    for (i, p) in ps.iter().enumerate().rev() {
                        {
                            let st = kind_style(p.kind, mono);
                            let t = fmt_tokens(p.tokens);
                            rsx! {
                                li { key: "{p.kind}", class: "oxcxs-lg", "data-hov": "{hov() == Some(i)}",
                                    onpointerenter: move |_| hov.set(Some(i)),
                                    onpointerleave: move |_| hov.set(None),
                                    i { class: "oxcxs-sw oxcv-k", style: "{st}" }
                                    span { class: "oxcxs-ln", "{p.label}" }
                                    span { class: "oxcxs-lt", "{t}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxcxs-acts",
                button { class: "oxcxs-b", r#type: "button",
                    onclick: move |_| {
                        let mut v = list();
                        ctx_add_turn(&mut v, per_turn);
                        if ctx_pct(ctx_used(&v), limit) >= auto && !v.iter().any(|p| p.kind == "summary") {
                            v = ctx_compact(&v, 180);
                            note.set(format!("Reached {auto}% – the chat was compacted on its own. {}", ctx_said(ctx_used(&v), limit)));
                            on_compact.call(true);
                        } else {
                            note.set(format!("Turn added. {}", ctx_said(ctx_used(&v), limit)));
                        }
                        list.set(v);
                    },
                    "Add a turn"
                }
                button { class: "oxcxs-b", r#type: "button", disabled: squeezed,
                    onclick: move |_| {
                        let v = ctx_compact(&list(), 180);
                        note.set(format!("Compacted. {}", ctx_said(ctx_used(&v), limit)));
                        list.set(v);
                        on_compact.call(false);
                    },
                    "Compact"
                }
                button { class: "oxcxs-b oxcxs-ghost", r#type: "button",
                    onclick: move |_| { list.set(first.clone()); note.set("Reset.".into()); },
                    "Reset"
                }
            }
            p { class: "oxcxs-note", role: "status", "aria-live": "polite", "{note}" }
        }
    }
}

/// Context Visualizer – Context tide (an Oxyn original): a tall glass whose water is the used
/// share, with marks at 75 % and at the auto-compact line. Each turn raises it; Compact drains it.
/// The water stays grey until it is high (amber) and then at the line (red) unless `tint` is off.
#[component]
pub fn ContextTide(
    /// The parts (the demo window when `None`).
    #[props(default)]
    parts: Option<Vec<CtxPart>>,
    /// Window size in tokens.
    #[props(default = 200_000)]
    limit: u32,
    /// Auto-compact threshold in percent.
    #[props(default = 92)]
    auto: u32,
    /// Tokens one turn adds.
    #[props(default = 6_000)]
    per_turn: u32,
    /// Amber and red water as it fills.
    #[props(default = true)]
    tint: bool,
    /// The used tokens changed.
    #[props(default)]
    on_change: EventHandler<u32>,
) -> Element {
    let start = parts.unwrap_or_else(ctx_demo);
    let first = start.clone();
    let mut list = use_signal(move || start);
    let mut note = use_signal(String::new);
    let used = ctx_used(&list());
    let pct = ctx_pct(used, limit);
    let level = if tint {
        fill_level(used, limit, auto)
    } else {
        "calm"
    };
    let turns = match turns_left(used, limit, auto, per_turn) {
        None | Some(0) => "Compacts on the next turn".to_string(),
        Some(1) => "≈ 1 turn before it compacts".to_string(),
        Some(n) => format!("≈ {n} turns before it compacts"),
    };
    let u = fmt_tokens(used);
    let l = fmt_tokens(limit);
    let water = format!("height:{pct}%");
    let mark = format!("bottom:{auto}%");
    let squeezed = list().iter().any(|p| p.kind == "summary");

    rsx! {
        div { class: "oxcxt", "data-level": "{level}",
            div { class: "oxcxt-glass", role: "meter", "aria-label": "Context used", "aria-valuemin": "0", "aria-valuemax": "100",
                "aria-valuenow": "{pct}", "aria-valuetext": "{u} of {l} tokens",
                div { class: "oxcxt-water", style: "{water}",
                    svg { class: "oxcxt-wave", view_box: "0 0 240 12", preserve_aspect_ratio: "none", "aria-hidden": "true",
                        path { d: "M0 6 Q15 0 30 6 T60 6 T90 6 T120 6 T150 6 T180 6 T210 6 T240 6 V12 H0Z" }
                    }
                }
                div { class: "oxcxt-mark", style: "bottom:75%", span { "75" } }
                div { class: "oxcxt-mark oxcxt-auto", style: "{mark}", span { "{auto}" } }
            }
            div { class: "oxcxt-side",
                span { class: "oxcxt-k", "Context" }
                for k in std::iter::once(pct) { span { key: "{k}", class: "oxcxt-pct", "{k}%" } }
                span { class: "oxcxt-of", "{u} of {l} tokens" }
                span { class: "oxcxt-turns", "{turns}" }
                div { class: "oxcxt-acts",
                    button { class: "oxcxs-b", r#type: "button",
                        onclick: move |_| {
                            let mut v = list();
                            ctx_add_turn(&mut v, per_turn);
                            if ctx_pct(ctx_used(&v), limit) >= auto && !v.iter().any(|p| p.kind == "summary") {
                                v = ctx_compact(&v, 180);
                                note.set(format!("High tide at {auto}% – compacted. {}", ctx_said(ctx_used(&v), limit)));
                            } else {
                                note.set(ctx_said(ctx_used(&v), limit));
                            }
                            on_change.call(ctx_used(&v));
                            list.set(v);
                        },
                        "Send a turn"
                    }
                    button { class: "oxcxs-b", r#type: "button", disabled: squeezed,
                        onclick: move |_| {
                            let v = ctx_compact(&list(), 180);
                            note.set(format!("Drained. {}", ctx_said(ctx_used(&v), limit)));
                            on_change.call(ctx_used(&v));
                            list.set(v);
                        },
                        "Compact"
                    }
                    button { class: "oxcxs-b oxcxs-ghost", r#type: "button",
                        onclick: move |_| { list.set(first.clone()); note.set("Reset.".into()); },
                        "Reset"
                    }
                }
            }
            p { class: "oxcv-sr", role: "status", "aria-live": "polite", "{note}" }
        }
    }
}
