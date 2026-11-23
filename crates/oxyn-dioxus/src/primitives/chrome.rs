//! App chrome (plan #27): `UserMenu`, `PresenceDot`, `TeamSwitcher`, `WorkspaceTile`,
//! `NotificationCenter` and `CodeBlock`.
//!
//! Logic lives in `oxyn_core::primitives::chrome`. Menus and panels render below their trigger
//! inside a positioned wrapper (no portal), so they work in contained previews. A menu that
//! starts open draws no click-outside layer and takes no focus; only a user-opened menu does.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        chrome::{
            Note, NoteView, NotificationCenter as CenterModel, Presence, SWITCH_DELAY, SwitcherRow,
            SwitcherView, TeamSwitcher as SwitcherModel, ThemePref, UserMenu as MenuModel, UserRow,
            Workspace, account_label, code_collapses, highlight, language_label,
        },
        layout::initials,
        nav::NavKey,
    },
};

use super::{CopyButton, cls, nav_key};
use crate::composer::icons::Icon;

// ─────────────────────────────────────── Code block ──

/// Code block: a source viewer with a filename header, language label, light highlighting, a
/// [`CopyButton`] and optional collapse (`max_lines` → Show all, `aria-expanded`). The `pre` is
/// focusable so it can scroll. Ports Beautiful UI's Code Block header (MIT, © 2026 Shane Levine)
/// and Arc UI's behaviour. Oxyn plus: line numbers, highlighted lines, `diff`, wrap toggle and a
/// streaming caret.
#[component]
pub fn CodeBlock(
    /// Source.
    code: String,
    /// File name.
    #[props(default)]
    filename: Option<String>,
    /// Language id (`rust` · `ts` · `tsx` · `py` · `sh` · `json` · `diff`).
    #[props(default = "tsx".to_string())]
    language: String,
    /// Collapse beyond this many lines (0 = never).
    #[props(default)]
    max_lines: usize,
    /// Line numbers.
    #[props(default)]
    line_numbers: bool,
    /// 1-based lines to highlight.
    #[props(default)]
    highlight_lines: Vec<usize>,
    /// Wrap long lines.
    #[props(default)]
    wrap: bool,
    /// Streaming caret after the last line.
    #[props(default)]
    streaming: bool,
    /// `default` · `bare` (no header).
    #[props(default = "default".to_string())]
    variant: String,
) -> Element {
    let mut open = use_signal(|| false);
    let mut wrapping = use_signal(|| wrap);
    let lines = highlight(&code, &language);
    let collapses = code_collapses(&code, max_lines);
    let shown = if collapses && !open() {
        max_lines
    } else {
        lines.len()
    };
    let name = filename.clone().unwrap_or_else(|| "Source code".into());
    let id = format!(
        "oxyn-code-{}",
        name.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    );
    rsx! {
        section { class: cls(&["oxyn-code", &variant, if wrapping() { "cb-wrapped" } else { "" }, if collapses && !open() { "clamped" } else { "" }]), "aria-label": "{name} source code",
            if variant != "bare" {
                div { class: "cb-hd",
                    Icon { name: "code", size: 13.0 }
                    span { class: "cb-fn", "{name}" }
                    span { class: "cb-lang", "{language_label(&language)}" }
                    span { class: "cb-sp" }
                    button { class: "cb-wrap", r#type: "button", "aria-pressed": "{wrapping()}", "aria-label": "Wrap lines", title: "Wrap lines", onclick: move |_| { let w = !wrapping(); wrapping.set(w) }, Icon { name: "text", size: 13.0 } }
                    CopyButton { value: code.clone(), icon_only: true, variant: "plain", size: "sm" }
                }
            }
            pre { class: "cb-pre", id: "{id}", tabindex: "0",
                code {
                    for (i, ln) in lines.iter().take(shown).enumerate() {
                        span { key: "{i}", class: cls(&["ln", if highlight_lines.contains(&(i + 1)) { "is-hl" } else { "" }, ln.first().map(|t| match t.0 { oxyn_core::primitives::chrome::Tok::Add => "add", oxyn_core::primitives::chrome::Tok::Del => "del", _ => "" }).unwrap_or("")]),
                            if line_numbers { span { class: "no", "aria-hidden": "true", "{i + 1}" } }
                            span { class: "src",
                                for (k, (t, s)) in ln.iter().enumerate() { span { key: "{k}", class: "t-{t.id()}", "{s}" } }
                                if streaming && i + 1 == shown { span { class: "caret", "aria-hidden": "true" } }
                                "\n"
                            }
                        }
                    }
                }
            }
            if collapses {
                button { class: "cb-more", r#type: "button", "aria-expanded": "{open()}", "aria-controls": "{id}", onclick: move |_| { let o = !open(); open.set(o) },
                    for _k in std::iter::once(open()) { span { key: "{_k}", class: "cb-l", if open() { "Show less" } else { "Show all {lines.len()} lines" } } }
                    Icon { name: "chevron", size: 12.0 }
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
