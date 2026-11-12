//! `SendButton` (`docs/specs/composer-send-control.md`). Controlled: draws a [`SendView`] and
//! reports the pressed segment's action.

use dioxus::prelude::*;
use oxyn_core::composer::{
    SendAction, SendVariant, SendView,
    send::{SendGlyph, Tone},
};

use super::icons::Icon;

fn icon(g: SendGlyph) -> (&'static str, f32) {
    match g {
        SendGlyph::Up => ("up", 16.0),
        SendGlyph::Stop => ("stop", 11.0),
        SendGlyph::Pause => ("pause", 13.0),
        SendGlyph::Resume => ("play", 13.0),
        SendGlyph::Check => ("check", 16.0),
        SendGlyph::Retry => ("retry", 15.0),
        SendGlyph::Menu => ("chevron", 12.0),
        SendGlyph::Steer => ("steer", 14.0),
    }
}

fn tone(t: Tone) -> &'static str {
    match t {
        Tone::Primary => "primary",
        Tone::Strong => "strong",
        Tone::Muted => "muted",
        Tone::Soft => "soft",
        Tone::Danger => "danger",
    }
}

/// The send control.
#[component]
pub fn SendButton(
    /// Frame from `SendControl::view`.
    view: SendView,
    /// Variant id (`classic`, `queue-pill`, `split`, …).
    #[props(default = "classic".to_string())]
    variant: String,
    /// A segment was pressed.
    #[props(default)]
    on_action: EventHandler<SendAction>,
    /// Pointer down / up on stop (hold-to-stop).
    #[props(default)]
    on_hold: EventHandler<bool>,
) -> Element {
    let v = SendVariant::from_id(&variant).unwrap_or_default();
    let text = matches!(v, SendVariant::Minimal | SendVariant::Glyph);
    let n = view.segments.len();
    let joined = v == SendVariant::Split && n > 1;
    let ring = view.ring;
    let c = std::f32::consts::TAU * 15.0;
    let style = format!(
        "transform:translateX({:.1}px) scale({:.3})",
        view.shake_px, view.scale
    );
    let root = format!(
        "oxyn-send {}{}",
        v.id(),
        if joined { " joined" } else { "" }
    );
    rsx! {
        div { class: "{root}", style: "{style}",
            if let Some(h) = view.confirm_hint { span { class: "hint", role: "status", "{h}" } }
            if let Some(n) = view.note.clone() { span { class: "note", role: "alert", "{n}" } }
            for (i, s) in view.segments.iter().cloned().enumerate() {
                {
                    let a = s.action;
                    let cls = format!(
                        "seg {}{}{}{}",
                        tone(s.tone),
                        if s.label.is_some() { " labelled" } else { "" },
                        if text { " text" } else { "" },
                        if joined && i == 0 { " first" } else if joined { " last" } else { "" },
                    );
                    let stop = a == SendAction::Stop;
                    let kind = if a == SendAction::Send { "submit" } else { "button" };
                    rsx! {
                        button { r#type: "{kind}", class: "{cls}", "aria-label": "{s.aria}", title: "{s.aria}",
                            "aria-disabled": "{!s.enabled}",
                            onclick: move |e| { e.prevent_default(); if a != SendAction::None { on_action.call(a) } },
                            onpointerdown: move |_| if stop { on_hold.call(true) },
                            onpointerup: move |_| if stop { on_hold.call(false) },
                            if let (Some(r), true) = (ring, stop) {
                                svg { class: "ring", view_box: "0 0 34 34", width: "34", height: "34",
                                    circle { cx: "17", cy: "17", r: "15", class: "track" }
                                    circle { cx: "17", cy: "17", r: "15", class: "arc", "stroke-dasharray": "{c * r:.2} {c:.2}" }
                                }
                            }
                            if let Some(g) = s.glyph {
                                {
                                    let (name, size) = icon(g);
                                    rsx! { span { class: "gl", style: "opacity:{0.35 + 0.65 * view.morph:.2}", Icon { name, size, stroke: 2.0 } } }
                                }
                            }
                            if let Some(l) = s.label.clone() { span { class: "lb", "{l}" } }
                        }
                    }
                }
            }
        }
    }
}
