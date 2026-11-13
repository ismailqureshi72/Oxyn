//! Inputs: switch, checkbox, radio group, segmented control, select, slider, text field.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        segmented::Segment,
        slider::{Mark, Slider as SliderModel},
        toggle::RadioOption,
    },
};
use oxyn_dioxus::{Checkbox, RadioGroup, SegmentedControl, Select, Slider, Switch, TextField};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals, data};

fn switch(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w360",
            Switch {
                label: v.string("label"),
                description: v.opt("description"),
                checked: v.b("checked"),
                placement: v.string("placement"),
                size: v.string("size"),
                disabled: v.b("disabled"),
                pending: v.b("pending"),
                restart_note: v.b("restart_note"),
            }
        }
    }
}

fn switch_code(v: &Vals) -> String {
    Code::new("Switch")
        .raw("label", q(v.s("label")))
        .opt("description", v.s("description"))
        .b("checked", v.b("checked"))
        .s("placement", v.s("placement"), "start")
        .s("size", v.s("size"), "md")
        .b("disabled", v.b("disabled"))
        .b("pending", v.b("pending"))
        .b("restart_note", v.b("restart_note"))
        .raw(
            "on_change",
            "move |on: bool| settings.write().auto_approve_reads = on",
        )
        .done()
}

fn checkbox(v: &Vals, _: ClockMs) -> Element {
    let error = v.b("error").then(|| "Tick this to continue".to_string());
    rsx! {
        div { class: "oxd-w360",
            Checkbox {
                label: v.string("label"),
                description: v.opt("description"),
                value: v.string("value"),
                error,
                variant: v.string("variant"),
                size: v.string("size"),
                tri_state: v.b("tri_state"),
                disabled: v.b("disabled"),
            }
        }
    }
}

fn checkbox_code(v: &Vals) -> String {
    Code::new("Checkbox")
        .raw("label", q(v.s("label")))
        .opt("description", v.s("description"))
        .s("value", v.s("value"), "unchecked")
        .opt(
            "error",
            if v.b("error") {
                "Tick this to continue"
            } else {
                ""
            },
        )
        .s("variant", v.s("variant"), "plain")
        .s("size", v.s("size"), "md")
        .b("tri_state", v.b("tri_state"))
        .b("disabled", v.b("disabled"))
        .raw("on_change", "move |c: Check| run_tests.set(c)")
        .done()
}

fn radio_options(disabled: bool) -> Vec<RadioOption> {
    vec![
        RadioOption {
            description: Some("Asks before every edit".into()),
            ..RadioOption::new("ask", "Ask first")
        },
        RadioOption {
            description: Some("Edits files, asks before shell commands".into()),
            ..RadioOption::new("edits", "Auto-edit")
        },
        RadioOption {
            description: Some("Runs everything; you review at the end".into()),
            disabled: disabled.then(|| "Your admin turned this off".into()),
            ..RadioOption::new("full", "Full auto")
        },
    ]
}

fn radio(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w420",
            RadioGroup {
                label: "Approval mode",
                options: radio_options(v.b("disabled_option")),
                value: v.opt("value"),
                layout: v.string("layout"),
            }
        }
    }
}

fn radio_code(v: &Vals) -> String {
    Code::new("RadioGroup")
        .pre("let options = vec![")
        .pre("    RadioOption { description: Some(\"Asks before every edit\".into()), ..RadioOption::new(\"ask\", \"Ask first\") },")
        .pre("    RadioOption { description: Some(\"Edits files, asks before shell commands\".into()), ..RadioOption::new(\"edits\", \"Auto-edit\") },")
        .pre("    RadioOption { disabled: Some(\"Your admin turned this off\".into()), ..RadioOption::new(\"full\", \"Full auto\") },")
        .pre("];")
        .raw("label", q("Approval mode"))
        .raw("options", "options")
        .opt("value", v.s("value"))
        .s("layout", v.s("layout"), "list")
        .raw("on_change", "move |mode: String| set_mode(mode)")
        .done()
}

fn segments(n: usize, icons: bool) -> Vec<Segment> {
    let mut v = vec![
        Segment::new("ask", "Ask"),
        Segment::new("plan", "Plan"),
        Segment::new("agent", "Agent"),
        Segment::new("debug", "Debug"),
        Segment::new("review", "Review"),
        Segment::new("research", "Research"),
    ];
    v.truncate(n);
    if icons {
        for (s, i) in v
            .iter_mut()
            .zip(["thread", "checklist", "agents", "code", "eye", "search"])
        {
            s.icon = Some(i.into());
        }
        if let Some(s) = v.get_mut(2) {
            s.count = Some(3);
        }
    }
    v
}

fn segmented(v: &Vals, _: ClockMs) -> Element {
    let opts = segments(v.u("count"), v.b("icons"));
    rsx! {
        div { class: if v.b("full_width") { "oxd-w480" } else { "" },
            SegmentedControl {
                options: opts,
                value: Some("plan".to_string()),
                size: v.string("size"),
                variant: v.string("variant"),
                kind: v.string("kind"),
                full_width: v.b("full_width"),
                vertical: v.b("vertical"),
                disabled: v.b("disabled"),
                aria_label: "Mode",
            }
        }
    }
}

fn segmented_code(v: &Vals) -> String {
    Code::new("SegmentedControl")
        .pre("let options = vec![")
        .pre("    Segment::new(\"ask\", \"Ask\"),")
        .pre("    Segment::new(\"plan\", \"Plan\"),")
        .pre("    Segment::new(\"agent\", \"Agent\"),")
        .pre("];")
        .raw("options", "options")
        .raw("value", "Some(mode())")
        .s("size", v.s("size"), "md")
        .s("variant", v.s("variant"), "tray")
        .s("kind", v.s("kind"), "radio")
        .b("full_width", v.b("full_width"))
        .b("vertical", v.b("vertical"))
        .b("disabled", v.b("disabled"))
        .raw("aria_label", q("Mode"))
        .raw("on_change", "move |m: String| mode.set(m)")
        .done()
}

fn select(v: &Vals, now: ClockMs) -> Element {
    let error = v.b("error").then(|| "Choose a fallback model".to_string());
    rsx! {
        div { class: "oxd-w320 oxd-room-menu",
            Select {
                options: data::model_options(),
                value: v.opt("value"),
                label: Some("Model".to_string()),
                hint: Some("Applies to new runs".to_string()),
                error,
                size: v.string("size"),
                searchable: v.b("searchable"),
                loading: v.b("loading"),
                disabled: v.b("disabled"),
                optional: v.b("optional"),
                open: v.b("open"),
                now,
            }
        }
    }
}

fn select_code(v: &Vals) -> String {
    Code::new("Select")
        .pre("let options = vec![")
        .pre("    SelectOption::new(\"swift\", \"Ember Swift 2\").desc(\"Fastest · $1 / Mtok\").icon(\"bolt\").group(\"Ember\"),")
        .pre("    SelectOption::new(\"deep\", \"Ember Deep 2\").desc(\"Deepest · $5 / Mtok\").icon(\"brain\").group(\"Ember\"),")
        .pre("    SelectOption::new(\"q7\", \"Local Q 7B\").desc(\"Free · on device\").icon(\"bolt\").group(\"On device\"),")
        .pre("];")
        .raw("options", "options")
        .opt("value", v.s("value"))
        .raw("label", q("Model"))
        .raw("hint", q("Applies to new runs"))
        .opt(
            "error",
            if v.b("error") {
                "Choose a fallback model"
            } else {
                ""
            },
        )
        .s("size", v.s("size"), "md")
        .b("searchable", v.b("searchable"))
        .b("loading", v.b("loading"))
        .b("optional", v.b("optional"))
        .b("disabled", v.b("disabled"))
        .raw("on_change", "move |id: String| model.set(id)")
        .done()
}

fn slider_model(kind: &str, disabled: bool) -> SliderModel {
    let mut m = match kind {
        "range" => {
            let mut m = SliderModel::range(20.0, 60.0, 0.0, 100.0, 5.0);
            m.min_steps_between = 2;
            m
        }
        "detents" => {
            let mut m = SliderModel::new(1.0, 0.0, 3.0, 0.01);
            m.snap = 0.25;
            m.marks = ["Low", "Medium", "High", "Max"]
                .iter()
                .enumerate()
                .map(|(i, l)| Mark {
                    value: i as f64,
                    label: (*l).into(),
                    detent: true,
                })
                .collect();
            m
        }
        "vertical" => {
            let mut m = SliderModel::new(60.0, 0.0, 100.0, 1.0);
            m.vertical = true;
            m
        }
        _ => {
            let mut m = SliderModel::new(0.7, 0.0, 2.0, 0.1);
            m.marks = [0.0, 1.0, 2.0]
                .iter()
                .map(|x| Mark {
                    value: *x,
                    label: format!("{x}"),
                    detent: false,
                })
                .collect();
            m
        }
    };
    m.disabled = disabled;
    m
}

fn slider(v: &Vals, _: ClockMs) -> Element {
    let kind = v.s("kind");
    let m = slider_model(kind, v.b("disabled"));
    let (label, format, decimals, tag, thumbs) = match kind {
        "range" => (
            "Monthly budget",
            "$#",
            0,
            None,
            vec!["Minimum".to_string(), "Maximum".to_string()],
        ),
        "detents" => (
            "Effort",
            "#",
            0,
            v.b("tag").then(|| "≈ $0.42 / run".to_string()),
            vec![],
        ),
        "vertical" => ("Volume", "#%", 0, None, vec![]),
        _ => ("Temperature", "#", 1, None, vec![]),
    };
    rsx! {
        div { class: if kind == "vertical" { "" } else { "oxd-w420" },
            Slider { slider: m, label, format, decimals, show_value: v.b("show_value"), tag, thumb_labels: thumbs }
        }
    }
}

fn slider_code(v: &Vals) -> String {
    let (setup, label, extra): (&[&str], &str, &str) = match v.s("kind") {
        "range" => (
            &[
                "let mut budget = SliderModel::range(20.0, 60.0, 0.0, 100.0, 5.0);",
                "budget.min_steps_between = 2;",
            ],
            "Monthly budget",
            "format: \"$#\", thumb_labels: vec![\"Minimum\".into(), \"Maximum\".into()]",
        ),
        "detents" => (
            &[
                "let mut effort = SliderModel::new(1.0, 0.0, 3.0, 0.01);",
                "effort.snap = 0.25; // pulls to the named detents",
                "effort.marks = [\"Low\", \"Medium\", \"High\", \"Max\"].iter().enumerate()",
                "    .map(|(i, l)| Mark { value: i as f64, label: (*l).into(), detent: true }).collect();",
            ],
            "Effort",
            "tag: \"≈ $0.42 / run\"",
        ),
        "vertical" => (
            &[
                "let mut volume = SliderModel::new(60.0, 0.0, 100.0, 1.0);",
                "volume.vertical = true;",
            ],
            "Volume",
            "format: \"#%\"",
        ),
        _ => (
            &["let temperature = SliderModel::new(0.7, 0.0, 2.0, 0.1);"],
            "Temperature",
            "decimals: 1",
        ),
    };
    let mut c = Code::new("Slider");
    for l in setup {
        c = c.pre(*l);
    }
    let model = match v.s("kind") {
        "range" => "budget",
        "detents" => "effort",
        "vertical" => "volume",
        _ => "temperature",
    };
    let mut c = c.raw("slider", model).raw("label", q(label));
    for part in extra.split(", thumb") {
        let (k, val) = part.split_once(": ").unwrap_or(("", ""));
        let k = if k.starts_with("_labels") {
            "thumb_labels"
        } else {
            k
        };
        c = c.raw(k, val);
    }
    if !v.b("show_value") {
        c = c.raw("show_value", "false");
    }
    c.raw("on_commit", "move |v: Vec<f64>| save(v)").done()
}

fn text_field(v: &Vals, now: ClockMs) -> Element {
    let kind = v.s("kind");
    let (label, value, prefix, suffix, placeholder) = match kind {
        "password" => ("Password", "hunter22", None, None, None),
        "email" => ("Email", "ada@", None, None, Some("name@example.com")),
        "search" => ("Search runs", "flaky", Some("icon:search"), None, None),
        "number" => ("Budget", "250", Some("$"), Some("/ month"), None),
        _ => (
            "Commit message",
            "fix: retry flaky network calls in the sync worker",
            None,
            None,
            None,
        ),
    };
    let error = v
        .b("error")
        .then(|| "Enter an email like name@example.com".to_string());
    let counter = v.b("counter");
    rsx! {
        div { class: "oxd-w360",
            TextField {
                label,
                value,
                kind: kind.to_string(),
                placeholder: placeholder.map(str::to_string),
                hint: v.opt("hint"),
                error,
                prefix: prefix.map(str::to_string),
                suffix: suffix.map(str::to_string),
                size: v.string("size"),
                clearable: v.b("clearable"),
                copyable: v.b("copyable"),
                counter,
                max: counter.then_some(50),
                soft: counter.then_some(40),
                auto_hide_s: (kind == "password").then_some(10),
                optional: v.b("optional"),
                disabled: v.b("disabled"),
                now,
            }
        }
    }
}

fn text_field_code(v: &Vals) -> String {
    let kind = v.s("kind");
    let label = match kind {
        "password" => "Password",
        "email" => "Email",
        "search" => "Search runs",
        "number" => "Budget",
        _ => "Commit message",
    };
    let mut c = Code::new("TextField")
        .raw("label", q(label))
        .raw("value", "draft()")
        .s("kind", kind, "text")
        .opt("hint", v.s("hint"));
    c = match kind {
        "search" => c.raw("prefix", q("icon:search")),
        "number" => c.raw("prefix", q("$")).raw("suffix", q("/ month")),
        "password" => c.raw("auto_hide_s", "Some(10)"),
        _ => c,
    };
    if v.b("counter") {
        c = c
            .raw("counter", "true")
            .raw("max", "Some(50)")
            .raw("soft", "Some(40)");
    }
    c.s("size", v.s("size"), "md")
        .b("clearable", v.b("clearable"))
        .b("copyable", v.b("copyable"))
        .b("optional", v.b("optional"))
        .b("disabled", v.b("disabled"))
        .raw("on_input", "move |s: String| draft.set(s)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "switch",
            name: "Switch",
            category: "inputs",
            summary: "On or off for things that apply right away: a label and second line, at the start or across a settings row, a thumb that stretches as you press it, a pending state and a restart note.",
            dioxus: "Switch",
            gpui: "SwitchEl",
            spec: "toggles",
            source: "primitives/forms.rs",
            controls: vec![
                txt("label", "Label", "Auto-approve reads"),
                txt(
                    "description",
                    "Second line",
                    "Agents read files without asking",
                ),
                tog("checked", "Checked", true),
                sel("placement", "Placement", &["start", "end"], "start"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("pending", "Pending", false),
                tog("restart_note", "Restart note", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("start", "Switch first, label after.", &[]),
                var(
                    "settings-row",
                    "Label first, switch at the end of the row.",
                    &[
                        ("placement", "end"),
                        ("label", "Stream reasoning"),
                        ("description", "Show the model's thinking as it arrives"),
                        ("checked", "false"),
                    ],
                ),
                var(
                    "restart",
                    "Says the change needs a restart.",
                    &[
                        ("placement", "end"),
                        ("label", "GPU acceleration"),
                        ("description", ""),
                        ("restart_note", "true"),
                        ("checked", "false"),
                    ],
                ),
                var(
                    "pending",
                    "Applying: the thumb waits for the host.",
                    &[("pending", "true")],
                ),
            ],
            render: switch,
            code: switch_code,
            keyboard: vec![
                ("Space", "Toggle"),
                ("Enter", "Toggle (like most settings panels)"),
                ("Tab", "Next control"),
            ],
            a11y: vec![
                "A button with role=switch and aria-checked inside a real label element.",
                "Pending sets aria-busy while the host applies the change.",
            ],
            responsive: vec![
                "end placement stretches across the row, label left, switch right: the settings-list layout.",
                "The label is part of the hit area.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "checkbox",
            name: "Checkbox",
            category: "inputs",
            summary: "A real checkbox with a label and second line, an indeterminate state for Select all, an error, a card row and a check that draws itself.",
            dioxus: "Checkbox",
            gpui: "CheckboxEl",
            spec: "toggles",
            source: "primitives/forms.rs",
            controls: vec![
                txt("label", "Label", "Run tests after every edit"),
                txt(
                    "description",
                    "Second line",
                    "Slower, but catches breakage early",
                ),
                sel(
                    "value",
                    "Value",
                    &["unchecked", "checked", "indeterminate"],
                    "checked",
                ),
                sel("variant", "Variant", &["plain", "card"], "plain"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("error", "Error", false),
                tog("tri_state", "Tri-state cycle", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("plain", "Label and second line.", &[]),
                var("card", "A whole card row.", &[("variant", "card")]),
                var(
                    "indeterminate",
                    "Some of them: Select all.",
                    &[
                        ("value", "indeterminate"),
                        ("label", "Select all files"),
                        ("description", ""),
                    ],
                ),
                var(
                    "error",
                    "Required, with its error in place.",
                    &[
                        ("value", "unchecked"),
                        ("label", "I understand this deletes the branch"),
                        ("description", ""),
                        ("error", "true"),
                    ],
                ),
            ],
            render: checkbox,
            code: checkbox_code,
            keyboard: vec![("Space", "Toggle"), ("Tab", "Next control")],
            a11y: vec![
                "role=checkbox with aria-checked (mixed when indeterminate).",
                "The error is linked with aria-describedby and sets aria-invalid.",
            ],
            responsive: vec![
                "The card variant fills its container.",
                "The label is part of the hit area.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "radio-group",
            name: "Radio group",
            category: "inputs",
            summary: "Pick one: a list, cards or an inline row, with descriptions and options that are disabled with a reason.",
            dioxus: "RadioGroup",
            gpui: "RadioGroupEl",
            spec: "toggles",
            source: "primitives/forms.rs",
            controls: vec![
                sel("layout", "Layout", &["list", "cards", "inline"], "list"),
                sel("value", "Value", &["ask", "edits", "full"], "edits"),
                tog("disabled_option", "Disable Full auto", true),
            ],
            variants: vec![
                var("list", "Stacked with descriptions.", &[]),
                var(
                    "cards",
                    "Each option is a card.",
                    &[("layout", "cards"), ("value", "ask")],
                ),
                var("inline", "One row.", &[("layout", "inline")]),
            ],
            render: radio,
            code: radio_code,
            keyboard: vec![
                ("↑ / ↓ or ← / →", "Move and select (skips disabled options)"),
                ("Tab", "Into the group lands on the selected option"),
            ],
            a11y: vec![
                "role=radiogroup with a label; one tab stop with roving focus.",
                "Disabled options stay readable and say why (\"Your admin turned this off\").",
            ],
            responsive: vec![
                "list and cards stack vertically; inline puts the options in one row for short labels.",
            ],
            height: 280,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "segmented-control",
            name: "Segmented control",
            category: "inputs",
            summary: "Switch between two to six options: a raised pill or underline that glides and can be dragged across, with icons, counts and badges.",
            dioxus: "SegmentedControl",
            gpui: "SegmentedEl",
            spec: "segmented-control",
            source: "primitives/forms.rs",
            controls: vec![
                rng("count", "Options", 2.0, 6.0, 1.0, "3"),
                sel("variant", "Variant", &["tray", "underline"], "tray"),
                sel("kind", "Semantics", &["radio", "tabs"], "radio"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("icons", "Icons and count", false),
                tog("full_width", "Full width", false),
                tog("vertical", "Vertical", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("tray", "Raised pill in a tray.", &[]),
                var(
                    "underline-tabs",
                    "Underline, tab semantics.",
                    &[("variant", "underline"), ("kind", "tabs")],
                ),
                var(
                    "icons",
                    "Icons and a count.",
                    &[("icons", "true"), ("size", "lg")],
                ),
                var(
                    "six-full-width",
                    "Six options stretched.",
                    &[("count", "6"), ("full_width", "true"), ("size", "sm")],
                ),
            ],
            render: segmented,
            code: segmented_code,
            keyboard: vec![
                ("← / →", "Move and select (↑ / ↓ when vertical)"),
                ("Home / End", "First / last option"),
                ("Drag", "Drag the pill across to choose"),
            ],
            a11y: vec![
                "Roving tabindex; radio kind follows radio-group keys, tabs kind follows the WAI tabs pattern.",
                "Counts and badges are part of the option's name; the pill is aria-hidden.",
            ],
            responsive: vec![
                "full_width shares the width equally.",
                "Many options in a narrow panel: use full_width and sm; labels truncate before they wrap.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "select",
            name: "Select",
            category: "inputs",
            summary: "Pick one option in the text field's look: groups, icons and a second line per option, a gliding highlight, type-ahead, search and a loading state.",
            dioxus: "Select",
            gpui: "SelectEl",
            spec: "select",
            source: "primitives/forms.rs",
            controls: vec![
                sel(
                    "value",
                    "Value",
                    &["none", "swift", "deep", "one", "q7"],
                    "swift",
                ),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("open", "Open", true),
                tog("searchable", "Searchable", false),
                tog("loading", "Loading", false),
                tog("error", "Error", false),
                tog("optional", "Optional", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("closed", "A value picked.", &[("open", "false")]),
                var("open", "Open with groups and second lines.", &[]),
                var(
                    "searchable",
                    "A filter field on top.",
                    &[("searchable", "true")],
                ),
                var(
                    "error",
                    "Required, nothing chosen.",
                    &[("open", "false"), ("value", "none"), ("error", "true")],
                ),
            ],
            render: select,
            code: select_code,
            keyboard: vec![
                ("Enter / Space / ↓", "Open"),
                ("↑ / ↓, Home / End", "Move the highlight"),
                ("Type", "Jump to the next match (type-ahead)"),
                ("Enter", "Choose"),
                ("Esc", "Close without changing"),
            ],
            a11y: vec![
                "The trigger is a combobox labelled by the field label and its value; the list is a listbox.",
                "aria-activedescendant follows the highlight, so focus stays on the trigger.",
                "Esc returns without changing the value.",
            ],
            responsive: vec![
                "The list opens upward when the host passes up (no room below).",
                "Long lists scroll inside a 280 px list; on narrow screens the list matches the trigger width.",
            ],
            height: 400,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "slider",
            name: "Slider",
            category: "inputs",
            summary: "One value or a range, with marks, named detents that pull the thumb, a live cost tag, a vertical form and a value bubble.",
            dioxus: "Slider",
            gpui: "SliderEl",
            spec: "slider",
            source: "primitives/forms.rs",
            controls: vec![
                sel(
                    "kind",
                    "Kind",
                    &["single", "range", "detents", "vertical"],
                    "detents",
                ),
                tog("tag", "Cost tag", true),
                tog("show_value", "Show value", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("detents", "Effort with named stops and a cost tag.", &[]),
                var("single", "Temperature with marks.", &[("kind", "single")]),
                var("range", "Two thumbs for a budget.", &[("kind", "range")]),
                var("vertical", "Vertical volume.", &[("kind", "vertical")]),
            ],
            render: slider,
            code: slider_code,
            keyboard: vec![
                ("← / ↓ and → / ↑", "One step"),
                ("Shift + arrows, Page Up / Down", "Ten steps"),
                ("Home / End", "Min / max"),
            ],
            a11y: vec![
                "Each thumb is a role=slider with min, max, now and a formatted aria-valuetext.",
                "Range thumbs are named by thumb_labels (\"Minimum\", \"Maximum\").",
            ],
            responsive: vec![
                "The track fills its container.",
                "Very small tracks keep a 16 px thumb and a 20 px hit height.",
            ],
            height: 220,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "text-field",
            name: "Text field",
            category: "inputs",
            summary: "One input for every form: label, hint and error in place, prefix and suffix, clear, copy and show-password buttons, and a character count with a soft limit.",
            dioxus: "TextField",
            gpui: "TextFieldEl",
            spec: "text-field",
            source: "primitives/forms.rs",
            controls: vec![
                sel(
                    "kind",
                    "Kind",
                    &["text", "email", "search", "number", "password"],
                    "text",
                ),
                txt("hint", "Hint", ""),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("counter", "Counter (50, soft 40)", true),
                tog("clearable", "Clearable", false),
                tog("copyable", "Copyable", false),
                tog("error", "Error", false),
                tog("optional", "Optional", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("counter", "Commit message near its limit.", &[]),
                var(
                    "search",
                    "Search with an icon and clear.",
                    &[
                        ("kind", "search"),
                        ("counter", "false"),
                        ("clearable", "true"),
                    ],
                ),
                var(
                    "prefix-suffix",
                    "Number with a prefix and suffix.",
                    &[
                        ("kind", "number"),
                        ("counter", "false"),
                        ("optional", "true"),
                    ],
                ),
                var(
                    "password",
                    "Show, then hide again after 10 s.",
                    &[
                        ("kind", "password"),
                        ("counter", "false"),
                        ("hint", "Hides again after 10 s"),
                    ],
                ),
                var(
                    "error",
                    "Email with an error in place.",
                    &[("kind", "email"), ("counter", "false"), ("error", "true")],
                ),
            ],
            render: text_field,
            code: text_field_code,
            keyboard: vec![
                ("Esc", "Clear (search and clearable fields)"),
                ("Tab", "Into the clear, copy or reveal buttons"),
            ],
            a11y: vec![
                "Native input with a label; aria-describedby points at the message slot, which is aria-live=polite.",
                "aria-invalid and aria-required reflect the model.",
                "Clear, copy and reveal are named buttons, reachable after the input in tab order.",
            ],
            responsive: vec![
                "Fills its container; long prefixes shrink the input, never overflow.",
                "Pasting over the hard limit keeps the value and turns the counter red.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
    ]
}
