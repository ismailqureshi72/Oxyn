//! Composer parts (plan #27, chunk 3): `VoiceRecorder`, `DictationButton`, `MentionInput`,
//! `ModeSwitch`, `PermissionSelector`, `ToolsMenu`, `QueueStack` and `AgentSteering`.
//!
//! Logic lives in `oxyn_core::composer::parts`. Time comes from the host: pass `now_ms` (the
//! host clock in milliseconds) and the parts read their timers from it, so the same markup renders
//! on the server and plays live in the browser. Styles are in [`PARTS_CSS`]. Behaviour follows
//! Arc UI (Voice recorder, Dictation button, Mention input) and affectUI (Agent Steering), rebuilt
//! from their docs; no source used.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::parts::{
        Applied, Choice, DictPhase, Mode, ModeSwitch as ModeModel, Permissions, Preset, Queue,
        RecPhase, Recorder, Risk, Steer, SteerPhase, Suggestion, Tool, Trigger, accept,
        active_token, default_modes, default_presets, insert_dictation, interim, levels, mmss,
        pills, rank, toggle_group, toggle_tool, tool_groups,
    },
};

use super::icons::Icon;

/// Stylesheet for the composer parts.
pub const PARTS_CSS: &str = include_str!("parts.css");

fn cls(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

// ─────────────────────────────────────── Voice recorder ──

/// Voice recorder: mic → permission → recording (live level bars, timer, pause) → review
/// (play, discard, send). Stops itself at `max_s`; the last ten seconds count down.
#[component]
pub fn VoiceRecorder(
    /// Host clock (ms).
    now_ms: u64,
    /// Limit in seconds.
    #[props(default = 120)]
    max_s: u64,
    /// Starting phase (`idle` · `recording` · `paused` · `review` · `denied`), for demos.
    #[props(default = "idle".to_string())]
    phase: String,
    /// The permission prompt answers no.
    #[props(default)]
    deny: bool,
    /// Bars in the meter.
    #[props(default = 28)]
    bars: usize,
    /// Variant: `inline` (inside a composer row) or `card`.
    #[props(default = "inline".to_string())]
    variant: String,
    /// A recording was sent (its length in ms).
    #[props(default)]
    on_send: EventHandler<u64>,
) -> Element {
    let now = ClockMs(now_ms);
    let mut m = use_signal(|| {
        let mut r = Recorder::new(Duration::from_secs(max_s));
        match RecPhase::from_id(&phase) {
            RecPhase::Recording => {
                r.request();
                r.permission(true, ClockMs(now_ms.saturating_sub(4_200)));
            }
            RecPhase::Paused => {
                r.request();
                r.permission(true, ClockMs(now_ms.saturating_sub(12_600)));
                r.toggle_pause(ClockMs(now_ms));
            }
            RecPhase::Review => {
                r.request();
                r.permission(true, ClockMs(0));
                r.stop(ClockMs(18_400));
            }
            RecPhase::Denied => {
                r.request();
                r.permission(false, ClockMs(0));
            }
            _ => {}
        }
        r
    });
    let mut live = use_signal(String::new);
    let mut sent = use_signal(|| None::<u64>);
    if m.peek().phase == RecPhase::Recording && m.peek().elapsed(now) >= m.peek().max {
        // Reached the limit: the next render shows review (writes are deferred to an event).
    }
    let r = m.read().clone();
    let el = r.elapsed(now);
    let at_limit = r.phase == RecPhase::Recording && el >= r.max;
    let shown_phase = if at_limit { RecPhase::Review } else { r.phase };
    let lv = levels(
        if shown_phase == RecPhase::Recording {
            now_ms
        } else {
            0
        },
        bars,
        if shown_phase == RecPhase::Recording {
            1.0
        } else {
            0.35
        },
    );
    let near = r.near_limit(now) && !at_limit;
    let timer = if near {
        format!("−{}", mmss(r.max - el))
    } else {
        mmss(el)
    };
    rsx! {
        div { class: cls(&["oxp-rec", if variant == "card" { "card" } else { "" }, shown_phase.id()]), role: "group", "aria-label": "Voice message",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            match shown_phase {
                RecPhase::Idle | RecPhase::Denied => rsx! {
                    button { class: "p-mic", r#type: "button", "aria-label": "Record a voice message",
                        onclick: move |_| { m.write().request(); live.set("Waiting for microphone permission".into()); },
                        Icon { name: "mic", size: 16.0 }
                    }
                    if shown_phase == RecPhase::Denied {
                        div { class: "p-denied", role: "alert",
                            Icon { name: "ban", size: 13.0 }
                            span { "Microphone blocked. Allow it in your browser’s site settings, then try again." }
                        }
                    } else if let Some(ms) = sent() {
                        span { class: "p-hint", Icon { name: "check", size: 12.0, stroke: 2.0 } "Sent a {mmss(ms)} voice message" }
                    } else {
                        span { class: "p-hint", "Hold a thought? Record up to {mmss(max_s * 1000)}." }
                    }
                },
                RecPhase::Requesting => rsx! {
                    span { class: "p-req", span { class: "p-spin" } "Allow microphone access…" }
                    button { class: "oxyn-btn primary xs", r#type: "button", onclick: move |_| { m.write().permission(!deny, now); live.set(if deny { "Microphone blocked".into() } else { "Recording".into() }) }, if deny { "Answer prompt (Block)" } else { "Answer prompt (Allow)" } }
                    button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| m.write().discard(), "Cancel" }
                },
                RecPhase::Recording | RecPhase::Paused => rsx! {
                    button { class: "p-ic danger", r#type: "button", "aria-label": "Discard recording", onclick: move |_| { m.write().discard(); live.set("Recording discarded".into()) }, Icon { name: "trash", size: 14.0 } }
                    span { class: cls(&["p-dot", if shown_phase == RecPhase::Paused { "held" } else { "" }]), "aria-hidden": "true" }
                    span { class: cls(&["p-time", if near { "near" } else { "" }]), "aria-label": "Recorded {mmss(el)}", "{timer}" }
                    div { class: "p-bars", "aria-hidden": "true",
                        for (i, h) in lv.iter().enumerate() { i { key: "{i}", style: "--h:{h}" } }
                    }
                    button { class: "p-ic", r#type: "button", "aria-label": if shown_phase == RecPhase::Paused { "Resume" } else { "Pause" }, "aria-pressed": "{shown_phase == RecPhase::Paused}",
                        onclick: move |_| { m.write().toggle_pause(now); let p = m.peek().phase == RecPhase::Paused; live.set(if p { "Paused".into() } else { "Recording".into() }) },
                        Icon { name: if shown_phase == RecPhase::Paused { "play" } else { "pause" }, size: 14.0 }
                    }
                    button { class: "p-ic solid", r#type: "button", "aria-label": "Stop and review", onclick: move |_| { m.write().stop(now); live.set("Stopped. Review your recording".into()) }, Icon { name: "stop", size: 13.0 } }
                },
                RecPhase::Review => rsx! {
                    button { class: "p-ic", r#type: "button", "aria-label": "Discard recording", onclick: move |_| { m.write().discard(); live.set("Recording discarded".into()) }, Icon { name: "trash", size: 14.0 } }
                    span { class: "p-play", Icon { name: "play", size: 12.0 } }
                    div { class: "p-bars review", "aria-hidden": "true",
                        for (i, h) in levels(7_777, bars, 0.8).iter().enumerate() { i { key: "{i}", style: "--h:{h}" } }
                    }
                    span { class: "p-time", {mmss(r.elapsed(now).max(if at_limit { r.max } else { 0 }))} }
                    button { class: "p-ic solid", r#type: "button", "aria-label": "Send voice message",
                        onclick: move |_| { let mut w = m.write(); if w.phase == RecPhase::Recording { w.stop(now) } let ms = w.banked; w.discard(); drop(w); sent.set(Some(ms)); live.set(format!("Sent a {} voice message", mmss(ms))); on_send.call(ms) },
                        Icon { name: "up", size: 14.0, stroke: 2.0 }
                    }
                },
            }
        }
    }
}

// ─────────────────────────────────────────── Queue stack ──

/// Queue stack: follow-ups typed while the agent runs wait above the composer. One sends at the
/// end of each turn (`turn_ms`) unless paused; items can be edited, reordered (Alt+↑/↓), removed or
/// sent now.
#[component]
pub fn QueueStack(
    /// Queued texts.
    items: Vec<String>,
    /// The agent is running (turns end every `turn_ms`).
    #[props(default = true)]
    busy: bool,
    /// Turn length (ms).
    #[props(default = 6000)]
    turn_ms: u64,
    /// Start paused.
    #[props(default)]
    paused: bool,
    /// Collapse beyond this many rows (0 = never).
    #[props(default = 3)]
    max_rows: usize,
    /// An item was sent (`true` = interrupting now).
    #[props(default)]
    on_send: EventHandler<(String, bool)>,
) -> Element {
    let mut q = use_signal(|| {
        let v: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        let mut x = Queue::new(&v);
        x.paused = paused;
        x
    });
    let mut draft = use_signal(String::new);
    let mut edit_text = use_signal(String::new);
    let mut expanded = use_signal(|| false);
    let mut sent = use_signal(Vec::<String>::new);
    let mut live = use_signal(String::new);
    let st = q.read().clone();
    let n = st.items.len();
    let shown = if max_rows == 0 || expanded() {
        n
    } else {
        n.min(max_rows)
    };
    rsx! {
        div { class: "oxp-queue",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            if !sent().is_empty() {
                div { class: "qu-sent", for (i, s) in sent().iter().enumerate() { div { key: "{i}", class: "qu-msg", "{s}" } } }
            }
            div { class: "qu-run", span { class: cls(&["qu-dot", if busy { "busy" } else { "" }]) } if busy { "Agent is working · turn ends in a moment" } else { "Agent is idle" } }
            if busy && !st.paused && n > 0 {
                span { key: "{n}-{sent().len()}", class: "oxyn-timer", style: "animation-duration:{turn_ms}ms",
                    onanimationend: move |_| { if let Some(it) = q.write().turn_end() { live.set(format!("Sent: {}", it.text)); sent.write().push(it.text.clone()); on_send.call((it.text, false)) } }
                }
            }
            if n > 0 {
                div { class: "qu-stack",
                    div { class: "qu-h",
                        span { "Queued · {n}" }
                        button { class: "qu-link", r#type: "button", "aria-pressed": "{st.paused}", onclick: move |_| { let p = !q.peek().paused; q.write().paused = p; live.set(if p { "Queue paused".into() } else { "Queue resumed".into() }) },
                            Icon { name: if st.paused { "play" } else { "pause" }, size: 11.0 } if st.paused { "Resume" } else { "Pause" }
                        }
                    }
                    ol { class: "qu-list", "aria-label": "Queued messages",
                        for (i, it) in st.items.iter().take(shown).cloned().enumerate() {
                            li { key: "{it.id}", class: cls(&["qu-row", if i == 0 && !st.paused { "next" } else { "" }]), tabindex: "0",
                                onkeydown: { let id = it.id.clone(); move |e: KeyboardEvent| if e.modifiers().alt() { match e.key() { Key::ArrowUp => { e.prevent_default(); q.write().shift(&id, -1) } Key::ArrowDown => { e.prevent_default(); q.write().shift(&id, 1) } _ => {} } } },
                                span { class: "qu-i", "{i + 1}" }
                                if st.editing.as_deref() == Some(it.id.as_str()) {
                                    input { class: "qu-ed", r#type: "text", value: "{edit_text}", "aria-label": "Edit queued message",
                                        oninput: move |e| edit_text.set(e.value()),
                                        onkeydown: { let id = it.id.clone(); move |e: KeyboardEvent| { if e.key() == Key::Enter { e.prevent_default(); let t = edit_text.peek().clone(); q.write().edit(&id, &t) } else if e.key() == Key::Escape { e.prevent_default(); q.write().editing = None } } },
                                    }
                                } else {
                                    span { class: "qu-t", "{it.text}" }
                                }
                                span { class: "qu-act",
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Edit", onclick: { let id = it.id.clone(); let tx = it.text.clone(); move |_| { edit_text.set(tx.clone()); q.write().editing = Some(id.clone()) } }, Icon { name: "pencil", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Move up", disabled: i == 0, onclick: { let id = it.id.clone(); move |_| q.write().shift(&id, -1) }, Icon { name: "up", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Send now (interrupts)", onclick: { let id = it.id.clone(); move |_| { if let Some(x) = q.write().send_now(&id) { live.set(format!("Interrupted with: {}", x.text)); sent.write().push(x.text.clone()); on_send.call((x.text, true)) } } }, Icon { name: "bolt", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Remove", onclick: { let id = it.id.clone(); move |_| { q.write().remove(&id); live.set("Removed from queue".into()) } }, Icon { name: "x", size: 12.0 } }
                                }
                            }
                        }
                    }
                    if shown < n { button { class: "qu-more", r#type: "button", onclick: move |_| expanded.set(true), "+{n - shown} more" } }
                }
            }
            div { class: "qu-comp",
                input { class: "qu-in", r#type: "text", placeholder: if busy { "Queue a follow-up…" } else { "Message…" }, value: "{draft}", "aria-label": "Message",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); let d = draft.peek().trim().to_string(); if !d.is_empty() { if busy { q.write().push(&d); live.set("Queued".into()) } else { sent.write().push(d.clone()); on_send.call((d, false)) } draft.set(String::new()) } },
                }
                span { class: "qu-kbd", if busy { "↵ queue" } else { "↵ send" } }
            }
        }
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
