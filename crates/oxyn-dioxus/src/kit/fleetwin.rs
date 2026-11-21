//! Screens S11 (Agent apps III): **Fleet Window**, a desktop coding-agent app in the
//! agents-window pattern. Agents in the sidebar (each on its own worktree, with its model mark and
//! lines changed), the open agent's steps, a changes pane (accept or revert per hunk), a mode
//! picker (Agent, Plan, Ask), a multi-model picker that runs one prompt on several models at once,
//! and a Compare view that ranks the attempts and applies one. Rules in
//! `oxyn_core::kit::fleetwin`; styles in `fleetwin.css` (`oxs11-`), on the S10 app frame.
//! Also holds the S11 account row with its menu, shared by the other S11 apps.

use dioxus::prelude::*;
use oxyn_core::kit::cmdstudio::plan_for;
use oxyn_core::kit::fleetwin::{
    AgentMode, Attempt, COMPARE_PROMPT, FW_APP, FW_MODELS, FW_PROMPT, FW_STATES, FW_STEPS,
    FleetWindowData, PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, TState,
    apply_attempt, best_attempt, fw_split, mmss, plan_progress, prompt_error, reopen_attempts,
    spawn, thread_diff, toggle_model, worktree_for,
};

use super::cmdstudio::{DiffPane, Frame, Pick, branch_ico, pill, tmark};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Fleet Window (for a host's `ScreenStage { states }`).
pub fn s11_fleet_states() -> Vec<(&'static str, &'static str)> {
    FW_STATES.to_vec()
}

const ACCT: [(&str, &str, &str); 5] = [
    ("gear", "Settings", "⌘,"),
    ("chart", "Usage", ""),
    ("kbd", "Keyboard shortcuts", "?"),
    ("help", "Help", ""),
    ("logout", "Log out", ""),
];

/// The S11 account row: avatar, name and plan, opening the account menu (Escape closes it).
#[component]
pub(super) fn AcctFoot(rail: bool, said: Signal<String>) -> Element {
    let mut open = use_signal(|| false);
    let mut said = said;
    rsx! {
        div { class: "oxs11-acct", onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
            button { r#type: "button", class: "oxs11-acb", "aria-haspopup": "menu", "aria-expanded": "{open()}", "aria-label": "Account, {PERSON}, {PERSON_PLAN}",
                onclick: move |_| open.toggle(),
                span { class: "oxs10-av", "aria-hidden": "true", "{PERSON_INI}" }
                if !rail {
                    span { class: "oxs10-an", b { class: "oxs10-ell", "{PERSON}" } span { class: "oxs10-ell", "{PERSON_PLAN}" } }
                    span { class: "oxs11-ud", {nico("updown")} }
                }
            }
            if open() {
                div { class: "oxscn-menu oxs11-amenu", role: "menu", "aria-label": "Account",
                    div { class: "oxs11-ahd",
                        span { class: "oxs10-av", "aria-hidden": "true", "{PERSON_INI}" }
                        span { class: "oxs10-an", b { class: "oxs10-ell", "{PERSON}" } span { class: "oxs10-ell", title: "{PERSON_EMAIL}", "{PERSON_EMAIL}" } }
                    }
                    for (i, (g, l, k)) in ACCT.iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem",
                            onclick: move |_| { open.set(false); said.set(l.to_string()); },
                            {nico(g)} span { class: "oxscn-mi-l", "{l}" } if !k.is_empty() { span { class: "oxscn-mi-k", "{k}" } } }
                    }
                }
            }
        }
    }
}

/// The S11 sidebar head: the app mark and name, and Hide sidebar.
pub(super) fn s11_top(
    name: &'static str,
    mark: Element,
    rail: bool,
    mut side: Signal<&'static str>,
    mut dr: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "oxs10-stop",
            span { class: "oxs10-brand", {mark} if !rail { span { class: "oxs10-ell", "{name}" } } }
            if !rail {
                span { class: "oxs10-grow" }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Hide sidebar", onclick: move |_| { side.set("hidden"); dr.set(false); }, {nico("side")} }
            }
        }
    }
}

fn fw_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "6", y: "1.8", width: "12.2", height: "9.5", rx: "2.4", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            rect { x: "1.8", y: "8.7", width: "12.2", height: "9.5", rx: "2.4", fill: "var(--oxth-card)", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M5 13.4h5.8", stroke: "currentColor", stroke_width: "1.7", stroke_linecap: "round" }
        }
    }
}

fn mk(i: usize, size: f32) -> Element {
    let m = FW_MODELS.get(i).map(|m| m.1).unwrap_or("claude");
    rsx! { span { class: "oxs10-mk", BrandIcon { name: m.to_string(), size, coloured: true } } }
}

/// The multi-model picker: menuitemcheckbox per model; more than one runs them in parallel.
#[component]
fn Models(sel: Signal<Vec<usize>>, said: Signal<String>) -> Element {
    let mut open = use_signal(|| false);
    let mut sel = sel;
    let mut said = said;
    let s = sel();
    let label = if s.len() > 1 {
        format!("{} models", s.len())
    } else {
        FW_MODELS[s.first().copied().unwrap_or(0)].0.to_string()
    };
    rsx! {
        div { class: "oxs10-pk",
            button { r#type: "button", class: "oxs10-chip", "aria-haspopup": "menu", "aria-expanded": "{open()}", "aria-label": "Models: {label}",
                onclick: move |_| open.toggle(),
                span { class: "oxs11-mks", for m in s.iter().copied() { span { key: "{m}", {mk(m, 13.0)} } } }
                span { class: "oxs10-ell", "{label}" }
                if s.len() > 1 { span { class: "oxs11-x", "×{s.len()}" } }
                {nico("caret")}
            }
            if open() {
                div { class: "oxscn-menu oxs10-pkm oxs11-mm", "data-up": "true", role: "menu", "aria-label": "Models",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
                    p { class: "oxs11-mh", "Pick more than one to run them side by side" }
                    for (i, (n, _)) in FW_MODELS.iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-mi oxs10-pki", role: "menuitemcheckbox", "aria-checked": "{s.contains(&i)}",
                            onclick: move |_| { if toggle_model(&mut sel.write(), i) { let n2 = sel.peek().len(); said.set(if n2 > 1 { format!("{n2} models in parallel") } else { format!("Model: {n}") }); } },
                            {mk(i, 15.0)} span { class: "oxs10-pkt", b { "{n}" } }
                            span { class: "oxscn-mi-k oxs11-cb", "data-on": "{s.contains(&i)}", if s.contains(&i) { {nico("check")} } }
                        }
                    }
                }
            }
        }
    }
}

/// The Compare view: each model's attempt at the same prompt, ranked, one to apply.
#[component]
fn Compare(
    attempts: Signal<Vec<Attempt>>,
    said: Signal<String>,
    on_view: EventHandler<u32>,
) -> Element {
    let mut attempts = attempts;
    let mut said = said;
    let a = attempts();
    if a.is_empty() {
        return rsx! { div { class: "oxs10-feed oxs10-start", p { class: "oxs10-faint", "No parallel runs yet. Pick more than one model in the composer." } } };
    }
    let best = best_attempt(&a);
    let applied = a.iter().position(|x| x.picked == Some(true));
    rsx! {
        div { class: "oxs10-feed",
            div { class: "oxs10-in oxs11-cin",
                div { class: "oxs10-you", p { "{COMPARE_PROMPT}" } }
                p { class: "oxs11-ch", "{a.len()} models ran this in parallel, each on its own worktree. Compare, then apply one." }
                if let Some(k) = applied {
                    div { class: "oxs10-banner", role: "note", {branch_ico()}
                        span { class: "oxs10-nl", "Applied {FW_MODELS[a[k].model].0}'s changes from " code { "{worktree_for(a[k].agent)}" } " to main." }
                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { reopen_attempts(&mut attempts.write()); said.set("Apply undone; all attempts open".to_string()); }, "Undo" }
                    }
                }
                div { class: "oxs11-cmp", role: "list", "aria-label": "Attempts",
                    for (i, x) in a.iter().cloned().enumerate() {
                        {
                            let (n, _) = FW_MODELS[x.model];
                            let ok = x.tests.0 == x.tests.1;
                            let st = match x.picked { Some(true) => "applied", Some(false) => "discarded", None => "open" };
                            let agent = x.agent;
                            rsx! {
                                div { key: "{i}", class: "oxs11-att", role: "listitem", "data-st": "{st}", "aria-label": "{n} attempt",
                                    div { class: "oxs11-ah", {mk(x.model, 16.0)} b { class: "oxs10-nl", "{n}" }
                                        if Some(i) == best && applied.is_none() { span { class: "oxs11-tag", "Suggested" } }
                                        if st == "applied" { span { class: "oxs11-tag", "data-tone": "ok", "Applied" } }
                                        if st == "discarded" { span { class: "oxs11-tag", "data-tone": "mute", "Discarded" } }
                                    }
                                    div { class: "oxs11-am",
                                        span { class: "oxs11-tst", "data-ok": "{ok}", if ok { {nico("check")} } else { {nico("close")} } "{x.tests.0}/{x.tests.1} tests" }
                                        span { class: "oxs10-st", span { class: "oxs10-add", "+{x.stat.0}" } span { class: "oxs10-del", "−{x.stat.1}" } }
                                        span { class: "oxs10-ago", {nico("clock")} " {mmss(x.secs)}" }
                                    }
                                    p { class: "oxs11-as", "{x.summary}" }
                                    div { class: "oxs11-aa",
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "View changes from {n}", onclick: move |_| on_view.call(agent), "View changes" }
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", "aria-label": "Apply {n}'s changes", disabled: applied.is_some(),
                                            onclick: move |_| { if apply_attempt(&mut attempts.write(), i) { said.set(format!("Applied {n}'s changes; the other attempts were discarded")); } }, "Apply" }
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

#[component]
fn FleetWindowApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<FleetWindowData>,
) -> Element {
    let (st, own) = fw_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| FleetWindowData::for_state(st)));
    let dd = use_hook(|| Signal::new(d0.clone()));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.agents);
    let first = d0.agents.first().map(|a| a.id).unwrap_or(0);
    let open0 = if own == "compare" { 3 } else { first };
    let mut open = use_signal(move || open0);
    let mut view = use_signal(move || if own.is_empty() { "agent" } else { own });
    let d2 = d0.clone();
    let mut files = use_signal(move || d2.diff(open0));
    let d3 = d0.clone();
    let attempts = use_signal(move || d3.attempts);
    let d4 = d0.clone();
    let mut plan = use_signal(move || d4.plan);
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
    let mut pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut mode = use_signal(move || {
        if own == "plan" {
            AgentMode::Plan
        } else {
            AgentMode::Agent
        }
    });
    let sel = use_signal(|| vec![0usize]);
    let mut q = use_signal(String::new);
    let all = list();
    let oid = open();
    let v = view();
    let cur = all.iter().find(|a| a.id == oid).cloned();
    let running = v == "agent" && cur.as_ref().is_some_and(|a| a.state == TState::Running);
    let fs = files();
    let (fa, fr) = oxyn_core::kit::cmdstudio::diff_stat(&fs);
    let pane_label = if fs.is_empty() {
        "Changes".to_string()
    } else {
        format!("Changes +{fa} −{fr}")
    };
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        let m = *mode.peek();
        if m == AgentMode::Plan {
            if let Some(e) = prompt_error(&p) {
                err.set(Some(e));
                said.set(e.to_string());
                return;
            }
            plan.set(plan_for(&p).into_iter().map(|s| (s, false)).collect());
            view.set("plan");
            text.set(String::new());
            err.set(None);
            said.set("Plan ready to review".to_string());
            return;
        }
        let ms = sel.peek().clone();
        let r = spawn(&mut list.write(), &p, &ms);
        match r {
            Ok(ids) => {
                open.set(ids[0]);
                view.set("agent");
                files.set(Vec::new());
                text.set(String::new());
                err.set(None);
                said.set(if ids.len() > 1 {
                    format!("{} agents started in parallel", ids.len())
                } else {
                    "Agent started on its own worktree".to_string()
                });
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let modes: Vec<(String, String, String)> = AgentMode::ALL
        .iter()
        .map(|m| (m.label().to_string(), m.hint().to_string(), String::new()))
        .collect();
    let midx = AgentMode::ALL
        .iter()
        .position(|x| *x == mode())
        .unwrap_or(0);
    let can = prompt_error(&text()).is_none();
    let ql = q().trim().to_lowercase();
    let shown: Vec<_> = all
        .iter()
        .filter(|a| ql.is_empty() || a.title.to_lowercase().contains(&ql))
        .cloned()
        .collect();

    let new_on = oid == 0 && v == "agent";
    let cmp_on = v == "compare";
    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{FW_APP}",
            {super::fleetwin::s11_top(FW_APP, fw_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{new_on}", title: "New agent",
                onclick: move |_| { open.set(0); view.set("agent"); files.set(Vec::new()); drawer.set(false); said.set("New agent".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New agent" } span { class: "oxs10-kbd", "⌘I" } } }
            button { r#type: "button", class: "oxs10-nav", "data-on": "{cmp_on}", "aria-current": if cmp_on { "page" } else { "false" }, title: "Compare",
                onclick: move |_| { view.set("compare"); drawer.set(false); said.set("Compare".to_string()); },
                {nico("layers")} if !rl { span { class: "oxs10-nl", "Compare" } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search agents", "aria-label": "Search agents", value: "{q}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    p { class: "oxs10-sh", "Agents" }
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No agents yet. Start one from the composer." } }
                    else if shown.is_empty() { p { class: "oxs10-faint", "No agents match." } }
                    else {
                        ul { class: "oxs10-ul",
                            for (i, a) in shown.iter().cloned().enumerate() {
                                {
                                    let id = a.id;
                                    let on = id == oid && v == "agent";
                                    let head = a.group > 0 && (i == 0 || shown[i - 1].group != a.group);
                                    let n = shown.iter().filter(|x| x.group == a.group).count();
                                    let title = a.title.clone();
                                    let (mn, _) = FW_MODELS[a.model];
                                    rsx! {
                                        Fragment { key: "{id}",
                                        if head {
                                            li {
                                                button { r#type: "button", class: "oxs10-gh oxs11-grp", "aria-label": "Compare {n} attempts: {title}",
                                                    onclick: move |_| { view.set("compare"); drawer.set(false); said.set("Compare".to_string()); },
                                                    {nico("layers")} span { class: "oxs10-nl", "{a.title}" } span { class: "oxs10-cnt", "×{n}" } }
                                            }
                                        }
                                        li {
                                            button { r#type: "button", class: "oxs10-row oxs11-row", "data-sub": "{a.group > 0}", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); view.set("agent"); files.set(dd.peek().diff(id)); drawer.set(false); said.set(format!("Opened {title}")); },
                                                {tmark(a.state.tone())}
                                                span { class: "oxs10-nl", if a.group > 0 { "{mn}" } else { "{a.title}" } }
                                                {mk(a.model, 13.0)}
                                                if a.stat.0 + a.stat.1 > 0 { span { class: "oxs10-st", span { class: "oxs10-add", "+{a.stat.0}" } span { class: "oxs10-del", "−{a.stat.1}" } } }
                                                else { span { class: "oxs10-ago", "{a.ago}" } }
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
        span { class: "oxs10-ttl oxs10-ell",
            if loading { "" } else if v == "compare" { "Compare · Paginate the invoices API" } else if v == "plan" { "Plan" } else if let Some(a) = &cur { "{a.title}" } else { "New agent" } }
        if let (true, Some(a)) = (v == "agent", &cur) {
            {pill(a.state.label(), a.state.tone())}
            span { class: "oxs10-envc oxs11-wt", {branch_ico()} span { class: "oxs10-ell", "{worktree_for(a.id)}" } }
        }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}", "data-plan": "{mode() == AgentMode::Plan}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Ask the agent", placeholder: "Plan, build, fix anything · @ for files",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add files", onclick: move |_| said.set("Add files: pick from the repo".to_string()), {nico("plus")} }
                Pick { label: "Mode".to_string(), lead: rsx! { {nico(if mode() == AgentMode::Plan { "list" } else if mode() == AgentMode::Ask { "chat" } else { "bot" })} }, items: modes, cur: midx, up: true,
                    on_pick: move |i: usize| { mode.set(AgentMode::ALL[i]); said.set(format!("Mode: {}", AgentMode::ALL[i].label())); } }
                Models { sel, said }
                span { class: "oxs10-grow" }
                if running {
                    button { r#type: "button", class: "oxs10-stopb", "aria-label": "Stop the agent",
                        onclick: move |_| { let id = *open.peek(); if let Some(a) = list.write().iter_mut().find(|a| a.id == id) { a.state = TState::Review; } said.set("Stopped. Changes so far are in Changes".to_string()); }, span { class: "oxs10-sq" } }
                }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if v == "compare" {
        rsx! {
            Compare { attempts, said, on_view: move |id: u32| { files.set(dd.peek().diff(id)); pane.set(true); pnar.set(true); said.set(format!("Changes from {}", worktree_for(id))); } }
            div { class: "oxs10-foot", {composer} }
        }
    } else if v == "plan" {
        let p = plan();
        let (dn, tn) = plan_progress(&p);
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { "{FW_PROMPT}" } }
                    div { class: "oxs11-plan", role: "group", "aria-label": "Plan",
                        div { class: "oxs11-ph", {nico("list")} b { "Plan" } span { class: "oxs10-grow" } span { class: "oxs10-ago", "{dn} of {tn} checked" } }
                        ol { class: "oxs11-pl",
                            for (i, (t, done)) in p.iter().cloned().enumerate() {
                                li { key: "{i}",
                                    label { class: "oxs11-pi",
                                        input { r#type: "checkbox", checked: done, onchange: move |_| { if let Some(s) = plan.write().get_mut(i) { s.1 = !s.1; } } }
                                        span { "{t}" }
                                    }
                                }
                            }
                        }
                        div { class: "oxs11-aa",
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", disabled: p.is_empty(),
                                onclick: move |_| { mode.set(AgentMode::Agent); let ms = sel.peek().clone(); if let Ok(ids) = spawn(&mut list.write(), "Build the approved plan for upload retries", &ms) { open.set(ids[0]); files.set(Vec::new()); } view.set("agent"); said.set("Building the plan".to_string()); }, "Build" }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { mode.set(AgentMode::Agent); view.set("agent"); said.set("Plan dismissed".to_string()); }, "Dismiss" }
                        }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else if let Some(a) = cur.clone() {
        let st = a.state;
        let sample = a.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{FW_PROMPT}" } else { "{a.title}" } } }
                    if sample {
                        ol { class: "oxs10-work", "aria-label": "Steps",
                            for (i, (g, l, d)) in FW_STEPS.iter().copied().enumerate() {
                                {
                                    let last = i == 4;
                                    let live = last && st == TState::Running;
                                    let l2 = if last && !live { "Ran cargo test retry" } else { l };
                                    let d2 = if last && !live { "3 passed" } else { d };
                                    rsx! {
                                        li { key: "{i}", class: "oxs10-wi", "data-live": "{live}",
                                            if live { span { class: "oxs10-spin" } } else { {nico(g)} }
                                            span { class: "oxs10-nl", "{l2}" } span { class: "oxs10-wd", "{d2}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if st == TState::Running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            let fresh = if id == 1 { thread_diff(1) } else { Vec::new() };
                            let s2 = oxyn_core::kit::cmdstudio::diff_stat(&fresh);
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id && x.state == TState::Running) { x.state = TState::Review; x.stat = s2; }
                            if id == 1 { files.set(fresh); }
                            said.set("Agent finished · changes ready to review".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if sample { "Running tests · 4 of 5 steps" } else { "Exploring the repo" } } }
                    } else {
                        div { class: "oxs10-agent",
                            p { if sample { "Uploads now retry a 503 up to 4 times with exponential backoff and jitter. A new test fails twice, then succeeds; all 3 retry tests pass." } else if a.stat.0 + a.stat.1 > 0 { "Done. The changes are on this agent's worktree; review them, then keep or revert." } else { "Finished without changing any files." } }
                            if !fs.is_empty() {
                                div { class: "oxs10-chg",
                                    span { {nico("doc")} } span { class: "oxs10-nl", "{oxyn_core::kit::cmdstudio::review_line(&fs)}" }
                                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs10-narrow-only", onclick: move |_| pnar.set(true), "Review" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        let s = sel();
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {fw_mark(34)} }
                h1 { class: "oxs10-h1", "What should the agents work on?" }
                div { class: "oxs10-under",
                    span { class: "oxs10-envc", {nico("folder")} span { "relay-api" } }
                    span { class: "oxs10-envc", {branch_ico()} span { "New worktree each" } }
                    span { class: "oxs10-envc", if s.len() > 1 { "{s.len()} models in parallel" } else { "{FW_MODELS[s[0]].0}" } }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel = rsx! { DiffPane { files, loading, said, title: "Changes".to_string() } };
    rsx! { Frame { app: "fw", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label } }
}

/// Fleet Window: the agents-window desktop coding app. Agents in the sidebar, each on its own
/// worktree with its model and lines changed; the open agent's steps and a changes pane (accept or
/// revert per hunk); Agent / Plan / Ask; and parallel runs: one prompt on several models, compared
/// by tests, size and time, with one applied. States `compare` and `plan`.
#[component]
pub fn FleetWindowScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `compare` or `plan`).
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
    /// Your data. `None` shows the sample for `state`; start from `FleetWindowData::default()` and
    /// replace what you have.
    #[props(default)]
    data: Option<FleetWindowData>,
) -> Element {
    rsx! { FleetWindowApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
