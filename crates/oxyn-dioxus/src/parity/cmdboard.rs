//! Screens S11 (Agent apps III): **Command Board**, a desktop app for an autonomous software agent
//! in the session-and-workspace pattern. Sessions in the sidebar; the session's chat with the
//! agent's progress and its questions (quick answers); the pull request it opened with checks; a
//! Workspace pane (Shell, Browser, Editor, Planner) with a timeline you scrub back through, and
//! Live. Sleep / Wake in the top bar. Rules in `oxyn_core::parity::cmdboard`; styles in
//! `cmdboard.css` (`oxs11b-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::parity::cmdboard::{
    BSession, CB_APP, CB_ASK, CB_PLAN, CB_REPOS, CB_STATES, CommandBoardData, Event, Msg, Pr,
    SStatus, ScreenState, TOOLS, Tool, add_session, cb_split, checks_passed, file_lines, last_of,
    plan_state, prompt_error, reply, sample_pr, scrub_at, shell_upto, toggle_sleep,
};

use super::cmdstudio::{Frame, Pick, pill, tmark};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Command Board (for a host's `ScreenStage { states }`).
pub fn s11_board_states() -> Vec<(&'static str, &'static str)> {
    CB_STATES.to_vec()
}

fn cb_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "2", y: "2", width: "16", height: "16", rx: "4", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M2 8h16M8 8v10", stroke: "currentColor", stroke_width: "1.7" }
            circle { cx: "13", cy: "13", r: "1.6", fill: "currentColor" }
        }
    }
}

fn bmark(id: &str, size: f32) -> Element {
    rsx! { span { class: "oxs10-mk", BrandIcon { name: id.to_string(), size, coloured: true } } }
}

fn check_mark(s: &str) -> Element {
    match s {
        "ok" => rsx! { span { class: "oxs10-ok", {nico("check")} } },
        "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } },
        _ => rsx! { span { class: "oxs10-dot", "data-tone": "bad" } },
    }
}

/// The Workspace pane: tabs, the tool's view at the scrubbed moment, and the timeline.
#[component]
fn BoardPane(
    events: Vec<Event>,
    tab: Signal<usize>,
    loading: bool,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut pos = use_signal(|| None::<usize>);
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if events.is_empty() {
        return rsx! {
            div { class: "oxs10-pempty", {nico("laptop")} p { "The agent's shell, browser, editor and plan show up here once a session starts. Scrub the timeline to see any earlier moment." } }
        };
    }
    let n = events.len();
    let at = scrub_at(n, pos());
    let live = pos().is_none();
    let t = tab();
    let tool = TOOLS[t];
    let e = events[at].clone();
    let max = n - 1;
    rsx! {
        div { class: "oxs11w-tabs oxs11b-tabs", role: "tablist", "aria-label": "Workspace",
            onkeydown: move |ev: KeyboardEvent| { let now = *tab.peek(); match ev.key() { Key::ArrowRight => tab.set((now + 1) % 4), Key::ArrowLeft => tab.set((now + 3) % 4), _ => {} } },
            for (i, tl) in TOOLS.iter().copied().enumerate() {
                button { key: "{i}", r#type: "button", role: "tab", class: "oxs11w-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), {nico(tl.glyph())} span { "{tl.label()}" } }
            }
        }
        if !live {
            p { class: "oxs11b-past", role: "note", {nico("clock")} span { class: "oxs10-nl", "Viewing {e.at} · {e.label}" }
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { pos.set(None); said.set("Back to live".to_string()); }, "Back to live" } }
        }
        div { class: "oxs11w-body", role: "tabpanel", "aria-label": "{tool.label()}",
            match tool {
                Tool::Shell => {
                    let lines = shell_upto(&events, at);
                    rsx! {
                        pre { class: "oxs11w-term", "aria-label": "Shell",
                            if lines.is_empty() { span { class: "oxs11w-tl", "No commands yet at this point." } }
                            for (i, (cmd, l)) in lines.iter().copied().enumerate() {
                                span { key: "{i}", class: "oxs11w-tl", "data-cmd": "{cmd}", if cmd { "$ " } "{l}" }
                            }
                        }
                    }
                }
                Tool::Editor => {
                    match last_of(&events, Tool::Editor, at) {
                        Some(k) => {
                            let path = events[k].body.first().copied().unwrap_or("");
                            let lines = file_lines(path);
                            rsx! {
                                div { class: "oxs11w-code",
                                    div { class: "oxs11w-fh", {nico("doc")} span { class: "oxs10-nl", "{path}" } span { class: "oxs10-ago", "{events[k].at}" } }
                                    pre { class: "oxs11w-pre",
                                        for (i, l) in lines.iter().enumerate() {
                                            span { key: "{i}", class: "oxs11w-ln", span { class: "oxs11w-no", "{i + 1}" } span { "{l}" } }
                                        }
                                    }
                                }
                            }
                        }
                        None => rsx! { p { class: "oxs10-faint", "No file opened yet at this point." } },
                    }
                }
                Tool::Browser => {
                    match last_of(&events, Tool::Browser, at) {
                        Some(k) => {
                            let addr = events[k].body.first().copied().unwrap_or("");
                            let local = addr.starts_with("localhost");
                            rsx! {
                                div { class: "oxs11w-br",
                                    div { class: "oxs11w-addr", {nico("globe")} span { class: "oxs10-nl", "{addr}" } }
                                    if local {
                                        div { class: "oxs11b-app",
                                            div { class: "oxs11b-apph", b { "Reports" } span { class: "oxs10-grow" } span { class: "oxs11b-fake", "Filters" } span { class: "oxs11b-fake", "data-hi": "true", "Export CSV" } }
                                            for (i, (a, b, c)) in [("Q3 field review", "Active", "12"), ("Solar grants", "Active", "8"), ("Fleet audit", "Active", "5")].iter().copied().enumerate() {
                                                div { key: "{i}", class: "oxs11b-tr", span { class: "oxs10-nl", "{a}" } span { "{b}" } span { "{c}" } }
                                            }
                                        }
                                    } else {
                                        div { class: "oxs11b-doc",
                                            b { "unparse(data, config)" }
                                            p { "Turns an array of objects into CSV text. Fields with commas, quotes or line breaks are wrapped in quotes; quotes inside are doubled." }
                                            code { "unparse(rows, {{ quotes: true }})" }
                                        }
                                    }
                                }
                            }
                        }
                        None => rsx! { p { class: "oxs10-faint", "The browser wasn't used yet at this point." } },
                    }
                }
                Tool::Planner => rsx! {
                    ol { class: "oxs11w-tl2 oxs11b-plan", "aria-label": "Plan",
                        for (i, (s, _)) in CB_PLAN.iter().copied().enumerate() {
                            {
                                let st = plan_state(i, at);
                                rsx! {
                                    li { key: "{i}", class: "oxs11w-ti", "data-s": "{st}",
                                        match st { "done" => rsx! { span { class: "oxs10-ok", {nico("check")} } }, "run" => rsx! { span { class: "oxs10-spin oxs10-spin-s" } }, _ => rsx! { span { class: "oxs10-ring" } } }
                                        span { "{s}" }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
        div { class: "oxs11b-tline",
            div { class: "oxs11b-th", span { class: "oxs10-nl", "{e.at} · {e.label}" }
                button { r#type: "button", class: "oxs11b-live", "aria-pressed": "{live}", onclick: move |_| { pos.set(None); said.set("Live".to_string()); }, span { class: "oxs11b-ld" } "Live" } }
            input { class: "oxs11b-range", r#type: "range", min: "0", max: "{max}", step: "1", value: "{at}", "aria-label": "Timeline", "aria-valuetext": "{e.at}, {e.label}",
                oninput: move |ev| { let v: usize = ev.value().parse().unwrap_or(0); pos.set(if v >= max { None } else { Some(v) }); } }
            div { class: "oxs11b-ticks", "aria-hidden": "true",
                for (i, x) in events.iter().enumerate() { span { key: "{i}", class: "oxs11b-tk", "data-on": "{i <= at}", title: "{x.at}" } }
            }
        }
    }
}

fn pr_card(
    pr: &Pr,
    mut tab: Signal<usize>,
    mut pnar: Signal<bool>,
    mut said: Signal<String>,
) -> Element {
    let (ok, n) = checks_passed(pr);
    let num = pr.number;
    rsx! {
        div { class: "oxs11b-pr", role: "group", "aria-label": "Pull request #{num}",
            div { class: "oxs11b-prh", {bmark("github", 16.0)} b { class: "oxs10-nl", "#{num} {pr.title}" } span { class: "oxs11-tag", "data-tone": if ok == n { "ok" } else { "" }, "{ok}/{n} checks" } }
            div { class: "oxs11b-prm", code { class: "oxs10-ell", "{pr.branch}" } span { class: "oxs10-st", span { class: "oxs10-add", "+{pr.stat.0}" } span { class: "oxs10-del", "−{pr.stat.1}" } } }
            ul { class: "oxs11b-checks",
                for (i, (c, s)) in pr.checks.iter().copied().enumerate() {
                    li { key: "{i}", {check_mark(s)} span { class: "oxs10-nl", "{c}" } span { class: "oxs10-ago", if s == "ok" { "passed" } else if s == "run" { "running" } else { "failed" } } }
                }
            }
            div { class: "oxs11-aa",
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { tab.set(2); pnar.set(true); said.set("Editor: the changed file".to_string()); }, "View diff" }
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| said.set(format!("Opening PR #{num}")), "Open PR" }
            }
        }
    }
}

fn status_mark(s: SStatus) -> Element {
    match s {
        SStatus::Sleeping => {
            rsx! { span { class: "oxs10-dot", "data-tone": "idle", "aria-hidden": "true" } }
        }
        SStatus::PrOpen => rsx! { span { class: "oxs11b-prm2", {nico("route")} } },
        x => tmark(x.tone()),
    }
}

#[component]
fn CommandBoardApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<CommandBoardData>,
) -> Element {
    let (st, own) = cb_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| CommandBoardData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let is_pr = own == "pr";
    let d1 = d0.clone();
    let mut list = use_signal(move || {
        let mut l: Vec<BSession> = d1.sessions;
        if is_pr && let Some(s) = l.first_mut() {
            s.status = SStatus::PrOpen;
        }
        l
    });
    let first = d0.sessions.first().map(|s| s.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let d2 = d0.clone();
    let mut msgs = use_signal(move || {
        let mut m: Vec<Msg> = d2.msgs;
        if is_pr {
            m.push((false, CB_ASK.1[1].to_string()));
            m.push((true, "Matched the table. PR #482 is up and every check passed; the preview is linked in it.".to_string()));
        }
        m
    });
    let d3 = d0.clone();
    let mut pr = use_signal(move || {
        if is_pr {
            Some(sample_pr(true))
        } else {
            let st0 = d3.sessions.first().map(|s| s.status);
            d3.pr
                .filter(|_| matches!(st0, Some(SStatus::PrOpen | SStatus::Done)))
        }
    });
    let tab0 = if own == "planner" { 3 } else { 0 };
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
    let pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut repo = use_signal(|| 0usize);
    let mut q = use_signal(String::new);
    let pr_later = d0.pr.clone();
    let all = list();
    let oid = open();
    let cur = all.iter().find(|s| s.id == oid).cloned();
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        let id = *open.peek();
        if id == 0 {
            let rp = *repo.peek();
            match add_session(&mut list.write(), &p, rp) {
                Ok(nid) => {
                    open.set(nid);
                    text.set(String::new());
                    err.set(None);
                    said.set(format!("Session started in {}", CB_REPOS[rp]));
                }
                Err(e) => {
                    err.set(Some(e));
                    said.set(e.to_string());
                }
            }
            return;
        }
        match reply(&mut msgs.write(), &p) {
            Ok(()) => {
                text.set(String::new());
                err.set(None);
                if let Some(s) = list
                    .write()
                    .iter_mut()
                    .find(|s| s.id == id && s.status == SStatus::NeedsYou)
                {
                    s.status = SStatus::Working;
                }
                said.set("Reply sent".to_string());
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let can = !text().trim().is_empty() && (oid != 0 || prompt_error(&text()).is_none());
    let ql = q().trim().to_lowercase();
    let shown: Vec<_> = all
        .iter()
        .filter(|s| ql.is_empty() || s.title.to_lowercase().contains(&ql))
        .cloned()
        .collect();
    let new_on = oid == 0;

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{CB_APP}",
            {s11_top(CB_APP, cb_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{new_on}", "aria-current": if new_on { "page" } else { "false" }, title: "New session",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New session".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New session" } span { class: "oxs10-kbd", "⌘N" } } }
            button { r#type: "button", class: "oxs10-nav", title: "Knowledge", onclick: move |_| said.set("Knowledge: notes the agent reads before it starts".to_string()),
                {nico("book")} if !rl { span { class: "oxs10-nl", "Knowledge" } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search sessions", "aria-label": "Search sessions", value: "{q}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    p { class: "oxs10-sh", "Sessions" }
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No sessions yet. Describe a task to start one." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", "No sessions match." } }
                    else {
                        ul { class: "oxs10-ul",
                            for s in shown {
                                {
                                    let id = s.id;
                                    let on = id == oid;
                                    let t2 = s.title.clone();
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs10-srow", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                {status_mark(s.status)}
                                                span { class: "oxs10-s2",
                                                    span { class: "oxs10-stt oxs10-ell", "{s.title}" }
                                                    span { class: "oxs10-ssub oxs11b-sub",
                                                        if !s.from.is_empty() { {bmark(s.from, 12.0)} }
                                                        span { class: "oxs10-ell", "{CB_REPOS[s.repo]} · {s.status.label()}" }
                                                    }
                                                }
                                                span { class: "oxs10-ago", "{s.ago}" }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(s) = &cur { "{s.title}" } else { "New session" } }
        if let Some(s) = &cur {
            {pill(s.status.label(), s.status.tone())}
            span { class: "oxs10-envc oxs11b-cr", {nico("coin")} span { "3.4 credits" } }
        }
        span { class: "oxs10-grow" }
        if let Some(s) = &cur {
            if matches!(s.status, SStatus::Working | SStatus::NeedsYou | SStatus::Sleeping) {
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11b-sleep",
                    onclick: move |_| { let id = *open.peek(); let mut l = list.write(); if let Some(x) = l.iter_mut().find(|x| x.id == id) { let ns = toggle_sleep(x); said.set(if ns == SStatus::Sleeping { "Sleeping. Wake it to continue" } else { "Awake and working" }.to_string()); } },
                    {nico(if s.status == SStatus::Sleeping { "sun" } else { "pause" })} span { class: "oxs10-ell", if s.status == SStatus::Sleeping { "Wake" } else { "Sleep" } } }
            }
        }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": if oid == 0 { "Describe a task" } else { "Reply to the agent" },
                placeholder: if oid == 0 { "Describe a task, link a ticket or paste an error" } else { "Reply, or add something for the agent to do" },
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Attach", onclick: move |_| said.set("Attach: a file, a screenshot or a log".to_string()), {nico("clip")} }
                if oid == 0 {
                    Pick { label: "Repository".to_string(), lead: rsx! { {bmark("github", 14.0)} }, items: CB_REPOS.iter().map(|r| (r.to_string(), String::new(), String::new())).collect::<Vec<_>>(), cur: repo(), up: true,
                        on_pick: move |i: usize| { repo.set(i); said.set(format!("Repository: {}", CB_REPOS[i])); } }
                } else if let Some(s) = &cur {
                    span { class: "oxs10-envc", {bmark("github", 13.0)} span { class: "oxs10-ell", "{CB_REPOS[s.repo]}" } }
                }
                span { class: "oxs10-grow" }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(s) = cur.clone() {
        let sample = s.id == 1;
        let ms = if sample {
            msgs()
        } else {
            vec![
                (false, s.title.clone()),
                (
                    true,
                    "On it. I'll post updates here and ask if anything is unclear.".to_string(),
                ),
            ]
        };
        let p = pr();
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    for (i, (agent, t)) in ms.iter().cloned().enumerate() {
                        if agent {
                            div { key: "{i}", class: "oxs11b-am", span { class: "oxs11b-av", {cb_mark(14)} } p { "{t}" } }
                        } else {
                            div { key: "{i}", class: "oxs10-you", p { "{t}" } }
                        }
                    }
                    if sample && s.status == SStatus::NeedsYou {
                        div { class: "oxs11b-ask", role: "group", "aria-label": "The agent is asking",
                            p { span { class: "oxs10-dot", "data-tone": "wait" } "{CB_ASK.0}" }
                            div { class: "oxs11-aa",
                                for (i, a) in CB_ASK.1.iter().copied().enumerate() {
                                    button { key: "{i}", r#type: "button", class: "oxscn-btn oxs10-sbtn",
                                        onclick: move |_| { let _ = reply(&mut msgs.write(), a); let id = *open.peek(); if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.status = SStatus::Working; } said.set(format!("Answered: {a}")); }, "{a}" }
                                }
                            }
                        }
                    }
                    if sample && s.status == SStatus::Working {
                        span { key: "{oid}-tick", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id && x.status == SStatus::Working) { x.status = SStatus::PrOpen; }
                            pr.set(pr_later.clone().or(Some(sample_pr(false))));
                            msgs.write().push((true, "PR #482 is open. Build, tests and lint pass; the preview deploy is still running.".to_string()));
                            said.set("Pull request opened".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", "Working · updating the export and opening a PR" } }
                    }
                    if sample && s.status == SStatus::Sleeping {
                        p { class: "oxs11b-zz", {nico("pause")} "Sleeping. Wake it from the top bar or send a message." }
                    }
                    if let (true, Some(pr0)) = (sample, p) {
                        {pr_card(&pr0, tab, pnar, said)}
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {cb_mark(34)} }
                h1 { class: "oxs10-h1", "What should the agent take on?" }
                div { class: "oxs10-under",
                    span { class: "oxs10-envc", {bmark("linear", 13.0)} span { "Link a ticket" } }
                    span { class: "oxs10-envc", {bmark("slack", 13.0)} span { "Or tag it in a channel" } }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let events = if cur.as_ref().is_some_and(|s| s.id == 1) || loading {
        d0.events.clone()
    } else {
        Vec::new()
    };
    let panel = rsx! { BoardPane { key: "{oid}", events, tab, loading, said } };
    rsx! { Frame { app: "cb", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Workspace".to_string() } }
}

/// Command Board: the session-and-workspace desktop app for an autonomous agent. Sessions in the
/// sidebar (working, needs you, sleeping, PR open, done); the session's chat with questions and
/// quick answers; the pull request with its checks; a Workspace pane (Shell, Browser, Editor,
/// Planner) with a timeline to scrub and Live; Sleep / Wake. States `pr` and `planner`.
#[component]
pub fn CommandBoardScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `pr` or `planner`).
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
    /// Your data. `None` shows the sample for `state`; start from `CommandBoardData::default()` and
    /// replace what you have.
    #[props(default)]
    data: Option<CommandBoardData>,
) -> Element {
    rsx! { CommandBoardApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
