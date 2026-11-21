//! Tone tuner (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::tone::{
    DETAIL, FORMALITY, MODEL, PRESETS, RECIPIENT, Tone, ToneTuner as Tuner, VOICES, changed_words,
    compose, diff,
};

use super::{cls, dots, ico, key_of, live, mark};

fn slider(
    label: &'static str,
    part: &'static str,
    levels: [&'static str; 3],
    v: usize,
    on: EventHandler<String>,
    on_key: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "oxaik-tt-sl",
            div { class: "oxaik-row oxaik-between", span { class: "oxaik-tt-lab", "{label}" } span { class: "oxaik-mute", "{levels[v]}" } }
            div { class: "oxaik-tt-track", role: "slider", tabindex: "0", "aria-label": "{label}", "aria-valuemin": "0", "aria-valuemax": "2", "aria-valuenow": "{v}", "aria-valuetext": "{levels[v]}",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End") { e.prevent_default(); on_key.call(k) }
                },
                i { class: "oxaik-tt-fill", style: "width:{v * 50}%" }
                for i in 0..3usize {
                    button { key: "{i}", r#type: "button", tabindex: "-1", class: cls("oxaik-tt-stop", i <= v), style: "left:{i * 50}%", "aria-label": "{label}: {levels[i]}",
                        onclick: move |_| on.call(format!("tt-{part}-{i}")) }
                }
            }
            div { class: "oxaik-row oxaik-between oxaik-mute oxaik-tt-ends", span { "{levels[0]}" } span { "{levels[2]}" } }
        }
    }
}

/// Tone tuner: adjust how a support reply sounds with two three-step sliders (formality, detail)
/// and voice chips (Friendly, Confident, Empathetic; pressing the active chip goes back to
/// neutral). After a short pause the new version arrives: only changed sentences swap, words that
/// are new compared with the draft can be highlighted, and Copy grabs whatever is showing.
/// Untouched, it cycles through the voices until the first press or focus. Sliders speak their
/// level name.
#[component]
pub fn ToneTuner(
    /// Host clock.
    now: ClockMs,
    /// Starting formality (0 casual, 1 neutral, 2 formal).
    #[props(default = 1)]
    formality: usize,
    /// Starting detail (0 brief, 1 standard, 2 detailed).
    #[props(default = 1)]
    detail: usize,
    /// Starting voice (0 neutral … 3 empathetic).
    #[props(default)]
    voice: usize,
    /// Highlight words that are new compared with the draft.
    #[props(default = true)]
    show_changes: bool,
    /// Tour the voices until someone touches the card.
    #[props(default = true)]
    tour: bool,
    /// Rewrites fail (error scenario).
    #[props(default)]
    fail: bool,
) -> Element {
    let t0 = now.0;
    let start = Tone {
        formality: formality.min(2),
        detail: detail.min(2),
        voice: voice.min(3),
    };
    let mut tt = use_signal(move || Tuner::new(start, tour, show_changes, fail, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = tt.read().clone();
    let mut act = move |id: String| {
        let x = tt.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut keyed = move |part: &'static str, k: String| {
        let x = tt.write().key(part, &k, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let c = t.controls(n);
    let shown = t.text_tone(n);
    let ph = t.phase(n);
    let paras = compose(shown);
    let changed = changed_words(shown);
    let copied = t.copied.is_some_and(|x| n < x + 1_500);
    let marks = t.changes;
    rsx! {
        section { class: "oxaik oxaik-tt", "aria-label": "Tone tuner",
            onfocusin: move |_| act("tt-touch".into()),
            onpointerdown: move |_| act("tt-touch".into()),
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h3 { class: "oxaik-h", "To {RECIPIENT.0}" }
                    p { class: "oxaik-sub", "{RECIPIENT.1}" }
                }
                if changed > 0 && marks { span { class: "oxaik-pill oxaik-tt-cnt", "{changed} words changed" } }
            }
            div { class: "oxaik-tt-voices", role: "group", "aria-label": "Voice",
                for (i, (v, _, _)) in PRESETS.iter().enumerate() {
                    button { key: "{i}", r#type: "button", class: cls("oxaik-chip", c.voice == *v), "aria-pressed": "{c.voice == *v}",
                        onclick: move |_| act(format!("tt-voice-{i}")),
                        {ico(match i { 0 => "heart", 1 => "bolt", _ => "users" }, 12.0)} span { "{VOICES[*v]}" }
                    }
                }
            }
            div { class: "oxaik-tt-sls",
                {slider("Formality", "f", FORMALITY, c.formality, EventHandler::new(act), EventHandler::new(move |k| keyed("f", k)))}
                {slider("Detail", "d", DETAIL, c.detail, EventHandler::new(act), EventHandler::new(move |k| keyed("d", k)))}
            }
            div { class: "oxaik-tt-msg", "aria-busy": "{ph == \"rewriting\"}", "data-ph": "{ph}",
                for (pi, para) in paras.iter().enumerate() {
                    p { key: "{pi}",
                        for s in para.iter() {
                            span { key: "{s}", class: "oxaik-tt-s",
                                for (wi, (w, new)) in diff(s).into_iter().enumerate() {
                                    if new && marks { mark { key: "{wi}", "{w} " } } else { span { key: "{wi}", "{w} " } }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "oxaik-row oxaik-between oxaik-tt-foot",
                span { class: "oxaik-row oxaik-mute oxaik-tt-st",
                    if ph == "waiting" || ph == "rewriting" { {dots()} {mark(MODEL.1, 12.0)} span { class: "oxaik-shim", "{t.status(n)}" } }
                    else if ph == "error" { span { class: "oxaik-ps-err oxaik-row", role: "alert", {ico("alert", 12.0)} "{t.status(n)}" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tt-retry".into()), "Try again" } }
                    else { span { key: "{t.status(n)}", class: "oxaik-fade oxaik-tt-stt", "{t.status(n)}" } }
                }
                div { class: "oxaik-ctl",
                    button { r#type: "button", role: "switch", class: cls("oxaik-switch", marks), "aria-checked": "{marks}", onclick: move |_| act("tt-changes".into()), span { class: "oxaik-knob" } "Changes" }
                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tt-copy".into()), {ico(if copied { "check" } else { "copy" }, 12.0)} if copied { "Copied" } else { "Copy" } }
                }
            }
            {live(&said())}
        }
    }
}
