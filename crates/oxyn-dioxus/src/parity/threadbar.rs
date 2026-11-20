//! Screens S16 (Agent apps IV): **Threadbar**, a code editor with an agent panel. Threads grouped
//! by project (agent and terminal threads); the open thread's tool calls with an edit to keep or
//! reject; a composer with Profile, Agent (outside agents too) and Model; a Project pane with
//! Files (git letters) and Git (stage, message, commit). Rules in
//! `oxyn_core::parity::threadbar`; styles in `threadbar.css` (`oxs16t-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::parity::tabstack::run_cmd;
use oxyn_core::parity::threadbar::{
    ScreenState, TB_AGENTS, TB_APP, TB_CALLS, TB_COMMIT, TB_DIFF, TB_DONE, TB_MODELS, TB_PROFILES,
    TB_PROJECTS, TB_PROMPT, TB_STATES, TB_TABS, TB_TERM, TB_TREE, ThreadbarData, add_thread,
    commit_error, prompt_error, staged_sum, tb_split,
};

use super::cmdstudio::{Frame, Pick, branch_ico, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Threadbar (for a host's `ScreenStage { states }`).
pub fn s16_threadbar_states() -> Vec<(&'static str, &'static str)> {
    TB_STATES.to_vec()
}

fn tb_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "2", y: "2.5", width: "16", height: "15", rx: "3", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M7.5 2.5v15", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M10.5 7h4.5M10.5 10h4.5M10.5 13h3", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round" }
        }
    }
}

/// The Project pane: Files (a tree with git letters) and Git (stage, message, commit).
#[component]
fn Project(
    tab: Signal<usize>,
    data: ThreadbarData,
    loading: bool,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let c0 = data.changes.clone();
    let mut ch = use_signal(move || c0);
    let mut msg = use_signal(|| TB_COMMIT.to_string());
    let mut err = use_signal(|| None::<&'static str>);
    let mut done = use_signal(|| None::<String>);
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if data.threads.is_empty() {
        return rsx! {
            div { class: "oxs10-pempty", {nico("folder")} p { "Open a folder to see its files and changes here." } }
        };
    }
    let list = ch();
    let (n, a, d) = staged_sum(&list);
    rsx! {
        div { class: "oxs16t-ph",
            b { "Project" }
            span { class: "oxs16t-br", {branch_ico()} span { class: "oxs10-ell", "{data.branch}" } }
        }
        div { class: "oxs16t-tabs", role: "tablist", "aria-label": "Project",
            onkeydown: move |e: KeyboardEvent| { if matches!(e.key(), Key::ArrowRight | Key::ArrowLeft) { let now = *tab.peek(); tab.set(1 - now); } },
            for (i, (k, l)) in TB_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16t-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}"
                    if k == "git" && !list.is_empty() { span { class: "oxs16t-cnt", "{list.len()}" } }
                }
            }
        }
        div { class: "oxs16t-body", role: "tabpanel", "aria-label": "{TB_TABS[t].1}",
            if t == 0 {
                ul { class: "oxs16t-tree", "aria-label": "Files",
                    for (i, (name, depth, g)) in TB_TREE.iter().copied().enumerate() {
                        {
                            let folder = TB_TREE.get(i + 1).is_some_and(|x| x.1 > depth);
                            let pad = format!("padding-left:{}px", 8 + depth * 16);
                            rsx! {
                                li { key: "{i}", class: "oxs16t-fi", style: "{pad}", "data-g": "{g}",
                                    if folder { {nico("folderopen")} } else { {nico("file")} }
                                    span { class: "oxs10-nl", "{name}" }
                                    if g != ' ' { span { class: "oxs16t-g", title: if g == 'M' { "Modified" } else { "Added" }, "{g}" } }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs16t-git",
                    p { class: "oxs10-sh", "Changes · {list.len()}" }
                    ul { class: "oxs16t-chs", "aria-label": "Changed files",
                        for (i, c) in list.iter().cloned().enumerate() {
                            li { key: "{c.path}",
                                label { class: "oxs16t-ch",
                                    input { r#type: "checkbox", checked: c.staged, "aria-label": "Stage {c.path}",
                                        onchange: move |_| { let mut w = ch.write(); w[i].staged = !w[i].staged; let s = w[i].staged; drop(w); err.set(None); said.set(format!("{} {}", if s { "Staged" } else { "Unstaged" }, c.path)); } }
                                    span { class: "oxs10-nl oxs16t-path", "{c.path}" }
                                    span { class: "oxs16t-add", "+{c.add}" }
                                    span { class: "oxs16t-del", "−{c.del}" }
                                }
                            }
                        }
                    }
                    p { class: "oxs16t-sum", "{n} staged · +{a} −{d}" }
                    textarea { class: "oxs16t-msg", rows: "2", "aria-label": "Commit message", "aria-invalid": "{err().is_some()}", value: "{msg}",
                        oninput: move |e| { msg.set(e.value()); err.set(None); } }
                    if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri oxs16t-commit",
                        onclick: move |_| {
                            let m = msg.peek().clone();
                            let bad = commit_error(&ch.peek(), &m);
                            match bad {
                                Some(e) => { err.set(Some(e)); said.set(e.to_string()); }
                                None => {
                                    ch.write().retain(|c| !c.staged);
                                    done.set(Some(m.clone()));
                                    said.set(format!("Committed: {m}"));
                                }
                            }
                        },
                        "Commit {n} file" if n != 1 { "s" } }
                    if let Some(m) = done() { p { class: "oxs16t-ok", {nico("check")} span { class: "oxs10-nl", "Committed “{m}”" } } }
                }
            }
        }
    }
}

#[component]
fn ThreadbarApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<ThreadbarData>,
) -> Element {
    let (st, own) = tb_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| ThreadbarData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.threads);
    let first = match own {
        "terminal" => d0.threads.iter().find(|t| t.terminal).map(|t| t.id),
        _ => d0.threads.first().map(|t| t.id),
    }
    .unwrap_or(0);
    let mut open = use_signal(move || first);
    let tab = use_signal(move || if own == "git" { 1 } else { 0 });
    let mut said = use_signal(String::new);
    let side = use_signal(move || {
        if hidden {
            "hidden"
        } else if rail {
            "rail"
        } else {
            "full"
        }
    });
    let mut drawer = use_signal(|| false);
    let pane = use_signal(|| true);
    let pnar = use_signal(move || own == "git");
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let mut profile = use_signal(|| 0usize);
    let mut agent = use_signal(|| 0usize);
    let mut kept = use_signal(|| None::<bool>);
    let mut term = use_signal(|| {
        TB_TERM
            .iter()
            .map(|(c, l)| (*c, l.to_string()))
            .collect::<Vec<_>>()
    });
    let mut cmd = use_signal(String::new);
    let mut q = use_signal(String::new);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let running = cur.as_ref().is_some_and(|c| c.running);
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        match add_thread(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set(format!("New thread with {}", TB_AGENTS[*agent.peek()].0));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let profiles: Vec<(String, String, String)> = TB_PROFILES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let agents: Vec<(String, String, String)> = TB_AGENTS
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), m.2.to_string()))
        .collect();
    let models = opts2(&TB_MODELS);
    let mk = TB_MODELS[model()].1;
    let ak = TB_AGENTS[agent()].2;
    let can = prompt_error(&text()).is_none();
    let ql = q().trim().to_lowercase();
    let shown: Vec<_> = all
        .iter()
        .filter(|c| ql.is_empty() || c.title.to_lowercase().contains(&ql))
        .cloned()
        .collect();

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{TB_APP}",
            {s11_top(TB_APP, tb_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New thread",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New thread".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New thread" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search threads", "aria-label": "Search threads", value: "{q}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No threads yet. Ask the agent, or open a terminal." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", "No threads match." } }
                    else {
                        for pj in TB_PROJECTS {
                            {
                                let inner: Vec<_> = shown.iter().filter(|c| c.project == pj).cloned().collect();
                                rsx! {
                                    if !inner.is_empty() {
                                        div { key: "{pj}",
                                            p { class: "oxs10-sh oxs16t-pj", {nico("folder")} span { class: "oxs10-ell", "{pj}" } }
                                            ul { class: "oxs10-ul",
                                                for c in inner {
                                                    {
                                                        let id = c.id;
                                                        let on = id == oid;
                                                        let t2 = c.title.clone();
                                                        let day = c.at.get(..10).unwrap_or(c.at);
                                                        rsx! {
                                                            li { key: "{id}",
                                                                button { r#type: "button", class: "oxs10-row oxs16t-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                                    onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                                    if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else if c.terminal { span { class: "oxs16t-k", {nico("term")} } } else { span { class: "oxs16t-k", {nico("chat")} } }
                                                                    span { class: "oxs10-nl", "{c.title}" }
                                                                    span { class: "oxs10-ago oxs16t-at", "{day}" }
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
            } else { span { class: "oxs10-grow" } }
            AcctFoot { rail: rl, said }
        }
    };

    let bar = rsx! {
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New thread" } }
        if let Some(c) = &cur {
            if c.running { {pill("Working", "run")} } else if c.terminal { {pill("Terminal", "")} } else { {pill(TB_PROFILES[profile()].0, "")} }
        }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Message the agent · @ to add files",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add context", onclick: move |_| said.set("Add context: files, symbols, threads".to_string()), {nico("plus")} }
                Pick { label: "Profile".to_string(), lead: rsx! { {nico(TB_PROFILES[profile()].2)} }, items: profiles, cur: profile(), up: true,
                    on_pick: move |i: usize| { profile.set(i); said.set(format!("Profile: {}", TB_PROFILES[i].0)); } }
                Pick { label: "Agent".to_string(), lead: rsx! { if ak.is_empty() { {tb_mark(14)} } else { span { class: "oxs10-mk", BrandIcon { name: ak.to_string(), size: 14.0, coloured: true } } } }, items: agents, cur: agent(), up: true,
                    on_pick: move |i: usize| { agent.set(i); said.set(format!("Agent: {}", TB_AGENTS[i].0)); } }
                span { class: "oxs10-grow" }
                if agent() == 0 {
                    Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                        on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", TB_MODELS[i].0)); } }
                }
                if running {
                    button { r#type: "button", class: "oxs10-stopb", "aria-label": "Stop",
                        onclick: move |_| { let id = *open.peek(); if let Some(c) = list.write().iter_mut().find(|c| c.id == id) { c.running = false; } said.set("Stopped".to_string()); }, span { class: "oxs10-sq" } }
                }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(c) = cur.clone().filter(|c| c.terminal) {
        let mut run = move || {
            let c2 = cmd.peek().trim().to_string();
            if c2.is_empty() {
                return;
            }
            let (out, code) = run_cmd(&c2);
            let mut w = term.write();
            w.push((true, c2.clone()));
            w.extend(out.into_iter().map(|l| (false, l)));
            drop(w);
            cmd.set(String::new());
            said.set(format!("{c2} exited {code}"));
        };
        rsx! {
            div { class: "oxs10-feed oxs16t-tfeed",
                pre { class: "oxs16t-term", "aria-label": "Terminal: {c.title}",
                    for (i, (isc, l)) in term().into_iter().enumerate() {
                        span { key: "{i}", class: "oxs16t-tl", "data-cmd": "{isc}", if isc { span { class: "oxs16t-ps", "ledger-api $ " } } "{l}" }
                    }
                }
            }
            div { class: "oxs10-foot",
                div { class: "oxs16t-cmd",
                    span { class: "oxs16t-ps", "$" }
                    input { class: "oxs16t-ci", r#type: "text", "aria-label": "Command", placeholder: "Type a command, then Enter", value: "{cmd}",
                        oninput: move |e| cmd.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); run(); } }
                }
            }
        }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{TB_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        ol { class: "oxs10-work", "aria-label": "Tool calls",
                            for (i, (g, v, o, x)) in TB_CALLS.iter().copied().enumerate() {
                                li { key: "{i}", class: "oxs10-wi", {nico(g)} span { class: "oxs16t-v", "{v}" } span { class: "oxs10-nl oxs16t-o", "{o}" } span { class: "oxs10-ago", "{x}" } }
                            }
                        }
                        div { class: "oxs16t-edit", role: "group", "aria-label": "Edit to src/webhooks/deliver.rs",
                            div { class: "oxs16t-eh", {nico("file")} span { class: "oxs10-nl oxs16t-o", "src/webhooks/deliver.rs" }
                                span { class: "oxs16t-add", "+18" } span { class: "oxs16t-del", "−4" } }
                            pre { class: "oxs16t-diff",
                                for (i, (s, l)) in TB_DIFF.iter().copied().enumerate() {
                                    span { key: "{i}", class: "oxs16t-dl", "data-s": "{s}", span { class: "oxs16t-sg", "{s}" } "{l}" }
                                }
                            }
                            div { class: "oxs16t-eb",
                                match kept() {
                                    Some(true) => rsx! { span { class: "oxs16t-ok", {nico("check")} "Kept" } },
                                    Some(false) => rsx! { span { class: "oxs16t-rej", {nico("undo")} "Rejected · the file is back as it was" } },
                                    None => rsx! {
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { kept.set(Some(false)); said.set("Edit rejected".to_string()); }, "Reject" }
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| { kept.set(Some(true)); said.set("Edit kept".to_string()); }, "Keep" }
                                    },
                                }
                            }
                        }
                    }
                    if c.running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.running = false; }
                            said.set("Thread finished".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if sample { "Running the webhook tests" } else { "Thinking" } } }
                    } else if sample {
                        div { class: "oxs10-agent", p { "{TB_DONE}" } }
                    } else {
                        div { class: "oxs10-agent", p { "Here's what I found. Ask a follow-up, or switch the profile to Write to let me change files." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {tb_mark(34)} }
                h1 { class: "oxs10-h1", "Start a thread" }
                div { class: "oxs10-under",
                    for (i, s) in ["Explain this project's structure", "Find and fix the failing test", "Write tests for the open file"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16t-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel = rsx! { Project { tab, data: d0.clone(), loading, said } };
    rsx! { Frame { app: "tb", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Project".to_string() } }
}

/// Threadbar: a code editor's agent panel as a desktop app. Threads by project (agent and
/// terminal threads); tool calls with an edit to Keep or Reject; Profile (Write, Ask, Minimal),
/// Agent (built in or an outside agent) and Model in the composer; a Project pane with Files and
/// Git (stage, message, Commit). States `terminal` and `git`.
#[component]
pub fn ThreadbarScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `terminal` or `git`).
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px (outside a stage).
    #[props(default = 900)]
    height: u32,
    /// Oxyn: open with the sidebar as a 56 px icon rail.
    #[props(default)]
    rail: bool,
    /// Oxyn: open with the sidebar hidden (the top bar brings it back).
    #[props(default)]
    sidebar_hidden: bool,
    /// Your data. `None` shows the sample for `state`; start from `ThreadbarData::default()`.
    #[props(default)]
    data: Option<ThreadbarData>,
) -> Element {
    rsx! { ThreadbarApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
