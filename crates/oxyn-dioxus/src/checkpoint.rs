//! Checkpoints (`checkpoint-timeline.md`): `CheckpointLanes` (attempts as lanes, like git for
//! agents), `CheckpointMarker` (inline marker and hover-revert), `CheckpointList` (side list and
//! terminal), `RestoreToast` (undo window), and the C1a variants: `CheckpointTree` (forks as an
//! indented tree), `CheckpointFilmStrip` (a thumbnail per checkpoint on a scrubbable track),
//! `UndoLadder` (a vertical stepper of reversible steps) and `RestorePreviewSheet` (the diff
//! summary with Restore, a confirm step and Cancel). Keys follow `oxyn_core::checkpoint::CpKey`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    checkpoint::{
        Checkpoint, CheckpointKind, CheckpointStatus, CpKey, FilmStrip, PreviewFile, PreviewPhase,
        RestoreAction, RestoreFlow, RestorePreview, UndoLadder as LadderModel, UndoStep,
        branch_tree, format_at, format_cost, header, lanes, roving, step_cost, thumb_bars,
    },
};

type Els = Signal<Vec<Option<Rc<MountedData>>>>;

fn keep(mut els: Els, i: usize, e: MountedEvent) {
    let mut v = els.write();
    if v.len() <= i {
        v.resize(i + 1, None);
    }
    v[i] = Some(e.data());
}

fn focus_at(els: Els, i: usize) {
    if let Some(Some(d)) = els.peek().get(i).cloned() {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn cp_key(e: &KeyboardEvent) -> Option<CpKey> {
    let m = e.modifiers();
    let k = e.key().to_string();
    if m.meta() || m.ctrl() {
        return match k.to_ascii_lowercase().as_str() {
            "z" if m.shift() => Some(CpKey::Redo),
            "z" => Some(CpKey::Undo),
            "y" => Some(CpKey::Redo),
            _ => None,
        };
    }
    CpKey::from_name(&k)
}

fn label_at(times: &[String], c: &Checkpoint, i: usize) -> String {
    times.get(i).cloned().unwrap_or_else(|| format_at(c.at))
}

use crate::composer::icons::Icon;

fn status_cls(s: CheckpointStatus) -> &'static str {
    match s {
        CheckpointStatus::Running => "running",
        CheckpointStatus::Done => "done",
        CheckpointStatus::Failed => "failed",
        CheckpointStatus::Stopped => "stopped",
    }
}

/// Run checkpoints as lanes: rewinding branches a new attempt; the old one fades but stays.
/// One tab stop: ↑ / ↓ / Home / End move between rows, Enter or Space rewinds to the focused row.
#[component]
pub fn CheckpointLanes(
    /// Checkpoints in the order taken.
    checkpoints: Vec<Checkpoint>,
    /// Where the agent stands (default: last).
    #[props(default)]
    head: Option<String>,
    /// What the run is for.
    #[props(default)]
    task: Option<String>,
    /// Heading of the bar.
    #[props(default = "Checkpoints".to_string())]
    title: String,
    /// Time labels, same order (absolute ISO, `2026-11-14 15:31:09`); default the run time
    /// (`1:05`).
    #[props(default)]
    times: Vec<String>,
    /// How many rows show before the rows scroll within the card.
    #[props(default = 14)]
    max_rows: usize,
    /// Rewind pressed (button, Enter or Space): the checkpoint id.
    #[props(default)]
    on_rewind: EventHandler<String>,
    /// Show the tool that made each checkpoint as an icon before its title.
    #[props(default = true)]
    tool_icons: bool,
    /// Show what each step cost (`+$0.11`) beside its time.
    #[props(default = true)]
    step_costs: bool,
    /// Cost formatter, cents in (default `format_cost`: `$0.42`, `$1.2k`).
    #[props(default)]
    format: Option<Callback<u32, String>>,
) -> Element {
    let rows = lanes(&checkpoints, head.as_deref());
    let head_i = rows.iter().position(|r| r.current).unwrap_or(0);
    let mut focus = use_signal(|| head_i);
    let els: Els = use_signal(Vec::new);
    let n = checkpoints.len();
    let fmt = move |c: u32| match format {
        Some(f) => f.call(c),
        None => format_cost(c),
    };
    let head_txt = match checkpoints.iter().filter_map(|c| c.cost_cents).max() {
        Some(c) => {
            let base = header(&checkpoints);
            let cut = base.rfind(" · ").unwrap_or(base.len());
            format!("{} · {}", &base[..cut], fmt(c))
        }
        None => header(&checkpoints),
    };
    let n_lanes = rows.iter().map(|r| r.lane).max().unwrap_or(0) + 1;
    let gutter = 18 * n_lanes + 8;
    let scroll = if n > max_rows {
        format!("max-height:{}px;overflow-y:auto", max_rows * 36 + 12)
    } else {
        String::new()
    };
    let f = focus().min(n.saturating_sub(1));
    let ids: Vec<(String, bool)> = rows
        .iter()
        .map(|r| {
            let c = &checkpoints[r.index];
            (
                c.id.clone(),
                !r.current && c.status != CheckpointStatus::Running,
            )
        })
        .collect();
    rsx! {
        div { class: "oxyn-cp lanes",
            div { class: "bar",
                span { class: "t", "{title}" }
                if let Some(t) = task { span { class: "task", "· {t}" } }
                span { class: "sp" }
                span { class: "n", "aria-live": "polite", "{head_txt}" }
            }
            ol { class: "rows oxyn-cp-scroll", style: "--g:{gutter}px;{scroll}", "aria-label": "{title}",
                onkeydown: move |e: KeyboardEvent| {
                    let Some(k) = cp_key(&e) else { return };
                    e.prevent_default();
                    if k == CpKey::Enter {
                        if let Some((id, ok)) = ids.get(f) && *ok { on_rewind.call(id.clone()) }
                        return;
                    }
                    let to = roving(n, f, k);
                    focus.set(to);
                    focus_at(els, to);
                },
                for r in rows {
                    {
                        let c = checkpoints[r.index].clone();
                        let id = c.id.clone();
                        let i = r.index;
                        let st = status_cls(c.status);
                        let cls = format!("row {st}{}{}{}", if r.on_path { " path" } else { " off" }, if r.current { " cur" } else { "" }, if i == f { " foc" } else { "" });
                        let time = label_at(&times, &c, i);
                        let label = format!("{}, {time}", r.aria);
                        rsx! {
                            li { class: "{cls}", "aria-current": if r.current { "step" } else { "false" }, "aria-label": "{label}", tabindex: if i == f { "0" } else { "-1" },
                                onmounted: move |e| keep(els, i, e), onfocus: move |_| focus.set(i),
                                svg { class: "lane", width: "{gutter}", height: "36", view_box: "0 0 {gutter} 36", style: "--d:{r.index * 70}ms", "aria-hidden": "true",
                                    for (l, on) in r.through.iter().copied() {
                                        line { x1: "{9 + l * 18}", y1: "0", x2: "{9 + l * 18}", y2: "36", "pathLength": "1", class: if on { "ln dr on" } else { "ln dr" } }
                                    }
                                    if let Some(on) = r.up {
                                        line { x1: "{9 + r.lane * 18}", y1: "0", x2: "{9 + r.lane * 18}", y2: "18", "pathLength": "1", class: if on { "ln dr on" } else { "ln dr" } }
                                    }
                                    if let Some(on) = r.down {
                                        line { x1: "{9 + r.lane * 18}", y1: "18", x2: "{9 + r.lane * 18}", y2: "36", "pathLength": "1", class: if on { "ln dr on" } else { "ln dr" } }
                                    }
                                    for (l, on) in r.forks.iter().copied() {
                                        path { d: "M{9 + r.lane * 18} 18 C {9 + r.lane * 18} 31, {9 + l * 18} 23, {9 + l * 18} 36", "pathLength": "1", class: if on { "ln dr on" } else { "ln dr" }, fill: "none" }
                                    }
                                    circle { cx: "{9 + r.lane * 18}", cy: "18", r: if r.current { "5" } else { "4" }, class: "dot {st}" }
                                }
                                if tool_icons {
                                    if let Some(ic) = c.tool_icon() {
                                        span { class: "ti", title: "{c.tool.clone().unwrap_or_default()}", "aria-hidden": "true", Icon { name: ic, size: 12.0 } }
                                    } else {
                                        span { class: "ti none", "aria-hidden": "true" }
                                    }
                                }
                                span { class: if c.status == CheckpointStatus::Running { "tt shim" } else { "tt" }, "{c.title}" }
                                if c.status == CheckpointStatus::Failed { span { class: "badge failed", "Failed" } }
                                if c.status == CheckpointStatus::Stopped { span { class: "badge", "Stopped" } }
                                if r.lane > 0 && r.branch_from.is_some() { span { class: "badge att", "Attempt {r.lane + 1}" } }
                                if r.current { span { class: "badge here", "Head" } }
                                span { class: "sp" }
                                span { class: "meta",
                                    if c.files == 1 { span { class: "files", "1 file" } } else if c.files > 1 { span { class: "files", "{c.files} files" } }
                                    if step_costs {
                                        if let Some(sc) = step_cost(&checkpoints, r.index) { span { class: "cost", "+{fmt(sc)}" } }
                                    }
                                    span { class: "du", "{time}" }
                                }
                                if !r.current && c.status != CheckpointStatus::Running {
                                    button { class: "rw", r#type: "button", tabindex: "-1", "aria-hidden": "true", onclick: move |_| on_rewind.call(id.clone()), Icon { name: "retry", size: 12.0 } "Rewind" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A checkpoint between turns (`inline-marker`) or on a user message (`hover-revert`).
/// Restore opens the menu; in it R / C / B / F pick an action, ↑ / ↓ move, Esc closes.
#[component]
pub fn CheckpointMarker(
    /// The checkpoint.
    checkpoint: Checkpoint,
    /// Time label (absolute ISO, `2026-11-14 15:42`).
    time: String,
    /// `inline-marker` or `hover-revert`.
    #[props(default = "inline-marker".to_string())]
    variant: String,
    /// Actions shown at first (as if hovered or focused); hover and focus reveal them too.
    #[props(default)]
    revealed: bool,
    /// Menu open at first.
    #[props(default)]
    menu: bool,
    /// The agent is running (restore blocked).
    #[props(default)]
    blocked: bool,
    /// Current checkpoint ("You are here").
    #[props(default)]
    current: bool,
    /// Action chosen.
    #[props(default)]
    on_action: EventHandler<RestoreAction>,
) -> Element {
    let c = checkpoint;
    let mut open = use_signal(|| menu);
    let mut hot = use_signal(|| revealed);
    let mut item = use_signal(|| 0usize);
    let els: Els = use_signal(Vec::new);
    let glyph = if c.chat_only { "◇" } else { "◆" };
    let what = if c.chat_only {
        "no file changes".to_string()
    } else {
        format!(
            "{} {}",
            c.files,
            if c.files == 1 { "file" } else { "files" }
        )
    };
    let kind = match c.kind {
        CheckpointKind::Manual => " · saved",
        CheckpointKind::Milestone => " · milestone",
        CheckpointKind::Auto => "",
    };
    let offered: Vec<(RestoreAction, char, bool)> = RestoreAction::ALL
        .iter()
        .map(|&(a, k)| {
            (
                a,
                k,
                a.offered(&c) && !(blocked && a != RestoreAction::Fork),
            )
        })
        .collect();
    let off2 = offered.clone();
    let mut pick = move |a: RestoreAction| {
        open.set(false);
        on_action.call(a);
    };
    let onkey = move |e: KeyboardEvent| {
        if !open() {
            return;
        }
        let k = e.key().to_string();
        if k == "Escape" {
            e.prevent_default();
            open.set(false);
            return;
        }
        if let Some(&(a, _, ok)) = off2
            .iter()
            .find(|(_, ch, _)| k.eq_ignore_ascii_case(&ch.to_string()))
        {
            e.prevent_default();
            if ok {
                pick(a);
            }
            return;
        }
        if let Some(ck) = CpKey::from_name(&k).filter(|k| !matches!(k, CpKey::Enter)) {
            e.prevent_default();
            let to = roving(4, item(), ck);
            item.set(to);
            focus_at(els, to);
        }
    };
    let menu_el = rsx! {
        if open() {
            div { class: "menu", role: "menu", "aria-label": "Restore checkpoint",
                for (i, (a, key, ok)) in offered.into_iter().enumerate() {
                    button { class: "mi", role: "menuitem", r#type: "button", disabled: !ok, "aria-keyshortcuts": "{key}", tabindex: if i == item() { "0" } else { "-1" },
                        onmounted: move |e| { keep(els, i, e); if i == 0 { focus_at(els, 0) } },
                        onclick: move |_| pick(a),
                        span { "{a.menu()}" }
                        if !ok && blocked && a != RestoreAction::Fork { span { class: "why", "Stop the agent to restore" } }
                        span { class: "sp" }
                        kbd { "{key}" }
                    }
                }
            }
        }
    };
    if variant == "hover-revert" {
        let on = hot() || open();
        return rsx! {
            div { class: "oxyn-cp hover", style: "position:relative", onkeydown: onkey,
                onmouseenter: move |_| hot.set(true), onmouseleave: move |_| if !revealed { hot.set(false) },
                button { class: if on { "rev on" } else { "rev" }, r#type: "button", "aria-label": "Restore checkpoint from {time}", "aria-haspopup": "menu", "aria-expanded": "{open()}",
                    onfocus: move |_| hot.set(true),
                    onclick: move |_| { item.set(0); open.toggle() },
                    Icon { name: "retry", size: 13.0 }
                }
                {menu_el}
            }
        };
    }
    let show = hot() || open();
    rsx! {
        div { class: if current { "oxyn-cp marker cur" } else if show { "oxyn-cp marker rev" } else { "oxyn-cp marker" }, tabindex: "0",
            "aria-label": "Checkpoint, {time}, {what}",
            onmouseenter: move |_| hot.set(true), onmouseleave: move |_| if !revealed && !open() { hot.set(false) },
            onfocus: move |_| hot.set(true),
            onkeydown: onkey,
            span { class: "hl" }
            span { class: "dm", "aria-hidden": "true", "{glyph}" }
            span { class: "lb", if current { "You are here · " } "Checkpoint · {time} · {what}{kind}" }
            span { class: "hl" }
            if show {
                span { class: "acts",
                    button { class: "rs", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{open()}",
                        onclick: move |_| { item.set(0); open.toggle() },
                        Icon { name: "retry", size: 12.0 } "Restore" Icon { name: "chevron", size: 11.0 }
                    }
                }
            }
            {menu_el}
        }
    }
}

/// Checkpoints as a side list grouped by day (`side-list`) or a rewind picker (`terminal`).
/// One tab stop: ↑ / ↓ / Home / End move the selection; Enter restores (side list) or opens the
/// action line (terminal), where 1–4 pick Restore files / Rewind chat / Both / Fork and Esc
/// steps back.
#[component]
pub fn CheckpointList(
    /// Checkpoints, newest last.
    checkpoints: Vec<Checkpoint>,
    /// Time labels, same order. Absolute ISO (`2026-11-14 15:31:09`): the date part groups the
    /// side list by day.
    times: Vec<String>,
    /// `side-list` or `terminal`.
    #[props(default = "side-list".to_string())]
    variant: String,
    /// Selected index at first.
    #[props(default)]
    selected: Option<usize>,
    /// Selection moved: the index.
    #[props(default)]
    on_select: EventHandler<usize>,
    /// A restore was chosen: the index and the action (the side list restores files and chat).
    #[props(default)]
    on_restore: EventHandler<(usize, RestoreAction)>,
) -> Element {
    let n = checkpoints.len();
    let mut sel = use_signal(|| {
        selected
            .unwrap_or(n.saturating_sub(1))
            .min(n.saturating_sub(1))
    });
    let mut picking = use_signal(|| selected.is_some() && variant == "terminal");
    let els: Els = use_signal(Vec::new);
    let term = variant == "terminal";
    // Rows are drawn newest first: row r shows checkpoint n − 1 − r.
    let onkey = move |e: KeyboardEvent| {
        let k = e.key().to_string();
        if term && picking() {
            let a = match k.as_str() {
                "1" => Some(RestoreAction::Files),
                "2" => Some(RestoreAction::Chat),
                "3" => Some(RestoreAction::Both),
                "4" => Some(RestoreAction::Fork),
                _ => None,
            };
            if let Some(a) = a {
                e.prevent_default();
                on_restore.call((sel(), a));
                picking.set(false);
                return;
            }
            if k == "Escape" {
                e.prevent_default();
                picking.set(false);
                return;
            }
        }
        let Some(ck) = CpKey::from_name(&k) else {
            return;
        };
        e.prevent_default();
        match ck {
            CpKey::Enter if term => picking.set(true),
            CpKey::Enter => on_restore.call((sel(), RestoreAction::Both)),
            CpKey::Escape => picking.set(false),
            _ => {
                let row = n - 1 - sel();
                let to = roving(n, row, ck);
                let i = n - 1 - to;
                sel.set(i);
                picking.set(false);
                on_select.call(i);
                focus_at(els, to);
            }
        }
    };
    let s = sel();
    if term {
        return rsx! {
            div { class: "oxyn-cp term", role: "listbox", "aria-label": "Rewind to before", tabindex: "0", "aria-activedescendant": "oxcp-t{s}",
                onkeydown: onkey,
                div { class: "dim", "Rewind to before…" }
                for (i, c) in checkpoints.iter().enumerate().rev() {
                    div { class: if s == i { "opt sel" } else { "opt" }, id: "oxcp-t{i}", role: "option", "aria-selected": "{s == i}",
                        onclick: move |_| { sel.set(i); picking.set(true); on_select.call(i) },
                        if s == i { "› " } else { "  " }
                        "{n - i}. {c.title}"
                        span { class: "dim", "  {times.get(i).cloned().unwrap_or_default()}" }
                        if c.files > 0 { span { class: "dim", "  +{c.lines.0} \u{2212}{c.lines.1}" } }
                    }
                }
                if picking() {
                    div { class: "dim acts", "aria-live": "polite", "  1 Restore files · 2 Rewind chat · 3 Both · 4 Fork · esc never mind" }
                } else {
                    div { class: "dim", "  ↑↓ choose · enter pick" }
                }
            }
        };
    }
    // Group rows (newest first) by the date part of their ISO label.
    let mut groups: Vec<(String, Vec<usize>)> = vec![];
    for i in (0..n).rev() {
        let t = times.get(i).cloned().unwrap_or_default();
        let day = t.split(' ').next().unwrap_or("").to_string();
        match groups.last_mut() {
            Some((d, v)) if *d == day => v.push(i),
            _ => groups.push((day, vec![i])),
        }
    }
    rsx! {
        div { class: "oxyn-cp side",
            for (day, idx) in groups {
                div { class: "day", "{day}" }
                ul { "aria-label": "Checkpoints {day}", onkeydown: onkey,
                    for i in idx {
                        {
                            let c = checkpoints[i].clone();
                            let row = n - 1 - i;
                            let tm = times.get(i).cloned().unwrap_or_default();
                            let clock = tm.split(' ').nth(1).unwrap_or(&tm).to_string();
                            rsx! {
                                li { class: if s == i { "it sel" } else { "it" }, tabindex: if s == i { "0" } else { "-1" }, "aria-current": if s == i { "true" } else { "false" },
                                    "aria-label": "{c.title}, {tm}, {c.files} files",
                                    onmounted: move |e| keep(els, row, e),
                                    onclick: move |_| { sel.set(i); on_select.call(i) },
                                    span { class: "dm", "aria-hidden": "true", if c.chat_only { "◇" } else { "◆" } }
                                    div { class: "tx",
                                        div { class: "tt", "{c.title}" }
                                        div { class: "mt",
                                            time { datetime: "{tm}", "{clock}" }
                                            if c.kind == CheckpointKind::Manual { " · saved" }
                                        }
                                    }
                                    if c.files > 0 { span { class: "ln", span { class: "add", "+{c.lines.0}" } span { class: "del", "\u{2212}{c.lines.1}" } } }
                                    button { class: "rs mini", r#type: "button", tabindex: "-1", "aria-hidden": "true",
                                        onclick: move |e| { e.stop_propagation(); on_restore.call((i, RestoreAction::Both)) },
                                        Icon { name: "retry", size: 11.0 } "Restore"
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

/// The restore toast with its undo hairline.
#[component]
pub fn RestoreToast(
    /// The flow.
    flow: RestoreFlow,
    /// Shared clock.
    now: ClockMs,
    /// Toast action pressed.
    #[props(default)]
    on_action: EventHandler<()>,
) -> Element {
    let Some((text, action, left)) = flow.toast(now) else {
        return rsx! {};
    };
    let tone = match flow {
        RestoreFlow::Partial { .. } => "warn",
        RestoreFlow::Failed { .. } => "danger",
        _ => "",
    };
    rsx! {
        div { class: "oxyn-cp toast {tone}", role: "status",
            Icon { name: "retry", size: 13.0 }
            span { "{text}" }
            if let Some(a) = action { button { r#type: "button", onclick: move |_| on_action.call(()), "{a}" } }
            if matches!(flow, RestoreFlow::Restored { .. }) { span { class: "hair", style: "width:{left * 100.0:.0}%" } }
        }
    }
}

/// Forked checkpoints as an indented tree: each rewind opens a fork one level under the
/// checkpoint it started from, with a count of what it holds. One tab stop: ↑ / ↓ / Home / End
/// move, ← folds and → unfolds a checkpoint with forks, Enter or Space rewinds to it.
#[component]
pub fn CheckpointTree(
    /// Checkpoints in the order taken.
    checkpoints: Vec<Checkpoint>,
    /// Where the agent stands (default: last).
    #[props(default)]
    head: Option<String>,
    /// Time labels, same order (absolute ISO); default the run time.
    #[props(default)]
    times: Vec<String>,
    /// Ids whose forks start folded.
    #[props(default)]
    folded: Vec<String>,
    /// Heading.
    #[props(default = "Branches".to_string())]
    title: String,
    /// Rewind pressed (button, Enter or Space): the checkpoint id.
    #[props(default)]
    on_rewind: EventHandler<String>,
) -> Element {
    let mut fold = use_signal(|| folded.clone());
    let fl = fold();
    let all = branch_tree(&checkpoints, head.as_deref(), &fl);
    let rows: Vec<_> = all.into_iter().filter(|r| !r.hidden).collect();
    let n = rows.len();
    let head_row = rows.iter().position(|r| r.current).unwrap_or(0);
    let mut focus = use_signal(|| head_row);
    let els: Els = use_signal(Vec::new);
    let f = focus().min(n.saturating_sub(1));
    let forks = rows.iter().map(|r| r.depth).max().map_or(0, |d| d + 1);
    let keyinfo: Vec<(String, bool, bool)> = rows
        .iter()
        .map(|r| {
            let c = &checkpoints[r.index];
            (
                c.id.clone(),
                !r.current && c.status != CheckpointStatus::Running,
                checkpoints
                    .iter()
                    .filter(|x| x.parent.as_deref() == Some(&c.id))
                    .count()
                    > 1,
            )
        })
        .collect();
    rsx! {
        div { class: "oxyn-cp tree",
            div { class: "bar",
                span { class: "t", "{title}" }
                span { class: "sp" }
                span { class: "n", "aria-live": "polite", "{checkpoints.len()} checkpoints · {forks} lines" }
            }
            ul { class: "trows oxyn-cp-scroll", role: "tree", "aria-label": "{title}",
                onkeydown: move |e: KeyboardEvent| {
                    let Some(k) = cp_key(&e) else { return };
                    e.prevent_default();
                    let Some((id, can, has)) = keyinfo.get(f).cloned() else { return };
                    match k {
                        CpKey::Enter => if can { on_rewind.call(id) },
                        CpKey::Left if has => { if !fold.peek().contains(&id) { fold.write().push(id) } }
                        CpKey::Right if has => fold.write().retain(|x| *x != id),
                        CpKey::Left | CpKey::Right => {}
                        _ => { let to = roving(n, f, k); focus.set(to); focus_at(els, to); }
                    }
                },
                for (ri, r) in rows.into_iter().enumerate() {
                    {
                        let c = checkpoints[r.index].clone();
                        let id = c.id.clone();
                        let id2 = c.id.clone();
                        let st = status_cls(c.status);
                        let has = checkpoints.iter().filter(|x| x.parent.as_deref() == Some(&c.id)).count() > 1;
                        let is_folded = fl.contains(&c.id);
                        let time = label_at(&times, &c, r.index);
                        let cls = format!("tr {st}{}{}{}", if r.on_path { " path" } else { " off" }, if r.current { " cur" } else { "" }, if ri == f { " foc" } else { "" });
                        let can = !r.current && c.status != CheckpointStatus::Running;
                        let aria = format!(
                            "{}, {st}, {time}{}{}",
                            c.title,
                            if r.fork { ", fork" } else { "" },
                            if r.current { ", current" } else if r.on_path { "" } else { ", left behind" }
                        );
                        rsx! {
                            li { class: "{cls}", role: "treeitem", "aria-level": "{r.depth + 1}", tabindex: if ri == f { "0" } else { "-1" },
                                "aria-expanded": if has { if is_folded { "false" } else { "true" } } else { "undefined" },
                                "aria-current": if r.current { "step" } else { "false" },
                                "aria-label": "{aria}",
                                style: "--dep:{r.depth}",
                                onmounted: move |e| keep(els, ri, e), onfocus: move |_| focus.set(ri),
                                span { class: "gd", "aria-hidden": "true",
                                    for _ in 0..r.depth { span { class: "gl" } }
                                    span { class: if r.fork { "gk fork" } else { "gk" }, if r.fork { if r.last { "└" } else { "├" } } }
                                }
                                if has {
                                    button { class: "fold", r#type: "button", tabindex: "-1", "aria-hidden": "true",
                                        onclick: move |_| { if fold.peek().contains(&id2) { fold.write().retain(|x| *x != id2) } else { fold.write().push(id2.clone()) } },
                                        if is_folded { "+{r.below}" } else { "−" }
                                    }
                                }
                                span { class: "dot {st}", "aria-hidden": "true" }
                                span { class: if c.status == CheckpointStatus::Running { "tt shim" } else { "tt" }, "{c.title}" }
                                if r.fork { span { class: "badge", "Fork" } }
                                if c.status == CheckpointStatus::Failed { span { class: "badge failed", "Failed" } }
                                if c.status == CheckpointStatus::Stopped { span { class: "badge", "Stopped" } }
                                if r.current { span { class: "badge here", "Head" } }
                                span { class: "sp" }
                                span { class: "meta",
                                    if c.files > 0 { span { class: "ln", span { class: "add", "+{c.lines.0}" } span { class: "del", "\u{2212}{c.lines.1}" } } }
                                    span { class: "du", "{time}" }
                                }
                                if can {
                                    button { class: "rw", r#type: "button", tabindex: "-1", "aria-hidden": "true", onclick: move |_| on_rewind.call(id.clone()), Icon { name: "retry", size: 12.0 } "Rewind" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A thumbnail per checkpoint on a scrubbable track: drag the playhead or press ← / → / Home /
/// End to move through the run, Enter or the button restores the frame on the playhead.
#[component]
pub fn CheckpointFilmStrip(
    /// Checkpoints in the order taken.
    checkpoints: Vec<Checkpoint>,
    /// Time labels, same order (absolute ISO); default the run time.
    #[props(default)]
    times: Vec<String>,
    /// Frame on the playhead at first (default: last).
    #[props(default)]
    selected: Option<usize>,
    /// Lines drawn in each thumbnail.
    #[props(default = 6)]
    thumb_rows: usize,
    /// Playhead moved: the index.
    #[props(default)]
    on_select: EventHandler<usize>,
    /// Restore pressed on the frame: the index.
    #[props(default)]
    on_restore: EventHandler<usize>,
) -> Element {
    let n = checkpoints.len();
    let mut strip = use_signal(|| FilmStrip::new(n, selected.unwrap_or(n.saturating_sub(1))));
    let els: Els = use_signal(Vec::new);
    let at = strip.read().at;
    let ph = strip.read().playhead();
    let cur = checkpoints.get(at).cloned();
    let vt = cur.as_ref().map(|c| c.title.clone()).unwrap_or_default();
    let ticks: Vec<f32> = (0..n)
        .map(|i| {
            if n > 1 {
                i as f32 * 100.0 / (n - 1) as f32
            } else {
                0.0
            }
        })
        .collect();
    let mut go = move |i: usize, focus: bool| {
        strip.write().at = i;
        on_select.call(i);
        if focus {
            focus_at(els, i);
        }
    };
    rsx! {
        div { class: "oxyn-cp film",
            div { class: "frames oxyn-cp-scroll", role: "listbox", "aria-label": "Checkpoint frames", "aria-orientation": "horizontal",
                onkeydown: move |e: KeyboardEvent| {
                    let Some(k) = cp_key(&e) else { return };
                    match k {
                        CpKey::Enter => { e.prevent_default(); on_restore.call(at) }
                        CpKey::Left | CpKey::Right | CpKey::Home | CpKey::End | CpKey::Up | CpKey::Down => {
                            e.prevent_default();
                            let to = strip.write().key(k);
                            go(to, true);
                        }
                        _ => {}
                    }
                },
                for (i, c) in checkpoints.iter().enumerate() {
                    {
                        let st = status_cls(c.status);
                        let bars = thumb_bars(c, thumb_rows);
                        let time = label_at(&times, c, i);
                        rsx! {
                            div { class: if i == at { "fr {st} on" } else { "fr {st}" }, role: "option", "aria-selected": "{i == at}", tabindex: if i == at { "0" } else { "-1" },
                                "aria-label": "{i + 1}. {c.title}, {st}, {time}",
                                onmounted: move |e| keep(els, i, e),
                                onclick: move |_| go(i, false),
                                div { class: "th", "aria-hidden": "true",
                                    for (w, hot) in bars { span { class: if hot { "b hot" } else { "b" }, style: "width:{w}%" } }
                                    if c.status == CheckpointStatus::Failed { span { class: "x", "!" } }
                                }
                                div { class: "fl", span { class: "ix", "{i + 1}" } span { class: "tt", "{c.title}" } }
                            }
                        }
                    }
                }
            }
            div { class: "track",
                span { class: "rail", "aria-hidden": "true",
                    for left in ticks.iter().copied() { span { class: "tick", style: "left:{left:.2}%" } }
                    span { class: "head", style: "left:{ph * 100.0:.2}%" }
                }
                input { class: "scrub", r#type: "range", min: "0", max: "{n.saturating_sub(1)}", step: "1", value: "{at}",
                    "aria-label": "Scrub checkpoints", "aria-valuetext": "{vt}",
                    oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<usize>() { go(v.min(n.saturating_sub(1)), false) } },
                }
            }
            if let Some(c) = cur {
                div { class: "cap", "aria-live": "polite",
                    span { class: "ix", "{at + 1} of {n}" }
                    span { class: "tt", "{c.title}" }
                    if c.status == CheckpointStatus::Failed { span { class: "badge failed", "Failed" } }
                    if c.status == CheckpointStatus::Running { span { class: "badge", "Running" } }
                    span { class: "sp" }
                    if c.files > 0 { span { class: "ln", span { class: "add", "+{c.lines.0}" } span { class: "del", "\u{2212}{c.lines.1}" } } }
                    span { class: "du", "{label_at(&times, &c, at)}" }
                    button { class: "rs", r#type: "button", disabled: c.status == CheckpointStatus::Running, onclick: move |_| on_restore.call(at),
                        Icon { name: "retry", size: 12.0 } "Restore"
                    }
                }
            }
        }
    }
}

/// A vertical stepper of the agent's steps, newest at the bottom: undo walks back one rung at a
/// time, redo walks forward, and a step that can't be undone (an email sent, a deploy) locks the
/// rungs under it. One tab stop: ↑ / ↓ / Home / End move, Enter jumps to the focused rung,
/// ⌘Z / Ctrl+Z undo and ⇧⌘Z / Ctrl+Y redo.
#[component]
pub fn UndoLadder(
    /// Steps, oldest first.
    steps: Vec<UndoStep>,
    /// Steps in effect at first (default: all).
    #[props(default)]
    applied: Option<usize>,
    /// Heading.
    #[props(default = "Steps".to_string())]
    title: String,
    /// The number of steps in effect changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let init = steps.clone();
    let mut m = use_signal(move || {
        let mut l = LadderModel::new(init);
        if let Some(a) = applied {
            l.applied = a.clamp(l.floor(), l.steps.len());
            l.focus = l.applied.saturating_sub(1);
        }
        l
    });
    let els: Els = use_signal(Vec::new);
    let l = m.read().clone();
    let n = l.steps.len();
    let mut act = move |k: CpKey| {
        let before = m.peek().applied;
        m.write().key(k);
        let (after, f) = (m.peek().applied, m.peek().focus);
        if after != before {
            on_change.call(after);
        }
        if !matches!(k, CpKey::Undo | CpKey::Redo | CpKey::Enter) {
            focus_at(els, f);
        }
    };
    let can_undo = l.applied > l.floor();
    let can_redo = l.applied < n;
    rsx! {
        div { class: "oxyn-cp ladder",
            div { class: "bar",
                span { class: "t", "{title}" }
                span { class: "sp" }
                span { class: "n", "aria-live": "polite", "{l.summary()}" }
                button { class: "rs", r#type: "button", disabled: !can_undo, "aria-keyshortcuts": "Control+Z Meta+Z", onclick: move |_| act(CpKey::Undo), Icon { name: "retry", size: 12.0 } "Undo" }
                button { class: "rs", r#type: "button", disabled: !can_redo, "aria-keyshortcuts": "Control+Y Meta+Shift+Z", onclick: move |_| act(CpKey::Redo), "Redo" }
            }
            ol { class: "rungs oxyn-cp-scroll", "aria-label": "{title}",
                onkeydown: move |e: KeyboardEvent| {
                    if let Some(k) = cp_key(&e) { e.prevent_default(); act(k) }
                },
                for (i, s) in l.steps.iter().enumerate() {
                    {
                        let st = l.state(i);
                        let edge = i + 1 == l.applied;
                        let aria = format!(
                            "Step {}, {}, {}, {st}{}",
                            i + 1,
                            s.title,
                            s.detail,
                            if s.reversible { "" } else { ", can't be undone" }
                        );
                        rsx! {
                            li { class: if i == l.focus { "rg {st} foc" } else { "rg {st}" }, tabindex: if i == l.focus { "0" } else { "-1" },
                                "aria-current": if edge { "step" } else { "false" },
                                "aria-label": "{aria}",
                                onmounted: move |e| keep(els, i, e),
                                onclick: move |_| { m.write().focus = i; act(CpKey::Enter) },
                                span { class: "num", "aria-hidden": "true", if st == "undone" { "{i + 1}" } else if s.reversible { "✓" } else { "■" } }
                                span { class: "tx",
                                    span { class: "tt", "{s.title}" }
                                    span { class: "mt", "{s.detail}" }
                                }
                                span { class: "sp" }
                                if !s.reversible { span { class: "badge warn", "Can't undo" } }
                                if st == "undone" { span { class: "badge", "Undone" } }
                                if edge { span { class: "badge here", "Now" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The restore preview sheet: what restoring a checkpoint changes (files with `M` / `A` / `D`
/// and line counts, plus the turns rewound), then Restore, a confirm step and Cancel. A modal
/// dialog: focus starts on the primary button and Tab stays inside; Enter presses it, Esc steps
/// back from confirm and then closes.
#[component]
pub fn RestorePreviewSheet(
    /// Files that change.
    files: Vec<PreviewFile>,
    /// Conversation turns rewound.
    #[props(default)]
    turns: u32,
    /// What is restored.
    #[props(default = RestoreAction::Both)]
    action: RestoreAction,
    /// The checkpoint's title.
    title: String,
    /// The checkpoint's time (absolute ISO).
    time: String,
    /// Start on the confirm step.
    #[props(default)]
    confirm: bool,
    /// Restore confirmed.
    #[props(default)]
    on_restore: EventHandler<RestoreAction>,
    /// Cancelled or closed.
    #[props(default)]
    on_cancel: EventHandler<()>,
) -> Element {
    let f0 = files.clone();
    let mut m = use_signal(move || {
        let mut p = RestorePreview::new(f0, turns, action);
        if confirm {
            p.press();
        }
        p
    });
    let els: Els = use_signal(Vec::new);
    let p = m.read().clone();
    let mut primary = move || {
        let was = m.peek().phase;
        m.write().press();
        let now = m.peek().phase;
        if was == PreviewPhase::Confirm && now == PreviewPhase::Done {
            on_restore.call(action);
        }
        if matches!(was, PreviewPhase::Done | PreviewPhase::Cancelled) {
            m.write().phase = PreviewPhase::Review;
        }
        focus_at(els, 0);
    };
    let mut cancel = move || {
        m.write().cancel();
        if m.peek().phase == PreviewPhase::Cancelled {
            on_cancel.call(());
        }
        focus_at(els, 0);
    };
    let (add, del) = p.totals();
    let phase = p.phase;
    if matches!(phase, PreviewPhase::Done | PreviewPhase::Cancelled) {
        let msg = if phase == PreviewPhase::Done {
            format!("Restored {} to {time}", p.summary())
        } else {
            "Nothing changed".to_string()
        };
        return rsx! {
            div { class: "oxyn-cp sheet closed", role: "status",
                span { class: "dm", "aria-hidden": "true", "◆" }
                span { "{msg}" }
                span { class: "sp" }
                button { class: "rs", r#type: "button", onmounted: move |e| keep(els, 0, e), onclick: move |_| primary(), "Preview again" }
            }
        };
    }
    rsx! {
        div { class: "oxyn-cp sheet", role: "dialog", "aria-modal": "true", "aria-labelledby": "oxcp-sh-t", "aria-describedby": "oxcp-sh-s",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape { e.prevent_default(); cancel() }
            },
            div { class: "sh-h",
                div { class: "sh-t", id: "oxcp-sh-t", "Restore “{title}”" }
                div { class: "sh-s", id: "oxcp-sh-s",
                    time { datetime: "{time}", "{time}" }
                    " · {p.summary()}"
                }
            }
            if !matches!(action, RestoreAction::Chat) {
                ul { class: "files oxyn-cp-scroll", "aria-label": "Files that change",
                    for f in p.files.iter() {
                        li { class: "fi",
                            span { class: "tag {f.change.tag()}", title: "{f.change:?}", "{f.change.tag()}" }
                            span { class: "pa", "{f.path}" }
                            span { class: "sp" }
                            span { class: "ln", if f.lines.0 > 0 { span { class: "add", "+{f.lines.0}" } } if f.lines.1 > 0 { span { class: "del", "\u{2212}{f.lines.1}" } } }
                            span { class: "bar2", "aria-hidden": "true",
                                span { class: "a", style: "flex:{f.lines.0}" }
                                span { class: "d", style: "flex:{f.lines.1}" }
                            }
                        }
                    }
                }
            }
            div { class: "sh-tot", span { class: "add", "+{add}" } span { class: "del", "\u{2212}{del}" } if p.turns > 0 && !matches!(action, RestoreAction::Files) { span { "{p.turns} turns rewound" } } }
            if phase == PreviewPhase::Confirm {
                div { class: "warnline", role: "alert", "Your current changes are kept as a checkpoint, so you can come back to them." }
            }
            div { class: "sh-f",
                span { class: "sp" }
                button { class: "btn", r#type: "button", onmounted: move |e| keep(els, 1, e),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Tab { e.prevent_default(); e.stop_propagation(); focus_at(els, 0) },
                    onclick: move |_| cancel(), if phase == PreviewPhase::Confirm { "Back" } else { "Cancel" } }
                button { class: if phase == PreviewPhase::Confirm { "btn pri go" } else { "btn pri" }, r#type: "button",
                    onmounted: move |e| { keep(els, 0, e); focus_at(els, 0) },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Tab { e.prevent_default(); e.stop_propagation(); focus_at(els, 1) },
                    onclick: move |_| primary(), "{p.primary()}" }
            }
        }
    }
}
