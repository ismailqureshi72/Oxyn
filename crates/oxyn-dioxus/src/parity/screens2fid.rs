//! Screens 2: the agent screens on the Screens 1 frame ([`super::screensfid::AppFrame`] with the
//! agent sidebar) – [`OrchestrationScreen`] and [`AgentThreadScreen`] so far – each with Sample,
//! Loading and Empty data. Models: `oxyn_core::parity::screens2fid`. Styles: `screens2fid.css`
//! (`oxsag-` shared, `oxsor-` orchestration, `oxstd-` thread). Every agent, person, project and
//! line of copy is fictional and our own.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::parity::screens2fid::{
    AG_PROJECTS, AgStatus, OrchNode, ScreenState, ThreadItem, ToolRun, call_ms, clock, dur,
    error_count, eur, first_words, orch_cents, orch_children, orch_done, orch_events,
    orch_next_running, orch_path, orch_rollup, orch_trace, orch_tree, stream_at, thread_counts,
    thread_items, thread_reply, toks, tools_line, trace_rows, trace_span, upload_pct, zoom_step,
};

use super::efflistfid::chevron;
use super::screensfid::{
    AppFrame, MenuButton, MenuItem, ScreenComposer, empty, ico, since, skel, use_threads,
};

/// A status tag in the status colours.
pub(crate) fn tag(s: AgStatus) -> Element {
    rsx! { span { class: "oxsag-tag", "data-tone": "{s.key()}", "{s.label()}" } }
}

/// A row of tabs (`role=tablist`): ←/→ move between them.
pub(crate) fn tabs(
    aria: &'static str,
    items: Vec<(&'static str, String)>,
    cur: Signal<&'static str>,
) -> Element {
    let keys: Vec<&'static str> = items.iter().map(|(k, _)| *k).collect();
    let mut cur = cur;
    rsx! {
        div { class: "oxsag-tabs", role: "tablist", "aria-label": "{aria}",
            for (k, l) in items {
                {
                    let on = cur() == k;
                    let ks = keys.clone();
                    rsx! {
                        button { key: "{k}", r#type: "button", role: "tab", class: "oxsag-tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" },
                            onclick: move |_| cur.set(k),
                            onkeydown: move |e: KeyboardEvent| {
                                let i = ks.iter().position(|x| *x == k).unwrap_or(0);
                                let n = ks.len();
                                let j = match e.key() { Key::ArrowRight => (i + 1) % n, Key::ArrowLeft => (i + n - 1) % n, _ => return };
                                e.prevent_default();
                                cur.set(ks[j]);
                            },
                            "{l}"
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── Orchestration ──

fn context_for(role: &str) -> &'static [&'static str] {
    match role {
        "Research" => &[
            "operator-reports/ (14 files)",
            "web: harbour authority pages",
            "brief.md",
        ],
        "Analysis" => &["passengers-2025.csv", "Scout's notes", "brief.md"],
        "Writing" => &["report-outline.md", "style guide"],
        "Review" => &["every source Scout read"],
        _ => &[
            "brief.md",
            "Harbor launch board",
            "your last three messages",
        ],
    }
}

/// A live multi-agent run: the agents as a tree on a zoomable canvas, run controls, and a
/// resizable run panel with the trace, tokens and cost, and events.
#[component]
pub fn OrchestrationScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let nodes = use_signal(move || orch_tree(sample));
    let calls = use_signal(move || orch_trace(sample));
    let events = use_signal(move || orch_events(sample));
    let start = use_signal(|| now.0);
    // Some(elapsed) while paused or stopped.
    let mut frozen = use_signal(|| None::<u64>);
    let mut stopped = use_signal(|| false);
    let mut folded = use_signal(Vec::<u32>::new);
    let mut pick = use_signal(move || if sample { Some(2u32) } else { None });
    let mut zoom = use_signal(|| 100u32);
    let tab = use_signal(|| "trace");
    let mut ph = use_signal(|| 220u32);
    let mut pfold = use_signal(|| false);
    let mut tfold = use_signal(Vec::<u32>::new);
    let mut said = use_signal(String::new);
    let mut proj = use_signal(|| "Harbor launch".to_string());
    let mut run = use_signal(|| "run 7c41".to_string());
    let cards = use_signal(Vec::<(u32, Rc<MountedData>)>::new);
    // A card to focus once it is (re)mounted (after its branch unfolds).
    let mut want = use_signal(|| None::<u32>);
    let mut drag = use_signal(|| None::<(f64, u32)>);
    let loading = st == ScreenState::Loading;
    let all = nodes();
    let elapsed = frozen().unwrap_or(31_500 + since(now, start()));
    let paused = frozen().is_some() && !stopped();
    let live = sample && frozen().is_none();
    let (dn, tot) = orch_done(&all);
    let cents = orch_cents(&all, elapsed);
    let tokens: u32 =
        all.iter().map(|n| n.tokens).sum::<u32>() + if sample { (elapsed / 100) as u32 } else { 0 };
    let ev = events();
    let errs = error_count(&ev);
    let projs: Vec<MenuItem> = AG_PROJECTS.iter().map(|p| (*p, *p, "")).collect();
    let runs: Vec<MenuItem> = vec![
        ("run 7c41", "run 7c41 · now", ""),
        ("run 6a02", "run 6a02 · yesterday", ""),
        ("run 5e19", "run 5e19 · Monday", ""),
    ];
    let state_tag = if !sample {
        None
    } else if stopped() {
        Some(("bad", "Stopped"))
    } else if paused {
        Some(("warn", "Paused"))
    } else {
        Some(("live", "Live"))
    };
    let actions = rsx! {
        div { class: "oxsor-acts",
            MenuButton { label: proj(), aria: "Project", class: "oxsor-path".to_string(), items: projs, checked: proj(), on_pick: move |k: String| proj.set(k) }
            MenuButton { label: run(), aria: "Run", class: "oxsor-path".to_string(), items: runs, checked: run(), on_pick: move |k: String| run.set(k) }
            if let Some((t, l)) = state_tag { span { class: "oxsag-tag oxsor-live", "data-tone": "{t}", "{l}" } }
            if sample && !stopped() {
                button { r#type: "button", class: "oxscn-btn oxsor-pause", "aria-pressed": "{paused}",
                    onclick: move |_| {
                        if frozen.peek().is_some() { frozen.set(None); said.set("Run resumed".into()); }
                        else { frozen.set(Some(31_500 + since(now, *start.peek()))); said.set("Run paused".into()); }
                    },
                    if paused { {ico("runs")} span { "Resume" } } else { {ico("pause")} span { "Pause" } }
                }
                button { r#type: "button", class: "oxscn-btn oxsor-stop",
                    onclick: move |_| {
                        let e = frozen.peek().unwrap_or(31_500 + since(now, *start.peek()));
                        frozen.set(Some(e)); stopped.set(true); said.set("Run stopped".into());
                    },
                    {ico("stop")} span { "Stop" }
                }
            }
        }
    };
    let picked = pick().and_then(|id| all.iter().find(|n| n.id == id).cloned());
    let work = picked.map(|n| rsx! { OrchAgentDetail { key: "{n.id}", node: n, nodes, pick } });
    let focus_card = move |id: u32| {
        if let Some((_, m)) = cards.peek().iter().find(|(k, _)| *k == id) {
            let m = m.clone();
            spawn(async move {
                let _ = m.set_focus(true).await;
            });
        }
    };
    let span = trace_span(&calls(), elapsed);
    let rows = trace_rows(&calls(), &tfold());
    let pf = pfold();
    let style = format!(
        "--oxsor-ph:{}px;--oxsor-z:{}",
        ph(),
        f64::from(zoom()) / 100.0
    );
    let dragging = drag().is_some();
    rsx! {
        AppFrame { layout, state: state.clone(), height, side: "agent", nav_on: "runs", threads: thr, active, title: "Runs", actions, work, work_label: "Agent",
            div { class: "oxsor", style: "{style}", "data-drag": "{dragging}",
                onpointermove: move |e: PointerEvent| if let Some((y0, h0)) = drag() { let v = (f64::from(h0) - (e.client_coordinates().y - y0)).clamp(120.0, 380.0); ph.set(v as u32); },
                onpointerup: move |_| if drag.peek().is_some() { drag.set(None) },
                onpointerleave: move |_| if drag.peek().is_some() { drag.set(None) },
                section { class: "oxsor-canvas", "aria-label": "Agent tree",
                    div { class: "oxsor-tools", role: "toolbar", "aria-label": "Canvas",
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Next agent at work", title: "Next agent at work", disabled: !sample,
                            onclick: move |_| {
                                let n = nodes.peek().clone();
                                let cur = *pick.peek();
                                if let Some(id) = orch_next_running(&n, cur) {
                                    let path = orch_path(&n, id);
                                    folded.write().retain(|f| !path.contains(f));
                                    pick.set(Some(id));
                                    let nm = n.iter().find(|x| x.id == id).map(|x| x.name).unwrap_or("");
                                    said.set(format!("{nm} is running"));
                                    want.set(Some(id));
                                    focus_card(id);
                                }
                            }, {ico("next")} }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Fit the tree", title: "Fit the tree", onclick: move |_| zoom.set(80), {ico("fit")} }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Zoom out", disabled: zoom() <= 50, onclick: move |_| { let z = zoom_step(*zoom.peek(), -1); zoom.set(z); }, {ico("zoomout")} }
                        button { r#type: "button", class: "oxscn-btn oxsor-z", "aria-label": "Zoom {zoom}%, reset to 100%", onclick: move |_| zoom.set(100), "{zoom}%" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Zoom in", disabled: zoom() >= 200, onclick: move |_| { let z = zoom_step(*zoom.peek(), 1); zoom.set(z); }, {ico("zoomin")} }
                    }
                    if loading {
                        div { class: "oxsor-skel", {skel(5)} }
                    } else if all.is_empty() {
                        {empty("No run yet", "Ask for something in the Thread; the agents it starts appear here as a tree.")}
                    } else {
                        div { class: "oxsor-scroll",
                            div { class: "oxsor-zoom",
                                ul { class: "oxsor-tree", role: "tree", "aria-label": "Agents in this run",
                                    {branch(&all, 1, folded, pick, cards, want, 1)}
                                }
                            }
                        }
                    }
                }
                div { class: "oxsor-sep", role: "separator", tabindex: "0", "aria-orientation": "horizontal", "aria-label": "Resize run panel",
                    "aria-valuenow": "{ph}", "aria-valuemin": "120", "aria-valuemax": "380", hidden: pf,
                    onpointerdown: move |e: PointerEvent| { e.prevent_default(); drag.set(Some((e.client_coordinates().y, *ph.peek()))); },
                    onkeydown: move |e: KeyboardEvent| {
                        let d: i32 = match e.key() { Key::ArrowUp => 20, Key::ArrowDown => -20, _ => 0 };
                        if d != 0 { e.prevent_default(); let v = (*ph.peek() as i32 + d).clamp(120, 380); ph.set(v as u32); }
                    },
                }
                section { class: "oxsor-panel", "aria-label": "Run panel", "data-fold": "{pf}",
                    div { class: "oxsor-ph",
                        {tabs("Run panel", vec![("trace", "Trace".to_string()), ("cost", "Tokens & cost".to_string()), ("events", if errs > 0 { format!("Events · {errs}") } else { "Events".to_string() })], tab)}
                        span { class: "oxscn-grow" }
                        dl { class: "oxsor-sum",
                            div { dt { "Agents done" } dd { "{dn}/{tot}" } }
                            div { dt { "Elapsed" } dd { if sample { "{clock((elapsed / 1000) as u32)}" } else { "–" } } }
                            div { dt { "Tokens" } dd { "{toks(tokens)}" } }
                            div { dt { "Spent" } dd { "{eur(cents)}" } }
                        }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": if pf { "Unfold run panel" } else { "Fold run panel" }, "aria-expanded": "{!pf}",
                            onclick: move |_| pfold.toggle(), {chevron(pf)} }
                    }
                    if !pf {
                        div { class: "oxsor-pb", role: "tabpanel",
                            if loading { {skel(4)} }
                            else if all.is_empty() { p { class: "oxscn-hint", "Calls, tokens and events land here while a run works." } }
                            else if tab() == "trace" {
                                div { class: "oxsor-axis", "aria-hidden": "true", span { "0s" } span { "{dur(span / 2)}" } span { "{dur(span)}" } }
                                ul { class: "oxsor-trace", role: "tree", "aria-label": "Trace",
                                    for c in rows {
                                        {
                                            let id = c.id;
                                            let kids = calls().iter().any(|x| x.parent == Some(id));
                                            let open = !tfold().contains(&id);
                                            let ms = call_ms(&c, elapsed);
                                            let l = c.at as f64 * 100.0 / span as f64;
                                            let w = (ms as f64 * 100.0 / span as f64).max(0.6);
                                            let bar = format!("left:{l:.2}%;width:{w:.2}%");
                                            let growing = c.ms == 0 && live;
                                            let lvl = if c.parent.is_some() { 2 } else { 1 };
                                            rsx! {
                                                li { key: "{id}", class: "oxsor-tr", role: "treeitem", "aria-level": "{lvl}", "aria-expanded": if kids { Some(if open { "true" } else { "false" }) } else { None }, "data-err": "{c.err}", "data-lvl": "{lvl}",
                                                    span { class: "oxsor-tl",
                                                        if kids { button { r#type: "button", class: "oxscn-ib oxsor-tf", "aria-label": if open { format!("Fold {}", c.label) } else { format!("Unfold {}", c.label) },
                                                            onclick: move |_| { let mut f = tfold.write(); if let Some(i) = f.iter().position(|x| *x == id) { f.remove(i); } else { f.push(id); } }, {chevron(open)} } }
                                                        span { class: "oxscn-row-t", "{c.label}" }
                                                        span { class: "oxsor-ms", if c.err { "failed · " } "{dur(ms)}" }
                                                    }
                                                    span { class: "oxsor-lane", span { class: "oxsor-bar", "data-grow": "{growing}", style: "{bar}" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if tab() == "cost" {
                                table { class: "oxsor-tbl",
                                    thead { tr { th { "Agent" } th { "Model" } th { class: "oxsor-num", "Tokens" } th { class: "oxsor-num", "Cost" } th { class: "oxsor-num", "Time" } } }
                                    tbody {
                                        for n in all.iter().cloned() {
                                            tr { key: "{n.id}", td { "{n.name}" } td { "{n.model}" } td { class: "oxsor-num", "{toks(n.tokens)}" } td { class: "oxsor-num", "{eur(n.cents)}" } td { class: "oxsor-num", "{dur(n.ms)}" } }
                                        }
                                    }
                                }
                            } else {
                                ul { class: "oxsor-ev",
                                    for (i, e) in ev.iter().enumerate() {
                                        li { key: "{i}", "data-err": "{e.err}",
                                            span { class: "oxsor-ms", "+{dur(e.at)}" }
                                            if e.err { span { class: "oxsag-tag", "data-tone": "bad", "Error" } }
                                            span { "{e.text}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

fn branch(
    all: &[OrchNode],
    id: u32,
    mut folded: Signal<Vec<u32>>,
    mut pick: Signal<Option<u32>>,
    mut cards: Signal<Vec<(u32, Rc<MountedData>)>>,
    mut want: Signal<Option<u32>>,
    lvl: u32,
) -> Element {
    let Some(n) = all.iter().find(|n| n.id == id).cloned() else {
        return rsx! {};
    };
    let kids = orch_children(all, id);
    let open = !folded().contains(&id);
    let (c, t) = orch_rollup(all, id);
    let sel = pick() == Some(id);
    let has = !kids.is_empty();
    let name = n.name;
    rsx! {
        li { key: "{id}", class: "oxsor-li", role: "treeitem", "aria-level": "{lvl}", "aria-selected": "{sel}",
            "aria-expanded": if has { Some(if open { "true" } else { "false" }) } else { None },
            div { class: "oxsor-node",
                button { r#type: "button", class: "oxsor-card", "data-sel": "{sel}", "data-s": "{n.status.key()}", "aria-label": "{name}, {n.role}, {n.status.label()}",
                    onmounted: move |e: MountedEvent| {
                        let d = e.data();
                        cards.write().retain(|(k, _)| *k != id);
                        cards.write().push((id, d.clone()));
                        if *want.peek() == Some(id) { want.set(None); spawn(async move { let _ = d.set_focus(true).await; }); }
                    },
                    onclick: move |_| { want.set(None); pick.set(Some(id)); },
                    span { class: "oxsor-ct",
                        span { class: "oxsor-dot", "data-s": "{n.status.key()}" }
                        span { class: "oxsor-nm", "{name}" }
                        {tag(n.status)}
                    }
                    span { class: "oxsor-task", "{n.task}" }
                    span { class: "oxsor-meta", span { "{n.role} · {n.model}" } span { "{eur(c)} · {dur(t)}" } }
                }
                if has {
                    button { r#type: "button", class: "oxsor-fold", "aria-label": if open { format!("Fold {name}'s branch") } else { format!("Unfold {name}'s branch ({})", kids.len()) },
                        onclick: move |_| { let mut f = folded.write(); if let Some(i) = f.iter().position(|x| *x == id) { f.remove(i); } else { f.push(id); } },
                        if open { {chevron(true)} } else { span { "{kids.len()}" } }
                    }
                }
            }
            if has && open {
                ul { class: "oxsor-kids", role: "group",
                    for k in kids { {branch(all, k.id, folded, pick, cards, want, lvl + 1)} }
                }
            }
        }
    }
}

/// The picked agent beside the canvas: what it does, its branch, its helpers and its context.
#[component]
fn OrchAgentDetail(
    node: OrchNode,
    nodes: Signal<Vec<OrchNode>>,
    pick: Signal<Option<u32>>,
) -> Element {
    let all = nodes();
    let (c, t) = orch_rollup(&all, node.id);
    let kids = orch_children(&all, node.id);
    let boss = node
        .parent
        .and_then(|p| all.iter().find(|n| n.id == p))
        .map(|n| n.name)
        .unwrap_or("You");
    let mut pick = pick;
    rsx! {
        div { class: "oxsag-det",
            p { class: "oxsag-cap", "{node.role} · {node.model}" }
            h2 { class: "oxsag-h2", "{node.name}" }
            div { {tag(node.status)} }
            p { class: "oxsag-p", "{node.task}" }
            dl { class: "oxsag-dl",
                div { dt { "Reports to" } dd { "{boss}" } }
                div { dt { "Its own cost" } dd { "{eur(node.cents)} · {dur(node.ms)}" } }
                div { dt { "With its helpers" } dd { "{eur(c)} · {dur(t)}" } }
                div { dt { "Tokens" } dd { "{toks(node.tokens)}" } }
            }
            p { class: "oxscn-gh", "Tasks it handed out" }
            if kids.is_empty() { p { class: "oxsag-p oxsag-mute", "None – it does this one itself." } }
            ul { class: "oxsag-list",
                for k in kids {
                    li { key: "{k.id}",
                        button { r#type: "button", class: "oxsag-li", onclick: move |_| pick.set(Some(k.id)),
                            span { class: "oxsor-dot", "data-s": "{k.status.key()}" }
                            span { class: "oxscn-row-t", "{k.name}" }
                            span { class: "oxscn-row-a", "{k.status.label()}" }
                        }
                    }
                }
            }
            p { class: "oxscn-gh", "Context" }
            ul { class: "oxsag-list",
                for (i, x) in context_for(node.role).iter().enumerate() { li { key: "{i}", class: "oxsag-ctx", {ico("file")} span { "{x}" } } }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────── Thread ──

const ARTIFACTS: [(&str, &[&str]); 2] = [
    (
        "guest-list.csv",
        &[
            "name, table, plus one",
            "Ines Marlow, 1, yes",
            "Teo Brand, 1, no",
            "Ada Fenn, 2, yes",
            "… 58 more rows",
        ],
    ),
    (
        "press-note-draft.md",
        &[
            "# Small boats, big evenings",
            "The Harbor launch opens on Thursday at the pier hall.",
            "“Small boats, big evenings.” – Rowan Vale",
            "Doors at 17:00; the first ferry leaves at 18:30.",
        ],
    ),
];

const TASKS: [(u32, &str, &str, AgStatus, u8, u8); 5] = [
    (
        21,
        "Book the venue",
        "Holding the pier hall until 12:00",
        AgStatus::Running,
        1,
        3,
    ),
    (
        22,
        "Press note",
        "Waiting for your quote",
        AgStatus::NeedsYou,
        2,
        4,
    ),
    (23, "Guest list", "Finished", AgStatus::Done, 3, 3),
    (
        24,
        "Catering quotes",
        "Asked four caterers",
        AgStatus::Running,
        1,
        2,
    ),
    (
        25,
        "Ferry timetable",
        "Pick Thursday or Friday times",
        AgStatus::NeedsYou,
        0,
        2,
    ),
];

/// One conversation with the orchestrator: plain answers, folded tool lines, task cards and
/// files, a stream that survives a dropped connection, a slow upload and time away.
#[component]
pub fn AgentThreadScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// `sample`, `loading` or `empty`.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The clock.
    now: ClockMs,
) -> Element {
    let st = ScreenState::from_key(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let mut items = use_signal(move || thread_items(sample));
    let mut open_tools = use_signal(Vec::<usize>::new);
    let text = use_signal(String::new);
    // The reply being streamed: (text, sent at, drops after n words, drop ms).
    let mut stream = use_signal(|| None::<(String, u64, usize, u64)>);
    let mut sends = use_signal(|| 0u32);
    let mut said = use_signal(String::new);
    let start = use_signal(|| now.0);
    let mut wtab = use_signal(|| "files");
    let mut file = use_signal(|| 1usize);
    let mut filter = use_signal(|| None::<AgStatus>);
    let mut task_open = use_signal(|| None::<u32>);
    let loading = st == ScreenState::Loading;
    let (running, need) = thread_counts(sample);
    // Stream state.
    let mut shown = None::<(String, bool, bool)>;
    if let Some((t, at, d, dm)) = stream() {
        let words = t.split_whitespace().count();
        let (n, down) = stream_at(words, since(now, at), d, dm);
        shown = Some((first_words(&t, n), down, n >= words));
    }
    let busy = shown.as_ref().is_some_and(|(_, _, fin)| !fin);
    let up = upload_pct(since(now, start()), 14_000);
    let list = items();
    let finish = move |mut items: Signal<Vec<ThreadItem>>,
                       mut stream: Signal<Option<(String, u64, usize, u64)>>,
                       cut: Option<String>| {
        let cur = stream.peek().clone();
        if let Some((t, _, _, _)) = cur {
            let text = cut.unwrap_or(t);
            items.write().push(ThreadItem::Agent { text, at: "now" });
        }
        stream.set(None);
    };
    if shown.as_ref().is_some_and(|(_, _, fin)| *fin) {
        // Land the finished reply in the log (after this render).
        spawn(async move {
            finish(items, stream, None);
            said.set("Reply finished".into());
        });
    }
    let actions = rsx! {
        button { r#type: "button", class: "oxscn-btn oxstd-art", onclick: move |_| wtab.set("files"), {ico("file")} span { "Artifacts" } }
    };
    let tasks_view = filter();
    let work = Some(rsx! {
        div { class: "oxsag-det",
            {tabs("Work area", vec![("files", "Artifacts".to_string()), ("tasks", "Tasks".to_string())], wtab)}
            if wtab() == "files" {
                if !sample { p { class: "oxsag-p oxsag-mute", "Files the orchestrator makes for you open here." } }
                else {
                    div { class: "oxstd-files", role: "group", "aria-label": "Files",
                        for (i, (f, _)) in ARTIFACTS.iter().enumerate() {
                            button { key: "{i}", r#type: "button", class: "oxscn-chip oxscn-chip-b", "aria-pressed": "{file() == i}", onclick: move |_| file.set(i), {ico("file")} "{f}" }
                        }
                    }
                    pre { class: "oxstd-pre", "aria-label": "{ARTIFACTS[file()].0}",
                        for (i, l) in ARTIFACTS[file()].1.iter().enumerate() { span { key: "{i}", "{l}\n" } }
                    }
                }
            } else {
                div { class: "oxstd-filt",
                    for (k, l) in [(None, "All"), (Some(AgStatus::Running), "Running"), (Some(AgStatus::NeedsYou), "Needs you")] {
                        button { key: "{l}", r#type: "button", class: "oxscn-chip oxscn-chip-b", "aria-pressed": "{tasks_view == k}", onclick: move |_| filter.set(k), "{l}" }
                    }
                }
                if !sample { p { class: "oxsag-p oxsag-mute", "Tasks it starts for you are listed here." } }
                ul { class: "oxsag-list",
                    for (id, n, s, stt, d, of) in TASKS.iter().copied().filter(|t| sample && tasks_view.is_none_or(|f| f == t.3)) {
                        li { key: "{id}", class: "oxstd-task", "data-open": "{task_open() == Some(id)}",
                            div { class: "oxstd-tt", span { class: "oxscn-row-t", "{n}" } {tag(stt)} }
                            p { class: "oxsag-p oxsag-mute", "{s}" }
                            div { class: "oxstd-prog", role: "progressbar", "aria-label": "{n} steps", "aria-valuenow": "{d}", "aria-valuemin": "0", "aria-valuemax": "{of}",
                                span { style: "width:{u32::from(d) * 100 / u32::from(of.max(1))}%" } }
                        }
                    }
                }
            }
        }
    });
    rsx! {
        AppFrame { layout, state: state.clone(), height, side: "agent", nav_on: "thread", threads: thr, active, title: "Thread", actions, work, work_label: "Artifacts",
            on_nav: move |k: String| { if k == "tasks" { wtab.set("tasks"); filter.set(None); } },
            div { class: "oxstd",
                header { class: "oxstd-head",
                    h1 { class: "oxscn-h1", "Today" }
                    if sample {
                        button { r#type: "button", class: "oxscn-chip oxscn-chip-b oxstd-c", onclick: move |_| { wtab.set("tasks"); filter.set(Some(AgStatus::Running)); said.set(format!("{running} tasks running")); },
                            span { class: "oxsor-dot", "data-s": "run" } "{running} running" }
                        button { r#type: "button", class: "oxscn-chip oxscn-chip-b oxstd-c", onclick: move |_| { wtab.set("tasks"); filter.set(Some(AgStatus::NeedsYou)); said.set(format!("{need} tasks need you")); },
                            span { class: "oxsor-dot", "data-s": "warn" } "{need} need you" }
                    }
                }
                div { class: "oxstd-log", role: "log", "aria-label": "Conversation",
                    if loading { {skel(7)} }
                    else if list.is_empty() && shown.is_none() {
                        {empty("Start with the orchestrator", "Ask for an outcome. It plans the tasks, picks the models, runs them and checks in here.")}
                    }
                    for (i, it) in list.into_iter().enumerate() {
                        {
                            match it {
                                ThreadItem::Me { text, at } => rsx! {
                                    div { key: "{i}", class: "oxstd-me", p { "{text}" } span { class: "oxscn-row-a", "{at}" } }
                                },
                                ThreadItem::Agent { text, at } => rsx! {
                                    div { key: "{i}", class: "oxstd-ag", span { class: "oxscn-mono oxscn-mono-s", "C" }
                                        div { p { "{text}" } span { class: "oxscn-row-a", "Conductor · {at}" } } }
                                },
                                ThreadItem::Tools(calls) => {
                                    let o = open_tools().contains(&i);
                                    let line = tools_line(&calls);
                                    rsx! { ToolFold { key: "{i}", calls, line, open: o, on_toggle: move |_| { let mut v = open_tools.write(); if let Some(p) = v.iter().position(|x| *x == i) { v.remove(p); } else { v.push(i); } } } }
                                },
                                ThreadItem::Task { id, name, step, done, of } => rsx! {
                                    div { key: "{i}", class: "oxstd-card",
                                        div { class: "oxstd-tt", span { class: "oxscn-row-t", "{name}" } span { class: "oxscn-row-a", "{done} of {of} steps" } }
                                        p { class: "oxsag-p oxsag-mute", "{step}" }
                                        div { class: "oxstd-prog", role: "progressbar", "aria-label": "{name} steps", "aria-valuenow": "{done}", "aria-valuemin": "0", "aria-valuemax": "{of}",
                                            span { style: "width:{u32::from(done) * 100 / u32::from(of.max(1))}%" } }
                                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { wtab.set("tasks"); filter.set(None); task_open.set(Some(id)); said.set(format!("{name} open in Tasks")); }, "Open in Tasks" }
                                    }
                                },
                                ThreadItem::Files(fs) => rsx! {
                                    div { key: "{i}", class: "oxstd-files",
                                        for (j, f) in fs.into_iter().enumerate() {
                                            button { key: "{j}", r#type: "button", class: "oxscn-chip oxscn-chip-b",
                                                onclick: move |_| { wtab.set("files"); file.set(ARTIFACTS.iter().position(|(n, _)| *n == f).unwrap_or(0)); }, {ico("file")} "{f}" }
                                        }
                                    }
                                },
                                ThreadItem::Away { hours, what } => rsx! {
                                    div { key: "{i}", class: "oxstd-away", role: "separator", span { "While you were away · {hours}h" } p { "{what}" } }
                                },
                            }
                        }
                    }
                    if sample {
                        div { class: "oxstd-up", "data-done": "{up >= 100}",
                            {ico("clip")}
                            span { class: "oxscn-row-t", "venue-floorplan.pdf" }
                            span { class: "oxscn-row-a", if up >= 100 { "Uploaded" } else { "{up}% · slow link, still going" } }
                            div { class: "oxstd-prog", role: "progressbar", "aria-label": "Upload", "aria-valuenow": "{up}", "aria-valuemin": "0", "aria-valuemax": "100", span { style: "width:{up}%" } }
                        }
                    }
                    if let Some((t, down, _)) = shown.clone() {
                        div { class: "oxstd-ag", span { class: "oxscn-mono oxscn-mono-s", "C" }
                            div {
                                if t.is_empty() { p { class: "oxstd-think", span { class: "oxscn-pulse" } "Thinking…" } } else { p { "{t}" } }
                                if down {
                                    div { class: "oxstd-drop", role: "status",
                                        span { "Connection dropped – it keeps going and picks up here." }
                                        button { r#type: "button", class: "oxscn-link",
                                            onclick: move |_| {
                                                let cur = stream.peek().clone();
                                                if let Some((tx, at, d, _)) = cur {
                                                    let after = since(now, at).saturating_sub(600 + d as u64 * 70);
                                                    stream.set(Some((tx, at, d, after)));
                                                }
                                            }, "Retry now" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxstd-cmp",
                    ScreenComposer { text, model: false, busy, placeholder: "Tell the orchestrator what you need".to_string(),
                        on_send: move |t: String| {
                            items.write().push(ThreadItem::Me { text: t.clone(), at: "now" });
                            let n = *sends.peek();
                            sends.set(n + 1);
                            // The first reply drops its connection for a moment.
                            let (d, dm) = if n == 0 { (6, 2_400) } else { (0, 0) };
                            stream.set(Some((thread_reply(&t), now.0, d, dm)));
                            let mut tx = text;
                            tx.set(String::new());
                        },
                        on_stop: move |_| {
                            let cur = stream.peek().clone();
                            if let Some((t, at, d, dm)) = cur {
                                let words = t.split_whitespace().count();
                                let (n, _) = stream_at(words, since(now, at), d, dm);
                                let cut = format!("{} — stopped", first_words(&t, n));
                                finish(items, stream, Some(cut));
                                said.set("Stopped".into());
                            }
                        },
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

/// Calls that ran side by side, folded into one line that opens into a row per call.
#[component]
fn ToolFold(calls: Vec<ToolRun>, line: String, open: bool, on_toggle: EventHandler<()>) -> Element {
    rsx! {
        div { class: "oxstd-tools",
            button { r#type: "button", class: "oxstd-tb", "aria-expanded": "{open}", onclick: move |_| on_toggle.call(()),
                {ico("term")} span { "{line}" } {chevron(open)} }
            if open {
                ul { class: "oxstd-calls",
                    for (i, c) in calls.iter().enumerate() {
                        li { key: "{i}", span { class: "oxscn-row-t", "{c.label}" } span { class: "oxscn-row-a", "{dur(c.ms)}" } }
                    }
                }
            }
        }
    }
}
