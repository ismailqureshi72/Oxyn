//! Secret card, Split confirm and Agent prompt copy (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::xpost::promptcopy::{AGENTS, FORMATS, PromptCopy};
use oxyn_core::xpost::secret::{Ask, SCOPES, SECRET, Secret, USED_BY};
use oxyn_core::xpost::split::{Phase as SpPhase, Split};

use super::{cls, ico, key_of, live, mark, owned};

fn ago(min: u64) -> String {
    if min >= 1_440 {
        format!("{}d ago", min / 1_440)
    } else if min >= 60 {
        format!("{}h ago", min / 60)
    } else {
        format!("{min} min ago")
    }
}

/// Secret card: an API key the agent may use but never sees. The value stays masked to its last
/// four characters and the chat only ever gets a handle. Show needs a confirm and hides itself
/// after five seconds; Rotate swaps the key with a short progress; Revoke pauses every routine
/// that used it, with Undo for a few seconds.
#[component]
pub fn SecretCard(
    /// Host clock.
    now: ClockMs,
    /// Start revoked.
    #[props(default)]
    revoked: bool,
    /// Show the chat line where the agent refers to the key by handle.
    #[props(default = true)]
    show_chat: bool,
    /// Hide scopes and routines.
    #[props(default)]
    compact: bool,
) -> Element {
    let mut st = use_signal(move || Secret::new(revoked));
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let rot = s.rotation(n);
    let off = s.is_revoked();
    let copied = s.copied.is_some_and(|c| n < c + 1_500);
    let handle = Secret::handle();
    rsx! {
        section { class: "oxxp oxxp-sc", "aria-label": "Secret {SECRET.0}", "data-off": "{off}",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if let Some(x) = st.write().key(&k, n) { e.prevent_default(); said.set(x); }
            },
            if show_chat {
                div { class: "oxxp-sc-chat",
                    span { class: "oxxp-sc-av", "aria-hidden": "true", {ico("agents", 12.0)} }
                    p { "I'll issue the refund with " code { class: "oxxp-sc-handle", {ico("lock", 10.0)} "{handle}" } ". The key itself never enters this chat." }
                }
            }
            div { class: "oxxp-sc-card",
                div { class: "oxxp-row oxxp-between",
                    span { class: "oxxp-row oxxp-sc-name",
                        span { class: "oxxp-sc-tile", {mark(SECRET.1, 18.0)} }
                        span { class: "oxxp-col", b { class: "oxxp-mono", "{SECRET.0}" } small { class: "oxxp-mute", "Stripe · live mode" } }
                    }
                    span { class: "oxxp-pill", "data-hue": if off { "red" } else if rot.is_some() { "amber" } else { "green" },
                        if off { "Revoked" } else if rot.is_some() { "Rotating" } else { "Active" } }
                }
                div { class: "oxxp-sc-val", "data-shown": "{s.shown(n)}",
                    code { class: "oxxp-mono", "aria-label": if s.shown(n) { "Key value" } else { "Key, hidden except the last four characters" }, "{s.value(n)}" }
                    if !off {
                        button { class: "oxxp-ib", r#type: "button", "aria-label": if s.shown(n) { "Hide key" } else { "Show key" }, onclick: move |_| act("sc-reveal".into()),
                            {ico(if s.shown(n) { "eye-off" } else { "eye" }, 13.0)} }
                    }
                }
                if let Some(r) = rot { div { class: "oxxp-bar", "data-hue": "amber", span { style: "width:{r * 100.0:.0}%" } } }
                if s.ask != Ask::None {
                    div { class: "oxxp-sc-ask oxxp-in", role: "alertdialog", "aria-label": "Confirm",
                        span { class: "oxxp-row", {ico("alert", 12.0)}
                            match s.ask { Ask::Reveal => "Show the full key on this screen for 5 seconds?", Ask::Rotate => "Rotate now? Routines switch to the new key when it's ready.", _ => "Revoke? Both routines that use it will pause." }
                        }
                        div { class: "oxxp-ctl",
                            button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("sc-cancel".into()), "Cancel" }
                            button { class: if s.ask == Ask::Revoke { "oxxp-btn sm danger" } else { "oxxp-btn sm pri" }, r#type: "button", onclick: move |_| act("sc-confirm".into()),
                                match s.ask { Ask::Reveal => "Show", Ask::Rotate => "Rotate", _ => "Revoke" } }
                        }
                    }
                }
                if !compact {
                    div { class: "oxxp-sc-scopes", role: "list", "aria-label": "Scopes",
                        for (i, sc) in SCOPES.iter().enumerate() { span { key: "{i}", role: "listitem", class: "oxxp-pill", "{sc}" } }
                    }
                    ul { class: "oxxp-sc-used", "aria-label": "Used by",
                        for (i, (r, m)) in USED_BY.iter().enumerate() {
                            li { key: "{i}", class: "oxxp-row oxxp-between",
                                span { class: "oxxp-row", span { class: "oxxp-dot", "data-hue": if off { "amber" } else { "green" } } "{r}" }
                                span { class: "oxxp-mute", if off { "Paused" } else { "{ago(*m)}" } }
                            }
                        }
                    }
                }
                div { class: "oxxp-row oxxp-between oxxp-sc-foot",
                    span { class: "oxxp-mute oxxp-sc-st", "{s.status(n)}" }
                    div { class: "oxxp-ctl",
                        if off {
                            if s.can_undo(n) { button { class: "oxxp-btn sm pri", r#type: "button", onclick: move |_| act("sc-undo".into()), {ico("undo", 12.0)} "Undo" } }
                            else { button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("sc-restore".into()), "Restore" } }
                        } else {
                            button { class: "oxxp-btn sm", r#type: "button", onclick: move |_| act("sc-copy".into()), {ico(if copied { "check" } else { "copy" }, 12.0)} if copied { "Copied" } else { "Copy handle" } }
                            button { class: "oxxp-btn sm", r#type: "button", disabled: rot.is_some(), onclick: move |_| act("sc-rotate".into()), {ico("retry", 12.0)} "Rotate" }
                            button { class: "oxxp-btn sm danger", r#type: "button", onclick: move |_| act("sc-revoke".into()), "Revoke" }
                        }
                    }
                }
            }
            {live(&said())}
        }
    }
}

/// Split confirm: a pill that confirms by splitting. Pressed, it stretches with a small
/// overshoot, a round check bubble pinches off its end and the label turns to the past tense.
/// The bubble is the Undo for four seconds. A failure shows a cross bubble, shakes once and offers
/// Try again.
#[component]
pub fn SplitConfirm(
    /// Host clock.
    now: ClockMs,
    /// Label pair (0 Approve, 1 Send, 2 Merge).
    #[props(default)]
    label: usize,
    /// The action fails.
    #[props(default)]
    fail: bool,
    /// `md` or `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// A context line above the button.
    #[props(default = true)]
    context: bool,
) -> Element {
    let mut st = use_signal(move || Split::new(label, fail));
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
        SpPhase::Idle => "idle",
        SpPhase::Stretch(_) => "stretch",
        SpPhase::Split(_) => "split",
        SpPhase::Done => "done",
        SpPhase::Failed => "failed",
    };
    let bub = s.bubble(n);
    let ctx = [
        "Refund $84.00 to Ada Lin",
        "Reply to the vendor thread",
        "PR #214 into main",
    ][s.label];
    rsx! {
        div { class: "oxxp-spw", "data-size": "{size}",
            if context { p { class: "oxxp-mute oxxp-sp-ctx", "{ctx}" } }
            div { class: "oxxp-sp", "data-ph": "{ph_name}",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if owned(&k) || k == "Backspace" { e.prevent_default(); }
                    if let Some(x) = st.write().key(&k, n) { said.set(x); }
                },
                button { class: "oxxp-sp-pill", r#type: "button", style: "--sx:{s.scale(n):.3}",
                    "aria-disabled": "{ph != SpPhase::Idle && ph != SpPhase::Failed}",
                    onclick: move |_| act(if ph == SpPhase::Failed { "sp-retry".into() } else { "sp-go".into() }),
                    span { class: "oxxp-sp-txt", key: "{s.text(n)}", "{s.text(n)}" }
                    if ph == SpPhase::Failed { span { class: "oxxp-sp-retry", "Try again" } }
                }
                if let Some((off, sz)) = bub {
                    button { class: "oxxp-sp-bub", r#type: "button", style: "--off:{off:.3};--sz:{sz:.3}",
                        "aria-label": if ph == SpPhase::Failed { "Failed" } else if s.can_undo(n) { "Undo" } else { "Done" },
                        disabled: !s.can_undo(n),
                        onclick: move |_| act("sp-undo".into()),
                        {ico(if ph == SpPhase::Failed { "x" } else { "check" }, 14.0)}
                    }
                }
            }
            if s.can_undo(n) { p { class: "oxxp-mute oxxp-sp-hint", "Tap the check to undo" } }
            {live(&said())}
        }
    }
}

/// Agent prompt copy: a "Give to your agent" split button. The main part copies an agent-ready
/// hand-off for the chosen coding agent; the chevron opens a menu of agents (official marks) and
/// formats (prompt, skill file, install command). The preview shows exactly what will be copied
/// and the button turns into a check with the agent's name for a moment.
#[component]
pub fn AgentPromptCopy(
    /// Host clock.
    now: ClockMs,
    /// Starting agent (index into the agents list).
    #[props(default)]
    agent: usize,
    /// Starting format (0 prompt, 1 skill file, 2 install command).
    #[props(default)]
    format: usize,
    /// Show the preview of what will be copied.
    #[props(default = true)]
    preview: bool,
    /// The clipboard refuses (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let mut st = use_signal(move || PromptCopy::new(agent, format, fail));
    let mut said = use_signal(String::new);
    let n = now.0;
    let c = st.read().clone();
    let mut act = move |id: String| {
        if let Some(x) = st.write().press(&id, n) {
            said.set(x);
        }
    };
    let copied = c.copied(n);
    let (aname, amark, _) = AGENTS[c.agent];
    let text = c.text();
    let menu = c.menu;
    rsx! {
        section { class: "oxxp oxxp-ga", "aria-label": "Give to your agent",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if owned(&k) && menu.is_some() { e.prevent_default(); }
                if k == "ArrowDown" { e.prevent_default(); }
                if let Some(x) = st.write().key(&k, n) { said.set(x); }
            },
            div { class: "oxxp-row oxxp-between",
                div { class: "oxxp-hl",
                    h3 { class: "oxxp-h", "Test ticks" }
                    p { class: "oxxp-sub", "Hand this component to your coding agent" }
                }
                div { class: "oxxp-ga-split", "data-copied": "{copied}", "data-fail": "{copied && c.fail}",
                    button { class: "oxxp-ga-main", r#type: "button", onclick: move |_| act("ga-copy".into()),
                        if copied && c.fail { {ico("alert", 13.0)} span { "Copy blocked" } }
                        else if copied { {ico("check", 13.0)} span { class: "oxxp-in", "Copied for {aname}" } }
                        else { {mark(amark, 14.0)} span { "Give to your agent" } }
                    }
                    button { class: "oxxp-ga-chev", r#type: "button", "aria-label": "Choose agent and format", "aria-haspopup": "menu", "aria-expanded": "{menu.is_some()}",
                        onclick: move |_| act("ga-menu".into()), {ico("chevron", 13.0)} }
                    if let Some(m) = menu {
                        div { class: "oxxp-menu oxxp-in", role: "menu", "aria-label": "Agent and format",
                            div { class: "oxxp-menu-h", "Agent" }
                            for (i, (name, mk, _)) in AGENTS.iter().enumerate() {
                                button { key: "a{i}", r#type: "button", role: "menuitemradio", "aria-checked": "{c.agent == i}", class: cls("oxxp-mi", m == i),
                                    onclick: move |_| act(format!("ga-agent-{i}")),
                                    {mark(mk, 14.0)} span { "{name}" } if c.agent == i { {ico("check", 12.0)} } }
                            }
                            div { class: "oxxp-menu-h", "Format" }
                            for (i, f) in FORMATS.iter().enumerate() {
                                button { key: "f{i}", r#type: "button", role: "menuitemradio", "aria-checked": "{c.format == i}", class: cls("oxxp-mi", m == AGENTS.len() + i),
                                    onclick: move |_| act(format!("ga-format-{i}")),
                                    {ico(["prompt", "file", "code"][i], 13.0)} span { "{f}" } if c.format == i { {ico("check", 12.0)} } }
                            }
                        }
                    }
                }
            }
            if preview {
                div { class: "oxxp-ga-pre",
                    div { class: "oxxp-row oxxp-between oxxp-mute",
                        span { class: "oxxp-row", {mark(amark, 12.0)} "{aname} · {FORMATS[c.format]}" }
                        span { "{text.len()} characters" }
                    }
                    pre { class: "oxxp-mono", tabindex: "0", "aria-label": "What will be copied", "{text}" }
                }
            }
            {live(&said())}
        }
    }
}
