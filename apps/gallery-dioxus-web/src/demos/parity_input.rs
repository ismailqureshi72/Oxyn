//! Plan #28 parity demos, inputs (chunk 4a): text area, labelled input, labelled textarea and
//! ghost text.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{GhostText, LabeledInput, LabeledTextarea, TextArea};

use super::{Code, mk, q, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

fn none_if_empty(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

// ---------- text area ----------

fn text_area(v: &Vals, _: ClockMs) -> Element {
    let prefill = match v.string("prefill").as_str() {
        "short" => "Summarise the call notes and list the three decisions with owners.".to_string(),
        "over" => "x".repeat(212),
        _ => String::new(),
    };
    let max = if v.b("counter") { Some(200) } else { None };
    let mut sent = use_signal(String::new);
    rsx! {
        div { class: "oxd-w420", style: "display:flex;flex-direction:column;gap:10px;width:420px;max-width:100%",
            TextArea { label: none_if_empty(v.string("label")), value: prefill,
                placeholder: Some("Describe what the agent should do…".to_string()),
                hint: none_if_empty(v.string("hint")), error: none_if_empty(v.string("error")),
                optional: v.b("optional"), counter_on: v.b("counter"), max_length: max,
                expandable: v.b("expandable"), mono: v.b("mono"), disabled: v.b("disabled"),
                on_submit: move |t: String| sent.set(format!("⌘ Enter sent {} characters", t.chars().count())),
            }
            if !sent().is_empty() { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted)", role: "status", "{sent}" } }
        }
    }
}

fn text_area_code(v: &Vals) -> String {
    Code::new("TextArea")
        .raw("label", format!("Some({}.into())", q(&v.string("label"))))
        .opt("hint", &v.string("hint"))
        .opt("error", &v.string("error"))
        .b("optional", v.b("optional"))
        .b("counter_on", v.b("counter"))
        .raw(
            "max_length",
            if v.b("counter") { "Some(200)" } else { "None" },
        )
        .b("expandable", v.b("expandable"))
        .b("mono", v.b("mono"))
        .raw("on_submit", "move |text: String| send(text)")
        .done()
}

// ---------- labelled input / textarea ----------

fn labeled_input(v: &Vals, _: ClockMs) -> Element {
    let kind = v.string("kind");
    rsx! {
        div { style: "width:340px;max-width:100%",
            LabeledInput { label: v.string("label"), kind: kind.clone(),
                placeholder: Some(if kind == "email" { "you@company.com" } else { "Atlas" }.to_string()),
                description: none_if_empty(v.string("description")), error: none_if_empty(v.string("error")),
                disabled: v.b("disabled"),
            }
        }
    }
}

fn labeled_input_code(v: &Vals) -> String {
    Code::new("LabeledInput")
        .raw("label", q(&v.string("label")))
        .s("kind", &v.string("kind"), "text")
        .opt("description", &v.string("description"))
        .opt("error", &v.string("error"))
        .done()
}

fn labeled_textarea(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { style: "width:380px;max-width:100%",
            LabeledTextarea { label: v.string("label"),
                value: "Ship the inputs chunk after CI is green.".to_string(),
                placeholder: Some("Notes for the reviewer".to_string()),
                description: none_if_empty(v.string("description")), error: none_if_empty(v.string("error")),
                count: v.b("count"), max_length: Some(500), disabled: v.b("disabled"),
            }
        }
    }
}

fn labeled_textarea_code(v: &Vals) -> String {
    Code::new("LabeledTextarea")
        .raw("label", q(&v.string("label")))
        .opt("description", &v.string("description"))
        .opt("error", &v.string("error"))
        .b("count", v.b("count"))
        .done()
}

// ---------- ghost text ----------

fn ghost(v: &Vals, now: ClockMs) -> Element {
    let multi = v.b("multiline");
    let mut log = use_signal(String::new);
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:10px;width:460px;max-width:100%",
            div { class: if v.string("variant") == "plain" { "oxd-card" } else { "" },
                style: if v.string("variant") == "plain" { "border:1px solid var(--oxyn-color-border-default);border-radius:12px;padding:6px" } else { "" },
                GhostText { value: v.string("value"), multiline: multi, variant: v.string("variant"),
                    label: if multi { Some("Reply".to_string()) } else { None },
                    placeholder: Some("Start typing… try “Thanks for” or “Let me know”".to_string()),
                    model: v.string("model"), hint: v.b("hint"), offline: v.b("offline"), disabled: v.b("disabled"), now,
                    on_accept: move |(s, how): (String, &'static str)| log.set(format!("Accepted ({how}): “{}”", s.trim())),
                }
            }
            span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{log}" }
        }
    }
}

fn ghost_code(v: &Vals) -> String {
    Code::new("GhostText")
        .pre("// Suggestions come from your model; the demo answers from a small table after 380 ms.")
        .b("multiline", !v.b("multiline"))
        .s("variant", &v.string("variant"), "field")
        .s("model", &v.string("model"), "Autocomplete")
        .raw("now", "clock.now()")
        .raw("on_accept", "move |(text, how)| log(text, how)")
        .done()
}

pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                txt("label", "Label", "Instructions"),
                txt(
                    "hint",
                    "Hint",
                    "⌘ Enter saves. The agent reads this before every run.",
                ),
                txt("error", "Error", ""),
                sel("prefill", "Text", &["empty", "short", "over"], "short"),
                tog("counter", "Counter (200)", true),
                tog("expandable", "More room button", true),
                tog("optional", "Optional", false),
                tog("mono", "Monospace", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "default",
                    "Grows with the text; counter and more-room button.",
                    &[],
                ),
                var(
                    "error",
                    "The error takes the hint's place; red edge and faint fill.",
                    &[("error", "Add at least one instruction for the agent.")],
                ),
                var(
                    "over",
                    "A prefilled value over the limit turns the counter red.",
                    &[("prefill", "over")],
                ),
                var(
                    "prompt",
                    "Monospace for prompts, optional, empty.",
                    &[("mono", "true"), ("optional", "true"), ("prefill", "empty")],
                ),
            ],
            render: text_area,
            code: text_area_code,
            keyboard: vec![
                ("⌘/Ctrl + Enter", "Submits (save or send)"),
                ("Enter", "New line"),
                ("Tab", "Moves to the more-room button"),
            ],
            a11y: vec![
                "The label is a <label for>: clicking it focuses the field.",
                "aria-invalid and aria-describedby point at the hint or error row.",
                "The more-room button has aria-expanded and aria-controls.",
            ],
            responsive: vec![
                "Fills its container; the height follows the text up to ten lines, then scrolls.",
            ],
            height: 330,
            align: Align::Center,
            live: false,
            ..mk(
                "text-area",
                "Text area",
                "inputs",
                "The multi-line twin of the text field: grows with the text, a character count, a button for more room and ⌘ Enter to send.",
                "TextArea",
                "",
                "text-area",
                "parity/input.rs",
            )
        },
        Demo {
            controls: vec![
                txt("label", "Label", "Agent name"),
                sel("kind", "Type", &["text", "email", "password"], "text"),
                txt(
                    "description",
                    "Description",
                    "Shown in the sidebar. 3 to 24 characters.",
                ),
                txt("error", "Error", ""),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "helper",
                    "Helper copy linked through aria-describedby.",
                    &[],
                ),
                var(
                    "error",
                    "The error row opens on a spring; changed words rise in.",
                    &[("error", "Use 3 to 24 characters.")],
                ),
                var("bare", "No helper row.", &[("description", "")]),
            ],
            render: labeled_input,
            code: labeled_input_code,
            keyboard: vec![("Tab", "Focuses the field")],
            a11y: vec![
                "The error row is role=alert and sets aria-invalid.",
                "Reduced motion opens rows at full height and drops the rise.",
            ],
            responsive: vec!["Fills its container."],
            height: 200,
            align: Align::Center,
            live: false,
            ..mk(
                "labeled-input",
                "Input",
                "inputs",
                "A single-line field with a label, a helper row and an error row that open on a spring; changed words rise in and numbers roll digit by digit.",
                "LabeledInput",
                "",
                "labeled-input",
                "parity/input.rs",
            )
        },
        Demo {
            controls: vec![
                txt("label", "Label", "Review notes"),
                txt("description", "Description", "Markdown works"),
                txt("error", "Error", ""),
                tog("count", "Character count", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "count",
                    "The count rolls only the digits that change as you type.",
                    &[],
                ),
                var(
                    "error",
                    "Error row with role=alert.",
                    &[("error", "Notes are required before you approve.")],
                ),
            ],
            render: labeled_textarea,
            code: labeled_textarea_code,
            keyboard: vec![("Tab", "Focuses the field")],
            a11y: vec!["aria-describedby links the helper or error row; the error is role=alert."],
            responsive: vec!["Fills its container; resizes vertically."],
            height: 260,
            align: Align::Center,
            live: false,
            ..mk(
                "labeled-textarea",
                "Textarea",
                "inputs",
                "A multiline field for notes and longer text, with helper and error rows that open on a spring and a count that rolls only the digits that changed.",
                "LabeledTextarea",
                "",
                "labeled-textarea",
                "parity/input.rs",
            )
        },
        Demo {
            controls: vec![
                tog("multiline", "Multi-line", true),
                sel("variant", "Variant", &["field", "plain"], "field"),
                txt("value", "Text", "Thanks for"),
                txt("model", "Model", "Claude Haiku"),
                tog("hint", "Chip and key hints", true),
                tog("offline", "Model offline", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "field",
                    "A suggestion waits after the text: Tab takes it.",
                    &[],
                ),
                var(
                    "single",
                    "Single line with the chip inline at its end.",
                    &[("multiline", "false"), ("value", "Let me know")],
                ),
                var(
                    "plain",
                    "Plain: flush inside a card of your own.",
                    &[("variant", "plain"), ("value", "Looking forward")],
                ),
                var(
                    "offline",
                    "The chip says No suggestion.",
                    &[("offline", "true")],
                ),
            ],
            render: ghost,
            code: ghost_code,
            keyboard: vec![
                (
                    "Tab",
                    "Accepts the whole suggestion (moves focus when none shows)",
                ),
                ("⌘/Ctrl/Alt + →", "Accepts the next word"),
                (
                    "Esc",
                    "Dismisses the suggestion (left to the page when none shows)",
                ),
                (
                    "Typing",
                    "Matching characters consume the ghost; others clear it",
                ),
                ("← ↑ Home, click", "Moving the caret away clears it"),
            ],
            a11y: vec![
                "The ghost is drawn in an aria-hidden mirror; the field keeps its own text.",
                "aria-describedby explains the keys; the chip is a polite live region.",
                "Reduced motion drops the fade and the accept ease.",
            ],
            responsive: vec!["Key hints hide on narrow fields; the chip stays."],
            height: 280,
            align: Align::Center,
            live: true,
            ..mk(
                "ghost-text",
                "Ghost text",
                "inputs",
                "Inline AI autocomplete: muted ghost text continues your sentence at the caret, Tab turns it solid in place, ⌘→ takes a word, Esc fades it away, and a chip names the model.",
                "GhostText",
                "",
                "ghost-text",
                "parity/input.rs",
            )
        },
    ]
}
