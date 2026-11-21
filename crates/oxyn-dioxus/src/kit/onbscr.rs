//! Screens S8: the onboarding wizard – Website, Brand, Tone and Ready. No app shell: a 56 px top
//! bar (Back, the signed-in address, Log out), a form column on the left and, on the right, the
//! assistant's dashboard as a live preview that glides to the part each step changes. Every step
//! is real: sites are cleaned up and read (simulated, with progress and failures), the brand takes
//! names, a colour (swatches or a hex) and a mark, the voice drives the preview's replies, and
//! Ready sums it all up with Edit links and turns the assistant Live. Back and Continue move
//! through the steps in place. Rules in `oxyn_core::kit::onbscr`; styles in `onbscr.css`
//! (`oxs8-`). Chrome stays monochrome; colour is for the brand colour the user picks and status.

use dioxus::prelude::*;
use oxyn_core::kit::onbscr::{
    Domain, LENGTHS, MARKS, OWN_DOMAIN, PERSON_EMAIL, PRODUCT, Phase, STEPS, SWATCHES, ScreenState,
    TEST_Q, TONES, Wiz, add_domain, add_name, answer, blocker, colour_warning, company,
    email_error, first_gap, greeting, heading, hex_ok, initials, ink_on, name_error, pages_total,
    retry, sources_words, split, states_for, step_of, summary, tick, wiz,
};

use super::screensfid::{nico, skel};
use super::setscr::focus_on;

/// The state bar entries of each S8 screen (for a host's `ScreenStage { states }`).
pub fn s8_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn start_wiz(st: ScreenState, own: &str) -> Wiz {
    let mut w = wiz(st != ScreenState::Empty);
    match own {
        "fetching" => {
            w.domains.push(Domain {
                host: format!("docs.{OWN_DOMAIN}"),
                phase: Phase::Reading(50),
            });
            w.domains.push(Domain {
                host: format!("help.{OWN_DOMAIN}"),
                phase: Phase::Queued,
            });
        }
        "custom" => {
            w.colour = "#0E7C86".to_string();
            w.mark = 3;
        }
        "live" => w.live = true,
        _ => {}
    }
    w
}

/// The brand mark: initials, a square, a round badge or the uploaded (fictional) image.
fn mark_el(w: &Wiz, size: u32) -> Element {
    let c = w.colour.clone();
    let ink = ink_on(&c);
    let ini = initials(company(w));
    let fs = size * 4 / 10;
    let kind = MARKS.get(w.mark).map(|m| m.0).unwrap_or("initials");
    let st =
        format!("--oxs8-c:{c};--oxs8-ink:{ink};width:{size}px;height:{size}px;font-size:{fs}px");
    rsx! {
        span { class: "oxs8-mark", "data-kind": "{kind}", style: "{st}", "aria-hidden": "true",
            match kind {
                "square" => rsx! { span { class: "oxs8-mark-sq" } },
                "upload" => rsx! {
                    svg { view_box: "0 0 24 24", width: "70%", height: "70%",
                        path { d: "M3 17 12 5l9 12-9-4z", fill: "var(--oxs8-c)" }
                        path { d: "M12 13v6", stroke: "var(--oxs8-c)", stroke_width: "2", stroke_linecap: "round" }
                    }
                },
                _ => rsx! { "{ini}" },
            }
        }
    }
}

fn product_mark() -> Element {
    rsx! {
        svg { class: "oxs8-logo", view_box: "0 0 20 20", width: "20", height: "20", "aria-hidden": "true",
            rect { x: "0", y: "0", width: "12", height: "12", rx: "2", fill: "currentColor" }
            rect { x: "8", y: "8", width: "12", height: "12", rx: "2", fill: "currentColor", opacity: ".45" }
        }
    }
}

fn phase_pill(p: Phase) -> Element {
    let t = p.tone();
    let l = p.label();
    rsx! { span { class: "oxs8-pill", "data-tone": "{t}", span { class: "oxs8-dot" } "{l}" } }
}

// ─────────────────────────────────────────────────────────── Preview ──

/// The assistant's dashboard, drawn from the wizard so far; `focus` is the step it glides to.
#[component]
fn Preview(w: Wiz, focus: usize, typing: bool, layout: String, loading: bool) -> Element {
    let live = w.live;
    let comp = company(&w).to_string();
    let comp_l = if comp.is_empty() {
        "Your company".to_string()
    } else {
        comp.clone()
    };
    let nm = if w.name.trim().is_empty() {
        "Your assistant".to_string()
    } else {
        w.name.trim().to_string()
    };
    let aliases: Vec<String> = w.names.iter().skip(1).cloned().collect();
    let rows = w.domains.len().max(3);
    // Rough section heights, so the focused one lands near the top.
    let hs = [
        104u32,
        196 + if aliases.is_empty() { 0 } else { 30 },
        70 + 41 * rows as u32,
        260,
    ];
    let sec = match focus {
        0 => 2,
        1 => 1,
        2 => 3,
        _ => 0,
    };
    let off: u32 = hs
        .iter()
        .take(sec)
        .sum::<u32>()
        .saturating_sub(if sec == 0 { 0 } else { 28 });
    let pv_style = format!("transform:translateY(-{off}px)");
    let all = focus == 3;
    let status = if live { "Live" } else { "Draft" };
    let st_tone = if live { "ok" } else { "" };
    let colour = w.colour.clone();
    let hand = w.handover && !w.handover_to.trim().is_empty();
    let to = w.handover_to.trim().to_string();
    let ans = answer(w.tone, w.length);
    let greet = greeting(w.tone, &nm, &comp);
    let pages = pages_total(&w.domains);
    let srcs = sources_words(&w.domains);
    rsx! {
        div { class: "oxs8-win", "data-layout": "{layout}", "aria-hidden": "true",
            div { class: "oxs8-wside",
                div { class: "oxs8-wws",
                    if loading { span { class: "oxs8-sk oxs8-sk-m" } } else { {mark_el(&w, 20)} }
                    span { class: "oxs8-wws-n", "{comp_l}" }
                }
                for i in 0..6u32 {
                    div { key: "{i}", class: "oxs8-wnav", span { class: "oxs8-sk oxs8-sk-d" } span { class: "oxs8-sk", style: "width:{60 + (i * 23) % 50}px" } }
                }
            }
            div { class: "oxs8-wbody",
                div { class: "oxs8-wtop", {nico("side")} span { "Overview" } }
                div { class: "oxs8-pvw",
                if loading {
                    div { class: "oxs8-pvc", {skel(4)} div { class: "oxs8-card", {skel(3)} } div { class: "oxs8-card", {skel(5)} } }
                } else {
                    div { class: "oxs8-pvc", style: "{pv_style}",
                        section { class: "oxs8-ps oxs8-tiles", "data-on": "{all || sec == 0}",
                            div { class: "oxs8-tile", span { class: "oxs8-tl", "Knowledge" } b { "{srcs}" } span { class: "oxs8-tq", "{pages} pages learned" } }
                            div { class: "oxs8-tile", span { class: "oxs8-tl", "Conversations" } b { "0" } span { class: "oxs8-tq", "None yet" } }
                            div { class: "oxs8-tile", span { class: "oxs8-tl", "Status" } b { class: "oxs8-tstat", "data-tone": "{st_tone}", "{status}" } span { class: "oxs8-tq", if live { "Answering" } else { "Not public yet" } } }
                        }
                        section { class: "oxs8-ps", "data-on": "{all || sec == 1}",
                            h3 { class: "oxs8-h3", "Your assistant" }
                            div { class: "oxs8-card oxs8-asst",
                                div { class: "oxs8-asst-h", {mark_el(&w, 22)} b { class: "oxs8-ell", "{nm}" }
                                    span { class: "oxs8-pill", "data-tone": "{st_tone}", span { class: "oxs8-dot" } "{status}" } }
                                p { class: "oxs8-bub", "{greet}" }
                                if !aliases.is_empty() {
                                    div { class: "oxs8-also", span { class: "oxs8-tq", "Also answers to" }
                                        for a in aliases.iter() { span { key: "{a}", class: "oxs8-tag", style: "--oxs8-c:{colour}", "{a}" } } }
                                }
                                div { class: "oxs8-btnpv", style: "--oxs8-c:{colour};--oxs8-ink:{ink_on(&colour)}", "Chat with us" }
                            }
                        }
                        section { class: "oxs8-ps", "data-on": "{all || sec == 2}",
                            h3 { class: "oxs8-h3", "Knowledge" }
                            div { class: "oxs8-card oxs8-kt",
                                div { class: "oxs8-kr oxs8-kh", span { "Source" } span { "Status" } span { "Pages" } }
                                for d in w.domains.iter() {
                                    div { key: "{d.host}", class: "oxs8-kr",
                                        span { class: "oxs8-kn", span { class: "oxs8-fav", "{d.host.chars().next().unwrap_or('?').to_ascii_uppercase()}" } span { class: "oxs8-ell", "{d.host}" } }
                                        span { {phase_pill(d.phase)} }
                                        span { class: "oxs8-tq", if let Phase::Ready(n) = d.phase { "{n}" } else { "–" } }
                                    }
                                }
                                for i in w.domains.len()..rows {
                                    div { key: "sk{i}", class: "oxs8-kr", span { class: "oxs8-sk", style: "width:140px" } span { class: "oxs8-sk", style: "width:60px" } span { class: "oxs8-sk", style: "width:24px" } }
                                }
                            }
                        }
                        section { class: "oxs8-ps", "data-on": "{all || sec == 3}",
                            h3 { class: "oxs8-h3", "Test chat" }
                            div { class: "oxs8-card oxs8-chat",
                                div { class: "oxs8-q", span { class: "oxs8-qa", {nico("user")} } p { class: "oxs8-bub oxs8-bub-q", "{TEST_Q}" } }
                                div { class: "oxs8-a",
                                    {mark_el(&w, 20)}
                                    div { class: "oxs8-a-c",
                                        span { class: "oxs8-tq oxs8-ell", "{nm}" }
                                        if typing {
                                            p { class: "oxs8-bub oxs8-typing", span {} span {} span {} }
                                        } else {
                                            p { class: "oxs8-bub", "{ans}" }
                                        }
                                    }
                                }
                                p { class: "oxs8-hand",
                                    if hand { "Not sure? It hands over to " span { class: "oxs8-tag", style: "--oxs8-c:{colour}", "{to}" } } else { "It answers everything itself" }
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

// ─────────────────────────────────────────────────────────── Wizard ──

#[derive(Clone, Copy, PartialEq)]
struct W {
    w: Signal<Wiz>,
    said: Signal<String>,
    typing: Signal<bool>,
}

/// The wizard around one step. Back / Continue / Edit switch the step in place.
#[component]
fn Wizard(
    slug: String,
    state: String,
    layout: String,
    height: u32,
    named: bool,
    form_only: bool,
    on_done: EventHandler<()>,
    on_logout: EventHandler<()>,
) -> Element {
    let (st, own) = split(&state);
    let loading = st == ScreenState::Loading;
    let start = step_of(&slug);
    let w = use_signal(move || start_wiz(st, own));
    let mut step = use_signal(move || start);
    let mut said = use_signal(String::new);
    let mut typing = use_signal(move || own == "testing");
    let mut pv_open = use_signal(|| false);
    let mut reached = use_signal(move || start);
    let ctx = W { w, said, typing };
    let s = step();
    let (h1, lead) = heading(s);
    let wz = w();
    let mut go = move |to: usize| {
        step.set(to);
        if to > *reached.peek() {
            reached.set(to);
        }
        said.set(format!("Step {} of 4: {}", to + 1, STEPS[to].1));
    };
    let far = reached();
    let pv = pv_open();
    let root_style = format!("--oxs8-h:{height}px");
    rsx! {
        div { class: "oxs8", "data-step": "{STEPS[s].0}", "data-form-only": "{form_only}", "data-pv": "{pv}", style: "{root_style}",
            header { class: "oxs8-top",
                {product_mark()}
                if s > 0 && !loading {
                    button { r#type: "button", class: "oxscn-btn oxs8-back", onclick: move |_| go(s - 1), {nico("back")} span { "Back" } }
                }
                span { class: "oxs8-grow" }
                if !form_only {
                    button { r#type: "button", class: "oxscn-btn oxs8-pvb", "aria-expanded": "{pv}", "aria-controls": "oxs8-pv", onclick: move |_| pv_open.toggle(), {nico("eye")} span { "Preview" } }
                }
                span { class: "oxs8-who", "Logged in as " b { "{PERSON_EMAIL}" } }
                button { r#type: "button", class: "oxscn-btn oxs8-out", onclick: move |_| { said.set("Logged out".to_string()); on_logout.call(()); }, {nico("logout")} span { "Log out" } }
            }
            div { class: "oxs8-body",
                main { class: "oxs8-main",
                    div { class: "oxs8-col",
                        if loading {
                            div { class: "oxs8-load", "aria-busy": "true", {skel(2)} span { class: "oxs8-sk oxs8-sk-f" } {skel(3)} }
                        } else {
                            ol { class: "oxs8-steps", "data-named": "{named}", "aria-label": "Steps",
                                for (i, (k, l)) in STEPS.iter().copied().enumerate() {
                                    li { key: "{k}",
                                        button { r#type: "button", class: "oxs8-st", "data-done": "{i != s && i <= far}", "data-on": "{i == s}",
                                            "aria-current": if i == s { "step" } else { "false" },
                                            "aria-label": "Step {i + 1} of 4: {l}",
                                            disabled: i > far,
                                            onclick: move |_| go(i),
                                            span { class: "oxs8-dash" }
                                            if named { span { class: "oxs8-stl", "{l}" } }
                                        }
                                    }
                                }
                            }
                            h1 { class: "oxs8-h1", tabindex: "-1", "{h1}" }
                            p { class: "oxs8-lead", "{lead}" }
                            match s {
                                0 => rsx! { WebsiteStep { ctx, on_next: move |_| go(1) } },
                                1 => rsx! { BrandStep { ctx, on_next: move |_| go(2) } },
                                2 => rsx! { ToneStep { ctx, on_next: move |_| go(3) } },
                                _ => rsx! { ReadyStep { ctx, on_edit: move |i: usize| go(i), on_done } },
                            }
                        }
                    }
                }
                if !form_only {
                    aside { class: "oxs8-pv", id: "oxs8-pv", "aria-label": "Dashboard preview",
                        button { r#type: "button", class: "oxscn-btn oxs8-pvx", onclick: move |_| pv_open.set(false), {nico("close")} span { "Close preview" } }
                        Preview { w: wz.clone(), focus: s, typing: typing(), layout: layout.clone(), loading }
                        if typing() && !loading {
                            span { key: "{wz.tone}-{wz.length}", class: "oxs8-tick oxs8-tick-l", onanimationend: move |_| typing.set(false) }
                        }
                    }
                }
            }
            footer { class: "oxs8-foot", span { "© 2026 {PRODUCT}" } a { href: "#terms", "Terms" } a { href: "#privacy", "Privacy" } }
            // Site reading runs on CSS ticks: each tick moves one site on by 25 %.
            for d in wz.domains.iter().filter(|d| matches!(d.phase, Phase::Reading(_) | Phase::Queued)).take(1) {
                {
                    let host = d.host.clone();
                    let k = format!("{}-{:?}", d.host, d.phase);
                    let mut w = w;
                    rsx! {
                        span { key: "{k}", class: "oxs8-tick",
                            onanimationend: move |_| {
                                let mut cur = w.write();
                                if let Some(x) = cur.domains.iter_mut().find(|x| x.host == host) {
                                    tick(x);
                                    match x.phase {
                                        Phase::Ready(n) => said.set(format!("{} read: {n} pages", x.host)),
                                        Phase::Failed => said.set(format!("Can't reach {}", x.host)),
                                        _ => {}
                                    }
                                }
                            } }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// The field with + (add) and an optional → (continue) used by Website and Brand.
#[component]
fn AddField(
    id: String,
    label: String,
    placeholder: String,
    add_label: String,
    arrow: bool,
    err: Option<String>,
    on_add: EventHandler<String>,
    on_next: EventHandler<String>,
) -> Element {
    let mut v = use_signal(String::new);
    let bad = err.is_some();
    let eid = format!("{id}-err");
    rsx! {
        div { class: "oxs8-glow",
            div { class: "oxs8-fld", "data-bad": "{bad}",
                input { class: "oxs8-in", id: "{id}", value: "{v}", placeholder: "{placeholder}", "aria-label": "{label}", "aria-invalid": "{bad}",
                    "aria-describedby": if bad { eid.clone() } else { String::new() },
                    autocomplete: "off", spellcheck: "false",
                    oninput: move |e| v.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            e.prevent_default();
                            let t = v.peek().clone();
                            if t.trim().is_empty() { on_next.call(String::new()); } else { on_add.call(t); v.set(String::new()); }
                        } else if e.key() == Key::Escape { v.set(String::new()); }
                    } }
                button { r#type: "button", class: "oxs8-ib", "aria-label": "{add_label}", title: "{add_label}",
                    onclick: move |_| { let t = v.peek().clone(); on_add.call(t); v.set(String::new()); }, {nico("plus")} }
                if arrow {
                    button { r#type: "button", class: "oxs8-go", "aria-label": "Continue", title: "Continue",
                        onclick: move |_| { let t = v.peek().clone(); v.set(String::new()); on_next.call(t); }, {nico("next")} }
                }
            }
        }
        if let Some(m) = err { p { class: "oxs8-err", id: "{eid}", role: "alert", "{m}" } }
    }
}

// ─────────────────────────────────────────────────────────── Website ──

#[component]
fn WebsiteStep(ctx: W, on_next: EventHandler<()>) -> Element {
    let mut w = ctx.w;
    let mut said = ctx.said;
    let mut err = use_signal(|| None::<String>);
    let wz = w();
    let mut add = move |t: String| -> bool {
        let r = add_domain(&mut w.write().domains, &t);
        match r {
            Ok(h) => {
                err.set(None);
                said.set(format!("Added {h}; reading it now"));
                true
            }
            Err(m) => {
                err.set(Some(m.to_string()));
                false
            }
        }
    };
    let n = wz.domains.len();
    rsx! {
        AddField { id: "oxs8-site", label: "Website or docs domain", placeholder: "{OWN_DOMAIN}", add_label: "Add domain", arrow: true, err: err(),
            on_add: move |t: String| { add(t); },
            on_next: move |t: String| {
                if !t.trim().is_empty() && !add(t) { return; }
                match blocker(&w.peek(), 0) { Some(m) => err.set(Some(m.to_string())), None => on_next.call(()) }
            } }
        if n > 0 {
            ul { class: "oxs8-chips", "aria-label": "Sites it learns from",
                for (i, d) in wz.domains.iter().enumerate() {
                    {
                        let h = d.host.clone();
                        let h2 = d.host.clone();
                        let failed = d.phase == Phase::Failed;
                        rsx! {
                            li { key: "{d.host}", class: "oxs8-chip", "data-tone": "{d.phase.tone()}",
                                span { class: "oxs8-fav", "{d.host.chars().next().unwrap_or('?').to_ascii_uppercase()}" }
                                span { class: "oxs8-ell oxs8-chip-n", "{d.host}" }
                                if i == 0 { span { class: "oxs8-own", "Yours" } }
                                {phase_pill(d.phase)}
                                if failed {
                                    button { r#type: "button", class: "oxscn-link oxs8-retry", "aria-label": "Retry {d.host}",
                                        onclick: move |_| { if let Some(x) = w.write().domains.iter_mut().find(|x| x.host == h2) { retry(x); } said.set("Trying again".to_string()); }, "Retry" }
                                }
                                button { r#type: "button", class: "oxs8-x", "aria-label": "Remove {d.host}",
                                    onclick: move |_| { w.write().domains.retain(|x| x.host != h); said.set(format!("Removed {h}")); }, {nico("close")} }
                            }
                        }
                    }
                }
            }
        } else {
            p { class: "oxs8-note", "Nothing added yet. Start with your main site; we clean up https://, www. and paths for you." }
        }
        if wz.domains.iter().any(|d| d.phase == Phase::Failed) {
            p { class: "oxs8-note oxs8-warn", "A private address can't be read from outside. Remove it, or add a public copy of those pages." }
        }
    }
}

// ───────────────────────────────────────────────────────────── Brand ──

#[component]
fn BrandStep(ctx: W, on_next: EventHandler<()>) -> Element {
    let mut w = ctx.w;
    let mut said = ctx.said;
    let mut err = use_signal(|| None::<String>);
    let wz = w();
    let custom = !SWATCHES
        .iter()
        .any(|s| s.1.eq_ignore_ascii_case(&wz.colour));
    let mut hex = use_signal(move || {
        if custom {
            wz.colour.clone()
        } else {
            String::new()
        }
    });
    let wz = w();
    let hx = hex();
    let hex_err = if hx.trim().is_empty() {
        None
    } else {
        hex_ok(&hx).err()
    };
    let warn = colour_warning(&wz.colour);
    let mut add = move |t: String| -> bool {
        match add_name(&mut w.write().names, &t) {
            Ok(n) => {
                err.set(None);
                said.set(format!("Added {n}"));
                true
            }
            Err(m) => {
                err.set(Some(m.to_string()));
                false
            }
        }
    };
    let upload = MARKS[3].0;
    rsx! {
        AddField { id: "oxs8-name", label: "Company name", placeholder: "Kestrel Studio", add_label: "Add name", arrow: false, err: err(),
            on_add: move |t: String| { add(t); },
            on_next: move |_| { match blocker(&w.peek(), 1) { Some(m) => err.set(Some(m.to_string())), None => on_next.call(()) } } }
        if wz.names.is_empty() {
            p { class: "oxs8-note", "Add the name people know you by, then any short forms." }
        } else {
            ul { class: "oxs8-chips", "aria-label": "Names it answers to",
                for (i, n) in wz.names.iter().enumerate() {
                    {
                        let n2 = n.clone();
                        rsx! {
                            li { key: "{n}", class: "oxs8-chip",
                                span { class: "oxs8-ell oxs8-chip-n", "{n}" }
                                span { class: "oxs8-own", if i == 0 { "Company" } else { "Short form" } }
                                button { r#type: "button", class: "oxs8-x", "aria-label": "Remove {n}",
                                    onclick: move |_| { w.write().names.retain(|x| *x != n2); said.set(format!("Removed {n2}")); }, {nico("close")} }
                            }
                        }
                    }
                }
            }
        }
        div { class: "oxs8-grp",
            p { class: "oxs8-lb", id: "oxs8-col-l", "Colour" }
            div { class: "oxs8-sw", role: "radiogroup", "aria-labelledby": "oxs8-col-l",
                for (nm, c) in SWATCHES.iter().copied() {
                    {
                        let on = wz.colour.eq_ignore_ascii_case(c);
                        rsx! {
                            button { key: "{c}", r#type: "button", role: "radio", class: "oxs8-swb", "aria-checked": "{on}", "aria-label": "{nm}", title: "{nm} {c}", style: "--oxs8-c:{c}",
                                onclick: move |_| { w.write().colour = c.to_string(); hex.set(String::new()); said.set(format!("Colour: {nm}")); },
                                if on { span { class: "oxs8-swk", style: "color:{ink_on(c)}", {nico("check")} } } }
                        }
                    }
                }
                label { class: "oxs8-hex", "data-on": "{custom}", "data-bad": "{hex_err.is_some()}",
                    span { class: "oxs8-hex-s", style: "--oxs8-c:{wz.colour}" }
                    input { class: "oxs8-hex-in", value: "{hx}", placeholder: "#hex", "aria-label": "Custom colour (hex)", maxlength: "7", "aria-invalid": "{hex_err.is_some()}",
                        oninput: move |e| { let v = e.value(); if let Ok(c) = hex_ok(&v) { w.write().colour = c; } hex.set(v); } }
                }
            }
            if let Some(m) = hex_err { p { class: "oxs8-err", "{m}" } } else if let Some(m) = warn { p { class: "oxs8-note oxs8-warn", "{m}" } }
        }
        div { class: "oxs8-grp",
            p { class: "oxs8-lb", id: "oxs8-mk-l", "Mark" }
            div { class: "oxs8-marks", role: "radiogroup", "aria-labelledby": "oxs8-mk-l",
                for (i, (k, l)) in MARKS.iter().copied().enumerate() {
                    {
                        let on = wz.mark == i;
                        let mut m = wz.clone();
                        m.mark = i;
                        rsx! {
                            button { key: "{k}", r#type: "button", role: "radio", class: "oxs8-mk", "aria-checked": "{on}",
                                onclick: move |_| { w.write().mark = i; said.set(if k == upload { "Logo uploaded: kestrel-mark.svg".to_string() } else { format!("Mark: {l}") }); },
                                {mark_el(&m, 28)} span { "{l}" } }
                        }
                    }
                }
            }
            if wz.mark == 3 {
                p { class: "oxs8-file", {nico("image")} span { class: "oxs8-ell", "kestrel-mark.svg · 4 KB" }
                    button { r#type: "button", class: "oxscn-link", onclick: move |_| { w.write().mark = 0; said.set("Logo removed".to_string()); }, "Remove" } }
            }
        }
        button { r#type: "button", class: "oxscn-primary oxs8-cont", disabled: blocker(&wz, 1).is_some(),
            onclick: move |_| on_next.call(()), "Continue" {nico("next")} }
    }
}

// ────────────────────────────────────────────────────────────── Tone ──

fn seg(
    id: &str,
    label: &str,
    opts: &'static [&'static str],
    cur: usize,
    set: impl FnMut(usize) + 'static,
) -> Element {
    let set = Callback::new(set);
    rsx! {
        div { class: "oxs8-grp",
            p { class: "oxs8-lb", id: "{id}", "{label}" }
            div { class: "oxscn-seg oxs8-seg", role: "radiogroup", "aria-labelledby": "{id}",
                for (i, o) in opts.iter().copied().enumerate() {
                    button { key: "{o}", r#type: "button", role: "radio", class: "oxscn-seg-b", "aria-checked": "{i == cur}", "aria-selected": "{i == cur}",
                        onclick: move |_| set.call(i), "{o}" }
                }
            }
        }
    }
}

#[component]
fn ToneStep(ctx: W, on_next: EventHandler<()>) -> Element {
    let mut w = ctx.w;
    let mut said = ctx.said;
    let mut typing = ctx.typing;
    let mut tried = use_signal(|| false);
    let mut touched = use_signal(|| false);
    let wz = w();
    let ne = name_error(&wz.name);
    let ee = if wz.handover {
        email_error(&wz.handover_to)
    } else {
        None
    };
    let show_ne = ne.is_some() && (tried() || touched() || !wz.name.is_empty());
    let show_ee = ee.is_some() && (tried() || !wz.handover_to.is_empty());
    let n = wz.name.chars().count();
    let on = wz.handover;
    rsx! {
        div { class: "oxs8-grp",
            label { class: "oxs8-lb", r#for: "oxs8-an", "Name" }
            input { class: "oxs8-in oxs8-box", id: "oxs8-an", value: "{wz.name}", placeholder: "Kestrel Studio Assistant", "aria-invalid": "{show_ne}",
                oninput: move |e| { touched.set(true); w.write().name = e.value(); } }
            div { class: "oxs8-meta",
                if show_ne { span { class: "oxs8-err", role: "alert", "{ne.unwrap_or_default()}" } }
                span { class: "oxs8-cnt", "data-over": "{n > 40}", "{n}/40" }
            }
        }
        {seg("oxs8-tn", "Tone", &TONES, wz.tone, move |i| { w.write().tone = i; typing.set(true); said.set(format!("Tone: {}", TONES[i])); })}
        {seg("oxs8-ln", "Reply length", &LENGTHS, wz.length, move |i| { w.write().length = i; typing.set(true); said.set(format!("Reply length: {}", LENGTHS[i])); })}
        div { class: "oxs8-grp",
            div { class: "oxs8-swrow",
                div { class: "oxs8-swt", label { class: "oxs8-lb", r#for: "oxs8-ho", "Hand over to a person" } span { class: "oxs8-tq", "When it can't answer for sure" } }
                button { r#type: "button", class: "oxscn-switch", role: "switch", id: "oxs8-ho", "aria-checked": "{on}",
                    onclick: move |_| { let v = !on; w.write().handover = v; said.set(if v { "Hands over to a person".to_string() } else { "Answers everything itself".to_string() }); }, span {} }
            }
            if on {
                input { class: "oxs8-in oxs8-box", r#type: "email", value: "{wz.handover_to}", placeholder: "support@kestrel.example", "aria-label": "Hand over to (email)", "aria-invalid": "{show_ee}",
                    onmounted: move |e| { if w.peek().handover_to.is_empty() { focus_on(e) } },
                    oninput: move |e| w.write().handover_to = e.value() }
                if show_ee { p { class: "oxs8-err", role: "alert", "{ee.unwrap_or_default()}" } }
            }
        }
        button { r#type: "button", class: "oxscn-primary oxs8-cont",
            onclick: move |_| { tried.set(true); match blocker(&w.peek(), 2) { Some(m) => said.set(m.to_string()), None => on_next.call(()) } }, "Continue" {nico("next")} }
    }
}

// ───────────────────────────────────────────────────────────── Ready ──

#[component]
fn ReadyStep(ctx: W, on_edit: EventHandler<usize>, on_done: EventHandler<()>) -> Element {
    let mut w = ctx.w;
    let mut said = ctx.said;
    let wz = w();
    let gap = first_gap(&wz);
    let live = wz.live;
    rsx! {
        dl { class: "oxs8-sum",
            for (l, v, s) in summary(&wz) {
                {
                    let miss = blocker(&wz, s).is_some() && gap == Some(s);
                    rsx! {
                        div { key: "{l}", class: "oxs8-sr", "data-miss": "{miss}",
                            dt { "{l}" }
                            dd { class: "oxs8-ell",
                                if l == "Name" && !wz.name.trim().is_empty() { {mark_el(&wz, 16)} }
                                span { class: "oxs8-ell", "{v}" }
                            }
                            dd { class: "oxs8-sre",
                                button { r#type: "button", class: "oxs8-edit", "aria-label": "Edit {l.to_lowercase()}", onclick: move |_| on_edit.call(s),
                                    if miss { "Add" } else { "Edit" } }
                            }
                        }
                    }
                }
            }
        }
        if let Some(g) = gap {
            p { class: "oxs8-note oxs8-warn", role: "status", "{STEPS[g].1} needs something first: {blocker(&wz, g).unwrap_or_default()}." }
        }
        if live {
            p { class: "oxs8-livenote", {nico("check")} span { "Your assistant is live. Share the chat link or add it to your site." } }
        }
        button { r#type: "button", class: "oxscn-primary oxs8-cont oxs8-open", disabled: gap.is_some(),
            onclick: move |_| { w.write().live = true; said.set("Assistant is live; opening the dashboard".to_string()); on_done.call(()); },
            "Open dashboard" {nico("next")} }
    }
}

// ─────────────────────────────────────────────────────────── Screens ──

macro_rules! s8_screen {
    ($(#[$m:meta])* $name:ident, $slug:literal) => {
        $(#[$m])*
        #[component]
        pub fn $name(
            /// `inset` or `flat`: the dashboard picture as a rounded panel or edge to edge.
            #[props(default = "inset".to_string())]
            layout: String,
            /// A state key (`sample`, `loading`, `empty` or the screen's own state).
            #[props(default = "sample".to_string())]
            state: String,
            /// Height px (outside a stage).
            #[props(default = 900)]
            height: u32,
            /// Oxyn: name each step under its dash.
            #[props(default)]
            named_steps: bool,
            /// Oxyn: the form alone, without the dashboard preview.
            #[props(default)]
            form_only: bool,
            /// Open dashboard pressed (Ready).
            #[props(default)]
            on_done: EventHandler<()>,
            /// Log out pressed.
            #[props(default)]
            on_logout: EventHandler<()>,
        ) -> Element {
            rsx! { Wizard { slug: $slug.to_string(), state, layout, height, named: named_steps, form_only, on_done, on_logout } }
        }
    };
}

s8_screen!(
    /// Onboarding / Website: the sites the assistant learns from. Typed addresses are cleaned up
    /// (no https://, www. or paths) and checked; each one is read with progress in the preview's
    /// Knowledge table, private ones fail with Retry. State `fetching`.
    OnboardingWebsiteScreen,
    "onboarding-website"
);
s8_screen!(
    /// Onboarding / Brand: the company name and short forms, a brand colour (swatches or a checked
    /// hex, with a contrast warning) and a mark (initials, square, round or an uploaded logo). The
    /// preview glides to the assistant card. State `custom`.
    OnboardingBrandScreen,
    "onboarding-brand"
);
s8_screen!(
    /// Onboarding / Tone: what the assistant is called, how it sounds, how long it answers and who it
    /// hands over to; the preview's test chat re-types its reply in the new voice. State `testing`.
    OnboardingToneScreen,
    "onboarding-tone"
);
s8_screen!(
    /// Onboarding / Ready: a summary with Edit links (a missing step shows Add and blocks), then Open
    /// dashboard turns the assistant from Draft to Live. State `live`.
    OnboardingReadyScreen,
    "onboarding-ready"
);
