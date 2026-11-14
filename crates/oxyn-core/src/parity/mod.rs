//! Parity models (plan #28): the behaviour that closes the gaps between Oxyn and the affectUI,
//! Arc UI and Beautiful UI components listed in `docs/parity/status.csv`, plus the X-post
//! references. Behaviour-only rebuilds: nothing here is copied from those libraries.
//!
//! - [`confirm`]: confirm morph, hold to confirm, confirm popover (destructive buttons that ask).
//! - [`gates`]: approval gates, permission requests and the agent approvals block.
//!
//! Like the rest of the core, time is passed in as [`crate::ClockMs`]; nothing reads a clock.

pub mod confirm;
pub mod fleet;
pub mod gates;
pub mod plan;

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
