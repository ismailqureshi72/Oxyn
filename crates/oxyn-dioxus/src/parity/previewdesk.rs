//! Screens S16 (Agent apps IV): **Preview Desk**, a desktop app that shows a coding agent's work
//! as previews. Sessions on machines of their own; the session's chat and steps; preview tabs
//! beside it (the live site, a doc, the diff); Design mode, where you pick an element on the live
//! site and comment on it; an inline diff where a line takes a comment. Machine, folder and model
//! in the composer. Rules in `oxyn_core::parity::previewdesk`; styles in `previewdesk.css`
//! (`oxs16p-`), on the S10 app frame.

use dioxus::prelude::*;
use oxyn_core::parity::previewdesk::{
    PD_APP, PD_DIFF, PD_DOC, PD_DONE, PD_ELEMENTS, PD_MACHINES, PD_MODELS, PD_PLANS, PD_PROMPT,
    PD_STATES, PD_STEPS, PD_TABS, PdNote, PreviewDeskData, ScreenState, add_note, add_session,
    pd_notes, pd_split, price, prompt_error,
};

use super::cmdstudio::{Frame, Pick, opts2, pill};
use super::fleetwin::{AcctFoot, s11_top};
use super::screensfid::{nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of Preview Desk (for a host's `ScreenStage { states }`).
pub fn s16_previewdesk_states() -> Vec<(&'static str, &'static str)> {
    PD_STATES.to_vec()
}

fn pd_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "2", y: "3.5", width: "16", height: "13", rx: "2.5", fill: "none", stroke: "currentColor", stroke_width: "1.7" }
            path { d: "M2 7.5h16", stroke: "currentColor", stroke_width: "1.6" }
            path { d: "M11 10.5l4 2-1.8.6-.6 1.8z", fill: "currentColor", stroke: "currentColor", stroke_width: "1", stroke_linejoin: "round" }
        }
    }
}

/// The preview pane: the live site with Design mode, a doc and the diff with line comments.
#[component]
fn Previews(
    tab: Signal<usize>,
    own: String,
    loading: bool,
    empty: bool,
    notes: Signal<Vec<PdNote>>,
    said: Signal<String>,
) -> Element {
    let mut tab = tab;
    let mut said = said;
    let mut notes = notes;
    let own2 = own.clone();
    let mut design = use_signal(move || own2 == "design");
    let own3 = own.clone();
    let mut pick = use_signal(move || if own3 == "design" { Some(2usize) } else { None });
    let mut line = use_signal(|| None::<usize>);
    let mut draft = use_signal(String::new);
    let mut nerr = use_signal(|| None::<&'static str>);
    let mut yearly = use_signal(|| false);
    let mut approved = use_signal(|| false);
    let t = tab();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if empty {
        return rsx! {
            div { class: "oxs10-pempty", {nico("eye")} p { "Previews of what the agent makes show up here: the running site, documents and the diff." } }
        };
    }
    let mut send = move |on: String| {
        let tx = draft.peek().clone();
        match add_note(&mut notes.write(), &on, &tx) {
            Ok(n) => {
                draft.set(String::new());
                nerr.set(None);
                pick.set(None);
                line.set(None);
                said.set(format!("Comment sent to the agent ({n} open)"));
            }
            Err(e) => {
                nerr.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let all = notes();
    let y = yearly();
    let dz = design();
    let pk = pick();
    let note_box = move |on: String| {
        let on2 = on.clone();
        rsx! {
            div { class: "oxs16p-nbox", role: "group", "aria-label": "Comment on {on}",
                p { class: "oxs16p-non", {nico("pen")} span { class: "oxs10-ell", "{on}" } }
                textarea { class: "oxs16p-nta", rows: "2", "aria-label": "Comment for the agent", placeholder: "Tell the agent what to change", value: "{draft}",
                    "aria-invalid": "{nerr().is_some()}",
                    oninput: move |e| { draft.set(e.value()); nerr.set(None); },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send(on.clone()); } }
                if let Some(e) = nerr() { p { class: "oxs10-err", role: "alert", "{e}" } }
                div { class: "oxs16p-nact",
                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { pick.set(None); line.set(None); draft.set(String::new()); nerr.set(None); }, "Cancel" }
                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| send(on2.clone()), "Send to agent" }
                }
            }
        }
    };
    rsx! {
        div { class: "oxs16p-tabs", role: "tablist", "aria-label": "Previews",
            onkeydown: move |e: KeyboardEvent| { let now = *tab.peek(); match e.key() { Key::ArrowRight => tab.set((now + 1) % 3), Key::ArrowLeft => tab.set((now + 2) % 3), _ => {} } },
            for (i, (k, l, g)) in PD_TABS.iter().copied().enumerate() {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs16p-tab", "aria-selected": "{i == t}", tabindex: if i == t { "0" } else { "-1" },
                    onclick: move |_| tab.set(i), {nico(g)} span { class: "oxs10-ell", "{l}" } }
            }
        }
        div { class: "oxs16p-pb", role: "tabpanel", "aria-label": "{PD_TABS[t].1}",
            match PD_TABS[t].0 {
                "site" => rsx! {
                    div { class: "oxs16p-tool",
                        span { class: "oxs16p-url", {nico("lock")} span { class: "oxs10-ell", "localhost:3000/pricing" } }
                        button { r#type: "button", class: "oxs16p-dz", "aria-pressed": "{dz}", onclick: move |_| { let on = !*design.peek(); design.set(on); pick.set(None); said.set(if on { "Design mode on: pick an element" } else { "Design mode off" }.to_string()); },
                            {nico("finger")} span { "Design mode" } }
                    }
                    div { class: "oxs16p-site", "data-design": "{dz}",
                        {
                            let el = move |i: usize| if dz { if pk == Some(i) { "picked" } else { "pickable" } } else { "" };
                            let mut choose = move |i: usize| { if *design.peek() { pick.set(Some(i)); draft.set(String::new()); said.set(format!("Picked {}", PD_ELEMENTS[i].1)); } };
                            rsx! {
                                h2 { class: "oxs16p-h", "data-el": "{el(0)}", onclick: move |_| choose(0), "Plans that grow with you" }
                                div { class: "oxs16p-sw", "data-el": "{el(1)}", onclick: move |_| choose(1),
                                    button { r#type: "button", "aria-pressed": "{!y}", disabled: dz, onclick: move |_| yearly.set(false), "Monthly" }
                                    button { r#type: "button", "aria-pressed": "{y}", disabled: dz, onclick: move |_| yearly.set(true), "Yearly" }
                                }
                                div { class: "oxs16p-plans",
                                    for (i, (n, _, _)) in PD_PLANS.iter().copied().enumerate() {
                                        div { key: "{n}", class: "oxs16p-plan", "data-feat": "{i == 1}", "data-el": if i == 1 { el(2) } else { "" },
                                            onclick: move |_| if i == 1 { choose(2) },
                                            if i == 1 { span { class: "oxs16p-badge", "Most popular" } }
                                            b { "{n}" }
                                            p { class: "oxs16p-price", "${price(i, y)}" span { "/mo" } }
                                            span { class: "oxs16p-cta", "data-el": if i == 1 { el(3) } else { "" }, onclick: move |e| if i == 1 && *design.peek() { e.stop_propagation(); choose(3) }, "Start trial" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if dz {
                        if let Some(i) = pk { {note_box(PD_ELEMENTS[i].1.to_string())} } else { p { class: "oxs16p-hint", {nico("finger")} span { "Pick an element on the page to comment on it." } } }
                    }
                },
                "doc" => rsx! {
                    div { class: "oxs16p-doc",
                        h3 { "{PD_DOC.0}" }
                        for (i, l) in PD_DOC.1.iter().copied().enumerate() { p { key: "{i}", "{l}" } }
                    }
                },
                _ => rsx! {
                    div { class: "oxs16p-dh", {nico("file")} span { class: "oxs10-nl oxs16p-mono", "components/PlanCard.tsx" } span { class: "oxs10-ago", "Click a line to comment" } }
                    div { class: "oxs16p-diff", role: "list", "aria-label": "Diff of components/PlanCard.tsx",
                        for (i, (s, n, l)) in PD_DIFF.iter().copied().enumerate() {
                            {
                                let on = if s == '-' { format!("PlanCard.tsx:{n} (removed)") } else { format!("PlanCard.tsx:{n}") };
                                let kind = match s { '+' => "Added line", '-' => "Removed line", _ => "Line" };
                                let here: Vec<PdNote> = all.iter().filter(|x| x.on == on).cloned().collect();
                                let on3 = on.clone();
                                rsx! {
                                    div { key: "{i}", role: "listitem",
                                        button { r#type: "button", class: "oxs16p-dl", "data-s": "{s}", "data-on": "{line() == Some(i)}", "aria-label": "{kind} {n}: {l}. Comment",
                                            onclick: move |_| { line.set(Some(i)); draft.set(String::new()); said.set(format!("Comment on line {n}")); },
                                            span { class: "oxs16p-ln", "{n}" } span { class: "oxs16p-sg", "{s}" } span { class: "oxs16p-lt", "{l}" } }
                                        for (j, x) in here.iter().enumerate() {
                                            div { key: "{j}", class: "oxs16p-cmt", span { class: "oxs10-av", "aria-hidden": "true", "RV" } p { "{x.text}" } }
                                        }
                                        if line() == Some(i) { {note_box(on3)} }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxs16p-ra",
                        if approved() { span { class: "oxs16p-ok", {nico("check")} "Approved" } }
                        else { button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs11-pri", onclick: move |_| { approved.set(true); said.set("Changes approved".to_string()); }, "Approve changes" } }
                    }
                },
            }
            if !all.is_empty() && t != 2 {
                div { class: "oxs16p-notes",
                    p { class: "oxs10-sh", "Comments for the agent · {all.len()}" }
                    for (i, x) in all.iter().enumerate() {
                        p { key: "{i}", class: "oxs16p-note", span { class: "oxs16p-on", "{x.on}" } span { "{x.text}" } }
                    }
                }
            }
        }
    }
}

#[component]
fn PreviewDeskApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
    data: Option<PreviewDeskData>,
) -> Element {
    let (st, own) = pd_split(&state);
    let d0 = use_hook(move || data.unwrap_or_else(|| PreviewDeskData::for_state(st)));
    let loading = st == ScreenState::Loading;
    let finished0 = !own.is_empty();
    let d1 = d0.clone();
    let mut list = use_signal(move || {
        let mut l = d1.sessions;
        if finished0 && let Some(c) = l.first_mut() {
            c.running = false;
        }
        l
    });
    let first = d0.sessions.first().map(|t| t.id).unwrap_or(0);
    let mut open = use_signal(move || first);
    let ptab = use_signal(move || if own == "diff" { 2 } else { 0 });
    let notes = use_signal(move || pd_notes(own));
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
    let pnar = use_signal(move || finished0);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let mut machine = use_signal(|| 1usize);
    let all = list();
    let oid = open();
    let cur = all.iter().find(|c| c.id == oid).cloned();
    let running = cur.as_ref().is_some_and(|c| c.running);
    let rl = side() == "rail";
    let nn = notes().len() as u32;
    let mut submit = move || {
        let p = text.peek().clone();
        let m = *machine.peek();
        match add_session(&mut list.write(), &p, m) {
            Ok(id) => {
                open.set(id);
                text.set(String::new());
                err.set(None);
                said.set(format!("Session started on {}", PD_MACHINES[m].0));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let machines: Vec<(String, String, String)> = PD_MACHINES
        .iter()
        .map(|m| (m.0.to_string(), m.1.to_string(), String::new()))
        .collect();
    let models = opts2(&PD_MODELS);
    let mk = PD_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let folder = d0.folder.clone();

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{PD_APP}",
            {s11_top(PD_APP, pd_mark(18), rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" }, title: "New session",
                onclick: move |_| { open.set(0); drawer.set(false); said.set("New session".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New session" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                p { class: "oxs10-sh", "Sessions" }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(5)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No sessions yet. Describe a change to start one." } }
                    else {
                        ul { class: "oxs10-ul",
                            for c in all.iter().cloned() {
                                {
                                    let id = c.id;
                                    let on = id == oid;
                                    let t2 = c.title.clone();
                                    let n = if id == 1 { c.notes + nn } else { c.notes };
                                    let (mn, _) = PD_MACHINES[c.machine];
                                    rsx! {
                                        li { key: "{id}",
                                            button { r#type: "button", class: "oxs16p-row", "data-on": "{on}", "aria-current": if on { "page" } else { "false" },
                                                onclick: move |_| { open.set(id); drawer.set(false); said.set(format!("Opened {t2}")); },
                                                if c.running { span { class: "oxs10-spin oxs10-spin-s" } } else { span { class: "oxs16p-k", {nico("chat")} } }
                                                span { class: "oxs16p-rt", b { class: "oxs10-ell", "{c.title}" }
                                                    span { class: "oxs16p-rs", if c.machine == 0 { {nico("laptop")} } else { {nico("cloud")} } span { class: "oxs10-ell", "{mn}" } } }
                                                if n > 0 { span { class: "oxs16p-nb", title: "Comments for the agent", "{n}" } }
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
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(c) = &cur { "{c.title}" } else { "New session" } }
        if let Some(c) = &cur { if c.running { {pill("Working", "run")} } else { {pill("Ready to review", "ok")} } }
        span { class: "oxs10-grow" }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", placeholder: "Describe a change, or reply to the agent",
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                Pick { label: "Machine".to_string(), lead: rsx! { if machine() == 0 { {nico("laptop")} } else { {nico("cloud")} } }, items: machines, cur: machine(), up: true,
                    on_pick: move |i: usize| { machine.set(i); said.set(format!("Machine: {}", PD_MACHINES[i].0)); } }
                button { r#type: "button", class: "oxs10-chip oxs16p-fold", "aria-label": "Working folder: {folder}", onclick: move |_| said.set("Choose a working folder".to_string()),
                    {nico("folder")} span { class: "oxs10-ell", "{folder}" } }
                span { class: "oxs10-grow" }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true, right: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", PD_MODELS[i].0)); } }
                if running {
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
    } else if let Some(c) = cur.clone() {
        let sample = c.id == 1;
        let shown = if c.running { 3 } else { PD_STEPS.len() };
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    div { class: "oxs10-you", p { if sample { "{PD_PROMPT}" } else { "{c.title}" } } }
                    if sample {
                        ol { class: "oxs10-work", "aria-label": "Steps",
                            for (i, (g, s)) in PD_STEPS.iter().copied().take(shown).enumerate() {
                                li { key: "{i}", class: "oxs10-wi", {nico(g)} span { class: "oxs10-nl", "{s}" } }
                            }
                        }
                    }
                    if c.running {
                        span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                            let id = *open.peek();
                            if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.running = false; }
                            said.set("Preview ready".to_string());
                        } }
                        p { class: "oxs10-thinking", span { class: "oxs10-shim", if sample { "Starting the dev server" } else { "Reading the project" } } }
                    } else if sample {
                        div { class: "oxs10-agent", p { "{PD_DONE}" }
                            div { class: "oxs16p-links",
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { let mut t = ptab; t.set(0); said.set("Preview: the live site".to_string()); }, {nico("globe")} span { "Open preview" } }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { let mut t = ptab; t.set(2); said.set("Preview: the diff".to_string()); }, {nico("file")} span { "Review the diff" } }
                            }
                        }
                    } else {
                        div { class: "oxs10-agent", p { "Here's a first pass. Open the preview, or point at what to change in Design mode." } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {pd_mark(34)} }
                h1 { class: "oxs10-h1", "What should we build?" }
                div { class: "oxs10-under",
                    for (i, s) in ["Redesign the pricing page", "Write a rollout plan as a doc", "Fix the layout at phone width"].iter().copied().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxs10-envc oxs16p-sug", onclick: move |_| { text.set(s.to_string()); err.set(None); }, {nico("spark")} span { class: "oxs10-ell", "{s}" } }
                    }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let empty = d0.sessions.is_empty();
    let panel = rsx! { Previews { tab: ptab, own: own.to_string(), loading, empty, notes, said } };
    rsx! { Frame { app: "pd", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label: "Preview".to_string() } }
}

/// Preview Desk: a coding agent's work as previews. Sessions on machines of their own; the
/// session's steps; preview tabs (live site, doc, diff); Design mode to pick an element on the
/// site and comment; line comments on the diff and Approve; Machine, folder and model in the
/// composer. States `design` and `diff`.
#[component]
pub fn PreviewDeskScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty`, `design` or `diff`).
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
    /// Your data. `None` shows the sample for `state`; start from `PreviewDeskData::default()`.
    #[props(default)]
    data: Option<PreviewDeskData>,
) -> Element {
    rsx! { PreviewDeskApp { state, layout, height, rail, hidden: sidebar_hidden, data } }
}
