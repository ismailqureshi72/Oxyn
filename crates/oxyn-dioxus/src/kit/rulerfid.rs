//! Thinking Selector family, variant "Ruler card" ([`EffortRuler`]): the level's name and line over a
//! dense tick ruler; the ticks behind the capsule knob tint toward it and a reset button returns to
//! the default. The model is `oxyn_core::kit::thinkfid`. Styles: `rulerfid.css` (classes
//! `oxer-*`, plus the `--oxth-*` palette every Thinking Selector variant shares).

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::kit::thinkfid::{
    EffortLevel, effort_said, effort_scale, effort_step, level_frac, snap_level, tick_tint,
    track_frac,
};

/// A horizontal track the pointer can press and drag along: remembers where it sits on screen
/// (measured on press) so moves map to a fraction without waiting.
#[derive(Clone, Copy)]
pub(crate) struct Track {
    el: Signal<Option<Rc<MountedData>>>,
    left: Signal<f64>,
    width: Signal<f64>,
    /// True while the pointer is held down on it.
    pub(crate) drag: Signal<bool>,
}

impl Track {
    pub(crate) fn new() -> Self {
        Self {
            el: Signal::new(None),
            left: Signal::new(0.0),
            width: Signal::new(0.0),
            drag: Signal::new(false),
        }
    }
    /// Remember the element (`onmounted`).
    pub(crate) fn mount(mut self, d: Rc<MountedData>) {
        self.el.set(Some(d));
    }
    /// A press at client `x`: measures the track, then calls `at` with the fraction.
    pub(crate) fn press(mut self, x: f64, at: Callback<f32>) {
        self.drag.set(true);
        let el = self.el.peek().clone();
        let (mut left, mut width) = (self.left, self.width);
        if let Some(el) = el {
            spawn(async move {
                if let Ok(r) = el.get_client_rect().await {
                    left.set(r.origin.x);
                    width.set(r.size.width);
                    at.call(track_frac(x, r.origin.x, r.size.width));
                }
            });
        }
    }
    /// A move at client `x` while held: the fraction from the last measure.
    pub(crate) fn moved(self, x: f64) -> Option<f32> {
        (*self.drag.peek() && *self.width.peek() > 0.0)
            .then(|| track_frac(x, *self.left.peek(), *self.width.peek()))
    }
    /// The pointer was let go or left.
    pub(crate) fn release(mut self) {
        self.drag.set(false);
    }
}

/// The keyboard step a slider key asks for (`None` for other keys): arrows ±1, Page keys ±1,
/// Home / End to the ends.
pub(crate) fn slider_key(k: &Key) -> Option<i64> {
    match k {
        Key::ArrowLeft | Key::ArrowDown | Key::PageDown => Some(-1),
        Key::ArrowRight | Key::ArrowUp | Key::PageUp => Some(1),
        Key::Home => Some(i64::MIN),
        Key::End => Some(i64::MAX),
        _ => None,
    }
}

pub(crate) fn reset_glyph() -> Element {
    rsx! {
        svg { width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M9 14 4 9l5-5" } path { d: "M4 9h10.5a5.5 5.5 0 0 1 0 11H11" } }
    }
}

/// Thinking Selector – Ruler card: a card with the level's name (in the accent at the top level)
/// and its line, over a ruler of dense ticks. Ticks behind the capsule knob tint toward it; the
/// ruler is a slider (drag, click, arrows, Home/End) that snaps to the scale's levels, and the
/// reset button goes back to `reset`.
#[component]
pub fn EffortRuler(
    /// The scale (Low … Max when `None`).
    #[props(default)]
    levels: Option<Vec<EffortLevel>>,
    /// Controlled level index.
    #[props(default)]
    value: Option<usize>,
    /// Starting level when uncontrolled (the top level when `None`).
    #[props(default)]
    default_value: Option<usize>,
    /// Where reset returns (the middle level when `None`).
    #[props(default)]
    reset: Option<usize>,
    /// How many ticks the ruler draws.
    #[props(default = 44)]
    ticks: usize,
    /// The control's accessible name.
    #[props(default = "Thinking effort".to_string())]
    label: String,
    /// The level changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let levels = levels.unwrap_or_else(effort_scale);
    let n = levels.len().max(1);
    let mut own = use_signal(move || default_value.unwrap_or(n - 1).min(n - 1));
    let mut said = use_signal(String::new);
    let track = use_hook(Track::new);
    let cur = value.unwrap_or(own()).min(n - 1);
    let home = reset.unwrap_or(n / 2).min(n - 1);
    let lv = levels[cur].clone();
    let lv_all = levels.clone();
    let pick = use_callback(move |i: usize| {
        let i = i.min(n - 1);
        if i != *own.peek() || value.is_some() {
            own.set(i);
            said.set(effort_said(&lv_all[i]));
            on_change.call(i);
        }
    });
    let at = use_callback(move |f: f32| pick.call(snap_level(f, n)));
    let knob = level_frac(cur, n);
    let top = cur == n - 1;
    let ticks = ticks.max(2);
    let pct = knob * 100.0;
    let reset_label = format!("Reset to {}", levels[home].label);
    let dragging = (track.drag)();

    rsx! {
        section { class: "oxer", "data-top": "{top}", "aria-label": "{label}",
            button { class: "oxer-reset", r#type: "button", "aria-label": "{reset_label}", title: "{reset_label}",
                disabled: cur == home, onclick: move |_| pick.call(home), {reset_glyph()} }
            div { class: "oxer-head",
                p { class: "oxer-lv", "{lv.label}" }
                p { class: "oxer-sub", "{lv.detail}" }
            }
            div { class: "oxer-well",
                div { class: "oxer-track", role: "slider", tabindex: "0", "aria-label": "{label}",
                    "aria-valuemin": "0", "aria-valuemax": "{n - 1}", "aria-valuenow": "{cur}", "aria-valuetext": "{lv.label}",
                    "data-drag": "{dragging}",
                    onmounted: move |e| track.mount(e.data()),
                    onpointerdown: move |e: PointerEvent| track.press(e.client_coordinates().x, at),
                    onpointermove: move |e: PointerEvent| if let Some(f) = track.moved(e.client_coordinates().x) { at.call(f) },
                    onpointerup: move |_| track.release(),
                    onpointerleave: move |_| track.release(),
                    onkeydown: move |e: KeyboardEvent| if let Some(d) = slider_key(&e.key()) {
                        e.prevent_default();
                        pick.call(effort_step(cur, d, n));
                    },
                    for t in 0..ticks {
                        {
                            let w = tick_tint(t, ticks, knob, 0.55);
                            let x = t as f32 / (ticks - 1) as f32 * 100.0;
                            rsx! { i { key: "{t}", "data-k": "{t}", class: "oxer-tick", style: "left:{x}%;--w:{w}" } }
                        }
                    }
                    span { class: "oxer-knob", style: "left:{pct}%", "aria-hidden": "true" }
                }
            }
            div { class: "oxer-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
