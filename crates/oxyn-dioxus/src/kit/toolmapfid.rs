//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Tool Access Map**
//! ([`ToolReach`]). Behaviour and look only; the model is `oxyn_core::kit::toolmapfid`.
//! Styles: `toolmapfid.css` (classes `oxtm-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::toolmapfid::{
        TmAgent, TmCall, TmState, TmStatus, TmTool, tm_calls, tm_clock, tm_counts, tm_demo_agents,
        tm_demo_tools, tm_dur, tm_replay, tm_sides, tm_state, tm_summary,
    },
};

use crate::brand::BrandIcon;

/// How long a replay stays in flight in the demo, ms.
const REPLAY_MS: u64 = 1800;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// A tool's mark: its brand, or a glyph for the shell and web search.
fn mark(t: &TmTool, size: f32, coloured: bool) -> Element {
    if !t.brand.is_empty() {
        return rsx! { BrandIcon { name: t.brand.clone(), size, coloured, fallback: true } };
    }
    match t.id.as_str() {
        "shell" => icon("M3 4.5l3.5 3.5L3 11.5M8 12h5", size as u32),
        "search" => icon(
            "M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10zM10.6 10.6 14 14",
            size as u32,
        ),
        _ => rsx! { BrandIcon { name: t.name.clone(), size, coloured, fallback: true } },
    }
}

/// Replays finish after [`REPLAY_MS`]: shown as done with a made-up result from then on.
fn resolve(a: &TmAgent, starts: &[(String, ClockMs)], now: ClockMs) -> TmAgent {
    let mut a = a.clone();
    for (_, cs) in a.tools.iter_mut() {
        for c in cs.iter_mut() {
            if let Some((_, t0)) = starts.iter().find(|(id, _)| *id == c.id) {
                let el = now.since(*t0).as_millis() as u64;
                if el >= REPLAY_MS {
                    c.status = TmStatus::Done;
                    c.ms = 1400 + (c.input.len() as u32 % 9) * 700;
                    c.output = "{ \"ok\": true, \"replayed\": true }".into();
                }
            }
        }
    }
    a
}

fn state_line(st: TmState, last: Option<&TmCall>) -> (String, &'static str) {
    match st {
        TmState::NoAccess => ("No access".into(), "mute"),
        TmState::Unused => ("Not used yet".into(), "mute"),
        TmState::Running => ("Running".into(), "run"),
        TmState::Failed => ("Failed".into(), "fail"),
        TmState::Used => (
            format!("Last call {}", tm_dur(last.map(|c| c.ms).unwrap_or(0))),
            "mute",
        ),
    }
}

/// Tool Access Map (affectUI parity): an agent at the centre, the run's tools on both sides with a
/// line to each one the agent may use. Cards say whether a tool was used, is in use, failed or is
/// out of reach; open one to read its calls and replay one, as is or with a changed input.
#[component]
pub fn ToolReach(
    /// Every tool of the run, in layout order (the demo's when `None`).
    #[props(default)]
    tools: Option<Vec<TmTool>>,
    /// The agents (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<TmAgent>>,
    /// The agent shown first (the first one when `None`).
    #[props(default)]
    agent: Option<String>,
    /// A tool whose calls are open from the start.
    #[props(default)]
    open: Option<String>,
    /// The call picked in the calls view (the newest when `None`).
    #[props(default)]
    call: Option<String>,
    /// Start with the input being edited.
    #[props(default)]
    editing: bool,
    /// Heading.
    #[props(default = "Tool access".to_string())]
    title: String,
    /// The clock (replays settle with it).
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Another agent was picked (id).
    #[props(default)]
    on_agent_change: EventHandler<String>,
    /// A call was replayed: (agent, tool, input).
    #[props(default)]
    on_replay: EventHandler<(String, String, String)>,
) -> Element {
    let tools = tools.clone().unwrap_or_else(tm_demo_tools);
    let mut all = use_signal(|| agents.clone().unwrap_or_else(tm_demo_agents));
    let mut cur = use_signal(|| agent.clone());
    let mut opened = use_signal(|| open.clone());
    let mut picked = use_signal(|| call.clone());
    let mut edit = use_signal(move || editing);
    let mut draft = use_signal(String::new);
    let mut starts = use_signal(Vec::<(String, ClockMs)>::new);

    let list = all();
    let a_id = cur().unwrap_or_else(|| list.first().map(|a| a.id.clone()).unwrap_or_default());
    let Some(raw) = list.iter().find(|a| a.id == a_id).or(list.first()).cloned() else {
        return rsx! {};
    };
    let a = resolve(&raw, &starts(), now);
    let (reach, total, n_calls, n_run) = tm_counts(&a, &tools);
    let (left, right) = tm_sides(tools.len());
    let open_tool = opened().and_then(|id| tools.iter().find(|t| t.id == id).cloned());
    let view = if open_tool.is_some() { "calls" } else { "map" };
    let ids: Vec<String> = list.iter().map(|x| x.id.clone()).collect();

    let tool_card = |t: &TmTool, side: &'static str| {
        let st = tm_state(&a, &t.id);
        let calls = tm_calls(&a, &t.id).cloned().unwrap_or_default();
        let (line, tone) = state_line(st, calls.last());
        let n = calls.len();
        let tid = t.id.clone();
        let name = format!(
            "{}, {}{} – {}",
            t.name,
            match st {
                TmState::NoAccess => "no access",
                TmState::Unused => "not used yet",
                TmState::Used => "used",
                TmState::Running => "running",
                TmState::Failed => "last call failed",
            },
            if n > 0 {
                format!(", {n} calls")
            } else {
                String::new()
            },
            if st == TmState::NoAccess {
                "cannot be opened"
            } else {
                "open its calls"
            }
        );
        rsx! {
            button { class: "oxtm-tool", r#type: "button", key: "{t.id}", "data-k": "{t.id}", "data-st": st.id(), "data-side": side,
                disabled: st == TmState::NoAccess, "aria-label": "{name}",
                onclick: move |_| { opened.set(Some(tid.clone())); picked.set(None); edit.set(false); },
                i { class: "oxtm-wire", "aria-hidden": "true" }
                span { class: "oxtm-ti", {mark(t, 14.0, st != TmState::NoAccess)} }
                span { class: "oxtm-tx",
                    span { class: "oxtm-tn", b { "{t.name}" } if n > 0 { span { class: "oxtm-n", "{n}×" } } }
                    span { class: "oxtm-tl", "data-tone": tone,
                        if st == TmState::Running { i { class: "oxtm-pulse" } }
                        "{line}"
                    }
                }
            }
        }
    };

    let body = if let Some(t) = open_tool.clone() {
        let calls = tm_calls(&a, &t.id).cloned().unwrap_or_default();
        let sel_id = picked()
            .filter(|id| calls.iter().any(|c| &c.id == id))
            .or(calls.last().map(|c| c.id.clone()));
        let sel = calls
            .iter()
            .find(|c| Some(&c.id) == sel_id.as_ref())
            .cloned();
        let order: Vec<String> = calls.iter().rev().map(|c| c.id.clone()).collect();
        let back = move |_: MouseEvent| {
            opened.set(None);
            edit.set(false);
        };
        rsx! {
            div { class: "oxtm-calls", role: "region", "aria-label": "{t.name} calls",
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); opened.set(None); edit.set(false); },
                div { class: "oxtm-ch",
                    button { class: "oxtm-back", r#type: "button", "aria-label": "Back to the map", onclick: back, {icon("M10 3.5 5.5 8l4.5 4.5", 14)} }
                    span { class: "oxtm-ti", {mark(&t, 14.0, true)} }
                    b { "{t.name}" }
                    span { class: "oxtm-scope", "{t.scope}" }
                    span { class: "oxtm-sp" }
                    span { class: "oxtm-cnt", "{calls.len()} calls" }
                }
                div { class: "oxtm-cbody",
                    div { class: "oxtm-list", role: "listbox", "aria-label": "Calls", tabindex: "0",
                        onkeydown: {
                            let order = order.clone();
                            let cur_sel = sel_id.clone();
                            move |e: KeyboardEvent| {
                                let i = order.iter().position(|x| Some(x) == cur_sel.as_ref()).unwrap_or(0);
                                let j = match e.key() {
                                    Key::ArrowDown => Some((i + 1).min(order.len().saturating_sub(1))),
                                    Key::ArrowUp => Some(i.saturating_sub(1)),
                                    _ => None,
                                };
                                if let Some(j) = j { e.prevent_default(); if let Some(id) = order.get(j) { picked.set(Some(id.clone())); edit.set(false); } }
                            }
                        },
                        for c in calls.iter().rev() {
                            {
                                let cid = c.id.clone();
                                let on = Some(&c.id) == sel_id.as_ref();
                                rsx! {
                                    div { class: "oxtm-call", key: "{c.id}", "data-k": "{c.id}", role: "option", "aria-selected": on, "data-st": c.status.id(),
                                        onclick: move |_| { picked.set(Some(cid.clone())); edit.set(false); },
                                        span { class: "oxtm-cg", "aria-hidden": "true",
                                            match c.status {
                                                TmStatus::Running | TmStatus::Queued => rsx! { i { class: "oxtm-spin" } },
                                                TmStatus::Failed => icon("M4.5 4.5l7 7M11.5 4.5l-7 7", 11),
                                                _ => icon("M3.5 8.5l3 3 6-7", 11),
                                            }
                                        }
                                        span { class: "oxtm-cs", "{tm_summary(&c.input, 46)}" }
                                        if c.replay { span { class: "oxtm-rp", "Replay" } }
                                        span { class: "oxtm-cd", if c.status == TmStatus::Running { "…" } else { "{tm_dur(c.ms)}" } }
                                    }
                                }
                            }
                        }
                        if calls.is_empty() { p { class: "oxtm-none", "No calls yet." } }
                    }
                    if let Some(c) = sel.clone() {
                        {
                            let busy = c.status == TmStatus::Running;
                            let (aid, tid) = (a.id.clone(), t.id.clone());
                            let input0 = c.input.clone();
                            let run = move || {
                                let input = if edit() && !draft().is_empty() { draft() } else { input0.clone() };
                                let mut v = all();
                                let at = c.at.max(1) + 30;
                                if let Some(ag) = v.iter_mut().find(|x| x.id == aid)
                                    && let Some(id) = tm_replay(ag, &tid, &input, at)
                                {
                                    starts.write().push((id.clone(), now));
                                    picked.set(Some(id));
                                }
                                all.set(v);
                                edit.set(false);
                                on_replay.call((aid.clone(), tid.clone(), input));
                            };
                            let mut run2 = run.clone();
                            let mut run1 = run;
                            let start_edit = c.input.clone();
                            rsx! {
                                div { class: "oxtm-detail", key: "{c.id}", "data-k": "{c.id}",
                                    div { class: "oxtm-blk",
                                        span { class: "oxtm-bl", "Input" span { "{tm_clock(c.at)}" } }
                                        if edit() {
                                            textarea { class: "oxtm-code oxtm-ta", "aria-label": "Input", value: if draft().is_empty() { c.input.clone() } else { draft() }, spellcheck: "false",
                                                oninput: move |e| draft.set(e.value()),
                                                onkeydown: move |e: KeyboardEvent| {
                                                    if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) { e.prevent_default(); run2(); }
                                                    if e.key() == Key::Escape { e.stop_propagation(); edit.set(false); }
                                                }
                                            }
                                        } else {
                                            pre { class: "oxtm-code", "{c.input}" }
                                        }
                                    }
                                    div { class: "oxtm-blk",
                                        span { class: "oxtm-bl", "Output" if !busy && c.ms > 0 { span { "{tm_dur(c.ms)}" } } }
                                        if busy {
                                            p { class: "oxtm-code oxtm-wait", role: "status", i { class: "oxtm-spin" } "Waiting for the result" }
                                        } else if c.status == TmStatus::Failed {
                                            pre { class: "oxtm-code", "data-st": "failed", "{c.output}" }
                                        } else {
                                            pre { class: "oxtm-code", "{c.output}" }
                                        }
                                    }
                                    div { class: "oxtm-acts",
                                        if edit() {
                                            button { class: "oxtm-btn", r#type: "button", onclick: move |_| edit.set(false), "Cancel" }
                                        } else {
                                            button { class: "oxtm-btn", r#type: "button", disabled: busy,
                                                onclick: move |_| { draft.set(start_edit.clone()); edit.set(true); }, "Edit input" }
                                        }
                                        button { class: "oxtm-btn oxtm-pri", r#type: "button", disabled: busy, "aria-keyshortcuts": "Meta+Enter Control+Enter",
                                            onclick: move |_| run1(),
                                            {icon("M13 8a5 5 0 1 1-1.5-3.6M13 2.5V5h-2.5", 12)}
                                            if edit() { "Run changed input" } else { "Replay" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "oxtm-map", role: "tabpanel", "aria-label": "{a.name}",
                div { class: "oxtm-col", "data-side": "left", for t in tools[left].iter() { {tool_card(t, "left")} } }
                div { class: "oxtm-mid",
                    div { class: "oxtm-agent", key: "{a.id}", "data-k": "{a.id}",
                        span { class: "oxtm-ah",
                            BrandIcon { name: a.brand.clone(), size: 14.0, coloured: true, fallback: true }
                            b { "{a.name}" }
                        }
                        span { class: "oxtm-am", "{a.model}" }
                        span { class: "oxtm-badge", "data-st": a.status.id(), "{a.status.label()}" }
                        p { class: "oxtm-task", "{a.task}" }
                    }
                }
                div { class: "oxtm-col", "data-side": "right", for t in tools[right].iter() { {tool_card(t, "right")} } }
            }
        }
    };

    rsx! {
        section { class: "oxtm", "data-style": style_attr(&style), "data-view": view, "aria-label": "{title}",
            header { class: "oxtm-bar",
                span { class: "oxtm-logo", {icon("M2.5 8h3M10.5 8h3M8 2.5v3M8 10.5v3M6 6h4v4H6z", 14)} }
                h2 { class: "oxtm-ttl", "{title}" }
                span { class: "oxtm-sp" }
                if n_run > 0 { span { class: "oxtm-run", i { class: "oxtm-pulse" } "{n_run} running" } }
                span { class: "oxtm-cnt", role: "status", "{reach}/{total} tools · {n_calls} calls" }
            }
            div { class: "oxtm-tabs", role: "tablist", "aria-label": "Agents",
                onkeydown: {
                    let ids = ids.clone();
                    let a_id = a.id.clone();
                    move |e: KeyboardEvent| {
                        let i = ids.iter().position(|x| *x == a_id).unwrap_or(0);
                        let j = match e.key() {
                            Key::ArrowRight => Some((i + 1) % ids.len().max(1)),
                            Key::ArrowLeft => Some((i + ids.len().max(1) - 1) % ids.len().max(1)),
                            _ => None,
                        };
                        if let Some(j) = j { e.prevent_default(); cur.set(Some(ids[j].clone())); opened.set(None); on_agent_change.call(ids[j].clone()); }
                    }
                },
                for x in list.iter() {
                    {
                        let on = x.id == a.id;
                        let xid = x.id.clone();
                        rsx! {
                            button { class: "oxtm-tab", r#type: "button", role: "tab", key: "{x.id}", "data-k": "{x.id}", "aria-selected": on, tabindex: if on { "0" } else { "-1" },
                                onclick: move |_| { cur.set(Some(xid.clone())); opened.set(None); on_agent_change.call(xid.clone()); },
                                BrandIcon { name: x.brand.clone(), size: 12.0, coloured: on, fallback: true }
                                "{x.name}"
                            }
                        }
                    }
                }
            }
            {body}
        }
    }
}
