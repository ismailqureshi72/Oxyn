//! Number entry and choice (plan #28, chunk 4b): `NumberField` (Arc Number field), `ScrubInput`
//! (Arc Scrub input), `RubberSlider` (Arc Elastic slider) and `UnfoldSelect` (Arc Morph select).
//! Behaviour-only rebuilds; models in `oxyn_core::kit::fields`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::fields::{
        Choice, Edge, Gear, NumberField as Nf, OpenAt, Scrub, UnfoldSelect as Ms, detent,
        format_number, held_steps, next_unit, plural, stretch, tag_tilt,
    },
};

use crate::composer::icons::Icon;

fn slug(prefix: &str, label: &str) -> String {
    let s: String = label
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_lowercase();
    format!("oxf-{prefix}-{s}")
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// Digits that roll on wheels: keyed by place from the end, so only the places that changed
/// remount and roll (up or down with the parent's `data-dir`).
#[component]
fn Wheels(text: String) -> Element {
    let n = text.chars().count();
    rsx! {
        span { class: "oxf-wheels", "aria-hidden": "true",
            for (i, c) in text.chars().enumerate() {
                span { key: "{n - i}-{c}", class: if c.is_ascii_digit() { "oxf-dg" } else { "oxf-sep" }, "{c}" }
            }
        }
    }
}

/// Number field (Arc behaviour): a bounded number with − and + buttons. Holding a button
/// repeats after 400 ms and speeds up from 150 ms to 40 ms a step; at a limit the value strains
/// toward the press (further for pushes in a row), the refused button shakes and a note such as
/// "Max 10 seats" rises beside the label for about 1.5 s. A typed value past a limit tints the
/// shell and springs back to the limit on Enter or blur. Digits roll in the direction of change.
/// With `scrub` the label drags sideways to change the value.
#[component]
pub fn NumberField(
    /// Label.
    label: String,
    /// Starting value.
    #[props(default)]
    value: f64,
    /// Lower bound.
    #[props(default)]
    min: f64,
    /// Upper bound.
    #[props(default = 1e15)]
    max: f64,
    /// Increment (its precision sets the decimals).
    #[props(default = 1.0)]
    step: f64,
    /// PageUp/PageDown and Shift+arrow distance.
    #[props(default)]
    large_step: Option<f64>,
    /// Text before the number ("$").
    #[props(default)]
    prefix: String,
    /// Text after the number (" seats").
    #[props(default)]
    suffix: String,
    /// Singular suffix (" seat") used when the value is 1.
    #[props(default)]
    suffix_one: Option<String>,
    /// Helper copy under the field.
    #[props(default)]
    description: Option<String>,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Drag the label sideways to change the value.
    #[props(default)]
    scrub: bool,
    /// Show the limit note.
    #[props(default = true)]
    limit_hint: bool,
    /// Thousands separators.
    #[props(default = true)]
    grouping: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock (hold-to-repeat and the note's timing).
    now: ClockMs,
    /// Every committed change.
    #[props(default)]
    on_change: EventHandler<f64>,
) -> Element {
    let mut f = use_signal(move || {
        let mut n = Nf::new(value, min, max, step);
        n.large_step = large_step;
        n
    });
    let mut held = use_signal(|| None::<(i32, ClockMs)>);
    let mut dir = use_signal(|| 0i32);
    let mut editing = use_signal(|| false);
    let mut scrub_from = use_signal(|| None::<(f64, f64)>);
    let id = slug("nf", &label);
    // The view settles a copy on the clock while a button is held.
    let mut view = f();
    if let Some((d, t)) = held() {
        for _ in 0..held_steps(now.since(t).as_millis() as u64) {
            let _ = view.step_by(d, false, now);
        }
    }
    let v = view.value;
    let suf = match &suffix_one {
        Some(one) => plural(v, one, &suffix),
        None => suffix.clone(),
    };
    let shown = format_number(v, view.places(), grouping);
    let note = if limit_hint {
        view.limit_note(now, &prefix, &suf).or_else(|| {
            view.draft_over().map(|e| match e {
                Edge::Max => format!(
                    "Max {prefix}{}{suf}",
                    format_number(max, view.places(), grouping)
                ),
                Edge::Min => format!(
                    "Min {prefix}{}{suf}",
                    format_number(min, view.places(), grouping)
                ),
            })
        })
    } else {
        None
    };
    let strain = view.strain_px(now)
        * match view.refused {
            Some((Edge::Max, _)) => 1.0,
            _ => -1.0,
        };
    let refused = view
        .refused
        .filter(|(_, at)| now.since(*at).as_millis() < 320)
        .map(|r| r.0);
    let over = view.draft_over().is_some();
    let text = if editing() {
        f().draft.clone().unwrap_or_else(|| shown.clone())
    } else {
        shown.clone()
    };
    let mut release = move || {
        if let Some((d, t)) = held() {
            let mut w = f.write();
            for _ in 0..held_steps(now.since(t).as_millis() as u64) {
                let _ = w.step_by(d, false, now);
            }
            held.set(None);
            on_change.call(w.value);
        }
    };
    let can_dn = view.can(-1);
    let can_up = view.can(1);
    rsx! {
        div { class: "oxf-nf {size}", "data-disabled": "{disabled}",
            div { class: "oxf-head",
                label { class: if scrub { "oxf-lab scrub" } else { "oxf-lab" }, r#for: "{id}",
                    onpointerdown: move |e: PointerEvent| if scrub && !disabled {
                        e.prevent_default();
                        scrub_from.set(Some((e.client_coordinates().x, f().value)));
                    },
                    "{label}"
                }
                if let Some(n) = &note { span { key: "{n}", class: "oxf-note", role: "status", "{n}" } }
            }
            div { class: "oxf-shell", "data-over": "{over}",
                button { class: if refused == Some(Edge::Min) { "oxf-step shake" } else { "oxf-step" }, r#type: "button", tabindex: "-1",
                    disabled: disabled, "aria-label": "Decrease", "data-limit": "{!can_dn}",
                    onpointerdown: move |_| { dir.set(-1); held.set(Some((-1, now))); },
                    onpointerup: move |_| release(), onpointerleave: move |_| release(),
                    Icon { name: "minus", size: 13.0 }
                }
                div { class: "oxf-num", "data-dir": if dir() > 0 { "up" } else { "down" }, style: "--strain:{strain}px",
                    if !prefix.is_empty() { span { class: "oxf-aff", "{prefix}" } }
                    span { class: "oxf-val",
                        input { id: "{id}", class: if editing() { "oxf-in editing" } else { "oxf-in" }, r#type: "text", inputmode: "decimal",
                            role: "spinbutton", value: "{text}", disabled,
                            style: "width:{text.chars().count().max(1)}ch",
                            "aria-valuemin": "{min}", "aria-valuemax": "{max}", "aria-valuenow": "{v}",
                            "aria-valuetext": "{prefix}{shown}{suf}",
                            "aria-describedby": if description.is_some() { "{id}-d" },
                            onfocus: move |_| editing.set(true),
                            onblur: move |_| { editing.set(false); let o = f().value; let n = f.write().commit(); if n != o { dir.set(if n > o { 1 } else { -1 }); on_change.call(n) } },
                            oninput: move |e| f.write().type_draft(&e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                let sh = e.modifiers().shift();
                                let step = |d: i32, large: bool, f: &mut Signal<Nf>| { let r = f.write().step_by(d, large, now); if let Ok(n) = r { on_change.call(n) } };
                                match e.key() {
                                    Key::ArrowUp => { e.prevent_default(); dir.set(1); step(1, sh, &mut f) }
                                    Key::ArrowDown => { e.prevent_default(); dir.set(-1); step(-1, sh, &mut f) }
                                    Key::PageUp => { e.prevent_default(); dir.set(1); step(1, true, &mut f) }
                                    Key::PageDown => { e.prevent_default(); dir.set(-1); step(-1, true, &mut f) }
                                    Key::Home if f().draft.is_none() => { e.prevent_default(); dir.set(-1); on_change.call(f.write().jump(Edge::Min)) }
                                    Key::End if f().draft.is_none() => { e.prevent_default(); dir.set(1); on_change.call(f.write().jump(Edge::Max)) }
                                    Key::Enter => { let o = f().value; let n = f.write().commit(); dir.set(if n >= o { 1 } else { -1 }); on_change.call(n) }
                                    Key::Escape => f.write().revert(),
                                    _ => {}
                                }
                            },
                        }
                        if !editing() { Wheels { text: shown.clone() } }
                    }
                    if !suf.is_empty() { span { key: "{suf}", class: "oxf-aff suf", "{suf}" } }
                }
                button { class: if refused == Some(Edge::Max) { "oxf-step shake" } else { "oxf-step" }, r#type: "button", tabindex: "-1",
                    disabled: disabled, "aria-label": "Increase", "data-limit": "{!can_up}",
                    onpointerdown: move |_| { dir.set(1); held.set(Some((1, now))); },
                    onpointerup: move |_| release(), onpointerleave: move |_| release(),
                    Icon { name: "plus", size: 13.0 }
                }
            }
            if let Some(d) = &description { div { id: "{id}-d", class: "oxf-desc", "{d}" } }
            if let Some((x0, v0)) = scrub_from() {
                div { class: "oxf-drag",
                    onpointermove: move |e: PointerEvent| {
                        let steps = ((e.client_coordinates().x - x0) / 4.0).trunc();
                        let mut w = f.write();
                        let n = (v0 + steps * w.step).clamp(w.min, w.max);
                        let n = oxyn_core::kit::fields::round_to(n, w.places());
                        if n != w.value { dir.set(if n > w.value { 1 } else { -1 }); w.value = n; on_change.call(n); }
                    },
                    onpointerup: move |_| scrub_from.set(None),
                }
            }
        }
    }
}

/// Scrub input (Arc behaviour): a Figma-style number field. Drag the label sideways to scrub
/// (Shift ten times coarser and snapped to the coarse grid, Alt ten times finer, with the
/// multiplier shown mid-drag); type values, units and arithmetic such as `12*2`; Escape cancels
/// a scrub or restores typing; double-click the label to reset. Past a limit the field strains up
/// to 12 px with rubber-band resistance and springs back; invalid input and steps against a limit
/// kick it. With both limits a quiet track shows where the value sits; `wrap` turns it into a
/// rotation that wraps from max to min.
#[component]
pub fn ScrubInput(
    /// Label (the drag handle).
    label: String,
    /// Starting value.
    #[props(default)]
    value: f64,
    /// Lower limit.
    #[props(default = f64::NEG_INFINITY)]
    min: f64,
    /// Upper limit.
    #[props(default = f64::INFINITY)]
    max: f64,
    /// One step.
    #[props(default = 1.0)]
    step: f64,
    /// Pointer travel for one step.
    #[props(default = 2.0)]
    px_per_step: f32,
    /// Wrap from max to min.
    #[props(default)]
    wrap: bool,
    /// Units a typed value may carry; more than one makes the unit a button that cycles them.
    #[props(default)]
    units: Vec<String>,
    /// Starting unit.
    #[props(default)]
    unit: String,
    /// Double-click the label to return here.
    #[props(default)]
    reset_value: Option<f64>,
    /// Show the track fill (needs both limits).
    #[props(default = true)]
    track: bool,
    /// `sm` · `md`.
    #[props(default = "md".to_string())]
    size: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock (the kick).
    now: ClockMs,
    /// Every change: value and unit.
    #[props(default)]
    on_change: EventHandler<(f64, String)>,
    /// Once a change settles.
    #[props(default)]
    on_commit: EventHandler<(f64, String)>,
) -> Element {
    let mut s = use_signal(move || {
        let mut x = Scrub::new(value, min, max, step);
        x.wrap = wrap;
        x.px_per_step = px_per_step;
        x
    });
    let start_unit = if unit.is_empty() {
        units.first().cloned().unwrap_or_default()
    } else {
        unit.clone()
    };
    let mut u = use_signal(move || start_unit);
    let mut draft = use_signal(|| None::<String>);
    let mut gear = use_signal(|| Gear::Normal);
    let mut kick_at = use_signal(|| None::<ClockMs>);
    let id = slug("sc", &label);
    let sv = s();
    let multi = units.len() > 1;
    let ul = units.clone();
    let text = draft().unwrap_or_else(|| sv.text());
    let dragging = sv.drag.is_some();
    let kicking = sv.kicking(now) || kick_at().is_some_and(|k| now.since(k).as_millis() < 320);
    let frac = sv.fraction();
    let lean = sv.over_px;
    let ul2 = ul.clone();
    rsx! {
        div { class: "oxf-sc {size}", "data-disabled": "{disabled}", "data-drag": "{dragging}",
            div { class: if kicking { "oxf-scbox kick" } else { "oxf-scbox" }, style: "--lean:{lean}px",
                label { class: "oxf-sclab", r#for: "{id}", title: "Drag to change · Shift ×10 · Alt ×0.1 · double-click to reset",
                    onpointerdown: move |e: PointerEvent| if !disabled {
                        e.prevent_default();
                        let m = e.modifiers();
                        let g = Gear::from_mods(m.shift(), m.alt());
                        gear.set(g);
                        s.write().begin(e.client_coordinates().x as f32, g);
                    },
                    ondoubleclick: move |_| if let Some(r) = reset_value {
                        let mut w = s.write();
                        w.value = r;
                        on_change.call((r, u()));
                        on_commit.call((r, u()));
                    },
                    "{label}"
                }
                input { id: "{id}", class: "oxf-scin", r#type: "text", inputmode: "decimal", value: "{text}", disabled,
                    role: "spinbutton", "aria-valuenow": "{sv.value}",
                    "aria-valuemin": if min.is_finite() { "{min}" }, "aria-valuemax": if max.is_finite() { "{max}" },
                    "aria-valuetext": "{sv.text()} {u}",
                    onfocus: move |_| draft.set(Some(s().text())),
                    oninput: move |e| draft.set(Some(e.value())),
                    onblur: move |_| if let Some(d) = draft() {
                        let list: Vec<String> = ul.clone();
                        let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                        let r = s.write().commit_text(&d, &refs, now);
                        draft.set(None);
                        match r {
                            Some((v, nu)) => { if let Some(n) = nu { u.set(n) } on_commit.call((v, u())) }
                            None => kick_at.set(Some(now)),
                        }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        let m = e.modifiers();
                        let g = Gear::from_mods(m.shift(), m.alt());
                        match e.key() {
                            Key::ArrowUp | Key::ArrowDown => {
                                e.prevent_default();
                                let d = if e.key() == Key::ArrowUp { 1 } else { -1 };
                                let v = s.write().key(d, g, now);
                                draft.set(Some(s().text()));
                                on_change.call((v, u()));
                                on_commit.call((v, u()));
                            }
                            Key::Enter => {
                                let list: Vec<String> = ul2.clone();
                                let refs: Vec<&str> = list.iter().map(String::as_str).collect();
                                let d = draft().unwrap_or_else(|| s().text());
                                let r = s.write().commit_text(&d, &refs, now);
                                match r {
                                    Some((v, nu)) => { if let Some(n) = nu { u.set(n) } draft.set(Some(s().text())); on_change.call((v, u())); on_commit.call((v, u())) }
                                    None => kick_at.set(Some(now)),
                                }
                            }
                            Key::Escape => { e.prevent_default(); draft.set(Some(s().text())) }
                            _ => {}
                        }
                    },
                }
                if multi {
                    button { key: "{u}", class: "oxf-unit", r#type: "button", disabled, "aria-label": "Unit {u}, change unit",
                        onclick: move |_| {
                            let refs: Vec<&str> = units.iter().map(String::as_str).collect();
                            let n = next_unit(&refs, &u());
                            u.set(n.clone());
                            on_change.call((s().value, n.clone()));
                            on_commit.call((s().value, n));
                        },
                        "{u}"
                    }
                } else if !u().is_empty() {
                    span { class: "oxf-unitq", "{u}" }
                }
                if track && sv.bounded() {
                    span { class: "oxf-track", "aria-hidden": "true", span { class: "oxf-fill", style: "width:{frac * 100.0}%" } }
                }
            }
            if dragging {
                div { class: "oxf-drag",
                    onpointermove: move |e: PointerEvent| {
                        let m = e.modifiers();
                        let g = Gear::from_mods(m.shift(), m.alt());
                        gear.set(g);
                        let before = s().value;
                        let v = s.write().drag_to(e.client_coordinates().x as f32, g);
                        if v != before { draft.set(None); on_change.call((v, u())) }
                    },
                    onpointerup: move |_| { let v = s.write().end(); on_commit.call((v, u())) },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { let v = s.write().cancel(); on_change.call((v, u())) },
                    if let Some(b) = gear().badge() { span { class: "oxf-gear", "{b}" } }
                }
            }
        }
    }
}

/// Elastic slider (Arc behaviour): a slider that stretches like rubber past its ends and springs
/// back on release, pushing the start and end icons outward; the thumb swells while dragged and a
/// value tag trails it, tilting with its speed; detents click the thumb in and hold it for about
/// 7 px (their tick pops). Home/End at a limit stretch the track toward the press.
#[component]
pub fn RubberSlider(
    /// Label.
    label: String,
    /// Starting value.
    #[props(default)]
    value: f64,
    /// Lowest value.
    #[props(default)]
    min: f64,
    /// Highest value.
    #[props(default = 100.0)]
    max: f64,
    /// Step.
    #[props(default = 1.0)]
    step: f64,
    /// Detents: value and label (empty for an unlabelled tick). Labelled ones can be clicked.
    #[props(default)]
    marks: Vec<(f64, String)>,
    /// Text after the value ("%").
    #[props(default)]
    suffix: String,
    /// Show the value beside the label.
    #[props(default = true)]
    show_value: bool,
    /// Icon before the track.
    #[props(default)]
    start_icon: Option<String>,
    /// Icon after the track.
    #[props(default)]
    end_icon: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Every change.
    #[props(default)]
    on_change: EventHandler<f64>,
    /// Once a drag ends, a key lands or a mark is clicked.
    #[props(default)]
    on_commit: EventHandler<f64>,
) -> Element {
    let mut v = use_signal(|| value.clamp(min, max));
    let mut drag = use_signal(|| None::<(f32, f32)>);
    let mut over = use_signal(|| 0f32);
    let mut origin = use_signal(|| 0f64);
    let mut len = use_signal(|| 0f32);
    let mut el = use_signal(|| None::<Rc<MountedData>>);
    let mut hit = use_signal(|| None::<(usize, u32)>);
    let mut keyed = use_signal(|| false);
    let span = (max - min).max(f64::EPSILON);
    let frac = ((v() - min) / span) as f32;
    let q = move |x: f64| ((x - min) / step).round() * step + min;
    let fmt = |x: f64| {
        let d = oxyn_core::kit::fields::decimals_of(step);
        format!("{x:.d$}{suffix}")
    };
    let measure = move || {
        if let Some(r) = el.peek().clone() {
            spawn(async move {
                if let Ok(rc) = r.get_client_rect().await {
                    origin.set(rc.origin.x);
                    len.set(rc.size.width as f32);
                }
            });
        }
    };
    let marks2 = use_signal(|| marks.clone());
    let mut at_px = move |px: f32| {
        let l = *len.peek();
        if l <= 0.0 {
            return;
        }
        let clamped = px.clamp(0.0, l);
        let dets: Vec<f32> = marks2
            .peek()
            .iter()
            .map(|(m, _)| ((m - min) / span) as f32 * l)
            .collect();
        let (land, h) = detent(clamped, &dets, 7.0);
        if let Some(i) = h
            && (*hit.peek()).map(|x| x.0) != Some(i)
        {
            let n = hit.peek().map_or(0, |x| x.1) + 1;
            hit.set(Some((i, n)));
        } else if h.is_none() {
            hit.set(None);
        }
        over.set(if px < 0.0 {
            px
        } else if px > l {
            px - l
        } else {
            0.0
        });
        let nv = q(min + (land / l) as f64 * span).clamp(min, max);
        if nv != *v.peek() {
            v.set(nv);
            on_change.call(nv);
        }
    };
    let (st, push) = stretch(over());
    let tilt = drag().map_or(0.0, |d| tag_tilt(d.1));
    let show_tag = drag().is_some() || keyed();
    let track_style = if st < 0.0 {
        format!("transform:translateX({st}px);width:calc(100% + {}px)", -st)
    } else {
        format!("width:calc(100% + {st}px)")
    };
    let ps = if st < 0.0 { push } else { 0.0 };
    let pe = if st > 0.0 { push } else { 0.0 };
    let id = slug("es", &label);
    rsx! {
        div { class: "oxf-es", "data-disabled": "{disabled}", "data-drag": "{drag().is_some()}",
            div { class: "oxf-head",
                span { id: "{id}-l", class: "oxf-lab", "{label}" }
                if show_value { span { class: "oxf-esv", "{fmt(v())}" } }
            }
            div { class: "oxf-esrow",
                if let Some(i) = &start_icon { span { class: "oxf-esi", style: "transform:translateX({ps}px)", Icon { name: i.clone(), size: 14.0 } } }
                div { class: "oxf-eswrap",
                    onmounted: move |e| { el.set(Some(e.data())); measure(); },
                    onpointerdown: move |e: PointerEvent| if !disabled {
                        measure();
                        let x = (e.client_coordinates().x - *origin.peek()) as f32;
                        drag.set(Some((x, 0.0)));
                        at_px(x);
                    },
                    div { class: "oxf-estrack", style: "{track_style}",
                        span { class: "oxf-esfill", style: "width:{frac * 100.0}%" }
                    }
                    for (i, (m, l)) in marks.iter().enumerate() {
                        {
                            let f = ((m - min) / span) as f32 * 100.0;
                            let pop = hit().filter(|h| h.0 == i).map(|h| h.1);
                            let mv = *m;
                            rsx! {
                                span { key: "{i}-{pop:?}", class: if pop.is_some() { "oxf-tick pop" } else { "oxf-tick" }, style: "left:{f}%" }
                                if !l.is_empty() {
                                    button { class: "oxf-ml", r#type: "button", style: "left:{f}%", tabindex: "-1", disabled,
                                        onpointerdown: move |e| e.stop_propagation(),
                                        onclick: move |_| { v.set(mv); on_change.call(mv); on_commit.call(mv) },
                                        "{l}"
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxf-esthumb", style: "left:{frac * 100.0}%",
                        role: "slider", tabindex: if disabled { "-1" } else { "0" },
                        "aria-labelledby": "{id}-l", "aria-valuemin": "{min}", "aria-valuemax": "{max}", "aria-valuenow": "{v()}",
                        "aria-valuetext": "{fmt(v())}", "aria-disabled": "{disabled}",
                        onblur: move |_| keyed.set(false),
                        onkeyup: move |_| over.set(0.0),
                        onkeydown: move |e: KeyboardEvent| {
                            let large = span / 10.0;
                            let sh = e.modifiers().shift();
                            let cur = v();
                            let (want, dirn) = match e.key() {
                                Key::ArrowRight | Key::ArrowUp => (cur + if sh { large } else { step }, 1.0),
                                Key::ArrowLeft | Key::ArrowDown => (cur - if sh { large } else { step }, -1.0),
                                Key::PageUp => (cur + large, 1.0),
                                Key::PageDown => (cur - large, -1.0),
                                Key::Home => (min, -1.0),
                                Key::End => (max, 1.0),
                                _ => return,
                            };
                            e.prevent_default();
                            keyed.set(true);
                            let n = q(want).clamp(min, max);
                            if n == cur { over.set(14.0 * dirn); } else { v.set(n); on_change.call(n); on_commit.call(n); }
                        },
                        span { class: if show_tag { "oxf-estag on" } else { "oxf-estag" }, style: "transform:translateX(-50%) rotate({tilt}deg)", "aria-hidden": "true", "{fmt(v())}" }
                    }
                }
                if let Some(i) = &end_icon { span { class: "oxf-esi", style: "transform:translateX({pe}px)", Icon { name: i.clone(), size: 14.0 } } }
            }
            if drag().is_some() {
                div { class: "oxf-drag",
                    onpointermove: move |e: PointerEvent| {
                        let x = (e.client_coordinates().x - *origin.peek()) as f32;
                        let vx = drag().map_or(0.0, |d| x - d.0);
                        drag.set(Some((x, vx)));
                        at_px(x);
                    },
                    onpointerup: move |_| { drag.set(None); over.set(0.0); on_commit.call(v()) },
                }
            }
        }
    }
}

/// Morph select (Arc behaviour): the trigger surface grows into the list (corner radius morphing
/// with it) and folds back; one highlight glides between options; the chosen value rolls up or
/// down in the trigger depending on where it sits in the list. Type-ahead jumps to a label (it
/// picks while closed); above eight options (`searchable: "auto"`) a search field takes the
/// trigger's place when open. PageUp/PageDown move eight options, Home/End go to the ends (and
/// open on them when closed), Alt+Up picks and closes, Escape closes back to the trigger, Tab
/// closes without picking.
#[component]
pub fn UnfoldSelect(
    /// Label.
    label: String,
    /// Keep the label for assistive technology only.
    #[props(default)]
    hide_label: bool,
    /// Options.
    options: Vec<Choice>,
    /// Selected value.
    #[props(default)]
    value: Option<String>,
    /// Trigger text with no selection.
    #[props(default = "Select".to_string())]
    placeholder: String,
    /// `auto` (above eight options) · `on` · `off`.
    #[props(default = "auto".to_string())]
    searchable: String,
    /// Width of the open surface (never narrower than the trigger).
    #[props(default = 272.0)]
    panel_width: f64,
    /// Tallest the list gets before it scrolls.
    #[props(default = 296.0)]
    max_list_height: f64,
    /// `start` · `end`: which edge of the trigger stays put.
    #[props(default = "start".to_string())]
    align: String,
    /// Hidden input name for forms.
    #[props(default)]
    name: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock (type-ahead).
    now: ClockMs,
    /// Picked value.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let sel = value
        .as_ref()
        .and_then(|v| options.iter().position(|o| &o.value == v));
    let mut m = use_signal(move || Ms::new(options.clone(), sel));
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut list = use_signal(|| None::<Rc<MountedData>>);
    let mut tw = use_signal(|| 160f64);
    let id = slug("ms", &label);
    let ms = m();
    let search = ms.searchable(&searchable);
    let vis = ms.visible();
    let row = ms.active_row().unwrap_or(0);
    let lab = ms.label(&placeholder);
    let roll = match ms.roll {
        1 => "up",
        -1 => "down",
        _ => "none",
    };
    let w = panel_width.max(tw());
    let measure = move || {
        if let Some(r) = trig.peek().clone() {
            spawn(async move {
                if let Ok(rc) = r.get_client_rect().await {
                    tw.set(rc.size.width);
                }
            });
        }
    };
    let keys = move |e: KeyboardEvent| {
        let k = e.key();
        let alt = e.modifiers().alt();
        let open = m().open;
        let space = k == Key::Character(" ".to_string());
        let home = k == Key::Home;
        match k {
            Key::ArrowDown | Key::ArrowUp if !open => {
                e.prevent_default();
                measure();
                m.write().open_at(OpenAt::Selected);
                focus(list());
            }
            Key::Enter if !open => {
                e.prevent_default();
                measure();
                m.write().open_at(OpenAt::Selected);
                focus(list());
            }
            Key::Character(_) if !open && space => {
                e.prevent_default();
                measure();
                m.write().open_at(OpenAt::Selected);
                focus(list());
            }
            Key::Home | Key::End if !open => {
                e.prevent_default();
                measure();
                m.write()
                    .open_at(if home { OpenAt::First } else { OpenAt::Last });
                focus(list());
            }
            Key::ArrowUp if open && alt => {
                e.prevent_default();
                if let Some(c) = m.write().pick_active().cloned() {
                    on_change.call(c.value)
                }
                focus(trig());
            }
            Key::ArrowDown if open => {
                e.prevent_default();
                m.write().move_by(1)
            }
            Key::ArrowUp if open => {
                e.prevent_default();
                m.write().move_by(-1)
            }
            Key::PageDown if open => {
                e.prevent_default();
                m.write().move_by(8)
            }
            Key::PageUp if open => {
                e.prevent_default();
                m.write().move_by(-8)
            }
            Key::Home if open => {
                e.prevent_default();
                m.write().move_by(-10_000)
            }
            Key::End if open => {
                e.prevent_default();
                m.write().move_by(10_000)
            }
            Key::Enter if open => {
                e.prevent_default();
                if let Some(c) = m.write().pick_active().cloned() {
                    on_change.call(c.value)
                }
                focus(trig());
            }
            Key::Escape if open => {
                e.prevent_default();
                e.stop_propagation();
                m.write().close();
                focus(trig());
            }
            Key::Tab if open => m.write().close(),
            Key::Character(c) if !search || !open => {
                if let Some(ch) = c
                    .chars()
                    .next()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                {
                    if ch == ' ' && open {
                        e.prevent_default();
                        if let Some(c) = m.write().pick_active().cloned() {
                            on_change.call(c.value)
                        }
                        focus(trig());
                    } else if ch != ' ' {
                        let before = m().selected;
                        m.write().type_ahead(ch, now);
                        let after = m().selected;
                        if !open
                            && after != before
                            && let Some(i) = after
                        {
                            on_change.call(m().options[i].value.clone())
                        }
                    }
                }
            }
            _ => {}
        }
    };
    rsx! {
        div { class: "oxf-ms {align}", "data-open": "{ms.open}", "data-disabled": "{disabled}",
            label { id: "{id}-l", class: if hide_label { "oxf-lab oxf-sr" } else { "oxf-lab" }, "{label}" }
            div { class: "oxf-msanchor",
                button { id: "{id}", class: "oxf-mstrig", r#type: "button", disabled,
                    "aria-haspopup": "listbox", "aria-expanded": "{ms.open}", "aria-labelledby": "{id}-l {id}",
                    onmounted: move |e| { trig.set(Some(e.data())); measure(); },
                    onclick: move |_| { measure(); if m().open { m.write().close() } else { m.write().open_at(OpenAt::Selected); focus(list()); } },
                    onkeydown: keys,
                    span { class: "oxf-msval", "data-roll": "{roll}",
                        span { key: "{lab}", class: if ms.selected.is_none() { "oxf-msv ph" } else { "oxf-msv" }, "{lab}" }
                    }
                    Icon { name: "chevron", size: 12.0 }
                }
                if ms.open {
                    div { class: "oxf-scrim", onclick: move |_| m.write().close() }
                    div { class: "oxf-mssurf", style: "--tw:{tw()}px;width:{w}px",
                        if search {
                            div { class: "oxf-mssearch",
                                Icon { name: "search", size: 12.0 }
                                input { r#type: "text", value: "{ms.query}", placeholder: "{lab}", "aria-label": "Search {label}",
                                    "aria-controls": "{id}-lb", "aria-activedescendant": "{id}-o{ms.active}",
                                    onmounted: move |e| list.set(Some(e.data())),
                                    oninput: move |e| m.write().search(&e.value()),
                                    onkeydown: keys,
                                }
                            }
                        } else {
                            div { class: "oxf-mshead", "aria-hidden": "true",
                                span { class: "oxf-msv", "{lab}" }
                                Icon { name: "chevron", size: 12.0 }
                            }
                        }
                        div { id: "{id}-lb", class: "oxf-mslist", role: "listbox", tabindex: if search { "-1" } else { "0" },
                            "aria-labelledby": "{id}-l", "aria-activedescendant": "{id}-o{ms.active}",
                            style: "max-height:{max_list_height}px",
                            onmounted: move |e| if !search { list.set(Some(e.data())) },
                            onkeydown: keys,
                            if !vis.is_empty() { span { class: "oxf-mshl", style: "transform:translateY({row as f64 * 34.0}px)", "aria-hidden": "true" } }
                            for i in vis.iter().copied() {
                                {
                                    let o = ms.options[i].clone();
                                    let selected = ms.selected == Some(i);
                                    rsx! {
                                        div { key: "{o.value}", id: "{id}-o{i}", class: "oxf-msopt", role: "option",
                                            "aria-selected": "{selected}", "aria-disabled": "{o.disabled}",
                                            "data-active": "{ms.active == i}",
                                            onpointerenter: move |_| if !m().options[i].disabled { m.write().active = i },
                                            onclick: move |_| { if let Some(c) = m.write().choose(i).cloned() { on_change.call(c.value) } focus(trig()); },
                                            span { class: "oxf-msol", "{o.label}" if !o.hint.is_empty() { span { class: "oxf-msoh", "{o.hint}" } } }
                                            if selected { Icon { name: "check", size: 12.0 } }
                                        }
                                    }
                                }
                            }
                            if vis.is_empty() { div { class: "oxf-msempty", role: "status", "No matches" } }
                        }
                    }
                }
            }
            if let Some(n) = &name { input { r#type: "hidden", name: "{n}", value: "{ms.selected.map(|i| ms.options[i].value.clone()).unwrap_or_default()}" } }
        }
    }
}
