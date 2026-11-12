//! `verb-cycler` (`docs/specs/verb-cycler.md`).

use std::time::Duration;

use super::{ease_in, ease_out, reading_weight, timing};
use crate::{
    ActivityKind, ClockMs, VerbPack,
    state::AgentState,
    verbs::{self, Order},
};

/// The five free cycler variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CyclerVariant {
    /// 80 ms out with a 4 px lift, 140 ms in from +4 px (≈ 180 ms total).
    #[default]
    Fade,
    /// Old slides up and out (160 ms), new slides up from below (200 ms), clipped.
    Slide,
    /// Blur out 0 → 6 px (120 ms), blur in 6 → 0 px (180 ms).
    BlurSwap,
    /// `Fade` plus an animated three-dot suffix (1200 ms loop).
    Ellipsis,
    /// No flavour cycling; pinned and state changes crossfade in 80 ms.
    Static,
}

impl CyclerVariant {
    /// Parses a variant id (`fade` or `verb-cycler/fade`).
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.strip_prefix("verb-cycler/").unwrap_or(id) {
            "fade" => Self::Fade,
            "slide" => Self::Slide,
            "blur-swap" => Self::BlurSwap,
            "ellipsis" => Self::Ellipsis,
            "static" => Self::Static,
            _ => return None,
        })
    }

    /// (out ms, in delay ms, in ms).
    fn timing(self, reduced: bool) -> (u64, u64, u64) {
        if reduced {
            return (80, 0, 80);
        }
        match self {
            Self::Fade | Self::Ellipsis => (80, 40, 140),
            Self::Slide => (160, 0, 200),
            Self::BlurSwap => (120, 60, 180),
            Self::Static => (80, 0, 80),
        }
    }
}

/// Selection and dwell rules (`verb-cycler.md` §3).
mod rules {
    use std::time::Duration;
    pub const PIN_SHOW: Duration = Duration::from_millis(300);
    pub const PIN_MIN: Duration = Duration::from_millis(900);
    pub const GAP_AFTER_CHANGE: Duration = Duration::from_millis(1200);
    pub const HEADER_GAP: Duration = Duration::from_millis(1500);
    pub const LONG_RUN: Duration = Duration::from_secs(300);
}

/// What the cycler shows at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct CyclerView {
    /// The verb (and target) now shown; also the accessible name during a swap.
    pub text: String,
    /// The outgoing text during a swap.
    pub outgoing: Option<String>,
    /// Outgoing progress `[0, 1]` (eased): drives its fade, lift, slide or blur.
    pub out_t: f32,
    /// Incoming progress `[0, 1]` (eased).
    pub in_t: f32,
    /// Opacity of the three ellipsis dots (`Ellipsis` only, else 0).
    pub dots: [f32; 3],
    /// True once the run has ended (past tense, interrupted, stopped).
    pub settled: bool,
    /// True for "Waiting for you": the host must not shimmer it.
    pub paused: bool,
}

#[derive(Clone, Debug)]
struct Swap {
    from: String,
    at: ClockMs,
}

/// A verb cycler instance.
#[derive(Clone, Debug)]
pub struct VerbCycler {
    variant: CyclerVariant,
    pack: VerbPack,
    rng: u64,
    bag: Vec<usize>,
    last_flavour: Option<usize>,
    shown: String,
    swap: Option<Swap>,
    shown_at: ClockMs,
    /// The text the state asks for (activity, pin, past tense) and since when.
    wanted: Option<(String, ClockMs, bool)>,
    flavour: bool,
    pin_shown: bool,
    run_at: Option<ClockMs>,
    settled: bool,
    paused: bool,
}

impl VerbCycler {
    /// A cycler over `pack`. `seed` should be a hash of the run id (stable across windows).
    pub fn new(variant: CyclerVariant, pack: VerbPack, seed: u64) -> Self {
        Self {
            variant,
            pack,
            rng: seed | 1,
            bag: Vec::new(),
            last_flavour: None,
            shown: String::new(),
            swap: None,
            shown_at: ClockMs(0),
            wanted: None,
            flavour: false,
            pin_shown: false,
            run_at: None,
            settled: false,
            paused: false,
        }
    }

    /// The variant.
    pub fn variant(&self) -> CyclerVariant {
        self.variant
    }

    /// The pack.
    pub fn pack(&self) -> &VerbPack {
        &self.pack
    }

    fn next_rand(&mut self) -> u64 {
        // xorshift64*
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_flavour(&mut self) -> Option<String> {
        let n = self.pack.cycle.len();
        if n == 0 {
            return None;
        }
        let idx = match self.pack.order {
            Order::Sequential => self.last_flavour.map_or(0, |i| (i + 1) % n),
            Order::Shuffle => {
                if self.bag.is_empty() {
                    self.bag = (0..n).collect();
                    for i in (1..n).rev() {
                        let j = (self.next_rand() % (i as u64 + 1)) as usize;
                        self.bag.swap(i, j);
                    }
                    // First of a new bag ≠ last of the old one.
                    if n > 1 && self.bag.last().copied() == self.last_flavour {
                        self.bag.swap(0, n - 1);
                    }
                }
                self.bag.pop().unwrap_or(0)
            }
        };
        self.last_flavour = Some(idx);
        Some(self.pack.cycle[idx].present.clone())
    }

    /// What the state asks to show: (text, pinned, settled, paused). `None` hides the cycler.
    fn wanted_for(&self, state: &AgentState) -> Option<(String, bool, bool, bool)> {
        let p = &self.pack;
        Some(match state {
            AgentState::Idle | AgentState::Composing => return None,
            AgentState::Submitted => (p.activity(ActivityKind::Think).present, false, false, false),
            AgentState::Thinking { header: Some(h) } => (h.clone(), true, false, false),
            AgentState::Thinking { header: None }
            | AgentState::Queued { .. }
            | AgentState::Steering => {
                (p.activity(ActivityKind::Think).present, false, false, false)
            }
            AgentState::ToolRunning { tool, concurrent } => {
                let text = if *concurrent > 1 {
                    format!("Running {concurrent} tools")
                } else {
                    let verb = p.activity(tool.activity).present;
                    match &tool.target {
                        Some(t) => format!("{verb} {t}"),
                        None => verb,
                    }
                };
                (text, true, false, false)
            }
            AgentState::Streaming => (p.activity(ActivityKind::Write).present, true, false, false),
            AgentState::AwaitingApproval { .. } => ("Waiting for you".into(), true, false, true),
            // Stalls freeze the last verb; status-text escalates the words.
            AgentState::Stalled { .. } => (self.shown.clone(), true, false, false),
            AgentState::Done { .. } | AgentState::Cancelled { .. } | AgentState::Error { .. } => {
                (verbs::summary(state, p)?, true, true, false)
            }
        })
    }

    fn dwell(&self, now: ClockMs, reduced: bool) -> Duration {
        let ms = 2800f32.max(1200.0 + 70.0 * reading_weight(&self.shown));
        let mut d = Duration::from_millis(ms as u64).max(timing::VERB_DWELL);
        if self.run_at.is_some_and(|r| now.since(r) >= rules::LONG_RUN) {
            d = d.mul_f32(1.5);
        }
        if reduced {
            d *= 2;
        }
        d
    }

    fn show(&mut self, text: String, now: ClockMs) {
        if text == self.shown {
            return;
        }
        let from = std::mem::replace(&mut self.shown, text);
        self.swap = (!from.is_empty()).then_some(Swap { from, at: now });
        self.shown_at = now;
    }

    fn reset(&mut self) {
        self.bag.clear();
        self.last_flavour = None;
        self.shown.clear();
        self.swap = None;
        self.wanted = None;
        self.flavour = false;
        self.pin_shown = false;
        self.run_at = None;
        self.settled = false;
        self.paused = false;
    }

    /// Feeds the agent state. Call every frame; cheap when nothing changed.
    pub fn update(&mut self, state: &AgentState, now: ClockMs, reduced: bool) {
        let Some((text, pinned, settled, paused)) = self.wanted_for(state) else {
            self.reset();
            return;
        };
        self.run_at.get_or_insert(now);
        self.settled = settled;
        self.paused = paused;
        let changed = self.wanted.as_ref().is_none_or(|(w, _, _)| *w != text);
        if changed {
            self.wanted = Some((text, now, pinned));
        }
        let Some((want, since, pinned)) = self.wanted.clone() else {
            return;
        };
        // A shown pin holds at least PIN_MIN so it is never gone unread.
        let held = self.pin_shown && now.since(self.shown_at) < rules::PIN_MIN;
        if want != self.shown {
            let go = if self.shown.is_empty() || settled || paused {
                true
            } else if pinned {
                // Pins wait PIN_SHOW so quick tool bursts never flash.
                let header_wait = matches!(state, AgentState::Thinking { header: Some(_) })
                    && now.since(self.shown_at) < rules::HEADER_GAP;
                now.since(since) >= rules::PIN_SHOW && !held && !header_wait
            } else {
                // The activity verb returns after a pin, but a flavour verb cycling over the
                // same activity stays until its dwell ends.
                !held && !(self.flavour && !changed)
            };
            if go {
                self.flavour = false;
                self.pin_shown = pinned;
                self.show(want.clone(), now);
            }
        }
        let on_activity = self.shown == want || self.flavour;
        let cycles = self.variant != CyclerVariant::Static
            && !pinned
            && !settled
            && !paused
            && !held
            && on_activity;
        if cycles
            && now.since(self.shown_at) >= self.dwell(now, reduced)
            && now.since(since) >= rules::GAP_AFTER_CHANGE
            && let Some(v) = self.next_flavour()
        {
            self.flavour = true;
            self.pin_shown = false;
            self.show(v, now);
        }
    }

    /// The frame to draw, or `None` when hidden.
    pub fn view(&self, now: ClockMs, reduced: bool) -> Option<CyclerView> {
        if self.shown.is_empty() {
            return None;
        }
        let (out_ms, in_delay, in_ms) = self.variant.timing(reduced);
        let (outgoing, out_t, in_t) = match &self.swap {
            Some(s) => {
                let e = now.since(s.at).as_millis() as u64;
                let out_t = ease_in(e as f32 / out_ms as f32);
                let in_t = ease_out(e.saturating_sub(in_delay) as f32 / in_ms as f32);
                let out = (out_t < 1.0).then(|| s.from.clone());
                (out, out_t, if e < in_delay { 0.0 } else { in_t })
            }
            None => (None, 1.0, 1.0),
        };
        let dots = if self.variant == CyclerVariant::Ellipsis && !self.settled && !self.paused {
            if reduced {
                [1.0; 3]
            } else {
                // Each dot fades 0.3 → 1 over 300 ms in turn, hold, reset: 1200 ms loop.
                let t = (now.0 % 1200) as f32;
                std::array::from_fn(|i| {
                    let start = i as f32 * 300.0;
                    0.3 + 0.7 * ease_out((t - start) / 300.0).max(0.0)
                })
            }
        } else {
            [0.0; 3]
        };
        Some(CyclerView {
            text: self.shown.clone(),
            outgoing,
            out_t,
            in_t,
            dots,
            settled: self.settled,
            paused: self.paused,
        })
    }

    /// True while a swap or the ellipsis is animating (repaint ≈ 30–60 Hz), else the host can
    /// wait for the next state change or dwell.
    pub fn animating(&self, now: ClockMs, reduced: bool) -> bool {
        let (_, d, i) = self.variant.timing(reduced);
        let swapping = self
            .swap
            .as_ref()
            .is_some_and(|s| now.since(s.at).as_millis() as u64 <= d + i);
        swapping
            || self.variant == CyclerVariant::Ellipsis
                && !reduced
                && !self.settled
                && !self.shown.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolRef;

    fn tool(target: &str) -> AgentState {
        AgentState::ToolRunning {
            tool: ToolRef {
                id: target.into(),
                name: "read".into(),
                activity: ActivityKind::Read,
                target: Some(target.into()),
            },
            concurrent: 1,
        }
    }

    fn cycler(v: CyclerVariant) -> VerbCycler {
        VerbCycler::new(v, VerbPack::neutral().clone(), 42)
    }

    #[test]
    fn first_verb_is_the_activity_then_flavours_cycle_without_repeats() {
        let mut c = cycler(CyclerVariant::Fade);
        let th = AgentState::Thinking { header: None };
        c.update(&th, ClockMs(0), false);
        assert_eq!(c.view(ClockMs(0), false).unwrap().text, "Thinking");
        let mut seen = Vec::new();
        let mut t = 0;
        while seen.len() < 12 {
            t += 100;
            c.update(&th, ClockMs(t), false);
            let v = c.view(ClockMs(t), false).unwrap().text;
            if seen.last() != Some(&v) && v != "Thinking" {
                seen.push(v);
            }
        }
        let mut uniq = seen.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), 12, "{seen:?}");
        assert!(t >= 12 * 2800);
    }

    #[test]
    fn quick_tools_never_flash() {
        let mut c = cycler(CyclerVariant::Fade);
        c.update(&AgentState::Thinking { header: None }, ClockMs(0), false);
        c.update(&tool("a.rs"), ClockMs(1000), false);
        c.update(&tool("a.rs"), ClockMs(1200), false);
        assert_eq!(c.view(ClockMs(1200), false).unwrap().text, "Thinking");
        c.update(&AgentState::Thinking { header: None }, ClockMs(1250), false);
        c.update(&AgentState::Thinking { header: None }, ClockMs(1600), false);
        assert_eq!(c.view(ClockMs(1600), false).unwrap().text, "Thinking");
        // A tool that lasts shows after 300 ms and holds ≥ 900 ms.
        c.update(&tool("b.rs"), ClockMs(2000), false);
        c.update(&tool("b.rs"), ClockMs(2300), false);
        assert!(c.view(ClockMs(2300), false).unwrap().text.ends_with("b.rs"));
        c.update(&AgentState::Thinking { header: None }, ClockMs(2400), false);
        assert!(c.view(ClockMs(2400), false).unwrap().text.ends_with("b.rs"));
        c.update(&AgentState::Thinking { header: None }, ClockMs(3300), false);
        assert_eq!(c.view(ClockMs(3300), false).unwrap().text, "Thinking");
    }

    #[test]
    fn static_never_cycles_and_done_settles() {
        let mut c = cycler(CyclerVariant::Static);
        let th = AgentState::Thinking { header: None };
        for t in (0..20_000).step_by(100) {
            c.update(&th, ClockMs(t), false);
        }
        assert_eq!(c.view(ClockMs(20_000), false).unwrap().text, "Thinking");
        let done = AgentState::Done {
            elapsed: Duration::from_secs(72),
            waited: Duration::ZERO,
        };
        c.update(&done, ClockMs(20_100), false);
        let v = c.view(ClockMs(20_100), false).unwrap();
        assert_eq!(v.text, "Worked for 1m 12s");
        assert!(v.settled);
    }

    #[test]
    fn swap_progress_and_ellipsis() {
        let mut c = cycler(CyclerVariant::Ellipsis);
        c.update(&AgentState::Thinking { header: None }, ClockMs(0), false);
        c.update(&tool("a.rs"), ClockMs(1000), false);
        c.update(&tool("a.rs"), ClockMs(1300), false);
        let v = c.view(ClockMs(1340), false).unwrap();
        assert_eq!(v.outgoing.as_deref(), Some("Thinking"));
        assert!(v.in_t == 0.0 && v.out_t > 0.0);
        assert!(v.dots.iter().all(|d| *d >= 0.3));
        let v = c.view(ClockMs(1500), false).unwrap();
        assert!(v.outgoing.is_none() && v.in_t >= 0.99);
    }
}
