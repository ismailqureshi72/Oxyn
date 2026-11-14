//! Forms (plan #25): tag input, combobox, multi-select, OTP, password field, password strength,
//! inline edit, phone input, filter menu, stepper form, multi-step form, smart form fill.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        combobox::ComboOption,
        filter_menu::FilterGroup,
        stepper::{FieldKind, Step, StepField},
    },
};
use oxyn_dioxus::{
    Combobox, FilterMenu, InlineEdit, MultiSelect, OtpInput, PasswordField, PasswordStrength,
    PhoneInput, SmartFormFill, StepperForm, TagInput,
};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

fn opt_usize(v: &Vals, k: &str) -> Option<usize> {
    let n = v.u(k);
    (n > 0).then_some(n)
}

// ---- Tag input

fn tag_input(v: &Vals, _: ClockMs) -> Element {
    let people = v.s("preset") == "people";
    let value: Vec<String> = if people {
        vec!["sam@acme.dev".into(), "ari@acme.dev".into()]
    } else if v.s("preset") == "context" {
        vec![
            "@maya".into(),
            "src/auth.rs".into(),
            "tool:search".into(),
            "#bug".into(),
        ]
    } else {
        vec!["research".into(), "q4".into(), "agents".into()]
    };
    let error = v.b("error").then(|| "Add at least one owner".to_string());
    rsx! {
        div { class: "oxd-w420",
            TagInput {
                label: v.string("label"),
                value,
                placeholder: v.string("placeholder"),
                hint: v.opt("hint"),
                error,
                optional: v.b("optional"),
                validate: if people { "email".to_string() } else { v.string("validate") },
                normalize: if people { "lower".to_string() } else { v.string("normalize") },
                split_on_space: v.b("split_on_space") || people,
                max: opt_usize(v, "max"),
                avatar: people,
                typed: v.b("typed"),
                suggestions: if v.b("suggestions") { vec!["research".into(), "retrieval".into(), "release".into(), "routing".into(), "red-team".into()] } else { vec![] },
                fold: opt_usize(v, "fold"),
                disabled: v.b("disabled"),
                name: Some("tags".to_string()),
            }
        }
    }
}

fn tag_input_code(v: &Vals) -> String {
    Code::new("TagInput")
        .raw("label", q(v.s("label")))
        .raw("value", "tags()")
        .s("placeholder", v.s("placeholder"), "Add…")
        .opt("hint", v.s("hint"))
        .s("validate", v.s("validate"), "any")
        .s("normalize", v.s("normalize"), "trim")
        .b("split_on_space", v.b("split_on_space"))
        .raw("max", format!("{:?}", opt_usize(v, "max")))
        .b("typed", v.b("typed"))
        .raw("fold", format!("{:?}", opt_usize(v, "fold")))
        .b("optional", v.b("optional"))
        .b("disabled", v.b("disabled"))
        .raw("on_change", "move |t: Vec<String>| tags.set(t)")
        .done()
}

// ---- Combobox

fn people() -> Vec<ComboOption> {
    vec![
        ComboOption::new("maya", "Maya Patel")
            .hint("Design")
            .group("Team")
            .avatar(),
        ComboOption::new("sam", "Sam Okafor")
            .hint("Engineering")
            .group("Team")
            .avatar(),
        ComboOption::new("ari", "Ari Lindqvist")
            .hint("Research")
            .group("Team")
            .avatar(),
        ComboOption::new("jo", "Jo Ellis")
            .hint("Harbor · guest")
            .group("Guests")
            .avatar(),
        ComboOption::new("coder", "Coder")
            .hint("Agent · writes and runs code")
            .group("Agents")
            .icon("code")
            .keywords(&["engineer", "dev"]),
        ComboOption::new("scout", "Scout")
            .hint("Agent · searches the web")
            .group("Agents")
            .icon("globe")
            .keywords(&["research", "browse"]),
        ComboOption::new("scribe", "Scribe")
            .hint("Agent · drafts docs")
            .group("Agents")
            .icon("page"),
    ]
}

fn combobox(v: &Vals, _: ClockMs) -> Element {
    let multiple = v.b("multiple");
    let value: Vec<String> = if multiple {
        vec!["maya".into(), "coder".into()]
    } else {
        vec!["sam".into()]
    };
    rsx! {
        div { class: "oxd-w420 oxd-room-menu-lg",
            Combobox {
                options: people(),
                value,
                multiple,
                label: Some("Assign to".to_string()),
                placeholder: "Search people and agents…",
                creatable: v.b("creatable"),
                create_label: "Invite “…”",
                max_visible_tags: opt_usize(v, "max_visible_tags"),
                default_open: v.b("open"),
                match_mode: v.string("match_mode"),
                max_selected: opt_usize(v, "max_selected"),
                select_all: v.b("select_all"),
                disabled: v.b("disabled"),
            }
        }
    }
}

fn combobox_code(v: &Vals) -> String {
    Code::new("Combobox")
        .pre("let options = vec![")
        .pre("    ComboOption::new(\"maya\", \"Maya Patel\").hint(\"Design\").group(\"Team\").avatar(),")
        .pre("    ComboOption::new(\"coder\", \"Coder\").hint(\"Agent\").group(\"Agents\").icon(\"code\"),")
        .pre("];")
        .raw("options", "options")
        .raw("value", "assignees()")
        .b("multiple", v.b("multiple"))
        .raw("label", "Some(\"Assign to\".into())")
        .b("creatable", v.b("creatable"))
        .raw("max_visible_tags", format!("{:?}", opt_usize(v, "max_visible_tags")))
        .s("match_mode", v.s("match_mode"), "contains")
        .raw("max_selected", format!("{:?}", opt_usize(v, "max_selected")))
        .b("select_all", v.b("select_all"))
        .raw("on_change", "move |ids: Vec<String>| assignees.set(ids)")
        .done()
}

fn tools() -> Vec<ComboOption> {
    [
        "Search", "Browser", "Shell", "Files", "Calendar", "Email", "GitHub",
    ]
    .iter()
    .map(|t| ComboOption::new(&t.to_lowercase(), t))
    .collect()
}

fn multi_select(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w360 oxd-room-menu-lg",
            MultiSelect {
                label: "Tools the agent may use",
                options: tools(),
                value: vec!["search".into(), "files".into(), "github".into()],
                description: v.opt("description"),
                max_visible: v.u("max_visible").max(1),
                select_all: v.b("select_all"),
                max_selected: opt_usize(v, "max_selected"),
                open: v.b("open"),
                disabled: v.b("disabled"),
            }
        }
    }
}

fn multi_select_code(v: &Vals) -> String {
    Code::new("MultiSelect")
        .raw("label", q("Tools the agent may use"))
        .raw("options", "tools")
        .raw("value", "allowed()")
        .opt("description", v.s("description"))
        .raw("max_visible", v.s("max_visible").to_string())
        .b("select_all", v.b("select_all"))
        .raw(
            "max_selected",
            format!("{:?}", opt_usize(v, "max_selected")),
        )
        .b("disabled", v.b("disabled"))
        .raw("on_change", "move |ids: Vec<String>| allowed.set(ids)")
        .done()
}

// ---- OTP

fn otp(v: &Vals, _: ClockMs) -> Element {
    let len = v.u("length").clamp(4, 8);
    let numeric = v.b("numeric");
    let expect = if numeric {
        "482913".chars().cycle().take(len).collect::<String>()
    } else {
        "K7Q2MX".chars().cycle().take(len).collect()
    };
    let hint_code = expect.clone();
    rsx! {
        div { class: "oxd-w420",
            OtpInput {
                length: len,
                separator: if v.b("separator") { Some(0) } else { None },
                numeric,
                status: v.string("status"),
                value: if v.s("status") == "idle" { String::new() } else { expect.clone() },
                align: v.string("align"),
                masked: v.b("masked"),
                attempts: if v.b("attempts") { Some(3) } else { None },
                resend_in: Some(v.u("resend_in") as u32),
                disabled: v.b("disabled"),
                expect: Some(expect),
            }
            p { class: "oxd-note", "Type {hint_code} to verify, or any other code to see the error." }
        }
    }
}

fn otp_code(v: &Vals) -> String {
    Code::new("OtpInput")
        .raw("length", v.s("length").to_string())
        .raw("numeric", if v.b("numeric") { "true" } else { "false" })
        .raw(
            "separator",
            if v.b("separator") { "Some(0)" } else { "None" },
        )
        .raw("status", "status()")
        .s("align", v.s("align"), "start")
        .b("masked", v.b("masked"))
        .raw("attempts", if v.b("attempts") { "Some(3)" } else { "None" })
        .raw("resend_in", format!("Some({})", v.s("resend_in")))
        .raw("on_complete", "move |code: String| verify(code)")
        .raw("on_resend", "move |_| send_code()")
        .done()
}

// ---- Password field / strength

fn password_field(v: &Vals, _: ClockMs) -> Element {
    let error = v
        .b("error")
        .then(|| "That password is incorrect".to_string());
    rsx! {
        div { class: "oxd-w360",
            PasswordField {
                label: "Password",
                value: "correct-horse-7",
                description: v.opt("description"),
                autocomplete: "current-password",
                error,
                caps_warning: v.b("caps_warning"),
                reveal_for: if v.b("reveal_for") { Some(10) } else { None },
                vault: v.b("vault"),
                disabled: v.b("disabled"),
            }
        }
    }
}

fn password_field_code(v: &Vals) -> String {
    Code::new("PasswordField")
        .raw("label", q("Password"))
        .opt("description", v.s("description"))
        .raw("autocomplete", q("current-password"))
        .b("vault", v.b("vault"))
        .raw(
            "reveal_for",
            if v.b("reveal_for") {
                "Some(10)"
            } else {
                "None"
            },
        )
        .raw("on_input", "move |pw: String| password.set(pw)")
        .done()
}

fn password_strength(v: &Vals, _: ClockMs) -> Element {
    let error = v
        .b("error")
        .then(|| "Choose a password you haven't used here".to_string());
    rsx! {
        div { class: "oxd-w360",
            PasswordStrength {
                label: "New password",
                value: v.string("value"),
                policy: v.string("policy"),
                variant: v.string("variant"),
                entropy: v.b("entropy"),
                short_length: v.u("short_length").max(1),
                error,
                revealed: v.b("revealed"),
            }
        }
    }
}

fn password_strength_code(v: &Vals) -> String {
    Code::new("PasswordStrength")
        .raw("label", q("New password"))
        .s("policy", v.s("policy"), "basic")
        .s("variant", v.s("variant"), "checklist")
        .b("entropy", v.b("entropy"))
        .raw("short_length", v.s("short_length").to_string())
        .raw(
            "on_change",
            "move |(pw, level): (String, String)| draft.set(pw)",
        )
        .done()
}

// ---- Inline edit

fn inline_edit(v: &Vals, _: ClockMs) -> Element {
    let body = v.s("variant") == "body";
    rsx! {
        div { class: "oxd-w420",
            InlineEdit {
                value: if body { "Fix the refresh-token race in the auth middleware and add a regression test.".to_string() } else { v.string("value") },
                label: if body { "Task description".to_string() } else { "Project name".to_string() },
                variant: v.string("variant"),
                multiline: body || v.b("multiline"),
                max: opt_usize(v, "max"),
                fail: v.b("fail"),
                latency_ms: v.u("latency_ms") as u32,
                suggestions: if v.b("suggestions") { vec!["Atlas · auth hardening".into(), "Refresh race fix".into()] } else { vec![] },
                conflict: v.b("conflict").then(|| "Atlas (renamed by Coder)".to_string()),
            }
        }
    }
}

fn inline_edit_code(v: &Vals) -> String {
    Code::new("InlineEdit")
        .raw("value", "name()")
        .raw("label", q("Project name"))
        .s("variant", v.s("variant"), "title")
        .b("multiline", v.b("multiline"))
        .raw("max", format!("{:?}", opt_usize(v, "max")))
        .raw("on_save", "move |next: String| spawn(save_name(next))")
        .done()
}

// ---- Phone input

fn phone(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w360 oxd-room-menu-lg",
            PhoneInput {
                label: "Phone",
                value: v.string("value"),
                default_country: v.string("country"),
                description: v.opt("description"),
                masked: v.b("masked"),
                open: v.b("open"),
                validate: v.b("validate"),
                required: v.b("required"),
                disabled: v.b("disabled"),
                name: Some("phone".to_string()),
            }
        }
    }
}

fn phone_code(v: &Vals) -> String {
    Code::new("PhoneInput")
        .raw("label", q("Phone"))
        .raw("value", "phone()")
        .s("default_country", v.s("country"), "US")
        .opt("description", v.s("description"))
        .b("masked", v.b("masked"))
        .b("required", v.b("required"))
        .raw("on_change", "move |e164: String| phone.set(e164)")
        .done()
}

// ---- Filter menu

fn filter_groups(counts: bool) -> Vec<FilterGroup> {
    let mut g = vec![
        FilterGroup::new(
            "status",
            "Status",
            &[
                ("running", "Running"),
                ("waiting", "Waiting on you"),
                ("failed", "Failed"),
                ("done", "Done"),
            ],
        ),
        FilterGroup::new(
            "agent",
            "Agent",
            &[("coder", "Coder"), ("scout", "Scout"), ("scribe", "Scribe")],
        ),
        FilterGroup::new(
            "repo",
            "Repository",
            &[
                ("web", "acme/web"),
                ("api", "acme/api"),
                ("infra", "acme/infra"),
            ],
        ),
    ];
    if counts {
        g[0] = g[0].clone().counts(&[4, 2, 1, 38]);
        g[1] = g[1].clone().counts(&[21, 14, 10]);
        g[2] = g[2].clone().counts(&[25, 17, 3]);
    }
    g
}

fn filter_menu(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-w420",
            FilterMenu {
                groups: filter_groups(v.b("counts")),
                value: vec![("status".into(), "waiting".into()), ("agent".into(), "coder".into())],
                title: v.string("title"),
                hints: v.b("hints"),
                logic: v.b("logic"),
                presets: v.b("presets"),
                summary: v.b("summary"),
            }
        }
    }
}

fn filter_menu_code(v: &Vals) -> String {
    Code::new("FilterMenu")
        .pre("let groups = vec![")
        .pre("    FilterGroup::new(\"status\", \"Status\", &[(\"running\", \"Running\"), (\"waiting\", \"Waiting on you\")]),")
        .pre("    FilterGroup::new(\"agent\", \"Agent\", &[(\"coder\", \"Coder\"), (\"scout\", \"Scout\")]),")
        .pre("];")
        .raw("groups", "groups")
        .s("title", v.s("title"), "Filters")
        .b("logic", v.b("logic"))
        .b("presets", v.b("presets"))
        .b("summary", v.b("summary"))
        .raw("on_change", "move |f: Vec<(String, String)>| filters.set(f)")
        .done()
}

// ---- Stepper / multi-step

fn onboarding(conditional: bool) -> Vec<Step> {
    let mut team = Step::new(
        "team",
        "Team",
        "Invite your team",
        vec![
            StepField::text("invites", "Invite by email")
                .kind(FieldKind::Tags { max: Some(5) })
                .optional()
                .hint("Paste a list; comma or Enter adds"),
        ],
    );
    if conditional {
        team.only_if = Some(("plan".into(), "team".into()));
    }
    let plan = Step::new(
        "plan",
        "Plan",
        "Pick a plan",
        vec![
            StepField::text("plan", "Plan").kind(FieldKind::Choices(vec![
                (
                    "free".into(),
                    "Free".into(),
                    "$0".into(),
                    "One seat, 50 runs a month".into(),
                ),
                (
                    "team".into(),
                    "Team".into(),
                    "$12 / seat".into(),
                    "Shared agents and approvals".into(),
                ),
                (
                    "scale".into(),
                    "Scale".into(),
                    "Talk to us".into(),
                    "SSO, audit log, SLAs".into(),
                ),
            ])),
        ],
    );
    let ws = Step::new(
        "ws",
        "Workspace",
        "Name your workspace",
        vec![
            StepField::text("name", "Workspace name"),
            StepField::text("url", "URL")
                .prefix("acme.app/")
                .hint("Lowercase letters and dashes"),
        ],
    )
    .describe("You can change both later in Settings.");
    let mut v = vec![ws];
    if conditional {
        v.push(plan);
        v.push(team);
    } else {
        v.push(team);
        v.push(plan);
    }
    v.push(Step::review("Review"));
    v
}

fn stepper(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        StepperForm {
            steps: onboarding(v.b("conditional")),
            style: v.string("style"),
            initial_step: v.u("initial_step"),
            fail: v.b("fail"),
            drafts: v.b("drafts"),
            initial_values: if v.u("initial_step") > 0 { vec![("name".into(), "Acme".into()), ("url".into(), "acme".into()), ("invites".into(), "sam@acme.dev, ari@acme.dev".into()), ("plan".into(), "team".into())] } else { vec![] },
        }
    }
}

fn stepper_code(v: &Vals) -> String {
    Code::new("StepperForm")
        .pre("let steps = vec![")
        .pre("    Step::new(\"ws\", \"Workspace\", \"Name your workspace\", vec![StepField::text(\"name\", \"Workspace name\")]),")
        .pre("    Step::new(\"team\", \"Team\", \"Invite your team\", vec![StepField::text(\"invites\", \"Invite\").kind(FieldKind::Tags { max: Some(5) })]),")
        .pre("    Step::review(\"Review\"),")
        .pre("];")
        .raw("steps", "steps")
        .s("style", v.s("style"), "stepper")
        .b("drafts", v.b("drafts"))
        .raw("on_complete", "move |values| spawn(create_workspace(values))")
        .done()
}

fn multistep(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        StepperForm {
            steps: onboarding(false),
            style: "minimal",
            submit_label: v.string("complete_label"),
            initial_step: v.u("initial_step"),
            drafts: v.b("drafts"),
            initial_values: if v.u("initial_step") > 0 { vec![("name".into(), "Acme".into()), ("url".into(), "acme".into())] } else { vec![] },
        }
    }
}

fn multistep_code(v: &Vals) -> String {
    Code::new("StepperForm")
        .raw("steps", "steps")
        .raw("style", q("minimal"))
        .s("submit_label", v.s("complete_label"), "Finish")
        .raw("initial_step", v.s("initial_step").to_string())
        .raw("on_complete", "move |values| submit(values)")
        .done()
}

// ---- Smart form fill

fn smart_fill(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        SmartFormFill {
            prefill: v.b("prefill"),
            review_threshold: v.u("review_threshold") as u8,
            gate: v.b("gate"),
            model: v.string("model"),
        }
    }
}

fn smart_fill_code(v: &Vals) -> String {
    Code::new("SmartFormFill")
        .raw("review_threshold", v.s("review_threshold").to_string())
        .b("gate", v.b("gate"))
        .s("model", v.s("model"), "Haiku")
        .raw("on_submit", "move |contact| spawn(save_contact(contact))")
        .done()
}

/// Demos.
pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "tag-input",
            name: "Tag input",
            category: "forms",
            summary: "Several values in one field as chips: paste a whole list, a check per value, duplicates flash, Backspace and arrows to edit.",
            dioxus: "TagInput",
            gpui: "TagInputEl",
            spec: "tag-input",
            source: "primitives/fields.rs",
            controls: vec![
                txt("label", "Label", "Labels"),
                sel(
                    "preset",
                    "Sample",
                    &["labels", "people", "context"],
                    "labels",
                ),
                txt("placeholder", "Placeholder", "Add…"),
                txt("hint", "Hint", "Enter or comma adds; paste a list"),
                sel(
                    "validate",
                    "Check",
                    &["any", "email", "domain", "max:12"],
                    "any",
                ),
                sel("normalize", "Clean", &["trim", "lower", "slug"], "trim"),
                rng("max", "Max (0 = none)", 0.0, 10.0, 1.0, "8"),
                rng("fold", "Fold after (0 = wrap)", 0.0, 6.0, 1.0, "0"),
                tog("split_on_space", "Split on space", false),
                tog("typed", "Typed chips", false),
                tog("suggestions", "Suggestions", false),
                tog("error", "Error", false),
                tog("optional", "Optional", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("labels", "Short labels with a counter.", &[]),
                var(
                    "people",
                    "Emails with initials, lowercased and checked.",
                    &[
                        ("preset", "people"),
                        ("label", "Invite"),
                        ("hint", "Paste a list of emails"),
                        ("max", "0"),
                    ],
                ),
                var(
                    "typed-context",
                    "Person, file, tool and label chips (Oxyn plus).",
                    &[
                        ("preset", "context"),
                        ("label", "Context"),
                        ("typed", "true"),
                        ("max", "0"),
                    ],
                ),
                var(
                    "suggestions",
                    "Agent suggestions as you type (Oxyn plus).",
                    &[("suggestions", "true"), ("hint", "Type “re”")],
                ),
                var(
                    "folded",
                    "Folds past three into +N (Oxyn plus).",
                    &[("fold", "2")],
                ),
                var("error", "Invalid from outside.", &[("error", "true")]),
            ],
            render: tag_input,
            code: tag_input_code,
            keyboard: vec![
                (
                    "Enter / ,",
                    "Turn what you typed into a chip (Tab does too, then moves on)",
                ),
                ("Space", "Also adds, with split_on_space"),
                (
                    "⌫",
                    "On an empty field: first press marks the last chip, the second removes it",
                ),
                (
                    "← / →",
                    "Walk the chips; Backspace or Delete removes the one in focus",
                ),
                (
                    "Enter on a chip",
                    "Moves it back into the field to edit (Oxyn plus)",
                ),
                ("Esc", "Clears the pick, then what you were typing"),
            ],
            a11y: vec![
                "A real text input with a label; chips sit before it, each with a labelled Remove button.",
                "Adds, removals, duplicates and limits are announced in a polite live region.",
                "A message sets aria-invalid and is linked with aria-describedby.",
                "Chips are reached with ← from the start of the field, so Tab moves straight on.",
            ],
            responsive: vec![
                "Chips wrap; after about five lines the field scrolls inside.",
                "Long values end in … with the full text on hover.",
                "Touch: taller chips, a bigger × and 16 px input text (no iOS zoom).",
            ],
            height: 260,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "combobox",
            name: "Combobox",
            category: "forms",
            summary: "Pick one or many from a searchable, grouped list: chips with avatars, highlighted matches, create-on-the-fly and full keyboard support.",
            dioxus: "Combobox",
            gpui: "ComboboxEl",
            spec: "combobox",
            source: "primitives/fields.rs",
            controls: vec![
                tog("multiple", "Multiple", true),
                tog("open", "Open", true),
                tog("creatable", "Creatable (Invite “…”)", true),
                rng(
                    "max_visible_tags",
                    "Chips before +N (0 = all)",
                    0.0,
                    5.0,
                    1.0,
                    "0",
                ),
                sel(
                    "match_mode",
                    "Match",
                    &["contains", "prefix", "fuzzy"],
                    "contains",
                ),
                rng("max_selected", "Max picks (0 = none)", 0.0, 5.0, 1.0, "0"),
                tog("select_all", "Select all", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("multiple", "Chips with avatars, grouped list.", &[]),
                var(
                    "single",
                    "One value; the list closes after a pick.",
                    &[("multiple", "false"), ("creatable", "false")],
                ),
                var(
                    "overflow",
                    "+N past two chips.",
                    &[("max_visible_tags", "1"), ("open", "false")],
                ),
                var(
                    "fuzzy",
                    "Letters in order match (Oxyn plus).",
                    &[("match_mode", "fuzzy")],
                ),
                var(
                    "capped",
                    "Up to three, with Select all (Oxyn plus).",
                    &[("max_selected", "3"), ("select_all", "true")],
                ),
            ],
            render: combobox,
            code: combobox_code,
            keyboard: vec![
                ("↑ / ↓", "Open the list, then move between rows"),
                ("Home / End", "First / last row"),
                ("↵", "Pick or unpick the active row (or create)"),
                (
                    "⌫",
                    "In the empty field: mark the last chip, press again to remove it",
                ),
                ("Esc", "Close the list; when closed, clear the typed text"),
                ("Tab", "Close the list and move on"),
            ],
            a11y: vec![
                "ARIA combobox: the input has role=combobox with aria-expanded, aria-controls and aria-activedescendant; the list is a grouped listbox.",
                "Picked rows carry aria-selected; the listbox is aria-multiselectable in multiple mode.",
                "Picks and removals are announced politely with the count.",
                "Focus stays in the input while clicking rows and chip × buttons.",
            ],
            responsive: vec![
                "The list is as wide as the field.",
                "Chips wrap and long names truncate, down to 320 px.",
                "Touch: 44 px rows, bigger chip buttons and 16 px input text.",
            ],
            height: 460,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "multi-select",
            name: "Multi-select",
            category: "forms",
            summary: "Several values in a compact field: chips that grow in, a rolling +N, drawn checks in the list and a Clear button.",
            dioxus: "MultiSelect",
            gpui: "ComboboxEl",
            spec: "multi-select",
            source: "primitives/fields.rs",
            controls: vec![
                rng("max_visible", "Chips before +N", 1.0, 4.0, 1.0, "2"),
                txt(
                    "description",
                    "Description",
                    "The agent asks before using anything else",
                ),
                tog("open", "Open", true),
                tog("select_all", "Select all", false),
                rng("max_selected", "Max picks (0 = none)", 0.0, 6.0, 1.0, "0"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("default", "Three picked, two chips and +1.", &[]),
                var("closed", "Closed trigger.", &[("open", "false")]),
                var(
                    "select-all",
                    "Select all row (Oxyn plus).",
                    &[("select_all", "true")],
                ),
                var(
                    "capped",
                    "Max with a reason (Oxyn plus).",
                    &[("max_selected", "4")],
                ),
            ],
            render: multi_select,
            code: multi_select_code,
            keyboard: vec![
                (
                    "Enter / Space",
                    "Open or close; Enter toggles the active option while open",
                ),
                ("↓ / ↑", "Open, then move through enabled options (wraps)"),
                ("Home / End", "First / last option"),
                ("Esc", "Close"),
            ],
            a11y: vec![
                "Trigger: aria-haspopup=listbox, aria-expanded, labelled by the label and a hidden list of the picked labels.",
                "The menu is role=listbox with aria-multiselectable; options carry aria-selected and aria-disabled.",
                "Clear is labelled “Clear selections”; chips are aria-hidden in favour of the spoken summary.",
            ],
            responsive: vec![
                "Chips cap at 9rem and ellipsize; max_visible keeps the trigger on one line.",
                "The menu spans the field; keep lists short on small screens.",
            ],
            height: 420,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "otp-input",
            name: "OTP input",
            category: "forms",
            summary: "The verification-code field: a gliding focus ring, paste and SMS autofill, a calm shake on errors and a morph into Verified.",
            dioxus: "OtpInput",
            gpui: "OtpEl",
            spec: "otp-input",
            source: "primitives/fields.rs",
            controls: vec![
                rng("length", "Length", 4.0, 8.0, 1.0, "6"),
                sel(
                    "status",
                    "Status",
                    &["idle", "verifying", "error", "success"],
                    "idle",
                ),
                tog("separator", "Separator", true),
                tog("numeric", "Digits only", true),
                sel("align", "Align", &["start", "center"], "start"),
                rng("resend_in", "Resend in (s)", 0.0, 60.0, 5.0, "30"),
                tog("masked", "Masked", false),
                tog("attempts", "3 tries", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("idle", "Six digits with a dash in the middle.", &[]),
                var(
                    "verifying",
                    "Light passes over the locked boxes.",
                    &[("status", "verifying")],
                ),
                var(
                    "error",
                    "Shake, red edges, then the code clears.",
                    &[("status", "error"), ("attempts", "true")],
                ),
                var(
                    "success",
                    "Boxes merge into the Verified pill.",
                    &[("status", "success")],
                ),
                var(
                    "alphanumeric",
                    "Five letters or digits, masked (Oxyn plus).",
                    &[
                        ("length", "5"),
                        ("numeric", "false"),
                        ("separator", "false"),
                        ("masked", "true"),
                    ],
                ),
            ],
            render: otp,
            code: otp_code,
            keyboard: vec![
                (
                    "0–9",
                    "Fill the active box and move on; typing on a filled box replaces it",
                ),
                ("⌫", "Clear the box and go back one"),
                (
                    "← / →",
                    "Move between the filled boxes and the next empty one",
                ),
                ("Home / End", "First box / the next empty one"),
                (
                    "⌘ / Ctrl + V",
                    "Paste a whole code; spaces and dashes are ignored",
                ),
            ],
            a11y: vec![
                "One real input carries the whole code: “Verification code, 6 digits”.",
                "autocomplete=one-time-code and inputmode=numeric for SMS autofill and the number pad.",
                "Errors set aria-invalid and point to the message; verifying, errors, success and resend are announced politely.",
            ],
            responsive: vec![
                "The boxes shrink to their container: six with a separator fit a 320 px phone.",
                "The input uses 16 px text so iOS doesn't zoom.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "password-field",
            name: "Password field",
            category: "forms",
            summary: "A password input whose reveal button slashes the eye with a drawn line, with a Caps Lock warning and a reveal that hides itself again.",
            dioxus: "PasswordField",
            gpui: "PasswordFieldEl",
            spec: "password-field",
            source: "primitives/fields.rs",
            controls: vec![
                txt("description", "Description", "At least 12 characters"),
                tog("caps_warning", "Caps Lock warning", true),
                tog("reveal_for", "Hide again after 10 s", false),
                tog("vault", "Use saved password", false),
                tog("error", "Error", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("default", "Label, description and reveal.", &[]),
                var(
                    "timed",
                    "Hides again after 10 s (Oxyn plus).",
                    &[("reveal_for", "true")],
                ),
                var(
                    "vault",
                    "Fill from the vault (Oxyn plus).",
                    &[("vault", "true")],
                ),
                var(
                    "error",
                    "Wrong password.",
                    &[("error", "true"), ("description", "")],
                ),
            ],
            render: password_field,
            code: password_field_code,
            keyboard: vec![
                ("Enter / Space", "On the toggle: show or hide the password"),
                ("Tab", "Input, then the vault button, then the toggle"),
            ],
            a11y: vec![
                "The toggle is a native button with aria-pressed, aria-controls and a label that switches Show / Hide password.",
                "The description (and the Caps Lock warning) is linked with aria-describedby.",
                "autocomplete current-password / new-password so password managers work.",
            ],
            responsive: vec![
                "Fills its column; the reveal button sits inside the shell and never wraps.",
                "Hover styles only on fine pointers.",
            ],
            height: 180,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "password-strength",
            name: "Password strength",
            category: "forms",
            summary: "A new-password field with a four-segment meter, a rule checklist with remaining counts and a strength word that morphs up or down.",
            dioxus: "PasswordStrength",
            gpui: "PasswordStrengthEl",
            spec: "password-strength",
            source: "primitives/fields.rs",
            controls: vec![
                txt("value", "Value", "agent-Ops2"),
                sel(
                    "policy",
                    "Policy",
                    &["basic", "nist", "enterprise"],
                    "basic",
                ),
                sel("variant", "Variant", &["checklist", "meter"], "checklist"),
                rng("short_length", "Too short below", 4.0, 16.0, 1.0, "8"),
                tog("entropy", "Entropy", false),
                tog("revealed", "Revealed", true),
                tog("error", "Error", false),
            ],
            variants: vec![
                var("checklist", "Meter and rules.", &[]),
                var("too-short", "Holds at the first step.", &[("value", "abc")]),
                var("strong", "All rules met.", &[("value", "Lantern#River42")]),
                var(
                    "nist",
                    "Length and not common, no composition (Oxyn plus).",
                    &[("policy", "nist"), ("value", "password1")],
                ),
                var(
                    "meter-only",
                    "Meter with entropy (Oxyn plus).",
                    &[("variant", "meter"), ("entropy", "true")],
                ),
            ],
            render: password_strength,
            code: password_strength_code,
            keyboard: vec![("Enter / Space", "On the reveal button: show or hide")],
            a11y: vec![
                "The meter is role=meter, aria-valuenow 0–4 and aria-valuetext set to the word.",
                "The rule list is linked to the input with aria-describedby, each rule says Met / Not met.",
                "A status region summarises strength and rules met.",
            ],
            responsive: vec![
                "The four segments share the row and shrink with the column.",
                "Coarse pointers get 16 px input text.",
            ],
            height: 300,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "inline-edit",
            name: "Inline edit",
            category: "forms",
            summary: "Rename in place: the text becomes a field without moving, saves optimistically with a drawn check, and rolls back with Retry when the save fails.",
            dioxus: "InlineEdit",
            gpui: "InlineEditEl",
            spec: "inline-edit",
            source: "primitives/fields.rs",
            controls: vec![
                txt("value", "Value", "Atlas"),
                sel("variant", "Variant", &["title", "body"], "title"),
                tog("multiline", "Multiline", false),
                rng("max", "Max characters (0 = none)", 0.0, 60.0, 5.0, "40"),
                rng("latency_ms", "Save latency (ms)", 0.0, 2000.0, 100.0, "600"),
                tog("fail", "Next save fails", false),
                tog("suggestions", "Agent suggestions", false),
                tog("conflict", "Agent renames mid-edit", false),
            ],
            variants: vec![
                var("title", "A project name.", &[]),
                var("body", "A description that grows.", &[("variant", "body")]),
                var(
                    "rollback",
                    "The save fails and rolls back.",
                    &[("fail", "true")],
                ),
                var(
                    "suggestions",
                    "Rename suggestions (Oxyn plus).",
                    &[("suggestions", "true")],
                ),
                var(
                    "conflict",
                    "The agent edits too (Oxyn plus).",
                    &[("conflict", "true")],
                ),
            ],
            render: inline_edit,
            code: inline_edit_code,
            keyboard: vec![
                ("Enter / Space", "On the text: start editing"),
                ("Enter", "Save (Shift+Enter breaks a line in multiline)"),
                ("Esc", "Cancel and roll the text back"),
                ("Tab", "Leaving saves, like a rename"),
            ],
            a11y: vec![
                "The resting text is a button named “Project name: Atlas”.",
                "The field has aria-label and aria-invalid; errors are linked with aria-describedby.",
                "Saving and results go to a status region; Save and Cancel are labelled.",
            ],
            responsive: vec![
                "Text wraps and reserves room at the end for Save and Cancel.",
                "Multiline grows in height as it wraps; on touch a tap starts editing.",
            ],
            height: 200,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "phone-input",
            name: "Phone input",
            category: "forms",
            summary: "A phone field with a searchable country picker, formatting as you type and E.164 out; paste an international number and the country follows.",
            dioxus: "PhoneInput",
            gpui: "PhoneInputEl",
            spec: "phone-input",
            source: "primitives/fields.rs",
            controls: vec![
                txt("value", "Value (E.164)", "+14155550132"),
                sel(
                    "country",
                    "Default country",
                    &["US", "GB", "PK", "IN", "DE", "BR"],
                    "US",
                ),
                txt(
                    "description",
                    "Description",
                    "We text a code to this number",
                ),
                tog("open", "Picker open", false),
                tog("masked", "Masked", false),
                tog("validate", "Validate after blur", true),
                tog("required", "Required", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("us", "Formatted with a valid check.", &[]),
                var(
                    "picker",
                    "The country picker open.",
                    &[("open", "true"), ("value", "")],
                ),
                var(
                    "uk-trunk",
                    "A UK number keeps the typed 0.",
                    &[("value", "+447400123456"), ("country", "GB")],
                ),
                var(
                    "masked",
                    "All but the last four masked (Oxyn plus).",
                    &[("masked", "true")],
                ),
            ],
            render: phone,
            code: phone_code,
            keyboard: vec![
                (
                    "+ (in the number)",
                    "Open the picker with a calling-code search",
                ),
                ("↓ / ↑ (country button)", "Open the picker"),
                (
                    "A letter (country button)",
                    "Open the picker searching for it",
                ),
                ("↓ / ↑ / Page Down / Page Up", "Move through countries"),
                (
                    "Home / End",
                    "First / last country when the search is empty",
                ),
                ("Enter", "Pick and return to the number"),
                ("Esc", "Close the picker"),
            ],
            a11y: vec![
                "The search is a combobox with aria-activedescendant over a listbox grouped into Suggested and All countries.",
                "The country button reads “Country, United States +1”; picks and pasted country changes are announced.",
                "type=tel, autocomplete=tel; errors show after blur with role=alert.",
            ],
            responsive: vec![
                "The list is as wide as the field up to 340 px.",
                "inputmode=tel opens the phone keypad.",
            ],
            height: 420,
            align: Align::Start,
            live: false,
        },
        Demo {
            slug: "filter-menu",
            name: "Filter menu",
            category: "forms",
            summary: "Grouped filters with removable chips, collapsible checkbox groups, Clear all and full keyboard control.",
            dioxus: "FilterMenu",
            gpui: "FilterMenuEl",
            spec: "filter-menu",
            source: "primitives/flows.rs",
            controls: vec![
                txt("title", "Title", "Filters"),
                tog("hints", "Keyboard hints", true),
                tog("counts", "Counts", false),
                tog("logic", "Any / All", false),
                tog("presets", "Saved views", false),
                tog("summary", "Summary", false),
            ],
            variants: vec![
                var("default", "Two filters on, first group open.", &[]),
                var(
                    "counts",
                    "Result counts per option (Oxyn plus).",
                    &[("counts", "true")],
                ),
                var(
                    "logic",
                    "Any / All per group with a summary (Oxyn plus).",
                    &[("logic", "true"), ("summary", "true")],
                ),
                var("views", "Saved views (Oxyn plus).", &[("presets", "true")]),
            ],
            render: filter_menu,
            code: filter_menu_code,
            keyboard: vec![
                (
                    "↑ / ↓",
                    "Move through group headers and the options of open groups",
                ),
                (
                    "Enter / Space",
                    "Pick or unpick an option, or open and close a group",
                ),
                (
                    "→ / ←",
                    "Open a group, or close it and jump back to its header",
                ),
                ("Home / End", "First / last row"),
            ],
            a11y: vec![
                "Group headers carry aria-expanded; options are role=checkbox with aria-checked.",
                "One tab stop for the list (aria-activedescendant), so Tab moves past it in one step.",
                "Each chip has a remove button named for the filter; changes are announced.",
            ],
            responsive: vec!["Fills its container up to 420 px; chips wrap."],
            height: 440,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "stepper-form",
            name: "Stepper form",
            category: "forms",
            summary: "A multi-step setup flow: steps that fill and check off, sliding steps, inline validation, chip and plan fields, a review with Edit links and a success state.",
            dioxus: "StepperForm",
            gpui: "StepperEl",
            spec: "stepper-form",
            source: "primitives/flows.rs",
            controls: vec![
                sel(
                    "style",
                    "Style",
                    &["stepper", "minimal", "vertical"],
                    "stepper",
                ),
                rng("initial_step", "Start at step", 0.0, 3.0, 1.0, "0"),
                tog("fail", "Submit fails once", false),
                tog("drafts", "Save draft", false),
                tog("conditional", "Team step only on Team plan", false),
            ],
            variants: vec![
                var("stepper", "Numbered steps.", &[]),
                var("review", "At the review.", &[("initial_step", "3")]),
                var(
                    "vertical",
                    "A vertical rail (Oxyn plus).",
                    &[("style", "vertical")],
                ),
                var(
                    "conditional",
                    "A step that shows only for one plan (Oxyn plus).",
                    &[("conditional", "true")],
                ),
                var(
                    "error",
                    "The submit fails next to the button.",
                    &[("initial_step", "3"), ("fail", "true")],
                ),
            ],
            render: stepper,
            code: stepper_code,
            keyboard: vec![
                ("↵", "Continue (in a chip field: add the entry first)"),
                ("Tab", "Fields, then Back and Continue"),
                (
                    "← / →",
                    "In the plan cards: move and select; ↵ picks and continues",
                ),
                (
                    ", / ⌫",
                    "In a chip field: comma adds, Backspace removes the last chip",
                ),
            ],
            a11y: vec![
                "The steps are an ordered list with aria-current=step; finished steps are buttons that jump back.",
                "Below 420 px the list stays for screen readers while “Step 2 of 4” shows.",
                "After a failed Continue the first invalid field shakes; errors are announced politely.",
                "Continue looks dimmed while invalid but still answers, so everyone hears what's missing.",
            ],
            responsive: vec![
                "Up to 520 px wide, down to 320 px; plan cards stack and review rows wrap below 420 px.",
                "Touch: 16 px fields and 40 px buttons.",
            ],
            height: 520,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "multi-step-form",
            name: "Multi-step form",
            category: "forms",
            summary: "One decision at a time with a rolling step count, a progress bar, sliding steps and a submit button whose width follows its label.",
            dioxus: "StepperForm",
            gpui: "StepperEl",
            spec: "multi-step-form",
            source: "primitives/flows.rs",
            controls: vec![
                txt("complete_label", "Complete label", "Create workspace"),
                rng("initial_step", "Start at step", 0.0, 3.0, 1.0, "0"),
                tog("drafts", "Save draft", false),
            ],
            variants: vec![
                var("first", "Step 1 of 4.", &[]),
                var(
                    "middle",
                    "Step 2 with values kept.",
                    &[("initial_step", "1")],
                ),
                var(
                    "drafts",
                    "Save and resume (Oxyn plus).",
                    &[("drafts", "true")],
                ),
            ],
            render: multistep,
            code: multistep_code,
            keyboard: vec![
                ("Enter", "Next / submit"),
                ("Tab", "Through the step's fields, then Back and Next"),
            ],
            a11y: vec![
                "Each step is a fieldset with a legend; focus moves to the new step's first field.",
                "A visually hidden polite line announces “Step 2 of 4”.",
                "The success state is role=status.",
            ],
            responsive: vec!["Below 420 px the header stacks and padding tightens."],
            height: 460,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "smart-form-fill",
            name: "Smart form fill",
            category: "forms",
            summary: "Paste a signature or messy notes and the agent fills a contact form field by field, with confidence, source highlights, review and Undo.",
            dioxus: "SmartFormFill",
            gpui: "SmartFillEl",
            spec: "smart-form-fill",
            source: "primitives/flows.rs",
            controls: vec![
                tog("prefill", "Start filled", true),
                rng(
                    "review_threshold",
                    "Review below (%)",
                    50.0,
                    99.0,
                    1.0,
                    "80",
                ),
                tog("gate", "Threshold control", false),
                txt("model", "Model", "Haiku"),
            ],
            variants: vec![
                var(
                    "filled",
                    "A signature, filled with one field to review.",
                    &[],
                ),
                var(
                    "empty",
                    "Waiting for a paste; try a sample.",
                    &[("prefill", "false")],
                ),
                var(
                    "strict",
                    "A stricter gate (Oxyn plus).",
                    &[("review_threshold", "93"), ("gate", "true")],
                ),
            ],
            render: smart_fill,
            code: smart_fill_code,
            keyboard: vec![
                (
                    "⌘ / Ctrl + V",
                    "Paste into the text area; the read starts right away",
                ),
                ("⌘ / Ctrl + Enter", "Read typed text"),
                ("Esc", "Stop a read; nothing changes"),
                (
                    "Tab",
                    "Text area, Try sample, every field, the check buttons, Undo, Save",
                ),
            ],
            a11y: vec![
                "A polite region announces each step: reading, how many filled, which need review, undo and save.",
                "Each filled input has an aria-description with its confidence and reason; review fields are aria-invalid.",
                "Status is never colour alone: a dot plus a percentage or a word (Review, Checked, Edited).",
            ],
            responsive: vec![
                "Side by side from a 720 px container; below that the paste well stacks on top.",
                "Under 440 px every field takes the full row.",
            ],
            height: 520,
            align: Align::Center,
            live: false,
        },
    ]
}
