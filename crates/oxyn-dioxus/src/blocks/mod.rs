//! Blocks (Oxyn Pro): larger compositions for AI and agent products. Live session, Mission
//! control, Mission timeline, Run inspector, Background agents, Agent routines, MCP servers,
//! Knowledge base, Eval suite, Agent builder, Agent gallery, AI insight card, Model arena, Prompt
//! studio, Assistant memory, Catch up, AI feature launch, AI setup wizard, Call insights and Lead
//! scoring. Rules live in
//! `oxyn_core::blocks` (shared with the GPUI view); styles in `blocks.css` (prefix `oxbk-`) on top
//! of the AI components' base classes (`oxaik` buttons, chips and segmented controls). Chrome is
//! neutral grey; colour is kept for status, data series and the official brand and model marks.
//!
//! Every block takes the host clock as `now` and keeps its own state; what moves on screen is
//! derived from `now`, so a host only advances it.

use dioxus::prelude::*;

mod arena;
mod bgagents;
mod builder;
mod call;
mod catchup;
mod evals;
mod gallery;
mod insight;
mod inspector;
mod kb;
mod launch;
mod leads;
mod mcp;
mod memory;
mod mission;
mod prompt;
mod routines;
mod session;
mod wizard;

pub use arena::ModelArena;
pub use bgagents::BackgroundAgents;
pub use builder::AgentBuilder;
pub use call::CallInsights;
pub use catchup::CatchUp;
pub use evals::EvalSuite;
pub use gallery::AgentGallery;
pub use insight::AiInsightCard;
pub use inspector::RunInspector;
pub use kb::KnowledgeBase;
pub use launch::AiFeatureLaunch;
pub use leads::LeadScoring;
pub use mcp::McpServers;
pub use memory::AssistantMemory;
pub use mission::{MissionControl, MissionTimeline};
pub use prompt::PromptStudio;
pub use routines::AgentRoutines;
pub use session::LiveSession;
pub use wizard::AiSetupWizard;

/// A switch (role switch) with a visible label.
pub(crate) fn switch(label: &str, on: bool, onclick: EventHandler<()>) -> Element {
    rsx! {
        button { class: "oxbk-sw", r#type: "button", role: "switch", "aria-checked": "{on}", "aria-label": "{label}",
            onclick: move |_| onclick.call(()),
            span { class: "oxbk-sw-t", i {} }
        }
    }
}

/// A status word with its icon in its semantic hue (never colour alone).
pub(crate) fn status(word: &str, hue: &str) -> Element {
    let icon = match hue {
        "green" => "check-circle",
        "red" => "x-circle",
        "amber" => "alert",
        "blue" => "play",
        "violet" => "merge",
        _ => "info",
    };
    rsx! { span { class: "oxbk-st", "data-hue": "{hue}", {crate::aikit::ico(icon, 12.0)} span { "{word}" } } }
}

/// A small avatar: the model's mark, or initials for a person.
pub(crate) fn avatar(brand: &str, name: &str) -> Element {
    if brand.is_empty() {
        let ini: String = name.chars().take(1).collect();
        rsx! { span { class: "oxbk-av oxbk-av-you", "aria-hidden": "true", "{ini}" } }
    } else {
        rsx! { span { class: "oxbk-av", "aria-hidden": "true", {crate::aikit::mark(brand, 14.0)} } }
    }
}
