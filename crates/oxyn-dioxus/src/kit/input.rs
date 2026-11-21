//! Inputs (plan #28, chunk 4a): `TextArea` (affectUI Text Area), `LabeledInput` and
//! `LabeledTextarea` (Arc UI Input and Textarea) and `GhostText` (Arc UI Ghost text).
//! Behaviour-only rebuilds; models in `oxyn_core::kit::input`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::input::{Ghost, GhostStatus, counter, demo_suggestion, grow_rows},
};

use crate::composer::icons::Icon;

fn slug(prefix: &str, label: &str) -> String {
    let s: String = label
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_lowercase();
    format!("oxi-{prefix}-{s}")
}

/// Text area: the multi-line twin of the text field. Grows with the text from `min_rows` to
/// `max_rows` then scrolls inside, a "120/2,000" counter (red when a prefilled value is over),
/// a corner button that gives the field more room, and ⌘/Ctrl+Enter to submit (Enter alone
/// still makes a new line). The error replaces the hint and marks the field invalid.
#[component]
pub fn TextArea(
    /// Label above the field; clicking it focuses the field.
    #[props(default)]
    label: Option<String>,
    /// Initial text.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// Quiet line under the field.
    #[props(default)]
    hint: Option<String>,
    /// Error: red edge, faint red fill, aria-invalid; shown where the hint was.
    #[props(default)]
    error: Option<String>,
    /// "Optional" next to the label.
    #[props(default)]
    optional: bool,
    /// Show the counter (needs `max_length`).
    #[props(default)]
    counter_on: bool,
    /// Character limit.
    #[props(default)]
    max_length: Option<usize>,
    /// Lines when empty.
    #[props(default = 3)]
    min_rows: usize,
    /// Lines before it scrolls inside.
    #[props(default = 10)]
    max_rows: usize,
    /// The more-room button.
    #[props(default)]
    expandable: bool,
    /// Lines while expanded.
    #[props(default = 16)]
    expanded_rows: usize,
    /// Monospace text (prompts).
    #[props(default)]
    mono: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Every edit.
    #[props(default)]
    on_input: EventHandler<String>,
    /// ⌘/Ctrl+Enter.
    #[props(default)]
    on_submit: EventHandler<String>,
    /// Expanded or not.
    #[props(default)]
    on_expanded: EventHandler<bool>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let mut exp = use_signal(|| false);
    let id = slug("ta", label.as_deref().unwrap_or("text"));
    let text = v();
    let (rows, scrolls) = if exp() {
        (
            expanded_rows,
            grow_rows(&text, 56, expanded_rows, expanded_rows).1,
        )
    } else {
        grow_rows(&text, 56, min_rows, max_rows)
    };
    let count = match (counter_on, max_length) {
        (true, Some(m)) => Some(counter(text.chars().count(), m)),
        _ => None,
    };
    let over = count.as_ref().is_some_and(|c| c.1);
    let invalid = error.is_some() || over;
    let msg = error.clone().or(hint.clone());
    rsx! {
        div { class: "oxi-ta", "data-invalid": "{invalid}", "data-disabled": "{disabled}",
            if let Some(l) = &label {
                label { class: "oxi-lab", r#for: "{id}", "{l}" if optional { span { class: "oxi-opt", "Optional" } } }
            }
            div { class: "oxi-box",
                textarea { id: "{id}", class: if mono { "oxi-in mono" } else { "oxi-in" },
                    style: "--rows:{rows}", "data-scroll": "{scrolls}",
                    value: "{text}", disabled,
                    placeholder: if let Some(p) = &placeholder { "{p}" },
                    maxlength: if let (Some(m), false) = (max_length, over) { "{m}" },
                    "aria-invalid": "{invalid}", "aria-describedby": "{id}-m",
                    oninput: move |e| { v.set(e.value()); on_input.call(e.value()) },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) {
                            e.prevent_default();
                            on_submit.call(v());
                        }
                    },
                }
                if expandable {
                    button { class: "oxi-exp", r#type: "button", disabled,
                        "aria-label": if exp() { "Give the field less room" } else { "Give the field more room" },
                        "aria-expanded": "{exp()}", "aria-controls": "{id}",
                        onclick: move |_| { let n = !exp(); exp.set(n); on_expanded.call(n) },
                        Icon { name: if exp() { "minimize" } else { "maximize" }, size: 13.0 }
                    }
                }
            }
            if msg.is_some() || count.is_some() {
                div { class: "oxi-foot",
                    div { id: "{id}-m", class: if error.is_some() { "oxi-msg err" } else { "oxi-msg" }, "aria-live": "polite",
                        if let Some(m) = &msg { "{m}" }
                    }
                    if let Some((c, o)) = &count { span { class: if *o { "oxi-count over" } else { "oxi-count" }, "{c}" } }
                }
            }
        }
    }
}

/// A message row whose height opens on a spring and whose changed words rise in (numbers roll
/// only the digits that changed). Keys do the work: an unchanged word keeps its node.
#[component]
fn MsgRow(id: String, text: Option<String>, error: bool) -> Element {
    let open = text.is_some();
    let words: Vec<String> = text
        .as_deref()
        .unwrap_or("")
        .split(' ')
        .map(str::to_string)
        .collect();
    rsx! {
        div { class: "oxi-row", "data-open": "{open}",
            div { class: "oxi-rowin",
                div { id: "{id}", class: if error { "oxi-msg err" } else { "oxi-msg" },
                    role: if error { "alert" },
                    for (i, w) in words.into_iter().enumerate() {
                        if w.chars().any(|c| c.is_ascii_digit()) {
                            span { key: "n{i}", class: "oxi-num",
                                // Keyed by place from the end: only digits that changed remount and roll.
                                for (j, c) in w.chars().rev().enumerate().collect::<Vec<_>>().into_iter().rev() {
                                    span { key: "{j}-{c}", class: "oxi-d", "{c}" }
                                }
                                " "
                            }
                        } else {
                            span { key: "{i}-{w}", class: "oxi-w", "{w} " }
                        }
                    }
                }
            }
        }
    }
}

/// Input (Arc behaviour): a single-line field with a label, a helper row linked through
/// aria-describedby, and an error row (role=alert, aria-invalid). Rows open on a spring; changed
/// words rise in and unblur, numbers roll digit by digit.
#[component]
pub fn LabeledInput(
    /// Label.
    label: String,
    /// Initial value.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// Helper copy.
    #[props(default)]
    description: Option<String>,
    /// Error copy (replaces the helper).
    #[props(default)]
    error: Option<String>,
    /// `text` · `email` · `password` · `url`.
    #[props(default = "text".to_string())]
    kind: String,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Every edit.
    #[props(default)]
    on_input: EventHandler<String>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let id = slug("in", &label);
    let invalid = error.is_some();
    rsx! {
        div { class: "oxi-field", "data-invalid": "{invalid}",
            label { class: "oxi-lab", r#for: "{id}", "{label}" }
            input { id: "{id}", class: "oxi-line", r#type: "{kind}", value: "{v}", disabled,
                placeholder: if let Some(p) = &placeholder { "{p}" },
                "aria-invalid": "{invalid}", "aria-describedby": "{id}-m",
                oninput: move |e| { v.set(e.value()); on_input.call(e.value()) },
            }
            MsgRow { id: "{id}-m", text: error.clone().or(description.clone()), error: invalid }
        }
    }
}

/// Textarea (Arc behaviour): the multi-line `LabeledInput`. With `count` the helper row ends in
/// "120 characters", whose digits roll as you type.
#[component]
pub fn LabeledTextarea(
    /// Label.
    label: String,
    /// Initial value.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// Helper copy.
    #[props(default)]
    description: Option<String>,
    /// Error copy (replaces the helper).
    #[props(default)]
    error: Option<String>,
    /// Append the character count to the helper.
    #[props(default)]
    count: bool,
    /// Visible rows.
    #[props(default = 4)]
    rows: usize,
    /// Character limit.
    #[props(default)]
    max_length: Option<usize>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Every edit.
    #[props(default)]
    on_input: EventHandler<String>,
) -> Element {
    let mut v = use_signal(|| value.clone());
    let id = slug("tx", &label);
    let invalid = error.is_some();
    let n = v().chars().count();
    let msg = error.clone().or_else(|| match (count, &description) {
        (true, Some(d)) => Some(format!("{d} · {n} characters")),
        (true, None) => Some(format!("{n} characters")),
        (false, d) => d.clone(),
    });
    rsx! {
        div { class: "oxi-field", "data-invalid": "{invalid}",
            label { class: "oxi-lab", r#for: "{id}", "{label}" }
            textarea { id: "{id}", class: "oxi-line oxi-multi", rows: "{rows}", value: "{v}", disabled,
                placeholder: if let Some(p) = &placeholder { "{p}" },
                maxlength: if let Some(m) = max_length { "{m}" },
                "aria-invalid": "{invalid}", "aria-describedby": "{id}-m",
                oninput: move |e| { v.set(e.value()); on_input.call(e.value()) },
            }
            MsgRow { id: "{id}-m", text: msg, error: invalid }
        }
    }
}

/// Ghost text (Arc behaviour): an inline autocomplete. After a pause the model is asked for a
/// continuation of the text; it shows muted right after the text and fades in. Typing that
/// matches consumes it, Tab takes it all, ⌘/Ctrl/Alt+Right takes one word, Escape dismisses it
/// (and only then; with nothing showing Escape is left to the page). Moving the caret away
/// clears it. Accepted text eases from muted to solid in place. A chip names the model and
/// shows Thinking / Tab to accept / No suggestion.
#[component]
pub fn GhostText(
    /// Starting text.
    #[props(default)]
    value: String,
    /// Placeholder.
    #[props(default)]
    placeholder: Option<String>,
    /// Quiet label row above the text (multi-line only).
    #[props(default)]
    label: Option<String>,
    /// Model named in the chip.
    #[props(default = "Autocomplete".to_string())]
    model: String,
    /// Growing textarea with a footer row, or a single line with the chip at its end.
    #[props(default = true)]
    multiline: bool,
    /// `field` (own border and ring) · `plain` (sits flush in a card).
    #[props(default = "field".to_string())]
    variant: String,
    /// Minimum lines.
    #[props(default = 3)]
    rows: usize,
    /// Show the chip and key hints.
    #[props(default = true)]
    hint: bool,
    /// The model fails every ask (demos the error chip).
    #[props(default)]
    offline: bool,
    /// Pause before asking, ms.
    #[props(default = 450)]
    delay_ms: u64,
    /// Simulated model latency, ms.
    #[props(default = 380)]
    latency_ms: u64,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock.
    now: ClockMs,
    /// Accepted text, with "all" or "word".
    #[props(default)]
    on_accept: EventHandler<(String, &'static str)>,
    /// Every edit.
    #[props(default)]
    on_input: EventHandler<String>,
) -> Element {
    let mut text = use_signal(|| value.clone());
    let mut g = use_signal(move || Ghost {
        delay_ms,
        ..Ghost::default()
    });
    let mut at_end = use_signal(|| true);
    let ask = move |t: &str| -> Result<Option<String>, ()> {
        if offline {
            Err(())
        } else {
            Ok(demo_suggestion(t))
        }
    };
    // The view settles a copy on the clock; handlers settle the real one before acting.
    let mut view = g();
    if at_end() && !disabled {
        view.settle(&text(), now, latency_ms, ask);
    }
    let ghost = if at_end() {
        view.suggestion.clone()
    } else {
        None
    };
    let status = if disabled {
        GhostStatus::Idle
    } else {
        view.status
    };
    let t = text();
    let (head, acc) = match &view.accepted {
        Some((a, _))
            if t.ends_with(a.as_str())
                && now
                    .0
                    .saturating_sub(view.accepted.as_ref().map_or(0, |x| x.1.0))
                    < 600 =>
        {
            (t[..t.len() - a.len()].to_string(), Some(a.clone()))
        }
        _ => (t.clone(), None),
    };
    let acc_key = view.accepted.as_ref().map_or(0, |x| x.1.0);
    let lines = if multiline {
        grow_rows(
            &format!("{t}{}", ghost.clone().unwrap_or_default()),
            60,
            rows,
            10,
        )
        .0
    } else {
        1
    };
    let chip = status.chip(&model);
    let id = slug("gh", label.as_deref().unwrap_or(&model));
    let keydown = move |e: KeyboardEvent| {
        let mut w = g.write();
        if at_end() {
            w.settle(&text(), now, latency_ms, ask);
        }
        let showing = at_end() && w.suggestion.is_some();
        let m = e.modifiers();
        match e.key() {
            Key::Tab if showing && !m.shift() => {
                e.prevent_default();
                if let Some(s) = w.accept_all(now) {
                    let n = format!("{}{s}", text());
                    text.set(n.clone());
                    on_input.call(n);
                    on_accept.call((s, "all"));
                }
            }
            Key::ArrowRight if showing && (m.meta() || m.ctrl() || m.alt()) => {
                e.prevent_default();
                if let Some(s) = w.accept_word(now) {
                    let n = format!("{}{s}", text());
                    text.set(n.clone());
                    on_input.call(n);
                    on_accept.call((s, "word"));
                }
            }
            Key::Escape if showing => {
                e.prevent_default();
                e.stop_propagation();
                w.dismiss();
            }
            Key::ArrowLeft | Key::ArrowUp | Key::Home | Key::PageUp => {
                w.caret_moved();
                at_end.set(false);
            }
            Key::End => at_end.set(true),
            _ => {}
        }
    };
    let oninput = move |e: FormEvent| {
        let new = e.value();
        let old = text();
        let end = at_end() && (new.starts_with(&old) || old.starts_with(&new));
        let mut w = g.write();
        w.settle(&old, now, latency_ms, ask);
        w.edit(&old, &new, end, now);
        at_end.set(end || new.is_empty());
        text.set(new.clone());
        on_input.call(new);
    };
    rsx! {
        div { class: "oxi-ghost {variant}", "data-multi": "{multiline}", "data-disabled": "{disabled}",
            if multiline {
                if let Some(l) = &label { label { class: "oxi-glab", r#for: "{id}", "{l}" } }
            }
            div { class: "oxi-gbox", style: "--rows:{lines}",
                div { class: "oxi-mirror", "aria-hidden": "true",
                    span { "{head}" }
                    if let Some(a) = &acc { span { key: "{acc_key}", class: "oxi-acc", "{a}" } }
                    if let Some(s) = &ghost { span { key: "{s}", class: "oxi-gt", "{s}" } }
                    "\u{200b}"
                }
                if multiline {
                    textarea { id: "{id}", class: "oxi-gin", value: "{t}", disabled, spellcheck: "false",
                        placeholder: if let Some(p) = &placeholder { "{p}" },
                        "aria-label": if label.is_none() { "Text with suggestions" },
                        "aria-describedby": "{id}-k",
                        oninput, onkeydown: keydown,
                        onmousedown: move |_| { g.write().caret_moved(); at_end.set(false) },
                        onblur: move |_| { g.write().dismiss(); },
                        onfocus: move |_| at_end.set(true),
                    }
                } else {
                    input { id: "{id}", class: "oxi-gin", r#type: "text", value: "{t}", disabled, spellcheck: "false",
                        placeholder: if let Some(p) = &placeholder { "{p}" },
                        "aria-label": "Text with suggestions", "aria-describedby": "{id}-k",
                        oninput, onkeydown: keydown,
                        onmousedown: move |_| { g.write().caret_moved(); at_end.set(false) },
                        onblur: move |_| { g.write().dismiss(); },
                        onfocus: move |_| at_end.set(true),
                    }
                }
                if hint && !multiline {
                    span { class: "oxi-chip", "data-s": "{status:?}", "{chip}" }
                }
            }
            span { id: "{id}-k", class: "oxi-sr", "Suggestions appear after a pause. Tab accepts, Command or Alt with Right Arrow accepts a word, Escape dismisses." }
            if hint && multiline {
                div { class: "oxi-gfoot",
                    span { class: "oxi-chip", "data-s": "{status:?}", "aria-live": "polite", "{chip}" }
                    if ghost.is_some() {
                        span { class: "oxi-keys", kbd { "Tab" } " accept · " kbd { "⌘→" } " word · " kbd { "Esc" } " dismiss" }
                    }
                }
            }
        }
    }
}
