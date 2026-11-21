//! Screens S4: agent ops II on the Screens 1 frame with the Fieldwork persona's sidebar. Agents
//! (cards with a pixel mark drawn from the name), Skills, Schedules, Usage, Permissions and Run
//! Detail, each with full-view states (`oxyn_core::kit::opsscr::states_for`) and real
//! interactions. Styles in `opsscr.css` (`oxo-` shared, `oxog-` Agents, `oxos-` Skills, `oxoh-`
//! Schedules, `oxou-` Usage, `oxop-` Permissions, `oxor-` Run Detail); the S3 `oxa-` helpers are
//! reused. Chrome stays monochrome; colour is for official marks, identity hues and status.

use dioxus::prelude::*;
use oxyn_core::kit::opsscr::{
    AGENT_TABS, Agent, AgentState, BUDGET, COST_MOST, LAST_RUNS, LATELY, LEVELS, MODELS,
    MONTH_DAYS, NOW_MINS, RED_LINES, RUN_NOW, Rule, SKILL_CATS, STRIP_MINS, ScreenState, Skill,
    Span, SpanKind, TODAY, agent_counts, agent_view, agents, apps_words, bar, bars, breakdown,
    counts_words, cumulative, daily_spend, eur, forecast, forecast_words, install_step, mark_bits,
    mark_hue, month_cost, new_agent, next_sched, pct, perm_apps, perm_counts, rule_for, run_totals,
    sched_counts, schedules, secs, skill_says, skill_steps, skill_view, skills, span_io, span_view,
    spans, split, states_for, strip_hours, strip_marks, ticks, toks, usage_totals, uses_n,
};
use oxyn_core::kit::screensfid::{PERSON, PERSON_INI};

use super::screensfid::{AppFrame, ico, nico, use_threads};
use super::workscr::{find, tabs};
use crate::brand::BrandIcon;

/// The state bar entries of each S4 screen (for a host's `ScreenStage { states }`).
pub fn s4_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn focus_on(e: MountedEvent) {
    let d = e.data();
    spawn(async move {
        let _ = d.set_focus(true).await;
    });
}

fn pill(label: &str, tone: &'static str) -> Element {
    rsx! { span { class: "oxa-pill", "data-tone": "{tone}", "{label}" } }
}

fn kv(k: &'static str, v: Element) -> Element {
    rsx! { div { class: "oxa-kv", span { class: "oxa-k", "{k}" } span { class: "oxa-v", {v} } } }
}

fn h3(t: &str, right: Element) -> Element {
    rsx! { div { class: "oxo-h3", span { class: "oxv-ell", "{t}" } span { class: "oxscn-grow" } {right} } }
}

fn none(icon: &'static str, t: &'static str, l: &'static str) -> Element {
    rsx! {
        div { class: "oxv-none",
            span { class: "oxv-none-g", {nico(icon)} }
            p { class: "oxv-none-t", "{t}" }
            p { class: "oxv-none-l", "{l}" }
        }
    }
}

fn sk_cards(n: usize) -> Element {
    rsx! {
        div { class: "oxo-skgrid", "aria-hidden": "true",
            for i in 0..n { div { key: "{i}", class: "oxo-skcard", span { class: "oxv-skb" } span { class: "oxv-skl" } span { class: "oxv-skl" } } }
        }
    }
}

fn sk_rows(n: usize) -> Element {
    rsx! {
        div { class: "oxa-skrows", "aria-hidden": "true",
            for i in 0..n { div { key: "{i}", class: "oxa-skrow", span { class: "oxv-skb" } span { class: "oxv-skl" } } }
        }
    }
}

/// The work area's tab strip: one open item with a close button.
fn wtab(head: Element, label: String, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "oxa-wtabs", role: "tablist", "aria-label": "Open items",
            span { class: "oxa-wtab", role: "tab", "aria-selected": "true",
                {head} span { class: "oxv-ell", "{label}" }
                button { r#type: "button", class: "oxa-wtab-x", "aria-label": "Close {label}", onclick: move |_| on_close.call(()), {nico("close")} }
            }
        }
    }
}

/// An app mark on a white tile (official colours), or the globe for the web.
fn app_logo(app: &str, size: f32) -> Element {
    if app == "web" {
        return rsx! { span { class: "oxo-globe", {nico("globe")} } };
    }
    rsx! { BrandIcon { name: app.to_string(), size, coloured: true, fallback: true } }
}

/// A model chip: the official mark and its name.
fn model_chip(m: usize) -> Element {
    let (l, b) = MODELS[m.min(MODELS.len() - 1)];
    rsx! { span { class: "oxo-model", BrandIcon { name: b.to_string(), size: 12.0, coloured: true } span { class: "oxv-ell", "{l}" } } }
}

/// The pixel mark of a name in its hue (3 × 3, mirrored).
fn pmark(name: &str, hue: &'static str, big: bool) -> Element {
    let bits = mark_bits(name);
    rsx! {
        span { class: "oxo-pm", "data-hue": "{hue}", "data-big": "{big}", "aria-hidden": "true",
            span { class: "oxo-pm-g",
                for (i, on) in bits.iter().enumerate() { span { key: "{i}", "data-on": "{on}" } }
            }
        }
    }
}

/// A two-week run chart in a hue.
fn runs_chart(days: &[u8], hue: &'static str) -> Element {
    let b = bars(days);
    rsx! {
        div { class: "oxo-chart", "data-hue": "{hue}", role: "img", "aria-label": "Runs per day over two weeks",
            div { class: "oxo-chart-y", span { "10" } span { "5" } span { "0" } }
            div { class: "oxo-chart-p",
                div { class: "oxo-chart-bars",
                    for (i, h) in b.iter().enumerate() {
                        span { key: "{i}", class: "oxo-bar", "data-last": "{i + 4 >= b.len()}", style: "height:{h.max(&3)}%" }
                    }
                }
                div { class: "oxo-chart-x", span { "2 weeks ago" } span { "Today" } }
            }
        }
    }
}

fn hue_of(a: &Agent) -> &'static str {
    match a.id {
        1 => "blue",
        2 => "purple",
        3 => "pink",
        4 => "teal",
        5 => "orange",
        6 => "",
        _ => mark_hue(&a.name),
    }
}

fn euro0(c: u32) -> String {
    format!("€{}", (c + 50) / 100)
}

fn thou(n: u32) -> String {
    let s = n.to_string();
    let mut o = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            o.push(',');
        }
        o.push(ch);
    }
    o
}

// ───────────────────────────────────────────────────────────── Agents ──

/// Screen – Agent / Agents: each agent as a card (pixel mark, role, a status tag, official app
/// marks, model, runs and spend) under tabs by state with counts, in a grid or a list. Typing a
/// name in New agent builds its mark live. The work area (348) holds the agent: status, model,
/// owner, two weeks of runs, its brief, skills, apps and latest work, with Message and Pause.
/// States: `sample`, `loading`, `empty`, `creating`.
#[component]
pub fn AgentsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: sort the cards by spend, most first.
    #[props(default)]
    by_spend: bool,
    /// Oxyn: a spend bar against a €50 cap on each card.
    #[props(default)]
    budget_bars: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut list = use_signal(move || agents(sample));
    let tab = use_signal(|| "all");
    let mut as_list = use_signal(|| false);
    let mut sel = use_signal(move || if sample { Some(1u32) } else { None });
    let mut making = use_signal(move || sk == "creating");
    let mut nm = use_signal(move || {
        if sk == "creating" {
            "Scout".to_string()
        } else {
            String::new()
        }
    });
    let mut role = use_signal(move || {
        if sk == "creating" {
            "Finds leads and qualifies them".to_string()
        } else {
            String::new()
        }
    });
    let mut model = use_signal(|| 1usize);
    let mut msg_open = use_signal(|| false);
    let mut msg = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let c = agent_counts(&all);
    let view = agent_view(&all, tab(), by_spend);
    let cur = sel().and_then(|id| all.iter().find(|a| a.id == id).cloned());
    let mut create = move || {
        let n = nm.peek().clone();
        let r = role.peek().clone();
        if let Some(id) = new_agent(&mut list.write(), &n, &r, *model.peek()) {
            sel.set(Some(id));
            making.set(false);
            nm.set(String::new());
            role.set(String::new());
            said.set(format!("{} is ready", n.trim()));
        }
    };
    let mut toggle_pause = move |id: u32| {
        if let Some(a) = list.write().iter_mut().find(|a| a.id == id) {
            a.state = if a.state == AgentState::Paused {
                AgentState::Idle
            } else {
                AgentState::Paused
            };
            said.set(format!(
                "{} {}",
                a.name,
                if a.state == AgentState::Paused {
                    "paused"
                } else {
                    "resumed"
                }
            ));
        }
    };
    let tab_items: Vec<_> = AGENT_TABS
        .iter()
        .enumerate()
        .map(|(i, (k, l))| (*k, l.to_string(), Some(c[i])))
        .collect();
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(a) = cur.clone() {
                {wtab(rsx! { {nico("user")} }, a.name.clone(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxog-hd",
                        span { class: "oxog-tile", {pmark(&a.name, hue_of(&a), false)} }
                        span { class: "oxo-tt", b { class: "oxv-ell", "{a.name}" } span { class: "oxv-q oxv-ell", "{a.role}" } }
                        button { r#type: "button", class: "oxv-pri", "aria-expanded": "{msg_open}", onclick: move |_| msg_open.toggle(), {nico("chat")} span { "Message" } }
                        button { r#type: "button", class: "oxscn-btn oxo-ib", "aria-label": if a.state == AgentState::Paused { "Resume {a.name}" } else { "Pause {a.name}" },
                            "aria-pressed": "{a.state == AgentState::Paused}", onclick: move |_| toggle_pause(a.id), {nico("pause")} }
                    }
                    if msg_open() {
                        div { class: "oxog-msg",
                            textarea { class: "oxog-ta", rows: "2", placeholder: "Message {a.name}…", "aria-label": "Message {a.name}", value: "{msg}", onmounted: focus_on,
                                oninput: move |e| msg.set(e.value()),
                                onkeydown: {
                                    let n = a.name.clone();
                                    move |e: KeyboardEvent| {
                                        if e.key() == Key::Escape { msg_open.set(false); }
                                        if e.key() == Key::Enter && !e.modifiers().shift() {
                                            e.prevent_default();
                                            if !msg.peek().trim().is_empty() { said.set(format!("Sent to {n}")); msg.set(String::new()); msg_open.set(false); }
                                        }
                                    }
                                } }
                            span { class: "oxv-q", "Enter sends · Esc closes" }
                        }
                    }
                    div { class: "oxa-kvs",
                        {kv("Status", rsx! { span { class: "oxa-dot", "data-tone": if a.state == AgentState::Working { "ok" } else { a.state.tone() } } span { class: "oxv-ell", "{a.now}" } })}
                        {kv("Model", rsx! { BrandIcon { name: MODELS[a.model].1.to_string(), size: 14.0, coloured: true } span { "{MODELS[a.model].0}" } })}
                        {kv("Reports to", rsx! { span { class: "oxo-av", "{PERSON_INI}" } span { class: "oxv-ell", "{PERSON}" } })}
                    }
                    {h3("Runs", rsx! { span { class: "oxv-q", "{a.runs} this month · {eur(a.spend)}" } })}
                    {runs_chart(&a.days, "blue")}
                    {h3("Instructions", rsx! {})}
                    Clamp { text: a.brief.to_string() }
                    {h3("Skills", rsx! { button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Opened Skills".into()), "Skills" {nico("next")} } })}
                    div { class: "oxo-chips",
                        if a.skills.is_empty() { span { class: "oxv-q", "No skills yet" } }
                        for s in a.skills.iter().copied() { span { key: "{s}", class: "oxo-chip", {nico("spark")} span { class: "oxv-ell", "{s}" } } }
                    }
                    {h3("Can use", rsx! { button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Opened Permissions".into()), "Permissions" {nico("next")} } })}
                    div { class: "oxo-chips",
                        if a.apps.is_empty() { span { class: "oxv-q", "No apps yet" } }
                        for p in a.apps.iter().copied() { span { key: "{p}", class: "oxo-chip", {app_logo(p, 13.0)} span { class: "oxv-ell", "{app_name(p)}" } } }
                    }
                    {h3("Recent tasks", rsx! {})}
                    ul { class: "oxo-list",
                        if a.recent.is_empty() { li { class: "oxv-q", "Nothing yet" } }
                        for (i, (t, w)) in a.recent.iter().enumerate() {
                            li { key: "{t}", class: "oxo-li", span { class: "oxa-ring-s", "data-tone": if i == 0 { "run" } else { "ok" } } span { class: "oxv-ell", "{t}" } span { class: "oxscn-grow" } span { class: "oxv-q", "{w}" } }
                        }
                    }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick an agent to see what it is doing, what it may use and what it costs." }
            }
        }
    };
    let new_btn = rsx! {
        div { class: "oxv-pop-w",
            button { r#type: "button", class: "oxv-pri", "aria-label": "New agent", "aria-haspopup": "dialog", "aria-expanded": "{making}", onclick: move |_| making.toggle(), {nico("plus")} span { class: "oxa-bl", "New agent" } }
            if making() {
                div { class: "oxscn-catch", onclick: move |_| making.set(false) }
                div { class: "oxv-pop oxog-new", role: "dialog", "aria-label": "New agent",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { making.set(false) },
                    div { class: "oxog-newhd",
                        span { class: "oxog-tile oxog-tile-l", {pmark(&nm(), mark_hue(&nm()), true)} }
                        span { class: "oxo-tt", b { class: "oxv-ell", if nm().trim().is_empty() { "New agent" } else { "{nm}" } } span { class: "oxv-q", "The mark draws itself from the name" } }
                    }
                    label { class: "oxo-fl", "Name"
                        input { class: "oxv-in", r#type: "text", placeholder: "Scout", value: "{nm}", onmounted: focus_on,
                            oninput: move |e| nm.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { create() } }
                    }
                    label { class: "oxo-fl", "What it does"
                        input { class: "oxv-in", r#type: "text", placeholder: "Finds leads and qualifies them", value: "{role}",
                            oninput: move |e| role.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { create() } }
                    }
                    div { class: "oxo-fl", role: "radiogroup", "aria-label": "Model", "Model"
                        div { class: "oxo-chips",
                            for (i, (l, b)) in MODELS.iter().copied().enumerate() {
                                button { key: "{l}", r#type: "button", role: "radio", class: "oxo-chip oxo-pick", "aria-checked": "{model() == i}", onclick: move |_| model.set(i),
                                    BrandIcon { name: b.to_string(), size: 12.0, coloured: true } span { "{l}" } }
                            }
                        }
                    }
                    div { class: "oxv-pop-a",
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| making.set(false), "Cancel" }
                        button { r#type: "button", class: "oxv-pri", disabled: nm().trim().is_empty(), onclick: move |_| create(), "Create agent" }
                    }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "agents", title: "Agents",
            badge: rsx! { {tabs("Agents", tab_items, tab)} },
            actions: rsx! {
                div { class: "oxv-seg oxo-seg", role: "group", "aria-label": "View",
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "Grid", "aria-pressed": "{!as_list()}", onclick: move |_| as_list.set(false), {nico("apps")} }
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "List", "aria-pressed": "{as_list()}", onclick: move |_| as_list.set(true), {nico("list")} }
                }
                {new_btn}
            },
            work, work_label: "Agent", work_w: 348,
            div { class: "oxog",
                if loading { {sk_cards(6)} }
                else if all.is_empty() {
                    {none("bot", "No agents yet", "Make one with New agent: give it a name, a job and the apps it may use.")}
                } else if view.is_empty() {
                    p { class: "oxa-mute", role: "status", "No agents here." }
                } else {
                    ul { class: "oxog-grid", "data-view": if as_list() { "list" } else { "grid" }, "aria-label": "Agents",
                        for a in view {
                            {
                                let id = a.id;
                                let on = sel() == Some(id);
                                let hue = hue_of(&a);
                                let cap = (a.spend * 100 / 5000).min(100);
                                rsx! {
                                    li { key: "{id}",
                                        button { r#type: "button", class: "oxog-card", "data-on": "{on}", "data-paused": "{a.state == AgentState::Paused}", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                                            span { class: "oxog-top",
                                                span { class: "oxog-tile", {pmark(&a.name, hue, false)} }
                                                span { class: "oxo-tt", b { class: "oxv-ell", "{a.name}" } span { class: "oxv-q oxv-ell", "{a.role}" } }
                                                {pill(a.state.label(), a.state.tone())}
                                            }
                                            span { class: "oxog-apps",
                                                span { class: "oxog-logos", for p in a.apps.iter().copied() { span { key: "{p}", class: "oxog-logo", {app_logo(p, 12.0)} } } }
                                                span { class: "oxv-ell oxv-q", "{apps_words(a.apps.len())}" }
                                            }
                                            if budget_bars {
                                                span { class: "oxog-cap", title: "{eur(a.spend)} of €50", span { style: "width:{cap}%" } }
                                            }
                                            span { class: "oxog-foot",
                                                {model_chip(a.model)}
                                                span { class: "oxscn-grow" }
                                                span { class: "oxv-q", "{a.runs} runs" }
                                                span { class: "oxog-sep" }
                                                span { class: "oxv-q", "{euro0(a.spend)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

/// Instructions clamped to four lines with Show all.
#[component]
fn Clamp(text: String) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "oxo-quote",
            p { class: "oxo-clamp", "data-open": "{open}", "{text}" }
            button { r#type: "button", class: "oxv-link", "aria-expanded": "{open}", onclick: move |_| open.toggle(), if open() { "Show less" } else { "Show all" } }
        }
    }
}

fn app_name(id: &str) -> &'static str {
    match id {
        "github" => "GitHub",
        "linear" => "Linear",
        "slack" => "Slack",
        "gmail" => "Gmail",
        "googledrive" => "Google Drive",
        "googlecalendar" => "Google Calendar",
        "hubspot" => "HubSpot",
        "stripe" => "Stripe",
        "notion" => "Notion",
        "postgresql" => "Postgres",
        _ => "Web",
    }
}

// ───────────────────────────────────────────────────────────── Skills ──

/// Screen – Agent / Skills: installed skills as cards (a glyph in the skill's hue, a switch, who
/// made it), category chips and a search, then Discover with Install that fills a ring. The work
/// area shows the skill: runs over two weeks, when it steps in, what it does, what it needs and
/// what ran lately. States: `sample`, `loading`, `empty`, `installing`.
#[component]
pub fn SkillsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: cards without the description line.
    #[props(default)]
    compact: bool,
    /// Oxyn: Discover sorted by how many use each skill.
    #[props(default)]
    by_use: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut list = use_signal(move || skills(sample));
    let cat = use_signal(|| "all");
    let q = use_signal(String::new);
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "installing" { 13u32 } else { 1 })
        } else {
            None
        }
    });
    let mut inst = use_signal(move || {
        if sk == "installing" {
            vec![(13u32, 50u32)]
        } else {
            Vec::new()
        }
    });
    let mut hold = use_signal(move || sk == "installing");
    let mut adding = use_signal(|| false);
    let mut new_t = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let (installed, mut discover) = skill_view(&all, cat(), &q());
    if by_use {
        discover.sort_by_key(|s| std::cmp::Reverse(uses_n(s.uses)));
    }
    let n_inst = all.iter().filter(|s| s.installed).count();
    let n_disc = all.len() - n_inst;
    let cur = sel().and_then(|id| all.iter().find(|s| s.id == id).cloned());
    let mut flip = move |id: u32| {
        if let Some(s) = list.write().iter_mut().find(|s| s.id == id) {
            s.on = !s.on;
            said.set(format!(
                "{} turned {}",
                s.name,
                if s.on { "on" } else { "off" }
            ));
        }
    };
    // Install: each press of the ring (or the animation tick) adds a quarter; full installs it.
    let mut install = move |id: u32| {
        let p = inst
            .peek()
            .iter()
            .find(|x| x.0 == id)
            .map(|x| x.1)
            .unwrap_or(0);
        let (np, done) = install_step(p);
        inst.write().retain(|x| x.0 != id);
        if done {
            if let Some(s) = list.write().iter_mut().find(|s| s.id == id) {
                s.installed = true;
                s.on = true;
                said.set(format!("{} installed", s.name));
            }
        } else {
            inst.write().push((id, np));
        }
    };
    let mut remove = move |id: u32| {
        if let Some(s) = list.write().iter_mut().find(|s| s.id == id) {
            s.installed = false;
            s.on = false;
            said.set(format!("{} removed", s.name));
        }
    };
    let mut add = move || {
        let t = new_t.peek().trim().to_string();
        if t.is_empty() {
            return;
        }
        let id = list.peek().iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let name: &'static str = Box::leak(t.into_boxed_str());
        list.write().push(Skill {
            id,
            name,
            blurb: "Your new skill – teach it by showing it once",
            glyph: "spark",
            hue: "purple",
            cat: "writing",
            by: "Yours",
            installed: true,
            on: true,
            uses: "",
            days: [0; 14],
        });
        sel.set(Some(id));
        new_t.set(String::new());
        adding.set(false);
        said.set(format!("{name} added"));
    };
    let cat_items: Vec<_> = SKILL_CATS
        .iter()
        .map(|(k, l)| (*k, l.to_string(), None))
        .collect();
    let card = move |s: Skill, disc: bool| {
        let id = s.id;
        let on = sel() == Some(id);
        let prog = inst().iter().find(|x| x.0 == id).map(|x| x.1);
        rsx! {
            li { key: "{id}", class: "oxos-card", "data-on": "{on}", "data-off": "{!disc && !s.on}",
                button { r#type: "button", class: "oxos-sel", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                    span { class: "oxos-top",
                        span { class: "oxos-g", "data-hue": "{s.hue}", {nico(s.glyph)} }
                        b { class: "oxv-ell", "{s.name}" }
                    }
                    if !compact { span { class: "oxos-blurb", "{s.blurb}" } }
                    span { class: "oxos-foot",
                        span { class: "oxos-by", "{s.by}" }
                        span { class: "oxscn-grow" }
                        if disc { span { class: "oxv-q", "{s.uses} use it" } }
                    }
                }
                if disc {
                    button { r#type: "button", class: "oxos-add", "data-p": "{prog.unwrap_or(0)}", "data-hold": "{hold()}", "aria-label": "Install {s.name}",
                        "aria-busy": "{prog.is_some()}",
                        onclick: move |_| { hold.set(false); install(id); },
                        onanimationend: move |_| install(id),
                        if let Some(p) = prog {
                            svg { class: "oxos-ring", width: "18", height: "18", view_box: "0 0 36 36", "aria-hidden": "true",
                                circle { cx: "18", cy: "18", r: "15", fill: "none", stroke: "currentColor", stroke_width: "3", opacity: ".2" }
                                circle { class: "oxos-ring-f", cx: "18", cy: "18", r: "15", fill: "none", stroke: "currentColor", stroke_width: "3", path_length: "100", stroke_dasharray: "{p} 100", transform: "rotate(-90 18 18)", stroke_linecap: "round" }
                            }
                        } else { {nico("plus")} }
                    }
                } else {
                    button { r#type: "button", role: "switch", class: "oxscn-switch oxos-sw", "aria-checked": "{s.on}", "aria-label": "{s.name}", onclick: move |_| flip(id), span {} }
                }
            }
        }
    };
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(s) = cur.clone() {
                {wtab(rsx! { span { class: "oxos-gs", "data-hue": "{s.hue}", {nico(s.glyph)} } }, s.name.to_string(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxos-hd",
                        span { class: "oxos-g oxos-g-l", "data-hue": "{s.hue}", {nico(s.glyph)} }
                        span { class: "oxo-tt", b { class: "oxo-big oxv-ell", "{s.name}" } span { class: "oxv-q", if s.by == "Yours" { "Yours · v1.4" } else { "{s.by} · v2.0" } } }
                    }
                    p { class: "oxa-brief", "{s.blurb}" }
                    div { class: "oxa-acts",
                        if s.installed {
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| flip(s.id), span { class: "oxa-dot", "data-tone": if s.on { "run" } else { "" } } span { if s.on { "Turn off" } else { "Turn on" } } }
                            button { r#type: "button", class: "oxscn-btn oxv-ghost", onclick: move |_| remove(s.id), {nico("trash")} span { "Remove" } }
                        } else {
                            button { r#type: "button", class: "oxv-pri", onclick: move |_| install(s.id), {nico("plus")} span { "Install" } }
                        }
                    }
                    {h3("Runs", rsx! { span { class: "oxv-q", if s.installed { "46× this week" } else { "{s.uses} use it" } } })}
                    {runs_chart(&s.days, s.hue)}
                    {h3("Steps in when you say", rsx! {})}
                    div { class: "oxo-quote oxos-says", for t in skill_says(s.id) { p { key: "{t}", "“{t}”" } } }
                    {h3("What it does", rsx! {})}
                    ol { class: "oxo-quote oxos-steps", for (i, t) in skill_steps(s.id).iter().enumerate() { li { key: "{i}", span { class: "oxos-n", "{i + 1}" } span { "{t}" } } } }
                    {h3("What it needs", rsx! {})}
                    div { class: "oxo-li",
                        span { class: "oxo-chip", {nico("shield")} BrandIcon { name: "googledrive".to_string(), size: 12.0, coloured: true } span { class: "oxv-ell", "Read files in Google Drive" } }
                        span { class: "oxscn-grow" }
                        button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Opened Permissions".into()), "Permissions" {nico("next")} }
                    }
                    {h3("Ran lately", rsx! {})}
                    ul { class: "oxo-list",
                        for (t, w) in [("Weekly update", "40 minutes ago"), ("Onboarding emails", "3 hours ago")] {
                            li { key: "{t}", class: "oxo-li", span { class: "oxv-ell", "{t}" } span { class: "oxscn-grow" } span { class: "oxv-q", "{w}" } }
                        }
                    }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick a skill to see when it steps in and what it needs." }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "skills", title: "Skills",
            actions: rsx! {
                {find("Search skills", "Search skills", q, "oxos-q")}
                div { class: "oxv-pop-w",
                    button { r#type: "button", class: "oxscn-btn", "aria-label": "New skill", "aria-haspopup": "dialog", "aria-expanded": "{adding}", onclick: move |_| adding.toggle(), {nico("plus")} span { class: "oxa-bl", "New skill" } }
                    if adding() {
                        div { class: "oxscn-catch", onclick: move |_| adding.set(false) }
                        div { class: "oxv-pop", role: "dialog", "aria-label": "New skill",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false) },
                            p { class: "oxv-pop-h", "What should the skill be called?" }
                            input { class: "oxv-in", r#type: "text", placeholder: "Board memo", "aria-label": "Skill name", value: "{new_t}", onmounted: focus_on,
                                oninput: move |e| new_t.set(e.value()), onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { add() } }
                            div { class: "oxv-pop-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                                button { r#type: "button", class: "oxv-pri", disabled: new_t().trim().is_empty(), onclick: move |_| add(), "Create" }
                            }
                        }
                    }
                }
            },
            work, work_label: "Skill", work_w: 368,
            div { class: "oxos", "data-compact": "{compact}",
                if loading { {sk_cards(9)} }
                else if all.is_empty() {
                    {none("bolt", "No skills yet", "Skills teach agents one job well. Make your own with New skill, or add one others use.")}
                } else {
                    {tabs("Category", cat_items, cat)}
                    section { class: "oxos-sec", "aria-label": "Installed",
                        h2 { class: "oxo-sh", "Installed " span { class: "oxa-n", "{n_inst}" } }
                        if installed.is_empty() { p { class: "oxa-mute", role: "status", "Nothing installed matches." } }
                        ul { class: "oxos-grid", for s in installed { {card(s, false)} } }
                    }
                    section { class: "oxos-sec", "aria-label": "Discover",
                        h2 { class: "oxo-sh", "Discover " span { class: "oxa-n", "{n_disc}" } }
                        p { class: "oxv-q oxos-sub", "Skills others use – add one and your agent can do it right away" }
                        if discover.is_empty() { p { class: "oxa-mute", role: "status", "Nothing to discover here." } }
                        ul { class: "oxos-grid", for s in discover { {card(s, true)} } }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Schedules ──

const TODAY_W: &str = "Today ";

fn hhmm(m: u32) -> String {
    let t = 4 * 60 + m;
    format!("{:02}:{:02}", t / 60, t % 60)
}

/// Screen – Agent / Schedules: the hours around now as a strip (agent activity before now, a mark
/// for each run after it that fills while it runs), then the schedule rows with the rhythm in
/// words, next run, last result, cost and a switch. The work area shows the schedule with Run now,
/// Pause, what it does, next runs (Skip) and last runs. States: `sample`, `loading`, `empty`,
/// `running`.
#[component]
pub fn SchedulesScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: hide paused schedules.
    #[props(default)]
    hide_paused: bool,
    /// Oxyn: a footer with what the schedules cost in a month.
    #[props(default)]
    month: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let mut list = use_signal(move || schedules(sample));
    let mut sel = use_signal(move || if sample { Some(1u32) } else { None });
    let mut running = use_signal(move || if sk == "running" { Some(1u32) } else { None });
    let mut skipped = use_signal(Vec::<usize>::new);
    let mut said = use_signal(String::new);
    let all = list();
    let (on_n, tot) = sched_counts(&all);
    let marks = strip_marks(&all);
    let next = next_sched(&all).cloned();
    let cur = sel().and_then(|id| all.iter().find(|s| s.id == id).cloned());
    let mut flip = move |id: u32| {
        if let Some(s) = list.write().iter_mut().find(|s| s.id == id) {
            s.on = !s.on;
            said.set(format!(
                "{} {}",
                s.name,
                if s.on { "resumed" } else { "paused" }
            ));
        }
    };
    let mut run_now = move |id: u32, name: &'static str| {
        running.set(Some(id));
        said.set(format!("{name} is running"));
    };
    let hours = strip_hours();
    let now_pct = NOW_MINS * 100 / STRIP_MINS;
    let line = "M0 72 C 4 70, 6 64, 9 68 S 14 74, 18 70 S 24 62, 27 69 S 31 72, 33.6 66";
    let rows: Vec<_> = all
        .iter()
        .filter(|s| !hide_paused || s.on)
        .cloned()
        .collect();
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(s) = cur.clone() {
                {wtab(rsx! { {nico("cycle")} }, s.name.to_string(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxoh-hd", span { class: "oxoh-sq", "data-hue": "{s.hue}" } b { class: "oxo-big oxv-ell", "{s.name}" } }
                    p { class: "oxoh-rh", {nico("cycle")} span { class: "oxv-ell", "{s.rhythm} · {s.project}" } }
                    div { class: "oxa-acts",
                        if running() == Some(s.id) {
                            button { r#type: "button", class: "oxv-pri", disabled: true, span { class: "oxa-spin", "data-tone": "run" } span { "Running" } }
                        } else {
                            button { r#type: "button", class: "oxv-pri", onclick: move |_| run_now(s.id, s.name), {nico("next")} span { "Run now" } }
                        }
                        button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{!s.on}", onclick: move |_| flip(s.id), {nico("pause")} span { if s.on { "Pause" } else { "Resume" } } }
                        span { class: "oxscn-grow" }
                        span { class: "oxv-q", "{eur(s.cost)} per run" }
                    }
                    {h3("What it does", rsx! {})}
                    p { class: "oxo-bar-q", "{s.what}" }
                    {h3("Next runs", rsx! {})}
                    div { class: "oxa-card oxo-rows",
                        for (i, (w, r)) in [(s.next.0, s.next.1), ("Nov 24 07:23", "in 7 days")].into_iter().enumerate() {
                            {
                                let sk = skipped().contains(&i);
                                rsx! {
                                    div { key: "{i}", class: "oxo-row", "data-skip": "{sk}",
                                        b { class: "oxv-ell", "{w}" } span { class: "oxv-q", "{r}" } span { class: "oxscn-grow" }
                                        button { r#type: "button", class: "oxv-link", "aria-label": if sk { "Undo skip {w}" } else { "Skip {w}" },
                                            onclick: move |_| { let mut v = skipped.write(); if let Some(p) = v.iter().position(|x| *x == i) { v.remove(p); } else { v.push(i); } },
                                            if sk { "Undo" } else { "Skip" } }
                                    }
                                }
                            }
                        }
                    }
                    {h3("Last runs", rsx! {})}
                    div { class: "oxa-card oxo-rows",
                        if running() == Some(s.id) {
                            div { class: "oxo-row", span { class: "oxo-tt", b { class: "oxv-ell", "Checking nine makers' sites" } span { class: "oxv-q", "Today 07:23 · step 2 of 5" } } span { class: "oxscn-grow" } {pill("Running", "run")} }
                        }
                        for (t, w, tone) in LAST_RUNS {
                            div { key: "{t}", class: "oxo-row", span { class: "oxo-tt", b { class: "oxv-ell", "{t}" } span { class: "oxv-q", "{w}" } } span { class: "oxscn-grow" } {pill(if tone == "bad" { "Failed" } else { "Done" }, tone)} }
                        }
                    }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick a schedule to run it now, skip a run or see how the last ones went." }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "schedules", title: "Schedules",
            badge: rsx! { span { class: "oxa-n", "{on_n} / {tot}" } },
            work, work_label: "Schedule", work_w: 368,
            div { class: "oxoh",
                if loading { {sk_rows(9)} }
                else if all.is_empty() {
                    {none("clock", "No schedules yet", "Ask an agent to do something every week, every morning or on the 1st, and it shows up here.")}
                } else {
                    h2 { class: "oxo-sh", "Around now" }
                    div { class: "oxoh-card",
                        div { class: "oxoh-ch", span { class: "oxv-q2", "Agent activity" } span { class: "oxscn-grow" } span { b { "2" } " actions / 10 min" } }
                        div { class: "oxoh-strip", role: "group", "aria-label": "Runs today",
                            svg { class: "oxoh-line", view_box: "0 0 100 80", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                path { d: "{line} L 33.6 80 L 0 80 Z", class: "oxoh-area" }
                                path { d: "{line}", fill: "none", stroke: "currentColor", stroke_width: "1.2", vector_effect: "non-scaling-stroke" }
                            }
                            span { class: "oxoh-now", style: "left:{now_pct}%", title: "Now, {hhmm(NOW_MINS)}" }
                            for (i, (id, x, hue)) in marks.iter().copied().enumerate() {
                                {
                                    let nm = all.iter().find(|s| s.id == id).map(|s| s.name).unwrap_or("");
                                    let run = running() == Some(id) && i == 0;
                                    rsx! {
                                        button { key: "{i}", r#type: "button", class: "oxoh-mk", "data-hue": "{hue}", "data-run": "{run}", "data-on": "{sel() == Some(id)}", style: "left:{x}%",
                                            "aria-label": "{nm} at {hhmm(x * STRIP_MINS / 100)}", onclick: move |_| sel.set(Some(id)), span {} }
                                    }
                                }
                            }
                            div { class: "oxoh-x", for (l, x) in hours.iter() { span { key: "{l}", style: "left:{x}%", "{l}" } } }
                        }
                        if let Some(n) = next.clone() {
                            p { class: "oxoh-next",
                                if running().is_some() { b { "Running now: " } } else { b { "Next: " } }
                                span { class: "oxv-ell", "{n.name}" } span { class: "oxv-q", " · {n.next.0.trim_start_matches(TODAY_W)}, {n.next.1}" } }
                        }
                    }
                    h2 { class: "oxo-sh", "Schedules" }
                    ul { class: "oxoh-ul", "aria-label": "Schedules",
                        for s in rows {
                            {
                                let id = s.id;
                                let on = sel() == Some(id);
                                let run = running() == Some(id);
                                rsx! {
                                    li { key: "{id}", class: "oxoh-row", "data-on": "{on}", "data-off": "{!s.on}",
                                        button { r#type: "button", class: "oxoh-sel", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                                            span { class: "oxoh-sq", "data-hue": "{s.hue}", "data-hollow": "{!s.on}" }
                                            span { class: "oxoh-nm", b { class: "oxv-ell", "{s.name}" } span { class: "oxv-q2 oxv-ell oxoh-rhy", "{s.rhythm}" } }
                                            span { class: "oxoh-when",
                                                if s.on { b { "{s.next.0}" } span { class: "oxv-q", "{s.next.1}" } } else { span { class: "oxv-q", "Paused" } }
                                            }
                                            span { class: "oxoh-last",
                                                if run { {pill("Running", "run")} }
                                                else { {pill(match s.last { "warn" => "Needs you", "bad" => "Failed", _ => "Done" }, match s.last { "warn" => "warn", "bad" => "bad", _ => "" })} }
                                            }
                                            span { class: "oxoh-cost", "{eur(s.cost)}" }
                                        }
                                        button { r#type: "button", role: "switch", class: "oxscn-switch", "aria-checked": "{s.on}", "aria-label": "{s.name} on", onclick: move |_| flip(id), span {} }
                                    }
                                }
                            }
                        }
                    }
                    if month {
                        p { class: "oxoh-month", {nico("coin")} span { "These schedules cost about " b { "{eur(month_cost(&all))}" } " a month" } }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Usage ──

/// Screen – Agent / Usage: spend this month against the budget (a running total with a budget
/// line and a plain-language forecast), the month's totals, where it went by project or model, and
/// what is worth a look. The work area shows the project: day-by-day spend and what cost the most.
/// States: `sample`, `loading`, `empty`, `over`.
#[component]
pub fn UsageScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: day-by-day bars in place of the running total.
    #[props(default)]
    daily: bool,
    /// Oxyn: an alert line at 80 % of the budget.
    #[props(default)]
    alert: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let over = sk == "over";
    let mut by_models = use_signal(|| false);
    let mut sel = use_signal(move || if sample { Some("tide") } else { None });
    let mut said = use_signal(String::new);
    let mut seen = use_signal(Vec::<usize>::new);
    let days = daily_spend(sample, over);
    let cum = cumulative(&days);
    let spent = cum.last().copied().unwrap_or(0);
    let rows = breakdown(by_models());
    let top = BUDGET.max(forecast(spent, TODAY)) as f32;
    // Chart geometry: 0..100 across, 0..60 high.
    let px = |d: u32| (d as f32) * 100.0 / (MONTH_DAYS as f32 - 1.0);
    let py = |c: u32| 60.0 - (c as f32) * 56.0 / top;
    let mut path = String::new();
    for (i, c) in cum.iter().enumerate() {
        path.push_str(&format!(
            "{}{:.2} {:.2} ",
            if i == 0 { "M" } else { "L" },
            px(i as u32),
            py(*c)
        ));
    }
    let fx = px(MONTH_DAYS - 1);
    let fy = py(forecast(spent, TODAY));
    let lx = px(TODAY - 1);
    let ly = py(spent);
    let by = py(BUDGET);
    let ay = py(BUDGET * 4 / 5);
    let dmax = days.iter().copied().max().unwrap_or(1).max(1);
    let (tasks_n, runs_n, per, tk) = usage_totals(spent);
    let cur = sel().and_then(|k| breakdown(false).into_iter().find(|r| r.0 == k));
    let worth = [
        (
            "Competitor scan cost €18.40 yesterday",
            "4× what it usually costs – it read 212 pages instead of 40",
            "4×",
            "warn",
        ),
        (
            "Benchkit is up 102% on October",
            "Its evals moved to Opus 5.5 on Nov 12",
            "+102%",
            "warn",
        ),
    ];
    let work = rsx! {
        div { class: "oxa-work",
            if let Some((k, n, hue, cents, d)) = cur {
                {wtab(rsx! { {ico("folder")} }, n.to_string(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxoh-hd", span { class: "oxoh-sq", "data-hue": "{hue}" } b { class: "oxo-big", "{n}" } }
                    p { class: "oxou-big", "{eur(cents)}" span { class: "oxv-q", " {pct(cents, spent)}% of November" } }
                    p { class: "oxou-delta", "data-up": "{d > 0}", if d > 0 { "↑{d}% vs last month" } else { "↓{-d}% vs last month" } }
                    {h3("Day by day", rsx! {})}
                    div { class: "oxou-days", "data-hue": "{hue}", role: "img", "aria-label": "{n} spend per day",
                        for i in 0..MONTH_DAYS {
                            {
                                let v = days.get(i as usize).copied().unwrap_or(0);
                                let h = v * 100 / dmax * 44 / 100;
                                let wob = i * 37 % 23;
                                let hh = if v == 0 { 2 } else { (h + wob).min(100) };
                                rsx! { span { key: "{i}", "data-future": "{v == 0}", style: "height:{hh}%" } }
                            }
                        }
                    }
                    p { class: "oxv-q", "Nov 1 – Nov {TODAY} · {eur(cents)}" }
                    {h3("What cost the most", rsx! {})}
                    div { class: "oxa-card oxo-rows",
                        for (t, r, c) in COST_MOST {
                            div { key: "{t}", class: "oxo-row", span { class: "oxo-tt", b { class: "oxv-ell", "{t}" } span { class: "oxv-q", "{r} runs" } } span { class: "oxscn-grow" } span { class: "oxou-num", "{eur(c)}" } }
                        }
                    }
                    if k == "tide" { p { class: "oxv-q oxo-pad0", "Competitor scan read more pages than usual on Nov 18." } }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick a project to see its spend day by day." }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "usage", title: "Usage",
            badge: rsx! { span { class: "oxa-n", "November" } },
            work, work_label: "Project", work_w: 368,
            div { class: "oxou",
                if loading { {sk_rows(9)} }
                else if days.is_empty() {
                    {none("chart", "No spend yet this month", "When agents run, what they cost lands here by project and model, against your budget.")}
                } else {
                    div { class: "oxou-col",
                        div { class: "oxou-card",
                            div { class: "oxou-head",
                                p { class: "oxv-q2", "Spent in November" }
                                p { class: "oxou-total", "{eur(spent)}" span { class: "oxv-q", " of {euro0(BUDGET)} · {pct(spent, BUDGET)}%" } }
                                p { class: "oxou-fc", "data-over": "{forecast(spent, TODAY) > BUDGET}", "{forecast_words(spent, TODAY)}" }
                            }
                            div { class: "oxou-chart", role: "img", "aria-label": "Spend so far {eur(spent)} against a budget of {euro0(BUDGET)}; {forecast_words(spent, TODAY)}",
                                svg { view_box: "0 0 100 62", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                    line { x1: "0", x2: "100", y1: "{by}", y2: "{by}", class: "oxou-budget", vector_effect: "non-scaling-stroke" }
                                    if alert { line { x1: "0", x2: "100", y1: "{ay}", y2: "{ay}", class: "oxou-alert", vector_effect: "non-scaling-stroke" } }
                                    if daily {
                                        for (i, d) in days.iter().enumerate() {
                                            rect { key: "{i}", x: "{px(i as u32) - 1.1}", width: "2.2", y: "{60.0 - (*d as f32) * 40.0 / dmax as f32}", height: "{(*d as f32) * 40.0 / dmax as f32}", class: "oxou-dbar" }
                                        }
                                    } else {
                                        path { d: "{path}", class: "oxou-line", vector_effect: "non-scaling-stroke" }
                                        line { x1: "{lx}", y1: "{ly}", x2: "{fx}", y2: "{fy}", class: "oxou-fline", vector_effect: "non-scaling-stroke" }
                                    }
                                }
                                span { class: "oxou-bl", style: "top:{by / 62.0 * 148.0 - 16.0}px", "Budget {euro0(BUDGET)}" }
                                if !daily { span { class: "oxou-dot", style: "left:{lx}%;top:{ly / 62.0 * 148.0}px" } }
                                div { class: "oxou-x", for (l, d) in [("Nov 1", 0u32), ("Nov 8", 7), ("Nov 15", 14), ("Nov 23", 22), ("Nov 30", 29)] { span { key: "{l}", style: "left:{px(d)}%", "{l}" } } }
                            }
                            div { class: "oxou-stats",
                                for (l, v) in [("Tasks", tasks_n.to_string()), ("Runs", thou(runs_n)), ("Per task", eur(per)), ("Tokens", toks(tk).to_uppercase())] {
                                    div { key: "{l}", class: "oxou-stat", span { class: "oxv-q2", "{l}" } b { "{v}" } }
                                }
                            }
                        }
                        div { class: "oxo-sh oxou-wh", span { "Where it went" } span { class: "oxscn-grow" }
                            div { class: "oxv-seg", role: "group", "aria-label": "Break down by",
                                button { r#type: "button", class: "oxv-seg-t", "aria-pressed": "{!by_models()}", onclick: move |_| by_models.set(false), "Projects" }
                                button { r#type: "button", class: "oxv-seg-t", "aria-pressed": "{by_models()}", onclick: move |_| by_models.set(true), "Models" }
                            }
                        }
                        ul { class: "oxou-card oxou-ul", "aria-label": "Where it went",
                            for (k, n, hue, c, d) in rows {
                                {
                                    let on = sel() == Some(k);
                                    let model = by_models();
                                    let w = pct(c, spent);
                                    rsx! {
                                        li { key: "{k}",
                                            button { r#type: "button", class: "oxou-row", "data-on": "{on}", "aria-current": if on { "true" } else { "false" },
                                                onclick: move |_| if !model { sel.set(Some(k)) } else { said.set(format!("{n}: {}", eur(c))) },
                                                if model { BrandIcon { name: hue.to_string(), size: 13.0, coloured: true } } else { span { class: "oxoh-sq", "data-hue": "{hue}" } }
                                                span { class: "oxv-ell oxou-nm", "{n}" }
                                                span { class: "oxou-track", "data-hue": if model { "blue" } else { hue }, span { style: "width:{(w * 2).min(100)}%" } }
                                                span { class: "oxou-num", "{eur(c)}" }
                                                span { class: "oxv-q oxou-pc", "{w}%" }
                                                span { class: "oxou-d", "data-up": "{d > 0}", if d > 0 { "↑{d}%" } else { "↓{-d}%" } }
                                                span { class: "oxou-ch", {nico("next")} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        h2 { class: "oxo-sh", "Worth a look" }
                        ul { class: "oxou-card oxou-ul", "aria-label": "Worth a look",
                            for (i, (t, l, tag, tone)) in worth.iter().copied().enumerate() {
                                li { key: "{i}", class: "oxou-w", "data-seen": "{seen().contains(&i)}",
                                    span { class: "oxou-wg", "data-tone": "{tone}", {nico(if i == 0 { "bell" } else { "trendup" })} }
                                    span { class: "oxo-tt", b { class: "oxv-ell", "{t}" } span { class: "oxv-q oxv-ell", "{l}" } }
                                    {pill(tag, tone)}
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { seen.write().push(i); said.set(format!("Opened: {t}")); }, "Open" }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Permissions ──

/// Screen – Agent / Permissions: a three-step autonomy choice (a radio group with arrow keys)
/// with the counts it leads to, each connected app with its own counts, and the fixed rules that
/// win over any level. The work area lists one app's actions by kind (reading, changing, risky),
/// each with a select of three settings, a refund limit and recent events. States: `sample`,
/// `loading`, `empty`, `handsoff`.
#[component]
pub fn PermissionsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: rows without the account line.
    #[props(default)]
    compact: bool,
    /// Oxyn: red lines locked (their selects are read-only, with a lock).
    #[props(default)]
    lock_red: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let apps = perm_apps(sample);
    let mut level = use_signal(move || {
        if sk == "handsoff" {
            "handsoff"
        } else {
            "balanced"
        }
    });
    let mut over = use_signal(Vec::<(&'static str, Rule)>::new);
    let mut sel = use_signal(move || if sample { Some("stripe") } else { None });
    let mut limit = use_signal(|| 50u32);
    let mut said = use_signal(String::new);
    let ov = over();
    let lv = level();
    let c = perm_counts(&apps, lv, &ov);
    let tot = (c.0 + c.1 + c.2).max(1);
    let rule_of = move |l: &str, k: &str| -> Rule {
        over()
            .iter()
            .find(|o| o.0 == l)
            .map(|o| o.1)
            .unwrap_or_else(|| rule_for(level(), k, l))
    };
    let mut set_rule = move |l: &'static str, r: Rule| {
        let mut v = over.write();
        v.retain(|o| o.0 != l);
        v.push((l, r));
        said.set(format!("{l}: {}", r.label()));
    };
    let cur = sel().and_then(|k| apps.iter().find(|a| a.0 == k).copied());
    let rule_sel = move |l: &'static str, k: &'static str, locked: bool| {
        let r = rule_of(l, k);
        rsx! {
            span { class: "oxop-rs", "data-r": "{r.key()}",
                span { class: "oxa-dot", "data-tone": match r { Rule::Own => "run", Rule::Ask => "warn", Rule::Never => "" } }
                if locked { span { class: "oxop-lock", title: "Locked red line", {nico("lock")} } }
                select { class: "oxop-sel", "aria-label": "{l}", disabled: locked, value: "{r.key()}",
                    onchange: move |e| set_rule(l, Rule::from_key(&e.value())),
                    for rr in [Rule::Own, Rule::Ask, Rule::Never] { option { key: "{rr.key()}", value: "{rr.key()}", selected: rr == r, "{rr.label()}" } }
                }
            }
        }
    };
    let work = rsx! {
        div { class: "oxa-work",
            if let Some((id, name, acct, acts)) = cur {
                {wtab(rsx! { {app_logo(id, 13.0)} }, name.to_string(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxop-hd", span { class: "oxop-logo oxop-logo-l", {app_logo(id, 22.0)} } span { class: "oxo-tt", b { class: "oxo-big", "{name}" } span { class: "oxv-q oxv-ell", "{acct}" } } }
                    for (gk, gl) in [("read", "Reads"), ("change", "Changes"), ("care", "Needs care")] {
                        {
                            let items: Vec<_> = acts.iter().copied().filter(|a| a.1 == gk).collect();
                            rsx! {
                                if !items.is_empty() {
                                    {h3(&format!("{gl}  {}", items.len()), rsx! {})}
                                    div { class: "oxa-card oxo-rows",
                                        for (l, k, g) in items {
                                            div { key: "{l}", class: "oxop-act",
                                                div { class: "oxo-row", {nico(g)} span { class: "oxv-ell", "{l}" } span { class: "oxscn-grow" } {rule_sel(l, k, lock_red && RED_LINES.iter().any(|r| r.0 == l))} }
                                                if l == "Issue refunds" && rule_of(l, k) == Rule::Own {
                                                    label { class: "oxop-lim", span { "On its own up to" }
                                                        input { class: "oxv-in oxop-num", r#type: "number", min: "0", step: "10", value: "{limit}", "aria-label": "Refund limit in euros",
                                                            oninput: move |e| if let Ok(v) = e.value().parse() { limit.set(v) } }
                                                        span { "€, asks above" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    {h3("Lately", rsx! {})}
                    div { class: "oxa-card oxo-rows",
                        if id == "stripe" {
                            for (t, w, tone) in LATELY {
                                div { key: "{t}", class: "oxo-row", span { class: "oxo-tt", b { class: "oxv-ell", "{t}" } span { class: "oxv-q", "{w}" } } span { class: "oxscn-grow" } {pill(if tone == "run" { "Approved" } else { "Done" }, tone)} }
                            }
                        } else {
                            p { class: "oxv-q oxo-row", "Nothing in the last week." }
                        }
                    }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick an app to set what the agent may do in it." }
            }
        }
    };
    let lv_keys: Vec<&'static str> = LEVELS.iter().map(|l| l.0).collect();
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "permissions", title: "Permissions",
            work, work_label: "App", work_w: 388,
            div { class: "oxop",
                if loading { {sk_rows(10)} }
                else if apps.is_empty() {
                    {none("shield", "No apps connected", "Connect an app and choose what the agent may read, change or never touch in it.")}
                } else {
                    div { class: "oxop-col",
                        p { class: "oxop-lead", "What your agent – and every agent it starts – may do without asking you first" }
                        h2 { class: "oxo-sh", "How much it does on its own" }
                        div { class: "oxop-card",
                            div { class: "oxop-levels", role: "radiogroup", "aria-label": "How much it does on its own",
                                for (k, l, d, g) in LEVELS {
                                    {
                                        let ks = lv_keys.clone();
                                        rsx! {
                                            button { key: "{k}", r#type: "button", role: "radio", class: "oxop-lv", "aria-checked": "{lv == k}", tabindex: if lv == k { "0" } else { "-1" },
                                                onclick: move |_| { level.set(k); over.write().clear(); said.set(l.to_string()); },
                                                onkeydown: move |e: KeyboardEvent| {
                                                    let i = ks.iter().position(|x| *x == k).unwrap_or(0);
                                                    let j = match e.key() { Key::ArrowRight | Key::ArrowDown => (i + 1) % 3, Key::ArrowLeft | Key::ArrowUp => (i + 2) % 3, _ => return };
                                                    e.prevent_default();
                                                    level.set(ks[j]);
                                                    over.write().clear();
                                                },
                                                span { class: "oxop-lt", {nico(g)} b { "{l}" } }
                                                span { class: "oxop-ld", "{d}" }
                                            }
                                        }
                                    }
                                }
                            }
                            p { class: "oxop-sum", "Does " b { "{c.0}" } " things on its own · asks before " b { "{c.1}" } " · never does " b { "{c.2}" } }
                            div { class: "oxop-tri", "aria-hidden": "true",
                                span { "data-r": "own", style: "flex:{c.0}" } span { "data-r": "ask", style: "flex:{c.1}" } span { "data-r": "never", style: "flex:{c.2}" }
                            }
                            span { class: "oxscn-sr", "{c.0} of {tot} on its own" }
                        }
                        h2 { class: "oxo-sh", "Apps " span { class: "oxa-n", "{apps.len()}" } }
                        ul { class: "oxop-card oxop-apps", "aria-label": "Apps",
                            for (id, name, acct, _) in apps.iter().copied() {
                                {
                                    let ac = perm_counts(&apps.iter().copied().filter(|a| a.0 == id).collect::<Vec<_>>(), lv, &ov);
                                    let on = sel() == Some(id);
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxop-app", "data-on": "{on}", "aria-current": if on { "true" } else { "false" }, onclick: move |_| sel.set(Some(id)),
                                                span { class: "oxop-logo", {app_logo(id, 18.0)} }
                                                span { class: "oxo-tt", b { class: "oxv-ell", "{name}" } if !compact { span { class: "oxv-q oxv-ell", "{acct}" } } }
                                                span { class: "oxop-mini", "aria-hidden": "true", for (r, n) in [("own", ac.0), ("ask", ac.1), ("never", ac.2)] { if n > 0 { span { key: "{r}", "data-r": "{r}", style: "width:{n * 8}px" } } } }
                                                span { class: "oxv-q oxv-ell oxop-cw", "{counts_words(ac)}" }
                                                span { class: "oxou-ch", {nico("next")} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        h2 { class: "oxo-sh oxop-redh", {nico("shield")} span { "Red lines" } }
                        p { class: "oxv-q oxop-redl", "These always ask – or never happen – whatever you pick above" }
                        div { class: "oxop-card oxo-rows",
                            for (l, app, _) in RED_LINES {
                                div { key: "{l}", class: "oxo-row oxop-red", span { class: "oxop-logo oxop-logo-s", {app_logo(app, 13.0)} } span { class: "oxv-ell", "{l}" } span { class: "oxscn-grow" } {rule_sel(l, "care", lock_red)} }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Run Detail ──

fn span_glyph(s: &Span) -> Element {
    if !s.app.is_empty() {
        return rsx! { span { class: "oxor-g", BrandIcon { name: s.app.to_string(), size: 13.0, coloured: true } } };
    }
    let (g, k) = match s.kind {
        SpanKind::Sub => ("people", "sub"),
        _ => ("spark", "think"),
    };
    rsx! { span { class: "oxor-g", "data-k": "{k}", {nico(g)} } }
}

/// Screen – Agent / Run Detail: one run on one time axis. The header holds the task, agent, model
/// and start, a Running tag and Stop; tiles show time, cost, tokens and steps; the trace draws
/// thinking, tool calls (official marks), sub-agents (their steps indented) and failures as
/// tinted bars with a now line. ↑ / ↓ move through the steps; the work area shows the step with
/// its input and output. States: `sample`, `loading`, `empty`, `stopped`.
#[component]
pub fn RunDetailScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: only the failures and the steps around them.
    #[props(default)]
    errors_only: bool,
    /// Oxyn: label each bar with its cost instead of its length.
    #[props(default)]
    cost_labels: bool,
) -> Element {
    let (st, sk) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st == ScreenState::Sample;
    let loading = st == ScreenState::Loading;
    let all = spans(sample);
    let mut stopped = use_signal(move || sk == "stopped");
    let mut sel = use_signal(move || {
        if sample {
            Some(if sk == "stopped" { 9u32 } else { 4 })
        } else {
            None
        }
    });
    let mut said = use_signal(String::new);
    let view = span_view(&all, errors_only, stopped());
    let (t, cost, tk, n, f) = run_totals(&all, stopped());
    let ids: Vec<u32> = view.iter().map(|s| s.id).collect();
    let cur = sel().and_then(|id| all.iter().find(|s| s.id == id).cloned());
    let now_pct = RUN_NOW as f32 * 100.0 / 700.0;
    let work = rsx! {
        div { class: "oxa-work",
            if let Some(s) = cur.clone() {
                {wtab(rsx! { {nico("doc")} }, s.label.to_string(), EventHandler::new(move |_| sel.set(None)))}
                div { class: "oxa-wbody",
                    div { class: "oxor-hd", {span_glyph(&s)} span { class: "oxo-tt", b { class: "oxv-ell", "{s.label}" } span { class: "oxv-q", if s.app.is_empty() { "{s.kind.label()}" } else { "Tool call · {app_name(s.app)}" } } }
                        {match s.kind { SpanKind::Fail => pill("Failed", "bad"), _ if s.at + s.dur > RUN_NOW && !stopped() => pill("Running", "run"), _ => pill("Done", "ok") }} }
                    div { class: "oxa-kvs",
                        {kv("Started", rsx! { span { "+{secs(s.at)}" } })}
                        {kv("Took", rsx! { span { "{secs(s.dur)}" } })}
                        {kv("Tokens", rsx! { span { class: "oxv-ell", "{thou(s.toks.0)} in · {thou(s.toks.1)} out" } })}
                        {kv("Cost", rsx! { span { "{eur(s.cost)}" } })}
                    }
                    {h3("Input", rsx! { button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Input copied".into()), {nico("copy")} "Copy" } })}
                    pre { class: "oxor-pre", "{span_io(s.id).0}" }
                    {h3("Output", rsx! { button { r#type: "button", class: "oxv-link", onclick: move |_| said.set("Output copied".into()), {nico("copy")} "Copy" } })}
                    pre { class: "oxor-pre", "data-bad": "{s.kind == SpanKind::Fail}", "{span_io(s.id).1}" }
                }
            } else {
                p { class: "oxa-mute oxo-pad", "Pick a step to see its input and output." }
            }
        }
    };
    let key_nav = {
        let ids = ids.clone();
        move |e: KeyboardEvent| {
            let i = sel().and_then(|id| ids.iter().position(|x| *x == id));
            let j = match (e.key(), i) {
                (Key::ArrowDown, Some(i)) => (i + 1).min(ids.len().saturating_sub(1)),
                (Key::ArrowUp, Some(i)) => i.saturating_sub(1),
                (Key::ArrowDown | Key::ArrowUp, None) => 0,
                (Key::Home, _) => 0,
                (Key::End, _) => ids.len().saturating_sub(1),
                _ => return,
            };
            e.prevent_default();
            if let Some(id) = ids.get(j) {
                sel.set(Some(*id));
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "fieldwork", nav_on: "agents", title: "Migrate users table",
            badge: rsx! { if sample { if stopped() { {pill("Stopped", "bad")} } else { {pill("Running", "run")} } } },
            actions: rsx! {
                if sample {
                    if stopped() {
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { stopped.set(false); said.set("Run resumed".into()); }, {nico("next")} span { class: "oxa-bl", "Resume" } }
                    } else {
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { stopped.set(true); said.set("Run stopped".into()); }, {nico("stop")} span { class: "oxa-bl", "Stop" } }
                    }
                }
            },
            work, work_label: "Step", work_w: 348,
            div { class: "oxor",
                if loading { {sk_rows(12)} }
                else if all.is_empty() {
                    {none("runs", "No run to show", "Start a task and its run shows here step by step, on one time axis.")}
                } else {
                    p { class: "oxor-sub", span { class: "oxog-tile oxor-tile", {pmark("Builder", "blue", false)} } span { class: "oxv-ell", "Builder · Opus 5.5 · started 07:22" } }
                    if stopped() {
                        div { class: "oxor-ban", role: "status", span { class: "oxa-dot", "data-tone": "bad" } span { class: "oxv-ell", "Stopped after “Dry run on staging” failed" } span { class: "oxscn-grow" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { stopped.set(false); said.set("Retrying from the dry run".into()); }, "Retry" } }
                    }
                    div { class: "oxor-tiles",
                        for (l, v, bad) in [("Time", secs(t), false), ("Cost", eur(cost), false), ("Tokens", toks(tk).to_uppercase(), false), ("Steps", format!("{n} · {f} failed"), f > 0)] {
                            div { key: "{l}", class: "oxor-tile2", span { class: "oxv-q2", "{l}" } b { "data-bad": "{bad}", "{v}" } }
                        }
                    }
                    div { class: "oxor-legend", "aria-hidden": "true",
                        for k in [SpanKind::Think, SpanKind::Tool, SpanKind::Sub, SpanKind::Fail] { span { key: "{k.key()}", span { class: "oxor-sw", "data-k": "{k.key()}" } "{k.label()}" } }
                    }
                    div { class: "oxor-trace",
                        div { class: "oxor-ax", "aria-hidden": "true",
                            span { class: "oxor-axl" }
                            span { class: "oxor-axt", for (l, x) in ticks() { span { key: "{l}", style: "left:{x}%", "{l}" } } }
                        }
                        div { class: "oxor-rows", role: "listbox", tabindex: "0", "aria-label": "Steps", "aria-activedescendant": if let Some(id) = sel() { "oxor-s{id}" } else { "" },
                            onkeydown: key_nav,
                            div { class: "oxor-grid", "aria-hidden": "true", span { class: "oxor-axl" } span { class: "oxor-gl", for (l, x) in ticks() { span { key: "{l}", style: "left:{x}%" } } if !stopped() { span { class: "oxor-now", style: "left:{now_pct}%" } } } }
                            for s in view {
                                {
                                    let id = s.id;
                                    let on = sel() == Some(id);
                                    let (x, w) = bar(&s);
                                    let lab = if s.kind == SpanKind::Fail { "failed".to_string() } else if cost_labels { eur(s.cost) } else { secs(s.dur) };
                                    rsx! {
                                        div { key: "{id}", id: "oxor-s{id}", class: "oxor-row", role: "option", "aria-selected": "{on}", "data-on": "{on}", "data-depth": "{s.depth}", onclick: move |_| sel.set(Some(id)),
                                            span { class: "oxor-l", if s.depth > 0 { span { class: "oxor-tree" } } {span_glyph(&s)} span { class: "oxv-ell", "{s.label}" } }
                                            span { class: "oxor-t",
                                                span { class: "oxor-b", "data-k": "{s.kind.key()}", style: "left:{x}%;width:{w}%" }
                                                span { class: "oxor-bl", "data-k": "{s.kind.key()}", style: "left:calc({x + w}% + 6px)", "{lab}" }
                                            }
                                            span { class: "oxscn-sr", ", {s.kind.label()}, {secs(s.dur)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}
