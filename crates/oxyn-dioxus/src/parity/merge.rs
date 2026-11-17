//! Merge conflicts (plan #31): `MergeConflict`, the 3-way merge-conflict editor, and
//! `ConflictRadar`, agents on parallel branches and where they clash. Styles: `merge.css`
//! (classes `oxyn-mrg-*` and `oxyn-crd-*`). Models: `oxyn_core::parity::merge`.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use oxyn_core::parity::merge::{
    Merge, MergeCmd, MergeFile, Phase, Pick, Radar, RadarAgent, RadarFile, Seg, TakeKind,
    count_markers, marker_block, merge_key, pick_lines,
};

use crate::composer::icons::Icon;

// ---------------------------------------------------------------------------------------------
// Merge conflict editor
// ---------------------------------------------------------------------------------------------

/// Pane names for the two vocabularies.
fn names(labels: &str) -> (&'static str, &'static str) {
    if labels == "editor" {
        ("Current", "Incoming")
    } else {
        ("Ours", "Theirs")
    }
}

/// Which lines of a clean run of `n` lines stay visible: `(head, tail)` counts, or `None` to show
/// everything.
fn fold(n: usize, first: bool, last: bool, keep: usize) -> Option<(usize, usize)> {
    let head = if first { 0 } else { 2 };
    let tail = if last { 0 } else { 2 };
    if keep == 0 || n <= keep.max(head + tail + 1) {
        None
    } else {
        Some((head, tail))
    }
}

fn plural(n: usize, word: &str, pre: &str) -> String {
    format!("{pre}{n} {word}{}", if n == 1 { "" } else { "s" })
}

fn line_el(no: Option<usize>, text: &str, cls: &'static str) -> Element {
    let t = if text.is_empty() { " " } else { text };
    rsx! {
        div { class: "oxyn-mrg-ln {cls}",
            span { class: "oxyn-mrg-no", "aria-hidden": "true", if let Some(n) = no { "{n}" } }
            span { class: "oxyn-mrg-tx", "{t}" }
        }
    }
}

/// The 3-way merge-conflict editor. Ours · Base · Theirs input panes and a Result pane; each
/// conflict is settled with Accept ours / theirs / both / base, the agent's suggestion or a manual
/// edit; next / previous navigation, a resolved count, undo, Mark resolved per file, Complete
/// merge and Abort with an inline confirm. The component never runs git: every decision is
/// reported through the callbacks.
#[component]
pub fn MergeConflict(
    /// Conflicted files (one or more).
    files: Vec<MergeFile>,
    /// Our branch (the checked-out side).
    #[props(default = "main".to_string())]
    ours_ref: String,
    /// Their branch (being merged in).
    #[props(default = "feature".to_string())]
    theirs_ref: String,
    /// Merge-base label, usually a short sha.
    #[props(default = "merge-base".to_string())]
    base_ref: String,
    /// `three-pane` · `result-center` · `inline`.
    #[props(default = "three-pane".to_string())]
    layout: String,
    /// Show the base pane (`three-pane`) or base blocks (`inline`).
    #[props(default = true)]
    show_base: bool,
    /// `git` (Ours / Theirs) · `editor` (Current / Incoming).
    #[props(default = "git".to_string())]
    labels: String,
    /// Move to the next unresolved conflict after a pick.
    #[props(default = true)]
    auto_advance: bool,
    /// Show the agent's suggestions where the files carry them.
    #[props(default = true)]
    suggestions: bool,
    /// Clean runs longer than this fold to "⋯ n unchanged lines" (0 never folds).
    #[props(default = 6)]
    fold_after: usize,
    /// Picks already made: (file, conflict, pick).
    #[props(default)]
    default_picks: Vec<(usize, usize, Pick)>,
    /// A conflict was settled or reopened (`None`): (path, conflict, pick).
    #[props(default)]
    on_resolve: EventHandler<(String, usize, Option<Pick>)>,
    /// A file was marked resolved.
    #[props(default)]
    on_mark: EventHandler<String>,
    /// Complete merge was pressed with every file marked.
    #[props(default)]
    on_complete: EventHandler<()>,
    /// Abort was confirmed.
    #[props(default)]
    on_abort: EventHandler<()>,
) -> Element {
    let init = files.clone();
    let mut m = use_signal(move || {
        let mut s = Merge::new(init);
        for (f, n, p) in default_picks {
            s.file = f.min(s.files.len().saturating_sub(1));
            s.take(n, p, false);
        }
        s.open_file(0);
        s
    });
    let mut editing = use_signal(|| None::<(usize, usize, String)>);
    let mut preview = use_signal(|| None::<(usize, Pick)>);
    let mut open_runs = use_signal(BTreeSet::<(usize, usize)>::new);
    let mut tab = use_signal(String::new);
    let mut live = use_signal(String::new);
    let (on, tn) = names(&labels);
    let st = m();
    let f = st.file;
    let file = st.cur().clone();
    let total = st.total(f);
    let done = st.resolved(f);
    let total_all = st.total_all();
    let done_all = st.resolved_all();
    let marked = st.marked.contains(&f);
    let phase = st.phase;
    let focus = st.focus;
    let mark_why = st.mark_blocker();
    let complete_why = st.complete_blocker();
    let markers = st.markers_left();
    let multi = st.files.len() > 1;
    let pct = if total_all == 0 {
        100.0
    } else {
        done_all as f64 * 100.0 / total_all as f64
    };
    let readonly = marked || phase != Phase::Open;
    let three = layout == "three-pane";
    let center = layout == "result-center";
    let inline = layout == "inline";
    let base_on = show_base && file.conflicts().iter().any(|c| c.base.is_some());
    let path = file.path.clone();
    let path2 = path.clone();

    let mut act = move |cmd: MergeCmd| {
        let mut s = m.write();
        let n = s.focus;
        let p = s.cur().path.clone();
        match cmd {
            MergeCmd::Next => s.go(true),
            MergeCmd::Prev => s.go(false),
            MergeCmd::Take(k) => {
                let pick = k.pick();
                if s.take(n, pick.clone(), auto_advance) {
                    on_resolve.call((p, n, Some(pick)));
                }
            }
            MergeCmd::Reopen => {
                if s.pick(n).is_some() {
                    s.reopen(n);
                    on_resolve.call((p, n, None));
                }
            }
            MergeCmd::Edit => {
                if !s.marked.contains(&s.file) && s.phase == Phase::Open {
                    let c = s.cur().conflicts()[n].clone();
                    let lines = match s.pick(n) {
                        Some(pk) => pick_lines(&c, pk),
                        None => marker_block(&c, &s.cur().path, ""),
                    };
                    let file = s.file;
                    drop(s);
                    editing.set(Some((file, n, lines.join("\n"))));
                    return;
                }
            }
            MergeCmd::Undo => {
                s.undo();
            }
            MergeCmd::Mark => {
                if s.mark() {
                    on_mark.call(p);
                }
            }
        }
        let a = s.announce();
        drop(s);
        preview.set(None);
        live.set(a);
    };

    // one clean run, folded when long
    let run = move |si: usize,
                    lines: Vec<String>,
                    first: bool,
                    last: bool,
                    start: usize,
                    cls: &'static str|
          -> Element {
        let opened = open_runs.read().contains(&(f, si));
        match fold(lines.len(), first, last, fold_after).filter(|_| !opened) {
            None => {
                rsx! { for (i, l) in lines.iter().enumerate() { {line_el(Some(start + i), l, cls)} } }
            }
            Some((h, t)) => {
                let hidden = lines.len() - h - t;
                rsx! {
                    for i in 0..h { {line_el(Some(start + i), &lines[i], cls)} }
                    button { class: "oxyn-mrg-fold", r#type: "button", "data-run": "{si}",
                        onclick: move |_| { open_runs.write().insert((f, si)); },
                        {plural(hidden, "unchanged line", "⋯ ")}
                    }
                    for i in (lines.len() - t)..lines.len() { {line_el(Some(start + i), &lines[i], cls)} }
                }
            }
        }
    };

    // line numbers per side, per segment
    let mut nums: Vec<(usize, usize, usize, usize)> = Vec::new(); // ours, base, theirs, result
    {
        let (mut a, mut b, mut c, mut r) = (1, 1, 1, 1);
        let mut k = 0;
        for s in &file.segs {
            nums.push((a, b, c, r));
            match s {
                Seg::Same(l) => {
                    a += l.len();
                    b += l.len();
                    c += l.len();
                    r += l.len();
                }
                Seg::Conflict(x) => {
                    a += x.ours.len();
                    b += x.base.as_ref().map_or(0, Vec::len);
                    c += x.theirs.len();
                    r += match st.picks[f].get(&k) {
                        Some(p) => pick_lines(x, p).len(),
                        None => marker_block(x, "", "").len(),
                    };
                    k += 1;
                }
            }
        }
    }
    let nseg = file.segs.len();

    // conflict index per segment
    let mut cidx = Vec::new();
    {
        let mut k = 0;
        for s in &file.segs {
            if matches!(s, Seg::Conflict(_)) {
                cidx.push(Some(k));
                k += 1;
            } else {
                cidx.push(None);
            }
        }
    }

    // the result block for one conflict
    let result_block = {
        let st = st.clone();
        let on = on.to_string();
        let tn = tn.to_string();
        let theirs_ref = theirs_ref.clone();
        let ours_ref = ours_ref.clone();
        move |n: usize, start: usize| -> Element {
            let c = st.cur().conflicts()[n].clone();
            let pk = st.pick(n).cloned();
            let resolved = st.is_resolved(st.file, n);
            let is_focus = n == st.focus;
            let ed = editing().filter(|e| e.0 == st.file && e.1 == n);
            let pv = preview().filter(|p| p.0 == n).map(|p| p.1);
            let sug = c.suggestion.clone().filter(|_| suggestions);
            let state = if resolved {
                "resolved"
            } else if pk.is_some() {
                "edited"
            } else {
                "open"
            };
            let lines: Vec<(String, &'static str)> = match &pk {
                Some(p) => pick_lines(&c, p)
                    .into_iter()
                    .map(|l| {
                        let k = if count_markers(&l) > 0 { "mk" } else { "pk" };
                        (l, k)
                    })
                    .collect(),
                None => {
                    // the marker block, each line tagged with its section so a hover preview
                    // can mark what stays without changing the block's height
                    let keep = |sec: &str| -> &'static str {
                        match &pv {
                            None => {
                                if sec == "m" {
                                    "mk"
                                } else {
                                    "sd"
                                }
                            }
                            Some(p) => {
                                let kept = matches!(
                                    (p, sec),
                                    (Pick::Ours, "ours")
                                        | (Pick::Theirs, "theirs")
                                        | (Pick::Both { .. }, "ours" | "theirs")
                                        | (Pick::Base, "base")
                                );
                                if kept { "keep" } else { "drop" }
                            }
                        }
                    };
                    let mut v = vec![(format!("<<<<<<< {ours_ref}"), keep("m"))];
                    v.extend(c.ours.iter().map(|l| (l.clone(), keep("ours"))));
                    if let Some(b) = &c.base {
                        v.push(("||||||| base".to_string(), keep("m")));
                        v.extend(b.iter().map(|l| (l.clone(), keep("base"))));
                    }
                    v.push(("=======".to_string(), keep("m")));
                    v.extend(c.theirs.iter().map(|l| (l.clone(), keep("theirs"))));
                    v.push((format!(">>>>>>> {theirs_ref}"), keep("m")));
                    v
                }
            };
            let pv_agent = matches!(pv, Some(Pick::Agent));
            let label = format!(
                "Conflict {} of {}, {}{}",
                n + 1,
                st.total(st.file),
                if resolved {
                    "resolved, took "
                } else if pk.is_some() {
                    "edited, markers left"
                } else {
                    "unresolved"
                },
                if resolved {
                    pk.as_ref().map_or("", |p| p.tag())
                } else {
                    ""
                }
            );
            let tag = pk.as_ref().map(|p| p.tag()).unwrap_or("");
            let has_base = c.base.is_some();
            let left = count_markers(&ed.as_ref().map(|e| e.2.clone()).unwrap_or_default());
            let conf = sug.as_ref().and_then(|s| s.confidence).unwrap_or("");
            let rationale = sug
                .as_ref()
                .map(|s| s.rationale.clone())
                .unwrap_or_default();
            let has_sug = sug.is_some();
            let on2 = on.clone();
            let tn2 = tn.clone();
            rsx! {
                div { class: "oxyn-mrg-blk", "data-state": state, "data-focus": "{is_focus}", "data-n": "{n}", role: "group", "aria-label": "{label}",
                    onclick: move |_| { let mut s = m.write(); if s.focus != n { s.focus_on(n); } },
                    div { class: "oxyn-mrg-bh",
                        span { class: "oxyn-mrg-bt", "Conflict {n + 1}" }
                        if resolved { span { class: "oxyn-mrg-tag", "data-t": "{tag}", Icon { name: "check", size: 11.0 } "{tag}" } }
                        else if pk.is_some() { span { class: "oxyn-mrg-tag", "data-t": "marker", "⚑ markers left" } }
                        else { span { class: "oxyn-mrg-tag", "data-t": "open", "unresolved" } }
                        span { class: "oxyn-mrg-sp" }
                        if pk.is_some() && !readonly && ed.is_none() {
                            button { class: "oxyn-mrg-lnk", r#type: "button", "aria-keyshortcuts": "Alt+R", onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Reopen); }, "Reopen" }
                        }
                    }
                    if let Some((_, _, text)) = ed.clone() {
                        div { class: "oxyn-mrg-edit",
                            textarea { class: "oxyn-mrg-ta", "aria-label": "Edit conflict {n + 1} by hand", rows: "{text.lines().count().max(2)}", value: "{text}",
                                onmounted: move |e| { spawn(async move { let _ = e.data().set_focus(true).await; }); },
                                oninput: move |e| { if let Some(x) = editing.write().as_mut() { x.2 = e.value(); } },
                                onkeydown: move |e: KeyboardEvent| {
                                    e.stop_propagation();
                                    let md = e.modifiers();
                                    if e.key() == Key::Escape { e.prevent_default(); editing.set(None); }
                                    if e.key() == Key::Enter && (md.meta() || md.ctrl()) {
                                        e.prevent_default();
                                        if let Some((ef, en, t)) = editing() {
                                            let lines: Vec<String> = t.lines().map(str::to_string).collect();
                                            let p = m.peek().files[ef].path.clone();
                                            if m.write().take(en, Pick::Manual(lines.clone()), false) { on_resolve.call((p, en, Some(Pick::Manual(lines)))); }
                                            editing.set(None);
                                            let a = m.peek().announce(); live.set(a);
                                        }
                                    }
                                },
                            }
                            div { class: "oxyn-mrg-bar",
                                span { class: "oxyn-mrg-mk", "data-zero": "{left == 0}", if left == 0 { "No markers left" } else { {plural(left, "marker", "⚑ ")} " left" } }
                                span { class: "oxyn-mrg-sp" }
                                button { class: "oxyn-mrg-btn q", r#type: "button", onclick: move |e| { e.stop_propagation(); editing.set(None); }, "Cancel" }
                                button { class: "oxyn-mrg-btn p", r#type: "button", "aria-keyshortcuts": "Control+Enter Meta+Enter",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        if let Some((ef, en, t)) = editing() {
                                            let lines: Vec<String> = t.lines().map(str::to_string).collect();
                                            let p = m.peek().files[ef].path.clone();
                                            if m.write().take(en, Pick::Manual(lines.clone()), false) { on_resolve.call((p, en, Some(Pick::Manual(lines)))); }
                                            editing.set(None);
                                            let a = m.peek().announce(); live.set(a);
                                        }
                                    },
                                    "Apply" span { class: "oxyn-mrg-kbd", "⌘↵" }
                                }
                            }
                        }
                    } else {
                        if inline && pk.is_none() {
                            div { class: "oxyn-mrg-mkline", "⚑ {lines.len()} lines with conflict markers stay in the file until you pick a side (lines {start}–{start + lines.len() - 1})" }
                        } else {
                            div { class: "oxyn-mrg-code", "data-preview": "{pv.is_some()}",
                                for (i, (l, k)) in lines.iter().enumerate() {
                                    {line_el(Some(start + i), l, k)}
                                }
                            }
                        }
                        if has_sug && !resolved {
                            div { class: "oxyn-mrg-sug", "data-pv": "{pv_agent}",
                                span { class: "oxyn-mrg-spark", "aria-hidden": "true", Icon { name: "spark", size: 12.0 } }
                                span { class: "oxyn-mrg-why", "{rationale}" }
                                if !conf.is_empty() { span { class: "oxyn-mrg-conf", "{conf} confidence" } }
                            }
                        }
                        if !resolved && !readonly {
                            div { class: "oxyn-mrg-acts", role: "toolbar", "aria-label": "Settle conflict {n + 1}",
                                for (k, txt, keys) in [(TakeKind::Ours, format!("Accept {}", on2.to_lowercase()), "Alt+1"), (TakeKind::Theirs, format!("Accept {}", tn2.to_lowercase()), "Alt+2"), (TakeKind::Both, "Accept both".to_string(), "Alt+3")] {
                                    button { class: "oxyn-mrg-btn", "data-k": "{keys}", r#type: "button", "aria-keyshortcuts": "{keys}",
                                        onmouseenter: move |_| preview.set(Some((n, k.pick()))),
                                        onfocus: move |_| preview.set(Some((n, k.pick()))),
                                        onmouseleave: move |_| preview.set(None),
                                        onblur: move |_| preview.set(None),
                                        onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Take(k)); },
                                        "{txt}"
                                    }
                                }
                                button { class: "oxyn-mrg-btn q", r#type: "button", title: "Both, {tn2.to_lowercase()} first",
                                    onmouseenter: move |_| preview.set(Some((n, Pick::Both { theirs_first: true }))),
                                    onmouseleave: move |_| preview.set(None),
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        let p = m.peek().cur().path.clone();
                                        let pick = Pick::Both { theirs_first: true };
                                        if m.write().take(n, pick.clone(), auto_advance) { on_resolve.call((p, n, Some(pick))); }
                                        preview.set(None);
                                        let a = m.peek().announce(); live.set(a);
                                    },
                                    "{tn2} first"
                                }
                                if has_base {
                                    button { class: "oxyn-mrg-btn q", r#type: "button", "aria-keyshortcuts": "Alt+4",
                                        onmouseenter: move |_| preview.set(Some((n, Pick::Base))),
                                        onmouseleave: move |_| preview.set(None),
                                        onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Take(TakeKind::Base)); },
                                        "Keep base"
                                    }
                                }
                                if has_sug {
                                    button { class: "oxyn-mrg-btn s", r#type: "button", "aria-keyshortcuts": "Alt+5",
                                        onmouseenter: move |_| preview.set(Some((n, Pick::Agent))),
                                        onmouseleave: move |_| preview.set(None),
                                        onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Take(TakeKind::Agent)); },
                                        Icon { name: "spark", size: 12.0 } "Accept suggestion"
                                    }
                                }
                                button { class: "oxyn-mrg-btn q", r#type: "button", "aria-keyshortcuts": "E",
                                    onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Edit); },
                                    Icon { name: "pencil", size: 12.0 } "Edit"
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    // an input-pane cell for one segment and side
    let input_cell = {
        let file = file.clone();
        let st = st.clone();
        let nums = nums.clone();
        let cidx = cidx.clone();
        move |si: usize, side: &'static str| -> Element {
            let (a, b, c, _) = nums[si];
            match &file.segs[si] {
                Seg::Same(l) => {
                    let start = match side {
                        "ours" => a,
                        "base" => b,
                        _ => c,
                    };
                    rsx! { div { class: "oxyn-mrg-cell", "data-side": side, {run(si, l.clone(), si == 0, si + 1 == nseg, start, "")} } }
                }
                Seg::Conflict(x) => {
                    let n = cidx[si].unwrap_or(0);
                    let lines: Vec<String> = match side {
                        "ours" => x.ours.clone(),
                        "base" => x.base.clone().unwrap_or_default(),
                        _ => x.theirs.clone(),
                    };
                    let start = match side {
                        "ours" => a,
                        "base" => b,
                        _ => c,
                    };
                    let resolved = st.is_resolved(st.file, n);
                    let pk = st.pick(n).map(|p| p.tag()).unwrap_or("");
                    let taken = resolved && (pk == side || pk == "both" && side != "base");
                    let is_focus = n == st.focus;
                    let take = match side {
                        "ours" => TakeKind::Ours,
                        "base" => TakeKind::Base,
                        _ => TakeKind::Theirs,
                    };
                    let empty = lines.is_empty();
                    let no_base = side == "base" && x.base.is_none();
                    rsx! {
                        div { class: "oxyn-mrg-cell oxyn-mrg-cf", "data-side": side, "data-focus": "{is_focus}", "data-resolved": "{resolved}", "data-taken": "{taken}",
                            onclick: move |_| m.write().focus_on(n),
                            div { class: "oxyn-mrg-cfh",
                                span { "Conflict {n + 1}" }
                                if resolved { span { class: "oxyn-mrg-cfs", "data-taken": "{taken}", if taken { "kept" } else { "dropped" } } }
                                span { class: "oxyn-mrg-sp" }
                                if !resolved && !readonly && !no_base {
                                    button { class: "oxyn-mrg-take", r#type: "button", "aria-label": "Take {side} for conflict {n + 1}",
                                        onclick: move |e| { e.stop_propagation(); m.write().focus_on(n); act(MergeCmd::Take(take)); },
                                        Icon { name: "check", size: 11.0 } "Take"
                                    }
                                }
                            }
                            if no_base { div { class: "oxyn-mrg-none", "No common ancestor" } }
                            else if empty { div { class: "oxyn-mrg-none", "(deleted)" } }
                            for (i, l) in lines.iter().enumerate() { {line_el(Some(start + i), l, "")} }
                        }
                    }
                }
            }
        }
    };

    let sides: Vec<&'static str> = if center {
        vec!["ours", "result", "theirs"]
    } else if base_on {
        vec!["ours", "base", "theirs"]
    } else {
        vec!["ours", "theirs"]
    };
    let ncol = sides.len();
    let cur_tab = if sides.contains(&tab().as_str()) {
        tab()
    } else if center {
        "result".to_string()
    } else {
        sides[0].to_string()
    };
    let pane_name = |s: &str| -> String {
        match s {
            "ours" => format!("{on} · {ours_ref}"),
            "theirs" => format!("{tn} · {theirs_ref}"),
            "base" => format!("Base · {base_ref}"),
            _ => "Result".to_string(),
        }
    };
    let heads: Vec<(String, String)> = sides
        .iter()
        .map(|s| (s.to_string(), pane_name(s)))
        .collect();
    let title = format!("Merging {theirs_ref} into {ours_ref}");

    rsx! {
        div { class: "oxyn-mrg", "data-layout": "{layout}", "data-tab": "{cur_tab}", "data-phase": match phase { Phase::Open => "open", Phase::ConfirmAbort => "confirm", Phase::Completed => "done", Phase::Aborted => "aborted" },
            role: "region", "aria-label": "Merge conflicts: {theirs_ref} into {ours_ref}", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                if editing.peek().is_some() { return; }
                let md = e.modifiers();
                let k = e.key().to_string();
                let code = e.code().to_string();
                if let Some(c) = merge_key(&k, &code, md.alt(), md.meta() || md.ctrl(), md.shift()) {
                    e.prevent_default();
                    act(c);
                }
            },
            div { class: "oxyn-mrg-hd",
                div { class: "oxyn-mrg-ttl",
                    span { class: "oxyn-mrg-ic", Icon { name: "merge", size: 15.0 } }
                    span { class: "oxyn-mrg-tt", "{title}" }
                }
                div { class: "oxyn-mrg-prog",
                    span { class: "oxyn-mrg-cnt", "{done_all} of {total_all} resolved" }
                    span { class: "oxyn-mrg-pb", role: "progressbar", "aria-label": "Conflicts resolved", "aria-valuemin": "0", "aria-valuemax": "{total_all}", "aria-valuenow": "{done_all}",
                        span { style: "width:{pct}%" }
                    }
                }
                span { class: "oxyn-mrg-sp" }
                if phase == Phase::Open || phase == Phase::ConfirmAbort {
                    div { class: "oxyn-mrg-nav",
                        button { class: "oxyn-mrg-ib", r#type: "button", "aria-label": "Previous conflict", "aria-keyshortcuts": "Alt+ArrowUp [", title: "Previous (Alt+↑)", disabled: total == 0, onclick: move |_| act(MergeCmd::Prev), Icon { name: "chevron-left", size: 14.0 } }
                        span { class: "oxyn-mrg-pos", if total == 0 { "No conflicts" } else if done == total { "All {total} resolved" } else { "Conflict {focus + 1} of {total}" } }
                        button { class: "oxyn-mrg-ib", r#type: "button", "aria-label": "Next conflict", "aria-keyshortcuts": "Alt+ArrowDown ]", title: "Next (Alt+↓)", disabled: total == 0, onclick: move |_| act(MergeCmd::Next), Icon { name: "chevron-right", size: 14.0 } }
                        button { class: "oxyn-mrg-ib", r#type: "button", "aria-label": "Undo", "aria-keyshortcuts": "Control+Z Meta+Z", title: "Undo (⌘Z)", onclick: move |_| act(MergeCmd::Undo), Icon { name: "undo", size: 13.0 } }
                    }
                }
            }
            if multi {
                div { class: "oxyn-mrg-files", role: "tablist", "aria-label": "Conflicted files",
                    for (i, mf) in st.files.iter().enumerate() {
                        button { class: "oxyn-mrg-file", r#type: "button", role: "tab", "aria-selected": "{i == f}", "data-i": "{i}",
                            onclick: move |_| { m.write().open_file(i); editing.set(None); },
                            span { class: "oxyn-mrg-fn", "{mf.path}" }
                            if st.marked.contains(&i) { span { class: "oxyn-mrg-fc", "data-done": "true", Icon { name: "check", size: 11.0 } "staged" } }
                            else { span { class: "oxyn-mrg-fc", "{st.resolved(i)}/{st.total(i)}" } }
                        }
                    }
                }
            }
            match phase {
                Phase::Completed => rsx! {
                    div { class: "oxyn-mrg-end", "data-k": "done", role: "status",
                        span { class: "oxyn-mrg-endic", Icon { name: "check-circle", size: 18.0 } }
                        div { div { class: "oxyn-mrg-endt", "Merged {theirs_ref} into {ours_ref}" } div { class: "oxyn-mrg-ends", "{total_all} conflicts resolved in " {plural(st.files.len(), "file", "")} "." } }
                    }
                },
                Phase::Aborted => rsx! {
                    div { class: "oxyn-mrg-end", "data-k": "aborted", role: "status",
                        span { class: "oxyn-mrg-endic", Icon { name: "x-circle", size: 18.0 } }
                        div { div { class: "oxyn-mrg-endt", "Merge aborted" } div { class: "oxyn-mrg-ends", "Files restored to {ours_ref}. Nothing was committed." } }
                    }
                },
                _ => rsx! {
                    div { class: "oxyn-mrg-path",
                        span { class: "oxyn-mrg-pf", "{path}" }
                        span { class: "oxyn-mrg-pc", "{done} of {total} in this file" }
                        if marked { span { class: "oxyn-mrg-tag", "data-t": "staged", Icon { name: "check", size: 11.0 } "Marked resolved" } }
                    }
                    if !inline {
                        div { class: "oxyn-mrg-tabs", role: "tablist", "aria-label": "Pane",
                            for (s, nm) in heads.clone() {
                                button { r#type: "button", role: "tab", "aria-selected": "{s == cur_tab}", "data-s": "{s}", onclick: move |_| tab.set(s.clone()), "{nm}" }
                            }
                        }
                        div { class: "oxyn-mrg-in", style: "--n:{ncol}",
                            for (s, nm) in heads.clone() {
                                div { class: "oxyn-mrg-ph", "data-side": "{s}", role: "presentation",
                                    span { class: "oxyn-mrg-dot", "data-side": "{s}" }
                                    span { class: "oxyn-mrg-phn", "{nm}" }
                                }
                            }
                            for si in 0..nseg {
                                for s in sides.clone() {
                                    if s == "result" {
                                        if let Some(n) = cidx[si] {
                                            div { class: "oxyn-mrg-cell", "data-side": "result", {result_block(n, nums[si].3)} }
                                        } else if let Seg::Same(l) = &file.segs[si] {
                                            div { class: "oxyn-mrg-cell", "data-side": "result", {run(si, l.clone(), si == 0, si + 1 == nseg, nums[si].3, "")} }
                                        }
                                    } else {
                                        {input_cell(si, s)}
                                    }
                                }
                            }
                        }
                    }
                    if three || inline {
                        div { class: "oxyn-mrg-res", role: "group", "aria-label": "Result, {path2}",
                            div { class: "oxyn-mrg-rh",
                                span { class: "oxyn-mrg-dot", "data-side": "result" }
                                span { class: "oxyn-mrg-phn", "Result" }
                                span { class: "oxyn-mrg-sp" }
                                span { class: "oxyn-mrg-mk", "data-zero": "{markers == 0}", if markers == 0 { "No markers left" } else { {plural(markers, "marker", "⚑ ")} " left" } }
                            }
                            for si in 0..nseg {
                                match &file.segs[si] {
                                    Seg::Same(l) => rsx! { {run(si, l.clone(), si == 0, si + 1 == nseg, nums[si].3, "")} },
                                    Seg::Conflict(x) => {
                                        let n = cidx[si].unwrap_or(0);
                                        let open = !st.is_resolved(st.file, n);
                                        let pvn = preview().filter(|p| p.0 == n).map(|p| p.1);
                                        let pv_of = |side: &str| -> &'static str {
                                            match &pvn {
                                                None => "",
                                                Some(p) => {
                                                    if matches!((p, side), (Pick::Ours, "ours") | (Pick::Theirs, "theirs") | (Pick::Both { .. }, "ours" | "theirs") | (Pick::Base, "base")) { "keep" } else { "drop" }
                                                }
                                            }
                                        };
                                        let (pvo, pvb, pvt) = (pv_of("ours"), pv_of("base"), pv_of("theirs"));
                                        rsx! {
                                            if inline && open {
                                                div { class: "oxyn-mrg-stack",
                                                    div { class: "oxyn-mrg-sb", "data-side": "ours", "data-pv": pvo, div { class: "oxyn-mrg-sbh", span { class: "oxyn-mrg-dot", "data-side": "ours" } "{on} · {ours_ref}" } for (i, l) in x.ours.iter().enumerate() { {line_el(Some(nums[si].0 + i), l, "")} } }
                                                    if base_on && x.base.is_some() {
                                                        div { class: "oxyn-mrg-sb", "data-side": "base", "data-pv": pvb, div { class: "oxyn-mrg-sbh", span { class: "oxyn-mrg-dot", "data-side": "base" } "Base · {base_ref}" } for (i, l) in x.base.clone().unwrap_or_default().iter().enumerate() { {line_el(Some(nums[si].1 + i), l, "")} } }
                                                    }
                                                    div { class: "oxyn-mrg-sb", "data-side": "theirs", "data-pv": pvt, div { class: "oxyn-mrg-sbh", span { class: "oxyn-mrg-dot", "data-side": "theirs" } "{tn} · {theirs_ref}" } for (i, l) in x.theirs.iter().enumerate() { {line_el(Some(nums[si].2 + i), l, "")} } }
                                                }
                                            }
                                            {result_block(n, nums[si].3)}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if phase == Phase::Open || phase == Phase::ConfirmAbort {
                div { class: "oxyn-mrg-ft",
                    if phase == Phase::ConfirmAbort {
                        div { class: "oxyn-mrg-confirm", role: "alertdialog", "aria-label": "Abort merge?",
                            span { "Abort the merge? " {plural(done_all, "resolution", "")} " will be lost." }
                            button { class: "oxyn-mrg-btn q", r#type: "button", onclick: move |_| m.write().abort("cancel"), "Keep resolving" }
                            button { class: "oxyn-mrg-btn d", r#type: "button", onclick: move |_| { m.write().abort("confirm"); on_abort.call(()); }, "Abort merge" }
                        }
                    } else {
                        button { class: "oxyn-mrg-btn q", r#type: "button", onclick: move |_| m.write().abort("ask"), "Abort" }
                        span { class: "oxyn-mrg-sp" }
                        if let Some(why) = mark_why.clone().filter(|_| !marked) { span { class: "oxyn-mrg-why2", "{why}" } }
                        if !marked {
                            button { class: if multi { "oxyn-mrg-btn" } else { "oxyn-mrg-btn p" }, r#type: "button", disabled: mark_why.is_some(), "aria-keyshortcuts": "Control+Shift+Enter Meta+Shift+Enter",
                                onclick: move |_| act(MergeCmd::Mark),
                                Icon { name: "check", size: 12.0 } "Mark resolved"
                            }
                        }
                        if multi || marked {
                            if let Some(why) = complete_why.clone().filter(|_| marked || mark_why.is_none()) { span { class: "oxyn-mrg-why2", "{why}" } }
                            button { class: "oxyn-mrg-btn p", r#type: "button", disabled: complete_why.is_some(),
                                onclick: move |_| { if m.write().complete() { on_complete.call(()); live.set("Merge complete".to_string()); } },
                                "Complete merge"
                            }
                        }
                    }
                }
            }
            div { class: "oxyn-mrg-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Conflict Radar
// ---------------------------------------------------------------------------------------------

/// One radar cell: agent, label, bar (left %, width %, active, from, to), kept.
type RadarCell = (
    RadarAgent,
    String,
    Option<(f32, f32, bool, u32, u32)>,
    Option<bool>,
);

const MARKS: [&str; 8] = [
    "blue", "violet", "cyan", "pink", "green", "orange", "yellow", "red",
];

/// Agents working on their own branches at once and where they get in each other's way: a row
/// per file, a column per agent, a line between agents that changed the same lines, each agent's
/// way in one sentence to pick from, and Merge once every conflict is settled and every agent is
/// done. Oxyn plus: overlap heat, a suggested merge order and a conflicts-only filter.
#[component]
pub fn ConflictRadar(
    /// Columns: one agent each.
    agents: Vec<RadarAgent>,
    /// Rows: one file each, with every agent's edit.
    files: Vec<RadarFile>,
    /// Heading in the bar.
    #[props(default = "Parallel work".to_string())]
    title: String,
    /// Controlled picks (path → agent id); `None` keeps them inside.
    #[props(default)]
    resolved: Option<BTreeMap<String, String>>,
    /// Tint conflicting cells by how many lines overlap.
    #[props(default)]
    heat: bool,
    /// Show the suggested merge order.
    #[props(default)]
    order: bool,
    /// Only rows that conflict.
    #[props(default)]
    conflicts_only: bool,
    /// Open the first unsettled conflict's ways on mount.
    #[props(default)]
    default_open: bool,
    /// A way was picked: (path, agent id).
    #[props(default)]
    on_resolve: EventHandler<(String, String)>,
    /// Merge was pressed.
    #[props(default)]
    on_merge: EventHandler<()>,
) -> Element {
    let mut picks = use_signal(BTreeMap::<String, String>::new);
    let mut merged = use_signal(|| false);
    let mut info = use_signal(String::new);
    let mut r = Radar::new(agents.clone(), files.clone());
    r.resolved = resolved.clone().unwrap_or_else(|| picks.cloned());
    let first_open = r.open().first().map(|f| f.path.clone());
    let mut expanded = use_signal(move || if default_open { first_open } else { None });
    let n = r.agents.len().max(1);
    let max_line = r
        .files
        .iter()
        .flat_map(|f| f.edits.iter().map(|e| e.to))
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let conflicts = r.conflicts().len();
    let open = r.open().len();
    let blocker = r.merge_blocker();
    let shown: Vec<RadarFile> = r
        .files
        .iter()
        .filter(|f| !conflicts_only || f.conflicting())
        .cloned()
        .collect();
    let order_names: Vec<String> = r
        .merge_order()
        .iter()
        .filter_map(|id| {
            r.agents
                .iter()
                .find(|a| &a.id == id)
                .map(|a| a.name.clone())
        })
        .collect();
    let mark_of = |id: &str| -> &'static str {
        r.agents
            .iter()
            .position(|a| a.id == id)
            .map_or("blue", |i| MARKS[i % MARKS.len()])
    };
    let summary = if conflicts == 0 {
        format!("{} files · no conflicts", r.files.len())
    } else {
        format!(
            "{} files · {} conflict{} · {} settled",
            r.files.len(),
            conflicts,
            if conflicts == 1 { "" } else { "s" },
            conflicts - open
        )
    };
    let is_merged = merged();
    let controlled = resolved.is_some();
    rsx! {
        div { class: "oxyn-crd", style: "--n:{n}", "data-heat": "{heat}", "data-merged": "{is_merged}",
            div { class: "oxyn-crd-bar",
                span { class: "oxyn-crd-ic", Icon { name: "branch", size: 14.0 } }
                span { class: "oxyn-crd-tt", "{title}" }
                span { class: "oxyn-crd-sum", "{summary}" }
                span { class: "oxyn-crd-sp" }
                if is_merged {
                    span { class: "oxyn-crd-merged", role: "status", Icon { name: "check", size: 12.0 } "Merged" }
                } else {
                    if let Some(b) = blocker.clone() { span { class: "oxyn-crd-why", "{b}" } }
                    button { class: "oxyn-crd-merge", r#type: "button", disabled: blocker.is_some(),
                        onclick: move |_| { merged.set(true); on_merge.call(()); info.set("Merged every branch".to_string()); },
                        Icon { name: "merge", size: 13.0 } "Merge"
                    }
                }
            }
            div { class: "oxyn-crd-tbl", role: "table", "aria-label": "{title}: files by agent",
                div { class: "oxyn-crd-row oxyn-crd-head", role: "row",
                    div { class: "oxyn-crd-fh", role: "columnheader", "File" }
                    for a in r.agents.clone() {
                        {
                            let nb = r.numbers(&a.id);
                            let txt = format!("{}: {} file{} · {} open conflict{} · {} kept", a.name, nb.files, if nb.files == 1 { "" } else { "s" }, nb.open, if nb.open == 1 { "" } else { "s" }, nb.kept);
                            let t2 = txt.clone();
                            let mk = mark_of(&a.id);
                            let run_word = if a.running { "working" } else { "done" };
                            rsx! {
                                div { class: "oxyn-crd-ah", role: "columnheader", tabindex: "0", "data-run": "{a.running}", "aria-label": "{a.name}, {a.model}, {run_word}",
                                    onmouseenter: move |_| info.set(txt.clone()), onfocus: move |_| info.set(t2.clone()),
                                    span { class: "oxyn-crd-mark", "data-c": mk, "{a.monogram()}" }
                                    span { class: "oxyn-crd-an",
                                        span { class: "oxyn-crd-nm", "{a.name}" }
                                        span { class: "oxyn-crd-md", crate::brand::MarkFor { name: a.model.clone(), size: 11.0 } "{a.model}" }
                                    }
                                    if a.running { span { class: "oxyn-crd-live", "aria-hidden": "true" } }
                                }
                            }
                        }
                    }
                }
                for file in shown.clone() {
                    {
                        let conflicting = file.conflicting();
                        let (dir, name) = file.split();
                        let pick = r.resolved.get(&file.path).cloned();
                        let pick_name = pick.as_ref().and_then(|p| r.agents.iter().find(|a| &a.id == p)).map(|a| a.name.clone());
                        let is_open = expanded().as_deref() == Some(file.path.as_str());
                        let clash = file.clash_agents();
                        let cols: Vec<usize> = clash.iter().filter_map(|id| r.agents.iter().position(|a| &a.id == id)).collect();
                        let (lo, hi) = (cols.iter().copied().min().unwrap_or(0), cols.iter().copied().max().unwrap_or(0));
                        let heat_v = (file.overlap_lines() as f32 / 20.0).clamp(0.15, 1.0);
                        let p1 = file.path.clone();
                        let p2 = file.path.clone();
                        let state = if !conflicting { "" } else if pick.is_some() { "settled" } else { "conflict" };
                        let ways: Vec<(RadarAgent, String)> = clash.iter().filter_map(|id| {
                            let a = r.agents.iter().find(|a| &a.id == id)?.clone();
                            let w = file.edit(id).and_then(|e| e.summary.clone()).unwrap_or_else(|| "Keep this agent's version".to_string());
                            Some((a, w))
                        }).collect();
                        let nways = ways.len();
                        let cells: Vec<RadarCell> = r.agents.iter().map(|a| {
                            let e = file.edit(&a.id).map(|e| ((e.from - 1) as f32 / max_line * 100.0, (e.len() as f32 / max_line * 100.0).max(3.0), e.active, e.from, e.to));
                            (a.clone(), r.cell_label(&file.path, a), e, r.kept(&file.path, &a.id))
                        }).collect();
                        rsx! {
                            div { class: "oxyn-crd-row", role: "row", "data-state": state, "data-path": "{file.path}",
                                style: "--lo:{lo};--hi:{hi};--heat:{heat_v}",
                                if conflicting {
                                    div { class: "oxyn-crd-file", role: "rowheader", tabindex: "0", "aria-expanded": "{is_open}",
                                        "aria-label": "{file.path}, {state}",
                                        onclick: move |_| { let cur = expanded(); expanded.set(if cur.as_deref() == Some(p1.as_str()) { None } else { Some(p1.clone()) }); },
                                        onkeydown: move |e: KeyboardEvent| {
                                            if e.key() == Key::Enter || e.key().to_string() == " " {
                                                e.prevent_default();
                                                let cur = expanded();
                                                expanded.set(if cur.as_deref() == Some(p2.as_str()) { None } else { Some(p2.clone()) });
                                            }
                                        },
                                        span { class: "oxyn-crd-dir", "{dir}" }
                                        span { class: "oxyn-crd-name", "{name}" }
                                        if let Some(pn) = pick_name.clone() { span { class: "oxyn-crd-st", "data-s": "settled", Icon { name: "check", size: 10.0 } "{pn}" } }
                                        else { span { class: "oxyn-crd-st", "data-s": "conflict", "Conflict" } }
                                        span { class: "oxyn-crd-chev", "data-open": "{is_open}", Icon { name: "chevron", size: 12.0 } }
                                    }
                                } else {
                                    div { class: "oxyn-crd-file", role: "rowheader",
                                        span { class: "oxyn-crd-dir", "{dir}" }
                                        span { class: "oxyn-crd-name", "{name}" }
                                    }
                                }
                                for (a, label, e, kept) in cells.clone() {
                                    {
                                        let lb2 = label.clone();
                                        let lb3 = label.clone();
                                        let mk = mark_of(&a.id);
                                        let in_clash = clash.contains(&a.id);
                                        rsx! {
                                            div { class: "oxyn-crd-cell", role: "cell", "aria-label": "{label}", "data-clash": "{in_clash}",
                                                "data-kept": match kept { Some(true) => "true", Some(false) => "false", None => "open" },
                                                if let Some((left, w, active, from, to)) = e {
                                                    span { class: "oxyn-crd-bar-i", "data-c": mk, "data-active": "{active}", tabindex: "-1", title: "{a.name}: lines {from}–{to}",
                                                        style: "--l:{left / 100.0};--w:{w / 100.0}",
                                                        onmouseenter: move |_| info.set(lb2.clone()),
                                                        onclick: move |_| info.set(lb3.clone()),
                                                    }
                                                } else {
                                                    span { class: "oxyn-crd-none", "aria-hidden": "true" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if conflicting && cols.len() > 1 {
                                    span { class: "oxyn-crd-link", "data-settled": "{pick.is_some()}", "aria-hidden": "true" }
                                }
                            }
                            if is_open && conflicting {
                                div { class: "oxyn-crd-ways", role: "radiogroup", "aria-label": "Ways to settle {file.path}",
                                    for (i, (a, w)) in ways.into_iter().enumerate() {
                                        {
                                            let checked = pick.as_deref() == Some(a.id.as_str());
                                            let tab_ix = if checked || (pick.is_none() && i == 0) { "0" } else { "-1" };
                                            let mk = mark_of(&a.id);
                                            let path = file.path.clone();
                                            let path2 = file.path.clone();
                                            let id1 = a.id.clone();
                                            let id2 = a.id.clone();
                                            let all_ids: Vec<String> = clash.clone();
                                            let cur_ix = pick.as_ref().and_then(|p| clash.iter().position(|x| x == p)).unwrap_or(i);
                                            let an = a.name.clone();
                                            rsx! {
                                                div { class: "oxyn-crd-way", role: "radio", "aria-checked": "{checked}", tabindex: "{tab_ix}", "data-i": "{i}",
                                                    onclick: move |_| {
                                                        if !controlled { picks.write().insert(path.clone(), id1.clone()); }
                                                        on_resolve.call((path.clone(), id1.clone()));
                                                        info.set(format!("Kept {an}'s way for {path}"));
                                                    },
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        let k = e.key();
                                                        if k == Key::Enter || k.to_string() == " " {
                                                            e.prevent_default();
                                                            if !controlled { picks.write().insert(path2.clone(), id2.clone()); }
                                                            on_resolve.call((path2.clone(), id2.clone()));
                                                        } else if matches!(k, Key::ArrowDown | Key::ArrowRight | Key::ArrowUp | Key::ArrowLeft) {
                                                            e.prevent_default();
                                                            let fwd = matches!(k, Key::ArrowDown | Key::ArrowRight);
                                                            let j = if fwd { (cur_ix + 1) % nways } else { (cur_ix + nways - 1) % nways };
                                                            if !controlled { picks.write().insert(path2.clone(), all_ids[j].clone()); }
                                                            on_resolve.call((path2.clone(), all_ids[j].clone()));
                                                        }
                                                    },
                                                    span { class: "oxyn-crd-radio", "aria-hidden": "true" }
                                                    span { class: "oxyn-crd-mark", "data-c": mk, "{a.monogram()}" }
                                                    span { class: "oxyn-crd-wn", "{a.name}" }
                                                    span { class: "oxyn-crd-ww", "{w}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if shown.is_empty() {
                    div { class: "oxyn-crd-empty", role: "row", div { role: "cell", "No conflicts: every agent changed different lines." } }
                }
            }
            if order && !order_names.is_empty() {
                div { class: "oxyn-crd-order",
                    span { class: "oxyn-crd-ol", "Suggested merge order" }
                    for (i, nm) in order_names.iter().enumerate() {
                        if i > 0 { span { class: "oxyn-crd-arrow", "aria-hidden": "true", "→" } }
                        span { class: "oxyn-crd-on", "{nm}" }
                    }
                }
            }
            div { class: "oxyn-crd-info", role: "status", "aria-live": "polite", if info().is_empty() { "Point at a bar or an agent for details." } else { "{info}" } }
        }
    }
}
