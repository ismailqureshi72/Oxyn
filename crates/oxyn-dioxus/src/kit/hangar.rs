//! Screens S16 (Agent apps IV): **Hangar**, an agent manager. Conversations by workspace with an
//! Inbox of those waiting on review; the open conversation's task list and artifacts; Planning /
//! Fast and a model in the composer; a Surfaces pane with the Editor, the Browser the agent
//! drives and the Plan artifact, whose lines take comments before Proceed. Rules in
//! `oxyn_core::kit::hangar`; styles in `hangar.css` (`oxs16h-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::kit::hangar::{
    HG_APP, HG_ARTIFACTS, HG_BROWSE, HG_CART, HG_CODE, HG_FILE, HG_MODELS, HG_MODES, HG_PLAN,
    HG_PROMPT, HG_STATES, HG_SURFACES, HG_TASKS, HG_URL, HG_WORKSPACES, HangarData, HgNote,
    ScreenState, add_conv, add_note, cart_total, hg_split, price, prompt_error, review_label,
    tasks_done, waiting,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Hangar (for a host's `ScreenStage { states }`).
pub fn s16_hangar_states() -> Vec<(&'static str, &'static str)> {
    HG_STATES.to_vec()
}

fn hg_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            path { d: "M2.5 16.5V9l7.5-5.5L17.5 9v7.5", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linejoin: "round" }
            path { d: "M6.5 16.5v-5h7v5", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linejoin: "round" }
        }
    }
}

fn status_word(s: &str) -> &'static str {
    match s {
        "run" => "Working",
        "wait" => "Needs review",
        "ok" => "Done",
        _ => "Idle",
    }
}

/// The Surfaces pane: Editor, Browser and the Plan artifact with line comments.
#[component]
fn Surfaces(
    tab: Signal<usize>,
    notes: Signal<Vec<HgNote>>,
    sent: Signal<bool>,
    loading: bool,
    empty: bool,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut notes = notes;
    let mut sent = sent;
    let mut line = use_signal(|| None::<usize>);
    let mut draft = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("layers")} p { "The editor, the browser and the agent's artifacts open here." } }
        };
    }
    let n = notes();
    let total = cart_total(&HG_CART);
    rsx! {
        div { class: "oxs16h-tabs", role: "tablist", "aria-label": "Surfaces",
            onkeydown: move |e: KeyboardEvent| {
                let now = *tab.peek();
                match e.key() { Key::ArrowRight => tab.set((now + 1) % 3), Key::ArrowLeft => tab.set((now + 2) % 3), _ => {} }
            },
            for (i, (k, l)) in HG_SURFACES.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16h-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i),
                    {nico(match k { "editor" => "file", "browser" => "globe", _ => "doc" })} "{l}"
                    if k == "artifact" && !n.is_empty() { span { class: "oxs16h-cnt", "{n.len()}" } }
                }
            }
        }
        div { class: "oxs16h-body", role: "tabpanel", "aria-label": "{HG_SURFACES[t].1}",
            if t == 0 {
                div { class: "oxs16h-fh", {nico("file")} span { class: "oxs10-nl oxs16h-mono", "{HG_FILE}" } span { class: "oxs16h-add", "+5" } }
                pre { class: "oxs16h-code", "aria-label": "{HG_FILE}",
                    for (i, (s, l)) in HG_CODE.iter().copied().enumerate() {
                        span { key: "{i}", class: "oxs16h-cl", "data-s": "{s}", span { class: "oxs16h-ln", "{i + 12}" } span { class: "oxs16h-sg", "{s}" } "{l}" }
                    }
                }
            } else if t == 1 {
                div { class: "oxs16h-url",
                    {nico("lock")} span { class: "oxs10-nl oxs16h-mono", "{HG_URL}" }
                    {pill("Agent driving", "run")}
                }
                div { class: "oxs16h-site", "aria-label": "Checkout preview at 390 px",
                    div { class: "oxs16h-phone",
                        p { class: "oxs16h-ph", "Your bag" }
                        ul { class: "oxs16h-items",
                            for (i, (name, c)) in HG_CART.iter().copied().enumerate() {
                                li { key: "{i}", span { class: "oxs10-nl", "{name}" } span { class: "oxs16h-num", "{price(c)}" } }
                            }
                        }
                        div { class: "oxs16h-foot",
                            span { class: "oxs16h-tot", "Total " b { "{price(total)}" } }
                            span { class: "oxs16h-pay", "Pay" }
                        }
                    }
                }
                p { class: "oxs10-sh", "Agent actions" }
                ol { class: "oxs16h-log", "aria-label": "Agent actions",
                    for (i, (g, v, o, at)) in HG_BROWSE.iter().copied().enumerate() {
                        li { key: "{i}", {nico(g)} b { "{v}" } span { class: "oxs10-nl", "{o}" } span { class: "oxs16h-at", "{at}" } }
                    }
                }
            } else {
                div { class: "oxs16h-doc",
                    p { class: "oxs16h-dt", "Implementation plan" }
                    p { class: "oxs16h-dsub", "Pick a line to comment on it; the agent reads every comment before it goes on." }
                    ol { class: "oxs16h-pl", "aria-label": "Plan lines",
                        for (i, l) in HG_PLAN.iter().copied().enumerate() {
                            li { key: "{i}",
                                button { r#type: "button", class: "oxs16h-pline", "aria-pressed": "{line() == Some(i)}", "aria-label": "Comment on line {i + 1}: {l}",
                                    disabled: sent(),
                                    onclick: move |_| { line.set(Some(i)); err.set(None); },
                                    span { class: "oxs16h-pn", "{i + 1}" } span { class: "oxs16h-pt", "{l}" }
                                }
                                for (j, x) in n.iter().filter(|x| x.line == i).cloned().enumerate() {
                                    p { key: "{j}", class: "oxs16h-note", span { class: "oxs10-av oxs16h-nav", "aria-hidden": "true", "RV" } span { class: "oxs16h-nt", "{x.text}" } }
                                }
                                if line() == Some(i) {
                                    div { class: "oxs16h-box", role: "group", "aria-label": "Comment on line {i + 1}",
                                        textarea { class: "oxs16h-ta", rows: "2", "aria-label": "Comment", placeholder: "What should the agent change?", value: "{draft}",
                                            "aria-invalid": "{err().is_some()}",
                                            oninput: move |e| { draft.set(e.value()); err.set(None); } }
                                        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
                                        div { class: "oxs16h-bb",
                                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { line.set(None); draft.set(String::new()); }, "Cancel" }
                                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri",
                                                onclick: move |_| {
                                                    let d = draft.peek().clone();
                                                    match add_note(&mut notes.write(), i, &d) {
                                                        Ok(()) => { line.set(None); draft.set(String::new()); said.set(format!("Comment added to line {}", i + 1)); }
                                                        Err(e) => { err.set(Some(e)); said.set(e.to_string()); }
                                                    }
                                                },
                                                "Comment" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxs16h-rev",
                        if sent() {
                            span { class: "oxs16h-ok", {nico("check")} if n.is_empty() { "Approved · the agent goes on" } else { "Sent · the agent is revising the plan" } }
                        } else {
                            span { class: "oxs16h-ncount", if n.is_empty() { "No comments" } else { "{n.len()} on the plan" } }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri",
                                onclick: move |_| { let k = notes.peek().len(); sent.set(true); said.set(if k == 0 { "Plan approved".to_string() } else { format!("Sent {k} comments to the agent") }); },
                                "{review_label(n.len())}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HangarApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<HangarData>,
) -> Element {
    let (st, own) = hg_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| HangarData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.convs);
    let n0 = d0.notes.clone();
    let notes = use_signal(move || n0);
    let sent = use_signal(|| false);
    let first = d0.convs.first().map(|c| c.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let mut tab = use_signal(move || match own {
        "browser" => 1,
        "artifact" => 2,
        _ => 0,
    });
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
    let mut pane = use_signal(|| true);
    let mut pnar = use_signal(move || !own.is_empty());
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let mut mode = use_signal(|| 0usize);
    let mut inbox = use_signal(|| false);
    let mut q = use_signal(String::new);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let rl = side() == "rail";
    let waits = waiting(&all);
    let mut submit = move || {
        let p = text.peek().clone();
        match add_conv(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set(format!("New conversation in {}", HG_MODES[*mode.peek()].0));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let modes: Vec<(String, String, String)> = HG_MODES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let models = opts2(&HG_MODELS);
    let mk = HG_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let ql = q().trim().to_lowercase();
    let shown: Vec<_> = all
        .iter()
        .filter(|c| !inbox() || c.status == "wait")
        .filter(|c| ql.is_empty() || c.title.to_lowercase().contains(&ql))
        .cloned()
        .collect();

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{HG_APP}",
            {s11_top(HG_APP, hg_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New conversation",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New conversation".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New conversation" } span { class: "oxs10-kbd", "⌘N" } } }
            button { r#type: "button", class: "oxs10-nav", "aria-pressed": "{inbox()}", "data-on": "{inbox()}", title: "Inbox",
                onclick: move |_| { inbox.toggle(); said.set(if *inbox.peek() { "Inbox: conversations that need review".to_string() } else { "All conversations".to_string() }); },
                {nico("inbox")} if !rl { span { class: "oxs10-nl", "Inbox" } if waits > 0 { span { class: "oxs16h-cnt", "aria-label": "{waits} need review", "{waits}" } } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search conversations", "aria-label": "Search conversations", value: "{q}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No conversations yet. Open a workspace and ask the agent." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", if inbox() { "Nothing waits on you." } else { "No conversations match." } } }
                    else {
                        for ws in HG_WORKSPACES {
                            {
                                let inner: Vec<_> = shown.iter().filter(|c| c.workspace == ws).cloned().collect();
                                rsx! {
                                    if !inner.is_empty() {
                                        div { key: "{ws}",
                                            p { class: "oxs10-sh oxs16h-ws", {nico("folder")} span { class: "oxs10-ell", "{ws}" } }
                                            ul { class: "oxs10-ul",
                                                for c in inner {
                                                    {
                                                        let id = c.id;
                                                        let on = id == oid;
                                                        let t2 = c.title.clone();
                                                        let day = c.at.get(..10).unwrap_or(c.at);
                                                        rsx! {
                                                            li { key: "{id}",
                                                                button { r#type: "button", class: "oxs10-row oxs16h-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                                    "aria-label": "{c.title}, {status_word(c.status)}",
                                                                    onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                                    match c.status {
                                                                        "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } },
                                                                        "wait" => rsx! { span { class: "oxs16h-dot", "data-t": "wait" } },
                                                                        "ok" => rsx! { span { class: "oxs16h-k", "data-t": "ok", {nico("check")} } },
                                                                        _ => rsx! { span { class: "oxs16h-k", {nico("chat")} } },
                                                                    }
                                                                    span { class: "oxs10-nl", "{c.title}" }
                                                                    span { class: "oxs10-ago oxs16h-day", "{day}" }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New conversation" } }
        if let Some(c) = &cur {
            {pill(status_word(c.status), match c.status { "run" => "run", "wait" => "wait", "ok" => "ok", _ => "" })}
        }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Ask the agent to build, fix or check something",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add context", onclick: move |_| said.set("Add context: files, images, a URL".to_string()), {nico("plus")} }
                Pick { label: "Mode".to_string(), lead: rsx! { {nico(if mode() == 0 { "list" } else { "bolt" })} }, items: modes, cur: mode(), up: true,
                    on_pick: move |i: usize| { mode.set(i); said.set(format!("Mode: {}", HG_MODES[i].0)); } }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", HG_MODELS[i].0)); } }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let (done, of) = tasks_done(&HG_TASKS);
    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{HG_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        div { class: "oxs16h-card", role: "group", "aria-label": "Task list, {done} of {of} done",
                            div { class: "oxs16h-ch", {nico("tasks")} b { "Task list" } span { class: "oxs10-grow" } span { class: "oxs16h-num", "{done} of {of}" } }
                            ol { class: "oxs16h-tasks",
                                for (i, (l, s)) in HG_TASKS.iter().copied().enumerate() {
                                    li { key: "{i}", "data-s": "{s}",
                                        match s { "ok" => rsx! { span { class: "oxs16h-k", "data-t": "ok", {nico("check")} } }, "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } }, _ => rsx! { span { class: "oxs16h-ring" } } }
                                        span { class: "oxs10-nl", "{l}" }
                                        span { class: "oxs16h-sr", if s == "ok" { "done" } else if s == "run" { "in progress" } else { "next" } }
                                    }
                                }
                            }
                        }
                        p { class: "oxs10-sh", "Artifacts" }
                        div { class: "oxs16h-arts",
                            for (k, t, g, l) in HG_ARTIFACTS.iter().copied() {
                                button { key: "{k}", r#type: "button", class: "oxs16h-art", "aria-label": "Open {t}",
                                    onclick: move |_| { tab.set(if k == "plan" { 2 } else { 1 }); pane.set(true); pnar.set(true); said.set(format!("Opened {t}")); },
                                    span { class: "oxs16h-ai", {nico(g)} }
                                    span { class: "oxs16h-atx", b { class: "oxs10-ell", "{t}" } span { class: "oxs10-ell", "{l}" } }
                                }
                            }
                        }
                        div { class: "oxs10-agent",
                            p { if sent() { "Got it. I'm updating the plan from your comments, then I'll move the total and Pay into the footer." } else { "I wrote a plan for the sticky footer. Read it, comment on any line, then Proceed and I'll check it in the browser." } }
                        }
                    }
                    if c.status == "run" {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.status = "ok"; }
                            said.set("Conversation finished".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", "Working in the editor" } }
                    } else if !sample {
                        div { class: "oxs10-agent", p { "Done. The changes are in the Editor, and I checked them in the Browser." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {hg_mark(34)} }
                h1 { class: "oxs10-h1", "What should the agent build?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Add a dark theme to the settings page", "Find why the cart test is flaky", "Check the sign-up form on a phone"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16h-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel =
        rsx! { Surfaces { tab, notes, sent, loading, empty: all.is_empty() && !loading, said } };
    rsx! { Frame { app: "hg", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Surfaces".to_string() } }
}

/// Hangar: an agent manager as a desktop app. Conversations by workspace with an Inbox of
/// those waiting on review; the task list and artifacts in the conversation; Planning / Fast and
/// a Model in the composer; a Surfaces pane with Editor, Browser and the Plan artifact (comment
/// on a line, then Proceed or send the comments). States `artifact` and `browser`.
#[component]
pub fn HangarScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `artifact` or `browser`).
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
    /// Your data. `None` shows the sample for `state`; start from `HangarData::default()`.
    #[props(default)]
    data: Option<HangarData>,
) -> Element {
    rsx! { HangarApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
