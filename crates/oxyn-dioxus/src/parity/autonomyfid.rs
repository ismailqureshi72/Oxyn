//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32): the exact **Autonomy Dial**
//! ([`AutonomyDial`]). Behaviour and look only; the model is `oxyn_core::parity::autonomyfid`.
//! Styles: `autonomyfid.css` (classes `oxad-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::parity::autonomyfid::{
    AUTONOMY_LEVELS, AutonomyAction, AutonomyRisk, asks_first, autonomy_demo, autonomy_demo_pinned,
    autonomy_meaning, autonomy_split, autonomy_status, autonomy_step, autonomy_toggle,
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn risk_glyph(r: AutonomyRisk) -> Element {
    match r {
        AutonomyRisk::Safe => rsx! {},
        AutonomyRisk::Risky => rsx! {
            svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                path { d: "M10.3 4.1 2.6 17.5A2 2 0 0 0 4.3 20.5h15.4a2 2 0 0 0 1.7-3L13.7 4.1a2 2 0 0 0-3.4 0z" }
                path { d: "M12 9.5v4M12 17h.01" } }
        },
        AutonomyRisk::Destructive => rsx! {
            svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "9" } path { d: "M12 7.5v5M12 16.2h.01" } }
        },
    }
}

fn lock_glyph(closed: bool) -> Element {
    let shackle = if closed {
        "M8 10.5V8a4 4 0 0 1 8 0v2.5"
    } else {
        "M8 10.5V8a4 4 0 0 1 7.6-1.7"
    };
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.6",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            rect { x: "5", y: "10.5", width: "14", height: "10", rx: "3" } path { d: "{shackle}" } path { d: "M12 14.5v2" } }
    }
}

/// Autonomy Dial (affectUI): how much an agent may do on its own. Three levels in a segmented
/// radio group; the agent's actions sit under "Runs on its own" or "Asks you first" and glide to
/// the other group, a beat after each other, when the level or a lock changes. Risky and
/// destructive actions carry a tag; a lock makes an action always ask.
#[component]
pub fn AutonomyDial(
    /// The agent's actions (the demo's when `None`).
    #[props(default)]
    actions: Option<Vec<AutonomyAction>>,
    /// Controlled level (0 Always ask, 1 If risky, 2 Full auto).
    #[props(default)]
    level: Option<u8>,
    /// Starting level when uncontrolled.
    #[props(default = 1)]
    default_level: u8,
    /// Controlled locks (ids that always ask).
    #[props(default)]
    pinned: Option<Vec<String>>,
    /// Starting locks when uncontrolled (the demo's when `None` and `actions` is `None`).
    #[props(default)]
    default_pinned: Option<Vec<String>>,
    /// Heading, e.g. the agent's name.
    #[props(default = "Agent permissions".to_string())]
    title: String,
    /// Line under the heading.
    #[props(default = "What it may do without asking you".to_string())]
    subtitle: String,
    /// An action shown pointed at (its row tinted, its lock revealed).
    #[props(default)]
    point: String,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Another level was picked.
    #[props(default)]
    on_level_change: EventHandler<u8>,
    /// A lock was set or removed (the whole set).
    #[props(default)]
    on_pinned_change: EventHandler<Vec<String>>,
) -> Element {
    let demo = actions.is_none();
    let mut own_level = use_signal(move || default_level.min(2));
    let mut own_pins = use_signal(move || {
        default_pinned.unwrap_or_else(|| if demo { autonomy_demo_pinned() } else { vec![] })
    });
    let mut stops = use_signal(|| [None, None, None] as [Option<Rc<MountedData>>; 3]);
    let mut said = use_signal(String::new);
    let mut moved = use_signal(|| 0u32);
    // The level and locks before the last change: a row whose group changed glides in.
    let mut prev = use_signal(|| None::<(u8, Vec<String>)>);
    let list = actions.unwrap_or_else(autonomy_demo);
    let lv = level.unwrap_or(own_level()).min(2);
    let pins = pinned.clone().unwrap_or(own_pins());
    let (own, ask) = autonomy_split(&list, lv, &pins);
    let meaning = autonomy_meaning(lv);
    let tick = moved();
    let list_for_level = list.clone();
    let pins_for_level = pins.clone();
    let pick = use_callback(move |n: u8| {
        if n == lv {
            return;
        }
        prev.set(Some((lv, pins_for_level.clone())));
        own_level.set(n);
        moved += 1;
        said.set(autonomy_status(&list_for_level, n, &pins_for_level));
        on_level_change.call(n);
    });
    let list_for_lock = list.clone();
    let toggle = use_callback(move |id: String| {
        let next = autonomy_toggle(&pins, &id);
        prev.set(Some((lv, pins.clone())));
        own_pins.set(next.clone());
        moved += 1;
        said.set(autonomy_status(&list_for_lock, lv, &next));
        on_pinned_change.call(next);
    });
    let pins_now = pinned.unwrap_or(own_pins());

    rsx! {
        section { class: "oxad", "data-style": style_attr(&style), "data-level": "{lv}", "aria-label": "{title}",
            div { class: "oxad-top",
                div { class: "oxad-head",
                    span { class: "oxad-ic",
                        svg { width: "14", height: "14", view_box: "0 0 24 24", "aria-hidden": "true",
                            path { fill: "var(--oxad-blue)", d: "M10 2.5c.5 4.4 2.6 6.5 7 7-4.4.5-6.5 2.6-7 7-.5-4.4-2.6-6.5-7-7 4.4-.5 6.5-2.6 7-7z" }
                            path { fill: "var(--oxad-blue2)", d: "M18.5 14c.25 2.1 1.2 3.05 3.3 3.3-2.1.25-3.05 1.2-3.3 3.3-.25-2.1-1.2-3.05-3.3-3.3 2.1-.25 3.05-1.2 3.3-3.3z" } }
                    }
                    div { class: "oxad-ht",
                        h3 { class: "oxad-ttl", "{title}" }
                        p { class: "oxad-sub", "{subtitle}" }
                    }
                }
                div { class: "oxad-seg", role: "radiogroup", "aria-label": "Autonomy", "aria-description": "{meaning}",
                    for (i, name) in AUTONOMY_LEVELS.iter().enumerate() {
                        button { key: "{i}", "data-k": "{i}", class: "oxad-stop", r#type: "button", role: "radio",
                            "aria-checked": "{i as u8 == lv}", tabindex: if i as u8 == lv { "0" } else { "-1" },
                            onmounted: move |e| stops.write()[i] = Some(e.data()),
                            onclick: move |_| pick.call(i as u8),
                            onkeydown: move |e: KeyboardEvent| {
                                let d = match e.key() {
                                    Key::ArrowLeft | Key::ArrowUp => -1,
                                    Key::ArrowRight | Key::ArrowDown => 1,
                                    _ => 0,
                                };
                                if d != 0 {
                                    e.prevent_default();
                                    let n = autonomy_step(lv, d);
                                    pick.call(n);
                                    focus(stops.read()[n as usize].clone());
                                }
                            },
                            "{name}" }
                    }
                }
                p { class: "oxad-mean", "{meaning}" }
            }
            div { class: "oxad-body",
                for (gi, (name, rows)) in [("Runs on its own", own.clone()), ("Asks you first", ask.clone())].into_iter().enumerate() {
                    section { key: "{gi}", "data-k": "{gi}", class: "oxad-grp", "aria-label": "{name}",
                        div { class: "oxad-gh",
                            span { "{name}" }
                            span { key: "{rows.len()}", "data-k": "{rows.len()}", class: "oxad-n", "data-roll": "{tick > 0}", "{rows.len()}" }
                        }
                        if rows.is_empty() {
                            div { class: "oxad-empty", if gi == 0 { "Every action waits for you" } else { "Nothing waits for you" } }
                        } else {
                            ul { class: "oxad-list",
                                for (ri, a) in rows.iter().cloned().enumerate() {
                                    {
                                        let locked = pins_now.contains(&a.id);
                                        let id = a.id.clone();
                                        let key = format!("{gi}-{}", a.id);
                                        let glide = prev().is_some_and(|(pl, pp)| asks_first(&a, pl, &pp) != (gi == 1));
                                        let lit = point == a.id;
                                        let tag = a.risk.tag();
                                        let label = if locked {
                                            format!("{}: always asks – unlock", a.label)
                                        } else {
                                            format!("{}: always ask", a.label)
                                        };
                                        rsx! {
                                            li { key: "{key}", "data-k": "{key}", class: "oxad-row", "data-lit": "{lit}", "data-locked": "{locked}", "data-glide": "{glide}",
                                                style: "--d:{ri * 45}ms",
                                                span { class: "oxad-logo", BrandIcon { name: a.brand.clone(), size: 16.0, coloured: true, fallback: true } }
                                                span { class: "oxad-lt",
                                                    span { class: "oxad-l1",
                                                        span { class: "oxad-label", "{a.label}" }
                                                        if !tag.is_empty() {
                                                            span { class: "oxad-tag", "data-risk": "{a.risk.id()}", {risk_glyph(a.risk)} "{tag}" }
                                                        }
                                                    }
                                                    if !a.detail.is_empty() { span { class: "oxad-det", "{a.detail}" } }
                                                }
                                                button { class: "oxad-lock", r#type: "button", "aria-pressed": "{locked}", "aria-label": "{label}", title: "{label}",
                                                    onclick: move |_| toggle.call(id.clone()),
                                                    {lock_glyph(locked)} }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxad-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
