//! Form fields (plan #25): `TagInput`, `Combobox`, `MultiSelect`, `OtpInput`, `PasswordField`,
//! `PasswordStrength`, `InlineEdit`, `PhoneInput`.
//!
//! Logic lives in `oxyn_core::primitives::{tags, combobox, otp, password, inline_edit, phone}`;
//! these components hold the model in a signal, map DOM events onto it and draw with
//! `PRIMITIVES_CSS`. Timers (an error clearing, a save landing) run on the shared `oxyn-timer`
//! animation so they need no clock.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        combobox::{
            ComboEvent, ComboKey, ComboOption, ComboRow, Combobox as ComboModel, MatchMode,
        },
        inline_edit::{EditAction, EditKey, EditPhase, EditVariant, InlineEdit as EditModel},
        otp::{Otp, OtpKey, OtpStatus},
        password::{Level, Policy, caps_lock_from, evaluate, segment_delay},
        phone::{COUNTRIES, CountryRow, PhoneInput as PhoneModel, PhoneStatus},
        tags::{Normalize, Overflow, TagEvent, TagInput as TagModel, TagKey, TagRule},
    },
};

use super::{buttons::Spin, cls};
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

/// Splits `label` into `(text, matched)` runs for highlighted matches.
fn runs(label: &str, ranges: &[(usize, usize)]) -> Vec<(String, bool)> {
    let chars: Vec<char> = label.chars().collect();
    let mut out = vec![];
    let mut at = 0;
    for &(a, b) in ranges {
        if a > at {
            out.push((chars[at..a].iter().collect(), false));
        }
        out.push((chars[a..b.min(chars.len())].iter().collect(), true));
        at = b;
    }
    if at < chars.len() {
        out.push((chars[at..].iter().collect(), false));
    }
    out
}

/// Initials avatar.
#[component]
fn Initials(text: String) -> Element {
    rsx! { span { class: "oxyn-ini", "aria-hidden": "true", "{text}" } }
}

// ---------------------------------------------------------------------------------------------
// Tag input

/// Tag input: several values in one field as chips. Enter, comma or Tab add; paste a whole list;
/// a check per value with its message in place; a duplicate flashes the chip it repeats;
/// Backspace marks then removes the last chip; ← → walk the chips. Oxyn plus: typed chips
/// (person / file / tool / label icons), suggestions, and a `+N` fold past a count.
#[component]
pub fn TagInput(
    /// Label.
    label: String,
    /// Starting chips.
    #[props(default)]
    value: Vec<String>,
    /// Placeholder while empty (`Add more…` after the first chip).
    #[props(default = "Add…".to_string())]
    placeholder: String,
    /// Hint (a message takes its place while there is one).
    #[props(default)]
    hint: Option<String>,
    /// Error from the app.
    #[props(default)]
    error: Option<String>,
    /// "Optional" next to the label.
    #[props(default)]
    optional: bool,
    /// Check: `any` · `email` · `domain` · `max:24`.
    #[props(default = "any".to_string())]
    validate: String,
    /// Clean: `trim` · `lower` · `slug`.
    #[props(default = "trim".to_string())]
    normalize: String,
    /// Also split on spaces.
    #[props(default)]
    split_on_space: bool,
    /// Max chips (shows `3/10`).
    #[props(default)]
    max: Option<usize>,
    /// Initials circle on every chip (people).
    #[props(default)]
    avatar: bool,
    /// Oxyn plus: icon per chip kind (person, file, tool, label).
    #[props(default)]
    typed: bool,
    /// Oxyn plus: suggestions offered as you type.
    #[props(default)]
    suggestions: Vec<String>,
    /// Oxyn plus: fold chips past this count into `+N`.
    #[props(default)]
    fold: Option<usize>,
    /// Form field name (one hidden input per chip).
    #[props(default)]
    name: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let v: Vec<&str> = value.iter().map(String::as_str).collect();
        let mut t = TagModel::new(&v);
        t.max = max;
        t.split_on_space = split_on_space;
        t.normalize = Normalize::from_id(&normalize);
        t.rule = TagRule::from_id(&validate);
        t.disabled = disabled;
        t.suggestions = suggestions.clone();
        t.overflow = fold.map_or(Overflow::Wrap, Overflow::Fold);
        t
    });
    let mut flash_n = use_signal(|| 0u32);
    let id = uid("tag", &label);
    let st = m.read().clone();
    let msg = error.clone().or(st.message.clone());
    let (shown, extra) = st.shown();
    let mut apply = move |ev: TagEvent| match ev {
        TagEvent::Added(_) | TagEvent::Removed(..) => on_change.call(m.peek().values()),
        TagEvent::Duplicate(_) => flash_n += 1,
        _ => {}
    };
    let matches: Vec<String> = st.matches().into_iter().map(str::to_string).collect();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-tags", if msg.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: "flabel", r#for: "{id}", "{label}" if optional { span { class: "opt", "Optional" } } }
            div { class: "control box",
                onclick: move |_| m.write().picked = None,
                for (i, t) in st.tags.iter().enumerate().take(shown) {
                    {
                        let picked = st.picked == Some(i);
                        let text = t.text.clone();
                        let flashing = st.flash.is_some_and(|f| f.0 == i);
                        let kind = t.kind;
                        rsx! {
                            span { key: "{text}-{i}", class: cls(&["chip", kind.id(), if picked { "picked" } else { "" }, if flashing { "flash" } else { "" }]),
                                "data-n": if flashing { "{flash_n}" },
                                title: "{text}",
                                if avatar { Initials { text: text.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default() } }
                                if typed { if let Some(ic) = kind.icon() { span { class: "ki", Icon { name: ic, size: 12.0 } } } }
                                span { class: "ct", "{text}" }
                                if !disabled {
                                    button { class: "rm", r#type: "button", tabindex: "-1", "aria-label": "Remove {text}",
                                        onclick: move |e| { e.stop_propagation(); let ev = m.write().remove(i); apply(ev) },
                                        Icon { name: "x", size: 10.0, stroke: 2.0 }
                                    }
                                }
                            }
                        }
                    }
                }
                if extra > 0 { span { class: "chip more", key: "more-{extra}", "+{extra}" } }
                input { id: "{id}", class: "in", value: "{st.draft}", disabled,
                    placeholder: "{st.placeholder(&placeholder)}",
                    "aria-invalid": "{msg.is_some()}", "aria-describedby": "{id}-m",
                    autocomplete: "off",
                    oninput: move |e: FormEvent| {
                        let v = e.value();
                        if v.contains([',', '\n', ';']) {
                            let ev = { let mut w = m.write(); w.draft.clear(); w.paste(&v, ClockMs(0)) };
                            apply(ev);
                        } else {
                            m.write().input(&v);
                        }
                    },
                    onblur: move |_| { if !m.peek().draft.trim().is_empty() { let ev = m.write().commit(ClockMs(0)); apply(ev) } },
                    onkeydown: move |e: KeyboardEvent| {
                        let empty = m.peek().draft.is_empty();
                        let k = match e.key() {
                            Key::Enter => Some(TagKey::Enter),
                            Key::Tab if !empty => Some(TagKey::Tab),
                            Key::Backspace => Some(TagKey::Backspace),
                            Key::Delete => Some(TagKey::Delete),
                            Key::ArrowLeft if empty => Some(TagKey::Left),
                            Key::ArrowRight if empty => Some(TagKey::Right),
                            Key::Escape => Some(TagKey::Escape),
                            Key::Character(c) if c == " " && m.peek().split_on_space => Some(TagKey::Space),
                            _ => None,
                        };
                        if let Some(k) = k {
                            let has_pick = m.peek().picked.is_some();
                            let swallow = matches!(k, TagKey::Enter | TagKey::Space) || (matches!(k, TagKey::Backspace | TagKey::Delete) && (empty || has_pick));
                            if swallow { e.prevent_default(); }
                            let ev = m.write().key(k, empty, ClockMs(0));
                            apply(ev);
                        }
                    },
                }
            }
            if !matches.is_empty() {
                div { class: "sugg", role: "listbox", "aria-label": "Suggestions",
                    for s in matches {
                        button { class: "sg", r#type: "button", role: "option", "aria-selected": "false",
                            onmousedown: move |e| e.prevent_default(),
                            onclick: { let s = s.clone(); move |_| { let ev = m.write().accept(&s, ClockMs(0)); apply(ev) } },
                            Icon { name: "spark", size: 11.0 } "{s}"
                        }
                    }
                }
            }
            div { class: "foot",
                div { id: "{id}-m", class: if msg.is_some() { "msg err" } else { "msg" },
                    if let Some(e) = msg.clone() { span { key: "{e}", "{e}" } } else if let Some(h) = hint.clone() { "{h}" }
                }
                if let Some(c) = st.counter() { span { class: if st.full() { "count near" } else { "count" }, "{c}" } }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
            if let Some(n) = name { for v in st.values() { input { r#type: "hidden", name: "{n}", value: "{v}" } } }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Combobox and multi-select

fn combo_key(k: &Key, searchable: bool) -> Option<ComboKey> {
    Some(match k {
        Key::ArrowDown => ComboKey::Down,
        Key::ArrowUp => ComboKey::Up,
        Key::Home => ComboKey::Home,
        Key::End => ComboKey::End,
        Key::Enter => ComboKey::Enter,
        Key::Escape => ComboKey::Escape,
        Key::Tab => ComboKey::Tab,
        Key::Backspace => ComboKey::Backspace,
        Key::Character(c) if c == " " && !searchable => ComboKey::Space,
        _ => return None,
    })
}

#[component]
fn ComboList(
    m: Signal<ComboModel>,
    id: String,
    on_change: EventHandler<Vec<String>>,
    on_create: EventHandler<String>,
) -> Element {
    let st = m.read().clone();
    let rows = st.rows();
    rsx! {
        div { class: "list", id: "{id}-list", role: "listbox", "aria-multiselectable": "{st.multiple}",
            onmousedown: move |e| e.prevent_default(),
            for (ri, r) in rows.iter().enumerate() {
                {
                    let active = st.active == Some(ri);
                    match r.clone() {
                        ComboRow::Group(g) => rsx! { div { class: "grp", role: "presentation", "{g}" } },
                        ComboRow::Loading => rsx! { div { class: "row skel", "aria-hidden": "true", span {} } div { class: "row skel", "aria-hidden": "true", span {} } },
                        ComboRow::Empty => rsx! { div { class: "empty", role: "status", "{st.empty_text}" } },
                        ComboRow::Create(q) => rsx! {
                            div { id: "{id}-r{ri}", class: cls(&["row create", if active { "active" } else { "" }]), role: "option", "aria-selected": "false",
                                onpointerenter: move |_| m.write().hover(ri),
                                onclick: move |_| { let r = m.write().create(ClockMs(0)); if let ComboEvent::Created(t) = r { on_create.call(t); on_change.call(m.peek().value.clone()) } },
                                Icon { name: "plus", size: 13.0 } span { class: "lb", "{st.create_text(&q)}" }
                            }
                        },
                        ComboRow::SelectAll(n) => {
                            let all = rows.iter().filter_map(|x| if let ComboRow::Option(i, _) = x { Some(*i) } else { None }).all(|i| st.is_selected(i));
                            rsx! {
                                div { id: "{id}-r{ri}", class: cls(&["row all", if active { "active" } else { "" }]), role: "option", "aria-selected": "{all}",
                                    onpointerenter: move |_| m.write().hover(ri),
                                    onclick: move |_| { m.write().toggle_all(ClockMs(0)); on_change.call(m.peek().value.clone()) },
                                    span { class: cls(&["cb", if all { "on" } else { "" }]), svg { view_box: "0 0 16 16", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } }
                                    span { class: "lb", if all { "Clear all {n}" } else { "Select all {n}" } }
                                }
                            }
                        }
                        ComboRow::Option(i, rg) => {
                            let o = st.options[i].clone();
                            let on = st.is_selected(i);
                            rsx! {
                                div { id: "{id}-r{ri}", class: cls(&["row", if active { "active" } else { "" }, if o.disabled { "disabled" } else { "" }]), role: "option",
                                    "aria-selected": "{on}", "aria-disabled": "{o.disabled}",
                                    onpointerenter: move |_| m.write().hover(ri),
                                    onclick: move |_| { let r = m.write().toggle(i, ClockMs(0)); if let ComboEvent::Changed(v) = r { on_change.call(v) } },
                                    if st.multiple { span { class: cls(&["cb", if on { "on" } else { "" }]), svg { view_box: "0 0 16 16", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                                    if o.avatar { Initials { text: o.initials() } }
                                    if let Some(ic) = o.icon.clone() { span { class: "oi", Icon { name: ic, size: 14.0 } } }
                                    span { class: "txt",
                                        span { class: "lb",
                                            for (t, hit) in runs(&o.label, &rg) { if hit { mark { "{t}" } } else { "{t}" } }
                                        }
                                        if let Some(h) = o.hint.clone() { span { class: "ds", "{h}" } }
                                    }
                                    if !st.multiple && on { span { class: "tick", svg { view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Combobox: pick one or many from a searchable, grouped list — chips with avatars, highlighted
/// matches, create-on-the-fly, a `+N` chip past `max_visible_tags`, full combobox keyboard.
/// Oxyn plus: match mode (contains / prefix / fuzzy), a max selection with a reason, Select all
/// within the filter, and an async source with a debounce window.
#[component]
pub fn Combobox(
    /// Options.
    options: Vec<ComboOption>,
    /// Starting values.
    #[props(default)]
    value: Vec<String>,
    /// Pick many (chips) or one.
    #[props(default = true)]
    multiple: bool,
    /// Placeholder.
    #[props(default = "Search…".to_string())]
    placeholder: String,
    /// Offer a create row when nothing matches exactly.
    #[props(default)]
    creatable: bool,
    /// Create row text; `…` becomes the typed text.
    #[props(default = "Create “…”".to_string())]
    create_label: String,
    /// Show at most this many chips, the rest as `+N`.
    #[props(default)]
    max_visible_tags: Option<usize>,
    /// Visible label (also names the list).
    #[props(default)]
    label: Option<String>,
    /// Open right away.
    #[props(default)]
    default_open: bool,
    /// Shown when nothing matches.
    #[props(default = "No results".to_string())]
    empty_text: String,
    /// `contains` · `prefix` · `fuzzy`.
    #[props(default = "contains".to_string())]
    match_mode: String,
    /// Max picks.
    #[props(default)]
    max_selected: Option<usize>,
    /// Select all row.
    #[props(default)]
    select_all: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
    /// Created an option.
    #[props(default)]
    on_create: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let v: Vec<&str> = value.iter().map(String::as_str).collect();
        let mut c = ComboModel::new(options.clone(), &v);
        c.multiple = multiple;
        c.creatable = creatable;
        c.create_label = create_label.clone();
        c.max_visible = max_visible_tags;
        c.empty_text = empty_text.clone();
        c.mode = MatchMode::from_id(&match_mode);
        c.max_selected = max_selected.map(|n| (n, format!("Pick up to {n}")));
        c.select_all = select_all;
        c.disabled = disabled;
        if default_open {
            c.open_list();
        }
        c
    });
    let id = uid("cbx", label.as_deref().unwrap_or(&placeholder));
    let st = m.read().clone();
    let (chips, extra) = st.chips();
    let single_label = (!st.multiple)
        .then(|| st.selected().first().map(|o| o.label.clone()))
        .flatten();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-combo", if st.open { "open" } else { "" }, if disabled { "disabled" } else { "" }]),
            if let Some(l) = label.clone() { label { class: "flabel", id: "{id}-l", r#for: "{id}", "{l}" } }
            div { class: "anchor",
                div { class: "control box", onclick: move |_| { if !m.peek().open { m.write().open_list() } },
                    if st.multiple {
                        for (ci, o) in chips.iter().enumerate() {
                            {
                                let v = o.value.clone();
                                let marked = st.chip_marked == Some(ci);
                                rsx! {
                                    span { key: "{v}", class: cls(&["chip", if marked { "picked" } else { "" }]),
                                        if o.avatar { Initials { text: o.initials() } }
                                        span { class: "ct", "{o.label}" }
                                        button { class: "rm", r#type: "button", tabindex: "-1", "aria-label": "Remove {o.label}",
                                            onmousedown: move |e| e.prevent_default(),
                                            onclick: move |e| { e.stop_propagation(); let r = m.write().remove(&v, ClockMs(0)); if let ComboEvent::Changed(x) = r { on_change.call(x) } },
                                            Icon { name: "x", size: 10.0, stroke: 2.0 }
                                        }
                                    }
                                }
                            }
                        }
                        if extra > 0 { span { class: "chip more roll", key: "more-{extra}", "+{extra}" } }
                    }
                    input { id: "{id}", class: "in", role: "combobox", value: "{st.query}", disabled, autocomplete: "off",
                        placeholder: if st.value.is_empty() || !st.multiple { single_label.clone().unwrap_or(placeholder.clone()) } else { String::new() },
                        "aria-expanded": "{st.open}", "aria-controls": "{id}-list", "aria-autocomplete": "list",
                        "aria-labelledby": if label.is_some() { "{id}-l" },
                        "aria-activedescendant": if let (true, Some(a)) = (st.open, st.active) { "{id}-r{a}" },
                        oninput: move |e: FormEvent| m.write().input(&e.value(), ClockMs(0)),
                        onblur: move |_| m.write().close(),
                        onkeydown: move |e: KeyboardEvent| {
                            if let Some(k) = combo_key(&e.key(), true) {
                                if matches!(k, ComboKey::Enter | ComboKey::Up | ComboKey::Down) || (k == ComboKey::Escape && m.peek().open) { e.prevent_default(); }
                                let ev = m.write().key(k, ClockMs(0));
                                match ev {
                                    ComboEvent::Changed(v) => on_change.call(v),
                                    ComboEvent::Created(t) => { on_create.call(t); on_change.call(m.peek().value.clone()) }
                                    ComboEvent::None => {}
                                }
                            }
                        },
                    }
                    span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
                }
                if st.open { ComboList { m, id: id.clone(), on_change, on_create } }
            }
            div { class: "foot", if st.multiple && !st.value.is_empty() { span { class: "count", key: "{st.value.len()}", "{st.value.len()} selected" } } }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}

/// Multi-select: several values in a compact field — chips with a rolling `+N`, a listbox with
/// drawn checks, a Clear button. Oxyn plus: Select all, a max with a reason, a searchable mode.
#[component]
pub fn MultiSelect(
    /// Label (also names the listbox).
    label: String,
    /// Options.
    options: Vec<ComboOption>,
    /// Starting values.
    #[props(default)]
    value: Vec<String>,
    /// Placeholder.
    #[props(default = "Select options".to_string())]
    placeholder: String,
    /// Helper copy.
    #[props(default)]
    description: Option<String>,
    /// Chips before `+N`.
    #[props(default = 2)]
    max_visible: usize,
    /// Select all row.
    #[props(default)]
    select_all: bool,
    /// Max picks.
    #[props(default)]
    max_selected: Option<usize>,
    /// Start open (review sheets).
    #[props(default)]
    open: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let v: Vec<&str> = value.iter().map(String::as_str).collect();
        let mut c = ComboModel::multi_select(options.clone(), &v);
        c.max_visible = Some(max_visible);
        c.select_all = select_all;
        c.max_selected = max_selected.map(|n| (n, format!("Pick up to {n}")));
        c.disabled = disabled;
        if open {
            c.open_list();
        }
        c
    });
    let id = uid("ms", &label);
    let st = m.read().clone();
    let (chips, extra) = st.chips();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-combo oxyn-multi", if st.open { "open" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: "flabel", id: "{id}-l", "{label}" }
            div { class: "anchor",
                div { class: "control trig",
                    button { id: "{id}", class: "tb", r#type: "button", disabled,
                        "aria-haspopup": "listbox", "aria-expanded": "{st.open}", "aria-controls": "{id}-list",
                        "aria-labelledby": "{id}-l {id}-v",
                        "aria-activedescendant": if let (true, Some(a)) = (st.open, st.active) { "{id}-r{a}" },
                        onclick: move |_| { let o = m.peek().open; if o { m.write().close() } else { m.write().open_list() } },
                        onblur: move |_| m.write().close(),
                        onkeydown: move |e: KeyboardEvent| {
                            if let Some(k) = combo_key(&e.key(), false) {
                                if k != ComboKey::Tab { e.prevent_default(); }
                                let r = m.write().key(k, ClockMs(0)); if let ComboEvent::Changed(v) = r { on_change.call(v) }
                            }
                        },
                        span { class: "sr", id: "{id}-v", "{st.summary(&placeholder)}" }
                        span { class: "chips", "aria-hidden": "true",
                            if chips.is_empty() { span { class: "ph", "{placeholder}" } }
                            for o in chips.iter() { span { key: "{o.value}", class: "chip grow", span { class: "ct", "{o.label}" } } }
                            if extra > 0 { span { class: "chip more roll", key: "more-{extra}", "+{extra}" } }
                        }
                    }
                    if !st.value.is_empty() && !disabled {
                        button { class: "act", r#type: "button", "aria-label": "Clear selections",
                            onclick: move |_| { let r = m.write().clear(ClockMs(0)); if let ComboEvent::Changed(v) = r { on_change.call(v) } },
                            Icon { name: "x", size: 12.0, stroke: 2.0 }
                        }
                    }
                    span { class: "chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
                }
                if st.open { ComboList { m, id: id.clone(), on_change, on_create: move |_| {} } }
            }
            if let Some(d) = description { div { class: "foot", div { class: "msg", "{d}" } } }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// OTP

/// OTP input: the verification-code field. One real input carries the whole code; a single
/// focus ring glides from box to box; paste and SMS autofill fill every box; verifying shimmers
/// and locks; an error shakes and clears the code; success morphs into a `Verified` pill; a resend
/// countdown becomes a Resend button. Oxyn plus: lengths 4–8 with letters, masked display, an
/// attempt counter.
#[component]
pub fn OtpInput(
    /// Number of boxes (4–8).
    #[props(default = 6)]
    length: usize,
    /// A dash after this many boxes (`0` = the middle).
    #[props(default)]
    separator: Option<usize>,
    /// Digits only (else letters too, in capitals).
    #[props(default = true)]
    numeric: bool,
    /// Starting code.
    #[props(default)]
    value: String,
    /// `idle` · `verifying` · `error` · `success`.
    #[props(default = "idle".to_string())]
    status: String,
    /// Shown after an error.
    #[props(default = "That code didn't work. Try again.".to_string())]
    error_message: String,
    /// Clear the boxes after the shake.
    #[props(default = true)]
    clear_on_error: bool,
    /// Text in the success pill.
    #[props(default = "Verified".to_string())]
    success_label: String,
    /// Seconds until resend (`None` hides resend).
    #[props(default = Some(30))]
    resend_in: Option<u32>,
    /// `start` · `center`.
    #[props(default = "start".to_string())]
    align: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Focus on mount.
    #[props(default)]
    auto_focus: bool,
    /// Accessible name (the number of digits is added).
    #[props(default = "Verification code".to_string())]
    label: String,
    /// Oxyn plus: dots instead of characters.
    #[props(default)]
    masked: bool,
    /// Oxyn plus: tries left.
    #[props(default)]
    attempts: Option<u32>,
    /// Oxyn plus (demos): verify locally against this code.
    #[props(default)]
    expect: Option<String>,
    /// Every change.
    #[props(default)]
    on_change: EventHandler<String>,
    /// The last box was filled.
    #[props(default)]
    on_complete: EventHandler<String>,
    /// Resend pressed.
    #[props(default)]
    on_resend: EventHandler<()>,
) -> Element {
    let mut m = use_signal(|| {
        let mut o = Otp::new(length);
        o.separator = separator;
        o.numeric = numeric;
        o.clear_on_error = clear_on_error;
        o.resend_in = resend_in;
        o.masked = masked;
        o.attempts = attempts;
        o.disabled = disabled;
        if !value.is_empty() {
            o.paste(&value, ClockMs(0));
        }
        o.status = OtpStatus::from_id(&status);
        o
    });
    let mut focused = use_signal(|| auto_focus);
    let mut left = use_signal(|| resend_in.unwrap_or(0));
    let mut shake_n = use_signal(|| 0u32);
    let mut verdict = use_signal(|| None::<bool>);
    let id = uid("otp", &label);
    let st = m.read().clone();
    let s = st.status;
    let sep = st.separator_at();
    let has_expect = expect.is_some();
    let mut check = move |code: String| {
        on_complete.call(code.clone());
        if let Some(x) = expect.clone() {
            verdict.set(Some(code.eq_ignore_ascii_case(&x)));
            m.write().set_status(OtpStatus::Verifying, ClockMs(0));
        }
    };
    let verify_done = move |_| {
        let ok = verdict.peek().unwrap_or(false);
        m.write().set_status(
            if ok {
                OtpStatus::Success
            } else {
                OtpStatus::Error
            },
            ClockMs(10),
        );
        if !ok {
            shake_n += 1;
        }
    };
    rsx! {
        div { class: cls(&["oxyn-otp", s.id(), &align, if disabled { "disabled" } else { "" }, if *focused.read() { "focused" } else { "" }]),
            style: "--n:{st.length}",
            div { class: "boxes", key: "{shake_n}",
                input { id: "{id}", class: "real", value: "", disabled: st.locked(), autofocus: auto_focus,
                    inputmode: if numeric { "numeric" } else { "text" }, autocomplete: "one-time-code",
                    "aria-label": "{st.label(&label)}", "aria-invalid": "{s == OtpStatus::Error}", "aria-describedby": "{id}-m",
                    onfocus: move |_| focused.set(true),
                    onblur: move |_| focused.set(false),
                    oninput: move |e: FormEvent| {
                        let v = e.value();
                        if v.is_empty() { return; }
                        let done = if v.chars().count() > 1 { m.write().paste(&v, ClockMs(0)) } else { v.chars().next().and_then(|c| m.write().type_char(c, ClockMs(0))) };
                        on_change.call(m.peek().code());
                        if let Some(code) = done { check(code) }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        let k = match e.key() {
                            Key::Backspace => Some(OtpKey::Backspace), Key::ArrowLeft => Some(OtpKey::Left), Key::ArrowRight => Some(OtpKey::Right),
                            Key::Home => Some(OtpKey::Home), Key::End => Some(OtpKey::End), _ => None,
                        };
                        if let Some(k) = k { e.prevent_default(); m.write().key(k, ClockMs(0)); on_change.call(m.peek().code()); }
                    },
                }
                if s == OtpStatus::Success {
                    span { class: "pill", role: "status",
                        svg { view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } }
                        "{success_label}"
                    }
                } else {
                    for i in 0..st.length {
                        if sep == Some(i) { span { class: "sep", "aria-hidden": "true" } }
                        {
                            let g = st.glyph(i);
                            let active = *focused.read() && st.active == i && s == OtpStatus::Idle;
                            rsx! {
                                span { class: cls(&["cell", if g.is_some() { "filled" } else { "" }, if active { "active" } else { "" }]),
                                    style: "--i:{i}", "aria-hidden": "true",
                                    onclick: move |_| m.write().focus(i),
                                    if let Some(c) = g { span { class: "dg", key: "{c}{i}", "{c}" } } else if active { span { class: "caret" } }
                                }
                            }
                        }
                    }
                }
                if s == OtpStatus::Verifying && has_expect { span { class: "oxyn-timer", style: "animation-duration:900ms", onanimationend: verify_done } }
                if s == OtpStatus::Error && clear_on_error && st.full() {
                    span { class: "oxyn-timer", style: "animation-duration:420ms", onanimationend: move |_| { m.write().tick(ClockMs(u64::MAX / 2)); on_change.call(String::new()); } }
                }
            }
            div { id: "{id}-m", class: "below",
                if s == OtpStatus::Error {
                    span { class: "err", "{error_message}" if let Some(a) = st.attempts { {format!(" {a} {} left.", if a == 1 { "try" } else { "tries" })} } }
                } else if s != OtpStatus::Success && resend_in.is_some() {
                    if *left.read() > 0 {
                        span { class: "oxyn-timer", key: "t{left}", style: "animation-duration:1000ms", onanimationend: move |_| { let l = *left.peek(); left.set(l.saturating_sub(1)) } }
                        span { class: "rs", "Didn't get a code? Resend in " span { class: "num", "0:{left:02}" } }
                    } else {
                        span { class: "rs", "Didn't get a code? "
                            button { class: "lnk", r#type: "button", onclick: move |_| { left.set(resend_in.unwrap_or(30)); on_resend.call(()) }, "Resend" }
                        }
                    }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce(&error_message, &success_label)}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Password field and strength

#[component]
fn EyeSlash(revealed: bool) -> Element {
    rsx! {
        svg { class: cls(&["eye", if revealed { "open" } else { "" }]), view_box: "0 0 24 24", width: "15", height: "15", "aria-hidden": "true",
            path { d: "M2.5 12s3.5-6.5 9.5-6.5S21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z" }
            circle { cx: "12", cy: "12", r: "2.8" }
            path { class: "mask", d: "M4 4l16 16" }
            path { class: "slash", d: "M4 4l16 16", path_length: "1" }
        }
    }
}

/// Password field: a password input whose reveal button slashes the eye with a drawn line instead
/// of swapping icons; the value resolves through a short reveal on each change. Oxyn plus: a Caps
/// Lock warning, reveal-for-N-seconds, and a paste-from-vault button.
#[component]
pub fn PasswordField(
    /// Label.
    label: String,
    /// Helper copy (linked through aria-describedby).
    #[props(default)]
    description: Option<String>,
    /// Starting value.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// `current-password` · `new-password`.
    #[props(default = "current-password".to_string())]
    autocomplete: String,
    /// Form field name.
    #[props(default)]
    name: Option<String>,
    /// Required.
    #[props(default)]
    required: bool,
    /// Error.
    #[props(default)]
    error: Option<String>,
    /// Toggle name when hidden.
    #[props(default = "Show password".to_string())]
    show_label: String,
    /// Toggle name when shown.
    #[props(default = "Hide password".to_string())]
    hide_label: String,
    /// Oxyn plus: warn when Caps Lock is on.
    #[props(default = true)]
    caps_warning: bool,
    /// Oxyn plus: hide again after this many seconds.
    #[props(default)]
    reveal_for: Option<u32>,
    /// Oxyn plus: a "Use saved password" button (the host fills from its vault).
    #[props(default)]
    vault: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed.
    #[props(default)]
    on_input: EventHandler<String>,
    /// Vault button pressed.
    #[props(default)]
    on_vault: EventHandler<()>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let mut shown = use_signal(|| false);
    let mut toggled = use_signal(|| false);
    let mut caps = use_signal(|| false);
    let id = uid("pw", &label);
    let on = *shown.read();
    let desc = error.clone().or(description.clone());
    rsx! {
        div { class: cls(&["oxyn-field oxyn-pw", if error.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: "flabel", r#for: "{id}", "{label}" }
            div { class: "control",
                input { id: "{id}", class: cls(&["in", if *toggled.read() { "resolve" } else { "" }]), key: "{on}",
                    r#type: if on { "text" } else { "password" }, value: "{v}", disabled, required,
                    name: if let Some(n) = &name { "{n}" }, autocomplete: "{autocomplete}",
                    placeholder: if let Some(p) = &placeholder { "{p}" },
                    spellcheck: "false", "aria-invalid": "{error.is_some()}",
                    "aria-describedby": if desc.is_some() || *caps.read() { "{id}-m" },
                    oninput: move |e: FormEvent| { v.set(e.value()); on_input.call(e.value()) },
                    onkeydown: move |e: KeyboardEvent| {
                        if let Key::Character(c) = e.key() && let Some(ch) = c.chars().next() && let Some(on) = caps_lock_from(ch, e.modifiers().shift()) { caps.set(on) }
                        if e.key() == Key::CapsLock { let c = *caps.peek(); caps.set(!c) }
                    },
                    onblur: move |_| caps.set(false),
                }
                if vault && !disabled {
                    button { class: "act", r#type: "button", "aria-label": "Use saved password", title: "Use saved password", onclick: move |_| on_vault.call(()), Icon { name: "key", size: 13.0 } }
                }
                button { class: "act eye-btn", r#type: "button", disabled, "aria-pressed": "{on}", "aria-controls": "{id}",
                    "aria-label": if on { "{hide_label}" } else { "{show_label}" },
                    onclick: move |_| { shown.set(!on); toggled.set(true) },
                    EyeSlash { revealed: on }
                }
                if let (true, Some(s)) = (on, reveal_for) { span { class: "oxyn-timer", style: "animation-duration:{s}s", onanimationend: move |_| shown.set(false) } }
            }
            div { class: "foot",
                div { id: "{id}-m", class: if error.is_some() { "msg err" } else { "msg" },
                    if *caps.read() && caps_warning { span { class: "caps", Icon { name: "up", size: 11.0, stroke: 2.0 } "Caps Lock is on" } }
                    else if let Some(d) = desc.clone() { span { key: "{d}", "{d}" } }
                }
            }
        }
    }
}

/// Password strength: a new-password field with a four-segment meter that fills in a staggered
/// wave, a rule checklist with drawn ticks and remaining counts, and a strength word that morphs
/// up or down. Oxyn plus: policy presets (Basic, NIST, Enterprise), a common-password check, an
/// entropy readout and a meter-only variant.
#[component]
pub fn PasswordStrength(
    /// Label.
    label: String,
    /// Starting value.
    #[props(default)]
    value: String,
    /// `basic` · `nist` · `enterprise`.
    #[props(default = "basic".to_string())]
    policy: String,
    /// Below this many characters the meter reads `Too short`.
    #[props(default = 8)]
    short_length: usize,
    /// Error (shakes the field once per new error).
    #[props(default)]
    error: Option<String>,
    /// Start revealed.
    #[props(default)]
    revealed: bool,
    /// `checklist` (meter + rules) · `meter` (meter only).
    #[props(default = "checklist".to_string())]
    variant: String,
    /// Show the entropy estimate.
    #[props(default)]
    entropy: bool,
    /// Meter name.
    #[props(default = "Password strength".to_string())]
    meter_label: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Changed: the value and its level id.
    #[props(default)]
    on_change: EventHandler<(String, String)>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let mut shown = use_signal(|| revealed);
    let mut prev = use_signal(|| None::<Level>);
    let rules = Policy::from_id(&policy).rules();
    let pw = v.read().clone();
    let r = evaluate(&pw, &rules, short_length);
    let up = prev.read().is_none_or(|p| r.level >= p);
    let id = uid("pws", &label);
    let on = *shown.read();
    let met = r.met.iter().filter(|&&m| m).count();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-pws", r.level.id(), if error.is_some() { "invalid" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: "flabel", r#for: "{id}", "{label}" }
            div { class: "control", key: "{error.clone().unwrap_or_default()}",
                input { id: "{id}", class: "in", r#type: if on { "text" } else { "password" }, value: "{pw}", disabled,
                    autocomplete: "new-password", spellcheck: "false", "aria-invalid": "{error.is_some()}", "aria-describedby": "{id}-rules {id}-e",
                    oninput: move |e: FormEvent| {
                        let lvl = evaluate(&v.peek(), &Policy::from_id(&policy).rules(), short_length).level;
                        prev.set(Some(lvl));
                        v.set(e.value());
                        let n = evaluate(&e.value(), &Policy::from_id(&policy).rules(), short_length).level;
                        on_change.call((e.value(), n.id().to_string()));
                    },
                }
                button { class: "act eye-btn", r#type: "button", "aria-pressed": "{on}", "aria-controls": "{id}",
                    "aria-label": if on { "Hide password" } else { "Show password" },
                    onclick: move |_| shown.set(!on),
                    EyeSlash { revealed: on }
                }
            }
            div { class: "meter", role: "meter", "aria-label": "{meter_label}", "aria-valuemin": "0", "aria-valuemax": "4",
                "aria-valuenow": "{r.level.segments()}", "aria-valuetext": "{r.level.word()}",
                for i in 0..4usize {
                    span { class: if (i as u8) < r.level.segments().max(u8::from(!pw.is_empty())) && (r.level.segments() > 0 || i == 0) { "seg on" } else { "seg" },
                        style: format!("transition-delay:{}ms", segment_delay(if up { i } else { 3 - i })) }
                }
                span { class: "word", key: "{r.level.id()}", "data-dir": if up { "up" } else { "down" }, if pw.is_empty() { "" } else { "{r.level.word()}" } }
            }
            if entropy && !pw.is_empty() { div { class: "bits", "≈ {r.bits} bits" if r.common { " · common password" } } }
            if variant != "meter" {
                ul { class: "rules", id: "{id}-rules", "aria-label": "Password rules",
                    for (i, rule) in rules.iter().enumerate() {
                        li { class: if r.met[i] { "ok" } else { "" },
                            span { class: "tk", "aria-hidden": "true", svg { view_box: "0 0 16 16", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } }
                            span { class: "sr", if r.met[i] { "Met: " } else { "Not met: " } }
                            "{rule.label}"
                            if let Some(n) = r.remaining[i] { span { class: "rem", key: "{n}", " · {n} more" } }
                        }
                    }
                }
            }
            if let Some(e) = error.clone() { div { class: "msg err", id: "{id}-e", role: "alert", "{e}" } }
            span { class: "sr", role: "status", "aria-live": "polite", if pw.is_empty() { "No password" } else { "{r.level.word()}, {met} of {rules.len()} rules met" } }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Inline edit

/// Inline edit: rename in place. The text becomes a field with the same metrics, saves
/// optimistically with a drawn check, and rolls back with Retry when the save fails. Enter saves
/// (Shift+Enter breaks a line in multiline), Esc cancels, Tab saves. Oxyn plus: agent rename
/// suggestions, a conflict state when the agent changes the value mid-edit, and a simulated
/// latency / failure for demos.
#[component]
pub fn InlineEdit(
    /// The saved value.
    value: String,
    /// Accessible name (`Project name`).
    label: String,
    /// Shown when empty.
    #[props(default)]
    placeholder: String,
    /// Wraps and grows.
    #[props(default)]
    multiline: bool,
    /// `title` · `body`.
    #[props(default = "title".to_string())]
    variant: String,
    /// Max characters.
    #[props(default)]
    max: Option<usize>,
    /// Oxyn plus: rename suggestions.
    #[props(default)]
    suggestions: Vec<String>,
    /// Save latency in ms (the host's promise; 0 = instant).
    #[props(default = 600)]
    latency_ms: u32,
    /// The next save fails (demo / test of the rollback).
    #[props(default)]
    fail: bool,
    /// Oxyn plus: simulate the agent renaming it while you edit.
    #[props(default)]
    conflict: Option<String>,
    /// Persist.
    #[props(default)]
    on_save: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut e = EditModel::new(&value);
        e.multiline = multiline;
        e.variant = if variant == "body" {
            EditVariant::Body
        } else {
            EditVariant::Title
        };
        e.max = max;
        e.required = placeholder.is_empty();
        e.suggestions = suggestions.clone();
        e
    });
    let mut fails = use_signal(|| fail);
    let id = uid("ie", &label);
    let st = m.read().clone();
    let mut run = move |a: EditAction| {
        if let EditAction::Save(v) = a {
            on_save.call(v);
            if latency_ms == 0 && !*fails.peek() {
                m.write().saved(ClockMs(0));
            }
        }
    };
    let land = move |_| {
        if *fails.peek() {
            fails.set(false);
            m.write().failed("Network error", ClockMs(0));
        } else {
            m.write().saved(ClockMs(0));
        }
    };
    let editing = st.editing();
    let empty = st.value.is_empty();
    let tag = if st.variant == EditVariant::Body {
        "body"
    } else {
        "title"
    };
    rsx! {
        div { class: cls(&["oxyn-iedit", tag, st.phase.id(), if multiline { "multi" } else { "" }]),
            if editing {
                div { class: "fieldwrap",
                    if multiline {
                        textarea { id: "{id}", class: "in", value: "{st.draft}", "aria-label": "{label}", "aria-invalid": "{st.error.is_some()}", "aria-describedby": "{id}-m",
                            rows: "{st.draft.lines().count().max(1)}", autofocus: true,
                            oninput: move |e: FormEvent| m.write().input(&e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                let k = match e.key() { Key::Enter => Some(EditKey::Enter { shift: e.modifiers().shift() }), Key::Escape => Some(EditKey::Escape), _ => None };
                                if let Some(k) = k { if !matches!(k, EditKey::Enter { shift: true }) { e.prevent_default(); } let a = m.write().key(k, ClockMs(0)); run(a) }
                            },
                            onblur: move |_| { let a = m.write().key(EditKey::Tab, ClockMs(0)); run(a) },
                        }
                    } else {
                        input { id: "{id}", class: "in", value: "{st.draft}", "aria-label": "{label}", "aria-invalid": "{st.error.is_some()}", "aria-describedby": "{id}-m",
                            autofocus: true, size: "{st.draft.chars().count().max(4)}",
                            oninput: move |e: FormEvent| m.write().input(&e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                let k = match e.key() { Key::Enter => Some(EditKey::Enter { shift: false }), Key::Escape => Some(EditKey::Escape), _ => None };
                                if let Some(k) = k { e.prevent_default(); let a = m.write().key(k, ClockMs(0)); run(a) }
                            },
                            onblur: move |_| { let a = m.write().key(EditKey::Tab, ClockMs(0)); run(a) },
                        }
                    }
                    span { class: "btns",
                        button { class: "act", r#type: "button", "aria-label": "Save {label}", onmousedown: move |e| e.prevent_default(), onclick: move |_| { let a = m.write().commit(ClockMs(0)); run(a) }, Icon { name: "check", size: 13.0, stroke: 2.0 } }
                        button { class: "act", r#type: "button", "aria-label": "Cancel", onmousedown: move |e| e.prevent_default(), onclick: move |_| m.write().cancel(), Icon { name: "x", size: 12.0, stroke: 2.0 } }
                    }
                    if let (Some(c), EditPhase::Editing) = (conflict.clone(), &st.phase) { span { class: "oxyn-timer", style: "animation-duration:1500ms", onanimationend: move |_| m.write().external(&c, ClockMs(0)) } }
                }
                if !st.suggestions.is_empty() && st.phase == EditPhase::Editing {
                    div { class: "sugg",
                        span { class: "sl", Icon { name: "spark", size: 11.0 } "Suggestions" }
                        for (i, s) in st.suggestions.iter().enumerate() {
                            button { class: "sg", r#type: "button", onmousedown: move |e| e.prevent_default(), onclick: move |_| m.write().suggest(i), "{s}" }
                        }
                    }
                }
                if let EditPhase::Conflict(theirs) = &st.phase {
                    div { class: "conflict", role: "alert",
                        span { "The agent renamed this to “{theirs}”." }
                        button { class: "lnk", r#type: "button", onmousedown: move |e| e.prevent_default(), onclick: move |_| { let a = m.write().resolve(true, ClockMs(0)); run(a) }, "Keep mine" }
                        button { class: "lnk", r#type: "button", onmousedown: move |e| e.prevent_default(), onclick: move |_| { m.write().resolve(false, ClockMs(0)); }, "Use theirs" }
                    }
                }
            } else {
                button { class: cls(&["txt", if empty { "empty" } else { "" }]), r#type: "button", "aria-label": "{st.label(&label)}",
                    onclick: move |_| m.write().start(),
                    span { class: "val", key: "{st.value}", if empty { "{placeholder}" } else { "{st.value}" } }
                    span { class: "pen", "aria-hidden": "true", Icon { name: "pencil", size: 12.0 } }
                }
                match &st.phase {
                    EditPhase::Saving => rsx! {
                        span { class: "state", Spin { size: 11.0 } }
                        if latency_ms > 0 || *fails.read() { span { class: "oxyn-timer", style: "animation-duration:{latency_ms.max(1)}ms", onanimationend: land } }
                    },
                    EditPhase::Saved(_) => rsx! {
                        span { class: "state ok", svg { view_box: "0 0 16 16", width: "13", height: "13", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } }
                        span { class: "oxyn-timer", style: "animation-duration:1400ms", onanimationend: move |_| m.write().tick(ClockMs(u64::MAX / 2)) }
                    },
                    EditPhase::Failed(_) => rsx! {
                        button { class: "lnk retry", r#type: "button", onclick: move |_| { let a = m.write().retry(ClockMs(0)); run(a) }, Icon { name: "retry", size: 11.0 } "Retry" }
                    },
                    _ => rsx! {},
                }
            }
            div { id: "{id}-m", class: "msg", "aria-live": "polite",
                if let Some(e) = st.error.clone() { span { class: "err", "{e}" } }
                else if matches!(st.phase, EditPhase::Failed(_)) { span { class: "err", "{st.status()}" } }
            }
            span { class: "sr", role: "status", "{st.status()}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Phone input

/// Phone input: a country picker that grows out of the flag button (searchable, Suggested then
/// All countries), formatting as you type, a valid check, a message after blur, E.164 out; `+` in
/// the number opens a calling-code search; a pasted international number switches the country.
/// Oxyn plus: a masked display, the country from the user's locale, and an international format.
#[component]
pub fn PhoneInput(
    /// Label.
    label: String,
    /// Keep the label for screen readers only.
    #[props(default)]
    hide_label: bool,
    /// Starting number in E.164.
    #[props(default)]
    value: String,
    /// ISO code until someone picks one.
    #[props(default = "US".to_string())]
    default_country: String,
    /// Oxyn plus: pick the country from this locale (`en-GB`).
    #[props(default)]
    locale: Option<String>,
    /// Limit the picker to these ISO codes.
    #[props(default)]
    countries: Vec<String>,
    /// Pinned under Suggested.
    #[props(default = vec!["US".to_string(), "CA".to_string(), "GB".to_string()])]
    preferred: Vec<String>,
    /// Hint.
    #[props(default)]
    description: Option<String>,
    /// Replaces the built-in message.
    #[props(default)]
    error: Option<String>,
    /// Message after blur on the wrong length.
    #[props(default = true)]
    validate: bool,
    /// Oxyn plus: mask all but the last four digits when not focused.
    #[props(default)]
    masked: bool,
    /// Start with the picker open (review sheets).
    #[props(default)]
    open: bool,
    /// Required.
    #[props(default)]
    required: bool,
    /// Form field name (hidden input with E.164).
    #[props(default)]
    name: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Every edit: E.164.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Country changed.
    #[props(default)]
    on_country_change: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let iso = locale
            .as_deref()
            .and_then(oxyn_core::primitives::phone::country_from_locale)
            .map(|c| c.iso.to_string())
            .unwrap_or(default_country.clone());
        let mut p = PhoneModel::from_e164(&value, &iso);
        p.validate = validate;
        p.masked = masked;
        p.disabled = disabled;
        let leak = |v: &[String]| -> Vec<&'static str> {
            v.iter()
                .filter_map(|s| oxyn_core::primitives::phone::country(s).map(|c| c.iso))
                .collect()
        };
        p.preferred = leak(&preferred);
        if !countries.is_empty() {
            p.allowed = Some(leak(&countries));
        }
        if open {
            p.open_picker("");
        }
        p
    });
    let mut focused = use_signal(|| false);
    let id = uid("ph", &label);
    let st = m.read().clone();
    let err = error.clone().or(st.error());
    let valid = st.status() == PhoneStatus::Valid;
    let shown = if st.masked && !*focused.read() && valid {
        st.masked_text()
    } else {
        st.formatted()
    };
    let rows = st.rows();
    rsx! {
        div { class: cls(&["oxyn-field oxyn-phone", if err.is_some() { "invalid" } else { "" }, if st.open { "open" } else { "" }, if disabled { "disabled" } else { "" }]),
            label { class: if hide_label { "flabel sr" } else { "flabel" }, r#for: "{id}", "{label}" }
            div { class: "anchor",
                div { class: "control",
                    button { class: "cc", r#type: "button", disabled, "aria-label": "{st.button_label()}", "aria-haspopup": "listbox", "aria-expanded": "{st.open}",
                        onclick: move |_| { let o = m.peek().open; if o { m.write().close_picker() } else { m.write().open_picker("") } },
                        onkeydown: move |e: KeyboardEvent| match e.key() {
                            Key::ArrowDown | Key::ArrowUp => { e.prevent_default(); m.write().open_picker("") }
                            Key::Character(c) if c.chars().all(char::is_alphabetic) => { e.prevent_default(); m.write().open_picker(&c) }
                            _ => {}
                        },
                        span { class: "flag", key: "{st.country.iso}", "data-dir": if st.rolled_down { "down" } else { "up" }, "{st.country.flag()}" }
                        span { class: "code", key: "c{st.country.iso}", "+{st.country.code}" }
                        Icon { name: "chevron", size: 11.0, stroke: 2.0 }
                    }
                    input { id: "{id}", class: "in", r#type: "tel", inputmode: "tel", autocomplete: "tel", value: "{shown}", disabled, required,
                        placeholder: "{oxyn_core::primitives::phone::format_national(st.country, st.country.example)}",
                        "aria-invalid": "{err.is_some()}", "aria-describedby": "{id}-m",
                        onfocus: move |_| focused.set(true),
                        oninput: move |e: FormEvent| {
                            let changed = m.write().input(&e.value());
                            if changed { on_country_change.call(m.peek().country.iso.to_string()) }
                            on_change.call(m.peek().e164());
                        },
                        onblur: move |_| { focused.set(false); m.write().blur() },
                    }
                    if valid { span { class: "ok", "aria-hidden": "true", svg { view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } } }
                }
                if st.open {
                    div { class: "picker",
                        div { class: "filter", Icon { name: "search", size: 13.0 }
                            input { class: "q", role: "combobox", value: "{st.query}", placeholder: "Search countries or +code", autofocus: true,
                                "aria-expanded": "true", "aria-controls": "{id}-list", "aria-activedescendant": if let Some(a) = st.active { "{id}-c{a}" },
                                oninput: move |e: FormEvent| m.write().search(&e.value()),
                                onkeydown: move |e: KeyboardEvent| match e.key() {
                                    Key::ArrowDown => { e.prevent_default(); m.write().move_active(1) }
                                    Key::ArrowUp => { e.prevent_default(); m.write().move_active(-1) }
                                    Key::PageDown => { e.prevent_default(); m.write().move_active(8) }
                                    Key::PageUp => { e.prevent_default(); m.write().move_active(-8) }
                                    Key::Home if m.peek().query.is_empty() => { e.prevent_default(); m.write().move_active(-999) }
                                    Key::End if m.peek().query.is_empty() => { e.prevent_default(); m.write().move_active(999) }
                                    Key::Enter => { e.prevent_default(); let r = m.write().pick(None); if let Some(iso) = r { on_country_change.call(iso.to_string()); on_change.call(m.peek().e164()) } }
                                    Key::Escape => { e.prevent_default(); m.write().close_picker() }
                                    _ => {}
                                },
                            }
                        }
                        div { class: "list", id: "{id}-list", role: "listbox",
                            for (ri, r) in rows.iter().enumerate() {
                                match r {
                                    CountryRow::Group(g) => rsx! { div { class: "grp", role: "presentation", "{g}" } },
                                    CountryRow::Empty => rsx! { div { class: "empty", "No countries match" } },
                                    CountryRow::Country(ci) => {
                                        let c = &COUNTRIES[*ci];
                                        let on = c.iso == st.country.iso;
                                        rsx! {
                                            div { id: "{id}-c{ri}", class: cls(&["row", if st.active == Some(ri) { "active" } else { "" }]), role: "option", "aria-selected": "{on}",
                                                onmousedown: move |e| e.prevent_default(),
                                                onpointerenter: move |_| m.write().active = Some(ri),
                                                onclick: move |_| { let r = m.write().pick(Some(ri)); if let Some(iso) = r { on_country_change.call(iso.to_string()); on_change.call(m.peek().e164()) } },
                                                span { class: "flag", "{c.flag()}" }
                                                span { class: "lb", "{c.name}" }
                                                span { class: "ds", "+{c.code}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "foot",
                div { id: "{id}-m", class: if err.is_some() { "msg err" } else { "msg" }, role: if err.is_some() { "alert" } else { "" },
                    if let Some(e) = err.clone() { span { key: "{e}", "{e}" } } else if let Some(d) = description { "{d}" }
                }
            }
            if let Some(n) = name { input { r#type: "hidden", name: "{n}", value: "{st.e164()}" } }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}
