//! Proactive card, Write sweep, Done shockwave and Publish popover (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::xpost::proactive::{
    OFFER, Phase as PcPhase, Proactive, RESULT, SNOOZE, STEPS as PC_STEPS,
};
use oxyn_core::xpost::publish::{ACCESS, DOC, HOSTS, Phase as PpPhase, Publish, STEPS as PP_STEPS};
use oxyn_core::xpost::shock::{Phase as SwPhase, STEPS as SW_STEPS, Shock, TASK as SW_TASK};
use oxyn_core::xpost::sweep::{ASK, ROWS, Sweep};

use super::{cls, ico, key_of, live, mark, owned};

/// Proactive card: the assistant drops an offer into a feed when it notices something. Accept
/// runs the steps in place and resolves into a result with Review and Dismiss; Not now opens a
/// snooze menu and folds the card into one line with Undo; the cross removes it.
#[component]
pub fn ProactiveCard(
    /// Host clock.
    now: ClockMs,
    /// The second step fails (error scenario with Try again).
    #[props(default)]
    fail: bool,
    /// Show the card between two feed posts.
    #[props(default = true)]
    in_feed: bool,
) -> Element {
    let mut st = use_signal(move || Proactive::new(fail));
    let mut said = use_signal(String::new);
    let n = now.0;
    let p = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let ph = p.phase(n);
    let done = p.steps_done(n);
    let post = |who: &'static str, txt: &'static str, t: &'static str| {
        rsx! {
            div { class: "oxxp-pc-post", "aria-hidden": "true",
                span { class: "oxxp-pc-ava", "{&who[..1]}" }
                div { class: "oxxp-col", span { class: "oxxp-row", b { "{who}" } span { class: "oxxp-mute", "{t}" } } p { "{txt}" } }
            }
        }
    };
    rsx! {
        section { class: "oxxp oxxp-pc", "aria-label": "Feed with a suggestion",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if p.menu.is_some() && (owned(&k) || k == "Enter") { e.prevent_default(); }
                if let Some(x) = st.write().key(&k, n) { said.set(x); }
            },
            if in_feed { {post("Mira Holt", "Shipped the new onboarding flow today. Small change, big drop in support tickets.", "2h")} }
            match ph {
                PcPhase::Gone => rsx! {
                    div { class: "oxxp-pc-fold oxxp-in", span { class: "oxxp-mute", "Suggestion removed" } button { class: "oxxp-link", r#type: "button", onclick: move |_| act("pc-undo".into()), "Undo" } }
                },
                PcPhase::Snoozed => rsx! {
                    div { class: "oxxp-pc-fold oxxp-in", span { class: "oxxp-row oxxp-mute", {ico("clock", 12.0)} "{p.folded()}" } button { class: "oxxp-link", r#type: "button", onclick: move |_| act("pc-undo".into()), "Undo" } }
                },
                _ => rsx! {
                    article { class: "oxxp-pc-card oxxp-in", "data-ph": match ph { PcPhase::Offer => "offer", PcPhase::Working => "working", PcPhase::Result => "result", _ => "failed" }, "aria-label": "Suggestion: {OFFER.2}",
                        div { class: "oxxp-row oxxp-between",
                            span { class: "oxxp-row oxxp-mute", span { class: "oxxp-pc-spark", {ico("spark", 12.0)} } "Assistant · " {mark(OFFER.1, 12.0)} "{OFFER.0}" }
                            button { class: "oxxp-ib", r#type: "button", "aria-label": "Remove suggestion", onclick: move |_| act("pc-close".into()), {ico("x", 12.0)} }
                        }
                        h4 { class: "oxxp-pc-t", "{OFFER.2}" }
                        if ph == PcPhase::Offer { p { class: "oxxp-pc-b", "{OFFER.3}" } }
                        if matches!(ph, PcPhase::Working | PcPhase::Failed) {
                            ol { class: "oxxp-pc-steps", "aria-busy": "{ph == PcPhase::Working}",
                                for (i, (doing, did)) in PC_STEPS.iter().enumerate() {
                                    if i <= done {
                                        li { key: "{i}", class: "oxxp-in", "data-st": if ph == PcPhase::Failed && i == done.min(1) { "fail" } else if i < done { "done" } else { "run" },
                                            span { class: "oxxp-pc-ic", {ico(if ph == PcPhase::Failed && i == done.min(1) { "x-circle" } else if i < done { "check" } else { "clock" }, 12.0)} }
                                            if ph == PcPhase::Failed && i == done.min(1) { "Couldn't open the billing page" } else if i < done { "{did}" } else { span { class: "oxxp-shim", "{doing}" } }
                                        }
                                    }
                                }
                            }
                            if ph == PcPhase::Working { div { class: "oxxp-bar", span { style: "width:{p.progress(n) * 100.0:.0}%" } } }
                        }
                        if ph == PcPhase::Result {
                            div { class: "oxxp-pc-res",
                                b { "data-hue": "green", "{RESULT.0}" }
                                p { class: "oxxp-mute", "{RESULT.1}" }
                                if p.reviewing {
                                    blockquote { class: "oxxp-pc-draft oxxp-in", "Hi Fernpoint billing team, please move us to the Starter plan from the next cycle. We use 3 seats. Thanks, Quillworks." }
                                }
                            }
                        }
                        div { class: "oxxp-row oxxp-pc-acts",
                            match ph {
                                PcPhase::Offer => rsx! {
                                    button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("pc-accept".into()), "Compare and draft" }
                                    div { class: "oxxp-pc-later",
                                        button { class: "oxxp-btn sm", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{p.menu.is_some()}", onclick: move |_| act("pc-later".into()), "Not now" }
                                        if let Some(m) = p.menu {
                                            div { class: "oxxp-menu oxxp-in", role: "menu", "aria-label": "Snooze",
                                                for (i, s) in SNOOZE.iter().enumerate() {
                                                    button { key: "{i}", r#type: "button", role: "menuitem", class: cls("oxxp-mi", m == i), onclick: move |_| act(format!("pc-snooze-{i}")), "{s}" }
                                                }
                                            }
                                        }
                                    }
                                },
                                PcPhase::Working => rsx! { span { class: "oxxp-mute", "Working on it · you can keep scrolling" } },
                                PcPhase::Failed => rsx! {
                                    button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("pc-retry".into()), {ico("retry", 12.0)} "Try again" }
                                },
                                _ => rsx! {
                                    button { class: "oxxp-btn sm pri", r#type: "button", "aria-expanded": "{p.reviewing}", onclick: move |_| act("pc-review".into()), if p.reviewing { "Hide draft" } else { "Review draft" } }
                                    button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("pc-close".into()), "Dismiss" }
                                },
                            }
                        }
                    }
                },
            }
            if in_feed { {post("Owen Park", "Anyone else moving their docs to plain markdown this year?", "3h")} }
            {live(&said())}
        }
    }
}

/// Write sweep: an "agent is writing here" pass. A soft band of light sweeps down the region being
/// changed; each row it crosses blurs out its old value and settles into the new one. Afterwards
/// the footer counts the changes with Undo (Cmd/Ctrl+Z), and changed rows keep a marker.
#[component]
pub fn WriteSweep(
    /// Host clock.
    now: ClockMs,
    /// Ms the band takes top to bottom.
    #[props(default = 1_800)]
    duration_ms: u64,
    /// Start on its own after a moment.
    #[props(default = true)]
    autoplay: bool,
    /// `aurora` (a soft colour band) or `mono` (a grey band).
    #[props(default = "aurora".to_string())]
    tone: String,
    /// Show the old value struck through beside changed rows.
    #[props(default)]
    show_before: bool,
) -> Element {
    let t0 = now.0;
    let mut st = use_signal(move || Sweep::new(duration_ms, autoplay, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let band = s.band(n);
    let done = s.done(n);
    rsx! {
        section { class: "oxxp oxxp-ws", "aria-label": "Training plan", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if owned(&k) || k == "Enter" { e.prevent_default(); }
                if let Some(x) = st.write().key(&k, n) { said.set(x); }
            },
            header { class: "oxxp-hd",
                div { class: "oxxp-hl",
                    h3 { class: "oxxp-h", "Week 3 · Half marathon plan" }
                    p { class: "oxxp-sub", {ico("spark", 12.0)} "\u{201c}{ASK}\u{201d}" }
                }
                if band.is_some() { span { class: "oxxp-pill oxxp-ws-tag", "data-hue": "blue", span { class: "oxxp-ws-caret" } "Agent is writing" } }
            }
            div { class: "oxxp-ws-area", "data-tone": "{tone}", "aria-busy": "{band.is_some()}",
                if let Some(b) = band { i { class: "oxxp-ws-band", "aria-hidden": "true", style: "--p:{b:.3}" } }
                for (i, (day, before, after)) in ROWS.iter().enumerate() {
                    div { key: "{i}", class: "oxxp-ws-row", "data-changed": "{before != after && s.crossed(i, n)}",
                        "data-settle": "{s.since(i, n).is_some_and(|x| x < 500)}",
                        span { class: "oxxp-ws-day", "{day}" }
                        span { class: "oxxp-ws-val", key: "{s.value(i, n)}", "{s.value(i, n)}" }
                        if show_before && before != after && s.crossed(i, n) { s { class: "oxxp-mute oxxp-ws-was", "{before}" } }
                    }
                }
            }
            footer { class: "oxxp-row oxxp-between",
                span { class: "oxxp-mute", "{s.status(n)}" }
                div { class: "oxxp-ctl",
                    if done { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("ws-undo".into()), {ico("undo", 12.0)} "Undo" } }
                    else if s.undone { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("ws-redo".into()), "Redo" } }
                    button { class: "oxxp-btn sm pri", r#type: "button", disabled: band.is_some(), onclick: move |_| act("ws-run".into()), {ico("spark", 12.0)} if done || s.undone { "Rewrite" } else { "Update plan" } }
                }
            }
            {live(&said())}
        }
    }
}

/// Done shockwave: when a long task finishes, two rings leave its status mark and roll across the
/// panel, the mark turns into a check and a line says what finished and how long it took. A
/// failure sends one short red pulse instead. Reduced motion swaps the rings for a still halo.
#[component]
pub fn DoneShockwave(
    /// Host clock.
    now: ClockMs,
    /// The task fails.
    #[props(default)]
    fail: bool,
    /// Ring strength 0..1.
    #[props(default = 1.0)]
    strength: f64,
    /// Speed factor for the task.
    #[props(default = 1.0)]
    speed: f64,
    /// Start on its own.
    #[props(default = true)]
    autoplay: bool,
    /// A still halo instead of moving rings.
    #[props(default)]
    reduced: bool,
) -> Element {
    let t0 = now.0;
    let mut st = use_signal(move || Shock::new(fail, strength, speed, autoplay, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let ph = s.phase(n);
    let ph_name = match ph {
        SwPhase::Idle => "idle",
        SwPhase::Running => "running",
        SwPhase::Done => "done",
        SwPhase::Failed => "failed",
    };
    let rings = if reduced { vec![] } else { s.rings(n) };
    let halo = reduced && s.since_end(n).is_some_and(|e| e < 1_200);
    rsx! {
        section { class: "oxxp oxxp-sw", "data-ph": "{ph_name}", "aria-label": "Deploy task",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if k == "Space" { e.prevent_default(); }
                if let Some(x) = st.write().key(&k, n) { said.set(x); }
            },
            div { class: "oxxp-sw-stage", "aria-hidden": "true",
                for (k, (r, a)) in rings.iter().enumerate() {
                    i { key: "{k}", class: "oxxp-sw-ring", style: "--r:{r:.3};--a:{a:.3}" }
                }
                if halo { i { class: "oxxp-sw-halo" } }
            }
            div { class: "oxxp-sw-card",
                span { class: "oxxp-sw-mark", "data-ph": "{ph_name}",
                    {ico(match ph { SwPhase::Done => "check", SwPhase::Failed => "x", SwPhase::Running => "bolt", _ => "play" }, 16.0)}
                }
                div { class: "oxxp-col oxxp-sw-txt",
                    b { "{SW_TASK.0}" }
                    span { class: "oxxp-mute", "{SW_TASK.1}" }
                }
                div { class: "oxxp-ctl",
                    match ph {
                        SwPhase::Running => rsx! { span { class: "oxxp-num oxxp-mute", "{s.progress(n) * 100.0:.0}%" } },
                        SwPhase::Failed => rsx! { button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("sw-retry".into()), "Retry" } },
                        SwPhase::Done => rsx! { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("sw-run".into()), "Deploy again" } },
                        SwPhase::Idle => rsx! { button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("sw-run".into()), "Deploy" } },
                    }
                }
            }
            if ph == SwPhase::Running || ph == SwPhase::Failed {
                div { class: "oxxp-bar", "data-hue": if ph == SwPhase::Failed { "red" } else { "neutral" }, span { style: "width:{s.progress(n) * 100.0:.1}%" } }
            }
            ol { class: "oxxp-sw-steps",
                for (i, st_) in SW_STEPS.iter().enumerate() {
                    li { key: "{i}", "data-st": if ph == SwPhase::Done || (ph != SwPhase::Idle && i < s.step(n)) { "done" } else if ph == SwPhase::Failed && i == s.step(n) { "fail" } else if ph == SwPhase::Running && i == s.step(n) { "run" } else { "todo" }, "{st_}" }
                }
            }
            p { class: "oxxp-sw-st", role: if ph == SwPhase::Failed { "alert" } else { "none" }, key: "{ph_name}", "{s.status(n)}" }
            {live(&said())}
        }
    }
}

/// Publish popover: Publish in a document toolbar opens a popover that slides out of the button:
/// the page address (editable), who can open it, a host (with its official mark) and two
/// switches. Publishing walks through upload, build and live; once live it shows the link with
/// Copy, a view count and Unpublish. A taken address offers a free one.
#[component]
pub fn PublishPopover(
    /// Host clock.
    now: ClockMs,
    /// Start open.
    #[props(default = true)]
    open: bool,
    /// The first address is taken (error scenario).
    #[props(default)]
    taken: bool,
) -> Element {
    let mut st = use_signal(move || Publish::new(open, taken));
    let mut said = use_signal(String::new);
    let n = now.0;
    let p = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let ph = p.phase(n);
    let live_now = p.live(n);
    let copied = p.copied.is_some_and(|c| n < c + 1_500);
    let keyed = move |part: &'static str| {
        move |e: KeyboardEvent| {
            let k = key_of(&e);
            if owned(&k) {
                e.prevent_default();
            }
            if let Some(x) = st.write().key(part, &k, n) {
                said.set(x);
            }
        }
    };
    rsx! {
        section { class: "oxxp oxxp-pp", "aria-label": "Document",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if k == "Escape" { if let Some(x) = st.write().key("", &k, n) { said.set(x); } }
            },
            div { class: "oxxp-pp-bar",
                div { class: "oxxp-hl", b { "{DOC.0}" } span { class: "oxxp-mute", "{DOC.1}" } }
                div { class: "oxxp-pp-anchor",
                    button { class: if live_now { "oxxp-btn sm" } else { "oxxp-btn sm pri" }, r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{p.open}",
                        onclick: move |_| act("pp-toggle".into()),
                        if live_now { span { class: "oxxp-dot", "data-hue": "green" } "Live" } else { {ico("globe", 12.0)} "Publish" }
                    }
                    if p.open {
                        div { class: "oxxp-pp-pop", role: "dialog", "aria-label": "Publish to the web", "data-ph": match ph { PpPhase::Live => "live", PpPhase::Publishing(_) => "busy", PpPhase::Taken => "taken", _ => "form" },
                            div { class: "oxxp-row oxxp-between",
                                b { if live_now { "Published" } else { "Publish to the web" } }
                                button { class: "oxxp-ib", r#type: "button", "aria-label": "Close", onclick: move |_| act("pp-close".into()), {ico("x", 12.0)} }
                            }
                            if live_now {
                                div { class: "oxxp-pp-link",
                                    {mark(HOSTS[p.host].1, 14.0)}
                                    span { class: "oxxp-mono", "{p.url()}" }
                                    button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("pp-copy".into()), {ico(if copied { "check" } else { "copy" }, 12.0)} if copied { "Copied" } else { "Copy" } }
                                }
                                p { class: "oxxp-row oxxp-mute", {ico("eye", 12.0)} b { class: "oxxp-num", key: "{p.views(n)}", "{p.views(n)}" } " views · {ACCESS[p.access]}" }
                                div { class: "oxxp-row oxxp-between",
                                    a { class: "oxxp-link", href: "#", onclick: move |e: MouseEvent| e.prevent_default(), {ico("external", 12.0)} "Open page" }
                                    button { class: "oxxp-btn sm danger", r#type: "button", onclick: move |_| act("pp-unpublish".into()), "Unpublish" }
                                }
                            } else {
                                label { class: "oxxp-pp-lab", r#for: "oxxp-pp-slug", "Address" }
                                div { class: "oxxp-pp-url", "data-bad": "{ph == PpPhase::Taken}",
                                    span { class: "oxxp-mute oxxp-mono", "{oxyn_core::xpost::publish::BASE}" }
                                    input { id: "oxxp-pp-slug", class: "oxxp-mono", value: "{p.slug}", "aria-invalid": "{ph == PpPhase::Taken}", disabled: matches!(ph, PpPhase::Publishing(_)),
                                        oninput: move |e: FormEvent| st.write().set_slug(&e.value()),
                                        onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Enter" { e.prevent_default(); act("pp-go".into()); } } }
                                }
                                if ph == PpPhase::Taken { p { class: "oxxp-pp-err", role: "alert", {ico("alert", 12.0)} "That address is taken. Publish as {p.slug}-2?" } }
                                span { class: "oxxp-pp-lab", "Who can open it" }
                                div { class: "oxxp-seg", role: "radiogroup", "aria-label": "Who can open it", onkeydown: keyed("access"),
                                    for (i, a) in ACCESS.iter().enumerate() {
                                        button { key: "{i}", r#type: "button", role: "radio", class: cls("oxxp-sg", p.access == i), "aria-checked": "{p.access == i}", tabindex: if p.access == i { "0" } else { "-1" },
                                            onclick: move |_| act(format!("pp-access-{i}")), "{a}" }
                                    }
                                }
                                span { class: "oxxp-pp-lab", "Host" }
                                div { class: "oxxp-seg", role: "radiogroup", "aria-label": "Host", onkeydown: keyed("host"),
                                    for (i, (h, mk)) in HOSTS.iter().enumerate() {
                                        button { key: "{i}", r#type: "button", role: "radio", class: cls("oxxp-sg", p.host == i), "aria-checked": "{p.host == i}", tabindex: if p.host == i { "0" } else { "-1" },
                                            onclick: move |_| act(format!("pp-host-{i}")), {mark(mk, 12.0)} "{h}" }
                                    }
                                }
                                div { class: "oxxp-pp-sw",
                                    button { r#type: "button", role: "switch", class: cls("oxxp-switch", p.index), "aria-checked": "{p.index}", onclick: move |_| act("pp-index".into()), span { class: "oxxp-knob" } "Let search engines find it" }
                                    button { r#type: "button", role: "switch", class: cls("oxxp-switch", p.history), "aria-checked": "{p.history}", onclick: move |_| act("pp-history".into()), span { class: "oxxp-knob" } "Show the agent's edits" }
                                }
                                if let PpPhase::Publishing(i) = ph {
                                    div { class: "oxxp-pp-busy", "aria-busy": "true",
                                        div { class: "oxxp-bar", span { style: "width:{p.progress(n) * 100.0:.0}%" } }
                                        span { class: "oxxp-shim", key: "{i}", "{PP_STEPS[i].0}…" }
                                    }
                                } else {
                                    button { class: "oxxp-btn pri oxxp-pp-go", r#type: "button", disabled: p.slug.is_empty(), onclick: move |_| act("pp-go".into()),
                                        if ph == PpPhase::Taken { "Publish as {p.slug}-2" } else { "Publish" } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxxp-pp-doc", "aria-hidden": "true",
                h4 { "Goals" }
                p { "Ship the beta to 40 teams, keep weekly churn under 2% and close the onboarding gaps found in the October interviews." }
                p { "Owners: Ada, Kenji, Sofia." }
            }
            {live(&said())}
        }
    }
}
