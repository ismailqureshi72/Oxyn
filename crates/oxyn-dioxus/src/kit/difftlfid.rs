//! affectUI fidelity pass, family B part 9 (plan #32): the **Diff Timeline** run view
//! ([`DiffTimelineRun`]), the upgrade of `diff-timeline` next to the revision rail
//! ([`crate::DiffTimeline`]). Behaviour and look only; the model is
//! `oxyn_core::kit::difftlfid`. Styles: `difftlfid.css` (classes `oxdl-*`), in
//! `PRIMITIVES_CSS`. Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set.

use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::difftlfid::{
        DtEdit, dt_clock, dt_count, dt_current, dt_demo, dt_end, dt_ext, dt_files, dt_landed,
        dt_mark, dt_series, dt_split, dt_status, dt_step, dt_totals, dt_x_axis, dt_y_axis,
    },
};

static SEQ: AtomicUsize = AtomicUsize::new(0);
/// Rest at the end before the demo loops, ms of run time.
const REST_MS: u32 = 5_000;
/// The chart's drawing box (stretched to the card).
const VW: f64 = 1000.0;
const VH: f64 = 300.0;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn path(points: &[(f64, f64)]) -> String {
    let mut s = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        s.push_str(&format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { " L" }));
    }
    s
}

fn file_glyph(ext: &str) -> &'static str {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => {
            "M3.5 2.5h9v11h-9zM3.5 11l3-3 2.5 2.5 1.5-1.5 2 2M10 5.5h.01"
        }
        "css" | "scss" => {
            "M5 3.5c-1.5 0-1.5 1-1.5 2v1L2.5 8l1 1.5v1c0 1 0 2 1.5 2M11 3.5c1.5 0 1.5 1 1.5 2v1l1 1.5-1 1.5v1c0 1 0 2-1.5 2"
        }
        _ => "M4 2.5h5.5l2.5 2.5v8.5H4zM9.5 2.5V5H12M6 8.5l-1.2 1.2L6 11M10 8.5l1.2 1.2L10 11",
    }
}

/// Diff Timeline (affectUI parity): one agent's run as lines added and removed over time – a
/// chart of the running totals (added, removed and a dashed net), one row per file touched with
/// its edits as pieces of a bar, totals in the header, and any edit opened with its changed lines.
#[component]
pub fn DiffTimelineRun(
    /// The edits (the demo's when `None`).
    #[props(default)]
    edits: Option<Vec<DtEdit>>,
    /// Ms into the run; `None` replays the demo (looping).
    #[props(default)]
    at: Option<u32>,
    /// The agent's id (gives its pixel mark).
    #[props(default = "ui-agent".to_string())]
    agent_id: String,
    /// The agent's name.
    #[props(default = "UI agent".to_string())]
    agent: String,
    /// Context after the name.
    #[props(default = "PR #57".to_string())]
    context: String,
    /// Rows shown before the list scrolls.
    #[props(default = 7)]
    max_rows: usize,
    /// An edit open from the start (index among the landed edits).
    #[props(default)]
    open: Option<usize>,
    /// The clock (drives the replay).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// An edit was opened (index into the edits) or closed.
    #[props(default)]
    on_open: EventHandler<Option<usize>>,
) -> Element {
    let uid = use_hook(|| {
        format!(
            "{:x}{}",
            now.0 % 0xFFFFF,
            SEQ.fetch_add(1, Ordering::Relaxed)
        )
    });
    let t0 = use_signal(move || now);
    let mut opened = use_signal(move || open);
    let mut hover = use_signal(|| None::<String>);
    let ed = edits.clone().unwrap_or_else(dt_demo);
    let end = dt_end(&ed);
    let t = at.unwrap_or_else(|| {
        let el = now.since(t0()).as_millis() as u32;
        (el % (end + REST_MS)).min(end)
    });
    let live = t < end;
    let landed = dt_landed(&ed, t);
    let nl = landed.len();
    let files = dt_files(&ed, t);
    let tot = dt_totals(&ed, t);
    let series = dt_series(&ed, t);
    let (xmax, xstep) = dt_x_axis(end);
    let peak = series.iter().map(|s| s.1.max(s.2)).max().unwrap_or(0);
    let (ymax, ystep) = dt_y_axis(dt_totals(&ed, end).added.max(peak));
    let px_ = |ms: u32| ms as f64 / xmax as f64 * VW;
    let py = |v: i64| VH - (v.max(0) as f64 / ymax as f64 * VH);
    let pa: Vec<(f64, f64)> = series.iter().map(|s| (px_(s.0), py(s.1 as i64))).collect();
    let pr: Vec<(f64, f64)> = series.iter().map(|s| (px_(s.0), py(s.2 as i64))).collect();
    let pn: Vec<(f64, f64)> = series
        .iter()
        .map(|s| (px_(s.0), py(s.1 as i64 - s.2 as i64)))
        .collect();
    let d_add = path(&pa);
    let d_rem = path(&pr);
    let d_net = path(&pn);
    let lastx = pa.last().map(|p| p.0).unwrap_or(0.0);
    let d_area = format!("{d_add} L{lastx:.1} {VH} L0 {VH} Z");
    let d_rarea = format!("{d_rem} L{lastx:.1} {VH} L0 {VH} Z");
    let lp = |p: &(f64, f64)| (p.0 / VW * 100.0, 100.0 - p.1 / VH * 100.0);
    let ends: Vec<(&'static str, f64, f64)> = [("add", pa.last()), ("rem", pr.last())]
        .into_iter()
        .filter_map(|(k, p)| p.map(|p| (k, lp(p).0, lp(p).1)))
        .collect();
    let yticks: Vec<u32> = (0..=ymax / ystep).map(|k| k * ystep).collect();
    let xticks: Vec<u32> = (0..=xmax / xstep).map(|k| k * xstep).collect();
    let cur = dt_current(&ed, t, end).map(|i| ed[i].file.clone());
    let status = dt_status(&ed, t);
    let net = tot.net();
    let net_s = if net >= 0 {
        format!("+{net}")
    } else {
        format!("−{}", -net)
    };
    let add_s = dt_count(tot.added, false);
    let rem_s = dt_count(tot.removed, false);
    let add_k = dt_count(tot.added, true);
    let rem_k = dt_count(tot.removed, true);
    let files_s = format!(
        "across {} file{}",
        tot.files,
        if tot.files == 1 { "" } else { "s" }
    );
    // The open edit, as a position among the landed edits.
    let op = opened().filter(|&k| k < nl);
    let open_edit = op.map(|k| landed[k]);
    let open_file = open_edit.map(|i| ed[i].file.clone());
    let hov = hover();
    let mark: Vec<bool> = dt_mark(&agent_id);
    let mut set_open = move |k: Option<usize>, landed_idx: Option<usize>| {
        opened.set(k);
        on_open.call(landed_idx);
    };
    let landed_k = landed.clone();
    let row_h = 22usize;
    let list_h = format!("{}px", max_rows * row_h);
    let pop = open_edit.map(|i| {
        let e = &ed[i];
        let upto: Vec<usize> = landed.iter().copied().take_while(|&j| j != i).collect();
        let a: u32 = upto.iter().map(|&j| ed[j].added).sum::<u32>() + e.added;
        let r: u32 = upto.iter().map(|&j| ed[j].removed).sum::<u32>() + e.removed;
        let x = px_(e.at) / VW * 100.0;
        (
            e.file.clone(),
            dt_clock(e.at),
            a,
            r,
            e.added,
            e.removed,
            e.binary,
            e.lines.clone(),
            x,
        )
    });
    let chart_lab = format!("Lines changed over the run, {} edits", nl);

    rsx! {
        section { class: "oxdl", "data-style": style_attr(&style), "data-live": live, "aria-label": "Diff timeline for {agent}",
            header { class: "oxdl-hd",
                span { class: "oxdl-mark", "data-live": live, "aria-hidden": "true",
                    for (k, on) in mark.iter().enumerate() { i { key: "{k}", "data-k": "{k}", "data-on": *on } }
                }
                div { class: "oxdl-who",
                    div { class: "oxdl-l1", b { "{agent}" } if !context.is_empty() { span { class: "oxdl-ctx", "· {context}" } } }
                    div { class: "oxdl-st", role: "status", "aria-live": "polite", "data-live": live, "{status}" }
                }
                div { class: "oxdl-tot",
                    span { class: "oxdl-chip", "data-k": "add", title: "{add_s} lines added", span { class: "oxdl-full", "+{add_s}" } span { class: "oxdl-short", "+{add_k}" } }
                    span { class: "oxdl-chip", "data-k": "rem", title: "{rem_s} lines removed", span { class: "oxdl-full", "−{rem_s}" } span { class: "oxdl-short", "−{rem_k}" } }
                    span { class: "oxdl-across", "{files_s}" }
                }
            }
            div { class: "oxdl-panel oxdl-chartp",
                div { class: "oxdl-chart", role: "group", tabindex: "0", "aria-label": "{chart_lab}", "aria-keyshortcuts": "ArrowLeft ArrowRight Home End Escape",
                    onkeydown: move |e: KeyboardEvent| {
                        let k = match e.key() {
                            Key::ArrowRight => "next",
                            Key::ArrowLeft => "prev",
                            Key::Home => "first",
                            Key::End => "last",
                            Key::Escape => {
                                if opened().is_some() { e.prevent_default(); set_open(None, None); }
                                return;
                            }
                            _ => return,
                        };
                        e.prevent_default();
                        let n = landed_k.len();
                        let nk = dt_step(opened().filter(|&x| x < n), k, n);
                        set_open(nk, nk.map(|x| landed_k[x]));
                    },
                    div { class: "oxdl-y", "aria-hidden": "true",
                        for v in yticks.iter().rev() { span { key: "{v}", "data-k": "{v}", style: "bottom:{*v as f64 / ymax as f64 * 100.0}%", "{v}" } }
                    }
                    div { class: "oxdl-plot",
                        div { class: "oxdl-grid", "aria-hidden": "true",
                            for v in yticks.iter() { i { key: "{v}", "data-k": "{v}", style: "bottom:{*v as f64 / ymax as f64 * 100.0}%" } }
                        }
                        svg { class: "oxdl-svg", view_box: "0 0 {VW} {VH}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                            path { class: "oxdl-area", "data-k": "add", d: "{d_area}" }
                            path { class: "oxdl-area", "data-k": "rem", d: "{d_rarea}" }
                            path { class: "oxdl-ln", "data-k": "rem", d: "{d_rem}" }
                            path { class: "oxdl-ln", "data-k": "net", d: "{d_net}" }
                            path { class: "oxdl-ln", "data-k": "add", d: "{d_add}" }
                        }
                        for (k, x, y) in ends.iter() {
                            span { class: "oxdl-end", key: "{k}", "data-k": "{k}", "data-live": live, style: "left:{x}%;bottom:{y}%", "aria-hidden": "true" }
                        }
                        div { class: "oxdl-base",
                            for (pos, &i) in landed.iter().enumerate() {
                                {
                                    let e = &ed[i];
                                    let on = op == Some(pos);
                                    let hl = hov.as_deref() == Some(e.file.as_str());
                                    let x = px_(e.at) / VW * 100.0;
                                    let tip = format!("{} at {}", e.file, dt_clock(e.at));
                                    rsx! {
                                        i { class: "oxdl-tick", key: "{i}", "data-k": "{i}", "data-on": on, "data-hl": hl, style: "left:{x}%", title: "{tip}",
                                            onclick: move |_| { let v = if opened() == Some(pos) { None } else { Some(pos) }; set_open(v, v.map(|_| i)); }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some((f, at_s, a, r, da, dr, bin, lines, x)) = pop.clone() {
                            {
                                let (fd, fnm) = dt_split(&f);
                                let left = format!("clamp(0px, calc({x}% - 110px), calc(100% - 230px))");
                                let pid = format!("oxdl-{uid}-pop");
                                rsx! {
                                    div { class: "oxdl-pop", id: "{pid}", role: "dialog", "aria-label": "Edit to {f} at {at_s}", "aria-live": "polite", style: "left:{left}",
                                        div { class: "oxdl-ph",
                                            span { class: "oxdl-pf", span { class: "oxdl-fd", "{fd}" } b { "{fnm}" } }
                                            span { class: "oxdl-pt", "{at_s}" }
                                        }
                                        dl { class: "oxdl-pd",
                                            div { dt { "Added" } dd { "{a} " if !bin { span { "data-k": "add", "(+{da})" } } } }
                                            div { dt { "Removed" } dd { "{r} " if !bin { span { "data-k": "rem", "(−{dr})" } } } }
                                        }
                                        if bin {
                                            p { class: "oxdl-bin", "Binary file changed" }
                                        } else if !lines.is_empty() {
                                            div { class: "oxdl-code",
                                                span { "Changed lines" }
                                                pre {
                                                    for (li, l) in lines.iter().enumerate() {
                                                        code { key: "{li}", "data-k": "{li}", "data-sign": if l.starts_with('-') { "rem" } else { "add" }, "{l}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxdl-x", "aria-hidden": "true",
                        for v in xticks.iter() { span { key: "{v}", "data-k": "{v}", style: "left:{*v as f64 / xmax as f64 * 100.0}%", "{dt_clock(*v)}" } }
                    }
                }
            }
            div { class: "oxdl-panel oxdl-files", style: "--oxdl-list-h:{list_h}", role: "list", "aria-label": "Files changed",
                for f in files.iter() {
                    {
                        let (fd, fnm) = dt_split(&f.file);
                        let ext = dt_ext(&f.file);
                        let total = (f.added + f.removed).max(1) as f64;
                        let biggest = files.iter().map(|x| x.added + x.removed).max().unwrap_or(1).max(1) as f64;
                        let bar_w = if f.binary { 18.0 } else { ((f.added + f.removed) as f64 / biggest * 100.0).max(6.0) };
                        let is_cur = cur.as_deref() == Some(f.file.as_str());
                        let dim = open_file.as_ref().is_some_and(|o| *o != f.file);
                        let fk = f.file.clone();
                        let fa = dt_count(f.added, false);
                        let fr = dt_count(f.removed, false);
                        let fak = dt_count(f.added, true);
                        let frk = dt_count(f.removed, true);
                        let n_ed = f.pieces.len();
                        let lab = if f.binary { format!("{}, binary, {} edit{}", f.file, n_ed, if n_ed == 1 { "" } else { "s" }) } else { format!("{}, {} added, {} removed, {} edit{}", f.file, f.added, f.removed, n_ed, if n_ed == 1 { "" } else { "s" }) };
                        rsx! {
                            div { class: "oxdl-row", key: "{f.file}", "data-k": "{f.file}", role: "listitem", "aria-label": "{lab}", "data-cur": is_cur, "data-dim": dim,
                                onmouseenter: move |_| hover.set(Some(fk.clone())),
                                onmouseleave: move |_| hover.set(None),
                                span { class: "oxdl-fi", "data-ext": "{ext}", "aria-hidden": "true",
                                    svg { width: "13", height: "13", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.3", stroke_linecap: "round", stroke_linejoin: "round", path { d: file_glyph(&ext) } }
                                }
                                span { class: "oxdl-fn", span { class: "oxdl-fd", "{fd}" } b { "{fnm}" } }
                                span { class: "oxdl-lane", "aria-hidden": "true",
                                    span { class: "oxdl-bar", "data-bin": f.binary, style: "width:{bar_w}%",
                                        if !f.binary {
                                            for (ei, a, r) in f.pieces.iter() {
                                                {
                                                    let wa = *a as f64 / total * 100.0;
                                                    let wr = *r as f64 / total * 100.0;
                                                    rsx! {
                                                        if *a > 0 { i { key: "a{ei}", "data-k": "a{ei}", "data-s": "add", style: "width:{wa}%" } }
                                                        if *r > 0 { i { key: "r{ei}", "data-k": "r{ei}", "data-s": "rem", style: "width:{wr}%" } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if f.binary {
                                    span { class: "oxdl-n", "data-bin": "true", "binary" }
                                } else {
                                    span { class: "oxdl-n",
                                        span { "data-k": "add", span { class: "oxdl-full", "+{fa}" } span { class: "oxdl-short", "+{fak}" } }
                                        " "
                                        span { "data-k": "rem", span { class: "oxdl-full", "−{fr}" } span { class: "oxdl-short", "−{frk}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                if files.is_empty() { div { class: "oxdl-empty", "No files changed yet" } }
            }
            div { class: "oxdl-legend", "aria-hidden": "true",
                span { "data-k": "add", i {} "Added" }
                span { "data-k": "rem", i {} "Removed" }
                span { "data-k": "net", i {} "Net {net_s}" }
            }
        }
    }
}
