//! Command and search (plan #26): `CommandPalette` (also affectUI's command menu), `AiCommand`,
//! `AiSearchBar`, `SettingsCommand`, `SearchField`, `ExpandingSearch`, `ShortcutRecorder`,
//! `ShortcutList`, `ControlCenter`.
//!
//! Logic lives in `oxyn_core::primitives::{command, search, control_center}`. The input keeps
//! focus while the arrow keys move the active option (`aria-activedescendant`), and one shared
//! highlight glides between rows for the pointer and jumps for the keyboard. AI answers take the
//! host clock (`now`) so they stream with the same timing on the web and in GPUI.

use std::{collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        command::{
            ActionState, AiAction, AiCommand as AiModel, AiMode, AiSearch, AnswerPhase,
            AnswerStream, Command, Palette, PaletteKey, PaletteOutcome, SavePhase, Seg, Setting,
            SettingKind, SettingValue, SettingsCommand as SettingsModel, Source, highlight,
            segments,
        },
        control_center::{Channel, ControlCenter as CcModel, Detail, FocusMode, Meter, StepKey},
        display::Platform,
        search::{
            Anchor, Binding, ExpandingSearch as XsModel, Mods, RecOutcome, RecState, Recorder,
            SearchOutcome, ShortcutGroup, caps, clear_visible, dimmed, filter_cheatsheet,
            filter_strings, spoken,
        },
    },
};

use super::{buttons::Keys, cls, use_glide};
use crate::composer::icons::Icon;

fn slug(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .take(18)
        .collect::<String>()
        .to_lowercase()
}

fn marked(label: &str, ranges: &[(usize, usize)]) -> Element {
    rsx! {
        for (i, (t, m)) in highlight(label, ranges).into_iter().enumerate() {
            if m { b { key: "{i}", class: "hit", "{t}" } } else { span { key: "{i}", "{t}" } }
        }
    }
}

fn logo(text: &str) -> Element {
    rsx! { span { class: "logo", "aria-hidden": "true", "{text}" } }
}

fn pal_key(e: &KeyboardEvent) -> Option<PaletteKey> {
    let m = e.modifiers();
    Some(match e.key() {
        Key::ArrowDown => PaletteKey::Down,
        Key::ArrowUp => PaletteKey::Up,
        Key::Home => PaletteKey::Home,
        Key::End => PaletteKey::End,
        Key::Tab if m.shift() => PaletteKey::ShiftTab,
        Key::Tab => PaletteKey::Tab,
        Key::Enter => PaletteKey::Enter,
        Key::Escape => PaletteKey::Escape,
        Key::Backspace => PaletteKey::Backspace,
        Key::Character(c)
            if (m.meta() || m.ctrl()) && matches!(c.as_str(), "1" | "2" | "3" | "4" | "5") =>
        {
            PaletteKey::Digit(c.parse().ok()?)
        }
        _ => return None,
    })
}

/// Command palette: search and run anything from one field. Every word of the query must match
/// (label anywhere, a word start in the description or keywords, or letters in order); matched
/// letters show in medium weight; groups follow their best match. ↑ ↓ wrap, Home / End, Tab jumps
/// a section, Enter runs, Escape clears and then closes. `variant: menu` is affectUI's command
/// menu: tool logos, ⌘1–⌘5 on the first five suggestions, a tray confirming what ran and an
/// "Ask the assistant" row when nothing matches. Oxyn plus: scoped modes (`>` commands, `@`
/// files, `#` threads) and recents.
#[component]
pub fn CommandPalette(
    /// Commands.
    items: Vec<Command>,
    /// `palette` · `menu`.
    #[props(default = "palette".to_string())]
    variant: String,
    /// Placeholder.
    #[props(default = "Type a command or search…".to_string())]
    placeholder: String,
    /// Hidden label for the input.
    #[props(default = "Command palette".to_string())]
    label: String,
    /// Focus the field on mount.
    #[props(default)]
    auto_focus: bool,
    /// Initial query.
    #[props(default)]
    query: String,
    /// Scoped modes by prefix.
    #[props(default = true)]
    scopes: bool,
    /// "Ask the assistant" row when nothing matches.
    #[props(default = true)]
    ask_fallback: bool,
    /// Footer with key hints.
    #[props(default = true)]
    footer: bool,
    /// Floating shadow (inside a dialog).
    #[props(default)]
    elevated: bool,
    /// Results max height px.
    #[props(default = 342.0)]
    max_height: f32,
    /// Width px.
    #[props(default = 560.0)]
    width: f32,
    /// Clock (the tray line fades after a while).
    #[props(default)]
    now: ClockMs,
    /// A command ran (id, query).
    #[props(default)]
    on_select: EventHandler<(String, String)>,
    /// The assistant was asked.
    #[props(default)]
    on_ask: EventHandler<String>,
    /// Escape on an empty field.
    #[props(default)]
    on_close: EventHandler<()>,
) -> Element {
    let mut m = use_signal(|| {
        let mut p = Palette::new(items.clone());
        p.scopes = scopes;
        p.ask_fallback = ask_fallback;
        p.set_query(&query);
        p
    });
    let g = use_glide();
    let mut input = use_signal(|| None::<Rc<MountedData>>);
    let id = format!("oxyn-cp-{}", slug(&label));
    let menu = variant == "menu";
    let st = m.read().clone();
    let secs = st.sections();
    let asking = st.asking();
    let mut flat_i = 0usize;
    let sync = move || {
        let a = m.peek().active;
        let kb = m.peek().jump;
        g.go(Some(a), kb);
    };
    let outcome = move |o: PaletteOutcome| match o {
        PaletteOutcome::Run(id, q) => on_select.call((id, q)),
        PaletteOutcome::Ask(q) => on_ask.call(q),
        PaletteOutcome::Close => on_close.call(()),
        PaletteOutcome::None => {}
    };
    let active_id = st.active_item().map(|i| format!("{id}-o{i}"));
    rsx! {
        div { class: cls(&["oxyn-cmd", if menu { "menu" } else { "" }, if elevated { "elevated" } else { "" }]), style: "width:{width}px;max-width:100%",
            div { class: "cin",
                Icon { name: "search", size: 15.0 }
                if let Some(l) = st.scope_label() { span { class: "scope", "{l}" } }
                input { class: "q", r#type: "text", role: "combobox", "aria-expanded": "true", "aria-controls": "{id}-list", "aria-autocomplete": "list",
                    "aria-activedescendant": if let Some(a) = &active_id { "{a}" }, "aria-label": "{label}", placeholder: "{placeholder}", value: "{st.query}",
                    autocomplete: "off", spellcheck: "false",
                    onmounted: move |e| { input.set(Some(e.data())); if auto_focus { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                    oninput: move |e| { m.write().set_query(&e.value()); sync() },
                    onkeydown: move |e: KeyboardEvent| if let Some(k) = pal_key(&e) {
                        if k == PaletteKey::Backspace && !m.peek().query.is_empty() { return; }
                        if !matches!(k, PaletteKey::Tab | PaletteKey::ShiftTab) || !m.peek().flat().is_empty() { e.prevent_default(); }
                        let o = m.write().key(k, now);
                        sync();
                        outcome(o)
                    },
                }
                if !st.query.is_empty() || st.scope.is_some() {
                    button { class: "oxyn-x", r#type: "button", "aria-label": "Clear", onclick: move |_| { m.write().set_query(""); m.write().scope = None; }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                } else if menu {
                    span { class: "esc", "aria-hidden": "true", "esc" }
                }
            }
            div { id: "{id}-list", class: "clist", role: "listbox", "aria-label": "{label} results", style: "max-height:{max_height}px",
                onmounted: move |e| g.set_root(e.data()),
                span { class: g.class("hl"), style: "{g.style()}", "aria-hidden": "true" }
                if asking {
                    div { class: "row ask on", role: "option", "aria-selected": "true", id: "{id}-ask",
                        onclick: move |_| { let o = m.write().key(PaletteKey::Enter, now); outcome(o) },
                        span { class: "lead", Icon { name: "spark", size: 15.0 } }
                        span { class: "txt", span { class: "lb", "Ask the assistant: " b { "{st.query}" } } }
                        Keys { keys: "Enter" }
                    }
                } else if secs.is_empty() {
                    div { class: "empty", role: "presentation", Icon { name: "search", size: 18.0 } div { class: "et", "No results found" } div { class: "eh", "Adjust your search to try again" } }
                }
                for sec in secs.iter() {
                    div { key: "{sec.label}", class: "grp", role: "group", "aria-label": "{sec.label}",
                        div { class: "gl", role: "presentation", "{sec.label}" }
                        for (item, hit) in sec.rows.iter().cloned() {
                            {
                                let i = flat_i;
                                flat_i += 1;
                                let c = st.items[item].clone();
                                let on = st.active == i;
                                let digit = if menu { st.digit_for(item) } else { None };
                                let oid = format!("{id}-o{item}");
                                rsx! {
                                    div { key: "{c.id}", id: "{oid}", class: cls(&["row", if on && !g.measured() { "on" } else { "" }, if on { "act" } else { "" }]),
                                        role: "option", "aria-selected": "{on}",
                                        onmounted: move |e| g.set_item(i, e.data()),
                                        onpointermove: move |_| if m.peek().active != i { m.write().hover(i); sync() },
                                        onclick: move |_| { let o = m.write().run(item, now); outcome(o) },
                                        span { class: "lead",
                                            if let Some(l) = c.logo.clone() { {logo(&l)} } else if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 15.0 } }
                                        }
                                        span { class: "txt",
                                            span { class: "lb", {marked(&c.label, &hit.ranges)} }
                                            if let Some(d) = c.description.clone() { span { class: "ds", "{d}" } }
                                        }
                                        if let Some(h) = c.hint.clone() { span { class: "hint", "{h}" } }
                                        if let Some(d) = digit { Keys { keys: "Mod+{d}" } } else if let Some(s) = c.shortcut.clone() { Keys { keys: s } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if menu {
                div { class: cls(&["tray", if st.ran.is_some() { "show" } else { "" }]), role: "status", "aria-live": "polite",
                    if let Some((what, _)) = st.ran.clone() { Icon { name: "check", size: 13.0, stroke: 2.0 } span { "{what}" } }
                }
            }
            if footer {
                div { class: "cfoot", "aria-hidden": "true",
                    span { Keys { keys: "Up" } Keys { keys: "Down" } " Move" }
                    span { Keys { keys: "Enter" } " Run" }
                    span { Keys { keys: "Tab" } " Section" }
                    span { class: "lastk", Keys { keys: "Esc" } " Close" }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce()}" }
        }
    }
}

fn answer_view(
    text: &str,
    sources: &[Source],
    lit: Signal<Option<String>>,
    mut on_cite: impl FnMut(usize) + Copy + 'static,
) -> Element {
    let mut lit = lit;
    rsx! {
        for (i, s) in segments(text).into_iter().enumerate() {
            match s {
                Seg::Text(t) => rsx! { span { key: "{i}", "{t}" } },
                Seg::Code(c) => rsx! { code { key: "{i}", "{c}" } },
                Seg::Cite(n) => {
                    let src = sources.get(n.saturating_sub(1)).cloned();
                    let sid = src.as_ref().map(|x| x.id.clone()).unwrap_or_default();
                    let sid2 = sid.clone();
                    let title = src.map(|x| x.title).unwrap_or_default();
                    rsx! {
                        button { key: "{i}", class: "cite", r#type: "button", "aria-label": "Source {n}: {title}",
                            onpointerenter: move |_| lit.set(Some(sid.clone())), onfocus: move |_| lit.set(Some(sid2.clone())),
                            onpointerleave: move |_| lit.set(None), onblur: move |_| lit.set(None),
                            onclick: move |_| on_cite(n),
                            "{n}"
                        }
                    }
                }
            }
        }
    }
}

/// A scripted answer for demos and replays: text with `[n]` citations, its sources and the next
/// steps it suggests.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptedAnswer {
    /// Text.
    pub text: String,
    /// Sources.
    pub sources: Vec<Source>,
    /// Suggested actions.
    pub actions: Vec<AiAction>,
}

/// AI command: a fixed-size command palette with an Ask mode. Search pages and actions, or press
/// Tab to ask: the answer streams in place with numbered sources and runnable next steps
/// (Running → Done). The palette never changes size. Escape closes the answer, then goes back to
/// Search, then closes.
#[component]
pub fn AiCommand(
    /// Pages and actions.
    items: Vec<Command>,
    /// Questions offered while Ask is open and empty.
    #[props(default)]
    suggested_questions: Vec<String>,
    /// The model that answers.
    #[props(default = "Claude Sonnet".to_string())]
    model: String,
    /// `search` · `ask`.
    #[props(default = "search".to_string())]
    default_mode: String,
    /// Initial query.
    #[props(default)]
    default_query: String,
    /// Status before text arrives.
    #[props(default = "Searching your workspace".to_string())]
    thinking_label: String,
    /// Results height px.
    #[props(default = 392.0)]
    height: f32,
    /// Floating shadow.
    #[props(default)]
    elevated: bool,
    /// The answer streamed for any question (demo engine).
    answer: ScriptedAnswer,
    /// Ask on mount with the default query (demo).
    #[props(default)]
    autoplay: bool,
    /// Clock.
    now: ClockMs,
    /// Width px.
    #[props(default = 600.0)]
    width: f32,
    /// A page or action was chosen.
    #[props(default)]
    on_select: EventHandler<String>,
    /// A question was asked.
    #[props(default)]
    on_ask: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut c = AiModel::new(items.clone());
        c.suggested_questions = suggested_questions.clone();
        c.palette.set_query(&default_query);
        if default_mode == "ask" {
            c.mode = AiMode::Ask;
        }
        c
    });
    let mut asked_at = use_signal(|| None::<ClockMs>);
    let lit = use_signal(|| None::<String>);
    let g = use_glide();
    let st = m.read().clone();
    let mut ans_s = use_signal(|| answer.clone());
    if *ans_s.peek() != answer {
        ans_s.set(answer.clone());
    }
    let mut ask = move |q: String, t: ClockMs| {
        if q.trim().is_empty() {
            return;
        }
        on_ask.call(q.clone());
        let ans = ans_s.peek().clone();
        m.write().ask(
            &q,
            AnswerStream::scripted(&ans.text, ans.sources.clone(), t),
            ans.actions.clone(),
        );
        asked_at.set(Some(t));
    };
    if autoplay && asked_at.peek().is_none() && !default_query.is_empty() && default_mode == "ask" {
        ask(default_query.clone(), now);
    }
    let phase = st.answer.phase(now);
    let secs = st.palette.sections();
    let mut flat_i = 0usize;
    let ask_mode = st.mode == AiMode::Ask;
    let visible = st.answer.visible(now);
    let cited = st.answer.cited(now);
    let lit_id = lit();
    rsx! {
        div { class: cls(&["oxyn-cmd ai", if elevated { "elevated" } else { "" }, if ask_mode { "asking" } else { "" }]), style: "width:{width}px;max-width:100%", "aria-label": "Command palette",
            div { class: "cin",
                if ask_mode { span { class: "mode", Icon { name: "spark", size: 13.0 } "Ask AI" } } else { Icon { name: "search", size: 15.0 } }
                input { class: "q", r#type: "text", role: "combobox", "aria-expanded": "true", "aria-autocomplete": "list", autocomplete: "off",
                    placeholder: if ask_mode { "Ask anything about your workspace…" } else { "Search pages and actions, or press Tab to ask" },
                    value: if ask_mode { "{st.question}" } else { "{st.palette.query}" },
                    oninput: move |e| { let v = e.value(); let mut w = m.write(); if w.mode == AiMode::Ask { w.question = v; w.answer = AnswerStream::default(); } else { w.palette.set_query(&v); } },
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::Tab => { e.prevent_default(); let q = m.peek().palette.query.clone(); let mode = m.write().tab(); if mode == AiMode::Ask && !q.trim().is_empty() { m.write().question = q.clone(); ask(q, now) } }
                        Key::Escape => { e.prevent_default(); let _ = m.write().escape(now); asked_at.set(None) }
                        Key::Enter => {
                            e.prevent_default();
                            if m.peek().mode == AiMode::Ask {
                                let ph = m.peek().answer.phase(now);
                                if ph == AnswerPhase::Done && !m.peek().actions.is_empty() { let i = m.peek().ask_active; m.write().run_action(i, now) }
                                else { let q = m.peek().question.clone(); ask(q, now) }
                            } else if let PaletteOutcome::Run(id, _) = m.write().palette.key(PaletteKey::Enter, now) { on_select.call(id) }
                        }
                        Key::ArrowDown | Key::ArrowUp => {
                            e.prevent_default();
                            let down = e.key() == Key::ArrowDown;
                            if m.peek().mode == AiMode::Ask { let n = m.peek().actions.len().max(m.peek().suggested_questions.len()).max(1); let a = m.peek().ask_active; m.write().ask_active = if down { (a + 1) % n } else { (a + n - 1) % n }; }
                            else { m.write().palette.key(if down { PaletteKey::Down } else { PaletteKey::Up }, now); let a = m.peek().palette.active; g.go(Some(a), true) }
                        }
                        _ => {}
                    },
                }
                span { class: "model", title: "{model}", Icon { name: "spark", size: 11.0 } "{model}" }
            }
            div { class: "abody", style: "height:{height}px",
                if !ask_mode {
                    div { class: "view clist", role: "listbox", onmounted: move |e| g.set_root(e.data()),
                        span { class: g.class("hl"), style: "{g.style()}", "aria-hidden": "true" }
                        for sec in secs.iter() {
                            div { key: "{sec.label}", class: "grp", role: "group", "aria-label": "{sec.label}",
                                div { class: "gl", "{sec.label}" }
                                for (item, hit) in sec.rows.iter().cloned() {
                                    {
                                        let i = flat_i;
                                        flat_i += 1;
                                        let c = st.palette.items[item].clone();
                                        let on = st.palette.active == i;
                                        let cid = c.id.clone();
                                        rsx! {
                                            div { key: "{c.id}", class: cls(&["row", if on && !g.measured() { "on" } else { "" }]), role: "option", "aria-selected": "{on}",
                                                onmounted: move |e| g.set_item(i, e.data()),
                                                onpointermove: move |_| if m.peek().palette.active != i { m.write().palette.hover(i); g.go(Some(i), false) },
                                                onclick: move |_| on_select.call(cid.clone()),
                                                span { class: "lead", if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 15.0 } } }
                                                span { class: "txt", span { class: "lb", {marked(&c.label, &hit.ranges)} } }
                                                if let Some(h) = c.hint.clone() { span { class: "hint", "{h}" } }
                                                if let Some(s) = c.shortcut.clone() { Keys { keys: s } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !st.palette.query.trim().is_empty() {
                            button { class: "row askrow", r#type: "button", onclick: move |_| { let q = m.peek().palette.query.clone(); m.write().tab(); m.write().question = q.clone(); ask(q, now) },
                                span { class: "lead", Icon { name: "spark", size: 15.0 } }
                                span { class: "txt", span { class: "lb", "Ask AI: " b { "{st.palette.query}" } } }
                                Keys { keys: "Tab" }
                            }
                        }
                    }
                } else if phase == AnswerPhase::Idle {
                    div { class: "view sugg",
                        div { class: "gl", "Suggested questions" }
                        for (i, q) in st.suggested_questions.iter().cloned().enumerate() {
                            button { key: "{i}", class: if st.ask_active == i { "row on" } else { "row" }, r#type: "button",
                                onclick: move |_| { m.write().question = q.clone(); ask(q.clone(), now) },
                                span { class: "lead", Icon { name: "comment", size: 14.0 } } span { class: "txt", span { class: "lb", "{q}" } }
                            }
                        }
                    }
                } else {
                    div { class: "view answer", role: "region", "aria-label": "Answer", "aria-busy": "{phase != AnswerPhase::Done}",
                        div { class: "qline", Icon { name: "comment", size: 13.0 } span { "{st.question}" } }
                        if phase == AnswerPhase::Thinking {
                            div { class: "thinking", span { class: "oxyn-shtext", "{thinking_label}…" } }
                        } else {
                            if !st.answer.sources.is_empty() {
                                div { class: "srcs",
                                    for (k, s) in st.answer.sources.iter().enumerate() {
                                        span { key: "{s.id}", class: cls(&["src", if cited.contains(&(k + 1)) { "on" } else { "" }, if lit_id.as_deref() == Some(s.id.as_str()) { "lit" } else { "" }]),
                                            span { class: "n", "{k + 1}" } if let Some(mk) = s.mark.clone() { {logo(&mk)} } "{s.title}" }
                                    }
                                }
                            }
                            p { class: "atext", {answer_view(&visible, &st.answer.sources, lit, |_| {})}
                                if phase == AnswerPhase::Streaming { span { class: "caret", "aria-hidden": "true" } }
                            }
                            if phase == AnswerPhase::Done && !st.actions.is_empty() {
                                div { class: "acts",
                                    div { class: "gl", "Next steps" }
                                    for (i, a) in st.actions.iter().cloned().enumerate() {
                                        {
                                            let s = a.state(now);
                                            rsx! {
                                                button { key: "{a.id}", class: cls(&["row act", if st.ask_active == i { "on" } else { "" }, match s { ActionState::Done => "done", ActionState::Running => "running", _ => "" }]), r#type: "button",
                                                    disabled: s == ActionState::Running || s == ActionState::Done,
                                                    onclick: move |_| m.write().run_action(i, now),
                                                    span { class: "lead", match s { ActionState::Running => rsx! { span { class: "oxyn-spn", style: "--s:13px" } }, ActionState::Done => rsx! { Icon { name: "check", size: 14.0, stroke: 2.0 } }, _ => rsx! { if let Some(mk) = a.mark.clone() { {logo(&mk)} } else { Icon { name: "bolt", size: 14.0 } } } } }
                                                    span { class: "txt", span { class: "lb", if s == ActionState::Done { "{a.done_label}" } else if s == ActionState::Running { "{a.label}…" } else { "{a.label}" } } }
                                                    if s == ActionState::Idle { Keys { keys: "Enter" } }
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
            div { class: "cfoot",
                span { class: "st", role: "status", "aria-live": "polite", "{st.status(now, &thinking_label)}" }
                span { "aria-hidden": "true", Keys { keys: "Tab" } if ask_mode { " Back to search" } else { " Ask AI" } }
                span { class: "lastk", "aria-hidden": "true", Keys { keys: "Esc" } " Close" }
            }
        }
    }
}

/// AI search bar: one search card. Typing lists grouped results at once; the first row is "Ask
/// AI about …" — press Tab (or click it) and a short answer streams in its place with numbered
/// citations that light their result below. Empty, it shows recent searches and suggested
/// questions. The card never changes size.
#[component]
pub fn AiSearchBar(
    /// Results (group = section).
    results: Vec<Command>,
    /// Recent searches.
    #[props(default)]
    recent: Vec<String>,
    /// Suggested questions.
    #[props(default)]
    suggestions: Vec<String>,
    /// Model name.
    #[props(default = "Claude Sonnet".to_string())]
    model: String,
    /// Placeholder.
    #[props(default = "Search docs, customers and issues".to_string())]
    placeholder: String,
    /// Initial query.
    #[props(default)]
    query: String,
    /// Answer on mount for the initial query (demo).
    #[props(default)]
    autoplay: bool,
    /// The answer (citations map to result ids through its sources).
    answer: ScriptedAnswer,
    /// Body height px.
    #[props(default = 320.0)]
    height: f32,
    /// Width px.
    #[props(default = 600.0)]
    width: f32,
    /// Clock.
    now: ClockMs,
    /// A result opened.
    #[props(default)]
    on_select: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = AiSearch::new(results.clone());
        s.recent = recent.clone();
        s.suggestions = suggestions.clone();
        s.set_query(&query);
        s
    });
    let mut started = use_signal(|| false);
    let lit = use_signal(|| None::<String>);
    let mut ans_s = use_signal(|| answer.clone());
    if *ans_s.peek() != answer {
        ans_s.set(answer.clone());
    }
    let mut ask = move |t: ClockMs| {
        let q = m.peek().query.clone();
        if q.trim().is_empty() {
            return;
        }
        let ans = ans_s.peek().clone();
        m.write().remember(&q);
        m.write().answer = AnswerStream::scripted(&ans.text, ans.sources.clone(), t);
    };
    if autoplay && !*started.peek() && !query.is_empty() {
        started.set(true);
        ask(now);
    }
    let st = m.read().clone();
    let secs = st.sections();
    let phase = st.answer.phase(now);
    let lit_id = lit();
    let mut flat = 1usize;
    let empty = st.query.trim().is_empty();
    rsx! {
        div { class: "oxyn-aisearch", style: "width:{width}px;max-width:100%",
            div { class: "cin",
                Icon { name: "search", size: 15.0 }
                input { class: "q", r#type: "search", role: "combobox", "aria-expanded": "true", "aria-label": "{placeholder}", placeholder: "{placeholder}", value: "{st.query}", autocomplete: "off",
                    oninput: move |e| m.write().set_query(&e.value()),
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::ArrowDown => { e.prevent_default(); m.write().step(true) }
                        Key::ArrowUp => { e.prevent_default(); m.write().step(false) }
                        Key::Tab if !m.peek().query.trim().is_empty() && m.peek().answer.asked.is_none() => { e.prevent_default(); ask(now) }
                        Key::Escape => { e.prevent_default(); m.write().escape(); }
                        Key::Enter => {
                            e.prevent_default();
                            let s = m.peek().clone();
                            if s.query.trim().is_empty() {
                                let a = s.active;
                                if a < s.recent.len() { let q = s.recent[a].clone(); m.write().set_query(&q) } else if let Some(q) = s.suggestions.get(a - s.recent.len()).cloned() { m.write().set_query(&q); ask(now) }
                            } else if s.active == 0 { ask(now) } else {
                                let ids: Vec<String> = s.sections().iter().flat_map(|x| x.rows.iter().map(|r| s.results[r.0].id.clone())).collect();
                                if let Some(id) = ids.get(s.active - 1) { on_select.call(id.clone()) }
                            }
                        }
                        _ => {}
                    },
                }
                if !empty { button { class: "oxyn-x", r#type: "button", "aria-label": "Clear search", onclick: move |_| m.write().set_query(""), Icon { name: "x", size: 12.0, stroke: 2.0 } } }
            }
            div { class: "abody", style: "height:{height}px",
                if empty {
                    div { class: "view",
                        if !st.recent.is_empty() {
                            div { class: "gl", "Recent" }
                            for (i, r) in st.recent.iter().cloned().enumerate() {
                                button { key: "r{i}", class: if st.active == i { "row on" } else { "row" }, r#type: "button", onclick: move |_| m.write().set_query(&r),
                                    span { class: "lead", Icon { name: "clock", size: 14.0 } } span { class: "txt", span { class: "lb", "{r}" } } }
                            }
                        }
                        if !st.suggestions.is_empty() {
                            div { class: "gl", "Ask AI" }
                            for (i, q) in st.suggestions.iter().cloned().enumerate() {
                                button { key: "s{i}", class: if st.active == st.recent.len() + i { "row on" } else { "row" }, r#type: "button", onclick: move |_| { m.write().set_query(&q); ask(now) },
                                    span { class: "lead", Icon { name: "spark", size: 14.0 } } span { class: "txt", span { class: "lb", "{q}" } } }
                            }
                        }
                    }
                } else {
                    div { class: "view",
                        if phase == AnswerPhase::Idle {
                            button { class: if st.active == 0 { "row askrow on" } else { "row askrow" }, r#type: "button", onclick: move |_| ask(now),
                                span { class: "lead", Icon { name: "spark", size: 15.0 } }
                                span { class: "txt", span { class: "lb", "Ask AI about " b { "“{st.query}”" } } }
                                Keys { keys: "Tab" }
                            }
                        } else {
                            div { class: "answer", role: "region", "aria-label": "AI answer", "aria-busy": "{phase != AnswerPhase::Done}",
                                div { class: "ah", Icon { name: "spark", size: 13.0 } span { "{model}" } if phase == AnswerPhase::Thinking { span { class: "oxyn-shtext", "Reading results…" } } }
                                if phase != AnswerPhase::Thinking {
                                    p { class: "atext", {answer_view(&st.answer.visible(now), &st.answer.sources, lit, |_| {})} if phase == AnswerPhase::Streaming { span { class: "caret", "aria-hidden": "true" } } }
                                }
                            }
                        }
                        for sec in secs.iter() {
                            div { key: "{sec.label}", class: "grp", role: "group", "aria-label": "{sec.label}",
                                div { class: "gl", "{sec.label}" }
                                for (item, hit) in sec.rows.iter().cloned() {
                                    {
                                        let i = flat;
                                        flat += 1;
                                        let c = st.results[item].clone();
                                        let badge = st.badge(&c.id, now);
                                        let on = st.active == i;
                                        let is_lit = lit_id.as_deref() == Some(c.id.as_str());
                                        let cid = c.id.clone();
                                        rsx! {
                                            div { key: "{c.id}", class: cls(&["row", if on { "on" } else { "" }, if is_lit { "lit" } else { "" }]), role: "option", "aria-selected": "{on}",
                                                onclick: move |_| on_select.call(cid.clone()),
                                                span { class: "lead", if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 15.0 } } }
                                                span { class: "txt", span { class: "lb", {marked(&c.label, &hit.ranges)} } if let Some(d) = c.description.clone() { span { class: "ds", "{d}" } } }
                                                if let Some(h) = c.hint.clone() { span { class: "hint", "{h}" } }
                                                if let Some(n) = badge { span { class: "nb", "aria-label": "Cited as {n}", "{n}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "cfoot", "aria-hidden": "true",
                span { Keys { keys: "Up" } Keys { keys: "Down" } " Move" }
                span { Keys { keys: "Tab" } " Ask AI" }
                span { class: "lastk model", Icon { name: "spark", size: 11.0 } "{model}" }
            }
        }
    }
}

/// Settings command: workspace settings on one page with a section nav, a search field that grows
/// into a palette and flies to a setting (the row breathes twice), and an unsaved-changes bar that
/// counts edits, validates, saves (or fails with Retry) and discards. ⌘K, Ctrl+K or `/` open the
/// search while focus is inside.
#[component]
pub fn SettingsCommand(
    /// Sections (id, label, one line).
    sections: Vec<(String, String, String)>,
    /// Settings.
    settings: Vec<Setting>,
    /// Saved values.
    saved: BTreeMap<String, SettingValue>,
    /// Ids offered before typing.
    #[props(default)]
    suggested: Vec<String>,
    /// Heading.
    #[props(default = "Settings".to_string())]
    title: String,
    /// Simulated save time ms.
    #[props(default = 900)]
    save_ms: u32,
    /// The first save fails (to show Retry).
    #[props(default)]
    fail_first: bool,
    /// Start with the palette open and this query.
    #[props(default)]
    open_query: Option<String>,
    /// Width px.
    #[props(default = 640.0)]
    width: f32,
    /// Saved (changed ids).
    #[props(default)]
    on_save: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = SettingsModel::new(sections.clone(), settings.clone(), saved.clone());
        s.suggested = suggested.clone();
        if let Some(q) = &open_query {
            s.open_palette();
            s.query = q.clone();
        }
        s
    });
    let mut fails = use_signal(|| fail_first);
    let mut rows = use_signal(BTreeMap::<String, Rc<MountedData>>::new);
    let mut flash_n = use_signal(|| 0u32);
    let st = m.read().clone();
    let res = st.results();
    let errors = st.errors();
    let changed = st.changed();
    let mut jump_to = move |id: String| {
        flash_n += 1;
        if let Some(d) = rows.peek().get(&id).cloned() {
            spawn(async move {
                let _ = d.scroll_to(ScrollBehavior::Smooth).await;
                let _ = d.set_focus(true).await;
            });
        }
    };
    let flash = st.flash.as_ref().map(|f| f.0.clone());
    let fl = flash_n();
    rsx! {
        div { class: "oxyn-settings", style: "width:{width}px;max-width:100%",
            onkeydown: move |e: KeyboardEvent| {
                let k = e.key();
                let md = e.modifiers().meta() || e.modifiers().ctrl();
                if (md && k == Key::Character("k".into())) || (k == Key::Character("/".into()) && !m.peek().open) {
                    e.prevent_default();
                    m.write().open_palette();
                }
            },
            header { class: "shd",
                h2 { "{title}" }
                button { class: cls(&["sfield", if st.open { "grown" } else { "" }]), r#type: "button", "aria-haspopup": "dialog", onclick: move |_| m.write().open_palette(),
                    Icon { name: "search", size: 14.0 } span { "Search settings" } Keys { keys: "Mod+K" }
                }
            }
            div { class: "sgrid",
                nav { class: "snav", "aria-label": "Sections",
                    for (id, label, _) in st.sections.iter().cloned() {
                        {
                            let n = st.edited_in(&id);
                            rsx! { a { key: "{id}", href: "#oxyn-set-{id}", class: "si", "{label}" if n > 0 { span { class: "dot", "aria-hidden": "true" } span { class: "sr", "Edited" } } } }
                        }
                    }
                }
                div { class: "sbody",
                    for (sid, slabel, sdesc) in st.sections.iter().cloned() {
                        section { key: "{sid}", id: "oxyn-set-{sid}", class: "ssec", "aria-labelledby": "oxyn-set-{sid}-h",
                            h3 { id: "oxyn-set-{sid}-h", "{slabel}" }
                            if !sdesc.is_empty() { p { class: "sdesc", "{sdesc}" } }
                            for s in st.settings.iter().filter(|s| s.section == sid).cloned() {
                                {
                                    let id = s.id.clone();
                                    let v = st.value(&id).cloned();
                                    let err = errors.iter().find(|e| e.0 == id).map(|e| e.1.clone());
                                    let edited = changed.contains(&id);
                                    let fid = format!("oxyn-set-f-{id}");
                                    let is_flash = flash.as_deref() == Some(id.as_str());
                                    let rk = if is_flash { fl } else { 0 };
                                    let control = match (&s.kind, v.clone()) {
                                        (SettingKind::Switch, Some(SettingValue::Bool(b))) => {
                                            let id2 = id.clone();
                                            rsx! { button { id: "{fid}", class: cls(&["oxyn-mini-switch", if b { "on" } else { "" }]), r#type: "button", role: "switch", "aria-checked": "{b}", "aria-describedby": "{fid}-h",
                                                onmounted: { let id3 = id.clone(); move |e: MountedEvent| { rows.write().insert(id3.clone(), e.data()); } },
                                                onclick: move |_| m.write().set(&id2, SettingValue::Bool(!b)), span { class: "kn" } } }
                                        }
                                        (SettingKind::Segmented(opts), Some(SettingValue::Text(cur))) => rsx! {
                                            div { id: "{fid}", class: "seg", role: "radiogroup", "aria-describedby": "{fid}-h",
                                                for o in opts.iter().cloned() {
                                                    {
                                                        let on = o == cur;
                                                        let id2 = id.clone();
                                                        let id3 = id.clone();
                                                        rsx! { button { key: "{o}", class: if on { "o on" } else { "o" }, r#type: "button", role: "radio", "aria-checked": "{on}",
                                                            onmounted: move |e: MountedEvent| if on { rows.write().insert(id3.clone(), e.data()); },
                                                            onclick: move |_| m.write().set(&id2, SettingValue::Text(o.clone())), "{o}" } }
                                                    }
                                                }
                                            }
                                        },
                                        (SettingKind::Select(opts), Some(SettingValue::Text(cur))) => {
                                            let id2 = id.clone();
                                            let id3 = id.clone();
                                            rsx! { select { id: "{fid}", class: "sel", "aria-describedby": "{fid}-h",
                                                onmounted: move |e| { rows.write().insert(id3.clone(), e.data()); },
                                                onchange: move |e| m.write().set(&id2, SettingValue::Text(e.value())),
                                                for o in opts.iter().cloned() { option { key: "{o}", value: "{o}", selected: o == cur, "{o}" } } } }
                                        }
                                        (SettingKind::Text { prefix, .. }, Some(SettingValue::Text(cur))) => {
                                            let id2 = id.clone();
                                            let id3 = id.clone();
                                            rsx! { div { class: cls(&["txtf", if err.is_some() { "bad" } else { "" }]),
                                                if let Some(p) = prefix.clone() { span { class: "pre", "{p}" } }
                                                input { id: "{fid}", r#type: "text", value: "{cur}", "aria-invalid": if err.is_some() { "true" }, "aria-describedby": "{fid}-h",
                                                    onmounted: move |e| { rows.write().insert(id3.clone(), e.data()); },
                                                    oninput: move |e| m.write().set(&id2, SettingValue::Text(e.value())) } } }
                                        }
                                        _ => rsx! {},
                                    };
                                    rsx! {
                                        div { key: "{id}-{rk}", class: cls(&["srow", if is_flash { "flash" } else { "" }, if edited { "edited" } else { "" }]),
                                            div { class: "sl",
                                                label { r#for: "{fid}", class: "lb", "{s.label}" if edited { span { class: "dot", "aria-hidden": "true" } span { class: "sr", " Edited" } } }
                                                div { id: "{fid}-h", class: if err.is_some() { "hint bad" } else { "hint" }, if let Some(e) = err.clone() { "{e}" } else { "{s.description}" } }
                                            }
                                            div { class: "sc", {control} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if st.bar() {
                div { class: cls(&["sbar", match st.save { SavePhase::Saved => "saved", SavePhase::Failed => "failed", SavePhase::Saving => "saving", SavePhase::Idle => "" }]), role: "status", "aria-live": "polite",
                    for _k in std::iter::once(st.bar_label()) { span { key: "{_k}", class: "bl", if st.save == SavePhase::Saved { Icon { name: "check", size: 13.0, stroke: 2.0 } } "{st.bar_label()}" } }
                    if st.save != SavePhase::Saved {
                        button { class: "oxyn-btn ghost xs", r#type: "button", disabled: st.save == SavePhase::Saving, onclick: move |_| m.write().discard(), "Discard" }
                        button { class: "oxyn-btn primary xs", r#type: "button", "aria-busy": if st.save == SavePhase::Saving { "true" },
                            onclick: move |_| { match m.write().save(ClockMs(0)) { Ok(_) => {}, Err(id) => jump_to(id) } },
                            if st.save == SavePhase::Saving { span { class: "oxyn-spn", style: "--s:11px" } }
                            if st.save == SavePhase::Failed { "Retry" } else { "Save" }
                        }
                    }
                    if st.save == SavePhase::Saving {
                        span { class: "oxyn-timer", style: "animation-duration:{save_ms}ms", onanimationend: move |_| {
                            let fail = *fails.peek();
                            if fail { fails.set(false); m.write().saved(false) } else { let ch = m.peek().changed(); m.write().saved(true); on_save.call(ch) }
                        } }
                    }
                    if st.save == SavePhase::Saved { span { class: "oxyn-timer", style: "animation-duration:1600ms", onanimationend: move |_| m.write().save = SavePhase::Idle } }
                }
            }
            if st.open {
                div { class: "oxyn-layer contained spal",
                    div { class: "oxyn-scrim", onpointerdown: move |_| m.write().close_palette() }
                    div { class: "oxyn-cmd elevated pal", role: "dialog", "aria-modal": "true", "aria-label": "Search settings",
                        div { class: "cin",
                            Icon { name: "search", size: 15.0 }
                            input { class: "q", r#type: "text", role: "combobox", "aria-expanded": "true", "aria-label": "Search settings", placeholder: "Search settings…", value: "{st.query}", autocomplete: "off",
                                onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                oninput: move |e| { let mut w = m.write(); w.query = e.value(); w.active = 0; },
                                onkeydown: move |e: KeyboardEvent| match e.key() {
                                    Key::ArrowDown => { e.prevent_default(); m.write().step(true) }
                                    Key::ArrowUp => { e.prevent_default(); m.write().step(false) }
                                    Key::Enter => { e.prevent_default(); if let Some(id) = m.write().enter(ClockMs(0)) { jump_to(id) } }
                                    Key::Escape => { e.prevent_default(); m.write().close_palette() }
                                    _ => {}
                                },
                            }
                        }
                        div { class: "clist", role: "listbox",
                            if res.is_empty() { div { class: "empty", div { class: "et", "No settings match" } } }
                            if st.query.trim().is_empty() && !res.is_empty() { div { class: "gl", "Suggested" } }
                            for (k, i) in res.iter().copied().enumerate() {
                                {
                                    let s = st.settings[i].clone();
                                    let sec = st.sections.iter().find(|x| x.0 == s.section).map(|x| x.1.clone()).unwrap_or_default();
                                    let on = st.active == k;
                                    let id = s.id.clone();
                                    let v = st.value(&s.id).map(|v| v.display()).unwrap_or_default();
                                    rsx! {
                                        div { key: "{s.id}", class: if on { "row on" } else { "row" }, role: "option", "aria-selected": "{on}",
                                            onpointermove: move |_| m.write().active = k,
                                            onclick: move |_| { m.write().jump(&id, ClockMs(0)); jump_to(id.clone()) },
                                            span { class: "lead", Icon { name: "cog", size: 14.0 } }
                                            span { class: "txt", span { class: "lb", "{s.label}" } span { class: "ds", "{sec} · {s.description}" } }
                                            span { class: "hint", "{v}" }
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

/// Search field: a search input whose clear button scales in inside a reserved slot (the width
/// never shifts); Escape clears. With `items` it filters them live with a gliding highlight and an
/// empty state (Beautiful UI Search, MIT). Oxyn plus: scope chips and an "Ask AI" row.
#[component]
pub fn SearchField(
    /// Label.
    label: String,
    /// Initial value.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default = "Search".to_string())]
    placeholder: String,
    /// Visually hide the label.
    #[props(default)]
    hide_label: bool,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Items to filter live.
    #[props(default)]
    items: Vec<String>,
    /// Scope chips (Oxyn plus).
    #[props(default)]
    scopes: Vec<String>,
    /// "Ask AI" row for the query (Oxyn plus).
    #[props(default)]
    ask_ai: bool,
    /// Width px.
    #[props(default = 320.0)]
    width: f32,
    /// Value changed.
    #[props(default)]
    on_change: EventHandler<String>,
    /// A result picked.
    #[props(default)]
    on_pick: EventHandler<String>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let mut scope = use_signal(|| scopes.first().cloned());
    let mut input = use_signal(|| None::<Rc<MountedData>>);
    let g = use_glide();
    let id = format!("oxyn-sf-{}", slug(&label));
    let q = v();
    let hits = filter_strings(&items, &q);
    let show_clear = clear_visible(&q, disabled);
    let has_list = !items.is_empty();
    rsx! {
        div { class: cls(&["oxyn-search", &size, if disabled { "disabled" } else { "" }, if has_list { "listing" } else { "" }]), style: "width:{width}px;max-width:100%",
            label { r#for: "{id}", class: if hide_label { "sr" } else { "lb" }, "{label}" }
            if !scopes.is_empty() {
                div { class: "scopes", role: "radiogroup", "aria-label": "Search in",
                    for s in scopes.iter().cloned() {
                        { let on = scope().as_deref() == Some(s.as_str()); rsx! { button { key: "{s}", class: if on { "sc on" } else { "sc" }, r#type: "button", role: "radio", "aria-checked": "{on}", onclick: move |_| scope.set(Some(s.clone())), "{s}" } } }
                    }
                }
            }
            div { class: "box",
                Icon { name: "search", size: 15.0 }
                input { id: "{id}", r#type: "search", value: "{q}", placeholder: "{placeholder}", disabled, autocomplete: "off",
                    "aria-controls": if has_list { "{id}-list" },
                    onmounted: move |e| input.set(Some(e.data())),
                    oninput: move |e| { v.set(e.value()); on_change.call(e.value()) },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); v.set(String::new()); on_change.call(String::new()) },
                }
                span { class: "slot",
                    if show_clear {
                        button { class: "clr", r#type: "button", "aria-label": "Clear search",
                            onclick: move |_| { v.set(String::new()); on_change.call(String::new()); if let Some(d) = input.peek().clone() { spawn(async move { let _ = d.set_focus(true).await; }); } },
                            Icon { name: "x", size: 11.0, stroke: 2.2 }
                        }
                    }
                }
            }
            if has_list {
                div { id: "{id}-list", class: "res", role: "listbox", "aria-label": "{label} results", onmounted: move |e| g.set_root(e.data()), onpointerleave: move |_| g.go(None, false),
                    span { class: g.class("hl"), style: "{g.style()}", "aria-hidden": "true" }
                    if ask_ai && !q.trim().is_empty() {
                        div { class: "row ask", role: "option", "aria-selected": "false", onclick: move |_| on_pick.call(format!("ask:{}", v.peek())),
                            Icon { name: "spark", size: 14.0 } span { "Ask AI: " b { "{q}" } }
                        }
                    }
                    if hits.is_empty() {
                        div { class: "empty", role: "status", div { class: "et", "No results found" } div { class: "eh", "Adjust your search to try again" } }
                    }
                    for (k, (i, r)) in hits.iter().cloned().enumerate() {
                        {
                            let t = items[i].clone();
                            let t2 = t.clone();
                            rsx! {
                                div { key: "{t}", class: "row", role: "option", "aria-selected": "false",
                                    onmounted: move |e| g.set_item(k, e.data()),
                                    onpointerenter: move |_| g.go(Some(k), false),
                                    onclick: move |_| on_pick.call(t2.clone()),
                                    span { class: "lb", {marked(&t, &r)} }
                                }
                            }
                        }
                    }
                }
                span { class: "sr", role: "status", "aria-live": "polite", if q.is_empty() { "" } else if hits.is_empty() { "No results" } else { "{hits.len()} results" } }
            }
        }
    }
}

/// Expanding search: an icon button that morphs into a search field with grouped, highlighted
/// results beneath; the panel tracks the field width. ↑ ↓ wrap, Enter picks (the field folds
/// back), Escape folds. Oxyn plus: recent queries and a ⌘K binding (`hotkey`).
#[component]
pub fn ExpandingSearch(
    /// Names the button, field and results.
    label: String,
    /// Items (group = section).
    items: Vec<Command>,
    /// Shown before typing.
    #[props(default)]
    suggestions: Vec<Command>,
    /// Heading for suggestions.
    #[props(default = "Recent".to_string())]
    suggestions_label: String,
    /// Placeholder (defaults to the label).
    #[props(default)]
    placeholder: Option<String>,
    /// Widest the field grows px.
    #[props(default = 360.0)]
    expanded_width: f32,
    /// Most results.
    #[props(default = 6)]
    max_results: usize,
    /// `start` · `center` · `end`.
    #[props(default = "end".to_string())]
    anchor: String,
    /// Tip under the empty state.
    #[props(default = "Try a shorter word or check the spelling.".to_string())]
    empty_hint: String,
    /// Start expanded.
    #[props(default)]
    default_expanded: bool,
    /// Initial query.
    #[props(default)]
    query: String,
    /// ⌘K expands it (while focus is inside the host).
    #[props(default)]
    hotkey: bool,
    /// Lane width px (the container it grows in).
    #[props(default = 420.0)]
    lane: f32,
    /// Picked (id).
    #[props(default)]
    on_select: EventHandler<String>,
    /// Expanded changed.
    #[props(default)]
    on_expanded_change: EventHandler<bool>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = XsModel::new(items.clone());
        s.suggestions = suggestions.clone();
        s.max_results = max_results;
        s.expanded_width = expanded_width;
        s.anchor = Anchor::from_id(&anchor);
        if default_expanded {
            s.expand();
            s.set_query(&query);
        }
        s
    });
    let mut picked = use_signal(|| None::<String>);
    let g = use_glide();
    let id = format!("oxyn-xs-{}", slug(&label));
    let st = m.read().clone();
    let w = st.width(lane, 36.0);
    let secs = st.sections();
    let mut flat_i = 0usize;
    let ph = placeholder.clone().unwrap_or(label.clone());
    let act_id = st
        .active
        .and_then(|a| st.flat().get(a).copied())
        .map(|i| format!("{id}-o{i}"));
    let mut handle = move |o: SearchOutcome| match o {
        SearchOutcome::Select(id) => {
            picked.set(Some(id.clone()));
            on_select.call(id);
            on_expanded_change.call(false)
        }
        SearchOutcome::Folded => on_expanded_change.call(false),
        SearchOutcome::None => {}
    };
    rsx! {
        div { class: cls(&["oxyn-xsearch", st.anchor.id(), if st.expanded { "open" } else { "" }]), style: "width:{lane}px;max-width:100%",
            onkeydown: move |e: KeyboardEvent| if hotkey && (e.modifiers().meta() || e.modifiers().ctrl()) && e.key() == Key::Character("k".into()) { e.prevent_default(); m.write().expand(); on_expanded_change.call(true) },
            div { class: "field", style: "width:{w:.0}px",
                if st.expanded {
                    Icon { name: "search", size: 15.0 }
                    input { class: "q", r#type: "text", role: "combobox", "aria-expanded": "{!secs.is_empty()}", "aria-controls": "{id}-list", "aria-autocomplete": "list",
                        "aria-activedescendant": if let Some(a) = &act_id { "{a}" }, "aria-label": "{label}", placeholder: "{ph}", value: "{st.query}", autocomplete: "off",
                        onmounted: move |e| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                        oninput: move |e| { m.write().set_query(&e.value()); let a = m.peek().active; g.go(a, true) },
                        onkeydown: move |e: KeyboardEvent| match e.key() {
                            Key::ArrowDown | Key::ArrowUp => { e.prevent_default(); m.write().step(e.key() == Key::ArrowDown); let a = m.peek().active; g.go(a, true) }
                            Key::Enter => { e.prevent_default(); let o = m.write().enter(); handle(o) }
                            Key::Escape => { e.prevent_default(); let o = m.write().escape(); handle(o) }
                            _ => {}
                        },
                    }
                    button { class: "oxyn-x", r#type: "button", "aria-label": "Close search", onclick: move |_| { let o = m.write().escape(); handle(o) }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                } else {
                    button { class: "ibtn", r#type: "button", "aria-label": "{label}", "aria-expanded": "false",
                        onclick: move |_| { m.write().expand(); on_expanded_change.call(true) },
                        Icon { name: "search", size: 16.0 }
                    }
                }
            }
            if st.expanded && (!secs.is_empty() || !st.query.trim().is_empty()) {
                div { id: "{id}-list", class: "panel", style: "width:{w:.0}px", role: "listbox", "aria-label": "{label} results",
                    onmounted: move |e| g.set_root(e.data()),
                    span { class: g.class("hl"), style: "{g.style()}", "aria-hidden": "true" }
                    if secs.is_empty() {
                        div { class: "empty", div { class: "et", "No results for “{st.query}”" } div { class: "eh", "{empty_hint}" } }
                    }
                    for sec in secs.iter() {
                        div { key: "{sec.label}", class: "grp", role: "group", "aria-label": "{sec.label}",
                            div { class: "gl", if st.query.trim().is_empty() { "{suggestions_label}" } else { "{sec.label}" } }
                            for (item, hit) in sec.rows.iter().cloned() {
                                {
                                    let i = flat_i;
                                    flat_i += 1;
                                    let c = st.item(item).cloned().unwrap_or_default();
                                    let on = st.active == Some(i);
                                    rsx! {
                                        div { key: "{c.id}", id: "{id}-o{item}", class: cls(&["row", if on && !g.measured() { "on" } else { "" }]), role: "option", "aria-selected": "{on}",
                                            onmounted: move |e| g.set_item(i, e.data()),
                                            onpointermove: move |_| { m.write().active = Some(i); g.go(Some(i), false) },
                                            onclick: move |_| { let o = m.write().pick(item); handle(o) },
                                            span { class: "lead", if let Some(ic) = c.icon.clone() { Icon { name: ic, size: 14.0 } } else { Icon { name: if st.query.trim().is_empty() { "clock" } else { "file" }, size: 14.0 } } }
                                            span { class: "txt", span { class: "lb", {marked(&c.label, &hit.ranges)} } }
                                            if let Some(h) = c.hint.clone() { span { class: "hint", "{h}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !st.expanded { if let Some(p) = picked() { span { class: "picked", "Opened {p}" } } }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce()}" }
        }
    }
}

fn mods_of(e: &KeyboardEvent) -> Mods {
    let m = e.modifiers();
    Mods {
        meta: m.meta(),
        ctrl: m.ctrl(),
        alt: m.alt(),
        shift: m.shift(),
    }
}

/// Shortcut recorder: records a key combination into caps, requires ⌘ / Ctrl / ⌥ (function keys
/// excepted), warns about combinations the browser keeps and asks before taking one already bound
/// (Use anyway / Cancel). Backspace clears; Escape cancels; Tab leaves. Oxyn plus: two-key
/// sequences ("G then I") and a reset button.
#[component]
pub fn ShortcutRecorder(
    /// Label.
    label: String,
    /// Visually hide the label.
    #[props(default)]
    hide_label: bool,
    /// Value (`mod+shift+k`).
    #[props(default)]
    value: Option<String>,
    /// What reset restores (defaults to the value).
    #[props(default)]
    reset_value: Option<String>,
    /// Shortcuts in use.
    #[props(default)]
    bindings: Vec<Binding>,
    /// Warn about reserved combinations.
    #[props(default = true)]
    warn_reserved: bool,
    /// Require a modifier.
    #[props(default = true)]
    require_modifier: bool,
    /// `mac` · `other`.
    #[props(default = "mac".to_string())]
    platform: String,
    /// Placeholder.
    #[props(default = "Record shortcut".to_string())]
    placeholder: String,
    /// Hint.
    #[props(default)]
    description: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Two-key sequences.
    #[props(default)]
    sequences: bool,
    /// Start recording.
    #[props(default)]
    recording: bool,
    /// Value changed (value, replaced binding label).
    #[props(default)]
    on_change: EventHandler<(Option<String>, Option<String>)>,
) -> Element {
    let p = if platform == "other" {
        Platform::Other
    } else {
        Platform::Mac
    };
    let mut m = use_signal(|| {
        let mut r = Recorder::new(value.as_deref(), p);
        r.reset_value = reset_value.clone().or(value.clone());
        r.bindings = bindings.clone();
        r.warn_reserved = warn_reserved;
        r.require_modifier = require_modifier;
        r.sequences = sequences;
        if recording && !disabled {
            r.start();
        }
        r
    });
    let mut btn = use_signal(|| None::<Rc<MountedData>>);
    let id = format!("oxyn-sr-{}", slug(&label));
    let st = m.read().clone();
    let rec = st.recording();
    let held = match &st.state {
        RecState::Recording { held, .. } => *held,
        _ => Mods::default(),
    };
    let first = match &st.state {
        RecState::Recording { first, .. } => first.clone(),
        _ => None,
    };
    let emit = move |o: RecOutcome| {
        if let RecOutcome::Changed(v, b) = o {
            on_change.call((v, b.map(|b| b.label)));
        }
    };
    let held_caps: Vec<String> = {
        let mut v = vec![];
        if p == Platform::Mac {
            if held.ctrl {
                v.push("⌃".to_string())
            }
            if held.alt {
                v.push("⌥".into())
            }
            if held.shift {
                v.push("⇧".into())
            }
            if held.meta {
                v.push("⌘".into())
            }
        } else {
            if held.ctrl {
                v.push("Ctrl".to_string())
            }
            if held.alt {
                v.push("Alt".into())
            }
            if held.shift {
                v.push("Shift".into())
            }
            if held.meta {
                v.push("Win".into())
            }
        }
        v
    };
    let refocus = move || {
        if let Some(d) = btn.peek().clone() {
            spawn(async move {
                let _ = d.set_focus(true).await;
            });
        }
    };
    rsx! {
        div { class: cls(&["oxyn-recorder", &size, if disabled { "disabled" } else { "" }]),
            label { r#for: "{id}", class: if hide_label { "sr" } else { "lb" }, "{label}" }
            div { class: "rrow",
                button { id: "{id}", class: cls(&["rec", if rec { "on" } else { "" }, match st.state { RecState::Conflict { .. } => "warn", RecState::Invalid(_) => "bad", _ => "" }]),
                    r#type: "button", disabled, "aria-pressed": "{rec}", "aria-label": "{st.aria(&label)}", "aria-describedby": "{id}-msg",
                    onmounted: move |e| btn.set(Some(e.data())),
                    onclick: move |_| if !m.peek().recording() { m.write().start() },
                    onkeydown: move |e: KeyboardEvent| {
                        let rec = m.peek().recording();
                        let k = e.key().to_string();
                        if !rec && matches!(k.as_str(), "Enter" | " ") { e.prevent_default(); m.write().start(); return; }
                        if rec && k != "Tab" { e.prevent_default(); }
                        let o = m.write().key_down(&k, mods_of(&e), ClockMs(0));
                        emit(o)
                    },
                    onkeyup: move |e: KeyboardEvent| if m.peek().recording() { let mut w = m.write(); if let RecState::Recording { held, .. } = &mut w.state { *held = mods_of(&e) } },
                    onblur: move |_| if m.peek().recording() { m.write().state = RecState::Idle },
                    if rec {
                        span { class: "dot", "aria-hidden": "true" }
                        span { class: "caps", "aria-hidden": "true",
                            if let Some(f) = first.clone() { for (i, c) in caps(&f, p).into_iter().enumerate() { kbd { key: "f{i}", class: "cap", "{c}" } } span { class: "then", "then" } }
                            for (i, c) in held_caps.iter().enumerate() { kbd { key: "h{i}", class: "cap held", "{c}" } }
                            if held_caps.is_empty() && first.is_none() { span { class: "ph", "Press keys…" } }
                        }
                    } else if let Some(v) = st.value.clone() {
                        span { class: "caps set", "aria-hidden": "true",
                            for (i, part) in v.split(' ').enumerate() {
                                if i > 0 { span { class: "then", "then" } }
                                for (j, c) in caps(part, p).into_iter().enumerate() { kbd { key: "{i}-{j}", class: "cap", style: "--i:{j}", "{c}" } }
                            }
                        }
                    } else {
                        span { class: "ph", "{placeholder}" }
                    }
                }
                if st.value.is_some() && !rec {
                    button { class: "oxyn-x", r#type: "button", "aria-label": "Clear shortcut", disabled, onclick: move |_| { let o = m.write().clear(); emit(o); refocus() }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                }
                if st.reset_value.is_some() && st.value != st.reset_value && !rec {
                    button { class: "oxyn-btn ghost xs", r#type: "button", disabled, onclick: move |_| { let o = m.write().reset(); emit(o); refocus() }, Icon { name: "undo", size: 12.0 } "Reset" }
                }
            }
            div { id: "{id}-msg", class: cls(&["msg", match st.state { RecState::Conflict { .. } => "warn open", RecState::Invalid(_) => "bad open", _ => if description.is_some() || rec { "open" } else { "" } }]),
                match st.state.clone() {
                    RecState::Conflict { shortcut, with, reserved } => rsx! {
                        Icon { name: "alert", size: 13.0 }
                        span { if reserved { "{spoken(&shortcut, p)} is {with.to_lowercase()}." } else { "{spoken(&shortcut, p)} is already used by {with}." } }
                        button { class: "oxyn-btn secondary xs", r#type: "button", onclick: move |_| { let o = m.write().use_anyway(); emit(o); refocus() }, "Use anyway" }
                        button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| { m.write().cancel(); refocus() }, "Cancel" }
                    },
                    RecState::Invalid(msg) => rsx! { Icon { name: "info", size: 13.0 } span { "{msg}" } },
                    RecState::Recording { .. } => rsx! { span { if sequences { "Press a shortcut, or two keys in a row. Esc cancels." } else { "Press a shortcut. Esc cancels, Backspace clears." } } },
                    RecState::Idle => rsx! { if let Some(d) = description.clone() { span { "{d}" } } },
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", if let Some(msg) = st.message.clone() { "{msg}" } }
        }
    }
}

/// Shortcut list: a searchable, grouped cheatsheet. Holding a modifier lights its caps and fades
/// rows that don't use it. Two columns from 520 px.
#[component]
pub fn ShortcutList(
    /// Groups.
    groups: Vec<ShortcutGroup>,
    /// Accessible name.
    #[props(default = "Keyboard shortcuts".to_string())]
    label: String,
    /// Search field.
    #[props(default = true)]
    searchable: bool,
    /// Light held caps and fade other rows.
    #[props(default = true)]
    highlight_pressed: bool,
    /// `mac` · `other`.
    #[props(default = "mac".to_string())]
    platform: String,
    /// Width px.
    #[props(default = 560.0)]
    width: f32,
) -> Element {
    let p = if platform == "other" {
        Platform::Other
    } else {
        Platform::Mac
    };
    let mut q = use_signal(String::new);
    let mut held = use_signal(Mods::default);
    let shown = filter_cheatsheet(&groups, &q(), p);
    let h = held();
    rsx! {
        div { class: "oxyn-cheats", style: "width:{width}px;max-width:100%", "aria-label": "{label}", role: "region", tabindex: "-1",
            onkeydown: move |e: KeyboardEvent| if highlight_pressed { held.set(mods_of(&e)) },
            onkeyup: move |e: KeyboardEvent| if highlight_pressed { held.set(mods_of(&e)) },
            if searchable {
                div { class: "box", Icon { name: "search", size: 14.0 }
                    input { r#type: "search", placeholder: "Search shortcuts", "aria-label": "Search shortcuts", value: "{q}", oninput: move |e| q.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
                }
            }
            div { class: "cols",
                if shown.is_empty() { div { class: "empty", "No shortcuts match" } }
                for gr in shown.iter() {
                    div { key: "{gr.label}", class: "cg", role: "group", "aria-label": "{gr.label}",
                        div { class: "gl", "{gr.label}" }
                        for (l, s) in gr.items.iter().cloned() {
                            div { key: "{l}", class: if dimmed(&s, h, p) { "cr dim" } else { "cr" },
                                span { class: "cl", "{l}" }
                                span { class: "keys", role: "img", "aria-label": "{spoken(&s, p)}",
                                    for (i, c) in caps(&s, p).into_iter().enumerate() {
                                        {
                                            let lit = (h.meta && c == "⌘") || (h.shift && (c == "⇧" || c == "Shift")) || (h.alt && (c == "⌥" || c == "Alt")) || (h.ctrl && (c == "⌃" || c == "Ctrl"));
                                            rsx! { kbd { key: "{i}", class: if lit { "cap lit" } else { "cap" }, "aria-hidden": "true", "{c}" } }
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

/// Control center: workspace quick settings. Tiles toggle Focus, Notifications and Presenting;
/// their chevrons open a detail over the grid (focus modes + a session dial that counts across
/// twelve o'clock and snaps to 5 minutes; channel switches). Meters for alert volume and text size
/// follow the pointer 1:1 with rubber-banded ends; a tap jumps. Oxyn plus: agent tiles (effort,
/// permissions, voice) and "silence while agents run".
#[component]
pub fn ControlCenter(
    /// Focus modes.
    modes: Vec<FocusMode>,
    /// Channels.
    channels: Vec<Channel>,
    /// People who can still reach you.
    #[props(default)]
    people: Vec<String>,
    /// Initial volume 0–100.
    #[props(default = 60.0)]
    default_volume: f32,
    /// Initial text size 85–130.
    #[props(default = 100.0)]
    default_text_size: f32,
    /// Now, minutes after midnight.
    #[props(default = 890)]
    now_min: u32,
    /// Agent tiles.
    #[props(default = true)]
    agent_tiles: bool,
    /// Open this detail on mount (`focus`, `notifications`, `agent`).
    #[props(default)]
    open_detail: Option<String>,
    /// Panel name.
    #[props(default = "Quick settings".to_string())]
    label: String,
    /// Any change (focus label, notifications label).
    #[props(default)]
    on_state_change: EventHandler<(String, String)>,
) -> Element {
    let mut m = use_signal(|| {
        let mut c = CcModel::new(modes.clone(), channels.clone(), now_min);
        c.people = people.clone();
        c.volume = default_volume;
        c.text_size = default_text_size;
        match open_detail.as_deref() {
            Some("focus") => c.open(Detail::Focus),
            Some("notifications") => c.open(Detail::Notifications),
            Some("agent") => c.open(Detail::Agent),
            _ => {}
        }
        c
    });
    let mut dial_box = use_signal(|| None::<(f64, f64)>);
    let mut dialing = use_signal(|| false);
    let mut meter_w = use_signal(|| 260.0f32);
    let st = m.read().clone();
    let changed = move || {
        let s = m.peek();
        on_state_change.call((s.focus_label(), s.notifications_label()))
    };
    let meter = move |which: Meter, icon: &'static str, name: &'static str| {
        let s = m.read();
        let (lo, hi, _) = which.range();
        let v = s.meter(which);
        let pct = (v - lo) / (hi - lo) * 100.0;
        let held = s.held() == Some(which);
        let txt = s.meter_text(which);
        drop(s);
        rsx! {
            div { class: cls(&["meter", if held { "held" } else { "" }]),
                role: "slider", tabindex: "0", "aria-label": "{name}", "aria-valuemin": "{lo}", "aria-valuemax": "{hi}", "aria-valuenow": "{v}", "aria-valuetext": "{txt}",
                onmounted: move |e| { let d = e.data(); spawn(async move { if let Ok(r) = d.get_client_rect().await { meter_w.set(r.size.width as f32) } }); },
                onpointerdown: move |e| { m.write().grab(which, e.element_coordinates().x as f32) },
                onpointermove: move |e| if m.peek().held() == Some(which) { let w = *meter_w.peek(); m.write().drag_to(e.element_coordinates().x as f32, w); },
                onpointerup: move |e| { let w = *meter_w.peek(); m.write().release(e.element_coordinates().x as f32, w); changed() },
                onpointerleave: move |e| if m.peek().held() == Some(which) { let w = *meter_w.peek(); m.write().release(e.element_coordinates().x as f32 + 1000.0, w); },
                onkeydown: move |e: KeyboardEvent| {
                    let k = match e.key() { Key::ArrowRight | Key::ArrowUp => Some(StepKey::Inc), Key::ArrowLeft | Key::ArrowDown => Some(StepKey::Dec), Key::PageUp => Some(StepKey::PageUp), Key::PageDown => Some(StepKey::PageDown), Key::Home => Some(StepKey::Home), Key::End => Some(StepKey::End), _ => None };
                    if let Some(k) = k { e.prevent_default(); m.write().meter_key(which, k); changed() }
                },
                span { class: "fill", style: "width:{pct:.1}%" }
                span { class: "mi", Icon { name: icon, size: 14.0 } }
                span { class: "mn", "{name}" }
                span { class: "mv", "{txt}" }
            }
        }
    };
    let tile = move |id: &'static str,
                     icon: &'static str,
                     name: &'static str,
                     on: bool,
                     line: String,
                     detail: Option<Detail>| {
        rsx! {
            div { class: cls(&["tile", if on { "on" } else { "" }]),
                button { class: "tb", r#type: "button", "aria-pressed": "{on}",
                    onclick: move |_| { match id { "focus" => m.write().toggle_focus(), "notifications" => m.write().toggle_notifications(), "presenting" => m.write().toggle_presenting(), _ => { let v = m.peek().agent.voice; m.write().agent.voice = !v } } changed() },
                    span { class: "ti", Icon { name: icon, size: 16.0 } }
                    span { class: "tt", "{name}" }
                    for _k in std::iter::once(line.to_string()) { span { key: "{_k}", class: "tl", "{line}" } }
                }
                if let Some(d) = detail {
                    button { class: "chev", r#type: "button", "aria-haspopup": "dialog", "aria-label": "{name} settings", onclick: move |_| m.write().open(d), Icon { name: "chevron-right", size: 13.0, stroke: 2.0 } }
                }
            }
        }
    };
    let knob = st.knob_angle();
    let r = 64.0f32;
    let (kx, ky) = (
        (knob.to_radians().sin() * r),
        (-knob.to_radians().cos() * r),
    );
    let frac = (st.draft_minutes / 120.0).clamp(0.0, 1.0);
    let circ = std::f32::consts::TAU * r;
    rsx! {
        div { class: "oxyn-cc", role: "group", "aria-label": "{label}",
            div { class: "grid", inert: st.detail.is_some(), "aria-hidden": if st.detail.is_some() { "true" },
                {tile("focus", "moon", "Focus", st.focus.is_some(), st.focus_label(), Some(Detail::Focus))}
                {tile("notifications", "bell", "Notifications", st.notifications(), st.notifications_label(), Some(Detail::Notifications))}
                {tile("presenting", "monitor", "Presenting", st.presenting, if st.presenting { "On".into() } else { "Off".into() }, None)}
                if agent_tiles { {tile("agent", "agents", "Agents", st.agent.voice, format!("{} effort · {}", st.agent.effort, st.agent.permissions), Some(Detail::Agent))} }
                {meter(Meter::Volume, "volume", "Alert volume")}
                {meter(Meter::TextSize, "type", "Text size")}
            }
            if let Some(d) = st.detail {
                div { class: "scrim", onpointerdown: move |_| m.write().close() }
                div { class: "detail", role: "dialog", "aria-modal": "true", "aria-label": match d { Detail::Focus => "Focus", Detail::Notifications => "Notifications", Detail::Agent => "Agents" }, tabindex: "-1",
                    onmounted: move |e| { let dd = e.data(); spawn(async move { let _ = dd.set_focus(true).await; }); },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); m.write().close() },
                    header { class: "dh2",
                        button { class: "oxyn-btn ghost xs", r#type: "button", "aria-label": "Back", onclick: move |_| m.write().close(), Icon { name: "chevron-left", size: 13.0, stroke: 2.0 } }
                        h3 { match d { Detail::Focus => "Focus", Detail::Notifications => "Notifications", Detail::Agent => "Agents" } }
                    }
                    match d {
                        Detail::Focus => rsx! {
                            div { class: "modes", role: "radiogroup", "aria-label": "Focus mode",
                                for (i, fm) in st.modes.iter().cloned().enumerate() {
                                    button { key: "{fm.id}", class: if st.draft_mode == i { "mode on" } else { "mode" }, r#type: "button", role: "radio", "aria-checked": "{st.draft_mode == i}",
                                        onclick: move |_| m.write().pick_mode(i),
                                        Icon { name: fm.icon.clone(), size: 14.0 } span { class: "ml", "{fm.label}" } span { class: "md", "{fm.description}" } }
                                }
                            }
                            div { class: cls(&["dial", if dialing() { "turning" } else { "" }]), role: "slider", tabindex: "0", "aria-label": "Session length", "aria-valuemin": "5", "aria-valuemax": "120",
                                "aria-valuenow": "{st.draft_minutes.round()}", "aria-valuetext": "{st.draft_minutes.round()} minutes, until {st.draft_until()}",
                                onmounted: move |e| { let d = e.data(); spawn(async move { if let Ok(r) = d.get_client_rect().await { dial_box.set(Some((r.origin.x + r.size.width / 2.0, r.origin.y + r.size.height / 2.0))) } }); },
                                onpointerdown: move |e| { dialing.set(true); let c = e.client_coordinates(); if let Some((cx, cy)) = *dial_box.peek() { let a = ((c.x - cx).atan2(-(c.y - cy))).to_degrees() as f32; m.write().dial_to((a + 360.0) % 360.0) } },
                                onpointermove: move |e| if dialing() { let c = e.client_coordinates(); if let Some((cx, cy)) = *dial_box.peek() { let a = ((c.x - cx).atan2(-(c.y - cy))).to_degrees() as f32; m.write().dial_to((a + 360.0) % 360.0) } },
                                onpointerup: move |_| { dialing.set(false); m.write().dial_release(); },
                                onpointerleave: move |_| if dialing() { dialing.set(false); m.write().dial_release(); },
                                onkeydown: move |e: KeyboardEvent| {
                                    let k = match e.key() { Key::ArrowRight | Key::ArrowUp => Some(StepKey::Inc), Key::ArrowLeft | Key::ArrowDown => Some(StepKey::Dec), Key::PageUp => Some(StepKey::PageUp), Key::PageDown => Some(StepKey::PageDown), Key::Home => Some(StepKey::Home), Key::End => Some(StepKey::End), _ => None };
                                    if let Some(k) = k { e.prevent_default(); m.write().dial_key(k); }
                                },
                                svg { view_box: "-80 -80 160 160", width: "164", height: "164", "aria-hidden": "true",
                                    circle { r: "{r}", class: "track", fill: "none" }
                                    circle { r: "{r}", class: "prog", fill: "none", stroke_dasharray: "{circ * frac:.1} {circ:.1}", transform: "rotate(-90)" }
                                    for t in 0..12 { line { key: "{t}", class: "tick", x1: "0", y1: "-74", x2: "0", y2: "-70", transform: "rotate({t * 30})" } }
                                    circle { class: "knob", cx: "{kx:.1}", cy: "{ky:.1}", r: "8" }
                                }
                                div { class: "dv", span { class: "mins", "{st.draft_minutes.round()}" } span { class: "unit", "min" } span { class: "until", "until {st.draft_until()}" } }
                            }
                            if !st.people.is_empty() {
                                div { class: "people", Icon { name: "users", size: 13.0 } span { "Can still reach you: {st.people.join(\", \")}" } }
                            }
                            div { class: "dfoot",
                                if st.focus.is_some() { button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| { m.write().toggle_focus(); m.write().close(); changed() }, "End session" } }
                                button { class: "oxyn-btn primary sm", r#type: "button", onclick: move |_| { m.write().apply_session(); changed() }, "{st.session_button()}" }
                            }
                        },
                        Detail::Notifications => rsx! {
                            div { class: "chans",
                                for c in st.channels.iter().cloned() {
                                    {
                                        let cid = c.id.clone();
                                        rsx! {
                                            div { key: "{c.id}", class: "chan",
                                                span { class: "cl", "{c.label}" }
                                                button { class: cls(&["oxyn-mini-switch", if c.on { "on" } else { "" }]), r#type: "button", role: "switch", "aria-checked": "{c.on}", "aria-label": "{c.label}",
                                                    onclick: move |_| { m.write().toggle_channel(&cid); changed() }, span { class: "kn" } }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Detail::Agent => rsx! {
                            div { class: "chans",
                                div { class: "chan", span { class: "cl", "Effort" }
                                    div { class: "seg", role: "radiogroup", "aria-label": "Effort",
                                        for e in ["low", "medium", "high"] { button { key: "{e}", class: if st.agent.effort == e { "o on" } else { "o" }, r#type: "button", role: "radio", "aria-checked": "{st.agent.effort == e}", onclick: move |_| m.write().agent.effort = e.to_string(), "{e}" } }
                                    }
                                }
                                div { class: "chan", span { class: "cl", "Permissions" }
                                    div { class: "seg", role: "radiogroup", "aria-label": "Permissions",
                                        for e in ["ask", "auto-edit", "full"] { button { key: "{e}", class: if st.agent.permissions == e { "o on" } else { "o" }, r#type: "button", role: "radio", "aria-checked": "{st.agent.permissions == e}", onclick: move |_| m.write().agent.permissions = e.to_string(), "{e}" } }
                                    }
                                }
                                div { class: "chan", span { class: "cl", "Silence alerts while agents run" }
                                    button { class: cls(&["oxyn-mini-switch", if st.agent.dnd_while_running { "on" } else { "" }]), r#type: "button", role: "switch", "aria-checked": "{st.agent.dnd_while_running}", "aria-label": "Silence alerts while agents run",
                                        onclick: move |_| { let v = m.peek().agent.dnd_while_running; m.write().agent.dnd_while_running = !v }, span { class: "kn" } }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn command_components_render() {
        let h = render(|| {
            let items = vec![
                Command::new("a", "Share project")
                    .group("Actions")
                    .shortcut("Mod+S"),
                Command::new("b", "Summarize")
                    .group("Suggestions")
                    .logo("AI")
                    .suggested(),
            ];
            let ans = ScriptedAnswer {
                text: "Use the link [1].".into(),
                sources: vec![Source::new("a", "Share")],
                actions: vec![],
            };
            rsx! {
                CommandPalette { items: items.clone(), variant: "menu", query: "sha" }
                AiCommand { items: items.clone(), answer: ans.clone(), now: ClockMs(0) }
                AiSearchBar { results: items.clone(), answer: ans, now: ClockMs(0), query: "share" }
                SearchField { label: "Search", value: "bill", items: vec!["Billing".to_string(), "Team".to_string()] }
                ExpandingSearch { label: "Search docs", items: items.clone(), default_expanded: true, query: "sha" }
                ShortcutRecorder { label: "Search", value: Some("mod+k".to_string()) }
            }
        });
        assert!(h.contains(r#"role="combobox""#) && h.contains(r#"class="hit""#));
        assert!(h.contains("Ask AI about") && h.contains("Search pages and actions"));
        assert!(
            h.contains("Clear search") && h.contains("1 results") || h.contains("Clear search")
        );
        assert!(h.contains("Search, Command K") && h.contains(r#"aria-pressed="false""#));
    }

    #[test]
    fn control_center_renders() {
        let h = render(|| {
            rsx! {
                ControlCenter {
                    modes: vec![FocusMode::new("deep", "Deep work", "brain", "Mentions only", 50)],
                    channels: vec![Channel::new("m", "Mentions", true), Channel::new("d", "Deploys", false)],
                    open_detail: Some("focus".to_string()),
                }
            }
        });
        assert!(h.contains("1 of 2 channels") && h.contains(r#"aria-pressed="false""#));
        assert!(h.contains(r#"role="slider""#) && h.contains("50 minutes, until"));
        assert!(h.contains("Start session"));
    }
}
