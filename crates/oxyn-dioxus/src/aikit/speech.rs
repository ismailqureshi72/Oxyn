//! Speech player (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::speech::{Phase, Speech, VOICES};

use super::{cls, ico, key_of, live, mark};

/// Speech player: read aloud for an AI reply. Read aloud swaps the action row for a compact
/// player in the same 32 px slot: play, elapsed, a scrubbable bar, total, speed (1×, 1.5×, 2×),
/// a voice menu and Stop reading. The spoken word is highlighted while upcoming words stay
/// dimmed; clicking a word jumps there; a voice switch mid-sentence resumes from the same word.
/// Keys on the bar: ←/→ two seconds, Page up/down ten, Home/End, Space or K play/pause.
#[component]
pub fn SpeechPlayer(
    /// Host clock.
    now: ClockMs,
    /// Open the player and start reading on mount.
    #[props(default = true)]
    autoplay: bool,
    /// `card` (bordered) or `plain` (inside a chat column).
    #[props(default = "card".to_string())]
    variant: String,
    /// The first prepare fails (Retry demo).
    #[props(default)]
    fail: bool,
    /// Dim the words still to come (karaoke); off keeps the reply in full contrast.
    #[props(default = true)]
    karaoke: bool,
    /// Starting voice (0 Ava, 1 Leo, 2 Maya, 3 Noah).
    #[props(default)]
    voice: usize,
) -> Element {
    let t0 = now.0;
    let mut sp = use_signal(move || {
        let mut s = Speech::new(false, fail, t0);
        s.voice = voice.min(3);
        if autoplay {
            s.press("sp-read", t0);
        }
        s
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = sp.read().clone();
    let mut act = move |id: String| {
        let x = sp.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let ph = s.phase_at(n);
    let cur = s.word_at(n);
    let reading = s.open && !matches!(ph, Phase::Idle);
    let (el, tot) = s.times(n);
    let pct = s.pos_at(n) as f64 / s.total.max(1) as f64 * 100.0;
    let words = s.words();
    let vname = VOICES[s.voice].0;
    rsx! {
        article { class: "oxaik oxaik-sp", "data-variant": "{variant}", "aria-label": "Reply from Claude Sonnet 4.5",
            div { class: "oxaik-row oxaik-sp-by", {mark("claude", 16.0)} b { "Claude Sonnet 4.5" } }
            p { class: "oxaik-sp-text", "data-reading": "{reading && karaoke}",
                for (i, (w, para)) in words.iter().enumerate() {
                    if *para { br {} br {} }
                    span { class: if reading && i == cur { "oxaik-w cur" } else if reading && i > cur { "oxaik-w up" } else { "oxaik-w" },
                        onclick: move |_| { if sp.peek().open { act(format!("sp-word-{i}")) } },
                        "{w}"
                    }
                    " "
                }
            }
            div { class: "oxaik-sp-dock", "data-open": "{s.open}",
                div { class: "oxaik-sp-row", "data-layer": "actions",
                    button { class: "oxaik-btn sm", r#type: "button", tabindex: if s.open { "-1" } else { "0" }, onclick: move |_| act("sp-read".into()), {ico("volume", 13.0)} "Read aloud" }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Copy", tabindex: if s.open { "-1" } else { "0" }, onclick: move |_| act("sp-copy".into()), {ico("copy", 13.0)} }
                    button { class: cls("oxaik-ib", s.rating == Some("up")), r#type: "button", "aria-label": "Good response", "aria-pressed": "{s.rating == Some(\"up\")}", tabindex: if s.open { "-1" } else { "0" }, onclick: move |_| act("sp-up".into()), {ico("up", 13.0)} }
                    button { class: cls("oxaik-ib", s.rating == Some("down")), r#type: "button", "aria-label": "Bad response", "aria-pressed": "{s.rating == Some(\"down\")}", tabindex: if s.open { "-1" } else { "0" }, onclick: move |_| act("sp-down".into()), span { class: "oxaik-flip", {ico("up", 13.0)} } }
                }
                div { class: "oxaik-sp-row", "data-layer": "player", role: "group", "aria-label": "Speech player",
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "{s.play_name(n)}", disabled: ph == Phase::Preparing, tabindex: if s.open { "0" } else { "-1" },
                        onclick: move |_| act("sp-play".into()),
                        {ico(match ph { Phase::Playing => "pause", Phase::Done | Phase::Failed => "retry", _ => "play" }, 13.0)}
                    }
                    if ph == Phase::Preparing {
                        span { class: "oxaik-sp-prep oxaik-shim", "Preparing {vname}'s voice" }
                    } else if ph == Phase::Failed {
                        span { class: "oxaik-sp-prep oxaik-bad", role: "alert", {ico("alert", 12.0)} "Couldn't generate audio" }
                    } else {
                        span { class: "oxaik-num oxaik-mute oxaik-sp-t", "{el}" }
                        input { class: "oxaik-range oxaik-sp-bar", r#type: "range", min: "0", max: "{s.total}", step: "100", value: "{s.pos_at(n)}",
                            tabindex: if s.open { "0" } else { "-1" }, "aria-label": "Playback position", "aria-valuetext": "{el} of {tot}", style: "--p:{pct}%",
                            oninput: move |e| { if let Ok(v) = e.value().parse::<u64>() { act(format!("sp-seek-{v}")) } },
                            onkeydown: move |e: KeyboardEvent| {
                                let k = key_of(&e);
                                if matches!(k.as_str(), "ArrowLeft"|"ArrowRight"|"PageUp"|"PageDown"|"Home"|"End"|" "|"k"|"K") {
                                    e.prevent_default();
                                    let x = sp.write().key("bar", &k, n);
                                    if let Some(x) = x { said.set(x) }
                                }
                            },
                        }
                        span { class: "oxaik-num oxaik-mute oxaik-sp-t oxaik-sp-tot", "{tot}" }
                    }
                    button { class: "oxaik-chip oxaik-num", r#type: "button", "aria-label": "Speed {s.speed_label()}", tabindex: if s.open { "0" } else { "-1" }, onclick: move |_| act("sp-speed".into()), "{s.speed_label()}" }
                    div { class: "oxaik-menu-wrap",
                        button { class: "oxaik-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{s.menu}", tabindex: if s.open { "0" } else { "-1" },
                            onclick: move |_| act("sp-menu".into()), "{vname}" {ico("chevron", 11.0)}
                        }
                        if s.menu {
                            ul { class: "oxaik-menu oxaik-sp-menu", role: "menu", "aria-label": "Voice",
                                onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowDown"|"ArrowUp"|"Escape") { e.prevent_default(); let x = sp.write().key("menu", &k, n); if let Some(x) = x { said.set(x) } } },
                                for (i, (nm, note)) in VOICES.iter().enumerate() {
                                    li { key: "{i}", role: "menuitemradio", "aria-checked": "{i == s.voice}", tabindex: "0", class: cls("oxaik-opt oxaik-sp-v", i == s.voice),
                                        onclick: move |_| act(format!("sp-voice-{i}")),
                                        onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Enter" { e.prevent_default(); act(format!("sp-voice-{i}")) } },
                                        span { b { "{nm}" } span { class: "oxaik-mute", " · {note}" } }
                                        if i == s.voice { {ico("check", 12.0)} }
                                    }
                                }
                                li { class: "oxaik-mute oxaik-sp-sim", role: "none", "Simulated, no audio plays" }
                            }
                        }
                    }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Stop reading", tabindex: if s.open { "0" } else { "-1" }, onclick: move |_| act("sp-stop".into()), {ico("x", 13.0)} }
                }
            }
            {live(&said())}
        }
    }
}
