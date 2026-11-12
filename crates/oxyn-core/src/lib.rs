//! UI-free core for Oxyn components. No renderer dependencies; builds for `wasm32-unknown-unknown`.
//!
//! - [`state`]: the one [`AgentState`] every component reads, and the [`AgentMachine`] that folds
//!   adapter events into it (including the derived `Stalled` state).
//! - [`clock`]: the shared [`MotionClock`], one per window, with the frame-rate policy from
//!   `docs/motion.md` §9.
//! - [`ease`]: cubic-Bézier easing evaluation for the `motion::ease::*` tokens.
//! - [`verbs`]: verb packs (present and past tense), the activity table and duration phrases.
//!
//! Time is passed in, never read: hosts call `apply`/`tick`/`advance` with a monotonic
//! [`ClockMs`] from their own clock (GPUI's frame time, `performance.now()` on the web). That keeps
//! everything deterministic and testable, and avoids `std::time::Instant`, which panics on
//! `wasm32-unknown-unknown`.
//!
//! ```
//! use oxyn_core::{AgentEvent, AgentMachine, AgentState, ClockMs, StallLevel};
//!
//! let mut m = AgentMachine::default();
//! m.apply(AgentEvent::Submit, ClockMs(0));
//! m.apply(AgentEvent::TextDelta, ClockMs(300));
//! m.tick(ClockMs(6_000)); // 5.7 s of silence
//! assert!(matches!(m.state(), AgentState::Stalled { level: StallLevel::Soft, .. }));
//! m.apply(AgentEvent::Finished, ClockMs(7_000));
//! assert_eq!(oxyn_core::verbs::summary(m.state(), oxyn_core::verbs::VerbPack::neutral()).as_deref(), Some("Worked for 7s"));
//! ```

pub mod clock;
pub mod ease;
pub mod indicator;
pub mod loader;
pub mod state;
mod time;
pub mod verbs;

pub use clock::{MotionClock, MotionKind, WindowState};
pub use state::{
    AgentEvent, AgentMachine, AgentState, ErrorKind, RequestRef, StallLevel, StallPolicy, ToolRef,
};
pub use time::ClockMs;
pub use verbs::{ActivityKind, VerbPack, VerbPair};

/// Crate version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
