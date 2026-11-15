//! Parity models (plan #28): the behaviour that closes the gaps between Oxyn and the affectUI,
//! Arc UI and Beautiful UI components listed in `docs/parity/status.csv`, plus the X-post
//! references. Behaviour-only rebuilds: nothing here is copied from those libraries.
//!
//! - [`compose`]: AI composer triggers, the rich-text block editor, run controls, effort levels.
//! - [`fields`]: number field, scrub input, elastic slider and morph select.
//! - [`meters`]: gauge, usage meter, split bar and both context windows.
//! - [`upload`]: dropzone rows, accept and size checks, and the avatar cropper.
//! - [`home`]: thread groups, heat maps, token usage, uploads that land in turn, run modes and
//!   experiments, document views and polling cost (the chunk 6 X-post references).
//! - [`confirm`]: confirm morph, hold to confirm, confirm popover (destructive buttons that ask).
//! - [`edit`]: a text buffer with selection and IME composition (GPUI text entry).
//! - [`input`]: text area rows and counter, changed-word rows, ghost-text autocomplete.
//! - [`gates`]: approval gates, permission requests and the agent approvals block.
//! - [`text`]: loaders and moving text (morph, slot reels, phases, action swap, streams).
//! - [`tools`]: tool calls, the tool-kind map, JSON trees and link unfurls.
//!
//! Like the rest of the core, time is passed in as [`crate::ClockMs`]; nothing reads a clock.

pub mod compose;
pub mod confirm;
pub mod edit;
pub mod fields;
pub mod fleet;
pub mod gates;
pub mod home;
pub mod input;
pub mod meters;
pub mod plan;
pub mod text;
pub mod tools;
pub mod upload;

/// Smoothstep ease used for rewinds and fills (0..1 in, 0..1 out).
pub fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `m:ss` for a number of seconds.
pub fn mss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Splits `text` on backticks: `(segment, is_code)` pairs, so a title like
/// ``"Run `vercel deploy --prod`"`` sets its command in the mono face.
pub fn code_spans(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for (i, part) in text.split('`').enumerate() {
        if !part.is_empty() {
            out.push((part.to_string(), i % 2 == 1));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_time() {
        assert_eq!(
            code_spans("Run `ls -la` now"),
            vec![
                ("Run ".to_string(), false),
                ("ls -la".to_string(), true),
                (" now".to_string(), false)
            ]
        );
        assert_eq!(mss(62), "1:02");
        assert!((smooth(0.5) - 0.5).abs() < 1e-6);
    }
}
