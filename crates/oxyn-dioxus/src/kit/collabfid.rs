//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32): the exact
//! **Crew chat** ([`CrewChat`]). Behaviour and look only; the model is
//! `oxyn_core::kit::collabfid`. Styles: `collabfid.css` (classes `oxcs-*`), in `PRIMITIVES_CSS`.
//! Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::collabfid::{
        AttKind, StreamAgent, StreamKind, StreamMessage, att_kind, stream_at_query, stream_demo,
        stream_demo_agents, stream_involves, stream_mark, stream_mentions, stream_name,
        stream_needs_you, stream_quote, stream_segments, stream_time,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// The 3×3 pixel mark of an agent (or `you`), in its colour slot.
fn pmark(agents: &[StreamAgent], id: &str, size: f32, twinkle: bool) -> Element {
    let (slot, pat) = stream_mark(agents, id);
    rsx! {
        svg { class: "oxcs-pm", "data-slot": "{slot}", "data-tw": "{twinkle}", width: "{size}", height: "{size}", view_box: "0 0 12 12", "aria-hidden": "true",
            for (i, on) in pat.iter().enumerate() {
                if *on { rect { key: "{i}", "data-k": "{i}", x: "{(i % 3) * 4}", y: "{(i / 3) * 4}", width: "3", height: "3", rx: ".5", style: "--i:{i}" } }
            }
        }
    }
}

fn att_icon(k: AttKind) -> Element {
    let d = match k {
        AttKind::Code => "M6 5 2.5 8 6 11M10 5l3.5 3L10 11",
        AttKind::Table => "M2.5 3.5h11v9h-11zM2.5 6.5h11M2.5 9.5h11M6.5 3.5v9",
        AttKind::Doc => "M4 2.5h5l3 3v8H4zM9 2.5v3h3M6 8.5h4M6 10.5h3",
        AttKind::Link => {
            "M7 9a2.5 2.5 0 0 0 3.5 0l2-2A2.5 2.5 0 0 0 9 3.5l-.7.7M9 7a2.5 2.5 0 0 0-3.5 0l-2 2A2.5 2.5 0 0 0 7 12.5l.7-.7"
        }
        AttKind::Folder => {
            "M2.5 4.5a1 1 0 0 1 1-1h3l1.5 1.5h4.5a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1z"
        }
        AttKind::Frame => "M2.5 3.5h11v9h-11zM2.5 10l3-3 3 3 2-2 3 3",
    };
    rsx! { svg { width: "13", height: "13", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.3", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } } }
}

fn rich(text: &str) -> Element {
    rsx! {
        for (i, (t, b)) in stream_segments(text).into_iter().enumerate() {
            if b.is_empty() {
                span { key: "{i}", "data-k": "{i}", "{t}" }
            } else {
                span { key: "{i}", "data-k": "{i}", class: "oxcs-logo", BrandIcon { name: b.to_string(), size: 13.0, coloured: true } "{t}" }
            }
        }
    }
}

/// Collaboration Stream (affectUI): what the agents of a run say to each other. Agent tabs on top
/// fade everyone else; every message shows who it is from and for, handoffs and questions for you
/// are tagged, answers quote what they answer, files are chips that open a preview, a picked
/// message drops its details in, and the composer names agents with @ or answers a question.
#[component]
pub fn CrewChat(
    /// The agents (the demo's when `None`).
    #[props(default)]
    agents: Option<Vec<StreamAgent>>,
    /// Messages in the order they were sent (the demo's when `None`; then sent ones are kept).
    #[props(default)]
    messages: Option<Vec<StreamMessage>>,
    /// The run is still going: a Live dot, and messages come in one by one (`step` ms apart).
    #[props(default)]
    live: bool,
    /// Gap between messages while live, ms.
    #[props(default = 1_400)]
    step: u64,
    /// A message picked from the start (its details open, the rest fade).
    #[props(default)]
    selected: String,
    /// An agent tab picked from the start.
    #[props(default)]
    focus_agent: String,
    /// An attachment preview open from the start: (message id, chip index).
    #[props(default)]
    preview: Option<(String, usize)>,
    /// The @ list open from the start.
    #[props(default)]
    mention_open: bool,
    /// Show the composer.
    #[props(default = true)]
    composer: bool,
    /// The stream scrolls past this height (px); 0 lets it grow.
    #[props(default)]
    max_height: u32,
    /// The composer's placeholder.
    #[props(default = "Write to the crew – type @ to pick an agent".to_string())]
    placeholder: String,
    /// Label of the stream for assistive tech.
    #[props(default = "Agent chat".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A message was picked (id) or let go (empty).
    #[props(default)]
    on_selected_change: EventHandler<String>,
    /// A tab was picked (agent id) or let go (empty).
    #[props(default)]
    on_focus_agent_change: EventHandler<String>,
    /// Sent: (text, to – agent ids or `all`, reply_to).
    #[props(default)]
    on_send: EventHandler<(String, Vec<String>, Option<String>)>,
) -> Element {
    let start = use_signal(move || now);
    let ags = agents.clone().unwrap_or_else(stream_demo_agents);
    let demo = messages.is_none();
    let mut sel = use_signal(move || (!selected.is_empty()).then(|| selected.clone()));
    let mut fag = use_signal(move || (!focus_agent.is_empty()).then(|| focus_agent.clone()));
    let mut pv = use_signal(move || preview.clone());
    let mut menu = use_signal(move || mention_open);
    let mut mi = use_signal(|| 0usize);
    let mut draft = use_signal(String::new);
    let mut named = use_signal(Vec::<String>::new);
    let mut reply = use_signal(|| None::<String>);
    let mut sent = use_signal(Vec::<StreamMessage>::new);
    let mut cur = use_signal(|| 0usize);
    let mut rows = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let mut field = use_signal(|| None::<Rc<MountedData>>);
    let mut chip_back = use_signal(|| None::<Rc<MountedData>>);
    let mut chips = use_signal(Vec::<(String, usize, Rc<MountedData>)>::new);
    let mut tabrefs = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let mut tabcur = use_signal(|| 0usize);

    let el = now.since(start()).as_millis() as u64;
    let mut all = messages.clone().unwrap_or_else(stream_demo);
    if demo {
        all.extend(sent());
    }
    // While live, message i arrives at i·step and is "Writing…" for the 900 ms before.
    let total = all.len();
    let base_n = if demo {
        total - sent.read().len()
    } else {
        total
    };
    let mut list: Vec<StreamMessage> = vec![];
    for (i, m) in all.iter().enumerate() {
        if !live || i >= base_n {
            list.push(m.clone());
            continue;
        }
        let at = i as u64 * step;
        if el >= at {
            list.push(m.clone());
        } else if el + 900 >= at && m.kind != StreamKind::Spawn {
            let mut t = m.clone();
            t.typing = true;
            list.push(t);
            break;
        } else {
            break;
        }
    }
    let picked = sel();
    let agent = fag();
    let open_pv = pv().and_then(|(mid, ci)| {
        list.iter()
            .find(|m| m.id == mid)
            .and_then(|m| m.attachments.get(ci).cloned().map(|a| (m.clone(), a)))
    });
    let q = stream_at_query(&draft()).filter(|_| menu() || draft().contains('@'));
    let menu_on = menu() || q.is_some();
    let opts: Vec<StreamAgent> = stream_mentions(&ags, q.as_deref().unwrap_or(""))
        .into_iter()
        .cloned()
        .collect();
    let to_label = match (reply(), named().is_empty()) {
        (Some(r), _) => {
            let who = list
                .iter()
                .find(|m| m.id == r)
                .map_or_else(String::new, |m| stream_name(&ags, &m.from));
            format!("reply to {who}")
        }
        (None, true) => "to everyone".to_string(),
        (None, false) => format!(
            "to {}",
            named()
                .iter()
                .map(|n| stream_name(&ags, n))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    let mut tabs: Vec<(String, String)> = vec![
        (String::new(), "Everyone".into()),
        ("you".into(), "You".into()),
    ];
    tabs.extend(ags.iter().map(|a| (a.id.clone(), a.name.clone())));
    let n = list.len();
    let list_send = list.clone();
    let send = use_callback(move |_: ()| {
        let t = draft().trim().to_string();
        if t.is_empty() {
            return;
        }
        let to: Vec<String> = if let Some(r) = reply() {
            list_send
                .iter()
                .find(|m| m.id == r)
                .map_or_else(|| vec!["all".to_string()], |m| vec![m.from.clone()])
        } else if named().is_empty() {
            vec!["all".into()]
        } else {
            named()
        };
        let rt = reply();
        on_send.call((t.clone(), to.clone(), rt.clone()));
        if demo {
            let k = sent.read().len();
            let at = 14 * 60 + 13 + k as u32;
            sent.write().push(StreamMessage {
                id: format!("you-{k}"),
                from: "you".into(),
                to: to.first().cloned().unwrap_or_else(|| "all".into()),
                kind: StreamKind::Reply,
                text: t,
                at,
                reply_to: rt,
                typing: false,
                attachments: vec![],
                details: vec![],
            });
        }
        draft.set(String::new());
        named.set(vec![]);
        reply.set(None);
        menu.set(false);
    });
    let list_for_to = list.clone();

    rsx! {
        div { class: "oxcs", "data-style": style_attr(&style), "data-faded": "{picked.is_some() || agent.is_some()}",
            div { class: "oxcs-bar",
                div { class: "oxcs-tabs", role: "toolbar", "aria-label": "Agents",
                    onkeydown: move |e: KeyboardEvent| {
                        let len = tabrefs.read().len();
                        if len == 0 { return; }
                        let next = match e.key() {
                            Key::ArrowRight => Some(1i32),
                            Key::ArrowLeft => Some(-1),
                            Key::Home => Some(-1000),
                            Key::End => Some(1000),
                            _ => None,
                        };
                        if let Some(d) = next {
                            e.prevent_default();
                            let c = tabcur() as i32;
                            let i = (c + d).clamp(0, len as i32 - 1) as usize;
                            tabcur.set(i);
                            focus(tabrefs.read().get(i).cloned().flatten());
                        }
                    },
                    for (ti, (id, name)) in tabs.iter().cloned().enumerate() {
                        {
                            let on = agent.clone().unwrap_or_default() == id;
                            let idc = id.clone();
                            rsx! {
                                button { key: "{ti}", "data-k": "{ti}", class: "oxcs-tab", r#type: "button", "aria-pressed": "{on}",
                                    tabindex: if tabcur() == ti { "0" } else { "-1" },
                                    onmounted: move |e| { let mut r = tabrefs.write(); if r.len() <= ti { r.resize(ti + 1, None) } r[ti] = Some(e.data()) },
                                    onfocus: move |_| tabcur.set(ti),
                                    onclick: move |_| {
                                        let next = if idc.is_empty() || fag().as_deref() == Some(idc.as_str()) { None } else { Some(idc.clone()) };
                                        fag.set(next.clone());
                                        on_focus_agent_change.call(next.unwrap_or_default());
                                    },
                                    if !id.is_empty() { {pmark(&ags, &id, 12.0, false)} }
                                    "{name}" }
                            }
                        }
                    }
                }
                if live { span { class: "oxcs-live", i {} "Live" } }
            }
            ol { class: "oxcs-log", role: "log", "aria-label": "{title}", "aria-live": if live { "polite" } else { "off" },
                style: if max_height > 0 { format!("max-height:{max_height}px;overflow:auto") } else { String::new() },
                onkeydown: move |e: KeyboardEvent| {
                    let c = cur();
                    let next = match e.key() {
                        Key::ArrowDown => Some((c + 1).min(n.saturating_sub(1))),
                        Key::ArrowUp => Some(c.saturating_sub(1)),
                        Key::Home => Some(0),
                        Key::End => Some(n.saturating_sub(1)),
                        Key::Escape => { sel.set(None); fag.set(None); on_selected_change.call(String::new()); None }
                        _ => None,
                    };
                    if let Some(i) = next { e.prevent_default(); cur.set(i); focus(rows.read().get(i).cloned().flatten()); }
                },
                for (i, m) in list.iter().cloned().enumerate() {
                    {
                        let dim = agent.as_ref().is_some_and(|a| !stream_involves(&m, a)) || picked.as_ref().is_some_and(|p| p != &m.id);
                        let is_sel = picked.as_deref() == Some(m.id.as_str());
                        let mid = m.id.clone();
                        let mid2 = m.id.clone();
                        let label = format!("{} to {}, {}: {}", stream_name(&ags, &m.from), stream_name(&ags, &m.to), stream_time(m.at), m.text);
                        if m.kind == StreamKind::Spawn {
                            rsx! {
                                li { key: "{m.id}", "data-k": "{m.id}", class: "oxcs-spawn", "data-dim": "{dim}", tabindex: if cur() == i { "0" } else { "-1" },
                                    onmounted: move |e| { let mut r = rows.write(); if r.len() <= i { r.resize(i + 1, None) } r[i] = Some(e.data()) },
                                    onfocus: move |_| cur.set(i),
                                    "{m.text}" }
                            }
                        } else {
                            let needs = stream_needs_you(&list, &m);
                            let quote = stream_quote(&list, i).cloned();
                            let show_reply = needs || (is_sel && m.from != "you");
                            rsx! {
                                li { key: "{m.id}", "data-k": "{m.id}", class: "oxcs-msg", "data-dim": "{dim}", "data-sel": "{is_sel}", "data-you": "{m.from == \"you\"}",
                                    tabindex: if cur() == i { "0" } else { "-1" }, "aria-label": "{label}",
                                    onmounted: move |e| { let mut r = rows.write(); if r.len() <= i { r.resize(i + 1, None) } r[i] = Some(e.data()) },
                                    onfocus: move |_| cur.set(i),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                                        e.prevent_default();
                                        let next = if sel().as_deref() == Some(mid.as_str()) { None } else { Some(mid.clone()) };
                                        sel.set(next.clone()); on_selected_change.call(next.unwrap_or_default());
                                    },
                                    span { class: "oxcs-mcol", {pmark(&ags, &m.from, 12.0, m.typing)} }
                                    div { class: "oxcs-col",
                                        div { class: "oxcs-bub",
                                            onclick: move |_| {
                                                let next = if sel().as_deref() == Some(mid2.as_str()) { None } else { Some(mid2.clone()) };
                                                sel.set(next.clone()); cur.set(i); on_selected_change.call(next.unwrap_or_default());
                                            },
                                            div { class: "oxcs-h",
                                                b { class: "oxcs-name", "{stream_name(&ags, &m.from)}" }
                                                if m.kind == StreamKind::Handoff { span { class: "oxcs-tag", "data-k": "handoff", "Handoff" } }
                                                if needs { span { class: "oxcs-tag", "data-k": "needs", "Needs you" } }
                                                if show_reply && composer {
                                                    {
                                                        let rid = m.id.clone();
                                                        rsx! { button { class: "oxcs-reply", r#type: "button", tabindex: if cur() == i { "0" } else { "-1" },
                                                            onclick: move |e| { e.stop_propagation(); reply.set(Some(rid.clone())); named.set(vec![]); focus(field()) }, "Reply" } }
                                                    }
                                                }
                                                span { class: "oxcs-sp" }
                                                time { class: "oxcs-tm", "{stream_time(m.at)}" }
                                            }
                                            if let Some(qm) = quote {
                                                div { class: "oxcs-q",
                                                    span { class: "oxcs-qh", "to " {pmark(&ags, &m.to, 10.0, false)} b { "{stream_name(&ags, &m.to)}" } }
                                                    span { class: "oxcs-qt", "{qm.text}" }
                                                }
                                            } else {
                                                span { class: "oxcs-to", "to " if m.to != "all" { {pmark(&ags, &m.to, 10.0, false)} } b { "{stream_name(&ags, &m.to)}" } }
                                            }
                                            if m.typing {
                                                p { class: "oxcs-txt oxcs-writing", "Writing…" }
                                            } else {
                                                p { class: "oxcs-txt", {rich(&m.text)} }
                                            }
                                            if !m.attachments.is_empty() && !m.typing {
                                                div { class: "oxcs-atts",
                                                    for (ci, a) in m.attachments.iter().cloned().enumerate() {
                                                        {
                                                            let pid = m.id.clone();
                                                            let pid2 = m.id.clone();
                                                            rsx! {
                                                                button { key: "{ci}", "data-k": "{ci}", class: "oxcs-chip", r#type: "button", tabindex: if cur() == i { "0" } else { "-1" },
                                                                    "aria-haspopup": "dialog",
                                                                    onclick: move |e| {
                                                                        e.stop_propagation();
                                                                        let back = chips.read().iter().find(|c| c.0 == pid && c.1 == ci).map(|c| c.2.clone());
                                                                        chip_back.set(back);
                                                                        pv.set(Some((pid.clone(), ci)));
                                                                    },
                                                                    onmounted: move |e| {
                                                                        let d = e.data();
                                                                        if pv().is_some_and(|(p, c)| c == ci && p == pid2) { chip_back.set(Some(d.clone())) }
                                                                        chips.write().push((pid2.clone(), ci, d));
                                                                    },
                                                                    {att_icon(att_kind(&a.label))} "{a.label}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if is_sel && !m.details.is_empty() {
                                            dl { class: "oxcs-det",
                                                for (k, (a, b)) in m.details.iter().cloned().enumerate() {
                                                    div { key: "{k}", "data-k": "{k}", dt { "{a}" } dd { "{b}" } }
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
            if composer {
                div { class: "oxcs-comp",
                    if menu_on && !opts.is_empty() {
                        ul { class: "oxcs-menu", role: "listbox", id: "oxcs-at", "aria-label": "Agents",
                            for (k, a) in opts.iter().cloned().enumerate() {
                                {
                                    let aid = a.id.clone();
                                    rsx! {
                                        li { key: "{a.id}", "data-k": "{a.id}", class: "oxcs-opt", role: "option", "aria-selected": "{k == mi().min(opts.len() - 1)}",
                                            onmousedown: move |e| e.prevent_default(),
                                            onclick: move |_| {
                                                let d = draft();
                                                let cut = d.rfind('@').map_or(d.clone(), |i| d[..i].to_string());
                                                draft.set(cut);
                                                if !named().contains(&aid) { named.write().push(aid.clone()); }
                                                menu.set(false); mi.set(0); focus(field());
                                            },
                                            {pmark(&ags, &a.id, 12.0, false)} b { "{a.name}" } span { class: "oxcs-sp" } span { class: "oxcs-model", "{a.model}" } }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(r) = reply().and_then(|r| list_for_to.iter().find(|m| m.id == r).cloned()) {
                        div { class: "oxcs-rq",
                            span { class: "oxcs-rqt", "Replying to " b { "{stream_name(&ags, &r.from)}" } ": {r.text}" }
                            button { class: "oxcs-x", r#type: "button", "aria-label": "Drop the reply", onclick: move |_| reply.set(None), "×" }
                        }
                    }
                    textarea { class: "oxcs-field", rows: "2", placeholder: "{placeholder}", value: "{draft}", "aria-label": "Message",
                        "aria-expanded": "{menu_on}", "aria-controls": "oxcs-at",
                        onmounted: move |e| field.set(Some(e.data())),
                        oninput: move |e| { draft.set(e.value()); mi.set(0); },
                        onkeydown: move |e: KeyboardEvent| {
                            let olen = opts.len();
                            match e.key() {
                                Key::ArrowDown if menu_on && olen > 0 => { e.prevent_default(); mi.set((mi() + 1) % olen); }
                                Key::ArrowUp if menu_on && olen > 0 => { e.prevent_default(); mi.set((mi() + olen - 1) % olen); }
                                Key::Enter | Key::Tab if menu_on && olen > 0 => {
                                    e.prevent_default();
                                    let a = opts[mi().min(olen - 1)].id.clone();
                                    let d = draft();
                                    let cut = d.rfind('@').map_or(d.clone(), |i| d[..i].to_string());
                                    draft.set(cut);
                                    if !named().contains(&a) { named.write().push(a); }
                                    menu.set(false); mi.set(0);
                                }
                                Key::Enter if !e.modifiers().shift() => { e.prevent_default(); send.call(()); }
                                Key::Escape => { menu.set(false); reply.set(None); }
                                _ => {}
                            }
                        },
                    }
                    div { class: "oxcs-cf",
                        button { class: "oxcs-at", r#type: "button", "aria-label": "Name an agent", "aria-expanded": "{menu_on}", onclick: move |_| { menu.set(!menu()); mi.set(0); }, "@" }
                        span { class: "oxcs-tol", "{to_label}" }
                        span { class: "oxcs-sp" }
                        button { class: "oxcs-send", r#type: "button", "aria-label": "Send", disabled: draft().trim().is_empty(), onclick: move |_| send.call(()),
                            svg { width: "13", height: "13", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "M8 13V3.5M4 7l4-4 4 4" } } }
                    }
                }
            }
            if let Some((m, a)) = open_pv {
                div { class: "oxcs-scrim", onclick: move |_| { pv.set(None); focus(chip_back()); } }
                div { class: "oxcs-pv", role: "dialog", "aria-modal": "true", "aria-label": "{a.label}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { pv.set(None); focus(chip_back()); },
                    div { class: "oxcs-pvh",
                        span { class: "oxcs-pvi", {att_icon(att_kind(&a.label))} }
                        div { class: "oxcs-pvt", b { "{a.label}" } span { "{stream_name(&ags, &m.from)} · {stream_time(m.at)} · {a.meta}" } }
                        button { class: "oxcs-x", r#type: "button", "aria-label": "Close", onmounted: move |e| focus(Some(e.data())), onclick: move |_| { pv.set(None); focus(chip_back()); }, "×" }
                    }
                    pre { class: "oxcs-pre", for (k, l) in a.preview.iter().cloned().enumerate() { span { key: "{k}", "data-k": "{k}", "{l}\n" } } }
                }
            }
        }
    }
}
