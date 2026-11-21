//! Screens S16 (Agent apps IV): **Halo Chat**, a desktop assistant app. Helpers and chats (pinned
//! first) in the sidebar; tool calls as cards in the chat (a web search with its sources, code it
//! ran with the output, a table); Tools, Mode and Model in the composer; a pane with Sources and
//! a Canvas draft; a mini chat over any app; and a research plan to edit before it starts. Rules
//! in `oxyn_core::kit::halochat`; styles in `halochat.css` (`oxs16c-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::kit::halochat::{
    HC_ANSWER, HC_APP, HC_BIKES, HC_CANVAS, HC_CODE, HC_HELPERS, HC_MODELS, HC_MODES, HC_PLAN,
    HC_PROMPT, HC_QUERY, HC_SHORTCUT, HC_SOURCES, HC_STATES, HC_TABS, HC_TOOLS, HaloChatData,
    ScreenState, add_chat, dollars, hc_split, plan_error, plan_minutes, prompt_error, toggle_pin,
    usd_per_km,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Halo Chat (for a host's `ScreenStage { states }`).
pub fn s16_halochat_states() -> Vec<(&'static str, &'static str)> {
    HC_STATES.to_vec()
}

fn hc_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            circle { cx: "10", cy: "10", r: "6.5", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M10 1.5v2.2M10 16.3v2.2M1.5 10h2.2M16.3 10h2.2", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round" }
        }
    }
}

/// A tool card: a header that opens and closes the body.
#[component]
fn ToolCard(
    glyph: &'static str,
    title: String,
    meta: String,
    open0: bool,
    children: Element,
) -> Element {
    let mut open = use_signal(move || open0);
    rsx! {
        div { class: "oxs16c-card", "data-open": "{open()}",
            button { r#type: "button", class: "oxs16c-chd", "aria-expanded": "{open()}", onclick: move |_| open.toggle(),
                span { class: "oxs16c-ci", {nico(glyph)} }
                b { class: "oxs10-ell", "{title}" }
                span { class: "oxs10-nl oxs16c-meta", "{meta}" }
                span { class: "oxs16c-car", {nico("caret")} }
            }
            if open() { div { class: "oxs16c-cbody", {children} } }
        }
    }
}

/// The pane: the sources the search read, and the canvas draft.
#[component]
fn Side(tab: Signal<usize>, loading: bool, empty: bool, said: Signal<String>) -> Element {
    let mut tab = tab;
    let mut said = said;
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("book")} p { "Sources and canvas drafts from your chats show here." } }
        };
    }
    rsx! {
        div { class: "oxs16c-tabs", role: "tablist", "aria-label": "Side panel",
            onkeydown: move |e: KeyboardEvent| { if matches!(e.key(), Key::ArrowRight | Key::ArrowLeft) { let now = *tab.peek(); tab.set(1 - now); } },
            for (i, (k, l)) in HC_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16c-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), "{l}"
                    if k == "sources" { span { class: "oxs16c-cnt", "{HC_SOURCES.len()}" } }
                }
            }
        }
        div { class: "oxs16c-body", role: "tabpanel", "aria-label": "{HC_TABS[t].1}",
            if t == 0 {
                ol { class: "oxs16c-srcs", "aria-label": "Sources",
                    for (i, (title, site, at)) in HC_SOURCES.iter().copied().enumerate() {
                        li { key: "{i}",
                            span { class: "oxs16c-sn", "{i + 1}" }
                            span { class: "oxs16c-st", b { "{title}" } span { class: "oxs10-ell", "{site} · {at}" } }
                        }
                    }
                }
            } else {
                article { class: "oxs16c-doc", "aria-label": "Canvas",
                    h2 { "{HC_CANVAS[0]}" }
                    for (i, p) in HC_CANVAS.iter().skip(1).copied().enumerate() { p { key: "{i}", "{p}" } }
                    div { class: "oxs16c-db",
                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| said.set("Canvas copied".to_string()), {nico("copy")} "Copy" }
                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| said.set("Shared a link to the canvas".to_string()), {nico("share")} "Share" }
                    }
                }
            }
        }
    }
}

#[component]
fn HaloChatApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<HaloChatData>,
) -> Element {
    let (st, own) = hc_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| HaloChatData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let d1 = d0.clone();
    let mut list = use_signal(move || d1.chats);
    let first = d0.chats.first().map(|c| c.id).unwrap_or(0);
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
    let pane = use_signal(|| true);
    let pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 1usize);
    let mut mode = use_signal(|| 1usize);
    let mut tool = use_signal(move || if own == "research" { 1usize } else { 0 });
    let mut helper = use_signal(|| None::<usize>);
    let mut mini = use_signal(move || own == "mini");
    let mut steps = use_signal(|| HC_PLAN.iter().map(|s| s.1).collect::<Vec<bool>>());
    let mut started = use_signal(|| false);
    let mut perr = use_signal(|| None::<&'static str>);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let rl = side() == "rail";
    let research = own == "research";
    let mut submit = move || {
        let p = text.peek().clone();
        match add_chat(&mut list.write(), &p) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set("New chat".to_string());
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let modes: Vec<(String, String, String)> = HC_MODES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let mut tools: Vec<(String, String, String)> = vec![(
        "No tool".to_string(),
        "Just chat".to_string(),
        String::new(),
    )];
    tools.extend(
        HC_TOOLS
            .iter()
            .map(|t| (t.1.to_string(), String::new(), String::new())),
    );
    let models = opts2(&HC_MODELS);
    let mk = HC_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let tg = if tool() == 0 {
        "fx"
    } else {
        HC_TOOLS[tool() - 1].2
    };
    let pinned: Vec<_> = all.iter().filter(|c| c.pinned).cloned().collect();
    let recent: Vec<_> = all.iter().filter(|c| !c.pinned).cloned().collect();

    let row = move |c: oxyn_core::kit::halochat::HcChat, oid: u32| {
        let id = c.id;
        let on = id == oid;
        let t2 = c.title.clone();
        rsx! {
            li { key: "{id}",
                button { r#type: "button", class: "oxs10-row oxs16c-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                    onclick: move |_| { open.set(id); mini.set(false); drawer.set(false); said.set(format!("Opened {t2}")); },
                    if c.pinned { span { class: "oxs16c-k", {nico("pin")} } }
                    span { class: "oxs10-nl", "{c.title}" }
                    span { class: "oxs10-ago oxs16c-day", "{c.at}" }
                }
            }
        }
    };

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{HC_APP}",
            {s11_top(HC_APP, hc_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New chat",
                onclick: move |_| { open.set(0); mini.set(false); drawer.set(false); said.set("New chat".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New chat" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                div { class: "oxs10-list",
                    p { class: "oxs10-sh", "Helpers" }
                    ul { class: "oxs10-ul",
                        for (i, (n, l, g)) in HC_HELPERS.iter().copied().enumerate() {
                            li { key: "{n}",
                                button { r#type: "button", class: "oxs10-row oxs16c-help", "aria-pressed": "{helper() == Some(i)}", title: "{l}",
                                    onclick: move |_| { let on = *helper.peek() == Some(i); helper.set(if on { None } else { Some(i) }); said.set(if on { "Helper off".to_string() } else { format!("Helper: {n}") }); },
                                    span { class: "oxs16c-hi", {nico(g)} } span { class: "oxs10-nl", "{n}" } }
                            }
                        }
                    }
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(6)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No chats yet. Ask anything to start one." } }
                    else {
                        if !pinned.is_empty() {
                            p { class: "oxs10-sh", "Pinned" }
                            ul { class: "oxs10-ul", for c in pinned { {row(c, oid)} } }
                        }
                        if !recent.is_empty() {
                            p { class: "oxs10-sh", "Recent" }
                            ul { class: "oxs10-ul", for c in recent { {row(c, oid)} } }
                        }
                    }
                }
            } else { span { class: "oxs10-grow" } }
            AcctFoot { rail: rl, said }
        }
    };

    let bar = rsx! {
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New chat" } }
        if let Some(h) = helper() { {pill(HC_HELPERS[h].0, "")} }
        span { class: "oxs10-grow" }
        if let Some(c) = &cur {
            button { r#type: "button", class: "oxscn-ib oxs16c-pinb", "aria-pressed": "{c.pinned}", "aria-label": if c.pinned { "Unpin chat" } else { "Pin chat" },
                onclick: move |_| { let id = *open.peek(); toggle_pin(&mut list.write(), id); said.set("Pin changed".to_string()); },
                {nico("pin")} }
        }
        button { r#type: "button", class: "oxscn-ib", "aria-label": "Mini chat ({HC_SHORTCUT})", "aria-pressed": "{mini()}", onclick: move |_| { mini.toggle(); said.set(if *mini.peek() { "Mini chat open".to_string() } else { "Mini chat closed".to_string() }); }, {nico("zoomout")} }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Ask anything",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add files or share a window", onclick: move |_| said.set("Add: files, photos, share a window".to_string()), {nico("plus")} }
                Pick { label: "Tools".to_string(), lead: rsx! { {nico(tg)} }, items: tools, cur: tool(), up: true,
                    on_pick: move |i: usize| { tool.set(i); said.set(if i == 0 { "No tool".to_string() } else { format!("Tool: {}", HC_TOOLS[i - 1].1) }); } }
                span { class: "oxs10-grow" }
                Pick { label: "Mode".to_string(), lead: rsx! { {nico("brain")} }, items: modes, cur: mode(), up: true, right: true,
                    on_pick: move |i: usize| { mode.set(i); said.set(format!("Mode: {}", HC_MODES[i].0)); } }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", HC_MODELS[i].0)); } }
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Dictate", onclick: move |_| said.set("Listening".to_string()), {nico("mic")} }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let on_steps = steps();
    let mins = plan_minutes(&on_steps);
    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if mini() {
        rsx! {
            div { class: "oxs10-feed oxs16c-desk", "aria-label": "Mini chat over your desktop",
                div { class: "oxs16c-mini", role: "dialog", "aria-label": "Mini chat",
                    div { class: "oxs16c-mh",
                        span { class: "oxs16c-mm", {hc_mark(16)} } b { "{HC_APP}" }
                        span { class: "oxs10-grow" }
                        span { class: "oxs10-kbd oxs16c-kb", "{HC_SHORTCUT}" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Open full chat", onclick: move |_| { mini.set(false); said.set("Full chat".to_string()); }, {nico("zoomin")} }
                    }
                    div { class: "oxs16c-mx",
                        p { class: "oxs16c-mq", "How do I say “see you on Thursday” in Portuguese?" }
                        p { class: "oxs16c-ma", "“Até quinta-feira.” Informal and friendly; add “então” at the end to mean “then”." }
                    }
                    div { class: "oxs16c-mi",
                        input { class: "oxs16c-min", r#type: "text", "aria-label": "Ask in the mini chat", placeholder: "Ask about anything on screen", value: "{text}",
                            oninput: move |e| { text.set(e.value()); err.set(None); },
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); mini.set(false); submit(); } }
                        button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| { mini.set(false); submit(); }, {nico("send")} }
                    }
                }
            }
        }
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{HC_PROMPT}" } else { "{c.title}" } } }
                    if sample && research {
                        div { class: "oxs16c-plan", role: "group", "aria-label": "Research plan",
                            div { class: "oxs16c-ph", {nico("scope")} b { "Research plan" } span { class: "oxs10-grow" }
                                if started() { {pill("Researching", "run")} } else { span { class: "oxs16c-meta", "About {mins} min" } } }
                            ul { class: "oxs16c-steps",
                                for (i, (l, _)) in HC_PLAN.iter().copied().enumerate() {
                                    li { key: "{i}",
                                        label { class: "oxs16c-step",
                                            input { r#type: "checkbox", checked: on_steps[i], disabled: started(), "aria-label": "{l}",
                                                onchange: move |_| { let mut w = steps.write(); w[i] = !w[i]; drop(w); perr.set(None); } }
                                            span { class: "oxs10-nl", "{l}" }
                                        }
                                    }
                                }
                            }
                            if let Some(e) = perr() { p { class: "oxs10-err", role: "alert", "{e}" } }
                            div { class: "oxs16c-pb",
                                if started() {
                                    span { class: "oxs16c-meta", "Reading sources · the report lands here in about {mins} min" }
                                } else {
                                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri",
                                        onclick: move |_| {
                                            match plan_error(&steps.peek()) {
                                                Some(e) => { perr.set(Some(e)); said.set(e.to_string()); }
                                                None => { started.set(true); said.set("Research started".to_string()); }
                                            }
                                        },
                                        "Start research" }
                                }
                            }
                        }
                    } else if sample {
                        ToolCard { glyph: "search", title: "Searched the web".to_string(), meta: format!("{} sources", HC_SOURCES.len()), open0: false,
                            p { class: "oxs16c-q", {nico("search")} span { class: "oxs10-nl", "{HC_QUERY}" } }
                            ul { class: "oxs16c-sl",
                                for (i, (title, site, _)) in HC_SOURCES.iter().copied().enumerate() {
                                    li { key: "{i}", span { class: "oxs16c-sn", "{i + 1}" } span { class: "oxs10-nl", "{title}" } span { class: "oxs16c-meta", "{site}" } }
                                }
                            }
                        }
                        ToolCard { glyph: "term", title: "Ran code".to_string(), meta: "Python · 0.4 s".to_string(), open0: true,
                            pre { class: "oxs16c-code", for (i, l) in HC_CODE.iter().copied().enumerate() { span { key: "{i}", "{l}" } } }
                            pre { class: "oxs16c-out", "aria-label": "Output",
                                for (i, (n, usd, km, _)) in HC_BIKES.iter().copied().enumerate() { span { key: "{i}", "{n}: {usd_per_km(usd, km)} per km of range" } }
                            }
                        }
                        ToolCard { glyph: "table", title: "Made a table".to_string(), meta: "3 bikes".to_string(), open0: true,
                            div { class: "oxs16c-tw",
                                table { class: "oxs16c-tbl",
                                    thead { tr { th { "Bike" } th { "Price" } th { "Range" } th { "Torque" } th { "$ per km" } } }
                                    tbody {
                                        for (n, usd, km, nm) in HC_BIKES.iter().copied() {
                                            tr { key: "{n}", td { "{n}" } td { "{dollars(usd)}" } td { "{km} km" } td { "{nm} N·m" } td { "{usd_per_km(usd, km)}" } }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "oxs10-agent", p { "{HC_ANSWER}" } }
                        div { class: "oxs16c-acts",
                            button { r#type: "button", class: "oxscn-ib", "aria-label": "Copy answer", onclick: move |_| said.set("Answer copied".to_string()), {nico("copy")} }
                            button { r#type: "button", class: "oxscn-ib", "aria-label": "Regenerate", onclick: move |_| said.set("Regenerating".to_string()), {nico("refresh")} }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { tab.set(1); said.set("Opened in Canvas".to_string()); }, {nico("draw")} "Open in Canvas" }
                        }
                    } else {
                        div { class: "oxs10-agent", p { "Here's a start. Ask a follow-up, or turn on a tool like Deep research for more." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {hc_mark(34)} }
                h1 { class: "oxs10-h1", "Hello, Rowan" }
                div { class: "oxs10-under",
                    for (i, (s, g)) in [("Plan a weekend in Porto", "globe"), ("Summarise a PDF", "pdf"), ("Help me write a toast", "pen")].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16c-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico(g)} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let panel = rsx! { Side { tab, loading, empty: all.is_empty() && !loading, said } };
    rsx! { Frame { app: "hc", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Sources".to_string() } }
}

/// Halo Chat: a desktop assistant app. Helpers and chats (Pinned, Recent) in the sidebar; tool
/// cards for a web search, code it ran and a table; Tools, Mode and Model in the composer; a
/// pane with Sources and Canvas; a mini chat (`mini`) and a research plan to edit and start
/// (`research`).
#[component]
pub fn HaloChatScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `mini` or `research`).
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
    /// Your data. `None` shows the sample for `state`; start from `HaloChatData::default()`.
    #[props(default)]
    data: Option<HaloChatData>,
) -> Element {
    rsx! { HaloChatApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
