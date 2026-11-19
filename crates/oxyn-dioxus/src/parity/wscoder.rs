//! Screens S11 (Agent apps III): **Workspace Coder**, a desktop agent app in the
//! chat-plus-computer pattern. Chats in the sidebar; the agent's todo list and tool log in the
//! conversation; a Workspace pane (Files, Terminal, Browser, Preview) showing its own computer; a
//! deliverable to preview at desktop or phone width and publish. Rules in
//! `oxyn_core::parity::wscoder`; styles in `wscoder.css` (`oxs11w-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::parity::wscoder::{
    ScreenState, WC_APP, WC_AT, WC_DONE, WC_INTRO, WC_LOG, WC_MODELS, WC_MODES, WC_PROMPT,
    WC_STATES, WC_TODOS, WS_BROWSER, WS_TABS, WS_TERM, WorkspaceCoderData, WsFile, add_chat,
    cheapest, log_upto, preview_rows, prompt_error, publish_url, tab_step, todo_count, wc_split,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{ico, nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Workspace Coder (for a host's `ScreenStage { states }`).
pub fn s11_coder_states() -> Vec<(&'static str, &'static str)> {
    WC_STATES.to_vec()
}

fn wc_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "2", y: "3", width: "16", height: "11", rx: "2.5", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M7 17h6M10 14v3", stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round" }
            path { d: "M7.5 7l-1.8 1.6L7.5 10.2M12.5 7l1.8 1.6-1.8 1.6", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round" }
        }
    }
}

/// The Workspace pane: the agent's computer, as tabs.
#[component]
fn Workspace(
    tab: Signal<usize>,
    files: Vec<WsFile>,
    loading: bool,
    running: bool,
    title: String,
    said: Signal<String>,
    file0: usize,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut fi = use_signal(move || file0);
    let mut phone = use_signal(|| false);
    let mut published = use_signal(|| false);
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if files.is_empty() {
        return rsx! {
            div { class: "oxs10-pempty", {nico("laptop")} p { "The agent's computer shows up here: the files it writes, the commands it runs, the pages it reads and a preview of what it builds." } }
        };
    }
    let rows = preview_rows(&files);
    let low = cheapest(&rows);
    let max = rows.iter().map(|r| r.1).max().unwrap_or(1).max(1);
    let url = publish_url(&title);
    let url2 = url.clone();
    let f = files.get(fi()).cloned().unwrap_or_else(|| files[0].clone());
    rsx! {
        div { class: "oxs11w-ph",
            b { "Workspace" }
            span { class: "oxs11w-live", "data-on": "{running}", if running { span { class: "oxs10-spin oxs10-spin-s" } "Writing app.js" } else { "Idle" } }
        }
        div { class: "oxs11w-tabs", role: "tablist", "aria-label": "Workspace",
            onkeydown: move |e: KeyboardEvent| { let now = *tab.peek(); match e.key() { Key::ArrowRight => tab.set(tab_step(now, true)), Key::ArrowLeft => tab.set(tab_step(now, false)), _ => {} } },
            for (i, (k, l)) in WS_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs11w-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}" }
            }
        }
        div { class: "oxs11w-body", role: "tabpanel", "aria-label": "{WS_TABS[t].1}",
            match WS_TABS[t].0 {
                "files" => rsx! {
                    div { class: "oxs11w-files",
                        ul { class: "oxs11w-tree", "aria-label": "Files",
                            for (i, x) in files.iter().enumerate() {
                                li { key: "{x.path}",
                                    button { r#type: "button", class: "oxs11w-fi", "aria-current": if i == fi() { "true" } else { "false" }, "data-on": "{i == fi()}",
                                        onclick: move |_| fi.set(i), {nico("file")} span { class: "oxs10-nl", "{x.path}" } }
                                }
                            }
                        }
                        div { class: "oxs11w-code", "aria-label": "{f.path}",
                            div { class: "oxs11w-fh", span { class: "oxs10-nl", "{f.path}" } span { class: "oxs10-ago", "{f.lines.len()} lines" } }
                            pre { class: "oxs11w-pre",
                                for (n, l) in f.lines.iter().enumerate() {
                                    span { key: "{n}", class: "oxs11w-ln", span { class: "oxs11w-no", "{n + 1}" } span { class: "oxs11w-lt", "{l}" } }
                                }
                            }
                        }
                    }
                },
                "terminal" => rsx! {
                    pre { class: "oxs11w-term", "aria-label": "Terminal output",
                        for (i, (cmd, l)) in WS_TERM.iter().copied().enumerate() {
                            span { key: "{i}", class: "oxs11w-tl", "data-cmd": "{cmd}", if cmd { "$ " } "{l}" }
                        }
                        if running { span { class: "oxs11w-tl oxs11w-cur", "▍" } }
                    }
                },
                "browser" => rsx! {
                    div { class: "oxs11w-br",
                        div { class: "oxs11w-addr", {nico("globe")} span { class: "oxs10-nl", "{WS_BROWSER.0}" } }
                        ul { class: "oxs11w-res",
                            for (i, (h, site, line)) in WS_BROWSER.1.iter().copied().enumerate() {
                                li { key: "{i}", span { class: "oxs11w-site", "{site}" } b { "{h}" } p { "{line}" } }
                            }
                        }
                    }
                },
                _ => rsx! {
                    div { class: "oxs11w-pv",
                        div { class: "oxs11w-pvbar",
                            div { class: "oxs11w-seg", role: "group", "aria-label": "Preview width",
                                button { r#type: "button", "aria-pressed": "{!phone()}", onclick: move |_| phone.set(false), {nico("laptop")} span { "Desktop" } }
                                button { r#type: "button", "aria-pressed": "{phone()}", onclick: move |_| phone.set(true), {nico("phone")} span { "Phone" } }
                            }
                            span { class: "oxs10-grow" }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", disabled: running || published(),
                                onclick: move |_| { published.set(true); said.set(format!("Published at {url2}")); },
                                if published() { "Published" } else { "Publish" } }
                        }
                        if published() { p { class: "oxs11w-pub", {nico("link")} span { class: "oxs10-nl", "{url}" } } }
                        div { class: "oxs11w-page", "data-phone": "{phone()}", "aria-label": "Preview of {title}",
                            p { class: "oxs11w-pt", "A month in three cities" }
                            p { class: "oxs11w-ps", "One person, typical costs in €" }
                            div { class: "oxs11w-bars", role: "img", "aria-label": "Monthly totals by city",
                                for (i, (c, v)) in rows.iter().cloned().enumerate() {
                                    {
                                        let w = format!("width:{}%", v * 100 / max);
                                        rsx! {
                                            div { key: "{i}", class: "oxs11w-bar", "data-low": "{Some(i) == low}",
                                                span { class: "oxs11w-bl", "{c}" }
                                                span { class: "oxs11w-bt", span { class: "oxs11w-bf", style: "{w}" } }
                                                span { class: "oxs11w-bv", "€{v}" }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(k) = low { p { class: "oxs11w-ps", "Cheapest: {rows[k].0}" } }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn WorkspaceCoderApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<WorkspaceCoderData>,
) -> Element {
    let (st, own) = wc_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| WorkspaceCoderData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let finished0 = !own.is_empty();
    let d1 = d0.clone();
    let mut list = use_signal(move || {
        let mut l = d1.chats;
        if finished0 && let Some(c) = l.first_mut() {
            c.running = false;
        }
        l
    });
    let first = d0.chats.first().map(|c| c.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let tab0 = match own {
        "preview" => 3,
        "files" => 0,
        _ => 0,
    };
    let tab = use_signal(move || tab0);
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
    let mut pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let mut mode = use_signal(|| 2usize);
    let mut think = use_signal(|| true);
    let mut todo_open = use_signal(|| true);
    let mut q = use_signal(String::new);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let running = cur.as_ref().is_some_and(|c| c.running);
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        let m = *mode.peek();
        let r = add_chat(&mut list.write(), &p, m);
        match r {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set(format!("New chat in {} mode", WC_MODES[m].0));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let modes: Vec<(String, String, String)> = WC_MODES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let models = opts2(&WC_MODELS);
    let mk = WC_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let ql = q().trim().to_lowercase();
    let shown: Vec<_> = all
        .iter()
        .filter(|c| ql.is_empty() || c.title.to_lowercase().contains(&ql))
        .cloned()
        .collect();
    let new_on = oid == 0;
    let title = d0.deliverable.clone();

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{WC_APP}",
            {s11_top(WC_APP, wc_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{new_on}", "aria-current": if new_on { "page" } else { "false" }, title: "New chat",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New chat".to_string()); },
                {nico("pen")} if !rl { span { class: "oxs10-nl", "New chat" } span { class: "oxs10-kbd", "⌘K" } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search chats", "aria-label": "Search chats", value: "{q}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No chats yet. Ask for something to build." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", "No chats match." } }
                    else {
                        for day in ["Today", "Earlier"] {
                            {
                                let inner: Vec<_> = shown.iter().filter(|c| c.day == day).cloned().collect();
                                rsx! {
                                    if !inner.is_empty() {
                                        div { key: "{day}",
                                            p { class: "oxs10-sh", "{day}" }
                                            ul { class: "oxs10-ul",
                                                for c in inner {
                                                    {
                                                        let id = c.id;
                                                        let on = id == oid;
                                                        let t2 = c.title.clone();
                                                        rsx! {
                                                            li { key: "{id}",
                                                                button { r#type: "button", class: "oxs10-row oxs11w-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                                    onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                                    if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs11w-mg", {nico(WC_MODES[c.mode].2)} } }
                                                                    span { class: "oxs10-nl", "{c.title}" }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New chat" } }
        if let Some(c) = &cur { if c.running { {pill("Working", "run")} } else { {pill(WC_MODES[c.mode].0, "")} } }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Ask anything, or describe what to build",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Attach files", onclick: move |_| said.set("Attach: PDF, sheets, images or code".to_string()), {nico("clip")} }
                Pick { label: "Mode".to_string(), lead: rsx! { {nico(WC_MODES[mode()].2)} }, items: modes, cur: mode(), up: true,
                    on_pick: move |i: usize| { mode.set(i); said.set(format!("Mode: {}", WC_MODES[i].0)); } }
                button { r#type: "button", class: "oxs10-chip oxs11w-think", "aria-pressed": "{think()}", onclick: move |_| { think.toggle(); said.set(if *think.peek() { "Thinking on" } else { "Thinking off" }.to_string()); },
                    {nico("brain")} span { class: "oxs10-ell", "Thinking" } }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", WC_MODELS[i].0)); } }
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
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        let done = !c.running;
        let (dn, tn) = todo_count(WC_AT, done);
        let shown_log = log_upto(WC_AT, done);
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{WC_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        div { class: "oxs10-agent", p { "{WC_INTRO}" } }
                        div { class: "oxs11w-todo", role: "group", "aria-label": "Progress",
                            button { r#type: "button", class: "oxs11w-th", "aria-expanded": "{todo_open()}", onclick: move |_| todo_open.toggle(),
                                {nico("tasks")} b { "Progress" } span { class: "oxs10-grow" } span { class: "oxs10-ago", "{dn} of {tn}" }
                                span { class: "oxs10-car", "data-open": "{todo_open()}", {nico("caret")} } }
                            if todo_open() {
                                ol { class: "oxs11w-tl2",
                                    for (i, t) in WC_TODOS.iter().copied().enumerate() {
                                        {
                                            let s = if done || i < WC_AT { "done" } else if i == WC_AT { "run" } else { "todo" };
                                            rsx! {
                                                li { key: "{i}", class: "oxs11w-ti", "data-s": "{s}",
                                                    match s { "done" => rsx! { span { class: "oxs10-ok", {nico("check")} } }, "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } }, _ => rsx! { span { class: "oxs10-ring" } } }
                                                    span { class: "oxs10-nl", "{t}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ol { class: "oxs10-work", "aria-label": "Tool log",
                            for (i, (g, v, o)) in WC_LOG.iter().copied().take(shown_log).enumerate() {
                                li { key: "{i}", class: "oxs10-wi", {nico(g)} span { class: "oxs11w-v", "{v}" } span { class: "oxs10-nl oxs11w-o", "{o}" } }
                            }
                        }
                    }
                    if c.running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.running = false; }
                            said.set("Finished · preview ready".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if sample { "Building the page · step {WC_AT + 1} of {tn}" } else { "Thinking" } } }
                    } else if sample {
                        div { class: "oxs10-agent",
                            p { "{WC_DONE}" }
                            div { class: "oxs11w-deliv",
                                span { class: "oxs11w-thumb", "aria-hidden": "true", span {} span {} span {} }
                                span { class: "oxs10-s2", b { class: "oxs10-ell", "{title}" } span { class: "oxs10-ssub oxs10-ell", "Website · 5 files" } }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { let mut t = tab; t.set(3); pnar.set(true); said.set("Preview".to_string()); }, "Preview" }
                            }
                        }
                    } else {
                        div { class: "oxs10-agent", p { "Here's a first pass. Ask a follow-up, or switch to Workspace mode to have me build files." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {wc_mark(34)} }
                h1 { class: "oxs10-h1", "What can I build for you?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Build a budget page with a chart", "Turn my notes into slides", "Research and compare three tools"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs11w-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {ico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let file0 = if own == "files" { 1 } else { 0 };
    let panel =
        rsx! { Workspace { tab, files: d0.files.clone(), loading, running, title, said, file0 } };
    rsx! { Frame { app: "wc", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Workspace".to_string() } }
}

/// Workspace Coder: the chat-plus-computer desktop agent app. Chats in the sidebar; the agent's
/// todo list and tool log; a Workspace pane with Files, Terminal, Browser and Preview; a deliverable
/// to preview at desktop or phone width and publish. Mode (Chat, Researcher, Workspace), Thinking
/// and model in the composer. States `preview` and `files`.
#[component]
pub fn WorkspaceCoderScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `preview` or `files`).
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
    /// Your data. `None` shows the sample for `state`; start from `WorkspaceCoderData::default()`
    /// and replace what you have.
    #[props(default)]
    data: Option<WorkspaceCoderData>,
) -> Element {
    rsx! { WorkspaceCoderApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
