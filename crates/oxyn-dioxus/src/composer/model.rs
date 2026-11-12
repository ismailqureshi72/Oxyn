//! `ModelPicker` (`docs/specs/composer-model-picker.md`). Controlled: the composer (or host) owns
//! the selection and the [`Picker`] state; this draws the pill and popover and reports intent.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::{
        DraftNeeds, ModelCatalog, ModelInfo, ModelPickerVariant, ModelSelection, ModelStatus,
        Picker, PickerKey, PickerRow,
        model::{Tier3, format_context, status_note},
    },
};
use oxyn_tokens::ColorRole;

use super::icons::Icon;
use crate::color_var;

/// A provider mark: a tinted disc with the provider's initial (hosts may swap in a logo).
pub(crate) fn provider_color(provider: &str) -> String {
    const P: [ColorRole; 6] = [
        ColorRole::AgentOrange,
        ColorRole::AgentBlue,
        ColorRole::AgentGreen,
        ColorRole::AgentViolet,
        ColorRole::AgentPink,
        ColorRole::AgentCyan,
    ];
    let h = provider
        .bytes()
        .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
    color_var(P[h as usize % P.len()])
}

#[component]
fn Mark(provider: String, #[props(default = 14.0)] size: f32) -> Element {
    let c = provider_color(&provider);
    let initial = provider
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?');
    let fs = size * 0.62;
    rsx! {
        span { class: "oxyn-mark", style: "width:{size}px;height:{size}px;background:{c};font-size:{fs}px", "{initial}" }
    }
}

fn mark_of(catalog: &ModelCatalog, sel: &ModelSelection) -> Option<String> {
    sel.model_id()
        .and_then(|id| catalog.get(id))
        .map(|m| m.provider.clone())
}

/// Highlighted name: matched characters bold.
#[component]
fn Name(text: String, matched: Vec<usize>) -> Element {
    if matched.is_empty() {
        return rsx! { span { class: "nm", "{text}" } };
    }
    rsx! {
        span { class: "nm",
            for (i, c) in text.chars().enumerate() {
                if matched.contains(&i) { b { "{c}" } } else { "{c}" }
            }
        }
    }
}

/// The model pill and its popover.
#[component]
pub fn ModelPicker(
    /// Host catalogue.
    catalog: ModelCatalog,
    /// Selected model.
    selected: ModelSelection,
    /// Waiting for the next turn (`→ Deep 4` badge).
    #[props(default)]
    pending: Option<ModelSelection>,
    /// Popover state.
    picker: Picker,
    /// Favourites (★, cycle order).
    #[props(default)]
    favorites: Vec<String>,
    /// Recently used, newest first.
    #[props(default)]
    recent: Vec<String>,
    /// What the draft needs (incompatibility notes).
    #[props(default)]
    needs: DraftNeeds,
    /// Auto is resolving during a run (three breathing dots).
    #[props(default)]
    routing: bool,
    /// `pill-popover` · `morph-list` · `command` · `glyph` · `compact` · `favorites-segmented`.
    #[props(default = "pill-popover".to_string())]
    variant: String,
    /// Shared clock time (countdowns, routing dots).
    now: ClockMs,
    /// Pill pressed.
    #[props(default)]
    on_toggle: EventHandler<()>,
    /// A key in the popover.
    #[props(default)]
    on_key: EventHandler<PickerKey>,
    /// Search text changed.
    #[props(default)]
    on_query: EventHandler<String>,
    /// A row chosen.
    #[props(default)]
    on_choose: EventHandler<ModelSelection>,
    /// Pending badge × pressed.
    #[props(default)]
    on_cancel_pending: EventHandler<()>,
) -> Element {
    let v = ModelPickerVariant::from_id(&variant).unwrap_or_default();
    let rows = picker.rows(&catalog, &favorites, &recent, &selected);
    let label = selected.label(&catalog);
    let mark = mark_of(&catalog, &selected);
    let open = picker.open;
    let hl = picker.highlighted_row(&rows).cloned();
    let mut si = 0usize;

    if v == ModelPickerVariant::FavoritesSegmented && !open {
        let favs: Vec<&ModelInfo> = favorites
            .iter()
            .filter_map(|f| catalog.get(f))
            .take(4)
            .collect();
        return rsx! {
            div { class: "oxyn-mp seg", role: "radiogroup", "aria-label": "Model",
                for m in favs {
                    {
                        let id = m.id.clone();
                        let on = selected.model_id() == Some(m.id.as_str());
                        rsx! {
                            button { class: if on { "on" } else { "" }, role: "radio", "aria-checked": "{on}",
                                onclick: move |_| on_choose.call(ModelSelection::Model(id.clone())),
                                Mark { provider: m.provider.clone(), size: 12.0 }
                                "{m.short()}"
                            }
                        }
                    }
                }
                button { class: "more", onclick: move |_| on_toggle.call(()), "More…" }
            }
        };
    }

    let pill_class = format!("oxyn-mp pill {}{}", v.id(), if open { " open" } else { "" });
    let dots = (0..3)
        .map(|i| {
            let p = ((now.0 + 1200 - i * 150) % 1200) as f32 / 1200.0;
            0.35 + 0.65 * (0.5 - 0.5 * (p * std::f32::consts::TAU).cos())
        })
        .collect::<Vec<_>>();
    let tooltip = selected
        .model_id()
        .and_then(|id| catalog.get(id))
        .map(|m| {
            format!(
                "{} · {} context · ⌘⇧M",
                m.name,
                format_context(m.context_tokens)
            )
        })
        .unwrap_or_else(|| "Auto picks a model per turn · ⌘⇧M".into());

    rsx! {
        div { class: "oxyn-mp-wrap {v.id()}",
            button { class: "{pill_class}", "aria-haspopup": "listbox", "aria-expanded": "{open}", title: "{tooltip}",
                "aria-label": "Model: {label}",
                onclick: move |_| on_toggle.call(()),
                if v == ModelPickerVariant::Glyph {
                    span { class: "glyph", "model " } span { class: "nm", "{label}" }
                } else {
                    if routing {
                        span { class: "dots",
                            for a in dots { i { style: "opacity:{a:.2}" } }
                        }
                    } else if let Some(p) = mark.clone() {
                        Mark { provider: p }
                    } else {
                        span { class: "auto-ic", Icon { name: "spark", size: 13.0 } }
                    }
                    if v != ModelPickerVariant::Compact { span { class: "nm", "{label}" } }
                    span { class: "chev", Icon { name: "chevron", size: 11.0, stroke: 2.0 } }
                }
            }
            if let Some(p) = pending.clone() {
                span { class: "oxyn-pending", title: "Applies to your next message",
                    "→ {p.label(&catalog)}"
                    button { "aria-label": "Cancel", onclick: move |_| on_cancel_pending.call(()), Icon { name: "x", size: 9.0, stroke: 2.0 } }
                }
            }
            if open {
                if v == ModelPickerVariant::Glyph {
                    GlyphList { catalog: catalog.clone(), rows: rows.clone(), highlighted: hl.clone(), selected: selected.clone(), on_choose, on_key }
                } else {
                    div { class: "oxyn-pop oxyn-mp-pop {v.id()}", role: "dialog", "aria-label": "Choose a model",
                        onkeydown: move |e: KeyboardEvent| {
                            let k = match e.key() {
                                Key::ArrowUp => Some(PickerKey::Up),
                                Key::ArrowDown => Some(PickerKey::Down),
                                Key::Enter => Some(PickerKey::Enter),
                                Key::Escape => Some(PickerKey::Escape),
                                _ => None,
                            };
                            if let Some(k) = k { e.prevent_default(); on_key.call(k); }
                        },
                        div { class: "search",
                            Icon { name: "search", size: 14.0 }
                            input { r#type: "text", placeholder: "Search models", value: "{picker.query}", autofocus: true,
                                "aria-label": "Search models", oninput: move |e| on_query.call(e.value()) }
                            span { class: "kbd", "⌘⇧M" }
                        }
                        div { class: "rows", role: "listbox",
                            for row in rows.iter() {
                                {
                                    let is_hl = row.selectable() && si == picker.highlighted;
                                    if row.selectable() { si += 1; }
                                    let auto_sel = matches!(selected, ModelSelection::Auto { .. });
                                    match row {
                                        PickerRow::Header(h) => rsx! { div { class: "hd", "{h}" } },
                                        PickerRow::NoResults(t) => rsx! { div { class: "empty", "{t}" } },
                                        PickerRow::Auto => rsx! {
                                            div { class: if is_hl { "row hl" } else { "row" }, role: "option",
                                                "aria-selected": "{auto_sel}",
                                                onclick: move |_| on_choose.call(ModelSelection::Auto { resolved: None }),
                                                span { class: "auto-ic", Icon { name: "spark", size: 14.0 } }
                                                span { class: "nm", "Auto" }
                                                span { class: "note", "Picks per turn" }
                                                if auto_sel { span { class: "tick", Icon { name: "check", size: 13.0, stroke: 2.0 } } }
                                            }
                                        },
                                        PickerRow::Model { id, matched } => {
                                            let Some(m) = catalog.get(id) else { return rsx! {} };
                                            let note = status_note(m, &needs, now);
                                            let dim = matches!(m.status, ModelStatus::Unavailable { .. } | ModelStatus::NeedsKey);
                                            let warn = note.is_some() && !dim;
                                            let sel = selected.model_id() == Some(id.as_str());
                                            let star = favorites.contains(id);
                                            let cls = format!("row{}{}{}", if is_hl { " hl" } else { "" }, if dim { " dim" } else { "" }, if warn { " warn" } else { "" });
                                            let choose = ModelSelection::Model(id.clone());
                                            rsx! {
                                                div { class: "{cls}", role: "option", "aria-selected": "{sel}",
                                                    onclick: move |_| on_choose.call(choose.clone()),
                                                    Mark { provider: m.provider.clone() }
                                                    Name { text: m.name.clone(), matched: matched.clone() }
                                                    span { class: "caps",
                                                        if m.caps.vision { span { title: "Vision", Icon { name: "eye", size: 12.0 } } }
                                                        if m.caps.tools { span { title: "Tools", Icon { name: "tool", size: 12.0 } } }
                                                        if !matches!(m.caps.reasoning, oxyn_core::composer::ReasoningSupport::None) { span { title: "Reasoning", Icon { name: "brain", size: 12.0 } } }
                                                    }
                                                    span { class: "ctx", "{format_context(m.context_tokens)}" }
                                                    if let Some(n) = note {
                                                        span { class: "note",
                                                            match m.status {
                                                                ModelStatus::NeedsKey => rsx! { Icon { name: "key", size: 11.0 } },
                                                                ModelStatus::RateLimited { .. } => rsx! { Icon { name: "clock", size: 11.0 } },
                                                                _ => rsx! {},
                                                            }
                                                            "{n}"
                                                        }
                                                    }
                                                    if star { span { class: "star", Icon { name: "star", size: 11.0 } } }
                                                    if sel { span { class: "tick", Icon { name: "check", size: 13.0, stroke: 2.0 } } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(PickerRow::Model { id, .. }) = hl.as_ref() {
                            if let Some(m) = catalog.get(id) {
                                div { class: "tray",
                                    span { "Context " b { "{format_context(m.context_tokens)}" } }
                                    if let Some(s) = m.speed { span { "Speed " b { "{speed(s)}" } } }
                                    if let Some(c) = &m.cost { span { "{c}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn speed(s: Tier3) -> &'static str {
    match s {
        Tier3::Slow => "Slow",
        Tier3::Medium => "Medium",
        Tier3::Fast => "Fast",
    }
}

#[component]
fn GlyphList(
    catalog: ModelCatalog,
    rows: Vec<PickerRow>,
    highlighted: Option<PickerRow>,
    selected: ModelSelection,
    on_choose: EventHandler<ModelSelection>,
    on_key: EventHandler<PickerKey>,
) -> Element {
    let mut n = 0;
    let mut seen = std::collections::HashSet::new();
    let rows: Vec<PickerRow> = rows
        .into_iter()
        .filter(|r| match r {
            PickerRow::Model { id, .. } => seen.insert(id.clone()),
            r => r.selectable(),
        })
        .collect();
    rsx! {
        div { class: "oxyn-mp-glyph", role: "listbox", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                let k = match e.key() {
                    Key::ArrowUp => Some(PickerKey::Up),
                    Key::ArrowDown => Some(PickerKey::Down),
                    Key::Enter => Some(PickerKey::Enter),
                    Key::Escape => Some(PickerKey::Escape),
                    _ => None,
                };
                if let Some(k) = k { e.prevent_default(); on_key.call(k); }
            },
            div { class: "title", "Select model" }
            for row in rows.iter() {
                {
                    n += 1;
                    let hl = highlighted.as_ref() == Some(row);
                    let (name, ctx, choice) = match row {
                        PickerRow::Model { id, .. } => {
                            let m = catalog.get(id);
                            (m.map(|m| m.name.clone()).unwrap_or_default(),
                             m.map(|m| format_context(m.context_tokens)).unwrap_or_default(),
                             ModelSelection::Model(id.clone()))
                        }
                        _ => ("Auto".to_string(), String::new(), ModelSelection::Auto { resolved: None }),
                    };
                    let cur = match (&choice, &selected) {
                        (ModelSelection::Auto { .. }, ModelSelection::Auto { .. }) => true,
                        _ => choice == selected,
                    };
                    rsx! {
                        div { class: if hl { "ln hl" } else { "ln" }, onclick: move |_| on_choose.call(choice.clone()),
                            span { class: "cur", if hl { "›" } else { " " } }
                            "{n}. {name:<18}"
                            span { class: "dim", "{ctx}" }
                            if cur { span { class: "dim", " (current)" } }
                        }
                    }
                }
            }
            div { class: "foot", "↑↓ move · enter select · esc close" }
        }
    }
}
