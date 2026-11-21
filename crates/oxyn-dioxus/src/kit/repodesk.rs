//! Screens S16 (Agent apps IV): **Repo Desk**, a chat-first desktop app for an agent that works
//! on a code host. Home, Inbox (issues and pull requests that need you, with filters and Start
//! session) and Projects (a goal with milestones) in the nav, sessions on their own branches; a
//! session's chat beside canvases (Plan, Pull request, Terminal, Browser). Rules in
//! `oxyn_core::kit::repodesk`; styles in `repodesk.css` (`oxs16r-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::kit::repodesk::{
    Kind, RD_APP, RD_BROWSER, RD_CANVASES, RD_CHECKS, RD_FILTERS, RD_GOAL, RD_MILESTONES,
    RD_MODELS, RD_NAV, RD_PLAN, RD_PR, RD_PROMPT, RD_REPLY, RD_STATES, RD_TERM, RepoDeskData,
    ScreenState, add_session, filter_inbox, goal_progress, prompt_error, rd_split,
};

use super::cmdstudio::{Frame, Pick, branch_ico, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Repo Desk (for a host's `ScreenStage { states }`).
pub fn s16_repodesk_states() -> Vec<(&'static str, &'static str)> {
    RD_STATES.to_vec()
}

fn rd_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            path { d: "M3 5.5A2.5 2.5 0 0 1 5.5 3h9A2.5 2.5 0 0 1 17 5.5v6a2.5 2.5 0 0 1-2.5 2.5H9l-4 3v-3h0A2 2 0 0 1 3 12z", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linejoin: "round" }
            circle { cx: "7.5", cy: "8.5", r: "1.2", fill: "currentColor" }
            circle { cx: "12.5", cy: "8.5", r: "1.2", fill: "currentColor" }
        }
    }
}

fn host() -> Element {
    rsx! { span { class: "oxs10-mk oxs16r-host", BrandIcon { name: "github".to_string(), size: 13.0, coloured: true } } }
}

fn kind_glyph(k: Kind) -> &'static str {
    match k {
        Kind::Issue => "target",
        Kind::Pull => "branch",
        Kind::Review => "eye",
    }
}

fn kglyph(k: Kind) -> Element {
    if k == Kind::Pull {
        rsx! { span { class: "oxs16r-kg", "data-k": "pull", {branch_ico()} } }
    } else {
        rsx! { span { class: "oxs16r-kg", "data-k": "{kind_glyph(k)}", {nico(kind_glyph(k))} } }
    }
}

/// The canvases the agent shares beside the chat.
#[component]
fn Canvases(tab: Signal<usize>, loading: bool, empty: bool, said: Signal<String>) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut plan = use_signal(|| RD_PLAN.iter().map(|p| p.1).collect::<Vec<_>>());
    let mut ready = use_signal(|| false);
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("layers")} p { "When a session starts, the agent shares its plan, pull request, terminal and browser here." } }
        };
    }
    let done = plan().iter().filter(|x| **x).count();
    let (num, title, files, add, del) = RD_PR;
    rsx! {
        div { class: "oxs16r-tabs", role: "tablist", "aria-label": "Canvases",
            onkeydown: move |e: KeyboardEvent| { let now = *tab.peek(); match e.key() { Key::ArrowRight => tab.set((now + 1) % 4), Key::ArrowLeft => tab.set((now + 3) % 4), _ => {} } },
            for (i, (k, l, g)) in RD_CANVASES.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16r-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), if g == "branch" { {branch_ico()} } else { {nico(g)} } span { class: "oxs10-ell", "{l}" } }
            }
        }
        div { class: "oxs16r-cb", role: "tabpanel", "aria-label": "{RD_CANVASES[t].1}",
            match RD_CANVASES[t].0 {
                "plan" => rsx! {
                    div { class: "oxs16r-ph", b { "Plan" } span { class: "oxs10-ago", "{done} of {RD_PLAN.len()} done" } }
                    ul { class: "oxs16r-plan",
                        for (i, (s, _)) in RD_PLAN.iter().copied().enumerate() {
                            li { key: "{i}",
                                label { class: "oxs16r-pi", "data-done": "{plan()[i]}",
                                    input { r#type: "checkbox", checked: plan()[i], onchange: move |_| { let mut w = plan.write(); w[i] = !w[i]; let v = w[i]; drop(w); said.set(format!("{} {s}", if v { "Done:" } else { "To do:" })); } }
                                    span { "{s}" }
                                }
                            }
                        }
                    }
                },
                "pr" => rsx! {
                    div { class: "oxs16r-pr",
                        p { class: "oxs16r-prt", {host()} span { class: "oxs16r-num", "#{num}" } b { "{title}" } }
                        p { class: "oxs16r-prm", if ready() { {pill("Open", "ok")} } else { {pill("Draft", "")} }
                            span { class: "oxs16r-bri", {branch_ico()} span { class: "oxs10-ell", "agent/csv-export → main" } } }
                        p { class: "oxs16r-stat", "{files} files · " span { class: "oxs16r-add", "+{add}" } " " span { class: "oxs16r-del", "−{del}" } }
                        p { class: "oxs10-sh", "Checks" }
                        ul { class: "oxs16r-checks",
                            for (n, tone) in RD_CHECKS.iter().copied() {
                                li { key: "{n}", class: "oxs16r-ck",
                                    if tone == "run" { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs16r-cd", "data-tone": "{tone}" } }
                                    span { class: "oxs10-nl", "{n}" }
                                    span { class: "oxs10-ago", match tone { "ok" => "Passed", "run" => "Running", _ => "Failed" } }
                                }
                            }
                        }
                        div { class: "oxs16r-pa",
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| said.set(format!("Opened pull request #{num} on the code host")), "View on the code host" }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", disabled: ready(), onclick: move |_| { ready.set(true); said.set(format!("#{num} marked ready for review")); }, if ready() { "Ready for review" } else { "Mark ready" } }
                        }
                    }
                },
                "terminal" => rsx! {
                    pre { class: "oxs16r-term", "aria-label": "Terminal output",
                        for (i, (cmd, l)) in RD_TERM.iter().copied().enumerate() {
                            span { key: "{i}", class: "oxs16r-tl", "data-cmd": "{cmd}", if cmd { "$ " } "{l}" }
                        }
                    }
                },
                _ => rsx! {
                    div { class: "oxs16r-br",
                        div { class: "oxs16r-addr", {nico("globe")} span { class: "oxs10-nl", "{RD_BROWSER.0}" } }
                        div { class: "oxs16r-page",
                            p { class: "oxs16r-pt", "Monthly report · 2026-10" }
                            table { class: "oxs16r-tbl",
                                for (i, row) in RD_BROWSER.1.iter().copied().enumerate() {
                                    tr { key: "{i}", "data-head": "{i == 0}", "data-total": "{i + 1 == RD_BROWSER.1.len()}",
                                        for (j, c) in row.split(',').enumerate() { if i == 0 { th { key: "{j}", "{c}" } } else { td { key: "{j}", "{c}" } } }
                                    }
                                }
                            }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| said.set("Downloaded report-2026-10.csv".to_string()), {nico("download")} span { "Export CSV" } }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn RepoDeskApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<RepoDeskData>,
) -> Element {
    let (st, own) = rd_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| RepoDeskData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.sessions);
    let i0 = d0.inbox.clone();
    let mut inbox = use_signal(move || i0);
    let first = d0.sessions.first().map(|t| t.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let mut view = use_signal(move || if own.is_empty() { "session" } else { own });
    let mut filt = use_signal(|| "all");
    let mut isel = use_signal(|| 1u32);
    let mut miles = use_signal(|| RD_MILESTONES.iter().map(|m| m.1).collect::<Vec<_>>());
    let ctab = use_signal(|| 0usize);
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
    let pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let all = list();
    let oid = open();
    let v = view();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let running = cur.as_ref().is_some_and(|c| c.running);
    let rl = side() == "rail";
    let ninbox = inbox().len();
    let mut submit = move || {
        let p = text.peek().clone();
        match add_session(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                view.set("session");
                text.set(String::new());
                err.set(None);
                said.set("Session started on its own branch".to_string());
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let models = opts2(&RD_MODELS);
    let mk = RD_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let home_on = v == "session" && oid == 0;

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{RD_APP}",
            {s11_top(RD_APP, rd_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{home_on}", title: "New session",
                onclick: move |_| { open.set(0); view.set("session"); drawer.set(false); said.set("New session".to_string()); },
                {nico("pen")} if !rl { span { class: "oxs10-nl", "New session" } span { class: "oxs10-kbd", "⌘N" } } }
            for (k, l, g) in RD_NAV.iter().copied() {
                {
                    let on = if k == "home" { v == "session" && oid == 0 } else { v == k };
                    rsx! {
                        button { key: "{k}", r#type: "button", class: "oxs10-nav", "data-on": "{on}", "aria-current": if on { "page" } else { "false" }, title: "{l}",
                            onclick: move |_| { if k == "home" { open.set(0); view.set("session"); } else { view.set(k); } drawer.set(false); said.set(l.to_string()); },
                            {nico(g)} if !rl { span { class: "oxs10-nl", "{l}" } if k == "inbox" && ninbox > 0 { span { class: "oxs16r-nb", "{ninbox}" } } } }
                    }
                }
            }
            if !rl {
                p { class: "oxs10-sh", "Sessions" }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(5)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No sessions yet. Start one from Home or the Inbox." } }
                    else {
                        ul { class: "oxs10-ul",
                            for c in all.iter().cloned() {
                                {
                                    let id = c.id;
                                    let on = id == oid && v == "session";
                                    let t2 = c.title.clone();
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs16r-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); view.set("session"); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs16r-k", {nico("chat")} } }
                                                span { class: "oxs16r-rt", b { class: "oxs10-ell", "{c.title}" } span { class: "oxs10-ell oxs16r-rs", "{c.repo}" } }
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
        span { class: "oxs10-ttl oxs10-ell",
            if loading { "" } else if v == "inbox" { "Inbox" } else if v == "project" { "{RD_GOAL.0}" } else if let Some(c) = &cur { "{c.title}" } else { "Home" } }
        if v == "session" {
            if let Some(c) = &cur {
                if c.running { {pill("Working", "run")} } else { {pill("Idle", "")} }
                span { class: "oxs16r-bri oxs10-wide-only", {branch_ico()} span { class: "oxs10-ell", "{c.branch}" } }
            }
        }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Ask the agent to work on something · # for issues",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Attach", onclick: move |_| said.set("Attach: files, images, an issue".to_string()), {nico("clip")} }
                span { class: "oxs10-chip oxs16r-repo", {host()} span { class: "oxs10-ell", "acorn/insights" } }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", RD_MODELS[i].0)); } }
                if running && v == "session" {
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
    } else if v == "inbox" {
        let f = filt();
        let items = filter_inbox(&inbox(), f);
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs16r-filt", role: "radiogroup", "aria-label": "Show",
                        for (k, l) in RD_FILTERS.iter().copied() {
                            button { key: "{k}", r#type: "button", role: "radio", class: "oxs16r-fc", "aria-checked": "{f == k}", onclick: move |_| { filt.set(k); said.set(format!("Showing {l}")); }, "{l}" }
                        }
                    }
                    if items.is_empty() { p { class: "oxs10-faint", if inbox().is_empty() { "You're all caught up." } else { "Nothing here." } } }
                    ul { class: "oxs16r-inbox", "aria-label": "Needs you",
                        for it in items {
                            {
                                let id = it.id;
                                let on = isel() == id;
                                rsx! {
                                    li { key: "{id}", class: "oxs16r-item", "data-on": "{on}",
                                        button { r#type: "button", class: "oxs16r-ib", "aria-pressed": "{on}", onclick: move |_| isel.set(id),
                                            {kglyph(it.kind)}
                                            span { class: "oxs16r-it",
                                                span { class: "oxs16r-ir", {host()} span { class: "oxs10-ell", "{it.refs}" } span { class: "oxs10-ago", "{it.at}" } }
                                                b { class: "oxs10-ell", "{it.title}" }
                                                span { class: "oxs16r-why", "{it.kind.label()} · {it.why}" }
                                            }
                                        }
                                        div { class: "oxs16r-ia",
                                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { inbox.write().retain(|x| x.id != id); said.set("Done, removed from the Inbox".to_string()); }, "Done" }
                                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| {
                                                let p = format!("{} ({})", it.title, it.refs);
                                                if let Ok(nid) = add_session(&mut list.write(), &p) { open.set(nid); view.set("session"); inbox.write().retain(|x| x.id != id); said.set("Session started from the Inbox".to_string()); }
                                            }, "Start session" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if v == "project" {
        let m = miles();
        let pairs: Vec<(&str, bool)> = RD_MILESTONES
            .iter()
            .zip(m.iter())
            .map(|(a, b)| (a.0, *b))
            .collect();
        let (dn, tn, pc) = goal_progress(&pairs);
        let w = format!("width:{pc}%");
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs16r-goal",
                        p { class: "oxs10-sh", "Goal" }
                        p { class: "oxs16r-gt", "{RD_GOAL.1}" }
                        p { class: "oxs16r-gm", {nico("cal")} span { "Due {RD_GOAL.2}" } span { class: "oxs10-grow" } span { "{dn} of {tn} milestones · {pc}%" } }
                        div { class: "oxs16r-bar", role: "progressbar", "aria-label": "Goal progress", "aria-valuenow": "{pc}", "aria-valuemin": "0", "aria-valuemax": "100", span { style: "{w}" } }
                    }
                    p { class: "oxs10-sh", "Milestones" }
                    ul { class: "oxs16r-plan",
                        for (i, (s, _)) in RD_MILESTONES.iter().copied().enumerate() {
                            li { key: "{i}",
                                label { class: "oxs16r-pi", "data-done": "{m[i]}",
                                    input { r#type: "checkbox", checked: m[i], onchange: move |_| { let mut w = miles.write(); w[i] = !w[i]; drop(w); said.set(format!("Milestone: {s}")); } }
                                    span { "{s}" }
                                }
                            }
                        }
                    }
                    p { class: "oxs10-sh", "Sessions on this goal" }
                    ul { class: "oxs10-ul",
                        for c in all.iter().filter(|c| c.repo == "acorn/insights").cloned() {
                            {
                                let id = c.id;
                                rsx! {
                                    li { key: "{id}",
                                        button { r#type: "button", class: "oxs16r-row", onclick: move |_| { open.set(id); view.set("session"); },
                                            if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs16r-k", {nico("chat")} } }
                                            span { class: "oxs16r-rt", b { class: "oxs10-ell", "{c.title}" } span { class: "oxs10-ell oxs16r-rs", "{c.branch}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{RD_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        div { class: "oxs10-agent", p { "{RD_REPLY}" } }
                        div { class: "oxs16r-cards",
                            for (i, (k, l, g)) in RD_CANVASES.iter().copied().enumerate().take(2) {
                                button { key: "{k}", r#type: "button", class: "oxs16r-cc", onclick: move |_| { let mut t = ctab; t.set(i); said.set(format!("{l} canvas")); },
                                    if g == "branch" { {branch_ico()} } else { {nico(g)} }
                                    span { class: "oxs10-s2", b { class: "oxs10-ell", "{l}" } span { class: "oxs10-ssub oxs10-ell", if k == "plan" { "5 steps" } else { "Draft #418 · 3 checks" } } }
                                }
                            }
                        }
                    }
                    if c.running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.running = false; }
                            said.set("Session idle · pull request is a draft".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if sample { "Adding the Export button" } else { "Reading the repository" } } }
                    } else if sample {
                        div { class: "oxs10-agent", p { "The Export button is in and the tests pass. Pull request #418 is a draft; mark it ready when you're happy." } }
                    } else {
                        div { class: "oxs10-agent", p { "I made a branch for this and I'm reading the code. The plan will show up in the Plan canvas." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {rd_mark(34)} }
                h1 { class: "oxs10-h1", "What should we work on?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Fix the issue assigned to me", "Review my open pull requests", "Write release notes for this week"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16r-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let empty = d0.sessions.is_empty();
    let panel = rsx! { Canvases { tab: ctab, loading, empty, said } };
    rsx! { Frame { app: "rd", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Canvases".to_string() } }
}

/// Repo Desk: a chat-first desktop app for an agent on a code host. Home, Inbox (filters, Done,
/// Start session) and Projects (goal, due date, milestones) in the nav; sessions on their own
/// branches; canvases beside the chat (Plan, Pull request with checks and Mark ready, Terminal,
/// Browser). States `inbox` and `project`.
#[component]
pub fn RepoDeskScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `inbox` or `project`).
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
    /// Your data. `None` shows the sample for `state`; start from `RepoDeskData::default()`.
    #[props(default)]
    data: Option<RepoDeskData>,
) -> Element {
    rsx! { RepoDeskApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
