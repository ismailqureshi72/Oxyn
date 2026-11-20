//! AI translate (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::translate::{LANGS, TEXT, Translate};

use super::{cls, dots, ico, key_of, live, mark};

/// AI translate: a message card where picking a language streams the translation in place,
/// sentence by sentence over the dimmed original (each finished sentence replaces one dimmed
/// one, so the card never jumps), with a detected-language chip, Show original and a status line
/// credited to the model. Side by side shows both (Oxyn extra). The menu is a listbox: ↑/↓,
/// Home/End, Enter, Escape.
#[component]
pub fn AiTranslate(
    /// Host clock.
    now: ClockMs,
    /// Starting language (0 original English, 1 German, 2 French, 3 Spanish, 4 Japanese,
    /// 5 Portuguese); anything but 0 translates on mount.
    #[props(default = 1)]
    lang: usize,
    /// The first translation fails (Retry demo).
    #[props(default)]
    fail: bool,
    /// Show the original and the translation side by side (Oxyn extra).
    #[props(default)]
    side: bool,
) -> Element {
    let t0 = now.0;
    let mut tr = use_signal(move || {
        let mut t = Translate::new(lang, fail, t0);
        t.side = side;
        t
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let t = tr.read().clone();
    let mut act = move |id: String| {
        let x = tr.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let status = t.status(n);
    let show_orig = t.lang == 0 || t.original;
    rsx! {
        article { class: "oxaik oxaik-tr", "aria-label": "Message from Rina Okafor",
            header { class: "oxaik-hd",
                div { class: "oxaik-row oxaik-tr-who", span { class: "oxaik-av", "RO" } div { class: "oxaik-hl", b { "Rina Okafor" } span { class: "oxaik-mute", "#launch · 10:42" } } }
                div { class: "oxaik-row",
                    span { class: "oxaik-pill", {ico("globe", 11.0)} "Detected English" }
                    div { class: "oxaik-menu-wrap",
                        button { class: "oxaik-btn sm", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{t.menu}",
                            onclick: move |_| act("tr-menu".into()),
                            onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowDown"|"ArrowUp"|"Home"|"End"|"Enter"|" "|"Escape") { e.prevent_default(); let x = tr.write().key(&k, n); if let Some(x) = x { said.set(x) } } },
                            {ico("globe", 12.0)} if t.lang == 0 { "Translate" } else { "{LANGS[t.lang].1}" } {ico("chevron", 11.0)}
                        }
                        if t.menu {
                            ul { class: "oxaik-menu", role: "listbox", "aria-label": "Language",
                                for (i, (code, name)) in LANGS.iter().enumerate() {
                                    li { key: "{code}", role: "option", "aria-selected": "{i == t.lang}", class: cls("oxaik-opt", i == t.hi), onclick: move |_| act(format!("tr-lang-{i}")),
                                        span { class: "oxaik-mono oxaik-mute", "{code}" } span { if i == 0 { "Original" } else { "{name}" } }
                                        if i == t.lang { {ico("check", 12.0)} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-tr-body", "data-side": "{t.side && t.lang != 0}",
                if t.side && t.lang != 0 {
                    div { class: "oxaik-tr-col", span { class: "oxaik-mute oxaik-tr-lab", "English" } p { lang: "en", "{TEXT[0].join(\" \")}" } }
                }
                div { class: "oxaik-tr-col",
                    if t.side && t.lang != 0 { span { class: "oxaik-mute oxaik-tr-lab", "{LANGS[t.lang].1}" } }
                    p { class: "oxaik-tr-text", lang: if show_orig { "en" } else { LANGS[t.lang].0 },
                        for i in 0..3usize {
                            if show_orig || status == "error" {
                                span { key: "o{i}", "{TEXT[0][i]} " }
                            } else if t.done_sentence(i, n) {
                                span { key: "d{i}", class: "oxaik-fade", "{TEXT[t.lang][i]} " }
                            } else if t.shown(i, n) > 0 {
                                span { key: "s{i}", span { "{t.sentence(i, n)}" } span { class: "oxaik-tr-dim", " " } }
                            } else {
                                span { key: "w{i}", class: "oxaik-tr-dim", "{TEXT[0][i]} " }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-row oxaik-between oxaik-tr-foot",
                span { class: if status == "error" { "oxaik-row oxaik-bad" } else { "oxaik-row oxaik-mute" }, role: if status == "error" { "alert" } else { "none" },
                    if status == "translating" { {dots()} } else if status == "done" { {mark("claude", 12.0)} } else if status == "error" { {ico("alert", 12.0)} }
                    "{t.status_line(n)}"
                }
                div { class: "oxaik-row",
                    if status == "error" { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("tr-retry".into()), {ico("retry", 12.0)} "Try again" } }
                    if t.lang != 0 && status != "error" {
                        button { class: cls("oxaik-chip", t.original), r#type: "button", "aria-pressed": "{t.original}", onclick: move |_| act("tr-orig".into()), "Show original" }
                        button { class: cls("oxaik-chip", t.side), r#type: "button", "aria-pressed": "{t.side}", onclick: move |_| act("tr-side".into()), "Side by side" }
                    }
                }
            }
            {live(&said())}
        }
    }
}
