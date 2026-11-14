//! Form controls: `Switch`, `Checkbox`, `RadioGroup`, `SegmentedControl`, `Select`, `Slider`,
//! `TextField`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        segmented::{Segment, Segmented, SegmentedKind},
        select::{ListRow, Select as SelectModel, SelectKey, SelectOption},
        slider::{Mark, Slider as SliderModel, SliderKey},
        text_field::{CountLevel, FieldType, TextField as FieldModel, Validate},
        toggle::{Check, RadioGroup as RadioModel, RadioOption, Switch as SwitchModel},
    },
};

use super::{buttons::Spin, cls, nav_key, use_glide};
use crate::composer::icons::Icon;

fn uid(prefix: &str, label: &str) -> String {
    let slug: String = label
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_lowercase();
    format!("oxyn-{prefix}-{slug}")
}

/// Switch: on / off for things that apply right away. Label and second line, at the start or
/// across a settings row; the thumb stretches toward where it is going while pressed (pointer, or
/// Space held down). Oxyn plus: `xs` (16 px IDE) and `lg` sizes, a pending (applying) state with a
/// spinner thumb, a "requires restart" note, a hidden label and a form `name`.
#[component]
pub fn Switch(
    /// Label.
    label: String,
    /// Second line.
    #[props(default)]
    description: Option<String>,
    /// Initial value.
    #[props(default)]
    checked: bool,
    /// `start` (switch first) · `end` (settings row, switch at the end).
    #[props(default = "start".to_string())]
    placement: String,
    /// `xs` · `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Keep the label for screen readers only (Arc: the label is the switch's name).
    #[props(default)]
    hide_label: bool,
    /// Form field name (a hidden input carries `on` / `off`).
    #[props(default)]
    name: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Applying.
    #[props(default)]
    pending: bool,
    /// Show "Requires restart" after a change.
    #[props(default)]
    restart_note: bool,
    /// Review sheets: force `pressed` or `focus`.
    #[props(default)]
    state: Option<String>,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<bool>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = SwitchModel::new(checked);
        s.disabled = disabled;
        s.pending = pending;
        s
    });
    let mut changed = use_signal(|| false);
    let mut pressed = use_signal(|| false);
    let on = m.read().checked;
    let busy = m.read().pending;
    let id = uid("sw", &label);
    let class = cls(&[
        "oxyn-switch",
        &size,
        &placement,
        if on { "on" } else { "" },
        if *pressed.read() { "pressed" } else { "" },
        if busy { "pending" } else { "" },
        if disabled { "disabled" } else { "" },
        state.as_deref().unwrap_or(""),
    ]);
    let mut flip = move || {
        if let Some(v) = m.write().toggle(ClockMs(0)) {
            changed.set(true);
            on_change.call(v);
        }
    };
    rsx! {
        label { class: "{class}", r#for: "{id}",
            button { id: "{id}", class: "track", r#type: "button", role: "switch", "aria-checked": "{on}", "aria-busy": "{busy}",
                disabled, "aria-describedby": if description.is_some() { "{id}-d" },
                onpointerdown: move |_| pressed.set(true),
                onpointerup: move |_| pressed.set(false),
                onpointerleave: move |_| pressed.set(false),
                "aria-label": if hide_label { "{label}" },
                onclick: move |e| { e.prevent_default(); flip() },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter { e.prevent_default(); flip() }
                    // Holding Space stretches the thumb until release (the click toggles).
                    if e.key() == Key::Character(" ".into()) { pressed.set(true) }
                },
                onkeyup: move |e: KeyboardEvent| if e.key() == Key::Character(" ".into()) { pressed.set(false) },
                span { class: "thumb", if busy { Spin { size: 10.0 } } }
            }
            if let Some(n) = name.clone() { input { r#type: "hidden", name: "{n}", value: if on { "on" } else { "off" } } }
            span { class: if hide_label { "txt sr" } else { "txt" },
                span { class: "lb", "{label}" }
                if let Some(d) = description.clone() { span { class: "ds", id: "{id}-d", "{d}" } }
                if restart_note && *changed.read() { span { class: "note", role: "status", "Requires restart" } }
            }
        }
    }
}

/// Checkbox: a real checkbox with a label and second line, the some-of-them state for Select
/// all, an error, and a card row; the fill scales in while the check draws, and switching to
/// indeterminate morphs the same path into a dash. A full control-height hit box. Oxyn plus:
/// sizes, draw speed, a tri-state cycle, a strike-through todo style and a bulk "approve all" row.
#[component]
pub fn Checkbox(
    /// Label.
    label: String,
    /// Second line.
    #[props(default)]
    description: Option<String>,
    /// `unchecked` · `checked` · `indeterminate`.
    #[props(default = "unchecked".to_string())]
    value: String,
    /// Error text (also marks it invalid).
    #[props(default)]
    error: Option<String>,
    /// `plain` · `card` · `todo` (strike-through when checked) · `bulk` (approve-all row).
    #[props(default = "plain".to_string())]
    variant: String,
    /// Accessible name when there is no visible label (falls back to "Checkbox").
    #[props(default)]
    aria_label: Option<String>,
    /// Form field name.
    #[props(default)]
    name: Option<String>,
    /// Required.
    #[props(default)]
    required: bool,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Draw-time multiplier (1 = 240 ms).
    #[props(default = 1.0)]
    speed: f32,
    /// Cycle through indeterminate on click.
    #[props(default)]
    tri_state: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<Check>,
) -> Element {
    let init = match value.as_str() {
        "checked" => Check::Checked,
        "indeterminate" => Check::Indeterminate,
        _ => Check::Unchecked,
    };
    let mut v = use_signal(|| init);
    let cur = *v.read();
    let id = uid("cb", &label);
    let draw = 240.0 * speed;
    rsx! {
        label { class: cls(&["oxyn-check", &variant, &size, cur.id(), if error.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }]),
            style: "--draw:{draw:.0}ms",
            input { id: "{id}", r#type: "checkbox", class: "native", checked: cur == Check::Checked, disabled, required,
                name: if let Some(n) = &name { "{n}" },
                "aria-label": if label.is_empty() { aria_label.clone().unwrap_or("Checkbox".into()) } else { String::new() },
                "aria-checked": "{cur.aria()}", "aria-invalid": "{error.is_some()}",
                "aria-describedby": if description.is_some() || error.is_some() { "{id}-d" },
                onchange: move |_| { let n = v.peek().next(tri_state); v.set(n); on_change.call(n) },
            }
            span { class: "box", "aria-hidden": "true",
                svg { view_box: "0 0 16 16", width: "100%", height: "100%",
                    path { class: "mark", d: if cur == Check::Indeterminate { "M4 8 8 8 12 8" } else { "M3.5 8.5 6.5 11.5 12.5 4.5" }, path_length: "1" }
                }
            }
            span { class: "txt",
                if !label.is_empty() { span { class: "lb", "{label}" } }
                if let Some(e) = error.clone() { span { class: "ds err", id: "{id}-d", "{e}" } }
                else if let Some(d) = description.clone() { span { class: "ds", id: "{id}-d", "{d}" } }
            }
        }
    }
}

/// Radio group: a fieldset whose legend names it; one tab stop, arrows move and select, one
/// highlight glides to the chosen row while the new dot springs in and the old one shrinks away;
/// a native radio `name` for forms. Oxyn plus: list / cards / inline / segmented layouts,
/// descriptions, disabled-with-reason tooltips.
#[component]
pub fn RadioGroup(
    /// Group label.
    label: String,
    /// Options.
    options: Vec<RadioOption>,
    /// Selected value.
    #[props(default)]
    value: Option<String>,
    /// `list` · `cards` · `inline` · `segmented`.
    #[props(default = "list".to_string())]
    layout: String,
    /// Native radio name for form submission (a hidden input carries the value).
    #[props(default)]
    name: Option<String>,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| RadioModel::new(options.clone(), value.as_deref()));
    let cur_value = m.read().value.clone().unwrap_or_default();
    let glide = use_glide();
    let sel = m.read().selected();
    let tab = m.read().tab_stop();
    let id = uid("rg", &label);
    rsx! {
        div { class: "oxyn-radio {layout}", role: "radiogroup", "aria-labelledby": "{id}",
            onmounted: move |e| glide.set_root(e.data()),
            onkeydown: move |e: KeyboardEvent| {
                // Radios use both axes.
                let k = nav_key(&e.key(), true).or_else(|| nav_key(&e.key(), false));
                if let Some(k) = k {
                    e.prevent_default();
                    let r = m.write().key(k, ClockMs(0));
                    if let Some(v) = r {
                        glide.go(m.peek().selected(), true);
                        on_change.call(v);
                    }
                }
            },
            span { class: "legend", id: "{id}", "{label}" }
            if let Some(n) = name.clone() { input { r#type: "hidden", name: "{n}", value: "{cur_value}" } }
            span { class: "{glide.class(\"hl\")}", style: "{glide.style()}", "aria-hidden": "true" }
            for (i, o) in options.iter().enumerate() {
                {
                    let on = sel == Some(i);
                    let dis = o.disabled.is_some();
                    rsx! {
                        div { class: cls(&["opt", if on { "on" } else { "" }, if on && !glide.measured() { "hl-self" } else { "" }, if dis { "disabled" } else { "" }]),
                            role: "radio", "aria-checked": "{on}", "aria-disabled": "{dis}",
                            tabindex: if tab == Some(i) { "0" } else { "-1" },
                            title: if let Some(r) = &o.disabled { "{r}" },
                            onmounted: move |e| { glide.set_item(i, e.data()); if on { glide.go(Some(i), true) } },
                            onclick: move |_| { let r = m.write().pick(i, ClockMs(0)); if let Some(v) = r { glide.go(Some(i), false); on_change.call(v) } },
                            span { class: "dot", "aria-hidden": "true", span { class: "in" } }
                            span { class: "txt",
                                span { class: "lb", "{o.label}" }
                                if let Some(d) = o.description.clone() { span { class: "ds", "{d}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Segmented control: two to eight options, a raised pill (tray) or underline that glides with
/// its leading edge first and can be dragged across; icons, counts and badges; radio or tabs
/// keyboard. Oxyn plus: up to 8 options, vertical orientation, drag on / off.
#[component]
pub fn SegmentedControl(
    /// Options.
    options: Vec<Segment>,
    /// Selected value (default: first enabled option).
    #[props(default)]
    value: Option<String>,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// `tray` · `underline`.
    #[props(default = "tray".to_string())]
    variant: String,
    /// `radio` · `tabs`.
    #[props(default = "radio".to_string())]
    kind: String,
    /// Stretch to the container.
    #[props(default)]
    full_width: bool,
    /// Vertical.
    #[props(default)]
    vertical: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Drag the pill across.
    #[props(default = true)]
    draggable: bool,
    /// Accessible name.
    #[props(default = "Options".to_string())]
    aria_label: String,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Plan #27 (Arc UI `onOptionIntent`): the pointer or focus reached a segment before it was
    /// chosen, so the host can start loading what it shows.
    #[props(default)]
    on_option_intent: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = Segmented::new(options.clone(), value.as_deref());
        s.kind = if kind == "tabs" {
            SegmentedKind::Tabs
        } else {
            SegmentedKind::Radio
        };
        s.disabled = disabled;
        s.draggable = draggable;
        s
    });
    let glide = use_glide();
    let k = m.read().kind;
    let sel = m.read().selected();
    let focus = m.read().focus.or(sel);
    rsx! {
        div { class: cls(&["oxyn-seg", &variant, &size, if full_width { "full" } else { "" }, if vertical { "vertical" } else { "" }, if disabled { "disabled" } else { "" }]),
            role: "{k.role()}", "aria-label": "{aria_label}", "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            onmounted: move |e| glide.set_root(e.data()),
            onkeydown: move |e: KeyboardEvent| {
                if let Some(nk) = nav_key(&e.key(), vertical) {
                    e.prevent_default();
                    let r = m.write().key(nk, ClockMs(0));
                    if let Some(v) = r { glide.go(m.peek().selected(), false); on_change.call(v); }
                } else if matches!(e.key(), Key::Enter) || e.key() == Key::Character(" ".into()) {
                    e.prevent_default();
                    let r = m.write().activate(ClockMs(0));
                    if let Some(v) = r { glide.go(m.peek().selected(), false); on_change.call(v); }
                }
            },
            span { class: "{glide.class(\"pill\")}", style: "{glide.style()}", "aria-hidden": "true" }
            for (i, o) in options.iter().enumerate() {
                {
                    let on = sel == Some(i);
                    let radio = k == SegmentedKind::Radio;
                    rsx! {
                        button { class: cls(&["opt", if on { "on" } else { "" }, if on && !glide.measured() { "pill-self" } else { "" }]), r#type: "button",
                            role: "{k.item_role()}", tabindex: if focus == Some(i) { "0" } else { "-1" }, disabled: o.disabled || disabled,
                            "aria-checked": if radio { "{on}" },
                            "aria-selected": if !radio { "{on}" },
                            "aria-label": if o.label.is_empty() { o.icon.clone().unwrap_or_default() } else { String::new() },
                            onmounted: move |e| { glide.set_item(i, e.data()); if on { glide.go(Some(i), true) } },
                            onclick: move |_| { let r = m.write().select(i, ClockMs(0)); if let Some(v) = r { glide.go(Some(i), false); on_change.call(v) } },
                            onpointerenter: { let v = o.value.clone(); move |_| if !on { on_option_intent.call(v.clone()) } },
                            onfocus: { let v = o.value.clone(); move |_| if !on { on_option_intent.call(v.clone()) } },
                            if let Some(ic) = o.icon.clone() { Icon { name: ic, size: 14.0 } }
                            if !o.label.is_empty() { span { class: "lb", "data-text": "{o.label}", "{o.label}" } }
                            if let Some(c) = o.count { span { class: "cnt", key: "{c}", "{c}" } }
                            if let Some(b) = o.badge.clone() { span { class: "bdg", "{b}" } }
                        }
                    }
                }
            }
        }
    }
}

/// Select: one choice in the text field's look. Icons and a second line per option, a gliding
/// highlight, type-ahead, a list that opens upward when there is no room, a chevron that turns;
/// a later option rises in from below and an earlier one drops from above, while the real value
/// sits in a visually hidden span. Oxyn plus: searchable, grouped options, async options with
/// skeleton rows, and a native `<select>` renderer.
#[component]
pub fn Select(
    /// Options.
    options: Vec<SelectOption>,
    /// Value.
    #[props(default)]
    value: Option<String>,
    /// Placeholder.
    #[props(default = "Choose…".to_string())]
    placeholder: String,
    /// Label.
    #[props(default)]
    label: Option<String>,
    /// Hint.
    #[props(default)]
    hint: Option<String>,
    /// Error.
    #[props(default)]
    error: Option<String>,
    /// "Optional" next to the label.
    #[props(default)]
    optional: bool,
    /// Required.
    #[props(default)]
    required: bool,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Filter field.
    #[props(default)]
    searchable: bool,
    /// Options loading.
    #[props(default)]
    loading: bool,
    /// Open upward (the host passes the placement result).
    #[props(default)]
    up: bool,
    /// Start open (review sheets).
    #[props(default)]
    open: bool,
    /// Form field name.
    #[props(default)]
    name: Option<String>,
    /// Oxyn plus: render a native `<select>` (mobile pickers, very long lists).
    #[props(default)]
    native: bool,
    /// Clock (type-ahead window).
    #[props(default)]
    now: ClockMs,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = SelectModel::new(options.clone(), value.as_deref());
        s.placeholder = placeholder.clone();
        s.required = required;
        s.disabled = disabled;
        s.searchable = searchable;
        s.loading = loading;
        if open {
            s.open_list(ClockMs(0));
        }
        s
    });
    let glide = use_glide();
    let id = uid("sel", label.as_deref().unwrap_or(&placeholder));
    let st = m.read().clone();
    let shown = st.display().to_string();
    let empty = st.value.is_none();
    let hl = st.highlighted();
    let vis = st.visible();
    if native {
        let cur = st.value.clone().unwrap_or_default();
        return rsx! {
            div { class: cls(&["oxyn-field oxyn-select native", &size, if error.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }]),
                if let Some(l) = label.clone() { label { class: "flabel", r#for: "{id}", "{l}" if optional { span { class: "opt", "Optional" } } } }
                div { class: "control",
                    select { id: "{id}", class: "in", disabled, required, name: if let Some(n) = &name { "{n}" },
                        "aria-invalid": "{error.is_some()}",
                        onchange: move |e: FormEvent| { let v = e.value(); let i = m.peek().options.iter().position(|o| o.value == v); if let Some(i) = i { let r = m.write().choose(i, now); if let Some(v) = r { on_change.call(v) } } },
                        if cur.is_empty() { option { value: "", disabled: true, selected: true, "{placeholder}" } }
                        for o in options.iter() { option { value: "{o.value}", disabled: o.disabled, selected: o.value == cur, "{o.label}" } }
                    }
                    span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
                }
                if let Some(e) = error.clone() { div { class: "msg err", role: "alert", "{e}" } }
                else if let Some(h) = hint.clone() { div { class: "msg", "{h}" } }
            }
        };
    }
    let mut key = move |k: SelectKey| {
        let r = m.write().key(k, now);
        let pos = {
            let s = m.peek();
            s.highlighted()
                .and_then(|h| s.visible().iter().position(|&x| x == h))
        };
        glide.go(pos, true);
        if let Some(v) = r {
            on_change.call(v);
        }
    };
    rsx! {
        div { class: cls(&["oxyn-field oxyn-select", &size, if error.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }, if st.open { "open" } else { "" }, if up { "up" } else { "" }]),
            if let Some(l) = label.clone() { label { class: "flabel", id: "{id}-l", r#for: "{id}", "{l}" if optional { span { class: "opt", "Optional" } } } }
            div { class: "anchor",
            button { id: "{id}", class: cls(&["control trigger", if empty { "placeholder" } else { "" }]), r#type: "button", disabled,
                role: "combobox", "aria-haspopup": "listbox", "aria-expanded": "{st.open}", "aria-controls": "{id}-list",
                "aria-labelledby": if label.is_some() { "{id}-l {id}" },
                "aria-invalid": "{error.is_some()}", "aria-required": "{required}",
                "aria-activedescendant": if let (true, Some(h)) = (st.open, hl) { "{id}-o{h}" },
                onclick: move |_| { let o = m.peek().open; if o { m.write().close() } else { m.write().open_list(now); let p = m.peek().glide.target; glide.go(p, true); } },
                onkeydown: move |e: KeyboardEvent| {
                    let k = match e.key() {
                        Key::ArrowDown => Some(SelectKey::Down), Key::ArrowUp => Some(SelectKey::Up),
                        Key::Home => Some(SelectKey::Home), Key::End => Some(SelectKey::End),
                        Key::Enter => Some(SelectKey::Enter), Key::Escape => Some(SelectKey::Escape),
                        Key::Tab => Some(SelectKey::Tab), Key::Backspace => Some(SelectKey::Backspace),
                        Key::Character(c) if c == " " => Some(SelectKey::Space),
                        Key::Character(c) => c.chars().next().map(SelectKey::Char),
                        _ => None,
                    };
                    if let Some(k) = k { if k != SelectKey::Tab { e.prevent_default(); } key(k) }
                },
                if let Some(o) = st.selected() { if let Some(i) = o.icon.clone() { Icon { name: i, size: 14.0 } } }
                span { class: "sr", "{shown}" }
                span { class: "val", key: "{shown}", "aria-hidden": "true", "data-move": if let Some((d, _)) = st.moved { if d == oxyn_core::primitives::select::ValueMove::Later { "later" } else { "earlier" } } else { "" }, "{shown}" }
                span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
            }
            if let Some(n) = name { input { r#type: "hidden", name: "{n}", value: "{st.value.clone().unwrap_or_default()}" } }
            if st.open {
                div { class: "list", id: "{id}-list", role: "listbox", "aria-labelledby": if label.is_some() { "{id}-l" } else { "" },
                    onmounted: move |e| glide.set_root(e.data()),
                    onpointerleave: move |_| glide.go(None, false),
                    if searchable { div { class: "filter", Icon { name: "search", size: 13.0 } span { class: if st.query.is_empty() { "q ph" } else { "q" }, if st.query.is_empty() { "Filter…" } else { "{st.query}" } } } }
                    span { class: "{glide.class(\"hl\")}", style: "{glide.style()}", "aria-hidden": "true" }
                    for row in st.rows().iter() {
                        match row {
                            ListRow::Group(g) => rsx! { div { class: "grp", role: "presentation", "{g}" } },
                            ListRow::Skeleton => rsx! { div { class: "row skel", "aria-hidden": "true", span {} } },
                            ListRow::Empty => rsx! { div { class: "empty", role: "status", "No matches" } },
                            ListRow::Option(oi) => {
                                let oi = *oi;
                                let o = st.options[oi].clone();
                                let pos = vis.iter().position(|&x| x == oi).unwrap_or(0);
                                let on = st.value.as_deref() == Some(o.value.as_str());
                                let is_hl = hl == Some(oi);
                                rsx! {
                                    div { id: "{id}-o{oi}", class: cls(&["row", if is_hl && !glide.measured() { "hl-self" } else { "" }, if o.disabled { "disabled" } else { "" }]),
                                        role: "option", "aria-selected": "{on}", "aria-disabled": "{o.disabled}",
                                        onmounted: move |e| glide.set_item(pos, e.data()),
                                        onpointerenter: move |_| { m.write().hover(Some(pos), now); glide.go(Some(pos), false); },
                                        onclick: move |_| { let r = m.write().choose(oi, now); glide.go(None, false); if let Some(v) = r { on_change.call(v) } },
                                        if let Some(i) = o.icon.clone() { span { class: "oi", Icon { name: i, size: 14.0 } } }
                                        span { class: "txt", span { class: "lb", "{o.label}" } if let Some(d) = o.description.clone() { span { class: "ds", "{d}" } } }
                                        if on { span { class: "tick", svg { view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }
            if let Some(e) = error.clone() { div { class: "msg err", role: "alert", "{e}" } }
            else if let Some(h) = hint.clone() { div { class: "msg", "{h}" } }
        }
    }
}

/// Slider: a value or a range on a track that follows the finger; marks, steps, large steps,
/// minimum gap between thumbs, a bubble over the active thumb. Oxyn plus: named detents that
/// snap, a readout tag (e.g. cost) and vertical orientation.
#[component]
pub fn Slider(
    /// Model.
    slider: SliderModel,
    /// Accessible name.
    label: String,
    /// Readout format: `#` is replaced by the value (`"#%"`, `"$#"`).
    #[props(default = "#".to_string())]
    format: String,
    /// Decimals in the readout.
    #[props(default)]
    decimals: usize,
    /// Show the value next to the label.
    #[props(default = true)]
    show_value: bool,
    /// Thumb labels for a range ("Min", "Max").
    #[props(default)]
    thumb_labels: Vec<String>,
    /// Oxyn plus: a live tag after the readout ("≈ $0.42 / run").
    #[props(default)]
    tag: Option<String>,
    /// Review sheets: show the bubble over thumb `i`.
    #[props(default)]
    active: Option<usize>,
    /// Value changed.
    #[props(default)]
    on_change: EventHandler<Vec<f64>>,
    /// Committed (released / key up).
    #[props(default)]
    on_commit: EventHandler<Vec<f64>>,
) -> Element {
    let mut m = use_signal(|| slider.clone());
    let mut len = use_signal(|| 0.0f32);
    let mut root = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut origin = use_signal(|| 0.0f64);
    let s = m.read().clone();
    let fmt = |v: f64| format.replace('#', &format!("{v:.decimals$}"));
    let (a, b) = s.fill();
    let vertical = s.vertical;
    let readout: Vec<String> = (0..s.values.len()).map(|i| s.value_text(i, fmt)).collect();
    let marks: Vec<Mark> = s.marks.clone();
    let pos = |f: f64| {
        if vertical {
            format!("bottom:{:.2}%", f * 100.0)
        } else {
            format!("left:{:.2}%", f * 100.0)
        }
    };
    let fill_style = if vertical {
        format!("bottom:{:.2}%;height:{:.2}%", a * 100.0, (b - a) * 100.0)
    } else {
        format!("left:{:.2}%;width:{:.2}%", a * 100.0, (b - a) * 100.0)
    };
    let drag_on = s.active.is_some() || active.is_some();
    let measure = move || {
        if let Some(r) = root.peek().clone() {
            spawn(async move {
                if let Ok(rc) = r.get_client_rect().await {
                    let (o, l) = if vertical {
                        (rc.origin.y, rc.size.height)
                    } else {
                        (rc.origin.x, rc.size.width)
                    };
                    origin.set(o);
                    len.set(l as f32);
                }
            });
        }
    };
    rsx! {
        div { class: cls(&["oxyn-slider", if vertical { "vertical" } else { "" }, if s.disabled { "disabled" } else { "" }, if drag_on { "dragging" } else { "" }]),
            if show_value || tag.is_some() {
                div { class: "head",
                    span { class: "lb", "{label}" }
                    span { class: "sp" }
                    if show_value { span { class: "val", "{readout.join(\" – \")}" } }
                    if let Some(t) = tag.clone() { span { class: "tag", "{t}" } }
                }
            }
            div { class: "track",
                onmounted: move |e| { root.set(Some(e.data())); measure(); },
                onpointerdown: move |e: PointerEvent| {
                    measure();
                    let p = e.client_coordinates();
                    let x = if vertical { (p.y - *origin.peek()) as f32 } else { (p.x - *origin.peek()) as f32 };
                    let l = *len.peek();
                    if l > 0.0 { m.write().press(if vertical { l - x } else { x }, l); on_change.call(m.peek().values.clone()); }
                },
                onpointermove: move |e: PointerEvent| if m.peek().active.is_some() {
                    let p = e.client_coordinates();
                    let x = if vertical { (p.y - *origin.peek()) as f32 } else { (p.x - *origin.peek()) as f32 };
                    let l = *len.peek();
                    if l > 0.0 && m.write().drag(if vertical { l - x } else { x }, l) { on_change.call(m.peek().values.clone()); }
                },
                onpointerup: move |_| { let v = m.write().release(); on_commit.call(v) },
                onpointerleave: move |_| if m.peek().active.is_some() { let v = m.write().release(); on_commit.call(v) },
                span { class: "rail" }
                span { class: "fill", style: "{fill_style}" }
                for mk in marks.iter() {
                    span { class: if mk.detent { "mark detent" } else { "mark" }, style: "{pos(s.fraction(mk.value))}", "aria-hidden": "true",
                        if !mk.label.is_empty() { span { class: "ml", "{mk.label}" } }
                    }
                }
                for (i, v) in s.values.iter().enumerate() {
                    {
                        let f = s.fraction(*v);
                        let name = thumb_labels.get(i).cloned().unwrap_or(label.clone());
                        let text = readout[i].clone();
                        let show_bubble = active == Some(i) || s.active == Some(i);
                        rsx! {
                            span { class: cls(&["thumb", if show_bubble { "active" } else { "" }]), style: "{pos(f)}",
                                role: "slider", tabindex: if s.disabled { "-1" } else { "0" },
                                "aria-label": "{name}", "aria-valuemin": "{s.min}", "aria-valuemax": "{s.max}", "aria-valuenow": "{v}",
                                "aria-valuetext": "{text}", "aria-orientation": if vertical { "vertical" } else { "horizontal" },
                                "aria-disabled": "{s.disabled}",
                                onkeydown: move |e: KeyboardEvent| {
                                    let shift = e.modifiers().shift();
                                    let k = match e.key() {
                                        Key::ArrowRight | Key::ArrowUp => Some(if shift { SliderKey::IncLarge } else { SliderKey::Inc }),
                                        Key::ArrowLeft | Key::ArrowDown => Some(if shift { SliderKey::DecLarge } else { SliderKey::Dec }),
                                        Key::PageUp => Some(SliderKey::IncLarge), Key::PageDown => Some(SliderKey::DecLarge),
                                        Key::Home => Some(SliderKey::Min), Key::End => Some(SliderKey::Max),
                                        _ => None,
                                    };
                                    if let Some(k) = k { e.prevent_default(); if m.write().key(i, k) { let v = m.peek().values.clone(); on_change.call(v.clone()); on_commit.call(v) } }
                                },
                                span { class: "bubble", "aria-hidden": "true", "{text}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Text field: label, hint and error in one slot (nothing below moves), prefix and suffix, clear,
/// show-password and copy buttons, a character count. No focus ring: the grey edge darkens on
/// hover and a little more on focus. Oxyn plus: validation timing, soft limit, a secret field
/// whose reveal hides itself again.
#[component]
pub fn TextField(
    /// Label.
    label: String,
    /// Initial value.
    #[props(default)]
    value: String,
    /// `text` · `password` · `email` · `search` · `number` · `url`.
    #[props(default = "text".to_string())]
    kind: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// Hint.
    #[props(default)]
    hint: Option<String>,
    /// Error from the app.
    #[props(default)]
    error: Option<String>,
    /// Prefix text or icon name (`icon:search`).
    #[props(default)]
    prefix: Option<String>,
    /// Suffix text.
    #[props(default)]
    suffix: Option<String>,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// "Optional" next to the label.
    #[props(default)]
    optional: bool,
    /// Required.
    #[props(default)]
    required: bool,
    /// Clear button.
    #[props(default)]
    clearable: bool,
    /// Copy button.
    #[props(default)]
    copyable: bool,
    /// Character counter (needs `max`).
    #[props(default)]
    counter: bool,
    /// Hard limit.
    #[props(default)]
    max: Option<usize>,
    /// Soft limit (warning).
    #[props(default)]
    soft: Option<usize>,
    /// `blur` · `type` · `submit`.
    #[props(default = "blur".to_string())]
    validate: String,
    /// Hide a revealed password after this many seconds.
    #[props(default)]
    auto_hide_s: Option<u64>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Review sheets: force `hover` or `focus`.
    #[props(default)]
    state: Option<String>,
    /// Clock.
    #[props(default)]
    now: ClockMs,
    /// Value changed.
    #[props(default)]
    on_input: EventHandler<String>,
    /// Cleared.
    #[props(default)]
    on_clear: EventHandler<()>,
    /// Copied.
    #[props(default)]
    on_copy: EventHandler<String>,
) -> Element {
    let ft = match kind.as_str() {
        "password" => FieldType::Password,
        "email" => FieldType::Email,
        "search" => FieldType::Search,
        "number" => FieldType::Number,
        "url" => FieldType::Url,
        _ => FieldType::Text,
    };
    let mut f = use_signal(|| {
        let mut t = FieldModel::new(ft);
        t.value = value.clone();
        t.max = max;
        t.soft = soft;
        t.required = required;
        t.error = error.clone();
        t.disabled = disabled;
        t.auto_hide = auto_hide_s.map(std::time::Duration::from_secs);
        t.validate = match validate.as_str() {
            "type" => Validate::OnType,
            "submit" => Validate::OnSubmit,
            _ => Validate::OnBlur,
        };
        if error.is_some() {
            t.submit(ClockMs(0));
        }
        t
    });
    let mut copied = use_signal(|| false);
    let s = f.read().clone();
    let id = uid("tf", &label);
    let err = s.error().map(str::to_string);
    let count = if counter { s.counter() } else { None };
    let pw = ft == FieldType::Password;
    let left = s.reveal_left(now);
    rsx! {
        div { class: cls(&["oxyn-field", &size, if err.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }, state.as_deref().unwrap_or("")]),
            label { class: "flabel", r#for: "{id}", "{label}" if optional { span { class: "opt", "Optional" } } }
            div { class: "control", key: "{err.is_some()}",
                if let Some(p) = prefix.clone() {
                    if let Some(i) = p.strip_prefix("icon:") { span { class: "affix", Icon { name: i.to_string(), size: 14.0 } } } else { span { class: "affix", "{p}" } }
                }
                input { id: "{id}", class: if s.revealed { "in revealed" } else { "in" }, r#type: "{ft.attr(s.revealed)}", value: "{s.value}", disabled,
                    placeholder: if let Some(p) = &placeholder { "{p}" },
                    maxlength: if let (Some(m), false) = (max, soft.is_some()) { "{m}" },
                    "aria-invalid": "{err.is_some()}", "aria-required": "{required}", "aria-describedby": "{id}-m",
                    oninput: move |e| { f.write().input(&e.value(), now); on_input.call(e.value()) },
                    onfocus: move |_| f.write().focus(),
                    onblur: move |_| f.write().blur(now),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && (clearable || ft == FieldType::Search) && f.write().clear(now) { on_clear.call(()) },
                }
                if let Some(sx) = suffix.clone() { span { class: "affix", "{sx}" } }
                if clearable && s.can_clear() {
                    button { class: "act", r#type: "button", "aria-label": "Clear {label}", onclick: move |_| if f.write().clear(now) { on_clear.call(()) },
                        Icon { name: "x", size: 12.0, stroke: 2.0 }
                    }
                }
                if pw {
                    button { class: "act eye", r#type: "button", "aria-label": if s.revealed { "Hide password" } else { "Show password" }, "aria-pressed": "{s.revealed}",
                        style: if let Some(l) = left { format!("--left:{:.3}", l) } else { String::new() },
                        onclick: move |_| f.write().toggle_reveal(now),
                        Icon { name: if s.revealed { "eye-off" } else { "eye" }, size: 14.0 }
                    }
                }
                if copyable {
                    button { class: if *copied.read() || s.showing_copied(now) { "act copy done" } else { "act copy" }, r#type: "button", "aria-label": "Copy {label}",
                        onclick: move |_| { let v = f.write().copied(now); copied.set(true); on_copy.call(v) },
                        onanimationend: move |_| copied.set(false),
                        if *copied.read() || s.showing_copied(now) { Icon { name: "check", size: 13.0, stroke: 2.0 } } else { Icon { name: "copy", size: 13.0 } }
                    }
                }
            }
            div { class: "foot",
                div { id: "{id}-m", class: if err.is_some() { "msg err" } else { "msg" }, "aria-live": "polite",
                    if let Some(e) = err.clone() { span { key: "{e}", "{e}" } } else if let Some(h) = hint.clone() { "{h}" }
                }
                if let Some((t, lvl)) = count {
                    span { class: match lvl { CountLevel::Ok => "count", CountLevel::Near => "count near", CountLevel::Over => "count over" }, "aria-live": "polite", "{t}" }
                }
            }
        }
    }
}
