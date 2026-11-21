//! Blocks (Oxyn Pro): larger compositions for AI and agent products, each a whole working surface
//! built from the components. Like [`crate::aikit`], every block is a UI-free model the Dioxus
//! components and the GPUI view share: time comes in as `now` (ms), what moves on screen is
//! derived from it, presses are string ids (`press(id, now)`) and keys are key names
//! (`key(name, now)`); both return the line the polite live region should announce.
//!
//! Agents:
//! - [`session`]: Live session: a team of agents at work, their conversation, plan and context.
//! - [`mission`]: Fleet overview and Fleet timeline: every agent, what happened, what needs you.
//! - [`inspector`]: Run inspector: one multi-agent run as a tree with call bars and numbers.
//! - [`bgagents`]: Background agents: coding tasks from queued to ready for review and a PR.
//! - [`routines`]: Agent routines: scheduled agents as sentences with a countdown and run ticks.
//! - [`mcp`]: MCP servers: the servers behind an agent's tools and what they cost in context.
//! - [`kb`]: Knowledge base: sources through parse, chunk, embed and index, then test retrieval.
//! - [`evals`]: Eval suite: a candidate against its baseline, case by case.
//! - [`builder`]: Agent builder: configure, test and publish one agent.
//! - [`gallery`]: Agent gallery: agent templates by team with preview and install.
//!
//! AI:
//! - [`insight`]: AI insight card: a KPI that explains its own change.
//! - [`arena`]: Blind compare: two models answer one prompt blind; vote, then reveal.
//! - [`prompt`]: Prompt bench: a template with variables, test runs, versions and a diff.
//! - [`memory`]: Assistant memory: what an assistant remembers, to edit, forget or add.
//! - [`catchup`]: Catch up: a ranked digest of what happened in your tools.
//! - [`launch`]: AI feature launch: a tour card with a mini demo and an admin switch.
//! - [`wizard`]: AI setup wizard: connect sources, set behaviour, test and go live.
//! - [`call`]: Call insights: talk time, key moments, summary and next steps of a sales call.
//! - [`leads`]: Lead scoring: inbound leads ranked by fit score and pushed to a CRM.

pub mod arena;
pub mod bgagents;
pub mod builder;
pub mod call;
pub mod catchup;
pub mod evals;
pub mod gallery;
pub mod insight;
pub mod inspector;
pub mod kb;
pub mod launch;
pub mod leads;
pub mod mcp;
pub mod memory;
pub mod mission;
pub mod prompt;
pub mod routines;
pub mod session;
pub mod wizard;

pub use crate::aikit::{first_words, hash, norm_key, nth, rove, thousands, usd, words_at};

/// Page slugs of the blocks, in gallery order.
pub const BLOCK_SLUGS: [&str; 20] = [
    "live-session",
    "fleet-overview",
    "fleet-timeline",
    "run-inspector",
    "background-agents",
    "agent-routines",
    "mcp-servers",
    "knowledge-base",
    "eval-suite",
    "agent-builder",
    "agent-gallery",
    "ai-insight-card",
    "blind-compare",
    "prompt-bench",
    "assistant-memory",
    "catch-up",
    "ai-feature-launch",
    "ai-setup-wizard",
    "call-insights",
    "lead-scoring",
];

/// A duration as `2m 14s`, `14s` or `1h 05m`.
pub fn dur(ms: u64) -> String {
    let s = ms / 1000;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

/// A duration in words for screen readers (`2 minutes 14 seconds`).
pub fn dur_words(ms: u64) -> String {
    let s = ms / 1000;
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    let part = |n: u64, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
    let mut out = Vec::new();
    if h > 0 {
        out.push(part(h, "hour"));
    }
    if m > 0 {
        out.push(part(m, "minute"));
    }
    if sec > 0 || out.is_empty() {
        out.push(part(sec, "second"));
    }
    out.join(" ")
}

/// Minutes ago in words (`just now`, `12 min ago`, `3 h ago`).
pub fn ago(ms: u64) -> String {
    let m = ms / 60_000;
    if m < 1 {
        "just now".into()
    } else if m < 60 {
        format!("{m} min ago")
    } else {
        format!("{} h ago", m / 60)
    }
}

/// Fraction of `d` that has passed `elapsed` ms after a start, clamped to 0..=1.
pub fn frac(elapsed: u64, d: u64) -> f64 {
    (elapsed as f64 / d.max(1) as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!(dur(134_000), "2m 14s");
        assert_eq!(dur(9_000), "9s");
        assert_eq!(dur(3_900_000), "1h 05m");
        assert_eq!(dur_words(134_000), "2 minutes 14 seconds");
        assert_eq!(dur_words(1_000), "1 second");
        assert_eq!(ago(30_000), "just now");
        assert_eq!(ago(720_000), "12 min ago");
        assert_eq!(frac(5, 10), 0.5);
        assert_eq!(BLOCK_SLUGS.len(), 20);
    }
}
