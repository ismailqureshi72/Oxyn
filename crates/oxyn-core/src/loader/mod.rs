//! The loader engine (`docs/specs/loaders.md` §2–3): loader phases derived from [`AgentState`],
//! a tempo-continuous beat clock, show/hide hysteresis, busy-region arbitration and the glyph
//! (T1) renderer-free frame tables.
//!
//! Renderers own no timing: they hold a [`GlyphLoader`], call [`GlyphLoader::update`] with the
//! current state, and draw the [`GlyphFrame`] it returns.

mod busy;
mod glyph;
mod tempo;

use std::time::Duration;

pub use busy::{BusyArbiter, BusyPriority, BusyRegion, LIST_ANIMATION_CAP, LoaderId};
pub use glyph::{FREE_GLYPH_TABLES, GlyphFrame, GlyphKind, GlyphLoader, GlyphTable, Still};
pub use tempo::TempoClock;

use oxyn_tokens::ColorRole;

use crate::{
    ActivityKind,
    state::{AgentState, StallLevel},
};

/// Shared loader timings (`docs/specs/loaders.md` §3.3 and §5).
pub mod timing {
    use std::time::Duration;

    /// A thinking phase shorter than this never shows a loader.
    pub const SHOW_DELAY: Duration = Duration::from_millis(300);
    /// Once shown, a loader stays at least this long.
    pub const MIN_VISIBLE: Duration = Duration::from_millis(600);
    /// Tempo changes ease over this long (phase-continuous).
    pub const TEMPO_EASE: Duration = Duration::from_millis(400);
    /// Stall slow-down and colour ramp.
    pub const STALL_RAMP: Duration = Duration::from_millis(2000);
    /// Colour snap-back when an event ends a stall.
    pub const RECOVER: Duration = Duration::from_millis(200);
    /// Submitted: one bright frame dimming to 60 %.
    pub const SUBMITTED: Duration = Duration::from_millis(400);
    /// Done: `✓` hold before clearing.
    pub const DONE_HOLD: Duration = Duration::from_millis(600);
    /// Cancelled: frozen frame at 40 % before clearing.
    pub const CANCEL_CLEAR: Duration = Duration::from_millis(160);
    /// Enter fade for glyphs.
    pub const ENTER: Duration = Duration::from_millis(120);
}

/// What every loader is doing, derived from [`AgentState`] (`loaders.md` §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoaderPhase {
    /// Nothing to show.
    Hidden,
    /// One pulse after send, no loop.
    Submitted,
    /// Calm loop (1.0×).
    Thinking,
    /// Brisker loop (1.25×) while a tool runs.
    Working(ActivityKind),
    /// Slower loop (0.6×) while tokens stream.
    Streaming,
    /// A different motion while the agent waits on the user.
    Waiting,
    /// Slowing (0.5×) and ramping toward `warn`.
    StalledSoft,
    /// Stopped on the still frame in `warn`.
    StalledHard,
    /// `✓`, then cleared.
    Done,
    /// `✕`, static until dismissed.
    Failed,
    /// Frozen and dimmed, then cleared.
    Cancelled,
}

impl LoaderPhase {
    /// The phase for a state. `Queued` and `Steering` keep the loader thinking: the queue stack
    /// and the composer show those, not the loader.
    pub fn from_state(state: &AgentState) -> Self {
        match state {
            AgentState::Idle | AgentState::Composing => Self::Hidden,
            AgentState::Submitted => Self::Submitted,
            AgentState::Thinking { .. } | AgentState::Queued { .. } | AgentState::Steering => {
                Self::Thinking
            }
            AgentState::ToolRunning { tool, .. } => Self::Working(tool.activity),
            AgentState::Streaming => Self::Streaming,
            AgentState::AwaitingApproval { .. } => Self::Waiting,
            AgentState::Stalled {
                level: StallLevel::Soft,
                ..
            } => Self::StalledSoft,
            AgentState::Stalled {
                level: StallLevel::Hard,
                ..
            } => Self::StalledHard,
            AgentState::Done { .. } => Self::Done,
            AgentState::Error { .. } => Self::Failed,
            AgentState::Cancelled { .. } => Self::Cancelled,
        }
    }

    /// Loop speed relative to the variant's base tempo.
    pub fn tempo(self) -> f32 {
        match self {
            Self::Working(_) => 1.25,
            Self::Streaming => 0.6,
            Self::StalledSoft => 0.5,
            Self::Hidden | Self::StalledHard | Self::Done | Self::Failed | Self::Cancelled => 0.0,
            Self::Submitted | Self::Thinking | Self::Waiting => 1.0,
        }
    }

    /// True for phases that cycle frames.
    pub fn loops(self) -> bool {
        matches!(
            self,
            Self::Thinking | Self::Working(_) | Self::Streaming | Self::Waiting | Self::StalledSoft
        )
    }

    /// True while a run is in flight (anything between send and the end of the run).
    pub fn active(self) -> bool {
        self.loops() || matches!(self, Self::Submitted | Self::StalledHard)
    }

    /// Colour role (`color.loader.*`).
    pub fn color(self) -> ColorRole {
        match self {
            Self::Waiting => ColorRole::LoaderWaiting,
            Self::StalledSoft | Self::StalledHard => ColorRole::LoaderWarn,
            Self::Done => ColorRole::LoaderSuccess,
            Self::Failed => ColorRole::LoaderDanger,
            Self::Hidden | Self::Cancelled | Self::Streaming => ColorRole::LoaderFg,
            Self::Submitted | Self::Thinking | Self::Working(_) => ColorRole::LoaderAccent,
        }
    }

    /// How long the colour takes to reach this phase's role from the previous one.
    pub fn color_ease(self, from: LoaderPhase) -> Duration {
        match (from, self) {
            (_, Self::StalledSoft) => timing::STALL_RAMP,
            (Self::StalledSoft | Self::StalledHard, _) => timing::RECOVER,
            _ => timing::TEMPO_EASE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolRef;

    #[test]
    fn mapping_follows_the_spec_table() {
        let tool = ToolRef {
            id: "t".into(),
            name: "grep".into(),
            activity: ActivityKind::Search,
            target: None,
        };
        assert_eq!(
            LoaderPhase::from_state(&AgentState::Idle),
            LoaderPhase::Hidden
        );
        assert_eq!(
            LoaderPhase::from_state(&AgentState::ToolRunning {
                tool,
                concurrent: 1
            }),
            LoaderPhase::Working(ActivityKind::Search)
        );
        assert_eq!(
            LoaderPhase::from_state(&AgentState::Queued { count: 1 }),
            LoaderPhase::Thinking
        );
        assert_eq!(LoaderPhase::Working(ActivityKind::Run).tempo(), 1.25);
        assert_eq!(LoaderPhase::Streaming.tempo(), 0.6);
        assert!(!LoaderPhase::StalledHard.loops() && LoaderPhase::StalledHard.active());
    }
}
