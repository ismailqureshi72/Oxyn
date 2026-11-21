//! Screens S16 (Agent apps IV): **Courier**, a coding-agent app built around threads. Every active
//! thread with All / Running / Needs you; tool lines and one edits card per turn (open each
//! file's diff); suggested replies above the composer; Model and Effort; a Pod pane, the
//! thread's own cloud machine, with Desktop, Ports and Changes (revert a file). Rules in
//! `oxyn_core::kit::courier`; styles in `courier.css` (`oxs16m-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::kit::courier::{
    CR_APP, CR_DONE, CR_EFFORT, CR_FILTERS, CR_MODELS, CR_POD, CR_PORTS, CR_PROMPT, CR_STATES,
    CR_SUGGEST, CR_TABS, CR_TOOLS, CourierData, CrFile, ScreenState, add_thread, cr_split,
    cr_totals, filter_threads, prompt_error, revert,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Courier (for a host's `ScreenStage { states }`).
pub fn s16_courier_states() -> Vec<(&'static str, &'static str)> {
    CR_STATES.to_vec()
}

fn cr_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            path { d: "M3 6.5l7-3.5 7 3.5v7L10 17l-7-3.5z", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linejoin: "round" }
            path { d: "M3 6.5l7 3.5 7-3.5M10 10v7", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linejoin: "round" }
        }
    }
}

fn status_word(s: &str) -> &'static str {
    match s {
        "run" => "Running",
        "wait" => "Needs you",
        "ok" => "Done",
        _ => "Idle",
    }
}

/// The Pod pane: Desktop, Ports and Changes.
#[component]
fn Pod(
    tab: Signal<usize>,
    files: Signal<Vec<CrFile>>,
    loading: bool,
    empty: bool,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut files = files;
    let mut ports = use_signal(|| CR_PORTS.iter().map(|p| p.2).collect::<Vec<bool>>());
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("box")} p { "Each thread gets a pod, a cloud machine of its own. Start a thread to open one." } }
        };
    }
    let fl = files();
    let (n, a, d) = cr_totals(&fl);
    let open_ports = ports();
    rsx! {
        div { class: "oxs16m-ph",
            span { class: "oxs16m-pi", {nico("box")} }
            b { class: "oxs10-ell", "{CR_POD.0}" }
            span { class: "oxs16m-meta", "{CR_POD.1}" }
            span { class: "oxs10-grow" }
            {pill("Running", "run")}
        }
        div { class: "oxs16m-tabs", role: "tablist", "aria-label": "Pod",
            onkeydown: move |e: KeyboardEvent| {
                let now = *tab.peek();
                match e.key() { Key::ArrowRight => tab.set((now + 1) % 3), Key::ArrowLeft => tab.set((now + 2) % 3), _ => {} }
            },
            for (i, (k, l)) in CR_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16m-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}"
                    if k == "changes" && n > 0 { span { class: "oxs16m-cnt", "{n}" } }
                }
            }
        }
        div { class: "oxs16m-body", role: "tabpanel", "aria-label": "{CR_TABS[t].1}",
            if t == 0 {
                div { class: "oxs16m-desk", "aria-label": "Pod desktop",
                    div { class: "oxs16m-win",
                        div { class: "oxs16m-wb", span { class: "oxs16m-dots", span {} span {} span {} } span { class: "oxs10-nl oxs16m-mono", "localhost:3000/p/trail-runner" } }
                        div { class: "oxs16m-page",
                            span { class: "oxs16m-img", {nico("image")} }
                            div { class: "oxs16m-pt",
                                b { "Trail runner" }
                                span { class: "oxs16m-price", "€129.00" }
                                span { class: "oxs16m-tag", {nico("bolt")} "Price from cache · 4 ms" }
                                span { class: "oxs16m-buy", "Add to bag" }
                            }
                        }
                    }
                    p { class: "oxs16m-cap", "The pod's browser, as the agent left it at 2026-11-19 09:31." }
                }
            } else if t == 1 {
                ul { class: "oxs16m-ports", "aria-label": "Forwarded ports",
                    for (i, (port, what, _)) in CR_PORTS.iter().copied().enumerate() {
                        li { key: "{port}",
                            span { class: "oxs16m-port", ":{port}" }
                            span { class: "oxs10-nl oxs16m-what", "{what}" }
                            if open_ports[i] {
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Open port {port}", onclick: move |_| said.set(format!("Opened port {port} in a browser tab")), {nico("link")} "Open" }
                            } else {
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Forward port {port}",
                                    onclick: move |_| { ports.write()[i] = true; said.set(format!("Forwarded port {port}")); }, "Forward" }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs16m-chs",
                    p { class: "oxs16m-sum", if n == 0 { "No changes on the pod" } else { "{n} files · " span { class: "oxs16m-add", "+{a}" } " " span { class: "oxs16m-del", "−{d}" } } }
                    ul { class: "oxs16m-fl", "aria-label": "Changed files",
                        for f in fl.iter().cloned() {
                            li { key: "{f.path}",
                                span { class: "oxs16m-fi", {nico("file")} }
                                span { class: "oxs10-nl oxs16m-mono", "{f.path}" }
                                span { class: "oxs16m-add", "+{f.add}" }
                                span { class: "oxs16m-del", "−{f.del}" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Revert {f.path}",
                                    onclick: move |_| { revert(&mut files.write(), f.path); said.set(format!("Reverted {}", f.path)); }, {nico("undo")} "Revert" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CourierApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<CourierData>,
) -> Element {
    let (st, own) = cr_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| CourierData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.threads);
    let f0 = d0.files.clone();
    let files = use_signal(move || f0);
    let first = d0.threads.first().map(|c| c.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let mut tab = use_signal(move || if own == "changes" { 2usize } else { 0 });
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
    let mut effort = use_signal(|| 1usize);
    let mut filt = use_signal(|| 0usize);
    let mut shown_diff = use_signal(|| None::<&'static str>);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        match add_thread(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                filt.set(0);
                said.set("New thread".to_string());
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let models = opts2(&CR_MODELS);
    let efforts: Vec<(String, String, String)> = CR_EFFORT
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let mk = CR_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let shown = filter_threads(&all, CR_FILTERS[filt()].0);
    let fl = files();
    let (fn_, fa, fd) = cr_totals(&fl);

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{CR_APP}",
            {s11_top(CR_APP, cr_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New thread",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New thread".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New thread" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                div { class: "oxs16m-filt", role: "radiogroup", "aria-label": "Show threads",
                    for (i, (k, l)) in CR_FILTERS.iter().copied().enumerate() {
                        button { key: "{k}", r#type: "button", role: "radio", class: "oxs16m-fb", "aria-checked": "{filt() == i}",
                            onclick: move |_| { filt.set(i); said.set(format!("Showing {l}")); }, "{l}" }
                    }
                }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No threads yet. Start one and the agent gets a pod of its own." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", "No threads here." } }
                    else {
                        ul { class: "oxs10-ul",
                            for c in shown.iter().cloned() {
                                {
                                    let id = c.id;
                                    let on = id == oid;
                                    let t2 = c.title.clone();
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs10-row oxs16m-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                "aria-label": "{c.title}, {status_word(c.status)}",
                                                onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                match c.status {
                                                    "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } },
                                                    "wait" => rsx! { span { class: "oxs16m-dot" } },
                                                    _ => rsx! { span { class: "oxs16m-k", {nico("check")} } },
                                                }
                                                span { class: "oxs16m-rt", span { class: "oxs10-ell", "{c.title}" } span { class: "oxs16m-rm", "{c.repo} · {c.at}" } }
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
            {pill(status_word(c.status), match c.status { "run" => "run", "wait" => "wait", "ok" => "ok", _ => "" })}
        }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        if cur.is_some() {
            div { class: "oxs16m-sugs", role: "group", "aria-label": "Suggested replies",
                for (i, s) in CR_SUGGEST.iter().copied().enumerate() {
                    button { key: "{i}", r#type: "button", class: "oxs16m-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); said.set(format!("Suggested: {s}")); }, span { class: "oxs10-ell", "{s}" } }
                }
            }
        }
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Reply, or start something new · @ to mention files",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Attach", onclick: move |_| said.set("Attach: files, images, a thread".to_string()), {nico("plus")} }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", CR_MODELS[i].0)); } }
                Pick { label: "Effort".to_string(), lead: rsx! { {nico("pulse")} }, items: efforts, cur: effort(), up: true, right: true,
                    on_pick: move |i: usize| { effort.set(i); said.set(format!("Effort: {}", CR_EFFORT[i].0)); } }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{CR_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        ol { class: "oxs10-work", "aria-label": "Tool calls",
                            for (i, (g, v, o)) in CR_TOOLS.iter().copied().enumerate() {
                                li { key: "{i}", class: "oxs10-wi", {nico(g)} span { class: "oxs16m-v", "{v}" } span { class: "oxs10-nl oxs16m-mono", "{o}" } }
                            }
                        }
                        div { class: "oxs16m-edits", role: "group", "aria-label": "Edits in this turn",
                            div { class: "oxs16m-eh", {nico("pen")} b { if fn_ == 0 { "All edits reverted" } else { "Edited {fn_} files" } }
                                span { class: "oxs16m-add", "+{fa}" } span { class: "oxs16m-del", "−{fd}" }
                                span { class: "oxs10-grow" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { tab.set(2); pane.set(true); pnar.set(true); said.set("Opened Changes in the pod".to_string()); }, "Review" }
                            }
                            ul { class: "oxs16m-ef",
                                for f in fl.iter().cloned() {
                                    li { key: "{f.path}",
                                        button { r#type: "button", class: "oxs16m-efb", "aria-expanded": "{shown_diff() == Some(f.path)}",
                                            onclick: move |_| { let on = *shown_diff.peek() == Some(f.path); shown_diff.set(if on { None } else { Some(f.path) }); },
                                            span { class: "oxs10-nl oxs16m-mono", "{f.path}" } span { class: "oxs16m-add", "+{f.add}" } span { class: "oxs16m-del", "−{f.del}" } span { class: "oxs16m-car", {nico("caret")} } }
                                        if shown_diff() == Some(f.path) {
                                            pre { class: "oxs16m-diff", "aria-label": "Diff of {f.path}",
                                                for (i, (s, l)) in f.diff.iter().copied().enumerate() {
                                                    span { key: "{i}", class: "oxs16m-dl", "data-s": "{s}", span { class: "oxs16m-sg", "{s}" } "{l}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "oxs10-agent", p { "{CR_DONE}" } }
                    } else if c.status == "run" {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.status = "ok"; }
                            said.set("Thread finished".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", "Working in the pod" } }
                    } else if c.status == "wait" {
                        div { class: "oxs16m-q", role: "group", "aria-label": "The agent asks",
                            p { {nico("hand")} b { "The agent asks" } }
                            p { "The login e2e fails only when tests run in parallel. Run them one at a time on CI, or give each test its own user?" }
                        }
                    } else {
                        div { class: "oxs10-agent", p { "Done, and the pod is still up if you want to look around." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {cr_mark(34)} }
                h1 { class: "oxs10-h1", "Start a thread" }
                div { class: "oxs10-under",
                    for (i, s) in ["Upgrade a dependency and fix what breaks", "Write tests for the checkout", "Find dead code in src/"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16m-sst", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel = rsx! { Pod { tab, files, loading, empty: all.is_empty() && !loading, said } };
    rsx! { Frame { app: "cr", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Pod".to_string() } }
}

/// Courier: a coding-agent app built around threads. All / Running / Needs you; tool lines and an
/// edits card per turn (open each file's diff); suggested replies; Model and Effort; a Pod pane
/// with Desktop, Ports (open, forward) and Changes (revert a file). States `changes` and
/// `desktop`.
#[component]
pub fn CourierScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `changes` or `desktop`).
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
    /// Your data. `None` shows the sample for `state`; start from `CourierData::default()`.
    #[props(default)]
    data: Option<CourierData>,
) -> Element {
    rsx! { CourierApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
