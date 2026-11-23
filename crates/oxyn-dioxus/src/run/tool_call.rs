//! `ToolCallRow` (`tool-call.md`) and `ToolStack` (`tools.md` §3, §6.1).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::collections::BTreeSet;
use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs, VerbPack,
    run::{
        GroupState, McpStatus, RowAction, RowGlyph, StackItem, Tone, ToolCall, ToolDensity,
        ToolStackVariant, redact, row_view,
        stack::{Entry, layout},
    },
};

use crate::composer::icons::Icon;

pub(crate) fn tone(t: Tone) -> &'static str {
    match t {
        Tone::Muted => "muted",
        Tone::Default => "default",
        Tone::Success => "success",
        Tone::Danger => "danger",
        Tone::Waiting => "waiting",
        Tone::Warn => "warn",
    }
}

fn glyph(g: RowGlyph, animate: bool) -> Element {
    match g {
        RowGlyph::Loader => rsx! { span { class: if animate { "ld" } else { "ld still" } } },
        RowGlyph::Hollow => rsx! { span { class: "hollow" } },
        RowGlyph::Kind(k) => rsx! { Icon { name: k.icon(), size: 14.0 } },
        RowGlyph::Check => {
            rsx! { span { class: "draw", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
        }
        RowGlyph::Cross => rsx! { Icon { name: "x", size: 14.0, stroke: 2.0 } },
        RowGlyph::Stop => rsx! { Icon { name: "stop", size: 11.0 } },
        RowGlyph::Denied => rsx! { Icon { name: "ban", size: 14.0 } },
        RowGlyph::Waiting => rsx! { Icon { name: "hand", size: 14.0 } },
        RowGlyph::Key => rsx! { Icon { name: "key", size: 14.0 } },
    }
}

fn mcp_dot(s: McpStatus) -> Option<&'static str> {
    match s {
        McpStatus::Connected => None,
        McpStatus::Connecting => Some("info"),
        McpStatus::NeedsAuth => Some("waiting"),
        McpStatus::Disabled => Some("muted"),
        McpStatus::ToolsFetchFailed | McpStatus::Failed => Some("danger"),
    }
}

/// One tool call.
#[component]
pub fn ToolCallRow(
    /// The call.
    call: ToolCall,
    /// Shared clock.
    now: ClockMs,
    /// `row`, `card`, `terminal` or `minimal`.
    #[props(default = "row".to_string())]
    variant: String,
    /// Expanded (`None` = the system decides: approval and failure open).
    #[props(default)]
    expanded: Option<bool>,
    /// Header pressed.
    #[props(default)]
    on_toggle: EventHandler<String>,
    /// Retry / Stop / Connect pressed.
    #[props(default)]
    on_action: EventHandler<(String, RowAction)>,
) -> Element {
    let pack = VerbPack::neutral();
    let v = row_view(&call, pack, now);
    let open = expanded.unwrap_or(v.auto_expand && v.error_line.is_none());
    let id = call.id.clone();
    let id2 = call.id.clone();
    let cls = format!(
        "oxyn-tc {variant} t-{}{}{}",
        tone(v.tone),
        if open { " open" } else { "" },
        if v.animate { " enter" } else { "" }
    );
    if variant == "terminal" {
        let bullet = match v.glyph {
            RowGlyph::Loader | RowGlyph::Hollow => "•",
            g => g.text(),
        };
        let blink = matches!(v.glyph, RowGlyph::Loader) && (now.0 / 600) % 2 == 1;
        return rsx! {
            div { class: "{cls}", role: "listitem", "aria-label": "{v.aria}",
                div { class: "tl",
                    span { class: "b", style: if blink { "opacity:.25" } else { "" }, "{bullet}" }
                    span { class: "vb", "{v.verb}" }
                    if let Some(t) = v.target.clone() { span { class: "tg", " {t}" } }
                    if let Some(p) = v.preview.clone() { span { class: "pv", "  {p}" } }
                    if let Some(d) = v.duration.clone() { span { class: "du", " · {d}" } }
                }
                if let Some(n) = v.note.clone() { div { class: "tc-sub", "└ {n}" } }
                if let Some(e) = v.error_line.clone() { div { class: "tc-sub te", "└ {e}" } }
                if let Some(s) = v.stalled.clone() { div { class: "tc-sub tw", "└ {s}" } }
            }
        };
    }
    let minimal = variant == "minimal";
    let input = call.args.as_ref().map(|a| redact(a).pretty(12));
    let output = call.output.clone().map(|o| {
        let lines: Vec<&str> = o.lines().collect();
        let more = lines.len().saturating_sub(200);
        (
            lines.into_iter().take(200).collect::<Vec<_>>().join("\n"),
            more,
        )
    });
    let roll = v.roll.clone();
    let lines = v.lines;
    rsx! {
        div { class: "{cls}", role: "listitem",
            button { class: "hd", r#type: "button", "aria-expanded": "{open}", "aria-label": "{v.aria}",
                style: "--nudge:{v.nudge_px:.1}px",
                onclick: move |_| on_toggle.call(id.clone()),
                if !minimal { span { class: "gl", {glyph(v.glyph, v.animate)} } }
                if minimal && matches!(v.glyph, RowGlyph::Cross) { span { class: "gl", "✕" } }
                span { class: "vb",
                    if let Some((old, p)) = roll {
                        span { class: "roll", style: "--p:{p:.2}", span { class: "old", "{old}" } span { class: "new", "{v.verb}" } }
                    } else {
                        span { class: if v.shimmer { "shim" } else { "" }, "{v.verb}" }
                    }
                }
                if let Some(t) = v.target.clone() { span { class: if v.struck { "tg struck" } else { "tg" }, "{t}" } }
                if let Some((a, r)) = lines { span { class: "ln", span { class: "add", "+{a}" } span { class: "del", "\u{2212}{r}" } } }
                if let Some(p) = v.preview.clone() { span { class: if v.shimmer { "pv shim" } else { "pv" }, "{p}" } }
                if let Some(n) = v.note.clone() { span { class: "nt", "{n}" } }
                span { class: "sp" }
                if let Some(s) = v.stalled.clone() { span { class: "stall", "{s}" } }
                if let Some(a) = v.attempt.clone() { span { class: "att", "{a}" } }
                if let Some((name, st)) = v.server.clone() {
                    span { class: "mcp", title: "{name}",
                        span { class: "mk", "{name.chars().next().unwrap_or('?')}" }
                        "{name}"
                        if let Some(d) = mcp_dot(st) { span { class: "dot {d}" } }
                    }
                }
                if let Some(d) = v.duration.clone() { span { class: "du", if v.paused { Icon { name: "pause", size: 10.0, stroke: 2.0 } } "{d}" } }
                if !minimal { span { class: "chev", Icon { name: "chevron", size: 12.0 } } }
            }
            if let Some(e) = v.error_line.clone() {
                div { class: "err", role: "alert",
                    span { "{e}" }
                    for a in v.actions.clone() {
                        {
                            let id = id2.clone();
                            rsx! { button { r#type: "button", class: "act", onclick: move |_| on_action.call((id.clone(), a)), "{a.label()}" } }
                        }
                    }
                }
            } else if !v.actions.is_empty() {
                div { class: "acts",
                    for a in v.actions.clone() {
                        {
                            let id = id2.clone();
                            rsx! { button { r#type: "button", class: "act", onclick: move |_| on_action.call((id.clone(), a)), "{a.label()}" } }
                        }
                    }
                }
            }
            if open {
                div { class: "detail",
                    if let Some((text, more)) = input {
                        section { "aria-label": "Input",
                            div { class: "ph", "Input" span { class: "cp", Icon { name: "copy", size: 12.0 } } }
                            pre { "{text}" }
                            if more > 0 { div { class: "more", "Show all ({more} more lines)" } }
                        }
                    }
                    if let Some((text, more)) = output {
                        section { "aria-label": "Output", "aria-busy": "{call.state.is_live()}",
                            div { class: "ph", "Output" span { class: "cp", Icon { name: "copy", size: 12.0 } } }
                            pre { "{text}" }
                            if more > 0 { div { class: "more", "Open full output ({more} more lines)" } }
                        }
                    }
                }
            }
        }
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
