//! Composer parts (plan #27, chunk 3): voice recorder, dictation button, mention input, mode
//! switch, permission selector, tools menu, queue stack and agent steering.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::parts::{Suggestion, Tool},
};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    AgentSteering, DictationButton, MentionInput, ModeSwitch, PermissionSelector, ToolsMenu,
};
use oxyn_dioxus::{QueueStack, VoiceRecorder};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

/// A key that changes with every control, so a new variant remounts the part with its props.
fn key(v: &Vals, keys: &[&str]) -> String {
    keys.iter().map(|k| v.s(k)).collect::<Vec<_>>().join("|")
}

fn queue_items(n: usize) -> Vec<String> {
    [
        "Also bump the jsonwebtoken crate",
        "Then rerun the auth suite three times",
        "Open a draft PR when it passes",
        "Ping Maya for review",
        "Update the changelog",
    ]
    .iter()
    .take(n)
    .map(|s| s.to_string())
    .collect()
}

fn voice(v: &Vals, now: ClockMs) -> Element {
    let k = key(v, &["phase", "variant", "max", "deny", "bars"]);
    rsx! {
        div { key: "{k}", style: "width:100%;display:flex;justify-content:center",
            VoiceRecorder { now_ms: now.0, phase: v.string("phase"), variant: v.string("variant"), max_s: v.f("max") as u64, deny: v.b("deny"), bars: v.u("bars") }
        }
    }
}

fn voice_code(v: &Vals) -> String {
    Code::new("VoiceRecorder")
        .raw("now_ms", "clock.now()")
        .s("variant", v.s("variant"), "inline")
        .raw("max_s", format!("{}", v.f("max") as u64))
        .raw("on_send", "move |ms: u64| upload(ms)")
        .done()
}

fn queue(v: &Vals, _: ClockMs) -> Element {
    let k = key(v, &["n", "busy", "paused", "rows", "turn"]);
    rsx! {
        div { key: "{k}", style: "width:100%;max-width:560px",
            QueueStack { items: queue_items(v.u("n")), busy: v.b("busy"), paused: v.b("paused"), max_rows: v.u("rows"), turn_ms: v.f("turn") as u64 }
        }
    }
}

fn queue_code(v: &Vals) -> String {
    Code::new("QueueStack")
        .raw("items", "queued")
        .b("paused", v.b("paused"))
        .raw("max_rows", v.s("rows").to_string())
        .raw(
            "on_send",
            "move |(text, now): (String, bool)| agent.send(text, now)",
        )
        .done()
}

/// Chunk-3 demos.
pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![
        Demo {
            controls: vec![
                sel(
                    "phase",
                    "Start in",
                    &["idle", "recording", "paused", "review", "denied"],
                    "idle",
                ),
                sel("variant", "Variant", &["inline", "card"], "inline"),
                rng("max", "Limit s", 10.0, 300.0, 5.0, "120"),
                rng("bars", "Bars", 12.0, 48.0, 2.0, "28"),
                tog("deny", "Permission denied", false),
            ],
            variants: vec![
                var("idle", "A mic in the composer row.", &[]),
                var(
                    "recording",
                    "Live level bars, timer and pause.",
                    &[("phase", "recording")],
                ),
                var("review", "Play, discard or send.", &[("phase", "review")]),
                var(
                    "card",
                    "Card variant near its limit.",
                    &[("variant", "card"), ("phase", "recording"), ("max", "15")],
                ),
                var("denied", "The microphone was refused.", &[("deny", "true")]),
            ],
            render: voice,
            code: voice_code,
            keyboard: vec![
                ("Enter / Space", "Start, pause, resume, stop or send"),
                ("Tab", "Pause, stop, discard, send"),
            ],
            a11y: vec![
                "Every button names its action; the bars are aria-hidden.",
                "A polite region announces start, pause, the last ten seconds and send.",
            ],
            responsive: vec!["The inline variant fits a 320 px composer row."],
            height: 260,
            live: true,
            ..mk(
                "voice-recorder",
                "Voice recorder",
                "composer",
                "Record a voice message in the composer: permission, live level bars, a timer that counts down near the limit, pause, then play, discard or send.",
                "VoiceRecorder",
                "VoiceRecorderEl",
                "composer-voice-input",
                "composer/parts.rs",
            )
        },
        Demo {
            controls: vec![
                rng("n", "Queued", 0.0, 5.0, 1.0, "3"),
                rng("rows", "Collapse after", 0.0, 5.0, 1.0, "3"),
                rng("turn", "Turn ms", 2000.0, 12000.0, 500.0, "6000"),
                tog("busy", "Agent running", true),
                tog("paused", "Paused", false),
            ],
            variants: vec![
                var("queue", "Messages wait for the turn to end.", &[]),
                var(
                    "collapsed",
                    "Long queues collapse.",
                    &[("n", "5"), ("rows", "2")],
                ),
                var("paused", "Queue paused.", &[("paused", "true")]),
                var(
                    "idle",
                    "Agent idle: the next item sends.",
                    &[("busy", "false")],
                ),
            ],
            render: queue,
            code: queue_code,
            keyboard: vec![
                ("Tab", "Each row: send now, edit, remove"),
                ("Alt + ↑ / ↓", "Reorder a row"),
                ("Enter / Escape", "Save or cancel an edit"),
            ],
            a11y: vec![
                "The stack is a list labelled “Queued messages”.",
                "Sending, removing and reordering are announced politely.",
            ],
            responsive: vec!["Rows truncate to one line; edit opens the full text."],
            height: 320,
            align: Align::End,
            live: true,
            ..mk(
                "queue-stack",
                "Queue stack",
                "composer",
                "Messages typed while the agent works wait above the composer: they send in order as turns end, and each can be sent now, edited, reordered or removed.",
                "QueueStack",
                "QueueStackEl",
                "composer-queue-stack",
                "composer/parts.rs",
            )
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 8] = [
    "voice-recorder",
    "dictation-button",
    "mention-input",
    "mode-switch",
    "permission-selector",
    "tools-menu",
    "queue-stack",
    "agent-steering",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
