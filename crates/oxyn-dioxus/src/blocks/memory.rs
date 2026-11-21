//! Assistant memory (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::memory::{CAPTURE, GROUPS, Memory};

use super::switch;
use crate::aikit::{cls, ico, key_of, live};

/// Assistant memory: what an assistant remembers about you, one plain sentence per memory. The
/// header shows how full memory is and switches for saved memories and chat history; filter
/// pills and a search sit over the grouped list; each row's source, Edit and Forget sit in
/// reserved space (shown on hover and focus, always on touch). The chat's new memory files into
/// its group with a highlight, Forget leaves an Undo bar with a countdown, and the field at the
/// foot sorts a typed memory into a group.
#[component]
pub fn AssistantMemory(
    /// Host clock.
    now: ClockMs,
    /// Start near capacity (meter warning).
    #[props(default)]
    full: bool,
    /// Start with saved memories off.
    #[props(default)]
    paused: bool,
    /// Starting filter (0 All, then each group).
    #[props(default)]
    filter: usize,
) -> Element {
    let mut mem = use_signal(move || {
        let mut m = Memory::new(full, paused);
        m.filter = filter.min(GROUPS.len());
        m
    });
    let mut said = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let n = now.0;
    let m = mem.read().clone();
    let mut act = move |id: String| {
        let x = mem.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut keyed = move |k: String| {
        let x = mem.write().key(&k, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let counts = m.counts();
    let pct = (m.usage() * 100.0).round();
    let shown = m.shown();
    rsx! {
        section { class: "oxaik oxbk oxbk-mm", "aria-labelledby": "oxbk-mm-h",
            header { class: "oxaik-hd",
                div { class: "oxaik-hl",
                    h2 { id: "oxbk-mm-h", class: "oxaik-h", "Memory" }
                    p { class: "oxaik-sub", "What the assistant remembers about you" }
                }
                div { class: "oxbk-mm-meter", role: "meter", "aria-label": "Memory used", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{pct}", "aria-valuetext": "{pct}% full",
                    span { class: "oxaik-num", "data-hue": if m.warn() { "amber" } else { "neutral" }, if m.warn() { {ico("alert", 11.0)} } "{pct}% full" }
                    span { class: "oxaik-bar", span { style: "width:{pct}%", "data-hue": if m.warn() { "amber" } else { "neutral" } } }
                }
            }
            div { class: "oxbk-mm-sws",
                div { class: "oxbk-mm-sw", {switch("Saved memories", m.saved, EventHandler::new(move |_| act("mm-saved".into())))}
                    div { b { "Saved memories" } p { class: "oxaik-mute", "Remember details across chats" } } }
                div { class: "oxbk-mm-sw", {switch("Chat history", m.history, EventHandler::new(move |_| act("mm-history".into())))}
                    div { b { "Chat history" } p { class: "oxaik-mute", "Use past chats when answering" } } }
            }
            if m.pending {
                button { class: "oxbk-mm-cap oxaik-fade", r#type: "button", "aria-label": "Memory updated: {CAPTURE.0}. File it",
                    onclick: move |_| act("mm-file".into()),
                    {ico("brain", 12.0)} b { "Memory updated" } span { class: "oxbk-mm-capt", "{CAPTURE.0}" }
                }
            }
            div { class: "oxaik-row oxbk-mm-tools",
                if let Some(q) = m.search.clone() {
                    div { class: "oxbk-mm-search oxaik-fade",
                        {ico("search", 12.0)}
                        input { class: "oxaik-in", r#type: "search", value: "{q}", placeholder: "Search memories", "aria-label": "Search memories",
                            oninput: move |e| { let x = mem.write().set_query(&e.value()); if let Some(x) = x { said.set(x) } },
                            onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Escape" { keyed("Escape".into()) } }
                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Close search", onclick: move |_| act("mm-search".into()), {ico("x", 12.0)} }
                    }
                } else {
                    div { class: "oxbk-filter oxbk-mm-pills", role: "group", "aria-label": "Show",
                        onkeydown: move |e: KeyboardEvent| { let k = key_of(&e); if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") { e.prevent_default(); keyed(k) } },
                        for i in 0..=GROUPS.len() {
                            button { key: "{i}", class: cls("oxaik-sg", m.filter == i), r#type: "button", "aria-pressed": "{m.filter == i}", tabindex: if m.filter == i { "0" } else { "-1" },
                                onclick: move |_| act(format!("mm-f-{i}")),
                                if i == 0 { "All" } else { "{GROUPS[i - 1]}" } span { class: "oxbk-cnt", "{counts[i]}" } }
                        }
                    }
                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Search memories", onclick: move |_| act("mm-search".into()), {ico("search", 13.0)} }
                }
            }
            if !m.saved { p { class: "oxbk-mm-note", {ico("pause", 12.0)} "Saved memories are off. The assistant keeps what is here but saves nothing new." } }
            div { class: "oxbk-mm-list oxaik-scroll", "data-off": "{!m.saved}",
                if shown.is_empty() { p { class: "oxbk-empty", {ico("search", 12.0)} "No memories match." } }
                for (g, items) in shown {
                    div { key: "{g}", class: "oxbk-mm-group", role: "group", "aria-labelledby": "oxbk-mm-g{g}",
                        h3 { id: "oxbk-mm-g{g}", class: "oxbk-cap", "{GROUPS[g]} · {items.len()}" }
                        ul { class: "oxbk-mm-ul",
                            for it in items {
                                {
                                    let id = it.id;
                                    let editing = m.editing.as_ref().filter(|e| e.0 == id).map(|e| e.1.clone());
                                    let t = it.text.clone();
                                    rsx! {
                                        li { key: "{id}", class: "oxbk-mm-row", "data-glow": "{m.glow(it, n)}",
                                            if let Some(d) = editing {
                                                textarea { class: "oxaik-in oxbk-mm-edit", value: "{d}", "aria-label": "Edit memory", autofocus: true,
                                                    oninput: move |e| mem.write().set_draft(&e.value()),
                                                    onkeydown: move |e: KeyboardEvent| match key_of(&e).as_str() {
                                                        "Enter" if !e.modifiers().shift() => { e.prevent_default(); act("mm-save".into()) }
                                                        "Escape" => keyed("Escape".into()),
                                                        _ => {}
                                                    } }
                                                div { class: "oxaik-row",
                                                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mm-cancel".into()), "Cancel" }
                                                    button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("mm-save".into()), "Save" } }
                                            } else {
                                                p { class: "oxbk-mm-t", "{t}" }
                                                span { class: "oxbk-mm-acts",
                                                    span { class: "oxaik-mute oxbk-mm-src", "{it.source}" }
                                                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Edit memory: {t}", title: "Edit", onclick: move |_| act(format!("mm-edit-{id}")), {ico("pencil", 12.0)} }
                                                    button { class: "oxaik-ib", r#type: "button", "aria-label": "Forget memory: {t}", title: "Forget", onclick: move |_| act(format!("mm-forget-{id}")), {ico("trash", 12.0)} }
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
            if m.can_undo(n) {
                if let Some((f, _, _)) = m.forgotten.as_ref() {
                    div { class: "oxbk-undo oxaik-row oxaik-fade", span { class: "oxbk-mm-capt", "Forgot “{f.text}”" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mm-undo".into()), {ico("undo", 12.0)} "Undo · {m.undo_left(n)}s" } }
                }
            }
            div { class: "oxbk-mm-add",
                input { class: "oxaik-in", value: "{draft}", placeholder: "Add something to remember", "aria-label": "Add a memory",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Enter" {
                        let t = draft.read().clone();
                        let x = mem.write().add(&t, n);
                        if let Some(x) = x { said.set(x); if mem.read().added.as_ref().is_some_and(|a| a.0) { draft.set(String::new()) } }
                    } }
                if let Some((ok, w)) = m.added.clone() { span { class: "oxbk-st", "data-hue": if ok { "green" } else { "red" }, {ico(if ok { "check-circle" } else { "alert" }, 12.0)} span { "{w}" } } }
            }
            {live(&said.read())}
        }
    }
}
