//! Screens S16 (Agent apps IV): **Tabstack**, an agent-first terminal. Vertical tabs with the
//! agent's mark, branch, worktree and pull request; a terminal of blocks (command, output, exit
//! status, time) with the agent's turns between them; a command the agent wants to run waits on
//! Run / Edit / Deny; an input that switches between Terminal and Agent with Profile and Model;
//! a pane with Agents (local and cloud runs) and Review. Rules in `oxyn_core::kit::tabstack`;
//! styles in `tabstack.css` (`oxs16w-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::kit::tabstack::{
    Block, ScreenState, TS_APP, TS_DIFF, TS_MODELS, TS_PANE, TS_PENDING, TS_PROFILES, TS_RUNS,
    TS_STATES, TabstackData, TsState, add_tab, prompt_error, run_cmd, took, ts_split, waiting,
};

use super::cmdstudio::{Frame, Pick, branch_ico, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Tabstack (for a host's `ScreenStage { states }`).
pub fn s16_tabstack_states() -> Vec<(&'static str, &'static str)> {
    TS_STATES.to_vec()
}

fn ts_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "2", y: "3", width: "16", height: "14", rx: "3", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M5.5 8l2.5 2-2.5 2", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round" }
            path { d: "M10 13h4.5", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round" }
        }
    }
}

fn agent_mark(id: &str, size: f32) -> Element {
    if id.is_empty() {
        rsx! { span { class: "oxs16w-am", {ts_mark(size as u32)} } }
    } else {
        rsx! { span { class: "oxs10-mk oxs16w-am", BrandIcon { name: id.to_string(), size, coloured: true } } }
    }
}

/// The pane: Agents (runs) and Review (the session's diff).
#[component]
fn Side(
    tab: Signal<usize>,
    loading: bool,
    empty: bool,
    open: Signal<u32>,
    ids: Vec<(String, u32)>,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut verdict = use_signal(|| None::<&'static str>);
    let mut open = open;
    let oid = open();
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("bot")} p { "Agent runs, local and in the cloud, and their changes to review show up here." } }
        };
    }
    rsx! {
        div { class: "oxs16w-tabs", role: "tablist", "aria-label": "Agents and review",
            onkeydown: move |e: KeyboardEvent| { if matches!(e.key(), Key::ArrowRight | Key::ArrowLeft) { let now = *tab.peek(); tab.set(1 - now); } },
            for (i, (k, l)) in TS_PANE.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16w-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}" }
            }
        }
        div { class: "oxs16w-pb", role: "tabpanel", "aria-label": "{TS_PANE[t].1}",
            if t == 0 {
                for wh in ["Local", "Cloud"] {
                    div { key: "{wh}",
                        p { class: "oxs10-sh oxs16w-wh", if wh == "Cloud" { {nico("cloud")} } else { {nico("laptop")} } span { "{wh}" } }
                        ul { class: "oxs16w-runs",
                            for (i, (title, _, s, at)) in TS_RUNS.iter().copied().filter(|r| r.1 == wh).enumerate() {
                                {
                                let tid = ids.iter().find(|x| x.0 == title).map(|x| x.1);
                                rsx! {
                                li { key: "{i}",
                                    button { r#type: "button", class: "oxs16w-run", "data-on": "{tid == Some(oid)}",
                                        onclick: move |_| { if let Some(t) = tid { open.set(t); said.set(format!("Opened {title}")); } else { said.set(format!("{title} ran in the cloud; its log opens in the browser")); } },
                                        span { class: "oxs10-dot", "data-tone": "{s.tone()}", "aria-hidden": "true" }
                                        span { class: "oxs10-s2", b { class: "oxs10-ell", "{title}" } span { class: "oxs10-ssub oxs10-ell", "Started {at}" } }
                                        span { class: "oxs16w-st", match s { TsState::Working => "Working", TsState::Waiting => "Needs you", TsState::Done => "Done", TsState::Shell => "" } }
                                    }
                                }
                                }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs16w-rv",
                    div { class: "oxs16w-rh", {nico("file")} span { class: "oxs10-nl oxs16w-mono", "tests/upload.rs" } span { class: "oxs16w-add", "+3" } span { class: "oxs16w-del", "−2" } }
                    pre { class: "oxs16w-diff",
                        for (i, (s, l)) in TS_DIFF.iter().copied().enumerate() {
                            span { key: "{i}", class: "oxs16w-dl", "data-s": "{s}", span { class: "oxs16w-sg", "{s}" } "{l}" }
                        }
                    }
                    div { class: "oxs16w-rb",
                        if let Some(v) = verdict() {
                            span { class: "oxs16w-ok", {nico("check")} span { class: "oxs10-ell", "{v}" } }
                        } else {
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { verdict.set(Some("Changes requested")); said.set("Requested changes".to_string()); }, "Request changes" }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| { verdict.set(Some("Approved · ready to open a pull request")); said.set("Approved".to_string()); }, "Approve" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabstackApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<TabstackData>,
) -> Element {
    let (st, own) = ts_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| TabstackData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || {
        let mut l = d1.tabs;
        if own == "approval"
            && let Some(t) = l.first_mut()
        {
            t.state = TsState::Waiting;
        }
        l
    });
    let first = d0.tabs.first().map(|t| t.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let b0 = d0.blocks.clone();
    let mut blocks = use_signal(move || b0);
    let mut pending = use_signal(move || own == "approval");
    let ptab = use_signal(move || if own == "review" { 1 } else { 0 });
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
    let pnar = use_signal(move || own == "review");
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut agent_mode = use_signal(|| true);
    let mut model = use_signal(|| 0usize);
    let mut profile = use_signal(|| 0usize);
    let mut sel = use_signal(|| None::<usize>);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let running = cur.as_ref().is_some_and(|c| c.state == TsState::Working);
    let rl = side() == "rail";
    let nwait = waiting(&all);
    let mut submit = move || {
        let p = text.peek().trim().to_string();
        if !*agent_mode.peek() {
            if p.is_empty() {
                return;
            }
            let (out, code) = run_cmd(&p);
            blocks.write().push(Block::Cmd(p.clone(), out, code, 120));
            text.set(String::new());
            said.set(format!("{p} exited {code}"));
            return;
        }
        if *open.peek() == 0 {
            match add_tab(&mut list.write(), &p) {
                Ok(id) => {
                    open.set(id);
                    text.set(String::new());
                    err.set(None);
                    said.set("New agent tab".to_string());
                }
                Err(e) => {
                    err.set(Some(e));
                    said.set(e.to_string());
                }
            }
            return;
        }
        if let Some(e) = prompt_error(&p) {
            err.set(Some(e));
            said.set(e.to_string());
            return;
        }
        let mut b = blocks.write();
        b.push(Block::Ask(p));
        b.push(Block::Say(
            "On it. I'll run what I need and ask before anything that writes.".to_string(),
        ));
        drop(b);
        text.set(String::new());
        err.set(None);
        said.set("Sent to the agent".to_string());
    };
    let profiles: Vec<(String, String, String)> = TS_PROFILES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let models = opts2(&TS_MODELS);
    let mk = TS_MODELS[model()].1;
    let am = agent_mode();
    let can = if am {
        prompt_error(&text()).is_none()
    } else {
        !text().trim().is_empty()
    };

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{TS_APP}",
            {s11_top(TS_APP, ts_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New tab",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New tab".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New tab" } span { class: "oxs10-kbd", "⌘T" } } }
            if !rl {
                p { class: "oxs10-sh", "Tabs" }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(6)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No tabs open. Start an agent or a shell." } }
                    else {
                        ul { class: "oxs10-ul",
                            for c in all.iter().cloned() {
                                {
                                    let id = c.id;
                                    let on = id == oid;
                                    let t2 = c.title.clone();
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs16w-vt", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); drawer.set(false); sel.set(None); said.set(format!("Opened {t2}")); },
                                                if c.state == TsState::Shell { span { class: "oxs16w-am oxs16w-sh", {nico("term")} } } else { {agent_mark(c.agent, 15.0)} }
                                                span { class: "oxs16w-vtt",
                                                    span { class: "oxs16w-vh", b { class: "oxs10-ell", "{c.title}" }
                                                        if c.state != TsState::Shell { span { class: "oxs10-dot", "data-tone": "{c.state.tone()}", "aria-hidden": "true" } } }
                                                    span { class: "oxs16w-meta",
                                                        span { class: "oxs16w-bri", {branch_ico()} span { class: "oxs10-ell", "{c.branch}" } }
                                                        if c.pr > 0 { span { class: "oxs16w-pr", "#{c.pr}" } }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New tab" } }
        if let Some(c) = &cur {
            match c.state { TsState::Working => pill("Working", "run"), TsState::Waiting => pill("Needs you", "wait"), TsState::Done => pill("Done", "ok"), TsState::Shell => pill("Shell", "") }
            span { class: "oxs16w-wt oxs10-wide-only", {nico("folder")} span { class: "oxs10-ell", "{c.worktree}" } }
        }
        span { class: "oxs10-grow" }
        button { r#type: "button", class: "oxscn-ib oxs16w-bell", "aria-label": "Needs approval: {nwait}",
            onclick: move |_| { if let Some(t) = list.peek().iter().find(|t| t.state == TsState::Waiting) { open.set(t.id); said.set(format!("{} needs you", t.title)); } else { said.set("Nothing needs you".to_string()); } },
            {nico("bell")} if nwait > 0 { span { class: "oxs16w-badge", "{nwait}" } } }
    };

    let input = rsx! {
        div { class: "oxs10-comp oxs16w-in", "data-bad": "{err().is_some()}", "data-agent": "{am}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": if am { "Message the agent" } else { "Command" },
                placeholder: if am { "Ask the agent · it asks before it writes" } else { "Type a command, then Enter" },
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                div { class: "oxs16w-seg", role: "group", "aria-label": "Input mode",
                    button { r#type: "button", "aria-pressed": "{!am}", onclick: move |_| { agent_mode.set(false); err.set(None); said.set("Terminal mode".to_string()); }, {nico("term")} span { "Terminal" } }
                    button { r#type: "button", "aria-pressed": "{am}", onclick: move |_| { agent_mode.set(true); said.set("Agent mode".to_string()); }, {nico("spark")} span { "Agent" } }
                }
                if am {
                    Pick { label: "Profile".to_string(), lead: rsx! { {nico(TS_PROFILES[profile()].2)} }, items: profiles, cur: profile(), up: true,
                        on_pick: move |i: usize| { profile.set(i); said.set(format!("Profile: {}", TS_PROFILES[i].0)); } }
                }
                span { class: "oxs10-grow" }
                if am {
                    Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                        on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", TS_MODELS[i].0)); } }
                }
                button { r#type: "button", class: "oxs10-send", "aria-label": if am { "Send" } else { "Run" }, disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(c) = cur.clone() {
        let mine = c.id == 1;
        let bl = if mine { blocks() } else { Vec::new() };
        let wt = c.worktree;
        let br = c.branch;
        rsx! {
            div { class: "oxs10-feed oxs16w-feed",
                div { class: "oxs16w-blocks", role: "list", "aria-label": "Blocks",
                    if !mine {
                        p { class: "oxs10-faint", if c.state == TsState::Shell { "A fresh shell in {wt}." } else { "This session's blocks live in its own worktree, {wt}." } }
                    }
                    for (i, b) in bl.iter().cloned().enumerate() {
                        match b {
                            Block::Cmd(cmd, out, code, ms) => { let cq = cmd.clone(); rsx! {
                                div { key: "{i}", role: "listitem", class: "oxs16w-blk", "data-on": "{sel() == Some(i)}", "data-bad": "{code != 0}", tabindex: "0",
                                    onclick: move |_| sel.set(Some(i)),
                                    div { class: "oxs16w-bh", span { class: "oxs10-ell", "{wt}" } span { class: "oxs16w-bri", {branch_ico()} span { class: "oxs10-ell", "{br}" } } }
                                    div { class: "oxs16w-bc", span { class: "oxs16w-pr2", "❯" } span { class: "oxs10-nl", "{cmd}" } }
                                    pre { class: "oxs16w-bo", for (j, l) in out.iter().enumerate() { span { key: "{j}", "{l}" } } }
                                    div { class: "oxs16w-bf",
                                        if code == 0 { span { class: "oxs16w-ex", "data-ok": "true", {nico("check")} "exit 0" } } else { span { class: "oxs16w-ex", "data-ok": "false", {nico("close")} "exit {code}" } }
                                        span { class: "oxs10-ago", "{took(ms)}" }
                                        span { class: "oxs10-grow" }
                                        if code != 0 {
                                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs16w-ask", onclick: move |e| { e.stop_propagation(); agent_mode.set(true); text.set(format!("Why did `{cq}` fail?")); said.set("Ask the agent about this block".to_string()); }, {nico("spark")} span { "Ask agent" } }
                                        }
                                    }
                                }
                            } },
                            Block::Ask(t) => rsx! { div { key: "{i}", role: "listitem", class: "oxs10-you oxs16w-you", p { "{t}" } } },
                            Block::Say(t) => rsx! { div { key: "{i}", role: "listitem", class: "oxs16w-say", {agent_mark("", 16.0)} p { "{t}" } } },
                        }
                    }
                    if mine && pending() {
                        div { class: "oxs16w-appr", role: "alertdialog", "aria-label": "The agent wants to run a command",
                            p { class: "oxs16w-ah", {nico("hand")} b { "The agent wants to run" } span { class: "oxs10-ago", "{TS_PROFILES[profile()].0} profile" } }
                            pre { class: "oxs16w-ac", "{TS_PENDING}" }
                            div { class: "oxs16w-ab",
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { pending.set(false); blocks.write().push(Block::Say("Skipped. Tell me what to run instead.".to_string())); if let Some(t) = list.write().first_mut() { t.state = TsState::Done; } said.set("Denied".to_string()); }, "Deny" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { pending.set(false); agent_mode.set(false); text.set(TS_PENDING.to_string()); if let Some(t) = list.write().first_mut() { t.state = TsState::Done; } said.set("Edit the command, then Enter".to_string()); }, "Edit" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| {
                                    pending.set(false);
                                    let mut b = blocks.write();
                                    b.push(Block::Cmd(TS_PENDING.to_string(), vec!["running 9 tests".to_string(), "test upload::resumes_after_drop ... ok".to_string(), "test result: ok. 9 passed; 0 failed".to_string()], 0, 5_870));
                                    b.push(Block::Say("Passes now, 50 runs in a row with 8 threads. The change is in Review.".to_string()));
                                    drop(b);
                                    if let Some(t) = list.write().first_mut() { t.state = TsState::Done; }
                                    said.set("Ran the command".to_string());
                                }, "Run" span { class: "oxs10-kbd", "⏎" } }
                            }
                        }
                    }
                    if mine && running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            if let Some(x) = list.write().first_mut() { x.state = TsState::Waiting; }
                            pending.set(true);
                            said.set("The agent needs approval".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", "Editing tests/upload.rs" } }
                    }
                }
            }
            div { class: "oxs10-foot", {input} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {ts_mark(34)} }
                h1 { class: "oxs10-h1", "What should the agent do?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Find why CI is slow and fix it", "Review my staged changes", "Set up a new Rust workspace"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16w-sug", onclick: move |_| { text.set(s.to_string()); agent_mode.set(true); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {input} }
        }
    };
    let empty = d0.tabs.is_empty();
    let ids: Vec<(String, u32)> = all.iter().map(|t| (t.title.clone(), t.id)).collect();
    let panel = rsx! { Side { tab: ptab, loading, empty, open, ids, said } };
    rsx! { Frame { app: "ts", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Agents".to_string() } }
}

/// Tabstack: an agent-first terminal. Vertical tabs with agent marks, branch, worktree and pull
/// request; command blocks with output, exit status and time; the agent's turns between them and
/// a command it asks to run (Run / Edit / Deny); Terminal / Agent input with Profile and Model; a
/// pane with Agents and Review. States `approval` and `review`.
#[component]
pub fn TabstackScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `approval` or `review`).
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
    /// Your data. `None` shows the sample for `state`; start from `TabstackData::default()`.
    #[props(default)]
    data: Option<TabstackData>,
) -> Element {
    rsx! { TabstackApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
