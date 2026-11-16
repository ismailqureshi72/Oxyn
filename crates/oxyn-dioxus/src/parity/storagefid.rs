//! Storage family (variants 2): usage drawn as dot matrices – [`StorageMeter`] (a capacity bar of
//! dots over a category grid with rescan and clear), [`StorageCleanup`] (skills and commands
//! loaded every session, picked by how long they sat unused), and two Oxyn originals:
//! [`StorageWaffle`] (a 20 × 10 waffle with a "what can I win back" lens) and [`StorageForecast`]
//! (a fortnight of dot columns and the days left until full). Models:
//! `oxyn_core::parity::storagefid`. Styles: `storagefid.css` (`oxsm-`, `oxsc-`, `oxsw-`, `oxsf-`).
//! Timed states (scan, clear) end on the CSS animation's end, so no timer is needed.

use dioxus::prelude::*;
use oxyn_core::parity::storagefid::{
    CleanItem, StoreCat, clean_demo, col_dots, days_until_full, dot_counts, dot_runs, fmt_gb,
    idle_line, store_clear, store_demo, store_detail, store_hex, store_history, store_pace,
    store_used, thousands, unused,
};

fn cat_style(key: &str) -> String {
    format!("--kc:#{}", store_hex(key))
}

/// A small category glyph (our own drawing), stroked in `currentColor`.
pub(crate) fn cat_glyph(key: &str) -> Element {
    let d = match key {
        "apps" => "M5 5h5v5H5zM14 5h5v5h-5zM5 14h5v5H5zM14 14h5v5h-5z",
        "dev" => "M9 7 4 12l5 5M15 7l5 5-5 5",
        "photos" => "M4 6h16v12H4zM4 15l5-4 4 3 3-2 4 3",
        "video" => "M4 6h12v12H4zM16 10l4-2v8l-4-2",
        "docs" => "M7 3h7l4 4v14H7zM14 3v4h4M10 12h5M10 16h5",
        "music" => {
            "M9 18V6l10-2v12M9 18a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0zM19 16a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0z"
        }
        "system" => {
            "M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7zM12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4 7 17M17 7l1.4-1.4"
        }
        "caches" => {
            "M12 3l1.8 4.6L18 9l-4.2 1.4L12 15l-1.8-4.6L6 9l4.2-1.4zM18 15l.8 2 2 .8-2 .8-.8 2-.8-2-2-.8 2-.8z"
        }
        "trash" => "M5 7h14M9 7V4h6v3M7 7l1 13h8l1-13",
        _ => "M5 5h14v14H5z",
    };
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

/// Storage – Storage meter: the used figure and Rescan over a capacity bar of dots, one colour
/// block per category, then a grid of categories. Pointing at a category tells what it holds;
/// a click keeps it. Caches and Trash can be cleared from the hint under the grid.
#[component]
pub fn StorageMeter(
    /// Categories (the demo disk when `None`).
    #[props(default)]
    cats: Option<Vec<StoreCat>>,
    /// Capacity in GB.
    #[props(default = 512.0)]
    cap: f64,
    /// Dot columns in the bar.
    #[props(default = 48)]
    cols: usize,
    /// Dot rows in the bar.
    #[props(default = 4)]
    rows: usize,
    /// Starts with this category picked.
    #[props(default)]
    default_pick: Option<usize>,
    /// Rescan and clear actions.
    #[props(default = true)]
    actions: bool,
    /// A category was cleared: (key, GB freed).
    #[props(default)]
    on_clear: EventHandler<(String, f64)>,
) -> Element {
    let start = cats.unwrap_or_else(store_demo);
    let mut list = use_signal(move || start);
    let mut mode = use_signal(|| "idle");
    let mut target = use_signal(|| None::<usize>);
    let mut hov = use_signal(|| None::<usize>);
    let mut pick = use_signal(move || default_pick);
    let mut run = use_signal(|| 0u32);
    let mut note = use_signal(String::new);
    let cs = list();
    let used = store_used(&cs);
    let n = cols.max(4) * rows.max(1);
    let dots = dot_runs(&cs, cap, n);
    let focus = hov().or(pick());
    let busy = mode() != "idle";
    let sub = if mode() == "scan" {
        "Calculating…".to_string()
    } else {
        format!("{} available", fmt_gb((cap - used).max(0.0)))
    };
    // The hint under the grid: the focused category, or the biggest thing that can be cleared.
    let clear_ix = cs
        .iter()
        .enumerate()
        .filter(|(_, c)| c.floor.is_some_and(|f| c.gb > f + 0.05))
        .max_by(|a, b| {
            a.1.gb
                .partial_cmp(&b.1.gb)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i);
    let hint_ix = focus.or(clear_ix);
    let total = format!("{} of {} used", fmt_gb(used), fmt_gb(cap));
    let grid = format!(
        "grid-template-columns:repeat({},1fr);grid-template-rows:repeat({},auto)",
        cols.max(4),
        rows.max(1)
    );
    let r = run();

    rsx! {
        div { class: "oxsm", "data-mode": "{mode}",
            div { class: "oxsm-head",
                div { class: "oxsm-fig",
                    span { class: "oxsm-num", "{fmt_gb(used)}" }
                    span { class: "oxsm-of", "of {fmt_gb(cap)} used" }
                }
                if actions {
                    button { class: "oxsm-btn", r#type: "button", disabled: busy,
                        onclick: move |_| { run.set(run() + 1); mode.set("scan"); note.set("Rescanning.".into()); },
                        "Rescan"
                    }
                }
            }
            p { class: "oxsm-sub", "{sub}" }
            for k in std::iter::once(r) {
                div { key: "{k}", class: "oxsm-track", role: "img", "aria-label": "{total}",
                    div { class: "oxsm-dots", style: "{grid}",
                        for (d, k) in dots.iter().enumerate() {
                            {
                                let st = match k { Some(i) => format!("{};--d:{}", cat_style(cs[*i].key), d / rows.max(1)), None => format!("--d:{}", d / rows.max(1)) };
                                let dim = focus.is_some() && *k != focus;
                                let on = k.is_some();
                                let out = mode() == "clear" && k.is_some() && *k == target();
                                rsx! { i { key: "{d}", class: "oxsm-dot", "data-on": "{on}", "data-dim": "{dim}", "data-out": "{out}", style: "{st}" } }
                            }
                        }
                    }
                    if mode() == "scan" {
                        span { class: "oxsm-sweep", "aria-hidden": "true",
                            onanimationend: move |_| { mode.set("idle"); note.set("Scan finished.".into()); }
                        }
                    }
                }
            }
            div { class: "oxsm-grid", role: "list",
                for (i, c) in cs.iter().enumerate() {
                    {
                        let st = cat_style(c.key);
                        let on = pick() == Some(i);
                        let size = fmt_gb(c.gb);
                        rsx! {
                            button { key: "{c.key}", class: "oxsm-cat", r#type: "button", role: "listitem", "aria-pressed": "{on}",
                                "data-dim": "{focus.is_some() && focus != Some(i)}",
                                onpointerenter: move |_| hov.set(Some(i)),
                                onpointerleave: move |_| hov.set(None),
                                onclick: move |_| pick.set(Some(i)),
                                span { class: "oxsm-ic", style: "{st}", {cat_glyph(c.key)} }
                                span { class: "oxsm-cl", "{c.label}" }
                                span { class: "oxsm-cs", "{size}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxsm-hint", role: "status", "aria-live": "polite",
                if mode() == "scan" {
                    span { class: "oxsm-ht", "Looking for space to win back…" }
                } else if mode() == "clear" {
                    {
                        let lab = target().map(|t| cs[t].label).unwrap_or("");
                        rsx! {
                            span { class: "oxsm-ht", "Clearing {lab}…" }
                            span { class: "oxsm-busy", "aria-hidden": "true",
                                onanimationend: move |_| {
                                    if let Some(t) = target() {
                                        let key = list.peek()[t].key;
                                        let freed = store_clear(&mut list.write(), key);
                                        note.set(format!("Freed {}.", fmt_gb(freed)));
                                        on_clear.call((key.to_string(), freed));
                                    }
                                    mode.set("idle");
                                }
                            }
                        }
                    }
                } else if let Some(h) = hint_ix {
                    {
                        let c = cs[h].clone();
                        let st = cat_style(c.key);
                        let can = actions && c.floor.is_some_and(|f| c.gb > f + 0.05);
                        let line = if focus.is_some() { store_detail(&c) } else { format!("{} hold {} you can win back.", c.label, fmt_gb(c.gb - c.floor.unwrap_or(0.0))) };
                        let act = if c.key == "trash" { "Empty Trash".to_string() } else { format!("Clear {}", c.label.to_lowercase()) };
                        rsx! {
                            span { class: "oxsm-ic", style: "{st}", {cat_glyph(c.key)} }
                            span { class: "oxsm-ht", "{line}" }
                            if can {
                                button { class: "oxsm-btn oxsm-act", r#type: "button",
                                    onclick: move |_| { target.set(Some(h)); mode.set("clear"); },
                                    "{act}"
                                }
                            }
                        }
                    }
                } else {
                    span { class: "oxsm-ht", "Nothing quick to clear right now." }
                }
            }
            p { class: "oxsm-note", "{note}" }
        }
    }
}

/// Storage – Storage cleanup: skills and commands an agent loads in every session, filtered by
/// how long they sat unused. Pick some, see the tokens it frees per session as a row of dots,
/// and move them to the Trash (with Undo).
#[component]
pub fn StorageCleanup(
    /// Items (the demo list when `None`).
    #[props(default)]
    items: Option<Vec<CleanItem>>,
    /// Starting window in days (30, 90 or 180).
    #[props(default = 30)]
    default_days: u32,
    /// Items were moved to the Trash (names).
    #[props(default)]
    on_trash: EventHandler<Vec<String>>,
) -> Element {
    let start = items.unwrap_or_else(clean_demo);
    let mut list = use_signal(move || start);
    let mut days = use_signal(move || default_days);
    let mut off = use_signal(Vec::<&'static str>::new);
    let mut undo = use_signal(|| None::<Vec<CleanItem>>);
    let mut note = use_signal(String::new);
    let all = list();
    let idx = unused(&all, days());
    let picked: Vec<usize> = idx
        .iter()
        .copied()
        .filter(|i| !off().contains(&all[*i].name))
        .collect();
    let never = idx.iter().filter(|i| all[**i].idle.is_none()).count();
    let total: u32 = all.iter().map(|x| x.tokens).sum();
    let frees: u32 = picked.iter().map(|i| all[*i].tokens).sum();
    let lit = if total == 0 {
        0
    } else {
        ((frees as f64 / total as f64) * 40.0).round() as usize
    };
    let k = picked.len();
    let label = format!("Move {k} to Trash");
    let tok = thousands(frees as u64);
    let tot = thousands(total as u64);
    let all_on = !idx.is_empty() && k == idx.len();

    rsx! {
        div { class: "oxsc",
            div { class: "oxsc-head",
                span { class: "oxsc-title", "Free up context" }
                span { class: "oxsc-sub", "Skills and commands your agents load in every session." }
            }
            div { class: "oxsc-win",
                span { class: "oxsc-wl", "Not used in" }
                div { class: "oxsc-seg", role: "radiogroup", "aria-label": "Not used in",
                    for (d, name) in [(30u32, "30 days"), (90, "90 days"), (180, "6 months")] {
                        button { key: "{d}", r#type: "button", role: "radio", class: "oxsc-sb", "aria-checked": "{days() == d}",
                            onclick: move |_| { days.set(d); off.set(Vec::new()); }, "{name}" }
                    }
                }
            }
            div { class: "oxsc-stats",
                span { b { "{idx.len()}" } " unused" }
                span { b { "{never}" } " never used" }
                span { b { "{k}" } " selected" }
            }
            div { class: "oxsc-meter",
                div { class: "oxsc-dots", role: "img", "aria-label": "Frees {tok} of {tot} tokens per session",
                    for d in 0..40usize { i { key: "{d}", class: "oxsc-dot", "data-on": "{d < lit}" } }
                }
                span { class: "oxsc-ml", b { "{tok}" } " of {tot} tokens freed every session" }
            }
            div { class: "oxsc-listhead",
                button { class: "oxsc-all", r#type: "button", role: "checkbox", "aria-checked": "{all_on}",
                    disabled: idx.is_empty(),
                    onclick: move |_| {
                        if all_on {
                            off.set(unused(&list(), days()).into_iter().map(|i| list()[i].name).collect());
                        } else {
                            off.set(Vec::new());
                        }
                    },
                    i { class: "oxsc-box", "data-on": "{all_on}" }
                    "Select all"
                }
            }
            ul { class: "oxsc-list",
                for i in idx.iter().copied() {
                    {
                        let it = all[i].clone();
                        let on = !off().contains(&it.name);
                        let idle = idle_line(it.idle);
                        let nm = it.name;
                        rsx! {
                            li { key: "{it.name}",
                                button { class: "oxsc-row", r#type: "button", role: "checkbox", "aria-checked": "{on}",
                                    onclick: move |_| {
                                        let mut o = off();
                                        if let Some(p) = o.iter().position(|x| *x == nm) { o.remove(p); } else { o.push(nm); }
                                        off.set(o);
                                    },
                                    i { class: "oxsc-box", "data-on": "{on}" }
                                    span { class: "oxsc-nm", "{it.name}" }
                                    span { class: "oxsc-pl", "{it.place}" }
                                    span { class: "oxsc-id", "data-never": "{it.idle.is_none()}", "{idle}" }
                                    span { class: "oxsc-tk", "{it.tokens}" }
                                }
                            }
                        }
                    }
                }
                if idx.is_empty() { li { class: "oxsc-empty", "Everything here was used recently." } }
            }
            div { class: "oxsc-foot",
                button { class: "oxsc-go", r#type: "button", disabled: k == 0,
                    onclick: move |_| {
                        let before = list();
                        let names: Vec<&'static str> = picked.iter().map(|i| before[*i].name).collect();
                        list.set(before.iter().filter(|x| !names.contains(&x.name)).cloned().collect());
                        undo.set(Some(before));
                        note.set(format!("Moved {} to Trash.", names.len()));
                        on_trash.call(names.iter().map(|s| s.to_string()).collect());
                    },
                    "{label}"
                }
                if let Some(prev) = undo() {
                    button { class: "oxsc-undo", r#type: "button",
                        onclick: move |_| { list.set(prev.clone()); undo.set(None); note.set("Restored.".into()); },
                        "Undo"
                    }
                }
                span { class: "oxsc-note", role: "status", "aria-live": "polite", "{note}" }
            }
        }
    }
}

/// Storage – Storage waffle (an Oxyn original): the disk as 200 dots, half a percent each, one
/// colour per category. The "Win back" lens keeps only what can be cleared lit.
#[component]
pub fn StorageWaffle(
    /// Categories (the demo disk when `None`).
    #[props(default)]
    cats: Option<Vec<StoreCat>>,
    /// Capacity in GB.
    #[props(default = 512.0)]
    cap: f64,
    /// Starting lens: `kind` or `clear` (what can be won back).
    #[props(default = "kind".to_string())]
    default_lens: String,
    /// Greys instead of category colours.
    #[props(default)]
    mono: bool,
) -> Element {
    let cs = cats.unwrap_or_else(store_demo);
    let mut lens = use_signal(move || default_lens.clone());
    let mut hov = use_signal(|| None::<usize>);
    let dots = dot_runs(&cs, cap, 200);
    let counts = dot_counts(&cs, cap, 200);
    let used = store_used(&cs);
    let win: f64 = cs
        .iter()
        .map(|c| c.floor.map_or(0.0, |f| (c.gb - f).max(0.0)))
        .sum();
    let clear_lens = lens() == "clear";
    let head = if clear_lens {
        format!("{} you can win back", fmt_gb(win))
    } else {
        format!("{} of {} used", fmt_gb(used), fmt_gb(cap))
    };

    rsx! {
        div { class: "oxsw", "data-mono": "{mono}",
            div { class: "oxsw-head",
                div { class: "oxsw-hl",
                    span { class: "oxsw-title", "Disk" }
                    for k in std::iter::once(head.clone()) { span { key: "{k}", class: "oxsw-fig", "{k}" } }
                }
                div { class: "oxsc-seg", role: "radiogroup", "aria-label": "Lens",
                    for (k, name) in [("kind", "By kind"), ("clear", "Win back")] {
                        button { key: "{k}", r#type: "button", role: "radio", class: "oxsc-sb", "aria-checked": "{lens() == k}",
                            onclick: move |_| lens.set(k.to_string()), "{name}" }
                    }
                }
            }
            div { class: "oxsw-body",
                div { class: "oxsw-grid", role: "img", "aria-label": "{head}",
                    for (d, k) in dots.iter().enumerate() {
                        {
                            let st = match k { Some(i) => cat_style(cs[*i].key), None => String::new() };
                            let dim = match k {
                                Some(i) => (clear_lens && cs[*i].floor.is_none()) || hov().is_some_and(|h| h != *i),
                                None => hov().is_some() || clear_lens,
                            };
                            rsx! { i { key: "{d}", class: "oxsw-dot", "data-on": "{k.is_some()}", "data-dim": "{dim}", style: "{st}" } }
                        }
                    }
                }
                ul { class: "oxsw-legend",
                    for (i, c) in cs.iter().enumerate() {
                        {
                            let st = cat_style(c.key);
                            let off = clear_lens && c.floor.is_none();
                            let size = fmt_gb(c.gb);
                            let n = counts[i];
                            rsx! {
                                li { key: "{c.key}", class: "oxsw-li", "data-off": "{off}", "data-hov": "{hov() == Some(i)}", tabindex: "0",
                                    "aria-label": "{c.label}: {size}, {n} dots",
                                    onpointerenter: move |_| hov.set(Some(i)),
                                    onpointerleave: move |_| hov.set(None),
                                    onfocus: move |_| hov.set(Some(i)),
                                    onblur: move |_| hov.set(None),
                                    i { class: "oxsw-sw", style: "{st}" }
                                    span { class: "oxsw-ln", "{c.label}" }
                                    span { class: "oxsw-ls", "{size}" }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxsw-foot", "One dot is {fmt_gb(cap / 200.0)}." }
        }
    }
}

/// Storage – Storage forecast (an Oxyn original): a fortnight of dot columns, each lit to that
/// day's use, then ghost columns at the current pace up to the day the disk is full.
#[component]
pub fn StorageForecast(
    /// Used GB per day, oldest first (the demo fortnight when `None`).
    #[props(default)]
    history: Option<Vec<f64>>,
    /// Capacity in GB.
    #[props(default = 512.0)]
    cap: f64,
    /// Dots per column.
    #[props(default = 12)]
    rows: usize,
    /// Ghost columns ahead.
    #[props(default = 8)]
    ahead: usize,
    /// GB a day for the ghost columns (the history's own pace when `None`).
    #[props(default)]
    pace: Option<f64>,
) -> Element {
    let h = history.unwrap_or_else(store_history);
    let mut hov = use_signal(|| None::<usize>);
    let p = pace.unwrap_or_else(|| store_pace(&h));
    let last = h.last().copied().unwrap_or(0.0);
    let n = h.len();
    let cols: Vec<(f64, bool)> = h
        .iter()
        .map(|g| (*g, false))
        .chain((1..=ahead).map(|d| ((last + p * d as f64 * 7.0).min(cap), true)))
        .collect();
    let left = days_until_full(last, cap, p);
    let summary = match left {
        Some(0) => "Full now".to_string(),
        Some(d) => format!("Full in about {d} days at {} a day", fmt_gb(p)),
        None => "Not growing".to_string(),
    };
    let rows = rows.max(4);
    let hot_row = ((rows as f64) * 0.9).floor() as usize;
    let shown = hov().unwrap_or(n - 1);
    let (g, ghost) = cols.get(shown).copied().unwrap_or((last, false));
    let when = if ghost {
        format!("in {} weeks", shown + 1 - n)
    } else if shown + 1 == n {
        "Today".to_string()
    } else {
        format!("{} days ago", n - 1 - shown)
    };
    let pct = (g / cap.max(0.001) * 100.0).round() as u32;

    rsx! {
        div { class: "oxsf",
            div { class: "oxsf-head",
                div { class: "oxsf-hl",
                    span { class: "oxsf-title", "Disk forecast" }
                    span { class: "oxsf-sum", "{summary}" }
                }
                div { class: "oxsf-read", "aria-live": "polite",
                    span { class: "oxsf-when", "{when}" }
                    span { class: "oxsf-val", "{fmt_gb(g)} · {pct}%" }
                }
            }
            div { class: "oxsf-chart", role: "img", "aria-label": "Used {fmt_gb(last)} of {fmt_gb(cap)}. {summary}.",
                onpointerleave: move |_| hov.set(None),
                for (c, (gb, ghost)) in cols.iter().enumerate() {
                    {
                        let lit = col_dots(*gb, cap, rows);
                        let gap = *ghost && c == n;
                        rsx! {
                            div { key: "{c}", class: "oxsf-col", "data-ghost": "{ghost}", "data-gap": "{gap}", "data-hov": "{hov() == Some(c)}",
                                onpointerenter: move |_| hov.set(Some(c)),
                                for r in 0..rows {
                                    i { key: "{r}", class: "oxsf-dot", "data-on": "{r < lit}", "data-hot": "{r >= hot_row}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxsf-axis", span { "2 weeks ago" } span { "Today" } span { "+{ahead} weeks" } }
        }
    }
}
