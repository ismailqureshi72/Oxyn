//! Agent micro-interactions first seen in posts the founders saved (C2a, free): Test ticks,
//! Triage metro, Loop pipeline, Secret card, Split confirm, Agent prompt copy, Proactive card,
//! Write sweep, Done shockwave and Publish popover. Rules live in `oxyn_core::xpost` (shared with
//! the GPUI `XPostView`); styles in `xpost.css` (prefix `oxxp-`). Chrome is neutral grey; colour is
//! kept for status and the official brand marks.
//!
//! Every component takes the host clock as `now` and keeps its own state; what moves on screen is
//! derived from `now`, so a host only advances it.

use dioxus::prelude::*;

use crate::brand::BrandIcon;
use crate::composer::icons::Icon;

mod moments;
mod run;
mod trust;

pub use moments::{DoneShockwave, ProactiveCard, PublishPopover, WriteSweep};
pub use run::{LoopPipeline, TestTicks, TriageMetro};
pub use trust::{AgentPromptCopy, SecretCard, SplitConfirm};

/// The polite live region every component announces through.
pub(crate) fn live(said: &str) -> Element {
    rsx! { div { class: "oxxp-sr", role: "status", "aria-live": "polite", "{said}" } }
}

/// A brand mark in its own colours (decorative).
pub(crate) fn mark(name: &str, size: f32) -> Element {
    rsx! { span { class: "oxxp-mk", "aria-hidden": "true", BrandIcon { name: name.to_string(), size, fallback: true } } }
}

/// A neutral icon.
pub(crate) fn ico(name: &str, size: f32) -> Element {
    rsx! { span { class: "oxxp-ic", "aria-hidden": "true", Icon { name: name.to_string(), size } } }
}

/// `class` plus `on` when set.
pub(crate) fn cls(base: &str, on: bool) -> String {
    if on {
        format!("{base} on")
    } else {
        base.to_string()
    }
}

/// The normalised key of a keyboard event (`ArrowLeft`, `Enter`, `Space`; `undo` for Cmd/Ctrl+Z).
pub(crate) fn key_of(e: &KeyboardEvent) -> String {
    let k = e.key().to_string();
    let m = e.modifiers();
    if (m.meta() || m.ctrl()) && k.eq_ignore_ascii_case("z") {
        return "undo".into();
    }
    oxyn_core::aikit::norm_key(&k).to_string()
}

/// Keys every component handles itself (so the page doesn't scroll).
pub(crate) fn owned(k: &str) -> bool {
    matches!(
        k,
        "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" | "Space" | "undo"
    )
}
