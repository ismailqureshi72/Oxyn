//! Destructive confirmations and run-level approvals (plan #28): `HoldToConfirm`,
//! `ConfirmMorph`, `ConfirmPopover`, `ApprovalGates`, `PermissionRequest`, `AgentApprovals`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        code_spans,
        confirm::{
            ConfirmMode, ConfirmMorph as MorphModel, ConfirmPopover as PopModel, HoldConfirm,
            MorphFace, PopoverPhase,
        },
        gates::{
            AgentApprovals as AgentModel, GateDecision, GateStatus, Gates, Outcome, PermRisk,
            PermissionRequest as PermModel, RuleScope, Scope,
        },
    },
};

use crate::composer::icons::Icon;

fn spans(text: &str) -> Element {
    rsx! {
        for (t, code) in code_spans(text) {
            if code { code { class: "oxp-code", "{t}" } } else { "{t}" }
        }
    }
}

/// Hold to confirm: the fill runs while held (pointer, Space or Enter), rewinds when released
/// early and resumes on a quick re-press; a check draws in when done.
#[component]
pub fn HoldToConfirm(
    /// Model (fill, confirmed).
    model: HoldConfirm,
    /// Shared clock.
    now: ClockMs,
    /// Resting label.
    #[props(default = "Hold to delete".to_string())]
    label: String,
    /// Label after the hold completes.
    #[props(default = "Done".to_string())]
    confirmed_label: String,
    /// Leading icon name.
    #[props(default = "trash".to_string())]
    icon: String,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Show the "Keep holding" hint below.
    #[props(default = true)]
    hint: bool,
    /// Hold started (`true`) or ended.
    #[props(default)]
    on_hold: EventHandler<bool>,
) -> Element {
    let fill = model.fill(now);
    let done = model.confirmed;
    let holding = model.holding();
    let text = if done {
        confirmed_label.clone()
    } else {
        label.clone()
    };
    let hint_text = model.hint(now);
    rsx! {
        div { class: "oxp-hold-wrap",
            button {
                class: "oxp-hold {model.tone.id()} {size}",
                r#type: "button",
                "data-holding": "{holding}",
                "data-done": "{done}",
                style: "--fill:{fill * 100.0:.1}%",
                "aria-label": "{label}, hold to confirm",
                "aria-describedby": "oxp-hold-hint",
                onpointerdown: move |_| on_hold.call(true),
                onpointerup: move |_| on_hold.call(false),
                onpointerleave: move |_| if holding { on_hold.call(false) },
                onblur: move |_| if holding { on_hold.call(false) },
                onkeydown: move |e: KeyboardEvent| {
                    if (e.key() == Key::Enter || e.key() == Key::Character(" ".into())) && !e.is_auto_repeating() {
                        e.prevent_default();
                        on_hold.call(true);
                    }
                },
                onkeyup: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                        on_hold.call(false);
                    }
                },
                span { class: "oxp-hold-fill", "aria-hidden": "true" }
                span { class: "oxp-hold-ic",
                    if done { svg { class: "oxp-draw", view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3 8.5l3.2 3L13 4.5" } } }
                    else { Icon { name: icon.clone(), size: 14.0 } }
                }
                span { class: "oxp-hold-lb", "{text}" }
            }
            if hint { div { id: "oxp-hold-hint", class: "oxp-hint", "aria-live": "polite", "{hint_text}" } }
        }
    }
}

/// Confirm morph: a destructive button that turns into an inline question, a spinner, then a
/// result with Undo, or an error with Retry. Press, hold or countdown confirm.
#[component]
pub fn ConfirmMorph(
    /// Model.
    model: MorphModel,
    /// Shared clock.
    now: ClockMs,
    /// Icon before the resting label.
    #[props(default = "trash".to_string())]
    icon: String,
    /// Resting button pressed.
    #[props(default)]
    on_open: EventHandler<()>,
    /// Confirm pressed (press and countdown modes) or Retry.
    #[props(default)]
    on_confirm: EventHandler<()>,
    /// Cancel, Escape or stop the countdown.
    #[props(default)]
    on_cancel: EventHandler<()>,
    /// Undo pressed.
    #[props(default)]
    on_undo: EventHandler<()>,
    /// Hold on Confirm started or ended (hold mode).
    #[props(default)]
    on_hold: EventHandler<bool>,
) -> Element {
    let face = model.face(now);
    let l = model.labels.clone();
    let text = model.text(now);
    let tone = model.tone.id();
    let hold_mode = model.mode == ConfirmMode::Hold;
    let fill = model.hold.fill(now);
    let ann = model.announcement(now);
    let body = match face {
        MorphFace::Idle => rsx! {
            button { class: "oxp-cm-rest", r#type: "button", onclick: move |_| on_open.call(()),
                Icon { name: icon.clone(), size: 14.0 } span { "{text}" }
            }
        },
        MorphFace::Confirming => rsx! {
            span { class: "oxp-cm-q", "{text}" }
            button { class: "oxp-cm-no", r#type: "button", onclick: move |_| on_cancel.call(()), "{l.cancel}" }
            if hold_mode {
                button { class: "oxp-cm-yes {tone} hold", r#type: "button", style: "--fill:{fill * 100.0:.1}%",
                    onpointerdown: move |_| on_hold.call(true), onpointerup: move |_| on_hold.call(false),
                    onpointerleave: move |_| on_hold.call(false),
                    span { class: "oxp-hold-fill" } span { class: "oxp-cm-lb", "Hold to {l.confirm.to_lowercase()}" }
                }
            } else {
                button { class: "oxp-cm-yes {tone}", r#type: "button", onclick: move |_| on_confirm.call(()), "{l.confirm}" }
            }
        },
        MorphFace::Counting(s) => rsx! {
            span { class: "oxp-cm-ring", style: "--s:{s}", "{s}" }
            span { class: "oxp-cm-q", "{l.pending} in {s}s" }
            button { class: "oxp-cm-no", r#type: "button", onclick: move |_| on_cancel.call(()), "{l.cancel}" }
        },
        MorphFace::Pending | MorphFace::Undoing => rsx! {
            span { class: "oxp-spin", "aria-hidden": "true" } span { class: "oxp-cm-q", "{text}" }
        },
        MorphFace::Done => rsx! {
            span { class: "oxp-cm-ok", svg { class: "oxp-draw", view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3 8.5l3.2 3L13 4.5" } } }
            span { class: "oxp-cm-q", "{text}" }
            if model.undoable { button { class: "oxp-cm-no", r#type: "button", onclick: move |_| on_undo.call(()), Icon { name: "undo", size: 13.0 } "{l.undo}" } }
        },
        MorphFace::Error => rsx! {
            span { class: "oxp-cm-err", Icon { name: "alert", size: 14.0 } }
            span { class: "oxp-cm-q", "{text}" }
            button { class: "oxp-cm-no", r#type: "button", onclick: move |_| on_cancel.call(()), "{l.cancel}" }
            button { class: "oxp-cm-yes {tone}", r#type: "button", onclick: move |_| on_confirm.call(()), "{l.retry}" }
        },
    };
    rsx! {
        div { class: "oxp-cm", "data-face": face.id(), role: "group", "aria-label": "{l.label}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { on_cancel.call(()) },
            {body}
            span { class: "oxp-sr", role: "status", "aria-live": "polite", "{ann}" }
        }
    }
}

/// Confirm popover: the trigger grows into a card with the consequence, an optional
/// type-to-confirm phrase, progress on Confirm, then a check (or the failure with Retry).
#[component]
pub fn ConfirmPopover(
    /// Model.
    model: PopModel,
    /// Shared clock.
    now: ClockMs,
    /// Trigger label.
    #[props(default = "Delete project".to_string())]
    label: String,
    /// Card heading.
    #[props(default = "Delete this project?".to_string())]
    title: String,
    /// The consequence (describes the dialog).
    #[props(default)]
    description: Option<String>,
    /// Trigger icon.
    #[props(default = "trash".to_string())]
    icon: String,
    /// Confirm label.
    #[props(default = "Delete".to_string())]
    confirm_label: String,
    /// Cancel label.
    #[props(default = "Cancel".to_string())]
    cancel_label: String,
    /// Progress label.
    #[props(default = "Deleting".to_string())]
    pending_label: String,
    /// Success label (no longer than the trigger label).
    #[props(default = "Deleted".to_string())]
    success_label: String,
    /// Failure heading.
    #[props(default = "That didn't work".to_string())]
    error_title: String,
    /// Retry label.
    #[props(default = "Try again".to_string())]
    retry_label: String,
    /// Trigger pressed.
    #[props(default)]
    on_show: EventHandler<()>,
    /// Cancel or Escape.
    #[props(default)]
    on_dismiss: EventHandler<()>,
    /// Phrase typed.
    #[props(default)]
    on_type: EventHandler<String>,
    /// Confirm pressed.
    #[props(default)]
    on_submit: EventHandler<()>,
) -> Element {
    let phase = model.phase(now);
    let open = model.open(now);
    let tone = model.tone.id();
    let fill = model.match_fill();
    let shake = model.shake(now);
    let sweep = model.sweep(now);
    let trig = match phase {
        PopoverPhase::Success => {
            rsx! { span { class: "oxp-cp-ok", svg { class: "oxp-draw", view_box: "0 0 16 16", width: "14", height: "14", path { d: "M3 8.5l3.2 3L13 4.5" } } "{success_label}" } }
        }
        _ => rsx! { Icon { name: icon.clone(), size: 14.0 } "{label}" },
    };
    let btn = match phase {
        PopoverPhase::Working(p) => {
            let (w, pct) = match p {
                Some(p) => (
                    format!("--w:{:.0}%", p * 100.0),
                    format!(" {:.0}%", p * 100.0),
                ),
                None => (
                    format!("--w:30%;--x:{:.0}%", sweep * 130.0 - 30.0),
                    String::new(),
                ),
            };
            rsx! { button { class: "oxp-cp-go {tone} working", r#type: "button", "aria-disabled": "true", style: "{w}",
            span { class: "oxp-cp-bar", "data-ind": "{p.is_none()}" } span { class: "oxp-cp-lb", "{pending_label}{pct}" } } }
        }
        PopoverPhase::Error => {
            rsx! { button { class: "oxp-cp-go {tone}", r#type: "button", onclick: move |_| on_submit.call(()), "{retry_label}" } }
        }
        _ => {
            rsx! { button { class: "oxp-cp-go {tone}", r#type: "button", "aria-disabled": "{!model.unlocked()}",
            style: "--m:{fill * 100.0:.0}%", onclick: move |_| on_submit.call(()),
            span { class: "oxp-cp-m" } span { class: "oxp-cp-lb", "{confirm_label}" } } }
        }
    };
    rsx! {
        div { class: "oxp-cp {model.align.id()}",
            button { class: "oxp-cp-trig {tone}", r#type: "button", "aria-expanded": "{open}", "aria-haspopup": "dialog",
                "data-phase": phase.id(), onclick: move |_| on_show.call(()), {trig} }
            if open {
                div { class: "oxp-cp-card", role: "alertdialog", "aria-label": "{title}", "aria-describedby": "oxp-cp-desc",
                    style: "transform:translateX({shake:.1}px)",
                    onkeydown: move |e: KeyboardEvent| match e.key() { Key::Escape => on_dismiss.call(()), Key::Enter => on_submit.call(()), _ => {} },
                    if phase == PopoverPhase::Error {
                        div { class: "oxp-cp-t err", Icon { name: "alert", size: 14.0 } "{error_title}" }
                        div { id: "oxp-cp-desc", class: "oxp-cp-d", "{model.error.clone().unwrap_or_default()}" }
                    } else {
                        div { class: "oxp-cp-t", "{title}" }
                        if let Some(d) = description.clone() { div { id: "oxp-cp-desc", class: "oxp-cp-d", "{d}" } }
                    }
                    if let Some(phrase) = model.confirm_text.clone() {
                        if phase == PopoverPhase::Confirming {
                            label { class: "oxp-cp-ty", "Type " code { "{phrase}" } " to confirm"
                                input { r#type: "text", value: "{model.typed}", autocomplete: "off", spellcheck: "false",
                                    "aria-invalid": "{fill == 0.0 && !model.typed.is_empty()}",
                                    oninput: move |e| on_type.call(e.value()) }
                            }
                        }
                    }
                    div { class: "oxp-cp-btns",
                        button { class: "oxp-cp-no", r#type: "button", onclick: move |_| on_dismiss.call(()), "{cancel_label}" }
                        {btn}
                    }
                }
            }
        }
    }
}

/// Approval gates: the run's open requests as a stack (decide the top one: approve, change it
/// first, or reject) and a Decided list; each open gate says how many steps it holds.
#[component]
pub fn ApprovalGates(
    /// Gates of the run.
    gates: Gates,
    /// Heading.
    #[props(default = "Approvals".to_string())]
    title: String,
    /// Show the Decided list.
    #[props(default = true)]
    decided: bool,
    /// A gate was decided.
    #[props(default)]
    on_decide: EventHandler<(String, GateDecision)>,
) -> Element {
    let mut editing = use_signal(|| None::<String>);
    let mut draft = use_signal(String::new);
    let open = gates.open();
    let done = gates.decided();
    let depth = open.len().min(3);
    let status = gates.status_line();
    let top = open.first().map(|g| (*g).clone());
    rsx! {
        section { class: "oxp-gates", "aria-label": "{title}",
            header { class: "oxp-g-hd",
                span { class: "oxp-g-t", "{title}" }
                span { class: "oxp-g-task", "{gates.task}" }
                span { class: "oxp-sp" }
                span { class: if open.is_empty() { "oxp-g-st calm" } else { "oxp-g-st wait" }, "{status}" }
            }
            if let Some(g) = top {
                div { class: "oxp-g-stack", style: "--depth:{depth}",
                    for i in (1..depth).rev() { div { class: "oxp-g-peek", style: "--i:{i}", "aria-hidden": "true" } }
                    article { class: "oxp-g-card", "data-kind": g.kind.id(), tabindex: "0",
                        onkeydown: {
                            let id = g.id.clone();
                            move |e: KeyboardEvent| {
                                if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) {
                                    on_decide.call((id.clone(), GateDecision::Approve));
                                }
                            }
                        },
                        div { class: "oxp-g-who",
                            span { class: "oxp-g-av", "{g.agent.chars().next().unwrap_or('A')}" }
                            span { class: "oxp-g-ag", "{g.agent}" }
                            span { class: "oxp-g-kind", Icon { name: g.kind.icon(), size: 12.0 } "{g.kind.id()}" }
                            span { class: "oxp-sp" }
                            if g.holds > 0 { span { class: "oxp-g-holds", "holds {g.holds}" } }
                        }
                        div { class: "oxp-g-act", "{g.action}" }
                        div { class: "oxp-g-why", "{g.reason}" }
                        if let Some(inp) = g.input.clone() {
                            if editing().as_deref() == Some(g.id.as_str()) {
                                textarea { class: "oxp-g-edit", rows: "3", value: "{draft}", oninput: move |e| draft.set(e.value()) }
                            } else { pre { class: "oxp-g-in", "{inp}" } }
                        }
                        if !g.diff.is_empty() {
                            div { class: "oxp-g-diff",
                                for l in g.diff.iter() {
                                    div { class: if l.starts_with('+') { "add" } else if l.starts_with('-') { "del" } else { "ctx" }, "{l}" }
                                }
                            }
                        }
                        div { class: "oxp-g-btns",
                            button { r#type: "button", class: "q", onclick: {
                                let id = g.id.clone();
                                move |_| on_decide.call((id.clone(), GateDecision::Reject(None)))
                            }, "Reject" }
                            if g.input.is_some() {
                                if editing().as_deref() == Some(g.id.as_str()) {
                                    button { r#type: "button", onclick: {
                                        let id = g.id.clone();
                                        move |_| { on_decide.call((id.clone(), GateDecision::Edit(draft()))); editing.set(None); }
                                    }, "Approve edit" }
                                } else {
                                    button { r#type: "button", onclick: {
                                        let id = g.id.clone();
                                        let v = g.input.clone().unwrap_or_default();
                                        move |_| { draft.set(v.clone()); editing.set(Some(id.clone())); }
                                    }, Icon { name: "pencil", size: 12.0 } "Change" }
                                }
                            }
                            button { r#type: "button", class: "go", onclick: {
                                let id = g.id.clone();
                                move |_| on_decide.call((id.clone(), GateDecision::Approve))
                            }, "Approve" kbd { "⌘↵" } }
                        }
                    }
                }
            } else {
                div { class: "oxp-g-empty", if gates.live { span { class: "oxp-spin" } "Running · new requests appear here" } else { "Nothing waiting" } }
            }
            if decided && !done.is_empty() {
                div { class: "oxp-g-dec",
                    div { class: "oxp-g-dh", "Decided" span { "{done.len()}" } }
                    for g in done.iter() {
                        div { class: "oxp-g-row {g.status.id()}",
                            span { class: "oxp-g-dot" }
                            span { class: "oxp-g-ra", "{g.action}" }
                            span { class: "oxp-sp" }
                            if let Some(n) = g.note.clone() { span { class: "oxp-g-note", "{n}" } }
                            span { class: "oxp-g-rs", "{g.status.label()}" }
                        }
                    }
                }
            }
        }
    }
}

/// Permission request: who asks for which right and why, exactly what will happen, then allow
/// once, for a while or always; it folds into one line with the time left and Revoke.
#[component]
pub fn PermissionRequest(
    /// Model.
    req: PermModel,
    /// Shared clock.
    now: ClockMs,
    /// Preview heading.
    #[props(default = "What will happen".to_string())]
    preview_title: String,
    /// Scope picked.
    #[props(default)]
    on_scope: EventHandler<Scope>,
    /// Allow pressed.
    #[props(default)]
    on_allow: EventHandler<()>,
    /// Deny pressed.
    #[props(default)]
    on_deny: EventHandler<()>,
    /// Revoke, Change or Ask again.
    #[props(default)]
    on_revoke: EventHandler<()>,
    /// "N more" pressed.
    #[props(default)]
    on_more: EventHandler<()>,
) -> Element {
    let risk = req.risk;
    if let Some(line) = req.folded(now) {
        let ok = req.active(now);
        let act = if ok { "Revoke" } else { "Ask again" };
        return rsx! {
            div { class: if ok { "oxp-pr folded ok" } else { "oxp-pr folded" }, role: "status",
                span { class: "oxp-pr-ic", if ok { Icon { name: "check", size: 12.0, stroke: 2.0 } } else { Icon { name: "ban", size: 12.0 } } }
                span { class: "oxp-pr-ln", b { "{req.agent}" } " · {req.action} · " span { class: "oxp-mono", "{req.target}" } }
                span { class: "oxp-sp" }
                span { class: "oxp-pr-left", "{line}" }
                button { class: "oxp-lnk", r#type: "button", onclick: move |_| on_revoke.call(()), "{act}" }
            }
        };
    }
    let offered = req.offered();
    let (rows, more) = req.visible_preview();
    let rows = rows.to_vec();
    rsx! {
        div { class: "oxp-pr risk-{risk.id()}", role: "group", "aria-label": "{req.agent} wants to {req.action}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { on_deny.call(()) },
            div { class: "oxp-pr-hd",
                span { class: "oxp-g-av", "{req.agent.chars().next().unwrap_or('A')}" }
                div { class: "oxp-pr-tt",
                    div { b { "{req.agent}" } " wants to " b { "{req.action}" } }
                    div { class: "oxp-pr-tg", Icon { name: "plug", size: 11.0 } span { class: "oxp-mono", "{req.target}" }
                        if let Some(m) = req.model.clone() { span { class: "oxp-pr-m", " · {m}" } } }
                }
                if risk != PermRisk::Safe { span { class: "oxp-tag {risk.id()}", "{risk.id()}" } }
            }
            if !req.reason.is_empty() { blockquote { class: "oxp-pr-why", "{req.reason}" } }
            if !rows.is_empty() {
                div { class: "oxp-pr-pv",
                    div { class: "oxp-pr-ph", "{preview_title}" }
                    for r in rows.iter() {
                        div { class: "oxp-pr-row", Icon { name: r.icon, size: 12.0 } span { "{r.label}" } span { class: "oxp-sp" }
                            if let Some(m) = r.meta.clone() { span { class: "oxp-mono oxp-dim", "{m}" } } }
                    }
                    if more > 0 { button { class: "oxp-lnk", r#type: "button", onclick: move |_| on_more.call(()), "{more} more" } }
                }
            }
            div { class: "oxp-pr-ft",
                if offered.len() > 1 {
                    div { class: "oxp-seg", role: "radiogroup", "aria-label": "How long",
                        for s in offered.iter().copied() {
                            button { r#type: "button", role: "radio", "aria-checked": "{req.scope == s}",
                                onclick: move |_| on_scope.call(s), "{s.label(req.duration_min)}" }
                        }
                    }
                }
                span { class: "oxp-sp" }
                button { class: "oxp-btn", r#type: "button", onclick: move |_| on_deny.call(()), "Deny" }
                button { class: "oxp-btn go {risk.id()}", r#type: "button", onclick: move |_| on_allow.call(()), "Allow" }
            }
        }
    }
}

/// Agent approvals: one card for a queue of tool calls (tool mark, the agent's reason, editable
/// arguments), approve / deny / always allow (this agent or the workspace), rules and a log.
#[component]
pub fn AgentApprovals(
    /// Model.
    model: AgentModel,
    /// Heading.
    #[props(default = "Approvals".to_string())]
    title: String,
    /// Tab shown first: `queue` · `rules` · `log`.
    #[props(default = "queue".to_string())]
    tab: String,
    /// Approve (`true`) or deny the top request; the flag says whether arguments were edited.
    #[props(default)]
    on_decide: EventHandler<(bool, bool)>,
    /// Always allow with a scope.
    #[props(default)]
    on_always: EventHandler<RuleScope>,
    /// An argument changed.
    #[props(default)]
    on_arg: EventHandler<(String, String)>,
    /// A rule removed.
    #[props(default)]
    on_remove_rule: EventHandler<usize>,
) -> Element {
    let mut view = use_signal(|| tab.clone());
    let mut edited = use_signal(|| false);
    let mut menu = use_signal(|| false);
    let cur = model.current().cloned();
    let n = model.queue.len();
    let tabs = [
        ("queue", format!("Waiting {n}")),
        ("rules", format!("Rules {}", model.rules.len())),
        ("log", "Log".to_string()),
    ];
    rsx! {
        section { class: "oxp-aa", "aria-label": "{title}",
            header { class: "oxp-aa-hd",
                span { class: "oxp-g-t", "{title}" }
                span { class: "oxp-sp" }
                div { class: "oxp-seg", role: "tablist",
                    for (id, lb) in tabs.iter().cloned() {
                        button { r#type: "button", role: "tab", "aria-selected": "{view() == id}", onclick: move |_| view.set(id.to_string()), "{lb}" }
                    }
                }
            }
            match view().as_str() {
                "rules" => rsx! {
                    div { class: "oxp-aa-list",
                        if model.rules.is_empty() { div { class: "oxp-dim oxp-aa-empty", "No always-allow rules yet" } }
                        for (i, r) in model.rules.iter().enumerate() {
                            div { class: "oxp-aa-rule",
                                Icon { name: "check-circle", size: 13.0 }
                                span { "{r.permission}" }
                                span { class: "oxp-mono oxp-dim", "{r.tool}" }
                                span { class: "oxp-sp" }
                                span { class: "oxp-tag", match &r.scope { RuleScope::Workspace => model.workspace.clone(), RuleScope::Agent(a) => a.clone() } }
                                button { class: "oxp-lnk", r#type: "button", "aria-label": "Remove rule", onclick: move |_| on_remove_rule.call(i), "Remove" }
                            }
                        }
                    }
                },
                "log" => rsx! {
                    div { class: "oxp-aa-list",
                        for e in model.log.iter() {
                            div { class: "oxp-aa-log",
                                span { class: match e.outcome { Outcome::Denied => "oxp-g-dot no", Outcome::Allowed => "oxp-g-dot auto", Outcome::Approved => "oxp-g-dot ok" } }
                                span { "{e.summary}" }
                                if e.edited { span { class: "oxp-tag", "edited" } }
                                span { class: "oxp-sp" }
                                span { class: "oxp-dim", "{e.outcome.label()} · {e.by}" }
                            }
                        }
                    }
                },
                _ => rsx! {
                    if let Some(a) = cur.clone() {
                        div { class: "oxp-aa-cur risk-{a.risk}",
                            div { class: "oxp-aa-who",
                                span { class: "oxp-aa-tool", Icon { name: "tool", size: 14.0 } }
                                span { b { "{a.agent}" } span { class: "oxp-dim", " · {a.model}" } }
                                span { class: "oxp-sp" }
                                span { class: "oxp-tag {a.risk}", "{a.risk} risk" }
                            }
                            div { class: "oxp-aa-title", {spans(&a.title)} }
                            div { class: "oxp-g-why", "{a.reason}" }
                            div { class: "oxp-aa-args",
                                for arg in a.args.iter().cloned() {
                                    label { class: "oxp-aa-arg",
                                        span { class: "oxp-mono oxp-dim", "{arg.key}" }
                                        if arg.editable {
                                            input { r#type: "text", value: "{arg.value}", oninput: move |e| { edited.set(true); on_arg.call((arg.key.clone(), e.value())) } }
                                        } else { span { class: "oxp-mono", "{arg.value}" } }
                                    }
                                }
                            }
                            div { class: "oxp-pr-ft",
                                if n > 1 { span { class: "oxp-dim", "{n - 1} more waiting" } }
                                span { class: "oxp-sp" }
                                button { class: "oxp-btn", r#type: "button", onclick: move |_| { on_decide.call((false, edited())); edited.set(false) }, "Deny" }
                                div { class: "oxp-aa-split",
                                    button { class: "oxp-btn go", r#type: "button", onclick: move |_| { on_decide.call((true, edited())); edited.set(false) }, if edited() { "Approve edited" } else { "Approve" } }
                                    button { class: "oxp-btn go caret", r#type: "button", "aria-label": "Always allow", "aria-expanded": "{menu()}", onclick: move |_| menu.toggle(), Icon { name: "chevron", size: 12.0 } }
                                    if menu() {
                                        div { class: "oxp-aa-menu", role: "menu",
                                            button { r#type: "button", role: "menuitem", onclick: {
                                                let ag = a.agent.clone();
                                                move |_| { menu.set(false); on_always.call(RuleScope::Agent(ag.clone())) }
                                            }, "Always allow for {a.agent}" }
                                            button { r#type: "button", role: "menuitem", onclick: move |_| { menu.set(false); on_always.call(RuleScope::Workspace) }, "Always allow in {model.workspace}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "oxp-aa-empty", Icon { name: "check-circle", size: 16.0 } "All caught up" }
                    }
                },
            }
        }
    }
}

/// Status of a gate as a dot class (for hosts drawing their own rows).
pub fn gate_dot(s: GateStatus) -> &'static str {
    match s {
        GateStatus::Open => "wait",
        GateStatus::Approved | GateStatus::Edited => "ok",
        GateStatus::Rejected => "no",
    }
}
