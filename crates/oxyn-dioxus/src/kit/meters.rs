//! Meters (plan #28, chunk 4b): `Gauge` (Arc Gauge), `UsageMeter` (Arc Usage meter), `SplitBar`
//! (Arc Split bar), `ContextField` (affectUI Context Window) and `ContextWindowGrid` (Arc Context
//! window). Behaviour-only rebuilds; models in `oxyn_core::kit::meters`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        fields::Choice,
        meters::{
            Band, Cat, CellField, ContextGrid, Fill, GridItem, Part, Share, Tone, UsageState,
            amount, arc_dash, gauge, next_visible, pct1, split, tokens, usage,
        },
    },
};

use super::fields::UnfoldSelect;
use crate::composer::icons::Icon;

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// Characters keyed by place from the end, so only changed places remount and roll.
#[component]
fn Roll(text: String) -> Element {
    let n = text.chars().count();
    rsx! {
        span { class: "oxm-roll",
            for (i, c) in text.chars().enumerate() {
                span { key: "{n - i}-{c}", class: if c.is_ascii_digit() { "oxm-d" } else { "" }, "{c}" }
            }
        }
    }
}

/// Gauge (Arc behaviour): a value against a known range as a 270° ring. The ring fills once on
/// mount without overshoot, then springs to new values while the percent rolls; threshold bands
/// set the tone and caption exactly as the value crosses them.
#[component]
pub fn Gauge(
    /// Value.
    value: f64,
    /// Lower bound.
    #[props(default)]
    min: f64,
    /// Upper bound.
    #[props(default = 100.0)]
    max: f64,
    /// Label under the number.
    #[props(default)]
    label: String,
    /// Second caption line.
    #[props(default)]
    detail: Option<String>,
    /// Tone when no band applies.
    #[props(default = "accent".to_string())]
    tone: String,
    /// Threshold bands.
    #[props(default)]
    bands: Vec<Band>,
    /// Diameter in px.
    #[props(default = 140.0)]
    size: f32,
    /// Show the raw value instead of the percent of the range.
    #[props(default)]
    raw: bool,
    /// Unit after a raw value.
    #[props(default)]
    unit: String,
) -> Element {
    let g = gauge(value, min, max, Tone::from_id(&tone), &bands);
    let r = 42.0;
    let (on, gap, arc) = arc_dash(g.frac, r, 270.0);
    let num = if raw {
        format!("{}{unit}", value.round())
    } else {
        format!("{}%", g.pct)
    };
    let label_txt = g.band.clone().unwrap_or(label.clone());
    rsx! {
        div { class: "oxm-gauge", "data-tone": "{g.tone.id()}", style: "width:{size}px",
            role: "meter", "aria-valuemin": "{min}", "aria-valuemax": "{max}", "aria-valuenow": "{value}",
            "aria-valuetext": "{num} {label_txt}", "aria-label": "{label}",
            svg { view_box: "0 0 100 100", width: "{size}", height: "{size}", "aria-hidden": "true",
                circle { class: "oxm-gtrack", cx: "50", cy: "50", r: "{r}", fill: "none", stroke_width: "8",
                    stroke_dasharray: "{arc} 999", transform: "rotate(135 50 50)", stroke_linecap: "round" }
                circle { class: "oxm-gfill", cx: "50", cy: "50", r: "{r}", fill: "none", stroke_width: "8",
                    stroke_dasharray: "{on} {gap}", transform: "rotate(135 50 50)", stroke_linecap: "round",
                    style: "--on:{on}" }
            }
            div { class: "oxm-gtext",
                span { class: "oxm-gnum", Roll { text: num.clone() } }
                span { key: "{label_txt}", class: "oxm-glab", "{label_txt}" }
                if let Some(d) = &detail { span { class: "oxm-gdet", "{d}" } }
            }
        }
    }
}

/// Usage meter (Arc behaviour): what fills an allowance. Segments grow in left to right, then
/// re-flow when values change; past the limit a hatched overage and a limit marker fade in. The
/// badge springs between "62% used", "Almost full" and "Over limit". The legend is a roving tab
/// stop: arrows move, Enter or Space pins a category (it stays lit), Escape clears.
#[component]
pub fn UsageMeter(
    /// Heading.
    title: String,
    /// Categories.
    shares: Vec<Share>,
    /// Allowance.
    limit: f64,
    /// Unit after numbers ("GB").
    #[props(default)]
    unit: String,
    /// Decimals.
    #[props(default = 1)]
    decimals: usize,
    /// Legend label for remaining space.
    #[props(default = "Free".to_string())]
    free_label: String,
    /// Legend label for the overage.
    #[props(default = "Over limit".to_string())]
    over_label: String,
    /// Share of the limit at which the badge warns.
    #[props(default = 0.9)]
    warn_at: f64,
) -> Element {
    let u = usage(&shares, limit, warn_at);
    let mut hover = use_signal(|| None::<usize>);
    let mut pinned = use_signal(|| None::<usize>);
    let mut cur = use_signal(|| 0usize);
    let mut refs = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let lit = hover().or(pinned());
    let n = shares.len();
    let badge = match u.state {
        UsageState::Ok => format!("{:.0}% used", u.used / limit.max(f64::EPSILON) * 100.0),
        UsageState::Almost => "Almost full".to_string(),
        UsageState::Over => format!("{over_label} by {}", amount(u.over, decimals, &unit)),
    };
    let state = match u.state {
        UsageState::Ok => "ok",
        UsageState::Almost => "almost",
        UsageState::Over => "over",
    };
    let free_w = (1.0 - u.segs.last().map_or(0.0, |s| s.0 + s.1)).max(0.0);
    rsx! {
        div { class: "oxm-usage", "data-state": "{state}",
            div { class: "oxm-uhead",
                span { class: "oxm-utitle", "{title}" }
                span { class: "oxm-uamt", Roll { text: amount(u.used, decimals, &unit) } span { class: "oxm-uof", " of {amount(limit, decimals, &unit)}" } }
                span { key: "{state}", class: "oxm-badge", role: "status", "{badge}" }
            }
            div { class: "oxm-ubar", role: "img", "aria-label": "{title}: {amount(u.used, decimals, &unit)} of {amount(limit, decimals, &unit)}, {badge}",
                for (i, (x, w)) in u.segs.iter().enumerate() {
                    span { key: "{shares[i].id}", class: "oxm-useg", "data-s": "{i % 5}",
                        "data-dim": "{lit.is_some() && lit != Some(i)}",
                        style: "left:{x * 100.0}%;width:{w * 100.0}%;animation-delay:{i * 70}ms" }
                }
                if u.over > 0.0 {
                    span { class: "oxm-uover", style: "left:{u.limit_at * 100.0}%;width:{(1.0 - u.limit_at) * 100.0}%" }
                    span { class: "oxm-ulimit", style: "left:{u.limit_at * 100.0}%" }
                }
            }
            div { class: "oxm-legend", role: "toolbar", "aria-label": "{title} categories",
                for (i, s) in shares.iter().enumerate() {
                    button { key: "{s.id}", class: "oxm-li", r#type: "button", tabindex: if cur() == i { "0" } else { "-1" },
                        "aria-pressed": "{pinned() == Some(i)}", "data-dim": "{lit.is_some() && lit != Some(i)}",
                        onmounted: move |e| { let mut r = refs.write(); if r.len() <= i { r.resize(i + 1, None) } r[i] = Some(e.data()); },
                        onpointerenter: move |_| hover.set(Some(i)), onpointerleave: move |_| hover.set(None),
                        onfocus: move |_| cur.set(i),
                        onclick: move |_| pinned.set(if pinned() == Some(i) { None } else { Some(i) }),
                        onkeydown: move |e: KeyboardEvent| {
                            let next = match e.key() {
                                Key::ArrowRight | Key::ArrowDown => Some((i + 1) % n),
                                Key::ArrowLeft | Key::ArrowUp => Some((i + n - 1) % n),
                                Key::Home => Some(0),
                                Key::End => Some(n - 1),
                                Key::Escape => { pinned.set(None); None }
                                _ => None,
                            };
                            if let Some(j) = next { e.prevent_default(); cur.set(j); focus(refs.peek().get(j).cloned().flatten()); }
                        },
                        span { class: "oxm-sw", "data-s": "{i % 5}" }
                        span { "{s.label}" }
                        span { class: "oxm-lv", "{amount(s.value, decimals, &unit)}" }
                    }
                }
                if u.over > 0.0 {
                    span { class: "oxm-li static", span { class: "oxm-sw over" } span { "{over_label}" } span { class: "oxm-lv", "{amount(u.over, decimals, &unit)}" } }
                } else {
                    span { class: "oxm-li static", "data-w": "{free_w}", span { class: "oxm-sw free" } span { "{free_label}" } span { class: "oxm-lv", "{amount(u.free, decimals, &unit)}" } }
                }
            }
        }
    }
}

/// Split bar (Arc behaviour): one 100 % bar cut into shares. Resting on (or arrowing onto) a
/// segment widens it by about 4.5 % of the bar, or enough to fit its label, while the others give
/// way and dim; the readout shows its exact figures. The legend switches segments off and on and
/// the rest reflow. Labels sit inside segments that fit them and below on leader lines otherwise.
#[component]
pub fn SplitBar(
    /// Heading (and accessible name).
    title: String,
    /// One line under the title.
    #[props(default)]
    description: Option<String>,
    /// Shares in display order.
    shares: Vec<Share>,
    /// Word after values in the readout ("sessions").
    #[props(default)]
    unit: String,
    /// Prefix on values ("$").
    #[props(default)]
    prefix: String,
    /// Readout label with nothing highlighted.
    #[props(default = "Total".to_string())]
    total_label: String,
    /// Segments switched off at first.
    #[props(default)]
    default_hidden: Vec<String>,
    /// Show the legend.
    #[props(default = true)]
    legend: bool,
    /// Called with the hidden ids when a segment is switched.
    #[props(default)]
    on_hidden_change: EventHandler<Vec<String>>,
) -> Element {
    let mut hidden = use_signal(move || default_hidden.clone());
    let mut focus_i = use_signal(|| None::<usize>);
    let layout = split(&shares, &hidden(), focus_i(), 0.045, 0.16);
    // Labels for segments too narrow to hold them sit below on leader lines; a label that would
    // run into the one before it drops a row (estimated at 6.2 px a character on a 420 px bar).
    let mut outs: Vec<(usize, f32, f32, f32, u32)> = Vec::new();
    let mut ends: Vec<f32> = Vec::new();
    for (i, (x, w, sh)) in layout.iter().copied().enumerate() {
        if w > 0.0 && w < 0.13 {
            let c = x + w / 2.0;
            let half = (shares[i].label.chars().count() + 7) as f32 * 6.2 / 420.0 / 2.0;
            let row = ends
                .iter()
                .position(|e| c - half > *e + 0.01)
                .unwrap_or(ends.len());
            if row == ends.len() {
                ends.push(c + half);
            } else {
                ends[row] = c + half;
            }
            outs.push((i, x, w, sh, row as u32));
        }
    }
    let out_rows = ends.len().saturating_sub(1) as u32;
    let total: f64 = shares
        .iter()
        .filter(|s| !hidden().contains(&s.id))
        .map(|s| s.value)
        .sum();
    let fmt = |v: f64| {
        format!(
            "{prefix}{}{}",
            oxyn_core::kit::fields::format_number(v, 0, true),
            if unit.is_empty() {
                String::new()
            } else {
                format!(" {unit}")
            }
        )
    };
    let readout = match focus_i() {
        Some(i) => format!(
            "{} · {} · {}",
            shares[i].label,
            fmt(shares[i].value),
            pct1(layout[i].2)
        ),
        None => format!("{total_label} · {}", fmt(total)),
    };
    let shares2 = use_signal(|| shares.clone());
    let mut toggle = move |i: usize| {
        let all = shares2.peek().clone();
        let id = all[i].id.clone();
        let mut h = hidden.write();
        if let Some(p) = h.iter().position(|x| *x == id) {
            h.remove(p);
        } else if all.len() - h.len() > 1 {
            h.push(id);
        }
        on_hidden_change.call(h.clone());
    };
    let shares3 = shares.clone();
    rsx! {
        div { class: "oxm-split",
            div { class: "oxm-shead",
                span { class: "oxm-utitle", "{title}" }
                if let Some(d) = &description { span { class: "oxm-sdesc", "{d}" } }
            }
            div { class: "oxm-sread", "aria-live": "polite", span { key: "{readout}", "{readout}" } }
            div { class: "oxm-sbar", tabindex: "0", role: "group", "aria-label": "{title}",
                onpointerleave: move |_| focus_i.set(None),
                onblur: move |_| focus_i.set(None),
                onkeydown: move |e: KeyboardEvent| {
                    let h = hidden();
                    let cur = focus_i();
                    let n = match e.key() {
                        Key::ArrowRight | Key::ArrowDown => next_visible(&shares3, &h, cur, 1),
                        Key::ArrowLeft | Key::ArrowUp => next_visible(&shares3, &h, cur, -1),
                        Key::Home => next_visible(&shares3, &h, None, 1),
                        Key::End => next_visible(&shares3, &h, None, -1),
                        Key::Enter | Key::Character(_) if cur.is_some() && (e.key() == Key::Enter || e.key() == Key::Character(" ".into())) => {
                            e.prevent_default();
                            if let Some(c) = cur { toggle(c); }
                            next_visible(&shares3, &hidden(), cur, 1)
                        }
                        Key::Escape => None,
                        _ => return,
                    };
                    e.prevent_default();
                    focus_i.set(n);
                },
                for (i, (x, w, sh)) in layout.iter().copied().enumerate() {
                    {
                        let s = shares[i].clone();
                        let dim = focus_i().is_some() && focus_i() != Some(i);
                        let inside = w >= 0.13;
                        rsx! {
                            span { key: "{s.id}", class: "oxm-sseg", "data-s": "{i % 6}", "data-dim": "{dim}", "data-off": "{w == 0.0}",
                                style: "clip-path:inset(0 {(1.0 - x - w).max(0.0) * 100.0}% 0 {x * 100.0}%)",
                                onpointerenter: move |_| focus_i.set(Some(i)),
                                span { class: "oxm-slab", style: "left:{x * 100.0}%;width:{w * 100.0}%", "data-in": "{inside}",
                                    if inside {
                                        span { class: "oxm-sname", "data-open": "{focus_i() == Some(i) || w > 0.22}", "{s.label}" }
                                        span { class: "oxm-spct", "{pct1(sh)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxm-souts", "aria-hidden": "true", style: "height:{22 + out_rows * 15}px",
                for (i, x, w, sh, row) in outs {
                    span { key: "{shares[i].id}", class: "oxm-sout", style: "left:{(x + w / 2.0) * 100.0}%;top:{6 + row * 15}px",
                        span { class: "oxm-lead", style: "top:{-7 - row as i32 * 15}px;height:{6 + row * 15}px" } "{shares[i].label} {pct1(sh)}"
                    }
                }
            }
            if legend {
                div { class: "oxm-legend",
                    for (i, s) in shares.iter().enumerate() {
                        button { key: "{s.id}", class: "oxm-li", r#type: "button", "aria-pressed": "{!hidden().contains(&s.id)}",
                            "data-off": "{hidden().contains(&s.id)}",
                            onpointerenter: move |_| focus_i.set(Some(i)), onpointerleave: move |_| focus_i.set(None),
                            onclick: move |_| toggle(i),
                            span { class: "oxm-sw", "data-s": "{i % 6}" }
                            span { "{s.label}" }
                        }
                    }
                }
            }
        }
    }
}

/// Context field (affectUI Context Window behaviour): what fills the model's context as a field
/// of cells, filled in part order. Pointing at a part lights its cells and quiets the rest; parts
/// with items open to list them with × to remove; Compact shimmers the part's cells and clears
/// the freed ones. From `warn_at` the percentage turns amber and the tray says how much is left;
/// from 95 % the window counts as full.
#[component]
pub fn ContextField(
    /// Heading.
    #[props(default = "Context window".to_string())]
    title: String,
    /// Small tag in the head (the model).
    #[props(default)]
    model: Option<String>,
    /// Parts in fill order.
    parts: Vec<Part>,
    /// Window size in tokens.
    limit: u64,
    /// Warning share.
    #[props(default = 0.8)]
    warn_at: f64,
    /// Number of cells.
    #[props(default = 216)]
    cells: usize,
    /// Cells per row.
    #[props(default = 36)]
    columns: usize,
    /// Button on the tray ("New chat").
    #[props(default)]
    action: Option<String>,
    /// Part and item removed.
    #[props(default)]
    on_remove: EventHandler<(String, String)>,
    /// Part compacted.
    #[props(default)]
    on_compact: EventHandler<String>,
    /// Tray action.
    #[props(default)]
    on_action: EventHandler<()>,
) -> Element {
    let mut f = use_signal(move || CellField {
        parts: parts.clone(),
        limit,
        warn_at,
    });
    let mut hover = use_signal(|| None::<usize>);
    let mut open = use_signal(|| None::<usize>);
    let mut clearing = use_signal(|| (Vec::<usize>::new(), 0u32));
    let fv = f();
    let grid = fv.cells(cells);
    let share = fv.share();
    let level = match fv.fill() {
        Fill::Ok => "ok",
        Fill::Warn => "warn",
        Fill::Full => "full",
    };
    let (clr, gn) = clearing();
    rsx! {
        div { class: "oxm-ctx", "data-level": "{level}",
            div { class: "oxm-chead",
                span { class: "oxm-utitle", "{title}" }
                if let Some(m) = &model { span { class: "oxm-tag", "{m}" } }
                span { class: "oxm-cpct", Roll { text: format!("{:.0}%", share * 100.0) } }
            }
            div { key: "{gn}", "data-gen": "{gn}", class: "oxm-cells", style: "grid-template-columns:repeat({columns},1fr)", role: "img",
                "aria-label": "{tokens(fv.used())} of {tokens(fv.limit)} tokens in use",
                for (i, c) in grid.iter().enumerate() {
                    span { key: "{i}", class: if clr.contains(&i) { "oxm-cell clr" } else { "oxm-cell" },
                        "data-p": if let Some(p) = c { "{p % 6}" } else { "free" },
                        "data-dim": "{hover().is_some() && *c != hover()}",
                        style: if clr.contains(&i) { "animation-delay:{(i % columns) * 8}ms" } }
                }
            }
            div { class: "oxm-parts",
                for (pi, p) in fv.parts.iter().enumerate() {
                    {
                        let p = p.clone();
                        let pid = p.id.clone();
                        let pid2 = p.id.clone();
                        let is_open = open() == Some(pi);
                        rsx! {
                            div { key: "{p.id}", class: "oxm-part", "data-open": "{is_open}",
                                onpointerenter: move |_| hover.set(Some(pi)), onpointerleave: move |_| hover.set(None),
                                div { class: "oxm-prow",
                                    span { class: "oxm-sw", "data-p": "{pi % 6}" }
                                    if p.items.is_empty() {
                                        span { class: "oxm-pl", "{p.label}" }
                                    } else {
                                        button { class: "oxm-pl btn", r#type: "button", "aria-expanded": "{is_open}",
                                            onclick: move |_| open.set(if open() == Some(pi) { None } else { Some(pi) }),
                                            "{p.label}" span { class: "oxm-pn", "{p.items.len()}" }
                                            Icon { name: "chevron", size: 10.0 }
                                        }
                                    }
                                    if p.locked { span { class: "oxm-lock", title: "Locked", Icon { name: "lock", size: 10.0 } } }
                                    span { class: "oxm-pt", "{tokens(p.size())}" }
                                    if p.compact.is_some() {
                                        button { class: "oxm-pbtn", r#type: "button",
                                            onclick: move |_| {
                                                let before = f().cells(cells);
                                                f.write().compact(&pid);
                                                let after = f().cells(cells);
                                                let freed: Vec<usize> = (0..cells).filter(|i| before[*i].is_some() && after[*i].is_none()).collect();
                                                let g = clearing().1 + 1;
                                                clearing.set((freed, g));
                                                on_compact.call(pid.clone());
                                            },
                                            Icon { name: "compact", size: 11.0 } "Compact"
                                        }
                                    }
                                }
                                if is_open {
                                    div { class: "oxm-items",
                                        for it in p.items.iter() {
                                            {
                                                let iid = it.id.clone();
                                                let pid3 = pid2.clone();
                                                rsx! {
                                                    div { key: "{it.id}", class: "oxm-item",
                                                        Icon { name: "file", size: 11.0 }
                                                        span { class: "oxm-il", "{it.label}" }
                                                        span { class: "oxm-pt", "{tokens(it.tokens)}" }
                                                        if !p.locked {
                                                            button { class: "oxm-x", r#type: "button", "aria-label": "Remove {it.label}",
                                                                onclick: move |_| { if f.write().remove(&pid3, &iid) { on_remove.call((pid3.clone(), iid.clone())) } },
                                                                Icon { name: "x", size: 10.0 }
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
            }
            div { class: "oxm-tray",
                span { key: "{level}", class: "oxm-trayt", role: "status", "{fv.tray()}" }
                if let Some(a) = &action { button { class: "oxm-pbtn solid", r#type: "button", onclick: move |_| on_action.call(()), "{a}" } }
            }
        }
    }
}

/// A model for [`ContextWindowGrid`]: id, name, window and reserve in tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct CtxModel {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Window in tokens.
    pub window: u64,
    /// Tokens kept free for the reply.
    pub reserve: u64,
}

/// Context window grid (Arc behaviour): the window as 200 cells. The system prompt, tools, files,
/// summary and messages fill it in order; a threshold row marks where compaction starts. Send
/// message, Attach file and Compact now show exactly how much still fits: new tokens fill in a
/// short wave from where the change happened, crossing the threshold compacts on its own (the
/// messages fold into a summary placed after the files), and switching models re-scales every
/// cell. Pointing at a legend row lights its cells.
#[component]
pub fn ContextWindowGrid(
    /// Models to switch between.
    models: Vec<CtxModel>,
    /// What is in the window.
    items: Vec<GridItem>,
    /// Compaction threshold.
    #[props(default = 0.8)]
    threshold: f64,
    /// Turns that Send message adds, in order.
    #[props(default)]
    turns: Vec<(String, u64)>,
    /// Files that Attach file adds, in order.
    #[props(default)]
    files: Vec<(String, u64)>,
    /// Number of cells.
    #[props(default = 200)]
    cells: usize,
    /// Shared clock (the model menu's type-ahead).
    now: ClockMs,
    /// After each compaction: folded, summary, before, after, automatic.
    #[props(default)]
    on_compact: EventHandler<(usize, u64, u64, u64, bool)>,
) -> Element {
    let m0 = models.first().cloned();
    let mut g = use_signal(move || ContextGrid {
        items: items.clone(),
        window: m0.as_ref().map_or(200_000, |m| m.window),
        reserve: m0.as_ref().map_or(0, |m| m.reserve),
        threshold,
        last: None,
    });
    let mut wave = use_signal(|| (0usize, 0u32));
    let mut hover = use_signal(|| None::<Cat>);
    let mut ti = use_signal(|| 0usize);
    let mut fi = use_signal(|| 0usize);
    let mut status = use_signal(String::new);
    let gv = g();
    let grid = gv.cells(cells);
    let cols = 20usize;
    let th = gv.threshold_cell(cells);
    let th_row = th.div_ceil(cols);
    let (origin, gn) = wave();
    let used_end = grid.iter().rposition(|c| c.0.is_some()).unwrap_or(0);
    let mut report = move |c: Option<oxyn_core::kit::meters::Compaction>, what: String| {
        let end = g()
            .cells(cells)
            .iter()
            .rposition(|c| c.0.is_some())
            .unwrap_or(0);
        let w = wave().1 + 1;
        wave.set((end, w));
        match c {
            Some(c) => {
                status.set(format!(
                    "{} {} messages into a {} summary · {} → {}",
                    if c.automatic {
                        "Auto-compacted"
                    } else {
                        "Compacted"
                    },
                    c.folded,
                    tokens(c.summary),
                    tokens(c.before),
                    tokens(c.after)
                ));
                on_compact.call((c.folded, c.summary, c.before, c.after, c.automatic));
            }
            None => status.set(what),
        }
    };
    let opts: Vec<Choice> = models
        .iter()
        .map(|m| Choice::new(&m.id, &m.name).hint(&tokens(m.window)))
        .collect();
    let models2 = models.clone();
    let turns2 = turns.clone();
    let files2 = files.clone();
    let pct = gv.used() as f64 / gv.window.max(1) as f64 * 100.0;
    rsx! {
        div { class: "oxm-grid",
            div { class: "oxm-chead",
                span { class: "oxm-utitle", "Context window" }
                div { class: "oxm-gmodel",
                    UnfoldSelect { label: "Model", hide_label: true, options: opts, value: models.first().map(|m| m.id.clone()),
                        searchable: "off", panel_width: 240.0, align: "end", now,
                        on_change: move |id: String| {
                            if let Some(m) = models2.iter().find(|m| m.id == id) {
                                let mut w = g.write();
                                w.window = m.window;
                                w.reserve = m.reserve;
                                drop(w);
                                let gn = wave().1 + 1;
                                wave.set((0, gn));
                                status.set(format!("{} · {} window, {} reserved for the reply", m.name, tokens(m.window), tokens(m.reserve)));
                            }
                        },
                    }
                }
            }
            div { class: "oxm-gnums",
                span { Roll { text: tokens(gv.used()) } span { class: "oxm-uof", " of {tokens(gv.window)} · " } Roll { text: format!("{pct:.1}%") } }
                span { class: "oxm-uof", "{tokens(gv.free())} free before the reply reserve" }
            }
            div { key: "{gn}", "data-gen": "{gn}", class: "oxm-gcells", style: "grid-template-columns:repeat({cols},1fr)", role: "img",
                "aria-label": "{tokens(gv.used())} of {tokens(gv.window)} tokens; compaction at {(threshold * 100.0).round()}%",
                for (i, (c, res)) in grid.iter().copied().enumerate() {
                    span { key: "{i}", class: "oxm-gcell",
                        "data-c": if let Some(c) = c { "{c.id()}" } else if res { "reserve" } else { "free" },
                        "data-dim": "{hover().is_some() && c != hover()}",
                        style: "animation-delay:{((i as i64 - origin as i64).unsigned_abs() as usize * 6).min(360)}ms",
                        "data-wave": "{i <= used_end}" }
                }
                span { class: "oxm-thline", style: "top:calc({th_row as f64 / (cells / cols) as f64 * 100.0}% - 1px)",
                    span { "Compacts at {(threshold * 100.0).round()}%" } }
            }
            div { class: "oxm-glegend",
                for c in Cat::ALL {
                    if gv.by_cat(c) > 0 {
                        div { key: "{c.id()}", class: "oxm-gl", onpointerenter: move |_| hover.set(Some(c)), onpointerleave: move |_| hover.set(None),
                            span { class: "oxm-sw", "data-c": "{c.id()}" }
                            span { "{c.label()}" }
                            span { class: "oxm-pt", Roll { text: tokens(gv.by_cat(c)) } }
                        }
                    }
                }
            }
            div { class: "oxm-gact",
                button { class: "oxm-pbtn", r#type: "button", disabled: ti() >= turns.len(),
                    onclick: move |_| if let Some((l, t)) = turns2.get(ti()).cloned() {
                        ti += 1;
                        let c = g.write().add(Cat::Messages, &l, t);
                        report(c, format!("Sent “{l}” · +{}", tokens(t)));
                    },
                    Icon { name: "up", size: 11.0 } "Send message"
                }
                button { class: "oxm-pbtn", r#type: "button", disabled: fi() >= files.len(),
                    onclick: move |_| if let Some((l, t)) = files2.get(fi()).cloned() {
                        fi += 1;
                        let c = g.write().add(Cat::Files, &l, t);
                        report(c, format!("Attached {l} · +{}", tokens(t)));
                    },
                    Icon { name: "clip", size: 11.0 } "Attach file"
                }
                button { class: "oxm-pbtn", r#type: "button", disabled: gv.by_cat(Cat::Messages) == 0,
                    onclick: move |_| { let c = g.write().compact(false); report(c, "Nothing to compact".into()); },
                    Icon { name: "compact", size: 11.0 } "Compact now"
                }
            }
            div { class: "oxm-gstatus", role: "status", span { key: "{status}", "{status}" } }
        }
    }
}
