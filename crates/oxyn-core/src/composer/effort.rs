//! Effort control model (`docs/specs/composer-effort-control.md`): the effort ladder, per-model
//! support, stepping, clamping on model switch, next-turn semantics and ignition with cooldown.

use super::mid_run;
use crate::{AgentState, ClockMs};

/// The ordered effort ladder owned by Oxyn; providers map their labels onto it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffortLevel {
    /// Minimal.
    Minimal,
    /// Low.
    Low,
    /// Medium.
    Medium,
    /// High.
    High,
    /// Extra high.
    XHigh,
    /// Max.
    Max,
    /// Ultra.
    Ultra,
}

impl EffortLevel {
    /// Display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Minimal => "Minimal",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::XHigh => "Extra high",
            Self::Max => "Max",
            Self::Ultra => "Ultra",
        }
    }

    /// Short label for tight chips (`Med`).
    pub fn short(self) -> &'static str {
        match self {
            Self::Minimal => "Min",
            Self::Low => "Low",
            Self::Medium => "Med",
            Self::High => "High",
            Self::XHigh => "XHigh",
            Self::Max => "Max",
            Self::Ultra => "Ultra",
        }
    }

    /// One-line hint in the menu.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Minimal => "Fastest, almost no reasoning",
            Self::Low => "Faster",
            Self::Medium => "Balanced",
            Self::High => "Deeper reasoning, slower",
            Self::XHigh | Self::Max => "Much deeper, more tokens",
            Self::Ultra => "Deepest, slowest",
        }
    }
}

/// How a model exposes reasoning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ReasoningSupport {
    /// No adjustable reasoning.
    #[default]
    None,
    /// Thinking on or off.
    Toggle,
    /// An ordered subset of the ladder.
    Levels {
        /// Allowed levels, ascending.
        allowed: Vec<EffortLevel>,
        /// The model's default.
        default: EffortLevel,
    },
}

/// The chosen effort.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffortSetting {
    /// A level.
    Level(EffortLevel),
    /// The model or router chooses; `resolved` once reported.
    Auto {
        /// Level reported for this turn.
        resolved: Option<EffortLevel>,
    },
    /// Binary thinking.
    Toggle(bool),
}

impl Default for EffortSetting {
    fn default() -> Self {
        Self::Level(EffortLevel::Medium)
    }
}

impl EffortSetting {
    /// Chip label: `High`, `Auto · High`, `Thinking on`.
    pub fn label(&self) -> String {
        match self {
            Self::Level(l) => l.label().into(),
            Self::Auto { resolved: None } => "Auto".into(),
            Self::Auto { resolved: Some(l) } => format!("Auto · {}", l.label()),
            Self::Toggle(true) => "Thinking on".into(),
            Self::Toggle(false) => "Thinking off".into(),
        }
    }

    /// The level, if one is set or resolved.
    pub fn level(&self) -> Option<EffortLevel> {
        match self {
            Self::Level(l) => Some(*l),
            Self::Auto { resolved } => *resolved,
            Self::Toggle(_) => None,
        }
    }
}

/// Effort control variants (`composer-effort-control.md` §6, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EffortVariant {
    /// Inline segmented control of allowed levels.
    Segmented,
    /// Chip with four signal bars + label; menu on click.
    #[default]
    SignalBars,
    /// Detented slider, "Faster ← → Smarter".
    Slider,
    /// The prompt glyph `›` / `»` is the indicator.
    Chevrons,
    /// Binary "Thinking" switch.
    Toggle,
    /// Plain "Effort: High ⌄" text chip.
    Menu,
}

impl EffortVariant {
    /// Every free variant.
    pub const ALL: [EffortVariant; 6] = [
        Self::Segmented,
        Self::SignalBars,
        Self::Slider,
        Self::Chevrons,
        Self::Toggle,
        Self::Menu,
    ];

    /// Short id; registry id is `effort-control/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Segmented => "segmented",
            Self::SignalBars => "signal-bars",
            Self::Slider => "slider",
            Self::Chevrons => "chevrons",
            Self::Toggle => "toggle",
            Self::Menu => "menu",
        }
    }

    /// Parses `slider` or `effort-control/slider`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("effort-control/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
}

/// Clamps to the nearest allowed level at or below `cur`; never raises cost silently.
pub fn clamp_on_model_switch(cur: EffortLevel, allowed: &[EffortLevel]) -> EffortLevel {
    allowed
        .iter()
        .copied()
        .filter(|l| *l <= cur)
        .max()
        .or_else(|| allowed.first().copied())
        .unwrap_or(cur)
}

/// One step up or down within `allowed` (`Alt+.` / `Alt+,`). Stops at the ends.
pub fn step(cur: EffortLevel, allowed: &[EffortLevel], up: bool) -> EffortLevel {
    let i = allowed.iter().position(|l| *l == cur);
    match (i, up) {
        (Some(i), true) => allowed.get(i + 1).copied().unwrap_or(cur),
        (Some(i), false) => i.checked_sub(1).map(|j| allowed[j]).unwrap_or(cur),
        (None, _) => clamp_on_model_switch(cur, allowed),
    }
}

/// Signal bars lit for `level` (of 4).
pub fn bars(level: EffortLevel, allowed: &[EffortLevel]) -> u8 {
    let n = allowed.len().max(1);
    let i = allowed.iter().position(|l| *l == level).unwrap_or(0);
    if n == 1 {
        return 4;
    }
    (1 + (i * 3 + (n - 1) / 2) / (n - 1)) as u8
}

/// When raising effort plays the ignition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IgnitePolicy {
    /// Never.
    Off,
    /// The top `n` allowed levels.
    TopTiers(u8),
    /// From this level up.
    From(EffortLevel),
}

impl Default for IgnitePolicy {
    fn default() -> Self {
        Self::TopTiers(2)
    }
}

impl IgnitePolicy {
    /// Ignite tier of `level`: 0 none, 1 = tier A (amber `›`), 2 = tier B (violet `»`).
    pub fn tier(self, level: EffortLevel, allowed: &[EffortLevel]) -> u8 {
        let top = |n: u8| -> u8 {
            let i = allowed.iter().position(|l| *l == level);
            let len = allowed.len();
            match i {
                Some(i) if len >= 2 && i + usize::from(n) >= len => {
                    (n as usize - (len - 1 - i)) as u8
                }
                _ => 0,
            }
        };
        match self {
            Self::Off => 0,
            Self::TopTiers(n) => top(n.min(2)),
            Self::From(from) if level >= from => {
                if allowed.last() == Some(&level) {
                    2
                } else {
                    1
                }
            }
            Self::From(_) => 0,
        }
    }
}

/// Ignition timing (`composer-effort-control.md` §5).
pub mod timing {
    use std::time::Duration;

    /// Tier A ignition.
    pub const IGNITE_A: Duration = Duration::from_millis(900);
    /// Tier B ignition.
    pub const IGNITE_B: Duration = Duration::from_millis(1600);
    /// Discharge on lowering.
    pub const DISCHARGE: Duration = Duration::from_millis(320);
    /// No replay of the same tier within this window.
    pub const COOLDOWN: Duration = Duration::from_secs(10);
    /// `Clamped` note on screen.
    pub const CLAMP_NOTE: Duration = Duration::from_millis(2400);
}

/// A one-shot visual the control is playing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffortFx {
    /// Colour bands wash the composer; `t` in `[0, 1]`.
    Igniting {
        /// 1 or 2.
        tier: u8,
        /// Progress.
        t: f32,
    },
    /// Tint fades back to neutral.
    Discharge {
        /// Progress.
        t: f32,
    },
}

/// What the control shows this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct EffortView {
    /// The level in effect.
    pub setting: EffortSetting,
    /// Waiting for the next turn (dashed `→ Max` badge).
    pub pending: Option<EffortSetting>,
    /// Inline note (`Ultra isn't available on Sonnet 5`).
    pub note: Option<String>,
    /// Ignition or discharge.
    pub fx: Option<EffortFx>,
    /// Tier of the setting in effect (prompt glyph colour).
    pub tier: u8,
}

/// The control's state machine.
#[derive(Clone, Debug)]
pub struct EffortControl {
    setting: EffortSetting,
    pending: Option<EffortSetting>,
    note: Option<(String, ClockMs)>,
    fx: Option<(EffortFx, ClockMs)>,
    last_ignite: [Option<ClockMs>; 2],
    /// Ignition policy.
    pub ignite: IgnitePolicy,
    /// Reduced motion: no ignition.
    pub reduced: bool,
}

impl EffortControl {
    /// A control at `setting`.
    pub fn new(setting: EffortSetting) -> Self {
        Self {
            setting,
            pending: None,
            note: None,
            fx: None,
            last_ignite: [None; 2],
            ignite: IgnitePolicy::default(),
            reduced: false,
        }
    }

    /// The setting in effect.
    pub fn setting(&self) -> &EffortSetting {
        &self.setting
    }

    /// Commits a new setting. Mid-run it waits for the next turn; otherwise it applies and may
    /// ignite (raise into a tier, not replayed within 10 s) or discharge (lower).
    pub fn commit(
        &mut self,
        new: EffortSetting,
        agent: &AgentState,
        support: &ReasoningSupport,
        now: ClockMs,
    ) {
        if mid_run(agent) {
            self.pending = (new != self.setting).then_some(new);
            return;
        }
        self.apply(new, support, now);
    }

    fn apply(&mut self, new: EffortSetting, support: &ReasoningSupport, now: ClockMs) {
        let allowed = allowed(support);
        let (old, newl) = (self.setting.level(), new.level());
        self.setting = new;
        self.pending = None;
        if self.reduced {
            return;
        }
        if let (Some(o), Some(n)) = (old, newl) {
            if n > o {
                let tier = self.ignite.tier(n, &allowed);
                if tier > 0 {
                    let slot = usize::from(tier - 1);
                    let fresh =
                        self.last_ignite[slot].is_none_or(|t| now.since(t) >= timing::COOLDOWN);
                    if fresh {
                        self.last_ignite[slot] = Some(now);
                        self.fx = Some((EffortFx::Igniting { tier, t: 0.0 }, now));
                    }
                }
            } else if n < o && self.ignite.tier(o, &allowed) > 0 {
                self.fx = Some((EffortFx::Discharge { t: 0.0 }, now));
            }
        }
    }

    /// Steps one level (`Alt+.` / `Alt+,`).
    pub fn step(&mut self, up: bool, agent: &AgentState, support: &ReasoningSupport, now: ClockMs) {
        let allowed = allowed(support);
        let base = self
            .pending
            .as_ref()
            .and_then(EffortSetting::level)
            .or(self.setting.level());
        if let Some(cur) = base {
            let next = step(cur, &allowed, up);
            if Some(next) != base {
                self.commit(EffortSetting::Level(next), agent, support, now);
            }
        }
    }

    /// The run ended: a pending setting takes effect.
    pub fn run_ended(&mut self, support: &ReasoningSupport, now: ClockMs) {
        if let Some(p) = self.pending.take() {
            self.apply(p, support, now);
        }
    }

    /// The model changed: clamps (never raising) and notes why for 2.4 s.
    pub fn model_switched(&mut self, support: &ReasoningSupport, model: &str, now: ClockMs) {
        match support {
            ReasoningSupport::Levels { allowed, .. } => {
                if let Some(cur) = self.setting.level() {
                    let to = clamp_on_model_switch(cur, allowed);
                    if to != cur {
                        self.note =
                            Some((format!("{} isn't available on {model}", cur.label()), now));
                        self.setting = EffortSetting::Level(to);
                    }
                } else if matches!(self.setting, EffortSetting::Toggle(_)) {
                    self.setting = EffortSetting::default();
                }
            }
            ReasoningSupport::Toggle => {
                let on = !matches!(self.setting.level(), Some(EffortLevel::Minimal));
                self.setting = EffortSetting::Toggle(on);
            }
            ReasoningSupport::None => {}
        }
        self.pending = None;
    }

    /// The frame to draw.
    pub fn view(&mut self, support: &ReasoningSupport, now: ClockMs) -> EffortView {
        if self
            .note
            .as_ref()
            .is_some_and(|(_, at)| now.since(*at) >= timing::CLAMP_NOTE)
        {
            self.note = None;
        }
        let fx = self.fx.and_then(|(fx, at)| {
            let dur = match fx {
                EffortFx::Igniting { tier: 2, .. } => timing::IGNITE_B,
                EffortFx::Igniting { .. } => timing::IGNITE_A,
                EffortFx::Discharge { .. } => timing::DISCHARGE,
            };
            let t = now.since(at).as_secs_f32() / dur.as_secs_f32();
            (t < 1.0).then_some(match fx {
                EffortFx::Igniting { tier, .. } => EffortFx::Igniting { tier, t },
                EffortFx::Discharge { .. } => EffortFx::Discharge { t },
            })
        });
        if fx.is_none() {
            self.fx = None;
        }
        let tier = self
            .setting
            .level()
            .map(|l| self.ignite.tier(l, &allowed(support)))
            .unwrap_or(0);
        EffortView {
            setting: self.setting.clone(),
            pending: self.pending.clone(),
            note: self.note.as_ref().map(|(n, _)| n.clone()),
            fx,
            tier,
        }
    }

    /// Frames are needed (an effect is playing or a note is up).
    pub fn animating(&self) -> bool {
        self.fx.is_some()
    }
}

/// Allowed levels of `support` (empty for none or toggle).
pub fn allowed(support: &ReasoningSupport) -> Vec<EffortLevel> {
    match support {
        ReasoningSupport::Levels { allowed, .. } => allowed.clone(),
        _ => Vec::new(),
    }
}

/// The prompt glyph for a tier (`›`, `›` amber, `»` violet).
pub fn prompt_glyph(tier: u8) -> &'static str {
    if tier >= 2 { "»" } else { "›" }
}

/// Ignition band offset for a travelling wash, in `[0, 1]` across the composer.
pub fn band_position(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use EffortLevel::*;

    fn lv(allowed: &[EffortLevel]) -> ReasoningSupport {
        ReasoningSupport::Levels {
            allowed: allowed.to_vec(),
            default: Medium,
        }
    }

    #[test]
    fn clamp_never_raises() {
        assert_eq!(clamp_on_model_switch(Ultra, &[Low, Medium, High]), High);
        assert_eq!(clamp_on_model_switch(Minimal, &[Low, Medium]), Low);
        assert_eq!(step(Medium, &[Low, Medium, High], true), High);
        assert_eq!(step(High, &[Low, Medium, High], true), High);
        assert_eq!(bars(Low, &[Low, Medium, High, Max]), 1);
        assert_eq!(bars(Max, &[Low, Medium, High, Max]), 4);
        assert_eq!(bars(Medium, &[Low, Medium, High]), 3);
    }

    #[test]
    fn tiers() {
        let a = [Low, Medium, High, Max];
        let p = IgnitePolicy::default();
        assert_eq!(p.tier(Medium, &a), 0);
        assert_eq!(p.tier(High, &a), 1);
        assert_eq!(p.tier(Max, &a), 2);
    }

    #[test]
    fn ignition_cooldown_and_next_turn() {
        let s = lv(&[Low, Medium, High, Max]);
        let mut c = EffortControl::new(EffortSetting::Level(Medium));
        c.commit(EffortSetting::Level(Max), &AgentState::Idle, &s, ClockMs(0));
        assert!(matches!(
            c.view(&s, ClockMs(100)).fx,
            Some(EffortFx::Igniting { tier: 2, .. })
        ));
        assert_eq!(c.view(&s, ClockMs(2_000)).fx, None);
        c.commit(
            EffortSetting::Level(Medium),
            &AgentState::Idle,
            &s,
            ClockMs(2_100),
        );
        assert!(matches!(
            c.view(&s, ClockMs(2_150)).fx,
            Some(EffortFx::Discharge { .. })
        ));
        c.commit(
            EffortSetting::Level(Max),
            &AgentState::Idle,
            &s,
            ClockMs(3_000),
        );
        assert_eq!(c.view(&s, ClockMs(3_100)).fx, None, "cooldown");

        c.commit(
            EffortSetting::Level(Low),
            &AgentState::Streaming,
            &s,
            ClockMs(4_000),
        );
        let v = c.view(&s, ClockMs(4_000));
        assert_eq!(v.setting, EffortSetting::Level(Max));
        assert_eq!(v.pending, Some(EffortSetting::Level(Low)));
        c.run_ended(&s, ClockMs(5_000));
        assert_eq!(c.setting(), &EffortSetting::Level(Low));
    }

    #[test]
    fn model_switch_clamps_with_note() {
        let mut c = EffortControl::new(EffortSetting::Level(Ultra));
        c.model_switched(&lv(&[Low, Medium, High]), "Swift 4", ClockMs(0));
        let v = c.view(&lv(&[Low, Medium, High]), ClockMs(100));
        assert_eq!(v.setting, EffortSetting::Level(High));
        assert_eq!(v.note.as_deref(), Some("Ultra isn't available on Swift 4"));
        assert_eq!(c.view(&lv(&[Low, Medium, High]), ClockMs(2_500)).note, None);
        for v in EffortVariant::ALL {
            assert_eq!(EffortVariant::from_id(v.id()), Some(v));
        }
    }
}
