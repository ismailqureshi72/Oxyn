//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Memory Panel**
//! ([`MemoryPanel`]). Behaviour and look only; the model is `oxyn_core::parity::memoryfid`.
//! Styles: `memoryfid.css` (classes `oxmp-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::memoryfid::{
        MpMemory, mp_categories, mp_counts, mp_demo, mp_groups, mp_quote, mp_status,
    },
};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

const I_SEARCH: &str = "M7 2.5a4.5 4.5 0 1 0 0 9 4.5 4.5 0 0 0 0-9zM10.3 10.3 13.5 13.5";
const I_X: &str = "M4.5 4.5l7 7M11.5 4.5l-7 7";
const I_EDIT: &str = "M10.5 3 13 5.5 6 12.5H3.5V10z";
const I_PAUSE: &str = "M5.5 3.5v9M10.5 3.5v9";
const I_PLAY: &str = "M5 3.5v9l7.5-4.5z";
const I_TRASH: &str = "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8.5h5.8l.6-8.5";
const I_PLUS: &str = "M8 3.5v9M3.5 8h9";
const I_UNDO: &str = "M5.5 5.5H10a3 3 0 0 1 0 6H7M5.5 5.5 8 3M5.5 5.5 8 8";

/// Memory Panel (affectUI): what the assistant remembers, by category – filter with chips or
/// search, pause a memory to keep it unused, edit it in place or forget it with a short undo, add
/// one yourself, or switch memory off without losing anything.
#[component]
pub fn MemoryPanel(
    /// The memories (the demo's when `None`).
    #[props(default)]
    memories: Option<Vec<MpMemory>>,
    /// Categories in display order (the demo's when `None`).
    #[props(default)]
    categories: Option<Vec<String>>,
    /// Memory on at the start.
    #[props(default = true)]
    enabled: bool,
    /// Category filter at the start (`None` = All).
    #[props(default)]
    filter: Option<String>,
    /// Open the search with this query at the start.
    #[props(default)]
    search: Option<String>,
    /// Open the new-memory editor at the start.
    #[props(default)]
    adding: bool,
    /// Show this memory's tray at the start (as if pointed at).
    #[props(default)]
    pointed: Option<String>,
    /// Forget this memory at the start (its undo line shows).
    #[props(default)]
    forgotten: Option<String>,
    /// How long Undo stays, ms.
    #[props(default = 5000)]
    undo_ms: u64,
    /// Heading.
    #[props(default = "Memory".to_string())]
    title: String,
    /// The list's maximum height, px (it scrolls past it).
    #[props(default = 360)]
    max_height: u32,
    /// The clock (drives Undo).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// The memories changed (after add, edit, pause, forget or undo).
    #[props(default)]
    on_change: EventHandler<Vec<MpMemory>>,
    /// The master switch changed.
    #[props(default)]
    on_enabled_change: EventHandler<bool>,
) -> Element {
    let init = memories.clone().unwrap_or_else(mp_demo);
    let cats = categories.clone().unwrap_or_else(mp_categories);
    let gone0 = forgotten.clone();
    let mut list = use_signal(move || {
        init.iter()
            .filter(|m| Some(&m.id) != gone0.as_ref())
            .cloned()
            .collect::<Vec<_>>()
    });
    let init2 = memories.clone().unwrap_or_else(mp_demo);
    let mut undo = use_signal(move || {
        forgotten.as_ref().and_then(|id| {
            init2
                .iter()
                .position(|m| m.id == *id)
                .map(|i| (init2[i].clone(), i, now))
        })
    });
    let mut on = use_signal(move || enabled);
    let mut cat = use_signal(move || filter.clone());
    let mut query = use_signal(move || search.clone());
    let mut add = use_signal(move || adding);
    let mut draft = use_signal(String::new);
    let mut draft_cat = use_signal(|| 0usize);
    let mut point = use_signal(move || pointed.clone());
    let mut edit = use_signal(|| Option::<(String, String)>::None);
    let mut seq = use_signal(|| 0u32);
    let mut live_msg = use_signal(String::new);

    let l = list();
    let counts = mp_counts(&l, &cats);
    let q = query().unwrap_or_default();
    let groups = mp_groups(&l, &cats, cat().as_deref(), &q);
    let pointed_m = point().and_then(|id| l.iter().find(|m| m.id == id).cloned());
    let undo_live = undo().filter(|(_, _, at)| now.since(*at).as_millis() as u64 <= undo_ms);
    let status = mp_status(&l, on(), pointed_m.as_ref());
    let off = !on();
    let chip_ids: Vec<Option<String>> = std::iter::once(None)
        .chain(cats.iter().cloned().map(Some))
        .collect();

    let mut commit = move |v: Vec<MpMemory>| {
        list.set(v.clone());
        on_change.call(v);
    };

    rsx! {
        section { class: "oxmp", "data-style": style_attr(&style), "data-off": off, "aria-label": "{title}",
            header { class: "oxmp-bar",
                h2 { class: "oxmp-ttl", "{title}" }
                span { class: "oxmp-badge", "{l.len()}" }
                span { class: "oxmp-sp" }
                span { class: "oxmp-swl", "aria-hidden": "true", "Use memory" }
                button { class: "oxmp-sw", r#type: "button", role: "switch", "aria-checked": on(), "aria-label": "Use memory",
                    onclick: move |_| { let v = !on(); on.set(v); on_enabled_change.call(v); },
                    i {}
                }
            }
            if off {
                p { class: "oxmp-note", role: "note", "New chats won't read or save memories while this is off. Nothing below is deleted." }
            }
            div { class: "oxmp-filter",
                if let Some(qv) = query() {
                    div { class: "oxmp-search",
                        span { class: "oxmp-sic", {icon(I_SEARCH, 13)} }
                        input { class: "oxmp-in", r#type: "search", value: "{qv}", placeholder: "Search memories", "aria-label": "Search memories",
                            oninput: move |e| query.set(Some(e.value())),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { query.set(None); } }
                        if !qv.is_empty() {
                            button { class: "oxmp-ib", r#type: "button", "aria-label": "Clear search", onclick: move |_| query.set(Some(String::new())), {icon(I_X, 12)} }
                        }
                        button { class: "oxmp-close", r#type: "button", onclick: move |_| query.set(None), "Close search" }
                    }
                } else {
                    div { class: "oxmp-chips", role: "radiogroup", "aria-label": "Category",
                        onkeydown: {
                            let chip_ids = chip_ids.clone();
                            move |e: KeyboardEvent| {
                                let i = chip_ids.iter().position(|c| *c == cat()).unwrap_or(0);
                                let n = chip_ids.len();
                                match e.key() {
                                    Key::ArrowRight => { e.prevent_default(); cat.set(chip_ids[(i + 1) % n].clone()); }
                                    Key::ArrowLeft => { e.prevent_default(); cat.set(chip_ids[(i + n - 1) % n].clone()); }
                                    _ => {}
                                }
                            }
                        },
                        for (i, c) in chip_ids.iter().enumerate() {
                            {
                                let c2 = c.clone();
                                let checked = cat() == *c;
                                let (label, k) = match c {
                                    None => ("All".to_string(), l.len()),
                                    Some(x) => (x.clone(), counts.iter().find(|(n, _)| n == x).map_or(0, |p| p.1)),
                                };
                                rsx! {
                                    button { class: "oxmp-chip", r#type: "button", role: "radio", key: "{i}", "data-k": "{i}", "aria-checked": checked,
                                        tabindex: if checked { "0" } else { "-1" },
                                        onclick: move |_| cat.set(c2.clone()),
                                        "{label}" span { class: "oxmp-n", "{k}" }
                                    }
                                }
                            }
                        }
                    }
                    button { class: "oxmp-ib oxmp-sbtn", r#type: "button", "aria-label": "Search memories", onclick: move |_| query.set(Some(String::new())), {icon(I_SEARCH, 13)} }
                }
            }
            div { class: "oxmp-list", style: "max-height:{max_height}px",
                if groups.is_empty() { p { class: "oxmp-empty", "Nothing matches." } }
                for (g, rows) in groups.iter() {
                    div { class: "oxmp-group", key: "{g}", "data-k": "{g}",
                        span { class: "oxmp-gh", "{g}" }
                        for m in rows.iter() {
                            {
                                let m = (*m).clone();
                                let id = m.id.clone();
                                let (id_e, id_p, id_f, id_h, id_s) = (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
                                let editing = edit().filter(|(e, _)| *e == m.id).map(|(_, t)| t);
                                let shown = point().as_deref() == Some(m.id.as_str());
                                let text = m.text.clone();
                                rsx! {
                                    div { class: "oxmp-row", key: "{m.id}", "data-k": "{m.id}", "data-paused": m.paused, "data-tray": shown,
                                        onmouseenter: move |_| point.set(Some(id_h.clone())),
                                        onmouseleave: move |_| if point().as_deref() == Some(id_s.as_str()) { point.set(None); },
                                        if let Some(t) = editing {
                                            textarea { class: "oxmp-ta", value: "{t}", rows: "2", "aria-label": "Edit memory",
                                                oninput: move |e| edit.set(Some((id_e.clone(), e.value()))),
                                                onkeydown: move |e: KeyboardEvent| {
                                                    if e.key() == Key::Escape { edit.set(None); }
                                                    if e.key() == Key::Enter && !e.modifiers().shift() {
                                                        e.prevent_default();
                                                        if let Some((eid, et)) = edit() && !et.trim().is_empty() {
                                                            let mut v = list(); if let Some(x) = v.iter_mut().find(|x| x.id == eid) { x.text = et.trim().to_string(); }
                                                            commit(v); edit.set(None); live_msg.set("Memory updated".into());
                                                        }
                                                    }
                                                } }
                                        } else {
                                            div { class: "oxmp-txt",
                                                span { class: "oxmp-t", "{m.text}" }
                                                span { class: "oxmp-meta", "{m.source} · {m.date}" if m.paused { span { class: "oxmp-paused", "Paused" } } }
                                            }
                                            div { class: "oxmp-tray", onfocusin: move |_| point.set(Some(id.clone())),
                                                button { class: "oxmp-ib", r#type: "button", "aria-label": "Edit", title: "Edit",
                                                    onclick: move |_| edit.set(Some((id_p.clone(), text.clone()))), {icon(I_EDIT, 13)} }
                                                {
                                                    let pid = m.id.clone();
                                                    let paused = m.paused;
                                                    rsx! {
                                                        button { class: "oxmp-ib", r#type: "button", "aria-label": if paused { "Resume" } else { "Pause" }, title: if paused { "Resume" } else { "Pause" }, "aria-pressed": paused,
                                                            onclick: move |_| {
                                                                let mut v = list(); if let Some(x) = v.iter_mut().find(|x| x.id == pid) { x.paused = !x.paused; }
                                                                commit(v); live_msg.set(if paused { "Memory resumed".into() } else { "Memory paused".into() });
                                                            },
                                                            if paused { {icon(I_PLAY, 13)} } else { {icon(I_PAUSE, 13)} } }
                                                    }
                                                }
                                                button { class: "oxmp-ib oxmp-del", r#type: "button", "aria-label": "Forget", title: "Forget",
                                                    onclick: move |_| {
                                                        let mut v = list();
                                                        if let Some(i) = v.iter().position(|x| x.id == id_f) {
                                                            let m = v.remove(i);
                                                            live_msg.set(format!("Forgot {}", m.text));
                                                            undo.set(Some((m, i, now)));
                                                            point.set(None);
                                                            commit(v);
                                                        }
                                                    }, {icon(I_TRASH, 13)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if add() {
                div { class: "oxmp-new",
                    textarea { class: "oxmp-ta", rows: "2", value: "{draft}", placeholder: "Something worth remembering…", "aria-label": "New memory",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: {
                            let cats = cats.clone();
                            move |e: KeyboardEvent| {
                                if e.key() == Key::Escape { add.set(false); draft.set(String::new()); }
                                if e.key() == Key::Enter && !e.modifiers().shift() {
                                    e.prevent_default();
                                    let t = draft().trim().to_string();
                                    if t.is_empty() { return; }
                                    let mut v = list(); seq.set(seq() + 1);
                                    v.insert(0, MpMemory { id: format!("new-{}", seq()), text: t, category: cats.get(draft_cat()).cloned().unwrap_or_default(), source: "You added".into(), date: "Today".into(), paused: false });
                                    commit(v); draft.set(String::new()); add.set(false); live_msg.set("Memory saved".into());
                                }
                            }
                        } }
                    div { class: "oxmp-nc", role: "radiogroup", "aria-label": "Category for the new memory",
                        for (i, c) in cats.iter().enumerate() {
                            button { class: "oxmp-chip", r#type: "button", role: "radio", key: "{c}", "data-k": "{c}", "aria-checked": draft_cat() == i, onclick: move |_| draft_cat.set(i), "{c}" }
                        }
                    }
                    div { class: "oxmp-na",
                        span { class: "oxmp-hint", "Choose a category · Enter saves" }
                        button { class: "oxmp-btn", r#type: "button", onclick: move |_| { add.set(false); draft.set(String::new()); }, "Cancel" }
                        {
                            let cats = cats.clone();
                            rsx! {
                                button { class: "oxmp-btn oxmp-pri", r#type: "button", disabled: draft().trim().is_empty(),
                                    onclick: move |_| {
                                        let t = draft().trim().to_string();
                                        if t.is_empty() { return; }
                                        let mut v = list(); seq.set(seq() + 1);
                                        v.insert(0, MpMemory { id: format!("new-{}", seq()), text: t, category: cats.get(draft_cat()).cloned().unwrap_or_default(), source: "You added".into(), date: "Today".into(), paused: false });
                                        commit(v); draft.set(String::new()); add.set(false); live_msg.set("Memory saved".into());
                                    },
                                    "Save" }
                            }
                        }
                    }
                }
            } else if let Some((m, i, _)) = undo_live.clone() {
                div { class: "oxmp-undo",
                    span { class: "oxmp-ut", "Forgot “{mp_quote(&m.text, 26)}”" }
                    button { class: "oxmp-btn", r#type: "button",
                        onclick: move |_| {
                            let mut v = list(); let at = i.min(v.len()); v.insert(at, m.clone());
                            commit(v); undo.set(None); live_msg.set("Memory restored".into());
                        },
                        {icon(I_UNDO, 12)} "Undo" }
                }
            } else {
                button { class: "oxmp-add", r#type: "button", onclick: move |_| add.set(true), {icon(I_PLUS, 13)} "New memory" }
            }
            footer { class: "oxmp-foot", "{status}" }
            div { class: "oxmp-sr", role: "status", "aria-live": "polite", "{live_msg}" }
        }
    }
}
