//! The renderer-free model behind `StatusIndicator` (`docs/specs/status-indicator.md`).
//!
//! Both renderers (GPUI and Dioxus) call [`indicator`] with the current [`AgentState`] and draw
//! what it returns, so the two can never disagree about a colour, a word or a motion.

use std::time::Duration;

use oxyn_tokens::{ColorRole, motion};

use crate::{
    ClockMs,
    state::{AgentState, StallLevel},
    verbs::{ActivityKind, VerbPack, error_label, format_summary, summary},
};

/// How the dot moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotMotion {
    /// No motion.
    Still,
    /// Opacity loop between `min_opacity` and 1 with this period (busy states).
    Pulse {
        /// Loop period.
        period: Duration,
        /// Lowest opacity.
        min_opacity: u8,
    },
    /// Slow breathe for "waiting for you" (calmer than busy).
    Breathe {
        /// Loop period.
        period: Duration,
    },
    /// Colour ramps from the stalled-from state's colour to the stall colour, once.
    Ramp {
        /// The colour it ramps from.
        from: ColorRole,
        /// Ramp length (`dur.stall.ramp`).
        over: Duration,
    },
}

/// Everything a renderer needs to draw one indicator.
#[derive(Clone, Debug, PartialEq)]
pub struct IndicatorView {
    /// Dot colour role.
    pub color: ColorRole,
    /// Visible label (also the accessible name).
    pub label: String,
    /// Dot motion (renderers drop it under reduced motion).
    pub motion: DotMotion,
    /// Coarse state class for the live region: the label is only announced when this changes,
    /// never on every timer tick.
    pub announce_key: &'static str,
    /// Show a hollow ring instead of a filled dot (idle, composing).
    pub hollow: bool,
}

/// Lowest pulse opacity, percent (`loop.pulse`: opacity 0.45 ↔ 1).
pub const PULSE_MIN_OPACITY: u8 = 45;

fn busy_pulse() -> DotMotion {
    DotMotion::Pulse {
        period: motion::loop_::PULSE,
        min_opacity: PULSE_MIN_OPACITY,
    }
}

/// The colour role for a state (stalled states use `state.stalled`).
pub fn color_role(state: &AgentState) -> ColorRole {
    match state {
        AgentState::Idle => ColorRole::StateIdle,
        AgentState::Composing => ColorRole::StateComposing,
        AgentState::Submitted => ColorRole::StateSubmitted,
        AgentState::Thinking { .. } => ColorRole::StateThinking,
        AgentState::Streaming => ColorRole::StateStreaming,
        AgentState::ToolRunning { .. } => ColorRole::StateToolRunning,
        AgentState::AwaitingApproval { .. } => ColorRole::StateAwaitingApproval,
        AgentState::Queued { .. } => ColorRole::StateQueued,
        AgentState::Steering => ColorRole::StateSteering,
        AgentState::Done { .. } => ColorRole::StateDone,
        AgentState::Error { .. } => ColorRole::StateError,
        AgentState::Cancelled { .. } => ColorRole::StateCancelled,
        AgentState::Stalled { .. } => ColorRole::StateStalled,
    }
}

/// The present-tense label for a live state.
fn live_label(state: &AgentState, pack: &VerbPack) -> String {
    match state {
        AgentState::Idle => "Idle".into(),
        AgentState::Composing => "Drafting".into(),
        AgentState::Submitted => "Sending".into(),
        AgentState::Thinking { header: Some(h) } => h.clone(),
        AgentState::Thinking { header: None } => pack.activity(ActivityKind::Think).present,
        AgentState::Streaming => pack.activity(ActivityKind::Write).present,
        AgentState::ToolRunning { concurrent, .. } if *concurrent > 1 => {
            format!(
                "{} {concurrent} tools",
                pack.activity(ActivityKind::Run).present
            )
        }
        AgentState::ToolRunning { tool, .. } => {
            let verb = pack.activity(tool.activity).present;
            match &tool.target {
                Some(t) => format!("{verb} {t}"),
                None => verb,
            }
        }
        AgentState::AwaitingApproval { .. } => pack.activity(ActivityKind::WaitUser).present,
        AgentState::Queued { count } => format!("{count} queued"),
        AgentState::Steering => "Steering".into(),
        AgentState::Done { .. } => "Done".into(),
        AgentState::Error { kind } => format!("Stopped · {}", error_label(kind)),
        AgentState::Cancelled { .. } => "Interrupted".into(),
        AgentState::Stalled { was, .. } => live_label(was, pack),
    }
}

/// The view for `state` at time `now` with the given verb pack.
pub fn indicator(state: &AgentState, pack: &VerbPack, now: ClockMs) -> IndicatorView {
    let color = color_role(state);
    let (label, motion, announce_key) = match state {
        AgentState::Idle => (live_label(state, pack), DotMotion::Still, "idle"),
        AgentState::Composing => (live_label(state, pack), DotMotion::Still, "composing"),
        AgentState::Submitted => (
            live_label(state, pack),
            DotMotion::Pulse {
                period: motion::loop_::BASE,
                min_opacity: PULSE_MIN_OPACITY,
            },
            "submitted",
        ),
        AgentState::Thinking { .. } => (live_label(state, pack), busy_pulse(), "thinking"),
        AgentState::Streaming => (live_label(state, pack), busy_pulse(), "streaming"),
        AgentState::ToolRunning { .. } => (live_label(state, pack), busy_pulse(), "tool"),
        AgentState::Queued { .. } => (live_label(state, pack), busy_pulse(), "queued"),
        AgentState::Steering => (live_label(state, pack), busy_pulse(), "steering"),
        AgentState::AwaitingApproval { .. } => (
            live_label(state, pack),
            DotMotion::Breathe {
                period: motion::loop_::CALM,
            },
            "waiting",
        ),
        AgentState::Done { .. } | AgentState::Error { .. } | AgentState::Cancelled { .. } => (
            summary(state, pack).unwrap_or_else(|| live_label(state, pack)),
            DotMotion::Still,
            "finished",
        ),
        AgentState::Stalled { since, level, was } => {
            let ramp = DotMotion::Ramp {
                from: color_role(was),
                over: motion::dur::stall::RAMP,
            };
            match level {
                StallLevel::Soft => (live_label(was, pack), ramp, "stalled-soft"),
                StallLevel::Hard => (
                    format!("No response for {}", format_summary(now.since(*since))),
                    DotMotion::Still,
                    "stalled-hard",
                ),
            }
        }
    };
    IndicatorView {
        color,
        label,
        motion,
        announce_key,
        hollow: matches!(state, AgentState::Idle | AgentState::Composing),
    }
}

/// Dot opacity in `[0, 1]` for a motion at loop phase `phase` (`MotionClock::phase(period)`), or
/// ramp progress for [`DotMotion::Ramp`]. Reduced motion → always 1.
pub fn dot_opacity(motion: DotMotion, phase: f32, reduced: bool) -> f32 {
    use std::f32::consts::TAU;
    if reduced {
        return 1.0;
    }
    match motion {
        DotMotion::Still | DotMotion::Ramp { .. } => 1.0,
        DotMotion::Pulse { min_opacity, .. } => {
            let lo = f32::from(min_opacity) / 100.0;
            // cosine: full at phase 0, lowest at 0.5; smooth, never a hard blink.
            lo + (1.0 - lo) * (0.5 + 0.5 * (phase * TAU).cos())
        }
        DotMotion::Breathe { .. } => 0.7 + 0.3 * (0.5 + 0.5 * (phase * TAU).cos()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentEvent, AgentMachine, ErrorKind, RequestRef, ToolRef};

    fn view(s: &AgentState) -> IndicatorView {
        indicator(s, VerbPack::neutral(), ClockMs(60_000))
    }

    #[test]
    fn every_state_has_a_label_and_colour() {
        let tool = ToolRef {
            id: "t".into(),
            name: "bash".into(),
            activity: ActivityKind::Run,
            target: Some("cargo test".into()),
        };
        let cases = [
            (AgentState::Idle, "Idle"),
            (AgentState::Composing, "Drafting"),
            (AgentState::Submitted, "Sending"),
            (AgentState::Thinking { header: None }, "Thinking"),
            (
                AgentState::Thinking {
                    header: Some("Checking auth".into()),
                },
                "Checking auth",
            ),
            (AgentState::Streaming, "Writing"),
            (
                AgentState::ToolRunning {
                    tool: tool.clone(),
                    concurrent: 1,
                },
                "Running cargo test",
            ),
            (
                AgentState::ToolRunning {
                    tool,
                    concurrent: 3,
                },
                "Running 3 tools",
            ),
            (
                AgentState::AwaitingApproval {
                    request: RequestRef {
                        id: "r".into(),
                        tool: None,
                    },
                },
                "Waiting for you",
            ),
            (AgentState::Queued { count: 2 }, "2 queued"),
            (AgentState::Steering, "Steering"),
            (
                AgentState::Done {
                    elapsed: Duration::from_secs(72),
                    waited: Duration::ZERO,
                },
                "Worked for 1m 12s",
            ),
            (
                AgentState::Done {
                    elapsed: Duration::from_millis(400),
                    waited: Duration::ZERO,
                },
                "Done",
            ),
            (
                AgentState::Error {
                    kind: ErrorKind::Offline,
                },
                "Stopped · offline",
            ),
            (
                AgentState::Cancelled {
                    after: Duration::from_secs(42),
                },
                "Interrupted after 42s",
            ),
        ];
        for (s, label) in cases {
            assert_eq!(view(&s).label, label, "{s:?}");
        }
    }

    #[test]
    fn stalls_keep_the_verb_then_say_how_long() {
        let mut m = AgentMachine::default();
        m.apply(AgentEvent::Submit, ClockMs(0));
        m.apply(AgentEvent::Thinking { header: None }, ClockMs(0));
        let soft = view(m.tick(ClockMs(6_000)));
        assert_eq!(soft.label, "Thinking");
        assert_eq!(soft.color, ColorRole::StateStalled);
        assert!(matches!(
            soft.motion,
            DotMotion::Ramp {
                from: ColorRole::StateThinking,
                ..
            }
        ));
        let hard = indicator(
            m.tick(ClockMs(45_000)),
            VerbPack::neutral(),
            ClockMs(45_000),
        );
        assert_eq!(hard.label, "No response for 45s");
        assert_eq!(hard.announce_key, "stalled-hard");
    }

    #[test]
    fn pulse_respects_photosensitivity_and_reduced_motion() {
        for s in [
            AgentState::Submitted,
            AgentState::Streaming,
            AgentState::Steering,
        ] {
            if let DotMotion::Pulse { period, .. } = view(&s).motion {
                assert!(period >= motion::loop_::LUMINANCE_MIN_PERIOD, "{s:?}");
            } else {
                panic!("{s:?} should pulse");
            }
        }
        let p = busy_pulse();
        assert!((dot_opacity(p, 0.0, false) - 1.0).abs() < 1e-6);
        assert!((dot_opacity(p, 0.5, false) - 0.45).abs() < 1e-6);
        assert_eq!(dot_opacity(p, 0.5, true), 1.0);
    }

    #[test]
    fn idle_reads_hollow() {
        assert!(view(&AgentState::Idle).hollow);
        assert!(!view(&AgentState::Streaming).hollow);
    }
}
