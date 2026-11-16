//! Thinking Selector family, variant "Model effort menu" ([`ModelEffortMenu`]): a list of models,
//! each with its effort as a grey suffix, and a cascading submenu that sets the effort of the
//! pointed model. Models: `oxyn_core::parity::effmenufid`. Styles: `modeleffid.css` (`oxem-*`).

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::parity::effmenufid::{EffortModel, effort_models, menu_move, model_effort_label};

use super::efflistfid::{check_glyph, chevron, focus, menu_key, use_uid};

fn caret_right() -> Element {
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "m9 6 6 6-6 6" } }
    }
}

/// Thinking Selector – Model effort menu: models with their effort as a suffix; → or hover opens a
/// submenu of that model's levels. Picking a level also picks the model. ← or Escape steps back.
#[component]
pub fn ModelEffortMenu(
    /// The models (seven fictional ones when `None`).
    #[props(default)]
    models: Option<Vec<EffortModel>>,
    /// Starting chosen model index.
    #[props(default = 5)]
    default_model: usize,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// Starts with this row's submenu open (for previews).
    #[props(default)]
    default_sub: Option<usize>,
    /// A model and level were picked: (model index, level index).
    #[props(default)]
    on_change: EventHandler<(usize, usize)>,
) -> Element {
    let start = models.unwrap_or_else(effort_models);
    let n = start.len().max(1);
    let mut list = use_signal(move || start);
    let mut chosen = use_signal(move || default_model.min(n - 1));
    let mut open = use_signal(move || default_open);
    // Only a menu the user opened gets the click-outside catcher; a preview that starts open
    // never covers the page.
    let mut armed = use_signal(|| false);
    let mut hi = use_signal(move || default_sub.unwrap_or(default_model).min(n - 1));
    let mut sub = use_signal(move || default_sub);
    let mut sub_hi = use_signal(|| 0usize);
    let mut want = use_signal(|| false);
    let mut said = use_signal(String::new);
    let mut chip = use_signal(|| None::<Rc<MountedData>>);
    let uid = use_uid("oxem");
    let pick = use_callback(move |(m, l): (usize, usize)| {
        list.write()[m].level = l;
        chosen.set(m);
        said.set(format!("Model: {}", model_effort_label(&list.peek()[m])));
        on_change.call((m, l));
        sub.set(None);
        open.set(false);
        focus(chip.peek().clone());
    });
    let mut open_sub = move |m: usize| {
        sub.set(Some(m));
        sub_hi.set(list.peek()[m].level);
    };
    let models = list();
    let cur = chosen().min(n - 1);
    let is_open = open();
    let sub_now = sub();
    let active = match sub_now {
        Some(m) => format!("{uid}-s{m}-{}", sub_hi()),
        None => format!("{uid}-{}", hi()),
    };
    let trigger = models[cur].name.clone();
    let trigger_lv = models[cur]
        .levels
        .get(models[cur].level)
        .map(|l| l.label.clone())
        .unwrap_or_default();

    let catch = is_open && armed();

    rsx! {
        div { class: "oxem", "data-open": "{is_open}", "data-armed": "{catch}",
            if catch { div { class: "oxth-catch", "aria-hidden": "true", onpointerdown: move |_| open.set(false) } }
            if is_open {
                div { class: "oxem-pop", role: "menu", tabindex: "-1", "aria-label": "Models",
                    "aria-activedescendant": "{active}",
                    onmounted: move |e| { if *want.peek() { want.set(false); focus(Some(e.data())); } },
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        match sub() {
                            None => {
                                if let Some(d) = menu_key(&k) { e.prevent_default(); hi.set(menu_move(hi(), d, n)); }
                                else if matches!(k, Key::ArrowRight | Key::Enter) || k == Key::Character(" ".into()) { e.prevent_default(); open_sub(hi()); }
                                else if k == Key::Escape || k == Key::Tab {
                                    if k == Key::Escape { e.prevent_default(); }
                                    open.set(false); focus(chip.peek().clone());
                                }
                            }
                            Some(m) => {
                                let nl = list.peek()[m].levels.len();
                                if let Some(d) = menu_key(&k) { e.prevent_default(); sub_hi.set(menu_move(sub_hi(), d, nl)); }
                                else if k == Key::Enter || k == Key::Character(" ".into()) { e.prevent_default(); pick.call((m, sub_hi())); }
                                else if matches!(k, Key::ArrowLeft | Key::Escape) { e.prevent_default(); sub.set(None); }
                            }
                        }
                    },
                    div { class: "oxem-mark", "aria-hidden": "true",
                        svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "currentColor",
                            path { d: "M13 2 4 14h7l-1 8 9-12h-7z" } } }
                    for (i, m) in models.iter().enumerate() {
                        {
                            let lv = m.levels.get(m.level).map(|l| l.label.clone()).unwrap_or_default();
                            let top = m.level + 1 == m.levels.len();
                            let this_open = sub_now == Some(i);
                            let levels = m.levels.clone();
                            let cur_lv = m.level;
                            rsx! {
                                div { key: "{i}", "data-k": "{i}", class: "oxem-rowwrap",
                                    div { id: "{uid}-{i}", class: "oxem-row", role: "menuitem", "aria-haspopup": "menu",
                                        "aria-expanded": "{this_open}", "aria-label": "{m.name}, {lv}",
                                        "data-hi": "{hi() == i && sub_now.is_none()}", "data-on": "{i == cur}", "data-sub": "{this_open}",
                                        onpointerenter: move |_| { hi.set(i); open_sub(i); },
                                        onclick: move |_| open_sub(i),
                                        span { class: "oxem-name", "{m.name}" span { class: "oxem-lv", "data-top": "{top}", " {lv}" } }
                                        if i == cur { span { class: "oxem-ck", {check_glyph()} } } else { span { class: "oxem-go", {caret_right()} } }
                                    }
                                    if this_open {
                                        div { class: "oxem-sub", role: "menu", "aria-label": "{m.name} effort",
                                            for (j, l) in levels.iter().enumerate() {
                                                div { key: "{j}", "data-k": "{j}", id: "{uid}-s{i}-{j}", class: "oxem-subrow", role: "menuitemradio",
                                                    "aria-checked": "{j == cur_lv}", "data-hi": "{sub_hi() == j}",
                                                    onpointerenter: move |_| sub_hi.set(j),
                                                    onclick: move |e| { e.stop_propagation(); pick.call((i, j)); },
                                                    span { "{l.label}" }
                                                    if j == cur_lv { span { class: "oxem-ck", {check_glyph()} } }
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
            button { class: "oxem-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{is_open}",
                "aria-label": "Model: {trigger}, {trigger_lv}",
                onmounted: move |e| chip.set(Some(e.data())),
                onclick: move |_| { if !open() { hi.set(cur); sub.set(None); want.set(true); armed.set(true); } open.set(!open()); },
                span { "{trigger}" } {chevron(false)} }
            div { class: "oxem-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
