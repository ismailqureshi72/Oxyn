//! Thinking Selector family, variant "Context + Effort menu" ([`ContextEffortMenu`]): one trigger
//! that opens a dropdown with two radio groups – the context window (with a "Higher usage" tag on
//! the big one) and the effort (default … max). Models: `oxyn_core::parity::{thinkfid, effmenufid,
//! reasonfid}`. Styles: `ctxefffid.css` (classes `oxec-*`).

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::parity::{
    effmenufid::{ContextOption, context_options, menu_move},
    reasonfid::context_label,
    thinkfid::{EffortLevel, effort_scale_api},
};

use super::efflistfid::{check_glyph, chevron, focus, menu_key, use_uid};

/// Thinking Selector – Context + Effort menu: a model chip whose dropdown sets the context window
/// and the effort in one place. Both groups share one keyboard list; a pick keeps the menu open so
/// both can be set, Escape closes it.
#[component]
pub fn ContextEffortMenu(
    /// The model name on the trigger.
    #[props(default = "Atlas 6 Swift".to_string())]
    model: String,
    /// Context windows (512K and 1M when `None`).
    #[props(default)]
    windows: Option<Vec<ContextOption>>,
    /// Effort scale (default, low … max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Starting window index.
    #[props(default)]
    default_window: usize,
    /// Starting effort index.
    #[props(default)]
    default_effort: usize,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// The window changed.
    #[props(default)]
    on_window_change: EventHandler<usize>,
    /// The effort changed.
    #[props(default)]
    on_effort_change: EventHandler<usize>,
) -> Element {
    let windows = windows.unwrap_or_else(context_options);
    let levels = levels.unwrap_or_else(effort_scale_api);
    let nw = windows.len();
    let total = nw + levels.len();
    let mut win = use_signal(move || default_window.min(nw.saturating_sub(1)));
    let mut eff = use_signal(move || default_effort);
    let mut open = use_signal(move || default_open);
    // Only a menu the user opened gets the click-outside catcher; a preview that starts open
    // never covers the page.
    let mut armed = use_signal(|| false);
    let mut hi = use_signal(|| 0usize);
    let mut want = use_signal(|| false);
    let mut said = use_signal(String::new);
    let mut chip = use_signal(|| None::<Rc<MountedData>>);
    let uid = use_uid("oxec");
    let wl: Vec<String> = windows.iter().map(|w| context_label(w.tokens)).collect();
    let el: Vec<String> = levels.iter().map(|l| l.label.clone()).collect();
    let (wl2, el2) = (wl.clone(), el.clone());
    let pick = use_callback(move |row: usize| {
        if row < nw {
            win.set(row);
            said.set(format!("Context window: {}", wl2[row]));
            on_window_change.call(row);
        } else {
            eff.set(row - nw);
            said.set(format!("Effort: {}", el2[row - nw]));
            on_effort_change.call(row - nw);
        }
    });
    let is_open = open();
    let active = format!("{uid}-{}", hi());
    let cur_w = win().min(nw.saturating_sub(1));
    let cur_e = eff().min(levels.len().saturating_sub(1));
    let summary = format!(
        "{} · {}",
        wl.get(cur_w).cloned().unwrap_or_default(),
        el.get(cur_e).cloned().unwrap_or_default()
    );

    let catch = is_open && armed();

    rsx! {
        div { class: "oxec", "data-open": "{is_open}", "data-armed": "{catch}",
            if catch { div { class: "oxth-catch", "aria-hidden": "true", onpointerdown: move |_| open.set(false) } }
            button { class: "oxec-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{is_open}",
                "aria-label": "{model}, {summary}",
                onmounted: move |e| chip.set(Some(e.data())),
                onclick: move |_| { if !open() { hi.set(cur_w); want.set(true); armed.set(true); } open.set(!open()); },
                onkeydown: move |e: KeyboardEvent| if matches!(e.key(), Key::ArrowDown | Key::ArrowUp) && !open() {
                    e.prevent_default(); hi.set(cur_w); want.set(true); armed.set(true); open.set(true);
                },
                span { class: "oxec-model", crate::brand::MarkFor { name: model.clone(), size: 12.0 } "{model}" }
                span { class: "oxec-sum", "{summary}" }
                {chevron(false)}
            }
            if is_open {
                div { class: "oxec-pop", role: "menu", tabindex: "-1", "aria-label": "{model} options",
                    "aria-activedescendant": "{active}",
                    onmounted: move |e| { if *want.peek() { want.set(false); focus(Some(e.data())); } },
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        if let Some(d) = menu_key(&k) { e.prevent_default(); hi.set(menu_move(hi(), d, total)); }
                        else if k == Key::Enter || k == Key::Character(" ".into()) { e.prevent_default(); pick.call(hi()); }
                        else if k == Key::Escape || k == Key::Tab {
                            if k == Key::Escape { e.prevent_default(); }
                            open.set(false);
                            focus(chip.peek().clone());
                        }
                    },
                    div { class: "oxec-grp", role: "group", "aria-labelledby": "{uid}-gw",
                        p { class: "oxec-h", id: "{uid}-gw", "Context window" }
                        for (i, w) in windows.iter().enumerate() {
                            div { key: "w{i}", "data-k": "w{i}", id: "{uid}-{i}", class: "oxec-item", role: "menuitemradio",
                                "aria-checked": "{i == cur_w}", "data-hi": "{hi() == i}",
                                onpointerenter: move |_| hi.set(i), onclick: move |_| pick.call(i),
                                span { class: "oxec-l", "{wl[i]}" if !w.tag.is_empty() { span { class: "oxec-tag", "{w.tag}" } } }
                                if i == cur_w { span { class: "oxec-ck", {check_glyph()} } }
                            }
                        }
                    }
                    div { class: "oxec-sep", role: "separator" }
                    div { class: "oxec-grp", role: "group", "aria-labelledby": "{uid}-ge",
                        p { class: "oxec-h", id: "{uid}-ge", "Effort" }
                        for (j, l) in el.iter().enumerate() {
                            {
                                let row = nw + j;
                                rsx! {
                                    div { key: "e{j}", "data-k": "e{j}", id: "{uid}-{row}", class: "oxec-item", role: "menuitemradio",
                                        "aria-checked": "{j == cur_e}", "data-hi": "{hi() == row}", "data-max": "{j + 1 == el.len()}",
                                        onpointerenter: move |_| hi.set(row), onclick: move |_| pick.call(row),
                                        span { class: "oxec-l", "{l}" }
                                        if j == cur_e { span { class: "oxec-ck", {check_glyph()} } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxec-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
