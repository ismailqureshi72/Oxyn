//! Screens S9 (Agent apps I): Taskdeck, a desktop agent app in the task-runner pattern. One app
//! with three views that switch in place: Start (the task composer with attach, run mode, model,
//! voice, folder and Local / Cloud), Task (the running task's step timeline with tool calls, an
//! approval, follow-ups and a side panel of artifacts, files, terminal and browser) and the task
//! Board. The sidebar holds New task, Tasks, Plugins & skills, the Local / Cloud switch, projects,
//! recent tasks and the account row with the inbox bell and the account menu. Rules in
//! `oxyn_core::kit::taskdeck`; styles in `taskdeck.css` (`oxs9-`). Chrome stays monochrome;
//! colour is for status, file types and model marks.

use dioxus::prelude::*;
use oxyn_core::kit::screensfid::thousands;
use oxyn_core::kit::taskdeck::{
    ADD_MENU, APP, BROWSED, CADENCES, CATALOG, COLUMNS, EnvMode, LastRun, MODELS, MODES,
    PANEL_TABS, PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, PLUGIN_CATS, PROJECTS, PROMPT,
    SHEET, SUGGEST, ScreenState, Status, Step, StepState, TREE, Task, account_menu, add_task,
    advance, approve, artifacts, card_acts, delete_task, elapsed, fmt_secs, headline, matches,
    move_task, progress, prompt_error, run_for, run_status, split, states_for, terminal_lines,
    usage, view_of,
};
use oxyn_core::kit::taskdeck::{
    Schedule, Site, SiteStatus, Skill, TaskdeckData, cadence_words, finish_deploys, next_run,
    plugins_in, redeploy, schedule_error, skill_error, view_title,
};

use super::screensfid::{file_ico, ico, nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of each S9 screen (for a host's `ScreenStage { states }`).
pub fn s9_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn status_pill(s: Status) -> Element {
    let t = s.tone();
    let l = s.label();
    rsx! { span { class: "oxs9-pill", "data-tone": "{t}", span { class: "oxs9-dot" } "{l}" } }
}

fn app_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs9-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "1", y: "3", width: "18", height: "4", rx: "2", fill: "currentColor" }
            rect { x: "1", y: "9", width: "13", height: "4", rx: "2", fill: "currentColor", opacity: ".6" }
            rect { x: "1", y: "15", width: "8", height: "4", rx: "2", fill: "currentColor", opacity: ".3" }
        }
    }
}

fn step_mark(s: StepState) -> Element {
    match s {
        StepState::Done => rsx! { span { class: "oxs9-sm", "data-s": "done", {nico("check")} } },
        StepState::Running => {
            rsx! { span { class: "oxs9-sm", "data-s": "run", span { class: "oxs9-spin" } } }
        }
        StepState::Waiting => rsx! { span { class: "oxs9-sm", "data-s": "wait", {nico("hand")} } },
        StepState::Skipped => rsx! { span { class: "oxs9-sm", "data-s": "skip", {nico("close")} } },
        StepState::Pending => rsx! { span { class: "oxs9-sm", "data-s": "pend" } },
    }
}

/// The app's shared state.
#[derive(Clone, Copy, PartialEq)]
struct Ctx {
    tasks: Signal<Vec<Task>>,
    view: Signal<&'static str>,
    open: Signal<u32>,
    env: Signal<EnvMode>,
    steps: Signal<Vec<Step>>,
    said: Signal<String>,
    drawer: Signal<bool>,
    /// The full prompt of the task started last: id and text.
    asked: Signal<(u32, String)>,
    /// Plugins: (installed, on) per catalogue entry.
    plugs: Signal<Vec<(bool, bool)>>,
    /// Skills.
    skills: Signal<Vec<Skill>>,
    /// Scheduled tasks.
    scheds: Signal<Vec<Schedule>>,
    /// Deployed sites.
    sites: Signal<Vec<Site>>,
}

impl Ctx {
    fn open_task(mut self, id: u32) {
        let t = self.tasks.peek().iter().find(|t| t.id == id).cloned();
        if let Some(t) = t {
            self.steps.set(run_for(&t));
            self.open.set(id);
            self.view.set("task");
            self.drawer.set(false);
            self.said.set(format!("Opened {}", t.title));
        }
    }
    fn go(mut self, v: &'static str) {
        self.view.set(v);
        self.drawer.set(false);
        self.said.set(view_title(v).to_string());
    }
    /// Copy the run's progress and status onto the open task.
    fn sync(mut self) {
        let id = *self.open.peek();
        let st = self.steps.peek().clone();
        let (d, n) = progress(&st);
        let s = run_status(&st);
        if let Some(t) = self.tasks.write().iter_mut().find(|t| t.id == id) {
            t.steps = (d as u8, n as u8);
            if t.status != Status::Failed || s == Status::Running {
                t.status = s;
            }
            t.ago = "now".to_string();
        }
    }
}

// ─────────────────────────────────────────────────────────── Sidebar ──

#[component]
fn Sidebar(
    ctx: Ctx,
    loading: bool,
    rail: bool,
    menu_open: bool,
    on_hide: EventHandler<()>,
) -> Element {
    let mut c = ctx;
    let mut acct = use_signal(move || menu_open);
    let mut sub = use_signal(|| "");
    let mut bell = use_signal(|| false);
    let sched_n = (c.scheds)().iter().filter(|x| !x.paused).count();
    let mut search = use_signal(|| false);
    let mut q = use_signal(String::new);
    let mut shut = use_signal(Vec::<usize>::new);
    let mut checked = use_signal(|| false);
    let env = (c.env)();
    let view = (c.view)();
    let open = (c.open)();
    let all = (c.tasks)();
    let list: Vec<Task> = all
        .iter()
        .filter(|t| t.env == env && matches(t, &q()))
        .cloned()
        .collect();
    let (run_n, need_n) = oxyn_core::kit::taskdeck::counts(&list);
    let needs: Vec<Task> = all
        .iter()
        .filter(|t| t.status == Status::NeedsYou)
        .cloned()
        .collect();
    let card_words = [
        (run_n > 0).then(|| format!("{run_n} running")),
        (need_n > 0).then(|| {
            format!(
                "{need_n} {}",
                if need_n == 1 { "needs you" } else { "need you" }
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    let use_v = usage();
    let use_pct = use_v.pct();
    let use_w = format!("width:{use_pct}%");
    let ac = acct();
    let sub_k = sub();
    let on_start = view == "start";
    let on_board = view == "board";
    let is_in = checked();
    let mut items: Vec<(&'static str, &'static str, &'static str, &'static str)> = Vec::new();
    for it in account_menu().iter().copied() {
        items.push(it);
        if it.0 == "upgrade" {
            items.push((
                "checkin",
                if is_in {
                    "Checked in today"
                } else {
                    "Daily check-in"
                },
                "cal",
                if is_in { "✓" } else { "+50" },
            ));
        }
    }
    let row = move |t: Task| {
        let id = t.id;
        let on = view == "task" && open == id;
        rsx! {
            li { key: "{id}",
                button { r#type: "button", class: "oxs9-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" }, title: "{t.title}",
                    onclick: move |_| c.open_task(id),
                    match t.status {
                        Status::Running => rsx! { span { class: "oxs9-spin oxs9-spin-s", "aria-label": "Running" } },
                        Status::NeedsYou => rsx! { span { class: "oxs9-need", "aria-label": "Needs you" } },
                        Status::Failed => rsx! { span { class: "oxs9-fail", "aria-label": "Failed" } },
                        _ => rsx! { span { class: "oxs9-rdot", "aria-hidden": "true" } },
                    }
                    span { class: "oxs9-ell", "{t.title}" }
                    span { class: "oxs9-ago", "{t.ago}" }
                }
            }
        }
    };
    rsx! {
        nav { class: "oxs9-side", "data-rail": "{rail}", "aria-label": "{APP}",
            div { class: "oxs9-stop",
                span { class: "oxs9-brand", {app_mark(18)} span { class: "oxs9-bn", "{APP}" } }
                span { class: "oxs9-grow" }
                button { r#type: "button", class: "oxscn-ib oxs9-sb", "aria-label": "Search tasks", "aria-pressed": "{search()}", onclick: move |_| { search.toggle(); q.set(String::new()); }, {nico("search")} }
                button { r#type: "button", class: "oxscn-ib oxs9-sb", "aria-label": "Hide sidebar", onclick: move |_| on_hide.call(()), {nico("side")} }
            }
            if search() && !rail {
                input { class: "oxs9-find", r#type: "search", placeholder: "Search tasks", "aria-label": "Search tasks", value: "{q}",
                    onmounted: super::setscr::focus_on,
                    oninput: move |e| q.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { search.set(false); q.set(String::new()); } }
            }
            div { class: "oxs9-prim",
                button { r#type: "button", class: "oxs9-nav", "data-on": "{on_start}", "aria-current": if on_start { "page" } else { "false" }, title: "New task",
                    onclick: move |_| c.go("start"), {nico("pen")} span { class: "oxs9-nl", "New task" } span { class: "oxs9-kbd", "⌘N" } }
                button { r#type: "button", class: "oxs9-nav", "data-on": "{on_board}", "aria-current": if on_board { "page" } else { "false" }, title: "Tasks",
                    onclick: move |_| c.go("board"), {nico("kanban")} span { class: "oxs9-nl", "Tasks" }
                    if run_n + need_n > 0 { span { class: "oxs9-cnt", "{run_n + need_n}" } } }
                for (vk, vl, vi) in [("plugins", "Plugins & skills", "plug"), ("scheduled", "Scheduled", "clock"), ("sites", "Sites", "web")] {
                    button { key: "{vk}", r#type: "button", class: "oxs9-nav", "data-on": "{view == vk}", "aria-current": if view == vk { "page" } else { "false" }, title: "{vl}",
                        onclick: move |_| c.go(vk), {nico(vi)} span { class: "oxs9-nl", "{vl}" }
                        if vk == "scheduled" && sched_n > 0 { span { class: "oxs9-cnt", "{sched_n}" } } }
                }
            }
            div { class: "oxs9-env", role: "radiogroup", "aria-label": "Where tasks run",
                for m in EnvMode::ALL {
                    button { key: "{m.key()}", r#type: "button", role: "radio", "aria-checked": "{env == m}", title: "{m.hint()}",
                        onclick: move |_| { c.env.set(m); c.said.set(format!("{}: {}", m.label(), m.hint())); },
                        {nico(if m == EnvMode::Local { "laptop" } else { "cloud" })} span { class: "oxs9-nl", "{m.label()}" } }
                }
            }
            div { class: "oxs9-list",
                if loading {
                    div { class: "oxs9-skl", "aria-busy": "true", {skel(6)} }
                } else if all.is_empty() {
                    p { class: "oxs9-faint", "No projects or tasks yet" }
                } else {
                    p { class: "oxs9-gh", "Projects" }
                    ul { class: "oxs9-ul",
                        for (pi, (pk, pn, _)) in PROJECTS.iter().copied().enumerate() {
                            {
                                let inner: Vec<Task> = list.iter().filter(|t| t.project == Some(pi)).cloned().collect();
                                let isopen = !shut().contains(&pi);
                                rsx! {
                                    li { key: "{pk}",
                                        button { r#type: "button", class: "oxs9-proj", "aria-expanded": "{isopen}", title: "{pn}",
                                            onclick: move |_| { let mut s = shut.write(); if let Some(i) = s.iter().position(|x| *x == pi) { s.remove(i); } else { s.push(pi); } },
                                            span { class: "oxs9-car", "data-open": "{isopen}", {nico("caret")} }
                                            if isopen { {ico("folderopen")} } else { {ico("folder")} }
                                            span { class: "oxs9-ell", "{pn}" }
                                            span { class: "oxs9-ago", "{inner.len()}" }
                                        }
                                        if isopen {
                                            ul { class: "oxs9-ul oxs9-in",
                                                if inner.is_empty() { li { p { class: "oxs9-faint oxs9-faint-s", "No {env.label().to_lowercase()} tasks" } } }
                                                for t in inner { {row(t)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if list.iter().any(|t| t.project.is_none()) {
                        p { class: "oxs9-gh", "Recents" }
                    }
                    ul { class: "oxs9-ul",
                        for t in list.iter().filter(|t| t.project.is_none()).cloned() { {row(t)} }
                    }
                    if list.is_empty() {
                        p { class: "oxs9-faint", role: "status", if q().trim().is_empty() { "No {env.label().to_lowercase()} tasks yet" } else { "Nothing matches “{q}”" } }
                    }
                }
            }
            div { class: "oxs9-foot",
                if !loading && run_n + need_n > 0 {
                    button { r#type: "button", class: "oxs9-card", onclick: move |_| c.go("board"),
                        span { class: "oxscn-live", "aria-hidden": "true" }
                        span { class: "oxs9-ell", "{card_words}" }
                        {nico("next")}
                    }
                }
                div { class: "oxscn-acct oxs9-acct",
                    button { r#type: "button", class: "oxscn-acct-b", "aria-haspopup": "menu", "aria-expanded": "{ac}", "aria-label": "Account, {PERSON}, {PERSON_PLAN}",
                        onclick: move |_| { acct.toggle(); sub.set(""); bell.set(false); },
                        span { class: "oxscn-av oxscn-av-id", "{PERSON_INI}" }
                        span { class: "oxscn-acct-txt",
                            span { class: "oxscn-acct-n", "{PERSON}" }
                            span { class: "oxscn-acct-p", "{PERSON_PLAN}" }
                        }
                        span { class: "oxscn-acct-ud", {nico("updown")} }
                    }
                    button { r#type: "button", class: "oxscn-ib oxscn-bell", title: "Inbox", "aria-label": "Inbox, {needs.len()} need you", "aria-expanded": "{bell()}", "aria-haspopup": "dialog",
                        onclick: move |_| { bell.toggle(); acct.set(false); },
                        {nico("bell")} if !needs.is_empty() { span { class: "oxscn-bell-n", "aria-hidden": "true", "{needs.len()}" } } }
                    if bell() {
                        div { class: "oxscn-menu oxscn-bell-pop oxs9-inbox", role: "dialog", "aria-label": "Inbox",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { bell.set(false); },
                            p { class: "oxscn-pop-h", "Inbox" }
                            if needs.is_empty() { p { class: "oxs9-faint", "Nothing needs you" } }
                            for t in needs.iter().cloned() {
                                button { key: "{t.id}", r#type: "button", class: "oxscn-mi oxscn-note", onclick: move |_| { bell.set(false); c.open_task(t.id); },
                                    span { class: "oxs9-need" } span { class: "oxscn-mi-l", "{t.title}" } span { class: "oxscn-mi-k", "{t.ago}" } }
                            }
                        }
                    }
                    if ac {
                        div { class: "oxscn-menu oxscn-acct-menu oxs9-amenu", role: "menu", "aria-label": "Account",
                            onkeydown: move |e: KeyboardEvent| match e.key() { Key::Escape => { if sub.peek().is_empty() { acct.set(false); } else { sub.set(""); } } Key::ArrowLeft => sub.set(""), _ => {} },
                            div { class: "oxscn-acct-hd",
                                span { class: "oxscn-av oxscn-av-id oxscn-av-l", "{PERSON_INI}" }
                                span { class: "oxscn-acct-txt",
                                    span { class: "oxscn-acct-n", "{PERSON}" }
                                    span { class: "oxscn-acct-p", title: "{PERSON_EMAIL}", "{PERSON_EMAIL}" }
                                }
                                span { class: "oxscn-plan", "{use_v.plan}" }
                            }
                            for (i, (k, l, g, h)) in items.iter().copied().enumerate() {
                                if k.is_empty() {
                                    div { key: "{i}", class: "oxscn-msep", role: "separator" }
                                } else {
                                    button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", "data-k": "{k}",
                                        "aria-haspopup": if h == "›" { "menu" } else { "false" },
                                        onmouseenter: move |_| sub.set(if h == "›" { k } else { "" }),
                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::ArrowRight && h == "›" { sub.set(k); },
                                        disabled: k == "checkin" && is_in,
                                        onclick: move |_| if h == "›" { sub.set(k); } else if k == "checkin" { checked.set(true); c.said.set("Checked in: 50 credits added".to_string()); } else { acct.set(false); sub.set(""); c.said.set(l.to_string()); },
                                        {nico(g)} span { class: "oxscn-mi-l", "{l}" }
                                        if h == "›" { span { class: "oxscn-mi-k oxscn-mi-sub", {nico("next")} } } else if !h.is_empty() { span { class: "oxscn-mi-k", "{h}" } }
                                    }
                                }
                            }
                            if sub_k == "usage" {
                                div { class: "oxscn-menu oxscn-fly", role: "menu", "aria-label": "Usage", "data-k": "usage",
                                    div { class: "oxscn-fly-hd",
                                        span { class: "oxscn-acct-n", "{use_v.plan} plan" }
                                        span { class: "oxscn-acct-p", "{use_v.renews}" }
                                    }
                                    div { class: "oxscn-meter", "data-near": "{use_v.near()}",
                                        div { class: "oxscn-meter-l", span { "Credits" } span { class: "oxscn-row-a", "{thousands(use_v.used)} of {thousands(use_v.total)} · {use_pct}%" } }
                                        div { class: "oxscn-prog", role: "progressbar", "aria-valuenow": "{use_pct}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "Credits used", span { style: "{use_w}" } }
                                    }
                                    for (i, (l, u, m)) in use_v.quotas.iter().copied().enumerate() {
                                        div { key: "{i}", class: "oxscn-quota", span { class: "oxscn-row-t", "{l}" } span { class: "oxscn-row-a", "{u} / {m}" } }
                                    }
                                }
                            }
                            if sub_k == "feedback" {
                                div { class: "oxscn-menu oxscn-fly oxscn-fly-b", role: "menu", "aria-label": "Feedback & help", "data-k": "feedback",
                                    for (i, (l, g)) in [("Send feedback", "chat"), ("Help centre", "help"), ("Contact support", "inbox")].iter().copied().enumerate() {
                                        button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| { acct.set(false); sub.set(""); c.said.set(l.to_string()); }, {nico(g)} span { class: "oxscn-mi-l", "{l}" } }
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

// ─────────────────────────────────────────────────────────── Composer ──

/// A chip with a small menu above or below it.
#[component]
fn Pick(
    label: String,
    lead: Element,
    items: Vec<(String, String, String)>,
    cur: usize,
    up: bool,
    on_pick: EventHandler<usize>,
) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "oxs9-pk",
            button { r#type: "button", class: "oxs9-chip", "aria-haspopup": "menu", "aria-expanded": "{open()}", "aria-label": "{label}",
                onclick: move |_| open.toggle(), {lead} span { class: "oxs9-ell", "{items.get(cur).map(|x| x.0.clone()).unwrap_or_default()}" } {nico("caret")} }
            if open() {
                div { class: "oxscn-menu oxs9-pkm", "data-up": "{up}", role: "menu", "aria-label": "{label}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
                    for (i, (n, l, m)) in items.iter().cloned().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-mi oxs9-pki", role: "menuitemradio", "aria-checked": "{i == cur}",
                            onclick: move |_| { open.set(false); on_pick.call(i); },
                            if !m.is_empty() { span { class: "oxs9-mk", BrandIcon { name: m.clone(), size: 15.0, coloured: true } } }
                            span { class: "oxs9-pkt", b { "{n}" } if !l.is_empty() { span { "{l}" } } }
                            if i == cur { span { class: "oxscn-mi-k", {nico("check")} } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StartView(ctx: Ctx, loading: bool) -> Element {
    let mut c = ctx;
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut mode = use_signal(|| 0usize);
    let mut model = use_signal(|| 0usize);
    let mut folder = use_signal(|| 0usize);
    let mut mic = use_signal(|| false);
    let mut add = use_signal(|| false);
    let mut attached = use_signal(Vec::<&'static str>::new);
    let env = (c.env)();
    let mut send = move || {
        let p = text.peek().clone();
        let proj = Some(*folder.peek());
        let r = add_task(&mut c.tasks.write(), &p, env, proj);
        match r {
            Ok(id) => {
                text.set(String::new());
                err.set(None);
                attached.set(Vec::new());
                c.asked.set((id, p.trim().to_string()));
                c.open_task(id);
                c.said.set(format!("Task started in {}", env.label()));
            }
            Err(e) => {
                err.set(Some(e));
                c.said.set(e.to_string());
            }
        }
    };
    let n = text().chars().count();
    let can = prompt_error(&text()).is_none();
    let modes: Vec<(String, String, String)> = MODES
        .iter()
        .map(|m| (m.1.to_string(), m.2.to_string(), String::new()))
        .collect();
    let models: Vec<(String, String, String)> = MODELS
        .iter()
        .map(|m| (m.0.to_string(), m.2.to_string(), m.1.to_string()))
        .collect();
    let folders: Vec<(String, String, String)> = PROJECTS
        .iter()
        .map(|p| (p.2.to_string(), p.1.to_string(), String::new()))
        .collect();
    let mk = MODELS[model()].1;
    if loading {
        return rsx! {
            div { class: "oxs9-start", "aria-busy": "true",
                span { class: "oxs9-sk oxs9-sk-mark" }
                span { class: "oxs9-sk", style: "width:300px;height:14px" }
                div { class: "oxs9-comp oxs9-comp-sk", {skel(3)} }
            }
        };
    }
    rsx! {
        div { class: "oxs9-start",
            span { class: "oxs9-hero", {app_mark(34)} }
            h1 { class: "oxs9-h1", "{headline(env)}" }
            div { class: "oxs9-comp", "data-bad": "{err().is_some()}", "data-env": "{env.key()}",
                if !attached().is_empty() {
                    div { class: "oxs9-atts",
                        for a in attached() {
                            span { key: "{a}", class: "oxs9-att", {file_ico(a)} span { class: "oxs9-ell", "{a}" }
                                button { r#type: "button", class: "oxs9-x", "aria-label": "Remove {a}", onclick: move |_| attached.write().retain(|x| *x != a), {nico("close")} } }
                        }
                    }
                }
                textarea { class: "oxs9-ta", rows: "3", placeholder: "Describe a task · @ for files, plugins and skills", "aria-label": "Describe a task",
                    "aria-invalid": "{err().is_some()}", "aria-describedby": "oxs9-hint", value: "{text}",
                    oninput: move |e| { text.set(e.value()); err.set(None); },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send(); } }
                if mic() {
                    div { class: "oxs9-mic", role: "status", span { class: "oxs9-wave", span {} span {} span {} span {} span {} } span { "Listening… speak your task" }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { mic.set(false); text.set(SUGGEST[0].2.to_string()); c.said.set("Dictation added".to_string()); }, "Done" } }
                }
                div { class: "oxs9-cbar",
                    div { class: "oxs9-pk",
                        button { r#type: "button", class: "oxs9-cib", "aria-label": "Add", "aria-haspopup": "menu", "aria-expanded": "{add()}", onclick: move |_| add.toggle(), {nico("plus")} }
                        if add() {
                            div { class: "oxscn-menu oxs9-pkm", "data-up": "false", role: "menu", "aria-label": "Add",
                                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { add.set(false); },
                                for (k, l, g) in ADD_MENU.iter().copied() {
                                    button { key: "{k}", r#type: "button", class: "oxscn-mi", role: "menuitem",
                                        onclick: move |_| {
                                            add.set(false);
                                            let f = match k { "files" => "brief.md", "folder" => "notes", "plugins" => "Browser plugin", _ => "Research skill" };
                                            if !attached.peek().contains(&f) { attached.write().push(f); }
                                            c.said.set(format!("Added {f}"));
                                        },
                                        if k == "folder" { {ico("folder")} } else { {nico(g)} } span { class: "oxscn-mi-l", "{l}" } }
                                }
                            }
                        }
                    }
                    Pick { label: "Run mode".to_string(), lead: rsx! { {nico("shield")} }, items: modes, cur: mode(), up: false, on_pick: move |i: usize| { mode.set(i); c.said.set(format!("Run mode: {}", MODES[i].1)); } }
                    span { class: "oxs9-grow" }
                    Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs9-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: false, on_pick: move |i: usize| { model.set(i); c.said.set(format!("Model: {}", MODELS[i].0)); } }
                    button { r#type: "button", class: "oxs9-cib", "aria-label": "Dictate", "aria-pressed": "{mic()}", onclick: move |_| mic.toggle(), {nico("mic")} }
                    button { r#type: "button", class: "oxs9-send", "aria-label": "Start task", disabled: !can, onclick: move |_| send(), {nico("send")} }
                }
            }
            p { id: "oxs9-hint", class: "oxs9-hint", "data-bad": "{err().is_some()}", role: if err().is_some() { "alert" } else { "none" },
                if let Some(e) = err() { "{e}" } else if n > 0 { "Enter to start · Shift + Enter for a new line" } else { "" } }
            div { class: "oxs9-under",
                Pick { label: "Folder".to_string(), lead: rsx! { {ico("folder")} }, items: folders, cur: folder(), up: false, on_pick: move |i: usize| folder.set(i) }
                span { class: "oxs9-envc", {nico(if env == EnvMode::Local { "laptop" } else { "cloud" })} span { "{env.label()}" } }
            }
            div { class: "oxs9-sugg", role: "list", "aria-label": "Ideas",
                for (g, l, p) in SUGGEST.iter().copied() {
                    button { key: "{l}", r#type: "button", class: "oxs9-sg", role: "listitem", onclick: move |_| { text.set(p.to_string()); err.set(None); c.said.set(format!("{l}: prompt filled in")); },
                        {nico(g)} span { "{l}" } }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── Task view ──

#[component]
fn SidePanel(ctx: Ctx) -> Element {
    let mut tab = use_signal(|| 0usize);
    let mut art = use_signal(|| None::<&'static str>);
    let steps = (ctx.steps)();
    let arts = artifacts(&steps);
    let lines = terminal_lines(&steps);
    rsx! {
        div { class: "oxs9-tabs", role: "tablist", "aria-label": "Side panel",
            for (i, (k, l)) in PANEL_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs9-tab", "aria-selected": "{tab() == i}", id: "oxs9-tab-{k}",
                    tabindex: if tab() == i { "0" } else { "-1" },
                    onclick: move |_| { tab.set(i); art.set(None); },
                    onkeydown: move |e: KeyboardEvent| match e.key() { Key::ArrowRight => tab.set((i + 1) % 4), Key::ArrowLeft => tab.set((i + 3) % 4), _ => {} },
                    "{l}" if k == "artifacts" && !arts.is_empty() { span { class: "oxs9-cnt", "{arts.len()}" } } }
            }
        }
        div { class: "oxs9-pbody", role: "tabpanel", "aria-labelledby": "oxs9-tab-{PANEL_TABS[tab()].0}",
            match (tab(), art()) {
                (0, Some(a)) => rsx! {
                    div { class: "oxs9-ahd",
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Back to artifacts", onclick: move |_| art.set(None), {nico("back")} }
                        {file_ico(a)} b { class: "oxs9-ell", "{a}" }
                    }
                    if a.ends_with(".csv") {
                        table { class: "oxs9-tbl",
                            thead { tr { th { "Desk" } th { "Price" } th { "Presets" } th { "Warranty" } } }
                            tbody { for r in SHEET { tr { key: "{r.0}", td { "{r.0}" } td { "{r.1}" } td { "{r.2}" } td { "{r.3}" } } } }
                        }
                    } else if a.ends_with(".png") {
                        div { class: "oxs9-bars", role: "img", "aria-label": "Prices of six desks, from $399 to $599",
                            for r in SHEET {
                                {
                                    let v: u32 = r.1.trim_start_matches('$').parse().unwrap_or(0);
                                    let w = format!("width:{}%", v * 100 / 600);
                                    rsx! { div { key: "{r.0}", class: "oxs9-bar", span { class: "oxs9-ell", "{r.0}" } span { class: "oxs9-bt", span { style: "{w}" } } b { "{r.1}" } } }
                                }
                            }
                        }
                    } else if a == "brief.md" {
                        div { class: "oxs9-doc", h4 { "Brief" } p { "Budget up to $600. Needs a memory preset for two heights and at least a 10-year warranty on the frame." } }
                    } else {
                        div { class: "oxs9-doc", h4 { "Buying note" } p { "Top pick: Arbor Lift 2 ($549), four presets and a 10-year warranty." } p { "Runners-up: Tessel One ($529) and Fenwick Duo ($589)." } }
                    }
                },
                (0, None) => rsx! {
                    if arts.is_empty() {
                        div { class: "oxs9-pempty", {nico("box")} p { "Files the task writes show up here." } }
                    }
                    ul { class: "oxs9-arts",
                        for (a, sz) in arts.iter().copied() {
                            li { key: "{a}",
                                button { r#type: "button", class: "oxs9-art", onclick: move |_| art.set(Some(a)),
                                    span { class: "oxs9-ai", {file_ico(a)} }
                                    span { class: "oxs9-at", b { class: "oxs9-ell", "{a}" } span { "{sz}" } }
                                    {nico("next")}
                                }
                            }
                        }
                    }
                },
                (1, _) => rsx! {
                    ul { class: "oxs9-tree", role: "tree", "aria-label": "Workspace files",
                        for (d, n, f) in TREE.iter().copied() {
                            {
                                let pad = format!("padding-left:{}px", 8 + d as u32 * 16);
                                rsx! { li { key: "{n}", role: "treeitem", class: "oxs9-tr", style: "{pad}", if f { {ico("folder")} } else { {file_ico(n)} } span { class: "oxs9-ell", "{n}" } } }
                            }
                        }
                    }
                },
                (2, _) => rsx! {
                    pre { class: "oxs9-term", "aria-label": "Terminal output",
                        for (i, l) in lines.iter().enumerate() { span { key: "{i}", "data-k": "{l.chars().next().unwrap_or(' ')}", "{l}\n" } }
                    }
                },
                _ => rsx! {
                    div { class: "oxs9-url", {nico("lock")} span { class: "oxs9-ell", "{BROWSED[0].1}" } }
                    div { class: "oxs9-page",
                        span { class: "oxs9-pimg" }
                        b { "{BROWSED[0].0}" }
                        span { class: "oxs9-faint-s", "$549 · 4 presets · 10-year warranty" }
                    }
                    p { class: "oxs9-gh", "Visited" }
                    ul { class: "oxs9-arts",
                        for (t, u) in BROWSED.iter().copied() {
                            li { key: "{u}", class: "oxs9-vis", {nico("globe")} span { class: "oxs9-at", b { class: "oxs9-ell", "{t}" } span { class: "oxs9-ell", "{u}" } } }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn TaskView(ctx: Ctx, loading: bool, panel: Signal<bool>, pnar: Signal<bool>) -> Element {
    let mut c = ctx;
    let mut panel = panel;
    let mut pnar = pnar;
    let mut paused = use_signal(|| false);
    let mut shut = use_signal(Vec::<usize>::new);
    let mut follow = use_signal(String::new);
    let mut notes = use_signal(Vec::<String>::new);
    let open = (c.open)();
    let all = (c.tasks)();
    let Some(task) = all.iter().find(|t| t.id == open).cloned() else {
        return rsx! {
            div { class: "oxs9-blank",
                if loading {
                    div { class: "oxs9-tl", "aria-busy": "true", {skel(8)} }
                } else {
                    span { class: "oxs9-hero", {nico("tasks")} }
                    p { class: "oxs9-bt1", "No task open" }
                    p { class: "oxs9-faint", "Start one and its steps, tool calls and files show up here." }
                    button { r#type: "button", class: "oxscn-primary", onclick: move |_| c.go("start"), {nico("plus")} "New task" }
                }
            }
        };
    };
    let steps = (c.steps)();
    let (d, n) = progress(&steps);
    let status = if task.status == Status::Failed {
        Status::Failed
    } else {
        run_status(&steps)
    };
    let secs = elapsed(&steps);
    let running = status == Status::Running;
    let pv = panel();
    let asked = (c.asked)();
    let prompt = if task.id == 1 {
        PROMPT.to_string()
    } else if asked.0 == task.id {
        asked.1
    } else {
        task.title.clone()
    };
    let pct = (d * 100).checked_div(n).unwrap_or(0);
    let bar = format!("width:{pct}%");
    let proj = task.project.map(|p| PROJECTS[p].1).unwrap_or("No project");
    let mut send_follow = move || {
        let f = follow.peek().trim().to_string();
        if f.is_empty() {
            c.said.set("Type a follow-up first".to_string());
            return;
        }
        notes.write().push(f);
        follow.set(String::new());
        c.said.set("Follow-up queued for the next step".to_string());
    };
    rsx! {
        div { class: "oxs9-task",
            div { class: "oxs9-thd",
                div { class: "oxs9-tti",
                    h1 { class: "oxs9-h2 oxs9-ell", "{task.title}" }
                    div { class: "oxs9-meta",
                        {status_pill(status)}
                        span { class: "oxs9-envc", {nico(if task.env == EnvMode::Local { "laptop" } else { "cloud" })} "{task.env.label()}" }
                        span { class: "oxs9-envc", {ico("folder")} span { class: "oxs9-ell", "{proj}" } }
                        span { class: "oxs9-faint-s", "Step {d.min(n)} of {n} · {fmt_secs(secs)}" }
                    }
                }
                if running || paused() {
                    button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{paused()}",
                        onclick: move |_| { paused.toggle(); c.said.set(if *paused.peek() { "Paused".to_string() } else { "Resumed".to_string() }); },
                        {nico(if paused() { "next" } else { "pause" })} span { if paused() { "Resume" } else { "Pause" } } }
                }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Side panel", "aria-expanded": "{pv}", "aria-controls": "oxs9-pnl",
                    onclick: move |_| { panel.toggle(); pnar.toggle(); }, {nico("panel")} }
            }
            div { class: "oxs9-prog", role: "progressbar", "aria-label": "Task progress", "aria-valuenow": "{pct}", "aria-valuemin": "0", "aria-valuemax": "100", span { style: "{bar}" } }
            div { class: "oxs9-tbody",
                div { class: "oxs9-tl",
                    if loading {
                        div { "aria-busy": "true", {skel(8)} }
                    } else {
                        div { class: "oxs9-you", span { class: "oxscn-av oxscn-av-id", "{PERSON_INI}" } p { class: "oxs9-bub", "{prompt}" } }
                        ol { class: "oxs9-steps", "aria-label": "Steps",
                            for (i, s) in steps.iter().cloned().enumerate() {
                                {
                                    let isopen = !shut().contains(&i) && s.state != StepState::Pending;
                                    rsx! {
                                        li { key: "{i}", class: "oxs9-step", "data-s": "{s.state:?}",
                                            {step_mark(s.state)}
                                            div { class: "oxs9-sc",
                                                button { r#type: "button", class: "oxs9-sh", "aria-expanded": "{isopen}", disabled: s.state == StepState::Pending,
                                                    onclick: move |_| { let mut v = shut.write(); if let Some(x) = v.iter().position(|x| *x == i) { v.remove(x); } else { v.push(i); } },
                                                    span { class: "oxs9-ell oxs9-stt", "{s.title}" }
                                                    if s.state == StepState::Done { span { class: "oxs9-ago", "{fmt_secs(s.secs)}" } }
                                                }
                                                if isopen && !s.tool.is_empty() {
                                                    div { class: "oxs9-call",
                                                        span { class: "oxs9-tool", {nico(match s.tool.split('.').next().unwrap_or("") { "browser" => "globe", "terminal" => "term", "sheet" => "table", _ => "file" })} "{s.tool}" }
                                                        code { class: "oxs9-arg oxs9-ell", "{s.arg}" }
                                                    }
                                                }
                                                if isopen && s.state == StepState::Done { p { class: "oxs9-out", "{s.out}" } }
                                                if s.state == StepState::Skipped { p { class: "oxs9-out", "Skipped" } }
                                                if s.state == StepState::Waiting {
                                                    div { class: "oxs9-ask", role: "group", "aria-label": "Approval needed",
                                                        p { b { "Run this in your {task.env.label().to_lowercase()} terminal?" } }
                                                        code { class: "oxs9-arg", "{s.arg}" }
                                                        div { class: "oxs9-askb",
                                                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| { approve(&mut c.steps.write(), true); c.sync(); c.said.set("Allowed once; running".to_string()); }, "Allow once" }
                                                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { approve(&mut c.steps.write(), true); c.sync(); c.said.set("Allowed for this folder".to_string()); }, "Always in this folder" }
                                                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { approve(&mut c.steps.write(), false); c.sync(); c.said.set("Denied; step skipped".to_string()); }, "Deny" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        for (i, f) in notes().iter().enumerate() {
                            div { key: "{i}", class: "oxs9-you oxs9-you-f", span { class: "oxscn-av oxscn-av-id", "{PERSON_INI}" } p { class: "oxs9-bub", "{f}" } span { class: "oxs9-pill", "Queued" } }
                        }
                        if status == Status::Done {
                            p { class: "oxs9-donel", role: "status", {nico("check")} span { "Done in {fmt_secs(secs)}. {artifacts(&steps).len()} files in Artifacts." } }
                        }
                    }
                }
                if !loading {
                    div { class: "oxs9-fu",
                        input { class: "oxs9-fin", r#type: "text", placeholder: if running { "Steer or add a follow-up…" } else { "Ask a follow-up…" }, "aria-label": "Follow-up", value: "{follow}",
                            oninput: move |e| follow.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { send_follow(); } }
                        button { r#type: "button", class: "oxs9-send", "aria-label": "Send follow-up", disabled: follow().trim().is_empty(), onclick: move |_| send_follow(), {nico("send")} }
                    }
                }
            }
            if running && !paused() && !loading {
                span { key: "{d}-{n}", class: "oxs9-tick", onanimationend: move |_| { if advance(&mut c.steps.write()) { c.sync(); let s = c.steps.peek().clone(); if run_status(&s) == Status::Done { c.said.set("Task done".to_string()); } } } }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── Board ──

#[component]
fn BoardView(ctx: Ctx, loading: bool) -> Element {
    let mut c = ctx;
    let mut list_mode = use_signal(|| false);
    let mut menu = use_signal(|| 0u32);
    let env = (c.env)();
    let all: Vec<Task> = (c.tasks)().into_iter().filter(|t| t.env == env).collect();
    let card = move |t: Task| {
        let id = t.id;
        let pct = (t.steps.0 as usize * 100)
            .checked_div(t.steps.1 as usize)
            .unwrap_or(0);
        let w = format!("width:{pct}%");
        let proj = t.project.map(|p| PROJECTS[p].1).unwrap_or("No project");
        let acts = card_acts(t.status);
        let mo = menu() == id;
        rsx! {
            li { key: "{id}", class: "oxs9-bc", "data-s": "{t.status.tone()}",
                div { class: "oxs9-bch",
                    button { r#type: "button", class: "oxs9-bct", onclick: move |_| c.open_task(id), "{t.title}" }
                    div { class: "oxs9-pk",
                        button { r#type: "button", class: "oxscn-ib oxs9-more", "aria-label": "Actions for {t.title}", "aria-haspopup": "menu", "aria-expanded": "{mo}",
                            onclick: move |_| menu.set(if mo { 0 } else { id }), {nico("more")} }
                        if mo {
                            div { class: "oxscn-menu oxs9-pkm oxs9-cm", role: "menu", "aria-label": "Task actions",
                                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { menu.set(0); },
                                button { r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| { menu.set(0); c.open_task(id); }, {nico("next")} span { class: "oxscn-mi-l", "Open" } }
                                for (k, l, to) in acts.iter().copied() {
                                    button { key: "{k}", r#type: "button", class: "oxscn-mi", role: "menuitem",
                                        onclick: move |_| { menu.set(0); move_task(&mut c.tasks.write(), id, to); c.said.set(format!("Moved to {}", to.label())); },
                                        {nico(match k { "run" => "next", "queue" => "clock", _ => "check" })} span { class: "oxscn-mi-l", "{l}" } }
                                }
                                div { class: "oxscn-msep", role: "separator" }
                                button { r#type: "button", class: "oxscn-mi oxs9-del", role: "menuitem", onclick: move |_| { menu.set(0); delete_task(&mut c.tasks.write(), id); c.said.set("Task deleted".to_string()); }, {nico("trash")} span { class: "oxscn-mi-l", "Delete" } }
                            }
                        }
                    }
                }
                div { class: "oxs9-bcm", {ico("folder")} span { class: "oxs9-ell", "{proj}" } span { class: "oxs9-grow" } span { class: "oxs9-ago", "{t.ago}" } }
                if matches!(t.status, Status::Running | Status::NeedsYou | Status::Failed) {
                    div { class: "oxs9-bcp", span { class: "oxs9-prog oxs9-prog-s", span { style: "{w}" } } span { class: "oxs9-ago", "{t.steps.0}/{t.steps.1}" } }
                }
                if t.status == Status::Failed { {status_pill(Status::Failed)} }
            }
        }
    };
    rsx! {
        div { class: "oxs9-board",
            div { class: "oxs9-thd",
                div { class: "oxs9-tti", h1 { class: "oxs9-h2", "Tasks" } span { class: "oxs9-faint-s", "{env.label()} · {all.len()} tasks" } }
                div { class: "oxs9-seg", role: "radiogroup", "aria-label": "View",
                    button { r#type: "button", role: "radio", "aria-checked": "{!list_mode()}", onclick: move |_| list_mode.set(false), {nico("kanban")} "Board" }
                    button { r#type: "button", role: "radio", "aria-checked": "{list_mode()}", onclick: move |_| list_mode.set(true), {nico("list")} "List" }
                }
                button { r#type: "button", class: "oxscn-primary", onclick: move |_| c.go("start"), {nico("plus")} span { "New task" } }
            }
            if loading {
                div { class: "oxs9-cols", "aria-busy": "true",
                    for i in 0..4 { div { key: "{i}", class: "oxs9-col", span { class: "oxs9-sk", style: "width:70px" } div { class: "oxs9-bc", {skel(3)} } div { class: "oxs9-bc", {skel(2)} } } }
                }
            } else if all.is_empty() {
                div { class: "oxs9-blank",
                    span { class: "oxs9-hero", {nico("kanban")} }
                    p { class: "oxs9-bt1", "No {env.label().to_lowercase()} tasks yet" }
                    p { class: "oxs9-faint", "Queued, running and finished tasks line up here." }
                    button { r#type: "button", class: "oxscn-primary", onclick: move |_| c.go("start"), {nico("plus")} "New task" }
                }
            } else if list_mode() {
                div { class: "oxs9-lst",
                    div { class: "oxs9-lr oxs9-lh", span { "Task" } span { "Project" } span { "Status" } span { "Steps" } span { "Updated" } }
                    for t in all.iter().cloned() {
                        {
                            let id = t.id;
                            let pn = t.project.map(|p| PROJECTS[p].1).unwrap_or("–");
                            rsx! {
                                button { key: "{id}", r#type: "button", class: "oxs9-lr", onclick: move |_| c.open_task(id),
                                    span { class: "oxs9-ell", b { "{t.title}" } }
                                    span { class: "oxs9-ell oxs9-faint-s", "{pn}" }
                                    span { {status_pill(t.status)} }
                                    span { class: "oxs9-ago", "{t.steps.0}/{t.steps.1}" }
                                    span { class: "oxs9-ago", "{t.ago}" }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs9-cols",
                    for (ci, s) in COLUMNS.iter().copied().enumerate() {
                        {
                            let col: Vec<Task> = all.iter().filter(|t| t.status.column() == ci).cloned().collect();
                            rsx! {
                                section { key: "{ci}", class: "oxs9-col", "aria-label": "{s.label()}",
                                    h2 { class: "oxs9-ch", {status_pill(s)} span { class: "oxs9-ago", "{col.len()}" } }
                                    ul { class: "oxs9-ul oxs9-cl",
                                        if col.is_empty() { li { p { class: "oxs9-faint oxs9-faint-s", "Nothing here" } } }
                                        for t in col { {card(t)} }
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

// ─────────────────────────────────────────────────── Plugins & skills ──

#[component]
fn PluginsView(ctx: Ctx, loading: bool, skills_tab: bool) -> Element {
    let mut c = ctx;
    let mut tab = use_signal(move || if skills_tab { "skills" } else { "plugins" });
    let mut cat = use_signal(|| "All");
    let mut q = use_signal(String::new);
    let mut open = use_signal(|| usize::MAX);
    let mut adding = use_signal(|| false);
    let mut nm = use_signal(String::new);
    let mut wh = use_signal(String::new);
    let mut tried = use_signal(|| false);
    let st = (c.plugs)();
    let sk = (c.skills)();
    let shown = plugins_in(cat(), &q());
    let installed = st.iter().filter(|x| x.0).count();
    let err = skill_error(&nm(), &wh(), &sk);
    let t = tab();
    let on_pl = t == "plugins";
    rsx! {
        div { class: "oxs9-board oxs9-pg",
            div { class: "oxs9-thd",
                div { class: "oxs9-tti", h1 { class: "oxs9-h2", "Plugins & skills" }
                    span { class: "oxs9-faint-s", "{installed} plugins installed · {sk.iter().filter(|s| s.on).count()} skills on" } }
                div { class: "oxs9-seg", role: "radiogroup", "aria-label": "Show",
                    button { r#type: "button", role: "radio", "aria-checked": "{on_pl}", onclick: move |_| tab.set("plugins"), {nico("plug")} "Plugins" }
                    button { r#type: "button", role: "radio", "aria-checked": "{!on_pl}", onclick: move |_| tab.set("skills"), {nico("spark")} "Skills" }
                }
            }
            if loading {
                div { class: "oxs9-pgrid", "aria-busy": "true", for i in 0..6 { div { key: "{i}", class: "oxs9-bc", {skel(3)} } } }
            } else if t == "plugins" {
                div { class: "oxs9-pbar",
                    input { class: "oxs9-find oxs9-pfind", r#type: "search", placeholder: "Search plugins", "aria-label": "Search plugins", value: "{q}", oninput: move |e| q.set(e.value()) }
                    div { class: "oxs9-chips", role: "radiogroup", "aria-label": "Category",
                        for k in PLUGIN_CATS {
                            button { key: "{k}", r#type: "button", class: "oxs9-fchip", role: "radio", "aria-checked": "{cat() == k}", onclick: move |_| cat.set(k), "{k}" }
                        }
                    }
                }
                if shown.is_empty() {
                    div { class: "oxs9-blank", span { class: "oxs9-hero", {nico("plug")} } p { class: "oxs9-bt1", "No plugins match" } p { class: "oxs9-faint", "Try another word or category." } }
                } else {
                    ul { class: "oxs9-pgrid", "aria-label": "Plugins",
                        for i in shown {
                            {
                                let (k, name, mk, gl, cat_l, line, perms) = CATALOG[i];
                                let (inst, on) = st.get(i).copied().unwrap_or((false, false));
                                let o = open() == i;
                                rsx! {
                                    li { key: "{k}", class: "oxs9-bc oxs9-pc", "data-on": "{on}",
                                        div { class: "oxs9-pch",
                                            span { class: "oxs9-plogo", if mk.is_empty() { {nico(gl)} } else { BrandIcon { name: mk.to_string(), size: 18.0, coloured: true } } }
                                            span { class: "oxs9-plt", b { "{name}" } span { "{cat_l}" } }
                                            if inst {
                                                input { r#type: "checkbox", role: "switch", class: "oxs9-sw", checked: on, "aria-label": "{name} on",
                                                    onchange: move |e| { c.plugs.write()[i].1 = e.checked(); c.said.set(format!("{name} {}", if e.checked() { "on" } else { "off" })); } }
                                            }
                                        }
                                        p { class: "oxs9-pl", "{line}" }
                                        div { class: "oxs9-pacts",
                                            if inst {
                                                button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{o}", onclick: move |_| open.set(if o { usize::MAX } else { i }), "Details" }
                                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| { c.plugs.write()[i] = (false, false); open.set(usize::MAX); c.said.set(format!("{name} removed")); }, "Remove" }
                                            } else {
                                                button { r#type: "button", class: "oxscn-primary", onclick: move |_| { c.plugs.write()[i] = (true, true); c.said.set(format!("{name} installed and on")); }, {nico("plus")} span { "Install" } }
                                            }
                                        }
                                        if o {
                                            div { class: "oxs9-perm", role: "group", "aria-label": "What {name} may do",
                                                p { class: "oxs9-gh", "Can" }
                                                ul { for pm in perms.iter() { li { key: "{pm}", {ico("check")} span { "{pm}" } } } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxs9-pbar",
                    p { class: "oxs9-faint-s oxs9-grow", "Skills are saved instructions the agent loads when a task matches." }
                    button { r#type: "button", class: "oxscn-primary", "aria-expanded": "{adding()}", onclick: move |_| { adding.toggle(); tried.set(false); nm.set(String::new()); wh.set(String::new()); }, {nico("plus")} span { "New skill" } }
                }
                if adding() {
                    div { class: "oxs9-form", role: "group", "aria-label": "New skill",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false); },
                        label { class: "oxs9-fl", "Name" input { class: "oxs9-inp", value: "{nm}", placeholder: "Weekly digest", onmounted: super::setscr::focus_on, "aria-invalid": "{tried() && err.is_some()}", oninput: move |e| nm.set(e.value()) } }
                        label { class: "oxs9-fl", "When it applies" input { class: "oxs9-inp", value: "{wh}", placeholder: "Friday summaries for the team", oninput: move |e| wh.set(e.value()) } }
                        div { class: "oxs9-facts",
                            if let (true, Some(m)) = (tried(), err) { span { class: "oxs9-err", role: "alert", "{m}" } }
                            span { class: "oxs9-grow" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| {
                                tried.set(true);
                                if skill_error(&nm.peek(), &wh.peek(), &c.skills.peek()).is_none() {
                                    let n = nm.peek().trim().to_string();
                                    c.skills.write().insert(0, Skill { name: n.clone(), when: wh.peek().trim().to_string(), on: true, mine: true });
                                    adding.set(false);
                                    c.said.set(format!("Skill {n} saved"));
                                }
                            }, "Save skill" }
                        }
                    }
                }
                if sk.is_empty() && !adding() {
                    div { class: "oxs9-blank", span { class: "oxs9-hero", {nico("spark")} } p { class: "oxs9-bt1", "No skills yet" } p { class: "oxs9-faint", "Save how you like things done once; every matching task uses it." } }
                } else {
                    ul { class: "oxs9-lst oxs9-sk-l", "aria-label": "Skills",
                        for (i, k) in sk.iter().cloned().enumerate() {
                            li { key: "{k.name}", class: "oxs9-skr",
                                span { class: "oxs9-plogo", {nico("spark")} }
                                span { class: "oxs9-plt", b { "{k.name}" } span { "{k.when}" } }
                                span { class: "oxs9-pill", if k.mine { "Yours" } else { "Bundled" } }
                                if k.mine {
                                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Delete {k.name}", onclick: move |_| { let n = c.skills.write().remove(i).name; c.said.set(format!("Skill {n} deleted")); }, {nico("trash")} }
                                }
                                input { r#type: "checkbox", role: "switch", class: "oxs9-sw", checked: k.on, "aria-label": "{k.name} on",
                                    onchange: move |e| { c.skills.write()[i].on = e.checked(); } }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── Scheduled ──

#[component]
fn ScheduledView(ctx: Ctx, loading: bool, editing: bool) -> Element {
    let mut c = ctx;
    // 0 = closed, u32::MAX = new, else the schedule being edited.
    let mut edit = use_signal(move || if editing { 2u32 } else { 0 });
    let mut nm = use_signal(String::new);
    let mut pr = use_signal(String::new);
    let mut cad = use_signal(|| 2usize);
    let mut at = use_signal(|| "09:00".to_string());
    let mut tried = use_signal(|| false);
    use_hook(move || {
        if editing && let Some(x) = c.scheds.peek().iter().find(|x| x.id == 2) {
            nm.set(x.name.clone());
            pr.set(x.prompt.clone());
            cad.set(x.cadence);
            at.set(x.at.clone());
        }
    });
    let list = (c.scheds)();
    let e = edit();
    let err = schedule_error(&nm(), &pr(), cad(), &at());
    let bad_time = tried() && err == Some("Use a 24-hour time like 08:30");
    let mut open_form = move |x: Option<Schedule>| {
        tried.set(false);
        match x {
            Some(x) => {
                nm.set(x.name);
                pr.set(x.prompt);
                cad.set(x.cadence);
                at.set(x.at);
                edit.set(x.id);
            }
            None => {
                nm.set(String::new());
                pr.set(String::new());
                cad.set(2);
                at.set("09:00".into());
                edit.set(u32::MAX);
            }
        }
    };
    let form = rsx! {
        div { class: "oxs9-form", role: "group", "aria-label": if e == u32::MAX { "New scheduled task" } else { "Edit scheduled task" },
            onkeydown: move |k: KeyboardEvent| if k.key() == Key::Escape { edit.set(0); },
            label { class: "oxs9-fl", "Name" input { class: "oxs9-inp", value: "{nm}", placeholder: "Morning briefing", onmounted: super::setscr::focus_on, "aria-invalid": "{tried() && nm().trim().is_empty()}", oninput: move |v| nm.set(v.value()) } }
            label { class: "oxs9-fl oxs9-span", "What it does" textarea { class: "oxs9-inp oxs9-ta2", rows: "2", value: "{pr}", placeholder: "Summarise new mail and today's calendar in five lines.", oninput: move |v| pr.set(v.value()) } }
            div { class: "oxs9-fl", span { "How often" }
                div { class: "oxs9-chips", role: "radiogroup", "aria-label": "How often",
                    for (i, (k, l)) in CADENCES.iter().copied().enumerate() {
                        button { key: "{k}", r#type: "button", class: "oxs9-fchip", role: "radio", "aria-checked": "{cad() == i}", onclick: move |_| cad.set(i), "{l}" }
                    }
                }
            }
            if cad() > 0 {
                label { class: "oxs9-fl", "At (24 h)" input { class: "oxs9-inp oxs9-time", value: "{at}", inputmode: "numeric", placeholder: "08:30", "aria-invalid": "{bad_time}", oninput: move |v| at.set(v.value()) } }
            }
            div { class: "oxs9-facts oxs9-span",
                if let (true, Some(m)) = (tried(), err) { span { class: "oxs9-err", role: "alert", "{m}" } }
                span { class: "oxs9-grow" }
                button { r#type: "button", class: "oxscn-btn", onclick: move |_| edit.set(0), "Cancel" }
                button { r#type: "button", class: "oxscn-primary", onclick: move |_| {
                    tried.set(true);
                    if schedule_error(&nm.peek(), &pr.peek(), *cad.peek(), &at.peek()).is_some() { return; }
                    let id = *edit.peek();
                    let env = *c.env.peek();
                    let mut l = c.scheds.write();
                    if id == u32::MAX {
                        let nid = l.iter().map(|x| x.id).max().unwrap_or(0) + 1;
                        l.insert(0, Schedule { id: nid, name: nm.peek().trim().into(), prompt: pr.peek().trim().into(), cadence: *cad.peek(), at: at.peek().trim().into(), paused: false, last: (LastRun::Never, String::new()), env });
                    } else if let Some(x) = l.iter_mut().find(|x| x.id == id) {
                        x.name = nm.peek().trim().into(); x.prompt = pr.peek().trim().into(); x.cadence = *cad.peek(); x.at = at.peek().trim().into();
                    }
                    drop(l);
                    edit.set(0);
                    c.said.set(format!("Saved: {}", cadence_words(*cad.peek(), &at.peek())));
                }, "Save" }
            }
        }
    };
    rsx! {
        div { class: "oxs9-board oxs9-pg",
            div { class: "oxs9-thd",
                div { class: "oxs9-tti", h1 { class: "oxs9-h2", "Scheduled" } span { class: "oxs9-faint-s", "{list.iter().filter(|x| !x.paused).count()} active · runs in the cloud while you're away" } }
                button { r#type: "button", class: "oxscn-primary", onclick: move |_| open_form(None), {nico("plus")} span { "New schedule" } }
            }
            if e == u32::MAX { div { class: "oxs9-pad", {form.clone()} } }
            if loading {
                div { class: "oxs9-lst", "aria-busy": "true", for i in 0..4 { div { key: "{i}", class: "oxs9-schr", {skel(2)} } } }
            } else if list.is_empty() && e == 0 {
                div { class: "oxs9-blank", span { class: "oxs9-hero", {nico("clock")} } p { class: "oxs9-bt1", "Nothing scheduled" } p { class: "oxs9-faint", "Run a task every morning, weekday or Monday, here or in the cloud." }
                    button { r#type: "button", class: "oxscn-primary", onclick: move |_| open_form(None), {nico("plus")} "New schedule" } }
            } else {
                ul { class: "oxs9-lst", "aria-label": "Scheduled tasks",
                    for x in list.iter().cloned() {
                        {
                            let id = x.id;
                            let xc = x.clone();
                            let nr = next_run(&x);
                            let (lr, ago) = x.last.clone();
                            let xn = x.name.clone();
                            rsx! {
                                li { key: "{id}", class: "oxs9-schr", "data-paused": "{x.paused}",
                                    div { class: "oxs9-schm",
                                        span { class: "oxs9-plogo", {nico(if x.env == EnvMode::Cloud { "cloud" } else { "laptop" })} }
                                        span { class: "oxs9-plt", b { "{x.name}" } span { "{cadence_words(x.cadence, &x.at)} · {nr}" } }
                                        span { class: "oxs9-pill", "data-tone": "{lr.tone()}", span { class: "oxs9-dot" } "{lr.label()}" if !ago.is_empty() { " · {ago}" } }
                                        div { class: "oxs9-schact",
                                            button { r#type: "button", class: "oxscn-btn", disabled: lr == LastRun::Running, onclick: move |_| {
                                                if let Some(s) = c.scheds.write().iter_mut().find(|s| s.id == id) { s.last = (LastRun::Running, "now".into()); }
                                                c.said.set(format!("Running {xn}")); }, {nico("next")} span { "Run now" } }
                                            button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{x.paused}", onclick: move |_| {
                                                let mut p = false;
                                                if let Some(s) = c.scheds.write().iter_mut().find(|s| s.id == id) { s.paused = !s.paused; p = s.paused; }
                                                c.said.set(if p { "Paused".to_string() } else { "Resumed".to_string() }); },
                                                {nico(if x.paused { "next" } else { "pause" })} span { if x.paused { "Resume" } else { "Pause" } } }
                                            button { r#type: "button", class: "oxscn-ib", "aria-label": "Edit {x.name}", "aria-expanded": "{e == id}", onclick: move |_| open_form(Some(xc.clone())), {nico("pen")} }
                                            button { r#type: "button", class: "oxscn-ib", "aria-label": "Delete {x.name}", onclick: move |_| { c.scheds.write().retain(|s| s.id != id); c.said.set("Schedule deleted".to_string()); }, {nico("trash")} }
                                        }
                                    }
                                    if lr == LastRun::Running {
                                        span { class: "oxs9-tick oxs9-tick-l", onanimationend: move |_| { if let Some(s) = c.scheds.write().iter_mut().find(|s| s.id == id) { s.last = (LastRun::Ok, "just now".into()); } c.said.set("Run finished".to_string()); } }
                                    }
                                    if e == id { {form.clone()} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Sites ──

#[component]
fn SitesView(ctx: Ctx, loading: bool, preview: bool) -> Element {
    let mut c = ctx;
    let mut pv = use_signal(move || if preview { 1u32 } else { 0 });
    let list = (c.sites)();
    let building = list.iter().any(|s| s.status == SiteStatus::Building);
    let open = list.iter().find(|s| s.id == pv()).cloned();
    rsx! {
        div { class: "oxs9-board oxs9-pg",
            div { class: "oxs9-thd",
                div { class: "oxs9-tti", h1 { class: "oxs9-h2", "Sites" } span { class: "oxs9-faint-s", "{list.len()} sites · built and deployed by tasks" } }
                button { r#type: "button", class: "oxscn-primary", onclick: move |_| c.go("start"), {nico("plus")} span { "Build a site" } }
            }
            if loading {
                div { class: "oxs9-pgrid", "aria-busy": "true", for i in 0..3 { div { key: "{i}", class: "oxs9-bc", div { class: "oxs9-shot oxs9-sk" } {skel(2)} } } }
            } else if list.is_empty() {
                div { class: "oxs9-blank", span { class: "oxs9-hero", {nico("web")} } p { class: "oxs9-bt1", "No sites yet" } p { class: "oxs9-faint", "Ask a task to build a page; it deploys here with its own address." }
                    button { r#type: "button", class: "oxscn-primary", onclick: move |_| c.go("start"), {nico("plus")} "Build a site" } }
            } else {
                div { class: "oxs9-sitesw", "data-pv": "{open.is_some()}",
                    ul { class: "oxs9-pgrid", "aria-label": "Sites",
                        for x in list.iter().cloned() {
                            {
                                let id = x.id;
                                let on = pv() == id;
                                let xn = x.name.clone();
                                rsx! {
                                    li { key: "{id}", class: "oxs9-bc oxs9-site", "data-on": "{on}",
                                        button { r#type: "button", class: "oxs9-shot", "aria-label": "Preview {x.name}", onclick: move |_| pv.set(if on { 0 } else { id }),
                                            span { class: "oxs9-mini", b { "{x.page.0}" } for k in 0..x.page.2 { i { key: "{k}" } } }
                                        }
                                        div { class: "oxs9-pch",
                                            span { class: "oxs9-plt", b { "{x.name}" } span { class: "oxs9-ell", "{x.url}" } }
                                            span { class: "oxs9-pill", "data-tone": "{x.status.tone()}", span { class: if x.status == SiteStatus::Building { "oxs9-dot oxs9-pulse" } else { "oxs9-dot" } } "{x.status.label()}" }
                                        }
                                        p { class: "oxs9-pl", "Deployed {x.deployed} · {x.deploys} deploys · from “{x.task}”" }
                                        div { class: "oxs9-pacts",
                                            button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{on}", onclick: move |_| pv.set(if on { 0 } else { id }), {nico("eye")} span { "Preview" } }
                                            button { r#type: "button", class: "oxscn-btn", disabled: x.status == SiteStatus::Building, onclick: move |_| { redeploy(&mut c.sites.write(), id); c.said.set(format!("Redeploying {xn}")); }, {nico("refresh")} span { "Redeploy" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(o) = open.clone() {
                        aside { class: "oxs9-pvw", "aria-label": "Preview of {o.name}",
                            div { class: "oxs9-pvbar",
                                span { class: "oxs9-tl3", i {} i {} i {} }
                                span { class: "oxs9-surl oxs9-ell", {nico("lock")} "{o.url}" }
                                button { r#type: "button", class: "oxscn-ib", "aria-label": "Close preview", onclick: move |_| pv.set(0), {nico("close")} }
                            }
                            div { class: "oxs9-webpg",
                                h2 { "{o.page.0}" }
                                p { "{o.page.1}" }
                                div { class: "oxs9-blocks", for k in 0..o.page.2 { span { key: "{k}" } } }
                            }
                        }
                    }
                }
                if building {
                    span { class: "oxs9-tick oxs9-tick-d", onanimationend: move |_| { let n = finish_deploys(&mut c.sites.write()); if n > 0 { c.said.set("Deploy finished: live".to_string()); } } }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── App ──

#[component]
fn TaskdeckApp(
    slug: String,
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<TaskdeckData>,
) -> Element {
    let (st, own) = split(&state);
    let loading = st == ScreenState::Loading;
    let start_view = view_of(&slug);
    let d0 = use_hook(move || data.unwrap_or_else(|| TaskdeckData::for_state(st)));
    let env0 = if own == "cloud" {
        EnvMode::Cloud
    } else {
        EnvMode::Local
    };
    let d1 = d0.clone();
    let tasks_s = use_signal(move || {
        let mut l = d1.tasks;
        if own == "approval"
            && let Some(t) = l.iter_mut().find(|t| t.id == 1)
        {
            t.status = Status::NeedsYou;
            t.steps = (4, 6);
        }
        l
    });
    let view = use_signal(move || start_view);
    let open = use_signal(move || if st == ScreenState::Sample { 1u32 } else { 0 });
    let env = use_signal(move || env0);
    let steps = use_signal(move || {
        tasks_s
            .peek()
            .iter()
            .find(|t| t.id == 1)
            .map(run_for)
            .unwrap_or_default()
    });
    let said = use_signal(String::new);
    let drawer = use_signal(move || own == "menu");
    let asked = use_signal(|| (0u32, String::new()));
    let (d2, d3, d4, d5) = (d0.clone(), d0.clone(), d0.clone(), d0);
    let plugs = use_signal(move || d2.plugins);
    let skills_s = use_signal(move || d3.skills);
    let scheds = use_signal(move || d4.schedules);
    let sites_s = use_signal(move || d5.sites);
    let ctx = Ctx {
        tasks: tasks_s,
        view,
        open,
        env,
        steps,
        said,
        drawer,
        asked,
        plugs,
        skills: skills_s,
        scheds,
        sites: sites_s,
    };
    let mut side = use_signal(move || {
        if hidden {
            "hidden"
        } else if rail {
            "rail"
        } else {
            "full"
        }
    });
    let panel = use_signal(|| true);
    let pnar = use_signal(|| false);
    let v = view();
    let sd = side();
    let oid = open();
    let mut dr = drawer;
    let root_style = format!("--oxs9-h:{height}px");
    rsx! {
        div { class: "oxs9", "data-layout": "{layout}", "data-side": "{sd}", "data-view": "{v}", "data-drawer": "{dr()}", "data-pn": "{panel()}", "data-pnar": "{pnar()}", style: "{root_style}",
            if sd != "hidden" {
                Sidebar { ctx, loading, rail: sd == "rail", menu_open: own == "menu", on_hide: move |_| { side.set("hidden"); dr.set(false); } }
            }
            button { r#type: "button", class: "oxs9-scrim", "aria-label": "Close sidebar", tabindex: "-1", onclick: move |_| dr.set(false) }
            main { class: "oxs9-main",
                div { class: "oxs9-bar",
                    button { r#type: "button", class: "oxscn-ib oxs9-menu", "aria-label": "Show sidebar", onclick: move |_| { if *side.peek() == "hidden" { side.set("full"); } dr.toggle(); }, {nico("side")} }
                    if sd == "rail" {
                        button { r#type: "button", class: "oxscn-ib oxs9-wide", "aria-label": "Expand sidebar", onclick: move |_| side.set("full"), {nico("side")} }
                    }
                    span { class: "oxs9-crumb oxs9-ell", "{view_title(v)}" }
                }
                match v {
                    "task" => rsx! {
                        div { class: "oxs9-split",
                            TaskView { ctx, loading, panel, pnar }
                            if panel() || pnar() {
                                aside { class: "oxs9-pnl", id: "oxs9-pnl", "aria-label": "Side panel",
                                    if loading { div { class: "oxs9-pbody", {skel(6)} } } else if oid == 0 {
                                        div { class: "oxs9-pempty", {nico("panel")} p { "Artifacts, files, the terminal and the browser show up here while a task runs." } }
                                    } else { SidePanel { key: "{oid}", ctx } }
                                }
                            }
                        }
                    },
                    "board" => rsx! { BoardView { ctx, loading } },
                    "plugins" => rsx! { PluginsView { ctx, loading, skills_tab: own == "skills" } },
                    "scheduled" => rsx! { ScheduledView { ctx, loading, editing: own == "editing" } },
                    "sites" => rsx! { SitesView { ctx, loading, preview: own == "preview" } },
                    _ => rsx! { StartView { ctx, loading } },
                }
            }
            p { class: "oxs9-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

macro_rules! s9_screen {
    ($(#[$m:meta])* $name:ident, $slug:literal) => {
        $(#[$m])*
        #[component]
        pub fn $name(
            /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
            #[props(default = "inset".to_string())]
            layout: String,
            /// A state key (`sample`, `loading`, `empty` or the screen's own state).
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
            /// Your data (tasks, plugins, skills, schedules, sites). `None` shows the sample for
            /// `state`; start from `TaskdeckData::default()` and replace what you have.
            #[props(default)]
            data: Option<TaskdeckData>,
        ) -> Element {
            rsx! { TaskdeckApp { slug: $slug.to_string(), state, layout, height, rail, hidden: sidebar_hidden, data } }
        }
    };
}

s9_screen!(
    /// Taskdeck / Start: the task composer with attach (+), run mode, model, dictation and send,
    /// the folder and where it runs, and starter ideas. Sending starts the task and opens it.
    /// State `cloud` (Cloud mode).
    TaskdeckStartScreen,
    "taskdeck-start"
);
s9_screen!(
    /// Taskdeck / Task: a running task's step timeline with tool calls and outputs, pause, an
    /// approval card, follow-ups and the side panel (Artifacts, Files, Terminal, Browser). State
    /// `approval` (the task waits for you).
    TaskdeckTaskScreen,
    "taskdeck-task"
);
s9_screen!(
    /// Taskdeck / Plugins & skills: the plugin catalogue (search, categories, install, on/off,
    /// what each may do, remove) and skills (bundled and yours, on/off, new skill with checks).
    /// State `skills` (the Skills tab).
    TaskdeckPluginsScreen,
    "taskdeck-plugins"
);
s9_screen!(
    /// Taskdeck / Scheduled: tasks that run on a schedule, with Run now, Pause / Resume, Edit,
    /// Delete and New schedule (name, prompt, how often, a 24-hour time). State `editing`.
    TaskdeckScheduledScreen,
    "taskdeck-scheduled"
);
s9_screen!(
    /// Taskdeck / Sites: sites tasks built and deployed, with status (Live, Deploying, Deploy
    /// failed), a preview pane with its address bar and Redeploy. State `preview`.
    TaskdeckSitesScreen,
    "taskdeck-sites"
);
s9_screen!(
    /// Taskdeck / Board: tasks by status (Queued, Running, Needs you, Done) as a board or a list,
    /// with Run now, Back to queue, Mark done and Delete on each card. State `menu` (the account
    /// menu open, with Usage and Daily check-in).
    TaskdeckBoardScreen,
    "taskdeck-board"
);
