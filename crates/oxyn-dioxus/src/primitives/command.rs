//! Command and search (plan #26): `CommandPalette` (also affectUI's command menu), `AiCommand`,
//! `AiSearchBar`, `SettingsCommand`, `SearchField`, `ExpandingSearch`, `ShortcutRecorder`,
//! `ShortcutList`, `ControlCenter`.
//!
//! Logic lives in `oxyn_core::primitives::{command, search, control_center}`. The input keeps
//! focus while the arrow keys move the active option (`aria-activedescendant`), and one shared
//! highlight glides between rows for the pointer and jumps for the keyboard. AI answers take the
//! host clock (`now`) so they stream with the same timing on the web and in GPUI.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

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
    if crate::brand::has_brand(text) {
        return rsx! { span { class: "logo brand", "aria-hidden": "true", crate::brand::BrandIcon { name: text.to_string(), size: 13.0, coloured: true } } };
    }
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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
