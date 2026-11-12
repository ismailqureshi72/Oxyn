//! `ApprovalCard` (`approval-card.md`): inline (tray), footer, terminal, modal and paged.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    approval::{ApprovalCard as Card, ApprovalQueue, CardPhase, Decision, Risk, pattern_ladder},
};

use crate::composer::icons::Icon;

fn risk_badge(r: Risk) -> Element {
    rsx! { span { class: "risk {r.id()}", span { class: "rg", "{r.glyph()}" } "{r.label()}" } }
}

/// What the buttons look like this frame.
struct Buttons {
    allow_label: String,
    allow_fill: Option<f32>,
    allow_disabled: bool,
    hold: bool,
    deny_default: bool,
    spinner: bool,
}

fn buttons(card: &Card, phase: &CardPhase, now: ClockMs) -> Buttons {
    let r = card.request.risk;
    let (fill, disabled) = match phase {
        CardPhase::Arming(p) => (Some(*p), true),
        CardPhase::Holding(p) | CardPhase::Draining(p) => (Some(*p), false),
        CardPhase::Entering => (None, true),
        CardPhase::Submitting(_) => (None, true),
        _ => (None, false),
    };
    Buttons {
        allow_label: match (r, phase) {
            (Risk::Critical, CardPhase::Holding(_)) => "Keep holding…".into(),
            (Risk::Critical, _) => "Hold to allow once".into(),
            _ => "Allow once".into(),
        },
        allow_fill: fill,
        allow_disabled: disabled,
        hold: r == Risk::Critical,
        deny_default: !r.enter_allows(),
        spinner: card.spinner(now),
    }
}

/// An approval request, as a card.
#[component]
pub fn ApprovalCard(
    /// The card (request + input state).
    card: Card,
    /// Shared clock.
    now: ClockMs,
    /// `inline`, `footer`, `terminal` or `modal`.
    #[props(default = "inline".to_string())]
    variant: String,
    /// Buttons on the grey tray below the card (`tray`) or inside it (`card`).
    #[props(default = "tray".to_string())]
    actions: String,
    /// Show key hints.
    #[props(default = true)]
    shortcuts: bool,
    /// "Always allow" checkbox ticked.
    #[props(default)]
    always: bool,
    /// Pager text ("1 of 3") and batch label, for `paged`.
    #[props(default)]
    pager: Option<String>,
    /// Batch row ("Allow all 3 once").
    #[props(default)]
    batch: Option<String>,
    /// A decision was pressed.
    #[props(default)]
    on_decide: EventHandler<Decision>,
    /// Hold started / ended (Critical).
    #[props(default)]
    on_hold: EventHandler<bool>,
) -> Element {
    let req = card.request.clone();
    let phase = card.phase(now);
    let b = buttons(&card, &phase, now);
    let countdown = card.countdown(now);
    let r = req.risk;
    if let CardPhase::Resolved {
        decision, morph, ..
    } = &phase
    {
        let row = card.resolved_row().unwrap_or_default();
        let ok = decision.allows();
        let pat = match decision {
            Decision::AllowAlways { .. } => Some("Edit rule"),
            _ => None,
        };
        return rsx! {
            div { class: if ok { "oxyn-ap resolved ok" } else { "oxyn-ap resolved no" }, tabindex: "-1",
                style: "--m:{morph:.2}",
                span { class: "ic", if ok { Icon { name: "check", size: 13.0, stroke: 2.0 } } else { Icon { name: "ban", size: 13.0 } } }
                span { class: "tt", "{req.title}" }
                span { class: "rw", "{row}" }
                span { class: "sp" }
                if let Some(p) = pat { button { class: "lnk", r#type: "button", "{p}" } }
            }
        };
    }
    if phase == CardPhase::Stale {
        return rsx! { div { class: "oxyn-ap resolved stale", span { class: "ic", Icon { name: "stop", size: 10.0 } } span { class: "tt", "{req.title}" } span { class: "rw", "Run stopped · no longer needed" } } };
    }
    if variant == "terminal" {
        let cmd = req.command.clone().unwrap_or(req.title.clone());
        let head = pattern_ladder(&cmd).get(1).cloned().unwrap_or(cmd.clone());
        let hold = b.allow_fill.map(|p| {
            let n = (p * 8.0).round() as usize;
            format!(" [hold y {}{}]", "▓".repeat(n), "░".repeat(8 - n))
        });
        let no_n = if req.allow_always { 3 } else { 2 };
        return rsx! {
            div { class: "oxyn-ap term risk-{r.id()}",
                div { span { class: "rg", "{r.glyph()} " } "Would you like to run " span { class: "cmd", "{cmd}" } "?" }
                for line in req.reasons.iter() { div { class: "dim", "  {line}" } }
                div { class: "opt sel", "› 1. Yes (y)" if let Some(h) = hold { span { class: "hold", "{h}" } } }
                if req.allow_always { div { class: "opt", "  2. Yes, and don't ask again for \u{201c}{head}\u{201d} (p)" } }
                div { class: "opt", "  {no_n}. No, and tell the agent what to do differently (esc)" }
                if let Some((c, warn)) = countdown { div { class: if warn { "dim warn" } else { "dim" }, "  {c}" } }
            }
        };
    }
    let allow_style = b
        .allow_fill
        .map(|p| format!("--fill:{:.0}%", p * 100.0))
        .unwrap_or_default();
    let hold = b.hold;
    let decide_allow = move |_| {
        if !hold {
            on_decide.call(if always {
                Decision::AllowAlways {
                    pattern: String::new(),
                }
            } else {
                Decision::AllowOnce
            });
        }
    };
    let btns = rsx! {
        div { class: "btns",
            if let Some((c, warn)) = countdown.clone() { span { class: if warn { "cd warn" } else { "cd" }, "aria-hidden": "true", "{c}" } }
            span { class: "sp" }
            button { class: if b.deny_default { "deny dflt" } else { "deny" }, r#type: "button", "aria-keyshortcuts": "Escape",
                onclick: move |_| on_decide.call(Decision::Deny { reason: None }),
                "Deny" if shortcuts { kbd { "Esc" } }
            }
            if req.allow_session && r < Risk::Critical {
                button { class: "sess", r#type: "button", disabled: b.allow_disabled, onclick: move |_| on_decide.call(Decision::AllowSession), "For this session" }
            }
            button { class: if b.deny_default { "allow" } else { "allow dflt" }, r#type: "button", style: "{allow_style}",
                "aria-disabled": "{b.allow_disabled}", "aria-keyshortcuts": "Meta+Enter",
                onclick: decide_allow,
                onpointerdown: move |_| if hold { on_hold.call(true) },
                onpointerup: move |_| if hold { on_hold.call(false) },
                if b.allow_fill.is_some() { span { class: "fill" } }
                if b.spinner { span { class: "ld" } }
                span { class: "lb", "{b.allow_label}" }
                if shortcuts && !hold { kbd { "⌘↵" } }
            }
        }
    };
    let preview = rsx! {
        if let Some(cmd) = req.command.clone() { pre { class: "cmd", "$ {cmd}" } }
        if let Some(a) = req.amount.clone() { div { class: "amt", "{a}" } }
        if !req.details.is_empty() {
            dl { class: "dt",
                for (k, v) in req.details.iter().cloned() { div { dt { "{k}" } dd { "{v}" } } }
            }
        }
        if !req.reasons.is_empty() {
            ul { class: "why", for w in req.reasons.iter() { li { "{w}" } } }
        }
    };
    let head = rsx! {
        div { class: "hd",
            span { class: "tile", Icon { name: if r >= Risk::High { "lock" } else { "hand" }, size: 16.0 } }
            div { class: "ttl",
                div { class: "tt", "{req.title}" }
                if let Some(d) = req.description.clone() { div { class: "ds", "{d}" } }
            }
            if let Some(p) = pager.clone() { span { class: "pager", button { r#type: "button", "‹" } "{p}" button { r#type: "button", "›" } } }
            {risk_badge(r)}
        }
    };
    let always_row = rsx! {
        if req.allow_always {
            label { class: "always", input { r#type: "checkbox", checked: always } "Always allow this action" }
        }
    };
    let batch_row = rsx! {
        if let Some(bt) = batch.clone() {
            div { class: "batch", span { "{bt}" } span { class: "sp" } button { r#type: "button", onclick: move |_| on_decide.call(Decision::AllowOnce), "Allow all" } button { r#type: "button", class: "q", "Deny all" } }
        }
    };
    let cls = format!(
        "oxyn-ap {variant} risk-{}{}{}",
        r.id(),
        if matches!(phase, CardPhase::Entering) {
            " entering"
        } else {
            ""
        },
        if matches!(phase, CardPhase::Submitting(_)) {
            " busy"
        } else {
            ""
        }
    );
    let failed = matches!(phase, CardPhase::SubmitFailed(_));
    let in_card = actions == "card";
    match variant.as_str() {
        "footer" => rsx! {
            div { class: "{cls}", role: "group", "aria-label": "{req.title}",
                div { class: "fline", {risk_badge(r)} if let Some(w) = req.reasons.first() { span { class: "q", "\u{201c}{w}\u{201d}" } } }
                {always_row}
                {btns}
            }
        },
        "modal" => rsx! {
            div { class: "oxyn-ap-scrim",
                div { class: "{cls}", role: "alertdialog", "aria-modal": "true", "aria-label": "{req.title}",
                    {head}
                    div { class: "pv", {preview} }
                    if r == Risk::Critical { div { class: "hatch" } }
                    {btns}
                }
            }
        },
        _ => rsx! {
            div { class: "{cls}", role: "group", "aria-label": "{req.title}", "aria-busy": "{matches!(phase, CardPhase::Submitting(_))}",
                if let Some(l) = req.label.clone() { div { class: "tray-label", "{l}" } }
                div { class: "body",
                    {head}
                    div { class: "pv", {preview} }
                    if r == Risk::Critical { div { class: "hatch" } }
                    if in_card { {always_row.clone()} {btns.clone()} }
                }
                if !in_card { div { class: "tray", {always_row} {btns} } }
                {batch_row}
                if failed { div { class: "fail", "Couldn't reach the agent" button { r#type: "button", "Retry" } } }
            }
        },
    }
}

/// A queue of requests as one paged card.
#[component]
pub fn ApprovalPager(
    /// Open requests.
    queue: ApprovalQueue,
    /// Shared clock.
    now: ClockMs,
    /// Page.
    #[props(default)]
    page: usize,
) -> Element {
    let open = queue.open();
    let Some(req) = open
        .get(page.min(open.len().saturating_sub(1)))
        .cloned()
        .cloned()
    else {
        return rsx! {};
    };
    let card = Card::new(req);
    rsx! { ApprovalCard { card, now, variant: "paged", pager: queue.pager(page), batch: queue.batch() } }
}
