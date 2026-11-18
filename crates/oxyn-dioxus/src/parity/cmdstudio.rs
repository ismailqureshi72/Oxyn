//! Screens S10 (Agent apps II): two desktop coding-agent apps. **Command Center** (threads and
//! review): projects and their threads in the sidebar, a running thread with its work log, a
//! review pane of the changes (files, hunks, accept or revert), a Local / Worktree / Cloud switch,
//! a task queue and a composer with model and reasoning pickers. **Session Studio** (sessions and
//! tools): a session list, a chat whose turns show read, edit and bash blocks, permission prompts,
//! a permission mode with plan mode, a todo list and a diff pane. Rules in
//! `oxyn_core::parity::cmdstudio`; styles in `cmdstudio.css` (`oxs10-`). Chrome stays monochrome;
//! colour is for status, diffs, the folder glyph and model marks.

use dioxus::prelude::*;
use oxyn_core::parity::cmdstudio::{
    ASK_CMD, Answer, BState, Block, CC_APP, CC_MODELS, CC_PROJECTS, CC_PROMPT, CC_WORK, DiffFile,
    Mode, PERSON, PERSON_INI, PERSON_PLAN, Place, REASONING, SS_APP, SS_MODELS, ScreenState,
    Session, TState, Thread, Verdict, add_thread, answer, approve_plan, branch_for, dequeue,
    diff_stat, enqueue, file_stat, file_verdict, finish_running, hunk_stat, prompt_error, queue_up,
    review_line, reviewed, send, set_file, set_hunk, split, states_for, todo_progress, toggle_todo,
};
use oxyn_core::parity::cmdstudio::{
    Automation, CommandCenterData, PAGES, PANES, SIDE_CHAT, SessionStudioData, TRIGGERS, Term,
    automation_error, close_term, move_pane, new_term, page_for, run_cmd, side_reply,
};

use super::screensfid::{ico, nico, skel};
use crate::brand::BrandIcon;

/// The state bar entries of each S10 screen (for a host's `ScreenStage { states }`).
pub fn s10_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn branch_ico() -> Element {
    rsx! {
        svg { class: "oxscn-ico", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", "aria-hidden": "true",
            circle { cx: "6", cy: "5", r: "2" } circle { cx: "6", cy: "19", r: "2" } circle { cx: "18", cy: "7", r: "2" }
            path { d: "M6 7v10" } path { d: "M18 9c0 5-6 4-11 8" }
        }
    }
}

fn place_ico(p: Place) -> Element {
    if p == Place::Worktree {
        branch_ico()
    } else {
        nico(p.glyph())
    }
}

fn cc_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            rect { x: "1.5", y: "1.5", width: "17", height: "17", rx: "5", fill: "none", stroke: "currentColor", stroke_width: "1.8" }
            path { d: "M6 8l3 2-3 2", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round" }
            path { d: "M11 13h3", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round" }
        }
    }
}

fn ss_mark(size: u32) -> Element {
    rsx! {
        svg { class: "oxs10-logo", view_box: "0 0 20 20", width: "{size}", height: "{size}", "aria-hidden": "true",
            circle { cx: "10", cy: "10", r: "8", fill: "none", stroke: "currentColor", stroke_width: "1.8" }
            circle { cx: "10", cy: "10", r: "3", fill: "currentColor" }
        }
    }
}

fn tmark(tone: &str) -> Element {
    match tone {
        "run" => rsx! { span { class: "oxs10-spin", "aria-hidden": "true" } },
        "ok" => rsx! { span { class: "oxs10-ok", {nico("check")} } },
        "" => rsx! { span { class: "oxs10-ring", "aria-hidden": "true" } },
        _ => rsx! { span { class: "oxs10-dot", "data-tone": "{tone}", "aria-hidden": "true" } },
    }
}

fn pill(label: &str, tone: &str) -> Element {
    rsx! { span { class: "oxs10-pill", "data-tone": "{tone}", span { class: "oxs10-pd" } "{label}" } }
}

/// A chip with a small menu (radio items).
#[component]
fn Pick(
    label: String,
    lead: Element,
    items: Vec<(String, String, String)>,
    cur: usize,
    up: bool,
    #[props(default)] right: bool,
    on_pick: EventHandler<usize>,
) -> Element {
    let mut open = use_signal(|| false);
    let shown = items.get(cur).map(|x| x.0.clone()).unwrap_or_default();
    rsx! {
        div { class: "oxs10-pk",
            button { r#type: "button", class: "oxs10-chip", "aria-haspopup": "menu", "aria-expanded": "{open()}", "aria-label": "{label}: {shown}",
                onclick: move |_| open.toggle(), {lead} span { class: "oxs10-ell", "{shown}" } {nico("caret")} }
            if open() {
                div { class: "oxscn-menu oxs10-pkm", "data-up": "{up}", "data-right": "{right}", role: "menu", "aria-label": "{label}",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
                    for (i, (n, l, m)) in items.iter().cloned().enumerate() {
                        button { key: "{i}", r#type: "button", class: "oxscn-mi oxs10-pki", role: "menuitemradio", "aria-checked": "{i == cur}",
                            onclick: move |_| { open.set(false); on_pick.call(i); },
                            if !m.is_empty() { span { class: "oxs10-mk", BrandIcon { name: m.clone(), size: 15.0, coloured: true } } }
                            span { class: "oxs10-pkt", b { "{n}" } if !l.is_empty() { span { "{l}" } } }
                            if i == cur { span { class: "oxscn-mi-k", {nico("check")} } }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────── Diff ──

/// The review pane: files, hunks, accept / revert per hunk and per file.
#[component]
fn DiffPane(
    files: Signal<Vec<DiffFile>>,
    loading: bool,
    said: Signal<String>,
    title: String,
) -> Element {
    let mut files = files;
    let mut said = said;
    let mut shut = use_signal(Vec::<usize>::new);
    let fs = files();
    if loading {
        return rsx! { div { class: "oxs10-pbody", "aria-busy": "true", {skel(8)} } };
    }
    if fs.is_empty() {
        return rsx! {
            div { class: "oxs10-pempty", {nico("doc")} p { "Changes show up here, file by file, as the agent edits. Accept or revert each hunk." } }
        };
    }
    let (d, n) = reviewed(&fs);
    let line = review_line(&fs);
    rsx! {
        div { class: "oxs10-phd",
            div { class: "oxs10-pht", b { "{title}" } span { class: "oxs10-ell", "{line}" } }
            div { class: "oxs10-acts",
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", disabled: d == n,
                    onclick: move |_| { let k = files.peek().len(); for i in 0..k { set_file(&mut files.write(), i, Verdict::Reverted); } said.set("Reverted all changes".to_string()); }, "Revert all" }
                button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs10-pri", disabled: d == n,
                    onclick: move |_| { let k = files.peek().len(); for i in 0..k { set_file(&mut files.write(), i, Verdict::Accepted); } said.set("Accepted all changes".to_string()); }, "Accept all" }
            }
        }
        div { class: "oxs10-pbody",
            for (fi, f) in fs.iter().cloned().enumerate() {
                {
                    let (a, r) = file_stat(&f);
                    let fv = file_verdict(&f);
                    let open = !shut().contains(&fi);
                    let path = f.path;
                    rsx! {
                        section { key: "{path}", class: "oxs10-file", "data-v": "{verdict_key(fv)}", "aria-label": "{path}",
                            div { class: "oxs10-fh",
                                button { r#type: "button", class: "oxs10-fold", "aria-expanded": "{open}", "aria-label": "Show {path}",
                                    onclick: move |_| { let had = shut.peek().contains(&fi); if had { shut.write().retain(|x| *x != fi); } else { shut.write().push(fi); } },
                                    span { class: "oxs10-car", "data-open": "{open}", {nico("caret")} } }
                                span { class: "oxs10-fp oxs10-ell", "{path}" }
                                if f.new { span { class: "oxs10-tag", "New" } }
                                span { class: "oxs10-st", span { class: "oxs10-add", "+{a}" } span { class: "oxs10-del", "−{r}" } }
                                if fv != Verdict::Open { span { class: "oxs10-vd", "data-v": "{verdict_key(fv)}", "{verdict_label(fv)}" } }
                                button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Revert {path}", title: "Revert file",
                                    onclick: move |_| { set_file(&mut files.write(), fi, Verdict::Reverted); said.set(format!("Reverted {path}")); }, {nico("undo")} }
                                button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Accept {path}", title: "Accept file",
                                    onclick: move |_| { set_file(&mut files.write(), fi, Verdict::Accepted); said.set(format!("Accepted {path}")); }, {nico("check")} }
                            }
                            if open {
                                for (hi, h) in f.hunks.iter().cloned().enumerate() {
                                    {
                                        let (ha, hr) = hunk_stat(&h);
                                        let v = h.verdict;
                                        rsx! {
                                            div { key: "{hi}", class: "oxs10-hunk", "data-v": "{verdict_key(v)}",
                                                div { class: "oxs10-hh",
                                                    code { class: "oxs10-ell", "{h.head}" }
                                                    span { class: "oxs10-grow" }
                                                    if v == Verdict::Open {
                                                        button { r#type: "button", class: "oxs10-hbtn", "aria-label": "Revert hunk {hi + 1} of {path}",
                                                            onclick: move |_| { set_hunk(&mut files.write(), fi, hi, Verdict::Reverted); said.set(format!("Reverted a hunk, −{ha} +{hr} undone")); }, "Revert" }
                                                        button { r#type: "button", class: "oxs10-hbtn oxs10-pri", "aria-label": "Accept hunk {hi + 1} of {path}",
                                                            onclick: move |_| { set_hunk(&mut files.write(), fi, hi, Verdict::Accepted); said.set("Accepted a hunk".to_string()); }, "Accept" }
                                                    } else {
                                                        span { class: "oxs10-vd", "data-v": "{verdict_key(v)}", "{verdict_label(v)}" }
                                                        button { r#type: "button", class: "oxs10-hbtn", "aria-label": "Undo decision on hunk {hi + 1} of {path}",
                                                            onclick: move |_| { set_hunk(&mut files.write(), fi, hi, Verdict::Open); said.set("Hunk back to review".to_string()); }, "Undo" }
                                                    }
                                                }
                                                pre { class: "oxs10-code",
                                                    for (li, (k, t)) in h.lines.iter().copied().enumerate() {
                                                        span { key: "{li}", class: "oxs10-ln", "data-k": "{diff_key(k)}", span { class: "oxs10-sg", "{k}" } span { class: "oxs10-lt", "{t}" } }
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
            }
        }
    }
}

fn verdict_key(v: Verdict) -> &'static str {
    match v {
        Verdict::Open => "open",
        Verdict::Accepted => "ok",
        Verdict::Reverted => "rev",
    }
}
fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Open => "",
        Verdict::Accepted => "Accepted",
        Verdict::Reverted => "Reverted",
    }
}
fn diff_key(k: char) -> &'static str {
    match k {
        '+' => "add",
        '-' => "del",
        _ => "ctx",
    }
}

// ──────────────────────────────────────────────────────── Frame ──

/// The shared app frame: sidebar slot, top bar, body and the pane (wide, or over the body when
/// narrow).
#[allow(clippy::too_many_arguments)]
#[component]
fn Frame(
    app: &'static str,
    layout: String,
    height: u32,
    side: Signal<&'static str>,
    drawer: Signal<bool>,
    pane: Signal<bool>,
    pnar: Signal<bool>,
    said: Signal<String>,
    sidebar: Element,
    bar: Element,
    body: Element,
    panel: Element,
    pane_label: String,
) -> Element {
    let mut side = side;
    let mut dr = drawer;
    let mut pane = pane;
    let mut pnar = pnar;
    let sd = side();
    let style = format!("--oxs10-h:{height}px");
    rsx! {
        div { class: "oxs10", "data-app": "{app}", "data-layout": "{layout}", "data-side": "{sd}", "data-drawer": "{dr()}", "data-pane": "{pane()}", "data-pnar": "{pnar()}", style: "{style}",
            if sd != "hidden" { {sidebar} }
            button { r#type: "button", class: "oxs10-scrim", "aria-label": "Close sidebar", tabindex: "-1", onclick: move |_| { dr.set(false); pnar.set(false); } }
            main { class: "oxs10-main",
                div { class: "oxs10-bar",
                    button { r#type: "button", class: "oxscn-ib oxs10-menu", "aria-label": "Show sidebar", onclick: move |_| { if *side.peek() == "hidden" { side.set("full"); } dr.toggle(); }, {nico("side")} }
                    if sd == "rail" {
                        button { r#type: "button", class: "oxscn-ib oxs10-wide", "aria-label": "Expand sidebar", onclick: move |_| side.set("full"), {nico("side")} }
                    }
                    {bar}
                    button { r#type: "button", class: "oxs10-ptog oxs10-wide-only", "aria-pressed": "{pane()}", "aria-controls": "oxs10-pane-{app}",
                        onclick: move |_| pane.toggle(), {nico("panel")} span { class: "oxs10-ell", "{pane_label}" } }
                    button { r#type: "button", class: "oxs10-ptog oxs10-narrow-only", "aria-pressed": "{pnar()}", "aria-controls": "oxs10-pane-{app}",
                        onclick: move |_| pnar.toggle(), {nico("panel")} span { class: "oxs10-ell", "{pane_label}" } }
                }
                div { class: "oxs10-split",
                    div { class: "oxs10-col", {body} }
                    aside { class: "oxs10-pane", id: "oxs10-pane-{app}", "aria-label": "{pane_label}", {panel} }
                }
            }
            p { class: "oxs10-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

fn side_top(
    app: &'static str,
    rail: bool,
    mut side: Signal<&'static str>,
    mut dr: Signal<bool>,
) -> Element {
    let name = if app == "cc" { CC_APP } else { SS_APP };
    rsx! {
        div { class: "oxs10-stop",
            span { class: "oxs10-brand", if app == "cc" { {cc_mark(18)} } else { {ss_mark(18)} } if !rail { span { class: "oxs10-ell", "{name}" } } }
            if !rail {
                span { class: "oxs10-grow" }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Hide sidebar", onclick: move |_| { side.set("hidden"); dr.set(false); }, {nico("side")} }
            }
        }
    }
}

fn side_foot(rail: bool) -> Element {
    rsx! {
        div { class: "oxs10-acct",
            span { class: "oxs10-av", "aria-hidden": "true", "{PERSON_INI}" }
            if !rail {
                span { class: "oxs10-an", b { class: "oxs10-ell", "{PERSON}" } span { class: "oxs10-ell", "{PERSON_PLAN}" } }
                span { class: "oxscn-ib oxs10-gear", "aria-hidden": "true", {nico("gear")} }
            }
        }
    }
}

fn opts2(v: &[(&str, &str)]) -> Vec<(String, String, String)> {
    v.iter()
        .map(|x| (x.0.to_string(), String::new(), x.1.to_string()))
        .collect()
}

const DOLLAR: &str = "$ ";

// ─────────────────────────────────────── Command Center: pane tabs ──

/// The Command Center pane: Review, Terminal (tabs) and Browser.
#[component]
fn CcPane(
    files: Signal<Vec<DiffFile>>,
    loading: bool,
    said: Signal<String>,
    tab0: &'static str,
    terms0: Vec<Term>,
) -> Element {
    let mut said = said;
    let mut tab = use_signal(move || tab0);
    let mut terms = use_signal(move || terms0);
    let mut cur_t = use_signal(|| 1u32);
    let mut cmd = use_signal(String::new);
    let mut addr = use_signal(|| PAGES[0].0.to_string());
    let mut page = use_signal(|| Some(0usize));
    let mut hist = use_signal(|| vec![0usize]);
    let mut loading_pg = use_signal(|| false);
    let t = tab();
    let tv = terms();
    let ct = cur_t();
    let open_t = tv
        .iter()
        .find(|x| x.id == ct)
        .cloned()
        .or_else(|| tv.first().cloned());
    let tabs = [
        ("review", "Review", "doc"),
        ("terminal", "Terminal", "term"),
        ("browser", "Browser", "globe"),
    ];
    let mut go = move |a: String| {
        let p = page_for(&a);
        page.set(p);
        if let Some(i) = p {
            hist.write().push(i);
            addr.set(PAGES[i].0.to_string());
        }
        loading_pg.set(true);
        said.set(match p {
            Some(i) => format!("Opened {}", PAGES[i].1),
            None => "Page not found".to_string(),
        });
    };
    rsx! {
        div { class: "oxs10-ptabs", role: "tablist", "aria-label": "Pane",
            onkeydown: move |e: KeyboardEvent| {
                let k = e.key();
                let i = tabs.iter().position(|x| x.0 == *tab.peek()).unwrap_or(0);
                if k == Key::ArrowRight { tab.set(tabs[(i + 1) % 3].0); } else if k == Key::ArrowLeft { tab.set(tabs[(i + 2) % 3].0); }
            },
            for (k, l, g) in tabs {
                button { key: "{k}", r#type: "button", role: "tab", class: "oxs10-ptab", "aria-selected": "{t == k}", tabindex: if t == k { "0" } else { "-1" }, onclick: move |_| tab.set(k), {nico(g)} span { "{l}" } }
            }
        }
        match t {
            "terminal" => rsx! {
                div { class: "oxs10-tt", role: "tablist", "aria-label": "Terminals",
                    for x in tv.iter().cloned() {
                        {
                            let id = x.id;
                            let on = open_t.as_ref().is_some_and(|o| o.id == id);
                            rsx! {
                                span { key: "{id}", class: "oxs10-tti", "data-on": "{on}",
                                    button { r#type: "button", role: "tab", class: "oxs10-ttb", "aria-selected": "{on}", onclick: move |_| cur_t.set(id), {nico("term")} span { class: "oxs10-ell", "{x.title}" } }
                                    if tv.len() > 1 {
                                        button { r#type: "button", class: "oxs10-ttx", "aria-label": "Close {x.title}", onclick: move |_| { close_term(&mut terms.write(), id); if *cur_t.peek() == id { let f = terms.peek().first().map(|y| y.id).unwrap_or(0); cur_t.set(f); } said.set("Terminal closed".to_string()); }, {nico("close")} }
                                    }
                                }
                            }
                        }
                    }
                    button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "New terminal", onclick: move |_| { let id = new_term(&mut terms.write()); cur_t.set(id); said.set("New terminal".to_string()); }, {nico("plus")} }
                }
                if let Some(o) = open_t.clone() {
                    div { class: "oxs10-tpane", role: "log", "aria-label": "{o.title} output",
                        pre { class: "oxs10-tout",
                            for (i, l) in o.lines.iter().enumerate() { span { key: "{i}", class: "oxs10-tl", "data-cmd": "{l.starts_with(DOLLAR)}", "{l}" } }
                        }
                        label { class: "oxs10-tin", span { "aria-hidden": "true", "$" }
                            input { class: "oxs10-tinp", value: "{cmd}", placeholder: "Type a command (clear wipes the output)", "aria-label": "Command for {o.title}", spellcheck: "false",
                                oninput: move |e| cmd.set(e.value()),
                                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { let c = cmd.peek().clone(); if run_cmd(&mut terms.write(), o.id, &c) { cmd.set(String::new()); } } }
                        }
                    }
                }
            },
            "browser" => rsx! {
                div { class: "oxs10-bbar",
                    button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Back", disabled: hist().len() < 2,
                        onclick: move |_| { let mut h = hist.write(); h.pop(); if let Some(i) = h.last().copied() { page.set(Some(i)); addr.set(PAGES[i].0.to_string()); } }, {nico("back")} }
                    button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Reload", onclick: move |_| { loading_pg.set(true); said.set("Reloaded".to_string()); }, {nico("refresh")} }
                    input { class: "oxs10-addr", value: "{addr}", "aria-label": "Address", spellcheck: "false",
                        oninput: move |e| addr.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { let a = addr.peek().clone(); go(a); } }
                }
                div { class: "oxs10-bview", "data-loading": "{loading_pg()}",
                    if loading_pg() { span { class: "oxs10-bload", onanimationend: move |_| loading_pg.set(false) } }
                    match page() {
                        Some(i) => rsx! {
                            div { class: "oxs10-bpage",
                                h2 { "{PAGES[i].1}" }
                                for (j, l) in PAGES[i].2.iter().enumerate() { p { key: "{j}", "{l}" } }
                                div { class: "oxs10-blinks",
                                    for (j, pg) in PAGES.iter().enumerate() {
                                        if j != i { button { key: "{j}", r#type: "button", class: "oxs10-blink", onclick: move |_| go(pg.0.to_string()), "{pg.1}" } }
                                    }
                                }
                            }
                        },
                        None => rsx! { div { class: "oxs10-pempty", {nico("globe")} p { "Can't reach that page. Try localhost:3000/uploads." } } },
                    }
                }
            },
            _ => rsx! { DiffPane { files, loading, said, title: "Review".to_string() } },
        }
    }
}

// ─────────────────────────────────────── Command Center: automations ──

#[component]
fn Automations(loading: bool, said: Signal<String>, list0: Vec<Automation>) -> Element {
    let mut said = said;
    let mut list = use_signal(move || list0);
    let mut adding = use_signal(|| false);
    let mut nm = use_signal(String::new);
    let mut pr = use_signal(String::new);
    let mut trig = use_signal(|| 0usize);
    let mut tried = use_signal(|| false);
    let l = list();
    let err = automation_error(&nm(), &pr());
    rsx! {
        div { class: "oxs10-feed oxs10-auto",
            div { class: "oxs10-in",
                div { class: "oxs10-ahd",
                    div { class: "oxs10-s2", h1 { class: "oxs10-h2", "Automations" } span { class: "oxs10-ssub", "Threads that start themselves on a schedule or an event, each in its own worktree." } }
                    button { r#type: "button", class: "oxscn-primary oxs10-pbtn", "aria-expanded": "{adding()}", onclick: move |_| { adding.toggle(); tried.set(false); nm.set(String::new()); pr.set(String::new()); }, {nico("plus")} span { "New automation" } }
                }
                if adding() {
                    div { class: "oxs10-aform", role: "group", "aria-label": "New automation",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false); },
                        label { class: "oxs10-fl", "Name" input { class: "oxs10-fin", value: "{nm}", placeholder: "Nightly lint fixes", onmounted: super::setscr::focus_on, "aria-invalid": "{tried() && nm().trim().is_empty()}", oninput: move |e| nm.set(e.value()) } }
                        div { class: "oxs10-fl", span { "When" }
                            div { class: "oxs10-chips", role: "radiogroup", "aria-label": "When",
                                for (i, (k, lb)) in TRIGGERS.iter().copied().enumerate() {
                                    button { key: "{k}", r#type: "button", role: "radio", class: "oxs10-pchip", "aria-checked": "{trig() == i}", onclick: move |_| trig.set(i), "{lb}" }
                                }
                            }
                        }
                        label { class: "oxs10-fl", "What it does" textarea { class: "oxs10-fin oxs10-fta", rows: "2", value: "{pr}", placeholder: "Fix lint warnings and open a review thread.", oninput: move |e| pr.set(e.value()) } }
                        div { class: "oxs10-facts",
                            if let (true, Some(m)) = (tried(), err) { span { class: "oxs10-err", role: "alert", "{m}" } }
                            span { class: "oxs10-grow" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| {
                                tried.set(true);
                                if automation_error(&nm.peek(), &pr.peek()).is_some() { return; }
                                let id = list.peek().iter().map(|a| a.id).max().unwrap_or(0) + 1;
                                let n = nm.peek().trim().to_string();
                                list.write().insert(0, Automation { id, name: n.clone(), trigger: *trig.peek(), prompt: pr.peek().trim().to_string(), project: 0, on: true, last: ("", String::new()) });
                                adding.set(false);
                                said.set(format!("Automation {n} on: {}", TRIGGERS[*trig.peek()].1));
                            }, "Create" }
                        }
                    }
                }
                if loading {
                    div { "aria-busy": "true", {skel(6)} }
                } else if l.is_empty() && !adding() {
                    div { class: "oxs10-pempty", {nico("bolt")} p { "No automations yet. Have a thread start itself every night, or when CI fails." } }
                } else {
                    ul { class: "oxs10-alist", "aria-label": "Automations",
                        for a in l.iter().cloned() {
                            {
                                let id = a.id;
                                let an = a.name.clone();
                                rsx! {
                                    li { key: "{id}", class: "oxs10-arow", "data-on": "{a.on}",
                                        span { class: "oxs10-aic", {nico("bolt")} }
                                        span { class: "oxs10-s2",
                                            b { class: "oxs10-ell oxs10-stt", "{a.name}" }
                                            span { class: "oxs10-ell oxs10-ssub", "{TRIGGERS[a.trigger].1} · {CC_PROJECTS[a.project].1}" }
                                            span { class: "oxs10-ell oxs10-ssub", "{a.prompt}" }
                                        }
                                        if a.last.0 == "run" { {pill("Running", "run")} }
                                        else if a.last.0 == "ok" { {pill(&format!("Ran {}", a.last.1), "ok")} }
                                        else if a.last.0 == "bad" { {pill(&format!("Failed {}", a.last.1), "bad")} }
                                        else { {pill("Not run yet", "")} }
                                        button { r#type: "button", class: "oxscn-btn oxs10-sbtn", disabled: a.last.0 == "run", onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.last = ("run", "now".into()); } said.set(format!("Running {an}")); }, "Run now" }
                                        input { r#type: "checkbox", role: "switch", class: "oxs10-sw", checked: a.on, "aria-label": "{a.name} on",
                                            onchange: move |e| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.on = e.checked(); } } }
                                        if a.last.0 == "run" {
                                            span { class: "oxs10-tick", onanimationend: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.last = ("ok", "just now".into()); } said.set("Automation finished · a review thread is ready".to_string()); } }
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
}

// ────────────────────────────────────────────── Command Center ──

#[component]
fn CommandCenterApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
) -> Element {
    let (st, own) = split(&state);
    let loading = st == ScreenState::Loading;
    let wt = own == "worktree";
    let mut view = use_signal(move || {
        if own == "automations" {
            "automations"
        } else {
            "thread"
        }
    });
    let d1 = d0.clone();
    let mut list = use_signal(move || {
        let mut l = threads(st);
        if wt && let Some(t) = l.first_mut() {
            t.state = TState::Review;
            t.place = Place::Worktree;
        }
        l
    });
    let mut open = use_signal(move || if st == ScreenState::Sample { 1u32 } else { 0 });
    let mut files = use_signal(move || {
        if st == ScreenState::Sample {
            cc_diff()
        } else {
            Vec::new()
        }
    });
    let mut queue = use_signal(move || {
        if st == ScreenState::Sample {
            sample_queue()
        } else {
            Vec::new()
        }
    });
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
    let mut pnar = use_signal(|| false);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut model = use_signal(|| 0usize);
    let mut effort = use_signal(|| 1usize);
    let mut proj = use_signal(|| 0usize);
    let mut q = use_signal(String::new);
    let mut qopen = use_signal(|| true);
    let mut shut = use_signal(Vec::<usize>::new);
    let all = list();
    let oid = open();
    let cur: Option<Thread> = all.iter().find(|t| t.id == oid).cloned();
    let place = cur.as_ref().map(|t| t.place).unwrap_or(Place::Local);
    let running = cur.as_ref().is_some_and(|t| t.state == TState::Running);
    let fs = files();
    let (a, r) = diff_stat(&fs);
    let pane_label = if fs.is_empty() {
        "Review".to_string()
    } else {
        format!("Review +{a} −{r}")
    };
    let rl = side() == "rail";
    let mut submit = move || {
        let p = text.peek().clone();
        let run = list
            .peek()
            .iter()
            .any(|t| t.id == *open.peek() && t.state == TState::Running);
        if run {
            match enqueue(&mut queue.write(), &p) {
                Ok(_) => {
                    text.set(String::new());
                    err.set(None);
                    qopen.set(true);
                    said.set("Queued for when this thread finishes".to_string());
                }
                Err(e) => {
                    err.set(Some(e));
                    said.set(e.to_string());
                }
            }
            return;
        }
        let pl = list
            .peek()
            .iter()
            .find(|t| t.id == *open.peek())
            .map(|t| t.place)
            .unwrap_or(Place::Local);
        let pj = *proj.peek();
        let r = add_thread(&mut list.write(), &p, pj, pl);
        match r {
            Ok(id) => {
                open.set(id);
                files.set(Vec::new());
                text.set(String::new());
                err.set(None);
                said.set(format!("Thread started in {}", pl.label()));
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let models = opts2(&CC_MODELS);
    let efforts: Vec<(String, String, String)> = REASONING
        .iter()
        .map(|x| (x.0.to_string(), x.1.to_string(), String::new()))
        .collect();
    let places: Vec<(String, String, String)> = Place::ALL
        .iter()
        .map(|p| (p.label().to_string(), p.hint().to_string(), String::new()))
        .collect();
    let pidx = Place::ALL.iter().position(|p| *p == place).unwrap_or(0);
    let mk = CC_MODELS[model()].1;
    let can = prompt_error(&text()).is_none();
    let ql = q();
    let auto_v = view() == "automations";

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{CC_APP}",
            {side_top("cc", rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0 && !auto_v}", "aria-current": if oid == 0 && !auto_v { "page" } else { "false" },
                onclick: move |_| { open.set(0); view.set("thread"); files.set(Vec::new()); drawer.set(false); said.set("New thread".to_string()); },
                {nico("pen")} if !rl { span { class: "oxs10-nl", "New thread" } span { class: "oxs10-kbd", "⌘N" } } }
            button { r#type: "button", class: "oxs10-nav", "data-on": "{auto_v}", "aria-current": if auto_v { "page" } else { "false" }, title: "Automations",
                onclick: move |_| { view.set("automations"); drawer.set(false); said.set("Automations".to_string()); },
                {nico("bolt")} if !rl { span { class: "oxs10-nl", "Automations" } } }
            if !rl {
                input { class: "oxs10-find", r#type: "search", placeholder: "Search threads", "aria-label": "Search threads", value: "{ql}", oninput: move |e| q.set(e.value()) }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if all.is_empty() { p { class: "oxs10-faint", "No threads yet. Start one from the composer." } }
                    else {
                        for (pi, (_, pname)) in CC_PROJECTS.iter().copied().enumerate() {
                            {
                                let inner: Vec<Thread> = all.iter().filter(|t| t.project == pi && (ql.trim().is_empty() || t.title.to_lowercase().contains(&ql.trim().to_lowercase()))).cloned().collect();
                                let isopen = !shut().contains(&pi);
                                rsx! {
                                    div { key: "{pname}", class: "oxs10-grp",
                                        button { r#type: "button", class: "oxs10-gh", "aria-expanded": "{isopen}",
                                            onclick: move |_| { let had = shut.peek().contains(&pi); if had { shut.write().retain(|x| *x != pi); } else { shut.write().push(pi); } },
                                            {ico("folder")} span { class: "oxs10-nl", "{pname}" } span { class: "oxs10-cnt", "{inner.len()}" } }
                                        if isopen {
                                            ul { class: "oxs10-ul",
                                                for t in inner {
                                                    {
                                                        let id = t.id;
                                                        let on = id == oid;
                                                        let title = t.title.clone();
                                                        rsx! {
                                                            li { key: "{id}",
                                                                button { r#type: "button", class: "oxs10-row", "data-on": "{on && !auto_v}", "aria-current": if on && !auto_v { "page" } else { "false" },
                                                                    onclick: move |_| { open.set(id); view.set("thread"); files.set(dd.peek().diff(id)); drawer.set(false); said.set(format!("Opened {title}")); },
                                                                    {tmark(t.state.tone())}
                                                                    span { class: "oxs10-nl", "{t.title}" }
                                                                    if t.stat.0 + t.stat.1 > 0 { span { class: "oxs10-st oxs10-st-s", span { class: "oxs10-add", "+{t.stat.0}" } span { class: "oxs10-del", "−{t.stat.1}" } } }
                                                                    else { span { class: "oxs10-ago", "{t.ago}" } }
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
                        }
                    }
                }
            } else { span { class: "oxs10-grow" } }
            {side_foot(rl)}
        }
    };

    let bar = rsx! {
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if auto_v { "Automations" } else if let Some(t) = &cur { "{t.title}" } else { "New thread" } }
        if let (false, Some(t)) = (auto_v, &cur) { {pill(t.state.label(), t.state.tone())} }
        span { class: "oxs10-grow" }
        if !loading {
            Pick { label: "Work in".to_string(), lead: place_ico(place), items: places, cur: pidx, up: false, right: true,
                on_pick: move |i: usize| {
                    let p = Place::ALL[i];
                    let id = *open.peek();
                    if let Some(t) = list.write().iter_mut().find(|t| t.id == id) { t.place = p; }
                    said.set(format!("Moved to {} · {}", p.label(), branch_for(p, id)));
                } }
        }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": if running { "Queue a follow-up" } else { "Ask for a change" },
                placeholder: if running { "Queue a follow-up for when this finishes…" } else { "Ask for a change · @ to add files" },
                "aria-invalid": "{err().is_some()}", value: "{text}",
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); } }
            div { class: "oxs10-cbar",
                button { r#type: "button", class: "oxs10-cib", "aria-label": "Add files", onclick: move |_| said.set("Add files: pick from the project".to_string()), {nico("plus")} }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: mk.to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", CC_MODELS[i].0)); } }
                Pick { label: "Reasoning".to_string(), lead: rsx! { {nico("brain")} }, items: efforts, cur: effort(), up: true,
                    on_pick: move |i: usize| { effort.set(i); said.set(format!("Reasoning: {}", REASONING[i].0)); } }
                span { class: "oxs10-grow" }
                if running {
                    button { r#type: "button", class: "oxs10-stopb", "aria-label": "Stop the thread",
                        onclick: move |_| { let id = *open.peek(); if let Some(t) = list.write().iter_mut().find(|t| t.id == id) { t.state = TState::Review; } said.set("Stopped. Changes so far are in Review".to_string()); }, span { class: "oxs10-sq" } }
                }
                button { r#type: "button", class: "oxs10-send", "aria-label": if running { "Queue" } else { "Send" }, disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let qv = queue();
    let queue_el = rsx! {
        if !qv.is_empty() {
            div { class: "oxs10-q", "aria-label": "Task queue",
                button { r#type: "button", class: "oxs10-qh", "aria-expanded": "{qopen()}", onclick: move |_| qopen.toggle(),
                    span { class: "oxs10-car", "data-open": "{qopen()}", {nico("caret")} } span { "Queued next" } span { class: "oxs10-cnt", "{qv.len()}" } }
                if qopen() {
                    ol { class: "oxs10-ql",
                        for (i, (id, t)) in qv.iter().cloned().enumerate() {
                            li { key: "{id}", class: "oxs10-qi",
                                span { class: "oxs10-qn", "{i + 1}" }
                                span { class: "oxs10-nl", "{t}" }
                                button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Move up: {t}", disabled: i == 0, onclick: move |_| { queue_up(&mut queue.write(), id); said.set("Moved up".to_string()); }, {nico("updown")} }
                                button { r#type: "button", class: "oxscn-ib oxs10-hb", "aria-label": "Remove from queue: {t}", onclick: move |_| { dequeue(&mut queue.write(), id); said.set("Removed from the queue".to_string()); }, {nico("close")} }
                            }
                        }
                    }
                }
            }
        }
    };

    let body = if auto_v {
        rsx! { Automations { loading, said, list0: d0.automations.clone() } }
    } else if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(5)} } } }
    } else if let Some(t) = cur.clone() {
        let st = t.state;
        let br = branch_for(t.place, t.id);
        let pl = t.place;
        let done_n = if st == TState::Running { 4 } else { 5 };
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    if pl == Place::Worktree {
                        div { class: "oxs10-banner", role: "note", {branch_ico()} span { class: "oxs10-nl", "Working in a worktree on " code { "{br}" } ". Your checkout stays untouched." }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| { let id = *open.peek(); if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.place = Place::Local; } said.set("Handed off to Local · main".to_string()); }, "Hand off to Local" } }
                    }
                    div { class: "oxs10-you", p { if t.id == 1 { "{CC_PROMPT}" } else { "{t.title}" } } }
                    if t.id == 1 {
                        ol { class: "oxs10-work", "aria-label": "Work log",
                            for (i, (g, l, d)) in CC_WORK.iter().copied().enumerate() {
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
                        if st == TState::Running {
                            span { key: "{oid}", class: "oxs10-tick", onanimationend: move |_| {
                                let id = *open.peek();
                                if let Some(x) = list.write().iter_mut().find(|x| x.id == id && x.state == TState::Running) { x.state = TState::Review; }
                                said.set("Thread finished · changes ready to review".to_string());
                            } }
                            p { class: "oxs10-thinking", span { class: "oxs10-shim", "Working · {done_n} of 5 steps" } }
                        } else {
                            div { class: "oxs10-agent",
                                p { "Uploads now retry a 503 up to 4 times with exponential backoff (250 ms, doubling). I added a test that fails twice and then succeeds; all 3 retry tests pass." }
                                div { class: "oxs10-chg",
                                    span { {nico("doc")} } span { class: "oxs10-nl", "{review_line(&fs)}" }
                                    button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs10-narrow-only", onclick: move |_| pnar.set(true), "Review" }
                                }
                            }
                        }
                    } else {
                        div { class: "oxs10-agent", p { if st == TState::Running { "Working on it. Edits will show up in Review as they land." } else { "This thread has finished. Open Review to see what changed." } } }
                    }
                }
            }
            div { class: "oxs10-foot", {queue_el} {composer} }
        }
    } else {
        let pname = CC_PROJECTS[proj()].1;
        let projs: Vec<(String, String, String)> = CC_PROJECTS
            .iter()
            .map(|p| (p.1.to_string(), String::new(), String::new()))
            .collect();
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {cc_mark(34)} }
                h1 { class: "oxs10-h1", "What should we build in {pname}?" }
                div { class: "oxs10-under",
                    Pick { label: "Project".to_string(), lead: ico("folder"), items: projs, cur: proj(), up: false, on_pick: move |i: usize| proj.set(i) }
                    span { class: "oxs10-envc", {place_ico(place)} span { "{place.label()}" } }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    };
    let tab0 = match own {
        "terminal" => "terminal",
        "browser" => "browser",
        _ => "review",
    };
    let panel = rsx! { CcPane { files, loading, said, tab0, terms0: d0.terminals.clone() } };
    rsx! { Frame { app: "cc", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label } }
}

// ────────────────────────────────────────────── Session Studio ──

fn block_el(
    b: &Block,
    i: usize,
    mut blocks: Signal<Vec<Block>>,
    mut todos: Signal<Vec<(String, BState)>>,
    mut mode: Signal<Mode>,
    mut said: Signal<String>,
) -> Element {
    let body = b.body.clone();
    match b.kind {
        "user" => rsx! { div { key: "{i}", class: "oxs10-you", p { "{body}" } } },
        "text" => rsx! { div { key: "{i}", class: "oxs10-agent", p { "{body}" } } },
        "plan" => {
            let asking = b.state == BState::Asking;
            rsx! {
                div { key: "{i}", class: "oxs10-plan", role: "group", "aria-label": "Plan",
                    div { class: "oxs10-bh", {nico("list")} b { "Plan" } span { class: "oxs10-wd", "{body}" } }
                    ol { class: "oxs10-pl", for (j, s) in b.out.iter().enumerate() { li { key: "{j}", "{s}" } } }
                    if asking {
                        div { class: "oxs10-acts",
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs10-pri", onclick: move |_| { if let Some(t) = approve_plan(&mut blocks.write()) { todos.set(t); mode.set(Mode::AcceptEdits); said.set("Plan approved · mode: Accept edits".to_string()); } }, "Approve plan" }
                            button { r#type: "button", class: "oxscn-btn oxs10-sbtn", onclick: move |_| said.set("Still in plan mode; say what to change".to_string()), "Keep planning" }
                        }
                    } else { p { class: "oxs10-wd", "Approved · now the todo list" } }
                }
            }
        }
        k => {
            let (verb, g) = match k {
                "read" => ("Read", "file"),
                "edit" => ("Edit", "pen"),
                _ => ("Bash", "term"),
            };
            let st = b.state;
            let tone = match st {
                BState::Running => "run",
                BState::Asking => "wait",
                BState::Denied => "bad",
                _ => "ok",
            };
            rsx! {
                div { key: "{i}", class: "oxs10-tool", "data-k": "{k}", "data-s": "{tone}",
                    div { class: "oxs10-bh",
                        {nico(g)} b { "{verb}" } code { class: "oxs10-ell", if k == "bash" { "$ {body}" } else { "{body}" } }
                        span { class: "oxs10-grow" }
                        if st == BState::Running { span { class: "oxs10-spin" } }
                        else if st == BState::Denied { span { class: "oxs10-vd", "data-v": "rev", "Denied" } }
                        else if k == "read" { span { class: "oxs10-wd", "{b.out.first().cloned().unwrap_or_default()}" } }
                    }
                    if k == "edit" {
                        pre { class: "oxs10-code oxs10-code-s",
                            for (j, l) in b.out.iter().enumerate() {
                                { let (s, t) = l.split_at(1); let dk = if s == "+" { "add" } else if s == "-" { "del" } else { "ctx" }; let t = t.trim_start().to_string(); let s = s.to_string();
                                  rsx! { span { key: "{j}", class: "oxs10-ln", "data-k": "{dk}", span { class: "oxs10-sg", "{s}" } span { class: "oxs10-lt", "{t}" } } } }
                            }
                        }
                    }
                    if k == "bash" && !b.out.is_empty() {
                        pre { class: "oxs10-term", for (j, l) in b.out.iter().enumerate() { span { key: "{j}", class: "oxs10-tl", "{l}" } } }
                    }
                    if st == BState::Asking {
                        div { class: "oxs10-perm", role: "group", "aria-label": "Permission needed",
                            onkeydown: move |e: KeyboardEvent| {
                                let k = e.key().to_string();
                                let a = match k.as_str() { "1" => Some(Answer::Once), "2" => Some(Answer::Always), "3" => Some(Answer::Deny), _ => None };
                                if let Some(a) = a {
                                    e.prevent_default();
                                    answer(&mut blocks.write(), a);
                                    if a != Answer::Deny { tick_todo(todos); }
                                    said.set(if a == Answer::Deny { "Denied · the command did not run".to_string() } else { "Allowed · tests passed".to_string() });
                                }
                            },
                            p { b { "Run this command?" } span { " It runs the invoice tests in billing-svc." } }
                            div { class: "oxs10-acts",
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn oxs10-pri", "aria-label": "Allow once (1)", onclick: move |_| { answer(&mut blocks.write(), Answer::Once); tick_todo(todos); said.set("Allowed once · tests passed".to_string()); }, span { class: "oxs10-kn", "1" } "Allow once" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Always allow this command in billing-svc (2)", onclick: move |_| { answer(&mut blocks.write(), Answer::Always); tick_todo(todos); said.set(format!("Always allowed: {ASK_CMD}")); }, span { class: "oxs10-kn", "2" } "Always allow" }
                                button { r#type: "button", class: "oxscn-btn oxs10-sbtn", "aria-label": "Deny (3)", onclick: move |_| { answer(&mut blocks.write(), Answer::Deny); said.set("Denied · the command did not run".to_string()); }, span { class: "oxs10-kn", "3" } "Deny" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tick_todo(mut todos: Signal<Vec<(String, BState)>>) {
    for t in todos.write().iter_mut() {
        if matches!(t.1, BState::Asking | BState::Running) {
            t.1 = BState::Done;
        }
    }
}

#[component]
fn SessionStudioApp(
    state: String,
    layout: String,
    height: u32,
    rail: bool,
    hidden: bool,
) -> Element {
    let (st, own) = split(&state);
    let loading = st == ScreenState::Loading;
    let ask = own == "permission";
    let list: Vec<Session> = sessions(st);
    let mut open = use_signal(move || if st == ScreenState::Sample { 1u32 } else { 0 });
    let mut blocks = use_signal(move || {
        if st == ScreenState::Sample {
            session_blocks(ask)
        } else {
            Vec::new()
        }
    });
    let mut todos = use_signal(move || {
        if st == ScreenState::Sample {
            sample_todos(ask)
        } else {
            Vec::new()
        }
    });
    let mut files = use_signal(move || {
        if st == ScreenState::Sample {
            ss_diff()
        } else {
            Vec::new()
        }
    });
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
    let mut mode = use_signal(|| Mode::Ask);
    let mut model = use_signal(|| 1usize);
    let mut text = use_signal(String::new);
    let mut err = use_signal(|| None::<&'static str>);
    let mut topen = use_signal(|| true);
    let rl = side() == "rail";
    let oid = open();
    let cur = list.iter().find(|s| s.0 == oid).copied();
    let fs = files();
    let (a, r) = diff_stat(&fs);
    let pane_label = if fs.is_empty() {
        "Diff".to_string()
    } else {
        format!("Diff +{a} −{r}")
    };
    let md = mode();
    let mut submit = move || {
        let p = text.peek().clone();
        let m = *mode.peek();
        let r = send(&mut blocks.write(), &p, m);
        match r {
            Ok(()) => {
                text.set(String::new());
                err.set(None);
                said.set(if m == Mode::Plan {
                    "Plan ready for review".to_string()
                } else {
                    "Sent".to_string()
                });
            }
            Err(e) => {
                err.set(Some(e));
                said.set(e.to_string());
            }
        }
    };
    let modes: Vec<(String, String, String)> = Mode::ALL
        .iter()
        .map(|m| (m.label().to_string(), m.hint().to_string(), String::new()))
        .collect();
    let midx = Mode::ALL.iter().position(|m| *m == md).unwrap_or(0);
    let models = opts2(&SS_MODELS);
    let can = prompt_error(&text()).is_none();

    let sidebar = rsx! {
        nav { class: "oxs10-side", "data-rail": "{rl}", "aria-label": "{SS_APP}",
            {side_top("ss", rl, side, drawer)}
            button { r#type: "button", class: "oxs10-nav", "data-on": "{oid == 0}", "aria-current": if oid == 0 { "page" } else { "false" },
                onclick: move |_| { open.set(0); blocks.set(Vec::new()); todos.set(Vec::new()); files.set(Vec::new()); drawer.set(false); said.set("New session".to_string()); },
                {nico("plus")} if !rl { span { class: "oxs10-nl", "New session" } span { class: "oxs10-kbd", "⌘N" } } }
            if !rl {
                p { class: "oxs10-sh", "Sessions" }
                div { class: "oxs10-list",
                    if loading { div { class: "oxs10-skl", "aria-busy": "true", {skel(7)} } }
                    else if list.is_empty() { p { class: "oxs10-faint", "No sessions yet. Pick a folder and describe the change." } }
                    else {
                        ul { class: "oxs10-ul",
                            for (id, title, folder, ago, tone) in list.iter().copied() {
                                li { key: "{id}",
                                    button { r#type: "button", class: "oxs10-srow", "data-on": "{id == oid}", "aria-current": if id == oid { "page" } else { "false" },
                                        onclick: move |_| { open.set(id); let one = id == 1; blocks.set(if one { session_blocks(ask) } else { Vec::new() }); todos.set(if one { sample_todos(ask) } else { Vec::new() }); files.set(if one { ss_diff() } else { Vec::new() }); drawer.set(false); said.set(format!("Opened {title}")); },
                                        {tmark(if tone.is_empty() { "idle" } else { tone })}
                                        span { class: "oxs10-s2", span { class: "oxs10-ell oxs10-stt", "{title}" } span { class: "oxs10-ell oxs10-ssub", "{folder} · {ago}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            } else { span { class: "oxs10-grow" } }
            {side_foot(rl)}
        }
    };

    let bar = rsx! {
        span { class: "oxs10-ttl oxs10-ell", if loading { "" } else if let Some(s) = cur { "{s.1}" } else { "New session" } }
        if let Some(s) = cur { span { class: "oxs10-envc oxs10-hide-s", {ico("folder")} span { class: "oxs10-ell", "{s.2}" } } }
        span { class: "oxs10-grow" }
    };

    let tv = todos();
    let (td, tn) = todo_progress(&tv);
    let todo_el = rsx! {
        if !tv.is_empty() {
            div { class: "oxs10-q oxs10-todo", "aria-label": "Todo list",
                button { r#type: "button", class: "oxs10-qh", "aria-expanded": "{topen()}", onclick: move |_| topen.toggle(),
                    span { class: "oxs10-car", "data-open": "{topen()}", {nico("caret")} } span { "Todos" } span { class: "oxs10-cnt", "{td}/{tn}" } }
                if topen() {
                    ul { class: "oxs10-ql",
                        for (i, (t, s)) in tv.iter().cloned().enumerate() {
                            li { key: "{i}", class: "oxs10-ti", "data-s": "{todo_key(s)}",
                                label { class: "oxs10-tl2",
                                    input { r#type: "checkbox", class: "oxs10-cb", checked: s == BState::Done, onchange: move |_| { toggle_todo(&mut todos.write(), i); said.set("Todo updated".to_string()); } }
                                    span { class: "oxs10-nl", "{t}" }
                                    if s == BState::Running { span { class: "oxs10-spin oxs10-spin-s" } }
                                    if s == BState::Asking { span { class: "oxs10-dot", "data-tone": "wait" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let composer = rsx! {
        div { class: "oxs10-comp", "data-bad": "{err().is_some()}", "data-plan": "{md == Mode::Plan}",
            textarea { class: "oxs10-ta", rows: "2", "aria-label": "Message", value: "{text}", "aria-invalid": "{err().is_some()}",
                placeholder: if md == Mode::Plan { "Describe the change; you'll get a plan first" } else { "Ask for a change · Shift + Tab switches mode" },
                oninput: move |e| { text.set(e.value()); err.set(None); },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Tab && e.modifiers().shift() { e.prevent_default(); let n = mode.peek().next(); mode.set(n); said.set(format!("Mode: {}", n.label())); }
                    else if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(); }
                } }
            div { class: "oxs10-cbar",
                Pick { label: "Permission mode".to_string(), lead: rsx! { {nico(if md == Mode::Plan { "list" } else { "shield" })} }, items: modes, cur: midx, up: true,
                    on_pick: move |i: usize| { mode.set(Mode::ALL[i]); said.set(format!("Mode: {}", Mode::ALL[i].label())); } }
                Pick { label: "Model".to_string(), lead: rsx! { span { class: "oxs10-mk", BrandIcon { name: "claude".to_string(), size: 14.0, coloured: true } } }, items: models, cur: model(), up: true,
                    on_pick: move |i: usize| { model.set(i); said.set(format!("Model: {}", SS_MODELS[i].0)); } }
                span { class: "oxs10-grow" }
                button { r#type: "button", class: "oxs10-send", "aria-label": "Send", disabled: !can, onclick: move |_| submit(), {nico("send")} }
            }
        }
        if let Some(e) = err() { p { class: "oxs10-err", role: "alert", "{e}" } }
    };

    let bv = blocks();
    let body = if loading {
        rsx! { div { class: "oxs10-feed", "aria-busy": "true", div { class: "oxs10-in", {skel(3)} {skel(6)} } } }
    } else if bv.is_empty() {
        rsx! {
            div { class: "oxs10-feed oxs10-start",
                span { class: "oxs10-hero", {ss_mark(34)} }
                h1 { class: "oxs10-h1", "What are we changing today?" }
                div { class: "oxs10-under",
                    span { class: "oxs10-envc", {nico("laptop")} span { "Local" } }
                    span { class: "oxs10-envc", {ico("folder")} span { "billing-svc" } }
                    span { class: "oxs10-envc", {nico(if md == Mode::Plan { "list" } else { "shield" })} span { "{md.label()}" } }
                }
            }
            div { class: "oxs10-foot", {composer} }
        }
    } else {
        let running = bv.iter().any(|b| b.state == BState::Running);
        rsx! {
            div { class: "oxs10-feed",
                div { class: "oxs10-in",
                    for (i, b) in bv.iter().enumerate() { {block_el(b, i, blocks, todos, mode, said)} }
                    if running {
                        span { class: "oxs10-tick", onanimationend: move |_| { if finish_running(&mut blocks.write()) { tick_todo(todos); said.set("Tests passed".to_string()); } } }
                    }
                }
            }
            div { class: "oxs10-foot", {todo_el} {composer} }
        }
    };
    let panel = rsx! { DiffPane { files, loading, said, title: "Diff".to_string() } };
    rsx! { Frame { app: "ss", layout, height, side, drawer, pane, pnar, said, sidebar, bar, body, panel, pane_label } }
}

fn todo_key(s: BState) -> &'static str {
    match s {
        BState::Done => "done",
        BState::Running => "run",
        BState::Asking => "wait",
        _ => "pend",
    }
}

/// Command Center: the threads-and-review desktop agent app. Projects and their threads in the
/// sidebar, a running thread with its work log and a review pane of the changes (accept or revert
/// per hunk and file), Local / Worktree / Cloud, a task queue, and a composer with model and
/// reasoning pickers. State `worktree` (the thread finished in a worktree, ready to review).
#[component]
pub fn CommandCenterScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty` or `worktree`).
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
) -> Element {
    rsx! { CommandCenterApp { state, layout, height, rail, hidden: sidebar_hidden } }
}

/// Session Studio: the sessions-and-tools desktop agent app. A session list, a chat whose turns
/// show read, edit and bash blocks, permission prompts (Allow once, Always allow, Deny), a
/// permission mode with plan mode (Shift + Tab), a todo list and a diff pane. State `permission`
/// (the session waits to run a command).
#[component]
pub fn SessionStudioScreen(
    /// `inset` or `flat`: the main area as a rounded card on the sidebar's grey, or edge to edge.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key (`sample`, `loading`, `empty` or `permission`).
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
) -> Element {
    rsx! { SessionStudioApp { state, layout, height, rail, hidden: sidebar_hidden } }
}
