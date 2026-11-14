//! Chat models (plan #27): streaming markdown, reasoning and thinking traces, citations and
//! sources, response feedback, thread grouping, starter prompts, research runs, document
//! extraction and support conversations.
//!
//! - [`markdown`]: tolerant block / inline parser for streaming text and the reveal clock.
//! - [`reasoning`]: the reasoning block state machine, step parsing and Beautiful UI traces.
//! - [`sources`]: source cards, citations, claim provenance, context and recommendation cards,
//!   document extraction.
//! - [`thread`]: response feedback, chat-thread grouping, starter prompts, research run, support.
//!
//! Behaviour follows the Arc UI and affectUI docs (rebuilt; no source used) and Beautiful UI's MIT
//! Thinking, Chat, Context Cards and Recommendation Card (© 2026 Shane Levine).
pub mod markdown;
pub mod reasoning;
pub mod sources;
pub mod thread;
