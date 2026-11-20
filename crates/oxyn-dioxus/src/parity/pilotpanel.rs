//! Screens S16 (Agent apps IV): **Pilot Panel**, an editor's agent panel as a desktop app. Task
//! history with cost; the task as steps (read, notes, edits, a command) with a checkpoint after
//! each change to Compare or Restore; Plan / Act; a command that waits on Approve / Reject unless
//! auto-approve covers it; a context meter with tokens and cost; an Editor pane with the diff and
//! a Checkpoints tab. Rules in `oxyn_core::parity::pilotpanel`; styles in `pilotpanel.css`
//! (`oxs16k-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::parity::pilotpanel::{
    PK_APP, PK_AUTO, PK_CTX, PK_DIFF, PK_FILE, PK_MODELS, PK_PENDING, PK_PLAN, PK_PROMPT,
    PK_STATES, PK_TABS, PilotPanelData, PkStep, ScreenState, add_task, checkpoints, ctx_pct, kilo,
    needs_approval, pk_split, prompt_error, restore, usd,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Pilot Panel (for a host's `ScreenStage { states }`).
pub fn s16_pilotpanel_states() -> Vec<(&'static str, &'static str)> {
    PK_STATES.to_vec()
}

fn pk_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            path { d: "M10 2.5l6.5 15-6.5-3.5-6.5 3.5z", fill: "none", stroke: "currentColor", stroke_width: "1.7", stroke_linejoin: "round" }
        }
    }
}

fn step_verb(kind: &str) -> &'static str {
    match kind {
        "read" => "Read",
        "edit" => "Edited",
        _ => "Ran",
    }
}

fn step_glyph(kind: &str) -> &'static str {
    match kind {
        "read" => "file",
        "edit" => "pen",
        "cmd" => "term",
        _ => "chat",
    }
}

/// The pane: the edited file's diff, and the checkpoints to restore.
#[component]
fn PkPane(
    tab: Signal<usize>,
    steps: Signal<Vec<PkStep>>,
    loading: bool,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut steps = steps;
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    let s = steps();
    if s.is_empty() {
        return rsx! {
            div { class: "oxs10-pempty", {nico("file")} p { "Files the agent edits, and the checkpoints after each change, show here." } }
        };
    }
    let cps = checkpoints(&s);
    let edited = s.iter().any(|x| x.kind == "edit");
    rsx! {
        div { class: "oxs16k-tabs", role: "tablist", "aria-label": "Editor",
            onkeydown: move |e: KeyboardEvent| { if matches!(e.key(), Key::ArrowRight | Key::ArrowLeft) { let now = *tab.peek(); tab.set(1 - now); } },
            for (i, (k, l)) in PK_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16k-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}"
                    if k == "checkpoints" { span { class: "oxs16k-cnt", "{cps.len()}" } }
                }
            }
        }
        div { class: "oxs16k-body", role: "tabpanel", "aria-label": "{PK_TABS[t].1}",
            if t == 0 {
                if edited {
                    div { class: "oxs16k-fh", {nico("file")} span { class: "oxs10-nl oxs16k-mono", "{PK_FILE}" } span { class: "oxs16k-add", "+14" } span { class: "oxs16k-del", "−3" } }
                    pre { class: "oxs16k-diff", "aria-label": "Diff of {PK_FILE}",
                        for (i, (sg, l)) in PK_DIFF.iter().copied().enumerate() {
                            span { key: "{i}", class: "oxs16k-dl", "data-s": "{sg}", span { class: "oxs16k-sg", "{sg}" } "{l}" }
                        }
                    }
                } else {
                    p { class: "oxs10-faint oxs16k-pf", "No edits yet in this task." }
                }
            } else {
                ol { class: "oxs16k-cps", "aria-label": "Checkpoints",
                    for (n, i) in cps.iter().copied().enumerate() {
                        li { key: "{i}",
                            span { class: "oxs16k-cpd", {nico("pin")} }
                            span { class: "oxs16k-cpt", b { "Checkpoint {n + 1}" } span { class: "oxs10-ell", "After: {s[i].title}" } }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Restore checkpoint {n + 1}",
                                onclick: move |_| { restore(&mut steps.write(), i); said.set(format!("Restored to checkpoint {}", n + 1)); }, {nico("undo")} "Restore" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PilotPanelApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<PilotPanelData>,
) -> Element {
    let (st, own) = pk_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| PilotPanelData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.tasks);
    let s0 = if own == "plan" {
        d0.steps.iter().take(1).copied().collect::<Vec<PkStep>>()
    } else {
        d0.steps.clone()
    };
    let mut steps = use_signal(move || s0);
    let first = d0.tasks.first().map(|t| t.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let mut tab = use_signal(|| 0usize);
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
    let mut model = use_signal(|| 0usize);
    let mut plan = use_signal(move || own == "plan");
    let mut auto = use_signal(|| [true, false, false]);
    let mut auto_open = use_signal(|| false);
    // The pending command: None until asked, Some(true) ran, Some(false) rejected.
    let mut pending = use_signal(|| None::<bool>);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let rl = side() == "rail";
    let asking = own == "approval" && pending().is_none();
    let waits = asking && needs_approval("cmd", auto(), plan());
    let mut submit = move || {
        let p = text.peek().clone();
        match add_task(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set(format!(
                    "New task in {}",
                    if *plan.peek() { "Plan" } else { "Act" }
                ));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let models = opts2(&PK_MODELS);
    let mk = PK_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let pct = ctx_pct(PK_CTX.0, PK_CTX.1);

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{PK_APP}",
            {s11_top(PK_APP, pk_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New task",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New task".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New task" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                div { class: "oxs10-list",
                    p { class: "oxs10-sh", "History" }
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(6)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No tasks yet. Describe one below to start." } }
                    else {
                        ul { class: "oxs10-ul",
                            for c in all.iter().cloned() {
                                {
                                    let id = c.id;
                                    let on = id == oid;
                                    let t2 = c.title.clone();
                                    let day = c.at.get(..10).unwrap_or(c.at);
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs10-row oxs16k-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs16k-k", {nico("chat")} } }
                                                span { class: "oxs16k-rt", span { class: "oxs10-ell", "{c.title}" } span { class: "oxs16k-rm", "{day} · {usd(c.cents)}" } }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New task" } }
        if cur.is_some() {
            if waits { {pill("Needs approval", "wait")} } else if plan() { {pill("Plan", "")} } else { {pill("Act", "ok")} }
        }
        span { class: "oxs10-grow" }
        if cur.is_some() && !loading {
            span { class: "oxs16k-ctx", title: "Context window", "aria-label": "Context {kilo(PK_CTX.0)} of {kilo(PK_CTX.1)} tokens, {pct} percent",
                span { class: "oxs16k-meter", span { class: "oxs16k-fill", style: "width:{pct}%" } }
                span { class: "oxs16k-ctxt", "{kilo(PK_CTX.0)} / {kilo(PK_CTX.1)}" }
            }
            if let Some(c) = &cur { span { class: "oxs16k-cost", "{usd(c.cents)}" } }
        }
    };

    let composer = rsx! {
        if auto_open() {
            div { class: "oxs16k-auto", role: "group", "aria-label": "Auto-approve",
                b { "Auto-approve" }
                for (i, (_, l)) in PK_AUTO.iter().copied().enumerate() {
                    label { key: "{i}", class: "oxs16k-ab",
                        input { r#type: "checkbox", checked: auto()[i], onchange: move |_| { let mut a = *auto.peek(); a[i] = !a[i]; auto.set(a); said.set(format!("Auto-approve {}: {}", l, if a[i] { "on" } else { "off" })); } }
                        span { "{l}" }
                    }
                }
            }
        }
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: if plan() { "Ask, or plan the change · @ to add files" } else { "Describe the change · @ to add files" },
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add context", onclick: move |_| said.set("Add context: files, folders, problems, a URL".to_string()), {nico("plus")} }
                div { class: "oxs16k-seg", role: "group", "aria-label": "Mode",
                    button { r#type: "button", class: "oxs16k-sb", "aria-pressed": "{plan()}", onclick: move |_| { plan.set(true); said.set("Plan mode: reads and proposes, changes nothing".to_string()); }, "Plan" }
                    button { r#type: "button", class: "oxs16k-sb", "aria-pressed": "{!plan()}", onclick: move |_| { plan.set(false); said.set("Act mode: edits files and runs commands".to_string()); }, "Act" }
                }
                button { r#type: "button", class: "oxs10-chip oxs16k-autob", "aria-expanded": "{auto_open()}", "aria-label": "Auto-approve settings", onclick: move |_| auto_open.toggle(),
                    {nico("shield")} span { class: "oxs10-ell", "Auto-approve" } }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", PK_MODELS[i].0)); } }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let s = steps();
    let cps = checkpoints(&s);
    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{PK_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        ol { class: "oxs16k-steps", "aria-label": "Steps",
                            for (i, x) in s.iter().copied().enumerate() {
                                li { key: "{i}", class: "oxs16k-st", "data-k": "{x.kind}",
                                    if x.kind == "say" {
                                        p { class: "oxs16k-say", "{x.title}" }
                                    } else {
                                        div { class: "oxs16k-tool",
                                            span { class: "oxs16k-tg", {nico(step_glyph(x.kind))} }
                                            span { class: "oxs16k-verb", "{step_verb(x.kind)}" }
                                            span { class: "oxs10-nl oxs16k-mono", "{x.title}" }
                                            span { class: "oxs16k-det", "{x.detail}" }
                                            if x.kind == "edit" {
                                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { tab.set(0); pane.set(true); pnar.set(true); said.set("Opened the diff".to_string()); }, "Diff" }
                                            }
                                        }
                                    }
                                    if let Some(n) = cps.iter().position(|j| *j == i) {
                                        div { class: "oxs16k-cp", role: "group", "aria-label": "Checkpoint {n + 1}",
                                            span { class: "oxs16k-cpl" } span { class: "oxs16k-cpn", {nico("pin")} "Checkpoint" }
                                            button { r#type: "button", class: "oxs16k-cpb", onclick: move |_| { tab.set(1); pane.set(true); pnar.set(true); said.set(format!("Compare checkpoint {}", n + 1)); }, "Compare" }
                                            button { r#type: "button", class: "oxs16k-cpb", onclick: move |_| { restore(&mut steps.write(), i); said.set(format!("Restored to checkpoint {}", n + 1)); }, "Restore" }
                                            span { class: "oxs16k-cpl" }
                                        }
                                    }
                                }
                            }
                        }
                        if plan() && own == "plan" {
                            div { class: "oxs16k-plan", role: "group", "aria-label": "Proposed plan",
                                div { class: "oxs16k-ph", {nico("list")} b { "Plan" } span { class: "oxs10-grow" } span { class: "oxs16k-det", "Nothing changed yet" } }
                                ol { class: "oxs16k-pl", for (i, l) in PK_PLAN.iter().copied().enumerate() { li { key: "{i}", "{l}" } } }
                                div { class: "oxs16k-pb",
                                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| { plan.set(false); said.set("Switched to Act".to_string()); }, "Switch to Act" }
                                }
                            }
                        }
                        if own == "approval" {
                            if waits {
                                div { class: "oxs16k-ask", role: "alertdialog", "aria-label": "Run this command?",
                                    p { class: "oxs16k-ah", {nico("hand")} b { "{PK_APP} wants to run a command" } }
                                    pre { class: "oxs16k-cmd", "$ {PK_PENDING}" }
                                    p { class: "oxs16k-det", "Changes the staging database. Run commands is not on auto-approve." }
                                    div { class: "oxs16k-pb",
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { pending.set(Some(false)); said.set("Command rejected".to_string()); }, "Reject" }
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| { pending.set(Some(true)); said.set("Command approved".to_string()); }, "Approve" }
                                    }
                                }
                            } else if pending() == Some(false) {
                                p { class: "oxs16k-res", {nico("close")} "Rejected · the agent will find another way" }
                            } else {
                                p { class: "oxs16k-res", "data-ok": "true", {nico("check")} "Ran {PK_PENDING} · 2 migrations applied" }
                            }
                        }
                        if !plan() && own != "approval" && own != "plan" {
                            div { class: "oxs10-agent", p { "Done. /orders now takes a cursor and a limit (at most 50) and returns next_cursor. The test covers two pages and the cap." } }
                        }
                    } else if c.running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.running = false; }
                            said.set("Task finished".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if plan() { "Reading the project" } else { "Working" } } }
                    } else {
                        div { class: "oxs10-agent", p { "Finished. Open a checkpoint to compare, or ask for a follow-up." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {pk_mark(34)} }
                h1 { class: "oxs10-h1", "What can I do for you?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Explain this file", "Fix the failing test", "Add input validation"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16k-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel = rsx! { PkPane { tab, steps, loading, said } };
    rsx! { Frame { app: "pk", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Editor".to_string() } }
}

/// Pilot Panel: an editor's agent panel as a desktop app. Task history with cost; steps with a
/// checkpoint after each change (Compare, Restore); Plan / Act; a command waiting on Approve or
/// Reject unless auto-approve covers it; a context meter; an Editor pane with the diff and the
/// checkpoints. States `plan` and `approval`.
#[component]
pub fn PilotPanelScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `plan` or `approval`).
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
    /// Your data. `None` shows the sample for `state`; start from `PilotPanelData::default()`.
    #[props(default)]
    data: Option<PilotPanelData>,
) -> Element {
    rsx! { PilotPanelApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
