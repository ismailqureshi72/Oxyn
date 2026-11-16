//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32): the exact **Away Summary**
//! ([`AwaySummary`]). Behaviour and look only; the model is `oxyn_core::parity::awayfid`.
//! Styles: `awayfid.css` (classes `oxaw-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::awayfid::{
        AWAY_MINUTES, AwayItem, AwayKind, away_clock, away_counts, away_demo, away_divider,
        away_duration, away_needs_word, away_order, away_resolve,
    },
};

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

fn mark(k: AwayKind) -> Element {
    match k {
        AwayKind::NeedsYou => rsx! {
            svg { class: "oxaw-mk", "data-k": "needs-you", width: "13", height: "13", view_box: "0 0 14 14", "aria-hidden": "true",
                circle { cx: "7", cy: "7", r: "5.6", fill: "none", stroke: "var(--oxaw-needs)", stroke_width: "1.6" }
                circle { class: "oxaw-breath", cx: "7", cy: "7", r: "2.3", fill: "var(--oxaw-needs)" } }
        },
        AwayKind::Failed => rsx! {
            svg { class: "oxaw-mk", "data-k": "failed", width: "13", height: "13", view_box: "0 0 14 14", "aria-hidden": "true",
                circle { cx: "7", cy: "7", r: "6.5", fill: "var(--oxaw-failed)" }
                path { d: "M7 3.6v4.2", stroke: "#fff", stroke_width: "1.7", stroke_linecap: "round" }
                circle { cx: "7", cy: "10.2", r: ".95", fill: "#fff" } }
        },
        AwayKind::Done => rsx! {
            svg { class: "oxaw-mk", "data-k": "done", width: "13", height: "13", view_box: "0 0 14 14", "aria-hidden": "true",
                circle { cx: "7", cy: "7", r: "6.5", fill: "var(--oxaw-done)" }
                path { class: "oxaw-tick", d: "M4.2 7.2l1.9 1.9 3.7-4", fill: "none", stroke: "#fff", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round" } }
        },
    }
}

fn chev(up: bool) -> Element {
    let d = if up {
        "M4 9.5 8 5.5l4 4"
    } else {
        "M4 6.5l4 4 4-4"
    };
    rsx! { svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } } }
}

/// Away Summary (affectUI): the "While you were away" card for a chat thread – how long the
/// person was gone, then what needs them, what failed and what's done as one row each with one
/// action; Show N more past `limit`; Got it (or Escape) folds it into a one-line divider that
/// opens it again. A row's action resolves it in place: the mark pops, the tick draws, counts roll.
#[component]
pub fn AwaySummary(
    /// Your own items (the demo's when `None`); what needs you comes first.
    #[props(default)]
    items: Option<Vec<AwayItem>>,
    /// How long the person was away, in minutes (the head reads "2h 14m").
    #[props(default = AWAY_MINUTES)]
    away: u32,
    /// When set (minutes after midnight), the head reads "since 9:40" instead.
    #[props(default)]
    since_clock: Option<u32>,
    /// Rows before Show N more.
    #[props(default = 5)]
    limit: usize,
    /// Starts folded into the divider.
    #[props(default)]
    default_collapsed: bool,
    /// Starts with every row shown.
    #[props(default)]
    expanded: bool,
    /// Rows that start resolved (ids).
    #[props(default)]
    resolved: Vec<String>,
    /// How long an action works before it settles, in ms (0 settles at once).
    #[props(default)]
    work_ms: u64,
    /// Heading.
    #[props(default = "While you were away".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A row's action (item id).
    #[props(default)]
    on_action: EventHandler<String>,
    /// Got it was pressed.
    #[props(default)]
    on_dismiss: EventHandler<()>,
    /// Folded (true) or opened (false).
    #[props(default)]
    on_collapsed_change: EventHandler<bool>,
) -> Element {
    // Order once, so a row that resolves stays where it is.
    let base = use_signal(move || away_order(&items.unwrap_or_else(away_demo)));
    let done = use_signal(move || resolved.clone());
    let mut busy = use_signal(Vec::<(String, ClockMs)>::new);
    let mut folded = use_signal(move || default_collapsed);
    let mut more = use_signal(move || expanded);
    let mut last_act = use_signal(|| None::<String>);
    let mut want_div = use_signal(|| false);
    let mut want_row = use_signal(|| None::<usize>);
    let mut last = use_signal(|| None::<usize>);

    // Actions that finished working count as resolved (no writes while rendering).
    let mut eff: Vec<String> = done();
    for (id, at) in busy.read().iter() {
        if now.since(*at).as_millis() as u64 >= work_ms && !eff.contains(id) {
            eff.push(id.clone());
        }
    }
    let rows: Vec<AwayItem> = base
        .read()
        .iter()
        .map(|i| {
            if eff.contains(&i.id) {
                away_resolve(i)
            } else {
                i.clone()
            }
        })
        .collect();
    let (n, f, d) = away_counts(&rows);
    let head = since_clock.map_or_else(|| away_duration(away), away_clock);
    let hidden = rows.len().saturating_sub(limit);
    let shown = if more() {
        rows.len()
    } else {
        rows.len().min(limit)
    };
    let total = rows.len();
    let div_text = away_divider(&head, total);
    let div_label = if n > 0 {
        format!("{div_text}, {}", away_needs_word(n))
    } else {
        div_text.clone()
    };
    let said = last_act()
        .filter(|id| eff.contains(id))
        .and_then(|id| rows.iter().find(|i| i.id == id))
        .map(|t| format!("{}: {}. {}", t.title, t.kind.word(), away_needs_word(n)))
        .unwrap_or_default();
    let mut fold = move |focus_div: bool| {
        folded.set(true);
        want_div.set(focus_div);
        on_collapsed_change.call(true);
    };

    if folded() {
        return rsx! {
            div { class: "oxaw-fold", "data-style": style_attr(&style),
                button { class: "oxaw-div", r#type: "button", "aria-expanded": "false", "aria-label": "{div_label}",
                    onmounted: move |e| {
                        let el = e.data();
                        if want_div() { want_div.set(false); focus(Some(el)); }
                    },
                    onclick: move |_| {
                        folded.set(false);
                        want_row.set(Some(last().unwrap_or(0)));
                        on_collapsed_change.call(false);
                    },
                    i { class: "oxaw-rule" }
                    span { class: "oxaw-dt",
                        if n > 0 { i { class: "oxaw-dot", "data-k": "needs-you" } }
                        "{div_text}"
                        {chev(false)}
                    }
                    i { class: "oxaw-rule" }
                }
            }
        };
    }

    rsx! {
        section { class: "oxaw", "data-style": style_attr(&style), "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); fold(true) },
            div { class: "oxaw-head",
                h3 { class: "oxaw-ttl", "{title}" span { class: "oxaw-dur", " · {head}" } }
                div { class: "oxaw-cnts",
                    if n > 0 { span { key: "n{n}", "data-k": "n{n}", class: "oxaw-c", i { class: "oxaw-dot", "data-k": "needs-you" } "{away_needs_word(n)}" } }
                    if n > 0 && f + d > 0 { i { class: "oxaw-sep", "·" } }
                    if f > 0 { span { key: "f{f}", "data-k": "f{f}", class: "oxaw-c", i { class: "oxaw-dot", "data-k": "failed" } "{f} failed" } }
                    if f > 0 && d > 0 { i { class: "oxaw-sep", "·" } }
                    if d > 0 { span { key: "d{d}", "data-k": "d{d}", class: "oxaw-c", i { class: "oxaw-dot", "data-k": "done" } "{d} done" } }
                }
            }
            ul { class: "oxaw-list",
                for (ix, it) in rows.iter().take(shown).cloned().enumerate() {
                    {
                        let working = busy.read().iter().any(|b| b.0 == it.id) && !eff.contains(&it.id);
                        let flash = last_act().as_deref() == Some(it.id.as_str()) && eff.contains(&it.id);
                        let id = it.id.clone();
                        let lab = it.action_label();
                        let grow = ix >= limit;
                        rsx! {
                            li { key: "{it.id}", "data-k": "{it.id}", class: "oxaw-row", "data-kind": "{it.kind.id()}", "data-flash": "{flash}", "data-grow": "{grow}",
                                span { key: "{it.kind.id()}", "data-k": "{it.kind.id()}", class: "oxaw-mkw", {mark(it.kind)} span { class: "oxaw-sr", "{it.kind.word()}: " } }
                                span { class: "oxaw-txt",
                                    span { class: "oxaw-rt", title: "{it.title}", "{it.title}" }
                                    if !it.detail.is_empty() { span { class: "oxaw-det", title: "{it.detail}", "{it.detail}" } }
                                }
                                if let Some(l) = lab {
                                    button { class: "oxaw-act", r#type: "button", "data-kind": "{it.kind.id()}", "aria-disabled": "{working}", "data-busy": "{working}",
                                        "aria-label": "{l}: {it.title}",
                                        onmounted: move |e| {
                                            let el = e.data();
                                            if want_row() == Some(ix) { want_row.set(None); focus(Some(el)); }
                                        },
                                        onfocus: move |_| last.set(Some(ix)),
                                        onclick: move |_| {
                                            if working { return; }
                                            on_action.call(id.clone());
                                            if it.kind == AwayKind::Done { return; }
                                            last_act.set(Some(id.clone()));
                                            busy.write().push((id.clone(), now));
                                        },
                                        if working { i { class: "oxaw-spin" } }
                                        "{l}" }
                                }
                            }
                        }
                    }
                }
            }
            if hidden > 0 || n + f + d > 0 {
                div { class: "oxaw-foot",
                    if hidden > 0 {
                        button { class: "oxaw-fb", r#type: "button", "aria-expanded": "{more()}",
                            onclick: move |_| {
                                let open = !more();
                                more.set(open);
                                if open { want_row.set(Some(limit)); }
                            },
                            if more() { "Show less" } else { "Show {hidden} more" }
                            {chev(more())} }
                    }
                    span { class: "oxaw-sp" }
                    button { class: "oxaw-fb", r#type: "button", "aria-expanded": "true",
                        onclick: move |_| { on_dismiss.call(()); fold(true) },
                        svg { width: "13", height: "13", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "M3.5 8.5l3 3 6-7" } }
                        "Got it" }
                }
            }
            div { class: "oxaw-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
