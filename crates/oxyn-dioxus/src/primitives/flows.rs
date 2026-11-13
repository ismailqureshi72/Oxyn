//! Flows (plan #25): `FilterMenu`, `StepperForm`, `DecisionCard`, `ApprovalQuestions`,
//! `SmartFormFill`.
//!
//! `ApprovalQuestions` ports Beautiful UI's Approval Card (MIT, © 2026 Shane Levine): paged
//! questions with an odometer counter, auto-advance on single choice, a custom answer row, Skip,
//! Continue and Send. The rest are rebuilt from the affectUI and Arc UI docs (no source used).

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        filter_menu::{FilterGroup, FilterKey, FilterMenu as FilterModel, FilterRow, Logic},
        questions::{
            Answer, DeckEvent, DeckKey, DeckMode, DeckPhase, Question, QuestionDeck, QuestionKind,
        },
        smart_fill::{
            FieldMark, FillField, FillKind, FillPhase, SmartFill, contact_fields, extract,
        },
        stepper::{FieldKind, FieldValue, Step, Stepper, StepperPhase, StepperStyle},
    },
};

use super::{buttons::Spin, cls, fields::TagInput};
use crate::composer::icons::Icon;

fn tick_svg() -> Element {
    rsx! { svg { view_box: "0 0 16 16", width: "12", height: "12", "aria-hidden": "true", path { d: "M3.5 8.5 6.5 11.5 12.5 4.5", path_length: "1" } } }
}

// ---------------------------------------------------------------------------------------------
// Filter menu

/// Filter menu: grouped filters with removable chips, collapsible checkbox groups, Clear all and
/// one roving tab stop (↑ ↓ move, Enter / Space pick or open, → opens, ← closes and jumps to the
/// header, Home / End). Oxyn plus: AND / OR per group, saved views, counts per option and a
/// plain-language summary.
#[component]
pub fn FilterMenu(
    /// Groups.
    groups: Vec<FilterGroup>,
    /// Starting filters: `(group id, option id)`.
    #[props(default)]
    value: Vec<(String, String)>,
    /// Groups that start open (default: the first).
    #[props(default)]
    default_open: Option<Vec<String>>,
    /// Heading.
    #[props(default = "Filters".to_string())]
    title: String,
    /// Keyboard hints on the tray.
    #[props(default = true)]
    hints: bool,
    /// Oxyn plus: Any / All per group.
    #[props(default)]
    logic: bool,
    /// Oxyn plus: saved views.
    #[props(default)]
    presets: bool,
    /// Oxyn plus: a one-line summary under the chips.
    #[props(default)]
    summary: bool,
    /// Changed: every active `(group, option)`.
    #[props(default)]
    on_change: EventHandler<Vec<(String, String)>>,
) -> Element {
    let mut m = use_signal(|| {
        let mut f = FilterModel::new(groups.clone());
        for (g, o) in &value {
            f = f.with(g, o);
        }
        if let Some(open) = &default_open {
            for (i, g) in f.groups.clone().iter().enumerate() {
                f.open[i] = open.contains(&g.id);
            }
        }
        if presets {
            f.presets = vec![("Needs me".into(), f.value.clone())];
        }
        f
    });
    let mut saving = use_signal(|| 0u32);
    let st = m.read().clone();
    let rows = st.rows();
    let emit = move || {
        let s = m.peek().clone();
        let v = s
            .value
            .iter()
            .enumerate()
            .flat_map(|(g, vs)| {
                let gid = s.groups[g].id.clone();
                vs.iter().map(move |o| (gid.clone(), o.clone()))
            })
            .collect::<Vec<_>>();
        on_change.call(v);
    };
    let chips = st.chips();
    rsx! {
        div { class: "oxyn-filter",
            div { class: "fh",
                span { class: "ft", "{title}" }
                if st.count() > 0 { span { class: "fc", key: "{st.count()}", "{st.count()}" } }
                span { class: "sp" }
                if st.count() > 0 { button { class: "lnk", r#type: "button", onclick: move |_| { m.write().clear(); emit() }, "Clear all" } }
            }
            if !chips.is_empty() {
                div { class: "chips",
                    for (g, o, text) in chips {
                        span { key: "{g}-{o}", class: "chip pop",
                            span { class: "ct", "{text}" }
                            button { class: "rm", r#type: "button", "aria-label": "Remove {text}", onclick: move |_| { m.write().toggle(g, o); emit() }, Icon { name: "x", size: 10.0, stroke: 2.0 } }
                        }
                    }
                }
            }
            if summary && st.count() > 0 { div { class: "fsum", "{st.summary()}" } }
            if presets {
                div { class: "views",
                    for (name, _) in st.presets.clone() {
                        button { class: "pc", r#type: "button", onclick: move |_| { m.write().apply_preset(&name); emit() }, Icon { name: "star", size: 11.0 } "{name}" }
                    }
                    button { class: "pc ghost", r#type: "button", onclick: move |_| { saving += 1; let n = format!("View {}", m.peek().presets.len() + 1); m.write().save_preset(&n) }, Icon { name: "plus", size: 11.0 } "Save view" }
                }
            }
            div { class: "fl", role: "tree", tabindex: "0", "aria-label": "{title}",
                "aria-activedescendant": "oxyn-fm-r{st.focus}",
                onkeydown: move |e: KeyboardEvent| {
                    let k = match e.key() {
                        Key::ArrowUp => Some(FilterKey::Up), Key::ArrowDown => Some(FilterKey::Down),
                        Key::ArrowRight => Some(FilterKey::Right), Key::ArrowLeft => Some(FilterKey::Left),
                        Key::Home => Some(FilterKey::Home), Key::End => Some(FilterKey::End),
                        Key::Enter => Some(FilterKey::Activate),
                        Key::Character(c) if c == " " => Some(FilterKey::Activate),
                        _ => None,
                    };
                    if let Some(k) = k { e.prevent_default(); let before = m.peek().count(); m.write().key(k); if m.peek().count() != before { emit() } }
                },
                for (ri, r) in rows.iter().enumerate() {
                    match *r {
                        FilterRow::Header(g) => {
                            let grp = st.groups[g].clone();
                            let open = st.open[g];
                            let n = st.group_count(g);
                            rsx! {
                                div { id: "oxyn-fm-r{ri}", class: cls(&["gh", if st.focus == ri { "focus" } else { "" }, if open { "open" } else { "" }]), role: "treeitem", "aria-expanded": "{open}",
                                    onclick: move |_| { m.write().set_open(g, !open); m.write().focus = ri },
                                    span { class: "chev", Icon { name: "chevron-right", size: 12.0, stroke: 2.0 } }
                                    span { class: "gl", "{grp.label}" }
                                    if n > 0 { span { class: "gn", "{n}" } }
                                    span { class: "sp" }
                                    if logic && n > 1 {
                                        button { class: "lg", r#type: "button", title: "How these combine",
                                            onclick: move |e| { e.stop_propagation(); m.write().flip_logic(g) },
                                            if st.logic[g] == Logic::Any { "Any" } else { "All" }
                                        }
                                    }
                                }
                            }
                        }
                        FilterRow::Option(g, o) => {
                            let opt = st.groups[g].options[o].clone();
                            let on = st.checked(g, o);
                            rsx! {
                                div { id: "oxyn-fm-r{ri}", class: cls(&["op", if st.focus == ri { "focus" } else { "" }, if on { "on" } else { "" }]), role: "checkbox", "aria-checked": "{on}",
                                    onclick: move |_| { m.write().toggle(g, o); m.write().focus = ri; emit() },
                                    span { class: "cb", "aria-hidden": "true", {tick_svg()} }
                                    span { class: "ol", "{opt.label}" }
                                    if let Some(c) = opt.count { span { class: "oc", "{c}" } }
                                }
                            }
                        }
                    }
                }
            }
            if hints { div { class: "hints", "aria-hidden": "true", kbd { "↑↓" } " move · " kbd { "↵" } " pick · " kbd { "←→" } " groups" } }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Stepper form

/// Stepper form: a multi-step flow for onboarding and setup. Steps fill and check off, slide in
/// from the side you're heading to, validate inline (the first invalid field shakes and takes
/// focus), keep values across Back and Edit, end in a review and a success state. `style` picks
/// affectUI's numbered stepper, Arc's minimal multi-step header or (Oxyn plus) a vertical rail.
/// Oxyn plus: save draft and resume, conditional steps.
#[component]
pub fn StepperForm(
    /// Steps.
    steps: Vec<Step>,
    /// `stepper` · `minimal` · `vertical`.
    #[props(default = "stepper".to_string())]
    style: String,
    /// First step shown.
    #[props(default)]
    initial_step: usize,
    /// Last step's button.
    #[props(default = "Finish".to_string())]
    submit_label: String,
    /// While submitting.
    #[props(default = "Creating…".to_string())]
    submitting_label: String,
    /// Success heading.
    #[props(default = "Workspace ready".to_string())]
    success_title: String,
    /// Simulated submit time (the host's promise).
    #[props(default = 900)]
    latency_ms: u32,
    /// The submit fails once (demo of the error next to the button).
    #[props(default)]
    fail: bool,
    /// Oxyn plus: Save draft.
    #[props(default)]
    drafts: bool,
    /// Values to start with: `(name, value)`; lists are comma-separated.
    #[props(default)]
    initial_values: Vec<(String, String)>,
    /// Step changed.
    #[props(default)]
    on_step_change: EventHandler<usize>,
    /// Completed: every `(name, value)`.
    #[props(default)]
    on_complete: EventHandler<Vec<(String, String)>>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = Stepper::new(steps.clone());
        s.style = StepperStyle::from_id(&style);
        s.submit_label = submit_label.clone();
        s.submitting_label = submitting_label.clone();
        for (k, v) in &initial_values {
            let is_list = s
                .steps
                .iter()
                .flat_map(|st| &st.fields)
                .any(|f| f.name == *k && matches!(f.kind, FieldKind::Tags { .. }));
            let val = if is_list {
                FieldValue::List(
                    v.split(',')
                        .map(str::trim)
                        .filter(|x| !x.is_empty())
                        .map(str::to_string)
                        .collect(),
                )
            } else {
                FieldValue::Text(v.clone())
            };
            s.values.insert(k.clone(), val);
        }
        for i in 0..initial_step.min(s.steps.len()) {
            s.done[i] = true;
        }
        s.current = initial_step.min(s.steps.len().saturating_sub(1));
        s
    });
    let mut fails = use_signal(|| fail);
    let mut shake = use_signal(|| 0u32);
    let st = m.read().clone();
    let step = st.steps[st.current].clone();
    let (pos, total) = st.progress();
    let dir = st.moved.map_or("", |m| if m.0 { "fwd" } else { "back" });
    let vis = st.visible();
    let mut next = move || {
        let before = m.peek().current;
        let r = m.write().next(ClockMs(0));
        match r {
            Err(_) => shake += 1,
            Ok(true) => {}
            Ok(false) => {
                if m.peek().current != before {
                    on_step_change.call(m.peek().current)
                }
            }
        }
    };
    let finish = move |_| {
        if *fails.peek() {
            fails.set(false);
            m.write()
                .finish(Err("That workspace URL is taken. Try another.".into()));
        } else {
            m.write().finish(Ok(()));
            let v = m
                .peek()
                .values
                .iter()
                .map(|(k, v)| (k.clone(), v.display()))
                .collect();
            on_complete.call(v);
        }
    };
    if st.phase == StepperPhase::Success {
        return rsx! {
            div { class: "oxyn-stepper success", role: "status",
                div { class: "ok", svg { view_box: "0 0 24 24", width: "28", height: "28", path { d: "m5 12.5 4.5 4.5L19 7.5", path_length: "1" } } }
                div { class: "sh", "{success_title}" }
                div { class: "sd", "{st.text(\"name\")} is set up. We sent the invites." }
                button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| m.write().reset(ClockMs(0)), "Start over" }
            }
        };
    }
    let primary = st.primary_label().to_string();
    let failed = if let StepperPhase::Failed(e) = &st.phase {
        Some(e.clone())
    } else {
        None
    };
    rsx! {
        div { class: cls(&["oxyn-stepper", &style]),
            if st.style == StepperStyle::Minimal {
                div { class: "mh",
                    span { class: "mc", "Step " span { class: "num", key: "{pos}", "{pos}" } " of {total}" }
                    div { class: "bar", span { style: "width:{pos as f32 / total.max(1) as f32 * 100.0:.0}%" } }
                }
            } else {
                ol { class: "steps", "aria-label": "Progress",
                    for (k, &i) in vis.iter().enumerate() {
                        {
                            let s = st.steps[i].clone();
                            let cur = i == st.current;
                            let done = st.done[i] && !cur;
                            rsx! {
                                li { class: cls(&["si", if cur { "cur" } else { "" }, if done { "done" } else { "" }]), "aria-current": if cur { "step" },
                                    button { r#type: "button", disabled: !done, onclick: move |_| { if m.write().go_to(i, ClockMs(0)) { on_step_change.call(i) } },
                                        span { class: "sn", if done { {tick_svg()} } else { "{k + 1}" } }
                                        span { class: "stt", "{s.title}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "compact", "aria-hidden": "true", "Step {pos} of {total} · {step.title}" }
            }
            div { class: "card", key: "{st.current}", "data-dir": "{dir}",
                fieldset {
                    legend { class: "hd", "{step.heading}" }
                    if let Some(dsc) = step.description.clone() { p { class: "dd", "{dsc}" } }
                    if step.review {
                        dl { class: "review",
                            for (lbl, val, si) in st.review_rows() {
                                div { class: "rr",
                                    dt { "{lbl}" }
                                    dd { "{val}" }
                                    button { class: "lnk", r#type: "button", onclick: move |_| { if m.write().go_to(si, ClockMs(0)) { on_step_change.call(si) } }, "Edit" }
                                }
                            }
                        }
                    }
                    for f in step.fields.clone() {
                        {
                            let name = f.name.clone();
                            let err = st.errors.get(&name).cloned();
                            let shaking = st.shake_at.as_ref().is_some_and(|s| s.0 == name) && err.is_some();
                            let fid = format!("oxyn-st-{name}");
                            match f.kind.clone() {
                                FieldKind::Tags { max } => rsx! {
                                    div { class: cls(&["sf", if shaking { "shake" } else { "" }]), key: "{name}-{shake}",
                                        TagInput { label: f.label.clone(), value: st.list(&name), validate: "email", normalize: "lower", split_on_space: true, max, avatar: true, optional: f.optional,
                                            hint: f.hint.clone(), error: err.clone(), placeholder: "name@company.com",
                                            on_change: move |v: Vec<String>| m.write().set(&name, FieldValue::List(v)) }
                                    }
                                },
                                FieldKind::Choices(opts) => {
                                    let cur = st.text(&name);
                                    rsx! {
                                        div { class: cls(&["sf", if shaking { "shake" } else { "" }]), key: "{name}-{shake}",
                                            div { class: "flabel", id: "{fid}-l", "{f.label}" }
                                            div { class: "plans", role: "radiogroup", "aria-labelledby": "{fid}-l",
                                                for (v, t, aside, dsc) in opts.clone() {
                                                    {
                                                        let on = cur == v;
                                                        let nm = name.clone();
                                                        let vv = v.clone();
                                                        rsx! {
                                                            div { class: cls(&["plan", if on { "on" } else { "" }]), role: "radio", "aria-checked": "{on}", tabindex: if on || (cur.is_empty() && v == opts[0].0) { "0" } else { "-1" },
                                                                onclick: move |_| m.write().set(&nm, FieldValue::Text(vv.clone())),
                                                                onkeydown: {
                                                                    let opts = opts.clone();
                                                                    let nm = name.clone();
                                                                    let v = v.clone();
                                                                    move |e: KeyboardEvent| {
                                                                        let i = opts.iter().position(|o| o.0 == v).unwrap_or(0);
                                                                        let n = opts.len();
                                                                        let j = match e.key() { Key::ArrowRight | Key::ArrowDown => Some((i + 1) % n), Key::ArrowLeft | Key::ArrowUp => Some((i + n - 1) % n), _ => None };
                                                                        if let Some(j) = j { e.prevent_default(); m.write().set(&nm, FieldValue::Text(opts[j].0.clone())) }
                                                                        if e.key() == Key::Enter { e.prevent_default(); m.write().set(&nm, FieldValue::Text(v.clone())); next() }
                                                                    }
                                                                },
                                                                div { class: "pt", span { "{t}" } span { class: "pa", "{aside}" } }
                                                                div { class: "pd", "{dsc}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if let Some(e) = err.clone() { div { class: "msg err", role: "alert", "{e}" } }
                                        }
                                    }
                                }
                                FieldKind::Text { .. } | FieldKind::Email => {
                                    let prefix = if let FieldKind::Text { prefix } = &f.kind { prefix.clone() } else { None };
                                    let email = matches!(f.kind, FieldKind::Email);
                                    let val = st.text(&name);
                                    rsx! {
                                        div { class: cls(&["oxyn-field sf", if err.is_some() { "invalid" } else { "" }, if shaking { "shake" } else { "" }]), key: "{name}-{shake}",
                                            label { class: "flabel", r#for: "{fid}", "{f.label}" if f.optional { span { class: "opt", "Optional" } } }
                                            div { class: "control",
                                                if let Some(p) = prefix { span { class: "affix", "{p}" } }
                                                input { id: "{fid}", class: "in", value: "{val}", r#type: if email { "email" } else { "text" },
                                                    autofocus: f.name == step.fields[0].name,
                                                    "aria-invalid": "{err.is_some()}", "aria-describedby": "{fid}-m",
                                                    oninput: move |e: FormEvent| m.write().set(&name, FieldValue::Text(e.value())),
                                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); next() },
                                                }
                                            }
                                            div { class: "foot", div { id: "{fid}-m", class: if err.is_some() { "msg err" } else { "msg" },
                                                if let Some(e) = err.clone() { span { key: "{e}", "{e}" } } else if let Some(h) = f.hint.clone() { "{h}" }
                                            } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "nav",
                if pos > 1 { button { class: "oxyn-btn ghost sm", r#type: "button", disabled: st.phase == StepperPhase::Submitting, onclick: move |_| { m.write().back(ClockMs(0)); on_step_change.call(m.peek().current) }, "Back" } }
                if drafts {
                    button { class: "oxyn-btn quiet sm", r#type: "button", onclick: move |_| m.write().save_draft(), if st.draft.is_some() { "Draft saved" } else { "Save draft" } }
                    if st.draft.is_some() && st.current == 0 { button { class: "oxyn-btn quiet sm", r#type: "button", onclick: move |_| { m.write().resume(ClockMs(0)); }, "Resume" } }
                }
                span { class: "sp" }
                if let Some(e) = failed { span { class: "ferr", role: "alert", "{e}" } }
                button { class: cls(&["oxyn-btn primary sm go", if st.valid() { "" } else { "dim" }]), r#type: "button", "aria-busy": "{st.phase == StepperPhase::Submitting}",
                    onclick: move |_| next(),
                    if st.phase == StepperPhase::Submitting { Spin { size: 12.0 } }
                    span { class: "lb", key: "{primary}", "{primary}" }
                }
                if st.phase == StepperPhase::Submitting { span { class: "oxyn-timer", style: "animation-duration:{latency_ms.max(1)}ms", onanimationend: finish } }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce()}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Decision card

/// Decision card: an agent's question in its own card just above the composer. Numbered options
/// with one Recommended, tiles for a pick, Approve / Decline with facts for an approval, and your
/// own words; `1`–`5` answer at once, ↑ ↓ move (← → on tiles), Esc folds it to "2 decisions
/// waiting · Show"; the next decision fades in; after the last answer the card folds away. Oxyn
/// plus: a timeout that answers with the recommended option, and "always answer this way".
#[component]
pub fn DecisionCard(
    /// Open decisions, oldest first.
    decisions: Vec<Question>,
    /// Start folded.
    #[props(default)]
    default_folded: bool,
    /// Head label.
    #[props(default = "Needs you".to_string())]
    needs_you: String,
    /// Oxyn plus: answer with Recommended after this many seconds.
    #[props(default)]
    timeout_s: Option<u32>,
    /// Oxyn plus: offer "Always answer this way".
    #[props(default)]
    always: bool,
    /// Answered: `(id, answer text)`.
    #[props(default)]
    on_decide: EventHandler<(String, String)>,
    /// Folded with Later or Esc.
    #[props(default)]
    on_later: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut d = QuestionDeck::new(decisions.clone(), DeckMode::Decisions);
        if default_folded {
            d.phase = DeckPhase::Folded;
        }
        d
    });
    let mut left = use_signal(|| timeout_s.unwrap_or(0));
    let mut remember = use_signal(|| false);
    let st = m.read().clone();
    let mut handle = move |ev: DeckEvent| match ev {
        DeckEvent::Decided(id, a) => {
            let text = match &a {
                Answer::Options(v) => v
                    .first()
                    .map(|i| format!("option {}", i + 1))
                    .unwrap_or_default(),
                Answer::Text(t) => t.clone(),
                Answer::Approve => "approve".into(),
                Answer::Decline => "decline".into(),
                _ => String::new(),
            };
            if *remember.peek() {
                m.write().rules.push((id.clone(), a));
                remember.set(false);
            }
            left.set(timeout_s.unwrap_or(0));
            on_decide.call((id, text))
        }
        DeckEvent::Later(id) => on_later.call(id),
        _ => {}
    };
    if st.phase == DeckPhase::Done {
        return rsx! { div { class: "oxyn-decide gone", span { class: "sr", role: "status", "{st.announce}" } } };
    }
    if st.phase == DeckPhase::Folded {
        return rsx! {
            div { class: "oxyn-decide folded",
                span { class: "dot" }
                span { "{st.waiting()}" }
                button { class: "lnk", r#type: "button", onclick: move |_| m.write().reveal(ClockMs(0)), "Show" }
                span { class: "sr", role: "status", "{st.announce}" }
            }
        };
    }
    let Some(q) = st.current().cloned() else {
        return rsx! {};
    };
    let (pos, n) = st.position();
    let tiles = q.kind == QuestionKind::Pick;
    let gid = format!("oxyn-dc-{}", q.id);
    rsx! {
        div { class: cls(&["oxyn-decide", if st.revealed_at.is_some() { "pulse" } else { "" }]), role: "group", "aria-labelledby": "{gid}-q",
            onkeydown: move |e: KeyboardEvent| {
                let in_field = m.peek().in_text();
                let k = match e.key() {
                    Key::Character(c) if !in_field && c.len() == 1 && c.chars().all(|x| x.is_ascii_digit()) => c.parse::<u8>().ok().map(DeckKey::Digit),
                    Key::ArrowUp => Some(DeckKey::Up), Key::ArrowDown => Some(DeckKey::Down),
                    Key::ArrowLeft if tiles && !in_field => Some(DeckKey::Up), Key::ArrowRight if tiles && !in_field => Some(DeckKey::Down),
                    Key::Home if !in_field => Some(DeckKey::Home), Key::End if !in_field => Some(DeckKey::End),
                    Key::Enter => Some(DeckKey::Activate),
                    Key::Character(c) if c == " " && !in_field => Some(DeckKey::Activate),
                    Key::Escape => Some(DeckKey::Escape),
                    _ => None,
                };
                if let Some(k) = k { e.prevent_default(); let ev = m.write().key(k, ClockMs(0)); handle(ev) }
            },
            div { class: "head",
                span { class: "nd", "{needs_you}" }
                if let Some(t) = q.task.clone() { span { class: "task", "· {t}" } }
                span { class: "sp" }
                if n > 1 {
                    span { class: "pager",
                        button { class: "pg", r#type: "button", "aria-label": "Previous decision", onclick: move |_| { m.write().key(DeckKey::Prev, ClockMs(0)); }, Icon { name: "chevron-left", size: 12.0 } }
                        span { class: "pn", "{pos} of {n}" }
                        button { class: "pg", r#type: "button", "aria-label": "Next decision", onclick: move |_| { m.write().key(DeckKey::Next, ClockMs(0)); }, Icon { name: "chevron-right", size: 12.0 } }
                    }
                }
                button { class: "later", r#type: "button", "aria-keyshortcuts": "Escape", onclick: move |_| { let ev = m.write().fold(); handle(ev) }, "Later" kbd { "Esc" } }
            }
            div { class: "body", key: "{q.id}",
                div { class: "q", id: "{gid}-q", "{q.prompt}" }
                if q.kind == QuestionKind::Approval {
                    dl { class: "facts", for (k, v) in q.facts.clone() { div { dt { "{k}" } dd { "{v}" } } } }
                    div { class: "appr", role: "listbox", "aria-label": "{q.prompt}",
                        for (i, (lbl, cls_)) in [("Approve", "yes"), ("Decline", "no")].into_iter().enumerate() {
                            button { class: cls(&["ab", cls_, if st.focus == i { "focus" } else { "" }]), r#type: "button", role: "option", "aria-selected": "{st.focus == i}", "aria-keyshortcuts": "{i + 1}",
                                onclick: move |_| { let ev = m.write().choose(i, ClockMs(0)); handle(ev) },
                                kbd { "{i + 1}" } "{lbl}"
                            }
                        }
                    }
                } else {
                    div { class: if tiles { "opts tiles" } else { "opts" }, role: "listbox", "aria-label": "{q.prompt}",
                        for (i, o) in q.options.iter().enumerate() {
                            div { class: cls(&["op", if st.focus == i { "focus" } else { "" }, if o.recommended { "rec" } else { "" }]), role: "option", "aria-selected": "{st.focus == i}",
                                "aria-keyshortcuts": "{i + 1}", tabindex: if st.focus == i { "0" } else { "-1" },
                                onclick: move |_| { let ev = m.write().choose(i, ClockMs(0)); handle(ev) },
                                onpointerenter: move |_| m.write().focus = i,
                                kbd { class: "num", "{i + 1}" }
                                span { class: "txt",
                                    span { class: "ot", "{o.title}" if o.recommended { span { class: "rtag", "Recommended" } } }
                                    if let Some(s) = o.summary.clone() { span { class: "os", "{s}" } }
                                }
                            }
                        }
                    }
                }
                if q.allow_text {
                    div { class: cls(&["own", if st.in_text() { "focus" } else { "" }]),
                        input { class: "in", placeholder: "Or tell it what to do…", value: "{st.text.get(st.index).cloned().unwrap_or_default()}",
                            "aria-label": "Your own answer",
                            onfocus: move |_| { let r = m.peek().current().map(|q| q.rows() - 1).unwrap_or(0); m.write().focus = r },
                            oninput: move |e: FormEvent| m.write().type_text(&e.value()),
                        }
                        kbd { "↵" }
                    }
                }
                if always {
                    label { class: "alw",
                        input { r#type: "checkbox", checked: *remember.read(), onchange: move |e: FormEvent| remember.set(e.checked()) }
                        "Always answer this way"
                    }
                }
                if let (Some(_), Some(r)) = (timeout_s, q.recommended()) {
                    div { class: "timeout",
                        span { class: "oxyn-timer", key: "{q.id}-{left}", style: "animation-duration:1000ms",
                            onanimationend: move |_| { let l = *left.peek(); if l <= 1 { left.set(timeout_s.unwrap_or(0)); let ev = m.write().choose(r, ClockMs(0)); handle(ev) } else { left.set(l - 1) } } }
                        "Picks {q.options[r].title} in {left}s"
                    }
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Approval questions (Beautiful UI port)

/// Approval questions: human-in-the-loop questions the agent asks before acting, one at a time.
/// The step counter rolls like an odometer, single choice auto-advances (480 ms), multi-select
/// waits, a custom answer row, Prev / Next, a quiet Skip and a dark Continue with ⏎, Send on the
/// last, "Answers sent", Dismiss and Start over. Ported from Beautiful UI's Approval Card (MIT,
/// © 2026 Shane Levine). Oxyn plus: a review page before sending, Recommended tags, and a timeout
/// that answers with the recommended option.
#[component]
pub fn ApprovalQuestions(
    /// Questions.
    questions: Vec<Question>,
    /// Skip label.
    #[props(default = "Skip".to_string())]
    skip_label: String,
    /// Continue label.
    #[props(default = "Continue".to_string())]
    continue_label: String,
    /// Send label.
    #[props(default = "Send".to_string())]
    send_label: String,
    /// Custom answer placeholder.
    #[props(default = "Something else…".to_string())]
    custom_placeholder: String,
    /// Sent message.
    #[props(default = "Answers sent".to_string())]
    sent_message: String,
    /// Offer Start over after sending.
    #[props(default = true)]
    resettable: bool,
    /// Oxyn plus: review page before Send.
    #[props(default)]
    review: bool,
    /// Oxyn plus: answer with Recommended after this many seconds.
    #[props(default)]
    timeout_s: Option<u32>,
    /// Sent: one answer per question (`"1,3"`, `"text: …"`, `"skipped"`).
    #[props(default)]
    on_submitted: EventHandler<Vec<String>>,
    /// An answer changed: `(question index, answer)`.
    #[props(default)]
    on_answer_change: EventHandler<(usize, String)>,
) -> Element {
    let mut m = use_signal(|| {
        let mut d = QuestionDeck::new(questions.clone(), DeckMode::Paged);
        d.review = review;
        d
    });
    let mut left = use_signal(|| timeout_s.unwrap_or(0));
    let st = m.read().clone();
    let fmt = |a: &Answer| match a {
        Answer::Options(v) => v
            .iter()
            .map(|i| (i + 1).to_string())
            .collect::<Vec<_>>()
            .join(","),
        Answer::Text(t) => format!("text: {t}"),
        Answer::Skipped => "skipped".into(),
        _ => String::new(),
    };
    let handle = move |ev: DeckEvent| match ev {
        DeckEvent::Submitted(a) => on_submitted.call(a.iter().map(fmt).collect()),
        DeckEvent::Changed(i, a) => on_answer_change.call((i, fmt(&a))),
        _ => {}
    };
    if st.phase == DeckPhase::Folded {
        return rsx! { button { class: "oxyn-btn secondary sm", r#type: "button", onclick: move |_| m.write().unfold(ClockMs(0)), "Open approval" } };
    }
    if st.phase == DeckPhase::Done {
        return rsx! {
            div { class: "oxyn-aq sent", role: "status",
                span { class: "ok", {tick_svg()} }
                span { "{sent_message}" }
                if resettable { button { class: "lnk", r#type: "button", onclick: move |_| m.write().reset(ClockMs(0)), "Start over" } }
            }
        };
    }
    let (pos, n) = st.position();
    let roll_dir = st
        .roll
        .map_or("up", |(p, _)| if pos < p { "down" } else { "up" });
    if st.phase == DeckPhase::Review {
        return rsx! {
            div { class: "oxyn-aq",
                div { class: "top", span { class: "ctr", "Review" } span { class: "sp" } }
                dl { class: "review",
                    for (i, (q, a)) in st.review_rows().into_iter().enumerate() {
                        div { class: "rr", dt { "{q}" } dd { "{a}" } button { class: "lnk", r#type: "button", onclick: move |_| m.write().edit(i, ClockMs(0)), "Edit" } }
                    }
                }
                div { class: "foot", span { class: "sp" }
                    button { class: "pill dark", r#type: "button", onclick: move |_| { let ev = m.write().send(); handle(ev) }, "{send_label}" kbd { "⏎" } }
                }
            }
        };
    }
    let Some(q) = st.current().cloned() else {
        return rsx! {};
    };
    let picked: Vec<usize> = match st.answer() {
        Answer::Options(v) => v.clone(),
        _ => vec![],
    };
    let custom = st.text.get(st.index).cloned().unwrap_or_default();
    let last = st.last();
    let primary = if last && !review {
        send_label.clone()
    } else {
        continue_label.clone()
    };
    rsx! {
        div { class: "oxyn-aq", "aria-live": "polite",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); let has = m.peek().has_answer(); if has { let ev = { let mut w = m.write(); if w.in_text() { w.send_text(ClockMs(0)) } else { w.advance(ClockMs(0)) } }; handle(ev) } }
                else if e.key() == Key::Escape { m.write().fold(); }
            },
            div { class: "top",
                span { class: "ctr", "aria-label": "Question {pos} of {n}",
                    span { class: "dgs", key: "{pos}", "data-dir": "{roll_dir}", "{pos}" } " / {n}"
                }
                span { class: "sp" }
                button { class: "ib", r#type: "button", "aria-label": "Previous question", disabled: pos == 1, onclick: move |_| { m.write().key(DeckKey::Prev, ClockMs(0)); }, Icon { name: "chevron-left", size: 13.0 } }
                button { class: "ib", r#type: "button", "aria-label": "Next question", disabled: pos == n, onclick: move |_| { m.write().key(DeckKey::Next, ClockMs(0)); }, Icon { name: "chevron-right", size: 13.0 } }
                button { class: "ib", r#type: "button", "aria-label": "Dismiss", onclick: move |_| { m.write().fold(); }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
            }
            div { class: "stack", key: "{st.index}", "data-dir": "{roll_dir}",
                div { class: "qq", "{q.prompt}" }
                div { class: "opts", role: if q.kind == QuestionKind::Multi { "group" } else { "radiogroup" }, "aria-label": "{q.prompt}",
                    for (i, o) in q.options.iter().enumerate() {
                        {
                            let on = picked.contains(&i);
                            let multi = q.kind == QuestionKind::Multi;
                            rsx! {
                                button { class: cls(&["opt", if on { "on" } else { "" }, if multi { "multi" } else { "" }]), r#type: "button",
                                    role: if multi { "checkbox" } else { "radio" }, "aria-checked": "{on}",
                                    onclick: move |_| { let ev = m.write().choose(i, ClockMs(0)); handle(ev) },
                                    span { class: "mk", "aria-hidden": "true", if multi { {tick_svg()} } else { span { class: "dot" } } }
                                    span { class: "ot", "{o.title}" }
                                    if o.recommended { span { class: "rtag", "Recommended" } }
                                }
                            }
                        }
                    }
                    div { class: cls(&["opt custom", if !custom.trim().is_empty() { "on" } else { "" }]),
                        span { class: "mk", "aria-hidden": "true", Icon { name: "pencil", size: 11.0 } }
                        input { class: "in", placeholder: "{custom_placeholder}", value: "{custom}", "aria-label": "{custom_placeholder}",
                            onfocus: move |_| { let r = m.peek().current().map(|q| q.rows() - 1).unwrap_or(0); m.write().focus = r },
                            oninput: move |e: FormEvent| m.write().type_text(&e.value()),
                        }
                    }
                }
            }
            if let (Some(_), Some(r)) = (timeout_s, q.recommended()) {
                if !st.has_answer() {
                    div { class: "timeout",
                        span { class: "oxyn-timer", key: "{st.index}-{left}", style: "animation-duration:1000ms",
                            onanimationend: move |_| { let l = *left.peek(); if l <= 1 { left.set(timeout_s.unwrap_or(0)); let ev = m.write().choose(r, ClockMs(0)); handle(ev) } else { left.set(l - 1) } } }
                        "Answers “{q.options[r].title}” in {left}s"
                    }
                }
            }
            div { class: "foot",
                span { class: "sp" }
                button { class: "pill quiet", r#type: "button", onclick: move |_| { let ev = m.write().skip(ClockMs(0)); handle(ev) }, "{skip_label}" }
                button { class: "pill dark", r#type: "button", disabled: !st.has_answer(),
                    onclick: move |_| { let ev = { let mut w = m.write(); if !w.text.get(w.index).is_none_or(|t| t.trim().is_empty()) && !matches!(w.answer(), Answer::Options(_)) { w.send_text(ClockMs(0)) } else { w.advance(ClockMs(0)) } }; handle(ev) },
                    span { class: "lb", key: "{primary}", "{primary}" } kbd { "⏎" }
                }
            }
            if st.advance_at.is_some() { span { class: "oxyn-timer", key: "adv{st.index}", style: "animation-duration:480ms", onanimationend: move |_| { let ev = m.write().advance(ClockMs(0)); handle(ev) } } }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Smart form fill

const SAMPLES: [(&str, &str); 2] = [
    (
        "Signature",
        "Thanks,\nMaya Patel\nHead of Partnerships at Lumen Labs\nmaya.patel@lumenlabs.io | +1 (415) 555-0132\n500 Howard St, Suite 300, San Francisco",
    ),
    (
        "Messy note",
        "met jo at the summit — wants a demo next wk\ncall 0300 1234567\njo.ellis@harbor.dev",
    ),
];

/// Smart form fill: paste an email signature or messy notes and the agent fills a contact form
/// field by field. Values fade in 140 ms apart with a confidence dot, the source lights up in the
/// pasted text, low-confidence fields ask for review and block Save until checked or edited, Undo
/// restores the values from before the fill, Esc stops a read. Oxyn plus: undo per field, a
/// confidence gate you can move, and a provenance line per field.
#[component]
pub fn SmartFormFill(
    /// Fields (default: name, title, company, email, phone, address).
    #[props(default)]
    fields: Vec<FillField>,
    /// Card title.
    #[props(default = "New contact".to_string())]
    title: String,
    /// One line under the title.
    #[props(default = "Paste a signature or notes. The agent fills the form.".to_string())]
    description: String,
    /// Model named beside the paste area.
    #[props(default = "Haiku".to_string())]
    model: String,
    /// Below this confidence (0–100) a field asks for review.
    #[props(default = 80)]
    review_threshold: u8,
    /// Oxyn plus: show the threshold control.
    #[props(default)]
    gate: bool,
    /// Primary action.
    #[props(default = "Save contact".to_string())]
    submit_label: String,
    /// Start with a sample pasted (review sheets).
    #[props(default)]
    prefill: bool,
    /// Every value change: `(field, value)`.
    #[props(default)]
    on_values_change: EventHandler<(String, String)>,
    /// Saved: every `(field, value)`.
    #[props(default)]
    on_submit: EventHandler<Vec<(String, String)>>,
) -> Element {
    let fields = if fields.is_empty() {
        contact_fields()
    } else {
        fields
    };
    let mut m = use_signal(|| {
        let mut f = SmartFill::new(fields.clone());
        f.threshold = f32::from(review_threshold) / 100.0;
        if prefill {
            f.text = SAMPLES[0].1.into();
            let r = extract(SAMPLES[0].1);
            f.fill(r, ClockMs(0));
            f.phase = FillPhase::Done;
        }
        f
    });
    let mut sample = use_signal(|| usize::from(prefill));
    let mut fill_n = use_signal(|| 0u32);
    let st = m.read().clone();
    let reading = matches!(st.phase, FillPhase::Reading(_));
    let filling = matches!(st.phase, FillPhase::Filling(_));
    let order = st.order();
    let hl = st.highlight(ClockMs(0));
    // The pasted text with the highlighted source span.
    let text = st.text.clone();
    let pieces: Vec<(String, bool)> = match hl {
        Some((a, b)) if b <= text.len() && text.is_char_boundary(a) && text.is_char_boundary(b) => {
            vec![
                (text[..a].to_string(), false),
                (text[a..b].to_string(), true),
                (text[b..].to_string(), false),
            ]
        }
        _ => vec![(text.clone(), false)],
    };
    let review_n = st.needs_review().len();
    rsx! {
        div { class: "oxyn-sff",
            div { class: "sh",
                div { class: "st", "{title}" }
                if !description.is_empty() { div { class: "sd", "{description}" } }
            }
            div { class: "cols",
                div { class: "well",
                    div { class: "wh",
                        span { class: if reading { "model reading" } else { "model" }, Icon { name: "spark", size: 12.0 } if reading { "Reading with {model}…" } else { "{model}" } }
                        span { class: "sp" }
                        button { class: "lnk", r#type: "button", onclick: move |_| { let i = *sample.peek() % SAMPLES.len(); sample.set(i + 1); m.write().paste(SAMPLES[i].1, ClockMs(0)) }, "Try sample" }
                    }
                    if st.text.is_empty() || reading {
                        textarea { class: "paste", placeholder: "Paste here (⌘V) or type and press ⌘↵", value: "{st.text}", "aria-label": "Text to read",
                            oninput: move |e: FormEvent| { let v = e.value(); if v.len() > m.peek().text.len() + 8 { m.write().paste(&v, ClockMs(0)) } else { m.write().text = v } },
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) { e.prevent_default(); let t = m.peek().text.clone(); m.write().paste(&t, ClockMs(0)) }
                                if e.key() == Key::Escape { m.write().stop(); }
                            },
                        }
                    } else {
                        div { class: "src", tabindex: "0", role: "region", "aria-label": "Pasted text",
                            for (t, on) in pieces { if on { mark { "{t}" } } else { "{t}" } }
                        }
                    }
                    if reading {
                        span { class: "oxyn-timer", style: "animation-duration:900ms", onanimationend: move |_| { let t = m.peek().text.clone(); let r = extract(&t); m.write().fill(r, ClockMs(0)); fill_n += 1; } }
                        button { class: "lnk stop", r#type: "button", onclick: move |_| { m.write().stop(); }, "Stop" kbd { "Esc" } }
                    }
                    if filling { span { class: "oxyn-timer", key: "f{fill_n}", style: "animation-duration:{order.len() as u64 * 140 + 900}ms", onanimationend: move |_| m.write().phase = FillPhase::Done } }
                }
                form { class: "form", onsubmit: move |e| e.prevent_default(),
                    for f in st.fields.iter() {
                        {
                            let id = f.id.clone();
                            let mk = st.mark(&id);
                            let conf = st.confidence(&id);
                            let k = order.iter().position(|x| *x == id);
                            let val = st.values.get(&id).cloned().unwrap_or_default();
                            let note = st.results.get(&id).and_then(|r| r.note.clone());
                            let wide = f.kind == FillKind::Address;
                            let id2 = id.clone();
                            let id3 = id.clone();
                            let id4 = id.clone();
                            let id5 = id.clone();
                            rsx! {
                                div { class: cls(&["sf", if wide { "wide" } else { "" }, match mk { FieldMark::Review => "review", FieldMark::Checked => "checked", FieldMark::Edited => "edited", FieldMark::Filled => "filled", FieldMark::Plain => "" }, if filling && k.is_some() { "landing" } else { "" }]),
                                    style: if let Some(k) = k { format!("--d:{}ms", k * 140) } else { String::new() },
                                    onpointerenter: move |_| m.write().hot = Some(id2.clone()),
                                    onpointerleave: move |_| m.write().hot = None,
                                    label { class: "fl", r#for: "oxyn-sff-{id}", "{f.label}"
                                        if let Some(c) = conf { span { class: "conf", span { class: "cd" } if mk.word().is_empty() { "{c}%" } else { "{mk.word()}" } } }
                                    }
                                    div { class: "control",
                                        if f.kind == FillKind::Company && !val.is_empty() { span { class: "logo", "aria-hidden": "true", Icon { name: "building", size: 13.0 } } }
                                        input { id: "oxyn-sff-{id}", class: "in", key: "{val}", value: "{val}",
                                            r#type: match f.kind { FillKind::Email => "email", FillKind::Phone => "tel", FillKind::Url => "url", _ => "text" },
                                            "aria-invalid": "{mk == FieldMark::Review}",
                                            "aria-description": if let Some(c) = conf { format!("Confidence {c}%{}", note.as_ref().map(|n| format!(". {n}")).unwrap_or_default()) } else { String::new() },
                                            onfocus: move |_| m.write().hot = Some(id3.clone()),
                                            oninput: move |e: FormEvent| { m.write().edit(&id4, &e.value()); on_values_change.call((id4.clone(), e.value())) },
                                        }
                                        if mk == FieldMark::Review {
                                            button { class: "act", r#type: "button", "aria-label": "Mark {f.label} as checked", onclick: move |_| m.write().check(&id5), Icon { name: "check", size: 12.0, stroke: 2.0 } }
                                        }
                                        if k.is_some() && st.before.is_some() {
                                            button { class: "act", r#type: "button", "aria-label": "Undo {f.label}", title: "Undo this field", onclick: { let id = id.clone(); move |_| m.write().undo_field(&id) }, Icon { name: "undo", size: 12.0 } }
                                        }
                                    }
                                    if let Some(n) = note.clone() { if mk == FieldMark::Review { div { class: "note", "{n}" } } }
                                }
                            }
                        }
                    }
                }
            }
            if gate {
                div { class: "gate",
                    label { r#for: "oxyn-sff-gate", "Ask to review below" }
                    input { id: "oxyn-sff-gate", r#type: "range", min: "50", max: "99", value: "{(st.threshold * 100.0).round()}", oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<f32>() { m.write().set_threshold(v / 100.0) } } }
                    output { "{(st.threshold * 100.0).round()}%" }
                }
            }
            div { class: "foot",
                span { class: "fs",
                    match &st.phase {
                        FillPhase::Saved => rsx! { Icon { name: "check", size: 12.0, stroke: 2.0 } "Saved" },
                        FillPhase::Error(e) => rsx! { Icon { name: "info", size: 12.0 } "{e}" },
                        _ if review_n > 0 => rsx! { Icon { name: "eye", size: 12.0 } "{review_n} to review" },
                        _ if !order.is_empty() => rsx! { Icon { name: "spark", size: 12.0 } "Filled {order.len()} fields" },
                        _ => rsx! {},
                    }
                }
                span { class: "sp" }
                if st.before.is_some() { button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| { m.write().undo(); }, Icon { name: "undo", size: 12.0 } "Undo" } }
                button { class: "oxyn-btn primary sm", r#type: "button", "aria-busy": "{st.phase == FillPhase::Saving}",
                    onclick: move |_| { let r = m.write().save(); if r.is_ok() { m.write().saved(Ok(())); let v = m.peek().values.iter().map(|(a, b)| (a.clone(), b.clone())).collect(); on_submit.call(v) } },
                    "{submit_label}"
                }
            }
            span { class: "sr", role: "status", "aria-live": "polite", "{st.announce}" }
        }
    }
}
