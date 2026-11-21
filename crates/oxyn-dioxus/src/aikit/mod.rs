//! AI special components (Oxyn Pro): Agent timeline, Model card, AI workflow, Image generation,
//! Speech player, Meeting summary, Reply draft, Human handoff, PII redaction, LLM cost
//! calculator, Provider keys, Generation stats, AI translate, TL;DR toggle, Connector setup,
//! Enrich column, Ticket router, Ask your data, Agent doc cursor, Provider status, Training run,
//! Tone tuner, AI mascot, Image reveal, AI spark, Pixel mascot, Ghost mascot, Cube mascot,
//! Upgrade prompt, Referral card, Trial banner, Live visitors and Social proof toast. Rules live in `oxyn_core::aikit` (shared with the GPUI
//! view); styles in `aikit.css` (prefix `oxaik-`). Chrome is neutral grey; colour is kept for
//! status, data series and the official brand and model marks.
//!
//! Every component takes the host clock as `now` and keeps its own state; what moves on screen
//! (streamed words, a playhead, progress) is derived from `now`, so a host only advances it.

use dioxus::prelude::*;

use crate::brand::BrandIcon;
use crate::composer::icons::Icon;

mod ask;
mod connector;
mod cost;
mod doccursor;
mod enrich;
mod genstats;
mod growth;
mod handoff;
mod imagegen;
mod keys;
mod mascots;
mod meeting;
mod model_card;
mod pii;
mod provstatus;
mod reply;
mod reveal;
mod router;
mod speech;
mod timeline;
mod tldr;
mod tone;
mod training;
mod translate;
mod workflow;

pub use ask::AskYourData;
pub use connector::ConnectorSetup;
pub use cost::LlmCostCalculator;
pub use doccursor::AgentDocCursor;
pub use enrich::EnrichColumn;
pub use genstats::GenerationStats;
pub use growth::{LiveVisitors, ReferralCard, SocialProofToast, TrialBanner, UpgradePrompt};
pub use handoff::HumanHandoff;
pub use imagegen::ImageGeneration;
pub use keys::ProviderKeys;
pub use mascots::{AiMascot, AiSpark, CubeMascot, GhostMascot, PixelMascot};
pub use meeting::MeetingSummary;
pub use model_card::ModelCard;
pub use pii::PiiRedaction;
pub use provstatus::ProviderStatus;
pub use reply::ReplyDraft;
pub use reveal::ImageReveal;
pub use router::TicketRouter;
pub use speech::SpeechPlayer;
pub use timeline::AgentTimeline;
pub use tldr::TldrToggle;
pub use tone::ToneTuner;
pub use training::TrainingRun;
pub use translate::AiTranslate;
pub use workflow::AiWorkflow;

/// The polite live region every component announces through.
pub(crate) fn live(said: &str) -> Element {
    rsx! { div { class: "oxaik-sr", role: "status", "aria-live": "polite", "{said}" } }
}

/// A brand or model mark in its own colours (decorative).
pub(crate) fn mark(name: &str, size: f32) -> Element {
    rsx! { span { class: "oxaik-mk", "aria-hidden": "true", BrandIcon { name: name.to_string(), size, fallback: true } } }
}

/// A neutral icon.
pub(crate) fn ico(name: &str, size: f32) -> Element {
    rsx! { span { class: "oxaik-ic", "aria-hidden": "true", Icon { name: name.to_string(), size } } }
}

/// A small dot spinner (3 × 3) for busy states.
pub(crate) fn dots() -> Element {
    rsx! {
        span { class: "oxaik-dots", "aria-hidden": "true",
            for i in 0..9 { i { style: "--i:{i}" } }
        }
    }
}

/// `class` plus `on` when set.
pub(crate) fn cls(base: &str, on: bool) -> String {
    if on {
        format!("{base} on")
    } else {
        base.to_string()
    }
}

/// The key name of a keyboard event (`ArrowLeft`, `Enter`, ` `).
pub(crate) fn key_of(e: &KeyboardEvent) -> String {
    e.key().to_string()
}

/// Cmd on macOS or Ctrl elsewhere is held.
pub(crate) fn cmd(e: &KeyboardEvent) -> bool {
    let m = e.modifiers();
    m.meta() || m.ctrl()
}

/// A segmented radio group (roving): options, the selected index and the press id stem.
pub(crate) fn seg(
    label: &str,
    opts: Vec<String>,
    sel: usize,
    on_pick: EventHandler<usize>,
    on_key: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "oxaik-seg", role: "radiogroup", "aria-label": "{label}",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End") {
                    e.prevent_default();
                    on_key.call(k);
                }
            },
            for (i, o) in opts.into_iter().enumerate() {
                button { r#type: "button", role: "radio", class: cls("oxaik-sg", i == sel), "aria-checked": "{i == sel}",
                    tabindex: if i == sel { "0" } else { "-1" },
                    onclick: move |_| on_pick.call(i),
                    "{o}"
                }
            }
        }
    }
}
